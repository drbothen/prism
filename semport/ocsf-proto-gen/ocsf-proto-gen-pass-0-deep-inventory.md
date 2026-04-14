# Pass 0 Deep Dive Round 1: Inventory -- ocsf-proto-gen

## Objective

Exhaustive, verified inventory of every file in the repository with exact line counts, precise dependency versions, feature gates, and complete file categorization. Cross-reference against broad sweep for corrections.

---

## Corrections to Broad Sweep

1. **Line count for Cargo.toml**: Broad sweep says 38. Actual: 37 lines.
2. **Line count for integration.rs**: Broad sweep says 603. Actual: 602 lines.
3. **Missing files from broad sweep manifest**: The following files were not listed:
   - `.github/workflows/release.yml` (49 lines) -- tag-based release pipeline (crates.io + GitHub Release)
   - `.github/workflows/validate-codeowners.yml` (28 lines) -- CODEOWNERS syntax validation
   - `.github/CODEOWNERS` (1 line) -- `* @drbothen @Zious11 @arcaven`
   - `.gitignore` (2 lines) -- `/target` and `Cargo.lock`
   - `CONTRIBUTING.md` (39 lines) -- development setup, code standards, PR process
   - `INGESTION.md` (612 lines) -- copy of the broad sweep analysis itself
   - `LICENSE` (21 lines) -- MIT license, copyright 1898 & Co. 2026
4. **Version in CHANGELOG**: Broad sweep says "v0.1.0, v0.1.1" but the Cargo.toml version is `0.1.2`. The CHANGELOG only documents v0.1.0 and v0.1.1; there is no v0.1.2 entry. This is a gap in the release process.
5. **Test count**: Broad sweep says "8 end-to-end tests". Correct count:
   - `tests/integration.rs`: 9 tests (`#[test]` functions)
   - `src/schema.rs`: 3 unit tests
   - `src/type_map.rs`: 12 unit tests
   - `src/lib.rs`: 1 doc test (compile-check only, `no_run`)
   - **Total: 25 tests** (24 runnable + 1 compile-check)
6. **Dependency details missed in broad sweep**:
   - `reqwest` uses `rustls-tls` feature (not default OpenSSL) and `default-features = false`
   - `tokio` uses `rt-multi-thread` and `macros` features
   - `clap` uses `derive` and `env` features (the `env` feature enables `#[arg(env = "...")]`)
   - `serde` appears in both `[dependencies]` and `[dev-dependencies]` (the dev-dep is redundant since it is already a regular dep with the same features)

---

## Complete File Manifest (Verified)

### Source Files

| Path | Lines | Category | Priority |
|------|-------|----------|----------|
| `src/main.rs` | 164 | Binary entry point | 1 (Entry) |
| `src/lib.rs` | 35 | Library root | 1 (Entry) |
| `src/codegen.rs` | 639 | Core module | 3 (Core domain) |
| `src/schema.rs` | 388 | Core module | 3 (Core domain) |
| `src/type_map.rs` | 230 | Core module | 3 (Core domain) |
| `src/error.rs` | 45 | Support | 3 (Core domain) |
| **Subtotal** | **1,501** | | |

### Test Files

| Path | Lines | Tests | Category |
|------|-------|-------|----------|
| `tests/integration.rs` | 602 | 9 | Integration tests |
| `src/schema.rs` (tests mod) | ~58 | 3 | Unit tests (inline) |
| `src/type_map.rs` (tests mod) | ~110 | 12 | Unit tests (inline) |
| `src/lib.rs` (doc test) | ~12 | 1 | Doc test (no_run) |
| **Test lines total** | **~782** | **25** | |

### Configuration Files

| Path | Lines | Purpose |
|------|-------|---------|
| `Cargo.toml` | 37 | Package manifest, dependencies, features |
| `.gitignore` | 2 | Excludes `/target` and `Cargo.lock` |
| `.github/CODEOWNERS` | 1 | 3 owners for all files |

### CI/CD Files

| Path | Lines | Trigger | Jobs |
|------|-------|---------|------|
| `.github/workflows/ci.yml` | 66 | push/PR to main | check, fmt, clippy, test, doc (5 parallel) |
| `.github/workflows/release.yml` | 49 | tag `v*` | test+clippy, publish to crates.io, GitHub Release |
| `.github/workflows/validate-codeowners.yml` | 28 | PR / manual | CODEOWNERS syntax + dup pattern check |

### Documentation Files

| Path | Lines | Purpose |
|------|-------|---------|
| `README.md` | 157 | Usage, type mapping table, CLI reference, library API |
| `CLAUDE.md` | 86 | Architecture reference for AI assistance |
| `CHANGELOG.md` | 29 | Release history (v0.1.0, v0.1.1 -- missing v0.1.2) |
| `CONTRIBUTING.md` | 39 | Dev setup, code standards, PR process |
| `INGESTION.md` | 612 | Codebase analysis (this project's own ingestion) |
| `LICENSE` | 21 | MIT, copyright 1898 & Co. 2026 |

### Grand Total

| Category | Files | Lines |
|----------|-------|-------|
| Source (non-test) | 6 | ~1,333 (excluding inline test mods) |
| Test code | 1 + 2 inline mods | ~782 |
| Config | 3 | 40 |
| CI/CD | 3 | 143 |
| Documentation | 6 | 944 |
| **Total** | **18 tracked files** | **~3,242** |

---

## Complete Dependency Map (with exact versions and features)

### Runtime Dependencies

| Crate | Version | Features | Optional | Purpose |
|-------|---------|----------|----------|---------|
| `clap` | 4 | `derive`, `env` | No | CLI argument parsing with derive macros and env var support |
| `serde` | 1 | `derive` | No | JSON deserialization with derive macro |
| `serde_json` | 1 | (default) | No | JSON parsing and Value type |
| `thiserror` | 2 | (default) | No | Error derive macro |
| `reqwest` | 0.12 | `json`, `rustls-tls` (default-features=false) | Yes (`download`) | HTTP client with Rust-native TLS |
| `tokio` | 1 | `rt-multi-thread`, `macros` | Yes (`download`) | Async runtime for reqwest |

### Dev Dependencies

| Crate | Version | Features | Purpose |
|-------|---------|----------|---------|
| `serde` | 1 | `derive` | Redundant (already in deps) |
| `serde_json` | 1 | (default) | Test schema construction |
| `tokio` | 1 | `rt-multi-thread`, `macros` | Integration test async support |

### Feature Flags

| Feature | Default | Activates | Impact |
|---------|---------|-----------|--------|
| `download` | Yes | `reqwest`, `tokio` | Adds `download-schema` subcommand, `Error::Download` variant |

### Transitive Dependency Analysis

- `reqwest` with `rustls-tls` avoids system OpenSSL dependency -- pure Rust TLS
- `reqwest` with `default-features = false` avoids `default-tls` (OpenSSL) being pulled in
- `clap` with `env` feature is specifically needed for `#[arg(env = "OCSF_SCHEMA_URL")]`
- No workspace-level configuration -- single-crate project

---

## Crate Publishing Metadata

| Field | Value |
|-------|-------|
| `name` | `ocsf-proto-gen` |
| `version` | `0.1.2` |
| `edition` | `2024` |
| `rust-version` | `1.85` (MSRV) |
| `license` | `MIT` |
| `repository` | `https://github.com/1898andCo/ocsf-proto-gen` |
| `homepage` | `https://github.com/1898andCo/ocsf-proto-gen` |
| `keywords` | `ocsf`, `protobuf`, `code-generation`, `cybersecurity`, `schema` |
| `categories` | `development-tools::build-utils`, `command-line-utilities` |

### Dual Target Configuration

```toml
[lib]
name = "ocsf_proto_gen"   # note: underscore (Rust convention)
path = "src/lib.rs"

[[bin]]
name = "ocsf-proto-gen"    # note: hyphen (CLI convention)
path = "src/main.rs"
```

The lib name uses underscores (`ocsf_proto_gen`) while the binary name uses hyphens (`ocsf-proto-gen`). This is standard Rust convention but important for import paths.

---

## Repository Governance

- **CODEOWNERS**: `* @drbothen @Zious11 @arcaven` -- all 3 owners for every file
- **CODEOWNERS validation**: Automated via `mszostok/codeowners-validator@v0.7.1` on PRs
- **Git conventions**: Conventional commits (`feat:`, `fix:`, `docs:`, `test:`, `chore:`)
- **Release process**: Tag-based (`v*`). Tag push triggers test, crates.io publish, and GitHub Release creation
- **GitHub Release action**: `softprops/action-gh-release` pinned to SHA `153bb8e04406b158c6c84fc1615b65b24149a1fe` (v2)
- **Checkout action**: CI uses `actions/checkout@v4`; CODEOWNERS validator pins to SHA `34e114876b0b11c390a56381ad16ebd13914f8d5` (v4)

---

## Delta Summary
- New items added: 7 files missing from broad sweep manifest, complete dependency feature analysis, crate publishing metadata, dual target configuration, repository governance details, corrected test counts (25 not 8), CHANGELOG gap (missing v0.1.2)
- Existing items refined: Line counts verified, dependency versions now include feature flags
- Remaining gaps: None at inventory level

## Novelty Assessment
Novelty: SUBSTANTIVE
Discovered 7 files entirely missing from the broad sweep inventory (release.yml, validate-codeowners.yml, CODEOWNERS, .gitignore, CONTRIBUTING.md, INGESTION.md, LICENSE). Corrected the test count from 8 to 25. Identified a CHANGELOG gap (v0.1.2 not documented). Added dependency feature analysis that materially affects how you would configure the crate in Prism. These findings change the inventory model.

## Convergence Declaration
Another round needed -- must verify that no structural details were missed in the file-level analysis and perform hallucination audit on all claims.

## State Checkpoint
```yaml
pass: 0
round: 1
status: complete
timestamp: 2026-04-13T23:10:00Z
novelty: SUBSTANTIVE
```
