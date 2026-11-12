import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource/dm-sans/400.css";
import "@fontsource/dm-sans/500.css";
import "@fontsource/dm-sans/600.css";
import AnalyticsConsent from "./AnalyticsConsent.tsx";
import "./styles/global.css";
import "./App.css";
import "./LegalPage.css";

const root = document.getElementById("consent-root");
if (!root) throw new Error("Missing privacy controls root element");

createRoot(root).render(
  <StrictMode>
    <AnalyticsConsent />
  </StrictMode>,
);
