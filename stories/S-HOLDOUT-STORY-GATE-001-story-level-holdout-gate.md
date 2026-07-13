---
document_type: story
story_id: "S-HOLDOUT-STORY-GATE-001"
title: "Story-level holdout gate — generalize T13 audit harness into per-story holdout driver with scenario lifecycle"
wave: maintenance
epic_id: maintenance
priority: P1
status: draft
version: "0.1"
spec_version: "v0.1"
level: ops
producer: story-writer
timestamp: "2026-07-13"
modified: "2026-07-13"
input-hash: ""
inputs:
  - scripts/t13-preflight-audit.py
  - .factory/SESSION-HANDOFF.md
  - .factory/STATE.md
origin_finding: "T13 live-audit triage D-1715/D-1716 — live-audit FAILs that unit tests and LOCAL 3-CLEAN missed"
origin_cascade: "AUDIT-COVERAGE-001 D-hardening; D-1715 live audit; D-1716 human-approved design 2026-07-13"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
# No subsystem anchor: this is pipeline infrastructure (holdout evaluator driver + scenario
# lifecycle management). It touches scripts/ and .factory/holdout/ — not a product crate.
# The holdout-evaluator agent spec (vsdd-factory plugin) is separate from the prism-local
# implementation. No ARCH-INDEX subsystem owns scripts/ or holdout management.
crates_touched: []
target_module: "scripts/holdout-driver/"
behavioral_contracts: []
# BC status: pending PO authorship.
# This story defines process and infrastructure for the story-level holdout gate.
# No pre-existing BC governs per-story holdout scenario lifecycle, contamination-control routing,
# or the delivery-pipeline integration point. The product-owner must author a BC covering:
#   - holdout scenario authoring and storage (what the PO creates, where it lives)
#   - scenario consumption (how the evaluator accesses scenarios without leaking to implementer)
#   - contamination-control routing (OBSERVED BEHAVIOR ONLY — no scenario text to upstream agents)
#   - blocking semantics (gate BLOCKS delivery; failure resets LOCAL 3-CLEAN streak per BC-5.39.001)
# A companion upstream PR to drbothen/vsdd-factory is being opened in parallel; this story is
# the prism-local implementation. Status must remain draft until BC authorship complete (S-7.01).
verification_properties: []
depends_on: []
blocks: []
points: 8
estimated_days: 2.5
risk: P1
acceptance_criteria_count: 5
red_gate_tests: 0
# Rationale for 0 Red Gate tests: this story is process/infrastructure (Python driver,
# holdout directory structure, delivery workflow integration). It does not touch a Rust
# crate; TDD strict mode applies to the Python driver where feasible. Red Gate count
# will be updated once the BC is authored and the driver's behavioral assertions are defined.
# The adversary will verify functional completeness, not TDD red-gate density, for this story.
estimated_passes: "2-3"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-HOLDOUT-STORY-GATE-001: Story-level holdout gate — per-story holdout driver with scenario lifecycle

## §Origin — T13 live-audit triage D-1715/D-1716

**Cascade:** AUDIT-COVERAGE-001 D-hardening; D-1715 live audit (98 PASS / 8 FAIL)
**Session record:** D-1715 (live audit 2026-07-13); D-1716 (human-approved design + story authorization)
**Authorization:** human-approved design parameters 2026-07-13 (D-1716)

The T13 106-check live audit at D-1715 returned 8 FAILs despite a 44-pass LOCAL adversarial
cascade (BC-5.39.001 3-CLEAN(strict) converged at D-1713). The gap: the LOCAL adversary
3-CLEAN protocol reviews spec and code against known contracts, but it has no access to
HIDDEN behavioral scenarios that probe the product from an agent's perspective. The live audit
filled that gap but only at the very end of development — too late in the cycle to prevent
rework.

The root insight from the triage: the T13 audit script (`scripts/t13-preflight-audit.py`)
is already a proven holdout-evaluator pattern:
- It drives the MCP server over real stdio (not mocked)
- It has access to DTU server fixtures
- It issues wire-level tool calls and asserts wire-level JSON outputs
- It is completely decoupled from the implementation (no Rust crate visibility)

What it lacks is the **per-story holdout gate infrastructure**:
1. Product-owner authors 2–4 HIDDEN scenarios per story at materialization time (separate
   from the story spec, stored where test-writer and implementer never read)
2. Gate runs AFTER LOCAL adversary 3-CLEAN, BEFORE demo recording and push
3. Gate is BLOCKING from day one — failure resets the LOCAL streak per BC-5.39.001
4. Consumed scenarios are marked used and never reused
5. Wave-level and Phase-4 holdout pools remain SEPARATE from story-level scenarios

**Human-approved design parameters (D-1716, 2026-07-13):**

| Parameter | Decision |
|-----------|----------|
| Authoring time | At story-materialization time (before delivery begins) |
| Author | Product-owner |
| Storage location | `.factory/holdout/story-scenarios/{story_id}/` (test-writer and implementer never read this path) |
| Gate placement | After LOCAL adversary 3-CLEAN (step 5 of per-story-delivery); before demo recording (step 6) |
| Blocking? | YES — blocking from day one; failure routes findings as OBSERVED BEHAVIOR ONLY |
| Scenario reuse | NO — each scenario is single-use; marked `consumed: true` after first evaluation |
| Contamination control | Evaluator reports OBSERVED BEHAVIOR ONLY (what the system did) to upstream agents; never scenario text |
| Scope | Story's touched MCP surfaces only; run duration target: minutes not hours |
| Wave/Phase-4 holdout | SEPARATE pools; unchanged by this story |
| Upstream PR | Companion PR to drbothen/vsdd-factory being opened in parallel; this story is prism-local implementation |

## Narrative

As a Prism product-owner and delivery orchestrator, I want a per-story holdout gate that runs
HIDDEN behavioral scenarios (authored by the PO at story-materialization time, never seen by
the test-writer or implementer) against the MCP server over real stdio immediately after the
LOCAL adversary 3-CLEAN converges, so that behavioral regressions and spec-drift are caught
before demo recording and push — preventing the scenario where a 44-pass LOCAL cascade converges
"clean" but a live audit immediately surfaces behavioral FAILs the suite never probed.

## Behavioral Contracts

No active BCs govern per-story holdout scenario lifecycle. This story defines the reference
implementation from which the product-owner will author the governing BC.

**Proposed BC anchors (for PO to formalize):**
- Story-level holdout gate lifecycle: scenario authoring → delivery placement → consumption →
  used-marking → OBSERVED-BEHAVIOR-ONLY routing to orchestrator
- Contamination-control invariant: evaluator agent receives scenario text; orchestrator receives
  ONLY observed wire-level behavior (not scenario text, not scenario IDs)
- Blocking semantics: gate failure resets LOCAL 3-CLEAN streak per BC-5.39.001; orchestrator
  dispatches fix-burst based on OBSERVED BEHAVIOR findings

## Acceptance Criteria

### AC-001 — Holdout-evaluator driver extracted and modularized from t13
(pending BC trace — BC authorship required before status=ready)

A new `scripts/holdout-driver/` module is created. The core driver functions that generalize
across stories are extracted from `scripts/t13-preflight-audit.py` into the holdout driver:

- `spawn_mcp_server(binary_path, config_path) -> McpSession` — launches `prism start` over
  stdio and returns a session handle. This is the same DTU harness startup pattern as the
  T13 audit; it is parameterized so any story can spawn a server against any config.
- `call_tool(session, tool_name, params) -> dict` — sends a tool call over the stdio MCP
  channel and returns the full parsed JSON response (wire-level, as `serde_json`-equivalent
  Python dict). Does NOT assert; returns raw response for scenario assertions.
- `read_resource(session, uri) -> dict` — sends a resource read over the stdio MCP channel
  and returns the full parsed JSON response.
- `shutdown_session(session)` — gracefully terminates the MCP server subprocess; verifies
  clean shutdown.

The driver module includes a `__main__.py` entry point that can be invoked as:
```
python -m holdout-driver --story-id S-STORY-NNN [--binary-path PATH] [--config-path PATH]
```

The T13 audit script itself is NOT modified — it remains the full-fidelity 106-check audit.
The driver is a NEW module that the T13 script's spawn/call patterns inspired.

### AC-002 — Scenario-pool lifecycle: authoring, storage, consumption, tracking
(pending BC trace — BC authorship required before status=ready)

The holdout directory structure is established under `.factory/holdout/`:

```
.factory/holdout/
  story-scenarios/
    {story_id}/
      SCENARIO-001.md     # authored by PO at materialization time
      SCENARIO-002.md
      manifest.yaml       # tracks which scenarios exist + consumed status
  wave-scenarios/         # UNTOUCHED — existing wave-level pool (separate)
  phase4-scenarios/       # UNTOUCHED — existing Phase-4 pool (separate)
```

`SCENARIO-NNN.md` format:
```markdown
---
scenario_id: SCENARIO-001
story_id: S-STORY-NNN
authored_by: product-owner
authored_date: YYYY-MM-DD
consumed: false
consumed_date: null
---

# [Hidden scenario text — never shown to test-writer or implementer]
...
```

`manifest.yaml` format:
```yaml
story_id: S-STORY-NNN
scenarios:
  - id: SCENARIO-001
    consumed: false
    consumed_date: null
  - id: SCENARIO-002
    consumed: false
    consumed_date: null
```

**Lifecycle rules (enforced by the driver):**
1. On each evaluation run, the driver reads `manifest.yaml` and loads all scenarios where
   `consumed: false`.
2. After successful evaluation of each scenario, the driver sets `consumed: true` and
   `consumed_date` in `manifest.yaml` and in the scenario file's frontmatter.
3. A scenario with `consumed: true` is NEVER loaded again. The driver ERRORS if it finds
   no unconsumed scenarios for the story (i.e., all scenarios already consumed).
4. The driver does NOT modify scenario file CONTENT — only frontmatter metadata.

### AC-003 — Per-story-delivery integration point
(pending BC trace — BC authorship required before status=ready)

The per-story-delivery workflow (`vsdd-factory:orchestrator:orchestrator-per-story-delivery`)
gains a new step between "LOCAL adversary 3-CLEAN converged" and "demo recording":

```
Step 5 (existing):  LOCAL adversary 3-CLEAN convergence (BC-5.39.001)
Step 5.5 (NEW):     Per-story holdout gate
  a. Orchestrator checks .factory/holdout/story-scenarios/{story_id}/manifest.yaml
  b. If manifest exists and has unconsumed scenarios:
       i.  Dispatch holdout-evaluator with: binary_path, config_path, story_id,
           scenario_dir (.factory/holdout/story-scenarios/{story_id}/)
      ii.  If evaluator returns PASS: proceed to Step 6 (demo recording)
     iii.  If evaluator returns FAIL (one or more scenarios failed): 
           - Driver marks consumed scenarios as consumed
           - Orchestrator receives OBSERVED BEHAVIOR ONLY (not scenario text)
           - Orchestrator resets LOCAL 3-CLEAN streak to 0/3 (BC-5.39.001)
           - Orchestrator dispatches fix-burst based on observed behavior
           - After fix-burst, re-dispatch implementer → LOCAL cascade → holdout gate again
  c. If no manifest or no unconsumed scenarios: LOG "no holdout scenarios — gate skipped";
     proceed to Step 6 without blocking.
Step 6 (existing):  Demo recording
```

The integration is documented in `VSDD-LOCAL-WORKFLOW.md` (new file in `.factory/docs/`) with
the full step-by-step procedure, and the per-story-delivery orchestrator workflow spec is
updated to include Step 5.5.

### AC-004 — Contamination-control routing rules
(pending BC trace — BC authorship required before status=ready)

The driver enforces contamination control at the architectural level:

1. **Evaluator agent isolation:** The holdout-evaluator agent receives:
   - Scenario text (full `.md` file contents)
   - Binary path + config path for server spawn
   - Story ID for scoping (but not the story spec itself)
   - Tool/resource inventory for the story's touched surfaces
   The evaluator agent does NOT receive: story spec, behavioral contracts, test code, prior
   adversary findings.

2. **Upstream routing (OBSERVED BEHAVIOR ONLY):** The evaluator's output to the orchestrator
   contains ONLY:
   - `pass: true/false`
   - For each FAILED scenario: `observed_behavior: <what the system returned>` — the wire-level
     response (truncated if large) but NEVER the scenario text or scenario ID
   - `surfaces_tested: [tool_name, ...]` — list of tools/resources called during evaluation

3. **Logging:** The full scenario text and scenario IDs are logged to a LOCAL-ONLY file
   `.factory/holdout/eval-logs/{story_id}-{timestamp}.log` which is git-ignored and never
   committed to `factory-artifacts` branch.

4. **No cross-story leakage:** Scenario files for story A are never readable by the holdout
   evaluator for story B. The driver enforces this by accepting only the story-specific
   directory, not the parent directory.

### AC-005 — Evaluator scoping to story's touched surface (minutes not hours)
(pending BC trace — BC authorship required before status=ready)

The driver accepts a `--surfaces` parameter (optional) listing the MCP tool names and resource
URIs that the story touched. The evaluator ONLY spawns the subset of the MCP server needed for
those surfaces:

```bash
python -m holdout-driver \
  --story-id S-TEST-WIRESHAPE-SWEEP-001 \
  --surfaces "query,prism_describe,prism://config/clients" \
  --binary-path ./target/debug/prism \
  --config-path .prism/test.toml
```

If `--surfaces` is omitted, all 14 LIVE_TOOLS + 6 resources are in scope (full evaluation).
With `--surfaces`, the evaluator only exercises those surfaces, keeping run time under 2 minutes
for a focused story.

**Timing target:** a story holdout evaluation with 2–4 scenarios across 3–5 surfaces
completes in under 2 minutes on the prism development machine (macOS aarch64, DTU already
warmed up). The driver MUST NOT spawn DTU instances that are not needed for the story's surfaces.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `holdout-driver/` Python module (new) | `scripts/holdout-driver/` | Effectful (subprocess spawn, file I/O, stdio MCP protocol) |
| Scenario storage | `.factory/holdout/story-scenarios/` | Effectful (file system; git-tracked scenario frontmatter; scenario TEXT is read-only for evaluator) |
| Eval logs | `.factory/holdout/eval-logs/` | Effectful (file system; git-ignored; not committed to factory-artifacts) |
| Delivery workflow update | `.factory/docs/VSDD-LOCAL-WORKFLOW.md` (new) | Doc (pure) |
| Per-story-delivery orchestrator spec | vsdd-factory plugin (upstream PR) | See upstream PR note |

Architecture section references:
- `architecture/module-decomposition.md` (no subsystem owns scripts/ — pipeline infrastructure)
- `vsdd-factory:orchestrator:orchestrator-per-story-delivery` (upstream workflow spec)

**Anchor justifications (POL-4/POL-5):**
- No subsystem anchor: `scripts/holdout-driver/` is pipeline infrastructure with no ARCH-INDEX
  subsystem. The architect should be consulted if the driver grows beyond a Python script and
  requires integration into a prism crate.
- No `depends_on`: the driver is independent infrastructure. It uses the same DTU harness
  boot pattern as the T13 audit script, which is already built and deployed.
- `blocks: []`: No product story depends on this. The gate is opt-in per story (skipped if
  no manifest). Future product stories can adopt it incrementally.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No scenarios exist for a story (manifest absent or empty) | Gate is skipped; LOG "no holdout scenarios — gate skipped"; does NOT block delivery |
| EC-002 | All scenarios for a story are already consumed | Driver ERRORs: "all holdout scenarios consumed — PO must author new scenarios"; blocks delivery until PO acts |
| EC-003 | MCP server fails to start (exit non-zero) | Evaluation aborts; returns `pass: false`; orchestrator receives `observed_behavior: "server startup failed"` |
| EC-004 | Evaluator times out (> 5 min for full suite) | Driver kills the server subprocess; returns `pass: false` with `observed_behavior: "evaluation timeout"` |
| EC-005 | Scenario text references a tool that is NOT in `--surfaces` | Evaluator still attempts the call (the scenario is authoritative); logs a WARN "surface not pre-declared" |
| EC-006 | Wave/Phase-4 scenario pool directory exists but story-scenarios/ does not | Story-scenarios path is created on first PO authorship; wave/phase-4 pools are not touched |
| EC-007 | DTU server is not running when driver is invoked | Driver checks for DTU health before spawning MCP server; returns structured error with `observed_behavior: "DTU not running"` |

## Token Budget Estimate

| Item | Lines | Tokens (est.) |
|------|-------|--------------|
| Story spec (this file) | ~300 | ~4,200 |
| scripts/t13-preflight-audit.py (reference for extraction) | ~2,000 | ~28,000 |
| vsdd-factory orchestrator-per-story-delivery (upstream spec) | ~300 | ~4,200 |
| New holdout-driver/ module (~4 Python files) | ~400 | ~5,600 |
| .factory/holdout/ directory structure + schema | ~60 | ~840 |
| .factory/docs/VSDD-LOCAL-WORKFLOW.md (new) | ~100 | ~1,400 |
| **Total estimate** | | **~44,240 tokens** |

The t13-preflight-audit.py reference context is large. Implementer should load ONLY the
spawn/call utility functions (not the 100+ check functions) when implementing the driver
extraction. Estimated effective context: ~15,000 tokens. Fits within 100k-token window (~15%).

## Tasks

- [ ] Read `.factory/SESSION-HANDOFF.md` §RESUME SNAPSHOT D-1716 for the exact human-approved design parameters before writing any code.
- [ ] Read `scripts/t13-preflight-audit.py` (spawn, call_tool, parse_envelope sections only) to understand the reusable patterns.
- [ ] Create `.factory/holdout/story-scenarios/` directory structure with a `.gitkeep` and a `README.md` explaining the authoring convention.
- [ ] Create `scripts/holdout-driver/__init__.py`, `spawn.py`, `call_tool.py`, `lifecycle.py`, `__main__.py` (module skeleton).
- [ ] Implement `spawn_mcp_server`, `call_tool`, `read_resource`, `shutdown_session` in the driver (AC-001).
- [ ] Implement `manifest.yaml` read/write + scenario consumption lifecycle (AC-002).
- [ ] Create `scripts/holdout-driver/routing.py` implementing contamination-control: OBSERVED BEHAVIOR ONLY output formatting (AC-004).
- [ ] Create `.factory/holdout/eval-logs/.gitignore` (ignore all log files in this directory).
- [ ] Write `scripts/holdout-driver/tests/test_driver.py` with basic unit tests for `spawn_mcp_server` (mock subprocess), scenario lifecycle (consumed/unconsumed), and surface scoping (AC-005).
- [ ] Create `.factory/docs/VSDD-LOCAL-WORKFLOW.md` documenting Step 5.5 in the per-story delivery procedure (AC-003).
- [ ] Open the upstream drbothen/vsdd-factory PR for the orchestrator-per-story-delivery Step 5.5 addition (separate action; tracked separately; prism-local implementation does not wait for upstream merge).
- [ ] Run `python -m pytest scripts/holdout-driver/tests/` — all unit tests GREEN.
- [ ] Smoke-test: run `python -m holdout-driver --story-id S-TEST-WIRESHAPE-SWEEP-001 --binary-path ./target/debug/prism --config-path .prism/test.toml` against a running DTU server; confirm driver spawns MCP server, calls `query` tool, and shuts down cleanly.

## Previous Story Intelligence

No prior story has implemented a per-story holdout gate for prism. Prior context:

- The T13 106-check audit harness (`scripts/t13-preflight-audit.py`) is the proven holdout
  pattern at the session level. It was built over the 44-pass AUDIT-COVERAGE-001 cascade
  (D-1609 through D-1713). Its spawn/call patterns are the implementation reference for the
  driver extraction.
- Phase-4 holdout evaluation (`vsdd-factory:holdout-evaluator`) provides story-level holdout
  at the WAVE-completion boundary. This story creates a finer-grained gate at the
  INDIVIDUAL-STORY boundary.
- BC-5.39.001 (3-CLEAN protocol) currently governs the LOCAL cascade convergence. Step 5.5
  of this story extends the gate to include holdout evaluation before allowing the streak
  to "count" for demo/push purposes.
- S-AUDIT-PROCESS-CONVENTIONS-001 (draft, maintenance epic) codifies audit-script authoring
  conventions. S-HOLDOUT-STORY-GATE-001 (this story) is the next-level infrastructure that
  makes those conventions enforceable as a blocking gate.

## Architecture Compliance Rules

- **Python tooling only:** The holdout driver is a Python module in `scripts/holdout-driver/`.
  It does NOT touch any Rust crate. It communicates with the MCP server over stdio only.
  No Rust-level changes are required by this story.
- **Git-ignored eval logs:** `.factory/holdout/eval-logs/` MUST be git-ignored. Scenario
  evaluation logs contain holdout scenario text and must NEVER be committed to
  `factory-artifacts` branch or any other branch. Add to `.factory/.gitignore` or to
  `.factory/holdout/eval-logs/.gitignore`.
- **Scenario text is single-use:** The `consumed: true` / `consumed_date` update in
  `manifest.yaml` must be committed atomically after evaluation in the same
  `factory-artifacts` burst commit. This prevents a race where evaluation fails partway
  through and leaves scenarios in an ambiguous state.
- **BC-5.39.001 streak reset is ORCHESTRATOR responsibility:** The holdout driver returns
  `pass: true/false` to the orchestrator. The orchestrator resets the LOCAL streak —
  the driver itself does NOT directly modify STATE.md or the adversary cascade state.
- **TD-VSDD-091:** Cite function names and file paths (not line numbers) in all workflow
  documentation.
- **Upstream PR tracking:** The upstream `drbothen/vsdd-factory` PR for Step 5.5 must be
  tracked in STATE.md once opened. The prism-local implementation (this story) ships
  regardless of upstream merge timing — the gate works locally via the manual `--story-id`
  invocation path even before orchestrator-per-story-delivery is updated upstream.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `Python` | `>=3.10` (project convention from scripts/) | Driver uses `subprocess`, `json`, `pathlib` — no third-party deps |
| `pytest` | project convention | Unit tests for driver module |
| `pyyaml` | project convention (already used in scripts/) | `manifest.yaml` read/write |
| `scripts/t13-preflight-audit.py` | reference only | Extract patterns; do NOT import from it |

No new Python dependencies beyond those already used in `scripts/`. Confirm project Python
version with `python --version` before writing `f-string` syntax.

**Forbidden patterns:**
- Driver MUST NOT import from `scripts/t13-preflight-audit.py` (the audit script is a
  monolith with 100+ check functions; the driver extracts patterns, it does not fork or
  import the script).
- Driver MUST NOT write scenario text to stdout or to any log that could be committed.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `scripts/holdout-driver/__init__.py` | Create | Module init |
| `scripts/holdout-driver/spawn.py` | Create | `spawn_mcp_server`, `shutdown_session` (AC-001) |
| `scripts/holdout-driver/call_tool.py` | Create | `call_tool`, `read_resource` (AC-001) |
| `scripts/holdout-driver/lifecycle.py` | Create | `load_scenarios`, `mark_consumed` — manifest.yaml lifecycle (AC-002) |
| `scripts/holdout-driver/routing.py` | Create | `format_observed_behavior` — contamination-control output formatter (AC-004) |
| `scripts/holdout-driver/__main__.py` | Create | CLI entry point (AC-001, AC-005) |
| `scripts/holdout-driver/tests/test_driver.py` | Create | Unit tests for driver (AC-001/002/005) |
| `.factory/holdout/story-scenarios/.gitkeep` | Create | Initialize directory structure |
| `.factory/holdout/story-scenarios/README.md` | Create | PO authoring instructions |
| `.factory/holdout/eval-logs/.gitignore` | Create | `*` (ignore all log files) |
| `.factory/docs/VSDD-LOCAL-WORKFLOW.md` | Create | Step 5.5 per-story-delivery documentation (AC-003) |
| `.factory/.gitignore` | Modify | Add `.factory/holdout/eval-logs/` if not already ignored |
