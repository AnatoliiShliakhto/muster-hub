use axum::http::header;
use axum::{Json, response::IntoResponse};
use mhub_derive::{api_handler, api_model};
use mhub_domain::SYSTEM;
use std::sync::LazyLock;
use std::time::Instant;

#[api_model]
/// Healthcheck endpoint response
struct HealthResponse {
    /// Api status
    status: &'static str,
    /// Api version
    version: &'static str,
    /// Api uptime in seconds
    uptime: u64,
}

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

#[api_handler(
    get,
    path = "/health",
    responses((status = OK, description = "Healthcheck endpoint", body = HealthResponse)),
    tag = SYSTEM,
)]
pub(super) async fn health_handler() -> impl IntoResponse {
    let body = HealthResponse {
        status: "up",
        version: env!("CARGO_PKG_VERSION"),
        uptime: START_TIME.elapsed().as_secs(),
    };

    (
        [
            (header::CACHE_CONTROL, "no-store, no-cache, must-revalidate"),
            (header::PRAGMA, "no-cache"),
        ],
        Json(body),
    )
}
