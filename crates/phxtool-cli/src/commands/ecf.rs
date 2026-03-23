//! ECF container subcommands.

use std::path::PathBuf;

use clap::Subcommand;
use phxtool::ops::ecf;
use phxtool::ops::util;

#[derive(Subcommand)]
pub enum EcfCommand {
    /// Show information about an ECF container
    Info {
        /// Path to the ECF file
        file: PathBuf,
    },
    /// Expand (dump chunks) from an ECF container
    Expand {
        /// Path to the ECF file
        file: PathBuf,
        /// Output directory (defaults to file stem)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Don't overwrite existing files
        #[arg(long)]
        no_overwrite: bool,
    },
    /// Build an ECF container from chunk files
    Build {
        /// Input directory containing chunk .bin files
        input: PathBuf,
        /// Output ECF file path
        #[arg(short, long)]
        output: PathBuf,
        /// ECF magic number (hex, default: 0xDABA7737)
        #[arg(long, default_value = "0xDABA7737")]
        magic: String,
    },
}

pub fn run(cmd: EcfCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        EcfCommand::Info { file } => {
            let info = ecf::info(&file)?;
            println!("ECF: {}", file.display());
            println!("  Magic:     0x{:08X}", info.magic);
            println!("  Chunks:    {}", info.chunk_count);
            println!("  File size: {}", util::format_size(info.total_size));
            println!();
            for c in &info.chunks {
                println!(
                    "  [{:3}] ID=0x{:08X}  offset=0x{:08X}  size={}  flags=0x{:02X}",
                    c.index,
                    c.id,
                    c.offset,
                    util::format_size(c.size as u64),
                    c.flags,
                );
            }
        }

        EcfCommand::Expand {
            file,
            output,
            no_overwrite,
        } => {
            let outdir = output.unwrap_or_else(|| {
                let stem = file.file_stem().unwrap_or_default();
                PathBuf::from(stem)
            });

            println!("Expanding {} -> {}", file.display(), outdir.display());
            let count = ecf::expand(&file, &outdir, !no_overwrite)?;
            println!("Extracted {} chunks.", count);
        }

        EcfCommand::Build {
            input,
            output,
            magic,
        } => {
            let magic_val = parse_hex_u32(&magic)?;
            println!("Building {} from {}", output.display(), input.display());
            let count = ecf::build(&input, &output, magic_val)?;
            println!("Done! {} chunks packed.", count);
        }
    }

    Ok(())
}

fn parse_hex_u32(s: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let s = s.strip_prefix("0x").or(s.strip_prefix("0X")).unwrap_or(s);
    Ok(u32::from_str_radix(s, 16)?)
}
