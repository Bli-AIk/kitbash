// Plugin loader — discovers and loads WASM plugin files.

use std::path::{Path, PathBuf};

/// Information about a discovered plugin file.
#[derive(Debug, Clone)]
pub struct PluginFile {
    pub path: PathBuf,
    pub name: String,
}

/// Scan a directory for `.wasm` plugin files.
pub fn scan_plugins(dir: &Path) -> Vec<PluginFile> {
    let mut plugins = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return plugins;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "wasm") {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            plugins.push(PluginFile { path, name });
        }
    }
    plugins
}
