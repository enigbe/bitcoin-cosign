use crate::domain::{UserEmail, UserId, UserTransactionId, AddressData};
use actix_web::{http::StatusCode, web, HttpResponse};
use sqlx::PgPool;

#[derive(Debug, serde::Deserialize)]
pub struct TransactionPayload {
    address: String, //destination address
    amount: u64,     //transaction amount in sats
    transaction_id: String,
    output_index: u16,
    email: String,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TransactionInputResponse {
    pub msg: String,
    pub status: u16,
    pub data: Option<UserEmail>,
}

//endpoint to collect a transaction inputs
pub async fn cosign(
    request: web::Json<TransactionPayload>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    //validate the user supplied email
    let user_email = match UserEmail::parse(request.email.clone()) {
        Ok(email) => email,
        Err(_) => {
            let resp = TransactionInputResponse {
                msg: "Supplied email is invalid".to_string(),
                status: StatusCode::BAD_REQUEST.as_u16(),
                data: None,
            };
            return HttpResponse::BadRequest().json(resp);
        }
    };
    //check that the email exists
    let user_id = match get_user_id(user_email, &pool).await {
        Ok(user_id) => user_id,
        Err(_) => {
            let rsp = TransactionInputResponse {
                msg: "Supplied email does not exist".to_string(),
                status: StatusCode::EXPECTATION_FAILED.as_u16(),
                data: None,
            };
            return HttpResponse::BadRequest().json(rsp);
        }
    };

    //validate the transaction id
   let transaction_id = match UserTransactionId::validate(request.transaction_id.clone()){
       Ok(trx_id) => trx_id,
       Err(_) => {
        let resp = TransactionInputResponse {
            msg: "Supplied transaction ID is invalid".to_string(),
            status: StatusCode::BAD_REQUEST.as_u16(),
            data: None,
        };
        return HttpResponse::BadRequest().json(resp);
   }
};
    //user key pairs
    let user_key_pairs = match get_all_user_key_pairs(user_id.id, &pool).await {
        Ok(user_keys) => user_keys,
        Err(_) => {
         let resp = TransactionInputResponse {
             msg: "User does not have any multisig address with service".to_string(),
             status: StatusCode::BAD_REQUEST.as_u16(),
             data: None,
         };
         return HttpResponse::BadRequest().json(resp);
         }
    };
    
    //check that the given transaction id can be signed by the service



    let suc_res = TransactionInputResponse {
        msg: "User transaction inputs collected".to_string(),
        status: StatusCode::OK.as_u16(),
        data: None,
    };
    
    HttpResponse::Ok().json(suc_res)

}


//get user id
pub async fn get_user_id(user_email: UserEmail, pool: &PgPool) -> Result<UserId, sqlx::Error> {
    let user_id = sqlx::query_as!(
        UserId,
        r#"
            SELECT id FROM users WHERE email = ($1) LIMIT 1
            "#,
        user_email.as_ref(),
    )
    .fetch_one(pool)
    .await?;

    Ok(user_id)
}

//derive all user addresses for the given network

pub async fn get_all_user_key_pairs(user_id:i32, pool: &PgPool) -> Result<Vec<AddressData>, sqlx::Error> {
    let address_data = sqlx::query_as!(
        AddressData, 
        r#"
        SELECT user_id, derivation_path, child_pubk_1, child_pubk_2, service_pubk FROM addresses WHERE user_id=($1)
    "#, user_id)
    .fetch_all(pool)
    .await?;

    Ok(address_data)
}
