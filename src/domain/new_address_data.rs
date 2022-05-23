
#[derive(serde::Deserialize)]
pub struct AddressData {
    pub user_id: i32,
    pub derivation_path: String,
    pub child_pubk_1: String,
    pub child_pubk_2: String,
    pub service_pubk: String,
}

#[derive(sqlx::Type)]
pub struct NewAddressData {
    pub user_id: i32,
    pub derivation_path: String,
    pub child_pubk_1: String,
    pub child_pubk_2: String,
    pub service_pubk: String,
}


pub struct DerivationIndex {
    pub derivation_path: String
}