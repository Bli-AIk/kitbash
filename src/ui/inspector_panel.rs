// Inspector panel — context-aware property editing based on selected node.

use crate::app::KitbashApp;
use crate::ui::node_graph_panel::KitbashNode;

/// Draw the inspector with context based on the selected node.
pub fn draw_inspector(ui: &mut egui::Ui, app: &mut KitbashApp) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let Some(node_id) = app.node_graph_panel.selected_node else {
            ui.label("Select a node to edit its properties.");
            return;
        };

        // Verify node still exists
        if app.node_graph_panel.snarl.get_node_info(node_id).is_none() {
            app.node_graph_panel.selected_node = None;
            ui.label("Select a node to edit its properties.");
            return;
        }

        let node_name = app.node_graph_panel.snarl[node_id].name().to_owned();
        ui.heading(&node_name);
        ui.separator();

        // Dispatch to node-specific inspector UI
        match app.node_graph_panel.snarl[node_id].clone() {
            KitbashNode::ImageInput { .. } => {
                draw_image_input_inspector(ui, app);
            }
            KitbashNode::SpriteOutput { .. } => {
                draw_sprite_output_inspector(ui, app);
            }
            KitbashNode::CanvasLayout { .. } => {
                draw_canvas_layout_inspector(ui, app, node_id);
            }
            KitbashNode::ReplaceColor { .. } => {
                draw_replace_color_inspector(ui, app, node_id);
            }
            KitbashNode::GridSlice { .. } => {
                draw_grid_slice_inspector(ui, app, node_id);
            }
            KitbashNode::ColorDetect { .. } => {
                draw_color_detect_inspector(ui, app, node_id);
            }
            KitbashNode::Composite { .. } => {
                draw_composite_inspector(ui, app, node_id);
            }
            KitbashNode::Transform { .. } => {
                draw_transform_inspector(ui, app, node_id);
            }
        }
    });
}

fn draw_image_input_inspector(ui: &mut egui::Ui, app: &mut KitbashApp) {
    if ui.button("Import Images...").clicked() {
        crate::app::trigger_import_public(app);
    }
    ui.separator();

    ui.label(format!("{} images loaded", app.image_store.images.len()));
    let mut to_remove = None;
    for (i, img) in app.image_store.images.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(&img.name);
            ui.label(format!("{}×{}", img.image.width(), img.image.height()));
            if ui.small_button("✕").clicked() {
                to_remove = Some(i);
            }
        });
    }
    if let Some(idx) = to_remove {
        let removed_id = app.image_store.images[idx].id;
        app.image_store.images.remove(idx);
        // Also remove from ImageInput node
        let node = &mut app.node_graph_panel.snarl[app.node_graph_panel.input_node];
        if let KitbashNode::ImageInput { image_ids } = node {
            image_ids.retain(|&id| id != removed_id);
        }
    }
}

fn draw_sprite_output_inspector(ui: &mut egui::Ui, app: &mut KitbashApp) {
    let node = &mut app.node_graph_panel.snarl[app.node_graph_panel.output_node];
    let KitbashNode::SpriteOutput {
        canvas_size,
        export_scale,
    } = node
    else {
        return;
    };

    let mut use_custom_size = canvas_size.is_some();
    ui.checkbox(&mut use_custom_size, "Custom Canvas Size");

    if use_custom_size {
        let size = canvas_size.get_or_insert([64, 64]);
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut size[0]).range(1..=4096));
            ui.label("×");
            ui.add(egui::DragValue::new(&mut size[1]).range(1..=4096));
        });
    } else {
        *canvas_size = None;
        ui.label("Size: auto (from input)");
    }

    ui.horizontal(|ui| {
        ui.label("Export Scale:");
        ui.add(egui::DragValue::new(export_scale).range(1..=10));
    });

    ui.separator();
    ui.label("Export is handled via the Sprite Output node.");
}

fn draw_canvas_layout_inspector(
    ui: &mut egui::Ui,
    app: &mut KitbashApp,
    node_id: egui_snarl::NodeId,
) {
    let node = &mut app.node_graph_panel.snarl[node_id];
    let KitbashNode::CanvasLayout {
        canvas_size,
        bg_color,
        num_inputs,
        layer_offsets,
        layer_scales,
    } = node
    else {
        return;
    };

    ui.horizontal(|ui| {
        ui.label("Canvas:");
        ui.add(egui::DragValue::new(&mut canvas_size[0]).range(1..=4096));
        ui.label("×");
        ui.add(egui::DragValue::new(&mut canvas_size[1]).range(1..=4096));
    });

    let mut color =
        egui::Color32::from_rgba_unmultiplied(bg_color[0], bg_color[1], bg_color[2], bg_color[3]);
    ui.horizontal(|ui| {
        ui.label("BG:");
        ui.color_edit_button_srgba(&mut color);
    });
    *bg_color = [color.r(), color.g(), color.b(), color.a()];

    ui.horizontal(|ui| {
        ui.label("Input Slots:");
        ui.add(egui::DragValue::new(num_inputs).range(1..=16));
    });

    ui.separator();
    ui.label("Layer Transforms:");
    for i in 0..*num_inputs {
        while layer_offsets.len() <= i {
            layer_offsets.push([0.0, 0.0]);
        }
        while layer_scales.len() <= i {
            layer_scales.push(1.0);
        }
        ui.horizontal(|ui| {
            ui.label(format!("[{i}]"));
            ui.add(
                egui::DragValue::new(&mut layer_offsets[i][0])
                    .speed(1.0)
                    .prefix("X:"),
            );
            ui.add(
                egui::DragValue::new(&mut layer_offsets[i][1])
                    .speed(1.0)
                    .prefix("Y:"),
            );
            ui.add(
                egui::DragValue::new(&mut layer_scales[i])
                    .range(0.1..=10.0)
                    .prefix("S:"),
            );
        });
    }
}

fn draw_replace_color_inspector(
    ui: &mut egui::Ui,
    app: &mut KitbashApp,
    node_id: egui_snarl::NodeId,
) {
    let node = &mut app.node_graph_panel.snarl[node_id];
    let KitbashNode::ReplaceColor { from, to } = node else {
        return;
    };
    let mut from_c = egui::Color32::from_rgba_unmultiplied(from[0], from[1], from[2], from[3]);
    let mut to_c = egui::Color32::from_rgba_unmultiplied(to[0], to[1], to[2], to[3]);
    ui.horizontal(|ui| {
        ui.label("From:");
        ui.color_edit_button_srgba(&mut from_c);
    });
    ui.horizontal(|ui| {
        ui.label("To:");
        ui.color_edit_button_srgba(&mut to_c);
    });
    *from = [from_c.r(), from_c.g(), from_c.b(), from_c.a()];
    *to = [to_c.r(), to_c.g(), to_c.b(), to_c.a()];
}

fn draw_grid_slice_inspector(ui: &mut egui::Ui, app: &mut KitbashApp, node_id: egui_snarl::NodeId) {
    let node = &mut app.node_graph_panel.snarl[node_id];
    let KitbashNode::GridSlice { cols, rows } = node else {
        return;
    };
    ui.horizontal(|ui| {
        ui.label("Columns:");
        ui.add(egui::Slider::new(cols, 1..=64));
    });
    ui.horizontal(|ui| {
        ui.label("Rows:");
        ui.add(egui::Slider::new(rows, 1..=64));
    });
}

fn draw_color_detect_inspector(
    ui: &mut egui::Ui,
    app: &mut KitbashApp,
    node_id: egui_snarl::NodeId,
) {
    let node = &mut app.node_graph_panel.snarl[node_id];
    let KitbashNode::ColorDetect { threshold } = node else {
        return;
    };
    ui.horizontal(|ui| {
        ui.label("Threshold:");
        ui.add(egui::Slider::new(threshold, 0.0..=255.0));
    });
}

fn draw_composite_inspector(ui: &mut egui::Ui, app: &mut KitbashApp, node_id: egui_snarl::NodeId) {
    let node = &mut app.node_graph_panel.snarl[node_id];
    let KitbashNode::Composite { num_inputs } = node else {
        return;
    };
    ui.horizontal(|ui| {
        ui.label("Input Slots:");
        ui.add(egui::DragValue::new(num_inputs).range(2..=16));
    });
}

fn draw_transform_inspector(ui: &mut egui::Ui, app: &mut KitbashApp, node_id: egui_snarl::NodeId) {
    let node = &mut app.node_graph_panel.snarl[node_id];
    let KitbashNode::Transform { offset, scale } = node else {
        return;
    };
    ui.horizontal(|ui| {
        ui.label("Offset:");
        ui.add(egui::DragValue::new(&mut offset[0]).speed(1.0).prefix("X:"));
        ui.add(egui::DragValue::new(&mut offset[1]).speed(1.0).prefix("Y:"));
    });
    ui.horizontal(|ui| {
        ui.label("Scale:");
        ui.add(egui::Slider::new(scale, 0.1..=10.0));
    });
}
