//! Worker accounts and sessions.
//!
//! - Sign-up creates an account with its own username and password (Argon2id). Short
//!   passwords and passwords known from data breaches are refused. If `WORKER_SIGNUP_CODE`
//!   is set, sign-up also needs that code.
//! - **Each account can log in exactly once.** That login returns a bearer token; after
//!   logging out (or once the session expires) the account is spent and the worker has
//!   to create a new one.
//! - Sessions live in memory (only token hashes), so restarting the API ends them too.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, Utc};
use sha2::{Digest, Sha256};

use crate::passwords::{self, LeakCheck};
use crate::repo::WorkerRepository;
use crate::{ApiError, token};

/// How long a login lasts.
pub const SESSION_TTL: Duration = Duration::hours(12);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub username: String,
    pub expires_at: DateTime<Utc>,
}

pub struct Accounts {
    repo: Arc<dyn WorkerRepository>,
    leaks: Arc<dyn LeakCheck>,
    signup_code: Option<String>,
    sessions: Sessions,
}

impl Accounts {
    pub fn new(repo: Arc<dyn WorkerRepository>, leaks: Arc<dyn LeakCheck>, signup_code: Option<String>) -> Self {
        Self { repo, leaks, signup_code, sessions: Sessions::default() }
    }

    /// Creates an account. Returns the username as stored.
    pub async fn sign_up(&self, username: &str, password: &str, code: Option<&str>) -> Result<String, ApiError> {
        if let Some(expected) = &self.signup_code
            && !constant_time_eq(code.unwrap_or_default(), expected)
        {
            return Err(ApiError::InvalidSignupCode);
        }
        let username = passwords::validate_username(username)?;
        passwords::validate_password_length(password)?;
        if self.leaks.is_leaked(password).await {
            return Err(ApiError::PasswordLeaked);
        }
        let hash = passwords::hash(password.to_owned()).await?;
        if !self.repo.create(&username, &hash, Utc::now()).await? {
            return Err(ApiError::UsernameTaken);
        }
        tracing::info!(target: "audit", %username, "worker account created");
        Ok(username)
    }

    /// The account's one login. Returns the token (shown once) and the session.
    pub async fn log_in(&self, username: &str, password: &str) -> Result<(String, Session), ApiError> {
        let account = self.repo.find(username.trim()).await?;
        // Unknown usernames still pay for a hash check, so timing doesn't reveal which exist.
        let phc = account.as_ref().map_or_else(passwords::dummy_hash, |a| a.password_hash.clone());
        let password_ok = passwords::verify(password.to_owned(), phc).await;
        let Some(account) = account.filter(|_| password_ok) else {
            tracing::warn!(target: "audit", username = %username.trim(), "worker login failed");
            return Err(ApiError::InvalidCredentials);
        };
        // Only after the right password: say the account is spent, and spend it atomically.
        if account.logged_in_at.is_some() || !self.repo.claim_login(account.id, Utc::now()).await? {
            tracing::warn!(target: "audit", username = %account.username, "worker login refused: account already used");
            return Err(ApiError::AccountAlreadyUsed);
        }
        let (token, session) = self.sessions.open(account.username)?;
        tracing::info!(target: "audit", username = %session.username, "worker logged in");
        Ok((token, session))
    }

    pub fn session(&self, token: &str) -> Option<Session> {
        self.sessions.get(token)
    }

    pub fn log_out(&self, token: &str) {
        if let Some(session) = self.sessions.remove(token) {
            tracing::info!(target: "audit", username = %session.username, "worker logged out; account spent");
        }
    }

    pub fn signup_code_required(&self) -> bool {
        self.signup_code.is_some()
    }
}

#[derive(Default)]
struct Sessions {
    by_token_hash: Mutex<HashMap<String, Session>>,
}

impl Sessions {
    fn open(&self, username: String) -> Result<(String, Session), ApiError> {
        let token = token::generate().map_err(|e| ApiError::WorksOnMyMachine(e.to_string()))?;
        let session = Session { username, expires_at: Utc::now() + SESSION_TTL };
        let mut sessions = self.lock();
        let now = Utc::now();
        sessions.retain(|_, s| s.expires_at > now);
        sessions.insert(token::hash(&token), session.clone());
        Ok((token, session))
    }

    fn get(&self, token: &str) -> Option<Session> {
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

    fn remove(&self, token: &str) -> Option<Session> {
        self.lock().remove(&token::hash(token))
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

    #[test]
    fn sessions_open_resolve_expire_and_close() {
        let sessions = Sessions::default();
        let (token, session) = sessions.open("kenji".into()).unwrap();
        assert_eq!(sessions.get(&token), Some(session));
        assert_eq!(sessions.get("made-up"), None);
        sessions.lock().values_mut().for_each(|s| s.expires_at = Utc::now() - Duration::seconds(1));
        assert_eq!(sessions.get(&token), None);

        let (token, _) = sessions.open("aiko".into()).unwrap();
        assert!(sessions.remove(&token).is_some());
        assert_eq!(sessions.get(&token), None);
    }

    #[test]
    fn constant_time_eq_compares() {
        assert!(constant_time_eq("code", "code"));
        assert!(!constant_time_eq("code", "Code"));
        assert!(!constant_time_eq("", "code"));
    }
}
