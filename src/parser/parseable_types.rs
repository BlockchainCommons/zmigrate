use std::collections::HashMap;

use anyhow::{Context, Result, bail};

use crate::{parse, parser::prelude::*};
use crate::CompactSize;

impl Parse for String {
    fn parse(p: &mut Parser) -> Result<Self> {
        let length = parse!(p, CompactSize, "string length")?;
        let bytes = parse!(p, bytes = *length, "string")?;
        String::from_utf8(bytes.to_vec()).context("string")
    }
}

pub fn parse_string<T>(p: &mut Parser) -> Result<String>
where
    T: Parse + TryInto<usize>,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    let parsed = parse!(p, T, "string length")?;
    let length = parsed
        .try_into()
        .context("converting string length to usize")?;
    let bytes = parse!(p, bytes = length, "string data")?;
    String::from_utf8(bytes.to_vec()).context("string")
}

impl Parse for bool {
    fn parse(p: &mut Parser) -> Result<Self> {
        let byte = parse!(p, u8, "bool")?;
        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => bail!("Invalid boolean value: {}", byte),
        }
    }
}

impl Parse for u8 {
    fn parse(p: &mut Parser) -> Result<Self> {
        let bytes = p.next(1).context("u8")?;
        Ok(bytes[0])
    }
}

impl Parse for u16 {
    fn parse(p: &mut Parser) -> Result<Self> {
        const SIZE: usize = std::mem::size_of::<u16>();
        let bytes = p.next(SIZE).context("u16")?;
        Ok(u16::from_le_bytes(bytes.try_into().context("u16")?))
    }
}

impl Parse for u32 {
    fn parse(p: &mut Parser) -> Result<Self> {
        const SIZE: usize = std::mem::size_of::<u32>();
        let bytes = p.next(SIZE).context("u32")?;
        Ok(u32::from_le_bytes(bytes.try_into().context("u32")?))
    }
}

impl Parse for u64 {
    fn parse(p: &mut Parser) -> Result<Self> {
        const SIZE: usize = std::mem::size_of::<u64>();
        let bytes = p.next(SIZE).context("u64")?;
        Ok(u64::from_le_bytes(bytes.try_into().context("u64")?))
    }
}

impl Parse for i8 {
    fn parse(p: &mut Parser) -> Result<Self> {
        let bytes = p.next(1).context("i8")?;
        Ok(bytes[0] as i8)
    }
}

impl Parse for i16 {
    fn parse(p: &mut Parser) -> Result<Self> {
        const SIZE: usize = std::mem::size_of::<i16>();
        let bytes = p.next(SIZE).context("i16")?;
        Ok(i16::from_le_bytes(bytes.try_into().context("i16")?))
    }
}

impl Parse for i32 {
    fn parse(p: &mut Parser) -> Result<Self> {
        const SIZE: usize = std::mem::size_of::<i32>();
        let bytes = p.next(SIZE).context("i32")?;
        Ok(i32::from_le_bytes(bytes.try_into().context("i32")?))
    }
}

impl Parse for i64 {
    fn parse(p: &mut Parser) -> Result<Self> {
        const SIZE: usize = std::mem::size_of::<i64>();
        let bytes = p.next(SIZE).context("i64")?;
        Ok(i64::from_le_bytes(bytes.try_into().context("i64")?))
    }
}

impl Parse for () {
    fn parse(_p: &mut Parser) -> Result<Self> {
        Ok(())
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
        items.push(parse!(
            p,
            param = param.clone(),
            format!("array item {} of {}", i, length - 1)
        )?);
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
