---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
topic: "Dual deployment axis — SaaS (vendor-operated, multi-customer) AND on-prem/self-managed (customer-operated, air-gap-capable) from one codebase"
program: "Prism day-2 vision SIDE-ANALYSIS (OUT-OF-BAND; SEPARATE from live VSDD factory pipeline)"
classification: "FOUNDATIONAL cross-cutting deployment-axis analysis"
settled_context_reconciled:
  - "C1: centralized control plane (MCP Streamable HTTP + OAuth2.1 RS + built-in AS + external IdP)"
  - "Storage taxonomy: Postgres central / SQLite satellite / Iceberg cold / RocksDB hot"
  - "C2: satellite mesh (outbound-only dial-home, per-hop mTLS, residency-by-construction, satellite-local credential resolution)"
  - "C9: config (DB-authoritative + DB-native-temporal versioning; embedded-git detection content; opt-in residency-gated remote; A/B + supervisor watchdog + autonomous satellite self-recovery + safe-mode; canary + fast-revert; approval-workflows deferred to day-3)"
  - "AD-017: AI-opaque, reference-based, satellite-local credential resolution"
  - "Air-gap is a hard requirement for on-prem"
caveat: "Leans are DISCUSSION INPUT ONLY. Not a decision record. Not an ADR. Do not promote to a live spec without architect adjudication."
---

# Dual Deployment: SaaS + On-Prem/Self-Managed from One Codebase

**Prism day-2 SIDE-ANALYSIS — capture artifact, do not execute.**

This pass grounds the new human-directed constraint (2026-06-27): Prism day-2 must support BOTH a vendor-operated, multi-customer **SaaS** edition AND a customer-operated, air-gap-capable **on-prem / self-managed** edition. It reconciles that deployment axis against the already-captured day-2 decisions (C1/C2/C9/storage/credentials/release), with focus on config-management (C9) impact and the cross-cutting reconciliations. Every claim is cited or flagged `[model-knowledge]` / `[INCONCLUSIVE]`.

---

## Executive thesis (read first)

Three findings dominate this pass:

1. **Single-codebase + a deployment-profile abstraction is the dominant, recommended pattern.** Every dual-mode product studied (GitLab, Sentry, Elastic, Mattermost, GitHub, Grafana, HashiCorp Terraform, Temporal) converged on ONE repository producing multiple deployment profiles via license/capability gating, feature flags, environment detection, and pluggable drivers. Divergent forks (separate SaaS vs on-prem codebases) are the documented anti-pattern: GitLab explicitly archived its separate Enterprise Edition repository to consolidate into a single codebase.[L1-GitLab]

2. **Prism's satellite mesh IS the BYOC (Bring-Your-Own-Cloud) data-plane pattern, already, by construction.** The single sharpest insight from this pass: the industry's premium data-sovereignty SaaS pattern — vendor control plane holds only metadata/orchestration; customer-resident data plane holds all raw data and credentials; vendor never sees payload or secrets — is *exactly* Prism's C2 residency-by-construction + AD-017 satellite-local credential resolution. The litmus "test for genuine BYOC" published by industry — *"can the vendor see the payload of a request to your application? In real BYOC the answer is no"*[S5-Northflank-test] — Prism passes by construction. This is plausibly Prism's strongest SaaS differentiator. See Topic 3.

3. **The on-prem air-gap requirement makes config-schema-versioning + data-migration + offline-content-delivery load-bearing, and these retroactively touch C9.** SaaS controls its own upgrade timing (roll-forward); on-prem customers control timing, may skip versions, and have zero outbound connectivity. That asymmetry is the single largest concrete delta and is the reason C9's "remote-git = managed-default SaaS vs opt-in/offline-bundle on-prem" split is mandatory rather than cosmetic. See Topics 2.4, 2.5, 4.

---

## Topic 1 — The deployment-profile abstraction (single codebase, SaaS-mode vs self-managed-mode)

### What the prior art does

| Product | Mechanism for SaaS-vs-self-managed from one codebase | Source |
|---|---|---|
| **GitLab** | Single codebase (EE repo archived). `ee/` top-level directory for Enterprise code; CE in main tree. Runtime "modes": CE, EE-unlicensed, EE-licensed, GitLab.com (SaaS), Dedicated. Predicates `Gitlab::Saas.feature_available?` / `Gitlab::Dedicated.feature_available?` gate profile-specific paths, backed by YAML feature-definition files under `ee/config/saas_features`. Strict rule: SaaS predicates must NOT appear in CE code paths; all SaaS-affected code must have tests in BOTH enabled+disabled states. | [L1-GitLab] |
| **Sentry** | One core codebase; self-hosted = "all of Sentry" minus a documented exception set (billing, spike-protection/spend-allocation tied to SaaS quotas, closed-source AI/Seer, console-vendor partner integrations). Profile boundary = "what can be open-sourced/customer-installed" vs "what must stay vendor-operated." | [L1-Sentry] |
| **Elastic** | Same Stack binaries for Cloud and self-managed; advanced features (security, ML, cross-cluster replication) gated by **license tier** (basic/gold/platinum/enterprise) — runtime license checks activate code paths already compiled in. | [L1-Elastic] |
| **Mattermost** | Open-source self-hosted core; Cloud wraps the SAME server in managed k8s + single-tenant VPC/data-residency for high-trust customers. Difference is operational layer, not core logic. | [L1-Mattermost] |
| **GitHub** | dotcom/Enterprise-Cloud (SaaS, multi-tenant) vs Enterprise Server (self-hosted appliance). Features ship to dotcom first, then backport to Enterprise Server releases — a cadence only sustainable with a shared codebase. (Architecture details inferred; GitHub does not publish internal gating mechanics — flagged partially `[model-knowledge]`.) | [L1-GitHub] |
| **Grafana** | OSS core + Enterprise features delivered as **dynamically-loaded plugins** gated by license validation. Same binary runs OSS (no enterprise plugins loaded) or Enterprise (license valid → plugins load). Grafana Cloud runs Enterprise under the hood. | [L1-Grafana] |
| **HashiCorp Terraform** | HCP Terraform (SaaS) and Terraform Enterprise are explicitly "different distributions of the same application." Enterprise = private instance, no resource limits, plus audit-logging + SAML. Core (state, remote exec, providers) identical; difference is packaging + license-gated enterprise features. | [L1-Terraform] |
| **Temporal** | Control-plane/data-plane split: Temporal Cloud = managed control plane (workflow state, scheduling); workers run in the CUSTOMER environment and Temporal never sees app logic or sensitive data (optional client-side payload encryption → cloud sees only ciphertext). Same core semantics across OSS + Cloud. | [L1-Temporal] |

### The maintenance-cost lesson (why divergent builds fail)

GitLab's documented decision to **archive the separate Enterprise Edition repository** and consolidate into one codebase is the load-bearing evidence: separate forks drift in behavior, complicate bug-fixes and feature ports, and create developer friction across trees.[L1-GitLab] Consolidation enables unified branching, one CI pipeline, and cross-edition tests. No surveyed vendor maintains genuinely separate SaaS-vs-self-managed source trees today.

### Shared-vs-profile-specific ratio

No vendor publishes an exact shared:specific code ratio `[INCONCLUSIVE]`. But the structural evidence is consistent: the **overwhelming majority** of business logic is shared, with profile-specific code confined to (a) licensing/entitlement, (b) tenancy/multi-customer management, (c) control-plane operational features (billing, quota, vendor telemetry), and (d) cloud-service bindings. GitLab's `ee/`-directory model and Grafana's enterprise-plugin model both keep the profile boundary narrow and explicit.[L1-GitLab][L1-Grafana]

### LEAN (discussion input only)

**Confirm single-codebase + deployment-profile abstraction.** Prism should adopt an explicit `DeploymentProfile { SaaS, SelfManaged }` (run-time-selected, config-driven) as a first-class concept, with the GitLab discipline borrowed wholesale: profile-specific code paths must be (1) explicitly gated behind a profile predicate, (2) never leak SaaS-only assumptions into shared/self-managed paths, and (3) tested in BOTH profiles. A Grafana-style plugin/capability gate is the natural Rust analogue (feature-flag + capability-descriptor gating, which Prism already has machinery for per the capability-descriptor research). Target ~90% shared, profile-specific confined to control-plane hosting, tenancy depth, identity wiring, storage binding, update mechanism, remote-dependency toggles.

---

## Topic 2 — What genuinely differs across the two modes (per axis)

### 2.1 Tenancy

**The canonical isolation taxonomy (convergent across AWS, Snowflake, Atlassian):**

| Model | AWS name | Snowflake name | Isolation | Cost |
|---|---|---|---|---|
| Shared table + tenant-id column + row-level security | Pool | Multi-Tenant Table (MTT) | Weakest (logical) | Lowest |
| Shared instance, schema-per-tenant | Bridge | Object-Per-Tenant (OPT) | Medium | Medium |
| Database/instance-per-tenant | Silo | (DB-level OPT) | Strong | High |
| Account/cell/pod-per-customer | (cell) | Account-Per-Tenant (APT) | Strongest (physical) | Highest |

Sources: [L2-AWS] (Silo/Bridge/Pool), [L2-Snowflake] (MTT/OPT/APT + hybrids like MTT/OPT = shared storage + dedicated compute), [L2-Atlassian] (warns DB-per-tenant scales poorly; row-level-security preferred for manageability).

**Cell/pod-per-customer in practice:** GitLab.com uses **Cells** — each cell hosts a subset of organizations; within a cell a single DB stores all customer data and every customer-data table MUST define a **sharding key** linking each row to an organization. GitLab **Dedicated** reuses the SAME Cells design as a single-tenant cell.[L2-GitLab-Cells] Temporal Cloud uses cells = self-contained units (own AWS account/VPC/EKS/DB) as failure domains.[L1-Temporal][L2-Temporal] AWS Lambda even offers execution-environment tenant-isolation keyed by a tenant-ID per invocation.[L2-Lambda]

**The SaaS-multi-customer ↔ on-prem-single-customer reconciliation (the key MSSP point):** The decisive finding is that **the same tenant abstraction serves both.** GitLab.com (multi-customer) and GitLab Dedicated (single-customer) both key on an organization-ID + sharding key; the difference is *tenancy cardinality and infrastructure topology, NOT code.*[L2-GitLab-Cells] For an MSSP running Prism on-prem serving multiple of ITS OWN clients within one install: the on-prem install is single-CUSTOMER (one legal tenant = the MSSP) but internally multi-tenant (the MSSP's clients map to internal tenant-IDs). Snowflake's guidance is explicit that the "tenant" can be an internal business unit and the identical OPT/RBAC code applies.[L2-Snowflake]

**LEAN (discussion input only):** Adopt one pervasive **tenant identifier** (call it `org_id` / `tenant_id`) as a first-class domain concept threaded through every customer-data table (as a sharding key), every authorization/RBAC policy, every config scope, and every tenant-scoped API entrypoint — decoupled from physical isolation. Then:
- **SaaS profile:** `tenant_id` = each paying customer; isolation = Pool (row-level + tenant column) within a cell, with cell/pod partitioning for blast-radius. Reconciles with C1 centralized control plane.
- **Self-managed profile:** `tenant_id` = the MSSP's internal client; one install = one cell; internal multi-tenancy via the SAME row-level mechanism.
- Critically separate the notions of *deployment environment* from *tenant boundary* so billing/audit/blast-radius logic works in both. The Postgres-central storage (settled) already supports row-level isolation; this is the lowest-divergence path.

### 2.2 Storage substrate

**SaaS:** managed cloud Postgres (RDS/Aurora — daily automated backups, multi-AZ, six-copy storage) + cloud object store (S3) for Iceberg.[L2-AWS][L2-Iceberg] **On-prem:** bundled/self-managed Postgres + S3-compatible store (MinIO) or local FS.

**Concrete design constraints that supporting BOTH imposes:**

1. **Do NOT assume cloud-only object-store features.** Apache Iceberg's `S3FileIO` leverages SSE-S3/SSE-KMS/SSE-C, ACLs, S3 strong-consistency, object tagging, and an `ObjectStoreLocationProvider` for prefix distribution — but exposes ALL of these as configurable, and supports swapping the FileIO impl (e.g. S3A via Hadoop) for non-AWS stores.[L2-Iceberg] MinIO is S3-API-compatible for the core set (Put/Get/Copy/multipart, bucket mgmt, versioning, conditional headers If-Match/If-None-Match/If-Modified-Since) but **lacks some S3 features** (certain ACL ops, BucketWebsite) and adds non-standard extensions (Fan-Out, AppendObject in S3 Express).[L2-MinIO] **Design rule:** rely only on the widely-supported S3 subset; treat ACLs/replication/S3-Express as optional, config-gated enhancements; never hard-code AWS IAM auth or endpoint format — make endpoint/credentials/region external parameters so a generic S3-compatible endpoint works.[L2-Iceberg][L2-MinIO]
2. **Postgres portability.** Must run on generic Postgres (bundled or external), not assume RDS/Aurora-specific parameter-group behaviors or cloud monitoring endpoints. GitLab's discipline of pinning a minimum Postgres major version (PG17 floor in GitLab 19.0) applies to BOTH cloud and self-managed.[L2-GitLab-PG] Core SQL stays portable; managed-service conveniences are SaaS-profile-only operational wiring. (Prism uses `sqlx`-style portable SQL `[model-knowledge]` — verify no RDS-specific extensions creep into migrations.)
3. **Iceberg cold-tier:** the settled Iceberg choice is sound for BOTH because Iceberg is storage-agnostic via FileIO; the constraint is just "config the object-store binding per profile, don't bake S3 assumptions into the table layer."

**LEAN:** Storage stays *same engines, pluggable bindings*. Postgres + Iceberg + RocksDB + SQLite all already span both modes; the only new work is a **storage-binding profile** (object-store endpoint/auth/region as config, not code) and a CI guard that the shared core never calls an AWS-only API.

### 2.3 Identity

**SaaS:** vendor IdP + per-customer SSO federation (SAML/OIDC) + SCIM provisioning. **On-prem:** customer's own IdP + the built-in authorization server (Prism C1 already has built-in AS).

**How dual-mode products fork identity from one codebase — pluggable auth backends:**
- **GitLab OmniAuth:** configurable provider array (`["saml","google_oauth2"]`), auto-link to LDAP, auto-create accounts, per-provider enable/disable, can disable OmniAuth entirely. Internal user DB = built-in AS; external IdPs = federated sources. SaaS: vendor configures providers + internal accounts primary. Self-managed: customer LDAP/IdP primary, internal DB secondary.[L2-GitLab-OmniAuth]
- **Grafana:** Generic OAuth + LDAP both configurable via UI/config; JWT validation via JWK-set URL; role-mapping + team-sync from OAuth claims. No single backend mandatory.[L2-Grafana-OAuth][L2-Grafana-LDAP]

**The built-in-AS-vs-external-IdP switch is config, not code.** The platform's internal user/group/role model = built-in authorization server (resource server). External IdPs handle authn (+ SCIM provisioning). SaaS: vendor IdP default, per-customer SSO. Self-managed: customer IdP primary, built-in AS for emergency/service accounts. Same connector interfaces (OIDC/SAML/LDAP/JWT) compiled into both; config selects which are active. SCIM-for-provisioning specifics flagged `[INCONCLUSIVE]` — well-established industry pattern but not in cited primary docs.[L2-identity]

**Reconciles with C1:** Prism's settled C1 (OAuth2.1 RS + built-in AS + external IdP) is ALREADY the dual-mode identity shape. SaaS profile = built-in AS issues for vendor tenants + per-customer external-IdP federation; self-managed = built-in AS + the customer's single external IdP. No architectural change — just a profile-conditional default for *which* IdP is primary and whether multi-customer SSO federation is enabled.

### 2.4 Update / release model

**SaaS:** vendor continuous delivery, vendor controls timing, roll-forward only, customer has no version choice. **On-prem:** CUSTOMER-controlled versioned upgrades — customer chooses when, **may skip versions** → requires forward schema migration + backward-compatibility windows + version-skew tolerance (commonly N-2).[L2-GitHub-upgrade][L2-GitLab-breaking]

**Why this makes schema-versioning load-bearing for on-prem but less for SaaS:** SaaS runs exactly one version at a time under vendor control, so a migration can be coupled to its deploy. On-prem customers jump between feature releases (GitLab's documented sequential-upgrade-path requirement; GitHub Enterprise Server's documented per-version migration steps, e.g. the 2.17 Elasticsearch→MySQL audit-log migration that increases restore time/disk).[L2-GitHub-upgrade][L2-GitLab-breaking] So on-prem MUST have: idempotent forward migrations, defined upgrade paths (possibly multi-hop through intermediate versions), backward-compat windows where new code reads old schema, and pre-flight capacity/snapshot checks.

**Reconciles with C9 Q3 (config-schema versioning):** This is the hard tie-in. Prism's settled C9 uses **DB-native-temporal versioning for runtime config** and **embedded-git for detection content**. The on-prem skip-version reality means:
- Config-schema migrations must be forward-only, idempotent, and version-path-aware (support N-2 skew minimum) — the DB-native-temporal model helps because it already versions config rows, but the *schema* of those rows still needs migration discipline.
- Detection-content (embedded-git) needs a content-format-version compatible across the skew window so an on-prem customer importing a newer content bundle onto an older binary (or vice versa) degrades gracefully.

**LEAN:** Make migration discipline a profile-conditional rigor, not a binary on/off. Both profiles run the same migration engine; SaaS exercises it continuously (low skew), on-prem exercises it across customer-chosen jumps (high skew → the rigorous path). Adopt explicit upgrade-path + version-skew contracts as a day-2 deliverable. This is the largest concrete delta and deserves its own follow-up design.

### 2.5 Remote-dependency posture

**SaaS:** may call cloud services (telemetry, license check, hosted content). **On-prem:** MUST be air-gap-capable — no external calls, no GitHub dependency, no cloud telemetry, offline license.

**How dual-mode products keep self-managed dependency-free behind the profile boundary:**
- **Sentry air-gap:** build images on a connected machine, `docker save`→tar→transfer→`docker load`; proxy support; no outbound calls at runtime.[L1-Sentry]
- **Elastic air-gapped endpoints:** default = auto-fetch artifacts from `artifacts.security.elastic.co`; air-gap = configurable `advanced.artifacts.global.base_url` per-OS pointing at a local mirror/file-server; manual artifact copy; verify via `manifest_version`; "update at least monthly."[L4-Elastic-airgap]
- **GitLab offline scanners:** disable GitLab.com checks; `Secure-Binaries.gitlab-ci.yml` template downloads analyzer images on a connected env → transfer → load into offline registry → point `SECURE_ANALYZERS_PREFIX` at the local registry.[L4-GitLab-offline]
- **Palo Alto air-gapped firewall:** offline license-key files; manual PAN-OS + dynamic-content upload via web UI; verify it CANNOT reach external hosts (air-gap integrity check).[L4-PaloAlto]

The unifying mechanism: **a config boundary (a URL/registry/endpoint variable) that points at vendor infra in SaaS and at a local mirror/offline-bundle in on-prem.** The shared core only knows "fetch content from `<configured source>`"; the profile decides what that source is.[L4-GitLab-offline][L4-Elastic-airgap]

**Reconciles with C9 (remote-git):** This is exactly the settled C9 "opt-in residency-gated remote." Formalize: detection-content remote-git = **managed-default (auto-pull) in SaaS** vs **opt-in / offline-signed-bundle in on-prem**, both behind ONE content-source config variable. Air-gap is the hard constraint that forces the shared core to never hard-code an outbound call.

---

## Topic 3 — The SaaS data-sovereignty / credential-trust model (the sharpest reconciliation)

### The problem

A SaaS security platform ingesting a customer's security data + holding credentials to the customer's sensors is a major trust/residency problem. Security-sensitive customers (finance, healthcare, public sector, defense) demand that raw data and credentials *never sit in the vendor cloud.*[L2-BYOC][S6-zerotrust]

### The industry answer: BYOC / split control-plane / customer-resident data plane

The prior art is now mature and remarkably uniform:

| Vendor | Pattern | What's in vendor control plane | What stays in customer environment | Source |
|---|---|---|---|---|
| **Databricks** | Control plane (Databricks account) + classic compute plane (CUSTOMER VPC) | orchestration, notebooks, job metadata, SQL metadata | raw data, EC2 workers, S3, IAM creds; secure-cluster-connectivity HTTPS tunnel, no public IPs; optional PrivateLink; short-lived (~1hr) tokens for serverless | [L3-Databricks] |
| **Snowflake** | Provider-hosted data, but Tri-Secret Secure dual-key | data + metadata (provider-hosted) | customer-managed key (CMK) in customer KMS/external HSM — revoke CMK ⇒ Snowflake CANNOT decrypt, queries halt | [L3-Snowflake] |
| **Confluent** | BYOC + PrivateLink; control/data-plane Kafka cluster split | metrics, authn, authz, audit metadata | business-event data clusters (warns: typical BYOC granting vendor VPC access defeats sovereignty) | [L3-Confluent] |
| **WarpStream** | **Zero-access BYOC** (sharpest exemplar) | ONLY metadata (offsets, partition assignments, cluster status) | ALL data in customer object store; stateless Agents in customer VPC; **zero cross-account IAM**; vendor cannot read data or act on customer's behalf | [L3-WarpStream] |
| **Redpanda** | Fully-managed BYOC | cluster ops, policy, RBAC org-level | data plane in customer VPC; "sensitive data and credentials never leave the customer's environment" | [S1-Redpanda][S8-Redpanda] |
| **Cribl** | Vendor control plane + customer-hosted Edge/Stream workers | config, version orchestration (pre-signed S3 URLs for upgrade tarballs) | raw telemetry processed locally on worker nodes; creds on workers | [L3-Cribl] |

**Key-custody taxonomy (CryptoMathic, cited):** BYOK (customer generates, imports to provider KMS — provider's infra sees key in secure components) < CYOK (keys never in clear to provider; customer HSM/enclave) < HYOK (keys entirely outside provider; data encrypted before entering cloud; provider only ever sees ciphertext).[L3-keymgmt]

**The published litmus test for genuine BYOC:** *"Can the vendor see the payload of a request to your application? In real BYOC, the answer is no... If the vendor's load balancer terminates traffic before forwarding to your VPC, that is NOT BYOC — that is a SaaS product with a private connection."*[S5-Northflank-test] The zero-trust corollary: *"how much should a vendor control in your cloud? The answer is zero if possible"* + *"ephemeral tokens to access your storage... if you have a static token to access S3 and somebody takes it, tomorrow you're done."*[S6-zerotrust]

### RECONCILE WITH PRISM — the central insight

**Prism's satellite mesh IS the BYOC data-plane pattern, by construction.** Map Prism's settled C2/C5/AD-017 onto the industry pattern:

| BYOC concept | Prism's existing design (settled) |
|---|---|
| Customer-resident data plane | The **satellites** — they live in/near the customer environment |
| Raw data never leaves customer env | C2/C5 **residency-by-construction**: raw normalized at the edge, only RESULTS transit |
| Vendor never holds credentials | **AD-017**: AI-opaque, reference-based, **satellite-local credential resolution** — creds resolve at the satellite, never at central |
| Outbound-only data-plane→control-plane connection | C2 **outbound-only dial-home** (matches the documented best practice: "only the data plane initiates connections")[L3-byoc-bestpractice] |
| Per-hop encryption | C2 **per-hop mTLS** |
| Thin control plane = metadata/orchestration only | C1 central = coordinator |

This means the Prism SaaS edition's central control plane is a **thin coordinator that never sees raw data or credentials** — which is precisely the WarpStream "zero-access BYOC" gold standard, and precisely what passes the Northflank litmus test. Prism does not have to *retrofit* BYOC; the architecture is BYOC-native.

**This may be Prism's strongest SaaS differentiator** (evidence supports emphasizing it): most SaaS security tools have to bolt on BYOC awkwardly (Confluent's own docs warn that typical BYOC still grants vendor VPC access, defeating sovereignty[L3-Confluent]). Prism's satellite-local-credential + residency-by-construction design means the SaaS story is *"we are structurally incapable of seeing your raw data or your sensor credentials"* — a claim WarpStream and Redpanda market as their headline.[L3-WarpStream][S8-Redpanda]

### Gaps that remain (honest)

1. **Control-plane metadata leakage.** Even in zero-access BYOC, the control plane sees metadata (org structure, table/sensor names, query shapes, health). Prism must define exactly what metadata transits to SaaS-central and confirm it cannot reconstruct sensitive data — WarpStream explicitly asserts its metadata "cannot be used to access or decrypt the data."[L3-WarpStream] Prism needs the equivalent assertion + audit.
2. **Result-transit residency.** C2 says "only results transit." Results of a security query CAN contain sensitive data (e.g. a query returning raw alert fields). Need a residency policy on *what results* may transit to SaaS-central, or keep results customer-resident too with central seeing only result-references/aggregates `[INCONCLUSIVE — needs design]`.
3. **Ephemeral-token discipline.** The industry standard is short-lived tokens (Databricks ~1hr) + revocable cross-account links over static credentials.[L3-Databricks][S6-zerotrust][S5-Northflank-cross-account] Prism's dial-home auth should be ephemeral/rotating, not static, to match the bar.
4. **CMEK/BYOK for any central-held metadata.** If SaaS-central persists ANY customer metadata in Postgres-central, consider customer-managed-key wrapping (Snowflake Tri-Secret model) so even metadata-at-rest is revocable.[L3-Snowflake]

---

## Topic 4 — Config-management (C9) deployment-aware specifics

### (a) Remote-git for detection content

**SaaS:** managed-default — central auto-pulls/curates content, pushes to satellites (analogous to Elastic auto-fetching from its artifact server).[L4-Elastic-airgap] **On-prem:** opt-in / OFF by default; air-gap requires **offline signed content bundles** + manual import (GitLab `Secure-Binaries` template pattern: download on connected env → tar → transfer → load into local registry → repoint a config variable).[L4-GitLab-offline] Both behind ONE content-source config variable (the `base_url` / `SECURE_ANALYZERS_PREFIX` analogue).

### (b) Bootstrap-recovery mechanism

| Profile | Central recovery mechanism | Prior art |
|---|---|---|
| **On-prem central** | Appliance A/B slots + supervisor watchdog auto-revert (settled C9) | Talos A-B image scheme (retains prior kernel+image, rollback on failure); Mender A/B partitions (persistent data on dedicated partition, NOT rootFS; confirm-before-commit); GitHub Enterprise Server VM-snapshot-before-upgrade; Palo Alto HA-pair blue-green at appliance level; Cisco IOS XE `install rollback`[L4-Talos][L4-Mender][L4-GitHub-appliance][L4-PaloAlto-HA][L4-Cisco] |
| **SaaS central** | k8s blue-green / canary / readiness-liveness probes + automated metric-driven rollback | Argo Rollouts / Flagger (canary steps, metric thresholds from Prometheus/Datadog, auto-promote-or-revert); k8s rolling/blue-green/canary strategies[L4-ArgoFlagger][L4-k8s-strategies] |
| **Satellite (BOTH modes)** | Autonomous self-recovery + safe-mode (settled C9) — IDENTICAL in both | declarative desired-state agent: poll desired version, verify signature, local health-check, confirm-or-revert-to-previous; mechanism-agnostic[L4-agent-selfrecovery] |

**The unifying abstraction:** an environment-agnostic control plane decides *what version/config* should apply; the **deployment profile** maps that to the concrete mechanism (k8s rollout CRD in SaaS; A/B-slot+watchdog on the on-prem appliance; offline package import in air-gap). Talos exemplifies this: the same `talosctl upgrade` API call works cloud or on-prem because A-B boot is an OS-implementation detail beneath the orchestration.[L4-Talos] **Satellite self-recovery semantics stay identical** — the agent only interprets profile parameters (how to roll back, what = success, how many prior versions retained), never the underlying mechanism.[L4-agent-selfrecovery] (Explicit single-product unified-profile guidance is sparse in primary sources → this synthesis flagged partially `[INCONCLUSIVE]`.)

### (c) Config-store tenancy

SaaS = multi-customer config store (tenant-keyed, per §2.1 row-level). On-prem = single-customer store (internally multi-tenant for MSSP). Same DB-authoritative C9 model + same tenant-id keying; differs only in cardinality.

### (d) Config canary

**Both modes:** customer-scoped canary + fast-revert (settled C9). **SaaS-ADDITIONAL layer:** the VENDOR canaries platform/config changes ACROSS the customer fleet — ring/cohort deployment, percentage rollout, per-tenant feature flags (Northflank: deploy to small tenant cohort → validate error/latency → progressive expansion with auto-rollback triggers; Microsoft ring-based progressive exposure; LaunchDarkly/Statsig-style tenant-targeted flags).[L4-Northflank][L4-MS-rings] This fleet-canary layer **does not exist on-prem** — the on-prem customer canaries only within their own install (intra-install staging/HA, e.g. Palo Alto HA-pair, GitHub staging-then-prod).[L4-PaloAlto-HA][L4-GitHub-appliance]

### (e) Who operates/owns config

SaaS: vendor SRE owns infra-config + fleet rollout; customer owns their tenant-scoped content/policy. On-prem: customer admin owns everything (GitLab self-managed: customer has root, controls security/downtime/upgrades).[L1-Mattermost][L2-GitHub-upgrade]

**LEAN:** C9 goes deployment-conditional behind the same profile boundary: content-source variable (managed-remote vs offline-bundle), recovery-mechanism mapping (k8s vs A/B-appliance, satellite self-recovery identical), config-store cardinality (multi vs single customer), and a SaaS-only fleet-canary layer LAYERED ON TOP of the shared customer-scoped canary. No C9 primitive is removed; the profile selects bindings + adds the SaaS fleet layer.

---

## Topic 5 — Edition/capability model: shared vs profile-specific (per-subsystem)

| Prism subsystem | Shared (identical both modes) | Profile-specific |
|---|---|---|
| PrismQL parser/grammar | ✅ identical | — |
| Query engine (DataFusion federated) | ✅ identical | — |
| Detection engine | ✅ identical logic | content **delivery** differs (managed vs offline bundle) |
| ML / behavior analytics | ✅ identical | (Sentry-style: any closed-source ML stays SaaS-only IF chosen — `[design choice]`) |
| Satellite mesh (C2) | ✅ identical (dial-home, mTLS, residency, self-recovery) | — — this is the BYOC data plane in BOTH |
| Connectors / sensor adapters | ✅ identical | — |
| Credential resolution (AD-017) | ✅ identical (satellite-local) | — |
| Storage engines (PG/Iceberg/RocksDB/SQLite) | ✅ same engines | **binding** differs (managed cloud vs bundled/MinIO/local) |
| Control-plane HOSTING | core logic shared | **k8s/multi-tenant SaaS infra** vs **appliance/single-tenant on-prem** |
| Tenancy isolation DEPTH | tenant-id abstraction shared | **multi-customer cells** (SaaS) vs **single-customer + internal tenants** (on-prem) |
| Identity wiring (C1) | connector interfaces shared | **vendor-IdP + per-customer SSO/SCIM** vs **customer-IdP-primary + built-in AS** |
| Update mechanism | migration engine shared | **continuous CD / k8s rollout** vs **customer-versioned A/B + offline** |
| Remote-dependency toggles | "fetch from configured source" shared | **cloud services on** vs **air-gap, all-local** |
| Recovery mechanism | satellite self-recovery + safe-mode shared | **k8s blue-green** vs **A/B-slot + watchdog** |
| Fleet-canary layer | — | **SaaS-only** (vendor canaries across customers) |
| Billing / quota / vendor telemetry | — | **SaaS-only** (Sentry precedent: omit from self-managed) |

**Estimate: ~90% shared.** The profile-specific surface is exactly the industry-consistent set: control-plane hosting, tenancy depth, identity wiring, storage binding, update mechanism, remote-deps, recovery, plus SaaS-only billing/fleet-canary. Prism's biggest win: the entire data plane (satellite mesh + creds + residency + query/detection/ML) is shared AND is the BYOC data plane in both modes.

---

## Topic 6 — Honest risks of dual-deployment from one codebase (security product)

1. **Feature-skew trap.** SaaS-only features silently break / are absent in self-managed (Sentry documents this as deliberate; the trap is UNDOCUMENTED skew). **Mitigation (GitLab discipline):** every profile-gated path tested in BOTH enabled+disabled states; explicit profile predicates; SaaS predicates forbidden in shared paths.[L1-GitLab][L1-Sentry]
2. **"Works in SaaS, breaks air-gapped" trap.** Cloud assumptions (an outbound call, an AWS-only S3 API, a hosted-content dependency, a license phone-home) leak into the shared core and break air-gap. This is the MOST dangerous for Prism because air-gap is a HARD requirement. **Mitigation:** a CI gate that builds+tests the self-managed/air-gap profile with network egress blocked, catching any leaked outbound dependency. Treat "no outbound call from shared core" as an invariant. (Northflank/Elastic/GitLab all structure this behind a single content-source variable specifically to prevent leakage.)[L4-Elastic-airgap][L4-GitLab-offline]
3. **Test-matrix explosion.** Must CI BOTH profiles (and air-gap as a third axis). **Mitigation:** profile as an explicit test dimension; GitLab's runtime-mode model (CE/EE/SaaS/Dedicated) is the precedent for making this tractable.[L1-GitLab] Cost is real and is a genuine ongoing tax `[honest cost]`.
4. **Security-posture divergence.** SaaS multi-customer isolation bugs (missing tenant filter → cross-customer data exposure, the Northflank-cited shared-schema risk[L2-Northflank-isolation]) are a DIFFERENT failure class than on-prem single-customer. **Mitigation:** the pervasive tenant-id + row-level-security discipline + an isolation test suite; on-prem's single-customer simplicity does NOT excuse SaaS isolation rigor since the same code runs both.
5. **SaaS cloud-dependency leak breaking air-gap** (the sharpest version of #2 for a security tool): hosted threat-intel auto-update, cloud telemetry, online license. **Mitigation:** offline-bundle + offline-license + no-telemetry as first-class self-managed-profile behaviors, behind the content-source/license/telemetry config boundaries. Elastic ("update at least monthly" via local mirror), GitLab offline scanners, Palo Alto offline license + air-gap-integrity verification are the proven patterns.[L4-Elastic-airgap][L4-GitLab-offline][L4-PaloAlto]

---

## VERDICT: deployment-profile model recommendation (CONFIRMED)

**Single codebase + an explicit run-time `DeploymentProfile { SaaS, SelfManaged }` abstraction is confirmed** as the production-grade pattern, matching every dual-mode prior-art product and avoiding the documented divergent-fork failure (GitLab's archived EE repo).[L1-GitLab] Refinements:
- Profile is **run-time, config-selected** (not build-time forks) — keeps ONE artifact, maximizes shared surface.
- Borrow GitLab's gating discipline: explicit profile predicates, SaaS-only code never in shared paths, both-state testing.
- Borrow Grafana's capability/plugin gating as the Rust-friendly mechanism (Prism already has capability-descriptor machinery).
- Air-gap is the invariant that disciplines the shared core: **no outbound call from shared code; all remote deps behind a config-source boundary.**

## What differs vs what's shared (concrete per-subsystem table)

See Topic 5 table. Headline: ~90% shared (entire data plane + query/detection/ML/connectors/creds); profile-specific = control-plane hosting, tenancy depth, identity wiring, storage binding, update mechanism, remote-dep toggles, recovery mechanism, + SaaS-only billing/fleet-canary.

## How each touched day-2 decision reconciles to be deployment-aware

| Decision | Reconciliation |
|---|---|
| **C1 (control plane: MCP HTTP + OAuth2.1 RS + built-in AS + external IdP)** | Already dual-mode-shaped. SaaS: built-in AS for vendor tenants + per-customer external-IdP SSO/SCIM federation, multi-tenant. Self-managed: built-in AS + the customer's single external IdP. Profile selects *which IdP is primary* + whether multi-customer federation is on. Identity connectors shared, config selects.[L2-GitLab-OmniAuth][L2-Grafana-OAuth] |
| **Storage (PG/SQLite/Iceberg/RocksDB)** | Same engines BOTH modes. Add a **storage-binding profile**: object-store endpoint/auth/region as config (S3 in SaaS, MinIO/local on-prem); Postgres managed (RDS) vs bundled. Constraint: shared core uses only the widely-supported S3 subset + portable Postgres SQL, never AWS-only APIs.[L2-Iceberg][L2-MinIO][L2-GitLab-PG] |
| **C2 (satellite mesh: dial-home, mTLS, residency, satellite-local creds)** | **Unchanged and central to the SaaS differentiator.** This IS the BYOC data plane in BOTH modes. In SaaS the satellites are the customer-resident data plane; central is the thin zero-access coordinator. Add: ephemeral/rotating dial-home auth (industry bar) + an explicit metadata-leakage audit.[L3-WarpStream][L3-Databricks][S6-zerotrust] |
| **Credentials (AD-017: AI-opaque, reference-based, satellite-local)** | **Unchanged and the headline trust property.** Matches WarpStream/Redpanda "credentials never leave the customer environment." Vendor SaaS-central is structurally incapable of holding sensor creds.[L3-WarpStream][S8-Redpanda] |
| **C9 (DB-authoritative config + temporal versioning + embedded-git content + opt-in remote + A/B+watchdog+satellite-self-recovery + canary+fast-revert)** | Goes deployment-conditional behind the profile: content-source = managed-remote (SaaS) vs offline-signed-bundle (on-prem air-gap); recovery = k8s blue-green (SaaS central) vs A/B-slot+watchdog (on-prem appliance), satellite self-recovery IDENTICAL; config-store = multi-customer (SaaS) vs single-customer (on-prem); canary = shared customer-scoped + SaaS-only fleet-canary layer. NO C9 primitive removed.[L4-GitLab-offline][L4-Talos][L4-ArgoFlagger][L4-Northflank] |
| **Release model** | NEW load-bearing rigor: on-prem customer-versioned upgrades with skip-version support ⇒ forward idempotent migrations + N-2 version-skew + defined upgrade paths + backward-compat windows. SaaS continuous CD exercises same engine at low skew. This is the largest concrete delta + warrants its own follow-up design.[L2-GitHub-upgrade][L2-GitLab-breaking] |

## Consolidated Open Design Questions

1. **Result-transit residency:** What query RESULTS may transit to SaaS-central vs stay customer-resident? Results can contain sensitive data. Define a residency policy or keep results customer-resident with central seeing only references/aggregates. `[INCONCLUSIVE — needs design]`
2. **Control-plane metadata boundary:** Exactly what metadata transits to SaaS-central, and a proof/audit that it cannot reconstruct sensitive data (WarpStream-style assertion). `[needs design]`
3. **Ephemeral dial-home auth:** Confirm C2 dial-home uses short-lived/rotating tokens, not static credentials (industry bar). `[verify against current C2 design]`
4. **CMEK/BYOK for central metadata-at-rest:** If SaaS-central persists any customer metadata, should it be customer-key-wrapped (Snowflake Tri-Secret model) for revocability? `[design choice]`
5. **Version-skew contract:** Pick the supported skew window (N-2?) + upgrade-path policy (multi-hop through intermediate versions?) for on-prem. `[needs decision]`
6. **Air-gap content bundle format + signing:** Signed offline content-bundle format, import workflow, signature-verification mechanism. Primary sources thin on signing specifics. `[needs design]`
7. **Closed-source feature policy:** Will any Prism feature (e.g. specific ML) be SaaS-only (Sentry model), or is full parity the goal? Affects the gating model. `[product decision]`
8. **Profile mechanism in Rust:** Confirm capability-descriptor + feature-flag gating (Grafana-analogue) vs build-time cargo features. Run-time config-selected is the lean. `[architect decision]`

## Honest Costs & Caveats

- **Test-matrix tax is real and permanent:** every feature CI'd in both profiles + an air-gap-egress-blocked profile. Ongoing cost, not one-time.
- **Air-gap is unforgiving:** a single leaked outbound dependency in shared core breaks the on-prem hard requirement. Needs an enforced CI invariant (egress-blocked build/test), not just discipline.
- **Migration rigor for on-prem skip-version is non-trivial engineering** and is the largest new work item the dual axis introduces.
- **Unified-deployment-profile abstraction is under-documented in primary sources** — Topic 4's unification synthesis is informed extrapolation from per-mechanism docs (Talos, k8s, Mender, GitHub appliance), flagged `[INCONCLUSIVE]` where it exceeds explicit vendor statements.
- **Leans are discussion input only** — not decisions, not an ADR. Architect adjudication required before any promotion to a live spec.
- **Metadata-leakage in zero-access BYOC is a genuine residual gap** even with Prism's strong architecture — the "we never see your data" claim needs the WarpStream-style explicit metadata audit to be defensible.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 4 (all reasoning_effort=high) | (1) single-codebase dual-mode patterns: GitLab/Sentry/Elastic/Mattermost/GitHub/Grafana/HashiCorp/Temporal; (2) per-axis SaaS-vs-self-managed deltas: tenancy/storage/identity/release/remote-deps; (3) BYOC/data-sovereignty/credential-trust: Databricks/Snowflake/Confluent/WarpStream/Cribl/CMEK-BYOK-HYOK; (4) deployment-conditional config/recovery/canary/air-gap-content + dual-deployment risks |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 1 | BYOC vendor-never-sees-credentials current product facts (Redpanda, Northflank, EZ-CDC, zero-trust talk) — cross-validated Topic 3 |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | No specific library/version claim required verification (Iceberg/MinIO/Postgres portability covered by primary docs via perplexity_research) |
| Tavily (all variants) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | ~3 areas (flagged inline) | GitHub Enterprise internal gating mechanics; SCIM provisioning specifics; Rust-mechanism mapping — all explicitly flagged `[model-knowledge]` / `[INCONCLUSIVE]` |

**Total MCP tool calls:** 5 (4 deep-research at high effort + 1 search)
**Training data reliance:** low — every substantive claim is sourced to a cited deep-research finding or the search results; the few model-knowledge areas are explicitly flagged inline and confined to (a) GitHub's undocumented internals, (b) SCIM specifics, (c) the Rust-mechanism lean.

**Citation legend** (citations reference the source bundle behind each perplexity_research pass; `L1`=Topic-1 pass, `L2`=Topic-2 pass, `L3`=BYOC pass, `L4`=Topic-4 pass, `S#`=perplexity_search result rank):
- L1 = dual-mode-pattern deep-research (GitLab/Sentry/Elastic/Mattermost/GitHub/Grafana/Terraform/Temporal official docs + engineering blogs)
- L2 = per-axis-delta deep-research (AWS multi-tenant guidance, Snowflake multi-tenant patterns, Atlassian Connect, GitLab Cells/OmniAuth/breaking-changes, Grafana OAuth/LDAP, GitHub Enterprise upgrade, MinIO S3-compat, Apache Iceberg AWS, Temporal, AWS Lambda tenant-isolation)
- L3 = BYOC/data-sovereignty deep-research (Databricks AWS control/compute-plane, Snowflake Tri-Secret Secure, Confluent BYOC/PrivateLink, WarpStream zero-access BYOC, Cribl Edge/Stream, CryptoMathic BYOK/CYOK/HYOK)
- L4 = config/recovery/canary/air-gap deep-research (Argo Rollouts, Flagger, k8s strategies, Talos A-B, Mender A/B OTA, GitHub Enterprise appliance, Palo Alto HA + air-gap firewall, Cisco IOS XE rollback, Zincati, Northflank multi-tenant, Microsoft progressive-exposure rings, Elastic air-gapped endpoints, GitLab offline scanners, Microsoft WSUS offline)
- S1/S5/S6/S8 = search results: Redpanda BYOC; Northflank BYOC ("genuine BYOC payload test" + cross-account links); zero-trust BYOC talk (ephemeral tokens, "vendor control = zero if possible"); Redpanda BYOC product page

*All findings as of 2026-06-27. Technology/product facts current to that date; landscapes change rapidly.*
