import { useState, useEffect } from "react";
import { MENU_ITEMS, MenuItem } from "../../constants";
import { useRepository } from "../../context/RepositoryContext";
import { Branch } from "../../models/Branch";
import { useNavigate } from "react-router-dom";

import "../../App.css";
import "./Menu.css";

export default function Menu() {
  const [expandedItems, setExpandedItems] = useState<string[]>(["workspace", "branches"]);
  const [branches, setBranches] = useState<MenuItem[]>([]);
  const { repository } = useRepository();
  const nav = useNavigate();

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

  const handleItemClick = (item: MenuItem) => {
    if (item.children && item.children.length > 0) {
      setExpandedItems((prev) =>
        prev.includes(item.id) ? prev.filter((i) => i !== item.id) : [...prev, item.id]
      );
    } else if (item.link) {
      nav(item.link);
    }
  };

  const renderMenuItem = (item: MenuItem, isChild = false) => (
    <li key={item.id} className={isChild ? "menu-child-item" : "menu-item"}>
      <button
        onClick={() => handleItemClick(item)}
        className={`menu-button ${isChild ? "child" : "parent"}`}
      >
        {!isChild && item.children && item.children.length > 0 && (
          <span className="toggle-icon">
            {expandedItems.includes(item.id) ? "⌄" : "›"}
          </span>
        )}
        {item.icon && <img className="icon" src={item.icon} alt="" />}
        <span className={`label ${item.className || ""}`}>
          {item.label}
        </span>
      </button>
      
      {item.children && expandedItems.includes(item.id) && item.children.length > 0 && (
        <ul className="submenu">
          {item.children.map((child) => renderMenuItem(child, true))}
        </ul>
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