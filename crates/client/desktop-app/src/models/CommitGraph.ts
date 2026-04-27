import { GraphEdge } from "./GraphEdge";
import { GraphNode } from "./GraphNode";

export interface CommitGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
}