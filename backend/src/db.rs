// This file is identical in logic to the desktop version (src-tauri/src/db.rs).
// The only difference is that open_db() takes a file path string instead of a Tauri AppHandle.
use rusqlite::Connection;

pub const DEFAULT_SLA_POLICIES: &[(&str, i64)] = &[
    ("Academic Support", 48),
    ("Attendance", 48),
    ("Assessment", 24),
    ("Device", 72),
    ("Learning Platform", 24),
    ("Operations", 72),
    ("Parent Communication", 48),
];

pub const DEFAULT_ASSIGNMENT_RULES: &[(&str, &str)] = &[
    ("Academic Support", "Academic Coordinator"),
    ("Learning Platform", "Platform Support"),
    ("IT / Device", "Device Desk"),
    ("Operations", "Operations Desk"),
    ("Parent Communication", "Parent Success"),
];

pub const DEFAULT_ESCALATION_AT_RISK_HOURS: i64 = 24;
pub const DEFAULT_ESCALATION_ASSIGNEE: &str = "Program Supervisor";

pub const DEFAULT_COMMUNICATION_TEMPLATES: &[(&str, &str, &str)] = &[
    ("Parent progress update", "Parent", "Dear Parent, we are reviewing the request and will share the next update after coordinating with the academic team."),
    ("Student follow-up", "Student", "Hi, please share the latest status from your side so we can close the loop on this support request."),
    ("SLA delay apology", "Parent", "We apologize for the delay. This request has been escalated and we are prioritizing resolution."),
    ("Issue resolved", "School", "The reported issue has been resolved. Please confirm if any further support is required."),
    ("Internal escalation note", "Internal", "Escalating for supervisor review due to SLA risk, repeated follow-up, or dependency on another team."),
];

pub const DEFAULT_LECTURE_MODELS: &[(&str, i64, i64)] =
    &[("3x3", 3, 3), ("4x3", 4, 3), ("4x2", 4, 2), ("5x2", 5, 2), ("5x3", 5, 3)];

/// Opens (or creates) the SQLite database at the given path and runs all migrations.
pub fn open_db(path: &str) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    initialize_db(&conn)?;
    Ok(conn)
}

pub fn initialize_db(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS tickets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            requester TEXT NOT NULL,
            assignee TEXT NOT NULL DEFAULT 'Unassigned',
            status TEXT NOT NULL DEFAULT 'Open',
            priority TEXT NOT NULL DEFAULT 'Medium',
            queue TEXT NOT NULL DEFAULT 'Academic Support',
            school_id INTEGER,
            school_name TEXT NOT NULL DEFAULT '',
            student_name TEXT NOT NULL DEFAULT '',
            grade_level TEXT NOT NULL DEFAULT '',
            program_track TEXT NOT NULL DEFAULT '',
            issue_category TEXT NOT NULL DEFAULT '',
            sla_due_at TEXT NOT NULL DEFAULT (datetime('now', '+2 days', 'localtime')),
            escalation_status TEXT NOT NULL DEFAULT 'None',
            escalated_at TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS ticket_comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ticket_id INTEGER NOT NULL,
            author TEXT NOT NULL,
            body TEXT NOT NULL,
            is_internal INTEGER NOT NULL DEFAULT 0,
            channel TEXT NOT NULL DEFAULT 'Local',
            audience TEXT NOT NULL DEFAULT 'Internal',
            recipient_name TEXT NOT NULL DEFAULT '',
            recipient_contact TEXT NOT NULL DEFAULT '',
            delivery_status TEXT NOT NULL DEFAULT 'Logged',
            last_contacted_at TEXT NOT NULL DEFAULT '',
            next_follow_up_due TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY(ticket_id) REFERENCES tickets(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        ",
    )
    .map_err(|e| e.to_string())?;

    apply_migrations(conn)?;

    let ticket_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM tickets", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if ticket_count == 0 {
        seed_tickets(conn)?;
    }

    seed_test_users(conn)?;

    Ok(())
}

fn apply_migrations(conn: &Connection) -> Result<(), String> {
    // Migrations 1–20 are identical to the desktop version.
    // Only the open_db() entry point changed — everything inside is portable Rust + SQLite.

    run_migration(conn, 1, "
        CREATE TABLE IF NOT EXISTS ticket_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ticket_id INTEGER NOT NULL,
            actor TEXT NOT NULL DEFAULT 'System',
            field TEXT NOT NULL,
            old_value TEXT NOT NULL DEFAULT '',
            new_value TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY(ticket_id) REFERENCES tickets(id) ON DELETE CASCADE
        );
    ")?;

    run_migration(conn, 2, "
        CREATE TABLE IF NOT EXISTS ticket_attachments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ticket_id INTEGER NOT NULL,
            original_filename TEXT NOT NULL,
            stored_path TEXT NOT NULL,
            size_bytes INTEGER NOT NULL DEFAULT 0,
            uploaded_by TEXT NOT NULL DEFAULT 'Service Desk',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY(ticket_id) REFERENCES tickets(id) ON DELETE CASCADE
        );
    ")?;

    // Migration 3: SIP columns (idempotent via column_exists checks)
    if !migration_applied(conn, 3)? {
        for col in ["school_name", "student_name", "grade_level", "program_track", "issue_category"] {
            if !column_exists(conn, "tickets", col)? {
                conn.execute(&format!("ALTER TABLE tickets ADD COLUMN {col} TEXT NOT NULL DEFAULT ''"), [])
                    .map_err(|e| e.to_string())?;
            }
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (3)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 4: sla_due_at
    if !migration_applied(conn, 4)? {
        if !column_exists(conn, "tickets", "sla_due_at")? {
            conn.execute_batch("
                ALTER TABLE tickets ADD COLUMN sla_due_at TEXT NOT NULL DEFAULT '';
                UPDATE tickets SET sla_due_at = CASE
                    WHEN issue_category IN ('Assessment', 'Learning Platform') THEN datetime(created_at, '+1 day')
                    WHEN issue_category IN ('Attendance', 'Parent Communication') THEN datetime(created_at, '+2 days')
                    WHEN issue_category IN ('Device', 'Operations') THEN datetime(created_at, '+3 days')
                    ELSE datetime(created_at, '+2 days')
                END WHERE sla_due_at = '';
            ").map_err(|e| e.to_string())?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (4)", [])
            .map_err(|e| e.to_string())?;
    }

    run_migration(conn, 5, "
        CREATE TABLE IF NOT EXISTS sla_policies (
            issue_category TEXT PRIMARY KEY,
            hours INTEGER NOT NULL CHECK(hours > 0),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
    ")?;
    seed_sla_policies(conn)?;

    run_migration(conn, 6, "
        CREATE TABLE IF NOT EXISTS schools (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE TABLE IF NOT EXISTS students (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            school_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            grade_level TEXT NOT NULL,
            program_track TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            UNIQUE(school_id, name),
            FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE
        );
    ")?;
    seed_master_data(conn)?;

    // Migration 7: queue column
    if !migration_applied(conn, 7)? {
        if !column_exists(conn, "tickets", "queue")? {
            conn.execute_batch("
                ALTER TABLE tickets ADD COLUMN queue TEXT NOT NULL DEFAULT 'Academic Support';
                UPDATE tickets SET queue = CASE
                    WHEN issue_category IN ('Academic Support', 'Assessment') THEN 'Academic Support'
                    WHEN issue_category = 'Learning Platform' THEN 'Learning Platform'
                    WHEN issue_category = 'Device' THEN 'IT / Device'
                    WHEN issue_category = 'Operations' THEN 'Operations'
                    WHEN issue_category IN ('Attendance', 'Parent Communication') THEN 'Parent Communication'
                    ELSE 'Academic Support'
                END;
            ").map_err(|e| e.to_string())?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (7)", [])
            .map_err(|e| e.to_string())?;
    }

    run_migration(conn, 8, "
        CREATE TABLE IF NOT EXISTS assignment_rules (
            queue TEXT PRIMARY KEY,
            assignee TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
    ")?;
    seed_assignment_rules(conn)?;

    // Migration 9: escalation columns
    if !migration_applied(conn, 9)? {
        if !column_exists(conn, "tickets", "escalation_status")? {
            conn.execute_batch("
                ALTER TABLE tickets ADD COLUMN escalation_status TEXT NOT NULL DEFAULT 'None';
                ALTER TABLE tickets ADD COLUMN escalated_at TEXT NOT NULL DEFAULT '';
                UPDATE tickets SET
                    escalation_status = CASE
                        WHEN status IN ('Resolved', 'Closed') THEN 'None'
                        WHEN datetime(sla_due_at) <= datetime('now', 'localtime') THEN 'Escalated'
                        WHEN datetime(sla_due_at) <= datetime('now', '+24 hours', 'localtime') THEN 'At Risk'
                        ELSE 'None'
                    END,
                    escalated_at = CASE
                        WHEN status NOT IN ('Resolved', 'Closed')
                             AND datetime(sla_due_at) <= datetime('now', '+24 hours', 'localtime')
                        THEN datetime('now', 'localtime')
                        ELSE ''
                    END;
            ").map_err(|e| e.to_string())?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (9)", [])
            .map_err(|e| e.to_string())?;
    }

    run_migration(conn, 10, "
        CREATE TABLE IF NOT EXISTS escalation_policy (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            at_risk_hours INTEGER NOT NULL CHECK(at_risk_hours >= 1),
            escalation_assignee TEXT NOT NULL,
            auto_assign_on_breach INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
    ")?;
    seed_escalation_policy(conn)?;

    run_migration(conn, 11, "
        CREATE TABLE IF NOT EXISTS communication_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            audience TEXT NOT NULL,
            body TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
    ")?;

    // Migration 12: school profile columns
    if !migration_applied(conn, 12)? {
        if !column_exists(conn, "schools", "program_model")? {
            let cols = [
                "program_model", "distance_classification",
                "sip_academic_owner_role", "sip_academic_owner_name",
                "sip_academic_owner_mobile", "sip_academic_owner_email",
                "center_head_name", "center_head_mobile", "center_head_email",
                "principal_name", "principal_mobile", "principal_email",
                "school_spoc_name", "school_spoc_mobile", "school_spoc_email",
                "central_academic_spoc_name", "central_academic_spoc_mobile", "central_academic_spoc_email",
                "central_business_spoc_name", "central_business_spoc_mobile", "central_business_spoc_email",
            ];
            for col in cols {
                conn.execute(&format!("ALTER TABLE schools ADD COLUMN {col} TEXT NOT NULL DEFAULT ''"), [])
                    .map_err(|e| e.to_string())?;
            }
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (12)", [])
            .map_err(|e| e.to_string())?;
    }

    run_migration(conn, 13, "
        CREATE TABLE IF NOT EXISTS lecture_models (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            days_per_week INTEGER NOT NULL CHECK(days_per_week > 0),
            lectures_per_day INTEGER NOT NULL CHECK(lectures_per_day > 0),
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE TABLE IF NOT EXISTS school_class_plans (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            school_id INTEGER NOT NULL,
            grade_level TEXT NOT NULL,
            lecture_model_id INTEGER NOT NULL,
            batch_pattern TEXT NOT NULL,
            aop_admissions INTEGER NOT NULL DEFAULT 0 CHECK(aop_admissions >= 0),
            actual_admissions INTEGER NOT NULL DEFAULT 0 CHECK(actual_admissions >= 0),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            UNIQUE(school_id, grade_level),
            FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE,
            FOREIGN KEY(lecture_model_id) REFERENCES lecture_models(id)
        );
    ")?;

    // Migration 14: regions + BH columns on schools
    if !migration_applied(conn, 14)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS regions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                regional_academic_head_name TEXT NOT NULL DEFAULT '',
                regional_academic_head_mobile TEXT NOT NULL DEFAULT '',
                regional_academic_head_email TEXT NOT NULL DEFAULT '',
                regional_business_head_name TEXT NOT NULL DEFAULT '',
                regional_business_head_mobile TEXT NOT NULL DEFAULT '',
                regional_business_head_email TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );
        ").map_err(|e| e.to_string())?;
        if !column_exists(conn, "schools", "region_id")? {
            conn.execute_batch("
                ALTER TABLE schools ADD COLUMN region_id INTEGER;
                ALTER TABLE schools ADD COLUMN bh_name TEXT NOT NULL DEFAULT '';
                ALTER TABLE schools ADD COLUMN bh_mobile TEXT NOT NULL DEFAULT '';
                ALTER TABLE schools ADD COLUMN bh_email TEXT NOT NULL DEFAULT '';
            ").map_err(|e| e.to_string())?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (14)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 15: AOM columns
    if !migration_applied(conn, 15)? {
        if !column_exists(conn, "schools", "aom_name")? {
            conn.execute_batch("
                ALTER TABLE schools ADD COLUMN aom_name TEXT NOT NULL DEFAULT '';
                ALTER TABLE schools ADD COLUMN aom_mobile TEXT NOT NULL DEFAULT '';
                ALTER TABLE schools ADD COLUMN aom_email TEXT NOT NULL DEFAULT '';
            ").map_err(|e| e.to_string())?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (15)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 16: school lifecycle + region history
    if !migration_applied(conn, 16)? {
        for (col, def) in [
            ("is_dropped", "INTEGER NOT NULL DEFAULT 0"),
            ("dropped_at", "TEXT NOT NULL DEFAULT ''"),
            ("dropped_reason", "TEXT NOT NULL DEFAULT ''"),
        ] {
            if !column_exists(conn, "schools", col)? {
                conn.execute(&format!("ALTER TABLE schools ADD COLUMN {col} {def}"), [])
                    .map_err(|e| e.to_string())?;
            }
        }
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS school_region_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                school_id INTEGER NOT NULL,
                old_region_id INTEGER,
                old_region_name TEXT NOT NULL DEFAULT '',
                new_region_id INTEGER,
                new_region_name TEXT NOT NULL DEFAULT '',
                changed_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE
            );
            INSERT INTO schema_migrations (version) VALUES (16);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 17: school_id FK on tickets + audit_log
    if !migration_applied(conn, 17)? {
        if !column_exists(conn, "tickets", "school_id")? {
            conn.execute("ALTER TABLE tickets ADD COLUMN school_id INTEGER", [])
                .map_err(|e| e.to_string())?;
        }
        conn.execute_batch("
            UPDATE tickets SET school_id = (
                SELECT schools.id FROM schools WHERE schools.name = tickets.school_name LIMIT 1
            ) WHERE school_id IS NULL AND school_name <> '';
            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_type TEXT NOT NULL,
                entity_id INTEGER NOT NULL,
                action TEXT NOT NULL,
                actor TEXT NOT NULL DEFAULT 'Service Desk',
                summary TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );
            INSERT INTO schema_migrations (version) VALUES (17);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 18: communication metadata on ticket_comments
    if !migration_applied(conn, 18)? {
        for (col, def) in [
            ("channel", "TEXT NOT NULL DEFAULT 'Local'"),
            ("audience", "TEXT NOT NULL DEFAULT 'Internal'"),
            ("recipient_name", "TEXT NOT NULL DEFAULT ''"),
            ("recipient_contact", "TEXT NOT NULL DEFAULT ''"),
            ("delivery_status", "TEXT NOT NULL DEFAULT 'Logged'"),
        ] {
            if !column_exists(conn, "ticket_comments", col)? {
                conn.execute(&format!("ALTER TABLE ticket_comments ADD COLUMN {col} {def}"), [])
                    .map_err(|e| e.to_string())?;
            }
        }
        conn.execute_batch("
            UPDATE ticket_comments
            SET channel = CASE WHEN is_internal = 1 THEN 'Internal Note' ELSE 'Local' END,
                audience = CASE WHEN is_internal = 1 THEN 'Internal' ELSE 'School' END,
                delivery_status = 'Logged'
            WHERE channel = '' OR audience = '' OR delivery_status = '';
            INSERT INTO schema_migrations (version) VALUES (18);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 19: follow-up tracking on ticket_comments
    if !migration_applied(conn, 19)? {
        for col in ["last_contacted_at", "next_follow_up_due"] {
            if !column_exists(conn, "ticket_comments", col)? {
                conn.execute(
                    &format!("ALTER TABLE ticket_comments ADD COLUMN {col} TEXT NOT NULL DEFAULT ''"),
                    [],
                ).map_err(|e| e.to_string())?;
            }
        }
        conn.execute_batch("
            UPDATE ticket_comments SET last_contacted_at = created_at
            WHERE is_internal = 0 AND last_contacted_at = '';
            INSERT INTO schema_migrations (version) VALUES (19);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 20: users table
    if !migration_applied(conn, 20)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'agent',
                password_hash TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                last_login_at TEXT NOT NULL DEFAULT ''
            );
        ").map_err(|e| e.to_string())?;

        let hash = bcrypt::hash("admin123", bcrypt::DEFAULT_COST)
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO users (username, display_name, role, password_hash) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["admin", "Administrator", "admin", hash],
        ).map_err(|e| e.to_string())?;

        conn.execute("INSERT INTO schema_migrations (version) VALUES (20)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 21: Mapped VP Center on schools
    if !migration_applied(conn, 21)? {
        if !column_exists(conn, "schools", "mapped_vp_center")? {
            conn.execute(
                "ALTER TABLE schools ADD COLUMN mapped_vp_center TEXT NOT NULL DEFAULT ''",
                [],
            ).map_err(|e| e.to_string())?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (21)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 22: class plans get track + registrations + new unique key
    // (school_id, grade_level, track) — supports JEE/NEET split for grades
    // 11/12/Dropper while leaving 6-10 with empty track.
    if !migration_applied(conn, 22)? {
        // Add new columns first if missing.
        if !column_exists(conn, "school_class_plans", "track")? {
            conn.execute(
                "ALTER TABLE school_class_plans ADD COLUMN track TEXT NOT NULL DEFAULT ''",
                [],
            ).map_err(|e| e.to_string())?;
        }
        if !column_exists(conn, "school_class_plans", "registrations")? {
            conn.execute(
                "ALTER TABLE school_class_plans ADD COLUMN registrations INTEGER NOT NULL DEFAULT 0",
                [],
            ).map_err(|e| e.to_string())?;
        }
        // Recreate table to swap the UNIQUE constraint from (school_id,
        // grade_level) to (school_id, grade_level, track).
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS school_class_plans_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                school_id INTEGER NOT NULL,
                grade_level TEXT NOT NULL,
                track TEXT NOT NULL DEFAULT '',
                lecture_model_id INTEGER NOT NULL,
                batch_pattern TEXT NOT NULL,
                aop_admissions INTEGER NOT NULL DEFAULT 0 CHECK(aop_admissions >= 0),
                registrations INTEGER NOT NULL DEFAULT 0 CHECK(registrations >= 0),
                actual_admissions INTEGER NOT NULL DEFAULT 0 CHECK(actual_admissions >= 0),
                updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(school_id, grade_level, track),
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE,
                FOREIGN KEY(lecture_model_id) REFERENCES lecture_models(id)
            );
            INSERT INTO school_class_plans_new
                (id, school_id, grade_level, track, lecture_model_id, batch_pattern,
                 aop_admissions, registrations, actual_admissions, updated_at)
            SELECT id, school_id, grade_level, COALESCE(track, ''), lecture_model_id,
                   batch_pattern, aop_admissions, COALESCE(registrations, 0),
                   actual_admissions, updated_at
            FROM school_class_plans;
            DROP TABLE school_class_plans;
            ALTER TABLE school_class_plans_new RENAME TO school_class_plans;
            INSERT INTO schema_migrations (version) VALUES (22);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 23: subjects table — track-scoped controlled vocabulary.
    // JEE: Physics/Chemistry/Mathematics. NEET: Physics/Chemistry/Botany/Zoology.
    // Foundation: Physics/Chemistry/Mathematics/Biology (default) + English/SST
    // (optional, opt-in per school via school_optional_subjects).
    if !migration_applied(conn, 23)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS subjects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                track TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 1,
                sort_order INTEGER NOT NULL DEFAULT 0,
                UNIQUE(name, track)
            );
            INSERT INTO subjects (name, track, is_default, sort_order) VALUES
                ('Physics',     'JEE',        1, 10),
                ('Chemistry',   'JEE',        1, 20),
                ('Mathematics', 'JEE',        1, 30),
                ('Physics',     'NEET',       1, 10),
                ('Chemistry',   'NEET',       1, 20),
                ('Botany',      'NEET',       1, 30),
                ('Zoology',     'NEET',       1, 40),
                ('Physics',     'Foundation', 1, 10),
                ('Chemistry',   'Foundation', 1, 20),
                ('Mathematics', 'Foundation', 1, 30),
                ('Biology',     'Foundation', 1, 40),
                ('English',     'Foundation', 0, 50),
                ('SST',         'Foundation', 0, 60);
            INSERT INTO schema_migrations (version) VALUES (23);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 24: school_optional_subjects — per-school opt-in for Foundation
    // optional subjects (English, SST). UI presents these as toggles.
    if !migration_applied(conn, 24)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS school_optional_subjects (
                school_id INTEGER NOT NULL,
                subject_id INTEGER NOT NULL,
                UNIQUE(school_id, subject_id),
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE,
                FOREIGN KEY(subject_id) REFERENCES subjects(id) ON DELETE CASCADE
            );
            INSERT INTO schema_migrations (version) VALUES (24);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 25: students.track for JEE/NEET assignment at Grades 11/12/Dropper.
    // Empty string for Foundation grades. Existing rows default to '' so the
    // column add is data-preserving.
    if !migration_applied(conn, 25)? {
        if !column_exists(conn, "students", "track")? {
            conn.execute(
                "ALTER TABLE students ADD COLUMN track TEXT NOT NULL DEFAULT ''",
                [],
            ).map_err(|e| e.to_string())?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (25)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 26: user_schools — many-to-many scope mapping. Used by AOM
    // role to limit which schools they can manage; later by faculty role for
    // attendance. Admin/agent/viewer have no entries (unscoped).
    if !migration_applied(conn, 26)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS user_schools (
                user_id INTEGER NOT NULL,
                school_id INTEGER NOT NULL,
                UNIQUE(user_id, school_id),
                FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE
            );
            INSERT INTO schema_migrations (version) VALUES (26);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 27: faculty_assignments — what each faculty teaches.
    // Many-to-many: a faculty can teach multiple (school, grade, track, subject)
    // combinations. The timetable references this implicitly via the same keys.
    if !migration_applied(conn, 27)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS faculty_assignments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                faculty_user_id INTEGER NOT NULL,
                school_id INTEGER NOT NULL,
                grade_level TEXT NOT NULL,
                track TEXT NOT NULL DEFAULT '',
                subject_id INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(faculty_user_id, school_id, grade_level, track, subject_id),
                FOREIGN KEY(faculty_user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE,
                FOREIGN KEY(subject_id) REFERENCES subjects(id) ON DELETE RESTRICT
            );
            INSERT INTO schema_migrations (version) VALUES (27);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 28: timetable_slots — recurring weekly schedule per class.
    // day_of_week is 0..6 (0 = Monday). period is 1..N. Unique on the schedule
    // grid coordinates so a slot can't be double-booked.
    if !migration_applied(conn, 28)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS timetable_slots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                school_id INTEGER NOT NULL,
                grade_level TEXT NOT NULL,
                track TEXT NOT NULL DEFAULT '',
                batch_pattern TEXT NOT NULL,
                day_of_week INTEGER NOT NULL CHECK(day_of_week >= 0 AND day_of_week <= 6),
                period INTEGER NOT NULL CHECK(period >= 1),
                subject_id INTEGER NOT NULL,
                faculty_user_id INTEGER,
                start_time TEXT NOT NULL DEFAULT '',
                end_time TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(school_id, grade_level, track, batch_pattern, day_of_week, period),
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE,
                FOREIGN KEY(subject_id) REFERENCES subjects(id) ON DELETE RESTRICT,
                FOREIGN KEY(faculty_user_id) REFERENCES users(id) ON DELETE SET NULL
            );
            INSERT INTO schema_migrations (version) VALUES (28);
        ").map_err(|e| e.to_string())?;
    }

    seed_communication_templates(conn)?;
    seed_lecture_models(conn)?;

    Ok(())
}

// ── Migration helpers ─────────────────────────────────────────────────────────

fn migration_applied(conn: &Connection, version: i64) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
            rusqlite::params![version],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

fn run_migration(conn: &Connection, version: i64, sql: &str) -> Result<(), String> {
    if !migration_applied(conn, version)? {
        conn.execute_batch(sql).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO schema_migrations (version) VALUES (?1)",
            rusqlite::params![version],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?;
    for name in rows {
        if name.map_err(|e| e.to_string())? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

// ── Seed functions (identical to desktop) ─────────────────────────────────────

pub fn seed_sla_policies(conn: &Connection) -> Result<(), String> {
    for (category, hours) in DEFAULT_SLA_POLICIES {
        conn.execute(
            "INSERT INTO sla_policies (issue_category, hours) VALUES (?1, ?2) ON CONFLICT(issue_category) DO NOTHING",
            rusqlite::params![category, hours],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn seed_master_data(conn: &Connection) -> Result<(), String> {
    let students = [
        ("Green Valley Public School", "Aarav Shah", "Grade 11", "JEE Foundation"),
        ("North City Senior Secondary", "Nisha Rao", "Grade 10", "NEET Foundation"),
        ("Sunrise International School", "Rohan Iyer", "Grade 9", "Integrated STEM"),
    ];
    for (school, student, grade, track) in students {
        conn.execute(
            "INSERT INTO schools (name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
            rusqlite::params![school],
        ).map_err(|e| e.to_string())?;
        let school_id: i64 = conn
            .query_row("SELECT id FROM schools WHERE name = ?1", rusqlite::params![school], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO students (school_id, name, grade_level, program_track) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(school_id, name) DO NOTHING",
            rusqlite::params![school_id, student, grade, track],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn seed_assignment_rules(conn: &Connection) -> Result<(), String> {
    for (queue, assignee) in DEFAULT_ASSIGNMENT_RULES {
        conn.execute(
            "INSERT INTO assignment_rules (queue, assignee) VALUES (?1, ?2) ON CONFLICT(queue) DO NOTHING",
            rusqlite::params![queue, assignee],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn seed_escalation_policy(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "INSERT INTO escalation_policy (id, at_risk_hours, escalation_assignee, auto_assign_on_breach) VALUES (1, ?1, ?2, 1) ON CONFLICT(id) DO NOTHING",
        rusqlite::params![DEFAULT_ESCALATION_AT_RISK_HOURS, DEFAULT_ESCALATION_ASSIGNEE],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn seed_communication_templates(conn: &Connection) -> Result<(), String> {
    for (name, audience, body) in DEFAULT_COMMUNICATION_TEMPLATES {
        conn.execute(
            "INSERT INTO communication_templates (name, audience, body) VALUES (?1, ?2, ?3) ON CONFLICT(name) DO NOTHING",
            rusqlite::params![name, audience, body],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn seed_lecture_models(conn: &Connection) -> Result<(), String> {
    for (name, days_per_week, lectures_per_day) in DEFAULT_LECTURE_MODELS {
        conn.execute(
            "INSERT INTO lecture_models (name, days_per_week, lectures_per_day) VALUES (?1, ?2, ?3) ON CONFLICT(name) DO NOTHING",
            rusqlite::params![name, days_per_week, lectures_per_day],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_tickets(conn: &Connection) -> Result<(), String> {
    let samples = [
        ("VPN access request for finance team", "Finance team members need VPN access before month-end close.", "Meera Shah", "Unassigned", "Open", "High", "Learning Platform", "Green Valley Public School", "Aarav Shah", "Grade 11", "JEE Foundation", "Academic Support", "2026-04-23 09:30:00", "None", ""),
        ("Printer queue stuck on third floor", "The shared printer queue is holding jobs and preventing printing.", "Arjun Rao", "Nina Patel", "In Progress", "Medium", "Operations", "North City Senior Secondary", "Nisha Rao", "Grade 10", "NEET Foundation", "Operations", "2026-04-24 10:15:00", "None", ""),
        ("Laptop replacement approval needed", "The current laptop has intermittent power failures.", "Dev Iyer", "Sam Thomas", "Pending", "Low", "IT / Device", "Sunrise International School", "Rohan Iyer", "Grade 9", "Integrated STEM", "Device", "2026-04-24 11:00:00", "None", ""),
    ];
    for (title, description, requester, assignee, status, priority, queue, school_name, student_name, grade_level, program_track, issue_category, sla_due_at, escalation_status, escalated_at) in samples {
        conn.execute(
            "INSERT INTO tickets (title, description, requester, assignee, status, priority, queue, school_id, school_name, student_name, grade_level, program_track, issue_category, sla_due_at, escalation_status, escalated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, (SELECT id FROM schools WHERE name = ?8 LIMIT 1), ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![title, description, requester, assignee, status, priority, queue, school_name, student_name, grade_level, program_track, issue_category, sla_due_at, escalation_status, escalated_at],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_test_users(conn: &Connection) -> Result<(), String> {
    let test_users = [
        ("aom1", "AOM One", "aom", "aom123"),
        ("faculty1", "Faculty One", "faculty", "faculty123"),
        ("viewer1", "Viewer One", "viewer", "viewer123"),
    ];
    for (username, display_name, role, password) in test_users {
        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO users (username, display_name, role, password_hash) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![username, display_name, role, hash],
        ).map_err(|e| e.to_string())?;
    }
    // Assign aom1 → Green Valley Public School (id=1)
    // Assign faculty1 → North City Senior Secondary (id=2)
    let mappings = [
        ("aom1", 1i64),
        ("faculty1", 2i64),
    ];
    for (username, school_id) in mappings {
        let user_id: i64 = conn
            .query_row("SELECT id FROM users WHERE username = ?1", rusqlite::params![username], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO user_schools (user_id, school_id) VALUES (?1, ?2)",
            rusqlite::params![user_id, school_id],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}
