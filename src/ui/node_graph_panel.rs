// Node graph editor panel — uses egui-snarl for visual node editing.

use crate::theme::colors::gray;
use egui_snarl::ui::{PinInfo, SnarlViewer, SnarlWidget};
use egui_snarl::{InPin, OutPin, Snarl};

// ─── Pin colors ─────────────────────────────────────────────────────

const IMAGE_PIN: egui::Color32 = egui::Color32::from_rgb(0x4B, 0x9C, 0xD3);
const PALETTE_PIN: egui::Color32 = egui::Color32::from_rgb(0xD3, 0x8B, 0x4B);
const MASK_PIN: egui::Color32 = egui::Color32::from_rgb(0x8B, 0xD3, 0x4B);
const SPRITE_PIN: egui::Color32 = egui::Color32::from_rgb(0xD3, 0x4B, 0xD3);
const COLOR_PIN: egui::Color32 = egui::Color32::from_rgb(0xD3, 0xD3, 0x4B);
const FLOAT_PIN: egui::Color32 = egui::Color32::from_rgb(0xB0, 0x60, 0x60);
const INT_PIN: egui::Color32 = egui::Color32::from_rgb(0x60, 0xB0, 0x60);
const BOOL_PIN: egui::Color32 = egui::Color32::from_rgb(0x60, 0x60, 0xB0);
const STRING_PIN: egui::Color32 = egui::Color32::from_rgb(0xB0, 0xB0, 0xB0);

// ─── Node definition ────────────────────────────────────────────────

/// Node types in the kitbash node graph.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum KitbashNode {
    ImageInput { path: String },
    SpriteOutput,
    ReplaceColor { from: [u8; 4], to: [u8; 4] },
    GridSlice { cols: u32, rows: u32 },
    ColorDetect { threshold: f32 },
}

impl KitbashNode {
    fn name(&self) -> &str {
        match self {
            Self::ImageInput { .. } => "Image Input",
            Self::SpriteOutput => "Sprite Output",
            Self::ReplaceColor { .. } => "Replace Color",
            Self::GridSlice { .. } => "Grid Slice",
            Self::ColorDetect { .. } => "Color Detect",
        }
    }

    fn inputs(&self) -> &[(&str, PinKind)] {
        match self {
            Self::ImageInput { .. } => &[],
            Self::SpriteOutput => &[("sprites", PinKind::Sprite)],
            Self::ReplaceColor { .. } => &[("image", PinKind::Image)],
            Self::GridSlice { .. } => &[("image", PinKind::Image)],
            Self::ColorDetect { .. } => &[("image", PinKind::Image)],
        }
    }

    fn outputs(&self) -> &[(&str, PinKind)] {
        match self {
            Self::ImageInput { .. } => &[("image", PinKind::Image)],
            Self::SpriteOutput => &[],
            Self::ReplaceColor { .. } => &[("image", PinKind::Image)],
            Self::GridSlice { .. } => &[("sprites", PinKind::Sprite)],
            Self::ColorDetect { .. } => &[("palette", PinKind::Palette)],
        }
    }

    fn header_color(&self) -> egui::Color32 {
        match self {
            Self::ImageInput { .. } => egui::Color32::from_rgb(0x2A, 0x4A, 0x6A),
            Self::SpriteOutput => egui::Color32::from_rgb(0x4A, 0x2A, 0x6A),
            Self::ReplaceColor { .. } => egui::Color32::from_rgb(0x6A, 0x3A, 0x2A),
            Self::GridSlice { .. } => egui::Color32::from_rgb(0x2A, 0x6A, 0x3A),
            Self::ColorDetect { .. } => egui::Color32::from_rgb(0x6A, 0x6A, 0x2A),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum PinKind {
    Image,
    Palette,
    #[allow(dead_code)]
    Mask,
    Sprite,
    #[allow(dead_code)]
    Color,
    #[allow(dead_code)]
    Float,
    #[allow(dead_code)]
    Int,
    #[allow(dead_code)]
    Bool,
    #[allow(dead_code)]
    String,
}

impl PinKind {
    fn color(self) -> egui::Color32 {
        match self {
            Self::Image => IMAGE_PIN,
            Self::Palette => PALETTE_PIN,
            Self::Mask => MASK_PIN,
            Self::Sprite => SPRITE_PIN,
            Self::Color => COLOR_PIN,
            Self::Float => FLOAT_PIN,
            Self::Int => INT_PIN,
            Self::Bool => BOOL_PIN,
            Self::String => STRING_PIN,
        }
    }
}

// ─── Viewer ─────────────────────────────────────────────────────────

struct KitbashViewer;

impl SnarlViewer<KitbashNode> for KitbashViewer {
    fn title(&mut self, node: &KitbashNode) -> String {
        node.name().to_owned()
    }

    fn inputs(&mut self, node: &KitbashNode) -> usize {
        node.inputs().len()
    }

    fn outputs(&mut self, node: &KitbashNode) -> usize {
        node.outputs().len()
    }

    #[allow(refining_impl_trait)]
    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<KitbashNode>,
    ) -> PinInfo {
        let node = &snarl[pin.id.node];
        let inputs = node.inputs();
        if let Some(&(name, kind)) = inputs.get(pin.id.input) {
            ui.label(name);
            show_input_inline(ui, &mut snarl[pin.id.node], pin.id.input);
            PinInfo::circle().with_fill(kind.color())
        } else {
            ui.label("?");
            PinInfo::circle().with_fill(gray::S500)
        }
    }

    #[allow(refining_impl_trait)]
    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<KitbashNode>,
    ) -> PinInfo {
        let node = &snarl[pin.id.node];
        let outputs = node.outputs();
        if let Some(&(name, kind)) = outputs.get(pin.id.output) {
            ui.label(name);
            PinInfo::circle().with_fill(kind.color())
        } else {
            ui.label("?");
            PinInfo::circle().with_fill(gray::S500)
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<KitbashNode>) {
        let from_kind = snarl[from.id.node]
            .outputs()
            .get(from.id.output)
            .map(|&(_, k)| std::mem::discriminant(&k));
        let to_kind = snarl[to.id.node]
            .inputs()
            .get(to.id.input)
            .map(|&(_, k)| std::mem::discriminant(&k));

        if from_kind == to_kind {
            // Disconnect existing wires to this input
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
        if ui.button("Image Input").clicked() {
            snarl.insert_node(
                pos,
                KitbashNode::ImageInput {
                    path: String::new(),
                },
            );
            ui.close();
        }
        if ui.button("Sprite Output").clicked() {
            snarl.insert_node(pos, KitbashNode::SpriteOutput);
            ui.close();
        }
        if ui.button("Replace Color").clicked() {
            snarl.insert_node(
                pos,
                KitbashNode::ReplaceColor {
                    from: [0, 0, 0, 255],
                    to: [255, 255, 255, 255],
                },
            );
            ui.close();
        }
        if ui.button("Grid Slice").clicked() {
            snarl.insert_node(pos, KitbashNode::GridSlice { cols: 4, rows: 4 });
            ui.close();
        }
        if ui.button("Color Detect").clicked() {
            snarl.insert_node(pos, KitbashNode::ColorDetect { threshold: 10.0 });
            ui.close();
        }
    }

    fn has_node_menu(&mut self, _node: &KitbashNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<KitbashNode>,
    ) {
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close();
        }
    }

    fn header_frame(
        &mut self,
        frame: egui::Frame,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        snarl: &Snarl<KitbashNode>,
    ) -> egui::Frame {
        frame.fill(snarl[node].header_color())
    }
}

/// Show inline parameter editors for node inputs.
fn show_input_inline(ui: &mut egui::Ui, node: &mut KitbashNode, _input_idx: usize) {
    match node {
        KitbashNode::ImageInput { ref mut path } => {
            egui::TextEdit::singleline(path)
                .desired_width(80.0)
                .hint_text("path")
                .show(ui);
        }
        KitbashNode::GridSlice {
            ref mut cols,
            ref mut rows,
        } => {
            ui.add(egui::DragValue::new(cols).range(1..=64).prefix("C:"));
            ui.add(egui::DragValue::new(rows).range(1..=64).prefix("R:"));
        }
        KitbashNode::ColorDetect {
            ref mut threshold, ..
        } => {
            ui.add(
                egui::DragValue::new(threshold)
                    .range(0.0..=255.0)
                    .prefix("T:"),
            );
        }
        _ => {}
    }
}

// ─── Panel state ────────────────────────────────────────────────────

/// State for the node graph panel.
pub struct NodeGraphPanel {
    pub snarl: Snarl<KitbashNode>,
    pub style: egui_snarl::ui::SnarlStyle,
}

impl Default for NodeGraphPanel {
    fn default() -> Self {
        let style = egui_snarl::ui::SnarlStyle::default();
        Self {
            snarl: Snarl::new(),
            style,
        }
    }
}

/// Draw the node graph panel.
pub fn draw_node_graph(ui: &mut egui::Ui, panel: &mut NodeGraphPanel) {
    SnarlWidget::new()
        .id(egui::Id::new("kitbash_snarl"))
        .style(panel.style)
        .show(&mut panel.snarl, &mut KitbashViewer, ui);
}
