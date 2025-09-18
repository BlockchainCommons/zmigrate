use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Args;
use zewif_zingo::ZingoParser;

use crate::file_args::{FileArgs, FileArgsLike};

/// Process a zingo wallet file
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
    let mut parser = ZingoParser::new(&file_data);
    let wallet = parser.parse()?;
    let mut dump = format!("{:#?}", wallet);
    let remaining = wallet.remaining();
    dump.push_str("\n---\n");
    if remaining == 0 {
        dump.push_str("✅ Success");
    } else {
        dump.push_str(&format!(
            "🛑 Unparsed bytes: {}\n✅ Success (partial)",
            remaining
        ));
    }
    Ok(dump)
}
