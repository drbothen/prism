# Prism

[![CI](https://github.com/drbothen/prism/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/drbothen/prism/actions/workflows/ci.yml)
[![v1.0.0](https://img.shields.io/badge/version-v1.0.0-blue)](https://github.com/drbothen/prism/releases/tag/v1.0.0)

Prism is a Rust MCP server that unifies multi-client security sensor management
for MSSP analysts. It provides a single PrismQL query interface over live sensor
APIs (CrowdStrike, Armis, Claroty, Cyberint), normalizes all responses to OCSF,
and executes queries via an ephemeral DataFusion engine. Write operations (alert
acknowledgment, host containment, etc.) are gated behind a two-tier feature flag
system and a confirmation-token workflow.

## Status

Prism v1.0.0 is production-ready. The workspace contains 25 Rust crates. The full
PrismQL query engine runtime, MCP server, multi-tenant architecture, sensor adapter
framework (Claroty xDome, CrowdStrike Falcon, Cyberint, Armis), Digital Twin Universe
behavioral clones, enrichment chain, credential subsystem, audit subsystem, and formal
verification infrastructure are all shipped and operational.

See `CHANGELOG.md` for the full list of what shipped in v1.0.0.

## Install

**macOS (Apple Silicon):**
```bash
curl -LO https://github.com/drbothen/prism/releases/download/v1.0.0/prism-v1.0.0-aarch64-apple-darwin.tar.gz
tar xzf prism-v1.0.0-aarch64-apple-darwin.tar.gz
chmod +x prism
./prism --version
```

**macOS (Intel):**
```bash
curl -LO https://github.com/drbothen/prism/releases/download/v1.0.0/prism-v1.0.0-x86_64-apple-darwin.tar.gz
tar xzf prism-v1.0.0-x86_64-apple-darwin.tar.gz
chmod +x prism
./prism --version
```

**Linux (glibc — most distros):**
```bash
curl -LO https://github.com/drbothen/prism/releases/download/v1.0.0/prism-v1.0.0-x86_64-unknown-linux-gnu.tar.gz
tar xzf prism-v1.0.0-x86_64-unknown-linux-gnu.tar.gz
chmod +x prism
./prism --version
```

**Linux (musl — Alpine, static binary):**
```bash
curl -LO https://github.com/drbothen/prism/releases/download/v1.0.0/prism-v1.0.0-x86_64-unknown-linux-musl.tar.gz
tar xzf prism-v1.0.0-x86_64-unknown-linux-musl.tar.gz
chmod +x prism
./prism --version
```

**Windows (x86_64):**
Download `prism-v1.0.0-x86_64-pc-windows-msvc.zip` from the [v1.0.0 release page](https://github.com/drbothen/prism/releases/tag/v1.0.0).

### Verify

Verify checksums (from the [release page](https://github.com/drbothen/prism/releases/tag/v1.0.0)):

```bash
sha256sum -c checksums.txt
```

Verify build provenance:

```bash
gh attestation verify prism-v1.0.0-<target>.tar.gz \
  --repo drbothen/prism \
  --signer-workflow drbothen/prism/.github/workflows/release.yml
```

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
