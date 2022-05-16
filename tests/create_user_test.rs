pub mod basetest;

use basetest::base;
use std::collections::HashMap;

/// Test user account creation
#[tokio::test]
async fn create_user_returns_201_valid_json_data_test() {
    // 1. Arrange
    let test_app = base::spawn_app().await;
    let client = reqwest::Client::new();

    // 2. Act
    let mut body = HashMap::new();
    body.insert("email".to_string(), "user@email.com".to_string());
    body.insert("password".to_string(), "password".to_string());

    let url = format!("{}/create_user", &test_app.address);

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request");

    // 3. Assert
    assert_eq!(201, response.status().as_u16());

    let saved = sqlx::query!("SELECT email FROM users",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved user");

    assert_eq!(saved.email, "user@email.com")
}

/// Test user creation with bad input data
#[tokio::test]
async fn create_user_returns_400_json_fields_present_but_empty_test() {
    // 1. Arrange
    let test_app = base::spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (
            HashMap::from([
                ("email".to_string(), "".to_string()),
                ("password".to_string(), "secret".to_string()),
            ]),
            "email cannot be empty",
        ),
        (
            HashMap::from([
                ("email".to_string(), "janedoe@email.com".to_string()),
                ("password".to_string(), "".to_string()),
            ]),
            "password cannot be empty",
        ),
    ];

    let url = format!("{}/create_user", &test_app.address);
    for (invalid_body, error_msg) in test_cases {
        // 2. Act
        let response = client
            .post(&url)
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        // 3. Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API failed with 400 Bad Request because {}.",
            error_msg
        );
    }
}