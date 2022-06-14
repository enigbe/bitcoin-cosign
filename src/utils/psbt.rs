use crate::configuration::get_configuration;
use crate::domain::{User, UserEmail};
use crate::routes::addresses::get_user_x_pubs;
use crate::routes::services::find_saved_service_masterkeys;
use bdk::bitcoin::hash_types::Txid;
use bdk::bitcoin::util::bip32::{ChildNumber, ExtendedPubKey, IntoDerivationPath};
use bdk::bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey};
use bdk::bitcoin::util::psbt::PartiallySignedTransaction;
use bdk::bitcoin::Network;
use bdk::bitcoin::{secp256k1::Secp256k1, Address};
use bdk::blockchain::rpc::{Auth, RpcBlockchain, RpcConfig};
use bdk::blockchain::ConfigurableBlockchain;
use bdk::descriptor::{get_checksum, Segwitv0};
use bdk::keys::DerivableKey;
use bdk::keys::DescriptorKey;
use bdk::keys::DescriptorKey::{Public, Secret};
use bdk::wallet::{wallet_name_from_descriptor, AddressIndex};
use bdk::{sled, SyncOptions};
use bdk::{SignOptions, Wallet};
use bitcoin::Transaction;
use electrsd::bitcoind::{BitcoinD, Error as BitcoinDError};
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
        let descriptor_key_4_derived_xpriv: DescriptorKey<Segwitv0> = derived_xpriv
            .into_descriptor_key(Some(origin), DerivationPath::default())
            .unwrap();

        if let Secret(key, _, _) = descriptor_key_4_derived_xpriv {
            let mut desc = "".to_string();
            desc.push_str(&key.to_string());
            keys.push(desc);
        }
    }

    keys
}

// Helper function to generate receive and change descriptors from xpubs
pub fn _xpub_receive_change_desc(xpub_str: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let secp = Secp256k1::new();

    let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
    for (index, path) in ["m/44h/1h/0h", "m/44h/1h/0h"].into_iter().enumerate() {
        let derivation_path = DerivationPath::from_str(path).unwrap();

        let child_number = ChildNumber::from_normal_idx(index.try_into().unwrap()).unwrap();
        let derived_xpub = xpub.ckd_pub(&secp, child_number).unwrap();
        let origin = (xpub.fingerprint(), derivation_path);
        let descriptor_key_4_derived_xpub: DescriptorKey<Segwitv0> = derived_xpub
            .into_descriptor_key(
                Some(origin),
                DerivationPath::from(vec![ChildNumber::from_normal_idx(
                    index.try_into().unwrap(),
                )
                .unwrap()]),
            )
            .unwrap();

        if let Public(key, _, _) = descriptor_key_4_derived_xpub {
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

    // 1. Descriptors for xpub1 and xpub2
    let xpub1_desc = _xpub_receive_change_desc(master_xpubs.xpub1.unwrap().as_str());
    let xpub2_desc = _xpub_receive_change_desc(master_xpubs.xpub2.unwrap().as_str());

    let user_desc = UserDescriptors {
        xpub1: xpub1_desc,
        xpub2: xpub2_desc,
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

// Get bitcoind
pub fn bitcoind_regtest() -> Result<BitcoinD, BitcoinDError> {
    // Setup bitcoind
    let bitcoind_conf = electrsd::bitcoind::Conf::default();
    let bitcoind_exe =
        electrsd::bitcoind::downloaded_exe_path().expect("We should always have downloaded exe");
    let bitcoind = electrsd::bitcoind::BitcoinD::with_conf(bitcoind_exe, &bitcoind_conf).unwrap();
    let bitcoind_auth = Auth::Cookie {
        file: bitcoind.params.cookie_file.clone(),
    };

    Ok(bitcoind)
}

// Create and sync BDK service wallet to blockchain
pub async fn create_service_wallet(
    email: UserEmail,
    network: Network,
    bitcoind: &BitcoinD,
) -> Wallet<bdk::sled::Tree> {
    // 1 Query DB for master private and public keys
    let configuration = get_configuration().expect("Failed to read configuration");
    let pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let masterkeys = find_saved_service_masterkeys(&pool, &network)
        .await
        .expect("Failed to retrieve master keys");

    // 2 Generate user and service descriptors
    let service_desc = generate_service_output_descriptors(
        &ExtendedPrivKey::from_str(masterkeys.master_xpriv.as_str()).unwrap(),
    );
    let user_desc = generate_user_descriptors(email).await;

    // 2.1 Compose multisig output receive and change descriptors for the service wallet
    let (receive, change) =
        compose_multisig_output_descriptor(user_desc.xpub1, user_desc.xpub2, service_desc);

    // Extra
    // 1. Create wallets for mobile and hardware, and fund them
    // 2. Send bitcoins to service wallet address
    // 3. Create and sign funded PSBT with address
    // 4. Complete the processing of signed PSBT with either of the wallets

    let secp = Secp256k1::new();
    let wallet_name = wallet_name_from_descriptor(&receive, Some(&change), network, &secp).unwrap();

    let mut datadir = dirs_next::home_dir().unwrap();
    datadir.push(".cosign");

    let database = sled::open(datadir).unwrap();
    let db_tree = database.open_tree(wallet_name.clone()).unwrap();

    // RPC configuration
    let url = bitcoind.params.rpc_socket.to_string();
    let rpc_config = RpcConfig {
        url,
        auth: Auth::Cookie {
            file: bitcoind.params.cookie_file.clone(),
        },
        network: Network::Regtest,
        wallet_name,
        skip_blocks: None,
    };

    let blockchain = RpcBlockchain::from_config(&rpc_config).unwrap();
    let wallet = Wallet::new(&receive, Some(&change), network, db_tree).unwrap();

    // Sync the wallet to the background bitcoind blockchain
    wallet.sync(&blockchain, SyncOptions::default()).unwrap();

    wallet
}

// Generate multisig address with internal BDK wallet for a given user with
// provided email
pub async fn multisig_address(wallet: &Wallet<bdk::sled::Tree>) -> Address {
    // Generate new address to receive bitcoins
    let address = wallet.get_address(AddressIndex::New).unwrap().address;

    address
}

// Create PSBT
pub fn create_psbt(
    wallet: &Wallet<bdk::sled::Tree>,
    multisig_addr: Address,
    amount: u64,
) -> Result<PartiallySignedTransaction, bdk::Error> {
    let mut tx_builder = wallet.build_tx();

    // For a regular transaction, just set the recipient and amount
    tx_builder.set_recipients(vec![(multisig_addr.script_pubkey(), amount)]);

    // Finalize the transaction and extract the PSBT
    let (psbt, _) = tx_builder.finish()?;

    Ok(psbt)
}

// Sign PSBT and return signed transaction
pub fn sign_psbt(
    wallet: &Wallet<bdk::sled::Tree>,
    mut psbt: PartiallySignedTransaction,
) -> Result<Transaction, bdk::Error> {
    let sign_options = SignOptions {
        assume_height: None,
        ..Default::default()
    };

    wallet.sign(&mut psbt, sign_options)?;

    let tx = psbt.extract_tx();

    Ok(tx)
}

#[cfg(test)]
mod tests {
    use crate::domain::UserEmail;
    use crate::utils::{
        _xpub_receive_change_desc, bitcoind_regtest, compose_multisig_output_descriptor,
        create_service_wallet, generate_service_output_descriptors, multisig_address,
    };
    use bdk::bitcoin::PublicKey;
    use bdk::bitcoin::{Address, Network, Script};
    // use bdk::bitcoincore_rpc::json::AddressType;
    use bdk::blockchain::rpc::{Auth, RpcBlockchain, RpcConfig};
    use bdk::blockchain::ConfigurableBlockchain;
    use bdk::electrum_client::Client;
    use bitcoin::hashes::hex::ToHex;
    use bitcoin::util::bip32::ExtendedPrivKey;
    use bitcoincore_rpc::{json::AddressType, RpcApi};
    use electrsd;

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
    async fn test_multisig_address_generation() {
        let email = UserEmail::parse("user@email.com".to_string()).unwrap();
        let bitcoind = bitcoind_regtest().unwrap();
        let wallet = create_service_wallet(email, Network::Regtest, &bitcoind).await;
        let addr = multisig_address(&wallet).await;
        println!("Multisig address: {}", addr);
        println!("Multisig address: {:?}", wallet.get_balance());
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
        let mobile = vec![
            "[57bdf03e/44'/1'/0']tpubDCagn6iMsE6rnnDimFVhuhAUnPANZPCWwiYuZZcGshgGxZ5CaVhuSjqLwPq3MernBKfnhX8Fb6ypcmGqRC3psoXnyAtPxcBBfuyfkHHQAeJ/0/*".to_string(),
            "[57bdf03e/44'/1'/0']tpubDCagn6iMsE6rnnDimFVhuhAUnPANZPCWwiYuZZcGshgGxZ5CaVhuSjqLwPq3MernBKfnhX8Fb6ypcmGqRC3psoXnyAtPxcBBfuyfkHHQAeJ/1/*".to_string(),
        ];
        let hardware = vec![
            "[c3c2a1a4/44'/1'/0']tpubDCcLyCpJonjU7UztMzxGYku9Qz1s73F5VhHCqU8desDBskS6FWgsU2gy4SF22WhYB9wWZT9j9eqRNpHWLbvBEXhs8BRJia3L9hJLCGwY6fd/0/*".to_string(),
            "[c3c2a1a4/44'/1'/0']tpubDCcLyCpJonjU7UztMzxGYku9Qz1s73F5VhHCqU8desDBskS6FWgsU2gy4SF22WhYB9wWZT9j9eqRNpHWLbvBEXhs8BRJia3L9hJLCGwY6fd/1/*".to_string(),
        ];

        let service = generate_service_output_descriptors(
            &ExtendedPrivKey::from_str("tprv8ZgxMBicQKsPeCZJCjscWgaagn9a3ijuLQYPbCJ4EEYbQML1SqxfFowsSgA9dsuoQXx72ixW799bpFkfN4jRdgVLTWzwSEkRwKL9YrqV26Y").unwrap(),
        );

        let desc = compose_multisig_output_descriptor(mobile, hardware, service);

        println!("Receive: {}", desc.0);
        println!("Change: {}", desc.1);
    }

    #[test]
    fn test_xpub_receive_change_desc() {
        let desc = _xpub_receive_change_desc("tpubDCcLyCpJonjU7UztMzxGYku9Qz1s73F5VhHCqU8desDBskS6FWgsU2gy4SF22WhYB9wWZT9j9eqRNpHWLbvBEXhs8BRJia3L9hJLCGwY6fd");
        println!("Desc: {:?}", desc);
        assert_eq!(2, desc.len());
    }

    #[tokio::test]
    async fn test_electrsd() {
        let bitcoind_conf = electrsd::bitcoind::Conf::default();
        let bitcoind_exe = electrsd::bitcoind::downloaded_exe_path()
            .expect("We should always have downloaded exe");
        let bitcoind =
            electrsd::bitcoind::BitcoinD::with_conf(bitcoind_exe, &bitcoind_conf).unwrap();
        let bitcoind_auth = Auth::Cookie {
            file: bitcoind.params.cookie_file.clone(),
        };
        // Get a new core address
        let core_address = bitcoind.client.get_new_address(None, None).unwrap();

        // Generate 101 blocks and use the above address as coinbase
        bitcoind
            .client
            .generate_to_address(101, &core_address)
            .unwrap();

        println!(">> bitcoind setup complete");
        println!(
            "Available coins in Core wallet : {}",
            bitcoind.client.get_balance(None, None).unwrap()
        );
    }

    #[tokio::test]
    async fn test_psbt_creation_and_signing_flow() {
        // 0. Get bitcoind
        let bitcoind = bitcoind_regtest().unwrap();
        let email = UserEmail::parse("user@email.com".to_string()).unwrap();
        let network = Network::Regtest;
        // 1. Create service wallet
        let service = create_service_wallet(email, network, &bitcoind).await;
        // 2. Create mobile and hardware rpc clients, and fund them
        let mobile = &bitcoind.client;
        let mobile_test_addr = mobile
            .get_new_address(Some("test"), Some(AddressType::Legacy))
            .unwrap();
        let priv_key = mobile.dump_private_key(&mobile_test_addr).unwrap();

        println!("Mobile priv: {:?}", priv_key.inner)
        // 3. Generate multisig address with xpubs from mobile and hardware clients
        // 4. Send bitcoins to 3 above
        // 5. Create and sign PSBT
    }

    #[test]
    fn test_bitcoind_regtest() {
        let bitcoind = bitcoind_regtest().unwrap();

        println!(
            "BitcoinD: {:?}",
            &bitcoind.client.get_best_block_hash().unwrap()
        );
    }
}
