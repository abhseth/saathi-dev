import React from "react";
import type { School, TimetableHealthStatus } from "../../types";

type AomMorningBriefProps = {
  schools: School[];
  healthData: TimetableHealthStatus[];
  onAssignSubstitute?: (schoolId: number) => void;
  onViewSchool?: (schoolId: number) => void;
};

export function AomMorningBrief({ schools, healthData, onAssignSubstitute, onViewSchool }: AomMorningBriefProps) {
  const cards = React.useMemo(() => {
    return schools
      .filter((s) => !s.is_dropped)
      .map((school) => {
        const health = healthData.find((h) => h.school_id === school.id);
        const unfilled = health?.gaps_count ?? 0;
        const status: "green" | "amber" | "red" =
          health?.status === "Red" ? "red" : health?.status === "Amber" ? "amber" : "green";
        return {
          school_id: school.id,
          school_name: school.name,
          unfilled_periods: unfilled,
          status,
        };
      })
      .sort((a, b) => {
        const order = { red: 0, amber: 1, green: 2 };
        return order[a.status] - order[b.status];
      });
  }, [schools, healthData]);

  return (
    <div className="mobile-digest">
      <div className="digest-header">
        <h2>Morning Brief</h2>
        <span className="text-muted">{cards.length} schools</span>
      </div>
      <div className="aom-card-list">
        {cards.map((card) => (
          <div key={card.school_id} className={`aom-card ${card.status}`}>
            <div className="aom-card-header">
              <div className="aom-card-title">
                <span className={`status-dot ${card.status}`} />
                <strong>{card.school_name}</strong>
              </div>
              <span className="aom-card-status">
                {card.status === "green"
                  ? "Healthy"
                  : card.status === "amber"
                  ? "Needs Attention"
                  : "Critical"}
              </span>
            </div>
            <div className="aom-card-body">
              <div className="aom-metrics">
                <div className="aom-metric">
                  <span>Unfilled periods</span>
                  <strong>{card.unfilled_periods}</strong>
                </div>
              </div>
              <div className="aom-card-actions">
                {onAssignSubstitute && card.unfilled_periods > 0 && (
                  <button
                    type="button"
                    className="primary-action small"
                    onClick={() => onAssignSubstitute(card.school_id)}
                  >
                    Assign Substitute
                  </button>
                )}
                {onViewSchool && (
                  <button
                    type="button"
                    className="ghost-button small"
                    onClick={() => onViewSchool(card.school_id)}
                  >
                    View School
                  </button>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
