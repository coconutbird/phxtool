//! UGX model subcommands.

use std::path::PathBuf;

use clap::Subcommand;
use phxtool::ops::ugx;

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
        } => {
            println!("Converting {} -> {}", input.display(), output.display());
            ugx::from_gltf(&input, &output, !no_skeleton)?;
            println!("Done!");
        }
    }

    Ok(())
}
