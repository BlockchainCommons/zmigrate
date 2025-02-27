use std::collections::HashMap;

use anyhow::{bail, Context, Result};

use crate::{parse, Parse, ParseWithParam, Parser};

pub fn parse_compact_size(p: &mut Parser) -> Result<usize> {
    match parse!(p, u8, "compact size")? {
        0xfd => {
            let n = parse!(p, u16, "compact size")?;
            if n < 253 {
                bail!("Compact size with 0xfd prefix must be >= 253, got {}", n);
            }
            Ok(n as usize)
        }
        0xfe => {
            let n = parse!(p, u32, "compact size")?;
            if n < 0x10000 {
                bail!(
                    "Compact size with 0xfe prefix must be >= 0x10000, got {}",
                    n
                );
            }
            Ok(n as usize)
        }
        0xff => {
            let n = parse!(p, u64, "compact size")?;
            if n < 0x100000000 {
                bail!(
                    "Compact size with 0xff prefix must be >= 0x100000000, got {}",
                    n
                );
            }
            Ok(n as usize)
        }
        size => Ok(size as usize),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompactSize(pub usize);

impl std::fmt::Display for CompactSize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Parse for CompactSize {
    fn parse(p: &mut Parser) -> Result<Self> {
        parse_compact_size(p).map(CompactSize)
    }
}

impl std::ops::Deref for CompactSize {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn parse_pair<T: Parse, U: Parse>(p: &mut Parser) -> Result<(T, U)> {
    let first = parse!(p, "first item of pair")?;
    let second = parse!(p, "second item of pair")?;
    Ok((first, second))
}

impl<T: Parse, U: Parse> Parse for (T, U) {
    fn parse(p: &mut Parser) -> Result<Self> {
        parse_pair(p)
    }
}

pub fn parse_fixed_length_vec<T: Parse>(p: &mut Parser, length: usize) -> Result<Vec<T>> {
    let mut items = Vec::with_capacity(length);
    for i in 0..length {
        items.push(parse!(p, format!("array item {} of {}", i, length - 1))?);
    }
    Ok(items)
}

pub fn parse_fixed_length_vec_with_param<T: ParseWithParam<U>, U: Clone>(
    p: &mut Parser,
    length: usize,
    param: U,
) -> Result<Vec<T>> {
    let mut items = Vec::with_capacity(length);
    for i in 0..length {
        items.push(parse!(p, param param.clone(), format!("array item {} of {}", i, length - 1))?);
    }
    Ok(items)
}

pub fn parse_fixed_length_array<T: Parse, const N: usize>(p: &mut Parser) -> Result<[T; N]> {
    let items = parse_fixed_length_vec(p, N)?;
    let array: [T; N] = items
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to convert Vec to fixed length array"))?;
    Ok(array)
}

pub fn parse_fixed_length_array_with_param<T: ParseWithParam<U>, U: Clone, const N: usize>(
    p: &mut Parser,
    param: U,
) -> Result<[T; N]> {
    let items = parse_fixed_length_vec_with_param(p, N, param)?;
    let array: [T; N] = items
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to convert Vec to fixed length array"))?;
    Ok(array)
}

pub fn parse_vec<T: Parse>(p: &mut Parser) -> Result<Vec<T>> {
    let length = *parse!(p, CompactSize, "array length")?;
    parse_fixed_length_vec(p, length)
}

pub fn parse_vec_with_param<T: ParseWithParam<U>, U: Clone>(
    p: &mut Parser,
    param: U,
) -> Result<Vec<T>> {
    let length = *parse!(p, CompactSize, "array length")?;
    parse_fixed_length_vec_with_param(p, length, param)
}

impl<T: Parse, const N: usize> Parse for [T; N] {
    fn parse(p: &mut Parser) -> Result<Self> {
        parse_fixed_length_array(p)
    }
}

impl<T: ParseWithParam<U>, U: Clone, const N: usize> ParseWithParam<U> for [T; N] {
    fn parse(p: &mut Parser, param: U) -> Result<Self> {
        parse_fixed_length_array_with_param(p, param)
    }
}

impl<T: Parse> Parse for Vec<T> {
    fn parse(p: &mut Parser) -> Result<Self> {
        parse_vec(p)
    }
}

impl<T: ParseWithParam<U>, U: Clone> ParseWithParam<U> for Vec<T> {
    fn parse(p: &mut Parser, param: U) -> Result<Self> {
        parse_vec_with_param(p, param)
    }
}

pub fn parse_map<K: Parse, V: Parse>(p: &mut Parser) -> Result<Vec<(K, V)>> {
    let length = *parse!(p, CompactSize, "map length")?;
    let mut items = Vec::with_capacity(length);
    for _ in 0..length {
        items.push(parse_pair::<K, V>(p).context("map item")?);
    }
    Ok(items)
}

pub fn parse_hashmap<K, V: Parse>(p: &mut Parser) -> Result<HashMap<K, V>>
where
    K: Parse + Eq + std::hash::Hash,
{
    Ok(parse_map::<K, V>(p)?.into_iter().collect())
}

impl<K: Parse, V: Parse> Parse for HashMap<K, V>
where
    K: Parse + Eq + std::hash::Hash,
{
    fn parse(p: &mut Parser) -> Result<Self> {
        parse_hashmap(p)
    }
}

/// A container that optionally holds a value, serialized with a presence flag followed by the value if present.                      | 1 byte (discriminant: 0x00 = absent, 0x01 = present) + serialized value `T` if present.
pub fn parse_optional<T: Parse>(p: &mut Parser) -> Result<Option<T>> {
    match parse!(p, u8, "optional discriminant")? {
        0x00 => Ok(None),
        0x01 => Ok(Some(parse!(p, "optional value")?)),
        discriminant => bail!("Invalid optional discriminant: 0x{:02x}", discriminant),
    }
}

impl<T: Parse> Parse for Option<T> {
    fn parse(p: &mut Parser) -> Result<Self> {
        parse_optional(p)
    }
}
