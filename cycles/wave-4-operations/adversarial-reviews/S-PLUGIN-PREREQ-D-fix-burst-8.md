---
document_type: fix-burst-closure-report
target_artifact: S-PLUGIN-PREREQ-D
fix_burst_N: 8
target_pass: 9
findings_closed: 2_actionable (MED + LOW)
findings_deferred: 1 (F-LP9-OBS-001 — version-pin-sweep-burst-vs-version-prose-distinction pattern — codification candidate routed to cycle-closing checklist alongside F-LP8-OBS-002)
producer: state-manager (orchestrator-coordinated; PO + story-writer + state-manager stages)
factory_shas: [4ed96e06, 0f126bbe, TBD-this-commit]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2"
next_action: "Adversary pass-10 dispatch — target streak 0/3 → 1/3 if CLEAN"
---

# Fix-Burst-8 Closure Report — S-PLUGIN-PREREQ-D

## §Closures

| Finding | Severity | Closure Agent | Closure SHA | Evidence / File Changes | Status |
|---------|----------|---------------|-------------|-------------------------|--------|
| F-LP9-MEDIUM-001 — Catalog Additions instructs amending BC-2.16.002 but 6 of 7 events lie outside BC-2.16.002's declared PipelineExecutor scope | MEDIUM | product-owner (Path B adjudication) | 4ed96e06 | BC-2.16.002 v1.10→v1.11: scope broadened from "PipelineExecutor and pipeline.rs helpers" to canonical universal catalog covering all `prism-spec-engine` + `prism-bin` boot-step plugin-load `event_type` sites. Catalog header renamed "Canonical Structured Event Catalog (v1.11)". 7 new rows added: `plugin_load_unsigned`, `plugin_load_disabled_via_envvar`, `plugin_load_failed_manifest_no_allowed_urls`, `plugin_load_failed_format_version_exceeded`, `plugin_load_failed_wit_invalid`, `plugin_http_request_blocked`, `pipeline_max_requests_exceeded`. Total catalog rows 16→23. BC-2.22.001 unchanged (delegation "per BC-2.16.002" correct under broadened scope). BC-INDEX v4.69 row annotated v1.11. Story-writer stage 2 (0f126bbe): Catalog Additions preamble synced to BC-2.16.002 v1.11 Path B wording; 5 metadata corrections (3 emitter/Level for TD-VSDD-091 compliance + 2 trigger prose alignment). | CLOSED |
| F-LP9-LOW-001 — AC-9 body line 373 version-vs-burst contradiction ("Closed by BC-2.17.002 v1.5 amendment (fix-burst-6)" but v1.5 is fix-burst-7 lifecycle-only; substantive 30s timeout at v1.4 fix-burst-6) | LOW | story-writer (Form A edit) | 0f126bbe | Story v1.7→v1.8: line 373 corrected to "Closed by BC-2.17.002 v1.4 amendment (fix-burst-6 substantive); current pinned version v1.5 (fix-burst-7 lifecycle_status-only sweep)." No other content changes required; substantive anchor (BC-2.17.002 line 77 30s timeout text) unaffected. | CLOSED |

## §Deferred Findings

| Finding | Severity | Routing | Rationale |
|---------|----------|---------|-----------|
| F-LP9-OBS-001 — Pattern: version-pin sweep without sibling-prose burst-vs-version distinction `[process-gap]` | OBS | cycle-closing checklist (codification candidate) | Recurrent process-gap (2nd instance this cycle: F-LP8-MEDIUM-002 was the first instance). Not a content defect — no document is wrong, no implementer is mislead. Codification candidate: when a BC version-bumps from a metadata-only burst, downstream references should distinguish "current pinned version" from "version that authored the substantive content." Routed alongside F-LP8-OBS-002 (lifecycle_status-drift-pattern) to session-reviewer post-cycle. |

## §Path B Adjudication Rationale

Pass-9 F-LP9-MEDIUM-001 presented three paths:

- **Path A (new BC-2.17.008 Plugin Structured Event Catalog):** Most architecturally clean — dedicated BC per subsystem. Rejected: creates a new BC ID under Subsystem 17, requires BC-INDEX and STORY-INDEX updates, adds a new entity that will be superseded when plugin BCs go active via POL-14. The 7 events cross 3 subsystems (SS-22 boot, SS-17 plugin runtime, SS-16 spec-engine), so a single-subsystem BC would itself be mis-scoped.

- **Path B (expand BC-2.16.002 scope to universal catalog — all `prism-spec-engine` + `prism-bin` boot-step emissions):** Lightest-touch. PG-LP11-001 architectural intent was explicitly "universal catalog" — the narrow pipeline.rs scope statement in BC-2.16.002 v1.10 was a drafting artifact from when BC-2.16.002 only described PipelineExecutor. Broadening the scope statement and adding 7 rows is a pure additive amendment. No new BC IDs. BC-2.22.001 v1.5 delegation "per BC-2.16.002" becomes correctly scoped. **CHOSEN.**

- **Path C (scatter — anchor each event to closest-domain BC):** Violates single-source-of-truth for `event_type` registry. Adversary (pass-8 + pass-9) already demonstrated that cross-BC `event_type` routing causes confusion. Rejected per PG-LP11-001 "universal catalog" codified SOP.

PO's Path B choice aligns with PG-LP11-001's architectural intent as recorded at D-419 and codified in BC-2.16.002 v1.9/v1.10/v1.11 changelog.

## §Verification Rederivation

Placeholder for pass-10 adversary fresh-context verification. Pass-10 will verify:
1. BC-2.16.002 v1.11 scope statement accurately covers all 7 new `event_type` rows' emission sources.
2. 7 new catalog rows have correct emitter function-name anchors (TD-VSDD-091), Level (WARN/INFO/ERROR), and trigger prose.
3. Story v1.8 line 373 Form A fix is internally consistent with BC-2.17.002 changelog (v1.4 substantive, v1.5 lifecycle-only).
4. No regressions from the 5 pass-8 confirmed closures (lifecycle_status sweep, WARN clarification, AC-9 trace header, ADR-022 step 7.5).
5. BC-INDEX v4.70 and STORY-INDEX v2.75 accurately reflect amendments.

## §Process-Gap Codifications (cycle-closing checklist)

Three recurrent process-gap candidates accumulated during PREREQ-D cascade. Routed to cycle-closing checklist for session-reviewer codification:

1. **adversary-cannot-write-reports** (3rd consecutive pass — pass-7 + pass-8 + pass-9): Structural tool-profile constraint. Adversary dispatched with read-only profile cannot write files. State-manager reifies pass reports after every adversary pass (Standing Rule 1). This workaround is working but the underlying constraint should be codified in vsdd-factory plugin documentation. Codification target: TD-VSDD-005 (already open) + session-reviewer post-cycle recommendation.

2. **lifecycle_status-drift-pattern** (from F-LP8-OBS-002): Recurrent pattern where BC files authored during a "sweep" burst (e.g., pre-build Wave-6 sweep, fix-burst-6) set `lifecycle_status: active` prematurely before the implementing story's PR merges via POL-14. Root cause: sweep scripts or manual BC edits do not check POL-14 merge status. Codification target: pre-burst checklist item — "verify `lifecycle_status` matches BC-INDEX `status` column before any BC amendment."

3. **version-pin-sweep burst-vs-version-prose distinction** (from F-LP9-OBS-001): When a BC version-bumps from a metadata-only burst (e.g., `lifecycle_status: active→draft` sweep), downstream references that pin to the current version (e.g., "BC-2.17.002 v1.5") need a clarifying phrase: "v1.5 (fix-burst-7 lifecycle-only; substantive content at v1.4 fix-burst-6)." This prevents readers from seeing "v1.5" and expecting to find the substantive amendment there. Codification target: story-writing SOP — when amending an AC closure note to pin a newer BC version, check the BC changelog to determine whether the new version is substantive or metadata-only, and add the disambiguation phrase.

## §Next Action

Pass-10 dispatch. Target: streak 0/3 → 1/3 if CLEAN.

Per pass-9 adversary prediction: 0–1 finding expected if Path B execution was clean. Mostly OBS-class risk (catalog metadata precision). Convergence within reach.

After 3-CLEAN convergence: test-writer → implementer → pr-manager 9-step PR lifecycle → squash-merge → PLUGIN-MIGRATION Wave 1 unblock.
