import React from "react";
import { createPortal } from "react-dom";
import "./Popup.css";

interface PopUpProps {
  showPopUp: boolean;
  closePopUp: () => void;
  children: React.ReactNode;
}

export default function Popup({ showPopUp, closePopUp, children }: PopUpProps) {
  if (!showPopUp) {
    return null;
  }

  return createPortal(
    <div className="popup-overlay">
      <div className="popup-content">
        <button className="close-btn" onClick={closePopUp} aria-label="Close">
          &times;
        </button>
        {children}
      </div>
    </div>,
    document.body,
  );
}
