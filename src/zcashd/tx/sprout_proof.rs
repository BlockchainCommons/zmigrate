use anyhow::{ Result, Context };

use crate::{ Parseable, Parser };

use super::{GrothProof, PHGRProof};

#[derive(Debug, Clone, PartialEq)]
pub enum SproutProof {
    PHGRProof(PHGRProof),
    GrothProof(GrothProof),
}

impl SproutProof {
    pub fn parse(parser: &mut Parser, use_groth: bool) -> Result<Self> where Self: Sized {
        if use_groth {
            let groth_proof = GrothProof::parse(parser).context("Parsing groth proof")?;
            Ok(Self::GrothProof(groth_proof))
        } else {
            let phgr_proof = PHGRProof::parse(parser).context("Parsing phgr proof")?;
            Ok(Self::PHGRProof(phgr_proof))
        }
    }
}
