use serde::Serialize;

use crate::models::{graph_edge::GraphEdge, graph_node::GraphNode};

#[derive(Serialize, Clone)]
pub struct CommitGraphInfo {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}
