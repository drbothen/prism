---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C19-1: Tree representation — adjacency list (authoritative parent_id) + closure table (auth/RBAC/policy/metering read-index) + optional materialized path (cell/shard placement)"
  - "ADR-PROP-C19-2: Isolation tier — bridge by default; isolation_tier (pool | silo | cell) as per-node tenant attribute; closes OQ-DEPLOY-1"
  - "ADR-PROP-C19-3: Config/policy inheritance — hybrid (values inherit-then-child-override; security guardrails intersect + parent-deny-is-final)"
  - "ADR-PROP-C19-4: Depth — unlimited depth technically; configurable soft cap default 8"
  - "ADR-PROP-C19-5: SF-3 visibility-grant matrix — configurable preset ladder P0..P3; P3 transparent-subtree gated to same-legal-entity; (c) forbidden as AP-ADS-11"
  - "ADR-PROP-C19-6: Key custody — per-child DEK always (leaf); flat DEK or nested KEK hierarchy configurable per-deployment; SF-4 configurable"
  - "ADR-PROP-C19-7: Reparenting — admin-only, audited, Central-authored; explicit re-pairing/re-consent for new ancestors"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Day-2 out-of-band SIDE-ANALYSIS capture; human-confirmed decisions 2026-06-27 session.
  Research basis: research/nested-tenancy-2026-06-27.md (PRIMARY — base C19 research,
  two perplexity_research sonar-deep-research calls covering tree-encoding tradeoffs,
  isolation taxonomy, RBAC delegation, envelope-encryption key custody, metering roll-up)
  + research/nested-tenancy-parent-visibility-2026-06-27.md (SF-3 deep dive — two additional
  perplexity_research sonar-deep-research calls at reasoning_effort=high covering parent-sees-all
  market precedents, key-custody mechanism analysis, OT/NERC-CIP regulatory implications,
  consent/governance/audit governance controls).
  This is an out-of-band side-analysis. CAPTURE ONLY. Does NOT modify any live spec, ADR-registry
  artifact (specs/architecture/), BC, story, STATE.md, or SESSION-HANDOFF.md. No git operation
  performed. Real ADR numbers and formal ARCH-INDEX.md rows deferred to the morph execution.
  touches_no_live_artifacts: true
seeded_from:
  - research/nested-tenancy-2026-06-27.md
  - research/nested-tenancy-parent-visibility-2026-06-27.md
cross_refs:
  - specs/day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md (ADS conformance)
  - specs/day2-design-decisions/ADR-PROP-sso-identity.md (C18 RBAC role model — (role, scope-node))
  - specs/day2-design-decisions/ADR-PROP-prism-context.md (C16 masking via C12; C12 per-tenant graph)
  - specs/day2-design-decisions/secret-subsystem-sketch.md (SS-26 per-tenant DEK / CMEK)
  - specs/day2-design-decisions/ADR-PROP-dual-deployment.md (C10 operating models / P-ADS-11)
  - specs/day2-design-decisions/ADR-PROP-satellite-mesh.md (C2 satellite headless / D-C2-12 residency)
  - specs/day2-design-decisions/ADR-PROP-config-management.md (C9 Config-DB-Authoritative / INV-ADS-04)
  - matured-vision-day2-requirements.md §16.4 (C19 decision log entry)
  - CLAUDE.md (AD-017 AI-opaque credentials; OrgSlug newtype; Arc-DI plumbing)
traces_to:
  - matured-vision-day2-requirements.md §3.1 (central deployment model)
  - matured-vision-day2-requirements.md §16.4 (C19 decisions log entry)
  - deployment-matrix OQ-DEPLOY-1 (tenancy-isolation depth — RESOLVED by this ADR-PROP)
  - SS-26 secret broker per-tenant DEK
  - C18 RBAC / 7-role console (role model — forward cross-link)
  - C11 metering (rollup — forward cross-link)
  - C20 OT / NERC-CIP (regulatory_class attribute — forward cross-link)
  - C17 key-escrow / Option-3 CMEK (forward cross-link)
  - C15 approver roles in gated-action path (forward cross-link)
  - C16 detokenize-at-surface via RBAC (forward cross-link)
---

# ADR-PROP — Nested / Hierarchical Tenancy: Tree, Isolation, Visibility, Key Custody (C19)

> **STATUS: DECIDED 2026-06-27 (human).** Full decision record for C19 — nested/hierarchical
> tenancy (unlimited depth), covering tree representation, isolation tier, config/policy
> inheritance, depth policy, parent visibility (SF-3), key custody (SF-4), reparenting (SF-5),
> RBAC, metering, and MSSP reconciliation. This is a CAPTURE artifact (`do_not_execute: true`).
> Real ADR numbers and formal ARCH-INDEX.md rows are deferred to the morph execution.

> **Research basis:**
> (1) `research/nested-tenancy-2026-06-27.md` — base C19 research (two perplexity_research
> sonar-deep-research calls at reasoning_effort=high/medium; covers resource-hierarchy precedents,
> tree-encoding tradeoffs, isolation taxonomy, RBAC delegation, envelope-encryption key custody under
> nesting, metering roll-up, OQ-DEPLOY-1 resolution).
> (2) `research/nested-tenancy-parent-visibility-2026-06-27.md` — SF-3 deep dive (two additional
> perplexity_research calls at reasoning_effort=high; covers all 12+ MSSP/cloud/SaaS parent-visibility
> precedents, four key-custody mechanisms against INV-K1/INV-K2, OT/NERC-CIP regulatory constraints,
> consent/governance/audit controls, configurable visibility-grant matrix design).

---

## 1 — Context and Scope

Prism requires hierarchical tenancy to support the MSSP operating model: partner → sub-partner →
customer → business-unit, with arbitrary depth. This ADR-PROP resolves all decisions in the C19
nested-tenancy design space, including the deployment-matrix open question OQ-DEPLOY-1 (tenancy-
isolation depth: pool/bridge/silo/cell).

The existing codebase uses `OrgId`/`OrgSlug` as the flat per-tenant abstraction. C19 adds depth
to this abstraction via an `parent_id` field on the existing tenant node plus two derived indexes.
No new fundamental isolation mechanism is required — the bridge model already present in the
architecture (satellite data-plane silo/cell by construction per D-C2-12; Central control-plane
pool-with-row-scoping) is extended with a per-node isolation tier attribute.

---

## 2 — Decisions

### D-C19-1 — Tree Representation

**Decision:** Adjacency list (`parent_id` on the existing OrgId/OrgSlug tenant node) as the
authoritative source of truth, plus a **closure table** (`ancestor, descendant, depth`) as the
read-index for auth/RBAC/policy/metering queries, plus an **optional materialized path** (`/1/3/…`)
as a lazy-derived index for cell/shard placement.

**Rejected:** Nested-set encoding (`lft`/`rgt`). It has excellent read performance but is
write-hostile — every tenant onboarding or tree modification requires a full subtree renumber.
The closure table delivers the same O(1) is-ancestor and O(n-subtree) all-descendants performance
without the write hostility (reparenting rewrites subtree closure pairs, not a full-table renumber).

**Rationale:** Every major precedent (AWS Organizations, GCP Resource Hierarchy, Azure Management
Groups, Salesforce, MSSP partner trees) uses adjacency-list-as-truth with a derived traversal index.
The closure table is the standard production composition for deep, write-active authorization
hierarchies. The optional materialized path serves only the cell/shard-placement optimization and
is never the authoritative record.

**Ties:** C9 Config-DB-Authoritative — the tenant tree lives in the Central control-plane PostgreSQL.
INV-ADS-04 — tree is authored only at Central UI, never at a satellite.

---

### D-C19-2 — Isolation Tier (Closes OQ-DEPLOY-1)

**Decision:** **Bridge by default**, with `isolation_tier` (enum: `pool | silo | cell`) as a
per-node tenant attribute on the tenant record. The isolation tier is orthogonal to the logical
hierarchy: a node's isolation tier does not need to match its parent's or children's.

**OQ-DEPLOY-1 → RESOLVED:** The question presupposed a single global isolation model. The correct
answer is bridge, with isolation tier as a per-tenant-node attribute. Prism is already a bridge:
the satellite data-plane is silo/cell-grade by construction (D-C2-12 — Central never stores raw
sensor data); the Central control-plane is pool-with-row-scoping. Nesting adds an `isolation_tier`
attribute; it does not require a new isolation mechanism.

**Mapping to Prism's stores:**

| Store | Default isolation | Per-node upgrade path |
|---|---|---|
| PostgreSQL (Central control-plane) | pool (row-level org_id) | dedicated schema (silo) or dedicated DB (cell) |
| RocksDB (satellite data-plane) | silo/cell by construction (one satellite = one footprint) | no change needed |
| Iceberg (cold RETAIN tier) | pool-with-partition + per-tenant encryption key | per-tenant table namespace for cell |
| Context stores (indradb/usearch/lancedb) | siloed at edge per PIV-C12-5 | no change |

---

### D-C19-3 — Config / Policy Inheritance (SF-1)

**Decision:** Hybrid inheritance model:

- **Config VALUES:** inherit-then-child-override. A child receives the parent's effective config
  as its default and may override any value within its tier. GCP IAM accumulate-flavor for usability.
- **Security GUARDRAILS:** intersect + parent-deny-is-final. A parent guardrail is a ceiling the
  child cannot widen. Any ancestor deny on a guardrail is final at all descendant nodes. AWS SCP
  flavor for safety — an MSSP parent can hard-cap a customer's capabilities.

**Effective config computation:** Computed at Central plan-time by folding the closure-table ancestor
set (root-to-leaf traversal applying the hybrid rule). Satellites receive the FLATTENED result; they
never perform inheritance computation. This keeps INV-ADS-04 (Config-DB-Authoritative) clean — no
physical object copying at the satellite layer.

**Ties:** C9 Config-DB-Authoritative / PIV-C9-001; INV-ADS-04.

---

### D-C19-4 — Depth Policy (SF-2)

**Decision:** Unlimited depth is supported technically (the closure table is depth-agnostic) plus a
**configurable soft cap**, default **8**, overridable per-deployment. The soft cap is a manageability
constraint, not an algorithmic one — vendor precedents (AWS OU 5, GCP folder 10, Azure 6) cap
for debuggability, not technical limits.

**Central UI requirements (regardless of soft cap):**
- Ancestor-chain breadcrumbs on every node view
- Inherited-vs-overridden config diff (GCP "view inherited policy" pattern)
- Effective-config preview at any node before saving a change

---

### D-C19-5 — RBAC Role Model Under Nesting

**Decision (C19 level; detailed role model = C18):** Role bindings take the form `(role, scope-node)`.
The closure table filters which scope-node bindings apply to a given resource. An admin holds bindings
on their home node; the closure table grants them reach over descendants that have **explicitly
consented via pairing** (D-C19-7). Effective grants = identity-policy ∩ permission-boundary; cannot
exceed.

**Confused-deputy guards:** Cross-tenant access is conditioned on source-org/source-path, so a parent
role can only reach its own subtree, never escalate to its own parent or to siblings.

**Cross-tenant parent→child MANAGE authority:** requires explicit child-admin-approved pairing
(Cortex-XDR model — parent sends request; child admin must approve; status stays Pending until
approved; persistent UI flag in child console shows who manages their config).

**Detailed role model (which roles exist, role-to-capability mapping) = C18 scope.** This decision
records only the structural form `(role, scope-node) filtered by closure table` as the C19-level
anchor.

---

### D-C19-6 — Metering Roll-Up (Ties C11)

**Decision:** Record every metered event (query, fan-out, retained-byte, action) at the **leaf** with
`org_id`. Aggregate up the closure-table ancestor edges on metadata **separate from data isolation**.
Billing roles are distinct from data/config roles.

**Mechanism:** The metering subsystem (C11) tags each event with the leaf `org_id` and walks the
closure table for roll-up. The parent's consolidated usage/cost report is a parent-scoped derived
metadata artifact — it does NOT require any read access to descendant raw data or keys. This is the
AWS Cost Categories / GCP org-billing model applied to the logical tenant tree.

---

### D-C19-7 — Reparenting Policy (SF-5)

**Decision:** Reparenting a tenant subtree is an **admin-only, audited, Central-authored** operation.
It requires:
1. Explicit re-pairing/re-consent for the new ancestor relationship (the moved subtree does not
   silently inherit the new parent's guardrails or key scope).
2. Recomputation of effective config for the entire moved subtree.
3. Explicit key-scope re-evaluation — the new ancestor's visibility grants do not automatically
   extend to the moved subtree's DEKs.

**Audited:** Full audit trail entry in the Central audit log including principal, old parent, new
parent, timestamp, and every tenant node affected.

---

### D-C19-8 — Key Custody Under Nesting (SF-4)

**Decision (per-child DEK, always):** The data-owning leaf tenant's DEK is the non-negotiable
invariant. A child's data is encrypted under the **child's** DEK, never a parent-derived key.
The Option-3 tenant-keyed cache key (PAT-ADS-02) is **always the data-owning child**, never the
parent.

**Configurable KEK topology (deployment choice):**
- **Option A — Flat per-tenant DEK:** Each tenant holds its own DEK directly under the KMS root.
  Simpler, suitable for most deployments.
- **Option B — Nested KEK hierarchy:** A per-sub-partner KEK wraps per-customer DEKs. Mirrors the
  logical tree for key-ops efficiency at large fan-out. Does NOT weaken per-child isolation — the
  leaf DEK is still unique per child, only its KEK wrapping changes.

Neither option changes the fundamental invariant: the leaf DEK is unique per child and is never
shared across tenants. Option B is a per-deployment operational choice, not an isolation tradeoff.

**MSSP default key custody:** CLIENT-HELD CMEK by default (SS-26 CMEK/HYOK). MSSP-custodied
SoftwareKms is an opt-down, not the default. Maximizes the zero-access differentiator even in the
MSSP-operated deployment model.

---

### D-C19-9 — Isolation Invariants Under Nesting

The following invariants are **unconditional**:

| ID | Invariant |
|---|---|
| PIV-C19-1 | No sibling bleed. Two children of the same parent never see each other's data — enforced by per-node org_id scoping + per-child keys, NOT by tree position. |
| PIV-C19-2 | Parent→child MANAGE authority requires explicit child-admin-approved pairing. Hierarchy position does NOT auto-grant management rights. |
| PIV-C19-3 | Option-3 cache key is always the data-owning child, never the parent. |
| PIV-C19-4 | The operator/vendor retains zero at-rest access to any tenant's data regardless of the tenant hierarchy. Parent-tenant != operator; INV-ADS-02 is unaffected by parent visibility grants. |
| PIV-C19-5 | Mechanism (c) — parent-as-additional-grantee on the child DEK — is FORBIDDEN (see AP-ADS-11). |
| PIV-C19-6 | The `regulatory_class` attribute can only tighten visibility/access grants; it can never loosen them regardless of parent/child preference. |

---

## 3 — SF-3 Parent Visibility: The Full Decision

### 3.1 The Core Principle

The correct dividing line for parent visibility is the **legal-entity boundary**, not the tree
position. Every market precedent where "parent sees everything below as if its own tenant" is raw
and position-implicit is an **intra-org (single legal entity)** deployment — AWS Organizations,
GCP Org Viewer, Azure mgmt-group Reader, Salesforce View All. Every cross-legal-entity product that
reaches raw data does so **consent-first** (Cortex XDR pairing, Azure Lighthouse delegation,
Microsoft Sentinel cross-workspace over delegated workspaces).

Prism supports the full spectrum but gates the upper end by `tenant_relationship`.

### 3.2 "Everything" Is Capped at the Derived Corpus

D-C2-12 / INV-ADS-01 is the hard ceiling. Raw sensor data NEVER leaves the edge. Prism's "transparent
subtree" = full visibility of the **derived corpus** the child surfaces to Central (query outputs,
findings, anomaly scores, GraphRAG summaries, conversation history) — NOT raw sensor rows.

This is a narrower (and correct) "everything" than the AWS/Sentinel/Salesforce precedents, which
expose raw telemetry. Prism cannot and must not match that breadth. The constraint is by invariant,
not by capability limitation.

### 3.3 Tenant Attributes

Three new tenant attributes govern visibility behavior:

| Attribute | Values | Effect |
|---|---|---|
| `tenant_relationship` | `same-legal-entity` \| `managed-client` \| `saas-customer` | Controls which presets the UI offers and what defaults apply |
| `isolation_tier` | `pool` \| `silo` \| `cell` | Per-node storage isolation (D-C19-2) |
| `regulatory_class` | `standard` \| `nerc_cip` \| `ot_critical` \| (extensible) | Overrides visibility presets and can force mechanism (d) or block P3 |

### 3.4 Visibility-Grant Matrix

The configurable dimensions:

- **Data-class (WHAT):** `derived_rows` (query outputs) | `findings_alerts` (detections, anomaly
  scores, advisories) | `config` (effective config view) | `audit` (child's audit trail) |
  `metering` (usage/cost rollup — metadata, lowest sensitivity)
- **Grant-scope (HOW):** `per_child` (explicit per-child grant) | `per_tier_policy` (policy that
  auto-applies to children of a given relationship tier) | `per_data_class` (toggle each data-class
  independently)
- **Default posture:** OFF for everything except `metering` rollup. `metering` defaults ON because
  it operates on metadata, is separate from data isolation, and is the minimum viable MSSP reporting
  surface.

### 3.5 Preset Ladder

| Preset | Data-classes granted | Key-custody mechanism | Offered for `tenant_relationship` = |
|---|---|---|---|
| **P0 Metering-only** (default everywhere) | `metering` rollup only (metadata) | metadata aggregation, no decryption required | all |
| **P1 Consented derived metrics** | `findings_alerts` summaries + `metering`; consent-gated | (b) re-encrypt under parent DEK | `managed-client`, `same-legal-entity` |
| **P2 Operational visibility** | `derived_rows` + `findings_alerts` + `config`; consent-gated; persistent child-side indicator | (a) transient in-query + (b) for persisted artifacts | `managed-client` (with pairing), `same-legal-entity` |
| **P3 Transparent subtree** | ALL derived data-classes including `audit`, across the subtree | (b) persisted + (a) live; (d) for OT/NERC-CIP | **`same-legal-entity` ONLY; `regulatory_class` override can block** |

P3 is the "parent sees all below as if its own tenant" preset. It is reachable **only** when
`tenant_relationship = same-legal-entity` and the node is not regulatory-blocked.

### 3.6 Key-Custody Composition Table

| Mechanism | What it does | INV-K1 (never-share-DEK) | INV-K2 (operator-zero-access) | Verdict |
|---|---|---|---|---|
| **(a) Consent-scoped transient decryption under the CHILD key** | Parent triggers query; data decrypts under child DEK for query duration only; parent never receives/stores the DEK; child grants + can revoke; every access audited child-side | ALLOW for in-query only — never persist the result under any key but the child's | PRESERVED (operator excluded) | **ALLOWED for live transparent-view queries only; never persist** |
| **(b) Re-encrypt result under the PARENT DEK** | KMS ReEncrypt transforms ciphertext from child key to parent key inside the KMS boundary; parent reads data under its OWN key | PRESERVED (cleanest) — the parent's copy is its own keyed artifact; no DEK shared | PRESERVED (operator excluded) | **PREFERRED for any PERSISTED parent-visible artifact** |
| **(c) Parent-as-additional-grantee on the child DEK** | Child adds parent's service account as a persistent KMS grantee on the child key | BROKEN — persistently extends decryption rights across tenants; violates INV-ADS-03 | Preserved re: operator, but the cross-tenant key sharing is the violation | **FORBIDDEN — see AP-ADS-11** |
| **(d) BYOC remote-op + HYOK + TEE** | Parent runs bounded query inside child's cloud; keys never leave the child | PRESERVED (parent never holds keys) | STRONGEST | **PREFERRED for OT / NERC-CIP / highest-assurance deployments** |

**Recommended Prism composition:**
- Persisted parent-visible artifacts → mechanism (b) re-encrypt under parent DEK
- Live transparent-view queries → mechanism (a) transient decryption under child key, scoped to query
- OT / NERC-CIP / highest-assurance client-managed → mechanism (d) BYOC remote-op
- Mechanism (c) → **FORBIDDEN** (AP-ADS-11)

### 3.7 Consent and Governance Requirements

For any preset above P0, ALL FIVE controls are required:

1. **Child-admin approval (pairing).** Parent requests; child Admin must approve. Status stays
   Pending until approved. The managed-service onboarding IS the consent ceremony for P2 in the
   MSSP-managed model — this is the legitimate path, not an exception.
2. **Persistent surfaced indicator in the child console.** The child console always shows that a
   parent has visibility and who. Non-negotiable for the trust story.
3. **Contractual / relationship flag.** The `tenant_relationship` attribute plus a recorded
   contractual basis gates which presets are offered. Distinct legal entities require explicit
   contractual basis; same-legal-org records the internal authorization.
4. **Granular revocation.** The child (or, in regulated contexts, a mandated authority) can revoke
   any data-class grant. Revocation must immediately stop future access. For mechanism (a),
   revocation = revoke the KMS grant. For mechanism (b), existing parent-keyed copies are the
   parent's data — define retention/deletion policy explicitly.
5. **Complete child-side audit trail.** Every parent access is logged child-side with parent-principal
   attribution at the APPLICATION layer, independent of KMS log granularity.

### 3.8 Regulatory Class Override

A `regulatory_class` attribute (e.g., `nerc_cip`, `ot_critical`) can:
- **Force P3 OFF** regardless of parent/child preference
- **Force mechanism (d) BYOC remote-op** as the only permitted path for derived-corpus sharing
- **Heighten audit** requirements for all access above P0

The override can **only tighten, never loosen** visibility or access. This applies when a regulator
may treat parent decryption of BES/OT data as a change in who "controls" critical-infrastructure
data, potentially extending CIP obligations. Consent may not be purely at the child's discretion
in regulated contexts. Ties C20 (OT/NERC-CIP).

**Compliance-Profile integration (ADR-PROP-compliance-profiles.md, D-PROF-6).** `regulatory_class`
is now realized as a **Compliance-Profile SELECTOR / FLOOR** via the profile engine (PAT-ADS-12),
not as bespoke regulatory logic in the tenancy code:

| `regulatory_class` value | Forced profile floor | Effect |
|---|---|---|
| `standard` | `≥ baseline` | Normal tighten-only inheritance; no additional forcing |
| `ot_critical` | `≥ iec-62443-ot` | Node + all descendants cannot drop below `iec-62443-ot` |
| `nerc_cip` | `≥ nerc-cip` | Node + all descendants cannot drop below `nerc-cip` |

The forced profile is a floor the node and its descendants cannot drop below — tighten-only, matching
the parent-deny-is-final semantics of D-C19-3 applied to profile settings. The `regulatory_class`
attribute SELECTS the floor; the ENFORCEMENT lives in the generic profile engine (PAT-ADS-12) rather
than as bespoke regulatory branches in the tenancy or authz code. This removes any
`if regulatory_class == nerc_cip { ... }` branches from the codebase.

The **behavioral semantics are unchanged** by this reframe: `nerc_cip` still forces P3 OFF and
forces mechanism (d) BYOC remote-op via the `nerc-cip` profile's `[settings.parent_visibility]
max_preset = { lock = "P0" }` and `[settings.key_custody] require_mechanism = { lock = "d" }` axes.
What changes is the implementation path — profile engine, not bespoke code branch — which conforms
to P-ADS-11 (Single-Codebase) and AP-ADS-07 (No-Deployment-Model-Code-Forks). See
`ADR-PROP-compliance-profiles.md` D-PROF-6 for the full profile-selector/floor decision record.

---

## 4 — MSSP Reconciliation

### 4.1 What "Operator-Zero-Access-At-Rest" Actually Means

P-ADS-02 / INV-ADS-02 (Operator-Zero-Access-At-Rest) is an **UNMEDIATED-AT-REST** guarantee: the
operator's own infrastructure keys cannot decrypt at rest. This is what blocks:
- Rogue infra access
- Raw disk / backup access
- Subpoena-to-vendor (operator is served, but does not hold the client's key)
- Non-onboarded-client access

It is **NOT** a rule that "no human at the MSSP ever sees client data." Authorized mediated access
by MSSP analysts is a separate, legitimate, governed path.

### 4.2 The Authorized Mediated Access Path

A named analyst identity → authenticated Central session (Central-Sole-Surface per P-ADS-01) →
RBAC-scoped to delegated clients (via `(role, scope-node)` bindings) → decryption under the
CLIENT's tenant DEK via a mediated, AUDITED path (mechanisms a or b) → client sees a persistent
indicator + can revoke.

The managed-service onboarding/delegation IS the consent. This is the MSSP value proposition —
authorized, governed, audited access — not a grudging carve-out from a zero-access principle.

### 4.3 Standard MSSP Operating Posture

**P2 Operational visibility** (derived_rows + findings_alerts + config) via per-client delegation
with full trust controls is the MSSP-managed standard operating posture. This differs from P3
transparent-subtree in two ways:
- **Legal basis:** P2 is cross-legal-entity consent (clients are distinct organizations); P3 is
  same-legal-entity governance (the org sees its own divisions).
- **Consent ceremony:** P2 requires explicit per-client delegation/pairing; P3 is an
  organizational-authority grant across owned subsidiaries.

The raw capability (see derived data across clients) is the same at P2 and P3. The governance
and legal basis differ.

### 4.4 Operator-Zero-Access Is a Spectrum

| Operating model | Zero-access character |
|---|---|
| **Client-managed / BYOC** | Strongest — client holds keys; operator CANNOT decrypt regardless of circumstance |
| **SaaS** | Vendor sees ciphertext at rest; tenant-keyed CMEK (P-ADS-04) means vendor cannot decrypt without the tenant's key |
| **MSSP-managed** | Key separation (per-client DEK/CMEK) + audited mediated access (mechanisms a/b) + per-client key isolation. Not cryptographic impossibility of analyst access — cryptographic proof that analyst access is authorized, attributable, and revocable |

The MSSP-managed model is the honest middle of the spectrum: the zero-access guarantee is meaningful
(no passive/silent/unauthorized access to raw data at rest), and authorized access is governed
(RBAC-scoped, audited, consent-based, revocable).

### 4.5 Default Key Custody for MSSP-Managed Deployments

Default = **CLIENT-HELD CMEK** (SS-26 CMEK/HYOK). The MSSP-custodied SoftwareKms is an opt-down.
This design choice maximizes the zero-access differentiator even in the MSSP-operated model: the
MSSP runs the infrastructure but the client holds the root of trust by default.

---

## 5 — New Tenant Attributes Summary

| Attribute | Type | Required | Default | Semantics |
|---|---|---|---|---|
| `parent_id` | `Option<OrgId>` | No | None (root tenant) | Authoritative adjacency-list pointer; `None` = top-level |
| `isolation_tier` | `enum {pool, silo, cell}` | Yes | `pool` | Per-node storage isolation level (orthogonal to tree position) |
| `tenant_relationship` | `enum {same-legal-entity, managed-client, saas-customer}` | Yes | `saas-customer` | Conditions preset ladder availability and default posture |
| `regulatory_class` | `enum {standard, nerc_cip, ot_critical}` | Yes | `standard` | Override that can only tighten visibility and access grants |

---

## 6 — Proposed Epics

**E-NESTED-TENANCY-001** (PROPOSED — not yet in STORY-INDEX): Implement the nested tenant tree.
Covers: `parent_id` on OrgId node; closure table schema + maintenance triggers; `isolation_tier`,
`tenant_relationship`, `regulatory_class` attributes; effective-config fold computation at Central;
satellite receives flattened config bundles; Central UI: ancestor breadcrumbs, inherited-vs-overridden
diff, effective-config preview.

**E-TENANT-VISIBILITY-001** (PROPOSED — not yet in STORY-INDEX): Implement the configurable
visibility-grant matrix. Covers: preset ladder (P0..P3); pairing request/approval workflow; persistent
child-side indicator; per-data-class grant UI; mechanism (b) re-encryption path for persisted
parent-visible artifacts; mechanism (a) transient decryption path for live queries; `regulatory_class`
override enforcement; consent audit trail.

Both epics are PROPOSED. Neither is registered in STORY-INDEX. Registration gated on morph execution
(post-demo, post-T14, brief-reframe sign-off §5.1).

---

## 7 — Cross-Wiring with Sibling Features

These are forward cross-links to be kept mutually consistent. They do not pre-decide those items.

| Feature | Cross-wire point |
|---|---|
| **C18 RBAC** | Role bindings = `(role, scope-node)` filtered by closure table. Detailed role names and capabilities = C18 scope. Permission-boundary + confused-deputy = C19 structural foundation for C18 to build on. |
| **C16 detokenize-at-surface** | Token/masked-field resolution at the Central surface is RBAC-scoped. C19's `(role, scope-node)` structure means C16's resolution paths must respect closure-table-scoped bindings. |
| **C15 approver roles in gated-action path** | The gated-action approval workflow (D-C15-6, P-ADS-10) uses approver roles. These roles are `(role, scope-node)` bindings filtered by the closure table — C19 defines the structural form; C15 defines which role IDs are approvers. |
| **C17 key-escrow / Option-3 CMEK** | SS-26 per-tenant DEK is the mechanism C19 relies on for per-child DEK isolation and mechanisms (a)/(b). Nested KEK hierarchy (D-C19-8 Option B) is an operational extension to SS-26 that C17 must accommodate. |
| **C11 metering** | Metering roll-up uses the same closure table as auth (D-C19-6). C11 must tag metered events with `org_id` and implement the ancestor-walk aggregation; C19 provides the structural contract. |
| **C9 config inheritance** | Config-DB-Authoritative (INV-ADS-04) is preserved: C19's effective-config fold runs at Central, output pushed as flattened signed bundles to satellites. The fold algorithm (D-C19-3) is the C9 config-inheritance mechanism at the hierarchy layer. |
| **C2 satellite headless / residency** | D-C2-12 (Central never stores raw sensor data) is the hard ceiling for all of C19. The satellite data-plane's silo/cell-grade isolation by construction is what makes the bridge-by-default decision (D-C19-2) correct. Reparenting (D-C19-7) does not change satellite-level residency enforcement. |
| **C20 regulatory_class / NERC-CIP** | `regulatory_class` tenant attribute (D-C19, §3.8) ties C20's OT/NERC-CIP requirements. C20 defines which `regulatory_class` values exist and their enforcement semantics. C19 provides the attribute and the override rule (can only tighten). |

---

## 8 — ADS Conformance Checklist

```
CONFORMANCE CHECKLIST — ADR-PROP-nested-tenancy.md (C19) — 2026-06-27

P-ADS-01: Central-Sole-Surface
  [YES] Every user-interaction path terminates at Central. The tenant tree and
        visibility grants are authored and administered at the Central UI/DB.
  [YES] Satellites are strictly headless. C19 adds no satellite-side user surface.
        Effective config is pushed as a flattened signed bundle; satellites auto-receive.

P-ADS-02: Operator-Zero-Access-At-Rest
  [YES] All derived results persisted at Central are encrypted under the child tenant's
        DEK (SS-26 per-tenant DEK). Parent-visible PERSISTED artifacts are re-encrypted
        under the parent DEK (mechanism b) — a separate keyed artifact. The operator
        holds ciphertext only.
  [YES] Operator has zero unmediated at-rest read access. The MSSP reconciliation (§4)
        clarifies that authorized mediated ANALYST access is a separate governed path
        and does not violate P-ADS-02 because the operator's infrastructure keys do not
        decrypt — the tenant's DEK (held by the client by default) is used.

P-ADS-03: Derived-Results-Only-At-Central
  [YES] "Everything" in P3 transparent-subtree is capped at the DERIVED corpus
        (D-C2-12 / INV-ADS-01). Raw sensor data NEVER leaves the edge. The visibility
        model does NOT replicate the AWS/Sentinel raw-telemetry breadth.
  [YES] No opt-in path where raw identifiers transit to Central is introduced by C19.
        The metering rollup operates on metadata. Derived-results-only is preserved.

P-ADS-04: Tenant-Keyed-Central-Persistence
  [YES] Option-3 tenant-keyed cache is preserved. The cache key remains the data-owning
        child (PIV-C19-3). Mechanism (b) re-encryption creates a SEPARATE parent-keyed
        artifact under the parent's DEK — the child's cache is untouched.
  [YES] No "forensic replay" confusion introduced. C19's visibility artifacts are
        derived outputs, not inputs. OQ-C8-DATASNAPSHOT distinction is preserved.

P-ADS-06: Per-Tenant-Isolation
  [YES] Per-child DEK is the non-negotiable invariant (D-C19-8). No DEK is shared across
        tenants. Mechanism (c) parent-as-grantee is explicitly FORBIDDEN (AP-ADS-11).
  [YES] Closure table enforces scope boundaries. No cross-tenant joins are introduced.
        Sibling bleed is impossible by construction (PIV-C19-1).

P-ADS-07: AI-Opaque
  [YES] No AI/ML components are introduced by C19. Visibility grants operate at the
        data-plane/key-custody layer. The AI-opaque invariant (AD-017) is unaffected.
  [YES] Credentials are not involved in the visibility grant mechanisms.

P-ADS-08: OCSF-Normalize-At-Boundary
  [YES] C19 does not introduce new data sources. Derived corpus arriving at Central
        is already OCSF-normalized by the satellite boundary (D-C2-12, P-ADS-08).
        The visibility-grant matrix operates on already-normalized derived results.

P-ADS-09: Config-DB-Authoritative
  [YES] The tenant tree, isolation tiers, tenant-relationship attributes, visibility
        grants, and regulatory-class overrides are ALL authored at the Central UI/DB.
        No satellite, CLI file, or git-committed TOML authors these values.

P-ADS-10: Idempotent-Gated-Actions
  [YES] Reparenting (D-C19-7) is an admin-only, audited, Central-authored operation.
        It is a write action that will pass through the gated-action path (C15/P-ADS-10).
  [YES] Pairing requests and approval are explicit, recorded, revocable — consistent
        with the idempotent-gated-actions pattern.

INV-ADS check (all ten):
  [YES] INV-ADS-01: No raw sensor data at Central — hard ceiling on "everything" (§3.2)
  [YES] INV-ADS-02: Operator zero-access at rest — MSSP reconciliation clarifies scope (§4)
  [YES] INV-ADS-03: Per-tenant isolation — per-child DEK; (c) forbidden (AP-ADS-11)
  [YES] INV-ADS-04: Config authored only at Central — tree + grants = Central UI/DB
  [YES] INV-ADS-05: Actions gated — reparenting + pairing approval pass through gate
  [YES] INV-ADS-06: AI-opaque — no AI components introduced by C19
  [YES] INV-ADS-07: OCSF normalization — no new data sources; derived corpus already normalized
  [YES] INV-ADS-08: Air-gap valid — SoftwareKms + signed bundles + flat/nested DEK both work
                    offline; P2/P3 presets function in air-gap with local-KMS key operations
  [YES] INV-ADS-09: Decision-level audit — C19 introduces no new authorization engine; existing
                    decision-log obligation (from C18/INV-ADS-09) applies to tree-traversal
                    auth decisions; C19 adds reparenting + pairing approval events to the audit
                    trail (D-C19-7; §3.7 governance requirement 5). Not violated by C19.
  [YES] INV-ADS-10: Recoverability preserves operator-zero-access — per-child DEK (D-C19-8) is
                    backed up via SS-26 sealed-blob escrow (PAT-ADS-16); the nested KEK hierarchy
                    (Option B) similarly uses operator-unwrappable blobs. Crypto-shred applies at
                    the leaf DEK level; operator cannot recover tenant data unilaterally. Not
                    violated by C19.
```

All checklist items PASS. C19 is ADS-conformant.
(Note: INV-ADS-09 and INV-ADS-10 were added to the ADS after C19 was originally captured. Both
are satisfied by the decisions recorded above; this checklist has been updated to reflect the
current ADS v1.6 invariant set of ten.)

---

## 9 — Open Questions

| ID | Question | Owner | Priority |
|---|---|---|---|
| OQ-C19-1 | Closure-table maintenance triggers: on-write (synchronous, consistent) vs async background recompute for very deep subtrees under high onboarding load | architect at morph | P1 |
| OQ-C19-2 | Exact PostgreSQL schema for the closure table (indexes, partitioning, update procedure) | data-engineer at morph | P1 |
| OQ-C19-3 | Materialized path lazy-derivation trigger: event-driven vs periodic | architect at morph | P2 |
| OQ-C19-4 | P3 transparent-subtree UX: how the Central UI presents "you are seeing this child's data as your own" to the parent-tenant analyst | ux-designer at morph | P2 |
| OQ-C19-5 | Retention / deletion policy for parent-keyed re-encrypted artifacts (mechanism b) after the child revokes the visibility grant | architect at morph | P1 |
| OQ-C19-6 | Nested KEK hierarchy (D-C19-8 Option B): SS-26 extension design to support per-sub-partner KEK wrapping per-customer DEKs | data-engineer + architect at morph | P2 |

---

## 10 — Decision Provenance

| Decision ID | Sub-fork | Human decision |
|---|---|---|
| D-C19-1 | SF-None (tree representation) | adjacency + closure table + optional materialized path |
| D-C19-2 | OQ-DEPLOY-1 | bridge by default; isolation_tier as per-node attribute; OQ-DEPLOY-1 CLOSED |
| D-C19-3 | SF-1 (inheritance semantic) | hybrid: values inherit-then-override, guardrails intersect+parent-deny-final |
| D-C19-4 | SF-2 (depth policy) | unlimited technically; configurable soft cap default 8 |
| D-C19-5 | SF-None (RBAC model) | (role, scope-node) + closure table; detailed role model = C18 |
| D-C19-6 | SF-None (metering) | leaf-record + closure-table ancestor roll-up; C11 integration |
| D-C19-7 | SF-5 (reparenting) | admin-only, audited, Central-authored; explicit re-pairing/re-consent |
| D-C19-8 | SF-4 (key custody) | per-child DEK always; flat-DEK or nested-KEK configurable per-deployment; MSSP default = client-held CMEK |
| D-C19-9 | Isolation invariants | PIV-C19-1..6 confirmed |
| SF-3a | Upper bound of visibility spectrum | P3 transparent-subtree SUPPORTED; gated by tenant_relationship = same-legal-entity |
| SF-3b | Persisted-visibility mechanism | (b) re-encrypt under parent DEK for persisted; (a) transient for live; (d) for OT; (c) FORBIDDEN |
| SF-3c | Visibility-grant matrix vs toggle | Matrix with safe defaults (OFF except metering) |
| SF-3d | OT / regulatory_class override | regulatory_class can force P3 OFF or force mechanism (d); override can only tighten |
| MSSP-REC | MSSP reconciliation | P-ADS-02 is unmediated-at-rest guarantee; authorized mediated access is the governed MSSP path; P2 = standard operating posture; default key custody = client-held CMEK |
