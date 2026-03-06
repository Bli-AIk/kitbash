// Node graph editor panel — uses egui-snarl for visual node editing.
//
// All processing nodes are designed to map 1:1 to WASM plugins.
// ImageInput and SpriteOutput are fixed (host-side I/O); all others
// run through the same plugin ABI.

use crate::theme::colors::gray;
use egui_snarl::ui::{PinInfo, SnarlViewer, SnarlWidget};
use egui_snarl::{InPin, NodeId, OutPin, Snarl};

// ─── Pin colors ─────────────────────────────────────────────────────

const IMAGE_PIN: egui::Color32 = egui::Color32::from_rgb(0x4B, 0x9C, 0xD3);
const PALETTE_PIN: egui::Color32 = egui::Color32::from_rgb(0xD3, 0x8B, 0x4B);
const SCATTERED_PIN: egui::Color32 = egui::Color32::from_rgb(0xA3, 0x5B, 0xC3);
const SPRITE_PIN: egui::Color32 = egui::Color32::from_rgb(0xD3, 0x4B, 0xD3);

// ─── Node definition ────────────────────────────────────────────────

/// Node types in the kitbash node graph.
/// Fixed nodes (ImageInput, SpriteOutput) are created at startup and cannot be deleted.
/// All processing nodes follow the WASM plugin ABI.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum KitbashNode {
    // Fixed host-side I/O nodes
    ImageInput {
        image_ids: Vec<u64>,
    },
    SpriteOutput {
        canvas_size: Option<[u32; 2]>,
        export_scale: u32,
    },
    // Processing nodes (WASM plugin ABI)
    CanvasLayout {
        canvas_size: [u32; 2],
        bg_color: [u8; 4],
        num_inputs: usize,
        layer_offsets: Vec<[f32; 2]>,
        layer_scales: Vec<f32>,
    },
    ReplaceColor {
        from: [u8; 4],
        to: [u8; 4],
    },
    GridSlice {
        cols: u32,
        rows: u32,
    },
    ColorDetect {
        threshold: f32,
    },
    Composite {
        num_inputs: usize,
    },
    Transform {
        offset: [f32; 2],
        scale: f32,
    },
}

impl KitbashNode {
    pub fn is_fixed(&self) -> bool {
        matches!(self, Self::ImageInput { .. } | Self::SpriteOutput { .. })
    }

    pub fn name(&self) -> &str {
        match self {
            Self::ImageInput { .. } => "Image Input",
            Self::SpriteOutput { .. } => "Sprite Output",
            Self::CanvasLayout { .. } => "Canvas Layout",
            Self::ReplaceColor { .. } => "Replace Color",
            Self::GridSlice { .. } => "Grid Slice",
            Self::ColorDetect { .. } => "Color Detect",
            Self::Composite { .. } => "Composite",
            Self::Transform { .. } => "Transform",
        }
    }

    pub fn input_count(&self) -> usize {
        self.input_pins().len()
    }

    fn input_pins(&self) -> Vec<(&str, PinKind)> {
        match self {
            Self::ImageInput { .. } => vec![],
            Self::SpriteOutput { .. } => vec![("result", PinKind::Image)],
            Self::CanvasLayout { num_inputs, .. } => (0..*num_inputs)
                .map(|_| ("image", PinKind::Image))
                .collect(),
            Self::ReplaceColor { .. } => {
                vec![("image", PinKind::Image), ("palette", PinKind::Palette)]
            }
            Self::GridSlice { .. } => vec![("image", PinKind::Image)],
            Self::ColorDetect { .. } => vec![("image", PinKind::Image)],
            Self::Composite { num_inputs, .. } => (0..*num_inputs)
                .map(|_| ("image", PinKind::Image))
                .collect(),
            Self::Transform { .. } => vec![("image", PinKind::Image)],
        }
    }

    fn output_pins(&self) -> Vec<(&str, PinKind)> {
        match self {
            Self::ImageInput { image_ids, .. } => {
                if image_ids.is_empty() {
                    vec![("image", PinKind::Image)]
                } else {
                    image_ids
                        .iter()
                        .map(|_| ("image", PinKind::Image))
                        .collect()
                }
            }
            Self::SpriteOutput { .. } => vec![],
            Self::CanvasLayout { .. } => {
                vec![
                    ("composed", PinKind::Image),
                    ("scattered", PinKind::Scattered),
                ]
            }
            Self::ReplaceColor { .. } => vec![("image", PinKind::Image)],
            Self::GridSlice { .. } => vec![("sprites", PinKind::Sprite)],
            Self::ColorDetect { .. } => vec![("palette", PinKind::Palette)],
            Self::Composite { .. } => vec![("image", PinKind::Image)],
            Self::Transform { .. } => vec![("image", PinKind::Image)],
        }
    }

    fn header_color(&self) -> egui::Color32 {
        match self {
            Self::ImageInput { .. } => egui::Color32::from_rgb(0x2A, 0x4A, 0x6A),
            Self::SpriteOutput { .. } => egui::Color32::from_rgb(0x4A, 0x2A, 0x6A),
            Self::CanvasLayout { .. } => egui::Color32::from_rgb(0x3A, 0x5A, 0x3A),
            Self::ReplaceColor { .. } => egui::Color32::from_rgb(0x6A, 0x3A, 0x2A),
            Self::GridSlice { .. } => egui::Color32::from_rgb(0x2A, 0x6A, 0x3A),
            Self::ColorDetect { .. } => egui::Color32::from_rgb(0x6A, 0x6A, 0x2A),
            Self::Composite { .. } => egui::Color32::from_rgb(0x3A, 0x3A, 0x6A),
            Self::Transform { .. } => egui::Color32::from_rgb(0x5A, 0x3A, 0x5A),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PinKind {
    Image,
    Palette,
    Sprite,
    Scattered,
}

impl PinKind {
    fn color(self) -> egui::Color32 {
        match self {
            Self::Image => IMAGE_PIN,
            Self::Palette => PALETTE_PIN,
            Self::Sprite => SPRITE_PIN,
            Self::Scattered => SCATTERED_PIN,
        }
    }
}

// ─── Viewer ─────────────────────────────────────────────────────────

struct KitbashViewer<'a> {
    selected_node: &'a mut Option<NodeId>,
}

impl SnarlViewer<KitbashNode> for KitbashViewer<'_> {
    fn title(&mut self, node: &KitbashNode) -> String {
        node.name().to_owned()
    }

    fn inputs(&mut self, node: &KitbashNode) -> usize {
        node.input_pins().len()
    }

    fn outputs(&mut self, node: &KitbashNode) -> usize {
        node.output_pins().len()
    }

    fn show_header(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<KitbashNode>,
    ) {
        let title = snarl[node].name().to_owned();
        let is_selected = *self.selected_node == Some(node);
        let label = if is_selected {
            egui::RichText::new(title).strong()
        } else {
            egui::RichText::new(title)
        };
        if ui
            .add(egui::Label::new(label).sense(egui::Sense::click()))
            .clicked()
        {
            *self.selected_node = Some(node);
        }
    }

    #[expect(
        refining_impl_trait,
        reason = "snarl viewer pattern requires concrete PinInfo"
    )]
    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<KitbashNode>,
    ) -> PinInfo {
        let pins = snarl[pin.id.node].input_pins();
        if let Some(&(name, kind)) = pins.get(pin.id.input) {
            ui.label(name);
            PinInfo::circle().with_fill(kind.color())
        } else {
            ui.label("?");
            PinInfo::circle().with_fill(gray::S500)
        }
    }

    #[expect(
        refining_impl_trait,
        reason = "snarl viewer pattern requires concrete PinInfo"
    )]
    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<KitbashNode>,
    ) -> PinInfo {
        let pins = snarl[pin.id.node].output_pins();
        if let Some(&(name, kind)) = pins.get(pin.id.output) {
            ui.label(name);
            PinInfo::circle().with_fill(kind.color())
        } else {
            ui.label("?");
            PinInfo::circle().with_fill(gray::S500)
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<KitbashNode>) {
        let from_kind = snarl[from.id.node]
            .output_pins()
            .into_iter()
            .nth(from.id.output)
            .map(|(_, k)| k);
        let to_kind = snarl[to.id.node]
            .input_pins()
            .into_iter()
            .nth(to.id.input)
            .map(|(_, k)| k);

        if from_kind == to_kind {
            for &remote in &to.remotes {
                snarl.disconnect(remote, to.id);
            }
            snarl.connect(from.id, to.id);
        }
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<KitbashNode>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<KitbashNode>,
    ) {
        ui.label("Add Node");
        ui.separator();
        show_add_node_menu(pos, ui, snarl);
    }

    fn has_node_menu(&mut self, _node: &KitbashNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<KitbashNode>,
    ) {
        *self.selected_node = Some(node);

        if snarl[node].is_fixed() {
            ui.label("(Fixed node)");
        } else if ui.button("🗑 Remove").clicked() {
            snarl.remove_node(node);
            *self.selected_node = None;
            ui.close();
        }
    }

    fn header_frame(
        &mut self,
        frame: egui::Frame,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        snarl: &Snarl<KitbashNode>,
    ) -> egui::Frame {
        frame.fill(snarl[node].header_color())
    }
}

fn show_add_node_menu(pos: egui::Pos2, ui: &mut egui::Ui, snarl: &mut Snarl<KitbashNode>) {
    let items: &[(&str, KitbashNode)] = &[
        (
            "Canvas Layout",
            KitbashNode::CanvasLayout {
                canvas_size: [64, 64],
                bg_color: [0, 0, 0, 0],
                num_inputs: 2,
                layer_offsets: vec![[0.0; 2]; 2],
                layer_scales: vec![1.0; 2],
            },
        ),
        (
            "Replace Color",
            KitbashNode::ReplaceColor {
                from: [0, 0, 0, 255],
                to: [255, 255, 255, 255],
            },
        ),
        ("Grid Slice", KitbashNode::GridSlice { cols: 4, rows: 4 }),
        ("Color Detect", KitbashNode::ColorDetect { threshold: 10.0 }),
        ("Composite", KitbashNode::Composite { num_inputs: 2 }),
        (
            "Transform",
            KitbashNode::Transform {
                offset: [0.0, 0.0],
                scale: 1.0,
            },
        ),
    ];

    for (label, template) in items {
        if ui.button(*label).clicked() {
            snarl.insert_node(pos, template.clone());
            ui.close();
        }
    }
}

// ─── Panel state ────────────────────────────────────────────────────

/// State for the node graph panel.
pub struct NodeGraphPanel {
    pub snarl: Snarl<KitbashNode>,
    pub style: egui_snarl::ui::SnarlStyle,
    pub selected_node: Option<NodeId>,
    pub input_node: NodeId,
    pub output_node: NodeId,
}

impl Default for NodeGraphPanel {
    fn default() -> Self {
        let mut snarl = Snarl::new();
        let input_node = snarl.insert_node(
            egui::pos2(-200.0, 0.0),
            KitbashNode::ImageInput { image_ids: vec![] },
        );
        let output_node = snarl.insert_node(
            egui::pos2(200.0, 0.0),
            KitbashNode::SpriteOutput {
                canvas_size: None,
                export_scale: 1,
            },
        );
        Self {
            snarl,
            style: egui_snarl::ui::SnarlStyle::default(),
            selected_node: None,
            input_node,
            output_node,
        }
    }
}

/// Draw the node graph panel.
pub fn draw_node_graph(ui: &mut egui::Ui, panel: &mut NodeGraphPanel) {
    let mut viewer = KitbashViewer {
        selected_node: &mut panel.selected_node,
    };
    SnarlWidget::new()
        .id(egui::Id::new("kitbash_snarl"))
        .style(panel.style)
        .show(&mut panel.snarl, &mut viewer, ui);
}
