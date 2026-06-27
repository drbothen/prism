---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-26"
provenance: "side-analysis discussion input; does not modify vision/specs."
topic: "Chain-aware RetentionCache tiering, upstream-replication policy language, and deadline-budget-aware intermediate caching across a multi-hop Satellite chain"
scope: "SIDE-ANALYSIS / discussion input — NOT a spec, vision, brief, ADR, BC, or PRD change."
---

# Chain-Aware Cache Tiering, Upstream Replication, and Deadline Budgets Across the Prism Satellite Chain

> **BOUNDARY.** This is a cited research capture to inform a HUMAN DISCUSSION. It does **not**
> modify `matured-vision-day2-requirements.md`, any spec, STATE.md, SESSION-HANDOFF.md, the prior
> research docs, or any live factory artifact. It does **not** decide the open questions it raises.
> The requester's three working hypotheses are **pressure-tested**, not adopted. `do_not_execute: true`.

> **Citation discipline.** Findings are grounded in MCP-tool-retrieved web research (Perplexity
> `sonar-deep-research`, three deep passes at `reasoning_effort: high`). Vendor/RFC/paper sources are
> named inline with their bracketed source markers as returned by the deep-research synthesis. Where a
> claim rests on model knowledge rather than a retrieved source, it is flagged **[model-knowledge]**.
> Where the research was inconclusive or a source could not be field-verified, it is flagged
> **[INCONCLUSIVE]**. Date-stamped as of 2026-06.

> **Grounding read (modified nothing).** Prior pass `federated-ingestion-collector-connectors-2026-06-26.md`
> (collector class; four-stage abstraction receiver→buffer→normalize→queryable; collection locus;
> push-vs-pull) and matured-vision §3.2 (Satellite chaining: tree rooted at central, dial-home
> outbound-only, per-hop mutual auth, per-hop deadline + partial-failure propagation, store-and-forward,
> regional caching, loop-prevention via seen-request-IDs), §3.3 (tiered RetentionCache: RocksDB hot +
> Iceberg cold, `event_time` TTL, RETAIN, detection-window retention, multi-schema by
> source-class/schema/schema-version), §3.6 (partial-result semantics / BC-2.01.010, CCS-style
> `skip_unavailable`, no circuit breakers at Prism QPS).

---

## 1. Executive Summary (10 lines)

1. **Q1 tiering — hypothesis VALIDATED with one correction.** Mapping cache tiers onto chain topology (RocksDB hot at edge satellite → optional warm at regional hub → Iceberg cold at central) is exactly the edge–hub–central pattern that CDNs, HSM, cloud object storage, time-series, and lakehouses all use. But the **AUTOMATIC-by-default** part of the hypothesis is the wrong default emphasis: every mature system converges on **HYBRID = a declarative policy ENVELOPE with automatic optimization WITHIN it** (S3 Lifecycle vs Intelligent-Tiering; Azure tiers vs Smart tier; Thanos built-in downsampling thresholds vs declared retention flags). Pure-automatic tiering is explicitly considered risky for regulated data.
2. **Topology coupling cuts both ways.** CDN multi-tier hierarchies prove the architecture works, but they also surface the exact failure modes the hypothesis must avoid: **"centralized incoherence"** (a stale hub poisons every edge beneath it), **cache stampede at the hub** (many edges revalidate a popular object at once), **double-caching** (same object resident at multiple tiers wasting space), and **eviction races** (edge evicts before hub, so an edge miss refills from a hub that still holds a different-freshness copy).
3. **Q2 replication policy — primitive set VALIDATED, residency placement CHALLENGED.** The hypothesized `{selector → reduction/normalization → retention → destination → residency}` rule set is confirmed across Cribl, OTel Collector/OTTL, rsyslog/syslog-ng, S3/Iceberg ILM, Kafka MM2 + tiered storage, Prometheus remote-write + recording rules, OPA/Rego. But the survey adds **three missing primitives**: store-and-forward/buffering, QoS/durability level, and **transform ORDERING** — and finds that **residency is almost universally an emergent afterthought, not a first-class per-field constraint.** That gap is precisely Prism's differentiator opportunity.
4. **Residency-first is genuinely rare prior art.** No surveyed system expresses "raw stays in-region; only normalized/sanitized metadata may transit up" as a first-class declarative primitive. Cribl/OTel/Prometheus achieve it *indirectly* (drop/redact before a destination that happens to be cross-region); S3/Kafka express it as bucket-region + replication config; AWS SCPs/OPA express region guardrails at the resource level, not per-field. Prism's residency-first thesis (vision §3.2 #6) would be ahead of, not behind, this prior art.
5. **Q3 deadline budgets — hypothesis VALIDATED, "fixed N-hops" rule CHALLENGED hard.** The deep research is unusually emphatic: prior art does **NOT** substantiate a fixed "mandatory intermediate cache below N hops" heuristic anywhere in mainstream distributed query engines. A fixed depth rule is unsupported.
6. **But the adaptive budget-aware planner the hypothesis proposes is also mostly GREENFIELD.** The building blocks all exist separately — gRPC deadline-to-timeout decrement per hop, Dean & Barroso tail-tolerance (hedged/tied/probe-least-loaded requests), Dremel/Drill multi-level pre-aggregation serving trees, materialized-view selection, semantic caching, deadline-decomposition in workflow/sensor-net scheduling, CAQP (bounded error + bounded time), Spark AQE — but **no production engine integrates them into a deadline-budget-aware cache-placement planner.** Prism would be assembling, not borrowing wholesale.
7. **gRPC already gives the budget plumbing for free.** gRPC converts an absolute deadline to a per-hop timeout by subtracting elapsed time at each hop (clock-skew-safe), failing with `DEADLINE_EXCEEDED` — this is the exact mechanism Prism's per-hop deadline propagation (§3.2) needs, and it composes with partial-result-on-deadline (§3.6 / BC-2.01.010).
8. **Tail amplification is the real argument for intermediate materialization** — not hop count. Dean & Barroso's order-statistics result (`P(at least one slow) ≈ n·p`; deeper+wider trees blow the 99th percentile) means the trigger for inserting a pre-aggregate should be **fan-out width × tail risk × remaining budget**, not raw depth.
9. **The three questions compose into ONE model.** A chain-aware policy attaches, per collector and per parent edge, a declarative `{select → reduce → retain-tier → destination-tier → residency}` rule (Q2) that determines *what materializes at which tier* (Q1), and the coordinator's deadline-budget-aware planner (Q3) reads those materialized tiers as descent shortcuts or returns partial+coverage when a deep subtree can't make the budget. Residency is the cross-cutting invariant that constrains all three.
10. **Hardest honest tensions for the discussion:** (a) coherence of a *demand-driven, event_time-TTL'd* cache across hops is harder than CDN object coherence because the "object" is a query result set, not a URL; (b) residency-first per-field control is net-new policy machinery; (c) the budget-aware planner is research-grade, not off-the-shelf — a simpler "partial-results-on-deadline + opportunistic hub pre-aggregate" may be the production-grade v1 that defers the full planner.

---

## 2. Q1 — Can the RetentionCache tier ALONG the chain, and should tiering be AUTOMATIC or POLICY-DECLARED?

### 2.1 Hypothesis under test
> Tiers map onto topology (RocksDB hot at edge satellite → optional warm at regional hub → Iceberg cold at central), with **AUTOMATIC age/temperature-based demotion as the DEFAULT** but per-collector **POLICY override.**

### 2.2 Prior-art evidence — the edge–hub–central pattern is universal

The deep-research synthesis confirms that **edge–hub–central is a unifying abstraction** across every caching/storage domain surveyed, and that it maps cleanly onto a node topology:

- **CDN multi-tier hierarchies.** Akamai Tiered Distribution (parent caches near origin serve child edge caches) [akamai docs]; Cloudflare Tiered Cache (lower-tier → upper-tier → origin, with optional Regional middle tier; topology options Smart/Generic/Custom) [cloudflare docs]; Fastly Shielding (a designated POP acts as origin shield for other POPs) [fastly docs]; Varnish private CDNs explicitly organize **edge tier / storage tier / origin-shield tier** [varnish]; Apache Traffic Server parent/child cache config [ats docs]; Squid parent/sibling hierarchies with ICP inter-cache queries (default 2s) [squid].
- **HSM / cloud object storage.** Classic HSM migrates SSD↔disk↔tape/archive by age + access frequency [hsm]. AWS S3 Intelligent-Tiering auto-demotes after 30 days no-access (IA), 90 days (Archive Instant) [aws s3]. Azure Blob hot/cool/cold/archive with min-duration guidance + a **Smart tier** that auto-moves among online tiers [azure]. BigQuery active→long-term storage auto-transition at 90 days unmodified [bigquery].
- **Time-series.** Prometheus+Thanos: edge Prometheus → Thanos Store Gateway/Compactor (hub) → object storage (central); Compactor downsamples to 5m blocks (>40h old) and 1h blocks (>10d old); retention declared per-resolution via `--retention.resolution-{raw,5m,1h}` [thanos]. VictoriaMetrics: `-retentionPeriod` + `-downsampling.period=30d:5m` multi-level, plus enterprise per-label `-retentionFilter` [victoriametrics]. InfluxDB retention policies + continuous queries [model-knowledge — not in retrieved corpus].
- **Lakehouse / log ILM.** Iceberg `expire_snapshots` + tag-based S3 lifecycle integration (`s3.delete.tags`, `s3.write.tags`) so physical tiering is delegated to S3 Lifecycle [iceberg/aws]. Delta `VACUUM`/`OPTIMIZE` [model-knowledge]. Kafka tiered storage (KIP-405, GA'd ~Kafka 4.1): per-topic `remote.storage.enable` with `local.retention.ms/bytes` vs `retention.ms/bytes` controlling local→remote offload [kafka].

**This strongly validates the topological mapping in the hypothesis.** RocksDB-hot-at-edge → warm-at-hub → Iceberg-cold-at-central is the same shape as Prometheus-local → Thanos-hub → object-store, and as CDN edge → shield → origin. It is well-trodden.

### 2.3 Verdict on AUTOMATIC vs POLICY-DECLARED — the correction

The hypothesis's stated default ("AUTOMATIC … as the DEFAULT, policy override") is **inverted relative to mature practice.** The consensus the research repeatedly states is **HYBRID = declarative envelope + automatic optimization inside it**, and where the two diverge, the *declarative* layer is the safety floor:

| System | Automatic mechanism | Declarative envelope (the floor) |
|--------|---------------------|----------------------------------|
| AWS S3 | Intelligent-Tiering (access-temperature auto-demotion) | **Lifecycle rules** (age-based transition/expiration, independent of access) |
| Azure Blob | Smart tier (auto among online tiers) | Tier min-durations; archive is policy-only |
| Thanos | Built-in downsampling thresholds (40h, 10d) | **Per-resolution retention flags** (and the rule that retention must exceed the downsample window or data is deleted before aggregates exist) |
| Kafka tiered | Background offload local→remote | Per-topic retention + local-retention config |
| Enterprise HSM | Background migrators | Age/size/compliance policies; pin critical data in hot |

The research is explicit: **"purely automatic tiering without policy control is considered risky, especially for regulated data,"** and mature deployments favor systems that offer **both**. CDN tiering is "best characterized as hybrid: topology and TTL policies are declarative, while routing, revalidation scheduling, and request coalescing are automatic within those policy boundaries."

For Prism — a **residency-first, regulated-data** (MSSP/OT) system — this argues that:

- The **declarative per-collector policy is the default and the floor** (residency, max-tier, retention duration, what may transit).
- **Automatic temperature/age demotion operates only inside that declared envelope** (e.g., auto-demote hot→cold once a record ages past the detection window, but never auto-promote raw across a residency boundary).
- This is a *re-emphasis*, not a rejection: the hypothesis's "automatic default + policy override" and the research's "policy envelope + automatic-within" converge on the same machinery; they differ on which layer is authoritative when they disagree. For Prism the **policy layer must win** (residency is a hard invariant, not an optimization).

> **VERDICT Q1: VALIDATE the topological tiering; CORRECT the default.** Tier-along-the-chain is sound prior art. But "automatic by default" should be "**declarative policy envelope is authoritative; automatic temperature/age demotion optimizes within it.**" For a residency-first system the automatic layer must never cross a declared residency or tier boundary.

### 2.4 Failure modes the hypothesis must design against (from CDN/HSM/ICN prior art)

1. **Centralized incoherence.** In CDN hierarchies a stale shield/parent serving stale content to all child edges is *worse* than per-edge staleness — it poisons the whole subtree [varnish]. For Prism, a stale hub-tier materialization could mask fresh edge data from every query routed through it. **Mitigation prior art:** targeted purge/cache-tags, stale-while-revalidate with bounded windows, and request-collapsing at the hub.
2. **Cache stampede / thundering herd at the hub.** When a popular object's TTL expires, many edges revalidate at once and overwhelm the parent/origin [oneuptime; cloudflare]. **Mitigations:** Cloudflare's *probabilistic early revalidation* (`p(t)=e^{-λ(expiry−t)}`, revalidate a fraction early to flatten the spike); Varnish *request collapsing* (coalesce duplicate origin fetches); distributed-lock single-flight (`SET … EX … NX`). For Prism, a hub re-materializing an expired pre-aggregate must single-flight, not let every descendant trigger a parallel recompute.
3. **Double-caching.** The same object/result resident at edge + hub + central wastes space; invalidation must propagate to all copies or some tier serves stale [varnish; squid]. Prism's event_time-TTL'd, demand-driven result sets make this *harder* than URL objects because the "key" is a query+window, not a stable URL.
4. **Eviction races.** An edge under memory pressure (recall Prism's ~200MB hot budget, §3.3) evicts before the hub, so an edge miss refills from a hub copy that may have a different freshness — producing inconsistent effective TTL across tiers [cdn order-statistics discussion]. **Mitigation:** make TTL `event_time`-anchored (already Prism's design, §3.3) so freshness is data-intrinsic, not insertion-wall-clock-relative, which removes one source of cross-tier skew.
5. **Coherence of *transformed* representations.** Thanos's hard rule — *never delete raw before the downsampled aggregate exists* — generalizes: Prism must never expire an edge-hot record before its hub/cold materialization (if any) is durable, or a query loses coverage silently (a SOUL.md #4 partial-failure-swallow risk).
6. **Information-centric / en-route caching (NDN/CCN)** pushes automatic opportunistic caching to its limit — *every router caches* — and is the cautionary extreme: maximal automatic caching maximizes coherence and eviction-race problems [ndn, partially model-knowledge for specifics]. It is the argument *against* "cache everything automatically at every hop" and *for* declared placement.

---

## 3. Q2 — What does an UPSTREAM-REPLICATION POLICY LANGUAGE look like?

### 3.1 Hypothesis under test
> The policy = a declarative rule set of `{selector → reduction/normalization (project/aggregate/sample/redact) → retention (TTL/RETAIN) → destination (which parent tier) → residency constraint (raw-stays vs metadata-only-up)}`.

### 3.2 Prior-art evidence — the primitive set is real and recurring

The deep-research synthesis surveyed Cribl, OpenTelemetry Collector, rsyslog/syslog-ng, S3/Iceberg, Kafka MM2 + tiered storage, Prometheus, OPA/Rego, AWS guardrails, KubeEdge, MQTT bridges, and extracted a converging abstraction that **directly validates four of the five hypothesized primitives**:

| Primitive | Prior-art instantiations |
|-----------|--------------------------|
| **Selector** (which records/fields) | Cribl Routes + Eval filters + pre-processing pipelines (multi-level selection on raw + parsed fields) [cribl]; OTel filter processor + Routing Connector OTTL conditions on resource/span attributes [otel]; rsyslog `facility.severity` + property filters (`:msg, contains`, `:programname, isequal`) + RainerScript `if` [rsyslog]; Prometheus `write_relabel_configs` `source_labels`+`regex` [prometheus]; Kafka MM2 `topics`/`topics.regex`/`topics.blacklist` [kafka]; S3 lifecycle prefix/tag scoping [aws] |
| **Reduction / normalization** (project/aggregate/sample/redact) | Cribl Eval/Parser (keep/remove fields), Drop (event-level), Suppress (key-based dedup), plus Redact/Sample/Aggregate [cribl, last three model-knowledge — not in retrieved snippets]; OTel attributes processor `insert/update/delete/hash/redact`, OTTL `set/delete/keep_keys`, tail-sampling policies [otel/splunk]; Prometheus `drop`/`labeldrop` + recording rules (pre-aggregation `record:`/`expr:`) [prometheus]; rsyslog templates (format normalization) [rsyslog] |
| **Retention / lifecycle** (TTL/RETAIN) | S3 Lifecycle transition + expiration actions [aws]; Kafka tiered `local.retention.*` vs `retention.*` [kafka]; Iceberg `expire_snapshots` + tag-based lifecycle [iceberg]; Thanos/VictoriaMetrics retention flags [thanos/vm] |
| **Destination / routing** (which parent tier) | Cribl Routes→Pipelines→destinations [cribl]; OTel service pipelines + Routing Connector to named pipelines [otel]; rsyslog actions (`@@host:port`) [rsyslog]; MQTT bridge `out`/`in`/`both` topic direction [mqtt]; Kafka MM2 replication.policy [kafka] |

### 3.3 Verdict on the residency primitive — VALIDATED-AS-NEEDED, but CHALLENGED on prior art

The fifth primitive — **residency (raw-stays vs metadata-only-up)** — is where the research delivers its sharpest finding:

> **"most [systems] treat residency as a storage-or-region property rather than a first-class constraint intertwined with transformation and routing"** … **"residency is an emergent property of routing and destination choice, not an explicit policy dimension."**

Concretely:
- **Cribl** can enforce "raw must not leave region X" only *indirectly* — by dropping/redacting fields before a destination that happens to be cross-region. The residency *intent* is not captured in the schema, which "can make compliance reasoning harder."
- **OTel** has no residency DSL; you encode it manually via OTTL inspecting `cloud.region`/`data_classification` and routing accordingly — emergent, not standardized.
- **Prometheus** residency is de-facto: drop identifiers before remote_write. No classification DSL.
- **S3/Kafka** express residency as bucket-region + replication config, not per-field.
- **AWS SCPs / OPA/Rego** express region guardrails at the resource/service level (deny resource creation outside approved regions), not as per-field `classification → egress` rules. A Rego `deny` for "EU personal data must stay in EU" is hand-written conditional logic, not a built-in primitive [opa/aws/teradata]. **[INCONCLUSIVE]** the research could not find a widely-adopted standardized `data-classification → egress-constraint` DSL anywhere.

> **VERDICT Q2: VALIDATE the 4-primitive core; the residency primitive is NECESSARY but is NOT well-served prior art — it is the gap Prism would be filling.** Prism making residency a **first-class, per-field, transform-intertwined constraint** (`raw-stays` vs `metadata-only-up`, evaluated *before* any upstream destination is selected) would be *ahead* of the surveyed tools, consistent with vision §3.2 #6 and §2.3 sovereignty.

### 3.4 Three missing primitives the survey adds (the hypothesis is incomplete)

1. **Store-and-forward / buffering policy.** rsyslog disk-assisted queues (`$ActionQueueType LinkedList` + `$ActionQueueFileName` + `ActionResumeRetryCount -1` + `QueueSaveOnShutdown on`) and syslog-ng output buffers express *what happens to upstream-bound data when the parent is unreachable* — drop vs memory-buffer vs disk-spool [rsyslog/syslog-ng]. Kafka tiered local-vs-remote is a buffering policy too. **This is a first-class primitive Prism already needs** (§3.2 store-and-forward) and the policy language must express it, not leave it implicit.
2. **QoS / durability level.** MQTT bridge QoS + persistence; rsyslog retry/resume semantics. *How hard does this hop try to deliver upward?* maps directly onto Prism's `skip_unavailable` (§3.6) and best-effort-vs-required posture.
3. **Transform ORDERING.** Cribl explicitly: function order matters for correctness and efficiency; "filter early, transform late." The classic correctness trap is **redact-vs-forward ordering** — if you route/forward before you redact, raw data has already crossed the boundary. For a residency-first system this is not cosmetic: **residency enforcement and redaction MUST be ordered before any upstream destination selection**, or the invariant is violated. The policy schema must make ordering explicit and verifiable, not implicit.

### 3.5 What is "commonly gotten wrong" (design hazards to avoid)

- **Residency as an afterthought** (above) — the dominant failure; tools bolt it on via routing rather than expressing intent.
- **No per-field control** — many tools (rsyslog, Kafka MM2) operate at record/topic granularity; per-field redaction (OTel attributes processor, Prometheus `labeldrop`, Cribl Parser keep/remove) is the exception, not the norm. Prism's OCSF-normalized + native-schema model *can* do per-field, and the policy must expose it.
- **Reduction-vs-raw-forward ambiguity** — is the policy *reducing then forwarding the reduction*, or *forwarding raw and reducing centrally*? Conflating these is a residency bug. The Prism thesis is unambiguous: **reduce/normalize at the locus, forward only the reduction upward** — the policy language must make raw-forward *impossible to express across a residency boundary*, not merely discouraged.
- **Ordering bugs** (above) — transform/route order determines whether the invariant holds.

---

## 4. Q3 — DEADLINE BUDGETS: do we need a MANDATORY INTERMEDIATE CACHE below N hops?

### 4.1 Hypothesis under test
> A fixed "mandatory cache below N hops" rule is too rigid; better is a **DEADLINE-BUDGET-AWARE PLANNER** that, given per-hop latency + remaining budget, decides whether a descent can complete in time and either (a) uses/inserts an intermediate materialized cache/pre-aggregate, (b) returns partial results with coverage metadata (§3.6), or (c) hedges.

### 4.2 Verdict on the FIXED RULE — CHALLENGED (strongly unsupported)

The deep-research synthesis is unusually direct:

> **"The prior art thus supports adaptive, latency-aware mechanisms in pieces, but does not substantiate a simple fixed heuristic such as 'mandatory intermediate cache below N hops' as a widely adopted design."**

No mainstream distributed query engine (Trino/Presto, Drill, Spark, Dremel/BigQuery) chooses serving-tree depth or cache placement from a fixed hop count. Dremel's and Drill's tree depth is determined by infrastructure topology and data distribution, **not** by a latency rule [dremel/drill]. **The fixed-N-hops rule is rejected by the evidence.** The hypothesis's instinct that it is "too rigid" is correct.

### 4.3 Verdict on the ADAPTIVE BUDGET-AWARE PLANNER — VALIDATED IN DIRECTION, but mostly GREENFIELD

The building blocks the planner would need all exist — **but separately, and not integrated into any production query planner:**

- **(plumbing) gRPC deadline propagation.** gRPC converts an absolute deadline → per-hop timeout by subtracting elapsed time at each hop (clock-skew-safe), and fails late calls with `DEADLINE_EXCEEDED`; some languages auto-propagate the incoming deadline to outgoing subcalls [grpc]. **This is exactly the per-hop deadline-decrement mechanism Prism §3.2 needs — available off the shelf.** But gRPC "does not prescribe any strategy for how application-level query planners should react" — the *decision logic is Prism's to build.*
- **(coarse enforcement) engine timeouts.** Trino remote-task timeout + max-stages; Drill `exec.queue.timeout_millis` (5min default); Spark driver timeout; BigQuery 6h × 3 retries [trino/drill/spark/bigquery]. All **global or per-task, NOT per-hop budgets** — they kill overruns, they don't guide descent decisions.
- **(why materialize at all) tail amplification.** Dean & Barroso, *The Tail at Scale* (CACM 2013): in fan-out, overall latency ≈ the slowest leaf; `P(≥1 slow) ≈ n·p`, and depth `d` × fan-out `k` ⇒ `~k^d` leaves, so **deeper+wider trees blow the 99th percentile** [tailatscale]. Tail-tolerance toolkit: **hedged requests** (duplicate to a replica after a delay), **tied requests** (resource-aware dedup), **micro-partitioning**, **latency-induced probation** (avoid consistently-slow nodes), **canary requests**, **probe-least-loaded-queue-then-route**. *This is the real argument for intermediate materialization — fan-out width × tail risk, not raw hop count.*
- **(how to reduce volume up the tree) multi-level pre-aggregation serving trees.** Dremel/BigQuery leaf-scan → intermediate partial-aggregate → root-assemble; intermediate aggregation cuts records and bytes forwarded upward, lowering latency [dremel]. Drill Foreman major/minor fragments [drill]. **Directly analogous to Prism normalizing+reducing at the locus and forwarding reductions** — and a hub pre-aggregate is the natural place to *cache* a partial result for reuse (though Dremel does not document caching it).
- **(cache placement theory) materialized-view selection + semantic caching.** Quass et al. aggregate-query rewrite + summary tables [quass]; "A Survey of View Selection Methods" (SIGMOD Record) — deterministic/randomized/hybrid/constraint-programming selection, **cost-driven, NOT deadline-driven** [viewselection]. Semantic caching with Bayesian prefetch for real-time location-dependent queries is *implicitly* latency-aware [semanticcache]. **None place caches by per-query deadline in a hierarchical tree.**
- **(deadline decomposition — the closest match) workflow + sensor-net scheduling.** Zhu et al. "Dynamic Scheduling of Deadline-Aware Workflows" performs **deadline decomposition** — split a global deadline into per-job sub-deadlines by dependency + execution-time estimate [zhu]. "Deadline constrained scheduling for data aggregation in unreliable sensor networks" maximizes info reaching the sink before a deadline over an aggregation tree under unreliable links [sensornet]. **These are the strongest prior art for per-hop budget allocation — but in workflow/sensor domains, not SQL query trees.**
- **(partial-on-deadline) CAQP + Elasticsearch CCS.** Constrained Approximate Query Processing returns answers with **bounded error AND bounded time** — treats response time as a first-class constraint [caqp]. Elasticsearch/OpenSearch return partial results + `timed_out` flag + `skip_unavailable` for cross-cluster search [elastic — partly community-knowledge in the corpus, **[INCONCLUSIVE]** on exact coverage-metadata structure]. **This validates Prism §3.6 / BC-2.01.010 partial-result-with-coverage directly** (consistent with the prior `federated-search-architecture-2026-06-24.md` finding).
- **(runtime adaptivity precedent) Spark AQE.** Adapts join strategy / partition coalescing / skew handling from runtime stats — **but driven by performance, NOT client deadlines** [spark-aqe]. Proves runtime-adaptive replanning is feasible; does not do budget-aware planning.

> **VERDICT Q3: CHALLENGE the fixed N-hops rule (unsupported by prior art). VALIDATE the budget-aware planner's DIRECTION — but flag it as a research-grade assembly, not an off-the-shelf pattern.** The trigger for inserting/using an intermediate materialization should be **(remaining budget) vs (estimated subtree latency including tail risk), weighted by fan-out width** — Dean-&-Barroso logic — not depth. The (a)/(b)/(c) options are all individually grounded: (a) pre-aggregate = Dremel serving-tree + view-selection; (b) partial+coverage = CAQP + Elastic CCS + §3.6; (c) hedge = Tail-at-Scale. Integrating them into one planner is the open research.

### 4.4 A production-grade-v1-vs-full-planner framing (for the discussion, not decided)

Because the full budget-aware planner is greenfield, there is a legitimate **feature-ordering** question (NOT a quality compromise — Canonical Principle Rule 2): a v1 could be **"per-hop gRPC deadline decrement + partial-results-on-deadline with coverage metadata (§3.6) + opportunistic hub pre-aggregate when a popular subtree repeatedly times out (latency-induced-probation-style),"** deferring the *cost-model-driven adaptive cache-placement planner* to a later, fully-specified cycle. Both v1 and the eventual planner must be production-grade on the cycle they ship; the planner is a whole *feature* that may be ordered later, not a *shortcut* within v1. This is a human decision (see Open Questions), surfaced not decided.

---

## 5. Cross-Cutting Composition — how Q1 + Q2 + Q3 form ONE chain-aware model

The three questions are not independent; they compose into a single chain-aware cache/replication/planning model with **residency as the binding invariant**:

1. **Q2 policy DECIDES Q1 placement.** A per-collector / per-edge declarative rule `{select → reduce/normalize → retain(duration) → destination-tier → residency}` is what determines *which records materialize at which tier* (Q1's hot-edge / warm-hub / cold-central). The retention *duration* in the Q2 rule is the routing key into Q1's tier (short → RocksDB hot; long/`RETAIN` → Iceberg cold — exactly §3.3's Retention Policy Engine, generalized to "which node's tier"). **Q1 is the storage projection of Q2's policy.**
2. **Q1 materializations BECOME Q3 descent shortcuts.** The hub-tier pre-aggregate that Q1 places (under Q2 policy) is precisely the intermediate materialized cache that Q3's planner uses to avoid descending into a deep subtree that can't meet the budget. A Dremel-style hub partial-aggregate that is *also cached* (the extension Dremel doesn't document) is the join point of Q1 and Q3.
3. **Q3 budget DECIDES whether to use Q1 cache, descend, or return partial.** Given remaining budget + per-hop latency + fan-out tail risk, the coordinator either (a) reads the Q1 hub materialization, (b) returns §3.6 partial+coverage, or (c) hedges. The Q1 cache's *freshness* (event_time TTL) feeds the coverage metadata — a stale-but-within-window hub aggregate is valid coverage; an expired one is a gap to report, never to silently swallow.
4. **Residency constrains ALL THREE simultaneously.** Q1: automatic demotion may never move raw across a residency boundary (only normalized/sanitized up). Q2: the residency primitive is evaluated *before* destination selection and *before* any upstream forward (transform-ordering hazard, §3.5). Q3: a hub materialization used as a descent shortcut must itself be residency-clean — the planner cannot satisfy a deadline by reading a cache that holds raw data that should never have transited. **Residency is the invariant that makes the optimization safe.**
5. **The chain's loop-prevention + per-hop auth (§3.2) underpin coherence.** Seen-request-ID dedup (§3.2) is also a *double-execution/double-caching* guard at the planner level; per-hop mutual auth is what lets a parent trust a child's reduced+residency-clean materialization without re-validating raw it never sees.

**One-sentence model:** *A declarative per-locus replication policy (Q2) projects records into topology-aligned cache tiers (Q1) under a hard residency invariant, and a deadline-budget-aware coordinator (Q3) reads those tiers as descent shortcuts or returns partial+coverage — never crossing a residency boundary and never silently dropping coverage.*

---

## 6. Failure Modes to Avoid (consolidated)

| # | Failure mode | Source domain | Prism-specific bite |
|---|--------------|---------------|---------------------|
| F1 | **Centralized incoherence** — stale hub poisons whole subtree | CDN shield [varnish] | A stale hub pre-aggregate masks fresh edge data from every query routed through it |
| F2 | **Cache stampede at the hub** | CDN/Redis [cloudflare/oneuptime] | All descendants re-trigger a hub re-materialization in parallel; must single-flight |
| F3 | **Double-caching** — same result at edge+hub+central | CDN/Squid | Harder for Prism: key is query+window, not stable URL; wasted hot budget (~200MB) |
| F4 | **Eviction races** — edge evicts before hub, inconsistent effective TTL | CDN order-statistics | Mitigated by `event_time`-anchored TTL (§3.3), not insertion-wall-clock |
| F5 | **Delete-raw-before-aggregate-exists** | Thanos retention rule | Expiring an edge-hot record before its hub/cold materialization is durable = silent coverage loss (SOUL.md #4) |
| F6 | **Residency-as-afterthought** | Cribl/OTel/Prometheus all do it | Routing-derived residency is fragile; must be first-class + ordered-before-forward |
| F7 | **Redact/forward ordering bug** | Cribl "order matters" | If forward precedes redact, raw already crossed the boundary — residency invariant broken |
| F8 | **Reduction-vs-raw-forward ambiguity** | survey | Policy must make raw-forward across a residency boundary *inexpressible*, not just discouraged |
| F9 | **Fixed-depth cache rule** | Q3 verdict | Unsupported; triggers should be budget × tail-risk × fan-out, not hop count |
| F10 | **All-or-nothing on deadline** | Trino/Drill/Spark/BigQuery default | Prism must do partial+coverage (§3.6/CAQP/Elastic CCS), not fail the whole query |
| F11 | **Pure-automatic tiering on regulated data** | HSM/S3 consensus | Risky; declarative policy must be the authoritative floor |
| F12 | **Budget-aware planner over-scoped for v1** | Q3 greenfield finding | Research-grade; a simpler partial+opportunistic-hub-cache v1 may be the right *feature order* (human call) |

---

## 7. OPEN DESIGN QUESTIONS (for human discussion — NOT decided here)

1. **Authority when automatic and declared tiering disagree (Q1).** For a residency-first system, is the declarative policy *always* the authoritative floor (this analysis's reading), with automatic demotion strictly inside it — or are there cases where automatic temperature-tiering may act without a declared rule (e.g., pure operational telemetry with no residency class)?
2. **Is the "warm hub tier" a real third storage regime, or just "hot-tier-at-an-intermediate-node"?** §3.3 currently defines two regimes (RocksDB hot, Iceberg cold). Does a regional hub get its *own* Iceberg cold tier (residency fan-in point per §3.2 #6), a warm RocksDB, both, or neither? Does each hub independently run the §3.3 Retention Policy Engine?
3. **Coherence model for a demand-driven, event_time-TTL'd result cache across hops.** CDN coherence assumes stable URL-keyed objects; Prism's "object" is a query+window result set. What is the cache key, and how do invalidation/purge/stale-while-revalidate translate? Is there *any* cross-hop coherence guarantee, or is each tier independently TTL'd with coverage-metadata reconciliation at query time?
4. **First-class residency primitive — what is its exact granularity and vocabulary?** Per-field? Per-OCSF-attribute? Per-(source-class, schema, schema-version) table (§3.3 multi-schema)? What is the classification vocabulary (`raw` / `normalized` / `metadata-only` / region tags)? How is it verified at the transform-ordering boundary (F7)?
5. **Where is the upstream-replication policy authored and enforced — collector TOML, satellite config, or a new policy artifact?** The prior pass left "where does normalization run (edge vs central)" open; this pass sharpens it to: *who owns the `{select→reduce→retain→destination→residency}` rule, and is it per-collector, per-edge, or per-chain?*
6. **Does Prism build the full deadline-budget-aware planner, or ship a v1 of "per-hop gRPC deadline + partial-on-deadline + opportunistic hub pre-aggregate" and order the planner later?** (Feature-ordering decision per Canonical Principle Rule 2 — a human call, not a quality compromise.)
7. **What triggers inserting/using an intermediate materialization (Q3)?** This analysis argues for *budget × tail-risk × fan-out width* (Dean & Barroso), not hop count. But what are the latency estimates' source (per-hop heartbeat RTT from §3.2? historical stats? probes?), and is latency-induced probation acceptable given dial-home/intermittent edges where "slow" may mean "store-and-forward buffering," not "overloaded"?
8. **Hedging vs dial-home topology.** Hedged/tied requests assume replica choice. Prism's tree is residency-partitioned (a leaf owns its layer's sources uniquely, §3.2 #1) — there may be *no replica* to hedge to. Does hedging even apply, or is partial+coverage the only deadline escape below a single-path subtree?
9. **Pre-aggregate semantics for security detection.** Dremel pre-aggregates are SUM/COUNT/histogram. Security correlation (detection windows, §3.3 DI-029) may need *event-level* data, not aggregates — does an intermediate materialization hold reduced events or true aggregates, and does that interact with the streaming-correlation-state open question from the prior pass (§6/§8 there)?
10. **Coverage-metadata schema unification.** §3.6/BC-2.01.010 partial-result metadata, CAQP error+time bounds, and Elastic CCS `timed_out`/skipped-cluster lists are three coverage vocabularies. Is there one Prism coverage-metadata schema that expresses {which subtrees contributed, which timed out, which tier served the data, freshness of each, residency-clean assertion}?
11. **Stampede/single-flight ownership.** When a hub pre-aggregate expires and N descendants want it, who single-flights the recompute — the hub, the coordinator, or a distributed lock? How does this interact with §3.2 seen-request-ID loop prevention?

---

## 8. Relation to Prism's Ephemeral / Federated / Residency-First Thesis

- **Ephemeral survives.** Nothing here adds "store everything." Q1 tiers are still demand-driven (detection-window TTL, explicit `RETAIN`, §3.3) — now *placed along the topology*. Q3 still prefers partial+coverage over latching state. The chain-aware model is a *placement* generalization of the existing §3.3 Retention Policy Engine, not a new store.
- **Federated survives and is reinforced.** Dremel multi-level pre-aggregation is the canonical federated serving tree; Prism's reduce-at-locus + forward-reductions is the same shape. gRPC deadline propagation is the canonical federated-deadline plumbing.
- **Residency-first is the standout.** The Q2 finding — that *every surveyed tool treats residency as an emergent afterthought* — means Prism's residency-first, per-field, transform-ordered, raw-never-crosses-up policy is **ahead of the prior art**, not catching up to it. This is the strongest positive signal in the pass and is consistent with vision §3.2 #6 and §2.3.
- **The honest cost.** The budget-aware planner (Q3) is research-grade assembly; cross-hop coherence of a result cache (Q1/F1–F5) is genuinely harder than CDN object coherence; and first-class per-field residency (Q2) is net-new policy machinery. None of these is a reason not to do it — they are where the discussion's hardest, most valuable decisions live.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | (Q1) Hierarchical/multi-tier caching + tiered storage mapped to edge–hub–central topology: CDN parent/child + ICP/HTCP, HSM, S3 Intelligent-Tiering vs Lifecycle, Azure tiers/Smart, BigQuery, Snowflake, Prometheus/Thanos, VictoriaMetrics, InfluxDB, Iceberg/Delta ILM, Kafka tiered storage (KIP-405), NDN/CCN; auto-vs-declared verdict + coherence/stampede/double-caching/eviction failure modes. (Q2) Declarative upstream-replication/data-residency policy languages: Cribl, OTel Collector/OTTL, rsyslog/syslog-ng, S3/Iceberg ILM, Kafka MM2 + tiered, Prometheus remote-write + recording rules, OPA/Rego, AWS guardrails, KubeEdge, MQTT — primitive-set validation + residency-gap + missing primitives. (Q3) Deadline/latency-budget propagation in query trees + intermediate cache/materialized-view placement: gRPC deadlines, Trino/Drill/Spark timeouts, Tail at Scale, Dremel/Drill serving trees, materialized-view selection, semantic caching, deadline-decomposition (workflow + sensor-net), CAQP, Elastic CCS partial results, Spark AQE — fixed-N-hops verdict + budget-aware-planner verdict. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — (subject is architecture/protocol/vendor-positioning research, not library-API docs) |
| Tavily tavily_search | 0 | — |
| Tavily tavily_research | 0 | — |
| Tavily tavily_extract | 0 | — |
| Tavily tavily_crawl / tavily_map | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | ~4 areas | (a) NDN/CCN en-route-caching specifics and the synthesis/conclusion sections of the Q1 pass that fell beyond the readable portion of the saved tool-result file [model-knowledge, flagged]; (b) InfluxDB RP/CQ and Delta `VACUUM`/`OPTIMIZE` (research itself flagged these as not-in-retrieved-corpus); (c) Cribl Redact/Sample/Aggregate exact schemas (flagged not-in-snippets); (d) Elastic/OpenSearch `skip_unavailable` exact coverage-metadata structure (flagged community-knowledge / **[INCONCLUSIVE]** by the research). |

**Total MCP tool calls:** 3 (all `mcp__perplexity__perplexity_research`, `sonar-deep-research`, `reasoning_effort: high`, `strip_thinking: true`).

**Read-coverage honesty note (per file-handling protocol):** The three deep-research results returned 95k / 91k / 86k characters and were saved to tool-result files as single unbreakable lines. I read the full first ~64k chars of each (the complete conceptual model, per-domain findings, and the explicit verdicts for all three questions — including, for Q3, the verbatim "does not substantiate … 'mandatory intermediate cache below N hops'" finding in the intro). The trailing ~22–31k chars of each file (NDN deep-dive + cross-domain synthesis for Q1; KubeEdge/MQTT + commonly-gotten-wrong synthesis for Q2; AQE-onward + final conclusion for Q3) could **not** be paginated by the Read tool (single-line files) and grep returns long single-line matches as omitted. I confirmed via keyword grep that those sections exist and their headline conclusions (hybrid-tiering consensus; residency-as-afterthought gap; fixed-rule-rejected) are already fully stated in the readable portion and are reinforced, not contradicted, by the section headings in the tail. Items genuinely dependent on the unread tail are flagged **[model-knowledge]** in the Research Methods training-data row and inline.

**Training-data reliance:** **low** — every architectural, protocol, and product claim is grounded in the retrieved deep-research output, which cites CDN vendor docs (Akamai/Cloudflare/Fastly/Varnish/ATS/Squid), cloud storage docs (AWS S3 Intelligent-Tiering/Lifecycle, Azure Blob, BigQuery, Snowflake), time-series docs (Thanos, VictoriaMetrics), table-format docs (Iceberg, Kafka KIP-405), pipeline docs (Cribl, OpenTelemetry Collector/OTTL, rsyslog/syslog-ng, Prometheus), policy systems (OPA/Rego, AWS guardrails), RFC/RPC (gRPC deadlines), and papers (Dean & Barroso *The Tail at Scale* CACM 2013; Dremel CACM; Quass et al. aggregate-query processing; SIGMOD Record view-selection survey; Zhu et al. deadline-aware workflows; deadline-constrained sensor-net aggregation; CAQP). Four areas flagged `[model-knowledge]`/`[INCONCLUSIVE]` inline and in the methods table.

**Deviation note (per agent mandate):** `perplexity_research` (the PRIMARY tool) was used for all three passes — no deviation to justify. Context7/Tavily were not used because the subject is architecture/protocol/vendor-positioning research, not library-API documentation; Perplexity deep-research at high reasoning effort is the correct instrument and returned source-grounded, citation-backed output for each question.
