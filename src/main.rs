mod styles;

use clap::{Parser as ClapParser, Subcommand};
use zmigrate::{exec::Exec, zcashd_cmd, zingo_cmd, zwl_cmd};

/// A tool for migrating Zcash wallets
#[derive(Debug, clap::Parser)]
#[command(author, version)]
#[command(propagate_version = true)]
#[command(styles=styles::get_styles())]
#[doc(hidden)]
struct Cli {
    #[command(subcommand)]
    command: MainCommands,
}

#[derive(Debug, Subcommand)]
#[doc(hidden)]
enum MainCommands {
    Zcashd(zcashd_cmd::CommandArgs),
    Zingo(zingo_cmd::CommandArgs),
    Zwl(zwl_cmd::CommandArgs),
}

#[doc(hidden)]
fn main() {
    if let Err(e) = inner_main() {
        eprintln!("---");
        eprintln!("🔴 Error: {}\n", e);
        // Print the error context chain
        for cause in e.chain().skip(1) {
            eprintln!("Caused by: {}", cause);
        }
        std::process::exit(1);
    }
}

#[doc(hidden)]
fn inner_main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let output = match cli.command {
        MainCommands::Zcashd(args) => args.exec(),
        MainCommands::Zingo(args) => args.exec(),
        MainCommands::Zwl(args) => args.exec(),
    };
    let output = output?;
    if !output.is_empty() {
        println!("{}", output);
    }
    Ok(())
}
