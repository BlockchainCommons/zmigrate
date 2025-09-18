use anyhow::{Result, bail};
use zmigrate::{zcashd_cmd /* zingo_cmd */};

use regex::Regex;
use std::fmt::Write;

// Import shared test utilities
mod test_utils;
use test_utils::fixtures_path;

fn dump_wallet(path_elements: &[&str]) -> Result<String> {
    let path = fixtures_path(path_elements);
    if path_elements[0] == "zcashd" {
        zcashd_cmd::dump_wallet(&path)
    } else if path_elements[0] == "zingo" {
        // zingo_cmd::dump_wallet(&path)
        bail!("zingo dump not yet implemented")
    } else {
        bail!("Unknown command: {}", path_elements[0]);
    }
}

fn test_dump(path_elements: &[&str]) {
    let output = dump_wallet(path_elements).unwrap_or_else(|_| {
        panic!("Unable to process file: {:?}", path_elements)
    });
    assert!(output.lines().last().unwrap().contains("Success"));
    // println!("{}", output);
}

fn test_migration_quality(path_elements: &[&str]) -> Result<String> {
    let output = dump_wallet(path_elements)?;

    // Extract the ZcashdWallet and Zewif sections from the output
    let sections: Vec<&str> = output.split("---").collect();
    if sections.len() < 4 {
        return Ok(format!(
            "❌ Missing migration data for {:?}",
            path_elements
        ));
    }

    let zcashd_section = sections[1].trim();
    let zewif_section = sections[2].trim();

    let mut report = String::new();
    writeln!(report, "Migration Quality Report for {:?}", path_elements)?;

    // Check address preservation
    let zcashd_address_count = zcashd_section.matches("Address").count();
    let zewif_address_count = zewif_section.matches("Address").count();
    writeln!(
        report,
        "- Addresses: {}/{} preserved",
        zewif_address_count, zcashd_address_count
    )?;

    // Check transaction preservation
    let zcashd_tx_count = zcashd_section.matches("TxId").count();
    let zewif_tx_count = zewif_section.matches("TxId").count();
    writeln!(
        report,
        "- Transactions: {}/{} preserved",
        zewif_tx_count, zcashd_tx_count
    )?;

    // Check position information
    let zero_positions_count = zewif_section.matches("Position(0)").count();
    let nonzero_positions_count = count_nonzero_positions(zewif_section);
    let total_positions = zero_positions_count + nonzero_positions_count;

    if total_positions > 0 {
        let preservation_rate = ((nonzero_positions_count as f64)
            / (total_positions as f64))
            * 100.0;
        writeln!(
            report,
            "- Positions: {}/{} preserved ({:.1}%)",
            nonzero_positions_count, total_positions, preservation_rate
        )?;
    } else {
        writeln!(report, "- Positions: No position data found")?;
    }

    // Check note commitment trees
    let has_orchard_tree = zcashd_section.contains("OrchardNoteCommitmentTree");
    let has_sapling_tree = zcashd_section.contains("SaplingNoteCommitmentTree");

    if has_orchard_tree || has_sapling_tree {
        writeln!(report, "- Note Commitment Trees:")?;
        if has_orchard_tree {
            writeln!(report, "  * Orchard tree: Present")?;
        }
        if has_sapling_tree {
            writeln!(report, "  * Sapling tree: Present")?;
        }
    } else {
        writeln!(report, "- Note Commitment Trees: None found")?;
    }

    // Check for key types
    writeln!(report, "- Key Type Preservation:")?;
    report_key_preservation(
        &mut report,
        zcashd_section,
        zewif_section,
        "Orchard",
    )?;
    report_key_preservation(
        &mut report,
        zcashd_section,
        zewif_section,
        "Sapling",
    )?;
    report_key_preservation(
        &mut report,
        zcashd_section,
        zewif_section,
        "Transparent",
    )?;
    report_key_preservation(
        &mut report,
        zcashd_section,
        zewif_section,
        "Spending",
    )?;
    report_key_preservation(
        &mut report,
        zcashd_section,
        zewif_section,
        "Viewing",
    )?;

    // Check for account handling
    let zcashd_accounts = count_pattern(zcashd_section, r"Account\s*\{");
    let zewif_accounts = count_pattern(zewif_section, r"Account\s*\{");
    writeln!(
        report,
        "- Accounts: {}/{} preserved",
        zewif_accounts, zcashd_accounts
    )?;

    // Check specific ZCash features
    check_feature_presence(
        &mut report,
        zcashd_section,
        zewif_section,
        "Unified",
        "Unified Address Support",
    )?;
    check_feature_presence(
        &mut report,
        zcashd_section,
        zewif_section,
        "SeedMaterial",
        "Seed Material",
    )?;
    check_feature_presence(
        &mut report,
        zcashd_section,
        zewif_section,
        "Network",
        "Network Information",
    )?;

    Ok(report)
}

fn count_nonzero_positions(text: &str) -> usize {
    let mut count = 0;
    let position_pattern = r"Position\((\d+)\)";
    let re = Regex::new(position_pattern).unwrap();

    for cap in re.captures_iter(text) {
        if let Some(num_match) = cap.get(1) {
            if let Ok(num) = num_match.as_str().parse::<u32>() {
                if num > 0 {
                    count += 1;
                }
            }
        }
    }
    count
}

fn count_pattern(text: &str, pattern: &str) -> usize {
    let re = Regex::new(pattern).unwrap();
    re.find_iter(text).count()
}

fn check_feature_presence(
    report: &mut String,
    source: &str,
    dest: &str,
    key_word: &str,
    feature_name: &str,
) -> Result<()> {
    let in_source = source.contains(key_word);
    let in_dest = dest.contains(key_word);

    if in_source && in_dest {
        writeln!(report, "- {}: Preserved ✓", feature_name)?;
    } else if in_source && !in_dest {
        writeln!(report, "- {}: MISSING ✗", feature_name)?;
    } else if !in_source && in_dest {
        writeln!(report, "- {}: Added in ZeWIF (not in source)", feature_name)?;
    }

    Ok(())
}

fn report_key_preservation(
    report: &mut String,
    source: &str,
    dest: &str,
    key_type: &str,
) -> Result<()> {
    let source_count = source.matches(key_type).count();
    let dest_count = dest.matches(key_type).count();

    if source_count > 0 {
        let preservation_rate =
            ((dest_count as f64) / (source_count as f64)) * 100.0;
        writeln!(
            report,
            "  * {} keys: {}/{} preserved ({:.1}%)",
            key_type, dest_count, source_count, preservation_rate
        )?;
    } else {
        writeln!(report, "  * {} keys: None found in source", key_type)?;
    }

    Ok(())
}

#[test]
fn test_zcashd() {
    let paths = vec![
        vec!["zcashd", "golden-v5.6.0", "node0_wallet.dat"],
        vec!["zcashd", "golden-v5.6.0", "node1_wallet.dat"],
        vec!["zcashd", "golden-v5.6.0", "node2_wallet.dat"],
        vec!["zcashd", "golden-v5.6.0", "node3_wallet.dat"],
        vec!["zcashd", "tarnished-v5.6.0", "node0_wallet.dat"],
        vec!["zcashd", "tarnished-v5.6.0", "node1_wallet.dat"],
        vec!["zcashd", "tarnished-v5.6.0", "node2_wallet.dat"],
        vec!["zcashd", "tarnished-v5.6.0", "node3_wallet.dat"],
        vec!["zcashd", "sprout", "node0_wallet.dat"],
        vec!["zcashd", "sprout", "node1_wallet.dat"],
        vec!["zcashd", "sprout", "node2_wallet.dat"],
        vec!["zcashd", "sprout", "node3_wallet.dat"],
        vec!["zcashd", "wallet0.dat"],
        vec!["zcashd", "wallet1.dat"],
        vec!["zcashd", "wallet2.dat"],
        vec!["zcashd", "wallet3.dat"],
        vec!["zcashd", "wallet4.dat"],
        vec!["zcashd", "wallet5.dat"],
        vec!["zcashd", "wallet6.dat"],
        vec!["zcashd", "wallet7.dat"],
    ];
    for path in &paths {
        test_dump(path);
    }
}

#[test]
#[ignore]
fn test_migration_quality_report() {
    // Test a variety of wallets for migration quality
    let test_paths = vec![
        // Golden reference wallets (expected to be fully working)
        vec!["zcashd", "golden-v5.6.0", "node0_wallet.dat"],
        vec!["zcashd", "golden-v5.6.0", "node2_wallet.dat"], // May have more shielded data
        // Tarnished wallets (may have issues)
        vec!["zcashd", "tarnished-v5.6.0", "node0_wallet.dat"],
        // Sprout wallets (older format)
        vec!["zcashd", "sprout", "node0_wallet.dat"],
        // Standard wallets
        vec!["zcashd", "wallet0.dat"], // Test standard wallet
        vec!["zcashd", "wallet5.dat"], // Test wallet likely with Orchard data
    ];

    // Create a summary table of all wallet reports
    let mut summary = String::new();
    writeln!(summary, "=== MIGRATION QUALITY SUMMARY ===").unwrap();
    writeln!(
        summary,
        "{:<40} | {:<15} | {:<15} | {:<15}",
        "Wallet", "Addresses", "Transactions", "Positions"
    )
    .unwrap();
    writeln!(
        summary,
        "{:-<40}-+-{:-<15}-+-{:-<15}-+-{:-<15}",
        "", "", "", ""
    )
    .unwrap();

    // Process each wallet and collect stats
    for path in &test_paths {
        let report = test_migration_quality(path).unwrap_or_else(|e| {
            format!("Error generating report for {:?}: {}", path, e)
        });

        // Print the detailed report
        println!("\n{}\n", report);

        // Basic assertions to verify the report contains meaningful data
        assert!(report.contains("Migration Quality Report"));
        assert!(report.contains("Addresses:"));
        assert!(report.contains("Transactions:"));

        // Extract stats for summary table
        let wallet_name = path.join("/");
        let addr_stats = extract_stat(&report, "Addresses:");
        let tx_stats = extract_stat(&report, "Transactions:");
        let pos_stats = if report.contains("Positions:") {
            extract_stat(&report, "Positions:")
        } else {
            "N/A".to_string()
        };

        writeln!(
            summary,
            "{:<40} | {:<15} | {:<15} | {:<15}",
            wallet_name, addr_stats, tx_stats, pos_stats
        )
        .unwrap();
    }

    // Print the summary table
    println!("\n{}\n", summary);
}

fn extract_stat(report: &str, label: &str) -> String {
    if let Some(line) = report.lines().find(|l| l.contains(label)) {
        let parts: Vec<&str> = line.split(label).collect();
        if parts.len() > 1 {
            return parts[1].trim().to_string();
        }
    }
    "unknown".to_string()
}

#[test]
#[ignore = "zingo dump not yet implemented"]
fn test_zingo() {
    let paths = vec![
        vec![
            "zingo",
            "mainnet",
            "hhcclaltpcckcsslpcnetblr-gf0aaf9347.dat",
        ],
        vec!["zingo", "mainnet", "hhcclaltpcckcsslpcnetblr-latest.dat"],
        // vec!["zingo", "mainnet", "vtfcorfbcbpctcfupmegmwbp-v28.dat"], // long
        vec!["zingo", "regtest", "hmvasmuvwmssvichcarbpoct-v27.dat"],
        vec!["zingo", "regtest", "aadaalacaadaalacaadaalac-orch-only.dat"],
        vec![
            "zingo",
            "regtest",
            "aadaalacaadaalacaadaalac-orch-and-sapling.dat",
        ],
        vec!["zingo", "regtest", "aaaaaaaaaaaaaaaaaaaaaaaa-v26.dat"],
        vec!["zingo", "testnet", "cbbhrwiilgbrababsshsmtpr-latest.dat"],
        vec!["zingo", "testnet", "G93738061a.dat"],
        vec!["zingo", "testnet", "Gab72a38b.dat"],
        vec!["zingo", "testnet", "glory_goddess.dat"],
        vec!["zingo", "testnet", "latest.dat"],
        vec!["zingo", "testnet", "v26.dat"],
        vec!["zingo", "testnet", "v28.dat"], // vec!["zingo", "testnet", "v27.dat"], // long
    ];
    for path in &paths {
        test_dump(path);
    }
}
