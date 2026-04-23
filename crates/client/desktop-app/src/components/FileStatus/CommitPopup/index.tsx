import { useState } from "react";
import { useRepository } from "../../../context/RepositoryContext";
import { StagedFile } from "../../../models/StagedFile";

import "./CommitPopup.css";
import "../../../App.css";

interface CommitPopupProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function CommitPopup({ isOpen, onClose }: CommitPopupProps) {
  const { repository } = useRepository();
  const [message, setMessage] = useState("");
  const staged = repository?.status.staged;

  if (!isOpen) return null;

  const handleCommit = () => {
    console.log("Committing:", message);
    onClose();
  };

  return (
    <div className="popup-overlay" onClick={onClose}>
      <div
        className="commit-popup-container"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="popup-header">
          <span className="header-title">Changes staged for commit</span>
        </div>

        <div className="staged-files-section">
          <ul className="staged-list">
            {staged?.map((item: StagedFile) => (
              <li key={item.path} className="staged-item">
                <span className="status-bullet">•</span>
                <span className="file-path">{item.path}</span>
                {item.change_type && <span className="file-status">[{item.change_type}]</span>}
              </li>
            ))}
          </ul>
        </div>

        <div className="commit-message-section">
          <textarea
            className="commit-textarea"
            placeholder="Commit message"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
          />
        </div>

        <div className="popup-actions">
          <button className="action-btn commit-btn" onClick={handleCommit}>
            Commit
          </button>
          <button className="action-btn cancel-btn" onClick={onClose}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}