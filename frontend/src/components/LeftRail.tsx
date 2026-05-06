import React from "react";
import type { Filter } from "../types";
import type { AppSection } from "../navigation";
import { SECTION_ORDER, SECTION_LABELS, canAccessSection } from "../navigation";

const filters: Filter[] = [
  "Inbox",
  "My Tickets",
  "Unassigned",
  "Pending SLA",
  "Escalated",
  "Resolved",
];

const SECTION_ICONS: Record<AppSection, React.ReactNode> = {
  work: (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round">
      <path d="M22 12h-6l-2-3H10L8 12H2" />
      <path d="M5.45 5.11L2 12v6a2 2 0 002 2h16a2 2 0 002-2v-6l-3.45-6.89A2 2 0 0016.76 4H7.24a2 2 0 00-1.79 1.11z" />
    </svg>
  ),
  schools: (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 21h18" />
      <path d="M5 21V7l8-4 8 4v14" />
      <path d="M9 21v-6h6v6" />
      <path d="M10 9h4" />
      <path d="M10 13h4" />
    </svg>
  ),
  faculty: (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round">
      <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" />
      <circle cx="12" cy="7" r="4" />
    </svg>
  ),
  timetable: (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
      <line x1="16" y1="2" x2="16" y2="6" />
      <line x1="8" y1="2" x2="8" y2="6" />
      <line x1="3" y1="10" x2="21" y2="10" />
    </svg>
  ),
  reports: (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round">
      <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <line x1="16" y1="13" x2="8" y2="13" />
      <line x1="16" y1="17" x2="8" y2="17" />
      <polyline points="10 9 9 9 8 9" />
    </svg>
  ),
  admin: (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" />
    </svg>
  ),
};

type LeftRailProps = {
  currentSection: AppSection;
  currentUserRole: string;
  activeFilter: Filter;
  filterCounts: Record<Filter, number>;
  onSectionChange: (section: AppSection) => void;
  onFilterChange: (filter: Filter) => void;
};

export function LeftRail({
  currentSection,
  currentUserRole,
  activeFilter,
  filterCounts,
  onSectionChange,
  onFilterChange,
}: LeftRailProps) {
  const visibleSections = React.useMemo(
    () => SECTION_ORDER.filter((s) => canAccessSection(s, currentUserRole)),
    [currentUserRole],
  );

  return (
    <aside className="left-rail" aria-label="Main navigation">
      <div className="left-rail-brand">
        <span className="brand-mark">S</span>
        <div>
          <strong>SAATHI</strong>
          <small>School integrated program</small>
        </div>
      </div>

      <nav className="left-rail-sections" aria-label="Sections">
        {visibleSections.map((section) => (
          <button
            key={section}
            type="button"
            className={`left-rail-section ${currentSection === section ? "active" : ""}`}
            onClick={() => onSectionChange(section)}
            aria-current={currentSection === section ? "page" : undefined}
          >
            {SECTION_ICONS[section]}
            <span>{SECTION_LABELS[section]}</span>
          </button>
        ))}
      </nav>

      {currentSection === "work" && (
        <nav className="left-rail-filters" aria-label="Ticket views">
          <span className="left-rail-filters-heading">Views</span>
          {filters.map((filter) => (
            <button
              key={filter}
              type="button"
              className={`left-rail-filter ${activeFilter === filter ? "active" : ""}`}
              onClick={() => onFilterChange(filter)}
            >
              <span>{filter}</span>
              {filterCounts[filter] > 0 ? (
                <span
                  className={`filter-badge${
                    filter === "Pending SLA" || filter === "Escalated" ? " filter-badge-urgent" : ""
                  }`}
                >
                  {filterCounts[filter]}
                </span>
              ) : null}
            </button>
          ))}
        </nav>
      )}
    </aside>
  );
}
