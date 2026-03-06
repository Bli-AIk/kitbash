# Kitbash

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/kitbash.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/kitbash.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development (Architecture refactor in progress)

**Kitbash** — A node-based 2D sprite compositing tool with WASM plugin extensibility.

| English         | Simplified Chinese                 |
|-----------------|---------------------------------|
| English | [简体中文](./README_zh-Hans.md) |

## Introduction

`Kitbash` is a node-based 2D asset compositing tool designed for game developers.
It solves the problem of assembling scattered sprite parts (like heads, bodies, weapons)
into cohesive assets through a visual node graph pipeline.

**Core concept**: All image processing operations are expressed as nodes in a directed
graph. Built-in operations (color replacement, grid slicing, compositing) use the same
WASM plugin ABI as third-party extensions.

## Features

* **Node Graph Pipeline**: Connect processing nodes (color replace, grid slice, composite, etc.) in a visual DAG.
* **Fixed I/O Nodes**: `Image Input` and `Sprite Output` are permanent nodes — always present, never deletable.
* **CanvasLayout Node**: Interactive multi-layer canvas with drag-to-reposition, acting as a node in the graph.
* **Batch Import**: Load multiple images at once (`PNG`, `JPG`, `WEBP`) via drag-and-drop or file dialog.
* **Pixel-Perfect Rendering**: Nearest-neighbor scaling for crisp pixel art.
* **WASM Plugin System**: Extend with custom processing nodes compiled to WebAssembly.
* **Dock Layout**: Resizable, draggable panel layout (node graph, preview, inspector, console, settings).
* **CJK Font Support**: Embedded Source Han Sans for Chinese/Japanese/Korean text rendering.
* **Theme System**: Rerun-inspired dark theme with brightness controls.
* **Cross-Platform**: Runs natively (Linux/macOS/Windows) and in the browser (WASM).

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone the repository**:
   ```bash
   git clone https://github.com/Bli-AIk/kitbash.git
   cd kitbash
   ```

3. **Run Locally (Native)**:
   ```bash
   cargo run --release
   ```

4. **Run Locally (Web/WASM)**:
   ```bash
   cargo install --locked trunk
   rustup target add wasm32-unknown-unknown
   trunk serve
   ```
   Open `http://localhost:8080`.

5. **Using just** (if [just](https://github.com/casey/just) is installed):
   ```bash
   just run          # Native debug
   just run-release  # Native release
   just web          # WASM dev server
   just check        # Clippy + tokei checks
   ```

## How to Build

### Prerequisites

* Rust 1.75 or later
* `wasm32-unknown-unknown` target (for Web)

### Build Steps

```bash
git clone https://github.com/Bli-AIk/kitbash.git
cd kitbash
cargo build --release          # Native
trunk build --release          # Web/WASM
```

## Architecture

```
src/
├── app.rs              # Core application state + ImageStore
├── theme.rs / theme/   # Rerun-inspired dark theme
├── ui/
│   ├── dock.rs              # egui_tiles dock layout
│   ├── menu_bar.rs          # Top menu bar
│   ├── node_graph_panel.rs  # egui-snarl node graph editor
│   ├── canvas_panel.rs      # Preview / interactive canvas
│   ├── inspector_panel.rs   # Context-aware property inspector
│   ├── settings_panel.rs    # Theme / font / scale settings
│   └── console_panel.rs     # Log output
├── node/
│   ├── graph.rs        # DAG engine + execution
│   ├── types.rs        # Data types (Image, Palette, ScatteredPack, ...)
│   ├── builtin.rs      # Built-in node implementations
│   └── scheduler.rs    # Execution scheduling
├── plugin/
│   ├── abi.rs          # WASM ABI (NodeManifest, encode/decode)
│   ├── runtime.rs      # WASM runtime wrapper
│   └── loader.rs       # Plugin discovery + loading
├── imaging/
│   ├── canvas.rs       # Image compositing
│   └── export.rs       # PNG / ZIP export
├── font.rs             # CJK font management
└── config.rs           # Persistent configuration
```

## Dependencies

| Crate | Version | Description |
|-------|---------|-------------|
| [eframe](https://crates.io/crates/eframe) | 0.33 | GUI framework (egui) |
| [egui_tiles](https://crates.io/crates/egui_tiles) | 0.14 | Dock layout system |
| [egui-snarl](https://crates.io/crates/egui-snarl) | 0.9 | Node graph editor |
| [catppuccin-egui](https://crates.io/crates/catppuccin-egui) | 5.7 | Theme presets |
| [image](https://crates.io/crates/image) | 0.25 | Image processing |
| [zip](https://crates.io/crates/zip) | 0.6 | ZIP generation |
| [rfd](https://crates.io/crates/rfd) | 0.15 | File dialogs |
| [toml](https://crates.io/crates/toml) | 0.8 | Config file parsing |

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.

## License

This project is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
  or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
