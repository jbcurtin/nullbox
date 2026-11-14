//! Request extractors. `Actor` is the auth seam.

use axum::Json;
use axum::extract::{FromRequest, FromRequestParts, Path, Query};
use axum::http::request::Parts;
use axum::http::{HeaderMap, header};

use crate::{ApiError, AppState};

pub const ACTOR_HEADER: &str = "x-actor-name";
pub const ADMIN_TOKEN_HEADER: &str = "x-admin-token";
pub const TICKET_TOKEN_HEADER: &str = "x-ticket-token";

/// The logged-in worker making the request.
///
/// Requires `Authorization: Bearer <token>` from `POST /api/auth/login`; without a live
/// session every worker endpoint answers 401 `unauthenticated`. `name` is the account's
/// username, so replies and notes always carry the account that wrote them. (The old
/// `X-Actor-Name` header is ignored.) To add SSO/JWT, replace only this extractor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    pub name: String,
}

/// The token from `Authorization: Bearer <token>`, if the header has one.
pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    let token = token.trim();
    (scheme.eq_ignore_ascii_case("bearer") && !token.is_empty()).then_some(token)
}

impl FromRequestParts<AppState> for Actor {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let session = bearer_token(&parts.headers).and_then(|token| state.accounts.session(token)).ok_or_else(|| {
            tracing::info!(target: "audit", path = %parts.uri.path(), "refused worker request without a valid session");
            ApiError::Unauthenticated
        })?;
        Ok(Actor { name: session.username })
    }
}

/// The customer's secret, from `X-Ticket-Token`. The frontend reads it from the URL fragment.
pub struct TicketToken(pub String);

impl<S: Send + Sync> FromRequestParts<S> for TicketToken {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get(TICKET_TOKEN_HEADER)
            .and_then(|v| v.to_str().ok())
            .filter(|v| !v.is_empty())
            .map(|v| TicketToken(v.to_owned()))
            .ok_or(ApiError::MissingTicketToken)
    }
}

/// `Json`, `Path` and `Query` with rejections in our `{ code, message }` shape.
#[derive(FromRequest)]
#[from_request(via(Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);

#[derive(FromRequestParts)]
#[from_request(via(Path), rejection(ApiError))]
pub struct ApiPath<T>(pub T);

#[derive(FromRequestParts)]
#[from_request(via(Query), rejection(ApiError))]
pub struct ApiQuery<T>(pub T);
