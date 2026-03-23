# phxtool

A Rust reimplementation of [KornnerStudios' PhxTool](https://github.com/KornnerStudios/KSoft.Phoenix) for working with Halo Wars: Definitive Edition game assets.

Built on top of [ensemble-formats](https://github.com/coconutbird/ensemble-formats) and [pcktool-rs](https://github.com/coconutbird/pcktool-rs).

## Features

| Format    | Operations                                  | Description                                          |
| --------- | ------------------------------------------- | ---------------------------------------------------- |
| **ERA**   | expand, build, list, info, decrypt, encrypt | Game archives with automatic XMB↔XML conversion      |
| **XMB**   | to-xml, to-xmb, info, batch                 | Binary XML format used throughout Halo Wars          |
| **UGX**   | info, to-gltf, from-gltf                    | 3D model format with glTF import/export              |
| **Wwise** | info, list, dump                            | PCK/BNK audio packages (sound banks, streaming WEMs) |
| **ECF**   | info, expand, build                         | Generic container format (terrain, etc.)             |
| **BDT**   | info, to-xml, to-bdt                        | BinaryDataTree format (.vis files)                   |
| **GFX**   | info, to-swf, to-gfx, decompress            | Scaleform UI files (GFX↔SWF conversion)              |

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
  ecf    ECF container operations
  bdt    BinaryDataTree operations
  gfx    Scaleform GFX ↔ SWF operations
```

### ERA Archives

```sh
# Extract an ERA archive (auto-converts XMB → XML)
phxtool era expand game.era

# Extract with Scaleform processing
phxtool era expand game.era --gfx-to-swf        # Convert GFX → SWF
phxtool era expand game.era --decompress-ui      # Decompress Scaleform files

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

# Batch convert all XMB/XML files in a directory tree
phxtool xmb batch ./extracted_data/

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

### ECF Containers

```sh
# Show ECF container info (chunks, sizes)
phxtool ecf info terrain.xtd

# Extract all chunks from an ECF file
phxtool ecf expand terrain.xtd

# Rebuild an ECF from extracted chunks
phxtool ecf build chunks_dir/ -o terrain.xtd
```

### BinaryDataTree

```sh
# Convert BDT to XML
phxtool bdt to-xml data.vis

# Convert XML back to BDT
phxtool bdt to-bdt data.xml

# Show BDT info
phxtool bdt info data.vis
```

### Scaleform GFX

```sh
# Convert GFX to SWF (for editing in JPEXS/FFDec)
phxtool gfx to-swf ui_file.gfx

# Convert SWF back to GFX
phxtool gfx to-gfx ui_file.swf

# Decompress a compressed Scaleform file
phxtool gfx decompress ui_file.gfx

# Show Scaleform file info
phxtool gfx info ui_file.gfx
```

## Project Structure

```
crates/
├── phxtool/              # Library — high-level operations
│   └── src/ops/
│       ├── era.rs        # ERA archive workflows
│       ├── xmb.rs        # XMB ↔ XML conversion
│       ├── ugx.rs        # UGX model operations
│       ├── wwise.rs      # Wwise PCK/BNK operations
│       ├── ecf.rs        # ECF container operations
│       ├── bdt.rs        # BinaryDataTree operations
│       ├── scaleform.rs  # Scaleform GFX↔SWF helpers
│       └── util.rs       # Shared utilities
└── phxtool-cli/          # CLI binary
    └── src/commands/
        ├── era.rs        # ERA subcommands
        ├── xmb.rs        # XMB subcommands
        ├── ugx.rs        # UGX subcommands
        ├── wwise.rs      # Wwise subcommands
        ├── ecf.rs        # ECF subcommands
        ├── bdt.rs        # BDT subcommands
        └── gfx.rs        # GFX subcommands
```

## Acknowledgments

- [KornnerStudios/KSoft.Phoenix](https://github.com/KornnerStudios/KSoft.Phoenix) — the original C# implementation
- [ensemble-formats](https://github.com/coconutbird/ensemble-formats) — low-level Ensemble Studios format parsers
- [pcktool-rs](https://github.com/coconutbird/pcktool-rs) — Wwise PCK/BNK parser

## License

MIT
