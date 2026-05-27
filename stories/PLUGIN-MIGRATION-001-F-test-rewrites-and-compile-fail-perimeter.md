---
document_type: story
story_id: PLUGIN-MIGRATION-001-F
title: "tests: Rewrite 12 Sensor-Named Test Files to TOML Fixture Loading + Compile-Fail Perimeter `no-hardcoded-sensors`"
wave: 2
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: draft
version: "v1.2"
level: "L4"
producer: story-writer
timestamp: "2026-05-27T00:00:00Z"
modified: "2026-05-27"
tdd_mode: strict
subsystems: [SS-01, SS-16, SS-17]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters, prism-sensors) — the 10 non-DTU-harness test files being
#   rewritten live in prism-dtu-*, prism-query, and prism-spec-engine; all exercise
#   the sensor adapter layer (SS-01) by name. The ADR-023 Rule 3 constraint
#   ("no sensor-named references in non-DTU code") is a SS-01 + SS-16 architectural
#   invariant enforced by the new compile-fail perimeter.
#   SS-16 (Spec Engine, prism-spec-engine) — four parity test files at
#   prism-spec-engine/tests/parity/ are being rewritten to use TOML fixture loading
#   instead of sensor-named PipelineExecutor construction. The spec-catalog lookup
#   path is SS-16 territory.
#   SS-17 (WASM Plugin Runtime) — crowdstrike_oauth2_plugin_tests.rs references plugin
#   dispatch by sensor name; the rewrite uses TOML-driven fixture loading consistent
#   with SS-17 plugin dispatch discipline.
crates_touched:
  - prism-spec-engine   # 6 test files: parity/{crowdstrike,claroty,cyberint,armis}.rs +
                        # bc_2_16_002_crowdstrike_two_step.rs + crowdstrike_oauth2_plugin_tests.rs
  - prism-query         # crowdstrike_session_isolation.rs
  - prism-dtu-armis     # bc_3_4_armis_generator.rs
  - prism-dtu-claroty   # bc_3_4_claroty_generator.rs
  - prism-dtu-crowdstrike # bc_3_4_crowdstrike_generator.rs
  - prism-dtu-cyberint  # bc_3_4_cyberint_generator.rs
  - prism-dtu-demo-server # ac_2_crowdstrike_fixture.rs
target_module: tests/external/no-hardcoded-sensors
capabilities: [CAP-029]
behavioral_contracts:
  - BC-2.01.013  # DataSource Trait — the compile-fail perimeter enforces that no non-test
                 # production code holds hardcoded sensor name references; the open-trait
                 # spec-driven path is the only valid adapter construction mechanism
  - BC-2.16.009  # Spec File Validation — Schema Validation, Variable Reference Resolution,
                 # OCSF Field Validation — rewritten test files must load sensor specs from
                 # TOML fixtures, not hardcode sensor types; this BC covers the spec-load
                 # validation contract exercised by the rewrites
  - BC-2.16.012  # PluginRegistry Dispatch — no sensor-named match arms in non-DTU
                 # production code; the compile-fail perimeter guards this invariant
# BC status: The above 3 BCs are active per BC-INDEX.md v5.53. No BC-TBD placeholders.
verification_properties:
  - VP-146  # VP-PLUGIN-001 alias — No production hardcoded sensor references (FORBIDDEN-
            # SYMBOLS-001 compile-fail perimeter): The no-hardcoded-sensors/ crate extends
            # this coverage to test files and adds the sensor-name string literal gate.
            # Note: VP-146 anchor story is PLUGIN-MIGRATION-001-A; this story strengthens
            # the perimeter to cover test-file discipline. A new VP-157 for the
            # no-hardcoded-sensors compile-fail crate itself is deferred to PO authorship
            # (no VP-TBD allowed in ready stories per Spec-First Gate S-7.01).
depends_on:
  - PLUGIN-MIGRATION-001-A  # auth modules deleted; sensor-named imports in test files must
                            # be removed before this story rewrites them (no-compile guard)
  - PLUGIN-MIGRATION-001-B  # prism-query dispatch sites converted to spec-catalog; the
                            # crowdstrike_session_isolation.rs rewrite depends on the
                            # new spec-catalog query path being present
  - S-CONFIG-MULTI-TENANT-OVERRIDE-001  # per-org overlay loading must be merged before
                                         # TOML fixture loading tests can exercise the
                                         # resolved spec path correctly (ADR-029 overlay)
blocks: []
# Note: PLUGIN-MIGRATION-001-G (doc/ADR/BC sweep) has no code dependency on 001-F.
# The doc sweep can proceed in parallel once 001-A/B/C are merged. No blocks entry needed.
points: 8
# Points justification:
#   - 4 parity test files (prism-spec-engine/tests/parity/): ~1 pt each = 4 pts
#     (each requires: read TOML fixture from .prism/specs/sensors/, construct
#     PipelineExecutor via spec-catalog, compare output against DTU fixture)
#   - bc_2_16_002_crowdstrike_two_step.rs rewrite: ~1 pt (two-step fetch test using
#     TOML spec + DTU harness instead of hardcoded CrowdStrike adapter construction)
#   - crowdstrike_oauth2_plugin_tests.rs rewrite: ~1 pt (plugin dispatch test using
#     .prx + TOML spec fixture instead of named OAuth2 module)
#   - crowdstrike_session_isolation.rs (prism-query): ~0.5 pt (session keying test using
#     spec-catalog SensorId instead of SensorType::CrowdStrike enum)
#   - 4 DTU generator tests (bc_3_4_*_generator.rs): ~0.5 pt each = 2 pts
#     (rename test functions; these ARE legitimately sensor-specific since they ARE
#     the DTU clone; the rewrite is doc/comment cleanup + sensor-name reference audit
#     rather than full structural change — see §DTU Harness Scope Decision)
#   - compile-fail perimeter crate (tests/external/no-hardcoded-sensors/): ~1.5 pts
#     (new crate; Cargo.toml; src/main.rs attempting sensor-name string imports;
#     CI job update in ci.yml; justfile recipe)
#   Total: 8 points (ADR-023 Wave 2 estimate: 5–8 SP). At upper bound due to
#   compile-fail crate being net-new infrastructure.
estimated_days: 3
risk: MEDIUM
# Risk justification: The parity test rewrites require functional TOML specs
# (from PLUGIN-MIGRATION-001-D) and the DTU harness (S-3.3.03+). If any
# sensor TOML spec is missing a field that was exercised by the old hardcoded
# test, the rewrite will expose a gap in the TOML spec rather than in this story.
# The DTU harness clone modules (armis/claroty/crowdstrike/cyberint) in
# prism-dtu-harness/src/clones/ are NOT in scope for this story per the scope
# decision in §DTU Harness Scope Decision — they ARE legitimately sensor-named.
acceptance_criteria_count: 8
red_gate_tests: 7
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "TOML spec completeness: if a sensor TOML spec lacks a field that a test asserts,
    the gap must be reported as a TOML spec defect (route to product-owner/ADR-028),
    not papered over with a test skip. Any such gap is a P1 finding per production-grade
    default (CLAUDE.md Canonical Principle Rule 1)."
  - "DTU harness clones: prism-dtu-harness/src/clones/{armis,claroty,crowdstrike,cyberint}.rs
    are explicitly OUT OF SCOPE for sensor-name removal — they ARE the DTU clones.
    The no-hardcoded-sensors compile-fail crate MUST NOT include prism-dtu-harness as
    a dependency, or it will false-positive on legitimate DTU clone code."
  - "Compile-fail crate isolation: the no-hardcoded-sensors crate must follow the
    perimeter-violation crate pattern (separate [workspace] stanza, excluded from
    workspace default-members, CI job asserts non-zero exit). Any accidental
    successful compilation means a sensor-named symbol is unintentionally public."
inputs:
  - "crates/prism-spec-engine/tests/parity/crowdstrike.rs"
  - "crates/prism-spec-engine/tests/parity/claroty.rs"
  - "crates/prism-spec-engine/tests/parity/cyberint.rs"
  - "crates/prism-spec-engine/tests/parity/armis.rs"
  - "crates/prism-spec-engine/tests/bc_2_16_002_crowdstrike_two_step.rs"
  - "crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs"
  - "crates/prism-query/tests/crowdstrike_session_isolation.rs"
  - "crates/prism-dtu-armis/tests/bc_3_4_armis_generator.rs"
  - "crates/prism-dtu-claroty/tests/bc_3_4_claroty_generator.rs"
  - "crates/prism-dtu-crowdstrike/tests/bc_3_4_crowdstrike_generator.rs"
  - "crates/prism-dtu-cyberint/tests/bc_3_4_cyberint_generator.rs"
  - "crates/prism-dtu-demo-server/tests/ac_2_crowdstrike_fixture.rs"
  - "crates/prism-dtu-harness/src/clones/armis.rs"
  - "crates/prism-dtu-harness/src/clones/claroty.rs"
  - "crates/prism-dtu-harness/src/clones/crowdstrike.rs"
  - "crates/prism-dtu-harness/src/clones/cyberint.rs"
  - "tests/external/perimeter-violation/Cargo.toml"
  - "tests/external/perimeter-violation/src/main.rs"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.012-plugin-registry-dispatch-migration.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-A-delete-4-named-auth-modules-and-replace-init-registry-for-org.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-B-convert-5-sensor-name-dispatch-sites-to-spec-catalog-lookup.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
---

# PLUGIN-MIGRATION-001-F: tests — Rewrite 12 Sensor-Named Test Files + Compile-Fail Perimeter `no-hardcoded-sensors`

**Story ID:** PLUGIN-MIGRATION-001-F
**Status:** draft
**Version:** v1.2
**Wave:** 2 (cleanup wave; ordered after PLUGIN-MIGRATION-001-A + 001-B both merged)

---

## §Origin

Registered in STORY-INDEX at D-334 (2026-05-10) as Wave 2 of the PLUGIN-MIGRATION saga.
ADR-023 §Migration Plan Wave 2 scope: "Rewrite approximately 10 sensor-named test files
to TOML fixture loading; compile-fail perimeter test at `tests/external/no-hardcoded-sensors/`
(5–8 SP)."

Wave 1 (7/7 stories merged) eliminated all sensor-named references from production code.
Wave 2/F eliminates them from test files and adds a compile-fail enforcement gate so the
pattern cannot regress.

---

## §DTU Harness Scope Decision

The four files at `crates/prism-dtu-harness/src/clones/{armis,claroty,crowdstrike,cyberint}.rs`
are **legitimately sensor-named** — they ARE the DTU clones and must reference sensor
names to provide behavioral clones of those sensors. They are **NOT** in scope for
sensor-name removal.

What IS in scope for these files: a structural audit to verify that:
1. The files do not import from the deleted `prism-sensors::auth::{armis,claroty,...}` modules.
2. Any `use prism_sensors::*` imports are updated to the new open-trait path post-001-A.
3. No `SensorType::CrowdStrike` (closed-enum) references remain (these should already be
   gone after S-PLUGIN-PREREQ-A).

If all three checks pass, the DTU harness clones require no structural changes — only a
confirmation comment added to each file: "// Legitimately sensor-named: this IS the DTU
clone for {sensor}. Exempt from no-hardcoded-sensors perimeter per ADR-023 §DTU-EXEMPT."

The `no-hardcoded-sensors` compile-fail crate MUST NOT declare `prism-dtu-harness` as
a dependency.

---

## Story-Level Goal

At merge:

1. All 8 non-DTU-harness sensor-named test files are rewritten to use TOML fixture loading
   via `SpecLoader::parse()` and the prism-dtu-harness `DtuHarness` API — no hardcoded
   `SensorType::CrowdStrike` or `CrowdStrikeAdapter::new()` style construction.
2. The 4 DTU generator test files (bc_3_4_*_generator.rs) are audited and receive
   exemption comments if clean; any stale sensor-named imports are updated.
3. A new compile-fail crate at `tests/external/no-hardcoded-sensors/` attempts to import
   sensor-named symbols from prism-sensors (post-001-A deletions) and MUST NOT compile.
   CI job `no-hardcoded-sensors-compile-fail` asserts non-zero exit.
4. The CI configuration (`ci.yml`) is updated to run the new compile-fail check.

---

## Narrative

As the Prism platform, I want all sensor-named test files rewritten to load sensors via
TOML specs and the spec-catalog API, and a compile-fail perimeter added at
`tests/external/no-hardcoded-sensors/` to enforce the no-hardcoded-sensor invariant at
CI time, so that the plugin-only sensor architecture (ADR-023) cannot regress in test
code and future contributors receive a clear "compile error" signal when they accidentally
reach for sensor names.

---

## Behavioral Contracts

| BC ID | Version | Title | Subsystem | Role in This Story |
|-------|---------|-------|-----------|-------------------|
| BC-2.01.013 | 1.7 | DataSource Trait Eliminates Per-Sensor Code Duplication | SS-01 | **Primary** — compile-fail perimeter enforces that no non-DTU code imports sensor-named adapter types; the open-trait spec-driven path is the only permitted construction |
| BC-2.16.009 | 1.5 | Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation | SS-16 | **Primary** — rewritten parity tests exercise spec loading via `SpecLoader::parse()` on the 4 bundled TOML specs; spec-load validation is the gate for each rewritten test |
| BC-2.16.012 | 1.33 | PluginRegistry Dispatch in spec_parser.rs — Hardcoded Sensor Names Replaced with Registry Lookup | SS-16 | **Anti-regression** — no sensor-named match arms must remain in non-DTU code after this story; the compile-fail perimeter guards this invariant |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~8,000 |
| BC-2.01.013 + BC-2.16.009 + BC-2.16.012 (3 BCs) | ~9,000 |
| ADR-023 §Migration Plan Wave 2 + §Constraints | ~5,000 |
| 8 test files to rewrite (avg ~100 lines each) | ~10,000 |
| 4 DTU generator test files (avg ~150 lines each) | ~8,000 |
| 4 DTU harness clone files (audit only, ~100 lines each) | ~5,000 |
| tests/external/perimeter-violation/src/main.rs (template) | ~4,000 |
| prism-spec-engine/src/spec_parser.rs (SpecLoader API) | ~4,000 |
| prism-dtu-harness DtuHarness API surface | ~3,000 |
| PLUGIN-MIGRATION-001-A (predecessor story body) | ~3,000 |
| **Total estimate** | **~59,000** |
| Agent context window (claude-sonnet-4-6) | ~200,000 |
| **% of context window** | **~29.5%** |

Near the 30% ceiling. Implementer should read BC files and ADR-023 selectively (Wave 2
section only) and use `just iter prism-spec-engine` for inner-loop validation rather
than full workspace checks.

---

## Acceptance Criteria

### AC-001: 4 parity test files rewritten — TOML fixture loading, no sensor name in test body (traces to BC-2.01.013 postcondition — spec-driven adapter construction; BC-2.16.009 postcondition — specs load without sensor-named imports)

The four parity test files at `crates/prism-spec-engine/tests/parity/` are rewritten:

| File | Old Pattern | New Pattern | Red Gate Test |
|------|-------------|-------------|---------------|
| `parity/crowdstrike.rs` | `CrowdStrikeAdapter::new(...)` or named sensor type | `SpecLoader::parse(&spec_dir)` → spec-catalog lookup by `"crowdstrike"` SensorId | `test_PLUGIN_MIGRATION_001_F_parity_crowdstrike_toml_fixture_loading` |
| `parity/claroty.rs` | Same pattern for Claroty | `SpecLoader::parse()` → `"claroty"` | `test_PLUGIN_MIGRATION_001_F_parity_claroty_toml_fixture_loading` |
| `parity/cyberint.rs` | Same pattern for Cyberint | `SpecLoader::parse()` → `"cyberint"` | `test_PLUGIN_MIGRATION_001_F_parity_cyberint_toml_fixture_loading` |
| `parity/armis.rs` | Same pattern for Armis | `SpecLoader::parse()` → `"armis"` | `test_PLUGIN_MIGRATION_001_F_parity_armis_toml_fixture_loading` |

Each rewritten test:
- Loads the production TOML spec from `.prism/specs/sensors/{sensor}.sensor.toml` via `SpecLoader::parse()` (required for test-time `base_url` override to point at the DTU clone)
- Constructs a `PipelineExecutor` via the spec-catalog without naming any sensor adapter type directly
- Runs against the prism-dtu-harness clone via `DtuHarness` in logical-isolation mode
- Asserts OCSF output matches expected fixture (TS-PLUGIN-PARITY-001 canonicalization)

(traces to BC-2.01.013 postcondition 3 — DataSource trait enables spec-driven lookup without sensor-named import; traces to BC-2.16.009 postcondition 1 — bundled specs are loadable via SpecLoader::parse, which accepts a base_url override for test-time DTU targeting)

### AC-002: `bc_2_16_002_crowdstrike_two_step.rs` rewritten — TOML + DTU harness (traces to BC-2.16.009 postcondition — spec-load validates two-step fetch configuration)

`crates/prism-spec-engine/tests/bc_2_16_002_crowdstrike_two_step.rs` is rewritten:
- Removes any direct `CrowdStrikeAdapter` or `CrowdStrikeAuth` import
- Loads the CrowdStrike TOML spec from the bundled spec directory
- Uses `DtuHarness` logical-isolation clone instead of a custom wiremock fixture
- Test function renamed to `test_PLUGIN_MIGRATION_001_F_bc_2_16_002_crowdstrike_two_step_toml_driven`
- Asserts: two-step fetch pattern (QueryV2 + PostEntities) fires against the DTU clone as configured by TOML `[fetch_step]` + `[fetch_step.enrich]` blocks

(traces to BC-2.16.009 postcondition — spec_parser validates two-step configuration blocks at load time; BC-2.01.013 postcondition — no sensor-named adapter type in test)

### AC-003: `crowdstrike_oauth2_plugin_tests.rs` rewritten — plugin dispatch via TOML (traces to BC-2.16.012 postcondition — no sensor-named match arms; plugin dispatch via spec-catalog)

`crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs` is rewritten:
- Removes any direct sensor-named imports for OAuth2 adapter construction
- Test uses TOML spec with `[auth] type = "oauth2_client_credentials"` and the `.prx` WASM plugin path from `S-PLUGIN-CI-001`
- Test is `#[ignore = "requires prism-dtu-crowdstrike DTU clone + built .prx artifact"]` if the DTU clone is not available in this crate's test context
- If kept live: uses `DtuHarness` + `plugin_runtime` wired from the boot path
- Test function: `test_PLUGIN_MIGRATION_001_F_crowdstrike_oauth2_plugin_dispatch_via_toml`

(traces to BC-2.16.012 postcondition 2 — PluginRegistry dispatch uses SensorId string key, not enum match arm)

### AC-004: `crowdstrike_session_isolation.rs` (prism-query) rewritten — SensorId string key (traces to BC-2.01.013 postcondition — sensor identified by SensorId Arc<str> not SensorType enum)

`crates/prism-query/tests/crowdstrike_session_isolation.rs` is rewritten:
- Any `SensorType::CrowdStrike` enum reference replaced with `SensorId::from("crowdstrike")` (or equivalent `Arc<str>`)
- Test asserts OrgId-keyed session isolation for pagination cursor scoping
- No import of deleted auth modules from prism-sensors
- Test function: `test_PLUGIN_MIGRATION_001_F_crowdstrike_session_isolation_sensor_id_key`

(traces to BC-2.01.013 postcondition — SensorId(Arc<str>) is the canonical sensor identifier; no closed-enum variant usage permitted in non-DTU code)

### AC-005: DTU generator tests audited — exemption comments added or stale imports fixed (traces to BC-2.01.013 invariant — no sensor-named closed-enum imports in any non-DTU-clone code)

The 4 DTU generator test files are audited:

| File | Exemption or Fix |
|------|-----------------|
| `crates/prism-dtu-armis/tests/bc_3_4_armis_generator.rs` | Add comment `// Legitimately sensor-named: this IS the Armis DTU generator test. Exempt from no-hardcoded-sensors perimeter per ADR-023 §DTU-EXEMPT.` if no stale imports from deleted prism-sensors::auth modules |
| `crates/prism-dtu-claroty/tests/bc_3_4_claroty_generator.rs` | Same for Claroty |
| `crates/prism-dtu-crowdstrike/tests/bc_3_4_crowdstrike_generator.rs` | Same for CrowdStrike |
| `crates/prism-dtu-cyberint/tests/bc_3_4_cyberint_generator.rs` | Same for Cyberint |

If any file imports from a module deleted in PLUGIN-MIGRATION-001-A (e.g., `prism_sensors::auth::crowdstrike::CrowdStrikeAuth`), those imports MUST be removed and replaced with the appropriate post-001-A equivalent.

`crates/prism-dtu-demo-server/tests/ac_2_crowdstrike_fixture.rs` is similarly audited:
- If it directly constructs a `CrowdStrikeAdapter` (not via spec-catalog), replace with TOML-driven construction
- Add exemption comment if it correctly uses `DtuHarness` or TOML fixture

(traces to BC-2.01.013 invariant — the only legitimate hardcoded sensor references in the codebase are in the DTU clone crates themselves, not in test files that exercise them via the adapter layer)

### AC-006: `tests/external/no-hardcoded-sensors/` compile-fail crate created (traces to BC-2.16.012 postcondition — no sensor-named match arms accessible from outside prism-sensors; BC-2.01.013 postcondition — DataSource trait is the only sensor interface)

A new compile-fail crate is created at `tests/external/no-hardcoded-sensors/` following
the exact pattern of `tests/external/perimeter-violation/`:

**`Cargo.toml`:**
```toml
[package]
name = "no-hardcoded-sensors"
version = "0.1.0"
edition = "2021"
publish = false

# ADR-023 Rule 3 — No hardcoded sensor name enforcement.
#
# This crate deliberately attempts to reference sensor-named symbols that
# were DELETED in PLUGIN-MIGRATION-001-A. It MUST NOT compile.
# CI job `no-hardcoded-sensors-compile-fail` asserts non-zero exit.
#
# If this crate ever compiles, a sensor-named symbol has been accidentally
# re-exported, which is an ADR-023 Rule 3 violation.

[dependencies]
prism-sensors = { path = "../../../crates/prism-sensors" }

[workspace]
```

**`src/main.rs`** attempts to import the 4 deleted auth modules:
```rust
// ADR-023 Rule 3 compile-fail test.
// These symbols were DELETED in PLUGIN-MIGRATION-001-A.
// This file MUST NOT compile successfully.
use prism_sensors::auth::armis::ArmisAuth;      // DELETED in 001-A
use prism_sensors::auth::claroty::ClarotyAuth;  // DELETED in 001-A
use prism_sensors::auth::crowdstrike::CrowdStrikeAuth; // DELETED in 001-A
use prism_sensors::auth::cyberint::CyberintAuth; // DELETED in 001-A
fn main() {}
```

The CI job `no-hardcoded-sensors-compile-fail` runs:
```bash
cargo check \
  --color=never \
  --manifest-path tests/external/no-hardcoded-sensors/Cargo.toml \
  > /tmp/no-hardcoded-check.log 2>&1 \
  && CARGO_RC=0 || CARGO_RC=$?
# Assert non-zero exit from `cargo check` (expected: E0432 unresolved import)
# --manifest-path is required: the crate has a separate [workspace] stanza and
# is excluded from the root workspace members; -p flag would not resolve it.
# --color=never is required: modern cargo (1.85+) emits ANSI codes without it,
# breaking downstream grep/pattern matching on error codes.
```

Red Gate test name: `test_PLUGIN_MIGRATION_001_F_006_no_hardcoded_sensors_compile_fail_gate` (this is a CI-job test, verified by asserting `cargo check --manifest-path tests/external/no-hardcoded-sensors/Cargo.toml` returns a non-zero exit code in the CI script, analogous to the perimeter-violation job).

(traces to BC-2.01.013 postcondition — deleted adapter modules are not accessible; BC-2.16.012 postcondition — sensor-named match arms are not constructible from sensor module imports)

### AC-007: `ci.yml` updated — `no-hardcoded-sensors-compile-fail` job added (traces to BC-2.01.013 invariant — compile-time enforcement prevents regression; ADR-023 Rule 3 structural constraint)

The CI configuration at `.github/workflows/ci.yml` (or equivalent) gains a new job:

```yaml
no-hardcoded-sensors-compile-fail:
  name: "ADR-023 No-Hardcoded-Sensors compile-fail gate"
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Assert no-hardcoded-sensors does NOT compile
      run: |
        set -uo pipefail
        # --color=never is REQUIRED: without it, modern cargo (1.85+) emits ANSI
        # color codes even when stdout is redirected, breaking downstream grep on
        # error codes (mirrors perimeter-compile-fail job rationale).
        # --manifest-path is REQUIRED: this crate has a separate [workspace] stanza
        # and is excluded from root workspace members; -p flag would not resolve it.
        cargo check \
          --color=never \
          --manifest-path tests/external/no-hardcoded-sensors/Cargo.toml \
          > /tmp/no-hardcoded-check.log 2>&1 \
          && CARGO_RC=0 || CARGO_RC=$?
        cat /tmp/no-hardcoded-check.log
        if [ "${CARGO_RC}" -eq 0 ]; then
          echo "::error::no-hardcoded-sensors compiled successfully — sensor-named auth modules are accessible (ADR-023 Rule 3 regression)"
          exit 1
        fi
        # Per-symbol positive-coverage: verify all 4 deleted auth symbols appear
        # in E0432 errors. A single-symbol regression (one symbol re-exported while
        # siblings remain deleted) produces non-zero cargo exit but is MISSING from
        # the error list — this assertion catches that case.
        for SYM in ArmisAuth ClarotyAuth CrowdStrikeAuth CyberintAuth; do
          if ! grep -q "error\[E0432\].*${SYM}\|unresolved import.*${SYM}" /tmp/no-hardcoded-check.log; then
            echo "::error::Expected E0432 for deleted symbol ${SYM} but it was not in cargo output. Symbol may have been re-exported."
            exit 1
          fi
        done
        echo "PASS: no-hardcoded-sensors correctly fails to compile; all 4 deleted auth symbols produce E0432 errors"
```

This job mirrors the existing `perimeter-compile-fail` job structure, including
`--color=never`, `--manifest-path`, log capture, and per-symbol positive-coverage
assertions — the same pattern used by the non-exhaustive-violation-compile-fail job.

(traces to BC-2.01.013 invariant 3 — sensor identification is by SensorId string, not by compiled-in enum or module; the CI gate enforces this at every PR)

### AC-008: DTU harness clone files audited — no stale sensor-named imports from deleted modules (traces to BC-2.01.013 invariant — DTU clones exempt but must not import deleted symbols)

The 4 files at `crates/prism-dtu-harness/src/clones/` are audited (NOT structurally rewritten):

| File | Action |
|------|--------|
| `crowdstrike.rs` | Remove any `use prism_sensors::auth::crowdstrike::*` imports (module deleted in 001-A); add exemption comment |
| `claroty.rs` | Same for Claroty auth module |
| `cyberint.rs` | Same for Cyberint auth module |
| `armis.rs` | Same for Armis auth module |

Each file receives a header comment:
```rust
// ADR-023 §DTU-EXEMPT: This file IS the DTU behavioral clone for {Sensor}.
// Sensor-named references here are intentional — this IS the clone, not a consumer.
// Exempt from tests/external/no-hardcoded-sensors/ compile-fail gate.
// Imports from deleted prism-sensors::auth::{sensor} modules (001-A) must be updated
// to use the post-001-A SensorAuth open trait path.
```

If any stale import from a deleted module is found, it is removed and replaced with
the correct post-001-A equivalent. This is a blocking requirement — a compile error
on these files would break the workspace build.

(traces to BC-2.01.013 invariant — the spec-driven DataSource pattern supersedes per-sensor Rust code in all non-clone contexts; DTU clones are the only legitimate sensor-named code remaining post-001-A)

---

## Tasks

- [ ] **Task 1:** Read the 8 non-DTU-generator test files listed in §Inputs; identify every
      sensor-named import (`use prism_sensors::auth::*`, `SensorType::*`, `CrowdStrikeAdapter::*`)
- [ ] **Task 2:** For each parity test file (4 files): rewrite to `SpecLoader::parse()` +
      `DtuHarness` construction pattern; rename test functions per AC-001 naming
- [ ] **Task 3:** Rewrite `bc_2_16_002_crowdstrike_two_step.rs` per AC-002; verify two-step
      fetch fires correctly against DTU clone in TOML-driven mode
- [ ] **Task 4:** Rewrite `crowdstrike_oauth2_plugin_tests.rs` per AC-003; add `#[ignore]`
      annotation with correct blocking-dep citation if DTU clone not available
- [ ] **Task 5:** Rewrite `crowdstrike_session_isolation.rs` per AC-004; verify SensorId
      string key works in session isolation test
- [ ] **Task 6:** Audit the 4 DTU generator test files (ac-005) and `ac_2_crowdstrike_fixture.rs`;
      add exemption comments or remove stale imports as appropriate
- [ ] **Task 7:** Create `tests/external/no-hardcoded-sensors/` crate per AC-006 template;
      verify `cargo check -p no-hardcoded-sensors` fails with E0432
- [ ] **Task 8:** Update `ci.yml` to add `no-hardcoded-sensors-compile-fail` job per AC-007
- [ ] **Task 9:** Audit 4 DTU harness clone files per AC-008; add exemption comments; remove
      stale imports from deleted prism-sensors::auth modules if present
- [ ] **Task 10:** Run `just check` to verify all rewritten tests pass (including `#[ignore]`
      tagged tests are syntactically valid); verify compile-fail crate fails as expected

---

## Architecture Compliance Rules

Extracted from ADR-023 and architecture section files:

1. **ADR-023 Rule 3 — No sensor-named references in non-DTU production code.** After
   this story, no test file outside `crates/prism-dtu-*/` may contain direct sensor
   adapter imports (`CrowdStrikeAdapter`, `ClarotyAuth`, etc.). The compile-fail crate
   enforces this at CI.

2. **perimeter-violation crate pattern.** The new `no-hardcoded-sensors/` crate MUST
   follow the exact Cargo.toml structure of `tests/external/perimeter-violation/`:
   separate `[workspace]` stanza, excluded from root `Cargo.toml` workspace members,
   CI-job assertion of non-zero exit from `cargo check`. Do NOT add it to
   `[workspace.members]` in the root `Cargo.toml`.

3. **DTU clone exemption.** `crates/prism-dtu-harness/src/clones/` and
   `crates/prism-dtu-{sensor}/` are explicitly exempt from the no-hardcoded-sensors
   perimeter. These ARE the DTU clones. The compile-fail crate must NOT depend on
   `prism-dtu-harness` or any `prism-dtu-{sensor}` crate.

4. **#[non_exhaustive] discipline** (CLAUDE.md §Conventions). Any new `pub` type added
   in the compile-fail crate's Cargo.toml dependencies should already have
   `#[non_exhaustive]`. No new types are added in this story — imports only.

5. **Forbidden dependency:** The `no-hardcoded-sensors` compile-fail crate MUST NOT
   depend on `prism-spec-engine`, `prism-query`, `prism-ocsf`, or `prism-dtu-*`.
   It depends only on `prism-sensors`. Build-time enforcement: adding any other
   crate dependency to this crate's `Cargo.toml` MUST cause an architect review finding.

---

## Library & Framework Requirements

| Library | Version | Usage |
|---------|---------|-------|
| `prism-sensors` (workspace crate) | current | compile-fail crate dependency only |
| `prism-spec-engine` (workspace crate) | current | rewritten test files: SpecLoader, PipelineExecutor |
| `prism-dtu-harness` (workspace crate) | current | rewritten test files: DtuHarness logical-isolation mode |
| `cargo nextest` | workspace pin | inner-loop test runner (`just iter prism-spec-engine`) |

---

## File Structure Requirements

| Action | File Path | Notes |
|--------|-----------|-------|
| CREATE | `tests/external/no-hardcoded-sensors/Cargo.toml` | New compile-fail crate |
| CREATE | `tests/external/no-hardcoded-sensors/src/main.rs` | Imports 4 deleted auth modules |
| MODIFY | `crates/prism-spec-engine/tests/parity/crowdstrike.rs` | Rewrite to TOML fixture loading |
| MODIFY | `crates/prism-spec-engine/tests/parity/claroty.rs` | Rewrite to TOML fixture loading |
| MODIFY | `crates/prism-spec-engine/tests/parity/cyberint.rs` | Rewrite to TOML fixture loading |
| MODIFY | `crates/prism-spec-engine/tests/parity/armis.rs` | Rewrite to TOML fixture loading |
| MODIFY | `crates/prism-spec-engine/tests/bc_2_16_002_crowdstrike_two_step.rs` | Rewrite to TOML + DTU harness |
| MODIFY | `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs` | Rewrite per AC-003 |
| MODIFY | `crates/prism-query/tests/crowdstrike_session_isolation.rs` | SensorId string key per AC-004 |
| MODIFY | `crates/prism-dtu-armis/tests/bc_3_4_armis_generator.rs` | Exemption comment / import fix |
| MODIFY | `crates/prism-dtu-claroty/tests/bc_3_4_claroty_generator.rs` | Exemption comment / import fix |
| MODIFY | `crates/prism-dtu-crowdstrike/tests/bc_3_4_crowdstrike_generator.rs` | Exemption comment / import fix |
| MODIFY | `crates/prism-dtu-cyberint/tests/bc_3_4_cyberint_generator.rs` | Exemption comment / import fix |
| MODIFY | `crates/prism-dtu-demo-server/tests/ac_2_crowdstrike_fixture.rs` | Audit + exemption or rewrite |
| MODIFY | `crates/prism-dtu-harness/src/clones/crowdstrike.rs` | Audit + stale import removal + exemption comment |
| MODIFY | `crates/prism-dtu-harness/src/clones/claroty.rs` | Same |
| MODIFY | `crates/prism-dtu-harness/src/clones/cyberint.rs` | Same |
| MODIFY | `crates/prism-dtu-harness/src/clones/armis.rs` | Same |
| MODIFY | `.github/workflows/ci.yml` | Add no-hardcoded-sensors-compile-fail job |

---

## Previous Story Intelligence

Previous stories PLUGIN-MIGRATION-001-A through -E and S-PLUGIN-CI-001 (all merged):

1. **001-A (merged PR #156):** Deleted `prism-sensors/src/auth/{armis,claroty,crowdstrike,cyberint}.rs`.
   Any test file that imported from these paths is now broken at compile time — that is the
   primary driver for this story. The init_registry_for_org replacement must be in place.
2. **001-B (merged PR #157):** Converted 5 prism-query dispatch sites. crowdstrike_session_isolation.rs
   now needs to use the spec-catalog SensorId path, not SensorType.
3. **001-C (merged PR #158):** Deleted 4 OCSF mapper modules. prism-ocsf parity tests are
   already updated (001-C scope); this story does not re-touch those.
4. **001-D (merged PR #153):** 4 production TOML sensor specs authored. The TOML files
   are the fixture source for the rewritten parity tests.
5. **S-PLUGIN-CI-001 (merged PR #159):** WASM plugin CI toolchain + built `crowdstrike-oauth2.prx`.
   The plugin artifact path must be cited in `crowdstrike_oauth2_plugin_tests.rs` rewrite.

Key lesson from prior PLUGIN-MIGRATION stories: **use grep to verify all callsites before
declaring a rewrite complete.** Use `rg 'CrowdStrikeAdapter\|CrowdStrikeAuth\|ClarotyAuth\|ArmisAuth\|CyberintAuth\|SensorType::' crates/ --type rust` to find residual sensor-named references.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A test file imports from a deleted module (compile error) | Fix the import; do not skip or `#[ignore]` — compile errors block the workspace build |
| EC-002 | A DTU generator test file contains `SensorType::*` enum reference | Replace with `SensorId::from("sensor_name")`; add exemption comment |
| EC-003 | The no-hardcoded-sensors crate accidentally compiles (CI false-pass) | This is a P0 defect: a deleted module was re-exported. Route to architect per Canonical Principle Rule 4. |
| EC-004 | A parity test rewrite fails because a TOML spec field is missing | Report as a TOML spec defect (P1); do NOT silently assert against an empty result |
| EC-005 | The CI yml job naming conflicts with existing jobs | Use exact name `no-hardcoded-sensors-compile-fail` to match the perimeter-violation job naming convention |

---

## Forbidden Dependencies

The `no-hardcoded-sensors` compile-fail crate must NOT depend on:
- `prism-spec-engine` — not needed; only `prism-sensors` required for the delete-check
- `prism-query` — same
- `prism-ocsf` — same
- `prism-dtu-harness` or `prism-dtu-*` — would false-positive on legitimate DTU clone exemptions
- `prism-bin` — not needed

If the compile-fail crate gains any of these dependencies, the build will transitively import
things that should not be tested here. This must fail a CI lint check or architect review.

---

## Changelog

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| v1.0 | 2026-05-27 | story-writer | Initial draft — 8 AC + 10 tasks; PLUGIN-MIGRATION-001-F Wave 2 materialization |
| v1.1 | 2026-05-27 | story-writer | MED-001: BC-2.16.009 title corrected to canonical form in BC table and frontmatter comment. LOW-001: `SpecLoader::load_all()` → `SpecLoader::parse()` in AC-001 (table, bullet, trace note) and BC table. LOW-002: `red_gate_tests` 6 → 7 (implementation delivers 7 named test functions). |
| v1.2 | 2026-05-27 | story-writer | IMP-001: BC-2.16.012 body table title corrected to canonical "PluginRegistry Dispatch in spec_parser.rs — Hardcoded Sensor Names Replaced with Registry Lookup" (was "PluginRegistry Dispatch Migration"). IMP-002: `inputs:` path corrected from `BC-2.16.009-bundled-spec-validation.md` to `BC-2.16.009-spec-file-validation.md`. IMP-003: `SpecLoader::load()` → `SpecLoader::parse()` in §Story-Level Goal bullet 1; `SpecLoader::load_all()` → `SpecLoader::parse()` in Task 2. OBS-003: BC-2.16.009 version v1.4 → v1.5; BC-2.16.012 version v1.3 → v1.33. OBS-001: AC-006 CI snippet updated to use `--manifest-path tests/external/no-hardcoded-sensors/Cargo.toml` + `--color=never` (was `-p no-hardcoded-sensors`). OBS-002: AC-007 CI YAML template updated with `--color=never`, `--manifest-path`, log capture, and per-symbol positive-coverage assertions for all 4 deleted auth symbols (mirrors perimeter-compile-fail job pattern). |
