---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-07T00:00:00
phase: maintenance
inputs:
  - PR #131 diff (6 files)
  - crates/prism-query/src/lib.rs
  - crates/prism-query/src/sql_parser.rs
  - crates/prism-query/src/tests/bc_gap_fill_tests.rs
  - crates/prism-query/src/tests/integration_tests.rs
  - crates/prism-query/src/tests/parser_tests.rs
  - tests/external/perimeter-violation/src/main.rs
  - .factory/specs/behavioral-contracts/BC-2.11.006-query-security-limits.md
input-hash: "[not-computed]"
traces_to: BC-2.11.006
pass: 55
previous_review: pass-54.md
scope: PR #131 diff (maintenance/clippy-unwrap-cleanup)
---

# Adversarial Review: PR #131 — clippy unwrap_used cleanup + delete dead build_dml_parser (Pass 55)

## Scope Note

This is a targeted PR diff review (not a full-corpus pass). The adversary reviewed the 6
changed files plus BC-2.11.006 as authoritative spec context. The review examines the
five adversary lenses specified in the dispatch: (1) hidden behavioral changes in test
files; (2) build_dml_parser deletion correctness; (3) perimeter docstring sync; (4)
doctest references; (5) version number drift.

## Finding ID Convention

Finding IDs use: `ADV-W3MT-P55-<SEV>-<SEQ>`

## Part A — Fix Verification (pass >= 2 only)

Not applicable — this is a PR diff review, not a convergence pass continuing from a prior pass in this cycle.

## Part B — New Findings

### CRITICAL

*None.*

### HIGH

#### ADV-W3MT-P55-HIGH-001: Perimeter-violation crate silently missing 5 S-3.04 alias symbols vs. BC-2.11.006 v1.17

- **Severity:** HIGH
- **Category:** spec-fidelity / security-surface
- **Location:** `tests/external/perimeter-violation/src/main.rs` vs. BC-2.11.006 v1.17 `restricted_symbols`
- **Description:** BC-2.11.006 is currently at v1.17 (S-3.04 closure). Version 1.17 added five alias-system symbols to `restricted_symbols`: `alias_tools::create_alias`, `alias_tools::create_alias_with_clients`, `alias_tools::create_alias_with_clients_gated_inner`, `alias_tools::delete_alias`, `alias_store::AliasStore::create_or_update`. The spec's total perimeter list is 31 entries generating 32 expected E-errors. The PR branch perimeter-violation crate has 23 active `use prism_query::` imports (none covering alias symbols). The PR's own docstring claims "9 new restricted symbols, BC-2.11.006 v1.16" with total implied at 27 E-errors, not 32.
- **Evidence:**
  - `BC-2.11.006-query-security-limits.md` frontmatter `version: "1.17"`, with 31 items in `symbols:` list (verified by count: 26 pre-S-3.04 + 5 alias symbols).
  - `tests/external/perimeter-violation/src/main.rs` (PR branch): 23 active `use prism_query::` imports — no alias symbols.
  - BC changelog v1.17: "Expected E-errors in perimeter-violation crate: 27→32"
  - `perimeter-violation/src/main.rs` line 1: `//! BC-2.11.006 v1.16` — version is stale vs. v1.17 (though the docstring reflects the last change to the S-3.06 layer-4 block).
- **Proposed Fix:** This finding is mitigated if S-3.04 has not yet been merged to `develop` (alias symbols don't yet exist as `pub(crate)` in the compiled crate). **Verify:** does `crates/prism-query/src/alias_tools.rs` exist on the PR branch? If S-3.04 code is absent, the perimeter-violation test cannot import non-existent symbols — the spec is ahead of the code and the test correctly omits them until the S-3.04 PR merges. If S-3.04 code IS present, the 5 missing imports are a perimeter coverage hole. Resolution: before merging PR #131, confirm S-3.04 is not merged to develop.

#### ADV-W3MT-P55-HIGH-002: lib.rs docstring version anchor lags BC-2.11.006 current version by 1 minor revision

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `crates/prism-query/src/lib.rs:114`
- **Description:** The PR updates the lib.rs docstring from `v1.14` → `v1.16` for the Write-parser internals paragraph. However BC-2.11.006 is currently at v1.17. The lib.rs docstring now says "BC-2.11.004 + BC-2.11.006 v1.16 DI-034 layer 4" when the authoritative spec is at v1.17.
- **Evidence:**
  - `BC-2.11.006-query-security-limits.md` frontmatter: `version: "1.17"` (S-3.04 layer-5 closure).
  - `crates/prism-query/src/lib.rs` (PR branch) line 114: `BC-2.11.004 + BC-2.11.006 v1.16 DI-034 layer 4`
  - v1.17 is in the .factory spec, which lives on the factory-artifacts branch (not git-tracked on develop). The v1.17 bump was made during S-3.04 worktree work.
- **Mitigating context:** The lib.rs comment is documenting when the *DI-034 layer-4 (S-3.06) symbols* were added, not a claim that the BC is currently at v1.16. The prose only covers the S-3.06 write-parser group, not S-3.04 aliases. However a reader will interpret "v1.16" as the current version, which is misleading.
- **Proposed Fix:** Either (a) update the version anchor to `v1.17` to match the current spec, or (b) add a parenthetical clarifying this pin is the S-3.06 layer-4 epoch: `(v1.16 for this layer; current BC version v1.17 adds S-3.04 alias symbols — see alias_tools.rs)`. Option (a) is simpler and correct.

### MEDIUM

#### ADV-W3MT-P55-MED-001: integration_tests.rs has mixed .unwrap() and .expect("msg") — inconsistent style within file

- **Severity:** MEDIUM
- **Category:** code-quality
- **Location:** `crates/prism-query/src/tests/integration_tests.rs`
- **Description:** The PR rewrites some `.unwrap()` calls to `.expect("descriptive msg")` form, but leaves other `.unwrap()` calls in the same file. The `#![allow(clippy::unwrap_used, clippy::expect_used)]` suppresses both lints so the file compiles, but the result is an inconsistent internal style: some call sites have human-readable failure context (.expect) while others don't (.unwrap). The PR's own stated intent is "rewrites .unwrap() calls to .expect("descriptive msg") form" but many raw `.unwrap()` calls remain.
- **Evidence:** Lines 72, 85, 103, 121, 126, 127, 223, 249, 252, 263, 278, 498, 504, 515 — all raw `.unwrap()` on the PR branch.
- **Proposed Fix:** Either (a) convert all remaining .unwrap() to .expect("descriptive msg") to fulfill the PR's stated intent; or (b) revise the PR description to accurately state the scope ("partial conversion — critical paths converted; remainder suppressed via #![allow]"). This is not a correctness defect but creates technical debt.

#### ADV-W3MT-P55-MED-002: bc_gap_fill_tests.rs — .ok_or("&str") on Option<&Arrow StringArray> changes semantics vs. prior .expect()

- **Severity:** MEDIUM
- **Category:** coverage-gap
- **Location:** `crates/prism-query/src/tests/bc_gap_fill_tests.rs:682`
- **Description:** The conversion of `.expect("_sensor must be StringArray")` → `.ok_or("_sensor must be StringArray")?` changes test failure semantics. With `.expect()`, a wrong array type causes a panic with a clear message in the test output. With `.ok_or()?`, the error propagates as a `Box<dyn Error>` and the test is marked as `FAILED` (not `panicked`) — the diagnostic output is different. More importantly, there is a semantic difference: `.expect()` requires the downcast to succeed OR panics (this is a test assertion), while `.ok_or(...)?` propagates the failure *as an early return* from the test function. If `#![allow(clippy::expect_used)]` is now on, the `.expect()` form was actually more idiomatic here and didn't need conversion.
- **Evidence:**
  ```rust
  // Before (develop):
  let col = result.column(col_idx).as_any()
      .downcast_ref::<StringArray>()
      .expect("_sensor must be StringArray");  // Test assertion — panics on wrong type
  
  // After (PR branch):
  let col = result.column(col_idx).as_any()
      .downcast_ref::<StringArray>()
      .ok_or("_sensor must be StringArray")?;  // Early return as Box<dyn Error>
  ```
  Since the file-level `#![allow(clippy::expect_used)]` was added, this `.expect()` is no longer a clippy violation — the conversion to `.ok_or()` was unnecessary for this specific call site.
- **Proposed Fix:** Revert line 682 to `.expect("_sensor must be StringArray")` since the file-level allow covers it and `.expect()` better expresses the test assertion intent. Alternatively, accept the change as cosmetically equivalent under the allow annotation (the test still fails if the downcast fails — just with a different error type).

### LOW

#### ADV-W3MT-P55-LOW-001: parser_tests.rs comment fix is correct but narrow — "build_dml_parser" also appears in surrounding test context comments

- **Severity:** LOW
- **Category:** code-quality
- **Location:** `crates/prism-query/src/tests/parser_tests.rs:3374`
- **Description:** The PR correctly fixes one comment line from "The DML parser (build_dml_parser) expects..." to "The DML parser expects...". This is the only remaining reference to `build_dml_parser` in parser_tests.rs on the PR branch. Verified: no other references remain in parser_tests.rs on the PR branch. Finding is informational only — the fix is complete.
- **Evidence:** `git show origin/maintenance/clippy-unwrap-cleanup:crates/prism-query/src/tests/parser_tests.rs | grep build_dml_parser` returns no output.
- **Proposed Fix:** None required. The fix is complete.

#### ADV-W3MT-P55-LOW-002: bc_gap_fill_tests.rs has one residual .unwrap() not covered by the conversion

- **Severity:** LOW
- **Category:** code-quality
- **Location:** `crates/prism-query/src/tests/bc_gap_fill_tests.rs:936`
- **Description:** One `.unwrap()` call remains in bc_gap_fill_tests.rs at line 936: `*drop_count.lock().unwrap()`. This is a `Mutex::lock()` unwrap — standard Rust idiom for non-poisoned mutex in tests. The file-level `#![allow(clippy::unwrap_used)]` covers it. The conversion is incomplete but acceptable since this particular `.unwrap()` on a Mutex is idiomatic.
- **Evidence:** Single remaining unwrap confirmed at `bc_gap_fill_tests.rs:936`.
- **Proposed Fix:** None required. Mutex lock unwrap is idiomatic in test code and the `#![allow]` covers it. Optional: convert to `.expect("mutex should not be poisoned")` for parity with converted call sites.

#### ADV-W3MT-P55-LOW-003: perimeter-violation/main.rs version tag "v1.14" historical attribution preserved at line 141 — correct per F-PASS6-LOW-001 but reviewers may not know the context

- **Severity:** LOW
- **Category:** code-quality
- **Location:** `tests/external/perimeter-violation/src/main.rs:141`
- **Description:** The commit `150e0b06` on the PR branch (the last of the 5 maintenance commits) deliberately restores "Added in BC-2.11.006 v1.14 (F-PR130-P1-HIGH-002)" for `parse_sql_dml_with_limits`. This is an explicit historical attribution comment — correct per finding F-PASS6-LOW-001 which required restoring the v1.14 anchor for provenance. However a reviewer unfamiliar with that finding history might flag it as stale (the file header now says "v1.16" while an internal comment says "v1.14"). A brief co-located inline note would remove the ambiguity.
- **Evidence:** `tests/external/perimeter-violation/src/main.rs:1` says `BC-2.11.006 v1.16`. `tests/external/perimeter-violation/src/main.rs:141` says `Added in BC-2.11.006 v1.14`.
- **Proposed Fix:** No code change required — this is purely a documentation readability concern. Optional: add a sentence like `// (File-level header shows v1.16 which removed build_dml_parser; this symbol was added in v1.14.)` to clarify the version discrepancy is intentional.

## Policy Rubric Compliance

### POL-1 (append_only_numbering): PASS
No VSDD identifiers were renumbered. The PR removes `build_dml_parser` from perimeter-violation (consistent with its deletion from sql_parser.rs) but doesn't retire a VSDD ID.

### POL-2 (lift_invariants_to_bcs): NOT APPLICABLE
This PR does not create or modify domain invariants.

### POL-3 (state_manager_runs_last): NOT APPLICABLE
This is a code PR, not a spec burst.

### POL-4 (semantic_anchoring_integrity): PASS
The lib.rs version update from v1.14 → v1.16 is semantically correct for the DI-034 layer-4 group. The comment anchor correctly describes what the layer-4 group contains after v1.16 deletion.

### POL-5 (creators_justify_anchors): NOT APPLICABLE
No new anchors created.

### POL-6 (architecture_is_subsystem_name_source_of_truth): NOT APPLICABLE

### POL-7 (bc_h1_is_title_source_of_truth): NOT APPLICABLE

### POL-8 (bc_array_changes_propagate_to_body_and_acs): NOT APPLICABLE
No story frontmatter changes.

### POL-9 (vp_index_is_vp_catalog_source_of_truth): NOT APPLICABLE

### POL-10 (demo_evidence_story_scoped): NOT APPLICABLE

### POL-11 (index_bump_required_for_index_mutations): NOT APPLICABLE
No index files (STORY-INDEX, BC-INDEX, VP-INDEX, ARCH-INDEX) modified.

## Deletion Correctness Verdict

`build_dml_parser` deletion is **CORRECT**:
1. On the PR branch: `sql_parser.rs` — zero occurrences of `build_dml_parser`. Deletion complete.
2. `lib.rs` perimeter docstring — `build_dml_parser` removed from the symbol list. ✓
3. `perimeter-violation/src/main.rs` — `use prism_query::sql_parser::build_dml_parser` import removed. ✓
4. `parser_tests.rs` — comment updated. ✓
5. No test or production code calls `build_dml_parser()` on the PR branch (grep confirmed zero results).
6. The function was protected by `#[cfg_attr(not(test), allow(dead_code))]` indicating it was already treated as test-only. With the `#![allow(clippy::unwrap_used)]` test convention and no remaining callers, deletion is safe.

**Attack-surface question:** Does deleting a `pub(crate)` function EXPAND the attack surface? No. Deleting a `pub(crate)` function shrinks the internal API surface. External callers were already blocked from calling it (E0603). The function was a dead convenience wrapper around `choice()` of the three per-op parsers. Its deletion removes an untested code path (the `#[cfg_attr(not(test), allow(dead_code))]` annotation confirms it was suppressed in non-test builds).

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 2 |
| LOW | 3 |

**Overall Assessment:** pass-with-findings

**Convergence:** FINDINGS_REMAIN

**Readiness:** The core changes (build_dml_parser deletion, perimeter docstring sync, test allow-attribute) are correct. HIGH-001 requires clarification before merge (S-3.04 alias symbols — is the perimeter-violation crate in sync with what's actually deployed?). HIGH-002 is a minor version pin drift. MEDIUM-001 and MEDIUM-002 are cosmetic/style concerns. LOW findings are optional cleanup.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 55 |
| **New findings** | 7 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 7 / (7 + 0) = 1.0 |
| **Median severity** | MEDIUM |
| **Trajectory** | First PR-targeted pass in this cycle |
| **Verdict** | FINDINGS_REMAIN — HIGH-001 (perimeter alias gap) requires resolution confirmation before merge |
