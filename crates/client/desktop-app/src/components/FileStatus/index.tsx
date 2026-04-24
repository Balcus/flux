import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import ActionBar from "../ActionBar";
import { useRepository } from "../../context/RepositoryContext";
import { StagedFile } from "../../models/StagedFile";
import FileRow from "./FileRow";
import Diff from "./Diff";

import "./FileStatus.css";

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
    for (const f of repository?.status.unstaged ?? [])
      await invoke("add", { path: f.path });
    for (const f of repository?.status.untracked ?? [])
      await invoke("add", { path: f });
    await refreshRepository();
  };

  const handleUnstageAll = async () => {
    for (const f of repository?.status.staged ?? [])
      await invoke("restore", { path: f.path });
    await refreshRepository();
  };

  return (
    <div className="fs-root">
      <ActionBar />
      <div className="fs-sidebar">
        <div className="fs-section staged">
          <header>
            <span>Staged</span>
            <button onClick={handleUnstageAll}>Unstage all</button>
          </header>
          <ul>
            {repository?.status.staged.map((item: StagedFile) => (
              <FileRow
                key={item.path}
                path={item.path}
                type={item.change_type}
                selected={selectedFile === item.path}
                onSelect={() => setSelectedFile(item.path)}
                onAction={() => handleUnstage(item.path)}
                actionLabel="-"
              />
            ))}
          </ul>
        </div>
        <div className="fs-section">
          <header>
            <span>Unstaged</span>
            <button onClick={handleStageAll}>Stage all</button>
          </header>
          <ul>
            {repository?.status.unstaged.map((item: StagedFile) => (
              <FileRow
                key={item.path}
                path={item.path}
                type={item.change_type}
                selected={selectedFile === item.path}
                onSelect={() => setSelectedFile(item.path)}
                onAction={() => handleStage(item.path)}
                actionLabel="+"
              />
            ))}
            {repository?.status.untracked.map((path: string) => (
              <FileRow
                key={path}
                path={path}
                selected={selectedFile === path}
                onSelect={() => setSelectedFile(path)}
                onAction={() => handleStage(path)}
                actionLabel="+"
              />
            ))}
          </ul>
        </div>
      </div>
      <Diff path={selectedFile} />
    </div>
  );
}