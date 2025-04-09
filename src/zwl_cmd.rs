use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Args;
use zewif_zwl::ZwlParser;

use crate::file_args::{FileArgs, FileArgsLike};

/// Process a zecwallet wallet file
#[derive(Debug, Args)]
#[group(skip)]
pub struct CommandArgs {
    #[command(flatten)]
    file_args: FileArgs,
}

impl FileArgsLike for CommandArgs {
    fn file(&self) -> &PathBuf {
        &self.file_args.file
    }
}

impl crate::exec::Exec for CommandArgs {
    fn exec(&self) -> Result<String> {
        let file = self.file();
        dump_wallet(file)
    }
}

pub fn dump_wallet(file: &Path) -> Result<String> {
    let file_data = std::fs::read(file)?.into();
    let mut parser = ZwlParser::new(&file_data);
    let wallet = parser.parse()?;
    let mut dump = format!("{:#?}", wallet);
    let remaining = wallet.remaining();
    if remaining == 0 {
        dump.push_str("\n---\n✅ Success");
    } else {
        dump.push_str(&format!("\n---\n🛑 Unparsed bytes: {}", remaining))
    }
    Ok(dump)
}
