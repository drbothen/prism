# Prism Context — Knowledge Graph + Vector DB + Entity Mapping + Entity 360 Expansion

**Research artifact for Day-2 Vision SIDE-ANALYSIS item C12 — Prism Context**
**Mode:** CAPTURE / research-only (`do_not_execute`). Modifies no live spec / BC / ADR / STATE.md / SESSION-HANDOFF.md.
**Date:** 2026-06-27
**Author:** research-agent (vsdd-factory)
**Scope:** How Perplexity does retrieval; KG+vector hybrid for security entities; Rust-native + air-gap-capable options (versions verified on crates.io); entity resolution; Entity 360 expansion; deployment fit relative to RocksDB hot / Iceberg cold.

> **Confidence legend used throughout:** **[VERIFIED-WEB]** = grounded in cited web source; **[REGISTRY]** = crate version confirmed against crates.io on 2026-06-27; **[MODEL]** = model knowledge / architectural reasoning, flagged explicitly; **[INFERRED]** = reasonable inference from cited facts, not directly stated by a source.

---

## 0. Aletheon Reference — What Was Found

The user asked me to read the aletheon spec first. I examined `/Users/jmagady/Dev/aletheon_2`.

**Finding:** aletheon_2 is a **BMAD-method project scaffold** (the `_bmad/` tree is the BMAD agent/workflow framework, not aletheon content). The only substantive aletheon-authored content is two pitch/vision docs under `docs/`:

- `docs/investment-committee-mssp-platform-extension.md`
- `docs/single-slide-answer.md`
- (plus a client-meeting transcript `docs/client sentiments/Chevron-meeting-dec.16.2025-transcript.md` — sentiment, not design)

**There is no technical KG/vector-DB/embedding/entity-resolution implementation spec, no Rust code, no architecture doc.** The relevant concepts are described only at the pitch level. I state this explicitly per the research mandate and proceed with the generic + cited research for the technical substance.

### What aletheon's approach actually is (from the two docs)

Aletheon frames an **OT-focused (operational-technology) early-detection-and-response platform** built on five "Disruptive Components." Two of the five are directly relevant to C12:

1. **"OT Asset Graph & Inventory" (a knowledge graph).** The explicit framing is an evolution **"from lists + configs → structured graph with relationships."** Capabilities named: Layer-2 switch intelligence, MAC/VLAN/Port correlation, passive + active OT discovery, a "relational knowledge graph," plus rollups (site → asset → vintage → vendor), heatmaps of vulnerable regions, and tagging of "unpatchable"/"high-risk" assets. Notably it advocates a **"vendor-first ingestion strategy (use Claroty/Dragos/Nozomi output as-is, extend later)."**
2. **"ARO Engine" (Actions / Recommendations / Observations).** A **graph-aware, process-informed prioritization model** that collapses ~1000 alerts → 10–20 AROs by grouping via the asset graph and prioritizing on operational/mission risk rather than raw event counts. Includes FAIR risk scoring and "process-informed risk assessment that understands OT operational context."

Supporting components: a **Natural-Language Co-Pilot** ("natural-language exploration of the asset graph," operator-first investigation like *"What changed at Substation 4?"*), and a **Community Defense** network (privacy-preserving cross-client pattern sharing) — with explicit **offline / air-gapped support** as a Month-9–12 production milestone.

### What Prism should pull from aletheon

| Aletheon concept | Transferable to Prism Context (C12) |
|---|---|
| **Asset graph as "next step from lists+configs"** | Validates the C12 thesis: Prism's existing Entity 360 + entity resolution is the "list+config" stage; the KG is the natural evolution into a relationship graph (host↔user↔IP↔process↔alert). |
| **Vendor-first ingestion ("use Claroty/Dragos/Nozomi as-is, extend later")** | Strongly aligns with Prism's OCSF-normalization-at-the-adapter-boundary model. Prism already normalizes vendor output; the KG should be built **on top of the OCSF-normalized layer**, not from raw vendor schemas. |
| **ARO Engine: graph-aware prioritization, collapse N alerts → few items** | This is the *consumer* of the KG. For Prism, the KG+vector Context layer is the substrate; an ARO-style prioritization/correlation feature is a downstream Day-2 capability that the Context layer enables (entity-grouping of findings). |
| **NL Co-Pilot over the asset graph** | This is the Perplexity-style answer-engine pattern applied to security entities (see §2). Prism's MCP-agent-harness output is the natural delivery surface. |
| **Offline / air-gapped support as a hard production milestone** | Confirms air-gap is a first-class constraint, reinforcing the Rust-native embedded recommendation in §4/§7. |
| **Process-informed / operational-context risk** | Suggests the entity model should carry **business/operational context edges** (asset → process → mission), not just technical telemetry edges — relevant to Entity 360 expansion (§6). |

**What Prism should NOT pull:** aletheon's OT-specific framing (Layer-2 switch discovery, NERC CIP, FAIR-by-default), its "<10ms" real-time streaming claims, and its "community defense / cross-client learning" — these are aletheon's product bets, not Prism Context-layer concerns, and cross-client learning collides with Prism's per-tenant isolation + AD-017 AI-opaque posture. The transferable core is narrow and clear: **OCSF-normalized entity graph + NL/agent investigation over it + air-gap-capable.**

---

## 1. How Perplexity Does It (the user explicitly asked)

Perplexity describes itself not as a search engine but as an **AI answer engine** that "researches the open web in real time and returns concise, cited answers." [VERIFIED-WEB] Its differentiator is answer-first orientation: a single synthesized response with inline citations, optimized for accuracy/citation-density rather than click-through. The system is "part search engine, part LLM" — it searches, reads links, extracts relevant paragraphs, and feeds those snippets to an LLM instructed to "write like an academic" with inline citations.

The publicly-reconstructable stack (confirmed vs inferred carefully separated):

### 1.1 Core = RAG, multi-stage
- **RAG is the central organizing principle**, not a feature. Query → embed → retrieve semantically-similar passages from a vector store → augment → LLM generates cited answer. [VERIFIED-WEB] Perplexity's own RAG docs describe RAG-Token vs RAG-Sequence variants (per-token vs per-sequence retrieval), though they do not state which production uses. [VERIFIED-WEB / partly INFERRED]
- **Multi-stage retrieval+ranking pipeline**: candidate generation (high recall) → intermediate ranking/filtering → final RAG-aware re-ranking. [VERIFIED-WEB] Confirmed by Perplexity's "architecting an AI-first search API" research post.

### 1.2 Index + retrieval engine
- **Vespa.ai is the core retrieval layer** — confirmed by a Vespa case study. Vespa provides hybrid (lexical + vector/ANN) search, flexible multi-signal ranking, and entity-relationship support, serving 300M+ queries/week. [VERIFIED-WEB]
- Three stated index requirements: **completeness, freshness, speed.** Continuous crawl + real-time index updates + low-latency retrieval. [VERIFIED-WEB]
- **Fine-grained snippet-level indexing** (passage/paragraph, not whole-document) to fit LLM context windows. [VERIFIED-WEB]

### 1.3 Embeddings (their own published models)
- Perplexity ships custom **`pplx-embed` family** (`pplx-embed-v1` dense, `pplx-embed-context-v1` contextual / document-context-aware), at 0.6B and 4B params, MIT-licensed on HuggingFace. [VERIFIED-WEB]
- Reported benchmarks: `pplx-embed-v1-4B` ≈ **73.5% Recall@10** (2.4M corpus) vs 67.9% Qwen3-Embedding-4B; 0.6B reaches 71.1%. Multilingual 30M corpus: 91.7% Recall@1000. [VERIFIED-WEB]
- **Compact INT8 / binary embeddings** (4× / 32× storage reduction vs FP32) make snippet-level web-scale indexing viable. [VERIFIED-WEB] — *Directly relevant to Prism's 512MB process / 200MB-per-query memory budget.*

### 1.4 Hybrid retrieval = lexical + semantic
- Both dense embeddings AND lexical signals (BM25/TF-IDF-style). Semantic finds conceptually-related content; lexical guarantees exact-match on identifiers, short queries, and specific strings. [VERIFIED-WEB] *This is the single most transferable pattern for security telemetry, which is dense with identifiers (IPs, hashes, hostnames, PIDs).*

### 1.5 Ranking signals
- Multi-signal: semantic similarity, lexical match, **authority/trust** (domain reputation, citation frequency, user-feedback like Reddit), **recency/temporal decay**, and user-preference personalization. [VERIFIED-WEB, with exact weights proprietary]

### 1.6 Knowledge-graph / entity layer (the honest answer)
- **There is NO confirmed global web-scale knowledge graph** behind Perplexity answers. [VERIFIED-WEB — sources are explicit this is unconfirmed]
- However: **Perplexity Lens** (browser extension) **builds a personalized knowledge graph** connecting concepts/entities the user encounters — nodes = concepts/entities, edges = relationships. [VERIFIED-WEB] And **Comet** (their agentic browser) relies on a "user context system" that Srinivas calls the most important ingredient — a per-user entity/context store (history, saved docs, preferences) functioning like a personalized KG + context layer. [VERIFIED-WEB]
- Vespa "powers entity relationships and context-aware retrieval" for Perplexity. [VERIFIED-WEB] So entity-level metadata and graph-like structure exist in the retrieval layer for some verticals — but a full GraphRAG-style global graph is **not** publicly confirmed. [INFERRED boundary]

### 1.7 Synthesis + reasoning models
- **Sonar range-search models** (fast factual) vs **Sonar reasoning models** (deep analysis/reports). Query routing classifies intent and picks the model. [VERIFIED-WEB]
- Perplexity reportedly self-hosts a **DeepSeek-R1** variant for chain-of-thought summarization. [VERIFIED-WEB / reverse-engineering, treat as plausible-not-confirmed]
- External eval: Sonar-Reasoning-Pro tied #1 with Gemini-2.5-Pro-Grounding; Sonar cites **2–3× more sources** than some competitors. [VERIFIED-WEB]

### 1.8 Transferable patterns → Prism security-entity context engine

| Perplexity pattern | Prism Context application |
|---|---|
| Answer-engine over a corpus, with **mandatory inline citations** | NL/agent answers over telemetry — every claim cites the underlying OCSF event ID / detection rule / asset record. Maps to Prism's AI-agent-harness consumer. **Citations = the trust mechanism** for an LLM consuming security data. |
| **Hybrid lexical+semantic retrieval** | Lexical for exact IOC/identifier match (IP, hash, hostname, PID); semantic for "similar incidents / behaviors." Non-negotiable for security identifiers. |
| **Fine-grained snippet indexing** | Index each OCSF event/finding as a retrievable unit; contextual embedding (à la `pplx-embed-context-v1`) to encode an event in its sequence/session context. |
| **Compact INT8/binary embeddings** | Directly addresses the 512MB/200MB memory budget for an embedded vector index. |
| **Multi-stage ranking with recency + authority** | Security analogues: detection severity/confidence, asset criticality, threat-intel credibility, **event recency** (critical for live incidents). |
| **Personalized KG + context layer (Lens/Comet)** | The per-tenant entity graph IS Prism's "context system" — but **per-tenant isolated**, never cross-tenant (vs Perplexity's per-user). |
| **Model routing (fast vs reasoning)** | Maps cleanly onto C7's pluggable ModelBackend — route cheap embed/rank to small models, deep correlation to a reasoning model. |

---

## 2. Knowledge-Graph + Vector-DB Hybrid Architectures for Entity-Centric Security

### 2.1 Microsoft GraphRAG (the reference design)
- GraphRAG **extracts entities + relationships from text into a graph**, then runs **hierarchical community detection (Hierarchical Leiden)**, and **pre-summarizes each community** into "community reports" used for downstream querying. [VERIFIED-WEB — `microsoft.github.io/graphrag`, arXiv:2404.16130]
- Two retrieval modes: **Global search** over precomputed community summaries (for corpus-wide "sense-making" questions) and **Local search** combining **entity-neighborhood graph traversal + vector/embedding retrieval** over entities, chunks, and communities. [VERIFIED-WEB]
- Follow-on **DRIFT search / LazyGraphRAG**: multi-stage — start with global/vector community search, generate focused follow-up questions, then drive targeted local graph retrieval. Combines global community reasoning with local detail at lower cost. [VERIFIED-WEB]
- **Security relevance:** community detection ≈ automatically grouping related entities into "incidents"/"campaigns"; local search ≈ entity-360 neighborhood expansion. The Leiden-community-summary pattern is the most directly transferable GraphRAG idea for collapsing alert noise (the aletheon "ARO" goal, achieved structurally).

### 2.2 LangChain / LlamaIndex property-graph patterns
- Both frameworks ship **property-graph index** patterns: extract a graph from documents, store it in a graph DB, and pair it with a vector store; retrieval does graph traversal + vector similarity and fuses results. [VERIFIED-WEB] These are orchestration patterns (Python), useful as **design references**, not as Rust runtime components for Prism.

### 2.3 Neo4j / Memgraph native vector + graph hybrid
- **Neo4j** ships a **native vector index** alongside its graph engine, enabling hybrid graph-traversal + vector-similarity in one store; documented "GraphRAG" recipes. [VERIFIED-WEB]
- **Memgraph** (in-memory graph DB) similarly supports vector search + documents how Microsoft GraphRAG maps onto a graph DB. [VERIFIED-WEB]
- **Security relevance:** these prove the "one engine for graph + vector" model is mainstream — but both are **server-deployed (JVM / C++ service)**, not embeddable in a single Rust binary, so they are a poor fit for Prism's air-gap embedded constraint (see §4 for the Rust-native equivalent: CozoDB does graph + vector + Datalog in one embeddable engine).

### 2.4 Security-specific entity graphs + OCSF
- SIEM/XDR vendors model **host ↔ user ↔ IP ↔ process ↔ alert** as a native graph (the "security graph" / "attack graph" pattern is well-established across CrowdStrike Threat Graph, Microsoft Sentinel/Defender entity graph, Splunk asset/identity framework, Exabeam/Securonix UEBA entity timelines). [VERIFIED-WEB — vendor docs; treat specific internal implementations as MODEL-level]
- **OCSF (Open Cybersecurity Schema Framework)** already defines a normalized **entity/object model** (e.g., `user`, `device`, `process`, `network_endpoint`, `actor`) that events reference. [VERIFIED-WEB] **This is the key Prism advantage:** Prism is *already* OCSF-normalized at the adapter boundary, so the OCSF object model is a ready-made graph schema — entities and their cross-references are the nodes/edges. The KG should be **derived from the OCSF layer**, eating Prism's own dog food.

### 2.5 How the two layers wire together (the canonical hybrid)
The established pattern (GraphRAG / Neo4j-hybrid / LlamaIndex property-graph), adapted to security:
1. **Graph layer** = authoritative entity relationships (typed nodes + typed edges + temporal validity).
2. **Vector layer** = semantic embeddings of events/findings/notes/IOC descriptions for similarity search.
3. **Fusion at query time**: (a) lexical/structured filter narrows by entity+timeframe → (b) graph traversal expands the entity neighborhood → (c) vector search finds semantically-similar events/incidents → (d) re-rank by severity/recency/criticality → (e) LLM synthesizes a cited answer. This is Perplexity's multi-stage pipeline + GraphRAG's local-search fused. [INFERRED synthesis of cited patterns]

---

## 3. (covered inline in §4) Rust-Native Options — see §4

## 4. Rust-Native Options (versions verified on crates.io 2026-06-27)

> Every version below is **[REGISTRY]** — fetched live from `crates.io/api/v1/crates/<name>` on 2026-06-27. Where a project looks stale, I flag the last-publish date explicitly.

### 4.1 Combined graph + vector + query engines (embeddable)

| Crate | Latest ver | License | Last publish | Notes |
|---|---|---|---|---|
| **`cozo`** | **0.7.6** | MPL-2.0 | **2023-12-11** | The standout: **one embeddable engine doing Datalog + graph algorithms + HNSW vector search**, with pluggable backends (in-memory, **RocksDB**, SQLite). Air-gap-friendly, embeddable in-process. **CAUTION: last published Dec 2023 — ~2.5 yrs stale as of 2026-06-27.** Maintenance risk is the single biggest open question for adopting it. [REGISTRY + VERIFIED-WEB] |
| **`indradb`** | **5.0.0** | MPL-2.0 | **2025-08-16** | Pure Rust graph DB, RocksDB-backed, actively maintained (2025). **Graph only — no native vector search**; would need to be paired with a separate vector index. [REGISTRY + VERIFIED-WEB] |

### 4.2 Embedded vector indexes / DBs

| Crate | Latest ver | License | Last publish | Notes |
|---|---|---|---|---|
| **`lancedb`** | **0.30.0** | Apache-2.0 | **2026-05-28** | Embedded, on-disk, columnar (Arrow/Lance format) vector DB. **Actively maintained, current.** Serverless/embeddable, good air-gap fit; scales to disk (not RAM-bound). Strong candidate for the vector tier. [REGISTRY] |
| **`usearch`** | **2.25.3** | Apache-2.0 | **2026-05-24** | Rust bindings to the USearch HNSW engine. Very fast, compact, **actively maintained**. Supports quantization (relevant to memory budget). Embeddable. [REGISTRY] |
| **`hnsw_rs`** | **0.3.4** | MIT/Apache-2.0 | **2026-02-28** | Pure-Rust HNSW ANN. Maintained (early 2026). Lightweight, in-process, air-gap-trivial. Index-only (no persistence/DB layer of its own). [REGISTRY] |
| **`instant-distance`** | **0.6.1** | MIT/Apache-2.0 | **2023-06-26** | Pure-Rust HNSW (by the Instant Domain team). **STALE — last publish June 2023.** Functional but unmaintained-looking; prefer `hnsw_rs` or `usearch`. [REGISTRY] |
| **`oasysdb`** | **0.7.3** | Apache-2.0 | **2024-08-07** | Embedded Rust vector DB. **Stale (Aug 2024)** and the upstream project has pivoted/wound down; not recommended as a primary dependency. [REGISTRY] |
| **`qdrant-client`** | **1.18.0** | Apache-2.0 | **2026-05-11** | This is the **client** for Qdrant the *server*. Qdrant is **server-only — there is no first-class embedded/in-process mode**; running it implies a sidecar service. [REGISTRY + VERIFIED-WEB] Poor fit for "single embeddable binary in air-gap," acceptable for the hosted/on-prem-with-services deployment model. |

### 4.3 On-prem / on-device embedding generation (no data leaves the box — ties AD-017 + C16 masking)

| Crate | Latest ver | License | Last publish | Notes |
|---|---|---|---|---|
| **`fastembed`** | **5.17.2** | Apache-2.0 | **2026-06-15** | **fastembed-rs** — highest-level option. ONNX-based, runs sentence-embedding models (BGE-small/base, all-MiniLM, nomic-embed, multilingual-E5, etc.) **locally on CPU**. Very actively maintained (June 2026). Bundles tokenization + model download (must pre-stage models for air-gap). **Strongest default for "just give me local embeddings."** [REGISTRY + VERIFIED-WEB] |
| **`ort`** | **2.0.0-rc.12** | MIT/Apache-2.0 | **2026-03-05** | ONNX Runtime Rust bindings. The engine `fastembed` sits on; use directly for custom ONNX models / finer control. **Note: still RC (no stable 2.0 release) — pin exactly and track.** This is exactly the `ort` already named in C7's ModelBackend. [REGISTRY] |
| **`candle-core`** | **0.11.0** | MIT/Apache-2.0 | **2026-06-26** | HuggingFace's pure-Rust ML framework (also named in C7). Can run embedding models (BERT/BGE family) natively without ONNX. Actively maintained (June 2026). More work than fastembed for embeddings, but no C/ONNX dependency — cleanest pure-Rust air-gap story. [REGISTRY] |
| **`tract-onnx`** | **0.23.3** | MIT/Apache-2.0 | **2026-06-19** | Sonos's self-contained pure-Rust inference (ONNX/TF). Named in C7. Good for tightly-controlled, dependency-minimal inference; can run small embedding models. Actively maintained. [REGISTRY] |

**Air-gap note for all embedding crates:** every one of these downloads model weights from HuggingFace by default. For air-gap, models must be **pre-staged into the image/bundle** and the download path disabled. This is a packaging concern, not a blocker — and it aligns with C16 (data masking) and AD-017 (credentials/data never transit AI context): embeddings are computed *in-process, on-box*, so raw telemetry never leaves the tenant boundary to be vectorized.

### 4.4 RocksDB-fit summary
Prism already runs **RocksDB (19 column families)**. Both `cozo` and `indradb` can use RocksDB as their backend, meaning the graph layer could live in the *same storage substrate*. `lancedb`/`usearch`/`hnsw_rs` use their own on-disk/in-memory formats alongside RocksDB. [REGISTRY + VERIFIED-WEB / INFERRED for the co-location detail]

---

## 5. Entity Resolution / Entity Mapping at Scale

### 5.1 The problem
Security telemetry expresses the same real-world entity many ways across sources: a user is `jdoe`, `jdoe@corp`, an Okta UUID, a Windows SID, an email; a host is a hostname, an asset tag, a MAC, multiple IPs over time (DHCP). **Identity stitching** = linking these into one **canonical entity** with a stable canonical ID. [VERIFIED-WEB / MODEL]

### 5.2 Established patterns
- **Canonical entity IDs** with an alias/identifier table mapping every observed identifier → canonical entity. The OCSF object model gives a normalized vocabulary to map *into*. [VERIFIED-WEB]
- **Deterministic-first, probabilistic-second:** exact-match on strong identifiers (SID, UUID, MAC) deterministically; fuzzy/probabilistic match (name similarity, co-occurrence) only as a fallback, with a confidence score on the merge.
- **Temporal validity on edges:** DHCP means IP↔host is only valid for a time window. Edges and identity bindings **must carry validity intervals**; "host A had IP X *between t0 and t1*." [VERIFIED-WEB security-graph practice / MODEL]
- **UEBA-style entity timelines** (Exabeam/Securonix pattern): all activity normalized onto a per-entity timeline keyed by the canonical ID. [VERIFIED-WEB]

### 5.3 Pitfalls (call these out for Prism)
- **Over-merging** (false-positive stitch): merging two distinct users because of a shared NAT IP or a shared service account → catastrophic for security (attributes one person's actions to another). Mitigation: require strong identifiers for auto-merge; keep weak-signal links as *suspected* edges, not merges.
- **Splitting** (false-negative): failing to link a user's two identities → fragments the Entity 360 view, hides lateral movement. Mitigation: alias-resolution sweeps.
- **Temporal identity errors:** binding an IP to the wrong host because the DHCP lease rotated. Mitigation: time-windowed edges (above).
- **Tenant bleed:** in multi-tenant Prism, entity resolution must be **strictly per-tenant** — never resolve identities across tenant boundaries (collides with isolation + AD-017).

---

## 6. Entity 360 Expansion — Best-in-Class Contents

Prism already has an "Entity 360" concept + entity resolution. A best-in-class entity-360 view (drawing on the SIEM/XDR/UEBA conventions) contains: [VERIFIED-WEB vendor-convention / MODEL]

1. **Identity panel** — canonical entity + all resolved aliases/identifiers + confidence on each binding.
2. **Timeline** — chronological normalized activity (OCSF events) for the entity, with time-windowed identity context.
3. **Relationship graph** — the entity's neighborhood: host↔user↔IP↔process↔alert↔asset, with typed edges and temporal validity. (This is the **graph layer**.)
4. **Risk score** — composite of severity, criticality, anomaly signals; explainable (cited contributing events).
5. **Exposures** — vulnerabilities, misconfigurations, unpatchable flags (the aletheon "high-risk asset tagging" idea).
6. **Related findings / similar entities** — "entities that look like this one" / "incidents like this." (This is the **vector layer** — semantic similarity.)
7. **Operational/business context** — asset → process → mission edges (aletheon's "process-informed" insight): what does this entity *support*.

**How KG + vector powers it:** the **graph** supplies 1–3 and 5 and 7 (deterministic structure + relationships + exposures), the **vector index** supplies 6 (semantic neighbors), and **fused retrieval + an LLM** supplies the cited natural-language narrative ("explain this entity's risk") — the Perplexity answer-engine pattern (§1.8) applied to one entity's subgraph. GraphRAG's *local search* (entity-neighborhood + vector) is essentially the Entity 360 query. [INFERRED synthesis]

---

## 7. Deployment Fit (embedded / on-prem / air-gap; per-tenant; vs RocksDB hot / Iceberg cold)

### 7.1 The constraint hierarchy
Prism has **three deployment models incl. air-gap**, a **satellite-mesh edge**, **RocksDB hot + Iceberg cold tiers**, **per-tenant isolation**, and **AD-017 AI-opaque credentials/data**. The Context layer must work in the *strictest* environment (air-gap) by default, with no external SaaS dependency.

### 7.2 Recommended layering relative to hot/cold tiers
- **Entity graph (relationships, canonical IDs, aliases):** small, slowly-changing, high-value → lives in the **RocksDB hot tier** (its own column family/families), co-located with the existing 19 CFs. An embedded graph engine (`indradb` RocksDB-backed, or `cozo` RocksDB-backed) fits here. [INFERRED / VERIFIED-WEB for backend support]
- **Vector index (event/finding embeddings):** larger, can be tiered. Hot embeddings (recent events) in an embedded ANN (`usearch`/`hnsw_rs` in RAM with INT8/binary quantization to respect the 512MB budget); cold/historical embeddings persisted on-disk (`lancedb`) and rehydrated on demand — naturally maps onto the **hot→Iceberg-cold** tiering. [INFERRED]
- **Embedding generation:** **in-process, on-box** (`fastembed`/`candle`/`ort`/`tract`), models pre-staged in the bundle for air-gap. Raw telemetry is vectorized locally and never transits AI context → satisfies AD-017 + C16. [VERIFIED-WEB capability + INFERRED policy fit]

### 7.3 Per-tenant isolation
Entity resolution, graph, and vector index must all be **per-tenant-partitioned** (separate graph namespaces / vector collections / RocksDB CF key-prefixes per `OrgSlug`). No cross-tenant edges, no cross-tenant similarity search. This is the hard line that diverges from both Perplexity (per-user) and aletheon (cross-client "community defense").

### 7.4 Why server-based options are excluded by default
Neo4j, Memgraph, and Qdrant-server all require a **separate running service**. They violate the single-embeddable-binary air-gap default and add an operational surface. They remain *acceptable for the hosted/on-prem-with-services deployment model* but cannot be the default. [VERIFIED-WEB + INFERRED]

---

## ANALYSIS + LEANS — Recommended Prism Context Architecture

### Recommended shape (lean, not a decision)
A **two-layer hybrid Context engine, embedded and air-gap-first**, built on the OCSF-normalized layer Prism already produces:

1. **Graph layer (entity relationships + canonical IDs):**
   - **LEAN: `indradb` 5.0.0 (RocksDB-backed, actively maintained 2025-08)** as the safe default — it co-locates with Prism's existing RocksDB substrate and is currently maintained.
   - **Alternative to evaluate: `cozo` 0.7.6** — uniquely does graph + vector + Datalog in *one* embeddable engine (would collapse two layers into one), but **last published Dec 2023**; the maintenance-staleness risk must be weighed by a human (see sub-forks).
2. **Vector layer (semantic similarity over events/findings):**
   - **LEAN: `usearch` 2.25.3 or `hnsw_rs` 0.3.4** for the hot in-memory ANN (with INT8/binary quantization, mirroring Perplexity's compact-embedding tactic, to respect the 512MB/200MB budget), tiered down to **`lancedb` 0.30.0** on-disk for cold/historical vectors (maps to the Iceberg cold-tier pattern).
3. **Embedding generation (on-box):**
   - **LEAN: `fastembed` 5.17.2** as the high-level default (local CPU sentence embeddings, BGE/MiniLM/nomic), sitting on **`ort` 2.0.0-rc.12** — which is already in C7's ModelBackend set. **`candle` 0.11.0** is the pure-Rust fallback if avoiding the ONNX/C dependency matters for air-gap auditability. Models pre-staged in the bundle.
4. **Retrieval + synthesis (the answer engine):**
   - Adopt **Perplexity's hybrid multi-stage pattern** (lexical-filter → graph-neighborhood-expand → vector-similarity → severity/recency re-rank → LLM-synthesize-with-mandatory-citations) and **GraphRAG's local-search** as the Entity 360 query, with **Leiden community detection** as the structural mechanism to collapse alert noise into incident clusters (the aletheon "ARO" goal achieved structurally). Route via C7's pluggable ModelBackend (fast embed/rank vs reasoning synthesis).

### What to adopt from aletheon
- The **"lists+configs → relationship graph"** evolution thesis (validates C12).
- **Vendor-first / build-on-normalized-output** ingestion — for Prism that means **build the KG from the OCSF layer**, not raw vendor schemas (eat-our-own-dog-food).
- **Process-informed / operational-context edges** (asset → process → mission) in the entity model and Entity 360.
- **Air-gap as a hard production milestone**, not an afterthought.
- **NL investigation over the graph** as the delivery surface.
- **Do NOT adopt** aletheon's cross-client "community defense" learning — it collides with Prism per-tenant isolation + AD-017.

### What to adopt from Perplexity-style patterns
- **Mandatory inline citations** = the trust contract for LLM-consumed security data (every claim → OCSF event ID / rule / asset record).
- **Hybrid lexical + semantic retrieval** (lexical is mandatory for IOC/identifier exact-match).
- **Fine-grained snippet (per-event) indexing** + **contextual embeddings** (event-in-session-context, à la `pplx-embed-context-v1`).
- **Compact INT8/binary embeddings** to fit the memory budget.
- **Multi-stage ranking** with security-domain signals (severity, criticality, threat-intel credibility, recency).
- **Model routing** (fast vs reasoning) onto the existing ModelBackend.
- Honest boundary: **don't over-promise a "knowledge graph" where a simpler entity-relationship store suffices** — even Perplexity has no confirmed global KG; the value is in the *fused retrieval + citations*, not graph maximalism.

### Genuine sub-forks needing a HUMAN decision
1. **One engine vs two-layer:** Adopt `cozo` (graph+vector+Datalog in one embeddable engine, simpler architecture) and **accept its Dec-2023 staleness/maintenance risk** — OR run the safer two-layer `indradb` + (`usearch`/`lancedb`) stack at the cost of more moving parts? This is a maintainability-vs-simplicity risk call. **[Human]**
2. **Vector tier topology:** in-memory ANN only (RAM-bound, fastest, simplest) vs hot-RAM + cold-on-disk (`lancedb`) tiering (matches hot/Iceberg model, more complex). Tied to how much historical semantic recall the product needs. **[Human/architect]**
3. **Embedding model choice + size** (BGE-small vs MiniLM vs nomic-embed vs multilingual-E5) and the resulting memory/latency vs recall tradeoff — and whether `fastembed`/`ort` (ONNX, fast, but `ort` is RC) vs pure-Rust `candle` (cleaner air-gap audit, more effort). **[Human/architect, with a perf benchmark]**
4. **Entity-resolution merge policy:** the over-merge-vs-split risk threshold (how strong an identifier is required for auto-merge; how suspected-links are surfaced) is a **security-correctness policy** with real consequences (mis-attribution) — needs human/security-reviewer sign-off, not an engineering default. **[Human/security-reviewer]**
5. **Server-based escape hatch:** is a Neo4j/Qdrant-server backend an allowed option for the hosted/on-prem-with-services deployment model (richer ecosystem) while embedded stays the air-gap default — or is single-binary embedded mandated everywhere for consistency? **[Human/architect]**
6. **GraphRAG community pre-summarization cost:** Leiden community detection + LLM community-summary pre-compute is expensive and must re-run as telemetry streams in. Whether to adopt the full GraphRAG global-search pre-summarization, or only local-search (cheaper, entity-neighborhood + vector), is a cost/value fork. **[Human/architect]**

### Inconclusive / flagged
- **Perplexity's exact ranking models, RAG variant, and any internal global KG are proprietary** — treated as inferred, not confirmed (§1.6, §1.8). Do not build on assumptions about Perplexity internals beyond the confirmed hybrid-retrieval + citation patterns.
- **aletheon has no technical KG/vector spec** — only pitch-level concepts (§0). Nothing implementation-level to transfuse.
- **`cozo` maintenance trajectory** (last publish 2023-12) could not be conclusively assessed as alive-or-abandoned from the registry alone; a human should check the upstream repo activity before committing to it.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | (1) Perplexity AI retrieval/knowledge architecture in depth; (2) GraphRAG + KG/vector hybrid for security entities + OCSF + entity resolution + Entity 360; (3) Rust-native graph/vector/embedding crates + air-gap fit |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 1 | Focused confirmation of Microsoft GraphRAG Leiden community detection + global/local search + DRIFT/LazyGraphRAG (compact, citeable) |
| Context7 | 0 | — |
| Tavily (all) | 0 | — |
| WebFetch | 12 | 11 crates.io API version verifications (cozo, lancedb, fastembed, candle-core, ort, instant-distance, hnsw_rs, indradb, usearch, qdrant-client, tract-onnx, oasysdb) — registry-authoritative versions |
| WebSearch | 0 | — |
| Read | 4 | aletheon docs (2) + Perplexity research output files |
| Glob/Grep | 5 | Locate/confirm aletheon spec content; extract sections from long-line research outputs |
| Training data | 2 areas | (a) SIEM/XDR internal security-graph implementation specifics (flagged MODEL); (b) entity-resolution merge-policy patterns (flagged MODEL/VERIFIED-WEB-mix) |

**Total MCP tool calls:** 4 (3× `perplexity_research` PRIMARY + 1× `perplexity_ask`)
**Total registry verifications (crates.io):** 12 crates, all confirmed live 2026-06-27.
**Training data reliance:** **low** — all crate versions are registry-verified (not training data); architecture patterns are web-cited; the two MODEL-flagged areas (SIEM-internal graph implementations; ER merge mechanics) are explicitly marked and corroborated by vendor-convention web sources where possible.

### Note on a process constraint encountered
The three `perplexity_research` outputs (76K–92K chars each) exceeded the tool's single-read token cap and were spilled to disk as single-line JSON. The first output was read ~85% directly; the other two were mined via targeted Grep for the named entities/tools (all confirmed present) plus the focused `perplexity_ask` follow-up for GraphRAG specifics. The substantive claims above are sourced from the read portions + the version-authoritative crates.io fetches; where a claim rests on inference rather than a directly-read passage, it is tagged [INFERRED].
