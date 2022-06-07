use crate::configuration::get_configuration;
use crate::domain::{User, UserEmail};
use crate::routes::addresses::get_user_x_pubs;
use crate::routes::services::find_saved_service_masterkeys;
use bdk::bitcoin::hash_types::Txid;
use bdk::bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey};
use bdk::bitcoin::Network;
use bdk::blockchain::rpc::{Auth as bdk_auth, RpcBlockchain, RpcConfig};
use bdk::blockchain::ConfigurableBlockchain;
use bdk::descriptor::{get_checksum, Segwitv0};
use bdk::keys::DescriptorKey;
use bdk::keys::DescriptorKey::Secret;
use bdk::wallet::{wallet_name_from_descriptor, AddressIndex};
use bdk::Wallet;
use bdk::{bitcoin::blockdata::script::Script, keys::DerivableKey};
use bdk::{sled, SyncOptions};
use bitcoin::util::base58::check_encode_slice;
use bitcoin::util::bip32::{ChildNumber, ExtendedPubKey};
use bitcoin::{network::Address, secp256k1::Secp256k1};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use sqlx::PgPool;
use std::str::FromStr;

pub struct UserDescriptors {
    xpub1: Vec<String>,
    xpub2: Vec<String>,
}

// Generate receive and change output descriptors for servive wallet
pub fn generate_service_output_descriptors(service_xpriv: &ExtendedPrivKey) -> Vec<String> {
    let mut keys = Vec::new();
    let secp = Secp256k1::new();

    for path in ["m/84h/1h/0h/0", "m/84h/1h/0h/1"] {
        let derivation_path = DerivationPath::from_str(path).unwrap();
        let derived_xpriv = service_xpriv.derive_priv(&secp, &derivation_path).unwrap();
        let origin = (service_xpriv.fingerprint(&secp), derivation_path);
        let description_key_4_derived_xpriv: DescriptorKey<Segwitv0> = derived_xpriv
            .into_descriptor_key(Some(origin), DerivationPath::default())
            .unwrap();

        if let Secret(key, _, _) = description_key_4_derived_xpriv {
            let mut desc = "".to_string();
            desc.push_str(&key.to_string());
            keys.push(desc);
        }
    }

    keys
}

// Generate receive and change output descriptors from user-supplied xpubs
pub async fn generate_user_descriptors(email: UserEmail) -> UserDescriptors {
    let configuration = get_configuration().expect("Failed to read configuration");
    let pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let master_xpubs = get_user_x_pubs(email, &pool)
        .await
        .expect("Failed to retrieved user xpubs");
    let secp = Secp256k1::new();

    // For both xpubs:
    // 1. Get its fingerprint, given that it's the master to derive from
    // 2. Give it a standard derivation path: ["m/44h/1h/0h", "m/44h/1h/0h"]
    // 3. Derive child extended public key from it
    // 4. Add a path for further keys derivation

    // 1. xpub_1
    let xpub1_str = master_xpubs.xpub1.unwrap();
    let xpub1 = ExtendedPubKey::from_str(&xpub1_str).unwrap();
    let child_number_1 = ChildNumber::from_normal_idx(1).unwrap();
    let derived_xpub1 = xpub1.ckd_pub(&secp, child_number_1).unwrap();

    let fingerprint1 = derived_xpub1.parent_fingerprint;
    let derived_xpub1_str = check_encode_slice(&derived_xpub1.encode());

    let xpub1_receive_desc = format!("[{}/44'/1'/0']{}/0/*", fingerprint1, derived_xpub1_str);
    let xpub1_change_desc = format!("[{}/44'/1'/0']{}/1/*", fingerprint1, derived_xpub1_str);

    // 1. xpub_2
    let xpub2_str = master_xpubs.xpub2.unwrap();
    let xpub2 = ExtendedPubKey::from_str(&xpub2_str).unwrap();
    let child_number_2 = ChildNumber::from_normal_idx(1).unwrap();
    let derived_xpub2 = xpub2.ckd_pub(&secp, child_number_2).unwrap();

    let fingerprint2 = derived_xpub2.parent_fingerprint;
    let derived_xpub2_str = check_encode_slice(&derived_xpub2.encode());

    let xpub2_receive_desc = format!("[{}/44'/1'/0']{}/0/*", fingerprint2, derived_xpub2_str);
    let xpub2_change_desc = format!("[{}/44'/1'/0']{}/1/*", fingerprint1, derived_xpub2_str);

    let user_desc = UserDescriptors {
        xpub1: vec![xpub1_receive_desc, xpub1_change_desc],
        xpub2: vec![xpub2_receive_desc, xpub2_change_desc],
    };

    user_desc
}

// Compose a 2-of-3 multisig output descriptor for the 3 wallets (1 service, 2 user)
pub fn compose_multisig_output_descriptor(
    mobile_desc: Vec<String>,
    hardware_desc: Vec<String>,
    service_desc: Vec<String>,
) -> (String, String) {
    let receive_desc = format!(
        "wsh(multi(2,{},{},{}))",
        mobile_desc[0], hardware_desc[0], service_desc[0]
    );
    let receive_checksum = get_checksum(receive_desc.as_str()).unwrap();

    let change_desc = format!(
        "wsh(multi(2,{},{},{}))",
        mobile_desc[1], hardware_desc[1], service_desc[1]
    );
    let change_checksum = get_checksum(change_desc.as_str()).unwrap();

    let final_receive_descriptor = format!("{}#{}", receive_desc, receive_checksum);
    let final_change_descriptor = format!("{}#{}", change_desc, change_checksum);

    (final_receive_descriptor, final_change_descriptor)
}

// 1. Create BDK wallet for the service
pub async fn create_service_wallet(email: UserEmail) {
    // 1.1 Query DB for master private and public keys
    let configuration = get_configuration().expect("Failed to read configuration");
    let pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let network = Network::Regtest;
    let masterkeys = find_saved_service_masterkeys(&pool, &network)
        .await
        .expect("Failed to retrieve master keys");

    // 1.1.1 Generate user and service descriptors
    let service_desc = generate_service_output_descriptors(
        &ExtendedPrivKey::from_str(masterkeys.master_xpriv.as_str()).unwrap(),
    );
    let user_desc = generate_user_descriptors(email).await;

    // 1.2 Generate receive and change descriptors for the service wallet
    let (receive, change) =
        compose_multisig_output_descriptor(user_desc.xpub1, user_desc.xpub2, service_desc);

    // 1.3 Sync with bitcoin core
    let rpc_auth = Auth::UserPass(
        "enie".to_string(),
        "hg1hNxSTnVrstb9kVYupJxuqAcSB_P2UhcKF3xA7nk4=".to_string(),
    );
    let core_rpc = Client::new("http://127.0.0.1:28332/wallet/service", rpc_auth).unwrap();

    // 1.4 Set up the bdk wallet
    let secp = Secp256k1::new();
    let wallet_name =
        wallet_name_from_descriptor(&receive, Some(&change), Network::Regtest, &secp).unwrap();
    let datadir = dirs_next::home_dir().unwrap();
    let database = sled::open(datadir).unwrap();
    let db_tree = database.open_tree(wallet_name.clone()).unwrap();
    // RPC configuration
    let auth = bdk_auth::UserPass {
        username: "enie".to_string(),
        password: "hg1hNxSTnVrstb9kVYupJxuqAcSB_P2UhcKF3xA7nk4=".to_string(),
    };
    let url = "http://127.0.0.1:28332".to_string();
    let rpc_config = RpcConfig {
        url,
        auth,
        network: Network::Regtest,
        wallet_name,
        skip_blocks: None,
    };
    let blockchain = RpcBlockchain::from_config(&rpc_config).unwrap();
    let sync_opts = SyncOptions { progress: None };
    let wallet = Wallet::new(&receive, Some(&change), Network::Regtest, db_tree).unwrap();
    // Sync the wallet
    wallet.sync(&blockchain, sync_opts).unwrap();

    // Fetch new address to receive bitcoins
    let address = wallet.get_address(AddressIndex::New).unwrap().address;
    println!("New address: {}", address);
}

/// Create a partially signed bitcoin transaction (PSBT) given the following
/// details as function parameters
/// ***
/// Parameters
///     txid (Txid): The user-supplied transaction ID that they want signed
///     amount (u32): The amount of bitcoins to be sent to the destination address
///     vout (u32): The output index of the transaction being spent. Note that this
///                 UTXO's script_pubkey is the 2-of-3 multisig address
///     address (Address): The destination address
pub fn create_psbt(txid: Txid, vout: u32, amount: u32, address: Address) {
    // let tx = Transaction
    // let psbt = PartiallySignedTransaction::from_unsigned_tx(tx);
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::domain::UserEmail;
    use crate::utils::{
        compose_multisig_output_descriptor, create_service_wallet,
        generate_service_output_descriptors,
    };
    use bdk::bitcoin::PublicKey;
    use bdk::bitcoin::{Address, Network, Script};
    use bitcoin::hashes::hex::ToHex;
    use bitcoin::util::bip32::ExtendedPrivKey;
    use bitcoincore_rpc::RpcApi;
    use std::str::FromStr;

    #[test]
    fn test_pkh_descriptor() {
        let pubkey_str = "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5";
        let pubkey = PublicKey::from_str(pubkey_str).unwrap();
        let pubkey_hash = pubkey.pubkey_hash();
        let script_pubkey = Script::new_p2pkh(&pubkey_hash);
        let address = Address::from_script(&script_pubkey, Network::Bitcoin).unwrap();

        println!("Public Key: {:?}", pubkey);
        println!("Script_PubKey: {:?}", script_pubkey);
        println!("Script_PubKey Hex: {:?}", script_pubkey.to_hex());
        println!("Address: {:?}", address);
    }

    #[tokio::test]
    async fn test_create_service_wallet() {
        let email = UserEmail::parse("user@email.com".to_string()).unwrap();
        create_service_wallet(email).await;
        // println!("{:?}", client.get_blockchain_info());
    }

    #[tokio::test]
    async fn test_generate_output_descriptors() {
        // let masterkeys = create_service_wallet()
        //     .await;
        //     .expect("Failed to retrieve master keys");
        // let serv_desc = generate_service_output_descriptors(
        //     &ExtendedPrivKey::from_str(&masterkeys.master_xpriv).unwrap(),
        // );
        // println!("Service wallet descriptors: {:?}", serv_desc);
        // assert_eq!(2, serv_desc.len());
    }

    #[tokio::test]
    async fn test_compose_multisig_descriptor() {
        // let mobile = vec![
        //     "[57bdf03e/44'/1'/0']tpubDCagn6iMsE6rnnDimFVhuhAUnPANZPCWwiYuZZcGshgGxZ5CaVhuSjqLwPq3MernBKfnhX8Fb6ypcmGqRC3psoXnyAtPxcBBfuyfkHHQAeJ/0/*".to_string(),
        //     "[57bdf03e/44'/1'/0']tpubDCagn6iMsE6rnnDimFVhuhAUnPANZPCWwiYuZZcGshgGxZ5CaVhuSjqLwPq3MernBKfnhX8Fb6ypcmGqRC3psoXnyAtPxcBBfuyfkHHQAeJ/1/*".to_string(),
        // ];
        // let hardware = vec![
        //     "[c3c2a1a4/44'/1'/0']tpubDCcLyCpJonjU7UztMzxGYku9Qz1s73F5VhHCqU8desDBskS6FWgsU2gy4SF22WhYB9wWZT9j9eqRNpHWLbvBEXhs8BRJia3L9hJLCGwY6fd/0/*".to_string(),
        //     "[c3c2a1a4/44'/1'/0']tpubDCcLyCpJonjU7UztMzxGYku9Qz1s73F5VhHCqU8desDBskS6FWgsU2gy4SF22WhYB9wWZT9j9eqRNpHWLbvBEXhs8BRJia3L9hJLCGwY6fd/1/*".to_string(),
        // ];
        // let masterkeys = create_service_wallet()
        //     .await
        //     .expect("Failed to retrieve master keys");
        // let service = generate_service_output_descriptors(
        //     &ExtendedPrivKey::from_str(&masterkeys.master_xpriv).unwrap(),
        // );

        // let desc = compose_multisig_output_descriptor(mobile, hardware, service);

        // println!("Receive: {}", desc.0);
        // println!("Change: {}", desc.1);
    }
}
