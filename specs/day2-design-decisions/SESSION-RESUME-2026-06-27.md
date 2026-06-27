---
document_type: session-resume
status: capture
do_not_execute: true
timestamp: "2026-06-27"
provenance: "Day-2 vision SIDE-ANALYSIS program resume snapshot. SEPARATE from the live factory pipeline. Does NOT modify .factory/STATE.md or SESSION-HANDOFF.md (those belong to the live factory track)."
---

# Day-2 Vision Side-Analysis — Zero-Context Resume Snapshot (2026-06-27)

> **READ THIS FIRST on resume.** This is OUT-OF-BAND day-2 vision work, fully SEPARATE from the live
> VSDD factory pipeline (which runs its own cascades on the same `factory-artifacts` branch). Everything
> here is a CAPTURE artifact (`do_not_execute: true`); the day-2 morph is gated on the brief-reframe
> HUMAN sign-off (§5.1 of the matured-vision doc). Nothing here modifies the live brief/PRD/BCs/ADR-registry.

## 0. Hard boundary rules (carry forward, non-negotiable)
- NEVER touch `.factory/STATE.md` or `.factory/SESSION-HANDOFF.md` — those are the live factory track.
- NEVER modify live specs, the live ADR registry (`.factory/specs/architecture/decisions/`), BCs, or stories.
- Commits: SINGLE commit per burst (TD-VSDD-053). Stage by EXPLICIT file path — NEVER `git add -A`/`.`/`-a`,
  NEVER `git add research/` (would sweep live-factory research files). Leave `.DS_Store` and any
  live-factory files (e.g. `research/demo-finding-remediation-plan-*.md`, `research/test-suite-performance-*.md`)
  UNSTAGED. Push to `origin/factory-artifacts` under standing auth D-1066; on divergence fetch + `pull --rebase`
  (the live factory pushes concurrently); never `--no-verify`/`--force`. Last side HEAD: **896456f6** (plus this snapshot's commit).
- Working rhythm per area: research-agent pass (background) → I synthesize + give leans → AskUserQuestion on the
  genuine forks → architect captures decisions into the doc/ADR-PROPs → state-manager path-scoped commit.
  Pipeline research ~2–3 areas ahead. Confirm decisions before writing big new sections (mirror→confirm→write).

## 1. Where everything lives
- **`specs/matured-vision-day2-requirements.md`** — the master capture (§1–§17). §16 = prior resume notes;
  §16.4 = running open-items/decisions log; §17 = federated ingestion (collector class, #4 pcap, #5 continuous
  operator, locus, chain-aware tiering/replication/deadline §17.8, dissectors §17.12, OT §17.13, reshape §17.14,
  A-leans §17.15).
- **`specs/day2-design-decisions/`** — PROPOSED ADRs + sketches (do_not_execute; real ADR #s deferred to morph):
  ADR-PROP-web-stack, -sso-identity, -s3-agent-runtime, -widget-dsl-render-and-schema-validation,
  -sandboxed-expression-evaluator, -storage-engine-taxonomy, -central-deployment-access-layer; plus
  secret-subsystem-sketch, prismql-sequence-sugar-decisions, ml-depth-phasing, po-ratifications; and this file.
- **`specs/day2-ui-design/`** — S2 (13-screen) + U1 (8-screen) console specs; `mockups/` = tokens.css +
  style-guide + 28 HTML panels (incl. S3 canvas ×2, S4 extension, 3 responsive, states-gallery) + ~64 light/dark
  screenshots; `S3-conversational-canvas-disposition.md`.
- **`research/` (our cited passes, dated 2026-06-25/26):** ui-requirements + 2 raw UI passes; queryio-federated-search;
  federated-query-language-patterns; deployment/credentials/UI; match-recognize-rpr-feasibility; axiathon-detection;
  federated-ingestion-collector-connectors; chain-cache-tiering-replication-deadlines;
  detection-reshape-protocol-dissectors; ingestion-open-subthreads; central-deployment-access-layer;
  satellite-mesh; capability-descriptor-pushdown.

## 2. DECISIONS MADE (do not re-litigate)
- **UI-D5:** web stack = **TypeScript SPA (React) + Rust backend**, OpenAPI→TS shared types (AG Grid/TanStack,
  Monaco, Cytoscape, ECharts). Heading font **Archivo** (no Adobe kit); Manrope body; IBM Plex Mono code.
  Light palette parsed from 1898 & Co brand; dark derived. Editor re-themes with page. Purple = AI. Admin pinned bottom.
- **Conversational canvas (aletheon_2 spike):** ADOPT as the **S3** embedded-AI surface, enhanced+hardened.
  Native Spicy-style dissector engine + prism-native continuous operator (see below). DSL is UI-generation,
  orthogonal to PrismQL.
- **4 prior human decisions:** PrismQL `NOT/WITHOUT` = **BOTH** exclusion (`{- B -}`) AND timeout/absence
  (`WATCH…UNLESS`→AbsenceWindowNode), both Phase-A; **S3 opt-in** by default; **SCIM 2.0 in day-2 scope**
  (OIDC+SAML+JIT); **multi-surface UI S2–S4+U1 in brief §1 In-Scope**.
- **Federated ingestion (§17):** collector = a *source+buffer* (not a sink); collector-connector subtype;
  only-new-primitive = a receiver endpoint; per-instance **edge-first locus**; **#4 full-packet pcap retrieval IN
  scope** (Arkime-style second/artifact regime, retrieve-by-session); **#5 continuous-operator capability** PHASED
  (v1 NRT-over-cache + edge Zeek/Suricata → later **prism-native** windowed operator on RocksDB; WATCH…UNLESS dual
  impl); **detection spec carries EXPLICIT temporal semantics** (lateness/accumulation/window-alignment in §14 YAML —
  planner picks engine not meaning); chain-aware model: declarative-policy-floor tiering, **residency-first per-field
  ordered-before-forward** replication policy, Q3 deadline v1 (gRPC + partial+coverage + opportunistic hub
  pre-aggregate) with full budget-aware planner ordered later.
- **Protocol dissectors:** **prism EMBEDS a native Spicy-style declarative dissector engine** (authors its own
  grammars incl. OT — not federating Zeek/Suricata). Dissector = §17 stage-3 normalization for packets; emits OCSF
  + native + Community ID.
- **OT/ICS:** **OCSF has NO OT classes** (confirmed; open proposal ocsf#1515) → OT = **flagship
  native-schema-on-read** (§13.6). Passive/read-only, TAP>SPAN, Purdue/IEC-62443 placement, no injection → OT
  dissectors on the OT-layer satellite. Encrypted-OT (OPC-UA/MQTT-TLS) = **metadata-only default + bounded
  decrypt/proxy opt-in later**.
- **FOUR-ENGINE STORAGE TAXONOMY (ADR-PROP-storage-engine-taxonomy.md):**
  **RocksDB** = ephemeral/hot data-plane (correlation/detection state, RetentionCache hot tier, operator window
  state, S&F queues; central+satellite). **Iceberg** = cold analytic tier (long-baseline OCSF+native, RETAIN).
  **PostgreSQL (BUNDLED in the appliance, NEVER external; central-ONLY)** = relational control-plane
  (case-management+alerts, config store §11.2, RBAC, audit, tenant/user, identity/AS, result-cache METADATA).
  **SQLite (embedded)** = satellite-local control-plane (enrollment/identity state, local config/policy).
  §14.3 reconciled: no-Postgres ruling protected the *ephemeral correlation path* (still RocksDB); control-plane is
  a distinct workload → Postgres is right-tool, conscious not silent. Iceberg rejected for cases (OLAP vs OLTP).
- **C1 central deployment (ADR-PROP-central-deployment-access-layer.md):** transport = MCP **Streamable HTTP**
  (rev 2025-06-18) + keep stdio, via in-tree `rmcp StreamableHttpService`; identity = **OAuth 2.1 Resource Server**
  + **built-in AS + external IdP** (OIDC/SAML); creds = SS-26 + per-connection-analyst audit binding; shared state =
  **FULL case-management on bundled Postgres**, optimistic CAS + soft ownership + presence (no hard locks); ops =
  stateless-front + shared state, SSE `Last-Event-ID` resumability, per-tenant Tokio Semaphore/WFQ fairness,
  readiness+drain. DI-017 → single-central-service.
- **A — ingestion open sub-threads (§17.15), 7 leans ACCEPTED:** (1) per-rule data-dependency manifest →
  tri-state runnable/degraded/unavailable; (2) result-cache key = query-intent+event-time-bucket+residency+schema-ver,
  **DECLINE cross-hop coherence guarantee in v1** (conscious scope-limit) + single-flight recompute; (3) two-tier state
  + at-least-once idempotent finding-emit; (4) decoupled capture→detect→policy-engine-issues-pin, first-N-bytes default
  + full-session escalation, Community ID key, PTP/NTP prereq; (5) prism-native entity registry (config-driven
  entity_type→ordered attribute paths, strong/weak tiers, temporal IP↔asset validity); (6) protect-capture-path-first
  edge budget (shed operator→reduce dissector depth→capture-loss last); (7) per-field residency scoped within
  per-(source-class,schema,version) tables, authored as a dedicated policy-as-code artifact.

## 3. THE C-PROGRAM (the active plan — "do each of the remaining day-2 areas")
Each area: research → discuss → decide → capture → commit. **B = integration capstone LAST.**

| Item | Status | Notes |
|---|---|---|
| A — ingestion open sub-threads | ✅ decided + captured (§17.15) | |
| C1 — central deployment / access layer | ✅ decided + captured | |
| Storage taxonomy | ✅ decided + captured | |
| **C2 — satellite mesh** | 🔬 research DONE (`research/satellite-mesh-2026-06-26.md`), **NEEDS DISCUSSION (resume here)** | Leans: SPIFFE-*model* identity native-Rust (not SPIRE runtime); **per-hop mutual mTLS, NO transitive trust**; transport fork **gRPC-bidi-streaming (tonic) vs NATS-leaf**; RocksDB-backed store-and-forward; loop-prevention = seen-request-ID + hop-count-TTL ± path-vector; role-noun lean **"Relay Satellite"**. |
| **C3 — capability-descriptor + PrismQL pushdown** | 🔬 research DONE (`research/capability-descriptor-pushdown-2026-06-26.md`), NEEDS DISCUSSION | Leans: declarative per-`[[tables]]` TOML descriptor mapped to DataFusion `Exact/Inexact/Unsupported`; DF 50.x pushdown = filter/projection/limit ONLY (agg/sort/join central in adapter); **hard-reject unbounded cross-source joins** at plan-time + restrict to inner-equi + exploit DF 50.x sideways-information-passing; **inject** default time-window with disclosure (reject reserved for joins); collector subtype `pushdown_target=buffer`. |
| C4 — dynamic-schema / configure-schema connectors | ⏳ not started | builds on C3 descriptor |
| C5 — SIEM / Security-Lake federation (Amazon Security Lake) | ⏳ not started | builds on C4 + Iceberg cold tier |
| C6 — detection engine + rule-editor depth | ⏳ not started | extended by continuous operator + dissectors |
| C7 — ML / behavior analytics depth | ⏳ not started | beyond §15 phasing sketch |
| C8 — PrismQL deliverables (entity-pivot, join-guard, ergonomics) | ⏳ not started | entity registry ↔ OT resolution |
| C9 — config-management pillar (server config store, GitOps) | ⏳ not started | config store → Postgres central / SQLite edge |
| C10 — Query.io competitive refresh | ⏳ not started | positioning gap-check |
| **B — integration capstone** | ⏳ LAST | brief-reframe deltas + consolidated epic/ADR/story list reflecting ALL decisions |

## 4. NEXT ACTION on resume
**Discuss C2 (satellite mesh)** using `research/satellite-mesh-2026-06-26.md` (leans above; the genuine fork to put to
the human is **gRPC-bidi vs NATS-leaf transport**, plus confirm "Relay Satellite" naming). Then C3 (capability-descriptor),
then research+discuss C4→C10 in order (pipeline research ~2–3 ahead), then B. Re-launch C4 research when C2 opens.

## 5. Gaps / epics introduced (PROPOSED, not in STORY-INDEX)
Gaps **G-1 … G-36**. Proposed epics: E-CACHE-DEMAND, E-CENTRAL-TRANSPORT/AUTHZ/OPS, E-SATELLITE-MESH,
E-LAKE-CONNECTOR, E-UI-ADMIN/CONSOLE/EMBEDDED-AI/EXTENSION, E-CONNECTOR-DYNAMIC, E-DETECT-*, E-RULE-XLATE,
E-ML-*, E-COLLECTOR-CLASS/PCAP, E-CHAIN-CACHE, E-STREAM-DETECT, E-DISSECTOR-NATIVE/OT. All deferred to morph
(brief-reframe §5.1 HUMAN GATE).
