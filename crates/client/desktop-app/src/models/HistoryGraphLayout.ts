import { GraphNode } from "./GraphNode";

export interface LaneInfo {
  lane: number;
  color: string;
}

export interface EdgeSegment {
  fromLane: number;
  toLane: number;
  fromRow: number;
  toRow: number;
  color: string;
}

export interface RowMeta {
  node: GraphNode;
  lane: number;
  color: string;
}

export interface LayoutResult {
  rows: RowMeta[];
  totalLanes: number;
  segments: EdgeSegment[];
}
