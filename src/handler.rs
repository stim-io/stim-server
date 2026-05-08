use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    schema::{
        AgentInstanceHeartbeatRequest, AgentInstanceListResponse, AgentInstanceRecord,
        AgentInstanceRegistrationRequest, ChatMessageChunkAppendRequest, ChatMessageCreateRequest,
        ChatMessageFinalizeRequest, ChatMessageListResponse, ChatMessageRecord,
        ChatSessionCreateRequest, ChatSessionListResponse, ChatSessionRecord, ErrorResponse,
        ParticipantDeliveryTargetResponse, ParticipantListResponse, ParticipantRecord,
        ParticipantSelectionRequest, ParticipantSelectionResponse,
    },
    state::AppState,
    store::ChatLedgerError,
};
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

    state.endpoint_registry.upsert(record.clone());
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
        .endpoint_registry
        .get(&endpoint_id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("endpoint not registered"))
}

#[utoipa::path(
    get,
    path = "/api/v1/agents/instances",
    operation_id = "list_agent_instances",
    tag = "agents",
    responses((status = 200, description = "Registered agent instances", body = AgentInstanceListResponse))
)]
pub async fn list_agent_instances(
    State(state): State<AppState>,
) -> Json<AgentInstanceListResponse> {
    Json(AgentInstanceListResponse {
        instances: state.agent_registry.list(),
    })
}

#[utoipa::path(
    put,
    path = "/api/v1/agents/instances/{instance_id}",
    operation_id = "register_agent_instance",
    tag = "agents",
    params(("instance_id" = String, Path, description = "Agent runtime instance identifier")),
    request_body = AgentInstanceRegistrationRequest,
    responses(
        (status = 200, description = "Registered agent instance projection", body = AgentInstanceRecord),
        (status = 400, description = "Path/body mismatch", body = ErrorResponse)
    )
)]
pub async fn register_agent_instance(
    State(state): State<AppState>,
    Path(instance_id): Path<String>,
    Json(request): Json<AgentInstanceRegistrationRequest>,
) -> Result<Json<AgentInstanceRecord>, ApiError> {
    if request.instance_id != instance_id {
        return Err(ApiError::bad_request(
            "instance_id path must match request.instance_id",
        ));
    }

    Ok(Json(state.agent_registry.register(request)))
}

#[utoipa::path(
    post,
    path = "/api/v1/agents/instances/{instance_id}/heartbeat",
    operation_id = "heartbeat_agent_instance",
    tag = "agents",
    params(("instance_id" = String, Path, description = "Agent runtime instance identifier")),
    request_body = AgentInstanceHeartbeatRequest,
    responses(
        (status = 200, description = "Updated agent instance projection", body = AgentInstanceRecord),
        (status = 400, description = "Path/body mismatch", body = ErrorResponse),
        (status = 404, description = "Agent instance not registered", body = ErrorResponse)
    )
)]
pub async fn heartbeat_agent_instance(
    State(state): State<AppState>,
    Path(instance_id): Path<String>,
    Json(request): Json<AgentInstanceHeartbeatRequest>,
) -> Result<Json<AgentInstanceRecord>, ApiError> {
    if request.instance_id != instance_id {
        return Err(ApiError::bad_request(
            "instance_id path must match request.instance_id",
        ));
    }

    state
        .agent_registry
        .heartbeat(request)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("agent instance not registered"))
}

#[utoipa::path(
    get,
    path = "/api/v1/agents/instances/{instance_id}",
    operation_id = "get_agent_instance",
    tag = "agents",
    params(("instance_id" = String, Path, description = "Agent runtime instance identifier")),
    responses(
        (status = 200, description = "Registered agent instance projection", body = AgentInstanceRecord),
        (status = 404, description = "Agent instance not registered", body = ErrorResponse)
    )
)]
pub async fn get_agent_instance(
    State(state): State<AppState>,
    Path(instance_id): Path<String>,
) -> Result<Json<AgentInstanceRecord>, ApiError> {
    state
        .agent_registry
        .get(&instance_id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("agent instance not registered"))
}

#[utoipa::path(
    get,
    path = "/api/v1/participants",
    operation_id = "list_participants",
    tag = "participants",
    responses((status = 200, description = "Product-visible participant projections", body = ParticipantListResponse))
)]
pub async fn list_participants(State(state): State<AppState>) -> Json<ParticipantListResponse> {
    Json(ParticipantListResponse {
        participants: state.agent_registry.list_participants(),
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/participants/{participant_id}",
    operation_id = "get_participant",
    tag = "participants",
    params(("participant_id" = String, Path, description = "Product participant identifier")),
    responses(
        (status = 200, description = "Product-visible participant projection", body = ParticipantRecord),
        (status = 404, description = "Participant not registered", body = ErrorResponse)
    )
)]
pub async fn get_participant(
    State(state): State<AppState>,
    Path(participant_id): Path<String>,
) -> Result<Json<ParticipantRecord>, ApiError> {
    state
        .agent_registry
        .get_participant(&participant_id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("participant not registered"))
}

#[utoipa::path(
    get,
    path = "/api/v1/participants/{participant_id}/delivery-target",
    operation_id = "get_participant_delivery_target",
    tag = "participants",
    params(("participant_id" = String, Path, description = "Product participant identifier")),
    responses(
        (status = 200, description = "Participant delivery target projection", body = ParticipantDeliveryTargetResponse),
        (status = 404, description = "Participant delivery target not registered", body = ErrorResponse)
    )
)]
pub async fn get_participant_delivery_target(
    State(state): State<AppState>,
    Path(participant_id): Path<String>,
) -> Result<Json<ParticipantDeliveryTargetResponse>, ApiError> {
    state
        .agent_registry
        .participant_delivery_target(&participant_id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("participant delivery target not registered"))
}

#[utoipa::path(
    get,
    path = "/api/v1/chat/participant-selection",
    operation_id = "get_chat_participant_selection",
    tag = "chat",
    responses((status = 200, description = "Current chat participant selection", body = ParticipantSelectionResponse))
)]
pub async fn get_chat_participant_selection(
    State(state): State<AppState>,
) -> Json<ParticipantSelectionResponse> {
    Json(state.agent_registry.participant_selection())
}

#[utoipa::path(
    put,
    path = "/api/v1/chat/participant-selection",
    operation_id = "select_chat_participant",
    tag = "chat",
    request_body = ParticipantSelectionRequest,
    responses(
        (status = 200, description = "Updated chat participant selection", body = ParticipantSelectionResponse),
        (status = 404, description = "Participant not registered", body = ErrorResponse)
    )
)]
pub async fn select_chat_participant(
    State(state): State<AppState>,
    Json(request): Json<ParticipantSelectionRequest>,
) -> Result<Json<ParticipantSelectionResponse>, ApiError> {
    state
        .agent_registry
        .select_participant(&request.participant_id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("participant not registered"))
}

#[utoipa::path(
    get,
    path = "/api/v1/chat/participant-selection/delivery-target",
    operation_id = "get_selected_chat_participant_delivery_target",
    tag = "chat",
    responses(
        (status = 200, description = "Selected chat participant delivery target", body = ParticipantDeliveryTargetResponse),
        (status = 404, description = "Selected participant delivery target not registered", body = ErrorResponse)
    )
)]
pub async fn get_selected_delivery_target(
    State(state): State<AppState>,
) -> Result<Json<ParticipantDeliveryTargetResponse>, ApiError> {
    state
        .agent_registry
        .selected_participant_delivery_target()
        .map(Json)
        .ok_or_else(|| ApiError::not_found("selected participant delivery target not registered"))
}

#[utoipa::path(
    post,
    path = "/api/v1/chat/sessions",
    operation_id = "create_chat_session",
    tag = "chat",
    request_body = ChatSessionCreateRequest,
    responses(
        (status = 200, description = "Created product chat session projection", body = ChatSessionRecord),
        (status = 400, description = "Invalid chat session request", body = ErrorResponse)
    )
)]
pub async fn create_chat_session(
    State(state): State<AppState>,
    Json(request): Json<ChatSessionCreateRequest>,
) -> Result<Json<ChatSessionRecord>, ApiError> {
    state
        .chat_ledger
        .create_session(request)
        .map(Json)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/api/v1/chat/sessions",
    operation_id = "list_chat_sessions",
    tag = "chat",
    responses((status = 200, description = "Product chat sessions", body = ChatSessionListResponse))
)]
pub async fn list_chat_sessions(State(state): State<AppState>) -> Json<ChatSessionListResponse> {
    Json(state.chat_ledger.list_sessions())
}

#[utoipa::path(
    get,
    path = "/api/v1/chat/sessions/{session_id}",
    operation_id = "get_chat_session",
    tag = "chat",
    params(("session_id" = String, Path, description = "Product chat session identifier")),
    responses(
        (status = 200, description = "Product chat session projection", body = ChatSessionRecord),
        (status = 404, description = "Chat session not found", body = ErrorResponse)
    )
)]
pub async fn get_chat_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<ChatSessionRecord>, ApiError> {
    state
        .chat_ledger
        .get_session(&session_id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("chat session not found"))
}

#[utoipa::path(
    post,
    path = "/api/v1/chat/sessions/{session_id}/messages",
    operation_id = "create_chat_message",
    tag = "chat",
    params(("session_id" = String, Path, description = "Product chat session identifier")),
    request_body = ChatMessageCreateRequest,
    responses(
        (status = 200, description = "Created product chat message projection", body = ChatMessageRecord),
        (status = 400, description = "Invalid chat message request", body = ErrorResponse),
        (status = 404, description = "Chat session not found", body = ErrorResponse)
    )
)]
pub async fn create_chat_message(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<ChatMessageCreateRequest>,
) -> Result<Json<ChatMessageRecord>, ApiError> {
    state
        .chat_ledger
        .create_message(&session_id, request)
        .map(Json)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/api/v1/chat/sessions/{session_id}/messages",
    operation_id = "list_chat_messages",
    tag = "chat",
    params(("session_id" = String, Path, description = "Product chat session identifier")),
    responses(
        (status = 200, description = "Product chat message projections", body = ChatMessageListResponse),
        (status = 404, description = "Chat session not found", body = ErrorResponse)
    )
)]
pub async fn list_chat_messages(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<ChatMessageListResponse>, ApiError> {
    state
        .chat_ledger
        .list_messages(&session_id)
        .map(Json)
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/api/v1/chat/sessions/{session_id}/messages/{message_id}/chunks",
    operation_id = "append_chat_message_chunk",
    tag = "chat",
    params(
        ("session_id" = String, Path, description = "Product chat session identifier"),
        ("message_id" = String, Path, description = "Product chat message identifier")
    ),
    request_body = ChatMessageChunkAppendRequest,
    responses(
        (status = 200, description = "Updated product chat message projection", body = ChatMessageRecord),
        (status = 400, description = "Invalid chat message chunk request", body = ErrorResponse),
        (status = 404, description = "Chat message not found", body = ErrorResponse)
    )
)]
pub async fn append_chat_message_chunk(
    State(state): State<AppState>,
    Path((session_id, message_id)): Path<(String, String)>,
    Json(request): Json<ChatMessageChunkAppendRequest>,
) -> Result<Json<ChatMessageRecord>, ApiError> {
    state
        .chat_ledger
        .append_message_chunk(&session_id, &message_id, request)
        .map(Json)
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/api/v1/chat/sessions/{session_id}/messages/{message_id}/finalize",
    operation_id = "finalize_chat_message",
    tag = "chat",
    params(
        ("session_id" = String, Path, description = "Product chat session identifier"),
        ("message_id" = String, Path, description = "Product chat message identifier")
    ),
    request_body = ChatMessageFinalizeRequest,
    responses(
        (status = 200, description = "Finalized product chat message projection", body = ChatMessageRecord),
        (status = 400, description = "Invalid chat message finalize request", body = ErrorResponse),
        (status = 404, description = "Chat message not found", body = ErrorResponse)
    )
)]
pub async fn finalize_chat_message(
    State(state): State<AppState>,
    Path((session_id, message_id)): Path<(String, String)>,
    Json(request): Json<ChatMessageFinalizeRequest>,
) -> Result<Json<ChatMessageRecord>, ApiError> {
    state
        .chat_ledger
        .finalize_message(&session_id, &message_id, request)
        .map(Json)
        .map_err(ApiError::from)
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

impl From<ChatLedgerError> for ApiError {
    fn from(value: ChatLedgerError) -> Self {
        match value {
            ChatLedgerError::InvalidInput(message) => Self::bad_request(message),
            ChatLedgerError::SessionNotFound => Self::not_found("chat session not found"),
            ChatLedgerError::MessageNotFound => Self::not_found("chat message not found"),
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
