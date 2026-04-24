import { useState } from "react";
import { useRepository } from "../../../context/RepositoryContext";
import { StagedFile } from "../../../models/StagedFile";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "react-toastify";

import "./CommitPopup.css";

export default function CommitPopup({
  isOpen,
  onClose,
}: {
  isOpen: boolean;
  onClose: () => void;
}) {
  const { repository, refreshRepository } = useRepository();
  const [message, setMessage] = useState("");
  const [loading, setLoading] = useState(false);
  const staged = repository?.status.staged;

  if (!isOpen) return null;

  const handleCommit = async () => {
    if (!message.trim()) {
      toast.error("Commit message cannot be empty");
      return;
    }
    if (!staged?.length) {
      toast.error("Nothing to commit");
      return;
    }
    try {
      setLoading(true);
      await invoke<string>("commit", { message });
      toast.success("Changes committed.");
      await refreshRepository();
      setMessage("");
      onClose();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="popup-overlay" onClick={onClose}>
      <div className="commit-popup" onClick={(e) => e.stopPropagation()}>
        <header>Changes staged for commit</header>
        <ul>
          {staged?.map((item: StagedFile) => (
            <li key={item.path}>
              {item.path}
              {item.change_type && <span>{item.change_type}</span>}
            </li>
          ))}
        </ul>
        <textarea
          placeholder="Commit message"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          disabled={loading}
        />
        <footer>
          <button onClick={onClose} disabled={loading}>
            Cancel
          </button>
          <button
            className="primary"
            onClick={handleCommit}
            disabled={loading || !staged?.length}
          >
            {loading ? "Committing..." : "Commit"}
          </button>
        </footer>
      </div>
    </div>
  );
}
