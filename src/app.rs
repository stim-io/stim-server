use axum::{Router, routing::get, routing::put};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{handler, openapi::ApiDoc, state::AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/health", get(handler::health))
        .route(
            "/api/v1/discovery/endpoints/{endpoint_id}",
            put(handler::register_endpoint).get(handler::discover_endpoint),
        )
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
