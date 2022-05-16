use std::str::FromStr;

use actix_web::{web::{self, Json}, HttpResponse};
// use crypto::sha2::Sha256;
use sqlx::PgPool;
use crate::domain::{ Xpubs };
use bdk::bitcoin::{Address, WScriptHash, util::bip32::ExtendedPubKey, hashes::sha256};
use bdk::bitcoin::blockdata::opcodes::all::{OP_PUSHNUM_2, OP_PUSHNUM_3, OP_CHECKMULTISIG};
// use bdk::bitcoin::util::base58::from_check;
use bdk::bitcoin::blockdata::script::Script;
use bdk::bitcoin::hashes::Hash;



//generate 2-0f-3 multisig address from user supplied xpubs
pub async fn gen_multisig_address(x_pubs: web::Json<Xpubs>, pool: web::Data<PgPool>)-> HttpResponse {

    //call the module that generates xpub;
    let server_x_pub : String = "tpubD6NzVbkrYhZ4XH6i354cNhhKD9F8yZjMguBxaKChhxJT328iDwQsJHPSvGS8xXMarT6RVETm8uMX2DC1RjYKbayXnJPGt7bbfj6UpmeLP4A".to_string();
    let x_server_pub_key = ExtendedPubKey::from_str(&server_x_pub.as_str()).unwrap();
    let user_x_pub_key_1 = ExtendedPubKey::from_str(x_pubs.x_pub_1.as_str()).unwrap();
    let user_x_pub_key_2 = ExtendedPubKey::from_str(x_pubs.x_pub_2.as_str()).unwrap();

    let script_from_xpubs = generate_script(user_x_pub_key_1, user_x_pub_key_2, x_server_pub_key).await;

    let script_from_wt_hash = Script::new_v0_wsh(&script_from_xpubs);

    let address = Address::p2wsh(&script_from_wt_hash, bdk::bitcoin::Network::Testnet);
   
    println!("The generated address is: {}", address);

    HttpResponse::Ok().finish()
}



pub async fn generate_script(x_pub : ExtendedPubKey, x_pub_2: ExtendedPubKey,  service_x_pub: ExtendedPubKey) -> WScriptHash {

   
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
