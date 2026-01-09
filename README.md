# Kitbash

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Aik2/kitbash.svg"/> <img src="https://img.shields.io/github/last-commit/Aik2/kitbash.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development (Initial version in progress)

**Kitbash** — A modular 2D character assembly tool for game developers.

| English         | Simplified Chinese                 |
|-----------------|---------------------------------|
| [English](./README.md) | [简体中文](./README_zh-CN.md) |

## Introduction

`Kitbash` is a specialized 2D asset compositing tool designed for game developers.  
It solves the problem of assembling scattered sprite parts (like heads, bodies, weapons) into cohesive assets, allowing users to import, position, scale, and export them with pixel-perfect precision.

With `Kitbash`, you only need to drag and drop your assets, arrange them on the canvas, and export the result.  
In the future, it may also support animation preview and spritesheet generation.

## Features

*   **Batch Import**: Load multiple images at once (`PNG`, `JPG`, `WEBP`) via drag-and-drop or file dialog.
*   **Pixel-Perfect Assembly**: Nearest Neighbor Scaling ensures your pixel art remains crisp.
*   **Interactive Canvas**: Middle-mouse drag to pan, scroll to zoom, integer snapping for precision.
*   **Layer Management**: Visibility toggles, Z-Index reordering, and individual transform controls.
*   **Advanced Export**:
    *   **Export Scale**: Output at 1x to 10x resolution.
    *   **Scattered ZIP Export**: Downloads a `.zip` containing each layer as a separate, full-canvas-size PNG.
    *   **Metadata**: Includes a `data.json` preserving original transform values.

## How to Use

1.  **Install Rust** (if not already installed):
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2.  **Clone the repository**:
    ```bash
    git clone https://github.com/Aik2/kitbash.git
    cd kitbash
    ```

3.  **Run Locally (Native)**:
    ```bash
    cargo run --release
    ```

4.  **Run Locally (Web/WASM)**:
    Install Trunk:
    ```bash
    cargo install --locked trunk
    rustup target add wasm32-unknown-unknown
    ```
    Start server:
    ```bash
    ./start_dev.sh
    # or
    trunk serve
    ```
    Open `http://localhost:8080`.

5.  **Configuration**:
    The UI panel provides settings for Canvas Size, Background Color, and Export Scale.

## How to Build

### Prerequisites

*   Rust 1.75 or later
*   `wasm32-unknown-unknown` target (for Web)

### Build Steps

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/Aik2/kitbash.git
    cd kitbash
    ```

2.  **Build (Native)**:
    ```bash
    cargo build --release
    ```

3.  **Build (Web)**:
    ```bash
    trunk build --release
    ```

## Dependencies

This project uses the following crates:

| Crate                                             | Version | Description                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [eframe](https://crates.io/crates/eframe)         | 0.30    | GUI framework (egui wrapper)|
| [image](https://crates.io/crates/image)           | 0.25    | Image processing            |
| [zip](https://crates.io/crates/zip)               | 0.6     | ZIP file generation         |
| [rfd](https://crates.io/crates/rfd)               | 0.15    | Native/Web file dialogs     |
| [wasm-bindgen](https://crates.io/crates/wasm-bindgen) | 0.2 | WASM JavaScript bindings    |

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

*   Submit an **Issue** or **Pull Request**.
*   Share ideas and discuss design or architecture.

## License

This project is licensed under either of

*   Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
*   MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
