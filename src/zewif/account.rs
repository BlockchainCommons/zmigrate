use std::collections::{HashMap, HashSet};

use bc_components::ARID;

use crate::{TxId, impl_attachable};

use super::{Address, Attachments, OrchardSentOutput, SaplingSentOutput};

/// Logical grouping within a wallet. Each account can have its own set of
/// addresses, transactions, and other metadata.
#[derive(Debug, Clone)]
pub struct Account {
    id: ARID,
    name: String, // May not be unique.
    zip32_account_id: Option<u32>,
    addresses: HashMap<String, Address>,
    relevant_transactions: HashSet<TxId>, // Subset of the global transaction history.
    // The following are intended for storage of information that may not be
    // recoverable from the chain.
    sapling_sent_outputs: Vec<SaplingSentOutput>,
    orchard_sent_outputs: Vec<OrchardSentOutput>,
    attachments: Attachments,
}

impl_attachable!(Account);

impl Account {
    pub fn new() -> Self {
        Self {
            id: ARID::new(),
            name: String::default(),
            zip32_account_id: None,
            addresses: HashMap::new(),
            relevant_transactions: HashSet::new(),
            sapling_sent_outputs: Vec::new(),
            orchard_sent_outputs: Vec::new(),
            attachments: Attachments::new(),
        }
    }

    pub fn id(&self) -> ARID {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn zip32_account_id(&self) -> Option<u32> {
        self.zip32_account_id
    }

    pub fn set_zip32_account_id(&mut self, id: u32) {
        self.zip32_account_id = Some(id);
    }

    pub fn addresses(&self) -> &HashMap<String, Address> {
        &self.addresses
    }

    pub fn addresses_mut(&mut self) -> &mut HashMap<String, Address> {
        &mut self.addresses
    }

    pub fn add_address(&mut self, address: Address) {
        self.addresses.insert(address.as_string(), address);
    }

    pub fn relevant_transactions(&self) -> &HashSet<TxId> {
        &self.relevant_transactions
    }

    pub fn relevant_transactions_mut(&mut self) -> &mut HashSet<TxId> {
        &mut self.relevant_transactions
    }

    pub fn add_relevant_transaction(&mut self, txid: TxId) {
        self.relevant_transactions.insert(txid);
    }

    pub fn sapling_sent_outputs(&self) -> &Vec<SaplingSentOutput> {
        &self.sapling_sent_outputs
    }

    pub fn sapling_sent_outputs_mut(&mut self) -> &mut Vec<SaplingSentOutput> {
        &mut self.sapling_sent_outputs
    }

    pub fn add_sapling_sent_output(&mut self, output: SaplingSentOutput) {
        self.sapling_sent_outputs.push(output);
    }

    pub fn orchard_sent_outputs(&self) -> &Vec<OrchardSentOutput> {
        &self.orchard_sent_outputs
    }

    pub fn orchard_sent_outputs_mut(&mut self) -> &mut Vec<OrchardSentOutput> {
        &mut self.orchard_sent_outputs
    }

    pub fn add_orchard_sent_output(&mut self, output: OrchardSentOutput) {
        self.orchard_sent_outputs.push(output);
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}
