use bc_envelope::prelude::*;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use zewif::Zewif;

use anyhow::{Context, Result};
use clap::Args;
use std::fmt::Write;

use crate::file_args::{FileArgs, FileArgsLike};

use zewif_zcashd::{BDBDump, DBKey, ZcashdDump, ZcashdParser, ZcashdWallet};

/// Process a zcashd wallet file
#[derive(Debug, Args)]
#[group(skip)]
pub struct CommandArgs {
    #[command(flatten)]
    file_args: FileArgs,
}

impl FileArgsLike for CommandArgs {
    fn file(&self) -> &PathBuf {
        &self.file_args.file
    }
}

impl crate::exec::Exec for CommandArgs {
    fn exec(&self) -> Result<String> {
        dump_wallet(self.file())
    }
}

#[allow(dead_code)]
fn output_keyname_summary(zcashd_dump: &ZcashdDump, output: &mut String) -> Result<()> {
    writeln!(output, "{}", zcashd_dump.keyname_summary())?;
    writeln!(output, "---")?;
    Ok(())
}

#[allow(dead_code)]
fn output_source_wallet_debug(zcashd_dump: &ZcashdDump, output: &mut String) {
    writeln!(output, "Source wallet:\n{:#?}", zcashd_dump).unwrap();
    writeln!(output, "---").unwrap();
}

#[allow(dead_code)]
fn output_unparsed_keys(
    zcashd_dump: &ZcashdDump,
    unparsed_keys: &HashSet<DBKey>,
    output: &mut String,
) {
    if unparsed_keys.is_empty() {
        writeln!(output, "✅ All keys parsed successfully").unwrap();
    } else {
        writeln!(output, "🛑 Unparsed keys:").unwrap();
        let mut sorted_keys: Vec<_> = unparsed_keys.iter().collect();
        sorted_keys.sort();
        let mut last_keyname: Option<String> = None;
        for key in sorted_keys {
            if let Some(ref last_keyname) = last_keyname {
                if *last_keyname != key.keyname {
                    writeln!(output).unwrap();
                }
            }
            last_keyname = Some(key.keyname.to_string());

            let value = zcashd_dump.value_for_key(key).unwrap();
            writeln!(output, "❌ key: {}\n\tvalue: {}", key, value).unwrap();
        }
    }
    writeln!(output, "---").unwrap();
}

#[allow(dead_code)]
fn output_zewif_debug(zewif: &Zewif, output: &mut String) {
    writeln!(output, "Zewif:\n{:#?}", zewif).unwrap();
    writeln!(output, "---").unwrap();
}

#[allow(dead_code)]
fn output_migration_quality_report(
    zcashd_wallet: &ZcashdWallet,
    zewif: &Zewif,
    output: &mut String,
) {
    writeln!(output, "Migration Quality Report").unwrap();

    // Count addresses in zcashd wallet
    let zcashd_address_count = zcashd_wallet.address_names().len();

    // Count addresses in zewif wallet - all accounts combined
    let zewif_address_count = zewif
        .wallets()
        .iter()
        .flat_map(|w| w.accounts())
        .flat_map(|a| a.addresses())
        .count();

    writeln!(
        output,
        "- Addresses: {}/{} preserved",
        zewif_address_count, zcashd_address_count
    ).unwrap();

    // Check transaction preservation
    let zcashd_tx_count = zcashd_wallet.transactions().len();
    let zewif_tx_count = zewif.transactions().len();
    writeln!(
        output,
        "- Transactions: {}/{} preserved",
        zewif_tx_count, zcashd_tx_count
    ).unwrap();

    writeln!(output, "---").unwrap();
}

#[allow(dead_code)]
fn output_envelope(envelope: &Envelope, output: &mut String) {
    writeln!(output, "Zewif envelope:\n{}", envelope.format()).unwrap();
    writeln!(output, "---").unwrap();
}

pub fn dump_wallet(file: &Path) -> Result<String> {
    let db_dump = BDBDump::from_file(file).context("Parsing BerkeleyDB file")?;

    let zcashd_dump = ZcashdDump::from_bdb_dump(&db_dump).context("Parsing Zcashd dump")?;

    let (zcashd_wallet, unparsed_keys) =
        ZcashdParser::parse_dump(&zcashd_dump).context("Parsing Zcashd dump")?;

    let zewif = zewif_zcashd::migrate_to_zewif(&zcashd_wallet).context("Migrating to Zewif")?;

    let envelope = Envelope::from(zewif.clone());

    let mut output = String::new();
    // output_keyname_summary(&zcashd_dump, &mut output);
    // output_source_wallet_debug(&zcashd_dump, &mut output);
    output_unparsed_keys(&zcashd_dump, &unparsed_keys, &mut output);
    // output_zewif_debug(&zewif, &mut output);
    // output_migration_quality_report(&zcashd_wallet, &zewif, &mut output);
    output_envelope(&envelope, &mut output);

    writeln!(output, "✅ Success")?;

    Ok(output)
}
