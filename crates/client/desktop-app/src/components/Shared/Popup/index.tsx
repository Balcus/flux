import React from 'react';

import '../../../App.css';
import './Popup.css';

interface PopUpProps {
  showPopUp: boolean;
  closePopUp: () => void;
  children: React.ReactNode;
}

export default function Popup({ showPopUp, closePopUp, children }: PopUpProps) {
    if (!showPopUp) return null;
    
    return (
        <div className="popup-overlay">
            <div className="popup-content">
                <button className="close-btn" onClick={closePopUp} aria-label="Close">
                    &times;
                </button>
                {children}
            </div>
        </div>
    );
}