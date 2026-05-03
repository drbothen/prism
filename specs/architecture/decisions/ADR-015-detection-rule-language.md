---
document_type: adr
adr_id: "ADR-015"
title: "Detection Rule Language"
status: PROPOSED
version: "0.1"
date: 2026-05-02
wave: 4
phase: 4.A
producer: architect
timestamp: 2026-05-02T00:00:00Z
inputs:
  - .factory/cycles/wave-4-operations/cycle-manifest.md
  - .factory/cycles/wave-4-operations/preflight-findings/research-findings.md
  - .factory/STATE.md
  - .factory/stories/S-4.03-detection-rules.md
  - .factory/stories/S-4.04-detection-evaluation.md
  - .factory/stories/S-4.05-alert-generation.md
  - .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
  - .factory/specs/architecture/decisions/ADR-008-dtu-state-segregation.md
  - .factory/specs/architecture/decisions/ADR-013-schedule-execution-semantics.md
anchor_stories: [S-4.03, S-4.04, S-4.05]
aligns_with: [ADR-006, ADR-008, ADR-010, ADR-013]
supersedes: []
superseded_by: null
amendments: []
locked_decisions: [D-208, D-211]
references_decisions: [D-207, D-208, D-209, D-211]
references_research: [R-1, R-7]
verification_properties: [VP-018, VP-027, VP-139, VP-140]
subsystems_affected: [SS-13]
traces_to: specs/architecture/ARCH-INDEX.md
---

# ADR-015: Detection Rule Language

## Status

PROPOSED 2026-05-02, v0.1. Pending review and acceptance prior to story remediation and BC authoring.

---

## 1. Context

### 1.1 The Detection Engine

The `prism-operations` crate (SS-13, per ADR-012 `src/` convention) includes a detection
engine that evaluates analyst-defined rules against incoming query results. Rules are
authored in `.detect` files (TOML-formatted) and managed via MCP CRUD tools. The engine
compiles rule predicates to DataFusion SQL WHERE clauses, evaluates them against ephemeral
in-memory data lakes built from sensor query results, and fires alerts when rules match.

### 1.2 Why This ADR Exists

Three design gaps in the Wave 4 story drafts (as of 2026-04-16) would produce
inconsistent behavior or verifiability failures if left to per-story implementation choice:

- **IOC pattern-matching architecture.** S-4.03 implicitly assumed `regex::RegexSet` could
  handle 100,000+ IOC patterns. R-7 confirms RegexSet's documented design point is
  hundreds-to-thousands of patterns; 100k triggers DFA size limits and a documented
  600+ MB memory regression (regex Issue #1059). A layered architecture is required.
- **DataFusion API surface.** S-4.03 and S-4.04 story drafts referenced APIs that do not
  exist in DataFusion 53.x: `register_batch` (no such method), `Expr::evaluate` for
  RecordBatch (no such method in 53.x), and `create_udf` without guidance on when to
  prefer `ScalarUDFImpl`. R-1 provides the authoritative 53.1.0 API surface.
- **Dedup window resolution mechanics.** D-211 (LOCKED via D-207 authoring plan) requires
  dedup-window values to be resolved at rule-load time (not per-evaluation), with
  invalidation on schedule-change. The notification hook was established in ADR-013 §2.7;
  this ADR owns the resolution semantics and the `detection_state` CF key design.

### 1.3 Scope

This ADR defines:
- `.detect` DSL syntax and `RuleCondition`/`RuleScope` type shapes
- DataFusion 53.x compilation strategy and API pins
- Security UDF registry (three UDFs, volatility, implementation deps)
- IOC pattern matching layered architecture (aho-corasick + RegexSet split)
- Dedup window resolution mechanics (D-211 — owned here per D-207 authoring plan)
- `detection_state` RocksDB CF key encoding

This ADR does NOT define:
- Alert payload format and delivery (ADR-016, S-4.05)
- SIEM output encodings (ADR-019)
- Schedule executor tick mechanics (ADR-013)

---

## 2. Decision

### 2.1 DSL Syntax: `.detect` TOML Format

**Decision:** Detection rules are authored in TOML-formatted files with the extension
`.detect`. The top-level field set is:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Unique rule identifier within scope |
| `description` | String | No | Human-readable rule summary |
| `severity` | String | Yes | One of: `critical`, `high`, `medium`, `low`, `info` |
| `scope` | Table | Yes | Rule scope — see `RuleScope` (§2.2) |
| `enabled` | Boolean | Yes | Whether the rule is active |
| `condition` | Table | Yes | Detection condition — see `RuleCondition` (§2.3) |
| `dedup_window` | String | No | Duration string (e.g., `"1h"`, `"30m"`). Absent = resolved per §2.5 |
| `schedule_id` | String | No | ScheduleId to link dedup-window resolution (see §2.5) |

Rules failing validation at load time are rejected with a structured error (VP-018).
Structural validation (required fields, type constraints) occurs before semantic
validation (DataFusion compilation, UDF references).

### 2.2 RuleScope Enum (D-208 — LOCKED)

```
enum RuleScope {
    Global,
    Client { org_id: OrgId, client_id: ClientId },
    Analyst { org_id: OrgId, analyst_id: AnalystId },
}
```

**Global**: applies to all orgs; requires platform-admin authority to create or modify.
Represented in `.detect` as `scope.kind = "global"`.

**Client(OrgId, ClientId)**: applies to a specific `(OrgId, ClientId)` pair (D-208 dual
hierarchy; DRIFT-403-002 calls out the prior `Client(ClientId)` form as incomplete — this
ADR locks the dual form). Represented as `scope.kind = "client"` + `scope.org_id` +
`scope.client_id`.

**Analyst(OrgId, AnalystId)**: applies to a specific analyst within an org. Represented as
`scope.kind = "analyst"` + `scope.org_id` + `scope.analyst_id`.

**Three-scope merge resolution order** at evaluation time:
1. Global rules evaluated first (apply to all orgs).
2. Client rules narrowed to the `(OrgId, ClientId)` pair in the current evaluation context.
3. Analyst rules narrowed to the `(OrgId, AnalystId)` pair for the current session.

Conflict semantics: explicit ENABLED beats inherited DISABLED. Analyst > Client > Global
precedence on overlap. Explicit-name match beats glob match within the same scope tier.

**Precedence invariant:** An Analyst rule can suppress or extend any Client or Global rule
for that analyst's session only. This override does not affect other sessions.

### 2.3 RuleCondition Enum

```
enum RuleCondition {
    Single { where_clause: String },
    Correlation {
        group_by: Vec<String>,
        min_count: u32,
        time_window: Duration,
        where_clause: String,
    },
    Sequence {
        events: Vec<SequenceStep>,
        time_window: Duration,
    },
}

struct SequenceStep { where_clause: String }
```

**Single**: condition fires when any record in the query result matches the WHERE clause.
TOML: `condition.kind = "single"` + `condition.where = "<DataFusion SQL WHERE clause>"`.

**Correlation**: fires when at least `min_count` records matching `where_clause` are
observed for the same `group_by` key within `time_window`. State persisted in
`detection_state` CF (§2.6). TOML: `condition.kind = "correlation"` + `condition.where`,
`condition.group_by`, `condition.min_count`, `condition.time_window`.

**Sequence**: fires when all ordered `events` steps match in sequence within `time_window`,
with each step's WHERE clause matching a distinct event in arrival order. TOML:
`condition.kind = "sequence"` + `condition.events = [{where = "..."}]` + `condition.time_window`.

### 2.4 DataFusion Compilation Strategy (R-1)

**Version pin:** `datafusion = "53.1"` (caret-compatible, locks major 53 for the Wave 4
cycle). A 54-bump task is pre-budgeted for post-Wave-4. Rationale: DataFusion 53.0.0
shipped 2026-04-13 with breaking changes in SQL parser and optimizer; locking the major
prevents mid-wave surprises.

**UDF registration — `ScalarUDFImpl` trait (NOT `create_udf`):** The three security UDFs
(§2.5) are registered via the `ScalarUDFImpl` trait, which requires:

```rust
fn as_any(&self) -> &dyn Any
fn name(&self) -> &str
fn signature(&self) -> &Signature
fn return_type(&self, arg_types: &[DataType]) -> Result<DataType>
fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue>
```

Optionally override `aliases`, `documentation`, `simplify`. Use `create_udf` only for
trivial one-liner UDFs where no advanced semantics (volatility, coerce_types,
output_ordering) are needed.

**In-memory data lake construction:** Build via `MemTable::try_new(schema, vec![vec![batch]])`
wrapped in `Arc::new`, registered with `ctx.register_table("events", Arc::new(table))`.
There is no `SessionContext::register_batch` shortcut in 53.x (apache/datafusion#3426 has
not landed). Do not reference it in story or BC specs.

**Predicate evaluation reuse:** Plan logical `Expr` to `PhysicalExpr` once via
`create_physical_expr` against a `DFSchema`; reuse `PhysicalExpr::evaluate(&RecordBatch) -> ColumnarValue`
across batches within the same evaluation context. There is no `Expr::evaluate` method for
full RecordBatch evaluation in 53.x — story drafts referencing this pattern must be
remediated.

**SessionContext lifecycle:** Construct `SessionContext` per query evaluation; drop on scope
exit. `SessionContext` is sync-`Drop`; no async cleanup needed. `scopeguard::defer!` may be
used for explicit audit logging at scope exit, subject to the `panic = "unwind"` caveat
from R-6 (defer does not run if profile uses `panic = "abort"`).

### 2.5 Security UDF Registry

Three UDFs are registered at `SessionContext` init. All are pure, deterministic, side-effect
free. Volatility: `Volatility::Immutable` for all three.

| UDF | Signature | Implementation | Pin |
|-----|-----------|---------------|-----|
| `subnet_contains` | `(ip: Utf8, cidr: Utf8) -> Boolean` | `ipnet` crate; checks whether `ip` is within `cidr` range | `ipnet = "2"` |
| `ioc_match` | `(value: Utf8, ioc_set: Utf8) -> Boolean` | Delegates to the `PatternStore` (§2.6); `ioc_set` is a named set identifier | — (internal) |
| `time_window` | `(timestamp: Timestamp, window: Utf8) -> Boolean` | Checks whether `timestamp` falls within a rolling window; `window` accepts duration strings (`"5m"`, `"1h"`, `"24h"`) | — (internal) |

UDFs are registered at `SessionContext` construction, not per-query. The `ioc_match` UDF
reads from the `PatternStore` (held in `Arc<ArcSwap<Arc<PatternStore>>>`); it does NOT
mutate the store. `time_window` resolves against `Utc::now()` at evaluation time; it is
`Immutable` within a single evaluation session but varies across sessions — this is
acceptable and by design.

**`ipnet = "2"` rationale:** R-7 confirms no soundness or CVE issues; resolves to 2.12.0
in Cargo.lock. Used exclusively for subnet CIDR math in the `subnet_contains` UDF — a
tight, well-defined dependency.

### 2.6 IOC Pattern Matching Architecture (R-7)

**Reject** the implicit S-4.03 claim that `regex::RegexSet` scales to 100,000+ patterns.
R-7 confirms: RegexSet's documented design point is "hundreds or thousands of regexes";
100k triggers DFA size limits; the 1.9.x memory regression allocated 600+ MB for moderate
sets (regex Issue #1059). This is an architectural defect, not a configuration issue.

**Decision — layered `PatternStore`:**

**Layer 1 — Literal IOCs** (`aho-corasick = "1.1"`):
- SHA-256, MD5, IPv4, IPv6, exact hostnames, exact URL strings.
- O(n + z) linear search regardless of pattern count. Scales to 100k+ patterns.
- Pin: `aho-corasick = "1.1"` (resolves to 1.1.4 per R-7 citation).

**Layer 2 — Regex IOCs** (`regex = "1.10"` with `RegexSet`):
- Pattern-based IOCs (e.g., suspicious domain templates, variable-suffix URL families).
- Capped at approximately 1,000 patterns. `dfa_size_limit` configured explicitly (10 MB).
- Targets the minority case; capacity cap enforced at `PatternStore::add_regex_pattern`.

**Escalation path (Wave 4+):** For mixed corpus exceeding ~10k regex patterns, escalate to
`regex_automata`'s `MultiPatternDfa`. This is NOT in Wave 4 scope. If story-writer adds a
10k+ regex IOC scenario, a sub-research task must precede implementation.

**Hot-reload mechanics:**
- `PatternStore` is stored in `Arc<ArcSwap<Arc<PatternStore>>>` for lock-free hot-reload.
- File watcher: `notify = "7"` + `notify-debouncer-full` (R-7 recommendation; story-writer
  adds to S-4.03 Library table per drift DRIFT-403-003).
- Per-file caps: 10 MB max file size, 50 IOC files maximum. Silent drop with WARN log if
  exceeded; flagged for human review in the Wave 4 operations gate.

**Overflow behavior:** When the regex-IOC cap is reached, new regex patterns are rejected
with `Err(RegexCapExceeded)` at validation time (load time), not silently dropped.

### 2.7 Dedup Window Resolution (D-211 — OWNED BY THIS ADR)

ADR-013 §2.7 established the schedule-change reload hook (`tokio::sync::watch` channel,
`ScheduleChangeNotification` enum). This ADR owns the dedup-window resolution semantics.

**Resolution at rule load time:**

Priority order for resolving `effective_dedup_window` at rule load:
1. If `dedup_window` field is explicit in the rule file, use it.
2. If `schedule_id` is set and the linked `ScheduleEntry` has an `interval` field: use the
   schedule's interval as the dedup window.
3. If neither: default to `Duration::hours(1)`.

The resolved `effective_dedup_window` is stored in the in-memory `DetectionRuleCache`
keyed by `RuleId`. This eliminates per-evaluation OrgRegistry round-trips (D-211 rationale:
cache + invalidate keeps dedup semantics dynamic without per-eval cost).

**Invalidation triggers:**
- **Schedule change** (via ADR-013 §2.7 `watch` channel): on receipt of
  `ScheduleChangeNotification::Updated(schedule_id)` or `::Deleted(schedule_id)`,
  the detection engine invalidates all `DetectionRuleCache` entries whose `schedule_id`
  matches the changed schedule and reloads them from the persistent rule store.
  `ScheduleChangeNotification::Created(schedule_id)` does not trigger invalidation
  (no rule yet references a brand-new schedule).
- **Rule file change** (via `notify` file watcher, same watcher as IOC hot-reload §2.6):
  invalidates the specific rule's cache entry and reloads that rule.

**NOT runtime resolution.** The dedup-window is never resolved during
per-detection evaluation. Any code path that calls OrgRegistry inside the detection
evaluation hot path is a correctness defect (VP-140 covers this invariant).

### 2.8 `detection_state` Column Family Design

CF name: `detection_state`. Key format follows ADR-008's universal re-keying rule and
D-208's OrgId prefix requirement (DRIFT-404-002). Single-byte type discriminators allow
efficient prefix scans per entry type within an org.

| Entry type | Key format | Value |
|------------|-----------|-------|
| Correlation tracker | `{org_id}:\x00:{rule_id}:{group_key}` | bincode 2.x: count (`u64`) + first_seen (`DateTime<Utc>`) |
| Sequence tracker | `{org_id}:\x01:{rule_id}` | bincode 2.x: sequence state machine |
| Dedup tracker | `{org_id}:\x02:{rule_id}:{dedup_key}` | bincode 2.x: last_fire (`DateTime<Utc>`) |

Per-org cap: 10,000 group keys per rule. At cap, new group-key insertions are silently
dropped with a WARN log; this value is flagged for human review if detection-state overflow
is observed in Wave 4 operations.

**ADR-008 compliance:** The `{org_id}:` prefix satisfies ADR-008's universal re-keying
rule. Per-org `reset_for(org_id)` semantics work correctly: a prefix-scan on `{org_id}:`
deletes all org-A detection state without touching org-B. The discriminator byte follows
the OrgId prefix to allow type-filtered prefix scans within an org.

---

## Rationale

The seven decisions in Section 2 are jointly necessary. Each addresses a distinct failure
mode that would otherwise manifest as a correctness defect, a verifiability gap, or an
operational hazard.

**Layered IOC architecture (§2.6) is required for memory safety.** The Prism process
budget is 512 MB (memory: `project_memory_budget.md`). R-7 documents a 600+ MB memory
regression in `regex::RegexSet` for moderate pattern counts (Issue #1059). A
`RegexSet`-only architecture at 100k+ patterns would exceed the process memory budget
before any query work occurs. The aho-corasick literal layer eliminates this hazard by
design: O(n+z) search with memory proportional to the automaton, not the corpus.

**`ScalarUDFImpl` trait (§2.4) is required for verifiability.** The three security UDFs
(`subnet_contains`, `ioc_match`, `time_window`) must be formally typed so their
signatures appear in DataFusion query plans and can be inspected in VP-018 harnesses.
The `create_udf` convenience API accepts a closure without enforcing a type contract;
the `ScalarUDFImpl` trait requires explicit `Signature`, `return_type`, and `invoke_with_args`
implementations. For security-critical UDFs, the stronger contract is required.

**Dedup-window resolved at load time (§2.5/2.7) is required for detection latency.** The
detection evaluation loop runs on every sensor query result. A per-evaluation OrgRegistry
round-trip would add an external service call to the hot path, introducing non-deterministic
latency under concurrent schedule load. Cache-at-load-time (D-211) gives the hot path a
single hash-map lookup. The `watch`-channel invalidation path (ADR-013 §2.7) ensures the
cached value stays current without any polling.

**`RuleScope::Client { org_id, client_id }` dual form (§2.2 — D-208 LOCKED) is required
for tenant isolation.** A `Client(ClientId)` key without `OrgId` allows cross-tenant
aliasing: two different orgs could define rules referencing the same `ClientId` value,
with no structural guarantee of isolation. The dual form `(OrgId, ClientId)` provides a
unique namespace per org, consistent with ADR-008's universal re-keying rule.

**Single-byte discriminators in `detection_state` CF (§2.8) are required for efficient
per-type scans.** Correlation state, sequence state, and dedup state have different TTL
and compaction strategies. The discriminator byte placed after the `{org_id}:` prefix
allows RocksDB prefix-scans to target one entry type within one org without scanning the
other types. Without the discriminator, a per-type scan would require reading and
filtering all detection-state records for an org.

---

## 3. Consequences

### 3.1 Positive

- **Scalable IOC matching.** The aho-corasick literal layer handles 100k+ patterns with
  O(n+z) complexity. The regex layer remains capped and safe. The hybrid architecture is
  correct at realistic MSSP IOC corpus sizes, where literals (file hashes, IPs, exact
  hostnames) dominate by count.
- **Formally verifiable compilation.** DataFusion 53.x `ScalarUDFImpl` trait requires
  explicit type declarations (`Signature`, `return_type`, volatility). These are amenable
  to property-based testing (VP-018) without runtime surprises from inferred types.
- **No per-eval OrgRegistry round-trips.** Dedup window resolved at load time (D-211) and
  invalidated on change via ADR-013's `watch` channel. Detection evaluation hot path
  touches only the `DetectionRuleCache` — no external service calls.
- **Clean purity boundary.** `RuleCondition` predicate evaluation (DataFusion
  `PhysicalExpr`) is pure: takes `RecordBatch` in, returns `ColumnarValue`. The effectful
  shell (IOC store reload, RocksDB state reads, dedup cache) is separated from the
  pure evaluation core.

### 3.2 Negative

- **Regex IOC cap introduces operational friction.** The ~1k regex cap requires IOC
  managers to classify patterns at ingest. If the regex corpus grows beyond 1k, an
  `ADR-015` amendment (escalation to `regex_automata`) is required before patterns can be
  added. Story-writer must document this cap in the operator guide.
- **`notify = "7"` adds file-system polling complexity.** File watchers are inherently
  platform-specific in behavior (inotify/kqueue/FSEvents). DRIFT-403-003 noted the missing
  dependency — it must be resolved in S-4.03 before implementation begins.
- **DataFusion 53 major-version lock.** The 53.x cycle lock prevents incremental adoption
  of 54.x improvements. The pre-budgeted 54-bump task post-Wave-4 must be tracked as a
  tech debt item.
- **`ArcSwap` adds read-path overhead.** Each IOC-matching call dereferences through
  `ArcSwap` to get the current `PatternStore`. For typical security query rates this is
  negligible, but it is a non-zero cost per evaluation.

---

## 4. Alternatives Considered

### 4.1 `RegexSet`-Only at 100k+ Patterns (Rejected — R-7)

The implicit S-4.03 story assumption was that `regex::RegexSet` could handle the full IOC
corpus including 100k+ patterns. R-7 establishes this is outside the documented design
point: RegexSet recommends "hundreds or thousands" of regexes; the 1.9.x memory regression
(600+ MB for moderate sets, Issue #1059) is unacceptable in a 512 MB process budget
(memory: `project_memory_budget.md`). No further analysis required; R-7 is definitive.

### 4.2 Runtime Dedup Window Resolution (Rejected — D-211)

An alternative to cache-at-load-time would resolve the dedup window on each detection
evaluation by querying OrgRegistry. Rejected by D-211: the per-eval OrgRegistry
round-trip is a latency hazard in the detection hot path, especially under high event
rates. The cache-plus-invalidation pattern trades a small complexity cost (invalidation
logic) for a guaranteed sub-microsecond dedup window lookup on every evaluation.

### 4.3 `create_udf` Convenience API for All Three UDFs (Rejected — R-1)

The `create_udf` convenience function is sufficient for trivial UDFs but lacks the
explicit type contract enforced by `ScalarUDFImpl` (required methods `as_any`,
`signature`, `return_type`). For security-critical UDFs like `subnet_contains` and
`ioc_match`, the trait-based form provides stronger type guarantees and is the DataFusion
53.x recommended pattern per R-1 (`ScalarUDFImpl` is the primary surface for advanced
semantics). Using `create_udf` for the three registered UDFs would produce correct behavior
but weaker static guarantees; the `ScalarUDFImpl` path is required.

### 4.4 Separate Pattern-Store Crate (Deferred)

Extracting the `PatternStore` (aho-corasick + RegexSet) into a standalone
`prism-pattern-store` crate was considered for reuse across future wave stories. Deferred
to Wave 5: the IOC pattern matching surface is currently consumed only by S-4.03 and the
`ioc_match` UDF. Premature extraction adds a dependency edge without clear benefit.
Flag for re-evaluation when a second consumer appears.

---

## Source / Origin

- **Architectural decisions (STATE.md §Wave 4 Decision Log):**
  - D-207: 6-ADR topology; ADR-015 scoped to detection rule language; dedup mechanics
    (D-211) owned here (logged 2026-05-02).
  - D-208 (LOCKED): OrgId/ClientId dual hierarchy; `RuleScope::Client` must carry both
    `OrgId` and `ClientId`; prior `Client(ClientId)` form incomplete per DRIFT-403-002
    (logged 2026-05-02).
  - D-211: Dedup window resolved at scheduling-time; baked into `RuleCondition`;
    invalidated on schedule change via ADR-013 §2.7 watch channel (logged 2026-05-02).
- **Research findings (research-findings.md):**
  - R-1 §DataFusion: `datafusion = "53.1"` current; `ScalarUDFImpl` trait as primary UDF
    surface; `MemTable::try_new` + `register_table` pattern; `create_physical_expr` +
    `PhysicalExpr::evaluate` for batch predicate evaluation; no `register_batch` in 53.x
    (2026-05-02).
  - R-7 §aho-corasick/RegexSet: RegexSet design point is hundreds-to-thousands of
    patterns; 100k triggers DFA limits and 600+ MB memory regression (Issue #1059);
    `aho-corasick = "1.1"` for literal layer; RegexSet capped at ~1k for regex minority
    (2026-05-02).
- **Story drafts:**
  - S-4.03-detection-rules.md v1.5: IOC file loading spec (BC-2.13.014), UDF registration
    (BC-2.13.010), three-scope resolution (BC-2.13.011) — story contains pre-R-7
    `RegexSet`-only IOC assumption and pre-R-1 `create_udf`-only UDF pattern; both
    superseded by this ADR.
  - S-4.04-detection-evaluation.md v1.4: detection state persistence (BC-2.13.012),
    alert deduplication (BC-2.13.013) — story references pre-D-211 per-eval dedup
    resolution; superseded by §2.5/2.7 of this ADR.
- **Prior ADRs:**
  - ADR-006 §2.1: OrgId canonical routing key; `RuleScope` org-scoped boundaries derive
    from this rule.
  - ADR-008: Universal `{org_id}:` CF key prefix rule; `detection_state` CF key encoding
    in §2.8 derives directly from this rule.
  - ADR-010: `PRISM_*` env-var convention applies to any detection-engine tuning
    parameters.
  - ADR-013 §2.7: `tokio::sync::watch` schedule-change reload hook established; ADR-015
    consumes it for dedup-window invalidation.
- **Verification properties:**
  - VP-018 (`vp-018-detection-rule-validation.md`): pre-existing; harness skeleton to be
    added by story-writer per S-4.03 remediation.
  - VP-027 (`vp-027-alert-dedup-key.md`): pre-existing; harness skeleton to be added per
    S-4.04 remediation.
  - VP-139: proposed in this ADR; VP file and VP-INDEX update to be produced before Phase
    4.B BC authoring begins.
  - VP-140: proposed in this ADR; VP file and VP-INDEX update to be produced before Phase
    4.B BC authoring begins.

---

## 5. Verification Plan

### 5.1 VP-018 — Detection Rule Validation: Rejects Invalid Rules (Pre-existing)

**Property:** For any input that is structurally invalid (missing required fields, wrong
types) or semantically invalid (DataFusion compilation error, unknown UDF reference),
`DetectionRule::from_toml` returns `Err(ValidationError)` and never panics.
**Method:** Proptest.
**Harness sketch:**

```rust
proptest! {
    #[test]
    fn rule_validation_never_panics(input in any::<Vec<u8>>()) {
        // Parse raw bytes as attempted TOML; check no panic
        let _ = DetectionRule::from_toml_bytes(&input);
    }

    #[test]
    fn invalid_rules_rejected(rule in invalid_rule_strategy()) {
        let result = DetectionRule::validate(&rule);
        prop_assert!(result.is_err());
    }
}
```

**Status:** draft (VP-018 file exists at `vp-018-detection-rule-validation.md`; harness
skeleton to be added by story-writer per S-4.03 remediation).
**Module:** `prism-operations` | **Priority:** P0 | **Anchor story:** S-4.03

### 5.2 VP-027 — Alert Dedup Key: Correct per Match Mode (Pre-existing)

**Property:** For each `RuleCondition` variant (Single, Correlation, Sequence), the
derived dedup key is stable (same inputs produce same key), and distinct for distinct
`(rule_id, match_context)` pairs.
**Method:** Proptest.
**Harness sketch:**

```rust
proptest! {
    #[test]
    fn dedup_key_deterministic(
        rule_id in any::<RuleId>(),
        ctx in any::<MatchContext>()
    ) {
        let k1 = compute_dedup_key(&rule_id, &ctx);
        let k2 = compute_dedup_key(&rule_id, &ctx);
        prop_assert_eq!(k1, k2);
    }
}
```

**Status:** draft (VP-027 file exists at `vp-027-alert-dedup-key.md`; harness skeleton
to be added per S-4.04 remediation).
**Module:** `prism-operations` | **Priority:** P0 | **Anchor story:** S-4.04

### 5.3 VP-139 — IOC Matching Layered Correctness (PROPOSED — NEW)

**Property:** For any input value and any IOC corpus (up to 100k literals + 1k regex
patterns), the layered `PatternStore` (aho-corasick literal layer + RegexSet regex layer)
produces the same match/no-match result as a correct reference implementation that
evaluates all patterns independently. Specifically: a literal IOC hit must be detected
by the aho-corasick layer; a regex IOC hit must be detected by the RegexSet layer; no
false negatives across the split.

**Method:** Proptest. Generate arbitrary literal sets and regex sets; for each (value,
corpus) pair, verify `PatternStore::matches(value)` equals the oracle (naive linear
scan over all patterns).

```rust
proptest! {
    #[test]
    fn ioc_layered_matches_oracle(
        value in any::<String>(),
        literals in prop::collection::vec(any::<String>(), 0..500),
        regexes in prop::collection::vec(valid_regex_strategy(), 0..50)
    ) {
        let store = PatternStore::new_with(&literals, &regexes);
        let oracle = oracle_match(&value, &literals, &regexes);
        prop_assert_eq!(store.matches(&value), oracle);
    }
}
```

**Status:** proposed; VP-139 assigned in this ADR. VP file and VP-INDEX update to be
produced before Phase 4.B BC authoring begins.
**Module:** `prism-operations` | **Priority:** P1 | **Anchor story:** S-4.03

### 5.4 VP-140 — Dedup Window Scheduling-Time Resolution + Invalidation Correctness (PROPOSED — NEW)

**Property:** (a) `effective_dedup_window` for any rule is never resolved during
detection-evaluation (i.e., no OrgRegistry call occurs in the detection hot path).
(b) After a `ScheduleChangeNotification::Updated(sid)` is delivered via the ADR-013 §2.7
`watch` channel, all `DetectionRuleCache` entries whose `schedule_id == sid` have their
`effective_dedup_window` recomputed from the updated `ScheduleEntry` before the next
evaluation cycle.
(c) After a rule file change, that rule's cache entry is invalidated and reloaded before
the next evaluation cycle.

**Method:** Proptest (structural) + integration test.

Structural test (proptest): verify that the `DetectionEvaluator::evaluate` function
does not hold a reference to `OrgRegistry` at any call site (module-boundary check);
build a mock `DetectionRuleCache` and verify `evaluate` takes no `OrgRegistry` parameter.

Integration test: construct a `DetectionRuleCache` with a rule linked to `schedule_id = S`.
Deliver `ScheduleChangeNotification::Updated(S)` via the watch channel. Assert the
`effective_dedup_window` in the cache changes to match the new `ScheduleEntry.interval`
within one watch-receive cycle.

**Status:** proposed; VP-140 assigned in this ADR. VP file and VP-INDEX update to be
produced before Phase 4.B BC authoring begins.
**Module:** `prism-operations` | **Priority:** P1 | **Anchor story:** S-4.03, S-4.04

---

## 6. References

### Research Findings

- **R-1** (`research-findings.md §R-1`): DataFusion 53.1.0 current; `ScalarUDFImpl` trait
  as primary UDF surface; `MemTable::try_new` + `register_table` for in-memory data lake;
  `create_physical_expr` + `PhysicalExpr::evaluate` for batch predicate evaluation; no
  `register_batch` shortcut in 53.x.
- **R-7** (`research-findings.md §R-7`): `regex::RegexSet` design point is
  hundreds-to-thousands of patterns; 100k triggers DFA size limits and memory regression
  (Issue #1059); `aho-corasick = "1.1"` for literal-IOC matching at scale; `RegexSet`
  capped at ~1k for regex-IOC minority.

### Architectural Decisions

- **D-207** (STATE.md §Wave 4 Decision Log): 6-ADR topology; ADR-015 scoped to detection
  rule language; D-211 dedup mechanics owned here.
- **D-208** (STATE.md §Wave 4 Decision Log — LOCKED): OrgId/ClientId dual hierarchy;
  `RuleScope::Client` must include both `OrgId` and `ClientId`; prior `Client(ClientId)`
  form is incomplete.
- **D-211** (STATE.md §Wave 4 Decision Log): Dedup window resolved at scheduling-time;
  baked into `RuleCondition`; invalidated on schedule change; ADR-015 documents resolution
  semantics.

### Prior ADRs

- **ADR-006 §2.1**: OrgId is canonical routing key; `RuleScope::Client` and `::Analyst`
  map to org-scoped boundaries.
- **ADR-008**: Universal `{org_id}:` CF key prefix rule; `detection_state` CF key format
  derived from this rule.
- **ADR-010**: `PRISM_*` env-var convention; any scheduler tuning parameters follow this
  pattern.
- **ADR-013 §2.7**: `tokio::sync::watch` schedule-change reload hook. ADR-015 is a
  consumer of this hook for dedup-window invalidation; the hook itself is not
  re-specified here.

### Drift Audit Items Addressed

- **DRIFT-403-002**: `RuleScope::Client(ClientId)` incomplete form — resolved by §2.2
  locking `Client { org_id: OrgId, client_id: ClientId }`.
- **DRIFT-403-003**: Missing `notify = "7"` + `notify-debouncer-full` in S-4.03 Library
  table — resolved by §2.6 specifying the file-watcher dependency.
- **DRIFT-404-002**: OrgId prefix required on `detection_state` CF keys — resolved by §2.8
  encoding table.
