import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.scss";
import "font-gis/css/font-gis.css";
import App from "./App.tsx";

document.documentElement.setAttribute(
  "data-bs-theme",
  window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light",
);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
