import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

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
    const result: React.ReactNode[] = [];
    raw.split("\n").forEach((line, i) => {
      if (line.startsWith("@")) {
        if (i > 0)
          result.push(
            <div key={`sep-${i}`} className="diff-separator">
              <span>···</span>
            </div>,
          );
        result.push(
          <div key={i} className="diff-line meta">
            {line}
          </div>,
        );
      } else {
        const cls = line.startsWith("+")
          ? "add"
          : line.startsWith("-")
            ? "del"
            : "";
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
      <header>
        <h1>{path ?? "Select a file to see the diff"}</h1>
      </header>
      <div>
        {diff ? (
          <div style={{ padding: "8px 0" }}>{renderDiff(diff)}</div>
        ) : (
          <div className="diff-empty">No diff to display.</div>
        )}
      </div>
    </div>
  );
}
