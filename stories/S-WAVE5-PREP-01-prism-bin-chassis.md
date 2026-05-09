---
story_id: S-WAVE5-PREP-01
title: "prism-bin: Binary Chassis, CLI, and Boot Sequence"
wave: 5
target_module: prism-bin
subsystems: [SS-06, SS-10, SS-11, SS-16]
priority: P0
depends_on: []
blocks: [S-3.02-FOLLOWUP-RUNTIME, S-1.12-FOLLOWUP, S-5.01-FOLLOWUP-MCP-BOOT, S-1.14-REDO-infusion-engine]
estimated_days: 3
points: 5
risk: HIGH
status: ready
document_type: story
version: "1.2"
level: "L4"
producer: story-writer
timestamp: "2026-05-08T00:00:00Z"
input-hash: "[md5]"
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-CLEANUP-02"
phase: 3
behavioral_contracts:
  - BC-2.10.001  # rmcp ServerHandler Implementation
  - BC-2.10.006  # Stdio Transport
  - BC-2.10.010  # Graceful Shutdown on SIGTERM/SIGINT
  - BC-2.06.011  # ConfigManager initialization validation (SS-06)
  - BC-2.21.001  # OrgRegistry init bijective-resolution contract (SS-21)
  - BC-2.03.013  # CredentialStore init reference-validation-only + no-leak invariant (SS-03)
  - BC-2.05.012  # AuditEmitter init audit_buffer-CF-open + boot.audit.initialized emitted (SS-05)
  - BC-2.22.001  # Boot orchestration: sequencing + exit-code map + traffic gate (SS-22)
verification_properties: []
assumption_validations: []
risk_mitigations: []
anchor_bcs:
  - BC-2.10.001
  - BC-2.10.006
  - BC-2.10.010
  - BC-2.06.011
  - BC-2.21.001
  - BC-2.03.013
  - BC-2.05.012
  - BC-2.22.001
anchor_capabilities: [CAP-034]
anchor_subsystem: ["SS-10", "SS-06", "SS-11", "SS-16", "SS-03", "SS-05", "SS-21", "SS-22"]
inputs:
  - ".factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md"
  - ".factory/specs/architecture/module-decomposition.md"
  - ".factory/specs/architecture/ARCH-INDEX.md"
  - ".factory/specs/behavioral-contracts/BC-INDEX.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.011-config-load-on-startup.md"
  - ".factory/specs/behavioral-contracts/BC-2.21.001-org-registry-init.md"
  - ".factory/specs/behavioral-contracts/BC-2.03.013-credential-store-init.md"
  - ".factory/specs/behavioral-contracts/BC-2.05.012-audit-subsystem-init.md"
  - ".factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md"
---

# S-WAVE5-PREP-01 — prism-bin: Binary Chassis, CLI, and Boot Sequence

## Narrative

As the Prism runtime platform, I want a `prism-bin` binary crate that provides the
CLI surface, 11-step ordered boot sequence, signal handlers, and stdio MCP transport
scaffolding, so that the entire Prism service can be launched with `prism start` and
all subsystems (QueryEngine, MCP server, hot-reload watcher) are wired together in
production for the first time.

## Objective

Create the `prism-bin` crate at `crates/prism-bin/`. Implement `main.rs`, `cli.rs`
(clap 4.x), `boot.rs` (11-step boot orchestrator per ADR-022 §B), `signals.rs`
(SIGTERM + SIGHUP handlers), and all necessary `Cargo.toml` dependencies. Wire steps
1–6 fully (tracing, config, org-registry, sensor TOML, credentials, audit). Wire steps
7–11 as clearly annotated `todo!()` stubs that sibling stories fill in. The binary
must compile and `prism start` must execute to step 6 cleanly in integration tests.

Per ADR-022 §A this is the **only** `[[bin]]` target in the workspace. No `todo!()` or
`unimplemented!()` may remain in steps 1–6 production paths before this story merges.

---

## Behavioral Contracts

| BC ID | Title | Subsystem | Role in This Story |
|-------|-------|-----------|-------------------|
| BC-2.10.001 | rmcp ServerHandler Implementation | SS-10 | Binary entry point contract |
| BC-2.10.006 | Stdio Transport | SS-10 | Stdio transport scaffold (step 9 stub) |
| BC-2.10.010 | Graceful Shutdown on SIGTERM/SIGINT | SS-10 | SIGTERM handler + clean-exit contract |
| BC-2.06.011 | ConfigManager initialization contract | SS-06 | Precondition for boot step 2 (prism.toml load + schema validation; exit 2 on failure) |
| BC-2.21.001 | OrgRegistry init bijective-resolution contract | SS-21 | Precondition for boot step 3 (org_id + org_slug pairs from config; exit 2 on failure) |
| BC-2.03.013 | CredentialStore init reference-validation-only contract | SS-03 | Precondition for boot step 5; carries no-leak invariant (reference-based only, no inline values; PermissionDenied → exit 5) |
| BC-2.05.012 | AuditEmitter init audit_buffer-CF-open + boot.audit.initialized emitted | SS-05 | Precondition for boot steps 6+; SOC 2 required; failure → exit 4 |
| BC-2.22.001 | Boot orchestration contract | SS-22 | Chains all 4 init BCs; defines ordered 11-step sequencing, exit-code map, and traffic gate (MCP blocked until step 8 completes) |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,500 |
| `crates/prism-bin/src/main.rs` (entry point, panic hook, tracing init) | ~800 |
| `crates/prism-bin/src/cli.rs` (clap 4.x subcommands, exit-code contract) | ~1,500 |
| `crates/prism-bin/src/boot.rs` (11-step orchestrator) | ~3,000 |
| `crates/prism-bin/src/signals.rs` (SIGTERM/SIGHUP tokio handlers) | ~1,200 |
| `crates/prism-bin/Cargo.toml` (new crate; dep list) | ~600 |
| `Cargo.toml` workspace member addition | ~200 |
| BC files (8 BCs: 3 existing + 5 boot BCs) | ~4,000 |
| Integration tests (boot-to-step-6 smoke) | ~2,500 |
| Total | ~17,300 |

Within the 30% context window budget (~40k tokens for a 128k-context agent).

---

## Tasks

1. Create `crates/prism-bin/` directory structure:
   ```
   crates/prism-bin/
     Cargo.toml
     src/
       main.rs
       cli.rs
       boot.rs
       signals.rs
       lib.rs   (re-exports for integration test access)
   ```

2. Add `crates/prism-bin` to workspace `Cargo.toml` `[workspace.members]`. This
   is the first and only `[[bin]]` target in the workspace per ADR-022 §A.

3. Implement `cli.rs` with clap 4.x `#[derive(Parser)]`:

   **Subcommands (minimum viable set):**
   ```
   prism start               -- Boot and serve (blocking)
   prism query <query-str>   -- Single PrismQL query → JSON stdout
   prism validate-config     -- Parse config; exit 0/2
   prism version             -- Print version + build metadata; exit 0
   ```

   **Exit-code contract (canonical per ADR-022 §A):**
   ```
   0 — success / clean shutdown
   1 — unhandled error (unexpected panic caught by hook)
   2 — config-invalid (TOML parse error, schema validation failure, credential ref failure)
   3 — sensor-fail (required sensor adapter failed to init)
   4 — internal-error (runtime invariant violation; RocksDB open failed)
   5 — permission-denied (credential store access denied at boot)
   ```
   Document exit codes in `--help` output via clap `long_about`.

4. Implement `main.rs` entry point:
   - Register custom panic hook: `std::panic::set_hook(Box::new(|info| { tracing::error!(...); std::process::exit(1); }))`. Panic hook must emit a `tracing::error!` log before exiting.
   - Call tracing init (Step 1) FIRST before any other code.
   - Dispatch to `run(cli).await` and map top-level errors to correct exit codes.
   - Use `tokio::main` multi-thread runtime (AD-013).

5. Implement `boot.rs` — the 11-step boot orchestrator per ADR-022 §B:

   **Steps 1–6 (FULLY IMPLEMENTED — no `todo!()`):**

   Step 1 [BLOCKING]: Initialize tracing subscriber.
   - `tracing_subscriber::registry().with(EnvFilter::from_default_env()).with(fmt_layer).init()`
   - Format: JSON if `PRISM_LOG_FORMAT=json`; pretty otherwise.
   - First log line: `tracing::info!("Prism v{}", env!("CARGO_PKG_VERSION"))`
   - Failure → `eprintln!("Failed to init tracing: {err}"); std::process::exit(4)`

   Step 2 [BLOCKING]: Load prism.toml from `PRISM_CONFIG_DIR` (default `~/.prism/`).
   - Deserialize via serde + toml crate.
   - Failure → `exit(2)` with structured error message naming missing/invalid fields.

   Step 3 [BLOCKING]: Construct `OrgRegistry` from config (org_id + org_slug pairs per AD-006).
   - Call `OrgRegistry::from_config(&config)`.
   - Failure → `exit(2)`.

   Step 4 [BLOCKING]: Load sensor TOML specs.
   - Call `parse_spec_directory(config.spec_dir)` → `ConfigSnapshot`.
   - Wrap in `Arc::new(ArcSwap::from_pointee(ConfigManager::new(snapshot)))`.
   - This is the **first production call site** for `parse_spec_directory` per ADR-022 §C.
   - Failure → `exit(2)`.

   Step 5 [BLOCKING]: Initialize credential store.
   - Call `CredentialStore::open(&config)`.
   - Resolve all credential refs declared in sensor specs (verify access only; no inline values per AD-017).
   - Permission-denied → `exit(5)`.
   - Config-invalid ref → `exit(2)`.

   Step 6 [BLOCKING]: Initialize audit subsystem.
   - Call `AuditEmitter::new(storage.clone())`.
   - Open `audit_buffer` RocksDB CF.
   - Failure → `exit(4)` (audit required for SOC 2; non-negotiable).

   **Steps 7–11 (annotated `todo!()` stubs for sibling stories):**

   Step 7 [BLOCKING]:
   ```rust
   // TODO(S-WAVE5-PREP-01/S-3.02-FOLLOWUP-RUNTIME): Open RocksDB + register internal tables.
   // Resolved by S-3.02-FOLLOWUP-RUNTIME (register_internal_tables) and
   // AdapterRegistry::init_registry_for_org from loaded sensor specs.
   todo!("S-WAVE5-PREP-01 step 7 — RocksDB + internal-tables — resolved by S-3.02-FOLLOWUP-RUNTIME")
   ```

   Step 8 [BLOCKING → BACKGROUND]:
   ```rust
   // TODO(S-WAVE5-PREP-01/S-3.02-FOLLOWUP-RUNTIME): Construct QueryEngine + WriteExecutor.
   // QueryEngine::execute is todo!() resolved by S-3.02-FOLLOWUP-RUNTIME.
   todo!("S-WAVE5-PREP-01 step 8 — QueryEngine/WriteExecutor — resolved by S-3.02-FOLLOWUP-RUNTIME")
   ```

   Step 9 [BACKGROUND]:
   ```rust
   // TODO(S-WAVE5-PREP-01/S-5.01-FOLLOWUP-MCP-BOOT): Start PrismServer stdio transport.
   // PrismServer struct does not exist yet — resolved by S-5.01-FOLLOWUP-MCP-BOOT.
   todo!("S-WAVE5-PREP-01 step 9 — MCP server boot — resolved by S-5.01-FOLLOWUP-MCP-BOOT")
   ```

   Step 10 [BACKGROUND]:
   ```rust
   // TODO(S-WAVE5-PREP-01/S-1.12-FOLLOWUP): Install HotReloadWatcher.
   // HotReloadWatcher::start is unimplemented!() — resolved by S-1.12-FOLLOWUP.
   // Non-fatal: boot continues if watcher fails; emit degraded-mode audit entry.
   todo!("S-WAVE5-PREP-01 step 10 — hot-reload watcher — resolved by S-1.12-FOLLOWUP")
   ```

   Step 11 [BACKGROUND]:
   ```rust
   // TODO(S-WAVE5-PREP-01/signals.rs): Install tokio signal handlers.
   // SIGTERM → graceful shutdown. SIGHUP → manual config reload.
   // See signals.rs for the handler implementations — wire them here.
   ```
   Note: signal handler registration itself is NOT a `todo!()` — it is implemented in
   `signals.rs`. What is deferred is the SIGHUP reload path which requires steps 7-10.

6. Implement `signals.rs`:
   ```rust
   // SIGTERM handler: drain in-flight queries, close MCP server, flush audit, close RocksDB, exit 0.
   // SIGHUP handler: trigger config reload (same path as HotReloadWatcher).
   pub async fn install_sigterm_handler(shutdown_tx: tokio::sync::broadcast::Sender<()>) { ... }
   pub async fn install_sighup_handler(reload_tx: tokio::sync::mpsc::Sender<()>) { ... }
   ```
   SIGTERM handler MUST flush the audit buffer before exit 0.
   SIGHUP handler sends on reload_tx channel; boot.rs step 10 consumer is a `todo!()` until
   S-1.12-FOLLOWUP wires it to HotReloadWatcher.

7. Write `Cargo.toml` for `prism-bin` with required dependencies:
   ```toml
   [package]
   name = "prism-bin"
   version = "0.1.0"
   edition = "2021"

   [[bin]]
   name = "prism"
   path = "src/main.rs"

   [dependencies]
   prism-mcp = { workspace = true }
   prism-query = { workspace = true }
   prism-spec-engine = { workspace = true }
   prism-storage = { workspace = true }
   prism-audit = { workspace = true }
   prism-credentials = { workspace = true }
   prism-core = { workspace = true }
   clap = { workspace = true, features = ["derive"] }
   tokio = { workspace = true, features = ["full"] }
   tracing = { workspace = true }
   tracing-subscriber = { workspace = true, features = ["env-filter", "json"] }
   arc-swap = { workspace = true }
   anyhow = { workspace = true }
   ```
   Add `clap = "4"` to `[workspace.dependencies]` in root `Cargo.toml` (no existing pin).

8. Write integration tests in `crates/prism-bin/tests/boot_tests.rs`:
   - Test: `prism version` exits 0 and prints version string.
   - Test: `prism validate-config --config-dir <test-fixtures>` exits 0 for valid config.
   - Test: `prism validate-config --config-dir <missing>` exits 2 with error text.
   - Test: `prism validate-config --config-dir <invalid-toml>` exits 2.
   - Test: boot steps 1–6 complete with valid test fixture config (without steps 7–11).

---

## Acceptance Criteria

**AC-1:** Given a workspace build with `cargo build -p prism-bin`, Then the binary
compiles without errors and `prism --help` lists all 4 subcommands with their descriptions
and exit codes.
(traces to BC-2.10.001 postcondition — ServerHandler binary entry point)

**AC-2:** Given `prism version`, When invoked, Then it prints `prism X.Y.Z` (semantic
version from `Cargo.toml`) to stdout and exits 0.
(traces to BC-2.10.006 postcondition — stdio transport scaffold present)

**AC-3:** Given `prism validate-config --config-dir <valid-fixtures>`, When the config
directory contains valid `prism.toml` and sensor spec TOMLs, Then exit code is 0 and
stdout contains a redacted summary of loaded sensors.
(traces to BC-2.06.011 postcondition — ConfigManager initialization succeeds on valid config)

**AC-4:** Given `prism validate-config --config-dir <dir-with-invalid-toml>`, When
`prism.toml` has a TOML syntax error, Then exit code is 2 and stderr contains the
line number and field name of the parse error.
(traces to BC-2.06.011 postcondition — ConfigManager init failure maps to exit 2; traces to BC-2.22.001 exit-code map — config-invalid = exit 2)

**AC-5:** Given `prism start` with a valid config, When boot step 1 (tracing init)
completes, Then the first structured log line emitted is `{"level":"INFO","message":"Prism vX.Y.Z",...}`.
(traces to BC-2.22.001 invariant — boot orchestration requires tracing init as step 1 before all other steps in the ordered sequence)

**AC-6:** Given `prism start` and a SIGTERM is delivered to the process, When the
SIGTERM handler fires, Then the process emits a `tracing::info!("Received SIGTERM — shutting down")` log entry and exits 0.
(traces to BC-2.10.010 postcondition — graceful shutdown on SIGTERM)

**AC-7:** Given `prism start` where the credential store returns PermissionDenied during
step 5, Then the process exits with code 5 (not 1 or 4).
(traces to BC-2.03.013 postcondition — CredentialStore PermissionDenied maps to exit 5; traces to BC-2.22.001 exit-code map — permission-denied = exit 5)

**AC-8:** Given `prism start` where the audit subsystem fails to open RocksDB in step 6,
Then the process exits with code 4.
(traces to BC-2.05.012 postcondition — AuditEmitter init failure maps to exit 4; traces to BC-2.22.001 exit-code map — internal-error = exit 4)

**AC-9:** Given `prism start` where the config declares zero org entries in the
OrgRegistry section, Then the process exits with code 2 and stderr contains "Config
must declare at least one org".
(traces to BC-2.21.001 postcondition — OrgRegistry init fails on empty org list; traces to BC-2.22.001 exit-code map — config-invalid = exit 2)

**AC-10:** Given `prism start` has completed boot steps 1–7 (RocksDB open) but has
not yet completed step 8 (QueryEngine ready), When an MCP tool call arrives on the
stdio transport, Then the binary does not service the request until step 8 completes
(traffic gate enforced by BC-2.22.001).
(traces to BC-2.22.001 invariant — MCP traffic gate must block until step 8 completes)

**AC-11:** Given any boot sequence step (1–6) completes, When any subsequent step fails,
Then no data corruption occurs (step failures are clean exits, not panics), and no step
leaves permanent state corruption. No `todo!()`, `unimplemented!()`, or
`panic!("stub")` may remain in the steps 1–6 production code paths before merge.

**AC-12:** Given a panic occurs anywhere in the process (injected for test), When the
custom panic hook fires, Then a `tracing::error!` log is emitted before the process
exits with code 1.
(traces to BC-2.10.010 — all exits are clean and observable)

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Binary entry point + panic hook | `prism-bin` (Layer 4) | Effectful |
| CLI arg parsing | `prism-bin/src/cli.rs` (clap) | Pure |
| Boot orchestrator | `prism-bin/src/boot.rs` | Effectful (I/O at every step) |
| Signal handlers | `prism-bin/src/signals.rs` | Effectful |
| Config load | `prism-spec-engine` (SS-06, SS-16) | Mixed |
| Credential store | `prism-credentials` (SS-03) | Effectful |
| Audit emitter | `prism-audit` (SS-05) | Effectful |

`prism-bin` is Layer 4 (binary) in the workspace layered architecture per
`architecture/module-decomposition.md`. It depends on prism-mcp (Layer 3),
prism-query, prism-spec-engine (Layer 2), prism-storage, prism-audit,
prism-credentials (Layer 1), and prism-core (Layer 0). No Layer 2+ crates may
depend on prism-bin.

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `prism-bin` (all modules) | Effectful | Binary entry point; all boot steps are I/O-heavy. Effectful-shell per `architecture/purity-boundary-map.md`. |
| `cli.rs` (arg parsing only) | Pure | Clap parsing is pure; no I/O until subcommand dispatch. |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `prism-bin` is the ONLY `[[bin]]` target in the workspace | ADR-022 §A | Confirmed in Cargo.toml; CI `just check` enforces no secondary bin targets |
| Boot steps 1–8 MUST be ordered and blocking — no parallelism across steps | ADR-022 §B | Code review; integration test verifies step ordering via log timestamps |
| Steps 1–6 MUST have zero `todo!()`/`unimplemented!()` before merge | POL-12 (production_stub_residue_blocks_merge) | CI `just check` + grep stub sweep in PR gate |
| Steps 7–11 stubs MUST include the resolving story ID in their message | ADR-022 §G annotation requirement | Code review |
| Panic hook MUST use `tracing::error!` before exit — no raw `eprintln!` as sole output | ADR-022 §A (logging spec) | AC-12 integration test |
| Exit codes 0–5 are the canonical contract — no other exit codes may be added without ADR | ADR-022 §A | Code review; CLI `--help` documents all 6 codes |
| `PRISM_CONFIG_DIR` env var MUST override the default config directory | ADR-022 §B step 2 | AC-3/AC-4 integration tests use this env var |

**Forbidden Dependencies:** `prism-bin` MUST NOT be a dependency of any library crate
in the workspace. If any library crate gains a dependency on `prism-bin`, the build
MUST fail (circular dependency).

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| clap | 4.x (new workspace pin; implementer sets exact) | CLI argument parsing with `#[derive(Parser)]` |
| tokio | 1.x (workspace) | Multi-threaded async runtime (AD-013); `tokio::main` macro |
| tracing | 0.1.x (workspace) | Structured logging throughout boot sequence |
| tracing-subscriber | 0.3.x | EnvFilter + JSON/pretty format; initialized in step 1 |
| arc-swap | 1.x (workspace) | `Arc<ArcSwap<ConfigManager>>` for hot reload |
| anyhow | 1.x (workspace) | Error propagation within boot.rs before domain errors take over |
| prism-spec-engine | workspace | `parse_spec_directory`, `ConfigManager` |
| prism-credentials | workspace | `CredentialStore` |
| prism-audit | workspace | `AuditEmitter` |
| prism-storage | workspace | `RocksDbBackend` (step 7 stub; real in S-3.02-FOLLOWUP) |
| prism-query | workspace | `QueryEngine`, `WriteExecutor` (step 8 stub) |
| prism-mcp | workspace | `PrismServer::serve_stdio` (step 9 stub) |
| prism-core | workspace | `OrgRegistry`, `PrismError` |

**MSRV:** Rust stable per `rust-toolchain.toml` (currently 1.85+).

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-bin/Cargo.toml` | Create | New crate manifest; `[[bin]]` entry; all dependencies |
| `crates/prism-bin/src/main.rs` | Create | Entry point; panic hook; tracing init; `#[tokio::main]` |
| `crates/prism-bin/src/cli.rs` | Create | clap CLI definition: 4 subcommands + exit-code docs |
| `crates/prism-bin/src/boot.rs` | Create | 11-step boot orchestrator (steps 1–6 real; 7–11 annotated `todo!()`) |
| `crates/prism-bin/src/signals.rs` | Create | SIGTERM + SIGHUP tokio signal handlers |
| `crates/prism-bin/src/lib.rs` | Create | Re-exports for integration test access |
| `crates/prism-bin/tests/boot_tests.rs` | Create | Integration tests: version, validate-config exit codes, step-1-6 smoke |
| `Cargo.toml` (workspace root) | Modify | Add `crates/prism-bin` to `[workspace.members]`; add `clap = "4"` to `[workspace.dependencies]` |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `PRISM_CONFIG_DIR` points to non-existent directory | Exit 2 with "Config directory not found: {path}" |
| EC-002 | `prism.toml` exists but has TOML syntax error | Exit 2 with line + context from toml parse error |
| EC-003 | Credential store returns PermissionDenied for any ref | Exit 5 (not exit 2 or 4) |
| EC-004 | Audit subsystem RocksDB CF open fails | Exit 4; "Audit subsystem init failed: {err}" |
| EC-005 | SIGTERM arrives during step 4 (sensor TOML load) | Handler registered only at step 11; SIGTERM before that causes unhandled signal default (OS SIGTERM kills process) — this is acceptable for MVP |
| EC-006 | OrgRegistry gets zero org entries from config | Exit 2; "Config must declare at least one org" |
| EC-007 | prism-bin binary invoked with unknown subcommand | Clap exits 2 with usage message (clap default behavior) |
| EC-008 | Panic in boot step 2–6 before panic hook is registered | Panic hook is registered in step 1 (main.rs entry) before any other code — step 1 panic goes to stderr with OS default |

---

## Previous Story Intelligence

N/A — first story for `prism-bin`; the crate does not yet exist. No predecessor
intelligence to carry forward.

Key context from existing stories:
- `prism-spec-engine` (S-1.11/S-1.12): `parse_spec_directory` and `ConfigManager::new`
  are real implementations (not stubs). This story makes the first production call site.
- `prism-credentials` (S-1.06/S-1.07): `CredentialStore` trait is implemented with
  keyring and AES-file backends.
- `prism-audit` (S-2.02): `AuditEmitter` is real; `audit_buffer` CF is declared in AD-004.
- `prism-storage` (S-2.01): `RocksDbBackend::open` is implemented.

---

## Dev Notes

- The `clap` workspace pin is new (no existing pin per ADR-022 §A). Implementer sets
  exact version in root `Cargo.toml`. Use latest stable 4.x at implementation time.
- The `start` subcommand will block indefinitely (serving MCP traffic) once steps 7–11
  are filled by sibling stories. For this story, it exits after step 6 with a log message:
  `tracing::warn!("Steps 7-11 are not yet implemented — exiting (prism-bin chassis only)")`.
  This is the partial-merge state.
- `prism query <query-str>` similarly is a stub that prints a `todo!()` message and
  exits 4 until S-3.02-FOLLOWUP-RUNTIME fills QueryEngine::execute.
- Integration tests run `prism` as a subprocess using `std::process::Command`. Do NOT
  start a `#[tokio::test]` runtime in the same process — binary tests require subprocess
  invocation to get correct exit codes.
- The panic hook must be installed before `tracing::subscriber::set_global_default` to
  avoid a race. Use `std::panic::set_hook` in `main()` before `tracing_subscriber::init()`.
  If tracing isn't initialized yet when a panic fires, the hook falls back to `eprintln!`.

## Graduation

N/A — this is a new story with no predecessor `partial-merge` story to graduate.

---

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 1.0 | Bundle-B-Phase-B-1 | 2026-05-08 | story-writer | Initial story creation from ADR-022 §G seed (Story 1). |
| 1.1 | Bundle-B-Phase-B-1b | 2026-05-08 | story-writer | BC back-fill: replaced 4 `[NEW-BC-NEEDED]` placeholders with authored BC IDs (BC-2.06.011, BC-2.21.001, BC-2.03.013, BC-2.05.012, BC-2.22.001). Updated frontmatter `behavioral_contracts`, `anchor_bcs`, `anchor_subsystem`, and `inputs`. Propagated BC traces to AC-3–AC-8 per `bc_array_changes_propagate_to_body_and_acs` policy. Added AC-9 (BC-2.21.001 OrgRegistry), AC-10 (BC-2.22.001 traffic gate); renumbered original AC-10 to AC-12. Token budget updated to 8 BCs (~17,300 tokens). |
| 1.2 | Bundle-B-Phase-B-1b | 2026-05-08 | state-manager | status draft → ready per orchestrator authorization; Spec-First Gate S-7.01 satisfied (BC anchors back-filled, every AC traces to a BC, POL-12 compliance preserved at AC-11). |
