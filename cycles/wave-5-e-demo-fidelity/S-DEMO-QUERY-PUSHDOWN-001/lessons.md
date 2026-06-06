# Lessons Learned — S-DEMO-QUERY-PUSHDOWN-001 (wave-5-e-demo-fidelity)

## Codification Candidates

### [recurrence] ADV-P05-HIGH-001 — Hand-fed-FQL-vs-real-path test anti-pattern: when fixing one hand-fed test, sweep ALL push-down tests across ALL crates (TD-VSDD-060)

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1012
**Finding:** ADV-P05-HIGH-001 (v2.x LOCAL pass-5)
**Tags:** [recurrence] [sibling-sweep] [TD-VSDD-060] [dead-code-via-test-layer]
**Classification:** BLOCKING — recurrence of a previously closed defect class; exhaustive sweep is the required response.

**Description:**
Pass 4 caught the hand-fed-FQL + direct-`PipelineExecutor::execute`-boundary test anti-pattern for AC-CWS-002 (in `prism-bin`). The pass-4 fix correctly addressed AC-CWS-002 but did not exhaustively sweep sibling tests across other crates in `crates_touched`. Pass 5 found the same defect class at AC-EQUIV-001 in `prism-spec-engine` — the test claimed "via real materialization path" in its name but bypassed `run_materialization_pipeline`.

This is a **cross-crate sibling-sweep gap**: the pass-4 fix-burst operated within `prism-bin` and did not review the `prism-spec-engine` AC-EQUIV-001 test, which was testing the same behavioral property (result equivalence via real path) but living in a different crate.

**Root cause of recurrence:**
The pass-4 AC-CWS-002 fix-burst swept within `prism-bin` but treated the fix as crate-scoped. The AC-EQUIV-001 test in `prism-spec-engine` existed independently and was NOT reviewed as a sibling despite both tests covering the same "real materialization path" behavioral domain (BC-2.11.007 result-equivalence).

**Correct response (codified rule):**
When fixing a hand-fed-FQL / direct-`PipelineExecutor::execute`-boundary test anti-pattern, the fix-burst MUST:
1. Sweep ALL push-down tests across ALL `crates_touched` (not just the crate where the defect was found).
2. Explicitly list and disposition each test: real-path vs boundary vs unit (per the 32-test sweep pattern used in pass 5).
3. Any test whose NAME claims "via real path" / "via run_materialization_pipeline" / "via production entry point" but whose BODY does NOT call that function = P1 HIGH finding; fix in the same burst.

**Outcome:** Pass-5 fix-burst added exhaustive sibling-sweep of 32 push-down tests across 4 crates (TD-VSDD-060 compliant). All 32 dispositioned. No further misnamed real-path claims remain.

**Codification direction (for future session-reviewer / VSDD process improvement):**
- This pattern should become a standing adversary probe: after any fix involving `run_materialization_pipeline` test rewrites, adversary MUST grep for ALL tests containing `"run_materialization_pipeline"` in their name or doc-comment across the full workspace, then verify each drives the function, not just PipelineExecutor directly.
- Codification could be SAP-3 or an amendment to SAP-1/SAP-2 scope (TD-VSDD-060 sweep extension).

---

### [recurrence][codification-candidate] Test-strength gap class — assertion weaker than AC/doc claim (passes 4/5/6 pattern)

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1013
**Findings:** ADV-P04-HIGH-002 (pass 4 — AC-CWS-002), ADV-P05-HIGH-001 (pass 5 — AC-EQUIV-001), ADV-P06-MED-001 (pass 6 — AC-CWS-003)
**Tags:** [recurrence] [test-strength-gap] [wire-level-assertion] [codification-candidate] [dead-code-via-test-layer]
**Classification:** CODIFICATION CANDIDATE — three consecutive passes found the same defect class; systematic test-writing discipline gap.

**Description:**
Three consecutive adversary passes (4, 5, 6) each found a push-down AC test whose assertion was weaker than its AC/doc claim:

- **Pass 4 — AC-CWS-002:** Test claimed "time-window both-bounds via run_materialization_pipeline" but hand-fed pre-built FQL to `PipelineExecutor::execute`, bypassing the production entry point. Wire-level DTU filter-log assertion absent.
- **Pass 5 — AC-EQUIV-001:** Test claimed "result equivalence via real materialization path" but called `PipelineExecutor::execute` directly (sibling-sweep miss from the pass-4 fix). Real-path coverage absent for BC-2.11.007 subset/no-fabrication invariant.
- **Pass 6 — AC-CWS-003:** Test name and AC claimed "DTU filter-log shows absence of `created_timestamp` clause when no time filter present, and all 50 records returned." Test only asserted `!is_empty()` and non-zero count. Wire-level absence assertion absent — if production code incorrectly injected a timestamp clause, test would still pass.

In all three cases, production code was correct at discovery time. The test-strength gap meant incorrect production behavior would not have been caught by the named test.

**Common root cause:**
Test-writer and implementer wrote structural proxies (non-empty, count > 0) or boundary-only assertions instead of wire-level evidence (DTU filter-log body, specific field presence/absence, exact count matching the fixture). The proxy assertion passes when production is correct AND when production incorrectly injects/omits the named property — it is not a falsifiable test for the named AC property.

**Codification direction (for future test-writer + implementer disciplines):**

1. **Wire-level assertion requirement for push-down AC tests:** Any AC test for a push-down property MUST assert the named property via DTU wire evidence (e.g., `/dtu/filter-log` body contains or does not contain the specific clause/field), not via structural proxies.
2. **Exact fixture count assertion:** When an AC claims "all N records returned" (AC-CWS-003: 50 records), the test MUST assert `result.len() == N` not `!result.is_empty()`.
3. **Doc-claim-vs-assertion audit at fix-burst time:** When a fix-burst touches any push-down AC test, the implementer MUST audit the assertion against the AC text and test name/doc-comment before declaring the closure load-bearing. This audit should be explicitly documented in the fix-burst commit message.
4. **Adversary standing probe extension (codification candidate):** The exhaustive doc-claim-vs-assertion audit performed in pass 6 (comparing all push-down AC test assertions against their AC text) should become a standing adversary probe for all push-down story passes. Candidate for SAP-3 or SAP-2 extension.

**Outcome (pass 6):** Pass-6 fix-burst added AC-CWS-003 wire-level absence assertion (DTU `/dtu/filter-log` body checked for absence of `created_timestamp` clause; `result.len() == 50` exact count). Exhaustive doc-claim-vs-assertion audit of all push-down AC tests confirmed all others CORRECT/load-bearing after the fix.

---

### [correctness] ADV-P08-MED-001 — Push-down must over-fetch at inclusive boundaries (Ge/Le): DTU filtering must use strict-outside exclusion, never inclusive exclusion

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1014
**Finding:** ADV-P08-MED-001 (v2.x LOCAL pass-8)
**Tags:** [correctness] [push-down] [boundary-semantics] [result-equivalence] [BC-2.11.007]
**Classification:** BLOCKING — correctness defect; BC-2.11.007 result-equivalence invariant violated.

**Description:**
Pass 8 found that inclusive time predicates (`>=`/`<=`, `CompareOp::Ge`/`Le`) caused push-down to UNDER-fetch boundary records. Two root causes:

1. **DTU inclusive-boundary semantics (CrowdStrike `detections.rs` + Armis `search.rs`):** The DTU filtering functions (`device_in_time_window`, `alert_in_time_window`) used inclusive-boundary exclusion — a record with `timestamp == bound` was excluded. DataFusion applies inclusive semantics for `>=`/`<=`. This asymmetry dropped exact-boundary records when push-down was active.

2. **RFC3339 `+00:00` vs `Z` lexicographic comparison bug:** `chrono::to_rfc3339()` emits `+00:00` (ASCII `+` = 43); fixture timestamps stored as `Z` (ASCII `Z` = 90). Lexicographic comparison `+00:00` < `Z` caused DataFusion's string-based comparison to drop the boundary record even when both strings represent the same UTC instant.

**Codified rules:**

1. **DTU filtering MUST over-fetch at boundaries.** When push-down generates a time-window filter for inclusive predicates (`>=`/`<=`), the DTU clone's filtering function MUST exclude only strictly-outside records (`ts < start` or `ts > end`). Records at the exact boundary (`ts == start` or `ts == end`) MUST pass through the DTU filter. DataFusion's post-filter applies the correct inclusive semantics and narrows the result set. This is the "over-fetch-never-under-fetch" DTU invariant for push-down.

2. **RFC3339 normalization is mandatory.** Any code path that serializes timestamps for push-down into DTU fixture filtering or DataFusion string comparison MUST use `to_rfc3339_opts(SecondsFormat::Secs, true)` to produce `Z`-suffix form. `to_rfc3339()` (`+00:00` form) must NOT be used for DTU push-down filter serialization — it produces a lexicographically-wrong string that drops exact-boundary records at DataFusion's string-comparison layer.

3. **BC-2.11.007 result-equivalence requires boundary Red Gate tests.** Any story implementing time-window push-down for a new sensor MUST include at least one boundary Red Gate test that: (a) uses an exact `>=` or `<=` predicate, (b) runs via `run_materialization_pipeline` (not `PipelineExecutor::execute` directly), and (c) asserts the boundary record IS included in the result (i.e., the push-down result count matches the non-pushed count at the boundary).

**Outcome:** Fix-burst `69aafcc7` corrected both root causes. 2 new boundary Red Gate tests added (CrowdStrike + Armis). Story v2.4→v2.5 (EC-009 + `red_gate_tests` 16→18). just check 4035/4035 PASS.

---

### [correctness] RFC3339 `+00:00` vs `Z` suffix — DTU push-down timestamps must normalize to `Z` form to prevent lexicographic boundary drop

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1014
**Finding:** ADV-P08-MED-001 root cause 2 (v2.x LOCAL pass-8)
**Tags:** [correctness] [rfc3339] [timestamp-normalization] [lexicographic-comparison] [push-down]
**Classification:** BLOCKING (co-root-cause of ADV-P08-MED-001); codified as standing rule.

**Description:**
`chrono::DateTime::to_rfc3339()` produces `2024-01-01T00:00:00+00:00`. Fixture timestamps are stored as `2024-01-01T00:00:00Z`. In lexicographic string ordering, `+` (ASCII 43) < `Z` (ASCII 90). DataFusion uses string-based comparison for timestamp predicates in this push-down path. The result: a `>=` predicate with the `+00:00` form evaluates `"2024-01-01T00:00:00+00:00" >= "2024-01-01T00:00:00Z"` as `false` (since `+` < `Z`), dropping the boundary record.

Both `2024-01-01T00:00:00+00:00` and `2024-01-01T00:00:00Z` represent the identical UTC instant. The bug is purely a string-representation asymmetry.

**Codified rule:**
In all push-down timestamp serialization sites, use `chrono::SecondsFormat::Secs` with `use_z: true`:
```rust
dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)  // → "2024-01-01T00:00:00Z" (CORRECT)
// NOT:
dt.to_rfc3339()  // → "2024-01-01T00:00:00+00:00" (WRONG for string comparison)
```
This rule applies to: DTU fixture comparison sites, DataFusion predicate generation, any push-down filter string serialization in `pipeline.rs` or equivalent.

---

### [process-gap] ADV-P04-OBS-001 — Test-docstring file.rs:NNN line-pin discipline gap (TD-VSDD-091 adjacency)

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1011
**Finding:** ADV-P04-OBS-001 (v2.x LOCAL pass-4)
**Tags:** [process-gap] [codification-candidate] [TD-VSDD-091-adjacency]
**Classification:** NOT blocking — process improvement candidate only.

**Description:**
Several test function doc-comments in `crates/prism-dtu-crowdstrike/` and `crates/prism-query/src/pushdown.rs` cite source-file locations as `file.rs:NNN` (line-number pins). TD-VSDD-091 (anti-volatile-pin discipline) prohibits line-number pins in narrative spec content because they decay on subsequent diffs. The rule is currently enforced by adversary pass probes for `.factory/` SPEC artifacts.

However, there is no automated enforcement path for test-docstring line-number pins inside `crates/**/*.rs` files. A test's `/// See file.rs:123` style comment is technically in-code narrative — it can decay the same way a spec pin does — but it falls outside the current TD-VSDD-091 adversary probe scope and no pre-commit lint exists for it.

**Codification direction (for future session-reviewer / factory process improvement):**
- Option A: Extend adversary SAP probe to grep `crates/**/*.rs` doc-comments for `\w+\.rs:\d+` pattern and flag as LOW findings.
- Option B: Add a clippy custom lint or a pre-commit hook regex scan for `\w+\.rs:\d+` inside `///` comment lines.
- Option C: Record as CLAUDE.md standing convention and address opportunistically during fix-bursts that touch doc-comments.

**Resolution:** Option C is the minimum bar; Option A is the right permanent fix. Codification decision deferred to session-reviewer at cycle-close. Do NOT add a new story now — this is a process/lint gap, not an unmet user-facing requirement.

**Non-blocking confirmation:** The line-pin sites do not affect test behavior or correctness. Code passes `just check 4032/4032`. This record exists only as a cycle-close codification prompt.
