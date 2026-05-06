/**
 * Browser notification utilities for SAATHI.
 * Push notifications require service workers (complex), so this is a lightweight
 * stub using the Notifications API where available, plus in-app scheduling.
 */

export function requestNotificationPermission(): Promise<NotificationPermission> {
  if (typeof Notification === "undefined") {
    return Promise.resolve("denied" as NotificationPermission);
  }
  if (Notification.permission === "granted") {
    return Promise.resolve("granted");
  }
  return Notification.requestPermission();
}

export function sendBrowserNotification(title: string, body: string): void {
  if (typeof Notification === "undefined") return;
  if (Notification.permission === "granted") {
    try {
      new Notification(title, { body, icon: "/favicon.ico" });
    } catch (e) {
      console.error("Failed to send browser notification:", e);
    }
  }
}

export function scheduleReminder(delayMs: number, message: string): number {
  return window.setTimeout(() => {
    sendBrowserNotification("SAATHI Reminder", message);
  }, delayMs);
}

export function cancelReminder(timerId: number): void {
  window.clearTimeout(timerId);
}
