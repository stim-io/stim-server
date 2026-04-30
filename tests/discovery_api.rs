use axum::{body::Body, http::Request};
use stim_server::{app::build_router, state::AppState};
use tower::util::ServiceExt;

const ENDPOINT_ID: &str = "endpoint-a";
const DISCOVERY_ROUTE: &str = "/api/v1/discovery/endpoints/endpoint-a";

#[tokio::test]
async fn registers_then_discovers_endpoint_record() {
    let app = build_router(AppState::in_memory());

    let register_response = app
        .clone()
        .oneshot(register_endpoint_request())
        .await
        .unwrap();

    assert_eq!(register_response.status(), 200);

    let discover_response = app.oneshot(discover_endpoint_request()).await.unwrap();

    assert_eq!(discover_response.status(), 200);
}

fn register_endpoint_request() -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(DISCOVERY_ROUTE)
        .header("content-type", "application/json")
        .body(Body::from(discovery_record_body()))
        .unwrap()
}

fn discover_endpoint_request() -> Request<Body> {
    Request::builder()
        .uri(DISCOVERY_ROUTE)
        .body(Body::empty())
        .unwrap()
}

fn discovery_record_body() -> String {
    format!(
        r#"{{
            "node_id":"node-a",
            "endpoint_declaration":{{
                "endpoint_id":"{ENDPOINT_ID}",
                "node_id":"node-a",
                "display_label":"stim endpoint",
                "endpoint_kind":"stim",
                "supported_protocol_versions":["stim/0.1"],
                "supported_carriers":["p2p"],
                "content_capabilities":["text","dom_fragment"],
                "security_capabilities":["sender_assertion"],
                "declared_features":["registration","discovery"]
            }},
            "carrier_kind":"p2p",
            "addresses":["127.0.0.1:7000"],
            "protocol_versions":["stim/0.1"]
        }}"#
    )
}
