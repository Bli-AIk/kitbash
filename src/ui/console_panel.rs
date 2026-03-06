// Console panel — displays log messages.

/// A log entry for display in the console.
#[derive(Clone)]
pub struct LogEntry {
    pub level: log::Level,
    pub message: String,
}

/// Console state holding log entries.
#[derive(Default)]
pub struct ConsoleState {
    pub entries: Vec<LogEntry>,
    #[allow(dead_code)]
    pub filter_level: Option<log::Level>,
    pub auto_scroll: bool,
}

impl ConsoleState {
    #[allow(dead_code)]
    pub fn push(&mut self, level: log::Level, message: String) {
        self.entries.push(LogEntry { level, message });
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Draw the console with full state access.
pub fn draw_console(ui: &mut egui::Ui, state: &mut ConsoleState) {
    ui.horizontal(|ui| {
        if ui.button("Clear").clicked() {
            state.clear();
        }
        ui.checkbox(&mut state.auto_scroll, "Auto-scroll");
    });

    ui.separator();

    let scroll = egui::ScrollArea::vertical().stick_to_bottom(state.auto_scroll);
    scroll.show(ui, |ui| {
        for entry in &state.entries {
            let color = match entry.level {
                log::Level::Error => egui::Color32::from_rgb(0xAB, 0x01, 0x16),
                log::Level::Warn => egui::Color32::from_rgb(0xFF, 0x7A, 0x0C),
                log::Level::Info => crate::theme::colors::TEXT_DEFAULT,
                log::Level::Debug => crate::theme::colors::TEXT_SUBDUED,
                log::Level::Trace => crate::theme::colors::gray::S500,
            };
            ui.colored_label(color, &entry.message);
        }
    });
}
