import type { Filter, Priority, Queue, Status } from "./types";

export const statuses: Status[] = ["Open", "In Progress", "Pending", "Resolved", "Closed"];
export const priorities: Priority[] = ["Low", "Medium", "High", "Critical"];
export const queues: Queue[] = [
  "Academic Support",
  "Learning Platform",
  "IT / Device",
  "Operations",
  "Parent Communication",
];
export const filters: Filter[] = [
  "Inbox",
  "My Tickets",
  "Unassigned",
  "Pending SLA",
  "Escalated",
  "Resolved",
];
export const gradeLevels = [
  "Grade 6",
  "Grade 7",
  "Grade 8",
  "Grade 9",
  "Grade 10",
  "Grade 11",
  "Grade 12",
  "Dropper",
];
// Grades 11/12/Dropper carry an academic-track split.
export const trackEligibleGrades = new Set(["Grade 11", "Grade 12", "Dropper"]);
export const academicTracks = ["JEE", "NEET"];
export const batchPatterns = ["Weekday", "Weekend", "Both"];
export const programTracks = [
  "Integrated STEM",
  "JEE Foundation",
  "NEET Foundation",
  "Olympiad",
  "Board Excellence",
];
export const issueCategories = [
  "Academic Support",
  "Attendance",
  "Assessment",
  "Device",
  "Learning Platform",
  "Operations",
  "Parent Communication",
];
