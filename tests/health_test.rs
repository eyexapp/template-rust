mod common;

use common::spawn_app;

#[tokio::test]
async fn test_health_returns_200() {
    let app = spawn_app().await;

    let response = app
        .client
        .get(format!("{}/health", app.address))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_health_returns_json() {
    let app = spawn_app().await;

    let response = app
        .client
        .get(format!("{}/health", app.address))
        .send()
        .await
        .expect("Failed to send request");

    let content_type = response
        .headers()
        .get("content-type")
        .expect("Missing content-type header")
        .to_str()
        .unwrap();

    assert!(content_type.contains("application/json"));
}

#[tokio::test]
async fn test_health_body_has_ok_status() {
    let app = spawn_app().await;

    let body: serde_json::Value = app
        .client
        .get(format!("{}/health", app.address))
        .send()
        .await
        .expect("Failed to send request")
        .json()
        .await
        .expect("Failed to parse JSON");

    assert_eq!(body["status"], "ok");
    assert!(body["timestamp"].is_string());
}

#[tokio::test]
async fn test_health_ready_returns_503_without_db() {
    let app = spawn_app().await;

    let response = app
        .client
        .get(format!("{}/health/ready", app.address))
        .send()
        .await
        .expect("Failed to send request");

    // Without a real database, ready should return 503
    assert_eq!(response.status(), 503, "Expected 503 Service Unavailable");
}
