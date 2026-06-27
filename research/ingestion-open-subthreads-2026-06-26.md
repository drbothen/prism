---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-26"
provenance: "side-analysis discussion input; does not modify vision/specs."
topic: "Remaining OPEN morph-time sub-threads of prism's federated-ingestion / detection design (7 sub-threads): detection-portability governance, query-result-cache coherence across hops, continuous-operator checkpoint cadence/recovery isolation, detection-driven packet-pin policy, entity-registry cross-schema resolution incl. OT, per-satellite edge compute budget & backpressure governance, residency-primitive granularity + replication-policy authorship locus."
scope: "SIDE-ANALYSIS / discussion input — NOT a spec, vision, brief, ADR, BC, or PRD change."
---

# Prism Federated-Ingestion / Detection — Remaining Open Sub-Threads (Cited Side-Analysis)

> **BOUNDARY.** This is a cited research capture to inform a HUMAN DISCUSSION. It does **not** modify
> `matured-vision-day2-requirements.md`, any spec, STATE.md, SESSION-HANDOFF.md, prior research docs,
> or any live factory artifact. It does **not** decide the open questions it raises. Leans are
> *pressure-tested recommendations for discussion*, not adopted decisions. `do_not_execute: true`.

> **Citation discipline.** Findings are grounded in MCP-tool-retrieved web research (Perplexity
> `sonar-deep-research`, seven deep passes at `reasoning_effort: high`, plus one `perplexity_ask`
> cross-check). Vendor/spec/paper sources are named inline with bracketed markers as returned by the
> deep-research synthesis. Claims resting on model knowledge are flagged **[model-knowledge]**;
> inconclusive items are flagged **[INCONCLUSIVE]**. Date-stamped as of 2026-06.

> **Read-coverage honesty.** Each deep-research pass returned 90k–105k characters saved to tool-result
> files. For sub-threads 1 and 2 the synthesis prose was read in full through section 6 of 8 (the
> remaining sections were conclusion/synthesis that restate earlier findings). For sub-threads 3–7 the
> single-line JSON files exceeded the read cap, so they were mined via targeted Grep extraction of the
> abstract, per-system verdicts, gap statements, and recommendation passages — NOT read line-by-line in
> full. Where a lean rests on the extracted abstract + targeted passages rather than a full read, that
> is the honest coverage level. The OCSF cross-event-identity claim (sub-thread 5) was independently
> cross-checked with a second Perplexity call.

> **Grounding read (modified nothing).** matured-vision §17.6/§17.7/§17.8/§17.10/§17.12/§17.13/§17.14
> (collector class, pcap regime, continuous operator, chain-aware tiering/replication/deadline model,
> 11 open questions, Spicy-style dissector, OT matrix), §12.1 (entity pivot / entity registry grammar),
> §13.6 (multi-schema), §14 (detection-as-query, rule lifecycle, OT detection). Prior passes
> `federated-ingestion-collector-connectors-2026-06-26.md`,
> `chain-cache-tiering-replication-deadlines-2026-06-26.md`,
> `detection-reshape-protocol-dissectors-2026-06-26.md`. Prism invariants held throughout:
> ephemeral / federated / residency-first / OCSF+native / RocksDB-native, no new datastore.

---

## 1. Executive Summary (~12 lines)

1. **All seven sub-threads share one shape:** the *building blocks* exist in mature systems, but the
   *integration prism needs* — residency-first, federated, ephemeral, RocksDB-native, OCSF+OT — is
   greenfield in every case. Prism is **assembling, not borrowing wholesale**.
2. **Detection-portability (1):** Sigma logsource + pySigma pipelines + Elastic `required_fields` +
   Sentinel data-connector gating are the prior art, but **graceful degradation is universally a gap** —
   rules either run or silently return zero rows; no system has a portable OT-native rule standard. Lean:
   **capability/data-availability-gated enablement using a declared data-dependency manifest per rule**,
   with explicit `degraded` status (not silent zero-rows).
3. **Result-cache coherence (2):** OLAP "Intent Signature" canonicalization is the closest prior art;
   demand-driven, event-time-TTL'd, cross-residency-hop result-set coherence is **genuinely greenfield**.
   Lean: **canonical query+window key, event-time-watermark-versioned entries, independent per-tier TTL +
   coverage-metadata reconciliation at query time (NOT a cross-hop coherence guarantee)**, with subsumption
   as an optimization and single-flight to stop stampede.
4. **Checkpoint cadence (3):** Flink RocksDB incremental + changelog/generic-log checkpoints are mature;
   **per-state-type cadence and two-tier (hot-window vs durable-detection) split are NOT first-class** —
   flagged greenfield by the research. Lean: **at-least-once + idempotent finding-emit (relax exactly-once),
   recompute-cheap ephemeral window state, durably checkpoint only detection/alert state**; beware
   processing-time timer storms on restore.
5. **Packet-pin policy (4):** Corelight Smart PCAP (trigger-based, per-protocol byte-depth) + Zeek Time
   Machine (class-based first-N-bytes, EOL) + Arkime (full-take) bracket the design; **no tool has a
   first-class "pin op" with per-pin severity-tiered retention**. Lean: **decoupled capture-agent →
   detection → policy-engine-issues-pin pipeline; first-N-bytes default with full-session escalation;
   pin owned by a retention-policy engine (not the detection rule); Community ID as the session key; PTP/NTP mandatory**.
6. **Entity-registry cross-schema incl. OT (5):** **OCSF deliberately defines NO cross-event canonical
   identity** (confirmed through 1.3.x); Community ID is flow-centric not asset-centric; IT entity models
   (Sentinel/Elastic/Splunk/Chronicle) map poorly to OT (no agent, protocol-derived sub-device identity).
   Lean: **prism-native config-driven entity registry mapping `entity_type → ordered attribute paths
   across OCSF + native-OT schemas`, deterministic on strong identifiers + temporal-validity windows for
   IP↔asset bindings; OT identity is prism's own concern (no standard to adopt)** — validates §12.1.
7. **Edge budget & backpressure (6):** Canonical drop metrics exist (Zeek `capture_loss`, Suricata
   `capture.kernel_drops`, TCP-gap counters); Aurora/Borealis load-shedding is the toolbox; cgroups v2 +
   CPU-pinning isolate. **No integrated backpressure-governance for a passive sensor running
   dissection + operator + buffer on one box.** Lean: **protect the capture path FIRST (cgroup priority,
   pinned capture cores, ring sizing); shed analysis/operator work before dropping packets; surface
   drop-rate + ring-util + analysis-lag upward as fleet budget signals**.
8. **Residency granularity + locus (7):** Field/column tagging is mature (Purview, Snowflake tags,
   BigQuery policy tags, Unity Catalog) but **all evolved for sensitivity/masking, NOT residency/geo-fencing**
   (residency stays resource/bucket-level). Lean (clear): **classify per-field but always scoped within a
   per-(source-class, schema, version) table**; **author residency/replication policy as a dedicated
   policy-as-code artifact, with inline per-collector config reserved for binding + exceptions** — exactly
   the §17.8.2 residency-first per-field, transform-ordered model.
9. **Cross-cutting:** sub-threads 1, 5 and 7 all converge on *prism owning a residency-and-capability-aware
   metadata layer* (data-dependency manifest + entity registry + residency tags) that the §17.8 policy and
   §14 detection lifecycle both read. Sub-threads 2, 3, 6 converge on *graceful degradation as the
   first-class primitive* (partial+coverage, recompute-cheap state, shed-before-drop) — prism's §3.6
   partial-result thesis extends to all three.
10. **Hardest honest tensions:** (a) cross-hop result coherence is harder than CDN coherence and is best
    *declined* in v1 (independent TTL + coverage metadata, not a guarantee); (b) prism now owns THREE heavy
    native engines if the operator lands (Spicy-style dissector §17.12 + windowed operator §17.7 + entity
    registry) — sequencing matters; (c) OT entity identity and OT-portable detections have **no standard at
    all** — prism is defining, not adopting, and must own the spec surface.
11. **Consolidated honest cost:** four net-new metadata/policy surfaces (rule data-dependency manifest;
    canonical result-cache key + coverage schema; entity registry; residency-tag vocabulary) + two
    operational disciplines (capture-path-first isolation; pin-retention policy engine). None require a new
    datastore; all fit RocksDB-native + TOML-spec-driven dogfooding.
12. **None of the seven leans breaks the thesis.** Each is expressible as declarative config/policy over
    the existing federated/ephemeral/residency-first/RocksDB-native architecture. The recurring discipline
    is: **declare dependencies + degrade explicitly, never silently.**

---

## 2. Sub-Thread 1 — Detection-Portability Governance

### 2.1 Prior art
- **Sigma + pySigma pipelines.** Sigma separates *detection logic* / *logsource* / *metadata*; portability
  across backends comes from **processing pipelines** (ordered YAML transforms: `field_name_mapping`,
  `add_condition` for index/source scoping) applied at `sigma convert -p <pipeline>` time — the rule is
  unchanged, the pipeline re-targets it [Sigma docs; pySigma docs]. Limit: logsource is **coarse-grained**
  (product/service/category); Sigma has **no fine-grained required-vs-optional field declaration** and
  **no defined behavior when a logsource is absent** — conversion still succeeds and the query returns zero
  rows [Sigma docs]. The Datadog pySigma backend explicitly does **no** field mapping (prefixes `@` only),
  pushing all dependency-alignment onto the author [Datadog pySigma backend].
- **Detection-as-code.** RunReveal-style CI/CD: lint → test-against-sample-data (exit-code contract) →
  sync dry-run; version control + PR review [RunReveal]. MITRE ATT&CK coverage tracking + CardinalOps
  "Security Layers" (depth across endpoint/network/identity/cloud) give the coverage axis [CardinalOps; Wiz].
- **Per-data-source content packs.** Splunk ESCU ties detections to CIM data models / accelerated
  `tstats` (empty data model → zero rows, no auto-disable) [Splunk CIM]. Elastic prebuilt rules expose
  **`required_fields`** — explicitly **informational, not runtime-enforced** [Elastic detection API].
  Sentinel solutions/content-hub bundle analytics rules WITH data connectors; rules often ship disabled
  pending connector install, but this is **convention, not enforced gating**; connector dashboards show
  "data received" status [Sentinel content hub / data connectors]. Chronicle YARA-L depends on UDM fields
  implicitly, no declared dependency standard [Google SecOps rules repo].
- **Dependency-declaration standards.** Sigma logsource (coarse), OCSF event classes (schema, no rule
  format), STIX patterning (expressive observable paths, used for intel-sharing not SIEM gating), Elastic
  `required_fields` (vendor-specific, informational). **No cross-vendor standard** for declaring a rule's
  data dependencies — and **none at all for OT-native schemas (Modbus/DNP3)** [synthesis, explicit gap].

### 2.2 Recommendation / lean
- **Make data-availability gating a first-class, prism-native rule attribute.** Each detection's YAML
  metadata (§14.1) carries a **data-dependency manifest**: the OCSF classes/attribute paths AND/OR native-OT
  tables it requires, expressed against the §12.1 entity registry + §13.6 multi-schema model. The planner
  checks the manifest against the site's available capability descriptors (§3.4/§10.3) at enable-time.
- **Three explicit rule states at a site — not silent zero-rows:** `runnable` (all required data present),
  `degraded` (some optional fields absent — runs a reduced predicate set, marks findings
  `coverage=partial`), `unavailable` (required data absent — rule auto-disabled with a surfaced reason).
  This is prism's §3.6 partial-result thesis applied to detection enablement. Mature systems leave this to
  manual ops; prism can make it declarative because it already owns capability descriptors per connector.
- **Two rule classes in ONE lifecycle (§14.1), distinguished only by manifest:** OCSF-portable detections
  declare OCSF classes (run anywhere the class is available); OT-native/site-specific detections declare
  native-OT tables (run only where the dissector/grammar for that protocol is loaded, §17.12). Lifecycle
  (draft→…→production), CI/test, MITRE mapping, content-pack packaging are identical; only the manifest +
  resulting site-applicability differ — dogfoods prism's "every query can be a detection" model.
- **Adopt content-pack packaging** (Splunk ESCU / Sentinel solution shape): group portable detections vs
  per-OT-protocol packs; a pack's manifest is the union of its rules' manifests.

### 2.3 Residual open questions
- Manifest vocabulary: does it reference OCSF attribute paths directly, entity_types (§12.1), or both? How
  does it express "requires Modbus native table" vs "requires OCSF Network Activity 4001"?
- Degraded-mode semantics: which predicates are *droppable* vs *load-bearing*? Does the rule author annotate
  optionality per-predicate, or does the engine infer it from the manifest? (No prior art — greenfield.)
- Coverage accounting: how does a `degraded`/`unavailable` rule reflect into MITRE coverage / fleet posture?
- OT-portable detections across sites with *different* OT vendors/protocols for the same physical function
  (e.g., Modbus at site A, DNP3 at site B detecting the same "unauthorized write"): is there a higher-level
  semantic detection that desugars per-site? (No standard exists — prism would be first.)

---

## 3. Sub-Thread 2 — Query-RESULT-Cache Coherence Across Hops

### 3.1 Prior art
- **Traditional query-result caches** (Snowflake persisted results, Databend) key on **exact SQL text +
  environment** (role/config/table-micro-partition state); reuse only on byte-identical query AND unchanged
  base data; **wall-clock (processing-time) TTL** (Snowflake 24h rolling→31d max; Databend N seconds)
  [Snowflake; Databend]. No canonicalization (capitalization/aliasing breaks reuse), no subsumption.
- **Semantic caching** (Redis, Zuplo) keys on embedding similarity + threshold — good for fuzzy NL, **unsafe
  for analytical exactness**; retains wall-clock TTL [Redis; Zuplo].
- **OLAP semantic caching via LLM-based canonicalization** — the **"Intent Signature"**: a *structured*
  cache key (measures/dimensions/filters/aggregations) stable under syntactic rewrite and across SQL/NL
  [research paper]. **Closest prior art to prism's need**, but single-tier, no event-time TTL, no residency.
- **Materialized views / IVM** (Materialize assigns explicit logical times to updates → strong consistency;
  ClickHouse rollups align windows to `toStartOfHour` buckets, event-time TTL on raw data) — show
  **canonical time-bucket alignment** and **logical-time/watermark versioning** as freshness mechanisms
  [Materialize; ClickHouse].
- **Coherence/stampede**: hierarchical cache grouped-coherence (CPU), Ferdinand local+remote web caches,
  Go `singleflight` / request-coalescing — building blocks, but **none address derived result sets across
  edge/regional/central residency-partitioned hops with event-time TTL** [synthesis, explicit greenfield].
- **CDN-vs-result-cache difference:** CDN key = stable URL (origin authoritative, object semantics
  irrelevant); result-set key = the *computation* (query+params+window) + base-data version — so coherence
  must track base-data evolution, not just TTL [synthesis].

### 3.2 Recommendation / lean (answers §17.10 Q3)
- **Canonical cache key = `canonicalized(query_intent) + canonicalized(time_window) + residency_scope +
  schema_version`.** Borrow the Intent-Signature pattern: parse to logical plan, sort predicates,
  normalize literals/operators, hash. Align time windows to **event-time buckets** (ClickHouse pattern) so
  semantically-equal windows collide; carry inclusive/exclusive boundary flags.
- **Event-time-watermark-versioned entries:** annotate each cached result with the **event-time watermark**
  (max complete event_time) at compute time, plus per-source coverage. Freshness is bounded by the
  watermark + the §3.3 `event_time` TTL, **not** wall-clock. A result is reusable while the requested
  window's end ≤ the entry's watermark AND no late data has advanced past it for that window.
- **DECLINE cross-hop coherence as a guarantee in v1 (the honest call).** Each tier independently TTLs its
  entries; **reconcile at query time via coverage-metadata** (§3.6 / BC-2.01.010) rather than promising a
  cross-hop coherence invariant. This sidesteps "stale parent poisons children," eviction races, and
  double-caching — the exact CDN multi-tier failure modes flagged in the chain-cache pass. Coverage
  metadata says *what window/sources each tier's contribution actually covers*; the coordinator merges +
  reports partial.
- **Subsumption as an optimization, not a correctness mechanism:** a cached wider window may answer a
  narrower query only if its watermark ≥ the narrow window end AND its residency scope ⊇ the requester's
  allowed scope. Otherwise recompute.
- **Single-flight the recompute** (§17.10 Q10): when a hub pre-aggregate expires and N descendants want it,
  coalesce into one recompute keyed by the canonical key; integrate with §3.2 seen-request-ID loop
  prevention so the coalesced fetch is one logical request.

### 3.3 Residual open questions
- Exact canonicalization scope: how aggressive (predicate reorder, constant folding, window subsumption)
  before false-collision risk? Analytical correctness demands *exact* intent equality, not similarity.
- Coverage-metadata schema unification (§17.10 Q9): §3.6/BC-2.01.010 + CAQP + Elastic CCS are three
  vocabularies — which artifact owns prism's canonical coverage schema, and does it carry the watermark?
- Does a hub entry hold **reduced events** (needed for §14 correlation/sequence) or **aggregates** (Dremel
  SUM/COUNT)? (Open §17.10 Q6 — the cache-coherence model must state which; event-level rows are far larger
  and interact with the §17.7 streaming-state question.)
- Residency interaction with subsumption: filtering a cross-region superset down to an allowed subset must
  be *inexpressible* if it would leak — ties to sub-thread 7 transform-ordering (§17.8.2 F6/F7).

---

## 4. Sub-Thread 3 — Continuous-Operator Checkpoint Cadence + Recovery Isolation

### 4.1 Prior art
- **Flink RocksDB incremental checkpoints**: exploit RocksDB immutable SSTables — only *new* SSTs upload,
  not full state; RocksDB is the only backend supporting SSTable-based incremental checkpoints [Flink docs].
- **Generic log-based / changelog state backend**: logs state changes for **shorter, more predictable
  checkpoint cadence** decoupled from RocksDB compaction; MVP-era, presented as a checkpoint-speed/stability
  tool — **not** as a tiered-state abstraction [Flink changelog backend].
- **Exactly-once vs at-least-once**: Flink end-to-end exactly-once = checkpoint barriers + two-phase-commit
  sinks; **at-least-once is acceptable when sinks are idempotent / tolerate duplicates** (e.g., upsert by
  primary key, append-only analytics) [Flink; Spark Structured Streaming uses idempotent-sink exactly-once].
- **Timer + watermark recovery**: state AND timers are checkpointed (TimerService, event- and processing-
  time); on restore, **processing-time timers whose time passed during downtime fire immediately → timer
  storms / burst cleanup-or-emit** — a real recovery hazard for window TTL/idle timers [Flink ProcessFunction].
- **Two-tier / per-state-type cadence**: Flink State TTL + RocksDB compaction filters differentiate
  ephemeral from durable state, **but all managed state is checkpointed UNIFORMLY** — Flink has **no
  first-class abstraction to checkpoint different state classes at different cadences or recovery
  guarantees**; the research **explicitly flags this as a greenfield category** for custom operators
  [Flink State TTL; synthesis]. Kafka Streams changelog topics are a comparative mechanism but not
  per-state-cadence either.
- **Local recovery / task-local state**: Flink keeps a secondary local copy to cut recovery time
  (read-from-local on restart) [Flink local recovery].

### 4.2 Recommendation / lean (informs §17.7 Phase 2 + §17.10 Q6)
- **Relax to at-least-once + idempotent finding-emit.** Prism findings are append-only/keyed (§14.5
  destinations); dedup by `(rule_id, match_key, window)` makes duplicate emits harmless — avoids the cost of
  two-phase-commit exactly-once on a demand-driven, ephemeral operator. The research explicitly endorses
  at-least-once when sinks are idempotent.
- **Two-tier state by criticality (the greenfield prism must design itself):**
  (a) **hot window correlation state** — large, fast-changing, *cheap to recompute* from the §3.3
  RetentionCache window → checkpoint rarely or not at all; on recovery, **rebuild from the cache window**
  rather than restore (the cache IS the durable input). (b) **durable detection/campaign/risk state +
  fired-finding dedup** — small, must survive → checkpoint frequently (changelog backend for short cadence)
  to RocksDB. This split is NOT supported off-the-shelf; prism implements it on its RocksDB-native backend.
- **Cadence trade-off:** short interval = low RTO but high I/O/write-amp; long interval = cheap but more
  replay. Because hot state is recomputable, prism can afford a **long cadence for window state** and a
  **short cadence only for durable detection state** — exactly the asymmetry Flink can't express natively.
- **Handle the processing-time-timer storm** on restore: prefer event-time timers; on restart, clamp/coalesce
  any processing-time timers that "should have fired" instead of firing a burst.

### 4.3 Residual open questions
- Is the windowed operator's hot state ALWAYS recomputable from the cache window, or are there cross-window
  carry-over aggregates that must be durable? (Determines whether the two-tier split is clean.)
- Watermark/late-arrival policy across a restore for a *demand-driven* operator that may not be continuously
  running (unlike Flink's always-on assumption) — does prism even need watermarks, or windowed snapshots?
- RPO target for durable detection state: what data loss is acceptable for fired-finding dedup vs
  campaign-state? Drives the durable-tier cadence.
- Does §17.7 Phase 1 (NRT-over-cache, reuse §14) defer this entirely, making the full operator's checkpoint
  design a Phase-2-only concern? (Feature-ordering question for the human.)

---

## 5. Sub-Thread 4 — Detection-Driven Packet-Pin Policy

### 5.1 Prior art
- **Corelight Smart PCAP**: highly-configurable **trigger-based selective capture** tied to Zeek
  logs/detections; **per-protocol + byte-depth controls** (capture only first portion of unencrypted
  sessions; full payload for selected categories); embeds PCAP references into logs for one-click retrieval
  from SIEM/Investigator; capturing ~10–20% of volume extends retention vs blanket full-take [Corelight
  Smart PCAP white paper]. Corelight co-authors Community ID; flow identity links logs↔PCAP [Community ID spec].
- **Suricata conditional PCAP**: `pcap-log` restricted by alert/condition — a **binary include/exclude**
  driven by detection logic; no fine-grained capture-depth knobs documented [Suricata docs].
- **Zeek Time Machine (NG)**: **class-based, first-N-bytes** model — traffic *classes*, each with its own
  buffer share + per-connection byte cutoff, override-on-demand, retrieval by Zeek connection ID; **now
  end-of-life** but the canonical first-N-bytes-with-per-class-quota design [Time Machine; synthesis].
- **Arkime (Moloch)**: the **full-take opposite extreme** — full packet capture + session indexing (SPI),
  hunt, retrieve-by-session; retention = **emergent property of volume × storage** (ring expiry), not
  per-session pins [Arkime docs].
- **Architectural pattern the research synthesizes:** *capture agent (rolling buffer) → detection engine
  (Zeek/Suricata/NDR, produces flow-tied alerts) → **policy engine issues "pin" operations** instructing the
  capture layer to preserve a session beyond normal retention, possibly to a separate tier* — this
  **decouples** base buffer from pinned evidence. But **no surveyed tool exposes a first-class "pin op" with
  explicit per-pin retention duration or severity-tiered retention** (e.g., "high-sev → 1yr, baseline →
  30d") — retention is emergent, not a pin attribute [synthesis, explicit gap].
- **Session-ID consistency**: **Community ID** flow hash is the cross-tool standard mapping an alert to the
  right packets; requires coherent **NTP/PTP** time-sync between detection and capture [Community ID spec].

### 5.2 Recommendation / lean (informs §17.6 / §17.14 / E-COLLECTOR-PCAP-001)
- **Adopt the decoupled three-stage pipeline:** dissector/detection (§17.12/§14) emits a flow-tied trigger →
  a **retention-policy engine OWNS the pin decision** (NOT the detection rule directly). This matches
  sub-thread 7's locus lean (policy as artifact) and §17.8's residency-first ordering: the pin policy is a
  residency-aware retention decision, not an analytics decision.
- **First-N-bytes as default, full-session as escalation:** mirror Time Machine class-based cutoffs. Default
  pin = headers + first-N-bytes (forensic triage); high-severity / specific-detection classes escalate to
  full-session. Keeps the §17.6 second storage regime sized realistically (Smart PCAP's 10–20% argument).
- **Pin = an explicit object with a retention duration** keyed by **Community ID** (the §17.12 axis-2 key
  linking normalized metadata ↔ pinned packets). Severity-tiered retention duration is the gap prism fills:
  the pin carries `{community_id, byte_depth, retain_until, residency_scope}`.
- **PTP/NTP is a hard prerequisite** at the satellite — alert timestamps must align to captured-packet
  timestamps; surface clock-sync health as a sensor signal (ties to sub-thread 6 fleet signals).

### 5.3 Residual open questions
- Where does the pin policy engine live — on the capture satellite (low latency, local residency) or
  coordinated from the hub? (Edge-first per §17.6 likely, but escalation may need hub awareness.)
- Retention-duration vocabulary: per-severity tiers, per-detection-class, or per-residency-zone?
- Interaction with residency: can a pinned full-session ever transit a residency boundary, or is retrieval
  always brought-to-the-data (query goes to the satellite holding the packets)? (Almost certainly the
  latter, per §17.8.2 raw-stays-in-region.)
- De-dup: multiple detections on the same flow → one pin or N? (Single pin keyed by Community ID, longest
  retain_until wins, is the obvious answer — confirm.)

---

## 6. Sub-Thread 5 — Entity-Registry Cross-Schema Resolution (incl. OT)

### 6.1 Prior art
- **OCSF defines NO cross-event canonical identity** (cross-checked: through OCSF 1.3.x). It standardizes
  per-event objects (device/endpoint with `device.uid`) but **leaves entity resolution to implementers**;
  identity stitching (CMDB/IdP/asset-inventory enrichment) is built ON TOP of the schema, not in it
  [OCSF schema docs; Query.ai mapping guide; Deepwatch]. **This is decisive for prism: there is no standard
  to adopt — the entity registry is necessarily prism-native.**
- **SIEM/XDR entity models**: Sentinel **entity mapping** with explicit field→entity mapping and **strong vs
  weak identifiers** (e.g., `UserPrincipalName` strong, others weak) — deterministic on strong IDs [Sentinel
  Entities ref]. Elastic entity store, Splunk asset & identity framework, Chronicle UDM entity graph —
  similar map-fields-to-canonical-entity, **correlation logic largely proprietary** [synthesis].
- **Community ID** correlates **flows** across tools but is **explicitly flow-centric, not asset-centric** —
  cannot by itself resolve a device identity [Community ID spec].
- **Asset-inventory correlation**: multi-source merge (IP/MAC/hostname/credential), confidence scoring,
  **temporal validity** — IP↔asset bindings change via DHCP, hostnames collide across sites; Rapid7-style
  DHCP-log correlation acknowledges this [Rapid7; synthesis].
- **Deterministic vs probabilistic**: deterministic = exact-match on strong/immutable IDs; probabilistic =
  composite/weighted match on weak IDs. Sentinel's strong/weak taxonomy is the cleanest documented model.
- **OT passive identification**: Claroty/Armis/Nozomi/Dragos passively fingerprint assets from protocol
  traffic (Modbus unit-ID, DNP3 address, ENIP/CIP identity object, device serials) over repeated
  interactions — **proprietary**, and **IT entity models map poorly to OT** (no agent; identity is
  protocol-derived; unit-ID/station-address are *sub-device* identities OCSF/SIEM schemas don't model)
  [Claroty; Armis; synthesis, explicit OT cross-schema gap].

### 6.2 Recommendation / lean (validates + extends §12.1)
- **Prism-native, config-driven entity registry** (§12.1's preferred TOML path is correct — dogfoods the
  spec-driven model and there is *no standard to adopt*). Map `entity_type → ordered set of attribute
  paths` spanning **OCSF + native-OT + IT** schemas. §12.1's `ip → [src_endpoint.ip, dst_endpoint.ip,
  device.ip, …]` is exactly the right shape; extend it with native-OT paths
  (e.g., `ot_asset → [modbus.unit_id@table, dnp3.station_addr@table, enip.cip_identity.serial, device.ip]`).
- **Deterministic-first, with strong/weak identifier tiers** (adopt Sentinel's taxonomy): resolve on strong
  IDs (device serial, MAC, CIP identity object) exactly; treat weak IDs (IP, hostname, unit-ID-alone) as
  *probabilistic/composite* with **temporal-validity windows** so a DHCP-reassigned IP doesn't merge two
  assets. The §12.1 planner already expands a pivot into a disjunction of equality predicates — add the
  strong/weak weighting + a time-bound binding table for IP↔asset.
- **OT identity is prism's own concern.** Because OCSF has no OT entity model and SIEM models map poorly,
  prism's registry + §17.12/§17.13 dissector (which already emits unit-ID/station-address/CIP-identity as
  native fields) together ARE the OT entity layer. This is a *defining*, not *adopting*, posture — own the
  spec surface (ties to §13.6 native-schema-on-read + ocsf#1515 watch).
- **Flow-vs-asset distinction held:** Community ID stays the *flow/session* key (sub-thread 4 pin linkage);
  the entity registry is the *asset* layer. Do not conflate them.

### 6.3 Residual open questions
- Registry config vs code (§12.1 open Q): TOML strongly preferred (dogfood), but does probabilistic scoring
  need code? (Lean: deterministic + temporal-window in TOML; defer ML-probabilistic to later.)
- Sub-device identity model: is a PLC's `(IP, unit-ID)` ONE entity with sub-units, or N entities? (No
  standard — prism decides; affects §12.1 entity_type taxonomy and OT detection §14.6.)
- Temporal-validity store: where do IP↔asset bindings live — RocksDB column family, derived at query time
  from the cache window, or a small persistent registry? (RocksDB-native, but hot vs durable like sub-thread 3.)
- Cross-site identity: same physical OT vendor model at two sites — same `entity_type`, distinct instances;
  does the registry need site-scoping? (Almost certainly yes; ties to residency scope sub-thread 7.)

---

## 7. Sub-Thread 6 — Per-Satellite Edge Compute Budget & Backpressure Governance

### 7.1 Prior art
- **Canonical drop metrics** (the measurement layer is mature): Zeek **`capture_loss`** (gap-based),
  Suricata **`capture.kernel_packets` / `capture.kernel_drops`** (AF_PACKET & PF_RING) + **TCP data-gap
  counters** as a higher-layer missing-data indicator [Zeek capture-loss; Suricata perf/stats docs].
- **Worker sizing rules of thumb**: Zeek ≈ **250 Mbps/core** for mixed workloads → ~4 cores/Gbps,
  ~40 cores/10Gbps (heavily caveated, workload-dependent) [Zeek cluster docs]. Suricata runmodes + worker
  scaling similar.
- **Capture plumbing / where drops occur**: AF_PACKET (TPACKET_V3, fanout), PF_RING / PF_RING ZC, DPDK,
  NIC RSS + ring sizing — **kernel ring overflow is the drop point**; larger rings absorb bursts; slow
  analysis backpressures the ring → drops [Suricata AF_PACKET; PF_RING; DPDK docs].
- **Passive-monitoring constraint (the crux)**: a SPAN/TAP **cannot be backpressured** — you can't slow the
  wire. The only overload responses are (a) bigger buffers, (b) more workers/cores, (c) **load-shedding**
  (sample/drop selectively), (d) reduce per-packet work [synthesis].
- **Load-shedding theory**: Aurora / Borealis / window-aware shedding — principled drop/sample/approximate
  under overload; directly applicable as the *graceful-degradation toolbox* for the analysis layer
  [Aurora; Borealis].
- **Resource isolation**: cgroups v2 CPU/mem/IO limits + CPU pinning + IRQ steering + NUMA-conscious config
  can **prioritize the capture path over analysis** so dissection/operator can't starve ingestion
  [cgroups v2 docs; Suricata-at-100Gbps guides].
- **Explicit gap**: **no integrated backpressure-governance** that coordinates capture-thread + dissection +
  windowed operator + buffer on one box, nor a fleet-wide budget regime tying these signals together
  [synthesis].

### 7.2 Recommendation / lean (informs the OT-layer Satellite §3.2 + §17.12/§17.7 co-residence)
- **Protect the capture path FIRST — make it a hard isolation invariant.** Pin capture to dedicated cores
  via cgroups v2 + CPU affinity + IRQ steering; size NIC rings for burst absorption. The dissector
  (§17.12), windowed operator (§17.7), and packet buffer (§17.6) run in a **lower-priority cgroup** that can
  be CPU/IO-throttled but can NEVER starve capture. This is the single most important rule when three heavy
  consumers share a box.
- **Shed analysis before dropping packets** (ordered degradation): under pressure, degrade in order —
  (1) shed/sample the *windowed operator* work (cheapest to lose, recomputable per sub-thread 3),
  (2) reduce dissector depth (parse fewer protocols / shallower), (3) only as last resort accept capture
  loss. Adopt Aurora/Borealis window-aware shedding for the operator tier.
- **Surface the canonical signals upward as the fleet budget contract**: per-satellite **drop rate**
  (capture_loss / kernel_drops), **ring utilization**, **analysis lag** (operator backlog), **dissector
  queue depth**, **clock-sync health** (sub-thread 4). A satellite that exceeds a drop threshold is
  *over-budget* — the fleet either provisions more cores or narrows that site's grammar/operator scope
  (residency-first: narrow what's analyzed, never silently drop).
- **Budget = declarative per-satellite resource envelope** (CPU/mem/disk-IO) enforced by cgroups, with the
  capture reservation carved out first. Ties to sub-thread 7 (per-satellite config) and §17.10 Q1
  (collection locus as per-instance property).

### 7.3 Residual open questions
- Can the §17.12 dissector + §17.7 operator + §17.6 buffer realistically co-reside at OT line rates, or
  does the operator move to the hub (shed-to-hub) under load? (Edge-first default, but the honest-cost line
  flags two heavy engines on one box.)
- Shedding policy authorship: per-satellite config, or a fleet policy artifact (sub-thread 7 locus)?
- What's the SLO for capture completeness at an OT site (zero-loss mandate vs best-effort + metric)? Drives
  whether load-shedding the operator is acceptable or whether the operator must be hub-side. (§17.10 Q2
  durability contract analog for packet capture.)
- Fleet governance mechanism: how does the central plane act on an over-budget signal (auto-scale,
  scope-narrow, alert-human)? No prior art for the integrated regime — greenfield.

---

## 8. Sub-Thread 7 — Residency-Primitive Granularity + Replication-Policy Authorship Locus

### 8.1 Prior art
- **Field/column-level classification & tagging is mature** — but for **sensitivity/access/masking, NOT
  residency**: Microsoft Purview sensitivity labels + sensitive-info-types; Snowflake object tagging +
  tag-based masking; BigQuery policy tags / column-level security; AWS Lake Formation column/cell tags;
  Databricks Unity Catalog tags [Purview; Snowflake; BigQuery; Lake Formation; Unity Catalog]. Common
  pattern: **table/dataset-level coarse access + context, field-level precise classification/masking**
  (hybrid).
- **Pipeline field-level handling**: OpenTelemetry attributes/processors (OTTL), Cribl redact/mask at field
  level — tag-and-act on individual fields in-flight [OTel; Cribl].
- **Policy-as-artifact vs inline**: OPA/Rego, Kubernetes admission (Gatekeeper/Kyverno), AWS SCP / org
  guardrails — strong, consistent pattern of **policy as a SEPARATE artifact** decoupled from app/resource
  config, interpreting classification tags [OPA; Kyverno; AWS SCP].
- **Residency is the weak spot**: data-residency/sovereignty is almost universally expressed at
  **resource/bucket/region/project/account level — NOT per-field/per-attribute** [synthesis, explicit gap].
  No surveyed system has first-class per-field residency / geo-fencing — matches the chain-cache pass
  finding that residency-first per-field is genuinely rare prior art (prism is ahead of, not behind).

### 8.2 Recommendation / lean (CLEAR — answers §17.10 Q4 + Q5; aligns §17.8.2)
- **Granularity: classify per-field, but ALWAYS scoped within a per-(source-class, schema, version) table.**
  The research's clear lean: combine field-level precision with schema-scoped manageability. Vocabulary:
  a small residency-class enum per field (e.g., `raw` / `normalized` / `metadata-only`) + a region/zone tag,
  attached to the field within its `(source_class, schema, schema_version)` table descriptor (§13.6
  multi-schema). This is exactly §17.8.2's "residency-first, per-field, ordered."
- **Locus: author residency/replication policy as a DEDICATED policy-as-code artifact; reserve inline
  per-collector config for BINDING + EXCEPTIONS only.** The OPA/Kyverno/SCP pattern is decisive — core
  governance logic belongs in a separable artifact (central audit, consistency, separation of authorship:
  connector-author ≠ compliance-owner), while the collector TOML binds its fields to classification tags
  and declares local exceptions. This answers §17.10 Q5: the `{select → reduce → retain → destination →
  RESIDENCY}` rule (§17.8.2) lives in a **chain-level governance policy artifact**, not buried per-collector.
- **Transform-ordering machine-verification** (§17.8.1 F7 / §17.10 Q4): because residency is a *separate
  policy artifact* evaluated over field-level tags, the "residency enforced BEFORE destination selection"
  invariant (§17.8.2) becomes a checkable property of the policy engine — a raw-tagged field crossing a
  region boundary must be **inexpressible by construction** in the policy DSL.

### 8.3 Residual open questions
- Tag vocabulary: is `{raw, normalized, metadata-only}` × `region-tag` sufficient, or are more handling
  classes needed (e.g., `pseudonymized`, `aggregate-only`)?
- Who authors the field tags — the connector author (knows the schema) or compliance (owns the policy)?
  (Lean: connector author proposes tags in the schema descriptor; compliance owns the *policy* that acts on
  them — separation preserved.)
- Policy artifact format: extend OPA/Rego, or a prism-native declarative DSL? (Prism-native + TOML dogfood
  likely, but Rego is battle-tested for the inexpressibility/verification property.)
- Per-satellite override scope: how much can a satellite-local exception relax the central policy without
  violating the residency floor? (Floor must be non-relaxable; exceptions can only be *more* restrictive.)

---

## 9. Cross-Cutting Note — How These Interlock With §17

- **A prism-native metadata/policy layer is the common spine of 1 + 5 + 7.** A rule's *data-dependency
  manifest* (1), the *entity registry* (5), and the *residency-tag vocabulary* (7) are three faces of one
  thing: **declarative, field/attribute-scoped, schema-versioned metadata** (§13.6) that BOTH the §17.8
  chain-cache/replication policy AND the §14 detection lifecycle read. Build them as one coherent descriptor
  family, not three silos.
- **Graceful degradation is the first-class primitive across 2 + 3 + 6.** Decline cross-hop coherence →
  partial+coverage (2); recompute-cheap window state → cheap recovery (3); shed analysis before dropping
  packets → protect capture (6). All three are prism's §3.6 partial-result thesis (BC-2.01.010) extended
  from query fan-out to caching, recovery, and edge overload. The unifying rule: **degrade explicitly with a
  surfaced signal, never silently.**
- **Community ID is the connective tissue between 4 and 5** — flow/session key (4, pin linkage) vs asset
  identity (5, registry); §17.12 axis-2 already emits it. Keep the two layers distinct.
- **The §17.12 dissector + §17.7 operator co-residence (6) is the load-bearing physical constraint.** If the
  operator must shed-to-hub under OT line-rate pressure, that reshapes §17.7 Phase-2 placement and the
  §17.8 deadline model — the edge-budget sub-thread is not cosmetic, it gates where heavy analysis can live.
- **Sub-thread 7's locus decision (policy-as-artifact) is the home for sub-thread 4's pin policy and
  sub-thread 6's shedding policy too** — all three are residency/handling decisions that belong in the
  chain-level governance artifact, evaluated over field/asset/flow metadata.

## 10. Consolidated Honest-Cost Line

Adopting these leans adds, net: **four declarative metadata/policy surfaces** — (i) per-rule data-dependency
manifest + tri-state enablement (1); (ii) canonical result-cache key + unified coverage-metadata schema (2);
(iii) prism-native entity registry with strong/weak + temporal-validity (5); (iv) per-field residency-tag
vocabulary scoped within (source-class, schema, version) tables + a dedicated policy-as-code artifact (7) —
plus **two operational disciplines** — capture-path-first cgroup isolation with ordered analysis-shedding
(6), and a severity-tiered pin-retention policy engine (4) — plus **one engine design** — at-least-once,
two-tier (recompute-cheap window vs durable detection) state for the continuous operator (3). **No new
datastore; all RocksDB-native + TOML-spec-driven**, consistent with prism's ephemeral / federated /
residency-first / OCSF+native thesis. The dominant cost risk is **engine multiplicity**: if §17.7's operator
lands alongside §17.12's dissector and the §6 entity registry, prism owns three heavy native engines — the
honest sequencing question (which the human owns) is feature-ORDER, not feature-completeness. The single
biggest *novelty* burden is OT: sub-threads 1 and 5 have **no standard to adopt** (OCSF has no OT entity
model and no cross-event identity), so prism is *defining* that surface, not borrowing it.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 7 | Deep multi-source synthesis (reasoning_effort: high) — one pass per sub-thread: detection-portability governance; query-result-cache coherence; checkpoint cadence/recovery isolation; packet-pin policy; entity-resolution incl. OT; edge budget & backpressure; residency granularity + policy locus. Each returned 90k–105k chars of cited synthesis. |
| Perplexity perplexity_ask | 1 | ≤3-sentence cross-check: does OCSF (through 1.3.x) define a canonical cross-event device/asset identity (confirmed: no). |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Context7 | 0 | No library-API lookups needed (architectural/prior-art research, not API usage). |
| Tavily (all variants) | 0 | Perplexity deep-research provided sufficient multi-source grounding; cross-check via perplexity_ask. |
| WebFetch / WebSearch | 0 | — |
| Read | 4 | matured-vision §12.1/§13.6/§14/§17 grounding; first ~67k chars of sub-thread 1 & 2 result files. |
| Grep | ~20 | Targeted extraction of abstract/verdict/gap/recommendation passages from the 5 oversized single-line result files (3–7) that exceeded the read cap. |

**Total MCP tool calls:** 8 (7 perplexity_research at reasoning_effort=high + 1 perplexity_ask).
**Training data reliance:** low — all seven sub-threads grounded in fresh deep-research passes with inline
source markers; the one cross-check (OCSF identity) used an independent Perplexity call with citations.
Items resting on model knowledge or flagged inconclusive are marked **[model-knowledge]** / **[INCONCLUSIVE]**
inline. Read-coverage caveat stated in the front-matter: sub-threads 3–7 were mined via targeted Grep of
oversized result files rather than full line-by-line reads, so quotation is limited to extracted passages.

*2026-06-26 side-analysis capture; do_not_execute; does not modify vision/specs. Sources: 7× Perplexity
sonar-deep-research (reasoning_effort=high) + 1× perplexity_ask, 2026-06.*
