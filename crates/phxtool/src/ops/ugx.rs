//! High-level UGX model operations.

use std::fs;
use std::path::Path;

use ugx::{Reader as UgxReader, Writer as UgxWriter};
use ugx_gltf::{
    GltfExportOptions, GltfImportOptions, export_to_gltf_with_buffer_name, import_from_gltf,
};

use crate::{Error, Result};

/// Information about a UGX model.
pub struct UgxInfo {
    pub materials: usize,
    pub bones: usize,
    pub sections: usize,
    pub total_vertices: usize,
    pub total_triangles: usize,
}

/// Get info about a UGX file.
pub fn info(input: &Path) -> Result<UgxInfo> {
    let data = fs::read(input).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound(input.to_path_buf())
        } else {
            Error::Io(e)
        }
    })?;
    let geom = UgxReader::read(&data).map_err(|e| Error::Other(e.to_string()))?;

    Ok(UgxInfo {
        materials: geom.materials.len(),
        bones: geom.bones.len(),
        sections: geom.sections.len(),
        total_vertices: geom.total_vertices(),
        total_triangles: geom.total_triangles(),
    })
}

/// Export options for UGX → glTF.
pub struct ExportOptions {
    pub external_buffer: bool,
    pub include_skeleton: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            external_buffer: false,
            include_skeleton: true,
        }
    }
}

/// Convert a UGX file to glTF.
pub fn to_gltf(input: &Path, output: &Path, opts: &ExportOptions) -> Result<()> {
    let data = fs::read(input).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound(input.to_path_buf())
        } else {
            Error::Io(e)
        }
    })?;
    let geom = UgxReader::read(&data).map_err(|e| Error::Other(e.to_string()))?;

    let gltf_opts = GltfExportOptions {
        embed_buffers: !opts.external_buffer,
        include_materials: true,
        include_skeleton: opts.include_skeleton,
    };

    let bin_name = output
        .with_extension("bin")
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let export = export_to_gltf_with_buffer_name(&geom, &gltf_opts, &bin_name)
        .map_err(|e| Error::Other(e.to_string()))?;

    fs::write(output, &export.json)?;

    if let Some(buffer_data) = export.buffer {
        let bin_path = output.with_extension("bin");
        fs::write(&bin_path, &buffer_data)?;
    }

    Ok(())
}

/// Re-export so CLI doesn't need a direct ugx dependency for the version enum.
pub use ugx::UgxVersion;

/// Convert a glTF/GLB file to UGX.
pub fn from_gltf(
    input: &Path,
    output: &Path,
    include_skeleton: bool,
    version: UgxVersion,
) -> Result<()> {
    let is_glb = input
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("glb"));

    let (json_str, buffer_data) = if is_glb {
        let data = fs::read(input)?;
        parse_glb(&data)?
    } else {
        let json_str = fs::read_to_string(input)?;
        let buffer_data = resolve_gltf_buffer(&json_str, input)?;
        (json_str, buffer_data)
    };

    let opts = GltfImportOptions {
        include_skeleton,
        include_materials: true,
        version,
    };

    let geom = import_from_gltf(&json_str, buffer_data.as_deref(), &opts)
        .map_err(|e| Error::Other(e.to_string()))?;

    let ugx_data = UgxWriter::write(&geom, version).map_err(|e| Error::Other(e.to_string()))?;
    fs::write(output, &ugx_data)?;
    Ok(())
}

fn parse_glb(data: &[u8]) -> Result<(String, Option<Vec<u8>>)> {
    if data.len() < 12 {
        return Err(Error::InvalidFormat("GLB file too small".into()));
    }
    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if magic != 0x46546C67 {
        return Err(Error::InvalidFormat(format!(
            "invalid GLB magic: 0x{:08X}",
            magic
        )));
    }

    let mut offset = 12usize;
    let mut json_str = String::new();
    let mut bin_data: Option<Vec<u8>> = None;

    while offset + 8 <= data.len() {
        let chunk_length =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let chunk_type = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
        offset += 8;
        if offset + chunk_length > data.len() {
            return Err(Error::InvalidFormat("GLB chunk extends beyond file".into()));
        }
        match chunk_type {
            0x4E4F534A => {
                json_str =
                    String::from_utf8_lossy(&data[offset..offset + chunk_length]).into_owned()
            }
            0x004E4942 => bin_data = Some(data[offset..offset + chunk_length].to_vec()),
            _ => {}
        }
        offset += chunk_length;
    }

    if json_str.is_empty() {
        return Err(Error::InvalidFormat("GLB missing JSON chunk".into()));
    }
    Ok((json_str, bin_data))
}

fn resolve_gltf_buffer(json_str: &str, input: &Path) -> Result<Option<Vec<u8>>> {
    // Simple URI resolution - look for external buffer files
    if let Some(start) = json_str.find("\"uri\"")
        && let Some(colon) = json_str[start..].find(':')
    {
        let after = &json_str[start + colon + 1..];
        let after = after.trim();
        if after.starts_with('"')
            && !after.starts_with("\"data:")
            && let Some(end) = after[1..].find('"')
        {
            let uri = &after[1..1 + end];
            let base_dir = input.parent().unwrap_or(Path::new("."));
            let bin_path = base_dir.join(uri);
            if bin_path.exists() {
                return Ok(Some(fs::read(&bin_path)?));
            }
        }
    }
    Ok(None)
}
