// Persistent editor configuration stored as TOML.

use std::path::PathBuf;

use crate::theme::ThemeConfig;

/// Persistent editor settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditorSettings {
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
    #[serde(default)]
    pub theme: ThemeConfig,
}

fn default_ui_scale() -> f32 {
    1.0
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            theme: ThemeConfig::default(),
        }
    }
}

impl EditorSettings {
    /// Load from a TOML file, or return defaults if not found.
    pub fn load(path: &std::path::Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                log::warn!("Failed to parse {}: {e}", path.display());
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    /// Save to a TOML file.
    pub fn save(&self, path: &std::path::Path) {
        let content = toml::to_string_pretty(self).expect("serialize EditorSettings");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(path, content) {
            log::warn!("Failed to save config to {}: {e}", path.display());
        }
    }
}

/// Resource holding the config file path.
pub struct ConfigPath(pub PathBuf);

impl Default for ConfigPath {
    fn default() -> Self {
        Self(PathBuf::from(".kitbash/settings.toml"))
    }
}
