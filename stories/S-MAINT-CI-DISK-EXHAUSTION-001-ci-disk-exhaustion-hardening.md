---
document_type: story
story_id: S-MAINT-CI-DISK-EXHAUSTION-001
title: "CI disk-exhaustion hardening — preflight + disk-space-reclaimer + cargo-config debug-invariant guard + failure annotation"
wave: tbd
epic_id: maintenance
priority: P2
status: ready
version: "0.19"
level: ops
producer: story-writer
timestamp: "2026-07-15"
modified: "2026-07-16"
input-hash: "[live-state]"
inputs:
  - .github/workflows/ci.yml
traces_to: ""
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
# Subsystem anchor: this story touches only the CI toolchain (.github/workflows/ci.yml),
# not any functional product subsystem. No SS-NN anchor is appropriate; devops/CI stories
# (e.g., W3-FIX-CI-001, S-MAINT-REQWEST-RUSTLS-GATE-001) follow the same pattern of
# subsystems: [] per the ARCH-INDEX Subsystem Registry, which defines SS-01..22 as product
# subsystems only. The CI toolchain has no registered SS-NN.
crates_touched: []
target_module: devops
behavioral_contracts: []
# BC status: CONFORMING (no BC required). PO adjudication 2026-07-15 (Option B).
#
# This story is CI toolchain-only: it modifies .github/workflows/ci.yml (runner
# disk reclaim, preflight, CARGO_PROFILE_DEV_DEBUG env, failure annotation) and
# adds structural assertions to the verify-workflow-structure job. No product
# subsystem (SS-01..SS-22) is touched; no production behavior observable by an
# MCP client is affected.
#
# Controlling precedent: W3-FIX-CI-001 (merged, PR #112, behavioral_contracts: [])
# shipped without BCs under the same rationale ("no formal BC governs dev toolchain
# speed. This story is tooling-only; no production behavior is affected."). That
# story was accepted as merged — establishing the project's no-BC convention for
# CI-toolchain-only maintenance stories.
#
# Why no BC can be authored in good conscience:
#   1. Every BC in this project anchors to a CAP-NNN from capabilities.md. No
#      CAP-NNN covers CI runner disk management, GitHub Actions workflow structure,
#      or developer-experience toolchain concerns. Inventing one would fabricate a
#      domain capability with no L2 domain spec basis.
#   2. ARCH-INDEX Subsystem Registry (SS-01..SS-22) contains product subsystems
#      only. No CI/devops subsystem exists to anchor a BC subsystem: field.
#   3. The verify-workflow-structure RED GATE tests embedded in ci.yml (AC-001,
#      AC-002) ARE the correct VSDD artifact for CI structural invariants — they
#      are self-describing CI assertions living inside the workflow, not product
#      behavioral contracts.
#   4. Authoring a BC for "CI disk ≥25 GB after reclaim" under a fabricated
#      CAP-NNN would violate the Capability Anchor Justification rule (S-7.01
#      Semantic Anchoring Audit) more severely than the current empty state.
#
# S-MAINT-REQWEST-RUSTLS-GATE-001 is in the same situation and benefits from the
# same ratification (its "pending PO authorship" comment is also CONFORMING under
# this precedent; state-manager should update it in the same burst).
#
# behavioral_contracts: [] is CONFORMING for CI-toolchain-only stories.
# The S-7.01 draft-blocker is RESOLVED by this adjudication.
verification_properties: []
depends_on: []
blocks: []
points: 5
estimated_days: 1
risk: MEDIUM
acceptance_criteria_count: 7
red_gate_tests: 9
estimated_passes: "1"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
triggered_by: "D-1780 watch-note (3rd disk-exhaustion occurrence on PR #223 CI runs 2026-07-15)"
---

# S-MAINT-CI-DISK-EXHAUSTION-001: CI disk-exhaustion hardening — preflight + disk-space-reclaimer + cargo-config debug-invariant guard + failure annotation

## §Origin

D-1780 recorded a watch-note: "3rd disk-exhaustion occurrence → maintenance story." This
story materializes that commitment. Three consecutive GitHub-hosted-runner disk-full
failures occurred on PR #223 during the `cargo nextest run --workspace --all-features
--profile ci` build phase — before any test executed:

| Run | Target | HEAD | Error |
|-----|--------|------|-------|
| 29394488318 | x86_64-unknown-linux-musl | 76c0fa60 (2026-07-15) | `mold: failed to write to an output file. Disk full?` + `rustc-LLVM ERROR: IO failure on output stream: No space left on device` |
| 29399778005 | x86_64-unknown-linux-gnu | 72d8ed8d (2026-07-15) | `couldn't create a temp dir: No space left on device (os error 28)` |
| 29404746333 | x86_64-unknown-linux-gnu | 97cb070e (2026-07-15) | `mold: Disk full` + `collect2: fatal error: ld terminated with signal 7 [Bus error]` |

All three re-runs succeeded. Failures occurred exclusively on Linux legs (musl + gnu),
which use the mold linker (W3-FIX-CI-001) and carry the largest debug-info artifact
footprint.

## §Root Cause Hypothesis

GitHub documents 14 GB as the guaranteed MINIMUM free disk space for `ubuntu-latest`
hosted runners; empirical ubuntu-24.04 measurements show ~22–29 GB free at job start
on a ~72 GB root filesystem. The observed failures indicate runners landing near the
documented floor. A 26-crate Rust workspace compiled in dev mode generates large `.d`,
`.rlib`, `.rmeta`, and `.o` artifacts under `target/`. The `Swatinem/rust-cache` action
restores prior artifacts, which reduces compile time but increases the effective starting
disk footprint — the cache restore consumes disk before the incremental build begins.
Under high-concurrency PR activity, a runner may land near or below the 14 GB guaranteed
minimum.

**Note on the debug-info axis:** `.cargo/config.toml` already set `debug = "line-tables-only"`
for first-party code and `debug = false` for all dependencies via `[profile.dev.package."*"]`
*prior to this story and prior to the observed failures*. The debug-info axis therefore provides
no available mitigation lever — it is a pre-existing invariant that AC-003 guards against
regression, not a new reduction. The `CARGO_PROFILE_DEV_DEBUG` env var MUST NOT be added to
`ci.yml` because `.cargo/config.toml` already sets the same values at higher precedence and
the env var would change nothing observable while misleading future readers (F-CIDISK-P4-HIGH-001,
adjudicated 2026-07-15). The effective levers are pre-build disk reclaim (AC-002) + early-fail
gate (AC-002 ≥25 GB) + preflight (AC-001) + failure annotation (AC-004).

Re-runs succeed because the failed run wrote partial artifacts to the cache; the
re-run increments on those artifacts and does less linking work, consuming less peak
disk.

**This is load-dependent and therefore flaky** — it does not occur on every run, only
when the runner disk is near the threshold.

## §Narrative

As a Prism CI maintainer, I want the Linux Test jobs to reliably complete without
disk-exhaustion failures so that contributors are not blocked by spurious CI failures
that require manual re-runs and obscure genuine test failures.

## §Acceptance Criteria

### AC-001 — Disk-free preflight step present in Linux Test job legs and test-no-default-features

A step named "Report initial disk space" (or equivalent) runs at the START of the Linux
Test job legs AND the `test-no-default-features` job (both ubuntu-latest Linux runners;
macOS/Windows legs are exempt), before `actions/checkout`, and emits `df -h` output to
the job log. This provides a baseline disk snapshot for every run, enabling post-hoc
analysis of whether a future failure was disk-related.

The `verify-workflow-structure` job gains an assertion confirming this step is present in
both Linux workspace-build jobs in `ci.yml` (Red Gate test 1 — count ≥ 2):

```bash
# Anchored to YAML step-name syntax (^\s+- name:) — indent-agnostic and self-match-proof.
# Count-based: must appear in ≥2 Linux jobs (test matrix + test-no-default-features).
# The assertion line starts with whitespace+count=$(grep..., not whitespace+"- name:", so
# the ^ anchor cannot self-match.
count=$(grep -cE '^\s+- name: Report initial disk space\s*$' .github/workflows/ci.yml)
[ "$count" -ge 2 ] || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-001: disk preflight step missing from ≥2 Linux jobs (found ${count}; need test (Test matrix) + test-no-default-features)"
  exit 1
}
```

### AC-002 — Pre-build disk reclaim targeting ≥25 GB free (Linux Test legs + test-no-default-features)

A disk-reclaim step using `insightsengineering/disk-space-reclaimer@dae9fabcb8febe09f6585471948acf9dc9a57489 # v1.1.2`
(actively-maintained drop-in fork of jlumbroso/free-disk-space; upstream unmaintained
since 2023-10) runs in the Linux Test job legs AND the `test-no-default-features` job
(both ubuntu-latest Linux runners; macOS/Windows legs are exempt) after `actions/checkout`
but before the `Swatinem/rust-cache` restore and build phase. It removes preinstalled
toolsets not needed by the Rust build with inputs: `android: true`, `dotnet: true`,
`haskell: true`, `docker-images: true`, `large-packages: true`, `swap-storage: false`.
Total expected reclaim: 21–31 GB on ubuntu-24.04 with these inputs enabled (swap
preserved as OOM headroom; see EC-008).

**The reclaimer step MUST include `continue-on-error: true` (BEST-EFFORT semantic).** The
`large-packages: true` input invokes `apt-get` against the runner's rotating apt mirror;
that mirror can return HTTP 404 Release files on mirror rotation (evidence: CI run
29437306537, 2026-07-15, mirror.enzu.com returned 404 Release files, apt exit code 100).
With the default `continue-on-error: false`, ALL THREE hardened Linux jobs failed on the
first live CI run BEFORE the ≥25 GB gate even had a chance to assess actual disk state.
The reclaimer step differs only in step name between the two Linux jobs (matching
AC-007 precedent from v0.12). The `with:` block inputs are identical in both cases
and use unquoted YAML booleans (matching the implemented ci.yml form):

**Test-matrix job (step name: `Reclaim disk space (Linux only)`):**

```yaml
- name: Reclaim disk space (Linux only)
  uses: insightsengineering/disk-space-reclaimer@dae9fabcb8febe09f6585471948acf9dc9a57489 # v1.1.2
  # Reclaim is BEST-EFFORT: continue-on-error absorbs apt-mirror 404s and other transient
  # reclaimer failures. Evidence: run 29437306537 (2026-07-15) — mirror.enzu.com returned
  # HTTP 404 Release files on the large-packages apt path; apt exit code 100 caused the
  # reclaimer step to fail, which failed ALL THREE hardened Linux jobs before the ≥25 GB
  # gate ran. The ≥25 GB gate (next step) is the sole authoritative arbiter of disk
  # readiness — it fails loud with the actual free-GB count if reclaim genuinely
  # under-delivers. Trade-off: continue-on-error masks persistent action breakage (e.g.,
  # action update breaking the inputs API); mitigated because the gate provides ground-truth
  # disk verification on every run.
  continue-on-error: true
  with:
    android: true
    dotnet: true
    haskell: true
    docker-images: true
    large-packages: true
    swap-storage: false
```

**`test-no-default-features` job (step name: `Reclaim disk space`):**

```yaml
- name: Reclaim disk space
  uses: insightsengineering/disk-space-reclaimer@dae9fabcb8febe09f6585471948acf9dc9a57489 # v1.1.2
  # Reclaim is BEST-EFFORT: continue-on-error absorbs apt-mirror 404s and other transient
  # reclaimer failures. Evidence: run 29437306537 (2026-07-15) — mirror.enzu.com returned
  # HTTP 404 Release files on the large-packages apt path; apt exit code 100 caused the
  # reclaimer step to fail, which failed ALL THREE hardened Linux jobs before the ≥25 GB
  # gate ran. The ≥25 GB gate (next step) is the sole authoritative arbiter of disk
  # readiness — it fails loud with the actual free-GB count if reclaim genuinely
  # under-delivers. Trade-off: continue-on-error masks persistent action breakage (e.g.,
  # action update breaking the inputs API); mitigated because the gate provides ground-truth
  # disk verification on every run.
  continue-on-error: true
  with:
    android: true
    dotnet: true
    haskell: true
    docker-images: true
    large-packages: true
    swap-storage: false
```

Documented fallback: `jlumbroso/free-disk-space@54081f138730dfa15788a46383842cd2f914a1be # v1.3.1`
(upstream, unmaintained since 2023-10). If the fork diverges or becomes unmaintained,
revert to this pin. The `tool-cache` / `tools-cache` input (renamed in the fork) is
NOT removed — this story does not remove tool cache; leave the input unset.

A verification step immediately after the reclaim step confirms at least 25 GB free
via `df -h`. If post-reclaim free space is below 25 GB, the job fails early with a
diagnostic message rather than proceeding to a late-stage OOM/disk-full linker crash:

```bash
# df -P / output: Filesystem, 1K-blocks, Used, Available, Use%, Mounted
# Available ($4) is in 1K-blocks; divide by (1024*1024) to convert to GiB
AVAIL_GB=$(df -P / | awk 'NR==2 { print int($4 / 1024 / 1024) }')
AVAIL_GB=${AVAIL_GB:-0}  # Guard: awk returns empty if df fails; default 0 prevents -ge test failure
[ "$AVAIL_GB" -ge 25 ] || {
  df -h
  echo "::error::Disk reclaim insufficient: only ${AVAIL_GB} GB free (need ≥25 GB). See S-MAINT-CI-DISK-EXHAUSTION-001."
  exit 1
}
```

The `verify-workflow-structure` job gains an assertion confirming the disk-space-reclaimer
action is present in both Linux workspace-build jobs in `ci.yml` (Red Gate test 2 — count ≥ 2):

```bash
# Anchored to YAML uses: key syntax (^\s+uses:) — self-match-proof.
# Count-based: must appear in ≥2 Linux jobs (test matrix + test-no-default-features).
# The assertion line starts with whitespace+count=$(grep..., not whitespace+"uses:", so
# the ^ anchor cannot self-match.
count=$(grep -cE '^\s+uses: insightsengineering/disk-space-reclaimer' .github/workflows/ci.yml)
[ "$count" -ge 2 ] || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-002: disk-space-reclaimer missing from ≥2 Linux jobs (found ${count}; need test (Test matrix) + test-no-default-features)"
  exit 1
}
```

**AC-002 adjudicated no-action items (PR-LEVEL pass-3, 2026-07-15):**
- **F-CIDISK-PR3-OBS-002 (AC-006 ≥12 threshold):** The `count ≥ 12` threshold is BY DESIGN — it encodes the exact number of `if ! sudo apt-get update; then` lines present at time of v0.14 ratification (10 apt-install sites + 2 AC-007 toolchain installs; F-CIDISK-PR1-MED-002 POL-34 slack elimination). Future legitimate site additions must update the threshold, error echo, and pass echo in lock-step; future site removals likewise.
- **F-CIDISK-PR3-OBS-003 (AC-007 step-position asymmetry):** The AC-007 step position differs between the two Linux jobs relative to the libdbus/rust-cache steps. This is intentional and within the spec's ordering contract: "after checkout + reclaim, before any cargo build phase." The relative position to libdbus and rust-cache within that window is unconstrained by this story and does not constitute drift.
- **F-CIDISK-PR3-OBS-004 (reclaimer continue-on-error masking):** The `continue-on-error: true` on the reclaimer step is the EC-009-ratified trade-off. The ≥25 GB gate is the authoritative arbiter of disk readiness; it fails loud with the actual free-GB count if reclaim genuinely under-delivers. This masking design is load-bearing — no change required.

### AC-003 — `.cargo/config.toml` minimal-DWARF invariant guarded in verify-workflow-structure

The `.cargo/config.toml` minimal-DWARF configuration (present and active *prior to this story
and prior to the observed failures*) is protected by regression assertions in the
`verify-workflow-structure` job. Specifically:

- `[profile.dev] debug = "line-tables-only"` — line-tables-only DWARF for first-party code
- `[profile.dev.package."*"] debug = false` — no debug info for all dependencies (stronger override)

The `CARGO_PROFILE_DEV_DEBUG` env var MUST NOT be added to `ci.yml`. It is a no-op:
`.cargo/config.toml` already sets `debug = "line-tables-only"` at the `[profile.dev]` level
and `debug = false` for all dependencies via `[profile.dev.package."*"]`, which takes precedence
over any env-var override for the dep packages. Adding the env var would change nothing observable
while misleading future readers into believing CI performs a debug-info reduction that was already
active before this story shipped (F-CIDISK-P4-HIGH-001, adjudicated 2026-07-15).

The `verify-workflow-structure` job gains two assertions confirming these invariants in
`.cargo/config.toml` (Red Gate tests 3 and 4):

```bash
# Assertion 3: [profile.dev] section must contain debug = "line-tables-only".
# Section-scoped awk: enters on exact [profile.dev] header, exits on any other [.
# Context-aware: a stray debug = "line-tables-only" in another section does not satisfy.
# Self-match impossible (file is .cargo/config.toml, not ci.yml).
awk '/^\[profile\.dev\]$/{s=1;next} /^\[/{s=0} s && /^debug = "line-tables-only"$/{found=1} END{exit !found}' .cargo/config.toml || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-003: .cargo/config.toml [profile.dev] section missing debug = \"line-tables-only\" — this disk-footprint invariant must not be removed"
  exit 1
}
# Assertion 4: [profile.dev.package."*"] section must contain debug = false.
# Section-scoped awk: enters on exact [profile.dev.package."*"] header, exits on any other [.
# Context-aware: section header alone is insufficient — the debug = false payload must be present.
# Self-match impossible (file is .cargo/config.toml, not ci.yml).
awk '/^\[profile\.dev\.package\."\*"\]$/{s=1;next} /^\[/{s=0} s && /^debug = false$/{found=1} END{exit !found}' .cargo/config.toml || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-003: .cargo/config.toml [profile.dev.package.\"*\"] section missing debug = false — dependency debug-info override must not be removed"
  exit 1
}
```

The implementer does NOT record `du -sh target/` sizes in the PR description — that requirement
was tied to the now-superseded env-var approach and has no bearing on a pre-existing config invariant.

### AC-004 — Disk-exhaustion failure annotation step (Linux Test legs + test-no-default-features)

A `if: failure()` step at the END of the Linux Test job legs AND the `test-no-default-features`
job (both ubuntu-latest Linux runners; macOS/Windows legs are exempt) runs `df -h`
and emits a `::warning::` workflow command if disk utilization is above 95%:

```bash
USED_PCT=$(df -P / | awk 'NR==2 { gsub(/%/, "", $5); print $5 }')
USED_PCT=${USED_PCT:-0}  # Guard: awk returns empty if df fails; default 0 prevents -ge test failure
df -h
if [ "${USED_PCT}" -ge 95 ]; then
  echo "::warning::Disk near-full (${USED_PCT}% used) on runner — likely S-MAINT-CI-DISK-EXHAUSTION-001 class failure. Evidence runs: 29394488318 / 29399778005 / 29404746333 (PR #223, 2026-07-15)."
fi
```

This makes disk-exhaustion failures distinguishable in the GitHub Actions job summary
from genuine test failures without requiring log scraping.

### AC-005 — Regression evidence: three consecutive green CI runs

After the `.github/workflows/ci.yml` changes are pushed to a maintenance branch
`maintenance/ci-disk-hardening`, three consecutive full-workspace PR CI runs complete
green (all Linux legs pass without re-run) before the PR is merged. The PR description
records the three GitHub Actions run IDs as evidence.

**NOTE (v0.19):** The following runs are DISQUALIFIED as green evidence: run 29524703679 attempt-1 on HEAD bd65e93a (F-MAINT-P9-HIGH-001/002, 2026-07-16); runs 29531645116 and 29531648104 (both attempt-1 on HEAD 0939973f, 2026-07-16) — both failed inside the wrapped steps because the v0.18 fallback was a structural no-op on image 20260714.240.1 (F-MAINT-P10-CRIT-001). The three-consecutive-green record restarts on the post-fix HEAD after v0.19 spec changes are applied to the implementation.

**Valid run acquisition (F-MAINT-P10-MED-004):** Three consecutive full-workspace PR CI runs means three DISTINCT GitHub Actions run IDs triggered by separate events — e.g., three distinct push-synchronize events to `maintenance/ci-disk-hardening` while the PR is open, or three workflow_dispatch triggers. Attempt-2 and attempt-3 re-runs of a failed run ID do NOT qualify (re-run ≠ distinct run). Push-event runs on the PR branch qualify as "PR CI runs" (F-MAINT-P10-OBS-008 adjudication: any GitHub Actions run triggered by a push to the PR branch counts; the intent is three independent trigger events proving stability across distinct runner allocations, not merely three attempts against the same commit).

The changes in this story MUST ride `maintenance/ci-disk-hardening` and NOT be merged
through any open defect PR branch (e.g., `fix/DEFECT-*`), to keep CI infrastructure
changes cleanly bisectable.

### AC-006 — apt-mirror resilience for pre-existing apt install steps (test matrix + test-no-default-features)

The ten `sudo apt-get update && sudo apt-get install ...` steps across the CI pipeline are
vulnerable to the same apt-mirror 404 class that motivated EC-009 (evidence: run
29438854846 rerun, 2026-07-15 — both the `musl-tools` step in the Test matrix AND the
`libdbus-1-dev pkg-config` step in `test-no-default-features` failed identically after
the reclaimer's `continue-on-error: true` fix had already landed; the broken mirror
blocked ALL remaining apt install steps in the same CI run). Each of the ten steps
MUST be converted from the single-attempt inline form to the two-attempt pattern below.
The reclaimer step itself is NOT given this wrapper — it already carries `continue-on-error:
true` best-effort semantics per EC-009; the ≥25 GB gate is the authoritative check there.

**Rationale for 7-job extension (F-CIDISK-PR1-OBS-001):** EC-010's mirror-flake class
affects ANY job that calls `sudo apt-get update`, not just the test-matrix jobs. `clippy`
is a `needs:` predecessor of several pipeline stages whose failure would block AC-005
evidence collection across the whole pipeline. A partial sweep (3-site) leaves 7 known-flaky
apt steps unresilient. Full sweep required per Canonical Principle Rule 4 (no deferral when
the fix scope is deterministic and bounded).

**Snippet — `Install musl-tools` (test matrix; `if: matrix.install_musl`):**

```yaml
      - name: Install musl-tools
        if: matrix.install_musl
        run: |
          # Apt-mirror resilience: combined two-attempt pattern (F-MAINT-P8-LOW-003).
          # Evidence: runs 29437306537 + 29438854846 rerun (PR #224, 2026-07-15) — mirror.enzu.com
          # returned HTTP 404 Release files; apt exit code 100. Runner-image mirror selection is
          # outside our control; on failure overwrite apt-mirrors.txt to pin canonical archive
          # and remove third-party source files — sed URL-rewriting is a structural no-op on
          # image 20260714.240.1 (ubuntu.sources uses mirror+file:, not http://; F-MAINT-P10-CRIT-001).
          # The outer condition wraps both update AND install so a mirror-failure during package
          # fetch ("E: Unable to fetch some archives") also triggers the fallback.
          if ! ( sudo apt-get update && sudo apt-get install -y musl-tools ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y musl-tools
          fi
```

**Snippet — `Install libdbus-1-dev (required by keyring build on Linux, all-features)` (test matrix; `if: runner.os == 'Linux'`):**

```yaml
      - name: Install libdbus-1-dev (required by keyring build on Linux, all-features)
        if: runner.os == 'Linux'
        run: |
          # Apt-mirror resilience: combined two-attempt pattern (F-MAINT-P8-LOW-003).
          # Evidence: runs 29437306537 + 29438854846 rerun (PR #224, 2026-07-15) — mirror.enzu.com
          # returned HTTP 404 Release files; apt exit code 100. Runner-image mirror selection is
          # outside our control; on failure overwrite apt-mirrors.txt to pin canonical archive
          # and remove third-party source files — sed URL-rewriting is a structural no-op on
          # image 20260714.240.1 (ubuntu.sources uses mirror+file:, not http://; F-MAINT-P10-CRIT-001).
          # The outer condition wraps both update AND install so a mirror-failure during package
          # fetch ("E: Unable to fetch some archives") also triggers the fallback.
          if ! ( sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y libdbus-1-dev pkg-config
          fi
```

**Snippet — `Install libdbus-1-dev (required by keyring Secret Service backend on Linux)` (test-no-default-features; unconditional):**

```yaml
      - name: Install libdbus-1-dev (required by keyring Secret Service backend on Linux)
        run: |
          # Apt-mirror resilience: combined two-attempt pattern (F-MAINT-P8-LOW-003).
          # Evidence: runs 29437306537 + 29438854846 rerun (PR #224, 2026-07-15) — mirror.enzu.com
          # returned HTTP 404 Release files; apt exit code 100. Runner-image mirror selection is
          # outside our control; on failure overwrite apt-mirrors.txt to pin canonical archive
          # and remove third-party source files — sed URL-rewriting is a structural no-op on
          # image 20260714.240.1 (ubuntu.sources uses mirror+file:, not http://; F-MAINT-P10-CRIT-001).
          # The outer condition wraps both update AND install so a mirror-failure during package
          # fetch ("E: Unable to fetch some archives") also triggers the fallback.
          if ! ( sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y libdbus-1-dev pkg-config
          fi
```

**Snippet — `clippy` job (`if: runner.os == 'Linux'`):**

```yaml
      - name: Install libdbus-1-dev (required by keyring build on Linux)
        if: runner.os == 'Linux'
        run: |
          # Apt-mirror resilience: combined two-attempt pattern per AC-006 (F-MAINT-P8-LOW-003).
          # Evidence: EC-010 mirror-flake class; clippy is a needs: predecessor blocking pipeline.
          # The outer condition wraps both update AND install so a mirror-failure during package
          # fetch ("E: Unable to fetch some archives") also triggers the fallback.
          if ! ( sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y libdbus-1-dev pkg-config
          fi
```

**Snippet — `semver-checks` job (`if: runner.os == 'Linux'`):**

```yaml
      - name: Install libdbus-1-dev (required by keyring build on Linux)
        if: runner.os == 'Linux'
        run: |
          # Apt-mirror resilience: combined two-attempt pattern per AC-006 (F-MAINT-P8-LOW-003).
          # The outer condition wraps both update AND install so a mirror-failure during package
          # fetch ("E: Unable to fetch some archives") also triggers the fallback.
          if ! ( sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y libdbus-1-dev pkg-config
          fi
```

**Snippet — `fuzz-smoke-vp021` job (`if: runner.os == 'Linux'`):**

```yaml
      - name: Install libdbus-1-dev (required by keyring build on Linux)
        if: runner.os == 'Linux'
        run: |
          # Apt-mirror resilience: combined two-attempt pattern per AC-006 (F-MAINT-P8-LOW-003).
          # The outer condition wraps both update AND install so a mirror-failure during package
          # fetch ("E: Unable to fetch some archives") also triggers the fallback.
          if ! ( sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y libdbus-1-dev pkg-config
          fi
```

**Snippet — `perimeter-compile-fail` job (`if: runner.os == 'Linux'`):**

```yaml
      - name: Install libdbus-1-dev (required by keyring build on Linux)
        if: runner.os == 'Linux'
        run: |
          # Apt-mirror resilience: combined two-attempt pattern per AC-006 (F-MAINT-P8-LOW-003).
          # The outer condition wraps both update AND install so a mirror-failure during package
          # fetch ("E: Unable to fetch some archives") also triggers the fallback.
          if ! ( sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y libdbus-1-dev pkg-config
          fi
```

**Snippet — `non-exhaustive-violation-compile-fail` job (`if: runner.os == 'Linux'`):**

```yaml
      - name: Install libdbus-1-dev (required by keyring build on Linux)
        if: runner.os == 'Linux'
        run: |
          # Apt-mirror resilience: combined two-attempt pattern per AC-006 (F-MAINT-P8-LOW-003).
          # The outer condition wraps both update AND install so a mirror-failure during package
          # fetch ("E: Unable to fetch some archives") also triggers the fallback.
          if ! ( sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y libdbus-1-dev pkg-config
          fi
```

**Snippet — `no-hardcoded-sensors-compile-fail` job (`if: runner.os == 'Linux'`):**

```yaml
      - name: Install libdbus-1-dev (required by keyring build on Linux)
        if: runner.os == 'Linux'
        run: |
          # Apt-mirror resilience: combined two-attempt pattern per AC-006 (F-MAINT-P8-LOW-003).
          # The outer condition wraps both update AND install so a mirror-failure during package
          # fetch ("E: Unable to fetch some archives") also triggers the fallback.
          if ! ( sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y libdbus-1-dev pkg-config
          fi
```

**Snippet — `shellcheck-demo-scripts` job (unconditional — ubuntu-only job):**

```yaml
      - name: Install shellcheck
        run: |
          # Apt-mirror resilience: combined two-attempt pattern per AC-006 (F-MAINT-P8-LOW-003).
          # The outer condition wraps both update AND install so a mirror-failure during package
          # fetch ("E: Unable to fetch some archives") also triggers the fallback.
          if ! ( sudo apt-get update && sudo apt-get install -y shellcheck ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y shellcheck
          fi
```

**e2e.yml scope extension (F-MAINT-P8-MED-004):** The e2e workflow (`.github/workflows/e2e.yml`)
contains one remaining single-attempt apt step: `Install keyring runtime dependencies
(libdbus-1-dev, gnome-keyring, dbus-x11)` (step name at line 104 of pre-rebase e2e.yml,
now anchored by step-name). This step is subject to the same EC-010 mirror-flake class.
It MUST be converted to the combined two-attempt form:

```yaml
      - name: Install keyring runtime dependencies (libdbus-1-dev, gnome-keyring, dbus-x11)
        run: |
          # Apt-mirror resilience: combined two-attempt pattern per AC-006 (F-MAINT-P8-MED-004).
          # The outer condition wraps both update AND install so a mirror-failure during package
          # fetch ("E: Unable to fetch some archives") also triggers the fallback.
          if ! ( sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config gnome-keyring dbus-x11 ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y libdbus-1-dev pkg-config gnome-keyring dbus-x11
          fi
```

The `verify-workflow-structure` job gains a Red Gate assertion (RG-7) confirming the combined
two-attempt pattern is present in ≥1 step in e2e.yml (Red Gate test 7):

```bash
# RG-7: e2e.yml apt-mirror resilience — count-based, self-match-proof.
count=$(grep -cE '^\s+if ! \( sudo apt-get update && sudo apt-get install' .github/workflows/e2e.yml)
[ "$count" -ge 1 ] || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-006 e2e.yml: apt-mirror two-attempt wrapper missing from e2e.yml (found ${count}; need ≥1: Install keyring runtime dependencies)"
  exit 1
}
echo "S-MAINT-CI-DISK-EXHAUSTION-001 AC-006 e2e.yml check passed: apt-mirror resilience found ${count} times (≥1 required)."
# RG-7b: e2e.yml fallback includes apt-mirrors.txt overwrite (F-MAINT-P10-CRIT-001).
# Self-match-proof: assertion lines start with whitespace+count=$(grep..., not whitespace+sudo.
count=$(grep -cE '^\s+sudo tee /etc/apt/apt-mirrors\.txt' .github/workflows/e2e.yml)
[ "$count" -ge 1 ] || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-006 e2e.yml: fallback apt-mirrors.txt overwrite missing (found ${count}; need ≥1 — F-MAINT-P10-CRIT-001)"
  exit 1
}
echo "S-MAINT-CI-DISK-EXHAUSTION-001 AC-006 e2e.yml fallback-mirrors check passed: ${count} sites (≥1 required)."
```

The implementer MUST also update the final summary echo in `verify-workflow-structure` to add
`+ S-MAINT-CI-DISK-EXHAUSTION-001-e2e-AC-006 (count≥1)` to the assertion list and bump the total
from `18` to `19` (`16 reachability` → `17 reachability`).

**Combined update+install rationale (F-MAINT-P8-LOW-003):** The original pattern only guarded the `apt-get update` phase; a mirror-failure during the subsequent `apt-get install` package-fetch phase ("E: Unable to fetch some archives") was unguarded. The combined form `if ! ( sudo apt-get update && sudo apt-get install -y <pkgs> ); then` wraps both phases in a single condition: if either phase fails the apt-mirrors.txt overwrite fallback triggers and both phases are retried against the canonical archive (archive.ubuntu.com priority:1, F-MAINT-P10-CRIT-001). The fallback block retries BOTH `sudo apt-get update` AND `sudo apt-get install -y <pkgs>` (not just update); if the canonical archive also fails on the install, the step fails loud.

**apt-mirrors.txt overwrite rationale (F-MAINT-P10-CRIT-001):** On runner image 20260714.240.1, `/etc/apt/sources.list.d/ubuntu.sources` uses `URIs: mirror+file:/etc/apt/apt-mirrors.txt` — NOT any `http://` URL. `/etc/apt/sources.list` is comment-only; no http(s):// URLs exist in any apt source file. Sed-based URL rewriting was always a structural no-op: the regex `https?://[^/ ]*/ubuntu` can never match `mirror+file:`. Both v0.17 and v0.18 fallback blocks ran without changing anything, leaving `apt-get update` to retry the same flaky mirror state (runs 29531645116/29531648104 on HEAD 0939973f, F-MAINT-P10-CRIT-001; probe 29540085270 confirmed the file layout). All sed logic deleted.

The redesigned fallback has four ordered steps:

**Step 1 — Diagnostic dump on fallback entry (F-MAINT-P10-PG-009):** `echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true` and `echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true` — captures mirror config and source-file listing at fallback entry for future flake forensics without log scraping. `2>/dev/null || true` guards prevent abort on absent file.

**Step 2 — Remove third-party source files (4 defensive variants):** `sudo rm -f /etc/apt/sources.list.d/microsoft-prod.list /etc/apt/sources.list.d/azure-cli.sources /etc/apt/sources.list.d/microsoft-prod.sources /etc/apt/sources.list.d/azure-cli.list 2>/dev/null || true`. Probe confirmed: image 20260714.240.1 has `azure-cli.sources` (deb822, NOT `azure-cli.list` — F-MAINT-P10-HIGH-002 falsification) and `microsoft-prod.list`. Both `.sources` and `.list` alternates are removed defensively; `rm -f` is idempotent. None of our wrapped install payloads require these third-party repos.

**Step 3 — Overwrite apt-mirrors.txt:** `printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | sudo tee /etc/apt/apt-mirrors.txt`. Priority:1 adjudicated as `http://archive.ubuntu.com/ubuntu/` — probe-confirmed reachable (HTTP 200 at probe time), canonical Ubuntu archive, bypasses azure.archive.ubuntu.com (the flaky original priority:1). The `mirror+file:` method re-reads this file on next `apt-get update`.

**Step 4 — dpkg state repair + retry:** `sudo dpkg --configure -a 2>/dev/null || true` repairs broken dpkg state from the reclaimer purge. The retry `sudo apt-get update` and `sudo apt-get install -y <pkgs>` are NOT wrapped in `|| true` — canonical archive failure must fail loud.

The `verify-workflow-structure` job gains a Red Gate assertion confirming the combined two-attempt pattern is present in ≥12 steps in ci.yml (10 apt-install steps + 2 AC-007 toolchain installs share the same `if ! ( sudo apt-get update && sudo apt-get install` keyword; Red Gate test 5, F-MAINT-P8-LOW-003 grep updated). Derivation: 3 original sites + 7 new sites = 10 AC-006 apt-install sites; the 2 AC-007 C toolchain steps also use the AC-006 wrapper, contributing 2 more matches for a total of 12. The e2e.yml site is checked separately by RG-7 (not counted here):

```bash
# Anchored to the combined two-attempt pattern keyword — count-based, self-match-proof.
# Pattern updated F-MAINT-P8-LOW-003: now matches combined update+install form.
# The assertion line starts with whitespace+count=$(grep..., so the if-pattern cannot self-match.
count=$(grep -cE '^\s+if ! \( sudo apt-get update && sudo apt-get install' .github/workflows/ci.yml)
[ "$count" -ge 12 ] || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-006: apt-mirror two-attempt wrapper missing from expected sites (found ${count}; need 12: 10 apt-install steps + 2 AC-007 toolchain installs)"
  exit 1
}
echo "S-MAINT-CI-DISK-EXHAUSTION-001 AC-006 check passed: apt-mirror resilience found ${count} times (≥12 required: 10 apt-install steps + 2 AC-007 toolchain installs)."
# RG-5b: canonical fallback includes apt-mirrors.txt overwrite (F-MAINT-P10-CRIT-001).
# Each of the 12 apt-wrapper fallback blocks must contain the sudo tee apt-mirrors.txt line.
# Self-match-proof: assertion lines start with whitespace+count=$(grep..., not whitespace+sudo.
count=$(grep -cE '^\s+sudo tee /etc/apt/apt-mirrors\.txt' .github/workflows/ci.yml)
[ "$count" -ge 12 ] || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-006: fallback apt-mirrors.txt overwrite missing from ≥12 apt-wrapper sites (found ${count}; need 12 — F-MAINT-P10-CRIT-001)"
  exit 1
}
echo "S-MAINT-CI-DISK-EXHAUSTION-001 AC-006 fallback-mirrors check passed: ${count} sites (≥12 required)."
```

The implementer MUST also update the final summary echo in `verify-workflow-structure` to add `+ S-MAINT-CI-DISK-EXHAUSTION-001-AC-006 (count≥12)` to the assertion list and bump the total from `16` to `17` (`14 reachability` → `15 reachability`).

### AC-007 — C toolchain baseline install in both Linux workspace-build jobs (test + test-no-default-features)

Both Linux workspace-build CI jobs (`test` matrix legs and `test-no-default-features`) install the C toolchain baseline explicitly via a dedicated step BEFORE any `cargo build` phase. The step installs `build-essential`, `libc6-dev`, `clang`, and `libclang-dev` using the AC-006 apt-mirror two-attempt resilience wrapper (so a mirror failure on this install also triggers the apt-mirrors.txt overwrite fallback, F-MAINT-P10-CRIT-001), making it robust to the same apt-mirror flake class (EC-010).

**Root cause (DRIFT-CI-STDBOOL-001, revised v0.13):** AC-002 disk-space-reclaimer `large-packages: true` purge removes `libclang-common-16-dev`, `libclang-common-17-dev`, `libclang-common-18-dev`, and `libclang-rt-{16,17,18}-dev` from the runner. The `rocksdb-sys v0.17.3+10.4.2` build script invokes **bindgen → libclang** for binding generation; in that context `stdbool.h` is a **clang builtin resource header** shipped by `libclang-common-<N>-dev` — NOT by `libc6-dev`. Without it, bindgen fails with `fatal error: 'stdbool.h' file not found`. The v0.11 hypothesis that the runner image omitted `libc6-dev` is **falsified** by CI evidence (job 87471517229, run 29450000494, 2026-07-15): AC-007 successfully installed `build-essential libc6-dev` at 21:05:52 AFTER the reclaimer ran at 21:05:21 — and the build still failed at 21:10:26 with the same stdbool error. True root cause: **this story's own AC-002 reclaim step removes the clang resource headers that bindgen needs**. Failure is masked on cache-hit runs because Swatinem/rust-cache restores a prior librocksdb-sys artifact, bindgen never re-runs, and the missing headers are invisible — explaining push-event vs pull_request divergence. Fix: add `clang` and `libclang-dev` (version-tracking meta-packages; do NOT pin `-18`-suffixed names which break on image/llvm bumps) in addition to `build-essential` and `libc6-dev`. See EC-011 (falsification note) and EC-012 (revised root cause).

The install is idempotent (`apt-get install -y` is a no-op when packages are already present) and deterministic (pinned package names; no version constraint required as `build-essential`, `libc6-dev`, `clang`, and `libclang-dev` are system packages — `clang` and `libclang-dev` are meta-packages tracking the runner's active LLVM version, which is exactly the behavior needed to restore headers removed by the reclaimer regardless of the LLVM version in use).

**Snippet — Test-matrix Linux legs (`if: runner.os == 'Linux'` required; mixed-OS matrix includes macOS/Windows legs where apt-get is unavailable):**

```yaml
      # S-MAINT-CI-DISK-EXHAUSTION-001 AC-007: C toolchain baseline required by rocksdb-sys
      # v0.17.3+10.4.2 build script (bindgen → libclang needs clang builtin resource headers).
      # Root cause: AC-002 reclaimer large-packages purge removes libclang-common-*-dev.
      # DRIFT-CI-STDBOOL-001 (revised 2026-07-15: self-inflicted toolchain removal, EC-012).
      - name: Install C toolchain baseline (build-essential, libc6-dev, clang, libclang-dev)
        if: runner.os == 'Linux'
        run: |
          # C toolchain baseline required by rocksdb-sys v0.17.3+10.4.2 build script.
          # Root cause (DRIFT-CI-STDBOOL-001 revised): AC-002 reclaimer large-packages purge removes
          # libclang-common-*-dev; bindgen/libclang needs these clang builtin resource headers for
          # stdbool.h. clang + libclang-dev are meta-packages tracking the image default LLVM version
          # (do not pin -18 suffix). build-essential + libc6-dev included as C compiler baseline.
          # Apt-mirror resilience: combined two-attempt pattern per AC-006 (F-MAINT-P8-LOW-003).
          if ! ( sudo apt-get update && sudo apt-get install -y build-essential libc6-dev clang libclang-dev ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y build-essential libc6-dev clang libclang-dev
          fi
```

**Snippet — `test-no-default-features` job (ubuntu-only job; unconditional — no `if:`):**

```yaml
      # S-MAINT-CI-DISK-EXHAUSTION-001 AC-007: C toolchain baseline required by rocksdb-sys
      # v0.17.3+10.4.2 build script (bindgen → libclang needs clang builtin resource headers).
      # Root cause: AC-002 reclaimer large-packages purge removes libclang-common-*-dev.
      # DRIFT-CI-STDBOOL-001 (revised 2026-07-15: self-inflicted toolchain removal, EC-012).
      - name: Install C toolchain baseline (build-essential, libc6-dev, clang, libclang-dev)
        run: |
          # C toolchain baseline required by rocksdb-sys v0.17.3+10.4.2 build script.
          # Root cause (DRIFT-CI-STDBOOL-001 revised): AC-002 reclaimer large-packages purge removes
          # libclang-common-*-dev; bindgen/libclang needs these clang builtin resource headers for
          # stdbool.h. clang + libclang-dev are meta-packages tracking the image default LLVM version
          # (do not pin -18 suffix). build-essential + libc6-dev included as C compiler baseline.
          # Apt-mirror resilience: combined two-attempt pattern per AC-006 (F-MAINT-P8-LOW-003).
          if ! ( sudo apt-get update && sudo apt-get install -y build-essential libc6-dev clang libclang-dev ); then
            # ubuntu.sources uses mirror+file:/etc/apt/apt-mirrors.txt (image 20260714.240.1,
            # probe 29540085270). Sed URL-rewriting is a structural no-op — no http(s):// URLs
            # exist in any apt source file. Overwrite apt-mirrors.txt to pin canonical archive;
            # remove third-party source files; dpkg --configure -a repairs reclaimer-induced dpkg state.
            echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true
            echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true
            sudo rm -f \
              /etc/apt/sources.list.d/microsoft-prod.list \
              /etc/apt/sources.list.d/azure-cli.sources \
              /etc/apt/sources.list.d/microsoft-prod.sources \
              /etc/apt/sources.list.d/azure-cli.list \
              2>/dev/null || true
            printf 'http://archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' | \
              sudo tee /etc/apt/apt-mirrors.txt
            sudo dpkg --configure -a 2>/dev/null || true
            sudo apt-get update
            sudo apt-get install -y build-essential libc6-dev clang libclang-dev
          fi
```

The `verify-workflow-structure` job gains a Red Gate assertion confirming the C toolchain baseline install is present in both Linux workspace-build jobs (Red Gate test 6 — count ≥ 2):

```bash
# Anchored to the specific install payload — count-based, self-match-proof.
# The assertion line starts with whitespace+count=$(grep..., not whitespace+"sudo apt-get install",
# so the ^\s+sudo anchor cannot self-match.
count=$(grep -cE '^\s+sudo apt-get install -y build-essential libc6-dev clang libclang-dev\s*$' .github/workflows/ci.yml)
[ "$count" -ge 2 ] || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-007: C toolchain baseline install (build-essential libc6-dev clang libclang-dev) missing from ≥2 Linux jobs (found ${count}; need test (Test matrix) + test-no-default-features)"
  exit 1
}
echo "S-MAINT-CI-DISK-EXHAUSTION-001 AC-007 check passed: C toolchain baseline install found ${count} times (≥2 required)."
```

The implementer MUST also update the final summary echo in `verify-workflow-structure` to add `+ S-MAINT-CI-DISK-EXHAUSTION-001-AC-007 (count≥2)` to the assertion list and bump the total from `17` to `18` (`15 reachability` → `16 reachability`).

## §Architecture Mapping

This story modifies only CI workflow files; no production Rust crates are touched.

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| CI workflow | `.github/workflows/ci.yml` | N/A — YAML/shell; no Rust module boundary |
| CI workflow (e2e scope, F-MAINT-P8-MED-004) | `.github/workflows/e2e.yml` | N/A — YAML/shell; no Rust module boundary |

## §Implementation Notes

**Disk-space-reclaimer action choice and SHA pinning:** The actively-maintained fork
`insightsengineering/disk-space-reclaimer@dae9fabcb8febe09f6585471948acf9dc9a57489 # v1.1.2`
supersedes the unmaintained upstream `jlumbroso/free-disk-space` (last maintained
2023-10). Configure with inputs: `android: true`, `dotnet: true`, `haskell: true`,
`docker-images: true`, `large-packages: true`, `swap-storage: false` (swap preserved as
OOM headroom; see EC-008). Leave `tools-cache` (formerly `tool-cache`) unset — this story
does not remove the tool cache. Documented fallback:
`jlumbroso/free-disk-space@54081f138730dfa15788a46383842cd2f914a1be # v1.3.1` (record in
PR description for posterity). The action MUST be pinned to the specific SHA shown —
identical SHA-pinning requirement to all other actions in `ci.yml`.

**Reclaimer step is BEST-EFFORT (`continue-on-error: true`):** The `large-packages: true`
input invokes `apt-get` against the runner's rotating apt mirror. On the first live CI run
of this story (run 29437306537, 2026-07-15), mirror.enzu.com returned HTTP 404 Release
files for the apt repository — apt exited 100, the reclaimer step failed, and ALL THREE
hardened Linux jobs failed BEFORE the ≥25 GB gate ran. With the original
`continue-on-error: false` (default), a transient mirror flake becomes a hard job
failure, defeating the entire hardening purpose. Fix: add `continue-on-error: true` to
BOTH reclaimer steps (test matrix legs and test-no-default-features). The ≥25 GB gate is
the sole authoritative disk-readiness check — it verifies actual free-GB after reclaim
(whether reclaim succeeded or not) and fails loud if the threshold is not met. See
EC-009 for full trade-off documentation.

**DO NOT add `CARGO_PROFILE_DEV_DEBUG` to ci.yml:** `.cargo/config.toml` already sets
`debug = "line-tables-only"` for first-party code and `debug = false` for all dependencies
via `[profile.dev.package."*"]`. The env var would be a no-op (config file wins) and would
mislead future readers. This is a forbidden pattern enforced by AC-003 and §Forbidden Patterns
(F-CIDISK-P4-HIGH-001, adjudicated 2026-07-15).

**test-no-default-features job coverage:** Mirror all three protective steps (preflight
"Report initial disk space", disk-space-reclaimer with `swap-storage: false`, ≥25 GB gate)
plus the failure annotation step (AC-004) into the `test-no-default-features` job. This job
runs on the same ubuntu-latest disk envelope and is subject to the same class of failure
(F-CIDISK-P4-MED-002, adjudicated 2026-07-15). The ordering constraint (checkout → reclaim →
cache restore) applies identically.

**Pre-existing apt install steps — two-attempt pattern (AC-006):** The ten `sudo apt-get update && sudo apt-get install ...` steps across the CI pipeline (Install musl-tools in test matrix; Install libdbus-1-dev in test matrix; Install libdbus-1-dev in test-no-default-features; Install libdbus-1-dev in clippy; Install libdbus-1-dev in semver-checks; Install libdbus-1-dev in fuzz-smoke-vp021; Install libdbus-1-dev in perimeter-compile-fail; Install libdbus-1-dev in non-exhaustive-violation-compile-fail; Install libdbus-1-dev in no-hardcoded-sensors-compile-fail; Install shellcheck in shellcheck-demo-scripts) must each be expanded from the single-attempt inline form into the two-attempt pattern specified in AC-006. The redesigned fallback (F-MAINT-P10-CRIT-001) emits a diagnostic dump of `/etc/apt/apt-mirrors.txt` and `ls /etc/apt/sources.list.d/` on entry (F-MAINT-P10-PG-009), removes third-party source files (4 defensive variants), overwrites `/etc/apt/apt-mirrors.txt` to pin `http://archive.ubuntu.com/ubuntu/` at priority:1 (the ubuntu package source uses `mirror+file:` scheme — sed URL-rewriting was always a structural no-op; F-MAINT-P10-CRIT-001), calls `sudo dpkg --configure -a 2>/dev/null || true`, then retries. The `2>/dev/null || true` guards silence absent-file errors on the rm and diagnostic commands. The fallback `sudo apt-get update` is NOT `|| true` — canonical archive failures fail loud. No `continue-on-error: true` on these steps; they are not best-effort (unlike the disk-space-reclaimer). See AC-006 for byte-exact snippets and the Red Gate assertions (RG-5 count ≥ 12 outer wrapper; RG-5b count ≥ 12 apt-mirrors.txt tee).

**Ordering constraint:** The disk-space-reclaimer step MUST run after `actions/checkout`
(so git metadata is intact) and BEFORE `Swatinem/rust-cache` (so the cache restore does
not fill disk before reclaim). The preflight `df -h` step runs BEFORE checkout as a
pristine baseline. This ordering is confirmed correct-and-important (remove-uncertainty
pass 2026-07-15).

**Swatinem/rust-cache pin (optional verification note):** The current pin should be
verified against v2.9.1 (`c19371144df3bb44fab255c43d04cbc2ab54d1c4`) at PR-author time.
Rust-cache default pruning behavior suffices for disk management — no additional
cache-pruning configuration is needed in this story.

**verify-workflow-structure assertions:** Add the four new assertions (AC-001 count ≥ 2,
AC-002 count ≥ 2, AC-003 assertion-3, AC-003 assertion-4) to the existing `run:` block
of the `verify-workflow-structure` job. In the same commit, update the 7 pre-existing
reachability assertions to their anchored forms per the §Tasks sibling-sweep task (5
original from LOCAL pass-2 + AC-7 `semver-checks` + AC-8 `test-no-default-features`
job-name anchors; F-CIDISK-P4-HIGH-002 + LOW-001, adjudicated 2026-07-15). Error messages
are preserved verbatim. No other structural changes to the `verify-workflow-structure` job.

**Pass-6 traceability (LOW-001 + OBS-002, updated v0.19):** The ci.yml summary echo must use the code-authoritative assertion count: 12 pre-existing reachability (9 original + 3 added by develop between story writing and rebase to 4f9a5c6f: wasm32-threatintel-staleness-check + F-MCPRS-PRL14-LOW-001 ×2) + 4 new count-based reachability (AC-001, AC-002, AC-006, AC-007) + 1 new e2e.yml reachability (RG-7, F-MAINT-P8-MED-004) + 2 new fallback-form reachability (RG-5b: apt-mirrors.txt tee count ≥ 12 in ci.yml; RG-7b: apt-mirrors.txt tee count ≥ 1 in e2e.yml; F-MAINT-P10-CRIT-001) + 2 new config-invariant (AC-003 assertions 3+4) = 21 total (19 reachability + 2 config-invariant, matching the ci.yml summary echo). All story §Tasks echo-bump instructions use the post-rebase base of 12 pre-existing + 2 post-AC-001/002 = 14 reachability as the starting point before AC-006. S-MAINT prefix required on AC-006/AC-007/e2e-AC-006 echo items to disambiguate from pre-existing PLUGIN-MIGRATION-001-F "AC-006" item.

## §Token Budget Estimate

| Item | Tokens (approx.) |
|------|-----------------|
| This story spec | ~3 k |
| `.github/workflows/ci.yml` (read + modify Test job + verify-workflow-structure job) | ~6 k |
| `insightsengineering/disk-space-reclaimer` README / SHA (pre-validated, no lookup needed) | ~1 k |
| Implementation scratch + comments | ~1 k |
| **Total** | **~11 k** |

Well within a single agent context window; no splitting required.

## §Tasks

- [ ] Create maintenance branch `maintenance/ci-disk-hardening` from `develop`
- [ ] Read `.github/workflows/ci.yml` in full (especially the Test job, `test-no-default-features` job, and `verify-workflow-structure` job)
- [ ] Use the pre-validated SHA `dae9fabcb8febe09f6585471948acf9dc9a57489` for `insightsengineering/disk-space-reclaimer # v1.1.2` (no lookup needed; validated 2026-07-15 remove-uncertainty pass)
- [ ] Linux Test job legs: add "Report initial disk space" step (`run: df -h`) as the FIRST step (before `actions/checkout`; Linux legs only) (AC-001)
- [ ] `test-no-default-features` job: add "Report initial disk space" step (`run: df -h`) as the FIRST step (before `actions/checkout`) (AC-001 + F-CIDISK-P4-MED-002)
- [ ] Linux Test job legs: add `insightsengineering/disk-space-reclaimer@dae9fabcb8febe09f6585471948acf9dc9a57489 # v1.1.2` step after checkout and before rust-cache; configure `android: true, dotnet: true, haskell: true, docker-images: true, large-packages: true, swap-storage: false`; include `continue-on-error: true` (reclaim is best-effort — apt-mirror flake class motivated this; the ≥25 GB gate is the sole authoritative check; see EC-009) (AC-002; swap=false per EC-008)
- [ ] `test-no-default-features` job: add the same reclaimer step with identical inputs (`swap-storage: false`) and `continue-on-error: true` after checkout and before rust-cache (AC-002 + F-CIDISK-P4-MED-002 + EC-009)
- [ ] Linux Test job legs: add "Verify ≥25 GB free" step immediately after the reclaimer step (see AC-002 snippet; uses `df -P /` + 1K-block arithmetic + `AVAIL_GB=${AVAIL_GB:-0}` guard — no gsub)
- [ ] `test-no-default-features` job: add identical "Verify ≥25 GB free" step immediately after its reclaimer step (F-CIDISK-P4-MED-002)
- [ ] DO NOT add `CARGO_PROFILE_DEV_DEBUG` to the Test job `env:` block — it is a no-op; `.cargo/config.toml` already sets identical values at higher precedence (AC-003; F-CIDISK-P4-HIGH-001 adjudication)
- [ ] Linux Test job legs: add `if: failure()` disk-annotation step at the END (after JUnit upload; see AC-004 snippet — includes `USED_PCT=${USED_PCT:-0}` guard)
- [ ] `test-no-default-features` job: add identical `if: failure()` disk-annotation step at the END (AC-004 + F-CIDISK-P4-MED-002)
- [ ] `verify-workflow-structure` job: add four new assertions to the existing `run:` block:
  - AC-001 count assertion: `count=$(grep -cE '^\s+- name: Report initial disk space\s*$' .github/workflows/ci.yml)` + `[ "$count" -ge 2 ]` (counts test matrix + test-no-default-features)
  - AC-002 count assertion: `count=$(grep -cE '^\s+uses: insightsengineering/disk-space-reclaimer' .github/workflows/ci.yml)` + `[ "$count" -ge 2 ]` (counts test matrix + test-no-default-features)
  - AC-003 assertion-3: `awk '/^\[profile\.dev\]$/{s=1;next} /^\[/{s=0} s && /^debug = "line-tables-only"$/{found=1} END{exit !found}' .cargo/config.toml` (section-scoped: verifies debug = "line-tables-only" is within [profile.dev], not merely present anywhere in the file; self-match impossible — different file)
  - AC-003 assertion-4: `awk '/^\[profile\.dev\.package\."\*"\]$/{s=1;next} /^\[/{s=0} s && /^debug = false$/{found=1} END{exit !found}' .cargo/config.toml` (section-scoped: verifies debug = false payload is within [profile.dev.package."*"], not just that the section header exists; self-match impossible — different file)
- [ ] Apply self-match-proof anchoring to the 7 pre-existing verify-workflow-structure reachability assertions IN THE SAME COMMIT (5 from LOCAL pass-2 + AC-7 semver-checks + AC-8 test-no-default-features; F-CIDISK-P4-HIGH-002 + LOW-001, adjudicated 2026-07-15). Exact replacements:
  - `non-exhaustive-violation-compile-fail`: `grep -qE 'non-exhaustive-violation-compile-fail'` → `grep -qE '^  non-exhaustive-violation-compile-fail:'` (job-name anchor; 2-space GitHub Actions job indent)
  - `wasm32-compile-check`: `grep -qE 'wasm32-compile-check'` → `grep -qE '^  wasm32-compile-check:'` (job-name anchor; 2-space indent)
  - `build-plugin-crowdstrike-oauth2`: `grep -qE 'build-plugin-crowdstrike-oauth2'` → `grep -qE '^\s+just build-plugin-crowdstrike-oauth2\s*$'` (just-recipe anchor; matches `          just build-plugin-crowdstrike-oauth2` at 10-space indent; `$` excludes comment lines)
  - `no-hardcoded-sensors-compile-fail`: `grep -qE 'no-hardcoded-sensors-compile-fail'` → `grep -qE '^  no-hardcoded-sensors-compile-fail:'` (job-name anchor; 2-space indent)
  - `shellcheck-demo-scripts`: `grep -qE 'shellcheck-demo-scripts'` → `grep -qE '^  shellcheck-demo-scripts:'` (job-name anchor; 2-space indent)
  - `semver-checks` (AC-7): `grep -qE 'semver-checks'` → `grep -qE '^  semver-checks:'` (job-name anchor; 2-space indent; F-CIDISK-P4-HIGH-002)
  - `test-no-default-features` (AC-8): `grep -qE 'test-no-default-features'` → `grep -qE '^  test-no-default-features:'` (job-name anchor; 2-space indent; F-CIDISK-P4-LOW-001)
  - Self-match proof for all seven: assertion lines start with whitespace+`grep`, so job-name anchors `^  <job-name>:` and just-recipe anchor `^\s+just ...\s*$` cannot match the assertion lines themselves
- [ ] Test matrix job — `Install musl-tools` step: replace `run: sudo apt-get update && sudo apt-get install -y musl-tools` with the two-attempt mirror-resilience form from AC-006 (byte-exact snippet; preserves `if: matrix.install_musl` conditional) (AC-006)
- [ ] Test matrix job — `Install libdbus-1-dev (required by keyring build on Linux, all-features)` step: replace `run: sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config` with the two-attempt form (byte-exact snippet; preserves `if: runner.os == 'Linux'` conditional) (AC-006)
- [ ] `test-no-default-features` job — `Install libdbus-1-dev (required by keyring Secret Service backend on Linux)` step: replace `run: sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config` with the two-attempt form (byte-exact snippet; unconditional step) (AC-006)
- [ ] 7-site full sweep (AC-006, F-CIDISK-PR1-OBS-001): convert the remaining seven single-attempt apt install steps to the AC-006 two-attempt wrapper form (wrapper identical; only the final `apt-get install -y <pkgs>` line varies per site; see AC-006 snippets): `clippy` job → `libdbus-1-dev pkg-config`; `semver-checks` job → `libdbus-1-dev pkg-config`; `fuzz-smoke-vp021` job → `libdbus-1-dev pkg-config`; `perimeter-compile-fail` job → `libdbus-1-dev pkg-config`; `non-exhaustive-violation-compile-fail` job → `libdbus-1-dev pkg-config`; `no-hardcoded-sensors-compile-fail` job → `libdbus-1-dev pkg-config`; `shellcheck-demo-scripts` job → `shellcheck`. Rationale: EC-010 mirror-flake class affects every apt job; `clippy` is a `needs:` predecessor whose failure blocks the whole pipeline including AC-005 evidence; full sweep required per Canonical Principle Rule 4 (no partial deferral)
- [ ] e2e.yml `Install keyring runtime dependencies (libdbus-1-dev, gnome-keyring, dbus-x11)` step: replace single-attempt `run: sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config gnome-keyring dbus-x11` with the combined two-attempt form (byte-exact snippet from AC-006 e2e.yml scope extension paragraph) (F-MAINT-P8-MED-004; AC-006 scope extension to e2e.yml)
- [ ] `verify-workflow-structure` job: add AC-006 Red Gate assertion (`count=$(grep -cE '^\s+if ! \( sudo apt-get update && sudo apt-get install' .github/workflows/ci.yml)` + `[ "$count" -ge 12 ]`) before the final summary echo; update summary echo: add `+ S-MAINT-CI-DISK-EXHAUSTION-001-AC-006 (count≥12)` to assertion list; change `14 reachability assertions` to `15 reachability assertions`; change `= 16 total checks` to `= 17 total checks` (AC-006 Red Gate test 5; grep pattern updated F-MAINT-P8-LOW-003)
- [ ] Linux Test job legs: add `Install C toolchain baseline (build-essential, libc6-dev, clang, libclang-dev)` step using the AC-006 two-attempt wrapper (byte-exact snippet from AC-007) BEFORE any `cargo build` phase (i.e., after `actions/checkout` and disk-reclaim steps, before the build step); `if: runner.os == 'Linux'` guard required (mixed-OS Test-matrix includes macOS/Windows legs where apt-get is unavailable; the step must run unconditionally on every Linux leg) (AC-007)
- [ ] `test-no-default-features` job: add identical `Install C toolchain baseline (build-essential, libc6-dev, clang, libclang-dev)` step using the AC-006 two-attempt wrapper BEFORE the build phase; unconditional (AC-007)
- [ ] `verify-workflow-structure` job: add AC-007 Red Gate assertion (`count=$(grep -cE '^\s+sudo apt-get install -y build-essential libc6-dev clang libclang-dev\s*$' .github/workflows/ci.yml)` + `[ "$count" -ge 2 ]`) before the final summary echo; update summary echo: add `+ S-MAINT-CI-DISK-EXHAUSTION-001-AC-007 (count≥2)` to assertion list; change `15 reachability assertions` to `16 reachability assertions`; change `= 17 total checks` to `= 18 total checks` (AC-007 Red Gate test 6)
- [ ] `verify-workflow-structure` job: add RG-7 assertion (`count=$(grep -cE '^\s+if ! \( sudo apt-get update && sudo apt-get install' .github/workflows/e2e.yml)` + `[ "$count" -ge 1 ]`) before the final summary echo; update summary echo: add `+ S-MAINT-CI-DISK-EXHAUSTION-001-e2e-AC-006 (count≥1)` to assertion list; change `16 reachability assertions` to `17 reachability assertions`; change `= 18 total checks` to `= 19 total checks` (RG-7, F-MAINT-P8-MED-004)
- [ ] `verify-workflow-structure` job: add RG-5b assertion (`count=$(grep -cE '^\s+sudo tee /etc/apt/apt-mirrors\.txt' .github/workflows/ci.yml)` + `[ "$count" -ge 12 ]`) before the final summary echo; update summary echo: add `+ S-MAINT-CI-DISK-EXHAUSTION-001-AC-006-mirrors (count≥12)` to assertion list; change `17 reachability assertions` to `18 reachability assertions`; change `= 19 total checks` to `= 20 total checks` (RG-5b fallback apt-mirrors.txt overwrite lock; F-MAINT-P10-CRIT-001)
- [ ] `verify-workflow-structure` job: add RG-7b assertion (`count=$(grep -cE '^\s+sudo tee /etc/apt/apt-mirrors\.txt' .github/workflows/e2e.yml)` + `[ "$count" -ge 1 ]`) before the final summary echo; update summary echo: add `+ S-MAINT-CI-DISK-EXHAUSTION-001-e2e-AC-006-mirrors (count≥1)` to assertion list; change `18 reachability assertions` to `19 reachability assertions`; change `= 20 total checks` to `= 21 total checks` (RG-7b fallback apt-mirrors.txt overwrite lock; F-MAINT-P10-CRIT-001)
- [ ] Verify each fallback block includes diagnostic dump commands on fallback entry (F-MAINT-P10-PG-009): `echo "=== [fallback] apt-mirrors.txt on entry ===" && cat /etc/apt/apt-mirrors.txt 2>/dev/null || true` and `echo "=== [fallback] sources.list.d on entry ===" && ls /etc/apt/sources.list.d/ 2>/dev/null || true` — these must appear in all 13 fallback blocks (12 ci.yml + 1 e2e.yml); they self-evidence future flake runs without requiring log scraping
- [ ] Record three consecutive green CI run IDs in the PR description (AC-005 evidence; NOTE: runs 29524703679 attempt-1/bd65e93a, 29531645116/0939973f, and 29531648104/0939973f are all DISQUALIFIED — see AC-005 v0.19 NOTE; three-green record restarts on post-fix HEAD; re-run attempts of a failed run ID do NOT qualify)

## §Previous Story Intelligence

**W3-FIX-CI-001** (merged PR #112 a3bd5a0f): Introduced the mold linker on Linux legs.
Mold writes its output file in a single large write rather than incrementally, making
it more sensitive to near-full disk conditions than the legacy BFD linker. The error
`mold: Disk full` / `mold: failed to write to an output file. Disk full?` is the direct
symptom. This story does NOT remove mold — the performance benefit (2–5 min savings)
is retained; this story addresses the underlying disk capacity gap.

**S-MAINT-REQWEST-RUSTLS-GATE-001** (draft v0.1, D-1497): The canonical recent
maintenance story using the same frontmatter schema and section structure. No
functional overlap.

**Key lesson from W3-FIX-CI-001:** Every new GitHub Actions step MUST be pinned to a
specific commit SHA with a `# vN.N.N` comment — the project has zero mutable-tag
action references in `ci.yml`. Fetch the current SHA at PR-author time; do not use
`@main`, `@v1`, or any mutable reference.

**Key lesson from S-MAINT-REQWEST-RUSTLS-GATE-001:** Red Gate tests for CI toolchain
stories belong in the `verify-workflow-structure` job (structural assertions), not in
a separate test script. This matches the existing `non-exhaustive-violation-compile-fail`
and `wasm32-compile-check` assertion pattern established during S-PLUGIN-PREREQ-C.

## §Architecture Compliance Rules

- No production Rust crate source files (`crates/**/src/**`) may be modified
- No changes to `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `.config/nextest.toml`
- The `fmt`, `deny`, and `audit` jobs in `ci.yml` must NOT be modified under any circumstances;
  no exceptions
- The `clippy`, `semver-checks`, `non-exhaustive-violation-compile-fail`, `perimeter-compile-fail`,
  `no-hardcoded-sensors-compile-fail`, `fuzz-smoke-vp021`, and `shellcheck-demo-scripts` jobs MAY
  be modified ONLY to add the AC-006 apt-mirror two-attempt resilience wrapper to their respective
  apt install step(s) — no other changes to these jobs are permitted (carve-out ratified AC-006,
  F-MAINT-P8-MED-001)
- The `test-no-default-features` job MAY be modified only to add: the four v0.6-ratified protective
  steps (preflight, disk-space-reclaimer + ≥25 GB gate, failure annotation), the AC-006 apt-mirror
  two-attempt wrapper on its libdbus install step, and the AC-007 C toolchain baseline install step
  — the job's existing `PROPTEST_CASES`, `RUSTFLAGS`, test-invocation lines, and cache configuration
  must NOT be changed (carve-out ratified v0.6 + F-MAINT-P8-MED-001)
- The `verify-workflow-structure` job's existing assertions (AC-5 `TARGET_COUNT >= 5`,
  AC-6 cargo-deny/audit, AC-7 semver, AC-8 no-default-features, non-exhaustive, wasm32
  checks) must ALL pass after this story's modifications; the nine new assertions (AC-001
  count ≥ 2, AC-002 count ≥ 2, AC-006 count ≥ 12, RG-5b apt-mirrors.txt overwrite count ≥ 12, AC-007 count ≥ 2,
  RG-7 e2e.yml count ≥ 1, RG-7b e2e.yml apt-mirrors.txt overwrite count ≥ 1, AC-003 assertions 3 and 4) are
  additive, and the 7 pre-existing reachability assertions (5 original + AC-7 semver-checks
  + AC-8 test-no-default-features) are updated in-place to self-match-proof anchored forms
  (see §Tasks sibling-sweep task); no other structural changes.
  The final summary echo must reflect 21 total checks (19 reachability + 2 config-invariant)
- All new GitHub Actions steps must be pinned to a specific commit SHA with a `# vN.N.N`
  comment per the project SHA-pinning policy (every existing action in ci.yml is
  commit-pinned; no exceptions)
- `CARGO_PROFILE_DEV_DEBUG` env var MUST NOT be added to `ci.yml` in any form — it is a
  no-op (`.cargo/config.toml` already sets `debug = "line-tables-only"` + `debug = false`
  for deps at higher precedence) and misleads future readers; see §Forbidden Patterns
  (F-CIDISK-P4-HIGH-001, adjudicated 2026-07-15)
- The Test job's existing `env:` block entries (`PROPTEST_CASES`, `RUSTFLAGS`) must NOT
  be replaced or renamed; no new entries are added under this story
- The disk-space-reclaimer step must appear AFTER `actions/checkout` and BEFORE
  `Swatinem/rust-cache` to ensure the reclaim occurs before cache restore fills disk
- This story's changes ride `maintenance/ci-disk-hardening`; they MUST NOT be merged
  through an open defect PR branch

## §Library & Framework Requirements

No new Rust dependencies. CI tooling only:

| Tool | Version constraint | Justification |
|------|--------------------|---------------|
| `insightsengineering/disk-space-reclaimer` | `dae9fabcb8febe09f6585471948acf9dc9a57489 # v1.1.2` (validated 2026-07-15) | Actively-maintained drop-in fork of unmaintained jlumbroso/free-disk-space (last 2023-10). Removes android/dotnet/haskell/docker-images/large-packages; swap preserved as OOM headroom (`swap-storage: false`; see EC-008). Fallback: `jlumbroso/free-disk-space@54081f138730dfa15788a46383842cd2f914a1be # v1.3.1` |
| `df` / `awk` | system utilities present on all ubuntu-latest runners | Used in preflight, ≥25 GB gate, and failure annotation steps |

New `apt-get` packages installed by AC-007 C toolchain baseline step (CI runner only; not Rust crate dependencies): `build-essential`, `libc6-dev`, `clang`, `libclang-dev`. These restore the clang resource headers (`libclang-common-*-dev`) removed by AC-002 disk-space-reclaimer `large-packages: true` purge, enabling bindgen-based build scripts (librocksdb-sys). No Python packages. No compiled Cargo dev-dependencies. No new GitHub Actions secrets required.

## §File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/ci.yml` | MODIFY | Linux Test job legs: add preflight (AC-001), disk-space-reclaimer with `swap-storage: false, continue-on-error: true` (AC-002; best-effort per EC-009; gate is the authoritative check), ≥25 GB gate (AC-002; `df -P /` 1K-block form + `AVAIL_GB=${AVAIL_GB:-0}` guard), failure annotation (AC-004; `USED_PCT=${USED_PCT:-0}` guard); `test-no-default-features` job: mirror same three protective steps + failure annotation (F-CIDISK-P4-MED-002); DO NOT add `CARGO_PROFILE_DEV_DEBUG` (AC-003 — it is a no-op; forbidden); convert ten `sudo apt-get update && sudo apt-get install ...` steps to two-attempt mirror-resilience form: 3 original steps (Install musl-tools in test matrix; Install libdbus-1-dev in test matrix; Install libdbus-1-dev in test-no-default-features) + 7 new steps (clippy → libdbus-1-dev pkg-config; semver-checks → libdbus-1-dev pkg-config; fuzz-smoke-vp021 → libdbus-1-dev pkg-config; perimeter-compile-fail → libdbus-1-dev pkg-config; non-exhaustive-violation-compile-fail → libdbus-1-dev pkg-config; no-hardcoded-sensors-compile-fail → libdbus-1-dev pkg-config; shellcheck-demo-scripts → shellcheck) (AC-006; F-CIDISK-PR1-OBS-001; byte-exact snippets in AC-006 section); add `Install C toolchain baseline (build-essential, libc6-dev, clang, libclang-dev)` step with outer step-level preamble (4-line comment block above `- name:`) via AC-006 two-attempt wrapper BEFORE any cargo build phase in both Linux workspace-build jobs (AC-007; DRIFT-CI-STDBOOL-001 revised — self-inflicted toolchain removal by reclaimer, not runner-image omission; F-CIDISK-PR1-MED-001); `verify-workflow-structure` job: AC-001 count assertion (≥2; `^\s+- name: Report initial disk space\s*$`) + AC-002 count assertion (≥2; `^\s+uses: insightsengineering/disk-space-reclaimer`) + AC-006 count assertion (≥12; `^\s+if ! \( sudo apt-get update && sudo apt-get install`; 10 apt-install steps + 2 AC-007 toolchain installs; F-CIDISK-PR1-MED-002, grep updated F-MAINT-P8-LOW-003) + AC-007 count assertion (≥2; `^\s+sudo apt-get install -y build-essential libc6-dev clang libclang-dev\s*$`) + RG-7 e2e.yml count assertion (≥1; `^\s+if ! \( sudo apt-get update && sudo apt-get install` against e2e.yml; F-MAINT-P8-MED-004) + RG-5b fallback-mirrors count assertion (≥12; `^\s+sudo tee /etc/apt/apt-mirrors\.txt` in ci.yml; F-MAINT-P10-CRIT-001) + RG-7b fallback-mirrors count assertion (≥1; `^\s+sudo tee /etc/apt/apt-mirrors\.txt` against e2e.yml; F-MAINT-P10-CRIT-001) + two AC-003 `.cargo/config.toml` invariant checks + anchor 7 pre-existing reachability assertions (5 original + AC-7 `semver-checks` + AC-8 `test-no-default-features`; see §Tasks sibling-sweep) + update summary echo to 21 total / 19 reachability; no other jobs touched |
| `.github/workflows/e2e.yml` | MODIFY | `Install keyring runtime dependencies (libdbus-1-dev, gnome-keyring, dbus-x11)` step: convert from single-attempt inline form to AC-006 combined two-attempt wrapper form (byte-exact snippet from AC-006 e2e.yml scope extension paragraph; redesigned fallback with apt-mirrors.txt overwrite + dpkg repair; F-MAINT-P8-MED-004 / F-MAINT-P10-CRIT-001/HIGH-002) |

No new files required. The `insightsengineering/disk-space-reclaimer` action is invoked
as an inline `uses:` step; it does not require a new config file in the repository.

## §Forbidden Patterns

| Pattern | Reason |
|---------|--------|
| `CARGO_PROFILE_DEV_DEBUG` in `ci.yml` (any value, including `"line-tables-only"`, `0`, `1`) | No-op: `.cargo/config.toml` already sets `debug = "line-tables-only"` for first-party code and `debug = false` for deps at higher precedence; adding the env var changes nothing and misleads future readers (F-CIDISK-P4-HIGH-001, adjudicated 2026-07-15). AC-003 guards the config-file invariant instead. |
| `swap-storage: true` in reclaimer inputs | Deliberate: swap (~4 GB) preserved as OOM headroom for the linux-gnu doctest leg; see EC-008. Do not change without verifying the doctest OOM risk is resolved. |
| `CARGO_PROFILE_RELEASE_*` modification | Release profile is not in scope; only dev/test builds are affected |
| `docker system prune -af` in CI steps | Removes Docker layers needed by other actions; unsafe without explicit scope guard |
| `sudo rm -rf /usr/share/dotnet` (direct removal) | Fragile manual removal; use `insightsengineering/disk-space-reclaimer` which handles ordering and dependencies correctly |
| Floating action reference (`@main`, `@v1`, `@latest`) | SHA-pinning policy; all ci.yml actions are commit-pinned |
| Mixing this change into an open defect PR branch | Branch isolation required per AC-005; complicates bisection |
| Modifying `fmt`, `deny`, or `audit` jobs in any way | Strictly out of scope; no exceptions |
| Modifying `clippy`, `semver-checks`, `non-exhaustive-violation-compile-fail`, `perimeter-compile-fail`, `no-hardcoded-sensors-compile-fail`, `fuzz-smoke-vp021`, or `shellcheck-demo-scripts` jobs beyond adding the AC-006 apt-mirror two-attempt wrapper | Only the AC-006 apt wrapper is permitted in these jobs; any other change is out of scope (F-MAINT-P8-MED-001 carve-out) |
| Modifying `test-no-default-features` job beyond the four v0.6 protective steps + AC-006 wrapper + AC-007 C toolchain install | The three explicitly-ratified modification types are the only permitted changes; existing `PROPTEST_CASES`, `RUSTFLAGS`, test-invocation lines, and cache configuration must not be changed (F-MAINT-P8-MED-001 carve-out) |
| Removing or renaming existing `PROPTEST_CASES` or `RUSTFLAGS` env entries | Existing entries must be preserved; no new env entries are added under this story |

## §Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `insightsengineering/disk-space-reclaimer` action fails or hangs (non-apt-mirror cause, e.g., GitHub Actions runner infra fault) | `continue-on-error: true` — the job proceeds to the ≥25 GB gate regardless of reclaimer exit code. The gate is the sole authoritative arbiter: if post-failure free disk is ≥25 GB the build continues; if below 25 GB the gate exits 1 with the actual free-GB count. Design changed v0.8→v0.9 from `continue-on-error: false`; the apt-mirror flake class (EC-009) motivated the change. |
| EC-002 | Post-reclaim disk still below 25 GB on an unusual runner topology | AC-002 gate exits 1 with a human-readable `::error::` message identifying available space; job fails early |
| EC-003 | _(retired v0.6)_ `CARGO_PROFILE_DEV_DEBUG` env var — no-op per F-CIDISK-P4-HIGH-001; env var removed from scope | _(retired)_ AC-003 now guards the pre-existing `.cargo/config.toml` invariant instead |
| EC-004 | _(retired v0.6)_ Future story adding `CARGO_PROFILE_DEV_DEBUG` — now forbidden pattern | _(retired)_ `CARGO_PROFILE_DEV_DEBUG` in `ci.yml` is forbidden; see §Forbidden Patterns |
| EC-005 | `verify-workflow-structure` AC-5 `target:` grep matches a new step name | The existing grep counts `^            target:` at 12-space indent (matrix field name). New step `name:` fields use `      - name:` (6-space indent). No conflict. |
| EC-006 | _(retired v0.6)_ macOS/Windows failing due to `CARGO_PROFILE_DEV_DEBUG` — no longer applicable | _(retired)_ env var not added to ci.yml per F-CIDISK-P4-HIGH-001 adjudication |
| EC-007 | Preflight `df -h` step placed after `actions/checkout` instead of before | The preflight MUST be before checkout to capture the true baseline; checkout restores git metadata and may trigger cache actions. Verify ordering in the delivered YAML. |
| EC-008 | `swap-storage: false` — OOM headroom preservation trade-off | Swap (~4 GB) is deliberately preserved. The linux-gnu leg runs a 1000-PROPTEST_CASES nextest run followed by doctests; a pre-existing OOM-kill risk exists in that leg. The remaining reclaim inputs (android/dotnet/haskell/docker-images/large-packages) still deliver 21–31 GB, satisfying the ≥25 GB gate. Future maintainers: do NOT re-enable `swap-storage: true` without first verifying the doctest OOM risk is resolved. (F-CIDISK-P4-MED-001, adjudicated 2026-07-15.) |
| EC-009 | apt-mirror flake / partial-reclaim: `large-packages: true` triggers `apt-get` against the runner's rotating apt mirror; the mirror can return HTTP 404 Release files on rotation (evidence: CI run 29437306537, 2026-07-15, mirror.enzu.com, apt exit code 100, reclaimer step failed — ALL THREE hardened Linux jobs failed BEFORE the ≥25 GB gate) | `continue-on-error: true` on the reclaimer step allows the job to proceed to the ≥25 GB gate regardless of reclaimer exit code. If disk is ≥25 GB despite partial/zero reclaim, the build continues. If below 25 GB, the gate exits 1 loud with the actual free-GB count. Trade-off: `continue-on-error` can mask persistent action breakage (e.g., an action update that breaks the inputs API); mitigated because the ≥25 GB gate provides ground-truth disk verification on every run and fails loud when reclaim genuinely under-delivers. |
| EC-010 | Persistent apt-mirror outage affecting pre-existing apt install steps: the runner image selects its apt mirror via `/etc/apt/apt-mirrors.txt` (image 20260714.240.1); ubuntu.sources uses `URIs: mirror+file:/etc/apt/apt-mirrors.txt` — NOT any http:// URL. A flaky priority:1 mirror (`azure.archive.ubuntu.com` at image-build time) causes `apt-get update` to fail with exit code 100. `/etc/apt/sources.list` is effectively empty (comment-only). Evidence: run 29438854846 rerun (2026-07-15) — enzu.com 404; runs 29531645116/29531648104 (0939973f, 2026-07-16) — v0.18 fallback was a structural no-op (sed cannot match `mirror+file:` scheme; probe 29540085270, image 20260714.240.1, confirmed the source-file layout). Runner-image mirror selection is entirely outside our control. | Redesigned two-attempt pattern per AC-006 (F-MAINT-P10-CRIT-001): on first-attempt failure, (1) emit diagnostic dump of `/etc/apt/apt-mirrors.txt` and `ls /etc/apt/sources.list.d/` for forensics (F-MAINT-P10-PG-009); (2) remove third-party source files (4 defensive variants: microsoft-prod.list, azure-cli.sources, microsoft-prod.sources, azure-cli.list); (3) overwrite `/etc/apt/apt-mirrors.txt` with canonical archive at priority:1 (`http://archive.ubuntu.com/ubuntu/`) — the `mirror+file:` method re-reads this file on next `apt-get update`; (4) `sudo dpkg --configure -a 2>/dev/null || true`; (5) retry `sudo apt-get update && sudo apt-get install -y <pkgs>`. `archive.ubuntu.com` is authoritative (probe-confirmed reachable HTTP 200). No `continue-on-error` on these steps — if the canonical archive also fails, the job fails loud. |
| EC-011 | Runner-image toolchain regression — `ubuntu-latest` image `ubuntu24/20260705.232` ships without `libc6-dev` and `build-essential`, causing C-compilation failures in native build scripts (rocksdb-sys v0.17.3+10.4.2). DRIFT-CI-STDBOOL-001 (2026-07-15): observed in PR #224 pull_request CI runs; both Linux legs (test matrix + test-no-default-features) failed with `fatal error: 'stdbool.h' file not found`. The push-event CI succeeded because that run landed on a different runner image revision that retained libc6-dev. Runner-image regressions of this class are unpredictable — GitHub does not allow pinning the runner image version on `ubuntu-latest`-labeled hosted runners; any `ubuntu-latest` relabelling can silently drop previously-installed packages. **[v0.13 FALSIFIED]** This hypothesis is refuted by CI evidence from job 87471517229 (run 29450000494, 2026-07-15): FA596F92 branch (which includes the v0.11 AC-007 `build-essential libc6-dev` install) still failed all 6 Linux test legs. The causal chain shows the reclaimer removing `libclang-common-*-dev` at 21:05:21, AC-007 successfully installing `build-essential libc6-dev` at 21:05:52, and the build STILL failing at 21:10:26 with `stdbool.h`. The root cause is self-inflicted clang-toolchain removal by this story's own AC-002 reclaim step, not a runner-image omission. See EC-012 for the corrected root cause. | **[v0.13 SUPERSEDED]** Original mitigation (install `build-essential libc6-dev`) was insufficient — those packages do not provide the clang builtin resource headers removed by the reclaimer. Corrected mitigation in EC-012: install `clang libclang-dev` meta-packages in addition. |
| EC-012 | Self-inflicted toolchain removal — AC-002 disk-space-reclaimer `large-packages: true` purge removes `libclang-common-16-dev`, `libclang-common-17-dev`, `libclang-common-18-dev`, and `libclang-rt-{16,17,18}-dev` from the runner image. `librocksdb-sys v0.17.3+10.4.2` generates bindings via bindgen → libclang; `stdbool.h` is a **clang builtin resource header** shipped by `libclang-common-<N>-dev` — NOT by `libc6-dev`. Failure is masked on cache-hit runs (Swatinem/rust-cache restores a prior librocksdb-sys artifact, bindgen never re-runs, and the missing headers are invisible), explaining push-event vs pull_request divergence. Evidence: CI job 87471517229 (run 29450000494, image ubuntu-24.04 Version 20260705.232.1, 2026-07-15): reclaimer removed `libclang-common-*-dev` at 21:05:21; AC-007 installed `build-essential libc6-dev` at 21:05:52 (successfully, proving AC-006 mirror fallback works); `librocksdb-sys` bindgen failed at 21:10:26 with `stdbool.h`. DRIFT-CI-STDBOOL-001 root-cause revision (2026-07-15). | AC-007 mitigates (corrected in v0.13): install `clang` and `libclang-dev` (version-tracking meta-packages; do NOT pin `-18`-suffixed names which break on image/llvm bumps) in addition to `build-essential` and `libc6-dev`, after the reclaimer step. Meta-packages automatically track the runner's active LLVM version and restore the clang resource headers removed by the reclaimer. Install is idempotent (no-op when packages already present). No `continue-on-error` — the packages are mandatory for the build; if the canonical archive fails, the job must fail loud. |
| EC-013 | apt-get install phase mirror failure ("E: Unable to fetch some archives") — the original two-attempt pattern guarded only `apt-get update`; a mirror that returns 200 on metadata but HTTP errors on package-fetch (partial mirror rotation) caused `apt-get install` to fail with "E: Unable to fetch some archives" while `apt-get update` succeeded, leaving the fallback untriggered | Combined update+install form per F-MAINT-P8-LOW-003: `if ! ( sudo apt-get update && sudo apt-get install -y <pkgs> ); then` — the outer condition spans both phases, so a failure at any point (update OR install) triggers the redesigned fallback (apt-mirrors.txt overwrite + dpkg repair + retry against archive.ubuntu.com priority:1; F-MAINT-P10-CRIT-001). |
| EC-014 | e2e.yml keyring dependency step vulnerable to EC-010 mirror-flake class — `.github/workflows/e2e.yml` `Install keyring runtime dependencies (libdbus-1-dev, gnome-keyring, dbus-x11)` step used single-attempt inline form; not covered by RG-5 (which only counts ci.yml sites) | AC-006 scope extended to e2e.yml per F-MAINT-P8-MED-004: step converted to combined two-attempt form; RG-7 in verify-workflow-structure confirms the pattern is present in e2e.yml (count ≥ 1). |
| EC-015 | Third-party apt source 403/404 causing fallback trigger — `packages.microsoft.com` repos are pre-installed on the ubuntu-24.04 runner image at `/etc/apt/sources.list.d/microsoft-prod.list` (classic .list) and `/etc/apt/sources.list.d/azure-cli.sources` (deb822). **NOTE: `azure-cli.list` does NOT exist on image 20260714.240.1** — v0.18's fallback `rm -f azure-cli.list` was ineffective (F-MAINT-P10-HIGH-002, probe 29540085270 confirmed). When those third-party sources return HTTP 403, `apt-get update` exits 100 and the two-attempt fallback wrapper triggers. The root cause is more fundamental: the ubuntu package source itself uses `mirror+file:` scheme (not http://), so even after third-party source removal, sed-based URL rewriting never had any effect on ubuntu package metadata. Both F-MAINT-P9-HIGH-001/002 and this finding are resolved together by the v0.19 redesign. | v0.19 redesigned fallback (F-MAINT-P10-CRIT-001): diagnostic dump on entry; remove four defensive variants of third-party source files (`microsoft-prod.list`, `azure-cli.sources`, `microsoft-prod.sources`, `azure-cli.list`); overwrite `/etc/apt/apt-mirrors.txt` to pin `http://archive.ubuntu.com/ubuntu/` at priority:1; `sudo dpkg --configure -a 2>/dev/null || true`; retry. RG-5b and RG-7b in `verify-workflow-structure` lock `sudo tee /etc/apt/apt-mirrors.txt` presence in all fallback blocks (count ≥ 12 in ci.yml, count ≥ 1 in e2e.yml). |

## §Purity Classification

This story modifies only CI workflow files; no production Rust modules are touched.
The pure-core / effectful-shell boundary does not apply.

| Module | Classification | Justification |
|--------|---------------|---------------|
| `.github/workflows/ci.yml` | N/A | CI toolchain YAML — not a Rust module |
| `.github/workflows/e2e.yml` | N/A | CI toolchain YAML — not a Rust module |

## §Research Trace

Remove-uncertainty pass applied 2026-07-15 (D-1110 directive; research-agent; 12 external
validations via Cargo reference docs, Rust 1.70/1.71 release notes, docs.rs, GitHub Actions
marketplace, insightsengineering fork README, GitHub Actions docs on runner disk provisioning,
and empirical ubuntu-24.04 runner measurements).

Findings applied in v0.3:

1. **CARGO_PROFILE_DEV_DEBUG value corrected (CRITICAL):** Numeric `1` maps to `"limited"`
   since Rust 1.70, NOT `"line-tables-only"`. Correct value is the string literal
   `"line-tables-only"` (Cargo 1.71+ accepts string profile values via env). All occurrences
   updated: AC-003 prose, AC-003 YAML snippet, AC-003 comment, §Library & Framework
   Requirements table, §Architecture Compliance Rules, §Forbidden Patterns, §Implementation
   Notes, §File Structure Requirements, §Tasks, §Edge Cases EC-003. The ban on `0` and `"0"`
   is preserved; the false claim that numeric `1 == line-tables-only` is removed.

2. **Disk-space-reclaimer action updated:** Replaced unmaintained `jlumbroso/free-disk-space`
   (last maintained 2023-10) with actively-maintained drop-in fork
   `insightsengineering/disk-space-reclaimer@dae9fabcb8febe09f6585471948acf9dc9a57489 # v1.1.2`.
   Documented fallback: `jlumbroso/free-disk-space@54081f138730dfa15788a46383842cd2f914a1be # v1.3.1`.
   Red Gate grep tightened from `free-disk-space` to `uses:.*insightsengineering/disk-space-reclaimer`.
   Inputs extended: `docker-images: true`, `large-packages: true`, `swap-storage: true` added.
   `tool-cache` / `tools-cache` rename noted; not relevant (story does not remove tool cache).

3. **Reclaim estimate corrected:** `14–18 GB` → `25–35 GB` on ubuntu-24.04 with
   android/dotnet/haskell/docker-images/large-packages/swap-storage enabled. The ≥25 GB
   post-reclaim gate is unchanged and achievable under the corrected input set.

4. **Root Cause Hypothesis clarified:** GitHub documents 14 GB as the guaranteed MINIMUM;
   empirical ubuntu-24.04 shows ~22–29 GB free at job start on ~72 GB root filesystem.
   Failures occur when runners land near the documented floor.

5. **Swatinem/rust-cache (optional note, no AC):** Current pin should be verified against
   v2.9.1 (`c19371144df3bb44fab255c43d04cbc2ab54d1c4`); default pruning suffices;
   reclaim-BEFORE-cache-restore ordering confirmed correct-and-important.

## §Changelog

- v0.19 (2026-07-16): PR-LEVEL pass-10 spec-layer fix-burst (HEAD frozen 0939973f; runs 29531645116/29531648104). **F-MAINT-P10-CRIT-001** [sed is a structural no-op on mirror+file: scheme]: probe run 29540085270 (image 20260714.240.1) confirmed that `/etc/apt/sources.list.d/ubuntu.sources` uses `URIs: mirror+file:/etc/apt/apt-mirrors.txt` — NOT any http:// URL. `/etc/apt/sources.list` is comment-only. ALL sed-based URL rewriting was always a structural no-op; v0.17 and v0.18 fallback blocks ran without changing anything. Root cause of all three fallback generations: wrong intervention point. Correct intervention: overwrite `/etc/apt/apt-mirrors.txt`. **F-MAINT-P10-HIGH-002** [azure-cli.sources not azure-cli.list]: probe confirmed `azure-cli.sources` (deb822) exists; `azure-cli.list` does NOT exist — v0.18 `rm -f azure-cli.list` was ineffective. Fixed: defensive removal of all four variants (microsoft-prod.list, azure-cli.sources, microsoft-prod.sources, azure-cli.list). **F-MAINT-P10-LOW-005** [|| true guards]: all non-fatal commands in new fallback carry `|| true` or `2>/dev/null || true`; retry update/install are intentionally NOT guarded. **F-MAINT-P10-LOW-006** [wrong sed `-e` expressions]: moot — all sed logic deleted. **F-MAINT-P10-MED-004** [AC-005 acquisition mechanism undefined]: AC-005 NOTE updated — three consecutive DISTINCT run IDs required; re-run attempts do NOT qualify; push-event runs on PR branch qualify (OBS-008 adjudication). **F-MAINT-P10-PG-009** [no self-evidencing on fallback entry]: diagnostic dump added to all 13 fallback blocks: `echo/cat apt-mirrors.txt` + `echo/ls sources.list.d/` with `|| true` guards. POL-29 sweep: all body references to old sed form deleted; new form uses `sudo tee /etc/apt/apt-mirrors.txt` as the canonical lock pattern. RG-5b pattern changed from `sudo rm -f .../microsoft-prod\.list` to `sudo tee /etc/apt/apt-mirrors\.txt` (count ≥ 12 in ci.yml). RG-7b pattern same change (count ≥ 1 in e2e.yml). All 13 YAML snippets redesigned via replace-all. EC-010 rewritten: mirror+file: mechanism explanation; cites probe 29540085270 + runs 29531645116/29531648104. EC-015 amended: azure-cli.list claim falsified; v0.19 resolution described. EC-013 updated: references new fallback. §ACR, §FSR, §Tasks, §Implementation Notes all updated. red_gate_tests: 9 (unchanged). Echo arithmetic: 21 total (unchanged — assertion count unchanged; only grep patterns change).
- v0.18 (2026-07-16): PR-LEVEL pass-9 spec-layer fix-burst (HEAD frozen bd65e93a). Two HIGH findings addressed. **F-MAINT-P9-HIGH-001** [fallback sed path-segment corruption]: the old pattern `s|https?://[^ ]*/ubuntu|https://azure.archive.ubuntu.com/ubuntu|g` matched `/ubuntu` as a PATH SEGMENT in third-party URLs (`packages.microsoft.com/ubuntu/24.04/prod`) because `[^ ]*` (greedy, no slash exclusion) matched `packages.microsoft.com` then `/ubuntu` matched literally — corrupting the MS source entry to `azure.archive.ubuntu.com/ubuntu/24.04/prod` (nonexistent). Evidence: run 29524703679 attempt-1 on bd65e93a. Redesigned fallback: (1) `sudo rm -f /etc/apt/sources.list.d/microsoft-prod.list /etc/apt/sources.list.d/azure-cli.list 2>/dev/null || true` — removes third-party source files whose packages we never install; (2) host-anchored sed using `[^/ ]*` (excludes both space and slash) with two `-e` expressions: trailing-space variant for classic .list format and trailing-`$` variant for deb822 .sources URIs: field — `/ubuntu` in a deeper path segment cannot match because it would be followed by `/` not whitespace/EOL. **F-MAINT-P9-HIGH-002** [https:// rewrite target on HTTP-only host]: old pattern emitted `https://azure.archive.ubuntu.com/ubuntu` but `azure.archive.ubuntu.com` is HTTP-only; `:443` timed out. Fixed: rewrite target changed to `http://azure.archive.ubuntu.com/ubuntu`. POL-29 exhaustive sweep: all 13 YAML snippets (10 AC-006 apt-install + 2 AC-007 C toolchain + 1 e2e.yml) updated via replace-all; `sed rewrite rationale (host-agnostic)` paragraph in AC-006 body replaced with redesigned rationale; §Implementation Notes "sed rewrite is host-agnostic" sentence updated; "rewrite any ubuntu mirror URL" comment lines (×3) updated. EC-010 rationale updated: "always-authoritative" → "authoritative over HTTP (HTTP-only)"; redesigned fallback described. EC-013: fallback-redesign reference added. EC-015 added: third-party-source 403 failure class (cites run 29524703679 attempt-1 evidence). RG-5b added: `^\s+sudo rm -f /etc/apt/sources.list.d/microsoft-prod\.list` count ≥ 12 in ci.yml. RG-7b added: same pattern count ≥ 1 in e2e.yml. red_gate_tests: 7 → 9. Echo arithmetic: 19 total (17 reachability + 2 config-invariant) → 21 total (19 reachability + 2 config-invariant). §Tasks: two new task bullets (RG-5b + RG-7b echo-bumps); final-task note added re: run 29524703679 disqualification. §ACR: seven → nine new assertions; 19 → 21 total. §FSR: RG-5b + RG-7b assertions added; 19/17 → 21/19 counts. AC-005: disqualification NOTE added for run 29524703679 attempt-1. Pass-9 LOW-002a (spec-adjacent nit): no `linux-test` phantom references introduced; all new content uses `test (Test matrix)` / `ci.yml` / `e2e.yml` forms.
- v0.17 (2026-07-16): PR-LEVEL pass-8 spec-side findings closed (PR #224 @4f9a5c6f; HEAD frozen — spec-only fix-burst). Six findings addressed: **F-MAINT-P8-MED-001** [§Architecture Compliance Rules + §Forbidden Patterns carve-outs]: replaced single prohibition row "fmt, clippy, deny, audit, semver-checks, test-no-default-features, non-exhaustive-violation-compile-fail must NOT be modified" with three targeted rules — fmt/deny/audit strictly prohibited; clippy/semver-checks/non-exhaustive-violation-compile-fail/perimeter-compile-fail/no-hardcoded-sensors-compile-fail/fuzz-smoke-vp021/shellcheck-demo-scripts permitted to add AC-006 wrapper only; test-no-default-features permitted to add four v0.6 steps + AC-006 + AC-007; §Forbidden Patterns table split into three rows accordingly. **F-MAINT-P8-MED-002** [arithmetic fixes]: 9 pre-existing → 12 (develop added wasm32-threatintel-staleness-check + F-MCPRS-PRL14-LOW-001 ×2 post-rebase); target echo bumped to 19 total (17 reachability + 2 config-invariant); §Implementation Notes pass-6 traceability rewritten; §Architecture Compliance Rules echo count 15→19, six→seven new assertions; AC-006/AC-007 echo-bump instructions updated (14→15 reachability/16→17 total and 15→16 reachability/17→18 total respectively); red_gate_tests: 6→7; §FSR echo count updated. **F-MAINT-P8-MED-004** [e2e.yml scope extension]: AC-006 extended to `.github/workflows/e2e.yml` `Install keyring runtime dependencies` step; YAML snippet added to AC-006 body; RG-7 assertion added to verify-workflow-structure (count ≥ 1 in e2e.yml); §Tasks: e2e.yml apt-wrapper task + RG-7 echo-bump bullet added; EC-014 added; §FSR: e2e.yml row added. **F-MAINT-P8-LOW-001** [phantom job name linux-test → test]: AC-001/AC-002/AC-007 error echoes corrected to `test (Test matrix)`; AC-007 heading + prose body corrected; §Tasks AC-001/AC-002 counting notes corrected; EC-011 corrected; POL-22 Phase C; changelog rows exempted per POL-32. **F-MAINT-P8-LOW-003** [combined update+install pattern]: all 12 apt snippets (10 AC-006 + 2 AC-007) updated from `if ! sudo apt-get update` to `if ! ( sudo apt-get update && sudo apt-get install -y <pkgs> )` combined form; fallback block retries both phases; RG-5 grep updated to `^\s+if ! \( sudo apt-get update && sudo apt-get install`; rationale paragraph added; EC-013 added. **F-MAINT-P8-OBS-001** [volatile line numbers removed]: §Tasks sibling-sweep sub-bullets stripped of `; line NNN in ci.yml` / `; line NNN` references per TD-VSDD-091; step-name/job-name anchors retained.
- v0.16 (2026-07-15): PR-LEVEL pass-4 spec-side findings closed (PR #224 @498ffb6c; HEAD frozen — spec-only fix-burst). F-CIDISK-PR4-MED-001 [§Architecture Compliance Rules AC-006 threshold propagation gap]: `AC-006 count ≥ 3` in the "six new assertions" sentence changed to `AC-006 count ≥ 12` — v0.14 (F-CIDISK-PR1-MED-002) tightened the Red Gate assertion and §Tasks/§FSR references but missed this prose sentence; POL-29 exhaustive sweep of all "≥3"-class AC-006 live sites confirms one site corrected, zero remaining outside historical changelog rows. F-CIDISK-PR4-LOW-001 [date corrections, orchestrator dispatch-date error]: v0.15 changelog entry date corrected 2026-07-16→2026-07-15; AC-002 adjudicated no-action items block header date corrected 2026-07-16→2026-07-15; full-file sweep of "2026-07-16" confirms zero occurrences remaining.
- v0.15 (2026-07-15): PR-LEVEL pass-3 spec-side findings closed (PR #224 @498ffb6c; HEAD frozen — spec-only fix-burst). F-CIDISK-PR3-LOW-001 [AC-002 snippet `with:` block quoted vs unquoted]: `with:` input values changed from quoted strings (`android: "true"`, `dotnet: "true"`, etc.) to unquoted YAML booleans (`android: true`, `dotnet: true`, ..., `swap-storage: false`) — matching the actual ci.yml implementation and the §Tasks/§Implementation Notes narrative (cosmetic; no semantic change to reclaimer behavior). F-CIDISK-PR3-OBS-001 [AC-002 snippet step-name drift]: single generic snippet replaced with two labeled variants following AC-007 v0.12 precedent — Test-matrix job uses `- name: Reclaim disk space (Linux only)`, `test-no-default-features` job uses `- name: Reclaim disk space`; intro prose updated to note step-name difference and identical `with:` blocks. F-CIDISK-PR3-OBS-002/003/004 [no-action ratification]: AC-006 ≥12 threshold by-design note, AC-007 step-position asymmetry within-ordering-contract note, and reclaimer `continue-on-error` EC-009 trade-off note added to AC-002 adjudicated no-action block; all three explicitly adjudicated to prevent re-finding in future fresh-context passes. No AC count changes; no red_gate_tests changes; no semantic behavior changes.
- v0.14 (2026-07-15): PR-LEVEL pass-1 findings closed (PR #224 @5cd2df5e). F-CIDISK-PR1-MED-001 [AC-007 outer preamble falsified-text correction]: both AC-007 snippets now include the 4-line outer step-level preamble comment block immediately above `- name:`, with corrected text citing self-inflicted toolchain removal (EC-012) rather than the falsified runner-image-omission hypothesis (EC-011); §FSR MODIFY row updated to document preamble requirement. F-CIDISK-PR1-MED-002 [AC-006 threshold 3→12, slack elimination, POL-34]: Red Gate assertion threshold changed from `[ "$count" -ge 3 ]` to `[ "$count" -ge 12 ]`; derivation: 10 AC-006 apt-install steps + 2 AC-007 toolchain installs all emit the `if ! sudo apt-get update; then` keyword; error/pass echo updated to name both categories and the 12-count basis; `(count≥3)` → `(count≥12)` in §Tasks verify-workflow-structure bullet and §FSR assertion references. F-CIDISK-PR1-OBS-001 [process-gap, full 7-site sweep per Canonical Principle Rule 4]: AC-006 scope extended from 3 to 10 sites; 7 new jobs converted to the two-attempt wrapper form — `clippy` (libdbus-1-dev pkg-config), `semver-checks` (libdbus-1-dev pkg-config), `fuzz-smoke-vp021` (libdbus-1-dev pkg-config), `perimeter-compile-fail` (libdbus-1-dev pkg-config), `non-exhaustive-violation-compile-fail` (libdbus-1-dev pkg-config), `no-hardcoded-sensors-compile-fail` (libdbus-1-dev pkg-config), `shellcheck-demo-scripts` (shellcheck); rationale: clippy is a `needs:` predecessor whose failure blocks the whole pipeline including AC-005 evidence; EC-010 mirror-flake class affects every apt job; 12-count derivation: 3 original + 7 new = 10 apt-install sites, plus 2 AC-007 C toolchain installs sharing the same two-attempt keyword; byte-exact snippets added to AC-006 for all 7 new sites; §Tasks: new 7-site sweep bullet added; §FSR and §Implementation Notes updated throughout.
- v0.13 (2026-07-15): root-cause correction per CI evidence (job 87471517229, run 29450000494, 2026-07-15). EC-011 v0.11 hypothesis falsified: original claim that runner image `ubuntu24/20260705.232` omits `libc6-dev` is refuted — AC-007 successfully installed `build-essential libc6-dev` at 21:05:52 but the build still failed at 21:10:26 with `stdbool.h`; true root cause is self-inflicted clang toolchain removal by this story's own AC-002 `large-packages: true` purge (removes `libclang-common-*-dev`; bindgen-based librocksdb-sys needs these clang builtin resource headers; masked on cache-hit runs). EC-011 amended with falsification note (ID preserved per POL-1 append-only). EC-012 added: self-inflicted toolchain removal root cause documentation. AC-007 corrected: both snippets' step names updated to `Install C toolchain baseline (build-essential, libc6-dev, clang, libclang-dev)`; comment lines updated with corrected root cause (DRIFT-CI-STDBOOL-001 revised); install lines extended to `sudo apt-get install -y build-essential libc6-dev clang libclang-dev`. Red Gate test 6 grep pattern updated to `^\s+sudo apt-get install -y build-essential libc6-dev clang libclang-dev\s*$`; error/pass echo updated to name all four packages. §Tasks AC-007 bullets, §FSR MODIFY row, and §Library & Framework Requirements updated throughout. Note: DRIFT-CI-STDBOOL-001 registry entry requires root-cause revision — state-manager owns the registry; reporting here, not editing directly. (DRIFT-CI-STDBOOL-001 revised, 2026-07-15.)
- v0.12 (2026-07-15): spec-precision amendment — AC-007 Test-matrix instance requires `if: runner.os == 'Linux'` guard (mixed-OS matrix includes macOS/Windows legs where apt-get is unavailable; semantic intent of "unconditional on Linux legs" preserved; DRIFT-CI-STDBOOL-001 lineage; discovered at implementation in fix-burst-9 @fa596f92). §Tasks AC-007 Linux Test leg bullet updated: "no `if:` condition (unconditional on Linux legs)" → "`if: runner.os == 'Linux'` guard required". AC-007 snippet split into two labeled forms: Test-matrix leg (with `if: runner.os == 'Linux'`) and test-no-default-features leg (unconditional, ubuntu-only). Red Gate test 6 grep unaffected (anchors to `run:` block content `^\s+sudo apt-get install -y build-essential libc6-dev\s*$`, not to any `if:` line).
- v0.11 (2026-07-15): DRIFT-CI-STDBOOL-001 adjudication per D-1791 (product-owner; POL-32). Runner-image regression: `ubuntu-latest` image `ubuntu24/20260705.232` ships without `libc6-dev` and `build-essential`, causing `rocksdb-sys v0.17.3+10.4.2` build script to fail with `fatal error: 'stdbool.h' file not found` in both Linux CI legs (pull_request runs). EC-011 added: runner-image toolchain regression class documentation. AC-007 added: explicit `sudo apt-get install -y build-essential libc6-dev` step via AC-006 two-attempt wrapper in both Linux workspace-build jobs, positioned BEFORE any cargo build phase. Red Gate test 6 added: `count=$(grep -cE '^\s+sudo apt-get install -y build-essential libc6-dev\s*$' .github/workflows/ci.yml)` count ≥ 2 in verify-workflow-structure. acceptance_criteria_count: 6→7. red_gate_tests: 5→6. §Architecture Compliance Rules: five→six new assertions. §Implementation Notes: pass-6 traceability count bumped 14→15 / 12→13 reachability. §File Structure Requirements and §Tasks updated. verify-workflow-structure summary echo must be updated from 14→15 total.
- v0.10 (2026-07-15): fix-burst-8 adjudication — PR #224 persistent apt-mirror blocker (product-owner; POL-32). AC-006 added: apt-mirror resilience for the three pre-existing `sudo apt-get update && sudo apt-get install ...` steps (Install musl-tools in test matrix; Install libdbus-1-dev pkg-config in test matrix; Install libdbus-1-dev pkg-config in test-no-default-features). Evidence: run 29438854846 rerun (2026-07-15) — mirror.enzu.com returned HTTP 404 Release files on BOTH the `musl-tools` and `libdbus-1-dev` steps after the EC-009 reclaimer `continue-on-error: true` fix had already landed; AC-005 unachievable while these steps depend on the broken mirror. Fix: two-attempt pattern — first `sudo apt-get update`; on failure sed-rewrite any ubuntu mirror URL in classic sources.list and deb822 .sources forms to `azure.archive.ubuntu.com`; then retry. EC-010 added: persistent apt-mirror outage class. Red Gate test 5 added: `count=$(grep -cE '^\s+if ! sudo apt-get update; then$' .github/workflows/ci.yml)` count ≥ 3 in verify-workflow-structure. acceptance_criteria_count: 5→6. red_gate_tests: 4→5. §Implementation Notes: apt-resilience note added; pass-6 traceability count bumped 13→14 / 11→12 reachability. §Architecture Compliance Rules: four→five new assertions. §File Structure Requirements and §Tasks updated. verify-workflow-structure summary echo must be updated from 13→14 total.
- v0.9 (2026-07-15): fix-burst-7 adjudication — PR #224 first live CI run failure (product-owner; POL-32). EC-009 added: apt-mirror flake / partial-reclaim class — `large-packages: true` invokes `apt-get` against the runner's rotating mirror; mirror.enzu.com returned HTTP 404 Release files (run 29437306537, 2026-07-15), apt exited 100, reclaimer step failed, ALL THREE hardened Linux jobs failed BEFORE the ≥25 GB gate ran. Fix: `continue-on-error: true` added to reclaimer step in BOTH Linux jobs (linux-test legs + test-no-default-features). AC-002 prose updated: YAML snippet added showing the step with `continue-on-error: true` + inline comment documenting the evidence and rationale. EC-001 updated: design changed from `continue-on-error: false` (default, prior) to `continue-on-error: true`; early-failure rationale replaced with gate-as-authoritative-check rationale. §Implementation Notes: new "Reclaimer step is BEST-EFFORT" paragraph added. §Tasks: `continue-on-error: true` added to both reclaimer task bullets. §File Structure Requirements: notes column updated. AC-002 count≥2 reachability assertion UNAFFECTED — grep anchors `^\s+uses: insightsengineering/disk-space-reclaimer`; `continue-on-error:` is a separate YAML key on a different line, not part of the `uses:` line. acceptance_criteria_count and red_gate_tests unchanged.
- v0.8 (2026-07-15): LOCAL pass-7 spec fix (product-owner fix-burst-6; POL-32). F-CIDISK-P7-MED-001: AC-003 assertions 3 and 4 redesigned from context-blind grep to section-scoped awk; assertion-3 now verifies debug = "line-tables-only" is within [profile.dev] (not just present anywhere in the file); assertion-4 now verifies debug = false payload is within [profile.dev.package."*"] (not just that the section header exists). §Tasks AC-003 assertion-3 and assertion-4 descriptions updated to awk forms. F-CIDISK-P7-LOW-001: AC-004 code snippet corrected to df -P / (was df /; now matches the ratified pass-6 form consistent with the AC-002 gate step); pass-6 traceability note shortened to drop the now-redundant df -P / annotation. F-CIDISK-P7-OBS-001: §Implementation Notes arithmetic rewritten to code-authoritative grouping: 9 pre-existing reachability + 2 new count-based reachability + 2 new config-invariant = 13 total (11 reachability + 2 config-invariant, matching the ci.yml summary echo).
- v0.7 (2026-07-15): LOCAL pass-6 F-CIDISK-P6-MED-001 spec fix (product-owner fix-burst-5; POL-32). §Architecture Compliance Rules: removed `test-no-default-features` from the "jobs that must NOT be modified" list — that claim directly contradicted the v0.6 scope expansion (AC-001, AC-002, AC-004, and every §Tasks/§File Structure bullet explicitly require adding four protective steps to that job); replaced with the precise invariant: the job's existing `PROPTEST_CASES`, `RUSTFLAGS`, test-invocation lines, and cache configuration must NOT be changed; only the four v0.6-ratified protective steps may be added. §Implementation Notes: added one-line traceability note for pass-6 LOW-001 (summary-echo count corrected 9→11: 7 anchored-in-place + 4 new = 11 total) and OBS-002 (AC-004 annotation step df call updated to df -P / for consistency with AC-002 gate step), both applied by implementer in same burst.
- v0.6 (2026-07-15): LOCAL pass-4 adjudication (F-CIDISK-P4-HIGH-001 + F-CIDISK-P4-MED-001 + F-CIDISK-P4-MED-002 + F-CIDISK-P4-HIGH-002 + F-CIDISK-P4-LOW-001). **HIGH-001** (AC-003 no-op): `CARGO_PROFILE_DEV_DEBUG` env var removed from scope — it is a no-op because `.cargo/config.toml` already sets `debug = "line-tables-only"` + `debug = false` for deps at higher precedence (config.toml was active during the failures). AC-003 replaced: now a `.cargo/config.toml` invariant guard with 2 new verify-workflow-structure assertions (Red Gate tests 3 + 4: `grep -qE '^debug = "line-tables-only"' .cargo/config.toml` + `grep -qF '[profile.dev.package."*"]' .cargo/config.toml`). §Root Cause Hypothesis rewritten: "full DWARF debug symbols" claim replaced with accurate description of pre-existing minimal-DWARF config and explicit statement that debug-info axis was not a mitigation lever. EC-003/EC-004/EC-006 retired (all referenced CARGO_PROFILE_DEV_DEBUG). `CARGO_PROFILE_DEV_DEBUG` added to §Forbidden Patterns. `du -sh target/` PR-description task removed. **MED-001** (swap-storage OOM): `swap-storage: true` → `swap-storage: false`; reclaim estimate 25–35 GB → 21–31 GB; AC-002 prose + §Implementation Notes + §Library + §File Structure + §Forbidden Patterns updated; EC-008 added (trade-off justification). **MED-002** (test-no-default-features unprotected): Three protective steps (preflight, reclaimer+gate, annotation) mirrored into `test-no-default-features` job. AC-001 + AC-002 assertions changed from `-q` presence to count ≥ 2 semantics. AC-001/AC-002/AC-004 headings/prose updated to cover both Linux workspace-build jobs. §Tasks, §Implementation Notes, §Architecture Compliance Rules, §File Structure Requirements updated accordingly. **HIGH-002 + LOW-001** (AC-7/AC-8 self-match): `semver-checks` assertion anchored to `^  semver-checks:`, `test-no-default-features` assertion anchored to `^  test-no-default-features:` (2-space job-name indent). Added to §Tasks sibling-sweep task (7 pre-existing, up from 5). §Implementation Notes updated: "5 pre-existing" → "7 pre-existing". red_gate_tests: 2 → 4 (AC-003 adds 2 new config-invariant assertions). Story title updated (frontmatter + H1): `CARGO_PROFILE_DEV_DEBUG="line-tables-only"` → `cargo-config debug-invariant guard`.
- v0.5 (2026-07-15): LOCAL pass-2 fix-burst-3 spec update (F-CIDISK-P2-MED-001 systemic sweep + F-CIDISK-P2-LOW-001). MED-001 (systemic): All 7 reachability assertions in `verify-workflow-structure` tightened to self-match-proof anchored forms. AC-001 grep: `grep -qE '^\s+- name: Report initial disk space\s*$'` (YAML step-name anchor; indent-agnostic; assertion line starts with whitespace+grep, not whitespace+"- name:", so `^\s+- name:` anchor cannot self-match). AC-002 Red Gate grep: `grep -qE '^\s+uses: insightsengineering/disk-space-reclaimer'` (YAML uses: key anchor; assertion line starts with whitespace+grep, not whitespace+"uses:"; same reasoning). 5 pre-existing assertions anchored by YAML structure type: job-name greps (non-exhaustive-violation-compile-fail → `^  non-exhaustive-violation-compile-fail:`, wasm32-compile-check → `^  wasm32-compile-check:`, no-hardcoded-sensors-compile-fail → `^  no-hardcoded-sensors-compile-fail:`, shellcheck-demo-scripts → `^  shellcheck-demo-scripts:`) use 2-space GitHub Actions job-name indent; build-plugin-crowdstrike-oauth2 → `^\s+just build-plugin-crowdstrike-oauth2\s*$` uses just-recipe anchor with `$` to exclude comment lines. LOW-001: AC-002 ≥25 GB gate snippet adds `AVAIL_GB=${AVAIL_GB:-0}` guard (mirroring AC-004 pattern; prevents `-ge` failure when `df` itself fails under `if: failure()` conditions). §Tasks updated with explicit 5-assertion sibling-sweep task specifying exact replacement commands and ci.yml line numbers. §Implementation Notes, §Architecture Compliance Rules, §File Structure Requirements updated to reflect 7-assertion scope (2 new + 5 anchored-in-place). red_gate_tests: 2 unchanged (anchoring improves quality, not count).
- v0.4 (2026-07-15): LOCAL pass-1 fix-burst (F-CIDISK-P1-MED-001 + F-CIDISK-P1-LOW-001 + F-CIDISK-P1-LOW-002 + F-CIDISK-P1-OBS-001). MED-001: AC-001 Red Gate grep tightened from `'Report initial disk space|df -h'` to `'name: Report initial disk space'` — the `df -h` alternation matched unrelated ci.yml lines and made step removal undetectable. LOW-001: AC-002 ≥25 GB gate snippet replaced `gsub(/G/, "", $4)` (no-op on 1K-block `df /` output; would break silently if df format changed) with `df -P /` and explicit `int($4 / 1024 / 1024)` arithmetic; added unit comment documenting that `$4` is 1K-blocks. LOW-002: AC-004 annotation snippet adds `USED_PCT=${USED_PCT:-0}` guard so the `[ -ge 95 ]` test never fails on empty awk output when `df` itself fails under the `if: failure()` step. OBS-001: AC-001, AC-002, and AC-004 headings and prose now carry explicit "Linux legs only — the failure locus is exclusive to Linux runners; macOS/Windows legs are exempt" scope qualifier. §Tasks updated: preflight/reclaim/annotation task descriptions say "Linux Test job legs"; grep assertion task records both AC-001 and AC-002 patterns. §File Structure Requirements notes column updated to distinguish Linux-only steps from all-legs env entry.
- v0.3 (2026-07-15): Remove-uncertainty pass (D-1110 directive; research-agent; 12 external validations). CRITICAL: `CARGO_PROFILE_DEV_DEBUG: 1` corrected to `"line-tables-only"` — numeric 1 maps to "limited" not line-tables-only since Rust 1.70 (Cargo reference + Rust 1.70/1.71 release notes). `jlumbroso/free-disk-space` replaced with actively-maintained fork `insightsengineering/disk-space-reclaimer@dae9fabcb8febe09f6585471948acf9dc9a57489 # v1.1.2`; fallback pin `jlumbroso@54081f138730dfa15788a46383842cd2f914a1be # v1.3.1` recorded. Reclaim estimate `14–18 GB` → `25–35 GB` on ubuntu-24.04 (docker-images/large-packages/swap-storage inputs added). Root Cause Hypothesis: 14 GB = GitHub-documented minimum; empirical ~22–29 GB; failures occur at the floor. Red Gate grep tightened to `uses:.*insightsengineering/disk-space-reclaimer`. Swatinem/rust-cache ordering confirmed correct (optional note added; no AC). All AC/body/rules/tasks/library-table updated consistently. Status: draft → ready (S-7.01 draft-blocker resolved by v0.2 PO adjudication; uncertainties closed by this pass).
- v0.2 (2026-07-15): PO BC adjudication — Option B ratified (no BC required for CI-toolchain-only stories). `behavioral_contracts: []` resolved as CONFORMING; S-7.01 draft-blocker cleared. Controlling precedent: W3-FIX-CI-001 (merged, PR #112). No BC created, no BC-INDEX row needed.
- v0.1 (2026-07-15): Initial draft — story-writer. D-1780 watch-note 3rd-occurrence materialization. 5 ACs, 2 Red Gate tests, 5 pts, P2.
