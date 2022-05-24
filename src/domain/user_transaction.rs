
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserTransactionId(String);

impl UserTransactionId {
    //check that the supplied transaction is valid

    pub fn validate(trx_id: String) -> Result<UserTransactionId, String> {
    //we assume that the length of a transaction is 64 chars (32 bytes) regardless of the network
        if trx_id.len() == 64 {
            Ok(Self(trx_id))
        } else {
            Err(format!("{} is not a valid transaction id.", trx_id))
        }
    }
}