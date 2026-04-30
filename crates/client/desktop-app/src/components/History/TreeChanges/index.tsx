import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { toast } from "react-toastify/unstyled";

import "./TreeChanges.css";

interface TreeChangesProps {
  commitId: string;
}

export default function TreeChanges({ commitId }: TreeChangesProps) {
  const [treeChanges, setTreeChanges] = useState<string[]>([]);

  const fetchTreeChanges = async (commitId: string) => {
    try {
      let res = await invoke<string[]>("get_tree_changes", {
        commitId,
      });
      console.log("commitId sent:", commitId);
      console.log("result:", res);
      setTreeChanges(res);
    } catch (error: any) {
      console.error("error:", error);
      toast.error("Failed to get tree changes");
    }
  };

  useEffect(() => {
    fetchTreeChanges(commitId);
  }, [commitId]);

  return (
    <div className="tree-changes">
      <p className="tree-changes-title">Changed files</p>
      {treeChanges.length === 0 ? (
        <p className="tree-changes-empty">No changes</p>
      ) : (
        <ul className="tree-changes-list">
          {treeChanges.map((file) => (
            <li key={file} className="tree-changes-item">
              {file}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
