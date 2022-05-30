use bitcoincore_rpc::bitcoin::Address;
use bitcoincore_rpc::bitcoin::Txid;
use serde::{Serialize, Deserialize};
use crate::domain::UserTransactionId;
use crate::domain::UserAddress;
use super::UserEmail;


#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionAmount(u64);

impl TransactionAmount {
    pub fn parse(amount: u64)->Result<u64, String> {
        if amount >= 1000 {
            Ok(amount)
        } else {
            Err(format!("{} does not meet min transaction limit.", amount))
        }
    }   
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TransactionPayload {
   pub address: String, //destination address
   pub amount: String,     //transaction amount in sats
   pub transaction_id: String,
   pub output_index: String,
   pub email: String,
}

#[derive(Debug)]
pub struct NewTransactionPayload {
   pub address: Address,
   pub amount: u64,
   pub transaction_id: Txid,
   pub output_index: u32,
   pub email: UserEmail,
}


impl TryFrom<TransactionPayload> for NewTransactionPayload {
    type Error = String;

    fn try_from(payload: TransactionPayload) -> Result<NewTransactionPayload, Self::Error> {
        let amount  = payload.amount.parse::<u64>().unwrap();
        let index :u32 = payload.output_index.parse::<u32>().unwrap();
        let address = UserAddress::validate(payload.address)?;
        let amount = TransactionAmount::parse(amount)?;
        let transaction_id = UserTransactionId::validate(payload.transaction_id)?;
        let output_index = index;
        let email = UserEmail::parse(payload.email)?;

        Ok(Self { address, amount, transaction_id, output_index, email })
    }
}