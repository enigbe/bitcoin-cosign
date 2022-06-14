use crate::domain::{
    RawTransaction, UserRawTransaction, BroadCastTrxResponse
};
use actix_web::{http::StatusCode, web, HttpResponse};
use bitcoincore_rpc::bitcoin::{Txid, Transaction};
use bitcoincore_rpc::json::TestMempoolAcceptResult;
use bitcoincore_rpc::{RpcApi};
use bitcoincore_rpc::RawTx;
use bitcoincore_rpc::Error;
use crate::routes::transactions::init_rpc_client;


//endpoint to collect a transaction inputs
pub async fn broadcast_psbt(
    req: web::Json<UserRawTransaction>,
) -> HttpResponse {

    //broadcast the transaction
    match broadcast_rawtrx(req.raw_tx.clone()) {
        Ok(txid) => {
            let succ = BroadCastTrxResponse {
                msg: "Transaction broadcasted successfully".to_string(),
                status: StatusCode::OK.as_u16(),
                data: Some(txid)
            };

            return HttpResponse::Ok().json(succ);
        },
        Err(error) => {
            let resp = BroadCastTrxResponse {
                msg: error.to_string(),
                status: StatusCode::BAD_REQUEST.as_u16(),
                data: None,
            };
            return HttpResponse::BadRequest().json(resp);
        }
    }; 

}


// test mempool acceptance
pub fn test_mempool_acceptance(raw_txid:&Transaction) -> Result<Vec<TestMempoolAcceptResult>, Error>
{
    let rpc_con = init_rpc_client();
    let rpc = rpc_con.unwrap();

    let trx_array  = [raw_txid.raw_hex();1];
    let response = rpc.test_mempool_accept(&trx_array);

  response
}

pub fn broadcast_rawtrx(raw_txid: String)->Result<Txid, Error>{

    let rpc_con = init_rpc_client();
    let rpc = rpc_con.unwrap();

    let raw_hex = RawTx::raw_hex(raw_txid.as_str()).raw_hex();

    let broadcast_response = rpc.send_raw_transaction(&*raw_hex);
    
    broadcast_response
}




