---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
provenance: "side-analysis discussion input; out-of-band, separate from the live VSDD factory pipeline. Does not modify vision/specs/STATE/SESSION-HANDOFF/ADR/BC/story or any prior research file."
topic: "DataFusion 50.x runtime resource-bounding + cost-mitigation mechanics behind the C3 cost-based degrade posture (degrade-not-reject; allow outer/non-equi cross-source joins)"
feeds: "day-2 vision SIDE-ANALYSIS hardening pass; complements capability-descriptor-pushdown-2026-06-26.md (C3) which owns the descriptor model, predicate-class vocabulary, and Exact/Inexact/Unsupported pushdown SPI"
engine: "DataFusion 50.x (Sept 2025 release line) + Chumsky parser; ephemeral/federated query thesis over remote sensor REST APIs with no source statistics"
relationship_to_c3: "C3 verified Trino's cost guards in depth and only lightly covered DataFusion's. This pass fills the DataFusion gap: memory pool / spill / cancellation, statistics-absent join distribution, outer/non-equi join cost + dynamic-filter coverage, row-cap mechanics, federated prior art, and agent-harness disclosure."
---

# DataFusion 50.x Cost-Degrade Mechanics — the Runtime Machinery Behind C3's Degrade-Not-Reject Posture

**Side-analysis / discussion input — NOT a spec or vision change.** In C3 the human chose a **cost-based degrade** posture for cross-source joins (NOT hard-reject) and chose to **allow outer / non-equi cross-source joins** (central-only, no dynamic-filter optimization). Those two choices move the load-bearing guardrail off a plan-time rejection and onto **DataFusion's own runtime resource-bounding and cost-mitigation machinery**. C3 verified Trino's guards in depth but covered DataFusion's only lightly. This document fills exactly that gap. `do_not_execute: true`. *Leans are discussion input only.*

> **C3 boundary respected.** This file does NOT re-cover the descriptor model, predicate-class vocabulary, or the `Exact/Inexact/Unsupported` pushdown SPI — C3 (`capability-descriptor-pushdown-2026-06-26.md`) owns those and was read first, not edited. Where this analysis touches limit-pushdown or the join guard, it builds on C3's findings rather than restating them.

> **Version-verification honesty.** All DataFusion API/config claims below were cross-checked against Context7's live `/apache/datafusion` snapshot (which surfaces `main`-branch `docs/source/user-guide/configs.md`, the `dev/changelog/50.0.0.md`, and docs.rs) on 2026-06-27. Items confirmed by Context7 are tagged `[ctx7-verified]`. Items sourced only from Perplexity deep-research web synthesis are tagged `[web]`. Items I could NOT confirm for 50.x specifically are tagged `[INCONCLUSIVE]`. Model reasoning is tagged `[model-knowledge]`.

---

## Executive Summary (~14 lines)

1. **DataFusion has a real per-query memory limit + spill subsystem**, configured via `RuntimeEnvBuilder::with_memory_limit(bytes, fraction)` / `SET datafusion.runtime.memory_limit = '2G'`, backed by a pluggable `MemoryPool`. `[ctx7-verified][web]`
2. **Spill is operator-selective.** `SortExec`, `AggregateExec` (grouped), and `SortMergeJoinExec` spill to disk under pressure; **`HashJoinExec` does NOT spill** — it returns a memory-reservation / `ResourcesExhausted` error when its in-memory hash table exceeds the limit. **`NestedLoopJoinExec` and `CrossJoinExec` spill behavior is undocumented/unconfirmed** — treat as non-spilling (error-on-OOM). `[web]` (SedonaDB memory docs reflect DataFusion behavior; partly `[INCONCLUSIVE]` for the nested-loop/cross operators).
3. **There is NO built-in query-level wall-clock timeout in DataFusion.** The embedder MUST enforce it — `tokio::time::timeout` around the stream consumption + dropping the `SendableRecordBatchStream`. `[web]`
4. **Cancellation is cooperative.** DataFusion 49.0.0 added Tokio-task-budget cooperative scheduling + an `EnsureCooperative` optimizer rule that wraps non-cooperative operators in `CooperativeExec`; built-in sources participate automatically. A custom `TableProvider` mid-network-fetch is cancelled when its `async` stream is dropped, **but only at `await` points** — a blocking/non-yielding custom operator can still refuse to stop. `[web]`
5. **DataFusion has NO documented equivalent of Trino's `join-max-broadcast-table-size`.** There is no config knob that prevents `JoinSelection` from collecting an unknown-size build side (`PartitionMode::CollectLeft`). The embedder must enforce no-broadcast-of-unknown-size itself. `[web]` (strong negative finding, multiple sources).
6. **The relevant knobs that DO exist** are `datafusion.optimizer.prefer_hash_join` (set `false` to favor sort-merge — which CAN spill), `datafusion.optimizer.hash_join_single_partition_threshold` (default `1048576` bytes), and `hash_join_single_partition_threshold_rows` (default `131072` rows) — these gate single-partition/collect behavior by SIZE, so an *absent* statistic means the optimizer cannot conclude "small," which *should* default away from CollectLeft, but this is **inferred, not documented**. `[ctx7-verified]` (the keys + defaults) / `[INCONCLUSIVE]` (the absent-stats default behavior).
7. **Statistics absent ⇒ `Precision::Absent`.** `TableProvider::statistics()` docs say table stats are "not presently used in mainline DataFusion" for join reordering — so a REST source returning `Absent` largely just means the optimizer falls back to structural heuristics. Good for safety (no false "small" broadcast) but means cost-based join order won't help either. `[web]`
8. **`NestedLoopJoinExec` is the operator for non-equi + CROSS-condition joins; `CrossJoinExec` for predicate-less Cartesian.** Both buffer one side fully in memory (CrossJoin explicitly buffers the LEFT side). Confirmed they participate in OUTER semantics (a 50.x-era bug fix addressed spurious unmatched-left rows in `NestedLoopJoinExec`). `[web]`
9. **DEFINITIVE resolution of C3's `[INCONCLUSIVE]` flag:** DataFusion 50.0.0 dynamic-filter "Sideways Information Passing" is documented **only for INNER hash equi-joins** (min/max key-range pushdown into the probe-side scan via `DynamicFilterPhysicalExpr` as `Arc<dyn PhysicalExpr>`). Sources state dynamic filters **do not currently optimize OUTER joins** and there is **no documentation of dynamic filtering for NON-EQUI / NestedLoop / Cross joins.** The semantic reason: outer joins must preserve unmatched rows, so probe-side pruning is unsafe. `[web]` (multiple corroborating statements).
10. **Therefore C3's "allow outer/non-equi, no dynamic-filter optimization" is exactly correct as a *description* of DataFusion 50.x reality** — there is no optimization to lose because none exists for those shapes. The cost burden falls entirely on central buffered execution.
11. **Row-cap mechanics:** OUTPUT cap = `LIMIT` → `GlobalLimitExec` (+ push to scan where C3's descriptor declares `limit.supported`). INPUT cap (per-join-side) = inject a `LIMIT`/`scan(limit)` into EACH side's scan subtree BEFORE the join — DataFusion does not auto-cap join inputs, so this is an embedder rewrite. A counting/abort operator above the join is the belt-and-suspenders ceiling. `[web][model-knowledge]`
12. **Federated prior art:** Trino is the only surveyed engine with a documented, multi-dimensional guard stack (time/CPU/memory/scan-bytes/broadcast-cap/dynamic-filter-caps). Among DataFusion-based systems, **only InfluxDB 3 / IOx publishes a concrete guardrail story — and it is *query timeouts enforced at the system level*, not inside DataFusion.** Ballista, datafusion-federation, Comet, Sail, Spice.ai have **no published query-level guardrails** — they push the responsibility to the embedder. `[web]`
13. **Net for Prism:** the degrade-not-reject posture is *viable* but only if Prism builds the guard stack the embedder is expected to build: embedder-enforced wall-clock timeout, a bounded `MemoryPool` (prefer `FairSpillPool`) + `DiskManager` spill, forced-partitioned / no-CollectLeft join rewriting, mandatory input + output row caps, and the mandatory time-window injection from C3 §Topic-4. DataFusion gives you the spill + cooperative-cancellation primitives; it does NOT give you the policy guards.
14. **Honest cost:** several load-bearing behaviors (CollectLeft default under absent stats; NestedLoop/Cross spill) are `[INCONCLUSIVE]` in public docs and must be pinned by reading `join_selection.rs` + empirical `EXPLAIN` inspection against the pinned DataFusion 50.x in Prism's `Cargo.lock`. Do not ship the degrade posture without that verification.

---

## Topic 1 — Runtime Resource-Bounded Execution (the mechanism behind "abort-after-consumption")

### 1.1 MemoryPool + per-query memory limit `[ctx7-verified][web]`

DataFusion centralizes memory bounding in the `MemoryPool` trait (`datafusion::execution::memory_pool`), wired into `RuntimeEnv` (`pub memory_pool: Arc<dyn MemoryPool>`). Concrete pools:

| Pool | Semantics |
|------|-----------|
| `UnboundedMemoryPool` | **Default.** No limit — allocates until the OS/container kills the process. `[web]` |
| `GreedyMemoryPool` | Fixed budget, first-come-first-served. "Works well for queries that do not need to spill or have a single spillable operator." `[web]` |
| `FairSpillPool` | Fixed budget, distributes fairly among **spillable** consumers, reserves a fraction (`unspillable_reserve_ratio`, default ~0.2) for **unspillable** consumers (e.g. hash join). "Stable under memory pressure, significantly less likely to cause reservation failures." `[web]` |
| `TrackConsumersPool` | Decorator over another pool; records top consumers to enrich `ResourcesExhausted` error messages (GH #20386). `[web]` |

Configuration paths `[ctx7-verified]`:
```rust
let runtime = RuntimeEnvBuilder::new()
    .with_memory_limit(5_000_000, 1.0) // bytes, utilization fraction
    .build_arc()?;
let ctx = SessionContext::new_with_config_rt(SessionConfig::new(), runtime);
```
or SQL: `SET datafusion.runtime.memory_limit = '2G';` `[ctx7-verified]`

Per-query vs per-session: a `MemoryPool` is attached to a `RuntimeEnv`, which is attached to a `SessionContext`. The cleanest per-query isolation is **a fresh `RuntimeEnv` (and pool) per query** — which fits Prism's *ephemeral* engine thesis precisely (one query = one short-lived context with its own bounded pool). `[web][model-knowledge]`

Memory accounting is via `MemoryConsumer` (one per operator-partition) → `MemoryReservation` (one per internal structure, e.g. a sort buffer or hash table) → `MemoryPool`. Operators that allocate proportional-to-rows must reserve before allocating; on denial they spill (if capable) or error. `[web]`

### 1.2 Disk spill — which operators spill `[web]` (partly `[INCONCLUSIVE]`)

| Operator | Spills? | On limit-exceeded |
|----------|---------|-------------------|
| `SortExec` / external sort | **Yes** — sorted runs to disk, multi-level merge. 50.0.0 specifically hardened this ("almost any sort that previously OOM'd now spills"). | spill |
| `AggregateExec` (grouped hash agg) | **Yes** — spills intermediate aggregation state to sorted spill files. | spill |
| `SortMergeJoinExec` | **Yes** — spills buffered batches; "can join arbitrarily large inputs where one or both don't fit in memory." | spill |
| `HashJoinExec` | **NO** — "does not support spilling yet. The query will fail with a memory reservation error if the hash table exceeds the memory limit." | **error** |
| `NestedLoopJoinExec` | **UNCONFIRMED** — not listed among spilling operators; build-probe buffers one side. | likely **error** `[INCONCLUSIVE]` |
| `CrossJoinExec` | **UNCONFIRMED** — buffers the LEFT side; not listed as spilling. | likely **error** `[INCONCLUSIVE]` |

DiskManager config `[web]`: `with_disk_manager_os()` / `with_disk_manager_specified(paths)` / `with_disk_manager_disabled()` / `with_temp_file_path(path)`; `SET datafusion.runtime.max_temp_directory_size = '500G'` (default 100 GB); `SET datafusion.execution.spill_compression = 'lz4_frame'` (default uncompressed). Heavy spill ⇒ raise `nofile` ulimit. `[web]`

**Load-bearing consequence for Prism:** because `HashJoinExec` (and probably NestedLoop/Cross) does NOT spill, a cross-source join whose buffered build side exceeds the pool limit produces a **`ResourcesExhausted` error mid-execution** — this IS the "abort-after-consumption" mechanism. The degrade posture relies on (a) a bounded pool so the abort is graceful rather than an OS OOM-kill, and (b) input row-caps (Topic 4) so the build side is bounded *before* it can blow the pool.

### 1.3 Error variant `[web]`

The error is `DataFusionError::ResourcesExhausted` (historically present; GH #20386 confirms ongoing use and enriches it with pool/consumer detail via `TrackConsumersPool`). Note: a `15.0.0` changelog entry confirms `ResourceExhausted` returns specifically from `GroupedHashAggregateStreamV2` `[ctx7-verified]`. Map this to a Prism `E-QUERY-NNN` taxonomy code at the engine boundary; surface it to the agent harness as a structured "resource-cap fired" signal (Topic 5).

### 1.4 Wall-clock timeout + cancellation — EMBEDDER's job `[web]`

- **No built-in query timeout.** No `datafusion.*` config key for time-based termination exists. Confirmed by absence across configs.md, RuntimeEnv docs, and the relevant GH issues. The embedder enforces it. `[web]`
- **Mechanism:** wrap the consumption of the `SendableRecordBatchStream` in `tokio::time::timeout(...)`; on elapse, **drop the stream** — dropping cancels the underlying Tokio tasks. `[web]`
- **Cooperative scheduling (49.0.0+):** Tokio maintains a per-task budget; resources return `Pending` when it hits zero, forcing a yield. DataFusion integrated this into all built-in sources, plus added an `ExecutionPlan` "cooperative" property and an `EnsureCooperative` optimizer rule that inserts `CooperativeExec` wrappers around operators that don't natively yield. `[web]`
- **`ExecutionPlan` contract:** "To enable timely cancellation, the Stream returned must not block the CPU indefinitely and must yield back to the tokio runtime regularly." `[web]`
- **Custom `TableProvider` mid-network-fetch:** if the adapter's `scan()` returns a stream that `await`s a remote HTTP response, dropping the consuming stream propagates cancellation **at the next `await` point** — i.e. the in-flight `reqwest` future is dropped and the connection aborted. This works precisely because the fetch is `async`/awaiting. **Risk:** a custom operator doing a long synchronous CPU loop (or a blocking I/O call) between `await`s will NOT yield and can refuse to cancel until it finishes that chunk; `EnsureCooperative`/`CooperativeExec` mitigates source operators but not arbitrary blocking inside a custom operator. `[web][model-knowledge]`

**Lean (Prism):** enforce a per-query wall-clock budget in Prism's engine wrapper (`tokio::time::timeout` + stream drop). Ensure the spec-driven adapter's `scan()` is fully `async` with `await` points around every network call and `reqwest` uses the mandated 30s client timeout (CLAUDE.md HTTP-client rule) as a second, per-request bound. The query-level timeout and the per-request timeout are complementary, not redundant.

---

## Topic 2 — Cost Estimation Without Source Statistics

### 2.1 What the optimizer does with `Precision::Absent` `[web]`

- `Statistics` carries `num_rows` / `total_byte_size` / per-column stats, each wrapped in `Precision::{Exact, Inexact, Absent}`. A REST source with no stats returns `Absent`. `[web]`
- **Crucial doc statement:** `TableProvider::statistics()` returns stats that are "not presently used in mainline DataFusion" for join reordering — they "allow implementation-specific behavior for downstream repositories, in conjunction with specialized optimizer rules." So absent table stats from a REST source largely just means **mainline join reordering doesn't fire** and the optimizer falls back to structural/syntactic heuristics. `[web]`
- `EnforceDistribution` operates on `Partitioning` metadata, NOT on row/byte stats — so it behaves identically whether stats are absent or present (it just inserts `RepartitionExec` to satisfy hash-partitioning requirements). Keep it enabled. `[web]`
- Filter/projection pushdown are structural — unaffected by absent stats. `[web]`

### 2.2 Join distribution selection + the broadcast guard gap `[web][ctx7-verified]`

`JoinSelection` chooses between `PartitionMode::Partitioned` (hash-redistribute both sides) and `PartitionMode::CollectLeft` (collect build side fully into memory — the broadcast analog). The decision is **statistics-driven**, gated by:

- `datafusion.optimizer.hash_join_single_partition_threshold` = **1048576** (bytes) `[ctx7-verified]`
- `datafusion.optimizer.hash_join_single_partition_threshold_rows` = **131072** (rows) `[ctx7-verified]`
- `datafusion.optimizer.prefer_hash_join` = **true** (set `false` → favors `SortMergeJoinExec`, which CAN spill) `[ctx7-verified]`

**THE LOAD-BEARING NEGATIVE FINDING:** DataFusion has **no documented equivalent of Trino's `join-max-broadcast-table-size`** and **no config flag to disable `CollectLeft` or force `Partitioned` globally.** Multiple sources state this explicitly. There is therefore **no built-in guard that prevents collecting an unknown-size build side into memory.** `[web]`

What *should* happen under absent stats: the optimizer cannot compare an `Absent` size against the threshold, so it *should* not classify the side as "small," *should* default to `Partitioned`, and `CrossJoinExec` still buffers left-side regardless. **But this absent-stats default is INFERRED, not documented** `[INCONCLUSIVE]` — it must be confirmed by reading `datafusion/physical-optimizer/src/join_selection.rs` at Prism's pinned version and by `EXPLAIN`-inspecting an actual cross-source join plan.

### 2.3 How the embedder forces partitioned / prevents CollectLeft `[web][model-knowledge]`

Since no config knob exists, the embedder options are:
1. **Physical-plan rewrite pass:** walk the optimized `ExecutionPlan`, downcast each `HashJoinExec`, and if `PartitionMode::CollectLeft`, reconstruct with `PartitionMode::Partitioned` (using `swap_inputs()` as needed). Reject/guard `CrossJoinExec` over two REST sources. `[web]`
2. **Set `prefer_hash_join = false`** so large equi-joins use spillable `SortMergeJoinExec` instead of non-spilling `HashJoinExec`. Trades latency for memory-safety — a defensible default for an ephemeral federated engine. `[ctx7-verified][model-knowledge]`
3. **Set the single-partition thresholds very low** to discourage CollectLeft — but this does NOT cover `CrossJoinExec` and is not a hard guarantee. `[web]`

**Lean (Prism):** combine (1) a post-optimization physical-plan rewrite that forbids `CollectLeft` for cross-source joins, with (2) `prefer_hash_join = false` as a memory-safety default. Treat #1 as the load-bearing guard; #2 as defense-in-depth.

---

## Topic 3 — Outer / Non-Equi Cross-Source Join Cost (resolves C3's `[INCONCLUSIVE]`)

### 3.1 Operators + buffering `[web]`

- **`NestedLoopJoinExec`** — the operator for non-equi predicates (`<`, `>`, `BETWEEN`, complex OR) and inner joins whose conditions can't map to hash/merge. Build-probe; buffers one side. **Which side is buffered is NOT documented** `[INCONCLUSIVE]`; by analogy with CrossJoin it is plausibly the left, but unconfirmed. Confirmed to participate in OUTER semantics — a 50.x-era bug fix (GH #22808) split probe-completion into a `ProbeEnd` state to stop emitting spurious unmatched-left rows in LEFT OUTER. **Spill: unconfirmed/unlisted ⇒ treat as non-spilling (error-on-OOM).** `[web][INCONCLUSIVE]`
- **`CrossJoinExec`** — predicate-less Cartesian. **Explicitly buffers the entire LEFT input in memory**, streams the right. `swap_inputs()` lets the planner pick the smaller side as build. Output cardinality is multiplicative ⇒ even a fitting build side can produce an enormous result that stresses downstream operators. **Spill: not listed ⇒ treat as non-spilling.** `[web]`
- **Outer `HashJoinExec` (LEFT/RIGHT/FULL):** same as inner — build-side hash table is in-memory and **does NOT spill**; large build ⇒ `ResourcesExhausted`. The outer-ness doesn't change the memory model. `SortMergeJoinExec` is the spillable alternative for large equi-(outer-)joins. `[web]`

### 3.2 Dynamic-filter coverage — DEFINITIVE `[web]`

C3 flagged this `[INCONCLUSIVE]`. **Resolved:**

- DataFusion 50.0.0 dynamic filtering / "Sideways Information Passing" pushes the build side's min/max join-key range into the probe-side scan as a `DynamicFilterPhysicalExpr` (`Arc<dyn PhysicalExpr>`), via `ExecutionPlan::gather_filters_for_pushdown` / `handle_filter_result`. 50.0.0 changelog confirms specific `HashJoinExec` dynamic-filter fixes (PR #17201 deterministic creation #17280). `[ctx7-verified][web]`
- **It is documented ONLY for INNER hash equi-joins.** Sources state plainly: dynamic filters **"do not currently optimize outer joins,"** are **"not available for outer joins,"** and there is **no documentation of dynamic filtering for NON-EQUI / NestedLoop / Cross joins.** `[web]`
- **Semantic reason:** outer joins must preserve unmatched rows on the outer side; probe-side row pruning would drop rows that must appear (with NULLs) in the output. So the optimization is *unsafe* for outer joins, not merely unimplemented. `[web][model-knowledge]`

**This validates C3's posture exactly.** C3 chose "allow outer/non-equi cross-source joins, no dynamic-filter optimization." There is no optimization being given up — DataFusion 50.x simply does not (and semantically cannot, for outer) apply sideways-information-passing to those shapes. The cost falls entirely on central buffered execution.

### 3.3 Concrete mitigations when dynamic filtering does NOT apply `[web][model-knowledge]`

For an outer / non-equi cross-source join (no dynamic filter available):
1. **Row-cap pushdown into EACH side's scan** (Topic 4) — bound the buffered side(s) before the join builds. This is the single most important mitigation: it bounds `NestedLoopJoinExec` / `CrossJoinExec` / outer-`HashJoinExec` memory at the *input*.
2. **Mandatory time-window injection** (C3 §Topic-4) on each side's scan — the other major bound on input size.
3. **Prefer the spillable path:** `prefer_hash_join = false` so a large outer EQUI-join uses `SortMergeJoinExec` (spills) rather than non-spilling `HashJoinExec`. (Does not help NON-equi, which is stuck on NestedLoop.)
4. **Bounded `MemoryPool` + `DiskManager`** so when a non-spilling operator does exceed the limit, it aborts gracefully with `ResourcesExhausted` rather than OS-OOM-killing the process.
5. **Partial results + coverage disclosure** (Topic 5) — if a cap/abort fires mid-stream, surface what was produced plus the fact of truncation.
6. **For predicate-less CROSS over two REST sources specifically:** this is the worst case (multiplicative output, non-spilling, no dynamic filter). Lean: still *allow* it per the human's C3 choice, but ONLY with a hard input row-cap on both sides AND an output `GlobalLimitExec` cap — i.e. a bounded Cartesian, never an unbounded one.

---

## Topic 4 — Row-Cap + LIMIT Enforcement Mechanics `[web][model-knowledge]`

### 4.1 OUTPUT cap

- `LIMIT n` in PrismQL → `GlobalLimitExec` (central, exact). `[web]`
- Push to the source scan via `scan(projection, filters, limit)` where C3's descriptor declares `limit.supported = exact`; otherwise enforce the residual centrally at `GlobalLimitExec`. (C3 §Topic-4 owns the limit-pushdown declaration semantics.)

### 4.2 INPUT cap (per join side) — the load-bearing one

DataFusion does **not** automatically cap the inputs to a join. To bound the buffered side(s) of a cross-source join, the embedder must **inject a `LIMIT` / `scan(limit)` into EACH side's scan subtree** before the join node, as a PrismQL plan-rewrite. This is what makes the buffered-side memory bounded and the non-spilling-operator abort avoidable. `[web][model-knowledge]`

### 4.3 Hard ceiling (belt-and-suspenders)

A custom counting `ExecutionPlan` wrapper that aborts (returns `ResourcesExhausted` or an `E-QUERY-NNN`) past an absolute row ceiling, inserted above the join, guarantees a hard cap even if the per-side limit injection is bypassed. `[model-knowledge]` This mirrors Trino's `query.max-scan-physical-bytes` intent but as an embedder operator since DataFusion lacks the config.

**Lean (Prism):** three layers — (a) per-side scan `LIMIT` injection (bounds the build side), (b) `GlobalLimitExec` output cap (bounds the result), (c) optional counting-abort operator as the absolute ceiling. Default + max caps as in C3 §Topic-4.

---

## Topic 5 — Prior Art: Cost-Based Degrade in Federated Engines `[web]`

### 5.1 Trino (the reference guard stack) `[web]`

| Property | Default | Effect class |
|----------|---------|--------------|
| `query.max-run-time` | `100d` | hard abort (wall-clock incl. queue) |
| `query.max-execution-time` | `100d` | hard abort (active exec only) |
| `query.max-cpu-time` | `1_000_000_000d` | hard abort (cluster CPU) |
| `query.max-memory` | `20GB` | hard abort (cluster user mem) |
| `query.max-memory-per-node` | 30% heap | hard abort (per-node) |
| `query.max-total-memory` | `2 × query.max-memory` | hard abort (incl. revocable) |
| `query.max-scan-physical-bytes` | (unset) | hard abort — **but only if the connector reports physical bytes** (GH issue shows it silently no-ops otherwise) |
| `join-max-broadcast-table-size` | `100MB` | **cost-based DEGRADE** — forces partitioned join instead of broadcast; does NOT abort |
| `dynamic-filtering.{max-distinct-values,max-size,range-row-limit}-per-driver` | (varies) | **cost-based DEGRADE** — truncates/disables oversized dynamic filters |
| `query.low-memory-killer.policy` | `total-reservation-on-blocked-nodes` | hard abort (query-level); `least-waste` task-level under `retry-policy=TASK` is a degrade |

Lesson: Trino's *degrade* guards are exactly `join-max-broadcast-table-size` (plan-shaping) and dynamic-filter size caps. The *abort* guards are time/CPU/memory/scan. Prism's degrade posture should mirror the plan-shaping (no-CollectLeft) + abort (memory pool + embedder timeout) split.

### 5.2 DataFusion-based systems `[web]`

- **InfluxDB 3 / IOx — the only one with a published guardrail story.** It is **query timeouts enforced at the system level (not inside DataFusion)**, tiered by workload: ~10s UI/dashboard, ~60s generic, ~2m mixed, ~5m analytical/batch; plus retry with exponential backoff + jitter + circuit breakers + deadline propagation. This confirms the embedder-enforces-timeout pattern from Topic 1.4. `[web]`
- **Ballista** (distributed DataFusion) — scheduler breaks queries into stages; **no documented per-query timeout/memory guardrail properties.** `[web][INCONCLUSIVE]`
- **datafusion-federation** — SQL-unparsing logical-plan pushdown to remote engines; **no published guardrails** — defers to host + remote engines. `[web]`
- **Comet / Sail** — Spark-ecosystem DataFusion engines; **no published DataFusion-level guardrails** — inherit Spark's (`spark.sql.autoBroadcastJoinThreshold`, etc.). `[web][INCONCLUSIVE]`
- **Spice.ai** — uses Ballista; guardrails in its orchestration layer, **not published.** `[web][INCONCLUSIVE]`

**Synthesis:** DataFusion is an embeddable library; every production deployment that bounds runaway queries does so in the *embedder*, not via DataFusion config. Prism is the embedder. The realistic guardrail stack for an ephemeral engine over remote APIs with no stats is the union below.

---

## Topic 6 — Agent-Harness Disclosure on Degrade/Abort/Cap `[model-knowledge — flagged]`

When a query is degraded, aborted, capped, or had a default window injected, the consuming LLM agent MUST be told, or it will reason over a silently-truncated result (a prompt-injection-adjacent correctness hazard per the agent-harness design memory). Tie to Prism's existing partial-result + coverage primitives:

- **Coverage banner (§3.6 / BC-2.01.010 partial-result handling):** when a per-side cap or output cap fired, mark the result as `partial` and state which source(s)/branch(es) were truncated and at what row count. `[model-knowledge]`
- **Injected-window disclosure (C3 §Topic-4):** surface the *effective* time window the query actually ran with, so the agent knows it saw "last 24h," not "all time."
- **Cap/abort signal:** if `ResourcesExhausted` fired mid-stream and partial rows were returned, emit a structured field (e.g. `degrade_reason = "memory_cap" | "row_cap" | "timeout"`) + the cap value. Map the DataFusion `ResourcesExhausted` to an `E-QUERY-NNN` code.
- **Structured event:** a `query.degrade.fired` event_type would require a Canonical Structured Event Catalog row in BC-2.16.002 per CLAUDE.md SAP-1 — flagged as a **downstream spec dependency, NOT actioned here.**

**Lean:** every degrade/cap/abort/window-injection is part of the *result envelope*, never silent. The agent reasons over coverage-aware results or it reasons wrong.

---

## Consolidated Findings → Recommended Prism Guardrail Stack

Ordered, concrete mechanisms to make **degrade-not-reject + allow-outer/non-equi** safe (discussion input):

1. **Embedder-enforced per-query wall-clock timeout** — `tokio::time::timeout` around stream consumption + drop the `SendableRecordBatchStream`. DataFusion has no built-in timeout; this is mandatory. (Topic 1.4)
2. **Bounded per-query `MemoryPool`** — use `FairSpillPool` with a configured byte limit via `RuntimeEnvBuilder::with_memory_limit`, fresh per ephemeral query. Turns OS-OOM-kills into graceful `ResourcesExhausted`. (Topic 1.1)
3. **`DiskManager` spill enabled** with a bounded `max_temp_directory_size` — lets `SortExec`/`AggregateExec`/`SortMergeJoinExec` spill instead of erroring. (Topic 1.2)
4. **`prefer_hash_join = false`** as the memory-safety default — routes large equi-joins to the spillable `SortMergeJoinExec` rather than the non-spilling `HashJoinExec`. (Topic 2.2)
5. **Post-optimization physical-plan rewrite that forbids `PartitionMode::CollectLeft` for cross-source joins** (force `Partitioned`) and guards predicate-less `CrossJoinExec` over two REST sources. DataFusion has NO config for this — it is the load-bearing embedder guard. (Topic 2.3)
6. **Mandatory per-join-side input row-cap** — inject `LIMIT`/`scan(limit)` into EACH side's scan subtree before the join, bounding the buffered build side. (Topic 4.2)
7. **Mandatory output row-cap** — `GlobalLimitExec` + push to scan where C3's descriptor declares limit support. (Topic 4.1)
8. **Mandatory time-window injection** on every source scan (C3 §Topic-4) — the second major input bound, reinforcing #6.
9. **Optional absolute counting-abort operator** above the join as a hard ceiling (mirrors Trino `query.max-scan-physical-bytes` intent). (Topic 4.3)
10. **Fully-`async` adapter `scan()`** with `await` points around every network call + the 30s `reqwest` client timeout (CLAUDE.md rule) — so cooperative cancellation can actually reach a mid-fetch scan. (Topic 1.4)
11. **Coverage-aware result envelope** — disclose every degrade/cap/abort/injected-window to the agent harness; map `ResourcesExhausted` → `E-QUERY-NNN` + `degrade_reason`. (Topic 6)

This is the "embedder builds the guards DataFusion doesn't provide" stack that InfluxDB 3 / IOx validates and Trino's guard taxonomy informs.

---

## Open Design Questions

1. **CollectLeft default under absent stats** — `[INCONCLUSIVE]`. Must read `join_selection.rs` at Prism's pinned DataFusion 50.x + `EXPLAIN`-verify that a stats-less cross-source join does NOT pick `CollectLeft`. If it can, mechanism #5 is mandatory (not optional).
2. **NestedLoopJoinExec / CrossJoinExec spill** — `[INCONCLUSIVE]`. Confirm empirically whether they error or spill under a tight pool. If they error (likely), input row-caps (#6) become the only thing standing between an outer/non-equi cross-source join and an abort.
3. **Which side does NestedLoopJoinExec buffer?** — `[INCONCLUSIVE]`. Affects whether the row-cap must bound both sides or just the build side. Default to capping BOTH sides until confirmed.
4. **Physical-plan rewrite vs custom optimizer rule** — implement the no-CollectLeft guard as a post-optimization tree rewrite, or as a registered `PhysicalOptimizerRule`? Lean: a registered rule (composes with DataFusion's optimizer pipeline cleanly).
5. **Per-query RuntimeEnv cost** — is constructing a fresh `RuntimeEnv` + `FairSpillPool` per ephemeral query cheap enough at Prism's QPS? Benchmark; if not, pool-of-pools.
6. **Default memory-pool size + timeout values** — tie to the 512MB process / 200MB per-query memory budget (project memory). The per-query `MemoryPool` limit should be ≤ the 200MB per-query budget.
7. **Catalog rows for degrade events** — `query.degrade.fired` / `query.window.injected` likely need BC-2.16.002 Canonical Structured Event Catalog rows (SAP-1). Downstream spec dependency.

---

## Honest Costs & Caveats

- **DataFusion gives Prism the *primitives* (bounded pool, spill, cooperative cancellation) but NONE of the *policy guards*.** Every guard in the recommended stack except #2/#3 is embedder code Prism must write and maintain. This is real, non-trivial work concentrated in the engine-wrapper + physical-plan-rewrite layer.
- **Three load-bearing behaviors are `[INCONCLUSIVE]` in public docs** (CollectLeft default under absent stats; NestedLoop/Cross spill; NestedLoop buffered side). The degrade posture must NOT ship until these are pinned by reading `join_selection.rs` at the locked version + empirical `EXPLAIN`/OOM testing. Public docs lag the code here.
- **`HashJoinExec` not spilling is the sharpest edge** — an outer EQUI cross-source join over an unexpectedly large source aborts with `ResourcesExhausted`. `prefer_hash_join = false` mitigates (routes to spillable SMJ) but trades latency. NON-equi outer joins have no spillable fallback (NestedLoop only) — input row-caps are the only defense.
- **Predicate-less CROSS over two REST sources** is the genuine worst case: non-spilling, multiplicative output, no dynamic filter, possibly large buffered left. The human's C3 choice to *allow* it is defensible ONLY with mandatory bounded inputs + bounded output. An *unbounded* such join WILL abort (best case) or OOM the process (if the pool is unbounded).
- **Dynamic-filter resolution is firm** but rests on the DataFusion blog + 50.0.0 changelog + docs.rs synthesis — the "do not optimize outer joins" claim is stated by the sources, not by an explicit DataFusion spec sentence. Treat as high-confidence-but-secondary-sourced.
- **C3's `matured-vision-day2-requirements.md` was not re-located** in this pass (same as C3's caveat); §-anchors (§3.6, §5.3, §12.2) are inherited from C3's paraphrase, not a direct read.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 4 | All `reasoning_effort=high`. (1) DataFusion 50.x MemoryPool variants / spill operators / `ResourcesExhausted` / cancellation + cooperative scheduling + custom-TableProvider mid-fetch cancellation (Topic 1). (2) Cost estimation under absent statistics: `Precision::Absent`, `JoinSelection`, `PartitionMode::CollectLeft` vs Partitioned, the missing `join-max-broadcast-table-size` analog, forcing partitioned (Topic 2). (3) Outer/non-equi/CROSS join operators (NestedLoopJoinExec/CrossJoinExec/outer HashJoinExec) memory+spill, and the definitive dynamic-filter-on-outer/non-equi coverage verdict (Topic 3, resolving C3's INCONCLUSIVE). (4) Federated-engine cost-degrade prior art: Trino full guard set + DataFusion-based systems (Ballista, datafusion-federation, InfluxDB 3/IOx, Comet, Sail, Spice.ai) (Topic 5). |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| **Context7** | 3 | `/apache/datafusion` — load-bearing 50.x version verification: (resolve) library ID; (query 1) MemoryPool/FairSpillPool/RuntimeEnvBuilder/memory_limit/DiskManager/ResourcesExhausted — confirmed `with_memory_limit(bytes, fraction)`, `SET datafusion.runtime.memory_limit`, FairSpillPool partition-division semantics, `15.0.0` ResourceExhausted-from-GroupedHashAggregateStreamV2; (query 2) PartitionMode/CollectLeft/hash_join_single_partition_threshold/prefer_hash_join/dynamic-filter — confirmed `prefer_hash_join` default true + `SET ...=false`, `hash_join_single_partition_threshold=1048576`, `hash_join_single_partition_threshold_rows=131072`, `hash_join_inlist_pushdown_max_size=131072`, and 50.0.0 changelog HashJoinExec dynamic-filter fixes (PR #17201, #17280). |
| Tavily (all) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Read | 5 | C3 file (`capability-descriptor-pushdown-2026-06-26.md`, read first, NOT edited) + 4 persisted perplexity_research outputs (each exceeded inline token cap; read from tool-result files). |
| Grep | 2 | Extract cancellation/TableProvider tail facts + the explicit dynamic-filter-on-outer-joins verdict from the long single-line persisted research files. |
| Training data | ~4 areas | OCSF/agent-harness disclosure UX (Topic 6); per-query RuntimeEnv-as-ephemeral-isolation reasoning; counting-abort-operator pattern; outer-join-pruning-unsafe semantic rationale. All flagged inline as `[model-knowledge]`. |

**Total MCP tool calls:** 7 (4 × `perplexity_research` high-effort + 3 × Context7 incl. resolve).
**Training data reliance:** low — every load-bearing DataFusion behavior (memory pool, spill matrix, no-built-in-timeout, cooperative cancellation, absent-stats join distribution, the missing broadcast guard, dynamic-filter inner-only coverage) is web-sourced via deep research, and the critical config keys + defaults + 50.0.0 changelog entries are Context7-verified against the live `/apache/datafusion` snapshot. Only the agent-harness disclosure UX and a few architectural inferences rest on model knowledge, each flagged. Three behaviors are honestly flagged `[INCONCLUSIVE]` (CollectLeft default under absent stats; NestedLoop/Cross spill; NestedLoop buffered side) rather than guessed.

### Citation key (sources from MCP web findings)
- **[DF-mempool]** docs.rs `datafusion::execution::memory_pool` — `MemoryPool` trait, `UnboundedMemoryPool`/`GreedyMemoryPool`/`FairSpillPool`/`TrackConsumersPool`, `MemoryConsumer`/`MemoryReservation`.
- **[DF-runtime]** docs.rs `datafusion::execution::runtime_env::RuntimeEnv` + `RuntimeEnvBuilder` (`with_memory_limit`, `with_disk_manager_*`, `with_temp_file_path`).
- **[DF-configs]** datafusion.apache.org `user-guide/configs.md` — `datafusion.runtime.memory_limit`, `max_temp_directory_size`, `spill_compression`, `optimizer.prefer_hash_join`, `optimizer.hash_join_single_partition_threshold(_rows)`, `hash_join_inlist_pushdown_max_size`. (Context7-verified.)
- **[DF-50]** datafusion.apache.org + `dev/changelog/50.0.0.md` — multi-level merge sort spill hardening; dynamic-filter HashJoinExec fixes (PR #17201, #17280); metadata caching. (Context7-verified.)
- **[DF-dynfilter]** datafusion.apache.org blog "Dynamic Filters: Passing Information Between Operators … 25x Faster" — sideways information passing, `DynamicFilterPhysicalExpr`, inner-hash-join min/max key pushdown; TopK example. States inner-equi-only.
- **[DF-execplan]** docs.rs `ExecutionPlan` — cooperative-yield cancellation note; `gather_filters_for_pushdown` / `handle_filter_result` / `reset_plan_state`.
- **[DF-cancel-blog]** datafusion.apache.org "Using Rust async for Query Execution and Cancelling Long Running Queries" — Tokio task-budget, `EnsureCooperative` rule, `CooperativeExec`, 49.0.0 integration.
- **[DF-joins]** docs.rs `datafusion::physical_plan::joins` — `HashJoinExec`, `SortMergeJoinExec`, `NestedLoopJoinExec`, `CrossJoinExec` (`left()`, `swap_inputs()`), `PartitionMode`.
- **[DF-stats]** docs.rs `TableProvider::statistics()` ("not presently used in mainline"), `Statistics`/`Precision` (Exact/Inexact/Absent), `StatisticsRegistry`; GH #20388 (`collect_new_statistics`), arrow-rs #5037.
- **[DF-joinsel]** apache/datafusion `physical-optimizer/src/join_selection.rs` — partitioned-vs-CollectLeft, swap logic. (To be re-read at Prism's pinned version.)
- **[DF-nlj-fix]** apache/datafusion GH #22808 — NestedLoopJoinExec ProbeEnd / spurious unmatched-left rows fix (confirms outer participation).
- **[DF-dynfilter-bugs]** apache/datafusion GH #16998 (shared DynamicFilterPhysicalExpr breaks recursive queries), #22495 (FilePruner dynamic-filter gating).
- **[Sedona-mem]** SedonaDB memory-management docs (embeds DataFusion) — spill operator matrix, HashJoinExec no-spill statement, FairSpillPool `unspillable_reserve_ratio` 0.2, 100GB temp default, 75% default memory limit.
- **[Trino-rm]** trino.io resource-management + query-management docs — `query.max-run-time`/`max-execution-time`/`max-cpu-time`/`max-memory(-per-node)`/`max-total-memory`/`max-scan-physical-bytes`/`low-memory-killer.policy` + defaults.
- **[Trino-cbo]** trino.io CBO + `join-max-broadcast-table-size` (100MB) + dynamic-filtering caps + `retry-policy=TASK` least-waste.
- **[Trino-scan-issue]** Trino GH issue — `query.max-scan-physical-bytes` no-ops when connector doesn't report physical bytes.
- **[Influx-timeout]** InfluxDB 3 / Cloud Dedicated query-timeout guidance — tiered timeouts (10s/60s/2m/5m), retry/backoff/jitter/circuit-breaker/deadline-propagation (embedder-level, not in DataFusion).
- **[Ballista]** Apache Ballista architecture docs — scheduler/stages; no documented per-query guardrails.
- **[DF-fed]** datafusion-federation crate — logical-plan pushdown / SQL unparsing; no published guardrails.
- **[Spice]** Spice.ai blog — Ballista integration; guardrails unpublished.
