use serde::{Serialize, Deserialize};
use crate::domain::UserTransactionId;
use crate::domain::UserAddress;
use super::UserEmail;


#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionAmount(u64);

impl TransactionAmount {
    pub fn parse(amount: u64)->Result<Self, String> {
        if amount >= 1000 {
            Ok(Self(amount))
        } else {
            Err(format!("{} does not meet min transaction limit.", amount))
        }
    }   
}



#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TransactionPayload {
   pub address: String, //destination address
   pub amount: u64,     //transaction amount in sats
   pub transaction_id: String,
   pub output_index: u32,
   pub email: String,
}

#[derive(Debug)]
pub struct NewTransactionPayload {
   pub address: UserAddress,
   pub amount: TransactionAmount,
   pub transaction_id: UserTransactionId,
   pub output_index: u32,
   pub email: UserEmail,
}


impl TryFrom<TransactionPayload> for NewTransactionPayload {
    type Error = String;

    fn try_from(payload: TransactionPayload) -> Result<NewTransactionPayload, Self::Error> {
        let address = UserAddress::validate(payload.address)?;
        let amount = TransactionAmount::parse(payload.amount)?;
        let transaction_id = UserTransactionId::validate(payload.transaction_id)?;
        let output_index = payload.output_index;
        let email = UserEmail::parse(payload.email)?;

        Ok(Self { address, amount, transaction_id, output_index, email })
    }
}