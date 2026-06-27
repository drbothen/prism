---
document_type: session-resume
status: capture
do_not_execute: true
timestamp: "2026-06-27"
provenance: "Day-2 vision SIDE-ANALYSIS program resume snapshot. SEPARATE from the live factory pipeline. Does NOT modify .factory/STATE.md or SESSION-HANDOFF.md (those belong to the live factory track)."
---

# Day-2 Vision Side-Analysis — Zero-Context Resume Snapshot (2026-06-27)

> **Original C-program (C1–C10 + deployment-matrix + C3 hard-reject reconciliation) COMPLETE and committed.
> A NEW pre-B feature track C11–C20 is now IN PROGRESS. B remains LAST and gated on the §5.1
> brief-reframe HUMAN sign-off.**
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
  (the live factory pushes concurrently); never `--no-verify`/`--force`. Last side HEAD: **09c5584d**
  (C13 §16.4 open-items closeout + 4× C11–C15/C20 banked research files committed).
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

A new batch of feature areas was confirmed BEFORE B — all require research + discussion + decisions + capture. Research is banked for C12, C11, C15, C20. B remains the final capstone, gated on §5.1 brief-reframe HUMAN sign-off.

**Dependency-aware order (human-confirmed): C13 ✅ → C12 → C11 → C15 → C14 → C19 → C18 → C16 → C17 → C20 → B**

| ID | Area | Status | Cross-links |
|---|---|---|---|
| C13 | §16.4 open-items closeout (SSO, S3-runtime, secret/DEK, sequence-sugar, PO-ratif, ML-phasing) | ✅ DONE (commit 09c5584d) — 8 residuals resolved; per-credential-DEK = future enhancement (OQ-SECRET-DEK-GRANULARITY); ml OD-3 reconciled to C7 | C7, C2/AD-017, C16, C20 |
| C12 | Prism Context — knowledge graph + vector DB + entity mapping + Entity 360 expansion | 🔬 research BANKED (research/prism-context-kg-vector-2026-06-27.md) — DISCUSS NEXT | aletheon (../aletheon_2, thin BMAD scaffold), AD-017, C16, C7 |
| C11 | Prism Intel — hosted threat-intel/auto-advisory tied to Entity 360 | 🔬 research BANKED (research/prism-intel-threat-advisory-2026-06-27.md) | C12 Entity 360, C9 signed-bundle, BYOC zero-access |
| C15 | Actions in PrismQL / SOAR + on-prem models (Action·Recommendation·Observation) | 🔬 research BANKED (research/prismql-actions-soar-onprem-models-2026-06-27.md) | C7 ModelBackend, S3 agent, C10 GAP-Q2, C18, AD-017 |
| C14 | Active-query device support (Industrial Defender class) | ⏳ pending (no research yet) | C3/C4 connectors |
| C19 | Nested tenancy (parent→child→… unlimited) | ⏳ pending | CLOSES OQ-DEPLOY-1 (deployment-matrix tenancy-isolation depth) |
| C18 | RBAC depth (into connectors/satellites) | ⏳ pending | CLOSES C10 Query-RBAC gap; extends C9 / E-CENTRAL-AUTHZ-001 |
| C16 | Entity masking / tokenizing clearing house (AI-opaque data; BCSI→universal name) | ⏳ pending | extends AD-017; C20 confirms BCSI canonical + recommends "RSI" abstraction |
| C17 | Backup & recovery (first-class) | ⏳ pending | C9 config + bootstrap-recovery; C20 CIP-009 |
| C20 | NERC CIP support | 🔬 research BANKED (research/nerc-cip-support-2026-06-27.md) — SCHEDULED LAST (synthesizes C16/C17/C18/C19/C2/OT) | C16(BCSI/RSI), C17, C18, C19, C2, OT |
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

**The original C-program (C1–C10 + deployment-matrix + C3 reconciliation) is COMPLETE. The pre-B feature
track C11–C20 is now IN PROGRESS. C13 is done; C12 research is banked.**

**Ordered queue — work through left-to-right, do not skip:**
1. **C12 discussion** (research banked at `research/prism-context-kg-vector-2026-06-27.md`) — NEXT
2. C11 discussion (research banked at `research/prism-intel-threat-advisory-2026-06-27.md`)
3. C15 discussion (research banked at `research/prismql-actions-soar-onprem-models-2026-06-27.md`)
4. C14 — Active-query device support (research pass needed first)
5. C19 — Nested tenancy (research pass needed first)
6. C18 — RBAC depth (research pass needed first)
7. C16 — Entity masking / tokenizing clearing house (research pass needed first)
8. C17 — Backup & recovery (research pass needed first)
9. C20 — NERC CIP (research banked at `research/nerc-cip-support-2026-06-27.md`; scheduled LAST — synthesizes all prior)
10. **B — integration capstone** (GATED on §5.1 brief-reframe HUMAN sign-off)

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

## 6. BASELINE (git state at session wrap)

- **factory-artifacts HEAD (session end):** `09c5584d` — C13 §16.4 open-items closeout (8 residuals resolved)
- **Full commit chain for this day-2 side-analysis** (day-2 side commits; live factory interleaved its own commits on the shared branch between these):
  - `b53b22ba` — C9 schema-versioning (Q3) resolved + ADR-PROP-config-management + backing research
  - `91abba90` — deployment matrix captured (ADR-PROP-dual-deployment, three operating models)
  - `613d4ff3` — C7 fold done (D-C7-1 resolved, ADR-PROP-ml-behavior-analytics-depth updated)
  - `b6fa1465` — C8 fold done (bitemporality, ADR-PROP-prismql-deliverables updated)
  - `0c9ce71e` — C10 competitive refresh decided + captured (ADR-PROP-competitive-positioning, all 8 gaps)
  - `f83e3ec7` — C3 reconciliation sweep (§12.2 hard-reject language → D-C3-1; label fix in C4 ADR-PROP)
  - `09c5584d` — C13 §16.4 open-items closeout + 4× C11–C15/C20 banked research files committed
- Note: live-factory commits interleaved on `factory-artifacts` between the above (normal concurrent operation).
- Working tree otherwise clean (untracked `.DS_Store` only; live-factory files left unstaged).

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

**New C11–C15 proposed epics (research banked; decisions pending discussion):**
- E-PRISM-CONTEXT-001 — Prism Context: knowledge graph + vector DB + entity mapping (C12); includes Entity 360 expansion, OT/asset-layer ingestion, OCSF-layer graph construction, air-gap milestone
- E-PRISM-INTEL-001 — Prism Intel: hosted threat-intel enrichment + auto-advisory engine tied to Entity 360 (C11); BYOC zero-access constraint; signed bundle delivery
- E-SOAR-ACTIONS-001 — Actions in PrismQL: Action/Recommendation/Observation surface + SOAR connector egress (C15); human-gate on destructive actions; AD-017 AI-opaque constraint
- E-ONPREM-MODELS-001 — On-premises model execution: ModelBackend local-deployment profile, air-gap inference, model provenance / signing (C15 on-prem dimension; extends C7 pluggable ModelBackend)

All deferred to morph (brief-reframe §5.1 HUMAN GATE).
