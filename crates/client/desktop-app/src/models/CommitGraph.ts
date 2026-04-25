import { GraphEdge, GraphNode } from "./Graph";

export interface CommitGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
}