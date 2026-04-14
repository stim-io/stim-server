pub mod app;
pub mod handler;
pub mod openapi;
pub mod schema;
pub mod state;
pub mod store;

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

    use crate::{app::build_router, state::AppState};

    #[tokio::test]
    async fn registers_then_discovers_endpoint_record() {
        let app = build_router(AppState::in_memory());
        let request_body = r#"{
            "node_id":"node-a",
            "endpoint_declaration":{
                "endpoint_id":"endpoint-a",
                "node_id":"node-a",
                "display_label":"stim endpoint",
                "endpoint_kind":"stim",
                "supported_protocol_versions":["stim/0.1"],
                "supported_carriers":["p2p"],
                "content_capabilities":["text","dom_fragment"],
                "security_capabilities":["sender_assertion"],
                "declared_features":["registration","discovery"]
            },
            "carrier_kind":"p2p",
            "addresses":["127.0.0.1:7000"],
            "protocol_versions":["stim/0.1"]
        }"#;

        let register_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/discovery/endpoints/endpoint-a")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(register_response.status(), 200);

        let discover_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/discovery/endpoints/endpoint-a")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(discover_response.status(), 200);
    }
}
