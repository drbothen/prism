---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-08T00:00:00Z
phase: 3
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
lifecycle: draft
anchored_stories: [S-WAVE5-PREP-01]
verifying_vps: []
crates: [prism-bin, prism-spec-engine]
inputs:
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
  - .factory/specs/architecture/module-decomposition.md
input-hash: "[md5]"
traces_to: ["CAP-009"]
---

# BC-2.06.011: ConfigManager Initialization — prism.toml Schema Validation at Process Start

## Description

This BC is the Client Configuration subsystem's (SS-06) startup-time contract. It specifies
how `prism-bin` loads, deserializes, and validates `prism.toml` at the beginning of `prism start`
boot step 2 (per ADR-022 §B). The orchestration of this and the other 3 subsystem init contracts
in §B order is specified separately in BC-2.22.001.

`prism-bin` reads `prism.toml` from the config directory (`$PRISM_CONFIG_DIR` or default
`~/.prism/`), deserializes it via serde/toml, and validates it against the config-schema.md
contract. This step is BLOCKING: no subsequent boot step begins until config load and schema
validation complete successfully. On any failure (file not found, TOML syntax error, schema
violation, type mismatch), the process exits with code 2 before step 3 (OrgRegistry init) begins.

No `todo!()`, `unimplemented!()`, or `panic!("stub...")` may appear in the production code
path for this step at or after story S-WAVE5-PREP-01 merges (POL-12 enforcement).

## Preconditions

- The Prism binary has been invoked with the `start` subcommand (ADR-022 §A)
- Boot step 1 (tracing init) has completed successfully — the tracing subscriber is active
- The filesystem is readable by the process user
- `$PRISM_CONFIG_DIR` is set, or the default config directory (`~/.prism/`) is accessible

## Postconditions

**Happy path:**
- `prism.toml` is read from `$PRISM_CONFIG_DIR/prism.toml` (or default path)
- The file is fully deserialized into the config struct without error
- Schema validation passes (all required fields present, types match, values within bounds)
- A valid `PrismConfig` handle is available to all subsequent boot steps
- Boot continues to step 3 (OrgRegistry init) per ADR-022 §B ordering

**Failure path — file missing:**
- The process emits a `tracing::error!` log naming the expected path
- The process exits with code **2** (config-invalid) per ADR-022 §A exit-code contract
- Step 3 never begins

**Failure path — TOML syntax error:**
- The process emits a `tracing::error!` log with the line number and context of the parse error
- The process exits with code **2**
- Step 3 never begins

**Failure path — schema validation failure:**
- The process emits a `tracing::error!` log naming every invalid field (one-pass, not fail-fast)
- The process exits with code **2**
- Step 3 never begins

## Invariants

- Boot step 2 is blocking: no concurrent execution with step 3 or later (ADR-022 §B "Traffic gate")
- Exit code on any config failure is exactly 2, never 1, 4, or 5 (ADR-022 §A canonical table)
- The `$PRISM_CONFIG_DIR` env var, if set, always overrides the default path; the binary MUST
  NOT fall back to the default if `PRISM_CONFIG_DIR` is set to a non-existent directory — it
  must exit 2 with "Config directory not found: {path}"
- Validation is one-pass: all schema errors are collected and reported together, not stopped
  at the first error (usability invariant — per CAP-009 "Validation is multi-error")

## Error Cases

| Error Code | Condition | Behavior |
|------------|-----------|----------|
| Exit 2 | `prism.toml` file not found at expected path | Log path; exit 2 |
| Exit 2 | TOML parse error (syntax) | Log line + context; exit 2 |
| Exit 2 | Schema validation failure (missing required field, wrong type) | Log all invalid fields; exit 2 |
| Exit 2 | `$PRISM_CONFIG_DIR` set but directory does not exist | "Config directory not found: {path}"; exit 2 |
| Exit 2 | `prism.toml` exists but is not readable (permissions) | Log permission error; exit 2 |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-011-001 | `$PRISM_CONFIG_DIR` set to non-existent path | Exit 2 with "Config directory not found: {path}" — do NOT fall back to default |
| EC-06-011-002 | `prism.toml` has valid TOML syntax but unknown top-level key | Validation fails on unknown key (strict schema); exit 2 |
| EC-06-011-003 | `prism.toml` exists but has 0 bytes (empty file) | Parse fails; "Config file is empty or missing required sections"; exit 2 |
| EC-06-011-004 | `prism.toml` is a directory, not a file | OS returns EISDIR or equivalent; exit 2 |
| EC-06-011-005 | Multiple schema errors in one file | All errors reported in a single structured log entry; exit 2 once |
| EC-06-011-006 | `validate-config` subcommand (not `start`) uses this same step | Same boot step 2 logic applies; `validate-config` exits 0 on success, 2 on failure |

## Canonical Test Vectors

| ID | Scenario | Config Input | Expected Exit Code | Expected Log Output |
|----|----------|-------------|-------------------|---------------------|
| TV-06-011-001 | Valid config file at default path | Valid `~/.prism/prism.toml` | Boot continues (no exit) | `tracing::info!("Config loaded: {n} orgs, spec_dir={path}")` |
| TV-06-011-002 | Valid config via `$PRISM_CONFIG_DIR` override | Valid file at env var path | Boot continues (no exit) | Info log with env path |
| TV-06-011-003 | Config dir missing | `$PRISM_CONFIG_DIR=/nonexistent` | 2 | "Config directory not found: /nonexistent" |
| TV-06-011-004 | TOML syntax error | `prism.toml` with `org_id = {broken` | 2 | Line number + parse context |
| TV-06-011-005 | Missing required field | `prism.toml` without `spec_dir` | 2 | "Missing required field: spec_dir" |
| TV-06-011-006 | Empty file | `prism.toml` = 0 bytes | 2 | "Config file is empty or missing required sections" |

## Test Strategy

Integration tests in `crates/prism-bin/tests/boot_tests.rs` invoke `prism` as a subprocess
via `std::process::Command`. Tests for this BC:

- `test_BC_2_06_011_valid_config` — assert boot step 2 completes (subprocess does not exit before step 3 log line appears)
- `test_BC_2_06_011_missing_dir` — assert exit code 2 + stderr contains "not found"
- `test_BC_2_06_011_toml_syntax_error` — assert exit code 2 + stderr contains line number
- `test_BC_2_06_011_missing_required_field` — assert exit code 2 + stderr contains field name

All integration tests use fixture configs under `crates/prism-bin/fixtures/config/`.
Do NOT use unit tests with mocked filesystems for this BC — the exit-code contract requires
subprocess-level assertions.

## Verification Properties

No formal VP is proposed at this time. The boot step 2 logic is effectful I/O (filesystem
read + TOML parse), which limits Kani/proptest applicability. The canonical test vectors
above cover the contract via integration tests.

## Related BCs

- BC-2.22.001 — Boot Orchestration (orchestrates: this BC is one of 4 subsystem init contracts
  whose ordering and exit-code mapping are specified in BC-2.22.001)
- BC-2.21.001 — OrgRegistry init (depends on: step 3 begins only after this BC's happy path)
- BC-2.03.013 — Credential store init (depends on: step 5 begins only after this BC + BC-2.21.001)
- BC-2.05.012 — Audit subsystem init (depends on: step 6 begins only after all preceding steps)
- BC-2.06.001 — TOML Configuration Loads and Deserializes at Startup (composes with: BC-2.06.001
  specifies the schema contract; this BC specifies the boot-sequence ordering and exit-code contract)
- BC-2.06.005 — Configuration Validation Reports All Errors in One Pass (enforced by: this BC
  cites the one-pass invariant from CAP-009)

## Architecture Anchors

- `specs/architecture/decisions/ADR-022-production-runtime-wiring.md` §B step 2 (canonical boot step spec)
- `specs/architecture/decisions/ADR-022-production-runtime-wiring.md` §A exit-code contract (exit code 2 source)
- `specs/architecture/module-decomposition.md` COMP-001 `prism-bin` (SS-22), COMP-014 `prism-spec-engine` (SS-06)
- `specs/architecture/config-schema.md` (schema contract for validation)

## Story Anchor

S-WAVE5-PREP-01 — prism-bin: Binary Chassis, CLI, and Boot Sequence

## VP Anchors

None (see Verification Properties)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| Capability Anchor Justification | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 — this BC specifies the startup-time load and validation of `prism.toml`, which is exactly the "Load and validate per-client sensor mappings, credential references, and capability overrides from TOML configuration" behavior that CAP-009 defines. The multi-error validation invariant in CAP-009 ("Validation is multi-error: all problems reported in one pass") is directly reflected in this BC's one-pass invariant. |
| L2 Invariants | No DI directly covers boot ordering. DI-030 (sensor spec validation at load time) is enforced downstream in step 4; this BC governs the earlier prism.toml load. |
| ADR Source | ADR-022 §B step 2, §A exit-code table |
| Priority | P0 |
| POL-12 Note | The production code path satisfying this BC MUST contain no `todo!()`, `unimplemented!()`, or `panic!("stub...")` before S-WAVE5-PREP-01 transitions to `merged`. |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | bundle-B-phase-B-1b-ss22-bcs-2026-05-08 | 2026-05-08 | product-owner | Initial authorship — Bundle B Phase B-1b SS-22 boot-sequence BCs |
| 1.0 | redirect-option-d-2026-05-08 | 2026-05-08 | product-owner | Relocated from BC-2.22.001 (SS-22) to BC-2.06.011 (SS-06 Client Configuration) per Option (d) decomposition. Capability anchor updated CAP-034 → CAP-009. EC/TV IDs renumbered to EC-06-011-NNN / TV-06-011-NNN. |
