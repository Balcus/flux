import { useEffect } from "react";
import { useRepository } from "../../context/RepositoryContext";
import "./OpenRepository.css";
import "../../App.css";
import OpenRepositoryBg from "../../assets/images";
import { toast } from "react-toastify";

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
        <button
          className="open-repo-button"
          onClick={openRepository}
          disabled={isLoading}
        >
          Open Repository
        </button>
      </div>
    </div>
  );
}
