use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatSessionCreateRequest {
    pub session_id: Option<String>,
    pub title: Option<String>,
    pub created_by_participant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatSessionListResponse {
    pub sessions: Vec<ChatSessionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatSessionRecord {
    pub session_id: String,
    pub title: Option<String>,
    pub created_by_participant_id: Option<String>,
    pub state: ChatSessionState,
    pub created_at: String,
    pub updated_at: String,
    pub last_event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ChatSessionState {
    Active,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatMessageCreateRequest {
    pub message_id: Option<String>,
    pub participant_id: String,
    pub message_kind: ChatMessageKind,
    pub content_kind: ChatMessageContentKind,
    pub state: ChatMessageState,
    pub initial_text: Option<String>,
    pub operation_id: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatMessageListResponse {
    pub messages: Vec<ChatMessageRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatMessageRecord {
    pub session_id: String,
    pub message_id: String,
    pub participant_id: String,
    pub message_kind: ChatMessageKind,
    pub content_kind: ChatMessageContentKind,
    pub state: ChatMessageState,
    pub text: String,
    pub chunks: Vec<ChatMessageChunkRecord>,
    pub version: u64,
    pub operation_id: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub failure_detail: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ChatMessageKind {
    User,
    Assistant,
    System,
    Thinking,
    ToolCall,
    ToolResult,
    Compact,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ChatMessageContentKind {
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ChatMessageState {
    Pending,
    Streaming,
    Completed,
    Failed,
    Redacted,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatMessageChunkAppendRequest {
    pub chunk_id: Option<String>,
    pub sequence: Option<u64>,
    pub text: String,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatMessageChunkRecord {
    pub chunk_id: String,
    pub sequence: u64,
    pub text: String,
    pub created_at: String,
    pub event_id: String,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatMessageFinalizeRequest {
    pub state: ChatMessageState,
    pub failure_detail: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentInstanceListResponse {
    pub instances: Vec<AgentInstanceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentInstanceRegistrationRequest {
    pub agent_id: String,
    pub instance_id: String,
    pub participant_id: Option<String>,
    pub delivery_endpoint_id: Option<String>,
    pub label: String,
    pub agent_kind: String,
    pub endpoint: Option<String>,
    pub profile: Option<String>,
    pub capabilities: Vec<String>,
    pub status: AgentInstanceStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentInstanceHeartbeatRequest {
    pub agent_id: String,
    pub instance_id: String,
    pub participant_id: Option<String>,
    pub delivery_endpoint_id: Option<String>,
    pub endpoint: Option<String>,
    pub status: AgentInstanceStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentInstanceRecord {
    pub agent_id: String,
    pub instance_id: String,
    pub participant_id: String,
    pub delivery_endpoint_id: Option<String>,
    pub label: String,
    pub agent_kind: String,
    pub endpoint: Option<String>,
    pub profile: Option<String>,
    pub capabilities: Vec<String>,
    pub status: AgentInstanceStatus,
    pub detail: Option<String>,
    pub registered_at: String,
    pub last_seen_at: String,
    pub last_event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AgentInstanceStatus {
    Ready,
    Degraded,
    Unreachable,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParticipantListResponse {
    pub participants: Vec<ParticipantRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParticipantSelectionRequest {
    pub participant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParticipantSelectionResponse {
    pub selected_participant_id: Option<String>,
    pub participant: Option<ParticipantRecord>,
    pub last_event_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParticipantRecord {
    pub participant_id: String,
    pub display_label: String,
    pub markers: Vec<String>,
    pub capabilities: Vec<String>,
    pub status: ParticipantStatus,
    pub source: ParticipantSource,
    pub delivery_target: Option<ParticipantDeliveryTarget>,
    pub detail: Option<String>,
    pub registered_at: String,
    pub last_seen_at: String,
    pub last_event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParticipantSource {
    pub source_kind: String,
    pub agent_id: Option<String>,
    pub instance_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParticipantDeliveryTarget {
    pub endpoint_id: String,
    pub address: Option<String>,
    pub carrier_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParticipantDeliveryTargetResponse {
    pub participant_id: String,
    pub delivery_target: ParticipantDeliveryTarget,
    pub last_event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ParticipantStatus {
    Ready,
    Degraded,
    Unreachable,
    Offline,
}

impl From<AgentInstanceStatus> for ParticipantStatus {
    fn from(value: AgentInstanceStatus) -> Self {
        match value {
            AgentInstanceStatus::Ready => Self::Ready,
            AgentInstanceStatus::Degraded => Self::Degraded,
            AgentInstanceStatus::Unreachable => Self::Unreachable,
            AgentInstanceStatus::Offline => Self::Offline,
        }
    }
}
