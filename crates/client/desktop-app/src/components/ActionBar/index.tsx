import { useState } from "react";
import { ACTION_BAR_ITEMS, ActionBarItem } from "../../constants";

import "../../App.css";
import "./ActionBar.css";
import CommitPopup from "../FileStatus/CommitPopup";

export default function ActionBar() {
  const [isPopupOpen, setIsPopupOpen] = useState(false);

  const togglePopup = (): void => {
    setIsPopupOpen(!isPopupOpen);
  };

  const renderActionBarItem = (item: ActionBarItem) => {
    const isCommit = item.name.toLowerCase() === "commit";

    return (
      <li key={item.id} className="action-button-wrapper">
        <button onClick={isCommit ? togglePopup : undefined}>
          <img src={item.icon} alt={item.name} className="action-item-icon" />
        </button>
        <span className="action-item-label">{item.name}</span>
      </li>
    );
  };

  return (
    <nav>
      <ul className="action-bar-items">
        {ACTION_BAR_ITEMS.map((item) => renderActionBarItem(item))}
      </ul>
      <CommitPopup isOpen={isPopupOpen} onClose={() => setIsPopupOpen(false)} />
    </nav>
  );
}
