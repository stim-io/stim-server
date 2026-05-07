use std::sync::Arc;

use crate::store::{
    AgentRegistryStore, ChatLedgerStore, EndpointRegistryStore, InMemoryAgentRegistryStore,
    InMemoryChatLedgerStore, InMemoryEndpointRegistryStore,
};

#[derive(Clone)]
pub struct AppState {
    pub endpoint_registry: Arc<dyn EndpointRegistryStore>,
    pub agent_registry: Arc<dyn AgentRegistryStore>,
    pub chat_ledger: Arc<dyn ChatLedgerStore>,
}

impl AppState {
    pub fn in_memory() -> Self {
        Self {
            endpoint_registry: Arc::new(InMemoryEndpointRegistryStore::default()),
            agent_registry: Arc::new(InMemoryAgentRegistryStore::default()),
            chat_ledger: Arc::new(InMemoryChatLedgerStore::default()),
        }
    }
}
