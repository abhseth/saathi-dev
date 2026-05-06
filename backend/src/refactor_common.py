#!/usr/bin/env python3
"""Move functions out of common.rs to their proper domain modules."""

import os

COMMON = "/home/abhi/saathi-dev/backend/src/repo/common.rs"
TICKETS = "/home/abhi/saathi-dev/backend/src/repo/tickets.rs"
SCHOOLS = "/home/abhi/saathi-dev/backend/src/repo/schools.rs"
AUDIT = "/home/abhi/saathi-dev/backend/src/repo/audit.rs"
MOD_RS = "/home/abhi/saathi-dev/backend/src/repo/mod.rs"

with open(COMMON) as f:
    common_lines = f.readlines()

def get(lines, start, end):
    return "".join(lines[start-1:end])

def delete(lines, start, end):
    return lines[:start-1] + lines[end:]

# ── Extract sections from common.rs ──────────────────────────────────────────

# 1. audit.rs: record_audit (802-819) + insert_audit_log (910-922)
audit_content = get(common_lines, 802, 819) + "\n" + get(common_lines, 910, 922)
common_lines = delete(common_lines, 910, 922)
common_lines = delete(common_lines, 802, 819)

# 2. tickets.rs: record_history (778-801), current_local_timestamp (620-624),
#    audience_for_audit (612-619), channel_for_audit (604-611),
#    normalize_follow_up_due (587-603), validate_comment_status (572-586),
#    validate_queue (495-506), validate_status_transition (477-494),
#    validate_status (465-476), validate_priority (453-464),
#    ALLOWED_QUEUES (24-30), ALLOWED_PRIORITIES (22), ALLOWED_STATUSES (23)
tickets_content = (
    get(common_lines, 22, 23) + "\n" +
    get(common_lines, 24, 30) + "\n" +
    get(common_lines, 453, 464) + "\n" +
    get(common_lines, 465, 476) + "\n" +
    get(common_lines, 477, 494) + "\n" +
    get(common_lines, 495, 506) + "\n" +
    get(common_lines, 572, 586) + "\n" +
    get(common_lines, 587, 603) + "\n" +
    get(common_lines, 604, 611) + "\n" +
    get(common_lines, 612, 619) + "\n" +
    get(common_lines, 620, 624) + "\n" +
    get(common_lines, 778, 801) + "\n"
)
common_lines = delete(common_lines, 778, 801)
common_lines = delete(common_lines, 620, 624)
common_lines = delete(common_lines, 612, 619)
common_lines = delete(common_lines, 604, 611)
common_lines = delete(common_lines, 587, 603)
common_lines = delete(common_lines, 572, 586)
common_lines = delete(common_lines, 495, 506)
common_lines = delete(common_lines, 477, 494)
common_lines = delete(common_lines, 465, 476)
common_lines = delete(common_lines, 453, 464)
common_lines = delete(common_lines, 24, 30)
common_lines = delete(common_lines, 22, 23)

# 3. schools.rs: get_student_by_school_and_name (274-296), get_student (297-315), student_from_row (316-338)
schools_content = (
    get(common_lines, 274, 296) + "\n" +
    get(common_lines, 297, 315) + "\n" +
    get(common_lines, 316, 338) + "\n"
)
common_lines = delete(common_lines, 316, 338)
common_lines = delete(common_lines, 297, 315)
common_lines = delete(common_lines, 274, 296)

# ── Write common.rs (trimmed) ────────────────────────────────────────────────
with open(COMMON, "w") as f:
    f.write("".join(common_lines))
print("Updated common.rs")

# ── Write audit.rs ───────────────────────────────────────────────────────────
with open(AUDIT, "w") as f:
    f.write("use rusqlite::{params, Connection};\n\n")
    f.write(audit_content)
print("Created audit.rs")

# ── Append to tickets.rs ─────────────────────────────────────────────────────
with open(TICKETS, "a") as f:
    f.write("\n")
    f.write(tickets_content)
print("Appended to tickets.rs")

# ── Append to schools.rs ─────────────────────────────────────────────────────
with open(SCHOOLS, "a") as f:
    f.write("\n")
    f.write(schools_content)
print("Appended to schools.rs")

# ── Update mod.rs ────────────────────────────────────────────────────────────
with open(MOD_RS) as f:
    mod_content = f.read()

# Add pub mod audit;
mod_content = mod_content.replace(
    "pub mod common;\n",
    "pub mod audit;\npub mod common;\n"
)

# Update common re-exports: remove record_history, record_audit, insert_audit_log, get_student
# and add audit re-exports
mod_content = mod_content.replace(
    "    record_history, record_audit, insert_audit_log, get_student,",
    ""
)

# Add audit re-export block before the common block
mod_content = mod_content.replace(
    "// Common helpers",
    "// Audit\npub use audit::{record_audit, insert_audit_log};\n\n// Common helpers"
)

# Add get_student to schools re-export block
mod_content = mod_content.replace(
    "    list_students, create_student, update_student, delete_student,",
    "    list_students, create_student, update_student, delete_student, get_student,"
)

# Remove get_student from common re-exports if it exists there
mod_content = mod_content.replace(
    "// Common helpers (re-exported for backward compat where routes use them directly)\npub use common::{\n};",
    "// Common helpers (re-exported for backward compat where routes use them directly)\npub use common::{\n    validate_nonempty, assignment_rule_from_row, get_region_by_name,\n};"
)

with open(MOD_RS, "w") as f:
    f.write(mod_content)
print("Updated mod.rs")

# ── Update imports in domain modules ─────────────────────────────────────────

# tickets.rs needs record_audit from audit.rs
with open(TICKETS) as f:
    tickets_src = f.read()
tickets_src = tickets_src.replace(
    "use super::common::*;",
    "use super::common::*;\nuse super::audit::*;"
)
with open(TICKETS, "w") as f:
    f.write(tickets_src)
print("Updated tickets.rs imports")

# schools.rs needs record_audit from audit.rs
with open(SCHOOLS) as f:
    schools_src = f.read()
schools_src = schools_src.replace(
    "use super::common::*;",
    "use super::common::*;\nuse super::audit::*;"
)
with open(SCHOOLS, "w") as f:
    f.write(schools_src)
print("Updated schools.rs imports")

# faculty.rs needs record_audit and insert_audit_log from audit.rs
with open("/home/abhi/saathi-dev/backend/src/repo/faculty.rs") as f:
    faculty_src = f.read()
faculty_src = faculty_src.replace(
    "use super::common::*;",
    "use super::common::*;\nuse super::audit::*;"
)
with open("/home/abhi/saathi-dev/backend/src/repo/faculty.rs", "w") as f:
    f.write(faculty_src)
print("Updated faculty.rs imports")

# ops.rs needs insert_audit_log from audit.rs
with open("/home/abhi/saathi-dev/backend/src/repo/ops.rs") as f:
    ops_src = f.read()
ops_src = ops_src.replace(
    "use super::common::*;",
    "use super::common::*;\nuse super::audit::*;"
)
with open("/home/abhi/saathi-dev/backend/src/repo/ops.rs", "w") as f:
    f.write(ops_src)
print("Updated ops.rs imports")

print("\nRefactor complete.")
