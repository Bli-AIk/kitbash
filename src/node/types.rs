// Node graph data types used for inter-node communication.
//
// These types define the data that flows between nodes in the processing pipeline.

/// RGBA8 pixel image data — the core data type for the node graph.
#[derive(Clone, Debug)]
pub struct NodeImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl NodeImage {
    pub fn new(width: u32, height: u32) -> Self {
        let len = (width * height * 4) as usize;
        Self {
            width,
            height,
            pixels: vec![0; len],
        }
    }

    pub fn from_rgba(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        debug_assert_eq!(pixels.len(), (width * height * 4) as usize);
        Self {
            width,
            height,
            pixels,
        }
    }
}

/// A named RGBA color.
#[derive(Clone, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// A palette of named colors.
#[derive(Clone, Debug, Default)]
pub struct Palette {
    pub colors: Vec<Color>,
}

/// A binary mask (one byte per pixel, 0 or 255).
#[derive(Clone, Debug)]
pub struct Mask {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// A list of sprite sub-images produced by slicing operations.
#[derive(Clone, Debug, Default)]
pub struct SpriteList {
    pub sprites: Vec<NodeImage>,
}

/// A scattered-format export: named sprite images with their original positions.
///
/// Used by the Sprite Output node for preserving spatial layout in exports.
#[derive(Clone, Debug, Default)]
pub struct ScatteredPack {
    pub entries: Vec<ScatteredEntry>,
}

/// A single entry in a `ScatteredPack`.
#[derive(Clone, Debug)]
pub struct ScatteredEntry {
    pub name: String,
    pub image: NodeImage,
    pub x: i32,
    pub y: i32,
}

/// The value types that can flow between node ports.
#[derive(Clone, Debug)]
pub enum PortValue {
    Image(NodeImage),
    Palette(Palette),
    Mask(Mask),
    SpriteList(SpriteList),
    ScatteredPack(ScatteredPack),
    Color(Color),
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
}

/// Type tag for port compatibility checking (without carrying data).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PortType {
    Image,
    Palette,
    Mask,
    SpriteList,
    ScatteredPack,
    Color,
    Float,
    Int,
    Bool,
    String,
}

impl PortValue {
    pub fn port_type(&self) -> PortType {
        match self {
            Self::Image(_) => PortType::Image,
            Self::Palette(_) => PortType::Palette,
            Self::Mask(_) => PortType::Mask,
            Self::SpriteList(_) => PortType::SpriteList,
            Self::ScatteredPack(_) => PortType::ScatteredPack,
            Self::Color(_) => PortType::Color,
            Self::Float(_) => PortType::Float,
            Self::Int(_) => PortType::Int,
            Self::Bool(_) => PortType::Bool,
            Self::String(_) => PortType::String,
        }
    }
}

impl std::fmt::Display for PortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Image => write!(f, "Image"),
            Self::Palette => write!(f, "Palette"),
            Self::Mask => write!(f, "Mask"),
            Self::SpriteList => write!(f, "SpriteList"),
            Self::ScatteredPack => write!(f, "ScatteredPack"),
            Self::Color => write!(f, "Color"),
            Self::Float => write!(f, "Float"),
            Self::Int => write!(f, "Int"),
            Self::Bool => write!(f, "Bool"),
            Self::String => write!(f, "String"),
        }
    }
}
