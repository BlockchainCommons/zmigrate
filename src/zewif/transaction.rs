use crate::{BlockHeight, Data, TxId, impl_attachable};

use super::{
    Attachments, JoinSplitDescription, OrchardActionDescription, SaplingOutputDescription,
    SaplingSpendDescription, TxIn, TxOut,
};

/// A transaction that can combine both transparent and shielded components.
#[derive(Debug, Clone)]
pub struct Transaction {
    /// The transaction id.
    txid: TxId,
    /// The raw transaction data, if known.
    raw: Option<Data>,
    /// The height at which the transaction was mined, if known.
    /// It is possible that if a rollback occurred just after the zeWIF
    /// export, the transaction could have been unmined, and possibly
    /// remined at a different height.
    mined_height: Option<BlockHeight>,

    // Design issue: do we want to parse out all of this? All wallets will
    // necessarily have code to parse a transaction. The only information
    // that is not redundant with the raw transaction encoding is the
    // *decrypted* note plaintexts (and it might be sufficient to just
    // indicate which output indices are expected to be decryptable with
    // which keys). I don't see the point of duplicating the raw data in a
    // different format (that still needs to be parsed!)
    // -- Daira-Emma
    /// Optional data for transparent inputs
    inputs: Option<Vec<TxIn>>,
    /// Optional data for transparent outputs
    outputs: Option<Vec<TxOut>>,
    /// Optional data for Sapling spends
    sapling_spends: Option<Vec<SaplingSpendDescription>>,
    /// Optional data for Sapling outputs
    sapling_outputs: Option<Vec<SaplingOutputDescription>>,
    /// Optional data for Orchard actions
    orchard_actions: Option<Vec<OrchardActionDescription>>,
    /// Optional data for Sprout JoinSplit descriptions
    sprout_joinsplits: Option<Vec<JoinSplitDescription>>,
    // Additional metadata such as confirmations or timestamp may be added here.
    attachments: Attachments,
}

impl_attachable!(Transaction);

impl Transaction {
    pub fn new(txid: TxId) -> Self {
        Self {
            txid,
            raw: None,
            mined_height: None,
            inputs: None,
            outputs: None,
            sapling_spends: None,
            sapling_outputs: None,
            orchard_actions: None,
            sprout_joinsplits: None,
            attachments: Attachments::new(),
        }
    }

    pub fn txid(&self) -> TxId {
        self.txid
    }

    pub fn raw(&self) -> Option<&Data> {
        self.raw.as_ref()
    }

    pub fn set_raw(&mut self, raw: Data) {
        self.raw = Some(raw);
    }

    pub fn mined_height(&self) -> Option<&BlockHeight> {
        self.mined_height.as_ref()
    }

    pub fn set_mined_height(&mut self, height: BlockHeight) {
        self.mined_height = Some(height);
    }

    pub fn inputs(&self) -> Option<&Vec<TxIn>> {
        self.inputs.as_ref()
    }

    pub fn add_input(&mut self, input: TxIn) {
        self.inputs.get_or_insert_with(Vec::new).push(input);
    }

    pub fn outputs(&self) -> Option<&Vec<TxOut>> {
        self.outputs.as_ref()
    }

    pub fn add_output(&mut self, output: TxOut) {
        self.outputs.get_or_insert_with(Vec::new).push(output);
    }

    pub fn sapling_spends(&self) -> Option<&Vec<SaplingSpendDescription>> {
        self.sapling_spends.as_ref()
    }

    pub fn add_sapling_spend(&mut self, spend: SaplingSpendDescription) {
        self.sapling_spends.get_or_insert_with(Vec::new).push(spend);
    }

    pub fn sapling_outputs(&self) -> Option<&Vec<SaplingOutputDescription>> {
        self.sapling_outputs.as_ref()
    }

    pub fn add_sapling_output(&mut self, output: SaplingOutputDescription) {
        self.sapling_outputs
            .get_or_insert_with(Vec::new)
            .push(output);
    }

    pub fn orchard_actions(&self) -> Option<&Vec<OrchardActionDescription>> {
        self.orchard_actions.as_ref()
    }

    pub fn add_orchard_action(&mut self, action: OrchardActionDescription) {
        self.orchard_actions
            .get_or_insert_with(Vec::new)
            .push(action);
    }

    pub fn sprout_joinsplits(&self) -> Option<&Vec<JoinSplitDescription>> {
        self.sprout_joinsplits.as_ref()
    }

    pub fn add_sprout_joinsplit(&mut self, joinsplit: JoinSplitDescription) {
        self.sprout_joinsplits
            .get_or_insert_with(Vec::new)
            .push(joinsplit);
    }
}
