---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-DD-1: Single-codebase + runtime DEPLOYMENT-PROFILE (~90% shared; no per-model fork)"
  - "ADR-PROP-DD-2: THREE-operating-model deployment matrix (SaaS / MSSP-managed / client-managed)"
  - "ADR-PROP-DD-3: Operator-role as profile dimension, not build target"
  - "ADR-PROP-DD-4: Uniform tenant-id abstraction across the full single-org→multi-customer spectrum"
  - "ADR-PROP-DD-5: BYOC zero-access by construction (satellite-local creds + C2 residency invariant)"
  - "ADR-PROP-DD-6: Deployment-conditional C9 migration posture (cross-reference only)"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Side-analysis deployment-matrix capture; human-confirmed decisions 2026-06-27 session.
  Research basis: research/dual-deployment-saas-onprem-2026-06-27.md (PRIMARY — sonar-deep-research
  at reasoning_effort=high; 6 SaaS/on-prem deployment case studies confirmed single-codebase+profile
  pattern; divergent-fork failure mode documented). Cross-refs: ADR-PROP-config-management.md
  (D-C9-Q3-DEPLOYMENT migration posture); ADR-PROP-central-deployment-access-layer.md (C1 transport/
  identity/ops); ADR-PROP-satellite-mesh.md (C2 residency + per-hop mTLS); secret-subsystem-sketch.md
  + AD-017 (satellite-local credentials); matured-vision §3.1/§3.2.
  Does NOT modify live ADR files, ARCH-INDEX.md, STATE.md, SESSION-HANDOFF.md, or any live factory artifact.
traces_to:
  - matured-vision-day2-requirements.md §16.4 (C10 deployment-matrix decision log entry)
  - day2-design-decisions/ADR-PROP-config-management.md (D-C9-Q3-DEPLOYMENT — deployment-conditional migration posture)
  - day2-design-decisions/ADR-PROP-central-deployment-access-layer.md (C1 — deployment access, scaling, ops)
  - day2-design-decisions/ADR-PROP-satellite-mesh.md (C2 — residency enforcement + satellite-local credential resolution)
  - day2-design-decisions/secret-subsystem-sketch.md (SS-26 SecretBackend; AD-017 AI-opaque credentials)
  - matured-vision-day2-requirements.md §3.1 (central deployment pivot)
  - matured-vision-day2-requirements.md §3.2 (satellite mesh / residency)
  - research/dual-deployment-saas-onprem-2026-06-27.md (PRIMARY research basis)
  - CLAUDE.md (AD-017 satellite-local credentials; ArcSwap config hot-reload; single-workspace MSRV)
---

# ADR-PROP — Dual Deployment: Three-Operating-Model Matrix (C10)

> **STATUS: FULLY DECIDED 2026-06-27 (human) — single-codebase+profile model, THREE-model deployment
> matrix, BYOC zero-access thesis, and deployment-conditional migration posture confirmed.**
> This is a CAPTURE artifact for the side-analysis day-2 program. `do_not_execute: true`.
> Real ADR numbers and formal ARCH-INDEX.md rows are deferred to the morph execution
> (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

> **Research basis:** `research/dual-deployment-saas-onprem-2026-06-27.md` — PRIMARY
> (sonar-deep-research at `reasoning_effort=high`; six SaaS+on-prem deployment case studies
> covering GitLab, Sentry, Elastic, Mattermost, GitHub Enterprise, Grafana; divergent-fork
> failure mode documented as the canonical anti-pattern). All load-bearing claims are
> source-grounded in that document.

---

## Context

Prism is designed for analysts at MSSPs (1898 & Co) and their clients. From day one the
product brief implied more than a single deployment shape. Three distinct deployment shapes
have emerged from the side-analysis sessions:

1. A vendor-hosted SaaS service serving multiple MSSP customers.
2. An MSSP-operated service running on customer or MSSP infrastructure.
3. A client-SOC-operated service running on the client's own infrastructure, with no MSSP
   in the loop.

These shapes differ in WHO HOSTS the compute and WHO OPERATES the service — but the
core query engine, sensor adapters, detection logic, config subsystem, satellite mesh,
and PrismQL surface are identical across all three.

This document records the DECIDED cross-cutting architecture for the deployment matrix.
It IS the authoritative capture for the "dual deployment" question; the name "dual" is
retained for continuity but the actual decision is a THREE-model matrix.

---

## Decision Ledger

### D-DEPLOY-001 — Single Codebase + Runtime Deployment-Profile (~90% shared)

**DECIDED 2026-06-27 (human).**

Prism ships ONE codebase. Deployment-model variation is expressed entirely via a
**runtime DEPLOYMENT-PROFILE** flag set; approximately 90% of the code is shared
across all three operating models.

**What varies by profile:**

| Dimension | SaaS | MSSP-managed | Client-managed |
|-----------|------|--------------|----------------|
| Hosting | Vendor (Prism/1898) infra | Customer / MSSP infra | Client's own infra |
| Operator | Vendor SRE | MSSP (1898) admin | Client SOC |
| Tenancy | Multi-CUSTOMER | Multi-CLIENT | Single-org (or internal BU) |
| RBAC default | Vendor SRE admin role | MSSP admin role | Client SOC admin role |
| Release cadence | Continuous (k8s blue-green) | Offline-signed-bundle + A/B appliance | Same as MSSP-managed |
| Migration exposure | LOW (walks every release) | MEDIUM (may skip minor) | HIGH (skip-version realistic) |
| Skip-version | Rarely exercised | Bundle carries full chain | Full chain + required-stop CI required |

**What is identical across all three:**

- The Rust crate workspace, all sensor adapters, the PrismQL parser + planner + executor,
  the detection engine, the satellite mesh protocol, the config-management subsystem
  (D-C9-* decisions), the secret broker (SS-26), the OCSF normalization pipeline, and
  the data-model schema.
- The config-authority model (DB-authoritative + UI + git-backed; D-C9-Q1-AUTHORITY/VERSION).
- The canary + fast-revert + bootstrap-recovery posture (D-C9-Q2 / D-C9-FAST-REVERT /
  D-C9-BOOTSTRAP) — only the operator who triggers each action changes.
- The satellite autonomous self-recovery mechanics (D-C9-BOOTSTRAP Layer 4).

**Rationale (research-grounded):**

Surveyed prior art across six products (GitLab, Sentry, Elastic, Mattermost, GitHub Enterprise,
Grafana). All six converged on a single-codebase + profile/flag model after divergent-fork
experiments failed. The divergent-fork failure mode is documented:

- GitLab.com vs GitLab CE/EE: originally separate; converged to one codebase after
  maintaining two divergent trees proved untenable. The "availability" (SaaS) vs
  "reliability" (self-managed) operational posture is now a config flag, not a fork.
- Sentry: same codebase ships as Sentry Cloud and self-hosted `getsentry/self-hosted`
  (Docker Compose). Feature flags and deployment config differentiate the profiles.
- Elastic Cloud on Kubernetes (ECK) vs self-managed: same Elasticsearch binary; ECK
  adds the orchestration layer. Kibana, Logstash, and Beats binaries are identical.
- Mattermost Team vs Enterprise: same binary; Enterprise features unlocked by license key.
- GitHub.com vs GitHub Enterprise Server (GHES): same core; GHES is a packaged appliance
  of the same codebase.
- Grafana Cloud vs self-hosted: same Grafana binary; deployment profile controls
  multi-tenancy, billing integration, and fleet management.

Across all six: the boundary between shared code and deployment-specific code is ~90/10.
The 10% variation covers: operator RBAC defaults, licensing/entitlement, fleet-management
wiring, and multi-tenancy scope binding. Prism mirrors this split.

[research/dual-deployment-saas-onprem-2026-06-27.md §single-codebase-validation]

---

### D-DEPLOY-002 — THREE Named Operating Models

**DECIDED 2026-06-27 (human).**

The deployment matrix has TWO AXES — WHO HOSTS × WHO OPERATES — yielding THREE
named operating models:

| Model | Hosting | Operator | Tenancy |
|-------|---------|----------|---------|
| **SaaS** | Vendor (Prism/1898) | Vendor SRE | Multi-CUSTOMER |
| **MSSP-managed** | Customer or MSSP infra | MSSP (1898) | Multi-CLIENT |
| **Client-managed** | Client's own infra | Client SOC | Single-org (or internal BU) |

**SaaS model specifics:**

- Central service hosted and operated by Prism/1898.
- Multiple MSSP customers share one central deployment, each with cryptographically
  isolated tenant scope (C1 per-OrgId DEK isolation; ADR-PROP-central-deployment-access-layer.md).
- Release cadence: continuous (k8s blue-green). Canary is FLEET-CANARY over SaaS
  customer cohorts PLUS the standard per-deployment satellite-scoped canary (two-tier).
- Backup/DR is vendor responsibility.
- Satellite mesh (C2) is still satellite-local; data never transits central SaaS for
  residency-constrained sources. BYOC zero-access thesis applies in full (D-DEPLOY-005).

**MSSP-managed model specifics:**

- Infrastructure is customer-provided or MSSP-provided (on-prem, private cloud, or
  dedicated cloud tenant).
- MSSP (1898) operates and maintains the deployment: upgrades, monitoring, incident
  response.
- Multi-CLIENT tenancy: multiple client organizations share one MSSP-managed Prism
  instance, each with isolated OrgId scope.
- Release cadence: offline-signed-bundle + A/B appliance (D-C9-Q3-DEPLOYMENT; same
  as client-managed); skip-version exposure MEDIUM.
- BYOC zero-access thesis applies — credentials and raw data remain on MSSP/client
  infra; central (if any SaaS layer exists) sees only sanitized OCSF results.

**Client-managed model specifics:**

- Infrastructure, operations, and upgrades are entirely the CLIENT SOC's responsibility.
- No MSSP in the loop post-deployment. MSSP may provide initial onboarding.
- Single-org tenancy by default; internal BU segmentation is possible via OrgId
  sub-registration without re-architecting.
- Release cadence: same offline-signed-bundle + A/B appliance as MSSP-managed; skip-
  version exposure HIGH (v1.2 → v1.7 is realistic).
- BYOC zero-access thesis is the STRONGEST under this model: the CLIENT operates their
  own instance; vendor and MSSP never touch it. This is the "air-gap" deployment shape.

**Tenancy bridging:**

- A client-managed deployment that later admits MSSP oversight does NOT require a
  re-architecture. The operator role is a PROFILE DIMENSION; adding an MSSP admin
  role binding is a config change, not a codebase change.
- A multi-CLIENT MSSP-managed deployment that acquires a new client is a new OrgId
  registration; no schema migration required beyond the normal config push.

---

### D-DEPLOY-003 — Operator-Role as Profile Dimension, NOT Build Target

**DECIDED 2026-06-27 (human).**

The DEPLOYMENT-PROFILE carries an **OPERATOR-ROLE dimension** (vendor / MSSP / client-SOC)
alongside the HOSTING dimension. The operator role governs:

- **RBAC default roles:** which roles are created by default at first-boot and what
  permissions they carry (vendor SRE default vs MSSP admin default vs client SOC admin
  default).
- **Day-3 approval-workflow defaults** (deferred; D-C9-APPROVAL): the approval chains
  that matter for a client SOC are different from those for a vendor SRE — but the
  MECHANISM is the same configurable engine.
- **Support / onboarding / licensing surface:** these differ by operator role and are
  go-to-market / operational, NOT architecture.

**What operator-role does NOT affect:**

- The codebase, the data model, the query engine, the sensor adapters, the detection
  engine, the satellite mesh, or the config-management subsystem. All of these are
  identical across all three operating models.

**Anti-pattern: operator-role as a build target.**

Building a separate binary or separate Cargo workspace feature-set per operator role
is the divergent-fork anti-pattern. It is explicitly rejected. The operator role is a
runtime-profile flag that controls RBAC defaults and workflow defaults — it is never
a compile-time gate on functionality.

---

### D-DEPLOY-004 — Uniform Tenant-Id Abstraction Across the Full Spectrum

**DECIDED 2026-06-27 (human).**

The `OrgId` / `OrgSlug` / `OrgRegistry` abstraction (already wired in the current
data-plane) is the UNIFORM tenant-id abstraction across the FULL spectrum:

| Deployment shape | OrgId scope |
|-----------------|-------------|
| Client-managed, single-org | ONE OrgId = the client organization |
| Client-managed, internal BU segmentation | Multiple OrgIds = internal business units |
| MSSP-managed, multi-client | One OrgId per client in the MSSP's portfolio |
| SaaS, multi-customer | One OrgId per MSSP customer organization |

This single abstraction spans the full range without requiring a separate "super-tenant"
or "account" layer above OrgId. An MSSP admin who manages all client OrgIds does so
via the existing per-connection OrgId scope binding (C1 D-C1-D2; the analyst's token
claims determine which OrgIds are accessible).

**No new abstraction layer is introduced by this decision.** The OrgId data-plane is
already correct; D-DEPLOY-004 records the formal invariant that it is the authoritative
single abstraction across all three models.

---

### D-DEPLOY-005 — BYOC Zero-Access By Construction (HEADLINE)

**DECIDED 2026-06-27 (human). THIS IS PRISM'S STRONGEST SAAS DIFFERENTIATOR.**

Prism's satellite mesh IS the BYOC (Bring Your Own Cloud) zero-access data-plane
BY CONSTRUCTION:

1. **C2 residency invariant** (ADR-PROP-satellite-mesh.md D-C2-12): raw sensor data and
   credentials NEVER leave the residency zone. The satellite resolves credentials locally
   (AD-017 satellite-local credential resolution hard invariant) and sends only sanitized
   OCSF results to the coordinator/central.
2. **AD-017 AI-opaque credentials**: credential values never transit the AI context or
   the central service. The secret broker (SS-26) resolves references; the resolved
   secret value stays at the satellite.
3. **Egress-blocked CI invariant** (existing; STANDING GUARD): the test suite CI cannot
   make real egress calls. This invariant must remain enforced across ALL three deployment
   models. Air-gap-leak is the most dangerous risk; the CI guard is the standing sentinel.

**BYOC litmus:** A client who runs client-managed Prism has a vendor/MSSP that never sees
their raw data or credentials. A client who uses MSSP-managed Prism has a central SaaS
layer (if any) that never sees their raw data or credentials (only sanitized OCSF results
transit). A SaaS customer has a vendor that never sees their raw sensor data (satellite-local
processing, OCSF normalization at the boundary).

**This thesis is STRENGTHENED across all three operating models.** The client-managed
model is the purest expression: the vendor and MSSP are entirely out of the loop. The
MSSP-managed model provides an intermediate assurance level. The SaaS model provides the
satellite-mesh BYOC guarantees even against the vendor.

**SaaS differentiator framing:** "We built BYOC zero-access into the architecture
by construction, not as an add-on. Our satellite mesh ensures the vendor never sees your
raw data or credentials — regardless of which operating model you choose."

---

### D-DEPLOY-006 — Deployment-Conditional C9 Migration Posture (Cross-Reference)

**DECIDED 2026-06-27 (human). CROSS-REFERENCE ONLY — see ADR-PROP-config-management.md §D-C9-Q3-DEPLOYMENT.**

The C9 config-management decisions (ADR-PROP-config-management.md) include a deployment-
conditional migration posture (D-C9-Q3-DEPLOYMENT) that is the C9-local slice of the
cross-cutting deployment matrix:

| Operating model | Skip-version exposure | Migration posture |
|-----------------|----------------------|-------------------|
| **SaaS** (k8s blue-green, continuous) | LOW | Blue-green rollback; chain barely exercised; required-stops not needed. |
| **MSSP-managed** (offline-signed-bundle, A/B appliance, watchdog) | MEDIUM | Bundle carries full ordered chain; A/B validates migrated state before cutover; watchdog covers boot-bricking migration. |
| **Client-managed** (self-operated) | HIGH | Full chain + supported-window skip-version check + required-stop at oldest in-window LTS + golden-fixture upgrade-matrix CI + idempotent/atomic/resumable on-open. |

This decision block is recorded in C9 and cross-linked here. The full C9 migration posture
is the canonical source of truth for this slice. The deployment matrix (this document) is
the canonical source of truth for the three-model naming and the axis definition.

---

## Open Sub-Choices (EXPLICITLY OPEN — Do NOT Resolve Here)

### OQ-DEPLOY-1 — Tenancy-Isolation Depth

**OPEN. Requires a targeted decision pass.**

The deployment matrix decides the SCOPE of the tenant-id abstraction (single-org → multi-BU
→ multi-client → multi-customer on ONE OrgId abstraction). It does NOT decide the
ISOLATION DEPTH within the shared infrastructure:

| Isolation model | Description | Tradeoffs |
|----------------|-------------|-----------|
| **Pool** | All tenants share one DB schema + one set of tables; row-level filtering | Cheapest operationally; weakest isolation; blast radius = full fleet |
| **Bridge** | Shared schema, per-tenant schemas or namespaces within one DB instance | Moderate isolation; single DB failure still affects all |
| **Silo** | Fully separate DB instances per tenant (or per-org) | Strongest isolation; highest operational cost |
| **Cell** | Fully separate infrastructure cells per customer | Maximum isolation; only viable for large customers |

Recommendation for day-2: **pool model** for the hot-path RocksDB data (current state — OrgId
key-prefix isolation already wired), **bridge model** for the PostgreSQL case/alert store
(per-tenant schema in one bundled Postgres instance), with the silo model as a future
upgrade path for large SaaS customers.

**THIS RECOMMENDATION IS NOT DECIDED.** Record as open; flag for a targeted architectural
decision at morph time. The choice has significant operational, cost, and compliance
implications (SOC 2 Type II, GDPR data isolation requirements) that require a dedicated
research pass.

### OQ-DEPLOY-2 — Residual BYOC Hardening Gaps

**OPEN. Record as open hardening items — NOT blocking architecture decisions.**

The BYOC zero-access thesis (D-DEPLOY-005) is structurally sound by construction. Several
residual hardening items require dedicated investigation before the SaaS model can be
certified as meeting enterprise BYOC requirements:

| Gap | Description | Status |
|----|-------------|--------|
| **Result-transit residency** | OCSF-normalized results transit from satellite to central. Even sanitized results may carry PII (entity names, IP addresses). Is the transit path encrypted end-to-end? Does it respect residency zone boundaries? | Open — C2 per-hop mTLS covers encryption; residency-of-results (vs residency-of-raw) needs explicit policy decision. |
| **Metadata-leakage audit** | Query execution metadata (query text, table names, timing, plan shape) transits to central for observability. Does query text contain PII? Does table/column naming reveal sensitive schema? | Open — a metadata-scrubbing or metadata-minimization policy pass is needed before SaaS launch. |
| **Ephemeral dial-home tokens** | Satellites use dial-home tokens to authenticate to the coordinator. Are these tokens ephemeral? What is their rotation cadence? Can a compromised token be revoked before its expiry window? | Open — C2 D-C2-5 bootstrap (join-token OOB + optional TPM) covers initial trust; rotation cadence is OQ-C2 (open in ADR-PROP-satellite-mesh.md). |
| **CMEK for central metadata** | The central service stores query plans, audit logs, and OCSF-normalized event summaries. Does it support Customer-Managed Encryption Keys so SaaS customers retain key custody for their metadata? | Open — SS-26 per-tenant DEK covers the data-plane; whether CMEK extends to central metadata storage (PostgreSQL, observability) is an open question. |

Record these four items as open hardening items. They do NOT block the day-2 architecture
decisions; they are items for the security-reviewer at the relevant morph stories.

---

## Provable Invariants (PIV-DEPLOY-*)

| ID | Invariant |
|----|-----------|
| **PIV-DEPLOY-001** | Exactly ONE Rust crate workspace exists. There is no per-operating-model fork, branch, or Cargo workspace. The deployment-profile flag is a runtime/config distinction, never a compile-time target. |
| **PIV-DEPLOY-002** | Operator-role (vendor / MSSP / client-SOC) is a PROFILE DIMENSION, not a build target. No `#[cfg(feature = "mssp")]` or equivalent compile-time gate exists for operator-role-specific behavior. |
| **PIV-DEPLOY-003** | The `OrgId` abstraction is the UNIFORM tenant boundary across the full single-org → multi-BU → multi-client → multi-customer spectrum. No "super-tenant" or "account" abstraction layer sits above OrgId. |
| **PIV-DEPLOY-004** | SaaS central NEVER receives raw sensor data or credential values. Only OCSF-normalized results transit from satellite to central. (AD-017 + C2 residency invariant.) |
| **PIV-DEPLOY-005** | Egress-blocked CI invariant is ALWAYS enforced across ALL three operating models. No CI job in any operating-model branch may make real egress calls. |
| **PIV-DEPLOY-006** | The config-authority model (DB-authoritative + UI + git-backed) is IDENTICAL across all three operating models. What varies by operator is RBAC defaults and day-3 workflow defaults, not the config-authority model itself. |
| **PIV-DEPLOY-007** | A client-managed deployment CAN admit an MSSP operator role WITHOUT re-architecting. Adding an MSSP admin role binding is a config change (new OrgId scope binding in the profile), not a codebase change or schema migration. |
| **PIV-DEPLOY-008** | The skip-migration-STEP rule (D-C9-Q3-MODEL: no step may be skipped even on a skip-version release) applies uniformly across all three operating models. The migration runner is the same binary component in all three. |

---

## Honest Costs

| Item | Cost / Risk |
|------|-------------|
| **~10% per-model variation must be actively maintained** | The ~90% shared code claim requires discipline. Every new feature must be reviewed for whether it introduces a new profile-conditional branch. If profile-conditional branches accumulate without pruning, the effective shared-percentage erodes. The anti-pattern is "just add a flag" for every operator-specific nuance; the correct pattern is designing the feature so the core is shared and the variation is truly at the profile boundary. |
| **MSSP-managed and client-managed require offline-bundle tooling** | Building, signing, and distributing offline upgrade bundles is a distinct engineering surface from continuous k8s deployment. The bundle must carry: the full binary, the full migration chain, all satellite WASM plugins, sensor TOML specs, and the bootstrap recovery materials. This is a dedicated DevOps workstream. |
| **Client-managed creates a long-tail support surface** | Once a client runs their own deployment, Prism/1898 has limited visibility into their configuration state and upgrade status. The fleet-staged canary and watchdog mechanisms help, but the MSSP/vendor cannot force upgrades. Required-stop LTS discipline (D-C9-Q3-SKIP) is the primary mitigant; it bounds the tail of unsupported configurations. |
| **BYOC hardening gaps (OQ-DEPLOY-2) must be closed before SaaS launch** | The BYOC thesis is architecturally sound; the four open hardening items (result-transit residency, metadata-leakage, ephemeral dial-home tokens, CMEK) are implementation details that must be closed before the SaaS model can be marketed with BYOC claims. Shipping SaaS without these is a compliance and marketing risk. |
| **Tenancy-isolation depth choice (OQ-DEPLOY-1) has cost implications** | Pool is cheapest; silo is most defensible to enterprise procurement. The choice affects both infrastructure cost and the compliance artifacts needed for SOC 2 / GDPR. |

---

## Alternatives Considered and Rejected

### Alternative A: Per-Model Fork (Separate Branches or Repos per Operating Model)

Maintain separate codebases (or long-lived divergent branches) for the SaaS, MSSP-managed,
and client-managed models.

**Rejected (D-DEPLOY-001) because:** The research documents the divergent-fork failure
mode across six mature products. The maintenance cost of keeping three code variants in
sync with a shared core is superlinear in the number of features shipped. GitLab's own
history (CE vs EE divergence → convergence to one codebase) is the canonical proof case.
Runtime-profile variation is strictly better than compile-time fork variation for products
that share >80% of their logic.

### Alternative B: Separate "Super-Tenant" Layer Above OrgId

Introduce a new abstraction (e.g., `AccountId`, `TenantGroupId`) above the existing `OrgId`
to handle multi-MSSP-customer or multi-client-of-MSSP tenancy.

**Rejected (D-DEPLOY-004) because:** The existing OrgId abstraction already spans the
full spectrum — the number of OrgIds in a deployment is a config/operational parameter,
not an architectural one. A super-tenant layer would require changes to every BC, every
query plan, every audit record, and every BC-enforced invariant that currently operates
on OrgId. The additional abstraction layer adds complexity with no architectural benefit
over the existing design.

### Alternative C: Separate BYOC Product as an Architecture Fork

Position the client-managed deployment as a separate "BYOC Prism" product with a distinct
codebase or distinct data-plane path.

**Rejected (D-DEPLOY-005) because:** The BYOC zero-access property is delivered BY
CONSTRUCTION in the satellite mesh + AD-017 satellite-local credentials. It is NOT a
special BYOC variant of the product; it is the default behavior of every deployment where
a satellite is present. Forking into a "BYOC edition" would dilute the thesis (the SaaS
edition would no longer be BYOC-credible) and create the fork maintenance problem
(Alternative A above).

---

## Ripple Effects (Must Be Picked Up at Morph Time)

| Affected area | Ripple |
|---------------|--------|
| **ADR-PROP-config-management.md D-C9-Q3-DEPLOYMENT** | The three-model naming and axis definition in this document is the canonical source. D-C9-Q3-DEPLOYMENT cross-links to this document; morph ADRs must cite both in both directions. |
| **ADR-PROP-central-deployment-access-layer.md (C1)** | C1 decisions (transport, identity, credentials, shared state, ops) apply to the SaaS and MSSP-managed deployment shapes. Client-managed uses the same rmcp stdio transport (no Streamable HTTP required for single-analyst client-managed). The morph ADRs for C1 must document per-model scope clearly. |
| **ADR-PROP-satellite-mesh.md (C2)** | The satellite mesh topology (max chain depth 8, per-hop mTLS, store-and-forward) is IDENTICAL across all three models. What varies is the satellite COUNT and the OPERATOR who manages the trust anchors. Morph satellite-mesh ADRs must note this. |
| **secret-subsystem-sketch.md (SS-26)** | The SecretBackend trait + satellite-local resolution is the mechanism that delivers D-DEPLOY-005. The morph ADR for SS-26 must cite PIV-DEPLOY-004 as a binding invariant. |
| **Offline bundle tooling** | The MSSP-managed and client-managed models require a signed offline bundle distribution pipeline. This is not in the current devops scope. A new epic (E-BUNDLE-DEPLOY-001) must be proposed at morph for the bundle tooling. |
| **OQ-DEPLOY-1 (tenancy-isolation depth)** | Requires a dedicated architectural decision pass at morph. The pool/bridge/silo/cell tradeoff has compliance and cost implications. Proposed route: architect → targeted perplexity research pass → human decision → morph ADR. |
| **OQ-DEPLOY-2 (BYOC hardening gaps)** | Four open items (result-transit residency, metadata-leakage, ephemeral dial-home tokens, CMEK). Must be closed before SaaS launch. Proposed route: security-reviewer + architect at the relevant morph stories. |
| **matured-vision §16.4** | C10 deployment-matrix decision block appended in-place (2026-06-27). |
