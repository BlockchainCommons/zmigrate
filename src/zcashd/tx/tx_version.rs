use anyhow::{Result, Context, bail};

use crate::{parse, Parse, Parser};

use super::IntID;

const OVERWINTER_VERSION_GROUP_ID: IntID = IntID::new(0x03c48270);
const OVERWINTER_TX_VERSION: u32 = 3;
const SAPLING_VERSION_GROUP_ID: IntID = IntID::new(0x892f2085);
const SAPLING_TX_VERSION: u32 = 4;
const ZIP225_VERSION_GROUP_ID: IntID = IntID::new(0x26a7270a);
const ZIP225_TX_VERSION: u32 = 5;
const ZFUTURE_VERSION_GROUP_ID: IntID = IntID::new(0xffffffff);
const ZFUTURE_TX_VERSION: u32 = 0x0000ffff;

#[derive(Debug, Clone, PartialEq)]
pub enum TxVersionGroup {
    PreOverwinter,
    OverwinterV3,
    SaplingV4,
    Zip225V5,
    Future,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TxVersion {
    pub group: TxVersionGroup,
    pub number: u32,
}

impl TxVersion {
    pub fn group(&self) -> &TxVersionGroup {
        &self.group
    }

    pub fn number(&self) -> u32 {
        self.number
    }

    pub fn is_overwinter(&self) -> bool {
        self.group != TxVersionGroup::PreOverwinter
    }

    pub fn is_sapling(&self) -> bool {
        self.group == TxVersionGroup::SaplingV4
    }

    pub fn is_zip225(&self) -> bool {
        self.group == TxVersionGroup::Zip225V5
    }

    pub fn is_future(&self) -> bool {
        self.group == TxVersionGroup::Future
    }
}

impl Parse for TxVersion {
    fn parse(parser: &mut Parser) -> Result<Self> where Self: Sized {
        let header = u32::parse(parser).context("Transaction header")?;
        let overwintered = (header >> 31) == 1;
        let number = header & 0x7fffffff;

        let version_group_id = if overwintered {
            parse!(parser, "Transaction version group ID")?
        } else {
            IntID::default()
        };

        let group = match (overwintered, version_group_id, number) {
            (false, _, _) => TxVersionGroup::PreOverwinter,
            (true, OVERWINTER_VERSION_GROUP_ID, OVERWINTER_TX_VERSION) => TxVersionGroup::OverwinterV3,
            (true, SAPLING_VERSION_GROUP_ID, SAPLING_TX_VERSION) => TxVersionGroup::SaplingV4,
            (true, ZIP225_VERSION_GROUP_ID, ZIP225_TX_VERSION) => TxVersionGroup::Zip225V5,
            (true, ZFUTURE_VERSION_GROUP_ID, ZFUTURE_TX_VERSION) => TxVersionGroup::Future,
            _ => bail!("Unsupported transaction format: overwintered={}, version={}, version_group_id={}", overwintered, number, version_group_id),
        };

        Ok(Self {
            group,
            number,
        })
    }
}
