use crate::domain::{
    DerivationIndex, GenerateAddressData, GenerateAddressResponse, NewAddressData, UserEmail, Xpubs,
};
use crate::routes::masterkeys::MasterKeys;
use crate::utils::keys;
use actix_web::{
    http::header::ContentType,
    web::{self},
    HttpResponse,
};
use bdk::bitcoin::blockdata::opcodes::all::{OP_CHECKMULTISIG, OP_PUSHNUM_2, OP_PUSHNUM_3};
use bdk::bitcoin::blockdata::script::Script;
use bdk::bitcoin::hashes::Hash;
use bdk::bitcoin::{hashes::sha256, util::bip32::ExtendedPubKey, Address, WScriptHash};
use serde::Deserialize;
use sqlx::PgPool;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct RequestEmail {
    email: String,
}

//generate 2-0f-3 multisig address from user supplied xpubs
pub async fn gen_multisig_address(
    req: web::Json<RequestEmail>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    //validate the supplied user email
    let user_email = match UserEmail::parse(req.email.clone()) {
        Ok(email) => email,
        Err(_) => {
            let resp = GenerateAddressResponse::new("Supplied user email is invalid", None);
            return HttpResponse::BadRequest().json(resp);
        }
    };

    //get the user saved data (xpubs)
    let saved_user_data = match get_user_x_pubs(user_email, &pool).await {
        Ok(user_data) => user_data,
        Err(_) => {
            let resp =
                GenerateAddressResponse::new("Supplied user email does not exist", None);
            return HttpResponse::BadRequest().json(resp);
        }
    };

    //derive the user x-pubs from their saved data
    let user_xpubk1 = ExtendedPubKey::from_str(saved_user_data.xpub1.unwrap().as_str()).unwrap();
    let user_xpubk2 = ExtendedPubKey::from_str(saved_user_data.xpub2.unwrap().as_str()).unwrap();

    //get the user last derivation index

    let derivation_index = user_derivation_index(&pool, saved_user_data.id).await;

    let server_x_pub_key = match service_x_pub_key(&pool).await {
        Ok(server_x_pub_key) => server_x_pub_key,
        Err(_error) => {
            let resp = GenerateAddressResponse::new("Error retreiving service keys", None);
            return HttpResponse::ExpectationFailed().json(resp);
        }
    };

    //generate address
    let new_address_data: NewAddressData =
        generate_address(server_x_pub_key, user_xpubk1, user_xpubk2, derivation_index, saved_user_data.id).await;

    if let Ok(_) = insert_address_data(&pool, &new_address_data).await {
        HttpResponse::Created().finish();
    } else {
        // HttpResponse::InternalServerError().finish();
        let resp = GenerateAddressResponse::new(
            "Error inserting generated address for user in the database",
            None,
        );
        return HttpResponse::ExpectationFailed().json(resp);
    }

    let resp = GenerateAddressResponse::new(
        "Address generated successfully",
        Some(GenerateAddressData::new(&new_address_data.address)),
    );

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(&resp)
}

pub async fn user_derivation_index(pool: &PgPool,  user_id: i32,) -> u32 {
    let last_index = get_user_derivation_index(user_id, &pool).await;

    let mut derivation_index: u32 = 0;

    match last_index {
        Ok(last_index) => {
            derivation_index = last_index.derivation_path.parse().unwrap();
            derivation_index += 1;
        }
        Err(_error) => {
            derivation_index += 1;
        }
    }
    derivation_index
}

//generate address from the script hash
pub async fn generate_address(
    server_x_pub_key: ExtendedPubKey,
    user_xpubk1: ExtendedPubKey,
    user_xpubk2: ExtendedPubKey,
    derivation_index: u32,
    user_id: i32,
) -> NewAddressData {
    let service_child_pub_key =
        keys::generate_child_xpub(&server_x_pub_key, derivation_index).unwrap();
    let user_child_pubk1 = keys::generate_child_xpub(&user_xpubk1, derivation_index).unwrap();
    let user_child_pubk2 = keys::generate_child_xpub(&user_xpubk2, derivation_index).unwrap();

    let script_from_xpubs =
        generate_script(user_child_pubk1, user_child_pubk2, service_child_pub_key).await;

    let script_from_wt_hash = Script::new_v0_wsh(&script_from_xpubs);

    let address: Address = Address::p2wsh(&script_from_wt_hash, service_child_pub_key.network);

    // [TODO] validate address

    let new_address_data = NewAddressData {
        user_id,
        derivation_path: derivation_index.to_string(),
        child_pubk_1: user_child_pubk1.to_string(),
        child_pubk_2: user_child_pubk2.to_string(),
        service_pubk: service_child_pub_key.to_string(),
        address
    };

    new_address_data
}

//generate script hash
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
//get the service keys
pub async fn service_x_pub_key(pool: &PgPool) -> Result<ExtendedPubKey, sqlx::Error> {
    let master_keys = get_master_service_keys(&pool).await;

    match master_keys {
        Ok(master_key) => {
            let x_pub_key = ExtendedPubKey::from_str(&master_key.master_xpub).unwrap();
            Ok(x_pub_key)
        }
        Err(error) => {
            println!("{:?}", error);
            Err(error)
        }
    }
}

pub async fn get_user_derivation_index(
    user_id: i32,
    pool: &PgPool,
) -> Result<DerivationIndex, sqlx::Error> {
    let derivation_index = sqlx::query_as!(
        DerivationIndex,
        r#"
        select derivation_path from addresses where user_id=$1 ORDER by derivation_path DESC LIMIT 1
        "#,
        user_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(derivation_index)
}

pub async fn get_user_x_pubs(user_email: UserEmail, pool: &PgPool) -> Result<Xpubs, sqlx::Error> {
    let user_data = sqlx::query_as!(
        Xpubs,
        r#"
            SELECT id, xpub1, xpub2 FROM users WHERE email = ($1)
            "#,
        user_email.as_ref(),
    )
    .fetch_one(pool)
    .await?;

    Ok(user_data)
}

pub async fn get_master_service_keys(pool: &PgPool) -> Result<MasterKeys, sqlx::Error> {
    let network = option_env!("NETWORK");
    let keys = sqlx::query_as!(
        MasterKeys,
        r#"
            SELECT master_xpub, master_xpriv FROM service_keys WHERE network=$1 LIMIT 1
            "#,
        network,
    )
    .fetch_one(pool)
    .await?;

    Ok(keys)
}

/// Insert address related data
pub async fn insert_address_data(
    pool: &PgPool,
    new_address_data: &NewAddressData,
) -> Result<(), sqlx::Error> {
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
    use crate::routes::addresses::{generate_script, get_master_service_keys};
    use bitcoin::util::bip32::ExtendedPubKey;
    use std::str::FromStr;

    #[actix_rt::test]
    async fn generate_valid_script() {
        let x_pub_1 =  ExtendedPubKey::from_str(&"tpubD6NzVbkrYhZ4XubsZFiR1YuVq16dxAzt3hWYFtu1sEH7w1LN5gqJnWVtzqZVKrwSej6Pja8tLr4FvyQ9gUuthQ3HVPcfy9cLXhFRjBYMcR9".to_string()).unwrap();
        let x_pub_2 = ExtendedPubKey::from_str(&"tpubD6NzVbkrYhZ4Yb7XhcQBGeovnM5Bk5tHw7Zse5Pm5yC5q4ouAj6dSY7inH1pqQKZptFy9ZQNK7E4iDiG8WaM4pDG3T5KWpjpXjSH3r4RdPy".to_string()).unwrap();
        let x_pub_3 = ExtendedPubKey::from_str(&"tpubD6NzVbkrYhZ4XH6i354cNhhKD9F8yZjMguBxaKChhxJT328iDwQsJHPSvGS8xXMarT6RVETm8uMX2DC1RjYKbayXnJPGt7bbfj6UpmeLP4A".to_string()).unwrap();
        let generated_script = generate_script(x_pub_1, x_pub_2, x_pub_3).await;
        assert_eq!(generated_script.len(), 32);
    }
}
