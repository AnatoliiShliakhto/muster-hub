use super::health;
use crate::api::ApiState;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub fn system_router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::<ApiState>::new().routes(routes!(health::health_handler))
}
