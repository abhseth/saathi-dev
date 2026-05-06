import React from "react";
import { SectionLanding } from "./SectionLanding";

type ReportsLandingProps = {
  currentUserRole: string;
  onOpenTool: (toolId: string) => void;
};

export function ReportsLanding({ currentUserRole, onOpenTool }: ReportsLandingProps) {
  return (
    <SectionLanding
      section="reports"
      currentUserRole={currentUserRole}
      onOpenTool={onOpenTool}
    >
      <p className="landing-intro">
        Explore analytics, compliance metrics, and operational dashboards across your schools.
      </p>
    </SectionLanding>
  );
}
