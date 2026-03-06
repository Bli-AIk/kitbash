// Theme system for the kitbash editor.
//
// Supports built-in themes (Rerun dark, egui Dark/Light) and Catppuccin palette themes.
// Ported from bevy_workbench's theme system for use with pure eframe/egui.

#[allow(dead_code)]
pub mod colors;

use egui::{epaint::Shadow, Color32, Stroke, Vec2};

use colors::{blue, gray};

/// Available theme presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ThemePreset {
    #[default]
    Rerun,
    EguiDark,
    EguiLight,
    CatppuccinMocha,
    CatppuccinMacchiato,
    CatppuccinFrappe,
    CatppuccinLatte,
}

impl ThemePreset {
    pub const ALL: &[Self] = &[
        Self::Rerun,
        Self::EguiDark,
        Self::EguiLight,
        Self::CatppuccinMocha,
        Self::CatppuccinMacchiato,
        Self::CatppuccinFrappe,
        Self::CatppuccinLatte,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Rerun => "Rerun Dark",
            Self::EguiDark => "egui Dark",
            Self::EguiLight => "egui Light",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::CatppuccinMacchiato => "Catppuccin Macchiato",
            Self::CatppuccinFrappe => "Catppuccin Frappé",
            Self::CatppuccinLatte => "Catppuccin Latte",
        }
    }
}

/// Theme configuration stored in settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThemeConfig {
    #[serde(default)]
    pub theme: ThemePreset,
    #[serde(default = "default_brightness")]
    pub brightness: f32,
}

fn default_brightness() -> f32 {
    1.0
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            theme: ThemePreset::Rerun,
            brightness: 1.0,
        }
    }
}

/// Darken a `Color32` by a factor (0.0 = black, 1.0 = unchanged).
fn dim_color(c: Color32, factor: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (c.r() as f32 * factor) as u8,
        (c.g() as f32 * factor) as u8,
        (c.b() as f32 * factor) as u8,
        c.a(),
    )
}

fn dim_stroke(s: Stroke, factor: f32) -> Stroke {
    Stroke::new(s.width, dim_color(s.color, factor))
}

/// Apply a theme preset to an egui context.
pub fn apply_theme(ctx: &egui::Context, preset: ThemePreset, brightness: f32) {
    match preset {
        ThemePreset::Rerun => apply_rerun_theme(ctx, brightness),
        ThemePreset::EguiDark => {
            ctx.set_visuals(egui::Visuals::dark());
            apply_brightness(ctx, brightness);
        }
        ThemePreset::EguiLight => {
            ctx.set_visuals(egui::Visuals::light());
            apply_brightness(ctx, brightness);
        }
        ThemePreset::CatppuccinMocha => {
            catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);
            apply_brightness(ctx, brightness);
        }
        ThemePreset::CatppuccinMacchiato => {
            catppuccin_egui::set_theme(ctx, catppuccin_egui::MACCHIATO);
            apply_brightness(ctx, brightness);
        }
        ThemePreset::CatppuccinFrappe => {
            catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE);
            apply_brightness(ctx, brightness);
        }
        ThemePreset::CatppuccinLatte => {
            catppuccin_egui::set_theme(ctx, catppuccin_egui::LATTE);
            apply_brightness(ctx, brightness);
        }
    }
}

/// Apply brightness dimming on top of an existing style.
fn apply_brightness(ctx: &egui::Context, brightness: f32) {
    if brightness >= 1.0 {
        return;
    }
    let mut style = (*ctx.style()).clone();
    let b = brightness;
    dim_visuals(&mut style.visuals, b);
    ctx.set_style(style);
}

/// Dim all visual colors by the given factor.
fn dim_visuals(visuals: &mut egui::Visuals, b: f32) {
    visuals.faint_bg_color = dim_color(visuals.faint_bg_color, b);
    visuals.extreme_bg_color = dim_color(visuals.extreme_bg_color, b);
    visuals.panel_fill = dim_color(visuals.panel_fill, b);
    visuals.window_fill = dim_color(visuals.window_fill, b);
    visuals.selection.bg_fill = dim_color(visuals.selection.bg_fill, b);
    visuals.selection.stroke = dim_stroke(visuals.selection.stroke, b);
    visuals.hyperlink_color = dim_color(visuals.hyperlink_color, b);
    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.weak_bg_fill = dim_color(w.weak_bg_fill, b);
        w.bg_fill = dim_color(w.bg_fill, b);
        w.bg_stroke = dim_stroke(w.bg_stroke, b);
        w.fg_stroke = dim_stroke(w.fg_stroke, b);
    }
}

/// Apply the Rerun-inspired dark theme (ported from bevy_workbench).
fn apply_rerun_theme(ctx: &egui::Context, brightness: f32) {
    let mut style = (*ctx.style()).clone();
    let b = brightness;

    configure_typography(&mut style);
    configure_spacing(&mut style);
    configure_rerun_colors(&mut style, b);

    ctx.set_style(style);
}

fn configure_typography(style: &mut egui::Style) {
    let font_size = 12.0;
    for text_style in [
        egui::TextStyle::Body,
        egui::TextStyle::Monospace,
        egui::TextStyle::Button,
    ] {
        if let Some(font_id) = style.text_styles.get_mut(&text_style) {
            font_id.size = font_size;
        }
    }
    if let Some(font_id) = style.text_styles.get_mut(&egui::TextStyle::Heading) {
        font_id.size = 16.0;
    }
    if let Some(font_id) = style.text_styles.get_mut(&egui::TextStyle::Small) {
        font_id.size = 10.0;
    }
}

fn configure_spacing(style: &mut egui::Style) {
    style.spacing.interact_size.y = 15.0;
    style.visuals.button_frame = true;

    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    style.visuals.widgets.active.bg_stroke = Stroke::NONE;
    style.visuals.widgets.open.bg_stroke = Stroke::NONE;

    style.visuals.widgets.hovered.expansion = 2.0;
    style.visuals.widgets.active.expansion = 2.0;
    style.visuals.widgets.open.expansion = 2.0;

    let window_radius = egui::CornerRadius::same(6);
    let small_radius = egui::CornerRadius::same(4);
    style.visuals.window_corner_radius = window_radius;
    style.visuals.menu_corner_radius = window_radius;
    style.visuals.widgets.noninteractive.corner_radius = small_radius;
    style.visuals.widgets.inactive.corner_radius = small_radius;
    style.visuals.widgets.hovered.corner_radius = small_radius;
    style.visuals.widgets.active.corner_radius = small_radius;
    style.visuals.widgets.open.corner_radius = small_radius;

    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.menu_margin = egui::Margin::same(12);
    style.spacing.menu_spacing = 1.0;
    style.visuals.clip_rect_margin = 0.0;
    style.visuals.striped = false;
    style.visuals.indent_has_left_vline = false;
    style.spacing.button_padding = Vec2::new(1.0, 0.0);
    style.spacing.indent = 14.0;
    style.spacing.combo_width = 8.0;
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.bar_outer_margin = 2.0;
    style.spacing.tooltip_width = 600.0;
    style.visuals.image_loading_spinners = false;
}

fn configure_rerun_colors(style: &mut egui::Style, b: f32) {
    style.visuals.dark_mode = true;
    style.visuals.faint_bg_color = dim_color(gray::S150, b);
    style.visuals.extreme_bg_color = dim_color(gray::S200, b);

    style.visuals.widgets.noninteractive.weak_bg_fill = dim_color(gray::S100, b);
    style.visuals.widgets.noninteractive.bg_fill = dim_color(gray::S100, b);
    style.visuals.text_edit_bg_color = Some(dim_color(gray::S250, b));

    style.visuals.widgets.inactive.weak_bg_fill = dim_color(gray::S250, b);
    style.visuals.widgets.inactive.bg_fill = dim_color(gray::S300, b);

    let hovered = dim_color(gray::S325, b);
    style.visuals.widgets.hovered.weak_bg_fill = hovered;
    style.visuals.widgets.hovered.bg_fill = hovered;
    style.visuals.widgets.active.weak_bg_fill = hovered;
    style.visuals.widgets.active.bg_fill = hovered;
    style.visuals.widgets.open.weak_bg_fill = hovered;
    style.visuals.widgets.open.bg_fill = hovered;

    style.visuals.selection.bg_fill = dim_color(blue::S350, b);
    style.visuals.selection.stroke.color = dim_color(blue::S900, b);

    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, dim_color(gray::S250, b));

    let subdued = dim_color(gray::S550, b);
    let default_text = dim_color(gray::S775, b);
    let strong = dim_color(gray::S1000, b);

    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, subdued);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, default_text);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, strong);
    style.visuals.widgets.active.fg_stroke = Stroke::new(2.0, strong);
    style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, default_text);

    style.visuals.selection.stroke = dim_stroke(Stroke::new(2.0, blue::S900), b);

    let shadow = Shadow {
        offset: [0, 15],
        blur: 50,
        spread: 0,
        color: Color32::from_black_alpha(128),
    };
    style.visuals.popup_shadow = shadow;
    style.visuals.window_shadow = shadow;

    style.visuals.window_fill = dim_color(gray::S200, b);
    style.visuals.window_stroke = Stroke::NONE;
    style.visuals.panel_fill = dim_color(gray::S100, b);

    style.visuals.hyperlink_color = default_text;
    style.visuals.error_fg_color = dim_color(Color32::from_rgb(0xAB, 0x01, 0x16), b);
    style.visuals.warn_fg_color = dim_color(Color32::from_rgb(0xFF, 0x7A, 0x0C), b);
}
