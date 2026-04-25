use crate::database::commit::Commit;
use crate::database::database::Database;
use crate::internals::refs::Refs;
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Serialize, Clone)]
pub struct GraphNode {
    pub id: String,
    pub short_id: String,
    pub message: String,
    pub author: String,
    pub branches: Vec<String>,
    pub parents: Vec<String>,
    pub is_merge: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Serialize)]
pub struct CommitGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub struct CommitWalker<'a> {
    db: &'a Database,
}

impl<'a> CommitWalker<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn walk(&self, refs: &Refs) -> anyhow::Result<CommitGraph> {
        let mut branch_index: HashMap<String, Vec<String>> = HashMap::new();
        for (branch, hash) in &refs.branches {
            if !hash.is_empty() {
                branch_index
                    .entry(hash.clone())
                    .or_default()
                    .push(branch.clone());
            }
        }

        let mut queue: VecDeque<String> = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();

        let mut branch_tips: Vec<String> = refs
            .branches
            .values()
            .filter(|h| !h.is_empty())
            .cloned()
            .collect();
        branch_tips.sort();
        branch_tips.dedup();

        for tip in branch_tips {
            if visited.insert(tip.clone()) {
                queue.push_back(tip);
            }
        }

        let mut nodes: Vec<GraphNode> = Vec::new();
        let mut edges: Vec<GraphEdge> = Vec::new();

        while let Some(hash) = queue.pop_front() {
            let obj = match self.db.read_object(&hash) {
                Ok(o) => o,
                Err(_) => continue,
            };

            let commit = match obj.as_any().downcast_ref::<Commit>() {
                Some(c) => c,
                None => continue,
            };

            let parents: Vec<String> = commit
                .parent_hash()
                .map(|p| vec![p.to_string()])
                .unwrap_or_default();

            for parent_hash in &parents {
                edges.push(GraphEdge {
                    id: format!(
                        "{}→{}",
                        &hash[..7.min(hash.len())],
                        &parent_hash[..7.min(parent_hash.len())]
                    ),
                    source: hash.clone(),
                    target: parent_hash.clone(),
                });
                if visited.insert(parent_hash.clone()) {
                    queue.push_back(parent_hash.clone());
                }
            }

            nodes.push(GraphNode {
                short_id: hash[..7.min(hash.len())].to_string(),
                branches: branch_index.remove(&hash).unwrap_or_default(),
                parents: parents.clone(),
                is_merge: parents.len() > 1,
                message: commit.message.clone(),
                author: commit.author.clone(),
                id: hash,
            });
        }

        Ok(CommitGraph { nodes, edges })
    }
}
