import React from "react";
import { createRoot } from "react-dom/client";

import { App } from "./App";
import "./fonts.css";
import "./library.css";
import "./styles.css";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("SAYT could not find its application root element.");
}

createRoot(rootElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
