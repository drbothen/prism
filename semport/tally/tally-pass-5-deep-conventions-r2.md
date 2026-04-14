# Pass 5 Deep: Conventions & Pattern Catalog -- Round 2

## Hallucination Audit

Reviewing Round 1 claims against source:

1. **to_mcp_err is a method that maps variants:** INCORRECT. Per Architecture R2, `to_mcp_err` is a free function at line 2689, always returns `ErrorCode(-1)`. The R1 "Error-to-McpError Mapping" section showing variant-based mapping is wrong. Corrected: the error handling pattern is actually TWO patterns:
   - `to_mcp_err(e)` for generic domain errors -> INTERNAL_ERROR
   - Inline `McpError { code: ErrorCode::INVALID_REQUEST, ... }` for validation errors

2. **All Option fields have skip_serializing_if:** Verified against Finding struct (lines 33-86 in finding.rs). All Option fields: original_rule_id, suggested_fix, evidence, branch, pr_number, commit_sha, suppression -- all have `skip_serializing_if = "Option::is_none"`. Confirmed.

3. **All Vec fields have skip_serializing_if:** Verified. tags, state_history, relationships, notes, edit_history -- all have `skip_serializing_if = "Vec::is_empty"`. Note: `locations` and `discovered_by` do NOT have this annotation (they're expected to always have at least one element). This is a refinement -- the pattern is "empty-able Vecs skip, identity Vecs don't."

4. **15 CLI handler functions:** Recount: handle_init, handle_record, handle_query, handle_update, handle_suppress, handle_rebuild_index, handle_record_batch, handle_export, handle_import, handle_stats, handle_update_fields, handle_add_note, handle_manage_tags, handle_sync, handle_mcp_capabilities = **15 confirmed**.

5. **24 MCP input types:** Confirmed from Pass 2 R2.

6. **`_pub` wrapper pattern on 4 methods:** Confirmed from git_store.rs grep: upsert_file_pub, read_file_pub, list_directory_pub, remove_file_pub.

7. **RuleStatus omits #[non_exhaustive]:** Verified from Pass 2 R1 (rule.rs:73-81). RuleStatus derives Default but NOT non_exhaustive. Confirmed deliberate (semantically closed: Active/Deprecated/Experimental).

### Correction: Vec skip_serializing_if Is Not Universal

R1 claimed "every Vec field has skip_serializing_if = Vec::is_empty." This is mostly true but with two exceptions:
- `locations: Vec<Location>` -- NO skip_serializing_if (always expected to have content)
- `discovered_by: Vec<AgentRecord>` -- NO skip_serializing_if (always expected to have content)

The actual pattern is: **empty-able auxiliary Vecs skip; identity/required Vecs don't.** This distinction is architecturally meaningful -- locations and discovered_by are core identity data that should always be present.

## MCP Tool Composition Patterns (New)

Analyzing how the 23 tool methods compose domain operations:

### Pattern A: Direct CRUD (8 tools)
```
store -> load/save -> format -> CallToolResult::success
```
Tools: init_findings, export_findings, import_findings, sync_findings, rebuild_index, get_rule, list_rules, delete_rule

### Pattern B: Identity-Resolved Operation (7 tools)
```
store -> load_all -> build SessionIdMapper -> resolve ID (UUID or short) -> load specific finding -> mutate -> save
```
Tools: update_status, get_finding_context, suppress_finding, update_finding, add_note, manage_tags, update_batch_status

### Pattern C: Record with Dedup (2 tools)
```
store -> load_all -> build FindingIdentityResolver -> compute fingerprint -> resolve identity -> save
  -> build RuleMatcher -> match rule -> check scope
```
Tools: record_finding, record_batch

### Pattern D: Query with Filters (1 tool)
```
store -> load_all -> check_expiry_and_reopen -> apply CLI filters -> parse TallyQL -> evaluate -> sort -> limit -> format
```
Tools: query_findings

### Pattern E: Registry Operations (5 tools)
```
store -> RuleStore::load_all_rules -> build RuleMatcher -> [operation] -> RuleStore::save
```
Tools: create_rule, update_rule, search_rules, add_rule_example, migrate_rules

**Observation:** Patterns B and C both call `load_all()`, meaning every ID-resolved operation and every record operation loads ALL findings. This is the O(N) bottleneck noted in the architecture analysis.

## Test Helper Conventions (New)

### tests/cli_common/mod.rs
Provides shared setup and assertion utilities:
- `setup_repo()` -- creates temp dir with git init + tally init
- `setup_mcp()` -- creates TallyMcpServer for in-process tests  
- `tally_cmd()` -- creates assert_cmd Command for subprocess tests
- `record_finding()` -- convenience wrapper for recording test findings

### Property Test Strategy Conventions
- Custom `Arbitrary` implementations for domain types (Severity, LifecycleState)
- Small state spaces: property tests enumerate all enum variants
- Bounded depth: FilterExpr trees limited to depth 4
- Focus on invariants: idempotency, roundtrips, no-panic, equivalence

## Refined Pattern Consistency Assessment

| Pattern | Scope | Consistency | Exceptions |
|---------|-------|-------------|------------|
| `#[tracing::instrument(skip_all)]` | CLI handlers | 13/15 (87%) | handle_mcp_capabilities, handle_manage_tags(?) |
| `#[tracing::instrument(skip_all)]` | Storage | 6/15 (40%) | Most _pub wrappers, branch_exists, git_context |
| `#[serde(skip_serializing_if)]` on Options | Finding/Rule | 100% | None |
| `#[serde(skip_serializing_if)]` on Vecs | Finding/Rule | 5/7 (71%) | locations, discovered_by intentionally excluded |
| `#[non_exhaustive]` on growing enums | Domain enums | 6/7 (86%) | RuleStatus intentionally excluded |
| `handle_*` naming | CLI handlers | 100% | None |
| `*Input` naming | MCP DTOs | 100% | None |
| `to_mcp_err()` for error conversion | MCP tools | ~60% | Many inline McpError constructions for INVALID_REQUEST |
| Fresh store per operation | All interfaces | 100% | None |
| `#[must_use]` on pure getters | Service objects | ~30% | Inconsistent -- some have it, some don't |

## Additional Conventions

### Documentation Reference Convention
Two source files use a specific documentation pattern:
```rust
//! Deep research (Mar 2026) confirmed:
//! - [specific API finding]
//! - [specific behavior confirmed]
```
Found in: `git_store.rs` (TreeUpdateBuilder, one-file-per-finding, ErrorCode::Locked), `semantic.rs` (TextEmbedding API, sync-only, model details).

This "deep research confirmed" pattern documents external API decisions that were validated through research, reducing future investigation time.

### Comment Motivation Pattern (from SOUL.md)
Good comments explain WHY, not WHAT:
```rust
// One file per finding — git auto-merges new files without conflicts
// git2::Repository is not Send/Sync, so we open the repo fresh per tool call
```

### Const vs Static Convention
- `const` used for all module-level values (strings, numbers, arrays)
- No `static` items in the codebase
- `&'static str` appears as type in SessionIdMapper counters (severity prefixes)

## Delta Summary
- New items added: MCP tool composition patterns (5 categories covering all 23 tools), test helper conventions, "deep research confirmed" documentation pattern, const vs static convention, Vec skip_serializing_if refinement
- Existing items refined: to_mcp_err correction (error pattern is dual: free function + inline), Vec skip_serializing_if not universal (locations/discovered_by excluded intentionally), #[must_use] inconsistency noted
- Remaining gaps: None significant

## Novelty Assessment
Novelty: NITPICK
The MCP tool composition patterns (5 categories) are a useful decomposition but they describe combinations of domain operations already documented in other passes. The Vec skip_serializing_if refinement is an accuracy improvement. The "deep research confirmed" documentation pattern is a style finding that doesn't change the spec model.

## Convergence Declaration
Pass 5 has converged -- conventions are comprehensively documented. All naming, serde, derive, error handling, tracing, visibility, and test patterns cataloged with consistency assessments. The MCP tool composition patterns complete the behavioral picture.

## State Checkpoint
```yaml
pass: 5
round: 2
status: complete
files_scanned: 30
timestamp: 2026-04-14T01:15:00Z
novelty: NITPICK
```
