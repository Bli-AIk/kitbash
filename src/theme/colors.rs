// Rerun-inspired color palette constants.
//
// Ported from bevy_workbench's theme system to provide a consistent
// dark editor aesthetic.

use egui::Color32;

/// Grayscale palette (Rerun dark theme).
pub mod gray {
    use egui::Color32;
    pub const S0: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
    pub const S100: Color32 = Color32::from_rgb(0x0d, 0x10, 0x11);
    pub const S125: Color32 = Color32::from_rgb(0x11, 0x14, 0x15);
    pub const S150: Color32 = Color32::from_rgb(0x14, 0x18, 0x19);
    pub const S200: Color32 = Color32::from_rgb(0x1c, 0x21, 0x23);
    pub const S250: Color32 = Color32::from_rgb(0x26, 0x2b, 0x2e);
    pub const S300: Color32 = Color32::from_rgb(0x31, 0x38, 0x3b);
    pub const S325: Color32 = Color32::from_rgb(0x37, 0x3f, 0x42);
    pub const S350: Color32 = Color32::from_rgb(0x3e, 0x46, 0x4a);
    pub const S500: Color32 = Color32::from_rgb(0x6c, 0x79, 0x7f);
    pub const S550: Color32 = Color32::from_rgb(0x7d, 0x8c, 0x92);
    pub const S700: Color32 = Color32::from_rgb(0xae, 0xc2, 0xca);
    pub const S775: Color32 = Color32::from_rgb(0xca, 0xd8, 0xde);
    pub const S800: Color32 = Color32::from_rgb(0xd3, 0xde, 0xe3);
    pub const S1000: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
}

/// Blue accent palette (Rerun dark theme).
pub mod blue {
    use egui::Color32;
    pub const S350: Color32 = Color32::from_rgb(0x00, 0x3d, 0xa1);
    pub const S400: Color32 = Color32::from_rgb(0x00, 0x4b, 0xc2);
    pub const S450: Color32 = Color32::from_rgb(0x00, 0x5a, 0xe6);
    pub const S500: Color32 = Color32::from_rgb(0x2a, 0x6c, 0xff);
    pub const S750: Color32 = Color32::from_rgb(0xc2, 0xcc, 0xff);
    pub const S900: Color32 = Color32::from_rgb(0xf0, 0xf2, 0xff);
}

// Semantic color aliases
pub const PANEL_BG: Color32 = gray::S100;
pub const HEADER_BG: Color32 = gray::S150;
pub const ROW_EVEN_BG: Color32 = gray::S100;
pub const ROW_ODD_BG: Color32 = gray::S125;
pub const ROW_SELECTED_BG: Color32 = Color32::from_rgb(0x00, 0x25, 0x69);
pub const BAR_COLOR: Color32 = blue::S400;
pub const SEPARATOR_COLOR: Color32 = gray::S250;
pub const TEXT_SUBDUED: Color32 = gray::S550;
pub const TEXT_DEFAULT: Color32 = gray::S775;
pub const TEXT_STRONG: Color32 = gray::S1000;
