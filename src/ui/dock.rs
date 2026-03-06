// egui_tiles-based dock layout system.
//
// Provides a tiling panel system matching the bevy_workbench layout style.
// Panel rendering is dispatched by string ID — the `DockBehavior` carries
// a mutable reference to `KitbashApp` so every panel has full state access.

use std::collections::HashMap;

use crate::app::KitbashApp;

/// Identifies a panel in the tile tree.
pub type PanelId = usize;

/// A pane entry stored in the egui_tiles tree.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaneEntry {
    pub panel_id: PanelId,
}

/// Metadata for a registered panel.
struct PanelMeta {
    str_id: String,
    title: String,
}

/// Where a panel should be placed in the default layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelSlot {
    RightTop,
    RightBottom,
    Bottom,
    Center,
}

/// State for the tile layout and registered panels.
pub struct TileLayoutState {
    pub tree: Option<egui_tiles::Tree<PaneEntry>>,
    panels: HashMap<PanelId, PanelMeta>,
    panel_id_map: HashMap<String, PanelId>,
    panel_tile_map: HashMap<PanelId, egui_tiles::TileId>,
}

impl Default for TileLayoutState {
    fn default() -> Self {
        let mut state = Self {
            tree: None,
            panels: HashMap::new(),
            panel_id_map: HashMap::new(),
            panel_tile_map: HashMap::new(),
        };
        state.build_default_tree();
        state
    }
}

/// All panels in the editor, with their string IDs, display titles, and slots.
const PANEL_DEFS: &[(&str, &str, PanelSlot)] = &[
    ("node_graph", "Node Graph", PanelSlot::Center),
    ("preview", "Preview", PanelSlot::RightTop),
    ("settings", "Settings", PanelSlot::RightTop),
    ("inspector", "Inspector", PanelSlot::RightBottom),
    ("console", "Console", PanelSlot::Bottom),
];

/// Build the right column: top (Preview+Settings tabs) and bottom (Inspector), vertically split.
fn build_right_column(
    tiles: &mut egui_tiles::Tiles<PaneEntry>,
    top_tiles: Vec<egui_tiles::TileId>,
    bottom_tiles: Vec<egui_tiles::TileId>,
) -> Option<egui_tiles::TileId> {
    let top = match top_tiles.len() {
        0 => None,
        1 => Some(top_tiles[0]),
        _ => Some(tiles.insert_tab_tile(top_tiles)),
    };
    let bottom = match bottom_tiles.len() {
        0 => None,
        1 => Some(bottom_tiles[0]),
        _ => Some(tiles.insert_tab_tile(bottom_tiles)),
    };
    match (top, bottom) {
        (Some(t), Some(b)) => {
            let v = tiles.insert_vertical_tile(vec![t, b]);
            if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Linear(linear))) =
                tiles.get_mut(v)
            {
                linear.shares.set_share(t, 0.6);
                linear.shares.set_share(b, 0.4);
            }
            Some(v)
        }
        (Some(t), None) => Some(t),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

impl TileLayoutState {
    fn build_default_tree(&mut self) {
        self.panels.clear();
        self.panel_id_map.clear();
        self.panel_tile_map.clear();

        let mut tiles = egui_tiles::Tiles::default();
        let mut center_tiles = Vec::new();
        let mut right_top_tiles = Vec::new();
        let mut right_bottom_tiles = Vec::new();
        let mut bottom_tiles = Vec::new();

        for (idx, &(str_id, title, slot)) in PANEL_DEFS.iter().enumerate() {
            let panel_id = idx;
            self.panels.insert(
                panel_id,
                PanelMeta {
                    str_id: str_id.to_string(),
                    title: title.to_string(),
                },
            );
            self.panel_id_map.insert(str_id.to_string(), panel_id);
            let tile_id = tiles.insert_pane(PaneEntry { panel_id });
            self.panel_tile_map.insert(panel_id, tile_id);

            match slot {
                PanelSlot::Center => center_tiles.push(tile_id),
                PanelSlot::RightTop => right_top_tiles.push(tile_id),
                PanelSlot::RightBottom => right_bottom_tiles.push(tile_id),
                PanelSlot::Bottom => bottom_tiles.push(tile_id),
            }
        }

        let center = if center_tiles.len() > 1 {
            tiles.insert_tab_tile(center_tiles)
        } else if let Some(t) = center_tiles.into_iter().next() {
            t
        } else {
            tiles.insert_pane(PaneEntry { panel_id: 9999 })
        };

        // Build right column: top tabs + bottom tabs, vertically split
        let right = build_right_column(&mut tiles, right_top_tiles, right_bottom_tiles);

        let main_area = if let Some(right_col) = right {
            let h = tiles.insert_horizontal_tile(vec![center, right_col]);
            if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Linear(linear))) =
                tiles.get_mut(h)
            {
                linear.shares.set_share(center, 0.7);
                linear.shares.set_share(right_col, 0.3);
            }
            h
        } else {
            center
        };

        let bottom = if bottom_tiles.len() > 1 {
            Some(tiles.insert_tab_tile(bottom_tiles))
        } else {
            bottom_tiles.into_iter().next()
        };

        let root = if let Some(bottom_tab) = bottom {
            let v = tiles.insert_vertical_tile(vec![main_area, bottom_tab]);
            if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Linear(linear))) =
                tiles.get_mut(v)
            {
                linear.shares.set_share(main_area, 0.75);
                linear.shares.set_share(bottom_tab, 0.25);
            }
            v
        } else {
            main_area
        };

        self.tree = Some(egui_tiles::Tree::new("kitbash_dock", root, tiles));
    }

    /// Get a list of all panel names and their visibility.
    pub fn panel_visibility(&self) -> Vec<(String, String, bool)> {
        let tree = self.tree.as_ref();
        self.panels
            .iter()
            .map(|(&id, meta)| {
                let visible = self
                    .panel_tile_map
                    .get(&id)
                    .and_then(|tile_id| tree.map(|t| t.tiles.is_visible(*tile_id)))
                    .unwrap_or(true);
                (meta.str_id.clone(), meta.title.clone(), visible)
            })
            .collect()
    }

    /// Toggle visibility of a panel by its string ID.
    pub fn toggle_panel(&mut self, panel_str_id: &str) {
        let Some(&panel_id) = self.panel_id_map.get(panel_str_id) else {
            return;
        };
        let Some(&tile_id) = self.panel_tile_map.get(&panel_id) else {
            return;
        };
        if let Some(tree) = &mut self.tree {
            let currently_visible = tree.tiles.is_visible(tile_id);
            tree.tiles.set_visible(tile_id, !currently_visible);
        }
    }

    /// Look up the string ID for a panel by its numeric ID.
    fn str_id_for(&self, panel_id: PanelId) -> &str {
        self.panels
            .get(&panel_id)
            .map(|m| m.str_id.as_str())
            .unwrap_or("???")
    }
}

/// The egui_tiles behavior — dispatches to per-panel draw functions.
struct DockBehavior<'a> {
    app: &'a mut KitbashApp,
}

impl<'a> egui_tiles::Behavior<PaneEntry> for DockBehavior<'a> {
    fn tab_title_for_pane(&mut self, pane: &PaneEntry) -> egui::WidgetText {
        self.app
            .tile_state
            .panels
            .get(&pane.panel_id)
            .map(|m| m.title.as_str())
            .unwrap_or("???")
            .into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut PaneEntry,
    ) -> egui_tiles::UiResponse {
        let panel_str_id = self.app.tile_state.str_id_for(pane.panel_id).to_string();
        dispatch_panel(ui, self.app, &panel_str_id);
        egui_tiles::UiResponse::None
    }

    fn is_tab_closable(
        &self,
        _tiles: &egui_tiles::Tiles<PaneEntry>,
        _tile_id: egui_tiles::TileId,
    ) -> bool {
        false
    }

    fn tab_bar_color(&self, _visuals: &egui::Visuals) -> egui::Color32 {
        crate::theme::colors::gray::S150
    }

    fn dragged_overlay_color(&self, _visuals: &egui::Visuals) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(0x00, 0x4b, 0xc2, 100)
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }
}

/// Route a panel string ID to the correct draw function.
fn dispatch_panel(ui: &mut egui::Ui, app: &mut KitbashApp, panel_id: &str) {
    match panel_id {
        "node_graph" => super::node_graph_panel::draw_node_graph(ui, &mut app.node_graph_panel),
        "preview" => super::canvas_panel::draw_preview(ui, app),
        "inspector" => super::inspector_panel::draw_inspector(ui, app),
        "settings" => super::settings_panel::draw_settings(ui, &mut app.settings_panel),
        "console" => super::console_panel::draw_console(ui, &mut app.console),
        _ => {
            ui.label(format!("Unknown panel: {panel_id}"));
        }
    }
}

/// Show the dock layout — temporarily takes the tree out to satisfy the borrow checker.
pub fn show_dock(app: &mut KitbashApp, ctx: &egui::Context) {
    let mut tree = app.tile_state.tree.take();

    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(tree) = &mut tree {
            let mut behavior = DockBehavior { app };
            tree.ui(&mut behavior, ui);
        }
    });

    app.tile_state.tree = tree;
}
