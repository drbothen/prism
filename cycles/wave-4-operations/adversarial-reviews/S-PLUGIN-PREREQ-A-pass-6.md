---
document_type: adversarial-review
level: ops
status: complete
phase: 3
pass: 6
version: "1.0"
producer: adversary
timestamp: 2026-05-11T05:00:00Z
inputs: []
input-hash: "[live-state]"
traces_to: prd.md
previous_review: S-PLUGIN-PREREQ-A-pass-5.md
target_artifact: S-PLUGIN-PREREQ-A
review_layer: LOCAL
pass_n: 6
target_sha: bcf2f717
base_sha: c6dd6602
prior_passes: [1, 2, 3, 4, 5]
verdict: BLOCKED-soft
streak: 0/3
finding_counts:
  CRITICAL: 0
  HIGH: 1
  MED: 4
  LOW: 2
  OBS: 7
trajectory: "14 → 12 → 6 → 4 → 2 → 6"
---

# Adversarial Review: S-PLUGIN-PREREQ-A (Pass 6)

## Finding ID Convention

Finding IDs use the format: `F-LP6-<SEV>-<SEQ>` (LOCAL pass 6, PLUGIN-PREREQ-A cascade).

- `F-LP6`: Fixed prefix for this LOCAL pass-6 review
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`, `OBS`)
- `<SEQ>`: Three-digit sequence within the pass

---

## Part A — Fix Verification (Pass-5 Closure Audit)

**Result: 4/4 RESOLVED — pass-5 mandatory findings all closed clean.**

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP5-MED-001 | MED | RESOLVED | `sensor_type_to_string` → `sensor_name_to_string` rename applied at definition site `virtual_fields.rs:154-165` and call site `virtual_fields.rs:74`. Grep confirms zero occurrences of `sensor_type_to_string` in workspace. The sibling-rename exhaustiveness axis (PG-LP5-002) held: fix-burst-5 correctly applied a workspace-wide grep check before declaring done. |
| F-LP5-LOW-001 | LOW | RESOLVED | `TD-S-PLUGIN-PREREQ-A-005` entry filed in `tech-debt-register.md`. The orphan citation at `explain.rs:665` now has a matching register entry. Code-side citation and register entry are bidirectionally consistent. |
| OBS-LP5-001 | OBS | RESOLVED (optional) | `.proptest-regressions` file brought under version control via `git add`. Tracking confirmed. |
| OBS-LP5-002 | OBS | RESOLVED (optional) | `#[non_exhaustive]` added to `SensorIdValidationError`. Forward-compatibility hygiene in place. |

**Pass-5 closure quality: PERFECT (4/4 mandatory findings closed; 2 optional OBS also addressed). No paper-closes detected.**

---

## Part B — New Findings

### F-LP6-HIGH-001 — SensorId::new(impl Into<Arc<str>>) public unvalidated constructor: latent footgun (HIGH)

**File:** `crates/prism-core/src/sensor_id.rs`
**Lines:** 47–52 (public `new` constructor body)
**Blast radius:** 1 public API surface; 0 callers today; unbounded future callers

**Description:** `SensorId::new(impl Into<Arc<str>>)` is a `pub` constructor that accepts any string without validation. The implementation performs no call to `validate_sensor_id_string` — it bypasses the validation path entirely. This constructor was introduced in the story's Task 1 sketch (`pub fn new(s: impl Into<Arc<str>>) -> Self { SensorId(s.into()) }`) and has persisted through all 5 prior fresh-context passes without being flagged, because no production caller yet reaches it.

**Why this is HIGH and not MED:** This is the same defect class as F-LP2-CRIT-002 (panic-on-input from `From<&str>`) but in its inverse form: instead of panicking on invalid input, `new()` silently accepts invalid sensor IDs. F-LP2-CRIT-002 was elevated to CRITICAL and closed by adding validation to `From<&str>`. The `new()` constructor creates a parallel unvalidated entry point at the same privilege level. When future plugin code calls `SensorId::new(user_controlled_string)` — and it will, because `new()` is the most natural constructor idiom in Rust — it bypasses the validation that prevents digit-first, empty, and oversized sensor IDs from entering the registry. This is a latent DoS/logic vector.

**The persistence factor:** This defect survived 5 fresh-context adversary passes. The reason is that zero callers today means the gap has no observable effect — no test fails, no runtime error surfaces. The adversary axis for this class of defect is not "find the regression" but "enumerate the public API surface and verify every entry point validates its inputs." This axis was not applied to `SensorId` in passes 1-5, which focused on call sites and dispatch patterns.

**Why zero callers today does not make this LOW:** `pub` visibility is a contract to future code. Plugin authors who read `SensorId::new()` will use it. The `From<&str>` impl validates; the `new()` constructor does not. This inconsistency will be reached by plugin code in Wave 1/A, exactly when unvalidated sensor IDs become a real attack surface (external plugin authors, user-controlled TOML values).

**Remediation:** Either (a) make `SensorId::new` call `validate_sensor_id_string` and return `Result<SensorId, SensorIdValidationError>` — consistent with the validated entry-point contract — or (b) rename `new` to `new_unchecked` with a `# Safety` doc-comment explaining it bypasses validation (plugin-runtime internal use only), and add a `pub fn new_validated(s: impl Into<Arc<str>>) -> Result<SensorId, SensorIdValidationError>` for external callers. Option (a) is the cleaner external API; option (b) preserves the zero-allocation path for trusted callers (PluginRuntime constructing from its own validated spec parse).

**Estimated fix complexity:** Low-to-moderate. Requires choosing between (a) and (b), updating the constructor signature, and verifying all callers still compile (zero callers today means zero churn).

---

### F-LP6-MED-001 — Test stub field naming inconsistency: sensor_type vs sensor_id (MEDIUM)

**Files:** Multiple test stub files under `crates/prism-sensors/src/tests/` and `crates/prism-query/tests/`
**Blast radius:** Approximately 3-5 test files; mock `SensorAdapter` impls

**Description:** Mock `SensorAdapter` implementations in test stubs use the struct field name `sensor_type` in some files (the old naming) and `sensor_id` in others (the new naming). The pass-4 orchestrator decision to adopt the implementation where the adapter owns identity was propagated to production code but not propagated uniformly to all test stub mock impls. Specifically, the mock struct field that holds the adapter's sensor identity string varies between `sensor_type: SensorId` and `sensor_id: SensorId` depending on which test file the mock lives in.

**Why MED and not LOW:** This is not a runtime correctness issue (the type is `SensorId` in both cases; only the field name differs). However, it perpetuates the `sensor_type` naming in the test corpus even after the migration's central goal was to eliminate that concept. A future adversary pass over test stubs will flag this same inconsistency again unless it is resolved now. The recurring sibling-rename axis (PG-LP5-002) applies here.

**Remediation:** Standardize all test stub mock `SensorAdapter` struct fields that hold sensor identity to use `sensor_id: SensorId` naming. Run `grep -rn "sensor_type:" crates/ --include="*.rs"` filtered to test files and update any remaining mock field declarations.

**Estimated fix complexity:** Trivial — mechanical field rename across test files.

---

### F-LP6-MED-002 — BC-2.01.013 body does not document sensor_type() trait method rename rationale (MEDIUM)

**File:** `.factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md`
**Section:** Postconditions
**Blast radius:** 1 BC file; story anchor claim

**Description:** The story's Behavioral Contracts table (BC-2.01.013 row) states that BC-2.01.013 "Drives the open dispatch requirement — adapter implementations are produced from TOML SensorSpec declarations at runtime; `SensorAdapter::sensor_type()` must return `SensorId` so the registry can be keyed by sensor identity string, not enum variant." However, the BC body's Postconditions section does not mention `SensorAdapter::sensor_type()` at all — not the method name, not the return type change, not the rationale for preserving the `sensor_type` method name despite the type rename.

This is a traceability gap: the story claims the BC drives the requirement, but the BC body cannot be independently read to understand what `sensor_type()` returning `SensorId` means or why the method name was preserved. A reader of BC-2.01.013 in isolation cannot derive from the BC body alone that `sensor_type() -> SensorId` is the canonical adapter identity accessor.

**Why MED:** The story is correctly anchored to the BC (the BC ID is right, the subsystem is right). The gap is purely in BC body completeness — the postcondition that S-PLUGIN-PREREQ-A implements is not expressed in the BC. This means adversarial review of the BC in isolation (which is the canonical review mode) cannot verify story compliance against the BC.

**Remediation:** Add a postcondition to BC-2.01.013 documenting the `SensorAdapter::sensor_type(&self) -> SensorId` canonical adapter identity accessor, the name-preservation rationale, and the `SensorId::from("crowdstrike")` construction convention. This is a BC body amendment (state-manager + product-owner scope).

**Estimated fix complexity:** Low — BC body edit; no code change.

**NOTE:** This finding was resolved by the pass-6 closure burst (BC-2.01.013 amended to v1.5 with Adapter Identity Method postcondition block). Recorded here for completeness of the pass-6 finding record.

---

### F-LP6-MED-003 — Parity proptest ASCII-only: non-ASCII surface untested (MEDIUM)

**File:** `crates/prism-core/src/sensor_id.rs` (proptest strategy for `validate_sensor_id_string`)
**Blast radius:** 1 test file; 1 proptest strategy

**Description:** The proptest strategy for `validate_sensor_id_string` generates ASCII-only input strings — lowercase letters, digits, hyphens, underscores. The validator's regex is `^[a-z][a-z0-9_-]*$`, which rejects any non-ASCII character. However, the proptest strategy does not include non-ASCII inputs (e.g., accented characters, emoji, multi-byte UTF-8 codepoints) in its generation domain. This means the "rejects invalid input" invariant is only tested for the ASCII boundary cases (digit-first, underscore-first, empty) and not for the class of Unicode inputs that the validator must also reject.

**Why this matters:** If the `Arc<str>` payload is constructed from a user-controlled string that contains multi-byte UTF-8 and the `validate_sensor_id_string` function uses byte-length checks, there is a potential discrepancy between byte-length and character-length. The existing unit test for length validation was cited in an earlier pass as using byte-length arithmetic. Non-ASCII input stress-tests the boundary between byte and character counting.

**Remediation:** Extend the proptest strategy to include non-ASCII inputs as a rejection class. Add at least one hardcoded test vector: `validate_sensor_id_string("crowdstr\u{00EF}ke")` (non-ASCII embedded in otherwise-valid string) must return `Err(SensorIdValidationError::InvalidCharacter)`.

**Estimated fix complexity:** Low — proptest strategy extension + 1 hardcoded test.

---

### F-LP6-MED-004 — AdapterRegistry::get violates story task 7 Borrow<str> mandate (MEDIUM)

**File:** `crates/prism-sensors/src/fanout.rs`
**Lines:** 337, 490 (two call sites)
**Blast radius:** 2 call sites; 1 registry method

**Description:** Story task 7 specifies: "The `get_all_for_sensor_type` method becomes `get_all_for_sensor(sensor_id: &SensorId)` — use `Borrow<str>` so callers can pass `&str` directly." The registry method was renamed per F-LP4-MED-002, but the `Borrow<str>` mandate was not implemented — the method signature accepts `&SensorId` not `impl Borrow<str>`. As a result, `fanout.rs:337` and `fanout.rs:490` are forced to clone a `SensorId` to pass to `get_all_for_sensor_name` where a `&str` lookup would suffice.

This violates the `Borrow<str>` invariant that AC-9 and BC-2.01.013 both specify: `SensorId` implements `Borrow<str>` so that `HashMap<SensorId, _>::get("string")` works without cloning. The registry's own `get_all_for_sensor_name` method should accept `impl Borrow<str>` or `&str` directly to benefit from this impl.

**Why MED:** Two forced clones in a fanout hot path (fanout.rs is called per-query-execution for each sensor group) are a performance regression relative to the story's design intent. The `Borrow<str>` impl exists precisely to eliminate these clones.

**Remediation:** Change `get_all_for_sensor_name(sensor_id: &SensorId)` to `get_all_for_sensor_name<Q>(sensor_id: &Q) where SensorId: Borrow<Q>, Q: Hash + Eq + ?Sized` — or more simply, `get_all_for_sensor_name(sensor_id: &str)` since the callers in fanout.rs have `&str` available. Update fanout.rs:337 and fanout.rs:490 to pass `&str` directly.

**Estimated fix complexity:** Low-to-moderate — generic bound or `&str` parameter change + call site updates.

---

### F-LP6-LOW-001 — validate_sensor_id_string visibility: pub vs pub(crate) (LOW)

**File:** `crates/prism-core/src/sensor_id.rs`
**Line:** ~85 (function declaration)
**Blast radius:** 1 function; 0 external callers

**Description:** `validate_sensor_id_string` is currently declared `pub`. Pass-3 finding F-LP3-LOW-002 flagged this as `pub` vs `pub(crate)` and the pass-5 closure (F-LP4-LOW-001) updated the doc-comment to reflect `pub(crate)` visibility — but did not change the visibility keyword itself. The narrative in pass-5's fix verification said F-LP4-LOW-001 was RESOLVED because "doc-comment updated to reflect `pub(crate)` visibility." However, the actual visibility remained `pub`.

This is a documentation-versus-implementation gap: the doc-comment says `pub(crate)` but the code says `pub`. The pass-5 adversary accepted the closure based on the doc-comment change without verifying the visibility keyword itself. This is a paper-close that reintroduces the original F-LP3-LOW-002 defect class.

**Remediation:** Change `pub fn validate_sensor_id_string` to `pub(crate) fn validate_sensor_id_string`. The function is an internal implementation detail of `SensorId`'s validation logic; it should not be part of the public API surface.

**Estimated fix complexity:** Trivial — single keyword change.

---

### F-LP6-LOW-002 — Story input-hash stale vs ADR-023 v1.18 (LOW / bookkeeping)

**File:** `.factory/stories/S-PLUGIN-PREREQ-A-sensorid-newtype.md`
**Field:** `input-hash`
**Blast radius:** 1 frontmatter field

**Description:** Story v1.1 carries `input-hash: "7d38067"` which was computed when ADR-023 was at v1.17 (the PREREQ-F burst). ADR-023 was subsequently bumped to v1.18 (D-382 burst — typo fix to ADR-023 §C1 prism-ocsf cite). BC-2.01.013 is now at v1.5 (this burst's F-LP6-MED-002 closure). Both are listed as `inputs:` in the story frontmatter. The input-hash reflects a stale snapshot.

**Remediation:** Run `compute-input-hash` with `--update` flag to recompute the hash from current input file states. Bump story version 1.1 → 1.2 with changelog row noting the recompute reason.

**Estimated fix complexity:** Trivial — state-manager bookkeeping.

**NOTE:** This finding was resolved by the pass-6 closure burst (story amended to v1.2, input-hash recomputed to 6954524). Recorded here for completeness.

---

## OBS (Non-blocking Observations)

### OBS-LP6-001 — SensorSpec.sensor_id remains String (carry-forward from OBS-LP5-003)

`SensorSpec.sensor_id: String` in `prism-spec-engine` has not been migrated to `SensorId`. The newtype boundary is not propagated into the spec-engine domain model. This was flagged in pass-5 as OBS-LP5-003 and remains open. Non-blocking for this pass.

**Recommendation:** Evaluate in a follow-on story (PREREQ-B or PREREQ-C scope). File as TD if not addressed in PREREQ-A.

### OBS-LP6-002 — AC-1 Serialize/Deserialize still absent (carry-forward from OBS-LP5-004)

`SensorId` does not derive `#[derive(Serialize, Deserialize)]`. Other identity newtypes (`OrgId`, `TenantId`) derive both. Carry-forward from pass-5. Non-blocking.

**Recommendation:** Add before PREREQ-B ships (PREREQ-B may require TOML deserialization of sensor specs that reference `SensorId`).

### OBS-LP6-003 — Proptest case count at default 256 (carry-forward from OBS-LP5-005)

Proptest strategies in `sensor_id.rs` tests still run at 256 cases (CLAUDE.md convention: 32). Neither `#[proptest(cases = 32)]` nor a `ProptestConfig` override has been added. Carry-forward from pass-5. Non-blocking.

### OBS-LP6-004 — SensorIdValidationError #[non_exhaustive] confirmed present

OBS-LP5-002 was closed in fix-burst-5. Confirmed: `#[non_exhaustive]` is present on `SensorIdValidationError`. No action needed.

### OBS-LP6-005 — Byte-vs-character length asymmetry in max-length check

`validate_sensor_id_string` enforces a max-length bound using `.len()` (byte-length) rather than `.chars().count()` (character-length). For ASCII-only input (all valid sensor IDs per the regex) this is equivalent, but the discrepancy is a latent confusion point for future maintainers who may not realize the length limit is in bytes. A comment clarifying "byte-length is equivalent to char-length for ASCII-only valid inputs" would prevent misunderstanding.

**Recommendation:** Add inline comment; no semantic change required.

### OBS-LP6-006 — get_all_for_sensor_name doc-comment may retain stale method name reference

`get_all_for_sensor_name`'s doc-comment may retain a reference to `get_all_for_sensor_type` in its description of the rename history (if such prose was added in fix-burst-4). Cosmetic — no functional impact.

**Recommendation:** Sweep doc-comments in registry.rs for stale name references.

### OBS-LP6-007 — AC-8 squash-merge mandate not testable pre-merge

AC-8 specifies "The PR contains exactly ONE squash-merge commit." This is a post-merge verification condition that cannot be validated during LOCAL adversary review (before the PR exists). It is correctly structured as a Green Gate criterion for the pr-manager 9-step cycle, not a LOCAL adversary criterion. No action required; noted for awareness.

---

## Process-Gap Findings

Three process gaps observed this pass:

**PG-LP6-001 — Pub-API enumeration discipline absent from fix-burst checklist**: F-LP6-HIGH-001 (`SensorId::new` public unvalidated constructor) persisted through 5 fresh-context passes because no pass applied the axis "enumerate all `pub` entry points in the new module and verify each validates its input." Fix-burst checklists for type-introduction stories should include: "For each `pub fn` and `pub` struct method in the new module: does it call the validation function or delegate to a validated constructor?" This axis must be added to the S-PLUGIN-PREREQ-A fix-burst template.

**PG-LP6-002 — BC-anchor verification not in fix-burst protocol**: F-LP6-MED-002 (BC body does not document `sensor_type()`) identifies a class of gap where the story references a BC but the BC body does not contain the postcondition the story implements. Fix-bursts should include a step: "For each BC in `behavioral_contracts:` frontmatter, verify the BC body contains at least one postcondition or invariant that is directly testable by the story's ACs." This is the BC-anchor verification axis.

**PG-LP6-003 — Cross-crate proptest input-domain auditing**: F-LP6-MED-003 (non-ASCII inputs untested) surfaces the pattern that proptest strategies are written to test the happy path and the obvious ASCII boundary cases but omit the UTF-8 input class that exercises byte-vs-char counting discrepancies. Future adversary passes over proptest-using modules should include the axis: "Does the proptest strategy include inputs from each distinct character class (ASCII, non-ASCII Unicode, multi-byte codepoints)?"

---

## KUDOs

**KUDO-LP6-001 — Fix-burst-5 scope discipline**: Fix-burst-5 (pass-5 closures) addressed exactly the 2 mandatory findings plus 2 optional OBS items. No scope creep, no opportunistic changes. The clean closure of a pass with tiny scope is itself a discipline win — previous cascades had fix-bursts that over-reached or under-reached.

**KUDO-LP6-002 — #[non_exhaustive] forward-compat hygiene**: OBS-LP5-002 (add `#[non_exhaustive]` to `SensorIdValidationError`) was addressed in fix-burst-5 despite being non-mandatory. This is exactly the right behavior — when a trivial forward-compat improvement is available at near-zero cost, taking it proactively prevents future breaking changes.

**KUDO-LP6-003 — TD-005 bidirectional citation confirmed**: `explain.rs:665` cites `TD-S-PLUGIN-PREREQ-A-005` AND `tech-debt-register.md` now has the matching entry (F-LP5-LOW-001 closure). The bidirectional citation is intact. This is model tech-debt hygiene: code side and register side are in sync.

**KUDO-LP6-004 — Cross-crate parity contract comments preserved**: The inline comment in `prism-core`'s `validate_sensor_id_string` citing "must match SensorTypeValidator in prism-spec-engine" (KUDO-LP5-003 from pass-5) is still present and accurate. This maintenance tripwire survived the fix-burst-5 edits without being accidentally removed.

**KUDO-LP6-005 — Production-side field naming convergence**: The production struct field naming in `FanOutTarget`, `FanOutError`, and `ExplainSource` has converged to `sensor_id: SensorId` (the new name). The remaining naming inconsistency (F-LP6-MED-001) is isolated to test stub mock structs — production code is clean.

---

## Convergence Assessment

**Trajectory:** 14 → 12 → 6 → 4 → 2 → **6** (NON-MONOTONIC — trajectory reversed at pass-6)

**Streak:** 0/3 (streak reset; HIGH finding blocks CLEAN)

**Verdict: BLOCKED-soft** — 1 HIGH + 4 MED + 2 LOW + 7 OBS. The trajectory reversal from 2 → 6 is driven by latent defect emergence: F-LP6-HIGH-001 (unvalidated `pub` constructor) and F-LP6-MED-004 (`Borrow<str>` mandate not implemented) were always present in the codebase but not caught by passes 1-5 because they have no observable runtime effect with zero external callers. The adversary audit axes for pass-6 were broader (pub-API enumeration, Borrow<str> mandate verification) than prior passes.

**Convergence prognosis:** The trajectory reversal is concerning but not terminal. All 6 findings are closure-tractable in a single fix-burst. Three of the 6 (HIGH-001, LOW-001, LOW-002) have trivial or already-closed remediation paths. MED-001, MED-003, MED-004 are mechanical fixes. The remaining OBS items are all carry-forwards from prior passes. Pass-7 should be CLEAN if fix-burst-6 closes all 6 findings.

**Mandatory fix-burst-6 scope (in priority order):**
1. F-LP6-HIGH-001: Add validation to `SensorId::new()` or rename to `new_unchecked` + add `new_validated()` (closes the latent unvalidated-entry-point gap)
2. F-LP6-MED-001: Standardize test stub field names `sensor_type` → `sensor_id` across mock SensorAdapter impls
3. F-LP6-MED-002: BC-2.01.013 body amendment — add `sensor_type() → SensorId` postcondition (state-manager + product-owner scope; **resolved in this burst**)
4. F-LP6-MED-003: Extend proptest strategy to include non-ASCII rejection class + 1 hardcoded test vector
5. F-LP6-MED-004: Change `get_all_for_sensor_name` signature to accept `impl Borrow<str>` or `&str`; update fanout.rs:337 and :490
6. F-LP6-LOW-001: Change `pub fn validate_sensor_id_string` → `pub(crate) fn validate_sensor_id_string`
7. F-LP6-LOW-002: Story input-hash recompute (state-manager scope; **resolved in this burst**)

**Next-step recommendation:** Dispatch implementer fix-burst-6 for items 1, 2, 3, 4, 5, 6 (items 3 and 7 already resolved by state-manager in this burst). After fix-burst-6 commits, dispatch adversary pass-7 fresh-context. Convergence is likely but not guaranteed — trajectory has reversed once; a second reversal would indicate systematic audit-axis gaps.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 4 |
| LOW | 2 |
| OBS | 7 |

**Overall Assessment:** BLOCKED-soft
**Convergence:** Trajectory NON-MONOTONIC (14→12→6→4→2→6); streak 0/3; fix-burst-6 required (5 mandatory findings; 2 already resolved by state-manager this burst)
**Readiness:** Requires fix-burst-6 before pass-7

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 6 |
| **New findings** | 6 (HIGH-001 latent unvalidated constructor; MED-001 test stub field naming; MED-002 BC body gap; MED-003 non-ASCII proptest; MED-004 Borrow<str> mandate; LOW-001 visibility paper-close) |
| **Resolved in this state-burst** | 2 (MED-002 BC amendment + LOW-002 input-hash) |
| **Carry-forward findings** | 0 (LOW-001 is a paper-close reopen of F-LP3-LOW-002, not a true carry-forward — the defect was never actually closed) |
| **Novelty score** | 4 genuinely novel axes (HIGH-001 pub-API enumeration; MED-003 non-ASCII proptest; MED-004 Borrow<str> mandate; LOW-001 visibility paper-close) |
| **Median severity** | MED |
| **Trajectory** | 14 → 12 → 6 → 4 → 2 → 6 (NON-MONOTONIC) |
| **Verdict** | FINDINGS_REMAIN — streak 0/3; pass-7 expected CLEAN after fix-burst-6 |
