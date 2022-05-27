use crate::basetest::spawn_app;
pub use cosign::domain::{GenerateAddressResponse, NewUser, UserEmail, UserPassword};
pub use cosign::routes::users::insert_user;
use std::collections::HashMap;

#[tokio::test]
async fn generate_address_returns_valid_json_data_test() {
    // 1. Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let email = "user@email.com".to_string();
    let password = "password".to_string();
    let xpub1 = "tpubD6NzVbkrYhZ4X4vdoXjofpxTvwJF4Sn9BTRyQsVNXFo9K2qhaUE9e8mCBhYJnCbeoM8CPpj59dpedVB6tZUL8QetjKz4y9zAiFXUrzFbX71".to_string();
    let xpub2 = "tpubD6NzVbkrYhZ4Ya3TiAR7aQaWqBCRKqTS2HPEacgYeFxHUTsxWp71g4A5NFvYm8RBwjbgnSeQBK2Y2jYQXrb5m3Y3qfAyQnvjoGP5UA8691B".to_string();

    let url = format!("{}/gen_multisig_addr", &test_app.address);
    let create_user_url = format!("{}/create_user", &test_app.address);
    let collect_xpub_url = format!("{}/collect_xpubs", &test_app.address);
    let masterkeys_url = format!("{}/masterkeys", &test_app.address);

    // 1.1 create a user in the database
    let mut create_user_body = HashMap::new();
    create_user_body.insert("email".to_string(), &email);
    create_user_body.insert("password".to_string(), &password);

    let user_resp = client
        .post(&create_user_url)
        .json(&create_user_body)
        .send()
        .await
        .expect("Failed to execute request");

    //1.2 update user with xpub
    assert_eq!(201, user_resp.status().as_u16());
    let mut xpub_body = HashMap::new();
    xpub_body.insert("email".to_string(), email.clone());
    xpub_body.insert("xpub1".to_string(), xpub1.clone());
    xpub_body.insert("xpub2".to_string(), xpub2.clone());

    let collect_xpubs_resp = client
        .patch(&collect_xpub_url)
        .json(&xpub_body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, collect_xpubs_resp.status().as_u16());

    // 1.3 ensure there is a record of service masterkeys
    let network_to_use = option_env!("NETWORK");
    let mut keys_body = HashMap::new();
    keys_body.insert("network", network_to_use);

    let masterkeys_resp = client
        .post(&masterkeys_url)
        .json(&keys_body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, masterkeys_resp.status().as_u16());

    // 1.4 generate address
    let mut body = HashMap::new();
    body.insert("email".to_string(), &email);

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request");

    let resp_body = response.json::<GenerateAddressResponse>().await.unwrap();
    // 3. Assert
    assert_eq!("Address generated successfully", resp_body.msg);
    assert_eq!(62, resp_body.data.unwrap().address.len());
    assert_eq!(201, resp_body.status);
}