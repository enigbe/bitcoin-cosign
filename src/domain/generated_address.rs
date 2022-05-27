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
    // Construct address data
    pub fn new(generated_address: &Address) -> GenerateAddressData {
        GenerateAddressData {
            address: generated_address.to_string(),
        }
    }
}

// impl GenerateAddressResponse {
//     //construct response
//     pub fn new(
//         resp_message: &str,
//         resp_data: Option<GenerateAddressData>,
//     ) -> GenerateAddressResponse {
//         GenerateAddressResponse {
//             message: resp_message.to_string(),
//             data: resp_data,
//         }
//     }
// }
