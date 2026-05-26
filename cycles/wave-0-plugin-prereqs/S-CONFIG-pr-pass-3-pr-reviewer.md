---
document_type: pr-level-pr-reviewer-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 3
reviewer: pr-reviewer
fresh_context: true
model_family: opus-4.7
feature_head: 792573d9
develop_baseline: f19575ff
pass_2_head: 792573d9
timestamp: 2026-05-25T23:30:00Z
---

# PR #155 -- pr-reviewer pass-3

Fresh-context pass-3 review of PR #155 (S-CONFIG-MULTI-TENANT-OVERRIDE-001 -- per-org
sensor endpoint overlay loading per ADR-029). Diff baseline `develop@f19575ff`; feature
HEAD `792573d9`. This is the third pr-reviewer pass, reviewing the full diff including
all fix-bursts from passes 1 and 2. CI GREEN (36/36). Target: CLEAN(strict).

51 files changed; +4760 / -40 (including demo evidence binaries). Production code delta
~1670 LOC (overlay.rs ~899 + boot ~477 + fanout ~477 + adapters ~60 + error variants ~35).
Test code: ~2015 LOC in overlay_loading_tests.rs (20+ tests) + ~257 LOC in boot step4
overlay tests + ~290 LOC in fanout unit tests.

---

## 8-Item Review Checklist

| # | Item | Assessment |
|---|------|-----------|
| 1 | Diff Coherence | PASS -- all changes relate to S-CONFIG-MULTI-TENANT-OVERRIDE-001 overlay loading. No unrelated changes. |
| 2 | Description Accuracy | PASS -- PR body matches actual changes; fix-burst provenance chain documented. |
| 3 | Test Coverage | PASS -- 20+ tests in overlay_loading_tests.rs; 4 boot step4 tests; 3 fanout unit tests; 1 end-to-end CapturingAdapter wiring test; negative-path coverage (oversized file, unreadable file, mixed-case org). All changed production code paths have at least one driving test. |
| 4 | Demo Evidence | PASS -- 7 ACs x (gif + webm + tape) = 21 recording artifacts + evidence-report.md. All ACs have both success and error path coverage. |
| 5 | Commit Quality | PASS -- conventional commits; story ID in every message; clear intent per commit. |
| 6 | Diff Size | ADVISORY -- 4760 lines total, but ~2015 are test code, ~1500 are demo evidence binaries, ~59 are a new shell script. Net production code delta ~1670 LOC is reasonable for a new subsystem. |
| 7 | Missing Changes | No missing changes detected against the 7 ACs. |
| 8 | Dependency Status | PASS -- no upstream PR dependencies. |

---

## Pass-2 Finding Disposition

### F-PR155-P2-001 (LOW) -- CrowdStrike/Cyberint base_url overlay no-op

**Status: STILL PRESENT (accepted)**

CrowdStrike and Cyberint adapters do not read `sensor_config["base_url"]`. A user
who creates a CrowdStrike overlay with `base_url` gets no error and no behavioral
change. Architecturally correct (those sensors use auth-tied base URLs). Future
story concern. No change from pass-2 assessment.

**Severity: LOW** (design gap for future story)

---

### F-PR155-P2-002 (NIT) -- Doc comment says "sensor_id_string" but type is SensorId

**Status: STILL PRESENT**

boot.rs lines 124 and 150 still say `Key: (OrgSlug, sensor_id_string)` but the
actual `ResolvedSpecKey` type is `(OrgSlug, SensorId)`. Not addressed in the
pass-2 fix-burst. The fix-burst focused on `sanitize_for_log` for `extends_value`
and the hard-abort boot test.

**Severity: NIT** (doc-comment stale naming)

---

### F-PR155-P2-003 (NIT) -- sanitize_for_log has no direct unit test

**Status: STILL PRESENT**

`sanitize_for_log()` (overlay.rs:752-758) is called in 5 error constructors
(extends_value added in pass-2 fix-burst makes it 5, not 4). Still has no
dedicated unit test exercising control characters or truncation. Indirectly
exercised by the mixed-case org dir test.

**Severity: NIT** (test gap)

---

### F-PR155-P2-004 (LOW) -- SSRF: base_url scheme check allows http://169.254.169.254

**Status: STILL PRESENT (accepted)**

SEC-REDUX-006 validation allows http:// scheme, which includes IMDS endpoints.
Acceptable because overlay files are operator-controlled config, not user-facing input.
No change from pass-2 assessment.

**Severity: LOW** (hardening improvement for future story)

---

### F-PR155-P2-005 (NIT) -- Validation ordering hides SSRF URL behind structural errors

**Status: STILL PRESENT (accepted)**

`validate_overlay_toml` early-returns on structural errors before checking URL
scheme. Acceptable UX (fix one class of error at a time). No change.

**Severity: NIT** (defense-in-depth)

---

## New Findings (Pass-3)

### F-PR155-P3-001 -- OverlayLoadResult missing #[non_exhaustive]

**Severity: NIT**
**Category: convention / non_exhaustive discipline**

**Finding:**

`OverlayLoadResult` (overlay.rs:359-365) is a `pub struct` re-exported from
`prism_spec_engine::lib.rs`. Per CLAUDE.md convention: "All public TOML-deserialized
types and pub-API surface types require `#[non_exhaustive]`." The three TOML-deserialized
overlay types (`SensorInstanceOverlay`, `OverlayProvenance`, `ResolvedSensorSpec`) all
correctly carry `#[non_exhaustive]` and are covered by the compile-fail gate
(violations 33-35). However, `OverlayLoadResult` -- a return-type struct -- does not.

This is a borderline case: `OverlayLoadResult` is a return-only container (like a
`Result` tuple), not a TOML-deserialized config type. Adding `#[non_exhaustive]` would
prevent external crates from constructing it via struct literal, but external crates
have no reason to construct it (only `OverlayLoader::load_overlays` returns it). The
risk is minimal.

**Suggestion:**

Add `#[non_exhaustive]` and a corresponding violation entry (v36) to the compile-fail
gate. This maintains the "all public structs" convention consistently. Not a merge
blocker.

---

## SAP-1 Compliance (tracing event_type catalog)

New `event_type` values in this PR:

| event_type | File | Location |
|-----------|------|----------|
| `overlay.loaded` | overlay.rs | load_overlays() per-file merge success |
| `boot.overlays_loaded` | boot.rs | step4_load_sensor_specs_with_overlays() completion |
| `boot.type_spec_read_failed` | boot.rs | build_type_spec_map_for_overlay() I/O failure |
| `boot.type_spec_parse_failed` | boot.rs | build_type_spec_map_for_overlay() parse failure |

I cannot verify BC-2.16.002 catalog rows directly (information wall). The commit
messages claim ADV-009 added the catalog rows. The orchestrator/adversary should
verify SAP-1 compliance on these 4 entries.

---

## CLAUDE.md Convention Compliance Scan

| Convention | Status |
|-----------|--------|
| `#[non_exhaustive]` on public types | PASS (3 overlay types) + NIT (OverlayLoadResult) |
| No `unwrap()`/`expect()` in non-test code | PASS -- all instances are in `#[cfg(test)]` blocks |
| No `println!` in production code | PASS -- no new println! |
| `tracing::*!` with structured fields | PASS -- all new tracing calls use structured event_type |
| 30s timeout on reqwest::Client | PASS -- all 4 adapters now have `.timeout(Duration::from_secs(30))` |
| No `OrgSlug::new_unchecked` outside test-helpers | PASS -- struct_violations uses `OrgSlug::new("acme")` |
| Error taxonomy codes registered | PASS -- E-SPEC-019..023 have SpecErrorCode enum variants |
| Arc-DI plumbing | PASS -- resolved_spec_map threaded via Arc through RunningServer -> QueryEngine -> MaterializationContext |
| Forbidden pattern: `reqwest::Client::new()` without timeout | PASS -- all 4 adapter constructors + audit_logs pagination client use .timeout(Duration::from_secs(30)) |

---

## Error Handling Review

All error paths use structured `BootError::ConfigInvalid` or `PrismError::Spec(SpecError {...})`
variants. No `unwrap()` or `expect()` in production paths. Parse failures in
`build_type_spec_map_for_overlay` are collected and produce a hard boot abort (PRR-005
closure). I/O errors are propagated via `PrismError::Io`. The multi-error aggregation
pattern (INV-ERR-003) correctly accumulates all overlay errors before returning.

---

## Test Coverage Assessment

| Area | Tests | Quality |
|------|-------|---------|
| Overlay discovery + merge | 3 (AC-001, AC-003 Case A/B, AC-007) | Strong -- covers single-org, two-org, no-base_url |
| Scalar enforcement | 3 (E-SPEC-020/021/023) | Strong -- all three rejection paths |
| OrgRegistry cross-validation | 2 (AC-004, mixed-case) | Strong -- unknown slug + case sensitivity |
| Error taxonomy byte-compare | 1 (AC-005 drives all 5 codes) | Thorough -- include_str() fixture + template rendering |
| Multi-error aggregation | 3 (EC-016-001/002/003) | Thorough -- 2-error, 2-error cross-level, 5-error full sweep |
| Backwards compatibility | 1 (AC-006: absent dir + .gitkeep) | Complete -- both edge cases |
| Rate limit hints merge | 1 (EC-012-005: rps + burst_size independent) | Good |
| Timeout_secs provenance | 1 | Adequate -- provenance tracking verified |
| Boot step4 integration | 4 (absent dir, unknown org, happy path, corrupt TYPE spec) | Thorough |
| Fanout resolution | 3 (Case A, Case B, unknown org) + 1 E2E | Strong -- CapturingAdapter wiring test |
| Negative paths | 3 (oversized file, unreadable, SSRF scheme) | Good security coverage |
| Non-exhaustive gate | 3 violations (v33-v35) | Complete for TOML-deserialized types |

---

## Fix-Burst Regression Check

The pass-2 fix-burst (commit `2d4f82aa`) added:
1. `sanitize_for_log()` applied to `extends_value` in `make_e_spec_019_unknown_extends` --
   correct, consistent with the 4 other sanitized error constructors.
2. `test_F_P2_MED_001_corrupt_type_spec_file_aborts_boot_with_config_invalid` -- new
   boot integration test. Exercises the hard-abort path for corrupt TYPE specs. Well
   structured with proper assertion messages.

Neither change introduces regressions. The `sanitize_for_log` addition is a pure
additive security hardening. The test is isolated in a tempdir.

---

## Summary

| Severity | Count | Finding IDs |
|----------|-------|-------------|
| CRIT | 0 | -- |
| HIGH | 0 | -- |
| MED | 0 | -- |
| LOW | 3 | F-PR155-P2-001 (CS/Cyberint no-op), F-PR155-P2-004 (IMDS SSRF), F-PR155-P2-002 (reclassified -- sensor_id_string doc, see below) |
| NIT | 4 | F-PR155-P2-002 (sensor_id_string doc), F-PR155-P2-003 (sanitize_for_log test), F-PR155-P2-005 (validation ordering), F-PR155-P3-001 (OverlayLoadResult non_exhaustive) |

Correction: F-PR155-P2-002 stays NIT (doc-comment only). Summary should read:

| Severity | Count | Finding IDs |
|----------|-------|-------------|
| CRIT | 0 | -- |
| HIGH | 0 | -- |
| MED | 0 | -- |
| LOW | 2 | F-PR155-P2-001 (CS/Cyberint base_url no-op), F-PR155-P2-004 (IMDS SSRF) |
| NIT | 4 | F-PR155-P2-002 (sensor_id_string doc), F-PR155-P2-003 (sanitize_for_log test), F-PR155-P2-005 (validation ordering), F-PR155-P3-001 (OverlayLoadResult non_exhaustive) |

### Verdict

**CLEAN(strict): no** -- 2 LOW + 4 NIT findings present

**CLEAN(PR-merge): yes** -- zero CRIT + HIGH + MED findings

### Recommendation

**APPROVE** PR #155. All pass-1 and pass-2 findings at HIGH and MED severity have been
closed by the fix-bursts. The remaining findings are:

- 2 LOW: one is an architectural design gap (CS/Cyberint overlay no-op) correctly
  scoped to a future story, and one is a hardening improvement (IMDS SSRF) where the
  threat model requires filesystem-level attacker access.
- 4 NIT: a stale doc comment, a missing unit test for a log sanitizer, a validation
  ordering preference, and a pub struct without `#[non_exhaustive]`.

None of these represent production-grade violations that should block merge. The
production code is well-structured, error handling is comprehensive, test coverage
is thorough (20+ tests covering all ACs plus negative paths), security hardening
exceeds the story scope (file size caps, symlink rejection, URL scheme validation,
log sanitization), and the Arc-DI plumbing is correctly threaded through the full
boot -> query engine -> materialization -> fanout dispatch chain.
