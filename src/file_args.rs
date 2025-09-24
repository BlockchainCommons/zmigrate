use std::{io::Read, path::PathBuf};

use anyhow::Result;
use clap::Args;
use zewif::Data;

pub trait FileArgsLike {
    fn file(&self) -> &PathBuf;

    fn read_file(&self) -> Result<Data> {
        let mut vec = Vec::new();
        std::fs::File::open(self.file())?.read_to_end(&mut vec)?;
        Ok(vec.into())
    }
}

#[derive(Debug, Args)]
#[group(skip)]
pub struct FileArgs {
    pub file: PathBuf,
}
