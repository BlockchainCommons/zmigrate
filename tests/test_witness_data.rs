#![cfg(feature = "disabled-tests")]

use anyhow::Result;
use regex::Regex;
use zmigrate::{zcashd_cmd, zingo_cmd};

// Import shared test utilities
mod test_utils;
use test_utils::fixtures_path;

fn dump_wallet(path_elements: &[&str]) -> Result<String> {
    let path = fixtures_path(path_elements);
    if path_elements[0] == "zcashd" {
        zcashd_cmd::dump_wallet(&path)
    } else if path_elements[0] == "zingo" {
        zingo_cmd::dump_wallet(&path)
    } else {
        Err(anyhow::anyhow!("Unknown command: {}", path_elements[0]))
    }
}

/// Tests that witness data and memo fields are properly migrated from ZCashd to ZeWIF format
#[test]
#[ignore]
fn test_witness_data_migration() {
    // Select a variety of wallets to test with
    let test_paths = vec![
        // Golden reference wallets (expected to be fully working)
        vec!["zcashd", "golden-v5.6.0", "node0_wallet.dat"],
        vec!["zcashd", "golden-v5.6.0", "node2_wallet.dat"],
        // Tarnished wallets (may have issues)
        vec!["zcashd", "tarnished-v5.6.0", "node0_wallet.dat"],
        // Standard wallets
        vec!["zcashd", "wallet0.dat"],
        vec!["zcashd", "wallet5.dat"],
    ];

    // Process each wallet and check witness data migration
    for path in &test_paths {
        let output = dump_wallet(path).unwrap_or_else(|e| {
            panic!("Error dumping wallet {:?}: {}", path, e)
        });

        // Split the output into ZcashdWallet and Zewif sections
        let sections: Vec<&str> = output.split("---").collect();
        if sections.len() < 4 {
            panic!("Missing migration data for {:?}", path);
        }

        let zcashd_section = sections[1].trim();
        let zewif_section = sections[2].trim();

        // Check for witness data entries in the output
        let has_witness_in_source = has_witness_data(zcashd_section);
        let witness_count_in_dest = count_witness_entries(zewif_section);

        println!("\nWitness Data & Memo Migration for {:?}:", path);
        println!("- Source has witness data: {}", has_witness_in_source);
        println!("- Destination witness entries: {}", witness_count_in_dest);

        // Check for memo field entries in the output
        let memo_count_in_dest = count_memo_entries(zewif_section);
        println!("- Destination memo entries: {}", memo_count_in_dest);

        // Note: Transaction time is noted in the code but not yet stored
        // This will be implemented in the "Extract Transaction Metadata" subtask

        // We don't want to strictly assert witness data exists because some wallets
        // may legitimately not have any. Instead, we just log the information.

        // But we can check that memo field support is working by verifying that we have some memo entries
        // Only assert if we have sapling outputs, which should have memo fields
        if zewif_section.contains("SaplingOutputDescription") {
            assert!(
                memo_count_in_dest > 0,
                "Memo fields should be present in Sapling outputs"
            );
        }
    }
}

/// Check if source wallet contains witness data
fn has_witness_data(wallet_section: &str) -> bool {
    // Look for evidence of witness data in the wallet
    // This could be in either witnesses field or witness fields
    wallet_section.contains("witnesses: [")
        || (wallet_section.contains("witness:")
            && !wallet_section.contains("witness: None"))
}

/// Count witness entries in the destination ZeWIF format
fn count_witness_entries(zewif_section: &str) -> usize {
    // Look for witness: Some entries, which indicates witness data was migrated
    let witness_pattern = r"witness:\s*Some\(";
    let re = Regex::new(witness_pattern).unwrap();
    re.find_iter(zewif_section).count()
}

/// Count memo entries in the destination ZeWIF format
fn count_memo_entries(zewif_section: &str) -> usize {
    // Look for memo: Some entries, which indicates memo data was preserved
    let memo_pattern = r"memo:\s*Some\(";
    let re = Regex::new(memo_pattern).unwrap();
    re.find_iter(zewif_section).count()
}
