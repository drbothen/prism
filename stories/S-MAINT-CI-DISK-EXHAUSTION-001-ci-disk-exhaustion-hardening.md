---
document_type: story
story_id: S-MAINT-CI-DISK-EXHAUSTION-001
title: "CI disk-exhaustion hardening — preflight + disk-space-reclaimer + cargo-config debug-invariant guard + failure annotation"
wave: tbd
epic_id: maintenance
priority: P2
status: ready
version: "0.10"
level: ops
producer: story-writer
timestamp: "2026-07-15"
modified: "2026-07-15"
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
acceptance_criteria_count: 6
red_gate_tests: 5
estimated_passes: "1"
holdout_scenarios: []
assumption_validations: []
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
# Count-based: must appear in ≥2 Linux jobs (linux-test + test-no-default-features).
# The assertion line starts with whitespace+count=$(grep..., not whitespace+"- name:", so
# the ^ anchor cannot self-match.
count=$(grep -cE '^\s+- name: Report initial disk space\s*$' .github/workflows/ci.yml)
[ "$count" -ge 2 ] || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-001: disk preflight step missing from ≥2 Linux jobs (found ${count}; need linux-test + test-no-default-features)"
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
The reclaimer step snippet applied to BOTH Linux jobs:

```yaml
- name: Free disk space (best-effort)
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
    android: "true"
    dotnet: "true"
    haskell: "true"
    docker-images: "true"
    large-packages: "true"
    swap-storage: "false"
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
# Count-based: must appear in ≥2 Linux jobs (linux-test + test-no-default-features).
# The assertion line starts with whitespace+count=$(grep..., not whitespace+"uses:", so
# the ^ anchor cannot self-match.
count=$(grep -cE '^\s+uses: insightsengineering/disk-space-reclaimer' .github/workflows/ci.yml)
[ "$count" -ge 2 ] || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-002: disk-space-reclaimer missing from ≥2 Linux jobs (found ${count}; need linux-test + test-no-default-features)"
  exit 1
}
```

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

The changes in this story MUST ride `maintenance/ci-disk-hardening` and NOT be merged
through any open defect PR branch (e.g., `fix/DEFECT-*`), to keep CI infrastructure
changes cleanly bisectable.

### AC-006 — apt-mirror resilience for pre-existing apt install steps (test matrix + test-no-default-features)

The three pre-existing `sudo apt-get update && sudo apt-get install ...` steps are
vulnerable to the same apt-mirror 404 class that motivated EC-009 (evidence: run
29438854846 rerun, 2026-07-15 — both the `musl-tools` step in the Test matrix AND the
`libdbus-1-dev pkg-config` step in `test-no-default-features` failed identically after
the reclaimer's `continue-on-error: true` fix had already landed; the broken mirror
blocked ALL remaining apt install steps in the same CI run). Each of the three steps
MUST be converted from the single-attempt inline form to the two-attempt pattern below.
The reclaimer step itself is NOT given this wrapper — it already carries `continue-on-error:
true` best-effort semantics per EC-009; the ≥25 GB gate is the authoritative check there.

**Snippet — `Install musl-tools` (test matrix; `if: matrix.install_musl`):**

```yaml
      - name: Install musl-tools
        if: matrix.install_musl
        run: |
          # Apt-mirror resilience: two-attempt pattern.
          # Evidence: runs 29437306537 + 29438854846 rerun (PR #224, 2026-07-15) — mirror.enzu.com
          # returned HTTP 404 Release files; apt exit code 100. Runner-image mirror selection is
          # outside our control; on failure rewrite any ubuntu mirror URL to the canonical archive.
          if ! sudo apt-get update; then
            sudo sed -i -E 's|https?://[^ ]*/ubuntu|https://azure.archive.ubuntu.com/ubuntu|g' \
              /etc/apt/sources.list /etc/apt/sources.list.d/*.list 2>/dev/null || true
            sudo sed -i -E 's|https?://[^ ]*/ubuntu|https://azure.archive.ubuntu.com/ubuntu|g' \
              /etc/apt/sources.list.d/*.sources 2>/dev/null || true
            sudo apt-get update
          fi
          sudo apt-get install -y musl-tools
```

**Snippet — `Install libdbus-1-dev (required by keyring build on Linux, all-features)` (test matrix; `if: runner.os == 'Linux'`):**

```yaml
      - name: Install libdbus-1-dev (required by keyring build on Linux, all-features)
        if: runner.os == 'Linux'
        run: |
          # Apt-mirror resilience: two-attempt pattern.
          # Evidence: runs 29437306537 + 29438854846 rerun (PR #224, 2026-07-15) — mirror.enzu.com
          # returned HTTP 404 Release files; apt exit code 100. Runner-image mirror selection is
          # outside our control; on failure rewrite any ubuntu mirror URL to the canonical archive.
          if ! sudo apt-get update; then
            sudo sed -i -E 's|https?://[^ ]*/ubuntu|https://azure.archive.ubuntu.com/ubuntu|g' \
              /etc/apt/sources.list /etc/apt/sources.list.d/*.list 2>/dev/null || true
            sudo sed -i -E 's|https?://[^ ]*/ubuntu|https://azure.archive.ubuntu.com/ubuntu|g' \
              /etc/apt/sources.list.d/*.sources 2>/dev/null || true
            sudo apt-get update
          fi
          sudo apt-get install -y libdbus-1-dev pkg-config
```

**Snippet — `Install libdbus-1-dev (required by keyring Secret Service backend on Linux)` (test-no-default-features; unconditional):**

```yaml
      - name: Install libdbus-1-dev (required by keyring Secret Service backend on Linux)
        run: |
          # Apt-mirror resilience: two-attempt pattern.
          # Evidence: runs 29437306537 + 29438854846 rerun (PR #224, 2026-07-15) — mirror.enzu.com
          # returned HTTP 404 Release files; apt exit code 100. Runner-image mirror selection is
          # outside our control; on failure rewrite any ubuntu mirror URL to the canonical archive.
          if ! sudo apt-get update; then
            sudo sed -i -E 's|https?://[^ ]*/ubuntu|https://azure.archive.ubuntu.com/ubuntu|g' \
              /etc/apt/sources.list /etc/apt/sources.list.d/*.list 2>/dev/null || true
            sudo sed -i -E 's|https?://[^ ]*/ubuntu|https://azure.archive.ubuntu.com/ubuntu|g' \
              /etc/apt/sources.list.d/*.sources 2>/dev/null || true
            sudo apt-get update
          fi
          sudo apt-get install -y libdbus-1-dev pkg-config
```

**sed rewrite rationale (host-agnostic):** The pattern `s|https?://[^ ]*/ubuntu|https://azure.archive.ubuntu.com/ubuntu|g` rewrites any ubuntu-suite mirror URL to the canonical azure archive regardless of which mirror the runner image was configured to use. It covers both classic `deb https://<mirror>/ubuntu <suite>` lines (in `/etc/apt/sources.list` and `/etc/apt/sources.list.d/*.list`) and deb822 `URIs: https://<mirror>/ubuntu` lines (in `/etc/apt/sources.list.d/*.sources`, which ubuntu-24.04 runner images ship). The `2>/dev/null || true` on each `sed` invocation handles the case where the target glob expands to no files (no-op, error suppressed). The fallback `sudo apt-get update` is NOT wrapped in `|| true` — if the canonical archive also fails, the step must fail loud.

The `verify-workflow-structure` job gains a Red Gate assertion confirming the two-attempt pattern is present in ≥3 steps (one per apt install step; Red Gate test 5):

```bash
# Anchored to the two-attempt pattern keyword — count-based, self-match-proof.
# The assertion line starts with whitespace+count=$(grep..., so the if-pattern cannot self-match.
count=$(grep -cE '^\s+if ! sudo apt-get update; then$' .github/workflows/ci.yml)
[ "$count" -ge 3 ] || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-006: apt-mirror resilience (two-attempt pattern) missing from ≥3 apt install steps (found ${count}; need install-musl + libdbus-linux-test + libdbus-no-default-features)"
  exit 1
}
echo "S-MAINT-CI-DISK-EXHAUSTION-001 AC-006 check passed: apt-mirror resilience found ${count} times (≥3 required)."
```

The implementer MUST also update the final summary echo in `verify-workflow-structure` to add `+ S-MAINT-CI-DISK-EXHAUSTION-001-AC-006 (count≥3)` to the assertion list and bump the total from `13` to `14` (`11 reachability` → `12 reachability`).

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
BOTH reclaimer steps (linux-test legs and test-no-default-features). The ≥25 GB gate is
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

**Pre-existing apt install steps — two-attempt pattern (AC-006):** The three existing `sudo apt-get update && sudo apt-get install ...` steps (Install musl-tools in test matrix; Install libdbus-1-dev in test matrix; Install libdbus-1-dev in test-no-default-features) must each be expanded from the single-attempt inline form into the two-attempt pattern specified in AC-006. The sed rewrite is host-agnostic: it matches any `https?://…/ubuntu` mirror URL (not just mirror.enzu.com), covering both classic sources.list and deb822 .sources file formats. The `2>/dev/null || true` on each sed silences missing-file errors. The fallback `sudo apt-get update` is NOT `|| true` — canonical archive failures fail loud. No `continue-on-error: true` on these steps; they are not best-effort (unlike the disk-space-reclaimer). See AC-006 for byte-exact snippets and the Red Gate assertion (count ≥ 3).

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

**Pass-6 traceability (LOW-001 + OBS-002):** The ci.yml summary echo must use the code-authoritative assertion count: 9 pre-existing reachability + 3 new count-based reachability + 2 new config-invariant = 14 total (12 reachability + 2 config-invariant, matching the ci.yml summary echo). (Bumped from 13→14 / 11→12 reachability by AC-006 v0.10 addition.)

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
  - AC-001 count assertion: `count=$(grep -cE '^\s+- name: Report initial disk space\s*$' .github/workflows/ci.yml)` + `[ "$count" -ge 2 ]` (counts linux-test + test-no-default-features)
  - AC-002 count assertion: `count=$(grep -cE '^\s+uses: insightsengineering/disk-space-reclaimer' .github/workflows/ci.yml)` + `[ "$count" -ge 2 ]` (counts linux-test + test-no-default-features)
  - AC-003 assertion-3: `awk '/^\[profile\.dev\]$/{s=1;next} /^\[/{s=0} s && /^debug = "line-tables-only"$/{found=1} END{exit !found}' .cargo/config.toml` (section-scoped: verifies debug = "line-tables-only" is within [profile.dev], not merely present anywhere in the file; self-match impossible — different file)
  - AC-003 assertion-4: `awk '/^\[profile\.dev\.package\."\*"\]$/{s=1;next} /^\[/{s=0} s && /^debug = false$/{found=1} END{exit !found}' .cargo/config.toml` (section-scoped: verifies debug = false payload is within [profile.dev.package."*"], not just that the section header exists; self-match impossible — different file)
- [ ] Apply self-match-proof anchoring to the 7 pre-existing verify-workflow-structure reachability assertions IN THE SAME COMMIT (5 from LOCAL pass-2 + AC-7 semver-checks + AC-8 test-no-default-features; F-CIDISK-P4-HIGH-002 + LOW-001, adjudicated 2026-07-15). Exact replacements:
  - `non-exhaustive-violation-compile-fail`: `grep -qE 'non-exhaustive-violation-compile-fail'` → `grep -qE '^  non-exhaustive-violation-compile-fail:'` (job-name anchor; 2-space GitHub Actions job indent; line 688 in ci.yml)
  - `wasm32-compile-check`: `grep -qE 'wasm32-compile-check'` → `grep -qE '^  wasm32-compile-check:'` (job-name anchor; 2-space indent; line 235)
  - `build-plugin-crowdstrike-oauth2`: `grep -qE 'build-plugin-crowdstrike-oauth2'` → `grep -qE '^\s+just build-plugin-crowdstrike-oauth2\s*$'` (just-recipe anchor; matches `          just build-plugin-crowdstrike-oauth2` at 10-space indent, line 286; `$` excludes comment lines)
  - `no-hardcoded-sensors-compile-fail`: `grep -qE 'no-hardcoded-sensors-compile-fail'` → `grep -qE '^  no-hardcoded-sensors-compile-fail:'` (job-name anchor; 2-space indent; line 727)
  - `shellcheck-demo-scripts`: `grep -qE 'shellcheck-demo-scripts'` → `grep -qE '^  shellcheck-demo-scripts:'` (job-name anchor; 2-space indent; line 1244)
  - `semver-checks` (AC-7): `grep -qE 'semver-checks'` → `grep -qE '^  semver-checks:'` (job-name anchor; 2-space indent; F-CIDISK-P4-HIGH-002)
  - `test-no-default-features` (AC-8): `grep -qE 'test-no-default-features'` → `grep -qE '^  test-no-default-features:'` (job-name anchor; 2-space indent; F-CIDISK-P4-LOW-001)
  - Self-match proof for all seven: assertion lines start with whitespace+`grep`, so job-name anchors `^  <job-name>:` and just-recipe anchor `^\s+just ...\s*$` cannot match the assertion lines themselves
- [ ] Test matrix job — `Install musl-tools` step: replace `run: sudo apt-get update && sudo apt-get install -y musl-tools` with the two-attempt mirror-resilience form from AC-006 (byte-exact snippet; preserves `if: matrix.install_musl` conditional) (AC-006)
- [ ] Test matrix job — `Install libdbus-1-dev (required by keyring build on Linux, all-features)` step: replace `run: sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config` with the two-attempt form (byte-exact snippet; preserves `if: runner.os == 'Linux'` conditional) (AC-006)
- [ ] `test-no-default-features` job — `Install libdbus-1-dev (required by keyring Secret Service backend on Linux)` step: replace `run: sudo apt-get update && sudo apt-get install -y libdbus-1-dev pkg-config` with the two-attempt form (byte-exact snippet; unconditional step) (AC-006)
- [ ] `verify-workflow-structure` job: add AC-006 Red Gate assertion (`count=$(grep -cE '^\s+if ! sudo apt-get update; then$' .github/workflows/ci.yml)` + `[ "$count" -ge 3 ]`) before the final summary echo; update summary echo: add `+ S-MAINT-CI-DISK-EXHAUSTION-001-AC-006 (count≥3)` to assertion list; change `11 reachability assertions` to `12 reachability assertions`; change `= 13 total checks` to `= 14 total checks` (AC-006 Red Gate test 5)
- [ ] Record three consecutive green CI run IDs in the PR description (AC-005 evidence)

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
- The `fmt`, `clippy`, `deny`, `audit`, `semver-checks`, and `non-exhaustive-violation-compile-fail`
  jobs in `ci.yml` must NOT be modified; the `test-no-default-features` job MAY be modified only
  to add the four v0.6-ratified protective steps (preflight, disk-space-reclaimer + ≥25 GB gate,
  failure annotation) — the job's existing `PROPTEST_CASES`, `RUSTFLAGS`, test-invocation lines,
  and cache configuration must NOT be changed
- The `verify-workflow-structure` job's existing assertions (AC-5 `TARGET_COUNT >= 5`,
  AC-6 cargo-deny/audit, AC-7 semver, AC-8 no-default-features, non-exhaustive, wasm32
  checks) must ALL pass after this story's modifications; the five new assertions (AC-001
  count ≥ 2, AC-002 count ≥ 2, AC-006 count ≥ 3, AC-003 assertions 3 and 4) are additive,
  and the 7 pre-existing reachability assertions (5 original + AC-7 semver-checks + AC-8
  test-no-default-features) are updated in-place to self-match-proof anchored forms (see
  §Tasks sibling-sweep task); no other structural changes. The final summary echo must
  reflect 14 total checks (12 reachability + 2 config-invariant)
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

No new `apt-get` packages. No Python packages. No compiled Cargo dev-dependencies.
No new GitHub Actions secrets required.

## §File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/ci.yml` | MODIFY | Linux Test job legs: add preflight (AC-001), disk-space-reclaimer with `swap-storage: false, continue-on-error: true` (AC-002; best-effort per EC-009; gate is the authoritative check), ≥25 GB gate (AC-002; `df -P /` 1K-block form + `AVAIL_GB=${AVAIL_GB:-0}` guard), failure annotation (AC-004; `USED_PCT=${USED_PCT:-0}` guard); `test-no-default-features` job: mirror same three protective steps + failure annotation (F-CIDISK-P4-MED-002); DO NOT add `CARGO_PROFILE_DEV_DEBUG` (AC-003 — it is a no-op; forbidden); convert three pre-existing `sudo apt-get update && sudo apt-get install ...` steps to two-attempt mirror-resilience form (AC-006; byte-exact snippets in AC-006 section); `verify-workflow-structure` job: AC-001 count assertion (≥2; `^\s+- name: Report initial disk space\s*$`) + AC-002 count assertion (≥2; `^\s+uses: insightsengineering/disk-space-reclaimer`) + AC-006 count assertion (≥3; `^\s+if ! sudo apt-get update; then$`) + two AC-003 `.cargo/config.toml` invariant checks + anchor 7 pre-existing reachability assertions (5 original + AC-7 `semver-checks` + AC-8 `test-no-default-features`; see §Tasks sibling-sweep) + update summary echo from 13→14 total / 11→12 reachability; no other jobs touched |

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
| Modifying `fmt`, `clippy`, `deny`, `audit`, `semver-checks`, `test-no-default-features`, or `non-exhaustive-violation-compile-fail` jobs | Out of scope for this story; single-responsibility |
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
| EC-010 | Persistent apt-mirror outage affecting pre-existing apt install steps: the runner image selects its apt mirror at image-build time; a mirror that returns HTTP 404 Release files (or is otherwise unreachable) causes all `sudo apt-get update` invocations in the job to fail with exit code 100, regardless of `continue-on-error` on the reclaimer step (the reclaimer's `continue-on-error` does not propagate to other steps). Evidence: run 29438854846 rerun (2026-07-15) — mirror.enzu.com 404 blocked `Install musl-tools` (test matrix) and `Install libdbus-1-dev pkg-config` (test-no-default-features) even after EC-009 fix had landed. Runner-image mirror selection is entirely outside our control; we cannot pre-screen or change the runner's default apt mirror. | Two-attempt pattern per AC-006: first attempt `sudo apt-get update`; on failure rewrite all ubuntu-suite mirror URLs in both classic sources.list forms (`/etc/apt/sources.list` and `/etc/apt/sources.list.d/*.list`) and deb822 form (`/etc/apt/sources.list.d/*.sources`) to `azure.archive.ubuntu.com` (always-authoritative canonical archive); then retry `sudo apt-get update`. Trade-off: the fallback adds ~seconds only on failure (rewrite + retry is cheap); azure.archive.ubuntu.com is slower than mirrors but always authoritative. No `continue-on-error` on these steps — they are not best-effort; if the canonical archive also fails, the job fails loud. |

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
