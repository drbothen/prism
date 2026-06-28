---
document_type: session-resume
status: capture
do_not_execute: true
timestamp: "2026-06-27"
provenance: "Day-2 vision SIDE-ANALYSIS program resume snapshot. SEPARATE from the live factory pipeline. Does NOT modify .factory/STATE.md or SESSION-HANDOFF.md (those belong to the live factory track)."
---

# Day-2 Vision Side-Analysis — Zero-Context Resume Snapshot (2026-06-27)

> **Original C-program (C1–C10 + deployment-matrix + C3 hard-reject reconciliation) COMPLETE and committed.
> PRE-B feature track: C13 ✅ C12 ✅ C11 ✅ C15 ✅ C14 ✅ C19 ✅ C18 ✅ C16 ✅ C17 ✅ C20 ✅ — C20 NOW FULLY RESOLVED (SF-2 folded = Defer + Leave-Seams-Open).
> ENTIRE pre-B feature track (C11–C20) is COMPLETE. ADS v1.7 (traceability rows C10/C14/C15 + freshness fixes). Only B (integration
> capstone) remains, GATED on §5.1 brief-reframe HUMAN sign-off.**
>
> **Latest factory-artifacts HEAD: `(this checkpoint)` — prism-as-ot-sensor-note.md working note captured (passive OT-sensor finding + 5 iterate-later threads); ADS v1.7; pre-B track C11–C20 COMPLETE**
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
  (the live factory pushes concurrently); never `--no-verify`/`--force`. Last side HEAD: **55963381**
  (prism-as-ot-sensor-note.md working note captured; prior: plain-language executive positioning narrative).
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

#### Problem-framed positioning CANDIDATE captured (ADR-PROP-positioning-problem-framed.md; 2026-06-28)

A SECOND positioning candidate was captured as `ADR-PROP-positioning-problem-framed.md`. It is an ALTERNATIVE to D-C10-5 (competitor-relative), NOT a supersession of it. Both are the two inputs to the §5.1 brief-reframe decision.

+ Fold-pass B (problem-framed candidate) applied: 22 iterate-list refs — MISAL-01/02/05 (S4=browser-extension, strong-ID set corrected, dropped borrowed 9-check), OVER-04/05/07 (model names→PARTIAL, Kani scope, egress PROPOSED), UNDER-01/02/03/06/07/09/11/12/14 + GAP-01/02/03/04/05/07 + NAME-01 (added passive-OT/Reading-C, bitemporal replay, backtest coverage-map, ML 'model is the memory', enterprise identity SSO/SCIM/RBAC, compliance presets, backup/crypto-shred, PrismQL RECOMMEND, piped surface, bundled-storage air-gap; §8.2 matrix 10→15 columns; S3-vs-AmazonS3 gloss). Fold log appended to the doc. Remaining fold passes: C (diagrams), D (competitive-positioning OVER-05/06 + UNDER-10).

+ Fold-pass C (diagrams) + D (competitive-positioning) applied — the iterate-list is now substantially reconciled. Diagrams: MISAL-03 (3 operating models + air-gap profile, both diagrams), GAP-07 (PCAP retrieve/query + download-PCAP flow, technical), NAME-03 (Detection Engine box relabeled RBA/MATCH_RECOGNIZE/staged-rollout), UNDER-03 (dual-tier backtest + coverage-map label), GAP-01 (Backup/Recovery/DR box), conceptual diagram brought to parity (S1 BYO-agent UNDER-05, edge-masking GAP-06, passive-OT made explicit UNDER-01/MISAL-04); all maturity-tagged honestly. Competitive-positioning: OVER-05 (Kani scoped to parser-safety VP-014/015, not 'verified language'), UNDER-10 (AI-rec rigor W3C-PROV/conformal/faithfulness added to 'what NOT to concede'), OVER-06 (A2A=PROPOSED note added to C1 transport ADR). Remaining iterate-list items: only low/cosmetic + a few medium not folded; the HIGH items are all applied across Pass A–D.

Built on the human-confirmed MANDATED-7 customer problems (cheap hunts / tuning / can't-ship-data-out / IT-watch-OT / device-discovery / OT-environment-context / talent), synthesized into 3 pillars:
- A: data-never-leaves + IT-sees-OT
- B: know-your-OT-environment
- C: affordable-AI-augmented-hunting

3 candidate headlines (H-PF-1..3). Adversarially pressure-tested 2026-06-28 = CONDITIONAL-PASS (0 gap; P2/P3-context SOLID; 5 binding caveats baked in: OOTB-content-empty, Reading-B-native-discovery-gated, OT-OCSF-schema-open-lean, not-data-diode, recommend-only-v1).

brief-reframe §5.1 remains PENDING-human-signoff; B remains gated. + §8 feature-map appendix added — per-problem mandated-7 → satisfying-feature enumeration (maturity-tagged DECIDED/PARTIAL/GATED/PROPOSED) + a 7×10 capability-coverage matrix; every mandated problem served by ≥1 DECIDED feature-cluster.

+ plain-language customer/executive narrative captured (positioning-executive-narrative.md) — jargon-free layman version of the problem-framed positioning, organized as 3 themes (data-home+IT-sees-OT / know-your-environment / affordable-hunting-with-the-team-you-have); all binding honest caveats translated into plain terms; gated on §5.1, do_not_execute, not a sales claim.

+ working note prism-as-ot-sensor-note.md captured — finding: passive OT-sensor capability (full-packet PCAP §17.6 E-COLLECTOR-PCAP-001 + native Spicy-style dissector §17.12 E-DISSECTOR-NATIVE-001/OT-001, OT flagship native-schema-on-read, strict passivity TAP>SPAN) is DECIDED + safe-by-design, distinct from the GATED active-polling path (C14 Reading B); records a positioning correction (OT-standalone is stronger than 'gated' — passive path is decided/safe) to fold later; 5 open threads enumerated (protocol-coverage phasing, unify §17+C14 OT-ingestion narrative, adversarial pressure-test the passive-sensor claim, positioning fold, C20 NERC-CIP passive-monitoring synergy). iterate_later working note; NOT yet folded into positioning.

+ Prism + Prism Satellite C4 diagram set captured under specs/day2-design-decisions/diagrams/ — prism-architecture-conceptual.drawio (C4 L1/L2 exec/customer: your-environment → encrypted-answers-only conduit → Prism Central; 'raw data stays on-site') and prism-architecture-technical.drawio (C4 L2/L3 container: full Central + Satellite component breakdown, OCSF-normalize chokepoint + derived-results-only hard invariants highlighted, maturity legend marking active-polling [GATED], OOTB/A2A/evidence-pkg [roadmap], GraphRAG-Phase2/continuous-operator [phased]); both exported to PNG/SVG/PDF (draw.io CLI 30.0.4). For customer/exec show-and-tell + engineer reference.

+ positioning-fidelity-iterate-list.md captured — master iterate-list from the 2026-06-28 multi-agent fidelity sweep (8 agents, 7 areas): 36 findings (14 undersell / 7 coverage-gap incl. architect-added GAP-07 PCAP-retrieve-affordance / 7 oversell / 5 misalignment / 3 naming; 12 HIGH). Verdict MEDIUM-to-HIGH drift, two patterns: (1) decided high-value capabilities with no customer/exec home (passive-OT sensor, bitemporal AS-OF-KNOWN replay, backup+crypto-shred, compliance profiles, SSO/SCIM/RBAC, on-demand ML, edge masking, MSSP nested tenancy, agent-native identity); (2) exec narrative oversells absolute residency/operator-zero-access (OVER-01/02 HIGH — fix BEFORE §5.1). Pre-§5.1 fixes flagged: OVER-01, OVER-02, MISAL-01 (S4=browser-extension not 'mobile'), MISAL-02 (hostname not a strong-ID auto-merge key). Most folds gated on §5.1. Master backlog for positioning reconciliation.

+ Fold-pass A (exec narrative) applied from positioning-fidelity-iterate-list: corrected OVER-01 (raw-vs-derived residency + jurisdiction caveat) + OVER-02 (operator-zero-access qualified by operating model + new spectrum note) + OVER-03 (auto-rollback = auto-demote/human-revert); added customer-facing homes for GAP-02 compliance presets, GAP-04/UNDER-04 enterprise identity (SSO/SCIM/RBAC), UNDER-05 agent-native/MCP identity, UNDER-02 bitemporal replay, GAP-01 backup+crypto-shred+CIP-evidence, UNDER-08 MSSP isolation, GAP-06 edge masking, UNDER-01 passive-OT (3-path Theme B), UNDER-13 on-box air-gap AI, GAP-07 on-demand PCAP retrieve; 'Where We Are Today' re-tiered (shipping/near-term/capture-stage). All honestly caveated. Pre-§5.1 oversell fixes (OVER-01/02) now applied to the candidate. Path-scoped commit.

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

#### Architecture Design System (ADS) — CREATED (commit 7c068714); AMENDED v1.1 + v1.2 + v1.3 + v1.4 + v1.5 + v1.6 + v1.7 (current)
`ARCHITECTURE-DESIGN-SYSTEM.md` ("The Prism Way") is the canonical architecture analog of the UI design system. ALL day-2 features + B must conform to it.
- **13 Principles** (P-ADS-01..13) — P-ADS-13 Configurable-Not-Prescriptive added in prior burst; P-ADS-07 sharpened in C16 (clearing-house enforcement + embeddings-are-sensitive)
- **17 Patterns** (PAT-ADS-01..17) — PAT-ADS-12 Configurable Compliance Profile + PAT-ADS-13 Layered-Authz added in C18; PAT-ADS-14 Edge-Tokenizing-Clearing-House added in C16; PAT-ADS-15/16 added in C17; PAT-ADS-17 evidence-owner forward-note added in C20
- **10 Invariants** (INV-ADS-01..10) — INV-ADS-09 Decision-Level Authorization Audit added in C18; INV-ADS-10 Recoverability-Preserves-Operator-Zero-Access added in C17 + conformance checklist
- **11 Anti-patterns** (AP-ADS-11 added in C19 burst; now 11 total)
- Basis: `research/central-surfacing-ripple-analysis-2026-06-27.md`
- ADS bumped v1.0→v1.1 (P-ADS-02, AP-ADS-11, P-ADS-13) → v1.2 (PAT-ADS-12/13, INV-ADS-09 — C18) → v1.3 (P-ADS-07, PAT-ADS-14 — C16) → v1.4 (PAT-ADS-15/16, INV-ADS-10 — C17) → v1.5 (PAT-ADS-17 Compliance-Evidence-Export — C20 bulk) → v1.6 (PAT-ADS-17 evidence-owner forward-note — C20 SF-2) → **v1.7** (traceability rows C10/C14/C15 added; cosmetic amendment-log fix — consistency audit 2026-06-28)

**P-ADS-13 — Configurable-Not-Prescriptive (Policy-as-Configuration) — ADDED (prior checkpoint)**
Restrictive security/compliance posture is configurable data + named shippable Profiles (baseline / SOC2 / ISO27001 / IEC-62443-OT / NERC-CIP), never hardcoded absolutes and never a single vertical's needs branched into code. OT is a shipped Profile, not a code fork. Floor: configurability sits ABOVE the INV-ADS invariant floor (operator-zero-access, no-raw-at-Central, per-tenant isolation, AI-opaque are NEVER configurable off). Profiles tighten-only down the tenant tree.

**PAT-ADS-12 + PAT-ADS-13 + INV-ADS-09 — ADDED (this checkpoint, C18)**
PAT-ADS-12 = Configurable Compliance Profile (named/versioned/signed monotone-tighten bundles; OSCAL param+constraint). PAT-ADS-13 = Layered-Authz (RBAC+ReBAC+ABAC three-layer spine; Zanzibar-tuple core). INV-ADS-09 = Decision-Level Authorization Audit (every authz decision logged with subject, resource, action, policy version, result — not just access events).

Every remaining C-item (C16, C17, C20) and B must pass the ADS conformance checklist before capture. The conformance pass against existing ADR-PROPs was its first run (10 ripple-audit items closed across 9 ADR-PROPs — see conformance pass entry below).

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

#### C16 — Entity masking / RSI tokenizing clearing house — FULLY DECIDED + CAPTURED (ADR-PROP-entity-masking.md, this checkpoint)

- **AD-017 extended to AI-opaque DATA:** Prior AD-017 scope was credentials-only. Extended to cover all RSI (Regulated Sensitive Information) fields flowing through the query path and into AI/RAG surfaces. AI never sees raw RSI values.
- **BUILD Prism-native Rust clearing house (SS-26):** Per-tenant DEK + aes-gcm; RocksDB-CF token vault; FF1 FPE optional (narrow-domain ≥10^6 only). Vault Transform Enterprise (HashiCorp) rejected (external runtime dependency, licensing, Vault Enterprise cost). SS-26 is a new subsystem designation.
- **Technique mix keyed by RSI field class:**
  - Deterministic vaulted tokenization = default for structured identifiers (IPs, MACs, hostnames, usernames).
  - FF1 FPE = narrow-domain structured fields where domain size ≥10^6 (e.g., full IPv4 space = 4×10^9).
  - Redaction = fields where token shape reveals sensitivity (e.g., raw credentials, secrets).
  - NER-based tokenization = free-text fields (entity recognition before tokenization; probabilistic, audited).
- **EDGE placement after OCSF normalization — FORCED by INV-ADS-01 / D-C2-12 / Option-3:** Clearing house sits at the edge satellite, immediately after OCSF normalization, before any data leaves the satellite boundary. Central holds surrogates only. Not configurable off.
- **RSI abstraction + pluggable masking profiles:** RSI (Regulated Sensitive Information) is the canonical cross-cutting abstraction over BCSI (CIP-011 / NERC CIP), PII, PHI, and other regulatory classes. `masking_profile` is a compliance-profile axis (configurable, tightens-only, BCSI profile first). OCSF `data_classification` wire format (v1.2.0+, ships in baseline) adopted as-is for RSI field tagging.
- **Per-field-class token-determinism matrix:** Tunable via compliance-profile masking axis. Controls whether tokens are deterministic (same input → same token within vault) vs non-deterministic (fresh token per occurrence). Deterministic required for cross-event entity correlation; non-deterministic reduces re-identification risk for single-event fields.
- **Per-tenant vault + DEK at edge; agent zero vault wiring:** Each tenant has an isolated RocksDB-CF vault and DEK at the edge satellite. Agent zero (the satellite management agent) holds the vault-init wire path; key custody follows C9 bootstrap-recovery 4-layer pattern + C16 DEK escrow (→ C18 key-escrow Option-3 CMEK for central metadata custody).
- **Detokenize-at-surface via C18 RBAC (transient, never re-persisted, audited):** The detokenization path is gated by the C18 ABAC tag-masking layer + CIP-004/007 audit trail. Detokenized values are transient in the query surface layer; never written to RocksDB or cold Iceberg tier. Every detokenization event is audited (INV-ADS-09).
- **Dual index (raw human-IR secure-zone vs masked AI/RAG):** Two logical indexes maintained: (a) raw-value index in the human-IR secure zone (detokenize-at-surface RBAC gate required for access), (b) tokenized/masked index for AI/RAG pipeline. Vectors (embeddings) are flagged as sensitive-data-class — inversion attacks can recover approximate raw values from embeddings; validates C12 on-box embedding decision (no external embedding API).
- **Deferred:** Embedding-perturbation defense (differential-privacy noise injection on embeddings before storage) — cost-gated open item (OQ-RSI-EMBED-PERTURB). HIPAA Expert Determination method — deferred to compliance-authority review.
- **ADS v1.3:** P-ADS-07 sharpened (clearing-house enforcement + embeddings-are-sensitive assertion); PAT-ADS-14 (Edge-Tokenizing-Clearing-House pattern) added; traceability row for C16.

#### C18 — RBAC depth (into connectors/satellites) — FULLY DECIDED + CAPTURED (ADR-PROP-rbac-depth.md + ADR-PROP-compliance-profiles.md, commit prior to this checkpoint)

- **Authorization model:** Layered RBAC+ReBAC+ABAC. Not a single model — all three layers coexist: role-based coarse access, relationship-based scoping (connector/satellite/source/table), attribute-based tag masking.
- **Engine decision:** BUILD Prism-native Rust Zanzibar-tuple core (SF-1). Fallback options evaluated and ordered: casbin-rs / opa-wasm / gatehouse / oso. SpiceDB (Go sidecar) and OpenFGA (Go sidecar) REJECTED — wrong runtime boundary, cross-language RPC latency in hot query path.
- **Scoping axes:** connector / satellite / source / table. ABAC tag masking at column-value level (not per-column roles — column-value masking at surface, not schema-level role proliferation).
- **Inheritance:** Strictly-downward only — inherits from parent nodes via C19 closure table. Escalation-UP prevention is a tested schema invariant (not just policy).
- **Audit:** Decision-level authorization audit (INV-ADS-09 — every authz decision logged with subject, resource, action, policy version, result). Not just access events — decision audit.
- **Policy distribution:** Central-authored / edge-enforced signed bundles. Deny-on-stale-beyond-N policy age threshold (configurable; default conservative). No edge-authored policy.
- **IdP integration:** IdP→internal-role mapping + SCIM 2.0 (SF-6) for tenant-configurable role mapping within central-defined bounds. External IdP roles map to Prism internal roles; Prism internal roles are the authorization spine.
- **PII bulk-export:** Hard-block on bulk PII export by default; configurable (authorized exemption + audit trail required). Not a soft warning — hard gate.
- **ADS bump:** v1.1→v1.2. New entries: PAT-ADS-12 (Configurable Compliance Profile), PAT-ADS-13 (Layered-Authz), INV-ADS-09 (Decision-Level Authorization Audit) + traceability rows.
- **Day-3 Workflow Engine epic:** E-WORKFLOW-ENGINE-001 (PROPOSED) created — consolidates C9 deferred approval/review workflows + C18 SF-5 (configurable approval gates) + C18 SF-2 (unmask-with-approval workflow) + C15 HITL gates. All four deferred workflow-class items now anchored to a single future epic.

#### Compliance-Profile mechanism — FULLY DECIDED + CAPTURED (ADR-PROP-compliance-profiles.md, this checkpoint)

- **Model:** Monotone tighten-only named/versioned/signed Compliance Profiles. A Profile can ONLY tighten constraints, never loosen them relative to baseline. Profiles form a partial order: `baseline ⊂ soc2 ⊂ iso27001 ⊂ iec-62443-ot ⊂ nerc-cip`. OT (IEC-62443) is a shipped Profile — NOT a code fork. No OT-specific branching in the codebase.
- **Format:** OSCAL param+constraint model. Each Profile contains: parameter overrides (tunables) and constraint locks (non-tunables). The distinction is machine-readable and enforced at deploy-time.
- **SF-PROF-1 — Two axes:** deployment-profile axis (SaaS / MSSP-managed / client-managed from ADR-PROP-dual-deployment.md) is DISTINCT from compliance-profile axis. Orthogonal dimensions; both apply simultaneously.
- **SF-PROF-2 — Exemptions:** compliance-authority-only exemptions on regulatory hard-locks. An operator cannot self-exempt from a regulatory hard-lock; only the compliance authority (Prism/1898) can grant an exemption, which must be time-boxed + audit-logged.
- **SF-PROF-3 — Conformance gate:** boolean `compliant | drifting` status + itemized drift report + audit-before-enforce mode (audit findings reported without blocking before the enforce deadline). No flat compliance percentage — itemized drift only.
- **SF-PROF-4 — Open custom profiles:** Organizations may author custom Profiles (must extend baseline; cannot relax it). Custom profiles reviewed + signed before activation.
- **C19 regulatory_class reframed:** `regulatory_class` field on tenant nodes is now a Compliance-Profile selector/floor (sets the minimum Profile that subtree nodes must satisfy). Behavior unchanged — field already existed in ADR-PROP-nested-tenancy.md — but the semantic is now Profile-scoped rather than ad hoc classification.
- **Research banked:** `research/configurable-security-profiles-2026-06-27.md` — Profile mechanism research committed in this burst.

#### C19 — Nested tenancy — FULLY DECIDED + CAPTURED (ADR-PROP-nested-tenancy.md, this checkpoint)
- **Tree model:** Adjacency list + closure table + materialized path — three complementary representations; adjacency = source-of-truth writes; closure = O(1) subtree queries; materialized-path = display breadcrumbs and prefix-routing.
- **SF-1 (inheritance):** Hybrid — policy + role explicit-override at any node; configurable propagation modes (inherit/override/isolate) per policy class; OQ-DEPLOY-1 CLOSED.
- **SF-2 (depth):** Unlimited tree depth + configurable soft-cap (default 8) + operator hard-override; no arbitrary depth ceiling in the data model.
- **SF-3 (parent-visibility):** Visibility-grant matrix; P3 (transparent-subtree to parent) gated to same-legal-entity + re-encrypt-on-grant + transient key custody (KEK never at rest on parent); parent-as-grantee for cross-org tenants forbidden (AP-ADS-11); regulatory_class field overrides visibility grant for CUI/BCSI.
- **SF-4 (encryption):** Configurable flat-DEK-or-nested-KEK; per-node DEK default; MSSP-managed = client-held CMEK by default; P-ADS-02 sharpened to distinguish unmediated-at-rest access from authorized-mediated access.
- **SF-5 (reparenting):** Audited reparenting (full audit trail, policy re-evaluation at move time, orphan-prevention guard); cross-legal-entity moves gated by regulatory_class check.
- **MSSP reconciliation:** P-ADS-02 sharpened; MSSP-managed operating model uses the bridge-node pattern (MSSP_ROOT → client subtrees); client-held CMEK is the default key custody; no cross-client data path by construction.
- **ADS conformance:** all 8 INV-ADS pass; AP-ADS-11 (Cross-Tenant DEK Grantee) added to ADS anti-patterns; Option-3 Central-Sole-Surface satisfied (tree stored at central; satellites receive subtree views only).

#### C17 — Backup & recovery (first-class) — FULLY DECIDED + CAPTURED (ADR-PROP-backup-recovery.md, this checkpoint)

- **Per-store backup mechanics:** pgBackRest PITR for PostgreSQL (control-plane DB); git bundle for detection-content git repo; RocksDB Checkpoint+BackupEngine for sensor state / token vault / config; Iceberg catalog+metadata+data for cold Iceberg tier; knowledge graph + vector store → KG authoritative (RocksDB-CF backed = standard RocksDB Checkpoint), vectors rebuildable from authoritative embeddings (full backup optional); sealed-blob keys in escrow; ARO delivery split (facts authoritative, derived AROs rebuildable).
- **Cross-store coherent PITR:** Logical-watermark + per-store time-travel. T = the C8 `AS OF KNOWN <T>` watermark + backup-set manifest ties the recovery point across stores. Selective physical freeze for tightly-coupled core stores only (RocksDB hot stores + PostgreSQL); Iceberg cold tier + git detection-content = already snapshot-consistent.
- **Key escrow:** Tenant-held recovery key default (tenant controls decryption of their own backup). Optional M-of-N threshold escrow tier for operator-assisted recovery under break-glass conditions. Crypto-shred: operator stores unwrappable blobs only — promise = no unilateral operator access. DEK escrow is consistent with SS-26 (C16) per-tenant DEK model and C18 deny-on-stale signed-bundle mechanics.
- **Per-tenant PITR:** Restore-to-side-instance (non-disruptive) + selective re-ingestion of sensor data after the recovery point (genuinely hard — marked as operationally complex, not deferred). Silo escape-hatch: tenant can export their data as a standard snapshot bundle if they migrate out. Nested parent/subtree/child scopes: each node in the C19 tenancy tree has an independent backup scope; subtree backups consistent within the subtree.
- **Satellite backup:** Reconstruct-from-central as the primary path (satellite state is derived; central holds the authoritative source). Local buffer backup for air-gap satellites (RocksDB Checkpoint to local storage). Air-gap signed bundles for offline satellites (same C9/C11 signed-bundle mechanism); satellite can restore local state without central connectivity.
- **DR tier ladder:** Full tier ladder per deployment-profile/contract — backup-restore (RPO hours, RTO hours) → pilot-light (warm schema, fast rehydrate) → warm-standby (replica, fast promote) → active-active (multi-region, near-zero RPO). Tier selected at deployment-profile + contract level; not all tiers required for all deployment-profiles.
- **Unified integrity:** Signed + customer-managed-key (CMK) encrypted everywhere. Backup artifacts signed with tenant key; integrity check before any restore. No unencrypted backup artifacts at rest.
- **Recovery-test evidence first-class NOW:** Timestamped automated restore runs + post-restore CIP-010 baseline diff are required evidence artifacts, not future enhancements. CIP-009 recovery evidence requirement drives this as a shipped day-2 capability.
- **RSAW export:** Deferred to C20 (NERC CIP synthesis — CIP-009 requires RSAW documentation; C20 will tie the evidence format to the CIP-009 standard).
- **ADS v1.4:** PAT-ADS-15 (Logical-Watermark Cross-Store Backup), PAT-ADS-16 (Sealed-Blob Key Escrow + Crypto-Shred), INV-ADS-10 (Recoverability-Preserves-Operator-Zero-Access) added.

#### C20 — NERC CIP support (synthesis capstone) — FULLY RESOLVED (ADR-PROP-nerc-cip-support.md; SF-2 = Defer + Leave-Seams-Open)

- **Posture:** CIP-deployable + CIP-evidence-generating. NOT pursuing NERC CIP certification as a product; Prism is the tool a covered entity uses to achieve and document compliance.
- **Synthesis map (prior C-items → CIP standards):**
  - C16 (RSI/BCSI clearing house) → CIP-011 (BES Cyber System Information protection)
  - C17 (backup & recovery + recovery-test evidence) → CIP-009 (Recovery Plans for BES Cyber Systems)
  - C18 (RBAC+ReBAC+ABAC + Compliance Profiles nerc-cip preset) → CIP-004 (Personnel & Training), CIP-005 (Electronic Security Perimeters), CIP-007 (Systems Security Management)
  - C19 (nested tenancy + regulatory_class Profile selector) → CIP-002 (BES Cyber System Categorization)
  - C2 (satellite mesh + IEC-62443 structural residency D-C2-12) → CIP-005 ESP (Electronic Security Perimeter) — satellite residency hard invariant naturally maps to ESP boundary enforcement
- **Regulatory anchor — CIP-004-7/CIP-011-3 (Jan 2024 provisioned-access + entity-key zero-access pivot):** The Jan-2024 amendments to CIP-004-7 (R4 provisioned-access controls) and CIP-011-3 (BCSI entity-key zero-access) BLESS the BYOC zero-access central plane as a compliant architecture. The central plane never holds raw BCSI; entity keys are held by the covered entity. This is a formal regulatory anchor, not just a design choice. Records as PIV-C20-001.
- **SF-1 DECIDED — Build first-class CIP audit-evidence/RSAW-export module:** Superset design: (a) substrate — structured compliance-evidence data model mapping CIP requirements to Prism observability outputs; (b) RSAW bundles — machine-generated, human-reviewable RSAW (Reliability Standard Audit Worksheet) export packs per standard (CIP-002, CIP-004, CIP-005, CIP-007, CIP-009, CIP-011); (c) GRC-consumable — structured export (JSON/CSV/OSCAL) consumable by third-party GRC platforms (Archer, ServiceNow GRC, etc.). Proposed epic: **E-CIP-EVIDENCE-EXPORT-001**.
- **SF-3 DECIDED — Lighter classification by default:** Passive read-only edge posture is the default. Write/control paths are feature-flagged (C15 autonomy ladder gating). No hidden control paths. The covered entity auditor sees no undisclosed remote-access or write capability in a CIP deployment.
- **SF-4 DECIDED — Both CIP-010 system-of-record AND CMDB feeder:** Prism can function as the authoritative CIP-010 BES Cyber System inventory (source of truth) OR as a CMDB feeder (exporting to an existing CMDB). Deployment-profile-configurable. Supports the full range of covered-entity maturity levels.
- **CIP-013 vendor commitments (pre-B capture):** SBOM generation (for supply-chain traceability), signed releases (for software integrity), no undisclosed remote access (AD-017 + BYOC zero-access invariant). These commitments are documentation + engineering discipline, not a separate certification.
- **nerc-cip Compliance Profile preset:** The `nerc-cip` Compliance Profile (in ADR-PROP-compliance-profiles.md partial order: `baseline ⊂ soc2 ⊂ iso27001 ⊂ iec-62443-ot ⊂ nerc-cip`) activates all relevant tightening: BCSI masking (C16), CIP-004/005/007 access controls (C18), CIP-009 recovery evidence (C17), CIP-002 asset categorization (C19 regulatory_class), CIP-005 ESP residency enforcement (C2 D-C2-12).
- **SF-2 RESOLVED = Defer + Leave-Seams-Open (D-C20-SF2, Sub-Option B):** Defer the speculative cloud-EACMS feature per P-ADS-12; keep three zero-cost seams open: S1 classification-as-data (OCSF `data_classification` remains a wire-format axis, not hardcoded CIP applicability), S2 open `#[non_exhaustive]` deployment-profile enum (future cloud-EACMS variant addable without breaking changes), S3 evidence-owner provenance dimension reserved in the compliance-evidence data model (entity/csp/shared — when the audit substrate is built). Zero-access invariant **PIV-C20-006** LOCKED regardless of how Project 2023-09 resolves. **PIV-C20-007** (seams S1/S2/S3 kept open). Sub-Option C (design-forward cloud-EACMS now) rejected as default — available only as explicit future market-timing decision. Live-re-verify Project-2023-09 status at morph. Research banked: `research/nerc-project-2023-09-cloud-bes-2026-06-27.md`. **C20 FULLY RESOLVED.**
- **ADS v1.6:** PAT-ADS-17 evidence-owner provenance dimension forward-note (entity/csp/shared) added; traceability row for C20 SF-2 closure.

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

**Dependency-aware order (human-confirmed): C13 ✅ → C12 ✅ → C11 ✅ → C15 ✅ → C14 ✅ → C19 ✅ → C18 ✅ → C16 ✅ → C17 ✅ → C20 ✅ (fully resolved incl. SF-2 fold) → B**

**Cross-cutting conformance frame:** ALL remaining items (C15, C14, C19, C18, C16, C17, C20, B) must pass the ADS conformance checklist (`ARCHITECTURE-DESIGN-SYSTEM.md`) before capture. The first conformance pass (10 ripple-audit items across 9 ADR-PROPs) is done.

| ID | Area | Status | Cross-links |
|---|---|---|---|
| C13 | §16.4 open-items closeout (SSO, S3-runtime, secret/DEK, sequence-sugar, PO-ratif, ML-phasing) | ✅ DONE (commit 09c5584d) — 8 residuals resolved; per-credential-DEK = future enhancement (OQ-SECRET-DEK-GRANULARITY); ml OD-3 reconciled to C7 | C7, C2/AD-017, C16, C20 |
| **ADS** | Architecture Design System — "The Prism Way" (13 Principles, 17 Patterns, 10 Invariants, conformance checklist) | ✅ DONE (commit 7c068714; amended v1.1 + v1.2 + v1.3 + v1.4 + v1.5 + v1.6 + v1.7) — `ARCHITECTURE-DESIGN-SYSTEM.md` v1.7 is the canonical conformance frame for all remaining work | All C-items + B |
| **Option-3** | Surfacing model locked — Tenant-Keyed-Central-Cache always; derived-results-only; operator-zero-access-at-rest; Central-Sole-Surface principle | ✅ LOCKED (commit 7c068714) — do not re-litigate | ADS INV-ADS, C12, C11, C15 |
| **Conformance pass** | Ripple audit of existing ADR-PROPs against ADS + Option-3 | ✅ DONE (commit 7c068714) — 10 items closed across 9 ADR-PROPs | All prior C-items |
| C12 | Prism Context — knowledge graph + vector DB + entity mapping + Entity 360 expansion | ✅ DONE (commit 76f1a3e2) — ADR-PROP-prism-context.md; two-layer embedded (indradb+usearch+lancedb); fastembed/ort+candle; deterministic entity-resolution + suspected-links; hybrid retrieval + mandatory citations; phased GraphRAG; aletheon corrected | aletheon `aros` table → C15 input, AD-017, C16, C7 |
| C11 | Prism Intel — hosted threat-intel/auto-advisory tied to Entity 360 | ✅ DONE (commit 7c068714) — ADR-PROP-prism-intel.md; feed-down/match-at-edge; opt-in central-match for non-BYOC SaaS; deployment-conditional metering; free public + paid curated tiers; C9 signed-bundle for air-gap; PSI rejected | C12 Entity 360, C9 signed-bundle, BYOC zero-access, Option-3 |
| C15 | Actions in PrismQL / SOAR + on-prem models (Action·Recommendation·Observation) | ✅ DONE (commit b6314532 — `ADR-PROP-soar-actions-aro.md`) — recommend-only v1 + autonomy ladder designed/enable-post-v1; three typed entities (Observation/Recommendation/Action) over common base; Recommendations from BOTH S3 agent layer AND read-only PrismQL `RECOMMEND` projection (perimeter-compile-fail-tested, source-discriminated provenance); full rec-rigor day one (W3C-PROV + calibrated confidence + conformal sets + mandatory post-hoc citation-faithfulness); separate `prism-orchestration` Action layer (HITL gates, idempotency, dry-run, rollback, AD-017 reference-based write-creds); on-prem models (Qwen3/Mistral central, Phi-4-mini/Ministral edge, Llama Prompt Guard guardrails) via C7 ModelBackend; ARO linkage rides C12 graph; ADS-conformant (all 8 INV pass) | C7 ModelBackend, S3 agent, C10 GAP-Q2, C18, AD-017, aletheon aros |
| C14 | Active-query device support (Industrial Defender class) | ✅ DONE (commit 59864881 — `ADR-PROP-active-query-devices.md`) — Reading A+B both in v1 (federate OT-platform APIs + direct OT-protocol field-device polling, poller-of-last-resort); active-query = capability-axis on C3/C4 (not a new connector class); read-only-perimeter (writes via C15); OT-safety guardrails as hard invariants; OT asset/config/vuln modeled as OCSF source tables; Reading-B protocol libs as plugins/sidecar; ADS-conformant | C3/C4 connectors, C15 (write perimeter), C20 (OT/ICS) |
| C19 | Nested tenancy (parent→child→… unlimited) | ✅ DONE (ADR-PROP-nested-tenancy.md) — bridge + per-node isolation_tier closes OQ-DEPLOY-1; adjacency+closure+materialized-path tree; SF-1 hybrid / SF-2 unlimited+soft-cap-8 / SF-3 visibility-grant matrix (P3 transparent-subtree gated same-legal-entity; (c) forbidden→AP-ADS-11; regulatory_class override) / SF-4 configurable flat-DEK-or-nested-KEK / SF-5 audited reparenting; MSSP reconciliation sharpened P-ADS-02; MSSP key custody default = client-held CMEK | CLOSES OQ-DEPLOY-1; C18 (role inheritance), C16 (detokenize-at-surface) |
| C18 | RBAC depth (into connectors/satellites) | ✅ DONE (ADR-PROP-rbac-depth.md + ADR-PROP-compliance-profiles.md, this checkpoint) — layered RBAC+ReBAC+ABAC; BUILD Prism-native Rust Zanzibar-tuple core; connector/satellite/source/table scoping + ABAC tag masking; strictly-downward inheritance via C19 closure table; decision-level audit (INV-ADS-09); central-authored/edge-enforced signed bundles; IdP→internal-role + SCIM; PII bulk-export hard-block-but-configurable; Compliance-Profile mechanism captured in companion ADR-PROP | CLOSES C10 Query-RBAC gap; extends C9 / E-CENTRAL-AUTHZ-001; C19 (role inheritance), C16 (detokenize-at-surface), C15 (approver roles), C17 (key-escrow Option-3 CMEK) |
| C16 | Entity masking / tokenizing clearing house (AI-opaque data; BCSI→RSI universal name) | ✅ DONE (ADR-PROP-entity-masking.md) — extends AD-017 to AI-opaque DATA; BUILD Prism-native Rust clearing house (SS-26 DEK + aes-gcm + RocksDB-CF vault, FF1 FPE optional); technique mix keyed by RSI field class; EDGE placement after OCSF normalization (forced INV-ADS-01/Option-3); RSI abstraction + pluggable profiles (BCSI first) over OCSF data_classification; per-field-class token-determinism matrix; per-tenant vault+DEK at edge; detokenize-at-surface via C18 RBAC; dual index; vectors are sensitive-data-class; ADS v1.3 (P-ADS-07 sharpened, PAT-ADS-14) | extends AD-017; C20 confirms BCSI canonical; C18 (detokenize-at-surface via RBAC); C17 (key-escrow/DEK custody) |
| C17 | Backup & recovery (first-class) | ✅ DONE (ADR-PROP-backup-recovery.md, this checkpoint) — per-store mechanics (pgBackRest/git-bundle/RocksDB-Checkpoint+BackupEngine/Iceberg); cross-store PITR via logical-watermark+C8-AS-OF-KNOWN-T; tenant-held recovery key + M-of-N escrow + crypto-shred (no unilateral operator access); per-tenant PITR + silo escape-hatch + nested scopes; satellite reconstruct-from-central + air-gap signed bundles; DR tier ladder per deployment-profile; unified integrity (signed + CMK-encrypted); recovery-test evidence first-class (CIP-009); RSAW export → C20; ADS v1.4 (PAT-ADS-15/16, INV-ADS-10) | C9 config + bootstrap-recovery; C20 CIP-009; C18 (key-escrow Option-3 CMEK) |
| C20 | NERC CIP support | ✅ DONE (fully resolved — ADR-PROP-nerc-cip-support.md; SF-2 = Defer + Leave-Seams-Open; D-C20-SF2; PIV-C20-006 LOCKED; PIV-C20-007 seams open; ADS v1.6) | C16(BCSI/RSI), C17, C18, C19, C2, OT |
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

**C20 ✅ FULLY RESOLVED (SF-2 folded = Defer + Leave-Seams-Open; D-C20-SF2; PIV-C20-007 seams open; ADS v1.6). C17 ✅ C16 ✅ C18 ✅ C19 ✅ C14 ✅ C15 ✅ C12 ✅ C11 ✅ done. ADS created (v1.6), Option-3 locked, conformance pass done. Pre-B feature track C11–C20 COMPLETE.**

**NEXT: B — integration capstone. GATED on §5.1 brief-reframe HUMAN sign-off (still PENDING; two positioning candidates are the inputs: D-C10-5 competitor-relative in ADR-PROP-competitive-positioning.md + new problem-framed candidate in ADR-PROP-positioning-problem-framed.md — do not begin B until the human confirms the brief-reframe direction).**

**Positioning fold-pass status (out-of-band, gated on §5.1):**
- Fold-pass A (exec narrative — positioning-executive-narrative.md): ✅ DONE (this burst)
- Fold-pass B (problem-framed candidate — ADR-PROP-positioning-problem-framed.md): ⏳ PENDING §5.1 gate
- Fold-pass C (diagrams — prism-architecture-conceptual.drawio / prism-architecture-technical.drawio): ⏳ PENDING §5.1 gate

**C14 follow-up research BANKED (committed alongside earlier checkpoint):**
- `research/ocsf-ot-coverage-2026-06-27.md` — C14 sub-fork F4: OCSF-OT schema coverage. COMPLETE. Committed in same burst as this resume-doc edit (path-scoped).
  **Findings (closes OQ-C14-OCSF):** OCSF v1.8.0 (2026-03-18). Asset identity (Device Inventory Info 5001), config baseline/drift/exception (Device Config State 5002/5019 + Compliance Finding 2003), and device-vuln (Vulnerability Finding 2002) all FIT CLEANLY into the C14 OCSF source-table model. OT semantics (PLC/RTU/HMI type, firmware, Purdue zone, control topology) NEED a private `prism_ot` extension — the Device type_id enum is IT-only in OCSF core. CPE-matching is a gap (flag — ties to C11 intel enrichment). Data Classification profile SHIPS (v1.5.0+) — adopt as-is for C16/RSI tagging. Upstream ICS work (OCSF Issue #1515, Corelight) is STALLED — **lean = author private `prism_ot` extension now (NOT upstream-first), reserving clean UIDs for future upstream contribution.** 5 sub-forks (SF-1..5) recorded in the research file.

**Research status for remaining items:**
- C19: ✅ DONE — `research/nested-tenancy-2026-06-27.md`
- C18: ✅ DONE — `research/rbac-depth-2026-06-27.md` + `research/configurable-security-profiles-2026-06-27.md`
- C16: ✅ DONE — `research/entity-masking-tokenization-2026-06-27.md` + ADR-PROP-entity-masking.md
- C17: research BANKED — `research/backup-recovery-2026-06-27.md`
- C20: research BANKED — `research/nerc-cip-support-2026-06-27.md` (scheduled LAST)

**Ordered queue — work through left-to-right, do not skip:**
1. **B — integration capstone** (GATED on §5.1 brief-reframe HUMAN sign-off — do not begin until human confirms)

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
- ~~C19 discussion + capture → ADR-PROP-nested-tenancy.md~~ ✅ 547319b2
- ~~C18 discussion + capture → ADR-PROP-rbac-depth.md + ADR-PROP-compliance-profiles.md~~ ✅ b8f59257
- ~~C16 discussion + capture → ADR-PROP-entity-masking.md~~ ✅ ee3a9e21
- ~~C17 discussion + capture → ADR-PROP-backup-recovery.md~~ ✅ (prior checkpoint)
- ~~C20 bulk discussion + capture → ADR-PROP-nerc-cip-support.md~~ ✅ (prior checkpoint)
- ~~C20 SF-2 fold → ADR-PROP-nerc-cip-support.md (Defer + Leave-Seams-Open; D-C20-SF2; PIV-C20-007; ADS v1.6)~~ ✅ (this checkpoint) — C20 FULLY RESOLVED; pre-B track C11–C20 COMPLETE

## 6. BASELINE (git state at session wrap)

- **factory-artifacts HEAD (current):** `(this checkpoint)` — prism-as-ot-sensor-note.md working note captured (PCAP+dissector passive-OT finding + OT-standalone correction; 5 iterate-later threads); path-scoped commit
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
  - `547319b2` — C19 nested-tenancy decided + ADR-PROP-nested-tenancy.md captured + SF-3 parent-visibility research banked + ADS amended (P-ADS-02 sharpened, AP-ADS-11 added); path-scoped commit
  - `(prior checkpoint)` — P-ADS-13 Configurable-Not-Prescriptive added to ARCHITECTURE-DESIGN-SYSTEM.md (configurability principle); ADS bumped v1.0→v1.1 amendment log; path-scoped commit. Prior side HEAD: **5c7fc02b**.
  - `b8f59257` — C18 RBAC depth + Compliance Profiles decided + ADR-PROP-rbac-depth.md & ADR-PROP-compliance-profiles.md captured; ADS v1.2 (PAT-ADS-12/13, INV-ADS-09); C19 regulatory_class reframed as profile-selector; Compliance-profile research banked (configurable-security-profiles-2026-06-27.md); path-scoped commit
  - `ee3a9e21` — C16 entity masking / RSI clearing house decided + ADR-PROP-entity-masking.md captured; ADS v1.3 (P-ADS-07 sharpened, PAT-ADS-14, C16 traceability row); SESSION-RESUME updated (C16 DONE, next=C17); path-scoped commit
  - `(prior checkpoint)` — C17 backup & recovery decided + ADR-PROP-backup-recovery.md captured; ADS v1.4 (PAT-ADS-15/16, INV-ADS-10); SESSION-RESUME updated (C17 DONE, next=C20); path-scoped commit. Prior side HEAD: **a261cc1d**.
  - `(prior checkpoint)` — C20 NERC CIP support bulk decided + ADR-PROP-nerc-cip-support.md captured (SF-1/3/4; SF-2 OPEN pending research); ADS v1.5 (PAT-ADS-17 Compliance-Evidence-Export RSAW-aligned); SESSION-RESUME updated (C20 bulk DONE, SF-2 fold + B next); path-scoped commit. Prior side HEAD: **a220d879**.
  - `10875181` — C20 SF-2 folded (Defer + Leave-Seams-Open); cloud-BES research banked; ADS v1.6 (PAT-ADS-17 evidence-owner forward-note); C20 fully resolved; pre-B track C11–C20 COMPLETE; path-scoped commit
  - `(prior checkpoint)` — fresh-context consistency audit (MINOR-DRIFT: 0 blocker/0 major; 3 minor + 5 obs all fixed); ADS v1.7 (traceability rows C10/C14/C15 + freshness fixes); audit report banked; path-scoped commit. Prior side HEAD: **10875181**.
  - `(this checkpoint)` — customer-problem-coverage-2026-06-28.md removed per explicit human direction (history preserved at a8caac9e); path-scoped commit. Prior side HEAD: **929a9758**.
  - `(this checkpoint)` — problem-framed positioning candidate captured (ADR-PROP-positioning-problem-framed.md; mandated-7; CONDITIONAL-PASS adversarial fitness; 5 caveats baked in); path-scoped commit.
  - `(this checkpoint)` — §8 feature-map appendix + coverage matrix added to ADR-PROP-positioning-problem-framed.md; path-scoped commit. Prior side HEAD: **6c2bf616**.
  - `(this checkpoint)` — plain-language executive positioning narrative captured (positioning-executive-narrative.md); path-scoped commit. Prior side HEAD: **86a2a334**.
  - `(this checkpoint)` — prism-as-ot-sensor-note.md working note captured (PCAP+dissector passive-OT finding + OT-standalone correction; 5 iterate-later threads); path-scoped commit. Prior side HEAD: **55963381**.
  - `(this checkpoint)` — Prism+Satellite C4 diagram set (conceptual + technical, .drawio + PNG/SVG/PDF) captured under diagrams/; path-scoped commit. Prior side HEAD: **82594cf1**.
  - `(this checkpoint)` — positioning-fidelity-iterate-list.md captured (36-finding master iterate-list; multi-agent corpus→positioning fidelity sweep); path-scoped commit. Prior side HEAD: **9a7434d9**.
  - `(this checkpoint)` — fold-pass A: exec-narrative audit folds applied (OVER-01/02/03 corrected; decided differentiators given plain-language homes; 'Where We Are Today' re-tiered); path-scoped commit. Prior side HEAD: **5951b7ce**.
  - `(this checkpoint)` — fold-pass B: problem-framed candidate audit folds (22 refs); path-scoped commit.
  - `(this checkpoint)` — fold-pass C+D: diagram + competitive-positioning audit folds; path-scoped commit. Prior side HEAD: **d672772f**.
- Note: live-factory commits interleaved on `factory-artifacts` between the above (normal concurrent operation). Prior side HEAD before this burst: **929a9758** (consistency audit complete; ADS v1.7).
- Working tree otherwise clean (untracked `.DS_Store` only; live-factory BC-2.16.002 left unstaged). No dangling in-flight research.

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

**New C18 proposed epics (PROPOSED):**
- E-CENTRAL-AUTHZ-001 — Central authorization engine: Prism-native Rust Zanzibar-tuple core; layered RBAC+ReBAC+ABAC; connector/satellite/source/table scoping axes; strictly-downward role inheritance via C19 closure table; ABAC tag masking at column-value surface; decision-level audit (INV-ADS-09); IdP→internal-role mapping + SCIM 2.0; PII bulk-export hard-block-but-configurable; central-authored/edge-enforced signed policy bundles; deny-on-stale-beyond-N threshold
- E-COMPLIANCE-PROFILES-001 (PROPOSED) — Compliance Profile mechanism: monotone tighten-only named/versioned/signed Profiles (baseline⊂soc2⊂iso27001⊂iec-62443-ot⊂nerc-cip); OT as a Profile not a code fork; OSCAL param+constraint model; SF-PROF-3 boolean compliant|drifting + itemized drift + audit-before-enforce mode; SF-PROF-4 open custom-profile authorship; compliance-authority-only exemptions on regulatory hard-locks; C19 regulatory_class reframed as Profile selector/floor

**New C16 proposed epics (PROPOSED):**
- E-RSI-CLEARING-HOUSE-001 (PROPOSED) — RSI Tokenizing Clearing House: Prism-native Rust edge clearing house (SS-26); per-tenant DEK + aes-gcm + RocksDB-CF token vault; technique mix keyed by RSI field class (deterministic vaulted tokenization / FF1 FPE narrow-domain≥10^6 / redaction / NER free-text); EDGE placement after OCSF normalization; RSI abstraction + pluggable masking profiles (BCSI first) over OCSF data_classification wire format; per-field-class token-determinism matrix tunable via compliance-profile masking axis; per-tenant vault+DEK at edge, agent zero vault wiring; detokenize-at-surface via C18 RBAC (transient, never re-persisted, audited CIP-004/007 + INV-ADS-09); dual index (raw human-IR secure-zone vs masked AI/RAG); vectors flagged as sensitive-data-class (inversion attack risk); C12 on-box embedding validated by this decision; OQ-RSI-EMBED-PERTURB + HIPAA Expert-Determination deferred

**New C17 proposed epics (PROPOSED):**
- E-BACKUP-RECOVERY-001 (PROPOSED) — First-class backup & recovery: per-store mechanics (pgBackRest PITR / git bundle / RocksDB Checkpoint+BackupEngine / Iceberg catalog+metadata+data / sealed-blob key escrow); cross-store coherent PITR via logical-watermark (C8 AS OF KNOWN T) + backup-set manifest; tenant-held recovery key default + optional M-of-N threshold escrow tier + crypto-shred (no unilateral operator access); per-tenant PITR = restore-to-side-instance + selective re-ingestion + silo escape-hatch + nested parent/subtree/child scopes; satellite = reconstruct-from-central + local-buffer backup + air-gap signed bundles; DR tier ladder per deployment-profile/contract (backup-restore → pilot-light → warm-standby → active-active); unified integrity (signed + CMK-encrypted everywhere); recovery-test evidence first-class (timestamped restore runs + post-restore CIP-010 baseline diff); RSAW export deferred to C20 (CIP-009). ADS v1.4 (PAT-ADS-15/16, INV-ADS-10).

**New C20 proposed epics (PROPOSED):**
- E-CIP-EVIDENCE-EXPORT-001 (PROPOSED) — CIP audit-evidence / RSAW-export module: structured compliance-evidence data model mapping CIP requirements (CIP-002, CIP-004, CIP-005, CIP-007, CIP-009, CIP-011) to Prism observability outputs; machine-generated RSAW (Reliability Standard Audit Worksheet) bundle export per standard; GRC-consumable structured export (JSON/CSV/OSCAL) for third-party GRC platforms (Archer, ServiceNow GRC, etc.); Compliance Profile nerc-cip preset activates full tightening stack (C16 BCSI masking, C18 CIP-004/005/007 access controls, C17 CIP-009 recovery evidence, C19 CIP-002 categorization, C2 CIP-005 ESP residency); CIP-013 vendor commitments (SBOM, signed releases, no undisclosed remote access); SF-3 lighter classification default (write/control feature-flagged); SF-4 both CIP-010 system-of-record AND CMDB feeder; zero-access invariant PIV-C20-006 LOCKED; SF-2 (cloud-BES Project-2023-09 scope) deferred to fold burst

**New Day-3 proposed epics (PROPOSED):**
- E-WORKFLOW-ENGINE-001 (PROPOSED, Day-3) — Workflow Engine: configurable approval/review workflow runtime consolidating C9 deferred approval-gate workflows + C18 SF-5 (policy-change approval gates) + C18 SF-2 (unmask-with-approval workflow) + C15 HITL gates; tenant-configurable within central-defined workflow schemas; per-client workflow customization; audit trail on every workflow transition

**ADS conformance frame:** `ARCHITECTURE-DESIGN-SYSTEM.md` (v1.6) is the canonical conformance reference for all epics above and remaining work. Every epic capture must satisfy the ADS conformance checklist (13 Principles, 17 Patterns, 10 Invariants, no anti-patterns) and the Central-Sole-Surface (Option-3) invariant.

All deferred to morph (brief-reframe §5.1 HUMAN GATE).
