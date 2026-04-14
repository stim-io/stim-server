use std::{collections::HashMap, sync::RwLock};

use stim_proto::DiscoveryRecord;

pub trait EndpointRegistryStore: Send + Sync {
    fn upsert(&self, record: DiscoveryRecord);
    fn get(&self, endpoint_id: &str) -> Option<DiscoveryRecord>;
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
