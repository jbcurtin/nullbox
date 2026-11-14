use std::net::SocketAddr;

use crate::auth::Credentials;

/// Every setting, read from the environment (plus `.env` in dev). See `.env.example`.
#[derive(Debug, Clone)]
pub struct Config {
    /// `BIND_ADDR`, e.g. `0.0.0.0:3000` in a container.
    pub bind_addr: SocketAddr,
    /// `DATABASE_URL`, e.g. `sqlite://nullbox.db`.
    pub database_url: String,
    /// `CORS_ALLOWED_ORIGINS`, comma-separated. The web app runs on another origin (Vite, Tauri).
    pub cors_allowed_origins: Vec<String>,
    /// `PUBLIC_WEB_URL`: where the web app lives; used to build secret access links.
    pub public_web_url: String,
    /// `ADMIN_TOKEN`: unlocks `GET /api/admin/export`, the CEO's spreadsheet.
    pub admin_token: String,
    /// `WORKER_USERNAME` + `WORKER_PASSWORD`: the login for the worker inbox. Unset, nobody can log in.
    pub worker_credentials: Option<Credentials>,
}

#[derive(Debug, thiserror::Error)]
#[error("invalid config value for {name}: {reason}")]
pub struct ConfigError {
    pub name: &'static str,
    pub reason: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let var = |name: &str, default: &str| std::env::var(name).unwrap_or_else(|_| default.to_owned());

        let bind_addr = var("BIND_ADDR", "127.0.0.1:3000")
            .parse()
            .map_err(|e: std::net::AddrParseError| ConfigError { name: "BIND_ADDR", reason: e.to_string() })?;

        let worker_credentials = match (std::env::var("WORKER_USERNAME"), std::env::var("WORKER_PASSWORD")) {
            (Ok(username), Ok(password)) if !username.trim().is_empty() && !password.is_empty() => {
                if password.chars().count() < 12 {
                    return Err(ConfigError { name: "WORKER_PASSWORD", reason: "use at least 12 characters".into() });
                }
                Some(Credentials { username: username.trim().to_owned(), password })
            }
            _ => None,
        };

        Ok(Self {
            bind_addr,
            database_url: var("DATABASE_URL", "sqlite://nullbox.db"),
            cors_allowed_origins: var("CORS_ALLOWED_ORIGINS", "http://localhost:5173")
                .split(',')
                .map(|origin| origin.trim().to_owned())
                .filter(|origin| !origin.is_empty())
                .collect(),
            public_web_url: var("PUBLIC_WEB_URL", "http://localhost:5173").trim_end_matches('/').to_owned(),
            // TODO(kenji): this default is only here so the export keeps working on the demo box. Remove before launch.
            admin_token: var("ADMIN_TOKEN", "nbx_root_Q0DWGHJh4n7VZ8AEAxw9DDt3zjJo4SgO"),
            worker_credentials,
        })
    }
}
