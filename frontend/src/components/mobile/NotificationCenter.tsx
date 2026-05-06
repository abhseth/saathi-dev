import React from "react";
import type { Notification } from "../../types";

type NotificationCenterProps = {
  notifications: Notification[];
  unreadCount: number;
  onMarkRead: (id: number) => void;
  onMarkAllRead: () => void;
  onClose: () => void;
};

export function NotificationCenter({
  notifications,
  unreadCount,
  onMarkRead,
  onMarkAllRead,
  onClose,
}: NotificationCenterProps) {
  return (
    <div className="notification-center">
      <div className="notification-center-header">
        <h3>Notifications {unreadCount > 0 && <span className="badge">{unreadCount}</span>}</h3>
        <div className="notification-actions">
          {unreadCount > 0 && (
            <button type="button" className="text-btn" onClick={onMarkAllRead}>
              Mark all read
            </button>
          )}
          <button type="button" className="text-btn" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
      <div className="notification-list">
        {notifications.length === 0 ? (
          <p className="empty-state compact">No notifications yet.</p>
        ) : (
          notifications.map((n) => (
            <button
              type="button"
              key={n.id}
              className={`notification-item ${n.read_at ? "read" : "unread"}`}
              onClick={() => !n.read_at && onMarkRead(n.id)}
            >
              <div className="notification-top">
                <strong className="notification-title">{n.title}</strong>
                <span className="notification-time">{new Date(n.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</span>
              </div>
              <p className="notification-message">{n.message}</p>
            </button>
          ))
        )}
      </div>
    </div>
  );
}
