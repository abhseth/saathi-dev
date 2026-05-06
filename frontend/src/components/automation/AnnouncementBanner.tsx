import React from "react";
import { api } from "../../api";
import type { Announcement } from "../../types";

export function AnnouncementBanner() {
  const [announcements, setAnnouncements] = React.useState<Announcement[]>([]);

  const load = React.useCallback(async () => {
    try {
      const data = await api<Announcement[]>("list_announcements");
      setAnnouncements(data);
    } catch (e) {
      console.error("Failed to load announcements:", e);
    }
  }, []);

  React.useEffect(() => {
    void load();
    const id = setInterval(load, 60000);
    return () => clearInterval(id);
  }, [load]);

  if (announcements.length === 0) return null;

  return (
    <div className="announcement-banner" style={{ background: "#fff3cd", border: "1px solid #ffc107", padding: "8px 12px", borderRadius: 6, marginBottom: 8, gridArea: "announcement" }}>
      {announcements.map((a) => (
        <div key={a.id} style={{ fontSize: 14 }}>
          <strong>📌 {a.school_name ? a.school_name + ": " : ""}</strong>
          {a.message}
        </div>
      ))}
    </div>
  );
}
