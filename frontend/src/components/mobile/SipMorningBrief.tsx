import React from "react";
import type { FacultyTodaySession, TimetableHealthStatus } from "../../types";

type SipMorningBriefProps = {
  sessions: FacultyTodaySession[];
  healthData: TimetableHealthStatus[];
};

export function SipMorningBrief({ sessions, healthData }: SipMorningBriefProps) {
  const cancelledCount = sessions.filter((s) => s.status === "Cancelled").length;
  const substitutedCount = sessions.filter((s) => s.status === "Substituted").length;
  const incompleteAttendance = sessions.filter(
    (s) => s.status !== "Cancelled" && s.total_students > 0 && s.present_count + s.late_count < s.total_students
  ).length;
  const allRedSchools = healthData.filter((h) => h.status === "Red");
  const redSchoolNames = allRedSchools.slice(0, 3).map((h) => h.school_name);
  const totalRedSchools = allRedSchools.length;

  const cards = [
    {
      title: "Cancelled / Substituted Sessions",
      count: cancelledCount + substitutedCount,
      items: [
        `${cancelledCount} cancelled`,
        `${substitutedCount} substituted`,
      ],
    },
    {
      title: "Incomplete Attendance",
      count: incompleteAttendance,
      items: incompleteAttendance > 0
        ? ["Some sessions have incomplete records"]
        : ["All sessions marked"],
    },
    {
      title: "Top Red Schools",
      count: totalRedSchools,
      items: totalRedSchools > 0
        ? [`${redSchoolNames.length} of ${totalRedSchools} shown`, ...redSchoolNames]
        : ["No red schools today"],
    },
  ];

  return (
    <div className="mobile-digest">
      <div className="digest-header">
        <h2>SIP Morning Brief</h2>
      </div>
      <div className="sip-card-list">
        {cards.map((card) => (
          <div key={card.title} className="sip-card">
            <div className="sip-card-top">
              <strong>{card.title}</strong>
              <span className="sip-count">{card.count}</span>
            </div>
            <ul className="sip-items">
              {card.items.map((item, idx) => (
                <li key={idx}>{item}</li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      <p className="provisional-note">Attendance counts rely on data entered by faculty and may not reflect real-time participation.</p>
    </div>
  );
}
