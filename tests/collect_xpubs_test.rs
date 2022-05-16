pub mod basetest;

use basetest::base;
use std::collections::HashMap;

/// Test that existing users can upload two xpubs
#[tokio::test]
async fn collect_xpubs_returns_200_for_existing_user() {
    // 1. Arrange
    let test_app = base::spawn_app().await;
    let client = reqwest::Client::new();
    let create_user_url = format!("{}/create_user", &test_app.address);
    let collect_xpub_url = format!("{}/collect_xpubs", &test_app.address);

    let mut user_body = HashMap::new();
    user_body.insert("email".to_string(), "user@email.com".to_string());
    user_body.insert("password".to_string(), "password".to_string());

    let mut xpub_body = HashMap::new();
    xpub_body.insert("email".to_string(), "user@email.com".to_string());
    xpub_body.insert("xpub1".to_string(), "tpubD6NzVbkrYhZ4X4vdoXjofpxTvwJF4Sn9BTRyQsVNXFo9K2qhaUE9e8mCBhYJnCbeoM8CPpj59dpedVB6tZUL8QetjKz4y9zAiFXUrzFbX71".to_string());
    xpub_body.insert("xpub2".to_string(), "tpubD6NzVbkrYhZ4Ya3TiAR7aQaWqBCRKqTS2HPEacgYeFxHUTsxWp71g4A5NFvYm8RBwjbgnSeQBK2Y2jYQXrb5m3Y3qfAyQnvjoGP5UA8691B".to_string());

    // 2. Act
    // 2.1 Save user to DB
    let user_resp = client
        .post(&create_user_url)
        .json(&user_body)
        .send()
        .await
        .expect("Failed to execute request");

    // 2.2 Save valid xpubs to user record
    let xpub_resp = client
        .patch(&collect_xpub_url)
        .json(&xpub_body)
        .send()
        .await
        .expect("Failed to execute request");

    // 3. Assert
    // 3.1 Assert user is saved to DB
    assert_eq!(201, user_resp.status().as_u16());

    let saved_user = sqlx::query!("SELECT email FROM users",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved user");

    assert_eq!(saved_user.email, "user@email.com");
    // 3.2 Assert xpub added to user record
    assert_eq!(200, xpub_resp.status().as_u16());
    let updated_user = sqlx::query!("SELECT email, xpub1, xpub2 FROM users",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch updated user record");

    assert_eq!(updated_user.email, "user@email.com");
    assert_eq!(updated_user.xpub1, Some("tpubD6NzVbkrYhZ4X4vdoXjofpxTvwJF4Sn9BTRyQsVNXFo9K2qhaUE9e8mCBhYJnCbeoM8CPpj59dpedVB6tZUL8QetjKz4y9zAiFXUrzFbX71".to_string()));
    assert_eq!(updated_user.xpub2, Some("tpubD6NzVbkrYhZ4Ya3TiAR7aQaWqBCRKqTS2HPEacgYeFxHUTsxWp71g4A5NFvYm8RBwjbgnSeQBK2Y2jYQXrb5m3Y3qfAyQnvjoGP5UA8691B".to_string()));
}

/// Test that non-existent users cannot upload their xpubs
#[tokio::test]
async fn collect_xpubs_returns_403_for_nonexistent_users() {
    // 1. Arrange
    let test_app = base::spawn_app().await;
    let client = reqwest::Client::new();
    let create_user_url = format!("{}/create_user", &test_app.address);
    let collect_xpub_url = format!("{}/collect_xpubs", &test_app.address);

    let mut user_body = HashMap::new();
    user_body.insert("email".to_string(), "user@email.com".to_string());
    user_body.insert("password".to_string(), "password".to_string());

    let mut xpub_body = HashMap::new();
    xpub_body.insert("email".to_string(), "nouser@email.com".to_string());
    xpub_body.insert("xpub1".to_string(), "tpubD6NzVbkrYhZ4X4vdoXjofpxTvwJF4Sn9BTRyQsVNXFo9K2qhaUE9e8mCBhYJnCbeoM8CPpj59dpedVB6tZUL8QetjKz4y9zAiFXUrzFbX71".to_string());
    xpub_body.insert("xpub2".to_string(), "tpubD6NzVbkrYhZ4Ya3TiAR7aQaWqBCRKqTS2HPEacgYeFxHUTsxWp71g4A5NFvYm8RBwjbgnSeQBK2Y2jYQXrb5m3Y3qfAyQnvjoGP5UA8691B".to_string());

    // 2. Act
    // 2.1 Save user to DB
    let user_resp = client
        .post(&create_user_url)
        .json(&user_body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(201, user_resp.status().as_u16());

    // 2.1 Request to update xpubs for nonexistent user
    let collect_xpub_resp = client
        .patch(&collect_xpub_url)
        .json(&xpub_body)
        .send()
        .await
        .expect("Failed to execute request");

    // 3. Assert
    // 3.1 Assert "user@email.com" user exists in database
    let saved_user = sqlx::query!("SELECT email FROM users",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved user");

    // 3.2 Assert error response from /collect_xpub
    assert_eq!(403, collect_xpub_resp.status().as_u16());

    assert_eq!(saved_user.email, "user@email.com");
}

// Test for invalid xpubs
#[tokio::test]
async fn collect_xpubs_returns_400_for_invalid_xpub() {
    // 1. Arrange
    let test_app = base::spawn_app().await;
    let client = reqwest::Client::new();
    let collect_xpub_url = format!("{}/collect_xpubs", &test_app.address);
    let mut xpub_body = HashMap::new();
    xpub_body.insert("email".to_string(), "nouser@email.com".to_string());
    xpub_body.insert("xpub1".to_string(), "notxD6NzVbkrYhZ4Ya3TiAR7aQaWqBCRKqTS2HPEacgYeFxHUTsxWp71g4A5NFvYm8RBwjbgnSeQBK2Y2jYQXrb5m3Y3qfAyQnvjoGP5UA8691B".to_string());
    xpub_body.insert("xpub2".to_string(), "notxD6NzVbkrYhZ4Ya3TiAR7aQaWqBCRKqTS2HPEacgYeFxHUTsxWp71g4A5NFvYm8RBwjbgnSeQBK2Y2jYQXrb5m3Y3qfAyQnvjoGP5UA8691B".to_string());

    // 2. Act
    let xpub_resp = client
        .patch(&collect_xpub_url)
        .json(&xpub_body)
        .send()
        .await
        .expect("Failed to execute request");

    // 3. Assert
    assert_eq!(400, xpub_resp.status().as_u16());
}