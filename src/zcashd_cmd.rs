use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use std::fmt::Write;

use crate::file_args::{FileArgs, FileArgsLike};

use zewif_zcashd::{BDBDump, ZcashdDump, ZcashdParser, migrate::migrate_to_zewif};

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

pub fn dump_wallet(file: &Path) -> Result<String> {
    let db_dump = BDBDump::from_file(file).context("Parsing BerkeleyDB file")?;

    let zcashd_dump = ZcashdDump::from_bdb_dump(&db_dump).context("Parsing Zcashd dump")?;

    let (zcashd_wallet, unparsed_keys) =
        ZcashdParser::parse_dump(&zcashd_dump).context("Parsing Zcashd dump")?;

    let mut output = String::new();

    // writeln!(output, "{}", zcashd_dump.keyname_summary())?;
    // writeln!(output, "---")?;

    writeln!(output, "Source wallet:\n{:#?}", zcashd_wallet)?;

    if !unparsed_keys.is_empty() {
        writeln!(output, "---")?;
        writeln!(output, "🛑 Unparsed keys:")?;
        let mut sorted_keys: Vec<_> = unparsed_keys.into_iter().collect();
        sorted_keys.sort();
        let mut last_keyname: Option<String> = None;
        for key in sorted_keys {
            if let Some(ref last_keyname) = last_keyname {
                if *last_keyname != key.keyname {
                    writeln!(output)?;
                }
            }
            last_keyname = Some(key.keyname.to_string());

            let value = zcashd_dump.value_for_key(&key)?;
            writeln!(output, "❌ key: {}\n\tvalue: {}", key, value)?;
        }
        return Ok(output);
    }

    let zewif_wallet = migrate_to_zewif(&zcashd_wallet).context("Migrating to Zewif")?;
    writeln!(output, "---")?;
    writeln!(output, "Migrated wallet:\n{:#?}", zewif_wallet)?;

    // Generate migration quality report
    writeln!(output, "---")?;

    // Create migration quality report
    let mut report = String::new();
    writeln!(report, "Migration Quality Report")?;

    // Count addresses in zcashd wallet
    let zcashd_address_count = zcashd_wallet.address_names().len();

    // Count addresses in zewif wallet - all accounts combined
    let zewif_address_count = zewif_wallet
        .wallets()
        .values()
        .flat_map(|w| w.accounts().values())
        .flat_map(|a| a.addresses())
        .count();

    writeln!(
        report,
        "- Addresses: {}/{} preserved",
        zewif_address_count, zcashd_address_count
    )?;

    // Check transaction preservation
    let zcashd_tx_count = zcashd_wallet.transactions().len();
    let zewif_tx_count = zewif_wallet.transactions().len();
    writeln!(
        report,
        "- Transactions: {}/{} preserved",
        zewif_tx_count, zcashd_tx_count
    )?;

    // Add the report to the output
    writeln!(output, "{}", report)?;

    writeln!(output, "---")?;
    writeln!(output, "✅ Success")?;

    Ok(output)
}
