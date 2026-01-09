#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use image::{imageops::FilterType, Rgba, RgbaImage};
use std::io::{Cursor, Write};
use std::sync::mpsc::{channel, Receiver, Sender};

// ----------------------------------------------------------------------------
// Data Structures
// ----------------------------------------------------------------------------

struct Part {
    id: u64,
    name: String,
    // The raw source image (kept in CPU memory for high-quality export)
    source_image: image::DynamicImage,
    // The texture for display in egui
    texture: Option<egui::TextureHandle>,
    
    // Transform properties
    offset: egui::Vec2, // Position on canvas
    scale: f32,         // Uniform scale
    
    // Editor state
    visible: bool,
}

impl Part {
    fn new(id: u64, name: String, image: image::DynamicImage) -> Self {
        Self {
            id,
            name,
            source_image: image,
            texture: None, // Created on first frame or upload
            offset: egui::Vec2::ZERO,
            scale: 1.0,
            visible: true,
        }
    }
}

enum AppMessage {
    ImageLoaded(String, Vec<u8>), // name, bytes
}

struct KitbashApp {
    // Canvas Config
    canvas_size: [u32; 2],
    bg_color: egui::Color32,
    
    // State
    parts: Vec<Part>,
    selected_part_id: Option<u64>,
    next_id: u64,
    
    // Async Communication
    msg_sender: Sender<AppMessage>,
    msg_receiver: Receiver<AppMessage>,
    
    // UI State
    preview_zoom: f32,
}

impl Default for KitbashApp {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            canvas_size: [64, 64],
            bg_color: egui::Color32::TRANSPARENT,
            parts: Vec::new(),
            selected_part_id: None,
            next_id: 0,
            msg_sender: sender,
            msg_receiver: receiver,
            preview_zoom: 4.0, // Default zoom for visibility
        }
    }
}

// ----------------------------------------------------------------------------
// Helper Functions
// ----------------------------------------------------------------------------

/// Composite the final image based on current state
fn composite_image(canvas_size: [u32; 2], bg_color: egui::Color32, parts: &[Part]) -> RgbaImage {
    let mut buffer = RgbaImage::new(canvas_size[0], canvas_size[1]);
    
    // Fill background
    for pixel in buffer.pixels_mut() {
        *pixel = Rgba([bg_color.r(), bg_color.g(), bg_color.b(), bg_color.a()]);
    }

    for part in parts {
        if !part.visible {
            continue;
        }

        // 1. Resize source image using Nearest Neighbor
        let src_width = part.source_image.width();
        let src_height = part.source_image.height();
        
        let target_width = (src_width as f32 * part.scale).round() as u32;
        let target_height = (src_height as f32 * part.scale).round() as u32;

        if target_width == 0 || target_height == 0 {
            continue;
        }

        let resized = part.source_image.resize_exact(
            target_width, 
            target_height, 
            FilterType::Nearest
        );

        // 2. Calculate position
        // offset is in pixels relative to top-left (0,0) of canvas
        let x = part.offset.x as i64;
        let y = part.offset.y as i64;

        // 3. Overlay
        image::imageops::overlay(&mut buffer, &resized, x, y);
    }
    
    buffer
}

// Function to trigger download in browser
#[cfg(target_arch = "wasm32")]
fn trigger_download(filename: &str, data: &[u8]) {
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, Url, HtmlAnchorElement};

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    // Create Blob
    let array = js_sys::Uint8Array::from(data);
    let parts = js_sys::Array::new();
    parts.push(&array);
    
    let mut props = BlobPropertyBag::new();
    // Default to binary/octet-stream if unknown, or png/zip specific
    if filename.ends_with(".zip") {
        props.type_("application/zip");
    } else {
        props.type_("image/png");
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
    // Fallback for local run (just save to disk)
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
                        let part = Part::new(id, name, img);
                        self.parts.push(part);
                        self.selected_part_id = Some(id);
                    } else {
                        eprintln!("Failed to decode image: {}", name);
                    }
                }
            }
        }

        let is_mobile = ctx.screen_rect().width() < 600.0;

        // --------------------------------------------------------------------
        // Layout Definition
        // --------------------------------------------------------------------
        
        let control_panel_ui = |ui: &mut egui::Ui, app: &mut KitbashApp| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Kitbash Config");
                ui.separator();

                // Canvas Settings
                ui.collapsing("Canvas Setup", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        ui.add(egui::DragValue::new(&mut app.canvas_size[0]).range(16..=1024));
                        ui.label("Height:");
                        ui.add(egui::DragValue::new(&mut app.canvas_size[1]).range(16..=1024));
                    });
                    ui.horizontal(|ui| {
                        ui.label("BG Color:");
                        ui.color_edit_button_srgba(&mut app.bg_color);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Preview Zoom:");
                        ui.add(egui::Slider::new(&mut app.preview_zoom, 0.5..=10.0));
                    });
                });

                ui.separator();
                
                // Asset Pipeline
                ui.heading("Assets");
                if ui.button("Import Image (RFD)").clicked() {
                    let sender = app.msg_sender.clone();
                    let task = async move {
                        if let Some(handle) = rfd::AsyncFileDialog::new()
                            .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                            .pick_file()
                            .await 
                        {
                            let data = handle.read().await;
                            let name = handle.file_name();
                            let _ = sender.send(AppMessage::ImageLoaded(name, data));
                        }
                    };
                    
                    #[cfg(target_arch = "wasm32")]
                    wasm_bindgen_futures::spawn_local(task);
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                         // For native testing simple block
                        std::thread::spawn(move || {
                            futures::executor::block_on(task);
                        });
                    }
                }

                // Part List
                ui.separator();
                ui.label("Parts (Drag to Reorder):");
                
                // Simple list with reordering and selection
                let mut to_move = None; // (from, to)
                let mut to_delete = None;
                
                let list_len = app.parts.len();
                for (idx, part) in app.parts.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        // Selection
                        let is_selected = Some(part.id) == app.selected_part_id;
                        if ui.selectable_label(is_selected, &part.name).clicked() {
                            app.selected_part_id = Some(part.id);
                        }
                        
                        // Visibility toggle
                        ui.checkbox(&mut part.visible, "");

                        // Up/Down buttons for Z-Index (Rendering Order)
                        if ui.button("⬆").clicked() && idx > 0 {
                            to_move = Some((idx, idx - 1));
                        }
                        if ui.button("⬇").clicked() && idx < list_len - 1 {
                             to_move = Some((idx, idx + 1));
                        }
                        
                        if ui.button("X").clicked() {
                            to_delete = Some(idx);
                        }
                    });
                }

                if let Some((from, to)) = to_move {
                    app.parts.swap(from, to);
                }
                if let Some(idx) = to_delete {
                    app.parts.remove(idx);
                    app.selected_part_id = None;
                }

                ui.separator();
                
                // Properties Panel
                if let Some(selected_id) = app.selected_part_id {
                    if let Some(part) = app.parts.iter_mut().find(|p| p.id == selected_id) {
                        ui.heading(format!("Properties: {}", part.name));
                        ui.horizontal(|ui| {
                            ui.label("Scale:");
                            ui.add(egui::Slider::new(&mut part.scale, 0.1..=5.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Offset:");
                            ui.add(egui::DragValue::new(&mut part.offset.x).speed(1.0).prefix("X: "));
                            ui.add(egui::DragValue::new(&mut part.offset.y).speed(1.0).prefix("Y: "));
                        });
                        if ui.button("Reset Transform").clicked() {
                            part.scale = 1.0;
                            part.offset = egui::Vec2::ZERO;
                        }
                    }
                } else {
                    ui.label("Select a part to edit properties.");
                }
                
                ui.separator();
                
                // Export System
                ui.heading("Export");
                ui.horizontal(|ui| {
                    if ui.button("Download PNG").clicked() {
                        let img = composite_image(app.canvas_size, app.bg_color, &app.parts);
                        let mut bytes: Vec<u8> = Vec::new();
                        img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png).unwrap();
                        trigger_download("character.png", &bytes);
                    }
                    
                    if ui.button("Download ZIP").clicked() {
                        let img = composite_image(app.canvas_size, app.bg_color, &app.parts);
                        let mut png_bytes: Vec<u8> = Vec::new();
                        img.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png).unwrap();
                        
                        // Generate Metadata
                        let meta: Vec<serde_json::Value> = app.parts.iter().map(|p| {
                            serde_json::json!({
                                "name": p.name,
                                "scale": p.scale,
                                "offset": { "x": p.offset.x, "y": p.offset.y },
                                "z_index": 0 // Order in list implies z-index
                            })
                        }).collect();
                        let json_str = serde_json::to_string_pretty(&meta).unwrap();
                        
                        // Zip It
                        let mut zip_buffer = Vec::new();
                        {
                            let mut zip = zip::ZipWriter::new(Cursor::new(&mut zip_buffer));
                            let options = zip::write::FileOptions::default()
                                .compression_method(zip::CompressionMethod::Deflated);
                                
                            zip.start_file("merged.png", options).unwrap();
                            zip.write_all(&png_bytes).unwrap();
                            
                            zip.start_file("data.json", options).unwrap();
                            zip.write_all(json_str.as_bytes()).unwrap();
                            
                            zip.finish().unwrap();
                        }
                        
                        trigger_download("character_pack.zip", &zip_buffer);
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
            // Draw checkerboard background
            let canvas_w = self.canvas_size[0] as f32 * self.preview_zoom;
            let canvas_h = self.canvas_size[1] as f32 * self.preview_zoom;
            
            // Center the canvas in the available rect
            let available_rect = ui.available_rect_before_wrap();
            let canvas_rect = egui::Rect::from_center_size(
                available_rect.center(),
                egui::vec2(canvas_w, canvas_h)
            );

            // Draw Background (Checkerboard)
            let painter = ui.painter_at(canvas_rect);
            painter.rect_filled(canvas_rect, 0.0, egui::Color32::from_gray(50)); // Dark base

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
                         
                         painter.rect_filled(rect, 0.0, egui::Color32::from_gray(100));
                    }
                }
            }
            
            // Draw User Background Color
            if self.bg_color != egui::Color32::TRANSPARENT {
                painter.rect_filled(canvas_rect, 0.0, self.bg_color);
            }

            // Draw Parts
            // We need to iterate again to update textures if needed
            for part in &mut self.parts {
                if !part.visible { continue; }
                
                // Ensure texture exists
                let texture_id = if let Some(tex) = &part.texture {
                    tex.id()
                } else {
                    let tex = ctx.load_texture(
                        &part.name,
                        egui::ColorImage::from_rgba_unmultiplied(
                            [part.source_image.width() as _, part.source_image.height() as _],
                            part.source_image.to_rgba8().as_flat_samples().as_slice(),
                        ),
                        egui::TextureOptions::NEAREST, // CRITICAL: Nearest Neighbor for preview
                    );
                    let id = tex.id();
                    part.texture = Some(tex);
                    id
                };

                // Calculate display rect
                // Part Offset is in "Canvas Pixels".
                // Screen Position = CanvasTopLeft + (PartOffset * Zoom)
                let part_screen_pos = canvas_rect.min + (part.offset * self.preview_zoom);
                let part_w = part.source_image.width() as f32 * part.scale * self.preview_zoom;
                let part_h = part.source_image.height() as f32 * part.scale * self.preview_zoom;
                
                let part_rect = egui::Rect::from_min_size(part_screen_pos, egui::vec2(part_w, part_h));

                // Interaction: Dragging
                // We create an invisible "Sense" rect over the part to handle dragging.
                // However, we must be careful with Z-Index. egui paints back-to-front.
                // To handle selection properly, we might need a separate loop or transparent widgets.
                // Simplest way: ui.put a transparent ImageButton or Area.
                
                // Let's use `ui.allocate_rect` to get interaction, then paint.
                // Problem: allocate_rect consumes space in the layout if not careful? 
                // No, we are in a Manual Layout (Painter). But we can still interact.
                
                let interact_response = ui.interact(part_rect, egui::Id::new(part.id), egui::Sense::drag());
                
                if interact_response.dragged() {
                    // Update offset
                    // Delta is in screen pixels. Need to convert to Canvas Pixels.
                    let delta = interact_response.drag_delta() / self.preview_zoom;
                    part.offset += delta;
                    self.selected_part_id = Some(part.id);
                }
                
                if interact_response.clicked() {
                    self.selected_part_id = Some(part.id);
                }

                // Visual Highlight for Selection
                if Some(part.id) == self.selected_part_id {
                    painter.rect_stroke(part_rect, 0.0, egui::Stroke::new(2.0, egui::Color32::YELLOW));
                }

                // Paint the Texture
                let mut mesh = egui::Mesh::with_texture(texture_id);
                mesh.add_rect_with_uv(
                    part_rect, 
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), 
                    egui::Color32::WHITE
                );
                painter.add(mesh);
            }
            
            // Canvas Border
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
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcoded in index.html usually
                web_options,
                Box::new(|_cc| Ok(Box::new(KitbashApp::default()))),
            )
            .await
            .expect("failed to start eframe");
    });
}