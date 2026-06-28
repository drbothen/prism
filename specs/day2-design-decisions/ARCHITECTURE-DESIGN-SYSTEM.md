---
document_type: architecture-design-system
status: capture
do_not_execute: true
provenance: "2026-06-27 out-of-band day-2 side-analysis. CAPTURE ONLY — does not modify any live spec, ADR-registry artifact, BC, story, STATE.md, or SESSION-HANDOFF.md."
produced_by: architect (analysis agent)
created: "2026-06-27"
touches_no_live_artifacts: true
seeded_from: "research/central-surfacing-ripple-analysis-2026-06-27.md"
cross_refs:
  - specs/day2-design-decisions/ (all ADR-PROPs)
  - specs/matured-vision-day2-requirements.md §16.4
  - CLAUDE.md §Canonical Principle, §Conventions
---

# Prism Architecture Design System (ADS)

> **READ THIS FIRST.** This is a CAPTURE artifact (`do_not_execute: true`).
> It does NOT modify any live ADR, BC, story, or STATE.md. It is the canonical
> cross-cutting architecture reference for Prism Day-2 features (C1–C20 + B
> capstone). The morph execution cycle will translate these principles into live
> ADRs; this document is the design-system source of truth that ADRs must cite.
>
> Every new ADR-PROP and every day-2 feature story MUST pass the Conformance
> Checklist in Section C before merging into the live spec.

---

## Introduction — What the ADS Is

The UI design system provides tokens, components, and constraints so that every
UI surface looks and behaves the same way. The Architecture Design System does
the same for the architecture tier: it provides **principles** (the tokens —
stable, named invariants), **patterns** (the components — reusable structural
solutions), and **conformance rules** (the constraints — what must be true of
every new subsystem, ADR, or epic).

It is the "Prism Way" at the architecture level. It complements CLAUDE.md's
code-level Conventions but operates one tier higher: CLAUDE.md governs how code
is written inside a module; the ADS governs how modules, services, and data
flows are structured in relation to each other.

**Seeded from:** `research/central-surfacing-ripple-analysis-2026-06-27.md`,
which was the first systematic conformance run against all Day-2 ADR-PROPs. That
audit identified six conflicts (two HIGH, three MEDIUM, one LOW) that the
conformance checklist below would have caught at design time.

---

## Section A — Principles

Each principle carries a stable ID, rationale, and the source decisions that
generalize it. Principles are the architecture tokens: the things that are
unconditionally true about the Prism architecture regardless of which feature
or deployment profile is in play.

---

### P-ADS-01 — Central-Sole-Surface

**Statement.** All user interaction — query authoring, results viewing, alert
triage, config authoring, admin, investigation, AI conversation — happens
exclusively at the Central console. Satellites are headless data-plane
appliances. A satellite is NOT a user-login surface except for initial setup
and maintenance.

**Rationale.** Central is the trust boundary for identity, RBAC, and audit.
Splitting user interaction across satellites would shatter that boundary,
require per-satellite IdP integration, and destroy the single-pane-of-glass
value proposition. "Central stays blind" does NOT mean the client cannot see
their own data — it means the operator/vendor has no at-rest access to raw
client data. The client views their data THROUGH their authenticated Central
session.

**Source decisions.**
- D-C9-Q1-AUTHORITY (`ADR-PROP-config-management.md`): config NEVER authored
  outside DB/UI in production. The strongest single enforcement of this principle.
- PIV-C9-001: DB-authoritative runtime config invariant — no filesystem TOML
  write path in production code.
- `ADR-PROP-satellite-mesh.md` §Constraint 2: satellites are headless.
- Ripple audit Section 1 "Central is the SOLE user-interaction surface."
- All 21 feature rows in Section 1 of the ripple audit — every feature satisfies
  this principle; C9 and SSO are the strongest positive examples.

**Boundary.** The "headless except setup/maintenance" carve-out is narrow and
deliberate. Satellite bootstrap UI (join-token onboarding, break-glass
maintenance mode) is the only legitimate exception. A new feature that requires
an analyst to interact with a satellite dashboard directly violates this
principle and must be redesigned.

---

### P-ADS-02 — Operator-Zero-Access-At-Rest

**Statement.** The operator/vendor running Central infrastructure has NO at-rest
read access to raw or derived client data. "Central blind" means OPERATOR-blind;
authenticated clients view their own data through their Central session.

**Rationale.** This is the primary MSSP trust differentiator. An MSSP customer
choosing to route all security telemetry through a managed Central faces a
rational objection: can the MSSP see my data? Option 3 (Tenant-Keyed-Central-
Persistence, P-ADS-04) answers that question cryptographically: the operator
holds the infrastructure but not the key. Without this principle, Prism cannot
compete in sensitive verticals (OT, financial, healthcare, government).

**Source decisions.**
- SS-26 Secret Broker per-tenant DEK design (`secret-subsystem-sketch.md`).
- Option 3 adopted globally 2026-06-27 (human decision, confirmed in ripple audit
  §5 Recommendation).
- S3 conversation store OD-1 resolution: server-side per-tenant-DEK-encrypted
  store, day one.
- OQ-DEPLOY-2 gap (d): "CMEK for central metadata" — now a pre-launch REQUIRED
  item (ripple audit Section 2 CONFLICT-2).

**Boundary.** This principle governs at-rest data, not in-transit data. In-transit
residency (OQ-DEPLOY-2 gap a) is a separate concern. This principle also does NOT
apply to control-plane data that is not client-data: RBAC role definitions, IdP
config, audit log metadata — these have their own operator-access governance.

---

### P-ADS-03 — Derived-Results-Only-At-Central

**Statement.** Only DERIVED results may surface at or persist at Central:
query outputs, detection findings, anomaly scores, advisories, GraphRAG
summaries, conversation history. RAW sensor data, raw asset identifiers, and
credentials NEVER leave the edge. This is an absolute invariant — no consented
exception exists for raw sensor data.

**Rationale.** "Raw" data at Central would (a) break BYOC and OT deployments
where the customer explicitly controls data residency, (b) expose the operator
to raw customer telemetry even under CMEK (the raw data must be decrypted to
process), and (c) create a single point of breach that compromises all tenants.
OCSF normalization at the boundary is the mechanism that converts raw vendor
API responses into derived OCSF events — the conversion is irreversible in the
information-compression direction that matters for residency.

**Caution:** OCSF-normalized data IS NOT PII-safe. OCSF events carry hostnames,
IP addresses, user accounts, and process names as first-class fields
(ripple audit CONFLICT-7). D-C2-12 governs data FORMAT (OCSF normalized vs raw
API response), not data CONTENT. OQ-DEPLOY-2(a) result-transit residency
governance applies to all conduit traffic including OCSF.

**Source decisions.**
- D-C2-12 HARD INVARIANT (`ADR-PROP-satellite-mesh.md`): only normalized OCSF
  results transit the conduit, never raw sensor data.
- PIV-C11-001 (`ADR-PROP-prism-intel.md`): Central NEVER receives raw asset
  identifiers in the default edge-match path.
- Ripple audit §Recommendation boundary condition: "Tenant-keyed cache ALWAYS
  should be scoped to DERIVED RESULTS … It must NEVER apply to RAW SENSOR DATA."

**Important exception note.** D-C11-2 defines an opt-in central-match mode
(consent-gated, SaaS non-BYOC only) where a privacy-preserving probe may transit
from satellite to Central. PIV-C11-001 must be amended to acknowledge this
consented exception before E-INTEL-FEED-001 story decomposition (ripple audit
CONFLICT-3 P0 correction). The exception is narrow: PIV-C11-004 limits it to
the ONLY inventory-leaves-edge path.

---

### P-ADS-04 — Tenant-Keyed-Central-Persistence (Option 3, locked 2026-06-27)

**Statement.** Any derived result persisted at Central is encrypted under a
TENANT-HELD key (CMEK, implemented via SS-26 per-tenant DEK hierarchy). The
operator infrastructure holds ciphertext only — decryption requires the tenant's
key. This applies universally to all Central-cached derived results (query
outputs, detection findings, conversation history, anomaly scores, GraphRAG
summaries) without per-feature opt-in.

**Rationale.** The S3 conversation store decision (OD-1, 2026-06-27) established
Option 3 for one subsystem. The ripple audit showed that Tier A features (SOC
dashboard trends, UEBA baselines, conversation history) architecturally require
Central persistence — ephemeral pass-through cannot deliver the stated UX. Once
Central persistence is required, applying CMEK universally is simpler and more
secure than a mixed model that creates per-feature storage policy complexity.
The SS-26 per-tenant DEK infrastructure is already committed; extension to all
Central-cached results is marginal engineering cost.

**Air-gap safety.** SoftwareKms (SS-26 HD-1 resolution, 2026-06-27) is the
CMEK backend for air-gap and BYOC-first deployments. Central caching with
local CMEK key custody works without internet connectivity.

**Source decisions.**
- SS-26 Secret Broker HD-1 and HD-4 resolutions (`secret-subsystem-sketch.md`).
- S3 OD-1 resolution: server-side per-tenant-DEK-encrypted store, day one.
- Ripple audit §5 Recommendation: Option 3 as default.
- Human lock 2026-06-27 (this session).

**Boundary.** PostgreSQL is CONTROL-PLANE only — never for query result caching
(ripple audit CONFLICT-5; `ADR-PROP-storage-engine-taxonomy.md` §14.3
reconciliation). The Option 3 cache layer uses RocksDB (hot) and Iceberg (cold),
not PostgreSQL. A Central CMEK result cache is NOT a substitute for the deferred
Iceberg data-snapshot mechanism for forensic query replay — the cache stores
OUTPUTS, not inputs (ripple audit CONFLICT-6).

---

### P-ADS-05 — Edge-Computes / Central-Surfaces

**Statement.** Compute, raw data, and credentials live at the edge or satellite.
Central orchestrates, plans, and surfaces derived results to the authenticated
client session. The edge-to-central data flow is always: normalize → derive →
encrypt → surface.

**Rationale.** This principle unifies P-ADS-01 through P-ADS-04 into a single
structural rule for building new features. When designing a new capability, the
first question is: where does the computation happen? The answer is always: at
the edge. The second question: what surfaces at Central? The answer: only the
derived output, encrypted under the tenant key.

**Source decisions.**
- C2 `ADR-PROP-satellite-mesh.md`: satellite query + normalization, Central
  planning + fan-out.
- C7 `ADR-PROP-ml-behavior-analytics-depth.md`: ML computes at edge (query-time
  Phase 1, ModelState CF Phase 2); anomaly scores surface at Central.
- C12 `ADR-PROP-prism-context.md`: graph + vector + embedding all at satellite/
  edge; Entity 360 query results surface at Central via S2.
- All Tier A/B features in ripple audit Section 3.

---

### P-ADS-06 — Per-Tenant-Isolation

**Statement.** Strict per-tenant partitioning is enforced everywhere: graph/
vector/entity-resolution state, keys, config, cache, and audit. No cross-tenant
edges, similarity scores, or data joins. Nested tenancy rides ONE OrgId
abstraction — no super-tenant visibility layer exists.

**Rationale.** An MSSP is a multi-tenant system where each client IS a
competitor's intelligence picture. Cross-tenant data leakage would be a
catastrophic breach of client trust. The per-tenant DEK (P-ADS-04) is the
key-level enforcement; per-tenant partitioning in the data stores is the
structural enforcement.

**Source decisions.**
- PIV-C12-5 (`ADR-PROP-prism-context.md`): per-tenant graph/vector/resolution
  isolation — no cross-tenant edges.
- SS-26 per-tenant DEK envelope.
- `ADR-PROP-sso-identity.md`: tenant/user/RBAC in PostgreSQL per-tenant.
- OrgSlug newtype in existing codebase (CLAUDE.md §Conventions).

---

### P-ADS-07 — AI-Opaque (AD-017)

**Statement.** Credentials and raw sensitive data never transit AI context.
Entity and data masking (C16) extends this to sensitive data fields. AI
components receive only the minimum information needed for the task; resolution
of credentials and sensitive identifiers happens outside the AI reasoning loop.

**Rationale.** AI context windows are logs, often cached, and may transit API
boundaries. A credential that appears in an AI context is effectively a leaked
credential. The AI-Opaque principle ensures that Prism's AI components (S3
agent runtime, C12 GraphRAG, C7 ML) are useful without being dangerous.

**Source decisions.**
- AD-017 (existing, CLAUDE.md §Conventions: Conventions §Credentials).
- `ADR-PROP-ml-behavior-analytics-depth.md` Phase 3: external ModelBackend
  receives feature vectors, not raw credentials or PII-containing OCSF records.
- C16 entity/data masking (referenced in C12 cross-links).
- S3 agent runtime Tool Mediator design: credentials resolved at the tool boundary,
  not exposed to the model layer.

---

### P-ADS-08 — OCSF-Normalize-At-Boundary

**Statement.** Normalize ALL sensor data to OCSF at the adapter boundary. Higher
layers (knowledge graph, detection engine, ML, intel correlation) build on the
OCSF-normalized layer, never on raw vendor schemas. The normalization chokepoint
applies to ALL connectors including existing prism-sensors — no trusted-source
exemption exists.

**Rationale.** Building higher-order reasoning on raw vendor schemas creates
N×M coupling between every reasoning system and every vendor API. OCSF
normalization at the boundary collapses this to a single canonical schema that
all higher layers can reason about uniformly. The "no exemption" rule prevents
the architectural rot that occurs when "well-understood" sensors bypass
normalization and accumulate technical debt.

**Source decisions.**
- D-C4-1 (`ADR-PROP-dynamic-schema-connectors.md`): boundary-normalization
  scope = ALL connectors, NO trusted-source exemption.
- C5 D-C5-5 federation sources as connector plugins: all SIEM/lake sources
  inherit mandatory boundary-normalization chokepoint.
- Core architecture insight (`project_core_architecture_insight.md`): ephemeral
  federated query engine over sensor APIs; OCSF+protobuf normalization is the
  fundamental design.
- CLAUDE.md §Conventions §OCSF normalization.

**Version axis.** Normalization must handle OCSF version skew (e.g., Security
Lake ships OCSF 1.1/1.3 vs upstream 1.6.0). Normalize inbound UP to the target
version at the boundary; carry `metadata.version` for audit. This is a C5
decision that generalizes to any connector consuming older OCSF versions.

---

### P-ADS-09 — Config-DB-Authoritative / UI-Authored-At-Central

**Statement.** Configuration is authored ONLY at the Central UI, persisted in
the DB-authoritative config store (PostgreSQL at Central), and pushed to
satellites as signed bundles. Satellites are auto-receivers — they do NOT author,
modify, or override config. No filesystem TOML write path exists in production
code. No config is authored at the edge, CLI-side, or via git-committed files
in a production deployment.

**Rationale.** Config authority is the control plane's single most important
invariant. Distributed config authoring means distributed trust — a satellite
that can modify its own config can silently diverge from the operator's intent.
The DB-authoritative model enables atomic versioning, canary apply, and fast
revert that would be impossible with distributed filesystem config.

**Source decisions.**
- D-C9-Q1-AUTHORITY (`ADR-PROP-config-management.md`): the strongest single
  enforcement in the corpus.
- PIV-C9-001: DB-authoritative runtime config invariant.
- C9 two-tier apply model (HIGH-BLAST canary required, D-C9-Q2-TIERS).
- Hub-and-Spoke migration model (D-C9-Q3-MODEL): forward schema migration is
  Central-driven with satellite skip-version window + LTS required-stops.
- Satellite signed bundle delivery uses the same mechanism as C11 air-gap
  offline bundle (PAT-ADS-07 below).

---

### P-ADS-10 — Idempotent-Gated-Actions

**Statement.** All writes and actions are gated by default: RBAC + optional
approval + configurable autonomy level. Every action carries a mandatory
idempotency key. Actions fail-safe on error — they halt and escalate rather
than partially applying. System designers set the autonomy gates; agents propose
and operators confirm. No action executes outside of a gated, audited path.

**Rationale.** Prism's ARO (Observe-Recommend-Act) loop in C15 gives AI agents
the ability to propose and execute security actions. An ungated action from an
AI agent in a security environment is a trust boundary violation — it is the
prompt-injection attack surface materialized. The idempotency key requirement
prevents the "duplicate action on retry" class of incidents that have caused
real-world security outages.

**Source decisions.**
- C15 ARO Loop architecture (referenced in C12 cross-links, ARO model).
- `ADR-PROP-s3-agent-runtime.md` autonomy levels + gating.
- CISA AI safety framing: "system designers set the gates, not the agent."
- `ADR-PROP-web-stack.md` §security: signed tenant tokens, CSRF, session expiry.

---

### P-ADS-11 — Single-Codebase / Deployment-Profile

**Statement.** One Rust codebase. Behavior varies ONLY by deployment profile
(the intersection of operator-role × hosting dimensions: SaaS, MSSP-managed,
BYOC, air-gap, OT). No per-deployment-model code forks. Feature flags and
profile configuration govern which capabilities activate; the code is the same.

**Rationale.** N deployment codebases multiply maintenance cost, test surface,
and security patch burden by N. The single-codebase principle ensures that a
security fix applied once is applied everywhere. Deployment-profile-gated
behavior is visible in code review; silently diverged codebases are not.

**Source decisions.**
- `ADR-PROP-dual-deployment.md`: BYOC, MSSP-managed, and client-managed are
  deployment profiles of a single product, not separate products.
- SS-26 HD-1 SoftwareKms: same CMEK infrastructure works air-gap and SaaS.
- S3 `s3_agent_runtime_enabled=false` default: opt-in flag, not a fork.
- CLAUDE.md Canonical Principle §1: no MVP-driven deferrals or "for now" forks.

---

### P-ADS-12 — Production-Grade-Default

**Statement.** Default behavior is enterprise/production-grade correctness.
Feature ORDER is the only speed lever — not feature completeness. No MVP
deferrals, no "we can fix later," no partial implementations that need cleanup.
Each shipped feature is production-grade on the cycle it ships.

**Rationale.** Cross-reference CLAUDE.md §Canonical Principle (authoritative
source). Stated here as a first-class architecture principle because architectural
shortcuts (partial purity boundaries, placeholder DI, ungated action paths)
compound differently than code shortcuts — they create structural debt that
affects all features built on top of them.

**Source decisions.**
- CLAUDE.md §Canonical Principle (binding, supersedes this document on conflict).
- ADR-022 Arc-DI wiring contract: no placeholder-construct in the boot path.
- Standing Rule 3 §4: adding Arc<dyn Foo> to a constructor is "wiring not
  redesign" and must be done in-scope.

---

## Section B — Patterns

Each pattern carries a stable ID, when-to-use guidance, the originating feature,
and the structural form. Patterns are the architecture components: structural
solutions that recur across features. Using a named pattern instead of
re-inventing the solution prevents drift between features that solve the same
problem differently.

---

### PAT-ADS-01 — Feed-Down / Match-At-Edge

**When to use.** The product has a corpus of reference data (CVE database, threat
intel, compliance benchmarks) that must be correlated against edge-local asset
or sensor data. The asset data must not leave the edge (P-ADS-03); the corpus
can be distributed.

**Structure.**
1. Central aggregates and signs the reference corpus (e.g., CVE/CPE/KEV/EPSS/VEX
   feed bundle).
2. Central delivers the corpus as a signed offline bundle to each satellite
   (see PAT-ADS-07 for the bundle delivery mechanism).
3. The satellite maintains a local corpus store (RocksDB CF or embedded index).
4. Matching runs locally at the satellite against local asset inventory — no
   asset identifiers transit to Central.
5. Derived advisory results (not raw asset identifiers) surface at Central via
   Entity 360 queries.

**Originating feature.** C11 Prism Intel (`ADR-PROP-prism-intel.md`), D-C11-1.

**Exception.** D-C11-2 opt-in central-match mode is a narrowly consented
exception to the match-at-edge default. It requires OQ-C11-3 consent governance
to be closed before deployment. PIV-C11-001 + PIV-C11-004 bound the exception.
Any new feature that wants a similar "opt-in centralization" pattern must close
equivalent consent governance before story decomposition.

---

### PAT-ADS-02 — Tenant-Keyed-Central-Cache

**When to use.** A derived result computed at the edge needs to surface at
Central and persist beyond a single analyst session (dashboard aggregates,
investigation snapshots, conversation history, anomaly trend data).

**Structure.**
1. Derived result transits from satellite to Central (OCSF-normalized, never raw).
2. Central encrypts the result under the tenant's DEK from SS-26 per-tenant key
   hierarchy before writing to the cache store.
3. Cache store is partitioned by OrgId (per-tenant isolation, P-ADS-06).
4. Retrieval decrypts under the tenant DEK; operator infrastructure sees only
   ciphertext at rest (P-ADS-02).
5. Cache schema specifies: retention policy, key custody model, staleness TTL.

**Originating feature.** S3 conversation store (OD-1 resolution 2026-06-27);
generalized to all Central-cached derived results by Option 3 lock.

**Cache-vs-snapshot distinction.** This pattern stores OUTPUTS of queries, not
INPUTS. It is NOT a substitute for the deferred Iceberg data-snapshot mechanism
(OQ-C8-DATASNAPSHOT). Forensic replay of the same query against the same input
data requires input-level snapshots, not output caches. ADRs that cite this
pattern must state explicitly which of the two they are implementing.

---

### PAT-ADS-03 — Signed-Offline-Bundle

**When to use.** Content produced at Central must be delivered to satellites
that may be air-gapped, have intermittent connectivity, or operate in OT network
segments with strict ingress controls.

**Structure.**
1. Central produces the bundle (corpus, config update, model weights, schema
   update) and signs it with Ed25519 (sigstore attestation optional for higher
   assurance).
2. The bundle is content-addressed; the receiver verifies the signature before
   applying any content.
3. For air-gap delivery: bundle is exported to portable medium; no internet path
   required for the receiver.
4. For connected delivery: satellite pulls on a configurable dial-home interval;
   the bundle is idempotent to apply.
5. Satellite bootstrap (join-token OOB + optional TPM attestation) uses the same
   signature infrastructure.

**Originating feature.** C11 Prism Intel feed delivery (reused by C9 config
push, D-C2 satellite bootstrap in `ADR-PROP-satellite-mesh.md`).

---

### PAT-ADS-04 — Cost-Based-Degrade-Guard

**When to use.** A query operation has variable cost depending on source
capabilities or data volume, and the right behavior for an over-cost query is
degraded-but-honest execution rather than hard rejection.

**Structure.**
1. The capability descriptor (C3) carries per-source cost metadata: supports
   predicate, join type, time-bound requirements.
2. At plan time, Central evaluates the query cost against the descriptor.
3. If cost exceeds soft threshold: degrade the query (add time-bound injection,
   reduce join fanout, add coverage map disclosure) and continue.
4. Hard rejection is reserved for RESIDENCY violations (D-C5-3), not cost
   violations. Cost is a degrade surface; residency is a reject surface.
5. Disclosure: the analyst always sees a coverage map showing which sources were
   degrade-guarded and what the coverage gap is.

**Originating feature.** C3 capability descriptor (D-C3-1, `ADR-PROP-capability-
descriptor-pushdown.md`), superseding the earlier hard-reject framing in §5.3
and §12.2.

**Asymmetry note.** This asymmetry with residency (degrade vs reject) is
intentional. An implementer who applies degrade logic to residency violations,
or hard-reject to cost violations, is violating both this pattern and D-C5-3.

---

### PAT-ADS-05 — Pluggable-AI-Opaque-ModelBackend

**When to use.** A feature requires AI/ML inference that must support multiple
inference engines (for different hardware, deployment profiles, or air-gap
constraints) while maintaining the AI-opaque invariant (P-ADS-07).

**Structure.**
1. Define a `ModelBackend` trait with a single inference method taking a feature
   vector, returning a scored result.
2. Implementations: candle (pure Rust, air-gap-safe), ort/ONNX Runtime
   (acceleration via CUDA/CoreML when available), wasmtime (sandboxed WASM
   model), tract (pure-Rust ONNX inference, zero-C dependency).
3. The backend receives FEATURE VECTORS only — never raw credentials, raw OCSF
   records, or PII-containing fields. Masking (C16) and feature extraction
   happen before the backend call.
4. Backend selection is deployment-profile-driven (no code forks, P-ADS-11).
5. The `ModelBackend` trait is `#[non_exhaustive]` compatible so new backends
   can be added without breaking existing match arms.

**Originating feature.** C7 ML/Behavior Analytics Phase 3 (`ADR-PROP-ml-
behavior-analytics-depth.md`); resolved via C13 OD-3 reconciliation.

---

### PAT-ADS-06 — Hub-and-Spoke-Forward-Migration

**When to use.** A versioned schema (config, API, OCSF shape) must evolve over
time while supporting satellite fleets that may lag several versions behind
Central's current version.

**Structure.**
1. Central is the schema authority (hub). Satellites are version followers (spokes).
2. Forward migration only: satellites always upgrade toward Central's version,
   never the reverse.
3. Bounded skip-version window: satellites may skip non-LTS versions but must
   pass through LTS required-stops before upgrading past them.
4. During the skip window, Central must speak both the satellite's current version
   and its own version (dual-format fanout or version-translated delivery).
5. The A/B dual-slot bootstrap (PAT-ADS-09) enables the satellite to carry two
   active schema versions during the transition window.

**Originating feature.** C9 config management schema versioning
(`ADR-PROP-config-management.md`, D-C9-Q3-MODEL Hybrid + Hub-and-Spoke).

---

### PAT-ADS-07 — Two-Layer-Embedded-KG+Vector

**When to use.** A feature requires rich entity relationship traversal AND
semantic similarity search over the same corpus, embedded in a single binary
without external service dependencies.

**Structure.**
1. Layer 1 — Graph (hot): indradb (RocksDB-backed) for structured relationship
   traversal, community detection, identity resolution. Strictly per-tenant
   (`PIV-C12-5`).
2. Layer 2 — Vector (hot/cold): usearch (in-memory ANN) for hot similarity search;
   lancedb (on-disk) for cold vector storage. Same per-tenant partitioning.
3. Embedding runs on-box via fastembed/ort — raw telemetry never transits the
   network for embedding (PIV-C12-2). Embedding respects P-ADS-07 AI-Opaque.
4. Entity resolution is deterministic-first: strong identifiers (MAC, hostname,
   SPIFFE SVID) trigger auto-merge; weak identifiers produce suspected-link edges
   with explicit confidence score.
5. LLM-surfaced claims carry mandatory OCSF-event/rule/asset citations (see
   PAT-ADS-08).

**Originating feature.** C12 Prism Context (`ADR-PROP-prism-context.md`).

---

### PAT-ADS-08 — Mandatory-Faithful-Citations

**When to use.** Any AI-generated text (GraphRAG summaries, S3 agent responses,
C12 advisory overlays, C15 recommendations) is surfaced to an analyst in the UI.

**Structure.**
1. Every factual claim in AI-generated output carries an explicit citation to the
   source OCSF event, detection rule, or asset record that supports it.
2. Faithfulness must be ENFORCED and MEASURED, not assumed. Research finding:
   RAG systems can produce unfaithful citations at ~57% rates without explicit
   faithfulness enforcement (ARO research basis).
3. The Output Hardener component (S3 agent runtime) validates citations before
   surfacing them to the analyst. An uncited claim must either be cited or
   removed.
4. Metrics: faithfulness score tracked per-session and alerted if it falls below
   the configured threshold.

**Originating feature.** C12 Prism Context (GraphRAG faithfulness requirement);
generalized to S3 agent runtime Output Hardener design.

**Why this is a pattern, not just a guideline.** RAG faithfulness failures in a
security product are misattributed threat intelligence. An analyst who acts on a
hallucinated CVE citation could apply the wrong remediation to the wrong asset.
This is not a UX quality issue — it is a correctness invariant.

---

### PAT-ADS-09 — A/B-Dual-Slot-Bootstrap + Supervisor-Watchdog

**When to use.** A satellite must receive and apply a software or config update
with automatic rollback if the new version fails health checks.

**Structure.**
1. Satellite maintains two storage slots: ACTIVE and STANDBY.
2. Central delivers the new version (signed bundle, PAT-ADS-03) to STANDBY.
3. A supervisor process switches from ACTIVE to STANDBY atomically (file-rename
   or symlink swap).
4. Health checks run post-switch against the new version under a configurable
   watchdog timer.
5. On health failure: supervisor switches back to the ACTIVE slot automatically.
   Central is notified of the rollback event.
6. Old ACTIVE slot is garbage-collected only after the new version has been
   stable for the configured stabilization window.

**Originating feature.** C9 config management bootstrap + D-C6-3 auto-rollback
for detection rules (`ADR-PROP-detection-engine-depth.md`, `ADR-PROP-config-
management.md` D-C9-BOOTSTRAP).

---

### PAT-ADS-10 — Two-Tier-Canary-Apply

**When to use.** A config or schema change has HIGH-BLAST potential: it affects
all tenants, all satellites, or detection coverage in ways that cannot be
trivially reversed.

**Structure.**
1. Classify the change: HIGH-BLAST (canary required) or LOW-BLAST (direct apply
   permitted).
2. HIGH-BLAST changes deploy to a canary cohort first: either N% of satellites
   (for satellite-behavior changes) or a Central staging-tenant (for
   control-plane changes). Clarification required at morph time: the C9
   ADR-PROP is currently ambiguous on whether canary is satellite-percentage or
   staging-tenant (ripple audit Section 4 note; must be resolved before
   E-CONFIG-MGMT-001 story decomposition).
3. Soft trip signals (latency regression, error-rate spike) and hard signals
   (health check failure, assertion failure) are monitored during the canary
   window.
4. Full rollout proceeds only if both soft and hard signals are clean for the
   configured canary duration.
5. Fast revert (D-C9-FAST-REVERT) is always available regardless of blast tier.

**Originating feature.** C9 config management D-C9-Q2-TIERS and D-C9-Q2-HEALTH.

---

### PAT-ADS-11 — ARO-Loop (Observe-Recommend-Act)

**When to use.** A feature involves AI-generated recommendations that may result
in autonomous or semi-autonomous security actions.

**Structure.**
1. Observe: the system collects structured evidence (OCSF events, detection
   findings, Entity 360 state) into a context window.
2. Recommend: the AI agent (S3 runtime, C15 component) generates a
   recommendation with explicit evidence citations (PAT-ADS-08) and a confidence
   estimate.
3. Act: the recommended action is submitted to the gated action path (P-ADS-10
   Idempotent-Gated-Actions). The RBAC + autonomy-level gate fires before any
   action executes.
4. The autonomy level is a system-configured parameter, not an agent-configured
   parameter. The agent cannot elevate its own autonomy level.

**Originating feature.** C15 ARO architecture (referenced in `ADR-PROP-prism-
context.md` cross-links and S3 agent runtime autonomy-level design).

---

## Section C — Invariants and Conformance

### C.1 — Cross-Cutting Invariants (INV-ADS-NNN)

The following invariants are elevated from the per-feature PIV-* entries in
individual ADR-PROPs to cross-cutting invariants that apply to ALL features.
Any new ADR-PROP or epic that violates an INV-ADS-NNN is non-conforming.

| ID | Invariant | Strongest Enforcement | ADR-PROP Source |
|----|-----------|----------------------|-----------------|
| INV-ADS-01 | Raw sensor data NEVER persists at Central, in transit to Central, or in AI context. | D-C2-12 HARD INVARIANT | `ADR-PROP-satellite-mesh.md` |
| INV-ADS-02 | Operator has ZERO at-rest read access to client data (raw or derived) persisted at Central. | SS-26 per-tenant DEK; Option 3 lock. | `secret-subsystem-sketch.md` |
| INV-ADS-03 | Per-tenant partitioning is strict everywhere: graph, vector, entity-resolution, keys, config, cache. No cross-tenant edges or similarity. | PIV-C12-5 | `ADR-PROP-prism-context.md` |
| INV-ADS-04 | Config is authored ONLY at the Central UI/DB. No satellite, CLI file, or git-committed TOML authors production config. | PIV-C9-001, D-C9-Q1-AUTHORITY | `ADR-PROP-config-management.md` |
| INV-ADS-05 | All writes and actions are gated (RBAC + configurable autonomy level) and carry mandatory idempotency keys. | S3 autonomy-level architecture | `ADR-PROP-s3-agent-runtime.md` |
| INV-ADS-06 | Credentials and raw sensitive data NEVER transit AI context. AI backends receive feature vectors or masked representations only. | AD-017 | `CLAUDE.md` + `ADR-PROP-ml-behavior-analytics-depth.md` |
| INV-ADS-07 | OCSF normalization applies at ALL adapter boundaries. No trusted-source exemption. | D-C4-1 (no-exemption rule) | `ADR-PROP-dynamic-schema-connectors.md` |
| INV-ADS-08 | Embedded air-gap deployment is the default reference target. No feature may require internet connectivity for correct operation in the air-gap profile. | SS-26 SoftwareKms HD-1 | `secret-subsystem-sketch.md` |

### C.2 — Conformance Checklist

Every new ADR-PROP and every new epic definition MUST pass this checklist before
being promoted from CAPTURE to a live architecture artifact. The ripple audit
(`research/central-surfacing-ripple-analysis-2026-06-27.md`) was the FIRST
conformance run against the Day-2 corpus; it found six non-conformances.
Subsequent conformance passes will patch the identified ADR-PROPs.

**Instructions.** For each item, answer YES or NO. Any NO is a conformance
failure — do not promote until resolved.

```
CONFORMANCE CHECKLIST — [Feature/ADR name] — [Date]

P-ADS-01: Central-Sole-Surface
  [ ] Does every user-interaction path in this feature terminate at Central?
  [ ] If the feature involves a satellite, is the satellite strictly headless
      (data-plane only, no user login surface)?

P-ADS-02: Operator-Zero-Access-At-Rest
  [ ] Is every derived result persisted at Central encrypted under a tenant-held
      CMEK key (SS-26 per-tenant DEK)?
  [ ] Does the operator have zero read access to the persisted data at rest?

P-ADS-03: Derived-Results-Only-At-Central
  [ ] Does the feature transit only DERIVED results (never raw sensor data,
      raw asset identifiers, or credentials) from edge to Central?
  [ ] If the feature involves an opt-in path where identifiers may transit,
      is consent governance explicitly closed (equivalent to PIV-C11-004)?

P-ADS-04: Tenant-Keyed-Central-Persistence
  [ ] Does the feature use RocksDB (hot) or Iceberg (cold) — NOT PostgreSQL —
      for any Central-side query result cache?
  [ ] If the feature discusses "forensic replay," does it explicitly distinguish
      output caching from input-level Iceberg data-snapshots (OQ-C8-DATASNAPSHOT)?

P-ADS-06: Per-Tenant-Isolation
  [ ] Is all new state (graph edges, vector indices, cache entries, config) per-
      tenant partitioned? Is there any code path that produces cross-tenant joins?

P-ADS-07: AI-Opaque
  [ ] Do all AI/ML components receive feature vectors or masked data only?
  [ ] Are credentials resolved OUTSIDE the AI reasoning loop?

P-ADS-08: OCSF-Normalize-At-Boundary
  [ ] Does every new data source normalize to OCSF at the adapter boundary?
  [ ] If the source has known OCSF version skew, is the version axis declared
      in the capability descriptor?

P-ADS-09: Config-DB-Authoritative
  [ ] Does the feature have any config-authoring path at the satellite or edge?
      (If YES: non-conforming. No exceptions without a new invariant amendment.)

P-ADS-10: Idempotent-Gated-Actions
  [ ] Do all write/action paths carry idempotency keys?
  [ ] Is the autonomy gate system-configured, not agent-configured?

INV-ADS check (all eight):
  [ ] INV-ADS-01: No raw sensor data at Central
  [ ] INV-ADS-02: Operator zero-access at rest
  [ ] INV-ADS-03: Per-tenant isolation enforced
  [ ] INV-ADS-04: Config authored only at Central
  [ ] INV-ADS-05: Actions gated and idempotent
  [ ] INV-ADS-06: AI-opaque
  [ ] INV-ADS-07: OCSF normalization at all boundaries
  [ ] INV-ADS-08: Air-gap deployment is valid reference profile
```

### C.3 — Known Conformance Gaps (from Ripple Audit)

The following ADR-PROPs have confirmed non-conformances that must be resolved
during the morph execution cycle. These are not new findings — they are
already documented in the ripple audit. They are listed here so that the
conformance checklist has a clear punch list.

| Priority | ADR-PROP | Non-Conformance | Checklist Item |
|----------|----------|-----------------|----------------|
| P0 | `ADR-PROP-s3-agent-runtime.md` | Three stale passages in Deployment Gating + Consequences + Alternatives §D still describe browser-local-only model (superseded by OD-1 resolution). | P-ADS-02, P-ADS-04 |
| P0 | `ADR-PROP-dual-deployment.md` | OQ-DEPLOY-2 four gaps treated as a uniform block; must be disaggregated. Gap (d) CMEK pre-launch required. | P-ADS-02, INV-ADS-02 |
| P0 | `ADR-PROP-prism-intel.md` | PIV-C11-001 text does not acknowledge the consented exception in D-C11-2. Must be amended before E-INTEL-FEED-001. | P-ADS-03, INV-ADS-01 |
| P1 | `ADR-PROP-central-deployment-access-layer.md` | No explicit prohibition of PostgreSQL for query result caching. | P-ADS-04, INV-ADS-01 |
| P1 | `ADR-PROP-prismql-deliverables.md` | Missing note: Central CMEK cache is NOT a substitute for OQ-C8-DATASNAPSHOT. | P-ADS-04 |
| P1 | `ADR-PROP-satellite-mesh.md` | D-C12 note missing: OCSF normalization governs format, not PII content; OQ-DEPLOY-2(a) applies. | P-ADS-03, INV-ADS-07 |
| P2 | All Option 3-relevant ADR-PROPs | Missing "Central Cache" section per affected ADR-PROP with cache schema, per-tenant DEK, retention, key custody. | P-ADS-04 |
| P2 | `ADR-PROP-config-management.md` | Canary cohort unit ambiguity (satellite-% vs staging-tenant). Must close before E-CONFIG-MGMT-001. | PAT-ADS-10 |

---

## Section D — Anti-Patterns

The following patterns are FORBIDDEN. Each entry names the principle it
violates so that the conformance checklist can be applied precisely.

---

### AP-ADS-01 — Raw Data or Credentials at Central

**Forbidden.** Designing a code path that delivers raw sensor API responses,
raw asset inventories, or credential values to Central — whether at rest, in
transit through Central, or in AI context at Central.

**Violates.** P-ADS-03, INV-ADS-01, INV-ADS-06.

**Common form.** "We'll send the raw API response to Central and normalize there
— it's simpler." (Normalization at Central creates a raw-data-at-Central path
that breaks BYOC, OT, and air-gap deployments. Normalize at the boundary per
P-ADS-08.)

---

### AP-ADS-02 — Satellite as User-Login Surface

**Forbidden.** Designing a satellite that presents a login UI, analyst
dashboard, or interactive query surface to end users. The narrow exception
(initial setup, break-glass maintenance) must be explicitly scoped and must NOT
include query authoring, results viewing, or alert triage.

**Violates.** P-ADS-01, INV-ADS-04 (config authoring is the most common form).

**Common form.** "The OT satellite has its own dashboard for plant operators —
they're not on the Central network." (Plant operator access must be federated
through Central, possibly via a network-accessible relay; the satellite itself
remains headless.)

---

### AP-ADS-03 — Operator-Readable-At-Rest

**Forbidden.** Persisting client data at Central in a form that the
operator/vendor can decrypt with operator-controlled keys.

**Violates.** P-ADS-02, INV-ADS-02.

**Common form.** "We'll encrypt the result cache with the operator's database
key — that's standard practice." (The CMEK requirement means the tenant, not
the operator, holds the decryption key. Operator-controlled encryption does not
satisfy Option 3.)

---

### AP-ADS-04 — Config Authored at Edge

**Forbidden.** Any code path by which a satellite, CLI-operated file edit, or
git-committed TOML creates or modifies production configuration outside the
Central UI/DB flow.

**Violates.** P-ADS-09, INV-ADS-04.

**Common form.** "For air-gap deployments, operators will edit the TOML directly
on the satellite since they can't reach Central." (Air-gap delivery of signed
bundles, PAT-ADS-03, is the correct pattern. Direct TOML editing at the satellite
is the config-at-edge anti-pattern even in air-gap scenarios.)

---

### AP-ADS-05 — Cross-Tenant Edges or Similarity

**Forbidden.** Any graph edge, vector similarity score, or entity-resolution
result that spans two tenants' data — even in aggregate or anonymized form.

**Violates.** P-ADS-06, INV-ADS-03.

**Common form.** "We'll use cross-tenant threat intel correlation to improve
detection for all tenants — each tenant only sees their own results." (The
correlation computation itself creates cross-tenant data access, regardless of
what each tenant sees. Per-tenant isolation must hold at the computation layer,
not just the result layer.)

---

### AP-ADS-06 — Ungated or Non-Idempotent Auto-Actions

**Forbidden.** Any AI-generated or automation-triggered action that executes
without passing through the RBAC + autonomy-level gate, or that does not carry
an idempotency key.

**Violates.** P-ADS-10, INV-ADS-05.

**Common form.** "The remediation is low-risk — we'll just auto-apply it without
a confirmation step." (The agent does not determine what is low-risk. The system
designer, via the autonomy level configuration, makes that determination.
Agents propose; gates authorize.)

---

### AP-ADS-07 — Per-Deployment-Model Code Forks

**Forbidden.** Creating separate code paths, branches, or crates that implement
the same feature differently for different deployment profiles. Feature flags
and deployment-profile configuration are the correct mechanism.

**Violates.** P-ADS-11.

**Common form.** "We'll add a `#[cfg(feature = "saas")]` block here since the
SaaS version needs a different code path." (Profile-conditional behavior must
be expressed as configuration, not compile-time feature flags that create
divergent builds. The single-codebase invariant means one binary, not one
binary per deployment model.)

---

### AP-ADS-08 — Unverified Citations Surfaced to Users

**Forbidden.** Surfacing AI-generated text that contains factual claims without
verified citations to source OCSF events, detection rules, or asset records.

**Violates.** PAT-ADS-08, P-ADS-07 (AI-opaque is the complementary bound).

**Common form.** "The LLM response looks correct — we'll display it as-is."
(RAG systems can produce unfaithful citations at ~57% rates. In a security
product, a hallucinated CVE citation can cause misapplied remediation.
Every claim must pass Output Hardener validation before display.)

---

### AP-ADS-09 — Placeholder-Construct DI

**Forbidden.** Constructing a type in the production boot path without wiring
real `Arc<dyn Trait>` dependencies — using a "placeholder" or "stub" value
that does not implement real behavior.

**Violates.** P-ADS-12, ADR-022 Arc-DI wiring contract.

**Common form.** "I'll wire the real dependency later — for now I'll use
`SomeThing::placeholder()`." (This is Standing Rule 3 §4: adding Arc<dyn Foo>
to a constructor is "wiring not redesign" and must be done in-scope, not deferred.
See CLAUDE.md §Canonical Principle, §Conventions §Arc-DI plumbing.)

---

### AP-ADS-10 — MVP Deferral

**Forbidden.** Rationalizing an incomplete or incorrectly-implemented feature
with "MVP," "for now," "good enough," or "we can fix later."

**Violates.** P-ADS-12.

**Common form.** "This is good enough for the Day-2 launch — we'll harden it
in Day-3." (Feature ORDER is the speed lever. The current feature must be
production-grade on the cycle it ships. Defer the entire feature to Day-3 if
necessary; do not ship it incomplete.)

---

## Section E — Cross-References

### Ripple Audit (Seeding Document)

`research/central-surfacing-ripple-analysis-2026-06-27.md` — the first
systematic conformance run against the Day-2 ADR-PROP corpus. Sections:
- Section 1: Per-feature ripple matrix (all 21 features against both constraints)
- Section 2: Six conflicts (CONFLICT-1 through CONFLICT-7, ranked by severity)
- Section 3: Surfacing-burden analysis (Tier A/B/C features)
- Section 4: C9 config-authoring note
- Section 5: Option 1 vs Option 3 recommendation
- Section 6: ADR-PROP punch list for morph execution

### ADR-PROP Traceability

| ADR-PROP | Principles | Patterns | Invariants |
|----------|-----------|---------|------------|
| `ADR-PROP-central-deployment-access-layer.md` | P-ADS-01, P-ADS-05 | PAT-ADS-02 | INV-ADS-01..05 |
| `ADR-PROP-satellite-mesh.md` | P-ADS-01, P-ADS-03, P-ADS-05 | PAT-ADS-03 | INV-ADS-01, INV-ADS-07, INV-ADS-08 |
| `ADR-PROP-capability-descriptor-pushdown.md` | P-ADS-05, P-ADS-08 | PAT-ADS-04 | INV-ADS-07 |
| `ADR-PROP-dynamic-schema-connectors.md` | P-ADS-08 | PAT-ADS-04 | INV-ADS-07 |
| `ADR-PROP-siem-lake-federation.md` | P-ADS-03, P-ADS-08 | PAT-ADS-04 | INV-ADS-01, INV-ADS-07 |
| `ADR-PROP-detection-engine-depth.md` | P-ADS-01, P-ADS-09 | PAT-ADS-09, PAT-ADS-10 | INV-ADS-04 |
| `ADR-PROP-ml-behavior-analytics-depth.md` | P-ADS-05, P-ADS-07 | PAT-ADS-05 | INV-ADS-06 |
| `ADR-PROP-prismql-deliverables.md` | P-ADS-01, P-ADS-04 | PAT-ADS-02 | INV-ADS-01 |
| `ADR-PROP-config-management.md` | P-ADS-09 | PAT-ADS-06, PAT-ADS-10 | INV-ADS-04, INV-ADS-08 |
| `ADR-PROP-prism-intel.md` | P-ADS-03 | PAT-ADS-01, PAT-ADS-03 | INV-ADS-01 |
| `ADR-PROP-prism-context.md` | P-ADS-05, P-ADS-06, P-ADS-07 | PAT-ADS-07, PAT-ADS-08, PAT-ADS-11 | INV-ADS-03, INV-ADS-06 |
| `ADR-PROP-dual-deployment.md` | P-ADS-11 | PAT-ADS-02, PAT-ADS-03 | INV-ADS-02, INV-ADS-08 |
| `ADR-PROP-s3-agent-runtime.md` | P-ADS-02, P-ADS-07, P-ADS-10 | PAT-ADS-02, PAT-ADS-08, PAT-ADS-11 | INV-ADS-02, INV-ADS-05, INV-ADS-06 |
| `secret-subsystem-sketch.md` (SS-26) | P-ADS-02, P-ADS-04 | PAT-ADS-02 | INV-ADS-02 |
| `ADR-PROP-sso-identity.md` | P-ADS-01, P-ADS-06 | — | INV-ADS-03 |
| `ADR-PROP-storage-engine-taxonomy.md` | P-ADS-04 | PAT-ADS-02 | INV-ADS-01 |
| `ADR-PROP-web-stack.md` | P-ADS-01, P-ADS-10 | — | INV-ADS-05 |
| `ADR-PROP-sandboxed-expression-evaluator.md` | P-ADS-01 | — | — |
| `ADR-PROP-widget-dsl-render-and-schema-validation.md` | P-ADS-01 | — | — |
| `prismql-sequence-sugar-decisions.md` | P-ADS-01, P-ADS-09 | — | INV-ADS-04 |
| `ml-depth-phasing.md` | P-ADS-05, P-ADS-07 | PAT-ADS-05 | INV-ADS-06 |

### CLAUDE.md Cross-References

The following CLAUDE.md §Conventions cross-cut from the code tier to the
architecture tier and must be consistent with this ADS:

| CLAUDE.md Convention | ADS Principle/Invariant |
|----------------------|------------------------|
| Arc-DI plumbing, no placeholder-construct | AP-ADS-09, P-ADS-12 |
| OrgSlug newtype + redacted Debug | P-ADS-06, INV-ADS-06 |
| HTTP timeout 30s on all clients | P-ADS-12 (production-grade default) |
| No unwrap/expect in critical paths | P-ADS-12 |
| OCSF normalization | P-ADS-08, INV-ADS-07 |
| Structured event catalog (BC-2.16.002) | P-ADS-10 (gated/audited paths) |
| #[non_exhaustive] on public types | P-ADS-11 (single-codebase, no forks at type level) |
| Credentials never transit AI context (AD-017) | P-ADS-07, INV-ADS-06 |

---

*End of Prism Architecture Design System v1.0 — 2026-06-27*
