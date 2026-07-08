---
document_type: adversarial-review
scope: PR-LEVEL
passes: [5]
story: S-PRISMQL-CASE-INSENSITIVE-001
pr: 217
feature_head_at_review: fab7df00
base_develop_head: 7b1f6c51
closure_head: 36a094d6
date: 2026-07-08
clean_strict: false
clean_pr_merge: true
finding_counts:
  OBS: 3
  total: 3
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay from adversary pass-5 output
---
# PR-LEVEL Adversarial Review — Pass 5
## S-PRISMQL-CASE-INSENSITIVE-001

**Frozen HEAD:** fab7df00 (feature/S-PRISMQL-CASE-INSENSITIVE-001)
**Base:** develop@7b1f6c51
**Date:** 2026-07-08
**Authored by:** orchestrator-relay from adversary pass-5 output

---

## Verdict

| Criterion | Result |
|-----------|--------|
| CLEAN (strict) | **no** |
| CLEAN (PR-merge) | **yes** |

**Finding summary:** 3 OBS. Zero CRIT, HIGH, MED, LOW, PROCESS-GAP.

**Novelty:** LOW — OBS-001 is a known Unicode control-char scope class (CWE-117 variant); OBS-002 is an error-swallowing latent-bypass class; OBS-003 is academic Unicode casefold asymmetry.

**Streak status:** 0/3 — streak RESET (was 1/3 after pass-4; pass-5 is not CLEAN(strict)). Fix-burst @36a094d6 closes OBS-001 and OBS-002; OBS-003 is lessons-note only per adjudication below.

---

## Findings

### ADV-PR-P5-OBS-001 — `sanitize_for_log` strips only ASCII controls; Unicode C1 (U+0080–U+009F) and line/paragraph separators (U+2028/U+2029) pass through

**Severity:** OBS (upgraded to MEDIUM by security-reviewer triage; see `security-reviews/pr-217-c1-control-triage.md`)
**CWE:** CWE-117 — Improper Output Neutralization for Logs
**Files:** `crates/prism-core/src/sanitize.rs` (or equivalent), `crates/prism-bin/src/spec_driven_adapter.rs`, `crates/prism-ocsf/src/normalizer.rs`

**Description:** `prism_core::sanitize_for_log` filters ASCII control characters (0x00–0x1F + 0x7F) and the ASCII DEL byte but does not filter:

- **Unicode C1 control characters (U+0080–U+009F):** These include CSI (U+009B), which enables ANSI escape-sequence injection in terminals consuming raw Unicode from journald or similar structured log consumers. The C1 range is a well-known ANSI-extension attack surface.
- **Unicode line separator (U+2028) and paragraph separator (U+2029):** These function as newline equivalents in JavaScript engines and some log ingestion systems (NEL injection in log parsers that split on U+2028/29). They are also a log-spoofing vector in LLM-agent consumption contexts where the log stream is fed as structured context to an LLM — a U+2028/29 in a `value` or `sensor_type` field could introduce a false newline that the LLM treats as a field boundary.

**Blast-radius inventory (TD-VSDD-060 sweep at fab7df00):**
- PRIMARY: `spec_driven_adapter.rs` `build_column_array` — `ocsf.enum_label_unrecognized` warn event `value` + `sensor_type` fields
- SECONDARY: `normalizer.rs` `normalize_with_mappers` — `ocsf.enum_label_unrecognized` warn event same fields
- `infusion.coercion_failed` `truncated_value` field — same sanitize_for_log call per BC-2.16.002 row 91 annotation
- `connectivity.rs` `sanitize_error` function — codebase precedent for the widened scope (already strips C1 + line separators per prior PRs)

**Precedent:** `connectivity.rs` `sanitize_error` already strips C1 + U+2028/29, establishing the widened scope as the project standard. The `sanitize_for_log` implementation in `prism-core` predates that widening; this is an asymmetry gap, not a new decision.

**Adjudication:** Security-reviewer upgraded OBS→MEDIUM (CWE-117). Fix: widen `sanitize_for_log` to `!is_control() && ch != '\u{2028}' && ch != '\u{2029}'` (where `is_control()` covers both ASCII 0x00–0x1F/0x7F and Unicode Cc category including C1 U+0080–U+009F). BC-2.16.002 row 91 `value`/`sensor_type` + `infusion.coercion_failed` `truncated_value` field descriptions updated accordingly. error-taxonomy `sanitize_for_log` rendering notes updated.

**Closure note:** @36a094d6 (implementer). `sanitize_for_log` widened to `char::is_control() || ch == '\u{2028}' || ch == '\u{2029}'` filter; RG-082 `test_rg082_sanitize_for_log_strips_unicode_cc_and_line_separators` RED→GREEN (Unicode Cc U+0085 next-line + U+0091 private-use-one + U+2028 + U+2029 all stripped; ASCII letters/digits preserved). BC-2.16.002 v2.05→v2.06 (row 91 field descriptions widened; `infusion.coercion_failed` `truncated_value` field extended identically). error-taxonomy v2.19→v2.20 (E-INFUSE-013/014 rendering notes cite `prism_core::sanitize_for_log` widened scope). Story v1.35→v1.36 (story-writer; RGT 81→83; BC-2.16.002 pin v2.06 + error-taxonomy v2.20 propagated with per-class POL-29 evidence). just check 5319/5319 GREEN; non-exhaustive 89/89.

---

### ADV-PR-P5-OBS-002 — `check_ci_column_types` swallowed `Err` from `SchemaProvider::table()`; latent AC-022 bypass post async-schema-migration

**Severity:** OBS
**Category:** Silent failure / error-swallowing (Standing Rule 3 §2)
**File:** `crates/prism-spec-engine/src/spec_driven_adapter.rs` (or equivalent location of `check_ci_column_types`)

**Description:** `check_ci_column_types` calls `SchemaProvider::table()` to retrieve the DataFusion table schema for the sensor being queried. The call site uses a pattern equivalent to:

```rust
if let Some(schema) = schema_provider.table(table_name).await.ok() {
    // validate column types
}
```

The `.ok()` converts an `Err` from `SchemaProvider::table()` into a `None`, which causes the function to silently return `Ok(())` — skipping all CI column-type validation — instead of propagating the error. This is the AC-022 bypass: when the schema provider is in a transient error state (registration lag after async sensor reload, DataFusion catalog not-yet-populated, etc.), CI queries against that sensor would pass `check_ci_column_types` without validation and proceed to execution.

**Context:** At the time `check_ci_column_types` was written, `SchemaProvider::table()` was synchronous and infallible. A later async-schema-migration PR made it `async` and `Result`-returning. The call site was mechanically updated for the new signature but the error-handling semantics were not audited — `.ok()` was added to silence the compiler, creating a latent bypass.

**Adjudication:** OBS retained (not upgraded). The bypass is latent: the current `SchemaProvider::table()` implementation does not return `Err` in practice for in-memory sensors. However, per Standing Rule 3 §2 (no silent `Vec::new()` return where partial-failure data should propagate), the `.ok()` must be replaced with explicit error propagation. Fix: propagate the `SchemaProvider::table()` error via `?` and surface as `QueryExecutionFailed`.

**Closure note:** @36a094d6 (implementer). `check_ci_column_types` call site updated: `.ok()` removed; error propagated via `?` into `SpecEngineError::QueryExecutionFailed { reason }`. RG-083 `test_rg083_check_ci_column_types_err_schema_provider_propagates` RED→GREEN (mock `ErrSchemaProvider` returns `Err`; `check_ci_column_types` returns `QueryExecutionFailed`; not silently `Ok(())`). 9-site TD-VSDD-060 comment sweep across 5 files (adjacent call sites verified for equivalent `.ok()` patterns; none found). just check 5319/5319 GREEN; non-exhaustive 89/89.

---

### ADV-PR-P5-OBS-003 — `to_ascii_lowercase` in `OcsfEnumMap::normalize_enum_label` vs DataFusion `lower()` Unicode casefold asymmetry

**Severity:** OBS (lessons-note only; no fix required per adjudication)
**Category:** Academic asymmetry / latent Unicode edge case
**File:** `crates/prism-ocsf/src/enum_map.rs`

**Description:** `OcsfEnumMap::normalize_enum_label` uses `str::to_ascii_lowercase()` when building the lookup key for the enum map, while DataFusion's `lower()` SQL function uses Unicode-aware case folding (via ICU or Rust's `to_lowercase()`). These two operations are identical for all-ASCII strings (the universal case for OCSF enum labels in v1.7.0) but diverge for non-ASCII characters — e.g., `Σ` (U+03A3 GREEK CAPITAL LETTER SIGMA) lowercases to `σ` under Unicode but is unchanged by `to_ascii_lowercase()`.

**Blast-radius:** Zero in practice. OCSF v1.7.0 contains zero non-ASCII enum label captions. The divergence is purely theoretical for the current OCSF version set.

**Adjudication:** No code fix required. The existing behavior is correct for all supported OCSF versions. This finding is a lessons-note candidate only — if OCSF introduces non-ASCII captions in a future version, the asymmetry would need to be resolved (the fix would be switching `to_ascii_lowercase()` to `to_lowercase()` in `normalize_enum_label`). Recorded in lessons.md as a flag for OCSF version upgrade review.

**No closure action required.** Lessons note appended to `cycles/wave-5-e-demo-fidelity/lessons.md`.

---

## Probe Results

### SAP-1 — Tracing emission catalog completeness

**Result: CLEAN** (with one D-765-class note)

All `event_type =` sites verified at fab7df00. No new `event_type =` sites introduced by the story. The implementer @36a094d6 fix-burst added a bare `tracing::error!` at the `check_ci_column_types` error propagation site (OBS-002 closure). This bare `tracing::error!` carries no `event_type` field; per D-765 precedent, error-propagation `?` operator companion traces are exempt from catalog row obligation. No new catalog row required.

### SAP-2 — DTU↔TOML schema parity

**Result: N/A** — this story does not modify `.prism/specs/sensors/*.toml` or DTU clone route/type files.

### POL-22 — Phase A+C gates

**Result: CLEAN** — Phase A (story frontmatter completeness, v1.36 post-closure) and Phase C (BC traceability, all 8 BCs present including BC-2.16.002 v2.06 + error-taxonomy v2.20 pins) both verified clean at closure HEAD 36a094d6.

### CWE-117 — Log injection order at PRIMARY+SECONDARY

**Result: CLEAN both sites** — RG-079 (SECONDARY load-bearing helper test) and RG-080 (PRIMARY order-of-operations vector test with sensor_type mirror) both GREEN at fab7df00. OBS-001 closure @36a094d6 widened `sanitize_for_log` scope; both sites continue to call `sanitize_for_log` before the 50-codepoint truncation cap per BC-2.16.002 v2.06 row 91. RG-082 confirms the widened scope.

### Paper-fix audit

**Result: none detected** — OBS-001 closure is load-bearing (RG-082 new test, RED→GREEN; sanitize_for_log production code changed). OBS-002 closure is load-bearing (RG-083 new test, RED→GREEN; error propagation path changed from swallow to propagate). OBS-003 is lessons-note only — no code change, no paper-fix pattern applicable.

---

## Convergence Trajectory (PR-LEVEL)

| Pass | Frozen HEAD | CLEAN(strict) | CLEAN(PR-merge) | Findings | Streak |
|------|------------|---------------|-----------------|----------|--------|
| 1    | a2fc8940   | no            | no              | 2 MED + 2 LOW + 2 OBS (total 6) | 0/3 reset |
| 2    | 1172b15a   | no            | yes             | 1 LOW (total 1)                 | 0/3 (push resets) |
| 3    | dcb37099   | no            | yes             | 2 OBS (total 2)                 | 0/3 (push resets) |
| 4    | fab7df00   | yes           | yes             | 0 (total 0)                     | 1/3 |
| 5    | fab7df00   | **no**        | yes             | 3 OBS (total 3)                 | **0/3 RESET** |

---

## Post-Pass Action

Implementer @36a094d6 closed ADV-PR-P5-OBS-001 (sanitize_for_log widened; RG-082 RED→GREEN) and ADV-PR-P5-OBS-002 (SchemaProvider::table() error propagated; RG-083 RED→GREEN). ADV-PR-P5-OBS-003 lessons-note only — no code change. BC-2.16.002 v2.05→v2.06 (product-owner). error-taxonomy v2.19→v2.20 (product-owner). Story v1.35→v1.36 (story-writer; RGT 81→83). 9-site TD-VSDD-060 comment sweep completed. just check 5319/5319 GREEN; non-exhaustive 89/89. Feature HEAD fab7df00→36a094d6 LOCAL-ONLY.

**VERY NEXT ACTION:** Push feature HEAD 36a094d6 to origin → new frozen HEAD → PR-LEVEL adversary pass-6 on new frozen HEAD. Per DRIFT-ORCH-PRLEVEL-PUSH-001, the push resets the streak to 0/3. If passes 6/7/8 are all CLEAN(strict) on unchanged HEAD → 3-CLEAN CONVERGED → pr-manager squash-merge + post-merge burst (POL-14: BC-2.11.024 + BC-2.02.013 draft→active).
