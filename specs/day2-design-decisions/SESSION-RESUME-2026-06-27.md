---
document_type: session-resume
status: capture
do_not_execute: true
timestamp: "2026-06-27"
provenance: "Day-2 vision SIDE-ANALYSIS program resume snapshot. SEPARATE from the live factory pipeline. Does NOT modify .factory/STATE.md or SESSION-HANDOFF.md (those belong to the live factory track)."
---

# Day-2 Vision Side-Analysis — Zero-Context Resume Snapshot (2026-06-27)

> **Original C-program (C1–C10 + deployment-matrix + C3 hard-reject reconciliation) COMPLETE and committed.
> PRE-B feature track: C13 ✅ C12 ✅ C11 ✅ C15 ✅ C14 ✅ C19 ✅ — now at C18 (next). ADS (Architecture
> Design System) created and locked as conformance frame. Surfacing model LOCKED = Option 3. Conformance
> pass DONE. B remains LAST and gated on the §5.1 brief-reframe HUMAN sign-off.**
>
> **Latest factory-artifacts HEAD: `(this checkpoint)`**
>
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
  (the live factory pushes concurrently); never `--no-verify`/`--force`. Last side HEAD: **02599c9b**
  (ocsf-ot-coverage banked + SESSION-RESUME updated; prior: C15 decided + ADR-PROP-soar-actions-aro.md captured).
- Working rhythm per area: research-agent pass (background) → I synthesize + give leans → AskUserQuestion on the
  genuine forks → architect captures decisions into the doc/ADR-PROPs → state-manager path-scoped commit.
  Pipeline research ~2–3 areas ahead. Confirm decisions before writing big new sections (mirror→confirm→write).
- Human pattern to carry: often defers a fork to a targeted research pass before deciding; consistently chooses
  the fuller/production-grade + audit-grade option. On every C10 fork in this session, chose the fuller path
  with no declines.

## 1. Where everything lives

### Committed artifacts
- **`specs/matured-vision-day2-requirements.md`** — master capture (§1–§17). §16 = prior resume notes;
  §16.4 = running open-items/decisions log; §17 = federated ingestion (collector class, #4 pcap, #5 continuous
  operator, locus, chain-aware tiering/replication/deadline §17.8, dissectors §17.12, OT §17.13, reshape §17.14,
  A-leans §17.15). §12.2 join-language reconciled to D-C3-1 (3 `[SUPERSEDED by D-C3-1]` inline markers +
  history preserved).
- **`specs/day2-design-decisions/`** — PROPOSED ADRs + sketches (do_not_execute; real ADR #s deferred to morph):
  ADR-PROP-web-stack, -sso-identity, -s3-agent-runtime, -widget-dsl-render-and-schema-validation,
  -sandboxed-expression-evaluator, -storage-engine-taxonomy, -central-deployment-access-layer,
  -satellite-mesh, -capability-descriptor-pushdown, -dynamic-schema-connectors, -siem-lake-federation,
  -detection-engine-depth, -ml-behavior-analytics-depth, -prismql-deliverables,
  **-config-management** (NEW — C9 full Q1–Q3 + all cross-cutting deployment-conditional entries),
  **-dual-deployment** (NEW — three operating models: SaaS / MSSP-managed / client-managed),
  **-competitive-positioning** (NEW — C10 Query.io refresh, all 8 gaps addressed);
  plus secret-subsystem-sketch, prismql-sequence-sugar-decisions, ml-depth-phasing, po-ratifications,
  and this file.
- **`specs/day2-ui-design/`** — S2 (13-screen) + U1 (8-screen) console specs; `mockups/` = tokens.css +
  style-guide + 28 HTML panels + ~64 light/dark screenshots; `S3-conversational-canvas-disposition.md`.

### Research passes (cited, in `research/`)
Dated 2026-06-25/26 (prior sessions): ui-requirements, 2× raw UI passes, queryio-federated-search,
federated-query-language-patterns, deployment/credentials/UI, match-recognize-rpr-feasibility,
axiathon-detection, federated-ingestion-collector-connectors, chain-cache-tiering-replication-deadlines,
detection-reshape-protocol-dissectors, ingestion-open-subthreads, central-deployment-access-layer,
satellite-mesh, capability-descriptor-pushdown.

Dated 2026-06-27 (this session):
- `edge-ml-mergeability-depth-2026-06-27.md` — C7 fold research (mergeable-exact representations)
- `prismql-asof-version-resolution-2026-06-27.md` — C8 fold research (bitemporality)
- `queryio-competitive-refresh-2026-06-27.md` — C10 discussion research
- `config-authority-narrow-git-2026-06-27.md` — C9 narrow-git authority research
- `git-as-primary-vs-write-behind-2026-06-27.md` — C9 git-as-primary research
- `bootstrap-config-recovery-2026-06-27.md` — C9 bootstrap recovery research
- `dual-deployment-saas-onprem-2026-06-27.md` — dual-deployment research
- **`config-schema-versioning-migration-2026-06-27.md`** — C9 Q3 schema-versioning migration research

C11–C20 pre-B track (banked 2026-06-27, committed):
- **`prism-context-kg-vector-2026-06-27.md`** — C12: knowledge graph + vector DB + entity mapping + Entity 360 expansion. Aletheon (`/Users/jmagady/Dev/aletheon_2`) is a thin BMAD scaffold — pull only: lists→graph thesis, vendor-first/OCSF-layer ingestion, air-gap milestone. Do NOT pull aletheon's cross-client community-defense (collides with per-tenant isolation + AD-017).
- **`prism-intel-threat-advisory-2026-06-27.md`** — C11: hosted threat-intel / auto-advisory tied to Entity 360
- **`prismql-actions-soar-onprem-models-2026-06-27.md`** — C15: Actions in PrismQL, SOAR + on-prem model integration (Action·Recommendation·Observation)
- **`nerc-cip-support-2026-06-27.md`** — C20: NERC CIP compliance support (synthesizes C16/C17/C18/C19/C2/OT; scheduled LAST in pre-B track)
- **`central-surfacing-ripple-analysis-2026-06-27.md`** — ADS basis research: surfacing model options, ripple audit inputs, cross-cutting principle derivation; used to author ARCHITECTURE-DESIGN-SYSTEM.md and lock Option 3
- **`aro-model-depth-2026-06-27.md`** — C15 depth: OODA/MAPE-K/Endsley grounding; 3-tier first-class ARO; ~57%-unfaithful-RAG-citation caveat sharpening C12; autonomy gating; aletheon `aros` generalization. DOUBLE-banked for C15 (this + prior sweep).

## 2. DECISIONS MADE (do not re-litigate)

### From prior sessions (A, C1, storage taxonomy, C2–C8)
- **UI-D5:** web stack = TypeScript SPA (React) + Rust backend; Archivo/Manrope/IBM Plex Mono fonts;
  1898 & Co brand palette; Purple = AI; Admin pinned bottom.
- **Conversational canvas (aletheon_2 spike):** ADOPT as S3 embedded-AI surface.
- **4 prior human decisions:** PrismQL NOT/WITHOUT = BOTH; S3 opt-in by default; SCIM 2.0 in day-2;
  multi-surface UI S2–S4+U1 in brief §1.
- **Federated ingestion (§17):** edge-first locus; pcap in scope; continuous-operator phased;
  temporal semantics explicit; chain-aware replication; dissectors: prism-native Spicy-style engine;
  OT = flagship native-schema-on-read; four-engine storage taxonomy (RocksDB/Iceberg/PostgreSQL/SQLite).
- **C1 central deployment (ADR-PROP-central-deployment-access-layer.md):** MCP Streamable HTTP (rev 2025-06-18)
  + stdio; OAuth 2.1 RS + built-in AS + external IdP; full case-management on bundled Postgres; SSE resumability.
- **C2 satellite mesh (ADR-PROP-satellite-mesh.md):** gRPC-bidi primary transport / NATS-leaf recorded alt;
  per-hop mTLS only — Relay = mTLS-terminator-not-sub-CA; Coordinator/Relay/Edge Satellite nouns; SPIFFE-model
  native-Rust identity; RocksDB store-and-forward; IEC-62443 structural residency (D-C2-12 hard residency invariant);
  diode one-way deferred.
- **C3 pushdown (ADR-PROP-capability-descriptor-pushdown.md):** cost-based-degrade join guard (NOT hard-reject);
  inject-default time window; allow outer/non-equi central-only; audited override hint; fail-closed
  TOML descriptor→Exact/Inexact/Unsupported. DF50.x dynamic-filter=inner-equi-only; no broadcast-size guard;
  no built-in query timeout (tokio+cooperative cancel); HashJoinExec ResourcesExhausted=abort. PIV-1..3.
  **C3 stale-language reconciliation DONE** — matured-vision §12.2 "reject-at-plan-time" language reconciled
  to D-C3-1 (3 `[SUPERSEDED by D-C3-1]` inline markers + 1 label fix in ADR-PROP-dynamic-schema-connectors.md).
- **C4 dynamic-schema connectors (ADR-PROP-dynamic-schema-connectors.md):** boundary-normalization ALL connectors
  incl OCSF sensors; auto-narrow on column removal; WASM code-connector escape-hatch in day-2; quarantine+relabel
  hostile identifiers. Sanitization: NFC→bidi/control reject→unicode-script→UTS#39 skeleton→length cap;
  two-tier identifiers-vs-values; Wasmtime WASI-P2 no-ambient-authority. PIV-C4.
- **C5 SIEM/lake federation (ADR-PROP-siem-lake-federation.md):** "one engine, two TableProviders";
  Iceberg cold tier REAFFIRMED; S3 data-access default + LAKEFORMATION opt-in; residency reject-at-plan-time;
  federation sources = connector PLUGINS; two adapter archetypes; no SIEM bulk-readable;
  OCSF schema-version descriptor axis. (D-C5-5)
- **C6 detection-engine depth (ADR-PROP-detection-engine-depth.md):** backtest=cold-tier-deterministic;
  FP=risk-based-aggregation (suppression-as-code, time-boxed, dashboarded; auto-tune suggests-only);
  MATCH_RECOGNIZE=UserDefinedLogicalNode+custom ExecutionPlan Thompson-NFA; absence=timer-not-anti-join;
  Sigma→PrismQL pySigma-style; correlation-rules→MATCH_RECOGNIZE; fidelity-report.
  AUTO-ROLLBACK: demote-to-shadow auto (full-disable+revert human-gated); corroboration-master-gate
  (corroborated+concentrated→escalate; uncorroborated+dispersed+sustained→demote; ambiguous→hold+escalate);
  per-tenant alert-rate circuit-breaker; CUSUM/ADWIN; shared change-detector primitive with C7;
  shadow→canary auto / canary→production human-gated. PIV-C6-RB-1..9.
- **C7 ML/behavior-analytics (ADR-PROP-ml-behavior-analytics-depth.md):** full pluggable AI-opaque ModelBackend
  day-2 (candle+ort+wasmtime-WASM+tract); per-update changelog+materialization model-replay; dual-rate+quarantine
  drift/poison (ADWIN+Page-Hinkley MUST-BUILD); satellite-edge mergeability primary=mergeable-only
  (D-C7-1 set as default going in); statistical toolkit (Welford/CGL/EWMA/count-min + sketches-ddsketch/
  tdigests/hyperloglogplus); model-state in RocksDB; monoid VPs; entity-cardinality-cap cost-bound.
  **C7 FOLD DONE** — D-C7-1 resolved: EWMA→forward-decay (U,V), reservoir→random-key/bottom-k,
  clustering→BIRCH CF-vectors additive (all mergeable-exact); coarsening≠privacy (corrected category error);
  macro-clustering-drift = the one empirical caveat. PIV-C7-2/3 added; OQ-C7-1 narrowed. Folded into
  ADR-PROP-ml-behavior-analytics-depth.md.
- **C8 PrismQL deliverables (ADR-PROP-prismql-deliverables.md):** piped surface SHIPS day-2 (KQL/PRQL-style→
  same DataFusion plan; expose desugared SQL/EXPLAIN); FIND keyword; SQL/PGQ GRAPH_TABLE multi-hop forward-compat;
  single LSP server (Chumsky Rich+ariadne/Monaco) for console+CLI+NL-agent; native.<source>.<field> namespace
  +retain-originals; Sigma-aligned recipe format+CI fixtures.
  **C8 FOLD DONE** — OQ-C8-ASOF + OQ-C8-OCSFVER resolved via BITEMPORALITY: valid-time + transaction-time;
  one `AS OF KNOWN <T>` knob pins entity-resolution + OCSF-catalog-version; fresh-by-default; prism-novel.
  New cost-gated open item OQ-C8-DATASNAPSHOT (DataFusion+iceberg-rust lack native time-travel). D-C8-2/3;
  PIV-C8-1..6. Folded into ADR-PROP-prismql-deliverables.md.

### Decided this session

#### C9 config-management — FULLY RESOLVED (ADR-PROP-config-management.md, commit b53b22ba)

**Q1 LOCKED — config authority + versioning:**
- ALL config DB-authoritative, UI-only authoring (no hand-edited TOML in production).
- Versioning splits by domain:
  - RUNTIME CONFIG = DB-native temporal/system-versioned history ("git-semantics over the store", no real git,
    in-transaction exactly-once, dissolves dual-write problem).
  - DETECTION CONTENT + RECIPES = real embedded git (git2 0.19.0 production-grade; gix not yet stable) +
    opt-in residency-gated remote (GitHub/remote = day-2 for detection-content/recipes ONLY; off/air-gap by default).
  - Optional async non-authoritative git projection of runtime-config history = nicety only.
- Narrow-cut research verdict: detection-content-only-in-git is the MAINSTREAM pattern.
- High-blast items flagged for canary+rollback ceremony: satellite-trust, connector-defs, pushdown-descriptors,
  retention-policy.

**FAST-REVERT LOCKED:**
- One-action restore of prior generation (FORWARD-ONLY/append — never rewrite history).
- Atomic ArcSwap hot-swap; seconds, no rebuild/restart.
- Satellites self-revert + pick up on next dial-home.
- Same anchor canary auto-rollback uses.
- Applies to hot-reloadable config + detection content; NOT restart-class/bootstrap keys.

**APPROVAL-GATE:** DROPPED from day-2 → DAY-3 (configurable approval/review WORKFLOWS, per-client).

**BOOTSTRAP-RECOVERY LOCKED (4-layer, for restart-class keys):**
1. validate-before-persist: CHEAP checks only (cert parse/expiry, token well-formed); port-bindable +
   store-connects are RACY → boot-time backstops NOT write-time gates.
2. A/B dual-slot: active=last-known-good, pending=new; promote pending→active ONLY after readiness probe.
3. Supervisor watchdog auto-fallback: READINESS probe not liveness; N failed boots→revert+reboot;
   sd-notify 0.5.0/systemd mature; bundled-PID-1=vendor young 0.x (flag for day-2 maturity check).
4. Satellite AUTONOMOUS self-recovery, TIERED LOCAL SIGNAL:
   - Tier-1 local-ready = "confirm"/revert-on-fail.
   - Tier-2 dial-home = ESCALATION not revert; locally-healthy-but-isolated satellite reports DEGRADED,
     does NOT flap.
- Wrapped by fleet-staged bootstrap canary (Azure-Device-Update %+min-count→group-rollback).
- NEW ATTACK SURFACE: security-reviewed safe-mode console → route to security-reviewer before shipping.
- OPEN: NIST-800-82/IEC-62443 fail-safe normative anchor = separate standards pass.

**Q2 CANARY MECHANICS — RESOLVED:**
- Settled-going-in: canary in day-2; rollback action=fast-revert; reuses C6 circuit-breaker + shared
  change-detector primitive.
- **D-C9-Q2-HEALTH:** canary auto-rollback trip signal = INCLUDE SOFT REGRESSIONS (coverage-banner drop,
  availability-cache degradation, query error-rate uptick, empty-result-rate climb, normalization-failure rate)
  IN ADDITION TO hard-failure signals, at a CONSERVATIVE threshold, CORRELATED to this-config-push-hitting-this-cohort.
- **D-C9-Q2-COHORT:** canary cohort unit = CONFIG-SCOPE-DEPENDENT — tenant for tenant-scoped config;
  satellite/site for fleet-distributed config.
- **D-C9-Q2-TIERS:** TWO-TIER apply model — HIGH-BLAST classes get canary; LOW-BLAST config applies directly
  with fast-revert available (no staged cohort).

**Q3 SCHEMA-VERSIONING — RESOLVED (D-C9-Q3-MODEL / D-C9-Q3-SKIP / D-C9-Q3-FORMAT / D-C9-Q3-TIMING):**
- **D-C9-Q3-MODEL:** HYBRID + per-domain split. Runtime-config: serde_version inline field (no external registry).
  Detection-content: git-native schema-version in YAML front-matter. Boot migration: synchronous-at-boot (blocking).
- **D-C9-Q3-SKIP:** Bounded skip-version window + LTS required-stops. On-prem/client-managed skip-version upgrades
  supported within declared window; each LTS release is a mandatory migration stop (no jumping over LTS).
  Window size and cadence defined at GA (mechanism now, window TBD).
- **D-C9-Q3-FORMAT:** Stay serde + RocksDB value-bytes. savefile and serde_version crates evaluated and rejected
  (savefile: binary format lock-in; serde_version: no active maintenance). Standard serde with `schema_version`
  field + migration dispatch is the correct path.
- **D-C9-Q3-TIMING:** Synchronous-at-boot migration. No lazy/deferred migration — boot fails fast if migration
  cannot complete; keeps invariant that running system always has current schema.
- **`#[non_exhaustive]` decoupled from serialization-compat** — category error corrected: `#[non_exhaustive]`
  governs API/binary-compat for Rust consumers; it does NOT guarantee on-disk forward-compat. Schema versioning
  and `#[non_exhaustive]` are orthogonal mechanisms serving different contracts.

#### Deployment matrix — FULLY CAPTURED (ADR-PROP-dual-deployment.md, commit 91abba90)
- Single-codebase + run-time deployment-profile, ~90% shared. Three operating models:
  1. **SaaS** — vendor(Prism/1898)-hosted, vendor-operated, multi-CUSTOMER tenancy.
  2. **MSSP-managed** — customer/MSSP infra, MSSP(1898)-operated, multi-CLIENT tenancy.
  3. **Client-managed** — client infra, client's own SOC operates it, single-org tenancy (no MSSP in loop).
- BYOC zero-access by construction (C2 residency + AD-017 satellite-local creds → SaaS central never sees
  raw data/creds). Strongest SaaS differentiator.
- D-DEPLOY-001..006; PIV-DEPLOY-* recorded. OPEN: OQ-DEPLOY-1 (tenancy-isolation depth: pool/bridge/silo/cell)
  + OQ-DEPLOY-2 (residual BYOC hardening: result-transit residency, metadata-leakage audit, ephemeral dial-home
  tokens, CMEK for central metadata).

#### C7 fold — DONE (commit 613d4ff3)
See C7 entry above. ADR-PROP-ml-behavior-analytics-depth.md updated with resolved D-C7-1, PIV-C7-2/3, narrowed OQ-C7-1.

#### C8 fold — DONE (commit b6fa1465)
See C8 entry above. ADR-PROP-prismql-deliverables.md updated with D-C8-2/3, PIV-C8-1..6, OQ-C8-DATASNAPSHOT.

#### C10 — Query.io competitive refresh — FULLY DECIDED + CAPTURED (ADR-PROP-competitive-positioning.md, commit 0c9ce71e)

All 8 gaps addressed. Human chose the fuller path on every fork — zero declines.

- **GAP-Q1 (OOTB detection content):** OOTB detection content + rule-translation-OUT both in scope.
  Proposed epic: E-DETECTION-CONTENT-001 + E-RULE-XLATE-001 expansion.
- **GAP-Q2 (auditable agent evidence-package):** S3 agent evidence-package + self-QA gate in scope.
  Proposed epic: E-EVIDENCE-PACKAGE-001.
- **GAP-Q3 (A2A transport):** A2A ADDED to day-2 transport alongside MCP.
  Proposed epic: E-A2A-TRANSPORT-001.
- **GAP-Q4 (connector egress):** connector-egress in scope.
  Proposed epic: E-EGRESS-PIPELINE-001.
- **GAP-Q5 (Security-Lake-subscriber):** fold into C5 (SIEM/lake federation).
- **GAP-Q6 (alert-destination fan-out):** alert-destination fan-out in scope.
  Proposed epic: E-ALERT-ROUTING-001 (expand).
- **GAP-Q7 (graph-investigation UX):** graph-investigation UX in scope.
  Proposed epic: E-GRAPH-INVESTIGATION-001.
- **GAP-Q8 (Configure-Schema wizard):** BOTH Configure-Schema wizard AND optional managed-mapping SaaS service.
  Proposed epics: E-CONFIGURE-SCHEMA-WIZARD-001 + E-MANAGED-MAPPING-001.
- **C3 join framing CORRECTED:** research claim of "hard-reject" was wrong; D-C3-1 cost-based-degrade is the
  correct framing. Corrected in ADR-PROP-competitive-positioning.md and propagated to §12.2 via C3 reconciliation sweep.
- **Positioning split:** IDENTITY (agent-native) vs DIFFERENTIATION (OT/air-gap + trust wedge).
  Leading-candidate headline recorded in ADR-PROP-competitive-positioning.md. FINAL headline deferred to
  brief-reframe/B (§5.1 HUMAN sign-off required).

#### C3 reconciliation sweep — DONE (commit f83e3ec7)
- 3 `[SUPERSEDED by D-C3-1]` inline markers added to matured-vision §12.2 (history preserved, not deleted).
- 1 label fix in ADR-PROP-dynamic-schema-connectors.md.
- Stale "reject-at-plan-time" join language now reconciled to D-C3-1 cost-based-degrade throughout.

#### C13 — §16.4 open-items closeout — DONE (commit 09c5584d)
8 residuals from the running open-items/decisions log resolved:
- SSO: deferred integration modality confirmed (external IdP via C1; day-2 SCIM 2.0 scope locked).
- S3-runtime: runtime config stays DB-authoritative (no S3 runtime authority); S3 for cold Iceberg tier only (C5).
- Secret/DEK granularity: per-credential DEK = future enhancement, recorded as OQ-SECRET-DEK-GRANULARITY; current day-2 scope is per-tenant encryption.
- Sequence-sugar: PrismQL piped-surface sequence sugar decisions folded into ADR-PROP-prismql-deliverables (C8).
- PO-ratifications: Product-Owner ratifications for open UX/story questions recorded and closed.
- ML-phasing: C7 OD-3 (on-prem model deployment phasing) reconciled to C7 pluggable ModelBackend decision; WASM/ort/candle all day-2 capable.
- All 8 residuals resolved; §16.4 section marked CLOSED.

#### Architecture Design System (ADS) — CREATED (commit 7c068714)
`ARCHITECTURE-DESIGN-SYSTEM.md` ("The Prism Way") is the canonical architecture analog of the UI design system. ALL day-2 features + B must conform to it.
- **12 Principles** (P-ADS-01..12)
- **11 Patterns** (PAT-ADS-01..11)
- **8 Invariants** (INV-ADS-01..08) + conformance checklist
- **10 Anti-patterns**
- Basis: `research/central-surfacing-ripple-analysis-2026-06-27.md`

Every remaining C-item (C15, C14, C19, C18, C16, C17, C20) and B must pass the ADS conformance checklist before capture. The conformance pass against existing ADR-PROPs was its first run (10 ripple-audit items closed across 9 ADR-PROPs — see conformance pass entry below).

#### Surfacing model — LOCKED = Option 3 (commit 7c068714)
**Central-Sole-Surface** is the standing cross-cutting principle:
- Tenant-Keyed-Central-Cache always.
- Derived-results-only at central (satellites compute locally; central never holds raw sensor data).
- Operator-zero-access-at-rest.
- "Central blind" = operator-blind, NOT client-blind (clients interact only at central; satellites are headless except setup/maintenance).
Recorded in `ARCHITECTURE-DESIGN-SYSTEM.md` as INV-ADS cross-cutting invariant. Resolves the surfacing-model fork; do not re-litigate.

#### Conformance pass — DONE (commit 7c068714)
10 ripple-audit items closed across 9 ADR-PROPs:
- S3 stale-text struck from dual-deployment.
- PIV-C11-001 consented-exception recorded.
- OQ-DEPLOY-2 disaggregated including CMEK resolved by Option-3 (central metadata now tenant-keyed; CMEK constraint satisfied).
- Central-blind nuance corrected in C12 / dual-deployment / central-deployment and related docs.
- PostgreSQL-control-plane-only prohibition added.
- OCSF-format-vs-PII note added where applicable.

#### C12 — Prism Context — FULLY DECIDED + CAPTURED (ADR-PROP-prism-context.md, commit 76f1a3e2)
- **Embedding substrate:** Two-layer embedded (indradb + usearch + lancedb). Cozo rejected (query language overhead, limited ecosystem).
- **Embeddings:** fastembed/ort+candle (local, air-gap capable, no external embedding API).
- **Entity resolution:** deterministic-only entity-resolution + suspected-links (probabilistic links surfaced as "suspected", not asserted).
- **Retrieval:** Perplexity-style hybrid retrieval (dense + sparse + graph traversal) + mandatory citations on every answer.
- **GraphRAG phasing:** PHASED — Phase 1 = local-search (subgraph extraction, neighborhood queries); Phase 2 = full community-summarization (Leiden clustering, hierarchical summaries).
- **Aletheon CORRECTED:** aletheon DOES have a memory spike: Apache AGE + pgvector on single Postgres, OT asset graph with control/process edges, `aros` table, institutional-memory thesis. AGE+pgvector recorded as DEFERRED central-tier substrate option. Aletheon `aros` table banked as C15 input.
- **~57% unfaithful RAG caveat:** documented in C12 ADR-PROP; citation requirements and grounding discipline directly address this. Sharpened further by ARO depth research.

#### C15 — Actions in PrismQL / SOAR + on-prem models — FULLY DECIDED + CAPTURED (ADR-PROP-soar-actions-aro.md, commit b6314532)

- **Recommend-only v1:** v1 ships Observation + Recommendation only; Action tier deferred to post-v1 autonomy ladder (hardware-write risk + organizational-readiness gating).
- **Autonomy ladder designed/enable-post-v1:** three-rung ladder (Observe / Recommend / Act) with per-tenant autonomy cap ceiling; enables graduated activation without re-architecture.
- **Three typed entities over common base:** Observation, Recommendation, Action share a common `AroBase` (id, timestamp, tenant, provenance, confidence, citations); each adds tier-specific fields.
- **Dual Recommendation sources:** (a) S3 agent layer (conversational / multi-step reasoning); (b) read-only PrismQL `RECOMMEND` projection (deterministic, auditable, plan-time query path); source discriminated via `provenance.source` field; both tested with perimeter-compile-fail gate.
- **Full recommendation rigor day one:** W3C-PROV provenance graph, calibrated confidence scores (isotonic regression), conformal prediction sets, mandatory post-hoc citation-faithfulness check (RAG fidelity gate).
- **Separate `prism-orchestration` Action layer:** HITL approval gates (configurable: auto / human-confirm / human-initiate); idempotency tokens; dry-run mode mandatory before first live run; structured rollback plans required; write-credentials reference-based (AD-017 — never transit AI context).
- **On-prem model deployment via C7 ModelBackend:** central tier = Qwen3 / Mistral; edge tier = Phi-4-mini / Ministral; guardrails = Llama Prompt Guard; all air-gap capable; model provenance signed.
- **ARO linkage rides C12 graph:** AROs are nodes in the Prism Context knowledge graph; entity 360 surfaces ARO history; cross-ARO causation tracked via PROV-O `wasDerivedFrom`.
- **ADS conformance:** all 8 INV-ADS pass (reviewed pre-capture); no anti-patterns; Option-3 Central-Sole-Surface satisfied (action dispatch central; satellite headless for action execution only).

#### C11 — Prism Intel — FULLY DECIDED + CAPTURED (ADR-PROP-prism-intel.md, commit 7c068714)
- **Distribution model:** Feed-down / match-at-edge (central is blind to inventory by construction; intel delivered to satellite; matching happens at edge).
- **SaaS non-BYOC exception:** opt-in central-match for non-BYOC SaaS tenants (explicit consent required; default = edge-match only).
- **Metering:** deployment-conditional — SaaS: vendor-metered; MSSP-managed / client-managed: license-based.
- **Tiers:** free public feeds + paid curated/analyst feeds.
- **Air-gap:** reuse C9 signed-bundle mechanism for air-gap intel delivery.
- **PSI (Private Set Intersection):** evaluated and rejected (complexity vs. benefit; BYOC zero-access already satisfied by feed-down architecture).

#### C19 — Nested tenancy — FULLY DECIDED + CAPTURED (ADR-PROP-nested-tenancy.md, this checkpoint)
- **Tree model:** Adjacency list + closure table + materialized path — three complementary representations; adjacency = source-of-truth writes; closure = O(1) subtree queries; materialized-path = display breadcrumbs and prefix-routing.
- **SF-1 (inheritance):** Hybrid — policy + role explicit-override at any node; configurable propagation modes (inherit/override/isolate) per policy class; OQ-DEPLOY-1 CLOSED.
- **SF-2 (depth):** Unlimited tree depth + configurable soft-cap (default 8) + operator hard-override; no arbitrary depth ceiling in the data model.
- **SF-3 (parent-visibility):** Visibility-grant matrix; P3 (transparent-subtree to parent) gated to same-legal-entity + re-encrypt-on-grant + transient key custody (KEK never at rest on parent); parent-as-grantee for cross-org tenants forbidden (AP-ADS-11); regulatory_class field overrides visibility grant for CUI/BCSI.
- **SF-4 (encryption):** Configurable flat-DEK-or-nested-KEK; per-node DEK default; MSSP-managed = client-held CMEK by default; P-ADS-02 sharpened to distinguish unmediated-at-rest access from authorized-mediated access.
- **SF-5 (reparenting):** Audited reparenting (full audit trail, policy re-evaluation at move time, orphan-prevention guard); cross-legal-entity moves gated by regulatory_class check.
- **MSSP reconciliation:** P-ADS-02 sharpened; MSSP-managed operating model uses the bridge-node pattern (MSSP_ROOT → client subtrees); client-held CMEK is the default key custody; no cross-client data path by construction.
- **ADS conformance:** all 8 INV-ADS pass; AP-ADS-11 (Cross-Tenant DEK Grantee) added to ADS anti-patterns; Option-3 Central-Sole-Surface satisfied (tree stored at central; satellites receive subtree views only).

## 3. THE C-PROGRAM (the active plan — "do each remaining day-2 area")
Each area: research → discuss → decide → capture → commit. **B = integration capstone LAST.**

| Item | Status | Notes |
|---|---|---|
| A — ingestion open sub-threads | ✅ decided + captured (§17.15) | |
| C1 — central deployment / access layer | ✅ decided + captured | |
| Storage taxonomy | ✅ decided + captured | |
| C2 — satellite mesh | ✅ decided + captured (ADR-PROP-satellite-mesh.md) | |
| C3 — capability-descriptor + PrismQL pushdown | ✅ decided + captured (ADR-PROP-capability-descriptor-pushdown.md) | Reconciliation sweep ✅ DONE (stale hard-reject language → D-C3-1) |
| C4 — dynamic-schema / configure-schema connectors | ✅ decided + captured (ADR-PROP-dynamic-schema-connectors.md) | |
| C5 — SIEM / Security-Lake federation | ✅ decided + captured (ADR-PROP-siem-lake-federation.md) | |
| C6 — detection engine + rule-editor depth + auto-rollback | ✅ decided + captured (ADR-PROP-detection-engine-depth.md) | |
| C7 — ML / behavior analytics depth | ✅ decided + captured (ADR-PROP-ml-behavior-analytics-depth.md) | ✅ FOLD DONE (D-C7-1 resolved; PIV-C7-2/3 + OQ-C7-1 narrowed) |
| C8 — PrismQL deliverables | ✅ decided + captured (ADR-PROP-prismql-deliverables.md) | ✅ FOLD DONE (bitemporality; D-C8-2/3; PIV-C8-1..6; OQ-C8-DATASNAPSHOT) |
| C9 — config-management | ✅ FULLY RESOLVED (ADR-PROP-config-management.md) | Q3 schema-versioning resolved: HYBRID + bounded skip-version + serde + sync-at-boot |
| C10 — Query.io competitive refresh | ✅ decided + captured (ADR-PROP-competitive-positioning.md) | All 8 gaps addressed; C3 corrected; positioning identity-vs-differentiation; headline deferred to B |
| Deployment matrix | ✅ decided + captured (ADR-PROP-dual-deployment.md) | Three operating models; BYOC zero-access by construction |
| C3 reconciliation sweep | ✅ DONE | §12.2 stale language reconciled; label fix in ADR-PROP-dynamic-schema-connectors.md |
| **B — integration capstone** | ⏳ LAST | brief-reframe deltas + consolidated epic/ADR/story list — GATED on brief-reframe HUMAN sign-off §5.1 |

## PRE-B FEATURE TRACK (C11–C20)

A new batch of feature areas was confirmed BEFORE B — all require research + discussion + decisions + capture. Research is banked for C12 ✅, C11 ✅, C15 (double-banked) ✅, C14 ✅, and ALL remaining items (C19, C18, C16, C17, C20 — commit 664cbbd1). B remains the final capstone, gated on §5.1 brief-reframe HUMAN sign-off.

**Dependency-aware order (human-confirmed): C13 ✅ → C12 ✅ → C11 ✅ → C15 ✅ → C14 ✅ → C19 ✅ → C18 → C16 → C17 → C20 → B**

**Cross-cutting conformance frame:** ALL remaining items (C15, C14, C19, C18, C16, C17, C20, B) must pass the ADS conformance checklist (`ARCHITECTURE-DESIGN-SYSTEM.md`) before capture. The first conformance pass (10 ripple-audit items across 9 ADR-PROPs) is done.

| ID | Area | Status | Cross-links |
|---|---|---|---|
| C13 | §16.4 open-items closeout (SSO, S3-runtime, secret/DEK, sequence-sugar, PO-ratif, ML-phasing) | ✅ DONE (commit 09c5584d) — 8 residuals resolved; per-credential-DEK = future enhancement (OQ-SECRET-DEK-GRANULARITY); ml OD-3 reconciled to C7 | C7, C2/AD-017, C16, C20 |
| **ADS** | Architecture Design System — "The Prism Way" (12 Principles, 11 Patterns, 8 Invariants, 10 Anti-patterns, conformance checklist) | ✅ DONE (commit 7c068714) — `ARCHITECTURE-DESIGN-SYSTEM.md` is the canonical conformance frame for all remaining work | All C-items + B |
| **Option-3** | Surfacing model locked — Tenant-Keyed-Central-Cache always; derived-results-only; operator-zero-access-at-rest; Central-Sole-Surface principle | ✅ LOCKED (commit 7c068714) — do not re-litigate | ADS INV-ADS, C12, C11, C15 |
| **Conformance pass** | Ripple audit of existing ADR-PROPs against ADS + Option-3 | ✅ DONE (commit 7c068714) — 10 items closed across 9 ADR-PROPs | All prior C-items |
| C12 | Prism Context — knowledge graph + vector DB + entity mapping + Entity 360 expansion | ✅ DONE (commit 76f1a3e2) — ADR-PROP-prism-context.md; two-layer embedded (indradb+usearch+lancedb); fastembed/ort+candle; deterministic entity-resolution + suspected-links; hybrid retrieval + mandatory citations; phased GraphRAG; aletheon corrected | aletheon `aros` table → C15 input, AD-017, C16, C7 |
| C11 | Prism Intel — hosted threat-intel/auto-advisory tied to Entity 360 | ✅ DONE (commit 7c068714) — ADR-PROP-prism-intel.md; feed-down/match-at-edge; opt-in central-match for non-BYOC SaaS; deployment-conditional metering; free public + paid curated tiers; C9 signed-bundle for air-gap; PSI rejected | C12 Entity 360, C9 signed-bundle, BYOC zero-access, Option-3 |
| C15 | Actions in PrismQL / SOAR + on-prem models (Action·Recommendation·Observation) | ✅ DONE (commit b6314532 — `ADR-PROP-soar-actions-aro.md`) — recommend-only v1 + autonomy ladder designed/enable-post-v1; three typed entities (Observation/Recommendation/Action) over common base; Recommendations from BOTH S3 agent layer AND read-only PrismQL `RECOMMEND` projection (perimeter-compile-fail-tested, source-discriminated provenance); full rec-rigor day one (W3C-PROV + calibrated confidence + conformal sets + mandatory post-hoc citation-faithfulness); separate `prism-orchestration` Action layer (HITL gates, idempotency, dry-run, rollback, AD-017 reference-based write-creds); on-prem models (Qwen3/Mistral central, Phi-4-mini/Ministral edge, Llama Prompt Guard guardrails) via C7 ModelBackend; ARO linkage rides C12 graph; ADS-conformant (all 8 INV pass) | C7 ModelBackend, S3 agent, C10 GAP-Q2, C18, AD-017, aletheon aros |
| C14 | Active-query device support (Industrial Defender class) | ✅ DONE (commit 59864881 — `ADR-PROP-active-query-devices.md`) — Reading A+B both in v1 (federate OT-platform APIs + direct OT-protocol field-device polling, poller-of-last-resort); active-query = capability-axis on C3/C4 (not a new connector class); read-only-perimeter (writes via C15); OT-safety guardrails as hard invariants; OT asset/config/vuln modeled as OCSF source tables; Reading-B protocol libs as plugins/sidecar; ADS-conformant | C3/C4 connectors, C15 (write perimeter), C20 (OT/ICS) |
| C19 | Nested tenancy (parent→child→… unlimited) | ✅ DONE (ADR-PROP-nested-tenancy.md) — bridge + per-node isolation_tier closes OQ-DEPLOY-1; adjacency+closure+materialized-path tree; SF-1 hybrid / SF-2 unlimited+soft-cap-8 / SF-3 visibility-grant matrix (P3 transparent-subtree gated same-legal-entity; (c) forbidden→AP-ADS-11; regulatory_class override) / SF-4 configurable flat-DEK-or-nested-KEK / SF-5 audited reparenting; MSSP reconciliation sharpened P-ADS-02; MSSP key custody default = client-held CMEK | CLOSES OQ-DEPLOY-1; C18 (role inheritance), C16 (detokenize-at-surface) |
| C18 | RBAC depth (into connectors/satellites) | 🔬 research BANKED (commit 664cbbd1 — `research/rbac-depth-2026-06-27.md`) — layered RBAC+ReBAC+ABAC, engine build-vs-adopt fork | CLOSES C10 Query-RBAC gap; extends C9 / E-CENTRAL-AUTHZ-001; C19 (role inheritance), C16 (detokenize-at-surface), C15 (approver roles), C17 (key-escrow Option-3 CMEK) |
| C16 | Entity masking / tokenizing clearing house (AI-opaque data; BCSI→universal name) | 🔬 research BANKED (commit 664cbbd1 — `research/entity-masking-tokenization-2026-06-27.md`) — edge clearing-house, vaulted tokenization, RSI naming | extends AD-017; C20 confirms BCSI canonical + recommends "RSI" abstraction; C18 (detokenize-at-surface via RBAC) |
| C17 | Backup & recovery (first-class) | 🔬 research BANKED (commit 664cbbd1 — `research/backup-recovery-2026-06-27.md`) — logical-watermark+per-store time-travel tied to C8, key-escrow fork | C9 config + bootstrap-recovery; C20 CIP-009; C18 (key-escrow Option-3 CMEK) |
| C20 | NERC CIP support | 🔬 research BANKED (commit 664cbbd1 — `research/nerc-cip-support-2026-06-27.md`) — SCHEDULED LAST (synthesizes C16/C17/C18/C19/C2/OT) | C16(BCSI/RSI), C17, C18, C19, C2, OT |
| B | Integration capstone | ⏳ LAST — gated on §5.1 brief-reframe HUMAN sign-off | — |

## 4. PENDING FOLDS

### C7 FOLD — EXECUTED ✅
Resolved D-C7-1: representation-change escape hatches make non-mergeable primitives mergeable-EXACT.
EWMA→forward-decay (U,V), reservoir→random-key/bottom-k, clustering→BIRCH CF-vectors additive.
Edge-ML uses mergeable-exact representations broadly; coarsening≠privacy (local-DP separate);
empirical test narrows to macro-clustering drift. Folded into ADR-PROP-ml-behavior-analytics-depth.md (commit 613d4ff3).

### C8 FOLD — EXECUTED ✅
Resolved OQ-C8-ASOF + OQ-C8-OCSFVER via BITEMPORALITY: valid-time + transaction-time;
one `AS OF KNOWN <T>` decision-time knob pins entity-resolution + OCSF-catalog-version + ideally
C5/C6 data snapshot for reproducible forensics; fresh-by-default. DB-native temporal; prism-novel.
CAVEAT: DataFusion+iceberg-rust lacks native time-travel today (OQ-C8-DATASNAPSHOT = cost-gated open item).
Folded into ADR-PROP-prismql-deliverables.md (commit b6fa1465).

## 5. EXACT NEXT ACTION ON RESUME

**C19 ✅ DONE. C14 ✅ C15 ✅ C12 ✅ C11 ✅ done. ADS created, Option-3 locked, conformance pass done. The pre-B feature track continues at C18.**

**NEXT: C18 — RBAC depth (into connectors/satellites)**
- Research IS banked: `research/rbac-depth-2026-06-27.md` (commit 664cbbd1) — layered RBAC+ReBAC+ABAC, engine build-vs-adopt fork.
- Proceed directly to discussion + decision + capture (ADS conformance checklist required before capture).
- Cross-wiring to note: C18 RBAC role-inheritance ↔ C19 (role, scope-node)+closure-table model just decided (ADR-PROP-nested-tenancy.md). Decisions must be mutually consistent across C18↔C19↔C16↔C15↔C17.

**C14 follow-up research BANKED (committed alongside this checkpoint):**
- `research/ocsf-ot-coverage-2026-06-27.md` — C14 sub-fork F4: OCSF-OT schema coverage. COMPLETE. Committed in same burst as this resume-doc edit (path-scoped).
  **Findings (closes OQ-C14-OCSF):** OCSF v1.8.0 (2026-03-18). Asset identity (Device Inventory Info 5001), config baseline/drift/exception (Device Config State 5002/5019 + Compliance Finding 2003), and device-vuln (Vulnerability Finding 2002) all FIT CLEANLY into the C14 OCSF source-table model. OT semantics (PLC/RTU/HMI type, firmware, Purdue zone, control topology) NEED a private `prism_ot` extension — the Device type_id enum is IT-only in OCSF core. CPE-matching is a gap (flag — ties to C11 intel enrichment). Data Classification profile SHIPS (v1.5.0+) — adopt as-is for C16/RSI tagging. Upstream ICS work (OCSF Issue #1515, Corelight) is STALLED — **lean = author private `prism_ot` extension now (NOT upstream-first), reserving clean UIDs for future upstream contribution.** 5 sub-forks (SF-1..5) recorded in the research file.

**Research status for remaining items (ALL banked as of commit 664cbbd1):**
- C19: research BANKED — `research/nested-tenancy-2026-06-27.md`
- C18: research BANKED — `research/rbac-depth-2026-06-27.md`
- C16: research BANKED — `research/entity-masking-tokenization-2026-06-27.md`
- C17: research BANKED — `research/backup-recovery-2026-06-27.md`
- C20: research BANKED — `research/nerc-cip-support-2026-06-27.md` (scheduled LAST)

**Ordered queue — work through left-to-right, do not skip:**
1. **C18** — RBAC depth — **NEXT** (research banked; proceed to discussion + decision + capture; note cross-wiring with C19 closure-table model)
2. C16 — Entity masking / tokenizing clearing house (research banked)
3. C17 — Backup & recovery (research banked)
4. C20 — NERC CIP (research banked; scheduled LAST — synthesizes all prior)
5. **B — integration capstone** (GATED on §5.1 brief-reframe HUMAN sign-off)

Each remaining item must conform to `ARCHITECTURE-DESIGN-SYSTEM.md` conformance checklist before capture.

**B IS GATED on brief-reframe HUMAN sign-off (§5.1 of matured-vision doc).** Do not begin B until the
human confirms the brief-reframe direction. The leading-candidate headline is recorded in
ADR-PROP-competitive-positioning.md and requires human approval before the brief is updated.

Superseded capture queue (all DONE — do not re-execute):
- ~~C9 capture → ADR-PROP-config-management.md~~ ✅ b53b22ba
- ~~Deployment-matrix capture → ADR-PROP-dual-deployment.md~~ ✅ 91abba90
- ~~C7 fold → update ADR-PROP-ml-behavior-analytics-depth.md~~ ✅ 613d4ff3
- ~~C8 fold → update ADR-PROP-prismql-deliverables.md~~ ✅ b6fa1465
- ~~C10 discussion + capture → ADR-PROP-competitive-positioning.md~~ ✅ 0c9ce71e
- ~~C3 reconciliation sweep~~ ✅ f83e3ec7
- ~~C13 §16.4 open-items closeout~~ ✅ 09c5584d
- ~~C12 discussion + capture → ADR-PROP-prism-context.md~~ ✅ 76f1a3e2
- ~~ADS creation + Option-3 lock + conformance pass~~ ✅ 7c068714
- ~~C11 discussion + capture → ADR-PROP-prism-intel.md~~ ✅ 7c068714
- ~~ARO depth research for C15~~ ✅ 9199174c
- ~~C15 discussion + capture → ADR-PROP-soar-actions-aro.md~~ ✅ b6314532
- ~~C14 discussion + capture → ADR-PROP-active-query-devices.md~~ ✅ 59864881
- ~~C19/C18/C16/C17/C20 research banking~~ ✅ 664cbbd1
- ~~C19 discussion + capture → ADR-PROP-nested-tenancy.md~~ ✅ (this checkpoint)

## 6. BASELINE (git state at session wrap)

- **factory-artifacts HEAD (current):** `(this checkpoint)` — C19 nested-tenancy decided + captured (see commit chain below)
- **Full commit chain for this day-2 side-analysis** (day-2 side commits; live factory interleaved its own commits on the shared branch between these):
  - `b53b22ba` — C9 schema-versioning (Q3) resolved + ADR-PROP-config-management + backing research
  - `91abba90` — deployment matrix captured (ADR-PROP-dual-deployment, three operating models)
  - `613d4ff3` — C7 fold done (D-C7-1 resolved, ADR-PROP-ml-behavior-analytics-depth updated)
  - `b6fa1465` — C8 fold done (bitemporality, ADR-PROP-prismql-deliverables updated)
  - `0c9ce71e` — C10 competitive refresh decided + captured (ADR-PROP-competitive-positioning, all 8 gaps)
  - `f83e3ec7` — C3 reconciliation sweep (§12.2 hard-reject language → D-C3-1; label fix in C4 ADR-PROP)
  - `09c5584d` — C13 §16.4 open-items closeout + 4× C11–C15/C20 banked research files committed
  - `76f1a3e2` — C12 decided + ADR-PROP-prism-context.md captured (two-layer embedded; fastembed; phased GraphRAG; aletheon corrected)
  - `7c068714` — ADS created (ARCHITECTURE-DESIGN-SYSTEM.md) + Option-3 surfacing locked + conformance pass (10 ripple items) + C11 decided + ADR-PROP-prism-intel.md captured
  - `9199174c` — ARO depth research banked for C15 (aro-model-depth-2026-06-27.md)
  - `996f1d1d` — resume snapshot refresh (SESSION-RESUME-2026-06-27.md updated to current state)
  - `b6314532` — C15 decided + ADR-PROP-soar-actions-aro.md captured (recommend-only v1 + autonomy ladder + dual-source ARO + on-prem ModelBackend; ADS-conformant)
  - `abe5b077` — [live factory interleave — see factory track]
  - `59864881` — C14 decided + ADR-PROP-active-query-devices.md captured (Reading A+B in v1; active-query as capability-axis; OT-safety guardrails; OCSF OT source tables; Reading-B protocol libs as plugins/sidecar; ADS-conformant)
  - `664cbbd1` — C19/C18/C16/C17/C20 research files banked (nested-tenancy, rbac-depth, entity-masking-tokenization, backup-recovery, nerc-cip-support)
  - `02599c9b` — ocsf-ot-coverage-2026-06-27.md banked + SESSION-RESUME-2026-06-27.md updated; path-scoped commit
  - `(this checkpoint)` — C19 nested-tenancy decided + ADR-PROP-nested-tenancy.md captured + SF-3 parent-visibility research banked + ADS amended (P-ADS-02 sharpened, AP-ADS-11 added); path-scoped commit
- Note: live-factory commits interleaved on `factory-artifacts` between the above (normal concurrent operation).
- Working tree otherwise clean (untracked `.DS_Store` only; live-factory files left unstaged). No dangling in-flight research.

## 7. Gaps / epics introduced (PROPOSED, not in STORY-INDEX)
Gaps **G-1 … G-36** (plus new cross-cutting dual-deployment gap). Proposed epics:
E-CACHE-DEMAND, E-CENTRAL-TRANSPORT/AUTHZ/OPS, E-SATELLITE-MESH, E-LAKE-CONNECTOR,
E-UI-ADMIN/CONSOLE/EMBEDDED-AI/EXTENSION, E-CONNECTOR-DYNAMIC, E-DETECT-*, E-RULE-XLATE,
E-ML-*, E-COLLECTOR-CLASS/PCAP, E-CHAIN-CACHE, E-STREAM-DETECT, E-DISSECTOR-NATIVE/OT,
E-CONFIG-MGMT-001 (C9 — config authority + schema-versioning + canary),
E-DUAL-DEPLOYMENT-001 (three operating models + deployment-profile mechanism).

**New C10 proposed epics (all human-approved, fuller path chosen):**
- E-DETECTION-CONTENT-001 — OOTB detection content library
- E-RULE-XLATE-001 (expansion) — rule translation pipeline (Sigma→PrismQL + bidirectional)
- E-EVIDENCE-PACKAGE-001 — auditable agent evidence-package + self-QA gate
- E-A2A-TRANSPORT-001 — Agent-to-Agent (A2A) transport layer (alongside MCP)
- E-EGRESS-PIPELINE-001 — connector-egress / data-exfiltration pipeline
- E-CONFIGURE-SCHEMA-WIZARD-001 — guided schema configuration wizard (UI)
- E-MANAGED-MAPPING-001 — optional managed-mapping SaaS service
- E-GRAPH-INVESTIGATION-001 — graph-investigation UX (multi-hop traversal + visualization)
- E-ALERT-ROUTING-001 (expand) — alert-destination fan-out / routing rules

**New C11–C15 proposed epics (C12 + C11 decided; C15 research double-banked, decision pending):**
- E-PRISM-CONTEXT-001 — Prism Context: knowledge graph + vector DB + entity mapping (C12 ✅ DECIDED); two-layer embedded (indradb+usearch+lancedb); fastembed/ort+candle; phased GraphRAG; OT/asset-layer ingestion; OCSF-layer graph construction; air-gap milestone; mandatory citations + hybrid retrieval
- E-PRISM-INTEL-001 — Prism Intel: hosted threat-intel enrichment + auto-advisory engine tied to Entity 360 (C11 ✅ DECIDED); feed-down/match-at-edge architecture; deployment-conditional metering; C9 signed-bundle for air-gap; PSI rejected
- E-SOAR-ACTIONS-001 — Actions in PrismQL: Action/Recommendation/Observation surface + SOAR connector egress (C15 ✅ DECIDED — ADR-PROP-soar-actions-aro.md); recommend-only v1 + autonomy ladder; dual-source ARO (S3 agent + PrismQL RECOMMEND projection); W3C-PROV + conformal confidence; HITL gates + dry-run + rollback; AD-017 reference-based write-creds; on-prem ModelBackend (Qwen3/Mistral/Phi-4-mini)
- E-ONPREM-MODELS-001 — On-premises model execution: ModelBackend local-deployment profile, air-gap inference, model provenance / signing (C15 on-prem dimension; extends C7 pluggable ModelBackend)

**New C14 proposed epics:**
- E-ACTIVE-QUERY-001 — Active-query / polling device support: capability-axis on C3/C4 connector machinery; OT asset/config/vuln as OCSF source tables; Reading-A (OT-platform API federation) + Reading-B (direct OT-protocol field-device polling, poller-of-last-resort); read-only perimeter (writes gated via C15 autonomy ladder); OT-safety guardrails as hard invariants; ADS-conformant
- E-OT-PROTOCOL-CONNECTORS-001 — OT protocol connector library: Reading-B plugin/sidecar architecture for direct field-device polling (Modbus, EtherNet/IP, DNP3, IEC 61850, BACnet, OPC-UA); protocol libs as WASM plugins or sidecar processes; air-gap capable; phased rollout (platform-native APIs first, direct protocol second)

**New C19 proposed epics (PROPOSED):**
- E-NESTED-TENANCY-001 — Nested tenancy: adjacency+closure+materialized-path tree; unlimited depth + soft-cap-8 + soft-cap-override; per-node isolation_tier (bridge/silo-subtree/cell-subtree); audited reparenting; MSSP-managed operating model multi-client hierarchy; ADS-conformant (ADR-PROP-nested-tenancy.md)
- E-TENANT-VISIBILITY-001 — Tenant parent-visibility controls: SF-3 visibility-grant matrix; P3 transparent-subtree gated to same-legal-entity + re-encrypt + transient key custody; parent-as-grantee forbidden (AP-ADS-11); regulatory_class override; MSSP CMEK default = client-held

**ADS conformance frame:** `ARCHITECTURE-DESIGN-SYSTEM.md` is the canonical conformance reference for all epics above and all remaining C-items. Every epic capture must satisfy the ADS conformance checklist (12 Principles, 11 Patterns, 8 Invariants, no anti-patterns) and the Central-Sole-Surface (Option-3) invariant.

All deferred to morph (brief-reframe §5.1 HUMAN GATE).
