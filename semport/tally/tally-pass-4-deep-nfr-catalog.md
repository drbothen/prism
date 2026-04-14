# Pass 4 Deep: NFR Catalog -- Round 1

## Gaps Targeted from Broad Sweep

1. Broad sweep NFR catalog was organized by category but missed specific constants and their CWE references
2. No documentation of the semantic search NFRs (model caching, lazy computation, relevance threshold)
3. Missing import/export format compliance details
4. No coverage of the pre-commit hook pipeline as a quality NFR
5. Distribution and installation NFRs not documented
6. Schema evolution/backward compatibility not covered

## Performance NFRs (Complete)

### P-001: Query Length Limit (CWE-400)
- **Constant:** `MAX_QUERY_LENGTH = 8192` (8 KB)
- **Location:** `src/query/parser.rs:28`
- **Enforcement:** Byte length check BEFORE parsing begins
- **Threat model:** Prevents parser-level DoS from malicious TallyQL input
- **CWE reference:** CWE-400 (Uncontrolled Resource Consumption)

### P-002: Query Nesting Depth Limit (CWE-674)
- **Constant:** `MAX_NESTING_DEPTH = 64`
- **Location:** `src/query/parser.rs:31`
- **Enforcement:** `Rc<Cell<usize>>` counter incremented on each nested expression during parsing
- **Threat model:** Prevents stack overflow from deeply nested boolean expressions
- **CWE reference:** CWE-674 (Uncontrolled Recursion)
- **Design choice:** Uses `Rc<Cell>` instead of `Arc<AtomicUsize>` because parsing is synchronous (never crosses await points)

### P-003: Integer Overflow Protection (CWE-190)
- **Location:** `src/query/parser.rs` integer() function
- **Enforcement:** `try_map` with range check on parsed integers
- **CWE reference:** CWE-190 (Integer Overflow)

### P-004: Git Lock Retry
- **Constant:** `MAX_LOCK_RETRIES = 3`
- **Location:** `src/storage/git_store.rs:29`
- **Behavior:** Push retry on `ErrorCode::Locked` with fetch+merge between attempts
- **Backoff:** Exponential (thread::sleep with increasing duration)

### P-005: Auth Retry Limit
- **Constant:** Max 4 credential attempts
- **Location:** `src/storage/git_store.rs:38-97`
- **Enforcement:** `Cell<u32>` counter in credential callback closure
- **Threat model:** Prevents libgit2's infinite retry loop that would hang the process

### P-006: Fresh Repository Per MCP Call
- **Location:** `mcp/server.rs::store()`
- **Behavior:** Opens new `git2::Repository` for every tool call
- **Reason:** `git2::Repository` is not `Send`/`Sync`, cannot be stored in the async TallyMcpServer
- **Impact:** O(N) for N findings on every single tool call (load_all + SessionIdMapper build)

### P-007: Index Regenerability
- **Location:** `storage/git_store.rs::rebuild_index()`
- **Behavior:** `index.json` is always regenerable from finding files via `rebuild_index()`
- **Design:** Index is a derived artifact, not source of truth. Safe to delete and rebuild.
- **Merge strategy:** `.gitattributes` sets `index.json merge=ours` to avoid conflicts during sync

### P-008: CLI Query Default Limit
- **Constant:** `limit = 100` (default via clap)
- **Location:** `src/cli/mod.rs:206`
- **Behavior:** Caps query result count. Overridable via `--limit`

### P-009: Semantic Search Relevance Threshold
- **Constant:** `0.3` (minimum cosine similarity)
- **Location:** `src/registry/semantic.rs:222`
- **Behavior:** Rules with similarity below 0.3 are excluded from semantic search results

### P-010: Jaro-Winkler Suggestion Threshold
- **Constant:** `SUGGEST_THRESHOLD = 0.6`
- **Location:** `src/registry/matcher.rs:28`
- **Behavior:** Rule IDs with JW score below 0.6 are not included as suggestions

### P-011: Lazy Embedding Computation Warning
- **Threshold:** 50 rules computed on-the-fly
- **Location:** `src/registry/semantic.rs:204`
- **Behavior:** If more than 50 rules need embedding computation during a search, warns user to run `tally rule reindex --embeddings`

## Security NFRs (Complete)

### S-001: No Unsafe Code
- **Enforcement:** `#![forbid(unsafe_code)]` in both `main.rs` and `lib.rs`
- **Scope:** Entire crate; cannot be overridden by allow attributes

### S-002: No Unwrap in Production
- **Enforcement:** `clippy::unwrap_used = "deny"` in `Cargo.toml` lints section
- **Exception:** Test code (not subject to this lint)

### S-003: Credential Chain (4-strategy)
- **Location:** `src/storage/git_store.rs:38-97`
- **Strategies (in order):**
  1. Git credential helper (osxkeychain/GCM/store, gh auth setup-git)
  2. `GITHUB_TOKEN` or `GIT_TOKEN` environment variable
  3. SSH agent (Unix `ssh-agent`, Windows OpenSSH agent via `SSH_AUTH_SOCK`)
  4. SSH key files (`~/.ssh/id_ed25519`, `~/.ssh/id_rsa`, `~/.ssh/id_ecdsa`)
- **Platform awareness:** Cross-platform SSH key naming, credential helper auto-detection

### S-004: Error Display Sanitization Note
- **Documentation:** SOUL.md principle 5 states `Display` impl is for logging/CLI output; sanitize before sending to external systems
- **Status:** Documented principle, not enforced in code

### S-005: License Compliance
- **Enforcement:** cargo-deny in CI (ci.yml) and pre-commit (lefthook)
- **Allowed licenses:** Apache-2.0, MIT, BSD-2/3-Clause, ISC, Unicode-3.0, Zlib, CC0-1.0, MPL-2.0, NCSA, CDLA-Permissive-2.0
- **Advisory ignores:** RUSTSEC-2024-0436 (paste crate, unmaintained but feature-complete, no CVE)

### S-006: Dependency Security
- **Enforcement:** cargo-deny checks advisories on every CI run and pre-commit
- **Yanked crates:** Warned (not failed)

### S-007: CI Permission Minimization
- **Pattern:** `permissions: contents: read` on all CI workflows except release (which needs write)
- **Credential hygiene:** `persist-credentials: false` on all checkout steps
- **Shallow clone:** `fetch-depth: 1` on all CI steps (except release which needs history)

### S-008: GitHub Release Artifact Integrity
- **Pattern:** SHA-256 checksums for all release binaries
- **Verification:** `<archive>.sha256` file alongside each archive

## Observability NFRs (Complete)

### O-001: Structured Tracing
- **Framework:** `tracing` 0.1 + `tracing-subscriber` 0.3
- **Pattern:** `#[tracing::instrument(skip_all, fields(...))]` on all handlers
- **Count:** 21 instrumented functions (6 storage, 13 CLI, 2 MCP entry)
- **Field examples:** `uuid`, `remote`, `format`, `id`, `file`, `rule`, `severity`, `path`, `input`

### O-002: CLI Verbosity Control
- **Flags:** `-v` (info), `-vv` (debug), `-vvv` (trace), `-q` (error), `-qq` (off)
- **Override:** `RUST_LOG` environment variable takes precedence
- **Default:** warn
- **Global:** Flags apply to all subcommands

### O-003: Stderr-Only Diagnostics
- **Pattern:** `tracing-subscriber` writes to `std::io::stderr` exclusively
- **Reason:** stdout reserved for JSON-RPC (MCP mode) or data output (CLI mode)
- **Implementation:** `.with_writer(std::io::stderr)` in `init_tracing()`

### O-004: Malformed File Logging
- **Pattern:** `tracing::warn!(name, error = %e, "Skipping malformed finding/rule")`
- **Location:** `git_store.rs::load_all()`, `store.rs::load_all_rules()`
- **Behavior:** Logged at warn level with file name and error, then skipped

### O-005: Coverage Reporting
- **Tool:** cargo-llvm-cov
- **CI integration:** Posts coverage report as PR comment, updates existing comment
- **Threshold:** 50% advisory (warn, not fail)
- **Formats:** Summary, HTML, Codecov JSON

## Reliability NFRs (Complete)

### R-001: Idempotent Init
- **Behavior:** `init()` checks `branch_exists()` before creating; returns Ok on existing branch
- **Upgrade path:** Ensures `rules/` directory exists on pre-existing branches (added when rule registry was introduced)

### R-002: Partial Success (Batch Operations)
- **Pattern:** Each item in batch processed independently; failures don't block successes
- **Returns:** `{total, succeeded, failed}` with per-item error details
- **Location:** `mcp/server.rs::record_batch()`, `update_batch_status()`

### R-003: Graceful Degradation
- **Pattern:** Malformed findings/rules skipped during load, not fatal
- **Location:** `git_store.rs::load_all()`, `store.rs::load_all_rules()`

### R-004: Schema Versioning
- **Current:** `default_schema_version() = "1.1.0"`
- **Backward compatibility:** All Finding fields have `#[serde(default)]` annotations
- **Forward compatibility:** `#[non_exhaustive]` on all growing enums
- **Store-level:** `schema.json` on branch records `{"version": "1.1.0", "format": "one-file-per-finding"}`

### R-005: Merge Conflict Avoidance
- **Design:** One-file-per-finding on orphan branch
- **Git attribute:** `index.json merge=ours` prevents index conflicts
- **Rule conflicts:** During sync, newer timestamp wins (semantic merge)

### R-006: Suppression Expiry Auto-Reopen
- **Pattern:** `check_expiry_and_reopen()` iterates all findings during query
- **Agent:** "system" agent ID for auto-reopen transitions
- **Save behavior:** Best-effort (`let _ = store.save_finding(...)`) -- doesn't fail the query

### R-007: Conventional Commits + Changelog
- **Enforcement:** lefthook pre-commit hook
- **Generation:** git-cliff with cliff.toml configuration
- **Release:** Automated changelog in GitHub release from tag

## Scalability NFRs

### SC-001: No Pagination
- **Current:** `load_all()` loads entire findings directory into memory
- **Impact:** O(N) memory for N findings
- **Mitigation:** Default CLI query limit of 100

### SC-002: No Streaming
- **Current:** All results serialized to string before output
- **Impact:** Large result sets held in memory during serialization

### SC-003: Single-File-Per-Finding
- **Benefit:** Zero merge conflicts for concurrent writes
- **Cost:** Many small commits, many small files on branch
- **Scaling limit:** git2 performance degrades with very large number of tree entries

## Standards Compliance NFRs

### STD-001: SARIF 2.1.0 Export
- **Schema:** `sarif-schema-2.1.0.json`
- **Features:** Property bags for tally-specific data (notes, edit_history, tags)
- **Severity mapping:** Critical->error, Important->warning, Suggestion->note, TechDebt->none
- **Rule dedup:** Unique rules extracted from findings
- **Provenance:** `resultProvenance.firstDetectionTimeUtc` from created_at

### STD-002: CSV Export
- **Columns:** uuid, severity, status, rule_id, file_path, line_start, line_end, title, created_at
- **Escaping:** Commas in title replaced with semicolons (basic escaping, not RFC 4180)

### STD-003: MCP Protocol
- **SDK:** rmcp 0.8
- **Transport:** stdio only (JSON-RPC)
- **Capabilities:** tools + resources + prompts

### STD-004: UUID v7 (Time-Ordered)
- **Usage:** All finding IDs
- **Library:** uuid crate v1 with v7 feature

### STD-005: SHA-256 (Fingerprints)
- **Usage:** Content fingerprints for deduplication
- **Format:** "sha256:" prefix + 64 hex chars
- **Library:** sha2 crate 0.10

## Missing/Notable NFRs (Updated)

| Expected NFR | Status | Comment |
|-------------|--------|---------|
| Rate limiting on MCP | Missing | Relies on MCP client behavior |
| Connection pooling | N/A by design | git2 not Send/Sync, fresh per call |
| Health check endpoint | N/A | MCP over stdio has no HTTP surface |
| Telemetry/metrics | Missing | tracing only, no Prometheus/StatsD |
| Pagination | Missing | load_all() loads everything |
| Graceful shutdown | Not verified | MCP server `service.waiting().await` -- need to check signal handling |
| Data encryption at rest | Missing | Findings stored as plaintext JSON |
| Access control | Missing | No auth on CLI or MCP (relies on filesystem permissions) |
| Backup/restore | Partial | Git sync provides remote backup; no dedicated backup command |
| CSV RFC 4180 | Partial | Basic comma escaping (semicolon replacement), not full RFC 4180 |

## Delta Summary
- New items added: 11 performance NFRs (up from 6), 8 security NFRs (up from 5), 5 observability NFRs (up from 4), 7 reliability NFRs (up from 5), 3 scalability NFRs (new category), 5 standards compliance NFRs (new category), expanded missing NFRs (10 items)
- Existing items refined: Constants documented with exact values and CWE references, credential chain fully documented, semantic search thresholds added
- Remaining gaps: Graceful shutdown behavior needs verification

## Novelty Assessment
Novelty: SUBSTANTIVE
The semantic search NFRs (lazy computation, relevance threshold, model caching), the CI permission minimization pattern, the complete standards compliance inventory (SARIF property bags, CSV escaping limitations), the scalability analysis, and the schema evolution strategy all change how one would spec the system. The CSV escaping gap (not RFC 4180) is a quality issue worth noting.

## Convergence Declaration
Another round needed -- graceful shutdown behavior, Tokio runtime configuration details, and the sync merge algorithm details need verification.

## State Checkpoint
```yaml
pass: 4
round: 1
status: complete
files_scanned: 20
timestamp: 2026-04-14T00:00:00Z
novelty: SUBSTANTIVE
next_action: Round 2 -- hallucination audit, graceful shutdown, sync merge details
```
