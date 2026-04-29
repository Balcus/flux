import { useState, useEffect } from "react";
import {
  BRANCH_CONTEXT_MENU_ID,
  BRANCHES_HEADER_CONTEXT_MENU_ID,
  MENU_ITEMS,
  MenuItem,
} from "../../constants";
import { useRepository } from "../../context/RepositoryContext";
import { Branch } from "../../models/Branch";
import { useNavigate } from "react-router-dom";
import {
  useContextMenu,
  Menu as ContextMenu,
  Item,
  ItemParams,
} from "react-contexify";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "react-toastify";
import Popup from "../common/Popup";

import "./Menu.css";
import "react-contexify/ReactContexify.css";

export default function Menu() {
  const [expandedItems, setExpandedItems] = useState<string[]>([
    "workspace",
    "branches",
  ]);
  const [branches, setBranches] = useState<MenuItem[]>([]);
  const [openNewBranchPopup, setOpenNewBranchPopup] = useState<boolean>(false);
  const [newBranchName, setNewBranchName] = useState<string>();
  const { repository, refreshRepository } = useRepository();
  const { show } = useContextMenu({ id: BRANCH_CONTEXT_MENU_ID });
  const nav = useNavigate();

  useEffect(() => {
    if (repository?.branches) {
      setBranches(
        repository.branches.map((branch: Branch) => ({
          id: `branch-${branch.name}`,
          label: branch.name,
          className: branch.is_current ? "bold" : "",
        })),
      );
    }
  }, [repository]);

  const handleSwitch = async ({ props, data }: ItemParams) => {
    const name = props?.branchName;
    const force = data?.force || false;

    if (name) {
      try {
        await invoke("switch_branch", {
          name,
          force,
        });

        await refreshRepository();
        toast.success(`Switched to branch ${name} ${force ? "(force)" : ""}`);
      } catch (err) {
        toast.error(`Switch failed: ${err}`);
      }
    }
  };

  const handleDelete = async ({ props }: ItemParams) => {
    const name = props?.branchName;

    if (name) {
      try {
        await invoke("delete_branch", { name });
        await refreshRepository();
        toast.success(`Deleted branch ${name}`);
      } catch (err) {
        toast.error(`Delete failed: ${err}`);
      }
    }
  };

  const handleRename = ({ props }: ItemParams) => {
    console.log("rename:", props?.branchName);
  };

  const handleCreateNewBranch = async () => {
    if (!newBranchName) {
      return;
    }

    try {
      await invoke("create_branch", { name: newBranchName });
      await refreshRepository();
      toast.success(`Created branch ${newBranchName}`);
    } catch (error: any) {
      toast.error(error);
    } finally {
      setOpenNewBranchPopup(false);
    }
  };

  const handleItemClick = (item: MenuItem) => {
    if (item.children?.length) {
      setExpandedItems((prev) =>
        prev.includes(item.id)
          ? prev.filter((i) => i !== item.id)
          : [...prev, item.id],
      );
    } else if (item.link) {
      nav(item.link);
    }
  };

  const renderMenuItem = (
    item: MenuItem,
    isChild = false,
    contextMenuId?: string,
  ) => {
    const effectiveMenuId =
      item.id === "branches" ? BRANCHES_HEADER_CONTEXT_MENU_ID : contextMenuId;

    return (
      <li key={item.id} className={isChild ? "child" : ""}>
        <button
          onClick={() => handleItemClick(item)}
          onContextMenu={(e) => {
            if (effectiveMenuId) {
              show({
                event: e,
                id: effectiveMenuId,
                props: { branchName: item.label },
              });
            }
          }}
          className={`${isChild ? "" : "section-header"} ${item.className || ""}`}
        >
          {!isChild && (
            <span style={{ marginRight: 6, fontSize: 10 }}>
              {expandedItems.includes(item.id) ? "▾" : "▸"}
            </span>
          )}
          {item.icon && (
            <img
              src={item.icon}
              alt=""
              style={{ width: 14, height: 14, marginRight: 6, opacity: 0.5 }}
            />
          )}
          {item.label}
        </button>
        {item.children && expandedItems.includes(item.id) && (
          <ul>
            {item.children.map((child) =>
              renderMenuItem(child, true, item.contextMenuId),
            )}
          </ul>
        )}
      </li>
    );
  };

  const finalMenu = MENU_ITEMS.map((item) =>
    item.id === "branches" ? { ...item, children: branches } : item,
  );

  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <h1>flux</h1>
        <p>Version Control System</p>
      </div>
      <nav>
        <ul>{finalMenu.map((item) => renderMenuItem(item))}</ul>
      </nav>

      <ContextMenu id={BRANCH_CONTEXT_MENU_ID}>
        <Item onClick={handleSwitch} data={{ force: false }}>
          Switch
        </Item>
        <Item onClick={handleSwitch} data={{ force: true }}>
          Force Switch
        </Item>
        <Item onClick={handleRename}>Rename</Item>
        <Item onClick={handleDelete}>Delete</Item>
      </ContextMenu>

      <ContextMenu id={BRANCHES_HEADER_CONTEXT_MENU_ID}>
        <Item onClick={() => setOpenNewBranchPopup(true)}>New</Item>
      </ContextMenu>

      <Popup
        showPopUp={openNewBranchPopup}
        closePopUp={() => setOpenNewBranchPopup(false)}
      >
        <div className="new-branch-popup">
          <h2>Create new branch</h2>
          <input
            type="text"
            placeholder="Branch name"
            value={newBranchName}
            onChange={(e) => setNewBranchName(e.target.value)}
            autoFocus
          />

          <footer>
            <button onClick={() => setOpenNewBranchPopup(false)}>Cancel</button>
            <button
              className="primary"
              disabled={!newBranchName}
              onClick={handleCreateNewBranch}
            >
              Create
            </button>
          </footer>
        </div>
      </Popup>
    </div>
  );
}
