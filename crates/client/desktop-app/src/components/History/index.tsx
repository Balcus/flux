import { useEffect, useCallback, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useVirtualizer } from "@tanstack/react-virtual";
import { CommitGraph } from "../../models/CommitGraph";
import { LayoutResult } from "../../models/HistoryGraphLayout";
import { buildLayout } from "../shared/utils/history-graph.utils";
import { GraphColumn } from "./GraphColumn";
import { CommitRow } from "./CommitRow";
import { LANE_CONFIG } from "../../constants";
import { useRepository } from "../../context/RepositoryContext";

import "./History.css";

export default function History() {
  const { LANE_W, ROW_H } = LANE_CONFIG;
  const scrollRef = useRef<HTMLDivElement>(null);
  const [layout, setLayout] = useState<LayoutResult | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const { repository } = useRepository();

  const currentBranch =
    repository?.branches.find((b) => b.is_current)?.name ?? null;

  const fetchGraph = useCallback(() => {
    invoke<CommitGraph>("get_graph")
      .then((graph) => setLayout(buildLayout(graph.nodes, graph.edges)))
      .catch(console.error);
  }, []);

  useEffect(() => {
    fetchGraph();
  }, [fetchGraph, repository]);

  const handleScroll = useCallback(() => {
    setScrollTop(scrollRef.current?.scrollTop ?? 0);
  }, []);

  const virtualizer = useVirtualizer({
    count: layout?.rows.length ?? 0,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_H,
    overscan: 8,
  });

  if (!layout) return <div className="history-empty">Loading history…</div>;

  const { rows, totalLanes, segments } = layout;
  const graphWidth = totalLanes * LANE_W + LANE_W;
  const vItems = virtualizer.getVirtualItems();
  const visibleStart = vItems[0]?.index ?? 0;
  const visibleEnd = vItems[vItems.length - 1]?.index ?? 0;

  return (
    <div className="history-container">
      <div className="history-graph-col" style={{ width: graphWidth }}>
        <GraphColumn
          rows={rows}
          segments={segments}
          totalLanes={totalLanes}
          scrollTop={scrollTop}
          visibleStart={visibleStart}
          visibleEnd={visibleEnd}
        />
      </div>
      <div className="history-scroll" ref={scrollRef} onScroll={handleScroll}>
        <div
          style={{ height: virtualizer.getTotalSize(), position: "relative" }}
        >
          {vItems.map((vItem) => (
            <CommitRow
              key={rows[vItem.index].node.id}
              row={rows[vItem.index]}
              graphWidth={graphWidth}
              currentBranch={currentBranch}
              style={{
                position: "absolute",
                top: vItem.start,
                left: 0,
                right: 0,
                height: ROW_H,
              }}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
