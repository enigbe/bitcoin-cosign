use crate::domain::{GenerateAddressData, GenerateAddressResponse, Xpubs};
use crate::utils::keys;
use actix_web::{
    http::header::ContentType,
    web::{self},
    HttpResponse,
};
use bdk::bitcoin::blockdata::opcodes::all::{OP_CHECKMULTISIG, OP_PUSHNUM_2, OP_PUSHNUM_3};
use bdk::bitcoin::blockdata::script::Script;
use bdk::bitcoin::hashes::Hash;
use bdk::bitcoin::Network;
use bdk::keys::bip39::Mnemonic;
use bdk::{
    bitcoin::{hashes::sha256, util::bip32::ExtendedPubKey, Address, WScriptHash},
    keys::ExtendedKey,
};
use sqlx::PgPool;
use std::str::FromStr;

//generate 2-0f-3 multisig address from user supplied xpubs
pub async fn gen_multisig_address(
    x_pubs: web::Json<Xpubs>,
    _pool: web::Data<PgPool>,
) -> HttpResponse {
    //call the module that generates xpub;
    let server_x_pub: String = service_generated_x_pub_key().await;
    let x_server_pub_key = ExtendedPubKey::from_str(&server_x_pub.as_str()).unwrap();
    let user_x_pub_key_1 = ExtendedPubKey::from_str(x_pubs.x_pub_1.as_str()).unwrap();
    let user_x_pub_key_2 = ExtendedPubKey::from_str(x_pubs.x_pub_2.as_str()).unwrap();

    let script_from_xpubs =
        generate_script(user_x_pub_key_1, user_x_pub_key_2, x_server_pub_key).await;

    let script_from_wt_hash = Script::new_v0_wsh(&script_from_xpubs);

    let address = Address::p2wsh(&script_from_wt_hash, Network::Regtest);
    //validate address

    let resp = GenerateAddressResponse::new(
        "Address generated successfully",
        GenerateAddressData::new(&address),
    );

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(&resp)
}

pub async fn generate_script(
    x_pub: ExtendedPubKey,
    x_pub_2: ExtendedPubKey,
    service_x_pub: ExtendedPubKey,
) -> WScriptHash {
    let mut supplied_x_pub_byte = service_x_pub.public_key.to_bytes();
    let mut user_x_pub_1 = x_pub.public_key.to_bytes();
    let mut user_x_pub_2 = x_pub_2.public_key.to_bytes();

    // let p2wsh_hash = vec![OP_PUSHNUM_2.into_u8(), user_x_pub_1[0],  user_x_pub_2[0], supplied_x_pub_byte[0],  OP_PUSHNUM_3.into_u8(),  OP_CHECKMULTISIG.into_u8()];
    // 1. Create an empty vector called script_byte
    let mut script_byte = Vec::new();

    // 2. Push op_2 to thescript_byte
    script_byte.push(OP_PUSHNUM_2.into_u8());

    // 3. Call function push_xpub(script_byte, xpub_arr) that pushes each byte in each xpub to script_byte
    let mut pub_key_bytes = Vec::new();
    pub_key_bytes.append(&mut supplied_x_pub_byte);
    pub_key_bytes.append(&mut user_x_pub_1);
    pub_key_bytes.append(&mut user_x_pub_2);
    // 4. Push op_3 to script_byte

    script_byte.append(&mut pub_key_bytes);
    // 5. Push op_checkmultisig script_byte
    script_byte.push(OP_PUSHNUM_3.into_u8());
    script_byte.push(OP_CHECKMULTISIG.into_u8());

    let hasher = sha256::Hash::hash(&script_byte);

    // create a p2wsh script hash
    let script_hash = WScriptHash::from_slice(&hasher.into_inner());
    // create a script from the p2wsh
    script_hash.unwrap()
    // return the script from the fn
}

// validate generated address
// [TODO]- Implement proper validation of addresses
pub fn validate_address(address: String) -> bool {
    if address.len() > 20 {
        true
    } else {
        false
    }
}

pub async fn service_generated_x_pub_key() -> String {
    let mnemonic: Mnemonic = keys::generate_mnemonic();

    let xtended_key: ExtendedKey = keys::generate_extended_key(&mnemonic);

    keys::generate_base58_xpub(xtended_key, Network::Regtest)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::routes::addresses::generate_script;
    use bitcoin::util::bip32::ExtendedPubKey;

    #[actix_rt::test]
    async fn generate_valid_script() {
        let x_pub_1 =  ExtendedPubKey::from_str(&"tpubD6NzVbkrYhZ4XubsZFiR1YuVq16dxAzt3hWYFtu1sEH7w1LN5gqJnWVtzqZVKrwSej6Pja8tLr4FvyQ9gUuthQ3HVPcfy9cLXhFRjBYMcR9".to_string()).unwrap();
        let x_pub_2 = ExtendedPubKey::from_str(&"tpubD6NzVbkrYhZ4Yb7XhcQBGeovnM5Bk5tHw7Zse5Pm5yC5q4ouAj6dSY7inH1pqQKZptFy9ZQNK7E4iDiG8WaM4pDG3T5KWpjpXjSH3r4RdPy".to_string()).unwrap();
        let x_pub_3 = ExtendedPubKey::from_str(&"tpubD6NzVbkrYhZ4XH6i354cNhhKD9F8yZjMguBxaKChhxJT328iDwQsJHPSvGS8xXMarT6RVETm8uMX2DC1RjYKbayXnJPGt7bbfj6UpmeLP4A".to_string()).unwrap();
        let generated_script = generate_script(x_pub_1, x_pub_2, x_pub_3).await;
        assert_eq!(generated_script.len(), 32);
    }
}
