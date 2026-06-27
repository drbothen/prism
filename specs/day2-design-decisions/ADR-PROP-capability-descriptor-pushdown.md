---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C3-1: Join guard posture — cost-based degrade (NOT hard-reject)"
  - "ADR-PROP-C3-2: Missing time-bound — inject default window + disclose (NOT reject)"
  - "ADR-PROP-C3-3: Cross-source join shape — allow outer/non-equi (central-only)"
  - "ADR-PROP-C3-4: Override — audited PrismQL hint, capped at absolute max"
  - "ADR-PROP-C3-5: Declarative TOML capability descriptor per [[tables]], fail-closed"
  - "ADR-PROP-C3-6: Enumerated predicate-class vocabulary (eq/range/in_list/prefix/like/null)"
  - "ADR-PROP-C3-7: Contract split — DataFusion TableProvider vs PrismQL pre-pass"
  - "ADR-PROP-C3-8: Descriptor is per-(table, schema-class)"
  - "ADR-PROP-C3-9: Transform exactness — bijection test for exact vs inexact"
  - "ADR-PROP-C3-10: Collector subtype — pushdown_target = buffer"
produced_by: architect
timestamp: "2026-06-27"
provenance: "side-analysis C3 capture; human-confirmed decisions 2026-06-27 session. Research basis: research/capability-descriptor-pushdown-2026-06-26.md (2 perplexity_research sonar-deep-research calls at reasoning_effort=high + 2 perplexity_ask). Hardening pass on DataFusion mechanics in flight: research/datafusion-cost-degrade-mechanics-2026-06-27.md (see Open Questions). Does NOT modify live ADR files, ARCH-INDEX.md, STATE.md, SESSION-HANDOFF.md, or any live factory artifact."
traces_to:
  - matured-vision-day2-requirements.md §3.4 (per-connector capability descriptor addendum)
  - matured-vision-day2-requirements.md §5.3 (cross-source join cost guard NFR, mandatory time-bound NFR)
  - matured-vision-day2-requirements.md §12.2 (NFR-JOIN-GUARD concrete specification)
  - matured-vision-day2-requirements.md §13.6 (multi-schema; static/dynamic schema classes)
  - matured-vision-day2-requirements.md §17.2 (collector-connector subtype; descriptor pushdown_target=buffer)
  - matured-vision-day2-requirements.md §16.4 (C3 decisions log entry)
  - day2-design-decisions/ADR-PROP-satellite-mesh.md (Satellite mesh; per-hop deadline reinforces time-bound guard)
  - day2-design-decisions/ADR-PROP-storage-engine-taxonomy.md (DataFusion TableProvider + Iceberg cold tier)
  - research/capability-descriptor-pushdown-2026-06-26.md (primary research basis — all five topics)
  - CLAUDE.md (non_exhaustive discipline; SAP-1 structured event catalog; error taxonomy E-QUERY-NNN)
---

# ADR-PROP — Connector Capability-Descriptor + PrismQL Pushdown + Cross-Source Cost Guards

> **STATUS: DECIDED 2026-06-27 (human).** This is a CAPTURE artifact for the side-analysis C3
> program. `do_not_execute: true`. Real ADR numbers and formal ARCH-INDEX.md rows are deferred to
> the morph execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

> **Research basis:** `research/capability-descriptor-pushdown-2026-06-26.md` — two
> `perplexity_research` (sonar-deep-research, `reasoning_effort=high`) calls covering
> capability-descriptor / pushdown SPI deep comparison (Topics 1–2) and cross-source distributed
> join strategies, dynamic filtering, broadcast vs partitioned, cost guards (Topics 3–4), plus two
> `perplexity_ask` calls for DataFusion 50.x state and mandatory-time-bound platform survey.
> All load-bearing claims are source-grounded in that research document. Claims from model knowledge
> are flagged `[model-knowledge]`.

> **Hardening research in flight:** `research/datafusion-cost-degrade-mechanics-2026-06-27.md`
> (verifying DataFusion 50.x MemoryPool/disk-spill, timeout cancellation, broadcast-of-unknown-size
> guard, NestedLoopJoinExec cost + spill, and row-cap enforcement mechanics). Open Questions
> OQ-C3-1..OQ-C3-6 below are explicitly flagged "resolution pending hardening pass — fold on return."
> Do not block the capture on that pass.

> **§5.3/§12.2 Reconciliation.** The earlier §5.3 addendum and §12.2 NFR-JOIN-GUARD (DRAFT)
> specified that cross-source cross-products and non-equi-only cross-source joins are *rejected at
> plan time* (equality-key requirement #1, "A side that cannot be bounded is rejected" requirement
> #2). D-C3-1 and D-C3-3 (decided 2026-06-27, human) supersede that framing. **The operative guard
> is cost-based degrade, NOT hard-reject.** Later-more-specific-artifact-wins per CLAUDE.md
> Source-of-Truth Precedence. See §5.3/§12.2 reconciliation notes appended to
> `matured-vision-day2-requirements.md`.

---

## Context

Prism's §3.4 source/connector taxonomy generalizes the product from four security sensors to any
queryable source valuable to a security analyst. That generalization requires a technical spine:
every connector must declare what it can execute natively (at the source) vs. what PrismQL must
compute centrally in DataFusion. Without this contract, "any source" has no planner contract, and
cross-source joins and unbounded time scans become silent runaway risks.

The five driving design questions (from §3.4 addendum and §10.5 gap G-2):

1. **Descriptor model:** what shape does a connector declare its pushdown capabilities in?
2. **Planner contract:** how does PrismQL decide what to push vs execute centrally?
3. **Join guard:** how does PrismQL bound the cost of a cross-source join it cannot push?
4. **Time-bound:** how does PrismQL guarantee every query carries an effective time bound?
5. **Multi-schema:** how does the descriptor handle OCSF vs native schema predicate classes?

The research establishes that the industry has two pushdown architectures — Trino/Spark's
negotiation SPI and DataFusion's declaration enum — and that DataFusion 50.x specifically provides
sideways-information-passing (dynamic filter) for inner hash joins. The decisions below resolve all
five questions. They are numbered D-C3-N and cross-reference the research.

---

## Decision Ledger

### D-C3-1 — Join Guard Posture: COST-BASED DEGRADE (Trino-Lineage), Not Hard-Reject

**DECIDED 2026-06-27 (human).**

**Cross-source joins are ALLOWED.** The join guard is a cost-mitigation stack, not a rejection gate.

**What this means:**
- A PrismQL query that joins two sources which cannot execute the join natively (different
  connectors, no join-pushdown declared) executes the join centrally in DataFusion.
- Cost is bounded by: (a) mandatory per-side row-caps, (b) dynamic filtering (sideways-information-
  passing, DataFusion 50.x) as the primary cost lever, (c) partitioned hash distribution (never
  broadcast an unknown-size source), (d) resource-based abort backstop (query wall-clock timeout +
  memory limit).
- There is NO plan-time hard-rejection of an unbounded join. The cost-guards run at execution time
  and abort after consumption once limits are exceeded.

**Why degrade, not reject:**
The research [research/capability-descriptor-pushdown-2026-06-26.md §Topic 3] confirms that no
production engine in the survey (Trino, DataFusion, Spark, BigQuery) hard-rejects unbounded
cross-source joins at the planner. Trino's defenses are all cost-based (broadcast-size cap, dynamic
filtering, join reorder heuristics, resource timeouts). Hard-reject at plan time requires statistics
Prism cannot reliably obtain from sensor APIs with unknown result sizes. A cost-based-degrade guard
that enforces row-caps and aborts with structured errors + partial-result metadata is both safer
(doesn't reject valid queries that happen to lack predicable selectivity estimates) and more
consistent with the ephemeral/partial-failure thesis (abort-with-coverage, not silent reject).

**§5.3/§12.2 Supersedes:**
The §5.3 addendum (2026-06-25) and §12.2 NFR-JOIN-GUARD DRAFT both specified that cross-source
cross-products and non-equi-only joins are **rejected at plan time**. D-C3-1 supersedes that framing
with cost-based degrade as the operative posture. The equality-key / per-side-selectivity "reject"
language in §12.2 requirements #1 and #2 should be read as triggering the degrade path (cap + flag
+ disclose), not a hard planner error. D-C3-3 makes the outer/non-equi posture explicit.

[research/capability-descriptor-pushdown-2026-06-26.md §Topic 3]

---

### D-C3-2 — Missing Time-Bound: INJECT DEFAULT WINDOW + DISCLOSE, Not Reject

**DECIDED 2026-06-27 (human).**

**If a query carries no explicit time bound, PrismQL injects a configurable default window** (e.g.,
last 24 hours) at plan time and pushes it as an exact range predicate where the connector's
capability descriptor declares range-pushdown on the time column.

**Disclosure is mandatory.** The effective time-bound MUST surface in:
1. The query response envelope (a `time_window_injected: true` field with the effective range).
2. A structured audit event (`event_type = "query.injected_default_window"`, see Open Questions
   re: BC-2.16.002 catalog dependency below).

This ensures the consuming LLM agent reasons over the actual window, not a silently-narrowed one.
Agent-harness transparency is a first-class constraint per `project_agent_harness_design.md`.

**Why inject-and-disclose, not reject:**
D-C3-2 is asymmetric with D-C3-1 by design. Cross-source joins are the higher-cost runaway vector
(two potentially large source scans, centrally joined); missing a time bound on a single-source
query is a usability concern (user may not have realized the query was unbounded), not a safety
catastrophe. Injection-with-disclosure is the pattern confirmed by the Security Lake
`eventDay`/`time_dt` partition-projection model and Datadog Husky's time-fragment pruning
[research/capability-descriptor-pushdown-2026-06-26.md §Topic 4], generalized to all Prism sources.

**Where the injected predicate lands:**
- If the connector declares `range` pushdown on the time column as `exact`: the injected predicate
  is pushed to the connector scan; the central residual filter is omitted.
- If `inexact`: pushed as a hint, central `FilterExec` residual re-checks.
- If `unsupported` (connector cannot filter by time): scan returns unbounded set; the injected
  predicate is applied centrally as a `FilterExec` residual. Such a connector SHOULD carry a tight
  default row-cap in its descriptor to compensate.

**Reinforcement with D-C3-1:** The mandatory time-bound (D-C3-2) bounds the build side of a
cross-source inner equi-join's dynamic filter set (D-C3-1). The two guards reinforce each other:
an effective time-bound on both sides keeps the sideways-passed key set tractable for DataFusion's
dynamic filter to prune the probe side.

[research/capability-descriptor-pushdown-2026-06-26.md §Topic 4]

---

### D-C3-3 — Cross-Source Join Shape: ALLOW OUTER/NON-EQUI (Central-Only)

**DECIDED 2026-06-27 (human).**

**Inner equi-joins:** eligible for DataFusion 50.x dynamic filter (sideways-information-passing).
The smaller / more-selective source is placed as the build side; its join-key values prune the
probe-side scan at runtime.

**Outer/non-equi cross-source joins:** ALLOWED, but fall back to full central execution in DataFusion's
`NestedLoopJoinExec`. They do NOT receive the dynamic-filter cost reduction (DataFusion 50.x
sideways-information-passing is documented for inner hash joins only). The cost guarantee is weaker;
the row-cap still applies on both sides and the output.

**Bare cross-source CROSS JOIN / comma-join (implicit Cartesian):** ALLOWED, but LOUDLY FLAGGED.
The response envelope MUST include a cost/coverage disclosure. Bounded by the row-cap. Not silently
executed, not rejected.

**Rationale for allow-outer/non-equi:**
The research confirms that DataFusion 50.x dynamic filter covers inner hash joins. Outer/non-equi
joins execute via `NestedLoopJoinExec`, which does not yet receive the dynamic-filter benefit. Rather
than hard-banning these join shapes (which would be a significant PrismQL expressiveness constraint),
Prism allows them with the explicit acknowledgment that their cost guarantee is weaker. The row-cap
and resource-abort backstop still apply; the cost degradation is bounded, not unbounded.

**OQ-C3-1 (hardening pass):** Verify NestedLoopJoinExec / outer-join cost behavior and spill
characteristics in DataFusion 50.x under the MemoryPool configuration. Resolution pending
`research/datafusion-cost-degrade-mechanics-2026-06-27.md`. Fold on return.

[research/capability-descriptor-pushdown-2026-06-26.md §Topic 3, §Honest Costs]

---

### D-C3-4 — Override: AUDITED PrismQL HINT, Capped at Absolute Max

**DECIDED 2026-06-27 (human).**

An explicit query-level escape hatch (`-- PRISM HINT: ALLOW_LARGE_JOIN rows_per_side=N`) can raise
the per-side row-cap toward the configured maximum, but NEVER beyond the absolute maximum configured
at deployment. The equality-key requirement and time-bound injection (D-C3-1, D-C3-2) are not
bypassable by any override.

**Every override emits a structured audit event** (`event_type = "query.override_applied"`) with
the requesting context, the original cap, the overridden cap, and the query ID. This event MUST
have a BC-2.16.002 catalog row at morph time (SAP-1 downstream dependency, see Open Questions).

**Override authorization model (open question):** whether the hint is user-level or requires a
deployment-level grant is a morph-time decision. The hard invariant is: no query can exceed the
absolute max regardless of hint, and every use is audited.

[research/capability-descriptor-pushdown-2026-06-26.md §Topic 3 open questions]

---

## Resulting Guardrail Model (Operative Guard Stack)

This is the consequence of D-C3-1 / D-C3-3 — the full stack of cost guards, in order of activation:

| Guard | Mechanism | When Activated |
|-------|-----------|---------------|
| **Default time-bound injection** | PrismQL pre-pass injects the configured default window; pushes as range predicate; discloses in response envelope | Every query lacking an explicit time predicate |
| **Per-side row-cap** | Enforced on each source's scan in the cross-source join; limit-pushdown where descriptor declares `limit.supported = true` | Every cross-source join input and output |
| **Dynamic filtering (DataFusion 50.x)** | Inner equi-join sideways-information-passing: build side's key set pruned into probe-side scan | Inner equi-joins where the smaller side is the build side |
| **Partitioned hash distribution** | Default join distribution for unknown-cardinality sources; NEVER broadcast a source of unknown size | All cross-source joins; see OQ-C3-2 for DataFusion enforcement detail |
| **Resource-based abort** | Query wall-clock timeout + MemoryPool limit abort the query after consumption; surfaces partial-result coverage metadata per BC-2.01.010 / §3.6 | Queries that exceed time or memory budget mid-execution |
| **Cost/coverage disclosure** | Bare CROSS JOIN / comma-join flagged in response envelope with cost disclosure | Any cross-source Cartesian |

**Override hook:** the PrismQL hint (D-C3-4) raises the row-cap within the absolute max; every
use is audited.

---

## Confirmed Leans

These leans were presented in the research, not objected to by human, and are captured as decided
for purposes of morph-time ADR authorship.

### L-C3-1 — Declarative TOML Capability Descriptor per [[tables]], Fail-Closed Default

Every connector TOML `[[tables]]` block carries a `[tables.NAME.pushdown]` section declaring the
connector's pushdown profile. Runtime, this maps 1:1 onto DataFusion's `Exact / Inexact /
Unsupported` per predicate class.

**Fail-closed:** anything undeclared defaults to `Unsupported` → DataFusion computes centrally.
A missing declaration MUST NEVER silently drop a predicate (the universal correctness invariant
across Trino, Spark, DataFusion). [research/capability-descriptor-pushdown-2026-06-26.md §Topic 1]

Illustrative TOML shape (discussion sketch, NOT a final schema commitment):

```toml
[tables.alerts.pushdown]
predicates = [
  { class = "eq",      columns = ["severity","status"], exactness = "exact" },
  { class = "range",   columns = ["time_dt"],           exactness = "exact" },
  { class = "in_list", columns = ["sensor_id"],         exactness = "exact" },
  { class = "prefix",  columns = ["hostname"],          exactness = "inexact" },
]
projection      = { supported = true }
limit           = { supported = true, exactness = "exact" }
sort            = { supported = false }
aggregation     = { supported = false }
group_by        = { supported = false }
join            = { supported = false }
pushdown_target = "source"   # "buffer" for COLLECTOR subtype — see L-C3-5
```

**`#[non_exhaustive]` on all descriptor structs.** Required by CLAUDE.md TOML-deserialized pub-type
discipline. External match arms must include a wildcard `_ => {}` arm.

### L-C3-2 — Enumerated Predicate-Class Vocabulary (Spark-Style, Not Open Expression Trees)

Predicate classes are declared by name: `eq`, `range`, `in_list`, `prefix`, `like`, `null`.
Each entry carries an `exactness` tag: `exact` or `inexact`. Anything not declared is `Unsupported`.

This is the Spark DataSourceV2 `Filter` enumeration model (concrete, TOML-expressible) rather than
the Trino open `ConnectorExpression` model (arbitrary expression trees, requires a code connector
that can parse and execute arbitrary AST nodes). Sensor REST adapters map cleanly to a fixed
class vocabulary; the Trino-style approach is unnecessary complexity for this use case. Revisit
only if a future code connector genuinely needs arbitrary-expression offload.
[research/capability-descriptor-pushdown-2026-06-26.md §Topic 1 lean]

### L-C3-3 — Contract Split: DataFusion TableProvider vs PrismQL Pre-Pass

The pushdown contract lives in two places:

**Part A — DataFusion `TableProvider::supports_filters_pushdown`:** handles the filter + projection
+ limit subset that DataFusion natively negotiates with a TableProvider. The spec-driven adapter
implements this trait method, consulting the descriptor to return `Exact / Inexact / Unsupported`
per predicate.

**Part B — PrismQL pre-pass / custom optimizer rule:** handles the guards DataFusion models NOT AT
ALL as TableProvider negotiation: default-limit injection, mandatory-time-bound default-window
injection, and cross-source join cost estimation / guardrail stack. These run BEFORE DataFusion
execution; DataFusion sees an already-annotated plan.

**DataFusion 50.x reality check (load-bearing):** `TableProvider` pushdown in DataFusion 50.x is
**filter + projection + limit ONLY.** There is NO stable TableProvider API for aggregation, sort,
or join pushdown. Any aggregation or sort a descriptor declares as supported must be implemented
inside the spec-driven adapter's own `scan()` / `ExecutionPlan` — DataFusion will not negotiate it.
For sensor REST adapters, aggregation and sort pushdown are almost always `supported = false`; this
is the expected and correct default.
[research/capability-descriptor-pushdown-2026-06-26.md §Topic 1, §Honest Costs; DF-CTP-2026]

### L-C3-4 — Descriptor is Per-(Table, Schema-Class)

The descriptor is authored per `(table, schema-class)` pairing. An `alerts` table that exposes
both an OCSF schema and a native schema carries two pushdown blocks — one for each schema class.

**OCSF-field predicates:** pushable only if the OCSF field maps to a concrete native source field
the API can filter on. An OCSF field derived/computed at the normalization boundary (no native
source equivalent) is `Unsupported` → always central `FilterExec` after fetch+normalize.

**Native-field predicates:** can have a broader pushdown profile than OCSF-field predicates because
they reference source-native API parameters directly.

This pairs with C4 dynamic-schema (where a connector's schema is introspected rather than static)
and C5 lake federation (where Iceberg schema-evolution carries OCSF version drift). Both scenarios
use the per-(table, schema-class) descriptor structure.
[research/capability-descriptor-pushdown-2026-06-26.md §Topic 5]

### L-C3-5 — Transform Exactness: Bijection Test for Exact vs Inexact on Transformed Predicates

When an OCSF field maps to a native field via a transform (e.g., severity integer → OCSF severity
string), the predicate's exactness is determined by the bijection test:

- **Exact**: the transform is an order-preserving / value-preserving bijection over the predicate's
  operator. The pushed native predicate returns exactly the rows that would pass the OCSF predicate.
- **Inexact**: the transform is not a tight bijection (e.g., a coarsening, bucketing, or a mapping
  that is not order-preserving). Push a widened native predicate; re-check the OCSF predicate
  centrally via `FilterExec` residual.

This is directly analogous to DataFusion's "partition column under non-tight transform ⇒ inexact"
rule. [research/capability-descriptor-pushdown-2026-06-26.md §Topic 5 lean; DF-blog]

### L-C3-6 — Collector Subtype: pushdown_target = buffer

A collector-connector (§17.2 subtype) declares `pushdown_target = "buffer"` in its descriptor.
Predicates apply to the in-prism buffer (the RetentionCache / Satellite store-and-forward
MemTable-like surface); they NEVER reach the push source (which is not queryable — it only pushes).

This gives the PrismQL join-guard (D-C3-1 guardrail model) the information it needs to reason
about a no-pushdown surface: a collector source has no pushdown to a live API, only buffer-local
predicate evaluation. DataFusion standard filter/projection/limit pushdown applies to the buffer
because Prism owns it (analogous to a `MemTable` or bounded-stream source).
[research/capability-descriptor-pushdown-2026-06-26.md §Topic 5]

### L-C3-7 — Dynamic Introspection (C4) May Only NARROW the Static Profile

Dynamic introspection at connector boot (C4 — where a connector probes a live API to discover
its current capabilities) may discover that a declared-supported predicate class is unavailable
at runtime. It may downgrade a declaration to `Unsupported`.

Dynamic introspection MUST NOT silently upgrade a declaration (e.g., add predicate support not
declared in the TOML). The TOML descriptor is the authoritative ceiling; dynamic introspection
narrows within that ceiling. This is a correctness invariant: the planner must be able to trust
that a pushed predicate is actually supported.

### L-C3-8 — Minimum DataFusion Version: 50.x (for Dynamic Filter / Sideways-Information-Passing)

D-C3-1's primary cost-mitigation lever (inner equi-join sideways-information-passing) requires
DataFusion 50.0.0+, which introduced `DynamicFilterPhysicalExpr` for inner hash joins. Prism's
`Cargo.toml` must pin `datafusion >= "50.0.0"` (minimum; exact version resolved at morph).
[research/capability-descriptor-pushdown-2026-06-26.md; DF-50; DF-dynfilter]

---

## Open Questions (Resolution Pending Hardening Pass)

The following questions are flagged with `OQ-C3-N`. Each is resolvable by the targeted hardening
research in `research/datafusion-cost-degrade-mechanics-2026-06-27.md`. Do NOT block the ADR-PROP
capture on them; fold each answer on the research pass's return.

| # | Question | Domain | Notes |
|---|---------|--------|-------|
| **OQ-C3-1** | **NestedLoopJoinExec cost + spill behavior.** D-C3-3 allows outer/non-equi cross-source joins via `NestedLoopJoinExec`. What is the memory footprint and spill behavior under the MemoryPool configuration? Does DataFusion 50.x spill NestedLoopJoin to disk, or is it memory-only (OOM risk)? | DataFusion internals | Resolution pending `datafusion-cost-degrade-mechanics-2026-06-27.md`. If memory-only, the row-cap and resource-abort backstop take on full responsibility for bounding the join's footprint. |
| **OQ-C3-2** | **Broadcast-of-unknown-size guard.** D-C3-1 guardrail model states "NEVER broadcast a source of unknown size." DataFusion's join planner may not enforce this natively — does it fall back to NestedLoop vs broadcast based on statistics presence? Prism may need to explicitly intercept the join distribution choice in the PrismQL pre-pass to force partitioned hash when source statistics are absent. | DataFusion join planning | Resolution pending `datafusion-cost-degrade-mechanics-2026-06-27.md`. This is the Trino `join-max-broadcast-table-size` analog — verify whether DataFusion has it or whether Prism must enforce. |
| **OQ-C3-3** | **Query wall-clock timeout cancellation propagation.** When a timeout fires mid-join, does DataFusion's cancellation token propagate into a mid-fetch TableProvider `scan()` call at the source? The spec-driven adapter must honor the cancellation to avoid hanging I/O after the DataFusion executor has declared timeout. | DataFusion + adapter I/O | Resolution pending `datafusion-cost-degrade-mechanics-2026-06-27.md`. If cancellation does not propagate into `scan()`, Prism must implement explicit cooperative cancellation at the adapter boundary. |
| **OQ-C3-4** | **MemoryPool / disk-spill for hash join.** Under what conditions does DataFusion 50.x's `HashJoinExec` spill to disk vs OOM when source cardinality exceeds the MemoryPool configuration? What is the Prism-recommended MemoryPool ceiling for cross-source join workloads? | DataFusion MemoryPool | Resolution pending `datafusion-cost-degrade-mechanics-2026-06-27.md`. |
| **OQ-C3-5** | **Row-cap enforcement mechanics.** The per-side row-cap on cross-source join inputs must be enforced before the join executes (cap the scan output) AND on the join output (cap the result). What is the canonical DataFusion mechanism — a `LimitExec` node wrapping the source plan, or a custom `ExecutionPlan` that counts rows and aborts? | DataFusion plan construction | Resolution pending `datafusion-cost-degrade-mechanics-2026-06-27.md`. |
| **OQ-C3-6** | **Cost estimation with absent statistics.** DataFusion's join optimizer relies on statistics to choose join distribution (broadcast vs hash-repartition) and determine the build side. For sensor REST adapters, statistics are absent or wildly estimated. What is DataFusion's fallback behavior when `TableStatistics` returns `None`? Does it default to the safer partitioned distribution? | DataFusion CBO | Resolution pending `datafusion-cost-degrade-mechanics-2026-06-27.md`. If the fallback is unsafe, Prism's PrismQL pre-pass must explicitly override the distribution choice. |

---

## Downstream Spec Dependencies (Note — Not Actioned Here)

These downstream artifact updates are flagged as morph-time dependencies. They are consequences of
the D-C3-N decisions, not part of this capture.

**SAP-1 obligations (BC-2.16.002 Canonical Structured Event Catalog new rows needed at morph):**
- `event_type = "query.pushdown.decision"` — records which predicates were pushed vs central residual
  per query; audit role = planner transparency; recurrence = per-query.
- `event_type = "query.injected_default_window"` — records that a missing time-bound was filled by
  the default window; includes the effective range; recurrence = per-query when injection fires.
- `event_type = "query.override_applied"` — records a D-C3-4 hint override; includes the original
  cap, overridden cap, query ID; audit role = cost-override audit; recurrence = per-query when hint
  present.

**BC families needed (PO scope at morph):** new BC families for the PrismQL pre-pass guardrails:
time-bound injection contract; row-cap enforcement contract; override-hint contract; cross-source
join cost-disclosure contract.

**§12.2 NFR-JOIN-GUARD amendment:** requirements #1 and #2 language ("rejected at plan time", "A
side that cannot be bounded is rejected") should be amended at morph to reflect cost-based-degrade
posture per D-C3-1. The equality-key and selectivity requirements remain as guard triggers — but
the response is cap + flag + cost disclosure, not a `E-QUERY-NNN` planner rejection.

---

## Honest Costs

| Cost | Description |
|------|-------------|
| **PrismQL pre-pass is real new code** | The mandatory-time-bound injector, row-cap enforcer, cross-source join cost estimator, and broadcast-distribution guard are all custom logic sitting between Chumsky parsing and DataFusion execution. DataFusion provides none of these out of box. |
| **Adapter exactness obligation** | Every spec-driven adapter's `scan()` must honor the `Inexact` contract: if pushed an inexact predicate, it may over-return rows (central `FilterExec` re-checks), but it MUST NOT under-return rows. Exactness bugs in adapters are silent correctness failures. |
| **Degrade-not-reject under absent statistics** | Without source cardinality estimates, the cost-based degrade model relies on row-caps and timeouts rather than plan-time rejection. A pathological query (large outer join over two unlimited sources with no row-cap predicates) will consume memory and wall-clock time before the abort fires. The row-cap backstop is the primary defense; the guardrail model's depth-of-defense matters. |
| **Broadcast guard may need Prism enforcement** | OQ-C3-2 is non-trivial: if DataFusion does not natively prevent broadcast of an unknown-size source, Prism must intercept the join distribution choice in the pre-pass. This is additional pre-pass complexity. |
| **Dynamic filter is inner-equi only** | D-C3-3 accepts that outer/non-equi joins get no dynamic-filter cost reduction. The row-cap is the only cost lever for those join shapes. This is a known, conscious weaker-guarantee for a rare but permitted query pattern. |
| **Non_exhaustive on descriptor structs** | `#[non_exhaustive]` on all TOML-deserialized descriptor structs is required (CLAUDE.md) but means external connectors (if Prism gains a plugin-style connector model in day-2+) must maintain wildcard match arms. Future code-connector API must account for this. |

---

## Alternatives Considered and Rejected

### Alternative A: Hard-Reject Unbounded Joins (Original §5.3/§12.2 Posture)

Plan-time rejection of cross-source joins lacking an equality predicate or bounded selectivity
(the framing in §12.2 NFR-JOIN-GUARD requirements #1 and #2 as originally drafted).

**Rejected (D-C3-1) because:**
- No production engine in the survey hard-rejects on these grounds; Trino, DataFusion, and BigQuery
  all use cost-based mitigation (broadcast caps, dynamic filtering, resource timeouts).
- Hard rejection requires reliable selectivity estimates at plan time. For sensor REST adapters with
  absent or wildly incorrect statistics, the planner cannot reliably distinguish a "selectve equi-join
  that will return 10 rows" from a "selective equi-join that will return 10M rows." False rejections
  are UX failures.
- The cost-based degrade model is strictly more expressive (it handles valid but hard-to-estimate
  queries) and equally safe (row-caps + resource abort bound the cost).

### Alternative B: Inner-Equi-Only Cross-Source Joins (Hard Ban on Outer/Non-Equi)

Restrict cross-source joins to inner equi-joins only, since DataFusion 50.x dynamic filter covers
only that shape.

**Rejected (D-C3-3) because:**
- Outer/non-equi joins are valid and useful security analytics patterns (e.g., LEFT JOIN for
  "assets not in the detection alert set" — a key investigative pattern).
- The cost exposure for outer/non-equi joins is bounded by the same row-cap + resource-abort stack.
  The weaker cost guarantee (no dynamic filter) is a known trade-off, consciously owned.
- Banning these join shapes is a significant PrismQL expressiveness constraint with limited safety
  benefit given the guardrail depth.

### Alternative C: Open-Expression Capability Descriptor (Trino ConnectorExpression Model)

Rather than the enumerated predicate-class vocabulary, allow connectors to declare arbitrary
expression trees (any predicate the connector can evaluate).

**Rejected (L-C3-2) because:**
- A TOML-based descriptor cannot express arbitrary expression trees; this would require a code
  connector that parses and executes AST nodes.
- Sensor REST adapters have a fixed, small vocabulary of filterable parameters. An open-expression
  model adds engineering cost with no practical benefit for the current (and near-future) connector
  set.
- The enumerated-class model is TOML-friendly, `#[non_exhaustive]`-compatible, and directly maps to
  DataFusion's `Exact/Inexact/Unsupported` enum. Revisit only if a future connector genuinely
  requires arbitrary-expression pushdown.

---

## Ripple Effects (Must Be Picked Up at Morph Time)

| Affected area | Ripple |
|---------------|--------|
| **§12.2 NFR-JOIN-GUARD** | Requirements #1/#2 "rejected at plan time" language should be amended to "cost-based degrade" per D-C3-1. The equality-key and time-bound requirements remain as guard triggers. |
| **§5.3 PRD/NFR addendum** | "planner rejects" language in the cross-source join cost guard NFR item should be amended to reflect degrade posture. The mandatory-time-bound NFR (inject + disclose) is unchanged. |
| **BC-2.16.002 §Postconditions** | New Canonical Structured Event Catalog rows for `query.pushdown.decision`, `query.injected_default_window`, `query.override_applied` (SAP-1 obligation, morph-time). |
| **Architecture — new subsystem** | Connector Capability-Descriptor subsystem entry in ARCH-INDEX.md (owner of the TOML descriptor schema, PrismQL pre-pass logic, and DataFusion TableProvider binding). |
| **ADR-TBD: Connector capability-descriptor model** | This proposed-ADR covers D-C3-1..4 + leans. Real ADR number allocated at morph (ADR-055+). |
| **ADR-TBD: PrismQL pushdown & cross-source join semantics** | Companion ADR to the descriptor model, covering the PrismQL pre-pass design, DataFusion interface contract, and guardrail stack implementation. |
| **Cargo.toml** | `datafusion >= "50.0.0"` minimum version constraint. Exact pin resolved at morph. |
| **Descriptor structs** | All `#[non_exhaustive]` + `#[cfg(test)]` coverage per CLAUDE.md discipline. New compile-fail gate entry for any pub TOML-deserialized descriptor types (non_exhaustive-violation crate). |
| **E-CONNECTOR-CAPABILITY-DESCRIPTOR-001** | Proposed epic: implement the declarative TOML descriptor + DataFusion TableProvider binding + PrismQL pre-pass guardrail stack. Blocks all day-2 connector expansion. |
