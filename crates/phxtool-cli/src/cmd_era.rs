//! ERA archive subcommands.

use std::path::PathBuf;

use clap::Subcommand;
use phxtool::era_ops;

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
        } => {
            let outdir = output.unwrap_or_else(|| {
                let stem = file.file_stem().unwrap_or_default();
                PathBuf::from(stem)
            });

            let opts = era_ops::ExpandOptions {
                translate_xmb: !no_translate,
                overwrite: !no_overwrite,
                listing_only,
                filter,
            };

            println!("Expanding {} -> {}", file.display(), outdir.display());
            let result = era_ops::expand(&file, &outdir, &opts)?;

            println!(
                "Extracted {} files ({} XMB→XML translations)",
                result.files_extracted, result.files_translated
            );
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
            let opts = era_ops::BuildOptions {
                translate_xml: !no_translate,
                encrypt: !no_encrypt,
            };

            println!("Building {} from {}", output.display(), input.display());
            let count = era_ops::build(&input, &output, &opts)?;
            println!("Done! {} files archived.", count);
        }

        EraCommand::List { file } => {
            let entries = era_ops::list(&file)?;
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
                    era_ops::format_size(e.compressed_size as u64),
                    era_ops::format_size(e.decompressed_size as u64),
                    name,
                    width = max_name
                );
            }
            println!("\nTotal: {} entries", entries.len());
        }

        EraCommand::Info { file } => {
            let info = era_ops::info(&file)?;
            println!("Archive: {}", file.display());
            println!("  ECF Magic:    0x{:08X}", info.ecf_magic);
            println!("  ERA Magic:    0x{:08X}", info.archive_magic);
            println!("  Files:        {}", info.file_count);
            println!(
                "  Compressed:   {}",
                era_ops::format_size(info.total_compressed)
            );
            println!(
                "  Decompressed: {}",
                era_ops::format_size(info.total_decompressed)
            );
            if info.total_decompressed > 0 {
                let ratio = (info.total_compressed as f64 / info.total_decompressed as f64) * 100.0;
                println!("  Ratio:        {:.1}%", ratio);
            }
        }

        EraCommand::Decrypt { input, output } => {
            println!("Decrypting {} -> {}", input.display(), output.display());
            era_ops::decrypt(&input, &output)?;
            println!("Done!");
        }

        EraCommand::Encrypt { input, output } => {
            println!("Encrypting {} -> {}", input.display(), output.display());
            era_ops::encrypt(&input, &output)?;
            println!("Done!");
        }
    }

    Ok(())
}
