export function formatTimestamp(value: string) {
  if (!value) {
    return "";
  }

  return value.replace("T", " ");
}

export function formatField(value: string) {
  return value.replace(/_/g, " ").replace(/\b\w/g, (letter) => letter.toUpperCase());
}

export function formatBytes(value: number) {
  if (value < 1024) {
    return `${value} B`;
  }

  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }

  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

export function formatSlaDue(value: string) {
  return formatTimestamp(value);
}

export function formatSlaCountdown(sla_due_at: string): string {
  if (!sla_due_at) return "";
  const due = Date.parse(sla_due_at.replace(" ", "T"));
  if (Number.isNaN(due)) return "";
  const diffMs = due - Date.now();
  const abs = Math.abs(diffMs);
  const hours = Math.floor(abs / 3_600_000);
  const mins = Math.floor((abs % 3_600_000) / 60_000);
  const label = hours > 0 ? `${hours}h ${mins}m` : `${mins}m`;
  return diffMs >= 0 ? `${label} remaining` : `overdue by ${label}`;
}

export function getSlaState(ticket: { status: string; sla_due_at: string }) {
  if (["Resolved", "Closed"].includes(ticket.status)) {
    return "Met";
  }

  const dueTime = Date.parse(ticket.sla_due_at.replace(" ", "T"));
  if (Number.isNaN(dueTime)) {
    return "Unknown";
  }

  const hoursRemaining = (dueTime - Date.now()) / (1000 * 60 * 60);
  if (hoursRemaining < 0) {
    return "Breached";
  }

  if (hoursRemaining <= 24) {
    return "At Risk";
  }

  return "On Track";
}
