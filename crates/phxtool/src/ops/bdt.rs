//! High-level BinaryDataTree (BDT) operations.
//!
//! BDT is Ensemble's packed binary tree format, used in `.vis` and other
//! files. This module provides conversion between BDT binary and XML.

use std::fs;
use std::path::Path;

use crate::{Error, Result};

// Re-export so CLI doesn't need direct bdt dependency
pub use bdt::Endian;

/// Information about a BDT file.
pub struct BdtInfo {
    pub endian: Endian,
    pub root_name: Option<String>,
    pub node_count: usize,
    pub attribute_count: usize,
}

/// Get info about a BDT file.
pub fn info(input: &Path, endian: Endian) -> Result<BdtInfo> {
    let data = read_file(input)?;
    let root = bdt::Reader::read(&data, endian)
        .map_err(|e| Error::Other(format!("BDT parse error: {e}")))?;

    match root {
        Some(node) => {
            let attr_count = count_attributes(&node);
            Ok(BdtInfo {
                endian,
                root_name: Some(node.name.clone()),
                node_count: node.node_count(),
                attribute_count: attr_count,
            })
        }
        None => Ok(BdtInfo {
            endian,
            root_name: None,
            node_count: 0,
            attribute_count: 0,
        }),
    }
}

/// Convert a BDT binary file to XML.
pub fn to_xml(input: &Path, output: &Path, endian: Endian) -> Result<()> {
    let data = read_file(input)?;
    let root = bdt::Reader::read(&data, endian)
        .map_err(|e| Error::Other(format!("BDT parse error: {e}")))?;

    let xml = match root {
        Some(node) => node_to_xml(&node, 0),
        None => String::from("<!-- empty BDT document -->\n"),
    };

    fs::write(output, xml)?;
    Ok(())
}

/// Convert an XML file to BDT binary.
pub fn to_bdt(input: &Path, output: &Path, endian: Endian) -> Result<()> {
    let xml = fs::read_to_string(input).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound(input.to_path_buf())
        } else {
            Error::Io(e)
        }
    })?;

    let node = xml_to_node(&xml)?;
    let data = bdt::Writer::write(&node, endian)
        .map_err(|e| Error::Other(format!("BDT write error: {e}")))?;
    fs::write(output, data)?;
    Ok(())
}

// ── XML serialization ──────────────────────────────────────────────────

fn node_to_xml(node: &bdt::Node, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    let mut xml = String::new();

    if depth == 0 {
        xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    }

    xml.push_str(&format!("{}<{}", indent, node.name));

    for attr in &node.attributes {
        xml.push_str(&format!(
            " {}=\"{}\"",
            attr.name,
            escape_xml(&attr.value_string())
        ));
    }

    let text = node.text_string();
    let has_text = !text.is_empty();
    let has_children = node.has_children();

    if !has_children && !has_text {
        xml.push_str(" />\n");
    } else {
        xml.push('>');
        if has_text && !has_children {
            xml.push_str(&escape_xml(&text));
            xml.push_str(&format!("</{}>\n", node.name));
        } else {
            xml.push('\n');
            if has_text {
                xml.push_str(&format!("{}  {}\n", indent, escape_xml(&text)));
            }
            for child in &node.children {
                xml.push_str(&node_to_xml(child, depth + 1));
            }
            xml.push_str(&format!("{}</{}>\n", indent, node.name));
        }
    }

    xml
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn count_attributes(node: &bdt::Node) -> usize {
    let mut count = node.attributes.len();
    for child in &node.children {
        count += count_attributes(child);
    }
    count
}

// ── XML parsing (event-based) ───────────────────────────────────────────

/// Parse XML into a BDT Node tree using the event-based xml reader.
fn xml_to_node(xml_text: &str) -> Result<bdt::Node> {
    use xml::reader::{Event, Reader};

    let reader = Reader::new(xml_text);
    let mut stack: Vec<bdt::Node> = Vec::new();
    let mut root: Option<bdt::Node> = None;

    for event in reader {
        let event = event.map_err(|e| Error::Other(format!("XML parse error: {e}")))?;
        match event {
            Event::ElementStart { name } => {
                stack.push(bdt::Node::new(name));
            }
            Event::Attribute { name, value } => {
                if let Some(node) = stack.last_mut() {
                    node.add_attribute(bdt::Attribute::with_string(name, value));
                }
            }
            Event::ElementOpen => {
                // Tag is now open, children/text will follow
            }
            Event::ElementEmpty => {
                // Self-closing: pop the node and add as child
                if let Some(node) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        parent.add_child(node);
                    } else {
                        root = Some(node);
                    }
                }
            }
            Event::Text(text) | Event::Cdata(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty()
                    && let Some(node) = stack.last_mut()
                {
                    node.text = bdt::Variant::String(trimmed.to_string());
                }
            }
            Event::ElementClose { .. } => {
                if let Some(node) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        parent.add_child(node);
                    } else {
                        root = Some(node);
                    }
                }
            }
        }
    }

    root.ok_or_else(|| Error::Other("XML has no root element".into()))
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
