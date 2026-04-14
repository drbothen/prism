# Pass 3 Deep: Behavioral Contracts -- Round 1

Contracts are numbered as BC-S.SS.NNN where S=subsystem, SS=sub-area, NNN=sequence.

## Subsystem 1: Model / Finding

### BC-1.01.001: Finding field editing enforces immutability boundary

**Preconditions:**
- Finding exists
- field parameter is a string

**Postconditions:**
- If field is in EDITABLE_FIELDS ["title", "description", "suggested_fix", "evidence", "severity", "category", "tags"]: field is updated, FieldEdit appended to edit_history with old_value, new_value, timestamp, agent_id. updated_at is refreshed.
- If field is NOT in EDITABLE_FIELDS: returns TallyError::InvalidInput with message listing editable fields.
- UUID, content_fingerprint, rule_id, status, created_at are NEVER modified by edit_field().

**Error Cases:**
- Non-editable field name -> `TallyError::InvalidInput("field 'X' is not editable (editable: title, description, ...)")`
- title/description/category with non-string JSON value -> `TallyError::InvalidInput("X must be a string")`
- severity with non-string value -> `TallyError::InvalidInput("severity must be a string")`
- severity with invalid string -> `TallyError::InvalidInput("invalid severity: 'X' (valid: critical, ...)")`
- tags with non-string/non-array value -> `TallyError::InvalidInput("tags must be a string or array of strings")`

**Tags special behavior:** Accepts either a JSON array of strings or a comma-separated string. Both replace the entire tags list (not append).

**Evidence:** `tests/property_edit.rs` (property tests: arbitrary_field_edit_roundtrips, edit_field_never_modifies_uuid, edit_field_always_increments_history), `tests/cli_mutability_test.rs`
**Confidence:** HIGH

### BC-1.01.002: Finding note append is unconditional

**Preconditions:**
- Finding exists (mutable reference)
- text and agent_id are any strings

**Postconditions:**
- Note appended to notes vec with text, Utc::now() timestamp, agent_id
- updated_at refreshed
- No validation on text content (empty string allowed)

**Evidence:** `tests/property_edit.rs::arbitrary_note_text_roundtrips` (property test: any text roundtrips through serde)
**Confidence:** HIGH

### BC-1.01.003: Finding serialization roundtrip preserves all fields

**Preconditions:**
- Finding with fully populated fields

**Postconditions:**
- `serde_json::to_string` -> `serde_json::from_str` produces identical field values
- Optional fields with None values are omitted from JSON (skip_serializing_if)
- Empty Vec fields are omitted from JSON
- Default values are populated on deserialization of legacy files missing new fields

**Evidence:** `tests/model_test.rs::finding_serialization_roundtrip`
**Confidence:** HIGH

## Subsystem 1.02: State Machine

### BC-1.02.001: Lifecycle state transition validation

**Preconditions:**
- LifecycleState instance (from)
- Target LifecycleState (to)

**Postconditions:**
- `can_transition_to(target)` returns true if and only if target is in the static allowed_transitions() slice for the current state
- 26 total valid transitions (see transition table in Pass 2 deep)
- Closed state has empty allowed_transitions (terminal)

**Error behavior (at caller level, not in LifecycleState itself):**
- When caller detects invalid transition: `TallyError::InvalidTransition { from, to, valid: from.allowed_transitions().to_vec() }`
- The error message lists ALL valid target states

**Evidence:** `tests/model_test.rs` -- 28 positive transition tests (26 direct + 2 chain), 11 negative tests
**Confidence:** HIGH

### BC-1.02.002: Self-transition is always invalid

**Preconditions:** Any LifecycleState

**Postconditions:** `state.can_transition_to(state)` returns false for all 10 states

**Evidence:** `tests/model_test.rs::self_transition_invalid` -- iterates LifecycleState::all()
**Confidence:** HIGH

### BC-1.02.003: Closed is permanently terminal

**Preconditions:** Finding in Closed state

**Postconditions:** `can_transition_to(X)` returns false for ALL states including Open, Reopened, Closed

**Evidence:** `tests/model_test.rs::closed_to_anything_invalid` and `closed_still_terminal` -- both iterate all states
**Confidence:** HIGH

### BC-1.02.004: LifecycleState string roundtrip

**Preconditions:** Any LifecycleState variant

**Postconditions:**
- Display produces snake_case string: "open", "acknowledged", "in_progress", etc.
- FromStr parses case-insensitively, accepts both hyphens and underscores ("in-progress" == "in_progress")
- Display -> FromStr roundtrip is identity (property tested)

**Error Cases:**
- Unknown string -> `Err("invalid lifecycle state: 'X' (valid: open, acknowledged, ...)")`

**Evidence:** `tests/model_test.rs::lifecycle_from_str_valid`, `lifecycle_from_str_invalid`, `tests/property_identity.rs::proptest_lifecycle_display_roundtrip`
**Confidence:** HIGH

### BC-1.02.005: Severity string roundtrip (with case asymmetry)

**Preconditions:** Any Severity variant

**Postconditions:**
- Display produces UPPERCASE: "CRITICAL", "IMPORTANT", "SUGGESTION", "TECH_DEBT"
- FromStr is case-insensitive and accepts variants: "tech_debt", "tech-debt", "techdebt"
- Display -> lowercase -> FromStr roundtrip is identity (property tested)
- Note: Direct Display -> FromStr is NOT guaranteed because Display is uppercase and FromStr expects lowercase

**Error Cases:**
- Unknown string -> `Err("invalid severity: 'X' (valid: critical, important, suggestion, tech_debt)")`
- "high", "low", empty string -> all rejected

**Evidence:** `tests/model_test.rs::severity_from_str_valid`, `severity_from_str_invalid`, `tests/property_identity.rs::proptest_severity_display_roundtrip`
**Confidence:** HIGH

## Subsystem 1.03: Identity

### BC-1.03.001: Fingerprint is deterministic

**Preconditions:**
- Location with file_path, line_start, line_end
- rule_id string

**Postconditions:**
- `compute_fingerprint(loc, rule_id)` always returns same value for same inputs
- Format: "sha256:" followed by 64 hex characters (total length 71)
- Formula: SHA-256 of `"{file_path}:{line_start}-{line_end}:{rule_id}"`
- Only primary location is used (Location role is ignored in computation)
- Empty file_path and empty rule_id are valid inputs (produce valid hashes)
- u32::MAX line numbers produce valid hashes

**Evidence:** `tests/identity_test.rs` -- 7 fingerprint tests including determinism, component variation, empty inputs, max values
**Confidence:** HIGH

### BC-1.03.002: Fingerprint varies on each input component

**Preconditions:** Two Locations that differ in exactly one component

**Postconditions:**
- Different file_path -> different fingerprint
- Different line_start -> different fingerprint
- Different line_end -> different fingerprint
- Different rule_id -> different fingerprint

**Evidence:** `tests/identity_test.rs::fingerprint_changes_with_file`, `fingerprint_changes_with_line`, `fingerprint_changes_with_rule`, `fingerprint_line_end_matters`
**Confidence:** HIGH

### BC-1.03.003: Identity resolution three-priority algorithm

**Preconditions:**
- FindingIdentityResolver built from existing findings
- New finding with fingerprint, file_path, line_start, rule_id

**Postconditions:**
- Priority 1 (exact fingerprint match): Returns `ExistingFinding { uuid }` -- confidence 1.0
- Priority 2 (proximity match): Same file + same rule + within proximity_threshold lines -> `RelatedFinding { uuid, distance }`
- Priority 3 (no match): Returns `NewFinding`
- Distance is `abs_diff(new_line_start, existing_line_start)`
- Proximity at exact threshold (distance == threshold) -> RelatedFinding
- Proximity at threshold+1 -> NewFinding
- Different file, same rule, close lines -> NewFinding (file must match)
- Different rule, same file, same line -> NewFinding (rule must match)
- Empty resolver -> always NewFinding

**Evidence:** `tests/identity_test.rs` -- resolve_existing_by_fingerprint, resolve_related_by_proximity, resolve_new_when_distant, resolve_new_when_different_file, resolve_new_when_different_rule, resolve_empty_resolver_returns_new, resolver_proximity_at_boundary
**Confidence:** HIGH

### BC-1.03.004: Secondary locations are not indexed for proximity

**Preconditions:**
- Existing finding with Primary location at line 100 and Secondary location at line 10

**Postconditions:**
- New finding at line 10 with same rule is NOT matched as RelatedFinding
- Only Primary locations participate in proximity matching

**Evidence:** `tests/identity_test.rs::resolver_secondary_location_not_indexed`
**Confidence:** HIGH

### BC-1.03.005: Primary location extraction fallback

**Preconditions:** Vec of Locations

**Postconditions:**
- Returns first Location with role == Primary
- If no Primary role: returns first Location (regardless of role)
- If empty vec: returns None

**Evidence:** `tests/identity_test.rs::primary_location_finds_primary`, `primary_location_falls_back_to_first`, `primary_location_empty_returns_none`
**Confidence:** HIGH

## Subsystem 2: Session

### BC-2.01.001: Short ID assignment uses severity prefix and auto-incrementing counter

**Preconditions:** SessionIdMapper instance, UUID, Severity

**Postconditions:**
- First Critical UUID gets "C1", second gets "C2", etc.
- First Important gets "I1", first Suggestion gets "S1", first TechDebt gets "TD1"
- Each severity has independent counter (C1, I1, S1, TD1 can all exist simultaneously)
- Same UUID always returns same short ID (idempotent)

**Evidence:** `tests/session_test.rs` -- assign_critical_gets_c_prefix, sequential_same_severity_increments, different_severities_have_independent_counters, same_uuid_returns_same_short_id
**Confidence:** HIGH

### BC-2.01.002: Short ID resolution is case-insensitive

**Preconditions:** Short ID assigned to a UUID

**Postconditions:**
- resolve("c1") == resolve("C1") == Some(uuid)
- Stored format is "C1" (uppercase prefix)

**Evidence:** `tests/session_test.rs::resolve_case_insensitive`, `mapper_case_preserved_in_stored_format`
**Confidence:** HIGH

### BC-2.01.003: resolve_id accepts both UUID strings and short IDs

**Preconditions:** String input that may be a UUID or short ID

**Postconditions:**
- UUID string -> parsed and returned directly (no mapper lookup needed)
- Short ID string -> resolved via mapper
- Invalid string (neither UUID nor known short ID) -> None
- UUID parsing is attempted FIRST, then short ID

**Evidence:** `tests/session_test.rs::resolve_id_accepts_uuid_string`, `resolve_id_accepts_short_id`, `resolve_id_invalid_input_returns_none`
**Confidence:** HIGH

### BC-2.01.004: Mapper instances are independent

**Preconditions:** Two separate SessionIdMapper instances

**Postconditions:**
- Each has its own counter state
- Both assign "C1" to their first Critical UUID
- resolve("C1") on mapper_a returns mapper_a's UUID (not mapper_b's)

**Evidence:** `tests/session_test.rs::mapper_separate_instances_independent`
**Confidence:** HIGH

## Subsystem 3: Registry

### BC-3.01.001: Rule ID normalization transforms

**Preconditions:** Input string

**Postconditions (all applied in sequence):**
1. Lowercase the entire string
2. Replace underscores with hyphens
3. Replace spaces with hyphens
4. Strip agent namespace prefix (text before first ":")
5. Trim leading/trailing hyphens
6. Collapse consecutive hyphens

**Validation after normalization:**
- 2-64 characters
- First and last character must be [a-z0-9]
- Middle characters must be [a-z0-9-]

**Error Cases:**
- Empty string -> TallyError::InvalidInput
- Single character after normalization -> TallyError::InvalidInput
- Contains non-[a-z0-9-] characters after normalization (e.g., "/", ".", "\\") -> TallyError::InvalidInput
- Over 64 characters -> TallyError::InvalidInput
- All hyphens (empty after trim) -> TallyError::InvalidInput
- Namespace-only ("dclaude:") -> TallyError::InvalidInput
- Namespace with single char ("dclaude:a") -> TallyError::InvalidInput

**Evidence:** `tests/registry_normalize_test.rs` -- 26 tests covering all transforms, boundary conditions, idempotency
**Confidence:** HIGH

### BC-3.01.002: Normalization is idempotent

**Preconditions:** Any input that normalizes successfully

**Postconditions:** `normalize(normalize(input)) == normalize(input)` -- verified for simple, namespaced, consecutive hyphens, mixed transforms, and no- prefix inputs

**Evidence:** `tests/registry_normalize_test.rs` -- 5 idempotency tests, `tests/property_registry.rs::normalize_is_idempotent` (property test)
**Confidence:** HIGH

### BC-3.01.003: Semantic prefixes are preserved

**Preconditions:** Rule IDs starting with "no-", "check-", "disallow-"

**Postconditions:** These prefixes are NOT stripped -- they carry semantic meaning. Only agent namespace prefixes (before ":") are stripped.

**Evidence:** `tests/registry_normalize_test.rs::no_prefix_preserved`, `check_prefix_preserved`, `disallow_prefix_preserved`
**Confidence:** HIGH

### BC-3.02.001: Rule matching pipeline stages 1-3 (exact + alias) short-circuit

**Preconditions:** RuleMatcher with registered rules

**Postconditions:**
- Stage 2 (exact match): confidence=1.0, method="exact", similar_rules is empty
- Stage 3 (alias): confidence=1.0, method="alias", canonical_id is the rule's canonical ID, similar_rules is empty
- Both stages return immediately without running stages 4-7

**Evidence:** `tests/registry_matcher_test.rs::exact_match_returns_confidence_1`, `alias_maps_to_canonical`, `full_pipeline_exact_match_short_circuits`, `full_pipeline_alias_short_circuits`
**Confidence:** HIGH

### BC-3.02.002: Rule matching stages 4-7 produce suggestions, never auto-match

**Preconditions:** Input rule ID that does NOT exactly or alias-match any rule

**Postconditions:**
- CWE cross-reference (stage 4): Adds SimilarRule with confidence=0.7, method="cwe"
- Jaro-Winkler (stage 5): Adds SimilarRule with JW score >= 0.6, method="jaro_winkler"
- Token Jaccard (stage 6): Adds SimilarRule with jaccard >= 0.5, method="token_jaccard"
- Final result: method="auto_registered", confidence=0.0, canonical_id=normalized input
- CRITICAL: Fuzzy matches NEVER auto-resolve. "rule-crit3" does not auto-match to "rule-crit1" despite high JW score.

**Evidence:** `tests/registry_matcher_test.rs::cwe_match_adds_suggestion`, `jaro_winkler_adds_suggestion`, `token_jaccard_adds_suggestion`, `auto_registration_for_unknown`, `jw_does_not_auto_match`
**Confidence:** HIGH

### BC-3.02.003: Normalization + exact/alias combined

**Preconditions:** Mixed-case, underscored, or namespaced rule ID

**Postconditions:**
- "Unsafe_Unwrap" normalizes to "unsafe-unwrap" -> exact match
- "dclaude:unsafe-unwrap" -> strip namespace -> exact match
- "Unwrap_Usage" -> normalize -> "unwrap-usage" -> alias match
- "sonnet:unwrap-usage" -> strip + alias match

**Evidence:** `tests/registry_matcher_test.rs::normalize_then_exact`, `agent_namespace_stripped_then_exact`, `normalize_underscore_and_mixed_case_for_alias`, `agent_namespace_stripped_then_alias`
**Confidence:** HIGH

### BC-3.02.004: ID namespace conflict detection

**Preconditions:** RuleMatcher with existing rules

**Postconditions:**
- New rule with canonical ID that matches an existing alias -> rejected
- New rule with alias that matches an existing canonical ID -> rejected
- New rule with alias claimed by another rule -> rejected
- Existing rule re-declaring its own aliases -> allowed
- Completely new rule with new aliases -> allowed

**Evidence:** `tests/registry_matcher_test.rs::alias_shadows_canonical_rejected`, `canonical_shadows_alias_rejected`, `alias_claimed_by_other_rule_rejected`, `check_id_namespace_accepts_valid_new_rule`, `check_id_namespace_allows_own_aliases`
**Confidence:** HIGH

### BC-3.02.005: Matcher never panics on arbitrary input

**Preconditions:** Any String input (including empty, unicode, very long)

**Postconditions:** `matcher.resolve()` returns Ok or Err, never panics

**Evidence:** `tests/property_registry.rs::matcher_never_panics`, `matcher_never_panics_with_cwe_and_desc`
**Confidence:** HIGH

### BC-3.03.001: Scope enforcement with glob patterns

**Preconditions:** Rule with optional RuleScope (include/exclude globs)

**Postconditions:**
- No scope (None) -> file is in scope (returns None)
- Include patterns only -> file must match at least one include pattern
- Exclude patterns only -> file must NOT match any exclude pattern
- Both include and exclude -> exclude wins (checked first). File matching exclude is rejected even if it matches include.
- Out-of-scope returns Some(warning_message) containing rule_id and file_path

**Scope enforcement is ADVISORY, not blocking.** check_scope returns a warning string, not an error. Callers decide whether to block recording.

**Evidence:** `tests/registry_scope_test.rs` -- 8 tests covering all fixture cases from spec
**Confidence:** HIGH

## Subsystem 4: Query Engine

### BC-4.01.001: TallyQL parser produces correct AST for simple comparisons

**Preconditions:** Valid TallyQL string

**Postconditions:**
- `severity = critical` -> `Comparison { field: "severity", op: Eq, value: Enum("critical") }`
- `status != closed` -> `Comparison { field: "status", op: Ne, value: Enum("closed") }`
- `created_at > 7d` -> `Comparison { field: "created_at", op: Gt, value: Duration(7*86400s) }`
- `title CONTAINS "unwrap"` -> `StringMatch { field: "title", op: Contains, value: "unwrap" }`
- `HAS suggested_fix` -> `Has("suggested_fix")`
- `MISSING evidence` -> `Missing("evidence")`
- Keywords are case-insensitive: CONTAINS, Contains, contains all parse

**Evidence:** `tests/query_parser_test.rs` -- 6+ parser tests for each operator type
**Confidence:** HIGH

### BC-4.02.001: Evaluator boolean logic matches standard semantics

**Preconditions:** FilterExpr tree, Finding

**Postconditions:**
- AND(a, b) == evaluate(a) && evaluate(b)
- OR(a, b) == evaluate(a) || evaluate(b)
- NOT(a) == !evaluate(a)
- NOT(NOT(a)) == evaluate(a) (double negation identity)

**Evidence:** `tests/property_query.rs` -- proptest for and_is_conjunction, or_is_disjunction, not_is_negation, double_negation_is_identity
**Confidence:** HIGH

### BC-4.02.002: Evaluator never panics on arbitrary expression trees

**Preconditions:** Any FilterExpr tree (up to depth 4)

**Postconditions:** evaluate() returns bool, never panics

**Evidence:** `tests/property_query.rs::evaluate_never_panics`
**Confidence:** HIGH

### BC-4.02.003: Severity comparison uses ordinal ordering

**Preconditions:** Severity comparison expression

**Postconditions:**
- Critical(3) > Important(2) > Suggestion(1) > TechDebt(0)
- "severity > important" is true for Critical findings
- "severity < important" is false for Critical findings
- Status comparison supports ONLY equality (= and !=), not ordering (> < >= <= return false)

**Evidence:** `tests/query_eval_test.rs::severity_greater_than_important_is_true`, `severity_less_than_important_is_false`
**Confidence:** HIGH

### BC-4.02.004: Multi-value field matching uses any-match semantics

**Preconditions:** Finding with multiple locations, tags, or agents

**Postconditions:**
- For `=`: true if ANY value matches (e.g., file = "src/api/handler.rs" matches even if secondary location is different)
- For `!=`: true if NO value matches (ALL must differ)
- CONTAINS/STARTSWITH/ENDSWITH on multi-value fields: any-match
- All string comparisons are case-insensitive

**Evidence:** `tests/query_eval_test.rs::file_contains_api_is_true` (matches either location)
**Confidence:** HIGH

## Subsystem 5: Storage

### BC-5.01.001: Init creates orphan branch idempotently

**Preconditions:** Valid git repository

**Postconditions:**
- Creates "findings-data" branch as orphan (no shared ancestor with main)
- Branch contains: schema.json, findings/.gitkeep, rules/.gitkeep, .gitattributes
- .gitattributes sets `index.json merge=ours`
- Does NOT modify HEAD or working tree
- If branch already exists: returns Ok, ensures rules/ directory exists (upgrade path)

**Evidence:** `tests/storage_test.rs::init_creates_orphan_branch`
**Confidence:** HIGH

### BC-5.01.002: Save and load finding roundtrip

**Preconditions:** Initialized store, valid Finding

**Postconditions:**
- save_finding creates `findings/<uuid>.json` as a new commit
- load_finding reads back identical data
- UUID in filename matches finding.uuid
- Existing finding with same UUID is overwritten (update)

**Evidence:** `tests/storage_test.rs` (storage roundtrip tests)
**Confidence:** HIGH

### BC-5.01.003: Load all skips malformed entries

**Preconditions:** Branch with mix of valid and malformed JSON files

**Postconditions:**
- Valid findings are returned
- Malformed JSON files are logged at warn level and skipped
- .gitkeep and non-.json files are silently skipped
- No partial failure -- valid findings are always returned even if some are malformed

**Evidence:** `src/storage/git_store.rs::load_all()` code + `tests/storage_test.rs`
**Confidence:** HIGH (from code + test)

## Subsystem 6: Error

### BC-6.01.001: TallyError Display includes actionable context

**Preconditions:** Various TallyError variants

**Postconditions:**
- NotFound: message includes the UUID
- InvalidTransition: message includes from state, to state, and all valid target states
- BranchNotFound: message includes "run `tally init`"
- InvalidSeverity: message includes the invalid value
- NoLocation: message says "at least one location required"

**Evidence:** `tests/error_test.rs` -- 5 tests verifying error message content
**Confidence:** HIGH

## Subsystem 7: E2E Workflows

### BC-7.01.001: Full lifecycle workflow (Open -> Acknowledged -> InProgress -> Resolved -> Closed)

**Preconditions:** Initialized tally repo

**Postconditions:**
- Record creates finding in Open status
- Each update transitions to next state with reason and optional commit_sha
- Final state is Closed with 4 transitions in state_history
- state_history records from, to, reason, commit_sha for each transition
- Attempting to transition FROM Closed -> error with "invalid state transition"

**Evidence:** `tests/e2e_lifecycle_test.rs::e2e_full_finding_lifecycle`
**Confidence:** HIGH

### BC-7.01.002: Multi-agent deduplication

**Preconditions:** Two agents record finding with identical file, line, rule

**Postconditions:**
- First record: status="created", new UUID assigned
- Second record: status="deduplicated", same UUID returned
- discovered_by contains both agents
- Only one finding exists in the store

**Evidence:** `tests/e2e_lifecycle_test.rs::e2e_multi_agent_dedup`
**Confidence:** HIGH

### BC-7.01.003: Suppression with auto-reopen on expiry

**Preconditions:** Finding in Open status, suppress with past expiry date

**Postconditions:**
- Status transitions to Suppressed
- On next query: expired suppression triggers auto-reopen to Open
- state_history shows: Open -> Suppressed, then Suppressed -> Open (system-initiated)

**Evidence:** `tests/e2e_lifecycle_test.rs::e2e_suppression_lifecycle`
**Confidence:** HIGH

## Summary of Behavioral Contract Coverage

| Subsystem | Contracts | Coverage Source | Confidence |
|-----------|-----------|----------------|------------|
| Model/Finding | 3 | Unit + Property | HIGH |
| State Machine | 5 | Unit + Property + E2E | HIGH |
| Identity | 5 | Unit + Property | HIGH |
| Session | 4 | Unit | HIGH |
| Registry/Normalize | 3 | Unit + Property | HIGH |
| Registry/Matcher | 5 | Unit + Property | HIGH |
| Registry/Scope | 1 | Unit | HIGH |
| Query/Parser | 1 | Unit | HIGH |
| Query/Evaluator | 4 | Unit + Property | HIGH |
| Storage | 3 | Integration | HIGH |
| Error | 1 | Unit | HIGH |
| E2E Workflows | 3 | E2E CLI | HIGH |

**Total contracts extracted:** 38

## Gaps (Behaviors with No or Low Test Coverage)

1. **MCP server tool contracts** -- The 23 MCP tools in server.rs have test coverage in mcp_test.rs, mcp_unit_test.rs, mcp_enhanced_test.rs, and e2e_mcp_workflow_test.rs, but these have not yet been extracted into BC format.

2. **Sync operation contracts** -- Sync (fetch + merge + push + retry) is complex with multiple code paths (fast-forward, three-way merge, rule conflict resolution, push retry). Tests exist in storage_test.rs but contracts not yet extracted.

3. **Export format contracts** -- CSV and SARIF export (cli/export.rs) have tests in cli_export_test.rs but contracts not yet extracted.

4. **Parser security limits** -- MAX_QUERY_LENGTH (8KB) and MAX_NESTING_DEPTH (64) are mentioned in broad sweep but not yet verified from source or linked to specific test evidence.

5. **Rule store CRUD contracts** -- RuleStore save/load/delete/load_all operations have tests in cli_rule_test.rs and e2e_rule_registry_test.rs but not yet extracted.

6. **Batch operations with partial success** -- Described in broad sweep BC-007 but evidence needs verification from mcp_unit_test.rs source.

## Delta Summary
- New items added: 38 contracts with precise pre/postconditions and test file evidence
- Existing items refined: Broad sweep BC-001 through BC-009 decomposed into subsystem-specific contracts with verification
- Remaining gaps: MCP tool contracts, sync contracts, export contracts, parser security limits, rule store CRUD, batch operations

## Novelty Assessment
Novelty: SUBSTANTIVE
The decomposition from 9 broad contracts to 38 specific ones, with subsystem numbering and test file evidence, fundamentally changes how one would spec the system. Key discoveries: scope enforcement is advisory not blocking, severity Display/FromStr asymmetry, identity resolution indexes only primary locations, tags accept both array and comma-separated string.

## Convergence Declaration
Another round needed -- MCP tool contracts (23 tools), sync contracts, export contracts, and parser security limits remain unextracted. These are substantial behavioral surfaces.

## State Checkpoint
```yaml
pass: 3
round: 1
status: complete
files_scanned: 22
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
next_action: Round 2 -- extract MCP tool contracts, sync contracts, parser limits, export contracts
```
