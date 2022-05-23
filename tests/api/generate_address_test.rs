use crate::basetest::spawn_app;
use std::collections::HashMap;
pub use cosign::domain::{NewUser, UserEmail, UserPassword};
pub use cosign::routes::users::{insert_user};


// #[tokio::test] [TODO]
async fn generate_address_returns_valid_json_data_test() {
    // 1. Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let email = "user@email.com".to_string();
    let password = "password".to_string();
    let xpub1 = "notxD6NzVbkrYhZ4Ya3TiAR7aQaWqBCRKqTS2HPEacgYeFxHUTsxWp71g4A5NFvYm8RBwjbgnSeQBK2Y2jYQXrb5m3Y3qfAyQnvjoGP5UA8691B".to_string();
    let xpub2 = "notxD6NzVbkrYhZ4Ya3TiAR7aQaWqBCRKqTS2HPEacgYeFxHUTsxWp71g4A5NFvYm8RBwjbgnSeQBK2Y2jYQXrb5m3Y3qfAyQnvjoGP5UA8691B".to_string();

    let url = format!("{}/gen_multisig_addr", &test_app.address);
    let create_user_url = format!("{}/create_user", &test_app.address);
    let collect_xpub_url = format!("{}/collect_xpubs", &test_app.address);


    // create a user in the database 
    let mut create_user_body = HashMap::new();
    create_user_body.insert("email".to_string(), &email);
    create_user_body.insert("password".to_string(), &password);

    client
        .post(&create_user_url)
        .json(&create_user_body)
        .send()
        .await
        .expect("Failed to execute request");

        //update user with xpub

        let mut xpub_body = HashMap::new();
        xpub_body.insert("email".to_string(), &email);
        xpub_body.insert("xpub1".to_string(), &xpub1);
        xpub_body.insert("xpub2".to_string(), &xpub2);
    
        client
            .patch(&collect_xpub_url)
            .json(&xpub_body)
            .send()
            .await
            .expect("Failed to execute request");


    // generate address 
    let mut body = HashMap::new();
    body.insert("email".to_string(), &email);


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
