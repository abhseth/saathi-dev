# SAATHI — Handoff

Last updated: 2026-04-28

## What this is

A school operations web app — ticketing + master data + (in progress) faculty attendance — for SIP (School Integrated Program). Tickets track issues across schools; master data (schools, regions, faculty, timetables, etc.) is curated centrally and used by school-level operations.

Originated as a web rewrite of `/home/abhi/ticketing-desktop` (Tauri). The desktop app remains the historical reference for some helpers (CSV parsing patterns, original schema). Don't modify it.

## Live deployments

| Layer | URL | Platform |
|---|---|---|
| Frontend | https://saathi-pink.vercel.app | Vercel ("dist" project under `abhseth-8942s-projects`) |
| Backend  | https://saathi-production-aa2a.up.railway.app | Railway |
| Default login | `admin` / `admin123` | (change before public use) |

Deploy quirks are documented in memory at `~/.claude/projects/-home-abhi-ticketing-desktop/memory/project_web_deploy.md`. Read that before deploying.

## Stack

- **Backend**: Rust + Axum 0.7, rusqlite (bundled SQLite), bcrypt, jsonwebtoken, multipart for CSV imports
- **Frontend**: React 18 + TypeScript + Vite, PWA-installable on iOS via `apple-mobile-web-app-capable`
- **Auth**: JWT in `sessionStorage`, 8-hour expiry. Roles: admin, agent, viewer, aom (school-scoped), faculty (school-scoped)

## Repository layout

```
/home/abhi/ticketing-web/
├── .claude/skills/
│   ├── migrate/SKILL.md          /migrate — generates a SQLite migration following project rules
│   └── scaffold-crud/SKILL.md    /scaffold-crud — full-stack CRUD boilerplate generator
├── backend/
│   └── src/
│       ├── main.rs               Axum server bootstrap, JWT secret, DATABASE_PATH
│       ├── db.rs                 Schema + migrations (1..28). Latest = 28.
│       ├── models.rs             Input/output structs + JWT Claims + AppState
│       ├── repositories.rs       Pure data access; no Tauri/HTTP coupling
│       ├── error.rs              AppError → HTTP status mapping
│       ├── auth.rs               JWT issue/decode + require_auth middleware
│       └── routes/
│           ├── mod.rs            Router wiring (all endpoints listed here)
│           ├── auth.rs           /api/auth/login, /api/auth/me
│           ├── tickets.rs        Tickets, comments, history
│           ├── schools.rs        Schools, regions, students, class plans, dashboard
│           ├── admin.rs          Users, audit log, SLA, escalation, templates,
│           │                     DB snapshot/restore endpoints
│           ├── export.rs         CSV exports (tickets, communications, sip-master)
│           ├── imports.rs        CSV imports (schools, sip-master with preview)
│           └── faculty.rs        Subjects, faculty assignments, timetable slots
├── frontend/
│   ├── index.html                PWA meta tags + viewport-fit=cover
│   ├── public/manifest.json      PWA manifest
│   ├── src/
│   │   ├── main.tsx, App.tsx
│   │   ├── api.ts                Single dispatch table mapping command names → HTTP
│   │   ├── components.tsx        All admin panels, modals, forms (large file ~5200 lines)
│   │   ├── types.ts              Shared TS types matching backend models
│   │   ├── constants.ts          gradeLevels, batchPatterns, academicTracks, etc.
│   │   ├── formatters.ts         formatField, formatTimestamp, getSlaState
│   │   ├── ticketFilters.ts      Client-side filtering helpers
│   │   └── styles.css            All styling (large file ~3300 lines)
├── samples/
│   ├── schools-import-template.csv
│   └── sip-master-import-template.csv
├── deploy-frontend.sh            Builds dist + copies vercel.json + uploads to Vercel
├── Dockerfile                    Multi-stage Rust build for Railway
├── HANDOFF.md                    ← you are here
├── saathi-snapshot-*.sqlite3     Latest local DB snapshot (kept for safety)
└── secrets.txt, .env.local       Local-only secrets; not committed
```

## Current state (what's done, what's pending)

**Shipping today**: ticketing, schools/regions/students master data, class plans (with track + registrations), CSV imports (schools + SIP master), Directory with deduped contacts + bulk-mail multi-select, mobile PWA UI, DB snapshot/restore for backup.

**In progress**: faculty/attendance system. See memory `project_faculty_app_phase1.md` for the 5-phase plan and step-by-step status.

| Phase | Status |
|---|---|
| 1 — Schema + admin UI for subjects/faculty/timetable | Steps 1-5 of 8 done. Steps 6-8 pending. |
| 2 — Minimal faculty app (separate frontend) | Not started |
| 3 — Status expansion + substitutions | Not started |
| 4 — Reports + role-based scoping | Not started |
| 5 — Polish (holidays, makeup, CSV imports for timetable) | Not started |

Step 6 is **AOM scope-check middleware** — currently AOMs see all data; this needs filtering by `user_schools`. Step 7 is the Subjects panel UI (English/SST opt-in toggles per Foundation school). Step 8 is creating real test users.

## Architecture decisions

### Auth + scoping

- JWT in `sessionStorage`. 8-hour expiry. `Authorization: Bearer <token>` on every protected request.
- Roles: `admin` (unscoped — used by IT and Central SPOCs), `aom` (school-scoped via `user_schools`), `faculty` (school-scoped, used in Phase 2+), `agent` (ticketing only), `viewer` (read-only).
- `user_schools` is a many-to-many table mapping users to schools. Future scoped roles (Principal, Center Head) will use the same mechanism.

### Database

- SQLite via rusqlite, file path from `DATABASE_PATH` env (default `tickets.sqlite3`, set to `/data/tickets.sqlite3` on Railway with a mounted volume).
- WAL mode enabled in `db.rs`. Migrations are numbered, additive, and **must preserve existing rows** — see memory `feedback_preserve_server_data.md`. Use `/migrate` skill.
- Single `Mutex<Connection>` in `AppState` for serialized writes.

### Subjects + tracks

- Subjects are track-scoped: JEE has Phy/Chem/Math; NEET has Phy/Chem/Bot/Zoo; Foundation has Phy/Chem/Math/Bio default + English/SST opt-in per school.
- Track is a separate column on `school_class_plans`, `faculty_assignments`, `timetable_slots`. So Grade 11 + JEE and Grade 11 + NEET are distinct rows.
- Grades 6-10 use `track = ''` (empty); 11/12/Dropper carry JEE or NEET.

### Frontend

- One giant `App.tsx` holds all state and one giant `components.tsx` holds all panel components. Yes, monolithic; resist refactoring into smaller files unless you have a real reason.
- API calls go through `api.ts`'s `api(command, args)` dispatcher — every endpoint is registered there as `command_name: { method, path, bodyKey? }`.
- PWA: configured for installable home-screen icon on iPhone. `viewport-fit=cover` + `apple-mobile-web-app-status-bar-style=black-translucent`. Bottom nav rendered as React Portal to `document.body` for reliable viewport-edge anchoring.

### Deploy

- Frontend deploys are pre-built locally and uploaded to Vercel (project name "dist"). Don't try to redirect to a project named "saathi" — that one's broken; abandoned. See `project_web_deploy.md`.
- Backend: `cd /home/abhi/ticketing-web && railway up`. Railway runs migrations on container boot. The CLI may "time out" after upload; that's normal — build continues in cloud.
- The `git config --global user.email` must contain `@`. Railway's deploy-protection rejects malformed emails even on non-git deploys.

### Skills

Two project-local skills under `.claude/skills/`:

- **`/migrate`** — invoke for any schema change. Picks next migration number, generates the migration block following the data-preserving pattern, lists downstream files (model.rs / repo.rs / types.ts) that need matching updates, runs `cargo build`. Refuses DROP COLUMN.
- **`/scaffold-crud`** — invoke for a new entity. Generates migration + Rust model + repo + routes + TS type + api.ts mapping + stub component. Used heavily in Phase 2+ for `lecture_sessions` and `attendance_records`.

## Environment variables

**Backend (Railway)**:
| Variable | Required | Notes |
|---|---|---|
| `DATABASE_PATH` | yes | Set to `/data/tickets.sqlite3` once volume is attached |
| `JWT_SECRET` | yes | Must be set; do not use the dev default in production |
| `PORT` | auto | Railway sets this |

**Frontend (Vercel)**:
| Variable | Required | Notes |
|---|---|---|
| `VITE_API_URL` | no | Empty by default — Vercel uses `/api` rewrite to Railway via `vercel.json` `routes` schema |

## Local development

```bash
# Backend
cd /home/abhi/ticketing-web/backend
cargo run
# Listens on :3000 by default; reads DATABASE_PATH or creates ./tickets.sqlite3

# Frontend (separate terminal)
cd /home/abhi/ticketing-web/frontend
npm install
npm run dev
# Opens http://localhost:5173 ; vite proxies /api → :3000
# Default login: admin / admin123
```

## Deploy

```bash
# Backend
cd /home/abhi/ticketing-web && railway up

# Frontend
bash /home/abhi/ticketing-web/deploy-frontend.sh
# (Wraps build + dist link + vercel deploy + saathi-pink alias)
```

## Backup / restore

The DB is on Railway. Two paths:

1. **Snapshot endpoint** — admin-only, streams a consistent SQLite snapshot:
   ```bash
   TOKEN=$(curl -s -X POST .../api/auth/login -d '{"username":"admin","password":"..."}' | jq -r .token)
   curl -OJ -H "Authorization: Bearer $TOKEN" https://saathi-pink.vercel.app/api/admin/db-snapshot
   ```
2. **Restore endpoint** — uploads a SQLite file and atomically swaps. See `routes/admin.rs::db_restore`.

There's a versioned source backup at `/home/abhi/saathi-backups/saathi-v1.0.0-pre-faculty-app-2026-04-28.tar.gz` (with `.RESTORE.md` alongside) — pre-faculty-app baseline, useful as a known-good rewind point.

## Sample data + CSV templates

`samples/` has CSV templates with full headers + 1-2 example rows:

- `schools-import-template.csv` — bulk-import schools (8 contact-block trios)
- `sip-master-import-template.csv` — schools + region + class plans for all 11 grade/track slots (6, 7, 8, 9, 10, 11×{JEE,NEET}, 12×{JEE,NEET}, Dropper×{JEE,NEET}). Each grade has lecture_model, batch_pattern, AOP, registrations, actual columns.

## Memory files (read in order when picking up cold)

Located at `~/.claude/projects/-home-abhi-ticketing-desktop/memory/`:

1. `MEMORY.md` — index
2. `project_overview.md` + `project_web_branch.md` — high-level project shape
3. `project_web_deploy.md` — deploy gotchas (Vercel project name, vercel.json schema, etc.)
4. `feedback_preserve_server_data.md` — schema-change rule
5. `project_faculty_app_phase1.md` — current-phase progress + remaining steps

## Resume prompt

```
Project: /home/abhi/ticketing-web
Live: https://saathi-pink.vercel.app

Read HANDOFF.md (this file) and the memory files in order before writing any code.
Most recent work: faculty app Phase 1, Steps 1-5 done.
Next: Step 6 — AOM scope-check middleware. AOMs currently see all data; need to
filter list endpoints by user_schools and reject mutations on schools they don't
own. Then Step 7 (Subjects panel UI) and Step 8 (test users on prod).

Skills available: /migrate, /scaffold-crud (project-local under .claude/skills/).
Use /migrate for any schema change. Don't violate the data-preserve rule.
```
