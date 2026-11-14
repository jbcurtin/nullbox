//! Nullbox API: the HTTP layer, services and storage around `nullbox-domain`.
//!
//! Extension seams, each in one place:
//! - auth: [`http::extract::Actor`], accounts and sessions in [`auth::Accounts`]
//! - authorization: [`policy::can`]
//! - audit/webhooks/metrics: [`events::EventSink`]
//! - middleware (tracing, CORS, rate limits...): [`http::middleware`]
//! - OpenTelemetry: [`telemetry`]
//! - storage: [`repo::TicketRepository`]
//! - payments: [`payments::Payments`]
//! - settings: [`config::Config`]

pub mod auth;
pub mod config;
pub mod db;
pub mod dto;
pub mod error;
pub mod events;
pub mod http;
pub mod passwords;
pub mod payments;
pub mod policy;
pub mod repo;
pub mod service;
pub mod telemetry;
pub mod token;

use std::sync::Arc;

pub use config::Config;
pub use error::ApiError;

/// Shared state for all handlers.
#[derive(Clone)]
pub struct AppState {
    pub tickets: Arc<service::TicketService>,
    pub payments: Arc<payments::Payments>,
    pub config: Arc<Config>,
    pub accounts: Arc<auth::Accounts>,
}

impl AppState {
    pub fn new(pool: sqlx::SqlitePool, config: Config) -> Self {
        Self::with_leak_check(pool, config, Arc::new(passwords::PwnedPasswords::default()))
    }

    /// Like `new`, with a chosen leaked-password check (tests use the offline one).
    pub fn with_leak_check(pool: sqlx::SqlitePool, config: Config, leaks: Arc<dyn passwords::LeakCheck>) -> Self {
        let repo = Arc::new(repo::SqliteTicketRepository::new(pool.clone()));
        let events = Arc::new(events::TracingEventSink);
        let workers = Arc::new(repo::SqliteWorkerRepository::new(pool));
        let accounts = auth::Accounts::new(workers, leaks, config.worker_signup_code.clone());
        Self {
            tickets: Arc::new(service::TicketService::new(repo, events)),
            payments: Arc::new(payments::Payments::new(&config)),
            config: Arc::new(config),
            accounts: Arc::new(accounts),
        }
    }
}
