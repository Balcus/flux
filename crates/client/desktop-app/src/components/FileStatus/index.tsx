import ActionBar from "../ActionBar";

import '../../App.css';
import "./FileStatus.css";

export default function FileStatus() {
  // const [selectedFile, setSelectedFile] = useState<string | null>(null);

  return (
    <div className="file-status-grid">
      <div className="action-bar">
        <ActionBar />
      </div>
      <div className="tracked">

      </div>
      <div className="untracked">

      </div>
      <div className="diffs">

      </div>
    </div>
  );
}
