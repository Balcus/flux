import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import ActionBar from "../ActionBar";
import { useRepository } from "../../context/RepositoryContext";
import { StagedFile } from "../../models/StagedFile";
import "../../App.css";
import "./FileStatus.css";

export default function FileStatus() {
  const { repository, refreshRepository } = useRepository();
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [diff, setDiff] = useState<string | null>(null);

  useEffect(() => {
    const interval = setInterval(refreshRepository, 2000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (!selectedFile) {
      setDiff(null);
      return;
    }
    invoke<string>("get_diff", { path: selectedFile })
      .then(setDiff)
      .catch(() => setDiff(null));
  }, [selectedFile]);

  const handleStage = async (path: string) => {
    await invoke("add", { path });
    await refreshRepository();
  };

  const handleUnstage = async (path: string) => {
    await invoke("reset_hard", { path });
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
      await invoke("reset_hard", { path: f.path });
    await refreshRepository();
  };

  const badgeClass = (type: string) =>
    type === "Added"
      ? "badge-add"
      : type === "Deleted"
        ? "badge-del"
        : "badge-mod";

  const badgeLabel = (type: string) =>
    type === "Added" ? "A" : type === "Deleted" ? "D" : "M";

  const FileRow = ({
    path,
    type,
    onAction,
    actionLabel,
  }: {
    path: string;
    type?: string;
    onAction: () => void;
    actionLabel: string;
  }) => (
    <div
      className={`fs-file-row${selectedFile === path ? " active" : ""}`}
      onClick={() => setSelectedFile(path)}
    >
      <span className={`fs-badge ${type ? badgeClass(type) : "badge-add"}`}>
        {type ? badgeLabel(type) : "A"}
      </span>
      <span className="fs-file-name">{path}</span>
      <button
        onClick={(e) => {
          e.stopPropagation();
          onAction();
        }}
      >
        {actionLabel}
      </button>
    </div>
  );

  return (
    <div className="file-status-root">
      <div className="action-bar">
        <ActionBar />
      </div>

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
                onAction={() => handleStage(item.path)}
                actionLabel="↑"
              />
            ))}
            {repository?.status.untracked.map((path: string) => (
              <FileRow
                key={path}
                path={path}
                onAction={() => handleStage(path)}
                actionLabel="↑"
              />
            ))}
          </div>
        </div>
      </div>

      <div className="fs-diff-pane">
        <div className="fs-diff-header">
          {selectedFile ? (
            <h1>{selectedFile}</h1>
          ) : (
            <h1>A file needs to be selected in order to see its diff</h1>
          )}
        </div>
        <div className="fs-diff-body">
          {diff ? (
            <pre>{diff}</pre>
          ) : (
            <div className="fs-empty-diff">No file selected</div>
          )}
        </div>
      </div>
    </div>
  );
}
