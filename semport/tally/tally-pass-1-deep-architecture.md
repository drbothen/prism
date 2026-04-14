# Pass 1 Deep: Architecture -- Round 1

## Gaps Targeted from Broad Sweep

1. Broad sweep described the layered architecture but did not verify dependency direction exhaustively
2. MCP server internal structure (3300 lines) was treated as monolithic -- needs decomposition
3. CLI module internal dependency patterns not mapped
4. The dual error model (anyhow in MCP entry, thiserror everywhere else) was not documented
5. Cross-cutting concerns table was incomplete (missing shell completions, import/export, etc.)
6. Storage layer internal architecture (plumbing vs porcelain) not detailed

## Verified Dependency Direction (Module Import Analysis)

### Layer 0: Entry Points
- `main.rs` imports: `cli`, `storage::GitFindingsStore`, `error::TallyError`
- `mcp/server.rs` imports: `model::*`, `storage::GitFindingsStore`, `registry::*`, `query::*`, `session::SessionIdMapper`, `error::*`

### Layer 1: CLI Handlers
- `cli/mod.rs` imports: clap only (definitions)
- `cli/common.rs` imports: `model::*`, `session::SessionIdMapper`, `storage::GitFindingsStore`, `error::*`
- `cli/record.rs` imports: `model::*`, `registry::*`, `session::SessionIdMapper`, `storage::GitFindingsStore`, `error::*`, `cli::common::*`
- `cli/query.rs` imports: `model::*`, `query::*`, `session::SessionIdMapper`, `storage::GitFindingsStore`, `error::*`, `cli::common::*`
- `cli/rule.rs` imports: `registry::*`, `storage::GitFindingsStore`, `error::*`

**Pattern confirmed:** CLI handlers import downward (model, query, registry, storage, error) and sideways (cli::common). No CLI handler imports from mcp/.

### Layer 2: Domain
- `model/finding.rs` imports: `model::state_machine::*` only (peer within same layer)
- `model/identity.rs` imports: `model::finding::*` (peer), sha2, hex
- `model/state_machine.rs` imports: nothing from crate (self-contained)
- `query/parser.rs` imports: `query::ast::*`, `query::error::*`, `query::fields::*` (peers)
- `query/eval.rs` imports: `query::ast::*`, `query::fields::*`, `model::*` (peers + lower layer)
- `registry/matcher.rs` imports: `registry::rule::*`, `registry::normalize::*`, `registry::stopwords::*` (peers)
- `registry/store.rs` imports: `storage::GitFindingsStore`, `registry::rule::Rule`, `error::*`
- `session.rs` imports: `model::Severity` only

**Key finding:** `registry/store.rs` imports from `storage/` -- this is a cross-layer dependency where a domain module reaches into infrastructure. This is the one exception to the strict layering described in SOUL.md. The RuleStore uses `GitFindingsStore`'s `_pub` wrapper methods.

### Layer 3: Infrastructure
- `storage/git_store.rs` imports: `model::Finding`, `error::*` only
- `error.rs` imports: `model::state_machine::LifecycleState` only

**Note:** `error.rs` has a reverse dependency on `model/state_machine.rs` for the `InvalidTransition` variant's field types. This creates a technical cycle: model depends on error (via `Result` type alias), and error depends on model (via `LifecycleState` in error variant). In practice this is not a Rust compile cycle because the `Result` type alias is imported at use sites, not through module-level dependency.

## MCP Server Internal Structure

`src/mcp/server.rs` is ~3300 lines. Its internal organization:

| Section | Lines (approx) | Content |
|---------|----------------|---------|
| Imports + struct def | 1-40 | TallyMcpServer struct, module imports |
| Input type definitions | 42-478 | 24 MCP input DTOs (all derive Deserialize + JsonSchema) |
| ToolOutput type | 480-520 | Response wrapper |
| `impl TallyMcpServer` (new + helpers) | 520-580 | Constructor, `store()`, `to_mcp_err()`, `resolve_id_mcp()` |
| `#[tool_router] impl` | 580-2200 | 24 tool methods |
| `#[prompt_router] impl` | 2200-2550 | 8 prompt methods |
| `#[tool_handler] impl ServerHandler` | 2550-3280 | `get_info()`, `list_resources()`, `list_resource_templates()`, `read_resource()`, `list_prompts()`, `get_prompt()`, timeline helper |
| `run_mcp_server()` | 3285-3298 | Async entry point |

### Tool Method Pattern (All 24 Tools)

Every tool follows this exact structure:
1. Extract `input` from `Parameters<InputType>`
2. Call `self.store()?` to open fresh GitFindingsStore
3. Load data (load_all, load_finding, etc.)
4. Perform domain logic
5. Return `CallToolResult::success(vec![Content::text(json_string)])`

### Helper Methods

| Method | Purpose |
|--------|---------|
| `new(repo_path)` | Constructs TallyMcpServer with tool_router and prompt_router |
| `store()` | Opens fresh GitFindingsStore per call (git2 not Send/Sync) |
| `to_mcp_err(e: TallyError)` | Maps domain errors to McpError (INTERNAL_ERROR or INVALID_REQUEST) |
| `resolve_id_mcp(store, id_str)` | Resolves UUID or short ID, builds SessionIdMapper from all findings |

### The store() Pattern -- Architectural Constraint

```rust
fn store(&self) -> std::result::Result<GitFindingsStore, McpError> {
    GitFindingsStore::open(&self.repo_path).map_err(|e| self.to_mcp_err(e))
}
```

This opens a fresh `git2::Repository` for every single tool call. The `TallyMcpServer` struct holds only `repo_path: String` (plus the routers). The Repository itself is never stored because git2's `Repository` type does not implement `Send` or `Sync`, and rmcp's tool methods are `async`.

**Performance implication:** Each tool call does:
1. Open repository (git2 open)
2. Load all findings (deserialize every JSON file)
3. Build session mapper (iterate all findings)
4. Perform the actual operation
5. Repository dropped on return

For N findings, every tool call is O(N) even for point lookups.

## Dual Error Architecture

| Context | Error Type | Used For |
|---------|-----------|----------|
| Domain layer | `TallyError` (thiserror, 9 variants) | All internal operations |
| CLI layer | `TallyError` (propagated via ?) | CLI handlers bubble errors to main() |
| MCP tool layer | `McpError` (rmcp) | Tool method return types |
| MCP entry point | `anyhow::Result<()>` | `run_mcp_server()` only |
| main() | ExitCode | Maps TallyError variants to exit codes (2 for git errors, 1 for others) |

The `to_mcp_err()` helper maps `TallyError` variants to `McpError`:
- `InvalidTransition`, `InvalidSeverity`, `InvalidInput`, `NoLocation` -> `INVALID_REQUEST`
- Everything else -> `INTERNAL_ERROR` (-1)

**The anyhow anomaly:** `run_mcp_server()` returns `anyhow::Result<()>` even though the rest of the crate uses `TallyError`. In `main.rs`, this is handled by:
```rust
rt.block_on(tally_ng::mcp::server::run_mcp_server("."))
    .map_err(|e| tally_ng::error::TallyError::Io(std::io::Error::other(e.to_string())))
```
The anyhow error is converted to a TallyError::Io, losing the original error type. This is because rmcp's `ServiceExt::serve()` returns anyhow errors.

## CLI Dispatch Architecture

`main.rs::run()` is a single match on `Command` enum with ~35 arms. Each arm:
1. Calls `store()` to open GitFindingsStore at "."
2. Constructs argument structs where needed (RecordArgs, UpdateArgs)
3. Delegates to a `handle_*()` function in cli/

The `Command` enum has 19 variants. The `Rule` variant contains a nested `RuleCommand` with 9 variants, for 28 total commands.

### CLI Argument Patterns

| Pattern | Used By | Example |
|---------|---------|---------|
| Positional args | Import, Rule Get, AddNote | `tally import <path>` |
| `--flag value` | Record, Query, Update | `--severity critical` |
| Value enums | Export format, Output format, Shell | `--format sarif` |
| Repeatable flags | Rule aliases, CWE IDs, locations | `--alias foo --alias bar` |
| Global flags | verbose (-v), quiet (-q) | `tally -vv query` |
| Default values | agent="cli", limit=100, remote="origin" | Implicit if not specified |

## Storage Layer Plumbing Architecture

All git operations use git2's plumbing API directly. NO working tree operations:
- No `checkout`
- No `HEAD` modification
- No index/staging area usage
- All reads use `find_tree()` -> `get_path()` -> blob content
- All writes use `TreeUpdateBuilder` -> `create_updated()` -> `commit()`

### The upsert_file Pattern

Every write operation (save_finding, save_rule, rebuild_index, init) uses this flow:
1. Get current branch tip commit
2. Get its tree
3. Create blob with new content
4. Use `TreeUpdateBuilder::upsert()` to add/replace file in tree
5. Create new commit parenting the old tip
6. Update branch reference to new commit

This means every single finding save creates a new git commit. For batch operations, this creates N commits for N findings.

### Branch Reference Management

- Local branch: `refs/heads/findings-data`
- Remote branch: `refs/remotes/origin/findings-data`
- Branch tip: found via `repo.find_branch()` -> `peel_to_commit()`
- Signature: `repo.signature()` (from git config user.name/email)

## Cross-Cutting Concerns (Complete)

| Concern | Implementation | Consistency |
|---------|---------------|-------------|
| Logging/tracing | `tracing` + `tracing-subscriber` with env-filter | Universal -- all handlers instrumented |
| Error handling | `TallyError` (thiserror) + `McpError` (rmcp) | Universal with anyhow exception |
| Security | `forbid(unsafe_code)`, `clippy::unwrap_used = deny`, query limits | Universal |
| Observability | `#[tracing::instrument]` with skip_all + named fields | All storage ops + all CLI handlers (21 instrumented functions) |
| Shell completions | `clap_complete` for bash/zsh/fish/powershell | Single command |
| Import/Export | dclaude/zclaude import, SARIF 2.1.0/CSV/JSON export | Export standards-based |
| Schema versioning | `default_schema_version() = "1.1.0"`, serde defaults for backward compat | Consistent |
| Spell checking | typos (CI + lefthook) | CI and pre-commit |
| TOML formatting | taplo (CI + lefthook) | CI and pre-commit |
| License compliance | cargo-deny (advisories, licenses, bans) | CI and justfile |
| Changelog | git-cliff with conventional commits | Release pipeline |

## Component Responsibility Map (Refined)

| Component | Primary Responsibility | Secondary Responsibilities |
|-----------|----------------------|--------------------------|
| main.rs | CLI dispatch, exit code mapping | Tracing init, Tokio runtime creation |
| cli/mod.rs | CLI struct definitions (clap) | OutputFormat, ExportFormat enums |
| cli/common.rs | Shared utilities | ID resolution, expiry check, output formatting (JSON/table/summary) |
| cli/record.rs | Finding creation | Location parsing, identity resolution, rule matching, relationship linking |
| cli/query.rs | Finding search | Filter composition (TallyQL + CLI flags), sort, output formatting |
| cli/export.rs | Format conversion | SARIF 2.1.0 with property bags, CSV with comma escaping, JSON pretty-print |
| cli/import.rs | Legacy format import | dclaude + zclaude format detection and mapping |
| mcp/server.rs | MCP protocol surface | All 24 tools, 8 prompts, 14 resources, server lifecycle |
| model/finding.rs | Core entity definition | Field editing with audit trail, schema versioning |
| model/identity.rs | Deduplication | Fingerprint computation, proximity matching |
| model/state_machine.rs | Lifecycle enforcement | Transition validation, string parsing |
| storage/git_store.rs | Persistence | Orphan branch CRUD, sync, index rebuild, git context detection |
| query/parser.rs | TallyQL parsing | Security limits (length, depth), comment stripping |
| query/eval.rs | Query evaluation | Filter application, sorting, severity ordering |
| registry/matcher.rs | Rule resolution | 7-stage pipeline, namespace conflict detection |
| registry/store.rs | Rule persistence | CRUD via GitFindingsStore._pub methods |
| session.rs | Short ID mapping | Severity-prefixed counters, case-insensitive resolution |
| error.rs | Error types | 9 structured variants with actionable messages |

## Delta Summary
- New items added: MCP server internal decomposition (6 sections, ~3300 lines mapped), dual error architecture (anyhow anomaly), CLI dispatch architecture (19+9=28 commands), storage plumbing architecture (upsert_file pattern), cross-cutting concerns (11 items, up from 4), component responsibility map (18 components)
- Existing items refined: Dependency direction verified exhaustively, registry/store.rs cross-layer dependency identified, error.rs reverse dependency on model noted
- Remaining gaps: Mermaid diagrams not regenerated (already in broad sweep), thread/concurrency model for MCP server

## Novelty Assessment
Novelty: SUBSTANTIVE
The MCP server internal decomposition, the dual error architecture (anyhow anomaly), the O(N) per-tool-call performance characteristic, the cross-layer dependency in registry/store.rs, and the complete cross-cutting concerns inventory all change how one would spec the system. The store() pattern and its performance implications are critical for any reimplementation.

## Convergence Declaration
Another round needed -- the thread model for the MCP server (Tokio multi-thread + sync git2 inside async) and the exact sync/merge algorithm need verification.

## State Checkpoint
```yaml
pass: 1
round: 1
status: complete
files_scanned: 25
timestamp: 2026-04-13T23:45:00Z
novelty: SUBSTANTIVE
next_action: Round 2 -- hallucination audit, thread model, sync algorithm, Mermaid update
```
