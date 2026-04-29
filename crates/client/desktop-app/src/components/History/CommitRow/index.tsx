import { RowMeta } from "../../../models/HistoryGraphLayout";
import { HeadChip } from "../HeadChip";

import './CommitRow.css';

interface CommitRowProps {
  row: RowMeta;
  graphWidth: number;
  style: React.CSSProperties;
}

export function CommitRow({ row, graphWidth, style }: CommitRowProps) {
  const { node } = row;
  return (
    <div className="commit-row" style={style}>
      <div style={{ width: graphWidth, flexShrink: 0 }} />
      <div className="commit-row__info">
        <div className="commit-row__top">
          {node.branches.map((branch) => (
            <HeadChip name={branch} color={row.color} />
          ))}
          <span className="commit-row__message">{node.message}</span>
        </div>
        <div className="commit-row__bottom">
          <span className="commit-row__author">{node.author}</span>
          <code className="commit-row__hash">{node.short_id}</code>
        </div>
      </div>
    </div>
  );
}
