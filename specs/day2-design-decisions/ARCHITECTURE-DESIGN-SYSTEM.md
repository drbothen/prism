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
- `ADR-PROP-nested-tenancy.md` §4 (MSSP reconciliation; authorized mediated access
  clarification; operator-zero-access spectrum across three operating models).

**Boundary.** This principle governs at-rest data, not in-transit data. In-transit
residency (OQ-DEPLOY-2 gap a) is a separate concern. This principle also does NOT
apply to control-plane data that is not client-data: RBAC role definitions, IdP
config, audit log metadata — these have their own operator-access governance.

**Clarification (C19, 2026-06-27).** The guarantee is about **UNMEDIATED at-rest
access with operator-controlled keys**. It is NOT violated by authorized, RBAC-scoped,
audited, consented access by a managing party (an MSSP analyst or a parent tenant
with proper visibility grants) to the DERIVED corpus, provided that access travels
through a **mediated tenant-key path** — meaning: the client's own DEK decrypts (not
an operator-held key), the access is Central-session-authenticated (P-ADS-01), RBAC-
scoped to delegated clients, and fully attributed in the audit trail.

Key distinctions for the MSSP operating model:

- **What P-ADS-02 blocks:** rogue infra access; raw disk/backup access; subpoena-to-
  vendor (operator served but does not hold the key); access by non-onboarded parties.
- **What P-ADS-02 does NOT block:** authorized analyst access to delegated clients
  via an authenticated Central session where the CLIENT's DEK decrypts. The managed-
  service onboarding IS the consent. This is the MSSP value proposition — governed,
  audited, revocable — not a grudging exception to the principle.
- **MSSP-managed standard posture:** P2 Operational visibility (derived_rows +
  findings_alerts + config) via per-client delegation with full trust controls (pairing,
  persistent child-side indicator, granular revocation, child-side audit trail). This
  is the core MSSP operating mode; it differs from P3 transparent-subtree in legal
  basis and consent ceremony, not in raw capability.

Operator-zero-access is a **spectrum** across the three operating models:

| Operating model | Zero-access character |
|---|---|
| Client-managed / BYOC | Strongest — client holds keys; operator CANNOT decrypt |
| SaaS | Vendor sees ciphertext at rest; tenant-keyed CMEK means vendor cannot decrypt without tenant key |
| MSSP-managed | Key separation (per-client DEK/CMEK) + audited mediated access + per-client key isolation. Not cryptographic impossibility — cryptographic proof that access is authorized, attributable, and revocable |

The MSSP default key custody is CLIENT-HELD CMEK (SS-26 CMEK/HYOK) by default;
MSSP-custodied SoftwareKms is an opt-down. This design maximizes the zero-access
differentiator even in the MSSP-operated model.

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
edges, similarity scores, or data joins. Nested tenancy (C19) adds hierarchy to
the OrgId abstraction but does not weaken per-tenant isolation — each node in
the hierarchy retains its own DEK, and cross-tenant visibility requires explicit
consent via the visibility-grant matrix (see `ADR-PROP-nested-tenancy.md` §3).

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
- `ADR-PROP-nested-tenancy.md` PIV-C19-1..6 (isolation invariants under nesting).

**Key-plane enforcement:** AP-ADS-11 (Cross-Tenant DEK Grantee) is the specific
anti-pattern that violates this principle at the cryptographic key layer — adding
another tenant's principal as a persistent grantee on a tenant's DEK is the key-
plane form of cross-tenant isolation violation. See AP-ADS-11 for the full analysis
and the three correct alternatives.

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
- `ADR-PROP-entity-masking.md` — the concrete enforcement mechanism for this
  principle applied to DATA (see Clarification below).

**Clarification (C16, 2026-06-27).** The enforcement mechanism for AI-opaque DATA
is the **C16 edge tokenizing clearing house** (`ADR-PROP-entity-masking.md`). The
clearing house masks/tokenizes regulated sensitive fields immediately after OCSF
normalization at the edge, before any transit to Central or the AI path. The
agent/ModelBackend path is structurally unwired from the token vault — no route,
no credential, no role grant for detokenization. This is the data analogue of the
AD-017 credential broker.

Additionally, **embeddings and vector stores are a sensitive-data class** under this
principle. Embedding-inversion attacks reconstruct 50–90%+ of source text (including
names and identifiers) from embedding vectors. Therefore:
- The agent receives ONLY masked-view embeddings (from the AI/RAG index in the
  dual-index pattern, PAT-ADS-14). Raw embeddings are embedded on-box at the edge
  (PIV-C12-2), inside the edge trust boundary, and never shipped to an external
  embedding service.
- Raw embeddings and the raw-text human-IR index reside in the secure zone; the
  agent has NO access to either.
- Vector stores carry the same custody classification as the token vault: DEK-
  protected (P-ADS-04), per-tenant isolated (P-ADS-06), RBAC-gated, audited.

This clarification does not loosen any existing constraint; it specifies the concrete
structural mechanism (clearing house + dual index) that fulfills INV-ADS-06 for data.

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

Composes with P-ADS-13 (Configurable-Not-Prescriptive): deployment-profile selects
which capabilities activate; P-ADS-13 governs that restrictive POSTURE within them
is configurable data, not hardcoded absolutes.

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

### P-ADS-13 — Configurable-Not-Prescriptive (Policy-as-Configuration)

**Statement.** Restrictive security/compliance posture is expressed as configurable
DATA — discrete settings plus named, shippable **Profiles** (e.g. baseline / SOC2 /
ISO27001 / IEC-62443-OT / NERC-CIP) — never as hardcoded absolutes and never as a
single vertical's needs branched into code. The axes this governs include:
staleness/restriction thresholds (e.g. deny-on-stale-beyond-N), action-gating
strictness, data masking / bulk-export strictness, approval & separation-of-duties
requirements, key-custody requirements (e.g. force client-held CMEK or
BYOC-remote-op), and electronic-security-perimeter rules. The product ships SAFE,
restrictive DEFAULTS and named compliance Profiles; clients tune only WITHIN
profile-permitted bounds. The product does not prescribe absolute behaviors a client
cannot adjust within governed bounds, and does not hardcode one vertical (OT) as a
code branch — OT is a shipped Profile, not a fork.

**Rationale.** A multi-vertical MSSP platform serves clients with divergent and
sometimes conflicting compliance regimes. Hardcoding one regime's restrictiveness
(the OT temptation) either over-constrains other clients or forces per-vertical code
forks (violating P-ADS-11). Expressing posture as configurable data + named Profiles
lets a single codebase serve every regime, makes conformance auditable ("this tenant
runs the NERC-CIP Profile; these settings drift"), and lets the restrictive settings
ride the same DB-authoritative, central-authored, signed-bundle distribution and
tighten-only tenant-tree inheritance that already govern config (C9) and nested
tenancy (C19).

**Floor (critical boundary).** Configurability sits ABOVE the INV-ADS invariant
floor. The hard invariants are NEVER configurable off: operator-zero-access-at-rest
(INV-ADS-02), no-raw-sensor-data-at-Central (INV-ADS-01), per-tenant isolation
(INV-ADS-03), AI-opaque credentials/data (INV-ADS-06), config-authored-only-at-
Central (INV-ADS-04). "Configurable" means tunable-within-governed-bounds and
tighten-only down the tenant tree — it does NOT mean an invariant can be defeated
by configuration. A Profile may only TIGHTEN relative to baseline; it can never
loosen below the invariant floor or below an ancestor tenant's guardrail (C19 hybrid
inheritance: guardrails intersect, parent-deny-is-final).

**Source decisions.**
- Recurring human directive across C9 (approval/review workflows deferred to Day-3
  as configurable, per-client), C18 SF-2 (masking/bulk-export configurable), C18
  SF-3 ("configurable, not just OT — OT is a profile we ship"), C18 SF-5 (workflows
  fully client-configurable, no prescribed absolutes).
- C19 `regulatory_class` tenant attribute (to be reframed as a Profile assignment
  when C18 lands) — `ADR-PROP-nested-tenancy.md`.
- CLAUDE.md project memory: built-in sensors are config-driven (TOML specs), full
  sensor API behind a robust feature-flag system — the same eat-our-own-config
  philosophy generalized to security posture.
- Composes with P-ADS-09 (Config-DB-Authoritative), P-ADS-11 (Single-Codebase/
  Deployment-Profile), P-ADS-12 (Production-Grade-Default: defaults are restrictive,
  configurability never lowers the bar below production-grade), and PAT-ADS-03
  (Signed-Offline-Bundle) / PAT-ADS-06 (Hub-and-Spoke-Forward-Migration) for
  distribution.

**Forward note (pending, do not pre-author).** The concrete structure that
operationalizes this principle — the **Configurable Compliance Profile pattern
(provisional PAT-ADS-12)** and its data model, composition/inheritance, and shipped
presets — is under design in C18 (research pass
`research/configurable-security-profiles-2026-06-27.md` in flight). This principle
states the RULE; the forthcoming pattern will state the STRUCTURE. Do not author the
pattern here.

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

### PAT-ADS-12 — Configurable Compliance Profile

**When to use.** A feature has a restrictive security/compliance posture that varies by
vertical or regulatory regime and must NOT be hardcoded. Examples: staleness-window
thresholds for OT/NERC-CIP action-gating, masking/bulk-export strictness, key-custody
requirements, audit retention floors, electronic-security-perimeter rules, and
workflow-approval requirements.

**Structure.**
1. Define the posture axes (staleness, masking, action-gating, key-custody, audit, etc.)
   as a NAMED, VERSIONED, SIGNED TOML/JSON document in the Config-DB (P-ADS-09). Each
   axis is either **hard-locked** (`{ lock = V }`) or **tunable-within-range**
   (`{ min, max }` / `{ allowed = [...] }`).
2. Five shipped presets form a monotone tighten-only inclusion chain:
   `baseline ⊂ soc2 ⊂ iso27001 ⊂ iec-62443-ot ⊂ nerc-cip`. Every preset is a
   strict superset of restrictions relative to the preceding one; no preset loosens
   any setting of its parent.
3. A Compliance Profile is **Central-authored only** (INV-ADS-04), resolved at Central
   plan-time by folding the C19 closure-table ancestor set (tighten-only fold),
   and distributed to satellites as a FLATTENED signed bundle (PAT-ADS-03). Satellites
   never compute profile inheritance — they receive and enforce the flattened result.
4. **Tighten-only monotone:** the Central authoring engine programmatically REJECTS any
   profile document that attempts to loosen a setting below its parent preset's lock or
   tunable range. Out-of-range authoring is rejected (not warned), exceeding the
   precedents (AWS/Azure/SCAP rely on human governance).
5. Ride PAT-ADS-10 Two-Tier-Canary-Apply for HIGH-BLAST profile tightening (e.g.,
   flipping a subtree from `soc2` to `nerc-cip` enforce mode). Deploy in `report-only`
   (audit/warn) mode first — see drift, remediate, then flip to `enforce`.
6. **Conformance reporting:** per-setting `compliant | drifting | exempt` status PLUS
   a boolean gate (any hard-locked-setting drift ⟹ node is non-compliant) PLUS an
   itemized drift list. NOT a single flat compliance percentage (which masks critical
   drift). A `[workflow_requirements]` directive the Day-3 engine has not yet implemented
   reports as `drifting`, never `compliant`.
7. **Two distinct named axes — never collapsed:** deployment-profile (P-ADS-11: WHERE/HOW
   Prism runs) and Compliance-Profile (HOW STRICT the posture is) share the signed-bundle
   + canary mechanism but remain orthogonal. A `saas` deployment can host a `nerc-cip`
   tenant; an `air-gap` deployment can run `baseline`. A non-satisfiable combination
   (compliance profile demands a capability the deployment profile cannot provide) is a
   conformance ERROR at authoring time, not a silent downgrade.
8. **Custom profiles:** MSSPs may author custom profiles that `extends` a shipped preset
   and add further tighten-only restrictions. Custom profiles undergo the same Central
   authoring-boundary validation as shipped presets.

**Originating feature.** `ADR-PROP-compliance-profiles.md`.

**Canonical expression.** This pattern is the concrete structural expression of P-ADS-13
(Configurable-Not-Prescriptive) / P-ADS-11 (Single-Codebase) / AP-ADS-07
(No-Deployment-Model-Code-Forks) for the OT/regulatory-restrictiveness concern. "OT is a
shipped Profile, not a fork" — this pattern is how that statement is operationalized.

---

### PAT-ADS-13 — Layered-Authz (RBAC + ReBAC + ABAC)

**When to use.** A feature needs fine-grained, hierarchy-aware, resource-scoped
authorization — where a flat single-paradigm model would either produce role explosion
(per-column RBAC roles) or brittle traversal (ABAC attribute-encoding of hierarchy).
The layered model is the standard for MSSP platforms serving regulated industries
(NERC CIP-004/005, SOC2 CC6.x, ISO 27001 Annex A.9).

**Structure.**
1. **RBAC tier** — Human-facing role surface (`analyst / lead / admin / approver`).
   Maps to MSSP org structure; satisfies SOC2 CC6.x / ISO A.9 / NERC CIP-004 auditor
   expectations. Role bindings take the form `(role, scope-node)` filtered by the C19
   closure table; a binding without a scope-node anchor is invalid.
2. **ReBAC tier** — Zanzibar-style relationship-tuple graph for the resource hierarchy
   (`tenant → connector → source → table → action-class`). Inheritance propagates
   **strictly downward** along edges; escalation-up is a tested authz-schema invariant.
   The C19 closure table is the materialized tenant-tier projection of the broader
   relationship graph.
3. **ABAC tier** — Sensitivity tags and dynamic conditions for column/field masking
   (`sensitivity:PII`) and row filters. Columns are NOT first-class role-scoped
   resources; they are governed by ABAC sensitivity tags at catalog level to avoid
   role explosion. One policy covers all `sensitivity:PII` columns.
4. **Enforcement point:** the PrismQL query planner is the Policy Enforcement Point
   (PEP). Every PrismQL query is decomposed into source/table/column accesses at
   plan time; each is checked against the active policy before fan-out.
5. **Central-authored / edge-enforced:** policy is authored ONLY at Central (P-ADS-09 /
   INV-ADS-04) and distributed to satellites as signed, versioned policy bundles
   (PAT-ADS-03). Each satellite runs a local PDP + PEP and enforces fully offline.
6. **Decision-level audit:** every authorization DECISION (subject, resource, action,
   attributes considered, policy bundle version, outcome) is logged — not merely the
   access event (INV-ADS-09). Offline satellites buffer and reconcile.
7. **Revocation-lag posture** is a Compliance Profile setting (PAT-ADS-12), not a
   hardcoded constant in the authz engine. OT-affecting actions' staleness behavior
   lives in `profile:iec-62443-ot` and `profile:nerc-cip`, not in a `#[cfg]` branch.

**Originating feature.** `ADR-PROP-rbac-depth.md`.

---

### PAT-ADS-14 — Edge-Tokenizing-Clearing-House

**When to use.** A feature must keep regulated or sensitive fields out of AI context AND out
of Central at-rest storage. The data sensitivity boundary must coincide structurally with the
edge-to-Central conduit boundary — policy alone is insufficient because policy can be
misconfigured or bypassed; structural absence of raw data at Central is the correct invariant.

**Structure.**
1. **Mask at the edge immediately after OCSF normalization,** before any transit to Central
   (PAT-ADS-03 conduit). The clearing house is a pipeline stage in the OCSF normalization
   flow, not a filter at the Central ingestion boundary.
2. **Technique mix keyed by RSI field class** (driven by the active Compliance Profile
   masking axis, D-PROF-3):
   - High-risk identifiers (IP, hostname, asset_id, BCSI configs) → deterministic vaulted
     tokenization (default; joins preserved; token mathematically unlinked to plaintext).
   - Fields requiring format-valid surrogates with domain ≥ 10^6 → FF1 FPE (optional, narrow).
   - Fields the agent never legitimately needs → full redaction (irreversible).
   - Free-text fields → NER span detection → tokenize/redact spans.
3. **Per-tenant token vault + DEK at or near the edge** (reuse SS-26 per-tenant DEK
   hierarchy; vault = a DEK-guarded RocksDB column family). Central holds token values
   (ciphertext references) + masked OCSF records; Central has NO DEK for the token vault.
4. **Agent path has ZERO vault wiring.** No route to the detokenize endpoint, no credential
   for the token CF, no role grant for reveal. This is a structural wiring absence (the data
   analogue of AD-017), not a policy restriction.
5. **DUAL INDEX for embeddings:**
   - Human-IR index (raw text + raw embeddings, inside the secure zone): agent has NO access;
     used for authorized human investigation and machine correlation.
   - AI/RAG index (masked view: deterministic tokens + contextual OCSF text): the only index
     the agent may query. Both indexes use on-box embedding (PIV-C12-2); raw text never
     transits to an external embedding service.
6. **Detokenize-at-surface only via C18 RBAC,** gated by analyst identity × tenant × token
   class × Compliance Profile masking posture. Raw values returned TRANSIENTLY to the analyst
   client session; NEVER re-persisted to Central. Every reveal is audited.

**Originating feature.** `ADR-PROP-entity-masking.md` (C16 entity masking / RSI clearing house).

**Composes with.**
- PAT-ADS-07 (Two-Layer-Embedded-KG+Vector): the dual-index structure in item 5 above extends
  PAT-ADS-07 with the masking-boundary enforcement. The AI/RAG index is the masked projection
  of the raw human-IR index.
- PAT-ADS-12 (Configurable Compliance Profile): the masking axis `[settings.masking]` in the
  Compliance Profile data model controls which RSI field classes map to which technique and
  whether bulk-export is permitted.
- P-ADS-07 / INV-ADS-06: this pattern IS the concrete structural enforcement of the AI-Opaque
  principle for sensitive DATA (as distinct from credentials, which are governed by AD-017).

---

### PAT-ADS-15 — Logical-Watermark Cross-Store Backup

**When to use.** A coherent point-in-time recovery is needed across heterogeneous
independent stores with unsynchronized write-path timelines (e.g., PostgreSQL WAL,
RocksDB sequence numbers, Iceberg snapshot IDs, git commits, and LanceDB versions that
have no shared global commit clock).

**Structure.**
1. Stamp a single **Hybrid Logical Clock (HLC) transaction-time T** at the start of each
   coordinated backup. HLCs provide causally ordered, cross-store coherent timestamps.
2. Each store takes its own native snapshot or archive and is then restored/queried
   AS-OF ≤ T via its own PITR/time-travel mechanism:
   - Postgres: `recovery_target_time = T` (WAL replay)
   - RocksDB: checkpoint sequence# ≤ T + WAL replay to T
   - Iceberg: `AS OF TIMESTAMP T` (snapshot-as-of-timestamp)
   - Detection content git: commit at T / `git bundle`
   - LanceDB/KG+vector: version corresponding to T
3. A **backup-set manifest** binds the per-store snapshot identifiers (Postgres LSN,
   RocksDB checkpoint seq#, Iceberg snapshot-id, git bundle SHA, KG+vector version) to
   the single T. Restore = read the manifest, bring each store to its recorded identifier
   via its own time-travel mechanism.
4. T MUST be the same `AS OF KNOWN <T>` watermark the query engine exposes (C8
   bitemporality — `ADR-PROP-prismql-deliverables.md`): backup recovery point =
   bitemporal query point. A recovered cluster serves `AS OF KNOWN T` queries consistent
   with the pre-failure state without additional transforms.
5. Reserve a physical application-consistent freeze for the few most tightly-coupled
   components only (components with sub-second write-path synchrony and no independent
   time-travel mechanism). Default: empty set; identify at morph.
6. Set per-store retention windows collectively to a **common floor** so that the
   cross-store RPO is not silently bounded by the shortest per-store window.

**Originating feature.** `ADR-PROP-backup-recovery.md` (C17 D-C17-SF2).

**Composes with.** C8 bitemporality (`AS OF KNOWN <T>` semantics); PAT-ADS-03 (Signed-
Offline-Bundle, for air-gap and satellite backup bundles); PAT-ADS-12 (Compliance Profile
DR-tier axis governs the retention floor and target RPO/RTO tier).

---

### PAT-ADS-16 — Sealed-Blob Key Escrow + Crypto-Shred

**When to use.** Key backup/recovery must coexist with operator-zero-access (INV-ADS-02).
The operator needs to store backup artifacts (including key blobs) but MUST NOT acquire
the ability to decrypt tenant data unilaterally.

**Structure.**
1. **Envelope:** per-tenant DEK (AES-GCM, encrypts tenant data) wrapped by per-tenant
   CMEK/KEK (SS-26 HD-1 + HD-4). This is the standard Option-3 envelope; recovery does
   not alter it.
2. **Operator stores ONLY sealed/wrapped key blobs it cannot unwrap.** The wrapping key
   that seals the blob is held by the tenant (tenant-held recovery key / on-prem HSM /
   external key manager) or distributed across independent custodians (M-of-N split).
3. **DEFAULT — Tenant-held recovery key:** the tenant's CMEK hierarchy is backed up as
   blobs encrypted under a key only the tenant controls. On recovery, the operator
   facilitates blob transport; the tenant unwraps. Operator = zero unilateral recovery
   capability.
4. **OPTIONAL — M-of-N threshold escrow tier (Shamir):** the recovery/root key is split
   into `n` shares, threshold `k`; shares distributed across tenant + auditor + regulator
   + (optionally) operator. No single party recovers alone. Modelled on HashiCorp Vault
   recovery keys. Audited break-glass: every use logged, reviewed, dual-controlled.
5. **Configuration:** the escrow model is a per-tenant or Compliance-Profile key-custody
   setting (PAT-ADS-12). The zero-access promise wording is **"no unilateral operator
   access"** — precise and correct for both the tenant-held default and the M-of-N
   optional tier.
6. **Crypto-shredding = erasure primitive AND zero-access reconciliation:** destroy the
   tenant's CMEK key → ALL encrypted data in ALL stores (including old backup media)
   becomes unreadable ciphertext. This is the GDPR right-to-erasure mechanism for pooled
   stores, the tenant offboarding primitive, and the reconciliation that makes operator-
   zero-access hold end-to-end: the operator always held only ciphertext; once the key is
   destroyed, the data is cryptographically dead.

**Originating feature.** `ADR-PROP-backup-recovery.md` (C17 D-C17-SF1).

**Composes with.** P-ADS-02 (Operator-Zero-Access-At-Rest); P-ADS-04 (Tenant-Keyed-
Central-Persistence); PAT-ADS-03 (Signed-Offline-Bundle — backup bundles use the same
signing infrastructure); INV-ADS-02 (operator zero-access); INV-ADS-10 (new — see C.1).

---

### PAT-ADS-17 — Compliance-Evidence-Export (RSAW-Aligned)

**When to use.** A regulated deployment must produce per-requirement audit evidence
from the system's own operational record. The operator or their auditor needs
discrete, well-organized, per-requirement artifacts — not raw logs — aligned to
a regulatory audit instrument (e.g., NERC Reliability Standard Audit Worksheets).

**Structure.**
1. **Audit substrate (prerequisite):** a tamper-evident, long-retention,
   provenance-tagged audit record. This is the C17/C18 decision-level audit output
   (INV-ADS-09: decision-level authorization log) + integrity-signed backup records
   (INV-ADS-10) + CIP-007-class event log (logons, privilege changes, config
   changes, detected events). The substrate must already exist before the export
   module is built; it is not created by this pattern.
2. **Per-requirement evidence export module:** a dedicated module that reads the
   substrate and emits RSAW-aligned evidence bundles, one bundle per CIP
   requirement (or analogous requirement in any regulatory regime). Each bundle
   contains the discrete artifact types the auditor expects:
   - CIP-004 / access-governance: BCSI-access provisioning/review/revocation logs
   - CIP-007 / logging: ≥90-day online log-retrieval attestations, long-term
     archive records, log-review activity records
   - CIP-009 / recovery: recovery-test records, post-restore baseline diffs,
     integrity-verification records
   - CIP-010 / config management: baseline snapshots, unauthorized-change
     detection trails, 15-month vuln-cadence attestation, control-effectiveness
     reports
   - CIP-013 / supply chain: software-integrity verification logs, signed-release
     attestations, support-channel audit records
3. **GRC-consumable output:** bundles are also parseable by external GRC platforms
   (OSCAL-compatible structure where applicable; RSAW-structured otherwise).
4. **Compliance Profile gating:** the active Compliance Profile (PAT-ADS-12)
   determines which requirement sets are active. A tenant running `nerc-cip`
   profile generates CIP bundles; a tenant running `iso27001` profile generates
   ISO 27001 Annex A evidence bundles. The export module is regime-agnostic at
   the substrate layer and regime-specific at the bundle-emission layer.
5. **Offline / air-gap safety:** evidence bundles are self-contained and
   cryptographically verifiable offline (carry detached signatures + Merkle
   hashes, PAT-ADS-03 signing infrastructure). An auditor with physical access
   to the bundle file does not need an online control plane to verify integrity.

**Forward note (C20 SF-2 seam S3).** When this audit substrate is built, the
provenance model should carry an evidence-OWNER dimension (`entity` / `csp` /
`shared`) to remain extensible for future CSP-shared-responsibility evidence
(C20 SF-2 seam S3, D-C20-SF2). Carry this one field when the substrate is built;
do not pre-build the substrate for this seam alone.

**Originating feature.** `ADR-PROP-nerc-cip-support.md` D-C20-SF1
(CIP audit-evidence / RSAW-export module). Consolidates the deferred RSAW export
packaging from C17 (D-C17-CIP009) and C10 GAP-Q2 evidence-package lean.

**Composes with.**
- PAT-ADS-12 (Configurable Compliance Profile): the active profile selects which
  requirement sets drive bundle generation; this pattern is the output-side
  complement of PAT-ADS-12's posture-enforcement role.
- INV-ADS-09 (Decision-Level Authorization Audit): the BCSI-access audit log that
  CIP-004 access reviews require is produced by the INV-ADS-09 substrate.
- INV-ADS-10 (Integrity-Verified Backups, Sealed-Blob Key Escrow): CIP-009
  recovery evidence requires integrity-verified backup and restore records, which
  the INV-ADS-10 substrate provides.
- PAT-ADS-03 (Signed-Offline-Bundle): evidence bundles use the same
  signing infrastructure for offline verifiability.

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
| INV-ADS-09 | Every authorization DECISION (subject, resource, action, attributes considered, policy bundle version, outcome) is logged at decision-resolution time — not merely at the access event. Offline satellites buffer decision logs locally and reconcile to Central on reconnect. | C18 decision-log (strongest enforcement); offline buffer + reconcile | `ADR-PROP-rbac-depth.md` D-C18-7 |
| INV-ADS-10 | Backups are customer-managed-key encrypted (operator stores ciphertext only); key escrow stores only operator-unwrappable blobs (no unilateral operator recovery); crypto-shred is the erasure primitive. Recoverability MUST NOT grant the operator at-rest access to tenant data. | C17 sealed-blob escrow + CMK backup encryption (strongest enforcement) | `ADR-PROP-backup-recovery.md` D-C17-SF1 |

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

INV-ADS check (all ten):
  [ ] INV-ADS-01: No raw sensor data at Central
  [ ] INV-ADS-02: Operator zero-access at rest
  [ ] INV-ADS-03: Per-tenant isolation enforced
  [ ] INV-ADS-04: Config authored only at Central
  [ ] INV-ADS-05: Actions gated and idempotent
  [ ] INV-ADS-06: AI-opaque
  [ ] INV-ADS-07: OCSF normalization at all boundaries
  [ ] INV-ADS-08: Air-gap deployment is valid reference profile
  [ ] INV-ADS-09: authorization decisions are logged, not just access
  [ ] INV-ADS-10: backups + key recovery preserve operator-zero-access (CMK-encrypted
      backups, sealed-blob key escrow, crypto-shred erasure primitive)
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

**See also:** AP-ADS-11 (Cross-Tenant DEK Grantee) for the key-plane analog —
adding another tenant's principal as a persistent grantee on a tenant's DEK is
the cryptographic form of the same violation this anti-pattern addresses at the
data/graph layer.

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

### AP-ADS-11 — Cross-Tenant DEK Grantee (amended 2026-06-27, C19)

**Forbidden.** Adding a parent tenant's (or any other tenant's) service account
or principal as a **persistent KMS grantee** (`cryptoKeyEncrypterDecrypter` in
GCP terminology, or an equivalent persistent KMS key policy grant in AWS/Azure)
on another tenant's DEK.

**Violates.** P-ADS-06 (Per-Tenant-Isolation), INV-ADS-03 (per-tenant partition
strict everywhere, no cross-tenant key sharing).

**Common form.** "The easiest way to let the parent see the child's data is to
add the parent's service account to the child's KMS key." This "easiest path"
persistently extends decryption rights across tenant boundaries and blurs the
semantics of per-tenant key isolation — directly contradicting the GCP multi-
tenancy baseline ("dedicated service accounts per customer; strongly prohibit
service accounts accessing across customer tenants") and Prism's own INV-ADS-03.

**Why this feels tempting.** Mechanism (c) (parent-as-grantee on child DEK)
appears in cloud documentation as a legitimate sharing pattern — and it IS
legitimate for same-principal / same-tenant data sharing. It becomes the wrong
mechanism when the grantee is a DIFFERENT tenant, because it creates a persistent
cross-tenant decryption right that cannot be cleanly scoped to a single query or
operation, is harder to audit at the application layer (KMS logs may not
distinguish which tenant initiated the decrypt), and breaks the never-share-a-DEK
invariant that P-ADS-06 depends on.

**Correct alternatives:**
- **For persisted parent-visible artifacts:** mechanism (b) — KMS `ReEncrypt`
  / proxy-re-encryption transforms ciphertext from the child key to the parent
  key inside the KMS boundary; the parent reads data under its OWN key. No DEK
  is shared.
- **For live in-query parent visibility:** mechanism (a) — consent-scoped
  transient decryption under the child key, scoped to the query duration,
  never persisted under the child key for the parent, fully audited child-side.
- **For OT / highest-assurance:** mechanism (d) — BYOC remote-op + HYOK + TEE;
  parent never holds keys at all.

See `ADR-PROP-nested-tenancy.md` §3.6 (Key-Custody Composition Table) for the
full mechanism analysis. See also AP-ADS-05 (no cross-tenant edges or similarity)
for the data-plane analog of this key-plane rule.

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
| `ADR-PROP-nested-tenancy.md` | P-ADS-01, P-ADS-02, P-ADS-04, P-ADS-06, P-ADS-09, P-ADS-11 | PAT-ADS-02 | INV-ADS-01..08 (all pass) |
| `ADR-PROP-sandboxed-expression-evaluator.md` | P-ADS-01 | — | — |
| `ADR-PROP-widget-dsl-render-and-schema-validation.md` | P-ADS-01 | — | — |
| `prismql-sequence-sugar-decisions.md` | P-ADS-01, P-ADS-09 | — | INV-ADS-04 |
| `ml-depth-phasing.md` | P-ADS-05, P-ADS-07 | PAT-ADS-05 | INV-ADS-06 |
| `ADR-PROP-rbac-depth.md` | P-ADS-01, P-ADS-06, P-ADS-09, P-ADS-13 | PAT-ADS-13, PAT-ADS-03, PAT-ADS-10 | INV-ADS-03, INV-ADS-04, INV-ADS-05, INV-ADS-06, INV-ADS-09 |
| `ADR-PROP-compliance-profiles.md` | P-ADS-09, P-ADS-11, P-ADS-12, P-ADS-13 | PAT-ADS-12, PAT-ADS-03, PAT-ADS-10 | INV-ADS-02, INV-ADS-03, INV-ADS-04 |
| `ADR-PROP-entity-masking.md` | P-ADS-02, P-ADS-03, P-ADS-06, P-ADS-07 | PAT-ADS-14, PAT-ADS-07 | INV-ADS-01, INV-ADS-02, INV-ADS-03, INV-ADS-06 |
| `ADR-PROP-backup-recovery.md` | P-ADS-02, P-ADS-04, P-ADS-11 | PAT-ADS-15, PAT-ADS-16, PAT-ADS-03 | INV-ADS-02, INV-ADS-03, INV-ADS-10 |
| `ADR-PROP-nerc-cip-support.md` | P-ADS-09, P-ADS-11, P-ADS-12, P-ADS-13 | PAT-ADS-12, PAT-ADS-17 | INV-ADS-01, INV-ADS-02, INV-ADS-09, INV-ADS-10 |

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

---

**Amendment log**

| Version | Date | Change |
|---------|------|--------|
| v1.0 | 2026-06-27 | Initial capture — 12 principles, 11 patterns, 8 invariants, 11 anti-patterns; seeded from ripple audit. |
| v1.1 | 2026-06-27 | P-ADS-02 clarification (C19 operator-zero-access spectrum + MSSP mediated-access semantics); AP-ADS-11 Cross-Tenant DEK Grantee added (C19 key-plane isolation); P-ADS-13 Configurable-Not-Prescriptive (Policy-as-Configuration) added (C18 configurability directive); P-ADS-11 cross-reference to P-ADS-13 added. |
| v1.2 | 2026-06-27 | C18 capture: added PAT-ADS-12 (Configurable Compliance Profile), PAT-ADS-13 (Layered-Authz), INV-ADS-09 (Decision-Level Authorization Audit); INV-ADS-09 check line added to Section C.2 Conformance Checklist; traceability rows for ADR-PROP-rbac-depth.md + ADR-PROP-compliance-profiles.md added to Section E. |
| v1.3 | 2026-06-27 | C16 capture: P-ADS-07 sharpened (clearing-house enforcement mechanism + embeddings-are-sensitive-data-class + dual-index + zero-vault-wiring structural invariant); added PAT-ADS-14 (Edge-Tokenizing-Clearing-House); traceability row for ADR-PROP-entity-masking.md added to Section E. |
| v1.4 | 2026-06-27 | C17 capture: added PAT-ADS-15 (Logical-Watermark Cross-Store Backup), PAT-ADS-16 (Sealed-Blob Key Escrow + Crypto-Shred), INV-ADS-10 (Recoverability Preserves Operator-Zero-Access); INV-ADS-10 check line added to Section C.2 Conformance Checklist; traceability row for ADR-PROP-backup-recovery.md added to Section E. |
| v1.5 | 2026-06-27 | C20 capture: added PAT-ADS-17 (Compliance-Evidence-Export RSAW-aligned); traceability row for ADR-PROP-nerc-cip-support.md added to Section E. (C20 SF-2 cloud-BES-future OPEN pending research.) |
| v1.6 | 2026-06-27 | C20 SF-2 fold: PAT-ADS-17 evidence-owner-dimension forward-note (cloud-BES seam S3, D-C20-SF2); SF-2 resolved = Defer + Leave-Seams-Open (Sub-Option B). |

*End of Prism Architecture Design System v1.6 — 2026-06-27*
