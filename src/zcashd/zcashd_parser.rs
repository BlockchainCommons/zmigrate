use std::{cell::RefCell, collections::{HashMap, HashSet}};

use anyhow::{ Context, Result, bail };

use crate::{ parse, u256 };

use super::{
    u252, zcashd_dump::DBKey, Address, BlockLocator, ClientVersion, DBValue, Key, KeyMetadata, KeyPoolEntry, Keys, MnemonicHDChain, MnemonicSeed, NetworkInfo, OrchardNoteCommitmentTree, PrivKey, PubKey, SproutKeys, SproutPaymentAddress, SproutSpendingKey, WalletTx, ZcashdDump, ZcashdWallet
};

#[derive(Debug)]
pub struct ZcashdParser<'a> {
    pub dump: &'a ZcashdDump,
    pub unparsed_keys: RefCell<HashSet<DBKey>>,
}

impl<'a> ZcashdParser<'a> {
    pub fn parse_dump(dump: &ZcashdDump) -> Result<(ZcashdWallet, HashSet<DBKey>)> {
        let parser = ZcashdParser::new(dump);
        parser.parse()
    }

    fn new(dump: &'a ZcashdDump) -> Self {
        let unparsed_keys = RefCell::new(dump.records().keys().cloned().collect());
        Self {
            dump,
            unparsed_keys,
        }
    }

    // Keep track of which keys have been parsed
    fn mark_key_parsed(&self, key: &DBKey) {
        self.unparsed_keys.borrow_mut().remove(key);
    }

    fn value_for_keyname(&self, keyname: &str) -> Result<&DBValue> {
        let key = self.dump.key_for_keyname(keyname);
        self.mark_key_parsed(&key);
        self.dump.value_for_keyname(keyname)
    }

    fn parse(&self) -> Result<(ZcashdWallet, HashSet<DBKey>)> {
        //
        // Since version 3
        //

        // ~~acc~~: Removed in 4.5.0
        // ~~acentry~~: Removed in 4.5.0

        // **bestblock**: Empty in 6.0.0
        let bestblock = self.parse_block_locator("bestblock")?;

        // ~~**chdseed**~~: Removed in 5.0.0

        // ckey

        // csapzkey

        // cscript

        // czkey

        // **defaultkey**
        let default_key = self.parse_default_key()?;

        // destdata

        // **hdchain**

        // ~~hdseed~~: Removed in 5.0.0

        // key
        // keymeta
        let keys = self.parse_keys()?;

        // **minversion**
        let min_version = self.parse_client_version("minversion")?;

        // **mkey**

        // name
        let address_names = self.parse_address_names()?;

        // **orderposnext**
        let orderposnext = self.parse_opt_i64("orderposnext")?;

        // pool
        let key_pool = self.parse_key_pool()?;

        // purpose
        let address_purposes = self.parse_address_purposes()?;

        // sapzaddr

        // sapextfvk

        // sapzkey

        // tx
        let transactions = self.parse_transactions()?;

        // **version**
        let client_version = self.parse_client_version("version")?;

        // vkey

        // watchs

        // **witnesscachesize**
        let witnesscachesize = self.parse_i64("witnesscachesize")?;

        // wkey

        // zkey
        // zkeymeta
        let sprout_keys = self.parse_sprout_keys()?;

        //
        // Since version 5
        //

        // **networkinfo**
        let network_info = self.parse_network_info()?;

        // **orchard_note_commitment_tree**
        let orchard_note_commitment_tree = self.parse_orchard_note_commitment_tree()?;

        // unifiedaccount

        // unifiedfvk

        // unifiedaddrmeta

        // **mnemonicphrase**
        let mnemonic_phrase = self.parse_mnemonic_phrase()?;

        // **cmnemonicphrase**

        // **mnemonichdchain**
        let mnemonic_hd_chain = self.parse_mnemonic_hd_chain()?;

        // recipientmapping

        //
        // Since version 6
        //

        // **bestblock_nomerkle**
        let bestblock_nomerkle = self.parse_opt_block_locator("bestblock_nomerkle")?;

        let wallet = ZcashdWallet {
            bestblock_nomerkle,
            bestblock,
            client_version,
            default_key,
            keys,
            sprout_keys,
            min_version,
            mnemonic_hd_chain,
            mnemonic_phrase,
            address_names,
            address_purposes,
            network_info,
            orchard_note_commitment_tree,
            orderposnext,
            witnesscachesize,
            key_pool,
            transactions,
        };

        Ok((wallet, self.unparsed_keys.borrow().clone()))
    }

    fn parse_i64(&self, keyname: &str) -> Result<i64> {
        let value = self.value_for_keyname(keyname)?;
        parse!(buf value, i64, format!("i64 for keyname: {}", keyname))
    }

    fn parse_opt_i64(&self, keyname: &str) -> Result<Option<i64>> {
        if self.dump.has_value_for_keyname(keyname) {
            self.parse_i64(keyname).map(Some)
        } else {
            Ok(None)
        }
    }

    fn parse_client_version(&self, keyname: &str) -> Result<ClientVersion> {
        let value = self.value_for_keyname(keyname)?;
        parse!(buf value, ClientVersion, format!("client version for keyname: {}", keyname))
    }

    fn parse_block_locator(&self, keyname: &str) -> Result<BlockLocator> {
        let value = self.value_for_keyname(keyname)?;
        parse!(buf value, BlockLocator, format!("block locator for keyname: {}", keyname))
    }

    fn parse_opt_block_locator(&self, keyname: &str) -> Result<Option<BlockLocator>> {
        if self.dump.has_value_for_keyname(keyname) {
            self.parse_block_locator(keyname).map(Some)
        } else {
            Ok(None)
        }
    }

    fn parse_keys(&self) -> Result<Keys> {
        let key_records = self.dump.records_for_keyname("key").context("Getting 'key' records")?;
        let keymeta_records = self.dump
            .records_for_keyname("keymeta")
            .context("Getting 'keymeta' records")?;
        if key_records.len() != keymeta_records.len() {
            bail!("Mismatched key and keymeta records");
        }
        let mut keys_map = HashMap::new();
        for (key, value) in key_records {
            let pubkey = parse!(buf &key.data, PubKey, "pubkey")?;
            let privkey = parse!(buf value.as_data(), PrivKey, "privkey")?;
            let metakey = DBKey::new("keymeta", &key.data);
            let metadata_binary = self.dump.value_for_key(&metakey).context("Getting metadata")?;
            let metadata = parse!(buf metadata_binary, KeyMetadata, "metadata")?;
            let keypair = Key::new(pubkey.clone(), privkey.clone(), metadata).context(
                "Creating keypair"
            )?;
            keys_map.insert(pubkey, keypair);

            self.mark_key_parsed(&key);
            self.mark_key_parsed(&metakey);
        }
        Ok(Keys::new(keys_map))
    }

    fn parse_sprout_keys(&self) -> Result<Option<SproutKeys>> {
        if !self.dump.has_keys_for_keyname("zkey") {
            return Ok(None);
        }
        let zkey_records = self.dump.records_for_keyname("zkey").context("Getting 'zkey' records")?;
        let zkeymeta_records = self.dump
            .records_for_keyname("zkeymeta")
            .context("Getting 'zkeymeta' records")?;
        if zkey_records.len() != zkeymeta_records.len() {
            bail!("Mismatched zkey and zkeymeta records");
        }
        let mut zkeys_map = HashMap::new();
        for (key, value) in zkey_records {
            let payment_address = parse!(buf &key.data, SproutPaymentAddress, "payment_address")?;
            let spending_key = parse!(buf value.as_data(), u252, "spending_key")?;
            let metakey = DBKey::new("zkeymeta", &key.data);
            let metadata_binary = self.dump.value_for_key(&metakey).context("Getting metadata")?;
            let metadata = parse!(buf metadata_binary, KeyMetadata, "metadata")?;
            let keypair = SproutSpendingKey::new(spending_key, metadata);
            zkeys_map.insert(payment_address, keypair);

            self.mark_key_parsed(&key);
            self.mark_key_parsed(&metakey);
        }
        Ok(Some(SproutKeys::new(zkeys_map)))
    }

    fn parse_default_key(&self) -> Result<PubKey> {
        let value = self.value_for_keyname("defaultkey")?;
        parse!(buf value, PubKey, "defaultkey")
    }

    fn parse_mnemonic_hd_chain(&self) -> Result<MnemonicHDChain> {
        let value = self.value_for_keyname("mnemonichdchain")?;
        parse!(buf value, MnemonicHDChain, "mnemonichdchain")
    }

    fn parse_mnemonic_phrase(&self) -> Result<MnemonicSeed> {
        let (key, value) = self.dump
            .record_for_keyname("mnemonicphrase")
            .context("Getting 'mnemonicphrase' record")?;
        let fingerprint = parse!(buf &key.data, u256, "seed fingerprint")?;
        let seed = parse!(buf &value, MnemonicSeed, "mnemonic phrase")?
            .set_fingerprint(fingerprint);
        self.mark_key_parsed(&key);
        Ok(seed)
    }

    fn parse_address_names(&self) -> Result<HashMap<Address, String>> {
        let records = self.dump.records_for_keyname("name").context("Getting 'name' records")?;
        let mut address_names = HashMap::new();
        for (key, value) in records {
            let address = parse!(buf &key.data, Address, "address")?;
            let name = parse!(buf value.as_data(), String, "name")?;
            if address_names.contains_key(&address) {
                bail!("Duplicate address found: {}", address);
            }
            address_names.insert(address, name);

            self.mark_key_parsed(&key);
        }
        Ok(address_names)
    }

    fn parse_address_purposes(&self) -> Result<HashMap<Address, String>> {
        let records = self.dump.records_for_keyname("purpose").context("Getting 'purpose' records")?;
        let mut address_purposes = HashMap::new();
        for (key, value) in records {
            let address = parse!(buf &key.data, Address, "address")?;
            let purpose = parse!(buf value.as_data(), String, "purpose")?;
            if address_purposes.contains_key(&address) {
                bail!("Duplicate address found: {}", address);
            }
            address_purposes.insert(address, purpose);

            self.mark_key_parsed(&key);
        }
        Ok(address_purposes)
    }

    fn parse_network_info(&self) -> Result<NetworkInfo> {
        let value = self
            .value_for_keyname("networkinfo")
            .context("Getting 'networkinfo' record")?;
        let network_info = parse!(buf value.as_data(), NetworkInfo, "network info")?;
        Ok(network_info)
    }

    fn parse_orchard_note_commitment_tree(&self) -> Result<OrchardNoteCommitmentTree> {
        let value = self
            .value_for_keyname("orchard_note_commitment_tree")
            .context("Getting 'orchard_note_commitment_tree' record")?;
        let orchard_note_commitment_tree = parse!(buf value.as_data(), OrchardNoteCommitmentTree, "orchard note commitment tree")?;
        Ok(orchard_note_commitment_tree)
    }

    fn parse_key_pool(&self) -> Result<HashMap<i64, KeyPoolEntry>> {
        let records = self.dump.records_for_keyname("pool").context("Getting 'pool' records")?;
        let mut key_pool = HashMap::new();
        for (key, value) in records {
            let index = parse!(buf &key.data, i64, "key pool index")?;
            let entry = parse!(buf value.as_data(), KeyPoolEntry, "key pool entry")?;
            key_pool.insert(index, entry);

            self.mark_key_parsed(&key);
        }
        Ok(key_pool)
    }

    fn parse_transactions(&self) -> Result<HashMap<u256, WalletTx>> {
        let mut transactions = HashMap::new();
        // Some wallet files don't have any transactions
        if self.dump.has_keys_for_keyname("tx") {
            let records = self.dump.records_for_keyname("tx").context("Getting 'tx' records")?;
            let mut sorted_records: Vec<_> = records.into_iter().collect();
            sorted_records.sort_by(|(key1, _), (key2, _)| key1.data.cmp(&key2.data));
            for (key, value) in sorted_records {
                let txid = parse!(buf &key.data, u256, "transaction ID")?;
                // let trace = txid == u256::from_hex("2727577862fd0eae69a2c51cb43fd2866fb6f16253de3db0cacbbc22f49270b6");
                // if trace {
                //     println!("🔵 Transaction ID: {:?}", txid);
                // }
                let trace = false;
                let transaction = parse!(buf value.as_data(), WalletTx, "transaction", trace)?;
                if transactions.contains_key(&txid) {
                    bail!("Duplicate transaction found: {:?}", txid);
                }
                transactions.insert(txid, transaction);

                self.mark_key_parsed(&key);
            }
        }
        Ok(transactions)
    }
}
