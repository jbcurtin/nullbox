//! HTTP tests: the whole router against in-memory SQLite, via `oneshot`.
//! Copy a test, change the requests, done.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::Arc;

use nullbox_api::passwords::OfflineLeakCheck;
use nullbox_api::{AppState, Config, db, http};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn app() -> Router {
    app_with_signup_code(None).await
}

/// The router on a fresh in-memory database. The leaked-password check is the offline
/// list only, so tests never touch the network.
async fn app_with_signup_code(signup_code: Option<&str>) -> Router {
    let pool = db::connect_in_memory().await.expect("in-memory db");
    let config = Config {
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".into(),
        cors_allowed_origins: vec!["http://localhost:5173".into()],
        public_web_url: "http://localhost:5173".into(),
        admin_token: "test-admin-token".into(),
        worker_signup_code: signup_code.map(Into::into),
        mcp_token: Some(MCP_TOKEN.into()),
        stripe_api_key: None,
        stripe_api_base: "http://127.0.0.1:1".into(),
    };
    http::router(AppState::with_leak_check(pool, config, Arc::new(OfflineLeakCheck)))
}

/// Send a request; returns status and JSON body (`Null` if empty).
async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    headers: &[(&str, &str)],
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        req = req.header(*name, *value);
    }
    let body = match body {
        Some(json) => {
            req = req.header("content-type", "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };
    let res = app.clone().oneshot(req.body(body).unwrap()).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

const PASSWORD: &str = "violet otter gravel 42";

async fn sign_up(app: &Router, username: &str) {
    let (status, body) =
        send(app, "POST", "/api/auth/signup", &[], Some(json!({ "username": username, "password": PASSWORD }))).await;
    assert_eq!((status, &body["username"]), (StatusCode::CREATED, &json!(username)));
}

async fn try_login(app: &Router, username: &str, password: &str) -> (StatusCode, Value) {
    send(app, "POST", "/api/auth/login", &[], Some(json!({ "username": username, "password": password }))).await
}

/// Creates a fresh account and uses its one login; returns the `Authorization` header value.
async fn login(app: &Router, username: &str) -> String {
    sign_up(app, username).await;
    let (status, body) = try_login(app, username, PASSWORD).await;
    assert_eq!(status, StatusCode::OK);
    format!("Bearer {}", body["token"].as_str().unwrap())
}
const MCP_TOKEN: &str = "test-mcp-token";

#[tokio::test]
async fn customer_and_worker_round_trip() {
    let app = app().await;
    let auth = login(&app, "山田太郎").await;
    // The old display-name header is ignored: replies carry the account's username.
    let worker = [("authorization", auth.as_str()), ("x-actor-name", "someone-else")];

    // Anonymous customer creates a ticket.
    let (status, created) = send(
        &app,
        "POST",
        "/api/tickets",
        &[],
        Some(json!({ "subject": "Printer on fire", "body": "Again.", "severity": "p1" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["ticket"]["id"].as_str().unwrap().to_owned();
    let token = created["access_token"].as_str().unwrap().to_owned();
    assert_eq!(created["ticket"]["number"], 1);
    assert!(created["ticket"]["anon"]["animal"].is_string());
    assert!(created["access_link"].as_str().unwrap().ends_with(&format!("/t/{id}#{token}")));

    // The secret link works; a wrong token looks like a missing ticket.
    let (status, thread) =
        send(&app, "GET", &format!("/api/public/tickets/{id}"), &[("x-ticket-token", &token)], None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(thread["ticket"]["subject"], "Printer on fire");
    let (status, error) =
        send(&app, "GET", &format!("/api/public/tickets/{id}"), &[("x-ticket-token", "nope")], None).await;
    assert_eq!((status, &error["code"]), (StatusCode::NOT_FOUND, &json!("not_found")));

    // The worker sees it in the inbox, notes internally and yeets it.
    let (_, inbox) = send(&app, "GET", "/api/tickets", &worker, None).await;
    assert_eq!(inbox.as_array().unwrap().len(), 1);
    let (status, _) = send(
        &app,
        "POST",
        &format!("/api/tickets/{id}/messages"),
        &worker,
        Some(json!({ "body": "it's always DNS", "internal": true })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, yeeted) = send(&app, "POST", &format!("/api/tickets/{id}/yeet"), &worker, None).await;
    assert_eq!((status, &yeeted["status"]), (StatusCode::OK, &json!("resolved")));

    let (_, stats) = send(&app, "GET", "/api/stats/backlog", &worker, None).await;
    assert_eq!(stats, json!({ "open_count": 0, "beast_mood": "asleep", "oldest_open_age_secs": null }));

    // The customer never sees internal notes, and replying reopens the ticket.
    let (status, _) = send(
        &app,
        "POST",
        &format!("/api/public/tickets/{id}/messages"),
        &[("x-ticket-token", &token)],
        Some(json!({ "body": "Still on fire" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (_, thread) =
        send(&app, "GET", &format!("/api/public/tickets/{id}"), &[("x-ticket-token", &token)], None).await;
    assert_eq!(thread["ticket"]["status"], "open");
    let messages = thread["messages"].as_array().unwrap();
    assert!(messages.iter().all(|m| m["internal"] == false));
    assert!(
        messages
            .iter()
            .any(|m| m["system_event"] == json!({ "kind": "status_changed", "from": "resolved", "to": "open" }))
    );

    // The worker view does include the note, written by the 山田太郎 account.
    let (_, thread) = send(&app, "GET", &format!("/api/tickets/{id}"), &worker, None).await;
    let note = thread["messages"].as_array().unwrap().iter().find(|m| m["internal"] == true).unwrap();
    assert_eq!(note["author_name"], "山田太郎");
}

#[tokio::test]
async fn errors_share_one_shape() {
    let app = app().await;
    let auth = login(&app, "kenji").await;

    let (status, error) = send(&app, "GET", "/api/tickets", &[], None).await;
    assert_eq!((status, &error["code"]), (StatusCode::UNAUTHORIZED, &json!("unauthenticated")));

    let (status, error) =
        send(&app, "POST", "/api/tickets", &[], Some(json!({ "subject": " ", "body": "x", "severity": "p4" }))).await;
    assert_eq!((status, &error["code"]), (StatusCode::UNPROCESSABLE_ENTITY, &json!("validation_failed")));

    let (status, error) = send(&app, "GET", "/api/tickets/not-a-uuid", &[("authorization", auth.as_str())], None).await;
    assert_eq!((status, &error["code"]), (StatusCode::NOT_FOUND, &json!("not_found")));

    let (status, error) = send(&app, "GET", "/api/teapot", &[], None).await;
    assert_eq!((status, &error["code"]), (StatusCode::IM_A_TEAPOT, &json!("im_a_teapot")));
}

#[tokio::test]
async fn admin_export_needs_the_admin_token() {
    let app = app().await;
    let ticket = json!({ "subject": "Export me", "body": "x", "severity": "p3", "customer_email": "a@example.com" });
    send(&app, "POST", "/api/tickets", &[], Some(ticket)).await;

    let (status, error) = send(&app, "GET", "/api/admin/export", &[], None).await;
    assert_eq!((status, &error["code"]), (StatusCode::FORBIDDEN, &json!("forbidden")));
    let (status, _) = send(&app, "GET", "/api/admin/export", &[("x-admin-token", "nope")], None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let admin = [("x-admin-token", "test-admin-token")];
    let (status, tickets) = send(&app, "GET", "/api/admin/export", &admin, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tickets[0]["customer_email"], "a@example.com");
}

#[tokio::test]
async fn worker_endpoints_need_a_login() {
    let app = app().await;
    let name_only = ("x-actor-name", "%E5%B1%B1%E7%94%B0"); // 山田, percent-encoded

    // No session, a name header alone, or a made-up token: all refused, on every worker route.
    let id = "01a1d8b8-eddc-7267-aed2-2057c3b4f227";
    for (method, uri) in [
        ("GET", "/api/tickets".to_owned()),
        ("GET", format!("/api/tickets/{id}")),
        ("POST", format!("/api/tickets/{id}/messages")),
        ("PATCH", format!("/api/tickets/{id}")),
        ("POST", format!("/api/tickets/{id}/yeet")),
        ("GET", "/api/stats/backlog".to_owned()),
    ] {
        for headers in [vec![], vec![name_only], vec![("authorization", "Bearer made-up"), name_only]] {
            let (status, error) = send(&app, method, &uri, &headers, Some(json!({}))).await;
            assert_eq!(
                (status, &error["code"]),
                (StatusCode::UNAUTHORIZED, &json!("unauthenticated")),
                "{method} {uri}"
            );
        }
    }
}

#[tokio::test]
async fn each_account_logs_in_once() {
    let app = app().await;
    sign_up(&app, "Aiko").await;

    // Wrong password or unknown user: the same answer.
    for (username, password) in [("Aiko", "wrong password"), ("nobody", PASSWORD), ("", "")] {
        let (status, error) = try_login(&app, username, password).await;
        assert_eq!((status, &error["code"]), (StatusCode::UNAUTHORIZED, &json!("invalid_credentials")));
    }

    // The one login (usernames are case-insensitive), and a reply under the account's name.
    let (status, body) = try_login(&app, "aiko", PASSWORD).await;
    assert_eq!((status, &body["username"]), (StatusCode::OK, &json!("Aiko")));
    let auth = format!("Bearer {}", body["token"].as_str().unwrap());
    let session = [("authorization", auth.as_str())];
    let ticket = json!({ "subject": "Help", "body": "x", "severity": "p3" });
    let (_, created) = send(&app, "POST", "/api/tickets", &[], Some(ticket)).await;
    let id = created["ticket"]["id"].as_str().unwrap();
    let (status, message) =
        send(&app, "POST", &format!("/api/tickets/{id}/messages"), &session, Some(json!({ "body": "On it" }))).await;
    assert_eq!((status, &message["author_name"]), (StatusCode::CREATED, &json!("Aiko")));

    // A second login is refused, even while the first session is still open...
    let (status, error) = try_login(&app, "Aiko", PASSWORD).await;
    assert_eq!((status, &error["code"]), (StatusCode::FORBIDDEN, &json!("account_already_used")));

    // ...and after logging out, the old token is dead and the account stays spent.
    let (status, _) = send(&app, "POST", "/api/auth/logout", &session, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, error) = send(&app, "GET", "/api/tickets", &session, None).await;
    assert_eq!((status, &error["code"]), (StatusCode::UNAUTHORIZED, &json!("unauthenticated")));
    let (status, error) = try_login(&app, "Aiko", PASSWORD).await;
    assert_eq!((status, &error["code"]), (StatusCode::FORBIDDEN, &json!("account_already_used")));

    // A wrong password on a spent account says "wrong password", not "spent".
    let (_, error) = try_login(&app, "Aiko", "wrong password").await;
    assert_eq!(error["code"], "invalid_credentials");

    // A new account works.
    login(&app, "Aiko2").await;
}

#[tokio::test]
async fn sign_up_rules() {
    let app = app().await;
    let signup = |username: &str, password: &str| {
        let body = json!({ "username": username, "password": password });
        let app = app.clone();
        async move { send(&app, "POST", "/api/auth/signup", &[], Some(body)).await }
    };

    let (status, error) = signup("kenji", "short").await;
    assert_eq!((status, &error["code"]), (StatusCode::UNPROCESSABLE_ENTITY, &json!("password_too_short")));
    let (status, error) = signup("kenji", "Password1234").await;
    assert_eq!((status, &error["code"]), (StatusCode::UNPROCESSABLE_ENTITY, &json!("password_leaked")));
    let (status, error) = signup("no spaces allowed", PASSWORD).await;
    assert_eq!((status, &error["code"]), (StatusCode::UNPROCESSABLE_ENTITY, &json!("validation_failed")));

    let (status, _) = signup("kenji", PASSWORD).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, error) = signup("KENJI", "another good password").await;
    assert_eq!((status, &error["code"]), (StatusCode::CONFLICT, &json!("username_taken")));

    let (_, options) = send(&app, "GET", "/api/auth/options", &[], None).await;
    assert_eq!(options, json!({ "signup_code_required": false, "min_password_length": 12 }));
}

#[tokio::test]
async fn sign_up_code_when_configured() {
    let app = app_with_signup_code(Some("team-07")).await;
    let (_, options) = send(&app, "GET", "/api/auth/options", &[], None).await;
    assert_eq!(options["signup_code_required"], true);

    for code in [None, Some("wrong")] {
        let body = json!({ "username": "kenji", "password": PASSWORD, "signup_code": code });
        let (status, error) = send(&app, "POST", "/api/auth/signup", &[], Some(body)).await;
        assert_eq!((status, &error["code"]), (StatusCode::FORBIDDEN, &json!("invalid_signup_code")));
    }
    let body = json!({ "username": "kenji", "password": PASSWORD, "signup_code": "team-07" });
    let (status, _) = send(&app, "POST", "/api/auth/signup", &[], Some(body)).await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn a_ticket_token_opens_only_its_own_ticket() {
    let app = app().await;
    let mut tickets = Vec::new();
    for subject in ["Mine", "Someone else's"] {
        let body = json!({ "subject": subject, "body": "x", "severity": "p3" });
        let (_, created) = send(&app, "POST", "/api/tickets", &[], Some(body)).await;
        tickets.push((
            created["ticket"]["id"].as_str().unwrap().to_owned(),
            created["access_token"].as_str().unwrap().to_owned(),
        ));
    }
    let (my_id, my_token) = &tickets[0];
    let (other_id, _) = &tickets[1];
    let mine = [("x-ticket-token", my_token.as_str())];

    // Changing the ticket ID in the link shows nothing and answers nothing.
    let (status, error) = send(&app, "GET", &format!("/api/public/tickets/{other_id}"), &mine, None).await;
    assert_eq!((status, &error["code"]), (StatusCode::NOT_FOUND, &json!("not_found")));
    let reply = Some(json!({ "body": "Not my ticket" }));
    let (status, _) = send(&app, "POST", &format!("/api/public/tickets/{other_id}/messages"), &mine, reply).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let auth = login(&app, "kenji").await;
    let worker = [("authorization", auth.as_str())];
    let (_, thread) = send(&app, "GET", &format!("/api/tickets/{other_id}"), &worker, None).await;
    assert_eq!(thread["messages"].as_array().unwrap().len(), 0);

    // The token still opens its own ticket.
    let (status, thread) = send(&app, "GET", &format!("/api/public/tickets/{my_id}"), &mine, None).await;
    assert_eq!((status, &thread["ticket"]["subject"]), (StatusCode::OK, &json!("Mine")));
}

/// One MCP request with the bearer token; returns status, headers and JSON body.
async fn mcp(app: &Router, headers: &[(&str, &str)], body: Value) -> (StatusCode, axum::http::HeaderMap, Value) {
    let mut req = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("authorization", format!("Bearer {MCP_TOKEN}"))
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream");
    for (name, value) in headers {
        req = req.header(*name, *value);
    }
    let res = app.clone().oneshot(req.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let (status, headers) = (res.status(), res.headers().clone());
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, headers, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
async fn mcp_needs_the_bearer_token() {
    let app = app().await;
    let list = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" });

    let (status, _) = send(&app, "POST", "/mcp", &[], Some(list.clone())).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = send(&app, "POST", "/mcp", &[("authorization", "Bearer made-up")], Some(list.clone())).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    // A worker name is not a login here either.
    let (status, _) = send(&app, "POST", "/mcp", &[("x-actor-name", "kenji")], Some(list)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn mcp_2025_handshake_then_an_agent_answers_a_ticket() {
    let app = app().await;
    let ticket = json!({ "subject": "Printer on fire", "body": "Again.", "severity": "p1" });
    let (_, created) = send(&app, "POST", "/api/tickets", &[], Some(ticket)).await;
    let id = created["ticket"]["id"].as_str().unwrap().to_owned();
    let token = created["access_token"].as_str().unwrap().to_owned();

    // initialize hands out a session and agrees on the version.
    let init = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": { "name": "test", "version": "0" } } });
    let (status, headers, body) = mcp(&app, &[], init).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"]["protocolVersion"], "2025-11-25");
    assert!(body["result"]["capabilities"]["tools"].is_object());
    let session = headers["mcp-session-id"].to_str().unwrap().to_owned();
    let session = [("mcp-session-id", session.as_str()), ("mcp-protocol-version", "2025-11-25")];

    let (status, _, body) =
        mcp(&app, &session, json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })).await;
    assert_eq!((status, body), (StatusCode::ACCEPTED, Value::Null));

    let (_, _, body) = mcp(&app, &session, json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" })).await;
    let tools = body["result"]["tools"].as_array().unwrap();
    assert!(tools.iter().any(|tool| tool["name"] == "reply_to_ticket"));
    assert!(tools.iter().all(|tool| tool["inputSchema"]["type"] == "object"));
    assert!(body["result"].get("resultType").is_none());

    // Given only the ticket ID: read it, then answer the customer.
    let call = |id: u32, name: &str, arguments: Value| json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": name, "arguments": arguments } });
    let (_, _, body) = mcp(&app, &session, call(3, "get_ticket", json!({ "ticket_id": id }))).await;
    assert_eq!(body["result"]["isError"], false);
    assert!(body["result"]["content"][0]["text"].as_str().unwrap().contains("Printer on fire"));
    let reply = json!({ "ticket_id": "#1", "body": "Have you tried water?" });
    let (_, _, body) = mcp(&app, &session, call(4, "reply_to_ticket", reply)).await;
    assert_eq!(body["result"]["isError"], false);

    // The reply shows up on the customer's ticket.
    let (_, thread) =
        send(&app, "GET", &format!("/api/public/tickets/{id}"), &[("x-ticket-token", &token)], None).await;
    let message = &thread["messages"][0];
    assert_eq!((&message["body"], &message["author_kind"]), (&json!("Have you tried water?"), &json!("worker")));

    // A tool that fails says so in its result; an unknown tool is a protocol error.
    let missing = json!({ "ticket_id": "999" });
    let (status, _, body) = mcp(&app, &session, call(5, "get_ticket", missing)).await;
    assert_eq!((status, &body["result"]["isError"]), (StatusCode::OK, &json!(true)));
    let (_, _, body) = mcp(&app, &session, call(6, "nope", json!({}))).await;
    assert_eq!(body["error"]["code"], -32602);
}

#[tokio::test]
async fn mcp_2026_is_stateless() {
    let app = app().await;
    let meta = json!({
        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientCapabilities": {},
    });
    let version = ("mcp-protocol-version", "2026-07-28");

    let discover = json!({ "jsonrpc": "2.0", "id": "d", "method": "server/discover", "params": { "_meta": meta } });
    let (status, headers, body) = mcp(&app, &[version, ("mcp-method", "server/discover")], discover).await;
    assert_eq!(status, StatusCode::OK);
    assert!(headers.get("mcp-session-id").is_none());
    assert_eq!(body["result"]["supportedVersions"], json!(["2026-07-28"]));
    assert_eq!(body["result"]["resultType"], "complete");

    // No handshake: tools/list works as the first request.
    let list = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": { "_meta": meta } });
    let (status, _, body) = mcp(&app, &[version, ("mcp-method", "tools/list")], list.clone()).await;
    assert_eq!(status, StatusCode::OK);
    let result = &body["result"];
    assert!(!result["tools"].as_array().unwrap().is_empty());
    assert_eq!((&result["resultType"], &result["cacheScope"]), (&json!("complete"), &json!("public")));
    assert!(result["ttlMs"].is_number());
    assert_eq!(result["_meta"]["io.modelcontextprotocol/serverInfo"]["name"], "nullbox");

    // Headers must agree with the body; unknown versions and methods are refused.
    let (status, _, body) = mcp(&app, &[version, ("mcp-method", "tools/call")], list).await;
    assert_eq!((status, &body["error"]["code"]), (StatusCode::BAD_REQUEST, &json!(-32020)));
    let old = json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": { "_meta": {
        "io.modelcontextprotocol/protocolVersion": "1900-01-01" } } });
    let (status, _, body) = mcp(&app, &[], old).await;
    assert_eq!((status, &body["error"]["code"]), (StatusCode::BAD_REQUEST, &json!(-32022)));
    assert_eq!(body["error"]["data"]["requested"], "1900-01-01");
    let unknown = json!({ "jsonrpc": "2.0", "id": 3, "method": "resources/list", "params": { "_meta": meta } });
    let (status, _, body) = mcp(&app, &[version], unknown).await;
    assert_eq!((status, &body["error"]["code"]), (StatusCode::NOT_FOUND, &json!(-32601)));
}

#[tokio::test]
async fn checkout_without_a_stripe_key_is_unavailable() {
    let app = app().await;
    let (status, error) = send(&app, "POST", "/api/checkout", &[], None).await;
    assert_eq!((status, &error["code"]), (StatusCode::SERVICE_UNAVAILABLE, &json!("service_unavailable")));
    // The reason is logged, not sent.
    assert!(!error["message"].as_str().unwrap().contains("STRIPE"));
}
