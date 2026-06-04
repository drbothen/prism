---
document_type: story
story_id: S-SPEC-HTTP-METHOD-VALIDATION-001
title: "prism-spec-engine: HTTP Method Whitelist Validation in Sensor Spec (anchors DRIFT-D926-001)"
wave: wave-5-e-demo-fidelity
epic_id: E-SPEC-ENGINE
priority: P2
status: draft
version: "1.1"
level: "L3"
producer: story-writer
timestamp: "2026-05-31T18:00:00Z"
tdd_mode: strict
subsystems: [SS-16]
# Subsystem anchor justifications:
#   SS-16 (Spec Engine) owns `prism-spec-engine/src/validation.rs` where spec-load-time
#   validation runs post env-resolver pass. HTTP method validation is a validation.rs
#   concern — same layer as E-SPEC-024 env-var resolution (BC-2.16.009 v1.6).
crates_touched: [prism-spec-engine]
target_module: prism-spec-engine
capabilities: []
behavioral_contracts:
  - BC-2.16.009  # Spec File Validation — v1.8 (Wave-5 Phase-A PO burst 2026-06-03):
                 # §Validation Rules 7 HTTP Method Whitelist added. Whitelist: GET, POST, PUT,
                 # PATCH, DELETE, HEAD, OPTIONS (7 values; ALLOWED_HTTP_METHODS compile-time const).
                 # E-SPEC-025 assigned and registered in §Error Conditions.
                 # Case-sensitive; absent step.method is NOT an error; multi-error collection
                 # per INV-ERR-003; Rule 7 skips step.method fields that failed Rule 6 (E-SPEC-024).
                 # PO FLAG E-SPEC-NNN code assignment: CLOSED — code is E-SPEC-025 (BC-2.16.009
                 # v1.8 §Error Conditions confirms assignment).
# BC status: BC-2.16.009 is active (lifecycle_status: active — auto-promoted at
# PLUGIN-MIGRATION-001-D merge D-776). v1.8 adds the HTTP method whitelist rule + E-SPEC-025.
# S-7.01 gate CLEARED: behavioral_contracts is non-empty with active BC. Status may transition
# to ready once AC↔BC bidirectional traces are verified at dispatch.
verification_properties: []
depends_on: []
blocks: []
points: 3
# Points justification:
#   - Read validation.rs and confirm insertion point (post env-resolver pass): 0.5 pts
#   - Implement whitelist constant + validation fn: 0.5 pts
#   - Error return using E-SPEC-025 (confirmed by PO in BC-2.16.009 v1.8): 0.5 pts
#   - Unit tests: 3 ACs × ~0.5 pts each = 1.5 pts
#   Total: 3 points
estimated_days: 0.5
risk: LOW
# Risk justification:
#   The safe GET fallback already exists in the pipeline — this is hardening only.
#   There is no vulnerability to fix. The scope is a single new validation function
#   in validation.rs plus the E-SPEC-025 error variant. No behavior change for
#   valid sensor specs. Risk of regression is low.
acceptance_criteria_count: 3
red_gate_tests: 3
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "E-SPEC-025 code: CONFIRMED by PO in BC-2.16.009 v1.8 (Wave-5 Phase-A PO burst 2026-06-03).
    The error code is E-SPEC-025. Implementation MUST use this exact code. Do NOT use a
    placeholder. The PO flag in earlier story versions is now CLOSED."
  - "Validation pass ordering: method validation MUST run after env-resolver pass (same
    as E-SPEC-024 env-var resolution). An env-resolved method like 'GET' must be validated
    as 'GET', not as the raw '${env.SENSOR_METHOD}' token."
  - "Safe fallback NOT removed: the `_ => GET` fallback in PipelineExecutor remains in
    place. This story adds EARLY validation so that invalid methods are caught at spec-load
    time before any HTTP request is issued. The fallback is a belt-and-suspenders safety
    net for any method that slips past validation in future code paths."
  - "Hardcoded method paths: the step.method field may be a hardcoded string literal
    (e.g., method = 'GET') OR an env-resolved value. Both paths must be validated against
    the whitelist post env-resolution."
inputs:
  - "crates/prism-spec-engine/src/validation.rs"
  - "crates/prism-spec-engine/src/spec_parser.rs"
  - ".factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md"
  - ".factory/specs/prd-supplements/error-taxonomy.md"
  - "docs/demo-evidence/PLUGIN-MIGRATION-001-A/"
input-hash: null
traces_to: [DRIFT-D926-001]
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-SPEC-HTTP-METHOD-VALIDATION-001 v1.0 — HTTP Method Whitelist Validation in Sensor Spec

**Story ID:** S-SPEC-HTTP-METHOD-VALIDATION-001
**Status:** draft
**Version:** v1.0
**Wave:** wave-5-e-demo-fidelity
**Priority:** P2 (hardening — NOT a vulnerability; safe GET fallback exists)
**Points:** 3

---

## Origin

PR #165 review finding M-001 / SEC-001 (DRIFT-D926-001). The finding was dispositioned as a
follow-up hardening story rather than an in-PR fix:

> Sensor spec `[[tables.fetch_steps]]` steps accept a `step.method` field whose value is not
> validated against an HTTP-method whitelist at spec-load time. Environment-variable-resolved
> methods (via `${env.VAR}`) and hardcoded method strings are both unvalidated. The pipeline
> has a `_ => GET` fallback, so this is NOT a vulnerability — invalid methods fall back to
> GET silently rather than executing a dangerous method. However, invalid methods
> (CONNECT/TRACE/typos like "GETT") silently produce GET behavior instead of an actionable
> error, which is a spec-load correctness gap.

**DRIFT-D926-001 anchor:** This story anchors DRIFT-D926-001. The drift item is RESOLVED
when this story merges.

**Safety note:** The existing `_ => GET` fallback in `PipelineExecutor` is load-bearing.
This story does NOT remove it. This story adds EARLY validation so that invalid/unsupported
methods are caught at spec-load time and reported as a structured `E-SPEC-025` error,
giving operators an actionable signal instead of silent fallback.

---

## Narrative

As a Prism platform operator, I want invalid or unsupported HTTP methods in sensor spec
`[[tables.fetch_steps]]` blocks to produce an actionable structured error at spec-load time
(rather than silently falling back to GET), so that typos and unsupported methods like
CONNECT or TRACE are caught before the pipeline runs.

---

## Story-Level Goal

After this story merges:

1. `crates/prism-spec-engine/src/validation.rs` contains a `validate_step_methods()` function
   that runs post env-resolver pass (Rule 7, after Rule 6) and checks every `step.method` value
   against the whitelist per BC-2.16.009 v1.8 §Validation Rules 7.
2. The whitelist `const ALLOWED_HTTP_METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"]`
   is a compile-time constant in `validation.rs`. Methods not on the whitelist produce a structured
   `E-SPEC-025` error (confirmed assigned by PO in BC-2.16.009 v1.8 §Error Conditions).
3. The `_=>GET` fallback in `PipelineExecutor` remains in place as a belt-and-suspenders
   safety net (BC-2.16.009 v1.8 §Validation Rules 7 "belt-and-suspenders" clause).
4. BC-2.16.009 is ALREADY amended to v1.8 (Wave-5 Phase-A PO burst 2026-06-03). The PO
   flag is CLOSED. The implementer reads the BC but does NOT amend it in this story's PR.

---

## Behavioral Contracts

| BC ID | Version | Title | Role in This Story |
|-------|---------|-------|-------------------|
| BC-2.16.009 | v1.8 | Spec File Validation | §Validation Rules 7 HTTP Method Whitelist Validation (AC-7). E-SPEC-025 assigned. Whitelist: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`. Rule 7 runs AFTER Rule 6 env-var resolution; absent `step.method` is NOT an error; wrong-case/unsupported methods produce E-SPEC-025; multi-error collection per INV-ERR-003. This story implements the full §Validation Rules 7 specification from BC-2.16.009 v1.8. |

**E-SPEC-025 — CONFIRMED ASSIGNED (BC-2.16.009 v1.8 §Error Conditions):**
E-SPEC-025 is the canonical code for "invalid or unsupported HTTP method in sensor spec step".
The PO flag is CLOSED. Error message template (per BC-2.16.009 v1.8):
`"Step '<step_name>' in '<sensor_id>.<table_name>' declares method '<method_value>' which is not a supported HTTP method. Supported: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS"`

The method value is safe to echo (config text, not a credential per AD-017).

---

## Acceptance Criteria

### AC-001: Valid HTTP method in step.method passes validation (all 7 whitelist members)
Given: A sensor spec with a `[[tables.fetch_steps]]` block where `step.method` is set to any
of the 7 whitelist values: `"GET"`, `"POST"`, `"PUT"`, `"PATCH"`, `"DELETE"`, `"HEAD"`, `"OPTIONS"`
— either hardcoded or env-resolved. The whitelist is expressed as
`const ALLOWED_HTTP_METHODS: &[&str]` in `crates/prism-spec-engine/src/validation.rs`.
When: The spec is loaded (post env-resolver pass, Rule 7).
Then: No `E-SPEC-025` error is emitted; spec loads successfully.
(traces to BC-2.16.009 v1.8 §Validation Rules 7 — valid method passes; BC-2.16.009 v1.8
§Canonical Test Vectors: "HTTP method — valid GET" and "HTTP method — valid POST (Claroty pattern)")

Red Gate test: `test_BC_2_16_009_valid_http_method_passes_validation`

### AC-002: Invalid or unsupported method in step.method returns structured E-SPEC-025 error
Given: A sensor spec with a `[[tables.fetch_steps]]` block where `step.method` is set to an
invalid or unsupported value (e.g., `"CONNECT"`, `"TRACE"`, `"GETT"`, `"get"` [wrong case],
`""`).
When: The spec is loaded (post env-resolver pass, Rule 7 whitelist check runs).
Then: A structured `E-SPEC-025` error is returned with the step_name, sensor_id, table_name,
and the invalid method value in the message, matching the BC-2.16.009 v1.8 message template:
`"Step '<step_name>' in '<sensor_id>.<table_name>' declares method '<method_value>' which is not a supported HTTP method. Supported: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS"`.
Spec is rejected at load time. The pipeline `_=>GET` fallback is NEVER reached for this spec.
(traces to BC-2.16.009 v1.8 §Validation Rules 7 — invalid method produces E-SPEC-025;
§Error Conditions E-SPEC-025 canonical entry; §Canonical Test Vectors "HTTP method — CONNECT rejected"
and "HTTP method — lowercase rejected")

Red Gate test: `test_BC_2_16_009_invalid_http_method_returns_structured_e_spec_025`

### AC-003: Env-resolved method is validated post-resolution (Rule 7 runs after Rule 6)
Given: A sensor spec with `step.method = "${env.SENSOR_STEP_METHOD}"` and the environment
variable `SENSOR_STEP_METHOD` set to `"CONNECT"` (an unsupported method).
When: The spec is loaded (Rule 6 env-var resolution runs first; then Rule 7 whitelist check runs).
Then: Rule 7 runs on the resolved value `"CONNECT"`, not the raw `"${env.SENSOR_STEP_METHOD}"` token.
An `E-SPEC-025` error is returned citing the resolved method name `"CONNECT"` and the step.
If `SENSOR_STEP_METHOD` were unset instead, Rule 6 fires `E-SPEC-024` first; Rule 7 SKIPS
that step (double-reporting prevention per BC-2.16.009 v1.8 §Validation Rules 7 ordering).
(traces to BC-2.16.009 v1.8 §Validation Rules 7 — Rule 7 ordering: runs AFTER Rule 6;
§Canonical Test Vectors "HTTP method — env-resolved invalid" + EC-009-019/020)

Red Gate test: `test_BC_2_16_009_env_resolved_invalid_method_caught_post_resolution`

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Validation runs POST env-resolver pass | BC-2.16.009 v1.6 §Validation Rules 6 precedent (E-SPEC-024 runs in same pass) | Ensure `validate_step_methods()` is called after `resolve_env_vars()` in `validation.rs` |
| Whitelist constant is a compile-time array | Production-grade default | No runtime config; no serde; `const ALLOWED_HTTP_METHODS: &[&str]` |
| `_ => GET` fallback in `PipelineExecutor` is NOT removed | Belt-and-suspenders safety | Removing the fallback is out of scope; confirmed in risk_mitigations |
| `E-SPEC-025` MUST be confirmed in `error-taxonomy.md` before implementation | POL-1 (append-only error taxonomy) — already registered by PO in Wave-5 Phase-A burst | Implementer reads error-taxonomy.md to confirm E-SPEC-025 row exists before writing error-emitting code |
| Method value is safe to echo in error message | Not a credential per AD-017 | HTTP method string is config text; echoing is fine |
| Multi-error collection: all invalid methods collected before error return | INV-ERR-003 pattern | Multiple invalid steps → multiple errors in same pass (same as E-SPEC-024 behavior) |

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `prism-spec-engine` (workspace) | current workspace path | validation.rs home for new function |
| No new dependencies required | — | Whitelist is a `const` array; no external crate needed |

Version source: `Cargo.toml` workspace. No independent version pins.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-spec-engine/src/validation.rs` | MODIFY | Add `validate_step_methods()` fn + `ALLOWED_HTTP_METHODS` constant; wire into validation pipeline post env-resolver |
| `crates/prism-spec-engine/src/spec_parser.rs` | READ (no modify expected) | Confirm `step.method` field name and type in `FetchStep` struct |
| `.factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md` | READ ONLY | Already amended to v1.8 by PO (Wave-5 Phase-A PO burst 2026-06-03). Implementer reads to confirm §Validation Rules 7 + E-SPEC-025. Do NOT amend in this story's PR. |
| `.factory/specs/prd-supplements/error-taxonomy.md` | READ ONLY | E-SPEC-025 already registered by PO in same burst. Implementer confirms row before writing error-emitting code. |

---

## Tasks

1. **Read** BC-2.16.009 — understand §Validation Rules structure; confirm insertion point
   for new rule (after env-var resolution, before URL-format checks).
2. **Read** `crates/prism-spec-engine/src/spec_parser.rs` — confirm `FetchStep` struct field
   name for HTTP method (likely `method: Option<String>` or `method: String`).
3. **Read** `crates/prism-spec-engine/src/validation.rs` — locate where env-var resolution
   and other validation rules run; identify the correct call site for `validate_step_methods()`.
4. **Confirm** E-SPEC-025 is registered in `error-taxonomy.md` and in BC-2.16.009 v1.8
   §Error Conditions. This was done by PO in the Wave-5 Phase-A burst (2026-06-03). The
   implementer reads both to confirm before writing error-emitting code. Use `E-SPEC-025`
   — no placeholder.
5. **Write stub** `validate_step_methods()` in `validation.rs` with `todo!()` body.
6. **Write Red Gate tests** (AC-001, AC-002, AC-003 test names listed above) — all must fail
   (RED) before implementation.
7. **Implement** `validate_step_methods()`:
   - Define `const ALLOWED_HTTP_METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];`
   - For each `FetchStep` in every table in the spec: check `step.method` against the whitelist
     (case-sensitive — `"get"` is invalid per typical HTTP client conventions; implementer
     may normalize to uppercase if BC-2.16.009 amendment specifies; PO decides).
   - Collect all errors (multi-error pattern per INV-ERR-003).
   - Return `Err(SpecEngineError::...(E-SPEC-025))` for any invalid method.
8. **Wire** `validate_step_methods()` into the validation pipeline — call it after the
   env-resolver pass, before the spec is accepted.
9. **Run tests**: `just iter prism-spec-engine` — all 3 Red Gate tests GREEN.
10. **Run** `just check` — final pre-push gate.
11. **Verify PO artifacts** (READ ONLY — already done by PO): confirm `error-taxonomy.md`
    contains E-SPEC-025 row and BC-2.16.009 v1.8 §Error Conditions contains E-SPEC-025.
    No PO amendment needed in this story's PR — the BC and taxonomy were updated in the
    Wave-5 Phase-A PO burst (2026-06-03).

---

## Previous Story Intelligence

- **S-SPEC-ENV-VAR-001** (parallel, wave-5): Added E-SPEC-024 and BC-2.16.009 §Validation
  Rules 6 for env-var token resolution. Pattern for this story: same validation.rs insertion
  point, same multi-error collection, same post-env-resolver ordering. Read S-SPEC-ENV-VAR-001
  and its implementation before writing `validate_step_methods()`.
- **BC-2.16.009 v1.8** (current, Wave-5 Phase-A PO burst 2026-06-03): Added §Validation
  Rules 7 HTTP method whitelist + E-SPEC-025. The BC is already at v1.8. No further PO
  amendment needed. Implementer reads v1.8 as the authoritative spec for this story.
- This is the first story in the HTTP-method-validation sub-track; no predecessor lessons.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `step.method` is absent / `None` | No validation error — absent method defaults to GET in the pipeline; absence is not invalid |
| EC-002 | `step.method = "get"` (lowercase) | Invalid per whitelist (case-sensitive); E-SPEC-025 error. BC-2.16.009 v1.8 §Validation Rules 7: case-sensitive match; `"get"` is NOT equivalent to `"GET"`. |
| EC-003 | `step.method = "CONNECT"` | Invalid (unsupported, potentially dangerous); E-SPEC-025 error |
| EC-004 | `step.method = "TRACE"` | Invalid (unsupported); E-SPEC-025 error |
| EC-005 | `step.method = "GETT"` (typo) | Invalid; E-SPEC-025 error |
| EC-006 | `step.method = ""` (empty string) | Invalid; E-SPEC-025 error |
| EC-007 | Multiple invalid method steps in same spec | All invalid steps collected; multiple E-SPEC-025 errors in same pass (INV-ERR-003; BC-2.16.009 v1.8 multi-error collection clause) |
| EC-008 | `step.method = "${env.METHOD}"` with `METHOD="CONNECT"` | Env resolves to `"CONNECT"`; then method validation fires; E-SPEC-025 error citing resolved value `"CONNECT"` |
| EC-009 | `step.method = "${env.METHOD}"` with `METHOD` unset | E-SPEC-024 fires for unresolved env var (existing behavior); method validation does not run on unresolved token |
| EC-010 | Valid method `"POST"` for a write step | Passes validation; no error |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~3,500 |
| BC-2.16.009 (relevant sections only) | ~3,000 |
| crates/prism-spec-engine/src/validation.rs | ~4,000 |
| crates/prism-spec-engine/src/spec_parser.rs (FetchStep struct) | ~2,000 |
| error-taxonomy.md (E-SPEC section only) | ~2,000 |
| Test outputs (cargo nextest) | ~1,000 |
| **Total estimate** | **~15,500 tokens (~6% of 256K context)** |

Well within budget. Single-story delivery is straightforward.

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-06-03 | story-writer | Wave-5 Phase-A BC-array propagation burst (D-989). PO authored BC-2.16.009 v1.8 with §Validation Rules 7 + E-SPEC-025 assigned. Propagated into story: (1) `behavioral_contracts` frontmatter updated with v1.8 commentary; PO flag CLOSED. (2) Added §Behavioral Contracts table with BC-2.16.009 v1.8 role. (3) ACs rewritten: AC-001 cites all 7 whitelist methods + BC-2.16.009 v1.8 §Validation Rules 7; AC-002 cites E-SPEC-025 explicitly (not placeholder) + confirmed message template; AC-003 cites Rule 7 ordering clause + EC-009-019/020. (4) Story-Level Goal updated: E-SPEC-025 confirmed; BC already at v1.8; no PO amendment in this PR. (5) risk_mitigations: E-SPEC-NNN placeholder → E-SPEC-025 CONFIRMED. (6) Tasks 4+11 updated. (7) FSR: BC + taxonomy marked READ ONLY. (8) Previous Story Intelligence: BC v1.6→v1.8. Version bump 1.0 → 1.1. |
| 1.0 | 2026-05-31 | story-writer | Initial draft — anchors DRIFT-D926-001 per PR #165 M-001/SEC-001 disposition. HTTP-method whitelist validation in validation.rs post env-resolver pass. PO flag: NEW E-SPEC-NNN code required (next available E-SPEC-025 as of 2026-05-31 but PO assigns). Status: draft pending BC-2.16.009 amendment + E-SPEC-NNN code assignment per S-7.01 gate. |
