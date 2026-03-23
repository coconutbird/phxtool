# phxtool

A Rust reimplementation of [KornnerStudios' PhxTool](https://github.com/KornnerStudios/KSoft.Phoenix) for working with Halo Wars: Definitive Edition game assets.

Built on top of [ensemble-formats](https://github.com/coconutbird/ensemble-formats) and [pcktool-rs](https://github.com/coconutbird/pcktool-rs).

## Features

| Format    | Operations                                  | Description                                          |
| --------- | ------------------------------------------- | ---------------------------------------------------- |
| **ERA**   | expand, build, list, info, decrypt, encrypt | Game archives with automatic XMB↔XML conversion      |
| **XMB**   | to-xml, to-xmb, info                        | Binary XML format used throughout Halo Wars          |
| **UGX**   | info, to-gltf, from-gltf                    | 3D model format with glTF import/export              |
| **Wwise** | info, list, dump                            | PCK/BNK audio packages (sound banks, streaming WEMs) |

## Building

Requires Rust 2024 edition (1.85+).

```sh
cargo build --release
```

The binary is output to `target/release/phxtool`.

## Usage

```
phxtool <COMMAND>

Commands:
  era    ERA archive operations
  xmb    XMB ↔ XML conversion
  ugx    UGX model operations
  wwise  Wwise audio operations
```

### ERA Archives

```sh
# Extract an ERA archive (auto-converts XMB → XML)
phxtool era expand game.era

# Rebuild from a directory (auto-converts XML → XMB, encrypts)
phxtool era build extracted_dir/ -o game.era

# List contents
phxtool era list game.era

# Show metadata
phxtool era info game.era

# Decrypt / encrypt standalone
phxtool era decrypt encrypted.era -o decrypted.era
phxtool era encrypt decrypted.era -o encrypted.era
```

### XMB / XML Conversion

```sh
# Convert binary XMB to readable XML
phxtool xmb to-xml data.xmb

# Convert XML back to XMB (PC format)
phxtool xmb to-xmb data.xml

# Show XMB metadata
phxtool xmb info data.xmb
```

### UGX Models

```sh
# Show model info (meshes, vertices, materials)
phxtool ugx info model.ugx

# Export to glTF
phxtool ugx to-gltf model.ugx -o model.gltf

# Import from glTF
phxtool ugx from-gltf model.gltf -o model.ugx
```

### Wwise Audio

```sh
# Show PCK/BNK metadata (languages, entry counts)
phxtool wwise info sounds.pck

# List sound banks (or --streaming / --external)
phxtool wwise list sounds.pck
phxtool wwise list sounds.pck --streaming

# Extract all audio files (organized by language)
phxtool wwise dump sounds.pck

# Extract to a specific directory
phxtool wwise dump sounds.pck -o my_output/

# Extract a single source by ID
phxtool wwise dump sounds.pck --id 0x1A2B3C4D

# Extract embedded WEM from a standalone BNK
phxtool wwise dump soundbank.bnk
```

## Project Structure

```
crates/
├── phxtool/          # Library — high-level operations
│   ├── era_ops.rs    # ERA archive workflows
│   ├── xmb_ops.rs    # XMB ↔ XML conversion
│   ├── ugx_ops.rs    # UGX model operations
│   ├── wwise_ops.rs  # Wwise PCK/BNK operations
│   └── error.rs      # Unified error type
└── phxtool-cli/      # CLI binary
    ├── main.rs        # Entry point
    ├── cmd_era.rs     # ERA subcommands
    ├── cmd_xmb.rs     # XMB subcommands
    ├── cmd_ugx.rs     # UGX subcommands
    └── cmd_wwise.rs   # Wwise subcommands
```

## Acknowledgments

- [KornnerStudios/KSoft.Phoenix](https://github.com/KornnerStudios/KSoft.Phoenix) — the original C# implementation
- [ensemble-formats](https://github.com/coconutbird/ensemble-formats) — low-level Ensemble Studios format parsers
- [pcktool-rs](https://github.com/coconutbird/pcktool-rs) — Wwise PCK/BNK parser

## License

MIT
