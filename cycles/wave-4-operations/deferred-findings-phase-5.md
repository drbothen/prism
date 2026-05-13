---
document_type: deferred-findings-phase-5
cycle: wave-4-operations
status: active
created: 2026-05-13
producer: state-manager
rationale: "Findings that are out-of-perimeter for story-scoped fix-bursts but require resolution before Phase 5 (Adversarial Refinement, post-implementation cascade) closes. Per CLAUDE.md Canonical Principle Rule 3: not added to tech-debt-register (no human direction, no concrete future dependency, no story anchor). Phase 5 PO-led adjudication is the correct routing for cross-document governance gaps."
---

# Phase-5 Deferred Findings — Wave 4 Operations Cycle

This file accumulates findings that are out-of-perimeter for story-scoped fix-bursts during Phase 3
(TDD Implementation). Each finding is routed here when:
1. The defect is in a cross-cutting artifact (error-taxonomy.md, BC pair governance) rather than in the
   story body under review.
2. The fix requires PO adjudication or architectural decision that goes beyond story-writer scope.
3. The CLAUDE.md boundaries clause applies: "expanding into a new domain that requires new specs or
   new architecture decisions" requires explicit scope expansion request.

Findings here MUST be addressed before Phase 5 (Adversarial Refinement) convergence is declared.
They are NOT tech-debt-register entries (no human-directed deferral; these are production-grade gaps
awaiting the correct phase gate).

---

## F-LP12-OBS-001 — E-PLUGIN-008 Dual-Semantic Reuse

| Field | Value |
|-------|-------|
| **Finding ID** | F-LP12-OBS-001 |
| **Severity** | OBS (out-of-perimeter for story scope; substantive for phase-5) |
| **Confidence** | HIGH |
| **Story source** | S-PLUGIN-PREREQ-D |
| **Surfaced at** | Pass-12 (adversary fresh-context audit) |
| **Date routed** | 2026-05-13 |
| **Target** | Phase-5 product-owner-led error namespace adjudication |

### Evidence

- **BC-2.17.005** anchors E-PLUGIN-008 to hot-reload WASM compilation failure (message: retry with new binary).
- **BC-2.17.006** anchors the same E-PLUGIN-008 code to boot-time `Component::from_binary` failure on corrupt `.prx` bytes (message: initial-load failure, no previous version to retain).
- **error-taxonomy.md** message template for E-PLUGIN-008: "Plugin '{plugin_id}' hot-reload failed: WASM compilation error: {error}. Previous version retained." — anchored ONLY to BC-2.17.005 hot-reload context; misleading and incorrect at boot-time initial-load where no previous version exists to retain.

### Why It Matters

An implementer coding EC-D-007 (boot-time corrupt bytes scenario per BC-2.17.006) would use E-PLUGIN-008 per the story spec — correct per BC-2.17.006 anchor. But the error-taxonomy.md message template would produce a message saying "Previous version retained" when there is no previous version. At boot-time initial-load, retaining a previous version is not possible.

Story EC-D-007 (line 126) is internally consistent (cites E-PLUGIN-008 per BC-2.17.006) — the STORY is not the defect location. The gap is cross-doc governance between BC-2.17.005, BC-2.17.006, and error-taxonomy.md.

### Fix Options (for Phase-5 PO adjudication)

**Option A — Split E-PLUGIN-008**: Create E-PLUGIN-008a (hot-reload, retain current message template "hot-reload failed: WASM compilation error") + new E-PLUGIN-N (initial-load, message "initial-load failed: WASM compilation error; no previous version available"). POL-1 append-only new code; existing E-PLUGIN-008 hot-reload usage in BC-2.17.005 preserved.

**Option B — Conditional message template**: Update error-taxonomy.md template to context-aware messaging covering both anchors (e.g., template parameterized by context: hot-reload vs initial-load). Single code, two message forms.

**Option C — Re-anchor BC-2.17.006**: Assign a distinct new E-PLUGIN-N code to BC-2.17.006 initial-load failure; preserve E-PLUGIN-008 as hot-reload-only semantics. Simplest canonical split; highest BC-drift risk (BC-2.17.006 + story EC-D-007 both need updating).

### Pre-existing Gap Age

11 passes (story created for pass-1; gap existed from story creation but only surfaced at pass-12 via fresh-context deep audit).

### Resolution Criteria

Before Phase 5 convergence can be declared: error-taxonomy.md E-PLUGIN-008 entry updated so that the message template is not misleading for the boot-time initial-load context, OR a distinct error code for that context is established, with BC-2.17.005 and BC-2.17.006 updated accordingly.
