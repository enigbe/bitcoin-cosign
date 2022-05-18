use bdk::keys::bip39::{Language, Mnemonic};
use bdk::keys::{DerivableKey, ExtendedKey};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::base58::check_encode_slice;
use bitcoin::util::bip32::{ChildNumber, Error, ExtendedPrivKey, ExtendedPubKey};
use bitcoin::Network;
use rand::random;

pub enum KeysError {
    XpubError(String),
}

// 1. Generate mnemonic
pub fn generate_mnemonic() -> Mnemonic {
    let english = Language::English;
    let random_number: u8 = random();
    let entropy = [random_number; 32];
    let mnemonic = Mnemonic::from_entropy_in(english, &entropy).unwrap();

    mnemonic
}

// 2. Generate seed from mnemonic
pub fn generate_seed_from_mnemonic(mnemonic: &Mnemonic, passphrase: &str) -> [u8; 64] {
    let passphrase = format!("mnemonic{}", passphrase);
    let seed = mnemonic.to_seed(passphrase.as_str());
    seed
}

// 3. Generate extended key from mnemonic
pub fn generate_extended_key(mnemonic: &Mnemonic) -> ExtendedKey {
    let mnemonic_str = format!("{}", mnemonic);
    let xkey: ExtendedKey = Mnemonic::parse_in(Language::English, mnemonic_str)
        .unwrap()
        .into_extended_key()
        .unwrap();
    xkey
}

// 4.1 Generate master private key from extended key
pub fn generate_xpriv(xkey: ExtendedKey, network: Network) -> ExtendedPrivKey {
    let xpriv = xkey.into_xprv(network).unwrap();
    xpriv
}

// 4.2 Generate base58 encoding of master private key from extended key
pub fn generate_base58_xpriv(xkey: ExtendedKey, network: Network) -> String {
    let xpriv = xkey.into_xprv(network).unwrap();
    check_encode_slice(&xpriv.encode())
}

// 5.1 Generate master public key from master private key
pub fn generate_xpub(xkey: ExtendedKey, network: Network) -> ExtendedPubKey {
    let secp = Secp256k1::new();
    let xpub = xkey.into_xpub(network, &secp);
    xpub
}

// 5.2 Generate base58check-encoded master public key from master private key
pub fn generate_base58_xpub(xkey: ExtendedKey, network: Network) -> String {
    let secp = Secp256k1::new();
    let xpub = xkey.into_xpub(network, &secp);
    check_encode_slice(&xpub.encode())
}

// 6. Generate child public key from extended public key
pub fn generate_child_xpub(xpub: &ExtendedPubKey, index: u32) -> Result<ExtendedPubKey, Error> {
    let secp = Secp256k1::new();
    let child_number = ChildNumber::Normal { index };
    let child_xpub = xpub.ckd_pub(&secp, child_number);

    match child_xpub {
        Ok(xpub) => Ok(xpub),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{
        generate_base58_xpub, generate_child_xpub, generate_extended_key, generate_mnemonic,
        generate_seed_from_mnemonic, generate_xpub,
    };
    use bdk::bitcoin::network::constants::Network::Regtest;
    use bdk::bitcoin::util::base58::check_encode_slice;
    use bdk::keys::bip39::Language;

    #[test]
    fn generate_valid_mnemonic() {
        let mnemonic = generate_mnemonic();
        assert_eq!(mnemonic.language(), Language::English);
        assert_eq!(mnemonic.word_count(), 24);
    }

    #[test]
    fn generate_valid_seed_from_mnemonic() {
        let mnemonic = generate_mnemonic();
        let passphrase = "super-secret";
        let seed = generate_seed_from_mnemonic(&mnemonic, passphrase);

        assert_eq!(64, seed.len());
    }

    #[test]
    fn generate_valid_base58_xpub() {
        let mnemonic = generate_mnemonic();
        let xkey = generate_extended_key(&mnemonic);
        let xpub = generate_base58_xpub(xkey, Regtest);

        assert_eq!(111, xpub.len());
        assert_eq!(mnemonic.word_count(), 24);
    }

    #[test]
    fn generate_valid_child_xpub() {
        // 1. Arrange
        let mnemonic = generate_mnemonic();
        let xkey = generate_extended_key(&mnemonic);
        let xpub = generate_xpub(xkey, Regtest);
        let index = 9;

        // 2. Act
        let child_xpub = generate_child_xpub(&xpub, index).unwrap();

        // 3. Assert
        assert_eq!(111, check_encode_slice(&child_xpub.encode()).len());
    }
}
