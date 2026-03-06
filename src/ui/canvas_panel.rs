// Canvas preview panel — shows the composited sprite preview.

use crate::app::KitbashApp;

/// Draw the canvas content directly (called from the dock dispatcher).
pub fn draw_canvas(ui: &mut egui::Ui, app: &mut KitbashApp) {
    let available_rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(available_rect);

    handle_canvas_input(ui, app);

    let canvas_w = app.canvas_size[0] as f32 * app.preview_zoom;
    let canvas_h = app.canvas_size[1] as f32 * app.preview_zoom;
    let center = available_rect.center() + app.canvas_pan;
    let canvas_rect = egui::Rect::from_center_size(center, egui::vec2(canvas_w, canvas_h));

    draw_checkerboard(&painter, canvas_rect, app.canvas_size, app.preview_zoom);

    if app.bg_color != egui::Color32::TRANSPARENT {
        painter.rect_filled(canvas_rect, 0.0, app.bg_color);
    }

    draw_layers(ui, &painter, canvas_rect, app);

    painter.rect_stroke(
        canvas_rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::WHITE),
        egui::StrokeKind::Outside,
    );
}

fn handle_canvas_input(ui: &egui::Ui, app: &mut KitbashApp) {
    let input = ui.input(|i| i.clone());
    if input.pointer.button_down(egui::PointerButton::Middle) {
        app.canvas_pan += input.pointer.delta();
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

fn draw_layers(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    canvas_rect: egui::Rect,
    app: &mut KitbashApp,
) {
    let ctx = ui.ctx().clone();
    let mut drag_delta = egui::Vec2::ZERO;
    let mut dragged_id = None;

    for layer in &mut app.layers {
        if !layer.visible {
            continue;
        }

        let texture_id = ensure_texture(&ctx, layer);
        let (part_rect, part_screen_pos) = compute_layer_rect(layer, canvas_rect, app.preview_zoom);

        let _ = part_screen_pos;

        let interact_response =
            ui.interact(part_rect, egui::Id::new(layer.id), egui::Sense::drag());

        if interact_response.dragged() {
            dragged_id = Some(layer.id);
            drag_delta = interact_response.drag_delta() / app.preview_zoom;
            app.selected_layer_id = Some(layer.id);
        }
        if interact_response.clicked() {
            app.selected_layer_id = Some(layer.id);
        }

        if Some(layer.id) == app.selected_layer_id {
            painter.rect_stroke(
                part_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::YELLOW),
                egui::StrokeKind::Outside,
            );
        }

        let mut mesh = egui::Mesh::with_texture(texture_id);
        mesh.add_rect_with_uv(
            part_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        painter.add(mesh);
    }

    if let Some(id) = dragged_id {
        if let Some(layer) = app.layers.iter_mut().find(|l| l.id == id) {
            layer.transform.offset += drag_delta;
        }
    }
}

fn ensure_texture(ctx: &egui::Context, layer: &mut crate::app::LayerImage) -> egui::TextureId {
    if let Some(tex) = &layer.texture {
        return tex.id();
    }
    let tex = ctx.load_texture(
        &layer.name,
        egui::ColorImage::from_rgba_unmultiplied(
            [
                layer.source_image.width() as _,
                layer.source_image.height() as _,
            ],
            layer.source_image.to_rgba8().as_flat_samples().as_slice(),
        ),
        egui::TextureOptions::NEAREST,
    );
    let id = tex.id();
    layer.texture = Some(tex);
    id
}

fn compute_layer_rect(
    layer: &crate::app::LayerImage,
    canvas_rect: egui::Rect,
    zoom: f32,
) -> (egui::Rect, egui::Pos2) {
    let aligned_pos = egui::pos2(
        layer.transform.offset.x.round(),
        layer.transform.offset.y.round(),
    );
    let part_screen_pos = canvas_rect.min + (aligned_pos.to_vec2() * zoom);
    let part_w = layer.source_image.width() as f32 * layer.transform.scale * zoom;
    let part_h = layer.source_image.height() as f32 * layer.transform.scale * zoom;
    let part_rect = egui::Rect::from_min_size(part_screen_pos, egui::vec2(part_w, part_h));
    (part_rect, part_screen_pos)
}
