//! UGX model subcommands.

use std::path::PathBuf;

use clap::{Subcommand, ValueEnum};
use phxtool::ops::ugx;
use phxtool::ops::ugx::UgxVersion;

/// CLI-facing UGX version selector.
#[derive(Clone, Copy, ValueEnum)]
pub enum UgxVersionArg {
    /// Halo Wars: Definitive Edition
    Hw1,
    /// Halo Wars 2 (default)
    Hw2,
}

impl From<UgxVersionArg> for UgxVersion {
    fn from(v: UgxVersionArg) -> Self {
        match v {
            UgxVersionArg::Hw1 => UgxVersion::Hw1,
            UgxVersionArg::Hw2 => UgxVersion::Hw2,
        }
    }
}

#[derive(Subcommand)]
pub enum UgxCommand {
    /// Show information about a UGX model
    Info {
        /// Input UGX file
        input: PathBuf,
    },
    /// Convert UGX to glTF format
    ToGltf {
        /// Input UGX file
        input: PathBuf,
        /// Output glTF file
        #[arg(short, long)]
        output: PathBuf,
        /// Create separate .bin file instead of embedding data
        #[arg(long)]
        external_buffer: bool,
        /// Exclude skeleton/bones from export
        #[arg(long)]
        no_skeleton: bool,
    },
    /// Convert glTF/GLB to UGX format
    FromGltf {
        /// Input glTF or GLB file
        input: PathBuf,
        /// Output UGX file
        #[arg(short, long)]
        output: PathBuf,
        /// Exclude skeleton/bones from import
        #[arg(long)]
        no_skeleton: bool,
        /// Target UGX version: hw1 (Halo Wars: DE, default) or hw2 (Halo Wars 2)
        #[arg(long, value_enum, default_value = "hw1")]
        version: UgxVersionArg,
    },
}

pub fn run(cmd: UgxCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        UgxCommand::Info { input } => {
            let info = ugx::info(&input)?;
            println!("UGX File: {}", input.display());
            println!("  Materials:  {}", info.materials);
            println!("  Bones:      {}", info.bones);
            println!("  Sections:   {}", info.sections);
            println!("  Vertices:   {}", info.total_vertices);
            println!("  Triangles:  {}", info.total_triangles);
        }

        UgxCommand::ToGltf {
            input,
            output,
            external_buffer,
            no_skeleton,
        } => {
            let opts = ugx::ExportOptions {
                external_buffer,
                include_skeleton: !no_skeleton,
            };
            println!("Converting {} -> {}", input.display(), output.display());
            ugx::to_gltf(&input, &output, &opts)?;
            println!("Done!");
        }

        UgxCommand::FromGltf {
            input,
            output,
            no_skeleton,
            version,
        } => {
            println!("Converting {} -> {}", input.display(), output.display());
            ugx::from_gltf(&input, &output, !no_skeleton, version.into())?;
            println!("Done!");
        }
    }

    Ok(())
}
