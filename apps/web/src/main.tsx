import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource/dm-sans/400.css";
import "@fontsource/dm-sans/500.css";
import "@fontsource/dm-sans/600.css";
import "./styles/global.css";
import App from "./App.tsx";

const root = document.getElementById("root");
if (!root) throw new Error("Missing homepage root element");

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
