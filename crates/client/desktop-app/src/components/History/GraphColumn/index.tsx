import { EdgeSegment, RowMeta } from "../../../models/HistoryGraphLayout";
import { LANE_CONFIG } from "../../../constants";

interface GraphColumnProps {
  rows: RowMeta[];
  segments: EdgeSegment[];
  totalLanes: number;
  scrollTop: number;
  visibleStart: number;
  visibleEnd: number;
}

export function GraphColumn({
  rows,
  segments,
  totalLanes,
  scrollTop,
  visibleStart,
  visibleEnd,
}: GraphColumnProps) {
  const { LANE_W, ROW_H, DOT_R } = LANE_CONFIG;
  const width = totalLanes * LANE_W + LANE_W;
  const totalHeight = rows.length * ROW_H;
  const pad = 6;
  const rs = Math.max(0, visibleStart - pad);
  const re = Math.min(rows.length - 1, visibleEnd + pad);

  const visSegs = segments.filter(
    (s) =>
      Math.max(s.fromRow, s.toRow) >= rs && Math.min(s.fromRow, s.toRow) <= re,
  );

  return (
    <svg
      width={width}
      height={totalHeight}
      style={{ display: "block", transform: `translateY(-${scrollTop}px)` }}
    >
      {visSegs.map((seg, i) => {
        const x1 = seg.fromLane * LANE_W + LANE_W / 2;
        const y1 = seg.fromRow * ROW_H + ROW_H / 2;
        const x2 = seg.toLane * LANE_W + LANE_W / 2;
        const y2 = seg.toRow * ROW_H + ROW_H / 2;
        if (x1 === x2)
          return (
            <line
              key={i}
              x1={x1}
              y1={y1}
              x2={x2}
              y2={y2}
              stroke={seg.color}
              strokeWidth={1.5}
              strokeOpacity={0.9}
            />
          );
        const midY = (y1 + y2) / 2;
        return (
          <path
            key={i}
            fill="none"
            stroke={seg.color}
            strokeWidth={1.5}
            strokeOpacity={0.9}
            d={`M ${x1} ${y1} C ${x1} ${midY}, ${x2} ${midY}, ${x2} ${y2}`}
          />
        );
      })}
      {rows.slice(rs, re + 1).map((row, off) => {
        const ri = rs + off;
        return (
          <circle
            key={row.node.id}
            cx={row.lane * LANE_W + LANE_W / 2}
            cy={ri * ROW_H + ROW_H / 2}
            r={DOT_R}
            fill={row.color}
            stroke="var(--color-bg-secondary)"
            strokeWidth={1.5}
          />
        );
      })}
    </svg>
  );
}
