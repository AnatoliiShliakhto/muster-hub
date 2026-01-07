use super::health;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub fn system_router<S>() -> OpenApiRouter<S>
where
    S: Send + Sync + Clone + 'static,
{
    OpenApiRouter::<S>::new().routes(routes!(health::health_handler))
}
