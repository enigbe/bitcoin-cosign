use std::str::FromStr;

use bitcoincore_rpc::bitcoin::Address;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct UserAddress(String);

impl UserAddress {
    pub fn validate(address: String) -> Result<Address, String> {
        let valid_address = Address::from_str(address.as_str());
        match valid_address {
            Ok(addr) =>  Ok(addr),
            Err(err) => {
                Err(format!("{} is not a valid address: {}", address, err))
            }
        }
    }
}