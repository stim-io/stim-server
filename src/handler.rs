use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{schema::ErrorResponse, state::AppState};
use stim_proto::DiscoveryRecord;

#[utoipa::path(
    get,
    path = "/api/v1/health",
    operation_id = "health",
    tag = "health",
    responses((status = 200, description = "Health check response", body = String))
)]
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json("ok"))
}

#[utoipa::path(
    put,
    path = "/api/v1/discovery/endpoints/{endpoint_id}",
    operation_id = "register_endpoint",
    tag = "discovery",
    params(("endpoint_id" = String, Path, description = "Declared endpoint identifier")),
    request_body = DiscoveryRecord,
    responses(
        (status = 200, description = "Registered discovery record", body = DiscoveryRecord),
        (status = 400, description = "Path/body mismatch", body = ErrorResponse)
    )
)]
pub async fn register_endpoint(
    State(state): State<AppState>,
    Path(endpoint_id): Path<String>,
    Json(record): Json<DiscoveryRecord>,
) -> Result<Json<DiscoveryRecord>, ApiError> {
    if record.endpoint_declaration.endpoint_id != endpoint_id {
        return Err(ApiError::bad_request(
            "endpoint_id path must match endpoint_declaration.endpoint_id",
        ));
    }

    state.registry.upsert(record.clone());
    Ok(Json(record))
}

#[utoipa::path(
    get,
    path = "/api/v1/discovery/endpoints/{endpoint_id}",
    operation_id = "discover_endpoint",
    tag = "discovery",
    params(("endpoint_id" = String, Path, description = "Declared endpoint identifier")),
    responses(
        (status = 200, description = "Discovered endpoint record", body = DiscoveryRecord),
        (status = 404, description = "Endpoint not found", body = ErrorResponse)
    )
)]
pub async fn discover_endpoint(
    State(state): State<AppState>,
    Path(endpoint_id): Path<String>,
) -> Result<Json<DiscoveryRecord>, ApiError> {
    state
        .registry
        .get(&endpoint_id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("endpoint not registered"))
}

pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request",
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(ErrorResponse {
                code: self.code.to_string(),
                message: self.message,
            }),
        )
            .into_response()
    }
}
