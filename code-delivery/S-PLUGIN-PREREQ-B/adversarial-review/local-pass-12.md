---
pass: 12
story: S-PLUGIN-PREREQ-B
head_sha: 6e436d65
base_sha: 90d7c80f
factory_sha_at_pass: 9bc21533
verdict: BLOCKED-soft
streak_target: 1/3
streak_actual: 0/3
findings_summary: 0C / 0H / 1M / 2L / 3O (+ 1 process-gap)
adversary_run_date: 2026-05-11
novelty_score: 6/6 = 1.0
---

# Adversarial Review — LOCAL Pass 12 (S-PLUGIN-PREREQ-B)

## Executive Summary

Fix-burst-11 closures verified per TD-VSDD-059. BC v1.8 14-row Structured Event Catalog is byte-accurate to implementation (all 14 rows' field-schemas verified against tracing macros). 7 truncate_at_char_boundary_tests are load-bearing. PG-LP11-001 SOP exists in D-419.

However pass-12 surfaces 1 MED + 2 LOW + 3 OBS that block streak:

1. **F-LP12-MED-001 — BC catalog enumerates 3 execute_step rows whose code has ZERO test or production triggers.** Contract surface grew via v1.8 amendment without commensurate test coverage. execute_step has been dead since fix-burst-6 (OBS-LP8-003 acknowledged) but the v1.8 amendment promoted its emissions to contract surface.

2. **F-LP12-LOW-001 — Stale BC v1.5 reference in pipeline.rs:461 comment.** BC is now v1.8. Single-site occurrence; volatile version pin equivalent to TD-VSDD-091 line-number anti-pattern.

3. **F-LP12-LOW-002 [process-gap] — PG-LP11-001 SOP codification is shallow.** Lives only in STATE.md D-419. Fix-burst-11 closure report claimed "Recorded permanently in cycle lessons; orchestrator dispatch templates will include this reminder" — neither artifact exists. TD-VSDD-058 documents STATE.md compaction has lost decision rows before.

## Part A — Closure Verification of fix-burst-11

### F-LP11-MED-001 + F-LP11-MED-002 (BC v1.8 catalog) — CLEAN, load-bearing

Audited all 14 rows. Each field-schema byte-matches the tracing macro emissions in pipeline.rs:
- auth_initial × execute (lines 146-171) — 2 fields, no step_name — MATCH
- auth_initial × execute_step (lines 469-497) — 3 fields with step_name — MATCH (closes F-LP11-MED-002)
- auth_refresh × issue_request_with_retry (lines 630-688) — sensor_id+client_id+step_name (+detail on _failed) — MATCH
- pipeline_truncated (line 363) — sensor_id+client_id+step_name+max_records+accumulated — MATCH
- pagination_cursor_unsupported_type (line 902) — cursor_path+actual_type+cursor_preview — MATCH
- fanout_invalid_source_type (line 999) — step_name+var_name+actual_type — MATCH
- fanout_ambiguous_multi_array (line 1025) — step_name+array_vars_count+first_var+other_vars — MATCH

POL-7 H1 unchanged. POL-8 story arrays unchanged. TD-VSDD-091 no volatile line pins.

product-owner's `detail` vs `error` field-name correction is load-bearing — a grep at amendment-time would have failed if mis-named.

### F-LP11-LOW-001 (truncate_at_char_boundary tests) — CLEAN, load-bearing

7 tests at validation.rs:621-667 use assert_eq! on exact expected outputs. Regression modes covered:
- Ellipsis addition would FAIL exact-string assertions
- Cow<str> return would FAIL type signature
- Empty-input panic would FAIL both empty-input tests
- Off-by-one would FAIL boundary tests

Untested edges flagged as OBS-LP12-001: max_chars=usize::MAX, single-multi-byte at max=1.

### PG-LP11-001 codification — PARTIALLY CLEAN (D-419 exists; templates and lessons file don't)

D-419 contains operative SOP language. But:
- Glob `.factory/cycles/wave-4-operations/lessons*.md` → 0 files
- No orchestrator/implementer prompt template updated
- TD-VSDD-058 precedent: STATE.md compaction has lost rows D-214..D-320 before

See F-LP12-LOW-002.

## Part B — New Dimension Findings

### F-LP12-MED-001 — BC v1.8 catalog includes execute_step trio whose 3 emissions have ZERO test or production triggers

- Severity: MEDIUM | Confidence: HIGH | Dimension: P12-B+P12-D
- Evidence:
  - BC v1.8 catalog rows for execute_step variants documented as contract surface (lines 81-83)
  - pipeline.rs:453 defines pub async fn execute_step; lines 469-498 emit the three events
  - Test grep: Grep('execute_step', tests/) → 0 direct call sites (only documentation comments)
  - Production grep: Grep('PipelineExecutor::execute_step', workspace) → 0 callers
  - TD-S-PLUGIN-PREREQ-B-012 acknowledges execute_step dead-until-PREREQ-D
- The v1.8 amendment promoted execute_step's auth-acquire stanza into documented contract surface without promoting it into testable coverage. Future refactor could silently drift field-schema with no test to catch.
- This is novel vs OBS-LP8-003: OBS-LP8-003 noted execute_step has zero callers (was OBS because BC didn't enumerate execute_step emissions). v1.8 promoted them to contract → now the gap is load-bearing.
- **Recommendation:** Pre-emptive test anchoring (3 unit tests in pipeline.rs #[cfg(test)] mod tests) invoking execute_step with MockAuthProvider/NullAuthProvider/FailingAuthProvider and asserting structured event emissions. ~60 lines. Converts 3 contract-only rows into 3 test-anchored rows.

### F-LP12-LOW-001 — Stale BC version reference in pipeline.rs:461 comment

- Severity: LOW | Confidence: HIGH | Dimension: P12-G (BC version drift in code)
- pipeline.rs:461 says `// Eager token acquisition: symmetric with PipelineExecutor::execute (BC-2.16.002 v1.5).` BC is v1.8.
- Volatile version pin equivalent to TD-VSDD-091 line-number anti-pattern.
- Sibling sweep: Grep('BC-2.16.002 v', crates/prism-spec-engine) returns ONLY this one site.
- **Recommendation:** Change `(BC-2.16.002 v1.5)` to `(BC-2.16.002 — see Structured Event Catalog)`. Removes version pin entirely.

### F-LP12-LOW-002 — [process-gap] PG-LP11-001 SOP codification shallow

- Severity: LOW | Confidence: HIGH | Dimension: P12-A (codification durability)
- fix-burst-11.md claimed: "Recorded permanently in cycle lessons; orchestrator dispatch templates for fix-burst will include this reminder going forward"
- Reality: no cycle lessons file exists at .factory/cycles/wave-4-operations/; no template artifact updated; only D-419 in STATE.md carries the rule.
- TD-VSDD-058 precedent: project has lost D-214..D-320 to STATE.md compaction. SOP at risk of forgetting.
- **Recommendation:** Create .factory/cycles/wave-4-operations/lessons.md with PG-LP11-001 entry OR file TD-VSDD-092 P2 at engine governance layer.
- [process-gap] tag: gap in process/tooling codification, not content defect.

## Observations (non-blocking)

### OBS-LP12-001 — truncate_at_char_boundary edge gaps

- Missing: max_chars=usize::MAX, ("🎯", 1) single-multi-byte at max=1
- Non-blocking; bundle into PREREQ-C/D test sweep

### OBS-LP12-002 — BC v1.3→v1.8 in 11 bursts is a stability signal

- 5 amendment cycles on one BC in one cycle-day. Each justified — but rate worth tracking.
- Expect convergence: v1.8 was comprehensive sweep; fix-burst-12 unlikely to amend BC further. If pass-13 finds catalog drift, recurrence=3 escalates to HIGH.

### OBS-LP12-003 — AuthProvider trait doc-comments don't cross-reference v1.8 catalog

- auth_provider.rs:89-100 docstring doesn't mention catalog. BC is source of truth via separate citation; cross-ref would be nicety, not contract gap.

## Findings Table

| ID | Severity | Dimension | Category | Summary | Actionable |
|----|----------|-----------|----------|---------|-----------|
| F-LP12-MED-001 | MEDIUM | P12-B+D | spec-surface paper-fix | 3 BC catalog rows have zero test/production triggers | YES |
| F-LP12-LOW-001 | LOW | P12-G | code comment drift | pipeline.rs:461 cites BC v1.5 (current v1.8) | YES |
| F-LP12-LOW-002 | LOW+[process-gap] | P12-A | SOP shallow | PG-LP11-001 only in D-419; compaction risk | YES |
| OBS-LP12-001 | OBS | P12-A | test coverage edges | usize::MAX + single-multi-byte at max=1 | NO |
| OBS-LP12-002 | OBS | P12-A | BC stability signal | 5 amendments in 11 bursts; monitor | NO |
| OBS-LP12-003 | OBS | P12-K | trait doc cross-ref | AuthProvider doesn't cite catalog | NO |

## Process-Gap Findings

F-LP12-LOW-002 tagged [process-gap]. Codification-shallowness pattern, recurrence=1 (first occurrence). Watch for repeat in fix-burst-12 if a new PG emerges and is similarly codified D-only.

## Recommendations

### Fix-burst-12 scope (REQUIRED for streak 0/3 → 1/3)

1. F-LP12-MED-001: Add 3 unit tests in pipeline.rs #[cfg(test)] mod tests for execute_step (MockAuthProvider/NullAuthProvider/FailingAuthProvider) asserting structured event emissions
2. F-LP12-LOW-001: One-line edit at pipeline.rs:461 removing `v1.5` pin
3. F-LP12-LOW-002 [process-gap]: Create .factory/cycles/wave-4-operations/lessons.md OR file TD-VSDD-092 at engine governance layer

### Deferred (TD already filed or out of scope)

- OBS-LP12-001..003: defer per recommendations above.

After fix-burst-12, dispatch pass-13. Target: streak 0/3 → 1/3.

## Novelty Self-Check

6/6 = 1.0. All findings on dimensions never previously examined.

**Verdict: BLOCKED-soft. Streak 0/3. Fix-burst-12 required.**
