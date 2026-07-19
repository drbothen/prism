---
document_type: research
producer: research-agent
date: 2026-07-19
topic: S-REL release-engineering uncertainty resolution (CI/CD, install scripts, cross-platform packaging)
status: complete
research_type: general
mcp_tool_calls: 13
training_data_reliance: low
related_stories: S-REL-* (release-engineering wave)
---

# Release-Engineering Uncertainty Resolution (2026)

This artifact resolves the external-research questions raised in the S-REL uncertainty scan.
Each section maps to a numbered uncertainty ID and carries a **FINDING**, a **CONFIDENCE**
rating, and a **PROPOSED STORY FIX**. All version/behaviour claims are dated (technology
landscape as of **2026-07-19**) and sourced. Where sources conflict or evidence is thin, the
item is flagged **INCONCLUSIVE** and the residual verification step is named.

> Verification bias: version numbers were checked against live registries (crates.io API,
> GitHub releases, runner-images repo) rather than model training data. Items resting on
> architectural inference rather than an explicit vendor statement are marked accordingly.

---

## U3 — `gh release create` prerelease auto-detection, gh version, conditional-flag bash pattern

**FINDING**
- `gh release create v1.0.0-rc.1` does **NOT** auto-mark the release as a prerelease. The gh CLI
  does not inspect the tag for SemVer prerelease identifiers (`-rc.1`, `-alpha`, `-beta`). You
  **must** pass `--prerelease` explicitly, otherwise GitHub creates a normal (latest) release.
  (Contrast: the GitHub *web UI* offers a "set as prerelease" checkbox; the CLI has no
  auto-heuristic.)
- gh version on `ubuntu-latest`: **2.96.0**, per the Ubuntu 24.04 runner image `20260714.240.1`
  (dated 2026-07-14). `ubuntu-latest` currently aliases Ubuntu 24.04 (see U5).
- Conditional optional-flag bash pattern (avoids an empty positional arg): build a bash **array**,
  never pass a quoted-empty variable. `gh release create "$TAG" "$EMPTY"` would send an empty
  positional arg; the array form does not:
  ```bash
  args=()
  if [ "${{ steps.meta.outputs.is_prerelease }}" = "true" ]; then
    args+=(--prerelease)
  fi
  gh release create "$TAG" "${args[@]}" ./dist/*
  ```
  Equivalent single-line idiom using parameter expansion also works because unquoted empty
  expansion is dropped by word-splitting: `gh release create "$TAG" ${PRERELEASE_FLAG:+--prerelease}`.

**CONFIDENCE** High. gh prerelease behaviour and the 60-req/hr context are documented in the gh
manual and REST docs; gh 2.96.0 is verified against the runner-images `Ubuntu2404-Readme.md`
snapshot dated 2026-07-14. The bash array pattern is standard POSIX/bash idiom.

**PROPOSED STORY FIX** In the release workflow, derive `is_prerelease` from the tag
(`[[ "$TAG" == *-* ]]`) and pass `--prerelease` via the array pattern above. Do not rely on any
gh auto-detection. If a specific gh feature is needed beyond 2.96.0, add an explicit
`gh` install/pin step rather than trusting the runner default.

---

## U5 + U12 + U14 — GitHub Actions latest majors, upload/download compat, runner retirements, `*-latest` resolution

**FINDING** (verified against GitHub Marketplace, action release pages, and `actions/runner-images`)
- **actions/checkout** — latest major is **v6**; `v6.0.2` is real and in active use.
  (Story's assumption of v6.0.2 is valid; pin `@v6` or a SHA.)
- **actions/upload-artifact** — latest major is **v7**; `v7.0.1` confirmed (adopted by Apache/PHP).
- **actions/download-artifact** — latest major is **v8**; `v8.0.1` confirmed.
- **actions/attest-build-provenance** — latest is **v4.1.1**, NOT v4.1.0. The story's assumed
  `v4.1.0` is **stale** (superseded by v4.1.1). As of v4 the action is a thin wrapper over
  `actions/attest` and supports multi-subject attestation via `subject-path`.
- **upload v7 ↔ download v8 same-run compatibility**: compatible. The artifact blob/metadata is
  server-side (GitHub artifact service), and the actions are just clients; no format break is
  documented between these majors. (This rests on architectural reasoning + absence of any
  contrary vendor statement, not an explicit "v7↔v8 compatible" line — see INCONCLUSIVE note.)
- **macos-13**: **RETIRED 2025-12-04** (brownouts Nov 2025). Labels `macos-13`, `-large`,
  `-xlarge` fail outright. Do not reference.
- **macos-15-intel**: available; it is the **last x86_64 macOS image**, supported only **until
  August 2027**. Intel macOS on hosted runners ends when macOS 15 retires (fall 2027).
- **ubuntu-latest** → **Ubuntu 24.04 LTS** (migration completed Oct 2024). Verified.
- **macos-latest** → **macos-26** (migration completed ~2026-07-15; previously macOS 15). Note:
  `macos-latest` is **arm64**; Intel requires an explicit `macos-15-intel` / `-large` label.
- **windows-latest** → **Windows Server 2025** (default rollout Sept 2025). Verified-moderate:
  the WS2025 GA + default-label transition is documented; a mid-2026 doc line pinning the exact
  current mapping was not surfaced, so treat as high-probability rather than quoted-current.

**CONFIDENCE** High for checkout/upload/download majors, attest-build-provenance v4.1.1,
macos-13 retirement, macos-15-intel EOL, and ubuntu-latest=24.04. Medium for the v7↔v8 same-run
compatibility (inference) and windows-latest exact current mapping.

**INCONCLUSIVE (flagged)**: (a) v7↔v8 same-run interop has no explicit vendor compatibility
statement — verify with a one-job smoke test (upload@v7 then download@v8) at story time.
(b) windows-latest exact OS as-of-today — confirm via the runner's "Set up job" log line.

**PROPOSED STORY FIX** Update the workflow pins to: `checkout@v6`, `upload-artifact@v7`,
`download-artifact@v8`, `attest-build-provenance@v4.1.1` (**correct the stale v4.1.0**). Drop any
`macos-13`. For Intel macOS builds use `macos-15-intel` and add a comment noting the **Aug-2027
EOL** with a migration follow-up. Prefer explicit OS labels (`ubuntu-24.04`, `windows-2025`,
`macos-26` or `macos-15-intel`) over `*-latest` in release-critical jobs for reproducibility.

---

## U4 — actionlint install on macOS; is `cargo install actionlint` valid; version + invocation

**FINDING**
- actionlint (rhysd/actionlint) is written in **Go**. Supported macOS installs:
  1. `brew install actionlint` (official Homebrew formula, name `actionlint`);
  2. official script: `bash <(curl https://raw.githubusercontent.com/rhysd/actionlint/main/scripts/download-actionlint.bash)` (accepts optional `<version> <dir>` args);
  3. `go install github.com/rhysd/actionlint/cmd/actionlint@latest` (Go ≥1.16);
  4. Docker `rhysd/actionlint:latest` (also tagged versions e.g. `1.7.12`).
- **`cargo install actionlint` is INVALID** — there is no crates.io package named `actionlint`
  (actionlint is Go, not Rust; the closest Rust crate `github-actions` is unrelated). A
  pip wrapper `actionlint-py` exists (downloads the prebuilt binary) but that is not Cargo.
- Latest version: **≥ 1.7.12** (confirmed via Docker tags; treat as at-or-near latest — the
  absolute newest patch should be confirmed on the releases page at story time).
- Single-command repo-wide invocation: bare **`actionlint`** run at repo root — it auto-discovers
  `.github/workflows` and lints all YAML. (Single file: `actionlint path/to/wf.yml`.)

**CONFIDENCE** High for install methods, the "no crates.io / cargo install invalid" conclusion,
and the bare-`actionlint` invocation. Medium on "1.7.12 is the absolute latest" (verify on
releases page).

**PROPOSED STORY FIX** For a macOS dev/CI lint step, install via `brew install actionlint` locally
and via the `download-actionlint.bash` script (pinned version + dir) in CI — do **not** attempt
`cargo install actionlint`. Invoke as bare `actionlint`. If a version pin is required, pass it as
the script's first arg or use the Docker tag.

---

## U6 — cargo-semver-checks with pre-release current version (1.0.0-rc.1) vs 0.1.0 git baseline

**FINDING**
- SemVer treats `0.1.0 → 1.0.0-rc.1` as a **MAJOR bump** (MAJOR 0→1). Prerelease identifiers
  (`-rc.1`) affect only *precedence/ordering*, not the MAJOR/MINOR/PATCH numbers. cargo-semver-checks
  interprets the version string with standard SemVer semantics.
- With `--baseline-rev <rev-where-version=0.1.0>`, cargo-semver-checks reads the baseline crate's
  version from that revision's `Cargo.toml` (0.1.0), builds rustdoc JSON for both sides, and
  compares. Because the current version has MAJOR 1, it **allows breaking changes** consistent with
  a major bump. It does **not** skip the check, and there is **no documented "prerelease exemption"**
  that waives violations for a prerelease *current* version.
- Nuance: the **cargo-semver-checks-action** default (crates.io baseline) picks the "latest normal
  (not pre-release or yanked)" version. `--baseline-rev` bypasses that. Issue #275 concerns baseline
  *selection* when only prereleases/yanked versions exist (prefer a prerelease baseline) — this is
  about which baseline is chosen, not about exempting the current prerelease from checks.

**CONFIDENCE** High that the 0.1.0→1.0.0-rc.1 transition is evaluated as a major bump (runs checks,
allows breaking changes). Medium that no prerelease-specific special-casing of the *current* version
exists in edge cases (the tool doc does not spell out prerelease-current handling explicitly — it is
inferred from SemVer semantics + absence of a documented exemption).

**PROPOSED STORY FIX** Use `cargo semver-checks --baseline-rev <last-release-tag>` in the release
gate. Expect it to enforce the major bump correctly for `1.0.0-rc.1`. If the crate's history on
crates.io is only prereleases/yanked, prefer the explicit `--baseline-rev`/`--baseline-version`
flags over the action's crates.io default. Do not assume prereleases are exempt from semver gating.

---

## U2 — ubuntu-24.04 host packages for Rust workspace w/ keyring building gnu + musl

**FINDING**
- **musl-tools**: **required** for `x86_64-unknown-linux-musl` (provides `musl-gcc` used as the C
  cross-linker / for any C shims). `pkg-config`: **needed** whenever any dependency probes system
  libs. `libssl-dev`: **NOT needed** if the workspace uses rustls (per ADR-050, prism is rustls-tls
  mandatory) — only needed for openssl-sys/native-tls, which prism forbids.
- **libdbus-1-dev**: **NOT needed** by the modern `keyring` v3.x Secret Service backend, which uses
  the pure-Rust `zbus` D-Bus stack (`secret-service` crate over zbus) rather than linking the C
  `libdbus-1`. (libdbus-1-dev is only needed if you pin the legacy `dbus`-C-linked backend.)
- musl static target: D-Bus/Secret Service is a **runtime** dependency (needs a running session
  D-Bus + a Secret Service provider), not a build-time C-link dependency once zbus (pure Rust) is
  used. Therefore the **musl build does not need a different keyring backend at compile time** —
  the same zbus-based Secret Service backend compiles for both gnu and musl. Headless CI (no
  D-Bus/keyring daemon) is a *runtime* concern: mock/stub the keyring at the store boundary in
  tests (aligns with prism's credential-reference model, AD-017), do not switch backends per-target.
- keyring v3 relevant feature flags: `sync-secret-service` (blocking Secret Service backend),
  `crypto-rust` (pure-Rust session encryption, avoids OpenSSL — pairs well with musl static),
  `vendored` (vendor native crypto — generally unnecessary when using `crypto-rust`).

**CONFIDENCE** High on musl-tools/pkg-config needed and libssl-dev-not-needed-under-rustls.
Medium-high on "zbus means no libdbus-1-dev and no per-target backend switch" and the feature-flag
names — these should be reconciled against the **exact keyring version pinned in the prism
workspace `Cargo.toml`** (prism code/lockfile is the authority for the actual backend + features).

**INCONCLUSIVE (flagged)**: exact keyring crate version + enabled features in prism's manifest —
confirm the pinned `keyring = "3.x"` and its feature set before finalizing the apt package list.

**PROPOSED STORY FIX** ubuntu-24.04 apt list for the build job: `musl-tools pkg-config` (add
`libdbus-1-dev` **only if** the lockfile shows a libdbus-C-linked keyring backend; default: omit).
Do **not** add `libssl-dev` (rustls per ADR-050). Enable keyring `crypto-rust` for the musl target
so session encryption has no OpenSSL/native dependency. Keep one keyring backend across both
targets; handle absent-daemon in CI via a test-boundary mock, not a backend swap.

---

## U8 + U26 — prerelease-inclusive "latest" resolution; install-script distribution & version passing

**FINDING**
- `GET /repos/{owner}/{repo}/releases/latest` **excludes** prereleases and drafts (documented — it
  returns the most recent *non-prerelease, non-draft* release).
- To get the most recent release **including** prereleases without auth: `GET /repos/{o}/{r}/releases?per_page=1`
  — the list endpoint returns releases ordered newest-first and includes prereleases (each item has
  a `prerelease: bool` field). Take element `[0]`.
- Unauthenticated REST rate limit: **60 requests/hour per source IP** (authenticated: 5,000/hr).
  Budget install-script polling accordingly; cache or accept a token for high-volume CI.
- Install-script distribution: rustup, starship, and uv all serve installers from **stable,
  project-controlled vanity domains** (`sh.rustup.rs`, `starship.rs/install.sh`,
  `astral.sh/uv/install.sh`), NOT from `raw.githubusercontent.com@<tag>` and NOT as release assets.
  The durable pattern is a small project-hosted bootstrap stub that then downloads the correct
  release binary. (raw.githubusercontent@tag is discouraged: no stable CDN semantics, ties URL to a
  commit/tag.)
- Passing a VERSION through `irm <url> | iex` (PowerShell): piping to `iex` cannot carry positional
  args. Two documented durable patterns:
  1. **Environment variable before the pipe** (uv's approach): `$env:UV_INSTALL_VERSION="0.5.0"; irm https://astral.sh/uv/install.ps1 | iex` (rustup/starship expose analogous env knobs).
  2. **Invoke a downloaded scriptblock with args**: `& ([scriptblock]::Create((irm <url>))) -Version 0.5.0`.
  Prefer the env-var form for stock-Windows install one-liners (simplest, avoids scriptblock parsing).

**CONFIDENCE** High for the REST `/latest` exclusion, the `?per_page=1` include-prerelease pattern,
and the 60/hr unauth limit (official GitHub docs). High that rustup/starship/uv use project-owned
domains; medium on the exact env-var names per tool (verify each tool's current install doc — uv's
`UV_INSTALL_VERSION` is documented; rustup/starship names should be confirmed at story time).

**PROPOSED STORY FIX** For prism's own install-script "latest" resolution, call
`/releases?per_page=1` (not `/latest`) so prerelease/RC channels are discoverable; guard for the
60/hr unauth cap (cache the result, or allow a `GITHUB_TOKEN`). Serve prism's install script from a
prism-controlled stable URL (or, if none, pin `raw.githubusercontent.com` to an immutable tag as a
documented interim). For the Windows one-liner, pass the target version via a
`$env:PRISM_INSTALL_VERSION` env var read by the script, not via `iex` args.

---

## U18 — upload-artifact zero-file default, multi-path flattening, executable-bit preservation

**FINDING**
- `if-no-files-found` **default = `warn`** (options: `warn` | `error` | `ignore`). A zero-match glob
  does **not** fail the step by default.
- Multi-path: the artifact root is the **least common ancestor (LCA)** of all provided search paths;
  directory structure is preserved relative to that LCA (not arbitrarily flattened).
- **Executable bit is NOT preserved.** Artifacts are stored as a ZIP-based blob; Unix mode bits are
  lost on download. To preserve `+x` (or symlinks), **tar-wrap** the files before upload and untar
  after download.

**CONFIDENCE** High for the `warn` default and LCA path behaviour (README-documented). Medium for
the executable-bit loss — consistent, widely-reported behaviour of the ZIP artifact format, but the
README does not state permission handling explicitly (inference from the storage mechanism).

**PROPOSED STORY FIX** In release jobs that must fail on missing build output, set
`if-no-files-found: error` explicitly (do not rely on the `warn` default). For any uploaded
executable/binary (prism CLI, adapters), **tar the artifact** (`tar czf prism.tar.gz -C dist .`)
before `upload-artifact` and untar on the consuming side, so the executable bit survives.

---

## U20 — current SHAs/tags for dtolnay/rust-toolchain@stable + actions/checkout; v5/v6 breaking changes

**FINDING**
- `actions/checkout` latest major **v6** (v6.0.2 real). `dtolnay/rust-toolchain@stable` is a
  *moving* channel tag re-pointed as Rust releases; there is no stable "release version" to quote.
- **Commit SHAs cannot be reliably asserted from research/training data** — they change and must be
  resolved at pin time. Resolve with:
  ```bash
  git ls-remote https://github.com/actions/checkout      refs/tags/v6.0.2
  git ls-remote https://github.com/dtolnay/rust-toolchain refs/heads/stable  # (or refs/tags/stable)
  ```
- checkout v4 → v5/v6 breaking changes: the surfaced evidence points to **infrastructure-level**
  changes (Node runtime bump — e.g. Node 20 → newer — and a higher minimum Actions Runner version),
  consistent with the pattern seen in upload-artifact v6 (Node 24, runner ≥ 2.327.1). A behaviour-level
  breaking-change enumeration for checkout v5/v6 was **not** found in accessible release notes.

**CONFIDENCE** High that checkout is v6 and rust-toolchain@stable is a moving tag. Low on the
specific commit SHAs (must be resolved live) and on the exhaustive v5/v6 breaking-change list.

**INCONCLUSIVE (flagged)**: exact commit SHAs and the full checkout v5/v6 breaking-change list —
resolve SHAs via `git ls-remote` and read the checkout releases page at story-materialization time.

**PROPOSED STORY FIX** Pin actions by **immutable commit SHA** (resolve via `git ls-remote` in a
one-off setup step and record the SHA + human-readable tag in a comment). For rust-toolchain, pin
either `@stable` (accepting float) or a resolved SHA; the repo already pins the Rust channel via
`rust-toolchain.toml`, so the action mainly needs to honour that. Verify runner minimum-version
requirements when bumping checkout across a major.

---

## U21 — wasm-tools 1.248.0 availability; fastest pinned CI install; adapter compat

**FINDING** (verified against the crates.io API 2026-07-19)
- **wasm-tools 1.248.0 IS real and published** on crates.io. **Latest = 1.253.0** (published
  2026-07-07). Recent line: 1.249.0 (05-15), 1.250.0 (05-21), 1.251.0 (05-28), 1.252.0 (06-12),
  1.253.0 (07-07). (The GitHub releases page's "v1.237.0 Latest" is a **stale cached snapshot**;
  the crates.io registry is authoritative and shows 1.253.0.)
- Fastest pinned CI install (avoids `cargo install --locked` compile): **`taiki-e/install-action`**
  with `tool: wasm-tools@<version>` — it downloads prebuilt binaries from GitHub Releases over
  HTTPS and maps versioned tags (its `wasm-tools@latest` was bumped to 1.253.0). Alternatives:
  `cargo-binstall wasm-tools` (wasm-tools README acknowledges binstall; whether it fetches a prebuilt
  vs compiles depends on published binstall metadata — verify), or direct download of release assets
  (Bytecode Alliance uses target-triple asset names, e.g. `...-x86_64-unknown-linux-musl.tar.gz`).
- Component-model adapter: pin the `wasi_snapshot_preview1.wasm` adapter (from
  `wasi-preview1-component-adapter` / `-provider`) **together with** the wasm-tools version used for
  `wasm-tools component new --adapt`; a wasm-tools/adapter version mismatch can cause subtle ABI
  issues. Validate with `wasm-tools validate component.wasm --features component-model`.

**CONFIDENCE** High — 1.248.0 presence and 1.253.0-latest are verified via the crates.io API;
taiki-e/install-action support is confirmed by its release notes. Medium on cargo-binstall
avoiding compilation for wasm-tools (metadata-dependent).

**PROPOSED STORY FIX** Install wasm-tools in CI via `taiki-e/install-action` with an explicit pin
(`wasm-tools@1.253.0` or the version prism standardizes on) — do not `cargo install`. Do not trust
the GitHub releases "Latest" badge for version discovery; use crates.io. Pin the preview1 adapter
version alongside wasm-tools and add a `wasm-tools validate` gate.

---

## U10 — reliable musl-vs-glibc detection in POSIX/bash-3.2 minimal/busybox containers

**FINDING** Composite detection, ordered by reliability, all POSIX/bash-3.2/busybox-`ash` safe:
1. **glibc-positive probe**: `getconf GNU_LIBC_VERSION 2>/dev/null` — succeeds with a version string
   only on glibc (backed by glibc's `gnu_get_libc_version`); musl/busybox typically lack it. Success
   ⇒ glibc.
2. **musl-positive filesystem check**: `test -e /lib/ld-musl-x86_64.so.1` — present on musl/Alpine
   x86_64. Existence ⇒ musl. (Architecture-specific: use `ld-musl-aarch64.so.1` for arm64.)
3. **Fallback (advisory only)**: `ldd --version 2>&1` — glibc prints "GNU libc"/"GLIBC"; **busybox
   `ldd` may lack `--version`, print to stderr, or omit any libc label**, so this is unreliable in
   minimal containers and must not be the sole signal.

Recommended logic: try (1); if it fails, try (2); only then fall back to (3).

**CONFIDENCE** Medium-high. `getconf GNU_LIBC_VERSION` as glibc-positive and the `/lib/ld-musl-*`
path as musl-positive are well-grounded (glibc man page + Alpine wiki); the exact musl loader path
is conventional (widely true on Alpine x86_64) rather than quoted from a spec. `ldd --version`
unreliability in busybox is confirmed by the busybox docs.

**PROPOSED STORY FIX** In the install/detection script, implement the 3-step composite above
(getconf → ld-musl path → ldd fallback), parameterized by architecture for the loader filename.
Do not rely on `ldd --version` alone. Add a test matrix covering an Alpine (musl/busybox) and a
Debian/Ubuntu (glibc) container to prove both branches.

---

## U29 — PowerShell versions preinstalled; 5.1 vs 7.x differences; #Requires strategy

**FINDING**
- Preinstalled ("stock"): **Windows PowerShell 5.1** ships built-in on Win10 22H2, Win11 24H2+,
  Windows Server 2022, and Server 2025. **PowerShell 7.x is NOT preinstalled** (separate install via
  WinGet/MSI/MSIX; `pwsh.exe` runs side-by-side with `powershell.exe`, never replaces it). (Note:
  the legacy PS **2.0** engine is being removed from Win11 24H2 / Server 2025.)
- 5.1 vs 7.x for install scripts:
  - `ConvertFrom-Json -AsHashtable`: **7.0+ only** (absent in 5.1); returns an **OrderedHashtable
    from 7.3+** (order-preserving). `-Depth` exists in both; `ConvertTo-Json` default depth = 2
    (raise it, max 100) to avoid truncating nested config.
  - `Invoke-WebRequest -UseBasicParsing`: functional in 5.1 (avoids the IE parser dependency);
    **no-op in 7.x** (modern .NET HTTP stack, IE-free).
  - Native-exe stdin/encoding: 5.1 defaults to UTF-16LE-ish output governed by `$OutputEncoding`;
    7.1+ defaults to **UTF-8 (no BOM)**. `PSNativeCommandPreserveBytes` (**7.4+**) preserves raw
    bytes in native-command pipelines; not available in 5.1.
  - `&&` / `||` pipeline-chain operators: **7.0+ only** — syntax error in 5.1.
- `#Requires` strategy: `#Requires` is script-global. For stock-Windows scripts, use
  `#Requires -Version 5.1` and stick to the 5.1-∩-7.x feature intersection. For scripts that need
  7.x features, use `#Requires -Version 7.4` (or the minimum needed) **plus** `#Requires -PSEdition Core`.
  For dual-compatibility, omit `-PSEdition`, require `5.1`, and gate 7.x-only paths on runtime checks
  (`$PSVersionTable.PSVersion.Major -ge 7`).

**CONFIDENCE** High — all points are Microsoft-Learn documented (5.1 built-in / 7.x separate;
`-AsHashtable` 7.0/ordered-7.3; `&&`/`||` 7.0; UTF-8-default 7.1; PSNativeCommandPreserveBytes 7.4).

**PROPOSED STORY FIX** Prism's stock-Windows install/CLI-bootstrap script must target **5.1** with
`#Requires -Version 5.1` and avoid `-AsHashtable`, `&&`/`||`, and 7.4 byte-pipe features. Any helper
requiring 7.x must declare `#Requires -Version 7.x -PSEdition Core` and the workflow must install
pwsh first. Set `ConvertTo-Json -Depth` explicitly when serializing nested config.

---

## U30 — piping a secret to a native exe's stdin without corruption/newline/history leaks

**FINDING**
- **Do not use the naive pipeline `$secret | myapp.exe`** for exact secrets: PowerShell appends a
  platform trailing newline (CRLF/LF) to strings sent to native programs, and the byte encoding
  depends on `$OutputEncoding` / version defaults (UTF-16LE on 5.1 vs UTF-8 on 7.x) — both can
  corrupt or alter a token.
- **Canonical cross-version pattern**: `System.Diagnostics.Process` with
  `RedirectStandardInput = $true`, `UseShellExecute = $false`, then a `StreamWriter` over
  `StandardInput.BaseStream` using an explicit `[System.Text.UTF8Encoding]::new($false)` (no BOM),
  calling **`.Write($secret)` (not `WriteLine`)** to avoid a trailing newline. This is identical on
  5.1 and 7.x (stable .NET API) and bypasses pipeline encoding/newline behaviour entirely.
  `PSNativeCommandPreserveBytes` (7.4+) does not solve the trailing-newline issue and is not
  cross-version, so it is not the answer for 5.1-compatible scripts.
- Leak avoidance: never place the secret literally on a command line (PSReadLine history +
  `Start-Transcript` capture command text). Obtain it from `$env:...` or `Read-Host -AsSecureString`,
  keep it in a variable, never `Write-Host` it, and write only to the process stdin. Optionally
  `Set-PSReadlineOption -HistorySaveStyle SaveNothing` for interactive sessions.

**CONFIDENCE** High — trailing-newline behaviour and encoding-default differences are documented
(PowerShell GitHub issues + `about_Character_Encoding`); the Process/StreamWriter pattern is the
standard .NET solution and works in both engines.

**PROPOSED STORY FIX** Implement secret delivery to native tools (e.g. credential handoff) with the
`System.Diagnostics.Process` + `StreamWriter(UTF8 no-BOM).Write()` helper; forbid the `$s | exe.exe`
form in the codebase for secrets. Source secrets from env/SecureString, never from literal args
(aligns with AD-017 opaque-credential handling). Add a test asserting no trailing byte is appended.

---

## U31 — non-executing PowerShell syntax validation in CI; is PSScriptAnalyzer preinstalled?

**FINDING**
- Pure **non-executing syntax** validation: `[System.Management.Automation.Language.Parser]::ParseFile($path, [ref]$tokens, [ref]$errors)` then fail if `$errors.Count -gt 0`. This uses PowerShell's own parser, builds the AST, runs **no** code — the canonical "does it parse?" gate.
- Richer static analysis (style/best-practice/security lints, still non-executing):
  `Invoke-ScriptAnalyzer -Path . -Recurse -Severity Error -EnableExit` (PSScriptAnalyzer).
- **PSScriptAnalyzer preinstall**: **preinstalled on `ubuntu-latest`** (community-verified: runs
  without `Install-Module`). On **`windows-latest` it is NOT guaranteed** by any official doc —
  treat as **not preinstalled** and either `Install-Module PSScriptAnalyzer -Force -Scope CurrentUser`
  or use the `microsoft/psscriptanalyzer-action` (which bundles it and emits SARIF).

**CONFIDENCE** High for the two validation mechanisms and the ubuntu-latest preinstall. Medium for
windows-latest — absence of an official inventory line; safest posture is "install it explicitly".

**INCONCLUSIVE (flagged)**: PSScriptAnalyzer presence on windows-latest is not officially documented
— do not depend on it; install or use the action.

**PROPOSED STORY FIX** Use `Parser::ParseFile` for a fast fail-fast syntax gate on all `.ps1`
(portable, zero deps), and add PSScriptAnalyzer for lint depth. On Windows runners, install
PSScriptAnalyzer via `Install-Module` (or the microsoft action) rather than assuming it is present.

---

## U32 — Rust `directories`/`dirs` config_dir() Windows path (qualifier/org/app)

**FINDING**
- `directories::ProjectDirs::from("com", "1898andCo", "prism").config_dir()` returns
  **`%APPDATA%\1898andCo\prism\config`** (i.e. `{FOLDERID_RoamingAppData}\<org>\<app>\config`).
  On **Windows the qualifier (`"com"`) is IGNORED** — only organization + application appear in the
  path (the qualifier only shapes Linux/macOS reverse-domain identifiers).
- Contrast: the **`dirs` crate** `dirs::config_dir()` returns just the **root** `%APPDATA%`
  (`C:\Users\<u>\AppData\Roaming`) with no project subdirectory. `directories::ProjectDirs` is the
  one that appends `<org>\<app>\config`.

**CONFIDENCE** High — matches docs.rs and the directories-rs README example. **Prism source code is
the final authority** on the actual crate + arguments used; confirm the qualifier/org/app tuple the
codebase passes before hardcoding the expected path in any test/doc.

**PROPOSED STORY FIX** If prism uses `directories::ProjectDirs`, document the Windows config path as
`%APPDATA%\<org>\<app>\config` and note the qualifier is Windows-ignored. Verify the exact tuple in
prism's config crate; write any path-assertion test against the code's real arguments, not the
example above.

---

## U33 — is `-UseBasicParsing` still accepted (no-op) in PowerShell 7.5+?

**FINDING** **Yes.** `-UseBasicParsing` is still present in the `Invoke-WebRequest` parameter set in
PowerShell 7.6 (and thus 7.5); since PS 6 the cmdlet uses the IE-free .NET HTTP implementation, so
the parameter is **accepted but a no-op** (retained for backward compatibility with 5.1 scripts).
No breaking change removing it has been documented for 7.5/7.6.

**CONFIDENCE** High for 7.6 (parameter present in current docs), inferred-high for 7.5 (no
intervening removal). New 7.x-only scripts should simply omit it.

**PROPOSED STORY FIX** Safe to include `-UseBasicParsing` in dual-target (5.1 + 7.x) scripts — it
helps on 5.1 and is ignored on 7.x. Omit it in any script gated to 7.x-only for clarity.

---

## Summary of INCONCLUSIVE / verify-at-story-time items

| Item | Residual verification |
|------|-----------------------|
| U5 (v7↔v8 interop) | One-job smoke test: upload-artifact@v7 then download-artifact@v8 |
| U5 (windows-latest exact OS) | Read runner "Set up job" log line at story time |
| U2 (keyring version/features) | Confirm pinned `keyring` version + features in prism `Cargo.toml`/lockfile |
| U20 (action commit SHAs; checkout v5/v6 breaking list) | `git ls-remote` for SHAs; read checkout releases page |
| U8/U26 (per-tool env var names) | Confirm rustup/starship version-env-var names in their current install docs |
| U21 (cargo-binstall prebuilt for wasm-tools) | Test whether binstall fetches prebuilt vs compiles |
| U31 (PSScriptAnalyzer on windows-latest) | Not officially documented — install explicitly |
| U4 (actionlint absolute-latest patch) | Check releases page (≥1.7.12 confirmed) |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 9 | GitHub Actions latest majors + v7/v8 interop; hosted-runner image status (macos-13/15-intel, *-latest, gh version); gh release prerelease + REST latest + rate limits; actionlint install + cargo-semver-checks prerelease; keyring Linux/musl build deps; PowerShell 5.1-vs-7.x versions; PowerShell secret-stdin + syntax validation; install-script distribution (rustup/starship/uv) + irm-pipe-iex version passing; wasm-tools + musl-vs-glibc detection |
| Perplexity perplexity_ask | 2 | dirs/directories config_dir Windows path (U32); upload-artifact zero-file/LCA/executable-bit (U18) |
| Perplexity perplexity_search | 1 | wasm-tools latest version cross-check (registry snapshot) |
| WebFetch | 1 | crates.io API authoritative wasm-tools version list (verified 1.248.0 real, 1.253.0 latest) |
| Training data | 1 area | Bash array conditional-flag idiom (U3) — standard POSIX/bash, low-risk |

**Total MCP tool calls:** 12 MCP (9 research + 2 ask + 1 search) + 1 WebFetch = 13.
Two additional `perplexity_research` attempts failed transiently ("terminated" / oversize) and were
re-fired successfully. **Training-data reliance:** low — every version number was checked against a
live source (crates.io API, runner-images repo, action release pages, Microsoft Learn); only the
generic bash-array idiom (U3) rests on model knowledge, and it is a stable, verifiable construct.
