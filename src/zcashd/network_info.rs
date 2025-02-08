use anyhow::Result;
use crate::{parse, Parse, Parser};

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkInfo {
    zcash: String,
    identifier: String,
}

impl NetworkInfo {
    pub fn zcash(&self) -> &str {
        &self.zcash
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl Parse for NetworkInfo {
    fn parse(parser: &mut Parser) -> Result<Self> where Self: Sized {
        let (zcash, identifier) = parse!(parser, "(zcash, identifier)")?;
        Ok(Self { zcash, identifier })
    }
}
