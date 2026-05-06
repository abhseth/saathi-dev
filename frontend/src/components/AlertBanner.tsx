import React from "react";
import type { Alert } from "../types";

type AlertBannerProps = {
  alerts: Alert[];
  onDismiss?: (id: string) => void;
};

export function AlertBanner({ alerts, onDismiss }: AlertBannerProps) {
  if (alerts.length === 0) return null;

  return (
    <div className="alert-banner-stack" role="region" aria-label="Alerts">
      {alerts.map((alert, index) => (
        <div
          key={`banner-${index}-${alert.id || "no-id"}-${alert.category || "no-cat"}-${alert.message?.slice(0, 20) || "no-msg"}`}
          className={`alert-banner alert-banner-${alert.severity}`}
          role="alert"
        >
          <div className="alert-banner-content">
            <span className="alert-banner-badge">{alert.category}</span>
            <span className="alert-banner-message">{alert.message}</span>
          </div>
          {onDismiss && (
            <button
              type="button"
              className="alert-banner-dismiss"
              onClick={() => onDismiss(alert.id)}
              aria-label={`Dismiss ${alert.category} alert`}
            >
              ✕
            </button>
          )}
        </div>
      ))}
    </div>
  );
}
