import { useEffect, useState, useRef } from "react";
import { Commit } from "../../models/Commit";
import { invoke } from "@tauri-apps/api/core";
import { Gitgraph, Orientation } from "@gitgraph/react";

import "./History.css";

export default function History() {
  const [commits, setCommits] = useState<Commit[]>([]);
  const built = useRef(false);

  const fetchCommits = () => {
    invoke<Commit[]>("get_commits").then(setCommits).catch(console.error);
  };

  useEffect(() => {
    fetchCommits();
  }, []);

  const renderGraph = (gitgraph: any) => {
    if (built.current) return;
    built.current = true;

    const branches: Record<string, any> = {};
    const gitgraphCommits: Record<string, any> = {};

    commits.forEach((commit) => {
      if (!branches[commit.branch]) {
        const parentInGraph = commit.parent
          ? gitgraphCommits[commit.parent]
          : null;

        branches[commit.branch] = parentInGraph
          ? gitgraph.branch({ name: commit.branch, from: parentInGraph })
          : gitgraph.branch(commit.branch);
      }

      gitgraphCommits[commit.id] = branches[commit.branch].commit({
        subject: `${commit.id.slice(0, 5)} ${commit.message}`,
        author: commit.author,
        hash: commit.id.slice(0, 7),
      });
    });
  };

  return (
    <div className="history-container">
      <div className="history-list">
        <div className="history-graph">
          {commits.length > 0 && (
            // @ts-ignore
            <Gitgraph options={{ orientation: Orientation.Vertical }}>
              {renderGraph}
            </Gitgraph>
          )}
        </div>
      </div>
    </div>
  );
}
