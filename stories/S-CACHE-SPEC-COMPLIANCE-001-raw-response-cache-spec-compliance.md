---
document_type: story
story_id: S-CACHE-SPEC-COMPLIANCE-001
title: "Raw Pre-OCSF Response Caching at the Adapter Seam — BC-2.07.003 Spec Compliance (P1-03 Closure)"
# wave: NOT wave-scheduled — post-demo backlog per human P1-03 adjudication 2026-06-10.
# Sequenced AFTER the live-demo objective (T5 Story B → T6 → T8). See depends_on.
wave: post-demo-backlog
epic_id: maintenance
priority: P2
# Status rationale: draft (NOT ready) despite non-empty behavioral_contracts —
# (1) human-directed sequencing: this story must not be dispatched before the
#     live-demo objective completes (P1-03 adjudication, 2026-06-10);
# (2) the cache-seam decision (Option A adapter-seam vs Option B engine-seam,
#     §Seam Decision below) requires an architect ADR before implementation;
# (3) remove-uncertainty must run against the post-demo develop baseline (the
#     QRY-02 wiring on fix/review-2026-06-10-query-core must be merged first —
#     this story redesigns that wiring's value type and read path).
status: draft
version: "1.1"
level: "L4"
producer: story-writer
timestamp: "2026-06-10T00:00:00Z"
created: "2026-06-10"
modified: "2026-06-10T00:00:00Z"
phase: 3
tdd_mode: strict
subsystems: [SS-07, SS-11, SS-01, SS-22]
# Subsystem anchor justifications (per ARCH-INDEX Subsystem Registry):
#   SS-07 (Adapter Pagination & Response Cache) owns this story's PRIMARY scope —
#     the sensor-fetch response cache in prism-query (cache.rs, cache_key.rs) whose
#     stored value type and read-path semantics this story changes. BC-2.07.003/005
#     are SS-07 contracts.
#   SS-11 (Query Execution) owns the materialization fan-out read path
#     (materialization.rs cache-check-before-fan-out / populate-after wiring, QRY-02)
#     that must re-normalize raw cached records on read.
#   SS-01 (Sensor Adapters) owns the adapter-boundary normalization convention
#     ("raw API responses are normalized at the adapter boundary") that this story
#     re-sequences relative to caching: under spec compliance, the cache sits
#     BEFORE normalization, so the adapter boundary's raw/normalized split moves.
#   SS-22 (Binary Entrypoint) owns prism-bin/src/spec_driven_adapter.rs — the
#     SpecDrivenSensorAdapter::fetch() seam where PipelineResult (raw JSON) exists
#     before pipeline_result_to_record_batch; Option A places the cache here.
#   NOTE: the FINAL subsystem set narrows once the architect seam ADR (Phase 0
#   task) picks Option A or Option B; both options touch SS-07 + SS-11; Option A
#   emphasizes SS-22/SS-01, Option B may touch the SensorAdapter trait (SS-01).
target_module: prism-query
crates_touched: [prism-query, prism-bin]
# prism-sensors is touched ONLY under seam Option B (SensorAdapter trait surface
# change to expose raw records + normalization handle); confirmed at seam ADR time.
behavioral_contracts: [BC-2.07.003, BC-2.07.005]
verification_properties: [VP-025]
# VP-025 (cache key derivation: deterministic, kani) — anchor unchanged; the key
# derivation surface survives the redesign byte-identically (AC-011/AC-012). This
# story is the post-redesign regression vehicle for the VP-025 proof inputs.
depends_on:
  - S-DEMO-DTU-LIVE-SCENARIO-001-B
  - S-DEMO-MULTI-TENANT-DTU-001
  - S-DEMO-004
  # Dependency anchors — HUMAN-DIRECTED SEQUENCING, not build-order:
  #   These three stories constitute the live-demo objective (multi-client SOC
  #   demo: T5 Story B → T6 → T8). The human P1-03 adjudication (2026-06-10)
  #   explicitly sequences this compliance story AFTER that objective. There is
  #   no compile-time dependency on any of the three; the dependency expresses
  #   the human's scheduling decision so the wave scheduler cannot pull this
  #   story ahead of the demo.
  # ADDITIONAL MERGE PRECONDITION (build-order, branch not story):
  #   fix/review-2026-06-10-query-core (.worktrees/FIX-REVIEW-QRY-2026-06-10)
  #   must merge to develop first. It carries the QRY-02 cache wiring plus the
  #   P1-01 (fetch.limit in cache key, BC-2.07.005 v4.4) and P1-05 (forced-refresh
  #   invalidation, BC-2.07.003 v4.5 D3) closures that this story preserves, and
  #   the in-flight hot-reload→cache-flush interim mitigation this story supersedes.
blocks: []
points: 8
# Points justification:
#   1. Architect seam ADR (Option A vs B) + remove-uncertainty re-baseline: 1 pt
#   2. Cache value-type migration (production instantiation Vec<RecordBatch> →
#      raw pre-OCSF records; CacheValue impl + byte accounting + doc sweep): 2 pts
#   3. Read-path re-normalization (cache hit → normalize raw records with CURRENT
#      sensor-spec snapshot; seam plumbing per ADR option): 2.5 pts
#   4. Red Gate test suite (~12 tests incl. exact-response invariant, hot-reload
#      effectiveness, D3/EC-07-033/034 preservation at new seam): 1.5 pts
#   5. Regression adaptation of existing BC-2.07.003/004/005/006 suites to the
#      migrated value type + key-vector re-verification (EC-07-040..044): 1 pt
estimated_days: 4
risk: HIGH
# Risk justification:
#   The cross-query response cache is on the hot query path; changing its value
#   type and moving normalization to the read side has correctness risk (stale-spec
#   vs current-spec normalization), performance risk (re-normalization CPU cost on
#   every hit instead of once at fetch), and surface risk (Option B changes the
#   SensorAdapter::fetch boundary). The D1 (fetch.limit) and D3 (forced-refresh
#   invalidation) semantics just landed and must survive byte-identically.
acceptance_criteria_count: 12
red_gate_tests: 12
estimated_passes: "3-5 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Seam decision is an EXPLICIT Phase 0 architect ADR gate (human-directed); implementation must not begin until the ADR is ACCEPTED. Do not default to either option."
  - "BC-2.07.005 v4.4 fetch.limit semantics (single-binding coherence: the limit hashed is the limit fetched) MUST survive the seam move — re-run EC-07-040..044 canonical vectors against the new seam (AC-011/AC-012)."
  - "BC-2.07.003 v4.5 D3 forced-refresh-failure invalidation (EC-07-033/034) MUST survive — partial responses are never cached regardless of value type (AC-007/AC-008)."
  - "Re-normalization on read must use the CURRENT sensor-spec snapshot (ArcSwap::load per AD-007), not a snapshot captured at cache-populate time — otherwise the hot-reload benefit (the point of normalize-on-read) is lost."
  - "Re-normalization failure on a cache hit is handled per the BC-2.07.003 error table: log warning, proceed as cache miss, query the sensor API directly. Never panic; never serve a half-normalized batch."
  - "The per-query in-query cache (BC-2.11.005, MaterializationContext.in_query_cache) stays Vec<RecordBatch> — it is post-normalization by design (self-join reuse within one query). Do NOT conflate it with the cross-query response cache."
  - "Byte-budget accounting (DI-018 / BC-2.07.006, SEC-003 per-entry cap) must be re-derived for the raw-JSON value type — Arrow get_array_memory_size no longer applies to the production instantiation; CR-006 AVG_ROW_SIZE_BYTES estimate or a measured serde_json deep-size estimate per architect ADR."
  - "Performance: add a before/after benchmark for the cache-hit path (normalize-once-at-fetch vs normalize-on-every-hit). If hit-path regression exceeds the R-010 memory/latency envelope, the seam ADR must record the accepted tradeoff explicitly."
traces_to: ["P1-03 human adjudication 2026-06-10 (review-2026-06-10 QRY cycle)", "proposals/cache-envelope-adjudication-2026-06-10.md (P1-03 out-of-scope boundary, now adjudicated)"]
supersedes: []
---

# S-CACHE-SPEC-COMPLIANCE-001: Raw Pre-OCSF Response Caching at the Adapter Seam — BC-2.07.003 Spec Compliance (P1-03 Closure)

Bring the as-built response-cache implementation into compliance with the UNAMENDED
BC-2.07.003 v4.5 raw-response cache model. The human P1-03 adjudication (2026-06-10)
ruled: **the spec is right; the code moves.** BC-2.07.003's normalization language
("pre-OCSF-normalization sensor records", "raw sensor responses", "no transformation
applied before caching") — byte-frozen during the QRY cascade pending this adjudication —
stays as written. The as-built code, which caches post-normalization Arrow
`RecordBatch`es at the materialization fan-out level (a known, human-authorized
deviation documented in `cache.rs`), is redesigned to cache **raw pre-OCSF responses
at the adapter seam, with OCSF normalization applied on read**.

**Sequencing (human-directed):** AFTER the live-demo objective (S-DEMO-DTU-LIVE-SCENARIO-001-B
→ S-DEMO-MULTI-TENANT-DTU-001 → S-DEMO-004) and AFTER fix/review-2026-06-10-query-core
merges to develop.

---

## Authority

BC-2.07.003 is the governing spec for this story. The P1-03 adjudication (2026-06-10) ruled it
UNAMENDED: the cache stores raw pre-OCSF sensor responses; normalization is applied on read.
All ACs derive from BC-2.07.003 §Postconditions and §Invariants. Read it before implementing.

A seam decision ADR (Option A adapter-seam vs Option B engine-seam) is the Phase 0 architect
gate — implementation MUST NOT begin until that ADR is ACCEPTED.

ADR-023 §D3 governs the perimeter constraint: Option B normalization handle must not widen the
prism-query → prism-spec-engine import surface. Read §D3 before choosing Option B:
`.factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md`.

---

## Narrative

As a Prism operator relying on the VSDD contract chain, I want the production response
cache to store the exact raw sensor API response (pre-OCSF normalization) and apply
normalization after cache retrieval, so that (1) BC-2.07.003's invariants are
mechanically testable against the production code path instead of being satisfied
only "at this layer" by a documented deviation, and (2) sensor-spec hot-reloads take
effect immediately on cached entries — a query hitting an unexpired cache entry is
normalized with the CURRENT spec snapshot, not the spec that was live at fetch time.

---

## Behavioral Contracts

| BC | Title | Key Clauses Implemented |
|----|-------|------------------------|
| BC-2.07.003 | Query Engine Sensor-Fetch Cache with Configurable TTL | "cache stores raw sensor responses" postcondition; "OCSF normalization and PrismQL post-filters are applied after cache retrieval" postcondition; invariant "the cached response is the exact sensor API response that was fetched — no transformation applied before caching"; TTL/force_refresh/D3-invalidation postconditions preserved at the new seam |
| BC-2.07.005 | Cache Key Derivation from Push-Down Parameters | 4-tuple key + `push_down_hash` incl. `fetch.limit` (v4.4 D1) preserved byte-identically; single-binding coherence invariant re-anchored at the new seam |

### Preserved Contracts (regression surface — NOT in `behavioral_contracts`; no new ACs)

These contracts' existing test suites must remain green, mechanically adapted to the
migrated value type only (assertions unchanged):

| BC | Surface this story touches | Preservation requirement |
|----|---------------------------|--------------------------|
| BC-2.07.004 | Cache invalidation on writes (prefix-scan `invalidate_by_prefix` / `invalidate_by_client`) | Invalidation paths re-typed, semantics unchanged |
| BC-2.07.006 | LRU eviction + byte-budget bounds (DI-018, SEC-003 per-entry cap) | Byte accounting re-derived for raw-JSON value type; bound semantics unchanged (also covered by AC-010 via BC-2.07.003 invariant DI-018) |
| BC-2.01.013 v1.14 | OCSF Conformance Clause of `pipeline_result_to_record_batch` (`_sensor` injection from spec, class_uid/category_uid derived) | Normalization output unchanged; only WHERE it runs moves (post-cache-read instead of pre-cache-write) |
| BC-2.11.005 | Per-query in-query cache (`MaterializationContext.in_query_cache`) | UNCHANGED — stays `Vec<RecordBatch>` (post-normalization self-join reuse within one query); do not conflate with the cross-query response cache |

---

## Current State (as-built deviation — what this story removes)

Verified against `.worktrees/FIX-REVIEW-QRY-2026-06-10` (branch `fix/review-2026-06-10-query-core`):

- `prism-query/src/cache.rs`: `SensorResponseValue = Vec<arrow::record_batch::RecordBatch>`
  is the production cache value type (`SensorResponseCache = GenericQueryCache<SensorResponseValue>`);
  the `CacheValue` trait doc comment carries the authorized-deviation citation
  ("The as-built `SensorAdapter::fetch` boundary returns Arrow `RecordBatch`es
  (normalization happens inside the adapter), so the fan-out-level response cache
  stores the adapter's complete response in that form").
- `prism-bin/src/spec_driven_adapter.rs`: `SpecDrivenSensorAdapter::fetch()` receives
  `PipelineResult` (raw JSON records) from `PipelineExecutor` and normalizes via
  `pipeline_result_to_record_batch` BEFORE returning — so by the time the
  materialization fan-out sees the response, the raw form is gone.
- `prism-query/src/materialization.rs`: `derive_response_cache_key` (BC-2.07.005 +
  `fetch.limit`), cache-check before fan-out, `store_or_invalidate_response_cache`
  after fan-out (D3) — all operating on `Vec<RecordBatch>`.
- Consequence: BC-2.07.003 invariant "the cached response is the exact sensor API
  response — no transformation applied before caching" is satisfied only by a doc-comment
  re-interpretation ("at this layer"), not mechanically testable; sensor-spec hot-reloads
  do NOT affect unexpired cache entries (interim mitigation: hot-reload→cache-flush,
  landing in the review-2026-06-10 cycle).

### Interim mitigations in effect until this story ships

1. **Hot-reload → cache-flush** (landing this cycle on the QRY review branch): a spec
   hot-reload flushes affected cache entries so stale-spec normalization cannot be
   served. This story SUPERSEDES the flush for correctness purposes (normalize-on-read
   makes flushing unnecessary); the seam ADR decides whether to retain it as
   defense-in-depth or remove it.
2. **`cache.rs` authorized-deviation citation**: the `CacheValue` doc comment records
   the human-authorized deviation. This story REMOVES that citation as part of AC-005.

---

## Seam Decision (Phase 0 architect task — ADR required before implementation)

Two candidate seams satisfy BC-2.07.003. The architect must author an ADR choosing one
(human-directed: "present the seam decision as an in-story architect task"). This is
legitimate cross-component architect adjudication, not a defer-pattern — the decision
needs reasoning across the SensorAdapter trait surface, the QRY-02 wiring, the spec
hot-reload snapshot model (AD-007 ArcSwap), and the R-010 memory envelope.

| | Option A — Adapter seam (prism-bin) | Option B — Engine seam (prism-query) |
|---|---|---|
| Where raw JSON is cached | Inside/around `SpecDrivenSensorAdapter::fetch()`, on `PipelineResult.records` before `pipeline_result_to_record_batch` | At the materialization fan-out, with the adapter boundary changed to return raw records + a normalization handle (spec snapshot accessor) |
| Cache key availability | Key components (client_id, sensor_id, source_id, push-down params, fetch.limit) must be threaded INTO the adapter (or the key passed pre-derived from `derive_response_cache_key`) | Key derivation stays exactly where it is in `materialization.rs` (zero key-surface movement — strongest AC-011 story) |
| Normalization-on-read site | Adapter normalizes on both hit and miss paths (single normalization site preserved in prism-bin) | Materialization read path calls normalization; `pipeline_result_to_record_batch` (or its extracted equivalent) must become callable from prism-query — perimeter implications |
| Trait surface impact | `SensorAdapter::fetch` signature unchanged (`Vec<RecordBatch>` out) | `SensorAdapter` trait changes (raw-out or dual-out) — prism-sensors touched; wider blast radius |
| Tenant isolation | Adapter must enforce per-client key partitioning that materialization currently guarantees | Unchanged (materialization already iterates per-client partitions) |
| Forbidden-dependency exposure | prism-bin already depends on prism-query (cache types importable) | Normalization helper must not drag prism-spec-engine deeper into prism-query than the perimeter allows |

The ADR must also adjudicate: byte-size estimation for the raw-JSON production value
type (CR-006 flat estimate vs measured deep-size), retention/removal of the interim
hot-reload flush, and the accepted hit-path CPU tradeoff (see Risk Mitigations).

---

## Acceptance Criteria

### Group A — Raw-response storage + normalize-on-read (BC-2.07.003)

**AC-001 — Production cache stores raw pre-OCSF sensor records**
(traces to BC-2.07.003 postcondition "The cache stores the complete result set returned
by the fan-out fetch for the effective fetch-limit (pre-OCSF-normalization sensor records)")

Given a query that misses the cache and fetches from a sensor API (DTU clone),
when the response is stored in the cross-query response cache,
then the stored value is the raw sensor records as returned by the sensor API for the
effective fetch-limit — with NO `_sensor` spec-injection, NO derived `class_uid`/`category_uid`
OCSF envelope, and NO Arrow conversion applied before storage.

Red Gate: `test_BC_2_07_003_cache_stores_raw_pre_ocsf_records`

**AC-002 — Exact-response invariant is mechanically testable**
(traces to BC-2.07.003 invariant "The cached response is the exact sensor API response
that was fetched — no transformation applied before caching")

Given a sensor API (mock at the dependency boundary per SID-1) returning a known JSON
record set R, when the fetch populates the cache and the cached value is read back via
a test accessor, then the cached value is structurally equal to R (field-for-field,
including fields the sensor spec does NOT declare — proof that no spec-driven projection
ran before caching).

Red Gate: `test_BC_2_07_003_cached_value_structurally_equals_sensor_response`

**AC-003 — OCSF normalization and post-filters applied after cache retrieval**
(traces to BC-2.07.003 postcondition "The query engine's OCSF normalization and PrismQL
post-filters are applied after cache retrieval, not before — the cache stores raw sensor
responses")

Given an unexpired cache entry holding raw records, when an identical-key query hits the
cache, then the returned `RecordBatch`es are produced by running OCSF normalization
(per BC-2.01.013 v1.14 conformance: spec-driven columns, `_sensor` injected from spec,
derived class_uid/category_uid) on the cached raw records using the CURRENT sensor-spec
snapshot, followed by virtual-field injection and PrismQL post-filters — and the sensor
API is NOT contacted.

Red Gate: `test_BC_2_07_003_normalization_applied_on_cache_read_not_write`

**AC-004 — Sensor-spec hot-reload takes effect immediately on cached entries**
(traces to BC-2.07.003 postcondition "OCSF normalization … applied after cache retrieval"
— behavioral consequence that makes the raw-cache model load-bearing)

Given an unexpired cache entry populated while sensor-spec v1 was live, when the sensor
spec hot-reloads to v2 (e.g., a column added/retyped) and an identical-key query runs
within TTL, then the query is a cache HIT and its output reflects v2 normalization
(new column present/retyped) — no cache flush, no TTL expiry, no force_refresh required.

Red Gate: `test_BC_2_07_003_hot_reload_renormalizes_unexpired_cache_entry`

**AC-005 — RecordBatch production cache instantiation removed; deviation citations removed**
(traces to BC-2.07.003 invariant "The cached response is the exact sensor API response …"
— structural enforcement that the deviation cannot silently return)

Given the merged story, then: (a) the production cross-query response cache instantiation
no longer stores `Vec<RecordBatch>` (`SensorResponseValue` is re-bound to the raw record
type, or removed; any `impl CacheValue for Vec<RecordBatch>` is deleted or demoted to
test-only with a justification comment); (b) the authorized-deviation citation in the
`cache.rs` `CacheValue` doc comment is removed and replaced with the spec-compliant
description; (c) a workspace grep for the deviation language ("as-built", "normalization
happens inside the adapter" in cache.rs) returns zero production hits.

Red Gate: `test_BC_2_07_003_no_recordbatch_production_cache_instantiation`
(type-level/compile-structural assertion + doc sweep verified in adversarial review)

**AC-006 — Re-normalization failure on read degrades to cache miss**
(traces to BC-2.07.003 error case "Cache serialization failure → Log warning, proceed
as cache miss, query sensor API directly")

Given an unexpired cache entry whose raw records fail normalization under the current
spec snapshot (e.g., spec now declares an incompatible type for a cached field), when an
identical-key query runs, then the failure is logged as a warning (structured tracing;
BC-2.16.002 catalog row if a new `event_type` is added), the entry is treated as a cache
miss, and the sensor API is queried directly — no panic, no half-normalized batch served.

Red Gate: `test_BC_2_07_003_renormalization_failure_degrades_to_miss`

### Group B — Preserved cache semantics at the new seam (BC-2.07.003)

**AC-007 — TTL and force_refresh semantics preserved**
(traces to BC-2.07.003 postconditions "TTL values are configurable per data source type
(60s alerts / 300s devices / health not cached)", "When force_refresh: true … the cache
is bypassed and any existing entry for the tuple is replaced", and invariant "TTL is
measured from created_at … not from last access")

Given the migrated cache, then: 60s/300s/not-cached classification by source type,
TTL-from-`created_at`, expiry-on-miss removal, force_refresh bypass-and-replace
(EC-07-032), and the aggregate `total_hits` counter (CR-005) all behave identically to
the pre-migration suite — existing BC-2.07.003 tests pass with only mechanical
value-type adaptation.

Red Gate: `test_BC_2_07_003_ttl_and_force_refresh_preserved_raw_seam`

**AC-008 — D3 forced-refresh-failure invalidation and complete-responses-only preserved**
(traces to BC-2.07.003 postcondition "When force_refresh: true and the fresh fetch fails
to produce a complete response … the existing cache entry for the tuple is invalidated
(removed), not retained" and EC-07-033/EC-07-034)

Given an existing entry and `force_refresh: true` whose fresh fetch (a) fails for all
targets or (b) returns partial results, then the existing entry is invalidated, the
failure/partials surface via the BC-2.11.011 envelope, partial responses are never
cached, and a subsequent non-forced identical query is a cache MISS — at the new seam,
with raw-record values.

Red Gate: `test_BC_2_07_003_d3_forced_refresh_invalidation_preserved_raw_seam`

**AC-009 — Tenant isolation and cross-client partition behavior preserved**
(traces to BC-2.07.003 §Cross-Client Query Cache Interaction: per-client partitions
keyed by `(client_id, sensor_id, source_id, push_down_hash)`; mixed hit/miss transparent;
entries reusable across single-client and cross-client queries)

Given two clients A and B and a cross-client query (`clients: null`), when client A's
partition has an unexpired raw entry and client B's does not, then A is served from
cache (re-normalized) and B fetches live; B's fetch populates only B's partition; a
subsequent single-client query for A with identical push-down params hits A's entry.

Red Gate: `test_BC_2_07_003_cross_client_partitions_preserved_raw_seam`

**AC-010 — DI-018 bounds enforced with raw-JSON byte accounting**
(traces to BC-2.07.003 invariant "DI-018: Cache bounds (LRU eviction when entry count
exceeds configurable per-client-per-sensor bound)")

Given the raw-record production value type, then per-partition LRU entry bounds, the
byte budget, and the SEC-003 per-entry cap are enforced using the byte-size estimation
method recorded in the seam ADR (CR-006 flat estimate or measured deep-size) — existing
BC-2.07.006 bound tests pass with only the estimation constant/method adapted, and the
estimation method is documented at the `CacheValue` impl.

Red Gate: `test_BC_2_07_003_di_018_bounds_enforced_raw_value_type`

### Group C — Cache key surface survives the redesign (BC-2.07.005)

**AC-011 — Cache key derivation byte-identical: 4-tuple + fetch.limit canonicalization**
(traces to BC-2.07.005 postconditions: 4-tuple key with plain first-three components;
SHA-256 of canonicalized push-down params; `fetch.limit` included with exact u64 value,
omitted when 0/absent; PrismQL string / force_refresh / post-filters excluded)

Given the new seam, when cache keys are derived for the canonical BC-2.07.005 test
vectors, then all six canonical vectors — including EC-07-040 (syntax-different queries
share hash), EC-07-041 (force_refresh excluded), EC-07-042 (absent ≡ null),
EC-07-043 (limit 25 vs 1000 never alias), EC-07-044 (limit 0 ≡ absent) — produce hashes
identical to the pre-migration implementation (`derive_response_cache_key` semantics
unmoved or moved verbatim), and the VP-025 Kani proof inputs are unchanged.

Red Gate: `test_BC_2_07_005_key_vectors_identical_at_new_seam`

**AC-012 — Single-binding coherence: the limit hashed is the limit fetched**
(traces to BC-2.07.005 invariant "The fetch-limit hashed into push_down_hash is always
the limit value actually pushed into the fan-out fetch (single-binding coherence; the
key can never describe a different truncation than the stored response)")

Given the new seam (where key derivation and raw-response storage may live in different
crates depending on the ADR option), when a fetch executes with effective fetch-limit L,
then the SAME binding of L feeds both the `fetch.limit` hash input and the sensor fetch
that produced the stored raw response — verified by a test that would fail if the seam
move introduced a second, divergent limit binding.

Red Gate: `test_BC_2_07_005_single_binding_coherence_at_new_seam`

---

## Red Gate Test Plan

All tests written FAIL-first per SID-1. Sensor API mocked at the dependency boundary
(in-process; no `#[ignore]` for DTU availability — unit-level seams per SID-1 §2).

| # | Test Name | Crate | BC Clause | Type |
|---|-----------|-------|-----------|------|
| 1 | `test_BC_2_07_003_cache_stores_raw_pre_ocsf_records` | per seam ADR (prism-bin or prism-query) | BC-2.07.003 PC raw-storage | unit |
| 2 | `test_BC_2_07_003_cached_value_structurally_equals_sensor_response` | per seam ADR | BC-2.07.003 INV exact-response | unit |
| 3 | `test_BC_2_07_003_normalization_applied_on_cache_read_not_write` | per seam ADR | BC-2.07.003 PC normalize-after-retrieval | unit |
| 4 | `test_BC_2_07_003_hot_reload_renormalizes_unexpired_cache_entry` | prism-bin (integration w/ spec hot-reload) | BC-2.07.003 PC normalize-after-retrieval | integration |
| 5 | `test_BC_2_07_003_no_recordbatch_production_cache_instantiation` | prism-query | BC-2.07.003 INV exact-response (structural) | unit/compile-structural |
| 6 | `test_BC_2_07_003_renormalization_failure_degrades_to_miss` | per seam ADR | BC-2.07.003 error table | unit |
| 7 | `test_BC_2_07_003_ttl_and_force_refresh_preserved_raw_seam` | prism-query | BC-2.07.003 TTL/force_refresh PCs | unit |
| 8 | `test_BC_2_07_003_d3_forced_refresh_invalidation_preserved_raw_seam` | prism-query | BC-2.07.003 D3 PC, EC-07-033/034 | unit |
| 9 | `test_BC_2_07_003_cross_client_partitions_preserved_raw_seam` | prism-query | BC-2.07.003 §Cross-Client | unit |
| 10 | `test_BC_2_07_003_di_018_bounds_enforced_raw_value_type` | prism-query | BC-2.07.003 INV DI-018 | unit |
| 11 | `test_BC_2_07_005_key_vectors_identical_at_new_seam` | prism-query | BC-2.07.005 PCs + EC-07-040..044 | unit |
| 12 | `test_BC_2_07_005_single_binding_coherence_at_new_seam` | per seam ADR | BC-2.07.005 INV single-binding | unit |

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~7 000 |
| Seam ADR (new, architect-authored at Phase 0) | ~3 500 |
| BC-2.07.003 v4.5 (full) | ~2 500 |
| BC-2.07.005 v4.4 (full) | ~2 200 |
| proposals/cache-envelope-adjudication-2026-06-10.md (D1/D3 context) | ~3 500 |
| prism-query/src/cache.rs (full — value type migration target) | ~14 000 |
| prism-query/src/cache_key.rs (full) | ~3 200 |
| prism-query/src/materialization.rs (cache-relevant regions only: derive_response_cache_key, store_or_invalidate, fan-out hit/miss path — NOT full file) | ~6 000 |
| prism-query/src/engine.rs (cache wiring regions only) | ~2 500 |
| prism-bin/src/spec_driven_adapter.rs (full — adapter seam) | ~11 500 |
| Existing BC-2.07.x test files (adaptation surface) | ~5 000 |
| Test stubs (12 × ~50 lines) | ~3 000 |
| Tool outputs (nextest, clippy, kani VP-025) | ~3 000 |
| **Total estimate** | **~66 900** |

At ~256k context window this is ~26% — within the 20-30% ceiling, contingent on the
materialization.rs partial-read discipline (full file is 2 323 lines; load only the
cache-relevant regions listed).

---

## Tasks

**Phase 0: Seam ADR + re-baseline (gates implementation)**

- [ ] Orchestrator dispatches architect: author seam ADR (Option A adapter-seam vs
      Option B engine-seam; see §Seam Decision matrix) incl. byte-size estimation
      method, interim-flush retention/removal, and hit-path CPU tradeoff acceptance
- [ ] Run `dclaude:remove-uncertainty` against post-demo develop (after
      fix/review-2026-06-10-query-core + the three demo-objective stories merge):
      confirm cache.rs / materialization.rs / spec_driven_adapter.rs substrate
      assumptions in §Current State still hold
- [ ] Story-writer updates this story to the chosen seam (collapse per-seam-ADR
      placeholders in the Red Gate table and File Structure table); status draft → ready

**Phase 1: Cache value-type migration (prism-query)**

- [ ] Read `crates/prism-query/src/cache.rs` fully before editing
- [ ] Re-bind the production cache value type to raw pre-OCSF records; delete or
      test-gate `impl CacheValue for Vec<RecordBatch>`; remove the authorized-deviation
      doc citation (AC-005)
- [ ] Implement the ADR-selected byte-size estimation for the raw value type (AC-010)
- [ ] Mechanically adapt existing BC-2.07.003/004/005/006 suites to the value type
      (assertions unchanged); run `just iter prism-query` GREEN
- [ ] Write Red Gate tests 5, 7, 8, 9, 10, 11 (FAIL first where behavior is new)

**Phase 2: Raw-cache seam + normalize-on-read (per ADR option)**

- [ ] Read the seam files fully before editing (Option A:
      `crates/prism-bin/src/spec_driven_adapter.rs`; Option B: materialization read
      path + SensorAdapter trait)
- [ ] Move cache population to the raw seam: store `PipelineResult.records`-equivalent
      raw records for the effective fetch-limit (AC-001/AC-002)
- [ ] Implement normalize-on-read: cache hit → normalization with CURRENT spec snapshot
      (ArcSwap::load, AD-007) → virtual fields → post-filters (AC-003)
- [ ] Implement re-normalization-failure → warn + miss + live fetch (AC-006); register
      any new `event_type` in BC-2.16.002 in the SAME commit (SAP-1)
- [ ] Preserve the single `fetch_limit` binding feeding both key hash and fetch (AC-012)
- [ ] Write Red Gate tests 1, 2, 3, 6, 12 (FAIL first)

**Phase 3: Hot-reload effectiveness + interim-mitigation disposition**

- [ ] Write Red Gate test 4 (FAIL first): spec v1 entry, hot-reload v2, hit reflects v2
- [ ] Per seam ADR: remove the interim hot-reload→cache-flush or retain as
      defense-in-depth (with doc rationale); sibling-sweep all flush callsites
      (TD-VSDD-060) if removed
- [ ] Benchmark cache-hit path before/after (normalize-once vs normalize-on-hit);
      record result against the R-010 envelope in the PR description

**Phase 4: Closure**

- [ ] TD-VSDD-060 sibling sweep: `SensorResponseValue`, `store_or_invalidate_response_cache`,
      `derive_response_cache_key` callsites across prism-query + prism-bin
- [ ] SAP-1 probe: `rg 'event_type\s*=' crates/ --type rust` — all emissions have
      BC-2.16.002 rows
- [ ] `cargo kani -p prism-query` — VP-025 proof GREEN
- [ ] `just check` GREEN; doc sweep confirms zero deviation-language residue (AC-005c)

---

## Previous Story Intelligence

- **S-3.05 (pagination-caching)** built the original cache: CR-005 (aggregate
  `total_hits` instead of per-entry counters — clone-on-hit cost) and CR-006
  (AVG_ROW_SIZE_BYTES flat estimate for JSON rows) decisions live in cache.rs and
  BC-2.07.003 v4.4 changelog. The per-entry-counter rejection rationale (cloning
  `Vec<serde_json::Value>` on every hit) becomes MORE relevant with raw-JSON values —
  do not reintroduce per-entry counters.
- **QRY-02 closure (review-2026-06-10 cycle)** wired the engine-owned cache into
  `MaterializationContext::with_response_cache` — before that, the cache was
  constructed but unreachable. Lesson: cache wiring claims must be verified by a
  hit-path test that proves the sensor API is NOT contacted, not by construction-site
  inspection.
- **P1-01 closure (same cycle)**: the fetch.limit was hashed into the key precisely
  because cached responses are limit-truncated; the single-binding coherence invariant
  (BC-2.07.005 v4.4) was added to prevent key/fetch divergence. The seam move is the
  exact scenario that invariant anticipates — AC-012 is its regression vehicle.
- **P1-05 closure (same cycle)**: D3 forced-refresh invalidation
  (`store_or_invalidate_response_cache`, `remove_entry`) landed days before this story
  was authored; preserve it byte-identically (AC-008).
- **PLUGIN-MIGRATION-001-D lessons 16/17/24**: SAP-1 same-commit catalog rows; SID-1
  no-ignored-test rationalization prohibition (mock the sensor boundary in-process);
  read the real types before writing tests — for this story, read `PipelineResult`
  and `pipeline_result_to_record_batch` in spec_driven_adapter.rs, do NOT infer the
  raw-record shape from BC prose.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Cache stores the exact sensor API response — no transformation before caching | BC-2.07.003 invariant (P1-03: spec UNAMENDED) | Red Gate tests 1, 2, 5 |
| Normalization on read uses the CURRENT spec snapshot via `ArcSwap::load()`, never a Mutex, never a populate-time snapshot | AD-007 hot-reload model | Red Gate test 4 + adversary probe |
| `fetch.limit` single-binding coherence: one binding feeds key hash AND fetch | BC-2.07.005 v4.4 invariant | Red Gate test 12 |
| Partial responses are never cached; forced-refresh failure invalidates | BC-2.07.003 v4.5 D3 | Red Gate test 8 |
| Per-query in-query cache (BC-2.11.005) remains `Vec<RecordBatch>` — distinct from cross-query response cache | BC-2.07.003 invariant "only one cache type exists" applies to the CROSS-QUERY cache; in-query cache is a per-query planning structure | Adversary probe: no value-type change in `in_query_cache` |
| No new prism-query → prism-spec-engine perimeter widening (Option B normalization handle must respect the perimeter) | `tests/external/perimeter-violation/` | Compile-fail gate |
| Re-normalization failure: log warning, degrade to miss — never `unwrap()`/`expect()` | BC-2.07.003 error table + CLAUDE.md error handling | Red Gate test 6 + clippy/adversary |
| New `tracing::*!(event_type=…)` sites require same-commit BC-2.16.002 rows | SAP-1 | Adversary SAP-1 probe |
| New pub types in prism-core/prism-spec-engine/prism-query need `#[non_exhaustive]` + ci.yml EXPECTED bump | CLAUDE.md non-exhaustive discipline | Compile-fail gate `tests/external/non-exhaustive-violation/` |
| Production `reqwest::Client` (if any new) uses `.timeout(Duration::from_secs(30))` | CLAUDE.md conventions | Adversary |
| Spec content cites function names + behavioral anchors, not line numbers | TD-VSDD-091 | This story complies (function-name anchors throughout) |

---

## Library & Framework Requirements

Versions pinned from the workspace `Cargo.toml` / `dependency-graph.md`. Do NOT invent
versions; no NEW external dependencies are expected for this story.

| Crate | Version | Usage |
|-------|---------|-------|
| `serde_json` | workspace-pinned | Raw pre-OCSF record value type (`serde_json::Value` rows) |
| `arrow` | workspace-pinned | `RecordBatch` output of normalize-on-read (BC-2.01.013 conformance) — REMOVED from the cache value position |
| `sha2` | workspace-pinned | `push_down_hash` (unchanged; BC-2.07.005) |
| `arc-swap` | workspace-pinned | Current-spec snapshot on the read path (AD-007) |
| `tokio` | `1` (multi-threaded) | Async fan-out (unchanged) |
| `tracing` | workspace-pinned | Re-normalization-failure warning (structured fields; BC-2.16.002) |
| `kani-verifier` | workspace-pinned (non-Windows dev-dep) | VP-025 proof re-run |

**Forbidden patterns:**
- Do NOT add a second cache type or a "normalized-view cache" alongside the raw cache —
  BC-2.07.003 invariant: only one cache type exists
- Do NOT capture the spec snapshot at cache-populate time for later read-path use
- Do NOT introduce per-entry hit counters (CR-005 rejection stands)

---

## File Structure Requirements

Final MODIFY/CREATE set narrows at the Phase 0 seam ADR; superset shown.

| File | Action | Purpose |
|------|--------|---------|
| `.factory/specs/architecture/` (new ADR; architect-owned) | CREATE (Phase 0) | Seam decision Option A vs B + byte-estimation + flush disposition |
| `crates/prism-query/src/cache.rs` | MODIFY | Production value type → raw records; remove `impl CacheValue for Vec<RecordBatch>` (or test-gate); remove deviation citation; byte estimation |
| `crates/prism-query/src/cache_key.rs` | MODIFY (verify-only expected) | Key derivation must be byte-identical (AC-011); changes only if the seam ADR relocates derivation verbatim |
| `crates/prism-query/src/materialization.rs` | MODIFY | Read-path re-normalization wiring (Option B) or raw-value plumbing to/from adapter (Option A); `store_or_invalidate_response_cache` re-typed |
| `crates/prism-query/src/engine.rs` | MODIFY | `SensorResponseCache` type alias re-bind at construction sites |
| `crates/prism-bin/src/spec_driven_adapter.rs` | MODIFY (Option A primary site) | Cache raw `PipelineResult` records before `pipeline_result_to_record_batch`; normalize on hit path |
| `crates/prism-sensors/` (SensorAdapter trait) | MODIFY (Option B ONLY) | Raw-out / dual-out fetch surface |
| `crates/prism-query/src/tests/` + existing `bc_2_07_*` test files | MODIFY | Mechanical value-type adaptation + 12 new Red Gate tests |
| `.factory/specs/behavioral-contracts/BC-2.16.002-*.md` | MODIFY (only if new event_type) | Catalog row for re-normalization-failure warning (SAP-1, same commit) |
| `.github/workflows/ci.yml` | MODIFY (only if new pub types) | `EXPECTED=` bump per non-exhaustive discipline |

---

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-2.07.003 EC-07-030 | Two concurrent identical queries, both miss | Both correct; no coalescing; ≤2 API calls; both normalize their own results (unchanged) |
| EC-002 | BC-2.07.003 EC-07-031 | TTL expires between check and return | Stale-by-milliseconds acceptable (unchanged) |
| EC-003 | BC-2.07.003 EC-07-032 | force_refresh with no existing entry | Fetch + cache raw normally (AC-007) |
| EC-004 | BC-2.07.003 EC-07-033/034 | Forced refresh fails / partial | Invalidate entry; never cache partial; BC-2.11.011 envelope (AC-008) |
| EC-005 | This story | Spec hot-reload between populate and read | Hit re-normalizes with v2 snapshot (AC-004) |
| EC-006 | This story | Spec hot-reload REMOVES a column present in cached raw records | Hit re-normalizes: removed column absent from output; extra raw fields ignored (spec-driven projection); no error |
| EC-007 | This story | Spec hot-reload retypes a column incompatibly with cached raw values | Re-normalization failure → warn + miss + live fetch (AC-006) |
| EC-008 | This story | Raw records contain a vendor `_sensor` field | Cached verbatim (exact-response invariant); normalize-on-read OVERWRITES with spec-canonical SensorId per BC-2.01.013 (never trusts raw `_sensor`) |
| EC-009 | BC-2.07.005 EC-07-043 | limit=25 vs limit=1000 | Distinct hashes; no aliasing of truncated raw sets (AC-011) |
| EC-010 | BC-2.07.005 EC-07-044 | fetch-limit 0 vs absent | Same hash (omission rule) (AC-011) |
| EC-011 | BC-2.07.006 surface | Raw entry exceeds SEC-003 per-entry cap under new estimation | Entry not cached / evicted per existing bound semantics (AC-010) |
| EC-012 | AD-007 | In-flight query holds spec snapshot across its lifetime while reload lands | In-flight query completes under its snapshot; NEXT query re-normalizes under v2 (snapshot-per-query, not per-batch) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Anchor |
|-----------|--------|---------------|--------|
| Raw-record cache value type + byte estimation | `prism-query/src/cache.rs` (`CacheValue` impl) | Pure (data + size fn) | BC-2.07.003 INV exact-response; seam ADR |
| Cache key derivation (`derive_response_cache_key`, `CacheKey::derive`) | `prism-query/src/{materialization.rs,cache_key.rs}` | Pure (deterministic hash) | BC-2.07.005; VP-025 |
| Cache store/invalidate decision (`store_or_invalidate_response_cache`) | `prism-query/src/materialization.rs` | Effectful (cache mutation) | BC-2.07.003 D3 |
| Raw-response capture at seam | Option A: `prism-bin/src/spec_driven_adapter.rs`; Option B: materialization fan-out | Effectful (HTTP + cache write) | Seam ADR |
| Normalize-on-read (`pipeline_result_to_record_batch` or extracted equivalent) | Option A: prism-bin; Option B: callable from prism-query within perimeter | Pure given (records, spec snapshot) | BC-2.01.013 v1.14 |
| Spec snapshot access on read path | ArcSwap load at query start | Effectful (shared-state read) | AD-007 |

---

## Forbidden Dependencies

| Crate | Forbidden Dependency | Reason |
|-------|---------------------|--------|
| `prism-query` | deeper `prism-spec-engine` surface than the existing perimeter allows | `tests/external/perimeter-violation/` gate; Option B normalization handle must be injected (Arc-DI per ADR-022), not imported |
| `prism-core` | `arrow` (no new) | Core stays Arrow-free; raw value type is serde_json-based |
| any | new external crates for deep-size estimation without architect ADR approval | Seam ADR owns the estimation method decision |

---

## SAP-1 Compliance (Structured Event Catalog)

Expected potential emission (implementer must enumerate actual sites):
- Possibly `event_type = "cache.renormalization_failed"` on the AC-006 warn path.

If added, the BC-2.16.002 catalog row (field schema, audit role, recurrence policy)
lands in the SAME commit. If NO new `event_type` emissions are added, state so
explicitly in the PR description.

---

## Story Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.1 | 2026-08-02 | Added ## Authority section (DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001 Round 6, D-2084). |
| v1.0 | 2026-06-10 | Initial authoring per explicit human P1-03 adjudication (2026-06-10): BC-2.07.003 raw-response cache model stays UNAMENDED; the as-built post-normalization RecordBatch cache (human-authorized deviation, cache.rs CacheValue doc citation) is brought into spec compliance post-demo. Sequenced after live-demo objective (S-DEMO-DTU-LIVE-SCENARIO-001-B → S-DEMO-MULTI-TENANT-DTU-001 → S-DEMO-004) + fix/review-2026-06-10-query-core merge. Seam decision (Option A adapter-seam vs Option B engine-seam) gated on Phase 0 architect ADR per human direction. 12 ACs; 12 Red Gate tests; 8 pts. Interim mitigations documented: hot-reload→cache-flush (landing in review-2026-06-10 cycle) + cache.rs authorized-deviation citation. |
