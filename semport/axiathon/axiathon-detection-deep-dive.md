# Axiathon Detection Engine Deep Dive

**Date:** 2026-04-13
**Scope:** Detection rules (.axd DSL), alert generation, correlation engine, case management, field promotion, detection performance, tenant-scoped detection
**Source:** `spike/crates/axiathon-detection/` (1500+ LOC), `spike/rules/` (6 .axd files), `spike/crates/axiathon-api/src/pipeline.rs`, `spike/crates/axiathon-api/src/state.rs`, `spike/crates/axiathon-storage/` (field promotion)
**Context:** Complements axiathon-pass-8-deep-synthesis.md; focuses exclusively on detection/alerting/case management layer

---

## 1. Detection Rules (.axd Format)

### 1.1 Rule Structure

The `.axd` format is a custom DSL parsed by a Pest PEG grammar (`detection.pest`, 157 lines). Every rule has exactly four mandatory blocks:

```
rule <identifier> {
  meta { ... }
  match <match_clause>
  alert { ... }
}
```

**Grammar file:** `spike/crates/axiathon-detection/src/detection.pest`
**Parser:** `spike/crates/axiathon-detection/src/parser.rs` (477 LOC)
**AST types:** `spike/crates/axiathon-detection/src/ast.rs` (198 LOC)

### 1.2 Meta Block

Required fields: `name` (string), `severity` (enum: info/low/medium/high/critical)
Optional fields: `mitre` (string, ATT&CK technique ID), `description` (string), `enabled` (bool, default true)

```
meta {
  name        "Brute Force Detected"
  severity    high
  mitre       "T1110"
  description "Detects repeated failed login attempts"
  enabled     true
}
```

The parser rejects unknown meta keys (hard error, not warning).

### 1.3 Three Match Modes

**Mode 1: Single-Event** -- immediate, stateless evaluation per event.

```
match event where
  user.name == "root"
  and status_id == 1
  and activity_id == 1
```

**Mode 2: Correlation** -- stateful sliding-window count with group-by.

```
match count(event where status_id == 2 and activity_id == 1) >= 5
  group_by src_endpoint.ip
  within 5m
```

**Mode 3: Sequence** -- ordered multi-step temporal matching, keyed by a shared field.

```
match sequence by src_endpoint.ip within 10m {
  step failures: count(event where status_id == 2) >= 3
  step success:  event where status_id == 1 and activity_id == 1
}
```

### 1.4 Condition Language

Conditions form a boolean expression tree supporting:

- **Boolean combinators:** `and`, `or`, `not`, parenthesized grouping
- **Precedence:** `not` > `and` > `or` (standard boolean precedence via PEG rule nesting)
- **Field predicates:** `<field_path> <operator> <value>`

**Operators (PredicateOp enum, 10 variants):**

| Operator | Syntax | Description |
|----------|--------|-------------|
| Eq | `==` | Exact equality (string, int, float, bool-as-string) |
| NotEq | `!=` | Inequality |
| Gt/Gte/Lt/Lte | `>` `>=` `<` `<=` | Numeric comparison (auto-coerces int/float) |
| Contains | `contains` | Substring match (string only) |
| Matches | `matches` | Regex match (string only, compiled + cached) |
| Cidr | `cidr` | IP-in-CIDR-range (uses `ipnet` crate) |
| In | `in` | Set membership: `field in ("val1", "val2")` |

**Value types:** String (double-quoted), Integer (i64), Float (f64), List (for `in` operator only).

**Duration syntax:** `30s`, `5m`, `1h`, `24h`, `7d` (single-letter unit suffix).

**Field paths:** Dotted notation matching OCSF proto structure: `src_endpoint.ip`, `user.name`, `claroty.alert_type`. The four-tier field resolution chain (`get_field()`) handles both proto fields and vendor extensions in unmapped JSON.

### 1.5 Rule Loading and Validation

**Loading:** `load_rules_from_dir(dir)` scans a directory for `*.axd` files, reads each, and parses via `parse_rules()`. All 6 built-in rules are also hardcoded as string constants in `state.rs` (`BUILTIN_RULE_SOURCES`).

**Validation:** The parser performs structural validation at parse time:
- Meta block must contain `name` (error if missing)
- Alert block must contain both `title` and `description`
- Sequence must have at least one step
- List literals are only valid with the `in` operator
- Unknown meta keys cause hard errors

**What is NOT validated at parse time:**
- Field paths are not validated against the OCSF schema (any dotted identifier is accepted)
- Regex patterns are not validated at parse time (deferred to engine construction)
- CIDR strings are not validated at parse time (validated at evaluation time)
- No security limits on rule size, nesting depth, or regex complexity (unlike the production AxiQL parser)

**API integration:** Rules can also be created/updated via REST API (`POST /api/v1/rules`), which parses the source, stores it per-tenant, and rebuilds all three detection engines (single-event, correlation, sequence) via `rebuild_tenant_engines()`.

### 1.6 The 6 Built-in Rules

| File | Rule ID | Type | Severity | MITRE | Purpose |
|------|---------|------|----------|-------|---------|
| root-login.axd | root_login | Single | High | T1078.003 | Root user authentication |
| suspicious-ip.axd | suspicious_source_ip | Single | High | T1133 | Login from known-bad CIDR |
| privilege-escalation.axd | privilege_escalation | Single | High | T1548.003 | sudo/su regex match |
| brute-force.axd | brute_force | Correlation | High | T1110 | 5+ failures per IP in 5m |
| brute-then-success.axd | brute_then_success | Sequence | Critical | T1110 | 3+ failures then success from same IP in 10m |
| ot-unauthorized-plc-access.axd | ot_unauthorized_plc_access | Single | Critical | T0821 | Claroty vendor field detection (unmapped JSON) |

### 1.7 Prism Mapping

**P0 -- The .axd DSL itself is well-designed and worth adopting.** The three-tier match system (single/correlation/sequence) covers the core SIEM detection use cases. The Pest grammar is clean and extensible.

**What Prism needs differently:**

1. **Scheduled detection vs. streaming.** Axiathon evaluates rules per-event as they arrive (streaming model). Prism's MSSP model runs detection on a schedule across stored data for multiple clients. Prism needs a "scheduled rule" concept: `schedule "0 */5 * * *"` that runs a query over the last N minutes of stored data and generates alerts from matching rows. The .axd grammar should be extended with a `schedule` block or Prism can define detection in a different format (e.g., YAML with embedded AxiQL) and compile to execution plans.

2. **Security hardening.** The detection DSL has zero security limits (unlike the production AxiQL parser with CWE-cited limits). Prism must add: max rule size, max nesting depth, regex size limit via `RegexBuilder::size_limit()`, and regex pattern validation at parse time.

3. **Field path validation.** Rules should optionally validate field paths against the OCSF schema at load time, warning on unknown fields rather than silently failing at evaluation.

**Priority: P1** -- The DSL is important but Prism's detection model (scheduled) differs fundamentally from axiathon's (streaming). The grammar is a useful starting point but needs significant adaptation.

---

## 2. Alert Generation

### 2.1 Alert Structure

**Source file:** `spike/crates/axiathon-detection/src/alert.rs` (587 LOC incl. tests)

The `Alert` struct has these fields:

| Field | Type | Description |
|-------|------|-------------|
| id | String | UUID v7 (time-sortable) |
| rule_id | String | ID of the triggering rule |
| rule_name | String | Human-readable rule name |
| severity | Severity | Info/Low/Medium/High/Critical |
| title | String | Interpolated alert title |
| description | String | Interpolated alert description |
| tenant_id | TenantId | Owning tenant |
| created_at | DateTime<Utc> | Alert creation timestamp |
| rule_type | RuleType | Single/Correlation/Sequence |
| trigger_event_uids | Vec<String> | Event UIDs that caused the alert |

### 2.2 Alert Generation by Rule Type

Three factory functions create alerts from match results:

- `alert_from_single_event(RuleMatch)` -- 1 trigger event UID
- `alert_from_correlation(CorrelationMatch)` -- all event UIDs in the window that crossed the threshold
- `alert_from_sequence(SequenceMatch)` -- event UIDs from each completed step

### 2.3 Template Interpolation System

Alert titles and descriptions support `{variable}` placeholders. The `interpolate_template()` function resolves variables through a four-level resolution chain:

1. **Extra variables** (correlation-specific): `{count}` (number of matching events), `{window}` (duration string like "5m")
2. **Step-scoped variables** (sequence-specific): `{step_name.field}` resolves a field from the event matched by a specific step; `{step_name.count}` resolves the count for a count-type step
3. **Event field variables:** `{src_endpoint.ip}`, `{user.name}`, `{claroty.risk_score}` -- resolved via the same four-tier `get_field()` chain used by the detection engine
4. **Fallback:** Unresolved variables render as `{variable_name}` (literal, no error)

**Template examples from built-in rules:**

```
title: "Login success after {failures.count} failures from {src_endpoint.ip}"
description: "User {success.user.name} authenticated after {failures.count} failed attempts"
```

This resolves `failures.count` from the sequence's "failures" step count, `src_endpoint.ip` from the event's proto field, and `success.user.name` from the event matched by the "success" step.

### 2.4 AlertStore (In-Memory)

`AlertStore` is an in-memory `Vec<Alert>` behind an `RwLock` with a `broadcast::channel(1024)` for real-time notification.

- `add(alert)` -- stores and broadcasts (drops broadcast if no subscribers)
- `query(tenant_id, limit, offset)` -- tenant-scoped, paginated, most-recent-first
- `subscribe()` -- returns `broadcast::Receiver<Alert>` for SSE streaming
- No persistence -- alerts lost on restart

### 2.5 SSE Alert Streaming

The API exposes `GET /api/v1/alerts/stream` as an SSE endpoint. It subscribes to the `alert_tx` broadcast channel and filters by tenant_id, only forwarding alerts belonging to the requesting tenant. Keepalive every 15 seconds.

### 2.6 Prism Mapping

**P1 -- Alert structure and template system are well-designed.**

What Prism needs:

1. **Persistent alert storage.** Alerts must survive restarts. Use Iceberg/Parquet (same as events) or a separate relational store (SQLite/Postgres). The in-memory model is spike-only.

2. **Alert deduplication.** Axiathon has no dedup -- the same rule can fire repeatedly for similar events. Prism needs a dedup key (rule_id + group_by value hash + time window) to avoid alert flooding.

3. **Alert-to-case auto-linking.** Axiathon creates cases manually via API. Prism should support auto-case-creation rules: "if this rule fires 3 times in 1 hour, auto-create a case."

4. **Enrichment at alert time.** The template system interpolates raw field values. Prism should support enrichment (GeoIP on IP addresses, asset lookup on hostnames) at alert generation time.

**Priority: P1** (alert storage and streaming needed from early releases)

---

## 3. Correlation Engine

### 3.1 Architecture

**Source file:** `spike/crates/axiathon-detection/src/correlation.rs` (316 LOC incl. tests)

The correlation engine maintains per-(rule_id, group_by_value) sliding windows using `DashMap<CorrelationKey, SlidingWindow>`. DashMap is a lock-free concurrent HashMap.

**Key types:**
- `CorrelationKey { rule_id: String, group_value: String }` -- composite key
- `SlidingWindow { entries: VecDeque<(Instant, String)>, window_duration: Duration }` -- timestamp + event UID deque

### 3.2 Evaluation Flow

For each incoming event, for each correlation rule:

1. Evaluate the event against the rule's condition using `RuleEngine::evaluate_condition()`
2. If condition matches, build the group key by concatenating `group_by` field values with `|` separator (e.g., for `group_by src_endpoint.ip, user.name` on an event with IP=10.0.0.1, user=root: `"10.0.0.1|root"`)
3. Get or create a `SlidingWindow` for (rule_id, group_key)
4. Add the event timestamp and UID to the window, evicting expired entries
5. Check the threshold comparison (>=, >, ==, etc.)
6. If threshold met: fire alert and **clear the window** (reset-after-fire)

### 3.3 The "Reset-After-Fire" Pattern

When a correlation rule fires, `entry.entries.clear()` empties the sliding window for that group key. This prevents duplicate alerts from the same accumulation. A new accumulation must start from zero.

**Behavioral contract (tested):** After 5 failures fire an alert, 4 more failures do NOT fire. The 5th new failure DOES fire again.

This is the correct SIEM behavior -- without reset, a brute-force rule with threshold=5 would fire on events 5, 6, 7, 8... (every subsequent event).

### 3.4 Cross-Sensor Correlation

The question "How would CrowdStrike alert + Claroty event within 15 minutes correlate?" reveals a limitation: **axiathon's correlation engine operates on conditions within a single event, not across events from different sources.**

The `group_by` mechanism groups events by shared field values (e.g., same IP), and the condition matches events of the same type. There is no built-in mechanism to say "event A from source X and event B from source Y occur within N minutes."

To correlate CrowdStrike + Claroty, you would need:
- A sequence rule: `match sequence by dst_endpoint.ip within 15m { step cs: event where class_uid == 2001 and metadata.product.name == "CrowdStrike" ... step claroty: event where class_uid == 2001 and claroty.device_type == "PLC" ... }`
- The sequence engine handles ordered multi-step matching by shared key field
- But both events must flow through the same pipeline and share a common key field value

**Prism implication:** For scheduled detection across stored data, cross-sensor correlation is easier -- it becomes a SQL JOIN across event types within a time window. Prism's scheduled model is actually better suited for cross-sensor correlation than axiathon's streaming model.

### 3.5 Window Management

- `cleanup()` iterates all windows, evicts expired entries, removes empty windows via `DashMap::retain()`
- `active_windows()` returns the count of live windows (diagnostics)
- Window expiry uses `Instant` (monotonic clock), not wall-clock time
- No periodic cleanup task in the correlation engine itself -- the caller must invoke `cleanup()` (the pipeline does not currently do this, which is a memory leak for long-running processes)

### 3.6 Prism Mapping

**P1 for the pattern; P2 for the specific implementation.**

For Prism's scheduled detection model, correlation becomes a SQL query pattern:

```sql
SELECT src_endpoint_ip, COUNT(*) as cnt
FROM events
WHERE tenant_id = ? AND status_id = 2 AND activity_id = 1
  AND event_time > NOW() - INTERVAL '5 minutes'
GROUP BY src_endpoint_ip
HAVING COUNT(*) >= 5
```

The DashMap sliding-window implementation is optimized for streaming evaluation. Prism's scheduled model evaluates over stored Parquet data via DataFusion, making the SQL approach more natural and performant. However, if Prism adds a real-time detection tier (streaming alerts for critical rules), the DashMap pattern becomes relevant.

**Priority:** P2 for streaming correlation (future); the scheduled equivalent is just SQL aggregation.

---

## 4. Sequence Detection Engine

### 4.1 Architecture

**Source file:** `spike/crates/axiathon-detection/src/sequence.rs` (361 LOC incl. tests)

Uses `DashMap<SequenceKey, SequenceTracker>` where:
- `SequenceKey { rule_id: String, key_value: String }` -- unique tracker per (rule, key_field_value)
- `SequenceTracker` tracks: current step index, per-step counts, per-step matched events, start time, window duration

### 4.2 Sequence Evaluation Flow

1. Extract key field value from event (e.g., `src_endpoint.ip` = "10.0.0.1")
2. Get or create tracker for (rule_id, key_value)
3. If tracker is expired (beyond window duration): reset to step 0
4. Try to advance current step:
   - For `StepType::Event(condition)`: if condition matches, record event and advance to next step
   - For `StepType::Count { condition, op, threshold }`: if condition matches, increment count. If threshold met, advance to next step
5. If all steps complete: fire alert, collect step events and counts, **reset tracker**
6. Steps are strictly ordered -- step 2 cannot advance before step 1 completes

### 4.3 Temporal Behavior

- Sequence window starts when the first matching event creates the tracker (`started_at: Instant::now()`)
- If the window expires before all steps complete, the tracker is reset on the next event for that key
- Excess events beyond threshold (e.g., 5 failures when threshold is 3) are ignored after the step advances
- After a sequence fires, the tracker is reset, requiring a completely new sequence to fire again

### 4.4 Prism Mapping

Sequence detection in a scheduled model is harder than correlation. In SQL:

```sql
WITH failures AS (
  SELECT src_endpoint_ip, COUNT(*) as cnt, MAX(event_time) as last_fail
  FROM events WHERE status_id = 2 AND event_time > NOW() - INTERVAL '10 minutes'
  GROUP BY src_endpoint_ip HAVING COUNT(*) >= 3
),
successes AS (
  SELECT src_endpoint_ip, MIN(event_time) as first_success
  FROM events WHERE status_id = 1 AND event_time > NOW() - INTERVAL '10 minutes'
  GROUP BY src_endpoint_ip
)
SELECT f.src_endpoint_ip FROM failures f
JOIN successes s ON f.src_endpoint_ip = s.src_endpoint_ip
WHERE s.first_success > f.last_fail
```

This is more complex and less precise than the streaming tracker (the SQL version doesn't guarantee strict ordering within overlapping time ranges). Prism may need a hybrid approach: scheduled queries for correlation, streaming trackers for sequence detection on critical rules.

**Priority: P2** -- sequence detection is valuable but complex. Start with correlation (SQL) and add sequence detection later.

---

## 5. Case Management / CaseStatus State Machine

### 5.1 Architecture

**Source file:** `spike/crates/axiathon-detection/src/case.rs` (858 LOC incl. tests)

The case management system provides investigation tracking with a lifecycle state machine, annotations, timeline, and disposition.

### 5.2 CaseStatus State Machine

**5 states:** New, Acknowledged, Investigating, Resolved, Closed

**Valid transitions (12 total):**

```
Forward (linear):
  New -> Acknowledged -> Investigating -> Resolved -> Closed

Skip-ahead (any state can jump forward):
  New -> Investigating / Resolved / Closed
  Acknowledged -> Resolved / Closed
  Investigating -> Closed

Reopen (back to investigating from terminal states):
  Resolved -> Investigating
  Closed -> Investigating
```

**Invalid transitions (tested and rejected):**
- Self-transitions (New -> New)
- Backward to New (Closed -> New, Investigating -> New)
- Backward to Acknowledged (Closed -> Acknowledged, Resolved -> Acknowledged)

**Implementation:** `CaseStatus::can_transition_to(target)` returns bool via exhaustive `matches!()` macro.

When transitioning to Resolved or Closed, `closed_at` is set. When reopening (Resolved/Closed -> Investigating), `closed_at` is cleared.

### 5.3 Case Data Model

| Field | Type | Description |
|-------|------|-------------|
| id | String | UUID v7 |
| tenant_id | TenantId | Owning tenant |
| title | String | Case title |
| description | String | Case description |
| status | CaseStatus | Current lifecycle state |
| priority | Priority | Low/Medium/High/Critical |
| assignee | Option<String> | Assigned analyst |
| source_alert_ids | Vec<String> | Linked alert IDs (deduplicated) |
| annotations | Vec<Annotation> | Investigation notes |
| timeline | Vec<TimelineEntry> | Auto-generated audit trail |
| created_at | DateTime<Utc> | Case creation time |
| updated_at | DateTime<Utc> | Last modification time |
| closed_at | Option<DateTime<Utc>> | When resolved/closed (cleared on reopen) |
| disposition | Option<Disposition> | Final classification |

### 5.4 Disposition System

A tagged enum with per-variant metadata:

```rust
enum Disposition {
    TruePositive { impact_level: String },
    FalsePositive { reason: String },
    Benign { explanation: String },
    Inconclusive,
}
```

Disposition can be set at any time via `set_disposition()`, independent of status transitions.

### 5.5 Annotation System

Annotations have 5 types: Note, Finding, Decision, Question, OtImpact. Each annotation has content, author, and timestamp.

### 5.6 Timeline System

Every mutation auto-generates a `TimelineEntry` with event type, description, actor, and timestamp. Event types: Created, StatusChanged, AlertLinked, AnnotationAdded, PriorityChanged, DispositionSet. This provides a complete audit trail.

### 5.7 Metrics

`CaseStore::metrics(tenant_id)` computes:
- Total/open/closed counts
- Average MTTR (Mean Time to Resolve) for closed cases
- Counts by status and priority
- MTTD (Mean Time to Detect) computed on the Case struct: `case.created_at - earliest_alert_created_at`

### 5.8 Tenant Isolation

All CaseStore operations take `tenant_id` as a required parameter. Get, list, update, and link operations filter by tenant_id. The test suite includes an explicit tenant isolation test.

### 5.9 Broadcast

CaseStore has its own `broadcast::channel(1024)` that emits `CaseEvent { case_id, tenant_id, event_type }` on every mutation.

### 5.10 Prism Mapping

**P1 -- The case management model is solid and directly applicable.**

What Prism needs:

1. **Persistent storage.** Cases must survive restarts. SQLite or Postgres, not in-memory Vec.
2. **RBAC on case operations.** Different analysts can view vs. modify cases. The current system has no access control.
3. **Auto-case creation.** When certain high-severity rules fire, auto-create a case and link the alert. Axiathon requires manual case creation via API.
4. **SLA tracking.** MTTD and MTTR are good. Add SLA thresholds per severity: "Critical cases must be acknowledged within 15 minutes."
5. **Multi-tenant dashboard aggregation.** Prism's MSSP model needs cross-tenant case metrics for the provider-level dashboard.
6. **The OtImpact annotation type** is a nice touch for OT/ICS environments -- keep it for Prism's industrial clients.

**Priority: P1** (case management is core MSSP functionality)

---

## 6. Field Promotion (8-Phase Lifecycle)

### 6.1 What is Field Promotion?

Field promotion is the process of elevating a vendor-specific field from the `unmapped` JSON blob (tier 2) to a first-class Parquet column (tier 1). This improves query performance from "parse JSON string" to "columnar predicate pushdown."

### 6.2 The 8-Phase Lifecycle

The field promotion integration test (`spike/crates/axiathon-storage/tests/field_promotion.rs`, 547 LOC) validates all 8 phases:

**Phase 1: Pre-Promotion Write.** Events arrive with vendor fields in `unmapped` JSON. Example: `syslog.hostname` stored as `{"syslog.hostname": "server0", "syslog.facility": "auth"}` in the unmapped string column.

**Phase 2: Schema Evolution.** `promote_fields(catalog, table_ident, promotions)` adds a new typed column to the Iceberg schema via `Transaction::update_schema().add_column()`. The operation is idempotent -- promoting an already-present column is a no-op.

**Phase 3: Dual-Write.** `StorageWriter::with_promotions(vec![("syslog_hostname", "syslog.hostname")])` configures the writer to extract the field from unmapped JSON and write it to both the new typed column AND the unmapped JSON (for backward compatibility). New events get the column populated; old events have NULL.

**Phase 4: UDF Access.** `json_extract_string(unmapped, 'syslog.hostname')` UDF parses the unmapped JSON at query time to access old (pre-promotion) data that only exists in JSON.

**Phase 5: COALESCE Query.** The query engine wraps promoted field references in `COALESCE(typed_column, json_extract_string(unmapped, 'json_key'))`, transparently querying both old (JSON-only) and new (typed column) data.

**Phase 6: Compaction with Backfill.** The compaction task, when configured with promotions, reads old Parquet files, extracts the promoted field from unmapped JSON, and writes it to the typed column in the compacted output. After compaction, all data has the typed column populated.

**Phase 7: Post-Compaction Query.** After backfill compaction, queries can use the typed column directly without COALESCE/UDF fallback (though COALESCE is still safe).

**Phase 8: Tenant Isolation.** Tenant isolation is preserved across all promotion phases -- promoted columns for one tenant's data don't leak into another tenant's queries.

### 6.3 Key Types

```rust
struct FieldPromotion {
    column_name: String,    // "syslog_hostname" (Iceberg column name)
    json_key: String,       // "syslog.hostname" (key in unmapped JSON)
    iceberg_type: Type,     // Type::Primitive(PrimitiveType::String)
}
```

### 6.4 Prism Mapping

**P0 -- Field promotion is critical for Prism's MSSP model.**

Different clients have different vendor stacks. Client A uses CrowdStrike, Client B uses Claroty, Client C uses Palo Alto. Their vendor-specific fields start in unmapped JSON. As certain fields become commonly queried (e.g., `crowdstrike.detection_name` across many clients), promoting them to typed columns dramatically improves query performance.

Prism should:
1. Adopt the exact 8-phase lifecycle
2. Add a promotion recommendation engine: "field X is accessed in N queries/day across M tenants, recommend promotion"
3. Make promotion a self-service admin operation (API + UI)
4. Track promotion state per table (which fields are promoted, when, by whom)

**Priority: P0** (directly impacts query performance for the multi-tenant model)

---

## 7. Detection Performance

### 7.1 Evaluation Model

Detection is evaluated **per-event, synchronously in the ingestion pipeline** (`pipeline.rs` lines 63-109). For each parsed event:

1. Look up tenant's RuleEngine from `HashMap<TenantId, RuleEngine>`
2. Evaluate all single-event rules (serial iteration)
3. Look up tenant's CorrelationState from `HashMap<TenantId, CorrelationState>`
4. Evaluate all correlation rules (serial iteration, DashMap lookup per rule)
5. Look up tenant's SequenceState from `HashMap<TenantId, SequenceState>`
6. Evaluate all sequence rules (serial iteration, DashMap lookup per rule)

No parallelism within a single event's rule evaluation. Events are processed sequentially from the mpsc channel.

### 7.2 Optimizations Present

- **Regex cache:** `RuleEngine` pre-compiles all regex patterns in `matches` predicates at construction time, stored in `HashMap<String, Regex>`. Avoids recompilation per evaluation.
- **DashMap for stateful engines:** Lock-free concurrent HashMap for correlation windows and sequence trackers. Enables concurrent access from multiple pipeline tasks (though currently single-pipeline).
- **Rule filtering at construction:** `RuleEngine::new()` filters to only enabled single-event rules. Correlation and sequence engines similarly filter to their respective rule types.
- **Short-circuit evaluation:** `And` conditions use `.all()` (early exit on first false), `Or` uses `.any()` (early exit on first true).

### 7.3 Performance Characteristics (from benchmarks)

The benchmark suite (`benches/detection_stateless.rs`, `benches/detection_stateful.rs`) measures:

- Single-event evaluation: individual rule match/no-match/CIDR/regex scenarios
- Batch evaluation: 1000 mixed events (10% root login, 10% suspicious IP, 10% failed, 10% OT finding, 60% no-match)
- Correlation: single source, 100 source IPs, 1000 mixed events, cleanup of 500 windows

No benchmark results are stored in the repo, but the benchmark infrastructure is production-quality (Criterion).

### 7.4 Scalability Gaps

1. **Sequential event processing.** One pipeline task, one mpsc receiver. No worker pool. At high event rates, detection becomes the bottleneck.
2. **All rules evaluated per event.** No rule indexing by event class, field, or predicate. A single-event rule checking `class_uid == 3002` still runs its condition against class_uid 2001 events (the condition fails at the first predicate, but the overhead of iteration and field lookup is still paid).
3. **Clone-heavy.** `Rule` and `AxiathonEvent` are cloned into match results and alerts. For high-throughput, Arc-based sharing would reduce allocation pressure.
4. **No correlation window cleanup.** The pipeline never calls `correlation_state.cleanup()` or `sequence_state.cleanup()`, so expired windows accumulate until the process restarts.

### 7.5 Prism Mapping

**P1 -- Performance optimization needed for Prism's scale.**

For scheduled detection over stored data, the bottleneck shifts from per-event evaluation to SQL query efficiency:

1. **Rule-to-query compilation.** Single-event rules become `SELECT ... WHERE <conditions>`. Correlation rules become `GROUP BY ... HAVING`. Sequence rules become windowed JOINs or correlated subqueries.
2. **DataFusion as the execution engine.** Rule conditions compiled to DataFusion predicates get automatic predicate pushdown into Parquet column readers.
3. **Partition pruning.** `WHERE event_time > NOW() - INTERVAL '5m'` prunes Parquet files by hour partition, dramatically reducing I/O.
4. **Pre-filtered rule dispatch.** Index rules by `class_uid` to only evaluate relevant rules per event class. For 83 OCSF classes x N rules, this eliminates O(N) wasted evaluations.

For a future real-time tier, the per-event model needs:
- Worker pool (multiple pipeline consumers)
- Rule indexing by class_uid
- Arc-based event sharing (no cloning)
- Periodic cleanup of correlation/sequence state

**Priority: P1** (Prism's scheduled model avoids most streaming bottlenecks, but pre-filtered rule dispatch and query compilation are essential)

---

## 8. Tenant-Scoped Detection

### 8.1 Current Implementation

Detection is tenant-scoped through per-tenant engine maps stored in `AppState`:

```rust
pub rule_engines: Arc<RwLock<HashMap<TenantId, RuleEngine>>>,
pub correlation_engines: Arc<RwLock<HashMap<TenantId, CorrelationState>>>,
pub sequence_engines: Arc<RwLock<HashMap<TenantId, SequenceState>>>,
pub rules: Arc<RwLock<HashMap<TenantId, Vec<DetectionRule>>>>,
pub rule_sources: Arc<RwLock<HashMap<TenantId, Vec<String>>>>,
```

Each tenant gets its own independent copy of all three engine types. In the pipeline, the event's `tenant_id` is used to look up the correct engine.

### 8.2 Rule Scope

Currently, **all tenants get the same 6 built-in rules** (hardcoded in `AppState::new()`). Each tenant CAN have additional rules added via the REST API (`POST /api/v1/rules`), and those are per-tenant.

There is no concept of "global rules" vs. "tenant-specific rules" at the DSL level. The scoping happens at the application layer (state.rs). When a rule is created via API, it is parsed, stored per-tenant, and the tenant's engines are rebuilt.

### 8.3 Engine Rebuild

`rebuild_tenant_engines()` re-parses all rule sources for a tenant and constructs fresh RuleEngine, CorrelationState, and SequenceState instances. This is called on every rule create/update operation. **Stateful engines (correlation windows, sequence trackers) are lost on rebuild** -- an active brute-force accumulation would reset to zero.

### 8.4 Prism Mapping

**P0 -- Per-tenant rule scoping is essential for MSSP.**

What Prism needs:

1. **Three rule scopes:**
   - **Global rules** (managed by MSSP provider): apply to all tenants. Example: "Brute force detection" -- universal security baseline.
   - **Tenant-specific rules** (managed by MSSP per client): apply to one tenant. Example: "Claroty PLC access for Client X's OT network."
   - **Client-managed rules** (self-service): if clients have portal access, they can create/edit their own rules.

2. **Rule versioning.** When a global rule is updated, all tenants should pick up the change. Currently, rules are duplicated per tenant (no sharing).

3. **Stateful engine preservation on rule update.** Rebuilding correlation/sequence engines should carry over in-progress windows/trackers (or at minimum, log a warning that active accumulations are reset).

4. **Rule inheritance/override.** A tenant-specific rule with the same ID as a global rule should override it for that tenant (not both fire).

**Priority: P0** (multi-tenant rule management is core to MSSP)

---

## 9. Summary: Prism Priority Matrix

| Feature | Axiathon Status | Prism Priority | Prism Adaptation |
|---------|----------------|----------------|------------------|
| .axd DSL grammar (Pest) | Production-quality parser with 9 tests | P1 | Extend with `schedule` block for scheduled detection |
| Single-event detection | Working, benchmarked | P1 | Compile to DataFusion SQL predicates |
| Correlation (sliding window) | Working, DashMap-based | P2 streaming / P1 scheduled | SQL `GROUP BY ... HAVING` for scheduled; DashMap for real-time tier |
| Sequence detection | Working, ordered multi-step | P2 | Windowed JOINs for scheduled; streaming tracker for real-time |
| Alert generation + templates | Working, 3 factory functions | P1 | Add persistence, dedup, enrichment |
| Case management state machine | 5 states, 12 transitions, tested | P1 | Add persistence, RBAC, auto-case creation |
| Disposition system | 4-variant tagged enum | P1 | Adopt as-is |
| Field promotion lifecycle | 8 phases, integration-tested | P0 | Adopt fully; add recommendation engine |
| Tenant-scoped detection | Per-tenant engine maps | P0 | Add global/tenant/client rule scopes |
| Detection security hardening | Missing (no limits in DSL) | P0 | Add from day 1 (regex limits, nesting depth, rule size) |
| Alert persistence | In-memory only | P0 | Iceberg/Parquet or relational DB |
| SSE alert streaming | Working, tenant-filtered | P2 | WebSocket or SSE for real-time dashboard |

### Key Architectural Decisions for Prism

1. **Scheduled-first, streaming-optional.** Prism's MSSP model runs detection on stored data. Design the rule compiler to produce DataFusion SQL queries. Add a streaming tier later for critical real-time rules.

2. **Rule compilation, not interpretation.** Axiathon interprets rule conditions per-event. Prism should compile .axd rules into DataFusion logical plans at load time, then execute them over Parquet data. This leverages Parquet predicate pushdown, partition pruning, and columnar execution.

3. **Three-scope rule management** (global, tenant, client) from day 1. Axiathon's "duplicate rules per tenant" approach doesn't scale to 50+ MSSP clients.

4. **Security-hardened DSL.** Copy the production AxiQL parser's security limits (CWE-400, CWE-674, CWE-1333) into the detection DSL parser. Axiathon's spike detection parser has zero security controls.

5. **Persistent state everywhere.** Alerts, cases, correlation windows (for real-time tier), and sequence trackers must survive restarts. Axiathon's in-memory-only approach is the single biggest anti-pattern to avoid.

---

## State Checkpoint

```yaml
analysis: axiathon-detection-deep-dive
status: complete
files_analyzed:
  - spike/crates/axiathon-detection/src/ast.rs (198 LOC)
  - spike/crates/axiathon-detection/src/parser.rs (477 LOC)
  - spike/crates/axiathon-detection/src/engine.rs (543 LOC)
  - spike/crates/axiathon-detection/src/correlation.rs (316 LOC)
  - spike/crates/axiathon-detection/src/sequence.rs (361 LOC)
  - spike/crates/axiathon-detection/src/alert.rs (587 LOC)
  - spike/crates/axiathon-detection/src/case.rs (858 LOC)
  - spike/crates/axiathon-detection/src/detection.pest (157 LOC)
  - spike/crates/axiathon-detection/src/test_fixtures.rs (95 LOC)
  - spike/crates/axiathon-detection/benches/detection_stateless.rs (204 LOC)
  - spike/crates/axiathon-detection/benches/detection_stateful.rs (158 LOC)
  - spike/crates/axiathon-api/src/pipeline.rs (123 LOC)
  - spike/crates/axiathon-api/src/state.rs (~530 LOC)
  - spike/crates/axiathon-api/src/routes/rules.rs (309 LOC)
  - spike/crates/axiathon-api/src/routes/alerts.rs (98 LOC)
  - spike/crates/axiathon-storage/src/catalog.rs (field promotion)
  - spike/crates/axiathon-storage/tests/field_promotion.rs (547 LOC)
  - spike/rules/*.axd (6 files)
  - spike/crates/axiathon-core/src/event.rs (field resolution)
timestamp: 2026-04-13T00:00:00Z
```
