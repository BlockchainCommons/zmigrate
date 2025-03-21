use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use ripemd::{Digest, Ripemd160};
use sha2::Sha256;

use crate::{
    Blob32, Data, ProtocolAddress, SaplingIncomingViewingKey, TxId, u256,
    zcashd::{self, ZcashdWallet},
    zewif::{self, Account, Attachments, Position, ZewifTop, ZewifWallet},
};

/// Migrate a ZCashd wallet to the Zewif wallet format
pub fn migrate_to_zewif(wallet: &ZcashdWallet) -> Result<ZewifTop> {
    // Create a new ZewifDB
    let mut zewif_top = ZewifTop::new();

    // Convert seed material (mnemonic phrase)
    let seed_material = convert_seed_material(wallet)?;

    // Create a complete Zewif wallet
    let mut zewif_wallet = ZewifWallet::new(wallet.network());

    if let Some(seed_material) = seed_material {
        zewif_wallet.set_seed_material(seed_material);
    }

    // Process transactions and collect relevant transaction IDs
    let mut transactions = convert_transactions(wallet)?;

    // Convert orchard note commitment tree if available
    if !wallet
        .orchard_note_commitment_tree
        .unparsed_data
        .0
        .is_empty()
    {
        // Update transaction outputs with note positions from the note commitment tree
        update_transaction_positions(wallet, &mut transactions)?;
    }

    // If there are unified accounts, process them
    if let Some(unified_accounts) = &wallet.unified_accounts {
        // Create accounts based on unified_accounts structure
        let accounts_map = convert_unified_accounts(wallet, unified_accounts, &transactions)?;

        // Add all accounts to the wallet
        for account in accounts_map.values() {
            zewif_wallet.add_account(account.clone());
        }
    } else {
        // No unified accounts - create a single default account
        let mut default_account = Account::new();
        default_account.set_name("Default Account");

        // Convert transparent addresses
        convert_transparent_addresses(wallet, &mut default_account)?;

        // Convert sapling addresses
        convert_sapling_addresses(wallet, &mut default_account)?;

        // Add all transaction IDs to the default account's relevant transactions
        for txid in transactions.keys() {
            default_account.add_relevant_transaction(*txid);
        }

        // Add the default account to the wallet
        zewif_wallet.add_account(default_account);
    }

    // Add wallet and transactions to the ZewifDB
    zewif_top.add_wallet(zewif_wallet);
    zewif_top.transactions = transactions;

    Ok(zewif_top)
}

/// Convert ZCashd mnemonic seed to Zewif SeedMaterial
fn convert_seed_material(wallet: &ZcashdWallet) -> Result<Option<zewif::SeedMaterial>> {
    // Check if we have a mnemonic phrase
    if !wallet.bip39_mnemonic.mnemonic().is_empty() {
        return Ok(Some(zewif::SeedMaterial::Bip39Mnemonic(
            wallet.bip39_mnemonic.mnemonic().clone(),
        )));
    }
    // If no mnemonic, return None
    Ok(None)
}

/// Convert ZCashd transparent addresses to Zewif format
/// This version is used only when there are no unified accounts
fn convert_transparent_addresses(
    wallet: &ZcashdWallet,
    account: &mut zewif::Account,
) -> Result<()> {
    // Process address_names which contain transparent addresses
    for (zcashd_address, name) in &wallet.address_names {
        let transparent_address = zewif::TransparentAddress::new(zcashd_address.0.clone());

        // Create a ZewifAddress from the TransparentAddress
        let protocol_address = ProtocolAddress::Transparent(transparent_address);
        let mut zewif_address = zewif::Address::new(protocol_address);
        zewif_address.set_name(name.clone());

        // Set purpose if available
        if let Some(purpose) = wallet.address_purposes.get(zcashd_address) {
            zewif_address.set_purpose(purpose.clone());
        }

        // Add the address to the account with its string representation as key
        account.add_address(zewif_address);
    }

    Ok(())
}

/// Convert ZCashd sapling addresses to Zewif format
/// This version is used only when there are no unified accounts
fn convert_sapling_addresses(wallet: &ZcashdWallet, account: &mut zewif::Account) -> Result<()> {
    // Process sapling_z_addresses
    for (sapling_address, viewing_key) in &wallet.sapling_z_addresses {
        let address_str = sapling_address.to_string(wallet.network());

        // Create a new ShieldedAddress
        let mut shielded_address = zewif::ShieldedAddress::new(address_str.clone());
        shielded_address.set_incoming_viewing_key(viewing_key.to_owned());

        // Add spending key if available in sapling_keys
        if let Some(sapling_key) = find_sapling_key_for_ivk(wallet, viewing_key) {
            // Convert to Zewif spending key format
            let spending_key = convert_sapling_spending_key(&sapling_key.key)
                .context("Failed to convert sapling spending key")?;
            shielded_address.set_spending_key(spending_key);
        }

        let protocol_address = zewif::ProtocolAddress::Shielded(shielded_address);
        let mut zewif_address = zewif::Address::new(protocol_address);

        // Set purpose if available - convert to Address type for lookup
        let zcashd_address = zcashd::Address(address_str.clone());
        if let Some(purpose) = wallet.address_purposes.get(&zcashd_address) {
            zewif_address.set_purpose(purpose.clone());
        }

        // Add the address to the account with its string representation as key
        account.add_address(zewif_address);
    }

    Ok(())
}

/// Find a SaplingKey for a given incoming viewing key
fn find_sapling_key_for_ivk<'a>(
    wallet: &'a ZcashdWallet,
    ivk: &SaplingIncomingViewingKey,
) -> Option<&'a zcashd::SaplingKey> {
    wallet.sapling_keys.0.get(ivk)
}

/// Convert ZCashd SaplingExtendedSpendingKey to Zewif SpendingKey
fn convert_sapling_spending_key(
    key: &crate::sapling::SaplingExtendedSpendingKey,
) -> Result<zewif::SpendingKey> {
    // Create the Sapling spending key with all components including HD parameters
    // Since both structures use u256, we can directly use them without cloning
    let spending_key = zewif::SpendingKey::new_sapling_extended(
        key.expsk.ask,
        key.expsk.nsk,
        key.expsk.ovk,
        key.depth,
        key.parent_fvk_tag,
        key.child_index,
        key.chain_code,
        key.dk,
    );

    Ok(spending_key)
}

/// Extract all addresses involved in a transaction
fn extract_transaction_addresses(
    wallet: &ZcashdWallet,
    tx_id: &TxId,
    tx: &zcashd::WalletTx,
) -> Result<HashSet<String>> {
    let mut addresses = HashSet::new();

    // Check if we have recipient mappings for this transaction
    if let Some(recipients) = wallet.send_recipients.get(tx_id) {
        for recipient in recipients {
            // Add the unified address if it exists
            if !recipient.unified_address.is_empty() {
                addresses.insert(recipient.unified_address.clone());
            }

            // Add the recipient address based on the type
            match &recipient.recipient_address {
                zcashd::RecipientAddress::Sapling(addr) => {
                    let addr_str = addr.to_string(wallet.network());
                    addresses.insert(addr_str);
                }
                zcashd::RecipientAddress::Orchard(addr) => {
                    let addr_str = addr.to_string(wallet.network());
                    addresses.insert(addr_str);
                }
                zcashd::RecipientAddress::KeyId(key_id) => {
                    // Convert P2PKH key hash to a Zcash address
                    let addr_str = key_id.to_string(wallet.network());
                    addresses.insert(addr_str);
                }
                zcashd::RecipientAddress::ScriptId(script_id) => {
                    // Convert P2SH script hash to a Zcash address
                    let addr_str = script_id.to_string(wallet.network());
                    addresses.insert(addr_str);
                }
            }
        }
    }

    // For transparent inputs, extract addresses from the script signatures
    for tx_in in &tx.vin {
        // We'll derive a unique identifier from the previous outpoint to ensure we capture this transaction
        let txid_str = format!("{}", tx_in.prevout.txid);
        let input_addr = format!("input:{}:{}", txid_str, tx_in.prevout.vout);
        addresses.insert(input_addr);

        // Extract potential P2PKH or P2SH addresses from script signatures
        let script_data = &tx_in.script_sig.0.0;

        // We're looking for common script signature patterns that might contain addresses
        // P2PKH scriptSigs typically have format: <sig> <pubkey>
        // P2SH scriptSigs typically have format: <...> <redeemscript>

        // For P2PKH signatures, the pubkey is near the end and we can extract it
        if script_data.len() > 33 {
            // Minimum size for a compressed pubkey (33 bytes)
            let potential_pubkey = &script_data[script_data.len() - 33..];

            // Check if it could be a compressed pubkey (starts with 0x02 or 0x03)
            if potential_pubkey[0] == 0x02 || potential_pubkey[0] == 0x03 {
                // Hash the pubkey to get the pubkey hash (RIPEMD160(SHA256(pubkey)))
                // First calculate SHA256 hash
                let mut sha256 = Sha256::new();
                sha256.update(potential_pubkey);
                let sha256_result = sha256.finalize();

                // Calculate RIPEMD160 hash of the SHA256 result
                let mut ripemd160 = Ripemd160::new();
                ripemd160.update(sha256_result);
                let pubkey_hash = ripemd160.finalize();

                // Create a transparent P2PKH address from this pubkey hash
                // Create a KeyId for consistent address encoding
                let key_id = zcashd::KeyId(
                    crate::u160::from_slice(&pubkey_hash[..])
                        .expect("Creating u160 from RIPEMD160 hash"),
                );
                addresses.insert(key_id.to_string(wallet.network()));
            }
        }
    }

    // For transparent outputs, extract addresses from the scriptPubKey
    for (vout_idx, tx_out) in tx.vout.iter().enumerate() {
        let script_data = &tx_out.script_pub_key.0.0;

        // P2PKH detection - match the pattern: OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG
        if script_data.len() >= 25 && script_data[0] == 0x76 && script_data[1] == 0xA9 {
            if script_data.len() >= 25 && script_data[23] == 0x88 && script_data[24] == 0xAC {
                // The pubkey hash is 20 bytes starting at offset 3
                let pubkey_hash = &script_data[3..23];

                // Convert to a proper P2PKH Zcash address using KeyId
                let key_id = zcashd::KeyId(
                    crate::u160::from_slice(pubkey_hash).expect("Creating u160 from pubkey hash"),
                );
                addresses.insert(key_id.to_string(wallet.network()));
            }
        }
        // P2SH detection - match the pattern: OP_HASH160 <scriptHash> OP_EQUAL
        else if script_data.len() >= 23
            && script_data[0] == 0xA9
            && script_data.len() >= 23
            && script_data[22] == 0x87
        {
            // The script hash is 20 bytes starting at offset 2
            let script_hash = &script_data[2..22];

            // Convert to a proper P2SH Zcash address using ScriptId
            let script_id = zcashd::ScriptId(
                crate::u160::from_slice(script_hash).expect("Creating u160 from script hash"),
            );
            addresses.insert(script_id.to_string(wallet.network()));
        }

        // Always add an output identifier that links to this transaction
        let output_addr = format!("output:{}:{}", tx_id, vout_idx);
        addresses.insert(output_addr);
    }

    // For Sapling spends and outputs
    match &tx.sapling_bundle {
        zcashd::SaplingBundle::V4(bundle_v4) => {
            for spend in &bundle_v4.spends {
                // The nullifier uniquely identifies the spend
                // Use AsRef to get a reference to the underlying bytes
                let nullifier_hex = hex::encode(spend.nullifier.as_ref() as &[u8]);
                addresses.insert(format!("sapling_spend:{}", nullifier_hex));

                // If we have note data for this nullifier, we might find the address
                if let Some(sapling_note_data) = &tx.sapling_note_data {
                    for note_data in sapling_note_data.values() {
                        if let Some(nullifier) = &note_data.nullifer {
                            if *nullifier == spend.nullifier {
                                // Find the address for this viewing key
                                for (addr, ivk) in &wallet.sapling_z_addresses {
                                    if &note_data.incoming_viewing_key == ivk {
                                        let addr_str = addr.to_string(wallet.network());
                                        addresses.insert(addr_str);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            for output in &bundle_v4.outputs {
                // The commitment uniquely identifies the output
                // Use AsRef to get a reference to the underlying bytes
                let cm_hex = hex::encode(output.cmu.as_ref() as &[u8]);
                addresses.insert(format!("sapling_output:{}", cm_hex));

                // If we have note data for this output, we might find the address
                if let Some(sapling_note_data) = &tx.sapling_note_data {
                    for note_data in sapling_note_data.values() {
                        // We'd need to link the outpoint to this specific output
                        // Since we don't have enough information, we'll use the IVK
                        for (addr, ivk) in &wallet.sapling_z_addresses {
                            if &note_data.incoming_viewing_key == ivk {
                                let addr_str = addr.to_string(wallet.network());
                                addresses.insert(addr_str);
                                break;
                            }
                        }
                    }
                }
            }
        }
        zcashd::SaplingBundle::V5(bundle_v5) => {
            // Similar processing for V5 bundles
            // V5 has the same structure for spends and outputs
            for spend in &bundle_v5.shielded_spends {
                // Use AsRef to get a reference to the underlying bytes
                let nullifier_hex = hex::encode(spend.nullifier().as_ref() as &[u8]);
                addresses.insert(format!("sapling_spend_v5:{}", nullifier_hex));
            }

            for output in &bundle_v5.shielded_outputs {
                // Use AsRef to get a reference to the underlying bytes
                let cm_hex = hex::encode(output.cmu().as_ref() as &[u8]);
                addresses.insert(format!("sapling_output_v5:{}", cm_hex));
            }
        }
    }

    // Process sapling note data directly
    if let Some(sapling_note_data) = &tx.sapling_note_data {
        for note_data in sapling_note_data.values() {
            // If we have the incoming viewing key, we can find the corresponding address
            for (addr, ivk) in &wallet.sapling_z_addresses {
                if &note_data.incoming_viewing_key == ivk {
                    let addr_str = addr.to_string(wallet.network());
                    addresses.insert(addr_str);
                    break;
                }
            }
        }
    }

    // Handle Orchard actions if present
    if let zcashd::OrchardBundle(Some(orchard_bundle)) = &tx.orchard_bundle {
        // Extract data from Orchard actions
        for (idx, action) in orchard_bundle.actions.iter().enumerate() {
            // Add standard identifiers like nullifier and commitment
            let nullifier_hex = hex::encode(action.nf_old);
            addresses.insert(format!("orchard_nullifier:{}", nullifier_hex));

            // Extract potential address information if available
            if let Some(orchard_meta) = &tx.orchard_tx_meta {
                if let Some(_action_data) = orchard_meta.action_data.get(&(idx as u32)) {
                    // Try to recover the Orchard address components if we have enough data
                    // We don't have direct access to the receiver's complete address data here,
                    // but in a complete implementation, we'd follow the path:
                    // output_data -> full viewing key -> orchard receiver -> derive address

                    // For now, since we're missing the full path, we'll use what we have
                    // to create a unique identifier that links to metadata
                    let output_id = format!("orchard_output:{}:{}", tx_id, idx);
                    addresses.insert(output_id);

                    // If the outgoing viewing key is related to ours, we can possibly
                    // derive additional information, but that's complex and
                    // would be out of scope for this implementation
                }
            }

            // Include any Orchard address we know of directly (e.g., from recipient mappings)
            // which would have been added already when processing recipient_mappings

            // Also add the action index as a unique identifier
            addresses.insert(format!("orchard_action_idx:{}:{}", tx_id, idx));
        }
    }

    // If the transaction is marked as "from me", and we don't have other identifying information,
    // use all our addresses as potential sources
    if tx.from_me && addresses.is_empty() {
        for addr in wallet.sapling_z_addresses.keys() {
            let addr_str = addr.to_string(wallet.network());
            addresses.insert(addr_str);
        }

        // Also add transparent addresses if any are associated with this wallet
        for addr in wallet.address_names.keys() {
            addresses.insert(addr.0.clone());
        }
    }

    // Add the transaction ID itself as a last resort identifier
    addresses.insert(format!("tx:{}", tx_id));

    Ok(addresses)
}

/// Convert ZCashd transactions to Zewif format
fn convert_transactions(wallet: &ZcashdWallet) -> Result<HashMap<TxId, zewif::Transaction>> {
    let mut transactions = HashMap::new();

    for (tx_id, wallet_tx) in &wallet.transactions {
        let zewif_tx = convert_transaction(*tx_id, wallet_tx)
            .with_context(|| format!("Failed to convert transaction {}", tx_id))?;
        transactions.insert(*tx_id, zewif_tx);
    }

    Ok(transactions)
}

/// Convert a single ZCashd transaction to Zewif format
fn convert_transaction(tx_id: TxId, tx: &zcashd::WalletTx) -> Result<zewif::Transaction> {
    let mut zewif_tx = zewif::Transaction::new(tx_id);

    // Set raw transaction data
    if !tx.unparsed_data.is_empty() {
        zewif_tx.set_raw(tx.unparsed_data.clone());
    }

    // Add basic transaction metadata
    // Convert block height if we can infer it from hash_block
    // For this prototype, we'll just leave it as None

    // Convert transparent inputs
    for tx_in in &tx.vin {
        let zewif_tx_in = zewif::TxIn {
            previous_output: zewif::TxOutPoint {
                txid: tx_in.prevout.txid,
                index: tx_in.prevout.vout,
            },
            script_sig: Data(tx_in.script_sig.0.0.clone()),
            sequence: tx_in.sequence,
        };
        zewif_tx.add_input(zewif_tx_in);
    }

    // Convert transparent outputs
    for tx_out in &tx.vout {
        let amount = tx_out.value;
        let script_pubkey = Data(tx_out.script_pub_key.0.0.clone());

        let zewif_tx_out = zewif::TxOut { value: amount, script_pubkey };
        zewif_tx.add_output(zewif_tx_out);
    }

    // Convert Sapling spends and outputs
    match &tx.sapling_bundle {
        zcashd::SaplingBundle::V4(bundle_v4) => {
            // Convert Sapling spends
            for (idx, spend) in bundle_v4.spends.iter().enumerate() {
                let sapling_spend = zewif::SaplingSpendDescription {
                    spend_index: idx as u32,
                    value: Some(
                        zewif::Amount::try_from(i64::from(bundle_v4.amount))
                            .with_context(|| "Converting spend value")?,
                    ),
                    anchor_height: None, // Don't have this data
                    nullifier: spend.nullifier,
                    zkproof: Data(spend.zkproof.0.to_vec()),
                    attachments: Attachments::new(),
                };
                zewif_tx.add_sapling_spend(sapling_spend);
            }

            // Convert Sapling outputs
            for (idx, output) in bundle_v4.outputs.iter().enumerate() {
                let sapling_output = zewif::SaplingOutputDescription {
                    output_index: idx as u32,
                    commitment: output.cmu,
                    ephemeral_key: output.ephemeral_key,
                    enc_ciphertext: Data(output.enc_ciphertext.0.to_vec()),
                    memo: None, // We don't have this decrypted
                    note_commitment_tree_position: zewif::Position(0), // Placeholder
                    witness: None, // Don't have witness data
                    attachments: Attachments::new(),
                };
                zewif_tx.add_sapling_output(sapling_output);
            }
        }
        zcashd::SaplingBundle::V5(bundle_v5) => {
            // Processing for V5 bundles
            for (idx, spend) in bundle_v5.shielded_spends.iter().enumerate() {
                let sapling_spend = zewif::SaplingSpendDescription {
                    spend_index: idx as u32,
                    value: None, // V5 doesn't expose values
                    anchor_height: None,
                    nullifier: spend.nullifier(),
                    zkproof: Data(spend.zkproof().0.to_vec()),
                    attachments: Attachments::new(),
                };
                zewif_tx.add_sapling_spend(sapling_spend);
            }

            for (idx, output) in bundle_v5.shielded_outputs.iter().enumerate() {
                let sapling_output = zewif::SaplingOutputDescription {
                    output_index: idx as u32,
                    commitment: output.cmu(),
                    ephemeral_key: output.ephemeral_key(),
                    enc_ciphertext: Data(output.enc_ciphertext().to_vec()),
                    memo: None,
                    note_commitment_tree_position: zewif::Position(0), // Placeholder
                    witness: None,
                    attachments: Attachments::new(),
                };
                zewif_tx.add_sapling_output(sapling_output);
            }
        }
    }

    // Convert Orchard actions
    if let zcashd::OrchardBundle(Some(orchard_bundle)) = &tx.orchard_bundle {
        for (idx, action) in orchard_bundle.actions.iter().enumerate() {
            let orchard_action = zewif::OrchardActionDescription {
                action_index: idx as u32,
                anchor: u256::default(), // Anchor not directly available in OrchardAction
                nullifier: action.nf_old,
                zkproof: Data(Vec::new()), // Proof not directly available in OrchardAction
                commitment: action.cmx,    // Using cmx as commitment
                ephemeral_key: u256::default(), // Not available
                enc_ciphertext: Data(action.encrypted_note.enc_ciphertext().to_vec()),
                memo: None,
                note_commitment_tree_position: zewif::Position(0), // Placeholder
                witness: None,
                attachments: Attachments::new(),
            };
            zewif_tx.add_orchard_action(orchard_action);
        }
    }

    // Convert Sprout JoinSplits if present
    if let Some(join_splits) = &tx.join_splits {
        for js in &join_splits.descriptions {
            // Create arrays using from_fn to avoid needing Copy
            let nullifiers = js.nullifiers;
            let commitments = js.commitments;

            let join_split = zewif::JoinSplitDescription {
                anchor: zewif::Anchor(
                    Blob32::from_slice(js.anchor.as_ref()).expect("Converting anchor"),
                ),
                nullifiers,
                commitments,
                zkproof: Data(match &js.zkproof {
                    zcashd::SproutProof::PHGRProof(proof) => proof.to_bytes(),
                    zcashd::SproutProof::GrothProof(proof) => proof.0.to_vec(),
                }),
                attachments: Attachments::new(),
            };
            zewif_tx.add_sprout_joinsplit(join_split);
        }
    }

    Ok(zewif_tx)
}

/// Convert ZCashd UnifiedAccounts to Zewif accounts
fn convert_unified_accounts(
    wallet: &ZcashdWallet,
    unified_accounts: &zcashd::UnifiedAccounts,
    _transactions: &HashMap<TxId, zewif::Transaction>,
) -> Result<HashMap<u256, Account>> {
    let mut accounts_map = HashMap::new();

    // Step 1: Create an account for each UnifiedAccountMetadata
    for (key_id, account_metadata) in &unified_accounts.account_metadata {
        // Create a new account with the appropriate ZIP-32 account ID
        let mut account = Account::new();

        // Set the account name and ZIP-32 account ID
        let account_name = format!("Account #{}", account_metadata.account_id);
        account.set_name(account_name);
        account.set_zip32_account_id(account_metadata.account_id);

        // Store the account in our map using the key_id as the key
        accounts_map.insert(*key_id, account);
    }

    // If no accounts were created, create a default account
    if accounts_map.is_empty() {
        let mut default_account = Account::new();
        default_account.set_name("Default Account");
        accounts_map.insert(u256::default(), default_account);
    }

    // Step 2: Build a mapping from address to account key_id based on UnifiedAddressMetadata
    let mut address_to_account_map = HashMap::new();

    // Map unified addresses to their accounts
    for (address_id, address_metadata) in &unified_accounts.address_metadata {
        // Store the mapping from address_id to account key_id
        address_to_account_map.insert(*address_id, address_metadata.key_id);
    }

    // Step 3: Process all addresses and assign them to the appropriate accounts

    // Process transparent addresses
    for (zcashd_address, name) in &wallet.address_names {
        // Try to find which account this address belongs to based on metadata
        // This is a placeholder - in a real implementation, we would use the
        // address_to_account_map to look up the correct account
        // For now, fallback to the first account

        // Default to first account
        let account_key_id = match accounts_map.keys().next() {
            Some(key) => *key,
            None => u256::default(),
        };

        if let Some(account) = accounts_map.get_mut(&account_key_id) {
            let transparent_address = zewif::TransparentAddress::new(zcashd_address.0.clone());

            // Create a ZewifAddress from the TransparentAddress
            let protocol_address = ProtocolAddress::Transparent(transparent_address);
            let mut zewif_address = zewif::Address::new(protocol_address);
            zewif_address.set_name(name.clone());

            // Set purpose if available
            if let Some(purpose) = wallet.address_purposes.get(zcashd_address) {
                zewif_address.set_purpose(purpose.clone());
            }

            // Add the address to the account
            account.add_address(zewif_address);
        }
    }

    // Process sapling addresses
    for (sapling_address, viewing_key) in &wallet.sapling_z_addresses {
        // Try to find which account this address belongs to based on metadata
        // This is a placeholder - in a real implementation, we would use the
        // address_to_account_map to look up the correct account
        // For now, fallback to the first account

        // Default to first account
        let account_key_id = match accounts_map.keys().next() {
            Some(key) => *key,
            None => u256::default(),
        };

        if let Some(account) = accounts_map.get_mut(&account_key_id) {
            let address_str = sapling_address.to_string(wallet.network());

            // Create a new ShieldedAddress
            let mut shielded_address = zewif::ShieldedAddress::new(address_str.clone());
            shielded_address.set_incoming_viewing_key(viewing_key.to_owned());

            // Add spending key if available in sapling_keys
            if let Some(sapling_key) = find_sapling_key_for_ivk(wallet, viewing_key) {
                // Convert to Zewif spending key format
                let spending_key = convert_sapling_spending_key(&sapling_key.key)
                    .context("Failed to convert sapling spending key")?;
                shielded_address.set_spending_key(spending_key);
            }

            let protocol_address = zewif::ProtocolAddress::Shielded(shielded_address);
            let mut zewif_address = zewif::Address::new(protocol_address);

            // Set purpose if available - convert to Address type for lookup
            let zcashd_address = zcashd::Address(address_str.clone());
            if let Some(purpose) = wallet.address_purposes.get(&zcashd_address) {
                zewif_address.set_purpose(purpose.clone());
            }

            // Add the address to the account
            account.add_address(zewif_address);
        }
    }

    // Step 4: Process viewing keys in unified_accounts
    // Each full_viewing_key entry maps a key_id to a viewing key string
    for key_id in unified_accounts.full_viewing_keys.keys() {
        // Find the account for this key_id
        if let Some(_account) = accounts_map.get_mut(key_id) {
            // TODO: Process and add the viewing key to the account
            // This will be implemented when we add specific support for viewing keys
        }
    }

    // Step 5: Assign transactions to relevant accounts based on address involvement
    // Build a mapping of addresses to accounts
    let mut address_to_account: HashMap<String, Vec<u256>> = HashMap::new();

    // Map each address to its containing account(s)
    for (account_id, account) in &accounts_map {
        for address in account.addresses().keys() {
            address_to_account
                .entry(address.clone())
                .or_default()
                .push(*account_id);
        }
    }

    // Analyze each transaction to find which addresses are involved
    for (txid, wallet_tx) in &wallet.transactions {
        // Extract all addresses involved in this transaction
        match extract_transaction_addresses(wallet, txid, wallet_tx) {
            Ok(tx_addresses) => {
                let mut relevant_accounts = HashSet::new();

                // Determine which accounts the transaction is relevant to
                for address in tx_addresses {
                    if let Some(account_ids) = address_to_account.get(&address) {
                        for account_id in account_ids {
                            relevant_accounts.insert(*account_id);
                        }
                    }
                }

                // If we couldn't determine relevant accounts, add to all accounts as fallback
                if relevant_accounts.is_empty() {
                    for account_id in accounts_map.keys() {
                        relevant_accounts.insert(*account_id);
                    }
                }

                // Add the transaction to relevant accounts
                for account_id in relevant_accounts {
                    if let Some(account) = accounts_map.get_mut(&account_id) {
                        account.add_relevant_transaction(*txid);
                    }
                }
            }
            Err(e) => {
                // Log the error and fall back to adding the transaction to all accounts
                eprintln!("Error analyzing transaction {}: {}", txid, e);
                for account in accounts_map.values_mut() {
                    account.add_relevant_transaction(*txid);
                }
            }
        }
    }

    Ok(accounts_map)
}

/// Update transaction outputs with note positions from the note commitment tree
fn update_transaction_positions(
    wallet: &ZcashdWallet,
    transactions: &mut HashMap<TxId, zewif::Transaction>,
) -> Result<()> {
    // Check if we have a valid tree to process
    if wallet
        .orchard_note_commitment_tree
        .unparsed_data
        .0
        .is_empty()
    {
        return Ok(());
    }

    // We'll use the note metadata to link commitments to positions
    // Map from commitment to position
    let mut commitment_positions: HashMap<u256, Position> = HashMap::new();

    // For each transaction with Orchard actions
    for (tx_id, zewif_tx) in transactions.iter_mut() {
        // Find the corresponding zcashd transaction to get metadata
        if let Some(zcashd_tx) = wallet.transactions.get(tx_id) {
            // Check for Orchard bundle
            if let zcashd::OrchardBundle(Some(_orchard_bundle)) = &zcashd_tx.orchard_bundle {
                // Check for Orchard transaction metadata
                if let Some(orchard_meta) = &zcashd_tx.orchard_tx_meta {
                    // Process each Orchard action if we have any
                    if let Some(orchard_actions) = zewif_tx.orchard_actions() {
                        for (idx, action) in orchard_actions.iter().enumerate() {
                            // Use idx as action_index because it's the only identifier we have for now
                            if let Some(_action_data) = orchard_meta.action_data.get(&(idx as u32))
                            {
                                // Generate a placeholder position based on the action index
                                // In a real implementation, we'd extract this from the tree structure
                                let position = Position((idx as u32) + 1); // Placeholder, starting from 1

                                // Create a new action with the updated position
                                let mut updated_action = action.clone();
                                updated_action.note_commitment_tree_position = position;

                                // Here, we would normally update the action in the vector,
                                // but we can't because we only have an immutable reference through orchard_actions()
                                // For a real implementation, we'd need a mutable access method

                                // For now, we'll just record the position for later use
                                commitment_positions.insert(action.commitment, position);
                            }
                        }
                    }
                }
            }

            // Also process Sapling outputs
            if let Some(sapling_outputs) = zewif_tx.sapling_outputs() {
                for (idx, output) in sapling_outputs.iter().enumerate() {
                    // If we have sapling note data, use that to set positions
                    if let Some(sapling_note_data) = &zcashd_tx.sapling_note_data {
                        for (outpoint, note_data) in sapling_note_data.iter() {
                            // Check if this output matches our index
                            if outpoint.vout == idx as u32 {
                                // Get position information from the witnesses if available
                                if !note_data.witnesses.is_empty() {
                                    // Use the most recent witness, which is typically the first one
                                    // In a real implementation, we'd select the appropriate witness
                                    // based on anchor height or other criteria
                                    let _witness = &note_data.witnesses[0];

                                    // Create a position using the witness information
                                    // For now, just use a placeholder based on the index
                                    let position = Position((idx as u32) + 1); // Placeholder, starting from 1

                                    // Create a new output with the updated position
                                    let mut updated_output = output.clone();
                                    updated_output.note_commitment_tree_position = position;

                                    // Again, we can't update the output directly due to immutable reference
                                    // For a real implementation, we'd need a mutable access method

                                    // Record the position for later use
                                    commitment_positions.insert(output.commitment, position);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // After collecting all positions, we'd need a way to update the transactions with these values
    // In a full implementation, we would use a mutable access method to update the specific actions/outputs
    // For now, this serves as a structural proof of concept

    Ok(())
}

impl From<&ZcashdWallet> for Result<ZewifTop> {
    fn from(wallet: &ZcashdWallet) -> Self {
        migrate_to_zewif(wallet)
    }
}
