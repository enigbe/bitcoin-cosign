use crate::utils::{generate_service_master_keys, keys::ServiceMasterKeys};
use actix_web::{http::StatusCode, web, HttpResponse};
use bdk::bitcoin::Network;
use sqlx::PgPool;
use std::cmp::PartialEq;

#[derive(Debug, serde::Deserialize)]
pub struct RequestNetwork {
    network: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MasterKeysResponse {
    pub msg: String,
    pub status: u16,
    pub data: Option<MasterKeys>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MasterKeys {
    pub master_xpriv: String,
    pub master_xpub: String,
}

/// Return a record of the saved masterkeys if they exist or generate
/// and return service master keys and save a record to the database
/// The request body contains the network: e.g. {"network": "bitcoin"}
///
/// TODO: Authenticate this endpoint so that only internal requests from
/// authenticated staff are authorized to call this endpoint
pub async fn masterkeys(req: web::Json<RequestNetwork>, pool: web::Data<PgPool>) -> HttpResponse {
    let network = match req.network.as_str() {
        "bitcoin" => Network::Bitcoin,
        "regtest" => Network::Regtest,
        "testnet" => Network::Testnet,
        "signet" => Network::Signet,
        _ => {
            let rsp = MasterKeysResponse {
                msg: "ERROR: Invalid network. Enter one of 'bitcoin', 'regtest', 'testnet', 'signet'.".to_string(),
                status: StatusCode::BAD_REQUEST.as_u16(),
                data: None,
            };
            return HttpResponse::BadRequest().json(rsp);
        }
    };

    // 1. Check DB for saved masterkeys
    let existing_masterkeys = find_saved_service_masterkeys(&pool, &network).await;
    match existing_masterkeys {
        Ok(keys) => {
            // 2. Return if they exist

            let rsp_msg = MasterKeysResponse {
                msg: "SUCCESS: Existing masterkeys in database".to_string(),
                status: StatusCode::OK.as_u16(),
                data: Some(keys),
            };
            return HttpResponse::Ok().json(rsp_msg);
        }
        Err(e) => {
            // 3. Generate, save, and return masterkeys if they don't
            println!("ERROR: {}", e);
            // 1. Generate new service keys
            let new_masterkeys = generate_service_master_keys(network);
            // 2. Save them to the database and return
            match insert_service_masterkeys(&pool, &new_masterkeys).await {
                Ok(_) => {
                    let masterkeys = MasterKeys {
                        master_xpriv: new_masterkeys.xpriv,
                        master_xpub: new_masterkeys.xpub,
                    };
                    let rsp_msg = MasterKeysResponse {
                        msg: "SUCCESS: Master extended keys saved to database".to_string(),
                        status: StatusCode::OK.as_u16(),
                        data: Some(masterkeys),
                    };
                    return HttpResponse::Ok().json(rsp_msg);
                }
                Err(e) => {
                    let rsp_msg = MasterKeysResponse {
                        msg: format!("ERROR: Error querying for master keys {:?}", e),
                        status: StatusCode::BAD_REQUEST.as_u16(),
                        data: None,
                    };
                    return HttpResponse::BadRequest().json(rsp_msg);
                }
            }
        }
    }
}

/// Query the service_keys table for a saved record
/// /// ***
/// Parameters:
///     pool (&PgPool): A shared reference to a Postgres connection pool
///     service_masterkeys (&ServiceMasterKeys): A shared reference to a ServiceMasterKeys instance
pub async fn find_saved_service_masterkeys(
    pool: &PgPool,
    network: &Network,
) -> Result<MasterKeys, sqlx::Error> {
    let keys = sqlx::query_as!(
        MasterKeys,
        r#"
        SELECT master_xpriv, master_xpub FROM service_keys 
        WHERE network = ($1)
        "#,
        format!("{}", network),
    )
    .fetch_one(pool)
    .await?;

    Ok(keys)
}

/// Save master keys to service_keys table
/// ***
/// Parameters:
///     pool (&PgPool): A shared reference to a Postgres connection pool
///     service_masterkeys (&ServiceMasterKeys): A shared reference to a ServiceMasterKeys instance
pub async fn insert_service_masterkeys(
    pool: &PgPool,
    service_masterkeys: &ServiceMasterKeys,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO service_keys (mnemonic, network, master_xpriv, master_xpub)
        VALUES ($1, $2, $3, $4)
        "#,
        format!("{}", service_masterkeys.mnemonic),
        format!("{}", service_masterkeys.network),
        service_masterkeys.xpriv,
        service_masterkeys.xpub
    )
    .execute(pool)
    .await
    .map_err(|e| {
        println!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
