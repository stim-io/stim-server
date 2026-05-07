use std::{collections::HashMap, sync::RwLock};

use stim_proto::DiscoveryRecord;

use crate::schema::{
    AgentInstanceHeartbeatRequest, AgentInstanceRecord, AgentInstanceRegistrationRequest,
    ChatMessageChunkAppendRequest, ChatMessageChunkRecord, ChatMessageCreateRequest,
    ChatMessageFinalizeRequest, ChatMessageListResponse, ChatMessageRecord, ChatMessageState,
    ChatSessionCreateRequest, ChatSessionListResponse, ChatSessionRecord, ChatSessionState,
    ParticipantDeliveryTarget, ParticipantDeliveryTargetResponse, ParticipantRecord,
    ParticipantSelectionResponse, ParticipantSource,
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

#[derive(Default)]
pub struct InMemoryEndpointRegistryStore {
    records: RwLock<HashMap<String, DiscoveryRecord>>,
}

impl EndpointRegistryStore for InMemoryEndpointRegistryStore {
    fn upsert(&self, record: DiscoveryRecord) {
        self.records
            .write()
            .unwrap()
            .insert(record.endpoint_declaration.endpoint_id.clone(), record);
    }

    fn get(&self, endpoint_id: &str) -> Option<DiscoveryRecord> {
        self.records.read().unwrap().get(endpoint_id).cloned()
    }
}

#[derive(Default)]
pub struct InMemoryChatLedgerStore {
    state: RwLock<ChatLedgerState>,
}

#[derive(Default)]
struct ChatLedgerState {
    sessions: HashMap<String, ChatSessionRecord>,
    messages_by_session: HashMap<String, Vec<ChatMessageRecord>>,
    next_event_sequence: u64,
    next_session_sequence: u64,
    next_message_sequence: u64,
    next_chunk_sequence: u64,
}

impl ChatLedgerStore for InMemoryChatLedgerStore {
    fn create_session(
        &self,
        request: ChatSessionCreateRequest,
    ) -> Result<ChatSessionRecord, ChatLedgerError> {
        let mut state = self.state.write().unwrap();
        let session_id = normalize_optional_id(request.session_id.as_deref())
            .unwrap_or_else(|| next_generated_session_id(&mut state));
        if state.sessions.contains_key(&session_id) {
            return Err(ChatLedgerError::InvalidInput(
                "session already exists".into(),
            ));
        }

        let now = timestamp_now();
        let event_id = chat_next_event_id(&mut state, "chat.session-created");
        let record = ChatSessionRecord {
            session_id: session_id.clone(),
            title: normalize_optional_text(request.title.as_deref()),
            created_by_participant_id: normalize_optional_id(
                request.created_by_participant_id.as_deref(),
            ),
            state: ChatSessionState::Active,
            created_at: now.clone(),
            updated_at: now,
            last_event_id: event_id,
        };

        state.sessions.insert(session_id.clone(), record.clone());
        state.messages_by_session.insert(session_id, Vec::new());
        Ok(record)
    }

    fn list_sessions(&self) -> ChatSessionListResponse {
        let mut sessions = self
            .state
            .read()
            .unwrap()
            .sessions
            .values()
            .cloned()
            .collect::<Vec<_>>();
        sessions.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then_with(|| left.session_id.cmp(&right.session_id))
        });
        ChatSessionListResponse { sessions }
    }

    fn get_session(&self, session_id: &str) -> Option<ChatSessionRecord> {
        self.state.read().unwrap().sessions.get(session_id).cloned()
    }

    fn create_message(
        &self,
        session_id: &str,
        request: ChatMessageCreateRequest,
    ) -> Result<ChatMessageRecord, ChatLedgerError> {
        let mut state = self.state.write().unwrap();
        if !state.sessions.contains_key(session_id) {
            return Err(ChatLedgerError::SessionNotFound);
        }
        if request.state == ChatMessageState::Redacted {
            return Err(ChatLedgerError::InvalidInput(
                "redacted messages must be produced by a later ledger event".into(),
            ));
        }

        let participant_id = normalize_optional_id(Some(&request.participant_id))
            .ok_or_else(|| ChatLedgerError::InvalidInput("participant_id is required".into()))?;
        let message_id = normalize_optional_id(request.message_id.as_deref())
            .unwrap_or_else(|| next_generated_message_id(&mut state));
        if state
            .messages_by_session
            .get(session_id)
            .is_some_and(|messages| {
                messages
                    .iter()
                    .any(|message| message.message_id == message_id)
            })
        {
            return Err(ChatLedgerError::InvalidInput(
                "message already exists".into(),
            ));
        }

        let now = timestamp_now();
        let event_id = chat_next_event_id(&mut state, "chat.message-created");
        let record = ChatMessageRecord {
            session_id: session_id.to_string(),
            message_id,
            participant_id,
            message_kind: request.message_kind,
            content_kind: request.content_kind,
            state: request.state,
            text: request.initial_text.unwrap_or_default(),
            chunks: Vec::new(),
            version: 1,
            operation_id: normalize_optional_id(request.operation_id.as_deref()),
            correlation_id: normalize_optional_id(request.correlation_id.as_deref()),
            causation_id: normalize_optional_id(request.causation_id.as_deref()),
            failure_detail: None,
            created_at: now.clone(),
            updated_at: now.clone(),
            last_event_id: event_id.clone(),
        };

        state
            .messages_by_session
            .entry(session_id.to_string())
            .or_default()
            .push(record.clone());
        touch_chat_session(&mut state, session_id, now, event_id);
        Ok(record)
    }

    fn list_messages(&self, session_id: &str) -> Result<ChatMessageListResponse, ChatLedgerError> {
        let state = self.state.read().unwrap();
        if !state.sessions.contains_key(session_id) {
            return Err(ChatLedgerError::SessionNotFound);
        }

        Ok(ChatMessageListResponse {
            messages: state
                .messages_by_session
                .get(session_id)
                .cloned()
                .unwrap_or_default(),
        })
    }

    fn append_message_chunk(
        &self,
        session_id: &str,
        message_id: &str,
        request: ChatMessageChunkAppendRequest,
    ) -> Result<ChatMessageRecord, ChatLedgerError> {
        let mut state = self.state.write().unwrap();
        if !state.sessions.contains_key(session_id) {
            return Err(ChatLedgerError::SessionNotFound);
        }
        if request.text.is_empty() {
            return Err(ChatLedgerError::InvalidInput(
                "chunk text must not be empty".into(),
            ));
        }

        let chunk_id = normalize_optional_id(request.chunk_id.as_deref())
            .unwrap_or_else(|| next_generated_chunk_id(&mut state));
        let expected_sequence = {
            let message = chat_message(&state, session_id, message_id)?;
            if is_terminal_message_state(&message.state) {
                return Err(ChatLedgerError::InvalidInput(
                    "terminal messages cannot accept chunks".into(),
                ));
            }
            if message
                .chunks
                .iter()
                .any(|chunk| chunk.chunk_id == chunk_id)
            {
                return Err(ChatLedgerError::InvalidInput("chunk already exists".into()));
            }
            message.chunks.len() as u64 + 1
        };
        let sequence = request.sequence.unwrap_or(expected_sequence);
        if sequence != expected_sequence {
            return Err(ChatLedgerError::InvalidInput(format!(
                "chunk sequence must be {expected_sequence}"
            )));
        }

        let now = timestamp_now();
        let event_id = chat_next_event_id(&mut state, "chat.message-chunk-appended");
        let chunk = ChatMessageChunkRecord {
            chunk_id,
            sequence,
            text: request.text,
            created_at: now.clone(),
            event_id: event_id.clone(),
            correlation_id: normalize_optional_id(request.correlation_id.as_deref()),
            causation_id: normalize_optional_id(request.causation_id.as_deref()),
        };

        let message = chat_message_mut(&mut state, session_id, message_id)?;
        message.state = ChatMessageState::Streaming;
        message.text.push_str(&chunk.text);
        message.chunks.push(chunk);
        message.version += 1;
        message.updated_at = now.clone();
        message.last_event_id = event_id.clone();
        let message = message.clone();

        touch_chat_session(&mut state, session_id, now, event_id);
        Ok(message)
    }

    fn finalize_message(
        &self,
        session_id: &str,
        message_id: &str,
        request: ChatMessageFinalizeRequest,
    ) -> Result<ChatMessageRecord, ChatLedgerError> {
        let mut state = self.state.write().unwrap();
        if !state.sessions.contains_key(session_id) {
            return Err(ChatLedgerError::SessionNotFound);
        }
        if !is_supported_final_message_state(&request.state) {
            return Err(ChatLedgerError::InvalidInput(
                "final message state must be completed or failed".into(),
            ));
        }
        {
            let message = chat_message(&state, session_id, message_id)?;
            if is_terminal_message_state(&message.state) {
                return Err(ChatLedgerError::InvalidInput(
                    "terminal messages cannot be finalized again".into(),
                ));
            }
        }

        let now = timestamp_now();
        let event_id = chat_next_event_id(&mut state, "chat.message-finalized");
        let correlation_id = normalize_optional_id(request.correlation_id.as_deref());
        let causation_id = normalize_optional_id(request.causation_id.as_deref());
        let message = chat_message_mut(&mut state, session_id, message_id)?;
        message.state = request.state;
        message.failure_detail = normalize_optional_text(request.failure_detail.as_deref());
        if message.correlation_id.is_none() {
            message.correlation_id = correlation_id;
        }
        if message.causation_id.is_none() {
            message.causation_id = causation_id;
        }
        message.version += 1;
        message.updated_at = now.clone();
        message.last_event_id = event_id.clone();
        let message = message.clone();

        touch_chat_session(&mut state, session_id, now, event_id);
        Ok(message)
    }
}

#[derive(Default)]
pub struct InMemoryAgentRegistryStore {
    state: RwLock<AgentRegistryState>,
}

#[derive(Default)]
struct AgentRegistryState {
    records: HashMap<String, AgentInstanceRecord>,
    participants: HashMap<String, ParticipantRecord>,
    selected_participant_id: Option<String>,
    selected_participant_event_id: Option<String>,
    next_event_sequence: u64,
}

impl AgentRegistryStore for InMemoryAgentRegistryStore {
    fn register(&self, request: AgentInstanceRegistrationRequest) -> AgentInstanceRecord {
        let mut state = self.state.write().unwrap();
        let now = timestamp_now();
        let participant_id = normalize_optional_id(request.participant_id.as_deref())
            .unwrap_or_else(|| request.agent_id.clone());
        let registered_at = state
            .records
            .get(&request.instance_id)
            .map(|record| record.registered_at.clone())
            .unwrap_or_else(|| now.clone());
        let last_event_id = next_event_id(&mut state, "agent.registered");
        let record = AgentInstanceRecord {
            agent_id: request.agent_id,
            instance_id: request.instance_id,
            participant_id,
            delivery_endpoint_id: normalize_optional_id(request.delivery_endpoint_id.as_deref()),
            label: request.label,
            agent_kind: request.agent_kind,
            endpoint: request.endpoint,
            profile: request.profile,
            capabilities: request.capabilities,
            status: request.status,
            detail: request.detail,
            registered_at,
            last_seen_at: now,
            last_event_id,
        };

        state
            .records
            .insert(record.instance_id.clone(), record.clone());
        upsert_participant_from_agent(&mut state, &record, "participant.projected");
        record
    }

    fn heartbeat(&self, request: AgentInstanceHeartbeatRequest) -> Option<AgentInstanceRecord> {
        let mut state = self.state.write().unwrap();
        if !state.records.contains_key(&request.instance_id) {
            return None;
        }

        let now = timestamp_now();
        let last_event_id = next_event_id(&mut state, "agent.heartbeat-seen");
        let record = state.records.get_mut(&request.instance_id)?;

        record.agent_id = request.agent_id;
        if let Some(participant_id) = normalize_optional_id(request.participant_id.as_deref()) {
            record.participant_id = participant_id;
        }
        if let Some(delivery_endpoint_id) =
            normalize_optional_id(request.delivery_endpoint_id.as_deref())
        {
            record.delivery_endpoint_id = Some(delivery_endpoint_id);
        }
        record.endpoint = request.endpoint;
        record.status = request.status;
        record.detail = request.detail;
        record.last_seen_at = now;
        record.last_event_id = last_event_id;

        let record = record.clone();
        upsert_participant_from_agent(&mut state, &record, "participant.status-seen");

        Some(record)
    }

    fn get(&self, instance_id: &str) -> Option<AgentInstanceRecord> {
        self.state.read().unwrap().records.get(instance_id).cloned()
    }

    fn list(&self) -> Vec<AgentInstanceRecord> {
        let mut records = self
            .state
            .read()
            .unwrap()
            .records
            .values()
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            left.agent_id
                .cmp(&right.agent_id)
                .then_with(|| left.instance_id.cmp(&right.instance_id))
        });
        records
    }

    fn list_participants(&self) -> Vec<ParticipantRecord> {
        let mut participants = self
            .state
            .read()
            .unwrap()
            .participants
            .values()
            .cloned()
            .collect::<Vec<_>>();
        participants.sort_by(|left, right| {
            left.display_label
                .cmp(&right.display_label)
                .then_with(|| left.participant_id.cmp(&right.participant_id))
        });
        participants
    }

    fn get_participant(&self, participant_id: &str) -> Option<ParticipantRecord> {
        self.state
            .read()
            .unwrap()
            .participants
            .get(participant_id)
            .cloned()
    }

    fn participant_delivery_target(
        &self,
        participant_id: &str,
    ) -> Option<ParticipantDeliveryTargetResponse> {
        self.state
            .read()
            .unwrap()
            .participants
            .get(participant_id)
            .and_then(delivery_target_response)
    }

    fn selected_participant_delivery_target(&self) -> Option<ParticipantDeliveryTargetResponse> {
        let state = self.state.read().unwrap();
        state
            .selected_participant_id
            .as_ref()
            .and_then(|participant_id| state.participants.get(participant_id))
            .and_then(delivery_target_response)
    }

    fn participant_selection(&self) -> ParticipantSelectionResponse {
        let state = self.state.read().unwrap();
        selection_response(&state)
    }

    fn select_participant(&self, participant_id: &str) -> Option<ParticipantSelectionResponse> {
        let mut state = self.state.write().unwrap();
        if !state.participants.contains_key(participant_id) {
            return None;
        }

        state.selected_participant_id = Some(participant_id.to_string());
        state.selected_participant_event_id =
            Some(next_event_id(&mut state, "chat.participant-selected"));
        Some(selection_response(&state))
    }
}

fn next_event_id(state: &mut AgentRegistryState, event_type: &str) -> String {
    state.next_event_sequence += 1;
    format!("{}-{}", event_type, state.next_event_sequence)
}

fn upsert_participant_from_agent(
    state: &mut AgentRegistryState,
    record: &AgentInstanceRecord,
    event_type: &str,
) {
    let registered_at = state
        .participants
        .get(&record.participant_id)
        .map(|participant| participant.registered_at.clone())
        .unwrap_or_else(|| record.registered_at.clone());
    let last_event_id = next_event_id(state, event_type);
    let participant = ParticipantRecord {
        participant_id: record.participant_id.clone(),
        display_label: record.label.clone(),
        markers: stable_markers(&["agent", &record.agent_kind]),
        capabilities: stable_strings(record.capabilities.clone()),
        status: record.status.clone().into(),
        source: ParticipantSource {
            source_kind: "agent-instance".into(),
            agent_id: Some(record.agent_id.clone()),
            instance_id: Some(record.instance_id.clone()),
        },
        delivery_target: record.delivery_endpoint_id.as_ref().map(|endpoint_id| {
            ParticipantDeliveryTarget {
                endpoint_id: endpoint_id.clone(),
                address: record.endpoint.clone(),
                carrier_kind: Some("http".into()),
            }
        }),
        detail: record.detail.clone(),
        registered_at,
        last_seen_at: record.last_seen_at.clone(),
        last_event_id,
    };

    state
        .participants
        .insert(participant.participant_id.clone(), participant);
    if state.selected_participant_id.is_none() {
        state.selected_participant_id = Some(record.participant_id.clone());
        state.selected_participant_event_id =
            Some(next_event_id(state, "chat.participant-defaulted"));
    }
}

fn delivery_target_response(
    participant: &ParticipantRecord,
) -> Option<ParticipantDeliveryTargetResponse> {
    Some(ParticipantDeliveryTargetResponse {
        participant_id: participant.participant_id.clone(),
        delivery_target: participant.delivery_target.clone()?,
        last_event_id: participant.last_event_id.clone(),
    })
}

fn selection_response(state: &AgentRegistryState) -> ParticipantSelectionResponse {
    let participant = state
        .selected_participant_id
        .as_ref()
        .and_then(|participant_id| state.participants.get(participant_id))
        .cloned();

    ParticipantSelectionResponse {
        selected_participant_id: state.selected_participant_id.clone(),
        participant,
        last_event_id: state.selected_participant_event_id.clone(),
    }
}

fn chat_message<'a>(
    state: &'a ChatLedgerState,
    session_id: &str,
    message_id: &str,
) -> Result<&'a ChatMessageRecord, ChatLedgerError> {
    state
        .messages_by_session
        .get(session_id)
        .and_then(|messages| {
            messages
                .iter()
                .find(|message| message.message_id == message_id)
        })
        .ok_or(ChatLedgerError::MessageNotFound)
}

fn chat_message_mut<'a>(
    state: &'a mut ChatLedgerState,
    session_id: &str,
    message_id: &str,
) -> Result<&'a mut ChatMessageRecord, ChatLedgerError> {
    state
        .messages_by_session
        .get_mut(session_id)
        .and_then(|messages| {
            messages
                .iter_mut()
                .find(|message| message.message_id == message_id)
        })
        .ok_or(ChatLedgerError::MessageNotFound)
}

fn touch_chat_session(
    state: &mut ChatLedgerState,
    session_id: &str,
    updated_at: String,
    last_event_id: String,
) {
    if let Some(session) = state.sessions.get_mut(session_id) {
        session.updated_at = updated_at;
        session.last_event_id = last_event_id;
    }
}

fn chat_next_event_id(state: &mut ChatLedgerState, event_type: &str) -> String {
    state.next_event_sequence += 1;
    format!("{}-{}", event_type, state.next_event_sequence)
}

fn next_generated_session_id(state: &mut ChatLedgerState) -> String {
    state.next_session_sequence += 1;
    format!("session-{}", state.next_session_sequence)
}

fn next_generated_message_id(state: &mut ChatLedgerState) -> String {
    state.next_message_sequence += 1;
    format!("message-{}", state.next_message_sequence)
}

fn next_generated_chunk_id(state: &mut ChatLedgerState) -> String {
    state.next_chunk_sequence += 1;
    format!("chunk-{}", state.next_chunk_sequence)
}

fn is_terminal_message_state(state: &ChatMessageState) -> bool {
    matches!(
        state,
        ChatMessageState::Completed | ChatMessageState::Failed | ChatMessageState::Redacted
    )
}

fn is_supported_final_message_state(state: &ChatMessageState) -> bool {
    matches!(
        state,
        ChatMessageState::Completed | ChatMessageState::Failed
    )
}

fn normalize_optional_id(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn stable_markers(markers: &[&str]) -> Vec<String> {
    stable_strings(markers.iter().map(|marker| marker.to_string()).collect())
}

fn stable_strings(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        let value = value.trim();
        if !value.is_empty() && !deduped.iter().any(|item| item == value) {
            deduped.push(value.to_string());
        }
    }
    deduped
}

fn timestamp_now() -> String {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch");

    format!("{}-{:03}", duration.as_secs(), duration.subsec_millis())
}
