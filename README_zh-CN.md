# Kitbash

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Aik2/kitbash.svg"/> <img src="https://img.shields.io/github/last-commit/Aik2/kitbash.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态: 🚧 早期开发中 (Early Development)

**Kitbash** — 专为游戏开发者设计的模块化 2D 角色拼装工具。

| English         | Simplified Chinese                 |
|-----------------|---------------------------------|
| [English](./README.md) | [简体中文](./README_zh-CN.md) |

## 简介

`Kitbash` 是一个运行在 Web 和桌面端的 2D 资产合成工具。  
它解决了分散的精灵部件（如头部、身体、武器）难以预览和组装的问题，允许用户导入素材、调整位置与缩放，并以像素完美的精度导出。

使用 `Kitbash`，你只需拖入素材，在画布上拼装，即可一键导出适合游戏引擎使用的图层或合成图。  
未来计划支持动画预览和精灵表（Spritesheet）生成。

## 功能特性

*   **批量导入**: 支持拖拽或文件选择器批量导入图片 (`PNG`, `JPG`, `WEBP`)。
*   **像素级拼装**:
    *   **最近邻缩放 (Nearest Neighbor)**: 保证像素画风格清晰锐利，绝无模糊。
    *   **像素对齐**: 整数坐标吸附，确保渲染精准。
    *   **交互式画布**: 支持中键拖拽平移视图、滚轮缩放。
*   **图层管理**:
    *   支持显示/隐藏切换。
    *   图层 Z轴 顺序调整。
    *   每个部件独立的位移与缩放控制。
*   **高级导出**:
    *   **导出倍率**: 支持 1.0x 到 10.0x 的高分辨率输出。
    *   **分层 ZIP 导出**: 生成包含所有图层的 `.zip` 包，每个图层均为全尺寸透明 PNG，可直接在 Godot/Unity 中重组。
    *   **元数据**: 附带 `data.json` 记录所有图层的原始变换数据。

## 使用指南

1.  **安装 Rust** (如果尚未安装):
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2.  **克隆仓库**:
    ```bash
    git clone https://github.com/Aik2/kitbash.git
    cd kitbash
    ```

3.  **本地运行 (原生桌面版)**:
    ```bash
    cargo run --release
    ```

4.  **本地运行 (Web/WASM 版)**:
    首先安装 Trunk 工具和 WASM 目标:
    ```bash
    cargo install --locked trunk
    rustup target add wasm32-unknown-unknown
    ```
    启动开发服务器:
    ```bash
    ./start_dev.sh
    # 或者
    trunk serve
    ```
    在浏览器访问 `http://localhost:8080`。

5.  **配置**:
    右侧面板提供了画布尺寸、背景颜色及导出倍率的设置选项。

## 构建指南

### 前置要求

*   Rust 1.75 或更高版本
*   `wasm32-unknown-unknown` 编译目标 (仅 Web 版需要)

### 构建步骤

1.  **克隆仓库**:
    ```bash
    git clone https://github.com/Aik2/kitbash.git
    cd kitbash
    ```

2.  **构建 (原生)**:
    ```bash
    cargo build --release
    ```

3.  **构建 (Web)**:
    ```bash
    trunk build --release
    ```
    构建产物位于 `dist/` 目录。

## 依赖项

本项目使用了以下核心 Crates:

| Crate                                             | Version | Description                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [eframe](https://crates.io/crates/eframe)         | 0.30    | GUI 框架 (基于 egui)        |
| [image](https://crates.io/crates/image)           | 0.25    | 图像处理核心                |
| [zip](https://crates.io/crates/zip)               | 0.6     | ZIP 文件生成 (WASM 兼容)    |
| [rfd](https://crates.io/crates/rfd)               | 0.15    | 原生/Web 文件对话框         |
| [wasm-bindgen](https://crates.io/crates/wasm-bindgen) | 0.2 | WASM JavaScript 绑定        |

## 贡献

欢迎提交贡献！
无论是修复 Bug、添加新功能还是改进文档，请随时：

*   提交 **Issue** 或 **Pull Request**。
*   分享你的想法或讨论设计方案。

## 许可证

本项目采用双协议授权：

*   Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) 或 [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
*   MIT license ([LICENSE-MIT](LICENSE-MIT) 或 [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

由你任选其一。
