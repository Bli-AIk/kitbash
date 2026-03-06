// Layer management panel — list, reorder, and manage sprite layers.

/// Draw the layer list with full app state.
pub fn draw_layer_list(ui: &mut egui::Ui, app: &mut crate::app::KitbashApp) {
    if ui.button("Import Images...").clicked() {
        spawn_import_task(app.msg_sender.clone());
    }

    ui.separator();

    let mut move_op = None;
    let mut delete_op = None;
    let layers_len = app.layers.len();

    for (idx, layer) in app.layers.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            let is_selected = Some(layer.id) == app.selected_layer_id;
            if ui.selectable_label(is_selected, &layer.name).clicked() {
                app.selected_layer_id = Some(layer.id);
            }
            ui.checkbox(&mut layer.visible, "");
            if ui.button("⬆").clicked() && idx > 0 {
                move_op = Some((idx, idx - 1));
            }
            if ui.button("⬇").clicked() && idx < layers_len - 1 {
                move_op = Some((idx, idx + 1));
            }
            if ui.button("✕").clicked() {
                delete_op = Some(idx);
            }
        });
    }

    if let Some((from, to)) = move_op {
        app.layers.swap(from, to);
    }
    if let Some(idx) = delete_op {
        let id = app.layers[idx].id;
        app.layers.remove(idx);
        if app.selected_layer_id == Some(id) {
            app.selected_layer_id = None;
        }
    }
}

fn spawn_import_task(sender: std::sync::mpsc::Sender<crate::app::AppMessage>) {
    let task = async move {
        let Some(handles) = rfd::AsyncFileDialog::new()
            .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
            .pick_files()
            .await
        else {
            return;
        };
        for handle in handles {
            let data = handle.read().await;
            let name = handle.file_name();
            let _ = sender.send(crate::app::AppMessage::ImageLoaded(name, data));
        }
    };

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(task);
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        futures::executor::block_on(task);
    });
}
