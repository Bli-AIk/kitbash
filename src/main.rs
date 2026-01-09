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

struct LayerGroup {
    id: u64,
    name: String,
    children: Vec<LayerNode>,
    transform: Transform,
    visible: bool,
}

enum LayerNode {
    Image(LayerImage),
    Group(LayerGroup),
}

impl LayerNode {
    fn id(&self) -> u64 {
        match self {
            LayerNode::Image(img) => img.id,
            LayerNode::Group(grp) => grp.id,
        }
    }

    fn name(&self) -> &str {
        match self {
            LayerNode::Image(img) => &img.name,
            LayerNode::Group(grp) => &grp.name,
        }
    }

    fn visible(&self) -> bool {
        match self {
            LayerNode::Image(img) => img.visible,
            LayerNode::Group(grp) => grp.visible,
        }
    }

    fn set_visible(&mut self, visible: bool) {
        match self {
            LayerNode::Image(img) => img.visible = visible,
            LayerNode::Group(grp) => grp.visible = visible,
        }
    }

    fn transform_mut(&mut self) -> &mut Transform {
        match self {
            LayerNode::Image(img) => &mut img.transform,
            LayerNode::Group(grp) => &mut grp.transform,
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
    root_layers: Vec<LayerNode>,
    selected_layer_id: Option<u64>,
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
            root_layers: Vec::new(),
            selected_layer_id: None,
            next_id: 0,
            msg_sender: sender,
            msg_receiver: receiver,
            preview_zoom: 4.0,
        }
    }
}

// ----------------------------------------------------------------------------
// Helper Functions
// ----------------------------------------------------------------------------

struct RenderItem<'a> {
    image: &'a image::DynamicImage,
    texture: &'a mut Option<egui::TextureHandle>,
    // Absolute transform (accumulated)
    pos: egui::Vec2,
    scale: f32,
    id: u64,
    name: &'a str,
}

/// Recursively flatten the layer tree into a render list, accumulating transforms
fn flatten_layers<'a>(
    nodes: &'a mut [LayerNode], 
    parent_offset: egui::Vec2, 
    parent_scale: f32,
    items: &mut Vec<RenderItem<'a>>
) {
    for node in nodes {
        if !node.visible() {
            continue;
        }

        match node {
            LayerNode::Image(img) => {
                // Apply parent transform then local transform
                // Position: ParentPos + (LocalPos * ParentScale)
                // Scale: ParentScale * LocalScale
                let abs_scale = parent_scale * img.transform.scale;
                let abs_offset = parent_offset + (img.transform.offset * parent_scale);
                
                items.push(RenderItem {
                    image: &img.source_image,
                    texture: &mut img.texture,
                    pos: abs_offset,
                    scale: abs_scale,
                    id: img.id,
                    name: &img.name,
                });
            }
            LayerNode::Group(grp) => {
                let abs_scale = parent_scale * grp.transform.scale;
                let abs_offset = parent_offset + (grp.transform.offset * parent_scale);
                
                flatten_layers(&mut grp.children, abs_offset, abs_scale, items);
            }
        }
    }
}

/// Composite the final image based on current state
fn composite_image(canvas_size: [u32; 2], bg_color: egui::Color32, layers: &mut [LayerNode]) -> RgbaImage {
    let mut buffer = RgbaImage::new(canvas_size[0], canvas_size[1]);
    
    // Fill background
    for pixel in buffer.pixels_mut() {
        *pixel = Rgba([bg_color.r(), bg_color.g(), bg_color.b(), bg_color.a()]);
    }

    let mut items = Vec::new();
    flatten_layers(layers, egui::Vec2::ZERO, 1.0, &mut items);

    for item in items {
        let src_width = item.image.width();
        let src_height = item.image.height();
        
        let target_width = (src_width as f32 * item.scale).round() as u32;
        let target_height = (src_height as f32 * item.scale).round() as u32;

        if target_width == 0 || target_height == 0 {
            continue;
        }

        let resized = item.image.resize_exact(
            target_width, 
            target_height, 
            FilterType::Nearest
        );

        // Pixel-Perfect Integer Alignment
        let x = item.pos.x.round() as i64;
        let y = item.pos.y.round() as i64;

        image::imageops::overlay(&mut buffer, &resized, x, y);
    }
    
    buffer
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
// Logic Helpers for Tree Mutation
// ----------------------------------------------------------------------------

fn find_layer_mut<'a>(layers: &'a mut [LayerNode], id: u64) -> Option<&'a mut LayerNode> {
    for node in layers {
        if node.id() == id {
            return Some(node);
        }
        if let LayerNode::Group(grp) = node {
            if let Some(found) = find_layer_mut(&mut grp.children, id) {
                return Some(found);
            }
        }
    }
    None
}

fn delete_layer(layers: &mut Vec<LayerNode>, id: u64) -> bool {
    if let Some(idx) = layers.iter().position(|x| x.id() == id) {
        layers.remove(idx);
        return true;
    }
    for node in layers {
        if let LayerNode::Group(grp) = node {
            if delete_layer(&mut grp.children, id) {
                return true;
            }
        }
    }
    false
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
                        
                        // Logic to add to selected group or root
                        let mut target_group_id = None;
                        
                        if let Some(sel_id) = self.selected_layer_id {
                            // First pass: check if selected node is a group
                            // We can't hold mutable reference to self.root_layers while checking
                            // So we just find the ID
                            if let Some(node) = find_layer_mut(&mut self.root_layers, sel_id) {
                                if let LayerNode::Group(_) = node {
                                    target_group_id = Some(sel_id);
                                }
                            }
                        }

                        if let Some(grp_id) = target_group_id {
                            if let Some(node) = find_layer_mut(&mut self.root_layers, grp_id) {
                                if let LayerNode::Group(grp) = node {
                                    grp.children.push(LayerNode::Image(layer));
                                } else {
                                    // Should not happen given logic above, but fallback
                                    self.root_layers.push(LayerNode::Image(layer));
                                }
                            } else {
                                self.root_layers.push(LayerNode::Image(layer));
                            }
                        } else {
                            self.root_layers.push(LayerNode::Image(layer));
                        }
                        
                        // Select the new layer? Maybe just keep it simple.
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
                ui.heading("Assets & Layers");
                ui.horizontal(|ui| {
                    if ui.button("Import Images...").clicked() {
                        let sender = app.msg_sender.clone();
                        let task = async move {
                            if let Some(handles) = rfd::AsyncFileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                                .pick_files() // BATCH IMPORT
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
                    
                    if ui.button("New Folder").clicked() {
                        let id = app.next_id;
                        app.next_id += 1;
                        let folder = LayerGroup {
                            id,
                            name: format!("Folder {}", id),
                            children: Vec::new(),
                            transform: Transform::default(),
                            visible: true,
                        };
                        app.root_layers.push(LayerNode::Group(folder));
                    }
                });

                ui.separator();
                ui.label("Layers Tree:");
                
                // Recursive Tree UI
                fn draw_tree(
                    ui: &mut egui::Ui, 
                    layers: &mut Vec<LayerNode>, 
                    selected_id: &mut Option<u64>,
                    to_delete: &mut Option<u64>
                ) {
                    let mut move_op = None; // (index, direction -1 or +1)
                    let layers_len = layers.len();

                    for (idx, node) in layers.iter_mut().enumerate() {
                        let node_id = node.id();
                        let is_selected = Some(node_id) == *selected_id;
                        
                        ui.horizontal(|ui| {
                            // Indentation handled by recursion? No, by ui.indent or collapsing header
                            
                            // Checkbox for visibility
                            let mut visible = node.visible();
                            if ui.checkbox(&mut visible, "").changed() {
                                node.set_visible(visible);
                            }

                            match node {
                                LayerNode::Group(grp) => {
                                    let id = ui.make_persistent_id(node_id);
                                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
                                        .show_header(ui, |ui| {
                                            if ui.selectable_label(is_selected, &grp.name).clicked() {
                                                *selected_id = Some(node_id);
                                            }
                                        })
                                        .body(|ui| {
                                            draw_tree(ui, &mut grp.children, selected_id, to_delete);
                                        });
                                }
                                LayerNode::Image(img) => {
                                    if ui.selectable_label(is_selected, &img.name).clicked() {
                                        *selected_id = Some(node_id);
                                    }
                                }
                            }
                            
                            // Reorder buttons (local scope)
                            if ui.button("⬆").clicked() && idx > 0 {
                                move_op = Some((idx, -1));
                            }
                            if ui.button("⬇").clicked() && idx < layers_len - 1 {
                                move_op = Some((idx, 1));
                            }
                            
                            if ui.button("X").clicked() {
                                *to_delete = Some(node_id);
                            }
                        });
                    }

                    if let Some((idx, dir)) = move_op {
                        if dir == -1 {
                            layers.swap(idx, idx - 1);
                        } else {
                            layers.swap(idx, idx + 1);
                        }
                    }
                }

                let mut to_delete = None;
                draw_tree(ui, &mut app.root_layers, &mut app.selected_layer_id, &mut to_delete);
                
                if let Some(del_id) = to_delete {
                    delete_layer(&mut app.root_layers, del_id);
                    if app.selected_layer_id == Some(del_id) {
                        app.selected_layer_id = None;
                    }
                }

                ui.separator();
                
                // Properties Panel
                if let Some(selected_id) = app.selected_layer_id {
                    if let Some(node) = find_layer_mut(&mut app.root_layers, selected_id) {
                        ui.heading(format!("Properties: {}", node.name()));
                        let transform = node.transform_mut();
                        
                        ui.horizontal(|ui| {
                            ui.label("Scale:");
                            ui.add(egui::Slider::new(&mut transform.scale, 0.1..=5.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Offset:");
                            // PIXEL PERFECT: Step by 1.0
                            ui.add(egui::DragValue::new(&mut transform.offset.x).speed(1.0).prefix("X: "));
                            ui.add(egui::DragValue::new(&mut transform.offset.y).speed(1.0).prefix("Y: "));
                        });
                        
                        // Rounding button for convenience
                        if ui.button("Snap to Pixel").clicked() {
                            transform.offset.x = transform.offset.x.round();
                            transform.offset.y = transform.offset.y.round();
                        }
                        
                        if ui.button("Reset Transform").clicked() {
                            transform.scale = 1.0;
                            transform.offset = egui::Vec2::ZERO;
                        }
                    }
                } else {
                    ui.label("Select a layer or folder to edit properties.");
                }
                
                ui.separator();
                
                // Export System
                ui.heading("Export");
                ui.horizontal(|ui| {
                    if ui.button("Download PNG").clicked() {
                        let img = composite_image(app.canvas_size, app.bg_color, &mut app.root_layers);
                        let mut bytes: Vec<u8> = Vec::new();
                        img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png).unwrap();
                        trigger_download("character.png", &bytes);
                    }
                    
                    if ui.button("Download ZIP").clicked() {
                        let img = composite_image(app.canvas_size, app.bg_color, &mut app.root_layers);
                        let mut png_bytes: Vec<u8> = Vec::new();
                        img.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png).unwrap();
                        
                        // Metadata generation is complex with tree, let's just dump a simple structure or skip for now?
                        // Requirement said "data.json needed". Let's do a simplified flat dump of render state.
                        let mut items = Vec::new();
                        // Reset layers for metadata capture if needed, but we can just use the helper
                        flatten_layers(&mut app.root_layers, egui::Vec2::ZERO, 1.0, &mut items);
                        
                        let meta: Vec<serde_json::Value> = items.iter().map(|item| {
                            serde_json::json!({
                                "name": item.name,
                                "scale": item.scale,
                                "offset": { "x": item.pos.x.round(), "y": item.pos.y.round() },
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

            // Flatten layers for rendering
            let mut items = Vec::new();
            flatten_layers(&mut self.root_layers, egui::Vec2::ZERO, 1.0, &mut items);

            let mut drag_events = Vec::new(); // Collect drag events to apply later

            for item in items {
                // Ensure texture exists
                let texture_id = if let Some(tex) = item.texture {
                    tex.id()
                } else {
                    let tex = ctx.load_texture(
                        item.name,
                        egui::ColorImage::from_rgba_unmultiplied(
                            [item.image.width() as _, item.image.height() as _],
                            item.image.to_rgba8().as_flat_samples().as_slice(),
                        ),
                        egui::TextureOptions::NEAREST, // CRITICAL: Nearest Neighbor
                    );
                    let id = tex.id();
                    *item.texture = Some(tex);
                    id
                };

                // Calculate display rect
                // Position is absolute (relative to canvas 0,0)
                // Pixel Perfect Alignment for Preview: Round the position
                let aligned_pos = egui::pos2(item.pos.x.round(), item.pos.y.round());
                
                let part_screen_pos = canvas_rect.min + (aligned_pos.to_vec2() * self.preview_zoom);
                let part_w = item.image.width() as f32 * item.scale * self.preview_zoom;
                let part_h = item.image.height() as f32 * item.scale * self.preview_zoom;
                
                let part_rect = egui::Rect::from_min_size(part_screen_pos, egui::vec2(part_w, part_h));

                // Interaction: Dragging
                let interact_response = ui.interact(part_rect, egui::Id::new(item.id), egui::Sense::drag());
                
                if interact_response.dragged() {
                    let delta = interact_response.drag_delta() / self.preview_zoom;
                    drag_events.push((item.id, delta));
                    self.selected_layer_id = Some(item.id);
                }
                
                if interact_response.clicked() {
                    self.selected_layer_id = Some(item.id);
                }

                // Visual Highlight for Selection
                if Some(item.id) == self.selected_layer_id {
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
            
            // Apply deferred drag events
            for (id, delta) in drag_events {
                if let Some(node) = find_layer_mut(&mut self.root_layers, id) {
                    node.transform_mut().offset += delta;
                }
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