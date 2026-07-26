---
document_type: story
story_id: S-ADR055-WAVE-A-001
title: "Wire validate_sensor_spec() into Production Spec-Loading Pipeline — parse_and_validate_spec_toml() and SpecLoader::load_all()"
version: "1.1"
status: draft
producer: story-writer
phase: 3
wave: wave-a
epic_id: E-WAVE-A-SENSOR-REMEDIATION
priority: P0
points: 8
tdd_mode: strict
target_module: prism-spec-engine
subsystems: ["SS-06 (SensorSpec)"]
depends_on: []
blocks: []
behavioral_contracts:
  - BC-2.16.009
  - BC-2.16.001
  - BC-2.16.007
  - BC-2.16.008
  - BC-2.16.002
verification_properties:
  - VP-059
estimated_days: 3
# BC status: BC-2.16.009 §Description requires a documentation correction (PO task per ADR-055 §Story Scope)
# to state that parse_and_validate_spec_toml() now composes Rules 1–7. This is not a behavioral change —
# it does not block status: ready. BC-2.16.002 requires a new event_type = "spec.validation_warning" catalog
# row (SAP-1 obligation); see Product-Owner Dependencies section. All four BCs exist and are currently active.
assumption_validations: []
risk_mitigations: []
---

# S-ADR055-WAVE-A-001: Wire validate_sensor_spec() into Production Spec-Loading Pipeline

## Authority

ADR-055 v1.0 (2026-07-25) is the authoritative design document for this story. Read it
before implementing: `.factory/specs/architecture/decisions/ADR-055-validate-sensor-spec-production-wiring.md`.

---

## Narrative

As a Prism maintainer, I want `validate_sensor_spec()` called on every spec-loading path
— startup (`SpecLoader::load_all()`), hot-reload, and the `add_sensor_spec` MCP tool
(`parse_and_validate_spec_toml()`) — so that BC-2.16.009 Rules 1–5 are enforced in
production, a sensor spec with an invalid `base_url` is rejected at load time rather than
at query runtime, and SAP-3 is satisfied: EC-009-005 (`base_url` scheme rejection) is
reachable from the `add_sensor_spec` public surface.

---

## Finding Origin

Finding F-WASE-P63-HIGH-003: `validate_sensor_spec()` has zero production callers.
Workspace-wide grep for `validate_sensor_spec(` returns only: `examples/`, `src/proofs/`,
and `tests/` — no production paths. BC-2.16.009 Rules 1–5 are unenforced in production
today. The fix is surgical: two call sites added, two source files changed, one crate
affected (`prism-spec-engine`).

---

## Acceptance Criteria

### AC-001: parse_and_validate_spec_toml() rejects bad base_url via validate_sensor_spec()
(traces to BC-2.16.009 Rule 1 postcondition — base_url must start with http:// or https://)

A call to `parse_and_validate_spec_toml()` with a TOML containing
`base_url = "ftp://evil.example.com"` returns `Err(vec![ValidationError { ... }])` with
an error string matching E-SPEC-001 and citing `sensor.base_url` as the invalid field.

The test invokes `parse_and_validate_spec_toml()` directly (not `validate_sensor_spec()`
directly) — this satisfies SAP-3 reachability from the integration function's public
surface. The existing tests in `bc_2_16_009_test.rs` that call `validate_sensor_spec()`
directly are retained as defense-in-depth but are NOT sufficient for SAP-3 compliance.

### AC-002: EC-009-005 is reachable from add_sensor_spec() API surface
(traces to BC-2.16.009 Rule 1 postcondition — base_url scheme rejection is an E-SPEC-001 error)

A call to the `add_sensor_spec()` function (the public API in `src/add_sensor_spec.rs`)
with a TOML containing `base_url = "not-a-url"` returns a response containing an
E-SPEC-001 error. The test path is: `add_sensor_spec()` → `parse_and_validate_spec_toml()`
→ `validate_sensor_spec()` → Rule 1 base_url check → error.

This is the SAP-3 compliance test for EC-009-005: the error is reachable end-to-end from
the MCP tool's backing function, not just from the validator in isolation.

### AC-003: Multi-error collect-all — two errors surface together
(traces to BC-2.16.009 §Invariants postcondition — single-pass, all-errors-collected)

A TOML with BOTH `base_url = "ftp://evil.example.com"` AND a dangling variable reference
(`path_template = "/api/${nonexistent.field}/items"`) submitted via
`parse_and_validate_spec_toml()` returns a `ValidationError` whose `errors` vec contains
AT LEAST two entries — one for the base_url violation and one for the undefined variable.

The response carries both errors in a single function call, not sequentially (not fail-fast
on first error). This validates the collect-all invariant (VP-059).

### AC-004: Bundled specs still load clean after wiring
(traces to BC-2.16.001 postcondition — all bundled specs load without error at startup)

After wiring `validate_sensor_spec()` into `load_all()`, the bundled spec load test
(`tests/bc_2_16_001_bundled_spec_load.rs`) passes with all specs in
`crates/prism-sensors/specs/` loading without error.

This test must remain green: no bundled production spec must newly fail after this story.

### AC-005: SENSOR_ID_RE check is retained in parse_and_validate_spec_toml()
(traces to BC-2.16.009 §Security postcondition — CWE-22 path-traversal defense-in-depth Layer 1)

The `SENSOR_ID_RE` regex check (SEC-001) remains in `parse_and_validate_spec_toml()` at
its current position in the control flow. It is NOT removed, commented out, or relocated
inside `validate_sensor_spec()`. The check's presence can be verified by a unit test that
submits a TOML with `sensor_id = "../../etc/passwd"` and asserts rejection before
`validate_sensor_spec()` runs — i.e., at the SEC-001 layer.

### AC-006: Hot-reload path enforces Rules 1–5 via parse_and_validate_spec_toml()
(traces to BC-2.16.007 postcondition — hot-reload validates specs with the same rules as startup)

The hot-reload path (`hot_reload.rs::process_spec_changes()`) reaches Rules 1–5 via
`parse_and_validate_spec_toml()`. Because `process_spec_changes()` calls
`parse_and_validate_spec_toml()`, no additional change is needed to `hot_reload.rs` —
AC-006 is satisfied by verifying that `parse_and_validate_spec_toml()` is the entry point
for both Added and Modified events, and that AC-001 demonstrates the wiring is complete.

A comment must be added to `hot_reload.rs::process_spec_changes()` citing
`parse_and_validate_spec_toml()` as the composed validation entry point that covers
Rules 1–7, so future readers do not re-add individual rule checks.

### AC-007: Startup path (load_all()) enforces Rules 1–5 with collect-all semantics
(traces to BC-2.16.001 postcondition — startup rejects semantically invalid specs)

A test invokes `SpecLoader::load_all()` (or a test-helper equivalent that exercises the
`load_all` code path) with a directory containing one valid spec and one spec with
`base_url = "ftp://x"`. The function:
- Loads the valid spec successfully
- Rejects the invalid spec with an E-SPEC-001 error in the returned error collection
- Does NOT abort the entire load due to the invalid spec (multi-spec collect-all behavior)

The test verifies the multi-spec load-all pattern: one bad spec does not prevent other
specs from loading (DI-030 semantics per ADR-055 §D2).

### AC-008: validate_sensor_spec() runs AFTER resolve_env_var_tokens() on all paths
(traces to BC-2.16.009 Rule 6 + Rule 1 ordering invariant — env-var tokens must be resolved
before the base_url scheme check runs)

The ordering in `parse_and_validate_spec_toml()` is verified by reading the function body:
1. `SpecLoader::parse()` first
2. `resolve_env_var_tokens()` second
3. `validate_step_methods()` (Rule 7) third
4. `validate_sensor_spec()` [NEW] fourth

A spec with `base_url = "${env.ARMIS_INSTANCE_URL}"` and a matching env var set to
`https://armis.acme.io` must load without error (env-var resolution runs before Rule 1
base_url check). Without resolution ordering, unresolved token `${env.ARMIS_INSTANCE_URL}`
fails the `http://`/`https://` scheme check.

### AC-009: Test fixture audit completed — no test fixture newly fails after wiring
(traces to BC-2.16.009 §Behavioral Change invariant — wiring must not break existing tests)

All test fixtures in `crates/prism-spec-engine/tests/` and `crates/prism-spec-engine/src/`
have been audited for inputs that would newly trigger a Rules 1–5 rejection after wiring.
Any fixture that relied on `parse_and_validate_spec_toml()` accepting a spec that violates
Rules 1–5 (e.g., using `http://localhost:8080` without having a scheme before the wiring)
is updated to pass a valid spec or to explicitly test the now-correctly-rejected invalid spec.

The audit is documented in a `// FIXTURE-AUDIT-ADR055:` comment at the top of
`tests/bc_2_16_009_test.rs` listing each fixture reviewed and its outcome (PASS-unchanged /
UPDATED / ADDED-invalid-as-new-error-case). This AC is BLOCKING — the story is NOT done
until the audit comment is present and all tests pass.

### AC-010: spec.validation_warning tracing emission is present (SAP-1 gate)
(traces to BC-2.16.002 §Postconditions Canonical Structured Event Catalog — every new
event_type must have a catalog row in the same commit)

When `validate_sensor_spec()` returns `Ok(warnings)` (warnings present but no errors),
`parse_and_validate_spec_toml()` emits:
```
tracing::warn!(
    event_type = "spec.validation_warning",
    sensor_id = %spec.sensor_id,
    toml_path = %source_path,
    "sensor spec validation warning: {warning_message}"
);
```
for each warning in the `Ok(warnings)` vec.

**SAP-1 blocking gate:** The `event_type = "spec.validation_warning"` tracing emission
requires a corresponding row in the BC-2.16.002 §Postconditions Canonical Structured Event
Catalog before this story's PR can merge. This is a product-owner dependency (BC-2.16.002
amendment). The implementer adds the `tracing::warn!` call; the PR cannot merge until the
PO has added the catalog row. See Product-Owner Dependencies section.

---

## Product-Owner Dependencies

These two items are PO tasks that BLOCK this story's PR merge, but do NOT block
implementation dispatch:

### PO-001: BC-2.16.002 catalog row for spec.validation_warning (SAP-1)

**Blocker level:** PR merge gate (the PR cannot merge without this row)
**What:** Add an `event_type = "spec.validation_warning"` row to BC-2.16.002 §Postconditions
(Canonical Structured Event Catalog) with:
- `event_type`: `spec.validation_warning`
- `fields`: `sensor_id` (string), `toml_path` (string), message (via tracing format string)
- `audit_role`: `spec-load-diagnostic` (or the appropriate role from the catalog convention)
- `recurrence_policy`: once per warning per spec per load cycle
- `emitter`: `parse_and_validate_spec_toml()`

**Why now:** SAP-1 / CLAUDE.md §Conventions structured event catalog discipline. A new
`event_type =` emission without a same-commit catalog row is a P1 finding. Since the PO
owns BC-2.16.002, the implementer cannot add the catalog row; the PO must add it before
the PR merges.

### PO-002: BC-2.16.009 documentation correction (non-behavioral)

**Blocker level:** non-blocking (MAY follow the story merge)
**What:** Update BC-2.16.009 §Description "Integration function" paragraph to state that
`parse_and_validate_spec_toml()` now composes Rules 1–7 (not only Rules 6 and 7); update
the `load_all()` paragraph similarly.
**Why not blocking:** This is a documentation correction reflecting existing behavior after
the story lands. It does not change any contract postcondition or invariant.

---

## Architecture Mapping

| Component | File | Pure/Effectful | Change |
|-----------|------|---------------|--------|
| `parse_and_validate_spec_toml()` | `src/add_sensor_spec.rs` | Pure (validation function) | Add `validate_sensor_spec()` call per ADR-055 §D1 |
| `SpecLoader::load_all()` | `src/spec_parser.rs` | Effectful (disk reads + error collection) | Add `validate_sensor_spec()` call per ADR-055 §D2 |
| `process_spec_changes()` | `src/hot_reload.rs` | Effectful (filesystem watch) | No code change — add comment citing composed entry point |
| `validate_sensor_spec()` | `src/validation.rs` | Pure (validation only) | No change — this function is already correct |

Architecture section references:
- `architecture/module-decomposition.md §SS-06 SensorSpec` — subsystem boundary
- `architecture/dependency-graph.md §prism-spec-engine` — crate dependencies

---

## Behavioral Contracts

| BC | Version | Relevance |
|----|---------|-----------|
| BC-2.16.009 | v1.28 | Rules 1–5; Rule 1 base_url scheme gate; collect-all invariant (VP-059); SENSOR_ID_RE SEC-001 |
| BC-2.16.001 | v1.9 | Bundled spec load at startup; AC-004 |
| BC-2.16.007 | v1.7 | Hot-reload path; AC-006 |
| BC-2.16.008 | v1.6 | add_sensor_spec MCP tool; AC-002 SAP-3 surface |
| BC-2.16.002 | v2.11 | Canonical Structured Event Catalog; PO-001 catalog row for spec.validation_warning |

---

## Implementation Snippets (from ADR-055)

### §D1 — parse_and_validate_spec_toml() wiring (src/add_sensor_spec.rs)

After the `validate_step_methods()` call, before the required-fields non-empty block:

```rust
if let Err(spec_errors) = validate_sensor_spec(&spec) {
    return Err(vec![ValidationError {
        sensor_id: Some(spec.sensor_id.clone()),
        source_path: source_path.to_string(),
        errors: spec_errors.into_iter().map(|e| format!("{e}")).collect(),
    }]);
}
```

### §D2 — SpecLoader::load_all() wiring (src/spec_parser.rs)

In the multi-spec collection loop, after `validate_step_methods()`:

```rust
// BC-2.16.009 Rules 1–5 — semantic validation (post env-var resolution).
// Runs AFTER Rule 6 (env-var) and Rule 7 (HTTP methods) per ADR-055 §D4.
let semantic_errors = crate::validation::validate_sensor_spec(&spec);
if let Err(sem_errs) = semantic_errors {
    for e in sem_errs {
        errors.push(PrismError::Spec(SpecError {
            code: e.code,
            message: e.message,
            toml_path: e.toml_path,
            file_path: Some(file_name.clone()),
            line_number: None,
        }));
    }
    // DI-030: reject this spec, continue loading others (collect-all semantics).
    continue;
}
```

### §D6 — Warning surfacing (src/add_sensor_spec.rs)

After the `validate_sensor_spec()` Err branch, handle the Ok(warnings) path:

```rust
if let Ok(warnings) = validate_sensor_spec(&spec) {
    for w in &warnings {
        tracing::warn!(
            event_type = "spec.validation_warning",
            sensor_id = %spec.sensor_id,
            toml_path = %source_path,
            "sensor spec validation warning: {}",
            w
        );
    }
}
```

The `event_type = "spec.validation_warning"` emission requires a PO-001 catalog row before PR merge (SAP-1).

---

## UX Surfaces

None — no user-facing UI changes. The MCP tool `add_sensor_spec` returns richer validation
errors to the calling LLM agent when a spec violates Rules 1–5, but the error shape
(`isError: true` / `structuredContent.error`) is unchanged (that remap is S-WAVE-A-MCP-001).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | base_url = "http://localhost:8080" (no env var token) | AC-001: PASSES Rule 1 (http:// scheme is valid) — must not be broken by wiring |
| EC-002 | base_url = "${env.ARMIS_INSTANCE_URL}" with env var set to https://armis.acme.io | AC-008: PASSES after env-var resolution — must not fail due to unresolved token |
| EC-003 | base_url = "${env.ARMIS_INSTANCE_URL}" with env var NOT set | resolve_env_var_tokens() returns error before validate_sensor_spec() runs — correct behavior, error is at Rule 6 not Rule 1 |
| EC-004 | spec with both a bad base_url AND an empty table_name | AC-003: BOTH errors are collected and returned (collect-all invariant) |
| EC-005 | spec passes Rules 1–5 but has an OCSF-class warning | AC-010: tracing::warn! fires; no error returned; spec loads successfully |
| EC-006 | validate_sensor_spec() returns Ok([]) (no warnings, no errors) | No tracing emission; spec loads normally |
| EC-007 | customer overlay spec (e.g., customers/acme/armis.sensor.toml) lacks a standalone base_url | Overlay specs may have different validation semantics; verify that load_all() applies validate_sensor_spec() only to fully-resolved (non-overlay) specs, or that overlay specs pass validation without a base_url |
| EC-008 | Test fixture that previously used base_url = "localhost:8080" (without http:// prefix) | AC-009 fixture audit catches this; fixture must be updated to "http://localhost:8080" |

---

## Tasks

### T-01: Read validate_sensor_spec() and SpecError types before implementing
**File:** `crates/prism-spec-engine/src/validation.rs`

Read the full function signature, return type `ValidatorOutput = Result<Vec<ValidationWarning>, Vec<SpecError>>`,
and the `SpecError` struct fields (`code`, `message`, `toml_path`). Verify that `SpecError`
fields match what ADR-055 §D2 wires into `PrismError::Spec(SpecError { ... })`.

Do NOT implement from memory or from the ADR snippets alone — read the actual source.

### T-02: Read parse_and_validate_spec_toml() before implementing §D1
**File:** `crates/prism-spec-engine/src/add_sensor_spec.rs`

Identify the exact insertion point: after `validate_step_methods()` and before the
required-fields non-empty block. Confirm the `ValidationError` type's `errors: Vec<String>`
field exists and matches the ADR-055 §D1 snippet.

Confirm `SENSOR_ID_RE` is present in this function. Do NOT move or remove it.

### T-03: Read SpecLoader::load_all() before implementing §D2
**File:** `crates/prism-spec-engine/src/spec_parser.rs`

Identify the exact insertion point in the multi-spec loop. Verify the error accumulation
pattern already in use for Rule 7 findings (the `errors.push(PrismError::Spec(...))` +
`continue` pattern). Confirm the `SpecError` constructor matches the ADR-055 §D2 snippet.

### T-04: Run test fixture audit (MANDATORY — AC-009)
**Scope:** `crates/prism-spec-engine/tests/` and all TOML literals in `crates/prism-spec-engine/src/`

Grep for `base_url` values in test fixtures. For each:
- If it starts with `http://` or `https://`, it will PASS Rule 1 — no change needed
- If it starts with `${env.`, it will be resolved before Rule 1 runs — no change needed
- If it starts with anything else (bare hostname, `ftp://`, empty, etc.), it will NEWLY FAIL

For each newly-failing fixture:
- If it is a valid-spec fixture (testing happy-path behavior): update `base_url` to `http://localhost:PORT` or `https://valid.example.com`
- If it is an invalid-spec fixture (testing error cases): the fixture is now correctly rejected — update the test to expect an error rather than a load success

Add the `// FIXTURE-AUDIT-ADR055:` comment block at the top of `tests/bc_2_16_009_test.rs`
listing every fixture reviewed and its disposition (see AC-009).

Also check for: empty `table_name`, missing `columns`, `fan_out_batch_size = 0`, `page_size = 0`,
`cursor_response_path = ""`, `rate_limit_hints.requests_per_second` ≤ 0. Each of these newly
fails under Rules 1–5. Update any such fixtures that are not specifically testing error cases.

### T-05: Implement §D1 — wire into parse_and_validate_spec_toml()
**File:** `crates/prism-spec-engine/src/add_sensor_spec.rs`

Insert the `validate_sensor_spec()` call per ADR-055 §D1 snippet. Add the warning
surfacing per §D6 snippet. Verify SENSOR_ID_RE is still present and unchanged (AC-005).

After implementing: run `cargo nextest run -p prism-spec-engine --no-fail-fast` and verify
only the EXPECTED new test failures appear (AC-001, AC-002 tests will initially be RED if
not yet written).

### T-06: Implement §D2 — wire into SpecLoader::load_all()
**File:** `crates/prism-spec-engine/src/spec_parser.rs`

Insert the `validate_sensor_spec()` call per ADR-055 §D2 snippet.

After implementing: run `cargo nextest run -p prism-spec-engine -E 'test(bundled)'` and
verify AC-004 (bundled spec load test) remains green.

### T-07: Add hot-reload comment (AC-006)
**File:** `crates/prism-spec-engine/src/hot_reload.rs`

Add a code comment at the `parse_and_validate_spec_toml()` call site in
`process_spec_changes()` stating:
```
// parse_and_validate_spec_toml() composes Rules 1–7 (ADR-055 §D1):
// TOML parse → env-var resolution → HTTP methods (Rule 7) → semantic validation (Rules 1–5).
// No additional rule checks are needed here.
```

### T-08: Write AC-001 test (SAP-3 base_url via parse_and_validate_spec_toml)
**File:** `crates/prism-spec-engine/tests/bc_2_16_009_test.rs` (or a new test file)

Write a test that:
1. Constructs a TOML string with `base_url = "ftp://evil.example.com"`
2. Calls `parse_and_validate_spec_toml()` (NOT `validate_sensor_spec()` directly)
3. Asserts the result is `Err(...)` with an error string containing "E-SPEC-001"

This is the SAP-3 compliance test for the Rule 1 base_url arm.

### T-09: Write AC-002 test (SAP-3 EC-009-005 via add_sensor_spec API)
**File:** `crates/prism-spec-engine/tests/` (integration test file)

Write a test that calls `add_sensor_spec(org_slug, toml_with_bad_base_url)` and asserts
an error containing E-SPEC-001 is returned. This verifies the full public-surface path:
MCP tool backing function → parse_and_validate_spec_toml → validate_sensor_spec → Rule 1.

### T-10: Write AC-003 test (multi-error collect-all)
**File:** `crates/prism-spec-engine/tests/`

Write a test that submits a TOML with TWO Rule-1–5 violations and asserts that the
`errors` vec in the returned `ValidationError` has length ≥ 2.

### T-11: Write AC-007 test (load_all multi-spec DI-030)
**File:** `crates/prism-spec-engine/tests/`

Write a test that creates a temporary directory with one valid spec and one spec with
`base_url = "ftp://x"`, calls `SpecLoader::load_all(path)`, and asserts:
- The valid spec is in the returned success collection
- The invalid spec's error appears in the returned error collection
- The function does NOT abort on the first bad spec

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~4,500 |
| ADR-055 (authority doc) | ~3,500 |
| `src/add_sensor_spec.rs` | ~2,000 |
| `src/spec_parser.rs` (load_all section) | ~2,500 |
| `src/validation.rs` (validate_sensor_spec) | ~2,000 |
| `src/hot_reload.rs` (process_spec_changes) | ~1,000 |
| `tests/bc_2_16_009_test.rs` (existing tests + fixture audit) | ~3,000 |
| `tests/bc_2_16_001_bundled_spec_load.rs` (AC-004 verification) | ~1,500 |
| Grep results from fixture audit (base_url values) | ~1,500 |
| Running test output (nextest) | ~2,000 |
| BC-2.16.009 (authority contract for Rules 1–5) | ~2,000 |
| **Total estimate** | **~25,500** |

25,500 tokens is above the 20–30% single-story threshold for a 100k-token context. The
story should be implementable in one dispatch because the fixture audit is the largest
single task and its output (grep results + updated fixtures) drives everything else. If
the fixture audit returns more than 20 fixture files requiring updates, split into:
- Dispatch A: T-01 through T-07 (wiring + fixtures) 
- Dispatch B: T-08 through T-11 (new tests)

---

## Previous Story Intelligence

No direct predecessor story in this epic covers `prism-spec-engine` validation wiring.

Lessons from PLUGIN-MIGRATION-001-D cascade (general):
- Read the actual function before patching it. ADR-055 §D1/D2 provide Rust snippets, but
  field names in `SpecError`, `ValidationError`, and `PrismError::Spec(...)` must be
  verified against the live source — they may have changed since the ADR was authored.
- The fixture audit (T-04) is the highest-risk task. In past cascades, newly-enabled
  validation caused 3–5 fixture breakages that had to be caught and fixed before `just check`
  passed. Budget the audit before writing any new tests.
- Do NOT run `just check` (full workspace) between each task. Use
  `cargo nextest run -p prism-spec-engine --no-fail-fast` for the inner loop.

---

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md` and applicable ADRs:

1. **ADR-055 §D3 — Wire into callers, not into SpecLoader::parse().** `parse()` runs before
   env-var resolution; inserting Rules 1–5 there would incorrectly reject bundled specs
   with env-var-backed base_url values. Do not move the call site into `parse()`.

2. **ADR-055 §D5 — SENSOR_ID_RE is defense-in-depth Layer 1.** It checks the same
   character constraint as `validate_sensor_id()` but is independently motivated by CWE-22
   (path traversal). Removing it requires a security review that is out of scope for this
   story. If the code review flags it as duplicate, respond that it is intentional
   defense-in-depth and cite ADR-055 §D5.

3. **BC-2.16.009 §Invariants — collect-all (VP-059).** All Rule 1–5 errors must be collected
   in a single pass. The `Vec<SpecError>` from `validate_sensor_spec()` is already
   collect-all. The `parse_and_validate_spec_toml()` wiring must forward ALL errors into the
   `ValidationError.errors` vec — not just the first one.

4. **CLAUDE.md §Structured event catalog discipline (SAP-1).** The `tracing::warn!` with
   `event_type = "spec.validation_warning"` requires a BC-2.16.002 catalog row in the same
   PR. The implementer adds the emission; the PO adds the catalog row. The PR cannot merge
   until both are present.

5. **CLAUDE.md §No println! in production code.** Use `tracing::warn!` with structured
   fields for the warning surfacing (§D6). No `println!` or `eprintln!`.

---

## Library & Framework Requirements

| Library | Version | Source of truth |
|---------|---------|----------------|
| `tracing` | pinned in workspace `Cargo.toml` | `architecture/dependency-graph.md §External Dependencies` |

No new external dependencies are introduced by this story.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-spec-engine/src/add_sensor_spec.rs` | MODIFY | T-05: insert validate_sensor_spec() call + §D6 warning; preserve SENSOR_ID_RE (AC-005) |
| `crates/prism-spec-engine/src/spec_parser.rs` | MODIFY | T-06: insert validate_sensor_spec() call in load_all() loop |
| `crates/prism-spec-engine/src/hot_reload.rs` | MODIFY | T-07: add comment only — no behavioral change |
| `crates/prism-spec-engine/tests/bc_2_16_009_test.rs` | MODIFY | T-04 fixture audit comment + T-08 AC-001 test |
| `crates/prism-spec-engine/tests/` | ADD tests | T-09 (AC-002), T-10 (AC-003), T-11 (AC-007) |

---

## Verification Properties

| VP | Description | Applicability |
|----|-------------|---------------|
| VP-059 | validate_sensor_spec() collect-all invariant — all Rule 1–5 errors in single pass | AC-003 verifies this via parse_and_validate_spec_toml() integration surface |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.1 | 2026-07-26 | story-writer | FB60 MED-008 + MED-009: pin BC versions from `current` to actuals in §Behavioral Contracts table (BC-2.16.009→v1.28, BC-2.16.001→v1.9, BC-2.16.007→v1.7, BC-2.16.008→v1.6, BC-2.16.002→v2.11); add BC-2.16.002 to frontmatter `behavioral_contracts:` array (POL-8 bidirectional frontmatter↔body reconciliation) |
| 1.0 | 2026-07-25 | story-writer | Initial authoring from ADR-055 §Story Scope; SAP-1/SAP-3 compliance; fixture audit AC; PO dependency encoding |
