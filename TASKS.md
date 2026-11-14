# Level Up! tasks

Osaka FastHacks #2. Live site: https://team-07.fasthacks.cloud · Score: **1 / 26** (4 / 26 once the three Security checks pass)

Do not put secrets in this file. The repo is public. Stripe and OTLP keys go in the server's `~/nullbox/.env`.
Checked by: 🤖 HOT (automatic, when you press Check) · 🧑‍⚖️ a judge (comes to your table)

## ✅ Deploy (1 pt) 🤖

- [x] `/` loads and `/api/healthz` returns `{"status":"ok"}` (Docker Compose + Caddy, see `compose.yaml`)

## CI/CD (4 pts) 🤖

- [ ] **Automatic deployments (2)**: HOT commits a code to `apps/web/public/level-up.txt` on the default branch. Within 5 minutes, `/level-up.txt` must serve it, with no human action.
  - Built 2026-11-14 in `.github/workflows/cicd.yml`. A push to `main` runs `test` and `build` in parallel, then `deploy`: the images are built in Actions, sent to the server with `docker save | ssh docker load`, and started with `docker compose up -d --no-build`
  - Secret `DEPLOY_SSH_KEY`; variables `DEPLOY_HOST=team-07.fasthacks.cloud`, `DEPLOY_USER=deploy`
  - `origin` is `jbcurtin/nullbox` and we have push access, so no fork was needed
- [ ] **Fast builds with a cache (1)**: every Actions run for HOT's commit succeeds within 4 minutes, and the code goes live.
  - The images build in Actions with a buildx layer cache (`type=gha`), and the API image uses cargo-chef so compiled dependencies stay cached. No registry: the images go to the server over SSH
  - Measured on a branch: 5 min with every cache empty, 51 s for a change to `apps/web/public` with warm caches. The first run on `main` is a cold one, because caches are per branch
- [ ] **Tests run automatically (1)**: HOT pushes a failing Rust test and a failing frontend test. The pipeline must fail and keep that version off the site. After HOT removes the tests, the commit must go live within 5 minutes.
  - `cargo test` and `pnpm test` run in the `test` job, which the `deploy` job `needs:`

## Login (5 pts) 🤖 🧑‍⚖️

- [ ] **Members-only page stays closed (1)** 🤖: a customer can still read their ticket with its token. The worker API refuses requests with no login, with only `X-Actor-Name`, and with a made-up token.
  - Built by a teammate (commit `0e2fbbe`), live since 2026-11-14. Checked on the live site: the worker API answers 401 with no login, with only `X-Actor-Name`, and with a made-up token; after `POST /api/auth/login` it answers 200
  - The shared `WORKER_USERNAME`/`WORKER_PASSWORD` login was replaced by per-person accounts (see below). After that deploy, the 401 checks still hold. Still to do: press Check
- [ ] **Username and password (2)** 🧑‍⚖️: sign up, log in, reply under your own account name, locked out again after logging out. Short passwords and leaked passwords are refused at sign-up.
  - Built, **not deployed yet**: sign-up on `/inbox` with your own username and password (12+ characters, Argon2id, refused if in Pwned Passwords or a common-password list); replies carry the account's username
  - Each account logs in **once**: after logging out, that account is refused and you create a new one. Sessions are in memory, so a deploy logs everyone out
  - Optional `WORKER_SIGNUP_CODE` in the server `.env` requires a code at sign-up. Remove `WORKER_USERNAME`/`WORKER_PASSWORD` from it; they're no longer read
- [ ] **Single sign-on (2)** 🧑‍⚖️: sign in through a provider, reply under that account name, locked out after logging out. The judge also reviews how sign-in is built.

## Security (6 pts) 🤖 🧑‍⚖️

- [x] **Rotate the leaked secret (1)** 🤖: the old admin token is refused, and a new token returns an export containing HOT's ticket. Then rotate once more: the first new token must be refused too.
  - Done 2026-11-14. Round 1: the server's random `ADMIN_TOKEN` (set at deploy) was entered on the card. Round 2: a new token was generated and the API restarted. The leaked token and the round-1 token both get 403; the current one gets 200
  - To rotate again (about 10 s, no rebuild):
    ```sh
    ssh team-07 'cd ~/nullbox && sed -i "s/^ADMIN_TOKEN=.*/ADMIN_TOKEN=$(openssl rand -hex 32)/" .env && docker compose up -d api && grep ^ADMIN_TOKEN= .env'
    ```
- [x] **The secret out of git (1)** 🤖: the leaked token is in no file of the default branch, and gone from its history: the `.env` of commit `4351700` and the default in `crates/api/src/config.rs`.
  - Done 2026-11-14. `config.rs` has no default anymore: the API refuses to start without an `ADMIN_TOKEN` of at least 32 characters (the README quickstart and the dev container generate one)
  - History rewritten with `git filter-repo --invert-paths --path .env --replace-text` (token → `***REMOVED***`), then force-pushed to `main`. The two `.env` commits dropped out. Every commit ID changed, so teammates need `git fetch && git reset --hard origin/main` or a fresh clone
- [x] **Customer links (1)** 🧑‍⚖️: changing the ticket ID in a customer link no longer shows or answers someone else's ticket.
  - Done 2026-11-14, live. The bug: `ticket_for_customer` (`crates/api/src/service.rs`) accepted a token that belonged to *any* ticket, then loaded the ticket named in the link
  - Now the token must belong to that ticket; otherwise the answer is 404. Test: `a_ticket_token_opens_only_its_own_ticket`. Checked on the live site: own link 200, another ticket's ID 404 for both reading and replying
- [x] **Firewall (1)** 🤖: HTTPS answers on 443, and no port other than 22, 80 and 443 is open.
  - Done 2026-11-14. `ufw` was inactive and a `debug-console` systemd service (`python3 -m http.server 8888`) served `/opt/debug-console` to the internet
  - `ufw` now denies all incoming traffic except 22/tcp, 80/tcp, 443/tcp and 443/udp (IPv4 and IPv6), and is enabled on boot. An outside scan shows 22, 80 and 443 open and 8888 closed
  - The debug console is also stopped and disabled (`sudo systemctl disable --now debug-console`), so nothing listens on 8888 anymore
- [ ] **Audit log (2)** 🤖 then 🧑‍⚖️: HOT opens a ticket with a code in its subject and calls the worker API with a made-up token. A judge then looks for both in the audit log.
  - Start in `EventSink` (`crates/api/src/events.rs`), plus refused requests in the `Actor` extractor

## Observability (2 pts) 🤖

- [ ] HOT opens a ticket with a code in its subject and its own trace ID. Within 5 minutes, telemetry showing that code or trace ID must reach HOT.
  - Built 2026-11-14 in `crates/api/src/telemetry.rs`, live and exporting since 2026-11-14. Traces and logs go out over OTLP (http/protobuf). Each request is a span that continues the incoming `traceparent`, and creating a ticket puts its subject on the span and in a log line
  - Tested against a local fake collector: the trace and the log both carried the caller's trace ID and the subject
  - The 3 `OTEL_EXPORTER_OTLP_*` values from the platform page are in the server `.env`, and `compose.yaml` passes them through. The API logs "OpenTelemetry is on" and no export errors. Still to do: press Check

## SEO (2 pts) 🤖

- [ ] **Score 100 (1)**: PageSpeed Insights gives the home page 100 for SEO on mobile (99 fails).
  - A teammate added a title, a meta description, `robots.txt` and `llms.txt` (commit `5a1ebbf`), live since 2026-11-14. The PageSpeed score has not been measured here. Still to do: press Check
- [ ] **Agentic Browsing 100 (1)**: PageSpeed Insights gives the home page 100 for Agentic Browsing on mobile.

## Payments (3 pts) 🤖

- [ ] Stripe has a paid checkout for **"Nothing to See Here"**, opened by our app, that brings the buyer back to the live site. A purchase made before pressing Check counts too.
  - Built 2026-11-14, live. `POST /api/checkout` (`crates/api/src/payments.rs`) finds the product named "Nothing to See Here" in the Stripe account and creates a Checkout Session for its price (a subscription, ¥100 a day), and the shop page at `/shop` sends the buyer to it. Stripe brings them back to `/shop/thanks`
  - The first version sold its own ¥500 item and Stripe refused it: the account has Managed Payments on, which needs a product tax code. The account's own product already has one
  - `STRIPE_API_KEY` is in the server `.env` (2026-11-14). A test purchase went through from `/shop` on the live site and came back to `/shop/thanks` (2026-11-14). Still to do: press Check
  - Test card: 4242 4242 4242 4242, 12/34, CVC 123, Japan, leave "Save my information" unticked

## AI (3 pts) 🤖 🧑‍⚖️

- [ ] **MCP server (2)** 🤖: `POST /mcp` without credentials returns 401 (or 403). With a bearer token we give HOT, an MCP client (protocol 2026-07-28, or 2025-11-25 with initialize and a session) lists at least one tool. OAuth is optional.
  - Built 2026-11-14 in `crates/api/src/http/mcp.rs`, live. Checked on the live site: 401 with no token and with a made-up one; with the token, `initialize` (2025-11-25) returns a session and `tools/list` (2026-07-28) lists five tools. Caddy proxies `/mcp` to the API
  - Still to do: give HOT the token and press Check. Read it with `ssh team-07 'grep ^MCP_TOKEN= ~/nullbox/.env'`
- [ ] **An agent works a ticket (1)** 🧑‍⚖️: given only a ticket ID, the judge's agent answers the customer through our MCP server, and the reply shows up on the ticket.
  - Tools: `get_ticket`, `reply_to_ticket`, `add_internal_note`, `update_ticket`, `list_tickets`. The ticket ID can be the UUID or the number. The reply is signed "AI agent". Checked on the live site: a reply sent through MCP showed up on the customer's ticket page

## Suggested order

1. Quick wins: firewall, rotate secret, secret out of git (3 pts, under 30 min)
2. CI/CD with tests and cache (4 pts). It makes every later deploy free
3. SEO (2), Login: members-only (1), Observability (2)
4. Bigger items: MCP (3), Payments (3), passwords and SSO (4), audit log (2)
