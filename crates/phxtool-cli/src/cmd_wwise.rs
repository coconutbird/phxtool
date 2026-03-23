//! Wwise PCK/BNK subcommands.

use std::path::PathBuf;

use clap::Subcommand;
use phxtool::wwise_ops::{self, PckEntryKind};

#[derive(Subcommand)]
pub enum WwiseCommand {
    /// Show information about a PCK or BNK file.
    Info {
        /// Path to a .pck or .bnk file.
        file: PathBuf,
    },
    /// List entries in a PCK file.
    #[command(alias = "ls")]
    List {
        /// Path to a .pck file.
        file: PathBuf,
        /// Show streaming files instead of sound banks.
        #[arg(short = 's', long)]
        streaming: bool,
        /// Show external files instead of sound banks.
        #[arg(short = 'e', long)]
        external: bool,
    },
    /// Extract audio files from a PCK or BNK file.
    #[command(alias = "extract")]
    Dump {
        /// Path to a .pck or .bnk file.
        file: PathBuf,
        /// Output directory (default: <filename>_extracted).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Only extract a specific source ID (hex or decimal).
        #[arg(long, value_parser = wwise_ops::parse_id)]
        id: Option<u32>,
    },
}

pub fn run(cmd: WwiseCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        WwiseCommand::Info { file } => run_info(&file),
        WwiseCommand::List {
            file,
            streaming,
            external,
        } => run_list(&file, streaming, external),
        WwiseCommand::Dump { file, output, id } => run_dump(&file, output, id),
    }
}

type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

fn mmap_file(path: &std::path::Path) -> BoxResult<memmap2::Mmap> {
    let file = std::fs::File::open(path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    Ok(mmap)
}

fn file_ext(path: &std::path::Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn run_info(file: &std::path::Path) -> BoxResult<()> {
    let mmap = mmap_file(file)?;
    let ext = file_ext(file);

    match ext.as_str() {
        "pck" => {
            let info = wwise_ops::pck_info(&mmap)?;
            println!("PCK File Info");
            println!("─────────────────────────────────");
            println!("Languages:        {}", info.language_count);
            for (id, name) in &info.languages {
                println!("  [{id}] {name}");
            }
            println!("Sound Banks:      {}", info.sound_bank_count);
            println!("Streaming Files:  {}", info.streaming_file_count);
            println!("External Files:   {}", info.external_file_count);
            println!("─────────────────────────────────");
            println!("Bank data:        {} bytes", info.total_bank_bytes);
            println!("Streaming data:   {} bytes", info.total_streaming_bytes);
            println!("External data:    {} bytes", info.total_external_bytes);
        }
        "bnk" => {
            let info = wwise_ops::bnk_info(&mmap)?;
            println!("BNK File Info");
            println!("─────────────────────────────────");
            println!("Version:          0x{:X}", info.version);
            println!("Bank ID:          0x{:08X}", info.bank_id);
            println!("Language ID:      {}", info.language_id);
            println!("Project ID:       {}", info.project_id);
            println!("Feedback:         {}", info.has_feedback);
            println!("Embedded Media:   {}", info.embedded_media_count);
            println!("Media data:       {} bytes", info.total_media_bytes);
            if info.has_hirc {
                println!("HIRC:             present");
            }
        }
        _ => {
            return Err(Box::new(phxtool::Error::InvalidFormat(format!(
                "unsupported extension '.{ext}' (expected .pck or .bnk)"
            ))));
        }
    }
    Ok(())
}

fn run_list(file: &std::path::Path, streaming: bool, external: bool) -> BoxResult<()> {
    let mmap = mmap_file(file)?;
    let kind = if streaming {
        PckEntryKind::Streaming
    } else if external {
        PckEntryKind::External
    } else {
        PckEntryKind::Banks
    };

    let entries = wwise_ops::pck_list(&mmap, kind)?;
    println!("{:<18} {:<10} {:<10} Block", "ID", "Size", "Language");
    println!("{}", "─".repeat(56));
    for e in &entries {
        println!(
            "{:<18} {:<10} {:<10} {}",
            e.id_hex, e.size, e.language, e.start_block
        );
    }
    println!("\nTotal: {} entries", entries.len());
    Ok(())
}

fn run_dump(
    file: &std::path::Path,
    output: Option<PathBuf>,
    filter_id: Option<u32>,
) -> BoxResult<()> {
    let mmap = mmap_file(file)?;
    let ext = file_ext(file);
    let stem = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let out_dir = output.unwrap_or_else(|| PathBuf::from(format!("{stem}_extracted")));

    let result = match ext.as_str() {
        "pck" => wwise_ops::dump_pck(&mmap, &out_dir, filter_id)?,
        "bnk" => wwise_ops::dump_bnk(&mmap, &out_dir, filter_id)?,
        _ => {
            return Err(Box::new(phxtool::Error::InvalidFormat(format!(
                "unsupported extension '.{ext}' (expected .pck or .bnk)"
            ))));
        }
    };

    for (path, err) in &result.errors {
        eprintln!("  warning: {path}: {err}");
    }
    println!(
        "Extracted {} files to {}",
        result.files_extracted,
        out_dir.display()
    );
    Ok(())
}
