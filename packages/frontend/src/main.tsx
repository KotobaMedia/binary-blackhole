import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.scss";
import "font-gis/css/font-gis.css";
import App from "./App.tsx";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
