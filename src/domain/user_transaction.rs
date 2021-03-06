use std::str::FromStr;

use bitcoin::{hashes::{hex::{FromHex}, sha256d::Hash}};
use serde::{Deserialize, Serialize};
use bitcoincore_rpc::bitcoin::Txid;
use crate::domain::NewTransactionPayload;


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TransactionInputResponse {
    pub msg: String,
    pub status: u16,
    pub data: Option<NewTransactionPayload>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct UserTransactionId(String);

impl UserTransactionId {
    //check that the supplied transaction is valid

    pub fn validate(trx_id: String) -> Result<Txid, String> {
        let transaction_id = Txid::from_str(trx_id.as_str());
        match transaction_id {
            Ok(tx_id) => Ok(tx_id),
            Err(error) => {
                Err(format!("{} is not a valid transaction id: {}", trx_id, error))
            }
        }
    }

    //convert to txid
    pub fn convert_txid(trx_id: String) ->Result<Txid, String> {
        let txid = Txid::from_str(&trx_id);

        match txid {
            Ok(trx_id) => Ok(trx_id),
            Err(error) => {
                Err(format!("Error converting given transaction id: {}", error))

            }
        }

    }
}