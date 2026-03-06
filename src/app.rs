// Main application state and eframe::App implementation.

use std::sync::mpsc::{channel, Receiver, Sender};

use crate::config::{ConfigPath, EditorSettings};
use crate::font::FontState;
use crate::theme::{self, ThemeConfig};
use crate::ui::console_panel::ConsoleState;
use crate::ui::dock::TileLayoutState;
use crate::ui::menu_bar::MenuAction;
use crate::ui::node_graph_panel::NodeGraphPanel;
use crate::ui::settings_panel::SettingsPanel;

// ─── Image store ────────────────────────────────────────────────────

/// An imported image available for use in the node graph.
pub struct ImportedImage {
    pub id: u64,
    pub name: String,
    pub image: image::DynamicImage,
    pub texture: Option<egui::TextureHandle>,
}

/// Stores all imported images, referenced by nodes via ID.
pub struct ImageStore {
    pub images: Vec<ImportedImage>,
    next_id: u64,
}

impl Default for ImageStore {
    fn default() -> Self {
        Self {
            images: Vec::new(),
            next_id: 1,
        }
    }
}

impl ImageStore {
    pub fn add(&mut self, name: String, image: image::DynamicImage) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.images.push(ImportedImage {
            id,
            name,
            image,
            texture: None,
        });
        id
    }
}

// ─── Messages ───────────────────────────────────────────────────────

pub enum AppMessage {
    ImageLoaded(String, Vec<u8>),
}

// ─── App state ──────────────────────────────────────────────────────

pub struct KitbashApp {
    // Image store (replaces legacy layers)
    pub image_store: ImageStore,

    // Async messaging
    pub msg_sender: Sender<AppMessage>,
    msg_receiver: Receiver<AppMessage>,

    // Preview state
    pub preview_zoom: f32,
    pub canvas_pan: egui::Vec2,
    pub preview_auto_fit: bool,

    // Dock layout
    pub tile_state: TileLayoutState,

    // Theme
    pub theme_config: ThemeConfig,
    theme_applied: bool,
    scale_applied: bool,

    // Settings
    settings: EditorSettings,
    config_path: ConfigPath,

    // Settings panel
    pub settings_panel: SettingsPanel,

    // Font
    font_state: FontState,

    // Node graph
    pub node_graph_panel: NodeGraphPanel,

    // Console
    pub console: ConsoleState,
}

impl Default for KitbashApp {
    fn default() -> Self {
        let (sender, receiver) = channel();
        let config_path = ConfigPath::default();
        let settings = EditorSettings::load(&config_path.0);

        let mut settings_panel = SettingsPanel::default();
        settings_panel.sync_from(&settings);

        Self {
            image_store: ImageStore::default(),
            msg_sender: sender,
            msg_receiver: receiver,
            preview_zoom: 1.0,
            canvas_pan: egui::Vec2::ZERO,
            preview_auto_fit: true,
            tile_state: TileLayoutState::default(),
            theme_config: settings.theme.clone(),
            theme_applied: false,
            scale_applied: false,
            settings_panel,
            font_state: FontState::default(),
            settings,
            config_path,
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
        crate::font::install_fonts(ctx, &self.settings.font, &mut self.font_state);
        apply_theme_if_needed(ctx, &self.theme_config, &mut self.theme_applied);
        apply_scale_if_needed(ctx, self.settings.ui_scale, &mut self.scale_applied);
        process_messages(self);
        process_settings_save(self, ctx);

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

fn apply_scale_if_needed(ctx: &egui::Context, ui_scale: f32, applied: &mut bool) {
    if !*applied {
        ctx.set_pixels_per_point(ui_scale);
        *applied = true;
    }
}

fn process_messages(app: &mut KitbashApp) {
    while let Ok(msg) = app.msg_receiver.try_recv() {
        match msg {
            AppMessage::ImageLoaded(name, bytes) => {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let id = app.image_store.add(name, img);
                    add_image_to_input_node(&mut app.node_graph_panel, id);
                    app.preview_auto_fit = true;
                } else {
                    log::error!("Failed to decode image: {name}");
                }
            }
        }
    }
}

/// Register a new image ID in the fixed ImageInput node.
fn add_image_to_input_node(panel: &mut NodeGraphPanel, image_id: u64) {
    let node = &mut panel.snarl[panel.input_node];
    if let crate::ui::node_graph_panel::KitbashNode::ImageInput { image_ids } = node {
        image_ids.push(image_id);
    }
}

fn process_settings_save(app: &mut KitbashApp, ctx: &egui::Context) {
    if !app.settings_panel.save_requested {
        return;
    }
    app.settings_panel.save_requested = false;

    app.settings.ui_scale = app.settings_panel.edited_scale;
    app.settings.theme = ThemeConfig {
        theme: app.settings_panel.edited_theme,
        brightness: app.settings_panel.edited_brightness,
    };

    if app.settings.font.custom_font_path != app.settings_panel.edited_font_path {
        app.settings.font.custom_font_path = app.settings_panel.edited_font_path.clone();
        app.font_state.installed = false;
    }

    app.theme_config = app.settings.theme.clone();
    theme::apply_theme(ctx, app.theme_config.theme, app.theme_config.brightness);
    app.theme_applied = true;

    ctx.set_pixels_per_point(app.settings.ui_scale);
    app.settings.save(&app.config_path.0);
}

fn handle_menu_action(app: &mut KitbashApp, ctx: &egui::Context, action: MenuAction) {
    match action {
        MenuAction::ImportImages => {
            trigger_import(app);
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

/// Public wrapper for triggering image import (used by inspector).
pub fn trigger_import_public(app: &mut KitbashApp) {
    trigger_import(app);
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
