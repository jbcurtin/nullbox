use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use nullbox_domain::Message;
use uuid::Uuid;

use super::extract::{ApiJson, ApiPath, TicketToken};
use crate::dto::{CheckoutResponse, CreateTicketRequest, CreateTicketResponse, CustomerReplyRequest, TicketThread};
use crate::{ApiError, AppState};

pub async fn create_ticket(
    State(state): State<AppState>,
    ApiJson(req): ApiJson<CreateTicketRequest>,
) -> Result<(StatusCode, Json<CreateTicketResponse>), ApiError> {
    let created = state.tickets.create(req).await?;
    // On the request span and in the log, so a ticket can be found in telemetry by its subject.
    let ticket = &created.ticket;
    tracing::Span::current()
        .record("ticket.id", ticket.id.to_string())
        .record("ticket.number", ticket.number)
        .record("ticket.subject", &ticket.subject);
    tracing::info!(ticket.id = %ticket.id, ticket.number, ticket.subject, "ticket created");
    let access_link = format!("{}/t/{}#{}", state.config.public_web_url, created.ticket.id, created.access_token);
    Ok((
        StatusCode::CREATED,
        Json(CreateTicketResponse { ticket: created.ticket, access_token: created.access_token, access_link }),
    ))
}

pub async fn get_ticket(
    State(state): State<AppState>,
    TicketToken(token): TicketToken,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<TicketThread>, ApiError> {
    Ok(Json(state.tickets.customer_thread(id, &token).await?))
}

pub async fn reply(
    State(state): State<AppState>,
    TicketToken(token): TicketToken,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<CustomerReplyRequest>,
) -> Result<(StatusCode, Json<Message>), ApiError> {
    let message = state.tickets.customer_reply(id, &token, &req.body).await?;
    Ok((StatusCode::CREATED, Json(message)))
}

/// Start buying the shop's one product. The browser goes to the returned Stripe page.
pub async fn checkout(State(state): State<AppState>) -> Result<Json<CheckoutResponse>, ApiError> {
    Ok(Json(CheckoutResponse { url: state.payments.checkout().await? }))
}
