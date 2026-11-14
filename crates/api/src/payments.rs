//! Payments: one product, sold through Stripe Checkout.
//!
//! We never see a card. The API asks Stripe for a Checkout Session and hands the browser
//! its URL; Stripe takes the payment on its own page and sends the buyer back to us.

use serde::Deserialize;

use crate::{ApiError, Config};

/// The whole catalogue. Stripe shows this name on the checkout page and in the dashboard.
pub const PRODUCT_NAME: &str = "Nothing to See Here";
const PRODUCT_DESCRIPTION: &str = "Exactly what it says. Ships instantly, weighs nothing.";
/// Yen has no minor unit: 500 means ¥500.
const PRICE_JPY: u32 = 500;

pub struct Payments {
    client: reqwest::Client,
    api_key: Option<String>,
    api_base: String,
    web_url: String,
}

#[derive(Deserialize)]
struct Session {
    url: Option<String>,
}

#[derive(Deserialize)]
struct StripeFailure {
    error: StripeError,
}

#[derive(Deserialize)]
struct StripeError {
    message: Option<String>,
}

impl Payments {
    pub fn new(config: &Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: config.stripe_api_key.clone(),
            api_base: config.stripe_api_base.clone(),
            web_url: config.public_web_url.clone(),
        }
    }

    /// Start a checkout for the product. Returns the Stripe page to send the buyer to.
    pub async fn checkout(&self) -> Result<String, ApiError> {
        let api_key = self.api_key.as_deref().ok_or_else(|| unavailable("STRIPE_API_KEY is not set"))?;
        // Stripe replaces {CHECKOUT_SESSION_ID} itself when it sends the buyer back.
        let success_url = format!("{}/shop/thanks?session_id={{CHECKOUT_SESSION_ID}}", self.web_url);
        let cancel_url = format!("{}/shop", self.web_url);
        let price = PRICE_JPY.to_string();
        let form = [
            ("mode", "payment"),
            ("success_url", success_url.as_str()),
            ("cancel_url", cancel_url.as_str()),
            ("line_items[0][quantity]", "1"),
            ("line_items[0][price_data][currency]", "jpy"),
            ("line_items[0][price_data][unit_amount]", price.as_str()),
            ("line_items[0][price_data][product_data][name]", PRODUCT_NAME),
            ("line_items[0][price_data][product_data][description]", PRODUCT_DESCRIPTION),
        ];

        let response = self
            .client
            .post(format!("{}/v1/checkout/sessions", self.api_base))
            .bearer_auth(api_key)
            .form(&form)
            .send()
            .await
            .map_err(|error| unavailable(format!("Stripe is unreachable: {error}")))?;
        let status = response.status();
        if !status.is_success() {
            let reason = response.json::<StripeFailure>().await.ok().and_then(|failure| failure.error.message);
            return Err(unavailable(format!("Stripe answered {status}: {}", reason.unwrap_or_default())));
        }
        let session: Session =
            response.json().await.map_err(|error| unavailable(format!("unreadable Stripe answer: {error}")))?;
        session.url.ok_or_else(|| unavailable("Stripe returned a session without a URL"))
    }
}

fn unavailable(detail: impl Into<String>) -> ApiError {
    ApiError::PaymentsUnavailable(detail.into())
}
