# Prism

<!-- TODO: S-0.01 — CI badge (add once GitHub Actions workflow is wired) -->

Prism is a Rust MCP server that unifies multi-client security sensor management
for MSSP analysts. It provides a single PrismQL query interface over live sensor
APIs (CrowdStrike, Armis, Claroty, Cyberint), normalizes all responses to OCSF,
and executes queries via an ephemeral DataFusion engine. Write operations (alert
acknowledgment, host containment, etc.) are gated behind a two-tier feature flag
system and a confirmation-token workflow.

## Status

This project is under active development. The workspace contains 24 Rust crates.
Core data types, credential isolation, security primitives, and the DTU behavioral
clone test infrastructure are implemented. The query engine runtime, MCP server,
and several subsystems are planned for upcoming waves.

See `.factory/STATE.md` for the current pipeline phase and active work.

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
| `crates/` | 24-crate Rust workspace |
| `Justfile` | Task runner — `just --list` for all recipes |
