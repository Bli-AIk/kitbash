// DAG-based node graph for the processing pipeline.

use std::collections::HashMap;

use super::types::{PortType, PortValue};

/// Unique identifier for a node instance in the graph.
pub type NodeId = u64;

/// Unique identifier for a connection between nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub u64);

/// Describes an input or output port on a node.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PortInfo {
    pub name: String,
    pub port_type: PortType,
}

/// Describes a parameter that the user can configure on a node.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParamInfo {
    pub name: String,
    pub port_type: PortType,
    pub default_json: Option<String>,
}

/// Metadata describing a node type (provided by each node implementation).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub category: String,
    pub inputs: Vec<PortInfo>,
    pub outputs: Vec<PortInfo>,
    pub params: Vec<ParamInfo>,
}

/// Trait that all node implementations must satisfy.
pub trait NodeProcessor: Send + Sync {
    fn info(&self) -> NodeInfo;
    fn process(&self, inputs: &[Option<PortValue>], params: &[PortValue]) -> Vec<PortValue>;
}

/// A connection between two node ports.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Connection {
    pub from_node: NodeId,
    pub from_port: usize,
    pub to_node: NodeId,
    pub to_port: usize,
}

/// A node instance in the graph.
pub struct NodeInstance {
    pub id: NodeId,
    pub processor: Box<dyn NodeProcessor>,
    pub params: Vec<PortValue>,
    /// Screen position for the node graph UI.
    pub position: egui::Vec2,
}

/// The node graph, owning all nodes and connections.
pub struct NodeGraph {
    pub nodes: HashMap<NodeId, NodeInstance>,
    pub connections: Vec<Connection>,
    next_id: NodeId,
    next_conn_id: u64,
    /// Cached output values from the last execution.
    pub cache: HashMap<(NodeId, usize), PortValue>,
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            next_id: 1,
            next_conn_id: 1,
            cache: HashMap::new(),
        }
    }
}

impl NodeGraph {
    pub fn add_node(&mut self, processor: Box<dyn NodeProcessor>, position: egui::Vec2) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        let info = processor.info();
        let params = info
            .params
            .iter()
            .map(|p| default_value_for_type(p.port_type))
            .collect();
        let instance = NodeInstance {
            id,
            processor,
            params,
            position,
        };
        self.nodes.insert(id, instance);
        id
    }

    pub fn remove_node(&mut self, id: NodeId) {
        self.nodes.remove(&id);
        self.connections
            .retain(|c| c.from_node != id && c.to_node != id);
        self.cache.retain(|&(nid, _), _| nid != id);
    }

    pub fn add_connection(&mut self, conn: Connection) -> Option<ConnectionId> {
        // Validate port types match
        let from_info = self.nodes.get(&conn.from_node)?.processor.info();
        let to_info = self.nodes.get(&conn.to_node)?.processor.info();
        let from_type = from_info.outputs.get(conn.from_port)?.port_type;
        let to_type = to_info.inputs.get(conn.to_port)?.port_type;
        if from_type != to_type {
            return None;
        }
        // Remove existing connection to same input
        self.connections
            .retain(|c| !(c.to_node == conn.to_node && c.to_port == conn.to_port));
        let id = ConnectionId(self.next_conn_id);
        self.next_conn_id += 1;
        self.connections.push(conn);
        Some(id)
    }

    pub fn remove_connection(&mut self, from_node: NodeId, from_port: usize) {
        self.connections
            .retain(|c| !(c.from_node == from_node && c.from_port == from_port));
    }
}

fn default_value_for_type(port_type: PortType) -> PortValue {
    match port_type {
        PortType::Image => PortValue::Image(super::types::NodeImage::new(1, 1)),
        PortType::Palette => PortValue::Palette(super::types::Palette::default()),
        PortType::Mask => PortValue::Mask(super::types::Mask {
            width: 1,
            height: 1,
            data: vec![0],
        }),
        PortType::SpriteList => PortValue::SpriteList(super::types::SpriteList::default()),
        PortType::Color => PortValue::Color(super::types::Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }),
        PortType::Float => PortValue::Float(0.0),
        PortType::Int => PortValue::Int(0),
        PortType::Bool => PortValue::Bool(false),
        PortType::String => PortValue::String(String::new()),
        PortType::ScatteredPack => PortValue::ScatteredPack(super::types::ScatteredPack::default()),
    }
}
