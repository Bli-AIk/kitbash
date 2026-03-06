// WASM runtime abstraction.
//
// Provides a trait-based interface for executing WASM plugins, with the actual
// runtime implementation deferred until wasmtime integration is ready.

use super::abi;
use crate::node::graph::{NodeInfo, NodeProcessor};
use crate::node::types::PortValue;

/// Errors from the WASM plugin runtime.
#[derive(Debug)]
pub enum PluginError {
    Load(String),
    Abi(String),
    Execution(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(msg) => write!(f, "Plugin load failed: {msg}"),
            Self::Abi(msg) => write!(f, "ABI error: {msg}"),
            Self::Execution(msg) => write!(f, "Execution failed: {msg}"),
        }
    }
}

/// A loaded WASM plugin instance.
///
/// Currently a stub. Will be backed by wasmtime on native and browser WASM API on web.
pub struct WasmPlugin {
    pub name: String,
    pub node_info: NodeInfo,
    _wasm_bytes: Vec<u8>,
}

impl WasmPlugin {
    /// Attempt to load a plugin from raw WASM bytes.
    ///
    /// Currently validates the ABI header only (runtime execution is stubbed).
    pub fn load(name: String, wasm_bytes: Vec<u8>) -> Result<Self, PluginError> {
        // In the future, this will instantiate a wasmtime module and call node_info()
        let _ = abi::ABI_VERSION;
        log::info!("Plugin '{name}' loaded (stub runtime)");

        let node_info = NodeInfo {
            name: name.clone(),
            category: "Plugin".into(),
            inputs: vec![],
            outputs: vec![],
            params: vec![],
        };

        Ok(Self {
            name,
            node_info,
            _wasm_bytes: wasm_bytes,
        })
    }
}

/// Adapter that wraps a `WasmPlugin` as a `NodeProcessor`.
pub struct WasmNodeProcessor {
    pub plugin: WasmPlugin,
}

impl NodeProcessor for WasmNodeProcessor {
    fn info(&self) -> NodeInfo {
        self.plugin.node_info.clone()
    }

    fn process(&self, _inputs: &[Option<PortValue>], _params: &[PortValue]) -> Vec<PortValue> {
        // Stub: real implementation will call into the WASM module
        log::warn!("Plugin '{}' process() is a stub", self.plugin.name);
        vec![]
    }
}
