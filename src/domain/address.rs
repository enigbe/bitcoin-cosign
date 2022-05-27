use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserAddress(String);

impl UserAddress {
    pub fn validate(address: String) -> Result<UserAddress, String> {
        if address.len() == 62 {
            Ok(Self(address))
        }else {
            Err(format!("Supplied address is invalid"))
        }
    }
}