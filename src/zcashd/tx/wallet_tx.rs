use anyhow::{ Result, Context };
use crate::{ Data, Parseable };

use super::{ LockTime, SaplingBundle, TxIn, TxOut, TxVersion };

#[derive(Debug, Clone, PartialEq)]
pub struct WalletTx {
    version: TxVersion,
    vin: Vec<TxIn>,
    vout: Vec<TxOut>,
    lock_time: LockTime,
    expiry_height: u32,
    sapling_bundle: SaplingBundle,
    rest: Data,
}

impl WalletTx {
    pub fn version(&self) -> &TxVersion {
        &self.version
    }

    pub fn vin(&self) -> &[TxIn] {
        &self.vin
    }

    pub fn vout(&self) -> &[TxOut] {
        &self.vout
    }

    pub fn lock_time(&self) -> &LockTime {
        &self.lock_time
    }

    pub fn expiry_height(&self) -> u32 {
        self.expiry_height
    }

    pub fn sapling_bundle(&self) -> &SaplingBundle {
        &self.sapling_bundle
    }

    pub fn rest(&self) -> &Data {
        &self.rest
    }
}

impl Parseable for WalletTx {
    fn parse(parser: &mut crate::Parser) -> Result<Self> where Self: Sized {
        let version = TxVersion::parse(parser).context("Parsing transaction version")?;

        let mut vin = Vec::new();
        let mut vout = Vec::new();
        let mut lock_time = LockTime::default();
        let mut expiry_height = 0;
        let mut sapling_bundle = SaplingBundle::default();
        if version.is_zip225() {
            println!("⚠️ Unsupported transaction format: {:?}", version);
        } else {
            vin = Vec::<TxIn>::parse(parser).context("Parsing transaction inputs")?;
            vout = Vec::<TxOut>::parse(parser).context("Parsing transaction outputs")?;
            lock_time = LockTime::parse(parser).context("Parsing transaction lock time")?;
            if version.is_overwinter() || version.is_sapling() || version.is_future() {
                expiry_height = u32::parse(parser).context("Parsing transaction expiry height")?;
            }
            if version.is_overwinter() || version.is_sapling() || version.is_future() {
                sapling_bundle = SaplingBundle::parse(parser).context("Parsing Sapling bundle")?;
            }
        }

        let rest = parser.rest();
        Ok(Self {
            version,
            vin,
            vout,
            lock_time,
            expiry_height,
            sapling_bundle,
            rest,
        })
    }
}
