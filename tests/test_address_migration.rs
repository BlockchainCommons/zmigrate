#![cfg(feature = "disabled-tests")]

//! # Test Suite: Address Migration Tests
//!
//! This test suite verifies that all aspects of address migration work correctly:
//! 1. All address types (transparent, sapling, unified) are properly extracted and preserved
//! 2. Address metadata (names, purposes, HD paths, keys) is correctly migrated
//! 3. Addresses are assigned to the correct accounts
//!
//! Each test focuses on a specific aspect of address migration, with helpers to
//! extract addresses and their metadata for comparison.

use std::collections::HashMap;

use anyhow::Result;
use zewif::{
    Account, Address, Network, ProtocolAddress, ShieldedAddress,
    TransparentAddress, Zewif, ZewifWallet,
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
    println!("Testing address migration for wallet: {}", path.display());

    // Load the wallet using the same method as in zcashd_cmd::dump_wallet
    let db_dump = BDBDump::from_file(&path)?;
    let zcashd_dump = ZcashdDump::from_bdb_dump(&db_dump)?;
    let (zcashd_wallet, _) = ZcashdParser::parse_dump(&zcashd_dump)?;

    Ok((zcashd_dump, zcashd_wallet))
}

/// Helper struct to store address information for comparison
#[derive(Debug, Clone)]
struct AddressInfo {
    /// The string representation of the address
    #[allow(dead_code)]
    // This is used as a key in the HashMap, not directly accessed
    address_str: String,
    /// The type of the address (transparent, sapling, unified)
    address_type: String,
    /// The name/label of the address (if any)
    name: Option<String>,
    /// The purpose of the address (if any)
    purpose: Option<String>,
    /// Whether the address has an incoming viewing key
    has_ivk: bool,
    /// Whether the address has a spending key
    has_spending_key: bool,
}

/// Extract addresses and their info from a ZCashd wallet
///
/// This function extracts all addresses and their associated metadata
/// from a ZCashd wallet to facilitate comparison after migration.
///
/// # Parameters
/// - `wallet`: The ZCashd wallet to extract addresses from
///
/// # Returns
/// - HashMap mapping address strings to their associated metadata
fn extract_addresses_from_zcashd(
    wallet: &ZcashdWallet,
) -> HashMap<String, AddressInfo> {
    let mut addresses = HashMap::new();

    // Extract transparent addresses
    for (address, name) in wallet.address_names() {
        // Convert to string for lookup
        let address_str = address.to_string();
        let purpose = wallet.address_purposes().get(address).cloned();

        let address_info = AddressInfo {
            address_str: address_str.clone(),
            address_type: "transparent".to_string(),
            name: Some(name.clone()),
            purpose,
            has_ivk: false, // Transparent addresses don't have IVKs
            has_spending_key: false, // We don't track this for transparent addresses in our tests currently
        };

        addresses.insert(address_str, address_info);
    }

    // Extract sapling addresses
    for (sapling_address, ivk) in wallet.sapling_z_addresses() {
        let address_str = sapling_address.to_string(wallet.network());

        // Get purpose if available
        let zcashd_address = zewif_zcashd::Address::from(address_str.clone());
        let purpose = wallet.address_purposes().get(&zcashd_address).cloned();

        // Check if we have a spending key for this address
        let has_spending_key = wallet.sapling_keys().get(ivk).is_some();

        let address_info = AddressInfo {
            address_str: address_str.clone(),
            address_type: "sapling".to_string(),
            name: None, // Sapling addresses don't have names in our current ZCashd implementation
            purpose,
            has_ivk: true, // By definition, this has an IVK since we're iterating over sapling_z_addresses
            has_spending_key,
        };

        addresses.insert(address_str, address_info);
    }

    // Note: Unified addresses aren't directly supported in the current ZCashd implementation
    // If we add support for them later, we would extract them here

    addresses
}

/// Extract addresses and their info from a ZeWIF wallet
///
/// This function extracts all addresses and their associated metadata
/// from a ZeWIF wallet to facilitate comparison after migration.
///
/// # Parameters
/// - `wallet`: The ZeWIF wallet to extract addresses from
///
/// # Returns
/// - HashMap mapping address strings to their associated metadata
fn extract_addresses_from_zewif_wallet(
    wallet: &ZewifWallet,
) -> HashMap<String, AddressInfo> {
    let mut addresses = HashMap::new();

    // Iterate through all accounts in the wallet
    for account in wallet.accounts() {
        // Examine each address in the account
        for address in account.addresses() {
            let address_str = address.as_string();
            let address_type;
            let has_ivk;
            let has_spending_key;

            // Determine address type and check for IVK/spending key
            match address.address() {
                ProtocolAddress::Transparent(_) => {
                    address_type = "transparent".to_string();
                    has_ivk = false; // Transparent addresses don't have IVKs
                    has_spending_key = false; // We don't track this for transparent addresses currently
                }
                ProtocolAddress::Shielded(shielded) => {
                    address_type = "sapling".to_string(); // Default to sapling for shielded addresses
                    has_ivk = shielded.incoming_viewing_key().is_some();
                    has_spending_key = shielded.spending_key().is_some();
                }
                ProtocolAddress::Unified(unified) => {
                    address_type = "unified".to_string();
                    has_ivk = unified.sapling_component().is_some()
                        && unified
                            .sapling_component()
                            .unwrap()
                            .incoming_viewing_key()
                            .is_some();
                    has_spending_key = unified.sapling_component().is_some()
                        && unified
                            .sapling_component()
                            .unwrap()
                            .spending_key()
                            .is_some();
                }
            }

            let address_info = AddressInfo {
                address_str: address_str.clone(),
                address_type,
                name: Some(address.name().to_string()),
                purpose: address.purpose().map(String::from),
                has_ivk,
                has_spending_key,
            };

            addresses.insert(address_str, address_info);
        }
    }

    addresses
}

/// Extract addresses and their info from a ZeWIF top-level container
///
/// This function extends `extract_addresses_from_zewif` to work with the top-level ZeWIF
/// container, which may contain multiple wallets.
///
/// # Parameters
/// - `zewif`: Reference to a ZeWIF top-level container
///
/// # Returns
/// - HashMap mapping address strings to their associated metadata across all wallets
fn extract_addresses_from_zewif(zewif: &Zewif) -> HashMap<String, AddressInfo> {
    let mut all_addresses = HashMap::new();

    // Get all wallets from the top container
    for wallet in zewif.wallets() {
        // Extract addresses from each wallet and combine them
        let wallet_addresses = extract_addresses_from_zewif_wallet(wallet);
        all_addresses.extend(wallet_addresses);
    }

    all_addresses
}

/// Get a count of addresses by type from an address map
///
/// This helper function counts how many addresses of each type exist
/// in the provided address map.
///
/// # Parameters
/// - `addresses`: HashMap of address string to AddressInfo
///
/// # Returns
/// - HashMap mapping address type to count
fn count_address_types(
    addresses: &HashMap<String, AddressInfo>,
) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    for info in addresses.values() {
        *counts.entry(info.address_type.clone()).or_insert(0) += 1;
    }

    counts
}

/// Tests that all address types are properly preserved during migration
///
/// This test verifies that transparent and sapling addresses are correctly
/// extracted and preserved during migration to the ZeWIF format.
#[test]
fn test_address_type_preservation() -> Result<()> {
    // Test with several different wallet fixtures to ensure wide coverage
    let wallet_paths = [
        &["zcashd", "golden-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "tarnished-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "wallet0.dat"][..], // Use slice syntax to make array sizes compatible
    ];

    for path_elements in wallet_paths {
        // Load wallet and dump
        let (_, zcashd_wallet) = load_zcashd_wallet(path_elements)?;

        // Extract addresses from the ZCashd wallet
        let addresses_before = extract_addresses_from_zcashd(&zcashd_wallet);
        let address_counts_before = count_address_types(&addresses_before);

        println!(
            "Found addresses before migration: {:?}",
            address_counts_before
        );

        // Convert to ZeWIF format
        let zewif = zewif_zcashd::migrate_to_zewif(&zcashd_wallet)?;

        // Extract addresses from the migrated ZeWIF wallet
        let addresses_after = extract_addresses_from_zewif(&zewif);
        let address_counts_after = count_address_types(&addresses_after);

        println!(
            "Found addresses after migration: {:?}",
            address_counts_after
        );

        // Verify that all address types are preserved (count should match for each type)
        for (address_type, count) in &address_counts_before {
            assert_eq!(
                address_counts_after.get(address_type).unwrap_or(&0),
                count,
                "Number of {} addresses doesn't match after migration",
                address_type
            );
        }

        // Verify that all original addresses are preserved
        // Note: The total count might not match because unified addresses might be added
        // during migration from the UnifiedAccounts structure
        assert!(
            addresses_after.len() >= addresses_before.len(),
            "Migrated wallet has fewer addresses than the original"
        );

        // Verify that all original addresses exist after migration
        for addr_str in addresses_before.keys() {
            assert!(
                addresses_after.contains_key(addr_str),
                "Address {} was not preserved during migration",
                addr_str
            );
        }
    }

    Ok(())
}

/// Tests that address metadata is properly preserved during migration
///
/// This test verifies that address metadata, such as names, purposes,
/// and associated keys, are correctly preserved during migration.
#[test]
fn test_address_metadata_preservation() -> Result<()> {
    // Test with several different wallet fixtures to ensure wide coverage
    let wallet_paths = [
        &["zcashd", "golden-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "tarnished-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "wallet0.dat"][..],
    ];

    for path_elements in wallet_paths {
        // Load wallet and dump
        let (_, zcashd_wallet) = load_zcashd_wallet(path_elements)?;

        // Extract addresses from the ZCashd wallet
        let addresses_before = extract_addresses_from_zcashd(&zcashd_wallet);

        // Convert to ZeWIF format
        let zewif = zewif_zcashd::migrate_to_zewif(&zcashd_wallet)?;

        // Extract addresses from the migrated ZeWIF wallet
        let addresses_after = extract_addresses_from_zewif(&zewif);

        // Verify metadata preservation for each address
        for (addr_str, before_info) in &addresses_before {
            let after_info = addresses_after
                .get(addr_str)
                .expect("Address not found after migration");

            // Verify address type is preserved
            assert_eq!(
                before_info.address_type, after_info.address_type,
                "Address type doesn't match for {}",
                addr_str
            );

            // Verify purpose is preserved (if present)
            if let Some(purpose) = &before_info.purpose {
                assert_eq!(
                    after_info.purpose.as_ref(),
                    Some(purpose),
                    "Purpose doesn't match for {}",
                    addr_str
                );
            }

            // For transparent addresses, verify name is preserved
            if before_info.address_type == "transparent" {
                assert_eq!(
                    after_info.name.as_ref(),
                    before_info.name.as_ref(),
                    "Name doesn't match for transparent address {}",
                    addr_str
                );
            }

            // For sapling addresses, verify IVK status is preserved
            if before_info.address_type == "sapling" {
                assert_eq!(
                    before_info.has_ivk, after_info.has_ivk,
                    "IVK status doesn't match for sapling address {}",
                    addr_str
                );

                // Verify spending key status is preserved
                assert_eq!(
                    before_info.has_spending_key, after_info.has_spending_key,
                    "Spending key status doesn't match for sapling address {}",
                    addr_str
                );
            }
        }
    }

    Ok(())
}

/// Tests address assignment to accounts
///
/// This test verifies that addresses are correctly assigned to the default account
/// when no unified accounts are present.
#[test]
fn test_address_default_account_assignment() -> Result<()> {
    // Test with several different wallet fixtures to ensure wide coverage
    let wallet_paths = [
        &["zcashd", "golden-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "tarnished-v5.6.0", "node0_wallet.dat"],
        &["zcashd", "wallet0.dat"][..],
    ];

    for path_elements in wallet_paths {
        // Load wallet and dump
        let (_, zcashd_wallet) = load_zcashd_wallet(path_elements)?;

        // Extract addresses from the ZCashd wallet to get our baseline
        let addresses_before = extract_addresses_from_zcashd(&zcashd_wallet);

        // Convert to ZeWIF format
        let zewif = zewif_zcashd::migrate_to_zewif(&zcashd_wallet)?;

        // In default mode, all addresses should be in the default account
        // Verify this by checking that all addresses are in a single account
        let wallet = zewif.wallets().iter().next().expect("No wallet found");

        // Check if this wallet has unified accounts
        let has_unified_accounts = zcashd_wallet.unified_accounts().is_some();

        if !has_unified_accounts {
            // For wallets without unified accounts, we should have a single default account
            assert_eq!(
                wallet.accounts().len(),
                1,
                "Default migration should result in a single account for wallets without unified accounts"
            );

            let default_account =
                wallet.accounts().iter().next().expect("No account found");
            assert_eq!(
                default_account.addresses().len(),
                addresses_before.len(),
                "Default account should contain all addresses"
            );
        } else {
            // For wallets with unified accounts, addresses will be spread across multiple accounts
            // The unified accounts feature in ZCash is more complex and can create addresses
            // that aren't directly visible in the address list. We can only verify that:
            // 1. We have multiple accounts
            // 2. All the addresses we found in the source wallet are present in some account

            // Verify we have multiple accounts
            assert!(
                wallet.accounts().len() > 1,
                "Wallet with unified accounts should have multiple accounts, found {}",
                wallet.accounts().len()
            );

            // Extract all addresses from all accounts
            let addresses_after = extract_addresses_from_zewif(&zewif);

            // Verify all source addresses are present in the migrated wallet
            for addr_str in addresses_before.keys() {
                assert!(
                    addresses_after.contains_key(addr_str),
                    "Address {} was not preserved during migration",
                    addr_str
                );
            }
        }
    }

    Ok(())
}

/// Tests edge cases in address migration
///
/// This test verifies that edge cases, such as addresses without metadata
/// or with unusual configurations, are handled correctly during migration.
#[test]
fn test_address_migration_edge_cases() -> Result<()> {
    // Create a synthetic wallet with edge case addresses
    let mut wallet = ZewifWallet::new(Network::Test);
    let mut account = Account::new();

    // Case 1: A shielded address without an IVK
    let no_ivk_addr_str =
        "zs1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq93gnn49";
    let no_ivk = ShieldedAddress::new(no_ivk_addr_str.to_string());
    let no_ivk_addr = Address::new(ProtocolAddress::Shielded(no_ivk));

    // Case 2: A transparent address with a name but no purpose
    let named_addr_str = "t1a1zfft6qthfspr3khk7mpkqnufzgspzwj9l5cuq";
    let named_transparent = TransparentAddress::new(named_addr_str.to_string());
    let mut named_addr =
        Address::new(ProtocolAddress::Transparent(named_transparent));
    named_addr.set_name("Named Address".to_string());

    // Case 3: A transparent address with a purpose but no name
    let purposed_addr_str = "t3JZcvsuQZFENyDtJiTmQChs6Zb6VwYM8r3";
    let purposed_transparent =
        TransparentAddress::new(purposed_addr_str.to_string());
    let mut purposed_addr =
        Address::new(ProtocolAddress::Transparent(purposed_transparent));
    purposed_addr.set_purpose("change".to_string());
    purposed_addr.set_name("".to_string()); // Explicit empty name

    // Add addresses to account
    account.add_address(no_ivk_addr);
    account.add_address(named_addr);
    account.add_address(purposed_addr);
    wallet.add_account(account);

    // Now create a Zewif container with this wallet
    let mut zewif = Zewif::new();
    zewif.add_wallet(wallet);

    // Extract addresses and verify our edge cases
    let addresses = extract_addresses_from_zewif(&zewif);

    // Verify Case 1: Shielded address without IVK
    let no_ivk_info =
        addresses.get(no_ivk_addr_str).expect("Address not found");
    assert_eq!(
        no_ivk_info.address_type, "sapling",
        "Should be a sapling address"
    );
    assert!(!no_ivk_info.has_ivk, "Should not have an IVK");

    // Verify Case 2: Named transparent address
    let named_info = addresses.get(named_addr_str).expect("Address not found");
    assert_eq!(
        named_info.address_type, "transparent",
        "Should be a transparent address"
    );
    assert_eq!(
        named_info.name.as_ref().unwrap(),
        "Named Address",
        "Name should match"
    );
    assert_eq!(named_info.purpose, None, "Should not have a purpose");

    // Verify Case 3: Purposed transparent address with empty name
    let purposed_info =
        addresses.get(purposed_addr_str).expect("Address not found");
    assert_eq!(
        purposed_info.address_type, "transparent",
        "Should be a transparent address"
    );
    assert_eq!(
        purposed_info.name.as_ref().unwrap(),
        "",
        "Should have an empty name"
    );
    assert_eq!(
        purposed_info.purpose.as_ref().unwrap(),
        "change",
        "Purpose should match"
    );

    Ok(())
}
