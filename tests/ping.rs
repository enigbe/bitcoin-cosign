use std::{collections::HashMap, net::TcpListener};

use cosign::start_up::run;
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

/// test user account creation
#[tokio::test]
async fn create_user_returns_201_valid_json_data_test() {
    // 1. Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // 2. Act
    let mut body = HashMap::new();
    body.insert("email".to_string(), "user@email.com".to_string());
    body.insert("password".to_string(), "password".to_string());

    let url = format!("{}/create_user", &address);
    // let body = serde_json::to_string(&body_map).unwrap();

    let response = client.post(&url).json(&body).send().await;

    // 3. Assert
    match response {
        Ok(resp) => {
            assert_eq!(201, resp.status().as_u16())
        }
        Err(e) => {
            println!("{}", e)
        }
    }
}

/// test user creation with bad input data
async fn create_user_returns_400_invalid_json_data_test() {
    todo!()
}

/// Spawn an instance of the application
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
