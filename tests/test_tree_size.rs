#![cfg(feature = "disabled-tests")]

use anyhow::{Context, Result};
use zewif_zcashd::{BDBDump, ZcashdDump, ZcashdParser};

// Import shared test utilities
mod test_utils;
use test_utils::fixtures_path;

/// Load a wallet file and extract the OrchardNoteCommitmentTree for testing
fn extract_tree_data(wallet_path: &[&str]) -> Result<()> {
    let file_path = fixtures_path(wallet_path);
    println!("Loading wallet: {:?}", file_path);

    // Parse BDB file
    let db_dump = BDBDump::from_file(&file_path).context("Parsing BerkeleyDB file")?;

    // Parse to ZcashdDump
    let zcashd_dump = ZcashdDump::from_bdb_dump(&db_dump).context("Parsing Zcashd dump")?;

    // Parse into wallet structure
    let (zcashd_wallet, _unparsed_keys) =
        ZcashdParser::parse_dump(&zcashd_dump).context("Parsing Zcashd dump")?;

    // Access the OrchardNoteCommitmentTree
    let tree = zcashd_wallet.orchard_note_commitment_tree();

    // Print detailed information about the tree
    println!("\n=== OrchardNoteCommitmentTree Analysis ===");
    println!("Tree size: {}", tree.tree_size());
    println!("Tree depth: {}", tree.depth());
    println!("Nodes count: {}", tree.nodes().len());
    println!("Leaf nodes count: {}", tree.leaf_nodes().len());
    println!("Is fully parsed: {}", tree.is_fully_parsed());
    println!("Root present: {}", tree.root().is_some());
    println!("Unparsed data size: {}", tree.unparsed_data().len());

    // Migrate to ZeWIF
    let zewif_wallet = zewif_zcashd::migrate_to_zewif(&zcashd_wallet)
        .context("Migrating to ZeWIF")?;

    // Count total addresses and transactions after migration
    let address_count = zewif_wallet.wallets()
        .iter()
        .flat_map(|w| w.accounts())
        .flat_map(|a| a.addresses())
        .count();

    let tx_count = zewif_wallet.transactions().len();

    println!("\n=== Migration Results ===");
    println!("Addresses: {}", address_count);
    println!("Transactions: {}", tx_count);

    // Print the tree summary
    println!("\n=== Tree Summary ===");
    println!("{}", tree.get_tree_summary());

    Ok(())
}

#[test]
fn test_golden_wallet_tree_size() -> Result<()> {
    extract_tree_data(&["zcashd", "golden-v5.6.0", "node0_wallet.dat"])
}

#[test]
fn test_tarnished_wallet_tree_size() -> Result<()> {
    extract_tree_data(&["zcashd", "tarnished-v5.6.0", "node0_wallet.dat"])
}

#[test]
fn test_standard_wallet_tree_size() -> Result<()> {
    extract_tree_data(&["zcashd", "wallet0.dat"])
}
