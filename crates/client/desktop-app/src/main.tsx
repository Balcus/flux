import React from "react";
import ReactDOM from "react-dom/client";
import AppRoutes from "./config/AppRoutes";
import { BrowserRouter } from "react-router-dom";
import { RepositoryProvider } from "./context/RepositoryContext";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <RepositoryProvider>
        <AppRoutes />
      </RepositoryProvider>
    </BrowserRouter>
  </React.StrictMode>
);
