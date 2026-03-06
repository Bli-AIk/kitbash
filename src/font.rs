// CJK font management.
//
// Embeds Source Han Sans CN as the default CJK font and provides
// configuration to override with a custom font file.

/// Embedded CJK font (Source Han Sans CN Regular, ~8 MB).
const EMBEDDED_CJK_FONT: &[u8] = include_bytes!("../fonts/SourceHanSansCN-Regular.otf");

/// Font configuration stored in settings.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FontConfig {
    /// Optional path to a custom font file. When set, this font is used
    /// instead of the embedded CJK font.
    #[serde(default)]
    pub custom_font_path: Option<String>,
}

/// Whether fonts have been installed into the egui context.
#[derive(Default)]
pub struct FontState {
    pub installed: bool,
}

/// Install CJK font into the egui context (call once at startup or after config change).
pub fn install_fonts(ctx: &egui::Context, config: &FontConfig, state: &mut FontState) {
    if state.installed {
        return;
    }

    let font_data = load_font_data(config);

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "cjk".to_owned(),
        egui::FontData::from_owned(font_data).into(),
    );
    // Append CJK as fallback for Proportional family
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("cjk".to_owned());
    // Also add as fallback for Monospace
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("cjk".to_owned());

    ctx.set_fonts(fonts);
    state.installed = true;
    log::info!("CJK font installed into egui context");
}

fn load_font_data(config: &FontConfig) -> Vec<u8> {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(ref path) = config.custom_font_path {
        match std::fs::read(path) {
            Ok(data) => {
                log::info!("Loaded custom font from: {path}");
                return data;
            }
            Err(e) => {
                log::warn!("Failed to load custom font '{path}': {e}, using embedded CJK font");
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    let _ = config;

    EMBEDDED_CJK_FONT.to_vec()
}
