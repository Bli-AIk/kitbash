// Built-in node implementations that ship with kitbash.
//
// Each built-in node also provides a `manifest()` method that returns the same
// `NodeManifest` a WASM plugin would — eat our own dogfood.

use super::graph::{NodeInfo, NodeProcessor, ParamInfo, PortInfo};
use super::types::{Color, NodeImage, PortType, PortValue};
use crate::plugin::abi::{NodeManifest, ParamDecl, PortDecl, PortDirection};

// ─── Image Input ────────────────────────────────────────────────────

/// Outputs a user-loaded image. The actual image data is set via params.
pub struct ImageInputNode;

impl NodeProcessor for ImageInputNode {
    fn info(&self) -> NodeInfo {
        NodeInfo {
            name: "Image Input".into(),
            category: "Input".into(),
            inputs: vec![],
            outputs: vec![PortInfo {
                name: "image".into(),
                port_type: PortType::Image,
            }],
            params: vec![],
        }
    }

    fn process(&self, _inputs: &[Option<PortValue>], params: &[PortValue]) -> Vec<PortValue> {
        let image = params
            .first()
            .and_then(|v| match v {
                PortValue::Image(img) => Some(img.clone()),
                _ => None,
            })
            .unwrap_or_else(|| NodeImage::new(1, 1));
        vec![PortValue::Image(image)]
    }
}

// ─── Sprite Output ──────────────────────────────────────────────────

/// Terminal node that receives the final processed image.
pub struct SpriteOutputNode;

impl NodeProcessor for SpriteOutputNode {
    fn info(&self) -> NodeInfo {
        NodeInfo {
            name: "Sprite Output".into(),
            category: "Output".into(),
            inputs: vec![PortInfo {
                name: "image".into(),
                port_type: PortType::Image,
            }],
            outputs: vec![],
            params: vec![],
        }
    }

    fn process(&self, inputs: &[Option<PortValue>], _params: &[PortValue]) -> Vec<PortValue> {
        if let Some(Some(PortValue::Image(_img))) = inputs.first() {
            // Output node doesn't produce outputs; the canvas reads from the cache
        }
        vec![]
    }
}

// ─── Replace Color ──────────────────────────────────────────────────

pub struct ReplaceColorNode;

impl NodeProcessor for ReplaceColorNode {
    fn info(&self) -> NodeInfo {
        NodeInfo {
            name: "Replace Color".into(),
            category: "Color".into(),
            inputs: vec![PortInfo {
                name: "image".into(),
                port_type: PortType::Image,
            }],
            outputs: vec![PortInfo {
                name: "image".into(),
                port_type: PortType::Image,
            }],
            params: vec![
                ParamInfo {
                    name: "from_color".into(),
                    port_type: PortType::Color,
                    default_json: None,
                },
                ParamInfo {
                    name: "to_color".into(),
                    port_type: PortType::Color,
                    default_json: None,
                },
                ParamInfo {
                    name: "tolerance".into(),
                    port_type: PortType::Float,
                    default_json: Some("0.0".into()),
                },
            ],
        }
    }

    fn process(&self, inputs: &[Option<PortValue>], params: &[PortValue]) -> Vec<PortValue> {
        let Some(Some(PortValue::Image(src))) = inputs.first() else {
            return vec![PortValue::Image(NodeImage::new(1, 1))];
        };

        let from = extract_color(params, 0);
        let to = extract_color(params, 1);
        let tolerance = extract_float(params, 2);

        let mut out = src.clone();
        for chunk in out.pixels.chunks_exact_mut(4) {
            let pixel_color = Color {
                r: chunk[0],
                g: chunk[1],
                b: chunk[2],
                a: chunk[3],
            };
            if color_distance(&pixel_color, &from) <= tolerance {
                chunk[0] = to.r;
                chunk[1] = to.g;
                chunk[2] = to.b;
                chunk[3] = to.a;
            }
        }

        vec![PortValue::Image(out)]
    }
}

// ─── Grid Slice ─────────────────────────────────────────────────────

pub struct GridSliceNode;

impl NodeProcessor for GridSliceNode {
    fn info(&self) -> NodeInfo {
        NodeInfo {
            name: "Grid Slice".into(),
            category: "Sprite".into(),
            inputs: vec![PortInfo {
                name: "image".into(),
                port_type: PortType::Image,
            }],
            outputs: vec![PortInfo {
                name: "sprites".into(),
                port_type: PortType::SpriteList,
            }],
            params: vec![
                ParamInfo {
                    name: "cell_width".into(),
                    port_type: PortType::Int,
                    default_json: Some("16".into()),
                },
                ParamInfo {
                    name: "cell_height".into(),
                    port_type: PortType::Int,
                    default_json: Some("16".into()),
                },
            ],
        }
    }

    fn process(&self, inputs: &[Option<PortValue>], params: &[PortValue]) -> Vec<PortValue> {
        let Some(Some(PortValue::Image(src))) = inputs.first() else {
            return vec![PortValue::SpriteList(super::types::SpriteList::default())];
        };

        let cell_w = extract_int(params, 0).max(1) as u32;
        let cell_h = extract_int(params, 1).max(1) as u32;

        let cols = src.width / cell_w;
        let rows = src.height / cell_h;

        let mut sprites = Vec::new();
        for row in 0..rows {
            for col in 0..cols {
                sprites.push(extract_cell(src, col, row, cell_w, cell_h));
            }
        }

        vec![PortValue::SpriteList(super::types::SpriteList { sprites })]
    }
}

fn extract_cell(src: &NodeImage, col: u32, row: u32, cell_w: u32, cell_h: u32) -> NodeImage {
    let mut cell = NodeImage::new(cell_w, cell_h);
    for y in 0..cell_h {
        for x in 0..cell_w {
            let src_x = col * cell_w + x;
            let src_y = row * cell_h + y;
            let src_idx = ((src_y * src.width + src_x) * 4) as usize;
            let dst_idx = ((y * cell_w + x) * 4) as usize;
            if src_idx + 3 < src.pixels.len() {
                cell.pixels[dst_idx..dst_idx + 4]
                    .copy_from_slice(&src.pixels[src_idx..src_idx + 4]);
            }
        }
    }
    cell
}

// ─── Color Detect ───────────────────────────────────────────────────

pub struct ColorDetectNode;

impl NodeProcessor for ColorDetectNode {
    fn info(&self) -> NodeInfo {
        NodeInfo {
            name: "Color Detect".into(),
            category: "Color".into(),
            inputs: vec![PortInfo {
                name: "image".into(),
                port_type: PortType::Image,
            }],
            outputs: vec![PortInfo {
                name: "palette".into(),
                port_type: PortType::Palette,
            }],
            params: vec![],
        }
    }

    fn process(&self, inputs: &[Option<PortValue>], _params: &[PortValue]) -> Vec<PortValue> {
        let Some(Some(PortValue::Image(src))) = inputs.first() else {
            return vec![PortValue::Palette(super::types::Palette::default())];
        };

        let mut unique: Vec<Color> = Vec::new();
        for chunk in src.pixels.chunks_exact(4) {
            let c = Color {
                r: chunk[0],
                g: chunk[1],
                b: chunk[2],
                a: chunk[3],
            };
            if !unique.contains(&c) {
                unique.push(c);
            }
        }

        vec![PortValue::Palette(super::types::Palette { colors: unique })]
    }
}

// ─── Helpers ────────────────────────────────────────────────────────

fn extract_color(params: &[PortValue], idx: usize) -> Color {
    params
        .get(idx)
        .and_then(|v| match v {
            PortValue::Color(c) => Some(c.clone()),
            _ => None,
        })
        .unwrap_or(Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        })
}

fn extract_float(params: &[PortValue], idx: usize) -> f32 {
    params
        .get(idx)
        .and_then(|v| match v {
            PortValue::Float(f) => Some(*f),
            _ => None,
        })
        .unwrap_or(0.0)
}

fn extract_int(params: &[PortValue], idx: usize) -> i32 {
    params
        .get(idx)
        .and_then(|v| match v {
            PortValue::Int(i) => Some(*i),
            _ => None,
        })
        .unwrap_or(0)
}

fn color_distance(a: &Color, b: &Color) -> f32 {
    let dr = a.r as f32 - b.r as f32;
    let dg = a.g as f32 - b.g as f32;
    let db = a.b as f32 - b.b as f32;
    let da = a.a as f32 - b.a as f32;
    (dr * dr + dg * dg + db * db + da * da).sqrt()
}

// ─── Manifest Generation ────────────────────────────────────────────

/// Convert a `NodeInfo` into a `NodeManifest` (shared by all built-in nodes).
pub fn node_info_to_manifest(info: &NodeInfo) -> NodeManifest {
    let mut ports = Vec::new();
    for p in &info.inputs {
        ports.push(PortDecl {
            name: p.name.clone(),
            direction: PortDirection::Input,
            port_type: p.port_type.to_string(),
        });
    }
    for p in &info.outputs {
        ports.push(PortDecl {
            name: p.name.clone(),
            direction: PortDirection::Output,
            port_type: p.port_type.to_string(),
        });
    }
    let params = info
        .params
        .iter()
        .map(|pi| ParamDecl {
            name: pi.name.clone(),
            param_type: pi.port_type.to_string(),
            default_json: pi.default_json.clone().unwrap_or_default(),
        })
        .collect();
    NodeManifest {
        id: info.name.to_lowercase().replace(' ', "_"),
        display_name: info.name.clone(),
        category: info.category.clone(),
        ports,
        params,
    }
}
