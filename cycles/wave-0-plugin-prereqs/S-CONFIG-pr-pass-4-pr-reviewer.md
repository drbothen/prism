---
document_type: pr-level-pr-reviewer-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 4
reviewer: pr-reviewer
fresh_context: true
model_family: opus-4.7
feature_head: 3780ac27
develop_baseline: f19575ff
pass_3_head: 792573d9
timestamp: 2026-05-25T23:55:00Z
---

# PR #155 -- pr-reviewer pass-4

Fresh-context pass-4 review of PR #155 (S-CONFIG-MULTI-TENANT-OVERRIDE-001 -- per-org
sensor endpoint overlay loading per ADR-029). Diff baseline `develop@f19575ff`; feature
HEAD `3780ac27`. This pass includes the pass-3 fix-burst (commit `3780ac27`) which
addressed 2 of the pass-3 carry-forward findings: the `sensor_id_string` doc-comment
fix and the `sanitize_for_log` unit tests. CI GREEN (36/36). Target: CLEAN(strict).

51 files changed; +4856 / -40 (including demo evidence binaries). Production code delta
~1770 LOC (overlay.rs ~995 + boot ~477 + fanout ~477 + adapters ~63 + error variants ~35).
Test code: ~2015 LOC in overlay_loading_tests.rs + ~257 LOC in boot step4 overlay tests +
~290 LOC in fanout unit tests + ~98 LOC in sanitize_for_log unit tests.

---

## 8-Item Review Checklist

| # | Item | Assessment |
|---|------|-----------|
| 1 | Diff Coherence | PASS -- all changes relate to S-CONFIG-MULTI-TENANT-OVERRIDE-001 overlay loading (ADR-029). No unrelated changes. The HTTP timeout additions (`.timeout(Duration::from_secs(30))`) across 4 adapters and the `unwrap_or_default()` -> `unwrap_or_else(panic!())` hardening are in scope: they close the TD-S-PLUGIN-PREREQ-B-005 gap for these constructors (wired in the overlay dispatch path). |
| 2 | Description Accuracy | PASS -- PR body matches actual changes. Architecture mermaid, traceability matrix, and convergence narrative all match the diff. |
| 3 | Test Coverage | PASS -- 20+ tests in overlay_loading_tests.rs; 4 boot step4 tests; 3 fanout unit tests; 1 end-to-end CapturingAdapter wiring test; 6 sanitize_for_log unit tests; negative-path coverage (oversized file, unreadable file, mixed-case org, SSRF scheme, corrupt TYPE spec). All changed production code paths have at least one driving test. |
| 4 | Demo Evidence | PASS -- 7 ACs x (gif + webm + tape) = 21 recording artifacts + evidence-report.md. evidence-report.md present. All ACs have both success and error path coverage. |
| 5 | Commit Quality | PASS -- conventional commits; story ID in every message; clear intent per commit. 20 commits in the feature branch reflecting the TDD progression. |
| 6 | Diff Size | PASS -- 4856 lines total, but ~2360 are test code, ~1500 are demo evidence binaries, ~59 are a shell script, ~193 are the evidence report. Net production code delta ~1770 LOC is reasonable for a new subsystem with 5 BCs. |
| 7 | Missing Changes | No missing changes detected against the 7 ACs and 5 BCs. |
| 8 | Dependency Status | PASS -- S-WAVE5-PREP-01 merged as PR #138. No unmerged upstream dependencies. |

---

## Pass-3 Finding Disposition

### F-PR155-P2-001 (LOW) -- CrowdStrike/Cyberint base_url overlay no-op

**Status: STILL PRESENT (accepted)**

CrowdStrike and Cyberint adapters do not read `sensor_config["base_url"]`. Only
Armis and Claroty adapters resolve the effective base URL from `sensor_config`. A
user who creates a CrowdStrike or Cyberint overlay with `base_url` gets no error and
no behavioral change. Architecturally correct: CrowdStrike uses the Falcon cloud URL
derived from auth credentials, not a config-file base URL. Cyberint similarly uses
auth-tied endpoints. These sensors are unlikely to need per-org endpoint overrides.

**Severity: LOW** (design gap for future story if needed; no user-facing risk today)

---

### F-PR155-P2-002 (NIT) -- Doc comment says "sensor_id_string" but type is SensorId

**Status: CLOSED** (fixed in commit `3780ac27`)

Both `RunningServer.resolved_spec_map` and `BootContext.resolved_spec_map` doc
comments now correctly say `Key: (OrgSlug, SensorId)` instead of
`Key: (OrgSlug, sensor_id_string)`. Verified in diff.

---

### F-PR155-P2-003 (NIT) -- sanitize_for_log has no direct unit test

**Status: CLOSED** (fixed in commit `3780ac27`)

Six unit tests added in `overlay.rs mod tests`:
- `sanitize_for_log_replaces_newline_with_replacement_char`
- `sanitize_for_log_replaces_carriage_return`
- `sanitize_for_log_replaces_null_byte`
- `sanitize_for_log_truncates_at_256_chars`
- `sanitize_for_log_passes_clean_ascii_unchanged`
- `sanitize_for_log_preserves_unicode_non_control`

Good coverage: newline, CR, null, truncation, passthrough, and Unicode non-control
preservation. All six tests are load-bearing assertions per TD-VSDD-059.

---

### F-PR155-P2-004 (LOW) -- SSRF: base_url scheme check allows http://169.254.169.254

**Status: STILL PRESENT (accepted)**

SEC-REDUX-006 validation allows `http://` scheme, which includes IMDS endpoints.
Overlay files are operator-controlled config committed to the repository (not
user-supplied at runtime). An operator with write access to the overlay files already
has full control of the deployment. The threat model does not justify rejecting
`http://` at this layer. No change from pass-3 assessment.

**Severity: LOW** (hardening improvement for future story)

---

### F-PR155-P2-005 (NIT) -- Validation ordering hides SSRF URL behind structural errors

**Status: STILL PRESENT (accepted)**

`validate_overlay_toml` early-returns on structural errors (tables/unrecognized
fields) before reaching the URL scheme check. This means a file with both
`[[tables]]` and `ftp://` base_url will report only the tables error. Acceptable UX:
fix one class of error at a time (structural before semantic). The URL scheme check
fires reliably when structural checks pass.

**Severity: NIT** (defense-in-depth preference)

---

### F-PR155-P3-001 (NIT) -- OverlayLoadResult missing #[non_exhaustive]

**Status: STILL PRESENT (accepted)**

`OverlayLoadResult` is a return-only container type (analogous to a `Result` tuple).
External crates have no reason to construct it -- only `OverlayLoader::load_overlays`
returns it. Adding `#[non_exhaustive]` would prevent external struct-literal
construction, but this is a defensive nicety, not a functional concern. The type is
not TOML-deserialized. Borderline interpretation of the "all public structs"
convention. Not a merge blocker.

**Severity: NIT** (convention consistency)

---

## New Findings (Pass-4)

### No new findings.

I have reviewed every changed file in the diff (all 51 files) against the 8-item
checklist and the project conventions documented in CLAUDE.md. Below is what I
verified and why I am satisfied.

---

## Verification Details

### 1. overlay.rs (995 LOC new) -- Core overlay loading logic

**Reviewed for:**
- `SensorInstanceOverlay`, `OverlayProvenance`, `ResolvedSensorSpec` all carry
  `#[non_exhaustive]` -- confirmed.
- `OverlayLoader::load_overlays` correctly handles:
  - Absent `customers/` directory (EC-012-001) -- returns empty result.
  - `.gitkeep` and plain files skipped (EC-012-002 / INV-COMPAT-004).
  - Symlink rejection via `file_type.is_file()` (SEC-REDUX-002 / CWE-59).
  - File size limit `MAX_OVERLAY_FILE_BYTES = 64 KiB` (SEC-REDUX-005 / CWE-400).
  - Multi-error aggregation across ALL overlay files (INV-ERR-003).
  - OrgRegistry cross-validation (E-SPEC-022) fires before per-file processing.
  - Unregistered directories are still scanned for file-level errors (EC-016-002).
  - Defensive `type_specs.get(sensor_id)` fallback with `PrismError::Internal` for
    the logically-unreachable branch after successful validation.
- `validate_overlay_toml` checks:
  - TOML parse failure (E-SPEC-001).
  - `tables` key present (E-SPEC-021).
  - Unrecognized fields against `ALLOWED_OVERLAY_FIELDS` closed set (E-SPEC-023).
  - URL scheme validation for `base_url` -- http:// and https:// only (SEC-REDUX-006).
  - `instance_id` convention mismatch (E-SPEC-020).
  - `extends` references unknown TYPE spec (E-SPEC-019).
- `merge_overlay_onto_type_spec` correctly:
  - Clones the TYPE spec (never mutates the original).
  - Merges `base_url`, `rate_limit_hints.requests_per_second`,
    `rate_limit_hints.burst_size` individually with per-field provenance tracking.
  - Emits `overlay.timeout_secs_ignored` warning for `timeout_secs` (deferred
    wiring to S-CONFIG-MULTI-TENANT-OVERRIDE-002).
  - Does NOT override `tables`, `auth_type`, `version`, `sensor_id`, `name`,
    `credential_refs` (INV-OVL-001, INV-OVL-002).
- `sanitize_for_log` replaces control characters with U+FFFD, caps at 256 chars.
  Called on all 5 TOML-sourced error message values (TD-VSDD-060 sibling-sweep).
  Six unit tests cover edge cases.
- All 5 `make_e_spec_*` error constructors produce canonical message templates
  with `SpecError` structs carrying code, message, toml_path, file_path.
- No `unwrap()` or `expect()` in non-test code.
- No `println!` in production code.

### 2. boot.rs (+474 LOC) -- Boot step 4 extension

**Reviewed for:**
- `step4_load_sensor_specs_with_overlays` replaces bare `step4_load_sensor_specs`
  in the boot sequence. The old function remains available but is only called
  internally by the new function (step 4a).
- `build_type_spec_map_for_overlay` reads the spec directory a second time using
  `SpecLoader::parse` to produce `spec_parser::SensorSpec` (not `types::SensorSpec`).
  Parse failures are collected and produce a hard boot abort (PRR-005). I/O failures
  are also fatal. This prevents misleading E-SPEC-019 errors when a TYPE spec is
  corrupt.
- OrgRegistry is now non-discarded (`let org_registry = ...` instead of
  `let _org_registry = ...`) and passed to step 4.
- `resolved_spec_map` is threaded through `BootContext` and `RunningServer` via
  `Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>`.
- `tracing::info!` with `event_type = "boot.overlays_loaded"` emitted on success.
- `tracing::error!` with `event_type = "boot.type_spec_read_failed"` and
  `event_type = "boot.type_spec_parse_failed"` emitted on failure paths.
- 4 unit tests cover: absent customers dir (BC-2.06.012 backcompat), unknown org
  slug (BC-2.06.015), happy path overlay resolution, and corrupt TYPE spec hard abort.
- No `unwrap()` or `expect()` in production code (only in `#[cfg(test)]`).

### 3. engine.rs (+31 LOC) and materialization.rs (+49 LOC) -- Arc-DI threading

**Reviewed for:**
- `QueryEngine` gains `resolved_spec_map: Option<Arc<HashMap<...>>>` field.
- `QueryEngine::new()` initializes to `None` (test/MVP mode).
- `QueryEngine::new_full()` takes the map as a parameter and stores it as `Some(...)`.
- Both `execute_inner` call sites pass `self.resolved_spec_map.clone()` to
  `MaterializationContext::new_with_resolver`.
- `MaterializationContext` gains `resolved_spec_map: Option<Arc<HashMap<...>>>`.
- `run_materialization_pipeline` dispatches to `fan_out_with_overlay_map` when
  both `org_registry` and `resolved_spec_map` are `Some(...)`, otherwise falls back
  to bare `fan_out`. This is correct: test/MVP mode without overlay config uses
  the original fan_out path.
- All existing call sites in `execute_integration_tests.rs` (11 sites) pass
  `Arc::new(std::collections::HashMap::new())` or `None` for the new parameter,
  maintaining test compatibility without requiring overlay fixtures.

### 4. fanout.rs (+477 LOC) -- Per-org endpoint resolution

**Reviewed for:**
- `resolve_spec_for_fanout` performs O(1) map lookup (INV-FANOUT-002):
  `org_registry.slug_for(org_id)` -> `(org_slug, sensor_id)` -> `HashMap::get`.
  No filesystem I/O, no blocking.
- Case A (overlay found): injects overlay `base_url` into `sensor_config["base_url"]`.
  Handles both `Object(map)` and null/non-object `sensor_config` correctly.
- Case B (no overlay): returns `target.spec.clone()` unchanged.
- `fan_out_with_overlay_map` resolves all targets then delegates to `fan_out()`.
  Maintains all semaphore, partial-failure, and tracing behavior of `fan_out()`.
- 3 unit tests for resolve_spec_for_fanout (Case A, Case B, unknown org).
- 1 end-to-end test (`test_F_LP2_CRIT_001`) using `CapturingAdapter` and
  `StubOverlayCreds` -- verifies the overlay base_url reaches the adapter.

### 5. Adapter changes (armis.rs +23, claroty.rs +32, crowdstrike.rs +4, cyberint.rs +4)

**Reviewed for:**
- All 4 adapters now have `.timeout(Duration::from_secs(30))` on their
  `reqwest::Client::builder()` chains. This closes the TD-S-PLUGIN-PREREQ-B-005 gap
  for these constructors.
- `unwrap_or_default()` replaced with `unwrap_or_else(|e| panic!(...))` on all 4
  `Client::builder().build()` calls plus the Claroty audit_logs inner builder.
  Rationale: `Client::builder().build()` failing is unrecoverable (TLS backend
  missing). The panic message now includes the error for diagnostics.
- Armis and Claroty adapters gain `effective_base_url` parameter threading:
  - `ArmisAdapter::get_search` takes `effective_base_url: &str`.
  - `ClarotyAdapter::post_read` takes `effective_base_url: &str`.
  - Both `fetch()` implementations resolve the effective URL from
    `spec.sensor_config["base_url"]` with fallback to `self.instance_url`.
- CrowdStrike and Cyberint adapters do NOT read `sensor_config["base_url"]` --
  this is architecturally correct per pass-3 finding F-PR155-P2-001 assessment.

### 6. prism-core/error.rs (+35 LOC) -- SpecErrorCode variants

**Reviewed for:**
- 5 new variants: `ESpec019` through `ESpec023`. Each has a doc comment with the
  error message template, BC reference, ADR reference, and example message.
- Variants are added at the end of the enum (before the closing brace), maintaining
  the sequential ordering convention.

### 7. prism-core/org_registry.rs (+9 LOC) -- slug_exists method

**Reviewed for:**
- `pub fn slug_exists(&self, slug: &OrgSlug) -> bool` is a thin wrapper over
  `self.resolve(slug).is_some()`. Aligns with the story spec AC-004 method name.
  Pure read, no I/O.

### 8. CI/script/non-exhaustive changes

**Reviewed for:**
- `ci.yml EXPECTED` bumped from 32 to 35. Comment updated to explain the delta.
- `scripts/check-non-exhaustive.sh EXPECTED` bumped from 32 to 35 (TD-VSDD-060).
- `tests/external/non-exhaustive-violation/src/struct_violations.rs` adds violations
  33-35 (SensorInstanceOverlay, OverlayProvenance, ResolvedSensorSpec). Each has
  proper doc comments explaining the rationale for `#[non_exhaustive]`.
- `v35_resolved_sensor_spec` uses `OrgSlug::new("acme")` instead of
  `OrgSlug::new_unchecked` -- correct per CLAUDE.md forbidden patterns.
- `scripts/check-error-taxonomy-snapshot.sh` is a new CI-portable script that
  verifies the error-taxonomy fixture against the canonical source. Gracefully
  no-ops when `.factory/` is not mounted (CI default).
- `Justfile` adds `check-taxonomy-snapshot` recipe.

### 9. Demo evidence and fixtures

**Reviewed for:**
- `evidence-report.md` present with coverage summary table (7/7 ACs).
- 7 x (gif + webm + tape) = 21 recording artifacts.
- `error-taxonomy-snapshot.md` fixture contains E-SPEC-019..023 rows with canonical
  message templates matching the production `make_e_spec_*` constructors.
- `customers/.gitkeep` placeholder and 2 example overlay files (acme, contoso)
  committed.

### 10. Overlay TOML fixtures (acme, contoso)

**Reviewed for:**
- Both follow the overlay schema: `extends`, `instance_id`, `base_url`. No
  `[[tables]]`, no `auth_type`.
- `instance_id` values match the `{sensor_id}@{org_slug}` convention.
- Comments reference the relevant BCs and ADR-029.

---

## SAP-1 Compliance (tracing event_type catalog)

New `event_type` values in this PR (full set including pass-3 fix-burst addition):

| event_type | File | Location | Added in |
|-----------|------|----------|----------|
| `overlay.loaded` | overlay.rs:495 | load_overlays() per-file merge success | Initial impl |
| `overlay.timeout_secs_ignored` | overlay.rs:728 | merge_overlay_onto_type_spec() | Pass-3 fix-burst |
| `boot.overlays_loaded` | boot.rs:710 | step4_load_sensor_specs_with_overlays() completion | Initial impl |
| `boot.type_spec_read_failed` | boot.rs:771 | build_type_spec_map_for_overlay() I/O failure | Initial impl |
| `boot.type_spec_parse_failed` | boot.rs:791 | build_type_spec_map_for_overlay() parse failure | Initial impl |

I cannot verify BC-2.16.002 catalog rows directly (information wall). The
`overlay.timeout_secs_ignored` event was added in the pass-3 fix-burst and requires
a corresponding BC-2.16.002 catalog row per SAP-1. The orchestrator/adversary should
verify SAP-1 compliance on all 5 entries.

---

## CLAUDE.md Convention Compliance Scan

| Convention | Status |
|-----------|--------|
| `#[non_exhaustive]` on public types | PASS (3 overlay types + compile-fail gate v33-v35) |
| No `unwrap()`/`expect()` in non-test code | PASS -- all instances in `#[cfg(test)]` blocks |
| No `println!` in production code | PASS |
| `tracing::*!` with structured fields | PASS -- all new tracing calls use structured event_type |
| 30s timeout on reqwest::Client | PASS -- all 4 adapter constructors + Claroty audit_logs inner client |
| No `OrgSlug::new_unchecked` outside test-helpers | PASS |
| Error taxonomy codes registered | PASS -- E-SPEC-019..023 variants in SpecErrorCode enum |
| Arc-DI plumbing | PASS -- resolved_spec_map threaded via Arc through RunningServer -> QueryEngine -> MaterializationContext -> fan_out_with_overlay_map |
| Forbidden patterns | PASS -- no retired shadow enums, no placeholder-construct, no silent Vec::new() |

---

## Summary

| Severity | Count | Finding IDs |
|----------|-------|-------------|
| CRIT | 0 | -- |
| HIGH | 0 | -- |
| MED | 0 | -- |
| LOW | 2 | F-PR155-P2-001 (CS/Cyberint base_url no-op -- accepted), F-PR155-P2-004 (IMDS SSRF -- accepted) |
| NIT | 2 | F-PR155-P2-005 (validation ordering -- accepted), F-PR155-P3-001 (OverlayLoadResult non_exhaustive -- accepted) |

### Closed since pass-3

- F-PR155-P2-002 (NIT -> CLOSED): doc-comment `sensor_id_string` -> `SensorId`.
- F-PR155-P2-003 (NIT -> CLOSED): 6 sanitize_for_log unit tests added.

### Remaining findings assessment

All 4 remaining findings (2 LOW + 2 NIT) are:
- Architecturally accepted design decisions (CS/Cyberint overlay no-op, IMDS SSRF).
- Defensive nicety preferences (validation ordering, OverlayLoadResult non_exhaustive).
- None represent production-grade violations.
- None represent correctness risks.
- None are actionable in this PR scope.

### Verdict

**CLEAN(strict): no** -- 2 LOW + 2 NIT findings present

**CLEAN(PR-merge): yes** -- zero CRIT + HIGH + MED findings

### Recommendation

**APPROVE** PR #155. The pass-3 fix-burst successfully closed the two actionable
carry-forward findings (doc-comment fix + sanitize_for_log tests). The remaining
4 findings are all accepted design decisions or defensive preferences that do not
block merge.

Production code is well-structured with comprehensive error handling. Test coverage
is thorough across all ACs (20+ overlay tests, 4 boot tests, 4 fanout tests, 6
sanitizer tests). Security hardening exceeds story scope (file size caps, symlink
rejection, URL scheme validation, log sanitization with unit tests, HTTP timeout
wiring). The Arc-DI plumbing is correctly threaded through the full boot -> query
engine -> materialization -> fanout dispatch chain with end-to-end verification via
the CapturingAdapter wiring test.
