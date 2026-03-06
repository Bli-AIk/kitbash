// Snarl graph executor — bridges the UI node graph to actual processing.
//
// Walks the egui-snarl connections in topological order, executes each node's
// processing logic, and caches results for use by the preview panel.

use std::collections::{HashMap, HashSet, VecDeque};

use egui_snarl::{InPinId, NodeId, Snarl};

use crate::app::ImageStore;
use crate::ui::node_graph_panel::KitbashNode;

use super::types::{NodeImage, PortValue};

/// Cached execution results, keyed by `(NodeId, output_port_index)`.
pub type ExecutionCache = HashMap<(NodeId, usize), PortValue>;

/// Execute the entire snarl graph and return cached outputs.
pub fn execute_snarl(snarl: &Snarl<KitbashNode>, image_store: &ImageStore) -> ExecutionCache {
    let order = topological_sort_snarl(snarl);
    let mut cache = ExecutionCache::new();

    for node_id in order {
        let Some(node_info) = snarl.get_node_info(node_id) else {
            continue;
        };
        let node = &node_info.value;
        let outputs = execute_node(node, node_id, snarl, image_store, &cache);
        for (port_idx, value) in outputs.into_iter().enumerate() {
            cache.insert((node_id, port_idx), value);
        }
    }

    cache
}

/// Execute a single node, gathering inputs from upstream cache.
fn execute_node(
    node: &KitbashNode,
    node_id: NodeId,
    snarl: &Snarl<KitbashNode>,
    image_store: &ImageStore,
    cache: &ExecutionCache,
) -> Vec<PortValue> {
    match node {
        KitbashNode::ImageInput { image_ids } => execute_image_input(image_ids, image_store),
        KitbashNode::SpriteOutput { .. } => execute_passthrough(node_id, snarl, cache, 1),
        KitbashNode::ReplaceColor { from, to } => {
            let inputs = gather_inputs(node_id, snarl, cache, node.input_count());
            execute_replace_color(&inputs, from, to)
        }
        KitbashNode::GridSlice { cols, rows } => {
            let inputs = gather_inputs(node_id, snarl, cache, node.input_count());
            execute_grid_slice(&inputs, *cols, *rows)
        }
        KitbashNode::ColorDetect { .. } => {
            let inputs = gather_inputs(node_id, snarl, cache, node.input_count());
            execute_color_detect(&inputs)
        }
        KitbashNode::Composite { num_inputs, .. } => {
            let inputs = gather_inputs(node_id, snarl, cache, *num_inputs);
            execute_composite(&inputs)
        }
        KitbashNode::Transform { offset, scale } => {
            let inputs = gather_inputs(node_id, snarl, cache, node.input_count());
            execute_transform(&inputs, offset, *scale)
        }
        KitbashNode::CanvasLayout {
            canvas_size,
            layer_offsets,
            layer_scales,
            num_inputs,
            ..
        } => {
            let inputs = gather_inputs(node_id, snarl, cache, *num_inputs);
            execute_canvas_layout(&inputs, canvas_size, layer_offsets, layer_scales)
        }
    }
}

/// Gather input values from upstream connections.
fn gather_inputs(
    node_id: NodeId,
    snarl: &Snarl<KitbashNode>,
    cache: &ExecutionCache,
    input_count: usize,
) -> Vec<Option<PortValue>> {
    (0..input_count)
        .map(|input_idx| {
            let in_pin = snarl.in_pin(InPinId {
                node: node_id,
                input: input_idx,
            });
            in_pin
                .remotes
                .first()
                .and_then(|remote| cache.get(&(remote.node, remote.output)).cloned())
        })
        .collect()
}

// ─── Node executors ─────────────────────────────────────────────────

fn execute_image_input(image_ids: &[u64], image_store: &ImageStore) -> Vec<PortValue> {
    image_ids
        .iter()
        .map(|&id| {
            image_store
                .images
                .iter()
                .find(|img| img.id == id)
                .map(|img| {
                    let rgba = img.image.to_rgba8();
                    PortValue::Image(NodeImage::from_rgba(
                        rgba.width(),
                        rgba.height(),
                        rgba.into_raw(),
                    ))
                })
                .unwrap_or_else(|| PortValue::Image(NodeImage::new(1, 1)))
        })
        .collect()
}

fn execute_passthrough(
    node_id: NodeId,
    snarl: &Snarl<KitbashNode>,
    cache: &ExecutionCache,
    input_count: usize,
) -> Vec<PortValue> {
    // SpriteOutput just passes through — no outputs, but we store the input for preview
    let inputs = gather_inputs(node_id, snarl, cache, input_count);
    inputs.into_iter().flatten().collect()
}

fn execute_replace_color(
    inputs: &[Option<PortValue>],
    from: &[u8; 4],
    to: &[u8; 4],
) -> Vec<PortValue> {
    let Some(Some(PortValue::Image(src))) = inputs.first() else {
        return vec![PortValue::Image(NodeImage::new(1, 1))];
    };
    let mut out = src.clone();
    for chunk in out.pixels.chunks_exact_mut(4) {
        if chunk[0] == from[0] && chunk[1] == from[1] && chunk[2] == from[2] && chunk[3] == from[3]
        {
            chunk.copy_from_slice(to);
        }
    }
    vec![PortValue::Image(out)]
}

fn execute_grid_slice(inputs: &[Option<PortValue>], cols: u32, rows: u32) -> Vec<PortValue> {
    let Some(Some(PortValue::Image(src))) = inputs.first() else {
        return vec![PortValue::SpriteList(super::types::SpriteList::default())];
    };
    let cell_w = (src.width / cols.max(1)).max(1);
    let cell_h = (src.height / rows.max(1)).max(1);
    let mut sprites = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            sprites.push(extract_grid_cell(src, col, row, cell_w, cell_h));
        }
    }
    vec![PortValue::SpriteList(super::types::SpriteList { sprites })]
}

fn extract_grid_cell(src: &NodeImage, col: u32, row: u32, cell_w: u32, cell_h: u32) -> NodeImage {
    let mut cell = NodeImage::new(cell_w, cell_h);
    for y in 0..cell_h {
        for x in 0..cell_w {
            let sx = col * cell_w + x;
            let sy = row * cell_h + y;
            let si = ((sy * src.width + sx) * 4) as usize;
            let di = ((y * cell_w + x) * 4) as usize;
            if si + 3 < src.pixels.len() {
                cell.pixels[di..di + 4].copy_from_slice(&src.pixels[si..si + 4]);
            }
        }
    }
    cell
}

fn execute_color_detect(inputs: &[Option<PortValue>]) -> Vec<PortValue> {
    let Some(Some(PortValue::Image(src))) = inputs.first() else {
        return vec![PortValue::Palette(super::types::Palette::default())];
    };
    let mut unique = Vec::new();
    for chunk in src.pixels.chunks_exact(4) {
        let c = super::types::Color {
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

fn execute_composite(inputs: &[Option<PortValue>]) -> Vec<PortValue> {
    // Simple alpha-over compositing of all input images
    let images: Vec<&NodeImage> = inputs
        .iter()
        .filter_map(|i| i.as_ref())
        .filter_map(|v| match v {
            PortValue::Image(img) => Some(img),
            _ => None,
        })
        .collect();

    if images.is_empty() {
        return vec![PortValue::Image(NodeImage::new(1, 1))];
    }

    let w = images.iter().map(|i| i.width).max().unwrap_or(1);
    let h = images.iter().map(|i| i.height).max().unwrap_or(1);
    let mut result = NodeImage::new(w, h);

    for img in &images {
        alpha_blit(&mut result, img, 0, 0);
    }

    vec![PortValue::Image(result)]
}

fn execute_transform(
    inputs: &[Option<PortValue>],
    _offset: &[f32; 2],
    _scale: f32,
) -> Vec<PortValue> {
    // Passthrough for now — transform rendering handled at preview level
    let Some(Some(PortValue::Image(src))) = inputs.first() else {
        return vec![PortValue::Image(NodeImage::new(1, 1))];
    };
    vec![PortValue::Image(src.clone())]
}

fn execute_canvas_layout(
    inputs: &[Option<PortValue>],
    canvas_size: &[u32; 2],
    offsets: &[[f32; 2]],
    scales: &[f32],
) -> Vec<PortValue> {
    let mut result = NodeImage::new(canvas_size[0], canvas_size[1]);

    for (i, input) in inputs.iter().enumerate() {
        let Some(PortValue::Image(img)) = input else {
            continue;
        };
        let ox = offsets.get(i).map(|o| o[0]).unwrap_or(0.0) as i32;
        let oy = offsets.get(i).map(|o| o[1]).unwrap_or(0.0) as i32;
        let _scale = scales.get(i).copied().unwrap_or(1.0);
        alpha_blit(&mut result, img, ox, oy);
    }

    // First output is composed image, second is scattered pack (stub)
    vec![
        PortValue::Image(result),
        PortValue::ScatteredPack(super::types::ScatteredPack::default()),
    ]
}

/// Alpha-over blit `src` onto `dst` at pixel offset `(ox, oy)`.
fn alpha_blit(dst: &mut NodeImage, src: &NodeImage, ox: i32, oy: i32) {
    for sy in 0..src.height {
        let dy = oy + sy as i32;
        if dy < 0 || dy >= dst.height as i32 {
            continue;
        }
        for sx in 0..src.width {
            let dx = ox + sx as i32;
            if dx < 0 || dx >= dst.width as i32 {
                continue;
            }
            let si = ((sy * src.width + sx) * 4) as usize;
            let di = ((dy as u32 * dst.width + dx as u32) * 4) as usize;
            blend_pixel(&mut dst.pixels, di, &src.pixels, si);
        }
    }
}

fn blend_pixel(dst: &mut [u8], di: usize, src: &[u8], si: usize) {
    let sa = src[si + 3] as f32 / 255.0;
    if sa <= 0.0 {
        return;
    }
    let da = dst[di + 3] as f32 / 255.0;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        return;
    }
    for c in 0..3 {
        let sc = src[si + c] as f32;
        let dc = dst[di + c] as f32;
        dst[di + c] = ((sc * sa + dc * da * (1.0 - sa)) / out_a) as u8;
    }
    dst[di + 3] = (out_a * 255.0) as u8;
}

// ─── Topological sort ───────────────────────────────────────────────

fn topological_sort_snarl(snarl: &Snarl<KitbashNode>) -> Vec<NodeId> {
    let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
    let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    // Collect all node IDs
    for (node_id, _) in snarl.node_ids() {
        in_degree.entry(node_id).or_insert(0);
        adjacency.entry(node_id).or_default();
    }

    // Build adjacency from connections
    for (node_id, _) in snarl.node_ids() {
        let node = &snarl[node_id];
        let input_count = node.input_count();
        for input_idx in 0..input_count {
            let in_pin = snarl.in_pin(InPinId {
                node: node_id,
                input: input_idx,
            });
            for remote in &in_pin.remotes {
                adjacency.entry(remote.node).or_default().push(node_id);
                *in_degree.entry(node_id).or_insert(0) += 1;
            }
        }
    }

    let mut queue: VecDeque<NodeId> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut result = Vec::new();
    let mut visited = HashSet::new();

    while let Some(id) = queue.pop_front() {
        if !visited.insert(id) {
            continue;
        }
        result.push(id);
        decrement_neighbors(&adjacency, &mut in_degree, &mut queue, id);
    }

    result
}

fn decrement_neighbors(
    adjacency: &HashMap<NodeId, Vec<NodeId>>,
    in_degree: &mut HashMap<NodeId, usize>,
    queue: &mut VecDeque<NodeId>,
    id: NodeId,
) {
    let Some(neighbors) = adjacency.get(&id) else {
        return;
    };
    for &neighbor in neighbors {
        let Some(deg) = in_degree.get_mut(&neighbor) else {
            continue;
        };
        *deg = deg.saturating_sub(1);
        if *deg == 0 {
            queue.push_back(neighbor);
        }
    }
}
