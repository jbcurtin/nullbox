//! Worker sign-up, login and logout. See `crate::auth` for the rules.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};

use super::extract::{ApiJson, bearer_token};
use crate::dto::{AuthOptions, LoginRequest, LoginResponse, SignupRequest, SignupResponse};
use crate::passwords::MIN_PASSWORD_CHARS;
use crate::{ApiError, AppState};

pub async fn options(State(state): State<AppState>) -> Json<AuthOptions> {
    Json(AuthOptions {
        signup_code_required: state.accounts.signup_code_required(),
        min_password_length: MIN_PASSWORD_CHARS as u32,
    })
}

pub async fn signup(
    State(state): State<AppState>,
    ApiJson(req): ApiJson<SignupRequest>,
) -> Result<(StatusCode, Json<SignupResponse>), ApiError> {
    let result = state.accounts.sign_up(&req.username, &req.password, req.signup_code.as_deref()).await;
    if matches!(result, Err(ApiError::InvalidSignupCode)) {
        // Slows down guessing the code a little. Real rate limiting belongs in `middleware()`.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    let username = result?;
    Ok((StatusCode::CREATED, Json(SignupResponse { username })))
}

pub async fn login(
    State(state): State<AppState>,
    ApiJson(req): ApiJson<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let (token, session) = match state.accounts.log_in(&req.username, &req.password).await {
        Ok(ok) => ok,
        Err(err) => {
            // Slows down password guessing a little. Real rate limiting belongs in `middleware()`.
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            return Err(err);
        }
    };
    Ok(Json(LoginResponse { token, username: session.username, expires_at: session.expires_at }))
}

/// Ends the session, which spends the account. Always succeeds, so it's safe with a stale token.
pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> StatusCode {
    if let Some(token) = bearer_token(&headers) {
        state.accounts.log_out(token);
    }
    StatusCode::NO_CONTENT
}
