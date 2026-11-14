//! Worker login and logout. See `crate::auth` for how sessions work.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};

use super::extract::{ApiJson, bearer_token};
use crate::dto::{LoginRequest, LoginResponse};
use crate::{ApiError, AppState};

pub async fn login(
    State(state): State<AppState>,
    ApiJson(req): ApiJson<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let result = state.sessions.login(state.config.worker_credentials.as_ref(), req.username.trim(), &req.password);
    let (token, session) = match result {
        Ok(ok) => ok,
        Err(err) => {
            // Slows down password guessing a little. Real rate limiting belongs in `middleware()`.
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            return Err(err);
        }
    };
    Ok(Json(LoginResponse { token, username: session.username, expires_at: session.expires_at }))
}

/// Ends the session. Always succeeds, so it's safe to call with a stale token.
pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> StatusCode {
    if let Some(token) = bearer_token(&headers) {
        state.sessions.logout(token);
    }
    StatusCode::NO_CONTENT
}
