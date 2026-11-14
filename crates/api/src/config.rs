use std::net::SocketAddr;

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
    /// `WORKER_SIGNUP_CODE`: if set, creating a worker account needs this code. Unset, sign-up is open.
    pub worker_signup_code: Option<String>,
    /// `MCP_TOKEN`: the bearer token for the MCP server at `/mcp`. Unset means `/mcp` refuses everyone.
    pub mcp_token: Option<String>,
    /// `STRIPE_API_KEY`: the Stripe secret key. Unset means the shop cannot take payments.
    pub stripe_api_key: Option<String>,
    /// `STRIPE_API_BASE`: where the Stripe API lives. Only a test double needs to change it.
    pub stripe_api_base: String,
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

        let worker_signup_code =
            std::env::var("WORKER_SIGNUP_CODE").ok().map(|code| code.trim().to_owned()).filter(|code| !code.is_empty());

        Ok(Self {
            bind_addr,
            database_url: var("DATABASE_URL", "sqlite://nullbox.db"),
            cors_allowed_origins: var("CORS_ALLOWED_ORIGINS", "http://localhost:5173")
                .split(',')
                .map(|origin| origin.trim().to_owned())
                .filter(|origin| !origin.is_empty())
                .collect(),
            public_web_url: var("PUBLIC_WEB_URL", "http://localhost:5173").trim_end_matches('/').to_owned(),
            admin_token: secret("ADMIN_TOKEN")?.ok_or_else(|| too_short("ADMIN_TOKEN"))?,
            worker_signup_code,
            mcp_token: secret("MCP_TOKEN")?,
            stripe_api_key: secret("STRIPE_API_KEY")?,
            stripe_api_base: var("STRIPE_API_BASE", "https://api.stripe.com").trim_end_matches('/').to_owned(),
        })
    }
}

/// A secret from the environment: `None` when unset or blank, an error when too short to be one.
/// Secrets have no defaults: a default would be in the source code for anyone to read.
fn secret(name: &'static str) -> Result<Option<String>, ConfigError> {
    let value = std::env::var(name).unwrap_or_default().trim().to_owned();
    match value.len() {
        0 => Ok(None),
        len if len < MIN_SECRET_LEN => Err(too_short(name)),
        _ => Ok(Some(value)),
    }
}

const MIN_SECRET_LEN: usize = 32;

fn too_short(name: &'static str) -> ConfigError {
    ConfigError {
        name,
        reason: format!(
            "must be set to at least {MIN_SECRET_LEN} characters, e.g. the output of `openssl rand -hex 32`"
        ),
    }
}
