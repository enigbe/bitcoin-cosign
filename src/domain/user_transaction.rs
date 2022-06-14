use std::str::FromStr;
use bitcoin::{hashes::{hex::FromHex}};
use serde::{Deserialize, Serialize};
use bitcoincore_rpc::{RpcApi, bitcoin::{Txid, Transaction, consensus}};
use crate::domain::NewTransactionPayload;
use crate::routes::transactions::init_rpc_client;



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

    pub fn extract_txid_from_raw_hex(raw_tx: String)->Txid  {

        let tx_bytes = Vec::from_hex(&raw_tx).unwrap();
        let tx: Transaction = consensus::encode::deserialize(&tx_bytes).unwrap();
        let txid = tx.txid();
        txid
    }

    pub fn convert(raw_tx: String) -> Result<Transaction, String> {

        let rpc_con = init_rpc_client();
        let rpc = rpc_con.unwrap();

        let tx_id = RawTransaction::extract_txid_from_raw_hex(raw_tx);

        let raw_tx = rpc.get_raw_transaction(&tx_id, None);
        
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



#[cfg(test)]
mod tests {
    use super::*;
    

    #[actix_rt::test]
    async fn test_extract_txid_from_raw_hex() {
        let raw_tx =  "02000000000101faecaca08b7a598cf484ef0d8f6731e33860a5dfb46a12a130e78ac4730111150000000000ffffffff01d8ce052a01000000160014d03bb275e7ea501cf926b458702e1cf0d54e46210247304402205ea676c864c2f3e811f47d767f225b640d7dc47bf4cd4ed8ff1b434ee909216402205f43ce7f44c332a5dd8812edff3aede826877eb17e40c15ed2baea5fc0bd9d2a0121022a369ed941445f85da7703c2e8dbcedc88e767015aa6049e4de238f322868f9b00000000".to_string();

        let tx_hash = RawTransaction::extract_txid_from_raw_hex(raw_tx);
       
        assert_eq!(tx_hash.len(), 32);
    }
}