// Settings panel — editor preferences (theme, scale, font).

use crate::theme::ThemePreset;

/// Settings panel state with edited copies of settings values.
pub struct SettingsPanel {
    pub edited_scale: f32,
    pub edited_theme: ThemePreset,
    pub edited_brightness: f32,
    pub edited_font_path: Option<String>,
    pub save_requested: bool,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {
            edited_scale: 1.0,
            edited_theme: ThemePreset::default(),
            edited_brightness: 1.0,
            edited_font_path: None,
            save_requested: false,
        }
    }
}

impl SettingsPanel {
    /// Sync from current app settings (e.g. on first open).
    pub fn sync_from(&mut self, settings: &crate::config::EditorSettings) {
        self.edited_scale = settings.ui_scale;
        self.edited_theme = settings.theme.theme;
        self.edited_brightness = settings.theme.brightness;
        self.edited_font_path = settings.font.custom_font_path.clone();
    }
}

/// Draw the settings panel.
pub fn draw_settings(ui: &mut egui::Ui, panel: &mut SettingsPanel) {
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            settings_ui(ui, panel);
        });
}

fn settings_ui(ui: &mut egui::Ui, panel: &mut SettingsPanel) {
    ui.heading("Editor Settings");
    ui.separator();

    egui::Grid::new("settings_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label("UI Scale:");
            ui.add(egui::Slider::new(&mut panel.edited_scale, 0.5..=2.0).step_by(0.25));
            ui.end_row();

            ui.label("Theme:");
            egui::ComboBox::from_id_salt("theme_select")
                .selected_text(panel.edited_theme.label())
                .show_ui(ui, |ui| {
                    for preset in ThemePreset::ALL {
                        ui.selectable_value(&mut panel.edited_theme, *preset, preset.label());
                    }
                });
            ui.end_row();

            ui.label("Brightness:");
            ui.add(egui::Slider::new(&mut panel.edited_brightness, 0.2..=1.0).step_by(0.05));
            ui.end_row();

            #[cfg(not(target_arch = "wasm32"))]
            font_picker(ui, panel);
        });

    ui.separator();
    if ui.button("Save").clicked() {
        panel.save_requested = true;
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn font_picker(ui: &mut egui::Ui, panel: &mut SettingsPanel) {
    ui.label("Custom Font:");
    let display = panel.edited_font_path.as_deref().unwrap_or("(embedded)");
    if ui.button(display).clicked() {
        let picked = rfd::FileDialog::new()
            .add_filter("Font", &["otf", "ttf", "ttc"])
            .pick_file();
        if let Some(path) = picked {
            panel.edited_font_path = Some(path.display().to_string());
        }
    }
    ui.end_row();
}
