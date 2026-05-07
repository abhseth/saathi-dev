import React from "react";
import { api } from "../api";
import type {
  AppUser,
  AssignmentRule,
  AuditLogEntry,
  CommunicationTemplate,
  CreateUserDraft,
  CurrentUser,
  EscalationPolicy,
  Paginated,
  SlaPolicy,
  UpdateUserDraft,
} from "../types";

export interface UseAdminStateOptions {
  currentUser: CurrentUser | null;
  onError: (msg: string) => void;
  onNotice: (msg: string) => void;
}

export interface UseAdminStateReturn {
  // State
  users: AppUser[];
  showUsers: boolean;
  assignmentRules: AssignmentRule[];
  communicationTemplates: CommunicationTemplate[];
  escalationPolicy: EscalationPolicy | null;
  slaPolicies: SlaPolicy[];
  auditLog: AuditLogEntry[];
  // Setters
  setShowUsers: React.Dispatch<React.SetStateAction<boolean>>;

  // Loaders
  loadUsers: () => Promise<void>;
  loadAssignmentRules: () => Promise<void>;
  loadCommunicationTemplates: () => Promise<void>;
  loadEscalationPolicy: () => Promise<void>;
  loadSlaPolicies: () => Promise<void>;
  loadAuditLog: () => Promise<void>;

  // Mutations
  handleCreateUser: (draft: CreateUserDraft) => Promise<void>;
  handleUpdateUser: (draft: UpdateUserDraft) => Promise<void>;
  handleDeleteUser: (id: number) => Promise<void>;
  handleChangePassword: (currentPassword: string, newPassword: string) => Promise<void>;
  handleResetPassword: (id: number, newPassword: string) => Promise<void>;
  saveSlaPolicy: (issueCategory: string, hours: number) => Promise<void>;
  saveAssignmentRule: (queue: string, assignee: string, isActive: boolean) => Promise<void>;
  saveCommunicationTemplate: (input: {
    id?: number;
    name: string;
    audience: string;
    body: string;
    is_active: boolean;
  }) => Promise<void>;
  saveEscalationPolicy: (input: {
    at_risk_hours: number;
    escalation_assignee: string;
    auto_assign_on_breach: boolean;
  }) => Promise<void>;

}

export function useAdminState(options: UseAdminStateOptions): UseAdminStateReturn {
  const { onError, onNotice } = options;

  const [users, setUsers] = React.useState<AppUser[]>([]);
  const [showUsers, setShowUsers] = React.useState(false);
  const [assignmentRules, setAssignmentRules] = React.useState<AssignmentRule[]>([]);
  const [communicationTemplates, setCommunicationTemplates] = React.useState<CommunicationTemplate[]>([]);
  const [escalationPolicy, setEscalationPolicy] = React.useState<EscalationPolicy | null>(null);
  const [slaPolicies, setSlaPolicies] = React.useState<SlaPolicy[]>([]);
  const [auditLog, setAuditLog] = React.useState<AuditLogEntry[]>([]);
  const loadUsers = React.useCallback(async () => {
    try {
      setUsers(await api<AppUser[]>("list_users"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadAssignmentRules = React.useCallback(async () => {
    try {
      setAssignmentRules(await api<AssignmentRule[]>("list_assignment_rules"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadCommunicationTemplates = React.useCallback(async () => {
    try {
      setCommunicationTemplates(await api<CommunicationTemplate[]>("list_communication_templates"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadEscalationPolicy = React.useCallback(async () => {
    try {
      setEscalationPolicy(await api<EscalationPolicy>("get_escalation_policy"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadSlaPolicies = React.useCallback(async () => {
    try {
      setSlaPolicies(await api<SlaPolicy[]>("list_sla_policies"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadAuditLog = React.useCallback(async () => {
    try {
      const result = await api<Paginated<AuditLogEntry>>("list_audit_log");
      const items = result.items;
      setAuditLog(items);
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const handleCreateUser = React.useCallback(async (draft: CreateUserDraft) => {
    try {
      await api("create_user", { input: draft });
      await loadUsers();
      onNotice("User created.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadUsers, onError, onNotice]);

  const handleUpdateUser = React.useCallback(async (draft: UpdateUserDraft) => {
    try {
      await api("update_user", { input: draft });
      await loadUsers();
      onNotice("User updated.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadUsers, onError, onNotice]);

  const handleDeleteUser = React.useCallback(async (id: number) => {
    try {
      await api("delete_user", { id });
      await loadUsers();
      onNotice("User deleted.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadUsers, onError, onNotice]);

  const handleChangePassword = React.useCallback(async (currentPassword: string, newPassword: string) => {
    try {
      await api("change_password", { input: { current_password: currentPassword, new_password: newPassword } });
      onNotice("Password changed successfully.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const handleResetPassword = React.useCallback(async (id: number, newPassword: string) => {
    try {
      await api("reset_password", { id, input: { new_password: newPassword } });
      onNotice("Password reset successfully.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const saveSlaPolicy = React.useCallback(async (issueCategory: string, hours: number) => {
    try {
      const policy = await api<SlaPolicy>("update_sla_policy", { input: { issue_category: issueCategory, hours } });
      setSlaPolicies((current) => current.map((item) => (item.issue_category === policy.issue_category ? policy : item)));
      onNotice(`SLA policy updated for ${policy.issue_category}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const saveAssignmentRule = React.useCallback(async (queue: string, assignee: string, isActive: boolean) => {
    try {
      const rule = await api<AssignmentRule>("update_assignment_rule", { input: { queue, assignee, is_active: isActive } });
      setAssignmentRules((current) => current.map((item) => (item.queue === rule.queue ? rule : item)));
      onNotice(`Routing updated for ${rule.queue}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const saveCommunicationTemplate = React.useCallback(async (input: {
    id?: number;
    name: string;
    audience: string;
    body: string;
    is_active: boolean;
  }) => {
    try {
      await api<CommunicationTemplate>("update_communication_template", { input });
      await loadCommunicationTemplates();
      onNotice(`Template saved: ${input.name}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadCommunicationTemplates, onError, onNotice]);

  const saveEscalationPolicy = React.useCallback(async (input: {
    at_risk_hours: number;
    escalation_assignee: string;
    auto_assign_on_breach: boolean;
  }) => {
    try {
      const policy = await api<EscalationPolicy>("update_escalation_policy", { input });
      setEscalationPolicy(policy);
      onNotice("Escalation policy updated.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  return {
    users,
    showUsers,
    assignmentRules,
    communicationTemplates,
    escalationPolicy,
    slaPolicies,
    auditLog,
    setShowUsers,
    loadUsers,
    loadAssignmentRules,
    loadCommunicationTemplates,
    loadEscalationPolicy,
    loadSlaPolicies,
    loadAuditLog,
    handleCreateUser,
    handleUpdateUser,
    handleDeleteUser,
    handleChangePassword,
    handleResetPassword,
    saveSlaPolicy,
    saveAssignmentRule,
    saveCommunicationTemplate,
    saveEscalationPolicy,
  };
}
