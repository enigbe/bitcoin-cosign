use std::str::FromStr;

use crate::domain::{
    TransactionInputResponse, RawTransaction
};
use actix_web::{http::StatusCode, web, HttpResponse};
use bitcoincore_rpc::bitcoin::{Txid, Transaction};
use bitcoincore_rpc::json::TestMempoolAcceptResult;
use bitcoincore_rpc::{RpcApi};
use bitcoincore_rpc::RawTx;
use bitcoincore_rpc::Error;
use crate::routes::transactions::init_rpc_client;

#[derive(serde::Deserialize)]
pub struct SuppliedPsbt {
    raw_tx: String,
}

//endpoint to collect a transaction inputs
pub async fn broadcast_psbt(
    req: web::Json<SuppliedPsbt>,
) -> HttpResponse {
    // validate the supplied raw transaction is signed and finalised
    let raw_tx = RawTransaction::convert(req.raw_tx.clone());

    //testmempool acceptance

    let validate_trx = match test_mempool_acceptance(&raw_tx) {
        Ok(result) => {
            let status = result.into_iter().find(|pty|pty.allowed == true).unwrap();
            if status.allowed {
                // return raw_tx,
            }else {
                let resp = TransactionInputResponse {
                    msg: "Transaction not accepted into mempool".to_string(),
                    status: StatusCode::BAD_REQUEST.as_u16(),
                    data: None,
                };
                return HttpResponse::BadRequest().json(resp);
            }
        },
        Err(error) => {
            let resp = TransactionInputResponse {
                msg: error.to_string(),
                status: StatusCode::BAD_REQUEST.as_u16(),
                data: None,
            };
            return HttpResponse::BadRequest().json(resp);
        }
    };

    //broadcast the transaction
    match broadcast_rawtrx(raw_tx) {
        Ok(txid) => {
            let succ = TransactionInputResponse {
                msg: "Transaction broadcasted successfully".to_string(),
                status: StatusCode::OK.as_u16(),
                data: None
            };

            return HttpResponse::Ok().json(succ);
        },
        Err(error) => {
            let resp = TransactionInputResponse {
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

    // let trx = rpc.get_raw_transaction(&Txid::from_str(raw_txid.as_str()).unwrap(), None).unwrap();

    let trx_array  = [raw_txid.raw_hex();1];
    let response = rpc.test_mempool_accept(&trx_array);

  response
}

pub fn broadcast_rawtrx(raw_txid:Transaction)->Result<Txid, Error>{

    let rpc_con = init_rpc_client();
    let rpc = rpc_con.unwrap();

    // let raw = RawTx::raw_hex(raw_txid.as_str());
    // let trx = rpc.get_raw_transaction(&Txid::from_str(raw_txid.as_str()).unwrap(), None).unwrap();

    let broadcast_response = rpc.send_raw_transaction(&raw_txid);
    
    broadcast_response
}



