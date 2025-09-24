#![cfg(feature = "disabled-tests")]

//! # Test Suite: Unified Address Migration Tests
//!
//! This test suite verifies that unified addresses are properly migrated:
//! 1. Unified addresses are properly extracted and preserved during migration
//! 2. Diversifier indices are correctly preserved
//! 3. Receiver type information is maintained
//! 4. Unified addresses are correctly assigned to accounts
//!
//! Each test focuses on a specific aspect of unified address migration.

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use zewif::{ReceiverType, Zewif, ZewifWallet};
use zewif_zcashd::{BDBDump, ZcashdDump, ZcashdParser, ZcashdWallet};

// Import shared test utilities
mod test_utils;
use test_utils::fixtures_path;

/// Loads a wallet from the fixtures directory given path components
///
/// This helper function standardizes wallet loading for testing by:
/// 1. Resolving the path to the wallet fixture
/// 2. Loading the BDB database dump
/// 3. Parsing it into a ZcashdDump
/// 4. Converting to a ZcashdWallet structure
///
/// # Parameters
/// - `path_elements`: Array of path components to the wallet fixture
///
/// # Returns
/// Tuple containing:
/// - The raw ZcashdDump (for reference if needed)
/// - The parsed ZcashdWallet structure
fn load_zcashd_wallet(
    path_elements: &[&str],
) -> Result<(ZcashdDump, ZcashdWallet)> {
    let path = fixtures_path(path_elements);
    println!(
        "Testing unified address migration for wallet: {}",
        path.display()
    );

    // Load the wallet using the same method as in zcashd_cmd::dump_wallet
    let db_dump = BDBDump::from_file(&path)?;
    let zcashd_dump = ZcashdDump::from_bdb_dump(&db_dump)?;
    let (zcashd_wallet, _) = ZcashdParser::parse_dump(&zcashd_dump)?;

    Ok((zcashd_dump, zcashd_wallet))
}

/// Helper struct to store unified address information for comparison
#[derive(Debug, Clone)]
struct UnifiedAddressInfo {
    /// The string representation of the address
    #[allow(dead_code)]
    // This is used as a key in the HashMap, not directly accessed
    address_str: String,
    /// The list of receiver types this unified address contains
    receiver_types: HashSet<ReceiverType>,
    /// Whether the address has a diversifier index
    has_diversifier_index: bool,
    /// Whether the address has a transparent component
    has_transparent_component: bool,
    /// Whether the address has a sapling component
    has_sapling_component: bool,
    /// Whether the address has an orchard component
    has_orchard_component: bool,
}

/// Extract unified addresses and their info from a ZCashd wallet
///
/// This function extracts all unified addresses and their associated metadata
/// from a ZCashd wallet to facilitate comparison after migration.
///
/// # Parameters
/// - `wallet`: The ZCashd wallet to extract unified addresses from
///
/// # Returns
/// - HashMap mapping address strings to their associated metadata
fn extract_unified_addresses_from_zcashd(
    wallet: &ZcashdWallet,
) -> HashMap<String, UnifiedAddressInfo> {
    let mut unified_addresses = HashMap::new();

    // Check if the wallet has unified accounts
    if let Some(unified_accounts) = wallet.unified_accounts() {
        // Extract unified addresses from unified_accounts.address_metadata
        for (address_id, metadata) in &unified_accounts.address_metadata {
            // Create a unique string identifier for this unified address
            // The wallet.dat file doesn't actually store the complete unified
            // address strings. Instead, it stores the metadata
            // needed to derive them at runtime:
            // 1. Diversifier indices (which we preserve)
            // 2. Receiver types (which we preserve)
            // 3. References to keys (which we preserve)
            //
            // Since the actual UA string doesn't exist in the source data,
            // we use a placeholder based on the address_id that ensures
            // uniqueness. This is consistent with how the migration
            // code handles this data.
            let id_bytes: &[u8] = address_id.as_ref();
            let ua_string = format!("ua:{}", hex::encode(id_bytes));

            // Create a set of receiver types
            let receiver_types: HashSet<ReceiverType> =
                metadata.receiver_types.iter().cloned().collect();

            // Create the address info
            let address_info = UnifiedAddressInfo {
                address_str: ua_string.clone(),
                receiver_types,
                has_diversifier_index: true, /* Unified addresses always have
                                              * diversifier indices */
                has_transparent_component: metadata
                    .receiver_types
                    .contains(&ReceiverType::P2PKH)
                    || metadata.receiver_types.contains(&ReceiverType::P2SH),
                has_sapling_component: metadata
                    .receiver_types
                    .contains(&ReceiverType::Sapling),
                has_orchard_component: metadata
                    .receiver_types
                    .contains(&ReceiverType::Orchard),
            };

            unified_addresses.insert(ua_string, address_info);
        }
    }

    unified_addresses
}

/// Extract unified addresses and their info from a ZeWIF wallet
///
/// This function extracts all unified addresses and their associated metadata
/// from a ZeWIF wallet to facilitate comparison after migration.
///
/// # Parameters
/// - `wallet`: The ZeWIF wallet to extract unified addresses from
///
/// # Returns
/// - HashMap mapping address strings to their associated metadata
fn extract_unified_addresses_from_zewif_wallet(
    wallet: &ZewifWallet,
) -> HashMap<String, UnifiedAddressInfo> {
    let mut unified_addresses = HashMap::new();

    // Iterate through all accounts in the wallet
    for account in wallet.accounts() {
        // Examine each address in the account to find unified addresses
        for address in account.addresses() {
            if let Some(unified_addr) = address.as_unified() {
                let address_str = address.as_string();

                // Convert the receiver types array to a HashSet
                let receiver_types: HashSet<ReceiverType> =
                    unified_addr.receiver_types().iter().cloned().collect();

                // Create the address info
                let address_info = UnifiedAddressInfo {
                    address_str: address_str.clone(),
                    receiver_types,
                    has_diversifier_index: unified_addr
                        .diversifier_index()
                        .is_some(),
                    has_transparent_component: unified_addr
                        .has_transparent_component(),
                    has_sapling_component: unified_addr.has_sapling_component(),
                    has_orchard_component: unified_addr.has_orchard_component(),
                };

                unified_addresses.insert(address_str, address_info);
            }
        }
    }

    unified_addresses
}

/// Extract unified addresses and their info from a ZeWIF top-level container
///
/// This function extends `extract_unified_addresses_from_zewif` to work with
/// the top-level ZeWIF container, which may contain multiple wallets.
///
/// # Parameters
/// - `zewif`: Reference to a ZeWIF top-level container
///
/// # Returns
/// - HashMap mapping address strings to their associated metadata across all
///   wallets
fn extract_unified_addresses_from_zewif(
    zewif: &Zewif,
) -> HashMap<String, UnifiedAddressInfo> {
    let mut all_addresses = HashMap::new();

    // Get all wallets from the top container
    for wallet in zewif.wallets() {
        // Extract addresses from each wallet and combine them
        let wallet_addresses =
            extract_unified_addresses_from_zewif_wallet(wallet);
        all_addresses.extend(wallet_addresses);
    }

    all_addresses
}

/// Tests if the source wallet has unified addresses and if they're properly
/// preserved
///
/// This test verifies:
/// 1. That unified accounts are detected when present
/// 2. That unified addresses are properly preserved during migration
#[test]
fn test_unified_address_detection() -> Result<()> {
    // Test with wallet fixtures that may have unified addresses
    let wallet_paths = [
        &["zcashd", "golden-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "tarnished-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "wallet0.dat"][..],
    ];

    for path_elements in wallet_paths {
        // Load wallet and dump
        let (_, zcashd_wallet) = load_zcashd_wallet(path_elements)?;

        // Extract unified addresses from the ZCashd wallet
        let addresses_before =
            extract_unified_addresses_from_zcashd(&zcashd_wallet);

        // Print information about whether unified addresses were found
        if addresses_before.is_empty() {
            println!("No unified addresses found in the source wallet");
        } else {
            println!(
                "Found {} unified addresses in the source wallet",
                addresses_before.len()
            );
            for (addr, info) in &addresses_before {
                println!(
                    "Address {}: with {} receiver types",
                    addr,
                    info.receiver_types.len()
                );
            }
        }

        // Convert to ZeWIF format
        let zewif = zewif_zcashd::migrate_to_zewif(&zcashd_wallet)?;

        // Extract unified addresses from the migrated ZeWIF wallet
        let addresses_after = extract_unified_addresses_from_zewif(&zewif);

        // Print information about whether unified addresses were found after
        // migration
        if addresses_after.is_empty() {
            println!("No unified addresses found in the migrated wallet");
        } else {
            println!(
                "Found {} unified addresses in the migrated wallet",
                addresses_after.len()
            );
            for (addr, info) in &addresses_after {
                println!(
                    "Address {}: with {} receiver types",
                    addr,
                    info.receiver_types.len()
                );
            }
        }

        // Verify that all unified addresses are preserved
        assert_eq!(
            addresses_before.len(),
            addresses_after.len(),
            "Number of unified addresses doesn't match after migration"
        );
    }

    Ok(())
}

/// Tests that unified address metadata is properly preserved during migration
///
/// This test verifies that:
/// 1. Diversifier indices are preserved
/// 2. Receiver types are maintained
/// 3. Transparent/sapling/orchard component status is preserved
#[test]
fn test_unified_address_metadata_preservation() -> Result<()> {
    // Test with wallet fixtures that may have unified addresses
    let wallet_paths = [
        &["zcashd", "golden-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "tarnished-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "wallet0.dat"][..],
    ];

    for path_elements in wallet_paths {
        // Load wallet and dump
        let (_, zcashd_wallet) = load_zcashd_wallet(path_elements)?;

        // Extract unified addresses from the ZCashd wallet
        let addresses_before =
            extract_unified_addresses_from_zcashd(&zcashd_wallet);

        // Skip this test if there are no unified addresses in the source wallet
        if addresses_before.is_empty() {
            println!(
                "Skipping metadata preservation test as no unified addresses found in source wallet"
            );
            continue;
        }

        // Convert to ZeWIF format
        let zewif = zewif_zcashd::migrate_to_zewif(&zcashd_wallet)?;

        // Extract unified addresses from the migrated ZeWIF wallet
        let addresses_after = extract_unified_addresses_from_zewif(&zewif);

        // Verify metadata preservation for each address
        for (addr_str, before_info) in &addresses_before {
            // Find the corresponding address in the migrated wallet
            // The keys won't match directly because we generate placeholder
            // addresses, so we need to find the matching address by
            // comparing metadata
            let (_, after_info) = addresses_after
                .iter()
                .find(|(_, info)| {
                    // Match by the set of receiver types
                    info.receiver_types == before_info.receiver_types
                        && info.has_transparent_component
                            == before_info.has_transparent_component
                        && info.has_sapling_component
                            == before_info.has_sapling_component
                        && info.has_orchard_component
                            == before_info.has_orchard_component
                })
                .expect("Address not found after migration");

            // Verify diversifier index is preserved
            assert_eq!(
                before_info.has_diversifier_index,
                after_info.has_diversifier_index,
                "Diversifier index status doesn't match for {}",
                addr_str
            );

            // Verify receiver types are preserved (size)
            assert_eq!(
                before_info.receiver_types.len(),
                after_info.receiver_types.len(),
                "Number of receiver types doesn't match for {}",
                addr_str
            );

            // Verify each receiver type is preserved
            for receiver_type in &before_info.receiver_types {
                assert!(
                    after_info.receiver_types.contains(receiver_type),
                    "Receiver type {:?} not preserved for {}",
                    receiver_type,
                    addr_str
                );
            }

            // Verify component presence is preserved
            assert_eq!(
                before_info.has_transparent_component,
                after_info.has_transparent_component,
                "Transparent component status doesn't match for {}",
                addr_str
            );

            assert_eq!(
                before_info.has_sapling_component,
                after_info.has_sapling_component,
                "Sapling component status doesn't match for {}",
                addr_str
            );

            assert_eq!(
                before_info.has_orchard_component,
                after_info.has_orchard_component,
                "Orchard component status doesn't match for {}",
                addr_str
            );
        }
    }

    Ok(())
}

/// Tests that unified addresses are assigned to the correct accounts
///
/// This test verifies that:
/// 1. Unified addresses are assigned to accounts according to the address
///    registry
/// 2. In single-account mode, all unified addresses are assigned to the default
///    account
#[test]
fn test_unified_address_account_assignment() -> Result<()> {
    // Test with wallet fixtures that may have unified addresses
    let wallet_paths = [
        &["zcashd", "golden-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "tarnished-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "wallet0.dat"][..],
    ];

    for path_elements in wallet_paths {
        // Load wallet and dump
        let (_, zcashd_wallet) = load_zcashd_wallet(path_elements)?;

        // Extract unified addresses from the ZCashd wallet
        let addresses_before =
            extract_unified_addresses_from_zcashd(&zcashd_wallet);

        // Skip this test if there are no unified addresses in the source wallet
        if addresses_before.is_empty() {
            println!(
                "Skipping account assignment test as no unified addresses found in source wallet"
            );
            continue;
        }

        // Convert to ZeWIF format
        let zewif = zewif_zcashd::migrate_to_zewif(&zcashd_wallet)?;

        // Get the first wallet from the top container
        let wallet = zewif.wallets().iter().next().expect("No wallet found");

        // Check if this wallet has unified accounts
        let has_unified_accounts = zcashd_wallet.unified_accounts().is_some();

        if !has_unified_accounts {
            // For wallets without unified accounts, we should have a single
            // default account
            assert_eq!(
                wallet.accounts().len(),
                1,
                "Default migration should result in a single account for wallets without unified accounts"
            );

            // The single account should contain all unified addresses (if any)
            let default_account =
                wallet.accounts().iter().next().expect("No account found");
            let unified_addresses_in_account = default_account
                .addresses()
                .iter()
                .filter(|addr| addr.is_unified())
                .count();

            assert_eq!(
                unified_addresses_in_account,
                addresses_before.len(),
                "Default account should contain all unified addresses"
            );
        } else {
            // For wallets with unified accounts, verify that:
            // 1. We have multiple accounts
            // 2. Each account contains the expected unified addresses

            // Count the total number of unified addresses across all accounts
            let mut total_unified_addresses = 0;
            for account in wallet.accounts() {
                total_unified_addresses += account
                    .addresses()
                    .iter()
                    .filter(|addr| addr.is_unified())
                    .count();
            }

            // Verify the total count matches
            assert_eq!(
                total_unified_addresses,
                addresses_before.len(),
                "Total number of unified addresses doesn't match after migration"
            );
        }
    }

    Ok(())
}

/// Tests unified address migration with the Zingo wallet fixtures
///
/// This test is separate because Zingo wallets might have different
/// unified address formats and structures.
#[test]
fn test_zingo_unified_address_migration() -> Result<()> {
    // This test is a placeholder for now because we need to implement
    // support for Zingo unified addresses separately
    println!(
        "Zingo unified address migration tests will be implemented separately"
    );

    Ok(())
}
