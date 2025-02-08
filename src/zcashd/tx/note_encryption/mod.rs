use anyhow::{ Result, Context };

use crate::{ Blob, Parseable, Parser };

pub const NOTEPLAINTEXT_LEADING: usize = 1;
pub const SAPLING_DIVERSIFIER_SIZE: usize = 11;
pub const V_SIZE: usize = 8;
pub const R_SIZE: usize = 32;
pub const RHO_SIZE: usize = 32;
pub const MEMO_SIZE: usize = 512;
pub const NOTEPLAINTEXT_SIZE: usize = NOTEPLAINTEXT_LEADING + V_SIZE + RHO_SIZE + R_SIZE + MEMO_SIZE;
pub const MLEN: usize = NOTEPLAINTEXT_SIZE;

pub const NOTEENCRYPTION_AUTH_BYTES: usize = 16;

pub const CLEN: usize = MLEN + NOTEENCRYPTION_AUTH_BYTES;
#[derive(Clone, PartialEq)]
pub struct Ciphertext(Blob<CLEN>);

impl AsRef<Blob<CLEN>> for Ciphertext {
    fn as_ref(&self) -> &Blob<CLEN> {
        &self.0
    }
}

impl AsRef<[u8]> for Ciphertext {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Parseable for Ciphertext {
    fn parse(parser: &mut Parser) -> Result<Self> where Self: Sized {
        let blob = Blob::parse(parser).context("Parsing Ciphertext")?;
        Ok(Self(blob))
    }
}

impl std::fmt::Debug for Ciphertext {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Ciphertext({})", hex::encode(self))
    }
}
