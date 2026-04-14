use stim_proto::{DiscoveryRecord, EndpointDeclaration};

/// Minimal compile-path proof that `stim-server` can consume the canonical
/// shared protocol contracts from `stim-proto`.
pub fn registration_shape_example() -> DiscoveryRecord {
    DiscoveryRecord {
        node_id: "node-a".into(),
        endpoint_declaration: EndpointDeclaration {
            endpoint_id: "endpoint-a".into(),
            node_id: "node-a".into(),
            display_label: Some("stim-server proof endpoint".into()),
            endpoint_kind: Some("server".into()),
            supported_protocol_versions: vec![stim_proto::CURRENT_PROTOCOL_VERSION.into()],
            supported_carriers: vec!["p2p".into()],
            content_capabilities: vec!["text".into(), "dom_fragment".into()],
            security_capabilities: vec!["sender_assertion".into()],
            declared_features: vec!["registration".into(), "discovery".into()],
        },
        carrier_kind: "p2p".into(),
        addresses: vec!["127.0.0.1:7000".into()],
        protocol_versions: vec![stim_proto::CURRENT_PROTOCOL_VERSION.into()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_stim_proto_contracts() {
        let record = registration_shape_example();

        assert_eq!(record.node_id, "node-a");
        assert_eq!(record.endpoint_declaration.endpoint_id, "endpoint-a");
        assert_eq!(
            record.protocol_versions,
            vec![stim_proto::CURRENT_PROTOCOL_VERSION]
        );
    }
}
