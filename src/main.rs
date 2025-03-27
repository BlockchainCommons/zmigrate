mod styles;

use clap::{Parser as ClapParser, Subcommand};
use zmigrate::exec::Exec;
#[cfg(feature = "zcashd")]
use zmigrate::zcashd_cmd;
#[cfg(feature = "zingo")]
use zmigrate::zingo_cmd;

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
    #[cfg(feature = "zcashd")]
    Zcashd(zcashd_cmd::CommandArgs),
    #[cfg(feature = "zingo")]
    Zingo(zingo_cmd::CommandArgs),
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
        #[cfg(feature = "zcashd")]
        MainCommands::Zcashd(args) => args.exec(),
        #[cfg(feature = "zingo")]
        MainCommands::Zingo(args) => args.exec(),
    };
    let output = output?;
    if !output.is_empty() {
        println!("{}", output);
    }
    Ok(())
}
