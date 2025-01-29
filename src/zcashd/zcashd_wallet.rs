use super::{ClientVersion, Keys};

pub struct ZcashdWallet {
    client_version: ClientVersion,
    min_version: ClientVersion,
    keys: Keys,
}

impl std::fmt::Debug for ZcashdWallet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ZcashdWallet")
            .field("client_version", &self.client_version)
            .field("min_version", &self.min_version)
            .field("keys", &self.keys)
            .finish()
    }
}

impl ZcashdWallet {
    pub fn new(
        client_version: ClientVersion,
        min_version: ClientVersion,
        keys: Keys
    ) -> Self {
        Self {
            client_version,
            min_version,
            keys,
        }
    }

    pub fn client_version(&self) -> &ClientVersion {
        &self.client_version
    }

    pub fn min_version(&self) -> &ClientVersion {
        &self.min_version
    }

    pub fn keys(&self) -> &Keys {
        &self.keys
    }
}
