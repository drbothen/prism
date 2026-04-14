# Pass 2 Deep: Domain Model -- Round 1

## Entity Catalog (Verified Against Source)

### 1. Finding (Aggregate Root)

**Source:** `src/model/finding.rs:27-114`
**Derives:** `Debug, Clone, Serialize, Deserialize`
**No trait implementations beyond serde** -- not `PartialEq`, not `Eq`, not `Hash`. This is deliberate: findings are identity-equal by UUID, not structurally equal.

| Field | Type | Serde Behavior | Mutability |
|-------|------|----------------|------------|
| schema_version | String | `default = "1.1.0"` | Set on creation |
| uuid | Uuid | `default` (nil UUID) | Immutable |
| content_fingerprint | String | `default` (empty) | Immutable |
| rule_id | String | `default` (empty) | Immutable |
| original_rule_id | Option\<String\> | `skip_serializing_if = "Option::is_none"` | Immutable |
| locations | Vec\<Location\> | `default` (empty vec) | Immutable |
| severity | Severity | `default` (Suggestion) | **Editable** via edit_field() |
| category | String | `default` (empty) | **Editable** via edit_field() |
| tags | Vec\<String\> | `skip_serializing_if = "Vec::is_empty"` | **Editable** via edit_field() |
| title | String | `default` (empty) | **Editable** via edit_field() |
| description | String | `default` (empty) | **Editable** via edit_field() |
| suggested_fix | Option\<String\> | `skip_serializing_if` | **Editable** via edit_field() |
| evidence | Option\<String\> | `skip_serializing_if` | **Editable** via edit_field() |
| status | LifecycleState | `default` (Open) | Via state machine only |
| state_history | Vec\<StateTransition\> | `skip_serializing_if = "Vec::is_empty"` | Append-only |
| discovered_by | Vec\<AgentRecord\> | `default` (empty vec) | Append-only |
| created_at | DateTime\<Utc\> | `default = Utc::now()` | Immutable |
| updated_at | DateTime\<Utc\> | `default = Utc::now()` | Auto-updated on mutation |
| repo_id | String | `default` (empty) | Immutable |
| branch | Option\<String\> | `skip_serializing_if` | Immutable |
| pr_number | Option\<u64\> | `skip_serializing_if` | Immutable |
| commit_sha | Option\<String\> | `skip_serializing_if` | Immutable |
| relationships | Vec\<FindingRelationship\> | `skip_serializing_if = "Vec::is_empty"` | Append-only |
| suppression | Option\<Suppression\> | `skip_serializing_if` | Set during suppress operation |
| notes | Vec\<Note\> | `skip_serializing_if = "Vec::is_empty"` | Append-only via add_note() |
| edit_history | Vec\<FieldEdit\> | `skip_serializing_if = "Vec::is_empty"` | Append-only (auto on edit_field()) |

**Key methods:**
- `edit_field(&mut self, field: &str, new_value: Value, agent_id: &str) -> Result<()>` -- gated by EDITABLE_FIELDS const
- `add_note(&mut self, text: &str, agent_id: &str)` -- unconditional append, updates updated_at

**Invariants:**
- Identity triple (uuid, content_fingerprint, rule_id) is immutable after creation
- status changes only through LifecycleState::can_transition_to() -- but NOTE: this validation is NOT enforced inside Finding itself. The Finding struct has `pub status: LifecycleState` -- enforcement happens at the caller level (CLI handlers, MCP server methods).
- edit_field() rejects non-editable fields with TallyError::InvalidInput
- EDITABLE_FIELDS is a const array: ["title", "description", "suggested_fix", "evidence", "severity", "category", "tags"]

### 2. Location (Value Object)

**Source:** `src/model/finding.rs:117-125`
**Derives:** `Debug, Clone, Serialize, Deserialize, PartialEq, Eq`

| Field | Type |
|-------|------|
| file_path | String |
| line_start | u32 |
| line_end | u32 |
| role | LocationRole |
| message | Option\<String\> |

This is a proper value object -- it derives PartialEq and Eq, meaning equality is structural. Used for fingerprint computation (file_path + line_start + line_end).

### 3. LocationRole (Enum, Value Object)

**Source:** `src/model/finding.rs:128-138`
**Derives:** `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize`
**Attributes:** `#[serde(rename_all = "snake_case")]`, `#[non_exhaustive]`

Variants: `Primary`, `Secondary`, `Context`

### 4. Severity (Enum, Value Object)

**Source:** `src/model/finding.rs:143-203`
**Derives:** `Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize`
**Attributes:** `#[serde(rename_all = "snake_case")]`, `#[non_exhaustive]`
**Default:** `Suggestion`

Variants: `Critical`, `Important`, `Suggestion` (default), `TechDebt`

**Behavioral methods:**
- `short_prefix() -> &'static str` -- C, I, S, TD
- `to_sarif_level() -> &'static str` -- error, warning, note, none
- `Display` impl -- CRITICAL, IMPORTANT, SUGGESTION, TECH_DEBT (uppercase)
- `FromStr` impl -- case-insensitive, accepts "tech_debt", "tech-debt", "techdebt"

**Ordering (from eval.rs:406-413, NOT on Severity itself):**
- Critical=3, Important=2, Suggestion=1, TechDebt=0
- This ordinal is defined as a free function `severity_ordinal()` in eval.rs, NOT as a method on Severity
- Severity does NOT impl Ord/PartialOrd -- ordering is context-specific to query evaluation

### 5. LifecycleState (Enum, State Machine)

**Source:** `src/model/state_machine.rs:9-27`
**Derives:** `Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize`
**Attributes:** `#[serde(rename_all = "snake_case")]`, `#[non_exhaustive]`
**Default:** `Open`

10 variants: `Open`, `Acknowledged`, `InProgress`, `Resolved`, `FalsePositive`, `WontFix`, `Deferred`, `Suppressed`, `Reopened`, `Closed`

**Complete transition table (verified from source, 26 valid transitions):**

| From | Valid Targets |
|------|--------------|
| Open | Acknowledged, InProgress, FalsePositive, Deferred, Suppressed |
| Acknowledged | InProgress, FalsePositive, WontFix, Deferred |
| InProgress | Resolved, WontFix, Deferred |
| Resolved | Reopened, Closed |
| FalsePositive | Reopened, Closed |
| WontFix | Reopened, Closed |
| Deferred | Open, Reopened, Closed |
| Suppressed | Open, Reopened, Closed |
| Reopened | Acknowledged, InProgress |
| Closed | (none -- terminal) |

**Total valid transitions:** 5+4+3+2+2+2+3+3+2+0 = 26

**Correction from broad sweep:** The broad sweep stated "24 valid transitions" in the test section header (Task 2.6 comment), but the actual count is 26. The test file has 28 individual test functions covering valid transitions (some test chains like Deferred->Reopened->InProgress). The model_test.rs header says "24 valid state transitions" but the actual code tests 28 distinct transitions in the positive section, with some being chain tests.

**Self-transition invariant:** No state can transition to itself (verified by `self_transition_invalid` test which iterates all 10 states).

**Key methods:**
- `allowed_transitions(&self) -> &'static [LifecycleState]` -- returns static slices
- `can_transition_to(&self, target: Self) -> bool` -- membership check
- `all() -> &'static [LifecycleState]` -- all 10 states
- `FromStr` impl -- case-insensitive, accepts hyphen or underscore (e.g., "in-progress", "in_progress")

### 6. StateTransition (Value Object)

**Source:** `src/model/state_machine.rs:126-140`
**Derives:** `Debug, Clone, Serialize, Deserialize`

| Field | Type | Serde |
|-------|------|-------|
| from | LifecycleState | `default` (Open) |
| to | LifecycleState | `default` (Open) |
| timestamp | DateTime\<Utc\> | `default = Utc::now()` |
| agent_id | String | `default` (empty) |
| reason | Option\<String\> | `skip_serializing_if` |
| commit_sha | Option\<String\> | `skip_serializing_if` |

### 7. AgentRecord (Value Object)

**Source:** `src/model/finding.rs:206-217`
**Derives:** `Debug, Clone, Serialize, Deserialize`

| Field | Type |
|-------|------|
| agent_id | String |
| session_id | String |
| detected_at | DateTime\<Utc\> |
| session_short_id | Option\<String\> |

### 8. FindingRelationship (Value Object)

**Source:** `src/model/finding.rs:220-227`
**Derives:** `Debug, Clone, Serialize, Deserialize`

| Field | Type |
|-------|------|
| related_finding_id | Uuid |
| relationship_type | RelationshipType |
| reason | Option\<String\> |
| created_at | DateTime\<Utc\> |

### 9. RelationshipType (Enum, Value Object)

**Source:** `src/model/finding.rs:230-246`
**Derives:** `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize`
**Attributes:** `#[serde(rename_all = "snake_case")]`, `#[non_exhaustive]`

6 variants: `DuplicateOf`, `Blocks`, `RelatedTo`, `Causes`, `DiscoveredWhileFixing`, `Supersedes`

**FromStr** accepts lowercase with hyphens or underscores, plus shortened forms ("duplicate" -> DuplicateOf, "related" -> RelatedTo).

### 10. Note (Value Object)

**Source:** `src/model/finding.rs:283-290`
**Derives:** `Debug, Clone, Serialize, Deserialize`

| Field | Type |
|-------|------|
| text | String |
| timestamp | DateTime\<Utc\> |
| agent_id | String |

### 11. FieldEdit (Value Object)

**Source:** `src/model/finding.rs:293-302`
**Derives:** `Debug, Clone, Serialize, Deserialize`

| Field | Type |
|-------|------|
| field | String |
| old_value | serde_json::Value |
| new_value | serde_json::Value |
| timestamp | DateTime\<Utc\> |
| agent_id | String |

### 12. Suppression (Value Object)

**Source:** `src/model/finding.rs:305-312`
**Derives:** `Debug, Clone, Serialize, Deserialize`

| Field | Type |
|-------|------|
| suppressed_at | DateTime\<Utc\> |
| reason | String |
| expires_at | Option\<DateTime\<Utc\>\> |
| suppression_type | SuppressionType |

### 13. SuppressionType (Enum, Value Object)

**Source:** `src/model/finding.rs:315-325`
**Derives:** `Debug, Clone, Serialize, Deserialize`
**Attributes:** `#[serde(rename_all = "snake_case")]`, `#[non_exhaustive]`

Variants: `Global`, `FileLevel`, `InlineComment { pattern: String }`

Note: InlineComment carries data (the pattern string). This is a tagged union in JSON.

### 14. Rule (Aggregate)

**Source:** `src/registry/rule.rs:7-66`
**Derives:** `Debug, Clone, Serialize, Deserialize`

| Field | Type | Purpose |
|-------|------|---------|
| id | String | Canonical ID (2-64 chars, lowercase alphanumeric + hyphens) |
| name | String | Human-readable |
| description | String | What the rule checks |
| category | String | Domain category |
| severity_hint | String | Suggested severity (not enforced) |
| tags | Vec\<String\> | Searchable tags |
| cwe_ids | Vec\<String\> | CWE identifiers |
| aliases | Vec\<String\> | Alternative names |
| scope | Option\<RuleScope\> | File scope restrictions |
| examples | Vec\<RuleExample\> | Bad/good code examples |
| suggested_fix_pattern | Option\<String\> | Regex or template |
| references | Vec\<String\> | URLs |
| related_rules | Vec\<String\> | Related rule IDs |
| created_by | String | Who created |
| created_at | DateTime\<Utc\> | When created |
| updated_at | DateTime\<Utc\> | Last modified |
| status | RuleStatus | Active/Deprecated/Experimental |
| finding_count | u64 | Cached count |
| embedding | Option\<Vec\<f32\>\> | For semantic search |
| embedding_model | Option\<String\> | Model used for embedding |

**Constructor:** `Rule::new(id, name, description)` -- sets defaults for everything else.

### 15. RuleStatus (Enum)

**Source:** `src/registry/rule.rs:73-81`
**Derives:** `Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize`
**Default:** `Active`

Variants: `Active`, `Deprecated`, `Experimental`

**Method:** `promotion_rank(self) -> u8` -- Deprecated=0, Experimental=1, Active=2

### 16. RuleScope (Value Object)

**Source:** `src/registry/rule.rs:121-129`
**Derives:** `Debug, Clone, Serialize, Deserialize`

| Field | Type |
|-------|------|
| include | Vec\<String\> (glob patterns) |
| exclude | Vec\<String\> (glob patterns) |

### 17. RuleExample (Value Object)

**Source:** `src/registry/rule.rs:132-143`
**Derives:** `Debug, Clone, Serialize, Deserialize`

| Field | Type |
|-------|------|
| example_type | String (renamed from "type") |
| language | String |
| code | String |
| explanation | String |

### 18. IdentityResolution (Enum, Result Type)

**Source:** `src/model/identity.rs:43-51`
**Derives:** `Debug, Clone, PartialEq, Eq`

Variants:
- `ExistingFinding { uuid: Uuid }` -- exact fingerprint match
- `RelatedFinding { uuid: Uuid, distance: u32 }` -- proximity match
- `NewFinding` -- no match

### 19. FindingIdentityResolver (Service Object)

**Source:** `src/model/identity.rs:59-64`
**No derives** -- has private fields, not serializable.

Internal state:
- `by_fingerprint: HashMap<String, Uuid>` -- fingerprint -> UUID
- `by_location: HashMap<(String, String), Vec<(u32, Uuid)>>` -- (file_path, rule_id) -> [(line_start, uuid)]

**Key behavioral detail discovered:** The location index uses ONLY the primary location. Secondary and Context locations are NOT indexed. This is verified by the `resolver_secondary_location_not_indexed` test.

### 20. TallyQL AST Types

#### FilterExpr (Enum, AST Node)

**Source:** `src/query/ast.rs:10-37`
**Derives:** `Debug, Clone, PartialEq`
**Attributes:** `#[non_exhaustive]`

7 variants:
- `Comparison { field: String, op: CompareOp, value: Value }`
- `And(Box<FilterExpr>, Box<FilterExpr>)`
- `Or(Box<FilterExpr>, Box<FilterExpr>)`
- `Not(Box<FilterExpr>)`
- `Has(String)`
- `Missing(String)`
- `StringMatch { field: String, op: StringOp, value: String }`
- `InList { field: String, values: Vec<Value> }`

#### CompareOp (Enum)

6 variants: `Eq`, `Ne`, `Gt`, `Lt`, `GtEq`, `LtEq`

#### StringOp (Enum)

3 variants: `Contains`, `StartsWith`, `EndsWith`

#### Value (Enum, AST Literal)

4 variants: `String(String)`, `Integer(i64)`, `Duration(Duration)`, `Enum(String)`

#### SortSpec (Struct)

Fields: `field: String`, `descending: bool`

### 21. FieldType (Enum, Type System Metadata)

**Source:** `src/query/fields.rs:38-53`
**Derives:** `Debug, Clone, Copy, PartialEq, Eq`

7 variants: `StringField`, `OrderedEnumField`, `EnumField`, `DateTimeField`, `OptionalStringField`, `ArrayStringField`, `AgentArrayField`

### 22. TallyQLError (Enum, Error Type)

**Source:** `src/query/error.rs:14-27`
**Derives via thiserror**
**Attributes:** `#[non_exhaustive]`

Single variant: `Parse { span: Range<usize>, expected: String, found: Option<String>, hint: Option<String> }`

### 23. TallyError (Enum, Error Type)

**Source:** `src/error.rs:6-42`
**Derives via thiserror**
**Attributes:** `#[non_exhaustive]`

9 variants:
- `NotFound { uuid: String }`
- `InvalidTransition { from: LifecycleState, to: LifecycleState, valid: Vec<LifecycleState> }`
- `Git(git2::Error)` -- via From
- `Serialization(serde_json::Error)` -- via From
- `BranchNotFound { branch: String }`
- `Io(std::io::Error)` -- via From
- `InvalidSeverity(String)`
- `InvalidInput(String)`
- `NoLocation`

### 24. SessionIdMapper (Service Object)

**Source:** `src/session.rs:15-22`

Internal state:
- `uuid_to_short: HashMap<Uuid, String>`
- `short_to_uuid: HashMap<String, Uuid>` -- keys stored uppercase
- `counters: HashMap<&'static str, u32>` -- per-prefix counter

**Key behavioral detail:** The counters map uses `&'static str` keys (the severity prefixes from `Severity::short_prefix()`). This means the counter keys are interned at compile time.

### 25. RuleMatcher (Service Object)

**Source:** `src/registry/matcher.rs:52-59`

Internal state:
- `rules: HashMap<String, Rule>` -- indexed by canonical ID
- `alias_index: HashMap<String, String>` -- alias -> canonical ID
- `cwe_index: HashMap<String, Vec<String>>` -- CWE ID -> list of canonical IDs

### 26. MatchResult (Result Type)

**Source:** `src/registry/matcher.rs:32-41`
**Derives:** `Debug, Clone`

| Field | Type | Purpose |
|-------|------|---------|
| canonical_id | String | Resolved or auto-registered ID |
| confidence | f64 | 1.0=exact/alias, 0.7=CWE, 0.0=auto-registered |
| method | String | "exact", "alias", "cwe", "jaro_winkler", "token_jaccard", "auto_registered" |
| similar_rules | Vec\<SimilarRule\> | Fuzzy suggestions |

### 27. SimilarRule (Value Object)

**Source:** `src/registry/matcher.rs:44-49`
**Derives:** `Debug, Clone, Serialize, Deserialize, schemars::JsonSchema`

Note: This is one of the few domain types that derives `JsonSchema`. This is because it appears in MCP tool output and needs schema generation.

### 28. GitFindingsStore (Repository/Service Object)

**Source:** `src/storage/git_store.rs:153-156`

Internal state:
- `repo: Repository` (git2)
- `branch_name: String` -- defaults to "findings-data"

### 29. SyncResult (Value Object)

**Source:** `src/storage/git_store.rs:140-150`
**Derives:** `Debug`

| Field | Type |
|-------|------|
| fetched | bool |
| merged | bool |
| pushed | bool |
| rules_merged | usize |

### 30. RuleStore (Stateless Service)

**Source:** `src/registry/store.rs:15`
**No fields** -- all methods are associated functions taking `&GitFindingsStore`.

## Aggregate Boundaries

### Finding Aggregate
- Root: Finding
- Contains: Vec\<Location\>, Vec\<AgentRecord\>, Vec\<FindingRelationship\>, Option\<Suppression\>, Vec\<Note\>, Vec\<FieldEdit\>, Vec\<StateTransition\>
- Identity: UUID v7
- Persistence boundary: One JSON file per finding (`findings/<uuid>.json`)

### Rule Aggregate
- Root: Rule
- Contains: Vec\<RuleExample\>, Option\<RuleScope\>
- Identity: Canonical rule ID string
- Persistence boundary: One JSON file per rule (`rules/<id>.json`)

### Session (Transient, Non-Persisted)
- Root: SessionIdMapper
- Lifetime: Single CLI invocation or MCP server session
- Not serialized

## Entity Relationship Map

```
Finding ---[1:N]--- Location (embedded)
Finding ---[1:N]--- AgentRecord (embedded)
Finding ---[1:N]--- FindingRelationship --[ref]--> Finding (by UUID)
Finding ---[0:1]--- Suppression (embedded)
Finding ---[0:N]--- Note (embedded)
Finding ---[0:N]--- FieldEdit (embedded)
Finding ---[0:N]--- StateTransition (embedded)
Finding ---[N:1]--- Rule (by rule_id string, NOT by object reference)
Rule    ---[1:N]--- RuleExample (embedded)
Rule    ---[0:1]--- RuleScope (embedded)
Rule    ---[0:N]--- Rule (by related_rules string IDs)
```

**Cross-aggregate references:**
- Finding.rule_id -> Rule.id (string reference, NOT foreign key constraint)
- Finding.relationships[].related_finding_id -> Finding.uuid (UUID reference, NOT enforced)
- Rule.related_rules[] -> Rule.id (string references, NOT enforced)

**No referential integrity enforcement** -- all cross-references are soft. A finding can reference a rule_id that doesn't exist in the registry (it will be auto-registered). A relationship can reference a finding UUID that has been deleted.

## Bounded Context Map

### Core Domain: Finding Lifecycle
- Entities: Finding, LifecycleState, StateTransition, Location, Severity
- Operations: Record, Update status, Edit field, Add note, Suppress
- Invariants: State machine transitions, fingerprint immutability, edit audit trail

### Supporting Domain: Identity Resolution
- Services: FindingIdentityResolver, compute_fingerprint()
- Purpose: Deduplication across agents and sessions
- Strategy: Content-addressable identity (fingerprint) + proximity matching

### Supporting Domain: Rule Registry
- Entities: Rule, RuleStatus, RuleScope, RuleExample
- Services: RuleMatcher, RuleStore, normalize_rule_id()
- Purpose: Canonical rule management, matching pipeline, scope enforcement

### Supporting Domain: Query Engine
- Types: FilterExpr, CompareOp, StringOp, Value, SortSpec, FieldType, TallyQLError
- Services: parse_tallyql(), evaluate(), apply_filters(), apply_sort()
- Purpose: Structured querying of findings

### Infrastructure: Git Storage
- Services: GitFindingsStore, SyncResult
- Purpose: Persistence on orphan branch, sync with remote

### Infrastructure: Session Management
- Services: SessionIdMapper
- Purpose: Human-friendly short IDs within a session

## Corrections to Broad Sweep

1. **Transition count:** The broad sweep and test comments say "24 valid transitions" but the actual count from the source code is 26 (5+4+3+2+2+2+3+3+2+0). The test file has 28 test functions in the "positive" section because some test transition chains.

2. **Severity::Display:** The broad sweep does not mention that Display output is UPPERCASE while FromStr accepts lowercase. This asymmetry is verified by the property test `proptest_severity_display_roundtrip` which explicitly converts Display output to lowercase before parsing.

3. **Finding has no PartialEq/Eq:** The broad sweep does not call out this deliberate omission. Finding implements Clone but NOT equality traits. IdentityResolution, in contrast, DOES implement PartialEq + Eq.

4. **SuppressionType::InlineComment carries data:** The broad sweep lists suppression types as "global/file/inline" but does not mention that InlineComment is a struct variant carrying a `pattern: String` field.

5. **State machine enforcement gap:** The broad sweep says "validated transitions at the type level" but this is slightly misleading. The status field on Finding is `pub` and can be directly mutated by any code with a `&mut Finding`. The `can_transition_to()` method provides validation, but enforcement is at the caller level, not the type level. The compiler does not prevent `finding.status = LifecycleState::Closed`.

6. **FieldType enum:** The broad sweep does not mention this enum from query/fields.rs, which classifies TallyQL field names into 7 semantic types for evaluation dispatch.

7. **STOPWORDS:** The broad sweep mentions stopwords but does not document the deliberate exclusion of negation words ("not", "no", "without") from the stopword list, which is architecturally significant for the token Jaccard matching stage.

## Delta Summary
- New items added: 8 (FieldType, TallyQLError details, RuleStore as stateless service, SyncResult fields, STOPWORDS semantics, SuppressionType::InlineComment data, severity ordinal as free function, FindingIdentityResolver index details)
- Existing items refined: 6 (transition count, Severity Display asymmetry, Finding equality omission, state machine enforcement gap, STOPWORDS negation exclusion, FieldEdit value types)
- Remaining gaps: MCP Input types (not yet cataloged -- they are defined in mcp/server.rs), parser constants (MAX_QUERY_LENGTH, MAX_NESTING_DEPTH), index.json schema, semantic.rs entities

## Novelty Assessment
Novelty: SUBSTANTIVE
The discovery of the state machine enforcement gap (pub field, not type-enforced), the correction of the transition count, the FieldType classification system, and the detailed aggregate boundary mapping all change how one would spec the system.

## Convergence Declaration
Another round needed -- MCP Input types (24 tool input structs in server.rs), parser constants, and the semantic search module remain uncataloged. The state machine enforcement gap merits verification of how callers actually enforce it.

## State Checkpoint
```yaml
pass: 2
round: 1
status: complete
files_scanned: 18
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
next_action: Round 2 -- catalog MCP input types, parser constants, verify state transition enforcement patterns
```
