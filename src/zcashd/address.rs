use anyhow::Result;
use crate::{Parse, Parser};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address(String);

impl Address {
    pub fn new(address: impl Into<String>) -> Self {
        Self(address.into())
    }
}

impl Parse for Address {
    fn parse(parser: &mut Parser) -> Result<Self> where Self: Sized {
        let address = String::parse(parser)?;
        Ok(Self(address))
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
