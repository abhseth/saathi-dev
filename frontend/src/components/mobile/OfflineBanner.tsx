import React from "react";

type OfflineBannerProps = {
  isOnline: boolean;
  needsSync?: boolean;
  onSync?: () => void;
};

export function OfflineBanner({ isOnline, needsSync, onSync }: OfflineBannerProps) {
  if (isOnline && !needsSync) return null;
  return (
    <div className="offline-banner" role="status" aria-label={isOnline ? "Sync pending" : "Offline"}>
      {!isOnline ? (
        <span>⚠️ You are offline. Some features may not work.</span>
      ) : (
        <span>
          🔄 Changes made while offline are pending.
          {onSync && (
            <button type="button" className="sync-btn" onClick={onSync}>
              Sync now
            </button>
          )}
        </span>
      )}
    </div>
  );
}
