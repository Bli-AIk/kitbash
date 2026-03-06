// WASM plugin ABI definitions.
//
// Defines the interface contract between the host (kitbash) and WASM plugin modules.

/// ABI function names that plugins must export.
pub const FN_NODE_INFO: &str = "node_info";
pub const FN_PROCESS: &str = "process";
pub const FN_ALLOC: &str = "alloc";
pub const FN_DEALLOC: &str = "dealloc";

/// The magic bytes identifying a kitbash plugin.
pub const PLUGIN_MAGIC: &[u8; 4] = b"KBSH";

/// Current ABI version. Plugins must match this to be loaded.
pub const ABI_VERSION: u32 = 1;

/// Header written at the start of serialized data passed to/from plugins.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DataHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub data_len: u32,
}

impl DataHeader {
    pub fn new(data_len: u32) -> Self {
        Self {
            magic: *PLUGIN_MAGIC,
            version: ABI_VERSION,
            data_len,
        }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(12);
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.data_len.to_le_bytes());
        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 12 {
            return None;
        }
        let magic: [u8; 4] = data[0..4].try_into().ok()?;
        if &magic != PLUGIN_MAGIC {
            return None;
        }
        let version = u32::from_le_bytes(data[4..8].try_into().ok()?);
        let data_len = u32::from_le_bytes(data[8..12].try_into().ok()?);
        Some(Self {
            magic,
            version,
            data_len,
        })
    }
}

/// Image data layout for transferring to/from WASM plugins.
///
/// Wire format: `[width: u32 LE][height: u32 LE][RGBA pixel data...]`
pub fn encode_image(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(8 + pixels.len());
    buf.extend_from_slice(&width.to_le_bytes());
    buf.extend_from_slice(&height.to_le_bytes());
    buf.extend_from_slice(pixels);
    buf
}

/// Decode image data from the wire format.
pub fn decode_image(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    if data.len() < 8 {
        return None;
    }
    let width = u32::from_le_bytes(data[0..4].try_into().ok()?);
    let height = u32::from_le_bytes(data[4..8].try_into().ok()?);
    let expected = (width as usize) * (height as usize) * 4;
    if data.len() < 8 + expected {
        return None;
    }
    Some((width, height, data[8..8 + expected].to_vec()))
}
