//! Nullbox API: the HTTP layer, services and storage around `nullbox-domain`.
//!
//! Extension seams, each in one place:
//! - auth: [`http::extract::Actor`], sessions in [`auth::Sessions`]
//! - authorization: [`policy::can`]
//! - audit/webhooks/metrics: [`events::EventSink`]
//! - middleware (tracing, CORS, rate limits...): [`http::middleware`]
//! - storage: [`repo::TicketRepository`]
//! - settings: [`config::Config`]

pub mod auth;
pub mod config;
pub mod db;
pub mod dto;
pub mod error;
pub mod events;
pub mod http;
pub mod policy;
pub mod repo;
pub mod service;
pub mod token;

use std::sync::Arc;

pub use config::Config;
pub use error::ApiError;

/// Shared state for all handlers.
#[derive(Clone)]
pub struct AppState {
    pub tickets: Arc<service::TicketService>,
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
            config: Arc::new(config),
            sessions: Arc::new(auth::Sessions::default()),
        }
    }
}
