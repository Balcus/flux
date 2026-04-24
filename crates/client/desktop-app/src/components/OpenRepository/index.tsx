import { useEffect, useState } from "react";
import { useRepository } from "../../context/RepositoryContext";
import { open } from "@tauri-apps/plugin-dialog";
import OpenRepositoryBg from "../../assets/images";
import { toast } from "react-toastify";
import { BrowseIcon, CloneIcon, OpenIcon } from "../../assets/icons";
import Popup from "../Shared/Popup";

import "./OpenRepository.css";

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
      className={`open-root${openClonePopup ? " dimmed" : ""}`}
      style={{ backgroundImage: `url(${OpenRepositoryBg})` }}
    >
      <div>
        <h1>flux</h1>
        <p>Distributed Version Control made Easy</p>
        <div className="open-controls">
          <button onClick={openRepository} disabled={isLoading}>
            <img src={OpenIcon} alt="" />
            <span>Open</span>
          </button>
          <button onClick={() => setOpenClonePopup(true)} disabled={isLoading}>
            <img src={CloneIcon} alt="" />
            <span>Clone</span>
          </button>
        </div>
      </div>

      <Popup
        showPopUp={openClonePopup}
        closePopUp={() => setOpenClonePopup(false)}
      >
        <div className="clone-popup">
          <h2>Clone Repository</h2>
          <label>Destination Folder</label>
          <div className="path-row">
            <input
              type="text"
              placeholder="/users/desktop/my-repo"
              value={destPath}
              readOnly
            />
            <button onClick={handleSelectPath}>
              <img
                src={BrowseIcon}
                alt="Browse"
                style={{ width: 22, height: 22 }}
              />
            </button>
          </div>
          <label>Repository URL</label>
          <input
            type="text"
            placeholder="Remote repository url"
            value={repoUrl}
            onChange={(e) => setRepoUrl(e.target.value)}
            autoFocus
          />
          <footer>
            <button onClick={() => setOpenClonePopup(false)}>Cancel</button>
            <button
              className="primary"
              disabled={!repoUrl || !destPath}
              onClick={() => setOpenClonePopup(false)}
            >
              Clone
            </button>
          </footer>
        </div>
      </Popup>
    </div>
  );
}
