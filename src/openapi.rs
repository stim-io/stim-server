use utoipa::OpenApi;

use crate::{handler, schema::ErrorResponse};
use stim_proto::{DiscoveryRecord, EndpointDeclaration};

#[derive(OpenApi)]
#[openapi(
    paths(
        handler::health,
        handler::register_endpoint,
        handler::discover_endpoint,
    ),
    components(schemas(
        ErrorResponse,
        DiscoveryRecord,
        EndpointDeclaration,
        stim_proto::DeliveryTarget,
        stim_proto::MessageEnvelope,
        stim_proto::ProtocolAcknowledgement,
        stim_proto::DeliveryReceipt,
    )),
    tags(
        (name = "health"),
        (name = "discovery"),
    )
)]
pub struct ApiDoc;
