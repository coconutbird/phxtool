//! Scaleform GFX ↔ SWF subcommands.

use std::path::PathBuf;

use clap::Subcommand;
use phxtool::ops::scaleform;

#[derive(Subcommand)]
pub enum GfxCommand {
    /// Convert GFX to SWF (header swap)
    ToSwf {
        /// Input GFX file
        input: PathBuf,
        /// Output SWF file (defaults to input with .swf extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Convert SWF to GFX (header swap)
    ToGfx {
        /// Input SWF file
        input: PathBuf,
        /// Output GFX file (defaults to input with .gfx extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Decompress a compressed Scaleform file
    Decompress {
        /// Input compressed GFX/SWF file
        input: PathBuf,
        /// Output decompressed file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Show info about a Scaleform file
    Info {
        /// Path to the GFX/SWF file
        file: PathBuf,
    },
}

pub fn run(cmd: GfxCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        GfxCommand::ToSwf { input, output } => {
            let data = std::fs::read(&input)?;
            let out = output.unwrap_or_else(|| input.with_extension("swf"));
            let result = scaleform::gfx_to_swf(&data)?;
            std::fs::write(&out, result)?;
            println!("Converted {} -> {}", input.display(), out.display());
        }

        GfxCommand::ToGfx { input, output } => {
            let data = std::fs::read(&input)?;
            let out = output.unwrap_or_else(|| input.with_extension("gfx"));
            let result = scaleform::swf_to_gfx(&data)?;
            std::fs::write(&out, result)?;
            println!("Converted {} -> {}", input.display(), out.display());
        }

        GfxCommand::Decompress { input, output } => {
            let data = std::fs::read(&input)?;
            let out = output.unwrap_or_else(|| {
                let stem = input.file_stem().unwrap_or_default().to_string_lossy();
                input.with_file_name(format!("{stem}_decompressed.bin"))
            });
            let result = scaleform::decompress_scaleform(&data)?;
            std::fs::write(&out, &result)?;
            println!(
                "Decompressed {} -> {} ({} bytes)",
                input.display(),
                out.display(),
                result.len()
            );
        }

        GfxCommand::Info { file } => {
            let data = std::fs::read(&file)?;
            println!("File: {}", file.display());
            println!("  Size: {} bytes", data.len());

            if scaleform::is_scaleform(&data) {
                if scaleform::is_swf_header(&data) {
                    println!("  Type: SWF (Adobe Flash)");
                } else {
                    println!("  Type: GFX (Scaleform)");
                }
                // Check if compressed
                if data.len() >= 4 {
                    let sig = u32::from_le_bytes([data[0], data[1], data[2], 0]);
                    let compressed = sig == 0x00_53_57_43 || sig == 0x00_47_46_43;
                    println!("  Compressed: {}", if compressed { "yes" } else { "no" });
                    if compressed && data.len() >= 8 {
                        let decomp_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                        println!("  Decompressed size: {} bytes", decomp_size);
                    }
                }
            } else {
                println!("  Type: not a recognized Scaleform/SWF file");
            }
        }
    }

    Ok(())
}
