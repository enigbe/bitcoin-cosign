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

/// test user account creation
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

/// test user creation with bad input data
async fn create_user_returns_400_invalid_json_data_test() {
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
