use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Xpubs {
    pub id: i32,
    pub xpub1: Option<String>,
    pub xpub2: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UserId {
    pub id: i32,
}
