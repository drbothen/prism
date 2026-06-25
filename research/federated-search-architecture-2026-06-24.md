# Federated Search Architecture — Resilience Patterns & Value-Proposition Framing

**Type:** general (technology + market decision-support)
**Date:** 2026-06-24
**Author:** research-agent
**Scope:** Decision-support for two Prism workstreams — (1) federated-connectivity RESILIENCE design (per-sensor timeout/retry config schema), and (2) reframing the product brief around FEDERATED SEARCH as the core value proposition.
**Status:** complete

> **Provenance note.** All claims below are grounded in cited web sources retrieved 2026-06-24 via Perplexity `sonar-deep-research`. Where a conclusion rests on general distributed-systems practice rather than a specific cited document, it is flagged **[design-inference]**. Where vendor marketing may diverge from technical reality, it is flagged **[marketing≠reality]**. Citation numbers are namespaced per thread: `[R-n]` = Thread-1 resilience sources, `[V-n]` = Thread-2 value-prop sources (see Sources sections).

---

## Executive Summary (read this first)

**Thread 1 — Resilience.** The resilience canon strongly and consistently supports a layered approach for federated query over unreliable downstream systems: (a) explicit per-operation **deadlines** separated from transport connect/read timeouts, with deadline **propagation** into every subquery so one slow source cannot blow the whole budget; (b) **retry with exponential backoff + jitter** for transient `429/503/504`, disciplined by the remaining query budget and gated on idempotency; (c) **partial-result / best-effort federation semantics** — return what succeeded plus per-source diagnostics rather than failing the whole query (this is the single most important pattern for Prism, and it is exactly what mature systems like Elasticsearch/OpenSearch cross-cluster search do); (d) **bulkheads** (concurrency limits per source) which help at ALL traffic levels; and (e) **hedged requests** for tail-latency, applicable only to idempotent reads and largely *not* needed at Prism's low QPS.

**Circuit-breaker claim verdict: PARTIALLY VALIDATED, but the word "strictly" is too strong.** The architect's claim — that circuit breakers are net-negative for a low-frequency human-driven MCP query pattern — is **directly supported by vendor documentation** for the specific failure mode described (a misconfigured low-threshold breaker tripping Open on a handful of failures and worsening visible recovery latency). Microsoft's official Polly guidance explicitly states that below ~1 request/second "the traffic volume is too low to justify a circuit breaker policy" [R-12], and Polly's default `MinimumThroughput` of 100 executions per 30s window means a low-QPS service **will never trip the breaker even if every call fails** [R-11]. Hystrix's default request-volume threshold of 20 enforces the same floor [R-10]. So the documentation endorses the *conclusion* (don't run a circuit breaker at this QPS) while *refining the mechanism*: the harm comes from **misconfiguration**, not from the pattern being inherently bad. The defensible Prism decision is: **omit circuit breakers from the per-query downstream path; rely on fast-fail timeouts + bounded per-query retry + bulkheads + partial-result semantics.** This is well-supported. The "strictly better" phrasing should be softened to "the correct default for Prism's QPS profile" — there is a narrow contradicting case (a *prolonged* multi-minute outage where a properly-tuned, high-threshold breaker fails-fast instead of making each analyst eat a full connect timeout), but that benefit is better delivered in Prism by a cheap per-sensor **health/availability cache** (skip-unavailable flag) than by a statistical breaker. See §1.4.

**Recommended per-sensor config schema** (TOML): the two fields you are adding now (`connect_timeout_secs` default 5, `request_timeout_secs` default 30) are correctly named and correctly defaulted against the canon. Add `retry` (a small sub-table: `max_attempts`, `backoff_initial_ms`, `backoff_max_ms`, `jitter`, `retry_on_status`) and a `skip_unavailable` boolean. Keep a global default block with per-sensor override. Full schema in §1.7. **Note:** the codebase already has an accepted-but-unwired `timeout_secs` overlay field (`prism-spec-engine/src/overlay.rs:828-835`) and a `rate_limit_hints` field on `SensorSpec` (`spec_parser.rs:441`) — the new fields should be reconciled with both (see §1.7 "Codebase reconciliation").

**Thread 2 — Value proposition.** "Federated search" / "search across your security tools without centralizing the data" is now a **central, validated market theme** (2024–2026), not a niche feature. Query.AI is the purest exemplar of Prism's exact model — API-bridge, no ingestion, read-only, **OCSF normalization at query time** [V-4][V-11][V-12] — which both validates Prism's architecture and defines the competitor whose language Prism must differentiate against. The federated model wins on: **no data duplication/egress, query-in-place freshness, ingestion-cost avoidance, and data residency/sovereignty.** It loses on deep historical analytics and complex long-running correlation. Prism's differentiation thesis: **per-analyst, MCP-native, ephemeral federated query with OCSF normalization, purpose-built for the MSSP multi-tenant reality** — i.e., federation delivered as an AI-agent-consumable tool inside the analyst's workflow (Claude Code), not as another web console an analyst logs into. Five sharp value-prop statements in §2.4.

---

## THREAD 1 — Federated-Connectivity Resilience

### 1.1 Timeout strategy: connect vs read vs deadline, and deadline propagation

**Well-established.** Three distinct concepts must not be conflated:

- **Connect timeout** — how long to wait to establish TCP/TLS before declaring the peer unreachable. Short, tuned to expected RTT.
- **Read / response timeout** — how long to wait for data on an established connection. Longer; accommodates data transfer.
- **Deadline** — an *application-level absolute point in time* by which the operation must complete. gRPC defines a deadline as "a point in time past which a client is unwilling to wait for a response" and is explicit that this differs from a duration: a duration is converted to a deadline at call start, accounting for elapsed time and clock skew [R-1].

The canonical failure mode the deadline concept defends against is **exactly Prism's risk**: if the engine relies only on socket read timeouts with no query-level deadline, a single misbehaving source can stall the whole federated query until the longest socket timeout expires (minutes) [R-1]. gRPC's guidance: always set realistic explicit deadlines; treat absence of a deadline as an exceptional/diagnostic condition, not the default [R-1].

**Deadline propagation** is the load-bearing pattern for federation. gRPC implementations automatically propagate the deadline from an incoming RPC to outgoing RPCs, shielding against clock skew and avoiding error-prone manual subtraction [R-1]. Translated to Prism: a single end-user (analyst/agent) query defines a time budget → converted to an absolute deadline on the query context → each fan-out subquery gets `deadline = min(per_source_budget, original_query_deadline)`, never longer. Retries and any hedging must operate *inside* that same budget [R-1]. Google SRE's "Addressing Cascading Failures" reinforces that bounding request lifetimes prevents downstream slowness from metastasizing into systemic failure [R-6].

**Real-system anchors:** Trino exposes `query.client.timeout` (cluster keeps running a query without client contact, then cancels) [R-3], conceptually a server-side deadline. Denodo (data virtualization) cleanly separates a tool-level query timeout (`backend.connection.queryTimeout`, default 900,000 ms) from a per-HTTP-data-route timeout (default 120,000 ms, tunable in the VQL route definition) — concrete proof that mature data-virtualization platforms maintain *layered, independently-tunable* timeouts [R-16]. The Presto "one big bad query crashed all concurrent queries" issue is the cautionary tale of what happens with inadequate per-query limits and resource isolation [R-4].

> **[design-inference]** Public docs for Apollo GraphQL Federation/Router and for security products (Google SecOps/Chronicle, Splunk Federated Search) do **not** spell out their internal timeout-layering or deadline-propagation strategies. Conclusions about those specific products rest on general practice, not cited implementation detail. This was flagged as inconclusive by the source [R-1][R-3][R-15].

### 1.2 Retry with exponential backoff + jitter

**Well-established and strongly endorsed.** Retry is safe for **idempotent** operations (reads/searches — Prism's primary workload is read-only federated query, so this is favorable) and unsafe/needs-idempotency-keys for writes [R-5][R-6]. Transient signals to retry: `429`, `503`, `504`, connection resets, transient DNS.

The central rule: **never retry without jitter.** AWS's "Exponential Backoff and Jitter" (2015) demonstrates via simulation that naive exponential backoff *synchronizes* client retry schedules into periodic load spikes that prevent a struggling service from recovering; adding randomized jitter ("full jitter" — delay chosen uniformly in `[0, exp_cap]`) dramatically improves fairness and recovery [R-5]. Google SRE "Addressing Cascading Failures" independently mandates randomized exponential backoff and cites the same AWS work; uncontrolled retries amplify partial outages into cascading failures [R-6].

**Integration with deadlines is mandatory:** before each retry, check remaining budget; if remaining time can't accommodate another attempt + its backoff, **fail fast** for that subquery and return partial results rather than burning the whole query budget [R-1][R-5][R-6]. For a *federated* source contributing one of many results, prefer fewer retries / shorter backoff — fail the source quickly and degrade gracefully.

> Trino/Denodo/Elasticsearch docs describe their timeout and fault-tolerance behavior but do **not** publicly detail whether their internal retries use jitter — so the jitter mandate is taken from AWS/SRE canon (which is general and high-impact), not from those products' docs [R-15][R-16][R-13].

### 1.3 Hedged / speculative requests (tail-latency)

**Well-established pattern, low relevance to Prism's near-term path.** Dean & Barroso's "The Tail at Scale" (the canonical reference, summarized at [R-7]) shows that when one request fans out to many backends, overall latency is governed by the slowest, so p99.9 composed latency balloons. **Hedged requests** mitigate this: send a duplicate to another replica after a *delay*, use the first response, cancel the rest. The famous Bigtable benchmark: a hedged request after a 10 ms delay cut p99.9 from 1,800 ms → 74 ms for **+2% load** [R-7]. gRPC operationalizes this as a configurable hedging policy (max attempts + hedging delay) [R-8].

**Constraints that make this lower-priority for Prism:** hedging applies only to **idempotent reads**; it *adds* load to already-stressed backends; and it can skew rate-limiting (`429`) and any failure-rate statistics [R-7][R-8]. Most Prism downstreams are single-endpoint third-party APIs (no replica to hedge against), and at low QPS the tail-latency-at-fan-out-scale problem is muted. **Recommendation:** defer hedging; it is not part of the v1 per-sensor schema. Revisit only if a specific sensor exposes redundant regional endpoints. **[design-inference for Prism applicability; the pattern itself is well-cited.]**

### 1.4 Circuit breakers & bulkheads — the architect's claim, validated

**The claim under test:** *"Circuit breakers are net-negative for a low-frequency human-driven MCP query pattern because an analyst sending a few queries against a briefly-down sensor would trip the breaker Open and make visible recovery latency WORSE; fast-fail + per-query retry is strictly better at low QPS."*

**Verdict: PARTIALLY VALIDATED. The conclusion is well-supported by vendor documentation; the mechanism is more nuanced; "strictly" overstates.**

**Supporting evidence (strong, from primary vendor docs):**

- **Polly (Microsoft .NET official):** the circuit breaker samples results over a `SamplingDuration`; **"the circuit will not break even if all of the executions failed"** when call count is below `MinimumThroughput`. Defaults: `FailureRatio` 0.1, `MinimumThroughput` **100**, `SamplingDuration` 30 s, `BreakDuration` 5 s [R-11]. At Prism's QPS, you'd never reach 100 calls in 30 s, so the default breaker is *inert* — confirming a breaker adds state/complexity for no benefit.
- **Microsoft .NET Blog "Circuit Breaker Policy Fine-tuning Best Practice":** explicitly recommends `MinimumThroughput > SamplingDuration × 1` (≥ ~1 req/s average) and states that if lower, **"the traffic volume is too low to justify a circuit breaker policy."** [R-12] This is a near-verbatim endorsement of the architect's conclusion.
- **Hystrix:** `requestVolumeThreshold` default **20** — below 20 requests in the rolling window the breaker *never* opens regardless of failures [R-10]. Same low-QPS floor.
- **resilience4j:** uses a configurable minimum-number-of-calls before the breaker can open, implicitly recognizing that low call volumes can't yield a reliable failure-rate estimate [R-9].

**The mechanism refinement (why "strictly" is too strong):**

1. The harm the architect describes — a few failures tripping Open and blocking subsequent recoverable queries — arises specifically from a **misconfigured** breaker (low `MinimumThroughput`/`requestVolumeThreshold`, long `BreakDuration`). A *properly-tuned* breaker at low QPS is **inert**, not harmful [R-10][R-11][R-12]. So the precise statement is: "a circuit breaker tuned for this QPS does nothing useful; a circuit breaker mis-tuned does harm." Either way the engineering decision (don't run one) is correct.
2. **Narrow contradicting case:** during a *prolonged* (multi-minute) source outage, even low-QPS users each pay a full connect-timeout per query. A high-threshold breaker could fail-fast with a clear "source unavailable" instead [R-9][R-10][R-11][R-12]. **However**, for Prism this benefit is better and more cheaply obtained by a **per-sensor availability/health cache + `skip_unavailable` semantics** (see §1.5, §1.7) than by a statistical breaker — you get fast-fail-with-clear-diagnostic without the false-positive risk of a sliding-window breaker on sparse data.

**Bulkheads are a separate story — keep them.** Bulkheads (concurrency caps per downstream) help at **all** traffic levels because they address *concurrency*, not *frequency* [R-17]. Even a low-QPS human/agent workload bursts (an incident triggers several simultaneous fan-out queries); without a bulkhead, one slow source can consume all worker threads and starve healthy sources. resilience4j defaults: `SemaphoreBulkhead` `maxConcurrentCalls` 25, `maxWaitDuration` 0 (immediate rejection when saturated) [R-17]. **Prism already implements the right primitive** — `MAX_FANOUT_CONCURRENCY = 10` nested under `HTTP_SEMAPHORE_PERMITS = 200` (per CLAUDE.md / `prism-sensors/src/fanout.rs`) *is* a bulkhead. No new schema field needed for v1; document it as the bulkhead. **[design-inference linking the codebase pattern to the resilience4j bulkhead concept.]**

**Net recommendation for Prism:** No circuit breakers in the per-query downstream path. Use: fast-fail connect/read timeouts (§1.1) + bounded per-query retry with jitter (§1.2) + the existing fan-out bulkhead (§1.4) + partial-result semantics with per-source diagnostics (§1.5) + an optional lightweight per-sensor availability cache to fast-fail during prolonged outages without a statistical breaker.

### 1.5 Best-effort / partial-failure federation semantics (the most important pattern for Prism)

**Well-established; this is the model Prism should adopt wholesale.** Elasticsearch/OpenSearch cross-cluster search (CCS) is the mature reference implementation:

- A search with some shard/cluster failures returns **HTTP 2xx (success) with partial results**; a search fails (4xx/5xx) only when a cluster marked `skip_unavailable=false` is unavailable, disconnects, or fails on all shards [R-13].
- The response carries structured failure metadata: `_shards` (per-shard failures) and `_clusters` (per-cluster status: each marked `partial` / `skipped` / `failed` with failed-shard counts and specific errors) [R-13].
- Partial results are **configurable** at cluster and request level: OpenSearch `default_allow_partial_results` (cluster) and `allow_partial_search_results` (request) [R-18][R-19]; Elasticsearch defaults `allow_partial_search_results=true` [R-20]. Per-cluster opt-out via `skip_unavailable` [R-13].

**Design patterns extracted for Prism** [R-13][R-18][R-19][R-20]:
1. **Separate transport success from logical completeness** — a query that reached some sources successfully is a *success* with a `partial` flag, not an error.
2. **Structured per-source failure metadata** — emit per-sensor status, error code (map to Prism's `E-SENSOR-NNN` / `E-QUERY-NNN` taxonomy), and timestamp so the consuming agent can reason about what's missing.
3. **Configurable strictness** — a `skip_unavailable`-style per-sensor flag lets a query opt a critical source into all-or-nothing while letting best-effort sources degrade.

This aligns directly with Prism's existing standing rule (CLAUDE.md / Standing Rule 3 §2): *no silent `Vec::new()` return where partial-failure data should propagate* — propagate via `prism-query` partial-failure handling (BC-2.01.010). The research **confirms this is the industry-correct posture**, not just a project preference.

> GraphQL/Apollo Federation also support partial results (`data` + `errors`) but this was **not in the cited corpus** — treat as corroborating general practice, not a cited claim [R, design-inference]. Trino's fault-tolerant execution is about *internal* worker-failure recovery (spool + retry tasks), **not** external-source partial results [R-15] — do not over-cite Trino here.

### 1.6 Connection pools & automatic recovery under intermittent connectivity

**Moderately documented; general practice fills gaps.** Connection pools serve two roles: amortize TLS/connection setup, and act as a coarse bulkhead via concurrency limits [R-16][R-4]. Key risks and mitigations [R-1][R-6][R-7][R-17]:
- A pool that allows too many concurrent long-lived connections to a slow source lets threads pile up and starves healthy sources → cap per-source concurrency (bulkhead).
- **Asynchronous/background reconnection** is the right recovery pattern: mark a connector degraded, fail user queries fast (or return partial), and probe for recovery in the background — *no restart required*. Combine with backoff+jitter on reconnection attempts so all threads don't stampede a recovering source simultaneously [R-5][R-6].
- On deadline/cancel, ensure in-flight responses don't get attributed to a cancelled request; gRPC handles cancellation cleanly and is the model [R-1].

> Internal connection-management algorithms for the specific named products were **not** in public docs — this section is the most inference-heavy. Prism's `reqwest::Client` reuse with the mandated 30s timeout (CLAUDE.md HTTP-client rule) plus `ArcSwap` config hot-reload already provides the no-restart-recovery substrate. **[design-inference]**

### 1.7 DELIVERABLE — Recommended per-sensor timeout/retry config schema (TOML)

**Naming verdict on your proposed fields:** `connect_timeout_secs` (default 5) and `request_timeout_secs` (default 30) are **well-named and well-defaulted.** They correctly map to the canon's connect-vs-read distinction (§1.1). The `_secs` suffix matches the existing `timeout_secs` convention already in the codebase (`overlay.rs`). Keep them.

**Recommended full schema** (global defaults + per-sensor override; per-sensor wins):

```toml
# ---- GLOBAL defaults (e.g., in prism config / a [defaults.connectivity] block) ----
[defaults.connectivity]
connect_timeout_secs   = 5      # TCP/TLS establishment. Short. (§1.1)
request_timeout_secs   = 30     # read/response timeout on established conn. (§1.1; matches CLAUDE.md 30s rule)
# query-level deadline is NOT per-sensor — it is the caller's budget, propagated down (§1.1).

[defaults.connectivity.retry]
max_attempts           = 3      # total attempts incl. first; bounded by query deadline (§1.2)
backoff_initial_ms     = 200    # first backoff
backoff_max_ms         = 5000   # cap; full-jitter chooses uniform in [0, current_cap] (§1.2)
jitter                 = "full" # "full" | "none"  — default MUST be full (AWS/SRE) (§1.2)
retry_on_status        = [429, 503, 504]   # plus transport errors (conn reset). NOT 4xx-client. (§1.2)
respect_retry_after    = true   # honor Retry-After header on 429/503 (caps backoff floor)

# ---- PER-SENSOR override (in <sensor>.sensor.toml) ----
[connectivity]                  # all fields optional; fall back to global defaults
connect_timeout_secs   = 5
request_timeout_secs   = 45     # e.g., a slow sensor gets a longer read timeout
skip_unavailable       = true   # best-effort: partial results if this source is down (§1.5)
                                # false => this sensor's failure fails the whole query (critical source)

[connectivity.retry]
max_attempts           = 2      # e.g., a rate-limited sensor: fewer retries, fail fast
retry_on_status        = [429, 503]
```

**Field-by-field rationale & per-sensor-vs-global guidance:**

| Field | Per-sensor? | Default | Rationale / citation |
|---|---|---|---|
| `connect_timeout_secs` | yes (override) | 5 | Connect timeout, short. §1.1 [R-1][R-16] |
| `request_timeout_secs` | yes (override) | 30 | Read timeout; matches CLAUDE.md 30s mandate. §1.1 [R-1][R-16] |
| `retry.max_attempts` | yes | 3 | Bounded; must respect remaining deadline. §1.2 [R-5][R-6] |
| `retry.backoff_initial_ms` / `backoff_max_ms` | global usually; per-sensor for rate-limited APIs | 200 / 5000 | Exponential cap. §1.2 [R-5] |
| `retry.jitter` | global | `full` | **Non-negotiable default.** §1.2 [R-5][R-6] |
| `retry.retry_on_status` | yes (rate-limited sensors differ) | `[429,503,504]` | Transient-only; never retry non-idempotent or 4xx-client. §1.2 [R-5][R-6] |
| `retry.respect_retry_after` | global | true | Honor server backpressure (429/503 `Retry-After`). §1.2 [R-5] |
| `skip_unavailable` | yes | true | Best-effort federation; CCS-style partial results. §1.5 [R-13][R-18][R-19][R-20] |
| query **deadline / budget** | **NOT per-sensor** | caller-supplied | Belongs to the query context, propagated to subqueries. §1.1 [R-1] |
| fan-out concurrency (bulkhead) | **NOT per-sensor v1** | `MAX_FANOUT_CONCURRENCY=10` | Already implemented globally. §1.4 [R-17] |
| hedging | **omit v1** | — | Not needed at low QPS / single-endpoint sensors. §1.3 [R-7][R-8] |
| circuit breaker | **omit** | — | Net-negative/inert at Prism QPS. §1.4 [R-10][R-11][R-12] |

**Codebase reconciliation (important):**
- There is an existing **accepted-but-not-wired** `timeout_secs` overlay field (`prism-spec-engine/src/overlay.rs:828-835`, emits `overlay.timeout_secs_ignored`). The new `connectivity` block should **supersede or absorb** this: either wire `timeout_secs` → `request_timeout_secs`, or deprecate it in favor of the structured `[connectivity]` block. **Do not leave two parallel timeout concepts.** This is a TD-VSDD-060 sibling-sweep concern — route the reconciliation decision to the architect, but the production-grade default is to unify them now.
- `SensorSpec` already carries `rate_limit_hints: Option<RateLimitHints>` (`spec_parser.rs:441`). `retry.respect_retry_after` and `retry.retry_on_status=[429,...]` should be designed to *cooperate* with `rate_limit_hints`, not duplicate it — e.g., rate-limit hints inform proactive throttling; retry config informs reactive backoff. Flag the overlap for the architect.
- All new public TOML-deserialized types (`ConnectivityConfig`, `RetryConfig`) require `#[non_exhaustive]` per CLAUDE.md (and the `ci.yml EXPECTED=83` gate must be bumped). This is a hard project rule.

---

## THREAD 2 — Federated Search as Value Proposition

### 2.1 Federated/query-in-place vs centralized ingestion — articulated tradeoffs

**The market frames it as a spectrum, trending hybrid — not either/or** [V-3][V-10]. The "SOC data layer" is now actively contested: SIEM vendors are bolting on lake/federation features while lake vendors add detection [V-3].

**Where federated WINS (cited):**
- **No data movement / sovereignty:** keep data in-region, query in place. Query.AI's defining trio: "data access without data movement," "answers without pivots and friction," "context without complexity" [V-4]. Gurucul: federated SIEM searches across S3, AD, SaaS HR, hypervisors, Windows logs **without cross-cloud or restricted-region transfers** [V-5]. Splunk Federated Search: keep data where it lives for sovereignty/compliance [V-13]. Pax8 grounds *why* this matters (PIPEDA, Quebec Law 25 — residency vs sovereignty) [V-8].
- **Ingestion-cost avoidance:** concrete proof point — Splunk's Autodesk case: **28% ingestion-cost reduction** by routing only critical logs to Splunk and querying the rest in S3 [V-13]. SentinelOne notes SIEM volume-based pricing makes "ingest everything" impractical [V-10]. PwC white paper names "federated search support" as a way to query across stores "without re-ingestion," reducing duplication and cost [V-17].
- **Freshness / coverage:** query the live source, no ETL lag; reach data never justified for continuous ingestion [V-4][V-5][V-11].

**Where federated LOSES (cited, honest):**
- **Deep historical analytics & complex correlation:** centralized/indexed lakes win on low-latency, long-window, heavy analytical queries; federated queries inherit each source's API latency, rate limits, and query-expressiveness limits [V-3][V-10][V-13][V-17]. **[marketing≠reality]** Vendor "seamless/fast" claims gloss over API rate limits and network latency — buyers should expect federation to excel at *investigative, entity-centric, time-bounded* queries and underperform on massive historical analytics [V-4][V-11][V-13].
- **New security surface:** a federation layer with read-only API access to many systems concentrates credential power — a compromise queries everything [V-4][V-9]. (Directly relevant to Prism's AI-opaque-credential model and prompt-injection-defense posture — a *differentiation opportunity*, see §2.3.)

### 2.2 Competitive landscape & value language

| Vendor | Model | Buyer-facing language | Citation |
|---|---|---|---|
| **Query.AI** | **Pure federated, no ingestion, read-only API bridge, OCSF normalization at query time, FSQL + LLM copilot** | "Federated Security"; "security data mesh"; "one search across all security data without centralization"; "API bridge to data wherever it resides" | [V-4][V-11][V-12] |
| **Gurucul** | "Universal Federated Search" / "Federated SIEM" (feature of broader SIEM) | "query any data source from a single interface"; bridge silos, retain data locally, multi-cloud without egress | [V-1][V-5] |
| **Splunk** | Federated Search as extension of ingestion-centric platform (Splunk↔Splunk, Splunk↔S3) | "query data wherever it lives"; "eliminate the need to move or duplicate data"; smart routing + auto schema detection | [V-13] |
| **Google SecOps (Chronicle)** | Centralized cloud SIEM (UDM); federation added *via Query.AI connector* | high-scale telemetry retention; UDM normalization; federation is an overlay, not native | [V-12] |
| **Hunters** | Lake-centric next-gen SIEM on Snowflake, MSSP multi-tenant | "limitless scale"; multi-tenant SOC; data-lake economics + automation — **still centralizes into Snowflake** | [V-15] |
| **Anvilogic** | Detection-layer federation across Splunk + Snowflake | "unify SIEM and data lake without replacing Splunk"; run detections where data lives | [V-16] |
| **CardinalOps** | Federated *detection-posture* view across multiple SIEMs | "federated view of MITRE ATT&CK coverage across SIEMs" | [V-14] |
| **AWS Security Lake** | Centralized-but-query-in-place: OCSF in S3, queried via Athena/Redshift/Spark | standardized OCSF lake, multiple engines query in place via Lake Formation | [V-2][V-18][V-19] |
| **Elastic** | Cross-cluster search (intra-Elastic federation, ECS schema) | *No 2024–26 federated-security marketing doc in corpus* — treat as intra-Elastic only | **[inconclusive — flagged by source]** |

**Key competitive read:** **Query.AI is Prism's mirror** — same architecture (API bridge, no ingestion, read-only, OCSF-at-query-time). This *validates* Prism's technical bet and simultaneously sets the language Prism must out-differentiate. Everyone else either centralizes (Hunters, Google SecOps, Splunk-core) or federates only within their own ecosystem (Elastic CCS) or at the detection layer (Anvilogic, CardinalOps).

### 2.3 Where an MSSP per-analyst OCSF-federated tool fits & differentiates

**OCSF is the lingua franca** [V-6][V-7][V-19]. Created by Splunk/AWS/IBM et al.; AWS Security Lake normalizes to OCSF in Parquet; Splunk ingests OCSF→CIM; Query.AI normalizes to OCSF at query time [V-4][V-6][V-7][V-12][V-19]. For an MSSP, standardizing internally on OCSF means **write detection/analytics once, run across heterogeneous customer stacks** (one customer on Google SecOps, another on Splunk, another on Security Lake) — a structural advantage [V-4][V-6][V-7][V-19].

**MSSP-specific pains the federated model addresses** [V-5][V-8][V-14][V-15]: heterogeneous per-customer tool stacks; multi-tenant visibility; ingestion-cost margin pressure; customer data-residency mandates that forbid centralizing into the MSSP's own SIEM. CardinalOps documents the multi-SIEM reality; Hunters documents the multi-tenant requirement [V-14][V-15].

**Prism's unfilled gap (differentiation space):** every cited competitor delivers federation as **another web console an analyst logs into**. None of the cited sources describe a **per-analyst, MCP-native, AI-agent-consumable** federated query tool that runs *inside the analyst's existing agent workflow* (Claude Code) with **AI-opaque credential handling** and prompt-injection-defended output. That is Prism's whitespace. **[design-inference — absence-of-evidence in the cited corpus; flagged as such, not a proven negative.]**

### 2.4 DELIVERABLE — Value-proposition statements + differentiation thesis

**Five sharp value-prop statements (citation-grounded):**

1. **"Query your security tools where the data already lives — no ingestion, no duplication, no egress."** Prism is an ephemeral federated query engine, not another data lake to fill. *(Validated market language: Query.AI [V-4][V-11], Splunk [V-13], Gurucul [V-5], PwC [V-17].)*

2. **"Stop paying twice. Cut SIEM ingestion cost by querying in place."** Federation lets you keep low-value/high-volume telemetry in cheap storage and query it on demand — the Autodesk-via-Splunk pattern delivered **28% ingestion-cost reduction** [V-13]. *(Cost canon: [V-10][V-13][V-17].)*

3. **"Respect data residency by design — answers cross borders, your data doesn't."** Prism queries in-region and normalizes results at query time; raw customer data never leaves its jurisdiction. *(Sovereignty canon: Gurucul [V-5], Splunk [V-13], Pax8/PIPEDA/Law 25 [V-8].)*

4. **"One normalized view across every tool — OCSF at query time, not after a six-month ETL project."** Heterogeneous sensor APIs become a single OCSF schema the moment you query, so analysts learn one schema, not twenty. *(OCSF-as-differentiator: [V-4][V-6][V-7][V-12][V-19].)*

5. **"Federated search built for the analyst's agent, not for yet another browser tab."** Prism delivers federation as a per-analyst, MCP-native tool inside Claude Code, with credentials the AI never sees and output hardened against prompt injection. *(Prism-unique whitespace — not occupied by any cited competitor [V-4][V-11][V-12]; AI-opaque-credential security posture motivated by [V-4][V-9].)*

**Differentiation thesis (one paragraph for the brief):**

> Federated search is the core value proposition: Prism is an *ephemeral, per-analyst, federated query engine* that lets MSSP analysts and their AI agents search across every customer security tool **without centralizing the data** — query-in-place over heterogeneous APIs, normalized to OCSF at query time, returning best-effort partial results with per-source diagnostics. The market has validated this model (Query.AI, Gurucul, Splunk Federated Search all sell "search without centralization" on cost, freshness, and data-sovereignty grounds [V-4][V-5][V-13][V-17]), and OCSF has emerged as the normalization standard that makes cross-tool federation tractable [V-6][V-7][V-19]. Prism's differentiation is **delivery and trust model**, not just architecture: where every competitor ships federation as another centralized console an analyst logs into, Prism ships it as an **MCP-native tool the analyst's agent invokes directly** — purpose-built for the MSSP multi-tenant reality, with **AI-opaque credentials** and prompt-injection-defended output that no cited competitor offers. Federation is table stakes; *agent-native, credential-safe, OCSF-normalized federation for MSSPs* is the wedge.

> **[marketing≠reality caveat for the brief]:** Be honest internally that federation underperforms centralized lakes on deep historical/complex-correlation analytics and inherits downstream API rate-limit/latency constraints [V-3][V-10][V-13][V-17] — position Prism for *investigative, entity-centric, time-bounded* queries (its actual sweet spot), not as a SIEM replacement for long-window analytics.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | Thread-1 resilience canon (deep, `reasoning_effort=high`, 20 cited sources); Thread-2 federated-search market/value positioning (deep, `reasoning_effort=high`, 19 cited sources) |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — (no library-API question; resilience patterns are cross-cutting, not single-library) |
| Tavily (all) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Codebase reads (Read/Grep/Glob) | 6 | Grounded the config-schema deliverable in actual `SensorSpec` (`spec_parser.rs:424`), existing unwired `timeout_secs` overlay (`overlay.rs:828`), `rate_limit_hints`, and the fan-out bulkhead — NOT external research, used to make the deliverable actionable |
| Training data | 2 areas | (a) GraphQL/Apollo partial-error semantics (flagged [design-inference], not in cited corpus); (b) linking resilience4j bulkhead concept to Prism's existing `MAX_FANOUT_CONCURRENCY` (flagged) |

**Total MCP tool calls:** 2 (both `perplexity_research`, the mandated PRIMARY tool, both at `reasoning_effort=high` for architecture/competitive decision-support — exactly the high-default case).
**Training data reliance:** **low** — every substantive claim is tied to a numbered web source retrieved 2026-06-24; the two training-data areas are explicitly flagged as design-inference and are not load-bearing for the recommendations.

---

## Sources

### Thread 1 — Resilience (retrieved 2026-06-24)

- [R-1] gRPC — Deadlines. https://grpc.io/docs/guides/deadlines/
- [R-2] Google SRE Workbook (index). https://sre.google/workbook/index/
- [R-3] Trino — Query Management Properties (Trino 481). https://trino.io/docs/current/admin/properties-query-management.html
- [R-4] PrestoDB issue #8077 — "One big bad query crashed all concurrent running queries." https://github.com/prestodb/presto/issues/8077
- [R-5] AWS Architecture Blog — Exponential Backoff and Jitter (2015). https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/
- [R-6] Google SRE Book — Addressing Cascading Failures. https://sre.google/sre-book/addressing-cascading-failures/
- [R-7] Uwe Friedrichsen — "The Tail at Scale" (Dean & Barroso summary). https://www.ufried.com/blog/tail_at_scale/
- [R-8] gRPC — Request Hedging. https://grpc.io/docs/guides/request-hedging/
- [R-9] resilience4j — CircuitBreaker. https://resilience4j.readme.io/docs/circuitbreaker
- [R-10] D. Mydlarz — "Hystrix Circuit Breaker: how to set it up properly" (Medium). https://medium.com/@darekmydlarz/hystrix-circuit-breaker-how-to-set-it-up-properly-84c75cfbe3ee
- [R-11] Polly docs — Circuit Breaker resilience strategy. https://www.pollydocs.org/strategies/circuit-breaker.html
- [R-12] Microsoft .NET Blog — "Circuit Breaker Policy Fine-tuning Best Practice" (2023). https://devblogs.microsoft.com/dotnet/circuit-breaker-policy-finetuning-best-practice/
- [R-13] Elastic — Cross-cluster search. https://www.elastic.co/docs/explore-analyze/cross-cluster-search
- [R-14] OpenSearch — Search API reference. https://opensearch.org/docs/latest/api-reference/search/
- [R-15] Trino — Fault-tolerant execution. https://trino.io/docs/current/admin/fault-tolerant-execution.html
- [R-16] Denodo KB — "Read timed out creating data source with HTTP data route." https://community.denodo.com/kb/view/document/Read%20timed%20out%20creating%20data%20source%20with%20HTTP%20data%20route
- [R-17] resilience4j — Bulkhead. https://resilience4j.readme.io/docs/bulkhead
- [R-18] OpenSearch — Search settings (`default_allow_partial_results`). https://docs.opensearch.org/latest/install-and-configure/configuring-opensearch/search-settings/
- [R-19] OpenSearch — Search APIs (`allow_partial_search_results`). https://docs.opensearch.org/latest/api-reference/search-apis/search/
- [R-20] Elasticsearch — Search API (8.19). https://www.elastic.co/guide/en/elasticsearch/reference/8.19/search-search.html

### Thread 2 — Value proposition (retrieved 2026-06-24)

- [V-1] Gurucul — "Federated Search Tools: Query All Data and Save Costs." https://gurucul.com/blog/federated-search-tools-query-all-data-and-save-costs/
- [V-2] AWS Security Blog — Amazon Security Lake simplifying security data management. https://aws.amazon.com/blogs/security/how-amazon-security-lake-is-helping-customers-simplify-security-data-management-for-proactive-threat-analysis/
- [V-3] Software Analyst (Substack) — "The Convergence of SIEMs and Data Lakes." https://softwareanalyst.substack.com/p/the-convergence-of-siems-and-data
- [V-4] Query.AI — "What Federated Search Taught Us: The Case for a Security Data Mesh." https://www.query.ai/resources/blogs/what-federated-search-taught-us-the-case-for-a-security-data-mesh/
- [V-5] Gurucul — "Why Federated Search Software Is Crucial for Multi-Cloud Architectures." https://gurucul.com/blog/why-federated-search-software-is-crucial-for-multi-cloud-architectures/
- [V-6] Splunk — "Open Cybersecurity Schema Framework (OCSF)." https://www.splunk.com/en_us/blog/learn/open-cybersecurity-schema-framework-ocsf.html
- [V-7] Splunk — "Enhancing SOC Efficiency with OCSF + Splunk Enterprise Security." https://www.splunk.com/en_us/blog/security/enhancing-soc-efficiency-with-ocsf-splunk-enterprise-security.html
- [V-8] Pax8 — "Data Residency vs Data Sovereignty in Canada." https://www.pax8.com/blog/data-residency-vs-data-sovereignty-in-canada/
- [V-9] MongoDB — "Queryable Encryption and Ephemeral Security" (talk). https://www.youtube.com/watch?v=eHsipdkwdqg
- [V-10] SentinelOne — "Security Data Lake vs SIEM." https://www.sentinelone.com/cybersecurity-101/data-and-ai/security-data-lake-vs-siem/
- [V-11] Query.AI — Federated Search product page. https://www.query.ai/federated-search/
- [V-12] Query.AI — "Google SecOps Integrated into Query Federated Search." https://www.query.ai/resources/blogs/google-secops-integrated-into-query-federated-search/
- [V-13] Splunk — "Unifying Your Data with Federated Search." https://www.splunk.com/en_us/blog/platform/unifying-your-data-with-federated-search.html
- [V-14] CardinalOps — "Manage Detection Posture Across Multiple SIEMs." https://cardinalops.com/use-cases/manage-detection-postureacross-multiple-siems/
- [V-15] Hunters — MSSP page. https://www.hunters.security/mssp
- [V-16] Anvilogic — "Unify Your SIEM and Data Lake Without Replacing Splunk." https://www.anvilogic.com/solution-guide-unify
- [V-17] PwC — "Data Pipeline Management: Building and Operating the Security Data Layer" (white paper PDF). https://www.pwc.de/en/cyber-security/pwc-white-paper-data-pipeline-management-building-and-operating-the-security-data-layer.pdf
- [V-18] AWS — Security Lake subscriber query access. https://docs.aws.amazon.com/security-lake/latest/userguide/subscriber-query-access.html
- [V-19] AWS — Security Lake OCSF. https://docs.aws.amazon.com/security-lake/latest/userguide/open-cybersecurity-schema-framework.html

### Inconclusive / flagged

- **Elastic Security federated-search 2024–26 marketing:** no dated source in corpus; CCS is intra-Elastic only. Treat any cross-vendor "federation" claim for Elastic as unverified.
- **Apollo GraphQL Federation / Router internal timeout & partial-result semantics:** not in cited corpus; partial-error behavior cited from general GraphQL practice only ([design-inference]).
- **Internal retry/timeout/connection-pool algorithms of Trino, Denodo, Google SecOps, Splunk Federated Search:** public docs describe surface config, not internal jitter/propagation logic — those specifics rest on AWS/SRE/gRPC canon, not product docs.
