// Inspector panel — property editing for selected nodes and layers.

/// Draw the inspector with full app state access.
pub fn draw_inspector(ui: &mut egui::Ui, app: &mut crate::app::KitbashApp) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        draw_canvas_settings(ui, app);
        ui.separator();
        draw_selected_layer(ui, app);
        ui.separator();
        draw_export_section(ui, app);
    });
}

fn draw_canvas_settings(ui: &mut egui::Ui, app: &mut crate::app::KitbashApp) {
    ui.heading("Canvas");
    ui.horizontal(|ui| {
        ui.label("Size:");
        ui.add(egui::DragValue::new(&mut app.canvas_size[0]).range(16..=1024));
        ui.label("×");
        ui.add(egui::DragValue::new(&mut app.canvas_size[1]).range(16..=1024));
    });
    ui.horizontal(|ui| {
        ui.label("BG:");
        ui.color_edit_button_srgba(&mut app.bg_color);
    });
    ui.horizontal(|ui| {
        ui.label("Zoom:");
        ui.add(egui::Slider::new(&mut app.preview_zoom, 0.5..=10.0));
    });
    if ui.button("Reset View").clicked() {
        app.canvas_pan = egui::Vec2::ZERO;
        app.preview_zoom = 4.0;
    }
}

fn draw_selected_layer(ui: &mut egui::Ui, app: &mut crate::app::KitbashApp) {
    let Some(selected_id) = app.selected_layer_id else {
        ui.label("No layer selected.");
        return;
    };
    let Some(layer) = app.layers.iter_mut().find(|l| l.id == selected_id) else {
        ui.label("Layer not found.");
        return;
    };

    ui.heading(format!("Layer: {}", layer.name));
    ui.horizontal(|ui| {
        ui.label("Scale:");
        ui.add(egui::Slider::new(&mut layer.transform.scale, 0.1..=5.0));
    });
    ui.horizontal(|ui| {
        ui.label("Offset:");
        ui.add(
            egui::DragValue::new(&mut layer.transform.offset.x)
                .speed(1.0)
                .prefix("X: "),
        );
        ui.add(
            egui::DragValue::new(&mut layer.transform.offset.y)
                .speed(1.0)
                .prefix("Y: "),
        );
    });

    if ui.button("Snap to Pixel").clicked() {
        layer.transform.offset.x = layer.transform.offset.x.round();
        layer.transform.offset.y = layer.transform.offset.y.round();
    }

    if ui.button("Reset").clicked() {
        layer.transform.scale = 1.0;
        layer.transform.offset = egui::Vec2::ZERO;
    }
}

fn draw_export_section(ui: &mut egui::Ui, app: &mut crate::app::KitbashApp) {
    ui.heading("Export");
    ui.horizontal(|ui| {
        ui.label("Scale:");
        ui.add(
            egui::DragValue::new(&mut app.export_scale)
                .range(1..=10)
                .speed(0.1),
        );
    });

    let res = format!(
        "{} × {}",
        app.canvas_size[0] * app.export_scale,
        app.canvas_size[1] * app.export_scale
    );
    ui.label(format!("Output: {res}"));

    ui.horizontal(|ui| {
        if ui.button("PNGs").clicked() {
            crate::imaging::export::export_individual_pngs(
                &app.layers,
                app.canvas_size,
                app.export_scale,
            );
        }
        if ui.button("ZIP").clicked() {
            crate::imaging::export::export_zip(&app.layers, app.canvas_size, app.export_scale);
        }
    });
}
