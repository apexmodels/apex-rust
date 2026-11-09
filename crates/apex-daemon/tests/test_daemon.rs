use apex_daemon::{create_router, AppState, OllamaClient};
use apex_web::GroundingEngine;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

fn build_test_state() -> AppState {
    std::env::set_var("APEX_DISABLE_AUDIO", "1");
    AppState {
        ollama: Arc::new(OllamaClient::default()),
        grounding: Arc::new(GroundingEngine::default()),
        base_url: "http://localhost:8000".to_string(),
    }
}

#[tokio::test]
async fn test_agent_card_endpoint() {
    let state = build_test_state();
    let app = create_router(state);

    let req = Request::builder()
        .uri("/.well-known/agent-card.json")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "ApexPhi-Think-14B");
    assert_eq!(json["version"], "1.0.0");
    assert!(json["skills"].is_array());
    assert_eq!(json["capabilities"]["streaming"], true);
}

#[tokio::test]
async fn test_voice_speak_and_voices_endpoints() {
    let state = build_test_state();
    let app = create_router(state);

    // 1. Test GET /api/voice/voices
    let voices_req = Request::builder()
        .uri("/api/voice/voices")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let voices_resp = app.clone().oneshot(voices_req).await.unwrap();
    assert_eq!(voices_resp.status(), StatusCode::OK);
    let voices_body = voices_resp.into_body().collect().await.unwrap().to_bytes();
    let voices_json: serde_json::Value = serde_json::from_slice(&voices_body).unwrap();
    assert!(voices_json["voices"].is_array());

    // 2. Test POST /api/voice/speak with <think> tag cleaning
    let speak_payload = serde_json::json!({
        "text": "<think>private calculation steps</think>The final derivation equals 42.",
        "voice": "Samantha"
    });

    let speak_req = Request::builder()
        .uri("/api/voice/speak")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(speak_payload.to_string()))
        .unwrap();

    let speak_resp = app.oneshot(speak_req).await.unwrap();
    assert_eq!(speak_resp.status(), StatusCode::OK);
    let speak_body = speak_resp.into_body().collect().await.unwrap().to_bytes();
    let speak_json: serde_json::Value = serde_json::from_slice(&speak_body).unwrap();

    assert_eq!(speak_json["status"], "speaking");
    // Verify thoughts were completely stripped
    assert_eq!(speak_json["spoken_text"], "The final derivation equals 42.");
}

#[tokio::test]
async fn test_chat_stream_sse_endpoint() {
    let state = build_test_state();
    let app = create_router(state);

    let chat_payload = serde_json::json!({
        "messages": [
            { "role": "user", "content": "Hello Apex!" }
        ],
        "web_search": false,
        "think_depth": "fast"
    });

    let req = Request::builder()
        .uri("/api/chat/stream")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(chat_payload.to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    assert!(content_type.contains("text/event-stream"));
}

#[tokio::test]
async fn test_a2a_rpc_endpoint() {
    let state = build_test_state();
    let app = create_router(state);

    // Test JSON-RPC listTasks or invalid method handling
    let rpc_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tasks/list",
        "params": {},
        "id": "1"
    });

    let req = Request::builder()
        .uri("/rpc")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(rpc_payload.to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    // A2A JSON-RPC router responds with 200 HTTP code containing JSON-RPC result or error
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["id"], "1");
}
