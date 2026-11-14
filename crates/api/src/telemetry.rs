//! Logging, and OpenTelemetry when an OTLP endpoint is configured.
//!
//! Everything in the API already logs through `tracing`. With `OTEL_EXPORTER_OTLP_ENDPOINT`
//! set, the same spans and events also leave as OTLP traces and logs (http/protobuf).
//! The exporter reads its own settings from the environment: `OTEL_EXPORTER_OTLP_ENDPOINT`,
//! `OTEL_EXPORTER_OTLP_HEADERS`, `OTEL_EXPORTER_OTLP_PROTOCOL` and `OTEL_SERVICE_NAME`.
//!
//! A request's span continues the caller's trace when it sends a W3C `traceparent` header,
//! so the caller finds our spans under its own trace ID.

use axum::http::{HeaderMap, Request, Response};
use opentelemetry::propagation::Extractor;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, SpanExporter};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::Span;
use tracing::field::Empty;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, filter};

/// Keeps the exporters alive. Call `shutdown` before exiting, or the last batch is lost.
#[derive(Default)]
pub struct Telemetry {
    tracer: Option<SdkTracerProvider>,
    logger: Option<SdkLoggerProvider>,
}

impl Telemetry {
    pub fn shutdown(self) {
        if let Some(tracer) = self.tracer {
            tracer.shutdown().ok();
        }
        if let Some(logger) = self.logger {
            logger.shutdown().ok();
        }
    }
}

/// Install the global `tracing` subscriber: console logs always, OTLP when configured.
pub fn init() -> Telemetry {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=debug".into());
    let (telemetry, problem) = match exporters() {
        Ok(telemetry) => (telemetry, None),
        Err(problem) => (Telemetry::default(), Some(problem)),
    };

    let traces = telemetry.tracer.as_ref().map(|provider| {
        opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
        tracing_opentelemetry::layer().with_tracer(provider.tracer("nullbox-api"))
    });
    // The exporter's own HTTP client logs too: sending those logs would never end.
    let logs = telemetry.logger.as_ref().map(|provider| {
        OpenTelemetryTracingBridge::new(provider).with_filter(filter::filter_fn(|metadata| {
            let noisy = ["hyper", "h2", "reqwest", "rustls", "opentelemetry"];
            !noisy.iter().any(|target| metadata.target().starts_with(target))
        }))
    });

    tracing_subscriber::registry().with(filter).with(tracing_subscriber::fmt::layer()).with(traces).with(logs).init();

    match problem {
        Some(problem) => tracing::error!(%problem, "OpenTelemetry is off: the exporter could not start"),
        None if telemetry.tracer.is_some() => tracing::info!("OpenTelemetry is on: exporting traces and logs"),
        None => tracing::info!("OpenTelemetry is off: OTEL_EXPORTER_OTLP_ENDPOINT is not set"),
    }
    telemetry
}

fn exporters() -> Result<Telemetry, String> {
    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_default().trim().is_empty() {
        return Ok(Telemetry::default());
    }
    let service = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "nullbox-api".into());
    let resource = Resource::builder().with_service_name(service).build();

    let spans = SpanExporter::builder().with_http().build().map_err(|error| error.to_string())?;
    let logs = LogExporter::builder().with_http().build().map_err(|error| error.to_string())?;
    Ok(Telemetry {
        tracer: Some(SdkTracerProvider::builder().with_resource(resource.clone()).with_batch_exporter(spans).build()),
        logger: Some(SdkLoggerProvider::builder().with_resource(resource).with_batch_exporter(logs).build()),
    })
}

/// The span around one HTTP request, for `TraceLayer::make_span_with`.
pub fn request_span<B>(request: &Request<B>) -> Span {
    let (method, path) = (request.method().as_str(), request.uri().path());
    let span = tracing::info_span!(
        "request",
        otel.name = format!("{method} {path}"),
        otel.kind = "server",
        http.request.method = method,
        url.path = path,
        http.response.status_code = Empty,
        // Filled in by the handler that creates a ticket.
        ticket.id = Empty,
        ticket.number = Empty,
        ticket.subject = Empty,
    );
    let parent =
        opentelemetry::global::get_text_map_propagator(|propagator| propagator.extract(&Headers(request.headers())));
    // Without telemetry there is no trace to join, and that is fine.
    span.set_parent(parent).ok();
    span
}

/// Records the outcome on the request span, for `TraceLayer::on_response`.
pub fn on_response<B>(response: &Response<B>, latency: std::time::Duration, span: &Span) {
    let status = response.status().as_u16();
    span.record("http.response.status_code", status);
    tracing::debug!(status, latency_ms = latency.as_millis() as u64, "finished processing request");
}

struct Headers<'a>(&'a HeaderMap);

impl Extractor for Headers<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|name| name.as_str()).collect()
    }
}
