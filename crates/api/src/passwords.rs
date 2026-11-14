//! Password rules, hashing, and the leaked-password check.

use std::time::Duration;

use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use async_trait::async_trait;
use sha1::{Digest, Sha1};

use crate::ApiError;

pub const MIN_PASSWORD_CHARS: usize = 12;
pub const MAX_PASSWORD_CHARS: usize = 256;

/// 3 to 32 letters, digits, `_`, `.` or `-` (any script, so 山田 works). No spaces.
pub fn validate_username(username: &str) -> Result<String, ApiError> {
    let username = username.trim();
    let len = username.chars().count();
    let ok_chars = username.chars().all(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '-'));
    if !(3..=32).contains(&len) || !ok_chars {
        return Err(ApiError::Validation("username: 3 to 32 letters, digits, '_', '.' or '-'".into()));
    }
    Ok(username.to_owned())
}

pub fn validate_password_length(password: &str) -> Result<(), ApiError> {
    let len = password.chars().count();
    if len < MIN_PASSWORD_CHARS {
        return Err(ApiError::PasswordTooShort);
    }
    if len > MAX_PASSWORD_CHARS {
        return Err(ApiError::Validation(format!("password: at most {MAX_PASSWORD_CHARS} characters")));
    }
    Ok(())
}

/// Argon2id with a random salt, as a PHC string. Slow on purpose: run it off the async threads.
pub async fn hash(password: String) -> Result<String, ApiError> {
    tokio::task::spawn_blocking(move || Argon2::default().hash_password(password.as_bytes()).map(|h| h.to_string()))
        .await
        .map_err(|e| ApiError::WorksOnMyMachine(e.to_string()))?
        .map_err(|e| ApiError::WorksOnMyMachine(e.to_string()))
}

/// Checks a password against a stored hash. A malformed hash counts as a mismatch.
pub async fn verify(password: String, phc: String) -> bool {
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&phc)
            .is_ok_and(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
    })
    .await
    .unwrap_or(false)
}

/// A real Argon2 hash of nothing in particular, made once. Logins for unknown usernames are
/// checked against it, so they take as long as logins for real ones and don't reveal which exist.
pub fn dummy_hash() -> String {
    static DUMMY: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        Argon2::default().hash_password(b"nullbox dummy password").map(|h| h.to_string()).unwrap_or_default()
    });
    DUMMY.clone()
}

/// Has this password shown up in a data breach?
#[async_trait]
pub trait LeakCheck: Send + Sync {
    async fn is_leaked(&self, password: &str) -> bool;
}

/// Very common passwords, checked offline. Only ones long enough to pass the length rule matter.
const COMMON: &[&str] = &[
    "password1234",
    "password12345",
    "password123456",
    "123456789012",
    "1234567890123",
    "12345678901234",
    "qwertyuiop12",
    "qwertyuiopasdf",
    "qwerty123456",
    "iloveyou1234",
    "abcdefghijkl",
    "abc123456789",
    "letmein12345",
    "welcome12345",
    "administrator",
    "aaaaaaaaaaaa",
    "111111111111",
    "000000000000",
    "changeme1234",
    "passw0rd1234",
    "trustno1trustno1",
    "football1234",
    "baseball1234",
    "superman1234",
    "1q2w3e4r5t6y",
    "qazwsxedcrfv",
    "zaq12wsxcde3",
    "nullbox12345",
    "correcthorsebatterystaple",
];

/// [Pwned Passwords](https://haveibeenpwned.com/API/v3#PwnedPasswords) with k-anonymity:
/// only the first 5 hex characters of the password's SHA-1 leave the server, never the
/// password or its full hash. If the service can't be reached, only the offline list is used.
pub struct PwnedPasswords {
    client: reqwest::Client,
}

impl Default for PwnedPasswords {
    fn default() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .user_agent("nullbox-api")
            .build()
            .expect("building the HTTP client");
        Self { client }
    }
}

#[async_trait]
impl LeakCheck for PwnedPasswords {
    async fn is_leaked(&self, password: &str) -> bool {
        if is_common(password) {
            return true;
        }
        let digest: String = Sha1::digest(password.as_bytes()).iter().map(|b| format!("{b:02X}")).collect();
        let (prefix, suffix) = digest.split_at(5);
        let url = format!("https://api.pwnedpasswords.com/range/{prefix}");
        let response = self.client.get(url).header("Add-Padding", "true").send().await;
        let body = match response.and_then(reqwest::Response::error_for_status) {
            Ok(response) => response.text().await,
            Err(e) => Err(e),
        };
        match body {
            // Lines are "SUFFIX:COUNT"; padding entries have a count of 0.
            Ok(body) => body.lines().any(|line| {
                line.split_once(':').is_some_and(|(s, count)| s.eq_ignore_ascii_case(suffix) && count.trim() != "0")
            }),
            Err(e) => {
                tracing::warn!(error = %e, "Pwned Passwords unreachable; only the offline list was checked");
                false
            }
        }
    }
}

/// The offline check alone. Used in tests, so they never touch the network.
pub struct OfflineLeakCheck;

#[async_trait]
impl LeakCheck for OfflineLeakCheck {
    async fn is_leaked(&self, password: &str) -> bool {
        is_common(password)
    }
}

fn is_common(password: &str) -> bool {
    let lower = password.to_lowercase();
    COMMON.contains(&lower.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usernames() {
        assert_eq!(validate_username("  kenji ").unwrap(), "kenji");
        assert!(validate_username("山田太郎").is_ok());
        assert!(validate_username("a.b-c_d").is_ok());
        for bad in ["ab", "has space", "semi;colon", "", &"x".repeat(33)] {
            assert!(validate_username(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn password_length() {
        assert!(matches!(validate_password_length("short"), Err(ApiError::PasswordTooShort)));
        assert!(validate_password_length("twelve chars").is_ok());
    }

    #[tokio::test]
    async fn hash_and_verify() {
        let phc = hash("correct horse battery".into()).await.unwrap();
        assert!(phc.starts_with("$argon2id$"));
        assert!(verify("correct horse battery".into(), phc.clone()).await);
        assert!(!verify("wrong".into(), phc).await);
        assert!(dummy_hash().starts_with("$argon2id$"));
        assert!(!verify("anything".into(), dummy_hash()).await);
    }

    #[tokio::test]
    async fn common_passwords_are_leaked_offline() {
        assert!(OfflineLeakCheck.is_leaked("Password1234").await);
        assert!(!OfflineLeakCheck.is_leaked("violet-otter-gravel-42").await);
    }
}
