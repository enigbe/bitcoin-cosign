use actix_web::{http::StatusCode, web, HttpResponse};
use crate::domain::{ UserEmail };


pub struct TransactionPayload {
    address: String, //destination address
    amount: u64, //transaction amount in sats
    transaction_id: String,
    output_index: u16,
    user_email: String,
}

//endpoint to collect a transaction inputs
pub async fn cosign(request : web::Json<TransactionPayload>) {

     //validate the supplied user email
     let user_email = match UserEmail::parse(req.email.clone()) {
        Ok(email) => email,
        Err(_) => {
            let resp = GenerateAddressResponse::new("Supplied user email is invalid", None);
            return HttpResponse::BadRequest().json(resp);
        }
    };
}