use anyhow::Result;

use crate::{Blob, Blob32};

use crate::{parse, Parse, Parser};

#[derive(Debug, Clone, PartialEq)]
pub struct TransmittedNoteCiphertext {
    epk_bytes: Blob32,
    enc_ciphertext: Blob<580>,
    out_ciphertext: Blob<80>,
}

impl TransmittedNoteCiphertext {
    pub fn enc_ciphertext(&self) -> &Blob<580> {
        &self.enc_ciphertext
    }
}

impl Parse for TransmittedNoteCiphertext {
    fn parse(parser: &mut Parser) -> Result<Self> where Self: Sized {
        Ok(Self {
            epk_bytes: parse!(parser, "epk_bytes")?,
            enc_ciphertext: parse!(parser, "enc_ciphertext")?,
            out_ciphertext: parse!(parser, "out_ciphertext")?,
        })
    }
}
