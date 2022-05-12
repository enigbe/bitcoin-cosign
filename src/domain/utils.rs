use bdk::bitcoin::Network::Regtest;
use bdk::blockchain::rpc::{Auth, RpcBlockchain, RpcConfig};
use bdk::blockchain::ConfigurableBlockchain;
use bdk::keys::bip39::{Language, Mnemonic};
use bdk::keys::{DerivableKey, ExtendedKey};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
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
    // let xprv = xkey.into_xprv(Regtest).unwrap();
    // xprv
    xkey
}

// 4. Generate master private key from extended key
pub fn generate_xpriv(xkey: ExtendedKey, network: Network) -> ExtendedPrivKey {
    let xpriv = xkey.into_xprv(network).unwrap();
    xpriv
}

// 5. Generate master public key from master private key
pub fn generate_xpub(xkey: ExtendedKey, network: Network) -> ExtendedPubKey {
    let secp = Secp256k1::new();
    let xpub = xkey.into_xpub(network, &secp);
    xpub
}

// Connect to bitcoind with authenticated user info
pub fn connect_to_bitcoind() {
    let config = RpcConfig {
        url: "127.0.0.1:28332".to_string(),
        auth: Auth::UserPass {
            username: "enigbe".to_string(),
            password: "3cGBL4EZzv9P6ptSkXazVXXS-f08v5ctvHcEchfF62M=".to_string(),
        },
        network: Regtest,
        wallet_name: "eniwallet".to_string(),
        skip_blocks: None,
    };

    let blockchain = RpcBlockchain::from_config(&config);
    println!("{:?}", blockchain);
}

fn main() {
    let mnemonic = generate_mnemonic();
    let seed = generate_seed_from_mnemonic(&mnemonic, "mnemonic");
    let xkey = generate_extended_key(&mnemonic);
    // let xpriv = generate_xpriv(xkey, Regtest);
    let xpub = generate_xpub(xkey, Regtest);

    println!("Seed: {:?}", format!("{:?}", seed));
    println!("chaincode: {:?}", format!("{:?}", xpub.chain_code));
    println!("child_number: {:?}", format!("{:?}", xpub.child_number));
    println!("depth: {:?}", format!("{:?}", xpub.depth));
    // let pubk = xpub.public_key.key;
    println!(
        "public key: {:?}",
        format!("{:?}", xpub.public_key.key.to_string())
    );
}
