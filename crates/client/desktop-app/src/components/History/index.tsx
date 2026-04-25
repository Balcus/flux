// History.tsx
import { useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import ReactFlow, {
  Background,
  BackgroundVariant,
  Controls,
  useNodesState,
  useEdgesState,
  Handle,
  Position,
  NodeProps,
} from "reactflow";
import dagre from "@dagrejs/dagre";
import { CommitIcon } from "../../assets/icons";
import { CommitGraph } from "../../models/CommitGraph";
import { GraphNode } from "../../models/GraphNode";

import "reactflow/dist/style.css";
import "./History.css";

const NODE_W = 240;
const NODE_H = 64;

function layout(graph: CommitGraph) {
  const g = new dagre.graphlib.Graph();
  g.setGraph({ rankdir: "BT", ranksep: 50, nodesep: 24 });
  g.setDefaultEdgeLabel(() => ({}));
  graph.nodes.forEach((n) =>
    g.setNode(n.id, { width: NODE_W, height: NODE_H }),
  );
  graph.edges.forEach((e) => g.setEdge(e.source, e.target));
  dagre.layout(g);

  const nodes = graph.nodes.map((n) => {
    const pos = g.node(n.id);
    return {
      id: n.id,
      type: "commitNode",
      position: { x: pos.x - NODE_W / 2, y: pos.y - NODE_H / 2 },
      sourcePosition: Position.Top,
      targetPosition: Position.Bottom,
      data: n,
    };
  });

  const edges = graph.edges.map((e) => ({
    id: e.id,
    source: e.source,
    target: e.target,
    style: { stroke: "var(--color-text-main)", strokeWidth: 1.5, opacity: 0.3 },
  }));

  return { nodes, edges };
}

function CommitNode({ data }: NodeProps<GraphNode>) {
  return (
    <div className="commit-node">
      <Handle type="source" position={Position.Top} style={{ opacity: 0 }} />
      <img className="commit-icon" src={CommitIcon} alt="" />
      <div className="commit-content">
        {data.branches.length > 0 && (
          <div className="commit-heads">
            {data.branches.map((b) => (
              <span key={b} className="commit-head">
                {b}
              </span>
            ))}
          </div>
        )}
        <p className="commit-message">{data.message}</p>
        <p className="commit-author">{data.author}</p>
        <code className="commit-hash">{data.short_id}</code>
      </div>
      <Handle type="target" position={Position.Bottom} style={{ opacity: 0 }} />
    </div>
  );
}

const nodeTypes = { commitNode: CommitNode };

export default function History() {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);

  const fetchGraph = useCallback(() => {
    invoke<CommitGraph>("get_graph")
      .then((graph) => {
        const { nodes: laid, edges: laidEdges } = layout(graph);
        setNodes(laid);
        setEdges(laidEdges);
      })
      .catch(console.error);
  }, []);

  useEffect(() => {
    fetchGraph();
  }, [fetchGraph]);

  return (
    <div className="history-container">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        nodeTypes={nodeTypes}
        fitView
        fitViewOptions={{ padding: 0.3 }}
        proOptions={{ hideAttribution: true }}
      >
        <Background
          variant={BackgroundVariant.Dots}
          color="var(--color-border)"
          gap={15}
          size={2}
        />
        <Controls />
      </ReactFlow>
    </div>
  );
}
