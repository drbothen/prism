---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
program: day-2-vision SIDE-ANALYSIS (OUT-OF-BAND; SEPARATE from the live VSDD factory pipeline)
pillar: C9 — Config-Management
scope_fence: matured-vision-day2-requirements.md §11.1 (BYO secret-store), §11.2 (config store), §3.1 (central-deployment pivot), §14.3 (storage taxonomy — DECIDED, not reopened here)
storage_taxonomy_constraint: "DECIDED — config store = bundled PostgreSQL (central) / embedded SQLite (satellite-edge) per ADR-PROP-storage-engine-taxonomy.md §14.3. This research does NOT reopen the storage-engine choice."
non_contradiction_reads:
  - matured-vision-day2-requirements.md §3.1, §11.0, §11.1, §11.2, §11.3, §13.6, §14.3
  - project memory project_config_crate_decision (prism-config custom crate, lift-from/avoid figment+config-rs)
  - CLAUDE.md (AD-007 ArcSwap hot-reload; AD-017 AI-opaque reference-based credentials; #[non_exhaustive] discipline; eat-our-own-dogfood TOML connectors)
# CAPTURE artifact. Records cited prior art + discussion LEANS for the OPEN config-management
# pillar design questions. Modifies no live spec/BC/ADR/story/STATE/SESSION-HANDOFF/RESEARCH-INDEX.
# Leans are discussion input only — not decisions.
---

# Config-Management Pillar — Depth Research (C9)

> **READ FIRST.** This is an out-of-band side-analysis CAPTURE for the day-2 vision. It is `do_not_execute`.
> It does NOT modify any live artifact and was NOT added to RESEARCH-INDEX.md (per the hard boundary on this dispatch).
> The storage-engine choice (Postgres central / SQLite satellite) is SETTLED in §14.3 and is treated as a fixed
> premise throughout. The deliverable is the *config-management subsystem shape on top of* that storage, the
> Git-vs-store authority fork, hot-reload mechanics, schema/validation, fleet distribution, and audit/blast-radius.

**Confidence legend:** [web] = verified web/doc finding with citation · [c7] = Context7 library-docs verified · [model-knowledge] = model knowledge, not independently re-verified this pass · [INCONCLUSIVE] = could not verify.

**Landscape date:** findings current as of 2026-06-27. The GitOps/detection-as-code and Rust config-crate landscapes move quickly; version pins are spot-checked below.

---

## 0. Scope reconciliation with what is already DECIDED

So that the lean does not collide with settled day-2 decisions:

- **§14.3 storage taxonomy (DECIDED).** PostgreSQL is the *bundled, central-only* relational CONTROL-PLANE that holds — verbatim from §14.3 — "central config store (§11.2), RBAC, audit log, tenant/user, identity/AS state, result-cache METADATA." SQLite (embedded) is the "Satellite-local CONTROL-PLANE … local config, enrollment/identity state, local policy + operational metadata." This research takes that as fixed. The question C9 leaves OPEN is *the management model around that store* — not the store.
- **§11.1 secret-store stance (HUMAN-CONFIRMED).** Hybrid: built-in encrypted store AND external-vault backends, via a pluggable `SecretBackend` trait; reference-based, AI-opaque (AD-017) preserved; per-tenant DEK isolation. Config stores **references**, never secret values. This research treats secret *values* as out-of-scope for the config plane and only addresses secret *references* in config (Q6).
- **§3.1 / DC-005 central pivot.** Per-laptop TOML + ArcSwap hot-reload "do not generalize to multi-tenant central config with RBAC, audit, and versioned change control" (§11.2). The config pillar must serve BOTH the surviving single-analyst stdio mode (file + ArcSwap) AND the new central multi-tenant service (Postgres store + API/UI) AND the satellite edge (SQLite). That tri-modal target is the spine of every answer below.

---

## 1. The Config-Management Pillar Shape (layered model, static vs dynamic boundary)

### 1.1 Cited prior art — the static-bootstrap vs dynamic-store boundary

The single most consistent pattern across production data/observability/security platforms is a **two-tier split**: a *minimal static bootstrap file* whose only real job is to identify the process and point it at its dynamic control plane, and a *dynamic runtime store* that is authoritative for everything that changes during the process's life.

- **Envoy xDS** is the canonical example. The `bootstrap` file "defines core properties such as admin interfaces, initial clusters, listeners, and the xDS server endpoints, which Envoy uses to connect to its control plane"; thereafter "cluster and other resources can be provided dynamically via xDS and have a defined priority relative to static resources." A documented constraint: some Cluster xDS resources "must be specified first in the `static_resources` field of the bootstrap," underscoring that the bootstrap must at minimum define *how the process reaches its control plane*. [web]
- **OpenTelemetry Collector + OpAMP.** OpAMP ("a network protocol for remote management of large fleets of data collection agents") inverts the file: "an OpenTelemetry agent's bootstrap file primarily configures its OpAMP connection parameters and identity, while the server's database-backed configuration store becomes authoritative for its ongoing configuration." [web]
- **Elastic Agent + Fleet.** "The initial agent installation uses a local YAML configuration and enrollment token to connect to Fleet, after which the agent receives policies dynamically from the Elasticsearch-backed Fleet server … the local file is largely a bootstrap to connect the agent to its central configuration authority." [web]
- **Grafana** is the counter-pole: provisioning files "are then written into Grafana's database, and at runtime Grafana's DB state is typically treated as authoritative." [web] (This DB-authoritative posture is the fork in Q2.)
- **Kubernetes ConfigMaps / Consul KV.** ConfigMaps "store non-confidential configuration as key-value data … consumed as environment variables, command-line arguments, or configuration files," with "precedence via pod spec reference order." Consul KV / Nomad "function as runtime configuration sources that can override static configuration based on central policy or operator actions initiated via a UI or API." [web]

**Synthesized boundary rule (from the prior art):** the bootstrap file should hold "static contexts like listen addresses, TLS certificates, and enrollment tokens, and should not encode fast-changing policy or per-tenant settings"; the database is "authoritative at runtime; provisioning is bootstrap / import of desired state." [web]

### 1.2 Canonical layered precedence

The layered-source order Prism already lifts from config-rs (CLI > env > TOML > defaults, per `project_config_crate_decision`) matches the cross-platform consensus. The day-2 addition is a **store layer and a runtime-override layer** sitting *above* file/env for the central mode. A defensible precedence for the tri-modal platform:

```
defaults  <  file/TOML (bootstrap)  <  env  <  config store (Postgres/SQLite)  <  runtime override (API/UI mutation)
   (lowest authority)                                                                  (highest authority)
```

with the hard caveat that **bootstrap-class keys are NOT overridable by the store** (Q3): the store cannot move the listen port or swap the TLS cert out from under a running listener; Envoy enforces exactly this asymmetry ("changing bootstrap parameters generally requires a process restart … dynamic xDS updates … do not change Envoy's core identity or startup behavior"). [web]

### 1.3 What belongs where (bootstrap file vs runtime store)

| Belongs in **bootstrap file** (static, restart-class) | Belongs in **runtime store** (dynamic, hot-mutable) |
|---|---|
| Listen address/port, TLS cert/key, transport selection (stdio vs HTTP) | Connector enable/disable, connector endpoints, pushdown descriptors |
| Identity / enrollment token, store connection string, KMS master-key reference | Detection rules + thresholds, retention/RETAIN policies |
| OAuth AS/IdP discovery roots, DEK/KMS hierarchy bootstrap | Per-tenant overrides, RBAC role assignments, schema mappings (configure-schema) |
| Process budgets that pin allocator/runtime behavior at boot | Log-level, feature-flag toggles, availability-cache TTLs |

This maps cleanly onto §14.3: bootstrap is a small TOML the operator ships with the appliance; the runtime store is Postgres (central) / SQLite (satellite). Note this is the **config-plane / data-plane separation** §11.2 already names as an E-CENTRAL-OPS-001 deliverable.

### 1.4 Q1 LEAN

Adopt the **Envoy/OpAMP minimal-bootstrap model explicitly**: a small, typed, fail-closed bootstrap TOML whose contract is "identity + how to reach the store + restart-class invariants," and a Postgres/SQLite runtime store that is authoritative for all hot-mutable config. Keep the existing single-analyst stdio path as "bootstrap file is also the runtime store" (degenerate case — no central store, ArcSwap over the file). Make the **bootstrap-class key set an enumerated, `#[non_exhaustive]` allowlist** so the "what cannot be hot-reloaded" boundary (Q3) is a compile-time/schema-time fact, not folklore. This is consistent with §11.2 and reuses the dogfood TOML model for the bootstrap layer.

---

## 2. GitOps for Config + Detections-as-Code — THE CENTRAL FORK

### 2.1 Cited prior art — the two poles

The research is unusually crisp that this is a genuine, named architectural fork: "The dominant tension is the authority split between Git and databases: Kubernetes-style GitOps makes the cluster state derived from Git and subject to reconciliation, while platforms like Grafana and Kibana treat their databases as authoritative configuration sources with file provisioning acting as bootstrap or one-way import." [web]

**Pole A — Git-authoritative, store is a materialized cache (Argo CD / Flux / detection-as-code):**
- Argo CD "treats Git as the source of truth for the desired state … self-healing sync policy continuously monitors the cluster for drift and automatically corrects discrepancies." Drift is detected by "querying the Kubernetes API for the current state and comparing it against the desired state declared in the application manifest from Git." Default reconcile interval ~3 minutes; pruning removes resources deleted from Git; **rollback is a Git revert**, not a DB edit. "etcd holds the materialized view of Git's configuration, and Argo CD ensures that this view is convergent." [web]
- **Detection-as-code** is the security-domain instance: "Elastic's detection-as-code and Panther's detection-as-code both rely on Git for collaborative review and audit." For Elastic specifically, "Git [is] the authoritative source for detection configurations, while Elastic's backend database stores the materialized runtime representation"; "if a detection is manually edited in the database, this would be considered drift from the Git repository and could be overwritten." [web]
- **Terraform + Atlantis** is the same philosophy for infra: "Terraform configuration files define desired state, Git holds the authoritative state, and Atlantis … mediate[s] plan and apply operations in response to Git changes." [web]

**Pole B — DB-authoritative, Git is audit/export/bootstrap (Grafana / Kibana-Fleet):**
- "Grafana deviates by treating its own database as the authority at runtime, though it provides GitOps-friendly provisioning that can seed or reconcile data sources and dashboards from files." "Users can create data sources and dashboards directly in the UI, and those changes may not be immediately reflected in the provisioning files." [web]
- "Elastic Fleet and Kibana saved objects follow a similar DB-authoritative pattern … operators editing policies through Kibana's UI or API rather than via Git alone." [web]

### 2.2 The decisive tension for Prism: live runtime mutation from a UI/API

The research names exactly Prism's problem: "In a multi-tenant platform where tenants expect to toggle connectors, update credentials, or tweak detection thresholds via a UI, it is often impractical to require every change to flow through Git and PRs. A strict Git-authoritative model would imply that the UI writes changes back to Git, commits them, and triggers CI pipelines, which can create latency and complexity … [auto-commit] can cause repository noise and complicate human review." [web]

Conversely, the DB-authoritative pole's documented weakness is audit: "Unless the platform implements robust database-level audit logging and snapshotting, it may be harder to answer 'who changed what when' or to roll back to previous configuration states." [web] — which is precisely the §11.2 ENHANCEMENT requirement (versioned change control + audit + rollback) that the §14.3 Postgres control-plane audit table is meant to satisfy.

### 2.3 What real platforms actually ship: a domain-split hybrid

The research's strongest and most actionable conclusion: **don't pick one pole globally — split by configuration domain.** "Where configuration is global, high-risk, and relatively slow-changing — such as detection logic, pipeline structures, and baseline policies — Git-authoritative models are strongly beneficial, with the database treated as a cache or runtime representation." But "per-tenant runtime state, including connector toggles, credentials (represented via references), and tenant-specific thresholds, should be DB-authoritative, with Git used to export snapshots and audit major changes." The designer's job is "to clearly define which configuration domains are Git-managed and which are DB-managed, and to ensure that the boundaries between them are explicit, versioned, and auditable." [web] This is exactly how Elastic ships (detection rules Git-authoritative; Fleet agent policy DB-authoritative).

### 2.4 Q2 LEAN — the position on the central fork

**Domain-split hybrid, leaning Git-authoritative-with-store-as-cache for the global/high-blast-radius domain, DB-authoritative-with-Git-audit for the per-tenant/runtime-mutable domain.** Concretely:

- **Git-authoritative (store = materialized projection):** detection rules, retention/RETAIN policies, connector *definitions* (the TOML specs Prism already dogfoods — these ARE config-as-code today), pushdown capability descriptors (C3), configure-schema mappings as committed artifacts, satellite topology/trust policy, baseline global RBAC role definitions. These are global, high-blast-radius, slow-changing, and benefit maximally from PR review + CI validation + Git-revert rollback. The Postgres/SQLite store holds the *compiled/active projection*; a reconcile loop converges it to Git; manual store edits in this domain are **drift** (surfaced, optionally self-healed).
- **DB-authoritative (Git = audit export):** per-tenant connector enable/disable toggles, per-tenant threshold overrides, credential *references* and rotation state, ephemeral operational state (availability-cache TTLs, feature-flag toggles per tenant), RBAC role *assignments* to users. These originate from UI/API actions, must apply with sub-second latency, and would be intolerable to route through PR. The Postgres control-plane audit table (§14.3) supplies the "who-changed-what-when" that Git would otherwise provide; periodic Git/object-store snapshot export gives the GitOps-grade audit trail that pure-DB platforms lack.

This is defensible because it is **what Elastic itself ships** (its detection-rules repo is Git-authoritative; its Fleet agent policies are DB-authoritative) [web], and it directly honors §11.2's two simultaneous requirements: "declarative / GitOps config (enhancement over Query)" AND "multi-tenant hot-reload … without affecting other tenants." The fence line between the two domains MUST be an explicit, enumerated, versioned classification on every config key (binds to the Q1 bootstrap-allowlist discipline).

**Why not pure Git-authoritative for everything:** the UI-driven credential-rotate / connector-toggle path (a hard §11.1/§11.2 requirement) makes auto-commit-per-UI-action the only way to stay pure-Git, and the research flags that as repository-noise + latency + review-pollution. [web] **Why not pure DB-authoritative for everything:** loses Git-grade audit/review/rollback for exactly the high-blast-radius detection/policy domain where the CrowdStrike lesson (Q6) bites hardest.

---

## 3. Safe Hot-Reload Mechanics (Rust, ArcSwap)

### 3.1 Crate state verified this pass

- **arc-swap 1.9.1** — verified on crates.io [web] and via Context7 docs [c7]. The consistency contract that AD-007 depends on is documented directly: `load()` returns a `Guard` temporary borrow; the docs warn that calling `load()` twice can combine values "from different points, combining into an invalid point that never existed," and the rule is "call `load` just once and keep the result." [c7] `load_full()` returns an owned `Arc` clone, "lock-free and wait-free but usually more expensive than `load`"; the docs explicitly say `load`'s cheap guard is "suited for local variables on stack, but not in long-living data structures." [c7] **Implication for AD-017/AD-007:** an in-flight query that must hold a config snapshot across its whole lifetime should hold `load_full()` (owned `Arc`), NOT a `Guard` — the Guard is the read-mostly fast path, not the long-lived snapshot. This is a load-bearing nuance worth pinning in the morph BC. [c7]
- **notify 7.0.0** [web, INCONCLUSIVE on exact patch] + **notify-debouncer-full** — Context7 confirms `new_debouncer(timeout, tick_rate, handler)` over `RecommendedWatcher` + `RecommendedCache`, and that the debouncer "ensures only a single `Rename` event is emitted," dedupes "Create and Modify events," and emits one Remove for directory deletes. [c7] This matters because **editors save config atomically via rename/temp-file** (write tmp, rename over target), which a raw watcher reports as Remove+Create or Modify(Name) pairs; the debouncer collapses these so reload fires once on the settled file, not mid-write. The Perplexity-reported `notify-debouncer-full = 0.1.0` is **[INCONCLUSIVE]** — that pin looks stale and should be re-verified against crates.io at morph time; do not pin from this report.

### 3.2 Cited prior art — validate-before-swap + atomic snapshot

Envoy is the canonical safe-hot-reload reference and the discipline is explicit: "Envoy treats these updates as atomic snapshots: each xDS response contains a new versioned set of resources, and Envoy applies them only if they pass validation against the xDS resource schema and internal consistency checks. If a new configuration snapshot fails validation … Envoy rejects the update and continues running with the previous configuration, thereby preventing crashes or partial application." [web] OpenTelemetry Collector parses config "into a strongly typed internal representation, with invalid component references or type mismatches resulting in startup failures" — fail-closed. [web] Elastic Fleet "validates agent policies before distributing them, ensuring that invalid configurations do not propagate to agents." [web]

### 3.3 The safe hot-reload recipe (synthesized)

1. **Receive** new config (file change via debounced `notify`; OR store change via Postgres `LISTEN/NOTIFY` push, falling back to polling). The research endorses "agent-initiated outbound connections … combining push capabilities with agent-side polling for robustness." [web]
2. **Parse + validate into a typed snapshot** — fully, fail-closed. Validate cross-references (e.g. a detection rule referencing a connector that exists; a tenant override referencing a defined role). Envoy's "route references a nonexistent cluster → reject" is the model. [web]
3. **Stamp** the validated snapshot with a monotonic generation/version (Envoy versions every xDS resource set; the version is the audit and rollback anchor). [web]
4. **Atomic swap** only the validated snapshot via `ArcSwap::store()` (built on `swap` internally [c7]). Readers either see the whole old generation or the whole new one — never a torn mix.
5. **On validation failure: keep the previous snapshot running, emit a structured event, do NOT crash.** This is the single most important discipline and the one figment/config-rs do not give for free (Q4).

### 3.4 Hot-reloadable vs restart-class

The Envoy split generalizes directly: bootstrap-class keys (listen port, TLS material, transport selection, store connection string, identity/enrollment) are **restart-class** — "operators expect to restart Envoy when changing bootstrap-level settings." [web] Everything in the runtime store (connector toggles/endpoints, detection thresholds, retention policy, log level, per-tenant overrides) is **hot-reloadable**. Prism's existing AD-007 ArcSwap path already covers the hot side; the day-2 work is (a) wiring the Postgres `LISTEN/NOTIFY` → validate → swap path for central mode, and (b) the per-tenant snapshot generalization §11.2 calls for ("a single tenant's config reloads without affecting other tenants or in-flight queries").

### 3.5 Q3 LEAN

Make **validate-before-swap a hard, fail-closed invariant** (a bad config can never crash the process or partially apply — it logs + keeps the last-good generation). Use **`load_full()` for in-flight-query snapshots** (owned Arc held for the query lifetime), `load()` Guards only for short read-mostly hot paths. **Generation-stamp every snapshot** (the version is the rollback anchor and the audit key — ties to Q6). Use **debounced `notify` for the file path and Postgres `LISTEN/NOTIFY` + polling fallback for the store path.** Per-tenant config is a **per-tenant `ArcSwap` (map of `OrgId → ArcSwap<TenantConfig>`)** so one tenant's reload is a swap on its own cell — no global lock, no cross-tenant blast (this is the concrete realization of §11.2's "snapshot semantics preserved per tenant"). Enumerate the **restart-class key allowlist** (shared with the Q1 bootstrap allowlist) so "what must NOT be hot-reloadable" is schema-enforced.

---

## 4. Config Schema / Validation / Versioning + figment & config-rs anti-patterns

### 4.1 Documented figment / config-rs footguns (cited)

The research corroborates the `project_config_crate_decision` rationale with specific, observed anti-patterns:

- **Silent type coercion.** "Many developers have observed that figment and config-rs sometimes silently coerce types, such as treating strings as numeric or boolean values where conversions succeed, instead of failing explicitly." [web] (This is the config-rs anti-pattern #1 in the project memory; the research independently confirms it.)
- **Env-var ↔ nested-struct mapping confusion.** A figment issue reports it "is unable to correctly resolve an environment variable such as `APP_NODE_IDENTIFIER`" against a `node.identifier` nested struct — the snake_case/nesting mapping is a documented footgun. [web]
- **No first-class secret handling.** "config-rs and figment generally treat configuration values as regular strings, without dedicated secret types or secure handling strategies … This can encourage patterns where secrets are loaded and logged inadvertently, or where secrets are embedded in configuration files in plaintext, contrary to best practices in security platforms." [web]
- **Net verdict:** "while figment and config-rs are excellent for simple applications, they are not sufficient as the core configuration layer in a multi-tenant security or data platform." [web]

(Two project-memory anti-patterns — HashMap iteration nondeterminism and case-sensitivity — were not independently re-confirmed by name in this pass; treat those as [model-knowledge] carried from the 2026-04-14 decision. The figment "1,548-SLOC hand-rolled serde" and config-rs "10 anti-patterns" counts are from the prior brownfield ingest, not re-verified here.)

### 4.2 What a production-grade config layer does instead (cited requirements)

The research enumerates the requirements that a custom layer must add: "typed schemas; fail-closed validation; multi-error reporting; clear merge precedence; provenance-aware error messages; and secret-aware handling." [web] And the practical recommendation: "consider restricting environment variables to simple scalar overrides to avoid the figment-like confusion around nested structure mapping" and "provide a custom configuration crate that wraps figment or config-rs only for basic loading but imposes stricter semantics, or bypass those libraries entirely for core platform configuration." [web]

This is a near-verbatim match to the existing `prism-config` decision (custom crate; serde+toml+clap+secrecy; multi-error validation; secret redaction; deterministic BTreeMap; frozen-after-build; provenance/tag-identity from figment). The day-2 additions the research surfaces beyond the v1 crate:

- **Schema versioning + migration.** "Integrate configuration schema versioning and migration, inspired by Envoy's xDS versioning and Kubernetes' `apiVersion` model." [web] Envoy "configuration objects are tagged with their API version … allowing gradual migration of configuration schemas." [web] **Prism currently has no config-schema version field** — the day-2 store needs one (an `apiVersion`/`schema_version` on every stored config object) so that a platform upgrade can migrate old per-tenant configs forward without a flag day. This is a real gap relative to §11.2's "versioned change control."
- **`#[non_exhaustive]` on every config schema struct** — already a CLAUDE.md discipline (87-type gate); the store-backed config types inherit it. This is the Rust mechanism for forward-compatible schema evolution (new fields don't break deserialization of old configs and vice versa, with the wildcard match-arm discipline).
- **Provenance-aware errors** — the figment tag-identity pattern already lifted; for the store layer, provenance must extend to "which layer / which tenant override / which Git commit or store row this value came from" so an operator debugging an effective value can trace it.

### 4.3 Q4 LEAN

Keep `prism-config` custom (the decision is reaffirmed by the research, not challenged). For the day-2 store: **(a) add a mandatory `schema_version` to every stored config object and a forward-migration step at load** (Envoy/K8s `apiVersion` model); **(b) keep validation fail-closed + multi-error** (report every problem in one pass, never partial-apply — same discipline as the Q3 swap gate); **(c) extend provenance to "effective-value source"** (layer + tenant + Git-commit-or-store-row) so the UI/CLI can render "this value came from `<source>`"; **(d) restrict env vars to scalar overrides** to dodge the figment nested-mapping footgun. Do NOT reintroduce a runtime dependency on figment/config-rs for core config; the "wrap for basic loading only" option is acceptable but the existing custom crate already supersedes it.

---

## 5. Multi-Tenant + Satellite-Edge Config Distribution

### 5.1 Cited prior art — pull-from-edge over outbound-only

Every cited fleet system that works behind strict firewalls uses **agent-initiated outbound pull** (which is exactly the §3.2 dial-home mesh constraint):

- **Elastic Fleet:** "Agents connect to Fleet using enrollment tokens, then regularly query Fleet for policy updates … connections are typically agent-initiated, fitting outbound-only topologies." Tenant scoping via "spaces and privileges"; residency via "separate Fleet instances per region." [web]
- **OpAMP:** "Since agents connect outbound to OpAMP servers, this model fits outbound-only network topologies commonly found in security-sensitive environments." [web]
- **Consul/Nomad:** "Consul agents connect to Consul servers, typically over outbound connections from clients to servers … per-tenant isolation by mapping tenants to Consul namespaces or ACL roles." [web]
- **osquery/Fleet (fleetdm) pull model** and Chef/Puppet pull models are named as the same family. [web]

Net: "multi-tenant configuration distribution typically involves a central control plane with per-tenant and per-node configuration objects, plus edge agents that pull or receive configuration via outbound connections … the prior art suggests using agent-initiated outbound connections for central configuration, combining pull and push semantics." [web]

### 5.2 Residency-respecting distribution

The research is candid that residency for *config* (as opposed to data) is under-documented: "Data residency and regulatory constraints require that configuration distribution respect boundaries between regions and jurisdictions … the provided sources emphasize non-secret configuration more heavily." The pattern that exists is **region-pinned control planes**: "separate Fleet instances per region or environment," "Elastic's multi-region deployments." [web]

This dovetails with the *already-decided* §3.2 D-C2-12 residency invariant (raw normalized at the edge, only results transit; satellite-local credential resolution is a hard invariant) and D-C5-3 (residency = reject-at-plan-time). The config-plane corollary: **config that names region-bound resources, or that carries a secret reference resolvable only in-region, must not be pushed to an out-of-region satellite.** Because Prism stores secret *references* not values (§11.1, AD-017), the residency exposure on config is small — but a *reference* that points at an in-region vault path is still region-meaningful and must be classified.

### 5.3 Precedence: central policy vs satellite-local override

The Fleet/Consul prior art: "Agents receive a single effective policy that may reflect overlays, but the central … server decides precedence" / "the central Fleet server decides precedence." [web] The settled §14.3 SQLite-at-satellite holds "local config, enrollment/identity state, local policy + operational metadata," which implies satellites legitimately have *some* local config the center should not stomp.

### 5.4 Q5 LEAN

**Pull-from-edge over the existing dial-home conduit (D-C2-1), reusing the satellite-initiated reverse-RPC stream — do NOT invent a second config channel.** The coordinator computes a per-satellite *effective config bundle* (global Git-authoritative policy ∩ that satellite's tenant scope ∩ residency filter) and the satellite pulls it on heartbeat; the satellite materializes it into its SQLite control-plane and ArcSwaps it locally (same Q3 validate-before-swap discipline at the edge). **Precedence:** central policy wins for global/security-class config (detection rules, trust, pushdown); satellite-local wins for genuinely node-local operational config (local connector endpoints reachable only in-zone, local cache TTLs). The split is the SAME Git-vs-DB domain classification from Q2 — global=central-authoritative, node-local=edge-authoritative — applied across the hop. **Residency enforcement is structural:** the coordinator's bundle-computation step filters out any config object tagged region-bound to a region other than the satellite's, the same reject-at-plan-time discipline as D-C5-3 — a residency-violating config object is never placed on the wire, not merely refused at the edge. Tag every config object with a residency class so this filter is mechanical, not heuristic.

---

## 6. Config Audit + Safety / Blast-Radius

### 6.1 Cited prior art — the CrowdStrike lesson

The canonical "bad config bricked the fleet" event: "the CrowdStrike Falcon channel file incident of 2024 … CrowdStrike deployed a faulty content update to its Windows sensor fleet, resulting in widespread system failures and outages; post-incident analyses emphasized the need for staged content deployment, smaller blast radii, and robust rollback mechanisms." [web] (Note: the specific channel-file number / Content Validator out-of-bounds-read root cause from CrowdStrike's own Root Cause Analysis is [model-knowledge] — this pass confirmed the *staged-rollout lesson* but did not re-extract the RCA's technical specifics. Re-pull the CrowdStrike RCA PDF at morph time if the BC needs to cite the exact mechanism.)

### 6.2 Cited prior art — staged/canary config + auto-rollback

- **Flagger (Flux progressive delivery)** is the strongest config-canary prior art: "Flagger keeps track of ConfigMaps and Secrets referenced by a … Deployment and triggers a canary analysis if any of those objects change … modifying a detection threshold or changing a connector endpoint … cause Flagger to treat the new configuration as a candidate for rollout, performing analysis before fully applying it." It "upgrades a subset of pods … evaluates metrics to decide whether to proceed or roll back." [web]
- **Argo CD** supplies the Git-revert rollback: the OneUptime rollback script "identifies the previous revision … and then performing a rollback … waits for the application's health using `argocd app wait … --health`." "Rollback is accomplished by changing Git state (via revert) and letting Argo CD reconcile." [web]
- Net: "Safe hot-reload at scale must incorporate staged or canary application … apply updates to a subset of nodes, monitor behavior, and progressively expand rollout upon success." Operators "first use Flagger to introduce configuration changes gradually and then rely on Argo CD's Git-based rollback to revert the manifests if the rollout fails." [web]

### 6.3 Cited prior art — secret references, not values

Uniform across all systems: "best practice is to reference secrets by identifiers or paths rather than embedding them directly in configuration"; "the configuration subsystem should treat secret references as opaque identifiers and avoid resolving or logging secret contents outside the minimum necessary contexts"; Consul/Nomad "integrate with Vault … reference secrets by identifiers or paths"; K8s "separates ConfigMaps from Secrets … best practices discourag[e] logging of secret values." [web] This is a direct external confirmation of Prism's AD-017 reference-based AI-opaque model — the config plane holds references; the §11.1 `SecretBackend` resolves them at the I/O boundary; values never enter config rows, logs, or agent context.

### 6.4 Who-changed-what-when

The DB-authoritative weakness the research flags — "harder to answer 'who changed what when'" without "robust database-level audit logging and snapshotting" [web] — is exactly what §14.3's Postgres control-plane audit table is provisioned to fix, and §11.2 requires ("every config mutation carries analyst identity + timestamp"). Binds to ADR-051 per-connection analyst identity: every config mutation row carries the resolved analyst identity from the transport layer.

### 6.5 Q6 LEAN

**Apply the CrowdStrike lesson as a hard discipline: no config change of the global/high-blast-radius class reaches the whole fleet in one step.** Concretely: (a) **staged/canary rollout for global config** (detection rules, pushdown descriptors, connector definitions) — apply to a canary cohort of satellites/tenants, watch health (the §3.6 partial-result coverage signal + availability cache are ready-made health metrics), then progressively widen; auto-rollback on health regression (Flagger model). (b) **Git-revert rollback for the Git-authoritative domain** (Argo CD model) — the generation stamp from Q3 is the rollback anchor; for the DB-authoritative domain, rollback is "restore prior audited generation from the Postgres audit table." (c) **Every config mutation = one audit row** (analyst identity via ADR-051 + timestamp + before/after generation + Git commit hash where Git-authoritative) in the §14.3 audit table. (d) **Config holds secret REFERENCES only**, never values — externally confirmed best practice and a verbatim match to AD-017; the config validator must *reject* a config object that contains an inline secret-shaped value (fail-closed), reusing the redacted-`Debug` newtype discipline. (e) **A bad central push must be structurally incapable of bricking a satellite:** validate-before-swap at the edge (Q3) means a satellite that receives invalid config keeps its last-good generation and reports DEGRADED — it does NOT crash. That edge fail-closed + canary cohort + auto-rollback is the three-layer blast-radius defense the CrowdStrike post-mortem prescribes.

---

## Consolidated Open Design Questions

| # | Open question | Where it lands | Notes |
|---|---|---|---|
| OQ-C9-1 | Exact boundary between Git-authoritative and DB-authoritative config domains — the per-key classification | morph ADR (config-plane authority model) | Q2 lean gives the split; the precise key inventory needs PO+architect. Must be enumerated + `#[non_exhaustive]` + versioned. |
| OQ-C9-2 | Does the connector-TOML dogfood model become the Git-authoritative source feeding a Postgres projection, or stay file-only for single-analyst and Git-only for central? | morph ADR | Connector definitions are config-as-code TODAY; cleanest if Git-authoritative with store as projection. |
| OQ-C9-3 | Config `schema_version` / migration framework — Prism has none today | morph BC + prism-config extension | Envoy/K8s `apiVersion` model. Real gap vs §11.2 "versioned change control." |
| OQ-C9-4 | Postgres `LISTEN/NOTIFY` vs polling for store→process change propagation; latency/back-pressure under many tenants | morph ADR + prototype | Research endorses push+poll hybrid; needs a bake-off. |
| OQ-C9-5 | Per-tenant `ArcSwap` map memory/contention at high tenant counts | morph perf research | `OrgId → ArcSwap<TenantConfig>` realization of §11.2 per-tenant snapshot. |
| OQ-C9-6 | Canary cohort selection for satellite/tenant config rollout; health-regression auto-rollback metric definition | morph ADR | Reuse §3.6 coverage signal + availability cache as health inputs. |
| OQ-C9-7 | Residency classification taxonomy for config objects (which config is region-bound) | morph BC; ties D-C2-12 / D-C5-3 | Reject-at-bundle-time, uniform with data residency. |
| OQ-C9-8 | `notify-debouncer-full` version pin + atomic-save (rename) handling test | morph implementation | [INCONCLUSIVE] version this pass; re-verify crates.io at morph. |
| OQ-C9-9 | CrowdStrike RCA technical specifics if a BC needs to cite the exact channel-file mechanism | morph (optional) | Staged-rollout lesson confirmed; RCA internals [model-knowledge]. |

---

## Recommended Config-Management Pillar Architecture (concrete)

**Layering.** `defaults < bootstrap TOML < env (scalars only) < runtime store (Postgres central / SQLite satellite) < runtime override (API/UI)`. Bootstrap-class keys are an enumerated `#[non_exhaustive]` allowlist and are NOT store-overridable (restart-class). Minimal Envoy/OpAMP-style bootstrap: identity + how to reach the store + restart-class invariants only.

**Git-vs-store authority (the fork).** **Domain-split hybrid.** Global / high-blast-radius / slow-changing config (detection rules, retention policies, connector definitions, pushdown descriptors, configure-schema mappings, satellite trust/topology, baseline RBAC roles) is **Git-authoritative; the store is a validated materialized projection** with a reconcile loop and drift surfacing (Argo CD / Elastic-detection-as-code model). Per-tenant / runtime-mutable config (connector toggles, threshold overrides, credential references + rotation state, RBAC assignments, operational TTLs) is **store-authoritative; Git/object-store export provides snapshot audit** (Grafana / Kibana-Fleet model), with the Postgres control-plane audit table supplying who-changed-what-when. The classification is per-key, explicit, versioned, and enforced — this is the single most important design artifact.

**Hot-reload.** Validate-before-swap, fail-closed, atomic `ArcSwap` swap of a generation-stamped typed snapshot; in-flight queries hold `load_full()` owned snapshots for their lifetime (`load()` Guards for short hot paths only); per-tenant `ArcSwap` cells so one tenant's reload never blocks another. File path watched via debounced `notify`; store path via Postgres `LISTEN/NOTIFY` + polling fallback. Invalid config → keep last-good generation + structured event, never crash, never partial-apply.

**Distribution.** Pull-from-edge over the existing §3.2 dial-home reverse-RPC conduit (no second channel). Coordinator computes per-satellite effective bundle = global-policy ∩ tenant-scope ∩ residency-filter; satellite materializes to SQLite + ArcSwaps locally with the same validate-before-swap gate. Residency enforced structurally at bundle-computation time (reject-before-wire), uniform with D-C5-3.

**Audit + blast-radius.** Every mutation = one audit row (ADR-051 analyst identity + timestamp + before/after generation + Git commit hash). Global-class changes roll out staged/canary with health-regression auto-rollback (Flagger model) and Git-revert rollback (Argo CD model); generation stamp is the rollback anchor. Config holds secret references only — validator fail-rejects inline secret-shaped values. Three-layer blast defense: edge validate-before-swap (fail-closed to last-good) + canary cohort + auto-rollback.

---

## Honest Costs & Caveats

- **The domain-split hybrid is more complex than either pure pole.** Two authority models, a reconcile loop for one half, a UI/API mutation path for the other, and an explicit per-key classification that must be maintained as config keys are added. The research is candid that "where prior art is incomplete — such as integrating Git workflows with per-tenant DB-authoritative state … platform designers must innovate." [web] Prism is partly in unmapped territory at the Git↔per-tenant-DB seam; expect bespoke work there.
- **Reconcile loop + drift detection is net-new subsystem weight.** Argo CD's loop is a whole product. Prism's Git-authoritative half needs a (much smaller) converge-store-to-Git loop with drift surfacing — real day-2 implementation, not a wiring change.
- **Schema versioning/migration is a genuine gap, not a refinement.** Prism has no config `schema_version` today; retrofitting one plus a forward-migration framework is morph-scale work.
- **Canary config rollout requires a health-regression metric Prism must define.** §3.6 coverage + availability cache are candidate inputs but "config push made detections worse" is a subtler signal than "pod crashed"; defining the auto-rollback trigger is non-trivial.
- **Per-tenant `ArcSwap` map at scale is unproven for Prism's tenant counts** — memory and the rebuild cost on reload need a perf bake-off (OQ-C9-5).
- **Version pins:** arc-swap 1.9.1 [web+c7] and figment 0.10.19 / config 0.15.4 [web] are spot-checked this pass; **notify 7.0.0 patch and notify-debouncer-full version are [INCONCLUSIVE]** — re-verify at morph. Do not pin from this report.
- **Residency for *config* (vs data) is under-documented in the prior art** [web]; the region-pinned-control-plane pattern is the only well-attested approach. Prism's reference-based secret model shrinks the exposure but does not eliminate the region-meaningfulness of a reference.
- **CrowdStrike RCA internals are [model-knowledge]** this pass — the staged-rollout *lesson* is web-confirmed; the channel-file-291 / Content-Validator-out-of-bounds *mechanism* was not re-extracted. Re-pull the official RCA if a BC must cite the mechanism.
- **Leans are discussion input only.** The central fork (Q2) and the distribution/precedence model (Q5) are PO+architect adjudication at morph, not decided here.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | `reasoning_effort=high`, `strip_thinking=true`. Six-area depth pass: config-pillar shape (Vector/OTel/OpAMP/Envoy/Grafana/K8s/Consul), GitOps-vs-DB authority fork (Argo CD/Flux/Flagger/Panther/Elastic detection-as-code/Terraform-Atlantis), safe hot-reload (Envoy xDS validate-before-swap, Consul-template, NGINX/HAProxy), figment/config-rs anti-patterns, multi-tenant/edge fleet distribution (Fleet/osquery/Consul/Nomad/OpAMP/SaltStack/Ansible), audit+blast-radius (Flagger canary, Argo rollback, CrowdStrike 2024, secret references). Returned ~126k chars; read via targeted Grep extraction (single-line JSON exceeded Read line cap). |
| Perplexity perplexity_ask | 1 | Latest crates.io versions (arc-swap, notify, notify-debouncer-full, figment, config) — ≤2-sentence factual lookup. |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_reason | 0 | — |
| Context7 resolve-library-id | 2 | arc-swap, notify |
| Context7 query-docs | 2 | arc-swap 1.9.1 (load/load_full/store/swap consistency contract for in-flight snapshots); notify (debouncer, atomic-rename/temp-file event collapsing) |
| Tavily (all variants) | 0 | — (Perplexity + Context7 sufficient; not needed for cross-validation this pass) |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | 3 areas (flagged) | (1) config-rs HashMap-nondeterminism + case-sensitivity anti-patterns + figment SLOC counts — carried [model-knowledge] from 2026-04-14 brownfield ingest, not re-confirmed by name; (2) CrowdStrike RCA technical mechanism (channel file 291 / Content Validator) — staged-rollout lesson web-confirmed, internals [model-knowledge]; (3) notify-debouncer-full version [INCONCLUSIVE]. |

**Total MCP tool calls:** 6 (1 perplexity_research-high [PRIMARY], 1 perplexity_ask, 2 Context7 resolve, 2 Context7 query-docs).
**Training data reliance:** low — every load-bearing claim is [web] or [c7] cited; the three [model-knowledge]/[INCONCLUSIVE] items are explicitly flagged and routed to morph-time re-verification (OQ-C9-3, OQ-C9-8, OQ-C9-9).

**Deviation note (primary-tool mandate):** the non-trivial multi-area topic was led by a single `perplexity_research` call at `reasoning_effort=high` (the mandated default). No high-effort retry/fallback was needed — the call succeeded on the first attempt (it returned an oversized result that was read via Grep, not an overload failure). Context7 supplied the version-verified Rust crate state (arc-swap consistency contract, notify debouncer) per the dispatch's explicit instruction.
