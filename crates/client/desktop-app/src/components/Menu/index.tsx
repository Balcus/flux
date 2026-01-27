import { useState, useEffect } from "react";
import { MENU_ITEMS, MenuItem } from "../../constants";
import { useRepository } from "../../context/RepositoryContext";
import { Branch } from "../../models/Branch";
import "../../App.css";
import "./Menu.css";

export default function Menu() {
  const [expandedItems, setExpandedItems] = useState<string[]>([
    "workspace",
    "branches",
  ]);
  const [branches, setBranches] = useState<MenuItem[]>([]);
  const { repository } = useRepository();

  useEffect(() => {
    if (repository?.branches) {
      const branchMenuItems: MenuItem[] = repository.branches.map(
        (branch: Branch) => ({
          id: `branch-${branch.name}`,
          label: branch.name,
          className: branch.is_current ? "current-branch" : "",
        })
      );
      setBranches(branchMenuItems);
    }
  }, [repository]);

  const toggleItem = (id: string) => {
    setExpandedItems((prev) =>
      prev.includes(id) ? prev.filter((i) => i !== id) : [...prev, id]
    );
  };

  const renderMenuItem = (item: MenuItem, isChild = false) => (
    <li key={item.id} className={isChild ? "menu-child-item" : "menu-item"}>
      {item.children !== undefined ? (
        <>
          <button
            onClick={() => toggleItem(item.id)}
            className={`menu-button ${isChild ? "child" : "parent"}`}
          >
            {!isChild && item.children.length > 0 && (
              <span className="toggle-icon">
                {expandedItems.includes(item.id) ? "⌄" : "›"}
              </span>
            )}
            {item.icon && <img className="icon" src={item.icon} alt="" />}
            <span className={`label ${item.className || ""}`}>
              {item.label}
            </span>
          </button>
          {expandedItems.includes(item.id) && item.children.length > 0 && (
            <ul className="submenu">
              {item.children.map((child) => renderMenuItem(child, true))}
            </ul>
          )}
        </>
      ) : (
        <div className="menu-button child">
          {item.icon && <img className="icon" src={item.icon} alt="" />}
          <span className={`label ${item.className || ""}`}>{item.label}</span>
        </div>
      )}
    </li>
  );

  const finalMenu = MENU_ITEMS.map((item) => {
    if (item.id === "branches") {
      return { ...item, children: branches };
    }
    return item;
  });

  return (
    <nav className="sidebar-menu">
      <div className="sidebar-header">
        <div className="app-title">flux</div>
        <div className="app-subtitle">Version Control System</div>
      </div>
      <ul className="menu-list">
        {finalMenu.map((item) => renderMenuItem(item))}
      </ul>
    </nav>
  );
}
