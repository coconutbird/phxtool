//! Scaleform GFX ↔ SWF helpers.
//!
//! Halo Wars uses Scaleform for its UI. `.gfx` files are essentially `.swf`
//! files with the first 3 bytes of the header changed. Compressed variants
//! add an 8-byte header (4-byte signature + 4-byte decompressed size) followed
//! by zlib-compressed data.

use crate::{Error, Result};

// Signatures (first 3 bytes, stored as u32 with top byte zeroed)
const SWF_SIGNATURE: u32 = 0x00_53_57_46; // \x00SWF — uncompressed
const GFX_SIGNATURE: u32 = 0x00_47_46_58; // \x00GFX — uncompressed
const SWC_SIGNATURE: u32 = 0x00_53_57_43; // \x00SWC — zlib-compressed SWF
const GFC_SIGNATURE: u32 = 0x00_47_46_43; // \x00GFC — zlib-compressed GFX

/// Read the 3-byte Scaleform/SWF signature from a buffer.
fn read_signature(data: &[u8]) -> Option<u32> {
    if data.len() < 4 {
        return None;
    }
    // Little-endian read, mask top byte
    Some(u32::from_le_bytes([data[0], data[1], data[2], 0]))
}

/// Returns `true` if the buffer begins with a recognised Scaleform/SWF header.
pub fn is_scaleform(data: &[u8]) -> bool {
    matches!(
        read_signature(data),
        Some(SWF_SIGNATURE | GFX_SIGNATURE | SWC_SIGNATURE | GFC_SIGNATURE)
    )
}

/// Returns `true` if the file extension is `.swf` or `.gfx` (case-insensitive).
pub fn is_scaleform_extension(filename: &str) -> bool {
    let ext = std::path::Path::new(filename)
        .extension()
        .map(|e| e.to_ascii_lowercase());
    matches!(ext.as_deref(), Some(e) if e == "swf" || e == "gfx")
}

/// Returns `true` if the header is already SWF (not GFX).
pub fn is_swf_header(data: &[u8]) -> bool {
    matches!(read_signature(data), Some(SWF_SIGNATURE | SWC_SIGNATURE))
}

/// Convert a GFX header to the equivalent SWF header.
///
/// `GFX` → `SWF`, `GFC` → `SWC`. If it's already SWF, returns as-is.
pub fn gfx_to_swf(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 4 {
        return Err(Error::InvalidFormat(
            "buffer too small for Scaleform header".into(),
        ));
    }
    let sig = read_signature(data).unwrap();
    let new_sig = match sig {
        GFX_SIGNATURE => SWF_SIGNATURE,
        GFC_SIGNATURE => SWC_SIGNATURE,
        SWF_SIGNATURE | SWC_SIGNATURE => sig, // already SWF
        _ => {
            return Err(Error::InvalidFormat(format!(
                "not a Scaleform file (sig=0x{sig:08X})"
            )));
        }
    };
    let new_bytes = new_sig.to_le_bytes();
    let mut out = data.to_vec();
    out[0] = new_bytes[0];
    out[1] = new_bytes[1];
    out[2] = new_bytes[2];
    Ok(out)
}

/// Convert a SWF header back to GFX.
///
/// `SWF` → `GFX`, `SWC` → `GFC`. If it's already GFX, returns as-is.
pub fn swf_to_gfx(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 4 {
        return Err(Error::InvalidFormat(
            "buffer too small for Scaleform header".into(),
        ));
    }
    let sig = read_signature(data).unwrap();
    let new_sig = match sig {
        SWF_SIGNATURE => GFX_SIGNATURE,
        SWC_SIGNATURE => GFC_SIGNATURE,
        GFX_SIGNATURE | GFC_SIGNATURE => sig,
        _ => {
            return Err(Error::InvalidFormat(format!(
                "not a SWF file (sig=0x{sig:08X})"
            )));
        }
    };
    let new_bytes = new_sig.to_le_bytes();
    let mut out = data.to_vec();
    out[0] = new_bytes[0];
    out[1] = new_bytes[1];
    out[2] = new_bytes[2];
    Ok(out)
}

/// Decompress a Scaleform file that has a compressed header.
///
/// Layout: `[4-byte sig][4-byte decompressed_size_LE][zlib data...]`
///
/// Returns the decompressed payload (without the 8-byte header).
pub fn decompress_scaleform(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 8 {
        return Err(Error::InvalidFormat(
            "buffer too small for compressed Scaleform".into(),
        ));
    }
    let sig = read_signature(data).unwrap();
    match sig {
        GFC_SIGNATURE | SWC_SIGNATURE => {}
        GFX_SIGNATURE | SWF_SIGNATURE => {
            // Not compressed — return as-is
            return Ok(data.to_vec());
        }
        _ => {
            return Err(Error::InvalidFormat(format!(
                "not a Scaleform file (sig=0x{sig:08X})"
            )));
        }
    }

    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let _decompressed_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let compressed = &data[8..];

    let mut decoder = ZlibDecoder::new(compressed);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| Error::Other(format!("zlib decompress failed: {e}")))?;
    Ok(out)
}
