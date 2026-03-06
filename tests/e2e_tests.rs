//! End-to-end tests for the complete ClipShare system
//! Tests the integration between server and client

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Helper struct to manage test server and client
struct ClipShareTestEnvironment {
    server_addr: String,
    server_token: String,
    shutdown_signal: Arc<AtomicBool>,
}

impl ClipShareTestEnvironment {
    async fn start() -> Self {
        use axum::{Router, routing::{get, post}};
        use clip_server::{auth::auth_middleware, handlers::create_router, ClipboardState};
        use clip_server::models::ClipboardContent;
        use std::net::SocketAddr;
        use std::sync::RwLock;
        use tokio::net::TcpListener;

        let token = "e2e_test_token";
        let clipboard_state: ClipboardState = Arc::new(RwLock::new(None));
        let auth_state = Arc::new(token.to_string());

        let app = create_router()
            .layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
            .with_state(clipboard_state);

        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = TcpListener::bind(addr).await.unwrap();

        let port = listener.local_addr().unwrap().port();
        let server_addr = format!("http://127.0.0.1:{}", port);

        let shutdown_signal = Arc::new(AtomicBool::new(false));

        // Spawn server in background
        let shutdown_clone = shutdown_signal.clone();
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    while !shutdown_clone.load(Ordering::SeqCst) {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                })
                .await
                .unwrap();
        });

        // Give server time to start
        sleep(Duration::from_millis(200)).await;

        ClipShareTestEnvironment {
            server_addr,
            server_token: token.to_string(),
            shutdown_signal,
        }
    }

    fn server_url(&self) -> String {
        format!("{}/clipboard", self.server_addr)
    }

    async fn shutdown(self) {
        self.shutdown_signal.store(true, Ordering::SeqCst);
        sleep(Duration::from_millis(300)).await;
    }
}

#[tokio::test]
async fn e2e_text_content_workflow() {
    let env = ClipShareTestEnvironment::start().await;
    let client = reqwest::Client::new();

    // 1. Client uploads text content to server
    let upload_response = client
        .post(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .json(&serde_json::json!({
            "contentType": "text/plain",
            "data": "E2E Test Content"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(upload_response.status(), 200);

    // 2. Client retrieves text content from server
    let download_response = client
        .get(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .send()
        .await
        .unwrap();

    assert_eq!(download_response.status(), 200);

    let content: serde_json::Value = download_response.json().await.unwrap();
    assert_eq!(content["type"], "text");
    assert_eq!(content["data"], "E2E Test Content");

    env.shutdown().await;
}

#[tokio::test]
async fn e2e_authentication_required() {
    let env = ClipShareTestEnvironment::start().await;
    let client = reqwest::Client::new();

    // Test without authentication - should fail
    let response = client
        .get(&env.server_url())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 401);

    // Test with wrong token - should fail
    let response = client
        .get(&env.server_url())
        .header("Authorization", "Bearer wrong_token")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 401);

    // Test with correct token - should succeed (but no content yet)
    let response = client
        .get(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404); // No content yet

    env.shutdown().await;
}

#[tokio::test]
async fn e2e_content_type_detection() {
    let env = ClipShareTestEnvironment::start().await;
    let client = reqwest::Client::new();

    // Test 1: Text content
    client
        .post(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .json(&serde_json::json!({
            "contentType": "text/plain",
            "data": "Plain text content"
        }))
        .send()
        .await
        .unwrap();

    let response = client
        .get(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .send()
        .await
        .unwrap();

    let content: serde_json::Value = response.json().await.unwrap();
    assert_eq!(content["type"], "text");

    // Test 2: Image content
    let base64_image = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

    client
        .post(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .json(&serde_json::json!({
            "contentType": "image/png",
            "data": base64_image
        }))
        .send()
        .await
        .unwrap();

    let response = client
        .get(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .send()
        .await
        .unwrap();

    let content: serde_json::Value = response.json().await.unwrap();
    assert_eq!(content["type"], "image");
    assert_eq!(content["mimeType"], "image/png");

    // Test 3: File content
    client
        .post(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .json(&serde_json::json!({
            "contentType": "application/json",
            "filename": "test.json",
            "data": "{\"key\": \"value\"}"
        }))
        .send()
        .await
        .unwrap();

    let response = client
        .get(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .send()
        .await
        .unwrap();

    let content: serde_json::Value = response.json().await.unwrap();
    assert_eq!(content["type"], "file");
    assert_eq!(content["mimeType"], "application/json");

    env.shutdown().await;
}

#[tokio::test]
async fn e2e_content_overwrite() {
    let env = ClipShareTestEnvironment::start().await;
    let client = reqwest::Client::new();

    // Upload first content
    client
        .post(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .json(&serde_json::json!({
            "contentType": "text/plain",
            "data": "First"
        }))
        .send()
        .await
        .unwrap();

    // Upload second content (should overwrite)
    client
        .post(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .json(&serde_json::json!({
            "contentType": "text/plain",
            "data": "Second"
        }))
        .send()
        .await
        .unwrap();

    // Verify only second content exists
    let response = client
        .get(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .send()
        .await
        .unwrap();

    let content: serde_json::Value = response.json().await.unwrap();
    assert_eq!(content["data"], "Second");

    env.shutdown().await;
}

#[tokio::test]
async fn e2e_error_handling() {
    let env = ClipShareTestEnvironment::start().await;
    let client = reqwest::Client::new();

    // Test invalid JSON
    let response = client
        .post(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .header("Content-Type", "application/json")
        .body("invalid json")
        .send()
        .await
        .unwrap();

    assert!(response.status().is_client_error() || response.status().is_server_error());

    // Test missing required fields
    let response = client
        .post(&env.server_url())
        .header("Authorization", format!("Bearer {}", env.server_token))
        .json(&serde_json::json!({
            "contentType": "text/plain"
            // missing "data" field
        }))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_client_error() || response.status().is_server_error());

    env.shutdown().await;
}
