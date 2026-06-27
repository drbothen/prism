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
  (the live factory pushes concurrently); never `--no-verify`/`--force`. Last side HEAD: **this wrap commit**
  (run `git -C .factory log -1 --format='%h %s'` to get the live SHA).
- Working rhythm per area: research-agent pass (background) → I synthesize + give leans → AskUserQuestion on the
  genuine forks → architect captures decisions into the doc/ADR-PROPs → state-manager path-scoped commit.
  Pipeline research ~2–3 areas ahead. Confirm decisions before writing big new sections (mirror→confirm→write).
- Human pattern to carry: often defers a fork to a targeted research pass before deciding; consistently chooses
  the fuller/production-grade + audit-grade option.

## 1. Where everything lives

### Committed artifacts
- **`specs/matured-vision-day2-requirements.md`** — master capture (§1–§17). §16 = prior resume notes;
  §16.4 = running open-items/decisions log; §17 = federated ingestion (collector class, #4 pcap, #5 continuous
  operator, locus, chain-aware tiering/replication/deadline §17.8, dissectors §17.12, OT §17.13, reshape §17.14,
  A-leans §17.15).
- **`specs/day2-design-decisions/`** — PROPOSED ADRs + sketches (do_not_execute; real ADR #s deferred to morph):
  ADR-PROP-web-stack, -sso-identity, -s3-agent-runtime, -widget-dsl-render-and-schema-validation,
  -sandboxed-expression-evaluator, -storage-engine-taxonomy, -central-deployment-access-layer,
  -satellite-mesh, -capability-descriptor-pushdown, -dynamic-schema-connectors, -siem-lake-federation,
  -detection-engine-depth, -ml-behavior-analytics-depth, -prismql-deliverables;
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

Dated 2026-06-27 (landed this wrap commit):
- `edge-ml-mergeability-depth-2026-06-27.md` — C7 fold research (mergeable-exact representations)
- `prismql-asof-version-resolution-2026-06-27.md` — C8 fold research (bitemporality)
- `queryio-competitive-refresh-2026-06-27.md` — C10 discussion research
- `config-authority-narrow-git-2026-06-27.md` — C9 narrow-git authority research
- `git-as-primary-vs-write-behind-2026-06-27.md` — C9 git-as-primary research
- `bootstrap-config-recovery-2026-06-27.md` — C9 bootstrap recovery research
- `dual-deployment-saas-onprem-2026-06-27.md` — dual-deployment research

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
- **C8 PrismQL deliverables (ADR-PROP-prismql-deliverables.md):** piped surface SHIPS day-2 (KQL/PRQL-style→
  same DataFusion plan; expose desugared SQL/EXPLAIN); FIND keyword; SQL/PGQ GRAPH_TABLE multi-hop forward-compat;
  single LSP server (Chumsky Rich+ariadne/Monaco) for console+CLI+NL-agent; native.<source>.<field> namespace
  +retain-originals; Sigma-aligned recipe format+CI fixtures. DEFERRED: OQ-C8-ASOF + OQ-C8-OCSFVER
  (pending C8 fold via bitemporality research).

### Decided this session, not yet captured in ADR-PROPs

#### C9 config-management (PARTIAL — Q1 + fast-revert + bootstrap-recovery LOCKED; Q2 in progress; Q3 not started)

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

**Q2 CANARY MECHANICS — IN PROGRESS, NOT DECIDED:**
- Settled-going-in: canary in day-2; rollback action=fast-revert; reuses C6 circuit-breaker + shared
  change-detector primitive.
- OPEN — three points, the crux is #2 (human's next answer needed):
  1. Cohort unit: LEAN = cohort-by-config-scope (tenant for tenant-scoped, satellite/site for fleet-distributed).
  2. **THE CRUX: health signal posture** — hard-failure-only vs include-soft-regressions (coverage-drop /
     error-rate-uptick). My lean: include soft regressions with conservative threshold.
  3. Two-tier blast class: high-blast (connector defs, pushdown, retention, satellite trust, detection rules) =
     canary; low-blast (feature flags, log-level, TTLs) = direct-apply + fast-revert.

**Q3 SCHEMA-VERSIONING — NOT STARTED:**
- Now LOAD-BEARING due to dual-deployment (on-prem skip-version upgrades need forward migration).
- Fork: add schema_version + forward-migration framework (Envoy/K8s apiVersion style) vs
  `#[non_exhaustive]`-forward-compat-only.

#### Dual-deployment (DECIDED — needs its own cross-cutting ADR-PROP)
- Single-codebase + run-time deployment-profile (SaaS / on-prem-self-managed), ~90% shared.
  (Confirmed across GitLab/Sentry/Elastic/Mattermost/GitHub/Grafana/HashiCorp/Temporal; divergent forks =
  documented failure mode.)
- **HEADLINE:** Prism's satellite mesh IS the BYOC zero-access data-plane by construction (C2 residency +
  AD-017 satellite-local creds → SaaS central never sees raw data/creds). Passes BYOC litmus by construction.
  = Strongest SaaS differentiator.
- Egress-blocked CI invariant = standing guard (air-gap-leak is the most dangerous risk).
- On-prem ≠ single-tenant: MSSP-internal multi-tenancy via SAME tenant-id abstraction; tenants = MSSP's clients.
- Release = largest delta: on-prem customer-controlled skip-version upgrades → makes C9 Q3 schema-versioning
  load-bearing.
- C9 deployment-conditional:
  - SaaS: managed-remote-git; k8s blue-green (central); SaaS-only fleet-canary layer atop shared customer-scoped canary.
  - On-prem: offline-signed-bundle; A/B-appliance+watchdog (central); identical satellite self-recovery both modes.
- Touched decisions to mark deployment-aware: C1/storage/C2/credentials/C9/release.
- OPEN sub-choice: tenancy-isolation depth (pool/bridge/silo/cell-per-customer).
- Residual BYOC gaps: result-transit residency, metadata-leakage audit, ephemeral dial-home tokens,
  CMEK for central metadata.

## 3. THE C-PROGRAM (the active plan — "do each remaining day-2 area")
Each area: research → discuss → decide → capture → commit. **B = integration capstone LAST.**

| Item | Status | Notes |
|---|---|---|
| A — ingestion open sub-threads | ✅ decided + captured (§17.15) | |
| C1 — central deployment / access layer | ✅ decided + captured | |
| Storage taxonomy | ✅ decided + captured | |
| C2 — satellite mesh | ✅ decided + captured (ADR-PROP-satellite-mesh.md) | |
| C3 — capability-descriptor + PrismQL pushdown | ✅ decided + captured (ADR-PROP-capability-descriptor-pushdown.md) | |
| C4 — dynamic-schema / configure-schema connectors | ✅ decided + captured (ADR-PROP-dynamic-schema-connectors.md) | |
| C5 — SIEM / Security-Lake federation | ✅ decided + captured (ADR-PROP-siem-lake-federation.md) | |
| C6 — detection engine + rule-editor depth + auto-rollback | ✅ decided + captured (ADR-PROP-detection-engine-depth.md) | |
| C7 — ML / behavior analytics depth | ✅ decided + captured (ADR-PROP-ml-behavior-analytics-depth.md) | **C7 FOLD pending** (edge-mergeability research done; resolve D-C7-1 + fold into ADR-PROP) |
| C8 — PrismQL deliverables | ✅ decided + captured (ADR-PROP-prismql-deliverables.md) | **C8 FOLD pending** (AS-OF/OCSF bitemporality research done; resolve OQ-C8-ASOF + OQ-C8-OCSFVER + fold) |
| **C9 — config-management** | 🔶 PARTIAL — Q1+fast-revert+bootstrap-recovery LOCKED; Q2 in progress; Q3 not started | **RESUME HERE** — Q2 crux = health-signal posture (hard-failure-only vs soft-regressions) |
| **C10 — Query.io competitive refresh** | 🔬 research DONE (`queryio-competitive-refresh-2026-06-27.md`) | NEEDS DISCUSSION: 8 gaps (highest: OOTB detection content + rule-translation; auditable-agent evidence-package; Security-Lake-via-Athena) |
| **B — integration capstone** | ⏳ LAST | brief-reframe deltas + consolidated epic/ADR/story list — gated on brief-reframe HUMAN sign-off §5.1 |

## 4. PENDING FOLDS (research done — execute on resume)

### C7 FOLD
Research file: `research/edge-ml-mergeability-depth-2026-06-27.md`

Resolve D-C7-1: representation-change escape hatches make non-mergeable primitives mergeable-EXACT:
- EWMA → forward-decay (U,V) sufficient-stat
- reservoir → random-key/bottom-k
- clustering → BIRCH CF-vectors additive (confirms human hypothesis)

So edge-ML uses mergeable-exact representations broadly; coarsening ≠ privacy (local-DP separate);
empirical test narrows to macro-clustering drift.

**Action:** fold into `specs/day2-design-decisions/ADR-PROP-ml-behavior-analytics-depth.md`.

### C8 FOLD
Research file: `research/prismql-asof-version-resolution-2026-06-27.md`

Resolve OQ-C8-ASOF + OQ-C8-OCSFVER via BITEMPORALITY:
- valid-time + transaction-time; one `AS OF KNOWN <T>` decision-time knob pins entity-resolution +
  OCSF-catalog-version + ideally C5/C6 data snapshot for reproducible forensics; fresh-by-default.
- DB-native temporal; no commercial security tool does true bitemporality (prism-novel).
- CAVEAT: DataFusion+iceberg-rust lacks native time-travel today (pin-data-snapshot half has real cost).

**Action:** fold into `specs/day2-design-decisions/ADR-PROP-prismql-deliverables.md`.

## 5. EXACT NEXT ACTION ON RESUME

1. **Resume C9 Q2 canary chat.** Open question to human: health-signal posture — hard-failure-only vs
   include-soft-regressions (my lean: include soft regressions with conservative threshold). Then Q3
   schema-versioning (now deployment-aware).

2. **Capture queue** (in order after Q2/Q3 discussions):
   1. C9 capture → ADR-PROP-config-management.md
   2. Dual-deployment capture → ADR-PROP-dual-deployment.md (NEW, cross-cutting)
   3. C7 fold → update ADR-PROP-ml-behavior-analytics-depth.md
   4. C8 fold → update ADR-PROP-prismql-deliverables.md
   5. C10 discussion + capture → ADR-PROP-competitive-positioning.md (or fold into vision doc §16)
   6. **B integration capstone** (LAST — gated on brief-reframe HUMAN sign-off §5.1)

3. Each capture → single path-scoped commit on `factory-artifacts`.

## 6. BASELINE (git state at this wrap)
- **factory-artifacts HEAD:** this wrap commit (run `git -C .factory log -1 --format='%h %s'`).
- Prior committed HEAD before this wrap: C8 PrismQL deliverables capture commit.
- Working tree otherwise clean (untracked `.DS_Store` only; live-factory files left unstaged).

## 7. Gaps / epics introduced (PROPOSED, not in STORY-INDEX)
Gaps **G-1 … G-36** (plus new cross-cutting dual-deployment gap). Proposed epics:
E-CACHE-DEMAND, E-CENTRAL-TRANSPORT/AUTHZ/OPS, E-SATELLITE-MESH, E-LAKE-CONNECTOR,
E-UI-ADMIN/CONSOLE/EMBEDDED-AI/EXTENSION, E-CONNECTOR-DYNAMIC, E-DETECT-*, E-RULE-XLATE,
E-ML-*, E-COLLECTOR-CLASS/PCAP, E-CHAIN-CACHE, E-STREAM-DETECT, E-DISSECTOR-NATIVE/OT,
E-CONFIG-MGMT, E-DUAL-DEPLOYMENT.
All deferred to morph (brief-reframe §5.1 HUMAN GATE).
