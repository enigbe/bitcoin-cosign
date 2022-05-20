use crate::domain::{GenerateAddressData, GenerateAddressResponse, Xpubs, NewAddressData, DerivationIndex};
use crate::utils::keys;
use bitcoin::secp256k1::key;
use sqlx::PgPool;
use crate::routes::masterkeys::MasterKeys;
use actix_web::{
    http::header::ContentType,
    web::{self},
    HttpResponse,
};
use bdk::bitcoin::blockdata::opcodes::all::{OP_CHECKMULTISIG, OP_PUSHNUM_2, OP_PUSHNUM_3};
use bdk::bitcoin::blockdata::script::Script;
use bdk::bitcoin::hashes::Hash;
use bdk::keys::bip39::Mnemonic;
use bdk::{
    bitcoin::{hashes::sha256, util::bip32::ExtendedPubKey, Address, WScriptHash}
};
use std::str::FromStr;

//generate 2-0f-3 multisig address from user supplied xpubs
pub async fn gen_multisig_address(
    x_pubs: web::Json<Xpubs>,
    pool: web::Data<PgPool>,
)  -> HttpResponse {
    //call the module that generates xpub;
    let server_x_pub: String = service_child_x_pub_key(&pool).await;
    let x_server_pub_key = ExtendedPubKey::from_str(&server_x_pub.as_str()).unwrap();
    let user_x_pub_key_1 = ExtendedPubKey::from_str(x_pubs.x_pub_1.as_str()).unwrap();
    let user_x_pub_key_2 = ExtendedPubKey::from_str(x_pubs.x_pub_2.as_str()).unwrap();

    let last_index = get_db_last_derivation_index(&pool).await;

    let mut derivation_index = 0;

    match last_index {
        Ok(rt_derivation_index) => {
                
         derivation_index = rt_derivation_index.derivation_path.parse().unwrap();

         derivation_index += 1;

        },
        Err(error) => {
            
            derivation_index += 1;
        }
    }

    let child_server_x_pub = keys::generate_child_xpub(&x_server_pub_key, derivation_index).unwrap();
    let child_x_pub_1 = keys::generate_child_xpub(&user_x_pub_key_1, derivation_index).unwrap();
    let child_x_pub_2 = keys::generate_child_xpub(&user_x_pub_key_2, derivation_index).unwrap();

    let script_from_xpubs =
        generate_script(child_x_pub_1, child_x_pub_2, child_server_x_pub).await;

    let script_from_wt_hash = Script::new_v0_wsh(&script_from_xpubs);

    let address = Address::p2wsh(&script_from_wt_hash, x_server_pub_key.network);
    // [TODO] validate address
    // [TODO] replace the user id with a real user id
    let new_address_data  = NewAddressData  {
        user_id: 1, 
        derivation_path: derivation_index.to_string(),
        child_pubk_1: child_x_pub_1.to_string(),
        child_pubk_2: child_x_pub_2.to_string(),
        service_pubk: child_server_x_pub.to_string(),
    };

    if let Ok(_) = insert_keys(&pool, &new_address_data).await {
        HttpResponse::Created().finish();
    } else {
        HttpResponse::InternalServerError().finish();
    }

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
//we might also need to save the network
pub async fn service_child_x_pub_key(pool: &PgPool) -> String {

    let master_keys = get_master_service_keys(&pool).await;

    let x_pub_key = ExtendedPubKey::from_str(&master_keys.unwrap().master_xpub).unwrap();

    let child_number = x_pub_key.child_number.to_string();

    let num : u32 = child_number.parse().unwrap();

    let child_pub_key = keys::generate_child_xpub(&x_pub_key, num);

    child_pub_key.unwrap().to_string()
}

pub async fn get_db_last_derivation_index(pool: &PgPool) -> Result<DerivationIndex, sqlx::Error>{
    let derivation_index = sqlx::query_as!(
        DerivationIndex,
        r#"
        select derivation_path from addresses ORDER by derivation_path DESC LIMIT 1
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(derivation_index)
}

pub async fn get_master_service_keys( pool: &PgPool) -> Result<MasterKeys, sqlx::Error> {
        let keys = sqlx::query_as!(
            MasterKeys,
            r#"
            SELECT master_xpub, master_xpriv FROM service_keys LIMIT 1
            "#
        )
        .fetch_one(pool)
        .await?;
    
        Ok(keys)
}


/// Insert new user to database
/// ***
/// Parameters:
///     pool (&PgPool): A shared reference to a Postgres connection pool
///     new_user (&NewUser): A shared reference to a NewUser instance
pub async fn insert_keys(pool: &PgPool, new_address_data: &NewAddressData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO addresses (user_id, derivation_path, child_pubk_1, child_pubk_2, service_pubk)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        new_address_data.user_id,
        new_address_data.derivation_path,
        new_address_data.child_pubk_1,
        new_address_data.child_pubk_2,
        new_address_data.service_pubk,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        println!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use sqlx::PgPool;
    use crate::routes::addresses::{ generate_script, get_master_service_keys };
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
