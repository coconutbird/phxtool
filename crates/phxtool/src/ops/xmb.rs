//! High-level XMB ↔ XML conversion operations.

use std::fs;
use std::path::Path;

use xmb::{Document, Reader, Writer};

// Re-export Format so CLI doesn't need a direct xmb dependency
pub use xmb::Format;

use crate::{Error, Result};

/// Convert an XMB file to XML.
pub fn to_xml(input: &Path, output: &Path) -> Result<()> {
    let data = fs::read(input).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound(input.to_path_buf())
        } else {
            Error::Io(e)
        }
    })?;

    let doc = Reader::read(&data)?;
    let xml = doc.to_xml();
    fs::write(output, xml)?;
    Ok(())
}

/// Convert an XML file to XMB.
pub fn to_xmb(input: &Path, output: &Path, format: Format, compress: bool) -> Result<()> {
    let xml = fs::read_to_string(input).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound(input.to_path_buf())
        } else {
            Error::Io(e)
        }
    })?;

    let doc = Document::from_xml(&xml)?;
    let bytes = if compress {
        Writer::write(&doc, format)?
    } else {
        Writer::write_uncompressed(&doc, format)?
    };
    fs::write(output, bytes)?;
    Ok(())
}

/// Information about an XMB file.
pub struct XmbInfo {
    pub format: Format,
    pub root_element: Option<String>,
    pub total_nodes: usize,
    pub root_attributes: usize,
    pub root_children: usize,
}

/// Get info about an XMB file.
pub fn info(input: &Path) -> Result<XmbInfo> {
    let data = fs::read(input).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound(input.to_path_buf())
        } else {
            Error::Io(e)
        }
    })?;

    let doc = Reader::read(&data)?;

    let (root_element, total_nodes, root_attributes, root_children) = if let Some(root) = doc.root()
    {
        (
            Some(root.name.clone()),
            root.node_count(),
            root.attributes.len(),
            root.children.len(),
        )
    } else {
        (None, 0, 0, 0)
    };

    Ok(XmbInfo {
        format: doc.format(),
        root_element,
        total_nodes,
        root_attributes,
        root_children,
    })
}

/// Batch convert XMB/XML files.
/// Returns (successes, errors).
pub fn batch_convert(
    files: &[std::path::PathBuf],
    format: Format,
    overwrite: bool,
    compress: bool,
) -> (usize, Vec<(String, String)>) {
    let mut success = 0;
    let mut errors = Vec::new();

    for file in files {
        let ext = file
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let result = match ext.as_str() {
            "xml" => {
                let out = if overwrite {
                    file.with_extension("xmb")
                } else {
                    unique_path(&file.with_extension("xmb"))
                };
                to_xmb(file, &out, format, compress)
            }
            "xmb" => {
                let out = if overwrite {
                    xmb_to_xml_path(file)
                } else {
                    unique_path(&xmb_to_xml_path(file))
                };
                to_xml(file, &out)
            }
            _ => {
                errors.push((file.display().to_string(), "unknown extension".to_string()));
                continue;
            }
        };

        match result {
            Ok(()) => success += 1,
            Err(e) => errors.push((file.display().to_string(), e.to_string())),
        }
    }

    (success, errors)
}

/// Derive the XML output path for a `.xmb` input.
pub fn xmb_to_xml_path(path: &Path) -> std::path::PathBuf {
    let s = path.to_string_lossy().to_lowercase();
    if s.ends_with(".xml.xmb") {
        path.with_extension("") // strip .xmb, keeping .xml
    } else {
        path.with_extension("xml")
    }
}

fn unique_path(path: &Path) -> std::path::PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path.extension().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or(Path::new("."));
    for i in 1..1000 {
        let candidate = parent.join(format!("{}_{}.{}", stem, i, ext));
        if !candidate.exists() {
            return candidate;
        }
    }
    path.to_path_buf()
}
