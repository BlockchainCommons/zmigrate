#![cfg(feature = "disabled-tests")]

//! # Test Suite: Incoming Viewing Key (IVK) Preservation Tests
//!
//! This test suite verifies that Incoming Viewing Keys are properly preserved during
//! wallet migration. Unlike Full Viewing Keys (FVKs), IVKs are typically stored directly
//! in ZCash wallets and are critical for detecting incoming transactions.
//!
//! The tests verify:
//! 1. IVKs are properly extracted from source wallets
//! 2. IVKs are correctly associated with the right addresses after migration
//! 3. The exact IVK values match before and after migration
//! 4. Address without IVKs are handled correctly (no errors/crashes)
//! 5. Only shielded addresses (not transparent) can have IVKs

use std::collections::HashMap;

use anyhow::Result;
use zewif::{
    Account, Network, ProtocolAddress, ShieldedAddress, Zewif, ZewifWallet,
};
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
    println!("Testing IVK preservation for wallet: {}", path.display());

    // Load the wallet using the same method as in zcashd_cmd::dump_wallet
    let db_dump = BDBDump::from_file(&path)?;
    let zcashd_dump = ZcashdDump::from_bdb_dump(&db_dump)?;
    let (zcashd_wallet, _) = ZcashdParser::parse_dump(&zcashd_dump)?;

    Ok((zcashd_dump, zcashd_wallet))
}

/// Tests that incoming viewing keys (IVKs) are properly preserved during migration
///
/// This test verifies that all IVKs in the source wallet are correctly preserved
/// in the migrated ZeWIF wallet. It tests multiple wallet fixtures to ensure
/// robustness across different wallet configurations.
///
/// The test:
/// 1. Loads wallet fixtures from different source files
/// 2. Extracts all sapling addresses and their IVKs from source wallet
/// 3. Migrates the wallet to ZeWIF format
/// 4. Extracts IVKs from the migrated wallet
/// 5. Verifies all original IVKs are present and unchanged
/// 6. Confirms the count of IVKs matches before and after migration
#[test]
fn test_ivk_preservation() -> Result<()> {
    // Test with several different wallet fixtures to ensure wide coverage
    let wallet_paths = [
        &["zcashd", "golden-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "tarnished-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "wallet0.dat"][..], // Use slice syntax to make array sizes compatible
    ];

    for path_elements in wallet_paths {
        // Load wallet and dump
        let (_, zcashd_wallet) = load_zcashd_wallet(path_elements)?;

        // Extract all sapling addresses and their incoming viewing keys from the zcashd wallet
        let sapling_ivks_before: HashMap<String, String> = zcashd_wallet
            .sapling_z_addresses()
            .iter()
            .map(|(addr, ivk)| {
                (addr.to_string(zcashd_wallet.network()), ivk.to_string())
            })
            .collect();

        println!(
            "Found {} sapling addresses with IVKs",
            sapling_ivks_before.len()
        );

        // Convert to ZeWIF format
        let zewif = zewif_zcashd::migrate_to_zewif(&zcashd_wallet)?;

        // Extract the IVKs from the ZeWIF wallet
        let sapling_ivks_after = extract_ivks_from_zewif(&zewif);

        // Verify that all original IVKs are preserved
        for (addr, original_ivk) in &sapling_ivks_before {
            if let Some(migrated_ivk) = sapling_ivks_after.get(addr) {
                assert_eq!(
                    original_ivk, migrated_ivk,
                    "IVK for address {} was not properly preserved",
                    addr
                );
            } else {
                panic!(
                    "Address {} with IVK was not preserved in migration",
                    addr
                );
            }
        }

        // All IVKs should be preserved
        assert_eq!(
            sapling_ivks_before.len(),
            sapling_ivks_after.len(),
            "Number of IVKs before and after migration don't match"
        );
    }

    Ok(())
}

/// Tests the edge case where an address might not have an IVK
///
/// This test creates a synthetic wallet with two types of shielded addresses:
/// 1. Addresses with incoming viewing keys (IVKs)
/// 2. Addresses without IVKs
///
/// It then verifies:
/// - The addresses with IVKs are properly detected and their IVKs extracted
/// - Addresses without IVKs are handled gracefully (no errors/crashes)
/// - Only addresses with IVKs appear in the extraction map
/// - The extracted IVK matches what was initially set
///
/// This test is important because in real-world scenarios, wallets may contain
/// addresses for which IVKs are not available (e.g., watch-only addresses).
#[test]
fn test_missing_ivk_handling() -> Result<()> {
    // For this test, we'll create a ZeWIF wallet with addresses that have missing IVKs
    let mut wallet = ZewifWallet::new(Network::Test);
    let mut account = Account::new();

    // Create an address with an IVK
    let with_ivk_addr_str = "zs1z7rejlpsa98s2rrrfkwmaxu53e4ue0ulcrw0h4x5g8jl04tak0d3mm47vdtahatqrlkngh9sly";
    let mut with_ivk = ShieldedAddress::new(with_ivk_addr_str.to_string());
    let ivk = zewif::sapling::SaplingIncomingViewingKey::default(); // Default for testing
    with_ivk.set_incoming_viewing_key(ivk.clone());

    // Create an address without an IVK
    let without_ivk_addr_str =
        "zs1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq93gnn49";
    let without_ivk = ShieldedAddress::new(without_ivk_addr_str.to_string());

    // Add both addresses to the account
    let with_ivk_addr =
        zewif::Address::new(ProtocolAddress::Shielded(with_ivk));
    let without_ivk_addr =
        zewif::Address::new(ProtocolAddress::Shielded(without_ivk));

    account.add_address(with_ivk_addr);
    account.add_address(without_ivk_addr);
    wallet.add_account(account);

    // Extract IVKs from the wallet
    let ivks = extract_ivks_from_zewif_wallet(&wallet);

    // Verify we have only one IVK
    assert_eq!(ivks.len(), 1, "Should have exactly one IVK");

    // Verify the address with IVK is present
    assert!(
        ivks.contains_key(with_ivk_addr_str),
        "Address with IVK should be in the map"
    );

    // Verify the IVK matches what we set
    assert_eq!(
        ivks.get(with_ivk_addr_str).unwrap(),
        &ivk.to_string(),
        "IVK should match what we set"
    );

    // Verify the address without IVK is not in the map
    assert!(
        !ivks.contains_key(without_ivk_addr_str),
        "Address without IVK should not be in the map"
    );

    Ok(())
}

/// Tests that IVKs are associated only with shielded addresses and not transparent ones
///
/// This test validates the ZeWIF data model's handling of IVKs across different address types:
/// 1. Creates a synthetic wallet with both transparent and shielded addresses
/// 2. Associates an IVK with the shielded address
/// 3. Verifies the IVK extraction process correctly:
///    - Associates IVKs only with shielded addresses
///    - Never associates IVKs with transparent addresses
///    - Maintains the correct address-to-IVK mapping
///
/// This test ensures the implementation properly distinguishes between address types
/// and doesn't attempt to process IVKs for transparent addresses, which would be incorrect.
#[test]
fn test_address_type_ivk_associations() -> Result<()> {
    // Create a wallet with both transparent and shielded addresses
    let mut wallet = ZewifWallet::new(Network::Test);
    let mut account = Account::new();

    // Create a shielded address with an IVK
    let shielded_addr_str = "zs1z7rejlpsa98s2rrrfkwmaxu53e4ue0ulcrw0h4x5g8jl04tak0d3mm47vdtahatqrlkngh9sly";
    let mut shielded_addr = ShieldedAddress::new(shielded_addr_str.to_string());
    let ivk = zewif::sapling::SaplingIncomingViewingKey::default();
    shielded_addr.set_incoming_viewing_key(ivk.clone());

    // Create a transparent address
    let transparent_addr_str = "t1a1zfft6qthfspr3khk7mpkqnufzgspzwj9l5cuq";
    let transparent_addr =
        zewif::TransparentAddress::new(transparent_addr_str.to_string());

    // Add both addresses to the account
    let shielded =
        zewif::Address::new(ProtocolAddress::Shielded(shielded_addr));
    let transparent =
        zewif::Address::new(ProtocolAddress::Transparent(transparent_addr));

    account.add_address(shielded);
    account.add_address(transparent);
    wallet.add_account(account);

    // Extract IVKs from the wallet
    let ivks = extract_ivks_from_zewif_wallet(&wallet);

    // Verify we have exactly one IVK (from the shielded address)
    assert_eq!(ivks.len(), 1, "Should have exactly one IVK");

    // Verify only the shielded address has an IVK
    assert!(
        ivks.contains_key(shielded_addr_str),
        "Shielded address should have an IVK"
    );
    assert!(
        !ivks.contains_key(transparent_addr_str),
        "Transparent address should not have an IVK"
    );

    Ok(())
}

/// Extracts a mapping of address strings to incoming viewing key strings from a ZeWIF wallet
///
/// This helper function traverses the wallet structure to:
/// 1. Find all shielded addresses across all accounts
/// 2. Extract the incoming viewing key (IVK) from each address that has one
/// 3. Build a map from address string to IVK string for easy comparison
///
/// # Parameters
/// - `wallet`: Reference to a ZeWIF wallet to extract IVKs from
///
/// # Returns
/// HashMap mapping address strings to their associated IVK strings.
/// Only addresses with IVKs will be included in the map.
fn extract_ivks_from_zewif_wallet(
    wallet: &ZewifWallet,
) -> HashMap<String, String> {
    let mut ivks = HashMap::new();

    // Iterate through all accounts in the wallet
    for account in wallet.accounts() {
        // Examine each address in the account
        for address in account.addresses() {
            // Only shielded addresses can have IVKs
            if let ProtocolAddress::Shielded(shielded) = address.address() {
                // Extract the IVK if present (handles None case correctly)
                if let Some(ivk) = shielded.incoming_viewing_key() {
                    // Store address->IVK mapping using string representations
                    ivks.insert(
                        shielded.address().to_string(),
                        ivk.to_string(),
                    );
                }
            }
        }
    }

    ivks
}

/// Extracts a mapping of address strings to incoming viewing key strings from a ZeWIF top-level container
///
/// This function extends `extract_ivks_from_zewif` to work with the top-level ZeWIF
/// container, which may contain multiple wallets. It:
/// 1. Iterates through all wallets in the container
/// 2. Extracts IVKs from each wallet using `extract_ivks_from_zewif`
/// 3. Combines the results into a single map with all IVKs across all wallets
///
/// # Parameters
/// - `zewif`: Reference to a ZeWIF top-level container
///
/// # Returns
/// HashMap mapping address strings to their associated IVK strings across all wallets.
fn extract_ivks_from_zewif(zewif: &Zewif) -> HashMap<String, String> {
    let mut all_ivks = HashMap::new();

    // Get all wallets from the top container
    for wallet in zewif.wallets() {
        // Extract IVKs from each wallet and combine them
        let wallet_ivks = extract_ivks_from_zewif_wallet(wallet);
        all_ivks.extend(wallet_ivks);
    }

    all_ivks
}
