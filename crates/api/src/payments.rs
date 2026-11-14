//! Payments: one product, sold through Stripe Checkout.
//!
//! We never see a card. The API asks Stripe for a Checkout Session and hands the browser
//! its URL; Stripe takes the payment on its own page and sends the buyer back to us.

use serde::Deserialize;

use crate::{ApiError, Config};

/// The whole catalogue. The product and its price live in the Stripe account; we find
/// them by this name, so nothing here has to change when the price does.
pub const PRODUCT_NAME: &str = "Nothing to See Here";

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
struct ProductList {
    data: Vec<Product>,
}

#[derive(Deserialize)]
struct Product {
    name: String,
    default_price: Option<Price>,
}

#[derive(Deserialize)]
struct Price {
    id: String,
    /// `one_time` or `recurring`.
    #[serde(rename = "type")]
    kind: String,
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
        let price = self.price(api_key).await?;
        // Stripe replaces {CHECKOUT_SESSION_ID} itself when it sends the buyer back.
        let success_url = format!("{}/shop/thanks?session_id={{CHECKOUT_SESSION_ID}}", self.web_url);
        let cancel_url = format!("{}/shop", self.web_url);
        // A recurring price can only be bought as a subscription.
        let mode = if price.kind == "recurring" { "subscription" } else { "payment" };
        let form = [
            ("mode", mode),
            ("success_url", success_url.as_str()),
            ("cancel_url", cancel_url.as_str()),
            ("line_items[0][price]", price.id.as_str()),
            ("line_items[0][quantity]", "1"),
        ];
        let request = self.client.post(format!("{}/v1/checkout/sessions", self.api_base)).form(&form);
        let session: Session = send(request.bearer_auth(api_key)).await?;
        session.url.ok_or_else(|| unavailable("Stripe returned a session without a URL"))
    }

    /// The price of the product named `PRODUCT_NAME` in the Stripe account.
    async fn price(&self, api_key: &str) -> Result<Price, ApiError> {
        let query = [("active", "true"), ("limit", "100"), ("expand[]", "data.default_price")];
        let request = self.client.get(format!("{}/v1/products", self.api_base)).query(&query);
        let products: ProductList = send(request.bearer_auth(api_key)).await?;
        products
            .data
            .into_iter()
            .find(|product| product.name == PRODUCT_NAME)
            .ok_or_else(|| unavailable(format!("no active Stripe product named \"{PRODUCT_NAME}\"")))?
            .default_price
            .ok_or_else(|| unavailable(format!("the Stripe product \"{PRODUCT_NAME}\" has no default price")))
    }
}

/// Send a request to Stripe and read its JSON answer; a refusal becomes its error message.
async fn send<T: serde::de::DeserializeOwned>(request: reqwest::RequestBuilder) -> Result<T, ApiError> {
    let response = request.send().await.map_err(|error| unavailable(format!("Stripe is unreachable: {error}")))?;
    let status = response.status();
    if !status.is_success() {
        let reason = response.json::<StripeFailure>().await.ok().and_then(|failure| failure.error.message);
        return Err(unavailable(format!("Stripe answered {status}: {}", reason.unwrap_or_default())));
    }
    response.json().await.map_err(|error| unavailable(format!("unreadable Stripe answer: {error}")))
}

fn unavailable(detail: impl Into<String>) -> ApiError {
    ApiError::PaymentsUnavailable(detail.into())
}
