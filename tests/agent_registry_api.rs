use axum::{body::Body, http::Request};
use stim_server::{app::build_router, state::AppState};
use tower::util::ServiceExt;

const INSTANCE_ROUTE: &str = "/api/v1/agents/instances/local-santi";
const HEARTBEAT_ROUTE: &str = "/api/v1/agents/instances/local-santi/heartbeat";

#[tokio::test]
async fn registers_and_heartbeats() {
    let app = build_router(AppState::in_memory());

    let register_response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            INSTANCE_ROUTE,
            registration_body("ready"),
        ))
        .await
        .unwrap();
    assert_eq!(register_response.status(), 200);

    let heartbeat_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            HEARTBEAT_ROUTE,
            heartbeat_body("degraded"),
        ))
        .await
        .unwrap();
    assert_eq!(heartbeat_response.status(), 200);
    let body = axum::body::to_bytes(heartbeat_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json.pointer("/agent_id").and_then(|value| value.as_str()),
        Some("santi")
    );
    assert_eq!(
        json.pointer("/instance_id")
            .and_then(|value| value.as_str()),
        Some("local-santi")
    );
    assert_eq!(
        json.pointer("/participant_id")
            .and_then(|value| value.as_str()),
        Some("santi")
    );
    assert_eq!(
        json.pointer("/status").and_then(|value| value.as_str()),
        Some("degraded")
    );
    assert!(
        json.pointer("/last_event_id")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .starts_with("agent.heartbeat-seen-")
    );

    let list_response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/agents/instances")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), 200);
    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json.pointer("/instances/0/instance_id")
            .and_then(|value| value.as_str()),
        Some("local-santi")
    );
}

#[tokio::test]
async fn projects_participant_selection() {
    let app = build_router(AppState::in_memory());

    let register_response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            INSTANCE_ROUTE,
            registration_body("ready"),
        ))
        .await
        .unwrap();
    assert_eq!(register_response.status(), 200);

    let participants_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/participants")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(participants_response.status(), 200);
    let body = axum::body::to_bytes(participants_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json.pointer("/participants/0/participant_id")
            .and_then(|value| value.as_str()),
        Some("santi")
    );
    assert_eq!(
        json.pointer("/participants/0/markers/0")
            .and_then(|value| value.as_str()),
        Some("agent")
    );
    assert_eq!(
        json.pointer("/participants/0/source/instance_id")
            .and_then(|value| value.as_str()),
        Some("local-santi")
    );
    assert_eq!(
        json.pointer("/participants/0/delivery_target/endpoint_id")
            .and_then(|value| value.as_str()),
        Some("endpoint-b")
    );

    let selection_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/chat/participant-selection")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(selection_response.status(), 200);
    let body = axum::body::to_bytes(selection_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json.pointer("/selected_participant_id")
            .and_then(|value| value.as_str()),
        Some("santi")
    );
    assert!(
        json.pointer("/last_event_id")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .starts_with("chat.participant-defaulted-")
    );

    let selection_response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            "/api/v1/chat/participant-selection",
            r#"{"participant_id":"santi"}"#.into(),
        ))
        .await
        .unwrap();
    assert_eq!(selection_response.status(), 200);
    let body = axum::body::to_bytes(selection_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(
        json.pointer("/last_event_id")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .starts_with("chat.participant-selected-")
    );

    let delivery_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/participants/santi/delivery-target")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delivery_response.status(), 200);
    let body = axum::body::to_bytes(delivery_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json.pointer("/delivery_target/endpoint_id")
            .and_then(|value| value.as_str()),
        Some("endpoint-b")
    );

    let selected_delivery_response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/chat/participant-selection/delivery-target")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(selected_delivery_response.status(), 200);
}

#[tokio::test]
async fn heartbeat_requires_registration() {
    let app = build_router(AppState::in_memory());

    let response = app
        .oneshot(json_request(
            "POST",
            HEARTBEAT_ROUTE,
            heartbeat_body("ready"),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

fn json_request(method: &str, uri: &str, body: String) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

fn registration_body(status: &str) -> String {
    format!(
        r#"{{
            "agent_id":"santi",
            "instance_id":"local-santi",
            "participant_id":"santi",
            "delivery_endpoint_id":"endpoint-b",
            "label":"Local Santi",
            "agent_kind":"santi",
            "endpoint":"http://127.0.0.1:18081",
            "profile":"local",
            "capabilities":["santi","provider-probe"],
            "status":"{status}",
            "detail":"registered from stim-agents"
        }}"#
    )
}

fn heartbeat_body(status: &str) -> String {
    format!(
        r#"{{
            "agent_id":"santi",
            "instance_id":"local-santi",
            "participant_id":"santi",
            "delivery_endpoint_id":"endpoint-b",
            "endpoint":"http://127.0.0.1:18081",
            "status":"{status}",
            "detail":"heartbeat from stim-agents"
        }}"#
    )
}
