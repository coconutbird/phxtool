//! High-level ECF (Ensemble Common Format) container operations.
//!
//! ECF is the generic container format wrapping terrain (XTD/XTT), UGX models,
//! DDX textures, and other assets. This module provides expand (dump chunks)
//! and build (repack chunks) workflows.

use std::fs;
use std::path::Path;

use crate::{Error, Result};

/// Information about an ECF container.
pub struct EcfInfo {
    pub magic: u32,
    pub chunk_count: usize,
    pub total_size: u64,
    pub chunks: Vec<ChunkInfo>,
}

/// Information about a single ECF chunk.
pub struct ChunkInfo {
    pub index: usize,
    pub id: u64,
    pub offset: u32,
    pub size: u32,
    pub flags: u8,
}

/// Read ECF info from a file.
pub fn info(path: &Path) -> Result<EcfInfo> {
    let data = read_file(path)?;
    let reader = ecf::Reader::new(&data)?;
    let header = reader.header();
    let chunks: Vec<ChunkInfo> = reader
        .chunks()
        .iter()
        .enumerate()
        .map(|(i, c)| ChunkInfo {
            index: i,
            id: c.id,
            offset: c.offset,
            size: c.size,
            flags: c.flags,
        })
        .collect();

    Ok(EcfInfo {
        magic: header.magic,
        chunk_count: header.num_chunks as usize,
        total_size: data.len() as u64,
        chunks,
    })
}

/// Expand (dump) all chunks from an ECF file into a directory.
///
/// Each chunk is written as `<index>_<id_hex>.bin`.
/// Returns the number of chunks extracted.
pub fn expand(path: &Path, output: &Path, overwrite: bool) -> Result<usize> {
    let data = read_file(path)?;
    let reader = ecf::Reader::new(&data)?;

    fs::create_dir_all(output)?;

    let mut count = 0;
    for (i, chunk) in reader.chunks().iter().enumerate() {
        let filename = format!("{:03}_{:08X}.bin", i, chunk.id);
        let out_path = output.join(&filename);

        if !overwrite && out_path.exists() {
            continue;
        }

        let chunk_data = reader.chunk_data(i)?;
        fs::write(&out_path, chunk_data)?;
        count += 1;
    }

    Ok(count)
}

/// Build an ECF file from a directory of chunk files.
///
/// Expects files named `<index>_<id_hex>.bin` (as produced by `expand`).
/// Returns the number of chunks packed.
pub fn build(input: &Path, output: &Path, magic: u32) -> Result<usize> {
    if !input.is_dir() {
        return Err(Error::FileNotFound(input.to_path_buf()));
    }

    let mut entries: Vec<(u64, Vec<u8>)> = Vec::new();

    let mut files: Vec<_> = fs::read_dir(input)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "bin"))
        .collect();
    files.sort_by_key(|e| e.file_name());

    for entry in &files {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        // Parse "NNN_IIIIIIII.bin"
        let id = name_str
            .split('_')
            .nth(1)
            .and_then(|s| s.strip_suffix(".bin"))
            .and_then(|s| u64::from_str_radix(s, 16).ok())
            .unwrap_or(0);

        let data = fs::read(entry.path())?;
        entries.push((id, data));
    }

    let mut writer = ecf::Writer::new(magic);
    for (id, data) in &entries {
        writer.add_chunk(*id, data.clone());
    }

    let bytes = writer.finalize()?;
    fs::write(output, bytes)?;

    Ok(entries.len())
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound(path.to_path_buf())
        } else {
            Error::Io(e)
        }
    })
}
