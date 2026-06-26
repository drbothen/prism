---
document_type: design-decision
status: capture
do_not_execute: true
provenance: "2026-06-26 side-analysis — day-2 ML phasing decision; PROPOSED, gated on brief-reframe sign-off; separate from live factory."
traces_to:
  - matured-vision-day2-requirements.md §15 (On-Demand ML & Behavior Analytics)
  - matured-vision-day2-requirements.md §3.3 (RetentionCache — tiered, RocksDB hot + Iceberg cold)
  - matured-vision-day2-requirements.md §2.4 (honest tradeoffs)
  - matured-vision-day2-requirements.md §14.2 (statistical-anomaly row in correlation table)
  - matured-vision-day2-requirements.md §15.9 (three-ways-to-long-baseline)
---

# Day-2 Design Decision — ML Depth Phasing

## Summary

Section 15 of the matured vision confirms on-demand ML and online/continuous learning as
human-confirmed day-2 features. It identifies two model tiers (§15.7) and a pluggable backend
requirement, but the phasing and implementation boundary decisions are left open. This document
proposes a concrete phased plan so the architect can allocate stories, ADRs, and subsystem
ownership cleanly before the E-ML-* epics are dispatched.

The core constraint is the Prism law (§15.0): ML obeys the same on-demand, demand-cached, cost-
bounded discipline as everything else in Prism. It is NOT an always-on ingestion-time pipeline.
All phases must honor the mandatory time-bound + join-guard NFRs (§5.3, §12.2) and operate within
the configurable memory budget ceiling (DC-004).

---

## Recommendation

Start with Phase 1 (lightweight statistical, on-demand). Defer Phase 2 (online learning) to a
follow-on epic after Phase 1 is converged. Defer Phase 3 (heavier models + pluggable backends) to
after Phase 2. Rationale in each section below.

---

## Phase 1 — Lightweight Statistical (On-Demand)

### What is in scope

Anomaly/behavioral scoring via pure statistical estimators, computed at query time over data
Prism already fetches or has cached. No training loop, no persistent model state, no external
runtime dependency.

**Estimators in scope:**
- Z-score (mean/variance computed over a window)
- EWMA (exponentially weighted moving average for trend and baseline)
- MAD (median absolute deviation — more robust than z-score for skewed security telemetry)
- Robust quantile / t-digest (approximate percentile estimation over the window)
- Rarity / first-seen (set membership against a running sketch — count-min or HLL)
- Peer-group outlier (compare entity against a group aggregate in the same query window)

**Expressed as PrismQL functions** (§15.6): `ANOMALY_SCORE(expr, window)`,
`RARITY(expr, window)`, `FIRST_SEEN(expr, window)`, `BASELINE_DEVIATION(expr, window)`,
`PEER_OUTLIER(expr, group_by, window)`. These are PrismQL scalar/aggregate functions, not a
separate subsystem the analyst sees. A detection that uses `ANOMALY_SCORE` is still just a
`DETECT … WHERE ANOMALY_SCORE(…) > threshold` query.

### Compute model

**Query-time only.** No background computation, no separate process, no scheduler.

The estimators are computed inline during query execution over the window DataFusion has
materialized from the federated pull and/or RetentionCache. They are pure functions of their
inputs: given the same input rows, they return the same score. This puts them squarely in the
Prism pure-core / DataFusion physical-plan layer — formally verifiable in principle, deterministic
for a given row set.

**Baseline source = RetentionCache long-baseline tier (§3.3 Iceberg cold).** A detection
with a behavioral-baseline window larger than the hot RocksDB tier (e.g., `ANOMALY_SCORE` over
30 days) pulls the history from the Iceberg cold tier. A short-window anomaly (e.g., last 15
minutes) pulls from the hot RocksDB tier. The estimator does not distinguish — it receives rows
from DataFusion regardless of tier. For sources with no RetentionCache coverage, an on-demand
scoped federated pull is issued (subject to mandatory time-bound + join-guard NFRs).

**No scheduled baseline refresh in Phase 1.** The optional scheduled sampling (§15.4,
"learn-from-touch AND scheduled sampling") is a Phase 2 concern tied to the online-learning loop.
Phase 1 baselines are pure query-time aggregates over whatever is available.

### Dependencies (gates for Phase 1)

- E-CACHE-DEMAND-001 P1 (RocksDB hot-tier RetentionCache) must be deployed or Phase 1 has no
  short-window baseline data beyond what a single query pulls live.
- E-CACHE-DEMAND-001 cold-tier (Iceberg) is desirable for long-window anomaly detection but
  Phase 1 can degrade gracefully to federated pull if the cold tier is not yet available. Flag
  this as an honest-limit signal (see below).

**ADR dependency:** no new ADR required for Phase 1 alone. The `ANOMALY_SCORE` / `RARITY` /
`FIRST_SEEN` function contracts can be specified in BCs under E-ML-ONDEMAND-001. The statistical
algorithm choices (z-score, EWMA, MAD, t-digest) are implementation decisions, not architectural
decisions requiring an ADR. If Prism later adopts a specific streaming statistics library, that
warrants an ADR; Phase 1 does not need one.

### Honest-limit / explainability controls

Phase 1 must implement insufficient-data signalling from day one. A detection that calls
`ANOMALY_SCORE(x, 30d)` but only has 3 hours of baseline data must return a structured
insufficient-data result rather than a misleading score.

Required controls (in scope for Phase 1):
- **Minimum-sample guard:** every statistical function carries a `min_samples` requirement; if the
  baseline window contains fewer rows than `min_samples`, return a sentinel value (NULL or a
  structured error variant) rather than a spurious score.
- **Baseline-coverage metadata:** every query result that used a statistical estimator MUST include
  a `baseline_coverage` metadata field: how many rows were available, what time range was covered,
  whether the cold tier was used, whether a federated pull was required.
- **Explicit window in the query:** the analyst must specify the time window for the baseline
  (wired to the mandatory-time-bound NFR, §5.3). There is no unbounded background accumulation.

These controls tie directly to the trust-first AI UX principle: an analyst model (S3 embedded
agent or BYO agent) must be able to surface "this anomaly score is based on only 2 hours of
baseline; treat with lower confidence" rather than presenting a black-box score.

### Compute/storage footprint

Phase 1 has no persistent storage beyond what RetentionCache already holds. The estimators are
streamed within DataFusion's query execution — no ML-specific memory budget beyond what the
query's normal row materialization costs. The `ANOMALY_SCORE` aggregate over a window is
O(window_rows) in time and O(1) in memory for streaming estimators (EWMA, MAD, t-digest).

Phase 1 comfortably fits within the configurable memory budget ceiling (DC-004) at the same cost
profile as a comparable GROUP BY aggregation.

---

## Phase 2 — Online / Continuous Learning ("Model is the Memory")

### What is in scope

Incremental/streaming model updates that persist beyond a single query execution. The model
becomes a compact, per-tenant, per-entity-class retention artifact (§15.3) that accumulates
history across every data touch — queries, detection executions, federated pulls.

**Model state storage:** RocksDB / RetentionCache, a new column family (analogous to
`StorageDomain::RetentionCache`). NOT a new datastore. Specifically NOT PostgreSQL — this was
explicitly rejected in §14.3. The model artifact is a small serialized struct (sketch/params),
not a row store.

**Online algorithms (streaming, single-pass):**
- Streaming mean/variance with decay (EWMA/sliding window — extends Phase 1 estimators from
  query-time to continuously-updated state)
- t-digest for robust quantile tracking (persistent across queries)
- Count-min sketch + HyperLogLog for frequency/cardinality (persistent)
- Reservoir sampling (persistent sample of the last N events for distribution-based tests)
- Online isolation forest / half-space trees for anomaly scoring with learned thresholds
- Streaming clustering (k-means with online centroid updates — identifies behavioral archetypes
  across peer groups over time)

**Update triggers:** learn-from-touch (every query execution that touches data covered by an
active model policy updates the relevant model parameters) plus optional scheduled baseline-
refresh sampling pulls (§15.4). The scheduled-sampling path requires integration with
prism-operations scheduler — a gate dependency.

### Model as a third retention tier (§15.3)

The model state tier has distinct semantics from the hot/cold cache tiers:
- Hot (RocksDB CF): seconds to hours/days, exact rows, replayable.
- Cold (Iceberg): days to years, exact rows, replayable.
- Model state: longest horizon, lossy summary, NOT replayable.

**Model state is governed by the same policy discipline as the cache.** A model may only persist
for an entity-class if an active `RETAIN MODEL <entity_type> [WINDOW <duration>]` directive or an
active detection rule scope covers it. No unbounded background model accumulation.

### Honest-limit and drift controls (in scope for Phase 2)

Phase 2 honest-limit controls extend Phase 1 and are non-negotiable:

- **Concept drift / decay:** EWMA/sliding decay on all estimates; a model that has not been
  updated in > TTL_model (configurable per model policy) marks itself stale and returns the
  insufficient-data sentinel rather than an outdated score.
- **Adversarial poisoning resistance:** bounded update rates (a single query execution cannot
  shift a model by more than a configurable delta); robust estimators (MAD/t-digest resist
  outlier poisoning better than mean/variance); anomaly-gated learning (do not update the normal
  baseline with events the current model already scores as anomalous — prevents "boiling-frog"
  attacks).
- **Model versioning / snapshots:** every finding that fired based on a model score MUST attach
  a reference to the model snapshot as of that decision (ties to the finding replay link, §14.5).
  The replay link points at both the time window AND the model state at decision time. This is
  the explainability / audit guarantee for online-learned detections.
- **Coverage gap signalling:** the `baseline_coverage` metadata field from Phase 1 extends to
  report whether the model was updated via live touch, via scheduled sampling, or not at all
  since a configurable horizon.

### State storage design

New `StorageDomain::ModelState` column family in the existing RocksDB instance.

Key schema: `(tenant_id, entity_class, model_type, entity_key)` → serialized model struct
(bincode or protobuf). This mirrors the per-tenant key prefix pattern already used across prism
column families (OrgId-keyed isolation is an invariant of the existing storage layer).

Model artifact size targets: a per-entity EWMA/t-digest bundle is O(kilobytes). A complete per-
tenant behavioral model for all entities of a given class across a 30-day window is expected to
be in the low-MB range — well within DC-004's configurable GB-range budget.

### Dependencies (gates for Phase 2)

- Phase 1 (E-ML-ONDEMAND-001) converged. Phase 2 adds persistence on top of Phase 1's
  query-time estimators; the function contracts and PrismQL primitives must be stable first.
- E-CACHE-DEMAND-001 P1 (RocksDB hot-tier) deployed — the model state CF lives in the same
  RocksDB instance.
- prism-operations scheduler available for optional scheduled-sampling pulls.

**ADR requirement for Phase 2:** one new ADR is required before Phase 2 implementation begins,
covering:
1. Model-as-retention-tier: the `StorageDomain::ModelState` CF design, key schema, size
   governance (per-tenant cap, eviction policy), and policy-driven lifetime.
2. Online-learning update semantics: the update contract (which algorithms, max-delta bounds,
   anomaly-gate rule, EWMA decay constants).
3. Poisoning-resistance controls: bounded update rates, robust estimator selection, anomaly-
   gated learning invariant.
4. Model snapshot/versioning for replay/explainability: snapshot granularity, storage, and the
   finding-replay link protocol (ties to §14.5 ADR).

### Compute/storage footprint

Phase 2 adds:
- `StorageDomain::ModelState` CF in RocksDB. Size-governed by per-tenant model retention
  policies (same discipline as RetentionCache). Expected footprint: tens of MB per tenant for a
  full 30-day behavioral model. The configurable memory budget ceiling (DC-004) accommodates this.
- Online update cost at query time: O(1) additional work per row for streaming estimators.
  Negligible relative to the federated query I/O cost.
- Optional scheduled sampling: adds prism-operations scheduler load (one batch pull per model
  policy per refresh interval). Cost is bounded by the pull's mandatory time-bound and join-guard.

---

## Phase 3 — Heavier Learned Models + Pluggable Backends

### What is in scope

Model types that require more than streaming single-pass computation: batch-trained isolation
forests over large windows, autoencoder-based anomaly scoring, sequence models (LSTM/transformer)
for behavioral path analysis, and streaming clustering with richer centroid representations.

**Pluggable model backend abstraction** (§15.7): a `ModelBackend` trait (mirrors §11.1's
`SecretBackend` trait pattern) with implementations for:
- Built-in first-party backend (on-prem, air-gap-safe, no external dependency): the Phase 1/2
  estimators plus a local inference engine for compact trained models (linfa / candle / burn,
  or a minimal ONNX inference runtime for portable models).
- External-hosted backend: inference via an external API (e.g., a customer's self-hosted MLflow/
  BentoML endpoint, or a cloud inference service). Request/response mediated through the same AI-
  opacity + prompt-injection-hardened path as S3 (AD-017 applies — the backend receives feature
  vectors, never raw credentials or PII-containing OCSF records).

**Feature store considerations:** a Phase 3 heavier model needs precomputed feature vectors.
Phase 1/2 have no feature store dependency (estimators run inline). Phase 3 introduces a feature-
materialization stage: a configurable feature-engineering pipeline that runs over RetentionCache
rows and stores computed feature vectors in a `StorageDomain::FeatureStore` CF or in the Iceberg
cold tier (depending on feature retention window). This is the single largest new architectural
surface in Phase 3.

**Eval / guardrails:** model evaluation gates before a Phase 3 model is promoted to production:
offline evaluation against held-out RetentionCache windows, FP rate threshold, MTTD budget,
and shadow-mode operation before full promotion (mirrors the §14.4 rule editor staged-rollout
pattern).

### Backend abstraction boundary

The `ModelBackend` boundary is defined as: the caller passes a feature vector (schema-typed,
OCSF/native-schema-aware, with no raw credentials embedded) and receives a score + explanation
struct. Everything below that boundary (where inference runs, which model type, what hardware)
is the backend's concern.

Key invariants at the boundary:
- Feature vectors are AI-opaque (no credential material, no raw secrets — AD-017 applies).
- The backend never returns opaque scores without an explanation struct (explainability invariant,
  ties to §15.5 explainability/audit control).
- The backend call is bounded by a `model_inference_timeout` (mirrors `request_timeout_secs`
  discipline from §3.6). A slow or unavailable backend triggers the partial-result/degraded path.
- Built-in backend must be self-sufficient for air-gap deployments. External backend is optional
  and deployment-gated (same pattern as S3 server-hosted agent in §11.3.2).

### Dependencies (gates for Phase 3)

- Phase 2 (E-ML-ONLINE-001) converged. Phase 3 heavier models build on Phase 2's model state
  layer and online-learning infrastructure.
- E-CACHE-DEMAND-001 cold tier (Iceberg) should be stable — Phase 3 feature materialization
  uses the cold tier as its primary data source for batch training windows.
- E-CENTRAL-AUTHZ-001 in place — per-tenant model isolation and backend access RBAC require
  the central authZ layer.

**ADR requirement for Phase 3:**
1. Pluggable model backend abstraction (`ModelBackend` trait, feature vector contract,
   explanation struct, AI-opacity invariant, timeout/degraded behavior).
2. Feature store design (`StorageDomain::FeatureStore` vs Iceberg cold; feature-engineering
   pipeline; retention policy for feature vectors).
3. Phase 3 model promotion lifecycle (eval gates, shadow mode, canary, rollback).

### Compute/storage footprint

Phase 3 is where the ML footprint meaningfully grows:
- Batch model training is an explicit offline / background operation (not on the query-hot path).
  It is scheduled, cost-bounded (same mandatory time-bound + join-guard guardrails on the
  training pull), and can be deferred/throttled under memory pressure.
- Feature store adds a new CF or Iceberg partition — size governed by feature retention policy.
- Inference at query time via the built-in backend is bounded by `model_inference_timeout`. For
  external backends, it is also network-bounded. Both paths degrade gracefully (partial result
  + missing-score metadata if the inference times out).

Phase 3 is the first phase where deployment sizing (DC-004 configurable memory ceiling) becomes
a material factor for the ML subsystem. The recommendation is that the Phase 3 ADR include a
reference sizing guide: minimum footprint (built-in, laptop/small-server) and recommended
footprint (built-in + external, central MSSP deployment).

---

## Dependency Graph (summary)

```
E-CACHE-DEMAND-001 P1 (RocksDB hot tier)
  |
  +---> Phase 1 (E-ML-ONDEMAND-001): lightweight statistical, on-demand, query-time only
  |       Deliverables: ANOMALY_SCORE / RARITY / FIRST_SEEN / BASELINE_DEVIATION / PEER_OUTLIER
  |       PrismQL functions; insufficient-data signalling; baseline_coverage metadata.
  |       No new ADR needed; BCs under E-ML-ONDEMAND-001.
  |
  v
E-CACHE-DEMAND-001 cold tier (Iceberg) + prism-operations scheduler
  |
  +---> Phase 2 (E-ML-ONLINE-001): online learning, model-as-retention-tier, drift/decay,
  |       poisoning-resistance, model snapshots/versioning, scheduled sampling.
  |       New StorageDomain::ModelState CF. Requires one new ADR (model-as-retention-tier +
  |       online-update semantics + poisoning-resistance + snapshot/versioning).
  |
  v
E-CENTRAL-AUTHZ-001 + E-CACHE-DEMAND-001 cold tier stable
  |
  +---> Phase 3 (E-ML-PRIMITIVES-001 extensions + pluggable backends): heavier models,
          feature store, ModelBackend abstraction, eval/guardrails.
          Requires ADRs for backend abstraction, feature store, model promotion lifecycle.
```

---

## Recommendation — Where to Start for v1-of-day-2

**Start Phase 1 in parallel with E-CACHE-DEMAND-001 P1.** Phase 1 has no Phase 2 or Phase 3
dependencies. The PrismQL function contracts for `ANOMALY_SCORE` et al. are stable enough to
specify as BCs now. The statistical algorithms are well-understood and have no significant
implementation risk.

**Gate Phase 2 on Phase 1 convergence + E-CACHE-DEMAND-001 cold tier.** Phase 2 is a material
architectural addition (persistent model state, new StorageDomain CF, update semantics, poisoning
controls). It should not start until Phase 1's PrismQL function contracts are converged and
battle-tested in detection queries. Estimated sequencing: Phase 2 starts 2-3 waves after Phase 1
ships.

**Defer Phase 3 to post-Phase-2.** The pluggable backend abstraction depends on having a stable
Phase 2 model state layer to plug into. The feature store design depends on having the Iceberg
cold tier operating. Phase 3 is the right scope for a later wave when the central deployment
and multi-tenant auth layers are also in place.

---

## Open Decisions for Human

The following items require human input before Phase 1 or Phase 2 stories can be set to
`status: ready`. These are not deferrals — they are genuine scope decisions only the human
can make.

| # | Decision | Impact | Notes |
|---|----------|--------|-------|
| OD-1 | **Phase 1 ship gate: must E-CACHE-DEMAND-001 P1 be merged first, or can Phase 1 degrade gracefully to live-only federated baselines?** | Affects story ordering. If Phase 1 can degrade gracefully (live federated pull as baseline, with honest-limit insufficient-data signal when history is shallow), it can ship concurrently with E-CACHE-DEMAND-001 P1. If the RetentionCache is a hard dependency for any useful anomaly score, Phase 1 must sequence after P1. | Recommendation: allow graceful degradation. Ship Phase 1 concurrently. |
| OD-2 | **Phase 2 poisoning-resistance: is bounded-update-rate sufficient, or is anomaly-gated learning (do not update baseline with anomalous events) also required from day one?** | Anomaly-gated learning is the stronger control but requires the Phase 1 ANOMALY_SCORE function to be live before Phase 2's update loop can use it — a natural ordering dependency. If both are required from Phase 2 day one, it is simply an ordering constraint, not a scope change. | Recommendation: both in scope from Phase 2 day one. The ordering dependency is already captured in the Phase 2 gate dependencies above. |
| OD-3 | **Phase 3 built-in inference engine: ONNX runtime, linfa (pure Rust), or candle/burn (Rust, GPU-capable)?** | This is an ADR-level choice. linfa is pure Rust, no C/C++ dependency, air-gap friendly, limited model types. ONNX Runtime has a C++ core, wider model compatibility, harder for true air-gap. candle/burn offer GPU but increase build complexity. The correct choice depends on the model types the human wants in Phase 3. | Flag for architect at Phase 3 ADR time. Do not block Phase 1 or Phase 2. |
| OD-4 | **Phase 3 scope: is a feature store a Phase 3 deliverable or a follow-on epic?** | The feature store (StorageDomain::FeatureStore or Iceberg partition for feature vectors) may be out of scope for the initial Phase 3 delivery if Phase 3 focuses only on the `ModelBackend` abstraction and the built-in inference engine for compact models (no batch-training, no precomputed features). This would scope Phase 3 more narrowly and reduce its implementation risk. | Recommendation: define Phase 3 as "pluggable backend abstraction + built-in inference for compact trained models (no feature store)." Feature store = follow-on epic E-ML-FEATURE-STORE-001. |
| OD-5 | **Scheduled sampling (Phase 2): who configures it?** Is scheduled baseline-refresh sampling a per-detection-rule config (rule author sets a `baseline_refresh: 1h` alongside the detection window), or a per-tenant admin config (global policy via the central config store, §11.2)? | Affects BC authorship (product-owner) and the config-store schema (data-engineer). | Recommendation: per-detection-rule config initially (simpler, rule authors have the context); tenant-level default as a config-store feature in Phase 2 or Phase 3. |
