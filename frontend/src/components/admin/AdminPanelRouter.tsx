import React from "react";
import {
  AssignmentRulePanel,
  AuditLogPanel,
  CommunicationOperationsPanel,
  CommunicationTemplatePanel,
  DirectoryPanel,
  DroppedSchoolsPanel,
  EscalationPolicyPanel,
  ErrorBoundary,
  FacultyAssignmentsPanel,
  FacultyMembersPanel,
  MasterDataPanel,
  ProgramDashboardPanel,
  RegionHistoryPanel,
  ReportsPanel,
  SlaPolicyPanel,
  SubjectsPanel,
  TimetablePanel,
  HolidaysPanel,
} from "../../components";
import { BatchesPanel } from "../batches/BatchesPanel";
import { SubstitutionCommandCenter } from "../substitution/SubstitutionCommandCenter";
import { LeaveSwapPanel } from "../substitution/LeaveSwapPanel";
import { AlertInboxPanel } from "../automation/AlertInboxPanel";
import {
  ControlTowerPanelWrapper,
  ComplianceScorecardPanelWrapper,
  DeviationScoreboardPanelWrapper,
} from "../dashboards";
import type {
  UseAdminStateReturn,
  UseFacultyStateReturn,
  UseMasterDataStateReturn,
  UseTicketStateReturn,
} from "../../hooks";
import type { AdminView } from "../../toolRegistry";
import type { CurrentUser } from "../../types";

type Props = {
  adminView: AdminView | null;
  admin: UseAdminStateReturn;
  master: UseMasterDataStateReturn;
  ticket: UseTicketStateReturn;
  faculty: UseFacultyStateReturn;
  onClose: () => void;
  onSaveEscalationPolicy: (input: {
    at_risk_hours: number;
    escalation_assignee: string;
    auto_assign_on_breach: boolean;
  }) => Promise<void>;
  onRefreshSla?: () => void;
  currentUserRole: string;
  currentUser: CurrentUser;
};

export const AdminPanelRouter = React.memo(function AdminPanelRouter({
  adminView,
  admin,
  master,
  ticket,
  faculty,
  onClose,
  onSaveEscalationPolicy,
  onRefreshSla,
  currentUserRole,
  currentUser,
}: Props) {
  if (!adminView) return null;

  const dashboardSummary = master.programDashboard
    ? {
        total_schools: master.programDashboard.total_schools,
        total_batches: master.programDashboard.total_classes,
        total_students: master.programDashboard.total_actual_admissions,
        active_tickets: ticket.tickets.filter(
          (t) => t.status !== "Closed" && t.status !== "Resolved",
        ).length,
      }
    : null;

  const reportData = (() => {
    const tickets_by_status: Record<string, number> = {};
    const tickets_by_school: Record<string, number> = {};
    for (const t of ticket.tickets) {
      tickets_by_status[t.status] = (tickets_by_status[t.status] ?? 0) + 1;
      const schoolName = t.school_name ?? "Unknown";
      tickets_by_school[schoolName] = (tickets_by_school[schoolName] ?? 0) + 1;
    }
    return { tickets_by_status, tickets_by_school };
  })();

  const assignmentRulesForPanel = admin.assignmentRules.map((r, i) => ({
    id: i + 1,
    queue: String(r.queue),
    assignee: r.assignee,
    condition: r.is_active ? "Active" : "Inactive",
  }));

  const regionHistoryForPanel = master.schoolRegionHistory.map((h) => ({
    school_id: h.school_id,
    old_region_id: h.old_region_id,
    new_region_id: h.new_region_id,
    changed_at: h.changed_at,
    changed_by: "System",
  }));

  const auditEntriesForPanel = admin.auditLog.map((entry) => ({
    id: entry.id,
    entity_type: entry.entity_type,
    entity_id: entry.entity_id,
    action: entry.action,
    actor: entry.actor,
    created_at: entry.created_at,
    details: entry.summary,
  }));

  const createTemplate = () => {
    void admin.saveCommunicationTemplate({
      name: "New Template",
      audience: "School",
      body: "Draft template body",
      is_active: false,
    });
  };

  const toggleTemplate = (id: number) => {
    const template = admin.communicationTemplates.find((t) => t.id === id);
    if (template) {
      void admin.saveCommunicationTemplate({
        ...template,
        is_active: !template.is_active,
      });
    }
  };

  switch (adminView) {
    case "sla":
      return (
        <SlaPolicyPanel
          policies={admin.slaPolicies}
          onSave={(policies) =>
            policies.forEach((p) =>
              void admin.saveSlaPolicy(p.issue_category, p.hours),
            )
          }
          onRefreshSla={onRefreshSla}
        />
      );
    case "routing":
      return (
        <AssignmentRulePanel
          rules={assignmentRulesForPanel}
          onSave={(rules) =>
            rules.forEach((r) =>
              void admin.saveAssignmentRule(
                r.queue,
                r.assignee,
                r.condition !== "Inactive",
              ),
            )
          }
        />
      );
    case "escalation":
      return admin.escalationPolicy ? (
        <EscalationPolicyPanel
          policy={admin.escalationPolicy}
          onSave={(policy) =>
            void onSaveEscalationPolicy({
              at_risk_hours: policy.at_risk_hours,
              escalation_assignee: policy.escalation_assignee,
              auto_assign_on_breach: policy.auto_assign_on_breach,
            })
          }
        />
      ) : null;
    case "master-data":
      return (
        <MasterDataPanel
          classPlans={master.classPlans}
          lectureModels={master.lectureModels}
          regions={master.regions}
          schools={master.schools}
          batches={master.batches}
          students={master.students}
          studentTotalCount={master.studentTotalCount}
          studentPage={master.studentPage}
          studentPageSize={master.studentPageSize}
          studentSearch={master.studentSearch}
          sipImportPreview={master.sipImportReview}
          onCancelSipMasterImport={master.cancelSipMasterImport}
          onConfirmSipMasterImport={() =>
            master.confirmSipMasterImport("skip_existing")
          }
          onClose={onClose}
          onCreateSchool={master.createSchool}
          onUpdateSchool={master.updateSchool}
          onSaveRegion={master.saveRegion}
          onCreateLectureModel={master.createLectureModel}
          onSaveClassPlan={master.saveSchoolClassPlan}
          onCreateBatch={master.createBatch}
          onUpdateBatch={master.updateBatch}
          onArchiveBatch={master.archiveBatch}
          onImportSchools={master.importSchoolsCsv}
          onImportSipMaster={master.importSipMaster}
          onExportSipMaster={master.exportSipMasterExcel}
          onDeleteSchool={master.deleteSchool}
          onLoadSchoolDeleteImpact={master.loadSchoolDeleteImpact}
          onDropSchool={master.dropSchool}
          onDeleteRegion={master.deleteRegion}
          onRemapAndDeleteRegion={(oldId, newId) => {
            const mappings = master.schools
              .filter((s) => s.region_id === oldId)
              .map((s) => ({ school_id: s.id, target_region_id: newId }));
            void master.remapAndDeleteRegion(oldId, mappings);
          }}
          onCreateStudent={master.createStudent}
          onUpdateStudent={master.updateStudent}
          onDeleteStudent={master.deleteStudent}
          onStudentSearchChange={master.setStudentSearch}
          onLoadStudents={master.loadStudents}
          onImportStudentsCsv={master.importStudentsCsv}
          currentUserRole={currentUserRole}
        />
      );
    case "program-dashboard":
      return <ProgramDashboardPanel dashboard={dashboardSummary} />;
    case "reports":
      return (
        <ReportsPanel
          data={reportData}
          dasRows={faculty.dasReport}
          schools={master.schools}
          currentUserRole={currentUserRole}
          onLoadDasReport={(startDate, endDate, groupBy, schoolId) => {
            void faculty.loadDasReport(startDate, endDate, groupBy, schoolId);
          }}
        />
      );
    case "communications":
      return (
        <CommunicationOperationsPanel
          templates={admin.communicationTemplates}
          onAddTemplate={createTemplate}
          onToggleTemplate={toggleTemplate}
          currentUserRole={currentUserRole}
        />
      );
    case "directory":
      return (
        <ErrorBoundary fallback={<p className="empty-state">Something went wrong loading the Directory. Try refreshing the page.</p>}>
          <DirectoryPanel
            schools={master.schools}
            regions={master.regions}
            facultyMembers={faculty.facultyMembers}
            facultyMemberships={faculty.facultyMemberMemberships}
            users={admin.users}
            onLoadFacultyMemberships={faculty.loadFacultySchoolMemberships}
            onExport={master.exportSipMasterExcel}
          />
        </ErrorBoundary>
      );
    case "dropped-schools":
      return (
        <DroppedSchoolsPanel
          schools={master.droppedSchools}
          currentUserRole={currentUserRole}
          onRestoreSchool={master.restoreSchool}
          onDeleteSchool={master.deleteSchool}
          onLoadSchoolDeleteImpact={master.loadSchoolDeleteImpact}
        />
      );
    case "region-log":
      return (
        <RegionHistoryPanel
          history={regionHistoryForPanel}
          schools={master.schools}
        />
      );
    case "audit-log":
      return <AuditLogPanel entries={auditEntriesForPanel} />;
    case "templates":
      return (
        <CommunicationTemplatePanel
          templates={admin.communicationTemplates}
          onAddTemplate={createTemplate}
        />
      );
    case "faculty-assignments":
      return (
        <FacultyAssignmentsPanel
          assignments={faculty.facultyAssignments}
          schools={master.schools}
          batches={master.batches}
          facultyMembers={faculty.facultyMembers}
          subjects={faculty.subjects}
          onCreateAssignment={faculty.createFacultyAssignment}
          onDeleteAssignment={faculty.deleteFacultyAssignment}
        />
      );
    case "timetable":
      return (
        <TimetablePanel
          slots={faculty.timetableSlots}
          schools={master.schools}
          batches={master.batches}
          subjects={faculty.subjects}
          facultyMembers={faculty.facultyMembers}
          holidays={faculty.holidays}
          onSaveSlot={faculty.upsertTimetableSlot}
        />
      );
    case "subjects":
      return (
        <SubjectsPanel
          subjects={faculty.subjects}
          onCreateSubject={faculty.createSubject}
          onUpdateSubject={faculty.updateSubject}
          onDeleteSubject={faculty.deleteSubject}
        />
      );
    case "faculty-members":
      return (
        <FacultyMembersPanel
          members={faculty.facultyMembers}
          schools={master.schools}
          users={admin.users}
          memberships={faculty.facultyMemberMemberships}
          onCreateMember={faculty.createFacultyMember}
          onUpdateMember={faculty.updateFacultyMember}
          onDeleteMember={faculty.deleteFacultyMember}
          onCreateMembership={faculty.createFacultySchoolMembership}
          onDeleteMembership={faculty.deleteFacultySchoolMembership}
          onLoadMemberships={faculty.loadFacultySchoolMemberships}
          onImportCsv={faculty.importFacultyMembersCsv}
          onCreateLogin={faculty.createFacultyLogin}
          onLinkUser={faculty.linkFacultyUser}
        />
      );
    case "substitutions":
      return <SubstitutionCommandCenter schools={master.schools} />;
    case "leave-swap":
      return (
        <LeaveSwapPanel
          schools={master.schools}
          faculty={faculty.facultyMembers.map((f) => ({ id: f.id, display_name: f.name }))}
          currentUser={currentUser}
        />
      );
    case "alert-inbox":
      return <AlertInboxPanel onClose={onClose} />;
    case "control-tower":
      return <ControlTowerPanelWrapper onClose={onClose} />;
    case "compliance-scorecard":
      return <ComplianceScorecardPanelWrapper onClose={onClose} />;
    case "deviation-scoreboard":
      return <DeviationScoreboardPanelWrapper onClose={onClose} />;
    case "holidays":
      return (
        <HolidaysPanel
          holidays={faculty.holidays}
          schools={master.schools}
          regions={master.regions}
          onBulkCreateHoliday={faculty.bulkCreateHoliday}
          onDeleteHoliday={faculty.deleteHoliday}
        />
      );
    case "batches":
      return <BatchesPanel />;
    default:
      return null;
  }
});
