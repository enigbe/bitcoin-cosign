use crate::basetest::spawn_app;
use std::collections::HashMap;


// #[tokio::test]
async fn generate_address_returns_valid_json_data_test() {
    // 1. Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let url = format!("{}/gen_multisig_addr", &test_app.address);

    // 2. Act
    let mut body = HashMap::new();
    body.insert("x_pub_1".to_string(), "tpubD6NzVbkrYhZ4XubsZFiR1YuVq16dxAzt3hWYFtu1sEH7w1LN5gqJnWVtzqZVKrwSej6Pja8tLr4FvyQ9gUuthQ3HVPcfy9cLXhFRjBYMcR9".to_string());
    body.insert("x_pub_2".to_string(), "tpubD6NzVbkrYhZ4Yb7XhcQBGeovnM5Bk5tHw7Zse5Pm5yC5q4ouAj6dSY7inH1pqQKZptFy9ZQNK7E4iDiG8WaM4pDG3T5KWpjpXjSH3r4RdPy".to_string());

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request");

    // 3. Assert
    assert!(response.status().is_success());

    //assert response contains data
    //assert data contains address
    //assert address is not empty
}
