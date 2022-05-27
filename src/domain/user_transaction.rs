use bitcoin::{hashes::{hex::{FromHex}, sha256d::Hash}};
use serde::{Deserialize, Serialize};
use bitcoincore_rpc::bitcoin::Txid;


#[derive(Serialize, Deserialize, Debug)]
pub struct UserTransactionId(String);

impl UserTransactionId {
    //check that the supplied transaction is valid

    pub fn validate(trx_id: String) -> Result<UserTransactionId, String> {
        let tx_len:usize = trx_id.len();
    //we assume that the length of a transaction is 64 chars (32 bytes) regardless of the network
        if  tx_len == 64_usize {
            Ok(Self(trx_id))
        } else {
            Err(format!("{} is not a valid transaction id.", trx_id))
        }
    }

    //convert to txid
    pub fn convert_txid(trx_id: String) ->Txid {
        let tx_hash:Hash = Hash::from_hex(&trx_id).unwrap();
        // let trx_hash: Hash = FromHex::from_hex(&trx_id).unwrap();
        let txid: Txid = Txid::try_from(tx_hash).unwrap();
        txid
    }
}