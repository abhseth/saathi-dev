#!/usr/bin/env python3
"""Seed sample data for DAS (Daily Attendance Summary) calculation testing."""

import sqlite3
import random
from datetime import date

DB_PATH = "tickets.sqlite3"
TODAY = str(date.today())  # 2026-05-06

# Indian first and last names for realistic sample data
FIRST_NAMES = [
    "Aarav", "Vivaan", "Aditya", "Vihaan", "Arjun", "Sai", "Arnav", "Ayaan",
    "Krishna", "Ishaan", "Shaurya", "Atharv", "Darsh", "Aryan", "Ansh",
    "Diya", "Saanvi", "Ananya", "Aadhya", "Kiara", "Sara", "Myra", "Aria",
    "Navya", "Ira", "Disha", "Kavya", "Siya", "Pari", "Riya",
]
LAST_NAMES = [
    "Sharma", "Gupta", "Patel", "Verma", "Rao", "Iyer", "Shah", "Mehta",
    "Kumar", "Singh", "Reddy", "Nair", "Joshi", "Desai", "Pillai",
]

ATTENDANCE_STATUSES = ["Present", "Absent", "Late", "Excused", "Leave", "Out of Class"]
# Weighted probabilities for realistic attendance
STATUS_WEIGHTS = [70, 12, 10, 5, 2, 1]


def get_connection():
    conn = sqlite3.connect(DB_PATH)
    conn.execute("PRAGMA foreign_keys = ON")
    return conn


def fetch_batches(conn):
    """Fetch batch configurations from timetable_slots."""
    cursor = conn.execute(
        """
        SELECT DISTINCT ts.school_id, ts.grade_level, ts.track, ts.batch_ref_id
        FROM timetable_slots ts
        WHERE ts.deleted_at IS NULL
        ORDER BY ts.school_id, ts.grade_level, ts.track
        """
    )
    return cursor.fetchall()


def fetch_timetable_slots(conn):
    """Fetch all non-deleted timetable slots."""
    cursor = conn.execute(
        """
        SELECT id, school_id, grade_level, track, batch_ref_id
        FROM timetable_slots
        WHERE deleted_at IS NULL
        ORDER BY id
        """
    )
    return cursor.fetchall()


def existing_student_count(conn, school_id, batch_ref_id):
    cursor = conn.execute(
        "SELECT COUNT(*) FROM students WHERE school_id = ? AND batch_ref_id = ?",
        (school_id, batch_ref_id),
    )
    return cursor.fetchone()[0]


def insert_students(conn, batches, students_per_batch=6):
    """Add students to each batch that needs them."""
    cursor = conn.cursor()
    students_inserted = 0

    for school_id, grade_level, track, batch_ref_id in batches:
        existing = existing_student_count(conn, school_id, batch_ref_id)
        needed = max(0, students_per_batch - existing)

        for i in range(needed):
            name = f"{random.choice(FIRST_NAMES)} {random.choice(LAST_NAMES)}"
            reg_num = f"REG{school_id:02d}{batch_ref_id:02d}{existing + i + 1:03d}"
            cursor.execute(
                """
                INSERT OR IGNORE INTO students
                (school_id, name, grade_level, program_track, track, registration_number, batch_ref_id)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    school_id,
                    name,
                    grade_level,
                    track or "Foundation",
                    track,
                    reg_num,
                    batch_ref_id,
                ),
            )
            students_inserted += cursor.rowcount

    conn.commit()
    print(f"Inserted {students_inserted} new students.")
    return students_inserted


def insert_lecture_sessions(conn, slots):
    """Create lecture sessions for today for all timetable slots."""
    cursor = conn.cursor()
    sessions_inserted = 0

    for slot_id, school_id, grade_level, track, batch_ref_id in slots:
        try:
            cursor.execute(
                """
                INSERT INTO lecture_sessions (timetable_slot_id, session_date, status, school_id, grade_level, track)
                VALUES (?, ?, 'Scheduled', ?, ?, ?)
                """,
                (slot_id, TODAY, school_id, grade_level, track),
            )
            sessions_inserted += 1
        except sqlite3.IntegrityError:
            # Session already exists for this slot+date
            pass

    conn.commit()
    print(f"Inserted {sessions_inserted} lecture sessions for {TODAY}.")
    return sessions_inserted


def insert_attendance_records(conn):
    """Mark attendance for all students in all today's lecture sessions."""
    cursor = conn.cursor()

    # Get all lecture sessions for today with their slot details
    cursor.execute(
        """
        SELECT ls.id, ls.timetable_slot_id, ts.school_id, ts.grade_level, ts.track, ts.batch_ref_id
        FROM lecture_sessions ls
        JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
        WHERE ls.session_date = ? AND ls.status != 'Cancelled'
        """,
        (TODAY,),
    )
    sessions = cursor.fetchall()

    records_inserted = 0
    records_updated = 0

    for session_id, slot_id, school_id, grade_level, track, batch_ref_id in sessions:
        # Find eligible students for this session
        cursor.execute(
            """
            SELECT id FROM students
            WHERE school_id = ? AND batch_ref_id = ?
            """,
            (school_id, batch_ref_id),
        )
        students = cursor.fetchall()

        for (student_id,) in students:
            status = random.choices(ATTENDANCE_STATUSES, weights=STATUS_WEIGHTS, k=1)[0]

            try:
                cursor.execute(
                    """
                    INSERT INTO attendance_records (lecture_session_id, student_id, status)
                    VALUES (?, ?, ?)
                    """,
                    (session_id, student_id, status),
                )
                records_inserted += 1
            except sqlite3.IntegrityError:
                # Record already exists, update status
                cursor.execute(
                    """
                    UPDATE attendance_records
                    SET status = ?
                    WHERE lecture_session_id = ? AND student_id = ?
                    """,
                    (status, session_id, student_id),
                )
                records_updated += cursor.rowcount

    conn.commit()
    print(f"Inserted {records_inserted} attendance records, updated {records_updated}.")
    return records_inserted, records_updated


def verify_das_eligible_students(conn):
    """Show how many students match each timetable slot (DAS eligibility)."""
    cursor = conn.execute(
        """
        SELECT
            ts.id AS slot_id,
            ts.school_id,
            s.name AS school_name,
            ts.grade_level,
            ts.track,
            ts.batch_ref_id,
            COUNT(st.id) AS eligible_students
        FROM timetable_slots ts
        JOIN schools s ON s.id = ts.school_id
        LEFT JOIN students st ON st.school_id = ts.school_id
            AND st.batch_ref_id = ts.batch_ref_id
        WHERE ts.deleted_at IS NULL
        GROUP BY ts.id
        ORDER BY ts.school_id, ts.grade_level, ts.track
        """
    )
    rows = cursor.fetchall()
    print("\n--- DAS Eligibility per Timetable Slot ---")
    for row in rows:
        print(f"  Slot {row[0]}: {row[3]} {row[4] or '(no track)'} @ {row[2]} → {row[6]} students")
    return rows


def main():
    print(f"Seeding DAS test data for date: {TODAY}")
    print(f"Database: {DB_PATH}")
    print()

    conn = get_connection()

    # Step 1: Identify batches from timetable slots
    batches = fetch_batches(conn)
    print(f"Found {len(batches)} distinct batch configurations in timetable_slots:")
    for b in batches:
        print(f"  School {b[0]}, {b[1]}, track='{b[2]}', batch_ref_id={b[3]}")
    print()

    # Step 2: Insert students
    insert_students(conn, batches, students_per_batch=6)

    # Step 3: Create lecture sessions for today
    slots = fetch_timetable_slots(conn)
    insert_lecture_sessions(conn, slots)

    # Step 4: Mark attendance
    insert_attendance_records(conn)

    # Step 5: Verify
    verify_das_eligible_students(conn)

    # Summary stats
    cursor = conn.execute("SELECT COUNT(*) FROM students")
    total_students = cursor.fetchone()[0]

    cursor = conn.execute("SELECT COUNT(*) FROM lecture_sessions WHERE session_date = ?", (TODAY,))
    today_sessions = cursor.fetchone()[0]

    cursor = conn.execute("SELECT COUNT(*) FROM attendance_records")
    total_attendance = cursor.fetchone()[0]

    cursor = conn.execute(
        "SELECT status, COUNT(*) FROM attendance_records GROUP BY status ORDER BY COUNT(*) DESC"
    )
    status_breakdown = cursor.fetchall()

    print(f"\n--- Summary ---")
    print(f"Total students in database: {total_students}")
    print(f"Lecture sessions for {TODAY}: {today_sessions}")
    print(f"Total attendance records: {total_attendance}")
    print("Attendance status breakdown:")
    for status, count in status_breakdown:
        pct = count * 100 // total_attendance if total_attendance else 0
        print(f"  {status}: {count} ({pct}%)")

    conn.close()
    print("\nDone! You can now test DAS calculation via the Reports panel.")


if __name__ == "__main__":
    main()
