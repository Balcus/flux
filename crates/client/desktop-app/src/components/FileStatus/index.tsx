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
    }, 2000);
    return () => clearInterval(interval);
  }, []);

  const handleAdd = async (path: string) => {
    await invoke("add", { path });
    await refreshRepository();
  };

  const handleRm = async (path: string) => {
    await invoke("reset", { path });
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
              <button onClick={() => handleRm(item.path)}>-</button>
            </li>
          ))}
        </ul>
      </div>

      <div className="untracked">
        <ul>
          {repository?.status.unstaged.map((item) => (
            <li key={item.path}>
              <span>{item.path}</span>
              <button onClick={() => handleAdd(item.path)}>+</button>
            </li>
          ))}
          {repository?.status.untracked.map((item) => (
            <li key={item}>
              <span>{item}</span>
              <button onClick={() => handleAdd(item)}>+</button>
            </li>
          ))}
        </ul>
      </div>

      <div className="diffs"></div>
    </div>
  );
}
