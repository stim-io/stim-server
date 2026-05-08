mod agent;
mod chat;
mod endpoint;

pub use agent::InMemoryAgentRegistryStore;
pub use chat::InMemoryChatLedgerStore;
pub use endpoint::InMemoryEndpointRegistryStore;

use stim_proto::DiscoveryRecord;

use crate::schema::{
    AgentInstanceHeartbeatRequest, AgentInstanceRecord, AgentInstanceRegistrationRequest,
    ChatMessageChunkAppendRequest, ChatMessageCreateRequest, ChatMessageFinalizeRequest,
    ChatMessageListResponse, ChatMessageRecord, ChatSessionCreateRequest, ChatSessionListResponse,
    ChatSessionRecord, ParticipantDeliveryTargetResponse, ParticipantRecord,
    ParticipantSelectionResponse,
};

pub trait EndpointRegistryStore: Send + Sync {
    fn upsert(&self, record: DiscoveryRecord);
    fn get(&self, endpoint_id: &str) -> Option<DiscoveryRecord>;
}

pub trait AgentRegistryStore: Send + Sync {
    fn register(&self, request: AgentInstanceRegistrationRequest) -> AgentInstanceRecord;
    fn heartbeat(&self, request: AgentInstanceHeartbeatRequest) -> Option<AgentInstanceRecord>;
    fn get(&self, instance_id: &str) -> Option<AgentInstanceRecord>;
    fn list(&self) -> Vec<AgentInstanceRecord>;
    fn list_participants(&self) -> Vec<ParticipantRecord>;
    fn get_participant(&self, participant_id: &str) -> Option<ParticipantRecord>;
    fn participant_delivery_target(
        &self,
        participant_id: &str,
    ) -> Option<ParticipantDeliveryTargetResponse>;
    fn selected_participant_delivery_target(&self) -> Option<ParticipantDeliveryTargetResponse>;
    fn participant_selection(&self) -> ParticipantSelectionResponse;
    fn select_participant(&self, participant_id: &str) -> Option<ParticipantSelectionResponse>;
}

pub trait ChatLedgerStore: Send + Sync {
    fn create_session(
        &self,
        request: ChatSessionCreateRequest,
    ) -> Result<ChatSessionRecord, ChatLedgerError>;
    fn list_sessions(&self) -> ChatSessionListResponse;
    fn get_session(&self, session_id: &str) -> Option<ChatSessionRecord>;
    fn create_message(
        &self,
        session_id: &str,
        request: ChatMessageCreateRequest,
    ) -> Result<ChatMessageRecord, ChatLedgerError>;
    fn list_messages(&self, session_id: &str) -> Result<ChatMessageListResponse, ChatLedgerError>;
    fn append_message_chunk(
        &self,
        session_id: &str,
        message_id: &str,
        request: ChatMessageChunkAppendRequest,
    ) -> Result<ChatMessageRecord, ChatLedgerError>;
    fn finalize_message(
        &self,
        session_id: &str,
        message_id: &str,
        request: ChatMessageFinalizeRequest,
    ) -> Result<ChatMessageRecord, ChatLedgerError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatLedgerError {
    InvalidInput(String),
    SessionNotFound,
    MessageNotFound,
}
