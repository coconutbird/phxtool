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
            let output = output.unwrap_or_else(|| input.with_extension("xml"));
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
    }

    Ok(())
}
