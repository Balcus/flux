import { useRepository } from "../../context/RepositoryContext";
import "./OpenRepository.css";
import "../../App.css"
import OpenRepositoryBg from "../../assets/images";

export default function OpenRepository() {
  const { openRepository, isLoading, error } = useRepository();
  
  return (
    <div className="container" style={{ backgroundImage: `url(${OpenRepositoryBg})` }}>
      <div>
        <h1 style={{ color: 'var(--title-color)' }}>flux</h1>
        <p style={{ fontSize: '1.25rem', color: 'var(--subtitle-color)', margin: '0.2rem 0 2rem 0' }}>
          Version control made easy
        </p>
        <button
          className="open-repo-button"
          onClick={openRepository}
          disabled={isLoading}
        >
          Open Repository
        </button>
        {error && <div className="error-message">{error}</div>}
      </div>
    </div>
  );
}