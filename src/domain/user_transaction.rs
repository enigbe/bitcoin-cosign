use std::str::FromStr;
use bdk::bitcoincore_rpc::RawTx;
use bitcoin::{hashes::{hex::{FromHex}, sha256, sha256d, Hash}};
use serde::{Deserialize, Serialize};
use bitcoincore_rpc::{bitcoin::{Txid, Error, consensus::Decodable, Transaction}, RpcApi};
use tokio::io::AsyncReadExt;
use crate::domain::NewTransactionPayload;
use crate::routes::transactions::init_rpc_client;
use std::io::Read;



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


#[derive(Serialize, Deserialize, Debug)]
pub struct RawTransaction (String);

impl RawTransaction {

    pub fn extract_txid_from_raw_hex(raw_tx: String) -> sha256d::Hash {

        let txid_digest = sha256d::Hash::hash(&sha256::Hash::hash(&raw_tx.as_bytes()));

        txid_digest
    }

    pub fn convert(raw_tx: String) -> Result<Transaction, String> {

        let rpc_con = init_rpc_client();
        let rpc = rpc_con.unwrap();

        // let raw_txid = Txid::from_hex(&raw_tx.as_str()).unwrap();
        let raw_hash = RawTransaction::extract_txid_from_raw_hex(raw_tx);

        let raw_txid = Txid::from_hash(raw_hash);

        let raw_tx = rpc.get_raw_transaction(&raw_txid, None);
        
        match raw_tx {
            Ok(raw) => Ok(raw),
            Err(error) => {
                Err(format!("Error converting supplied raw tx: {}", error))
            }
        }
    }

}



#[derive(serde::Deserialize)]
pub struct UserRawTransaction {
   pub raw_tx: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BroadCastTrxResponse {
    pub msg: String,
    pub status: u16,
    pub data: Option<Txid>,
}

