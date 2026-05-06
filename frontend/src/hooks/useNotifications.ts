import React from "react";
import { api } from "../api";
import type { Notification } from "../types";

const POLL_INTERVAL = 30_000;

export function useNotifications() {
  const [notifications, setNotifications] = React.useState<Notification[]>([]);
  const [unreadCount, setUnreadCount] = React.useState(0);
  const [loading, setLoading] = React.useState(false);

  const loadNotifications = React.useCallback(async () => {
    setLoading(true);
    try {
      const items = await api<Notification[]>("list_notifications");
      setNotifications(items);
    } catch (e) {
      console.error("Failed to load notifications:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadUnreadCount = React.useCallback(async () => {
    try {
      const result = await api<{ count: number }>("unread_notification_count");
      setUnreadCount(result.count);
    } catch (e) {
      console.error("Failed to load unread notification count:", e);
    }
  }, []);

  const markRead = React.useCallback(async (id: number) => {
    try {
      await api("mark_notification_read", { id });
      setNotifications((prev) =>
        prev.map((n) => (n.id === id ? { ...n, read_at: new Date().toISOString() } : n))
      );
      setUnreadCount((c) => Math.max(0, c - 1));
    } catch (e) {
      console.error("Failed to mark notification read:", e);
    }
  }, []);

  const markAllRead = React.useCallback(async () => {
    try {
      await api("mark_all_notifications_read");
      setNotifications((prev) =>
        prev.map((n) => ({ ...n, read_at: new Date().toISOString() }))
      );
      setUnreadCount(0);
    } catch (e) {
      console.error("Failed to mark all notifications read:", e);
    }
  }, []);

  React.useEffect(() => {
    void loadNotifications();
    void loadUnreadCount();
    const interval = setInterval(() => {
      void loadNotifications();
      void loadUnreadCount();
    }, POLL_INTERVAL);
    return () => clearInterval(interval);
  }, [loadNotifications, loadUnreadCount]);

  return {
    notifications,
    unreadCount,
    loading,
    loadNotifications,
    markRead,
    markAllRead,
  };
}
