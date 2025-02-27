use anyhow::{ Result, bail };

use crate::Data;

pub trait Parse {
    fn parse(parser: &mut Parser) -> Result<Self> where Self: Sized;

    fn parse_buf(buf: &dyn AsRef<[u8]>, trace: bool) -> Result<Self> where Self: Sized {
        let mut parser = Parser::new(&buf);
        parser.set_trace(trace);
        let result = Self::parse(&mut parser)?;
        parser.check_finished()?;
        Ok(result)
    }
}

pub trait ParseWithParam<P> {
    fn parse(parser: &mut Parser, param: P) -> Result<Self> where Self: Sized;

    fn parse_buf(buf: &dyn AsRef<[u8]>, param: P, trace: bool) -> Result<Self> where Self: Sized {
        let mut parser = Parser::new(&buf);
        parser.set_trace(trace);
        let result = Self::parse(&mut parser, param)?;
        parser.check_finished()?;
        Ok(result)
    }
}

pub struct Parser<'a> {
    pub buffer: &'a [u8],
    pub offset: usize,
    pub trace: bool,
}

impl<'a> Parser<'a> {
    pub fn new(buffer: &'a dyn AsRef<[u8]>) -> Self {
        Self {
            buffer: buffer.as_ref(),
            offset: 0,
            trace: false,
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

    pub fn next(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.offset + n > self.buffer.len() {
            bail!("Buffer underflow at offset {}, needed {} bytes, only {} remaining", self.offset, n, self.remaining());
        }
        let bytes = &self.buffer[self.offset..self.offset + n];
        self.offset += n;
        if self.trace {
            println!("\t🟢 next({}): {:?}", n, hex::encode(bytes));
        }
        Ok(bytes)
    }

    pub fn peek(&self, n: usize) -> Result<&'a [u8]> {
        if self.offset + n > self.buffer.len() {
            bail!("Buffer underflow at offset {}, needed {} bytes, only {} remaining", self.offset, n, self.remaining());
        }
        Ok(&self.buffer[self.offset..self.offset + n])
    }

    pub fn rest(&mut self) -> Data {
        Data::parse_len(self, self.remaining()).unwrap()
    }

    pub fn peek_rest(&self) -> Data {
        Data::from_slice(&self.buffer[self.offset..])
    }

    pub fn set_trace(&mut self, trace: bool) {
        self.trace = trace;
    }

    pub fn trace(&self, msg: &str) {
        if self.trace {
            println!("🔵 {}: {:?}", msg, self.peek_rest());
        }
    }
}
