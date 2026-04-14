# Pass 0 Deep: Inventory -- Round 1

## Gaps Targeted from Broad Sweep

The broad sweep inventory was accurate but incomplete in several areas:
1. Missing dependencies (`anyhow`, `hex`, `home`, `clap_complete`)
2. No file-level LOC data
3. CI/CD pipeline not documented
4. Configuration files not inventoried
5. No documentation of feature flags
6. Missing `_pub` method variants on GitFindingsStore

## Corrected and Complete Dependency Map

### Runtime Dependencies (26 total, not 17)

| Dependency | Version | Purpose | Used In |
|-----------|---------|---------|---------|
| anyhow | 1 | Error handling for MCP server entry point | `mcp/server.rs::run_mcp_server()` only |
| chrono | 0.4 (serde) | DateTime handling | model/, storage/, cli/, mcp/ |
| chumsky | 0.10 | Parser combinators for TallyQL | query/parser.rs |
| clap | 4 (derive) | CLI argument parsing | cli/mod.rs |
| clap_complete | 4 | Shell completion generation (bash/zsh/fish/powershell) | main.rs, cli/mod.rs |
| comfy-table | 7 | Terminal table formatting | cli/common.rs |
| git2 | 0.20 | Git storage backend (libgit2) | storage/git_store.rs |
| globset | 0.4 | Glob pattern matching for rule scopes | registry/scope.rs |
| hex | 0.4 | Hex encoding for SHA-256 fingerprints | model/identity.rs |
| home | 0.5 | Home directory detection (SSH keys, model cache) | storage/git_store.rs, registry/semantic.rs |
| humantime | 2 | Duration parsing (7d, 24h) | query/parser.rs |
| rmcp | 0.8 (server, transport-io, macros) | MCP protocol server SDK | mcp/server.rs |
| schemars | 1 | JSON Schema generation for MCP tool inputs | mcp/server.rs |
| serde | 1 (derive) | Serialization framework | Throughout |
| serde_json | 1 | JSON serialization/deserialization | Throughout |
| sha2 | 0.10 | SHA-256 fingerprint computation | model/identity.rs |
| strsim | 0.11 | Jaro-Winkler string similarity | registry/matcher.rs |
| thiserror | 2 | Structured error derive macros | error.rs |
| tokio | 1 (io-std, rt, rt-multi-thread, macros) | Async runtime for MCP server | main.rs (on-demand) |
| tracing | 0.1 | Structured logging | Throughout |
| tracing-subscriber | 0.3 (env-filter) | Log subscriber with env filter | main.rs |
| uuid | 1 (v7, serde) | UUID v7 identity generation | model/, storage/, mcp/ |

### Optional Dependencies (1)

| Dependency | Version | Feature Flag | Purpose |
|-----------|---------|-------------|---------|
| fastembed | 5 | `semantic-search` | Local embedding model (all-MiniLM-L6-v2, ONNX) for semantic rule search |

### Dev Dependencies (4)

| Dependency | Version | Purpose |
|-----------|---------|---------|
| assert_cmd | 2 | CLI subprocess testing |
| predicates | 3 | Assertion predicates for assert_cmd |
| proptest | 1 | Property-based testing |
| tempfile | 3 | Temp directory auto-cleanup for test repos |

**Correction:** The broad sweep listed 17 key dependencies. The actual count is 22 runtime (plus 4 dev), for 26 total. Missing from broad sweep: `anyhow`, `hex`, `home`, `clap_complete`, `globset` (listed in table but not in the "Key Dependencies Map"). The `anyhow` dependency is notable because it is used ONLY for `run_mcp_server()` return type, despite the rest of the crate using `thiserror`.

## Feature Flags

| Feature | Default | Dependencies | Effect |
|---------|---------|-------------|--------|
| `default` | (empty) | none | Base binary without semantic search |
| `semantic-search` | off | fastembed 5 | Enables `src/registry/semantic.rs` module (embedding computation, cosine similarity, reindex) |

The `semantic-search` feature is guarded by `#[cfg(feature = "semantic-search")]` on all functions in `semantic.rs`. The Rule struct's `embedding` and `embedding_model` fields exist unconditionally (Option types), but the computation logic is gated.

## Configuration Files Inventory

| File | Purpose | Key Details |
|------|---------|-------------|
| `Cargo.toml` | Crate manifest | name=tally-ng, bin=tally, edition=2024, MSRV=1.85, Apache-2.0 |
| `Cargo.lock` | Dependency lockfile | Committed (binary crate) |
| `rust-toolchain.toml` | Rust toolchain pinning | channel = "stable" |
| `deny.toml` | cargo-deny config | Advisories (ignores RUSTSEC-2024-0436 paste), licenses (Apache/MIT/BSD/ISC/etc), bans (multi-versions=warn) |
| `cliff.toml` | git-cliff changelog gen | Conventional commits, grouped by type |
| `.typos.toml` | typos spellchecker | Extends words: actve, jsonl, rmcp, sarif. Excludes .claude/, target/, _bmad/ |
| `.lefthook.yml` | Git hooks (pre-commit) | Parallel: fmt-check, clippy, taplo, test, typos |
| `.mcp.json` | MCP client config | tally mcp-server, RUST_LOG=info |
| `.gitignore` | Git ignore rules | (not read but expected standard Rust) |
| `justfile` | Task runner (just) | Groups: build, test, check, format, dev, ci, setup, release |
| `LICENSE` | Apache-2.0 | Standard |

## CI/CD Pipeline (5 Workflows)

### ci.yml -- Primary CI
- **Triggers:** push to develop/main/release/hotfix, PRs (code changes only)
- **Concurrency:** cancel-in-progress per workflow+ref
- **Jobs (5, parallel):**
  1. `check-fmt` -- nightly rustfmt --check (5 min timeout)
  2. `check-clippy` -- clippy --all-targets --all-features -D warnings (10 min)
  3. `build` -- cargo build --all-targets (10 min)
  4. `test` -- cargo test --all-targets + --doc --all-features (10 min)
  5. `deny` -- cargo deny check advisories licenses bans (5 min)
- **Conditional:** `test-semantic` -- only on workflow_dispatch or commit message contains `[semantic]` (15 min, caches fastembed model)

### ci_coverage.yml -- Code Coverage
- **Triggers:** push to develop/main, PRs
- **Tool:** cargo-llvm-cov (llvm-tools-preview)
- **Threshold:** 50% advisory (warn, not fail)
- **PR comment:** Posts/updates coverage report via GitHub API

### ci_status.yml -- Doc-Only Change Handler
- **Purpose:** Provides passing status checks when only docs/markdown changed (no code to test)
- **Detects:** git diff for non-doc changes, creates skip jobs matching required check names

### ci_typos.yml -- Spell Check
- **Triggers:** push to develop/main, PRs
- **Tool:** crate-ci/typos@v1

### release.yml -- Release Pipeline
- **Triggers:** tag push matching `v[0-9]+.*`
- **Jobs:**
  1. `create-release` -- Verify Cargo.toml version matches tag, generate changelog (git-cliff), create GitHub release
  2. `build-binaries` -- Cross-platform builds: x86_64-linux, aarch64-apple-darwin, x86_64-windows. Packages tar.gz/zip + SHA256
  3. `build-semantic-binaries` -- Same matrix with `--features semantic-search` (separate archives named tally-semantic-*)
  4. `publish-crate` -- cargo publish to crates.io (non-prerelease only)
  5. `update-homebrew` -- Updates homebrew-tap formula via GitHub API (prebuilt binaries)
  6. `update-homebrew-semantic` -- Separate tally-semantic formula (conflicts_with tally)
  7. `sync-main` -- Fast-forward merge develop to main after release

## justfile Task Inventory

| Group | Task | Command |
|-------|------|---------|
| build | `build` | `cargo build --all-targets` |
| test | `test` | `cargo test --all-targets` |
| test | `test-doc` | `cargo test --doc --all-features` |
| test | `coverage` | `cargo llvm-cov --all-targets --summary-only` |
| test | `coverage-html` | `cargo llvm-cov --all-targets --html --open` |
| test | `coverage-json` | `cargo llvm-cov --all-targets --codecov` |
| check | `check` | check-fmt + check-clippy + check-deny |
| check | `check-fmt` | `rustup run nightly cargo fmt --all -- --check` |
| check | `check-clippy` | `cargo clippy --all-targets --all-features -- -D warnings` |
| check | `check-deny` | `cargo deny check advisories licenses bans` |
| check | `check-toml` | `taplo fmt --check` |
| check | `lint` | alias for check |
| format | `fmt` | `rustup run nightly cargo fmt --all` |
| format | `fmt-toml` | `taplo fmt` |
| format | `fmt-all` | fmt + fmt-toml |
| dev | `dev` | `cargo watch -x check -x 'test --lib'` |
| dev | `install-hooks` | `lefthook install` |
| dev | `clean` | `cargo clean` |
| ci | `ci` | check-fmt, check-clippy, build, check-deny, test, test-doc |
| setup | `setup` | Install all dev tools (clippy, nightly, cargo-watch, cargo-deny, cargo-nextest, cargo-llvm-cov, git-cliff, taplo, lefthook) |
| release | `changelog` | `git-cliff --config cliff.toml` |
| release | `changelog-unreleased` | `git-cliff --config cliff.toml --unreleased` |
| release | `release <version>` | Update Cargo.toml, generate changelog, commit, tag |

**Notable:** The justfile includes LLVM tool path detection for macOS Homebrew Rust installs (`_llvm-env`), which addresses a known cargo-llvm-cov path issue.

## Git Workflow (from CLAUDE.md + CI)

- **Branching model:** Git Flow -- branch from `develop`, PRs target `develop`
- **Branch naming:** `feature/desc`, `fix/desc`, `release/**`, `hotfix/**`
- **Commit conventions:** Conventional Commits enforced by lefthook pre-commit hook
- **AI attribution:** Explicitly prohibited in commit messages
- **Main branch:** Fast-forwarded from develop by release pipeline (not direct commits)

## Distribution Channels

1. **crates.io:** Published as `tally-ng` (cargo publish)
2. **GitHub Releases:** Pre-built binaries for linux-x64, macOS-arm64, windows-x64 (both standard and semantic variants)
3. **Homebrew:** Custom tap (`1898andCo/homebrew-tap`) with `tally` and `tally-semantic` formulae
4. **cargo-binstall:** Metadata in Cargo.toml for binary install from GitHub releases

## GitFindingsStore Public API Surface

The broad sweep listed the main CRUD methods but missed the `_pub` wrapper methods that delegate to private methods. Full public API:

| Method | Purpose | Instrumented |
|--------|---------|-------------|
| `open(repo_path)` | Open repository, create store | No |
| `init()` | Create/ensure orphan branch | Yes |
| `save_finding(finding)` | Write finding JSON | Yes |
| `load_finding(uuid)` | Read single finding | Yes |
| `load_all()` | Read all findings | Yes |
| `rebuild_index()` | Regenerate index.json | Yes |
| `git_context()` | Read repo_id, branch, commit_sha | No |
| `branch_exists()` | Check if findings branch exists | No |
| `has_remote_branch()` | Check if remote has findings branch | No |
| `sync(remote_name)` | Fetch + merge + push | Yes |
| `rebuild_rule_counts()` | Update finding_count on all rules | (need to check) |
| `upsert_file_pub(path, content, msg)` | Public wrapper for file upsert | No |
| `read_file_pub(path)` | Public wrapper for file read | No |
| `list_directory_pub(dir)` | Public wrapper for directory listing | No |
| `remove_file_pub(path, msg)` | Public wrapper for file removal | No |

The `_pub` methods are used by `RuleStore` since it lives in a different module (`registry/store.rs`) and needs access to git operations. The private `upsert_file`, `read_file`, `list_directory` methods are the actual implementations.

## Test File Inventory

| File | Type | Focus |
|------|------|-------|
| tests/cli_common/mod.rs | Helper | Shared test setup utilities |
| tests/cli_core_test.rs | Integration | Core CLI commands (init, record, query) |
| tests/cli_export_test.rs | Integration | Export CSV/SARIF/JSON |
| tests/cli_mutability_test.rs | Integration | Finding field editing |
| tests/cli_query_enhanced_test.rs | Integration | TallyQL advanced query features |
| tests/cli_query_test.rs | Integration | Basic query features |
| tests/cli_record_test.rs | Integration | Record command variations |
| tests/cli_rule_test.rs | Integration | Rule CRUD commands |
| tests/cli_update_test.rs | Integration | Status update commands |
| tests/e2e_lifecycle_test.rs | E2E | Full finding lifecycle |
| tests/e2e_mcp_workflow_test.rs | E2E | MCP tool workflow sequences |
| tests/e2e_rule_registry_test.rs | E2E | Rule registry end-to-end |
| tests/error_test.rs | Unit | TallyError Display messages |
| tests/identity_test.rs | Unit | Fingerprint + identity resolution |
| tests/mcp_enhanced_test.rs | Integration | Advanced MCP tool features |
| tests/mcp_test.rs | Integration | MCP subprocess tests |
| tests/mcp_unit_test.rs | Unit | In-process MCP tool tests |
| tests/model_test.rs | Unit | State machine, severity, serialization |
| tests/property_edit.rs | Property | Arbitrary field edit roundtrips |
| tests/property_identity.rs | Property | Identity invariants |
| tests/property_query.rs | Property | Query evaluation invariants |
| tests/property_registry.rs | Property | Registry normalization/matcher invariants |
| tests/query_eval_test.rs | Unit | Query evaluation logic |
| tests/query_foundation_test.rs | Unit | Field validation, filter/sort |
| tests/query_parser_test.rs | Unit | TallyQL parser |
| tests/registry_matcher_test.rs | Unit | 7-stage matching pipeline |
| tests/registry_model_test.rs | Unit | Rule model, RuleStatus |
| tests/registry_normalize_test.rs | Unit | Rule ID normalization |
| tests/registry_scope_test.rs | Unit | Glob scope enforcement |
| tests/registry_semantic_test.rs | Unit | Semantic search (feature-gated) |
| tests/session_test.rs | Unit | Short ID assignment/resolution |
| tests/storage_test.rs | Integration | Git-backed storage operations |

**Total: 32 test files** (4 property, 3 e2e, 9 integration CLI, 3 integration MCP, 1 integration storage, 12 unit)

## Delta Summary
- New items added: 9 missing dependencies documented (anyhow, hex, home, clap_complete, globset detailed), complete CI pipeline (5 workflows), justfile task inventory (23 tasks), distribution channels (4), complete config file inventory (11 files), feature flag documentation, `_pub` method variants, git workflow documentation
- Existing items refined: Dependency count corrected (17 -> 22 runtime), test file inventory enriched with types
- Remaining gaps: Exact LOC counts per file (Bash blocked by sandbox), git commit history

## Novelty Assessment
Novelty: SUBSTANTIVE
The CI/CD pipeline documentation (5 workflows including release with cross-compilation, Homebrew tap, crates.io publish), the distribution channel inventory, the justfile task catalog, the corrected dependency count, and the `_pub` method discovery on GitFindingsStore all change how one would spec the system. The release pipeline in particular (dual binary variants, automated Homebrew formula updates, main branch sync) is architecturally significant and was completely absent from the broad sweep.

## Convergence Declaration
Another round needed -- LOC counts per file/module would improve the inventory, and the `anyhow` vs `thiserror` usage asymmetry merits verification.

## State Checkpoint
```yaml
pass: 0
round: 1
status: complete
files_scanned: 35
timestamp: 2026-04-13T23:30:00Z
novelty: SUBSTANTIVE
next_action: Round 2 -- hallucination audit, verify anyhow usage, complete LOC estimates
```
