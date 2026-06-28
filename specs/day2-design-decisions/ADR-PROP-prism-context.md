---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C12-1: Storage architecture — two-layer maintained + tiered (indradb graph + usearch hot-ANN + lancedb cold-on-disk)"
  - "ADR-PROP-C12-2: Embeddings — on-box in-process (fastembed/ort default; candle pure-Rust fallback for air-gap audit)"
  - "ADR-PROP-C12-3: Entity resolution — deterministic-only auto-merge on strong IDs; suspected-links for fuzzy; temporal validity intervals; security-reviewer gate"
  - "ADR-PROP-C12-4: Retrieval — Perplexity-style hybrid multi-stage + mandatory citations"
  - "ADR-PROP-C12-5: GraphRAG phasing — Phase 1 local-search (Entity 360) ships first; Phase 2 global community-summarization committed"
  - "ADR-PROP-C12-6: Deployment — embedded air-gap-first universal default; deferred central-tier option (Apache AGE + pgvector, Postgres-colocated) recorded explicitly"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Side-analysis C12 capture; human-confirmed decisions 2026-06-27 session.
  Research basis: research/prism-context-kg-vector-2026-06-27.md (three perplexity_research
  sonar-deep-research calls covering Perplexity AI retrieval architecture, GraphRAG + KG/vector
  hybrid for security entities + OCSF + entity resolution + Entity 360, and Rust-native graph/
  vector/embedding crates + air-gap fit; 12 live crates.io version-verifications 2026-06-27;
  1 perplexity_ask for GraphRAG Leiden/local-search confirmation).
  Aletheon reference: /Users/jmagady/Dev/aletheon_2/spike/init-db.sql +
  spike/docs/aletheon-vision.md — real spike memory design (Apache AGE + pgvector in one
  PostgreSQL; OT asset graph; control/process edges; AROS table; institutional-memory thesis).
  Does NOT modify live ADR files, ARCH-INDEX.md, STATE.md, SESSION-HANDOFF.md, or any
  live factory artifact.
traces_to:
  - matured-vision-day2-requirements.md §16 (Prism Context — day-2 component)
  - matured-vision-day2-requirements.md §16.4 (C12 decisions log entry)
  - day2-design-decisions/ADR-PROP-ml-behavior-analytics-depth.md (C7 ModelBackend — fastembed sits on ort; candle shared backend)
  - day2-design-decisions/ADR-PROP-storage-engine-taxonomy.md (C1 — RocksDB hot tier; Iceberg cold tier; hot→cold tiering pattern)
  - day2-design-decisions/ADR-PROP-satellite-mesh.md (C2 — AD-017 satellite-local credential resolution; per-tenant isolation; residency invariant)
  - day2-design-decisions/ADR-PROP-dynamic-schema-connectors.md (C4 — OCSF-normalized layer as graph input; boundary-normalization chokepoint)
  - epics E-PRISM-CONTEXT-001
  - research/prism-context-kg-vector-2026-06-27.md (primary research basis)
  - CLAUDE.md (#[non_exhaustive] discipline; AD-017 AI-opaque credentials; RocksDB CFs; OrgSlug per-tenant isolation; SAP-1 structured event catalog; error taxonomy)
---

# ADR-PROP — Prism Context: Knowledge Graph + Vector + Entity 360 (C12)

> **STATUS: DECIDED 2026-06-27 (human).** This is a CAPTURE artifact for the side-analysis C12
> program. `do_not_execute: true`. Real ADR numbers and formal ARCH-INDEX.md rows are deferred to
> the morph execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

> **Research basis:** `research/prism-context-kg-vector-2026-06-27.md` — three
> `perplexity_research` (sonar-deep-research) calls covering:
> (1) Perplexity AI retrieval and knowledge architecture in depth;
> (2) GraphRAG + KG/vector hybrid for security entities + OCSF + entity resolution + Entity 360;
> (3) Rust-native graph/vector/embedding crates + air-gap deployment fit.
> Plus 12 live crates.io version-verification WebFetch calls, all performed 2026-06-27.
> Plus `aletheon_2` spike read (`spike/init-db.sql` + `spike/docs/aletheon-vision.md`).

> **Aletheon spike captures a real reference memory design — not thin pitch material.**
> The aletheon_2 spike implements Apache AGE (graph) + pgvector co-resident in ONE PostgreSQL;
> an OT asset graph (`create_graph('ot_assets')`); graph edges = control/process relationships
> (CONTROLS, MONITORS, CONNECTS_VIA) with asset criticality and `network_zone=Purdue level`;
> `assets.description_embedding vector(1536)` for semantic search; an `aros` table (Actions /
> Recommendations / Observations) with dual-audience text, AI provenance, and ack/resolve
> workflow. Vision: "Institutional Memory — the system learns your environment, your patterns,
> your preferences." The `aros` table is a direct reference input for C15 (SOAR ARO model) —
> NOT captured here; cross-linked to C15 as banked input.

> **Settled decisions NOT relitigated in this capture.**
> The per-tenant isolation + AD-017 AI-opaque-data posture + embedded air-gap-first defaults are
> HARD invariants from earlier decisions. The aletheon "community defense" cross-client learning
> is explicitly NOT pulled (collides with per-tenant isolation + AD-017).

---

## Context

Prism already produces OCSF-normalized telemetry at the adapter boundary. The C12 Context engine
builds a graph + vector knowledge layer ON TOP of that normalized layer — "vendor-first / build-on-
normalized-output" (the aletheon thesis applied to Prism's existing OCSF substrate). This gives
Prism Entity 360 query capability (entity-neighborhood graph traversal + vector similarity +
LLM-synthesized cited answer), the same Perplexity-style answer-engine pattern that Perplexity
uses for web queries but applied to per-tenant security telemetry with mandatory inline citations
and strict per-tenant isolation.

Six implementation questions drove the decisions:

1. **D-C12-1 / Storage:** What graph DB and vector DB fit the embedded air-gap + RocksDB substrate?
2. **D-C12-2 / Embeddings:** How are embeddings generated without telemetry leaving the box?
3. **D-C12-3 / Entity resolution:** What is the merge policy that avoids catastrophic over-merge?
4. **D-C12-4 / Retrieval:** What is the multi-stage retrieval + synthesis pipeline?
5. **D-C12-5 / GraphRAG phasing:** Which GraphRAG search modes ship first vs later?
6. **D-C12-6 / Deployment:** What is the deployment default and what is the server-backend escape hatch?

---

## Decision Ledger

### D-C12-1 — Storage: Two-Layer Maintained + Tiered

**DECIDED 2026-06-27 (human). Resolves the graph + vector storage substrate question.**

Two co-located layers, both embedded and air-gap-capable:

**Graph layer — `indradb` 5.0.0 (RocksDB-backed):**

- Pure-Rust graph DB. RocksDB-backed, meaning the graph co-locates with Prism's existing 19-CF
  RocksDB substrate. Actively maintained (last published 2025-08-16). [REGISTRY — crates.io 2026-06-27]
- MPL-2.0 license.
- Graph layer responsibility: entity nodes + typed edges + temporal validity intervals (host ↔ user
  ↔ IP ↔ process ↔ alert ↔ asset + control/process edges: CONTROLS / MONITORS / CONNECTS_VIA +
  Purdue-zone as a first-class entity attribute per aletheon reference). Per-tenant partitioned
  via graph namespace (one graph namespace per OrgSlug).

**REJECTED: `cozo` 0.7.6** — the single engine that does graph + vector + Datalog in one embeddable
store, last published 2023-12-11 (~2.5 years stale as of 2026-06-27). Maintenance staleness risk
is unacceptable as a primary dependency for a security product. Record considered-and-rejected:
if `cozo` upstream activity resumes and passes a maintenance-health gate, it may be reconsidered
at morph (see OQ-C12-4).

**Vector layer — `usearch` 2.25.3 hot-in-memory ANN + `lancedb` 0.30.0 on-disk COLD tier:**

- `usearch` 2.25.3: Rust bindings to the USearch HNSW engine. Actively maintained (last published
  2026-05-24). INT8/binary quantization built-in — directly addresses the 512MB process / 200MB
  per-query memory budget. The Perplexity compact-embedding tactic applied to the hot tier.
  [REGISTRY — crates.io 2026-06-27]
- `lancedb` 0.30.0: Embedded, on-disk columnar (Arrow/Lance format) vector DB. Actively maintained
  (last published 2026-05-28). Apache-2.0 license. On-disk cold vector tier — maps to the
  hot→Iceberg-cold tiering from C1 storage taxonomy (hot recent-event embeddings in usearch RAM;
  historical embeddings persisted on-disk in lancedb). [REGISTRY — crates.io 2026-06-27]
- Per-tenant partitioned: separate vector collections per OrgSlug. No cross-tenant similarity search.

**Why two layers, not one:** `indradb` is graph-only (no native vector); `cozo` (single engine) is
rejected on maintenance grounds. The two-layer architecture is more moving parts than `cozo` but
uses two actively-maintained crates vs one stale one. Safer for a production security product.

**HONEST COSTS:**

- More integration surface than a single combined engine would provide.
- `usearch` is HNSW in-memory — index must be loaded at boot, persisted periodically. Boot-time
  index load adds startup latency; must bound the in-memory index size (INT8/binary quantization
  is the primary lever — see D-C12-2).
- `lancedb` Lance format is not Apache Iceberg. The cold vector tier is a SEPARATE format from the
  Iceberg cold tier (C1/C5). Two distinct cold-tier formats coexist (Iceberg for structured OCSF
  events; Lance for embeddings). This is acceptable given the different access patterns (columnar
  queries vs ANN search) but must be documented clearly to avoid ops confusion.

---

### D-C12-2 — Embeddings: On-Box In-Process, Air-Gap-First

**DECIDED 2026-06-27 (human). Resolves how embeddings are generated without raw telemetry transiting AI context.**

**Primary: `fastembed` 5.17.2 on `ort` 2.0.0-rc.12 (C7 ModelBackend):**

- `fastembed` 5.17.2: fastembed-rs, the highest-level option. ONNX-based, runs sentence-embedding
  models (BGE-small/base, all-MiniLM, nomic-embed, multilingual-E5) locally on CPU. Actively
  maintained (last published 2026-06-15). Bundles tokenization + model download (must pre-stage
  models for air-gap). [REGISTRY — crates.io 2026-06-27]
- `fastembed` sits on `ort` 2.0.0-rc.12 — the SAME ort already committed in C7's ModelBackend
  (D-C7-2). This creates a shared dependency rather than a second ort pin. ort is still RC; pin
  exactly; budget for API churn (PIV-C12-1 mirrors PIV-C7-1).
- Embeddings computed in-process, on-box. Raw telemetry is vectorized locally and NEVER transits
  AI context — satisfies AD-017 + C16 masking invariant (PIV-C12-2).

**Fallback: `candle-core` 0.11.0 (pure-Rust, no ONNX/C dependency):**

- `candle-core` 0.11.0: HuggingFace pure-Rust ML framework, also named in C7 (D-C7-2). Can run
  BERT/BGE embedding models natively without an ONNX dependency. Actively maintained
  (last published 2026-06-26). [REGISTRY — crates.io 2026-06-27]
- Use when the ONNX/C dependency of ort is unacceptable for air-gap audit (e.g., high-assurance
  customer environments requiring pure-Rust supply chain). More implementation effort than
  fastembed; the clean air-gap supply chain story justifies it for those deployments.

**Final model selection pending benchmark (OQ-C12-1):**

The specific embedding model (BGE-small-en-v1.5, all-MiniLM-L6-v2, nomic-embed-text, multilingual-E5)
is NOT fixed by this ADR-PROP. The choice trades off recall quality vs memory footprint vs latency.
A perf/recall benchmark on representative security telemetry (OCSF events, finding summaries,
asset descriptions) is required before the model is pinned (OQ-C12-1). The winning model must fit
within the usearch hot-index memory budget after INT8/binary quantization.

**Air-gap packaging invariant (PIV-C12-3):**

Models MUST be pre-staged in the deployment bundle. Default download paths from HuggingFace MUST be
disabled in production builds. This is a packaging constraint, not an inference constraint — the
inference path is identical whether models are downloaded or pre-staged.

---

### D-C12-3 — Entity Resolution: Deterministic-Only Auto-Merge + Suspected-Links

**DECIDED 2026-06-27 (human). Resolves entity resolution merge policy and the over-merge risk.**

**Policy: auto-merge ONLY on strong identifiers:**

| Identifier strength | Examples | Auto-merge? |
|---|---|---|
| Strong IDs | SID, UUID, stable asset UUID, MAC address | YES — deterministic auto-merge |
| Weak/fuzzy | Name similarity, co-occurrence, shared NAT IP, shared service account | NO — `suspected` edge only, NEVER auto-merged |

**Rationale for the strong-ID-only line:** over-merging is catastrophic for security — merging two
distinct users because of a shared NAT IP or service account attributes one person's lateral-movement
activity to another, producing false findings and missed detections. The over-merge cost is
asymmetric: a split (false-negative) degrades Entity 360 completeness but is recoverable; a
merge (false-positive) introduces an active security error that propagates through all downstream
analytics.

**Temporal validity intervals on identity edges:**

All identity bindings carry validity intervals: `(entity_id, identifier, valid_from, valid_until)`.
DHCP IP rotation is the canonical example — "host A had IP X between t0 and t1." A query at time
T only applies bindings valid at T. Edge schema must enforce `valid_until` is either `null` (still
valid) or a concrete timestamp.

**Suspected-link surface:**

Weak-signal matches produce a `suspected` typed edge in the graph (NOT a merge). An analyst can
inspect suspected links, promote them to confirmed merges with an audit record, or dismiss them.
The S3 agent (§15.8 / C12 consumer) can surface suspected links for analyst review but CANNOT
auto-promote them.

**Strictly per-tenant:**

Entity resolution runs ONLY within a single tenant's graph namespace. Cross-tenant entity resolution
is structurally prohibited — there are no graph edges between different `OrgSlug` namespaces.

**Security-reviewer sign-off required (PIV-C12-4):**

The auto-merge policy (which identifiers qualify as "strong IDs") must be reviewed by the
security-reviewer agent before implementation. The strong-ID set should be conservative and errs
toward NOT auto-merging when uncertain. Any identifier that can be spoofed, shared, or rotated
(e.g., IP addresses, hostnames in some environments) must NOT be in the strong-ID set unless the
resolver has already validated it against a known-authoritative source.

---

### D-C12-4 — Retrieval: Perplexity-Style Hybrid Multi-Stage + Mandatory Citations

**DECIDED 2026-06-27 (human). Resolves the retrieval + synthesis pipeline shape.**

Multi-stage pipeline (the Perplexity pattern applied to security telemetry):

1. **Lexical / structured filter:** exact-match on IOC identifiers (IP, hash, hostname, PID, CVE,
   rule ID). Mandatory — security telemetry is dense with exact identifiers where semantic search
   would miss critical exact-string matches. BM25/TF-IDF-style lexical index alongside the ANN.
2. **Graph-neighborhood expand:** traverse `indradb` entity neighborhood from the filter results
   (GraphRAG local-search pattern). Expands from a single entity to the full related-entity
   subgraph within a configurable hop count.
3. **Vector similarity search:** `usearch` ANN over the embedding space (recent/hot tier) + fallback
   to `lancedb` cold tier for historical semantic similarity. Finds semantically similar events /
   findings / asset descriptions beyond the exact-match and graph neighborhood.
4. **Re-rank:** multi-signal scoring. Security-domain ranking signals (in approximate priority order):
   - Detection severity / confidence
   - Asset criticality / Purdue-zone (OT-elevated priority for OT assets)
   - Threat-intel credibility (C11 Prism Intel integration)
   - Event recency (temporal decay — more recent events weighted higher for live incidents)
   - Entity relevance to the query subject
5. **LLM synthesis + mandatory inline citations:** route to C7 ModelBackend (fast embed/rank model
   for simple Entity 360 lookups; reasoning model for deep correlation / incident narrative). Every
   LLM-surfaced claim MUST carry an inline citation — OCSF event ID / detection rule ID / asset
   canonical ID. Citations are the trust contract for LLM-consumed security data (the Perplexity
   invariant applied to security). A claim without a citation is a pipeline defect, not a
   style issue.

**Hybrid lexical+semantic is non-negotiable:** neither lexical alone (misses semantic similarity
across normalized forms) nor semantic alone (misses exact IOC/identifier match) is sufficient.
The mandatory combination is the Perplexity pattern that makes answer-engine trustworthy over a
domain with exact identifiers (IP, hash, hostname).

---

### D-C12-5 — GraphRAG: Phase 1 Local-Search Ships First; Phase 2 Global Community-Summarization Committed

**DECIDED 2026-06-27 (human). Resolves the GraphRAG scope and phasing.**

**Phase 1 — Local-Search (Entity 360 query) — SHIPS FIRST:**

Entity-neighborhood graph traversal + vector similarity = the GraphRAG "local search" mode. This
is the Entity 360 query: given an entity (host, user, IP, alert), expand its neighborhood in the
`indradb` graph, retrieve semantically-similar events from `usearch`/`lancedb`, re-rank, and
synthesize a cited answer. This mode is the primary day-2 deliverable for C12 and is computationally
cheap relative to Phase 2.

**Phase 2 — Global Community-Summarization (Hierarchical Leiden + LLM community summaries) — COMMITTED, phased:**

Full GraphRAG global-search requires:
1. Hierarchical Leiden community detection across the entity graph (groups related entities into
   "incidents" / "campaigns" — structurally the aletheon ARO-collapse behavior).
2. LLM pre-summarization of each community into a "community report" that can be retrieved for
   corpus-wide "sense-making" queries ("summarize all activity related to this campaign").

This is COMMITTED (not merely "deferred to roadmap") — it is a planned Phase 2 within day-2 scope.
The cost is explicitly acknowledged: Leiden community detection + LLM community-summary pre-compute
must re-run as telemetry streams in. The recompute cadence (triggered vs scheduled vs incremental)
is an open question (OQ-C12-2) and must be cost-bounded before Phase 2 implementation.

**Why not Phase 2 first:** local-search (Entity 360) delivers standalone value and can ship without
the Leiden/community-summary pre-compute pipeline. Phase 1 is a strict prerequisite for Phase 2
(the graph + vector layers must be operational before community detection makes sense).

---

### D-C12-6 — Deployment: Embedded Air-Gap-First Universal Default; Deferred Central-Tier Option Recorded

**DECIDED 2026-06-27 (human). Resolves deployment model and records the server-backend escape hatch.**

**Universal default: embedded single-binary, air-gap-first:**

`indradb` (RocksDB-backed), `usearch` (in-memory), and `lancedb` (on-disk) are all embeddable
in-process. The Context engine runs inside the Prism binary with no external service dependency.
This is the MANDATED default for all three deployment models (SaaS / MSSP-managed / Client-managed)
because:
- Air-gap and client-managed deployments require no-network-service default.
- Single-binary operational simplicity is a product differentiator.
- Per-tenant isolation is structural in the embedded model (graph namespaces + vector collections
  per OrgSlug within a single process).

**Server-based options are EXCLUDED as defaults:**

Neo4j, Memgraph, and Qdrant-server all require a separate running service, violating the
single-embeddable-binary air-gap default. They are NOT built speculatively.

**DEFERRED CENTRAL-TIER OPTION — record explicitly (DO NOT build speculatively):**

**Apache AGE (graph) + pgvector co-resident in ONE PostgreSQL** is the recorded concrete
server-backend escape-hatch for the CENTRAL deployment tier ONLY.

Rationale for recording this specific option:
- Validated by the aletheon spike (`spike/init-db.sql`): Apache AGE + pgvector in one PostgreSQL
  instance is a proven pattern for the OT asset graph + semantic search use case.
- Prism already bundles PostgreSQL for the central deployment tier (C1 storage taxonomy). Colocating
  the graph + vector extension in the ALREADY-BUNDLED PostgreSQL avoids a second service entirely.
- Apache AGE brings the Cypher/openCypher graph query surface; pgvector brings `vector(N)` column
  type + IVFFLAT/HNSW index; the two coexist in one Postgres instance per the aletheon spike.

This option is NOT triggered by architectural desire — it is an ESCAPE HATCH if the embedded
two-layer stack proves insufficient for the central-tier graph/vector workload under production load
(OQ-C12-3). Evaluation criteria: concurrent tenant graph traversal performance, Leiden community
detection at scale, and operational complexity vs the embedded default.

**DO NOT build AGE+pgvector central-tier support speculatively.** Revisit only if a concrete
central-tier performance or operational need emerges. The embedded edge/satellite decision is
UNCHANGED regardless of the central-tier escape hatch.

**ADS Option 3 alignment note (P-ADS-04; ADS conformance 2026-06-27; ripple-audit P3 correction):**
The Option 3 lock (Tenant-Keyed-Central-Persistence, locked 2026-06-27) does NOT accelerate
the AGE+pgvector central-tier escape hatch. Option 3 specifies that derived results persisted
at Central use RocksDB (hot) and Iceberg (cold), NOT PostgreSQL (P-ADS-04 boundary: "PostgreSQL
is CONTROL-PLANE only — never for query result caching"). The AGE+pgvector central-tier option,
if ever triggered (OQ-C12-3), would colocate in the BUNDLED PostgreSQL control-plane instance —
which is consistent with C1-D4 (case management on bundled PostgreSQL) but remains a DEFERRED
escape hatch for the graph/vector query layer, not a substitute for the Option 3 RocksDB/Iceberg
Central cache architecture. The distinction matters: C12 graph/vector traversal results surfaced
at Central via Entity 360 WOULD be cached under Option 3 CMEK (P-ADS-04); the AGE+pgvector
question is specifically about where the GRAPH SUBSTRATE lives, not where derived query results
are cached. These are orthogonal concerns.

**"Central blind" nuance (P-ADS-02; ADS conformance 2026-06-27):** Entity 360 query results and
GraphRAG community summaries surfaced at Central are operator-invisible — the operator has zero
at-rest read access (P-ADS-02 Operator-Zero-Access-At-Rest). Authenticated clients view their
own Entity 360 data through their Central session. Raw telemetry, graph topology, and vector
indices NEVER leave the satellite/edge (PIV-C12-2, PIV-C12-5). (ADS conformance 2026-06-27)

---

## Entity 360 — Seven-Part View

The Entity 360 view is the primary query surface delivered by the C12 Context engine. Each part
maps to a specific layer:

| Part | Contents | Layer |
|---|---|---|
| 1. Identity panel | Canonical entity ID + all resolved aliases/identifiers + binding confidence + temporal validity | Graph — `indradb` entity nodes + identity edges |
| 2. Timeline | Chronological normalized OCSF activity for the entity, with time-windowed identity context | Graph + OCSF event store (hot tier) |
| 3. Relationship graph | Host ↔ user ↔ IP ↔ process ↔ alert ↔ asset; CONTROLS / MONITORS / CONNECTS_VIA control/process edges; Purdue-zone / ot_level_0/1/2/dmz/it as first-class entity attribute | Graph — `indradb` typed edges with temporal validity |
| 4. Explainable risk score | Composite severity/criticality/anomaly score with cited contributing OCSF events + detection rules | Fused retrieval + C7 ML (ANOMALY_SCORE primitive) |
| 5. Exposures | Vulnerabilities, misconfigurations, unpatchable-flag, OT-specific risk flags (ties C20 NERC CIP) | Graph — asset attributes + C11 Prism Intel enrichment |
| 6. Related findings / similar entities | Entities that look like this one / incidents like this | Vector — `usearch`/`lancedb` semantic similarity |
| 7. Operational / business context | Asset → process → mission edges (what does this entity support) | Graph — `indradb` operational-context edges |

**Purdue-level / network-zone as first-class entity attribute:** OT assets carry `ot_level`
(0/1/2/DMZ) or `network_zone` (OT/IT boundary, DMZ) as a typed attribute on the asset node.
This enables queries like "all assets at Purdue Level 2 involved in alerts in the last 24h" and
feeds C20 NERC CIP compliance queries. The aletheon spike's `network_zone=Purdue level` on asset
nodes is the direct reference.

---

## Aletheon Spike — What Prism Pulls vs Does Not Pull

### What Prism pulls from the aletheon spike

| Aletheon element | Prism C12 application |
|---|---|
| **Apache AGE + pgvector in one PostgreSQL** | Validates graph + vector co-residence pattern. Recorded as the DEFERRED central-tier substrate option (D-C12-6). NOT built speculatively. |
| **Control/process edges** (CONTROLS, MONITORS, CONNECTS_VIA) | Adopted as first-class typed edges in the `indradb` graph schema (Entity 360 part 3). |
| **Purdue-zone / `network_zone` as asset attribute** | Adopted as first-class entity attribute (Entity 360 part 3, OT filter queries, C20 tie). |
| **Institutional memory thesis** | The C12 Context engine learns the tenant's environment over time, per-tenant. This is the product framing for the KG + vector layer. |
| **OT asset graph as "next step from lists+configs"** | Validates the C12 thesis that entity 360 + KG is the natural evolution of Prism's existing per-entity model. |
| **NL investigation over the graph** | Delivered via S3 agent runtime (§11.3 / day-2-design-decisions/ADR-PROP-s3-agent-runtime.md) consuming C12 as the Context substrate. |
| **`aros` table (Actions / Recommendations / Observations)** | NOT captured here. This is a direct reference input for **C15 (SOAR ARO model)** — banked as C15 input. Cross-link to C15; do NOT model the ARO layer in C12. |

### What Prism does NOT pull from aletheon

| Aletheon element | Reason not pulled |
|---|---|
| **Community defense / cross-client learning** | Collides with Prism's per-tenant isolation hard invariant + AD-017 AI-opaque data posture. Structurally incompatible. |
| **OT-specific Claroty/Dragos/Nozomi vendor-first ingestion** | Prism already normalizes at the adapter boundary (OCSF). The aletheon "vendor-first" stance = Prism's existing OCSF normalization applied at the KG input layer. |
| **FAIR risk scoring by default** | Domain-specific to aletheon's OT MSSP positioning. C12 provides the cited-risk-score primitive; FAIR scoring is a downstream PO decision on scoring formula, not a C12 architectural commitment. |

---

## Invariants (PIV-C12-*)

| ID | Invariant |
|---|---|
| **PIV-C12-1** | `ort` RC → stable: at morph time, check whether stable `ort` 2.0.x has been released (currently 2.0.0-rc.12 as of 2026-06-27). Mirrors PIV-C7-1 from C7. If stable exists, pin stable; if still RC, budget for API churn. |
| **PIV-C12-2** | **In-process on-box embeddings (invariant).** Raw telemetry, OCSF events, and asset descriptions MUST be vectorized in-process on-box via `fastembed`/`ort` or `candle`. Embedding model inference MUST NOT be routed to an external inference API. Any code path that sends telemetry content to an external embedding endpoint is a **P1 violation** of AD-017 + C16 masking. |
| **PIV-C12-3** | **Air-gap model pre-staging (invariant).** Production builds MUST disable default HuggingFace model download paths. Embedding models MUST be pre-staged in the deployment bundle or loaded from a local model registry. The `fastembed` default-download behavior MUST be patched/configured out before any air-gap-targeted release. |
| **PIV-C12-4** | **Security-reviewer sign-off on strong-ID list.** The set of identifiers that qualify for deterministic auto-merge (D-C12-3) MUST be reviewed by the security-reviewer agent before implementation. Any identifier that can be spoofed, shared across entities, or rotated (e.g., raw IP addresses in dynamic environments) MUST NOT be in the strong-ID set without explicit justified exception. |
| **PIV-C12-5** | **Strictly per-tenant graph/vector/resolution (invariant).** No cross-tenant graph edges, no cross-tenant vector similarity search, no cross-tenant entity resolution. `OrgSlug`-keyed namespaces are the isolation boundary. This is structural, not configuration. A code path that queries across `OrgSlug` namespaces without explicit multi-tenant aggregation authorization is a **P1 violation**. |
| **PIV-C12-6** | **Every LLM-surfaced claim carries an OCSF-event / rule / asset citation.** A synthesized answer that asserts facts without inline citations is a pipeline defect. The mandatory-citation contract (D-C12-4) must be enforced at the LLM synthesis output boundary — the synthesis prompt must instruct citation and the output must be validated for citation presence before delivery to the agent harness. |
| **PIV-C12-7** | **Auto-merge only on strong identifiers (invariant).** A code path that auto-merges entity identities on weak signals (name similarity, co-occurrence, shared transient IP) without an explicit analyst confirmation step is a **P1 violation** of D-C12-3. Weak-signal matches MUST produce `suspected` edges only. |

---

## Open Questions (Architect / Morph-Time)

| # | Question | Status | Dependency |
|---|---------|--------|------------|
| **OQ-C12-1** | Embedding model selection — benchmark BGE-small-en-v1.5 vs all-MiniLM-L6-v2 vs nomic-embed-text vs multilingual-E5 on representative security telemetry. Metrics: recall@10, latency, INT8-quantized index size within 512MB/200MB budget. Final model must be pinned before E-PRISM-CONTEXT-001 story decomposition. | Open — benchmark required before model pin | E-PRISM-CONTEXT-001 implementation milestone |
| **OQ-C12-2** | GraphRAG Phase 2 Leiden community-detection recompute cost and cadence — triggered (on entity-graph change), scheduled (hourly/daily), or incremental (delta-Leiden). Cost-bounded approach required. Community-summary LLM call count + token budget. Acceptance: community detection does not block the live query path. | Open design decision at morph | Phase 2 scoping (after Phase 1 Entity 360 ships) |
| **OQ-C12-3** | Central-tier AGE+pgvector evaluation trigger — what concrete performance or operational signal triggers the evaluation of the DEFERRED central-tier AGE+pgvector escape hatch (D-C12-6)? Should be an explicit gate: e.g., "concurrent tenant graph traversal latency exceeds Xms at Y tenants" or "embedded indradb memory footprint exceeds Zmb at central tier." Without a concrete gate, the escape hatch is never evaluated. | Open — gate criteria needed at morph | Central-tier production deployment milestone |
| **OQ-C12-4** | `cozo` upstream liveness check — if cozo 0.8.x or later is published before the C12 morph decision, reassess whether the maintenance-staleness blocking reason (D-C12-1) still applies. Cozo would simplify the architecture (one engine, not two). The evaluation must check: last commit date, open issue count, active maintainer confirmation, test coverage of the vector-over-RocksDB path. | Open — contingent on cozo upstream activity | Before C12 morph ADR finalizes storage pick |
| **OQ-C12-5** | `usearch` in-memory hot-index persistence and boot-time load strategy. The hot index must be rebuilt or reloaded at boot. For large tenants, boot-time load could be slow. Options: (a) serialize index to RocksDB CF + deserialize at boot; (b) rebuild from lancedb cold tier on first boot; (c) lazy-load (serve requests from cold tier until hot index is warm). Decision needed before E-PRISM-CONTEXT-001 Phase 1 implementation. | Open design decision at morph | E-PRISM-CONTEXT-001 Phase 1 implementation |

---

## Downstream SAP-1 Obligations (Not Actioned Here)

Several event types implied by C12 decisions will need BC-2.16.002 Canonical Structured Event
Catalog rows at morph time. Flagged here; NOT actioned (SAP-1 probe scope is per-story at
implementation time, not at ADR-PROP capture time).

- `event_type = "context.entity.merged"` — emitted on deterministic auto-merge; fields: tenant_id,
  canonical_entity_id, merged_identifier, identifier_type, merge_basis (strong-ID name).
  Audit role = entity-resolution audit trail; recurrence = per merge.
- `event_type = "context.entity.suspected_link"` — emitted when a weak-signal match creates a
  `suspected` edge; fields: tenant_id, entity_id_a, entity_id_b, signal_type, confidence.
  Audit role = entity-resolution transparency; recurrence = per suspected edge creation.
- `event_type = "context.embedding.indexed"` — emitted when OCSF events are embedded and indexed;
  fields: tenant_id, event_count, model_id, tier (hot/cold), index_size_bytes.
  Audit role = context engine coverage audit; recurrence = per indexing batch.
- `event_type = "context.retrieval.synthesized"` — emitted when the LLM synthesis pipeline
  produces a cited answer; fields: tenant_id, entity_id, query_type, citation_count,
  model_backend_id, latency_ms.
  Audit role = agent-harness reasoning audit; recurrence = per Entity 360 query.
- `event_type = "context.community.detection_run"` — emitted when Leiden community detection
  completes (Phase 2); fields: tenant_id, community_count, entity_count, leiden_resolution,
  duration_ms.
  Audit role = GraphRAG Phase 2 coverage audit; recurrence = per community-detection run.

All five categories above are flagged; BC-2.16.002 catalog rows are morph-time work.

---

## Honest Costs

| Item | Cost / Risk |
|------|-------------|
| **Two-layer architecture (indradb + usearch + lancedb)** is more moving parts than `cozo` would have been. | Three crate integrations vs one. All three are actively maintained and healthy; cozo is rejected on staleness grounds. The architecture complexity is the price of using maintained dependencies. |
| **`usearch` hot-index boot-time load** (OQ-C12-5). | Large tenant indices may slow boot. Must be addressed before Phase 1 ships. Three mitigation options exist (see OQ-C12-5); none is trivially free. |
| **`lancedb` Lance format ≠ Apache Iceberg.** | Two distinct cold-tier formats coexist (Iceberg for structured OCSF events; Lance for embeddings). Not a technical problem — the access patterns differ — but must be documented to avoid operational confusion. |
| **Embedding model pinning requires a benchmark** (OQ-C12-1). | Model is NOT pinned by this ADR-PROP. Failure to run the benchmark before story decomposition will leave a version-uncertainty in the implementation. This is a morph-time execution risk. |
| **GraphRAG Phase 2 Leiden + LLM community-summary pre-compute is expensive.** | Leiden over a large security entity graph + LLM summarization per community is a real compute budget. OQ-C12-2 must cost-bound Phase 2 before scoping. Phase 1 (local-search) is unaffected. |
| **`ort` is still RC (2.0.0-rc.12 as of 2026-06-27).** | Shared with C7 risk. Pin exactly; track toward stable release. Mirrors PIV-C7-1 / PIV-C12-1. |
| **Air-gap model pre-staging is a packaging concern that must be solved before GA.** | The `fastembed` default downloads from HuggingFace. This must be disabled and models pre-staged before any air-gap delivery. PIV-C12-3 enforces this. |

---

## Alternatives Considered and Rejected

### Alternative A: Single-Engine cozo (Graph + Vector + Datalog in One Embeddable Store)

`cozo` 0.7.6 would collapse both layers (graph + vector) into a single embedded engine with Datalog
query support. Architecture-simplifying, RocksDB-backed, MPL-2.0.

**Rejected (D-C12-1) because of maintenance staleness:**
Last published 2023-12-11 (~2.5 years stale as of 2026-06-27). For a security product that must
maintain reliable supply chain auditing and patch responsiveness, a stale primary dependency is
unacceptable. If upstream cozo activity resumes, this alternative may be revisited (OQ-C12-4).
Two actively-maintained crates (indradb + usearch/lancedb) are safer than one stale crate.

### Alternative B: Server-Based Neo4j / Memgraph (Graph) + Qdrant-Server (Vector)

These provide richer ecosystems, native vector+graph hybrid queries (Neo4j), and purpose-built
vector search (Qdrant).

**Rejected as default (D-C12-6) because:**
- All three require separate running services, violating the single-embeddable-binary air-gap default.
- They are acceptable for the hosted/on-prem-with-services model if a concrete need emerges, but
  not as the universal default. The Apache AGE+pgvector escape hatch (D-C12-6) is more
  architecturally coherent for Prism's already-bundled PostgreSQL than adding Neo4j/Qdrant-server.

### Alternative C: hnsw_rs Instead of usearch for Hot ANN

`hnsw_rs` 0.3.4 is a pure-Rust HNSW implementation, MIT/Apache-2.0, maintained as of early 2026.
A valid alternative to `usearch` for the hot in-memory ANN tier.

**Not selected (D-C12-1) — `usearch` preferred because:**
- `usearch` has built-in INT8/binary quantization, directly addressing the memory budget. `hnsw_rs`
  does not document quantization.
- `usearch` is more recently maintained (2026-05-24 vs 2026-02-28).
- `hnsw_rs` is a reasonable fallback if `usearch` has unforeseen issues at morph — record as such.

---

## Ripple Effects (Must Be Picked Up at Morph Time)

| Affected area | Ripple |
|---------------|--------|
| **E-PRISM-CONTEXT-001** | This ADR-PROP defines the full architecture for Phase 1 (local-search / Entity 360) and scopes Phase 2 (Leiden + community-summary). Epic ACs must include: OQ-C12-1 embedding benchmark gated before story decomposition; PIV-C12-2/3/4/5/6/7 as pre-ship invariant gates. |
| **C7 ModelBackend (ADR-PROP-ml-behavior-analytics-depth.md)** | D-C12-2 fastembed sits on ort 2.0.0-rc.12 — the SAME ort already in C7. Must confirm shared ort pin is consistent at morph. If a stable ort 2.0.x ships, both C7 and C12 must update to the same stable version in one atomic workspace bump (PIV-C7-1 + PIV-C12-1 are paired). |
| **C1 storage taxonomy (ADR-PROP-storage-engine-taxonomy.md)** | lancedb cold vector tier adds a second cold-tier format alongside Iceberg. The C1 storage taxonomy docs must acknowledge the lancedb format and its access pattern boundary (ANN search only, NOT structured OCSF queries). |
| **C11 Prism Intel** | Entity 360 part 4 (explainable risk score) + part 5 (exposures) consume C11 Prism Intel enrichment for threat-intel credibility signal and exposure data. C11 and C12 must align on the interface: C12 asks C11 for enrichment at retrieval time; C11 returns structured OCSF-normalized results. |
| **C15 SOAR ARO model** | The aletheon `aros` table (Actions / Recommendations / Observations with dual-audience text, AI provenance, confidence/model_version, source_event_ids[], affected_asset_ids[], ack/resolve workflow) is banked as a direct reference input for C15. C15 spec writer must pull from ADR-PROP-prism-context.md §Aletheon Spike — aros cross-reference. C12 is the substrate; C15 is the workflow surface built on top of it. |
| **C20 NERC CIP / OT** | Entity 360 part 3 (relationship graph with Purdue-zone / ot_level as first-class attribute) and part 5 (OT-specific risk flags) feed C20 compliance queries. The Purdue-zone entity attribute schema must be designed in coordination with C20 at morph. |
| **BC-2.16.002 §Postconditions** | Five SAP-1 event type categories (§Downstream SAP-1 Obligations above) — morph-time BC work. |
| **S3 agent runtime (ADR-PROP-s3-agent-runtime.md)** | The S3 agent consumes C12 Context as its primary knowledge substrate for natural-language investigation ("What changed at this host?", "What else has this user done?"). The S3 agent runtime must be able to call C12 retrieval APIs at query time. Interface design is morph-time work. |
| **matured-vision §16.4** | C12 decision block appended as §16.4 bullet (2026-06-27). |

---

## References

| Document | Role |
|---|---|
| `research/prism-context-kg-vector-2026-06-27.md` | Primary C12 research basis (3× `perplexity_research` + 1× `perplexity_ask`; 12 live crates.io version-verifications 2026-06-27) |
| `/Users/jmagady/Dev/aletheon_2/spike/init-db.sql` | Aletheon spike reference — Apache AGE + pgvector schema; OT asset graph; CONTROLS/MONITORS/CONNECTS_VIA edges; description_embedding vector(1536) |
| `/Users/jmagady/Dev/aletheon_2/spike/docs/aletheon-vision.md` | Aletheon spike vision — institutional-memory thesis; ARO engine; air-gap milestone |
| `day2-design-decisions/ADR-PROP-ml-behavior-analytics-depth.md` | C7 — ort + candle backends shared with D-C12-2; PIV-C7-1 mirrors PIV-C12-1 |
| `day2-design-decisions/ADR-PROP-storage-engine-taxonomy.md` | C1 — RocksDB hot/model tier co-location; hot→Iceberg-cold pattern |
| `day2-design-decisions/ADR-PROP-satellite-mesh.md` | C2 — AD-017 satellite-local credential resolution; per-tenant residency invariant |
| `day2-design-decisions/ADR-PROP-dynamic-schema-connectors.md` | C4 — OCSF-normalized layer as graph input; boundary-normalization applies to Context layer inputs |
| `day2-design-decisions/ADR-PROP-s3-agent-runtime.md` | C3 agent runtime — primary consumer of C12 Context for NL investigation |
