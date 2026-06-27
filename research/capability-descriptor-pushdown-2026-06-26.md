---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-26"
provenance: "side-analysis discussion input; does not modify vision/specs."
topic: "connector capability-descriptor + PrismQL pushdown + cross-source join guards"
feeds: "matured-vision-day2-requirements.md §3.4 / §12.2 / §5.3 / §13 / §17 (DISCUSSION input only)"
engine: "DataFusion (+ Chumsky parser); ephemeral/federated query thesis"
---

# Connector Capability-Descriptor + PrismQL Pushdown + Cross-Source Join Guards

**Side-analysis / discussion input — NOT a spec or vision change.** This document gathers cited prior art and offers *leans* to inform a discussion on day-2 §3.4 (per-connector capability descriptor), §12.2 (join-guard NFR), §5.3 (cross-source join cost guard + mandatory time-bound + result limits), §13 (static/dynamic, multi-schema), and §17 (the COLLECTOR push/buffer-backed subtype). It does not modify the vision, any spec, STATE.md, SESSION-HANDOFF.md, or prior research. `do_not_execute: true`.

> **Read-coverage honesty.** I could NOT locate `matured-vision-day2-requirements.md` on disk via Glob (file not present at the queried path on `develop`; the `.factory/` worktree may mount it elsewhere or it may be uncommitted). All section references (§3.4, §5.3, §12.2, §13, §17) are taken from the task brief's paraphrase, NOT from a direct read of the vision file. Where the brief's paraphrase and my prior-art findings interact, I flag it. A reviewer with the vision file open should re-check the §-anchors.

---

## Executive Summary (~12 lines)

1. **The capability-descriptor pattern is industry-settled**, but engines split into two schools: Trino/Spark use a *negotiation* SPI (`applyFilter`/`pushFilters` returns *what was pushed* + a *residual* the engine must re-apply), while DataFusion uses a *declaration* enum (`Exact`/`Inexact`/`Unsupported` per predicate). [Trino-SPI][DF-blog][Spark-SPDF]
2. **Correctness hinges on the residual.** Every engine's invariant: an unsupported/inexact pushed predicate MUST be re-checked centrally; silently dropping it is a correctness bug. DataFusion makes this explicit (`Inexact` → engine re-applies via `FilterExec`). [DF-blog]
3. **Prism's lean:** a *declarative* TOML descriptor (dogfood) whose runtime semantics map 1:1 onto DataFusion's `Exact/Inexact/Unsupported`. Declare predicate *classes* (eq/range/IN/prefix/LIKE/null), projection, aggregation set, group-by, sort, limit, join-type — each tagged with an exactness flag. Default for anything undeclared = `Unsupported` (fail-closed → central compute). [DF-CTP][DF-blog]
4. **DataFusion reality check (load-bearing):** as of DataFusion 50.x (Sept 2025), `TableProvider` pushdown is **filter + projection + limit ONLY**. There is **no** stable TableProvider API for aggregation, sort, or join pushdown. [DF-CTP-2026] Prism's descriptor can declare more than DataFusion's TableProvider natively consumes — the *spec-driven adapter* must implement the extra pushdown inside its own `scan()`/`ExecutionPlan`, not expect DataFusion to negotiate it.
5. **Cross-source joins are the single largest runaway risk** — confirmed: when two sources share no connector, NO engine can push the join down; it executes centrally. [Trino-PD] Trino's defenses are cost-based (broadcast-size cap, join reorder, dynamic filtering) — NOT hard rejection. [Trino-CBO][Trino-DF]
6. **Almost no engine HARD-rejects unbounded joins.** Trino offers only `ELIMINATE_CROSS_JOINS` (reorder heuristic) + general resource caps; explicit Cartesian rejection is rare (ScyllaDB caps certain `IN` Cartesian products). [Trino-JR][Scylla] Prism's proposed *mandatory selective key-predicate + row-cap reject* would be MORE aggressive than Trino — a legitimate, defensible design for a security tool, but novel.
7. **Dynamic filtering / semi-join reduction is the key cost-mitigation primitive** and DataFusion now has it: DataFusion 50.0.0 extended dynamic-filter pushdown to **inner hash joins** ("Sideways Information Passing") at the `ExecutionPlan` level via `Arc<dyn PhysicalExpr>` (surfaced as `DynamicFilterPhysicalExpr`). [DF-50][DF-dynfilter] This is prism's strongest lever for cross-source join cost control without rejecting the query.
8. **Mandatory time-bound is a sound generalization** of the Security-Lake `eventDay`/`time_dt` guardrail, but is a *design choice*, not universal product behavior — I could NOT confirm any named platform that hard-rejects time-less queries; the common pattern is time-partition *pruning* + planner-injected default windows. [SecLake-pattern][Husky] Prism injecting a default window + default/max limit is defensible and within the federated thesis.
9. **Limit pushdown is real and standard** (DataFusion `scan(limit)`, Trino `applyLimit`/`applyTopN`); push where the descriptor declares support, enforce the residual cap centrally. [DF-CTP][Trino-SPI]
10. **Multi-schema (OCSF vs native, §13/§17):** the descriptor must be *per-(table, schema-class)* — OCSF-field predicates and native-field predicates can have different pushdown profiles because they map to different source-native fields (or to none, if OCSF-only-derived). [model-knowledge — no direct prior art; OCSF mapping is prism-internal]
11. **The COLLECTOR subtype (§17)** cleanly fits the descriptor model: declare `pushdown_target: buffer` (not source); pushdown predicates apply to the in-prism buffer, never to a push-only source that cannot be queried. This is analogous to a streaming/`SymmetricHashJoinExec` bounded-buffer source in DataFusion. [DF-physplan][model-knowledge]
12. **Honest cost:** the declarative-descriptor + fail-closed model is the cheapest-correct option, but it pushes real implementation work into the spec-driven adapter (it must honor exactness, re-apply residuals, and enforce join/time/limit guards in its own execution path). The hard-reject join guard needs a planner hook DataFusion does not provide out-of-box — prism must wrap/extend the optimizer or gate at PrismQL plan-time before handing to DataFusion.

---

## Topic 1 — Per-Connector Capability Descriptor Model

### Prior art

**Two architectural schools:**

**(A) Negotiation SPI (Trino, Spark).** The connector is *handed* a request and returns *what it took* plus a *residual*.

- **Trino `ConnectorMetadata`** exposes `applyFilter`, `applyProjection`, `applyAggregation`, `applyLimit`, `applyTopN`, `applyJoin`. `applyFilter(Constraint)` returns a `ConstraintApplicationResult` = new table handle + the residual `Constraint` the engine must still enforce. The `Constraint` carries both a `TupleDomain` *summary* (per-column allowed ranges/value-sets — ideal for partition pruning) AND a general `ConnectorExpression` predicate; they are ANDed. Trino has **no** explicit Exact/Inexact flag — inexactness is encoded by leaving the unenforced part in the residual. Aggregation pushdown explicitly does NOT support `ROLLUP`/`CUBE`/`GROUPING SETS`, expressions inside aggregates (`sum(a*b)`), or coercions (`sum(int_col)`). Predicate-class support is NOT centrally enumerated — it is per-connector. [Trino-SPI][Trino-PD]
- **Spark DataSourceV2** uses mix-in interfaces: `SupportsPushDownFilters` (`pushFilters(Filter[])` returns residual filters), `SupportsPushDownRequiredColumns`, `SupportsPushDownAggregates` (`pushAggregation` + `supportCompletePushDown`), `SupportsPushDownLimit`, `SupportsPushDownTopN`. Spark's `Filter` hierarchy enumerates concrete predicate classes: `EqualTo`, `GreaterThan`, `LessThan`, `In`, `IsNull`, `IsNotNull`, `StringStartsWith`, etc. Key correctness rule: aggregates **cannot** be pushed if any pushed filter still needs post-scan evaluation (inexact) — otherwise aggregation would run over an unfiltered set. [Spark-SPDF][Spark-SPDA]

**(B) Declaration enum (DataFusion).** The connector *declares* per-predicate capability up front.

- **`TableProvider::supports_filters_pushdown(&[&Expr]) -> Vec<TableProviderFilterPushDown>`** with variants `Exact` (source enforces fully, engine does NOT re-check), `Inexact` (source prunes approximately, engine MUST re-apply via `FilterExec`), `Unsupported` (engine evaluates entirely). The `scan(projection, filters, limit)` signature carries the projection/filter/limit hints. Conceptual interface from the DataFusion blog: `push_predicates(Vec<Expr>) -> { pushed, inexact, unhandled }`. [DF-CTP][DF-blog]
- DataFusion enumerates *pushable-as-exact* predicate classes: `=,<,<=,>,>=,!=` col-vs-const on stats-bearing cols; `IN (const_list)`; `IS NULL`/`IS NOT NULL`; AND/OR of pushable. *Pushable-but-inexact*: predicates on partition columns under non-tight transforms, coarse-stats columns. *Not pushable*: function-on-column, multi-table, derived/aggregate cols, regex/JSON-path. [DF-blog]
- **CRITICAL for prism:** as of DataFusion 50.x (2025–2026) the documented `TableProvider` trait supports filter + projection + limit pushdown ONLY — **no** TableProvider-level aggregation, sort/order, or join pushdown API exists. Sort pushdown is an in-progress optimizer/operator epic (issue #23036), not a stable trait method. [DF-CTP-2026][DF-sortepic]

**AWS Athena federated** identifies `WHERE`-clause predicates and delegates to the Lambda connector via the Athena Federation SDK, but public docs are high-level and do NOT specify the descriptor shape or a formal residual classification. [Athena-PD] [INCONCLUSIVE on Athena descriptor internals — docs are performance-oriented, not SPI-shape-oriented.]

### Lean — proposed prism capability-descriptor shape

A *declarative* TOML block per `[[tables]]` (dogfooding the spec-driven connector model, §13), runtime-mapped onto DataFusion's `Exact/Inexact/Unsupported`. Illustrative shape (discussion sketch, NOT a schema commitment):

```toml
[tables.alerts.pushdown]
# predicate classes, each tagged exact|inexact (absence ⇒ unsupported)
predicates = [
  { class = "eq",       columns = ["severity","status"], exactness = "exact" },
  { class = "range",     columns = ["time_dt"],           exactness = "exact" },   # mandatory time-bound carrier
  { class = "in_list",   columns = ["sensor_id"],         exactness = "exact" },
  { class = "prefix",    columns = ["hostname"],          exactness = "inexact" }, # API does substring-ish match
  # "like", "null_check" likewise; anything absent ⇒ Unsupported ⇒ central compute
]
projection      = { supported = true }                 # column selection → narrow API fieldset
limit           = { supported = true, exactness = "exact" }   # API has native page-size/limit
sort            = { supported = false }                # source returns unordered ⇒ central sort
aggregation     = { supported = false }                # almost always false for REST sensor APIs
group_by        = { supported = false }
join            = { supported = false }                # sensor APIs never join ⇒ always central
pushdown_target = "source"                             # COLLECTOR subtype sets "buffer" (Topic-5/§17)
```

Design rules:
- **Fail-closed default.** Anything undeclared = `Unsupported` → prism computes centrally. This is the production-grade default: a missing declaration must NEVER silently drop a predicate. [DF-blog correctness invariant]
- **Exactness is mandatory per declared class.** Mirrors DataFusion's three-way enum; an `inexact` class means the spec-driven adapter MUST emit the residual predicate so the central engine re-checks it.
- **Declare classes, not free expressions** (Spark's enumerated-`Filter` model is the better fit for a TOML spec than Trino's open `ConnectorExpression`, which assumes a code connector that can interpret arbitrary expression trees). Sensor REST APIs map cleanly to a fixed class vocabulary.
- **Aggregation/group-by/join almost always `false`** for sensor REST adapters — and even if `true`, DataFusion's TableProvider won't negotiate them, so the adapter would have to issue the aggregate as a native API call and present pre-aggregated rows itself.

### Open Qs (Topic 1)
- Does prism want Trino-style *open expression* pushdown (for a future code-connector that can interpret arbitrary predicates) or is the enumerated-class vocabulary sufficient forever? Lean: enumerated classes; revisit only if a connector genuinely needs arbitrary-expression offload.
- Should the descriptor be *static TOML only*, or also support *dynamic introspection* (§13) where a connector probes the live API's capabilities at boot? Lean: static TOML is the dogfood default; dynamic introspection is a per-connector opt-in that *narrows* (never widens beyond) the declared static profile — a connector may discover at runtime that a declared-supported predicate is unavailable and downgrade to `Unsupported`, but must not silently upgrade.
- How does the descriptor version/evolve under `#[non_exhaustive]` discipline? (prism CLAUDE.md requires `#[non_exhaustive]` on TOML-deserialized pub types — the descriptor structs inherit this.)

---

## Topic 2 — PrismQL Planner Push-vs-Local Contract

### Prior art
- The universal invariant across Trino/Spark/DataFusion: **push what the source supports; re-apply the residual centrally; never assume an unsupported pushdown silently drops a predicate.** Trino encodes residual in `ConstraintApplicationResult`; Spark in the residual array from `pushFilters`; DataFusion in `Inexact`→`FilterExec`. [Trino-SPI][Spark-SPDF][DF-blog]
- DataFusion's optimizer inserts a `FilterExec` above `scan` for every `Inexact`/`Unsupported` predicate; `Exact` predicates are omitted (trusted to the source). Two-stage evaluation: coarse source pruning then precise central re-check. [DF-blog]

### Lean — map prism's contract onto DataFusion
1. **PrismQL plan-time:** the parser (Chumsky) → logical plan. Before/at TableProvider binding, the planner consults the connector's declared descriptor.
2. **Per predicate:** declared `exact` → push to adapter, omit from central residual. Declared `inexact` → push AND keep central residual `FilterExec`. Undeclared/`unsupported` → do NOT push, central `FilterExec` only. This is a direct restatement of `supports_filters_pushdown`. [DF-CTP]
3. **The spec-driven adapter's `scan()`** translates pushed predicates into native API query params (e.g., `?severity=high&since=...`). It MUST honor exactness: for an `inexact` class it may over-return rows (the central `FilterExec` cleans up); for an `exact` class it must return exactly the matching set.
4. **Aggregation/sort/join:** since DataFusion TableProvider won't negotiate these, if a descriptor ever declares them supported, the adapter must implement them as a custom `ExecutionPlan` (or issue a native aggregate API call and present pre-reduced rows). Otherwise they are ALWAYS central. For sensor adapters this is the expected case. [DF-CTP-2026]
5. **COLLECTOR (§17):** descriptor declares `pushdown_target = buffer`. The planner pushes predicates to the buffer scan, never to the (un-queryable) push source. Buffer behaves like an in-memory `MemTable`/bounded stream source — standard DataFusion filter/projection/limit pushdown applies to it.

### Open Qs (Topic 2)
- Where exactly does prism enforce the contract — inside DataFusion's optimizer (custom `OptimizerRule`/`TableProvider` impl) or in a PrismQL pre-planning pass before DataFusion sees the plan? Lean: TableProvider `supports_filters_pushdown` for the filter/projection/limit subset DataFusion natively handles; a PrismQL pre-pass for the guards DataFusion does NOT model (join reject, mandatory time-bound, default limit injection — Topics 3/4).
- How are residuals surfaced for audit (the agent-harness consumes the query plan)? Lean: emit a structured event recording which predicates were pushed vs residual-checked (ties into the Canonical Structured Event Catalog discipline; an `event_type = "query.pushdown.decision"` row would need a BC-2.16.002 catalog entry per CLAUDE.md SAP-1 — flagged as a downstream spec dependency, NOT actioned here).

---

## Topic 3 — Cross-Source Join Guards + Distributed Join Strategy

### Prior art
- **Confirmed runaway risk:** when two tables come from different catalogs/connectors, join pushdown is *impossible* (Trino requires same-catalog + all join predicates pushable); the join executes entirely in-engine. [Trino-PD]
- **Trino distributed join strategies:** `join-distribution-type` = `PARTITIONED` (hash-redistribute both sides), `BROADCAST` (replicate small right side to all nodes), or `AUTOMATIC` (cost-based, default; falls back to PARTITIONED when stats absent). Broadcast guarded by `join-max-broadcast-table-size` (default 100 MB) so a mis-estimated broadcast cannot blow up. Join *ordering* via `optimizer.join-reordering-strategy` = `AUTOMATIC` (full CBO enumeration) / `ELIMINATE_CROSS_JOINS` (heuristic cross-join removal) / `NONE`. [Trino-CBO][Trino-DF]
- **Dynamic filtering / semi-join reduction (the key primitive):** Trino collects build-side join-key values at runtime and pushes a derived predicate into the probe-side scan (dynamic partition pruning on Hive). Supports inner/right joins with `=,<,<=,>,>=,IS NOT DISTINCT FROM` and `IN` semi-joins. Guarded by `dynamic-filtering.max-distinct-values-per-driver`, `.max-size-per-driver`, `.range-row-limit-per-driver`. Bloom-filter dynamic filters proposed (GitHub issue) for large build sides — not yet production. [Trino-DF][Trino-Bloom]
- **DataFusion joins:** `HashJoinExec`, `SortMergeJoinExec`, `NestedLoopJoinExec` (non-equi), `CrossJoinExec` (explicit Cartesian), `SymmetricHashJoinExec` (streaming/bounded-window). Optimizer inserts `RepartitionExec` for distribution; join selection is heuristic/stats-driven (lighter than Trino's CBO). [DF-physplan]
- **DataFusion dynamic filters NOW EXIST (load-bearing):** DataFusion 50.0.0 (Sept 2025) extended dynamic-filter pushdown to **inner hash joins** — "Sideways Information Passing" — pushing build-side key knowledge into the probe-side scan as `Arc<dyn PhysicalExpr>` (`DynamicFilterPhysicalExpr`), implemented at the `ExecutionPlan` level. [DF-50][DF-dynfilter]
- **Hard rejection is rare.** No engine in the survey hard-rejects unbounded joins on selectivity grounds; Trino offers only cross-join *elimination* (a reorder heuristic) + general resource caps (`query_max_run_time`, memory limits) that abort *after* consumption. The lone explicit example: ScyllaDB caps the Cartesian product size of certain `IN`-clause queries and rejects beyond the cap. [Trino-JR][Scylla]

### Lean — prism's join-guard
prism's proposed guard (reject unbounded joins; require a selective key-based predicate; row-cap + override) is **more aggressive than Trino's defaults and defensible for a security tool**, because (a) sources are remote sensor APIs with no native join + often poor/absent statistics, so CBO has little to work with; (b) an ephemeral federated engine cannot afford a runaway Cartesian over the network.

Concretely:
1. **PrismQL plan-time guard (before DataFusion execution):** if a query joins two distinct sources AND lacks a selective key-based equi-predicate on the join, **reject** with a structured `E-QUERY-NNN` error (cite the missing-predicate). This is the ScyllaDB-style hard gate, applied at plan-time not runtime. DataFusion provides no such gate natively → prism must implement it as a PrismQL pre-pass / custom optimizer rule. [Scylla][DF-CTP-2026]
2. **Mandatory row-cap** on cross-source join inputs and output (default + max), with limit-pushdown into each side's scan where the descriptor declares limit support (Topic 4).
3. **Exploit DataFusion 50.x dynamic filtering** as the primary cost mitigation: ensure the smaller/more-selective source is the build side so its keys prune the probe-side source's scan via sideways information passing. This is prism's biggest lever to make a *necessary* cross-source join affordable rather than rejecting it. [DF-50][DF-dynfilter]
4. **Default join distribution:** with no reliable source statistics, default to a partitioned/hash strategy (Trino's safe fallback) and NEVER broadcast a source whose size is unknown. [Trino-DF]
5. **No silent cross join.** A literal `CROSS JOIN` / comma-join across sources without a key predicate is the prime reject target.

### Open Qs (Topic 3)
- What counts as a "selective key-based predicate"? Equi-join on a declared key column? Plus a bounded value-set / IN-list cardinality cap? Lean: require an equi-predicate on a column the descriptor marks as a join-eligible key, AND that the build side carries an effective time-bound + limit (so the dynamic filter set is bounded).
- Override mechanism (§5.3 mentions "reject/override + row cap"): who can override, and is the override audited? Lean: explicit query-level escape hatch (e.g., a PrismQL hint) that raises the row cap to the configured max but still cannot exceed the absolute max; every override emits a structured audit event.
- Can prism rely on DataFusion's inner-hash-join dynamic filter, or does it need outer-join / non-equi coverage that DataFusion 50.x does not yet push? [INCONCLUSIVE — DF-50 notes specify *inner* hash joins; outer/non-equi sideways-passing status not confirmed.] Lean: restrict pushed-dynamic-filter optimization to inner equi-joins (which is also the only join shape prism's guard would allow unbounded-free), sidestepping the gap.

---

## Topic 4 — Mandatory Time-Bound + Default/Max Limit

### Prior art
- The Security-Lake `eventDay`/`time_dt` pattern is a *partition-projection pruning* mechanism; time-partitioned engines (e.g., Datadog Husky) prune fragments by the query's time range and split queries into time-based steps. [Husky]
- **Limit pushdown is standard:** DataFusion `scan(limit)` and Trino `applyLimit`/`applyTopN` push a row cap to the source; for sort-heavy plans, `LIMIT` pushdown converts full buffering into a bounded top-K heap. [DF-CTP][Trino-SPI][QE-limit]
- **HONEST FINDING:** I could NOT confirm any named platform (Security Lake/Athena, Splunk, Elasticsearch/OpenSearch, Loki/Tempo, ClickHouse, Druid, Honeycomb, Chronosphere) that *hard-rejects* a query lacking a time predicate, nor documented auto-injection of a default window. The confirmed industry pattern is time-partition *pruning* + planner guidance to always filter on time — enforcement-by-rejection is a *design choice* some systems make at the front end, but the surveyed sources did not document it for the named products. [SecLake-pattern][Husky] [INCONCLUSIVE on universal hard-enforcement.]

### Lean — generalize to ALL prism sources
1. **Every federated query carries an effective time predicate.** If the user supplies an explicit time bound, use it; if not, **inject a default window** (configurable, e.g., last 24h) at PrismQL plan-time. This generalizes the Security-Lake guardrail and is defensible within the ephemeral/federated thesis (an unbounded time scan over remote sensor APIs is the other major runaway vector alongside joins). It is a design choice, not a copy of an existing product's hard gate — flag as such in discussion. [SecLake-pattern]
2. **Push the time predicate** as an `exact range` predicate into each source whose descriptor declares range-pushdown on its time column (most sensor APIs accept a `since`/`from`/`to`). Where a source can't filter by time (`unsupported`), prism applies it centrally as a residual `FilterExec` — but this means prism scanned the full window, so such sources should also carry a tight default limit.
3. **Default + maximum result limit**, with limit-pushdown where the descriptor's `limit.supported = true`. The default limit caps normal queries; the max limit is the absolute ceiling even under override. Residual (un-pushed) limit enforced centrally.
4. **Interaction with the join guard (Topic 3):** the mandatory time-bound is what makes a cross-source join's dynamic-filter set bounded — time-bound + limit on the build side bounds the sideways-passed key set. The two guards reinforce each other.

### Open Qs (Topic 4)
- Default window value and per-source override? Lean: global default with per-connector descriptor override (a slow/expensive source can declare a tighter default window).
- Does injecting a default window risk *silently* returning a misleadingly-narrow result to an AI agent consumer? This is an agent-harness UX concern (the agent must be *told* a default window was injected). Lean: always emit the effective time-bound in the response envelope + a structured event, so the consuming LLM reasons over the actual window. [model-knowledge — ties to project_agent_harness_design memory]
- Should time-bound enforcement be reject (like the join guard) or inject-default? Lean: inject-default for time (graceful, with disclosure), reject for unbounded cross-source joins (the higher-cost vector). Different vectors, different guard postures.

---

## Topic 5 — Multi-Schema Interaction (OCSF vs Native; §13.6 / §17.13)

### Prior art
- No direct external prior art maps cleanly: OCSF-normalized-vs-native schema-on-read is prism-specific. The general principle that *pushdown capability is per-(table, column) and tied to what the source can natively evaluate* is universal (Trino/Spark/DataFusion all declare capability per column/predicate). [Trino-SPI][Spark-SPDF][DF-CTP] [model-knowledge for the OCSF-specific mapping.]

### Lean
1. **Descriptor is per-(table, schema-class).** A predicate on an OCSF field pushes down ONLY if that OCSF field maps to a concrete native source field the API can filter on. An OCSF field that is *derived/computed* at the prism normalization boundary (no native source equivalent) is `unsupported` for pushdown → always central `FilterExec` after fetch+normalize.
2. **Native-field predicates** (schema-on-read against the raw source shape) can have a *different, often broader* pushdown profile than OCSF-field predicates, because they hit source-native fields directly. The descriptor should therefore carry pushdown declarations for BOTH schema classes where prism exposes both. [model-knowledge — ColumnType canonical naming per ADR-024 is the prism-internal anchor; the OCSF↔native mapping is the adapter's responsibility.]
3. **OT native tables (§17.13)** are just another schema class with their own descriptor block; nothing structurally new — they declare their own predicate-class support against their native fields.
4. **COLLECTOR (§17) restated:** `pushdown_target = buffer`. Predicates apply to the in-prism buffer (a `MemTable`-like or bounded-stream source); pushdown is full filter/projection/limit on the buffer because prism owns it. The push source itself is never queried. Analogous to DataFusion's `SymmetricHashJoinExec`/bounded-stream model where the queryable surface is prism's own buffer. [DF-physplan][model-knowledge]

### Open Qs (Topic 5)
- For an OCSF predicate that maps to a native field via a *transform* (e.g., severity-int → OCSF severity-string), is the pushdown `exact` or `inexact`? Lean: `exact` only if the transform is an order-preserving / value-preserving bijection over the predicate's operator; otherwise `inexact` (push a widened native predicate, re-check OCSF predicate centrally) — directly analogous to DataFusion's "partition column under non-tight transform ⇒ inexact" rule. [DF-blog]
- How does a single PrismQL query that references both OCSF and native columns of the same source resolve a unified pushdown set? Lean: compute the per-class pushable subset independently, push the union the adapter can express in one native call, residual-check the rest centrally.

---

## Consolidated Open Design Questions

1. **Descriptor expressiveness:** enumerated predicate-class vocabulary (Spark-style, TOML-friendly) vs open expression trees (Trino-style, code-connector). Lean: enumerated.
2. **Where the contract lives:** DataFusion `TableProvider::supports_filters_pushdown` for the filter/projection/limit subset; PrismQL pre-pass / custom optimizer rule for join-reject, mandatory time-bound, default-limit injection (DataFusion models none of these). This split is the central architectural decision.
3. **Join guard posture:** hard-reject unbounded cross-source joins (ScyllaDB-style, more aggressive than Trino) — confirm prism wants reject-not-degrade.
4. **Dynamic filtering reliance:** prism leans on DataFusion 50.x inner-hash-join sideways-information-passing as the primary join cost mitigation; confirm minimum DataFusion version and that prism restricts unbounded joins to inner equi-joins (matching DF's dynamic-filter coverage).
5. **Time-bound posture:** inject-default-with-disclosure (not reject) — confirm.
6. **Override/escape hatch:** who authorizes raising row caps, and the audit-event obligation.
7. **OCSF↔native transform exactness:** the bijection test for `exact` vs `inexact` on transformed predicates.
8. **Audit/observability:** pushdown-decision + injected-window + override events likely require new Canonical Structured Event Catalog rows in BC-2.16.002 (CLAUDE.md SAP-1) — a downstream spec dependency, NOT actioned here.

## Honest Costs & Caveats

- **DataFusion does the least of the surveyed engines** for non-filter pushdown. Any aggregation/sort/join pushdown prism declares must be implemented by prism's spec-driven adapter inside its own `scan()`/`ExecutionPlan` — DataFusion will not negotiate it. This is real, non-trivial implementation cost concentrated in the adapter layer. [DF-CTP-2026]
- **The hard join-reject guard has no DataFusion out-of-box hook.** prism must gate at PrismQL plan-time (a pre-DataFusion pass) or wrap the optimizer. Cost: a custom planning stage + a maintained join-shape analysis. [Trino-JR shows even mature engines only do soft elimination.]
- **Dynamic-filter coverage gap:** DataFusion 50.x sideways-info-passing is documented for *inner* hash joins; outer/non-equi coverage unconfirmed. prism's guard restricting to inner equi-joins conveniently aligns, but this is a constraint on PrismQL join expressiveness that should be a conscious decision. [INCONCLUSIVE on DF outer-join dynamic-filter status.]
- **Mandatory time-bound is a prism design choice**, not a copy of a confirmed product behavior. The closest confirmed analog is time-partition pruning + planner guidance, not hard rejection. Defensible, but own it as a novel guardrail. [SecLake-pattern][Husky]
- **`matured-vision-day2-requirements.md` was not read directly** (not found via Glob on `develop`). All §-anchors are from the task brief paraphrase. A reviewer should reconcile against the actual vision file.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) Capability-descriptor / pushdown SPI deep comparison across Trino, DataFusion, Spark DSv2, Athena (Topics 1–2). (2) Cross-source distributed join strategies, dynamic filtering / semi-join reduction, broadcast vs partitioned, cost guards, Cartesian rejection across Trino/Presto, DataFusion, ScyllaDB, BigQuery (Topics 3–4). Both `reasoning_effort=high`. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 2 | (1) Current (2025–2026) DataFusion TableProvider pushdown scope + dynamic-filter API state (DataFusion 50.x) — load-bearing version verification. (2) Mandatory-time-bound / default-window / limit-pushdown enforcement across observability+security platforms (Topic 4) — returned an honest "could not confirm hard-enforcement" result. |
| Context7 | 0 | Not used; DataFusion behavior verified via Perplexity against official datafusion.apache.org docs + release notes + GitHub issues (more current than Context7's snapshot for the 50.0.0 dynamic-filter feature). |
| Tavily (all) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Glob | 1 | Attempt to locate `matured-vision-day2-requirements.md` (NOT found on `develop` — read-coverage caveat flagged). |
| Training data | ~3 areas | OCSF↔native mapping specifics (prism-internal, no external prior art); COLLECTOR-as-bounded-stream analogy; agent-harness disclosure UX. All flagged inline as [model-knowledge]. |

**Total MCP tool calls:** 4 (2 × `perplexity_research` high-effort + 2 × `perplexity_ask`).
**Training data reliance:** low–medium — core pushdown/join/dynamic-filter claims are all web-sourced and version-verified (DataFusion 50.0.0 confirmed current); only the prism-specific OCSF mapping and analogies rely on model knowledge, each flagged.

### Citation key (sources from MCP web findings)
- **[Trino-SPI]** Trino Connector SPI / javadoc — `ConnectorMetadata` apply* methods, `Constraint`, `ConnectorExpression`.
- **[Trino-PD]** Trino docs — Pushdown (filter/projection/aggregation/join; join requires same catalog).
- **[Trino-CBO]** Trino docs/blog — Cost-Based Optimizer (statistics, join enumeration).
- **[Trino-DF]** Trino docs — Dynamic Filtering (`join-distribution-type`, `join-max-broadcast-table-size`, dynamic-filtering.* caps).
- **[Trino-JR]** Trino docs — `optimizer.join-reordering-strategy` (AUTOMATIC / ELIMINATE_CROSS_JOINS / NONE).
- **[Trino-Bloom]** Trino GitHub issue — Bloom-filter dynamic filtering proposal.
- **[Spark-SPDF]** Spark DataSourceV2 javadoc — `SupportsPushDownFilters` (`pushFilters`, residual, 3 filter kinds).
- **[Spark-SPDA]** Spark DataSourceV2 javadoc — `SupportsPushDownAggregates` (`pushAggregation`, `supportCompletePushDown`; no-push-if-inexact-filter rule).
- **[DF-CTP]** / **[DF-CTP-2026]** datafusion.apache.org — Custom Table Providers guide (`scan(projection,filters,limit)`, `supports_filters_pushdown`); 2026-updated guide confirming filter+projection+limit-only scope.
- **[DF-blog]** datafusion.apache.org blog — Predicate Pushdown (`Exact/Inexact/Unsupported`, `push_predicates` conceptual interface, pushable predicate classes, FilterExec residual).
- **[DF-50]** datafusion.apache.org — DataFusion 50.0.0 release notes (dynamic filter pushdown extended to inner hash joins; Sideways Information Passing).
- **[DF-dynfilter]** datafusion.apache.org blog (2025-09-10) — Dynamic Filters (`Arc<dyn PhysicalExpr>`, ExecutionPlan-level).
- **[DF-sortepic]** apache/datafusion GitHub issue #23036 — sort pushdown epic, `DynamicFilterPhysicalExpr`.
- **[DF-physplan]** datafusion.apache.org — physical plan operators (`HashJoinExec`, `SortMergeJoinExec`, `NestedLoopJoinExec`, `CrossJoinExec`, `SymmetricHashJoinExec`, `RepartitionExec`).
- **[Athena-PD]** AWS blog — federated query predicate pushdown (high-level; descriptor internals INCONCLUSIVE).
- **[Scylla]** ScyllaDB docs — Cartesian product limit on certain `IN` queries (explicit reject example).
- **[SecLake-pattern]** AWS Security Lake / Athena partition-projection time pruning (`eventDay`/`time_dt`) — pattern confirmed; hard-reject NOT confirmed.
- **[Husky]** Datadog Husky query architecture blog — time-range fragment pruning + time-based query splitting.
- **[QE-limit]** Query-engine execution internals — limit pushdown → bounded top-K heap.
