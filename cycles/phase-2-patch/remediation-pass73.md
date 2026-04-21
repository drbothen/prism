---
document_type: remediation-manifest
pass: 73
date: 2026-04-20
agent: state-manager
commit: pending
---

# Pass-73 Deterministic Remediation Manifest

## Summary

Pass-72 PO class audit produced false-clean signal — reported 18 BCs fixed, but 132 BCs still had
non-monotonic changelog order. Pass-73 used a deterministic bash script (not agent self-report) to
detect and fix all violations.

- **BCs reordered:** 132
- **BCs clean (unmodified):** 71
- **BCs total checked:** 203
- **Post-run violations:** 0
- **Verification method:** Python script checking descending order after bash run

---

## Task 1 (CRIT-001): Deterministic changelog reorder

### Script

Path: `.factory/cycles/phase-2-patch/scripts/reorder-bc-changelogs.sh`

Logic:
1. For each `BC-*.md` in `specs/behavioral-contracts/`
2. Locate `## Changelog` section
3. Parse header row + data rows
4. Sort data rows by version tuple (major.minor) descending
5. If order differs, rewrite section using Python replace (safe for special chars)
6. Track modified files

Post-script: Python version-bump script ran across all 132 git-diff'd files:
- Incremented frontmatter `version:` minor field
- Inserted `| <new_version> | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder... |` at top of changelog data rows

### Modified BC Files (132 total)

| # | File | Action |
|---|------|--------|
| 1 | BC-2.01.004-offset-based-pagination-claroty.md | reordered + bumped |
| 2 | BC-2.01.005-crowdstrike-oauth2-two-step-fetch.md | reordered + bumped |
| 3 | BC-2.01.006-cyberint-cookie-auth.md | reordered + bumped |
| 4 | BC-2.01.007-claroty-bearer-polymorphic-ids.md | reordered + bumped |
| 5 | BC-2.01.008-armis-bearer-aql.md | reordered + bumped |
| 6 | BC-2.01.013-datasource-trait-adapter-pattern.md | reordered + bumped |
| 7 | BC-2.01.014-sensor-api-http-503-mid-pagination.md | reordered + bumped |
| 8 | BC-2.02.001-ocsf-schema-build-time-loading.md | reordered + bumped |
| 9 | BC-2.02.002-dynamic-message-creation.md | reordered + bumped |
| 10 | BC-2.02.003-crowdstrike-field-mapping.md | reordered + bumped |
| 11 | BC-2.02.004-cyberint-field-mapping.md | reordered + bumped |
| 12 | BC-2.02.005-claroty-field-mapping.md | reordered + bumped |
| 13 | BC-2.02.006-armis-field-mapping.md | reordered + bumped |
| 14 | BC-2.02.007-raw-extensions-preservation.md | reordered + bumped |
| 15 | BC-2.02.008-field-alias-resolution.md | reordered + bumped |
| 16 | BC-2.02.009-ocsf-version-pinning.md | reordered + bumped |
| 17 | BC-2.02.010-enum-value-map-runtime-lookup.md | reordered + bumped |
| 18 | BC-2.02.011-normalization-error-handling.md | reordered + bumped |
| 19 | BC-2.02.012-ocsf-event-class-selection.md | reordered + bumped |
| 20 | BC-2.03.001-credential-store-trait.md | reordered + bumped |
| 21 | BC-2.03.002-keyring-backend.md | reordered + bumped |
| 22 | BC-2.03.003-encrypted-file-fallback.md | reordered + bumped |
| 23 | BC-2.03.004-namespace-isolation.md | reordered + bumped |
| 24 | BC-2.03.006-credential-resolution-at-query-time.md | reordered + bumped |
| 25 | BC-2.03.007-secret-redaction.md | reordered + bumped |
| 26 | BC-2.03.008-credential-name-sanitization.md | reordered + bumped |
| 27 | BC-2.03.009-resolve-secret-env-file.md | reordered + bumped |
| 28 | BC-2.03.010-credential-access-audit-logging.md | reordered + bumped |
| 29 | BC-2.03.011-keyring-startup-probe.md | reordered + bumped |
| 30 | BC-2.03.012-backend-selection-fallback.md | reordered + bumped |
| 31 | BC-2.04.003-hierarchical-flag-resolution.md | reordered + bumped |
| 32 | BC-2.04.010-confirmation-token-consumption.md | reordered + bumped |
| 33 | BC-2.04.014-tools-list-changed-notification.md | reordered + bumped |
| 34 | BC-2.05.001-audit-entry-per-tool-invocation.md | reordered + bumped |
| 35 | BC-2.05.011-audit-forwarding-at-least-once.md | reordered + bumped |
| 36 | BC-2.06.009-tools-list-changed-on-client-switch.md | reordered + bumped |
| 37 | BC-2.07.001-ephemeral-cursor-pagination.md | reordered + bumped |
| 38 | BC-2.07.002-pagination-token-lifecycle.md | reordered + bumped |
| 39 | BC-2.07.003-response-cache-ttl.md | reordered + bumped |
| 40 | BC-2.07.005-cache-key-derivation.md | reordered + bumped |
| 41 | BC-2.07.006-cache-memory-bounds-eviction.md | reordered + bumped |
| 42 | BC-2.08.001-on-demand-connectivity-check.md | reordered + bumped |
| 43 | BC-2.08.002-auth-validity-check.md | reordered + bumped |
| 44 | BC-2.08.003-rate-limit-state-detection.md | reordered + bumped |
| 45 | BC-2.08.004-last-successful-query-timestamp.md | reordered + bumped |
| 46 | BC-2.08.005-health-mcp-tool.md | reordered + bumped |
| 47 | BC-2.08.007-partial-health-status.md | reordered + bumped |
| 48 | BC-2.08.008-get-diagnostics-tool.md | reordered + bumped |
| 49 | BC-2.08.009-diagnostic-resource-templates.md | reordered + bumped |
| 50 | BC-2.09.001-structural-separation.md | reordered + bumped |
| 51 | BC-2.09.002-provenance-framing.md | reordered + bumped |
| 52 | BC-2.09.005-trust-level-metadata.md | reordered + bumped |
| 53 | BC-2.09.006-tool-description-security-warnings.md | reordered + bumped |
| 54 | BC-2.09.007-output-schema-type-safety.md | reordered + bumped |
| 55 | BC-2.09.008-response-envelope-trust-annotations.md | reordered + bumped |
| 56 | BC-2.10.001-server-handler-implementation.md | reordered + bumped |
| 57 | BC-2.10.003-conditional-tool-registration.md | reordered + bumped |
| 58 | BC-2.10.006-stdio-transport.md | reordered + bumped |
| 59 | BC-2.10.007-structured-error-responses.md | reordered + bumped |
| 60 | BC-2.10.008-mcp-resources.md | CRIT-002 gap-close (separate fix; see below) |
| 61 | BC-2.10.009-mcp-prompts.md | reordered + bumped |
| 62 | BC-2.10.010-graceful-shutdown.md | reordered + bumped |
| 63 | BC-2.10.011-list-capabilities-meta-tool.md | reordered + bumped |
| 64 | BC-2.11.001-query-mcp-tool.md | reordered + bumped |
| 65 | BC-2.11.002-prismql-filter-mode.md | reordered + bumped |
| 66 | BC-2.11.003-prismql-sql-mode.md | reordered + bumped |
| 67 | BC-2.11.004-prismql-pipe-mode.md | reordered + bumped |
| 68 | BC-2.11.005-ephemeral-materialization.md | reordered + bumped |
| 69 | BC-2.11.006-query-security-limits.md | reordered + bumped |
| 70 | BC-2.11.007-sensor-filter-push-down.md | reordered + bumped |
| 71 | BC-2.11.008-create-alias-tool.md | reordered + bumped |
| 72 | BC-2.11.009-alias-resolution.md | reordered + bumped |
| 73 | BC-2.11.010-explain-query-tool.md | reordered + bumped |
| 74 | BC-2.11.011-cross-client-query-scoping.md | reordered + bumped |
| 75 | BC-2.11.012-virtual-fields.md | reordered + bumped |
| 76 | BC-2.11.013-list-aliases-tool.md | reordered + bumped |
| 77 | BC-2.11.014-delete-alias-tool.md | reordered + bumped |
| 78 | BC-2.11.015-explain-alias-tool.md | reordered + bumped |
| 79 | BC-2.12.002-list-schedules-tool.md | reordered + bumped |
| 80 | BC-2.12.003-delete-schedule-tool.md | reordered + bumped |
| 81 | BC-2.12.004-schedule-execution-loop.md | reordered + bumped |
| 82 | BC-2.12.005-differential-result-computation.md | reordered + bumped |
| 83 | BC-2.12.006-epoch-counter-tracking.md | reordered + bumped |
| 84 | BC-2.12.007-get-diff-results-tool.md | reordered + bumped |
| 85 | BC-2.12.008-pack-loading-discovery.md | reordered + bumped |
| 86 | BC-2.12.009-pack-crud-tools.md | reordered + bumped |
| 87 | BC-2.12.010-schedule-state-persistence.md | reordered + bumped |
| 88 | BC-2.13.001-detection-rule-loading.md | reordered + bumped |
| 89 | BC-2.13.002-single-event-detection.md | reordered + bumped |
| 90 | BC-2.13.003-correlation-detection.md | reordered + bumped |
| 91 | BC-2.13.004-sequence-detection.md | reordered + bumped |
| 92 | BC-2.13.005-alert-generation.md | reordered + bumped |
| 93 | BC-2.13.007-list-rules-tool.md | reordered + bumped |
| 94 | BC-2.13.008-delete-rule-tool.md | reordered + bumped |
| 95 | BC-2.13.009-rule-to-sql-compilation.md | reordered + bumped |
| 96 | BC-2.13.010-security-udf-registration.md | reordered + bumped |
| 97 | BC-2.13.011-three-scope-rule-resolution.md | reordered + bumped |
| 98 | BC-2.13.012-detection-state-persistence.md | reordered + bumped |
| 99 | BC-2.13.013-alert-deduplication.md | reordered + bumped |
| 100 | BC-2.13.014-ioc-file-loading-pattern-store.md | reordered + bumped |
| 101 | BC-2.14.001-create-case-tool.md | reordered + bumped |
| 102 | BC-2.14.002-case-state-transitions.md | reordered + bumped |
| 103 | BC-2.14.003-update-case-tool.md | reordered + bumped |
| 104 | BC-2.14.004-list-cases-tool.md | reordered + bumped |
| 105 | BC-2.14.005-get-case-tool.md | reordered + bumped |
| 106 | BC-2.14.006-disposition-assignment.md | reordered + bumped |
| 107 | BC-2.14.007-timeline-annotations.md | reordered + bumped |
| 108 | BC-2.14.008-mttd-mttr-computation.md | reordered + bumped |
| 109 | BC-2.14.009-case-persistence.md | reordered + bumped |
| 110 | BC-2.14.010-case-metrics-tool.md | reordered + bumped |
| 111 | BC-2.14.012-acknowledge-alert.md | reordered + bumped |
| 112 | BC-2.14.013-auto-case-creation.md | reordered + bumped |
| 113 | BC-2.15.001-rocksdb-initialization.md | reordered + bumped |
| 114 | BC-2.15.002-domain-kv-operations.md | reordered + bumped |
| 115 | BC-2.15.003-buffered-audit-log-persistence.md | reordered + bumped |
| 116 | BC-2.15.004-audit-buffer-overflow.md | reordered + bumped |
| 117 | BC-2.15.005-crash-recovery-dirty-bits.md | reordered + bumped |
| 118 | BC-2.15.006-resource-watchdog-initialization.md | reordered + bumped |
| 119 | BC-2.15.007-watchdog-query-termination.md | reordered + bumped |
| 120 | BC-2.15.008-query-denylisting.md | reordered + bumped |
| 121 | BC-2.15.009-context-decorator-injection.md | reordered + bumped |
| 122 | BC-2.15.010-decorator-three-phase-model.md | reordered + bumped |
| 123 | BC-2.15.011-internal-table-registration.md | reordered + bumped |
| 124 | BC-2.16.001-sensor-spec-file-loading.md | reordered + bumped |
| 125 | BC-2.16.002-multi-step-fetch-pipeline.md | reordered + bumped |
| 126 | BC-2.16.003-column-to-ocsf-mapping.md | reordered + bumped |
| 127 | BC-2.16.004-rust-escape-hatch.md | reordered + bumped |
| 128 | BC-2.16.005-reload-config-tool.md | reordered + bumped |
| 129 | BC-2.16.006-arc-swap-config-access.md | reordered + bumped |
| 130 | BC-2.16.007-sensor-spec-hot-reload.md | reordered + bumped |
| 131 | BC-2.16.008-add-sensor-spec-tool.md | reordered + bumped |
| 132 | BC-2.16.009-spec-file-validation.md | reordered + bumped |
| 133 | BC-2.16.010-list-sensor-specs-tool.md | reordered + bumped |

Note: BC-2.10.008 appears in the list above as CRIT-002 gap-close (different fix; the reorder script
left it untouched since its rows were already descending; the v1.4 gap was a separate issue).

**Post-run verification:**

```
Total BCs checked: 203
Violations remaining: 0
Clean: 203
```

---

## Task 2 (CRIT-002): BC-2.10.008 missing v1.4 fix

**File:** `specs/behavioral-contracts/BC-2.10.008-mcp-resources.md`

**Problem:** Version gap — changelog had 1.6, 1.5, 1.3, 1.2, 1.1, 1.0 (v1.4 missing).

**Fix approach:** Renumber + new row (cleaner than inserting a gap note):
- Old v1.5 row → renamed to v1.4 (with note: "originally recorded as v1.5; renumbered by pass-73-fix")
- Old v1.6 row → renamed to v1.5 (with note: "originally recorded as v1.6; renumbered by pass-73-fix")
- New v1.6 row added at top: gap-close note documenting the renumber and original conflation

**Before rows:** 1.6, 1.5, 1.3, 1.2, 1.1, 1.0 (gap at 1.4)

**After rows:** 1.6, 1.5, 1.4, 1.3, 1.2, 1.1, 1.0 (no gaps)

**Frontmatter version:** stays at "1.6" (new v1.6 row is the latest after renumber)

**Verification:** Python check: `['1.6', '1.5', '1.4', '1.3', '1.2', '1.1', '1.0']` — CLEAN, no gaps.

---

## Task 3 (HIGH-002): INDEX.md + burst-log.md pass-72/73 status update

**INDEX.md changes:**
- pass-72 review: IN-PROGRESS → COMPLETE (5 findings, commit e3b313c)
- pass-72 remediation: IN-PROGRESS → COMPLETE (26 files, commit e3b313c)
- pass-73 review: new row, IN-PROGRESS
- pass-73 remediation: new row, IN-PROGRESS (will flip to COMPLETE on commit)

**burst-log.md changes:**
- Pass 72 Review status: IN-PROGRESS → COMPLETE (commit e3b313c); findings corrected to 5 (1C/2H/2M/1L)
- Pass 72 Remediation status: IN-PROGRESS → COMPLETE (commit e3b313c); description expanded with CRIT-001 details + false-clean retroactive note
- Pass 73 Review: new section added, IN-PROGRESS
- Pass 73 Remediation: new section added, IN-PROGRESS (pending commit)

---

## Task 4: STATE.md fields changed

| Field | Before | After |
|-------|--------|-------|
| `current_step` | "pass-72 remediation landed ... pass-73 pending" | "pass-73 deterministic remediation: 132 BCs..." |
| `convergence_status` | `PATTERN_DECAY_PENDING_PASS_73_OR_HOOK_INSTALL` | `PATTERN_RECURRING_DETERMINISTIC_REMEDIATION_APPLIED` |
| `bc_changelog_monotonicity_deterministic_fix_applied` | (absent) | `2026-04-20` |
| `recent_passes_summary` | ends at "p72:5 counter 0/3 (class audit...)" | extended with "p73 deterministic-reorder..." |
| Phase Progress row | `PASS-73-PENDING` | `PASS-73-ADVERSARIAL-PENDING` + trajectory updated |
| Current Phase Steps | pass-72 rows IN-PROGRESS | pass-72 COMPLETE + pass-73 deterministic IN-PROGRESS row added |
| Session Resume Checkpoint heading | PASS-72 REMEDIATED; COUNTER 0/3; PASS-73 PENDING | PASS-73 DETERMINISTIC REMEDIATION APPLIED; COUNTER 0/3; PASS-73 ADVERSARIAL REVIEW PENDING |
| Session Resume Checkpoint body | describes housekeeping/pass-72 state | prepends pass-73 lesson-learned paragraph; last commit updated to pending |
| Pass-72 paragraph | describes pass-72 findings | adds retroactive note about false-clean; pass-73 paragraph added |

---

## Task 5 (HIGH-001 — DEFERRED)

S-1.15 changelog burst-vs-version coherency issue requires story-writer judgment to determine
canonical history. Deferred to Phase 3 backlog or pass-74 work item.

Documented in STATE.md Session Resume Checkpoint.

---

## Commit

**SHA:** pending (commit to be made after manifest written)

**Branch:** factory-artifacts

**Files:** 136 total
- 132 BC files (reordered changelog + version bump + pass-73-fix row)
- 1 BC file (BC-2.10.008 v1.4 gap close)
- INDEX.md (pass-72 COMPLETE + pass-73 rows)
- burst-log.md (pass-72 COMPLETE + pass-73 sections)
- STATE.md (convergence_status + new field + checkpoint update)
- remediation-pass73.md (this file)
- scripts/reorder-bc-changelogs.sh (bash script for auditability)
