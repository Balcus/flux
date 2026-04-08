import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import ActionBar from "../ActionBar";
import { useRepository } from "../../context/RepositoryContext";

import "../../App.css";
import "./FileStatus.css";

export default function FileStatus() {
  const { repository, refreshRepository } = useRepository();

  useEffect(() => {
    const interval = setInterval(() => {
      refreshRepository();
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleStage = async (path: string) => {
    await invoke("add_file", { path });
    await refreshRepository();
  };

  const handleUnstage = async (path: string) => {
    await invoke("unstage_file", { path });
    await refreshRepository();
  };

  return (
    <div className="file-status-grid">
      <div className="action-bar">
        <ActionBar />
      </div>
      <div className="staged">
        <ul>
          {repository?.status.staged.map((item) => (
            <li key={item.path}>
              <span>{item.path}</span>
              <button onClick={() => handleUnstage(item.path)}>-</button>
            </li>
          ))}
        </ul>
      </div>
      <div className="untracked">
        <ul>
          {repository?.status.untracked.map((item) => (
            <li key={item}>
              <span>{item}</span>
              <button onClick={() => handleStage(item)}>+</button>
            </li>
          ))}
        </ul>
      </div>
      <div className="diffs"></div>
    </div>
  );
}
