---
name: scaffold-crud
description: Generate full-stack CRUD boilerplate (migration + Rust model/repo/route + TS type/api/component stub) for a new entity in SAATHI. Use when adding a new resource that needs list/create/update/delete endpoints.
---

# /scaffold-crud

You are scaffolding a new entity end-to-end across the SAATHI codebase. The goal: take an entity definition and produce all the matching boilerplate so the user only has to fill in the actual UI/business logic.

## What you generate

For an entity `<Entity>` with fields:

1. **SQLite migration** — invoke the `/migrate` skill or follow the same pattern inline.

2. **Rust model** in `backend/src/models.rs` (append to bottom):
   - `<Entity>` struct (Serialize + Debug + Clone)
   - `Create<Entity>Input` (Deserialize) — fields optional via `#[serde(default)]` where appropriate
   - `Update<Entity>Input` (Deserialize) — must include `id: i64`

3. **Repository functions** in `backend/src/repositories.rs` (insert before `#[cfg(test)]`):
   - `list_<entities>(conn) -> Result<Vec<Entity>, String>`
   - `create_<entity>(conn, &input) -> Result<Entity, String>`
   - `update_<entity>(conn, &input) -> Result<Entity, String>`
   - `delete_<entity>(conn, id) -> Result<(), String>`
   - `get_<entity>(conn, id) -> Result<Entity, String>` (private helper)
   - `<entity>_from_row(row)` row-mapper (private helper)

   Update the `use crate::models::{...}` import block to include the new types.

4. **Route handlers** in `backend/src/routes/<module>.rs`:
   - Default to `routes/admin.rs` for admin-only entities
   - Default to `routes/faculty.rs` for faculty/timetable-related entities
   - For something else, ask user where to put it (or create a new module)
   - Each handler: extract `Claims`, gate by role, lock state, call repo, return Json

5. **Route registration** in `backend/src/routes/mod.rs`:
   ```rust
   .route("/<entities>", get(<module>::list_<entities>))
   .route("/<entities>", post(<module>::create_<entity>))
   .route("/<entities>/:id", put(<module>::update_<entity>))
   .route("/<entities>/:id", delete(<module>::delete_<entity>))
   ```

6. **Frontend type** in `frontend/src/types.ts` (append):
   ```ts
   export type <Entity> = { id: number; ... };
   export type Create<Entity>Draft = { ... };  // matching Create<Entity>Input
   export type Update<Entity>Draft = { id: number; ... };
   ```

7. **API mappings** in `frontend/src/api.ts` (add to `dispatch` object):
   ```ts
   list_<entities>: { method: "GET",  path: () => "/<entities>" },
   create_<entity>: { method: "POST", path: () => "/<entities>",          bodyKey: "input" },
   update_<entity>: { method: "PUT",  path: (a) => `/<entities>/${(a.input as {id:number}).id}`, bodyKey: "input" },
   delete_<entity>: { method: "DELETE", path: (a) => `/<entities>/${a.id}` },
   ```

8. **Stub admin panel** in `frontend/src/components.tsx` — minimal skeleton with:
   - Modal-backdrop wrapper using `directory-modal` style
   - Header with title + Close button
   - List section showing entries
   - "Add" form section with the new entity's fields
   - Export the component name as `<Entity>Panel`

   Don't build polished UI — just a working list+create skeleton. The user will customize.

## Steps

1. **Parse `$ARGUMENTS`** for entity name + fields. Format expected:
   ```
   Holiday: school_id INTEGER FK schools, date TEXT, name TEXT, is_recurring BOOLEAN
   ```

   If not provided or unclear, ask the user for:
   - Entity name (PascalCase, singular)
   - Fields (name, type, FK references, nullability)
   - Auth gating (admin only, admin/aom, any authenticated)

2. **Plan and confirm before coding.** Output a brief preview:
   - Migration N assigned
   - Table name (snake_case plural) + schema
   - Rust struct field types
   - Auth gate
   - Where the route handlers will live
   - Wait for "go" / approval.

3. **Run the migration first** (use `/migrate` or inline). Do NOT proceed to model/repo if migration fails.

4. **Generate model + repo + routes** mirroring an existing similar entity exactly:
   - For school-scoped: mirror `faculty_assignments` shape
   - For globally-scoped admin entity: mirror `subjects` shape
   - For per-school config table: mirror `school_optional_subjects` shape

5. **Wire api.ts mappings** — add the four entries to the existing dispatch table.

6. **Build both:**
   - `cd /home/abhi/ticketing-web/backend && cargo build` — must compile
   - `cd /home/abhi/ticketing-web/frontend && npm run build` — must compile

7. **Stop short of UI polish.** Output the location of the stub component + the next thing the user typically wants (mounting it in `App.tsx` admin view chain + sidebar/more-menu nav entry).

## Important rules

- **Match existing patterns precisely.** Same indentation, naming conventions, error-handling style. Code should look like it was always there.
- **No invented features.** Don't add filtering, sorting, scope-checks, audit logging, etc. unless explicitly requested. Pure CRUD only.
- **Build before reporting done.** If either build fails, surface the errors before claiming complete.
- **Use the project memory rule.** Migrations preserve data — defaults on new columns, no DROP without forward-INSERT.

## Arguments

`$ARGUMENTS` describes the entity. Examples:
- `Holiday: school_id INTEGER FK schools, date TEXT, name TEXT, is_recurring BOOLEAN`
- `LectureSession: timetable_slot_id INTEGER FK timetable_slots, date TEXT, status TEXT, notes TEXT`
- `AttendanceRecord: lecture_session_id INTEGER FK lecture_sessions, student_id INTEGER FK students, status TEXT, marked_by_user_id INTEGER FK users, marked_at TEXT`

If empty, ask interactively.

## Output

End by summarizing:
- Files created/modified (full list with absolute paths)
- Migration number
- Endpoints exposed
- Build status (backend and frontend)
- Suggested next step: mount in App.tsx admin view chain + add nav entry
