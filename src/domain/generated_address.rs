use bdk::bitcoin::Address;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GenerateAddressResponse {
    pub msg: String,
    pub status: u16,
    pub data: Option<GenerateAddressData>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GenerateAddressData {
    pub address: String,
}

impl GenerateAddressData {
    pub fn new(generated_address: &Address) -> GenerateAddressData {
        GenerateAddressData {
            address: generated_address.to_string(),
        }
    }
}

