//! Worker login: one username and password from the config, and in-memory sessions.
//!
//! `POST /api/auth/login` checks the credentials and returns a random bearer token.
//! Only the token's SHA-256 is kept, next to the username and an expiry. Sessions live
//! in memory, so a restart logs everyone out. For real accounts, swap the credential
//! check for a users table with password hashes and keep the rest.

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};
use sha2::{Digest, Sha256};

use crate::{ApiError, token};

/// How long a login lasts.
pub const SESSION_TTL: Duration = Duration::hours(12);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub username: String,
    pub expires_at: DateTime<Utc>,
}

/// The configured worker credentials. `None` when unset: then nobody can log in.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Default)]
pub struct Sessions {
    by_token_hash: Mutex<HashMap<String, Session>>,
}

impl Sessions {
    /// Checks the credentials and opens a session. Returns the token (shown once) and the session.
    pub fn login(
        &self,
        configured: Option<&Credentials>,
        username: &str,
        password: &str,
    ) -> Result<(String, Session), ApiError> {
        let Some(configured) = configured else {
            tracing::warn!("worker login attempted, but WORKER_USERNAME/WORKER_PASSWORD are not set");
            return Err(ApiError::InvalidCredentials);
        };
        // Compare both fields every time, in constant time, so timing reveals neither.
        let user_ok = constant_time_eq(username, &configured.username);
        let pass_ok = constant_time_eq(password, &configured.password);
        if !(user_ok & pass_ok) {
            tracing::warn!(target: "audit", "worker login failed");
            return Err(ApiError::InvalidCredentials);
        }

        let token = token::generate().map_err(|e| ApiError::WorksOnMyMachine(e.to_string()))?;
        let session = Session { username: configured.username.clone(), expires_at: Utc::now() + SESSION_TTL };
        let mut sessions = self.lock();
        let now = Utc::now();
        sessions.retain(|_, s| s.expires_at > now);
        sessions.insert(token::hash(&token), session.clone());
        tracing::info!(target: "audit", username = %session.username, "worker logged in");
        Ok((token, session))
    }

    /// The live session for a bearer token, if any.
    pub fn get(&self, token: &str) -> Option<Session> {
        let hash = token::hash(token);
        let mut sessions = self.lock();
        match sessions.get(&hash) {
            Some(session) if session.expires_at > Utc::now() => Some(session.clone()),
            Some(_) => {
                sessions.remove(&hash);
                None
            }
            None => None,
        }
    }

    pub fn logout(&self, token: &str) {
        self.lock().remove(&token::hash(token));
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Session>> {
        // A panic while holding the lock leaves the map usable; don't take the server down with it.
        self.by_token_hash.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Equality that takes the same time wherever the inputs differ (and whatever their lengths).
fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (Sha256::digest(a.as_bytes()), Sha256::digest(b.as_bytes()));
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn creds() -> Credentials {
        Credentials { username: "agent".into(), password: "correct horse".into() }
    }

    #[test]
    fn login_issues_a_token_that_resolves_to_the_session() {
        let sessions = Sessions::default();
        let (token, session) = sessions.login(Some(&creds()), "agent", "correct horse").unwrap();
        assert_eq!(sessions.get(&token), Some(session));
        sessions.logout(&token);
        assert_eq!(sessions.get(&token), None);
    }

    #[test]
    fn wrong_or_missing_credentials_are_refused() {
        let sessions = Sessions::default();
        assert!(matches!(sessions.login(Some(&creds()), "agent", "wrong"), Err(ApiError::InvalidCredentials)));
        assert!(matches!(sessions.login(Some(&creds()), "Agent", "correct horse"), Err(ApiError::InvalidCredentials)));
        assert!(matches!(sessions.login(None, "agent", "correct horse"), Err(ApiError::InvalidCredentials)));
        assert_eq!(sessions.get("made-up"), None);
    }

    #[test]
    fn expired_sessions_are_gone() {
        let sessions = Sessions::default();
        let (token, _) = sessions.login(Some(&creds()), "agent", "correct horse").unwrap();
        sessions.lock().values_mut().for_each(|s| s.expires_at = Utc::now() - Duration::seconds(1));
        assert_eq!(sessions.get(&token), None);
    }
}
