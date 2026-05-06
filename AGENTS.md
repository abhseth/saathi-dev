# SAATHI — Agent Context

> **SAATHI** is a school operations web app for SIP (School Integrated Program). It combines a ticketing desk (support requests across schools) with master data management (schools, regions, students, faculty, timetables) and an in-progress faculty attendance system.

This file is the canonical reference for AI coding agents working on this project. When in doubt, prefer the conventions documented here over general best-practice advice.

---

## 1. Project Overview

- **What it does**: Tracks support tickets across partner schools; curates master data (school profiles, regions, class plans, faculty assignments, timetables); and will eventually manage faculty attendance and lecture sessions.
- **Architecture**: Split backend/frontend monorepo. Backend is a Rust HTTP API. Frontend is a React SPA.
- **Auth model**: JWT-based, role-scoped. Roles: `admin`, `agent`, `viewer`, `aom` (school-scoped), `faculty` (school-scoped), `head` (school-scoped approver for leave requests).
- **Data store**: SQLite (file-backed) with WAL mode, managed via `r2d2` connection pool.
- **Deployment target**: Backend on Railway (Docker), frontend on Vercel (static).

---

## 2. Technology Stack

### Backend (`backend/`)

| Layer | Choice | Version |
|---|---|---|
| Language | Rust | edition 2021 |
| Web framework | axum | 0.7 |
| Async runtime | tokio | 1 (full) |
| HTTP utilities | tower-http | 0.5 (cors, fs, trace) |
| Database | rusqlite | 0.31 (bundled, backup) |
| Connection pool | r2d2 + r2d2_sqlite | 0.8 / 0.24 |
| Serialization | serde + serde_json | 1 |
| Auth | bcrypt + jsonwebtoken | 0.15 / 9 |
| Date/time | chrono | 0.4 |
| Logging | tracing + tracing-subscriber | 0.1 / 0.3 |
| Env files | dotenvy | 0.15 |

### Frontend (`frontend/`)

| Layer | Choice | Version |
|---|---|---|
| Language | TypeScript | 5.5 |
| Framework | React | 18.3 |
| Build tool | Vite | 5.3 |
| Testing | vitest + jsdom + @testing-library/* | 2.0 |
| Styles | Plain CSS (`styles.css`, ~3300 lines) | — |

---

## 3. Repository Layout

```
/home/abhi/saathi-dev/
├── backend/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs              # Axum bootstrap, CORS, JWT secret, DB pool init
│       ├── db.rs                # Schema definition + migrations (1..56). NEVER DROP data.
│       ├── models.rs            # Input/output structs, Claims, AppState
│       ├── repositories.rs      # Pure data access (~5200 lines). No HTTP/Tauri coupling.
│       ├── error.rs             # AppError → HTTP status mapping
│       ├── auth.rs              # JWT issue/decode + require_auth middleware + scope helpers
│       └── routes/
│           ├── mod.rs           # Route wiring table
│           ├── auth.rs          # /api/auth/login, /api/auth/me
│           ├── tickets.rs       # Ticket CRUD, comments, history
│           ├── schools.rs       # Schools, regions, students, class plans, dashboard
│           ├── admin.rs         # Users, audit log, SLA, escalation, templates, DB snapshot/restore
│           ├── export.rs        # CSV exports
│           ├── imports.rs       # CSV imports (schools + SIP master with preview)
│           └── faculty.rs       # Subjects, faculty assignments, timetable slots, attendance, holidays
├── frontend/
│   ├── index.html               # PWA meta tags, viewport-fit=cover
│   ├── public/                  # manifest.json, icons
│   ├── vite.config.ts           # Dev proxy /api → localhost:3000
│   ├── vercel.json              # Rewrites /api/* to Railway backend
│   └── src/
│       ├── main.tsx             # Entry point
│       ├── App.tsx              # All state + loaders (~2100 lines)
│       ├── api.ts               # Command-name dispatcher (Tauri → HTTP bridge)
│       ├── components.tsx       # All panels, modals, forms (~8200 lines)
│       ├── types.ts             # Shared TS types mirroring backend models
│       ├── constants.ts         # Grade levels, tracks, queues, statuses, etc.
│       ├── formatters.ts        # SLA countdown, timestamps, byte formatting
│       ├── ticketFilters.ts     # Client-side filter helpers
│       └── styles.css           # All styling
├── samples/
│   ├── schools-import-template.csv
│   └── sip-master-import-template.csv
├── scripts/
│   └── verify-scoping.sh        # Integration test for AOM/faculty school scoping
├── .claude/skills/
│   ├── migrate/SKILL.md         # SQLite migration generator (data-preserving rule)
│   └── scaffold-crud/SKILL.md   # Full-stack CRUD boilerplate generator
├── Dockerfile                   # Multi-stage cargo-chef build for Railway
├── fly.toml                     # Fly.io config (backup target)
├── deploy-frontend.sh           # Build + deploy to Vercel (WARNING: points to PROD)
└── HANDOFF.md                   # Human handoff notes + deploy quirks
```

---

## 4. Build and Test Commands

### Backend

```bash
cd /home/abhi/saathi-dev/backend

# Dev server (listens on :3000, creates ./tickets.sqlite3 if absent)
cargo run

# Release build
cargo build --release

# Check only
cargo check

# Run tests (repository-level unit tests)
cargo test
```

### Frontend

```bash
cd /home/abhi/saathi-dev/frontend

# Install deps
npm install

# Dev server (opens http://localhost:5173, proxies /api → :3000)
npm run dev

# Production build (outputs to dist/)
npm run build

# Preview production build locally
npm run preview

# Run tests (vitest)
npm run test
```

---

## 5. Code Style Guidelines

### Rust

- **Error type**: Repositories return `Result<T, String>`. Route handlers convert to `AppError`.
- **Repository naming**: `list_<entities>`, `get_<entity>`, `create_<entity>`, `update_<entity>`, `delete_<entity>`, `<entity>_from_row`.
- **Validation helpers**: `validate_nonempty`, `validate_status`, `validate_priority`, `validate_queue` live in `repositories.rs`.
- **Scope enforcement**: Use `auth::scope_filter(&claims)` for list queries; use `auth::enforce_school_scope(&claims, school_id)?` before mutations.
- **String formatting**: Prefer `format!("...")` over string concatenation for SQL. Use parameterized queries (`params![]`) for values.
- **Timestamp convention**: SQLite stores datetimes as `TEXT` in `datetime('now', 'localtime')` format.

### TypeScript / React

- **No external UI library**: All UI is hand-rolled in `components.tsx` using plain CSS classes.
- **State pattern**: One giant `App.tsx` holds all state. One giant `components.tsx` holds all panel components. Resist splitting unless there is a concrete reason.
- **API calls**: Always go through `api.ts`'s `api(command, args)` dispatcher. Every endpoint must be registered in the `dispatch` object.
- **Types**: Mirror backend structs in `types.ts`. Draft types (e.g., `CreateTicketDraft`) live there too.
- **LocalStorage keys**: Prefix with `td:` (e.g., `td:activeFilter`, `td:search`).
- **Token storage**: JWT is stored in `sessionStorage` under key `td:token`.

### CSS

- **Class naming**: Semantic / BEM-ish (e.g., `.login-backdrop`, `.ticket-list`, `.metrics-card`).
- **Mobile-first**: The app is designed as a PWA for iOS. `viewport-fit=cover` is required. Bottom nav uses React Portal to `document.body`.
- **Color palette**: Primary brand color is `#1E7A6F` (teal). Status colors: Open = teal, In Progress = amber, Pending = slate, Resolved = green, Closed = neutral.

---

## 6. Testing Instructions

### Backend

Run the repository-level unit tests:

```bash
cd backend && cargo test
```

For manual scope-checking, use the provided script:
   ```bash
   # Terminal 1: start backend
   cd backend && cargo run
   # Terminal 2:
   bash scripts/verify-scoping.sh
   ```

You can also run the lightweight CI gate that checks both backend and frontend:
```bash
bash scripts/ci-gate.sh
```

### Frontend

```bash
cd frontend && npm run test
```

- Uses **vitest** + **jsdom**.
- Setup file is `src/test-setup.ts` (referenced in `vite.config.ts`).
- There are very few existing tests. Add new ones sparingly; the project values build-time type safety over unit-test coverage.

---

## 7. Security Considerations

### Auth & Scoping

- **JWT expiry**: 8 hours (`TOKEN_EXPIRY_SECS = 8 * 60 * 60`).
- **JWT secret**: Read from `JWT_SECRET` env. Falls back to a dev default (`dev-secret-change-in-production`). Warns if < 32 chars.
- **Role hierarchy**:
  - `admin` — unrestricted read/write.
  - `agent` / `viewer` — ticketing access; viewer is read-only.
  - `aom` — scoped to schools in `user_schools`; can create/update/drop schools they own.
  - `faculty` — scoped to schools in `user_schools`; used for timetable/attendance and self-service leave requests.
  - `head` — scoped to schools in `user_schools`; can approve/reject leave requests for their assigned schools.
- **Scope enforcement**:
  - `auth::scope_filter(&claims)` returns `None` for unscoped roles, `Some(&school_ids)` for scoped roles. Empty scoped lists return `Some(&[-1])` to produce zero rows.
  - `auth::enforce_school_scope(&claims, school_id)` returns 403 if a scoped user tries to access a non-assigned school.
- **Admin-only mutations**: `delete_ticket`, `delete_school`, `delete_region`, `program_dashboard`, user management all require `admin`.

### Database

- **Data preservation rule**: Migrations must never drop data. New columns get `NOT NULL DEFAULT`. Changing constraints requires recreate-and-swap (copy all rows, drop old table, rename new). See `.claude/skills/migrate/SKILL.md`.
- **SQL injection**: All repository queries use `rusqlite::params![]`. No string interpolation for values.
- **File uploads**: CSV imports are parsed server-side; never execute user-provided SQL.

### Deployment

- **CORS**: Currently allows `Any` origin. In production, restrict to your Vercel domain.
- **Env secrets**: `.env.local` and `secrets.txt` are gitignored. Never commit credentials.
- **Database path**: On Railway, `DATABASE_PATH=/data/tickets.sqlite3` with a mounted volume. On local dev, defaults to `./tickets.sqlite3`.

---

## 8. Deployment Process

### Backend (Railway)

```bash
# ONLY deploy from the production project directory (/home/abhi/ticketing-web)
# This dev copy has NO deployment configured.
cd /home/abhi/ticketing-web && railway up
```

- Railway builds from `Dockerfile` (cargo-chef multi-stage).
- Container boot runs migrations automatically.
- Volume must be mounted at `/data` before first deploy.

### Frontend (Vercel)

```bash
# WARNING: deploy-frontend.sh points to LIVE production.
# Do NOT run unless you created a NEW Vercel project.
bash deploy-frontend.sh
```

- Build locally with `npm run build`, copy `vercel.json` into `dist/`, then `vercel deploy --prod`.
- `vercel.json` rewrites `/api/(.*)` to the Railway backend.

---

## 9. Database Migrations

Migrations live in `backend/src/db.rs` inside `apply_migrations()`. They are numbered sequentially (currently 1–56).

### Rules

1. **Preserve all rows.** Never `DROP COLUMN` without a recreate-and-swap that copies data.
2. **Idempotent.** Use `migration_applied()` + `column_exists()` checks so re-running is safe.
3. **Default values.** Every `ALTER TABLE ADD COLUMN` must include `NOT NULL DEFAULT`.
4. **Seed after schema.** `seed_*` functions run after migrations complete.

### How to add a migration

Use the project-local skill:

```
/migrate add a notes text column to schools
```

Or manually:
1. Find next number: `grep -E "if !migration_applied\(conn, [0-9]+\)" backend/src/db.rs | tail -3`
2. Insert block before `seed_communication_templates(conn)?;`
3. Update `models.rs`, `repositories.rs`, `types.ts`, and `api.ts` if columns changed.
4. Run `cd backend && cargo build`.

---

## 10. Adding a New Entity

Use the project-local skill:

```
/scaffold-crud Holiday: school_id INTEGER FK schools, date TEXT, name TEXT, scope TEXT
```

This generates:
- SQLite migration
- Rust model + repository functions
- Route handlers + registration in `routes/mod.rs`
- TS types + `api.ts` dispatcher entries
- Stub React component in `components.tsx`

---

## 11. Environment Variables

| Variable | Required | Default | Notes |
|---|---|---|---|
| `DATABASE_PATH` | no | `tickets.sqlite3` | Use `/data/tickets.sqlite3` on Railway |
| `JWT_SECRET` | no | `dev-secret-change-in-production` | Must be ≥32 chars in production |
| `PORT` | no | `3000` | Railway sets this automatically |
| `FRONTEND_DIST` | no | — | If set, backend serves static files from this path |
| `VITE_API_URL` | no | `""` | Frontend build-time variable; empty means same-origin |

---

## 12. Key Conventions to Remember

- **One big file is okay.** `App.tsx` and `components.tsx` are intentionally monolithic. Don't refactor for the sake of it.
- **Mirror types exactly.** Backend `models.rs` ↔ Frontend `types.ts` should stay in sync.
- **Scoped roles use `user_schools`.** Many-to-many table linking `users.id` → `schools.id`.
- **Track splits at Grade 11/12/Dropper.** Foundation grades (6–10) use `track = ""`. JEE/NEET are explicit tracks.
- **Subjects are track-scoped.** JEE: Physics/Chemistry/Mathematics. NEET: Physics/Chemistry/Botany/Zoology. Foundation: Physics/Chemistry/Mathematics/Biology + optional English/SST per school.
- **Test users seeded on boot:** `admin`/`admin123`, `aom1`/`aom123`, `faculty1`/`faculty123`, `head1`/`head123`, `viewer1`/`viewer123`.

---

## 13. Troubleshooting

| Symptom | Likely Cause | Fix |
|---|---|---|
| `cargo build` fails with unresolved imports | `models.rs` and `repositories.rs` imports out of sync | Add new types to both `use` blocks |
| Frontend shows "Unknown API command" | Missing entry in `api.ts` `dispatch` object | Register the command with method + path |
| CORS errors in browser | Backend CORS allows `Any` but `VITE_API_URL` is wrong | Leave `VITE_API_URL` empty for local dev; use `vercel.json` rewrites in prod |
| SQLite "database is locked" | Too many concurrent writes | Already mitigated by `r2d2` pool; if still happening, check for long-held connections |
| Migration not applying | `schema_migrations` row missing or `column_exists` returning true unexpectedly | Inspect DB with `sqlite3 tickets.sqlite3 "SELECT * FROM schema_migrations;"` |
