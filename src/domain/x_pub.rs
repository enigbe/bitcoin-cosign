use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Xpubs {
    pub x_pub_1: String,
    pub x_pub_2: String,
}