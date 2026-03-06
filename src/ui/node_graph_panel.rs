// Node graph editor panel — visual node editing interface.

use crate::node::graph::{NodeGraph, NodeId};
use crate::theme::colors::{blue, gray};

/// State for the node graph editor UI.
pub struct NodeGraphPanel {
    pub pan: egui::Vec2,
    pub zoom: f32,
    #[allow(dead_code)]
    pub dragging_node: Option<NodeId>,
    #[allow(dead_code)]
    pub connecting_from: Option<(NodeId, usize, bool)>,
}

impl Default for NodeGraphPanel {
    fn default() -> Self {
        Self {
            pan: egui::Vec2::ZERO,
            zoom: 1.0,
            dragging_node: None,
            connecting_from: None,
        }
    }
}

/// Draw the node graph with full state access (called from dock dispatcher).
pub fn draw_node_graph(ui: &mut egui::Ui, graph: &mut NodeGraph, state: &mut NodeGraphPanel) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(rect);

    // Background
    painter.rect_filled(rect, 0.0, gray::S125);

    // Grid
    draw_grid(&painter, rect, state);

    // Handle panning
    let input = ui.input(|i| i.clone());
    if input.pointer.button_down(egui::PointerButton::Middle) {
        state.pan += input.pointer.delta();
    }

    // Draw connections first (below nodes)
    draw_connections(&painter, graph, state, rect);

    // Draw nodes
    let node_ids: Vec<NodeId> = graph.nodes.keys().copied().collect();
    for node_id in node_ids {
        draw_node(ui, &painter, graph, state, node_id, rect);
    }
}

fn draw_grid(painter: &egui::Painter, rect: egui::Rect, state: &NodeGraphPanel) {
    let grid_spacing = 20.0 * state.zoom;
    let grid_color = gray::S200;

    let offset_x = state.pan.x % grid_spacing;
    let offset_y = state.pan.y % grid_spacing;

    let mut x = rect.min.x + offset_x;
    while x < rect.max.x {
        painter.line_segment(
            [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
            egui::Stroke::new(1.0, grid_color),
        );
        x += grid_spacing;
    }

    let mut y = rect.min.y + offset_y;
    while y < rect.max.y {
        painter.line_segment(
            [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
            egui::Stroke::new(1.0, grid_color),
        );
        y += grid_spacing;
    }
}

fn draw_connections(
    painter: &egui::Painter,
    graph: &NodeGraph,
    state: &NodeGraphPanel,
    area_rect: egui::Rect,
) {
    for conn in &graph.connections {
        let Some(from_node) = graph.nodes.get(&conn.from_node) else {
            continue;
        };
        let Some(to_node) = graph.nodes.get(&conn.to_node) else {
            continue;
        };

        let from_pos = node_output_port_pos(from_node, conn.from_port, state, area_rect);
        let to_pos = node_input_port_pos(to_node, conn.to_port, state, area_rect);

        // Bezier curve
        let dx = (to_pos.x - from_pos.x).abs() * 0.5;
        let ctrl1 = egui::pos2(from_pos.x + dx, from_pos.y);
        let ctrl2 = egui::pos2(to_pos.x - dx, to_pos.y);

        let points: Vec<egui::Pos2> = (0..=20)
            .map(|i| {
                let t = i as f32 / 20.0;
                cubic_bezier(from_pos, ctrl1, ctrl2, to_pos, t)
            })
            .collect();

        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(2.0, blue::S500),
        ));
    }
}

fn draw_node(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    graph: &mut NodeGraph,
    state: &mut NodeGraphPanel,
    node_id: NodeId,
    area_rect: egui::Rect,
) {
    let Some(node) = graph.nodes.get(&node_id) else {
        return;
    };

    let info = node.processor.info();
    let node_pos = area_rect.min + node.position * state.zoom + state.pan;
    let node_width = 160.0 * state.zoom;
    let header_height = 24.0 * state.zoom;
    let port_height = 18.0 * state.zoom;
    let total_ports = info.inputs.len().max(info.outputs.len());
    let node_height = header_height + total_ports as f32 * port_height + 8.0 * state.zoom;

    let node_rect = egui::Rect::from_min_size(node_pos, egui::vec2(node_width, node_height));

    // Node body
    painter.rect_filled(node_rect, 4.0 * state.zoom, gray::S200);
    painter.rect_stroke(
        node_rect,
        4.0 * state.zoom,
        egui::Stroke::new(1.0, gray::S350),
        egui::StrokeKind::Outside,
    );

    // Header
    let header_rect = egui::Rect::from_min_size(node_pos, egui::vec2(node_width, header_height));
    painter.rect_filled(header_rect, 4.0 * state.zoom, gray::S300);
    painter.text(
        header_rect.center(),
        egui::Align2::CENTER_CENTER,
        &info.name,
        egui::FontId::proportional(12.0 * state.zoom),
        gray::S1000,
    );

    // Input ports
    for (i, input) in info.inputs.iter().enumerate() {
        let port_y = node_pos.y + header_height + i as f32 * port_height + port_height * 0.5;
        let port_pos = egui::pos2(node_pos.x, port_y);
        painter.circle_filled(port_pos, 4.0 * state.zoom, blue::S500);
        painter.text(
            egui::pos2(node_pos.x + 10.0 * state.zoom, port_y),
            egui::Align2::LEFT_CENTER,
            &input.name,
            egui::FontId::proportional(10.0 * state.zoom),
            gray::S775,
        );
    }

    // Output ports
    for (i, output) in info.outputs.iter().enumerate() {
        let port_y = node_pos.y + header_height + i as f32 * port_height + port_height * 0.5;
        let port_pos = egui::pos2(node_pos.x + node_width, port_y);
        painter.circle_filled(port_pos, 4.0 * state.zoom, blue::S500);
        painter.text(
            egui::pos2(node_pos.x + node_width - 10.0 * state.zoom, port_y),
            egui::Align2::RIGHT_CENTER,
            &output.name,
            egui::FontId::proportional(10.0 * state.zoom),
            gray::S775,
        );
    }

    // Node dragging
    let interact = ui.interact(
        node_rect,
        egui::Id::new(("node", node_id)),
        egui::Sense::drag(),
    );
    if interact.dragged() {
        let delta = interact.drag_delta() / state.zoom;
        if let Some(node) = graph.nodes.get_mut(&node_id) {
            node.position += delta;
        }
    }
}

fn node_output_port_pos(
    node: &crate::node::graph::NodeInstance,
    port_idx: usize,
    state: &NodeGraphPanel,
    area_rect: egui::Rect,
) -> egui::Pos2 {
    let node_pos = area_rect.min + node.position * state.zoom + state.pan;
    let node_width = 160.0 * state.zoom;
    let header_height = 24.0 * state.zoom;
    let port_height = 18.0 * state.zoom;
    let port_y = node_pos.y + header_height + port_idx as f32 * port_height + port_height * 0.5;
    egui::pos2(node_pos.x + node_width, port_y)
}

fn node_input_port_pos(
    node: &crate::node::graph::NodeInstance,
    port_idx: usize,
    state: &NodeGraphPanel,
    area_rect: egui::Rect,
) -> egui::Pos2 {
    let node_pos = area_rect.min + node.position * state.zoom + state.pan;
    let header_height = 24.0 * state.zoom;
    let port_height = 18.0 * state.zoom;
    let port_y = node_pos.y + header_height + port_idx as f32 * port_height + port_height * 0.5;
    egui::pos2(node_pos.x, port_y)
}

fn cubic_bezier(
    p0: egui::Pos2,
    p1: egui::Pos2,
    p2: egui::Pos2,
    p3: egui::Pos2,
    t: f32,
) -> egui::Pos2 {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    let x = uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x;
    let y = uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y;
    egui::pos2(x, y)
}
