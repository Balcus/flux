import React from "react";
import { createPortal } from "react-dom";

import "./Popup.css";

interface PopUpProps {
  showPopUp: boolean;
  children: React.ReactNode;
}

export default function Popup({ showPopUp, children }: PopUpProps) {
  if (!showPopUp) {
    return null;
  }

  return createPortal(
    <div className="popup-overlay">
      <div className="popup-content">{children}</div>
    </div>,
    document.body,
  );
}
