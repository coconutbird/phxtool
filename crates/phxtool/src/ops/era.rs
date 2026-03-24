//! High-level ERA archive operations.
//!
//! Provides expand (with optional XMB→XML translation), build, list, info,
//! encrypt, and decrypt operations matching PhxTool's ERA tool.

use std::fs;
use std::io::BufReader;
use std::path::Path;

use era::{Reader, TeaKeys, Writer};
use tiger::{Digest, Tiger};

type DecryptReader<R> = era::crypto::decrypt::Reader<R>;
type EncryptWriter<W> = era::crypto::encrypt::Writer<W>;

use crate::{Error, Result};

/// Options for expanding (extracting) an ERA archive.
pub struct ExpandOptions {
    /// Convert XMB files to XML during extraction.
    pub translate_xmb: bool,
    /// Overwrite existing files.
    pub overwrite: bool,
    /// Only dump a listing file, don't extract.
    pub listing_only: bool,
    /// Glob pattern to filter files.
    pub filter: Option<String>,
    /// Decompress Scaleform UI files (`.gfx`/`.swf`) to `.bin`.
    pub decompress_ui: bool,
    /// Convert GFX files to SWF (header swap).
    pub gfx_to_swf: bool,
    /// Skip hash/signature verification.
    pub skip_verify: bool,
}

impl Default for ExpandOptions {
    fn default() -> Self {
        Self {
            translate_xmb: true,
            overwrite: true,
            listing_only: false,
            filter: None,
            decompress_ui: false,
            gfx_to_swf: false,
            skip_verify: false,
        }
    }
}

/// Result of an expand operation.
pub struct ExpandResult {
    pub files_extracted: usize,
    pub files_translated: usize,
    pub files_verified: usize,
    pub hash_failures: Vec<String>,
    pub errors: Vec<(String, String)>,
}

/// Result of a standalone verify operation.
pub struct VerifyResult {
    pub files_checked: usize,
    pub hash_failures: Vec<String>,
    pub signature_valid: Option<bool>,
}

/// Information about an ERA archive entry.
pub struct EntryInfo {
    pub index: usize,
    pub filename: Option<String>,
    pub compressed_size: u32,
    pub decompressed_size: u32,
}

/// Summary information about an ERA archive.
pub struct ArchiveInfo {
    pub file_count: usize,
    pub total_compressed: u64,
    pub total_decompressed: u64,
    pub ecf_magic: u32,
    pub archive_magic: u32,
    pub has_signature: bool,
    pub signature_size: usize,
}

/// Open an encrypted ERA file for reading.
pub fn open_era(path: &Path) -> Result<Reader<DecryptReader<BufReader<fs::File>>>> {
    let file = fs::File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound(path.to_path_buf())
        } else {
            Error::Io(e)
        }
    })?;
    let buf = BufReader::new(file);
    Ok(Reader::from_encrypted(
        buf,
        TeaKeys::default_archive_keys(),
    )?)
}

/// List all entries in an ERA archive.
pub fn list(path: &Path) -> Result<Vec<EntryInfo>> {
    let archive = open_era(path)?;
    let entries = archive
        .iter()
        .enumerate()
        .map(|(i, entry)| EntryInfo {
            index: i,
            filename: if i == 0 {
                Some("<filename table>".to_string())
            } else {
                entry.filename.clone()
            },
            compressed_size: entry.compressed_size(),
            decompressed_size: entry.decompressed_size(),
        })
        .collect();
    Ok(entries)
}

/// Get summary info about an ERA archive.
pub fn info(path: &Path) -> Result<ArchiveInfo> {
    let archive = open_era(path)?;
    let mut total_compressed: u64 = 0;
    let mut total_decompressed: u64 = 0;
    for entry in archive.iter().skip(1) {
        total_compressed += entry.compressed_size() as u64;
        total_decompressed += entry.decompressed_size() as u64;
    }
    Ok(ArchiveInfo {
        file_count: archive.len().saturating_sub(1),
        total_compressed,
        total_decompressed,
        ecf_magic: archive.ecf_header.magic,
        archive_magic: archive.archive_header.archive_magic,
        has_signature: archive.has_signature(),
        signature_size: archive.signature().len(),
    })
}

/// Expand (extract) an ERA archive to a directory.
pub fn expand(path: &Path, output: &Path, opts: &ExpandOptions) -> Result<ExpandResult> {
    let mut archive = open_era(path)?;
    fs::create_dir_all(output)?;

    let pattern = opts
        .filter
        .as_ref()
        .map(|f| glob::Pattern::new(f))
        .transpose()
        .map_err(|e| Error::Other(format!("invalid glob pattern: {}", e)))?;

    let mut result = ExpandResult {
        files_extracted: 0,
        files_translated: 0,
        files_verified: 0,
        hash_failures: Vec::new(),
        errors: Vec::new(),
    };

    if opts.listing_only {
        // Just write a listing file
        let listing_path = output.join("_listing.txt");
        let mut listing = String::new();
        for i in 1..archive.len() {
            if let Some(entry) = archive.entry(i) {
                let name = entry.filename.as_deref().unwrap_or("<unnamed>");
                listing.push_str(&format!("{}\n", name));
            }
        }
        fs::write(listing_path, listing)?;
        return Ok(result);
    }

    for i in 1..archive.len() {
        let entry = archive.entry(i).unwrap().clone();
        let filename = match &entry.filename {
            Some(f) => f.clone(),
            None => continue,
        };

        let normalized = filename.replace('\\', "/");
        if let Some(ref pat) = pattern
            && !pat.matches(&normalized)
        {
            continue;
        }

        // Read compressed data and verify Tiger128 hash if enabled
        let data: Vec<u8> = if !opts.skip_verify {
            match archive.read_entry_compressed(i) {
                Ok((compressed, _decomp_size, stored_hash)) => {
                    // Verify Tiger128 hash of compressed data
                    let hash = Tiger::digest(&compressed);
                    let mut computed = [0u8; 16];
                    computed.copy_from_slice(&hash[..16]);
                    // Tiger outputs LE words; stored hashes use BE words
                    computed[0..8].reverse();
                    computed[8..16].reverse();
                    if stored_hash != [0u8; 16] && computed != stored_hash {
                        result.hash_failures.push(format!(
                            "{}: expected {:02x?}, got {:02x?}",
                            normalized, stored_hash, computed
                        ));
                    } else {
                        result.files_verified += 1;
                    }
                    // Decompress
                    match archive.entry(i).unwrap().decompress(&compressed) {
                        Ok(d) => d,
                        Err(e) => {
                            result.errors.push((filename, e.to_string()));
                            continue;
                        }
                    }
                }
                Err(e) => {
                    result.errors.push((filename, e.to_string()));
                    continue;
                }
            }
        } else {
            match archive.read_entry(i) {
                Ok(d) => d,
                Err(e) => {
                    result.errors.push((filename, e.to_string()));
                    continue;
                }
            }
        };

        let file_path = output.join(&normalized);
        if !opts.overwrite && file_path.exists() {
            continue;
        }

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Check if this is an XMB file and translation is enabled
        if opts.translate_xmb && normalized.ends_with(".xmb") {
            match xmb::Reader::read(&data) {
                Ok(doc) => {
                    let xml = doc.to_xml();
                    let xml_path = super::xmb::xmb_to_xml_path(&file_path);
                    fs::write(&xml_path, xml)?;
                    result.files_translated += 1;
                    result.files_extracted += 1;
                    continue;
                }
                Err(_) => {
                    // Fall through to write the raw XMB
                }
            }
        }

        // Scaleform processing
        if super::scaleform::is_scaleform_extension(&normalized) {
            if opts.decompress_ui && super::scaleform::is_scaleform(&data) {
                match super::scaleform::decompress_scaleform(&data) {
                    Ok(decompressed) => {
                        let bin_path = file_path.with_extension(format!(
                            "{}.bin",
                            file_path.extension().unwrap_or_default().to_string_lossy()
                        ));
                        fs::write(&bin_path, decompressed)?;
                        result.files_extracted += 1;
                    }
                    Err(e) => {
                        result
                            .errors
                            .push((filename.clone(), format!("decompress: {e}")));
                    }
                }
            }

            if opts.gfx_to_swf
                && super::scaleform::is_scaleform(&data)
                && !super::scaleform::is_swf_header(&data)
            {
                match super::scaleform::gfx_to_swf(&data) {
                    Ok(swf_data) => {
                        let swf_path = file_path.with_extension("swf");
                        fs::write(&swf_path, swf_data)?;
                        result.files_extracted += 1;
                    }
                    Err(e) => {
                        result
                            .errors
                            .push((filename.clone(), format!("gfx→swf: {e}")));
                    }
                }
            }
        }

        fs::write(&file_path, &data)?;
        result.files_extracted += 1;
    }

    Ok(result)
}

/// Options for building an ERA archive.
pub struct BuildOptions {
    /// Convert XML files to XMB before archiving.
    pub translate_xml: bool,
    /// Encrypt the output archive.
    pub encrypt: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            translate_xml: true,
            encrypt: true,
        }
    }
}

/// Build an ERA archive from a directory.
pub fn build(input: &Path, output: &Path, opts: &BuildOptions) -> Result<usize> {
    if !input.is_dir() {
        return Err(Error::FileNotFound(input.to_path_buf()));
    }

    let mut writer = Writer::new();
    let mut file_count = 0;

    collect_files_recursive(
        input,
        input,
        &mut writer,
        &mut file_count,
        opts.translate_xml,
    )?;

    if opts.encrypt {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(output)?;
        let keys = TeaKeys::default_archive_keys();
        writer
            .write_to_encrypted(file, keys)
            .map_err(|e| Error::Other(format!("encryption error: {}", e)))?;
    } else {
        let data = writer.finalize().map_err(Error::Era)?;
        fs::write(output, data)?;
    }

    Ok(file_count)
}

fn collect_files_recursive(
    base: &Path,
    dir: &Path,
    writer: &mut Writer,
    count: &mut usize,
    translate_xml: bool,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_files_recursive(base, &path, writer, count, translate_xml)?;
        } else if path.is_file() {
            let rel_path = path
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .replace('/', "\\");

            // Optionally translate XML→XMB
            if translate_xml
                && rel_path.to_lowercase().ends_with(".xml")
                && !rel_path.starts_with("_")
            {
                let xml = fs::read_to_string(&path)?;
                if let Ok(doc) = xmb::Document::from_xml(&xml)
                    && let Ok(bytes) = xmb::Writer::write(&doc, xmb::Format::PC)
                {
                    let xmb_path = rel_path
                        .strip_suffix(".xml")
                        .or(rel_path.strip_suffix(".XML"))
                        .unwrap_or(&rel_path);
                    let xmb_path = format!("{}.xmb", xmb_path);
                    writer.add_file(xmb_path, bytes);
                    *count += 1;
                    continue;
                }
            }

            let data = fs::read(&path)?;
            writer.add_file(rel_path, data);
            *count += 1;
        }
    }
    Ok(())
}

/// Decrypt an ERA file (encrypted → plaintext ECF).
pub fn decrypt(input: &Path, output: &Path) -> Result<()> {
    let file = fs::File::open(input).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound(input.to_path_buf())
        } else {
            Error::Io(e)
        }
    })?;
    let keys = TeaKeys::default_archive_keys();
    let mut reader = DecryptReader::new(BufReader::new(file), keys);

    let mut data = Vec::new();
    std::io::Read::read_to_end(&mut reader, &mut data)?;
    fs::write(output, data)?;
    Ok(())
}

/// Encrypt a plaintext ERA/ECF file.
pub fn encrypt(input: &Path, output: &Path) -> Result<()> {
    let data = fs::read(input).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound(input.to_path_buf())
        } else {
            Error::Io(e)
        }
    })?;
    let keys = TeaKeys::default_archive_keys();
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(output)?;
    let mut writer = EncryptWriter::new(file, keys);
    std::io::Write::write_all(&mut writer, &data)?;
    writer
        .finish()
        .map_err(|e| Error::Other(format!("encryption error: {}", e)))?;
    Ok(())
}

/// Verify all Tiger128 hashes and the archive signature without extracting.
pub fn verify(path: &Path) -> Result<VerifyResult> {
    let mut archive = open_era(path)?;

    let mut vr = VerifyResult {
        files_checked: 0,
        hash_failures: Vec::new(),
        signature_valid: None,
    };

    // Verify signature if present
    if archive.has_signature() {
        match archive.verify_signature() {
            Ok(valid) => vr.signature_valid = Some(valid),
            Err(_) => vr.signature_valid = Some(false),
        }
    }

    // Verify Tiger128 hashes for all entries (including filename table at 0)
    for i in 0..archive.len() {
        let entry = archive.entry(i).unwrap().clone();
        let name = entry.filename.as_deref().unwrap_or(if i == 0 {
            "<filename table>"
        } else {
            "<unnamed>"
        });

        match archive.read_entry_compressed(i) {
            Ok((compressed, _decomp_size, stored_hash)) => {
                if stored_hash == [0u8; 16] {
                    // No hash stored, skip
                    vr.files_checked += 1;
                    continue;
                }
                let hash = Tiger::digest(&compressed);
                let mut computed = [0u8; 16];
                computed.copy_from_slice(&hash[..16]);
                // Tiger outputs LE words; stored hashes use BE words
                computed[0..8].reverse();
                computed[8..16].reverse();
                if computed != stored_hash {
                    vr.hash_failures.push(format!(
                        "{}: expected {:02x?}, got {:02x?}",
                        name, stored_hash, computed
                    ));
                }
                vr.files_checked += 1;
            }
            Err(e) => {
                vr.hash_failures
                    .push(format!("{}: read error: {}", name, e));
            }
        }
    }

    Ok(vr)
}

// Re-export for backward compat
pub use super::util::format_size;
