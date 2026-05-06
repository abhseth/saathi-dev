# GEMINI AUDIT #03: INTERFACE & EXPERIENCE

**Date:** 2026-05-02
**Status:** MEDIUM RISK (Stiff UX & Fragile PWA)
**Scope:** Mobile Usability, PWA Offline-Readiness, and Interactive Polish

---

## 1. Executive Summary
The SAATHI frontend provides a solid visual foundation but fails to deliver the "alive" and "fluid" experience required for a production-grade PWA. The primary risks are the non-functional service worker (preventing true offline use), a lack of interactive feedback during API mutations, and high visual noise in mobile list views.

---

## 2. PWA & Offline Readiness

### **2.1 Non-Functional Service Worker**
*   **Observation:** `public/sw.js` is a stub that does not cache application assets.
*   **Risk:** The app will not load without an active internet connection, defeating the purpose of the "Installable PWA" configuration for school environments.

### **2.2 Brittle API Layer**
*   **Observation:** `api.ts` does not integrate with `useOfflineCache` for fallback retrieval or mutation queuing.
*   **Risk:** Data loss during transient network drops (e.g., moving between school buildings).

---

## 3. Interaction & Visual Polish

### **3.1 Feedback Gaps**
*   **Observation:** `SkeletonBlock` is largely unused. Static "Loading..." text is the default placeholder.
*   **Risk:** The UI feels "stiff" and less responsive than modern competitors.
*   **Observation:** "Save" buttons in modals (e.g., Ticket Creation) lack loading states.
*   **Risk:** Double-submission of data when users click multiple times during slow API responses.

### **3.2 Inconsistent UI Feedback**
*   **Observation:** Native `alert()` calls are used in `FacultyApp.tsx` instead of the standardized notification system.
*   **Risk:** Broken immersion and poor user experience on mobile devices.

---

## 4. Mobile Usability & Accessibility

### **4.1 iOS Safe Area Inconsistency**
*   **Observation:** `FacultyBottomNav` and several modal overlays (`.ticket-modal`) lack `safe-area-inset-bottom` handling.
*   **Risk:** UI elements overlapping with the iPhone home indicator or being cut off by the notch.

### **4.2 Data Density**
*   **Observation:** `TicketList` displays 9+ fields per row on mobile screens.
*   **Risk:** Extreme visual noise, making it difficult for agents to scan tickets on the go.

### **4.3 Accessibility (A11y)**
*   **Observation:** Custom clickable `div`s and `span`s (backdrops, icon-buttons) lack `tabIndex` and `onKeyDown` listeners.
*   **Risk:** The app is non-compliant with basic accessibility standards and unusable for keyboard-only users.

---

## 5. Recommendations

| Priority | Category | Action Item |
| :--- | :--- | :--- |
| **P0** | **PWA** | Implement asset caching and "Stale-While-Revalidate" in `sw.js`. |
| **P0** | **UX** | Add `loading` states to all mutation buttons (Create/Update/Delete). |
| **P1** | **UX** | Replace "Loading..." text with `SkeletonBlock` components across all panels. |
| **P1** | **Mobile** | Enforce `env(safe-area-inset-bottom)` on all fixed-position mobile elements. |
| **P2** | **A11y** | Add keyboard listeners to all custom interactive elements in `components.tsx`. |
| **P2** | **Mobile** | Simplify `TicketList` mobile view to show only 3-4 essential fields. |

---
**Lead Auditor:** Gemini CLI Agent
**Teams Involved:** Interface & Experience Team, Core Systems Team
