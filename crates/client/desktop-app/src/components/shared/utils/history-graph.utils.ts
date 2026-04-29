import { LANE_CONFIG } from "../../../constants";
import { GraphEdge } from "../../../models/GraphEdge";
import { GraphNode } from "../../../models/GraphNode";
import {
  EdgeSegment,
  LaneInfo,
  LayoutResult,
  RowMeta,
} from "../../../models/HistoryGraphLayout";

export function buildLayout(
  nodes: GraphNode[],
  edges: GraphEdge[],
): LayoutResult {
  const { LANE_COLORS } = LANE_CONFIG;
  const parentsOf: Record<string, string[]> = {};
  for (const e of edges) {
    if (!parentsOf[e.source]) parentsOf[e.source] = [];
    parentsOf[e.source].push(e.target);
  }

  const nodeIndex: Record<string, number> = {};
  nodes.forEach((n, i) => {
    nodeIndex[n.id] = i;
  });

  const laneOf: Record<string, LaneInfo> = {};
  const freeLanes: number[] = [];
  let nextLane = 0;
  let maxLane = 0;

  const rows: RowMeta[] = [];
  const segments: EdgeSegment[] = [];

  for (let i = 0; i < nodes.length; i++) {
    const node = nodes[i];
    const parents = parentsOf[node.id] ?? [];

    let myLane: number, myColor: string;
    if (laneOf[node.id]) {
      myLane = laneOf[node.id].lane;
      myColor = laneOf[node.id].color;
    } else {
      myLane = freeLanes.length > 0 ? freeLanes.shift()! : nextLane++;
      myColor = LANE_COLORS[myLane % LANE_COLORS.length];
      laneOf[node.id] = { lane: myLane, color: myColor };
    }
    maxLane = Math.max(maxLane, myLane);
    rows.push({ node, lane: myLane, color: myColor });

    let myLaneContinued = false;

    for (let pi = 0; pi < parents.length; pi++) {
      const parentId = parents[pi];
      const parentRow = nodeIndex[parentId];
      if (parentRow === undefined) continue;

      let pLane: number, pColor: string;
      if (laneOf[parentId]) {
        pLane = laneOf[parentId].lane;
        pColor = laneOf[parentId].color;
      } else if (pi === 0) {
        pLane = myLane;
        pColor = myColor;
        laneOf[parentId] = { lane: pLane, color: pColor };
        myLaneContinued = true;
      } else {
        pLane = freeLanes.length > 0 ? freeLanes.shift()! : nextLane++;
        pColor = LANE_COLORS[pLane % LANE_COLORS.length];
        laneOf[parentId] = { lane: pLane, color: pColor };
        maxLane = Math.max(maxLane, pLane);
      }

      segments.push({
        fromLane: myLane,
        toLane: pLane,
        fromRow: i,
        toRow: parentRow,
        color: pi === 0 ? myColor : pColor,
      });
    }

    if (!myLaneContinued) {
      freeLanes.push(myLane);
      freeLanes.sort((a, b) => a - b);
    }
  }

  return { rows, totalLanes: maxLane + 1, segments };
}
