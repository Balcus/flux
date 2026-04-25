import { useState } from "react";
import { ACTION_BAR_ITEMS, ActionBarItem } from "../../constants";
import { useRepository } from "../../context/RepositoryContext";
import CommitPopup from "../FileStatus/CommitPopup";
import { CloseIcon } from "../../assets/icons";

import "./ActionBar.css";

export default function ActionBar() {
  const [isPopupOpen, setIsPopupOpen] = useState(false);
  const { closeRepository } = useRepository();

  const renderActionBarItem = (item: ActionBarItem) => {
    const isCommit = item.name.toLowerCase() === "commit";
    return (
      <li key={item.id}>
        <button onClick={isCommit ? () => setIsPopupOpen(true) : undefined}>
          <img src={item.icon} alt={item.name} className="action-item-icon" />
        </button>
        <span className="action-item-label">{item.name}</span>
      </li>
    );
  };

  return (
    <nav>
      <ul className="action-bar-items">
        {ACTION_BAR_ITEMS.map(renderActionBarItem)}
        <li className="action-bar-close">
          <button onClick={closeRepository} title="Close repository">
            <img src={CloseIcon} alt="Close" className="action-item-icon" />
          </button>
          <span className="action-item-label">Close</span>
        </li>
      </ul>
      <CommitPopup isOpen={isPopupOpen} onClose={() => setIsPopupOpen(false)} />
    </nav>
  );
}