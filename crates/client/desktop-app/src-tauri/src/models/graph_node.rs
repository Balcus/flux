use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct GraphNode {
    pub id: String,
    pub short_id: String,
    pub message: String,
    pub author: String,
    pub branches: Vec<String>,
    pub parents: Vec<String>,
    pub is_merge: bool,
}
