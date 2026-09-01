---
document_type: story
story_id: "S-MAINT-SENSOR-TEST-DISCIPLINE-GATE-001"
title: "CI Gate — Enforce SAP2_STATUS Marker and Real Live-Test Bodies in Sensor Test Files"
wave: tbd
epic_id: maintenance
priority: P3
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-09-01"
modified: "2026-09-01"
input-hash: "[live-state]"
inputs: []
traces_to: ""
cycle: "wave-xdome-g-series"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched: []
target_module: "scripts/, ci.yml, crates/ (test-file generation guidance)"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship
verification_properties: []
depends_on: []
blocks: []
points: 3
estimated_days: 0.75
risk: MEDIUM
acceptance_criteria_count: 5
red_gate_tests: 0
estimated_passes: "tbd"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
tags:
  - process-gap
  - sensor-testing
  - factory-tooling
  - sap2-discipline
  - sid1-discipline
---

# S-MAINT-SENSOR-TEST-DISCIPLINE-GATE-001: CI Gate — Enforce SAP2_STATUS Marker and Real Live-Test Bodies in Sensor Test Files

## Origin

**Process-gap finding class:** Two distinct test-authoring defect classes, each meeting the lessons-codification 3-recurrence threshold across the Claroty xDome G-series.

### Defect Class A — Missing `const SAP2_STATUS` marker in no-DTU sensor test files

Every sensor table that ships without a DTU route must carry a `const SAP2_STATUS: &str = "N/A: …; deferred to D-2200 (…)"` constant in its test file (plus an `assert!` that verifies the constant starts with `"N/A:"`) so that the SAP-2 grep audit can confirm the absence is documented, not accidentally missing. This class has now recurred three times across the G-series:

| Instance | Finding | Detail |
|----------|---------|--------|
| G4 (S-CLAROTY-SERVERS-001) | F-001 | No `SAP2_STATUS` constant in any of the four servers/server-interfaces test files |
| G5 (S-CLAROTY-ORGPOLICY-001) | CR-001 | SAP2_STATUS constant absent in org-policy test files |
| G6 (S-CLAROTY-ACLPOLICY-001) | MED-001 / CR-002 | SAP2_STATUS constant absent; pre-merge front-loaded review added the constant as a fix before push |

Each occurrence was caught during adversarial review and corrected in a fix-burst; none reached `develop` without the constant. But the cost of three separate fix-bursts — and the adversarial finding-mint cycle each triggers — is avoidable with a mechanical gate.

### Defect Class B — Non-real live-test bodies (`todo!()`/`panic!()` stubs)

`#[ignore]`'d tests whose names contain `_live_` must have real, env-gated bodies: read `CLAROTY_INSTANCE_URL` (or the sensor's equivalent env var), skip gracefully if absent, and execute a real query when present. Authoring `panic!("implement me")` or `todo!()` as the sole body violates SID-1 (CLAUDE.md §Standing Adversary Probes), which requires a concrete blocking dependency citation — not a stub — when a live test cannot be ungated. This class has recurred three times:

| Instance | Finding | Detail |
|----------|---------|--------|
| G2 (S-CLAROTY-OT-EVENTS-001) | (pre-merge fix) | Live test body was `panic!("not yet implemented")` — replaced with env-gated real query in fix-burst |
| G5 (S-CLAROTY-ORGPOLICY-001) | CR-002 | Live test body was `todo!()` — caught in pre-PR front-loaded review |
| G6 (S-CLAROTY-ACLPOLICY-001) | CR-002 | Live test body was `todo!()` — caught in pre-PR front-loaded review |

Both defect classes share a common root cause: the test-writer (and stub-architect) agents do not have a machine-enforced template requirement. They rely on prior story intelligence and CLAUDE.md discipline references, which drift under context pressure. A CI gate + a generated skeleton that ships the correct structure by default eliminates the recurrence source.

---

## Narrative

As a test-writer authoring sensor adapter tests for a table that has no DTU route,
I want the CI check to fail immediately — before review — if my test file is missing a `const SAP2_STATUS` constant or if any `#[ignore]`'d live test has `todo!()`/`panic!()` as its entire body,
so that both defect classes are caught at commit time rather than in adversarial review, and the SAP-2 grep audit remains reliable without requiring a human to add the same constant in every G-series fix-burst.

---

## Acceptance Criteria

### AC-001 — CI gate detects missing `const SAP2_STATUS` in sensor test files flagged no-DTU
(Traceability to BCs is pending PO authorship)

A script (`scripts/check-sensor-test-discipline.sh` or an extension of `scripts/records-lint.sh`) MUST scan all `crates/**/tests/*.rs` and `crates/**/src/**/*_test*.rs` files that:
- Reside in a sensor adapter crate (`prism-sensors` or `prism-dtu-*`), AND
- Do NOT have a corresponding DTU route handler (determined by the absence of a matching route file in the relevant `prism-dtu-*` crate)

For each such file the script MUST fail with a non-zero exit code and a human-readable error if:
- The file does NOT contain `const SAP2_STATUS` at all, OR
- `SAP2_STATUS` is present but its value does not start with `"N/A:"`

The error MUST name the file path and cite D-2200 as the canonical deferred-DTU tracking decision.

The script is wired into `ci.yml` so it runs on every push to a feature branch touching `crates/prism-sensors/` or `crates/prism-dtu-*/`.

### AC-002 — CI gate verifies `SAP2_STATUS` carries a `D-2200` reference
(Traceability to BCs is pending PO authorship)

The same script MUST also fail if `const SAP2_STATUS` is present and starts with `"N/A:"` but does NOT contain the substring `"D-2200"` anywhere in the string value. This ensures the constant points to the canonical tracking decision rather than a freeform narrative that will not be found by the D-2200 grep audit.

Pass: `const SAP2_STATUS: &str = "N/A: no DTU route; deferred to D-2200 (xDome G2–G6 DTU-parity batch)";`
Fail: `const SAP2_STATUS: &str = "N/A: no DTU route yet";` (missing `D-2200`)

### AC-003 — CI gate forbids `todo!()`/`panic!()` as the sole body of `#[ignore]`'d `_live_` tests
(Traceability to BCs is pending PO authorship)

The same script MUST scan all Rust test files in the workspace for functions that meet ALL of:
- Have the `#[ignore]` attribute, AND
- Have a name containing `_live_`

For each such function, the script MUST fail if the function body consists ONLY of a `todo!()` macro call or a `panic!(…)` macro call (with any argument), with no other statements before or after it. The error MUST name the test function, cite SID-1 from CLAUDE.md as the governing discipline, and provide a one-line remediation hint: "Replace with an env-gated body: read SENSOR_INSTANCE_URL, skip if absent, execute a real query if present."

A function with a graceful-skip guard (e.g., `if env::var("SENSOR_INSTANCE_URL").is_err() { return; }`) followed by real query logic MUST pass the gate even if the skip branch would make the test a no-op in most CI environments.

### AC-004 — Authoring skeleton / test-writer generation guidance updated
(Traceability to BCs is pending PO authorship)

A prism-local authoring guidance document (either a new `docs/sensor-test-authoring.md` or an amendment to `CLAUDE.md` §Standing Adversary Probes) MUST be created or updated to include:

1. A copy-pasteable `const SAP2_STATUS` template with `N/A:` prefix and `D-2200` reference.
2. A copy-pasteable `#[ignore]`'d live-test skeleton with env-gated body and graceful skip per SID-1.
3. A pointer to this story (`S-MAINT-SENSOR-TEST-DISCIPLINE-GATE-001`) as the origin of the mechanical gate.

If the project ships a stub-architect or test-writer code-generation template for sensor adapters, those templates MUST be updated to emit the `SAP2_STATUS` constant and the env-gated live-test skeleton by default — closing the source of the recurrence, not just adding a gate over it.

### AC-005 — Upstream issue filed against drbothen/vsdd-factory
(Traceability to BCs is pending PO authorship)

A GitHub issue is filed against `drbothen/vsdd-factory` documenting:
- (a) The two defect classes and their G-series recurrence evidence (Defect Class A: G4 F-001, G5 CR-001, G6 MED-001/CR-002; Defect Class B: G2 panic!(), G5 todo!(), G6 todo!()).
- (b) The governing disciplines: SAP-2 probe (CLAUDE.md), SID-1 (CLAUDE.md), D-2200 (deferred-DTU tracking decision).
- (c) A proposed gate specification: the two checks in AC-001/AC-002/AC-003, scoped to sensor adapter test files.
- (d) A reference to this story.

The upstream issue URL is recorded in §Deliverables.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `scripts/check-sensor-test-discipline.sh` (new) or `scripts/records-lint.sh` extension | `scripts/` | Effectful (reads Rust source files, exits non-zero on violation) |
| `ci.yml` amendment | `.github/workflows/ci.yml` | Effectful (runs the script as a CI job step) |
| Authoring guidance doc | `docs/sensor-test-authoring.md` (new) or `CLAUDE.md` amendment | Pure (documentation) |
| Sensor adapter test files (read-only inputs to the gate) | `crates/prism-sensors/**`, `crates/prism-dtu-*/**` | Pure (read input) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A test file belongs to `prism-dtu-claroty` (has a DTU route) — should NOT require `SAP2_STATUS` | Gate skips files in crates that ARE a DTU crate; the constant is only required in no-DTU sensor table test files |
| EC-002 | `SAP2_STATUS` value contains `"D-2200"` in a comment rather than the string literal itself | Gate checks the string literal value; a comment does not satisfy AC-002. Emit a WARN (not a hard block) if it appears in a comment only, suggesting the comment be moved into the constant value |
| EC-003 | `#[ignore]`'d live test has `todo!()` on line 1 and a `// TODO(D-2200): deferred` comment on line 2 and nothing else | Gate still fails — a comment is not a real body. The todo!() call is the SOLE executable statement |
| EC-004 | `#[ignore]`'d live test name contains `_live_data_` but has a real env-gated body | Gate passes — the name-contains-`_live_` condition is met but the body is not a bare stub |
| EC-005 | New sensor crate added mid-sprint; the gate script does not know about its `prism-dtu-*` pair yet | Gate uses a heuristic: if a crate name starts with `prism-sensors` AND the test file's table name has no matching file under any `prism-dtu-*` route directory, treat it as no-DTU and enforce AC-001/AC-002 |
| EC-006 | The sensor table eventually gets a DTU route (a future G-series story merges its DTU companion) | The `SAP2_STATUS` constant is harmless once a DTU route exists; the gate logic skips the check for files where a matching DTU route handler is found. No cleanup required |

---

## Purity Classification

| Element | Classification | Rationale |
|---------|---------------|-----------|
| `SAP2_STATUS` constant presence check | **Pure** | Reads source file contents; no I/O side effects; deterministic for a given file state |
| `todo!()`/`panic!()` sole-body detector | **Pure** | Text pattern matching on source files; no mutation |
| CI gate script (`check-sensor-test-discipline.sh`) | **Effectful** | Exits non-zero to block CI; reads from the filesystem |
| `ci.yml` wiring | **Effectful** | Triggers the script as a side-effecting CI step |

---

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~3,000 |
| `scripts/records-lint.sh` (reference for script structure) | ~2,000 |
| Sample sensor test files (2–3 representative files for gate logic design) | ~4,000 |
| `ci.yml` (relevant job section for amendment) | ~1,000 |
| Authoring guidance doc authoring | ~1,500 |
| Upstream issue text authoring | ~1,000 |
| Total | ~12,500 |

Well within a single agent context window. No split required.

---

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation)

N/A at story-writing time — this story's deliverables are a CI gate script and authoring guidance. No Rust production code has `todo!()` stubs. The story is `tdd_mode: strict` and its test vehicle is the gate script itself. When the implementing agent is dispatched, the test-writer MUST write a failing test harness that:

- Feeds known-bad sensor test file fixtures (missing `SAP2_STATUS`, `SAP2_STATUS` without `D-2200`, bare `todo!()` live test) and asserts non-zero exit code.
- Feeds known-good fixtures (correct `SAP2_STATUS`, real env-gated live test) and asserts zero exit code.

**Red Gate density check** (BC-5.38.001): **0 pre-written named tests** at story-writing time. Tests will be enumerated in a follow-up story-writer pass when the implementing module (shell script vs records-lint extension) is confirmed. Density check deferred to implementation-time pre-pass (standard pattern for tooling stories where the test vehicle is not yet determined). This story's `status: draft` reflects BC status pending PO authorship — it does not transition to `ready` until `behavioral_contracts:` is non-empty per S-7.01.

### Implementation tasks

- [ ] T-01: Confirm implementing module — extend `scripts/records-lint.sh` (adding a new L-check arm) or create `scripts/check-sensor-test-discipline.sh`. Record decision. Note: the records-lint extension is preferred if the check logic can be isolated to a clearly named section, because it reuses the existing CI wiring.
- [ ] T-02: Implement AC-001/AC-002 — `SAP2_STATUS` presence and `D-2200` reference check for no-DTU sensor test files.
- [ ] T-03: Implement AC-003 — `todo!()`/`panic!()` sole-body detector for `#[ignore]`'d `_live_` tests.
- [ ] T-04: Wire the script into `ci.yml` so it runs on pushes to branches touching sensor adapter crates (AC-001/AC-002/AC-003).
- [ ] T-05: Author or amend the authoring guidance document (AC-004) — `SAP2_STATUS` template + env-gated live-test skeleton.
- [ ] T-06: If a stub-architect or test-writer code-generation template for sensor adapters exists in this repo or the vsdd-factory engine, update it to emit the correct skeleton by default (AC-004 second part).
- [ ] T-07: File upstream issue against `drbothen/vsdd-factory` (AC-005); record URL in §Deliverables.

---

## Previous Story Intelligence

**S-MAINT-RG-LIST-GATE-001** — direct structural precedent. Established the factory tooling gate pattern: identify root cause (3-recurrence threshold) → specify gate logic with pass/fail criteria → upstream issue. Match its frontmatter field set and section ordering. Both stories are `tdd_mode: strict`, `epic_id: maintenance`, `behavioral_contracts: [] pending PO authorship`.

**S-MAINT-ADR-ANCHOR-GATE-001** — second structural precedent. Gate story with two-tier enforcement (hard block + warning). Confirms the pattern of distinguishing hard-block (C1/C2) from warn-only (C3/C4) tiers; AC-003 of this story mirrors that approach.

**S-MAINT-BURST-COMMIT-COUNT-GATE-001** — another process-gap gate story. Its pattern of wiring a detection script into the factory hook chain is relevant; this story targets CI (ci.yml) rather than the factory hook chain, but the principle is identical.

**G-series recurrence evidence (direct predecessors):**
- S-CLAROTY-OT-EVENTS-001 (G2): `panic!()` live test body — corrected in fix-burst before push.
- S-CLAROTY-SERVERS-001 (G4): missing `SAP2_STATUS` — F-001 finding, corrected in fix-burst.
- S-CLAROTY-ORGPOLICY-001 (G5): missing `SAP2_STATUS` (CR-001) + `todo!()` live test body (CR-002) — both corrected in pre-PR front-loaded review, D-2408 burst.
- S-CLAROTY-ACLPOLICY-001 (G6): missing `SAP2_STATUS` (MED-001/CR-002) — corrected in pre-PR front-loaded review.

---

## Architecture Compliance Rules

1. **No prism crate logic modifications.** This story MUST NOT add, remove, or edit any file under `crates/` except: (a) existing sensor test files if they need a one-time SAP2_STATUS retrofit to pass the new gate (these are documentation-only additions, not logic changes), and (b) code-generation templates if they exist in `crates/`. All gate logic lives in `scripts/`.
2. **No STATE.md edits.** STATE.md is state-manager territory.
3. **No STORY-INDEX.md edits.** Registration is a state-manager burst, not this story's deliverable.
4. **TD-VSDD-053 single-commit-per-burst applies.** All `.factory/` and `scripts/` changes in the same logical burst go in one atomic commit.
5. **No CLAUDE.md section deletions.** If the authoring guidance is appended to CLAUDE.md, it MUST be added as a new subsection under `§Standing Adversary Probes` or `§Conventions`, never replacing existing content.
6. **SAP-2 and SID-1 are the governing disciplines.** The gate mechanizes what these disciplines require; the gate MUST NOT weaken or redefine them.

---

## Library & Framework Requirements

No Rust library dependencies. Deliverable is a bash script. Use standard POSIX tools (`grep`, `awk`, `find`, `sed`). The `SAP2_STATUS` constant detection uses `grep -rE 'const SAP2_STATUS' <target>`. The `todo!()`/`panic!()` sole-body detection uses an `awk` or `grep -A N` pattern to extract the function body between the opening `{` and closing `}` of `#[ignore]`'d `_live_` functions and check that the trimmed body is exactly `todo!()` or `panic!(…)`. No `yq` or YAML parsing required (inputs are Rust source, not YAML).

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `scripts/check-sensor-test-discipline.sh` | Create (preferred) OR extend `scripts/records-lint.sh` with a new L-check arm | AC-001/AC-002/AC-003 gate logic; decision at T-01 |
| `.github/workflows/ci.yml` | Amend | Add job step wiring the gate script for pushes touching sensor adapter crates (AC-001/AC-002/AC-003) |
| `docs/sensor-test-authoring.md` | Create (preferred) OR amend `CLAUDE.md` §Standing Adversary Probes | Authoring guidance: SAP2_STATUS template + env-gated live-test skeleton (AC-004) |
| Upstream issue in `drbothen/vsdd-factory` | Create | AC-005; URL recorded in §Deliverables |

---

## §Deliverables

| Item | Status | Reference |
|------|--------|-----------|
| Upstream issue URL | Pending | (to be filled at T-07 completion) |
| `scripts/check-sensor-test-discipline.sh` (or records-lint extension) | Pending | AC-001/AC-002/AC-003 |
| `docs/sensor-test-authoring.md` (or CLAUDE.md amendment) | Pending | AC-004 |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-09-01 | story-writer | New story registered at 3-recurrence threshold — two test-authoring defect classes codified as a CI gate. Defect Class A (missing SAP2_STATUS): G4 F-001, G5 CR-001, G6 MED-001/CR-002. Defect Class B (todo!()/panic!() live-test bodies): G2 panic!(), G5 todo!(), G6 todo!(). Gate: AC-001/AC-002 SAP2_STATUS presence + D-2200 ref; AC-003 sole-body stub detection; AC-004 authoring guidance; AC-005 upstream issue. status: draft; behavioral_contracts: [] pending PO authorship per S-7.01; post-v1 P3 |
