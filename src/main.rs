#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use image::{imageops::FilterType, Rgba, RgbaImage};
use std::io::{Cursor, Write};
use std::sync::mpsc::{channel, Receiver, Sender};

// ----------------------------------------------------------------------------
// Data Structures
// ----------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Transform {
    offset: egui::Vec2,
    scale: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            offset: egui::Vec2::ZERO,
            scale: 1.0,
        }
    }
}

struct LayerImage {
    id: u64,
    name: String,
    source_image: image::DynamicImage,
    texture: Option<egui::TextureHandle>,
    transform: Transform,
    visible: bool,
}

enum AppMessage {
    ImageLoaded(String, Vec<u8>), // name, bytes
}

struct KitbashApp {
    // Canvas Config
    canvas_size: [u32; 2],
    bg_color: egui::Color32,
    export_scale: u32, // New: Export multiplier
    
    // State
    layers: Vec<LayerImage>, // Flat list again
    selected_layer_id: Option<u64>,
    next_id: u64,
    
    // Async Communication
    msg_sender: Sender<AppMessage>,
    msg_receiver: Receiver<AppMessage>,
    
    // UI State
    preview_zoom: f32,
    canvas_pan: egui::Vec2, // New: Canvas panning
}

impl Default for KitbashApp {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            canvas_size: [64, 64],
            bg_color: egui::Color32::TRANSPARENT,
            export_scale: 1,
            layers: Vec::new(),
            selected_layer_id: None,
            next_id: 0,
            msg_sender: sender,
            msg_receiver: receiver,
            preview_zoom: 4.0,
            canvas_pan: egui::Vec2::ZERO,
        }
    }
}

// ----------------------------------------------------------------------------
// Helper Functions
// ----------------------------------------------------------------------------

/// Render a single layer to a buffer (full canvas size)
fn render_single_layer(
    canvas_size: [u32; 2], 
    layer: &LayerImage, 
    export_scale: u32
) -> Option<RgbaImage> {
    if !layer.visible {
        return None;
    }

    let scale_f = export_scale as f32;
    let width = canvas_size[0] * export_scale;
    let height = canvas_size[1] * export_scale;
    
    let mut buffer = RgbaImage::new(width, height);
    // Note: Individual layers are transparent background by default
    
    let src_width = layer.source_image.width();
    let src_height = layer.source_image.height();
    
    let final_scale = layer.transform.scale * scale_f;
    
    let target_width = (src_width as f32 * final_scale).round() as u32;
    let target_height = (src_height as f32 * final_scale).round() as u32;

    if target_width == 0 || target_height == 0 {
        return Some(buffer);
    }

    let resized = layer.source_image.resize_exact(
        target_width, 
        target_height, 
        FilterType::Nearest
    );

    let x = (layer.transform.offset.x * scale_f).round() as i64;
    let y = (layer.transform.offset.y * scale_f).round() as i64;

    image::imageops::overlay(&mut buffer, &resized, x, y);
    
    Some(buffer)
}

#[cfg(target_arch = "wasm32")]
fn trigger_download(filename: &str, data: &[u8]) {
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, Url, HtmlAnchorElement};

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    let array = js_sys::Uint8Array::from(data);
    let parts = js_sys::Array::new();
    parts.push(&array);
    
    let props = BlobPropertyBag::new();
    if filename.ends_with(".zip") {
        props.set_type("application/zip");
    } else {
        props.set_type("image/png");
    }
    
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &props).unwrap();
    let url = Url::create_object_url_with_blob(&blob).unwrap();

    let link = document.create_element("a").unwrap();
    let link: HtmlAnchorElement = link.dyn_into().unwrap();
    link.set_href(&url);
    link.set_download(filename);
    link.style().set_property("display", "none").unwrap();
    
    body.append_child(&link).unwrap();
    link.click();
    body.remove_child(&link).unwrap();
    Url::revoke_object_url(&url).unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
fn trigger_download(filename: &str, data: &[u8]) {
    if let Ok(mut file) = std::fs::File::create(filename) {
        let _ = file.write_all(data);
        println!("Saved to {}", filename);
    }
}

// ----------------------------------------------------------------------------
// App Implementation
// ----------------------------------------------------------------------------

impl eframe::App for KitbashApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle async messages
        while let Ok(msg) = self.msg_receiver.try_recv() {
            match msg {
                AppMessage::ImageLoaded(name, bytes) => {
                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let id = self.next_id;
                        self.next_id += 1;
                        let layer = LayerImage {
                            id,
                            name,
                            source_image: img,
                            texture: None,
                            transform: Transform::default(),
                            visible: true,
                        };
                        self.layers.push(layer);
                    } else {
                        eprintln!("Failed to decode image: {}", name);
                    }
                }
            }
        }

        let is_mobile = ctx.screen_rect().width() < 600.0;

        // --------------------------------------------------------------------
        // UI Components
        // --------------------------------------------------------------------
        
        let control_panel_ui = |ui: &mut egui::Ui, app: &mut KitbashApp| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Kitbash Config");
                ui.separator();

                // Canvas Settings
                ui.collapsing("Canvas Setup", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Base W:");
                        ui.add(egui::DragValue::new(&mut app.canvas_size[0]).range(16..=1024));
                        ui.label("Base H:");
                        ui.add(egui::DragValue::new(&mut app.canvas_size[1]).range(16..=1024));
                    });
                    ui.horizontal(|ui| {
                        ui.label("BG Color:");
                        ui.color_edit_button_srgba(&mut app.bg_color);
                    });
                    ui.horizontal(|ui| {
                        ui.label("View Zoom:");
                        ui.add(egui::Slider::new(&mut app.preview_zoom, 0.5..=10.0));
                    });
                    
                    if ui.button("Reset View").clicked() {
                        app.canvas_pan = egui::Vec2::ZERO;
                        app.preview_zoom = 4.0;
                    }
                });

                ui.separator();
                
                // Asset Pipeline
                ui.heading("Layers");
                if ui.button("Import Images (Batch)...").clicked() {
                    let sender = app.msg_sender.clone();
                    let task = async move {
                        if let Some(handles) = rfd::AsyncFileDialog::new()
                            .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                            .pick_files()
                            .await 
                        {
                            for handle in handles {
                                let data = handle.read().await;
                                let name = handle.file_name();
                                let _ = sender.send(AppMessage::ImageLoaded(name, data));
                            }
                        }
                    };
                    
                    #[cfg(target_arch = "wasm32")]
                    wasm_bindgen_futures::spawn_local(task);
                    #[cfg(not(target_arch = "wasm32"))]
                    std::thread::spawn(move || { futures::executor::block_on(task); });
                }

                ui.separator();
                
                // Layer List (Reorderable)
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
                        if ui.button("X").clicked() {
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

                ui.separator();
                
                // Properties Panel
                if let Some(selected_id) = app.selected_layer_id {
                    if let Some(layer) = app.layers.iter_mut().find(|l| l.id == selected_id) {
                        ui.heading(format!("Properties: {}", layer.name));
                        
                        ui.horizontal(|ui| {
                            ui.label("Scale:");
                            ui.add(egui::Slider::new(&mut layer.transform.scale, 0.1..=5.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Offset:");
                            ui.add(egui::DragValue::new(&mut layer.transform.offset.x).speed(1.0).prefix("X: "));
                            ui.add(egui::DragValue::new(&mut layer.transform.offset.y).speed(1.0).prefix("Y: "));
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
                } else {
                    ui.label("Select a layer to edit.");
                }
                
                ui.separator();
                
                // Export System
                ui.heading("Export (Scattered)");
                ui.horizontal(|ui| {
                    ui.label("Export Scale:");
                    ui.add(egui::DragValue::new(&mut app.export_scale).range(1..=10).speed(0.1));
                });
                
                let current_res = format!("{} x {}", 
                    app.canvas_size[0] * app.export_scale, 
                    app.canvas_size[1] * app.export_scale
                );
                ui.label(format!("Output Res: {}", current_res));

                ui.horizontal(|ui| {
                    if ui.button("Download Individual PNGs").clicked() {
                        for (i, layer) in app.layers.iter().enumerate() {
                            if let Some(img) = render_single_layer(app.canvas_size, layer, app.export_scale) {
                                let mut bytes: Vec<u8> = Vec::new();
                                img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png).unwrap();
                                let filename = format!("{}_{}.png", i, layer.name);
                                trigger_download(&filename, &bytes);
                            }
                        }
                    }
                    
                    if ui.button("Download ZIP").clicked() {
                        let mut zip_buffer = Vec::new();
                        {
                            let mut zip = zip::ZipWriter::new(Cursor::new(&mut zip_buffer));
                            let options = zip::write::FileOptions::default()
                                .compression_method(zip::CompressionMethod::Deflated);
                            
                            // 1. Export each visible layer as PNG
                            for (i, layer) in app.layers.iter().enumerate() {
                                if let Some(img) = render_single_layer(app.canvas_size, layer, app.export_scale) {
                                    let mut bytes = Vec::new();
                                    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png).unwrap();
                                    
                                    let filename = format!("{}_{}.png", i, layer.name);
                                    zip.start_file(filename, options).unwrap();
                                    zip.write_all(&bytes).unwrap();
                                }
                            }
                            
                            // 2. Export Metadata
                            let meta: Vec<serde_json::Value> = app.layers.iter().map(|l| {
                                serde_json::json!({
                                    "name": l.name,
                                    "visible": l.visible,
                                    "scale": l.transform.scale,
                                    "offset": { "x": l.transform.offset.x.round(), "y": l.transform.offset.y.round() },
                                })
                            }).collect();
                            let json_str = serde_json::to_string_pretty(&meta).unwrap();
                            
                            zip.start_file("data.json", options).unwrap();
                            zip.write_all(json_str.as_bytes()).unwrap();
                            
                            zip.finish().unwrap();
                        }
                        
                        trigger_download("kitbash_layers.zip", &zip_buffer);
                    }
                });
            });
        };

        // Render UI Panels
        if is_mobile {
            egui::TopBottomPanel::bottom("bottom_panel")
                .resizable(true)
                .default_height(300.0)
                .show(ctx, |ui| control_panel_ui(ui, self));
        } else {
            egui::SidePanel::right("right_panel")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| control_panel_ui(ui, self));
        }

        // Central Canvas Area
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_rect = ui.available_rect_before_wrap();
            let painter = ui.painter_at(available_rect);

            // Handle Canvas Panning (Middle Mouse or Alt+Drag or Space+Drag logic)
            // Or just drag on empty space.
            let input = ui.input(|i| i.clone());
            
            // Allow panning if middle mouse is dragging
            if input.pointer.button_down(egui::PointerButton::Middle) {
                self.canvas_pan += input.pointer.delta();
            }

            // Calculate Canvas Rect (Centered + Pan)
            let canvas_w = self.canvas_size[0] as f32 * self.preview_zoom;
            let canvas_h = self.canvas_size[1] as f32 * self.preview_zoom;
            
            let center = available_rect.center() + self.canvas_pan;
            let canvas_rect = egui::Rect::from_center_size(center, egui::vec2(canvas_w, canvas_h));

            // Draw Background (Checkerboard)
            painter.rect_filled(canvas_rect, 0.0, egui::Color32::from_gray(50));

            let check_size = 8.0 * self.preview_zoom;
            let cols = (self.canvas_size[0] as f32 / 8.0).ceil() as u32;
            let rows = (self.canvas_size[1] as f32 / 8.0).ceil() as u32;

            for r in 0..rows {
                for c in 0..cols {
                    if (r + c) % 2 == 0 {
                         let x = canvas_rect.min.x + c as f32 * check_size;
                         let y = canvas_rect.min.y + r as f32 * check_size;
                         // Clip to canvas size
                         let rect = egui::Rect::from_min_size(
                             egui::pos2(x, y), 
                             egui::vec2(check_size, check_size)
                         ).intersect(canvas_rect);
                         
                         if rect.is_positive() {
                             painter.rect_filled(rect, 0.0, egui::Color32::from_gray(100));
                         }
                    }
                }
            }
            
            if self.bg_color != egui::Color32::TRANSPARENT {
                painter.rect_filled(canvas_rect, 0.0, self.bg_color);
            }

            // Draw Layers
            let mut drag_delta = egui::Vec2::ZERO;
            let mut dragged_id = None;

            for layer in &mut self.layers {
                if !layer.visible { continue; }

                let texture_id = if let Some(tex) = &layer.texture {
                    tex.id()
                } else {
                    let tex = ctx.load_texture(
                        &layer.name,
                        egui::ColorImage::from_rgba_unmultiplied(
                            [layer.source_image.width() as _, layer.source_image.height() as _],
                            layer.source_image.to_rgba8().as_flat_samples().as_slice(),
                        ),
                        egui::TextureOptions::NEAREST,
                    );
                    let id = tex.id();
                    layer.texture = Some(tex);
                    id
                };

                let aligned_pos = egui::pos2(layer.transform.offset.x.round(), layer.transform.offset.y.round());
                let part_screen_pos = canvas_rect.min + (aligned_pos.to_vec2() * self.preview_zoom);
                let part_w = layer.source_image.width() as f32 * layer.transform.scale * self.preview_zoom;
                let part_h = layer.source_image.height() as f32 * layer.transform.scale * self.preview_zoom;
                
                let part_rect = egui::Rect::from_min_size(part_screen_pos, egui::vec2(part_w, part_h));

                // Interaction
                let interact_response = ui.interact(part_rect, egui::Id::new(layer.id), egui::Sense::drag());
                
                if interact_response.dragged() {
                    dragged_id = Some(layer.id);
                    drag_delta = interact_response.drag_delta() / self.preview_zoom;
                    self.selected_layer_id = Some(layer.id);
                }
                if interact_response.clicked() {
                    self.selected_layer_id = Some(layer.id);
                }

                if Some(layer.id) == self.selected_layer_id {
                    painter.rect_stroke(part_rect, 0.0, egui::Stroke::new(2.0, egui::Color32::YELLOW));
                }

                let mut mesh = egui::Mesh::with_texture(texture_id);
                mesh.add_rect_with_uv(
                    part_rect, 
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), 
                    egui::Color32::WHITE
                );
                painter.add(mesh);
            }

            if let Some(id) = dragged_id {
                if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
                    layer.transform.offset += drag_delta;
                }
            }
            
            painter.rect_stroke(canvas_rect, 0.0, egui::Stroke::new(1.0, egui::Color32::WHITE));
        });
    }
}

// ----------------------------------------------------------------------------
// Entry Point
// ----------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Kitbash",
        native_options,
        Box::new(|_cc| Ok(Box::new(KitbashApp::default()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().expect("No window found").document().expect("No document found");
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("No element with id the_canvas_id found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element is not a canvas");

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(KitbashApp::default()))),
            )
            .await
            .expect("failed to start eframe");
    });
}