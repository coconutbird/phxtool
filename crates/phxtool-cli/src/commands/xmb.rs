//! XMB ↔ XML conversion subcommands.

use std::path::PathBuf;

use clap::{Subcommand, ValueEnum};
use phxtool::ops::xmb;

#[derive(Subcommand)]
pub enum XmbCommand {
    /// Convert XMB to XML
    ToXml {
        /// Input XMB file
        input: PathBuf,
        /// Output XML file (defaults to input with .xml extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Convert XML to XMB
    ToXmb {
        /// Input XML file
        input: PathBuf,
        /// Output XMB file (defaults to input with .xmb extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Output format
        #[arg(short, long, value_enum, default_value = "pc")]
        format: FormatArg,
        /// Disable compression
        #[arg(short = 'u', long = "no-compress")]
        no_compress: bool,
    },
    /// Show XMB file information
    Info {
        /// Input XMB file
        input: PathBuf,
    },
    /// Batch convert all XMB/XML files in a directory (recursive)
    Batch {
        /// Input directory to scan
        input: PathBuf,
        /// Output format for XML→XMB conversion
        #[arg(short, long, value_enum, default_value = "pc")]
        format: FormatArg,
        /// Disable compression for XML→XMB
        #[arg(short = 'u', long = "no-compress")]
        no_compress: bool,
        /// Overwrite existing output files
        #[arg(long)]
        overwrite: bool,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum FormatArg {
    /// PC/HWDE format (little-endian)
    Pc,
    /// Xbox 360 format (big-endian)
    Xbox360,
}

impl From<FormatArg> for phxtool::ops::xmb::Format {
    fn from(arg: FormatArg) -> Self {
        match arg {
            FormatArg::Pc => phxtool::ops::xmb::Format::PC,
            FormatArg::Xbox360 => phxtool::ops::xmb::Format::Xbox360,
        }
    }
}

pub fn run(cmd: XmbCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        XmbCommand::ToXml { input, output } => {
            let output = output.unwrap_or_else(|| xmb::xmb_to_xml_path(&input));
            println!("Converting {} -> {}", input.display(), output.display());
            xmb::to_xml(&input, &output)?;
            println!("Done!");
        }

        XmbCommand::ToXmb {
            input,
            output,
            format,
            no_compress,
        } => {
            let output = output.unwrap_or_else(|| input.with_extension("xmb"));
            println!(
                "Converting {} -> {} ({:?}{})",
                input.display(),
                output.display(),
                format,
                if no_compress { ", uncompressed" } else { "" }
            );
            xmb::to_xmb(&input, &output, format.into(), !no_compress)?;
            println!("Done!");
        }

        XmbCommand::Info { input } => {
            let info = xmb::info(&input)?;
            println!("File: {}", input.display());
            println!("Format: {:?}", info.format);
            if let Some(root) = &info.root_element {
                println!("Root element: <{}>", root);
                println!("Attributes: {}", info.root_attributes);
                println!("Children: {}", info.root_children);
                println!("Total nodes: {}", info.total_nodes);
            } else {
                println!("(empty document)");
            }
        }

        XmbCommand::Batch {
            input,
            format,
            no_compress,
            overwrite,
        } => {
            println!("Scanning {} for XMB/XML files...", input.display());
            let files = collect_xmb_xml_files(&input)?;
            if files.is_empty() {
                println!("No XMB or XML files found.");
                return Ok(());
            }
            println!("Found {} files", files.len());
            let (success, errors) =
                xmb::batch_convert(&files, format.into(), overwrite, !no_compress);
            println!("Converted {} files", success);
            if !errors.is_empty() {
                eprintln!("{} errors:", errors.len());
                for (file, err) in &errors {
                    eprintln!("  {}: {}", file, err);
                }
            }
        }
    }

    Ok(())
}

fn collect_xmb_xml_files(
    dir: &std::path::Path,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    collect_recursive(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_recursive(
    dir: &std::path::Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_recursive(&path, files)?;
        } else if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if ext == "xmb" || ext == "xml" {
                files.push(path);
            }
        }
    }
    Ok(())
}
