//! High-level Wwise PCK/BNK operations.

use std::fs;
use std::path::Path;

use pcktool::bnk::SoundBank;
use pcktool::pck::PckFile;

use crate::{Error, Result};

// Re-export key types so the CLI doesn't need a direct pcktool dependency.
pub use pcktool::pck::ByteOrder;

/// Summary information about a PCK file.
pub struct PckInfo {
    pub language_count: usize,
    pub languages: Vec<(u32, String)>,
    pub sound_bank_count: usize,
    pub streaming_file_count: usize,
    pub external_file_count: usize,
    pub total_bank_bytes: usize,
    pub total_streaming_bytes: usize,
    pub total_external_bytes: usize,
}

/// Summary information about a BNK file.
pub struct BnkInfo {
    pub version: u32,
    pub bank_id: u32,
    pub language_id: u32,
    pub project_id: u32,
    pub has_feedback: bool,
    pub embedded_media_count: usize,
    pub total_media_bytes: usize,
    pub has_hirc: bool,
}

/// Get info about a PCK file.
pub fn pck_info(data: &[u8]) -> Result<PckInfo> {
    let pck = PckFile::parse(data).map_err(Error::Wwise)?;
    let mut languages: Vec<(u32, String)> = pck
        .languages
        .iter()
        .map(|(&id, name)| (id, name.clone()))
        .collect();
    languages.sort_by_key(|&(id, _)| id);

    Ok(PckInfo {
        language_count: pck.languages.len(),
        languages,
        sound_bank_count: pck.sound_banks.len(),
        streaming_file_count: pck.streaming_files.len(),
        external_file_count: pck.external_files.len(),
        total_bank_bytes: pck.sound_banks.iter().map(|e| e.data.len()).sum(),
        total_streaming_bytes: pck.streaming_files.iter().map(|e| e.data.len()).sum(),
        total_external_bytes: pck.external_files.iter().map(|e| e.data.len()).sum(),
    })
}

/// Get info about a BNK file.
pub fn bnk_info(data: &[u8]) -> Result<BnkInfo> {
    let bank = SoundBank::parse(data).map_err(Error::Wwise)?;
    Ok(BnkInfo {
        version: bank.header.version,
        bank_id: bank.header.id,
        language_id: bank.header.language_id,
        project_id: bank.header.project_id,
        has_feedback: bank.header.has_feedback(),
        embedded_media_count: bank.media.len(),
        total_media_bytes: bank.media.values().map(|d| d.len()).sum(),
        has_hirc: bank.hirc_data.is_some(),
    })
}

/// An entry from a PCK listing.
pub struct PckListEntry {
    pub id_hex: String,
    pub size: usize,
    pub language: String,
    pub start_block: u32,
}

/// Kind of entries to list from a PCK file.
pub enum PckEntryKind {
    Banks,
    Streaming,
    External,
}

/// List entries from a PCK file.
pub fn pck_list(data: &[u8], kind: PckEntryKind) -> Result<Vec<PckListEntry>> {
    let pck = PckFile::parse(data).map_err(Error::Wwise)?;
    let entries = match kind {
        PckEntryKind::Banks => pck
            .sound_banks
            .iter()
            .map(|e| PckListEntry {
                id_hex: format!("0x{:08X}", e.id),
                size: e.data.len(),
                language: pck.language_name(e.language_id).to_string(),
                start_block: e.start_block,
            })
            .collect(),
        PckEntryKind::Streaming => pck
            .streaming_files
            .iter()
            .map(|e| PckListEntry {
                id_hex: format!("0x{:08X}", e.id),
                size: e.data.len(),
                language: pck.language_name(e.language_id).to_string(),
                start_block: e.start_block,
            })
            .collect(),
        PckEntryKind::External => pck
            .external_files
            .iter()
            .map(|e| PckListEntry {
                id_hex: format!("0x{:016X}", e.id),
                size: e.data.len(),
                language: pck.language_name(e.language_id).to_string(),
                start_block: e.start_block,
            })
            .collect(),
    };
    Ok(entries)
}

/// Result of a dump operation.
pub struct DumpResult {
    pub files_extracted: u32,
    pub errors: Vec<(String, String)>,
}

/// Dump/extract all audio files from a PCK file, organized by language.
pub fn dump_pck(data: &[u8], out_dir: &Path, filter_id: Option<u32>) -> Result<DumpResult> {
    let pck = PckFile::parse(data).map_err(Error::Wwise)?;
    let mut count = 0u32;
    let mut errors = Vec::new();

    for entry in &pck.sound_banks {
        if filter_id.is_some() && filter_id != Some(entry.id) {
            continue;
        }
        let lang = pck.language_name(entry.language_id);
        let lang_dir = out_dir.join(lang).join("banks");
        fs::create_dir_all(&lang_dir)?;

        let path = lang_dir.join(format!("{:08X}.bnk", entry.id));
        fs::write(&path, entry.data)?;
        count += 1;

        if let Ok(bank) = SoundBank::parse(entry.data)
            && !bank.media.is_empty()
        {
            let media_dir = lang_dir.join(format!("{:08X}_media", entry.id));
            fs::create_dir_all(&media_dir)?;
            for (&id, &wem_data) in &bank.media {
                let wem_path = media_dir.join(format!("{id:08X}.wem"));
                if let Err(e) = fs::write(&wem_path, wem_data) {
                    errors.push((wem_path.display().to_string(), e.to_string()));
                } else {
                    count += 1;
                }
            }
        }
    }

    for entry in &pck.streaming_files {
        if filter_id.is_some() && filter_id != Some(entry.id) {
            continue;
        }
        let lang = pck.language_name(entry.language_id);
        let stm_dir = out_dir.join(lang).join("streaming");
        fs::create_dir_all(&stm_dir)?;

        let path = stm_dir.join(format!("{:08X}.wem", entry.id));
        if let Err(e) = fs::write(&path, entry.data) {
            errors.push((path.display().to_string(), e.to_string()));
        } else {
            count += 1;
        }
    }

    for entry in &pck.external_files {
        let lang = pck.language_name(entry.language_id);
        let ext_dir = out_dir.join(lang).join("external");
        fs::create_dir_all(&ext_dir)?;

        let path = ext_dir.join(format!("{:016X}.wem", entry.id));
        if let Err(e) = fs::write(&path, entry.data) {
            errors.push((path.display().to_string(), e.to_string()));
        } else {
            count += 1;
        }
    }

    Ok(DumpResult {
        files_extracted: count,
        errors,
    })
}

/// Dump/extract embedded WEM files from a standalone BNK file.
pub fn dump_bnk(data: &[u8], out_dir: &Path, filter_id: Option<u32>) -> Result<DumpResult> {
    let bank = SoundBank::parse(data).map_err(Error::Wwise)?;
    let mut count = 0u32;
    let mut errors = Vec::new();

    fs::create_dir_all(out_dir)?;

    for (&id, &wem_data) in &bank.media {
        if filter_id.is_some() && filter_id != Some(id) {
            continue;
        }
        let path = out_dir.join(format!("{id:08X}.wem"));
        if let Err(e) = fs::write(&path, wem_data) {
            errors.push((path.display().to_string(), e.to_string()));
        } else {
            count += 1;
        }
    }

    Ok(DumpResult {
        files_extracted: count,
        errors,
    })
}

/// Parse a hex or decimal ID string into a u32.
pub fn parse_id(s: &str) -> std::result::Result<u32, String> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).map_err(|e| format!("invalid hex ID '{s}': {e}"))
    } else {
        s.parse::<u32>()
            .map_err(|e| format!("invalid ID '{s}': {e}"))
    }
}
