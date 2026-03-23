//! phxtool CLI — unified tool for Halo Wars game assets.
//!
//! A Rust reimplementation of KornnerStudios' PhxTool, providing commands
//! for ERA archives, XMB/XML conversion, and UGX model operations.

mod commands;

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
    Era(commands::era::EraCommand),

    /// XMB ↔ XML conversion operations
    #[command(subcommand)]
    Xmb(commands::xmb::XmbCommand),

    /// UGX model operations (info, glTF export/import)
    #[command(subcommand)]
    Ugx(commands::ugx::UgxCommand),

    /// Wwise audio operations (info, list, dump/extract)
    #[command(subcommand)]
    Wwise(commands::wwise::WwiseCommand),

    /// ECF container operations (expand, build, info)
    #[command(subcommand)]
    Ecf(commands::ecf::EcfCommand),

    /// BinaryDataTree operations (info, to-xml, to-bdt)
    #[command(subcommand)]
    Bdt(commands::bdt::BdtCommand),

    /// Scaleform GFX ↔ SWF operations (to-swf, to-gfx, decompress, info)
    #[command(subcommand)]
    Gfx(commands::gfx::GfxCommand),
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Era(cmd) => commands::era::run(cmd),
        Commands::Xmb(cmd) => commands::xmb::run(cmd),
        Commands::Ugx(cmd) => commands::ugx::run(cmd),
        Commands::Wwise(cmd) => commands::wwise::run(cmd),
        Commands::Ecf(cmd) => commands::ecf::run(cmd),
        Commands::Bdt(cmd) => commands::bdt::run(cmd),
        Commands::Gfx(cmd) => commands::gfx::run(cmd),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
