---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C18-1: Layered authz model — RBAC + ReBAC (Zanzibar-tuple core) + ABAC"
  - "ADR-PROP-C18-2: SF-1 authz engine — Build Prism-native Rust Zanzibar-tuple core"
  - "ADR-PROP-C18-3: Resource scoping — connector/satellite/source/table first-class; columns via ABAC tags"
  - "ADR-PROP-C18-4: Hierarchy enforcement — strictly-downward inheritance; escalation-up prevention as tested invariant"
  - "ADR-PROP-C18-5: Action-tier authz — required_approver_role + min_approvals + backend SoD primitives"
  - "ADR-PROP-C18-6: Central-authored / edge-enforced; revocation-lag via Compliance Profile (see ADR-PROP-compliance-profiles.md)"
  - "ADR-PROP-C18-7: Decision-level audit log"
  - "ADR-PROP-C18-8: IdP group → Prism internal role mapping; SCIM deprovisioning as fast-path revocation"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Day-2 out-of-band SIDE-ANALYSIS capture; human-confirmed decisions 2026-06-27 session.
  Research basis: research/rbac-depth-2026-06-27.md (PRIMARY — three perplexity_research
  sonar-deep-research calls at reasoning_effort=high covering: (1) RBAC/ABAC/ReBAC +
  Zanzibar + SpiceDB/OpenFGA + OPA/Oso/Casbin/Gatehouse + Rust-native/air-gap; (2)
  action-tier SoD/four-eyes/maker-checker/JIT-PAM/break-glass/IEC-62443 + resource/field-level
  granularity + OCSF gap; (3) central-authored/edge-enforced + OPA bundles + ZedToken/Zookie/
  New-Enemy + air-gap offline PDP/PEP + NERC CIP-004/005 + SOC2 CC6.x + ISO 27001 A.9)
  + research/configurable-security-profiles-2026-06-27.md (SF-3 generalized into Compliance
  Profiles, referenced as ADR-PROP-compliance-profiles.md).
  This is an out-of-band side-analysis. CAPTURE ONLY. Does NOT modify any live spec,
  ADR-registry artifact (specs/architecture/), BC, story, STATE.md, or SESSION-HANDOFF.md.
  No git operation performed. Real ADR numbers and formal ARCH-INDEX.md rows deferred to
  the morph execution. touches_no_live_artifacts: true
seeded_from:
  - research/rbac-depth-2026-06-27.md
  - research/configurable-security-profiles-2026-06-27.md
cross_refs:
  - specs/day2-design-decisions/ADR-PROP-nested-tenancy.md (C19 — role model structural form, closure table, regulatory_class)
  - specs/day2-design-decisions/ADR-PROP-compliance-profiles.md (SF-3 revocation-lag / deny-on-stale generalized; OT = profile)
  - specs/day2-design-decisions/ADR-PROP-config-management.md (C9 — Config-DB-Authoritative, signed bundles, canary)
  - specs/day2-design-decisions/ADR-PROP-prism-context.md (C16 — ABAC masking = detokenize-at-surface enforcement point)
  - specs/day2-design-decisions/ADR-PROP-soar-actions-aro.md (C15 — ARO, required_approver_role, action-classes)
  - specs/day2-design-decisions/secret-subsystem-sketch.md (C17 — key-escrow ops governed by RBAC approver roles)
  - specs/day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md (P-ADS-09/11/12/13; AP-ADS-07; PAT-ADS-03/10; INV-ADS-01..08)
  - matured-vision-day2-requirements.md §16.4 (C18 RBAC; C10 Query.io no-RBAC differentiator)
---

# ADR-PROP — Robust RBAC Depth: Layered Authz Extending Into Connectors, Sources, Tables, and Actions (C18)

> **STATUS: DECIDED 2026-06-27 (human).** Full decision record for C18 — robust RBAC that
> extends into connectors, satellites, sources, tables/columns, and actions. Covers: layered
> authz model, engine choice, resource scoping, hierarchy enforcement, action-tier SoD,
> central-authored/edge-enforced policy, revocation-lag posture (generalized to Compliance
> Profiles), decision-level audit, and IdP linkage. CAPTURE artifact (`do_not_execute: true`).
> Real ADR numbers and formal ARCH-INDEX.md rows deferred to morph execution.

---

## 1 — Context and Scope

Prism must provide authorization depth commensurate with an MSSP's client-facing obligations:
NERC CIP-004/005, SOC2 CC6.x, ISO 27001 Annex A.9, and IEC 62443 all impose access-control,
least-privilege, audit, and revocation requirements that cannot be met with a simple flat-roles
model. Prism's differentiator vs Query.io is explicit: Query.io ships no in-product RBAC
(C10 — [VERIFIED-WEB]). C18 depth — per-connector/source/table scoping, hierarchy inheritance
across nested tenancy (C19), approver-gated actions (C15), central-authored/edge-enforced policy,
and decision-level audit — is therefore a clean, defensible differentiator that also directly
satisfies critical-infrastructure and SOC2/ISO buyer requirements.

The existing codebase provides a flat per-tenant RBAC surface. C18 adds depth without replacing
the flat surface: the flat human-role surface (analyst / lead / admin / approver) becomes the
RBAC tier; the resource hierarchy (tenant → connector → source → table) becomes the ReBAC tier;
dynamic field masking and conditions become the ABAC tier.

---

## 2 — Decisions

### D-C18-1 — Layered Authz Model

**Decision:** Prism adopts a **layered model: RBAC + ReBAC + ABAC.** No single paradigm
suffices for C18's requirements:

| Tier | Primary abstraction | Use in Prism |
|------|--------------------|----|
| **RBAC** | Roles → permissions | Human-facing role surface: `analyst / lead / admin / approver`. Maps to MSSP org structure; satisfies SOC2 CC6.x / ISO A.9 / NERC CIP-004 auditor expectations. **[VERIFIED-WEB]** |
| **ReBAC** (Zanzibar-style) | Relationship tuples + usersets | Tenant → sub-tenant → connector → source → table resource hierarchy. Hierarchy is a relationship graph; inheritance propagates along edges. This is the natural model for C19 nested-tenancy requirements. **[VERIFIED-WEB]** |
| **ABAC** | Attributes of subject/resource/environment | Column/field masking via `sensitivity:PII` tags; row filters; dynamic conditions (time, environment). ABAC is the correct tool for "role may not see column Z" without per-column roles. **[VERIFIED-WEB]** |

**Key insight (C18/C19):** A tenant → sub-tenant → connector → source → table chain IS a
relationship graph. ReBAC models it natively; RBAC and ABAC both must encode the hierarchy
indirectly (RBAC: role-per-level → role explosion; ABAC: parent-id attributes → brittle
traversal). ReBAC is the lower-defect path for the hierarchy layer. **[VERIFIED-WEB]**

**Relationship to C19:** C18's ReBAC resource graph GENERALIZES C19's closure-table tenant
slice. The closure table (D-C19-1) is the materialized tenant-tree projection of the broader
relationship graph. When C18 implements the relationship-tuple core, it extends naturally to
the full resource graph (connector / source / table / action-class nodes below the tenant
tier). C19's `(role, scope-node) + closure table` structural form (D-C19-5) is the C19-tier
slice of C18's full resource graph; C18 adds the sub-tenant-to-leaf-resource edges.

---

### D-C18-2 — Authz Engine Choice (SF-1 DECIDED)

**Decision (a): BUILD a Prism-native Rust Zanzibar-tuple core, in-process, embeddable at
headless satellites.**

The core implements:
- Zanzibar relationship-tuple data model (object=`type:id`, relation, userset)
- Namespace configuration (object types, relations, permission→relation composition)
- In-process evaluation (no separate process at each satellite)
- Offline tuple replication from Central (rides PAT-ADS-03 signed bundle mechanism)
- New-Enemy-Problem mitigation (consistency tokens + version-stamped decisions; see D-C18-6)
- Reuses C9 Config-DB authority for tuple storage at Central (INV-ADS-04)
- Reuses C19 closure table as the materialized tenant-tier traversal index (D-C19-1)

**Considered-and-rejected alternatives (documented FALLBACKS, ordered by maturity):**

| Alternative | Assessment | Rejection rationale |
|-------------|-----------|---------------------|
| **(b) `casbin-rs` v2.20.0** (multi-model, actively maintained 2026-02-04) **[CRATES.IO-VERIFIED]** | Most mature Rust-native option; hierarchical-RBAC + ABAC supported; `conf`-file model is flexible | PERM metamodel expresses rules as flat conf files rather than a typed resource graph — per-connector/source/table scope requires significant ceremony; not the natural home for the Zanzibar relationship-tuple semantics C18 needs |
| **(b) `opa-wasm` v0.2.1** (Rego-compiled-to-WASM in-process) **[CRATES.IO-VERIFIED]** | Great air-gap story; OPA bundles = the exact C9/C18 signed-bundle distribution model; strong offline enforcement | Rego is the policy language (not Prism-native); every policy change requires a Rego compilation step; the OPA bundle delivery IS the right pattern (C18 rides it for policy bundles per D-C18-6) but the evaluation engine is a WASM wrapper around a Go compilation artifact — in-process overhead and Rego DSL maintenance are concerns |
| **(b) `gatehouse` v0.5.0** (composable RBAC+ABAC+ReBAC, 2026-06-27) **[CRATES.IO-VERIFIED]** | Natively unifies all three models in-process in Rust; updated same day as this research | Very small ecosystem; no evidence of production hardening at MSSP/enterprise scale; v0.5.0 is pre-1.0 |
| **(b) `oso` v0.27.3** (Polar language, in-process ReBAC) **[CRATES.IO-VERIFIED]** | OSS crate is in-process Rust; Polar supports RBAC/ABAC/ReBAC; ~890k downloads | OSS crate last published 2024-01-13; vendor focus has shifted to Oso Cloud (SaaS). Maintenance cadence of the OSS crate is a maintenance risk for a security-critical dependency. **[INCONCLUSIVE — model knowledge, flagged]** |
| **(c) SpiceDB or OpenFGA as Go sidecar** | Most battle-tested Zanzibar semantics + ZedToken/Zookie consistency tokens; `spicedb-rust` v0.3.4 (2024-12-01) **[CRATES.IO-VERIFIED]** | Both are **Go services**, not embeddable Rust crates. Using them at headless satellites = operating a co-located sidecar process per node. Prism air-gap / OT deployments: one extra process per satellite edge node, no in-process linking. Documented offline edge replication for Zanzibar tuples was **not found** in SpiceDB/OpenFGA docs — offline edge tuple replication would need to be built anyway **[INCONCLUSIVE on offline replication]**. Weakest air-gap story of the three options. |

**Why build:** (a) Prism already owns Config-DB authority (C9) — the tuple store is a natural
extension; (b) offline edge replication must be built regardless of engine choice (no engine
documents this for offline satellites); (c) in-process = no per-satellite sidecar; (d) the
Zanzibar data model is well-specified (Google 2019 paper) so "build" is engineering to a
published spec, not inventing a novel model.

**Fallback routing (morph decision):** If build cost analysis at morph exceeds the estimated
engineering investment, the ordered fallback is: `casbin-rs` → `opa-wasm` (evaluation only,
keep the Prism-native tuple store for relationship data) → `gatehouse`. `oso` is the last
resort given maintenance risk. `casbin-rs` and `opa-wasm` both have the strongest case as
adopt-vs-build fallbacks because they are the most actively maintained.

---

### D-C18-3 — Resource Scoping (SF-2 DECIDED)

**Decision:** Prism's first-class authz resources are:

| Resource type | Authz role |
|--------------|-----------|
| `tenant` / `sub-tenant` | C19 hierarchy anchors; top of the resource graph |
| `satellite` | Scope "role may operate satellite S" |
| `connector` | Sensor adapter (e.g., `connector:crowdstrike`) |
| `source` | A specific configured connector instance for a tenant |
| `table` | OCSF-normalized table / schema surface |
| `action-class` | C15 ARO actions — destructive, write, OT-affecting |

**Column/field granularity:** Columns are NOT first-class role-scoped resources. They are
governed by **ABAC sensitivity tags + masking policies** applied at catalog/schema level.
One policy covers all `sensitivity:PII` columns rather than N per-column roles (avoids role
explosion). **[VERIFIED-WEB — Databricks Unity Catalog pattern; Microsoft Power Platform]**

**Granularity ceiling:** "Role may not see column Z" = ABAC tag. "Role may not query source Y"
= ReBAC relation. "Role may not run action-class destructive.disable_account" = RBAC + approver
gate. These three mechanisms compose; they do not collapse into one.

**PII column posture (CONFIGURED, not hardcoded):**
- PII columns masked-by-default in the `baseline` Compliance Profile (ADR-PROP-compliance-profiles.md).
- Bulk export of `sensitivity:PII` columns is **hard-blocked by default** in `baseline` (visibility
  ≠ capability — critical distinction: seeing a masked field ≠ bulk-exporting the column).
- Both masking strictness and bulk-export posture are **Compliance Profile settings** (axes:
  `settings.masking.bulk_export`). They can be loosened within profile-permitted bounds for
  deployments where bulk analytics on PII is a governed, consented use case. Conforms to
  P-ADS-13 Configurable-Not-Prescriptive.

**Per-query enforcement:** Prism is a query engine; the query planner is the natural Policy
Enforcement Point (PEP). Every PrismQL query is decomposed into source/table/column accesses
at plan time; each is checked against policy before fan-out. This also governs query shape
(aggregations that could re-identify masked data). **[VERIFIED-WEB — Unity Catalog object-
attached policy pattern]**

**OCSF field-level gap:** OCSF has no explicit field-level authorization guidance in public
docs. Prism must maintain its own PII/sensitivity tagging over OCSF fields — OCSF normalization
defines the schema; Prism's ABAC tag layer defines per-field access control. **[VERIFIED-WEB,
gap noted; OCSF is not the source of this metadata]**

---

### D-C18-4 — Hierarchy Enforcement (C19 consistency)

**Decision:** Role bindings take the form `(role, scope-node)` filtered by the C19 closure
table (reusing D-C19-5 directly). Inheritance is **strictly downward**:

- A permission granted at `tenant:acme` flows down to every sub-tenant / connector / source
  under it without requiring enumeration of each child.
- `sub-tenant:acme-east` admin bindings are anchored at that subtree root. They cannot reach
  sibling `sub-tenant:acme-west` or the parent `tenant:acme`.
- **Escalation-UP is a tested authz-schema invariant**, not merely a documentation rule.
  The schema's arrow operators are defined so that parent→child edges are used only to
  resolve inherited grants from parent to child, never to grant a child principal authority
  over the parent. This invariant must be expressed as an explicit test in the authz schema
  test suite (analogous to SpiceDB "assertions / expected-relations"; Prism authz-schema
  test in the Rust test suite). **[VERIFIED-WEB — Zanzibar escalation-prevention semantics]**

**Permission-boundary semantics:** Effective grants = identity-policy ∩ permission-boundary.
A parent admin cannot gain access beyond their own permission boundary regardless of
relationship-tuple composition.

**Confused-deputy guards:** Cross-tenant access is conditioned on source-org/source-path.
A parent role reaches only its own subtree (explicitly consented via C19 pairing, D-C19-7).
Sibling-reach is prevented by construction of the relation schema.

**Cross-tenant parent→child MANAGE authority:** requires explicit child-admin-approved pairing
(Cortex-XDR model per D-C19-5). The pairing IS the consent; hierarchy position does not
auto-grant management rights (PIV-C19-2).

---

### D-C18-5 — Action-Tier Authz + SoD Primitives (SF-5 provisionally DECIDED, hooks only)

**Decision — Day-2 scope (SoD backend primitives and hooks; workflow engine = Day-3):**

Every action-class in the Prism action registry carries the following METADATA:

```toml
# Example — authored at Central per P-ADS-09
[action_class."destructive.disable_account"]
risk          = "high"
affected_domains = ["identity"]
required_approver_role = ["security_lead"]
min_approvals  = 1

[action_class."ot.change_control_parameter"]
risk          = "critical"
affected_domains = ["ot"]
required_approver_role = ["security_lead", "safety_officer"]
min_approvals  = 2                 # NIST AC-3(2) dual authorization; IEC 62443 OT gating
```

**Backend SoD primitives (D-C18-5 scope):**
The backend enforces the following constraints at approval-check time (NOT UI-only):
- `approver_identity ≠ requester_identity` (recorded on the request object)
- Approver holds the action-class's `required_approver_role` binding (RBAC check)
- For `min_approvals: 2`: two DISTINCT approver identities required (ideally different org units)
- Executor identity = a dedicated JIT/short-lived workload credential (Zero Standing Privileges);
  distinct from requester and approver; revoked on completion
- Approval is CONTEXT-BOUND: target, data scope, parameters, time window. A generic approval
  cannot be replayed against a broader action. **[VERIFIED-WEB — NIST AC-5/AC-6/AC-3(2);
  Microsoft Entra PIM; CyberArk maker-checker]**
- Tamper-evident decision log records: requester, approver(s), workload credential, target, outcome

**What these are:** The SoD primitives are the contract the Day-3 Workflow Engine implements.
Day-2 ships: action-class metadata on every action-class definition; RBAC approver role bindings;
backend identity-distinctness check API. Day-2 does NOT ship the full approval workflow state
machine (requested → pending → approved → executing → completed | denied) — that is Day-3.

**Break-glass:** Emergency action-class (`break-glass`) invokable only by designated
break-glass identities; auto-alerting to leadership; enriched audit logging; minimal OT safety
interlocks still apply.

**Day-3 consolidation → E-WORKFLOW-ENGINE-001 (see §8).**

---

### D-C18-6 — Central-Authored / Edge-Enforced + Revocation-Lag Posture

**Decision:** Author policy ONLY at Central (C9 Config-DB-Authoritative, INV-ADS-04). Distribute
as signed, versioned policy bundles (PAT-ADS-03) to satellites. Each satellite runs a local
**PDP (Policy Decision Point) + PEP (Policy Enforcement Point)** and enforces fully offline.
Version-stamp every authorization decision with the policy bundle version used. **[VERIFIED-WEB —
OWASP NHI Top-10, NIST CSF 2.0 PR.AC-4]**

**Revocation-lag / staleness posture:** The acceptable staleness window per action-class and
the OT-action staleness behavior are **NOT hardcoded in the authz engine**. They are expressed
as settings in the active **Compliance Profile** (ADR-PROP-compliance-profiles.md), specifically
the `[settings.staleness]` block. Examples of what the profile controls:
- `baseline`: read queries continue under last-known-good; destructive actions deny if bundle
  older than profile-tunable N seconds (range `[60, 86400]`)
- `iec-62443-ot`: OT-affecting actions: `deny_on_stale_seconds` clamped to `[0, 60]`
- `nerc-cip`: OT-affecting actions: `deny_on_stale_seconds = { lock = 0 }` — any staleness
  hard-denies; this is a **Compliance Profile hard-lock**, not a hardcoded code branch

**Consistency token (New-Enemy-Problem):** Zanzibar mitigates stale-replica allow-after-revoke
with consistency tokens (Zookies). Prism's in-process build must own this mechanism: write
operations produce a version token; subsequent reads pin to that token to guarantee monotonic
consistency. Offline satellites cannot fetch fresh tokens — they rely on bundle TTL + the
`deny-on-stale-beyond-N` posture (configured per Compliance Profile) as the backstop.

**Offline decision log buffering:** Satellites buffer authorization decision logs locally and
reconcile to Central on reconnect (see D-C18-7). Offline buffer sizing / retention / transport
mechanism = OQ-C18-3 (open question).

---

### D-C18-7 — Decision-Level Audit Log

**Decision:** Log the authorization DECISION, not only the access event. Every authorization
decision log entry includes:
- Subject identity (user/workload, session ID)
- Resource (object type + ID)
- Action + action-class
- Attributes considered (sensitivity tags, role bindings used)
- Policy bundle version
- Outcome (allow / deny / deny-on-stale)

This exceeds the access-event minimum that NERC CIP / SOC2 / ISO require. It is the strongest
available evidence of least-privilege enforcement and is increasingly a built-in engine feature
(SpiceDB audit logging is the template: per-operation logs with token hash, method,
request/response, shipped to sinks). **[VERIFIED-WEB]**

**Offline satellite:** buffer decision logs locally; forward + reconcile on reconnect. Buffer
size / retention policy = OQ-C18-3.

**Proposed new cross-cutting ADS invariant (Pass 2 — do not author here):**
This decision is the basis for a proposed new cross-cutting invariant:
`INV-ADS-09: Authorization decisions MUST be logged at decision-resolution time (not merely at
access-event time). Offline satellites buffer and reconcile. Decision logs include policy version,
attributes considered, and outcome.`
This invariant is flagged for addition to ARCHITECTURE-DESIGN-SYSTEM.md Section C.1 in Pass 2.
It is NOT authored here per the HARD BOUNDARY.

---

### D-C18-8 — AuthN Linkage (SF-6 provisionally DECIDED)

**Decision:** Map IdP groups/roles → Prism INTERNAL roles. Keep internal roles for the
per-resource depth. **[VERIFIED-WEB — WorkOS RBAC IdP-role docs, Scalekit OIDC B2B,
Microsoft Entra group-claims/app-roles]**

| IdP protocol | Mapping mechanism | Fast-path revocation |
|-------------|-------------------|---------------------|
| OIDC | `groups` / `roles` claim → Prism role mapping rules per tenant | SCIM 2.0 group deprovisioning |
| SAML | Group/role attribute assertions → mapping rules | SCIM 2.0 group deprovisioning |
| SCIM 2.0 | Automated user + group provisioning/deprovisioning (create/update/deactivate) | SCIM deactivate = fast-path revocation |
| JIT provisioning | At first SSO login, create user + assign roles from incoming claims | n/a (SCIM preferred) |

**IdP groups as inputs, NOT as the authorization model:** IdP groups supply coarse group
membership; Prism's RBAC+ReBAC+ABAC layers supply the depth (resource-scoping, hierarchy,
masking, approver-gating). SCIM deprovisioning is the fast-path for revocation; the edge bundle
TTL + Compliance Profile staleness window is the backstop for offline satellites.

**Group → role mapping authority (SF-6 DECIDED — lean):** IdP group → Prism-role mapping
rules are **tenant-admin-configurable WITHIN central-set bounds**. This mirrors the C19 hybrid
inheritance: central sets the permission-boundary ceiling; tenant admins can map their IdP
groups to roles within that ceiling but cannot elevate beyond it. Conforms to P-ADS-09
(Config-DB-Authoritative for the bounds) and P-ADS-13 (Configurable-Not-Prescriptive).

---

## 3 — Invariants

| ID | Invariant |
|---|---|
| **PIV-C18-1** | Authz is always layered RBAC + ReBAC + ABAC. No code path enforces only one tier and ignores the others. |
| **PIV-C18-2** | Role bindings take the form `(role, scope-node)`. A binding without a scope-node anchor is invalid (no global unscoped roles except the system-level `super-admin` break-glass identity). |
| **PIV-C18-3** | Permission inheritance is strictly downward. Escalation-up (child gaining parent-scope) is prevented by authz-schema construction and verified by a dedicated schema test. |
| **PIV-C18-4** | Columns are governed by ABAC sensitivity tags, NOT per-column roles. No code path creates a role whose name encodes a column identifier. |
| **PIV-C18-5** | SoD is enforced in the backend. The UI may ASSIST but cannot be the sole enforcement layer. `approver ≠ requester`, approver holds `required_approver_role`, executor is a distinct JIT credential — all enforced server-side. |
| **PIV-C18-6** | Policy is authored ONLY at Central (INV-ADS-04). Satellite PDP evaluates the bundle it receives; it does NOT author or override policy. |
| **PIV-C18-7** | Authorization decisions are logged at decision time (not just access time) with policy version, resource, attributes, and outcome. Offline satellites buffer and reconcile. |
| **PIV-C18-8** | Mechanism (c) — parent-as-persistent-KMS-grantee on child DEK — is FORBIDDEN for cross-tenant visibility (AP-ADS-11 + PIV-C19-5). RBAC-scoped visibility uses mechanisms (a)/(b)/(d) per C19 §3.6. |

---

## 4 — C18 Differentiator Framing vs Query.io (C10)

Query.io ships no in-product RBAC **[VERIFIED-WEB — C10 prior finding]**. Prism's C18 depth —
per-connector/source/table/column scoping, hierarchy inheritance across nested tenancy,
approver-gated actions, central-authored/edge-enforced policy, decision-level audit — is
therefore a clean, defensible differentiator that directly satisfies NERC CIP (CIP-004/005),
SOC2 CC6.1/CC6.3, and ISO 27001 Annex A.9 buyer requirements an MSSP's clients demand.
The layered model (RBAC + ReBAC + ABAC) is not overengineering for a single-vertical product;
it is the minimum viable depth for an MSSP serving regulated industries.

---

## 5 — Compliance-Profile Integration (SF-3 DECIDED via ADR-PROP-compliance-profiles.md)

SF-3 (revocation-lag / staleness posture) is NOT resolved by a hardcoded OT code branch. It is
resolved by the **Compliance Profile mechanism** (ADR-PROP-compliance-profiles.md):

- `deny_on_stale_seconds` per action-class is a **profile setting** (not a constant in the authz engine)
- OT-affecting actions' staleness behavior is expressed in `profile:iec-62443-ot` and `profile:nerc-cip`
- "OT" = a profile we ship, not a code branch (P-ADS-13; AP-ADS-07)

The authz engine reads the active Compliance Profile's `[settings.staleness]` block to determine
per-action-class deny thresholds. This makes C18's revocation-lag posture fully client-configurable
within the profile's declared bounds, tighten-only, and verifiable in conformance reporting.

---

## 6 — Proposed Epics

**E-CENTRAL-AUTHZ-001** (PROPOSED — not yet in STORY-INDEX): Prism-native authz core.
Covers: Zanzibar relationship-tuple engine (in-process Rust); PDP + PEP at satellite; offline
tuple replication via PAT-ADS-03 signed bundles; New-Enemy-Problem consistency tokens; RBAC
role-binding CRUD; ReBAC resource graph (connector/source/table/action-class object types);
ABAC sensitivity tag evaluation + column masking; per-query PEP at PrismQL planner; decision-
level audit log; offline buffer + reconcile. This epic is PROPOSED, gated on morph execution.

**E-AUTHN-PROVISIONING-001** (PROPOSED — check whether covered by C18 SSO ADR): OIDC/SAML/
SCIM → Prism internal role mapping. If not covered by the SSO identity ADR-PROP (ADR-PROP-
sso-identity.md), this is a separate epic for: claim→role mapping rules (tenant-admin
configurable within central bounds); SCIM 2.0 user + group provisioning/deprovisioning;
JIT provisioning on first login; SCIM deactivate → fast-path revocation in the PDP.
Check ADR-PROP-sso-identity.md for overlap at morph.

Both epics are PROPOSED. Neither is registered in STORY-INDEX. Registration gated on morph
execution (post-demo, post-T14, brief-reframe sign-off).

---

## 7 — Day-3 Deferral Block

**E-WORKFLOW-ENGINE-001 (Day-3) — Configurable Approval/Review Workflow Engine**

A fully-configurable, client-driven approval/review workflow engine consolidating:

| Source | Day-3 deferred item |
|--------|---------------------|
| C9 | Deferred: configurable approval/review workflows (human-directed deferral from C9 ADR-PROP) |
| C18 SF-5 | Full four-eyes / SoD workflow state machine (requested → pending → approved → executing → completed|denied) |
| C18 SF-2 | Per-record PII-field unmask approval workflow |
| C15 HITL | Configurable human-in-the-loop gate for ARO actions |

**Requirements (Day-3 engine must satisfy):**
- Fully client-configurable; no prescribed absolutes
- Four-eyes-CAPABLE: `min_approvals`, `required_approver_roles`, org-unit-distinctness enforcement
- Honors the Compliance Profile `[workflow_requirements]` contract surface
  (ADR-PROP-compliance-profiles.md §5): `requires_separation_of_duties`, `dual_auth_on`,
  `min_approvals`, `required_approver_roles`, `break_glass_alerts_to`
- **Fail-safe if a required `[workflow_requirements]` directive is not yet implementable** —
  must NOT silently ignore it; must report as `drifting` in conformance, never `compliant`
- Backend SoD primitives shipped in Day-2 D-C18-5 are the contract the engine implements

**Day-3, gated, PROPOSED. Not in STORY-INDEX.** Registration at morph.

---

## 8 — Open Questions

| ID | Question | Owner | Priority |
|---|---|---|---|
| **OQ-C18-1** | New-Enemy-Problem consistency token design: what is the exact format and distribution mechanism for per-write version tokens in the Prism-native Zanzibar core? Do offline satellites receive inline-signed tokens or do they rely solely on bundle-version monotonicity? | architect at morph | P1 |
| **OQ-C18-2** | Granularity ceiling confirmation: "ABAC sensitivity tags + masking, not per-column roles" confirmed as the ceiling. Confirm whether bulk-export of any `sensitivity:PII` column is hard-blocked by the `baseline` profile default or only approver-gated. Per D-C18-3, the lean is hard-blocked-by-default, configurable via Compliance Profile. | product-owner at morph | P1 |
| **OQ-C18-3** | Decision-log retention and sink at the satellite edge: bounded ring-buffer size, retention floor, reconcile transport mechanism (push vs pull, tie to C9 dial-home). Lean: deny-or-degrade-on-buffer-full for high-risk action-classes. Exact sizing = legal/compliance guidance at morph. | architect + data-engineer at morph | P1 |
| **OQ-C18-4** | Build-vs-adopt final confirmation at morph: cost analysis of authz core build scope vs fallback adoption of `casbin-rs`. Should produce a time estimate comparison. `casbin-rs v2.20.0` is the leading fallback candidate. | architect at morph | P1 |
| **OQ-C18-5** | Executor identity (JIT) mechanism: how is the short-lived workload credential issued and bound to the approved action context? Ties C9 (config authority) and C17 (key-escrow ops). Lean: a dedicated JIT credential service at Central issues short-lived tokens bound to the approval record. | architect at morph | P2 |
| **OQ-C18-6** | Org-unit-distinctness enforcement for dual-auth: for `min_approvals: 2`, does Prism enforce that the two approvers come from distinct org-units in the tenant hierarchy, or only that they are distinct identities? Lean: distinct identities as the minimum; org-unit-distinctness as a `nerc-cip` Compliance Profile requirement. | product-owner at morph | P2 |

---

## 9 — Cross-Wiring with Sibling Features

| Feature | Cross-wire point |
|---|---|
| **C19 Nested Tenancy** | Role bindings = `(role, scope-node)` filtered by C19 closure table (D-C19-5). C18's ReBAC resource graph GENERALIZES the C19 tenant closure-table slice — the closure table IS the materialized tenant-tier projection of the broader relationship graph. C18 adds connector/source/table/action-class nodes below the tenant tier. Escalation-up prevention + permission-boundary semantics are C19's structural foundation that C18 inherits. |
| **C16 Detokenize-at-Surface / ABAC Masking** | ABAC column masking = C16's detokenize-at-surface enforcement point. The PrismQL planner (C18 PEP) intercepts column reads tagged `sensitivity:PII` before fan-out. C16 provides the masking semantics; C18 provides the enforcement gate. |
| **C15 ARO / Gated Actions** | Action-classes defined in C15 carry `required_approver_role` + `min_approvals` metadata authored in C18's authz engine. C15 defines which action-class IDs exist; C18 defines who may approve and what SoD constraints apply. The C15 `required_approver_role` field is a C18 RBAC role binding requirement. |
| **C17 Key-Escrow Operations** | Key-escrow operations (CMEK rotation, DEK derivation, HYOK handoff) are privileged `action-class` entries governed by C18 RBAC approver roles. An operator requesting a key-escrow operation must hold the corresponding approver role and the SoD constraint enforces a distinct approver. |
| **C11 Metering** | Metering roles are distinct from data/config roles (D-C19-6). C11 billing roles are expressed as a separate RBAC role tier; they share the `(role, scope-node)` structural form but do not grant data-access permissions. |
| **C9 Config-DB-Authoritative / Signed Bundles** | Authz policy (relationship tuples, role definitions, action-class metadata) is authored ONLY at Central (INV-ADS-04) and distributed to satellites via PAT-ADS-03 signed bundles. Staleness window per Compliance Profile references C9's bundle delivery TTL. |
| **C20 NERC-CIP** | NERC CIP-004/005 access-control requirements (revocation program, ESP rules, access logging) are satisfied via the `profile:nerc-cip` Compliance Profile (ADR-PROP-compliance-profiles.md), not via hardcoded OT code paths. C18 provides the RBAC/audit mechanism; the profile provides the NERC-CIP-specific settings. |
| **ADR-PROP-compliance-profiles.md** | SF-3 staleness posture, OT action gating, masking strictness, key-custody requirements, and workflow requirements are ALL Compliance Profile settings. C18 provides the enforcement engine; the profile provides the client-configured posture. |

---

## 10 — ADS Conformance Checklist

```
CONFORMANCE CHECKLIST — ADR-PROP-rbac-depth.md (C18) — 2026-06-27

P-ADS-01: Central-Sole-Surface
  [YES] All authz policy is authored at Central UI/DB only. Satellite PDP is
        headless; it receives and enforces the signed bundle. No satellite-side
        policy authoring surface. RBAC role management, action-class definitions,
        IdP group→role mapping = all Central-only interactions.

P-ADS-02: Operator-Zero-Access-At-Rest
  [YES] Decision logs are encrypted under the tenant's DEK (PAT-ADS-02) before
        persistence at Central. Operator infrastructure sees decision log
        ciphertext only.

P-ADS-03: Derived-Results-Only-At-Central
  [YES] The authz system processes metadata (role bindings, tuple relations,
        policy versions) — not raw sensor data. No raw sensor data transits
        from edge to Central as part of authz evaluation. The derived audit
        decision log (who was allowed/denied, with which attributes) is what
        surfaces at Central.

P-ADS-06: Per-Tenant-Isolation
  [YES] Authz tuples are per-tenant partitioned in the tuple store. No cross-
        tenant tuple joins. Per-tenant role bindings enforced by org_id scoping
        on the tuple namespace. PIV-C18-8 + AP-ADS-11: mechanism (c) FORBIDDEN.

P-ADS-07: AI-Opaque
  [YES] The authz engine receives metadata (role IDs, resource IDs, tag names) —
        never raw credentials or PII-field values. Column masking gates raw PII
        before any AI component sees the data.

P-ADS-09: Config-DB-Authoritative
  [YES] Role definitions, action-class metadata, authz schema (object types /
        relations / permissions), and IdP→role mapping rules are ALL authored at
        Central UI/DB. No satellite, CLI, or git-committed TOML authors production
        authz policy. Conforms to PIV-C9-001 and INV-ADS-04.

P-ADS-10: Idempotent-Gated-Actions
  [YES] Every action carries an idempotency key (P-ADS-10). The authz gate (RBAC
        check + SoD check + approval check) is the gated path all actions must
        pass. Autonomy level is system-configured (Compliance Profile + action-
        class metadata), not agent-configured. Satisfies ARO-loop discipline.

P-ADS-11: Single-Codebase / Deployment-Profile
  [YES] One authz engine in the Rust binary. OT-specific staleness / gating
        behavior is DATA (Compliance Profile settings), not a code fork. No
        #[cfg(feature="ot")] or #[cfg(feature="nerc_cip")] in the authz engine.

P-ADS-12: Production-Grade-Default
  [YES] SoD enforcement is backend-enforced (not UI-only). Decision-level audit
        exceeds the standards minimum. Per-column roles explicitly rejected in
        favor of tag-based masking (the correct scalable mechanism). Build
        rationale for engine choice documented with crate versions verified.

P-ADS-13: Configurable-Not-Prescriptive
  [YES] Staleness windows, masking strictness, action-gating requirements, and
        workflow requirements are all Compliance Profile settings — not hardcoded
        absolutes. OT gating = profile:iec-62443-ot. NERC-CIP gating =
        profile:nerc-cip. Client configures WITHIN profile-permitted bounds.
        The invariant floor (P-ADS-13 Floor) — operator-zero-access, per-tenant
        isolation, AI-opaque — is NEVER configurable off.

AP-ADS-07: No Per-Deployment-Model Code Forks
  [YES] Explicitly confirmed. "OT mode" is profile:iec-62443-ot, not a code
        branch. All deployment profiles share the same authz engine binary.

INV-ADS check (all eight):
  [YES] INV-ADS-01: authz evaluates metadata; no raw sensor data at Central
  [YES] INV-ADS-02: decision logs encrypted under tenant DEK; operator blind
  [YES] INV-ADS-03: per-tenant tuple partitioning; no cross-tenant joins
  [YES] INV-ADS-04: policy authored only at Central; INV enforced by PIV-C18-6
  [YES] INV-ADS-05: all actions gated; idempotency keys mandatory; gate before execute
  [YES] INV-ADS-06: authz engine receives metadata not PII; masking gates before AI
  [YES] INV-ADS-07: authz does not introduce new data sources; OCSF unaffected
  [YES] INV-ADS-08: in-process authz engine + SoftwareKms + offline bundle = air-gap valid

NOTE: New INV-ADS-09 (decision-level audit) is PROPOSED for Pass 2 addition to
ARCHITECTURE-DESIGN-SYSTEM.md Section C.1. Not authored here per HARD BOUNDARY.
```

All checklist items PASS. C18 also conforms to P-ADS-13 (the new configurable-not-prescriptive
principle added to the ADS v1.1).

---

## 11 — Decision Provenance

| Decision ID | Sub-fork | Human decision |
|-------------|----------|----------------|
| D-C18-1 | Authz model paradigm | Layered RBAC + ReBAC + ABAC; no single paradigm |
| D-C18-2 | SF-1 engine choice | Build Prism-native Rust Zanzibar-tuple core; alternatives documented as FALLBACKS in priority order |
| D-C18-3 | SF-2 resource granularity | connector/satellite/source/table first-class; columns via ABAC tags; bulk-export hard-blocked by default (configurable via Compliance Profile) |
| D-C18-4 | Hierarchy enforcement | Strictly-downward inheritance; escalation-up as tested schema invariant; permission-boundary + confused-deputy semantics |
| D-C18-5 | SF-5 action-tier (provisionally decided — hooks only) | SoD backend primitives + action-class metadata shipped Day-2; full workflow engine = Day-3 E-WORKFLOW-ENGINE-001 |
| D-C18-6 | SF-3 revocation-lag (DECIDED via Profile mechanism) | Revocation-lag posture is a Compliance Profile setting (ADR-PROP-compliance-profiles.md), not a hardcoded OT code branch |
| D-C18-7 | Audit depth | Decision-level audit (not just access-event); offline buffer + reconcile |
| D-C18-8 | SF-6 IdP mapping | IdP group → Prism internal role mapping; tenant-admin-configurable WITHIN central-set bounds; SCIM deprovisioning = fast-path revocation |
