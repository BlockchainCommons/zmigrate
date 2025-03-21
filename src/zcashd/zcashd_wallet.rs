#![allow(dead_code)]

use std::collections::HashMap;

use crate::{Bip39Mnemonic, Network, SaplingIncomingViewingKey, TxId};

use super::{
    Address, BlockLocator, ClientVersion, KeyPoolEntry, Keys, MnemonicHDChain, NetworkInfo,
    OrchardNoteCommitmentTree, PubKey, RecipientMapping, SaplingKeys, SaplingZPaymentAddress,
    SproutKeys, UnifiedAccounts, WalletTx,
};

#[derive(Debug)]
pub struct ZcashdWallet {
    pub address_names: HashMap<Address, String>,
    pub address_purposes: HashMap<Address, String>,
    pub bestblock_nomerkle: Option<BlockLocator>,
    pub bestblock: BlockLocator,
    pub client_version: ClientVersion,
    pub default_key: PubKey,
    pub key_pool: HashMap<i64, KeyPoolEntry>,
    pub keys: Keys,
    pub min_version: ClientVersion,
    pub mnemonic_hd_chain: MnemonicHDChain,
    pub bip39_mnemonic: Bip39Mnemonic,
    pub network_info: NetworkInfo,
    pub orchard_note_commitment_tree: OrchardNoteCommitmentTree,
    pub orderposnext: Option<i64>,
    pub sapling_keys: SaplingKeys,
    pub sapling_z_addresses: HashMap<SaplingZPaymentAddress, SaplingIncomingViewingKey>,
    pub send_recipients: HashMap<TxId, Vec<RecipientMapping>>,
    pub sprout_keys: Option<SproutKeys>,
    pub transactions: HashMap<TxId, WalletTx>,
    pub unified_accounts: Option<UnifiedAccounts>,
    pub witnesscachesize: i64,
}

impl ZcashdWallet {
    pub fn network(&self) -> Network {
        self.network_info.network()
    }
}
