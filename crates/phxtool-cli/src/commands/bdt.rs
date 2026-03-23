//! BinaryDataTree (BDT) subcommands.

use std::path::PathBuf;

use clap::Subcommand;
use phxtool::ops::bdt;

#[derive(Subcommand)]
pub enum BdtCommand {
    /// Show information about a BDT file
    Info {
        /// Path to the BDT binary file
        file: PathBuf,
        /// Parse as big-endian (Xbox 360) instead of little-endian (PC/DE)
        #[arg(long)]
        big_endian: bool,
    },
    /// Convert BDT binary to XML
    ToXml {
        /// Input BDT file
        input: PathBuf,
        /// Output XML file (defaults to input with .xml extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Parse as big-endian (Xbox 360)
        #[arg(long)]
        big_endian: bool,
    },
    /// Convert XML to BDT binary
    ToBdt {
        /// Input XML file
        input: PathBuf,
        /// Output BDT file (defaults to input with .bdt extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Write as big-endian (Xbox 360)
        #[arg(long)]
        big_endian: bool,
    },
}

pub fn run(cmd: BdtCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        BdtCommand::Info { file, big_endian } => {
            let endian = if big_endian {
                bdt::Endian::Big
            } else {
                bdt::Endian::Little
            };
            let info = bdt::info(&file, endian)?;
            println!("BDT: {}", file.display());
            println!(
                "  Endian:     {}",
                if big_endian {
                    "Big (Xbox 360)"
                } else {
                    "Little (PC/DE)"
                }
            );
            println!(
                "  Root:       {}",
                info.root_name.as_deref().unwrap_or("<empty>")
            );
            println!("  Nodes:      {}", info.node_count);
            println!("  Attributes: {}", info.attribute_count);
        }

        BdtCommand::ToXml {
            input,
            output,
            big_endian,
        } => {
            let endian = if big_endian {
                bdt::Endian::Big
            } else {
                bdt::Endian::Little
            };
            let out = output.unwrap_or_else(|| input.with_extension("xml"));
            println!("Converting {} -> {}", input.display(), out.display());
            bdt::to_xml(&input, &out, endian)?;
            println!("Done!");
        }

        BdtCommand::ToBdt {
            input,
            output,
            big_endian,
        } => {
            let endian = if big_endian {
                bdt::Endian::Big
            } else {
                bdt::Endian::Little
            };
            let out = output.unwrap_or_else(|| input.with_extension("bdt"));
            println!("Converting {} -> {}", input.display(), out.display());
            bdt::to_bdt(&input, &out, endian)?;
            println!("Done!");
        }
    }

    Ok(())
}
