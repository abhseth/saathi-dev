import React from "react";

type FacultyTab = "today" | "attendance" | "requests";

type FacultyBottomNavProps = {
  activeTab: FacultyTab;
  onChange: (tab: FacultyTab) => void;
  pendingSubstitutionCount?: number;
};

export function FacultyBottomNav({ activeTab, onChange, pendingSubstitutionCount }: FacultyBottomNavProps) {
  const tabs: { key: FacultyTab; label: string; icon: string }[] = [
    { key: "today", label: "Today", icon: "📅" },
    { key: "attendance", label: "Attendance", icon: "✓" },
    { key: "requests", label: "Requests", icon: "🔄" },
  ];

  return (
    <nav className="faculty-bottom-nav">
      {tabs.map((t) => (
        <button
          key={t.key}
          type="button"
          className={activeTab === t.key ? "active" : ""}
          onClick={() => onChange(t.key)}
          aria-label={t.label}
        >
          <span>{t.icon}</span>
          <small>{t.label}</small>
          {t.key === "requests" && (pendingSubstitutionCount ?? 0) > 0 && (
            <span className="bottom-nav-badge">
              {pendingSubstitutionCount! > 99 ? "99+" : pendingSubstitutionCount}
            </span>
          )}
        </button>
      ))}
    </nav>
  );
}
