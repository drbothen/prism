---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T02:00:00Z
phase: 3
inputs: []
input-hash: "[live-state]"
traces_to: S-PLUGIN-PREREQ-A
pass: 3
previous_review: S-PLUGIN-PREREQ-A-pass-2.md
target_artifact: S-PLUGIN-PREREQ-A
review_layer: LOCAL
pass_n: 3
target_sha: 9578f574
base_sha: c6dd6602
prior_passes: [1, 2]
verdict: BLOCKED-hard
streak: 0/3
finding_counts:
  CRITICAL: 0
  HIGH: 1
  MED: 3
  LOW: 2
  OBS: 3
trajectory: "14 → 12 → 6"
---

# Adversarial Review: S-PLUGIN-PREREQ-A (Pass 3)

## Finding ID Convention

Finding IDs use the format: `F-LP3-<SEV>-<SEQ>` for LOCAL pass 3 of S-PLUGIN-PREREQ-A.

- `F`: Finding prefix
- `LP3`: LOCAL Pass 3
- `<SEV>`: Severity abbreviation (`HIGH`, `MED`, `LOW`, `OBS`)
- `<SEQ>`: Three-digit sequence within the pass

Examples: `F-LP3-HIGH-001`, `F-LP3-MED-001`, `F-LP3-LOW-001`

## Part A — Fix Verification (Pass 2 — 12 findings + 2 OBS)

**Summary:** 6 fully CLOSED · 2 PAPER-CLOSE / REGRESSION DETECTED · 3 deferred acceptably · 1 OBS scope-out

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP2-CRIT-001 (E0432 CI regex missing E0432 match) | CRITICAL | RESOLVED | ci.yml regex extended; E0432 SensorType regression now detected by CI perimeter step |
| F-LP2-CRIT-002 (From<&str> panic on user-controlled PrismQL input) | CRITICAL | RESOLVED | Both dispatch sites (write_dispatch.rs:281, explain.rs:666) converted to try_from_str with E-QUERY-031 error mapping; panicking From<&str> impls removed at call sites |
| F-LP2-HIGH-001 (doc-comment try_from vs try_from_str) | HIGH | PAPER-CLOSE — REGRESSION | Doc-comments at sensor_id.rs:246-257 rewritten in fix-burst-2. However, L248-256 now claims "SensorId::try_from(s)? returns Err(SensorIdValidationError)" — TryFrom<&str> is still NOT implemented. L33-37 correctly explains the API. The two blocks contradict each other. → Escalated to F-LP3-HIGH-001 |
| F-LP2-HIGH-002 (WriteToolInvalidationMap doc lies runtime extensibility) | HIGH | RESOLVED | Doc-comment rewritten honestly in fix-burst-2; TD-S-PLUGIN-PREREQ-A-003 filed for PREREQ-E. Honest framing confirmed at invalidation.rs:38-57 |
| F-LP2-HIGH-003 (byte-vs-char length boundary) | HIGH | RESOLVED | validate_sensor_id_string reordered to perform charset check before length check; ASCII-only charset makes byte/char distinction moot; doc-comment updated to match |
| F-LP2-HIGH-004 (try_from_string dead code) | HIGH | RESOLVED | try_from_string deleted; callers verified using try_from_str exclusively; no dead-code lint suppression needed |
| F-LP2-MED-001 (stale SensorType doc-comments in 4 test-file locations) | MED | RESOLVED | 4 test-file doc-comments swept in fix-burst-2; current-tense SensorType references replaced with SensorId |
| F-LP2-MED-002 (no InvalidBoundary test) | MED | RESOLVED | test_sensorid_validation_rejects_boundary_chars added covering leading/trailing `-` and `_`; positive interior-separator case also present |
| F-LP2-MED-003 (is_empty() empty-registry short-circuit masks boot-failure) | MED | PAPER-CLOSE — DOCUMENTED NOT ENFORCED | Fix-burst-2 added doc-comment to boot.rs:817-833 explaining the defense-in-depth contract and referencing the assertion obligation. However, boot.rs step8 body remains todo!(). The assertion is documented-not-enforced. TD-S-PLUGIN-PREREQ-A-004 filed (this burst) tracking the obligation to the successor story. → Escalated to F-LP3-MED-001 (documentation-only close accepted with TD filed) |
| F-LP2-LOW-001 (residual Red Gate comment in sensorid_dispatch_redgate.rs:38) | LOW | RESOLVED | Comment updated to honest framing: "callers MUST use try_from_str for user-controlled data" |
| F-LP2-LOW-002 (ADR-023 §C1 references prism-ocsf incorrectly) | LOW | RESOLVED | ADR-023 v1.17→v1.18 §C1 updated to cite DTU generator crates instead of prism-ocsf |
| F-LP2-LOW-003 (sensor_type field name inconsistency) | LOW | RESOLVED | sensor_type renamed to sensor_id across FanOutTarget/FanOutError/ExplainSource and all 11 sibling call sites in 4 crates (S-7.01 sweep applied) |
| OBS-LP2-001 (LazyLock choice note) | OBS | ACKNOWLEDGED | Affirmed correct primitive choice; no action required |
| OBS-LP2-002 (is_sensor_registered naming note) | OBS | ACKNOWLEDGED | Naming confirmed intent-revealing; no action required |

## Part B — New Findings (Pass 3)

### HIGH

#### F-LP3-HIGH-001: sensor_id.rs doc-comment L248-256 contradicts L33-37 — TryFrom<&str> claim persists post-rewrite

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `sensor_id.rs:33-37` and `sensor_id.rs:246-257`
- **Description:** Fix-burst-2 rewrote the doc-comments to resolve F-LP2-HIGH-001 (four references to `SensorId::try_from(s)` where only `try_from_str` exists). The rewrite at L33-37 is correct — it explains the actual API surface and notes that TryFrom is not implemented. However, L248-256 (the doc-comment on the `impl SensorId` block containing `try_from_str`) was independently rewritten and now claims "SensorId::try_from(s)? returns Err(SensorIdValidationError) — use this for untrusted input." This is FALSE: `SensorId::try_from(s)` does not exist; calling it produces a compile error. The two doc-comment blocks are mutually contradictory. A developer reading L248-256 in isolation (the common case when browsing the impl block) gets the same false API promise that was the root cause of F-LP2-HIGH-001.
- **Evidence:** sensor_id.rs:33-37 correctly says `try_from_str`; sensor_id.rs:248-256 says `try_from(s)?` — verified by full-file read; TryFrom<&str> impl block does not exist anywhere in the file.
- **Paper-close detection:** This is a TD-VSDD-059 paper-close — F-LP2-HIGH-001 claimed RESOLVED but the doc-rewrite introduced a new contradictory site that re-asserts the same false API.
- **Proposed Fix (Option A — canonical):** Delete L248-256 doc-comment entirely and let L33-37 serve as the module-level API guidance; or align L248-256 to reference `try_from_str` explicitly as L33-37 does.
- **Proposed Fix (Option B — implement TryFrom):** Implement `impl TryFrom<&str> for SensorId` and `impl TryFrom<String> for SensorId` delegating to the appropriate try_from_* methods. Makes both doc-comment sites true. Closes the API surface gap permanently.

### MEDIUM

#### F-LP3-MED-001: boot.rs step8 assertion documented-not-enforced (paper-close of F-LP2-MED-003)

- **Severity:** MED
- **Category:** defense-in-depth gap
- **Location:** `crates/prism-bin/src/boot.rs:817-838`
- **Description:** Fix-burst-2 addressed F-LP2-MED-003 (empty-registry short-circuit masks boot-failure) by adding a doc-comment to boot.rs step8 explaining: "When step8 is wired, the first thing it does after receiving the AdapterRegistry is check is_empty() and emit a fatal BootError if true in production mode." The step8 body is still `todo!()`. The assertion is DOCUMENTED-NOT-ENFORCED. The materialization.rs:647-653 short-circuit remains the only runtime guard (defense-in-depth per the doc). This is an acceptable deferral because step8 is intrinsically blocked on story completion — however the TD must be filed to prevent the assertion being forgotten when step8 is wired.
- **Assessment:** Fix-burst-2 correctly documented the obligation. TD-S-PLUGIN-PREREQ-A-004 was filed this burst (D-383) tracking the assertion. This finding is ACCEPTED WITH TD as the closure mechanism. No additional implementer action required for pass-3.
- **Acceptance criteria for closure:** When step8 is wired in the successor story, the first thing after receiving AdapterRegistry MUST be the is_empty() check with fatal BootError emission. TD-S-PLUGIN-PREREQ-A-004 acceptance criteria define this precisely.

#### F-LP3-MED-002: prism-dtu-common Provenance.sensor_type field and 11 caller sites missed by fix-burst-2 sibling sweep (cross-crate boundary gap)

- **Severity:** MED
- **Category:** spec-fidelity / sibling-site sweep gap
- **Location:** `crates/prism-dtu-common/src/generator/fixture.rs:28` (Provenance struct); 11 caller sites across `prism-dtu-crowdstrike`, `prism-dtu-armis`, `prism-dtu-claroty`, `prism-dtu-cyberint` (DTU generator crates)
- **Description:** F-LP2-LOW-003 renamed `sensor_type: SensorId` to `sensor_id: SensorId` across FanOutTarget/FanOutError/ExplainSource in the prism-query and prism-sensors crates. The S-7.01 sibling sweep applied in fix-burst-2 targeted those 4 crates (11 files). However, prism-dtu-common's `Provenance` struct at fixture.rs:28 has a field `sensor_type: String` (holding a sensor type string identifier for fixture provenance). This field is accessed at 11 sites across the 4 DTU generator crates under the naming convention that now conflicts with the consensus cleanup. The Provenance struct field is a string (not SensorId) so the type is different — but the name `sensor_type` in the context of a SensorId-renamed codebase is a maintenance landmine. EC-007 "must be consistent" clause applies across crate boundaries.
- **Evidence:** `grep -rn 'sensor_type' crates/prism-dtu-common/src/generator/fixture.rs` returns L28; `grep -rn '\.sensor_type' crates/prism-dtu-{crowdstrike,armis,claroty,cyberint}/` returns 11 sites with Provenance field access.
- **Note:** Provenance.sensor_type is a String (sensor name/identifier string), NOT a SensorId. The rename to `sensor_name` or `sensor_id_str` would be semantically accurate and eliminate the naming conflict.
- **Proposed Fix:** Rename Provenance.sensor_type → Provenance.sensor_name (or sensor_id_str) across prism-dtu-common + 11 caller sites in 4 DTU generator crates. Sweep: `grep -rn 'sensor_type' crates/prism-dtu-*/` to find all sites.

#### F-LP3-MED-003: E-QUERY-031 introduced in code without error-taxonomy entry or unit test

- **Severity:** MED
- **Category:** coverage-gap / spec-fidelity
- **Location:** `crates/prism-query/src/write_dispatch.rs` and `crates/prism-query/src/explain.rs` (E-QUERY-031 emitted); `crates/prism-sensors/src/sensor_id.rs` (E-QUERY-031 referenced in doc-comment); `.factory/specs/prd-supplements/error-taxonomy.md` (E-QUERY-031 absent)
- **Description:** Fix-burst-2 (F-LP2-CRIT-002 closure) introduced `E-QUERY-031` as the error code emitted when `SensorId::try_from_str(plan.sensor)` returns Err at write dispatch sites. The error code appears in implemented code and is referenced in doc-comments. However: (1) E-QUERY-031 has no entry in the canonical error taxonomy document (error-taxonomy.md QUERY section ends at E-QUERY-025); (2) no unit test verifies that the path returns E-QUERY-031 (rather than silently succeeding, panicking, or returning a different code). AC-1 requires all new error codes to appear in the taxonomy. D-383 (this burst) adds the taxonomy entry; a unit test covering the E-QUERY-031 path must be added in fix-burst-3.
- **Evidence:** `grep -rn 'E-QUERY-031' crates/` returns multiple sites; `grep -n 'E-QUERY-031' .factory/specs/prd-supplements/error-taxonomy.md` returns zero (before this burst's taxonomy edit).
- **Proposed Fix:** (1) Add E-QUERY-031 entry to error-taxonomy.md — DONE in this burst per D-383 scope. (2) Add unit test in sensor_id.rs test module or write_dispatch.rs test module: provide an invalid sensor name string to the dispatch path and assert the returned error is E-QUERY-031 (or equivalent validation error struct).

### LOW

#### F-LP3-LOW-001: explain.rs:1304 stale metadata.sensor_type string not updated to sensor_id

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** `crates/prism-query/src/explain.rs:1304`
- **Description:** F-LP2-LOW-003 sweep renamed sensor_type → sensor_id across FanOutTarget/FanOutError/ExplainSource and 11 sibling sites. L1304 of explain.rs contains `metadata.sensor_type = lower.clone()` — a string metadata field that was missed by the sweep. The field is accessed by string key in a BTreeMap-like metadata struct; the string key "sensor_type" is user-visible in EXPLAIN output, creating an inconsistency with the renamed struct fields.
- **Evidence:** `grep -n 'sensor_type' explain.rs` returns L1304 alongside the already-fixed L230 (which now correctly uses sensor_id).
- **Proposed Fix:** Update L1304 metadata key from "sensor_type" to "sensor_id" for consistency with EXPLAIN output and struct field convention.

#### F-LP3-LOW-002: validate_sensor_id_string visibility pub vs pub(crate) — broader-than-necessary API surface

- **Severity:** LOW
- **Category:** code-quality / API surface
- **Location:** `crates/prism-query/src/sensor_id.rs:196` (validate_sensor_id_string function signature)
- **Description:** `validate_sensor_id_string` is declared `pub` (crate-external visibility). The function is used only within prism-query: by try_from_str and try_from_string. No external crate imports or re-exports it. Public visibility leaks an implementation detail into the crate's API surface; callers outside the crate should use try_from_str/TryFrom, not the raw validator. Reducing to `pub(crate)` prevents unintended external callers from bypassing the SensorId newtype and calling the validator directly.
- **Evidence:** `grep -rn 'validate_sensor_id_string' crates/` outside prism-query returns zero results; function is pub at sensor_id.rs:196.
- **Proposed Fix:** Change `pub fn validate_sensor_id_string` → `pub(crate) fn validate_sensor_id_string`. Verify no crate-external callers via `grep -rn` sweep.

### OBS (Observations — Non-blocking)

#### OBS-LP3-001: TryFrom<String> delegation opportunity not taken

- **Severity:** OBS
- **Category:** API completeness observation
- **Note:** try_from_string was correctly deleted (F-LP2-HIGH-004 RESOLVED). The natural completion would be implementing `impl TryFrom<String> for SensorId` delegating to try_from_str via `&*s`. This simultaneously resolves F-LP3-HIGH-001's Option B and provides the idiomatic Rust API. No action required for convergence; mentioning as a design suggestion for fix-burst-3 implementer discretion.

#### OBS-LP3-002: write_dispatch.rs E-QUERY-031 vs E-QUERY-006 choice rationale undocumented

- **Severity:** OBS
- **Category:** design intent
- **Note:** Fix-burst-2 mapped invalid sensor name at write dispatch to E-QUERY-031 (new code) rather than the pre-existing E-QUERY-006 (query scope too broad). The choice is sensible (different semantics — validation-error vs scope error), but the rationale is not documented in a comment or the error taxonomy. The E-QUERY-031 taxonomy entry added in this burst (D-383) provides the missing documentation. No further action needed.

#### OBS-LP3-003: Provenance.sensor_type in prism-dtu-common is String not SensorId — type mismatch note

- **Severity:** OBS
- **Category:** type-design observation
- **Note:** F-LP3-MED-002 observes the field naming conflict. The deeper observation is that Provenance.sensor_type holds a human-readable sensor name string (e.g., "crowdstrike") used only for fixture metadata, not for registry lookup. It is intentionally NOT a SensorId (which requires validation). Future consideration: if fixture metadata ever needs to correlate with the SensorId namespace, the field type should change to SensorId with proper validation. For now, renaming to sensor_name avoids confusion while preserving the String type.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 3 |
| LOW | 2 |
| OBS | 3 |
| **Total** | **6** |

**Overall Assessment:** BLOCKED-hard (1 HIGH finding blocks unconditionally regardless of streak)

## Process-Gap Callouts (4)

1. **TD-VSDD-059 paper-close recurrence (F-LP3-HIGH-001):** F-LP2-HIGH-001 was declared RESOLVED after fix-burst-2 rewrote doc-comments. A post-write grep for "try_from(s)" across the file would have caught L248-256's contradictory claim before the pass. Protocol gap: after rewriting any doc-comment block, verify no OTHER block in the same file makes a contradictory assertion about the same API. This is the second paper-close in three passes (first was F-LP2-CRIT-001 from pass-1 CRIT-002).

2. **Cross-crate sibling sweep gap (F-LP3-MED-002):** Fix-burst-2's S-7.01 sweep for sensor_type → sensor_id targeted prism-query and prism-sensors (4 crates, 11 files). The sweep command did not include prism-dtu-common or the DTU generator crates. Protocol addition: when a field rename touches "sensor" naming, the sweep MUST include `crates/prism-dtu-*/` and `crates/prism-dtu-common/`. DTU generator crates use Provenance structs that share terminology with the sensor adapter layer.

3. **New error code without taxonomy entry (F-LP3-MED-003):** E-QUERY-031 was introduced by the CRIT-002 fix and referenced in code, doc-comments, and the D-382 decision log — but the canonical error taxonomy was not updated. Protocol requirement: when introducing a new E-* code in a fix-burst, the implementer MUST include the taxonomy entry in the same commit. The D-383 state-manager burst covers the taxonomy gap; a unit test is required in fix-burst-3.

4. **Metadata string key out-of-sync with renamed struct fields (F-LP3-LOW-001):** The sensor_type → sensor_id rename sweep updated struct field definitions and constructor call sites. It did not sweep metadata string keys (BTreeMap-style `metadata.sensor_type = ...` string literals). Protocol: sibling-site sweeps for struct field renames must also search for string literal usages of the old field name in metadata insertion patterns (`grep -rn '"sensor_type"'`).

## KUDOs (8)

1. **E-QUERY-031 error code introduction** — the decision to introduce a new, specific error code rather than reusing a generic one (E-QUERY-006) shows good error taxonomy discipline; the code is semantically distinct and the fix-burst-2 description correctly distinguishes validation-error from scope-error semantics.

2. **try_from_str dual-site conversion quality** — both write_dispatch.rs:281 and explain.rs:666 were correctly converted to try_from_str with proper error mapping; the fix is structurally sound and eliminates the DoS vector cleanly.

3. **try_from_string deletion** — deleting dead code rather than patching it is the correct choice; the function served no callers and would have accumulated as a maintenance landmine.

4. **InvalidBoundary test coverage** — the test added for F-LP2-MED-002 covers all four boundary cases (leading/trailing `-`, leading/trailing `_`) plus positive interior separators; exhaustive coverage for a previously unguarded validation path.

5. **ADR-023 §C1 crate enumeration correction** — the fix-burst-2 ADR-023 v1.18 update correctly identifies the DTU generator crates as the actual location of SensorType references and removes the incorrect prism-ocsf citation; source-of-truth verification confirmed.

6. **sensor_type → sensor_id 11-site sweep completion** — the rename was applied atomically across FanOutTarget/FanOutError/ExplainSource and all constructor/accessor call sites in 4 crates; no partial-rename residue in the target crates.

7. **WriteToolInvalidationMap doc-comment honesty** — invalidation.rs:38-57 now correctly documents the LazyLock read-only constraint and defers runtime extensibility to PREREQ-E with an explicit TD citation; no future developer will be misled about the extension mechanism.

8. **CI E0432 regex extension (F-LP2-CRIT-001 closure)** — the ci.yml regex extension is targeted and correct; the SensorType perimeter assertion now produces a detectable CI failure when the regression scenario is triggered; the fix is architectural rather than cosmetic.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 3 |
| **New findings** | 4 (F-LP3-MED-002, F-LP3-MED-003, F-LP3-LOW-001, F-LP3-LOW-002) |
| **Duplicate/variant findings** | 2 (F-LP3-HIGH-001 paper-close of F-LP2-HIGH-001; F-LP3-MED-001 paper-close of F-LP2-MED-003) |
| **Novelty score** | 4 / (4 + 2) = 0.67 |
| **Median severity** | MED (0 CRIT, 1 HIGH, 3 MED, 2 LOW, 3 OBS) |
| **Trajectory** | 14 → 12 → 6 |
| **Verdict** | FINDINGS_REMAIN — streak 0/3 RESET; 1 HIGH blocks unconditionally; fix-burst-3 required |

**Convergence assessment:** DECLINING trajectory (50% reduction from pass-2). No CRITICAL findings. 1 HIGH (paper-close regression) blocks. 2 paper-close regressions detected via TD-VSDD-059 discipline — both caught before they could compound into pass-4. Fix-burst-3 scope is well-bounded: doc rewrite at one site (HIGH-001), DTU rename (MED-002), unit test for E-QUERY-031 (MED-003), and two LOW cleanups. MED-001 accepted with TD-S-PLUGIN-PREREQ-A-004 as closure mechanism. Expect pass-4 to be CLEAN or 1-2 LOW findings if fix-burst-3 applies the S-7.01 sweep correctly.

## Next-Step Recommendation

**Dispatch fix-burst-3 implementer** targeting:

1. F-LP3-HIGH-001: Rewrite sensor_id.rs:248-256 doc-comment to reference `try_from_str` (not `try_from(s)?`); OR implement `impl TryFrom<&str> for SensorId` + `impl TryFrom<String> for SensorId` (Option B — permanent fix).
2. F-LP3-MED-001: No implementer action needed — TD-S-PLUGIN-PREREQ-A-004 filed by state-manager (this burst D-383) is the accepted closure mechanism; verify TD-004 exists in tech-debt-register.md.
3. F-LP3-MED-002: Rename `Provenance.sensor_type` → `Provenance.sensor_name` in prism-dtu-common/src/generator/fixture.rs:28 and 11 caller sites across 4 DTU generator crates.
4. F-LP3-MED-003: Add unit test asserting invalid sensor name at write dispatch path returns E-QUERY-031 error. Taxonomy entry already added by D-383 state-manager burst.
5. F-LP3-LOW-001: Update explain.rs:1304 metadata key "sensor_type" → "sensor_id".
6. F-LP3-LOW-002: Change `pub fn validate_sensor_id_string` → `pub(crate) fn validate_sensor_id_string`.

Apply post-fix sweep per protocol: `grep -rn 'try_from(s)\|sensor_type\|"sensor_type"' crates/` to verify no residual sites remain before declaring fix-burst-3 complete.
