# Critical Review: SAATHI Ticketing-Web (Phase 1)

**Review Date:** 2026-04-28  
**Status:** Phase 1 (Step 5/8 Complete)  
**Reviewer:** Gemini CLI

## 1. Development Assessment (Done so far)

### Successes
- **Web Migration:** The abstraction of the desktop logic into `repositories.rs` and the use of `api.ts` to bridge the Tauri-to-HTTP gap is clean and effective.
- **Schema Design:** The track-scoped subjects and many-to-many `user_schools` relationship correctly anticipate the "floater" faculty and school-specific curriculum (Foundation/JEE/NEET).
- **Admin UI:** The implementation of the Timetable week grid and Faculty Assignment panels provides a solid operational foundation.

### Critical Critiques
- **Authorization & Security:** The system currently lacks row-level security (RLS) or scope enforcement. An AOM user can currently see and modify data for any school. This is a "blocker" for any production testing.
- **Database Architecture:** Using `Mutex<Connection>` in `AppState` serializes all DB access. While acceptable for a single-user desktop app, it is a bottleneck for a web app where multiple AOMs and Faculty will interact concurrently.
- **JWT Claims:** The current `Claims` struct is too thin. It forces the backend to resolve user identity to permissions on every single request, increasing database load.
- **Error Handling:** Repositories return `Result<T, String>`. This forces the route handlers to perform opaque error conversion. Structured error types in the repository layer would allow for more accurate HTTP status mapping (e.g., distinguishing between a 404 Not Found and a 400 Bad Request).

---

## 2. Planning Assessment (Phases 2-5)

### Strategy Critique
- **Separate Frontend (Phase 2):** Moving the faculty app to a separate frontend is a sound decision to keep the Admin/AOM "power user" UI distinct from the mobile-first "mark attendance" UI. However, this doubles the maintenance burden for shared components (types, constants, API client). 
- **Visibility Timing:** "Full role-based visibility" is slated for Phase 4. Given the data sensitivity, basic scoping (Step 6 of Phase 1) must be implemented with extreme rigor now, rather than waiting for Phase 4's "Polish".
- **Attendance Model:** The plan for "Present/Absent" in Phase 2 followed by status expansion in Phase 3 is a good iterative approach (MVP first).

### Infrastructure Risks
- **SQLite Persistence:** The deferred Railway volume setup is a high-risk operational debt. Manual snapshot/restore is not a viable strategy for an app tracking daily attendance.

---

## 3. Recommendations for Claude (Future Sessions)

1. **Implement `r2d2`:** Replace `Mutex<Connection>` with a connection pool to allow concurrent reads/writes.
2. **Enrich JWT Claims:** Add `school_ids` and `role` to the JWT. This allows the backend to perform initial scope validation without a DB hit.
3. **Formalize `enforce_school_scope`:** This shouldn't just be a helper but a core part of the request pipeline for any route taking a `school_id` or `ticket_id`.
4. **Shared Library Strategy:** If the Faculty App (Phase 2) is a separate project, consider how `api.ts` and `types.ts` will be synchronized. A shared directory or a private npm package may be necessary.
5. **Infrastructure First:** Resolve the Railway volume issue before proceeding to Phase 2. Data loss during a `railway up` deployment will break user trust immediately.
