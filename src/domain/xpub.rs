use std::str::FromStr;

use bitcoin::util::bip32::ExtendedPubKey;

#[derive(Debug, serde::Deserialize)]
pub struct Xpub(String);

impl Xpub {
    /// Returns an instance of Xpub if the user-provided xpub satisfies
    /// validation constraints
    /// Panics otherwise
    pub fn parse(s: String) -> Result<Xpub, String> {
        // Validate xpub string
        // 1. Check length of xpub
        let xpub_len = s.len();

        if xpub_len < 111 {
            Err(format!("{} is not a valid extended public key.", s))
        } else if !s.contains("tpub") {
            // 2. Check starting 'xpub' in string
            Err(format!("{} is not a valid extended public key.", s))
        } else {
            // 3. Convert xpub to extended public key. If successful return Xpub
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
