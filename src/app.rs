use axum::{
    Router,
    routing::{get, post, put},
};
use tower_http::cors::{Any, CorsLayer};
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
        .route(
            "/api/v1/agents/instances",
            get(handler::list_agent_instances),
        )
        .route(
            "/api/v1/agents/instances/{instance_id}",
            put(handler::register_agent_instance).get(handler::get_agent_instance),
        )
        .route(
            "/api/v1/agents/instances/{instance_id}/heartbeat",
            axum::routing::post(handler::heartbeat_agent_instance),
        )
        .route("/api/v1/participants", get(handler::list_participants))
        .route(
            "/api/v1/participants/{participant_id}",
            get(handler::get_participant),
        )
        .route(
            "/api/v1/participants/{participant_id}/delivery-target",
            get(handler::get_participant_delivery_target),
        )
        .route(
            "/api/v1/chat/participant-selection",
            get(handler::get_chat_participant_selection).put(handler::select_chat_participant),
        )
        .route(
            "/api/v1/chat/participant-selection/delivery-target",
            get(handler::get_selected_delivery_target),
        )
        .route(
            "/api/v1/chat/sessions",
            post(handler::create_chat_session).get(handler::list_chat_sessions),
        )
        .route(
            "/api/v1/chat/sessions/{session_id}",
            get(handler::get_chat_session),
        )
        .route(
            "/api/v1/chat/sessions/{session_id}/messages",
            post(handler::create_chat_message).get(handler::list_chat_messages),
        )
        .route(
            "/api/v1/chat/sessions/{session_id}/messages/{message_id}/chunks",
            post(handler::append_chat_message_chunk),
        )
        .route(
            "/api/v1/chat/sessions/{session_id}/messages/{message_id}/finalize",
            post(handler::finalize_chat_message),
        )
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any),
        )
        .with_state(state)
}
