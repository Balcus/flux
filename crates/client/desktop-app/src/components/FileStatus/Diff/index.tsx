import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import "../../../App.css";
import "./Diff.css";

export default function Diff({ path }: { path: string | null }) {
  const [diff, setDiff] = useState<string | null>(null);

  useEffect(() => {
    if (!path) {
      setDiff(null);
      return;
    }
    invoke<string>("get_diff", { path })
      .then(setDiff)
      .catch(() => setDiff(null));
  }, [path]);

  const renderDiff = (raw: string) => {
    const lines = raw.split("\n");
    const result: React.ReactNode[] = [];

    lines.forEach((line, i) => {
      if (line.startsWith("@")) {
        if (i > 0) {
          result.push(
            <div key={`sep-${i}`} className="diff-separator">
              <span>···</span>
            </div>,
          );
        }
        result.push(
          <div key={i} className="diff-line diff-meta">
            {line}
          </div>,
        );
      } else {
        const cls = line.startsWith("+")
          ? "diff-add"
          : line.startsWith("-")
            ? "diff-del"
            : "diff-ctx";
        result.push(
          <div key={i} className={`diff-line ${cls}`}>
            {line}
          </div>,
        );
      }
    });

    return result;
  };

  return (
    <div className="diff-pane">
      <div className="diff-header">
        <h1>
          {path ?? "Select a file to see the diff"}
        </h1>
      </div>
      <div className="diff-body">
        {diff ? (
          <div className="diff-content">{renderDiff(diff)}</div>
        ) : (
          <div className="diff-empty">No diff to display.</div>
        )}
      </div>
    </div>
  );
}
