use bitcoin::Address;


#[derive(serde::Deserialize)]
pub struct AddressData {
    pub user_id: i32,
    pub derivation_path: String,
    pub child_pubk_1: String,
    pub child_pubk_2: String,
    pub service_pubk: String,
    pub address: String,
}

#[derive(serde::Serialize)]
pub struct NewAddressData {
    pub user_id: i32,
    pub derivation_path: String,
    pub child_pubk_1: String,
    pub child_pubk_2: String,
    pub service_pubk: String,
    pub address: Address,
}


pub struct DerivationIndex {
    pub derivation_path: String
}