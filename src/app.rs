// Main application state and eframe::App implementation.

use std::sync::mpsc::{channel, Receiver, Sender};

use crate::config::{ConfigPath, EditorSettings};
use crate::node::graph::NodeGraph;
use crate::theme::{self, ThemeConfig};
use crate::ui::console_panel::ConsoleState;
use crate::ui::dock::TileLayoutState;
use crate::ui::menu_bar::MenuAction;
use crate::ui::node_graph_panel::NodeGraphPanel;

// ─── Data types ─────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Transform {
    pub offset: egui::Vec2,
    pub scale: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            offset: egui::Vec2::ZERO,
            scale: 1.0,
        }
    }
}

pub struct LayerImage {
    pub id: u64,
    pub name: String,
    pub source_image: image::DynamicImage,
    pub texture: Option<egui::TextureHandle>,
    pub transform: Transform,
    pub visible: bool,
}

pub enum AppMessage {
    ImageLoaded(String, Vec<u8>),
}

// ─── App state ──────────────────────────────────────────────────────

pub struct KitbashApp {
    // Canvas config
    pub canvas_size: [u32; 2],
    pub bg_color: egui::Color32,
    pub export_scale: u32,

    // Legacy layer state
    pub layers: Vec<LayerImage>,
    pub selected_layer_id: Option<u64>,
    next_id: u64,

    // Async messaging
    pub msg_sender: Sender<AppMessage>,
    msg_receiver: Receiver<AppMessage>,

    // UI state
    pub preview_zoom: f32,
    pub canvas_pan: egui::Vec2,

    // Dock layout
    pub tile_state: TileLayoutState,

    // Theme
    pub theme_config: ThemeConfig,
    theme_applied: bool,

    // Settings
    settings: EditorSettings,
    config_path: ConfigPath,

    // Node graph
    pub node_graph: NodeGraph,
    pub node_graph_panel: NodeGraphPanel,

    // Console
    pub console: ConsoleState,
}

impl Default for KitbashApp {
    fn default() -> Self {
        let (sender, receiver) = channel();
        let config_path = ConfigPath::default();
        let settings = EditorSettings::load(&config_path.0);

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
            tile_state: TileLayoutState::default(),
            theme_config: settings.theme.clone(),
            theme_applied: false,
            settings,
            config_path,
            node_graph: NodeGraph::default(),
            node_graph_panel: NodeGraphPanel::default(),
            console: ConsoleState {
                auto_scroll: true,
                ..Default::default()
            },
        }
    }
}

// ─── eframe::App impl ──────────────────────────────────────────────

impl eframe::App for KitbashApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme_if_needed(ctx, &self.theme_config, &mut self.theme_applied);
        process_messages(self);

        let action = crate::ui::menu_bar::show_menu_bar(ctx, &self.theme_config, &self.tile_state);
        if let Some(action) = action {
            handle_menu_action(self, ctx, action);
        }

        crate::ui::dock::show_dock(self, ctx);
    }
}

fn apply_theme_if_needed(ctx: &egui::Context, config: &ThemeConfig, applied: &mut bool) {
    if !*applied {
        theme::apply_theme(ctx, config.theme, config.brightness);
        *applied = true;
    }
}

fn process_messages(app: &mut KitbashApp) {
    while let Ok(msg) = app.msg_receiver.try_recv() {
        match msg {
            AppMessage::ImageLoaded(name, bytes) => {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let id = app.next_id;
                    app.next_id += 1;
                    app.layers.push(LayerImage {
                        id,
                        name,
                        source_image: img,
                        texture: None,
                        transform: Transform::default(),
                        visible: true,
                    });
                } else {
                    log::error!("Failed to decode image: {name}");
                }
            }
        }
    }
}

fn handle_menu_action(app: &mut KitbashApp, ctx: &egui::Context, action: MenuAction) {
    match action {
        MenuAction::ImportImages => {
            trigger_import(app);
        }
        MenuAction::ExportPng => {
            crate::imaging::export::export_individual_pngs(
                &app.layers,
                app.canvas_size,
                app.export_scale,
            );
        }
        MenuAction::ExportZip => {
            crate::imaging::export::export_zip(&app.layers, app.canvas_size, app.export_scale);
        }
        MenuAction::ThemeChanged(config) => {
            app.theme_config = config.clone();
            app.theme_applied = false;
            app.settings.theme = config;
            app.settings.save(&app.config_path.0);
            theme::apply_theme(ctx, app.theme_config.theme, app.theme_config.brightness);
            app.theme_applied = true;
        }
        MenuAction::TogglePanel(panel_id) => {
            app.tile_state.toggle_panel(&panel_id);
        }
        MenuAction::ResetLayout => {
            app.tile_state = TileLayoutState::default();
        }
    }
}

fn trigger_import(app: &mut KitbashApp) {
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
    std::thread::spawn(move || {
        futures::executor::block_on(task);
    });
}
