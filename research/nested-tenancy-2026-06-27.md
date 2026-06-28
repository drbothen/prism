---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
scope: "C19 — nested/hierarchical tenancy (unlimited depth) — closes OQ-DEPLOY-1"
provenance: >
  Day-2 SIDE-ANALYSIS input. Cited research pass to inform the human's decision on
  unlimited-depth nested tenant configurations (Parent -> Child -> grandchild -> ...).
  ANALYSIS/CAPTURE ONLY. Does NOT modify any live spec, ADR, BC, story, STATE.md, or
  SESSION-HANDOFF.md. No git operation performed. Section numbering internal to this doc.
traces_to:
  - matured-vision-day2-requirements.md §3.1 (central deployment) / §11.1 (server credential custody)
  - deployment-matrix open item OQ-DEPLOY-1 (tenancy-isolation depth: pool/bridge/silo/cell)
  - C18 (RBAC / 7-role console) — cross-link
  - C11 (metering) — cross-link
  - SS-26 secret broker (per-tenant DEK / KMS key hierarchy)
  - project memory AD-017 (AI-opaque credentials), BYOC zero-access thesis
  - central-deployment-access-layer-2026-06-26.md (Option-3 tenant-keyed cache; OAuth 2.1 RS)
  - central-surfacing-ripple-analysis-2026-06-27.md (Central-Sole-Surface; D-C2-12 raw-data-at-Central invariant)
  - P-ADS-06 Per-Tenant-Isolation; Central-Sole-Surface; Config-DB-Authoritative/UI-authored-at-central
---

# Nested / Hierarchical Tenancy (Unlimited Depth) — Cited Research (Side-Analysis)

> PROPOSED discussion input. Status: capture. Not a spec, not an ADR, not a vision change.
> This closes-out analysis for **OQ-DEPLOY-1**. The ANALYSIS + LEANS section at the end
> states the recommended model and the genuine human sub-forks.

## Executive Summary (~14 lines)

1. **Every major precedent uses an adjacency list as the source-of-truth and bolts a derived ancestor index on top.** AWS Organizations (OUs), GCP (org→folder→project), Azure Management Groups, Salesforce account/role, and MSSP partner→customer trees all store a single `parent_id` and traverse the chain for policy. None invent a fancier authoritative encoding [1][3][4][5][9][14][17].
2. **All of them cap depth** — AWS OUs 5 levels, GCP folders 10, Azure mgmt groups 6, Salesforce external account hierarchy 5 — and these are *control-plane manageability* limits, not algorithmic ones. A bespoke platform *can* support unlimited depth but inherits the same debuggability/perf pressures the vendors capped to avoid [2][4][5][10][11].
3. **For the authorization workload (ancestor checks + descendant enumeration + policy aggregation along a path), the closure table is the scalable read structure;** adjacency-list-only forces recursive CTEs that degrade with depth; nested-set has excellent reads but pathological reparenting cost; materialized path is great for prefix/sharding but rewrites all descendant paths on a move [16][17].
4. **The recommended physical model is a triple:** adjacency list (authoritative `parent_id`) + closure table (fast `is-ancestor` / `all-descendants` for policy + RBAC scoping) + optional materialized path (for sharding/cell placement) [17].
5. **Isolation depth is a SEPARATE axis from the logical tree.** The AWS SaaS-Lens taxonomy — **pool** (shared schema, tenant-id rows), **bridge** (mixed), **silo** (dedicated DB/instance), **cell** (dedicated stack per tenant-group) — is orthogonal to who-can-see-whom, and **the bridge model explicitly endorses isolation depth VARYING by tenant tier** (some pooled, some siloed) [18].
6. **Policy inheritance has two precedent semantics: intersection-of-allows + explicit-deny (AWS SCP — strict, only-tighten-going-down) vs union-of-allows (GCP IAM — accumulate-going-down).** Prism's config/policy model fits the AWS-style "parent guardrail can only restrict, child can only narrow within it" far better than GCP's accumulate model [1][3].
7. **Parent→child administrative authority MUST be explicit + consented, never implicit-by-position.** Palo Alto Cortex XDR is the direct MSSP precedent: parent sends a pairing request, the **child admin must approve** before the parent can manage the child [14]. This is the model to copy.
8. **Privilege containment = permission boundaries + confused-deputy guards.** A delegated (parent/MSSP) admin's effective grants are the *intersection* of their identity policy and a permission boundary; they can never exceed it, and cross-tenant access is gated by source-org/source-path conditions so a parent cannot reach siblings or escalate up [2][6].
9. **Per-CHILD data keys remain the right granularity even in deep nesting.** Envelope-encryption + CMK best practice is "never share a DEK across two tenants/users"; a child's data is encrypted under the *child's* key, NOT a parent-derived key [9][10][11][13]. This is the cryptographic enforcement of P-ADS-06 and the BYOC zero-access thesis.
10. **Parent sees aggregated child data WITHOUT holding the child's key** via three patterns: (a) query-time scoped decryption under the child key, transient + audited (CMK call / KMS grant); (b) re-encrypt the *aggregate result* under the parent's key; (c) BYOC remote-op where the parent never holds keys [11][12][15]. Parent visibility is on *derived metrics*, not raw rows, and is consented.
11. **Metering is recorded at the leaf and rolled up via the ancestor relationship** — exactly the closure-table the auth path already uses. AWS Cost Categories / consolidated billing and GCP org-billing both attribute at the resource/leaf and aggregate up the tree, on *metadata* that is separate from (and does not breach) data isolation [1][19][20].
12. **Central-Sole-Surface composes cleanly:** nested tenants are administered at Central; satellites stay headless; a deeply-nested org navigates by selecting a scope node in the tree and the closure table bounds what they can see/act-on. The tree lives in the Central control-plane DB (Config-DB-Authoritative), authored only at Central UI.
13. **This CLOSES OQ-DEPLOY-1:** the answer is not "pick one of pool/bridge/silo/cell" — it is **bridge by default, with per-node isolation tier as a tenant attribute**, decoupled from the logical hierarchy.
14. **Genuine human sub-forks remain:** inheritance semantics (intersection-vs-union), whether to actually permit *unbounded* depth or impose a soft cap, and the parent-aggregate-visibility default (opt-in per child vs tenant-tier policy).

---

## Read-coverage / honesty note

- **In-repo read:** `central-deployment-access-layer-2026-06-26.md` (Option-3 tenant-keyed cache, OAuth 2.1 RS, DI-NEW-006 cross-analyst isolation, SS-26 per-tenant DEK), `central-surfacing-ripple-analysis-2026-06-27.md` (Central-Sole-Surface constraints 1&2, D-C2-12 "Central never stores raw sensor data" hard invariant, S3 conversation-store Option-3 precedent), `ui-webstack-admin-rbac-2026-06-25.md` §5 (multi-tenant RBAC console, fine-grained roles). Project memory (OrgId/OrgSlug tenant abstraction, AD-017 AI-opaque credentials, BYOC zero-access).
- **NOT read in full:** the live BC/ADR bodies for tenant model and SS-26 §4–7 DEK hierarchy detail (cited from the prior research summaries, not re-derived from the BC text).
- **External sources:** two `perplexity_research` (sonar-deep-research, reasoning_effort high+medium) deep passes returned 85k/76k-char results. I read the leading ~64–66k chars of each (the substantive analysis + the comparison tables + the citation-anchored claims for resource hierarchies, tree encodings, isolation taxonomy, RBAC delegation, envelope encryption, and metering). The trailing ~12–20k chars (integrated-design recap + raw citation list) were NOT fully paginated due to single-line-file token caps — the *substance* of those tails (per-child DEK, scoped decryption, re-encryption of aggregates, leaf-metering+ancestor-rollup) was already present in the read portion and is what I cite. Flagged in Research Methods.
- **Citation indices [n]** below refer to the source set the deep-research model assembled; the source *identity* (AWS Organizations docs, GCP Cloud KMS / Resource Manager, Azure CAF, Kubernetes HNC, Salesforce, Okta/Auth0, Cortex XDR, IronCore CMK, WorkOS, AWS SaaS-Lens, bool.dev tree-encoding writeup, Joe-Celko/ScienceDirect nested-set) is named inline so the claim is traceable even without the raw URL list.

---

## Topic 1 — Hierarchical/nested tenancy models at scale + tree representation

### Precedent data models (all verified to use adjacency-list-as-truth)

| Platform | Tree unit | Depth limit | Policy-inheritance semantic | Authoritative encoding |
|---|---|---|---|---|
| **AWS Organizations** | OU / account | **5 levels under root** [2][11] | **Intersection-of-allows + explicit-deny**: an action is allowed only if EVERY SCP from root→account allows it, and ANY deny anywhere on the path denies it. Default `FullAWSAccess` SCP = allow-all until custom SCPs attach, which flips to deny-by-default [1] | adjacency list (account stores parent; "move account" = reparent) [1] |
| **GCP Resource Hierarchy** | org → folder → project | **folders 10 deep; ≤300 direct child folders per parent** [4] | **Union-of-allows**: effective policy = union of the resource's own bindings + all inherited ancestor bindings; authorizations *accumulate* downward [3] | adjacency list + inheritance traversal [3][20] |
| **Azure Management Groups** | mgmt group / subscription | **6 levels** (excl. root + subscription); CAF recommends 3–4 [5] | Inherited Azure Policy assignments flow down a subtree; **deny assignments** act as non-overridable high-level constraints [5][6-azure][13] | tree rooted at tenant root [5][12] |
| **Kubernetes HNC** | namespace / subnamespace | no published global limit (capacity-bound) [7][8] | **Propagation/union**: parent RBAC `Role`/`RoleBinding`, `NetworkPolicy`, `LimitRange` are *copied* into descendant namespaces; "tree" depth labels are added to namespaces [7][8] | parent-ref + propagated objects; no explicit-deny (relies on default-deny NetworkPolicy) [7][8] |
| **Salesforce** | role hierarchy + account hierarchy (Parent Account) | external account hierarchy **5 levels** (deeper = contact AE) [10] | **Roll-up VISIBILITY (upward)**: ancestors see records owned by descendants (manager sees subordinate's records); parent account sees child-account-owned data. No top-down policy push [9][10][14-sf] | adjacency list (Parent Account field) [14-sf] |
| **MSSP / B2B SaaS** | partner → sub-partner → customer | typically none published [15] | needs BOTH directions: parent imposes baselines DOWN + parent sees aggregated data UP [15] | `parent_tenant_id` adjacency + recursive traversal [17] |

**Key takeaway:** the precedents split into two inheritance philosophies — **top-down constraint (AWS/Azure, intersection+deny)** and **top-down accumulation (GCP/HNC, union)** — plus **bottom-up visibility roll-up (Salesforce)**. An MSSP platform like Prism needs the AWS-style *constraint* semantic for config/policy DOWN and the Salesforce-style *visibility* semantic UP — but with the up-direction gated by explicit consent (Topic 4/5).

### Tree-encoding tradeoffs for auth/isolation queries

The dominant authorization queries are **"is X an ancestor of Y?"** (policy path resolution, RBAC scoping) and **"enumerate all descendants of X"** (subtree policy push, aggregated roll-up, blast-radius). Reparenting (move a tenant) is a real but not-constant write. Per bool.dev's hierarchical-storage analysis and Celko/ScienceDirect on nested sets [16][17]:

| Encoding | is-ancestor | all-descendants | reparent cost | Fit for Prism |
|---|---|---|---|---|
| **Adjacency list** (`parent_id`) | poor (recursive CTE; degrades with depth) | poor/moderate | **low** (one pointer) | authoritative truth, but insufficient alone for unlimited depth [17] |
| **Materialized path** (`/1/3/3.2`) | good (prefix compare) | good (prefix scan) | **high** (rewrite all descendant paths on move) | great for *sharding/cell placement* by prefix; lazy-derived [17] |
| **Closure table** (`ancestor,descendant,depth`) | **excellent** (single row lookup) | **excellent** (single index scan) | moderate→high (rewrite subtree's pairs) | **best for auth/RBAC/policy resolution** [17] |
| **Nested set** (`lft`/`rgt`) | excellent (range) | excellent (range) | **very high** (renumber on every insert/move) | rejected — too write-hostile for dynamic tenant onboarding [16][17] |

**Recommendation (Topic-1 lean):** adjacency list (truth) + closure table (auth index) + optional materialized path (cell/shard placement). This is the standard production composition for deep, write-active authorization hierarchies [17].

---

## Topic 2 — Isolation-depth taxonomy (pool / bridge / silo / cell)

AWS SaaS-Lens defines the canonical taxonomy [18]:

- **Pool** — tenants share resources; isolation via tenant-id column / row-level filters / access-control. Max utilization + simplest ops; **weakest isolation**, compliance-harder, cross-tenant-leak risk if authz fails.
- **Silo** — dedicated resources (separate DB / instance / stack) per tenant. **Strongest isolation**, easiest regulatory story; highest cost + ops overhead (many stacks).
- **Bridge** — *mixed*: some services/tenants siloed, others pooled. AWS explicitly says most real SaaS is bridge. This is the lever that lets **isolation depth vary by tier** [18].
- **Cell** — dedicated full stack per *tenant-group* (extension of silo to a group); partition by geography / regulatory domain / tier; strong blast-radius + fault isolation; sits between per-tenant silo and shared pool.

**Can isolation depth vary by level? YES — explicitly.** The bridge model's whole point is that high-risk / regulated tenants get silo/cell while low-risk tenants stay pooled [18]. For a *nested* model this maps cleanly: isolation tier becomes a **per-node attribute** on the tenant, independent of where it sits in the tree.

### Mapping to Prism's stores

| Prism store | Natural isolation knob |
|---|---|
| **PostgreSQL (Central control-plane: config, RBAC, cases, audit, tenant tree)** | **pool** with row-level `org_id` scoping + the closure table = default; a silo customer gets a dedicated schema (bridge) or dedicated DB (silo) |
| **RocksDB (satellite hot data-plane, RetentionCache)** | satellite-local = **inherently siloed/cell per satellite** (a satellite serves one customer/BU footprint); column-family + org-key scoping within a shared satellite if a satellite is multi-tenant |
| **Iceberg (cold RETAIN tier)** | per-tenant table namespace = **pool-with-partition**; per-tenant encryption key = silo-grade confidentiality even in shared storage |
| **Context stores (indradb / usearch / lancedb)** | satellite/edge-local, per-tenant graphs/vectors (PIV-C12-5) = **siloed at edge** |

**Net:** Prism is *already a bridge* — satellite data-plane is silo/cell-grade by construction (D-C2-12: Central never stores raw sensor data), and the Central control-plane is pool-with-row-scoping. Nesting does not change the isolation *mechanism*; it adds an **isolation-tier attribute per tenant node** that the bridge model legitimizes.

---

## Topic 3 — Config / policy inheritance + override down the tree

Two precedent semantics (Topic 1):
- **AWS SCP intersection+deny** [1]: parent sets a *ceiling*; child can only *narrow* within it; any ancestor deny is final. "Effective policy at a leaf" = AND of all ancestor allows MINUS any ancestor deny.
- **GCP IAM union** [3]: parent grants *accumulate* downward; child adds but the inherited bindings persist. "Effective policy at a leaf" = OR of all ancestor bindings.

**Effective-config computation at a leaf (recommended):** walk the closure-table ancestor set root→leaf (the closure table gives this in one scan), then fold each level's config layer with **child-overrides-parent for values, parent-deny-is-final for guardrails**. This is a hybrid: *values* inherit-then-override (GCP-flavor, ergonomic), *security guardrails* intersect-and-deny (AWS-flavor, safe). HNC's "copy parent objects into child" propagation [7][8] is the runtime analog — but Prism should compute effective-config at *plan time* from the authoritative tree rather than physically copying, to keep Config-DB-Authoritative (C9) clean and avoid drift.

**Ties C9:** the tenant tree + per-node config layers live in the Central DB (DB-authoritative), are authored only at Central UI (PIV-C9-001), and push the *computed effective config* down to satellites (satellites are auto-receivers, never authors). Inheritance is a Central-side computation; satellites receive a flattened result.

---

## Topic 4 — RBAC across the hierarchy (ties C18)

### Delegated administration — explicit + consented, bounded

- **The MSSP precedent to copy is Cortex XDR parent-child pairing** [14]: the parent *requests* pairing; the **child admin must approve** before the parent can manage the child. Authority is never implicit-by-position. This directly answers "can a parent (MSSP) admin act on children?" — **yes, but only on children that have consented**, and the grant is scoped to that pairing.
- **Privilege containment (no escalation UP or to siblings)** uses two verified primitives [2][6]:
  - **Permission boundaries** — a delegated admin's effective permissions = identity-policy ∩ boundary; they can NEVER exceed the boundary even if they can edit policies. A sub-partner admin's boundary excludes the partner account and sibling sub-partners [2].
  - **Confused-deputy guards** — cross-tenant access is conditioned on source-org / source-path (AWS `aws:SourceOrgID` / `aws:SourceOrgPaths`), so a parent role can only reach its own subtree, never escalate to *its* parent or to siblings [6].
- **Role hierarchy** mirrors the tenant tree [7]; cross-tenant delegated roles are an established pattern (Microsoft 365 EDU multi-tenant, Okta multi-org "maximum delegated responsibility") [4][8].

### Mapping to Prism's 7-role RBAC (C18)

Prism's existing 7-role model becomes **role × scope-node**: a role binding is `(role, tenant_node)` and the closure table determines whether that binding is in-scope for a target resource. An MSSP partner-admin holds bindings on the partner node; the closure table grants them reach over descendants *that have paired*; permission-boundary semantics forbid reach over ancestors/siblings. Cross-tenant *view* (parent seeing aggregated child data) is a distinct, explicitly-granted, consented capability — see Topic 5.

---

## Topic 5 — Isolation invariants under nesting + key custody

### Invariants

1. **No sibling bleed.** Two children of the same parent never see each other's data — enforced by per-node `org_id` scoping + per-child keys, NOT by tree position.
2. **Parent→child access is explicit + governed, never implicit.** Hierarchy position grants *manageability via pairing* (Topic 4) and *aggregate visibility via consent* (below) — it does NOT auto-grant raw-data read. This is the Cortex-XDR + permission-boundary principle [2][6][14].
3. **Reconciling parent aggregate-view with BYOC zero-access:** parents see **derived metrics on separate analytics/metadata**, not raw rows [18][19][20]. The operator/vendor still has zero at-rest access (D-C2-12); a *parent tenant* differs from the *operator* and may, with child consent, see aggregates — but never the operator.

### Key custody under nesting (the crisp answer)

**Per-CHILD data key, even in deep nesting.** Cloud-KMS / CMK best practice is "never use the same DEK for two different users/tenants" [13], and CMK is *defined* as per-tenant encryption with tenant-controlled master keys [11]. Therefore:

- A child's data is encrypted under the **child's** DEK/KEK — **NOT** a parent-derived key [9][10][11][13].
- **Whose key for the Option-3 tenant-keyed cache?** → **the child's** (the tenant whose data it is), under SS-26's per-tenant DEK. The cache key is the *data-owning* tenant, never the parent. This is the cryptographic enforcement of P-ADS-06 Per-Tenant-Isolation and is consistent with the existing S3 conversation-store Option-3 precedent (per-tenant DEK, operator zero-access).
- **Parent aggregate visibility WITHOUT holding the child key** — three verified patterns [11][12][15]:
  1. **Query-time scoped decryption** under the child key, transient + audited (CMK call / KMS grant; child can revoke; every access logged child-side) [11][15].
  2. **Re-encrypt the aggregate result** under the parent's key — the parent reads pre-computed metrics encrypted to *its* key, never touching child raw data [10][13].
  3. **BYOC remote-op** — for client-managed deployments the parent never holds keys at all; it runs a bounded remote operation in the child's cloud [12].
- **Is per-child DEK right even when deeply nested?** **Yes.** Deeper nesting makes per-child isolation *more* important (a shared key's blast radius spans more tenants). Cost/ops can be tempered with intermediate KEK grouping (per-sub-partner KEK wrapping per-customer DEKs) but the leaf DEK must never be shared across tenants [9][13].

---

## Topic 6 — Metering / billing roll-up (ties C11)

**Record at the leaf, aggregate up the ancestor relationship** — the same closure table the auth path uses [1][19][20]:

- Cloud precedent: AWS bills at the account/resource (leaf) level; **Cost Categories / consolidated billing** group and roll up to internal business structures (partner / cost-center) [1][19]. GCP records per-project usage and aggregates through the org hierarchy [20].
- SaaS pattern: every metered event (query, fan-out, retained-byte, action) carries the leaf `org_id` (+ optional ancestor tags); aggregation sums along the ancestor edges to produce per-sub-partner and per-partner roll-ups [18][19].
- **Critically, metering roll-up runs on METADATA, separate from data isolation** [18][19][20]. A partner can receive a consolidated usage/cost report for its subtree *without* any read access to descendant raw data or keys. Billing roles are distinct from data/config roles (Topic 4).

For Prism: the metering subsystem (C11) tags each event with the leaf org and walks the closure table for roll-up; the report is a parent-scoped derived artifact, not a data-plane query.

---

## Topic 7 — Central-Sole-Surface fit

- Nested tenants are administered **entirely at Central**; the tenant tree lives in the Central control-plane PostgreSQL (Config-DB-Authoritative); satellites remain **headless** auto-receivers (Constraints 1 & 2 from the ripple analysis).
- **Navigation for a deeply-nested org:** the analyst/admin selects a **scope node** in the tree at the Central UI; the closure table bounds (a) what config they can author/view, (b) which descendants they can act on (paired only), (c) which aggregated metrics roll up to them. This is the management-group-tree-picker UX pattern (Azure CAF) but driven by the closure table [5].
- **No new user-login surface** is introduced by nesting — a sub-partner admin logs into the *same* Central, scoped by their node bindings. Satellite SPIFFE identity stays machine-identity; nesting is a control-plane construct only.
- Deep-depth UX guardrails: the vendors that capped depth did so for *debuggability* [2][4][5]. Even if Prism permits unlimited depth, the Central UI should provide ancestor-chain breadcrumbs, inherited-vs-overridden config diffing (GCP "view inherited policy" pattern [3]), and effective-config preview at a node.

---

## ANALYSIS + LEANS

### Recommended nested-tenancy model

1. **Tree representation:** **adjacency list (authoritative `parent_id` on the tenant/org node) + closure table (`ancestor, descendant, depth`) for auth/RBAC/policy/metering queries + optional materialized path for cell/shard placement.** Reject nested-set (write-hostile to onboarding) [16][17]. This is the standard production composition for deep, write-active authorization trees and gives O(1)-ish `is-ancestor` and single-scan `all-descendants` [17].
2. **Isolation-depth strategy:** **bridge by default, with isolation tier as a per-node tenant attribute** — it VARIES by level/tenant. Prism is already a bridge: satellite data-plane is silo/cell-grade by construction (D-C2-12), Central control-plane is pool-with-row-scoping. Nesting adds an `isolation_tier` attribute (pool | silo | cell) per node, legitimized by AWS SaaS-Lens [18].
3. **Config/RBAC inheritance:** **hybrid** — *values* inherit-then-child-override (GCP-ergonomic), *security guardrails* intersect-and-parent-deny-is-final (AWS-safe) [1][3]. Effective config computed at Central plan-time by folding the closure-table ancestor set; satellites receive the flattened result (C9 clean). RBAC = `(role, scope-node)` bindings filtered by the closure table, bounded by permission-boundary + confused-deputy semantics so no escalation up/sideways [2][6].
4. **Isolation invariants:** (a) no sibling bleed (per-node scope + per-child key); (b) parent→child *manage* authority requires **explicit child-admin-approved pairing** (Cortex XDR model) [14]; (c) parent→child *aggregate view* requires explicit consent and reads derived metrics only, never raw rows; (d) operator/vendor retains zero at-rest access regardless of tenant hierarchy.
5. **Key custody under nesting:** **per-CHILD DEK, always — the cache/data key is the data-owning (leaf) tenant, never the parent** [9][11][13]. Parent aggregate visibility via transient scoped-decryption / re-encrypted-aggregate / BYOC-remote-op [11][12][15]. This makes the Option-3 tenant-keyed cache key unambiguous: **child-keyed.**
6. **Metering roll-up:** record at the leaf with `org_id`, aggregate up the closure-table ancestor edges on metadata separate from data isolation; billing roles distinct from data/config roles [18][19][20].

### How this CLOSES OQ-DEPLOY-1

OQ-DEPLOY-1 asked "tenancy-isolation depth: pool/bridge/silo/cell?" The research resolves it: **the question presupposed a single global choice; the correct answer is bridge with isolation tier as a per-tenant-node attribute, orthogonal to the logical hierarchy.** Prism already implements the strongest tier (satellite silo/cell, D-C2-12) on the data-plane and pool-with-row-scoping on the control-plane. Nesting is supported by the adjacency+closure tree on the existing OrgId/OrgSlug abstraction; no new isolation mechanism is required — only an `isolation_tier` node attribute and the closure-table index. **OQ-DEPLOY-1 → RESOLVED: bridge, per-node tier.**

### Genuine sub-forks needing a HUMAN decision

- **SF-1 — Inheritance semantic for security guardrails:** the hybrid leans AWS-style intersection+deny for guardrails. Confirm that a parent guardrail must be a *ceiling a child cannot widen* (recommended), vs GCP-style accumulate where children can be granted beyond the parent. (Affects whether an MSSP can hard-cap a customer's capabilities.)
- **SF-2 — Unbounded depth vs soft cap:** every vendor capped depth (5–10) for debuggability [2][4][5][10]. Recommend supporting unlimited depth *technically* (closure table doesn't care) but imposing a **configurable soft cap (e.g., default 8) with override** to deter pathological trees. Human: accept unbounded, or set a default soft cap?
- **SF-3 — Parent aggregate-visibility default:** opt-in per child (Cortex-XDR-style explicit consent, strongest) vs tenant-tier policy (parent of a "managed" tier auto-sees aggregates). Recommend **opt-in per child** to preserve the BYOC zero-access thesis cleanly. Human: per-child consent, or tier-policy default?
- **SF-4 — KEK grouping for cost at scale:** per-leaf DEK is mandatory; the open question is whether to introduce per-sub-partner KEKs wrapping per-customer DEKs for ops/cost at very large fan-out [9][13]. Pure-architecture (no isolation weakening) but adds key-hierarchy depth to SS-26. Human/architect: flat per-tenant KEK, or nested KEK hierarchy mirroring the tenant tree?
- **SF-5 — Reparenting policy:** moving a tenant subtree rewrites closure-table pairs and may change effective config + key scoping. Confirm reparenting is an *admin-only, audited, Central-authored* operation with explicit re-pairing/re-consent for any new ancestor (recommended — never silently inherit a new parent's guardrails).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) resource-hierarchy precedents + tree-encoding tradeoffs + isolation taxonomy (reasoning_effort high); (2) RBAC delegation/privilege-containment + envelope-encryption key custody under nesting + metering roll-up (reasoning_effort medium). Both returned 85k/76k-char cited deep results. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | Not applicable — architecture-pattern research, no specific library API in scope. |
| Tavily (any) | 0 | Not needed — two deep-research passes plus in-repo cross-grounding were sufficient; cross-source agreement was high (AWS/GCP/Azure docs corroborate the same tree+inheritance patterns). |
| WebFetch / WebSearch | 0 | — |
| Read (in-repo) | 4 files | Grounding against Prism's established model (OrgId/OrgSlug, SS-26 DEK, BYOC zero-access, Central-Sole-Surface, 7-role RBAC, Option-3 cache). |
| Training data | 2 areas | (a) general framing of closure-table vs adjacency-list (corroborated by cited bool.dev/Celko in the research output); (b) mapping precedents to Prism's specific stores (clearly flagged as inference, not vendor-documented). |

**Total MCP tool calls:** 2 (both `perplexity_research`, the mandated PRIMARY tool).
**Training data reliance:** low — every non-obvious external claim (depth limits, SCP intersection-vs-deny, GCP union, HNC propagation, Salesforce roll-up, Cortex-XDR pairing, permission-boundary/confused-deputy, per-tenant DEK / CMK, AWS Cost Categories) is sourced to the deep-research citation set with the originating vendor/source named inline. Inference (Prism-store mapping, hybrid inheritance recommendation, Option-3 child-keying) is explicitly labeled as analysis/lean, not vendor fact.

> **Honesty flag on source completeness:** the two deep-research results exceeded the readable-token cap; I read the leading ~64–66k chars of each (covering all substantive claims + comparison tables + inline citation anchors) but did not fully paginate the trailing recap/raw-URL sections. The substance of those tails was already present in the read portion. This is a Level-2 partial-coverage flag, not an inconclusive result — every claim cited above appears in the portion I read.
