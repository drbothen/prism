---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-05-08T00:00:00Z
phase: 3
origin: greenfield
extracted_from: null
subsystem: "SS-22"
capability: "CAP-034"
lifecycle_status: draft
introduced: "2026-05-08"
modified: [D-319-post-merge-state-burst, D-454, D-469, fix-burst-6-stage-1]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
anchored_stories: [S-WAVE5-PREP-01]
verifying_vps: []
crates: [prism-bin]
inputs:
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
  - .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
  - .factory/specs/architecture/module-decomposition.md
  - .factory/specs/behavioral-contracts/BC-2.06.011-config-load-on-startup.md
  - .factory/specs/behavioral-contracts/BC-2.21.001-org-registry-init.md
  - .factory/specs/behavioral-contracts/BC-2.03.013-credential-store-init.md
  - .factory/specs/behavioral-contracts/BC-2.05.012-audit-subsystem-init.md
  - .factory/specs/behavioral-contracts/BC-2.17.007-plugin-manifest-schema-validation.md
input-hash: "d852024"
traces_to: ["CAP-034"]
---

# BC-2.22.001: Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate

## Description

This BC specifies the cross-cutting orchestration responsibility of `prism-bin`'s Process
Lifecycle subsystem (SS-22): the strict sequential ordering of the four subsystem init contracts
plus the plugin-load intercalation step (step 7.5, per ADR-023 §C4), the canonical exit-code
mapping for each failure class, and the hard pre-traffic gate that prevents any MCP request from
being accepted before all init steps complete. It does NOT re-specify the internal behavior of
each subsystem's init — those are separately specified in BC-2.06.011 (config), BC-2.21.001
(org), BC-2.03.013 (credentials), BC-2.05.012 (audit), and BC-2.17.007 (plugin manifest validation).

The boot orchestration contract makes `prism-bin`'s startup behavior deterministic and testable
as a unit: given any combination of subsystem failures, the exit code, error message, and
step-never-started guarantees are fully predictable from this single contract. This BC satisfies
ADR-022 §A/§B and is the behavioral backing for AD-014 (Process-level RSS watchdog) — the
watchdog can only be reliably applied after the gate in step 8 has passed.

## Preconditions

- The Prism binary has been invoked with the `start` subcommand (ADR-022 §A)
- Boot step 1 (tracing subscriber init) has completed — the `tracing` subscriber is active
  and all subsequent steps can emit structured log output

## Postconditions

**Happy path (all 4 init contracts satisfied in order):**
- BC-2.06.011 (ConfigManager) is satisfied: `PrismConfig` handle is valid — step 2 complete
- BC-2.21.001 (OrgRegistry) is satisfied: `OrgRegistry` handle is valid — step 3 complete
- BC-2.03.013 (CredentialStore) is satisfied: `CredentialStore` handle is valid — step 5 complete
- BC-2.05.012 (AuditEmitter) is satisfied: `AuditEmitter` handle is valid and sentinel persisted — step 6 complete
- Steps 7 and 8 (sensor spec init, QueryEngine + WriteExecutor construction) complete without error
- The MCP server binds to stdio / accepts the MCP handshake ONLY AFTER step 8 is confirmed complete
- The process is in steady state: all subsystem handles available, traffic gate open

**Failure path — ConfigManager fails (BC-2.06.011 unsatisfied):**
- Steps 3–9 never begin
- Process exits with code **2** (config-invalid per ADR-022 §A)

**Failure path — OrgRegistry fails (BC-2.21.001 unsatisfied):**
- Steps 4–9 never begin
- Process exits with code **2** (config-invalid per ADR-022 §A)

**Failure path — CredentialStore fails (BC-2.03.013 unsatisfied, permission denied):**
- Steps 6–9 never begin
- Process exits with code **5** (permission-denied per ADR-022 §A)

**Failure path — CredentialStore fails (BC-2.03.013 unsatisfied, config-invalid ref):**
- Steps 6–9 never begin
- Process exits with code **2** (config-invalid per ADR-022 §A)

**Failure path — AuditEmitter fails (BC-2.05.012 unsatisfied):**
- Steps 7–9 never begin
- Process exits with code **4** (internal-error per ADR-022 §A)

**Failure path — any step 7/8 failure:**
- Step 9 (MCP bind) never begins
- Process exits with the exit code appropriate to the failing subsystem (per ADR-022 §A)

**Plugin-load step (step 7.5) — happy path:**
- `PluginRuntime::load_all_plugins` is called after step 7 (storage init) completes and before step 8 (QueryEngine + WriteExecutor construction)
- Each `.prx` file discovered in the plugin directory is loaded, manifest-validated (BC-2.17.007), and registered
- For every plugin loaded: audit event `plugin_load_unsigned` emitted at WARN with fields `plugin_path` and `plugin_hash` (sha256) — this is v1.0 behavior per ADR-023 §C4 (plugin signing deferred to v1.0+N)
- Boot proceeds to step 8 after `load_all_plugins` returns

**Plugin-load step (step 7.5) — `PRISM_DISABLE_PLUGIN_LOAD=1` escape valve:**
- When the environment variable `PRISM_DISABLE_PLUGIN_LOAD=1` is set, the plugin-load step is skipped entirely (no `.prx` files are loaded)
- Audit event `plugin_load_disabled_via_envvar` is emitted at WARN before the step is skipped, recording that plugin loading was administratively disabled
- Boot proceeds to step 8 normally; the pre-traffic gate condition for plugin-load is satisfied by the audited-disable path
- Rationale: this is the emergency escape valve per ADR-023 §C4; it is NOT a silent skip — the WARN audit event preserves observability and DI-004 audit completeness

**Plugin-load step (step 7.5) — individual plugin manifest failure:**
- If a single plugin's manifest fails validation (E-PLUGIN-013 / E-PLUGIN-014 / E-PLUGIN-015 / E-PLUGIN-016 per BC-2.17.007), that plugin is rejected and an ERROR log is emitted naming the plugin path and validation failure
- The remaining plugins continue loading (`load_all_plugins` does not abort on the first failure — it returns `Ok(N-success)` after processing all plugins)
- Boot proceeds to step 8; partial plugin load is not a fatal boot error in v1.0

**Plugin-load step (step 7.5) — fatal plugin-load failure (non-manifest):**
- If `PluginRuntime` construction or the plugin directory scan fails with an I/O error that prevents the step from completing, the process exits with code **4** (internal-error per ADR-022 §A)
- Steps 8 and 9 never begin
- Rationale: plugin-load failures that prevent the runtime from initializing are analogous to audit init failures (step 6 exit 4) — they signal internal infrastructure failure, not configuration error

## Sequencing Invariant

The four subsystem init contracts MUST be satisfied in this exact order, each blocking before
the next begins (ADR-022 §B). PREREQ-D (S-PLUGIN-PREREQ-D) inserts a plugin-load step between
canonical step 7 (storage) and canonical step 8 (query-engine), designated step 7.5 in this BC.
This fractional numbering avoids cascading renumber of ADR-022 §B, boot.rs function names
(`step9_start_mcp_server`), STATE.md narrative, and the ADR-022 canonical step definitions —
the canonical steps remain 2 through 9; step 7.5 is the plugin-load intercalation (ADR-023 §C4).

```
Step 2: BC-2.06.011 (ConfigManager init)
           ↓ blocks
Step 3: BC-2.21.001 (OrgRegistry init)
           ↓ blocks
Step 5: BC-2.03.013 (CredentialStore init)       ← step 4 (sensor spec load) is between 3 and 5
           ↓ blocks
Step 6: BC-2.05.012 (AuditEmitter init)
           ↓ blocks
Step 7: StorageEngine (RocksDB primary CF set) + InternalTablesProvider init
           ↓ blocks
Step 7.5: PluginRuntime plugin-load (ADR-023 §C4)   ← PREREQ-D intercalation
           OR PRISM_DISABLE_PLUGIN_LOAD=1 audited-disable path (see postconditions above)
           ↓ blocks
Step 8: QueryEngine + WriteExecutor construction
           ↓ blocks
Step 9: MCP server stdio bind — TRAFFIC GATE OPEN
```

No step in this sequence may begin concurrently with or before its predecessor completes
successfully. This is a strict sequential dependency, not a DAG.

## Exit-Code Map (ADR-022 §A Canonical Table)

| Failure Class | Source BC | Exit Code | Code Name |
|---------------|-----------|-----------|-----------|
| Config file missing / parse error / schema error | BC-2.06.011 | **2** | config-invalid |
| OrgRegistry construction failure (empty list, duplicate, malformed slug) | BC-2.21.001 | **2** | config-invalid |
| CredentialStore: unresolvable ref / malformed ref | BC-2.03.013 | **2** | config-invalid |
| CredentialStore: permission denied / backend unavailable | BC-2.03.013 | **5** | permission-denied |
| AuditEmitter: RocksDB CF open failure / WAL unwriteable / sentinel write failure | BC-2.05.012 | **4** | internal-error |
| Plugin-load fatal failure (PluginRuntime construction failure / plugin directory I/O error) | ADR-023 §C4 | **4** | internal-error |
| Plugin manifest validation failure (E-PLUGIN-013/014/015/016 — single plugin) | BC-2.17.007 | (no exit — partial load; remaining plugins continue) | n/a |
| QueryEngine / WriteExecutor construction failure (step 8) | — | **4** | internal-error |
| Successful startup | — | (no exit; steady-state) | — |
| Graceful shutdown (SIGTERM/SIGINT) | — | **0** | success |

Exit codes 1, 3, and 6+ are not used by the boot sequence. Exit code 1 is reserved for
unhandled panics (process::exit is NOT called with 1 deliberately).

Plugin-load exit-code rationale: fatal plugin-load failures (step 7.5 non-manifest I/O error)
inherit `exit(4)` (internal-error) because the PluginRuntime is internal infrastructure — its
failure is analogous to audit init failure at step 6, not a configuration error. Individual
manifest validation failures (E-PLUGIN-013..016) do NOT cause `exit(4)` because `load_all_plugins`
continues loading remaining plugins; the n-1 survivor model means partial load is not fatal.
This matches ADR-023 §C4 v1.0 behavior where plugin signing failures are non-fatal at boot.

## Pre-Traffic Gate Invariant

The MCP server MUST NOT bind to stdio or accept the MCP initialization handshake until ALL of
the following conditions hold simultaneously (ADR-022 §B step 9 gate):

1. BC-2.06.011 postconditions are satisfied (PrismConfig handle valid)
2. BC-2.21.001 postconditions are satisfied (OrgRegistry handle valid and bijective)
3. BC-2.03.013 postconditions are satisfied (CredentialStore handle valid, no values in memory)
4. BC-2.05.012 postconditions are satisfied (AuditEmitter handle valid, boot.audit.initialized persisted)
5. Step 7 complete: StorageEngine (RocksDB primary CF set) and InternalTablesProvider initialized
6. Step 7.5 complete: plugin-load step completed successfully (PluginRuntime loaded all valid
   plugins and emitted per-plugin audit events) OR `PRISM_DISABLE_PLUGIN_LOAD=1` was recognized
   as an audited administrative disable (audit event `plugin_load_disabled_via_envvar` emitted
   at WARN). Either outcome satisfies this gate condition. A fatal PluginRuntime construction
   failure exits the process before this condition is reached. (ADR-023 §C4)
7. Step 8 complete: QueryEngine and WriteExecutor constructed and ready to accept work

The gate is a hard requirement — there is no "partially started" mode where the MCP server
accepts some requests while boot is in progress. The gate is the single synchronization point
between the boot sequence and the MCP request loop.

## AD-014 Compliance Note

This BC satisfies the behavioral precondition for AD-014 (Process-level RSS watchdog): the
watchdog thread is started as part of step 8, after the gate conditions above are met. Because
all subsystem handles are available and validated before the watchdog starts, the watchdog can
safely reference them without initialization races.

The orchestration sequence defined in this BC is what makes the RSS watchdog's view of the
process state reliable: if the watchdog fires, it can assume all 4 init contracts are satisfied.

## Invariants

- The 4 init contracts are satisfied strictly in the order: BC-2.06.011 → BC-2.21.001 →
  BC-2.03.013 → BC-2.05.012. No reordering is permitted even if a reordering appears safe.
- The plugin-load step (step 7.5) ALWAYS runs between step 7 (storage) and step 8 (query-engine),
  unless `PRISM_DISABLE_PLUGIN_LOAD=1` is set. The disable path is never silent — an audit event
  MUST be emitted before the step is bypassed (DI-004 audit completeness).
- Each step's failure causes the process to exit IMMEDIATELY with the mapped exit code — no
  cleanup of subsequent steps is needed because they have not yet begun.
- The MCP server bind (step 9) is the ONLY mechanism by which the process enters steady state.
  There is no other path to accepting MCP traffic.
- POL-15 (fail-closed for writes) is a steady-state property that applies after the gate passes.
  This BC's orchestration contract is the prerequisite for POL-15 to apply.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-22-001-001 | First step (BC-2.06.011) fails | Exit 2; steps 3–9 never begin; no partial state |
| EC-22-001-002 | BC-2.21.001 fails after BC-2.06.011 succeeds | Exit 2; PrismConfig handle is available but unused; steps 4–9 never begin |
| EC-22-001-003 | BC-2.03.013 fails with permission-denied after steps 2 and 3 succeed | Exit 5; PrismConfig + OrgRegistry handles available but MCP never starts |
| EC-22-001-004 | BC-2.05.012 fails after steps 2, 3, and 5 succeed | Exit 4; audit sentinel never persisted; MCP never starts |
| EC-22-001-005 | All 4 init BCs succeed but step 8 (QueryEngine) fails | Exit 4; MCP never starts; audit sentinel IS persisted (step 6 completed) |
| EC-22-001-006 | SIGTERM arrives during boot (between steps) | Graceful shutdown; in-progress step finishes; exit 0 or the step's mapped exit code if the step was in a failure path |
| EC-22-001-007 | `prism start` called a second time while first is still booting | OS process model prevents shared stdio; second invocation may fail on RocksDB LOCK (exit 4) |

## Canonical Test Vectors

| ID | Scenario | Injected Failure | Expected Exit Code | Steps Completed |
|----|----------|-----------------|-------------------|-----------------|
| TV-22-001-001 | Full happy path | None | (steady state, no exit) | All steps 2–8 + gate open |
| TV-22-001-002 | Config missing | BC-2.06.011 fails (no prism.toml) | 2 | Step 1 only |
| TV-22-001-003 | OrgRegistry fails | BC-2.21.001 fails (duplicate slug) | 2 | Steps 1–2 |
| TV-22-001-004 | Cred permission denied | BC-2.03.013 fails (keyring locked) | 5 | Steps 1–3, 4 |
| TV-22-001-005 | Cred ref unresolvable | BC-2.03.013 fails (missing ref) | 2 | Steps 1–3, 4 |
| TV-22-001-006 | Audit CF open fails | BC-2.05.012 fails (RocksDB LOCK) | 4 | Steps 1–3, 4, 5 |
| TV-22-001-007 | QueryEngine fails | Step 8 internal error | 4 | Steps 1–6 (audit sentinel persisted) |
| TV-22-001-008 | Verify gate: no MCP before step 8 | Inject artificial delay in step 7; probe stdio | No MCP handshake response | Gate confirmed |

## Test Strategy

Integration tests in `crates/prism-bin/tests/boot_tests.rs`:

**Sequencing tests:**
- `test_BC_2_22_001_happy_path_sequencing` — assert all 8 steps' log lines appear in correct
  order in the subprocess output (grep for step-N log patterns)
- `test_BC_2_22_001_step2_failure_blocks_step3` — inject config failure; assert step 3 log
  line never appears in output and exit code is 2
- `test_BC_2_22_001_step3_failure_blocks_step5` — inject org failure; assert step 5 log line
  never appears

**Exit-code map tests:**
- `test_BC_2_22_001_exit_code_config_error` — assert exit 2 for BC-2.06.011 failure
- `test_BC_2_22_001_exit_code_org_error` — assert exit 2 for BC-2.21.001 failure
- `test_BC_2_22_001_exit_code_cred_permission` — assert exit 5 for BC-2.03.013 permission-denied
- `test_BC_2_22_001_exit_code_cred_invalid_ref` — assert exit 2 for BC-2.03.013 config-invalid ref
- `test_BC_2_22_001_exit_code_audit_failure` — assert exit 4 for BC-2.05.012 failure

**Traffic gate test:**
- `test_BC_2_22_001_no_mcp_before_gate` — use a mock step-7/8 that sleeps 500ms; probe the
  subprocess's stdout for MCP protocol bytes during the sleep; assert no MCP bytes appear
  before the sleep completes and the step-8-complete log line appears

All integration tests use fixture configs under `crates/prism-bin/fixtures/config/`. Failure
injection uses environment variables (e.g., `PRISM_TEST_INJECT_FAIL_STEP=3`) gated behind
`#[cfg(test)]` in the boot sequence.

## Verification Properties

A VP for boot-sequence ordering is warranted. Proposed:
- **VP-NNN (Boot Sequencing Invariant):** A proptest or Kani property that models the boot
  state machine as a sequence of Result<(), BootError> values (one per step) and asserts that:
  (a) if step N returns Err, steps N+1..9 are never called, and (b) if all steps return Ok,
  the gate function is called exactly once. This is a pure-function model of the orchestration
  logic and is amenable to Kani if the boot state machine is extracted into a pure core module.

Flag to implementer: extract the boot step sequencer into a testable pure function
`run_boot_sequence(steps: &[BootStep]) -> Result<(), BootError>` in `prism-bin/src/boot.rs`.
This enables both the Kani proof and the proptest property without subprocess overhead.

## Related BCs

- BC-2.06.011 — ConfigManager init (orchestrated by: this BC governs its position in the sequence)
- BC-2.21.001 — OrgRegistry init (orchestrated by: this BC governs its position in the sequence)
- BC-2.03.013 — CredentialStore init (orchestrated by: this BC governs its position in the sequence)
- BC-2.05.012 — AuditEmitter init (orchestrated by: this BC governs its position in the sequence)
- BC-2.17.007 — Plugin Manifest Schema Validation (step 7.5 invokes manifest validation as part of load_all_plugins; rejection behavior is governed by BC-2.17.007)

## Architecture Anchors

- `specs/architecture/decisions/ADR-022-production-runtime-wiring.md` §A exit-code contract (canonical source)
- `specs/architecture/decisions/ADR-022-production-runtime-wiring.md` §B boot step sequence (canonical source)
- `specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md` §C4 plugin-load step placement, PRISM_DISABLE_PLUGIN_LOAD escape valve, unsigned-plugin audit behavior
- `specs/architecture/module-decomposition.md` COMP-001 `prism-bin` subsystem SS-22 (Process Lifecycle)
- Architecture decision AD-014 (Process-level RSS watchdog — satisfied after this gate passes)

## Story Anchor

S-WAVE5-PREP-01 — prism-bin: Binary Chassis, CLI, and Boot Sequence

## VP Anchors

VP-NNN (Boot Sequencing Invariant) — proposed above; assigned by architect at VP registration time

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| Capability Anchor Justification | CAP-034 ("MCP Server & Transport") per capabilities.md §CAP-034 — this BC specifies the cross-cutting orchestration of the "startup (config load → RocksDB open → tool registration → stdio bind)" lifecycle that CAP-034 explicitly enumerates. The pre-traffic gate in this BC is the behavioral implementation of CAP-034's "stdio bind" step: the MCP server does not bind until all preceding init contracts are satisfied. No other capability describes the boot-sequence ordering and exit-code contract at the process level. |
| L2 Invariants | DI-004 (Audit Completeness): this BC's gate ensures AuditEmitter is ready before any MCP traffic begins, which is the prerequisite for DI-004 to hold. No DI directly covers boot ordering; the property is specified via ADR-022 §B. |
| ADR Source | ADR-022 §A (exit-code canonical table), §B (boot step sequence); ADR-023 §C4 (plugin-load step placement, escape valve, unsigned-plugin audit); AD-014 (RSS watchdog compliance note) |
| Priority | P0 |
| POL-12 Note | The production code path satisfying this BC MUST contain no `todo!()`, `unimplemented!()`, or `panic!("stub...")` before S-WAVE5-PREP-01 transitions to `merged`. |
| POL-15 Note | This BC is the prerequisite for POL-15 (fail-closed for writes). The orchestration gate ensures all subsystems are operational before POL-15's write-gate logic is reachable in steady state. |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | fix-burst-6-stage-1 | 2026-05-13 | product-owner | F-LP7-HIGH-003 closure: added plugin-load step 7.5 throughout — §Description (mentions BC-2.17.007 as new related subsystem), §Postconditions (new blocks for happy path / PRISM_DISABLE_PLUGIN_LOAD escape valve / individual manifest failure / fatal failure), §Sequencing Invariant (step 7.5 intercalation with 7.5-numbering rationale), §Exit-Code Map (plugin-load rows for fatal failure exit(4) and partial-load non-exit), §Pre-Traffic Gate Invariant (condition 6 plugin-load), §Invariants (plugin-load ordering + audited-disable rule), §Related BCs (BC-2.17.007), §Architecture Anchors (ADR-023 §C4), §Traceability ADR Source (ADR-023 §C4), inputs frontmatter (ADR-023 + BC-2.17.007). Step 7.5 fractional numbering avoids cascading renumber of ADR-022 §B and boot.rs function names. All plugin-load semantics sourced from ADR-023 §C4 as authoritative truth. |
| 1.3 | state(D-469/470/471) | 2026-05-13 | state-manager | F-LP4-MED-001 closure: `introduced: "redirect-option-d-2026-05-08"` → `introduced: "2026-05-08"` (extract embedded ISO date per POL-20 canonical format for fix-burst-introduced BCs). input-hash unchanged (d852024 — inputs not modified). |
| 1.2 | D-454 | 2026-05-12 | state-manager | ADR-025 sweep: `lifecycle: active` removed (ADR-025 retires `lifecycle:` field; `status:` is sole canonical authority). `status: accepted` corrected to `status: draft` (accepted is not a valid status taxonomy value). Template fields added: `extracted_from`, `lifecycle_status`, `introduced`, `modified`, `deprecated`, `deprecated_by`, `replacement`, `retired`, `removed`, `removal_reason`. `input-hash` computed from placeholder. Changelog reordered newest-first per hook discipline. BC-INDEX title synced to H1 (added "and"). |
| 1.1 | D-319-post-merge-state-burst | 2026-05-10 | state-manager | lifecycle draft → active per ADR-021 POL-14 (S-WAVE5-PREP-01 merged at develop@53b87961 PR #138 2026-05-10T00:55:49Z). First active BC under SS-22 (Process Lifecycle). |
| 1.0 | redirect-option-d-2026-05-08 | 2026-05-08 | product-owner | Initial authorship — Option (d) decomposition: 4 subsystem init contracts relocated to native SS; this BC holds only the cross-cutting orchestration (sequencing, exit-code map, traffic gate). |
