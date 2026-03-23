//! phxtool CLI — unified tool for Halo Wars game assets.
//!
//! A Rust reimplementation of KornnerStudios' PhxTool, providing commands
//! for ERA archives, XMB/XML conversion, and UGX model operations.

mod cmd_era;
mod cmd_ugx;
mod cmd_wwise;
mod cmd_xmb;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "phxtool",
    author,
    version,
    about = "Halo Wars game asset tool",
    long_about = "A Rust reimplementation of KornnerStudios' PhxTool.\n\
                   Provides ERA archive management, XMB/XML conversion, \
                   and UGX model operations for Halo Wars."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// ERA archive operations (expand, build, list, info, decrypt, encrypt)
    #[command(subcommand)]
    Era(cmd_era::EraCommand),

    /// XMB ↔ XML conversion operations
    #[command(subcommand)]
    Xmb(cmd_xmb::XmbCommand),

    /// UGX model operations (info, glTF export/import)
    #[command(subcommand)]
    Ugx(cmd_ugx::UgxCommand),

    /// Wwise audio operations (info, list, dump/extract)
    #[command(subcommand)]
    Wwise(cmd_wwise::WwiseCommand),
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Era(cmd) => cmd_era::run(cmd),
        Commands::Xmb(cmd) => cmd_xmb::run(cmd),
        Commands::Ugx(cmd) => cmd_ugx::run(cmd),
        Commands::Wwise(cmd) => cmd_wwise::run(cmd),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
