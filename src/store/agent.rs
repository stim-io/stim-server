use std::{collections::HashMap, sync::RwLock};

use crate::schema::{
    AgentInstanceHeartbeatRequest, AgentInstanceRecord, AgentInstanceRegistrationRequest,
    ParticipantDeliveryTarget, ParticipantDeliveryTargetResponse, ParticipantRecord,
    ParticipantSelectionResponse, ParticipantSource,
};

use super::AgentRegistryStore;

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

fn normalize_optional_id(value: Option<&str>) -> Option<String> {
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
