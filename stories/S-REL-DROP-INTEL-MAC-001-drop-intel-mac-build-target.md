---
document_type: story
story_id: S-REL-DROP-INTEL-MAC-001
title: "devops: drop x86_64-apple-darwin (Intel mac) from release matrix — 4-target sweep (ADR-065)"
wave: F-A
epic_id: E-REL
priority: P0
status: ready
version: "1.1"
level: "L4"
producer: story-writer
timestamp: "2026-09-07T00:00:00Z"
tdd_mode: facade
# Facade justification: this story consists entirely of CI workflow edits, toolchain config
# changes, documentation updates, and shell test gate file renames/updates — no Rust source
# files are modified. Verification is by grep assertions + running the CI-gate and
# release-gate shell scripts. Mutation testing at wave gate replaces Red Gate density check
# per BC-8.30.001.
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) governs the release build pipeline and binary distribution
#   matrix. The removal of x86_64-apple-darwin from release.yml, ci.yml, toolchain config,
#   install scripts, and CI gate tests all fall within SS-22's process lifecycle boundary.
#   No other ARCH-INDEX subsystem owns the CI/CD release build matrix.
crates_touched: []
# crates_touched: [] — this story modifies only CI workflow YAML, toolchain config,
# documentation, shell scripts, and shell test files. No Rust crate source is changed.
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — dropping a build target is release-infrastructure governance.
# No subsystem behavioral contract governs the CI release matrix composition.
# Authority is ADR-065 (ACCEPTED v1.0 2026-09-07; human-directed). POL-14 NO-OP.
verification_properties: []
depends_on: []
# Dependency anchor justification:
#   No technical dependency on other stories. ADR-065 is already ACCEPTED. This story
#   must land on develop BEFORE v1.0.0-beta.1 is tagged (human directive 2026-09-07:
#   "fold in now, before the tag"). It does not block any other story (the dropped target
#   has no downstream build artifacts that other stories depend on in the 4-target world).
blocks: []
points: 3
estimated_days: 1
risk: LOW
# Risk justification: pure deletion and count-update across CI/docs/tests — no new logic
# introduced. Risk is LOW because Intel mac was not a target that other features depended
# on; removing it only reduces the build matrix. Verification is mechanical (grep gates +
# shell test runs).
acceptance_criteria_count: 5
red_gate_tests: 0
# Red Gate N/A — tdd_mode: facade. Verification via VF-001..VF-005 grep+run-gate steps.
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
# Holdout Applicability: HOLDOUT-N/A
# Exemption: infra-only story — no built-binary MCP surface is modified. This story
# touches only CI workflow files, toolchain config, docs, and shell test scripts. There
# is no MCP tool response surface, no Rust binary behavior, and no wire-level output
# for a holdout evaluator to exercise. Consistent with prior infra-story determinations
# for S-REL-CLIFF-001, S-REL-WRITER-001, and S-REL-BETA1-NOTES-001 (all holdout_scenarios: []).
assumption_validations: []
risk_mitigations:
  - "Historical records are immutable: docs/demo-evidence/**, .factory/cycles/**,
    .factory/phase-0-ingestion, .factory/research, and completed merged story files
    (S-0.01, S-0.02, S-REL-001, S-MAINT-CI-DISK-EXHAUSTION-001) MUST NOT be touched.
    The grep gate in AC-002 excludes these paths. Any grep hit inside these paths is
    a false positive, not a violation."
  - "File rename (ci-gate test): tests/ci-gate/test_AC-3_matrix-5-platforms.sh must be
    renamed to test_AC-3_matrix-4-platforms.sh via git mv, not deleted+recreated, to
    preserve git history. The ci.yml shellcheck or lint step referencing the old filename
    must also be updated (grep ci.yml for the old filename before committing)."
  - "CLAUDE.md update is human-authorized: the toolchain header in CLAUDE.md lists
    cross-compile targets. Removing x86_64-apple-darwin from this list is explicitly
    authorized per ADR-065 site inventory. This is one of the few permitted CLAUDE.md
    edits by a non-orchestrator agent (human-directed scope change)."
  - "install.sh Darwin-x86_64 arm removal: the detection arm must be removed cleanly
    (not just commented out). After removal, an unsupported arch on macOS (i.e.,
    Darwin-x86_64) must fall through to the wildcard error arm printing
    'Unsupported platform' and exiting non-zero. Do not add a 'Use Apple Silicon Mac'
    suggestion that could be construed as a support commitment."
  - "AC-002 grep scope: exclude .factory/stories/ (merged story files are historical);
    exclude docs/demo-evidence/; exclude .factory/cycles/; exclude
    .factory/phase-0-ingestion/; exclude .factory/research/. The grep gate is
    forward-looking only — it must not fire on historical records."
inputs: []
# Key reference inputs (not hashed — devops workstream B files owned by implementer):
#   .factory/specs/architecture/decisions/ADR-065-release-build-target-matrix.md
#   .github/workflows/release.yml, .github/workflows/ci.yml
#   .github/workflows/release-promote.yml, .github/workflows/release-tag.yml
#   rust-toolchain.toml, scripts/install.sh
#   tests/ci-gate/test_AC-3_matrix-5-platforms.sh
#   tests/release-gate/test_AC-006_matrix-targets.sh
#   tests/release-gate/test_AC-012_install-scripts.sh
input-hash: "[live-state]"
traces_to: []
cycle: "v1.0.0-beta.1-drop-intel-mac"
phase: "F3"
---

# S-REL-DROP-INTEL-MAC-001 — devops: drop x86_64-apple-darwin (Intel mac) from release matrix

**Story ID:** S-REL-DROP-INTEL-MAC-001
**Status:** ready
**Version:** v1.1
**Wave:** F-A
**Priority:** P0
**Points:** 3

---

## Origin

Human directive 2026-09-07: "only support latest macOS, drop Intel macs." Architecture
decision ADR-065 (ACCEPTED v1.0 2026-09-07) establishes the authoritative 4-target release
matrix and removes `x86_64-apple-darwin` (Intel macOS) permanently from the release build.

The previous 5-target matrix (`aarch64-apple-darwin`, `x86_64-apple-darwin`,
`x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`) is
retired. The new 4-target matrix is:

| Target | Platform | Archive format |
|--------|----------|----------------|
| `aarch64-apple-darwin` | macOS (Apple Silicon — M1/M2/M3/M4+) | `.tar.gz` |
| `x86_64-unknown-linux-gnu` | Linux (glibc) | `.tar.gz` |
| `x86_64-unknown-linux-musl` | Linux (static musl) | `.tar.gz` |
| `x86_64-pc-windows-msvc` | Windows (MSVC) | `.zip` |

This story executes the coordinated sweep of all non-historical files to reflect this matrix.
Authority: ADR-065 §D1 / §D2 / §D3. This story MUST land before v1.0.0-beta.1 is tagged.

---

## Narrative

As a devops engineer maintaining the prism release pipeline, I want to remove
`x86_64-apple-darwin` from every non-historical CI workflow, toolchain config, install
script, documentation file, and CI gate test in the repository, so that the 4-target matrix
established by ADR-065 is consistently enforced and the retired Intel mac target can never
accidentally re-enter the build matrix.

---

## Behavioral Contracts

No subsystem BCs govern the CI release matrix composition. Authority is ADR-065 (ACCEPTED
v1.0 2026-09-07; human-directed). POL-14 NO-OP (`behavioral_contracts: []`).

| Architecture Source | Clause |
|--------------------|--------|
| ADR-065 §D1 | 4-target matrix is the single source of truth; all artifacts MUST reflect this set |
| ADR-065 §D2 | `x86_64-apple-darwin` and `macos-15-intel` MUST NOT appear in release.yml or ci.yml |
| ADR-065 §D3 | ADR-064 "5-platform" / "5 build targets" / "5 legs" counts MUST be updated to 4 |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,500 |
| ADR-065 (full read) | ~3,000 |
| `.github/workflows/release.yml` | ~2,000 |
| `.github/workflows/ci.yml` | ~4,000 |
| `tests/ci-gate/test_AC-3_matrix-5-platforms.sh` | ~600 |
| `tests/release-gate/test_AC-006_matrix-targets.sh` | ~500 |
| `tests/release-gate/test_AC-012_install-scripts.sh` | ~500 |
| `scripts/install.sh` | ~1,500 |
| `RELEASING.md` | ~3,000 |
| Remaining docs (README.md, SETUP.md, RELEASE-CHANNELS.md, dev-setup.md) | ~2,000 |
| **Total** | **~20,600** |

Well within the 30% context window budget for a facade-mode devops story.

---

## Facade Verification Steps (replaces Red Gate for tdd_mode: facade)

Red Gate density check is N/A for `tdd_mode: facade`. The following 5 verification steps
constitute the facade readiness bar. All must pass before this story is merged.

| VF-ID | Command | Expected Result |
|-------|---------|----------------|
| VF-001 | `grep -r 'x86_64-apple-darwin\|macos-15-intel' .github/ scripts/ tests/ci-gate/ tests/release-gate/ docs/ rust-toolchain.toml README.md RELEASING.md CLAUDE.md CHANGELOG.md` | Zero matches (CHANGELOG.md scope: pending `## [VERSION]` section only; historical `## [X.Y.Z]` released sections are immutable and excluded) |
| VF-002 | `grep -r '5-platform\|5 build targets\|5 legs\|5 archive\|5 targets\|5 platform' .github/ docs/ RELEASING.md` | Zero matches |
| VF-003 | `bash tests/ci-gate/test_AC-3_matrix-4-platforms.sh` | Exit 0 (PASS) |
| VF-004 | `bash tests/release-gate/test_AC-006_matrix-targets.sh` | Exit 0 (PASS) |
| VF-005 | `bash tests/release-gate/test_AC-012_install-scripts.sh` | Exit 0 (PASS) |

**VF-001 grep scope exclusions (historical-immutable paths):**
- `docs/demo-evidence/` — IMMUTABLE per ADR-065 §Historical Records
- `.factory/cycles/` — IMMUTABLE adversarial review records
- `.factory/phase-0-ingestion/` — IMMUTABLE brownfield analysis
- `.factory/research/` — IMMUTABLE research cache
- `.factory/stories/` — merged story files (S-REL-003, W3-FIX-CI-001, etc.) are historical
- `CHANGELOG.md` historical sections — `## [X.Y.Z]` released sections are IMMUTABLE; only the pending `## [VERSION]` section (not yet tagged) is in scope for the grep gate

---

## Tasks

All tasks are devops workstream B (`.worktrees/E-REL-NOTES` or a new feature branch).
Read ADR-065 §Site Inventory in full before starting — it maps every required change.

**Preparation:**
1. Read ADR-065 full text (`.factory/specs/architecture/decisions/ADR-065-release-build-target-matrix.md`)
   to understand D1/D2/D3 and the complete Site Inventory.

**Workflows (CI/CD):**
2. Edit `.github/workflows/release.yml`:
   - Remove the `x86_64-apple-darwin` / `macos-15-intel` matrix leg (the entire `include:` entry)
   - Update line-1 comment from "5-platform builds" to "4-platform builds"

3. Edit `.github/workflows/ci.yml`:
   - Remove the `x86_64-apple-darwin` matrix `include:` entry (`runner: macos-15-intel`, `target: x86_64-apple-darwin`)
   - Remove the `macos-15-intel` timing comment (line citing `x86_64-apple-darwin` timing)
   - Update the `AC-5: 5 targets` assertion floor to `AC-5: 4 targets` (count floor 5→4)
   - Verify no other `x86_64-apple-darwin` or `macos-15-intel` references remain

4. Edit `.github/workflows/release-promote.yml`:
   - Update "full 5-platform build" → "full 4-platform build"

5. Edit `.github/workflows/release-tag.yml`:
   - Update all 3 "5-platform" references → "4-platform"

**Toolchain:**
6. Edit `rust-toolchain.toml`:
   - Remove `"x86_64-apple-darwin"` from the `targets` list

7. Edit `CLAUDE.md` (human-authorized per ADR-065 site inventory):
   - Remove `x86_64-apple-darwin` from the toolchain header target list
   - Update 5-target count reference if present

**Documentation:**
8. Edit `RELEASING.md`:
   - Remove the Intel mac row from the build matrix table (§5 or wherever the 5-row table lives)
   - Update "5 platform archives" → "4 platform archives"
   - Update "5 legs" → "4 legs"
   - Remove Intel macOS install instructions (§4 or wherever macOS Intel instructions live)

9. Edit `README.md`:
   - Remove "macOS Apple Silicon/Intel" → "macOS (Apple Silicon)"
   - Remove macOS Intel table row

10. Edit `docs/SETUP.md`:
    - Remove macOS Intel references and table row

11. Edit `docs/RELEASE-CHANNELS.md`:
    - Update "5-platform" → "4-platform" (2 occurrences in header/body)
    - Remove Intel table row from the build matrix table

12. Edit `docs/dev-setup.md`:
    - Remove the `x86_64-apple-darwin: PROPTEST_CASES` note

**Install scripts:**
13. Edit `scripts/install.sh`:
    - Remove the comment `x86_64-apple-darwin macOS (Intel)` near the top
    - Remove the `Darwin-x86_64` detection arm from the case statement
    - Verify the wildcard `*` error arm handles Darwin-x86_64 gracefully (prints error, exits 1)
    - Run `shellcheck scripts/install.sh` → 0 errors

**CI gate tests:**
14. Rename `tests/ci-gate/test_AC-3_matrix-5-platforms.sh` → `test_AC-3_matrix-4-platforms.sh`
    (`git mv` to preserve history):
    - Remove `x86_64-apple-darwin` from the TARGETS array
    - Remove `macos-15-intel` from the RUNNERS array
    - Update all "5 platform" count assertions to 4
    - Update script header comment

15. Edit `tests/ci-gate/README.md`:
    - Update "All 5 platform targets" → "All 4 platform targets"

16. Edit `tests/release-gate/test_AC-006_matrix-targets.sh`:
    - Remove `assert_contains ... x86_64-apple-darwin` assertion line
    - Update exact count from 5 to 4
    - Update "5-platform" header comment to "4-platform"

17. Edit `tests/release-gate/README.md`:
    - Update "5 platform targets" / "5 matrix entries" → 4

18. Edit `tests/release-gate/test_AC-012_install-scripts.sh`:
    - Remove `x86_64-apple-darwin` AC-002 assertion (Darwin-x86_64 detection arm no longer exists)
    - Update "5 targets" → "4 targets"

**Verification run:**
19. Run all 5 facade verification steps (VF-001 through VF-005) — all must exit 0 / zero hits.

---

## Acceptance Criteria

### AC-001: 4-target matrix reflected across all release/build/toolchain artifacts
Given: All modifications in Tasks 2–13 are applied.
When: The following grep is run:
```bash
grep -r 'x86_64-apple-darwin\|macos-15-intel' \
  .github/ scripts/install.sh rust-toolchain.toml \
  README.md RELEASING.md CLAUDE.md docs/ tests/ci-gate/ tests/release-gate/ \
  CHANGELOG.md
```
Then: Zero matches (excluding `.factory/stories/`, `docs/demo-evidence/`, `.factory/cycles/`,
`.factory/phase-0-ingestion/`, `.factory/research/`; for `CHANGELOG.md`: historical
`## [X.Y.Z]` released sections are immutable — only the pending `## [VERSION]` section
must be free of x86_64-apple-darwin/macos-15-intel references).
Additionally: `.github/workflows/release.yml` contains exactly 4 `include:` target entries;
`rust-toolchain.toml` lists exactly 4 targets; `scripts/install.sh` has no `Darwin-x86_64`
detection arm.
(traces to ADR-065 §D1 — all downstream artifacts MUST reflect the 4-target matrix)

### AC-002: x86_64-apple-darwin and macos-15-intel are permanently retired from non-historical files
Given: All modifications are committed to the feature branch.
When: The following forward-looking grep gate runs (in CI or locally):
```bash
grep -r 'x86_64-apple-darwin\|macos-15-intel' \
  .github/ scripts/ tests/ docs/ rust-toolchain.toml README.md RELEASING.md CLAUDE.md \
  2>/dev/null | grep -v 'demo-evidence\|cycles\|phase-0-ingestion\|research' | wc -l
```
Then: The count equals 0. Any future PR that reintroduces these strings in the scoped paths
must fail this gate check.
(traces to ADR-065 §D2 — x86_64-apple-darwin MUST NOT be reintroduced without a superseding ADR)

### AC-003: ADR-064 "5-platform" / "5 build targets" / "5 legs" stale counts are updated
Given: ADR-064 has been amended to v2.2 by the architect in this burst.
When: The following grep is run:
```bash
grep -r '5-platform\|5 build targets\|5 legs\|5 platform\|5 archive' \
  .factory/specs/architecture/decisions/ADR-064*.md
```
Then: Zero matches (all "5-platform" / "5 build targets" / "5 legs" references replaced
with "4-platform" / "4 build targets" / "4 legs").
Also: `grep -r '5-platform\|5 build targets\|5 legs' .github/ docs/ RELEASING.md` → zero matches
(the sweep is workspace-wide for forward-looking files).
(traces to ADR-065 §D3 — ADR-064 "5-platform" references MUST be updated)

### AC-004: CI-gate 4-platforms test passes; TARGET_COUNT floor updated to 4 in ci.yml
Given: Task 14 renamed the test file and Task 3 updated ci.yml.
When: The following verification steps run:
```bash
# (a) Renamed test file exists and old name does not:
test -f tests/ci-gate/test_AC-3_matrix-4-platforms.sh && echo PASS || echo FAIL
test ! -f tests/ci-gate/test_AC-3_matrix-5-platforms.sh && echo PASS || echo FAIL
# (b) Gate script passes:
bash tests/ci-gate/test_AC-3_matrix-4-platforms.sh
```
Then: Both `test` checks print PASS; gate script exits 0 with all assertions green.
Also: `grep 'AC-5.*4 target\|TARGET_COUNT.*4' .github/workflows/ci.yml` → at least 1 match
(the floor is updated from 5 to 4 in ci.yml).
(traces to ADR-065 §D1 — CI must reflect the 4-target matrix)

### AC-005: release-gate tests assert 4 targets and pass
Given: Tasks 16 and 18 updated the release-gate test scripts.
When: Both gate scripts run:
```bash
bash tests/release-gate/test_AC-006_matrix-targets.sh
bash tests/release-gate/test_AC-012_install-scripts.sh
```
Then: Both exit 0 with all assertions green.
Also: `grep 'x86_64-apple-darwin' tests/release-gate/test_AC-006_matrix-targets.sh
  tests/release-gate/test_AC-012_install-scripts.sh` → zero matches.
(traces to ADR-065 §D1 — release-gate tests must validate the 4-target matrix)

---

## Holdout Applicability

**HOLDOUT-N/A** — infra-only exemption (POL-35).

This story modifies CI workflow YAML, toolchain config, documentation, shell scripts, and
shell test gate files only. No Rust binary is changed; no MCP tool surface is modified;
no wire-level output is produced. There is no observable MCP output for a holdout evaluator
to exercise. This determination is consistent with prior infra-story holdout exemptions for
S-REL-CLIFF-001, S-REL-WRITER-001, and S-REL-BETA1-NOTES-001 (all `holdout_scenarios: []`).

---

## Previous Story Intelligence

**No story twin** — S-REL-DROP-INTEL-MAC-001 is the first and only story in the
ADR-065-anchored Intel-mac-drop workstream. No predecessor story in this workstream exists.

Related context from sibling stories:
- **S-REL-003** (merged PR #254): established the install.sh platform detection case statement
  (Darwin-arm64 → aarch64-apple-darwin; Darwin-x86_64 → x86_64-apple-darwin; etc.). Task 13
  of this story removes the `Darwin-x86_64` arm from that case statement.
- **W3-FIX-CI-001** (merged): established the per-target `proptest_cases` structure in ci.yml.
  The `x86_64-apple-darwin: proptest_cases: 256` entry added by that story is removed in Task 3.
- **ADR-065** (ACCEPTED v1.0 2026-09-07): authoritative decision record with a full Site
  Inventory (§Site Inventory) listing every file to update. Read it in full before starting.

Key lesson from ADR-065 §Consequences: Intel Mac users can build from source via
`cargo build --release`; the decision is about pre-built binary distribution only.
Install script changes (Task 13) must not suggest building from source — just error out cleanly
for Darwin-x86_64 and let users consult the GitHub README.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| 4-target matrix is the single source of truth | ADR-065 §D1 | AC-001 grep gate (VF-001) |
| x86_64-apple-darwin MUST NOT be reintroduced | ADR-065 §D2 | AC-002 grep gate (VF-002) |
| ADR-064 stale counts swept | ADR-065 §D3 | AC-003 grep gate |
| CI gate test renamed with git mv | This story Task 14 | Preserve git history — use `git mv` not delete+create |
| Historical records MUST NOT be touched | ADR-065 §Historical Records | VF-001 scope excludes demo-evidence, cycles, phase-0-ingestion, research, .factory/stories |
| CLAUDE.md edit is human-authorized | ADR-065 site inventory note | Authorized by human directive 2026-09-07 per ADR-065 |
| install.sh must remain shellcheck-clean | CLAUDE.md §Conventions + S-REL-003 AC-001 precedent | `shellcheck scripts/install.sh` → 0 errors (verify during Task 13) |

---

## Library & Framework Requirements

| Tool | Notes |
|------|-------|
| `shellcheck` | Must pass on `scripts/install.sh` after Task 13 edits |
| `bash` | Test gate scripts are bash; use POSIX-compatible features |
| `git mv` | Required for renaming `test_AC-3_matrix-5-platforms.sh` (Task 14) to preserve history |

No Rust crates are touched. No new dependencies are introduced.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/release.yml` | Modify | Remove Intel matrix leg; update "5-platform" comment |
| `.github/workflows/ci.yml` | Modify | Remove Intel matrix leg; update TARGET_COUNT floor to 4 |
| `.github/workflows/release-promote.yml` | Modify | "5-platform" → "4-platform" |
| `.github/workflows/release-tag.yml` | Modify | All 3 "5-platform" → "4-platform" |
| `rust-toolchain.toml` | Modify | Remove `"x86_64-apple-darwin"` from targets |
| `CLAUDE.md` | Modify | Remove `x86_64-apple-darwin` from toolchain target list (human-authorized) |
| `RELEASING.md` | Modify | Remove Intel row, update counts |
| `README.md` | Modify | Remove macOS Intel references |
| `docs/SETUP.md` | Modify | Remove macOS Intel references |
| `docs/RELEASE-CHANNELS.md` | Modify | "5-platform" → "4-platform", remove Intel table row |
| `docs/dev-setup.md` | Modify | Remove x86_64-apple-darwin PROPTEST_CASES note |
| `scripts/install.sh` | Modify | Remove Intel comment + Darwin-x86_64 detection arm |
| `tests/ci-gate/test_AC-3_matrix-5-platforms.sh` | Rename (git mv) | → `test_AC-3_matrix-4-platforms.sh`; update content |
| `tests/ci-gate/test_AC-3_matrix-4-platforms.sh` | Modify (after rename) | Remove Intel target/runner; update counts to 4 |
| `tests/ci-gate/README.md` | Modify | "5 platform" → "4 platform" |
| `tests/release-gate/test_AC-006_matrix-targets.sh` | Modify | Remove Intel assertion; count 5→4 |
| `tests/release-gate/README.md` | Modify | "5 platform" → "4 platform" |
| `tests/release-gate/test_AC-012_install-scripts.sh` | Modify | Remove x86_64-apple-darwin assertion; count 5→4 |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `.github/workflows/release.yml` | CI/CD | N/A (YAML config) |
| `.github/workflows/ci.yml` | CI/CD | N/A (YAML config) |
| `scripts/install.sh` | `scripts/` | N/A (shell script) |
| `tests/ci-gate/` | `tests/` | N/A (shell test scripts) |
| `tests/release-gate/` | `tests/` | N/A (shell test scripts) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| All modified files | N/A | CI/docs/scripts — no Rust purity boundary applies |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Darwin-x86_64 user runs `scripts/install.sh` after this story ships | Case arm removed; wildcard `*` arm fires: "Unsupported platform: Darwin-x86_64" to stderr + exit 1 |
| EC-002 | Future PR adds x86_64-apple-darwin to release.yml without a new ADR | VF-001 grep gate catches it; PR must be rejected until a superseding ADR exists per ADR-065 §D2 |
| EC-003 | ci.yml references old test file name `test_AC-3_matrix-5-platforms.sh` | Task 14 uses `git mv`; verify ci.yml is updated to reference new filename before merge |
| EC-004 | "5-platform" appears in a historical .factory/cycles/ record | Expected and allowed — historical records are IMMUTABLE; not a violation |
| EC-005 | ADR-064 retains a non-count "5" reference (e.g., "5 steps", "5 invariants") | Expected — grep is specific: "5-platform", "5 build targets", "5 legs", "5 archive"; generic "5" is not in scope |

---

## Forbidden Dependencies

- Do NOT modify any file under `docs/demo-evidence/`, `.factory/cycles/`,
  `.factory/phase-0-ingestion/`, `.factory/research/` — these are IMMUTABLE historical records
- Do NOT modify completed merged story files under `.factory/stories/` (S-0.01, S-0.02,
  S-REL-001, S-MAINT-CI-DISK-EXHAUSTION-001, etc.) — see ADR-065 §Historical Records
- Do NOT add `x86_64-apple-darwin` as a "community support" or "build from source" option
  in install.sh — human confirmed a clean drop with no tiered support (ADR-065 §Alternatives)

---

## TD-VSDD-097 Discharge

**Dim-1 (sibling pair):** S-REL-DROP-INTEL-MAC-001 has no story twin — it is the sole story
in the ADR-065-anchored Intel-mac-drop workstream. No sibling pair exists. CLEAR.

**Dim-2 (downstream copy targets):** Story-writer is the Dim-2 sweep executor for ADR-065
§Site Inventory `.factory/` rows. All `.factory/stories/` rows swept in this burst:
- S-REL-003: SUPERSESSION NOTE added (merged, historical ACs preserved) — Task 13 context
- S-REL-004: Target table updated (draft, not merged) — Intel row removed from target table section
- S-REL-005: SUPERSESSION NOTE added (merged, historical ACs preserved)
- W3-FIX-CI-001: Intel proptest_cases entry removed from task list (merged, but parked entry)
- STORY-INDEX.md: S-MAINT-EDITION-SYNC-001 row updated (3 cross-compile targets); new story row added

`.factory/planning/feature-release-engineering/delta-analysis.md` (macOS x86_64 row): DEFERRED
to spec-steward per correct-agent routing — this is a planning document outside story-writer scope.
The architect labeled this site "Spec-steward: remove row" in ADR-065 §Site Inventory.

`.factory/release-config.yaml`: ALREADY UPDATED by architect in the same burst as ADR-065
(per ADR-065 site inventory "Updated in this burst (architect)"). No story-writer action needed.

All other ADR-065 inventory rows (workflows, toolchain, docs, scripts, CI gate tests) are
devops-engineer workstream B — executed during story implementation, not story authoring.

Dim-2 COMPLETE (all .factory/ rows discharged or routed to correct specialist).

**Dim-3 (mandate anchors):** All ADR-065 MUSTs are anchored:
- ADR-065 §D1 MUST → S-REL-DROP-INTEL-MAC-001 AC-001 (and VF-001/VF-003/VF-004/VF-005)
- ADR-065 §D2 MUST → S-REL-DROP-INTEL-MAC-001 AC-002 (and VF-001/VF-002)
- ADR-065 §D3 MUST → S-REL-DROP-INTEL-MAC-001 AC-003

All MUSTs anchored to this story + specific ACs. COMPLETE.

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 1.1 | 2026-09-07 | TD-VSDD-097 Dim-2 downstream sweep (ADR-065 v1.1 §Site Inventory). AC-001 + VF-001: added CHANGELOG.md to grep scope — pending `## [VERSION]` section must not reference x86_64-apple-darwin or macos-15-intel; historical `## [X.Y.Z]` released sections are immutable/excluded from the gate. VF-001 scope exclusions: CHANGELOG.md historical section immutability noted. |
| 1.0 | 2026-09-07 | Initial story. Human-directed ADR-065 v1.0: drop x86_64-apple-darwin (Intel mac); 4-target matrix; 5 ACs anchoring D1/D2/D3; facade mode (CI/docs/shell-test-only); holdout_scenarios: [] HOLDOUT-N/A (infra-only); subsystems: [SS-22]; crates_touched: []; depends_on: []; blocks: []. TD-VSDD-097: Dim-1 CLEAR (no sibling twin); Dim-2 COMPLETE (.factory/ rows swept — S-REL-003/004/005 supersession notes + W3-FIX-CI-001 Intel entry removed + STORY-INDEX S-MAINT-EDITION-SYNC-001 row updated; delta-analysis.md deferred to spec-steward); Dim-3 COMPLETE (all ADR-065 MUSTs anchored to AC-001/AC-002/AC-003). |
