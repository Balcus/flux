import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import ActionBar from "../ActionBar";
import { useRepository } from "../../context/RepositoryContext";
import { StagedFile } from "../../models/StagedFile";
import FileRow from "./FileRow";
import "../../App.css";
import "./FileStatus.css";
import Diff from "./Diff";

export default function FileStatus() {
  const { repository, refreshRepository } = useRepository();
  const [selectedFile, setSelectedFile] = useState<string | null>(null);

  useEffect(() => {
    const interval = setInterval(refreshRepository, 2000);
    return () => clearInterval(interval);
  }, []);

  const handleStage = async (path: string) => {
    await invoke("add", { path });
    await refreshRepository();
  };

  const handleUnstage = async (path: string) => {
    await invoke("restore", { path });
    await refreshRepository();
  };

  const handleStageAll = async () => {
    for (const f of repository?.status.unstaged ?? []) await invoke("add", { path: f.path });
    for (const f of repository?.status.untracked ?? []) await invoke("add", { path: f });
    await refreshRepository();
  };

  const handleUnstageAll = async () => {
    for (const f of repository?.status.staged ?? []) await invoke("restore", { path: f.path });
    await refreshRepository();
  };

  return (
    <div className="file-status-root">
      <div className="action-bar"><ActionBar /></div>

      <div className="fs-sidebar">
        <div className="fs-section">
          <div className="fs-section-header">
            <h1>staged</h1>
            <button onClick={handleUnstageAll}>↓ Unstage All</button>
          </div>
          <div className="fs-file-list">
            {repository?.status.staged.map((item: StagedFile) => (
              <FileRow
                key={item.path}
                path={item.path}
                type={item.change_type}
                selected={selectedFile === item.path}
                onSelect={() => setSelectedFile(item.path)}
                onAction={() => handleUnstage(item.path)}
                actionLabel="↓"
              />
            ))}
          </div>
        </div>

        <div className="fs-section">
          <div className="fs-section-header">
            <h1>unstaged</h1>
            <button onClick={handleStageAll}>↑ Stage All</button>
          </div>
          <div className="fs-file-list">
            {repository?.status.unstaged.map((item: StagedFile) => (
              <FileRow
                key={item.path}
                path={item.path}
                type={item.change_type}
                selected={selectedFile === item.path}
                onSelect={() => setSelectedFile(item.path)}
                onAction={() => handleStage(item.path)}
                actionLabel="↑"
              />
            ))}
            {repository?.status.untracked.map((path: string) => (
              <FileRow
                key={path}
                path={path}
                selected={selectedFile === path}
                onSelect={() => setSelectedFile(path)}
                onAction={() => handleStage(path)}
                actionLabel="↑"
              />
            ))}
          </div>
        </div>
      </div>

      <div className="fs-diff-pane">
        <Diff path={selectedFile} />
      </div>
    </div>
  );
}