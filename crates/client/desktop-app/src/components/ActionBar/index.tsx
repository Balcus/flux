import { ACTION_BAR_ITEMS, ActionBarItem } from "../../constants";

import "../../App.css";
import "./ActionBar.css";

export default function ActionBar() {
  const commit = (): void => {};

  const renderActionBarItem = (item: ActionBarItem) => {
    return (
      <li key={item.id} className="action-button-wrapper">
        <button onClick={commit}>
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
    </nav>
  );
}
