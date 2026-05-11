---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T04:00:00Z
phase: 3
inputs: []
input-hash: "[live-state]"
traces_to: prd.md
pass: 5
previous_review: S-PLUGIN-PREREQ-A-pass-4.md
target_artifact: S-PLUGIN-PREREQ-A
review_layer: LOCAL
pass_n: 5
target_sha: fb4769c3
base_sha: c6dd6602
prior_passes: [1, 2, 3, 4]
verdict: BLOCKED-soft
streak: 0/3
finding_counts:
  CRITICAL: 0
  HIGH: 0
  MED: 1
  LOW: 1
  OBS: 6
trajectory: "14 → 12 → 6 → 4 → 2"
---

# Adversarial Review: S-PLUGIN-PREREQ-A (Pass 5)

## Finding ID Convention

Finding IDs use the format: `F-LP5-<SEV>-<SEQ>` (LOCAL pass 5, PLUGIN-PREREQ-A cascade).

- `F-LP5`: Fixed prefix for this LOCAL pass-5 review
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`, `OBS`)
- `<SEQ>`: Three-digit sequence within the pass

---

## Part A — Fix Verification (Pass-4 Closure Audit)

**Result: 5/6 RESOLVED, 1 PARTIALLY RESOLVED — sibling-miss cascading to F-LP5-MED-001.**

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP4-MED-001 | MED | RESOLVED | `validate_sensor_id_string` in `prism-core` now requires leading letter (regex `^[a-z][a-z0-9_-]*$`). Confirmed aligned with `prism-spec-engine` `SensorTypeValidator`. Cross-crate validator parity achieved. |
| F-LP4-MED-002 | MED | RESOLVED | `get_all_for_sensor_type` renamed to `get_all_for_sensor_name` per story task 7 mandate. No call sites remain using the old name. Grep confirms zero occurrences of `get_all_for_sensor_type` in workspace. |
| F-LP4-MED-003 | MED | PARTIALLY RESOLVED | `sensor_type_from_source_ref` and `sensor_type_from_table_name` private fn names updated in the public-facing / widely-referenced call graph. **HOWEVER:** `virtual_fields.rs:154-165` contains a private helper `sensor_type_to_string` and its call site at line 74 that still uses the old `sensor_type` naming concept. This is a sibling-miss at blast radius 2 sites in 1 file — the same closed-enum-concept residue that drove F-LP4-MED-003. Cascades to **F-LP5-MED-001** below. |
| F-LP4-MED-004 | MED | RESOLVED | `SensorAdapter::sensor_type` trait method now carries a doc-comment citing task-5 deferral rationale. Doc-comment is honest and non-aspirational. Confirmed. |
| F-LP4-LOW-001 | LOW | RESOLVED | `validate_sensor_id_string` doc-comment updated to reflect `pub(crate)` visibility. No longer aspirationally public. Confirmed. |
| F-LP4-LOW-002 | LOW | RESOLVED | `TD-S-PLUGIN-PREREQ-A-005` cited at `explain.rs:665` — the TODO comment cross-references the tech-debt entry for EXPLAIN silent-skip vs E-QUERY-031 UX inconsistency. Code-side citation present. **NOTE:** The TD entry itself was absent from `tech-debt-register.md` at time of review — this is filed as **F-LP5-LOW-001** (process-gap: orphan citation). |

**Pass-4 closure quality: STRONG.** Five clean closures. One sibling-miss (F-LP4-MED-003 → F-LP5-MED-001) is a known adversary axis: private helper rename exhaustiveness. The EXPLAIN TD citation (F-LP4-LOW-002) is code-side complete; the register-side gap is a bookkeeping miss, not an implementation gap.

---

## Part B — New Findings

### F-LP5-MED-001 — sensor_type_to_string private helper: sibling-miss at virtual_fields.rs (MEDIUM)

**File:** `crates/prism-query/src/virtual_fields.rs`
**Lines:** 154-165 (helper body), 74 (call site)
**Blast radius:** 2 sites in 1 file

**Description:** `virtual_fields.rs` contains a private helper function named `sensor_type_to_string` at lines 154-165. Its call site is at line 74. This helper name embeds the old `sensor_type` concept that was systematically retired across the codebase during passes 1-4 (Provenance rename, SensorType deletion, SensorAdapter method name updates). The `sensor_type_to_string` naming is the same closed-enum-concept residue that generated F-LP3-MED-002 (Provenance.sensor_type sibling-miss across 4 DTU crates) and F-LP4-MED-003 (sensor_type_from_* private fn names).

**Why MED and not LOW:** The function name is a _private helper_, not a public API surface, but it perpetuates the "sensor_type" nomenclature in an active computation path. The adversary axis here is cross-crate exhaustiveness: fix-burst-4 correctly renamed the public-facing and widely-referenced functions but missed this internal helper because `virtual_fields.rs` was not in the sweep target list. The fact that this exact pattern has recurred across 3 passes (dtu-common, sensor_type_from_*, now virtual_fields) elevates severity from LOW: it is a systematic sweep-discipline gap, not an isolated miss.

**Remediation:** Rename `sensor_type_to_string` → `sensor_name_to_string` (or similar `sensor_name`-aligned name) at both the definition site (lines 154-165) and the call site (line 74). Verify no other `sensor_type_to_string` references exist in workspace.

**Estimated fix complexity:** Trivial — 2 sites, 1 file, inline rename.

---

### F-LP5-LOW-001 — TD-S-PLUGIN-PREREQ-A-005 cited in explain.rs:665 but absent from tech-debt-register.md (LOW / process-gap)

**File:** `crates/prism-query/src/explain.rs`
**Line:** 665 (TODO comment citing TD-S-PLUGIN-PREREQ-A-005)
**Blast radius:** 1 citation → 0 register entries (orphan citation)

**Description:** `explain.rs:665-667` contains a TODO comment that references `TD-S-PLUGIN-PREREQ-A-005` by ID — the tech-debt entry for the EXPLAIN silent-skip vs E-QUERY-031 UX inconsistency filed as part of fix-burst-4 closure of F-LP4-LOW-002. The code-side citation is correct. However, `tech-debt-register.md` does not contain an entry for `TD-S-PLUGIN-PREREQ-A-005`. The state-manager backfill discipline missed the TD filing step.

This is a **process-gap**: the implementer correctly cited the TD in code (demonstrating awareness), but the state-manager did not register the entry. An orphan citation in code that has no corresponding register entry is a bookkeeping defect — future sessions resuming from STATE.md will not see this debt item in the register, creating invisible work.

**Remediation:** State-manager to file `TD-S-PLUGIN-PREREQ-A-005` in `tech-debt-register.md` with the EXPLAIN warning UX description. No code change required.

**Estimated fix complexity:** Trivial — state-manager bookkeeping only.

---

## OBS (Non-blocking Observations)

### OBS-LP5-001 — .proptest-regressions file untracked

The `.proptest-regressions/` directory (or relevant `.proptest-regressions` file written by the proptest framework when a failing case is found) is untracked in the worktree. Per project convention, proptest regression files are tracked in git so that failing examples are preserved across sessions and CI reruns. A `.gitignore` rule or accidental omission may be suppressing tracking.

**Recommendation:** Verify `.gitignore` does not suppress `.proptest-regressions`. If the file exists and is untracked, run `git add` to bring it under version control per project convention.

### OBS-LP5-002 — SensorIdValidationError not #[non_exhaustive]

`SensorIdValidationError` (introduced in fix-burst-1, `sensor_id.rs`) is a public error enum but lacks the `#[non_exhaustive]` attribute. Without this attribute, downstream crates that match on `SensorIdValidationError` variants will produce exhaustive-match compiler errors when new variants are added in future passes — a semver breaking change for any consumer. Adding `#[non_exhaustive]` now costs nothing and makes future variant additions non-breaking.

**Recommendation:** Add `#[non_exhaustive]` to `SensorIdValidationError` as a forward-compatibility hygiene measure.

### OBS-LP5-003 — SensorSpec.sensor_id is String, not SensorId (newtype boundary not propagated)

`SensorSpec` (in `prism-spec-engine`) has a `sensor_id: String` field. Given that S-PLUGIN-PREREQ-A's central deliverable is the `SensorId` open-newtype (replacing stringly-typed sensor identifiers), `SensorSpec.sensor_id` should be `SensorId` rather than `String` for the newtype boundary to propagate into the spec-engine domain model. The current `String` type forces downstream code to construct `SensorId` at the call site rather than receiving a validated `SensorId` from the spec layer.

**Recommendation:** Evaluate whether `SensorSpec.sensor_id: SensorId` is within S-PLUGIN-PREREQ-A scope or should be a follow-on story task. This is non-blocking for the current pass.

### OBS-LP5-004 — AC-1 missing Serialize/Deserialize derive

`SensorId` (AC-1) does not derive `Serialize` / `Deserialize`. Other identity newtypes in the codebase (`OrgId`, `TenantId`) derive both. The absence means `SensorId` cannot participate in JSON/TOML roundtrips without a manual impl. If `SensorSpec.sensor_id` migrates from `String` → `SensorId` (per OBS-LP5-003), serialization support becomes load-bearing for config file loading.

**Recommendation:** Add `#[derive(Serialize, Deserialize)]` with appropriate serde validation (validate on `deserialize`, transparent on `serialize`). Non-blocking for current pass.

### OBS-LP5-005 — Proptest case count at default 256

Proptest strategies in `sensor_id.rs` tests run at the default 256 cases. CLAUDE.md specifies `PROPTEST_CASES=32` for the TDD inner loop (8× lower than default). The test file does not carry a `#[proptest(cases = 32)]` annotation or a `ProptestConfig::default().with_cases(32)` override. This means the tests are significantly slower in the per-crate inner loop (`just iter prism-core`) than project convention intends.

**Recommendation:** Add `#[proptest(cases = 32)]` to the sensor_id proptest functions, or set `PROPTEST_CASES=32` in the `.cargo/config.toml` for the crate. Non-blocking.

### OBS-LP5-006 — Pass-4 narrative drift: "sensor_name" vs actual "sensor_id"

The pass-4 adversarial review body (and D-384 decisions log entry) refers at one point to "sensor_name" validation where "sensor_id" is the correct identifier. This is a narrative artifact from the Provenance.sensor_type → sensor_name rename context bleeding into the sensor_id validation discussion. The code itself is correct (all validation references `sensor_id`); only the review prose has the drift.

**Recommendation:** Cosmetic record-keeping note; no code change required. Flag for future pass authors to maintain `sensor_id` vs `sensor_name` precision in prose.

---

## Process-Gap Findings

Three process gaps observed this pass:

**PG-LP5-001 — TODO ↔ TD-register round-trip discipline**: The implementer correctly cited `TD-S-PLUGIN-PREREQ-A-005` in code before the state-manager filed the register entry. This exposes a sequencing gap: the TD ID should be allocated by the state-manager and passed to the implementer, not invented by the implementer and backfilled by the state-manager. Alternatively, state-manager filing should be an atomic part of every fix-burst commit, not a deferred step.

**PG-LP5-002 — Sibling-rename exhaustiveness**: Three consecutive passes have surfaced private-helper rename misses at cross-module or cross-crate boundaries (pass-3: dtu-common Provenance, pass-4: sensor_type_from_* private fns, pass-5: virtual_fields.rs sensor_type_to_string). Fix-burst sweep target lists should include a `grep -r sensor_type` pass over the **full workspace** as a mandatory post-rename verification step, not just the files edited in the burst.

**PG-LP5-003 — Cross-crate validator-parity-by-input as recurring adversary axis**: F-LP4-MED-001 (validator divergence: prism-core allows digit-first, prism-spec-engine rejects) and OBS-LP5-003 (SensorSpec.sensor_id: String vs SensorId) both reflect the same pattern: newtype boundaries introduced in one crate (prism-core) need active propagation sweeps into dependent crates (prism-spec-engine, prism-query). This is now a known adversary axis for all future plugin-migration passes.

---

## KUDOs

**KUDO-LP5-001 — Textbook F-LP4-MED-001 closure with bidirectional proptest invariant:** The cross-crate validator alignment (F-LP4-MED-001) was closed by adding a _bidirectional_ proptest invariant that generates both valid and boundary-invalid `sensor_id` strings and asserts both validators agree on the outcome. This is exactly the right approach for a "cross-crate parity" defect — not a unit test of one validator, but a property test proving equivalence. Strong methodology.

**KUDO-LP5-002 — Honest trait-method doc-comment (F-LP4-MED-004 closure):** The `SensorAdapter::sensor_type` doc-comment added in fix-burst-4 is truthful and precise — it names the story task that holds, cites the reason, and does not claim the method does more than it does. This is model documentation for intentionally-deferred implementation.

**KUDO-LP5-003 — Parity contract citation in prism-core validator:** After aligning `validate_sensor_id_string` with the spec-engine regex, the implementer added an inline comment citing "must match SensorTypeValidator in prism-spec-engine" with the exact regex. This creates a _maintenance tripwire_: if either validator changes, the comment flags the sibling site. Excellent defensive documentation practice.

**KUDO-LP5-004 — Proptest input strategy bias:** The proptest strategy for `validate_sensor_id_string` generates inputs biased toward boundary cases (digit-first, underscore-first, empty, max-length) rather than uniform random strings. This is sophisticated — uniform random strings almost never hit the leading-letter constraint, making the test ineffective without bias. The strategy design demonstrates understanding of proptest effectiveness, not just proptest usage.

**KUDO-LP5-005 — Step8 deferral honesty (TD-S-PLUGIN-PREREQ-A-004 follow-through):** The boot.rs step8 assertion filed as TD-004 in pass-3 was correctly NOT pre-empted in fix-burst-4 (it would have been out of scope). The doc-comment added in fix-burst-2/3 remains accurate, and the TD entry is intact. No scope creep, no paper-close temptation.

---

## Convergence Assessment

**Trajectory:** 14 → 12 → 6 → 4 → **2** (declining monotonically across all 5 passes)

**Streak:** 0/3 (blocked by F-LP5-MED-001; cannot CLEAN with any MED finding open)

**Verdict: BLOCKED-soft** — 1 MED + 1 LOW + 6 OBS. ZERO CRITICAL, ZERO HIGH across pass-5. The convergence signal is strongly positive: this is the lowest finding count in the cascade, all CRIT/HIGH have been closed for two consecutive passes, and the remaining MED is a trivial 2-site inline rename. OBS items are all housekeeping and non-blocking.

**Fix-burst-5 scope (tiny):**
1. Rename `sensor_type_to_string` → `sensor_name_to_string` at `virtual_fields.rs:154-165` and call site at `virtual_fields.rs:74` (closes F-LP5-MED-001)
2. Add `#[non_exhaustive]` to `SensorIdValidationError` (closes OBS-LP5-002 if desired before pass-6)
3. `git add` any untracked `.proptest-regressions` file (closes OBS-LP5-001 if file exists)
4. State-manager: file `TD-S-PLUGIN-PREREQ-A-005` in `tech-debt-register.md` (closes F-LP5-LOW-001)

**Pass-6 outlook:** If fix-burst-5 closes F-LP5-MED-001 and F-LP5-LOW-001, pass-6 should CLEAN. Streak 0/3 → 1/3 expected. The remaining OBS items are all non-blocking and can be deferred to the maintenance backlog.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 1 |
| LOW | 1 |

**Overall Assessment:** block
**Convergence:** findings remain — fix-burst-5 required (tiny scope: 1 inline rename + 1 TD register entry)
**Readiness:** requires revision before pass-6

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 5 |
| **New findings** | 2 (F-LP5-MED-001 sibling-miss; F-LP5-LOW-001 orphan TD citation) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 2 / (2 + 0) = 1.00 (both findings are novel; cascade not yet exhausted) |
| **Median severity** | MED (1 MED + 1 LOW; no CRIT/HIGH) |
| **Trajectory** | 14 → 12 → 6 → 4 → 2 |
| **Verdict** | FINDINGS_REMAIN — streak 0/3; pass-6 expected CLEAN after fix-burst-5 |
