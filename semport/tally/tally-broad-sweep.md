# Tally Codebase Ingestion: Complete 6-Pass Analysis

> **Purpose:** Primary Rust MCP reference implementation for the Prism project. This document captures architecture, domain model, behavioral contracts, patterns, and design decisions extracted from a systematic codebase analysis.

---

## Pass 0: Inventory

### Tech Stack

| Aspect | Detail |
|--------|--------|
| Language | Rust (Edition 2024, MSRV 1.85+) |
| Binary | `tally` (crate name: `tally-ng`, v0.7.2) |
| Async runtime | Tokio (multi-threaded, only for MCP server mode) |
| CLI framework | Clap 4 (derive macros) |
| MCP SDK | `rmcp` 0.8 (server + transport-io + macros features) |
| Schema generation | `schemars` 1 (JsonSchema derive for MCP tool inputs) |
| Serialization | serde + serde_json |
| Storage | git2 0.20 (libgit2 bindings, orphan branch storage) |
| Identity | uuid v7 (time-ordered), SHA-256 content fingerprints |
| Query language | TallyQL (Chumsky 0.10 parser combinator) |
| Error handling | thiserror 2 (structured error enum) |
| Observability | tracing 0.1 + tracing-subscriber 0.3 |
| String similarity | strsim 0.11 (Jaro-Winkler) |
| Optional | fastembed 5 (semantic search, feature-gated) |
| Test framework | Standard `#[test]` + assert_cmd, predicates, proptest, tempfile |
| Build orchestration | justfile (just check, just ci) |
| Lints | clippy::all = deny, clippy::pedantic = warn, unsafe_code = forbid, unwrap_used = deny |

### Crate Structure (Single Binary Crate, NOT a Workspace)

```
src/
  main.rs              -- Entry point: CLI dispatch + MCP server launch
  lib.rs               -- Public API: re-exports all modules
  error.rs             -- TallyError enum (thiserror)
  session.rs           -- Session-scoped short ID mapper (C1, I2, S3, TD4)
  model/
    mod.rs             -- Re-exports
    finding.rs         -- Finding struct, Severity, Location, relationships, edit_field()
    identity.rs        -- Fingerprint computation, FindingIdentityResolver (dedup)
    state_machine.rs   -- LifecycleState enum, 10 states, validated transitions
  storage/
    mod.rs             -- Re-exports
    git_store.rs       -- GitFindingsStore: orphan branch CRUD, sync, index rebuild
  cli/
    mod.rs             -- Cli struct (clap), Command enum, all subcommand definitions
    record.rs          -- handle_record()
    query.rs           -- handle_query()
    update.rs          -- handle_update()
    suppress.rs        -- handle_suppress()
    batch.rs           -- handle_record_batch()
    export.rs          -- handle_export(), export_csv(), export_sarif()
    import.rs          -- handle_import()
    init.rs            -- handle_init()
    stats.rs           -- handle_stats()
    sync_cmd.rs        -- handle_sync()
    rule.rs            -- All rule subcommand handlers
    note.rs            -- handle_add_note()
    tag.rs             -- handle_manage_tags()
    update_fields.rs   -- handle_update_fields()
    capabilities.rs    -- handle_mcp_capabilities()
    rebuild_index.rs   -- handle_rebuild_index()
    common.rs          -- Shared CLI utilities
  mcp/
    mod.rs             -- Module declaration
    server.rs          -- TallyMcpServer: 23 tools, 8 prompts, resources, ServerHandler impl
  query/
    mod.rs             -- Re-exports
    ast.rs             -- FilterExpr AST, CompareOp, StringOp, Value, SortSpec
    parser.rs          -- Chumsky 0.10 recursive descent parser with depth/length guards
    eval.rs            -- AST evaluator, filter/sort application
    fields.rs          -- Known field registry, field type validation
    error.rs           -- TallyQLError
  registry/
    mod.rs             -- Re-exports
    rule.rs            -- Rule struct, RuleStatus, RuleScope, RuleExample
    matcher.rs         -- 7-stage matching pipeline (exact->alias->CWE->JW->Jaccard->semantic)
    normalize.rs       -- Rule ID normalization (lowercase, hyphenate)
    scope.rs           -- Glob-based scope checking
    store.rs           -- Rule persistence on git branch (rules/<id>.json)
    stopwords.rs       -- Stopword list for token matching
    semantic.rs        -- Feature-gated semantic search (fastembed)
tests/
  32 test files: unit, integration, property-based, e2e lifecycle, MCP workflow
```

### Key Dependencies Map

| Dependency | Purpose | Version |
|-----------|---------|---------|
| rmcp | MCP protocol server SDK | 0.8 |
| git2 | Git storage backend (libgit2) | 0.20 |
| clap | CLI argument parsing (derive) | 4 |
| chumsky | Parser combinators for TallyQL | 0.10 |
| schemars | JSON Schema generation for MCP tool inputs | 1 |
| chrono | DateTime handling | 0.4 |
| uuid | UUID v7 identity generation | 1 |
| sha2 | SHA-256 fingerprints | 0.10 |
| strsim | Jaro-Winkler string similarity | 0.11 |
| globset | Glob pattern matching for rule scopes | 0.4 |
| comfy-table | Terminal table formatting | 7 |
| humantime | Duration parsing (7d, 24h) | 2 |
| tokio | Async runtime (MCP server only) | 1 |
| thiserror | Error derive macros | 2 |
| tracing | Structured logging | 0.1 |

### Entry Points

1. **`src/main.rs::main()`** -- Synchronous entry. Parses CLI args, dispatches to handlers. MCP server mode creates a Tokio runtime on demand.
2. **`src/mcp/server.rs::run_mcp_server()`** -- Async entry for MCP mode. Creates `TallyMcpServer`, binds to stdio transport, serves until disconnect.

---

## Pass 1: Architecture

### Layered Architecture (Strictly Acyclic)

```
                    +-------------------+    +-------------------+
                    |   main.rs (CLI)   |    | mcp/server.rs     |
                    |   clap dispatch   |    | rmcp ServerHandler |
                    +--------+----------+    +--------+----------+
                             |                        |
                    +--------v------------------------v----------+
                    |               cli/ handlers                |
                    |  (record, query, update, suppress, rule...)  |
                    +--------+----------------------------------+
                             |
              +--------------+--------------+
              |              |              |
     +--------v---+  +------v------+  +----v--------+
     |  model/    |  |  query/     |  |  registry/  |
     |  finding   |  |  TallyQL    |  |  rules      |
     |  identity  |  |  parser     |  |  matcher    |
     |  state_mc  |  |  evaluator  |  |  normalize  |
     +--------+---+  +------+------+  +----+--------+
              |              |              |
     +--------v--------------v--------------v--------+
     |                 storage/                       |
     |           GitFindingsStore                     |
     |    (orphan branch, one-file-per-finding)       |
     +--------+--------------------------------------+
              |
     +--------v---+
     |  error.rs  |
     | TallyError |
     +------------+
```

**Dependency direction:** Always downward. `model/` never imports from `cli/`. `storage/` never imports from `mcp/`. The `error` module is at the bottom, depended on by all.

### Dual Interface Pattern

Tally exposes the same domain operations through two interfaces:

1. **CLI (synchronous):** `clap` derives a `Cli` struct with `Command` enum. `main()` dispatches to handler functions in `cli/`. Each handler opens a `GitFindingsStore`, performs operations, and prints JSON/table/summary output.

2. **MCP Server (async):** `TallyMcpServer` implements `rmcp::ServerHandler`. Each tool method (23 total) opens a fresh `GitFindingsStore` per call (because `git2::Repository` is not `Send`/`Sync`). Transport is stdio (JSON-RPC over stdin/stdout).

Both interfaces share:
- The `model/` types (Finding, LifecycleState, Severity, etc.)
- The `storage/` layer (GitFindingsStore)
- The `query/` engine (TallyQL parsing and evaluation)
- The `registry/` matching pipeline

### Storage Architecture: Git-Backed Orphan Branch

- Findings live on an **orphan branch** named `findings-data` (never merged into main)
- Each finding is a single JSON file: `findings/<uuid>.json`
- Rules are stored as: `rules/<rule-id>.json`
- An `index.json` provides fast metadata queries (always regenerable from finding files)
- A `schema.json` records the store version
- `.gitattributes` sets `merge=ours` on index.json to avoid merge conflicts

**Key design decision:** One-file-per-finding ensures zero merge conflicts for concurrent writes from multiple agents. The orphan branch keeps findings data completely separate from source code history.

### Deployment Topology

Single binary, no network services. Two execution modes:
1. **CLI mode:** Direct invocation (`tally record`, `tally query`, etc.)
2. **MCP server mode:** Launched as a subprocess by an MCP client (`tally mcp-server`), communicates via stdio JSON-RPC

Configured in `.mcp.json`:
```json
{
  "mcpServers": {
    "tally": {
      "command": "tally",
      "args": ["mcp-server"],
      "env": { "RUST_LOG": "info" }
    }
  }
}
```

### Cross-Cutting Concerns

| Concern | Implementation |
|---------|---------------|
| Logging | `tracing` with `tracing-subscriber`. CLI controls verbosity (-v/-q). Stderr only -- stdout reserved for JSON-RPC in MCP mode |
| Error handling | `TallyError` enum with `thiserror`. Structured variants with domain context. All errors propagate via `?` |
| Observability | `#[tracing::instrument]` on storage operations. Structured fields (uuid, branch, remote) |
| Security | `#![forbid(unsafe_code)]`, `clippy::unwrap_used = deny`, CWE-400 query length limit (8KB), CWE-674 nesting depth limit (64), CWE-190 integer overflow protection in parser |

---

## Pass 2: Domain Model

### Core Entities

#### Finding
The central domain entity. Represents a code issue discovered by an AI agent.

| Field Group | Fields | Semantics |
|-------------|--------|-----------|
| Identity | uuid (v7), content_fingerprint (SHA-256), rule_id, original_rule_id | Immutable after creation. Fingerprint = SHA-256(file:line_range:rule_id) |
| Location | locations: Vec\<Location\> | Primary + secondary + context locations. Multi-file supported |
| Classification | severity (4-tier), category, tags | Severity is editable. Category and tags are free-form |
| Description | title, description, suggested_fix, evidence | All editable via edit_field() with audit trail |
| Lifecycle | status (LifecycleState), state_history | Validated state machine. Every transition recorded |
| Provenance | discovered_by: Vec\<AgentRecord\>, created_at, updated_at | Multiple agents can discover the same finding |
| Context | repo_id, branch, pr_number, commit_sha | Git context captured at creation time |
| Relationships | relationships: Vec\<FindingRelationship\> | Typed links: duplicate_of, blocks, causes, etc. |
| Suppression | suppression: Option\<Suppression\> | Global, file-level, or inline pattern suppression |
| Mutability | notes: Vec\<Note\>, edit_history: Vec\<FieldEdit\> | Append-only audit trail for edits and annotations |

#### LifecycleState (10-state machine)

```
                                    +--------+
                                    | Closed |  (terminal)
                                    +--------+
                                     ^  ^  ^  ^  ^
                                     |  |  |  |  |
     +------+    +-------------+    +----------+    +--------+    +-----------+
     | Open | -> | Acknowledged| -> |InProgress| -> |Resolved| -> | Closed    |
     +------+    +-------------+    +----------+    +--------+    +-----------+
       |  |            |  |  |          |  |            |
       |  |            |  |  |          |  |            v
       |  |            |  |  v          |  v        +----------+
       |  |            |  |  +----------+------->   | Reopened | -> Acknowledged
       |  |            |  v                         +----------+    | InProgress
       |  |            | +----------+                   ^
       |  |            +>| WontFix  | ------------------+-> Closed
       |  |              +----------+
       |  v                   ^
       | +--------------+    |
       +>| FalsePositive|----+-> Reopened -> Closed
       | +--------------+
       v
     +----------+
     | Deferred | -> Open | Reopened | Closed
     +----------+
       ^
       |
     +------------+
     | Suppressed | -> Open | Reopened | Closed
     +------------+
```

**Key transitions:**
- Open -> Acknowledged, InProgress, FalsePositive, Deferred, Suppressed
- Acknowledged -> InProgress, FalsePositive, WontFix, Deferred
- InProgress -> Resolved, WontFix, Deferred
- Resolved, FalsePositive, WontFix -> Reopened, Closed
- Deferred, Suppressed -> Open, Reopened, Closed
- Reopened -> Acknowledged, InProgress
- **Closed -> (terminal, no transitions)**

Invalid transitions return `TallyError::InvalidTransition { from, to, valid }` -- the error tells you what IS valid.

#### Severity (4-tier)

| Level | Prefix | SARIF Level | Semantics |
|-------|--------|-------------|-----------|
| Critical | C | error | Blocks PR approval |
| Important | I | warning | Blocks PR approval |
| Suggestion | S | note | Advisory |
| TechDebt | TD | none | Advisory |

#### Rule
A rule in the registry that groups related findings across files and PRs.

| Field | Type | Purpose |
|-------|------|---------|
| id | String | Canonical ID (lowercase, hyphens, 2-64 chars) |
| name | String | Human-readable name |
| description | String | What the rule checks |
| aliases | Vec\<String\> | Alternative names that resolve to this rule |
| scope | Option\<RuleScope\> | Include/exclude glob patterns for file matching |
| examples | Vec\<RuleExample\> | Bad/good code examples |
| status | RuleStatus | Active, Deprecated, Experimental |
| finding_count | u64 | Cached count of findings using this rule |
| cwe_ids | Vec\<String\> | Associated CWE identifiers |
| embedding | Option\<Vec\<f32\>\> | Cached vector for semantic search |

### Identity Resolution Algorithm

Three-priority deduplication when recording a finding:

1. **Exact fingerprint match** (confidence: 1.0) -- Same file + line + rule = same finding. Returns existing UUID. Appends agent to `discovered_by` if not already recorded.
2. **Nearby location match** (within 5 lines, same rule) -- Creates a NEW finding but links it as `related_to` the nearby finding with the distance.
3. **No match** -- Creates a genuinely new finding.

Fingerprint formula: `SHA-256(file_path + ":" + line_start + "-" + line_end + ":" + rule_id)`

### Rule Matching Pipeline (7 stages)

When a finding is recorded, the rule_id goes through:

1. **Normalize** -- Lowercase, replace spaces/underscores with hyphens
2. **Exact match** -- HashMap lookup on canonical IDs (confidence: 1.0)
3. **Alias lookup** -- Reverse index: alias -> canonical (confidence: 1.0)
4. **CWE cross-reference** -- Suggestion only (confidence: 0.7)
5. **Jaro-Winkler** -- String similarity on rule IDs (suggestion, threshold 0.6)
6. **Token Jaccard** -- Description token overlap (suggestion, threshold 0.5)
7. **Semantic embedding** -- Feature-gated, deferred

Only stages 2-3 auto-resolve. Stages 4-7 populate `similar_rules` as suggestions. Unknown IDs auto-register as experimental rules.

### Ubiquitous Language

| Term | Meaning |
|------|---------|
| Finding | A code issue discovered by an AI agent |
| Fingerprint | SHA-256 hash of (file + line_range + rule_id) for dedup |
| Rule | A named check pattern that groups related findings |
| Short ID | Session-scoped human-friendly ID (C1, I2, S3, TD4) |
| Lifecycle | The 10-state machine governing finding status |
| Suppression | Mechanism to silence re-reporting of a finding |
| Agent | An AI tool or human that discovers/modifies findings |
| Session | A bounded scope (e.g., one PR review pass) |
| Orphan branch | Git branch with no shared ancestor with main, holds findings data |

---

## Pass 3: Behavioral Contracts

### BC-001: Finding Recording with Dedup

**Preconditions:**
- Findings store initialized (findings-data branch exists)
- Valid severity, file path, line number, and rule_id provided

**Postconditions:**
- If exact fingerprint match: returns existing UUID, status="deduplicated", appends agent to discovered_by if not already recorded
- If nearby match (within 5 lines, same rule): creates new finding, links as related_to with distance
- If no match: creates new finding, status="created"
- Rule ID goes through 7-stage matching pipeline; unknown rules auto-registered as experimental
- Scope warning returned if rule has scope restrictions and file is out of scope

**Error Cases:**
- Invalid severity string -> `TallyError::InvalidSeverity`
- Missing required fields -> MCP `INVALID_REQUEST`

**Evidence:** `tests/mcp_unit_test.rs`, `tests/cli_record_test.rs`, `tests/identity_test.rs`
**Confidence:** HIGH

### BC-002: State Transition Validation

**Preconditions:**
- Finding exists with known current status
- Target status is a valid `LifecycleState`

**Postconditions:**
- If transition is valid: status changes, `StateTransition` appended to state_history with timestamp, agent_id, reason, optional commit_sha
- `updated_at` timestamp updated

**Error Cases:**
- Invalid transition -> `TallyError::InvalidTransition { from, to, valid }` listing all valid targets from current state
- Closed is terminal -- no transitions allowed

**Evidence:** `tests/model_test.rs`, `tests/e2e_lifecycle_test.rs`
**Confidence:** HIGH (from tests + type-level enforcement)

### BC-003: TallyQL Query Parsing

**Preconditions:**
- Input string is non-empty, within 8KB limit

**Postconditions:**
- Parses to `FilterExpr` AST supporting: AND, OR, NOT, comparison (=, !=, >, <, >=, <=), string ops (CONTAINS, STARTSWITH, ENDSWITH), HAS/MISSING, IN lists, duration literals (7d, 24h), date literals
- Keywords are case-insensitive
- Comments (// and #) are stripped, preserving byte offsets for error spans

**Error Cases:**
- Empty input -> `TallyQLError::unexpected_eof`
- Over 8KB -> rejected (CWE-400 protection)
- Nesting over 64 -> rejected (CWE-674 protection)
- Syntax error -> rich error with span, expected, found, and hint

**Evidence:** `tests/query_parser_test.rs`, `tests/query_foundation_test.rs`, `src/query/parser.rs` inline tests
**Confidence:** HIGH

### BC-004: Git-Backed Storage Operations

**Preconditions:**
- Valid git repository at repo_path
- For write operations: findings-data branch exists

**Postconditions:**
- `save_finding()`: Creates/updates `findings/<uuid>.json` as a new commit on orphan branch
- `load_all()`: Reads all JSON files from findings/ directory, skips malformed entries (logged to stderr)
- `init()`: Creates orphan branch with schema.json, empty findings/ and rules/ directories. Idempotent.
- `sync()`: Fetches remote, merges (fast-forward or three-way), pushes. Retries on lock contention (3 attempts with exponential backoff)

**Error Cases:**
- Branch not found -> `TallyError::BranchNotFound { branch }`
- Auth failure -> Wrapped with platform-specific credential guidance
- Merge conflict on findings -> Error (unexpected for one-file-per-finding design)
- Rule merge conflicts -> Resolved semantically (newer timestamp wins)

**Evidence:** `tests/storage_test.rs`, `src/storage/git_store.rs`
**Confidence:** HIGH

### BC-005: Finding Field Editing

**Preconditions:**
- Finding exists
- Field is in editable set: title, description, suggested_fix, evidence, severity, category, tags
- New value is valid for the field type

**Postconditions:**
- Field updated with new value
- `FieldEdit` appended to `edit_history` with old_value, new_value, timestamp, agent_id
- `updated_at` timestamp updated

**Error Cases:**
- Non-editable field -> `TallyError::InvalidInput` listing editable fields
- Invalid value type -> `TallyError::InvalidInput` (e.g., severity must be a string)
- Identity fields (uuid, fingerprint, rule_id, status, created_at) are immutable

**Evidence:** `tests/cli_mutability_test.rs`, `tests/property_edit.rs`
**Confidence:** HIGH

### BC-006: MCP Server Initialization and Transport

**Preconditions:**
- `tally mcp-server` invoked

**Postconditions:**
- Tokio runtime created (on demand, not at startup)
- `TallyMcpServer` created with repo_path, tool_router, prompt_router
- stdio transport established (stdout = JSON-RPC, stderr = diagnostics)
- ServerHandler responds to `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`, `prompts/get`
- Server capabilities: tools + resources + prompts
- Server info includes `instructions` field with usage guidance

**Error Cases:**
- Tokio runtime creation failure -> `TallyError::Io`
- MCP protocol errors -> `McpError` with ErrorCode

**Evidence:** `tests/mcp_test.rs`, `tests/mcp_unit_test.rs`, `tests/e2e_mcp_workflow_test.rs`
**Confidence:** HIGH

### BC-007: Batch Operations with Partial Success

**Preconditions:**
- Array of findings provided
- Store initialized

**Postconditions:**
- Each finding processed independently
- Valid findings recorded; invalid findings return per-item error
- Total/succeeded/failed counts returned
- Duplicates automatically deduplicated (counted as success, not failure)

**Error Cases:**
- Individual finding validation failures don't block other findings
- Per-item results include status + error message for failures

**Evidence:** `tests/mcp_unit_test.rs` batch tests
**Confidence:** HIGH

### BC-008: Session-Scoped Short IDs

**Preconditions:**
- Findings loaded into SessionIdMapper

**Postconditions:**
- Each finding assigned a severity-prefixed counter: C1, C2, I1, I2, S1, TD1, etc.
- Short IDs are case-insensitive for resolution
- Both UUID and short ID accepted anywhere a finding ID is expected
- Short IDs reset each session (not persisted)

**Evidence:** `tests/session_test.rs`, `tests/identity_test.rs`
**Confidence:** HIGH

### BC-009: Suppression with Expiry

**Preconditions:**
- Finding in Open status (or a status that can transition to Suppressed)

**Postconditions:**
- Status transitions to Suppressed
- Suppression metadata recorded: reason, optional expires_at, suppression_type (global/file/inline)
- If expires_at is set: finding auto-reopens on next query after expiry
- State transition recorded in state_history

**Error Cases:**
- Cannot suppress from non-Open status -> error listing valid transitions
- Invalid date format -> INVALID_REQUEST

**Evidence:** `tests/mcp_unit_test.rs`, `src/mcp/server.rs::suppress_finding()`
**Confidence:** HIGH

---

## Pass 4: NFR (Non-Functional Requirements) Catalog

### Performance

| NFR | Implementation | Location |
|-----|---------------|----------|
| Query length limit | 8KB max (CWE-400 DoS prevention) | `query/parser.rs::MAX_QUERY_LENGTH` |
| Query depth limit | 64 max nesting (CWE-674 stack overflow prevention) | `query/parser.rs::MAX_NESTING_DEPTH` |
| Lock retry | 3 attempts with exponential backoff for git ref locks | `storage/git_store.rs::MAX_LOCK_RETRIES` |
| Auth retry limit | 4 credential strategies, then fail (prevents infinite loop) | `storage/git_store.rs::build_remote_callbacks()` |
| Index regenerability | index.json is always regenerable from finding files | `storage/git_store.rs::rebuild_index()` |
| Fresh repo per MCP call | git2::Repository opened fresh per tool call (not Send/Sync) | `mcp/server.rs::store()` |

### Security

| NFR | Implementation | Location |
|-----|---------------|----------|
| No unsafe code | `#![forbid(unsafe_code)]` in main.rs and lib.rs | Root of both files |
| No unwrap in production | `clippy::unwrap_used = "deny"` in Cargo.toml | `[lints.clippy]` |
| Integer overflow protection | `try_map` with range check in parser | `query/parser.rs::integer()` |
| Credential chain | 4-strategy auth: git credential helper -> env token -> SSH agent -> SSH key file | `storage/git_store.rs` |
| Error sanitization note | `Display` impl is for logging/CLI; sanitize before sending to external systems | `SOUL.md` principle 5 |

### Observability

| NFR | Implementation | Location |
|-----|---------------|----------|
| Structured tracing | `#[tracing::instrument]` on all storage operations | Throughout `git_store.rs` |
| CLI verbosity control | -v info, -vv debug, -vvv trace, -q error, -qq off | `main.rs::init_tracing()` |
| Stderr-only diagnostics | All tracing goes to stderr; stdout reserved for JSON-RPC or data output | `main.rs` |
| Malformed file logging | Skipped files logged at warn level with filename and error | `git_store.rs::load_all()` |

### Reliability

| NFR | Implementation | Location |
|-----|---------------|----------|
| Idempotent init | `init()` checks if branch exists before creating | `git_store.rs::init()` |
| Partial success | Batch operations process each item independently | `mcp/server.rs::record_batch()`, `update_batch_status()` |
| Graceful degrade | Malformed findings skipped during load, not fatal | `git_store.rs::load_all()` |
| Schema versioning | `schema_version` field on findings + `schema.json` on branch | `finding.rs::default_schema_version()` |
| Merge conflict avoidance | One-file-per-finding design eliminates concurrent write conflicts | Architecture decision |

### Missing/Notable NFRs

- **No rate limiting** on MCP tool calls (relies on MCP client behavior)
- **No connection pooling** -- each call opens a fresh Repository (by design: git2 not Send)
- **No health check endpoint** -- MCP over stdio has no HTTP surface
- **No telemetry/metrics collection** -- tracing only, no Prometheus/StatsD
- **No pagination** on load_all() -- loads entire findings directory into memory

---

## Pass 5: Convention & Pattern Catalog

### Naming Conventions

| Convention | Example | Consistency |
|-----------|---------|-------------|
| Snake_case for functions | `handle_record()`, `compute_fingerprint()` | Universal |
| PascalCase for types | `LifecycleState`, `FindingIdentityResolver` | Universal |
| `handle_*` for CLI handlers | `handle_record()`, `handle_query()` | Universal in cli/ |
| `*Input` for MCP parameters | `RecordFindingInput`, `QueryFindingsInput` | Universal in mcp/ |
| `*_test.rs` for test files | `mcp_unit_test.rs`, `storage_test.rs` | Universal |
| `property_*` for property tests | `property_identity.rs`, `property_query.rs` | Universal |
| `e2e_*` for end-to-end tests | `e2e_lifecycle_test.rs` | Universal |

### Error Handling Pattern

**Consistent throughout:**
- `TallyError` enum with `thiserror::Error` derive
- `pub type Result<T> = std::result::Result<T, TallyError>` crate-wide alias
- MCP layer wraps to `McpError` via `to_mcp_err()` helper
- Errors include actionable context: `InvalidTransition` lists valid targets, `BranchNotFound` suggests `tally init`
- `#[non_exhaustive]` on enums expected to grow

### MCP Tool Registration Pattern (rmcp)

```rust
#[tool_router]
impl TallyMcpServer {
    #[tool(description = "...")]
    pub async fn tool_name(
        &self,
        params: Parameters<InputType>,
    ) -> Result<CallToolResult, McpError> {
        let input = params.0;
        let store = self.store()?;  // Fresh repo per call
        // ... domain logic ...
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&output).unwrap_or_default(),
        )]))
    }
}
```

**Key patterns:**
- `#[tool_router]` macro generates the tool router from method annotations
- `#[tool(description = "...")]` provides tool metadata
- `Parameters<T>` wraps deserialized input (T derives Deserialize + JsonSchema)
- `schemars::JsonSchema` derive auto-generates tool input schemas
- `#[schemars(description = "...")]` on each field provides per-parameter descriptions
- All tools return `CallToolResult::success(vec![Content::text(...)])`
- Error mapping: `to_mcp_err()` converts `TallyError` to `McpError`

### MCP Prompt Pattern

```rust
#[prompt_router]
impl TallyMcpServer {
    #[prompt(name = "prompt-name", description = "...")]
    pub async fn prompt_name(
        &self,
        Parameters(args): Parameters<ArgsType>,
    ) -> Result<Vec<PromptMessage>, McpError> {
        // Load data
        // Format as user message with structured instructions
        Ok(vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!("Here is the data:\n```json\n{}\n```\n\nPlease...", data),
        )])
    }
}
```

### MCP Resource Pattern

Resources are defined in `list_resources()` (static) and `list_resource_templates()` (parameterized). URI scheme: `findings://`.

| Resource Type | URI Pattern | Data |
|---------------|-------------|------|
| Static | `findings://summary` | Severity/status counts + recent findings |
| Static | `findings://docs/tallyql-syntax` | TallyQL reference (included at compile time) |
| Static | `findings://docs/rule-registry` | Rule registry docs |
| Static | `findings://version` | Version info |
| Static | `findings://rules/summary` | Rule registry summary |
| Template | `findings://file/{path}` | Findings in a specific file |
| Template | `findings://detail/{uuid}` | Full finding detail |
| Template | `findings://severity/{level}` | Findings by severity |
| Template | `findings://status/{status}` | Findings by status |
| Template | `findings://rule/{rule_id}` | Findings by rule |
| Template | `findings://pr/{pr_number}` | Findings by PR |
| Template | `findings://rules/{rule_id}` | Rule detail + findings |
| Template | `findings://agent/{agent_id}` | Findings by agent |
| Template | `findings://timeline/{duration}` | Creation/resolution timeline |

### ServerHandler Implementation Pattern

```rust
#[tool_handler]
impl ServerHandler for TallyMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
            server_info: Implementation { name: "tally".into(), ... },
            instructions: Some("usage guidance string".into()),
        }
    }
    // list_prompts, get_prompt, list_resources, list_resource_templates, read_resource
}
```

The `#[tool_handler]` macro delegates tool listing/calling to the `tool_router`. Prompts and resources are implemented manually in the `ServerHandler` trait impl.

### Storage Pattern: Orphan Branch CRUD

```rust
impl GitFindingsStore {
    pub fn open(repo_path: &str) -> Result<Self> { ... }
    pub fn init(&self) -> Result<()> { ... }           // Idempotent
    pub fn save_finding(&self, finding: &Finding) -> Result<()> { ... }
    pub fn load_finding(&self, uuid: &Uuid) -> Result<Finding> { ... }
    pub fn load_all(&self) -> Result<Vec<Finding>> { ... }
    pub fn rebuild_index(&self) -> Result<()> { ... }
    pub fn sync(&self, remote_name: &str) -> Result<SyncResult> { ... }
    // Internal: upsert_file(), read_file(), list_directory(), branch_tip()
}
```

All operations use `git2` plumbing API directly -- no working tree checkout, no HEAD modification.

### Test Patterns

| Pattern | Convention | Consistency |
|---------|-----------|-------------|
| Setup | `setup_repo()` / `setup_mcp()` creates temp git repo with initial commit + tally init | Universal |
| Test names | Descriptive: `fingerprint_deterministic_for_same_input()`, `deferred_can_transition_to_reopened()` | Universal |
| MCP tests | Two tiers: subprocess (`mcp_test.rs`) and in-process (`mcp_unit_test.rs`) | Universal |
| Property tests | proptest for identity, query, edit, and registry invariants | 4 files |
| E2E tests | Full CLI workflow via `assert_cmd` + `predicates` | 3 files |
| Cleanup | `tempfile::TempDir` auto-deletes on drop | Universal |

### Design Patterns in Use

| Pattern | Usage | Location |
|---------|-------|----------|
| Builder | `ServerCapabilities::builder()` for MCP server caps | `mcp/server.rs` |
| Strategy | Rule matching pipeline with ordered resolution stages | `registry/matcher.rs` |
| Repository | `GitFindingsStore` as persistence abstraction | `storage/git_store.rs` |
| State Machine | `LifecycleState` with validated transitions | `model/state_machine.rs` |
| Interpreter | TallyQL parser + evaluator (AST pattern) | `query/` |
| Visitor | `evaluate()` walks FilterExpr tree | `query/eval.rs` |
| Newtype | `Severity`, `LifecycleState`, `RelationshipType` as distinct types (not raw strings) | `model/` |
| Facade | `cli/mod.rs` re-exports all handler functions | `cli/mod.rs` |
| Content-Addressable Storage | SHA-256 fingerprints for deduplication | `model/identity.rs` |

### Anti-Patterns / Code Smells

| Issue | Location | Impact |
|-------|----------|--------|
| `load_all()` for short ID resolution | `mcp/server.rs::resolve_id_mcp()` | Loads ALL findings just to resolve one short ID. O(n) per call. |
| MCP server.rs is ~3300 lines | `mcp/server.rs` | Single file contains all 23 tools, 8 prompts, all resource handlers, helpers. Could be split by concern. |
| Fresh repo open per MCP tool call | `mcp/server.rs::store()` | Necessitated by git2 not being Send/Sync. Correct but non-obvious trade-off. |
| `unwrap_or_default()` on serialization | Throughout MCP tools | `serde_json::to_string_pretty(&output).unwrap_or_default()` silently returns empty string on serialization failure |

---

## Pass 6: Synthesis

### Executive Summary

Tally is a well-designed single-binary Rust application that serves as a persistent findings tracker for AI coding agents. It uses an innovative git-backed orphan branch storage model that enables conflict-free concurrent writes from multiple agents. The dual-interface design (CLI + MCP server) exposes the same domain operations through both a direct command-line tool and a JSON-RPC MCP server for AI agent integration. The codebase is disciplined: `#![forbid(unsafe_code)]`, comprehensive error handling with domain-specific error types, 747+ tests, and a 10-state lifecycle machine with type-level transition validation.

### Key Findings for Prism

1. **MCP Server Architecture Pattern:** Tally uses `rmcp` 0.8 with the `#[tool_router]` / `#[tool_handler]` / `#[prompt_router]` macro system. This is the primary pattern to replicate. Each tool is an async method with `Parameters<T>` input where T derives both `Deserialize` and `JsonSchema`. The JsonSchema derive auto-generates tool input schemas with per-field descriptions via `#[schemars(description)]`.

2. **Transport Layer:** Stdio only (`rmcp::transport::io::stdio()`). The server is launched as a subprocess. The Tokio runtime is created on demand in `main()` only when `Command::McpServer` is matched -- the rest of the app is synchronous.

3. **Tool Registration:** 23 tools covering CRUD, batch operations, query, export, import, sync, and rule registry management. Tools return `CallToolResult::success(vec![Content::text(json_string)])`. Error mapping uses a simple `to_mcp_err()` helper that wraps domain errors into `McpError`.

4. **Resource System:** 5 static resources + 9 resource templates using a `findings://` URI scheme. Resources provide read-only data access (summary stats, documentation, filtered views). Documentation resources (`tallyql-syntax`, `rule-registry`) are `include_str!()` from markdown files at compile time.

5. **Prompt System:** 8 prompts that load domain data and format structured instructions for AI consumption. Patterns include triage, fix generation, summarization, PR review, and rule consolidation.

6. **git2 is not Send/Sync:** This is a critical architectural constraint. Each MCP tool call opens a fresh `GitFindingsStore` (and thus `git2::Repository`). The comment explicitly acknowledges this: "git2::Repository is not Send/Sync, so we open the repo fresh per tool call."

7. **Error Design:** `TallyError` is an exemplary Rust error enum. Variants include structured fields that tell users not just what went wrong but what to do about it (e.g., `InvalidTransition` includes the list of valid transitions). This pattern should be replicated in Prism.

8. **Identity System:** Hybrid identity with UUID v7 (stable reference), SHA-256 fingerprint (dedup), and rule ID (grouping). The `FindingIdentityResolver` performs three-priority resolution. This is a well-thought-out approach to the "same finding reported by different agents" problem.

### Confidence Assessment

| Area | Confidence | Basis |
|------|-----------|-------|
| Architecture | HIGH | Explicit layering documented in SOUL.md, confirmed by code structure |
| Domain Model | HIGH | Comprehensive structs with serde, clear entity relationships |
| Behavioral Contracts | HIGH | 747+ tests, property tests, e2e lifecycle tests |
| NFRs | HIGH | Explicit security measures (CWE references), DoS protection in parser |
| Conventions | HIGH | Consistent patterns across all modules, enforced by clippy lints |
| MCP Integration | HIGH | Full rmcp integration with tools, resources, and prompts |

### Gaps and Risks

1. **No SSE/HTTP transport** -- Tally only supports stdio. If Prism needs SSE or HTTP transport, additional rmcp features would need to be enabled.
2. **MCP server is synchronous-in-async** -- All tool handlers do synchronous git2 operations inside async methods. This works because the Tokio runtime is multi-threaded, but heavy git operations could block a thread.
3. **No streaming/pagination** -- `load_all()` loads everything into memory. For very large finding sets this could be a concern.
4. **Single-crate limit** -- Tally is intentionally a single binary crate. If Prism needs a library crate for shared types, the module structure would need restructuring.

### Recommendations for Prism

1. **Replicate the rmcp macro pattern** -- `#[tool_router]` + `#[tool(description)]` + `Parameters<T>` + `schemars::JsonSchema` is the cleanest way to define MCP tools in Rust.
2. **Adopt the error design** -- `thiserror` enum with structured variants, `#[non_exhaustive]`, and actionable error messages.
3. **Consider the fresh-repo-per-call pattern** -- If Prism uses git2, the same Send/Sync constraint applies.
4. **Use the resource URI scheme pattern** -- `scheme://type/param` for custom resource URIs.
5. **Adopt the ServerHandler implementation structure** -- `get_info()` with capabilities + instructions, manual prompt/resource dispatch, macro-generated tool dispatch.
6. **Copy the CLI + MCP dual-interface approach** if Prism needs both programmatic and human interfaces to the same operations.
