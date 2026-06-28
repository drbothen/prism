# RBAC Depth — Robust Roles & Permissions Extending Into Connectors / Satellites / Data / Actions (C18)

- **Type:** general (technology / architecture research)
- **Date:** 2026-06-27
- **SIDE-ANALYSIS item:** C18 — robust RBAC that extends all the way down into connectors, satellites, sources, queries, tables/columns, and actions
- **Mode:** CAPTURE / research only (do_not_execute). No live spec/BC/ADR/STATE/SESSION-HANDOFF modified.
- **Ties:** C10 (Query.io has NO in-product RBAC — RBAC is a Prism differentiator), C19 (nested tenancy / role inheritance), C15 (ARO — `required_approver_role`), C9 (config DB-authoritative, central-authored), C20 (NERC CIP access control), Architecture Design System (Central-Sole-Surface; per-tenant isolation; gated actions).
- **Status:** complete

> Confidence labels: **[VERIFIED-WEB]** = grounded in cited web sources this pass; **[CRATES.IO-VERIFIED]** = version checked against the crates.io API on 2026-06-27; **[MODEL]** = model knowledge, flagged as such; **[INCONCLUSIVE]** = sources thin/conflicting.

---

## 0. Executive Summary / LEANS (read first)

Prism's differentiator vs Query.io (C10: Query.io ships **no** in-product RBAC) is *authorization depth*: a single central-authored policy model that scopes permissions per-connector, per-satellite, per-source, per-table/column, and per-action-class, with hierarchy inheritance across nested tenancy (C19) and approver-gated actions (C15), enforced even at offline edge satellites.

**Recommended model — layered, not single-paradigm:**

1. **RBAC core** for the human-facing role surface (analyst / lead / admin / approver), because it maps to MSSP org structure and is what auditors expect (SOC2 CC6.x, ISO 27001 A.9, NERC CIP-004). **[VERIFIED-WEB]**
2. **ReBAC (Zanzibar-style) for the hierarchy + resource graph** — tenant → sub-tenant → connector → source → table is naturally a relationship graph; inheritance "down the tree" and subtree-scoping are first-class via parent→child relations and the arrow/tupleset-to-userset operator. This is exactly the C19 nested-tenancy requirement. **[VERIFIED-WEB]**
3. **ABAC conditions for the dynamic / field-level layer** — column masking (PII), row filters, environment/time conditions, sensitivity tags. ABAC is the right tool for "may not see column Z." **[VERIFIED-WEB]**

**Engine choice (the central sub-fork — see §10):** Prism is Rust, air-gap-capable, edge-enforced. The leading Zanzibar engines (SpiceDB, OpenFGA) are **Go services**, not embeddable Rust crates — using them at headless satellites means shipping and operating a sidecar process per edge node. **[VERIFIED-WEB / MODEL]** The genuinely Rust-native / in-process options are **Oso** (embedded, Polar, supports RBAC+ABAC+ReBAC), **Casbin (casbin-rs)** (multi-model), **opa-wasm** (`matrix-org/rust-opa-wasm`, evaluate OPA/Rego compiled to WASM in-process), and **gatehouse** (Rust-native, composable RBAC+ABAC+ReBAC). **[CRATES.IO-VERIFIED]** My lean is a **purpose-built Prism authz core that implements the Zanzibar relationship-tuple data model internally** (it is well-specified and Prism already owns its config DB authority per C9), rather than adopting a Go sidecar at every satellite — with `opa-wasm` or `casbin-rs` as the fallback if "build vs adopt" tips toward adopt. This is a human decision (§10 SF-1).

**Resource-scoping granularity (lean):** scope to **connector / satellite / source / table** as load-bearing first-class resources; do **column/field-level via ABAC masking tags** (not per-column roles, which cause role explosion); per-query authz is the natural enforcement point since Prism is a query engine. **[VERIFIED-WEB]**

**Hierarchy (C19) lean:** model inheritance as parent→child relations; permissions propagate **down** the tree only; explicitly prevent escalation **up** the tree (a child-tenant admin must not gain parent scope). Scope a grant to a subtree by anchoring the relation at the subtree root. **[VERIFIED-WEB]**

**Action-tier (C15) lean:** each action-class carries `required_approver_role` + `min_approvals` metadata; enforce **separation of duties in the backend** (requester ≠ approver ≠ executor), bind the approval to the request context (target/scope/time-window), execute via a distinct short-lived/JIT execution identity, and emit a tamper-evident decision log. This is the NIST AC-5 / AC-6 / AC-3(2) + four-eyes/maker-checker pattern, plus IEC 62443 gating for OT-affecting actions. **[VERIFIED-WEB]**

**Central-authored / edge-enforced (C9) lean:** author policy ONLY at central (Config-DB-Authoritative), distribute as **signed, versioned policy bundles** to satellites, which run a **local PDP** and enforce locally even when isolated. Accept the explicit tradeoff: offline satellites have **revocation lag** — mitigate with short policy TTLs, version-stamped decisions, and a "deny-on-stale-beyond-N" posture for high-risk action-classes. **[VERIFIED-WEB]**

**Audit lean:** log the authorization **decision** (subject, resource, action, attributes considered, policy version, outcome), not just the access event — standards require access logging but increasingly engines treat decision-logging as best practice; for NERC CIP / SOC2 a decision log is the strongest evidence of least-privilege enforcement. Offline satellites buffer decision logs locally and reconcile to central on reconnect. **[VERIFIED-WEB]**

**AuthN linkage:** map IdP groups/roles (OIDC `groups`/`roles` claim, SAML group attributes, SCIM 2.0 group provisioning) → Prism **internal** roles. Keep internal roles even though groups come from the IdP, because Prism must own its fine-grained per-resource semantics. **[VERIFIED-WEB]**

---

## 1. Authorization Models — RBAC vs ABAC vs ReBAC (Q1)

### 1.1 The three models and their fit for fine-grained, resource-scoped, hierarchy-aware authz

| Model | Primary abstraction | Hierarchy expression | Resource scoping | Best for Prism layer |
|-------|--------------------|----------------------|------------------|----------------------|
| **RBAC** | Roles → permissions | Role hierarchies + tenant-scoped roles (indirect) | Assign roles at tenant/resource level, often indirectly; suffers **role explosion** as scope dimensions multiply | Human role surface (analyst/lead/admin/approver) |
| **ABAC** | Attributes of subject / resource / environment | Encoded in attributes, evaluated in policy predicates | Resource attributes (tenant, source, classification, PII tag) | Dynamic conditions + **field/column-level masking** |
| **ReBAC** | Relationships + usersets in a graph | **First-class**: hierarchy = relations between objects, permissions propagate along edges | Structure the relationship graph; compute permissions along edges | **Tenant/connector/source/table hierarchy + C19 inheritance** |

**[VERIFIED-WEB]** Auth0's multi-tenant SaaS guidance frames RBAC as the most familiar but requiring tenant-scoped roles to prevent cross-tenant escalation; ABAC as the dynamic/contextual model; and ReBAC as ideal where "access depends on complex, dynamic relationships among users, resources, and organizations." Oso's Authorization Academy frames ReBAC as the model for data ownership, parent-child resources, groups, and hierarchies.

**Key insight for C18/C19:** a tenant → sub-tenant → connector → source → table chain **is** a relationship graph. ReBAC models it natively; RBAC and ABAC both have to *encode* the hierarchy (role-per-level for RBAC → role explosion; parent-id attributes for ABAC → brittle traversal). This is the core argument for a relationship-tuple core in Prism.

### 1.2 Google Zanzibar (the reference design)

**[VERIFIED-WEB]** Zanzibar (Google, 2019) is a single unified ACL store + evaluation engine providing fine-grained authz across Google's products at extreme scale. Data model:

- **Object** = `type:id` (namespace + identifier), e.g. `connector:crowdstrike-acme`.
- **Relation** = named edge connecting an object to a user or to another userset (`owner`, `reader`, `member`, `parent`).
- **Userset** = a set of users with a relation to an object, defined directly OR indirectly (compositional: unions/intersections/differences of relations). This is what enables "all members of project P."
- **Namespace configuration** = declares object types, their relations, and how *permissions* are computed from relations (separate from the stored tuples).
- **Relationship tuples** = the stored edges (which user/userset has which relation to which object).

**Hierarchy is not special** — it is just relations between objects, with permission propagation defined declaratively. SpiceDB expresses this with the **arrow operator** (`docorg->view_all_documents` = "users who have `view_all_documents` on the org referenced by the `docorg` relation"). Arbitrary-depth hierarchies (tenant→org→department→project→folder→document) are modeled by defining object types + relations and composing permissions across them.

**Consistency — the "New Enemy Problem":** stale replicas/caches can wrongly permit access that was just revoked. Zanzibar mitigates with **Zookies** — opaque consistency tokens encoding a causal timestamp; a write returns a Zookie, subsequent reads pin to it to guarantee a monotonic (not-older-than) view. **[VERIFIED-WEB]** Directly relevant to C9 edge enforcement (§5) and revocation lag.

### 1.3 Zanzibar-inspired implementations

| System | Lang | Model | Consistency token | Rust integration | Air-gap |
|--------|------|-------|-------------------|------------------|---------|
| **SpiceDB** (Authzed) | Go service (gRPC/HTTP) | ReBAC-first; schema = `definition`/`relation`/`permission`; arrow op for inheritance; assertions + expected-relations for schema testing | **ZedToken** (= Zookie); per-request consistency: `fully_consistent` / pin-to-token | Via gRPC/HTTP client; community crate `spicedb-rust` v0.3.4 (2024-12-01) **[CRATES.IO-VERIFIED]**, also legacy `authzed` v0.0.1 (2021, effectively abandoned) | Open-source, self-hostable, air-gap-capable as a **sidecar service** |
| **OpenFGA** (Auth0 → CNCF Incubating) | Go service | ReBAC + explicitly supports RBAC/ABAC use-cases; authz model = type definitions + relations + tuples | Zookie + query consistency modes `MINIMIZE_LATENCY` (default, cache OK) / `HIGHER_CONSISTENCY` (skip cache) | Via HTTP/gRPC; community crate `openfga-rs` v0.1.0 (2024-04-08, very early) **[CRATES.IO-VERIFIED]** | Open-source, self-hostable; air-gap-capable as a **service** |

**[VERIFIED-WEB / MODEL]** Both SpiceDB and OpenFGA are **Go services**, not embeddable Rust libraries. The provided sources did not confirm first-party Rust SDKs; community crates exist but are early/thin. For Prism's headless edge satellites this means **operating a co-located sidecar process** per node (not in-process linking). This is the central engine sub-fork (§10 SF-1).

### 1.4 Policy-as-code engines and Rust-native / embeddable options

| Engine | Lang / form | Models | Rust-native / in-process? | Latest version (crates.io 2026-06-27) | Air-gap |
|--------|-------------|--------|---------------------------|----------------------------------------|---------|
| **OPA / Rego** | Go engine; Rego policy language; can compile policy → **WASM** | General (RBAC/ABAC/ReBAC all expressible; hierarchy lives in data) | Via **`opa-wasm`** crate (`matrix-org/rust-opa-wasm`) — evaluate Rego-compiled-to-WASM **in-process** in Rust | `opa-wasm` **v0.2.1** (2026-06-16), publisher Quentin Gliech / sandhose **[CRATES.IO-VERIFIED]** | Fully self-hostable; WASM modules embeddable; strong air-gap fit |
| **Oso** | Rust crate (Polar language); embedded in-app | RBAC + ABAC + ReBAC in one policy lang; can call into app objects/ORM | **Yes — in-process Rust crate** | `oso` **v0.27.3** (2024-01-13), ~890k downloads, not yanked **[CRATES.IO-VERIFIED]**. *Note:* open-source crate last published Jan 2024; vendor focus shifted to Oso Cloud (SaaS) — maintenance cadence of the OSS Rust crate is a risk to assess **[MODEL / INCONCLUSIVE]** | OSS crate fully embeddable, no external dep (Oso **Cloud** would need connectivity; avoid for air-gap) |
| **Casbin (casbin-rs)** | Rust port of Casbin | Multi-model via PERM metamodel (ACL, RBAC, hierarchical RBAC, ABAC, ReBAC, BLP/Biba, etc.); CONF model files + policy store | **Yes — in-process Rust crate** | `casbin` **v2.20.0** (2026-02-04) — actively maintained **[CRATES.IO-VERIFIED]** | Fully embeddable, policy in files/DB; strong air-gap fit |
| **Gatehouse** | Rust crate | Combines role-based + attribute-based + relationship-based policies; "composable policies and request-scoped fact loading" | **Yes — in-process Rust crate** | `gatehouse` **v0.5.0** (2026-06-27, updated same day as this research) **[CRATES.IO-VERIFIED]** | Fully embeddable; air-gap fit (smaller ecosystem / maturity to assess) |

**[VERIFIED-WEB]** OPA supports loading policy via REST push OR downloading **signed, versioned bundles** over HTTP — directly the central-authored/edge-enforced pattern (§5). Oso runs embedded and can navigate ORM relationships in-process for ReBAC. Casbin's hierarchical-RBAC handles role inheritance natively. Gatehouse is the only crate that natively unifies all three models in-process in Rust.

---

## 2. Resource-Scoped Permissions — connector / satellite / source / table / column / action (Q2)

### 2.1 Granularity dimensions (coarse → fine)

**[VERIFIED-WEB]** Practical granularity ladder, from data-platform practice (Databricks Unity Catalog ABAC, Microsoft Power Platform column-level security):

1. **Data-source / connector level** — "role may query connector X but not Y." Coarse, simplest; insufficient alone for PII.
2. **Row-level** — row-filter policies conditioned on subject/resource attributes (e.g., "analyst sees incidents only for tenants in their region"). Unity Catalog attaches row-filter policies to tables/views/streams, evaluated against **governed tags**.
3. **Column / field-level** — column-mask policies; mask PII by default, allow per-record unmask only for privileged roles, **block bulk export**. Microsoft Power Platform: mask sensitive fields, prohibit bulk reads/exports, audit every unmask. Unity Catalog: column-mask policies driven by ABAC tags (e.g., tag `sensitivity:PII` → mask for users lacking the attribute).

**[VERIFIED-WEB]** Critical distinction: **visibility ≠ capability.** "Can see masked field one record at a time" ≠ "can bulk-export the column." Prism must separate *which data a role can see* from *which operations it can perform on that data* (query vs export vs feed-into-action).

### 2.2 Mapping to Prism's resource graph

**[MODEL — design synthesis from the above patterns]** Prism's first-class authz resources (recommended):

- `tenant` / `sub-tenant` (C19 hierarchy anchors)
- `satellite` (edge node — scope "this role operates satellite S")
- `connector` (sensor adapter, e.g. `connector:crowdstrike`)
- `source` (a specific configured instance of a connector for a tenant)
- `table` (OCSF-normalized table / schema surface)
- `column` / field (PII / sensitivity — **ABAC tag, not a role**)
- `action-class` (C15 — destructive/write actions, OT-affecting actions)

Policy expression target: *"role R may query `source:X` but not `source:Y`; may run `action-class:Z` only with approval; may not read columns tagged `sensitivity:PII`."* RBAC names the role; ReBAC scopes it to the source/table sub-graph; ABAC masks the PII column.

### 2.3 Per-query authorization (Prism is a query engine — this is the natural PEP)

**[VERIFIED-WEB]** Object-attached policies (Unity Catalog) auto-apply to any query touching the object. For Prism, the query planner is the obvious Policy Enforcement Point: every PrismQL query is decomposed into source/table/column accesses, and each is checked against policy before fan-out. This also lets Prism govern *query shape* (aggregations that could re-identify masked data).

### 2.4 OCSF field-level note

**[VERIFIED-WEB / INCONCLUSIVE]** The deep-research pass explicitly flagged a **gap: OCSF (Open Cybersecurity Schema Framework) has no explicit field-level authorization guidance** in public docs. OCSF defines the schema/field shapes; access control over those fields is an application concern Prism must own. Implication: Prism should maintain its own PII/sensitivity tagging over OCSF fields rather than expecting OCSF to carry authz metadata.

### 2.5 Granularity tradeoff (practical ceiling)

**[VERIFIED-WEB]** Very fine-grained (per-column-per-role) authz causes role explosion and a maintainability/performance burden. The maintainable pattern is **ABAC tags + policies attached at catalog/schema level that auto-apply to tagged objects**, so one policy covers all `sensitivity:PII` columns rather than N per-column roles. Lean: scope **roles** to connector/source/table; do columns via **tags**, not roles.

---

## 3. Hierarchy-Aware Authz Across Nested Tenancy (C19) (Q3)

### 3.1 Inheritance down the tree

**[VERIFIED-WEB]** ReBAC models hierarchy as relations between objects; permissions propagate along edges via the arrow / tupleset-to-userset operator. Prism pattern:

- `sub-tenant:acme-east` has relation `parent → tenant:acme`.
- `source:crowdstrike-acme-east` has relation `owner_tenant → sub-tenant:acme-east`.
- A permission like `source.query` resolves as: direct readers of the source **∪** `owner_tenant->query_all_sources` **∪** (transitively) `owner_tenant->parent->query_all_sources`.

So a permission granted at `tenant:acme` flows down to every sub-tenant/connector/source under it, exactly as C19 requires — without enumerating each child.

### 3.2 Preventing escalation UP the tree (the safety-critical direction)

**[MODEL — design rule, grounded in Zanzibar semantics]** Inheritance must be **strictly downward**. Define relations so that child→parent edges (`parent`) are used only to *resolve inherited grants from parent to child*, never to grant a child principal authority over the parent. Concretely:

- A `sub-tenant:acme-east` admin is `admin → sub-tenant:acme-east`. There is **no** schema rule that makes `admin` on a child confer `admin` on `tenant:acme`.
- Subtree-scoping a grant = anchor the relation at the subtree root. Granting `lead → sub-tenant:acme-east` confers authority over the acme-east subtree only; it cannot reach sibling `sub-tenant:acme-west` or the parent.

This is the standard Zanzibar containment model; the escalation-prevention is a property of *which directions the schema's arrow operators traverse*, so it must be an explicit, tested schema invariant (assertions / expected-relations in SpiceDB terms; a Prism authz-schema test in our terms).

### 3.3 How RBAC/ABAC would express the same (for completeness)

**[VERIFIED-WEB]** RBAC: tenant-scoped roles + hierarchical-RBAC role inheritance (Casbin supports this) — works but requires a role per (tenant, level) and careful resource→role mapping. ABAC: encode `parent_tenant_id` chains as attributes and traverse in policy — works (OPA can traverse nested JSON) but the hierarchy lives in data and traversal logic is hand-rolled and easy to get wrong. ReBAC is the lower-defect path for C19.

---

## 4. Action-Tier Authz — `required_approver_role`, SoD, JIT, OT gating (ties C15) (Q4)

### 4.1 Standards backbone

**[VERIFIED-WEB]**
- **NIST SP 800-53 AC-5 (Separation of Duties):** partition responsibilities so no single identity can initiate + approve + execute a high-risk action; reduces abuse "without collusion."
- **AC-6 (Least Privilege):** only necessary access; non-privileged users cannot execute privileged functions; restrict privileged accounts to designated roles; **periodic privilege review**.
- **AC-3(2) (Dual Authorization):** certain actions require approval of **two** authorized individuals before execution. Standards basis for "four-eyes."
- **ISA/IEC 62443** (OT/ICS): strong identify-&-authenticate before access; zone segmentation; PAM-brokered remote sessions with per-session approval — the basis for stronger gating on **OT-affecting actions**.

### 4.2 `required_approver_role` pattern (directly C15 / ARO)

**[VERIFIED-WEB]** Encode approval as **metadata on the action-class**, mirroring Microsoft Entra PIM (roles configured to require approval; delegated approvers = users/groups; time-bounded activation; 24h approver window) and fintech **maker-checker** (annotate approval-required endpoints; request becomes a pending approval record routed to a checker; executes only after a distinct checker with the required role approves):

```
action-class "destructive.disable_account":
  risk: high
  affected_domains: [identity]
  required_approver_role: [security_lead]      # ARO required_approver_role
  min_approvals: 1
action-class "ot.change_control_parameter":
  risk: critical
  affected_domains: [ot]
  required_approver_role: [security_lead, safety_officer]
  min_approvals: 2                              # AC-3(2) dual authorization
```

### 4.3 Enforce SoD in the backend (not the UI)

**[VERIFIED-WEB]** Mandatory backend checks at approval time:
- approver identity ≠ requester identity recorded on the request;
- approver holds the action-class's `required_approver_role`;
- for `min_approvals: 2`, two **distinct** approver identities (ideally different org units);
- executor identity distinct from requester and approver — execute via a **dedicated workload/JIT identity** with narrowly scoped, short-lived credentials, revoked after completion (Zero Standing Privileges / JIT-PAM).
- **Bind the approval to context** (target, data scope, parameters, environment, time window) so a generic approval can't be replayed against a broader action.
- **Tamper-evident decision log**: who requested, who approved, which credential executed.

### 4.4 Break-glass

**[VERIFIED-WEB]** Emergency-access ("break-glass") accounts are a deliberate exception to ZSP: very high privilege, credentials stored offline, **all use alerts + heavily audited**, used only when normal admin paths fail. Prism should define an emergency action-class invokable only by designated break-glass identities, with auto-alerting to leadership and enriched logging, and ideally still subject to minimal safety interlocks for OT.

### 4.5 Workflow state machine

**[VERIFIED-WEB]** Model each high-risk action as an object transitioning `requested → pending_approval → approved → executing → completed | denied`, with role-gated transitions and identity-distinctness constraints enforced per transition; for dual auth, ≥2 distinct approvers required to reach `approved`.

---

## 5. Central-Authored, Edge-Enforced (ties C9, C20) (Q5)

### 5.1 Separate policy authority from policy execution

**[VERIFIED-WEB]** The dominant 2024–2026 pattern: **central** team owns the single approved policy model (authority); **edge** integrates local **PDP** (decision) + **PEP** (enforcement) that apply that policy locally. For Prism: central = Config-DB-Authoritative (C9) authors policy; satellites enforce. OWASP NHI Top-10 + NIST CSF 2.0 PR.AC-4 endorse central policy ownership to avoid inconsistent rules and revocation lag.

### 5.2 Distribution — signed, versioned bundles

**[VERIFIED-WEB]** OPA bundles are the canonical mechanism: Rego policy + data compiled into a compressed, **signed** archive, pulled over HTTP (or shipped via ConfigMap / air-gap CLI). The bundle manifest `roots` key scopes a bundle to a `data` sub-namespace — so per-tenant or per-satellite bundles can carry disjoint policy slices (`data.tenantA` vs `data.tenantB`). Edge PDP verifies the signature before loading; on verification failure, fall back to prior version or deny-by-default. Permit.io's on-prem model uses a **Git repo as the versioned policy source of truth** + Policy Sync to PDPs (versioning via commits) — a clean fit with Prism's Git-as-config patterns.

### 5.3 Offline / air-gapped enforcement

**[VERIFIED-WEB]** Air-gap pattern (Google Distributed Cloud air-gapped; Event-Driven Automation `edaadm` bundles): central prepares signed, version-stamped bundles; transfer via controlled channel; air-gapped PDP consumes and enforces fully locally. **Engine division of labor observed in the sources:** OPA + bundles is the common choice for **fully-offline** enforcement; SpiceDB/OpenFGA are typically run **centralized** with per-request consistency tokens, because the sources did **not** document full Zanzibar-style offline edge replication. **[INCONCLUSIVE on offline Zanzibar replication]** — if Prism wants the relationship-tuple model AT the edge offline, it likely must build/operate replication itself rather than rely on documented SpiceDB/OpenFGA offline patterns.

### 5.4 The revocation-lag / stale-policy tradeoff (must be explicit)

**[VERIFIED-WEB]** Caching trust-sensitive state (entitlements, decisions, revocation lists) creates "more risk than benefit" when it can outlive the decision moment — this is the New Enemy Problem at the edge. NHIMG guidance: cache **content not decisions**; short TTLs; **event-driven invalidation**; separate performance caches from security-control paths; log cache hits on sensitive objects. Zanzibar engines mitigate with consistency tokens (ZedToken/Zookie, `HIGHER_CONSISTENCY` mode) but **only when the edge can fetch fresh tokens** — an offline satellite cannot, so it must rely on locally stored state + bundle TTL.

**Prism posture (lean):** version-stamp every policy bundle; each decision log records the policy version used; for high-risk action-classes adopt **deny-on-stale-beyond-N** (if the satellite's bundle is older than N, refuse destructive/OT actions until refreshed) while allowing read queries to continue under the last-known-good policy. This bounds revocation lag for the operations that matter most.

### 5.5 Offline audit logging + deferred reconciliation

**[VERIFIED-WEB]** SpiceDB's audit logging is a template: per-operation logs with token hash (SHA-256), method, request/response bodies, IP, errors, shipped to sinks (Kinesis). Offline satellites can't stream in real time — buffer **decision logs locally**, forward + reconcile on reconnect. Log the **decision** (subject, resource, action, attributes, policy version, outcome), not just the access.

---

## 6. Standards / Compliance (ties C20 NERC CIP) (Q6)

### 6.1 NERC CIP

**[VERIFIED-WEB]**
- **CIP-004-8 (Personnel & Training):** each Responsible Entity must implement documented **access revocation program(s)**; revoke logical + physical access when personnel no longer need it. A cited enforcement case (CIP-004-6 R4) shows that *insufficiently detailed* revocation procedures (e.g., physical-key handling) caused a violation — i.e., the program must be specific and rigorous, not just high-level. Sources did **not** state an exact revocation deadline (commonly interpreted as same-day / short window — **[INCONCLUSIVE on exact timeline]**).
- **CIP-005 (Electronic Security Perimeter):** define ESPs, control electronic access points, secure/monitor remote access (MFA de facto, least privilege for remote sessions, log access through access points). Edge satellites inside an ESP must enforce centrally-defined perimeter + access policy locally.
- **Least privilege** is woven through CIP-004/005/007/011 even where not named.
- **Access logging:** CIP requires logging *access events* and security-relevant events; the sources found **no explicit standards-level mandate to log every authorization decision** — but decision-logging is the strongest available evidence of least-privilege enforcement and is increasingly a built-in engine feature. Prism should log decisions regardless.

### 6.2 SOC 2 / ISO 27001

**[VERIFIED-WEB]**
- **SOC 2 CC6.1 / CC6.3:** expect logical-access controls implementing least privilege, role-based access, provisioning/modification/removal tied to authorization, and **periodic access reviews**.
- **ISO/IEC 27001 Annex A.9** (access control): access-control policy, user access provisioning/deprovisioning, privilege management, **periodic access reviews**, and removal/adjustment of access rights on role change/termination.
- Both expect demonstrable least privilege + access reviews; neither (per the sources) mandates per-decision logging, but both are satisfied more strongly by it.

---

## 7. AuthN Linkage — SSO / OIDC / SAML / SCIM → Prism roles (ties §16.4) (Q7)

**[VERIFIED-WEB]**
- **OIDC:** IdP (Okta/Entra) emits `groups` / `roles` / app-role custom claims in the ID token; Prism normalizes claims → internal roles via tenant-defined mapping rules.
- **SAML:** IdP sends group/role **attribute assertions**; SP (Prism) maps attribute values → internal roles via group-to-role config.
- **SCIM 2.0:** automated user + group **provisioning/deprovisioning** (create/update/membership-change/deactivate) so directory changes propagate in near-real-time without waiting for next login — important for the **revocation timeliness** that NERC CIP-004 / SOC2 / ISO A.9 expect.
- **JIT provisioning:** at first SSO login, create the user + assign roles from incoming claims when no SCIM record exists yet.
- **Why keep internal roles even though groups come from the IdP:** Prism must own its fine-grained per-resource (connector/source/table/column/action-class) semantics. IdP groups are **inputs that map into** Prism roles; they do not replace Prism's authorization model. This is exactly why C18 depth lives in Prism, not the IdP.

**Design implication:** the IdP supplies *coarse group membership*; Prism's RBAC+ReBAC+ABAC layers supply the *depth* (resource-scoping, hierarchy, masking, approver-gating). SCIM deprovisioning is the fast-path for revocation; the edge bundle TTL (§5.4) is the backstop for offline satellites.

---

## 8. ANALYSIS — Recommended Authz Model for Prism

1. **Layered model:** RBAC (human role surface) + ReBAC (tenant/connector/source/table hierarchy & C19 inheritance) + ABAC (column/field masking, dynamic conditions). Do **not** pick a single paradigm.
2. **Engine:** prefer a **Prism-owned authz core implementing the Zanzibar relationship-tuple model in-process in Rust**, because (a) the leading Zanzibar engines are Go *services* requiring a sidecar at every headless satellite, (b) Prism already owns config-DB authority (C9), (c) offline edge replication of Zanzibar tuples is undocumented in the engines and would need building anyway, and (d) Rust-embeddable alternatives (`oso` v0.27.3, `casbin` v2.20.0, `opa-wasm` v0.2.1, `gatehouse` v0.5.0) exist as fallbacks if "adopt" beats "build." **[CRATES.IO-VERIFIED versions]**
3. **Resource scoping:** roles scoped to connector/satellite/source/table as first-class resources; columns via **ABAC sensitivity tags + masking policies**, not per-column roles (avoid role explosion); enforce per-query at the planner (the natural PEP).
4. **Hierarchy (C19):** parent→child relations; **strictly downward** inheritance; subtree-scoping by anchoring grants at the subtree root; **escalation-up prevention as a tested schema invariant.**
5. **Action-tier (C15):** `required_approver_role` + `min_approvals` metadata per action-class; backend-enforced SoD (requester≠approver≠executor); context-bound approvals; JIT/short-lived executor identity; dual-auth + IEC 62443 gating for OT-affecting actions; audited break-glass.
6. **Central-authored / edge-enforced (C9):** sign + version policy bundles at central; satellites run local PDP/PEP and enforce offline; explicit revocation-lag mitigation (short TTL, version-stamped decisions, deny-on-stale-beyond-N for high-risk actions); buffer + reconcile decision logs.
7. **Audit:** log authorization **decisions** (subject/resource/action/attrs/policy-version/outcome), exceeding the access-event minimum that NERC CIP / SOC2 / ISO require, as the strongest least-privilege evidence.
8. **AuthN:** OIDC/SAML claim + SCIM group → Prism internal role mapping; SCIM deprovisioning for fast revocation; keep internal roles for depth.

---

## 9. Differentiator framing vs Query.io (C10)

**[VERIFIED-WEB — C10 prior finding]** Query.io ships no in-product RBAC. Prism's C18 depth — per-connector/source/table/column scoping, hierarchy inheritance across nested tenancy, approver-gated actions, central-authored/edge-enforced policy, decision-level audit — is therefore a clean, defensible differentiator that also directly satisfies the critical-infrastructure (NERC CIP) and SOC2/ISO buyer requirements an MSSP's clients will demand.

---

## 10. Genuine Sub-Forks Needing a Human Decision

- **SF-1 (the big one) — Build vs Adopt the authz engine.**
  (a) **Build** a Prism-native Rust authz core implementing the Zanzibar tuple model in-process (max control, embeddable at satellites, owns offline replication; cost: significant net-new engineering + the New-Enemy-Problem to solve ourselves).
  (b) **Adopt embeddable Rust crate** — `casbin-rs` (multi-model, mature, v2.20.0), `oso` (ReBAC-capable but OSS crate last published 2024-01, vendor pivoting to SaaS — maintenance risk), `gatehouse` (unifies RBAC/ABAC/ReBAC in-process, v0.5.0 but young/small ecosystem), or `opa-wasm` (Rego-in-WASM, v0.2.1, great air-gap story but Rego is the policy language not Prism-native).
  (c) **Adopt SpiceDB/OpenFGA as a Go sidecar** at central + each satellite (most battle-tested ReBAC semantics + consistency tokens; cost: per-satellite process, no in-process integration, undocumented offline edge replication).
  *This is an architect decision (research → architect routing), not answerable in research scope.*

- **SF-2 — Column/field-level granularity ceiling.** Confirm "ABAC sensitivity tags + masking, not per-column roles" as the granularity ceiling, and whether bulk-export of any PII-tagged column is hard-blocked vs approver-gated.

- **SF-3 — Offline revocation-lag risk acceptance.** The acceptable staleness window `N` per action-class for the deny-on-stale-beyond-N posture, and whether OT-affecting actions are *hard-denied* on any staleness. (Risk-acceptance — human/business decision.)

- **SF-4 — Decision-log retention & sink at the edge.** Buffer size/retention for offline decision logs and the reconcile/transport mechanism to central (ties C9 config authority + the audit subsystem).

- **SF-5 — Approver model shape (C15).** Single `required_approver_role` vs primary/secondary approver roles vs `min_approvals`-threshold; whether approver org-unit-distinctness is enforced for dual auth.

- **SF-6 — IdP group→role mapping authority.** Whether group→Prism-role mapping is central-only (C9) or tenant-admin-configurable within central bounds.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | (1) RBAC/ABAC/ReBAC + Zanzibar + SpiceDB/OpenFGA + OPA/Oso/Casbin/Gatehouse + Rust-native/air-gap; (2) action-tier SoD/four-eyes/maker-checker/JIT-PAM/break-glass/IEC-62443 + resource/field-level granularity + OCSF gap; (3) central-authored/edge-enforced + OPA bundles + ZedToken/Zookie/New-Enemy + air-gap offline PDP/PEP + NERC CIP-004/005 + SOC2 CC6.x + ISO 27001 A.9. All `reasoning_effort: high`. |
| Perplexity perplexity_ask | 1 | OIDC/SAML/SCIM IdP-group → internal-role mapping + JIT provisioning (Q7, ≤6-sentence factual lookup). |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_reason | 0 | — |
| Context7 | 0 | — (no single-library deep-dive needed; crate facts verified via registry) |
| WebFetch (crates.io API) | 8 | Version/maintenance verification: `oso` 0.27.3, `casbin` 2.20.0, `opa-wasm` 0.2.1, `gatehouse` 0.5.0, `openfga-rs` 0.1.0, `spicedb-rust` 0.3.4, `authzed` 0.0.1 (+ one for the perplexity_ask sources). |
| WebSearch | 0 | — |
| Training data | ~3 areas | (a) SpiceDB/OpenFGA being Go-implemented (sources confirmed they're services but did not always name the language); (b) Oso vendor SaaS-pivot maintenance-risk note; (c) Prism-specific resource-graph design synthesis (§2.2, §3.2, §4.x) — all flagged [MODEL]. |

**Total MCP tool calls:** 4 Perplexity (3 deep-research high-effort + 1 ask). Plus 8 crates.io registry verifications via WebFetch.
**Training data reliance:** low-to-medium — all non-obvious model/standards/engine claims are [VERIFIED-WEB] from the three deep-research passes; every crate version is [CRATES.IO-VERIFIED] against the live registry on 2026-06-27; [MODEL] is confined to design synthesis and one maintenance-risk caveat, each explicitly flagged. Two items flagged [INCONCLUSIVE]: exact NERC CIP revocation timeline, and documented offline Zanzibar-engine edge replication.

### Source notes
- Deep-research citation sets (Authzed/SpiceDB docs incl. ZedToken + audit logging, OpenFGA docs incl. consistency modes + Zookie, OPA docs + Gloo Mesh signed-bundle integration, `matrix-org/rust-opa-wasm`, Oso Authorization Academy, Casbin docs, Auth0 multi-tenant authz guidance, Databricks Unity Catalog ABAC, Microsoft Power Platform column-level security, NIST SP 800-53 AC-3(2)/AC-5/AC-6, ISA/IEC 62443, Microsoft Entra PIM, CyberArk dual control, NHIMG centralized-authorization + caching guidance, OWASP NHI Top-10, NIST CSF 2.0 / AI RMF, Google Distributed Cloud air-gapped, Event-Driven Automation air-gap bundles, Permit.io on-prem, NERC CIP-004-8 + CIP-004-6 R4 case note, CIP-005) are preserved in the raw tool-result transcripts under `~/.claude/projects/-Users-jmagady-Dev-prism/.../tool-results/`.
- AuthN/SCIM citations: WorkOS RBAC IdP-role docs, LoginRadius RBAC+SSO federation, Scalekit OIDC B2B guide, WorkOS SAML attribute mapping, Microsoft Entra group-claims/app-roles docs.
