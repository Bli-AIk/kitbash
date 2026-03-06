// Preview panel — displays the Sprite Output result or interactive CanvasLayout canvas.

use crate::app::KitbashApp;
use crate::ui::node_graph_panel::KitbashNode;

/// Draw the preview panel.
pub fn draw_preview(ui: &mut egui::Ui, app: &mut KitbashApp) {
    let available_rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(available_rect);

    handle_preview_input(ui, app, available_rect);

    let canvas_size = resolve_canvas_size(app);

    // Auto-fit: zoom so canvas fits the available area with some margin
    if app.preview_auto_fit && canvas_size[0] > 0 && canvas_size[1] > 0 {
        let margin = 16.0;
        let avail_w = (available_rect.width() - margin * 2.0).max(1.0);
        let avail_h = (available_rect.height() - margin * 2.0).max(1.0);
        let fit_w = avail_w / canvas_size[0] as f32;
        let fit_h = avail_h / canvas_size[1] as f32;
        app.preview_zoom = fit_w.min(fit_h).max(0.1);
        app.canvas_pan = egui::Vec2::ZERO;
        app.preview_auto_fit = false;
    }

    let canvas_w = canvas_size[0] as f32 * app.preview_zoom;
    let canvas_h = canvas_size[1] as f32 * app.preview_zoom;
    let center = available_rect.center() + app.canvas_pan;
    let canvas_rect = egui::Rect::from_center_size(center, egui::vec2(canvas_w, canvas_h));

    draw_checkerboard(&painter, canvas_rect, canvas_size, app.preview_zoom);

    // If a CanvasLayout is selected, show interactive layer positioning
    let is_canvas_layout = app.node_graph_panel.selected_node.is_some_and(|node_id| {
        matches!(
            app.node_graph_panel
                .snarl
                .get_node_info(node_id)
                .map(|n| &n.value),
            Some(KitbashNode::CanvasLayout { .. })
        )
    });

    if let Some(node_id) = app
        .node_graph_panel
        .selected_node
        .filter(|_| is_canvas_layout)
    {
        draw_canvas_layout_interactive(ui, &painter, canvas_rect, app, node_id);
    } else {
        // Default: show images connected to Sprite Output
        draw_output_preview(ui, &painter, canvas_rect, app);
    }

    painter.rect_stroke(
        canvas_rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::WHITE),
        egui::StrokeKind::Outside,
    );

    // Status text
    ui.scope_builder(egui::UiBuilder::new().max_rect(available_rect), |ui| {
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            let status = format!(
                "{}×{} · {:.0}%",
                canvas_size[0],
                canvas_size[1],
                app.preview_zoom * 100.0
            );
            ui.label(egui::RichText::new(status).small().weak());
        });
    });
}

/// Resolve canvas size from selected node or Sprite Output defaults.
fn resolve_canvas_size(app: &KitbashApp) -> [u32; 2] {
    // Check if selected node is CanvasLayout — use its canvas size
    if let Some(node_id) = app.node_graph_panel.selected_node {
        if let Some(node_info) = app.node_graph_panel.snarl.get_node_info(node_id) {
            if let KitbashNode::CanvasLayout { canvas_size, .. } = &node_info.value {
                return *canvas_size;
            }
        }
    }
    // Check Sprite Output's explicit canvas size
    if let Some(node_info) = app
        .node_graph_panel
        .snarl
        .get_node_info(app.node_graph_panel.output_node)
    {
        if let KitbashNode::SpriteOutput {
            canvas_size: Some(size),
            ..
        } = &node_info.value
        {
            return *size;
        }
    }
    // Auto-detect from connected images — use bounding box of all input images
    let image_ids = collect_output_image_ids(app);
    let mut max_w: u32 = 0;
    let mut max_h: u32 = 0;
    for &id in &image_ids {
        if let Some(img) = app.image_store.images.iter().find(|im| im.id == id) {
            max_w = max_w.max(img.image.width());
            max_h = max_h.max(img.image.height());
        }
    }
    if max_w > 0 && max_h > 0 {
        [max_w, max_h]
    } else {
        [64, 64]
    }
}

fn handle_preview_input(ui: &egui::Ui, app: &mut KitbashApp, panel_rect: egui::Rect) {
    let response = ui.interact(
        ui.available_rect_before_wrap(),
        egui::Id::new("preview_interaction"),
        egui::Sense::click_and_drag(),
    );

    // Middle-mouse drag to pan
    if response.dragged_by(egui::PointerButton::Middle) {
        app.canvas_pan += response.drag_delta();
    }

    if !response.hovered() {
        return;
    }

    // Smooth exponential scroll-wheel zoom, centered on cursor
    let (scroll_delta, cursor_pos) = ui.input(|i| (i.smooth_scroll_delta.y, i.pointer.hover_pos()));
    if scroll_delta != 0.0 {
        let zoom_speed = 0.005;
        let factor = (scroll_delta * zoom_speed).exp();
        let old_zoom = app.preview_zoom;
        let new_zoom = (old_zoom * factor).clamp(0.1, 64.0);

        // Adjust pan so the point under the cursor stays fixed
        if let Some(cursor) = cursor_pos {
            let center = panel_rect.center() + app.canvas_pan;
            let offset = cursor - center;
            app.canvas_pan -= offset * (new_zoom / old_zoom - 1.0);
        }

        app.preview_zoom = new_zoom;
    }
}

fn draw_checkerboard(
    painter: &egui::Painter,
    canvas_rect: egui::Rect,
    canvas_size: [u32; 2],
    zoom: f32,
) {
    painter.rect_filled(canvas_rect, 0.0, egui::Color32::from_gray(50));

    let check_size = 8.0 * zoom;
    let cols = (canvas_size[0] as f32 / 8.0).ceil() as u32;
    let rows = (canvas_size[1] as f32 / 8.0).ceil() as u32;
    let light = egui::Color32::from_gray(100);

    for r in 0..rows {
        for c in 0..cols {
            if (r + c) % 2 != 0 {
                continue;
            }
            let x = canvas_rect.min.x + c as f32 * check_size;
            let y = canvas_rect.min.y + r as f32 * check_size;
            let rect =
                egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(check_size, check_size))
                    .intersect(canvas_rect);

            if rect.is_positive() {
                painter.rect_filled(rect, 0.0, light);
            }
        }
    }
}

/// Draw CanvasLayout layers interactively — users can drag to reposition.
fn draw_canvas_layout_interactive(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    canvas_rect: egui::Rect,
    app: &mut KitbashApp,
    node_id: egui_snarl::NodeId,
) {
    let image_ids: Vec<u64> = collect_connected_image_ids(app, node_id);
    let zoom = app.preview_zoom;

    // Collect current transforms as copies (avoids holding mutable borrow on snarl)
    let (offsets, scales) = {
        let node = &mut app.node_graph_panel.snarl[node_id];
        let (layer_offsets, layer_scales) = match node {
            KitbashNode::CanvasLayout {
                layer_offsets,
                layer_scales,
                ..
            } => (layer_offsets, layer_scales),
            _ => return,
        };
        while layer_offsets.len() < image_ids.len() {
            layer_offsets.push([0.0, 0.0]);
        }
        while layer_scales.len() < image_ids.len() {
            layer_scales.push(1.0);
        }
        (layer_offsets.clone(), layer_scales.clone())
    };

    // Render and collect drag deltas
    let mut drag_updates: Vec<(usize, egui::Vec2)> = Vec::new();

    for (i, &img_id) in image_ids.iter().enumerate() {
        let Some(img) = app.image_store.images.iter_mut().find(|im| im.id == img_id) else {
            continue;
        };

        let tex_id = ensure_texture(ui.ctx(), img);
        let offset = offsets.get(i).copied().unwrap_or([0.0, 0.0]);
        let scale = scales.get(i).copied().unwrap_or(1.0);

        let screen_pos = canvas_rect.min + egui::vec2(offset[0].round(), offset[1].round()) * zoom;
        let w = img.image.width() as f32 * scale * zoom;
        let h = img.image.height() as f32 * scale * zoom;
        let layer_rect = egui::Rect::from_min_size(screen_pos, egui::vec2(w, h));

        let mut mesh = egui::Mesh::with_texture(tex_id);
        mesh.add_rect_with_uv(
            layer_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        painter.add(mesh);

        let response = ui.interact(
            layer_rect,
            egui::Id::new(("preview_layer", i)),
            egui::Sense::drag(),
        );
        if response.dragged() {
            drag_updates.push((i, response.drag_delta() / zoom));
        }
    }

    // Apply drag updates back to the node
    if !drag_updates.is_empty() {
        apply_drag_updates(app, node_id, &drag_updates);
    }
}

fn apply_drag_updates(
    app: &mut KitbashApp,
    node_id: egui_snarl::NodeId,
    updates: &[(usize, egui::Vec2)],
) {
    let node = &mut app.node_graph_panel.snarl[node_id];
    let KitbashNode::CanvasLayout { layer_offsets, .. } = node else {
        return;
    };
    for &(idx, delta) in updates {
        if let Some(off) = layer_offsets.get_mut(idx) {
            off[0] += delta.x;
            off[1] += delta.y;
        }
    }
}

/// Trace back through snarl connections to find which image IDs feed into this node.
fn collect_connected_image_ids(app: &KitbashApp, _node_id: egui_snarl::NodeId) -> Vec<u64> {
    // For now, return all images from the ImageInput node
    let input = &app.node_graph_panel.snarl[app.node_graph_panel.input_node];
    if let KitbashNode::ImageInput { image_ids } = input {
        image_ids.clone()
    } else {
        vec![]
    }
}

/// Trace the Sprite Output's input connections back to collect image IDs.
fn collect_output_image_ids(app: &KitbashApp) -> Vec<u64> {
    let output_node = app.node_graph_panel.output_node;
    let pin_id = egui_snarl::InPinId {
        node: output_node,
        input: 0,
    };
    trace_image_ids_from_pin(app, pin_id)
}

/// Recursively trace image IDs from an input pin back through the graph.
fn trace_image_ids_from_pin(app: &KitbashApp, pin_id: egui_snarl::InPinId) -> Vec<u64> {
    let in_pin = app.node_graph_panel.snarl.in_pin(pin_id);
    let mut result = Vec::new();
    for &remote in &in_pin.remotes {
        let source_node = &app.node_graph_panel.snarl[remote.node];
        match source_node {
            KitbashNode::ImageInput { image_ids } => {
                // Map output pin index to image ID
                if let Some(&id) = image_ids.get(remote.output) {
                    result.push(id);
                }
            }
            _ => {
                // For processing nodes, trace through their inputs recursively
                let input_count = source_node.input_count();
                for i in 0..input_count {
                    let upstream = egui_snarl::InPinId {
                        node: remote.node,
                        input: i,
                    };
                    result.extend(trace_image_ids_from_pin(app, upstream));
                }
            }
        }
    }
    result
}

/// Draw the Sprite Output preview — renders images connected to the output node.
fn draw_output_preview(
    _ui: &mut egui::Ui,
    painter: &egui::Painter,
    canvas_rect: egui::Rect,
    app: &mut KitbashApp,
) {
    let image_ids = collect_output_image_ids(app);
    let zoom = app.preview_zoom;

    for &img_id in &image_ids {
        let Some(img) = app.image_store.images.iter_mut().find(|im| im.id == img_id) else {
            continue;
        };
        let tex_id = ensure_texture(painter.ctx(), img);
        let w = img.image.width() as f32 * zoom;
        let h = img.image.height() as f32 * zoom;
        let img_rect = egui::Rect::from_min_size(canvas_rect.min, egui::vec2(w, h));

        let mut mesh = egui::Mesh::with_texture(tex_id);
        mesh.add_rect_with_uv(
            img_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        painter.add(mesh);
    }
}

fn ensure_texture(ctx: &egui::Context, img: &mut crate::app::ImportedImage) -> egui::TextureId {
    if let Some(tex) = &img.texture {
        return tex.id();
    }
    let tex = ctx.load_texture(
        &img.name,
        egui::ColorImage::from_rgba_unmultiplied(
            [img.image.width() as _, img.image.height() as _],
            img.image.to_rgba8().as_flat_samples().as_slice(),
        ),
        egui::TextureOptions::NEAREST,
    );
    let id = tex.id();
    img.texture = Some(tex);
    id
}
