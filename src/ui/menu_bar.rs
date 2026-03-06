// Top menu bar with File / Edit / View menus.

use crate::theme::{ThemeConfig, ThemePreset};
use crate::ui::dock::TileLayoutState;

/// Actions triggered by the menu bar.
#[derive(Debug)]
pub enum MenuAction {
    ImportImages,
    ExportPng,
    ExportZip,
    ThemeChanged(ThemeConfig),
    TogglePanel(String),
    ResetLayout,
}

/// Draw the top menu bar. Returns any triggered action.
pub fn show_menu_bar(
    ctx: &egui::Context,
    theme_config: &ThemeConfig,
    tile_state: &TileLayoutState,
) -> Option<MenuAction> {
    let mut action = None;

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            action = file_menu(ui);
            if action.is_none() {
                action = view_menu(ui, theme_config, tile_state);
            }
        });
    });

    action
}

fn file_menu(ui: &mut egui::Ui) -> Option<MenuAction> {
    let mut action = None;
    ui.menu_button("File", |ui| {
        if ui.button("Import Images...").clicked() {
            action = Some(MenuAction::ImportImages);
            ui.close();
        }
        ui.separator();
        if ui.button("Export PNGs").clicked() {
            action = Some(MenuAction::ExportPng);
            ui.close();
        }
        if ui.button("Export ZIP").clicked() {
            action = Some(MenuAction::ExportZip);
            ui.close();
        }
    });
    action
}

fn view_menu(
    ui: &mut egui::Ui,
    theme_config: &ThemeConfig,
    tile_state: &TileLayoutState,
) -> Option<MenuAction> {
    let mut action = None;
    ui.menu_button("View", |ui| {
        action = theme_submenu(ui, theme_config);

        ui.separator();

        for (str_id, title, visible) in tile_state.panel_visibility() {
            if ui.selectable_label(visible, &title).clicked() {
                action = Some(MenuAction::TogglePanel(str_id));
                ui.close();
            }
        }

        ui.separator();
        if ui.button("Reset Layout").clicked() {
            action = Some(MenuAction::ResetLayout);
            ui.close();
        }
    });
    action
}

fn theme_submenu(ui: &mut egui::Ui, theme_config: &ThemeConfig) -> Option<MenuAction> {
    let mut action = None;
    ui.menu_button("Theme", |ui| {
        for preset in ThemePreset::ALL {
            let is_current = *preset == theme_config.theme;
            if ui.selectable_label(is_current, preset.label()).clicked() {
                action = Some(MenuAction::ThemeChanged(ThemeConfig {
                    theme: *preset,
                    brightness: theme_config.brightness,
                }));
                ui.close();
            }
        }
    });
    action
}
