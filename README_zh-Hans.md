# Kitbash

[![license](https://img.shields.io/badge/license-GPLv3-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/kitbash.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/kitbash.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态: 🚧 早期开发中（架构重构进行中）

**Kitbash** — 基于节点图的 2D 精灵合成工具，支持 WASM 插件扩展。

| English         | Simplified Chinese                 |
|-----------------|---------------------------------|
| [English](./README.md) | 简体中文 |

## 简介

`Kitbash` 是一个基于节点图的 2D 资产合成工具，专为游戏开发者设计。
它通过可视化节点图管线，解决分散精灵部件（如头部、身体、武器）的组装问题。

**核心理念**：所有图像处理操作均表达为有向图中的节点。内置操作（颜色替换、网格切割、合成）
与第三方扩展使用相同的 WASM 插件 ABI。

## 功能特性

* **节点图管线**: 在可视化 DAG 中连接处理节点（颜色替换、网格切割、合成等）。
* **固定 I/O 节点**: `Image Input` 和 `Sprite Output` 为永久节点——始终存在，不可删除。
* **CanvasLayout 节点**: 交互式多图层画布，支持拖拽定位，作为节点图中的节点。
* **批量导入**: 支持拖拽或文件选择器批量导入图片 (`PNG`, `JPG`, `WEBP`)。
* **像素完美渲染**: 最近邻缩放确保像素画清晰锐利。
* **WASM 插件系统**: 通过编译为 WebAssembly 的自定义处理节点进行扩展。
* **面板布局**: 可调整大小和拖拽的面板布局（节点图、预览、检查器、控制台、设置）。
* **CJK 字体支持**: 内嵌思源黑体，支持中日韩文字渲染。
* **主题系统**: 受 Rerun 启发的深色主题，支持亮度调节。
* **跨平台**: 支持原生桌面（Linux/macOS/Windows）和浏览器（WASM）运行。

## 使用指南

1. **安装 Rust**（如未安装）:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **克隆仓库**:
   ```bash
   git clone https://github.com/Bli-AIk/kitbash.git
   cd kitbash
   ```

3. **本地运行（原生桌面版）**:
   ```bash
   cargo run --release
   ```

4. **本地运行（Web/WASM 版）**:
   ```bash
   cargo install --locked trunk
   rustup target add wasm32-unknown-unknown
   trunk serve
   ```
   在浏览器访问 `http://localhost:8080`。

5. **使用 just**（如已安装 [just](https://github.com/casey/just)）:
   ```bash
   just run          # 原生 debug 模式
   just run-release  # 原生 release 模式
   just web          # WASM 开发服务器
   just check        # clippy + tokei 检查
   ```

## 构建指南

### 前置要求

* Rust 1.75 或更高版本
* `wasm32-unknown-unknown` 编译目标（仅 Web 版需要）

### 构建步骤

```bash
git clone https://github.com/Bli-AIk/kitbash.git
cd kitbash
cargo build --release          # 原生
trunk build --release          # Web/WASM
```

## 依赖项

| Crate | 版本 | 说明 |
|-------|------|------|
| [eframe](https://crates.io/crates/eframe) | 0.33 | GUI 框架 (egui) |
| [egui_tiles](https://crates.io/crates/egui_tiles) | 0.14 | 面板布局系统 |
| [egui-snarl](https://crates.io/crates/egui-snarl) | 0.9 | 节点图编辑器 |
| [catppuccin-egui](https://crates.io/crates/catppuccin-egui) | 5.7 | 主题预设 |
| [image](https://crates.io/crates/image) | 0.25 | 图像处理 |
| [zip](https://crates.io/crates/zip) | 0.6 | ZIP 生成 |
| [rfd](https://crates.io/crates/rfd) | 0.15 | 文件对话框 |
| [toml](https://crates.io/crates/toml) | 0.8 | 配置文件解析 |

## 贡献

欢迎提交贡献！
无论是修复 Bug、添加新功能还是改进文档，请随时：

* 提交 **Issue** 或 **Pull Request**。
* 分享你的想法或讨论设计方案。

## 许可证

本项目采用 [GNU 通用公共许可证 v3.0](LICENSE) 或更高版本授权。
