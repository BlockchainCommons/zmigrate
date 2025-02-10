use anyhow::Result;
use crate::{ parse, ParseWithParam, Parser };

use super::{ Ed25519Signature, Ed25519VerificationKey, JSDescription };

#[derive(Debug, Clone, PartialEq)]
pub struct JoinSplits {
    pub descriptions: Vec<JSDescription>,
    pub pub_key: Option<Ed25519VerificationKey>,
    pub sig: Option<Ed25519Signature>,
}

impl ParseWithParam<bool> for JoinSplits {
    fn parse(p: &mut Parser, use_groth: bool) -> Result<Self> {
        let descriptions = parse!(p, Vec<JSDescription>, param use_groth, "JoinSplit descriptions")?;
        if !descriptions.is_empty() {
            let pub_key = parse!(p, "JoinSplit public key")?;
            let sig = parse!(p, "JoinSplit signature")?;
            Ok(Self {
                descriptions,
                pub_key: Some(pub_key),
                sig: Some(sig),
            })
        } else {
            Ok(Self {
                descriptions,
                pub_key: None,
                sig: None,
            })
        }
    }
}
