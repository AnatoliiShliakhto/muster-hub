use axum::http::header;
use axum::{Json, response::IntoResponse};
use mhub_derive::{api_handler, api_model};
use std::sync::LazyLock;
use std::time::Instant;

#[api_model]
/// Healthcheck endpoint response
pub struct HealthResponse {
    /// Api status
    pub status: &'static str,
    /// Api version
    pub version: &'static str,
    /// Api uptime in seconds
    pub uptime: u64,
}

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

#[api_handler(
    get,
    path = "/health",
    responses((status = OK, description = "Healthcheck endpoint", body = HealthResponse)),
    tag = "System",
)]
pub async fn health_handler() -> impl IntoResponse {
    let body = Json(HealthResponse {
        status: "up",
        version: env!("CARGO_PKG_VERSION"),
        uptime: START_TIME.elapsed().as_secs(),
    });

    (
        [
            (header::CACHE_CONTROL, "no-store, no-cache, must-revalidate"),
            (header::PRAGMA, "no-cache"),
        ],
        body,
    )
}
