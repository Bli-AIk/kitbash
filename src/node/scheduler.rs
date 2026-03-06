// Node graph execution scheduler.
//
// Performs topological sort and executes nodes in dependency order.

use std::collections::{HashMap, HashSet, VecDeque};

use super::graph::{NodeGraph, NodeId};
use super::types::PortValue;

/// Execute the entire node graph, returning outputs keyed by `(NodeId, port_index)`.
pub fn execute_graph(graph: &mut NodeGraph) -> HashMap<(NodeId, usize), PortValue> {
    let order = topological_sort(graph);
    let mut outputs: HashMap<(NodeId, usize), PortValue> = HashMap::new();

    for node_id in order {
        let Some(node) = graph.nodes.get(&node_id) else {
            continue;
        };
        let info = node.processor.info();

        let inputs: Vec<Option<PortValue>> = (0..info.inputs.len())
            .map(|port_idx| {
                graph
                    .connections
                    .iter()
                    .find(|c| c.to_node == node_id && c.to_port == port_idx)
                    .and_then(|c| outputs.get(&(c.from_node, c.from_port)).cloned())
            })
            .collect();

        let result = node.processor.process(&inputs, &node.params);

        for (port_idx, value) in result.into_iter().enumerate() {
            outputs.insert((node_id, port_idx), value);
        }
    }

    graph.cache = outputs.clone();
    outputs
}

/// Topological sort of nodes using Kahn's algorithm.
fn topological_sort(graph: &NodeGraph) -> Vec<NodeId> {
    let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
    let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    for &id in graph.nodes.keys() {
        in_degree.entry(id).or_insert(0);
        adjacency.entry(id).or_default();
    }

    for conn in &graph.connections {
        *in_degree.entry(conn.to_node).or_insert(0) += 1;
        adjacency
            .entry(conn.from_node)
            .or_default()
            .push(conn.to_node);
    }

    let mut queue: VecDeque<NodeId> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut result = Vec::new();
    let mut visited = HashSet::new();

    while let Some(id) = queue.pop_front() {
        if !visited.insert(id) {
            continue;
        }
        result.push(id);

        let Some(neighbors) = adjacency.get(&id) else {
            continue;
        };
        for &neighbor in neighbors {
            let Some(deg) = in_degree.get_mut(&neighbor) else {
                continue;
            };
            *deg = deg.saturating_sub(1);
            if *deg == 0 {
                queue.push_back(neighbor);
            }
        }
    }

    result
}
