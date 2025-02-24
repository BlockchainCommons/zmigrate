use anyhow::Result;

use crate::{parse, Blob, Parse, Parser};

use super::{u256, ReceiverType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnifiedAddressMetadata {
    pub key_id: u256,
    pub diversifier_index: Blob<11>,
    pub receiver_types: Vec<ReceiverType>,
}

impl Parse for UnifiedAddressMetadata {
    fn parse(p: &mut Parser) -> Result<Self> {
        let key_id = parse!(p, "key_id")?;
        let diversifier_index = parse!(p, "diversifier_index")?;
        let receiver_types = parse!(p, "receiver_types")?;
        Ok(Self { key_id, diversifier_index, receiver_types })
    }
}
