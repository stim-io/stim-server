use axum::{body::Body, http::Request, response::Response};
use serde_json::Value;
use stim_server::{app::build_router, state::AppState};
use tower::util::ServiceExt;

#[tokio::test]
async fn chat_message_streaming_lifecycle_projects_chunks() {
    let app = build_router(AppState::in_memory());

    let session_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/chat/sessions",
            r#"{
                "session_id":"session-a",
                "title":"Architecture loop",
                "created_by_participant_id":"stim-user"
            }"#,
        ))
        .await
        .unwrap();
    assert_eq!(session_response.status(), 200);
    let json = response_json(session_response).await;
    assert_eq!(
        json.pointer("/session_id").and_then(Value::as_str),
        Some("session-a")
    );
    assert!(
        json.pointer("/last_event_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .starts_with("chat.session-created-")
    );

    let message_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/chat/sessions/session-a/messages",
            r#"{
                "message_id":"message-assistant",
                "participant_id":"santi",
                "message_kind":"assistant",
                "content_kind":"text",
                "state":"pending",
                "operation_id":"operation-a",
                "correlation_id":"correlation-a"
            }"#,
        ))
        .await
        .unwrap();
    assert_eq!(message_response.status(), 200);
    let json = response_json(message_response).await;
    assert_eq!(
        json.pointer("/state").and_then(Value::as_str),
        Some("pending")
    );
    assert_eq!(json.pointer("/text").and_then(Value::as_str), Some(""));
    assert_eq!(json.pointer("/version").and_then(Value::as_u64), Some(1));

    let first_chunk_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/chat/sessions/session-a/messages/message-assistant/chunks",
            r#"{
                "chunk_id":"chunk-a",
                "sequence":1,
                "text":"Streaming "
            }"#,
        ))
        .await
        .unwrap();
    assert_eq!(first_chunk_response.status(), 200);
    let json = response_json(first_chunk_response).await;
    assert_eq!(
        json.pointer("/state").and_then(Value::as_str),
        Some("streaming")
    );
    assert_eq!(
        json.pointer("/text").and_then(Value::as_str),
        Some("Streaming ")
    );
    assert_eq!(json.pointer("/version").and_then(Value::as_u64), Some(2));
    assert_eq!(
        json.pointer("/chunks/0/event_id").and_then(Value::as_str),
        Some("chat.message-chunk-appended-3")
    );

    let second_chunk_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/chat/sessions/session-a/messages/message-assistant/chunks",
            r#"{"text":"message."}"#,
        ))
        .await
        .unwrap();
    assert_eq!(second_chunk_response.status(), 200);
    let json = response_json(second_chunk_response).await;
    assert_eq!(
        json.pointer("/chunks/1/sequence").and_then(Value::as_u64),
        Some(2)
    );
    assert_eq!(
        json.pointer("/text").and_then(Value::as_str),
        Some("Streaming message.")
    );

    let finalize_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/chat/sessions/session-a/messages/message-assistant/finalize",
            r#"{"state":"completed"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(finalize_response.status(), 200);
    let json = response_json(finalize_response).await;
    assert_eq!(
        json.pointer("/state").and_then(Value::as_str),
        Some("completed")
    );
    assert_eq!(json.pointer("/version").and_then(Value::as_u64), Some(4));
    assert!(
        json.pointer("/last_event_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .starts_with("chat.message-finalized-")
    );

    let list_response = app
        .oneshot(empty_request(
            "GET",
            "/api/v1/chat/sessions/session-a/messages",
        ))
        .await
        .unwrap();
    assert_eq!(list_response.status(), 200);
    let json = response_json(list_response).await;
    assert_eq!(
        json.pointer("/messages/0/text").and_then(Value::as_str),
        Some("Streaming message.")
    );
    assert_eq!(
        json.pointer("/messages/0/chunks/0/text")
            .and_then(Value::as_str),
        Some("Streaming ")
    );
    assert_eq!(
        json.pointer("/messages/0/chunks/1/text")
            .and_then(Value::as_str),
        Some("message.")
    );
}

#[tokio::test]
async fn chat_message_chunks_require_live_message() {
    let app = build_router(AppState::in_memory());

    let missing_session_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/chat/sessions/missing/messages/message-a/chunks",
            r#"{"text":"late"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(missing_session_response.status(), 404);

    let session_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/chat/sessions",
            r#"{"session_id":"session-a"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(session_response.status(), 200);

    let message_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/chat/sessions/session-a/messages",
            r#"{
                "message_id":"message-a",
                "participant_id":"stim-user",
                "message_kind":"user",
                "content_kind":"text",
                "state":"completed",
                "initial_text":"done"
            }"#,
        ))
        .await
        .unwrap();
    assert_eq!(message_response.status(), 200);

    let terminal_append_response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/chat/sessions/session-a/messages/message-a/chunks",
            r#"{"text":"too late"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(terminal_append_response.status(), 400);
}

fn json_request(method: &str, uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn empty_request(method: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

async fn response_json(response: Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}
