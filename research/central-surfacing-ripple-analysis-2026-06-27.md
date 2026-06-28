---
document_type: analysis
status: capture
do_not_execute: true
provenance: "2026-06-27 out-of-band day-2 cross-cutting ripple + consistency audit. ANALYSIS ONLY — does not modify any live spec, ADR, BC, story, STATE.md, or SESSION-HANDOFF.md."
produced_by: architect (analysis agent)
created: "2026-06-27"
touches_no_live_artifacts: true
---

# Cross-Cutting Ripple + Consistency Audit — Day-2 Vision Features

> **SCOPE CONSTRAINT.** This artifact is analysis-only. It is produced against the
> corpus of ADR-PROP capture files in `specs/day2-design-decisions/` plus selected
> sections of `specs/matured-vision-day2-requirements.md`. It does NOT modify any
> live spec, ADR-registry artifact, BC, story, STATE.md, or SESSION-HANDOFF.md.
> It informs the human's Option 1 vs Option 3 decision and surfaces ADR-PROPs
> requiring correction once that choice is locked.

---

## The Two Constraints Under Test

**Constraint 1 — Central is the SOLE user-interaction surface.** Clients only log
into Central. "Central stays blind" means the operator/vendor has no AT-REST access to
raw sensor data — it does NOT mean the client cannot see their own data. The client
views data THROUGH their authenticated central session.

**Constraint 2 — Satellites are headless data-plane appliances.** All user
interaction, control, configuration-authoring, results-viewing, and administration
happens at Central. Satellites receive pushed config and return OCSF results;
they are not user-login surfaces.

---

## Section 1 — Per-Feature Ripple Matrix

The following matrix covers every feature captured in the Day-2 ADR-PROP corpus.
"Computes" = where the primary computation runs. "Stores" = where results or state
are persisted beyond a single query. "Surfaces at Central" = whether an analyst can
see or interact with it from the Central browser session. "Constraint Conflict" = any
violation of Constraints 1 or 2.

| Feature | Computes | Stores | Surfaces at Central | Constraint Conflict | Option 1 Impact | Option 3 Impact |
|---------|----------|--------|---------------------|--------------------|--------------------|-----------------|
| **C1 — Central deployment** (transport, OAuth 2.1, SS-26, case-management) | Central | PostgreSQL (bundled, central-only) | YES — full browser console (S1–S4), admin (U1) | None. PostgreSQL is control-plane only, never satellite. Case-management state is central by design. | n/a (control-plane) | n/a (control-plane) |
| **C2 — Satellite Mesh** (OCSF query execution, sensor credential resolution) | Satellite (query + normalization); Central (planning, fan-out) | Satellite-local RocksDB (store-and-forward, RetentionCache hot tier); Central NEVER stores raw sensor data (D-C2-12 HARD INVARIANT) | YES — query results surface at Central after normalization; analyst never interacts with satellite directly | None. Satellites are headless as required. D-C2-12 hard invariant ensures only normalized OCSF results transit. | Pass-through is native to C2 — Central proxies, never persists raw results. Analyst gets live result per query. | Central persists OCSF-normalized results under CMEK. Richer dashboard UX; no raw data at rest for operator. |
| **C3 — Capability Descriptor + Pushdown** (join feasibility, cost-based DEGRADE) | Central (capability catalog, plan-time); Satellite (execution) | No persistent state for capability catalog decisions — evaluated at plan time per query | YES — cost-based DEGRADE status visible in query results / coverage metadata | None. Capability resolution happens at Central planning layer. | Capability decisions are per-query ephemeral by nature — no persistence needed. Option 1 fully compatible. | No material difference. Capability decisions are stateless. |
| **C4 — Dynamic Schema Connectors** (WASM escape-hatch, boundary normalization) | Satellite/Edge (connector execution, normalization) or Central (for cloud sources) | Connector schema definitions stored in DB-authoritative config store (Central, C9) | YES — schema definitions viewed/authored ONLY at Central (PIV-C9-001). Connector execution is headless. | None. D-C9-Q1-AUTHORITY: no schema authored outside DB/UI. Connectors are config-received, not config-authoring. | Connector schemas are control-plane config, not data-plane results. Stored at Central regardless of option. | Same as Option 1 for schema config. Result data storage follows Option 3 pattern. |
| **C5 — SIEM/Lake Federation** (Amazon Security Lake Hive-Parquet, Iceberg cold tier) | Central or satellite (DataFusion query against external lake) | External SIEM/lake holds the data. Prism cold Iceberg tier at Central/regional for RETAIN results. Residency = hard REJECT at plan-time for non-residency-compliant queries. | YES — query results surface at Central | Potential tension: OQ-DEPLOY-2 "result-transit residency" — even OCSF-normalized results from SIEM/lake federation may carry PII. This is an OPEN hardening item, not a Constraint 1/2 violation per se. | Each analyst query re-federates to the lake. No central result store needed for core use. Residency REJECT is plan-time and does not require result persistence. | Central persists federation results (normalized) under CMEK. Eliminates repeated re-federation for dashboards. Residency enforcement still at plan-time; CMEK provides operator zero-access at rest. |
| **C6 — Detection Engine Depth** (MATCH_RECOGNIZE, RBA, backtesting, auto-rollback) | Satellite/Edge (MATCH_RECOGNIZE execution, RetentionCache hot-tier); Central (rule management, case creation, auto-rollback orchestration) | Detection state in RocksDB (satellite-local or Central RetentionCache); Findings/cases in PostgreSQL (Central); Rule versions in embedded git (git2) at Central (C9). | YES — detection findings, cases, rule editor ALL at Central (S2 console + admin) | None. Rule authoring is Central-only (C9 PIV-C9-001). Case management is Central-only (PostgreSQL). Detection state storage at satellite does NOT create a user-login surface (headless per Constraint 2). | Findings surface at Central via live re-query of RetentionCache. Dashboard aggregates require repeated re-queries per analyst load. No long-term trend data unless RETAIN is active. | Central caches findings under CMEK; dashboard aggregates are pre-materialized; auto-rollback history is persistent without re-query. This is the substantially richer UX for a SOC detection team. |
| **C7 — ML/Behavior Analytics** (streaming estimators, online learning, ModelState CF) | Satellite/Edge (Phase 1 query-time; Phase 2 online update in RocksDB ModelState CF) | Phase 1: no persistent state beyond RetentionCache. Phase 2: RocksDB ModelState CF (new column family) at satellite/edge. Phase 3: optional FeatureStore CF or Iceberg. | YES — ANOMALY_SCORE results surface in query results at Central; behavioral trend dashboards at Central (S2) | None. ML computes at edge (Constraint 2 compliant). Central surfaces results via query (Constraint 1 satisfied). PIV-C12-5 isolates graphs/vectors per tenant. | Anomaly scores are live per-query results. Historical trend dashboards require repeated re-queries per load. Long-baseline anomaly detection works via RetentionCache but without Central cache, each analyst load re-queries. | Central caches anomaly scores and behavioral trends under CMEK; dashboards materialize across time without per-load re-query; richer UEBA investigation UX at Central. |
| **C8 — PrismQL Deliverables** (pipe surface, BITEMPORALITY, entity resolution, LSP) | Central (plan, parse, desugar); Satellite (execution); Cold Iceberg tier for AS OF KNOWN queries | Query results ephemeral by default (RetentionCache at satellite/edge; no Central result store in base design). OQ-C8-DATASNAPSHOT: cold-tier data-snapshot pinning DEFERRED. | YES — PrismQL editor (Monaco, S2), results explorer, LSP all at Central | None. Query authoring is Central-only (Constraint 1). Satellites execute but do not surface to users (Constraint 2). | PrismQL is natively ephemeral-oriented. Option 1 is the base model. Forensic reproducibility (AS OF KNOWN) works within the RetentionCache for hot-tier; cold-tier data-snapshot gap remains open. | Central caches query results under CMEK; repeated queries for the same window avoid re-federation; AS OF KNOWN forensic replay could use the Central cache as a data snapshot (partially closes OQ-C8-DATASNAPSHOT). |
| **C9 — Config Management** (DB-authoritative, UI-ONLY, embedded git, ArcSwap) | Central (authorship, version control, canary apply); Satellite (auto-receive pushed config) | PostgreSQL at Central (DB-authoritative); git2 embedded at Central for detection content; Satellite receives config push (RocksDB or SQLite satellite-local). | YES — ALL config authoring and viewing at Central (PIV-C9-001). Satellites are auto-receivers, not config authors. | **FULLY CONSISTENT with both constraints.** D-C9-Q1-AUTHORITY is the strongest enforcement of Constraint 2 across the entire corpus. | Config is control-plane, not data-plane. Not affected by Option 1 vs Option 3 data result storage choice. Central always holds config. | Same as Option 1 for config. Config is already Option 3-patterned (always-at-Central-encrypted). |
| **C10 — Competitive Positioning** (C3 join framing, OT/trust moat, gap analysis) | Not a runtime feature — positioning document | Not a runtime feature | YES — positioning content informs product UX and honest-limit disclosure at Central | None. Analysis-only artifact. | n/a | n/a |
| **C11 — Prism Intel** (CVE/CPE/KEV feed-down, edge advisory generation, PIV-C11-001) | Central (feed aggregation, intel corpus management); Satellite/Edge (match-at-edge, advisory generation with local asset context) | Intel corpus at Central (CVE/CPE/KEV/EPSS/VEX). Local asset inventory NEVER at Central (PIV-C11-001 HARD INVARIANT). Advisory results in Entity 360 graph at edge. Opt-in central-match (D-C11-2): consent-gated, SaaS non-BYOC only — if used, asset identifiers DO transit to Central. | YES — advisory results and CVE exposure dashboards surface at Central via Entity 360 queries; intel corpus management at Central admin (U1). | **TENSION in D-C11-2 (opt-in central-match).** PIV-C11-001 states "Central NEVER receives raw asset identifiers." D-C11-2's opt-in central-match mode explicitly requires asset identifiers to transit to Central (for match). This is a CONSENTED exception, not a violation — but consent governance is OQ-C11-3, which is open. If Option 3 is chosen, the CMEK-at-rest protection for the consented central-match results must be explicitly specified. | Feed-down MATCH-AT-EDGE is the native model. Advisory results live in the edge graph and are surfaced via Entity 360 query at Central. Option 1 is fully consistent with C11's base architecture. | Opt-in central-match results (D-C11-2) would naturally use Option 3 CMEK encryption if Central stores match results. Richer central exposure dashboard without re-querying edge. Requires closing OQ-C11-3 consent model. |
| **C12 — Prism Context/KG+Vector** (indradb, usearch, lancedb, Entity 360, GraphRAG) | Satellite/Edge (graph construction, vector embedding via fastembed/ort on-box, ANN search, community summarization) | indradb (RocksDB-backed graph), usearch (hot-in-memory ANN), lancedb (cold on-disk vector) — ALL at satellite/edge per PIV-C12-5. Apache AGE + pgvector = DEFERRED central-tier escape hatch (not in scope). | YES — Entity 360 queries surface at Central via S2; GraphRAG summaries surface at Central. Raw telemetry NEVER leaves box for embedding (PIV-C12-2). | None. Compute at edge, results via query at Central — perfectly consistent with both constraints. | Entity 360 queries re-run at Central against edge graph on each analyst load. GraphRAG community summaries must be re-fetched per view. No Central cache means repeated cross-hop latency. | Central caches Entity 360 query snapshots and GraphRAG summaries under CMEK. Richer investigation panel performance. AGE+pgvector deferred Central escape hatch aligns with Option 3 if Central graph tier is ever activated. |
| **C2 Dual Deployment** (BYOC, MSSP-managed, client-managed; OQ-DEPLOY-2 open items) | Per deployment profile | Per deployment profile | YES — all three models serve Central as the user surface | **OQ-DEPLOY-2 is the most significant open risk.** Four hardening gaps: (a) result-transit residency (OCSF-normalized results may carry PII as they transit satellite→Central); (b) metadata-leakage audit (query text logged at Central may reveal business-sensitive patterns); (c) ephemeral dial-home tokens (satellite enrollment tokens need TTL/rotation); (d) CMEK for central metadata (SS-26 covers per-tenant DEK for credentials; does it extend to ALL Option 3 cached results?). Gap (d) is directly Option 3 scoping. | OQ-DEPLOY-2 gaps are partially mitigated: ephemeral pass-through minimizes Central-at-rest exposure for (a) and (d). Query text logging for (b) still occurs regardless of option. Dial-home tokens (c) are independent of option. | Option 3 adds risk surface for OQ-DEPLOY-2 gaps (a) and (d): OCSF results ARE persisted centrally, so result-transit residency becomes result-at-rest residency. CMEK resolves operator zero-access but does NOT resolve analyst-visible PII residency if tenant holds CMEK keys. |
| **S3 Agent Runtime** (4-component, Vercel AI SDK, per-tenant-DEK conversation store) | Central (all four components: Orchestrator, Model Router, Tool Mediator, Output Hardener) | Per-tenant-DEK encrypted conversation store at Central (RESOLVED 2026-06-27 to be server-side from day one) | YES — conversation canvas (S3) at Central browser session | **S3 conversation store is ALREADY Option 3 by explicit human decision (2026-06-27).** This is the reference implementation for CMEK/per-tenant-DEK central storage. Consistent with Constraint 1 (Central is the user surface) and Constraint 2 (satellite is headless). | Option 1 for S3 would mean browser-local only (the pre-resolution position). The human has OVERRIDDEN this: server-side store is required. Option 1 is NOT the chosen model for S3. | S3 conversation store is Option 3 by explicit human choice. This decision creates a PRECEDENT that certain derived/contextual data (not raw sensor data) IS stored centrally under per-tenant-DEK. It is the strongest signal in the corpus favoring Option 3 globally. |
| **SS-26 Secret Broker** (SecretBackend trait, per-tenant DEK, KMS key hierarchy) | Central (broker, KMS, DEK vault); Satellite (satellite-local SecretBackend for satellite-local credentials — HD-5 RESOLVED) | Central: encrypted credential ciphertext under per-tenant DEK under KMS master. Satellite: FULL-LOCAL credential storage (no Central round-trip for satellite cred resolution). | YES — credential management UI at U1 (admin console); credential rotation API at Central | None. HD-5 resolution (satellite FULL-LOCAL) is consistent with satellite headless design. Central-vend-then-cache is managed-mode option only. | SS-26 is pure control-plane key management. Not affected by Option 1 vs 3. | SS-26's per-tenant DEK is the ENABLING MECHANISM for Option 3. CMEK = tenant controls the DEK that encrypts their cached results. SS-26's key hierarchy is the foundation on which Option 3 is built. |
| **SSO Identity** (OIDC + SAML 2.0, 7-role RBAC, SCIM 2.0, JIT, TOTP break-glass) | Central (all identity and RBAC administration) | PostgreSQL at Central (tenant/user/IdP config, RBAC, sessions) | YES — all identity and access management at Central admin (U1) | None. Identity administration is Central-only by design. Satellites have no user-login surface. Satellite SPIFFE identity is machine identity, not user identity. | SSO is control-plane. Not affected by Option 1 vs 3. | Same as Option 1. |
| **Storage Engine Taxonomy** (RocksDB/Iceberg/PostgreSQL bundled/SQLite embedded) | Central: PostgreSQL (control-plane) + Iceberg (cold analytic). Satellite: RocksDB (hot data-plane) + SQLite (satellite control-plane). | Per engine as described. | YES — all four engines are invisible to the analyst; the analyst sees data, not engines. | **Load-bearing note:** PostgreSQL is BUNDLED central-only, NEVER external/cloud. This is the engine for case management, RBAC, config, audit, tenant/user, identity. The §14.3 reconciliation confirms PostgreSQL intro is for CONTROL-PLANE, NOT DATA-PLANE. This is consistent with both constraints. | Iceberg cold tier at Central/regional is for RETAIN + cold query path (OCSF results explicitly persisted by tenant directive). This is tenant-directed cold storage, compatible with Option 1 (tenant opts into RETAIN). | Option 3 adds a new Iceberg or RocksDB partition at Central for ALL result caching. SS-26 per-tenant DEK would encrypt the partition. CMEK key custody would be tenant-held. |
| **Web Stack** (TypeScript SPA React, Axum/Tokio backend, Monaco, nine security properties) | Central (Axum backend, React SPA served from Central) | Browser localStorage for ephemeral session state. No additional persistent store beyond what is already defined per-feature. | YES — the entire UI stack is Central-only by definition (Constraint 1 trivially satisfied) | None. The nine security canon properties (CSP, XSS, CSRF, clickjacking, HTTP-only cookies, session expiry, signed tenant tokens, prompt-injection hardening, S4 extension token) all apply at Central. | No impact. | No impact. |
| **Sandboxed Expression Evaluator + Widget DSL** (ANTLR4, Zod gate, 54 primitives, OCSF-aware extensions) | Central (Zod schema validation, React widget renderer, ANTLR4 expression evaluator — all browser-side in the SPA) | Browser-local: Zustand (in-session) + localStorage (cross-refresh). No server-side widget schema or canvas layout persistence in S3 v1. Canvas persistence deferred (future story). | YES — conversational canvas (S3) at Central browser session | None. Widget validation and rendering are Central browser-side. Ephemeral-only canvas state is explicitly production-grade for v1 (blast-radius argument). | Canvas state is browser-local. Option 1 is the native model for widget/canvas persistence. | Canvas persistence to Central (future story, deferred in D) would be Option 3. The ADR explicitly defers server-side canvas persistence; CMEK would be the encryption mechanism when/if added. |
| **PrismQL SEQUENCE Sugar + WATCH…UNLESS** (MATCH_RECOGNIZE, AbsenceWindowNode, RocksDB timer state) | Satellite/Edge (MATCH_RECOGNIZE NFA execution, AbsenceWindowNode timer evaluation) | RocksDB at satellite: partial-match state, per-partition timer state. NOT PostgreSQL (16.2 decision #5). | YES — detection results surface at Central via case/finding pipeline. SEQUENCE rule editor at Central (S2 + MCP + CLI). | None. Execution at satellite (headless). Results surface at Central. Rule authoring at Central (C9 PIV-C9-001 governs). | Per-query detection results surface as findings that flow to Central case management. Option 1: analyst sees findings via live re-query or PostgreSQL case records. | Central caches detection result streams under CMEK. Richer timeline reconstruction for SEQUENCE-based kill-chain investigations without repeated re-query. |
| **ML Phasing** (Phase 1 statistical, Phase 2 ModelState CF, Phase 3 pluggable backends) | Phase 1: satellite/edge query-time. Phase 2: satellite/edge (ModelState CF in RocksDB). Phase 3: satellite/edge + optional external ModelBackend (AD-017 applies — feature vectors, not raw data). | Phase 1: no persistent ML state. Phase 2: RocksDB ModelState CF at satellite/edge. Phase 3: optional FeatureStore CF or Iceberg at satellite/edge. | YES — anomaly scores, behavioral baselines, and model-driven detections surface at Central via query results and dashboards. | None for Phase 1 and 2. Phase 3 external ModelBackend: AD-017 applies at the boundary — feature vectors, not raw credentials or PII-containing OCSF records. No satellite-to-Central model state transit is specified. | Phase 1/2 are natively compatible with Option 1 (live query-time results). Behavioral trend dashboards require repeated re-queries per analyst load. | Central caches ML-derived scores and behavioral trends under CMEK. Richer UEBA dashboards; trend analysis without per-load re-federation. Consistent with C7 findings. |
| **PO Ratifications** (value-prop #5, §2.4 tradeoff softening, §1.x framing) | Not runtime | Not runtime | n/a | None. These are product brief amendments pending §5.1 sign-off. | n/a | n/a |

---

## Section 2 — Conflicts and Inconsistencies (Ranked by Severity)

### CONFLICT-1 — SEVERITY: HIGH — S3 Conversation Store Creates Option 3 Precedent That Is Not Propagated

**Finding:** The S3 ADR-PROP (ADR-PROP-s3-agent-runtime.md) was updated on 2026-06-27
(human decision) to require a server-side per-tenant-DEK encrypted conversation store
from day one. This is unambiguously an Option 3 pattern: Central ALWAYS stores this
derived data, encrypted under per-tenant DEK (SS-26 key hierarchy), with operator
zero-access at rest.

**The inconsistency:** Every other ADR-PROP in the corpus still describes itself as
"ephemeral" or "pass-through" as the base model, with Option 3 as a future possibility.
The S3 conversation store decision has already moved the corpus to Option 3 for one
subsystem without updating the framing in other subsystems that would naturally share
the same pattern.

**Affected ADR-PROPs:** ADR-PROP-s3-agent-runtime.md (sections describing the
pre-resolution "browser session/localStorage" position — these are now stale for the
Deployment Gating section and Alternatives sections, which the resolution note only
partially supersedes).

**Correction required:** When the Option 1 vs Option 3 choice is locked globally, the
S3 ADR-PROP's superseded sections (Deployment Gating §4 "Conversation state is
browser-local", Alternatives §D "server-DB persistence deferred") must be struck or
annotated as superseded. They currently exist in parallel with the resolution text and
will confuse story-writers.

---

### CONFLICT-2 — SEVERITY: HIGH — OQ-DEPLOY-2 Is Four Open Gaps That Interact Differently with Option 1 vs Option 3

**Finding:** ADR-PROP-dual-deployment.md lists four open hardening gaps under OQ-DEPLOY-2:
(a) result-transit residency, (b) metadata-leakage audit (query text), (c) ephemeral
dial-home tokens, (d) CMEK for central metadata.

Gap (d) "CMEK for central metadata" is directly the Option 3 enabling requirement.
If Option 3 is chosen, gap (d) must be closed before SaaS launch (the ADR-PROP says
so explicitly).

Gap (a) "result-transit residency" changes character depending on option:
- Under Option 1: OCSF-normalized results transit from satellite to Central during
  query execution but are NOT persisted at Central at rest. The residency risk is
  in-transit exposure, not at-rest exposure.
- Under Option 3: OCSF-normalized results ARE persisted at Central under CMEK.
  The operator has zero-access at rest, but the tenant holds the key and data is
  at rest at Central. This converts an in-transit residency risk to an at-rest
  residency risk where the tenant IS able to see their data (correct, per Constraint 1),
  but Central's jurisdiction/geography governs data residency even with CMEK.

**Inconsistency:** The four OQ-DEPLOY-2 gaps are listed together as if they are a
uniform block. They are not — they have different urgency and different relationships
to the option choice. They need to be disaggregated before brief-reframe.

**Correction required:** When option is locked, split OQ-DEPLOY-2 into:
- Pre-launch REQUIRED (independent of option): gap (b) metadata-leakage audit, gap (c)
  dial-home tokens.
- Pre-launch REQUIRED if Option 3 chosen: gap (d) CMEK for central metadata.
- In-transit hardening (independent of option): gap (a) residency for in-flight results.
- At-rest residency (only if Option 3): data jurisdiction for persisted CMEK results.

---

### CONFLICT-3 — SEVERITY: HIGH — PIV-C11-001 vs D-C11-2 (Central-Match Mode)

**Finding:** PIV-C11-001 states as a hard invariant: "Central NEVER receives raw asset
identifiers from any client network." D-C11-2 defines an "opt-in central-match mode"
(consent-gated, SaaS non-BYOC only) where the satellite sends a privacy-preserving
probe to Central to match local assets against the CVE/CPE corpus.

**The inconsistency:** If the privacy-preserving probe contains any identifier that
identifies a specific asset (even a hash or normalized identifier), and Central stores
the match result (especially under Option 3 CMEK), then PIV-C11-001's "NEVER receives
raw asset identifiers" is technically violated by the hash/probe content even if not
the plaintext identifier.

**OQ-C11-3** acknowledges this tension as an open consent governance question, but the
invariant text in PIV-C11-001 has not been amended to acknowledge the consented exception.

**Correction required (ADR-PROP-prism-intel.md):** Either (a) amend PIV-C11-001 to read
"Central NEVER receives raw asset identifiers except via opt-in central-match mode under
D-C11-2 consent governance (OQ-C11-3)," or (b) strengthen the central-match
privacy-preserving probe design so it is mathematically guaranteed to prevent
identifier reconstruction at Central even if Central is compromised. Option (b) is
technically stronger (PSI or differential privacy hash). Option (a) is pragmatic but
weakens the invariant.

This must be resolved before story decomposition for E-INTEL-FEED-001.

---

### CONFLICT-4 — SEVERITY: MEDIUM — ADR-PROP-s3-agent-runtime.md Has Stale Internal Framing (Pre-Resolution Text Still Present)

**Finding:** The ADR-PROP-s3-agent-runtime.md Deployment Gating section contains
(verbatim, in the body of the decision): "Conversation state is browser-local in v1
(Zustand/localStorage). No server-side conversation database is included in v1;
persistence is a named future story requiring explicit human authorization."

This text was NOT superseded by the 2026-06-27 resolution note in the Open Decisions
section. The resolution note says it "supersedes the 'browser session/localStorage only
for v1' framing in the Deployment Gating section," but the Deployment Gating section
body still contains the old text. A reader who does not reach the Open Decisions section
will see the old model as current.

**Correction required:** Strike or annotate the three stale passages in Deployment
Gating + Consequences + Alternatives §D of ADR-PROP-s3-agent-runtime.md at morph time.

---

### CONFLICT-5 — SEVERITY: MEDIUM — Storage Taxonomy §14.3 PostgreSQL-for-Control-Plane Reconciliation Not Fully Propagated

**Finding:** ADR-PROP-storage-engine-taxonomy.md explicitly reconciles that "PostgreSQL
introduction is for CONTROL-PLANE not DATA-PLANE" (§14.3 reconciliation). This is a
load-bearing design correction — it prevents the "but PostgreSQL could hold results"
interpretation.

However, ADR-PROP-central-deployment-access-layer.md (C1) references PostgreSQL for
case management, identity, and config — which is correct. But it does not cite the
§14.3 reconciliation or the storage taxonomy ADR-PROP. There is no explicit firewall in
C1 against future story-writers using PostgreSQL for caching query results "because it's
already there."

**Correction required:** At morph time, the central-deployment ADR (ADR-050) should
cite the four-engine storage taxonomy ADR and include an explicit prohibition: "PostgreSQL
is CONTROL-PLANE only. Query result caching and detection correlation state are
RocksDB (hot) and Iceberg (cold) only."

---

### CONFLICT-6 — SEVERITY: MEDIUM — C8 OQ-C8-DATASNAPSHOT and Option 3 Forensic Reproducibility

**Finding:** C8 (ADR-PROP-prismql-deliverables.md) defers cold-tier data-snapshot
pinning as OQ-C8-DATASNAPSHOT: "RETAIN at the data level (full AS OF KNOWN
reproducibility with cold-tier data) is DEFERRED." Under Option 3, Central persists
OCSF-normalized results under CMEK. This creates a pathway to partial forensic
reproducibility that is NOT the same as Iceberg time-travel data-snapshots, but could
be confused with it.

**Risk:** Story-writers for the forensic investigation features might treat Option 3's
Central result cache as a substitute for the deferred Iceberg data-snapshot mechanism.
These are different: a Central result cache under CMEK stores the OUTPUT of a query at
a point in time; an Iceberg data-snapshot enables REPLAY of the same query against the
same input data. They are not interchangeable for forensic correctness.

**Correction required:** When Option 3 is locked, add a PIV or Note to the Option 3
cache architecture that explicitly states: "Central CMEK result cache is NOT a
substitute for OQ-C8-DATASNAPSHOT (Iceberg data-snapshot for query replay). The
cache stores outputs; forensic replay requires input-level snapshots."

---

### CONFLICT-7 — SEVERITY: LOW — ADR-PROP-satellite-mesh.md D-C2-12 and OQ-DEPLOY-2(a) Require Alignment

**Finding:** D-C2-12 HARD INVARIANT: "Only normalized OCSF results transit the conduit,
never raw sensor data." OQ-DEPLOY-2(a) notes "result-transit residency: OCSF results
may carry PII." These two statements are NOT in conflict (OCSF normalization does not
strip PII — hostnames, IP addresses, process names, user accounts are all OCSF
fields), but the ADR-PROPs treat them as separately scoped concerns.

**Risk:** An implementer who reads D-C2-12 might conclude that the conduit is
PII-safe because it carries OCSF rather than raw API responses. This is incorrect —
OCSF results carry first-class PII in standard fields.

**Correction required:** Add a note to ADR-PROP-satellite-mesh.md (or the resulting
ADR-CCC): "D-C2-12 governs data format (OCSF normalized), not PII content. OCSF events
include PII (hostnames, user accounts, IPs) in standard fields. OQ-DEPLOY-2(a)
result-transit residency governance applies to all conduit traffic including OCSF."

---

## Section 3 — Surfacing-Burden Analysis

**Which features need Central persistence to deliver their stated UX?**

This is the core question for the Option 1 vs 3 decision. The answer is not binary —
it is a spectrum from "ephemeral pass-through works fine" to "ephemeral pass-through
fundamentally cannot deliver the UX."

### Tier A: Must Have Central Persistence (Ephemeral Cannot Deliver)

These features have a stated UX requirement that is architecturally impossible with
pure pass-through:

1. **S3 Conversation History** — Already resolved to Option 3 by human decision.
   Multi-device continuity and the C10 GAP-Q2 evidence-package/audit requirement both
   require server-side persistence. Browser-local cannot survive tab close.
   CMEK: per-tenant DEK via SS-26. Already decided.

2. **C6 Detection Findings Timeline / SOC Dashboard** — A SOC dashboard showing
   "alerts this week," "MTTR trend," "active incidents" requires accumulated findings
   data over time. If findings are only ephemeral (live re-query of RetentionCache),
   then after the RetentionCache TTL expires the findings are gone. The RetentionCache
   TTL is days-to-weeks for hot tier; detection findings for investigation purposes
   often need days to months retention. PostgreSQL (control-plane) already persists
   case records and alert findings. This is NOT an Option 1 vs 3 question for case
   records — case records are already in PostgreSQL (control-plane) by design.
   What Option 3 adds here is: raw detection result streams/events cached at Central
   for dashboard aggregation without re-querying the edge on every analyst load.

3. **C7/C12 Entity 360 / UEBA Trend Dashboards** — Showing behavioral baselines and
   anomaly trends over weeks/months requires either (a) repeated live re-federation
   every time an analyst opens the dashboard, or (b) a Central cache. For large
   deployments with many analysts, (a) has prohibitive fan-out cost. (b) is Option 3.

### Tier B: Works Ephemeral But Is Materially Better with Persistence

These features function correctly with Option 1 but provide substantially richer
analyst UX with Option 3 central caching:

4. **C11 Intel Advisory Decorations on Entity 360** — Live pass-through works.
   Option 3 pre-populates advisory overlays in the Central Entity 360 view without
   repeated satellite round-trips.

5. **C8 Forensic Investigation Results** — Live re-query per investigation works.
   Option 3 allows "pin this investigation result" to a Central cache for return to
   later without re-running the query.

6. **C6 Detection Backtesting Results** — Backtesting against Iceberg cold tier
   produces large result sets. Re-running them per analyst view is expensive.
   Option 3 caches backtesting results under CMEK.

7. **C12 GraphRAG Community Summaries** — Community detection and Leiden summarization
   are expensive operations. Re-running per analyst panel load is wasteful. Option 3
   caches summaries at Central.

### Tier C: Ephemeral Pass-Through Is Fully Sufficient

These features have no material UX benefit from Central persistence:

8. **C1 Transport + C9 Config Management** — Control-plane only. Already at Central
   in PostgreSQL/git2. Not affected by Option 1 vs 3.
9. **C2 Satellite Topology + C3 Capability Pushdown** — Stateless plan-time decisions.
10. **Widget DSL + Sandboxed Evaluator** — Browser-local by design. Canvas persistence
    deferred to future story.
11. **PrismQL Grammar + SEQUENCE Sugar** — Stateless query compilation.
12. **ML Phase 1 Statistical Functions** — Query-time aggregates with no persistent state.
13. **SSO + RBAC** — Already persisted in PostgreSQL (control-plane).
14. **SS-26 Secret Broker** — Already persisted under encryption at Central.
    Not a data-plane persistence question.

---

## Section 4 — Config-Authoring Note (C9 / Constraint 2 Consistency)

**Verification:** C9 (ADR-PROP-config-management.md) is the most explicit enforcement
of Constraint 2 in the entire corpus.

**Key findings:**

1. **D-C9-Q1-AUTHORITY** is unambiguous: "config NEVER authored outside DB/UI in
   production." This directly enforces Constraint 2 for all config surfaces —
   connector definitions, sensor TOML, detection rules, retention policies. No
   satellite, no CLI file, no git-committed TOML on an analyst machine enters
   production config.

2. **PIV-C9-001** ("DB-authoritative runtime config invariant") enforces this at
   the platform level — no filesystem TOML write path exists in production code.

3. **Satellite as auto-receiver:** Satellites receive pushed config via signed bundles
   (Ed25519/sigstore, reusing C11's air-gap delivery mechanism). They do NOT author,
   modify, or override config. This is fully consistent with Constraint 2.

4. **The two-tier apply model** (HIGH-BLAST canary required vs LOW-BLAST direct) also
   applies at Central only — there is no per-satellite canary gate. This is correct
   because satellites are data-plane executors, not config-authoring agents.

5. **Potential gap:** The C9 apply model describes canary rollout in terms of
   "High-Blast changes require N% canary before full rollout." In a multi-satellite
   deployment, does "canary" mean "deploy to N% of satellites" or "deploy to a Central
   staging tenant"? This needs clarification at morph time for the E-CONFIG-MGMT-001
   story decomposition. The current text implies Central-only canary gating, which
   is correct.

**Verdict:** C9 is FULLY CONSISTENT with Constraint 2. No conflict.

---

## Section 5 — Recommendation: Option 1 vs Option 3

### Summary Assessment

The corpus has ALREADY PARTIALLY COMMITTED to Option 3 via the S3 conversation store
human decision (2026-06-27). This was not framed as an Option 3 adoption — it was
resolved as a pragmatic UX need (multi-device continuity, audit trail). But it is
architecturally identical to Option 3 applied to one subsystem.

The question for the human is therefore not "Option 1 or Option 3 from scratch" but
rather "should the Option 3 pattern already established for S3 be extended to the
features in Tier A/B above?"

### Arguments for Option 1 (Hybrid, Ephemeral Default + Opt-in Cache)

**For:**
- Minimizes Central at-rest exposure for Tier C features (where ephemeral is sufficient).
- OQ-DEPLOY-2 gaps (a) and (d) are smaller scope under Option 1 — in-transit hardening
  is simpler than at-rest CMEK governance across all result types.
- Air-gap and BYOC deployments are simpler: no Central result cache infrastructure needed.
- Per-tenant opt-in allows tenants who DO NOT want Central persistence to operate fully
  ephemerally. Maximum flexibility.

**Against:**
- Tier A features (SOC dashboard trends, conversation history, UEBA baselines) have a
  hard UX wall. Ephemeral cannot deliver the stated analyst UX for these features.
  Option 1 for these features means the human must accept a degraded UX or re-architect
  those features specifically.
- Two-track implementation complexity: ephemeral path + opt-in cache path = two code
  paths to test, maintain, and secure. The opt-in cache path in practice becomes
  mandatory for the SOC console target persona, making the default ephemeral path
  vestigial for production deployments.
- The S3 conversation store already requires the CMEK/per-tenant-DEK infrastructure.
  Limiting it to S3 only and requiring opt-in everywhere else creates an inconsistent
  tenant experience.

### Arguments for Option 3 (Tenant-Keyed Cache Always)

**For:**
- Richest Central analyst UX by construction. SOC dashboard trends, UEBA baselines,
  conversation history, investigation pinning, GraphRAG summaries — all work without
  repeated re-federation.
- Operator zero-access at rest is a GENUINE MSSP trust differentiator. BYOC deployments
  specifically benefit: the MSSP operator cannot read client data even if they hold the
  Central infrastructure. The client holds CMEK keys.
- SS-26's per-tenant DEK hierarchy is already designed and committed — the encryption
  infrastructure for Option 3 already exists. Option 3 does not require a new security
  mechanism; it requires extending an existing one.
- The S3 conversation store precedent is set. Consistency across all Central-stored
  derived results is architecturally cleaner than a mixed model.
- Aligns with OQ-DEPLOY-2 gap (d) resolution: closing CMEK for central metadata IS
  the Option 3 design.

**Against:**
- OQ-DEPLOY-2 gap (d) must be explicitly closed before SaaS launch. This is scoped work.
- Data residency governance (OQ-DEPLOY-2 gap a, at-rest version) becomes a first-class
  requirement: the Central cluster's geography now determines residency for tenant data
  at rest, even if CMEK prevents operator access. EU tenants may require EU-region Central
  deployment. This is a deployment topology constraint, not an encryption constraint.
- CMEK key management burden shifts to the tenant. If a tenant loses their CMEK key,
  they lose access to all their Central-cached results. Key custody failure recovery
  must be designed (key escrow? key recovery ceremony?).
- Air-gap deployments that opt in to S3 must also provision CMEK infrastructure locally
  (SoftwareKms as default per SS-26 covers this; local SoftwareKms is the air-gap-safe
  default).

### Recommendation

**Choose Option 3 as the default, with air-gap/BYOC-first safe deployment posture
preserved via SoftwareKms.**

Rationale:

1. The S3 conversation store decision already committed the architecture to Option 3
   for one subsystem. Consistency demands extending this to all Central-stored derived
   results rather than a mixed model that creates per-feature storage policy complexity.

2. The SS-26 per-tenant DEK infrastructure is already designed and committed. The
   marginal cost of extending CMEK to all Central-cached results is primarily
   implementation and key-management UX, not a new architectural bet.

3. The Tier A features (SOC dashboard, UEBA trends, conversation history) cannot
   deliver their stated UX without Central persistence. The product brief and value
   propositions depend on these features. Option 1 would require downgrading the stated
   UX for core SOC console features.

4. Operator zero-access at rest is a genuine competitive differentiator for the MSSP
   target buyer. Option 3 delivers this by construction. Option 1 delivers it only for
   the ephemeral path, which the most valuable features cannot use.

5. OQ-DEPLOY-2 gap (d) is pre-launch required work regardless — it was already flagged.
   Choosing Option 3 does not ADD a new gap; it provides the design that closes the gap.

6. Air-gap and OT deployments are safe: SoftwareKms (the SS-26 default) is the
   CMEK backend for air-gap deployments. Central caching with local CMEK key custody
   works without internet connectivity.

**Boundary condition:** "Tenant-keyed cache ALWAYS" should be scoped to DERIVED RESULTS
(query outputs, detection findings, conversation history, anomaly scores, GraphRAG
summaries). It must NEVER apply to RAW SENSOR DATA. D-C2-12 hard invariant and
PIV-C11-001 are absolute — raw API responses and raw asset inventories never reach
Central. Option 3 applies only to the normalized/derived layer that already transits
to Central under both options.

---

## Section 6 — ADR-PROPs Needing Correction Once Option + Surfacing Principle Are Locked

The following ADR-PROP files contain stale, ambiguous, or incomplete content relative
to the analysis above. This list is the punch-list for the brief-reframe + morph
execution cycle.

| Priority | ADR-PROP File | Required Correction | Triggered By |
|----------|---------------|---------------------|--------------|
| P0 | `ADR-PROP-s3-agent-runtime.md` | Strike/annotate the three stale passages in Deployment Gating ("browser-local in v1"), Consequences ("browser-local limits blast radius"), and Alternatives §D ("deferred, named future story"). Replace with reference to the 2026-06-27 resolution. | CONFLICT-1, CONFLICT-4 |
| P0 | `ADR-PROP-dual-deployment.md` | Disaggregate OQ-DEPLOY-2 four gaps into: (a) pre-launch independent-of-option, (b) pre-launch required-if-Option-3, (c) in-transit hardening, (d) at-rest residency jurisdiction. | CONFLICT-2 |
| P0 | `ADR-PROP-prism-intel.md` | Amend PIV-C11-001 to acknowledge the consented exception in D-C11-2, OR specify the privacy-preserving probe design that prevents identifier reconstruction at Central. Close OQ-C11-3 before E-INTEL-FEED-001 stories. | CONFLICT-3 |
| P1 | `ADR-PROP-central-deployment-access-layer.md` | Add explicit prohibition: "PostgreSQL is CONTROL-PLANE only — not for query result caching." Cite storage taxonomy ADR. | CONFLICT-5 |
| P1 | `ADR-PROP-prismql-deliverables.md` | Add note: "Central CMEK result cache (Option 3) is NOT a substitute for OQ-C8-DATASNAPSHOT. Output caching ≠ input-level time-travel snapshot." | CONFLICT-6 |
| P1 | `ADR-PROP-satellite-mesh.md` | Add note to D-C2-12: "OCSF normalization governs data format, not PII content. OQ-DEPLOY-2(a) residency governance applies to all conduit traffic." | CONFLICT-7 |
| P2 | All Option 3-relevant ADR-PROPs | When Option 3 is locked, add a uniform "Central Cache" section to each affected ADR-PROP (C2, C6, C7, C8, C11, C12, S3) specifying: cache schema, per-tenant DEK encryption, retention policy, key custody model, and reference to SS-26. These are currently underdefined. | Surfacing-burden analysis — Tier A and B |
| P2 | `ADR-PROP-config-management.md` | Clarify "canary" apply model: is canary gating Central-staging-tenant or satellite-percentage? Add explicit statement. | Section 4 config-authoring note |
| P3 | `ADR-PROP-ml-behavior-analytics-depth.md` | If Option 3 is chosen: add a section on Central ML score caching (CMEK under per-tenant DEK). Phase 2 ModelState CF is satellite-edge-local; Central cache of derived scores is a separate concern. Ensure story-writers do not conflate them. | Surfacing-burden analysis — Tier B |
| P3 | `ADR-PROP-prism-context.md` | If Option 3 is chosen: the DEFERRED Apache AGE + pgvector central-tier escape hatch aligns with Option 3's direction. Clarify whether Option 3 choice accelerates or further defers this. | Surfacing-burden analysis — Tier B |

---

## Appendix — Files Read

All files under `/Users/jmagady/Dev/prism/.factory/specs/day2-design-decisions/`:

- `ADR-PROP-central-deployment-access-layer.md` (C1)
- `ADR-PROP-satellite-mesh.md` (C2)
- `ADR-PROP-capability-descriptor-pushdown.md` (C3)
- `ADR-PROP-dynamic-schema-connectors.md` (C4)
- `ADR-PROP-siem-lake-federation.md` (C5)
- `ADR-PROP-detection-engine-depth.md` (C6)
- `ADR-PROP-ml-behavior-analytics-depth.md` (C7)
- `ADR-PROP-prismql-deliverables.md` (C8)
- `ADR-PROP-config-management.md` (C9)
- `ADR-PROP-competitive-positioning.md` (C10)
- `ADR-PROP-prism-intel.md` (C11)
- `ADR-PROP-prism-context.md` (C12)
- `ADR-PROP-dual-deployment.md`
- `ADR-PROP-s3-agent-runtime.md`
- `secret-subsystem-sketch.md` (SS-26)
- `ADR-PROP-sso-identity.md`
- `ADR-PROP-storage-engine-taxonomy.md`
- `ADR-PROP-web-stack.md`
- `ADR-PROP-sandboxed-expression-evaluator.md`
- `ADR-PROP-widget-dsl-render-and-schema-validation.md`
- `prismql-sequence-sugar-decisions.md`
- `ml-depth-phasing.md`
- `po-ratifications.md`
- `SESSION-RESUME-2026-06-27.md` (not read — not in scope per task)

Selected sections from `/Users/jmagady/Dev/prism/.factory/specs/matured-vision-day2-requirements.md`:
- §3 (Architectural Pillars — §3.1 Central deployment, §3.2 Prism Satellite)
- §11 (Server-Deployment Pillars: Credentials, Configuration, UI)
- §16.4 (Open items / Next steps)
