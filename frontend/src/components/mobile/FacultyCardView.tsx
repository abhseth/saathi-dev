import React from "react";
import type { ClassCard } from "../../types";

type FacultyCardViewProps = {
  cards: ClassCard[];
  onCardClick?: (card: ClassCard) => void;
};

export function FacultyCardView({ cards, onCardClick }: FacultyCardViewProps) {
  if (cards.length === 0) {
    return <p className="empty-state compact">No classes scheduled.</p>;
  }

  return (
    <div className="faculty-card-view">
      {cards.map((card, idx) => (
        <button
          type="button"
          key={`${card.period}-${idx}`}
          className={`class-card ${card.is_substitution ? "substitution" : ""}`}
          onClick={() => onCardClick?.(card)}
        >
          <div className="class-card-header">
            <span className="class-period">Period {card.period}</span>
            <span className="class-time">
              {card.start_time || "--:--"} – {card.end_time || "--:--"}
            </span>
          </div>
          <div className="class-card-body">
            <strong className="class-subject">{card.subject_name}</strong>
            <div className="class-meta">
              <span className="class-room">🏫 {card.room || "TBD"}</span>
              <span className="class-section">
                {card.grade_level} {card.track}
              </span>
            </div>
            <div className="class-school">{card.school_name}</div>
            {card.is_substitution && card.original_faculty_name && (
              <div className="class-sub-info">Sub for {card.original_faculty_name}</div>
            )}
          </div>
          {card.room && (
            <div className="room-map-badge" title="Room location">
              <span>🗺️</span>
              <small>{card.room}</small>
            </div>
          )}
        </button>
      ))}
    </div>
  );
}
