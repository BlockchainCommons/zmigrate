#![allow(dead_code)]

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use anyhow::{Context, Result};

use super::u256;
use super::{Network, ProtocolAddress};

/// Represents a universal identifier for addresses across different ZCash protocols
/// (transparent, sapling, orchard, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AddressId {
    /// Transparent address (P2PKH or P2SH)
    Transparent(String),
    /// Sapling address
    Sapling(String),
    /// Orchard address
    Orchard(String),
    /// Unified address
    Unified(String),
    /// Internal identifier for address in a unified account
    UnifiedAccountAddress(u256),
}

impl AddressId {
    /// Create a new AddressId from a ProtocolAddress
    pub fn from_protocol_address(address: &ProtocolAddress) -> Self {
        match address {
            ProtocolAddress::Transparent(addr) => Self::Transparent(addr.address().to_string()),
            ProtocolAddress::Shielded(addr) => {
                // Determine if it's a Sapling or Orchard address based on the address format
                // This is a simple heuristic and might need refinement
                let addr_str = addr.address();
                if addr_str.starts_with("zs") {
                    Self::Sapling(addr_str.to_string())
                } else if addr_str.starts_with("zo") {
                    Self::Orchard(addr_str.to_string())
                } else {
                    // Default to Sapling if we can't determine the type
                    Self::Sapling(addr_str.to_string())
                }
            }
        }
    }

    /// Create a new AddressId from a string representation of an address and network info
    pub fn from_address_string(address: &str, _network: Network) -> Result<Self> {
        // Try to determine the type based on the address prefix
        if address.starts_with('t') {
            Ok(Self::Transparent(address.to_string()))
        } else if address.starts_with("zs") {
            Ok(Self::Sapling(address.to_string()))
        } else if address.starts_with("zo") {
            Ok(Self::Orchard(address.to_string()))
        } else if address.starts_with('u') {
            Ok(Self::Unified(address.to_string()))
        } else {
            // If we can't determine the type by prefix, use the network to try to parse it
            // This could be extended with more sophisticated address validation
            Err(anyhow::anyhow!(
                "Unable to determine address type for: {}",
                address
            ))
        }
    }

    /// Create an AddressId from a unified account address identifier (u256)
    pub fn from_unified_account_id(id: u256) -> Self {
        Self::UnifiedAccountAddress(id)
    }

    /// Returns true if this is a unified account address
    pub fn is_unified_account_address(&self) -> bool {
        matches!(self, Self::UnifiedAccountAddress(_))
    }

    /// Get the address string if this is a directly addressable address
    pub fn address_string(&self) -> Option<&str> {
        match self {
            Self::Transparent(addr) => Some(addr),
            Self::Sapling(addr) => Some(addr),
            Self::Orchard(addr) => Some(addr),
            Self::Unified(addr) => Some(addr),
            Self::UnifiedAccountAddress(_) => None,
        }
    }

    /// Get the unified account ID if this is a unified account address
    pub fn unified_account_id(&self) -> Option<&u256> {
        match self {
            Self::UnifiedAccountAddress(id) => Some(id),
            _ => None,
        }
    }

    /// Returns the address protocol type as a string
    pub fn protocol_type(&self) -> &'static str {
        match self {
            Self::Transparent(_) => "transparent",
            Self::Sapling(_) => "sapling",
            Self::Orchard(_) => "orchard",
            Self::Unified(_) => "unified",
            Self::UnifiedAccountAddress(_) => "unified_account",
        }
    }
}

impl Display for AddressId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transparent(addr) => write!(f, "t:{}", addr),
            Self::Sapling(addr) => write!(f, "zs:{}", addr),
            Self::Orchard(addr) => write!(f, "zo:{}", addr),
            Self::Unified(addr) => write!(f, "u:{}", addr),
            Self::UnifiedAccountAddress(id) => write!(f, "ua:{}", id),
        }
    }
}

impl FromStr for AddressId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(addr) = s.strip_prefix("t:") {
            Ok(Self::Transparent(addr.to_string()))
        } else if let Some(addr) = s.strip_prefix("zs:") {
            Ok(Self::Sapling(addr.to_string()))
        } else if let Some(addr) = s.strip_prefix("zo:") {
            Ok(Self::Orchard(addr.to_string()))
        } else if let Some(addr) = s.strip_prefix("u:") {
            Ok(Self::Unified(addr.to_string()))
        } else if let Some(id) = s.strip_prefix("ua:") {
            // Parse the u256 value
            let id_bytes =
                hex::decode(id).context("Invalid hex encoding for unified account ID")?;
            let id_u256 = u256::try_from(&id_bytes)
                .context("Failed to create u256 from unified account ID bytes")?;
            Ok(Self::UnifiedAccountAddress(id_u256))
        } else {
            Err(anyhow::anyhow!("Invalid AddressId format: {}", s))
        }
    }
}

/// Tracks address-to-account mappings during wallet migration
#[derive(Debug, Default)]
pub struct AddressRegistry {
    // Maps from AddressId to account identifier (u256)
    address_to_account: std::collections::HashMap<AddressId, u256>,
}

impl AddressRegistry {
    /// Create a new, empty address registry
    pub fn new() -> Self {
        Self {
            address_to_account: std::collections::HashMap::new(),
        }
    }

    /// Register an address with an account
    pub fn register(&mut self, address_id: AddressId, account_id: u256) {
        self.address_to_account.insert(address_id, account_id);
    }

    /// Find the account ID for a given address
    pub fn find_account(&self, address_id: &AddressId) -> Option<&u256> {
        self.address_to_account.get(address_id)
    }

    /// Find all addresses belonging to a specific account
    pub fn find_addresses_for_account(&self, account_id: &u256) -> Vec<&AddressId> {
        self.address_to_account
            .iter()
            .filter_map(|(addr_id, acct_id)| {
                if acct_id == account_id {
                    Some(addr_id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the number of registered addresses
    pub fn address_count(&self) -> usize {
        self.address_to_account.len()
    }

    /// Returns the number of unique accounts referenced
    pub fn account_count(&self) -> usize {
        self.address_to_account
            .values()
            .collect::<std::collections::HashSet<_>>()
            .len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zewif::{Network, ShieldedAddress, TransparentAddress};

    #[test]
    fn test_address_id_from_protocol_address() {
        // Test transparent address
        let transparent =
            ProtocolAddress::Transparent(TransparentAddress::new("t1abcdef".to_string()));
        let addr_id = AddressId::from_protocol_address(&transparent);
        assert!(matches!(addr_id, AddressId::Transparent(_)));
        assert_eq!(addr_id.protocol_type(), "transparent");

        // Test sapling address
        let shielded = ProtocolAddress::Shielded(ShieldedAddress::new("zs1abcdef".to_string()));
        let addr_id = AddressId::from_protocol_address(&shielded);
        assert!(matches!(addr_id, AddressId::Sapling(_)));
        assert_eq!(addr_id.protocol_type(), "sapling");
    }

    #[test]
    fn test_address_id_from_string() {
        // Test transparent address
        let result = AddressId::from_address_string("t1abcdef", Network::Main);
        assert!(result.is_ok());
        let addr_id = result.unwrap();
        assert!(matches!(addr_id, AddressId::Transparent(_)));

        // Test sapling address
        let result = AddressId::from_address_string("zs1abcdef", Network::Main);
        assert!(result.is_ok());
        let addr_id = result.unwrap();
        assert!(matches!(addr_id, AddressId::Sapling(_)));

        // Test unified address
        let result = AddressId::from_address_string("u1abcdef", Network::Main);
        assert!(result.is_ok());
        let addr_id = result.unwrap();
        assert!(matches!(addr_id, AddressId::Unified(_)));
    }

    #[test]
    fn test_address_id_display_and_fromstr() {
        // Test transparent address
        let addr_id = AddressId::Transparent("t1abcdef".to_string());
        let display_str = addr_id.to_string();
        assert_eq!(display_str, "t:t1abcdef");

        let parsed: AddressId = display_str.parse().unwrap();
        assert_eq!(parsed, addr_id);

        // Test unified account address
        let id = u256::default();
        let addr_id = AddressId::UnifiedAccountAddress(id);
        let display_str = addr_id.to_string();
        assert!(display_str.starts_with("ua:"));

        // We can't easily test the full round-trip for UnifiedAccountAddress
        // due to the hex encoding/decoding complexity
    }

    #[test]
    fn test_address_registry() {
        let mut registry = AddressRegistry::new();

        // Create some test addresses and account IDs
        let addr1 = AddressId::Transparent("t1111".to_string());
        let addr2 = AddressId::Sapling("zs2222".to_string());
        let addr3 = AddressId::Orchard("zo3333".to_string());

        let account1 = u256::default(); // Account ID 1
        // Create a u256 value with just the first byte set to 1
        let mut bytes = [0u8; 32];
        bytes[0] = 1;
        let account2 = u256::try_from(&bytes).unwrap(); // Account ID 2

        // Register addresses to accounts
        registry.register(addr1.clone(), account1);
        registry.register(addr2.clone(), account1);
        registry.register(addr3.clone(), account2);

        // Test finding account for address
        assert_eq!(registry.find_account(&addr1), Some(&account1));
        assert_eq!(registry.find_account(&addr2), Some(&account1));
        assert_eq!(registry.find_account(&addr3), Some(&account2));

        // Test finding addresses for account
        let addrs_acct1 = registry.find_addresses_for_account(&account1);
        assert_eq!(addrs_acct1.len(), 2);
        assert!(addrs_acct1.contains(&&addr1));
        assert!(addrs_acct1.contains(&&addr2));

        let addrs_acct2 = registry.find_addresses_for_account(&account2);
        assert_eq!(addrs_acct2.len(), 1);
        assert!(addrs_acct2.contains(&&addr3));

        // Test counts
        assert_eq!(registry.address_count(), 3);
        assert_eq!(registry.account_count(), 2);
    }
}
