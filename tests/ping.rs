use cosign::configuration::{get_configuration, DatabaseSettings};
use cosign::start_up::run;
use reqwest;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::{collections::HashMap, net::TcpListener};
use uuid::Uuid;

pub struct TestApplication {
    pub address: String,
    pub db_pool: PgPool,
}

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
    assert_eq!(Some(0), response.content_length());
}

/// Test user account creation
#[tokio::test]
async fn create_user_returns_201_valid_json_data_test() {
    // 1. Arrange
    let test_app = spawn_app().await;
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
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "user@email.com")
}

/// Test user creation with bad input data
#[tokio::test]
async fn create_user_returns_400_json_fields_present_but_empty_test() {
    // 1. Arrange
    let test_app = spawn_app().await;
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

/// Test that existing users can uoload two xpubs
#[tokio::test]
async fn collect_xpubs_returns_200_for_existing_user() {
    // 1. Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let create_user_url = format!("{}/create_user", &test_app.address);
    let collect_xpub_url = format!("{}/collect_xpub", &test_app.address);

    let mut user_body = HashMap::new();
    user_body.insert("email".to_string(), "user@email.com".to_string());
    user_body.insert("password".to_string(), "password".to_string());

    let mut xpub_body = HashMap::new();
    user_body.insert("email".to_string(), "user@email.com".to_string());
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
async fn collect_xpubs_returns_400_for_nonexistent_users() {
    todo!()
}

/// Spawn an instance of the application
async fn spawn_app() -> TestApplication {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to load configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApplication {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // 1. Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // 2. Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
