import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root")!).render(<App />);

// Service worker disabled to prevent stale cache issues during deployment transitions
if ("serviceWorker" in navigator) {
  navigator.serviceWorker.getRegistrations().then((regs) => {
    if (regs.length > 0) {
      for (const reg of regs) {
        reg.unregister();
        console.log("[SW] unregistered:", reg.scope);
      }
      // Reload once to ensure the page runs without any SW control
      window.location.reload();
    }
  });
}
