import { useEffect, useState } from "react";
import { useRepository } from "../../context/RepositoryContext";
import { open } from "@tauri-apps/plugin-dialog";
import OpenRepositoryBg from "../../assets/images";
import { toast } from "react-toastify";
import { BrowseIcon, CloneIcon, OpenIcon } from "../../assets/icons";
import Popup from "../Shared/Popup";

import "./OpenRepository.css";
import "../../App.css";

export default function OpenRepository() {
  const { openRepository, isLoading, error } = useRepository();
  const [openClonePopup, setOpenClonePopup] = useState(false);
  const [repoUrl, setRepoUrl] = useState("");
  const [destPath, setDestPath] = useState("");

  const handleSelectPath = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select Destination Folder",
    });
    if (selected) setDestPath(selected as string);
  };

  useEffect(() => {
    if (error) {
      toast.error(
        <div>
          <div
            style={{ fontWeight: "700", fontSize: "14px", marginBottom: "2px" }}
          >
            Failed to open repository
          </div>
          <div style={{ fontSize: "12px", opacity: 0.8, lineHeight: "1.4" }}>
            {error}
          </div>
        </div>,
        { autoClose: 6000, toastId: "open-repo-error" },
      );
    }
  }, [error]);

  return (
    <div
      className={`container ${openClonePopup ? "blurred" : ""}`}
      style={{ backgroundImage: `url(${OpenRepositoryBg})` }}
    >
      <div className="main-content">
        <h1 className="title">flux</h1>
        <p className="description">Distributed Version Control made Easy</p>
        <div className="repo-controls">
          <button
            className="repo-control-button"
            onClick={openRepository}
            disabled={isLoading}
          >
            <img className="icon" src={OpenIcon} alt="" />
            <span>Open</span>
          </button>
          <button
            className="repo-control-button"
            onClick={() => setOpenClonePopup(true)}
            disabled={isLoading}
          >
            <img className="icon" src={CloneIcon} alt="" />
            <span>Clone</span>
          </button>
        </div>
      </div>

      <Popup
        showPopUp={openClonePopup}
        closePopUp={() => setOpenClonePopup(false)}
      >
        <div className="clone-content">
          <h2>Clone Repository</h2>

          <label className="input-label">Destination Folder</label>
          <div className="path-input-container">
            <input
              type="text"
              placeholder="/users/desktop/my-repo"
              value={destPath}
              readOnly
            />
            <button className="btn-browse-icon" onClick={handleSelectPath}>
              <img className="icon-small" src={BrowseIcon} alt="Browse" />
            </button>
          </div>

          <label className="input-label">Repository URL</label>
          <input
            type="text"
            placeholder="Remote repository url"
            value={repoUrl}
            onChange={(e) => setRepoUrl(e.target.value)}
            autoFocus
          />

          <div className="clone-actions">
            <button
              className="btn-secondary"
              onClick={() => setOpenClonePopup(false)}
            >
              Cancel
            </button>
            <button
              className="btn-primary"
              disabled={!repoUrl || !destPath}
              onClick={() => {
                setOpenClonePopup(false);
              }}
            >
              Clone
            </button>
          </div>
        </div>
      </Popup>
    </div>
  );
}
