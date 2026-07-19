---
title: "F1 Delta Analysis — Release Engineering + Demo Bundle + Consumer Contract"
feature: DEMO-BLOCKING / v1.0.0-rc release engineering
phase: F1
status: draft
date: 2026-07-19
inputs:
  - .github/workflows/release.yml
  - .github/workflows/e2e.yml
  - lefthook.yml
  - crates/prism-bin/Cargo.toml
  - crates/prism-spec-engine/Cargo.toml
  - Cargo.toml
  - scripts/demo-setup.sh
  - scripts/demo-run.sh
  - scripts/demo-teardown.sh
  - docs/DEMO-RUNBOOK.md
  - crates/prism-bin/src/main.rs
input-hash: b48be6f
---

# F1 Delta Analysis: Release Engineering + Demo Bundle + Consumer Contract

## 1. Scope Summary

This feature cycle delivers:
1. A working GitHub Releases release workflow for `prism` (5-platform binary + install scripts)
2. A separate per-platform DTU demo bundle artifact (not consumed by secops-factory)
3. A versioned cross-repo consumption contract for secops-factory
4. A version-alignment ADR (ADR-053) for product version vs crate version relationship

The RC acceptance gate: successful live demo through the secops-factory plugin. Tag `v1.0.0-rc.1` is cut from `develop` AFTER that gate passes.

---

## 2. Impact Boundary — Files and Workflows Touched

### 2.1 Files That Must Change

| File | Why | Risk |
|------|-----|------|
| `.github/workflows/release.yml` | 5 defects (see §3); binary_exists guard removal; archive expansion; Chocolatey job removal; Homebrew job disable; crates.io job disable | HIGH — single point of failure for release distribution |
| `scripts/install.sh` | Does not exist; must be created for macOS/Linux install | MEDIUM — new file, no regressions |
| `scripts/install.ps1` | Does not exist; must be created for Windows install | MEDIUM — new file, no regressions |
| `scripts/demo-bundle.sh` | Does not exist; packages the DTU demo bundle artifact | MEDIUM — new file, no regressions |
| `docs/RELEASING.md` | Does not exist; operator runbook for cutting a release | LOW — docs only |
| `.factory/planning/feature-release-engineering/release-config.yaml` | Does not exist; machine-readable release config consumed by ADR and stories | LOW — config-adjacent |
| `.factory/specs/architecture/decisions/ADR-053-*.md` | New ADR: product version (git tag) vs crate version relationship | LOW — documentation |

### 2.2 Files That Need Audit but No Change Expected

| File | Audit Finding | Verdict |
|------|--------------|---------|
| `lefthook.yml` pre-tag | Runs `just semver-checks && just audit && just deny` — all correct for release gate | No change needed |
| `.github/workflows/e2e.yml` | Two-binary dependency (`prism` + `prism-dtu-demo-server`); both must be in the release build but the demo server ships in the demo bundle, NOT in the main release archive | Confirm separation in release.yml repair; e2e.yml itself is correct |
| `crates/prism-bin/Cargo.toml` | `publish = false`; `version = "0.1.0"` | Version alignment ADR must declare `prism-bin` version tracks the product tag (bump to `1.0.0-rc.1` on first RC); `publish = false` stays (binary crate ships via GitHub Releases, not crates.io) |
| `Cargo.toml` (workspace) | No `[workspace.package]` — crate versions are non-uniform (see §4) | Version alignment ADR addresses; no workspace.package change needed |
| `crates/prism-bin/src/main.rs` | `prism version` subcommand prints `env!("CARGO_PKG_VERSION")` — correct; but note it is `prism version` (subcommand), not `prism --version` (clap flag); clap also auto-generates `--version` | Consumer contract must document both forms |
| ADR-022 boot contract | No engine changes required; release engineering is entirely in the release.yml + packaging layer | No architecture changes |

### 2.3 Files That Must NOT Change

- All `crates/**/*.rs` — no engine changes
- `.github/workflows/ci.yml` — no CI logic changes  
- `.github/workflows/e2e.yml` — no changes (e2e is an input, not an output)
- `.factory/STATE.md`, `.factory/SESSION-HANDOFF.md` — state-manager owns

---

## 3. Release.yml Defect Catalog

These defects were found by reading the full file. All must be fixed before v1.0.0-rc.1 can ship.

### DEF-REL-001 — Stale `binary_exists` guard (MEDIUM)

**Location:** Lines 40–51, 107, 128, 170, 188.

**Defect:** The `binary_exists` guard checks `if [[ -d "crates/prism-bin" ]]; then` and gates all downstream jobs on `needs.build-release.outputs.binary_exists == 'true'`. This was scaffolded before `prism-bin` existed. `prism-bin` now exists permanently; the guard is dead weight. Additionally, `binary_exists` is a matrix-level output — GitHub Actions takes the last completed matrix job's output value, which is non-deterministic across concurrent runners.

**Fix:** Remove the `check_binary` step and all `if: steps.check_binary.outputs.binary_exists == 'true'` guards. Remove the `outputs` block from `build-release`. Update downstream `if:` conditions to always run (or remove the `if:` entirely on kept jobs).

**RC-1 blocking:** YES.

### DEF-REL-002 — Chocolatey job references nonexistent `packaging/` directory (CRITICAL)

**Location:** Lines 167–186.

**Defect:** `choco pack packaging/chocolatey/prism.nuspec` references `packaging/chocolatey/prism.nuspec` which does not exist. The entire `chocolatey-publish` job will fail with a file-not-found error on every tag push. The `packaging/` directory does not exist anywhere in the repository.

**Fix:** Remove the `chocolatey-publish` job entirely per approved decisions. Windows is first-class via `install.ps1` from GitHub Releases; Chocolatey is a v1.1+ consideration when a real packaging maintainer is available.

**RC-1 blocking:** YES.

### DEF-REL-003 — Homebrew job uses wrong tap org (`1898co` vs `drbothen`) (HIGH)

**Location:** Lines 125–165.

**Defect:** The `homebrew-update` job checks out `1898co/homebrew-tap`. Per approved decisions, this tap org is mismatched (the repo is under `drbothen`, not `1898co`). The job will fail with a 404 on checkout because `1898co/homebrew-tap` does not exist.

**Fix:** Disable (comment out or remove) the `homebrew-update` job per approved decisions. Homebrew tap requires its own tap org setup; defer until the tap exists under the correct org. Add a comment explaining the deferral and the correct future org.

**RC-1 blocking:** YES.

### DEF-REL-004 — crates.io publish job targets `prism-spec-engine` which now has `publish = false` (CRITICAL)

**Location:** Lines 188–205.

**Defect:** The `crates-io-publish` job runs `cargo publish -p prism-spec-engine --no-verify`. However, `crates/prism-spec-engine/Cargo.toml` now has `publish = false`. Cargo will reject the publish with: `error: publishing is not allowed for 'prism-spec-engine'`. The comment in the job claims this is "the only crate without publish = false" — this is factually incorrect as of the current state of the repository.

**Status of all workspace crates:** All 24 workspace member crates (checked) carry `publish = false` in their `Cargo.toml`. There is currently NO crate intended for crates.io publication.

**Fix:** Remove the `crates-io-publish` job entirely. No crate is ready for crates.io publication. The distribution strategy is GitHub Releases binary + install scripts. If crates.io publication is desired in the future (e.g., `prism-core` as a library), it requires a separate ADR and crate-level decision to remove `publish = false`.

**RC-1 blocking:** YES.

### DEF-REL-005 — Archive step only packages the `prism` binary; DTU demo server is not captured (MEDIUM)

**Location:** Lines 68–79.

**Defect:** The `Create archive` step archives only `target/${{ matrix.target }}/release/prism` (or `prism.exe`). The demo bundle (prism-dtu-demo-server + scripts + demo.toml + preflight tool + sensor specs) is not packaged as a separate release artifact.

**Fix:** The main release archive correctly ships only `prism`. A separate `demo-bundle.sh` script + release.yml step must produce `prism-demo-bundle-${TAG}-${target}.tar.gz` as a SEPARATE release asset. The demo bundle ships per approved decisions as a standalone artifact; it must never be mixed into the main `prism-${TAG}-${target}.tar.gz` binary archive.

**RC-1 blocking:** YES (demo is the RC acceptance gate; the demo bundle must be downloadable from the RC release).

---

## 4. Crate Version Non-Uniformity Analysis

### Current state

| Crate | Version | `publish` |
|-------|---------|-----------|
| prism-bin | 0.1.0 | false |
| prism-spec-engine | 0.9.0 | false |
| prism-core | 0.2.0 | false |
| prism-sensors | 0.2.0 | false |
| ocsf-proto-gen | 0.1.2 | false |
| All others (18 crates) | 0.1.0 | false |

No `[workspace.package]` exists. Crate versions drifted during development due to independent feature work on specific crates.

### Approach: product version = git tag; crate versions stay independent

The production-grade approach (ADR-053 proposed decision):

1. **Product version** = the git tag (`v1.0.0-rc.1`, `v1.0.0`, etc.). This is what users see in `prism version` output, download URLs, install scripts.
2. **prism-bin version** = aligned to the product tag. `prism-bin` is the distribution unit; its `CARGO_PKG_VERSION` is what `prism version` prints. For v1.0.0-rc.1, `prism-bin` version bumps to `1.0.0-rc.1`.
3. **All other crates** retain independent versioning. They are internal libraries (`publish = false`), never externally consumed. No workspace.package needed.
4. **No `[workspace.package]`**: The non-uniform versions reflect genuine independent evolution. Introducing workspace.package would require choosing an arbitrary "winner" version and potentially triggering semver-checks failures for crates that never had a 1.0.0. Deferred unless a crates.io publication story mandates it.

**Version bump scope for v1.0.0-rc.1:** Only `prism-bin/Cargo.toml` version changes from `0.1.0` to `1.0.0-rc.1`. All other crates unchanged.

**Cargo version format note:** `1.0.0-rc.1` is valid semver (pre-release identifier). Cargo accepts it; `cargo semver-checks` will treat it as a pre-release and skip the baseline comparison for the pre-release boundary (correct behavior — semver-checks compares against `origin/develop`, and a pre-release version does not establish a stability contract).

**`prism version` output for RC**: `prism 1.0.0-rc.1`. This satisfies the secops-factory version check contract.

---

## 5. Engine Contract: No Changes Needed

The approved decisions explicitly confirm: no changes to the Prism engine, MCP surface, boot contract (ADR-022), or any crate behavior. Release engineering is entirely in the delivery + packaging layer. Specifically:

- `prism --config-dir <dir> start` MCP stdio launch shape: unchanged
- E-SPEC-024 abort semantics: unchanged
- Config-dir layout (spec_dir, customers/, infusions/): unchanged
- Credential resolution chain: unchanged
- MCP tool surface (query, prism_describe, sensor-health): unchanged

This means ADR-022, ADR-026, ADR-034, ADR-050, and all BC files in `.factory/specs/behavioral-contracts/` require no updates.

---

## 6. e2e.yml Two-Binary Dependency

`e2e.yml` builds both `prism` and `prism-dtu-demo-server`:
```
cargo build --release -p prism-bin -p prism-dtu-demo-server
```

This is correct for CI. The release workflow must also build `prism-dtu-demo-server` (for the demo bundle), but it is a SEPARATE release asset. The separation rule:

- `prism-${TAG}-${target}.tar.gz` — contains ONLY `prism` binary (main release)
- `prism-demo-bundle-${TAG}.tar.gz` — contains `prism-dtu-demo-server` + demo scripts (separate asset, NOT per-platform compiled archive — see §7 for bundle structure)

The demo bundle is assembled on a single platform (Linux musl for maximum portability) OR as a per-platform bundle. The decision: **per-platform demo bundle** matching the same 5-platform matrix, because `prism-dtu-demo-server` is a native binary.

---

## 7. Demo Bundle Structure

**Human decision (2026-07-19, OQ-1 resolved):** The demo bundle MUST include pre-built `.prx` plugin artifacts. The bundle must be Rust-toolchain-free — a consumer can run the demo without a Rust toolchain or `cargo`.

The DTU demo bundle release asset (`prism-demo-bundle-${TAG}-${target}.tar.gz`) must contain:

```
prism-demo-bundle-${TAG}-${target}/
  prism-dtu-demo-server          # native binary (or .exe on Windows)
  plugins/
    crowdstrike-oauth2.prx       # pre-built wasm32-wasip1 Component
    threatintel-lookup.prx       # pre-built wasm32-wasip1 Component
    threatintel-lookup.manifest.toml
  scripts/
    demo-setup.sh
    demo-run.sh
    demo-teardown.sh
    demo-setup.ps1               # Windows PowerShell parity (OQ-2 resolved)
    demo-run.ps1
    demo-teardown.ps1
    demo.toml
  specs/
    crowdstrike.sensor.toml      # from crates/prism-sensors/specs/
    armis.sensor.toml
    claroty.sensor.toml
    cyberint.sensor.toml
  infusions/
    threatintel.infusion.toml
    nvd.infusion.toml
  preflight/
    t13-preflight-audit.py       # shipped as-is (OQ-3 resolved — no simplified variant)
  DEMO-RUNBOOK.md                # from docs/DEMO-RUNBOOK.md
```

The demo bundle does NOT include:
- `crowdstrike-oauth2.manifest.toml` (the DTU-safe manifest is dynamically generated by demo-setup.sh/ps1 because it extends the production allowed_urls with "127.0.0.1"; it cannot be pre-baked without hardcoding localhost semantics)
- Source code

### Plugin build recipes for release.yml

The two confirmed demo plugins and their Justfile recipes:

| Plugin artifact | Justfile recipe | Source |
|----------------|----------------|--------|
| `crowdstrike-oauth2.prx` | `build-plugin-crowdstrike-oauth2` | `crates/prism-spec-engine/plugins/crowdstrike-oauth2/` |
| `threatintel-lookup.prx` | `build-plugin-threatintel-infusion` | `crates/plugins/prism-threatintel-infusion/` |

**Naming flag (needs human confirmation):** The approved decision listed "ocsf-complex-transforms" as one of the two demo plugins. Verification shows:
- No `build-plugin-ocsf-complex-transforms` Justfile recipe exists
- `crates/prism-spec-engine/plugins/` contains only `crowdstrike-oauth2/` and `threatintel-lookup/`
- `demo-setup.sh` references only `crowdstrike-oauth2.prx` and `threatintel-lookup.prx`
- `ocsf-complex-transforms` source exists at `crates/plugins/ocsf-complex-transforms/` but has no build recipe and is not used by the demo

S-REL-004 is scoped to the two confirmed demo plugins. If `ocsf-complex-transforms` was intended, it requires a new Justfile recipe and a demo-setup.sh reference first — that is a separate story.

### Plugin build requirements for release.yml

The WASM `.prx` files are architecture-independent bytecode and must be built in a single dedicated `build-plugins` job (not inside the 5-platform matrix). The `build-plugins` job:
1. Installs the `wasm32-wasip1` target: `rustup target add wasm32-wasip1`
2. Installs `wasm-tools` at the pinned version (1.248.0): `cargo install wasm-tools --version 1.248.0`
3. Runs `just build-plugin-crowdstrike-oauth2`
4. Runs `just build-plugin-threatintel-infusion`
5. Uploads both `.prx` files as a single `plugin-artifacts` workflow artifact

**CI ordering (U19 adjudication):** `build-plugins` runs IN PARALLEL with the `build-release` matrix — both jobs are triggered by the same gate (e.g., `needs: [check]`). The `.prx` files are consumed only by `build-demo-bundle`, NOT by `build-release`. There is no ordering dependency between `build-plugins` and `build-release`. Both feed `build-demo-bundle`.

### Demo server binary: CI artifact passing (U13 adjudication)

`build-demo-bundle` downloads `prism-dtu-demo-server` as a per-platform artifact from `build-release`. It does NOT rebuild the binary. The `build-release` matrix must:

1. Build BOTH binaries in one invocation (same pattern as e2e.yml):
   ```
   cargo build --release -p prism-bin -p prism-dtu-demo-server
   ```
2. Tar-wrap `prism-dtu-demo-server` (or `.exe` on Windows) BEFORE `upload-artifact` to preserve Unix +x permissions through the artifact boundary:
   ```bash
   tar czf prism-dtu-demo-server-${TARGET}.tar.gz \
     target/${TARGET}/release/prism-dtu-demo-server   # or .exe on Windows
   ```
3. Upload as a separate workflow artifact named `prism-dtu-demo-server-${TARGET}` (job-to-job only — NOT a public release asset).
4. `build-demo-bundle` downloads `prism-dtu-demo-server-${TARGET}`, untars it, and includes the binary in the demo bundle archive.

The main `prism-${TARGET}.tar.gz` release archive continues to contain ONLY the `prism` binary.

### CI job dependency chain (U15 + U19 adjudication)

```
check ─────┬──→ build-plugins (WASM only, single job) ────────────────────┐
           │                                                                │
           └──→ build-release (5-platform matrix) ──→ publish-release ────┤
                  (builds prism + dtu-server per-platform)                 │
                                                                           ↓
                                                                  build-demo-bundle
                                                                  needs: [build-release,
                                                                          build-plugins,
                                                                          publish-release]

publish-release: creates GitHub Release + uploads prism binaries + install.sh/install.ps1
build-demo-bundle: assembles + uploads per-platform demo bundle archives
```

`build-demo-bundle` must declare `needs: [build-release, build-plugins, publish-release]`. Omitting `publish-release` causes `gh release upload` to race `gh release create` and fail non-deterministically (U15).

### Per-platform archive format (U22 adjudication)

Demo bundle archives use the same format convention as the main binary release — consistent with platform expectations and PowerShell extraction tooling:

| Platform | Target | Archive Format | Example filename |
|----------|--------|----------------|-----------------|
| Linux (glibc) | x86_64-unknown-linux-gnu | `.tar.gz` | `prism-demo-bundle-v1.0.0-rc.1-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (musl) | x86_64-unknown-linux-musl | `.tar.gz` | `prism-demo-bundle-v1.0.0-rc.1-x86_64-unknown-linux-musl.tar.gz` |
| macOS (ARM) | aarch64-apple-darwin | `.tar.gz` | `prism-demo-bundle-v1.0.0-rc.1-aarch64-apple-darwin.tar.gz` |
| macOS (x86_64) | x86_64-apple-darwin | `.tar.gz` | `prism-demo-bundle-v1.0.0-rc.1-x86_64-apple-darwin.tar.gz` |
| Windows | x86_64-pc-windows-msvc | `.zip` | `prism-demo-bundle-v1.0.0-rc.1-x86_64-pc-windows-msvc.zip` |

S-REL-007 AC-011/012 must assert `.zip` (not `.tar.gz`) for the Windows bundle. `demo-setup.ps1` must use `Expand-Archive` for extraction, not `tar`. The archive includes `demo-setup.ps1`, `demo-run.ps1`, and `demo-teardown.ps1` (no `.sh` files in the Windows `.zip`).

---

## 8. Regression Risk Assessment

| Area | Risk | Mitigation |
|------|------|-----------|
| **Linux release.yml build legs (never executed — U2)** | **HIGH** — `musl-tools`, `libdbus-1-dev`, `pkg-config` are present in ci.yml but absent from release.yml; `wasm32-wasip1` target add is also missing; first real tag push silently fails both Linux matrix legs | S-REL-001 scope expanded: add `sudo apt-get install -y musl-tools libdbus-1-dev pkg-config` step (Linux-conditional on `runner.os == 'Linux'`); add `rustup target add x86_64-unknown-linux-musl`; add fork-tag dry-run AC (`v0.0.0-dry-run`) to verify all 5 platform builds succeed before cutting v1.0.0-rc.1 |
| release.yml changes (removing dead jobs) | LOW — dead jobs already fail; removing them can only improve CI | Gate: fork-tag dry-run (added to S-REL-001 ACs) validates the repaired workflow before v1.0.0-rc.1 |
| prism-bin version bump (0.1.0 → 1.0.0-rc.1) | LOW — only affects `prism version` output; no semver-checks baseline for pre-release | Verify `just check` passes; verify `prism version` prints new string |
| install.sh / install.ps1 (new files) | MEDIUM — new complexity; checksum verification must be correct; uploaded as release assets (not raw.githubusercontent.com) per U26 adjudication | Test on all 5 platforms; include in RC acceptance gate; upload step in publish-release job |
| demo-bundle packaging with .prx prebuilds | MEDIUM — wasm-tools install on CI runner; WASM Component build is deterministic but build toolchain pin matters | Pin wasm-tools 1.248.0 per existing Justfile comment; run plugin build in dedicated `build-plugins` job parallel with build-release matrix |
| demo-bundle CI job dependency chain | MEDIUM — `build-demo-bundle` races `publish-release` if dependency is wrong | U15 fix: `build-demo-bundle` declares `needs: [build-release, build-plugins, publish-release]`; enforced in S-REL-004 ACs |
| PowerShell demo scripts (Windows) | MEDIUM — Windows subprocess mgmt, stdin piping, keyring behavior, and `.zip` extraction (not `.tar.gz`) need validation | See §10 Windows keyring analysis; Windows demo bundle uses `.zip` (U22); test under Windows runner in release.yml |
| lefthook pre-tag hook | NONE — already correct; semver-checks + audit + deny all pass before tagging | No change |
| e2e.yml | NONE — no changes to e2e.yml | Unchanged |

---

## 9. Open Questions — Resolved and Remaining

All five OQs from the original analysis are resolved per human adjudication on 2026-07-19:

### OQ-1: Plugin `.prx` artifacts in demo bundle — RESOLVED: INCLUDE

Include pre-built `.prx` artifacts in the demo bundle so it is Rust-toolchain-free. Folded into S-REL-004. The two demo plugins are `crowdstrike-oauth2.prx` and `threatintel-lookup.prx`. The "ocsf-complex-transforms" name mentioned in the human decision does not match any Justfile recipe or demo-setup.sh reference — flagged in §7 for human confirmation before adding to scope.

### OQ-2: Windows demo scripts — RESOLVED: FULL POWERSHELL PARITY NOW

`demo-setup.ps1`, `demo-run.ps1`, `demo-teardown.ps1` are RC-1 blocking. S-REL-007 moves from Wave F-C to Wave F-A with RC-1 blocking status. See §10 for Windows keyring analysis and sizing impact. This was originally sized L; it remains L and is now Wave F-A blocking.

### OQ-3: `t13-preflight-audit.py` — RESOLVED: SHIP AS-IS

Ship `t13-preflight-audit.py` as-is in the demo bundle. No simplified variant story. The full 106-check audit script ships to consumers without modification.

### OQ-4: `prism --version` canonical form — RESOLVED: `prism --version`

`prism --version` is the canonical version check form. Pinned in the consumer contract (§5.2). Both `prism --version` and `prism version` print identical output; `prism --version` is the canonical form for programmatic consumers.

### B5: Demo Jira project — RESOLVED: ticket-first via demo Jira project

Demo intake is ticket-first via `jr` against a demo Jira project. Full design details in secops-factory-handoff-brief.md §4. The monitor loop remains a full feature (patrol, correlation-first, watermarks, Perplexity+Tavily) but is NOT the demo intake path.

### Remaining open item: ocsf-complex-transforms naming

Requires human confirmation: was "ocsf-complex-transforms" intended to refer to `crowdstrike-oauth2` (the confirmed demo plugin), or is it a new requirement? S-REL-004 currently scoped to the two confirmed demo plugins only.

---

## 10. Windows Keyring Analysis (S-REL-007 sizing input)

### Finding: No Windows Credential Manager interactive-auth issue

The prism-credentials crate uses `keyring = 3.x` with `keyring-windows-native` feature, which maps to Windows Credential Manager (Credential Vault) via `windows-sys`. Key characteristics:

- **No interactive prompt:** Windows Credential Manager entries for the current user are accessible immediately in an interactive desktop session. Unlike macOS Keychain (which may prompt for unlock with a dialog), Windows Credential Manager for the current user is session-transparent — no UAC elevation, no dialog.
- **Non-interactive / service contexts:** In headless or service contexts, Credential Manager access may be restricted by domain policy. The `PRISM_CLIENTS_*` env var fallback (Tier 2) covers this case. The same fallback pattern used in demo-setup.sh must be replicated in demo-setup.ps1.
- **Stdin piping in PowerShell:** PowerShell supports `"<value>" | prism credential set ...` via the pipeline operator. The `rpassword` crate reads from piped stdin in non-TTY mode. This works correctly in PowerShell.

**Conclusion for demo-setup.ps1:** The Windows Credential Manager path works for interactive desktop demos (the primary demo scenario) without any special handling. The `.ps1` must replicate the fallback pattern from `.sh`:
```powershell
$result = "demo-cs-client-id-org-a" | prism --config-dir $DemoConfigDir credential set `
  --sensor crowdstrike --name client_id --org-slug org-a 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Warning "Keyring write failed. Fallback: set env var PRISM_CLIENTS_ORG_A_SENSORS_CROWDSTRIKE_CLIENT_ID"
}
```

**No additional Windows-specific credential bootstrap story needed.** The existing fallback pattern is sufficient. S-REL-007 sizing includes this credential handling as part of the PowerShell parity work.

### Windows demo-run.ps1 complexity: ephemeral port + Python3 dependency

`demo-run.sh` uses `python3` for JSON parsing of the nested URLs sidecar. On Windows, Python is not guaranteed to be available. Options:
1. Use `ConvertFrom-Json` (PowerShell built-in) instead of Python for JSON parsing — eliminates the Python3 dependency on Windows entirely.
2. Require Python3 as a demo prerequisite on Windows.

Recommended: use `ConvertFrom-Json` in `demo-run.ps1`. This is pure PowerShell, no external dependency. The overlay TOML generation logic (currently Python in demo-run.sh) is translated to PowerShell string formatting.

This is the primary complexity driver for the L sizing of S-REL-007.

---

## 11. Proposed Story Breakdown (F2/F3 Input) — Updated

Stories are listed in dependency order. RC-1 blocking status is explicit.

**Amendment 2026-07-19:** S-REL-007 (PowerShell parity) promoted from Wave F-C to Wave F-A as RC-1 blocking per human decision.

### Wave F-A: Release Infrastructure (all RC-1 blocking)

| Story ID | Title | Size | RC-1 Blocking | Dependencies |
|----------|-------|------|---------------|-------------|
| S-REL-001 | release.yml repair — remove dead jobs (DEF-REL-001 through DEF-REL-004), fix binary_exists guard; add Linux apt setup (musl-tools/libdbus-1-dev/pkg-config, Linux-conditional); add dual-binary build (-p prism-bin -p prism-dtu-demo-server); add tar-wrap + upload-artifact step for demo-server binary per-platform; add install.sh/install.ps1 upload step to publish-release; fork-tag dry-run AC | M | YES | None |
| S-REL-002 | Version alignment — bump prism-bin to 1.0.0-rc.1; ADR-053 | S | YES | None (parallel with S-REL-001) |
| S-REL-003 | install.sh + install.ps1 — checksum-verified install scripts (5-platform) | M | YES | S-REL-001 (needs release URL pattern) |
| S-REL-004 | demo-bundle packaging — build-plugins CI job + per-platform demo bundle archive (with .prx prebuilds, all scripts, t13-preflight-audit.py); release.yml integration | L | YES | S-REL-001, S-REL-002 |
| S-REL-007 | Windows PowerShell demo script parity — demo-setup.ps1, demo-run.ps1, demo-teardown.ps1; ConvertFrom-Json sidecar parsing; Windows Credential Manager fallback handling | L | YES (promoted from F-C) | S-REL-004 (demo bundle structure) |
| S-REL-005 | RELEASING.md — operator runbook for cutting v1.0.0-rc.1 and v1.0.0 | S | YES | S-REL-001, S-REL-002, S-REL-003, S-REL-004, S-REL-007 |

### Wave F-B: Consumer Contract (RC-1 blocking)

| Story ID | Title | Size | RC-1 Blocking | Dependencies |
|----------|-------|------|---------------|-------------|
| S-REL-006 | prism-consumer-contract.md — graduate draft to docs/consumer-contract.md + DEMO-RUNBOOK.md update for Windows parity | S | YES (secops-factory needs it) | S-REL-002 (version contract), S-REL-007 (Windows notes) |

### Wave F-C: Post-RC (NOT RC-1 blocking)

| Story ID | Title | Size | RC-1 Blocking | Dependencies |
|----------|-------|------|---------------|-------------|
| S-REL-008 | Homebrew tap — create tap under correct org, re-enable release.yml job | M | NO | Human: tap org decision |
| S-REL-009 | v1.0.0 final release — soak completion, main fast-forward | S | NO | RC acceptance gate + soak |

### Story Size Definitions

- S: ~1 day (single focused change)
- M: ~2-3 days (multiple related changes with tests)
- L: ~1 week (substantial new functionality with multiple components)

**Sizing note:** S-REL-004 is promoted from M to L due to the addition of the plugin build job (wasm-tools install, two Justfile recipe invocations, WASM artifact upload/download across job boundary) and full demo bundle manifest. S-REL-007 was already sized L.

---

## 12. Dependency Graph (Updated)

### Story dependency graph

```
S-REL-001 ────────┬──→ S-REL-003 ──────────────────────────→ S-REL-005
                  │                                              ↑
S-REL-002 ────────┤──→ S-REL-004 ──→ S-REL-007 ──────────────→ ┤
                  │                                              │
                  └──→ S-REL-006 ←── (S-REL-002 + S-REL-007) ──┘

S-REL-005 → [RC acceptance gate] → v1.0.0-rc.1 tag

[v1.0.0-rc.1 soak] → S-REL-009
```

S-REL-001 and S-REL-002 are parallel (different files). S-REL-003 depends on S-REL-001 for release URL patterns. S-REL-004 depends on both for the build system and version string. S-REL-007 depends on S-REL-004 for the demo bundle structure. S-REL-006 waits for S-REL-007 to capture Windows parity in the contract doc.

### CI job dependency graph (release.yml post-repair)

Adjudication outcomes U13/U15/U19 result in this job topology:

```
check ─────┬──→ build-plugins  (WASM only, single job, parallel) ──────────────┐
           │    needs: [check]                                                   │
           │                                                                     │
           └──→ build-release  (5-platform matrix, parallel) ───→ publish-release ──→ build-demo-bundle
                needs: [check]     builds prism + dtu-server          needs:              needs:
                                   tar-wraps dtu-server per-platform  [build-release]     [build-release,
                                   uploads artifact per-platform                           build-plugins,
                                                                                           publish-release]
```

- `build-plugins` ∥ `build-release` — no ordering dependency between them (U19)
- `publish-release` creates the GitHub Release and uploads prism binaries + install.sh/install.ps1
- `build-demo-bundle` must list all three predecessors in `needs` to prevent the `gh release upload` race (U15)
- `build-release` builds both `prism` and `prism-dtu-demo-server` in one cargo invocation; the demo-server binary flows to `build-demo-bundle` via workflow artifact (not rebuilt) (U13)

---

## 13. Pre-TDD Adjudication Record — S-REL-001 v0.2

**Adjudicated by:** architect  
**Date:** 2026-07-19  
**Story version adjudicated:** S-REL-001 v0.2  
**Context:** Mandatory pre-TDD uncertainty scan surfaced four findings. Story-writer applies all rulings in a v0.3 revision. This record is the authoritative source; the v0.3 story supersedes v0.2 for implementation scope per CLAUDE.md Source-of-Truth Precedence §1.

---

### ADJ-001 (Critical): libdbus-1-dev required unconditionally on both Linux legs

**Finding severity:** Critical  
**Finding source:** Pre-TDD scan contradicting U2 research conclusion.

#### Evidence

```
cargo tree --target x86_64-unknown-linux-gnu -p prism-credentials -i libdbus-sys
→ libdbus-sys v0.2.7

cargo tree --target x86_64-unknown-linux-musl -p prism-credentials -i libdbus-sys
→ libdbus-sys v0.2.7
```

`prism-credentials` `[features] default` includes `keyring-linux-native-sync-persistent = ["keyring/linux-native-sync-persistent", "keyring/crypto-rust"]`, which activates `dbus-secret-service 4.1.0 → dbus 0.9.11 → libdbus-sys 0.2.7` (C-linked). The `[features]` section does not branch on target triple — both Linux targets compile `dbus-secret-service`. The runtime distinction (musl: kernel-keyutils only; gnu: dbus Secret Service) is enforced in keyring's runtime logic, not at compile-time feature selection. Therefore libdbus-sys is compiled for BOTH Linux targets.

`ci.yml` installs `libdbus-1-dev pkg-config` on every Linux job (7 occurrences verified) with comment: "build.rs runs on the glibc host even for musl cross-targets. Both linux-gnu and linux-musl legs therefore need libdbus-1-dev + pkg-config on the host. ADR-034 / BC-2.06.003."

The U2 research conclusion "libdbus-1-dev NOT needed (keyring v3 uses pure-Rust zbus)" was a general finding about keyring v3 in the abstract. It does NOT apply to this workspace: prism-credentials uses the C-linked `dbus-secret-service` backend (not zbus). The research finding was correct in the abstract but inapplicable given prism's specific Cargo.toml feature configuration.

The S-REL-001 v0.2 verification probe `grep -n 'libdbus-1-dev' Cargo.lock` is structurally invalid: apt package names never appear in Cargo.lock. The correct probe is `cargo tree --target <triple> -i libdbus-sys`.

#### Ruling

`libdbus-1-dev` MUST be installed unconditionally alongside `musl-tools` and `pkg-config` on both Linux legs (`x86_64-unknown-linux-gnu` and `x86_64-unknown-linux-musl`), citing ADR-034/BC-2.06.003. The EC-006 "if keyring uses zbus" hedge is withdrawn — it does not use zbus. The broken Cargo.lock probe is replaced.

#### S-REL-001 v0.3 deltas (story-writer applies)

1. **`risk_mitigations` entry for "Linux cross-compile setup (U2)"** — replace the entire "libdbus-1-dev is INCONCLUSIVE..." clause with:
   > `libdbus-1-dev REQUIRED unconditionally on both Linux legs: prism-credentials default features enable keyring-linux-native-sync-persistent → dbus-secret-service 4.1.0 → dbus 0.9.11 → libdbus-sys 0.2.7 (C-linked). build.rs runs on the glibc host even for musl cross-targets. Verified: cargo tree --target x86_64-unknown-linux-{gnu,musl} -i libdbus-sys shows libdbus-sys v0.2.7. ci.yml installs libdbus-1-dev pkg-config unconditionally on every Linux job (ADR-034/BC-2.06.003). Do NOT add libssl-dev (ADR-050 mandates rustls-tls).`

2. **task 7**, "Install Linux build deps" step:
   - Remove the `# NOTE: libdbus-1-dev: VERIFY against Cargo.lock keyring backend before adding...` comment block entirely.
   - Change the apt-get install line to: `sudo apt-get install -y musl-tools pkg-config libdbus-1-dev`
   - Add a comment directly above the apt-get line:
     `# libdbus-1-dev: required by prism-credentials keyring-linux-native-sync-persistent → dbus-secret-service (C-linked libdbus). build.rs runs on glibc host even for musl cross-target. ADR-034/BC-2.06.003.`

3. **AC-010** "Then" clause — replace:
   > `libdbus-1-dev presence is justified by a comment citing the verified keyring crate backend.`

   with:
   > `` `libdbus-1-dev` is installed unconditionally alongside `musl-tools` and `pkg-config` on both Linux legs, with a comment citing ADR-034/BC-2.06.003 and the build.rs host-linkage rationale. Secondary probe: `cargo tree --target x86_64-unknown-linux-gnu -i libdbus-sys` and `cargo tree --target x86_64-unknown-linux-musl -i libdbus-sys` both return `libdbus-sys vX.Y.Z`. ``

4. **EC-006** — replace the row entirely:

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-006 | musl target build (libdbus-sys C-linked at build time) | `libdbus-1-dev` installed on host; build succeeds. musl binary does NOT dynamically link libdbus at runtime (kernel-keyutils path only). Build fails if `libdbus-1-dev` is absent from runner. |

---

### ADJ-002 (Important): install.sh/install.ps1 upload step moves to S-REL-003

**Finding severity:** Important (fork-tag dry-run gate blocker)  
**Finding source:** Pre-TDD scan detecting nonexistent file reference.

#### Reasoning

`scripts/install.sh` and `scripts/install.ps1` do not exist on develop. They are authored by S-REL-003. S-REL-001 blocks S-REL-003 (genuine dependency: S-REL-003 needs S-REL-001's release URL pattern to write install.sh). Therefore at S-REL-001 implementation time, the install scripts will not exist.

`gh release upload` hard-fails on nonexistent paths — no partial success. S-REL-001 task 13 (fork-tag dry-run) cannot go green with task 12 / AC-011 implemented as written: the files do not yet exist when S-REL-001 runs.

Option analysis:
- **(b) Re-adjudicate the dependency edge** — rejected. S-REL-003 genuinely needs S-REL-001's release URL pattern. Dropping the S-REL-001 → S-REL-003 dependency would mean S-REL-003 authors install.sh before the URL pattern is established. This is a real ordering constraint.
- **(c) Placeholder glob logic** — rejected per canonical principle (defer-pattern smell; no "for now" placeholders).
- **(a) Move upload step to S-REL-003** — adopted. The story that authors a file also owns the CI step that uploads it. This is correct-agent-pattern: S-REL-003 writes the files AND wires the upload. The U26 adjudication is preserved (install scripts ARE uploaded as release assets by publish-release); only the implementing story changes.

#### Ruling

Task 12 and AC-011 are removed from S-REL-001. S-REL-003 inherits ownership of both authoring the install scripts and amending the publish-release job in release.yml to upload them. The S-REL-001 → S-REL-003 dependency edge is unchanged. The CI job topology in §12 is unaffected (publish-release still uploads install.sh/install.ps1 at release time; it just takes effect after S-REL-003 merges).

#### S-REL-001 v0.3 deltas (story-writer applies)

1. Delete **task 12** ("Upload install.sh and install.ps1 as release assets (U26)") entirely.
2. Delete **AC-011** ("install.sh and install.ps1 uploaded as release assets") entirely.
3. Decrement **`acceptance_criteria_count`** frontmatter field: `12` → `11`.
4. Rename **AC-012** → **AC-011** (the actionlint AC; only a renumbering, content unchanged).
5. **Behavioral Contracts table**, "Architect U26 adjudication" row — update text:
   > `install.sh/.ps1 uploaded as release assets by publish-release — upload step implemented in S-REL-003 (which authors the files; upload moved per pre-TDD scan ADJ-002; files do not exist at S-REL-001 implementation time)`
6. **`risk_mitigations`**, entry for "install.sh and install.ps1 uploaded as release assets by publish-release job (U26)" — replace with:
   > `install.sh and install.ps1 are authored by S-REL-003. S-REL-003 also amends the publish-release job in release.yml to add the gh release upload step for those scripts. S-REL-001 does NOT implement the upload — it establishes the release URL pattern that S-REL-003 consumes. (U26 upload step ownership reassigned to S-REL-003 per pre-TDD scan ADJ-002.)`

#### S-REL-003 additions (story-writer applies when authoring S-REL-003)

Add the following to S-REL-003 (in addition to its existing scope of authoring the scripts):

1. **New task**: After the task that authors `scripts/install.ps1`, add:
   > Amend the `publish-release` job in `.github/workflows/release.yml` to include `scripts/install.sh` and `scripts/install.ps1` in the `gh release create` invocation or as a subsequent `gh release upload` step within publish-release. Pass the release version to install.ps1 consumers via `$env:PRISM_INSTALL_VERSION` env var before the `irm | iex` pipe — positional args cannot be carried through `iex` (U8).

2. **New AC**: Add to S-REL-003 acceptance criteria:
   > The `publish-release` job in the modified `.github/workflows/release.yml` includes `scripts/install.sh` and `scripts/install.ps1` in the upload invocation. Verification: `grep -n 'install\.sh\|install\.ps1' .github/workflows/release.yml` returns at least one match inside the publish-release job context.

3. **`depends_on`** of S-REL-003 must list `S-REL-001` (already implied by existing graph; make explicit in frontmatter if not present).

---

### ADJ-003 (Medium): Windows demo-server wrap requires .exe + 7z; all 5 targets

**Finding severity:** Medium  
**Finding source:** Pre-TDD scan detecting platform-specific binary naming gap.

#### Reasoning

`tar czf ... prism-dtu-demo-server` fails on `x86_64-pc-windows-msvc`: the Windows binary is `prism-dtu-demo-server.exe`. The existing main binary archive step already handles this correctly via `archive_ext` matrix variable and `if [ "${{ matrix.archive_ext }}" = "zip" ]` conditional — the same pattern must govern the demo-server wrap. The +x preservation rationale applies only to Unix legs; Windows has no Unix executable bit concept.

Strip handling: the existing story strips only `prism`. Both binaries should be stripped on Unix legs under the production-grade default.

Target coverage: all 5 matrix targets build the demo-server (delta-analysis §7: demo bundle is per-platform for all 5 targets).

#### Ruling

Task 7 must use per-OS conditional logic for the demo-server wrap, keyed on the existing `archive_ext` matrix variable. Unix legs: `tar czf` (preserves +x). Windows: `7z a ... prism-dtu-demo-server.exe`. Strip covers both binaries on Unix legs. Artifact upload uses `${{ matrix.archive_ext }}` in the filename. No target exclusion.

#### S-REL-001 v0.3 deltas (story-writer applies)

1. **task 7**, demo-server wrap step — replace the single tar command with the following two-step block:

   ```yaml
   - name: Strip demo-server (Unix)
     if: runner.os != 'Windows'
     run: strip target/${{ matrix.target }}/release/prism-dtu-demo-server 2>/dev/null || true

   - name: Wrap prism-dtu-demo-server for artifact upload
     shell: bash
     run: |
       if [ "${{ matrix.archive_ext }}" = "zip" ]; then
         cd target/${{ matrix.target }}/release
         7z a "../../../prism-dtu-demo-server-${{ matrix.target }}.zip" prism-dtu-demo-server.exe
       else
         tar czf prism-dtu-demo-server-${{ matrix.target }}.tar.gz \
           -C target/${{ matrix.target }}/release prism-dtu-demo-server
       fi
   ```

   Upload artifact step for demo-server (job-to-job only — not a public release asset):
   ```yaml
   - name: Upload demo-server artifact (job-to-job)
     uses: actions/upload-artifact@...  # same pin as main artifact step
     with:
       name: prism-dtu-demo-server-${{ matrix.target }}
       path: prism-dtu-demo-server-${{ matrix.target }}.${{ matrix.archive_ext }}
   ```

   `build-demo-bundle` (S-REL-004) downloads this artifact and extracts based on `archive_ext` — that extraction logic is S-REL-004's scope, not S-REL-001's.

2. **AC-009** "Then" clause — replace:
   > `prism-dtu-demo-server binary is tar-wrapped before upload (to preserve +x bit).`

   with:
   > `prism-dtu-demo-server binary is wrapped before upload-artifact using per-OS conditional logic: `.tar.gz` (tar, preserves +x bit) on Unix legs; `.zip` (7z, `.exe` suffix) on Windows. Uploaded as artifact `prism-dtu-demo-server-${{ matrix.target }}` with path `prism-dtu-demo-server-${{ matrix.target }}.${{ matrix.archive_ext }}`. All 5 matrix targets produce this artifact. prism-dtu-demo-server is stripped on Unix legs (alongside `prism`).`

3. **`risk_mitigations`** entry for "build-release builds prism-bin + prism-dtu-demo-server together (U13)" — append:
   > The demo-server wrap step uses per-OS conditional logic matching the `archive_ext` matrix variable (same pattern as the main `prism` archive step): `tar czf` on Unix (preserves +x); `7z a ... .exe` on Windows. Strip applies to both `prism` and `prism-dtu-demo-server` on Unix legs.

---

### ADJ-004 (Low — note only): Stale crate count "24" in task 6

**No formal adjudication required.** The Cargo workspace currently has 27 member entries (verified from root `Cargo.toml` `members` array). The CLAUDE.md header cites a different count from a prior session. Hardcoded counts in inline comments are a maintenance liability — they drift silently as crates are added.

**S-REL-001 v0.3 delta (story-writer applies):** In **task 6**, change the comment text from:
> `all 24 workspace crates carry publish = false`

to:
> `all workspace crates carry publish = false`

No number. The sentence remains true regardless of future crate additions.
