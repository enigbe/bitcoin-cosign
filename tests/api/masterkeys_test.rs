use crate::basetest::{spawn_app, MasterKeysResponse};
use reqwest::Client;
use std::collections::HashMap;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn masterkeys_returns_200_for_valid_networks() {
    // 1. Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let networks = vec!["bitcoin", "signet", "regtest", "testnet"];
    let url = format!("{}/masterkeys", &test_app.address);

    for network in networks {
        let mut request_body = HashMap::new();
        request_body.insert("network", network);

        // 2. Act
        let resp = client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .expect("Failed to execute request");

        // 3. Assert
        assert_eq!(200, resp.status().as_u16());
    }
}

#[tokio::test]
async fn masterkeys_returns_400_for_invalid_networks() {
    // 1. Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let invalid_networks = vec!["mainnet", "segnet", "ethernet", "internet"];
    let url = format!("{}/masterkeys", &test_app.address);

    for network in invalid_networks {
        let mut request_body = HashMap::new();
        request_body.insert("network", network);

        // 2. Act
        let resp = client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .expect("Failed to execute request");

        let resp_body = resp.json::<MasterKeysResponse>().await.unwrap();
        // 3. Assert
        assert_eq!(400, resp_body.status);
        assert_eq!(None, resp_body.data);
        assert_eq!(
            "ERROR: Invalid network. Enter one of 'bitcoin', 'regtest', 'testnet', 'signet'.",
            resp_body.msg
        );
    }
}

/// Test the server response when there is a failure to insert
/// generated master keys into the database
#[tokio::test]
async fn masterkeys_returns_500_for_database_insertion_failure() {
    // 1. Arrange
    let mock_server = MockServer::start().await;
    let client = Client::new();

    Mock::given(any())
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&mock_server)
        .await;

    // 2. Act
    let resp = client
        .post(mock_server.uri())
        .send()
        .await
        .expect("Failed to execute request");

    // 3. Assert
    assert_eq!(500, resp.status().as_u16());
}
