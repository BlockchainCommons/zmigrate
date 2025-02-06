use std::convert::TryInto;

use anyhow::{ Context, Result, bail };

use crate::{Blob, Blob20, Blob32, Data};

pub trait Parseable {
    fn parse_type() -> &'static str;

    fn parse(parser: &mut Parser) -> Result<Self> where Self: Sized;

    fn parse_binary(buffer: &dyn AsRef<[u8]>) -> Result<Self> where Self: Sized {
        let mut parser = Parser::new(&buffer);
        let result = Self::parse(&mut parser)?;
        parser.check_finished()?;
        Ok(result)
    }
}

pub struct Parser<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> Parser<'a> {
    pub fn new(buffer: &'a dyn AsRef<[u8]>) -> Self {
        Self {
            buffer: buffer.as_ref(),
            offset: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn remaining(&self) -> usize {
        self.len() - self.offset
    }

    pub fn check_finished(&self) -> Result<()> {
        if self.offset < self.buffer.len() {
            bail!("Buffer has {} bytes left", self.remaining());
        }
        Ok(())
    }

    pub fn parse_slice(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.offset + n > self.buffer.len() {
            bail!("Buffer underflow at offset {}, needed {} bytes, only {} remaining", self.offset, n, self.remaining());
        }
        let bytes = &self.buffer[self.offset..self.offset + n];
        self.offset += n;
        Ok(bytes)
    }

    pub fn parse_blob<const T: usize>(&mut self) -> Result<Blob<T>> {
        let bytes = self.parse_slice(T).context("Parsing blob")?;
        Blob::from_slice(bytes)
    }

    pub fn parse_data(&mut self, len: usize) -> Result<Data> {
        let bytes = self.parse_slice(len).context("Parsing data")?;
        Ok(Data::from_slice(bytes))
    }

    pub fn rest(&mut self) -> Data {
        let bytes = self.parse_slice(self.remaining()).unwrap();
        Data::from_slice(bytes)
    }

    pub fn parse_u8(&mut self) -> Result<u8> {
        let bytes = self.parse_slice(1).context("Parsing u8")?;
        Ok(bytes[0])
    }

    pub fn parse_bool(&mut self) -> Result<bool> {
        let byte = self.parse_u8().context("Parsing bool")?;
        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => bail!("Invalid boolean value: {}", byte),
        }
    }

    pub fn parse_u16(&mut self) -> Result<u16> {
        const SIZE: usize = std::mem::size_of::<u16>();
        let bytes = self.parse_slice(SIZE).context("Parsing u16")?;
        Ok(u16::from_le_bytes(bytes.try_into().context("Parsing u16")?))
    }

    pub fn parse_i32(&mut self) -> Result<i32> {
        const SIZE: usize = std::mem::size_of::<i32>();
        let bytes = self.parse_slice(SIZE).context("Parsing i32")?;
        Ok(i32::from_le_bytes(bytes.try_into().context("Parsing i32")?))
    }

    pub fn parse_u32(&mut self) -> Result<u32> {
        const SIZE: usize = std::mem::size_of::<u32>();
        let bytes = self.parse_slice(SIZE).context("Parsing u32")?;
        Ok(u32::from_le_bytes(bytes.try_into().context("Parsing u32")?))
    }

    pub fn parse_i64(&mut self) -> Result<i64> {
        const SIZE: usize = std::mem::size_of::<i64>();
        let bytes = self.parse_slice(SIZE).context("Parsing i64")?;
        Ok(i64::from_le_bytes(bytes.try_into().context("Parsing i64")?))
    }

    pub fn parse_u64(&mut self) -> Result<u64> {
        const SIZE: usize = std::mem::size_of::<u64>();
        let bytes = self.parse_slice(SIZE).context("Parsing u64")?;
        Ok(u64::from_le_bytes(bytes.try_into().context("Parsing u64")?))
    }

    pub fn parse_u160(&mut self) -> Result<Blob20> {
        const SIZE: usize = 20;
        let bytes = self.parse_slice(SIZE).context("Parsing u160")?;
        Blob20::from_slice(bytes)
    }

    pub fn parse_u256(&mut self) -> Result<Blob32> {
        const SIZE: usize = 32;
        let bytes = self.parse_slice(SIZE).context("Parsing u256")?;
        Blob32::from_slice(bytes)
    }

    /// Wrapper of uint256 with guarantee that first four bits are zero.
    pub fn parse_u252(&mut self) -> Result<Blob32> {
        const SIZE: usize = 32;
        let bytes = self.parse_slice(SIZE).context("Parsing u252")?;
        if (bytes[0] & 0xf0) != 0 {
            bail!("First four bits of u252 must be zero");
        }
        Blob32::from_slice(bytes)
    }

    /// 1 byte (length) + bytes of the string
    pub fn parse_utf8(&mut self) -> Result<String> {
        let length = self.parse_u8()? as usize;
        let bytes = self.parse_slice(length).context("Parsing utf8")?;
        String::from_utf8(bytes.to_vec()).context("Parsing utf8")
    }

    pub fn parse_compact_size(&mut self) -> Result<usize> {
        match self.parse_u8().context("Parsing compact size")? {
            0xfd =>
                self
                    .parse_u16()
                    .map(|n| n as usize)
                    .context("Parsing compact size"),
            0xfe =>
                self
                    .parse_u32()
                    .map(|n| n as usize)
                    .context("Parsing compact size"),
            0xff =>
                self
                    .parse_u64()
                    .map(|n| n as usize)
                    .context("Parsing compact size"),
            size => Ok(size as usize),
        }
    }

    pub fn parse_item<T: Parseable>(&mut self) -> Result<T> {
        T::parse(self).with_context(|| format!("Parsing item of type '{}'", T::parse_type()))
    }

    pub fn parse_pair<T: Parseable, U: Parseable>(&mut self) -> Result<(T, U)> {
        let first = self.parse_item::<T>().context("Parsing first item of pair")?;
        let second = self.parse_item::<U>().context("Parsing second item of pair")?;
        Ok((first, second))
    }

    pub fn parse_fixed_length_array<T: Parseable>(&mut self, length: usize) -> Result<Vec<T>> {
        let mut items = Vec::with_capacity(length);
        for i in 0..length {
            items.push(self.parse_item::<T>().with_context(|| format!("Parsing array item {} of {}", i, length - 1))?);
        }
        Ok(items)
    }

    pub fn parse_array<T: Parseable>(&mut self) -> Result<Vec<T>> {
        let length = self.parse_compact_size().context("Parsing array length")?;
        self.parse_fixed_length_array(length)
    }

    pub fn parse_map<K: Parseable, V: Parseable>(&mut self) -> Result<Vec<(K, V)>> {
        let length = self.parse_compact_size().context("Parsing map length")?;
        let mut items = Vec::with_capacity(length);
        for _ in 0..length {
            items.push(self.parse_pair::<K, V>().context("Parsing map item")?);
        }
        Ok(items)
    }

    pub fn parse_hashmap<K, V: Parseable>(&mut self) -> Result<Vec<(K, V)>>
        where K: Parseable + Eq + std::hash::Hash
    {
        let map = self.parse_map::<K, V>()?;
        let mut hashmap = std::collections::HashMap::new();
        for (key, value) in map {
            hashmap.insert(key, value);
        }
        Ok(hashmap.into_iter().collect())
    }

    /// A container that optionally holds a value, serialized with a presence flag followed by the value if present.                      | 1 byte (discriminant: 0x00 = absent, 0x01 = present) + serialized value `T` if present.
    pub fn parse_optional<T: Parseable>(&mut self) -> Result<Option<T>> {
        match self.parse_u8().context("Parsing optional discriminant")? {
            0x00 => Ok(None),
            0x01 => Ok(Some(self.parse_item::<T>().context("Parsing optional value")?)),
            discriminant => bail!("Invalid optional discriminant: {}", discriminant),
        }
    }
}
