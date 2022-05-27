use crate::basetest::spawn_app;
pub use cosign::domain::{GenerateAddressResponse, NewUser, UserEmail, UserPassword};
pub use cosign::routes::users::insert_user;
use std::collections::HashMap;

#[tokio::test]
async fn collect_trx_input_test() {
    // 1. declare vars
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let email = "user@email.com".to_string();
    let address = "tb1qp2266qcdu6rktlyk3ltsuqf2jl6p65kyxjwguqk0rfegkasf3nmqtdte6g".to_string();
    let amount = "10000".to_string();
    let transaction_id = "128fc0e4a5bbf7b229be05ce049706ab9f687e9d2769c2b25188ec0100216b99".to_string();
    let output_index = "1".to_string();
    let password = "password".to_string();
    let xpub1 = "tpubD6NzVbkrYhZ4X4vdoXjofpxTvwJF4Sn9BTRyQsVNXFo9K2qhaUE9e8mCBhYJnCbeoM8CPpj59dpedVB6tZUL8QetjKz4y9zAiFXUrzFbX71".to_string();
    let xpub2 = "tpubD6NzVbkrYhZ4Ya3TiAR7aQaWqBCRKqTS2HPEacgYeFxHUTsxWp71g4A5NFvYm8RBwjbgnSeQBK2Y2jYQXrb5m3Y3qfAyQnvjoGP5UA8691B".to_string();

    let url = format!("{}/gen_multisig_addr", &test_app.address);
    let create_user_url = format!("{}/create_user", &test_app.address);
    let collect_xpub_url = format!("{}/collect_xpubs", &test_app.address);
    let masterkeys_url = format!("{}/masterkeys", &test_app.address);
    let collect_tx_input = format!("{}/collect_trx_input", &test_app.address);


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

    // 1.3 create master keys
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
    assert_eq!("Address generated successfully", resp_body.message);
    assert_eq!(62, resp_body.data.unwrap().address.len());

//1.4 collect inputs
    let mut user_input = HashMap::new();
    user_input.insert("email".to_string(), &email);
    user_input.insert("address".to_string(), &address);
    user_input.insert("transaction_id".to_string(), &transaction_id);
    user_input.insert("amount".to_string(), &amount);
    user_input.insert("output_index".to_string(), &output_index);

    let user_resp = client
        .post(&collect_tx_input)
        .json(&user_input)
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(200, user_resp.status().as_u16());
    
}