import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root")!).render(<App />);

// Register service worker for PWA shell caching
if ("serviceWorker" in navigator) {
  window.addEventListener("load", () => {
    navigator.serviceWorker
      .register("/sw.js")
      .then((reg) => console.log("[SW] registered:", reg.scope))
      .catch((err) => console.error("[SW] registration failed:", err));
  });
}
