use anyhow::Result;

use crate::{Amount, Parse, Parser, Script, parse};

#[derive(Debug, Clone, PartialEq)]
pub struct TxOut {
    pub value: Amount,
    pub script_pub_key: Script,
}

impl Parse for TxOut {
    fn parse(p: &mut Parser) -> Result<Self> {
        Ok(Self {
            value: parse!(p, "value")?,
            script_pub_key: parse!(p, "script_pub_key")?,
        })
    }
}
