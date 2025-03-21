use anyhow::Result;

use crate::{Blob32, Parse, Parser, SecondsSinceEpoch, parse};

#[derive(Debug, Clone, PartialEq)]
pub struct MnemonicHDChain {
    pub version: i32,
    pub seed_fp: Blob32,
    pub create_time: SecondsSinceEpoch,
    pub account_counter: u32,
    pub legacy_tkey_external_counter: u32,
    pub legacy_tkey_internal_counter: u32,
    pub legacy_sapling_key_counter: u32,
    pub mnemonic_seed_backup_confirmed: bool,
}

impl Parse for MnemonicHDChain {
    fn parse(p: &mut Parser) -> Result<Self> {
        Ok(Self {
            version: parse!(p, "version")?,
            seed_fp: parse!(p, "seed_fp")?,
            create_time: parse!(p, "create_time")?,
            account_counter: parse!(p, "account_counter")?,
            legacy_tkey_external_counter: parse!(p, "legacy_tkey_external_counter")?,
            legacy_tkey_internal_counter: parse!(p, "legacy_tkey_internal_counter")?,
            legacy_sapling_key_counter: parse!(p, "legacy_sapling_key_counter")?,
            mnemonic_seed_backup_confirmed: parse!(p, "mnemonic_seed_backup_confirmed")?,
        })
    }
}
