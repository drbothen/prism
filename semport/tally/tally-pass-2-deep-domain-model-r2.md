# Pass 2 Deep: Domain Model -- Round 2

## Hallucination Audit

Reviewing Round 1 claims against source code:

1. **Transition count "26":** Re-verified. Open(5) + Acknowledged(4) + InProgress(3) + Resolved(2) + FalsePositive(2) + WontFix(2) + Deferred(3) + Suppressed(3) + Reopened(2) + Closed(0) = 26. Confirmed correct.

2. **State machine enforcement gap (pub field):** Confirmed. `src/model/finding.rs:76` has `pub status: LifecycleState`. No setter method, no runtime guard on direct mutation. Enforcement is at CLI handler level and MCP server level.

3. **FieldType enum from query/fields.rs:** Confirmed at line 38-53. Seven variants as described.

4. **SuppressionType::InlineComment data variant:** Confirmed at line 323: `InlineComment { pattern: String }`.

5. **Severity ordinal as free function:** Confirmed. `eval.rs:406-413` defines `const fn severity_ordinal(severity: Severity) -> u8`. Not a method on Severity.

All Round 1 claims verified. No hallucinations detected.

## MCP Input Types (24 structs)

All defined in `src/mcp/server.rs:42-478`. All derive `Debug, Deserialize, JsonSchema`. None derive Serialize (they are input-only). Per-field descriptions via `#[schemars(description = "...")]`.

### Finding Operations

| Input Struct | Tool | Fields |
|-------------|------|--------|
| RecordFindingInput | record_finding | file_path, line_start, line_end?, severity, title, rule_id, description?, agent?, locations?, suggested_fix?, evidence?, category?, tags?, pr_number?, session_id?, related_to?, relationship_type? |
| LocationInput | (nested in RecordFindingInput) | file_path, line_start, line_end?, role? |
| QueryFindingsInput | query_findings | status?, severity?, file?, rule?, limit?, tag?, filter?, sort?, since?, before?, agent?, category?, text? |
| UpdateFindingInput | update_finding | finding_id, title?, description?, suggested_fix?, evidence?, severity?, category?, tags?, agent? |
| AddNoteInput | add_note | finding_id, note, agent? |
| TagInput | manage_tags | finding_id, tags, agent? |
| UpdateStatusInput | update_status | finding_id, new_status, reason?, agent?, commit_sha?, related_to?, relationship? |
| GetContextInput | get_finding_context | finding_id |
| SuppressFindingInput | suppress_finding | finding_id, reason, expires_at?, agent?, suppression_type?, suppression_pattern? |
| RecordBatchInput | record_batch | findings (Vec\<BatchFindingInput\>), agent?, pr_number?, session_id? |
| BatchFindingInput | (nested in RecordBatchInput) | file_path, line_start, line_end?, severity, title, rule_id, description?, suggested_fix?, evidence?, category?, tags?, pr_number?, session_id? |
| UpdateBatchStatusInput | update_batch_status | finding_ids (Vec\<String\>), status, reason?, agent? |
| ExportFindingsInput | export_findings | format |
| ImportFindingsInput | import_findings | file_path |
| SyncFindingsInput | sync_findings | remote? |
| RebuildIndexInput | rebuild_index | include_rules? |

### Rule Registry Operations

| Input Struct | Tool | Fields |
|-------------|------|--------|
| CreateRuleInput | create_rule | rule_id, name, description, category?, severity_hint?, aliases?, cwe_ids?, tags?, scope_include?, scope_exclude? |
| GetRuleInput | get_rule | rule_id |
| SearchRulesInput | search_rules | query, method?, limit? |
| ListRulesInput | list_rules | category?, status? |
| UpdateRuleInput | update_rule | rule_id, name?, description?, status?, add_aliases?, remove_aliases?, add_cwe? |
| DeleteRuleInput | delete_rule | rule_id, reason |
| AddRuleExampleInput | add_rule_example | rule_id, example_type, language, code, explanation |
| MigrateRulesInput | migrate_rules | dry_run? |

### MCP Output Type

**ToolOutput** (`src/mcp/server.rs:480`): Derives `Debug, Serialize, Deserialize, JsonSchema`. Has fields: status, uuid?, message?, related_to?, and more with skip_serializing_if. This is the standard response wrapper.

### Key Observations

1. **Input types are NOT domain types.** They are MCP-specific DTOs. The MCP server converts them into domain operations (e.g., `RecordFindingInput` -> `Finding` construction + `GitFindingsStore::save_finding()`).

2. **All `finding_id` fields accept both UUID and short ID.** The resolution happens via `resolve_id_mcp()` in the server code, which loads ALL findings to build a SessionIdMapper.

3. **SuppressFindingInput has 3 suppression_type options:** "global" (default), "file", "inline". The "inline" type requires a `suppression_pattern` field.

4. **UpdateRuleInput uses additive aliases:** `add_aliases` and `remove_aliases` are separate fields (not a replacement list). This is different from Finding's tags editing which replaces entirely.

5. **DeleteRuleInput doesn't actually delete** -- it deprecates (note the description: "Rule ID to deprecate"). This is a soft delete via status change to Deprecated.

## Parser Constants (Verified from Source)

**Source:** `src/query/parser.rs:28-31`

| Constant | Value | Purpose | CWE |
|----------|-------|---------|-----|
| MAX_QUERY_LENGTH | 8192 (8 KB) | Prevents parser-level DoS | CWE-400 |
| MAX_NESTING_DEPTH | 64 | Prevents stack overflow from recursive expressions | CWE-674 |

**Implementation details:**
- MAX_QUERY_LENGTH: checked BEFORE parsing begins (byte length of input string)
- MAX_NESTING_DEPTH: tracked via `Rc<Cell<usize>>` counter incremented on each nested expression. Uses Rc+Cell (not Arc+AtomicUsize) because parsing is synchronous and never crosses await points.

## Storage Constants (Verified from Source)

**Source:** `src/storage/git_store.rs:23-29`

| Constant | Value | Purpose |
|----------|-------|---------|
| FINDINGS_BRANCH | "findings-data" | Default orphan branch name |
| FINDINGS_DIR | "findings" | Directory for finding JSON files |
| MAX_LOCK_RETRIES | 3 | Push retry attempts on lock contention |

## Registry Constants

**Source:** `src/registry/matcher.rs:28`

| Constant | Value | Purpose |
|----------|-------|---------|
| SUGGEST_THRESHOLD | 0.6 | Minimum JW score to include as suggestion |

**Source:** `src/registry/store.rs:12`

| Constant | Value | Purpose |
|----------|-------|---------|
| RULES_DIR | "rules" | Directory for rule JSON files |

## Query Field Registry (Verified from Source)

**Source:** `src/query/fields.rs:8-33`

**KNOWN_FIELDS (13):** severity, status, file, rule, title, description, suggested_fix, evidence, category, agent, tag, created_at, updated_at

**SORTABLE_FIELDS (7):** severity, status, created_at, updated_at, file, rule, title

**Note:** `file` maps to `locations[*].file_path`, `rule` maps to `rule_id`, `agent` maps to `discovered_by[*].agent_id`, `tag` maps to `tags[]`. These are TallyQL-specific aliases, not direct field names.

## Semantic Search Module (Feature-Gated)

**Source:** `src/registry/semantic.rs` (only compiled with `#[cfg(feature = "semantic-search")]`)

This module is feature-gated behind the `semantic-search` feature flag. It uses `fastembed` 5 for generating embeddings. The Rule.embedding and Rule.embedding_model fields exist on the Rule struct regardless of the feature flag, but the computation logic is gated.

## TallyMcpServer Entity

**Source:** `src/mcp/server.rs:33-38`
**Derives:** `Clone`

| Field | Type | Purpose |
|-------|------|---------|
| repo_path | String | Path to git repository |
| tool_router | ToolRouter\<Self\> | Generated tool dispatch table |
| prompt_router | PromptRouter\<Self\> | Generated prompt dispatch table |

This is a Clone type because rmcp requires it. The tool_router and prompt_router are generated by the `#[tool_router]` and `#[prompt_router]` macros.

## Credential Chain (Authentication Strategy)

**Source:** `src/storage/git_store.rs:38-97`

Not a domain type, but an important behavioral pattern:
1. Git credential helper (osxkeychain/GCM/store, gh auth setup-git)
2. GITHUB_TOKEN or GIT_TOKEN environment variable
3. SSH agent
4. SSH key files (~/.ssh/id_ed25519, id_rsa, id_ecdsa)

**Attempt counter:** Uses `Cell<u32>` with max 4 attempts to prevent libgit2's infinite retry loop.

## Complete Type Count Summary

| Category | Count | Examples |
|----------|-------|---------|
| Domain structs | 12 | Finding, Location, AgentRecord, Note, FieldEdit, etc. |
| Domain enums | 8 | LifecycleState, Severity, LocationRole, RelationshipType, etc. |
| Service objects | 4 | FindingIdentityResolver, SessionIdMapper, RuleMatcher, RuleStore |
| Storage types | 2 | GitFindingsStore, SyncResult |
| Query AST types | 6 | FilterExpr, CompareOp, StringOp, Value, SortSpec, FieldType |
| Error types | 2 | TallyError, TallyQLError |
| MCP Input DTOs | 24 | RecordFindingInput, QueryFindingsInput, etc. |
| MCP Output DTOs | 1 | ToolOutput |
| Result types | 2 | IdentityResolution, MatchResult |
| Registry types | 5 | Rule, RuleStatus, RuleScope, RuleExample, SimilarRule |
| MCP Server | 1 | TallyMcpServer |
| **Total** | **67** | |

## Delta Summary
- New items added: 25 (24 MCP Input DTOs + ToolOutput + parser constants + storage constants + registry constants + field registry mapping + credential chain + TallyMcpServer)
- Existing items refined: 3 (hallucination audit verified all Round 1 claims, query field aliases documented, semantic search module noted as feature-gated)
- Remaining gaps: None significant. The type catalog is now comprehensive.

## Novelty Assessment
Novelty: NITPICK
The 24 MCP Input types are important for completeness but are DTOs that mirror domain operations already cataloged. The parser and storage constants were mentioned in the broad sweep. The credential chain is infrastructure, not domain. No new domain entities, relationships, or invariants were discovered that would change how one would spec the system.

## Convergence Declaration
Pass 2 has converged -- findings are completeness items and refinements, not gaps. The domain model is fully cataloged: 67 types across domain, query, storage, MCP, and error subsystems.

## State Checkpoint
```yaml
pass: 2
round: 2
status: complete
files_scanned: 22
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
