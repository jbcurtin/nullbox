//! Nullbox API: the HTTP layer, services and storage around `nullbox-domain`.
//!
//! Extension seams, each in one place:
//! - auth: [`http::extract::Actor`], sessions in [`auth::Sessions`]
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
    pub sessions: Arc<auth::Sessions>,
}

impl AppState {
    pub fn new(pool: sqlx::SqlitePool, config: Config) -> Self {
        let repo = Arc::new(repo::SqliteTicketRepository::new(pool));
        let events = Arc::new(events::TracingEventSink);
        if config.worker_credentials.is_none() {
            tracing::warn!("WORKER_USERNAME/WORKER_PASSWORD not set: the worker inbox is locked for everyone");
        }
        Self {
            tickets: Arc::new(service::TicketService::new(repo, events)),
            payments: Arc::new(payments::Payments::new(&config)),
            config: Arc::new(config),
            sessions: Arc::new(auth::Sessions::default()),
        }
    }
}
