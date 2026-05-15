---
document_type: story
story_id: S-PLUGIN-PREREQ-D
title: "prism-bin/prism-spec-engine: Wire PluginRuntime into Boot Sequence; .prx Load Pipeline (Unsigned v1.0; Boot Warning + Audit Log; Allowlist Enforcement; PR Template)"
wave: 0
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: draft
# BC status: behavioral_contracts populated — BC-2.16.002 (Structured Event Catalog /
#            PipelineExecutor multi-step fetch — active since PREREQ-B merge; PREREQ-D
#            adds 9 new event_type rows and enforces MAX_REQUESTS_PER_PIPELINE cap),
#            BC-2.17.001..004 (plugin capability + WIT isolation contracts: panic
#            isolation, fs sandbox, memory, cpu), BC-2.17.006 (WIT interface validation),
#            BC-2.17.007 (manifest schema validation — NEW, landed
#            wave-4-fix-burst-F-LP1-HIGH-004), and BC-2.22.001 (boot orchestration).
#            BC-2.22.001 active; remaining 6 plugin BCs (BC-2.17.001/002/003/004/006/007) draft pending POL-14 promotion at this story's PR merge.
#            BC-2.17.005 (hot-reload watcher) is NOT in this list: PREREQ-D delivers only
#            the programmatic hot_reload() API surface; boot notify watcher wiring is
#            S-1.12-FOLLOWUP scope (see Out of Scope). BC-2.17.005 will be promoted by
#            S-1.12-FOLLOWUP, not by this story.
#            This story closes the TODO(S-4.08) in make_host_state() and the None-allowlist
#            short-circuit in host_http_request (ADR-023 §C4 F-CRIT-NEW-002).
behavioral_contracts: [BC-2.16.002, BC-2.17.001, BC-2.17.002, BC-2.17.003, BC-2.17.004, BC-2.17.006, BC-2.17.007, BC-2.22.001]
verification_properties:
  - VP-PLUGIN-004
  - VP-PLUGIN-007
depends_on:
  - S-PLUGIN-PREREQ-F
  - S-PLUGIN-PREREQ-A
  - S-PLUGIN-PREREQ-B
  - S-PLUGIN-PREREQ-C
blocks:
  - PLUGIN-MIGRATION-001-C
  - PLUGIN-MIGRATION-001-D
  - PLUGIN-MIGRATION-001-E
  - S-PLUGIN-PREREQ-E
points: 13
estimated_days: 5
risk: HIGH
tdd_mode: strict
crates_touched: [prism-bin, prism-spec-engine]
target_module: prism-bin
# Subsystem anchor justifications:
#   SS-22 (Process Lifecycle, prism-bin) owns the boot-sequence wiring: the new plugin-load
#   step is inserted into crates/prism-bin/src/boot.rs between ADR-022 canonical step 7
#   (storage init) and step 8 (query-engine). BC-2.22.001 is the primary contract.
#   SS-17 (WASM Plugin Runtime, prism-spec-engine) owns all sandbox BCs (BC-2.17.001..007) and
#   the PluginRuntime type itself. The allowlist enforcement, WIT validation, and manifest
#   format_version check all land in crates/prism-spec-engine/src/plugin/.
#   SS-16 (Spec Engine, prism-spec-engine) owns BC-2.16.002 (Multi-Step Fetch Pipeline
#   Execution — Sequential Steps with Variable Interpolation) and the pipeline.rs
#   MAX_REQUESTS_PER_PIPELINE cap (AC-16). Per S-PLUGIN-PREREQ-B precedent at
#   subsystems: [SS-16, SS-01], any story anchoring BC-2.16.002 must list SS-16.
subsystems: [SS-22, SS-17, SS-16]
capabilities: [CAP-029, CAP-032, CAP-034]
version: "1.36"
level: "L4"
producer: story-writer
timestamp: "2026-05-14T13:00:00Z"
updated: "2026-05-15"
input-hash: "6954524"
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
anchor_vps: [VP-PLUGIN-004, VP-PLUGIN-007]
anchor_bcs: [BC-2.16.002, BC-2.17.001, BC-2.17.002, BC-2.17.003, BC-2.17.004, BC-2.17.006, BC-2.17.007, BC-2.22.001]
anchor_capabilities: [CAP-029, CAP-032, CAP-034]
anchor_subsystem: [SS-22, SS-17, SS-16]
assumption_validations:
  - "reqwest::Client construction is fallible (per EC-D-009) — propagated via PrismError::Internal per ADR-022 §A exit code 4 (AC-9)"
  - "Plugin signing not implemented in v1.0 — unsigned plugins load with WARN + per-plugin audit entry per AC-4 (deferred to TD-PLUGIN-SIGNING-001)"
  - "PRISM_DISABLE_PLUGIN_LOAD envvar is read once at boot (no hot-reload of disable state) per AC-3 and AC-18"
  - "host_http_request enforces allowlist via host-only == comparison (no substring matching) per AC-7"
  - "PluginRuntime::new creates fresh Store per plugin call (no cross-call WASM state) per AC-10"
risk_mitigations:
  - "AC-9: 30s per-request timeout via reqwest::Client::builder().timeout() — bounded HTTP latency (closes TD-S-PLUGIN-PREREQ-B-005)"
  - "AC-13: Wasmtime epoch ticker started once at PluginRuntime::new() prevents resource leak across N plugins (BC-2.17.004)"
  - "AC-10: Fresh Store per plugin call prevents cross-call WASM state leakage; PluginError::Trapped on guest panic per BC-2.17.001"
  - "AC-1: Hard sequencing — plugin-load step 7.5 between storage init (step 7) and query-engine init (step 8) per BC-2.22.001 §Sequencing Invariant"
  - "AC-2: §Pre-Traffic Gate Invariant condition 6 — plugin-load step 7.5 completion required before MCP server bind (TRAFFIC GATE OPEN)"
  - "AC-3: PRISM_DISABLE_PLUGIN_LOAD escape valve — auditable disable via single tracing::warn! event (no plugin loaded; MCP server still binds)"
  - "AC-16: MAX_REQUESTS_PER_PIPELINE=10_000 hard cap prevents unbounded plugin HTTP calls per pipeline (closes TD-S-PLUGIN-PREREQ-B-004)"
  - "AC-7: allowed_urls: Vec<String> field-type contract per AC-17 makes None-branch type-system-impossible; host-only == comparison enforces allowlist without substring bypass"
# F-LP17-OBS-001 closure: arrays populated from existing AC body content per Path A.
# Process-gap candidate 7 (template enforcement for risk:HIGH stories) routes to cycle-closing for orchestrator session-reviewer adjudication.
acceptance_criteria_count: 18
red_gate_tests: 25
estimated_passes: "8-12 LOCAL adversary passes"
# TD items absorbed by this story
td_resolves:
  - TD-S-PLUGIN-PREREQ-B-002  # P3 — AuthToken zeroize on Drop (credential-store integration scope)
  - TD-S-PLUGIN-PREREQ-B-004  # P3 — MAX_REQUESTS_PER_PIPELINE cumulative cap (OBS-LP2-001)
  - TD-S-PLUGIN-PREREQ-B-005  # P2 — production reqwest::Client timeout(30s) in boot wiring
  - TD-S-PLUGIN-PREREQ-B-011  # P3 — execute_step eager-token semantic consistency in PREREQ-D wiring tests
  - TD-S-PLUGIN-PREREQ-B-012  # P3 — execute_step PREREQ-D wiring test coverage
inputs:
  - ".factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/behavioral-contracts/BC-2.17.001-plugin-panic-isolation.md"
  - ".factory/specs/behavioral-contracts/BC-2.17.002-plugin-sandbox-filesystem.md"
  - ".factory/specs/behavioral-contracts/BC-2.17.003-plugin-memory-limit.md"
  - ".factory/specs/behavioral-contracts/BC-2.17.004-plugin-cpu-time-limit.md"
  - ".factory/specs/behavioral-contracts/BC-2.17.005-plugin-hot-reload-atomic-swap.md"
  - ".factory/specs/behavioral-contracts/BC-2.17.006-plugin-wit-validation.md"
  - ".factory/specs/behavioral-contracts/BC-2.17.007-plugin-manifest-schema-validation.md"
  - ".factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md"
  - ".factory/specs/verification-properties/VP-INDEX.md"
  - ".factory/specs/prd-supplements/error-taxonomy.md"
  - ".factory/stories/S-PLUGIN-PREREQ-A-sensorid-newtype.md"
  - ".factory/stories/S-PLUGIN-PREREQ-B-real-pipeline-executor.md"
  - ".factory/stories/S-PLUGIN-PREREQ-C-toml-grammar-extensions-plus-pub-api-hardening.md"
  - ".factory/tech-debt-register.md"
  - ".factory/cycles/wave-4-operations/forward-task-map.md"
---

# S-PLUGIN-PREREQ-D: prism-bin/prism-spec-engine — PluginRuntime Boot Wiring + .prx Load Pipeline

## Narrative
- **As a** Prism platform operator
- **I want** `PluginRuntime` wired into the boot sequence so `.prx` WASM plugins are loaded, validated, and sandbox-enforced before MCP traffic is accepted
- **So that** first-party OCSF complex-transform plugins are available at startup, allowlist enforcement is active, and the platform is ready for Wave 1 plugin-migration stories (PLUGIN-MIGRATION-001-C/D/E)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `PluginRuntime` | `crates/prism-spec-engine/src/plugin/mod.rs` | Effectful (filesystem scan, WASM compilation, arc-swap registry) |
| `load_all_plugins` | `crates/prism-spec-engine/src/plugin/mod.rs` | Effectful (I/O, spawn_blocking, audit emit) |
| `host_http_request` | `crates/prism-spec-engine/src/plugin/host_functions.rs` | Effectful (outbound HTTP via reqwest) |
| `make_host_state` | `crates/prism-spec-engine/src/plugin/mod.rs` | Pure (constructs HostState from parsed manifest + injected Arc<reqwest::Client>; no I/O; receives http_client by Arc::clone — does NOT construct the client) |
| `validate_wit_interface` | `crates/prism-spec-engine/src/plugin/mod.rs` | Pure (inspects Component exports) |
| Boot plugin-load step | `crates/prism-bin/src/boot.rs` | Effectful (filesystem, audit, epoch ticker start) |
| `AuthToken` | `crates/prism-spec-engine/src/auth_provider.rs` | Pure-core (value type with zeroize-on-drop) |
| `PipelineExecutor` | `crates/prism-spec-engine/src/pipeline.rs` | Effectful (HTTP fetches, auth acquisition) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-D-001 | Plugin directory does not exist at boot | `load_all_plugins` returns `Ok(0)`; INFO log "plugin directory not found, skipping"; boot continues |
| EC-D-002 | Plugin directory exists but contains zero `.prx` files | `Ok(0)`; INFO log; unsigned-plugin WARN not emitted (no plugins loaded) |
| EC-D-003 | One of N plugins fails manifest validation (missing `allowed_urls`) | That plugin rejected with `E-PLUGIN-013`; remaining N-1 continue loading; `load_all_plugins` returns `Ok(N-1)` |
| EC-D-004 | `PRISM_DISABLE_PLUGIN_LOAD=1` set; plugin directory has valid plugins | Skip all loading; single `tracing::warn!(event_type = "plugin_load_disabled_via_envvar", ...)` emission (BC-2.16.002 catalog; AC-3); `Ok(0)` |
| EC-D-005 | Plugin manifest `format_version = 0` (below `CURRENT_SUPPORTED_VERSION = 1`) | Accepted (version <= supported); loaded normally |
| EC-D-006 | Plugin manifest `format_version = 2` (above `CURRENT_SUPPORTED_VERSION = 1`) | Rejected with `E-PLUGIN-014`; ERROR log naming `format_version` and `max_supported` |
| EC-D-007 | `Component::from_binary` fails (corrupt `.prx` bytes) | `E-PLUGIN-008` logged; plugin skipped; other plugins continue |
| EC-D-008 | Two `.prx` files declare the same `plugin_id` | Second load logs `WARN "Duplicate plugin_id '{id}': first-registered plugin retained"`; first wins (BC-2.17.006 invariant) |
| EC-D-009 | `reqwest::Client` construction fails (OS resource exhaustion) | `PluginRuntime::new` returns `Err`; boot exits code 4 |
| EC-D-010 | Plugin calls `host::http_request` to URL not in `allowed_urls` | HTTP 403 returned to plugin; single `tracing::warn!(event_type = "plugin_http_request_blocked", ...)` emission (BC-2.16.002 catalog; AC-7) |
| EC-D-011 | `PRISM_DISABLE_PLUGIN_LOAD` set to non-"1" value (e.g., "true", "yes") | Only exact string `"1"` disables loading; other values treated as unset |
| EC-D-012 | Plugin manifest `name` field is empty string or absent | Rejected with `E-PLUGIN-015`; ERROR log naming `'name'` field; other plugins continue (n-1 survivor) |
| EC-D-013 | Plugin manifest `version` field is not valid semver | Rejected with `E-PLUGIN-016`; ERROR log naming `'version'` field with offending value; other plugins continue (n-1 survivor) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `crates/prism-spec-engine/src/plugin/mod.rs` — `make_host_state()` | pure-core | Deterministic construction from inputs; no I/O. Receives `Arc<reqwest::Client>` by Arc::clone (injected dependency) — does NOT construct the client. purity preserved: same input → same HostState. |
| `crates/prism-spec-engine/src/plugin/mod.rs` — `validate_wit_interface()` | pure-core | Inspects Component export names; no I/O; suitable for unit test |
| `crates/prism-spec-engine/src/plugin/mod.rs` — `load_all_plugins()` | effectful-shell | Filesystem scan, WASM compilation (`spawn_blocking`), arc-swap mutation, audit emit |
| `crates/prism-spec-engine/src/plugin/host_functions.rs` — `host_http_request` | effectful-shell | Outbound HTTP via reqwest; allowlist check is pure sub-step |
| `crates/prism-spec-engine/src/pipeline.rs` — `MAX_REQUESTS_PER_PIPELINE` check | pure-core | Counter comparison against constant; extractable for unit test |
| `crates/prism-bin/src/boot.rs` — plugin-load step | effectful-shell | Calls `load_all_plugins`; reads env var; emits audit; constructs single shared `reqwest::Client` with 30s timeout and injects into `PluginRuntime::new()` |
| `crates/prism-spec-engine/src/auth_provider.rs` — `AuthToken` | pure-core (value type) | Holds credential bytes; `Zeroizing<String>` wrapper is drop-safe pure value |

## Library & Framework Requirements (MANDATORY)

| Library | Version | Purpose | Pin Note |
|---------|---------|---------|----------|
| `wasmtime` | `44` (exact crate pin) | WASM Component Model runtime; epoch interruption; StoreLimits | RUSTSEC advisory comment in `prism-spec-engine/Cargo.toml` — do not change without security rationale |
| `zeroize` | `"1"` | `AuthToken` zeroing on drop (TD-S-PLUGIN-PREREQ-B-002 / AD-017) | Accept any `1.x`; ADD to `crates/prism-spec-engine/Cargo.toml` — currently absent from that file |
| `sha2` | `"0.10"` | SHA-256 `plugin_hash` field in audit entry | Already pinned at `sha2 = "0.10"` in `crates/prism-spec-engine/Cargo.toml` (crate-local pin, line 21); no Cargo.toml change required for the sha2 dep itself. |
| `url` | `"2"` | URL host extraction for allowlist enforcement in `host_http_request` | NOT currently in `crates/prism-spec-engine/Cargo.toml`; ADD `url = "2"` (or current ecosystem-compatible 2.x version) with comment `# Used by PluginRuntime::host_http_request for allowlist host extraction`. |
| `reqwest` | `"0.12"` | HTTP client; MUST use `.timeout(Duration::from_secs(30))` builder | Already pinned at `reqwest = { version = "0.12", ... }` in `crates/prism-spec-engine/Cargo.toml` (crate-local pin, line 34); no version change required. Builder pattern mandatory; no bare `reqwest::Client::new()`. |
| `arc-swap` | `"1"` | Lock-free atomic registry updates for hot-reload | Already pinned at `arc-swap = "1"` in `crates/prism-spec-engine/Cargo.toml` (crate-local pin, line 20); no Cargo.toml change required. |
| `tokio` | `"1"` | `spawn_blocking` for CPU-intensive WASM compilation | Already pinned at `tokio = { version = "1", ... }` in `crates/prism-spec-engine/Cargo.toml` (crate-local pin, line 26); no Cargo.toml change required. |

Do NOT invent version numbers. Use the exact versions confirmed above from `crates/prism-spec-engine/Cargo.toml`. The `zeroize` dep is currently absent from that file (confirmed by inspection; not in `[dependencies]` lines 12-36 nor in `[dev-dependencies]` lines 38-54); add at version `"1"` with an explanatory comment. Note: the workspace root `Cargo.toml` has no `[workspace.dependencies]` table — all dep versions are crate-local pins.

## Summary

This story completes the `PluginRuntime` infrastructure in `prism-spec-engine` and wires it
into the `prism-bin` boot sequence as a new plugin-load step positioned between ADR-022
canonical step 7 (storage init) and step 8 (query-engine init). At boot, `PluginRuntime::load_all_plugins`
scans the plugin directory for `.prx` WASM Component files, validates each plugin's WIT interface
and manifest (`name`, `version`, `format_version`, `allowed_urls`), rejects plugins whose manifest
fails any of four schema validations (E-PLUGIN-013 missing `allowed_urls`; E-PLUGIN-014 `format_version` exceeds `CURRENT_SUPPORTED_VERSION`; E-PLUGIN-015 missing or empty `name`; E-PLUGIN-016 malformed `version` semver), and
emits per-plugin audit entries (`event_type: plugin_load_unsigned`) accompanied by a one-time
boot-level WARN log — because plugin signing is deferred to v1.0+N (TD-PLUGIN-SIGNING-001).
A `PRISM_DISABLE_PLUGIN_LOAD=1` escape valve skips loading entirely. The story also closes the
`TODO(S-4.08)` in `make_host_state()` by replacing `allowed_urls: None` with a per-plugin
allowlist parsed from the manifest, and replaces the `host_http_request` None-short-circuit with
host-only allowlist enforcement. The `.github/PULL_REQUEST_TEMPLATE.md` file with the three-item
sensor-pattern checklist is delivered here (ADR-023 §C4, F-PASS3-MED-001). Five carry-forward
technical debt items from PREREQ-B are absorbed: `AuthToken` zeroize-on-drop,
`MAX_REQUESTS_PER_PIPELINE` cumulative cap, production `reqwest::Client` 30-second timeout,
and two `execute_step` eager-token wiring test obligations.

## Background

ADR-023 §C4 defines PREREQ-D's scope as the keystone infrastructure delivery enabling all Wave 1
plugin-migration stories. Without this story, `PluginRuntime` exists in
`crates/prism-spec-engine/src/plugin/mod.rs` but is not called from boot; the `make_host_state()`
function constructs `HostState { allowed_urls: None }` with an open TODO; and `host_http_request`
permits all URLs when `allowed_urls` is `None`. As of the S-WAVE5-PREP-01 merge (commit
`53b87961`), `crates/prism-bin/src/boot.rs` implements canonical steps 7-11 as `todo!()` stubs.
PREREQ-D fills the plugin-load step stub, renumbers subsequent steps accordingly, and satisfies
the BC-2.22.001 sequencing invariant (plugin load after storage, before query-engine). The
unsigned-plugin warning and audit log are required v1.0 behavior per ADR-023 §C4 — operators must
be aware that plugins are not cryptographically verified. PLUGIN-MIGRATION-001-D (author 4
production TOML sensor specs), PLUGIN-MIGRATION-001-C (SpecDrivenMapper), PLUGIN-MIGRATION-001-E
(CrowdStrike OAuth2 .prx plugin), and S-PLUGIN-PREREQ-E (CustomAdapter removal) all depend on
this story being merged first.

## Scope

### In scope

- Complete `PluginRuntime::load_all_plugins(dir: &Path)` to scan `*.prx` files, compile each
  via `Component::from_binary` in `spawn_blocking`, validate WIT interface (BC-2.17.006), validate
  manifest schema (4 fields: `name` non-empty per E-PLUGIN-015; `version` semver-parseable per
  E-PLUGIN-016; `format_version <= CURRENT_SUPPORTED_VERSION` per E-PLUGIN-014; `allowed_urls`
  explicitly present per E-PLUGIN-013 — empty list `[]` accepted, absent/null rejected), and
  register in the arc-swap registry
- Wire `PluginRuntime::load_all_plugins` into `crates/prism-bin/src/boot.rs` as a new step
  between ADR-022 canonical step 7 (storage init) and the next step (query-engine init); renumber
  subsequent steps accordingly; update `BC-2.22.001` sequencing invariant reference comment in code
- Implement `PRISM_DISABLE_PLUGIN_LOAD=1` environment variable escape valve
- Boot-time WARN log: `"WARNING: Plugin signing not yet implemented (TD-PLUGIN-SIGNING-001). Loaded plugins are NOT cryptographically verified. Do not run untrusted plugins."`
- Audit log entry per plugin load: `event_type: plugin_load_unsigned`, `plugin_path: <path>`, `plugin_hash: <sha256-hex>`
- Boot-time audit entry when plugin load is disabled: `event_type: plugin_load_disabled_via_envvar`
- Close `TODO(S-4.08)` in `make_host_state()`: parse `allowed_urls` from manifest TOML, construct `HostState { allowed_urls: parsed_hostnames }`
- Replace `host_http_request` None-short-circuit with host-only comparison enforcement against allowlist (VP-PLUGIN-007)
- Reject plugins whose manifest omits `allowed_urls` with `E-PLUGIN-013` (new error code — see Error Taxonomy section)
- Reject plugins whose manifest `format_version` exceeds `CURRENT_SUPPORTED_VERSION` with `E-PLUGIN-014`
- Reject plugins whose manifest `name` field is missing or empty string with `E-PLUGIN-015`
- Reject plugins whose manifest `version` field is not valid semver with `E-PLUGIN-016`
- Validate the `wasmtime::Linker` import list at build time via `#[cfg(test)]` assertion (ADR-023 §C4)
- `AuthToken` zeroize-on-drop (`Zeroizing<String>` or explicit `Drop` impl) — TD-S-PLUGIN-PREREQ-B-002
- `MAX_REQUESTS_PER_PIPELINE = 10_000` cumulative cap in `pipeline.rs` executor loop — TD-S-PLUGIN-PREREQ-B-004
- Production `reqwest::Client` construction with `.timeout(Duration::from_secs(30))` in boot wiring — TD-S-PLUGIN-PREREQ-B-005
- Integration tests asserting `execute_step` eager-token semantics (`MockAuthProvider.calls() == 1` per invocation) — TD-S-PLUGIN-PREREQ-B-011/012
- `.github/PULL_REQUEST_TEMPLATE.md` with three-item sensor-pattern checklist (ADR-023 §C4 F-PASS3-MED-001)
- `tests/fixtures/minimal.prx` committed as a binary artifact for integration tests (see Fixture Strategy)

### Out of scope

- Plugin signing/verification (deferred to TD-PLUGIN-SIGNING-001, target v1.0+N)
- Hot-reload watcher wiring into boot (BC-2.17.005 hot-reload is implemented in `prism-spec-engine` already; boot `notify` watcher setup is S-1.12-FOLLOWUP scope, blocked on PLUGIN-MIGRATION-001-A)
- `CustomAdapter` / `CustomAdapterRegistry` deletion (S-PLUGIN-PREREQ-E scope)
- `.prx` build toolchain / `cargo component` setup (out of scope — plugins are pre-built WASM artifacts for v1.0)
- Plugin instance pool / cold-start optimization (deferred per ADR-023 §C4 latency-target note)
- `PLUGIN-MIGRATION-001-D` TOML sensor spec authoring (separate story)

## Behavioral Contracts

| BC | Title | Primary Coverage |
|----|-------|-----------------|
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | AC-16 (MAX_REQUESTS_PER_PIPELINE cap; traces to BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet) row pipeline_max_requests_exceeded); Structured Event Catalog Additions §intro (13 new event_type rows) |
| BC-2.17.001 | Plugin Panic Isolation — Crashed Plugin Does Not Terminate Host Process | AC-10 (panic isolation via fresh Store per call) |
| BC-2.17.002 | Plugin Sandbox — No Direct Filesystem or Network Access | AC-11 (WASI not linked; allowlist enforcement via host_http_request) |
| BC-2.17.003 | Plugin Sandbox — Memory Limit Enforced Per Plugin Instance (default 64MB) | AC-12 (StoreLimits 64MB; configurable per manifest) |
| BC-2.17.004 | Plugin Sandbox — CPU Time Limit Enforced via Epoch Interruption (default 5s) | AC-13 (epoch ticker started once; per-call deadline) |
| BC-2.17.006 | WIT Interface Validation Before Plugin Registration | AC-6 (WIT export check; E-PLUGIN-001 on missing export) |
| BC-2.17.007 | Plugin Manifest Schema Validation Before WIT Validation | AC-5 (manifest field presence + format_version + allowed_urls; E-PLUGIN-013/014/015/016) |
| BC-2.22.001 | Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate | AC-1, AC-2, AC-3, AC-4 (boot step placement; gate; escape valve; unsigned warning) |

Note: BC-2.17.005 (Plugin Hot Reload — Atomic Module Swap, In-Flight Calls Complete Against Old Version) is NOT anchored to this story.
PREREQ-D delivers the programmatic `hot_reload()` API surface (AC-14 confirms the arc-swap
mechanism remains intact after boot wiring). The `notify` file-watcher installation into the
boot chassis is S-1.12-FOLLOWUP scope (blocked on PLUGIN-MIGRATION-001-A). BC-2.17.005 will
be promoted to active by S-1.12-FOLLOWUP, not by this story.

## Acceptance Criteria

### AC-1 — Plugin-load step inserted between storage and query-engine init (traces to BC-2.22.001 §Sequencing Invariant — step 7.5 intercalation between storage step 7 and query-engine step 8)

`crates/prism-bin/src/boot.rs` calls `PluginRuntime::load_all_plugins(&plugin_dir)` in a step
that runs after the ADR-022 canonical step 7 (storage init / `StorageEngine` construction) and
before the query-engine init step. The plugin directory path is resolved from `PrismConfig`
(field: `plugin_dir`, default `"plugins/"` relative to the config file location). All subsequent
boot steps are renumbered in code comments to reflect the new sequence. The step log line emitted
is: `INFO "boot: plugin-load step complete ({n} plugins loaded)"`.

### AC-2 — Pre-traffic gate holds: MCP server does not bind before plugin-load step completes (traces to BC-2.22.001 §Pre-Traffic Gate Invariant condition 6 — plugin-load step 7.5 must complete or be audited-disabled before MCP server bind proceeds)

If `PluginRuntime::load_all_plugins` returns `Err`, the boot sequence exits immediately with exit
code 4 (internal-error per ADR-022 §A) and the MCP server never binds. If `load_all_plugins`
returns `Ok`, the MCP server bind (step 9 gate) may proceed. Verified by integration test with
injected plugin-load failure (see Red Gate Tests).

### AC-3 — PRISM_DISABLE_PLUGIN_LOAD=1 skips plugin loading; audit entry emitted (traces to BC-2.22.001 §Postconditions — `PRISM_DISABLE_PLUGIN_LOAD=1` escape valve postcondition; `plugin_load_disabled_via_envvar` audit event name; ADR-023 §C4)

When `PRISM_DISABLE_PLUGIN_LOAD=1` is set at boot, `PluginRuntime::load_all_plugins` is not
called. A single `tracing::warn!(event_type = "plugin_load_disabled_via_envvar", message =
"Plugin loading disabled via PRISM_DISABLE_PLUGIN_LOAD=1", env_var = "PRISM_DISABLE_PLUGIN_LOAD",
...)` emission is made before the step is skipped. This one structured emission satisfies BOTH
the WARN-level operator-visible log AND the audit-channel routing — WARN-level log and
audit-channel routing are orthogonal via `event_type` field per BC-2.16.002 v1.16 §Postconditions (Canonical Structured Event Catalog bullet) (row plugin_load_unsigned Trigger cell). The MCP server bind proceeds normally (zero plugins registered).

### AC-4 — Unsigned-plugin boot warning + per-plugin audit entry emitted (traces to BC-2.22.001 §Postconditions — happy-path plugin-load step 7.5 postcondition: `plugin_load_unsigned` audit event with `plugin_path` + `plugin_hash` fields; ADR-023 §C4; VP-PLUGIN-004)

For every `.prx` plugin successfully loaded at boot, the following are emitted before the
plugin-load step completes:

1. One boot-time WARN (emitted once per boot, not per plugin): `"WARNING: Plugin signing not yet implemented (TD-PLUGIN-SIGNING-001). Loaded plugins are NOT cryptographically verified. Do not run untrusted plugins."`
2. Per-plugin audit entry: `event_type: plugin_load_unsigned`, `plugin_path: <path-to-prx>`, `plugin_hash: <sha256-hex-of-prx-bytes>`

The `plugin_hash` is the SHA-256 hex digest of the raw `.prx` file bytes (computed before WASM
compilation). These entries are written to the audit emitter (BC-2.05.012). Both items are
verified by integration test `test_VP_PLUGIN_004_unsigned_plugin_boot_warn_audit`.

### AC-5 — Manifest schema validation: name, version, format_version, allowed_urls required before WIT compilation (traces to BC-2.17.007 postconditions 1–5; BC-2.17.007 invariants; ADR-023 §C4)

`PluginRuntime::load_plugin` validates the plugin manifest embedded in the `.prx` file BEFORE
any WASM Component compilation is attempted (BC-2.17.007 manifest-before-WIT ordering invariant):

- `name`: non-empty UTF-8 string; absent or empty → `E-PLUGIN-015: "Plugin manifest at '{path}' missing or empty required field 'name'"` (BC-2.17.007 postcondition 1)
- `version`: non-empty semver-parseable string; absent or malformed → `E-PLUGIN-016: "Plugin manifest at '{path}' field 'version' is not a valid semver string: '{value}'"` (BC-2.17.007 postcondition 2)
- `format_version`: u32; must be `<= CURRENT_SUPPORTED_VERSION` (crate constant, initial value `1`); absent or `> CURRENT_SUPPORTED_VERSION` → `E-PLUGIN-014: "Plugin manifest at '{path}' format_version {actual} exceeds maximum supported version {supported}"` (BC-2.17.007 postcondition 3)
- `allowed_urls`: `Vec<String>`; field must be explicitly present (empty list `[]` is accepted; absent/null → rejection); absent or `None` → `E-PLUGIN-013: "Plugin manifest at '{path}' missing required field 'allowed_urls'; field must be an explicit list (use `allowed_urls = []` for no URLs)"` (BC-2.17.007 postcondition 4; VP-PLUGIN-007)

Field validation order: `name` → `version` → `format_version` → `allowed_urls`. First failure
returns immediately; one error per load attempt (BC-2.17.007 EC-17-032).

Manifest validation errors are logged at ERROR level. The plugin is NOT registered. No partial
registration state (BC-2.17.007 no-partial-registration invariant). Other plugins in the scan
continue loading (BC-2.17.007 postcondition 9: sibling-plugin isolation).

Only after all four manifest fields pass does control flow to WIT interface validation (AC-6 /
BC-2.17.006). Manifest validation passing is necessary but not sufficient for registration
(BC-2.17.007 postcondition 5).

### AC-6 — WIT interface validation before registration (traces to BC-2.17.006 postconditions)

After manifest schema validation passes (AC-5 / BC-2.17.007), `PluginRuntime::load_plugin`
calls `validate_wit_interface(component)` on the compiled WASM Component. A plugin missing
required WIT exports (`name`, `version`, and primary dispatch function) is rejected with
`E-PLUGIN-001`. The error log names the missing export. This behavior is unchanged from the
S-1.15 implementation but is explicitly confirmed by a new integration test using the
`bad_wit.prx` fixture (a Component with a deliberately missing required WIT export).

The ordering guarantee: manifest validation (BC-2.17.007) → WIT compilation → WIT validation
(BC-2.17.006) → registration. A plugin that fails at any gate is not registered.

### AC-7 — Allowlist enforcement in host_http_request: host-only comparison (traces to BC-2.17.002 postcondition; ADR-023 §C4 F-CRIT-NEW-002; VP-PLUGIN-007)

After PREREQ-D lands, `make_host_state()` constructs `HostState { allowed_urls: parsed_hostnames }`
(type `Vec<String>`, not `Option<Vec<String>>` — see AC-17). The `parsed_hostnames` are the bare
hostnames from the manifest `allowed_urls` list (e.g., `"api.crowdstrike.com"` not
`"https://api.crowdstrike.com/"`). `host_http_request` enforces:

- Extract the host from the requested URL using `url::Url::parse`; compare against each entry in
  `allowed_urls` using `==` (exact host-only match, not substring); mismatch → HTTP 403 returned
  to plugin AND a single `tracing::warn!(event_type = "plugin_http_request_blocked", plugin_id, url, reason = "allowlist_mismatch")` emission (single structured emission per BC-2.16.002 v1.16 catalog routing convention — WARN-level log and audit-channel routing are orthogonal via `event_type` field)

The `TODO(S-4.08)` comment in `make_host_state()` is removed. The None-short-circuit in
`host_http_request` is removed and replaced with the enforcement logic. The `Option<Vec<String>>`
→ `Vec<String>` field type change (AC-17) makes the None branch type-system-impossible; no
defensive None handling is specified or required.

### AC-8 — Linker import list validated at build time via #[cfg(test)] assertion (traces to BC-2.17.002 invariant; ADR-023 §C4)

A `#[cfg(test)] #[test] fn test_linker_imports_match_host_functions()` test in
`crates/prism-spec-engine/src/plugin/host_functions.rs` (or nearby) enumerates all imports
registered on the `wasmtime::component::Linker` during `PluginRuntime::build_linker()` and
asserts the count and names match the canonical host function list. This prevents import list
drift when new host functions are added. The test panics with a descriptive message on mismatch:
`"Linker import count mismatch: expected {N}, found {M}. Did you add a host function without updating the import list?"`.

### AC-9 — Single shared reqwest::Client constructed once at boot with 30-second timeout; injected into PluginRuntime (traces to BC-2.17.002 v1.7 §Error Conditions E-PLUGIN-005; closes TD-S-PLUGIN-PREREQ-B-005)

A **single** `reqwest::Client` instance is constructed **once** in `crates/prism-bin/src/boot.rs`
during the plugin-load boot step, using:

```rust
let http_client = reqwest::Client::builder()
    .timeout(Duration::from_secs(PLUGIN_HTTP_CLIENT_TIMEOUT_SECS))
    .build()
    .map_err(|e| PrismError::Internal {
        detail: format!("PluginRuntime HTTP client construction failed: {}", e),
    })?;
```

Client construction is fallible (OS resource exhaustion per EC-D-009). Construction failures
propagate `Err` to the boot sequence caller, which exits with code 4 per ADR-022 §A
(internal-error class). Using `.expect()` on this `Result` is **forbidden** — `expect_used = "deny"`
is set in the workspace `Cargo.toml` `[workspace.lints.clippy]` table, and `expect()` would
panic instead of returning the structured error that EC-D-009 requires.

`PrismError::Internal { detail: String }` is the canonical variant for internal/init failures
(`crates/prism-core/src/error.rs` lines 881-883; E-INT-001). It is mapped to `exit(4)` per ADR-022
§A line 146 (internal-error class). This preserves `PrismError`'s `#[derive(PartialEq, Eq)]`
because `String` is `Eq`; `reqwest::Error` is not `Eq` and cannot be stored as a struct field on
a `#[derive(PartialEq, Eq)]` type. The stringify pattern (`format!("...{}", e)`) matches the
existing project convention (`PrismError::Io(String)` at error.rs:171 for non-Eq source errors).
No new `E-PLUGIN` variant is required for HTTP client construction failure.

The constant `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS: u64 = 30` is defined in
`crates/prism-spec-engine/src/plugin/mod.rs`. The 30-second value is a per-request timeout
(each individual HTTP call gets 30 seconds), not a global session timeout.

The constructed client is passed into `PluginRuntime::new(engine, linker, http_client, kv_store)`
as an owned value and stored (wrapped in `Arc<reqwest::Client>`) on `PluginRuntime`. All plugins
share this single client instance — `make_host_state()` receives it by `Arc::clone`, so the
function remains Pure (no I/O, no construction side-effects; it merely clones the Arc reference).
The boot.rs site owns construction; `make_host_state()` owns distribution.

This is TD-S-PLUGIN-PREREQ-B-005 closure: the canonical reqwest::Client constructor lives in
`crates/prism-bin/src/boot.rs` and is injected into PluginRuntime via `Arc<reqwest::Client>`.
No bare `reqwest::Client::new()` calls are permitted in plugin-related code.

Integration tests that need HTTP mocking construct their own test-scoped client and inject it
directly into `PluginRuntime::new(...)` in the test setup (already done in PREREQ-B test
conventions).

> **Closed by BC-2.17.002 v1.4 amendment (fix-burst-6); current pinned version v1.7 (fix-burst-30 phantom-variant removal per F-LP32-CRIT-001):** BC-2.17.002 E-PLUGIN-005 timeout
> updated from 10s to 30s by product-owner in fix-burst-6 stage 1, aligning with ADR-023 §C4
> and `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS = 30`. No cross-doc gap remains.

### AC-10 — Plugin panic isolation: trap caught at Rust boundary; fresh Store per call (traces to BC-2.17.001 postconditions; INV-PLUGIN-001)

Per BC-2.17.001: `wasmtime::Store` is created fresh per plugin call. Plugin traps are caught at
the `instance.call_*` boundary and returned as `Err(PluginError::Trapped { plugin_id, message })`.
The host process continues. WARN log emitted per trap. Plugin registry entry retained. This
behavior is confirmed existing from S-1.15; PREREQ-D adds an integration test using
`tests/fixtures/trap_plugin.prx` (compiled from `tests/fixtures/src/trap_plugin.wat`) that
explicitly verifies trap isolation after the plugin-load step wires the runtime into boot.

### AC-11 — Filesystem and network sandbox: WASI not linked; allowlist enforced (traces to BC-2.17.002 invariants; INV-PLUGIN-002)

The `wasmtime::component::Linker` built by `PluginRuntime::build_linker()` does NOT register any
`wasi:filesystem`, `wasi:sockets`, `wasi:process`, or `wasi:environment` interfaces. This is
enforced by the Kani proof VP-040 (`vp-040-plugin-linker-no-wasi-imports.md`) and the linker
import list assertion (AC-8). All plugin outbound HTTP routes through `host_http_request`.

### AC-12 — Memory limit 64MB default enforced via StoreLimits; configurable per manifest (traces to BC-2.17.003 postconditions; INV-PLUGIN-003)

Each `wasmtime::Store` is constructed with `StoreLimits` setting max linear memory to
`memory_limit_mb * 1024 * 1024` bytes (default 64MB). The manifest `memory_limit_mb` field
(optional, u64) overrides the default when present. Exceeding the limit returns
`Err(PluginError::MemoryExceeded { plugin_id, limit_mb })` and emits a WARN log. This is
confirmed existing from S-1.15; PREREQ-D confirms it remains intact after boot wiring via
the `test_BC_2_17_003_memory_limit_enforced_default_64mb` integration test.

### AC-13 — CPU time limit 5s default via epoch interruption; epoch ticker started once at PluginRuntime::new (traces to BC-2.17.004 postconditions; INV-PLUGIN-004)

The epoch background ticker task is started exactly once in `PluginRuntime::new()`, not per call.
Per-call `Store::epoch_deadline` is set proportional to `timeout_seconds` (default 5s, manifest-overridable). Timeout returns `Err(PluginError::Timeout { plugin_id, duration_ms })`. Verified by integration test `test_BC_2_17_004_cpu_timeout_enforced_infinite_loop` using the `infinite_loop.prx` fixture (a WAT module containing an infinite loop compiled to WASM Component).

### AC-14 — Hot-reload: arc-swap atomic registry update; failed reload retains old plugin (story-local API surface confirmation; full BC-2.17.005 promotion deferred to S-1.12-FOLLOWUP per F-LP1-MED-010 closure)

`PluginRuntime::hot_reload(plugin_id, new_bytes)` compiles the new bytes in `spawn_blocking`,
runs WIT validation, and on success atomically swaps the `Arc<LoadedPlugin>` in the registry via
arc-swap. Failed recompilation or WIT validation leaves the old plugin active. INFO or ERROR log
emitted per outcome. In-flight calls holding old `Arc<LoadedPlugin>` complete normally.

### AC-15 — AuthToken zeroize-on-drop (traces to TD-S-PLUGIN-PREREQ-B-002; AD-017 credential safety)

`AuthToken` in `crates/prism-spec-engine/src/auth_provider.rs` implements
`zeroize::Zeroize` on Drop. Either wrap the inner string as `Zeroizing<String>`, or implement
`Drop` manually to overwrite the bytes before deallocation. A doc comment at the `AuthToken`
definition cites AD-017 (credential safety) and explains the zeroize obligation. The `TD-S-PLUGIN-PREREQ-B-002` inline reference is removed when this is implemented.

### AC-16 — MAX_REQUESTS_PER_PIPELINE cumulative cap enforced in executor loop (traces to TD-S-PLUGIN-PREREQ-B-004; BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet) row pipeline_max_requests_exceeded)

`crates/prism-spec-engine/src/pipeline.rs` defines `MAX_REQUESTS_PER_PIPELINE: usize = 10_000`
constant. The `PipelineExecutor` executor loop maintains a cumulative request counter across all
steps. When the counter reaches `MAX_REQUESTS_PER_PIPELINE`, the executor returns
`Err(SpecEngineError::TooManyRequests { total: usize })` and emits `event_type: pipeline_max_requests_exceeded`.
The `TD-S-PLUGIN-PREREQ-B-004` inline reference in pipeline.rs is replaced with the implementation.

`SpecEngineError::TooManyRequests` is the canonical pipeline error type variant per
`crates/prism-spec-engine/src/error.rs:15` (`#[derive(Debug, Error)] #[non_exhaustive] pub enum
SpecEngineError`). Existing variants (`AuthAcquisitionFailed`, `AuthRefreshFailed`,
`HttpRequestFailed`, `JsonPathExtractionFailed`) demonstrate the additive variant pattern. The new
`TooManyRequests` variant is added per POL-1 append-only with canonical error code `E-PIPELINE-001`
(allocated by product-owner in fix-burst-20 stage 1A; see Error Taxonomy Additions section).

### AC-17 — HostState struct marked #[non_exhaustive] before allowed_urls field addition (traces to CLAUDE.md #[non_exhaustive] convention; BC-2.17.007 invariant: no partial registration state)

`crates/prism-spec-engine/src/plugin/loader.rs` defines `pub struct HostState`. Inspection of
the current source confirms `HostState` is `pub` but does NOT carry `#[non_exhaustive]`
(confirmed: `loader.rs` line ~101 shows bare `pub struct HostState`). Adding `allowed_urls:
Vec<String>` (replacing `Option<Vec<String>>` as part of AC-7) without `#[non_exhaustive]`
is a breaking change for any downstream crate constructing `HostState` with struct literal
syntax.

This story MUST add `#[non_exhaustive]` to `HostState` before or in the same commit as the
`allowed_urls` field type change. Per CLAUDE.md convention: all new pub types AND all pub
types modified to add fields must carry `#[non_exhaustive]`.

Verified by code review gate: `grep -n '#\[non_exhaustive\]' crates/prism-spec-engine/src/plugin/loader.rs`
must show the attribute on the line immediately preceding `pub struct HostState`.

The 6 in-tree integration test construction sites in `crates/prism-spec-engine/tests/plugin_tests.rs`
(lines 287, 305, 912, 946, 977, 1018) are enumerated in Match-Site Inventory. Remediation:
introduce a `HostState::test_default()` constructor inside a `#[cfg(any(test, feature = "test-helpers"))]`
block in `loader.rs` (mirroring the `#[cfg(any(test, feature = "test-helpers"))]` gate already used
throughout `crates/prism-spec-engine/src/auth_provider.rs` and `src/lib.rs` per CLAUDE.md Conventions
section). Migrate the 6 test sites to functional-update syntax
`HostState { allowed_urls: vec!["host".to_string()], plugin_id: "...".to_string(), ..HostState::test_default() }`
(or drop the field override entirely for default-deny tests since `allowed_urls: vec![]` is now the default). Both the constructor introduction and all 6 site
migrations must land in the SAME commit as the `#[non_exhaustive]` addition to preserve
compile-green workspace.

Recommended `HostState::test_default()` signature:
```rust
#[cfg(any(test, feature = "test-helpers"))]
impl HostState {
    /// Test-only constructor returning a `HostState` with safe defaults.
    /// Production callers use `make_host_state()` with explicit field values.
    /// Feature-gated: `#[cfg(any(test, feature = "test-helpers"))]` — same gate
    /// as `auth_provider.rs` test helpers per project convention.
    pub fn test_default() -> Self {
        HostState {
            http_client: Arc::new(reqwest::Client::new()),
            config: Arc::new(PluginConfigMap::new()),
            kv_store: Arc::new(PluginKvStore::new()),
            plugin_id: "test-plugin".to_string(),
            allowed_urls: vec![], // empty list = default-deny under AC-7 Vec<String> contract;
                                  // tests that need allowlist enforcement must override with vec!["host"].
            limits: wasmtime::StoreLimits::default(),
        }
    }
}
```

### Obsolete Tests under AC-7 Vec<String> Contract

The field-type change from `Option<Vec<String>>` to `Vec<String>` (AC-17 + AC-7) creates a
semantic break for tests that asserted the pre-AC-7 behavior: "no allowlist configured = allow all hosts."

**Pre-AC-7 semantics:** `allowed_urls: None` meant "no allowlist → permit all URLs."
**Post-AC-7 semantics:** `allowed_urls: vec![]` (empty list) means "empty allowlist → deny all URLs" (default-deny).

**Adjudication: Option A.ii — invert assertion (production-grade default-deny).**

Per CLAUDE.md Canonical Principle Rule 6 (pick production-grade default + document rationale):
the default-deny behavior (`vec![]` blocks all requests) is the correct security posture for
plugin sandboxing. A test that asserts "empty allowlist = allow all" is asserting a security
anti-pattern. Option A.ii inverts these tests to assert the correct secure behavior.

**Affected test:** `test_BC_2_17_002_ec17_007_http_request_no_allowlist_allowed` at `plugin_tests.rs:907`

This test constructs `HostState { allowed_urls: None }` and asserts `assert_ne!(response.status, 403)`,
meaning "no allowlist = not blocked." Under AC-7's Vec<String> contract:
- `allowed_urls` field type is `Vec<String>` — `None` is not a valid value.
- The default is `vec![]` (empty list = default-deny).
- Therefore: a request to ANY url with `allowed_urls: vec![]` MUST return 403.

**Required migration for the implementer:**

1. Rename `test_BC_2_17_002_ec17_007_http_request_no_allowlist_allowed` →
   `test_BC_2_17_002_ec17_007_http_request_empty_allowlist_blocked`
2. Update the `HostState` construction to use `HostState::test_default()` functional-update syntax
   (dropping the `allowed_urls: None` override — the default `vec![]` already provides default-deny)
3. Invert the assertion: `assert_eq!(response.status, 403, "EC-17-007 (post-AC-7): empty allowlist must block all requests (default-deny)")`
4. Update the doc comment from "request allowed to any URL" to "empty allowlist = default-deny = request blocked"
5. Update the section comment at line 899 from "http_request no allowlist allowed" to
   "http_request empty allowlist blocked (default-deny)"

**Rationale:** EC-17-007 in BC-2.17.002 described behavior when "no allowlist is configured."
Under AC-7, an empty allowlist IS a configuration choice — it means deny all. This is not a
behavior removal; it is a security semantic clarification. The inverted test correctly exercises
the AC-7 postcondition: `allowed_urls: vec![]` → all outbound HTTP blocked (403).

No other tests in `plugin_tests.rs` assert "None allowlist = allow all" semantics (confirmed by
external-anchor verification: `grep -n "allowed_urls: None"` returns lines 292, 917, 1023 only;
line 292 is test site 1 / `state_open` which does NOT assert allow-all — it tests invalid URL
handling and expects 400; line 917 is this obsolete test; line 1023 is timeout test which does not
call `host_http_request` directly).

### AC-18 — PRISM_DISABLE_PLUGIN_LOAD env var takes absolute precedence over plugin_dir config (traces to BC-2.22.001 postcondition; ADR-023 §C4 escape valve)

`PRISM_DISABLE_PLUGIN_LOAD=1` is checked in `crates/prism-bin/src/boot.rs` BEFORE any
`plugin_dir` config resolution or filesystem access. Precedence rule:

1. If `PRISM_DISABLE_PLUGIN_LOAD=1` is set: emit a single `tracing::warn!(event_type = "plugin_load_disabled_via_envvar", env_var = "PRISM_DISABLE_PLUGIN_LOAD", ...)` emission (BC-2.16.002 catalog; AC-3); return
   `Ok(0)` immediately. No `plugin_dir` resolution. No filesystem access. No "plugin directory
   not found" event (avoiding confusing double-signal when the operator deliberately disabled
   plugins).
2. If `PRISM_DISABLE_PLUGIN_LOAD` is unset or any value other than the exact string `"1"`:
   proceed with normal `plugin_dir` resolution from `PrismConfig`.

This precedence prevents the operator anti-pattern of: setting `PRISM_DISABLE_PLUGIN_LOAD=1`
without removing `plugin_dir` from `prism.toml`, which would otherwise emit a spurious
`plugin_directory_not_found` error before the disable check fires.

Only the exact string `"1"` disables loading (EC-D-011: values like `"true"`, `"yes"`,
`"TRUE"` are treated as unset).

## Tasks

1. **[prism-spec-engine] Complete `PluginRuntime::load_all_plugins(dir: &Path)`**
   - Scan `dir/*.prx` glob; for each file, read bytes, compute SHA-256 hash
   - Call `Component::from_binary` in `tokio::task::spawn_blocking`
   - Parse manifest fields: `name`, `version`, `format_version`, `allowed_urls`
   - Validate: name non-empty (E-PLUGIN-015); version semver-parseable (E-PLUGIN-016); format_version <= CURRENT_SUPPORTED_VERSION (E-PLUGIN-014); allowed_urls explicitly present — empty list `[]` accepted, absent/null rejected (E-PLUGIN-013); first-failure-returns per BC-2.17.007 EC-17-032
   - Call `validate_wit_interface(component)`; reject with E-PLUGIN-001 on failure
   - On success: construct `HostState { allowed_urls: parsed_hostnames, http_client, kv_store }`; register via arc-swap
   - On manifest rejection: log ERROR, emit structured event, continue to next plugin
   - Return `Ok(n_loaded)` after all files processed

2. **[prism-spec-engine] Close TODO(S-4.08) in `make_host_state()`**
   - Replace `allowed_urls: None` field-default with `allowed_urls: Vec<String>` parameter populated from manifest; field type retired from `Option<Vec<String>>` to `Vec<String>` per AC-17 (None-branch is type-system-impossible)
   - Update `make_host_state()` signature to accept `allowed_urls: Vec<String>`
   - Remove the `TODO(S-4.08)` comment

3. **[prism-spec-engine] Replace host_http_request None-short-circuit with allowlist enforcement**
   - Parse `allowed_urls` from `HostState`
   - Extract host from requested URL via `url::Url::parse`
   - Host-only comparison (`==`) against each entry in allowlist
   - On mismatch: return HTTP 403 to plugin; emit a single `tracing::warn!(event_type = "plugin_http_request_blocked", plugin_id, url, reason = "allowlist_mismatch")` emission (single structured emission per BC-2.16.002 catalog; AC-7)
   - On allowed: forward to reqwest client

4. **[prism-spec-engine] Add reqwest::Client 30-second timeout (TD-B-005) — sibling-site sweep required (TD-VSDD-060)**
   - In `PluginRuntime::new()` or the boot wiring call site, construct client with `.timeout(Duration::from_secs(PLUGIN_HTTP_CLIENT_TIMEOUT_SECS))`
   - Define `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS: u64 = 30` constant in `mod.rs`
   - **Sibling-site sweep (TD-VSDD-060):** `crates/prism-spec-engine/src/plugin/host_functions.rs` contains a per-request `.timeout(Duration::from_secs(10))` override on the RequestBuilder in `host_http_request`. Because `RequestBuilder::timeout()` overrides `Client::builder().timeout()`, leaving this site at 10s means the effective timeout remains 10s even after AC-9's `Client::builder().timeout(30)` lands in boot.rs — making the TD-B-005 closure functionally inert. In the same commit: remove the `.timeout(Duration::from_secs(10))` per-request call OR replace it with `.timeout(Duration::from_secs(PLUGIN_HTTP_CLIENT_TIMEOUT_SECS))` to be explicit. Also update any file-level doc comment in `host_functions.rs` that describes a "10-second per-request timeout" to say "30-second per-request timeout".

5. **[prism-spec-engine] Implement AuthToken zeroize-on-drop (TD-B-002)**
   - Add `zeroize = "1"` to `crates/prism-spec-engine/Cargo.toml` `[dependencies]` section (currently absent — confirmed by inspection; not in `[dev-dependencies]` either; `AuthToken` in `auth_provider.rs` is production code per AC-15 so requires `[dependencies]` not `[dev-dependencies]`)
   - Wrap `AuthToken` inner field as `Zeroizing<String>` or implement manual `Drop`
   - Remove `TD-S-PLUGIN-PREREQ-B-002` inline comment

6. **[prism-spec-engine] Add MAX_REQUESTS_PER_PIPELINE cumulative cap (TD-B-004)**
   - Define `MAX_REQUESTS_PER_PIPELINE: usize = 10_000` constant
   - Add cumulative counter to executor loop; return error on breach
   - Add structured event `event_type: pipeline_max_requests_exceeded`
   - Remove `TD-S-PLUGIN-PREREQ-B-004` inline comment

7. **[prism-spec-engine] Add linker import list #[cfg(test)] assertion (AC-8)**
   - Enumerate all imports registered in `build_linker()`
   - Write `test_linker_imports_match_host_functions` asserting count and names

8. **[prism-spec-engine] Add execute_step eager-token integration tests (TD-B-011/012)**
   - Write `test_execute_step_eager_token_calls_auth_once` using `MockAuthProvider`
   - Assert `mock_auth.calls() == 1` per `execute_step` invocation
   - Assert symmetric behavior with `execute()`
   - Remove TD inline references

9. **[prism-bin] Wire plugin-load step into boot.rs**
   - After storage init step: call `PluginRuntime::load_all_plugins(&config.plugin_dir)`
   - Check `PRISM_DISABLE_PLUGIN_LOAD` env var before calling
   - On disable: emit a single `tracing::warn!(event_type = "plugin_load_disabled_via_envvar", env_var = "PRISM_DISABLE_PLUGIN_LOAD", ...)` emission (single structured emission per BC-2.16.002 catalog; AC-3); continue with MCP server bind (zero plugins registered)
   - On success: emit INFO log with plugin count
   - On error: exit with code 4 (ADR-022 §A internal-error)
   - Renumber subsequent steps in comments: storage = step 7, **plugin-load = step 7.5**, query-engine = step 8, MCP server = step 9 (function `step9_start_mcp_server` retained). Rationale: step 7.5 chosen to avoid cascading renumber across ADR-022 §B canonical step table, boot.rs function names, and BC-2.22.001 §Sequencing Invariant.
   - Inject `reqwest::Client` with 30s timeout into `PluginRuntime::new()`

10. **[prism-bin] Add boot integration tests**
    Implement all 7 named boot integration tests enumerated in §Red Gate Tests
    (`crates/prism-bin/tests/plugin_boot_tests.rs` block) — that section is the
    single source of truth for canonical test names. Do not diverge from those names.

11. **[prism-spec-engine] Add plugin integration tests in crates/prism-spec-engine/tests/**
    Implement all 18 named test functions enumerated in §Red Gate Tests
    (`crates/prism-spec-engine/tests/plugin_integration_tests.rs` block) — that section is the
    single source of truth for canonical test names. Do not diverge from those names.

12. **[.github] Create `.github/PULL_REQUEST_TEMPLATE.md`** (ADR-023 §C4 F-PASS3-MED-001)
    - Three-item sensor-pattern checklist (content defined in Implementation Notes §PR Template)

13. **[tests/fixtures] Commit all 4 `.prx` test fixtures** — `minimal.prx`, `trap_plugin.prx`, `infinite_loop.prx`, `bad_wit.prx` (pre-built binaries) plus WAT sources in `tests/fixtures/src/` — see Fixture Strategy

14. **[prism-spec-engine] Verify Structured Event Catalog wiring** — emit each event from the function-name anchor recorded in BC-2.16.002 catalog row; if implementation discovers ANY new event_type site beyond the 13 already cataloged, amend BC-2.16.002 in the same commit per PG-LP11-001; see §Structured Event Catalog Additions for the canonical 13-row list and BC source-of-truth for Level/Emitter/Fields/Trigger

15. **[tech-debt-register] Mark TD-S-PLUGIN-PREREQ-B-002/004/005/011/012 RESOLVED** (state-manager responsibility in same commit)

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~8,100 |
| BC files (9 BCs × ~1,500) | ~13,500 |
| ADR-023 §C4 (relevant sections) | ~4,000 |
| crates/prism-spec-engine/src/plugin/ source (mod.rs, host_functions.rs) + src/pipeline.rs + src/auth_provider.rs | ~8,000 |
| crates/prism-bin/src/boot.rs | ~3,000 |
| Cargo.toml files (2) | ~1,000 |
| tests/fixtures/src/*.wat (4 WAT source files × ~50 LOC each) | ~800 |
| Test output / error messages during TDD | ~4,000 |
| **Total** | **~42,400** |

This is approximately 16.6% of a 256k-token context window — within the 20-30% limit.
No splitting required.

## File Structure Requirements

### New files

| File | Purpose |
|------|---------|
| `.github/PULL_REQUEST_TEMPLATE.md` | Three-item sensor-pattern checklist (non-crate root-repo deliverable; not under `crates/`) |
| `tests/fixtures/minimal.prx` | Pre-built minimal WASM Component for integration tests |
| `crates/prism-bin/tests/plugin_boot_tests.rs` | Boot-sequence integration tests for plugin-load step |

### Modified files

| File | Modification |
|------|-------------|
| `crates/prism-spec-engine/src/plugin/mod.rs` | Complete `load_all_plugins`; close `TODO(S-4.08)` in `make_host_state()`; add `CURRENT_SUPPORTED_VERSION` constant; add `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS` constant |
| `crates/prism-spec-engine/src/plugin/host_functions.rs` | Replace None-short-circuit in `host_http_request` with allowlist enforcement |
| `crates/prism-spec-engine/src/auth_provider.rs` | Add `Zeroizing<String>` wrapper on `AuthToken`; remove TD inline comment |
| `crates/prism-spec-engine/src/pipeline.rs` | Add `MAX_REQUESTS_PER_PIPELINE` constant; add cumulative counter; add structured event; remove TD inline comments |
| `crates/prism-bin/src/boot.rs` | Insert plugin-load step; renumber subsequent steps |
| `crates/prism-spec-engine/Cargo.toml` | Add 2 crate-local deps: `zeroize = "1"` (for AuthToken zeroize-on-drop per AC-15; currently absent from this file) and `url = "2"` (for host extraction in `host_http_request` per AC-7; currently absent from this file). `sha2 = "0.10"` is already present at line 21; no change required for sha2. |
| `crates/prism-bin/Cargo.toml` | No modification required — `prism-spec-engine` dep already present at line 35 (from S-WAVE5-PREP-01). |

All new `pub` types introduced in this story MUST be marked `#[non_exhaustive]` per project convention.

## Match-Site / Stub Replacement Inventory

The following concrete stub/TODO sites are closed by this story:

| Site | File | Line Reference | Closure |
|------|------|---------------|---------|
| `TODO(S-4.08)` in `make_host_state()` — primary definition | `crates/prism-spec-engine/src/plugin/mod.rs` | ~line 165: `allowed_urls: None` construction | AC-7: replaced with `parsed_hostnames` value (`Vec<String>` per AC-17; None-branch type-system-impossible) |
| `make_host_state()` call site | `crates/prism-spec-engine/src/plugin/mod.rs` | ~line 202: `make_host_state(plugin_id, config)` | AC-7: update to pass `allowed_urls` after signature change |
| `make_host_state()` call site | `crates/prism-spec-engine/src/plugin/mod.rs` | ~line 279: `make_host_state(plugin_id, config)` | AC-7: update to pass `allowed_urls` after signature change |
| `host_http_request` None-short-circuit | `crates/prism-spec-engine/src/plugin/host_functions.rs` | `if allowed_urls.is_none() { /* permit all */ }` | AC-7: replaced with host-only comparison |
| `TD-S-PLUGIN-PREREQ-B-002` inline comment | `crates/prism-spec-engine/src/auth_provider.rs` | `AuthToken` definition | AC-15: removed when zeroize implemented |
| `TD-S-PLUGIN-PREREQ-B-004` inline comment | `crates/prism-spec-engine/src/pipeline.rs` | `MAX_REQUESTS` note | AC-16: removed when cap implemented |
| `TD-S-PLUGIN-PREREQ-B-005` inline comment | `crates/prism-spec-engine/src/pipeline.rs` | reqwest timeout note | AC-9: removed when timeout wired |
| `TD-S-PLUGIN-PREREQ-B-011/012` doc comment | `crates/prism-spec-engine/src/pipeline.rs` | `execute_step` doc comment | Task 8: removed after tests added |
| Per-request `.timeout(Duration::from_secs(10))` override in `host_http_request` builder | `crates/prism-spec-engine/src/plugin/host_functions.rs` | `host_http_request` function — RequestBuilder `.timeout()` call (sibling site to TD-S-PLUGIN-PREREQ-B-005) | AC-9 sibling sweep (TD-VSDD-060): Remove per-request `.timeout(Duration::from_secs(10))` override OR change to `.timeout(Duration::from_secs(PLUGIN_HTTP_CLIENT_TIMEOUT_SECS))` to be explicit; rely on `Client::builder().timeout(30)` from boot.rs as source of truth. File doc-comment near top of `host_functions.rs` that reads "Enforces a 10-second per-request timeout" must be updated to "30-second per-request timeout" in the same commit. |
| `todo!()` in plugin-load step | `crates/prism-bin/src/boot.rs` | step between storage and query-engine | AC-1: replaced with `PluginRuntime::load_all_plugins` call |
| `TODO(S-4.08)` fire-alert dispatch stub | `crates/prism-spec-engine/src/plugin/mod.rs` | ~line 395 | OUT OF SCOPE — S-4.08. Remains open; tracked under separate story. **Implementer action:** when closing the `make_host_state` TODO above, rename this tag to `TODO(S-4.08-fire-alert-dispatch)` so post-merge `rg 'TODO(S-4.08)'` returns zero hits for the closed site. |
| `TODO(S-4.08)` fire-case dispatch stub | `crates/prism-spec-engine/src/plugin/mod.rs` | ~line 419 | OUT OF SCOPE — S-4.08. Remains open; tracked under separate story. **Implementer action:** rename to `TODO(S-4.08-fire-case-dispatch)` (same rationale as above). |
| `TODO(S-4.08)` fire-report dispatch stub | `crates/prism-spec-engine/src/plugin/mod.rs` | ~line 442 | OUT OF SCOPE — S-4.08. Remains open; tracked under separate story. **Implementer action:** rename to `TODO(S-4.08-fire-report-dispatch)` (same rationale as above). |
| `HostState { ... }` struct literal construction — test site 1 | `crates/prism-spec-engine/tests/plugin_tests.rs` | line 287 (`state_open` — full struct literal, 6 fields, `allowed_urls: None`) | AC-17: migrate to functional-update syntax — drop `allowed_urls` field override; `vec![]` is the default-deny default under AC-7 Vec<String> contract: `HostState { plugin_id: "ac5-test-plugin".to_string(), ..HostState::test_default() }` |
| `HostState { ... }` struct literal construction — test site 2 | `crates/prism-spec-engine/tests/plugin_tests.rs` | line 305 (`state_restricted` — full struct literal, 6 fields, `allowed_urls: Some(vec!["allowed-sensor.internal"...])`) | AC-17: migrate to functional-update syntax `HostState { allowed_urls: vec!["allowed-sensor.internal".to_string()], plugin_id: "ac5-test-plugin".to_string(), ..HostState::test_default() }` |
| `HostState { ... }` struct literal construction — test site 3 | `crates/prism-spec-engine/tests/plugin_tests.rs` | line 912 (`state` — full struct literal, 6 fields, `allowed_urls: None`) | AC-17 + obsolete-test adjudication (see §Obsolete Tests under AC-7 Vec<String> Contract below): this test (`test_BC_2_17_002_ec17_007_http_request_no_allowlist_allowed`) asserts semantics that are obsolete under AC-7. Migration: rename to `test_BC_2_17_002_ec17_007_http_request_empty_allowlist_blocked`; drop `allowed_urls` field override (default `vec![]` = default-deny); invert assertion to `assert_eq!(response.status, 403)`. |
| `HostState { ... }` struct literal construction — test site 4 | `crates/prism-spec-engine/tests/plugin_tests.rs` | line 946 (`state` — full struct literal, 6 fields, `allowed_urls: Some(vec!["example.com"...])`) | AC-17: migrate to functional-update syntax `HostState { plugin_id: "test-plugin".to_string(), allowed_urls: vec!["example.com".to_string()], ..HostState::test_default() }` |
| `HostState { ... }` struct literal construction — test site 5 | `crates/prism-spec-engine/tests/plugin_tests.rs` | line 977 (`state` — full struct literal, 6 fields, `allowed_urls: Some(vec!["example.com"...])`) | AC-17: migrate to functional-update syntax `HostState { plugin_id: "test-plugin".to_string(), allowed_urls: vec!["example.com".to_string()], ..HostState::test_default() }` |
| `HostState { ... }` struct literal construction — test site 6 | `crates/prism-spec-engine/tests/plugin_tests.rs` | line 1018 (`host_state` — full struct literal, 6 fields, `allowed_urls: None`, `plugin_id: "timeout-test"`) | AC-17: migrate to functional-update syntax — drop `allowed_urls` field override; `vec![]` is the default under AC-7 Vec<String> contract: `HostState { plugin_id: "timeout-test".to_string(), ..HostState::test_default() }` (allowed_urls: vec![] is the default = default-deny under AC-7 Vec<String> contract) |

## Red Gate Tests

Per BC-5.39.001, all production function bodies under this story use `todo!()` until the Red
Gate is passed. The following named tests must fail (RED) before implementation and pass (GREEN)
after:

### prism-bin tests (`crates/prism-bin/tests/plugin_boot_tests.rs`)

```
test_BC_2_22_001_boot_step_plugin_load_placement
test_BC_2_22_001_plugin_load_failure_exits_code_4
test_BC_2_22_001_plugin_load_disabled_env
test_BC_2_22_001_disable_env_takes_precedence_over_plugin_dir_config
test_VP_PLUGIN_004_unsigned_plugin_boot_warn_audit
test_VP_PLUGIN_007_plugin_load_rejected_no_allowlist
test_VP_PLUGIN_007_plugin_load_rejected_format_version_exceeded
```

### prism-spec-engine tests (`crates/prism-spec-engine/tests/plugin_integration_tests.rs`)

```
test_BC_2_17_001_plugin_panic_isolation
test_BC_2_17_002_wasi_not_linked_trap_on_fs_call
test_BC_2_17_002_allowlist_enforcement_blocks_non_allowlisted_url
test_BC_2_17_002_allowlist_enforcement_allows_listed_url
test_BC_2_17_003_memory_limit_enforced_default_64mb
test_BC_2_17_004_cpu_timeout_enforced_infinite_loop
test_hot_reload_atomic_swap_success
test_hot_reload_failed_recompile_retains_old
test_BC_2_17_006_wit_validation_rejects_missing_export
test_BC_2_17_006_duplicate_plugin_id_first_wins
test_BC_2_17_007_manifest_format_version_exceeded_rejected
test_BC_2_17_007_manifest_missing_allowed_urls_rejected
test_BC_2_17_007_manifest_name_empty_rejected
test_BC_2_17_007_manifest_version_malformed_rejected
test_BC_2_17_002_linker_imports_match_host_functions
test_TD_S_PLUGIN_PREREQ_B_011_execute_step_eager_token_calls_auth_once
test_BC_2_16_002_pipeline_max_requests_exceeded
test_TD_S_PLUGIN_PREREQ_B_002_authtoken_zeroize_on_drop
```

Test naming convention: `test_BC_<bc_id>_<descriptor>` for BC-anchored tests;
`test_VP_PLUGIN_<NNN>_<descriptor>` for VP-anchored tests;
`test_TD_<td_id>_<descriptor>` for TD-absorbed tests with no dedicated BC/VP anchor.

Red Gate density target: >= 15 failing tests before first implementation commit.

## Structured Event Catalog Additions

Per BC-2.16.002 §Postconditions (Canonical Structured Event Catalog) and PG-LP11-001: every new
`tracing::*!(event_type=…)` site introduced by this story is enumerated as a row in the
BC-2.16.002 Canonical Structured Event Catalog. The 13 events below have already been
added to BC-2.16.002 (7 in fix-burst-8; 2 in fix-burst-17; 3 in fix-burst-impl-1; 1 in fix-burst-impl-3); the implementer's responsibility is
to ensure each emission site is wired correctly during S-PLUGIN-PREREQ-D implementation,
with the BC-2.16.002 row as the source of truth for Level / Emitter / Fields / Trigger.

| Event Type | Level | Emitter | Fields | Trigger |
|-----------|-------|---------|--------|---------|
| `plugin_load_unsigned` | WARN | `PluginRuntime::load_all_plugins` | `plugin_path`, `plugin_hash` | Each successfully loaded plugin (v1.0 unsigned); audit-channel routing encoded by `event_type` field per ADR-023 §C4 |
| `plugin_load_disabled_via_envvar` | WARN | `boot::plugin_load_step` | `env_var: "PRISM_DISABLE_PLUGIN_LOAD"` | `PRISM_DISABLE_PLUGIN_LOAD=1` detected at boot before plugin-load step; emitted before skip to preserve DI-004 audit completeness |
| `plugin_load_failed_manifest_no_allowed_urls` | ERROR | `PluginRuntime::load_plugin` | `plugin_path`, `error: E-PLUGIN-013` | Plugin manifest missing required `allowed_urls` field; plugin rejected; remaining plugins continue loading |
| `plugin_load_failed_format_version_exceeded` | ERROR | `PluginRuntime::load_plugin` | `plugin_path`, `format_version`, `max_supported` | Plugin `format_version` exceeds `CURRENT_SUPPORTED_VERSION`; plugin rejected; remaining plugins continue loading |
| `plugin_load_failed_wit_invalid` | ERROR | `PluginRuntime::load_plugin` | `plugin_path`, `missing_export`, `error: E-PLUGIN-001` | WIT validation failure — plugin component is missing one or more required WIT exports; plugin rejected; remaining plugins continue loading |
| `plugin_http_request_blocked` | WARN | `host_http_request` | `plugin_id`, `url`, `reason: allowlist_mismatch` | Plugin attempted an outbound HTTP request to a URL not present in its manifest `allowed_urls` list; request blocked; plugin execution continues |
| `pipeline_max_requests_exceeded` | ERROR | `PipelineExecutor` executor loop | `sensor_id`, `total_requests`, `max: MAX_REQUESTS_PER_PIPELINE` | Cumulative HTTP request count across all pipeline steps reaches `MAX_REQUESTS_PER_PIPELINE` (10,000); pipeline aborts |
| `plugin_load_failed_manifest_name_missing` | ERROR | `PluginRuntime::load_plugin` | `plugin_path`, `error: E-PLUGIN-015` | Manifest `name` field absent or empty string; plugin rejected; remaining plugins continue (n-1 survivor) |
| `plugin_load_failed_manifest_version_malformed` | ERROR | `PluginRuntime::load_plugin` | `plugin_path`, `version_value`, `error: E-PLUGIN-016` | Manifest `version` field not valid semver; plugin rejected; remaining plugins continue (n-1 survivor) |
| `plugin_load_failed_manifest_parse_error` | ERROR | `PluginRuntime::load_all_plugins` | `plugin_path`, `error: E-PLUGIN-017`, `detail` | Companion `.manifest.toml` is present but fails TOML parse; plugin rejected (n-1 survivor) |
| `plugin_load_failed_manifest_not_found` | ERROR | `PluginRuntime::load_all_plugins` | `plugin_path`, `expected_manifest_path`, `error: E-PLUGIN-018` | Plugin `.prx` found but no companion `.manifest.toml` exists; plugin rejected (n-1 survivor) |
| `plugin_load_failed_format_version_missing` | ERROR | `PluginRuntime::load_all_plugins` | `plugin_path`, `supported`, `error: E-PLUGIN-019` | Manifest `format_version` field absent entirely; plugin rejected (n-1 survivor); distinct from exceeded (014) |
| `plugin_log_level_unrecognized` | WARN | `register_host_functions` → host::log callback (Component Model) | `plugin_id`, `received_name` (the unrecognized enum string) | Plugin passed an unrecognized log-level enum discriminant through the Component Model host::log callback; observability — surfaces plugin schema-violation attempts; first introduced in fix-burst-impl-3 (BC-2.16.002 v1.17 row 32) |

The BC-2.16.002 catalog row is the authoritative source for each event's field schema,
audit role, and recurrence policy. PG-LP11-001 requires that any new `event_type` sites added
during S-PLUGIN-PREREQ-D implementation that are not listed above MUST be enumerated in
BC-2.16.002 as a BC amendment in the same commit as the implementation, per SOP codified in
`.factory/cycles/wave-4-operations/lessons.md` Lesson 1.

## Fixture Strategy

**Decision: COMMIT all 4 `.prx` test fixtures as pre-built binary artifacts.**

Rationale: avoiding a `just plugin-build` or `cargo component build` bootstrap dependency in
the test suite is critical. The integration test suite must run with `cargo nextest run` and
`just iter prism-spec-engine` without any WASM toolchain pre-step. Pre-built fixtures enable
cold CI runs and developer onboarding without the `wasm32-wasip2` target or `cargo-component`
installed. The fixture is deterministic (locked to a specific plugin source); updates require an
explicit binary commit. The fixture commit will be gated by a CI check that asserts the `.prx`
file exists and is a valid WASM Component header (4-byte magic check).

Multiple fixtures required:

| Fixture | Location | Purpose |
|---------|----------|---------|
| `tests/fixtures/minimal.prx` | repo root `tests/fixtures/` | Minimal valid infusion plugin; used by AC-4, AC-5, AC-6 |
| `tests/fixtures/trap_plugin.prx` | repo root `tests/fixtures/` | WAT-compiled module with `unreachable`; used by AC-10 |
| `tests/fixtures/infinite_loop.prx` | repo root `tests/fixtures/` | WAT-compiled module with `loop {}`; used by AC-13 |
| `tests/fixtures/bad_wit.prx` | repo root `tests/fixtures/` | Component missing required WIT exports; used by AC-6 |

Fixture authorship (WAT source → `.prx` binary compilation) is implementer responsibility. The
WAT sources are committed alongside the binaries in `tests/fixtures/src/` for auditability.

## wasmtime Version Pin

`crates/prism-spec-engine/Cargo.toml` pins:
```toml
# wasmtime 44 resolves multiple RUSTSEC advisories in the RUSTSEC-2024-0438 through
# RUSTSEC-2026-0096 range — run `cargo audit` for the current count and advisory list.
wasmtime = { version = "44", features = ["component-model"] }
```

**Decision: retain `wasmtime = "44"` (current pin).** This version is already pinned with an
explicit security rationale comment (RUSTSEC advisory resolution). The WIT specification used by
Prism plugins (manifest: `name`, `version`, `format_version`, `allowed_urls`; dispatch:
`enrich_single`, `enrich_batch`, `fire_alert`) is supported by the Component Model feature in
wasmtime 44. No version change is required for PREREQ-D scope. If a security advisory against
wasmtime 44 is identified during implementation, bump to the lowest version resolving it and
update the RUSTSEC comment accordingly. No version bump for latency optimization or new API
surface is permitted without architect review.

## Implementation Notes

### Arc-DI Plumbing

Per ADR-022 (Arc-DI), `PluginRuntime` must be constructed with all dependencies via constructor
injection (`PluginRuntime::new(engine, linker, http_client, kv_store)`). The instance is wrapped
in `Arc<PluginRuntime>` and threaded through the boot chassis. No global statics. The boot step
receives `Arc<PluginRuntime>` and passes it to the query-engine and MCP server steps that need
plugin dispatch.

**TD-B-005 closure:** The constant `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS: u64 = 30` is defined in
`crates/prism-spec-engine/src/plugin/mod.rs`. The `reqwest::Client` constructor is in
`crates/prism-bin/src/boot.rs` (the boot step), which reads this constant. The client is passed
to `PluginRuntime::new()` as an owned value and stored as `Arc<reqwest::Client>` on the runtime.
This makes the construction site unambiguous: `prism-bin::boot` constructs, `prism-spec-engine`
distributes via Arc::clone in `make_host_state()`.

### Forbidden Dependencies

The `prism-bin` crate MUST NOT gain a dependency on `prism-query` beyond what S-WAVE5-PREP-01
already established. `prism-spec-engine` MUST NOT gain a dependency on `prism-storage` or
`prism-audit` — audit entries flow through the `AuditEmitter` trait injected at boot, not via
direct crate dependency. If the implementer finds a direct dependency is needed, escalate to
architect before adding.

### PR Template Content

`.github/PULL_REQUEST_TEMPLATE.md` must contain exactly this three-item sensor-pattern checklist
(ADR-023 §C4 F-PASS3-MED-001):

```markdown
## Sensor Pattern Checklist

Before merging any PR that touches sensor fetch, authentication, or data transformation:

- [ ] New sensor behaviour is expressed as TOML spec (`*.sensor.toml`) — not as a Rust module
- [ ] Outbound HTTP calls flow through `host_http_request` (not direct `reqwest` in plugin source)
- [ ] Plugin `allowed_urls` manifest field is populated with the minimum required hostname set
```

### Error Taxonomy Additions

Five new error codes are introduced. Four are `PluginError` variants added to
`crates/prism-core/src/error.rs` (the canonical `PluginError` enum at lines 984-1034);
prism-spec-engine consumers import via `use prism_core::PluginError` (existing pattern;
e.g., `crates/prism-spec-engine/src/plugin/mod.rs:16`). One is a `SpecEngineError` variant
added to `crates/prism-spec-engine/src/error.rs` (the canonical `SpecEngineError` enum at line 15).

| Code | Name | Message Template |
|------|------|-----------------|
| `E-PLUGIN-013` | `PluginError::MissingAllowedUrls` | ``"Plugin manifest at '{path}' missing required field 'allowed_urls'; field must be an explicit list (use `allowed_urls = []` for no URLs)"`` |
| `E-PLUGIN-014` | `PluginError::FormatVersionExceeded` | `"Plugin manifest at '{path}' format_version {actual} exceeds maximum supported version {supported}"` |
| `E-PLUGIN-015` | `PluginError::ManifestNameMissing` | `"Plugin manifest at '{path}' missing or empty required field 'name'"` |
| `E-PLUGIN-016` | `PluginError::ManifestVersionMalformed` | `"Plugin manifest at '{path}' field 'version' is not a valid semver string: '{value}'"` |
| `E-PIPELINE-001` | `SpecEngineError::TooManyRequests { total: usize }` | `"Pipeline executor reached MAX_REQUESTS_PER_PIPELINE cap of 10_000 ({total} requests attempted); aborting pipeline execution"` |

`PluginError` MUST be marked `#[non_exhaustive]` in the same commit as the new variant additions (E-PLUGIN-013/014/015/016). This aligns with `PrismError` at `crates/prism-core/src/error.rs:15-17` (already `#[non_exhaustive]`) and the 30+ pub-API types currently enforced via `tests/external/perimeter-violation/`. The conditional "if PluginError is a non-exhaustive enum" framing from prior story versions is an MVP-hedge under CLAUDE.md Canonical Principle Rule 1 and must not be retained.
E-PLUGIN-015 and E-PLUGIN-016 correspond to AC-5 validation steps for `name` and `version`
fields respectively (BC-2.17.007 postconditions 1 and 2). `E-PIPELINE-001` corresponds to AC-16:
the `SpecEngineError::TooManyRequests` variant is emitted by `PipelineExecutor::execute` when the
cumulative HTTP request counter across all pipeline steps reaches the 10,000-request hard cap.
Pipeline aborts immediately; partial results are discarded. Non-retryable: the cap is a hard invariant.
Structured event `pipeline_max_requests_exceeded` is emitted per BC-2.16.002 v1.16 catalog row.
Traces to BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet) row pipeline_max_requests_exceeded (anchored by AC-16 of S-PLUGIN-PREREQ-D).

### Credential Safety (AD-017)

`AuthToken` contains bearer token bytes. After the zeroize fix (AC-15), the token is overwritten
on drop. The `AuthToken` type must NEVER appear in tracing log fields, structured event catalog
entries, or audit log entries. The `Debug` impl for `AuthToken` must redact the value: the existing
implementation at `crates/prism-spec-engine/src/auth_provider.rs:68` uses `AuthToken(<redacted>)`
(angle-bracket form, lowercase). Per AD-017, the redaction discipline applies regardless of exact
form; confirm the existing `Debug` impl is intact during AC-15 zeroize work.

### #[non_exhaustive] Requirements

The following types MUST be marked `#[non_exhaustive]` per project convention (CLAUDE.md):

- `PluginError` (enum-level, in `crates/prism-core/src/error.rs`) — MUST be marked `#[non_exhaustive]` in the same commit as the new variant additions (E-PLUGIN-013/014/015/016). Sibling `PrismError` at `error.rs:15-17` already carries `#[non_exhaustive]`; `PluginError` at `error.rs:983` currently lacks it. This is an unconditional requirement — `PluginError` is a public enum in `prism-core` and the established convention applies.
- `PluginLoadResult` (if introduced as a return type)
- Any new manifest struct types (e.g., `PluginManifest`)

### BC-2.22.001 Sequencing Invariant Comment

In `crates/prism-bin/src/boot.rs`, the plugin-load step must be annotated with:
```rust
// BC-2.22.001: plugin-load step — positioned after step 7 (storage init) and before
// query-engine init per ADR-023 §C4 + ADR-022 §B sequencing invariant.
// PRISM_DISABLE_PLUGIN_LOAD=1 skips this step (emergency escape valve).
```

## Previous Story Intelligence

Previous stories in this epic (PREREQ-A/B/C) established the following lessons that apply here:

1. **PG-LP11-001: New structured event type sites MUST amend BC-2.16.002 in the same burst.**
   This story introduced 13 new event types (see §Structured Event Catalog Additions). All 13 rows
   exist in BC-2.16.002 (Path B Canonical Structured Event Catalog: 7 rows added in
   fix-burst-8; 2 for E-PLUGIN-015/016 in fix-burst-17; 3 for E-PLUGIN-017/018/019 in fix-burst-impl-1;
   1 for plugin_log_level_unrecognized in fix-burst-impl-3).
   The implementer's responsibility is to wire each emission site to match the BC row metadata
   (Level/Emitter/Fields/Trigger). If implementation discovers ANY new event_type site beyond the
   13 cataloged, amend BC-2.16.002 in the same commit per PG-LP11-001 invariant.

2. **Volatile pin discipline:** Do not add `# volatile pin` comments to wasmtime or any other
   dep without explicit architect approval. The wasmtime 44 pin has an explicit RUSTSEC rationale
   comment — that pattern is canonical; do not deviate.

3. **#[non_exhaustive] on all pub TOML-deserialized types.** The `PluginManifest` struct (if
   introduced) and any new error variants must carry `#[non_exhaustive]`.

4. **Load-bearing assertions in tests.** All `assert_eq!` and `assert!(result.is_err())`
   calls must verify the actual field values (error code, plugin_id, etc.), not merely
   `result.is_ok()` / `result.is_err()`. Per TD-W2-FIXK-002: BC-named tests must assert
   postcondition content, not just success/failure shape.

5. **execute_step eager-token symmetry is already implemented** (PREREQ-B fix-burst-6 Option A).
   The integration tests added by this story (TD-B-011/012) confirm the wiring is correct
   end-to-end; do not re-implement the logic.

6. **Adversarial review path:** Per TD-VSDD-094, adversarial-review reports for PREREQ-D must
   be written to `.factory/cycles/wave-4-operations/adversarial-reviews/` (canonical path).
   Not under `code-delivery/`.

## Architecture Compliance Rules

Extracted from architecture documents and ADRs:

| Rule | Source | Enforcement |
|------|--------|------------|
| WASI interfaces MUST NOT be linked into plugin instances | BC-2.17.002 INV-PLUGIN-002 | VP-040 Kani proof + linker assertion (AC-8) |
| Plugin compilation MUST run in `spawn_blocking` | BC-2.17.005 §Invariants | Code review; tokio lint |
| `wasmtime::Store` created fresh per plugin call | BC-2.17.001 invariant | Code review |
| Epoch ticker started once at PluginRuntime::new() | BC-2.17.004 invariant | Unit test assertion |
| Plugin-load step after storage init, before query-engine | BC-2.22.001 sequencing | Integration test (AC-1) |
| No credentials in log fields | AD-017 | AuthToken Debug redaction |
| All new pub types #[non_exhaustive] | CLAUDE.md convention | Code review |
| No println! in production code | CLAUDE.md convention | clippy lint |
| Arc-DI for all constructor injection | ADR-022 | Code review |
| prism-spec-engine MUST NOT depend on prism-storage or prism-audit | Forbidden dependency | cargo deny / code review |

## BC Amendments Landed

The following Behavioral Contract amendments landed alongside this story's pre-merge polish (fix-burst-29 + fix-burst-30):

### BC-2.17.002 v1.5 → v1.6 → v1.7 — EC-17-007 default-deny alignment

**v1.6 (fix-burst-29 stage-1, 2026-05-14):** EC-17-007 rewritten from pre-AC-7 allow-all semantics ("Request allowed to any URL (open by default)") to post-AC-7 default-deny semantics. AC-7 + AC-17 establish that `allowed_urls: Vec<String>` makes "no allowlist configured" representationally impossible — `vec![]` means "empty allowlist → deny all URLs" (default-deny).

**v1.7 (fix-burst-30 stage-1, 2026-05-14, F-LP32-CRIT-001 closure):** EC-17-007 amended to remove fabricated `PluginError::AllowlistRejected` variant reference (the variant does not exist in `crates/prism-core/src/error.rs` PluginError enum, in error-taxonomy.md, or in AC-7's prescription). EC-17-007 now uses existing `E-PLUGIN-005 SandboxViolation` semantics aligned with AC-7's prescribed "HTTP 403 returned" behavior and existing `host_http_request` synchronous-return code path. Zero new error variant introduced.

**Routing per CLAUDE.md:** product-owner owns BC content amendments; story-writer owns this §BC Amendments Landed retrospective section in story body.

## Library and Framework Requirements

| Library | Version | Purpose | Pin Note |
|---------|---------|---------|----------|
| `wasmtime` | `44` (exact) | WASM Component Model runtime | RUSTSEC rationale in `crates/prism-spec-engine/Cargo.toml` |
| `zeroize` | `"1"` | AuthToken zeroing on drop (TD-B-002) | Accept any 1.x; ADD to `crates/prism-spec-engine/Cargo.toml` — currently absent from that file |
| `sha2` | `"0.10"` | SHA-256 for plugin_hash audit field | Already pinned at `sha2 = "0.10"` in `crates/prism-spec-engine/Cargo.toml` (crate-local pin, line 21); no Cargo.toml change required for sha2. |
| `url` | `"2"` | URL parsing for allowlist enforcement | NOT currently in `crates/prism-spec-engine/Cargo.toml`; ADD `url = "2"` with comment `# Used by PluginRuntime::host_http_request for allowlist host extraction`. |
| `reqwest` | `"0.12"` | HTTP client with 30s timeout | Already pinned at `reqwest = { version = "0.12", ... }` in `crates/prism-spec-engine/Cargo.toml` (crate-local pin, line 34); builder pattern mandatory; no bare `reqwest::Client::new()`. |
| `arc-swap` | `"1"` | Lock-free registry updates | Already pinned at `arc-swap = "1"` in `crates/prism-spec-engine/Cargo.toml` (crate-local pin, line 20); no Cargo.toml change required. |
| `tokio` | `"1"` | `spawn_blocking` for WASM compilation | Already pinned at `tokio = { version = "1", ... }` in `crates/prism-spec-engine/Cargo.toml` (crate-local pin, line 26); no Cargo.toml change required. |

Do NOT invent version numbers. All versions above are confirmed from `crates/prism-spec-engine/Cargo.toml`. Note: the workspace root `Cargo.toml` has no `[workspace.dependencies]` table — all dep versions are crate-local pins in this project. The `zeroize` dep is currently absent from `crates/prism-spec-engine/Cargo.toml` (confirmed by inspection; not in `[dependencies]` lines 12-36 nor in `[dev-dependencies]` lines 38-54); add it at `"1"` with an explanatory comment.

## References

- [ADR-023 §C4](../specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md) — PRIMARY SCOPE SOURCE (PLUGIN-PREREQ-D)
- [BC-2.16.002](../specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md) — Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation
- [BC-2.17.001](../specs/behavioral-contracts/BC-2.17.001-plugin-panic-isolation.md) — Plugin Panic Isolation — Crashed Plugin Does Not Terminate Host Process
- [BC-2.17.002](../specs/behavioral-contracts/BC-2.17.002-plugin-sandbox-filesystem.md) — Plugin Sandbox — No Direct Filesystem or Network Access
- [BC-2.17.003](../specs/behavioral-contracts/BC-2.17.003-plugin-memory-limit.md) — Plugin Sandbox — Memory Limit Enforced Per Plugin Instance (default 64MB)
- [BC-2.17.004](../specs/behavioral-contracts/BC-2.17.004-plugin-cpu-time-limit.md) — Plugin Sandbox — CPU Time Limit Enforced via Epoch Interruption (default 5s)
- [BC-2.17.005](../specs/behavioral-contracts/BC-2.17.005-plugin-hot-reload-atomic-swap.md) — Plugin Hot Reload — Atomic Module Swap, In-Flight Calls Complete Against Old Version
- [BC-2.17.006](../specs/behavioral-contracts/BC-2.17.006-plugin-wit-validation.md) — WIT Interface Validation Before Plugin Registration
- [BC-2.17.007](../specs/behavioral-contracts/BC-2.17.007-plugin-manifest-schema-validation.md) — Plugin Manifest Schema Validation Before WIT Validation (NEW — landed wave-4-fix-burst-F-LP1-HIGH-004)
- [BC-2.22.001](../specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md) — Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate
- [VP-INDEX §VP-149/VP-PLUGIN-004](../specs/verification-properties/VP-INDEX.md) — Boot warning on unsigned plugin load
- [VP-INDEX §VP-152/VP-PLUGIN-007](../specs/verification-properties/VP-INDEX.md) — Allowlist explicit Vec<String> after PREREQ-D (default-deny semantics)
- [S-PLUGIN-PREREQ-B](S-PLUGIN-PREREQ-B-real-pipeline-executor.md) — Real PipelineExecutor (carry-forward TDs)
- [S-PLUGIN-PREREQ-C](S-PLUGIN-PREREQ-C-toml-grammar-extensions-plus-pub-api-hardening.md) — TOML Grammar Extensions
- [tech-debt-register §TD-S-PLUGIN-PREREQ-B-002/004/005/011/012](../tech-debt-register.md)
- [forward-task-map §TIER 2](../cycles/wave-4-operations/forward-task-map.md) — PREREQ-D scope and blocks

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.36 | S-PLUGIN-PREREQ-D-fix-burst-impl-6 | 2026-05-15 | story-writer | F-PASS6-LOW-003 closure + PG-IMPL-LP6-003 frontmatter sync. LOW-003: story v1.34 changelog Burst column corrected from "S-PLUGIN-PREREQ-D-fix-burst-impl-3" to "S-PLUGIN-PREREQ-D-fix-burst-impl-4" (the BC row 32 addition was D-552/fix-burst-impl-3; the story-body sweep 12→13 + 13th row append was D-554/fix-burst-impl-4 per actual commit history; body prose narrative is correct, only Burst attribution was inverted). Frontmatter `updated: "2026-05-14"` → `"2026-05-15"` syncing with v1.35 changelog row date (PG-IMPL-LP6-003 frontmatter-modified discipline). v1.35→v1.36. |
| 1.35 | S-PLUGIN-PREREQ-D-fix-burst-impl-5 | 2026-05-15 | implementer | F-PASS5-LOW-001/002 closure (cosmetic spec alignment). LOW-001 (STORY-INDEX attribution): F-PASS4-MED-001 story-body sibling-sweep was performed by fix-burst-impl-4 (D-554, b788d53c), not fix-burst-impl-3; fix-burst-impl-3 (D-552, d8f51552) added BC-2.16.002 v1.17 row 32. STORY-INDEX row annotation corrected to attribute each burst. LOW-002 (catalog row field asymmetry): `plugin_log_level_unrecognized` §Structured Event Catalog Additions row had `event_type` in the Fields column; removed per Option A (event_type is the row key, not a payload field; 12 sibling rows omit it; BC-2.16.002 v1.17 row 32 source-of-truth lists only `plugin_id, received_name`). v1.34→v1.35. |
| 1.34 | S-PLUGIN-PREREQ-D-fix-burst-impl-4 | 2026-05-14 | story-writer | F-PASS4-MED-001 closure: §Structured Event Catalog Additions count drift 12→13 (sibling-sweep TD-VSDD-060). fix-burst-impl-3 added BC-2.16.002 v1.17 row 32 (`plugin_log_level_unrecognized`) but story body remained at "12 events". Updated count 12→13 at 4 active-body sites: BC table intro (line 260 primary-coverage cell), Task 14 (12→13 in "beyond the N already cataloged" + "canonical N-row list"), §Structured Event Catalog Additions preamble ("The 12 events" → "The 13 events"; added fix-burst-impl-3 attribution), §Previous Story Intelligence item 1 (12→13 in narrative + per-burst breakdown). Appended 13th catalog row for `plugin_log_level_unrecognized` (WARN, register_host_functions → host::log callback, fields: plugin_id + received_name + event_type, observability — surface plugin schema-violation attempts) mirroring BC-2.16.002 v1.17 row 32 schema exactly. Sibling-sweep grep `"12 event\|12 new event\|12 structured\|12 catalog\|12 row\|12 already\|12-row"` — 0 active-body hits remaining; 1 historical hit in v1.33 changelog (correct). Traces to: F-PASS4-MED-001, BC-2.16.002 v1.17. v1.33→v1.34. |
| 1.33 | S-PLUGIN-PREREQ-D-fix-burst-impl-2 | 2026-05-14 | implementer | F-PASS2-MED-002 closure: 12 active-body occurrences of `BC-2.16.002 v1.12` version pin replaced with `BC-2.16.002 v1.16` per TD-VSDD-091 anti-volatile-pin (de-pinned to version-neutral anchors where appropriate; updated to current v1.16 in source-of-truth table citations). F-PASS2-MED-003 closure: §Structured Event Catalog Additions updated — count 9→12; added 3 rows for `plugin_load_failed_manifest_parse_error` (E-PLUGIN-017), `plugin_load_failed_manifest_not_found` (E-PLUGIN-018), `plugin_load_failed_format_version_missing` (E-PLUGIN-019) added in fix-burst-impl-1. Preamble updated to reflect 12 events from 3 fix-bursts. Task 14 count 9→12. Previous Story Intelligence item 1 count 9→12, version pin de-pinned. BC table intro count 9→12. F-PASS2-LOW-001 closure: story frontmatter uses `updated:` not `modified:` — this is intentional schema distinction (stories use `updated:`, BCs use `modified:`) confirmed by checking sibling stories S-PLUGIN-PREREQ-C; `updated:` field left unchanged; distinction codified in this changelog row. v1.32→v1.33. |
| 1.32 | fix-burst-32 | 2026-05-14 | story-writer | F-LP34-HIGH-001: §Changelog table row-delimiter corruption fixed — lines 1055 (4 rows: v1.22+v1.21+v1.20+v1.19) and 1056 (3 rows: v1.18+v1.17+v1.16) had multiple changelog rows concatenated without inter-row newlines; each row now occupies its own physical line per markdown-table integrity convention. F-LP34-MED-001: 3rd fix-burst-closure-introduced drift instance — fix-burst-31 introduced `§Canonical Structured Event Catalog` as anchor; pass-34 found this is a bold-labeled bullet within BC-2.16.002 §Postconditions (line 74), not a `##` section heading. 4 active-body sites (lines 260, 300, 466, 918) replaced with `§Postconditions (Canonical Structured Event Catalog bullet)` anchor making the actual ## ancestry visible. F-LP34-OBS-001 (Codification #14 refinement re: bold-labeled bullets) and F-LP34-OBS-002 (markdown-table integrity sweep) routed cycle-close. F-LP34-LOW-001 (VP-INDEX "not-None" Option-semantics drift) handled by state-manager in Burst S closure. Closure justification: production-grade default; zero scope expansion (story-only edits). |
| 1.31 | fix-burst-31 | 2026-05-14 | story-writer | F-LP33-MED-001: AC-9 trace header line 373 stale BC-2.17.002 v1.6 pin updated to v1.7 (8th instance of version-pin sibling-prose drift in cascade; pass-33 surfacing). F-LP33-MED-002: E-PLUGIN-013 message template aligned to canonical backtick form: story line 906 (single-quoted → double-backtick-fenced backticks) + story line 323 (no-delim → backtick-fenced inline). F-LP33-LOW-001: "catalog discipline" 2 active-body sites (lines 300–301, 357) replaced with resolvable anchors per Codification #14 (phantom-section-anchor sweep): line 300–301 uses precise form `§Canonical Structured Event Catalog (row plugin_load_unsigned Trigger cell)` (first occurrence, anchor established); line 357 uses lighter form `catalog routing convention` (back-reference in AC-7, anchor already established upstream). Sibling-site sweep results: (1) `BC-2.17.002 v1.[0-6]` active body — ZERO stale pins (lines 418/999 are legitimate historical references within §BC Amendments Landed and versioned narrative prose); (2) `catalog discipline` active body — ZERO hits (changelog rows 1.22/1.13/1.11 are historical); (3) `use allowed_urls = []` without backticks active body — ZERO hits (lines 322, 905 both use canonical backtick form). Closure justification: production-grade default (no LOW deferral); zero scope expansion (story-only edits). |
| 1.30 | fix-burst-30 stage-1 | 2026-05-14 | story-writer | F-LP32-MED-001: AC-9 closure note line 419 stale BC-2.17.002 v1.5 pin updated to v1.7 (fix-burst-30 phantom-variant removal closure). F-LP32-MED-002: §Changelog rows 1.27/1.28/1.29 schema-corruption fix — added Burst column values (fix-burst-27/28/29 stage-1) restoring 5-cell schema parity with rows 1.26+. F-LP32-OBS-001: §BC Amendments In-Scope retrospectively reframed past-tense as §BC Amendments Landed documenting v1.6 (fix-burst-29) and v1.7 (fix-burst-30) amendments. Product-owner BC-2.17.002 v1.6→v1.7 EC-17-007 phantom-variant removal dispatched in parallel. fix-burst-30 stage-1 story-writer scope. |
| 1.29 | fix-burst-29 stage-1 | 2026-05-14 | story-writer | F-LP31-HIGH-001 (POL-7 axis EXTENSION): §Error Taxonomy Additions table E-PLUGIN-013 + E-PLUGIN-014 message templates aligned to canonical (matching AC-5 body + error-taxonomy.md); E-PLUGIN-015/016 unchanged (already verbatim). F-LP31-HIGH-002 STORY SITE: new §BC Amendments In-Scope section added directing product-owner to amend BC-2.17.002 v1.5→v1.6 EC-17-007 to align with AC-7 default-deny semantics per Source-of-Truth Precedence Rule 1 (security-semantic cross-spec drift mitigation). F-LP31-MED-001: AC-15 §Credential Safety AuthToken Debug example aligned to existing code at auth_provider.rs:68 ('AuthToken(<redacted>)' angle-bracket form). fix-burst-29 stage-1 story-writer scope; product-owner BC amendment dispatched in parallel. |
| 1.28 | fix-burst-28 stage-1 | 2026-05-14 | story-writer | F-LP30-MED-001 (POL-7, codification #13 sub-extension): §References section appended BC-2.16.002 entry (verbatim H1 "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation") between ADR-023 §C4 and BC-2.17.001 in alphanumeric BC-ID order. Cross-table completeness gap: BC-2.16.002 was anchored in `behavioral_contracts:` since v1.2 (fix-burst-2) and appears verbatim in body BC table line 260, but never landed in §References. Total §References BC entries: 8 → 9 (8 `behavioral_contracts:` anchored + BC-2.17.005 exclusion-note per Codification #15). fix-burst-28 stage-1. |
| 1.27 | fix-burst-27 stage-1 | 2026-05-13 | story-writer | F-LP29-MED-001 (POL-7, codification #13 extension): story line 269 BC-2.17.005 title appended ", In-Flight Calls Complete Against Old Version" to make verbatim BC H1 / BC-INDEX line 219 / §References line 1016. 5th POL-7 recurrence; fix-burst-26 §References sweep targeted anchored BCs only (behavioral_contracts: array), missed exclusion-note paragraph for non-anchored BC-2.17.005. Sibling-site sweep: "Atomic Module Swap" now appears at line 269 (verbatim, fixed) + line 1016 (verbatim, unchanged) + changelog rows (historical). Zero active-body paraphrase instances remain. fix-burst-27 stage-1. |
| 1.26 | fix-burst-26 stage-1 | 2026-05-13 | story-writer | F-LP28-MED-001 (POL-4, story site): phantom §-section "BC-2.16.002 §S-PLUGIN-PREREQ-D AC-16" replaced with "BC-2.16.002 §Canonical Structured Event Catalog row pipeline_max_requests_exceeded (anchored by AC-16 of S-PLUGIN-PREREQ-D)" at line 918; product-owner handles error-taxonomy.md:464 sibling drift in parallel. F-LP28-MED-002 (POL-4): AC-16 trace header at line 466 "BC-2.16.002 preconditions" replaced with "BC-2.16.002 §Canonical Structured Event Catalog row pipeline_max_requests_exceeded" — preconditions doesn't contain MAX_REQUESTS_PER_PIPELINE; cap introduced by AC-16, emission documented in catalog. F-LP28-LOW-001: Token Budget BC count 8→9 (BC-2.17.005 in inputs since fix-burst-25 not propagated to Token Budget row); row recomputed ~12,000→~13,500; Total 40,900→42,400; percentage 16.0%→16.6%. F-LP28-LOW-003: inputs prepended ADR-022-production-runtime-wiring.md (cited ~17 times throughout story but missing from inputs). fix-burst-26 stage-1 story-writer scope. |
| 1.25 | fix-burst-25 stage-1 | 2026-05-13 | story-writer | F-LP27-MED-001 (POL-4): subsystems: [SS-22, SS-17] → [SS-22, SS-17, SS-16] (SS-16 added per BC-2.16.002 subsystem: SS-16 + S-PLUGIN-PREREQ-B precedent + AC-16 MAX_REQUESTS_PER_PIPELINE in prism-spec-engine/src/pipeline.rs SS-16 territory); YAML comment block updated with SS-16 justification; anchor_subsystem updated symmetrically. F-LP27-MED-002 (CLAUDE.md production-grade): PluginError #[non_exhaustive] conditional MVP-hedge language replaced with direct prescription — PluginError MUST be marked #[non_exhaustive] same-commit as new variants (aligns PrismError at error.rs:15-17 sibling + 30+-type perimeter audit); §non_exhaustive Requirements section updated to include PluginError enum-level as explicit unconditional requirement. F-LP27-MED-003 (POL-7): §References section rewritten with verbatim BC H1 titles for all 8 BCs (was 7/8 paraphrased — sibling pattern to codification #12); BC-2.17.007 parenthetical annotation preserved. F-LP27-LOW-001: inputs: appended BC-2.17.005-plugin-hot-reload-atomic-swap.md (cited at body line 980 + §References but absent from inputs since fix-burst-23). fix-burst-25 stage-1. |
| 1.24 | fix-burst-24 stage-1 | 2026-05-13 | story-writer | F-LP26-MED-001 BC-2.16.002 body BC table title (line 254) canonicalized: paraphrased sub-scope "Multi-Step Fetch Pipeline — Structured Event Catalog" → verbatim BC H1 + BC-INDEX "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation" per POL-7. Primary Coverage cell unchanged (story-specific sub-scope label preserved there). 7 other BCs in same table already verbatim; only BC-2.16.002 had asymmetric deviation. fix-burst-24 stage-1. |
| 1.23 | fix-burst-23 stage-1 | 2026-05-13 | story-writer | F-LP25-HIGH-001 spawn_blocking re-anchor (Architecture Compliance Rules row 980: ADR-023 §C4 → BC-2.17.005 §Invariants); F-LP25-LOW-001 SS-17 short-name normalization (line 48: "Plugin Runtime" → "WASM Plugin Runtime" per POL-6); F-LP25-LOW-002 AC-9 trace header strip "ADR-023 §C4 plugin HTTP defaults +" fabricated prose (line 367) — keep BC-2.17.002 v1.5 §Error Conditions E-PLUGIN-005 canonical only. fix-burst-23 stage-1. |
| 1.22 | pass-23 fix-burst-22 stage 1 | 2026-05-13 | story-writer | Closes F-LP23-HIGH-001 (type-contract regression introduced in fix-burst-21: 8 Option-syntax sites contradicted AC-7/AC-17/Task-2 declaration that `allowed_urls` field type is `Vec<String>` not `Option<Vec<String>>`). Fix 1 — `test_default()` body (Site 2): `allowed_urls: None` comment "default-deny (None = permit-all pre-AC-17)" replaced with `allowed_urls: vec![]` and comment "empty list = default-deny under AC-7 Vec<String> contract; tests that need allowlist enforcement must override with vec![\"host\"]". Fix 2 — AC-17 migration example (Site 1): functional-update example with `allowed_urls: Some(vec![...])` replaced with `allowed_urls: vec![\"host\".to_string()]` or drop-field-override note. Fix 3 — Match-Site Inventory test site 1 (line 287): prescription updated from `allowed_urls: None` migration to "drop field override; vec![] is default-deny default". Fix 4 — Match-Site Inventory test site 2 (line 305): `allowed_urls: Some(vec![...])` → `allowed_urls: vec![...]`. Fix 5 — Match-Site Inventory test site 3 (line 912): full obsolete-test adjudication — rename + invert assertion (Option A.ii). Fix 6 — Match-Site Inventory test site 4 (line 946): `allowed_urls: Some(vec!["example.com"...])` → `allowed_urls: vec!["example.com"...]`. Fix 7 — Match-Site Inventory test site 5 (line 977): same as site 4. Fix 8 — Match-Site Inventory test site 6 (line 1018): drop `allowed_urls: None` field override with parenthetical "(allowed_urls: vec![] is the default = default-deny under AC-7 Vec<String> contract)". New subsection "Obsolete Tests under AC-7 Vec<String> Contract" added to AC-17 body: adjudicates `test_BC_2_17_002_ec17_007_http_request_no_allowlist_allowed` (plugin_tests.rs:907) as obsolete under default-deny semantics; chooses Option A.ii (invert assertion to 403 blocked + rename); prescribes 5 implementer migration steps; rationale inline per CLAUDE.md Canonical Principle Rule 6; confirms no other obsolete-semantics tests via external-anchor grep. POL-22 Phase B (10th codification candidate: internal-cross-reference type-unification verification) raised — routes to state-manager for cycle-close tracking. External-anchor verifications: (1) plugin_tests.rs:292 `allowed_urls: None` confirmed (state_open, tests invalid URL → 400, not allow-all semantics) — PASS; (2) plugin_tests.rs:305 `allowed_urls: Some(vec!["allowed-sensor.internal"...])` confirmed — PASS; (3) plugin_tests.rs:907 `test_BC_2_17_002_ec17_007_http_request_no_allowlist_allowed` with `allowed_urls: None` + `assert_ne!(status, 403)` (obsolete allow-all semantics) confirmed — PASS; (4) plugin_tests.rs:946/982 `allowed_urls: Some(vec!["example.com"...])` confirmed — PASS; (5) plugin_tests.rs:1023 `allowed_urls: None` in timeout test (does not call host_http_request directly) confirmed — PASS; (6) loader.rs:106 `allowed_urls: Option<Vec<String>>` (pre-implementation state) confirmed — PASS. Sibling sweeps: (1) `allowed_urls: None` active body — ZERO hits (only historical changelog and justified external-anchor citation); (2) `allowed_urls: Some(` active body — ZERO hits; (3) `allowed_urls: vec!` active body — 3 hits (test_default body, site 2 migration, site 4/5 migration) all correct Vec<String> syntax; (4) `None.*pre-AC-17` active body — ZERO hits (obsolete comment removed from test_default); (5) `test_BC_2_17_002_ec17_007_http_request_no_allowlist_allowed` active body — appears in obsolete-test adjudication section with explicit migration prescription. Token Budget: new §Obsolete Tests subsection ~+600 chars; 8 site rewrites (net ~+200 chars); total net ~+800 chars (~200 tokens). Crosses 50-token threshold: story-spec row 7,900 → 8,100; Total 40,700 → 40,900; pct 40,900 / 256,000 = 15.977% → rounds to 16.0% (pct cell bumped from 15.9% to 16.0%). Still within 20-30% limit; no splitting required. |
| 1.21 | pass-22 fix-burst-21 stage 1 | 2026-05-13 | story-writer | Closes F-LP22-MED-001 (AC-17 Match-Site Inventory gap: 6 in-tree integration test `HostState` struct literal construction sites in `crates/prism-spec-engine/tests/plugin_tests.rs` — lines 287, 305, 912, 946, 977, 1018 — were not enumerated; adding `#[non_exhaustive]` to `HostState` per AC-17 would cause E0639 compile failures at all 6 sites). Fix 1 — Match-Site Inventory: 6 new rows appended after existing 13 rows, one per test-crate site, with file + line anchor per TD-VSDD-091 justified exception (test-code citations accepted in Match-Site Inventory), functional-update remediation prescription per site, and dependency on `HostState::test_default()` constructor. Fix 2 — AC-17 body augmentation: paragraph added after code-review gate verification line, enumerating the 6 test-crate sites, prescribing `#[cfg(any(test, feature = "test-helpers"))]` gate per project convention (mirroring `auth_provider.rs` and `lib.rs` existing usage), and requiring the constructor introduction + all 6 site migrations land in the SAME commit as the `#[non_exhaustive]` addition to preserve compile-green workspace. `HostState::test_default()` recommended signature included with all 6 fields (`http_client`, `config`, `kv_store`, `plugin_id`, `allowed_urls: None`, `limits: wasmtime::StoreLimits::default()`). F-LP22-OBS-001 (PluginError pre-condition out-of-perimeter) routes to phase-5 deferred-findings via state-manager (stage 2). External-anchor verifications: (1) `plugin_tests.rs` lines 287/305/912/946/977/1018 — all confirmed `HostState { ... }` struct literal patterns — PASS; (2) `loader.rs:101` `pub struct HostState` confirmed — PASS; (3) `#[cfg(any(test, feature = "test-helpers"))]` pattern confirmed in `auth_provider.rs` and `lib.rs` — PASS; (4) `sandbox.rs:58` same-crate construction (not affected by `#[non_exhaustive]`) — PASS; (5) `test-helpers = []` feature declared in `prism-spec-engine/Cargo.toml` line 60 — PASS. Token Budget: AC-17 augmentation ~+800 chars; 6 Match-Site rows ~+600 chars; total net ~+1,400 chars (~350 tokens). Crosses 50-token threshold: story-spec row 7,600 → 7,900; Total 40,400 → 40,700; pct 40,700/256,000 = 15.898% → 15.9% (pct cell bumped from 15.8% to 15.9%). |
| 1.20 | pass-21 fix-burst-20 stage 1B | 2026-05-13 | story-writer | Closes F-LP21-HIGH-001 story portion: AC-16 fabricated type `PipelineError::TooManyRequests` replaced with canonical `SpecEngineError::TooManyRequests { total: usize }` per `crates/prism-spec-engine/src/error.rs:15` (`#[derive(Debug, Error)] #[non_exhaustive] pub enum SpecEngineError`); rationale prose added to AC-16 body citing existing variants (`AuthAcquisitionFailed`, `AuthRefreshFailed`, `HttpRequestFailed`, `JsonPathExtractionFailed`) as additive variant template, canonical error code `E-PIPELINE-001` (allocated by PO stage 1A), and POL-1 append-only invariant. §Error Taxonomy Additions intro: "Four new error codes" → "Five new error codes"; intro rewritten to distinguish PluginError (4 variants in prism-core) from SpecEngineError (1 variant in prism-spec-engine); new `E-PIPELINE-001` row added to table with BC-2.16.002 v1.16 trace and AC-16 anchor; narrative sentence added describing non-retryable abort semantics. External-anchor verifications: (1) `pub enum SpecEngineError` confirmed at error.rs:15 with `#[derive(Debug, Error)] #[non_exhaustive]` — PASS; (2) existing variants `AuthAcquisitionFailed`/`AuthRefreshFailed`/`HttpRequestFailed`/`JsonPathExtractionFailed` confirmed — PASS; (3) `usize` field type is primitive (PartialEq+Eq compatible) — PASS. Sibling sweeps: (1) `PipelineError` active body — ZERO hits (changelog historical only) — PASS; (2) `TooManyRequests` body — all hits cite `SpecEngineError::TooManyRequests` — PASS; (3) `pipeline_max_requests_exceeded` body — consistent with BC-2.16.002 v1.16 catalog row — PASS; (4) extended `[A-Z]\w+Error::` sweep — `PrismError::Internal` (AC-9), `PluginError::*` (AC-5/§Error Taxonomy), `SpecEngineError::*` (AC-16/§Error Taxonomy) — all verified against canonical source — PASS; (5) `E-PIPELINE` body — new E-PIPELINE-001 row consistent with PO stage 1A allocation — PASS. 3rd recurrence of external-anchor mis-prescription pattern reaches threshold for codification candidate 9 (`adversary-must-verify-external-anchors-recursively`) — formal POL-22 proposal at cycle-close. Token Budget: AC-16 rationale prose ~+200 chars; Error Taxonomy row + intro rewrite ~+350 chars; total net ~+550 chars (~138 tokens). Crosses 50-token threshold: story-spec row 7,500 → 7,600; Total 40,300 → 40,400; pct 40,400 / 256,000 = 15.781% → rounds to 15.8% (pct cell bumped from 15.7% to 15.8%). |
| 1.19 | pass-20 fix-burst-19 stage 1 | 2026-05-13 | story-writer | Closes F-LP20-MED-001 (3 sibling-prose sites carrying stale `BC-2.16.002 v1.11` version pin — 6th recurrence of lexical-vs-semantic sweep pattern, version-pin-drift sub-pattern). Site 1 (AC-3 body line ~294): `v1.11` → `v1.12` in catalog discipline citation. Site 2 (AC-7 body line ~351): `v1.11` → `v1.12` in single-structured-emission catalog discipline citation. Site 3 (§Structured Event Catalog Additions intro line ~692): `Per BC-2.16.002 v1.11` → `Per BC-2.16.002 v1.16`. Extended deprecated-version sweep per pass-20 explicit recommendation (6th-recurrence codification threshold): (1) `BC-2.16.002 v1.1[01]` active body — ZERO hits after fix (only changelog historical lines remain); (2) `BC-2.16.002 v1.16` active body — 8 sites consistent; (3) Extended sweep of all `behavioral_contracts` array BCs — `BC-2.22.001 v1.[0-4]` hits in changelog only (line 941); `BC-2.17.002 v1.[0-4]` hits in changelog/historical prose only (lines 413, 939, 941); `BC-2.17.001 v1.X` — zero active-body versioned hits; `BC-2.17.007 v1.[01]` — zero active-body versioned hits. No additional stale version pins found in active body. Token Budget: 3 × 1-char digit changes ≈ 0 token delta; Total stays 40,300; pct stays 15.7%. |
| 1.18 | pass-19 fix-burst-18 stage 1 | 2026-05-13 | story-writer | Closes F-LP19-MED-001 (3 sibling-prose sites carrying incomplete manifest rejection enumeration and WRONG `allowed_urls non-empty` framing — 5th recurrence of lexical-vs-semantic sweep pattern). Site 1 (Summary lines 181-182): rejection enumeration rewritten to cover all 4 manifest codes: "fails any of four schema validations (E-PLUGIN-013 missing `allowed_urls`; E-PLUGIN-014 `format_version` exceeds `CURRENT_SUPPORTED_VERSION`; E-PLUGIN-015 missing or empty `name`; E-PLUGIN-016 malformed `version` semver)". Site 2 (§Scope bullets after line 228): 2 new rejection bullets added — E-PLUGIN-015 name field missing/empty; E-PLUGIN-016 version field not valid semver — completing 4-code symmetry in the in-scope rejection list. Site 3 (§Scope lines 214-217 multi-line wrap): "`allowed_urls` non-empty" WRONG framing eliminated; replaced with "validate manifest schema (4 fields: `name` non-empty per E-PLUGIN-015; `version` semver-parseable per E-PLUGIN-016; `format_version <= CURRENT_SUPPORTED_VERSION` per E-PLUGIN-014; `allowed_urls` explicitly present per E-PLUGIN-013 — empty list `[]` accepted, absent/null rejected)" per AC-5 line 313 semantics. Semantic + multi-line sweep applied to all 18 story sections; no additional sites found beyond the 3 targeted. F-LP19-LOW-001 = no action (Background correctly describes pre-PREREQ-D state). F-LP19-LOW-002 = out-of-perimeter (VP-INDEX framing); routes to state-manager phase-5 deferred. F-LP19-OBS-001 = codification candidate 5 (lexical-vs-semantic sweep formal POL-21 proposal); routes to state-manager for cycle-close tracking. Token Budget: 3 site rewrites + 2 new bullets net ~+486 chars (~122 tokens); crosses 50-token threshold; story-spec row 7,400 → 7,500; Total 40,200 → 40,300; pct 40,300 / 256,000 = 15.742% → rounds to 15.7% (pct cell unchanged). |
| 1.17 | pass-18 fix-burst-17 stage 1B | 2026-05-13 | story-writer | Closes F-LP18-MED-001 story portion: 2 new rows appended to §Structured Event Catalog Additions (`plugin_load_failed_manifest_name_missing` for E-PLUGIN-015; `plugin_load_failed_manifest_version_malformed` for E-PLUGIN-016); count narratives updated 7→9 at 4 sites (frontmatter comment line 11; BC table BC-2.16.002 row; §Structured Event Catalog Additions preamble; §Previous Story Intelligence item 1). BC-2.16.002 version reference advanced to v1.12 (parallel PO amendment in stage 1A). Closes F-LP18-LOW-001: Task 1 line 506 validation list replaced — 4-field complete validation with correct allowed_urls semantics (empty list accepted, absent/null rejected per AC-5 line 313; version semver check added per AC-5 line 311/EC-D-013; first-failure-returns per BC-2.17.007 EC-17-032; `allowed_urls non-empty` WRONG framing eliminated). Closes F-LP18-LOW-002: Task 10 enumeration replaced with deferred reference to §Red Gate Tests (7 named tests) matching Task 11 pattern — eliminates 3-test asymmetry (`test_BC_2_22_001_plugin_load_failure_exits_code_4`, `test_BC_2_22_001_disable_env_takes_precedence_over_plugin_dir_config`, `test_VP_PLUGIN_007_plugin_load_rejected_format_version_exceeded` no longer enumerated inline; §Red Gate Tests is single source of truth). F-LP18-OBS-001 routes to 5th codification candidate (lexical-vs-semantic sweep — 4th recurrence; cycle-closing tracking via state-manager). External-anchor verifications: §Red Gate Tests prism-bin block 7 tests PASS; AC-5 E-PLUGIN-013/014/015/016 mapping PASS; EC-D-012/D-013 rows confirmed PASS; BC-2.17.007 EC-17-032 ref confirmed PASS; §Structured Event Catalog Additions format PASS. Sibling-prose sweep: `7 event` ZERO active-body hits (changelog only); `7 new` ZERO active-body hits (changelog only); `9 events` appears in updated narratives; `allowed_urls non-empty` ZERO hits; Task 10 matches Task 11 deferred pattern; format_version+version validation cited consistently across Task 1/AC-5/EC/Catalog. Token Budget: 2 catalog rows ~+150 chars; Task 1 rewrite ~+50 chars; Task 10 net ~-100 chars; count narrative 4 sites ~+40 chars; total net ~+140 chars (~35 tokens); below 50-token recompute threshold; Total stays at 40,200. |
| 1.16 | pass-17 fix-burst-16 stage 1 | 2026-05-13 | story-writer | Closes F-LP17-LOW-001 (Task 5 zeroize directive: stale hedge "(or use already-present if available)" removed; "dev/prod deps" ambiguity resolved — `AuthToken` in `auth_provider.rs` is production code, so `[dependencies]` not `[dev-dependencies]`; full crate-qualified path `crates/prism-spec-engine/Cargo.toml` added). Closes F-LP17-LOW-002 (end-of-table prose hedges at 2 sites: §Library & Framework Requirements (MANDATORY) line 156 + §Library and Framework Requirements line 881 — both conditional `if zeroize is not yet` / `if zeroize is absent` replaced with firm assertion that `zeroize` is currently absent per confirmed Cargo.toml inspection, with explicit line ranges cited). Closes F-LP17-LOW-003 (EC table: 2 new rows added — EC-D-012 for `E-PLUGIN-015` name-field empty/absent; EC-D-013 for `E-PLUGIN-016` version-field malformed; append-only per POL-1; all 4 manifest error codes 013/014/015/016 now have EC table coverage). Closes F-LP17-OBS-001 (frontmatter arrays populated from existing AC body content per Path A: `assumption_validations` 5 items from EC-D-009/AC-4/AC-3/AC-18/AC-7/AC-10; `risk_mitigations` 8 items from AC-9/AC-13/AC-10/AC-1/AC-2/AC-3/AC-16/AC-7; frontmatter comment added citing Path A + process-gap candidate 7 routes to cycle-closing). Process-gap candidate 7 raised: story-writer template enforcement for risk:HIGH stories — routes to orchestrator session-reviewer adjudication. Token Budget: 2 EC rows ~+180 chars, frontmatter arrays ~+1,200 chars, end-of-table rewrites ~+100 chars, Task 5 rewrite ~+80 chars, total net ~+1,560 chars (~390 tokens). Crosses 50-token threshold: story-spec row 7,300 → 7,400; Total 40,100 → 40,200; pct 40,200 / 256,000 = 15.703% → rounds to 15.7% (pct cell unchanged). |
| 1.15 | pass-16 fix-burst-15 stage 1 | 2026-05-13 | story-writer | Closes F-LP16-HIGH-001 (AC-9 code sample `.map_err(|e| PrismError::PluginRuntimeInit { source: e })?` → `.map_err(|e| PrismError::Internal { detail: format!("PluginRuntime HTTP client construction failed: {}", e), })?`; non-existent variant `PluginRuntimeInit` replaced with canonical E-INT-001 `PrismError::Internal { detail: String }` at error.rs:881-883; rationale prose added: maps to exit(4) per ADR-022 §A line 146, preserves PartialEq+Eq derive, matches stringify convention at error.rs:171; recursive verification gap caught — pass-15 prescription introduced this HIGH defect by citing a variant that does not exist). Closes F-LP16-LOW-002 (punt prose "Note on error variant" lines deleted per Canonical Principle Rule 6 — Path A uses existing E-INT-001 variant, no punt to implementer). Closes F-LP16-MED-001 (Error Taxonomy Additions source file corrected from non-existent `crates/prism-spec-engine/src/plugin/error.rs` to canonical `crates/prism-core/src/error.rs` lines 984-1034; `use prism_core::PluginError` import pattern cited from mod.rs:16). Closes F-LP16-MED-002 (File Structure prism-spec-engine/Cargo.toml row: stale "if not present" hedge + "sha-2" guesswork replaced with explicit state: zeroize absent — ADD; url absent — ADD; sha2 already at line 21 — no change). Closes F-LP16-LOW-001 (File Structure prism-bin/Cargo.toml row: stale "if not already present" hedge replaced with Option (b) explicit no-modification confirmation — dep already present at line 35 from S-WAVE5-PREP-01). All 6 external-anchor verifications PASS (PrismError::Internal error.rs:881-883; PluginError enum error.rs:984-1034; sha2 prism-spec-engine/Cargo.toml:21; prism-spec-engine prism-bin/Cargo.toml:35; use prism_core::PluginError mod.rs:16; ADR-022 §A line 146 exit(4)). Token Budget: punt prose deletion ~-250 chars; AC-9 rationale expansion ~+300 chars; Error Taxonomy fix ~+30 chars; File Structure row rewrites ~+50 chars; net ~+130 chars (~33 tokens); below 50-token recompute threshold; Total stays at ~40,100. |
| 1.14 | pass-15 fix-burst-14 stage 1 | 2026-05-13 | story-writer | Closes F-LP15-MED-001 (AC-9 `.expect()` replaced with `.map_err(...)?` propagation; client construction is fallible per EC-D-009 — OS resource exhaustion returns Err, boot exits code 4 per ADR-022 §A; `expect_used = "deny"` in workspace lints makes `.expect()` a clippy-deny failure; error variant guidance added with POL-1 append-only note). Closes F-LP15-MED-002 (both Library Requirements tables — §Library & Framework Requirements (MANDATORY) and §Library and Framework Requirements — corrected from "workspace dep" / "workspace version" framing: workspace `Cargo.toml` has no `[workspace.dependencies]` table; all deps are crate-local pins in `crates/prism-spec-engine/Cargo.toml`; sha2 row: "confirmed workspace dep line 21" → "crate-local pin line 21, no Cargo.toml change required"; url row: "Must already be present" → "NOT currently present — ADD `url = "2"`"; reqwest row: "workspace version" → `"0.12"` crate-local pin line 34; arc-swap row: "workspace version" → `"1"` crate-local pin line 20; tokio row: "workspace version" → `"1"` crate-local pin line 26; both DRY tables updated symmetrically). Closes F-LP15-LOW-001 (Error Taxonomy Additions intro "Two new error codes" → "Four new error codes"; added E-PLUGIN-015 row `PluginError::ManifestNameMissing` + E-PLUGIN-016 row `PluginError::ManifestVersionMalformed` consistent with AC-5 BC-2.17.007 postconditions 1–2 and existing E-PLUGIN-013/014 naming conventions). Meta: pass-15 identified external-anchor verification gap (adversary-must-verify-external-anchors) as 5th process-gap codification candidate; 3 instances across passes 1/14/15 meets threshold. Token Budget delta: AC-9 code block + prose additions ~+400 chars; both table rewrites ~+300 chars; 2 new error taxonomy rows ~+200 chars; total net ~+900 chars (~225 tokens). Story-spec row: 7,200 → 7,300; Total: 40,000 → 40,100; pct: 40,100 / 256,000 = 15.664% → rounds to 15.7% (pct cell bumped from 15.6% to 15.7%). |
| 1.13 | pass-14 fix-burst-13 stage 1 | 2026-05-13 | story-writer | Closes F-LP14-LOW-001 (Summary lines 166-167 cardinality contradiction with AC-4: "emits a WARN-level boot log plus an audit entry for every successfully loaded plugin" rewritten to "emits per-plugin audit entries accompanied by a one-time boot-level WARN log" per Option A.2 — explicit cardinality disambiguation; WARN log is once-per-boot, audit entry is per-plugin). Closes F-LP14-OBS-001 (AC-3 + AC-7 cross-reference ambiguity resolved with Option B — drop "same convention as plugin_load_unsigned per AC-4" framing; both now anchor directly to "BC-2.16.002 v1.11 catalog discipline — WARN-level log and audit-channel routing are orthogonal via event_type field"). Extended semantic sibling-sweep (8 checks): (1) Summary section re-read — cardinality rewrite verified, no other compound emission claims found, PASS; (2) Background section — no cardinality claims contradicting AC body, PASS; (3) Scope section lines 205-206 — "Boot-time WARN log: WARNING..." vs "Audit log entry per plugin load: event_type: plugin_load_unsigned" correctly disambiguates once-per-boot vs per-plugin, no regression after Summary rewrite, PASS; (4) EC table cardinality — EC-D-004 single-emission framing intact, EC-D-002 correctly notes unsigned WARN not emitted when zero plugins loaded, no per-plugin vs per-boot conflation, PASS; (5) grep `for every` body — single remaining hit is "for every .prx plugin" in AC-4 body which is correct (it sets up the 2-emission deliberate framing: item 1 is once-per-boot, item 2 is per-plugin); PASS; (6) grep `per plugin` body — all hits correctly describe per-plugin audit entry behavior; PASS; (7) grep `accompanied by` — appears in corrected Summary wording; PASS; (8) AC-4 no-regression — lines describing "1. One boot-time WARN (emitted once per boot, not per plugin)" and "2. Per-plugin audit entry" preserved; NO REGRESSION. Pass-14 meta-finding (sibling-prose cardinality axis — Summary paraphrases AC body without preserving dual-cardinality semantics — noted as potential 5th codification candidate alongside lexical-vs-semantic). Token Budget delta: Summary rewrite net-neutral (~0 char change); AC-3 + AC-7 cross-ref rewrites net-neutral (~+20 chars); stays at ~40,000 (within 50-token tolerance; no recompute). |
| 1.12 | pass-13 fix-burst-12 stage 1 | 2026-05-13 | story-writer | Closes F-LP13-LOW-001 (3 sibling-prose sites carrying dual-emission "WARN log + audit entry" framing for events BC-2.16.002 catalogs as single-emission): Site 1 AC-7 body (allowlist-mismatch path) rewritten to single `tracing::warn!(event_type = "plugin_http_request_blocked", ...)` framing with orthogonal Level/routing cross-reference; Site 2 Task 3 bullet rewritten to single-emission framing referencing BC-2.16.002 catalog + AC-7; Site 3 Task 9 disable bullet rewritten to single-emission framing referencing BC-2.16.002 catalog + AC-3. Concise-form decision (pass-13 §7): Option (b) chosen — all 3 concise-form sites (EC-D-004, EC-D-010, AC-18 item 1) rewritten to unambiguous single-emission framing to achieve full semantic consistency; mixed convention in a single-file would create implementer ambiguity. EC table entries use compact but unambiguous form referencing BC-2.16.002 catalog. Semantic sibling sweep (8 checks, per pass-13 meta-finding on lexical-vs-semantic distinction): (1) `audit entry` — all remaining hits are Changelog historical text only, PASS; (2) `WARN log + audit` active body — ZERO hits, PASS; (3) `emit WARN + audit entry` active body — ZERO hits, PASS; (4) `WARN.*+.*audit` regex — ZERO active-body hits, PASS; (5) `+ audit` active body — ZERO active-body hits beyond Changelog, PASS; (6) `event_type:` references in story body — all use single-emission framing, PASS; (7) concise-form sites EC-D-004/EC-D-010/AC-18 — rewritten Option (b), PASS; (8) AC-4 2-emission no-regression — AC-4 boot-time aggregate WARN + per-plugin structured audit preserved, NO REGRESSION. Pass-13 meta-finding (lexical-vs-semantic sweep gap as 5th process-gap codification candidate) routed to state-manager. Token Budget delta: 6 rewrites ~+450 chars (~112 tokens); crosses 50-token threshold; story-spec row 7,100 → 7,200; Total 39,900 → 40,000; pct 40,000 / 256,000 = 15.625% → rounds to 15.6% (no pct cell change). |
| 1.11 | pass-12 fix-burst-11 stage 1 | 2026-05-13 | story-writer | Closes F-LP12-LOW-001 (AC-3 prose single-emission clarity: dual-emission framing "A WARN log is emitted... An audit log entry is written" replaced with explicit single `tracing::warn!(event_type = "plugin_load_disabled_via_envvar", ...)` emission prose; orthogonal Level/routing cross-reference to BC-2.22.001 v1.5 + BC-2.16.002 catalog discipline added, matching AC-4 `plugin_load_unsigned` convention). F-LP12-OBS-001 (E-PLUGIN-008 dual-semantic reuse gap between BC-2.17.005 and BC-2.17.006) is out-of-story-perimeter; routed to state-manager for phase-5 deferred-findings list. Sibling-sweep (TD-VSDD-060): 5 mandatory greps all PASS — (1) zero hits `audit log entry is written` active body; (2) single remaining `audit log entry` hit is Changelog historical text only; (3) `event_type` body references all use single-emission framing consistent with BC-2.16.002; (4) Structured Event Catalog `plugin_load_disabled_via_envvar` Trigger column consistent with corrected AC-3; (5) EC-D-004 row uses `WARN + audit plugin_load_disabled_via_envvar` concise framing (no dual-emission implication). AC-4 2-emission framing preserved — no regression. Token Budget delta +~160 chars (~40 tokens); Total ~39,900 (within 50-token tolerance; no recompute required). |
| 1.10 | pass-11 fix-burst-10 stage 1 | 2026-05-13 | story-writer | Closes F-LP11-LOW-001 (4 sibling-prose sites carrying `Some(...)` Option-wrapping from fix-burst-4 F-LP4-LOW-003 — surviving passes 5–10): Site 1 Scope bullet (construct `HostState { allowed_urls: parsed_hostnames }`); Site 2 Task 1 success path (same); Site 3 Task 2 first bullet (rewrite: `Replace allowed_urls: None field-default with Vec<String> parameter; field type retired from Option<Vec<String>> to Vec<String> per AC-17; None-branch type-system-impossible`); Site 4 Match-Site Inventory closure column (`parsed_hostnames value (Vec<String> per AC-17; None-branch type-system-impossible)`). Closes F-LP11-LOW-002 (Token Budget percentage cell 15.5% → 15.6%: 39,900 / 256,000 = 15.586%, rounds half-up to 15.6%; Total row unchanged at ~39,900). Sibling-sweep (TD-VSDD-060): 5 mandatory greps all PASS — zero hits for `Some(parsed_hostnames)`, `Some(urls_from_manifest)`, `allowed_urls: Some`, `approximately 15.5`; exactly one hit for `approximately 15.6`. |
| 1.9 | pass-10 fix-burst-9 stage 1 | 2026-05-13 | story-writer | Closes F-LP10-LOW-001 (Task 14 + Previous Story Intelligence item 1 Path B sibling-prose propagation). Task 14 reworded from "Update Structured Event Catalog" (implies implementer authors rows) to "Verify Structured Event Catalog wiring" with explicit instruction to emit from BC-2.16.002 v1.11 function-name anchors and amend only if new sites are discovered. Previous Story Intelligence item 1 corrected from "must add all 7 rows" to "all 7 rows already exist in BC-2.16.002 v1.11 (Path B, fix-burst-8 commit 4ed96e06); implementer wires emission sites to match BC row metadata." Upstream PO Path B at `4ed96e06`. Sibling-site sweep confirmed zero additional sites carrying old "add"/"author catalog rows" or "rows do not yet exist" framing. Token Budget delta +~110 tokens; Total recomputed below (see §Token Budget Estimate). |
| 1.8 | pass-9 fix-burst-8 stage 2 | 2026-05-13 | story-writer | F-LP9-MED-001 story portion: Catalog Additions preamble updated to Path B — events already added to BC-2.16.002 v1.11 in fix-burst-8 stage 1 (commit 4ed96e06); implementer role reframed from "will add" to "verify wiring matches BC row". Table metadata corrected to match BC-2.16.002 v1.11 canonical rows: (1) `plugin_load_disabled_via_envvar` emitter corrected from `boot.rs plugin-load step` → `boot::plugin_load_step` (TD-VSDD-091 function-name anchor); (2) `plugin_load_disabled_via_envvar` Level corrected from `WARN/AUDIT` → `WARN` (BC v1.11 authoritative); (3) `plugin_http_request_blocked` Level corrected from `WARN/AUDIT` → `WARN` (BC v1.11 authoritative); (4) trigger text for `plugin_load_unsigned` and 5 other rows aligned to BC v1.11 prose. Sister-site sweep: no "PipelineExecutor catalog" old framing found in story file (grep confirmed zero hits). F-LP9-LOW-001: AC-9 body closure note temporal contradiction resolved — Form A: "Closed by BC-2.17.002 v1.4 amendment (fix-burst-6); current pinned version v1.5 (fix-burst-7 lifecycle_status-only sweep)." Upstream PO stage 1 at factory SHA `4ed96e06`. |
| 1.7 | pass-8 fix-burst-7 stage 2 | 2026-05-13 | story-writer | Closes F-LP8-HIGH-001 story portion (line 16 frontmatter comment "All BCs are active" → accurate POL-14 lifecycle_status statement: BC-2.22.001 active; BC-2.17.001/002/003/004/006/007 draft pending PR-merge promotion). Closes F-LP8-MEDIUM-001 (Structured Event Catalog `plugin_load_unsigned` Level AUDIT → WARN; audit-channel routing captured via `event_type` field note; `plugin_load_disabled_via_envvar` WARN/AUDIT unchanged — correct per BC-2.22.001 §Postconditions escape valve). Closes F-LP8-MEDIUM-002 (AC-9 trace header extended: ADR-023 §C4 + BC-2.17.002 v1.5 §Error Conditions E-PLUGIN-005; AC-9 closure note version ref v1.4 → v1.5). Upstream stages: PO BC lifecycle sweep at factory SHA `a03d9d36`, architect ADR-022 v1.3 cross-ref at factory SHA `b0021477`. |
| 1.6 | pass-7 fix-burst-6 stage 2 | 2026-05-13 | story-writer | Closes F-LP7-HIGH-001 (path mis-anchor `src/plugin/pipeline.rs` → `src/pipeline.rs`: Architecture Mapping, Purity Classification, AC-16 body, Token Budget row, File Structure Modified Files, Match-Site Inventory ×4 rows — 8 sites swept). Closes F-LP7-HIGH-002 (path mis-anchor `src/plugin/auth_provider.rs` → `src/auth_provider.rs`: Architecture Mapping, Purity Classification, AC-15 body, File Structure Modified Files, Match-Site Inventory ×2 rows — 5 sites swept). Closes F-LP7-HIGH-004 (paper-fix risk AC-9 / TD-B-005: new Match-Site Inventory row for `host_functions.rs` per-request `.timeout(10)` override; Task 4 extended with TD-VSDD-060 sibling-site sweep instructions for `host_http_request` builder + doc-comment update). Closes F-LP7-MED-002 (Task 9 step numbering: removed "or new 8, query-engine=new 9" alternative; final wording: storage=7, plugin-load=7.5, query-engine=8, MCP=9 with `step9_start_mcp_server` retained, rationale cited). AC traces updated for BC-2.22.001 v1.4: AC-1 references step 7.5 in §Sequencing Invariant; AC-2 references §Pre-Traffic Gate Invariant condition 6; AC-3 references `plugin_load_disabled_via_envvar` audit event name from §Postconditions escape valve; AC-4 references happy-path step 7.5 postcondition. Event name `plugin_disabled_env` corrected to `plugin_load_disabled_via_envvar` throughout (Scope, EC-D-004, AC-3, AC-18 Task 9, Structured Event Catalog — 5 additional sites). AC-9 out-of-perimeter note removed and replaced with 1-line closed reference (MED-001 closed by BC-2.17.002 v1.4 amendment). State-manager stage 3 will close F-LP7-LOW-001 (BC-2.22.001 lifecycle_status) and update indexes. |
| 1.5 | pass-6 fix-burst-5 | 2026-05-13 | story-writer | Closes F-LP6-MEDIUM-001 (Token Budget arithmetic: row sum 39,800 corrected; Total ~38,300→~39,800; percentage ~15%→~15.5%). Closes F-LP6-LOW-002 (v1.1 changelog BC count notation: "8→7 BCs net" rewritten to "swap BC-2.17.005 for BC-2.17.007 (7→7 BCs net)"). Closes F-LP6-LOW-003 (Match-Site Inventory Closure column: "AC-8 tasks:" corrected to "Task 8:" to match column convention). Closes F-LP6-OBS-004 (AC-9 header re-anchored from BC-2.17.002 timeout citation to ADR-023 §C4; cross-doc gap BC-2.17.002 E-PLUGIN-005 10s vs 30s documented in AC-9 body as out-of-perimeter note for future PO-led amendment). 4/4 in-scope findings closed. |
| 1.4 | pass-4 fix-burst-4 | 2026-05-13 | story-writer | Closes F-LP4-MED-002 (v1.3 changelog row truthfulness: row now accurately discloses that pass-3 state-manager sweep covered 24 BCs initially at SHAs 4f1cd312+2385b188, that pass-4 adversary caught 8 remaining BCs missed by unanchored grep per F-LP4-MED-001, and that completion + POL-20 anchored-regex amendment land in parallel state-manager commit). Closes F-LP4-LOW-003 (AC-7 None-arm cleanup: `Option<Vec<String>>` language removed; None branch was type-system-impossible after AC-17 changes field to `Vec<String>`; dead-code defensive spec stripped per option-a recommendation). 2/2 in-scope findings closed. State-manager parallel commit handles F-LP4-MED-001 (8 remaining BCs) + F-LP4-OBS-004 (POL-20 regex amendment). |
| 1.3 | pass-3 fix-burst-3 | 2026-05-13 | story-writer | Closes F-LP3-MED-001 (Task 11 test list replaced with §Red Gate Tests reference; BC_2_17_006 mis-anchors on 2 test names corrected to BC_2_17_007 — manifest tests belong to BC-2.17.007 not BC-2.17.006/WIT), F-LP3-LOW-003 (AC-10 fixture path clarified: `trap_plugin.prx` compiled from `tests/fixtures/src/trap_plugin.wat`), F-LP3-LOW-004 (Match-Site Inventory out-of-scope TODO(S-4.08) rows now carry implementer rename instructions to distinguish closed vs open sites post-merge), F-LP3-OBS-005 (v1.2 changelog row updated: 6/8 in-story-file + 2/8 sibling artifacts = 8/8 closed across burst; VP-INDEX SHA and BC-2.17.007+policies.yaml SHA cited), F-LP3-OBS-006 (Architecture Compliance Rules spawn_blocking row re-anchored from BC-2.17.005 invariant to ADR-023 §C4 — BC-2.17.005 not in frontmatter). F-LP3-MED-002 dispatched to state-manager (pass-3 round 1) at SHAs 4f1cd312+2385b188 covering 24 BCs; pass-4 adversary caught 8 remaining BCs missed by unanchored verification grep (F-LP4-MED-001) → completion lands in parallel state-manager commit with POL-20 verification regex now anchored per F-LP4-OBS-004 closure (policies.yaml v1.10). 5/6 in-perimeter findings closed in this file. |
| 1.2 | pass-2 fix-burst-2 | 2026-05-13 | story-writer | Closes F-LP2-MED-001 (AC-14 re-anchored story-local; hot-reload test names drop BC-2.17.005 prefix), F-LP2-MED-002 (BC-2.16.002 added to behavioral_contracts + anchor_bcs + inputs + body BC table; capabilities/anchor_capabilities updated to [CAP-029,CAP-032,CAP-034]; Token Budget 7→8 BCs), F-LP2-MED-003 (red_gate_tests 0→25), F-LP2-LOW-004 (CAP-034 already in capabilities via BC-2.22.001; anchor_capabilities now union-correct; capabilities updated to [CAP-029,CAP-032,CAP-034]), F-LP2-LOW-005 (AC-17 moved to follow AC-16 — body order now matches AC numbering), F-LP2-OBS-008 (.github/PULL_REQUEST_TEMPLATE.md non-crate note added to File Structure). 6/8 pass-2 findings closed in this story file; F-LP2-LOW-006 closed in VP-INDEX v1.34 (SHA 4218e72a); F-LP2-OBS-007 closed in BC-2.17.007 v1.1 + policies.yaml v1.9 (POL-20 adopted) (SHA 97deaf37); 8/8 closed in-scope across the burst. |
| 1.1 | pass-1 fix-burst | 2026-05-13 | story-writer | Closes F-LP1-HIGH-002/003/005/006/007/008/009/010/011/012/013/014 + F-LP1-OBS-015/016 (14 findings). Re-anchors AC-5 to BC-2.17.007 (F-LP1-HIGH-004 PO fix). Drops BC-2.17.005 from frontmatter (scope gap per F-LP1-MED-010; S-1.12-FOLLOWUP owns watcher promotion). swap BC-2.17.005 for BC-2.17.007 (7→7 BCs net; BC-2.17.005 promotion deferred to S-1.12-FOLLOWUP per F-LP1-MED-010). Adds AC-17 (HostState #[non_exhaustive]) and AC-18 (PRISM_DISABLE_PLUGIN_LOAD precedence). Token budget updated. Fixture Strategy prose corrected to "4 fixtures". reqwest::Client single-instance semantics made explicit in AC-9, Architecture Mapping, and Implementation Notes. Test names standardized to BC/VP prefix convention. make_host_state sibling sites enumerated. sha2 workspace dep confirmed. wasmtime advisory count claim replaced with cargo-audit-based language. |
| 1.0 | PREREQ-D authorship | 2026-05-13 | story-writer | Initial authorship. Scope derived from ADR-023 v1.18 §C4. 16 ACs, 5 TDs absorbed (TD-B-002/004/005/011/012). |
