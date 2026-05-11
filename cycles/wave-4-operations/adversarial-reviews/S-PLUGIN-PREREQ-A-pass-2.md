---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T01:00:00Z
phase: 3
inputs: []
input-hash: "6b836a8"
traces_to: S-PLUGIN-PREREQ-A
pass: 2
previous_review: S-PLUGIN-PREREQ-A-pass-1.md
target_artifact: S-PLUGIN-PREREQ-A
review_layer: LOCAL
pass_n: 2
target_sha: 8a33d981
base_sha: c6dd6602
fix_burst_predecessor: 8a33d981
prior_passes: [1]
verdict: BLOCKED-hard
streak: 0/3
finding_counts:
  CRITICAL: 2
  HIGH: 4
  MED: 3
  LOW: 3
  OBS: 2
---

# Adversarial Review: S-PLUGIN-PREREQ-A (Pass 2)

## Finding ID Convention

Finding IDs use the format: `F-LP2-<SEV>-<SEQ>` for LOCAL pass 2 of S-PLUGIN-PREREQ-A.

- `F`: Finding prefix
- `LP2`: LOCAL Pass 2
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`, `OBS`)
- `<SEQ>`: Three-digit sequence within the pass

Examples: `F-LP2-CRIT-001`, `F-LP2-HIGH-002`, `F-LP2-MED-001`

## Part A — Fix Verification (Pass 1 — 14 findings)

**Summary:** 9 fully CLOSED · 3 PARTIAL-CLOSE / PAPER-CLOSED · 1 properly DEFERRED · 1 deferred-by-design

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP1-CRITICAL-001 (silent unknown-table regression) | CRITICAL | RESOLVED | is_sensor_registered guard added, regression test passes |
| F-LP1-CRITICAL-002 (AC-6 perimeter compile-fail absent) | CRITICAL | PARTIALLY_RESOLVED | perimeter-violation/src/main.rs gained `use prism_core::SensorType` but ci.yml regex at L448 only matches E0603/E0624 — never E0432. SensorType regression flies undetected. → Escalated to F-LP2-CRIT-001 |
| F-LP1-HIGH-001 (stale SensorType doc-comments, 9 instances) | HIGH | RESOLVED | Production source swept — 9 instances removed |
| F-LP1-HIGH-002 (Red Gate doc-comment sweep) | HIGH | PARTIALLY_RESOLVED | sensorid_dispatch_redgate.rs:38 inline comment still says "panics at todo!() in From<&str> — Red Gate confirmed" — factually wrong post-fix-burst-1. → Escalated to F-LP2-LOW-001 |
| F-LP1-HIGH-003 (register() drift vs AC-4) | HIGH | RESOLVED | Story AC-4 wording updated to adopted-implementation rationale per D-380 |
| F-LP1-HIGH-004 (duplicate registry methods 3 pairs) | HIGH | RESOLVED | get_all_for_sensor + get_by_id deleted |
| F-LP1-HIGH-005 (SensorId Deserialize injection surface) | HIGH | RESOLVED | Deserialize delegates to try_from_str; validation enforced |
| F-LP1-MED-001 (dead UnknownSensorId variant) | MED | RESOLVED | Variant removed |
| F-LP1-MED-002 (case-sensitivity asymmetry) | MED | RESOLVED | Lowercase normalization applied |
| F-LP1-MED-003 (WriteToolInvalidationMap closed-set residue) | MED | PARTIALLY_RESOLVED | LazyLock<Vec<...>> conversion with SensorId field done (structural). Doc-comment still claims runtime extensibility — FALSE. → Escalated to F-LP2-HIGH-002 |
| F-LP1-MED-004 (OrgRegistry wiring) | MED | DEFERRED | Properly deferred with TD-S-PLUGIN-PREREQ-A-002 citing W3-FIX-S307-002 dependency. ACCEPTED. |
| F-LP1-LOW-001 (AC-8 wording) | LOW | RESOLVED | Handled by state-manager scope in D-380; AC-8 wording updated |
| F-LP1-LOW-002 (doc redundancy) | LOW | RESOLVED | Duplicate doc-block removed |
| F-LP1-LOW-003 (latency heuristic perf) | LOW | RESOLVED | Latency-match TODO comment added |

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

#### F-LP2-CRIT-001: AC-6 perimeter CI assertion is FALSE-GREEN (E0432 SensorType regression undetected)

- **Severity:** CRITICAL
- **Category:** coverage-gap
- **Location:** `.github/workflows/ci.yml:374-495`; `tests/external/perimeter-violation/src/main.rs:69, :262`
- **Description:** Fix-burst-1 added `use prism_core::SensorType` to perimeter-violation/src/main.rs to create a compile-fail assertion for the SensorType regression scenario. However, the CI step that validates perimeter violations uses a regex that only matches `error\[E0603\]`/`error\[E0624\]` — never `error\[E0432\]` (unresolved import). A re-introduced `pub enum SensorType` in prism-core would produce an E0432 error in the perimeter-violation crate — but the CI regex would not capture it. Cargo exits non-zero (compile error → test failure), but the per-symbol granular assertion iterates only BC-2.11.006 restricted_symbols (which does not include SensorType by name). Result: CI binary check may or may not detect the regression depending on whether OTHER E0603/E0624 errors are present; the SensorType-specific assertion is structurally absent.
- **Evidence:** ci.yml L448 regex pattern does not include E0432; perimeter-violation/src/main.rs:69 adds `use prism_core::SensorType` which produces E0432 on compile; restricted_symbols list in BC-2.11.006 does not enumerate SensorType.
- **Proposed Fix:** Extend ci.yml regex to also match `error\[E0432\]` patterns capturing SensorType as name; OR add dedicated VP-PLUGIN-001 perimeter assertion step grep stderr for `unresolved import.*SensorType`; OR add SensorType to a new VP-PLUGIN-001-restricted-symbols spec section.

#### F-LP2-CRIT-002: SensorId::From<&str> panics on user-controlled PrismQL input (DoS / panic-safety surface)

- **Severity:** CRITICAL
- **Category:** security-surface
- **Location:** `write_dispatch.rs:281`, `explain.rs:666`; `sensor_id.rs:49-77` (panic-on-invalid impls)
- **Description:** Fix-burst-1 HIGH-005 added validation to the Deserialize path via try_from_str delegation. However, From<&str>/From<String>/From<Arc<str>> impls at sensor_id.rs:49-77 still PANIC on invalid input. Two dispatch sites feed unvalidated user-controlled strings through the panicking From<&str> path: (1) write_dispatch.rs:281 `SensorId::from(plan.sensor.as_str())` where plan.sensor originates from extract_sensor_prefix (empty-string panic possible); (2) explain.rs:666 `Some(SensorId::from(lower.as_str()))` where invalid charset chars survive lowercasing. An adversary-controlled PrismQL query crashes the worker thread. AC-1 states all public API surfaces that construct SensorId from external input MUST use try_from_str — these sites violate AC-1 directly.
- **Evidence:** sensor_id.rs:49-77 From<&str>/From<String>/From<Arc<str>> call validate_sensor_id_string and unwrap/expect on Err; write_dispatch.rs:281 passes plan.sensor (user-controlled) without try_from_str; explain.rs:666 passes lowercased input without try_from_str.
- **Proposed Fix:** Replace `SensorId::from(...)` with `SensorId::try_from_str(...)` at both dispatch sites; map errors to E-QUERY-006/E-QUERY-031 respectively.

### HIGH

#### F-LP2-HIGH-001: Doc-comment ↔ implementation drift on try_from (TryFrom impls absent)

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `sensor_id.rs:54, :70, :164, :166` (four doc-comments reference SensorId::try_from(s))
- **Description:** Four doc-comments instruct "For untrusted/external input use `SensorId::try_from(s)`." Only `try_from_str` and `try_from_string` exist — neither is named `try_from`. Standard `TryFrom<&str>` and `TryFrom<String>` traits are not implemented. A developer following the documented API gets an unresolved-method error at compile time.
- **Evidence:** `grep -n 'try_from' sensor_id.rs` shows doc-comments reference try_from but no `impl TryFrom<&str> for SensorId` block exists in the file.
- **Proposed Fix:** Implement `impl TryFrom<&str> for SensorId` and `impl TryFrom<String> for SensorId` delegating to `try_from_str`/`try_from_string`. Or rewrite all four doc-comments to reference the actual method names.

#### F-LP2-HIGH-002: WriteToolInvalidationMap doc-comment falsely claims runtime extensibility

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `invalidation.rs:38-41, :53-57`
- **Description:** Pass-1 MED-003 fix upgraded WriteToolInvalidationMap to `LazyLock<Vec<...>>`. The doc-comment at invalidation.rs:38-41 was NOT updated and still reads "future plugin-registered write tools can extend it at runtime without recompiling." This is FALSE — `LazyLock<T>` only `Deref<Target=T>` (read-only); no `&mut` accessor, no `register_write_tool` API. Pass-1 MED-003 close was structural rename only; semantic lie persists.
- **Evidence:** invalidation.rs:38-41 doc-comment text verbatim claim of runtime extensibility; LazyLock API surface has no write accessor.
- **Proposed Fix:** Rewrite doc-comment honestly (defer runtime extensibility to PREREQ-E with TD-S-PLUGIN-PREREQ-A-003) OR implement actual runtime extensibility via `RwLock<Vec<...>>` + `register_write_tool` API.

#### F-LP2-HIGH-003: validate_sensor_id_string len check is byte-based, not char-based (inconsistent boundary)

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `sensor_id.rs:209-211`
- **Description:** `validate_sensor_id_string` uses `s.len()` (byte count in UTF-8) to enforce "1..=64 characters" — doc-comment at L208 says "characters." Multi-byte UTF-8 32-char string has byte length 64 → accepted. 33-char string of 2-byte codepoints has byte length 66 → rejected as TooLong despite being 33 Unicode chars. Misleading error messages and latent correctness defect if charset is relaxed.
- **Evidence:** sensor_id.rs:209-211 uses `s.len()` not `s.chars().count()`; doc-comment says "characters" not "bytes."
- **Proposed Fix:** Check charset BEFORE length (ASCII-only charset makes byte/char distinction moot for current input); or replace `s.len()` with `s.chars().count()`; or document "characters" means "bytes" honestly.

#### F-LP2-HIGH-004: try_from_string is dead code (zero callers, sibling-pattern recurrence)

- **Severity:** HIGH
- **Category:** code-quality
- **Location:** `sensor_id.rs:245-248`
- **Description:** `try_from_string` declared but zero callers — all call sites use `try_from_str`. Sibling-pattern recurrence of pass-1 HIGH-004 (duplicate methods added without call-site wiring). Will generate dead-code lint warning; likely requires `#[allow(dead_code)]` to pass Clippy, which suppresses a legitimate signal.
- **Evidence:** `grep -rn 'try_from_string' crates/` returns only the definition; no callers found.
- **Proposed Fix:** Delete `try_from_string` OR convert to `impl TryFrom<String> for SensorId` (simultaneously closes F-LP2-HIGH-001's TryFrom gap).

### MEDIUM

#### F-LP2-MED-001: Stale SensorType doc-comments in 4 test-file locations

- **Severity:** MED
- **Category:** spec-fidelity
- **Location:** `bc_2_01_013.rs:8, :138-143`; `bc_2_01_013_sensorid.rs:27`; `org_id_binding.rs:138-146`
- **Description:** Fix-burst-1 HIGH-001 swept production source; test files were not swept. Four stale SensorType references remain in test-file doc-comments (current-tense, not historical migration documentation).
- **Evidence:** `grep -rn 'SensorType' crates/*/tests/` surfaces 4 locations outside historical/changelog context.
- **Proposed Fix:** Search-replace SensorType → SensorId in the identified test-file doc-comment locations.

#### F-LP2-MED-002: No unit test for InvalidBoundary validation rule

- **Severity:** MED
- **Category:** coverage-gap
- **Location:** `sensor_id.rs:331-393` test module
- **Description:** Tests cover InvalidChars, TooShort, TooLong. NO test for InvalidBoundary (leading/trailing `-` or `_`). Validation code at L221-223 rejects boundaries but path is uncovered. Future refactor of boundary-check logic has no regression protection.
- **Evidence:** `grep -n 'InvalidBoundary\|boundary' sensor_id.rs` in the test module returns zero results.
- **Proposed Fix:** Add `test_sensorid_validation_rejects_boundary_chars` covering leading `-`, leading `_`, trailing `-`, trailing `_`, and valid interior separators (positive case).

#### F-LP2-MED-003: is_sensor_registered empty-registry short-circuit masks production boot-failure detection

- **Severity:** MED
- **Category:** security-surface
- **Location:** `materialization.rs:647-653`
- **Description:** CRIT-001 guard: `if !adapter_registry.is_empty() && !adapter_registry.is_sensor_registered(...)`. Empty-registry short-circuit intended for test/early-boot. If `init_registry_for_org()` fails silently in production, registry stays empty → all queries silently short-circuit sensor validation → empty results returned → original ADV-W3MT-P58-LOW-002 regression returns under boot-failure trigger.
- **Evidence:** materialization.rs:647-653 `is_empty()` guard short-circuits the is_sensor_registered check when registry is empty; no production-mode assertion prevents empty-registry serving.
- **Proposed Fix:** Add `production_mode: bool` flag set to true after `init_registry_for_org()` completes. When `production_mode = true`, skip the `is_empty()` short-circuit. Or add startup assertion that registry has ≥1 adapter before query engine starts serving.

### LOW

#### F-LP2-LOW-001: Residual Red Gate doc-comment in sensorid_dispatch_redgate.rs:38

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** `sensorid_dispatch_redgate.rs:38`
- **Description:** Pass-1 HIGH-002 swept most Red Gate prose. L38 inline comment still says "Constructing SensorId panics at todo!() in From<&str> — Red Gate confirmed" — factually wrong post-fix-burst-1 (now panics on validation, not todo!()).
- **Evidence:** `grep -n 'Red Gate\|todo' sensorid_dispatch_redgate.rs` returns L38 comment with stale text.
- **Proposed Fix:** Update comment to: "SensorId::from() panics on invalid input — callers MUST use try_from_str for user-controlled data."

#### F-LP2-LOW-002: ADR-023 §C1 references prism-ocsf but no SensorType code exists there

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** `ADR-023:518-524`
- **Description:** ADR-023 v1.17 §C1 says "match SensorType::X arms across seven locations in four crates (prism-core, prism-sensors, prism-query, prism-ocsf)" but prism-ocsf has ZERO SensorType refs. Story v1.1 enumeration is correct (DTU generator crates). ADR not updated.
- **Evidence:** `grep -rn 'SensorType' crates/prism-ocsf/` returns zero results.
- **Proposed Fix:** Update ADR-023 §C1 to read "prism-core, prism-sensors, prism-query, plus DTU generator crates."

#### F-LP2-LOW-003: sensor_type field name persists in materialization.rs/explain.rs while invalidation.rs/fanout.rs use sensor_id (naming inconsistency)

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** `materialization.rs:118`, `explain.rs:230` (use `sensor_type: SensorId`); `invalidation.rs:50`, `fanout.rs:409` (use `sensor_id: SensorId`)
- **Description:** Story EC-007 said "must be consistent." Two sites use `sensor_type: SensorId` (field named after deleted type), two use `sensor_id: SensorId`. Violates EC-007 consistency clause; `sensor_type: SensorId` is semantically misleading.
- **Evidence:** `grep -n 'sensor_type\|sensor_id' materialization.rs explain.rs invalidation.rs fanout.rs` shows mixed conventions.
- **Proposed Fix:** Standardize on `sensor_id: SensorId` (field name matches type; sensor_type is the deleted enum's name).

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 2 |
| HIGH | 4 |
| MEDIUM | 3 |
| LOW | 3 |
| OBS | 2 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate (streak 0/3 RESET; 2 CRITICAL block regardless of streak)
**Readiness:** requires revision — fix-burst-2 must close all 12 findings before pass-3

### Process-Gap Callouts (4)

1. **META-GAP recurrence — CI workflow lacks generalized E0432 detection branch:** The positive-coverage assertion pattern (TD-VSDD-057 P2) was applied for E0603/E0624 but not extended to E0432. Implementer protocol must include: "after adding compile-fail perimeter assertions, verify CI regex captures ALL error codes the assertion can produce."

2. **Implementer protocol lacks panic-safety audit step for new From<&str> impls:** When adding new From<&str>/From<String>/From<Arc<str>> impls, the dispatch prompt must require: "For each new From<T> impl, verify no panic paths exist when T is user-controlled input; if panic paths exist, implement TryFrom instead and audit all call sites."

3. **Test-writer protocol lacks exhaustive error-variant coverage check:** After implementing a validation function with N error variants, the test-writer must verify ≥1 test per error variant. The InvalidBoundary gap (F-LP2-MED-002) would have been caught by this protocol step.

4. **Doc-comment ↔ API surface coherence check missing:** When introducing a new fallible constructor (try_from_str), post-implementation review must include: "scan all doc-comments in the file for references to the new API; verify referenced method names compile." Catches F-LP2-HIGH-001 (doc references try_from but method is try_from_str).

### KUDOs (8)

1. LazyLock choice for WriteToolInvalidationMap — correct thread-safety primitive for a read-only initialized collection.
2. is_sensor_registered naming — clear, intent-revealing API name consistent with AdapterRegistry conventions.
3. try_from_str validation completeness — 4 error arms covering the full validation surface.
4. Deserialize delegation pattern — correct composition; avoids duplicating validation logic.
5. CI perimeter step structure — architecturally sound; E0432 gap is a scope miss, not a design flaw.
6. CRIT-001 fix quality — is_sensor_registered guard placed at correct architectural layer (materialization, before fanout).
7. HIGH-005 fix quality — Deserialize injection-surface fix is correct and eliminates the DI-014 sibling pattern.
8. Adapter trait propagation — SensorId changes propagated cleanly across the adapter trait boundary; no cross-crate type errors.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 2 |
| **New findings** | 9 (F-LP2-CRIT-002, F-LP2-HIGH-001, F-LP2-HIGH-003, F-LP2-HIGH-004, F-LP2-MED-001, F-LP2-MED-002, F-LP2-MED-003, F-LP2-LOW-002, F-LP2-LOW-003) |
| **Duplicate/variant findings** | 3 (F-LP2-CRIT-001 escalated from pass-1 CRIT-002 paper-close; F-LP2-HIGH-002 escalated from pass-1 MED-003 partial; F-LP2-LOW-001 escalated from pass-1 HIGH-002 partial) |
| **Novelty score** | 9 / (9 + 3) = 0.75 |
| **Median severity** | HIGH (2 CRIT, 4 HIGH, 3 MED, 3 LOW, 2 OBS) |
| **Trajectory** | 14 (pass-1) → 12 (pass-2) |
| **Verdict** | FINDINGS_REMAIN — streak 0/3; 2 CRITICAL block unconditionally; fix-burst-2 required |
