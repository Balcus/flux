import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { toast } from "react-toastify/unstyled";

import "./TreeContents.css";

interface TreeContentsProps {
  commitId: string;
}

export default function TreeContents({ commitId }: TreeContentsProps) {
  const [TreeContents, setTreeContents] = useState<string[]>([]);

  const fetchTreeContents = async (commitId: string) => {
    try {
      let res = await invoke<string[]>("get_tree_changes", {
        commitId,
      });
      console.log("commitId sent:", commitId);
      console.log("result:", res);
      setTreeContents(res);
    } catch (error: any) {
      console.error("error:", error);
      toast.error("Failed to get tree changes");
    }
  };

  useEffect(() => {
    fetchTreeContents(commitId);
  }, [commitId]);

  return (
    <div className="tree-changes">
      <p className="tree-changes-title">Files included in this commit</p>
      {TreeContents.length === 0 ? (
        <p className="tree-changes-empty">No changes</p>
      ) : (
        <ul className="tree-changes-list">
          {TreeContents.map((file) => (
            <li key={file} className="tree-changes-item">
              {file}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
