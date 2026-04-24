import { useState, useEffect } from "react";
import { MENU_ITEMS, MenuItem } from "../../constants";
import { useRepository } from "../../context/RepositoryContext";
import { Branch } from "../../models/Branch";
import { useNavigate } from "react-router-dom";

import "./Menu.css";

export default function Menu() {
  const [expandedItems, setExpandedItems] = useState<string[]>([
    "workspace",
    "branches",
  ]);
  const [branches, setBranches] = useState<MenuItem[]>([]);
  const { repository } = useRepository();
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

  const renderMenuItem = (item: MenuItem, isChild = false) => (
    <li key={item.id} className={isChild ? "child" : ""}>
      <button
        onClick={() => handleItemClick(item)}
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
        <ul>{item.children.map((child) => renderMenuItem(child, true))}</ul>
      )}
    </li>
  );

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
    </div>
  );
}
