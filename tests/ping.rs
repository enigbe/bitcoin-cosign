use std::net::TcpListener;

use cosign;
use reqwest;

/// test the ping endpoint to confirm the server is running
#[tokio::test]
async fn ping_test() {
    // 1. Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // 2. Act
    let response = client
        .get(format!("{}/ping", &address))
        .send()
        .await
        .expect("Failed to execute request");

    // 3. Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

/// Spawn an instance of the application
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = cosign::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
