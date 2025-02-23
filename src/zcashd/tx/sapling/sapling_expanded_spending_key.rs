use anyhow::Result;

use crate::{parse, u256, Parse, Parser};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SaplingExpandedSpendingKey {
    pub ask: u256,
    pub nsk: u256,
    pub ovk: u256,
}

impl Parse for SaplingExpandedSpendingKey {
    fn parse(p: &mut Parser) -> Result<Self> {
        Ok(SaplingExpandedSpendingKey {
            ask: parse!(p, "ask")?,
            nsk: parse!(p, "nsk")?,
            ovk: parse!(p, "ovk")?,
        })
    }
}
