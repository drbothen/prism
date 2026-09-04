# Prism

[![CI](https://github.com/drbothen/prism/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/drbothen/prism/actions/workflows/ci.yml)
[![v1.0.0-rc.1](https://img.shields.io/badge/version-v1.0.0--rc.1-blue)](https://github.com/drbothen/prism/releases/tag/v1.0.0-rc.1)

Prism is a Rust MCP server that unifies multi-client security sensor management
for MSSP analysts. It provides a single PrismQL query interface over live sensor
APIs (CrowdStrike, Armis, Claroty, Cyberint), normalizes all responses to OCSF,
and executes queries via an ephemeral DataFusion engine. Write operations (alert
acknowledgment, host containment, etc.) are gated behind a two-tier feature flag
system and a confirmation-token workflow.

## Status

Prism v1.0.0-rc.1 is the first release candidate. It ships with Claroty xDome as the
sole supported sensor (14 tables), the full PrismQL query engine, MCP server,
multi-tenant architecture, formal verification, and structured audit logging.
CrowdStrike Falcon, Cyberint, and Armis code is present in the workspace but not
supported in rc.1; they return in the final v1.0.0 release.

See `CHANGELOG.md` for the full list of what shipped in v1.0.0-rc.1.

## Install

**macOS / Linux (recommended):**

```bash
curl -fsSL https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.sh | bash
```

Auto-detects platform (macOS Apple Silicon/Intel, Linux glibc, Linux musl), verifies
the SHA-256 checksum, and installs to `/usr/local/bin` or `~/.local/bin`. To pin a
specific version:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.sh) \
  --version v1.0.0-rc.1
```

**Windows (PowerShell 5.1+):**

```powershell
irm https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.ps1 | iex
```

Installs to `%LOCALAPPDATA%\prism\bin`. To pin a version:

```powershell
$env:PRISM_INSTALL_VERSION = 'v1.0.0-rc.1'
irm https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.ps1 | iex
```

### Manual download (fallback)

Download the archive for your platform from the
[v1.0.0-rc.1 release page](https://github.com/drbothen/prism/releases/tag/v1.0.0-rc.1):

| Platform | Archive |
|----------|---------|
| macOS (Apple Silicon) | `prism-v1.0.0-rc.1-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `prism-v1.0.0-rc.1-x86_64-apple-darwin.tar.gz` |
| Linux (glibc) | `prism-v1.0.0-rc.1-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (musl) | `prism-v1.0.0-rc.1-x86_64-unknown-linux-musl.tar.gz` |
| Windows | `prism-v1.0.0-rc.1-x86_64-pc-windows-msvc.zip` |

```bash
# macOS / Linux example
tar xzf prism-v1.0.0-rc.1-<target>.tar.gz
chmod +x prism && mv prism /usr/local/bin/prism
prism --version
```

### Verify

Verify checksums (from the [release page](https://github.com/drbothen/prism/releases/tag/v1.0.0-rc.1)):

```bash
# macOS
shasum -a 256 -c checksums.txt
# Linux
sha256sum -c checksums.txt
```

Verify build provenance (requires `gh` CLI):

```bash
gh attestation verify prism-v1.0.0-rc.1-<target>.tar.gz \
  --repo drbothen/prism \
  --signer-workflow drbothen/prism/.github/workflows/release.yml
```

The install scripts run these checks automatically. See [`docs/SETUP.md`](docs/SETUP.md)
for the full operator setup guide — config, credentials, sensor spec placement, and
Claude Code MCP wiring.

## Developer Quick Start

See [`CLAUDE.md`](CLAUDE.md) for build commands, toolchain requirements, and
repository conventions.

```bash
# Fast TDD inner loop (single crate)
just iter <crate>

# Pre-push gate (full workspace check)
just check
```

## Project References

| Path | Description |
|------|-------------|
| `CLAUDE.md` | Build commands, toolchain, git conventions |
| `.factory/STATE.md` | Live pipeline state — current phase, decisions log |
| `.factory/specs/architecture/ARCH-INDEX.md` | Architecture index and module overview |
| `.factory/specs/behavioral-contracts/BC-INDEX.md` | Behavioral contract registry (222 active) |
| `.factory/specs/verification-properties/VP-INDEX.md` | Verification property registry |
| `.factory/stories/STORY-INDEX.md` | Per-story implementation specs |
| `crates/` | 25-crate Rust workspace |
| `Justfile` | Task runner — `just --list` for all recipes |
