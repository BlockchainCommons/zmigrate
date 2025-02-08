use anyhow::Result;

use crate::{parse, Parse, Parser};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IntID(u32);

impl IntID {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for IntID {
    // Always display as hex with `0x` prefix
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}

impl std::fmt::Debug for IntID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Parse for IntID {
    fn parse(parser: &mut Parser) -> Result<Self> where Self: Sized {
        let id = parse!(parser, "IntID")?;
        Ok(Self(id))
    }
}
