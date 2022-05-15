use crate::domain::Xpub;

/// Struct that represents the request body from a user
#[derive(serde::Deserialize)]
pub struct CollectXpub {
    pub email: String,
    pub xpub1: String,
    pub xpub2: String,
}

/// UserXpubs type
#[derive(serde::Deserialize, Debug)]
pub struct UserXpubs {
    pub xpub1: Xpub,
    pub xpub2: Xpub,
}

impl TryFrom<CollectXpub> for UserXpubs {
    type Error = String;

    fn try_from(value: CollectXpub) -> Result<Self, Self::Error> {
        let xpub1 = Xpub::parse(value.xpub1)?;
        let xpub2 = Xpub::parse(value.xpub2)?;

        Ok(Self { xpub1, xpub2 })
    }
}
