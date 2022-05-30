use crate::basetest::spawn_app;
use reqwest;

/// test the ping endpoint to confirm the server is running
#[tokio::test]
async fn ping_test() {
    // 1. Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    // 2. Act
    let response = client
        .get(format!("{}/ping", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // 3. Assert
    assert!(response.status().is_success());                                      
}
