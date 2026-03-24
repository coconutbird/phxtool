//! ERA archive subcommands.

use std::path::PathBuf;

use clap::Subcommand;
use phxtool::ops::era;
use phxtool::ops::util;

#[derive(Subcommand)]
pub enum EraCommand {
    /// Expand (extract) an ERA archive
    Expand {
        /// Path to the ERA archive
        file: PathBuf,
        /// Output directory (defaults to archive name without extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Glob pattern to filter files (e.g., "*.ugx", "data/**/*.xmb")
        #[arg(short, long)]
        filter: Option<String>,
        /// Don't convert XMB files to XML during extraction
        #[arg(long)]
        no_translate: bool,
        /// Don't overwrite existing files
        #[arg(long)]
        no_overwrite: bool,
        /// Only dump a listing file
        #[arg(long)]
        listing_only: bool,
        /// Decompress Scaleform UI files (.gfx/.swf) to .bin
        #[arg(long)]
        decompress_ui: bool,
        /// Convert GFX files to SWF (header swap)
        #[arg(long)]
        gfx_to_swf: bool,
        /// Skip hash/signature verification
        #[arg(long)]
        skip_verify: bool,
    },
    /// Build an ERA archive from a directory
    Build {
        /// Input directory containing files to archive
        input: PathBuf,
        /// Output ERA file path
        #[arg(short, long)]
        output: PathBuf,
        /// Don't convert XML files to XMB before archiving
        #[arg(long)]
        no_translate: bool,
        /// Don't encrypt the output archive
        #[arg(long)]
        no_encrypt: bool,
    },
    /// List files in an ERA archive
    List {
        /// Path to the ERA archive
        file: PathBuf,
    },
    /// Show archive information
    Info {
        /// Path to the ERA archive
        file: PathBuf,
    },
    /// Decrypt an encrypted ERA file
    Decrypt {
        /// Input encrypted ERA file
        input: PathBuf,
        /// Output decrypted file
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Encrypt a plaintext ERA/ECF file
    Encrypt {
        /// Input plaintext ERA/ECF file
        input: PathBuf,
        /// Output encrypted file
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Verify archive integrity (Tiger128 hashes + signature)
    Verify {
        /// Path to the ERA archive
        file: PathBuf,
    },
}

pub fn run(cmd: EraCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        EraCommand::Expand {
            file,
            output,
            filter,
            no_translate,
            no_overwrite,
            listing_only,
            decompress_ui,
            gfx_to_swf,
            skip_verify,
        } => {
            let outdir = output.unwrap_or_else(|| {
                let stem = file.file_stem().unwrap_or_default();
                PathBuf::from(stem)
            });

            let opts = era::ExpandOptions {
                translate_xmb: !no_translate,
                overwrite: !no_overwrite,
                listing_only,
                filter,
                decompress_ui,
                gfx_to_swf,
                skip_verify,
            };

            println!("Expanding {} -> {}", file.display(), outdir.display());
            let result = era::expand(&file, &outdir, &opts)?;

            println!(
                "Extracted {} files ({} XMB→XML translations)",
                result.files_extracted, result.files_translated
            );
            if !skip_verify {
                println!("Verified {} file hashes", result.files_verified);
            }
            if !result.hash_failures.is_empty() {
                eprintln!("{} hash failures:", result.hash_failures.len());
                for msg in &result.hash_failures {
                    eprintln!("  {}", msg);
                }
            }
            if !result.errors.is_empty() {
                eprintln!("{} errors:", result.errors.len());
                for (name, err) in &result.errors {
                    eprintln!("  {}: {}", name, err);
                }
            }
        }

        EraCommand::Build {
            input,
            output,
            no_translate,
            no_encrypt,
        } => {
            let opts = era::BuildOptions {
                translate_xml: !no_translate,
                encrypt: !no_encrypt,
            };

            println!("Building {} from {}", output.display(), input.display());
            let count = era::build(&input, &output, &opts)?;
            println!("Done! {} files archived.", count);
        }

        EraCommand::List { file } => {
            let entries = era::list(&file)?;
            println!("Files in {}:\n", file.display());
            let max_name = entries
                .iter()
                .map(|e| e.filename.as_deref().unwrap_or("<unnamed>").len())
                .max()
                .unwrap_or(8)
                .max(8);

            println!(
                "{:>5}  {:>10}  {:>12}  {:<width$}",
                "Index",
                "Compressed",
                "Decompressed",
                "Filename",
                width = max_name
            );
            println!(
                "{:->5}  {:->10}  {:->12}  {:-<width$}",
                "",
                "",
                "",
                "",
                width = max_name
            );

            for e in &entries {
                let name = e.filename.as_deref().unwrap_or("<unnamed>");
                println!(
                    "{:>5}  {:>10}  {:>12}  {:<width$}",
                    e.index,
                    util::format_size(e.compressed_size as u64),
                    util::format_size(e.decompressed_size as u64),
                    name,
                    width = max_name
                );
            }
            println!("\nTotal: {} entries", entries.len());
        }

        EraCommand::Info { file } => {
            let info = era::info(&file)?;
            println!("Archive: {}", file.display());
            println!("  ECF Magic:    0x{:08X}", info.ecf_magic);
            println!("  ERA Magic:    0x{:08X}", info.archive_magic);
            println!("  Files:        {}", info.file_count);
            println!(
                "  Compressed:   {}",
                util::format_size(info.total_compressed)
            );
            println!(
                "  Decompressed: {}",
                util::format_size(info.total_decompressed)
            );
            if info.total_decompressed > 0 {
                let ratio = (info.total_compressed as f64 / info.total_decompressed as f64) * 100.0;
                println!("  Ratio:        {:.1}%", ratio);
            }
            if info.has_signature {
                println!("  Signature:    {} bytes", info.signature_size);
            } else {
                println!("  Signature:    none");
            }
        }

        EraCommand::Decrypt { input, output } => {
            println!("Decrypting {} -> {}", input.display(), output.display());
            era::decrypt(&input, &output)?;
            println!("Done!");
        }

        EraCommand::Encrypt { input, output } => {
            println!("Encrypting {} -> {}", input.display(), output.display());
            era::encrypt(&input, &output)?;
            println!("Done!");
        }

        EraCommand::Verify { file } => {
            println!("Verifying {}...", file.display());
            let vr = era::verify(&file)?;

            println!("  Files checked: {}", vr.files_checked);

            match vr.signature_valid {
                Some(true) => println!("  Signature:     VALID"),
                Some(false) => println!("  Signature:     INVALID"),
                None => println!("  Signature:     not present"),
            }

            if vr.hash_failures.is_empty() {
                println!("  Hashes:        all OK");
            } else {
                eprintln!("  {} hash failures:", vr.hash_failures.len());
                for msg in &vr.hash_failures {
                    eprintln!("    {}", msg);
                }
            }
        }
    }

    Ok(())
}
