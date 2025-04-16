
mod styles;

use bc_envelope::prelude::*;
use clap::Parser as ClapParser;
use zewif::ZewifEnvelope;
use zmigrate::zcashd_cmd::zcashd_to_zewif;
use clap::ValueEnum;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

/// Supported input formats for wallet migration
#[derive(Debug, Clone, ValueEnum)]
pub enum InputFormat {
    /// Input from a `zcashd` wallet
    Zcashd,

    /// Input from a `zingo` wallet
    Zingo,
}

/// Supported output formats for wallet migration
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Output in Zewif binary format (default)
    Zewif,

    /// Output in Zewif UR format
    UR,

    /// Output as Envelope Notation
    Format,

    /// Output as Debug dump
    Dump,
}

/// A tool for migrating Zcash wallets
#[derive(Debug, clap::Parser)]
#[command(author, version)]
#[command(propagate_version = true)]
#[command(styles=styles::get_styles())]
#[doc(hidden)]
pub struct Cli {
    /// Input format: zcashd or zingo
    #[arg(long, value_enum)]
    pub from: InputFormat,

    /// Output format: zewif (default) or dump
    #[arg(long, value_enum, default_value_t = OutputFormat::Zewif)]
    pub to: OutputFormat,

    /// Compress the output
    #[arg(long)]
    pub compress: bool,

    /// Encrypt the output
    #[arg(long)]
    pub encrypt: bool,

    /// Input file path
    pub input_file: String,

    /// Output file path, or `-` for stdout
    pub output_file: String,
}

#[doc(hidden)]
fn main() {
    bc_envelope::register_tags();
    if let Err(e) = inner_main() {
        eprintln!("---");
        eprintln!("🔴 Error: {}\n", e);
        for cause in e.chain().skip(1) {
            eprintln!("Caused by: {}", cause);
        }
        std::process::exit(1);
    }
}

#[doc(hidden)]
fn inner_main() -> Result<()> {
    let cli = Cli::parse();

    let input_path = PathBuf::from(cli.input_file.as_str());

    let from = cli.from.clone();
    let to = cli.to.clone();

    match cli.from {
        InputFormat::Zcashd => {
            let zewif = zcashd_to_zewif(&input_path)?;
            let envelope = Envelope::from(zewif.clone());
            let mut ze = ZewifEnvelope::new(envelope)?;
            if cli.compress {
                ze.compress()?;
            }

            let mut output: Box<dyn Write> = match cli.output_file.as_str() {
                "-" => Box::new(io::stdout()),
                path => Box::new(File::create(path).with_context(|| format!("Failed to create output file: {}", path))?),
            };
            match cli.to {
                OutputFormat::Zewif => {
                    output.write_all(&ze.envelope().to_cbor_data())?;
                }
                OutputFormat::UR => {
                    let envelope_ur = ze.envelope().ur_string();
                    writeln!(output, "{}", envelope_ur)?;
                }
                OutputFormat::Format => {
                    writeln!(output, "{}", ze.envelope().format())?;
                }
                OutputFormat::Dump => {
                    writeln!(output, "{:#?}", zewif)?;
                }
            }
        }
        _ => {
            unimplemented!("Unimplemented conversion from {:?} to {:?}", from, to);
        }
    };

    Ok(())
}
