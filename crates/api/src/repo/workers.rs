//! Worker accounts. Plain `sqlx::query` (checked at run time, not compile time), so no
//! `.sqlx` cache entry is needed; the HTTP tests cover every query.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::RepoError;

#[derive(Debug, Clone)]
pub struct WorkerAccount {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub logged_in_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait WorkerRepository: Send + Sync {
    /// Inserts the account. `Ok(false)` if the username (case-insensitively) is taken.
    async fn create(&self, username: &str, password_hash: &str, now: DateTime<Utc>) -> Result<bool, RepoError>;

    async fn find(&self, username: &str) -> Result<Option<WorkerAccount>, RepoError>;

    /// Marks the account's one login as used. `Ok(false)` if it already was: two logins
    /// racing each other can't both win.
    async fn claim_login(&self, id: Uuid, now: DateTime<Utc>) -> Result<bool, RepoError>;
}

pub struct SqliteWorkerRepository {
    pool: SqlitePool,
}

impl SqliteWorkerRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn key(username: &str) -> String {
    username.trim().to_lowercase()
}

#[async_trait]
impl WorkerRepository for SqliteWorkerRepository {
    async fn create(&self, username: &str, password_hash: &str, now: DateTime<Utc>) -> Result<bool, RepoError> {
        let result = sqlx::query(
            "INSERT INTO workers (id, username, username_key, password_hash, created_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT (username_key) DO NOTHING",
        )
        .bind(Uuid::now_v7().to_string())
        .bind(username)
        .bind(key(username))
        .bind(password_hash)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    async fn find(&self, username: &str) -> Result<Option<WorkerAccount>, RepoError> {
        let row = sqlx::query("SELECT id, username, password_hash, logged_in_at FROM workers WHERE username_key = ?")
            .bind(key(username))
            .fetch_optional(&self.pool)
            .await?;
        let Some(row) = row else { return Ok(None) };
        let id: String = row.try_get("id")?;
        Ok(Some(WorkerAccount {
            id: Uuid::parse_str(&id).map_err(|e| RepoError::Corrupt(e.to_string()))?,
            username: row.try_get("username")?,
            password_hash: row.try_get("password_hash")?,
            logged_in_at: row.try_get("logged_in_at")?,
        }))
    }

    async fn claim_login(&self, id: Uuid, now: DateTime<Utc>) -> Result<bool, RepoError> {
        let result = sqlx::query("UPDATE workers SET logged_in_at = ? WHERE id = ? AND logged_in_at IS NULL")
            .bind(now)
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() == 1)
    }
}
