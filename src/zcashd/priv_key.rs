use anyhow::{ Result, Context, bail };

use crate::{ Blob32, Data, Parseable, Parser };

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PrivKey {
    data: Data,
    hash: Blob32,
}

impl std::fmt::Debug for PrivKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PrivKey({:?})", &self.data)
    }
}

impl PrivKey {
    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn hash(&self) -> &Blob32 {
        &self.hash
    }
}

impl AsRef<Data> for PrivKey {
    fn as_ref(&self) -> &Data {
        &self.data
    }
}

impl AsRef<[u8]> for PrivKey {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl Parseable for PrivKey {
    fn parse_type() -> &'static str {
        "PrivKey"
    }

    fn parse(parser: &mut Parser) -> Result<Self> where Self: Sized {
        let size = parser.parse_compact_size().context("Failed to parse PrivKey size")?;
        if size != 214 && size != 279 {
            bail!("Invalid PrivKey size: {}", size);
        }
        let data = parser.parse_data(size).context("Failed to parse PrivKey")?;
        let hash = parser.parse_u256()?;
        Ok(Self { data, hash })
    }
}
