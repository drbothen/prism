---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T03:00:00Z
phase: 3
inputs: []
input-hash: "[live-state]"
traces_to: prd.md
pass: 4
previous_review: S-PLUGIN-PREREQ-A-pass-3.md
target_artifact: S-PLUGIN-PREREQ-A
review_layer: LOCAL
pass_n: 4
target_sha: 17b723e2
base_sha: c6dd6602
prior_passes: [1, 2, 3]
verdict: BLOCKED-soft
streak: 0/3
finding_counts:
  CRITICAL: 0
  HIGH: 0
  MED: 4
  LOW: 2
  OBS: 3
trajectory: "14 → 12 → 6 → 4"
---

# Adversarial Review: S-PLUGIN-PREREQ-A (Pass 4)

## Finding ID Convention

Finding IDs use the format: `F-LP4-<SEV>-<SEQ>` (LOCAL pass 4, PLUGIN-PREREQ-A cascade).

- `F-LP4`: Fixed prefix for this LOCAL pass-4 review
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`, `OBS`)
- `<SEQ>`: Three-digit sequence within the pass

---

## Part A — Fix Verification (pass >= 2 only)

**Result: PERFECT — 6/6 CLOSED, 0 paper-closes detected.**

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP3-HIGH-001 | HIGH | RESOLVED | `sensor_id.rs:248-256` doc-comment fully rewritten. No longer claims `try_from(s)?` returns `Err(SensorIdValidationError)`. Panic behavior now honestly documented with rationale (API boundary enforcement). L33-37 and L248-256 are now consistent. |
| F-LP3-MED-001 | MED | RESOLVED | `boot.rs` step8 body populated with actual assertion logic (not todo!()); TD-S-PLUGIN-PREREQ-A-004 filed with direct citation; no longer documented-but-not-enforced. Substantive closure confirmed — assertion executes at runtime. |
| F-LP3-MED-002 | MED | RESOLVED | `prism-dtu-common/src/generator/fixture.rs:28` `Provenance.sensor_type` field renamed to `sensor_name`; all 11 caller sites in 4 DTU-generator crates updated; cross-crate sweep confirmed via `grep -r sensor_type` returning zero hits in prism-dtu-* crates for the Provenance struct field. |
| F-LP3-MED-003 | MED | RESOLVED | E-QUERY-031 taxonomy entry present in `error-taxonomy.md v1.18`; unit test for E-QUERY-031 error path added in `prism-query/src/tests/`; test passes. Taxonomy entry + test in one burst — no split closure. |
| F-LP3-LOW-001 | LOW | RESOLVED | `explain.rs:1304` metadata key corrected from stale `metadata.sensor_type` string to `metadata.sensor_name`. Confirmed via direct line inspection. |
| F-LP3-LOW-002 | LOW | RESOLVED | `validate_sensor_id_string` visibility changed from `pub` to `pub(crate)`. Confirmed; zero external call sites affected. |

**Pass-3 closure discipline: EXCELLENT.** All 6 closures are substantive. No documentation-only masking of implementation gaps. The cross-crate Provenance rename at 11 sites demonstrates thorough sibling-site sweep discipline. E-QUERY-031 closure is particularly strong — taxonomy entry + test in one burst, not split.

---

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

_None._

### HIGH

_None._

### MEDIUM

#### F-LP4-MED-001: Cross-crate validator divergence — prism-core allows digit-first SensorId, prism-spec-engine requires letter-first

- **Severity:** MED
- **Category:** contradictions
- **Location:** `crates/prism-core/src/sensor_id.rs` vs `crates/prism-spec-engine/src/validation.rs`
- **Description:** Two crates implementing SensorId validation enforce different charset grammars. prism-core accepts `[a-z0-9_-]+` (digit or dash may lead); prism-spec-engine enforces `^[a-z][a-z0-9_-]*$` (letter must lead). The same input string may pass one gate and fail the other, producing inconsistent behavior depending on which validator is authoritative at a given call site.
- **Evidence:**

  `crates/prism-core/src/sensor_id.rs` — `validate_sensor_id_string`:
  ```rust
  // Charset check: [a-z0-9_-], no leading-char constraint
  fn validate_sensor_id_string(s: &str) -> Result<(), SensorIdValidationError> {
      if s.is_empty() {
          return Err(SensorIdValidationError::Empty);
      }
      for ch in s.chars() {
          if !matches!(ch, 'a'..='z' | '0'..='9' | '_' | '-') {
              return Err(SensorIdValidationError::InvalidChar(ch));
          }
      }
      Ok(())
  }
  ```

  `crates/prism-spec-engine/src/validation.rs` — `validate_sensor_id`:
  ```rust
  // Regex: ^[a-z][a-z0-9_-]*$  (letter-first mandatory)
  pub fn validate_sensor_id(id: &str) -> Result<(), SpecValidationError> {
      static RE: OnceLock<Regex> = OnceLock::new();
      let re = RE.get_or_init(|| Regex::new(r"^[a-z][a-z0-9_-]*$").unwrap());
      if !re.is_match(id) {
          return Err(SpecValidationError::InvalidSensorId(id.to_owned()));
      }
      Ok(())
  }
  ```

  A sensor ID like `"1crowdstrike"` or `"-local"` passes the prism-core gate but fails the prism-spec-engine gate. Depending on which gate runs first at a call site, illegal IDs may escape into the system or legal IDs may be rejected.

- **Proposed Fix:** Add a leading-letter constraint to `validate_sensor_id_string` in prism-core: reject inputs where `s.chars().next()` is not `'a'..='z'`. This aligns prism-core with prism-spec-engine (stricter direction is correct — sensor IDs should begin with a letter per conventional identifier rules). Update `SensorIdValidationError` with a new `LeadingCharNotLetter` variant. Add a proptest cross-crate equivalence check.

---

#### F-LP4-MED-002: `get_all_for_sensor_type` method name not renamed per story task 7 mandate

- **Severity:** MED
- **Category:** spec-fidelity
- **Location:** `crates/prism-core/src/sensor_registry.rs`
- **Description:** Story S-PLUGIN-PREREQ-A task 7 (line 150) explicitly mandates renaming `get_all_for_sensor_type` → `get_all_for_sensor_name` throughout the registry interface and all callers, consistent with the SensorType→sensor_name field rename. Fix-burst-3 addressed sibling field renames in the Provenance struct but did not sweep the registry method interface. The method signature retains the old name at 17b723e2.
- **Evidence:**

  ```rust
  // crates/prism-core/src/sensor_registry.rs — current state
  pub fn get_all_for_sensor_type(&self, sensor_type: &str) -> Vec<&SensorSpec> {
      // ...
  }
  ```

  Story task 7 mandate (from S-PLUGIN-PREREQ-A story file, task 7): rename required, no deferral note present.

  **Note:** This finding is distinct from F-LP4-MED-004 (SensorAdapter::sensor_type), which has an explicit task-5 hold. Task 7 has no analogous hold.

- **Proposed Fix:** Rename `get_all_for_sensor_type` → `get_all_for_sensor_name` in the registry impl and update all callers. Mechanical rename with zero behavioral change.

---

#### F-LP4-MED-003: `sensor_type_from_source_ref` and `sensor_type_from_table_name` private functions lag rename

- **Severity:** MED
- **Category:** spec-fidelity
- **Location:** `crates/prism-core/src/sensor_registry.rs` (or related sensor dispatch module)
- **Description:** Two private functions retain `sensor_type` in their names. Fix-burst-3 correctly renamed the Provenance field (11 external sites) but did not propagate into these private helpers within the registry/dispatch module. The function names are now semantically misleading — they look up `sensor_name` values but are named as if they look up a `SensorType` enum value (which was deleted in earlier fix-bursts).
- **Evidence:**

  ```rust
  fn sensor_type_from_source_ref(source: &SourceRef) -> Option<&str> { ... }
  fn sensor_type_from_table_name(table: &str) -> Option<&str> { ... }
  ```

  Private scope — zero API surface impact. The `SensorType` enum no longer exists; these names are orphaned references to a deleted concept.

- **Proposed Fix:** Rename to `sensor_name_from_source_ref` / `sensor_name_from_table_name`. Private scope means mechanical rename with zero callers outside the module to update.

---

#### F-LP4-MED-004: `SensorAdapter::sensor_type` trait method — pending-intent not captured in code

- **Severity:** MED
- **Category:** spec-fidelity
- **Location:** `crates/prism-core/src/sensor_adapter.rs` (trait definition)
- **Description:** Story task 5 explicitly holds `SensorAdapter::sensor_type` as retained (renaming would break DTU adapter API; deferred to Wave 1/A). At 17b723e2 the trait retains the old name, which is correct per the hold. However, the intent lives only in the story file — no doc-comment or in-code annotation explains why `sensor_type` persists after the broader rename sweep. Future readers of the trait will see `sensor_type` and assume it is an oversight, not a deliberate hold.
- **Evidence:**

  ```rust
  pub trait SensorAdapter: Send + Sync {
      fn sensor_type(&self) -> &str;
      // ...
  }
  ```

  No comment citing task-5 hold, no TD reference, no deprecation marker. The story task says "kept" but the code is silent.

- **Proposed Fix:** Add a doc-comment to `SensorAdapter::sensor_type`:
  ```rust
  /// Returns the sensor name as a string.
  ///
  /// # Note
  /// This method is named `sensor_type` rather than `sensor_name` for backward
  /// compatibility with existing DTU adapter implementations. Rename is deferred
  /// to Wave 1/A adapter-SDK story per S-PLUGIN-PREREQ-A task 5.
  fn sensor_type(&self) -> &str;
  ```

---

### LOW

#### F-LP4-LOW-001: `validate_sensor_id_string` doc-comment claims external reuse that `pub(crate)` prevents

- **Severity:** LOW
- **Category:** code-quality
- **Location:** `crates/prism-core/src/sensor_id.rs`
- **Description:** `validate_sensor_id_string` was correctly changed to `pub(crate)` in fix-burst-3 (closing F-LP3-LOW-002). The doc-comment on the function still claims external validators (e.g., prism-spec-engine) can reuse this function — but `pub(crate)` makes this impossible. The comment is aspirational and contradicts the actual visibility.
- **Evidence:**

  ```rust
  /// Public to allow external validators (e.g., prism-spec-engine) to reuse
  /// this validation without duplicating charset rules.
  pub(crate) fn validate_sensor_id_string(s: &str) -> Result<(), SensorIdValidationError> {
  ```

  prism-spec-engine has its own `validate_sensor_id` implementation and cannot call this function.

- **Proposed Fix:** Update the doc-comment to reflect `pub(crate)` reality. Suggested: "Validates the sensor ID charset within prism-core. For cross-crate validation, prism-spec-engine maintains its own `validate_sensor_id`."

---

#### F-LP4-LOW-002: EXPLAIN silent-skip vs `write_dispatch` E-QUERY-031 behavioral inconsistency

- **Severity:** LOW
- **Category:** missing-edge-cases
- **Location:** `crates/prism-query/src/explain.rs` vs `crates/prism-query/src/write_dispatch.rs`
- **Description:** The EXPLAIN path silently skips unknown sensors (no error, no warning in output). The write dispatch path surfaces E-QUERY-031 for the same condition. A user who runs `EXPLAIN SELECT ...` referencing an unknown sensor ID sees the explain output with that source silently omitted — no indication the sensor ID is invalid. They discover the error only when executing the query for real.
- **Evidence:**

  ```rust
  // explain.rs — silent skip on unknown sensor
  if let Some(spec) = registry.get_by_id(&sensor_id) {
      // include in explain output
  } else {
      // sensor not found: silently omitted, no warning
  }

  // write_dispatch.rs — E-QUERY-031 on unknown sensor
  let spec = registry
      .get_by_id(&sensor_id)
      .ok_or_else(|| QueryError::UnknownSensor { id: sensor_id.clone() })?; // E-QUERY-031
  ```

- **Proposed Fix:** Non-blocking for PREREQ-A merge. Recommended: file a behavioral consistency TD or story to add an `ExplainWarning::UnknownSensor` variant to the explain output when a source sensor ID is not found in the registry. The EXPLAIN path should surface the same information the real query would surface, not silently elide it.

---

### OBS

#### OBS-LP4-001: Bidirectional validator-equivalence proptest missing

Once F-LP4-MED-001 is resolved and the two validators are aligned, a cross-crate proptest asserting `∀ s: &str, validate_sensor_id_string(s).is_ok() ↔ validate_sensor_id(s).is_ok()` would prevent future divergence. No such test currently exists. Recommend adding to PREREQ-A acceptance test plan.

#### OBS-LP4-002: `sensor_id` local variable naming convention undocumented

In multiple call sites, local variables are named `sensor_id` when holding a raw `String`/`&str` and also when holding a `SensorId` newtype. The convention is context-dependent and not documented. A naming guide or type alias (`type SensorIdStr = str`) would reduce reading ambiguity. Non-blocking cosmetic.

#### OBS-LP4-003: `SensorSpec.sensor_id` typed as `String` not `SensorId` — spec-parse boundary unprotected

`SensorSpec.sensor_id` field is typed as `String` (raw), not `SensorId` (newtype). A sensor spec loaded from TOML carries an unvalidated `String` in `sensor_id` until explicitly converted. The newtype enforcement investment (SensorId, validate_sensor_id_string, try_from_str) is not propagated to the spec struct boundary. A fully type-safe approach would use `SensorId` in `SensorSpec` with TOML deserialization through the newtype's `Deserialize` impl. Non-blocking for PREREQ-A; worth capturing as a follow-up refinement.

---

## Process-Gap Callouts

**PG-1: Cross-crate validator alignment absent from PREREQ-A acceptance criteria.** The story AC list does not mention that prism-core and prism-spec-engine validators must agree on the SensorId charset grammar. F-LP4-MED-001 surfaced because fix-burst work correctly changed one validator without checking its sibling. Story AC should include: "prism-core `validate_sensor_id_string` and prism-spec-engine `validate_sensor_id` produce equivalent accept/reject verdicts on the same input."

**PG-2: Fix-burst-3 task-sweep did not extend to registry method-name rename (task 7).** The cross-crate rename discipline applied to Provenance (11 sites, 4 crates) did not apply to the registry method interface. A post-fix-burst checklist against open task items would have caught the gap before dispatch.

**PG-3: EXPLAIN behavioral contract not traced through E-QUERY-031 introduction.** F-LP4-LOW-002 is a downstream effect of E-QUERY-031 introduction that was not traced through all query paths. Recommend a TD or story to address explain-path warning for unknown sensors.

---

## KUDOs

**KUDO-1: Provenance rename scope — 11 sites, 4 crates, zero misses.** Cross-crate sweep discipline from TD-VSDD-060 applied correctly and completely. Closes a class of defect that recurred in passes 1-3.

**KUDO-2: E-QUERY-031 closure quality.** Taxonomy entry + unit test + code in one burst. No split closure. Test covers the error path, not just the happy path.

**KUDO-3: boot.rs step8 substantive fill.** The pass-3 HIGH-001 paper-close was reversed with actual implementation logic, not another doc-comment.

**KUDO-4: sensor_id.rs doc rewrite honesty.** The doc-comment now correctly characterizes panic behavior at the API boundary with explicit rationale. Harder to write than a euphemism.

**KUDO-5: explain.rs metadata key fix accuracy.** `metadata.sensor_type` → `metadata.sensor_name` at line 1304 is a small fix with high correctness value; it was caught and applied.

**KUDO-6: Zero paper-closes across 6 pass-3 findings.** Cleanest closure record in the PREREQ-A cascade so far. Indicates internalization of TD-VSDD-059 discipline.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 4 |
| LOW | 2 |
| OBS | 3 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate (fix-burst-4 required; then pass-5)
**Readiness:** requires revision — 4 MED findings must be substantively closed before streak can open

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 4 |
| **New findings** | 4 (MED-001 through MED-004 are all novel — validator divergence and rename residues not previously identified) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 4/4 = 1.0 |
| **Median severity** | MED (dropped from HIGH at pass-3; first pass with zero CRIT/HIGH) |
| **Trajectory** | 14 → 12 → 6 → 4 |
| **Verdict** | FINDINGS_REMAIN — streak 0/3; fix-burst-4 dispatched; pass-5 target for CLEAN |
