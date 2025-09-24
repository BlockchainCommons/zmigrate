mod styles;

use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use bc_envelope::prelude::*;
use clap::{Parser as ClapParser, ValueEnum};
use rpassword::prompt_password;
use zewif::ZewifEnvelope;
use zmigrate::zcashd_cmd;
#[cfg(feature = "zingo")]
use zmigrate::zingo_cmd;

/// Supported input formats for wallet migration
#[derive(Debug, Clone, ValueEnum)]
pub enum InputFormat {
    /// Input from a `zcashd` wallet
    Zcashd,

    /// Input from a `zingo` wallet
    #[cfg(feature = "zingo")]
    Zingo,

    /// Input from a `zewif` wallet
    Zewif,
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

    match cli.from {
        InputFormat::Zcashd => {
            let zewif = zcashd_cmd::zcashd_to_zewif(&input_path)?;
            let envelope = Envelope::from(zewif.clone());
            let mut ze = ZewifEnvelope::new(envelope)?;
            if cli.compress {
                ze.compress()?;
            }
            if cli.encrypt {
                let password = prompt_password("Enter encryption password: ")?;
                let key = ZewifEnvelope::derive_encryption_key(password);
                ze.encrypt(&key)?;
            }
            let mut output: Box<dyn Write> = match cli.output_file.as_str() {
                "-" => Box::new(io::stdout()),
                path => Box::new(File::create(path).with_context(|| {
                    format!("Failed to create output file: {}", path)
                })?),
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
        InputFormat::Zewif => {
            // Read the input file as CBOR and parse as Envelope
            let input_data = std::fs::read(&input_path).with_context(|| {
                format!("Failed to read input file: {}", cli.input_file)
            })?;
            let envelope = Envelope::try_from_cbor_data(input_data)
                .with_context(|| "Failed to parse input as Envelope")?;
            let mut ze = ZewifEnvelope::new(envelope)?;
            // If encrypted, prompt for password and decrypt
            if ze.is_encrypted() {
                let password = prompt_password("Enter decryption password: ")?;
                let key = ZewifEnvelope::derive_encryption_key(password);
                ze.decrypt(&key)?;
            }
            // If compressed, uncompress
            if ze.is_compressed() {
                ze.uncompress()?;
            }
            let mut output: Box<dyn Write> = match cli.output_file.as_str() {
                "-" => Box::new(io::stdout()),
                path => Box::new(File::create(path).with_context(|| {
                    format!("Failed to create output file: {}", path)
                })?),
            };
            match cli.to {
                OutputFormat::Format => {
                    writeln!(output, "{}", ze.envelope().format())?;
                }
                OutputFormat::Zewif => {
                    output.write_all(&ze.envelope().to_cbor_data())?;
                }
                OutputFormat::UR => {
                    let envelope_ur = ze.envelope().ur_string();
                    writeln!(output, "{}", envelope_ur)?;
                }
                OutputFormat::Dump => {
                    // Try to reconstruct Zewif for debug output
                    match zewif::Zewif::try_from(ze.envelope().clone()) {
                        Ok(zewif) => writeln!(output, "{:#?}", zewif)?,
                        Err(_) => writeln!(
                            output,
                            "Could not decode Zewif from envelope."
                        )?,
                    }
                }
            }
        }
        #[cfg(feature = "zingo")]
        InputFormat::Zingo => {
            if cli.compress {
                anyhow::bail!(
                    "Compression is not yet supported for zingo ingestion"
                );
            }
            if cli.encrypt {
                anyhow::bail!(
                    "Encryption is not yet supported for zingo ingestion"
                );
            }
            match cli.to {
                OutputFormat::Dump => {
                    let dump = zingo_cmd::dump_wallet(&input_path)?;
                    let mut output: Box<dyn Write> =
                        match cli.output_file.as_str() {
                            "-" => Box::new(io::stdout()),
                            path => Box::new(File::create(path).with_context(
                                || {
                                    format!(
                                        "Failed to create output file: {}",
                                        path
                                    )
                                },
                            )?),
                        };
                    writeln!(output, "{}", dump)?;
                }
                other => {
                    anyhow::bail!(
                        "Zingo ingestion supports only --to dump for now (requested {:?})",
                        other
                    );
                }
            }
        }
    };

    Ok(())
}
