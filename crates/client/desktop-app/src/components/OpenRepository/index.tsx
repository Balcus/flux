import { useEffect } from "react";
import { useRepository } from "../../context/RepositoryContext";
import OpenRepositoryBg from "../../assets/images";
import { toast } from "react-toastify";
import { CloneIcon, OpenIcon } from "../../assets/icons";

import "./OpenRepository.css";
import "../../App.css";

export default function OpenRepository() {
  const { openRepository, isLoading, error } = useRepository();

  useEffect(() => {
    if (error) {
      toast.error(
        <div>
          <div
            style={{
              fontWeight: "700",
              fontSize: "14px",
              marginBottom: "2px",
            }}
          >
            Failed to open repository
          </div>
          <div style={{ fontSize: "12px", opacity: 0.8, lineHeight: "1.4" }}>
            {error}
          </div>
        </div>,
        {
          autoClose: 6000,
          toastId: "open-repo-error",
        },
      );
    }
  }, [error]);

  return (
    <div
      className="container"
      style={{ backgroundImage: `url(${OpenRepositoryBg})` }}
    >
      <div>
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
            onClick={openRepository}
            disabled={isLoading}
          >
            <img className="icon" src={CloneIcon} alt="" />
            <span>Clone</span>
          </button>
        </div>
      </div>
    </div>
  );
}
