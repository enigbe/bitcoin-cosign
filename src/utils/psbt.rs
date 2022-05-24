use bdk::bitcoin::blockdata::script::Script;
use bdk::bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::network::Address;
use bdk::bitcoin::hash_types::Txid;
use bdk::bitcoin::blockdata::transaction::Transaction;

use bitcoincore_rpc;

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
    let tx = Transaction 
    let psbt = PartiallySignedTransaction::from_unsigned_tx(tx);
    todo!()
}
