---
document_type: adversarial-review
scope: PR-LEVEL
passes: [1]
story: S-PRISMQL-CASE-INSENSITIVE-001
pr: 217
feature_head_at_review: a2fc8940
base_develop_head: 7b1f6c51
fix_burst_head: 1172b15a
date: 2026-07-08
clean_strict: false
clean_pr_merge: false
finding_counts:
  MED: 2
  LOW: 2
  OBS: 2
  total: 6
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay from adversary pass-1 output
---

# PR-LEVEL Adversary Pass 1 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 1 (frozen a2fc8940; base develop@7b1f6c51 post-RUSTSEC PR #218; PR #217; streak candidate 1/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO
**Findings:** 6 (2 MED + 2 LOW + 2 OBS)
**Code HEAD at review:** a2fc8940 (frozen PR HEAD; incorporates develop@7b1f6c51 RUSTSEC-2026-0204 crossbeam-epoch fix + pre-PR-LEVEL fix-burst @54c89898 code; pushed post-RUSTSEC-develop-merge)
**Fix-burst HEAD:** 1172b15a (test-writer @56fb83d8 → implementer @f9be96fa → demo-recorder @1172b15a; LOCAL-ONLY at time of this record)
**PR-LEVEL 3-CLEAN(strict) streak after pass-1:** 0/3 (NOT CLEAN; fix-burst dispatched; push resets per DRIFT-ORCH-PRLEVEL-PUSH-001)

---

## Finding ADV-PR-P1-MED-001 — Spec-Code Drift: sanitize/truncate order at 5 warn sites vs BC-2.16.002 v2.05 row 91

**Severity:** MED
**Classification:** spec-code drift / POL-4 / SAP-1
**Affected files:** `crates/prism-spec-engine/src/adapters/spec_driven_adapter.rs` (5 warn sites in `build_column_array`)
**BC reference:** BC-2.16.002 v2.05 catalog row 91 (`ocsf.enum_label_unrecognized`)

**Finding:** BC-2.16.002 v2.05 catalog row 91 documents the `ocsf.enum_label_unrecognized` event emitting sanitized + truncated values. The catalog row establishes the canonical field contract: `sanitize_for_log(value)` is applied BEFORE the 50-codepoint cap truncation — producing safe, bounded output. At 5 warn sites in `build_column_array`, the implementation applied truncation FIRST then `sanitize_for_log`, inverting the spec-mandated order. This is a spec-code drift defect: the field values emitted by these 5 sites did not match the documented post-conditions.

**Note on infusion_udf.rs:** The `infusion.coercion_failed` emit site in `infusion_udf.rs` intentionally truncates before sanitizing per its OWN catalog row contract (a different event type with a different postcondition). This was verified NOT to be in scope for this finding — different event-type, different contract.

**Routing:** implementer
**Closure:** CLOSED — implementer commit f9be96fa: 5 sites in `build_column_array` reordered to `sanitize_for_log` first then truncation cap. 2 inline comments added clarifying the order requirement. SAP-1 re-pass confirmed all 5 sites now match BC-2.16.002 v2.05 row 91 postcondition.

---

## Finding ADV-PR-P1-MED-002 — RG-078 Paper-Fix (SID-1 / TD-VSDD-059)

**Severity:** MED
**Classification:** SID-1 / TD-VSDD-059 paper-fix detection
**Affected test:** `crates/prism-spec-engine/src/...` RG-078 test

**Finding:** RG-078 (added in pre-PR-LEVEL fix-burst @54c89898 for CR-004/SEC-001) was classified as a paper-fix: the test verified the presence of `sanitize_for_log` at the call-site level (testing that the function existed at the call site) but did NOT load-bearingly verify that the sanitized output differed from unsanitized input for adversarial inputs containing CWE-117 control characters. This is the SID-1 pattern: an `#[ignore]`-free test that does not actually exercise the behavioral contract. TD-VSDD-059 mandates that adversary independently verifies every claimed closure has a load-bearing test, not merely a doc-comment or rename.

**Routing:** test-writer (for load-bearing replacement), then implementer (existing code already correct; test is the fix)
**Closure:** CLOSED — test-writer commit 56fb83d8: RG-079 added as a load-bearing helper test directly exercising `sanitize_for_log` on adversarial CWE-117 inputs (control characters, CRLF sequences) and asserting the sanitized output differs measurably from raw input. RG-078 OLD test kept (not deleted) but re-pointed to `sanitize_enum_label_for_log` to verify that helper correctly delegates to `sanitize_for_log`. Combined: RG-078 (re-pointed) + RG-079 (load-bearing) together satisfy TD-VSDD-059 for this call site.

---

## Finding ADV-PR-P1-LOW-001 — Order-of-Operations Vector Gap

**Severity:** LOW
**Classification:** test coverage gap / behavioral correctness
**Affected area:** `build_column_array` / `sanitize_for_log` invocation sequencing

**Finding:** While the spec-code drift (ADV-PR-P1-MED-001) covered 5 sites, there was a related order-of-operations vector gap: no test verified that a value containing BOTH a truncatable-length portion AND CWE-117 control characters would be correctly handled when sanitize-then-truncate ordering was applied versus truncate-then-sanitize. Without this test, a future regression could silently reintroduce the inverted order without tripping any existing tests.

**Routing:** test-writer, implementer
**Closure:** CLOSED — test-writer commit 56fb83d8: RG-080 added as a RED test exercising the combined order-of-operations contract (value with embedded control chars + exceeding 50-codepoint threshold). Implementer commit f9be96fa: RG-080 turned GREEN with the 5-site reorder already applied for ADV-PR-P1-MED-001.

---

## Finding ADV-PR-P1-LOW-002 — Evidence Report Stale HEAD

**Severity:** LOW
**Classification:** documentation accuracy / ADV-PR-P1 evidence sync
**Affected file:** `docs/demo-evidence/S-PRISMQL-CASE-INSENSITIVE-001/evidence-report.md`

**Finding:** The evidence report referenced feature HEAD a71b8912 (the pre-RUSTSEC-develop-merge pushed HEAD) rather than the updated frozen PR HEAD a2fc8940 (which incorporates the develop@7b1f6c51 RUSTSEC merge + pre-PR-LEVEL fix-burst @54c89898). The Cluster J evidence section also did not reflect the 74→78 RGT count update. Evidence reports must be synchronized to the current frozen PR HEAD for the adversary pass to be reproducible.

**Routing:** demo-recorder
**Closure:** CLOSED — demo-recorder commit 1172b15a: evidence report synced to feature HEAD f9be96fa (@1172b15a; updated to show the post-fix-burst state). Cluster J evidence section updated to reflect RGT 74→81 after the fix-burst. Full evidence report re-synchronized.

---

## Finding ADV-PR-P1-OBS-002 — SuggestedSuffix Display Lock Missing

**Severity:** OBS
**Classification:** test coverage / Display contract lock
**Affected type:** `SuggestedSuffix` Display impl

**Finding:** `SuggestedSuffix` (from `E-QUERY-002` error taxonomy) had a Display implementation that formatted suffix strings, but no test locked the Display output against accidental future changes. Display output for error taxonomy types is BC-sensitive: a change to the formatted string would silently break any downstream consumer that pattern-matches on error message text.

**Routing:** test-writer
**Closure:** CLOSED — test-writer commit 56fb83d8: RG-081 added as a GREEN lock test verifying the exact `Display` output of `SuggestedSuffix` variants matches the BC-documented format strings. Test runs GREEN on a2fc8940 and post-fix-burst HEAD.

---

## Finding ADV-PR-P1-OBS-001 — Informational: SECONDARY Path Zero Production Callers

**Severity:** OBS (informational only; not a defect)
**Classification:** code health observation / not actionable in this cascade
**Affected area:** SECONDARY normalization path in `crates/prism-ocsf/src/normalizer.rs`

**Finding (informational):** The SECONDARY normalization path in `normalizer.rs` (the `normalize_with_mappers` function) has zero production call sites outside of tests. This was noted as a potential future cleanup target. However, this path was explicitly codified as a deliberate design choice in BC-2.02.013 v1.3 §F-CRIT-002: the SECONDARY path exists as an extension point to allow pluggable normalizer composition, even though it is not yet wired in the production adapter chain.

**Decision D-1602:** The SECONDARY path zero-production-callers status is KNOWN and ACCEPTABLE per BC-2.02.013 v1.3 F-CRIT-002. Future removal of the SECONDARY path requires a BC amendment to BC-2.02.013 first (removing F-CRIT-002 or updating its rationale). This is NOT a bug and NOT actionable in this cascade. Recorded here for completeness and audit trail.

**Routing:** n/a (informational; no action required)
**Closure:** ACKNOWLEDGED — no code change. Decision D-1602 records the BC-2.02.013 F-CRIT-002 anchor as the authoritative justification for the SECONDARY path's existence.

---

## Standing Probe Results

### SAP-1 — Tracing Emission Catalog Completeness

**Result:** PASS

Grep of `event_type =` across `crates/` workspace at frozen a2fc8940 identified 1 new event_type added by this story: `ocsf.enum_label_unrecognized` (row 91 in BC-2.16.002 catalog). Catalog row 91 was authored in BC-2.16.002 v2.05 (pre-PR-LEVEL fix-burst @54c89898). All emission sites for `ocsf.enum_label_unrecognized` have matching catalog rows. SAP-1 PASS.

### SAP-2 — DTU↔TOML Schema Parity

**Result:** N/A — this story does not modify `.prism/specs/sensors/*.toml` or DTU clone source files.

### POL-22 Phase A+C

**Result:** PASS — Phase A (production-grade defaults, no shortcut patterns) and Phase C (spec-code alignment) both verified at pass-1.

---

## Invariant Ledger Summary

| Invariant | Status |
|-----------|--------|
| SAP-1 (tracing emission catalog) | PASS |
| SAP-2 (DTU↔TOML parity) | N/A |
| SID-1 (no-ignored-test rationalization) | PASS (RG-079 load-bearing; RG-078 re-pointed) |
| TD-VSDD-059 (paper-fix detection) | PASS (ADV-PR-P1-MED-002 caught + CLOSED via RG-079) |
| TD-VSDD-060 (sibling-site sweep) | PASS (5-site sweep was exhaustive per implementer) |
| TD-VSDD-091 (no volatile line-number pins) | PASS |
| POL-22 Phase A | PASS |
| POL-22 Phase C | PASS |
| non-exhaustive gate 89/89 | PASS (unchanged) |
| just check 5317/5317 GREEN | PASS (post-fix-burst) |

---

## Novelty Assessment

**Novelty: MEDIUM**

ADV-PR-P1-MED-002 (paper-fix detection on a sanitize call-site) represents a novel pattern — the prior LOCAL cascade closed SEC-001 by adding `sanitize_for_log`, but the test confirming the call was not load-bearing against adversarial inputs. This is a distinct failure mode from the prior LOCAL cascade findings and warrants the MEDIUM novelty rating.

---

## Post-Fix-Burst State

| Field | Value |
|-------|-------|
| Fix-burst commits | 56fb83d8 → f9be96fa → 1172b15a |
| Feature HEAD after fix-burst | 1172b15a (LOCAL-ONLY at time of record) |
| Red Gate count after fix-burst | 81 (RGT 74→81: +3 from this pass via RG-079/080/081; +4 from D-1601 via RG-075/076/077/078) |
| workspace_test_count | 5317 (just check 5317/5317 GREEN) |
| non-exhaustive gate | 89/89 PASS |
| story version | v1.34 (story-writer; RGT 78→81; RG-078 annotation re-pointed) |
| BC-INDEX | v7.55 (unchanged) |
| STORY-INDEX | v2.633 (this burst) |
| PR-LEVEL streak | 0/3 (push to origin resets per DRIFT-ORCH-PRLEVEL-PUSH-001) |
| Very next action | push feature HEAD 1172b15a → new frozen HEAD → PR-LEVEL pass-2 |
