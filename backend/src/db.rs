// This file is identical in logic to the desktop version (src-tauri/src/db.rs).
// The only difference is that open_db() takes a file path string instead of a Tauri AppHandle.
use rusqlite::{Connection, OptionalExtension};

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

pub const DEFAULT_LECTURE_MODELS: &[(&str, i64, i64)] = &[
    ("3x3", 3, 3),
    ("4x3", 4, 3),
    ("4x2", 4, 2),
    ("5x2", 5, 2),
    ("5x3", 5, 3),
];

/// Opens (or creates) the SQLite database at the given path and runs all migrations.
pub fn open_db(path: &str) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    initialize_db(&conn)?;
    create_indexes(&conn)?;
    Ok(conn)
}

fn create_indexes(conn: &Connection) -> Result<(), String> {
    conn.execute_batch("
        CREATE INDEX IF NOT EXISTS idx_tickets_school      ON tickets(school_id);
        CREATE INDEX IF NOT EXISTS idx_tickets_updated     ON tickets(updated_at);
        CREATE INDEX IF NOT EXISTS idx_tickets_escalation  ON tickets(escalation_status, sla_due_at);
        CREATE INDEX IF NOT EXISTS idx_tickets_status      ON tickets(status);
        CREATE INDEX IF NOT EXISTS idx_attendance_student  ON attendance_records(student_id);
        CREATE INDEX IF NOT EXISTS idx_students_school     ON students(school_id);
        CREATE INDEX IF NOT EXISTS idx_students_batch      ON students(school_id, grade_level, track, batch_id);
        CREATE INDEX IF NOT EXISTS idx_audit_log_created   ON audit_log(created_at);
        CREATE INDEX IF NOT EXISTS idx_audit_log_entity    ON audit_log(entity_type, entity_id);
        CREATE INDEX IF NOT EXISTS idx_ticket_history_ticket ON ticket_history(ticket_id);
        CREATE INDEX IF NOT EXISTS idx_ticket_comments_ticket ON ticket_comments(ticket_id);
        CREATE INDEX IF NOT EXISTS idx_users_role          ON users(role);
        CREATE INDEX IF NOT EXISTS idx_faculty_assignments_faculty ON faculty_assignments(faculty_user_id);
        CREATE INDEX IF NOT EXISTS idx_faculty_assignments_school  ON faculty_assignments(school_id);
        CREATE INDEX IF NOT EXISTS idx_timetable_slots_room ON timetable_slots(room) WHERE room != '';
        CREATE INDEX IF NOT EXISTS idx_timetable_slots_session_type ON timetable_slots(session_type);
        CREATE INDEX IF NOT EXISTS idx_timetable_weekly_room ON timetable_weekly_slots(room) WHERE room != '';
        CREATE INDEX IF NOT EXISTS idx_timetable_weekly_session_type ON timetable_weekly_slots(session_type);
        CREATE INDEX IF NOT EXISTS idx_timetable_weekly_faculty_week ON timetable_weekly_slots(faculty_user_id, week_start_date);
        CREATE INDEX IF NOT EXISTS idx_timetable_weekly_room_conflicts ON timetable_weekly_slots(school_id, week_start_date, day_of_week, room);
        CREATE INDEX IF NOT EXISTS idx_timetable_slots_compliance ON timetable_slots(school_id, grade_level, track, subject_id);
    ").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn initialize_db(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 5000;

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

    if std::env::var("TEST_SEED").unwrap_or_default() == "1" {
        seed_test_users(conn)?;
    }

    Ok(())
}

fn apply_migrations(conn: &Connection) -> Result<(), String> {
    // Migrations 1–20 are identical to the desktop version.
    // Only the open_db() entry point changed — everything inside is portable Rust + SQLite.

    run_migration(
        conn,
        1,
        "
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
    ",
    )?;

    run_migration(
        conn,
        2,
        "
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
    ",
    )?;

    // Migration 3: SIP columns (idempotent via column_exists checks)
    if !migration_applied(conn, 3)? {
        for col in [
            "school_name",
            "student_name",
            "grade_level",
            "program_track",
            "issue_category",
        ] {
            if !column_exists(conn, "tickets", col)? {
                conn.execute(
                    &format!("ALTER TABLE tickets ADD COLUMN {col} TEXT NOT NULL DEFAULT ''"),
                    [],
                )
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

    run_migration(
        conn,
        5,
        "
        CREATE TABLE IF NOT EXISTS sla_policies (
            issue_category TEXT PRIMARY KEY,
            hours INTEGER NOT NULL CHECK(hours > 0),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
    ",
    )?;
    seed_sla_policies(conn)?;

    run_migration(
        conn,
        6,
        "
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
    ",
    )?;
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

    run_migration(
        conn,
        8,
        "
        CREATE TABLE IF NOT EXISTS assignment_rules (
            queue TEXT PRIMARY KEY,
            assignee TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
    ",
    )?;
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

    run_migration(
        conn,
        10,
        "
        CREATE TABLE IF NOT EXISTS escalation_policy (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            at_risk_hours INTEGER NOT NULL CHECK(at_risk_hours >= 1),
            escalation_assignee TEXT NOT NULL,
            auto_assign_on_breach INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
    ",
    )?;
    seed_escalation_policy(conn)?;

    run_migration(
        conn,
        11,
        "
        CREATE TABLE IF NOT EXISTS communication_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            audience TEXT NOT NULL,
            body TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
    ",
    )?;

    // Migration 12: school profile columns
    if !migration_applied(conn, 12)? {
        if !column_exists(conn, "schools", "program_model")? {
            let cols = [
                "program_model",
                "distance_classification",
                "sip_academic_owner_role",
                "sip_academic_owner_name",
                "sip_academic_owner_mobile",
                "sip_academic_owner_email",
                "center_head_name",
                "center_head_mobile",
                "center_head_email",
                "principal_name",
                "principal_mobile",
                "principal_email",
                "school_spoc_name",
                "school_spoc_mobile",
                "school_spoc_email",
                "central_academic_spoc_name",
                "central_academic_spoc_mobile",
                "central_academic_spoc_email",
                "central_business_spoc_name",
                "central_business_spoc_mobile",
                "central_business_spoc_email",
            ];
            for col in cols {
                conn.execute(
                    &format!("ALTER TABLE schools ADD COLUMN {col} TEXT NOT NULL DEFAULT ''"),
                    [],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (12)", [])
            .map_err(|e| e.to_string())?;
    }

    run_migration(
        conn,
        13,
        "
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
    ",
    )?;

    // Migration 14: regions + BH columns on schools
    if !migration_applied(conn, 14)? {
        conn.execute_batch(
            "
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
        ",
        )
        .map_err(|e| e.to_string())?;
        if !column_exists(conn, "schools", "region_id")? {
            conn.execute_batch(
                "
                ALTER TABLE schools ADD COLUMN region_id INTEGER;
                ALTER TABLE schools ADD COLUMN bh_name TEXT NOT NULL DEFAULT '';
                ALTER TABLE schools ADD COLUMN bh_mobile TEXT NOT NULL DEFAULT '';
                ALTER TABLE schools ADD COLUMN bh_email TEXT NOT NULL DEFAULT '';
            ",
            )
            .map_err(|e| e.to_string())?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (14)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 15: AOM columns
    if !migration_applied(conn, 15)? {
        if !column_exists(conn, "schools", "aom_name")? {
            conn.execute_batch(
                "
                ALTER TABLE schools ADD COLUMN aom_name TEXT NOT NULL DEFAULT '';
                ALTER TABLE schools ADD COLUMN aom_mobile TEXT NOT NULL DEFAULT '';
                ALTER TABLE schools ADD COLUMN aom_email TEXT NOT NULL DEFAULT '';
            ",
            )
            .map_err(|e| e.to_string())?;
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
        conn.execute_batch(
            "
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
        ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 17: school_id FK on tickets + audit_log
    if !migration_applied(conn, 17)? {
        if !column_exists(conn, "tickets", "school_id")? {
            conn.execute("ALTER TABLE tickets ADD COLUMN school_id INTEGER", [])
                .map_err(|e| e.to_string())?;
        }
        conn.execute_batch(
            "
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
        ",
        )
        .map_err(|e| e.to_string())?;
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
                conn.execute(
                    &format!("ALTER TABLE ticket_comments ADD COLUMN {col} {def}"),
                    [],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        conn.execute_batch(
            "
            UPDATE ticket_comments
            SET channel = CASE WHEN is_internal = 1 THEN 'Internal Note' ELSE 'Local' END,
                audience = CASE WHEN is_internal = 1 THEN 'Internal' ELSE 'School' END,
                delivery_status = 'Logged'
            WHERE channel = '' OR audience = '' OR delivery_status = '';
            INSERT INTO schema_migrations (version) VALUES (18);
        ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 19: follow-up tracking on ticket_comments
    if !migration_applied(conn, 19)? {
        for col in ["last_contacted_at", "next_follow_up_due"] {
            if !column_exists(conn, "ticket_comments", col)? {
                conn.execute(
                    &format!(
                        "ALTER TABLE ticket_comments ADD COLUMN {col} TEXT NOT NULL DEFAULT ''"
                    ),
                    [],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        conn.execute_batch(
            "
            UPDATE ticket_comments SET last_contacted_at = created_at
            WHERE is_internal = 0 AND last_contacted_at = '';
            INSERT INTO schema_migrations (version) VALUES (19);
        ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 20: users table
    if !migration_applied(conn, 20)? {
        conn.execute_batch(
            "
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
        ",
        )
        .map_err(|e| e.to_string())?;

        let hash = bcrypt::hash("admin123", bcrypt::DEFAULT_COST).map_err(|e| e.to_string())?;
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
            )
            .map_err(|e| e.to_string())?;
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
            )
            .map_err(|e| e.to_string())?;
        }
        if !column_exists(conn, "school_class_plans", "registrations")? {
            conn.execute(
                "ALTER TABLE school_class_plans ADD COLUMN registrations INTEGER NOT NULL DEFAULT 0",
                [],
            ).map_err(|e| e.to_string())?;
        }
        // Recreate table to swap the UNIQUE constraint from (school_id,
        // grade_level) to (school_id, grade_level, track).
        conn.execute_batch(
            "
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
        ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 23: subjects table — track-scoped controlled vocabulary.
    // JEE: Physics/Chemistry/Mathematics. NEET: Physics/Chemistry/Botany/Zoology.
    // Foundation: Physics/Chemistry/Mathematics/Biology (default) + English/SST
    // (optional, opt-in per school via school_optional_subjects).
    if !migration_applied(conn, 23)? {
        conn.execute_batch(
            "
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
        ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 24: school_optional_subjects — per-school opt-in for Foundation
    // optional subjects (English, SST). UI presents these as toggles.
    if !migration_applied(conn, 24)? {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS school_optional_subjects (
                school_id INTEGER NOT NULL,
                subject_id INTEGER NOT NULL,
                UNIQUE(school_id, subject_id),
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE,
                FOREIGN KEY(subject_id) REFERENCES subjects(id) ON DELETE CASCADE
            );
            INSERT INTO schema_migrations (version) VALUES (24);
        ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 25: students.track for JEE/NEET assignment at Grades 11/12/Dropper.
    // Empty string for Foundation grades. Existing rows default to '' so the
    // column add is data-preserving.
    if !migration_applied(conn, 25)? {
        if !column_exists(conn, "students", "track")? {
            conn.execute(
                "ALTER TABLE students ADD COLUMN track TEXT NOT NULL DEFAULT ''",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (25)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 26: user_schools — many-to-many scope mapping. Used by AOM
    // role to limit which schools they can manage; later by faculty role for
    // attendance. Admin/agent/viewer have no entries (unscoped).
    if !migration_applied(conn, 26)? {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS user_schools (
                user_id INTEGER NOT NULL,
                school_id INTEGER NOT NULL,
                UNIQUE(user_id, school_id),
                FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE
            );
            INSERT INTO schema_migrations (version) VALUES (26);
        ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 27: faculty_assignments — what each faculty teaches.
    // Many-to-many: a faculty can teach multiple (school, grade, track, subject)
    // combinations. The timetable references this implicitly via the same keys.
    if !migration_applied(conn, 27)? {
        conn.execute_batch(
            "
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
        ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 28: timetable_slots — recurring weekly schedule per class.
    // day_of_week is 0..6 (0 = Monday). period is 1..N. Unique on the schedule
    // grid coordinates so a slot can't be double-booked.
    if !migration_applied(conn, 28)? {
        conn.execute_batch(
            "
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
        ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 29: lecture_sessions + attendance_records — Phase 2 faculty
    // attendance core. lecture_sessions bridges timetable template to actual
    // class instances. attendance_records stores per-student presence.
    if !migration_applied(conn, 29)? {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS lecture_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timetable_slot_id INTEGER NOT NULL,
                session_date TEXT NOT NULL,
                actual_faculty_user_id INTEGER,
                status TEXT NOT NULL DEFAULT 'Scheduled',
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(timetable_slot_id, session_date),
                FOREIGN KEY(timetable_slot_id) REFERENCES timetable_slots(id) ON DELETE CASCADE,
                FOREIGN KEY(actual_faculty_user_id) REFERENCES users(id) ON DELETE SET NULL
            );
            CREATE TABLE IF NOT EXISTS attendance_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                lecture_session_id INTEGER NOT NULL,
                student_id INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'Absent',
                marked_by_user_id INTEGER,
                marked_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(lecture_session_id, student_id),
                FOREIGN KEY(lecture_session_id) REFERENCES lecture_sessions(id) ON DELETE CASCADE,
                FOREIGN KEY(student_id) REFERENCES students(id) ON DELETE CASCADE,
                FOREIGN KEY(marked_by_user_id) REFERENCES users(id) ON DELETE SET NULL
            );
            INSERT INTO schema_migrations (version) VALUES (29);
        ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 31: timetable_weekly_slots — date-based weekly timetables that
    // override the recurring template (timetable_slots) for specific calendar weeks.
    // week_start_date is the Monday of the week (YYYY-MM-DD).
    if !migration_applied(conn, 31)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS timetable_weekly_slots (
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
                week_start_date TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(school_id, grade_level, track, batch_pattern, week_start_date, day_of_week, period),
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE,
                FOREIGN KEY(subject_id) REFERENCES subjects(id) ON DELETE RESTRICT,
                FOREIGN KEY(faculty_user_id) REFERENCES users(id) ON DELETE SET NULL
            );
            CREATE INDEX IF NOT EXISTS idx_timetable_weekly_school_week
                ON timetable_weekly_slots(school_id, week_start_date);
            INSERT INTO schema_migrations (version) VALUES (31);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 32: holidays — prevents phantom sessions on holidays.
    if !migration_applied(conn, 32)? {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS holidays (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                name TEXT NOT NULL,
                scope TEXT NOT NULL DEFAULT 'global' CHECK(scope IN ('global', 'region', 'school')),
                region_id INTEGER,
                school_id INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                FOREIGN KEY(region_id) REFERENCES regions(id) ON DELETE CASCADE,
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_holidays_date ON holidays(date);
            CREATE INDEX IF NOT EXISTS idx_holidays_school ON holidays(school_id);
            CREATE INDEX IF NOT EXISTS idx_holidays_region ON holidays(region_id);
            INSERT INTO schema_migrations (version) VALUES (32);
        ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 33: pwid — persistent identifier for operational staff.
    if !migration_applied(conn, 33)? {
        conn.execute_batch("
            ALTER TABLE users ADD COLUMN pwid TEXT;
            CREATE UNIQUE INDEX IF NOT EXISTS idx_users_pwid ON users(pwid) WHERE pwid IS NOT NULL AND pwid != '';
            INSERT INTO schema_migrations (version) VALUES (33);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 34: registration_number — unique student identifier per school.
    if !migration_applied(conn, 34)? {
        conn.execute_batch("
            ALTER TABLE students ADD COLUMN registration_number TEXT NOT NULL DEFAULT '';
            CREATE UNIQUE INDEX IF NOT EXISTS idx_student_reg ON students(school_id, registration_number) WHERE registration_number != '';
            INSERT INTO schema_migrations (version) VALUES (34);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 35: batches — explicit batch identifiers per school.
    if !migration_applied(conn, 35)? {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS batches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                school_id INTEGER NOT NULL,
                batch_id TEXT NOT NULL,
                grade_level TEXT NOT NULL,
                track TEXT NOT NULL DEFAULT '',
                batch_pattern TEXT NOT NULL DEFAULT 'Weekday',
                capacity INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(school_id, batch_id),
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_batches_school ON batches(school_id);
            CREATE INDEX IF NOT EXISTS idx_batches_grade ON batches(school_id, grade_level, track);
            INSERT INTO schema_migrations (version) VALUES (35);
        ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 36: timetable_slots gets batch_id column (denormalized for fast lookup).
    if !migration_applied(conn, 36)? {
        if !column_exists(conn, "timetable_slots", "batch_id")? {
            conn.execute(
                "ALTER TABLE timetable_slots ADD COLUMN batch_id TEXT NOT NULL DEFAULT ''",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        // Seed batch_id from existing grade+track+batch_pattern for rows that don't have one yet.
        conn.execute("
            UPDATE timetable_slots
            SET batch_id = grade_level || '|' || COALESCE(NULLIF(track,''),'Foundation') || '|' || batch_pattern
            WHERE batch_id = ''
        ", []).map_err(|e| e.to_string())?;
        // Also seed the batches table from the distinct combinations.
        conn.execute("
            INSERT OR IGNORE INTO batches (school_id, batch_id, grade_level, track, batch_pattern)
            SELECT DISTINCT school_id,
                   grade_level || '|' || COALESCE(NULLIF(track,''),'Foundation') || '|' || batch_pattern,
                   grade_level, track, batch_pattern
            FROM timetable_slots
        ", []).map_err(|e| e.to_string())?;
        // Create index on the new column.
        conn.execute("CREATE INDEX IF NOT EXISTS idx_timetable_slots_batch ON timetable_slots(school_id, batch_id)", [])
            .map_err(|e| e.to_string())?;
        conn.execute("INSERT INTO schema_migrations (version) VALUES (36)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 37: timetable_weekly_slots gets batch_id column.
    if !migration_applied(conn, 37)? {
        if !column_exists(conn, "timetable_weekly_slots", "batch_id")? {
            conn.execute(
                "ALTER TABLE timetable_weekly_slots ADD COLUMN batch_id TEXT NOT NULL DEFAULT ''",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        conn.execute("
            UPDATE timetable_weekly_slots
            SET batch_id = grade_level || '|' || COALESCE(NULLIF(track,''),'Foundation') || '|' || batch_pattern
            WHERE batch_id = ''
        ", []).map_err(|e| e.to_string())?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_timetable_weekly_batch ON timetable_weekly_slots(school_id, batch_id)", [])
            .map_err(|e| e.to_string())?;
        conn.execute("INSERT INTO schema_migrations (version) VALUES (37)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 38: add student contact / parent / batch columns.
    if !migration_applied(conn, 38)? {
        let cols = [
            ("student_mobile", "TEXT NOT NULL DEFAULT ''"),
            ("student_email", "TEXT NOT NULL DEFAULT ''"),
            ("father_name", "TEXT NOT NULL DEFAULT ''"),
            ("father_email", "TEXT NOT NULL DEFAULT ''"),
            ("father_mobile", "TEXT NOT NULL DEFAULT ''"),
            ("mother_name", "TEXT NOT NULL DEFAULT ''"),
            ("mother_email", "TEXT NOT NULL DEFAULT ''"),
            ("mother_mobile", "TEXT NOT NULL DEFAULT ''"),
            ("batch_id", "TEXT NOT NULL DEFAULT ''"),
        ];
        for (col, def) in &cols {
            if !column_exists(conn, "students", col)? {
                conn.execute(&format!("ALTER TABLE students ADD COLUMN {col} {def}"), [])
                    .map_err(|e| e.to_string())?;
            }
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (38)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 39: lecture_sessions gets ad-hoc columns for makeup classes.
    if !migration_applied(conn, 39)? {
        conn.execute_batch("
            CREATE TABLE lecture_sessions_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timetable_slot_id INTEGER,
                session_date TEXT NOT NULL,
                actual_faculty_user_id INTEGER,
                subject_id INTEGER,
                grade_level TEXT,
                track TEXT,
                school_id INTEGER,
                start_time TEXT,
                end_time TEXT,
                status TEXT NOT NULL DEFAULT 'Scheduled',
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(timetable_slot_id, session_date)
            );
            INSERT INTO lecture_sessions_new (id, timetable_slot_id, session_date, actual_faculty_user_id, status, created_at)
                SELECT id, timetable_slot_id, session_date, actual_faculty_user_id, status, created_at FROM lecture_sessions;
            DROP TABLE lecture_sessions;
            ALTER TABLE lecture_sessions_new RENAME TO lecture_sessions;
            CREATE INDEX IF NOT EXISTS idx_lecture_sessions_date ON lecture_sessions(session_date);
            CREATE INDEX IF NOT EXISTS idx_lecture_sessions_faculty ON lecture_sessions(actual_faculty_user_id);
            INSERT INTO schema_migrations (version) VALUES (39);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 40: Restore foreign keys on lecture_sessions after Migration 39 stripped them.
    if !migration_applied(conn, 40)? {
        conn.execute_batch("
            DELETE FROM lecture_sessions
            WHERE timetable_slot_id IS NOT NULL
              AND timetable_slot_id NOT IN (SELECT id FROM timetable_slots);
            DELETE FROM lecture_sessions
            WHERE actual_faculty_user_id IS NOT NULL
              AND actual_faculty_user_id NOT IN (SELECT id FROM users);
            DELETE FROM lecture_sessions
            WHERE subject_id IS NOT NULL
              AND subject_id NOT IN (SELECT id FROM subjects);
            DELETE FROM lecture_sessions
            WHERE school_id IS NOT NULL
              AND school_id NOT IN (SELECT id FROM schools);

            CREATE TABLE lecture_sessions_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timetable_slot_id INTEGER,
                session_date TEXT NOT NULL,
                actual_faculty_user_id INTEGER,
                subject_id INTEGER,
                grade_level TEXT,
                track TEXT,
                school_id INTEGER,
                start_time TEXT,
                end_time TEXT,
                status TEXT NOT NULL DEFAULT 'Scheduled',
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(timetable_slot_id, session_date),
                FOREIGN KEY(timetable_slot_id) REFERENCES timetable_slots(id) ON DELETE CASCADE,
                FOREIGN KEY(actual_faculty_user_id) REFERENCES users(id) ON DELETE SET NULL,
                FOREIGN KEY(subject_id) REFERENCES subjects(id) ON DELETE SET NULL,
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE
            );

            INSERT INTO lecture_sessions_new
                (id, timetable_slot_id, session_date, actual_faculty_user_id,
                 subject_id, grade_level, track, school_id, start_time, end_time, status, created_at)
            SELECT id, timetable_slot_id, session_date, actual_faculty_user_id,
                   subject_id, grade_level, track, school_id, start_time, end_time, status, created_at
            FROM lecture_sessions;

            DROP TABLE lecture_sessions;
            ALTER TABLE lecture_sessions_new RENAME TO lecture_sessions;

            CREATE INDEX IF NOT EXISTS idx_lecture_sessions_date    ON lecture_sessions(session_date);
            CREATE INDEX IF NOT EXISTS idx_lecture_sessions_faculty ON lecture_sessions(actual_faculty_user_id);
            CREATE INDEX IF NOT EXISTS idx_lecture_sessions_school  ON lecture_sessions(school_id);
            CREATE INDEX IF NOT EXISTS idx_lecture_sessions_subject ON lecture_sessions(subject_id);
            CREATE INDEX IF NOT EXISTS idx_lecture_sessions_slot    ON lecture_sessions(timetable_slot_id);

            INSERT INTO schema_migrations (version) VALUES (40);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 41: Add soft-delete support to timetable_slots
    if !migration_applied(conn, 41)? {
        if !column_exists(conn, "timetable_slots", "deleted_at")? {
            conn.execute("ALTER TABLE timetable_slots ADD COLUMN deleted_at TEXT", [])
                .map_err(|e| e.to_string())?;
            conn.execute("CREATE INDEX IF NOT EXISTS idx_timetable_slots_deleted ON timetable_slots(deleted_at) WHERE deleted_at IS NOT NULL", [])
                .map_err(|e| e.to_string())?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (41)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 42: Add room and session_type to timetable slots
    if !migration_applied(conn, 42)? {
        for (table, cols) in [
            (
                "timetable_slots",
                vec![
                    ("room", "TEXT NOT NULL DEFAULT ''"),
                    ("session_type", "TEXT NOT NULL DEFAULT 'Lecture'"),
                ],
            ),
            (
                "timetable_weekly_slots",
                vec![
                    ("room", "TEXT NOT NULL DEFAULT ''"),
                    ("session_type", "TEXT NOT NULL DEFAULT 'Lecture'"),
                ],
            ),
        ] {
            for (col, def) in cols {
                if !column_exists(conn, table, col)? {
                    conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {col} {def}"), [])
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        conn.execute_batch("
            CREATE INDEX IF NOT EXISTS idx_timetable_slots_room ON timetable_slots(room) WHERE room != '';
            CREATE INDEX IF NOT EXISTS idx_timetable_slots_session_type ON timetable_slots(session_type);
            CREATE INDEX IF NOT EXISTS idx_timetable_weekly_room ON timetable_weekly_slots(room) WHERE room != '';
            CREATE INDEX IF NOT EXISTS idx_timetable_weekly_session_type ON timetable_weekly_slots(session_type);
            INSERT INTO schema_migrations (version) VALUES (42);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 43: ticket-timetable linking metadata + CHECK constraint on session_type + composite indexes
    if !migration_applied(conn, 43)? {
        // 1. Ticket linking columns (forward-migration for DBs where duplicate 42 skipped this)
        for (col, def) in [
            ("linked_grade_level", "TEXT NOT NULL DEFAULT ''"),
            ("linked_subject", "TEXT NOT NULL DEFAULT ''"),
        ] {
            if !column_exists(conn, "tickets", col)? {
                conn.execute(&format!("ALTER TABLE tickets ADD COLUMN {col} {def}"), [])
                    .map_err(|e| e.to_string())?;
            }
        }

        // 2. Add CHECK constraint on session_type via recreate-and-swap for both tables
        let _fk_before: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap_or(1);
        conn.execute("PRAGMA foreign_keys = OFF", [])
            .map_err(|e| e.to_string())?;

        conn.execute_batch("
            CREATE TABLE timetable_slots_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                school_id INTEGER NOT NULL,
                grade_level TEXT NOT NULL,
                track TEXT NOT NULL DEFAULT '',
                batch_pattern TEXT NOT NULL,
                batch_id TEXT NOT NULL DEFAULT '',
                day_of_week INTEGER NOT NULL CHECK(day_of_week >= 0 AND day_of_week <= 6),
                period INTEGER NOT NULL CHECK(period >= 1),
                subject_id INTEGER NOT NULL,
                faculty_user_id INTEGER,
                start_time TEXT NOT NULL DEFAULT '',
                end_time TEXT NOT NULL DEFAULT '',
                room TEXT NOT NULL DEFAULT '',
                session_type TEXT NOT NULL DEFAULT 'Lecture' CHECK(session_type IN ('Lecture','Tutorial','Activity','Assessment','Remedial')),
                deleted_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(school_id, grade_level, track, batch_pattern, day_of_week, period),
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE,
                FOREIGN KEY(subject_id) REFERENCES subjects(id) ON DELETE RESTRICT,
                FOREIGN KEY(faculty_user_id) REFERENCES users(id) ON DELETE SET NULL
            );
            INSERT INTO timetable_slots_new
                (id, school_id, grade_level, track, batch_pattern, batch_id, day_of_week, period, subject_id, faculty_user_id, start_time, end_time, room, session_type, deleted_at, created_at, updated_at)
            SELECT id, school_id, grade_level, track, batch_pattern, batch_id, day_of_week, period, subject_id, faculty_user_id, start_time, end_time, room, session_type, deleted_at, created_at, updated_at
            FROM timetable_slots;
            DROP TABLE timetable_slots;
            ALTER TABLE timetable_slots_new RENAME TO timetable_slots;

            CREATE TABLE timetable_weekly_slots_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                school_id INTEGER NOT NULL,
                grade_level TEXT NOT NULL,
                track TEXT NOT NULL DEFAULT '',
                batch_pattern TEXT NOT NULL,
                batch_id TEXT NOT NULL DEFAULT '',
                day_of_week INTEGER NOT NULL CHECK(day_of_week >= 0 AND day_of_week <= 6),
                period INTEGER NOT NULL CHECK(period >= 1),
                subject_id INTEGER NOT NULL,
                faculty_user_id INTEGER,
                start_time TEXT NOT NULL DEFAULT '',
                end_time TEXT NOT NULL DEFAULT '',
                week_start_date TEXT NOT NULL,
                room TEXT NOT NULL DEFAULT '',
                session_type TEXT NOT NULL DEFAULT 'Lecture' CHECK(session_type IN ('Lecture','Tutorial','Activity','Assessment','Remedial')),
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(school_id, grade_level, track, batch_pattern, week_start_date, day_of_week, period),
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE,
                FOREIGN KEY(subject_id) REFERENCES subjects(id) ON DELETE RESTRICT,
                FOREIGN KEY(faculty_user_id) REFERENCES users(id) ON DELETE SET NULL
            );
            INSERT INTO timetable_weekly_slots_new
                (id, school_id, grade_level, track, batch_pattern, batch_id, day_of_week, period, subject_id, faculty_user_id, start_time, end_time, week_start_date, room, session_type, created_at, updated_at)
            SELECT id, school_id, grade_level, track, batch_pattern, batch_id, day_of_week, period, subject_id, faculty_user_id, start_time, end_time, week_start_date, room, session_type, created_at, updated_at
            FROM timetable_weekly_slots;
            DROP TABLE timetable_weekly_slots;
            ALTER TABLE timetable_weekly_slots_new RENAME TO timetable_weekly_slots;

            CREATE INDEX IF NOT EXISTS idx_timetable_weekly_school_week ON timetable_weekly_slots(school_id, week_start_date);
            CREATE INDEX IF NOT EXISTS idx_timetable_slots_batch ON timetable_slots(school_id, batch_id);
            CREATE INDEX IF NOT EXISTS idx_timetable_weekly_batch ON timetable_weekly_slots(school_id, batch_id);
            CREATE INDEX IF NOT EXISTS idx_timetable_slots_deleted ON timetable_slots(deleted_at) WHERE deleted_at IS NOT NULL;

            CREATE INDEX IF NOT EXISTS idx_timetable_weekly_faculty_week ON timetable_weekly_slots(faculty_user_id, week_start_date);
            CREATE INDEX IF NOT EXISTS idx_timetable_weekly_room_conflicts ON timetable_weekly_slots(school_id, week_start_date, day_of_week, room);
            CREATE INDEX IF NOT EXISTS idx_timetable_slots_compliance ON timetable_slots(school_id, grade_level, track, subject_id);

            INSERT INTO schema_migrations (version) VALUES (43);
        ").map_err(|e| e.to_string())?;

        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 44: leave_requests table + link to lecture_sessions
    if !migration_applied(conn, 44)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS leave_requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                faculty_user_id INTEGER NOT NULL,
                school_id INTEGER NOT NULL,
                start_date TEXT NOT NULL,
                end_date TEXT NOT NULL,
                reason TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'Pending',
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                FOREIGN KEY(faculty_user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY(school_id) REFERENCES schools(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_leave_requests_faculty ON leave_requests(faculty_user_id);
            CREATE INDEX IF NOT EXISTS idx_leave_requests_school ON leave_requests(school_id);
            ALTER TABLE lecture_sessions ADD COLUMN leave_request_id INTEGER;
            INSERT INTO schema_migrations (version) VALUES (44);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 45: swap_requests table
    if !migration_applied(conn, 45)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS swap_requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                requester_faculty_id INTEGER NOT NULL,
                recipient_faculty_id INTEGER NOT NULL,
                slot_a_id INTEGER NOT NULL,
                slot_b_id INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'Pending',
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                FOREIGN KEY(requester_faculty_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY(recipient_faculty_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY(slot_a_id) REFERENCES timetable_slots(id) ON DELETE CASCADE,
                FOREIGN KEY(slot_b_id) REFERENCES timetable_slots(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_swap_requests_requester ON swap_requests(requester_faculty_id);
            CREATE INDEX IF NOT EXISTS idx_swap_requests_recipient ON swap_requests(recipient_faculty_id);
            INSERT INTO schema_migrations (version) VALUES (45);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 46: substitution_suggestions cache + faculty_substitution_balance
    if !migration_applied(conn, 46)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS substitution_suggestions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                candidate_faculty_id INTEGER NOT NULL,
                subject_match INTEGER NOT NULL DEFAULT 0,
                free_period INTEGER NOT NULL DEFAULT 0,
                same_school INTEGER NOT NULL DEFAULT 0,
                workload_score INTEGER NOT NULL DEFAULT 0,
                overall_score INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                FOREIGN KEY(session_id) REFERENCES lecture_sessions(id) ON DELETE CASCADE,
                FOREIGN KEY(candidate_faculty_id) REFERENCES users(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_substitution_suggestions_session ON substitution_suggestions(session_id);
            CREATE TABLE IF NOT EXISTS faculty_substitution_balance (
                faculty_user_id INTEGER PRIMARY KEY,
                given_count INTEGER NOT NULL DEFAULT 0,
                received_count INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                FOREIGN KEY(faculty_user_id) REFERENCES users(id) ON DELETE CASCADE
            );
            INSERT INTO schema_migrations (version) VALUES (46);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 47: notification_log — Phase 6 mobile & notifications layer
    run_migration(conn, 47, "
        CREATE TABLE IF NOT EXISTS notification_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            type TEXT NOT NULL DEFAULT 'info',
            title TEXT NOT NULL,
            message TEXT NOT NULL,
            payload_json TEXT NOT NULL DEFAULT '',
            read_at TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_notification_user ON notification_log(user_id);
        CREATE INDEX IF NOT EXISTS idx_notification_read ON notification_log(read_at) WHERE read_at = '';
        CREATE INDEX IF NOT EXISTS idx_notification_created ON notification_log(created_at);
    ")?;

    // Migration 48: building + room_location — Phase 6 room map support
    if !migration_applied(conn, 48)? {
        if !column_exists(conn, "schools", "building")? {
            conn.execute(
                "ALTER TABLE schools ADD COLUMN building TEXT NOT NULL DEFAULT ''",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        if !column_exists(conn, "timetable_slots", "room_location")? {
            conn.execute(
                "ALTER TABLE timetable_slots ADD COLUMN room_location TEXT NOT NULL DEFAULT ''",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        if !column_exists(conn, "timetable_weekly_slots", "room_location")? {
            conn.execute("ALTER TABLE timetable_weekly_slots ADD COLUMN room_location TEXT NOT NULL DEFAULT ''", [])
                .map_err(|e| e.to_string())?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (48)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 49: weekly_health_snapshots — historical trajectory for 8-week trends
    if !migration_applied(conn, 49)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS weekly_health_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                week_start_date TEXT NOT NULL,
                school_id INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'Green',
                gaps_count INTEGER NOT NULL DEFAULT 0,
                snapshot_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );
            CREATE INDEX IF NOT EXISTS idx_health_snapshots_week_school ON weekly_health_snapshots(week_start_date, school_id);
            CREATE INDEX IF NOT EXISTS idx_health_snapshots_school_week ON weekly_health_snapshots(school_id, week_start_date);
            INSERT INTO schema_migrations (version) VALUES (49);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 50: weekly_compliance_snapshots — per-school, per-subject weekly adherence
    if !migration_applied(conn, 50)? {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS weekly_compliance_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                week_start_date TEXT NOT NULL,
                school_id INTEGER NOT NULL,
                subject_id INTEGER NOT NULL,
                planned_periods INTEGER NOT NULL DEFAULT 0,
                actual_periods INTEGER NOT NULL DEFAULT 0,
                adherence_pct REAL NOT NULL DEFAULT 100.0,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );
            CREATE INDEX IF NOT EXISTS idx_compliance_snapshots_week_school ON weekly_compliance_snapshots(week_start_date, school_id);
            CREATE INDEX IF NOT EXISTS idx_compliance_snapshots_school_subject ON weekly_compliance_snapshots(school_id, subject_id);
            INSERT INTO schema_migrations (version) VALUES (50);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 51: Analytics performance indexes on existing tables
    if !migration_applied(conn, 51)? {
        conn.execute_batch("
            CREATE INDEX IF NOT EXISTS idx_lecture_sessions_date_status ON lecture_sessions(session_date, status);
            CREATE INDEX IF NOT EXISTS idx_timetable_weekly_school_date ON timetable_weekly_slots(school_id, week_start_date);
            CREATE INDEX IF NOT EXISTS idx_timetable_weekly_faculty_date ON timetable_weekly_slots(faculty_user_id, week_start_date);
            CREATE INDEX IF NOT EXISTS idx_timetable_slots_school_subject ON timetable_slots(school_id, subject_id);
            CREATE INDEX IF NOT EXISTS idx_timetable_slots_faculty ON timetable_slots(faculty_user_id);
            CREATE INDEX IF NOT EXISTS idx_schools_region ON schools(region_id);
            INSERT INTO schema_migrations (version) VALUES (51);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 52: central_policies — configurable thresholds & mandates
    run_migration(
        conn,
        52,
        "
        CREATE TABLE IF NOT EXISTS central_policies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key TEXT NOT NULL UNIQUE,
            value TEXT NOT NULL,
            region_id INTEGER,
            updated_at TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_central_policies_key ON central_policies(key);
        CREATE INDEX IF NOT EXISTS idx_central_policies_region ON central_policies(region_id);
        INSERT OR IGNORE INTO central_policies (key, value, updated_at) VALUES
            ('max_periods_per_faculty', '24', datetime('now')),
            ('attendance_marking_deadline', '11:00', datetime('now'));
    ",
    )?;

    // Migration 53: escalation_rules — multi-rule smart escalation engine
    run_migration(
        conn,
        53,
        "
        CREATE TABLE IF NOT EXISTS escalation_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            conditions_json TEXT NOT NULL DEFAULT '{}',
            action TEXT NOT NULL DEFAULT 'escalate',
            assignee_role TEXT NOT NULL DEFAULT '',
            hours_threshold INTEGER NOT NULL DEFAULT 24,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_escalation_rules_active ON escalation_rules(is_active);
    ",
    )?;

    // Migration 54: alert_states + announcements
    run_migration(conn, 54, "
        CREATE TABLE IF NOT EXISTS alert_states (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            alert_hash TEXT NOT NULL,
            user_id INTEGER NOT NULL,
            dismissed_at TEXT NOT NULL DEFAULT '',
            snoozed_until TEXT NOT NULL DEFAULT '',
            converted_ticket_id INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            UNIQUE(alert_hash, user_id)
        );
        CREATE INDEX IF NOT EXISTS idx_alert_states_user ON alert_states(user_id);
        CREATE INDEX IF NOT EXISTS idx_alert_states_hash ON alert_states(alert_hash);

        CREATE TABLE IF NOT EXISTS announcements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            school_id INTEGER,
            message TEXT NOT NULL,
            pinned_until TEXT NOT NULL DEFAULT '',
            created_by INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_announcements_school ON announcements(school_id);
        CREATE INDEX IF NOT EXISTS idx_announcements_pinned ON announcements(pinned_until) WHERE pinned_until != '';
    ")?;

    // Migration 55: bulk_operation_log
    run_migration(
        conn,
        55,
        "
        CREATE TABLE IF NOT EXISTS bulk_operation_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            payload_json TEXT NOT NULL DEFAULT '{}',
            result_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            completed_at TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_bulk_log_status ON bulk_operation_log(status);
        CREATE INDEX IF NOT EXISTS idx_bulk_log_type ON bulk_operation_log(type);
    ",
    )?;

    // Migration 56: leave request approval metadata
    if !migration_applied(conn, 56)? {
        conn.execute(
            "ALTER TABLE leave_requests ADD COLUMN approved_by_user_id INTEGER",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "ALTER TABLE leave_requests ADD COLUMN approved_at TEXT NOT NULL DEFAULT ''",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "ALTER TABLE leave_requests ADD COLUMN rejected_by_user_id INTEGER",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "ALTER TABLE leave_requests ADD COLUMN rejected_at TEXT NOT NULL DEFAULT ''",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "ALTER TABLE leave_requests ADD COLUMN rejection_reason TEXT NOT NULL DEFAULT ''",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute("INSERT INTO schema_migrations (version) VALUES (56)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 57: leave_request_audit_log
    run_migration(conn, 57, "
        CREATE TABLE IF NOT EXISTS leave_request_audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            leave_request_id INTEGER NOT NULL,
            actor_user_id INTEGER NOT NULL,
            action TEXT NOT NULL,
            old_status TEXT NOT NULL DEFAULT '',
            new_status TEXT NOT NULL DEFAULT '',
            reason TEXT NOT NULL DEFAULT '',
            school_id INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_leave_audit_leave_id ON leave_request_audit_log(leave_request_id);
        CREATE INDEX IF NOT EXISTS idx_leave_audit_actor ON leave_request_audit_log(actor_user_id);
        CREATE INDEX IF NOT EXISTS idx_leave_audit_school ON leave_request_audit_log(school_id);
    ")?;

    // Migration 58: vp_tagging on schools
    if !migration_applied(conn, 58)? {
        if !column_exists(conn, "schools", "vp_tagging")? {
            conn.execute(
                "ALTER TABLE schools ADD COLUMN vp_tagging TEXT NOT NULL DEFAULT ''",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (58)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 59: VP Centers and Buildings
    run_migration(
        conn,
        59,
        "
        CREATE TABLE IF NOT EXISTS vp_centers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            location TEXT NOT NULL DEFAULT '',
            contact_name TEXT NOT NULL DEFAULT '',
            contact_mobile TEXT NOT NULL DEFAULT '',
            contact_email TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE TABLE IF NOT EXISTS vp_center_buildings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            vp_center_id INTEGER NOT NULL REFERENCES vp_centers(id) ON DELETE CASCADE,
            building_name TEXT NOT NULL DEFAULT '',
            address TEXT NOT NULL DEFAULT '',
            center_head_name TEXT NOT NULL DEFAULT '',
            center_head_mobile TEXT NOT NULL DEFAULT '',
            center_head_email TEXT NOT NULL DEFAULT '',
            associate_center_head_name TEXT NOT NULL DEFAULT '',
            associate_center_head_mobile TEXT NOT NULL DEFAULT '',
            associate_center_head_email TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
    ",
    )?;
    seed_vp_centers(conn)?;

    // Migration 60: Faculty Profiles, Wings, Batches
    run_migration(
        conn,
        60,
        "
        CREATE TABLE IF NOT EXISTS faculty_profiles (
            faculty_user_id INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
            pwid TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '',
            mobile TEXT NOT NULL DEFAULT '',
            emergency_contact_name TEXT NOT NULL DEFAULT '',
            emergency_contact_mobile TEXT NOT NULL DEFAULT '',
            vp_center_id INTEGER REFERENCES vp_centers(id),
            sip_school_id INTEGER REFERENCES schools(id),
            primary_subject_id INTEGER REFERENCES subjects(id),
            employment_type TEXT NOT NULL DEFAULT 'VP Payroll',
            qualification TEXT NOT NULL DEFAULT '',
            experience_years INTEGER NOT NULL DEFAULT 0,
            designation TEXT NOT NULL DEFAULT '',
            specialization TEXT NOT NULL DEFAULT '',
            max_periods_per_week INTEGER NOT NULL DEFAULT 24,
            joining_date TEXT NOT NULL DEFAULT '',
            exit_date TEXT NOT NULL DEFAULT '',
            documents_verified INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE TABLE IF NOT EXISTS faculty_wings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            faculty_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            wing TEXT NOT NULL,
            UNIQUE(faculty_user_id, wing)
        );
        CREATE TABLE IF NOT EXISTS faculty_batches (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            faculty_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            batch_id INTEGER NOT NULL REFERENCES batches(id) ON DELETE CASCADE,
            UNIQUE(faculty_user_id, batch_id)
        );
    ",
    )?;

    seed_communication_templates(conn)?;
    seed_lecture_models(conn)?;
    seed_faculty_timetable(conn)?;

    // Migration 61: Faculty Members (master data, optional login account)
    run_migration(conn, 61, "
        CREATE TABLE IF NOT EXISTS faculty_members (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '',
            mobile TEXT NOT NULL DEFAULT '',
            pwid TEXT NOT NULL DEFAULT '',
            qualification TEXT NOT NULL DEFAULT '',
            experience_years INTEGER NOT NULL DEFAULT 0,
            designation TEXT NOT NULL DEFAULT '',
            specialization TEXT NOT NULL DEFAULT '',
            employment_type TEXT NOT NULL DEFAULT 'VP Payroll',
            is_active INTEGER NOT NULL DEFAULT 1,
            user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_faculty_members_user ON faculty_members(user_id);
        CREATE INDEX IF NOT EXISTS idx_faculty_members_active ON faculty_members(is_active);

        CREATE TABLE IF NOT EXISTS faculty_school_memberships (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            faculty_id INTEGER NOT NULL REFERENCES faculty_members(id) ON DELETE CASCADE,
            school_id INTEGER NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
            role_at_school TEXT NOT NULL DEFAULT 'Faculty',
            is_primary INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            UNIQUE(faculty_id, school_id)
        );
        CREATE INDEX IF NOT EXISTS idx_faculty_school_memberships_faculty ON faculty_school_memberships(faculty_id);
        CREATE INDEX IF NOT EXISTS idx_faculty_school_memberships_school ON faculty_school_memberships(school_id);

        -- Backfill existing faculty users into faculty_members
        INSERT INTO faculty_members (name, email, mobile, user_id, is_active)
        SELECT display_name, '', '', id, is_active
        FROM users WHERE role = 'faculty';

        -- Backfill school memberships from user_schools for faculty users
        INSERT INTO faculty_school_memberships (faculty_id, school_id, role_at_school, is_primary)
        SELECT fm.id, us.school_id, 'Faculty', 1
        FROM faculty_members fm
        JOIN users u ON fm.user_id = u.id
        JOIN user_schools us ON u.id = us.user_id;
    ")?;

    // Migration 62: Faculty Members — unique user_id linkage + archive behavior
    run_migration(
        conn,
        62,
        "
        CREATE UNIQUE INDEX IF NOT EXISTS idx_faculty_members_unique_user
        ON faculty_members(user_id) WHERE user_id IS NOT NULL;
    ",
    )?;

    // Migration 63: Faculty Assignments — primary reference becomes faculty_id
    if !migration_applied(conn, 63)? {
        // Ensure every user referenced by faculty_assignments has a faculty_members row
        conn.execute(
            "
            INSERT OR IGNORE INTO faculty_members (name, email, mobile, user_id, is_active)
            SELECT u.display_name, '', '', u.id, u.is_active
            FROM users u
            WHERE u.id IN (SELECT DISTINCT faculty_user_id FROM faculty_assignments)
              AND NOT EXISTS (SELECT 1 FROM faculty_members fm WHERE fm.user_id = u.id)
        ",
            [],
        )
        .map_err(|e| e.to_string())?;

        conn.execute_batch("
            CREATE TABLE faculty_assignments_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                faculty_id INTEGER NOT NULL REFERENCES faculty_members(id) ON DELETE CASCADE,
                faculty_user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
                school_id INTEGER NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
                grade_level TEXT NOT NULL,
                track TEXT NOT NULL DEFAULT '',
                subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE RESTRICT,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(faculty_id, school_id, grade_level, track, subject_id)
            );

            INSERT INTO faculty_assignments_new
                (id, faculty_id, faculty_user_id, school_id, grade_level, track, subject_id, created_at)
            SELECT
                fa.id,
                fm.id,
                fa.faculty_user_id,
                fa.school_id,
                fa.grade_level,
                fa.track,
                fa.subject_id,
                fa.created_at
            FROM faculty_assignments fa
            JOIN faculty_members fm ON fm.user_id = fa.faculty_user_id;

            DROP TABLE faculty_assignments;
            ALTER TABLE faculty_assignments_new RENAME TO faculty_assignments;

            CREATE INDEX idx_faculty_assignments_faculty ON faculty_assignments(faculty_id);
            CREATE INDEX idx_faculty_assignments_school ON faculty_assignments(school_id);

            INSERT INTO schema_migrations (version) VALUES (63);
        ").map_err(|e| e.to_string())?;
    }

    // Migration 64: Normalize faculty membership role label
    run_migration(
        conn,
        64,
        "
        UPDATE faculty_school_memberships
        SET role_at_school = 'Faculty'
        WHERE role_at_school = 'Teacher';
    ",
    )?;

    // Migration 65: Batches — soft archive support
    run_migration(
        conn,
        65,
        "
        ALTER TABLE batches ADD COLUMN deleted_at TEXT NOT NULL DEFAULT '';
        ",
    )?;

    // Migration 66: Faculty assignments target concrete batches
    if !migration_applied(conn, 66)? {
        conn.execute(
            "
            INSERT OR IGNORE INTO batches (school_id, batch_id, grade_level, track, batch_pattern, capacity)
            SELECT DISTINCT
                fa.school_id,
                fa.grade_level || '|' || COALESCE(NULLIF(fa.track,''),'Foundation') || '|Weekday',
                fa.grade_level,
                fa.track,
                'Weekday',
                0
            FROM faculty_assignments fa
            WHERE NOT EXISTS (
                SELECT 1
                FROM batches b
                WHERE b.school_id = fa.school_id
                  AND b.grade_level = fa.grade_level
                  AND b.track = fa.track
                  AND b.deleted_at = ''
            )
            ",
            [],
        )
        .map_err(|e| e.to_string())?;

        conn.execute_batch(
            "
            CREATE TABLE faculty_assignments_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                faculty_id INTEGER NOT NULL REFERENCES faculty_members(id) ON DELETE CASCADE,
                faculty_user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
                school_id INTEGER NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
                batch_id INTEGER NOT NULL REFERENCES batches(id) ON DELETE RESTRICT,
                grade_level TEXT NOT NULL,
                track TEXT NOT NULL DEFAULT '',
                subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE RESTRICT,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(faculty_id, batch_id, subject_id)
            );

            INSERT OR IGNORE INTO faculty_assignments_new
                (id, faculty_id, faculty_user_id, school_id, batch_id, grade_level, track, subject_id, created_at)
            SELECT
                fa.id,
                fa.faculty_id,
                fa.faculty_user_id,
                fa.school_id,
                (
                    SELECT b.id
                    FROM batches b
                    WHERE b.school_id = fa.school_id
                      AND b.grade_level = fa.grade_level
                      AND b.track = fa.track
                      AND b.deleted_at = ''
                    ORDER BY b.id
                    LIMIT 1
                ),
                fa.grade_level,
                fa.track,
                fa.subject_id,
                fa.created_at
            FROM faculty_assignments fa;

            DROP TABLE faculty_assignments;
            ALTER TABLE faculty_assignments_new RENAME TO faculty_assignments;

            CREATE INDEX idx_faculty_assignments_faculty ON faculty_assignments(faculty_id);
            CREATE INDEX idx_faculty_assignments_school ON faculty_assignments(school_id);
            CREATE INDEX idx_faculty_assignments_batch ON faculty_assignments(batch_id);

            INSERT INTO schema_migrations (version) VALUES (66);
            ",
        )
        .map_err(|e| e.to_string())?;
    }

    // Migration 67: Timetable slots reference concrete batches
    if !migration_applied(conn, 67)? {
        if !column_exists(conn, "timetable_slots", "batch_ref_id")? {
            conn.execute(
                "ALTER TABLE timetable_slots ADD COLUMN batch_ref_id INTEGER NOT NULL DEFAULT 0",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        conn.execute(
            "
            UPDATE timetable_slots
            SET batch_ref_id = COALESCE(
                (
                    SELECT b.id
                    FROM batches b
                    WHERE b.school_id = timetable_slots.school_id
                      AND b.batch_id = timetable_slots.batch_id
                      AND b.deleted_at = ''
                    LIMIT 1
                ),
                (
                    SELECT b.id
                    FROM batches b
                    WHERE b.school_id = timetable_slots.school_id
                      AND b.grade_level = timetable_slots.grade_level
                      AND b.track = timetable_slots.track
                      AND b.batch_pattern = timetable_slots.batch_pattern
                      AND b.deleted_at = ''
                    ORDER BY b.id
                    LIMIT 1
                ),
                0
            )
            WHERE batch_ref_id = 0
            ",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timetable_slots_batch_ref ON timetable_slots(batch_ref_id)",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute("INSERT INTO schema_migrations (version) VALUES (67)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 68: Students reference concrete batches
    if !migration_applied(conn, 68)? {
        if !column_exists(conn, "students", "batch_ref_id")? {
            conn.execute(
                "ALTER TABLE students ADD COLUMN batch_ref_id INTEGER NOT NULL DEFAULT 0",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        conn.execute(
            "
            INSERT OR IGNORE INTO batches (school_id, batch_id, grade_level, track, batch_pattern, capacity)
            SELECT DISTINCT
                school_id,
                CASE
                    WHEN batch_id != '' THEN batch_id
                    ELSE grade_level || '|' || COALESCE(NULLIF(track,''),'Foundation') || '|Weekday'
                END,
                grade_level,
                track,
                'Weekday',
                0
            FROM students
            WHERE grade_level != ''
            ",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "
            UPDATE students
            SET batch_ref_id = COALESCE(
                (
                    SELECT b.id
                    FROM batches b
                    WHERE b.school_id = students.school_id
                      AND b.batch_id = students.batch_id
                      AND b.deleted_at = ''
                    LIMIT 1
                ),
                (
                    SELECT b.id
                    FROM batches b
                    WHERE b.school_id = students.school_id
                      AND b.grade_level = students.grade_level
                      AND b.track = students.track
                      AND b.deleted_at = ''
                    ORDER BY b.id
                    LIMIT 1
                ),
                0
            )
            WHERE batch_ref_id = 0
            ",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_students_batch_ref ON students(batch_ref_id)",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute("INSERT INTO schema_migrations (version) VALUES (68)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 69: Holidays grade-level scoping
    if !migration_applied(conn, 69)? {
        if !column_exists(conn, "holidays", "grade_level")? {
            conn.execute(
                "ALTER TABLE holidays ADD COLUMN grade_level TEXT",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_holidays_grade ON holidays(grade_level)",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute("INSERT INTO schema_migrations (version) VALUES (69)", [])
            .map_err(|e| e.to_string())?;
    }

    // Migration 70: Deputy Regional Academic Head on regions
    if !migration_applied(conn, 70)? {
        for col in [
            "regional_deputy_academic_head_name",
            "regional_deputy_academic_head_mobile",
            "regional_deputy_academic_head_email",
        ] {
            if !column_exists(conn, "regions", col)? {
                conn.execute(
                    &format!("ALTER TABLE regions ADD COLUMN {col} TEXT NOT NULL DEFAULT ''"),
                    [],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (70)", [])
            .map_err(|e| e.to_string())?;
    }

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
        (
            "Green Valley Public School",
            "Aarav Shah",
            "Grade 11",
            "JEE Foundation",
        ),
        (
            "North City Senior Secondary",
            "Nisha Rao",
            "Grade 10",
            "NEET Foundation",
        ),
        (
            "Sunrise International School",
            "Rohan Iyer",
            "Grade 9",
            "Integrated STEM",
        ),
    ];
    for (school, student, grade, track) in students {
        conn.execute(
            "INSERT INTO schools (name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
            rusqlite::params![school],
        )
        .map_err(|e| e.to_string())?;
        let school_id: i64 = conn
            .query_row(
                "SELECT id FROM schools WHERE name = ?1",
                rusqlite::params![school],
                |row| row.get(0),
            )
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

fn seed_vp_centers(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "
        INSERT OR IGNORE INTO vp_centers (name)
        SELECT DISTINCT mapped_vp_center FROM schools WHERE mapped_vp_center <> ''
    ",
        [],
    )
    .map_err(|e| e.to_string())?;
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
        (
            "VPN access request for finance team",
            "Finance team members need VPN access before month-end close.",
            "Meera Shah",
            "Unassigned",
            "Open",
            "High",
            "Learning Platform",
            "Green Valley Public School",
            "Aarav Shah",
            "Grade 11",
            "JEE Foundation",
            "Academic Support",
            "2026-04-23 09:30:00",
            "None",
            "",
        ),
        (
            "Printer queue stuck on third floor",
            "The shared printer queue is holding jobs and preventing printing.",
            "Arjun Rao",
            "Nina Patel",
            "In Progress",
            "Medium",
            "Operations",
            "North City Senior Secondary",
            "Nisha Rao",
            "Grade 10",
            "NEET Foundation",
            "Operations",
            "2026-04-24 10:15:00",
            "None",
            "",
        ),
        (
            "Laptop replacement approval needed",
            "The current laptop has intermittent power failures.",
            "Dev Iyer",
            "Sam Thomas",
            "Pending",
            "Low",
            "IT / Device",
            "Sunrise International School",
            "Rohan Iyer",
            "Grade 9",
            "Integrated STEM",
            "Device",
            "2026-04-24 11:00:00",
            "None",
            "",
        ),
    ];
    for (
        title,
        description,
        requester,
        assignee,
        status,
        priority,
        queue,
        school_name,
        student_name,
        grade_level,
        program_track,
        issue_category,
        sla_due_at,
        escalation_status,
        escalated_at,
    ) in samples
    {
        conn.execute(
            "INSERT INTO tickets (title, description, requester, assignee, status, priority, queue, school_id, school_name, student_name, grade_level, program_track, issue_category, sla_due_at, escalation_status, escalated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, (SELECT id FROM schools WHERE name = ?8 LIMIT 1), ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![title, description, requester, assignee, status, priority, queue, school_name, student_name, grade_level, program_track, issue_category, sla_due_at, escalation_status, escalated_at],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_faculty_timetable(conn: &Connection) -> Result<(), String> {
    // Faculty assignments: link each test faculty to one subject at their school.
    // Idempotent via UNIQUE constraint on (faculty_user_id, school_id, grade_level, track, subject_id).
    let assignments = [
        // (faculty_username, school_id, grade_level, track, subject_name, subject_track)
        ("faculty1", 2i64, "Grade 10", "", "Chemistry", "Foundation"),
        ("faculty2", 1i64, "Grade 11", "", "Physics", "Foundation"),
        ("faculty3", 3i64, "Grade 9", "", "Biology", "Foundation"),
    ];
    for (faculty_username, school_id, grade_level, track, subject_name, subject_track) in
        assignments
    {
        let faculty_user_id: i64 = match conn
            .query_row(
                "SELECT id FROM users WHERE username = ?1",
                rusqlite::params![faculty_username],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
        {
            Some(id) => id,
            None => continue, // test users not seeded in non-TEST_SEED contexts
        };
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE name = ?1 AND track = ?2",
                rusqlite::params![subject_name, subject_track],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO faculty_assignments (faculty_user_id, school_id, grade_level, track, subject_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![faculty_user_id, school_id, grade_level, track, subject_id],
        ).map_err(|e| e.to_string())?;

        // Create recurring timetable slots: 2 periods × every weekday (days 0–4)
        for day in [0i64, 1, 2, 3, 4] {
            for period in [1i64, 2] {
                conn.execute(
                    "INSERT OR IGNORE INTO timetable_slots (school_id, grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id, start_time, end_time) VALUES (?1, ?2, ?3, 'Weekday', ?4, ?5, ?6, ?7, '', '')",
                    rusqlite::params![school_id, grade_level, track, day, period, subject_id, faculty_user_id],
                ).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

fn seed_test_users(conn: &Connection) -> Result<(), String> {
    let test_users = [
        ("aom1", "AOM One", "aom", "aom123"),
        ("aom2", "AOM Two", "aom", "aom123"),
        ("aom3", "AOM Three", "aom", "aom123"),
        ("faculty1", "Faculty One", "faculty", "faculty123"),
        ("faculty2", "Faculty Two", "faculty", "faculty123"),
        ("faculty3", "Faculty Three", "faculty", "faculty123"),
        ("head1", "Head One", "head", "head123"),
        ("viewer1", "Viewer One", "viewer", "viewer123"),
    ];
    for (username, display_name, role, password) in test_users {
        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO users (username, display_name, role, password_hash) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![username, display_name, role, hash],
        ).map_err(|e| e.to_string())?;
    }
    // Assign scoped users to schools
    let mappings = [
        ("aom1", 1i64),     // Green Valley Public School
        ("aom2", 2i64),     // North City Senior Secondary
        ("aom3", 3i64),     // Sunrise International School
        ("faculty1", 2i64), // North City Senior Secondary
        ("faculty2", 1i64), // Green Valley Public School
        ("faculty3", 3i64), // Sunrise International School
        ("head1", 1i64),    // Green Valley Public School
    ];
    for (username, school_id) in mappings {
        let user_id: i64 = conn
            .query_row(
                "SELECT id FROM users WHERE username = ?1",
                rusqlite::params![username],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO user_schools (user_id, school_id) VALUES (?1, ?2)",
            rusqlite::params![user_id, school_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
