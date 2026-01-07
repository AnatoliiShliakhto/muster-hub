use axum::Router;
use mhub::kernel::prelude::ApiState;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

#[derive(OpenApi)]
struct ApiDoc;

#[allow(unreachable_pub)]
pub fn init(state: ApiState) -> Router {
    let api = ApiDoc::openapi();

    // Separate the OpenAPI routes and the API documentation object
    let (openapi_routes, api_doc) = OpenApiRouter::with_openapi(api)
        .merge(mhub::server::router::system_router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .split_for_parts();

    // Create the Scalar UI routes
    let scalar_routes = Scalar::with_url("/api", api_doc);

    // Merge all routes and then apply the state to the final router
    Router::new().merge(openapi_routes).merge(scalar_routes)
}
