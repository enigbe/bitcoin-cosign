use bdk::bitcoin::Network::Regtest;
use bdk::keys::bip39::{Language, Mnemonic};
use bdk::keys::{DerivableKey, ExtendedKey};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::base58::check_encode_slice;
use bitcoin::util::bip32::ExtendedPrivKey;
use bitcoin::Network;
use rand::random;

/// Generate parent extended public key
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
    let seed = mnemonic.to_seed(passphrase);
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

// 4. Generate master private key from extended key
pub fn generate_xpriv(xkey: ExtendedKey, network: Network) -> ExtendedPrivKey {
    let xpriv = xkey.into_xprv(network).unwrap();
    xpriv
}

// 5. Generate base58check-encoded master public key from master private key
pub fn generate_base58_xpub(xkey: ExtendedKey, network: Network) -> String {
    let secp = Secp256k1::new();
    let xpub = xkey.into_xpub(network, &secp);
    check_encode_slice(&xpub.encode())
}

#[cfg(test)]
mod tests {
    use crate::domain::generate_mnemonic;
    use bdk::keys::bip39::Language;

    #[test]
    fn generate_valid_mnemonic() {
        let mnemonic = generate_mnemonic();
        assert_eq!(mnemonic.language(), Language::English);
        assert_eq!(mnemonic.word_count(), 24);
    }
}

// fn main() {
//     let mnemonic = generate_mnemonic();
//     let seed = generate_seed_from_mnemonic(&mnemonic, "mnemonic");
//     let xkey = generate_extended_key(&mnemonic);
//     let xpub = generate_base58_xpub(xkey, Regtest);

//     println!("xpub: {}", xpub);
// }
