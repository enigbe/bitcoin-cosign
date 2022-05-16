use std::str::FromStr;

use bitcoin::util::bip32::ExtendedPubKey;

#[derive(Debug, serde::Deserialize)]
pub struct Xpub(String);

impl Xpub {
    /// Returns an instance of Xpub if the user-provided xpub satisfies
    /// validation constraints
    /// Returns an error otherwise
    pub fn parse(s: String) -> Result<Xpub, String> {
        // Validate xpub string
        // 1. Check length of xpub
        let xpub_len = s.len();

        if xpub_len < 111 {
            Err(format!("{} is not a valid extended public key.", s))
        } else {
            // 2. Convert xpub to extended public key. If successful return Xpub
            let xpub = ExtendedPubKey::from_str(s.as_str());
            match xpub {
                Ok(_xpb) => Ok(Self(s)),
                Err(e) => Err(format!(
                    "{} is not a valid extended public key. Error: {}",
                    s, e
                )),
            }
        }
    }
}

impl AsRef<str> for Xpub {
    fn as_ref(&self) -> &str {
        &self.0.as_str()
    }
}