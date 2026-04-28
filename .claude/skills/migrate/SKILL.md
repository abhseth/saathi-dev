---
name: migrate
description: Add a new SQLite migration to backend/src/db.rs following SAATHI's data-preserving rule (ADD COLUMN with defaults, or recreate-and-swap; never DROP). Use when the user wants a schema change.
---

# /migrate

You are creating a new schema migration for the SAATHI backend at `backend/src/db.rs`.

## Project rule (NEVER violate)

Migrations must preserve every existing row. Never use `DROP COLUMN` or `DROP TABLE` without a corresponding INSERT-from-old in the same migration. When changing a constraint (UNIQUE, CHECK, FK), recreate the table and copy every row through.

## Steps

1. **Find the next migration number:**
   ```
   grep -E "if !migration_applied\(conn, [0-9]+\)" backend/src/db.rs | tail -3
   ```
   Use the latest +1.

2. **Pick the right pattern** based on `$ARGUMENTS` (or ask the user if not specified):
   - "add column X to table Y" → ADD COLUMN with default
   - "new table Z" → CREATE TABLE IF NOT EXISTS
   - "change UNIQUE/CHECK/FK on Y" → recreate-and-swap
   - "rename column" → soft-rename (add new, copy data, leave old in place; suggest a follow-up migration to drop old once code no longer reads it)

3. **Insert the migration block** in `backend/src/db.rs` immediately before the `seed_communication_templates(conn)?;` line.

   **ADD COLUMN template:**
   ```rust
   if !migration_applied(conn, N)? {
       if !column_exists(conn, "TABLE", "COL")? {
           conn.execute(
               "ALTER TABLE TABLE ADD COLUMN COL TYPE NOT NULL DEFAULT 'DEFAULT_VALUE'",
               [],
           ).map_err(|e| e.to_string())?;
       }
       conn.execute("INSERT INTO schema_migrations (version) VALUES (N)", [])
           .map_err(|e| e.to_string())?;
   }
   ```

   **CREATE TABLE template:**
   ```rust
   if !migration_applied(conn, N)? {
       conn.execute_batch("
           CREATE TABLE IF NOT EXISTS T (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               ...
               UNIQUE(...),
               FOREIGN KEY(fk_id) REFERENCES other(id) ON DELETE CASCADE
           );
           INSERT INTO schema_migrations (version) VALUES (N);
       ").map_err(|e| e.to_string())?;
   }
   ```

   **Recreate-and-swap template** (for changing constraints):
   ```rust
   if !migration_applied(conn, N)? {
       // Optionally ALTER TABLE ADD COLUMN first if also adding fields
       conn.execute_batch("
           CREATE TABLE T_new ( /* full new schema */ );
           INSERT INTO T_new (col1, col2, ...)
               SELECT col1, COALESCE(col2, ''), ... FROM T;
           DROP TABLE T;
           ALTER TABLE T_new RENAME TO T;
           INSERT INTO schema_migrations (version) VALUES (N);
       ").map_err(|e| e.to_string())?;
   }
   ```

4. **If a column was added, list the downstream files that need updates:**
   - `backend/src/models.rs` — add field to the struct (Serialize) and CreateInput (Deserialize, optionally `#[serde(default)]`)
   - `backend/src/repositories.rs` — every SELECT statement for that table, every INSERT/UPDATE, the row_mapper function, any test fixtures
   - `frontend/src/types.ts` — TS type
   - Any frontend draft initializers (e.g., `emptyXDraft`)

   Walk the user through these or do them inline if `$ARGUMENTS` is explicit enough.

5. **Build:** `cd /home/abhi/ticketing-web/backend && cargo build` — surface any errors.

6. **Refuse data-loss patterns.** If asked to DROP COLUMN, refuse and propose soft-rename. If asked to wipe a table, refuse and propose backup-via-snapshot-endpoint first.

## Arguments

`$ARGUMENTS` is the freeform description, e.g.:
- "add a notes text column to schools"
- "new lecture_sessions table with timetable_slot_id, date, status fields"
- "change unique key on attendance from (student_id, date) to (student_id, date, lecture_slot_id)"

If empty, ask the user what schema change they want.

## Output

End by summarizing in 4 lines:
1. Migration N assigned
2. Pattern used (ADD COLUMN / CREATE TABLE / recreate-and-swap)
3. Build status
4. Downstream files to update next (if any)

## Reference: project memory rule

The user has a durable rule saved in `~/.claude/projects/-home-abhi-ticketing-desktop/memory/feedback_preserve_server_data.md`. This skill exists to enforce that rule mechanically.
