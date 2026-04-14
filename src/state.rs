use std::sync::Arc;

use crate::store::{EndpointRegistryStore, InMemoryEndpointRegistryStore};

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<dyn EndpointRegistryStore>,
}

impl AppState {
    pub fn in_memory() -> Self {
        Self {
            registry: Arc::new(InMemoryEndpointRegistryStore::default()),
        }
    }
}
