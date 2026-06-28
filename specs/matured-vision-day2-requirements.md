---
document_type: matured-vision
level: L1
version: "1.0"
status: day-2-backlog
produced_by: product-owner
timestamp: "2026-06-24T00:00:00Z"
demo_target: FROZEN
brief_reframe: pending-human-signoff
traces_to:
  - STATE.md D-1326
  - STATE.md D-1327
  - STATE.md D-1328
  - STATE.md D-1329
  - STATE.md D-1330
  - research/federated-search-architecture-2026-06-24.md
  - research/siem-securitylake-datalake-federation-2026-06-24.md
sources_read:
  - STATE.md (D-1326 through D-1330, current_step narrative)
  - research/federated-search-architecture-2026-06-24.md
  - research/siem-securitylake-datalake-federation-2026-06-24.md
  - specs/product-brief.md v1.1
  - specs/domain-spec/architecture-concept.md v1.1
  - specs/domain-spec/differentiators.md v1.1
  - specs/domain-spec/invariants.md v1.7 (DI-017)
  - specs/domain-spec/failure-modes.md v1.0 (FM-002)
  - specs/domain-spec/L2-INDEX.md v1.17
  - specs/prd.md v1.12
  - specs/prd-supplements/nfr-catalog.md v1.6
  - specs/architecture/system-overview.md v1.4
  - stories/S-RESILIENCE-FEDERATED-001 v1.0
do_not_execute: true
# This document is a CAPTURE artifact. It records the intended to-be state
# discussed in the 2026-06-24 day-2 vision session. It does NOT modify any
# brief/PRD/BC/architecture artifact. The morph begins post-demo, post-T14,
# gated on explicit human sign-off of the brief reframe.
---

# Matured Vision — Day-2 Requirements Capture

> **READ THIS FIRST.** This document is a single-session capture artifact.
> It records the full intended to-be state that emerged from the 2026-06-24 vision
> session. Nothing here modifies the live specs. The demo target (T13 capstone) is
> UNCHANGED and ships on the current build. All items in this document are day-2
> work, post-T14, gated on human sign-off of Section 5.1 (brief reframe).

---

## Section 1 — Purpose and Boundary

### 1.1 What this document is

This document is the authoritative capture of the matured vision and all decisions
made in the 2026-06-24 session. It serves as the launching point for the day-2 "morph"
execution: brief reframe, domain spec evolution, PRD/NFR amendments, architecture ADRs,
new BC families, story decomposition, and index updates.

It consolidates from: D-1326 through D-1330 in STATE.md; the two research artifacts
committed to `.factory/research/`; and the architect design outputs produced this session.

### 1.2 What this document is NOT

- It does NOT modify brief.md, PRD, BCs, architecture docs, or stories.
- It does NOT authorize execution of any day-2 epic (each epic requires a separate
  dispatch with human confirmation).
- It does NOT change the T13 demo target or the active LOCAL adversarial cascade.

### 1.3 Demo target: FROZEN

The T13 capstone demo target is FROZEN. The demo story S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
v1.3 is the active workstream. The LOCAL 3-CLEAN cascade (0/3 streak at session close)
continues against code HEAD 3fa69207 with corrected specs. The day-2 vision captured here
does NOT affect the demo path.

The sequence is: T13 capstone demo -> T14 recording -> day-2 morph execution.

---

## Section 2 — Matured Value Proposition

### 2.1 The corrected central framing

The product brief (v1.1) leads with "complete MSSP security operations platform" and buries
the federated query engine as a support mechanism. This is a rhetorical mismatch: the
architecture-concept.md, DEMO-SCOPE.md, and all competitive research confirm that
federated query IS the product. The brief reframe corrects this.

**Corrected central framing (pending human sign-off):**

> Prism is an ephemeral federated query engine for security operations — central,
> multi-tenant, AI-native via MCP. It queries any source valuable to a security analyst,
> in place, normalizing security telemetry to OCSF and other sources to their native
> structured schema, on demand. Demand-driven caching delivers SIEM-grade stateful
> detection and historical correlation without store-everything cost. Resilient to its
> sources: fail-fast timeouts, partial-result semantics, and auto-recovery without restart.

**What changes from the current brief framing:**
- "Per-analyst stdio" becomes "central service, multi-tenant, multi-analyst" (data-plane
  already wired; gap is the access layer — Section 3.1).
- "No data lake" becomes "ephemeral by default, cache by demand" (Section 3.3).
- "Complete MSSP platform" becomes a consequence of capabilities, not the lead claim.

### 2.2 The per-analyst to central reconciliation

The existing architecture and code are NOT wrong — the data-plane is already fully
multi-tenant (OrgId/OrgSlug/OrgRegistry/Arc-DI threading). The per-analyst stdio
deployment was correct for v1 and remains valid for single-analyst use. The matured
vision adds a CENTRAL deployment mode on top of the existing multi-tenant data-plane.
The gap is the access layer only: HTTP/streamable transport, per-connection analyst
identity, central credential custody, and shared-state access for alerts/cases.

This is additive, not a rewrite. The single-process model (DI-017) becomes
"single-central-service" — the constraint is the deployment topology, not the process
architecture.

### 2.3 Five value-prop statements (research-grounded)

These five statements are derived from the federated-search research (39 cited sources,
2026-06-24) and are citation-grounded, not marketing copy.

1. **Query your security tools where the data already lives — no ingestion, no duplication,
   no egress.** Prism is an ephemeral federated query engine, not another data lake to fill.
   Validated market language: Query.AI [V-4][V-11], Splunk [V-13], Gurucul [V-5], PwC [V-17].

2. **Stop paying twice. Cut SIEM ingestion cost by querying in place.** Federation keeps
   low-value/high-volume telemetry in cheap storage and queries it on demand — the
   Autodesk-via-Splunk pattern delivered 28% ingestion-cost reduction [V-13].

3. **Respect data residency by design — answers cross borders, your data does not.** Prism
   queries in-region and normalizes results at query time; raw customer data never leaves
   its jurisdiction. Sovereignty canon: Gurucul [V-5], Splunk [V-13], Pax8 [V-8].

4. **One normalized view across every tool — OCSF at query time, not after a six-month ETL
   project.** Heterogeneous source APIs become a single OCSF schema the moment you query.
   OCSF-as-differentiator: [V-4][V-6][V-7][V-12][V-19].

5. **Federated search built for the analyst's agent, not for yet another browser tab.**
   MCP-native tool inside Claude Code, with credentials the AI never sees and output hardened
   against prompt injection. Prism-unique whitespace — not occupied by any cited competitor
   [V-4][V-11][V-12].

### 2.4 Honest tradeoffs (internal positioning discipline)

Federation is NOT equivalent to a centralized lake for:
- Deep historical analytics and complex long-window correlation (lake wins on retained data).
- Very large-scan queries (object-store + catalog round-trips add seconds; not SIEM-hot-path
  interactive at PB scale).

Prism's sweet spot: investigative, entity-centric, time-bounded queries. The demand-driven
cache (Section 3.3) narrows the gap for detection-window correlation without becoming a
store-everything lake.

### 2.5 Neutral-incentive credibility

Cribl's dual stance (federate OR ingest) is read as credible because it has no legacy SIEM
revenue to protect. Prism's incentive structure is similar — sensor-API-native, no
ingestion revenue — giving the "replace OR federate the SIEM/lake" dual stance genuine
credibility. This is an asset. The positioning must be framed as "capability-first,
source-agnostic," NOT "we can be your lake AND query your lake."

---

## Section 3 — Architectural Pillars

### 3.1 Central deployment (from per-analyst to multi-analyst service)

**Concept:** The data-plane is already fully multi-tenant. The gap is the access layer:
an analyst-facing HTTP/streamable-HTTP transport (replacing stdio for multi-client use),
per-connection analyst identity (authN/authZ), central credential custody (credentials
stored in the service, not on each analyst's machine), and shared-state access (a central
alert/case store visible to all analysts working on the same org).

**Current state (as-is):** Per-analyst stdio transport (DI-017 "serves a single analyst").
OrgId/OrgSlug/OrgRegistry/Arc-DI multi-tenant wiring is present and correct.

**Target state (to-be):** Central deployment option added. Stdio remains for single-analyst
local use. Central mode exposes a streamable HTTP transport with per-connection identity.

**Key implications:**
- DI-017 must be amended: "single-process" becomes "single-central-service; the stdio
  transport constrains to single-analyst, the central transport enables multi-analyst."
- BC-2.10.001 (per-analyst per-connection) and BC-2.10.006 (stdio transport) require
  amendment to reflect transport-selectable binding.
- BC-2.05.002 (per-connection audit identity) requires amendment to capture analyst
  identity from the transport layer, not from an assumed single-user process.
- BC-2.04.002 / BC-2.04.011 (credential access) require amendment for central credential
  custody model.
- New BC family needed for central authN/authZ.

**Epics and ADRs (all day-2):**
- E-CENTRAL-TRANSPORT-001: HTTP/streamable transport, per-connection identity propagation.
- E-CENTRAL-AUTHZ-001: Analyst authN/authZ, per-connection capability enforcement.
- E-CENTRAL-OPS-001: Central credential custody, shared alert/case state, operational
  tooling for the central service.
- ADR-050: Central deployment topology (transport selection).
- ADR-051: Per-connection analyst identity model.
- ADR-052: Central credential custody design.
- ADR-053: Shared state access (alerts/cases across analysts).
- ADR-054: Central service operational model (startup, health, scaling).

**Architect confirmation (D-1327, a129e53b8894aab78):** T13 demo needs no change from
this pivot. The demo runs on the current per-analyst stdio path, which remains valid.

> **Day-2 addendum (2026-06-26 side analysis — DECIDED 2026-06-26 (human)).** C1 decisions are
> now SETTLED and captured in `specs/day2-design-decisions/ADR-PROP-central-deployment-access-layer.md`.
> Summary of what is decided:
>
> - **Transport (→ADR-050):** MCP **Streamable HTTP** (spec rev 2025-06-18; replaces deprecated
>   HTTP+SSE) + keep stdio for single-analyst/local. Implement via in-tree **`rmcp 1.7.0`
>   `StreamableHttpService`** (`stateful_mode`, pluggable `SessionStore`, DNS-rebinding
>   `allowed_hosts/origins`, bearer plumbing) — mount + middleware, not new protocol code.
> - **Identity (→ADR-051):** MCP server = **OAuth 2.1 Resource Server** (MCP Authorization spec
>   2025-06-18; audience RFC 8707; Protected-Resource-Metadata RFC 9728; 401/WWW-Authenticate
>   discovery). Token → analyst identity → OrgId scope → per-connection capability gate
>   reusing the existing write-gate feature-flag model. Stdio stays env-credential unchanged.
>   **Built-in OAuth 2.1 AS + external IdP (OIDC/SAML)** — hybrid; mirrors the §11.1 hybrid
>   secret-store stance.
> - **Credentials (→ADR-052 + SS-26):** SS-26 design stands unchanged; the only C1 addition is
>   per-connection-analyst **audit binding** on every credential resolution.
> - **Shared state (→ADR-053):** **FULL case-management** (status/assignment/case-wall/links/
>   disposition + §11.3.1 case-detail UI surface) on **BUNDLED PostgreSQL** (per
>   `ADR-PROP-storage-engine-taxonomy.md` — Postgres central-only, never external/cloud, never
>   at a Satellite). **Optimistic concurrency (per-record version/ETag CAS) + soft ownership +
>   presence hints** — no hard pessimistic locks (SOC/TheHive/SOAR prior art). `409 Conflict`
>   on stale write.
> - **Ops (→ADR-054):** stateless-leaning front + shared state; SSE resumability via
>   `Last-Event-ID` (no sticky sessions); readiness-flip + connection-drain graceful shutdown
>   (rmcp `cancellation_token`); per-tenant fairness via nested Tokio `Semaphore` /
>   weighted-fair-queue (§5.3 NFR-CONCURRENCY-FAIRNESS).
> - **Storage taxonomy:** The storage decision in ADR-053 is part of a wider FOUR-ENGINE taxonomy
>   decided at the same time — see §14.3 addendum and `ADR-PROP-storage-engine-taxonomy.md`.
> - **Research citation:** `research/central-deployment-access-layer-2026-06-26.md` (Topics 1–5;
>   MCP spec 2025-06-18 transports + authorization; rmcp 1.7.0 docs; SOC concurrency + scaling
>   patterns prior art).

### 3.2 Prism Satellite and multi-hop chaining

**Concept:** A "Prism Satellite" is a remote query executor deployed at a client site,
plant, or network enclave. The central Prism service acts as coordinator/planner; satellites
act as remote executors. Communication is outbound-only from satellite to central
(dial-home), compatible with strict firewall policies that permit only upstream connections.

**Satellite chaining (hub-spoke + tree topology):** Satellites can chain:
satellite -> satellite -> ... -> Prism. Each hop relays execution requests inward and
results outward. The tree is rooted at Prism (the coordinator). A mid-chain offline node
drops its subtree from the fan-out; partial-result semantics (Section 3.6) propagate the
gap upward.

**Per-hop guarantees:**
- Mutual authentication at each hop.
- Deadline and partial-failure metadata propagate through the chain (no hop can silently
  swallow a downstream failure).
- Regional caching and store-and-forward for intermittent/low-bandwidth edges.
- Enrollment protocol (satellite registers with upstream, receives a trust anchor).
- Heartbeat through the chain (loop prevention, topology health).
- Loop prevention: each hop tracks seen request IDs; duplicate IDs are rejected.

**Primary use cases for satellite topology:**
1. **OT/ICS Purdue-model layered segmentation.** Industrial networks enforce layer
   separation (enterprise -> DMZ -> OT -> Level 2 -> Level 1). A chain of satellites
   traverses the layers; flattening the network is not an option. Each satellite queries
   only its own layer's sources.
2. **Air-gapped enclaves via single bastion.** The bastion satellite bridges the gap;
   sources inside the enclave are queryable from central Prism without opening the enclave.
3. **MSSP nested topology.** spoke -> regional-hub -> central. Subsidiaries or regional
   teams run their own satellites; the central Prism aggregates across the tree.
4. **Remote/intermittent/low-bandwidth edges.** A satellite with regional caching buffers
   results during connectivity gaps and delivers via store-and-forward when reconnected.
5. **Firewall-permits-only-upstream.** Many customer environments permit only
   customer-to-internet, not internet-to-customer. Dial-home satisfies this.
6. **Fan-in and data-residency hops.** A regional satellite aggregates local sources and
   enforces that raw data never leaves the region; only normalized, sanitized results
   transit to central.

**Key implications for spec changes (day-2):**
- New entity: Satellite (topology node, trust anchor, endpoint, health state).
- New invariant: per-hop mutual auth is non-negotiable.
- New subsystem in ARCH-INDEX.md: Satellite Mesh.
- New epic: E-SATELLITE-MESH-001 (satellite registration, dial-home transport, chaining,
  partial-failure propagation through hops).
- New ADRs: satellite enrollment protocol, chaining depth limits, loop prevention.

**Name confirmed by human (D-1330):** "Prism Satellite."

---

> **DECIDED 2026-06-27 (human) — C2 Satellite Mesh Design Decisions D-C2-1…13.**
> Full capture: `specs/day2-design-decisions/ADR-PROP-satellite-mesh.md`. Research basis:
> `research/satellite-mesh-2026-06-26.md`. Summary of decisions (do_not_execute until morph):
>
> - **D-C2-1 Transport:** gRPC bidirectional streaming over HTTP/2:443 via `tonic` = PRIMARY /
>   default (reverse-RPC: coordinator pushes requests DOWN, reads results UP the satellite-initiated
>   stream). NATS leaf-node hierarchy = STRONG ALTERNATIVE (topology + JetStream S&F + reconnect
>   for free; embedded-broker cost). Final either/or gated on prototype bake-off. Explicit cost:
>   TCP-HOLB on single-connection HTTP/2 mixed control+bulk; HTTP/3/QUIC (`quinn`) is the eventual
>   HOLB upgrade path.
> - **D-C2-2 Relay trust role:** Relay Satellite = mTLS TERMINATOR / re-originator ONLY. Does NOT
>   act as sub-CA; vends NO cross-hop credential.
> - **D-C2-3 Role nouns (architectural lean):** Coordinator (central root) / Relay Satellite
>   (interior executor+aggregator) / Edge Satellite (leaf executor). §3.4 BA+PO finalization owns
>   the real decision.
> - **D-C2-4 Diode / one-way OT mode:** DEFERRED. Day-2 = bidirectional mTLS mesh only. Recorded
>   as explicit open design question / future epic E-SATELLITE-DIODE-001. Mutual-auth mTLS likely
>   precludes a true unidirectional link without a separate store-and-forward + one-way-result
>   transport variant.
> - **D-C2-5 Identity:** SPIFFE-model (`prism-sat://<trust-domain>/<sat-id>` URI → short-lived
>   X.509-SVID chaining to a per-trust-domain CA). Implemented NATIVE RUST. SPIRE is NOT a runtime
>   dependency (air-gap/edge + ephemeral ethos). Bootstrap secret + private key use Prism newtype +
>   redacted-Debug credential discipline (AD-017).
> - **D-C2-6 Trust model:** per-hop mutual mTLS ONLY. No transitive trust. Explicitly rejects the
>   Teleport root-CA-reaches-leaf foot-gun. Required for IEC-62443 zone separation across Purdue
>   layers.
> - **D-C2-7 Bootstrap:** SPIRE-style one-time/TTL join token, out-of-band distribution at deploy.
>   Optional TPM attestation as hardening upgrade for high-assurance OT.
> - **D-C2-8 Loop prevention:** belt-and-suspenders — seen-request-ID set (existing §3.2) + hop-count
>   TTL decremented per hop (IP-TTL analog, hard ceiling) + OPTIONAL path-vector (BGP AS-path analog;
>   free topology/health diagnostics).
> - **D-C2-9 Deadlines:** gRPC per-hop deadline decrement verbatim (absolute deadline; residual =
>   deadline − now − hop-budget; fail-fast on non-positive residual). Ties §17.8 Q3 v1.
> - **D-C2-10 Store-and-forward:** RocksDB-backed durable queue (new CF) at collection-capable
>   Satellites (lowest-new-dependency; Prism already runs RocksDB). Bounded buffer; drop-oldest +
>   loud coverage signal on fill; NEVER silent loss. Transient (buffer-and-replay) vs. hard
>   (deadline-exceeded subtree → skipped) failure classes explicitly distinguished.
> - **D-C2-11 Partial-failure:** extend BC-2.01.010 partial-result + §3.6 coverage banner (CCS
>   skip_unavailable lineage). A relay surfaces a lost child's subtree as skipped (reason +
>   last-seen ts) and relays the gap UPWARD UNMODIFIED through every hop. No hop swallows a
>   downstream failure (binds Standing Rule 3 §2 no-silent-Vec::new()).
> - **D-C2-12 Residency:** structural enforcement — Satellite normalizes raw → OCSF/native
>   AT THE EDGE in-zone; only normalized result transits conduit upward; raw NEVER crosses a
>   Satellite boundary. IEC-62443 zones-and-conduits mapping explicit (one Satellite per zone,
>   inter-Satellite hop = conduit, per-hop mTLS = conduit authentication control; NIST SP 800-82
>   companion). Satellite-local credential resolution = HARD INVARIANT (creds resolved AT the
>   Satellite, never sent to central; binds AD-017 / project_ai_opaque_credentials.md).
> - **D-C2-13 Max chain depth:** hop-TTL ceiling = **8 hops** (production default). Rationale:
>   deepest expected MSSP topology is 7 hops (OT L1 → L2 → L3 → enterprise → DMZ → regional hub →
>   national hub → coordinator); +1 safety margin. Configurable; operators with simpler topologies
>   should set a tighter ceiling.

---

### 3.3 Demand-driven caching / smart retention (SIEM replacement by capability)

**Concept:** Prism's existing architecture is ephemeral-by-default. Demand-driven caching
adds a per-policy retention layer: data is cached IF AND ONLY IF it is needed by an active
detection rule window or an explicit RETAIN directive. This is not "store everything" — it
is "store exactly what detection needs, for exactly as long as detection needs it."

**Design (D-1328, architect output a7ad2eedf80ba88d1):**
Demand-driven caching is a GENERALIZATION of patterns already present in Prism
(diff_results, detection_state, infusion_cache, event_buffer, in-memory SensorResponseCache).
It is NOT an architectural rethink.

The implementation adds:
- A new RocksDB column family: `StorageDomain::RetentionCache`.
- A Retention Policy Engine that reads three policy sources in priority order:
  1. Detection-rule window TTL: a detection rule with a 1-hour correlation window
     causes records matching its scope to be cached for 1 hour. This is inferred
     automatically from the rule's window definition — no analyst action required.
  2. Explicit PrismQL RETAIN directive: `RETAIN <duration> [AS <name>]` in a query
     caches the result set and makes it queryable as `FROM cache.<name>`.
  3. Config-level `retention_default` per table: a fallback TTL for tables where
     explicit policies haven't been set.
- OCSF-normalized records only (Prism's existing normalization boundary).
- `event_time`-based TTL (not wall-clock insertion time).
- zstd compression.
- Approximately 200MB cap (within the raised process budget ceiling; Section 4 item 5).

**This directly fixes DI-029 (correlation window >= schedule interval):** the cache
enables detection rules to access records that arrived between schedule executions,
closing the interval-vs-window gap without requiring the analyst to tune intervals.

**The SIEM replacement claim is a capability claim, not a storage claim.** Prism's
detection-rule-driven retention delivers SIEM-grade stateful detection and historical
correlation for the time windows detections require — without requiring a permanent
data store.

**Epic and ADRs (day-2):**
- E-CACHE-DEMAND-001 (story prefix S-CACHE-DEMAND-NNN), four phases:
  - P1: detection-window retention (no PrismQL dependency; prism-operations/prism-storage/
    prism-core). Can start in parallel post-T14.
  - P2: PrismQL RETAIN syntax + FROM cache. virtual table. Gates on grammar stability
    (S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 merged).
  - P3: config retention defaults.
  - P4: cold-tier >7 days (deferred beyond day-2 scope).
- ADR-047: RetentionCache design (StorageDomain::RetentionCache CF, policy engine,
  TTL semantics).
- ADR-048: PrismQL RETAIN semantics (syntax, FROM cache. virtual table, naming rules).
- ADR-049: Detection window expansion (DI-029 amendment, cache-backed correlation).
- New subsystem: SS-23 (prism-retention crate or extension of prism-operations +
  prism-storage).

**Research queued pre-P1 implementation:**
- R1: DataFusion TableProvider over RocksDB range scans (canonical pattern).
- R2: Detection-driven caching in streaming security engines (prior art).
- R3: event_time-vs-wall-clock TTL semantics (correctness for out-of-order events).

> **Day-2 addendum (2026-06-25 side analysis — HUMAN DIRECTIVE).** The RetentionCache is **tiered**,
> and **multi-schema** (not OCSF-only). Demand spans seconds → multi-year, which is two storage
> regimes:
> - **Hot tier** — in-memory + the `StorageDomain::RetentionCache` RocksDB CF (current design):
>   seconds → hours/days, point lookups, detection-window correlation, low latency.
> - **Cold/long tier** — **Apache Iceberg** (Parquet-on-object-store + catalog): days → multi-year
>   `RETAIN`. Columnar, partition-pruned on `event_time`/`eventDay`, schema-evolution, time-travel,
>   zstd-in-Parquet, cheap object storage.
>
> The Retention Policy Engine routes by retention duration: short TTL → RocksDB hot; long /
> explicit-`RETAIN` → Iceberg cold. **`RETAIN <dur>`** writes rows to an Iceberg table; **`FROM
> cache.<name>`** reads them. This is the concrete implementation of the previously-deferred P4
> cold-tier (below), and it **unifies the cache with the Amazon Security Lake connector (§3.5) at the
> DataFusion ENGINE level: one DataFusion engine, two TableProviders — the self-managed cold tier reads
> via `IcebergTableProvider`; Amazon Security Lake reads via a distinct Glue/Hive-Parquet (`ListingTable`)
> provider. The engine-level unification (no second query engine) holds; the storage-format equivalence
> does not** (Security Lake is OCSF Parquet in Hive-style partitions, NOT Apache Iceberg — see D-C5-1,
> 2026-06-27). The long-baseline storage also serves statistical/anomaly detection (Section 14).
>
> **Multi-schema (critical):** the Iceberg cold tier is **NOT one OCSF table.** It is a set of tables
> keyed by **(source-class, schema, schema-version)** — OCSF-vN tables (OCSF is itself versioned:
> 1.1, 1.3, …) AND **native schema-on-read tables** for cached non-security-connector data (§3.4,
> §13). Iceberg schema-evolution handles OCSF version drift. Iceberg overhead is unsuited to the
> seconds-scale hot path, so it stays cold-tier only — hot stays RocksDB. See §13.6 for the full
> multi-schema model.

**Human decision H2 (D-1328):** SIEM-replacement = DECIDED. Prism's demand-driven cache
makes it a SIEM replacement by capability. Human directive. H1/H3/H4/H5 deferred to
caching-epic kickoff post-demo.

### 3.4 Source and connector taxonomy + analyst-value selection principle

**Concept:** The current ubiquitous language uses "sensor" for all data sources. This
is accurate for the four initial security telemetry sources (CrowdStrike, Cyberint,
Claroty, Armis) but incorrect as a generalization. As Prism's adapter model expands to
cover all sources valuable to a security analyst, "sensor" confuses a subtype
(security telemetry sensor) with the category (any queryable source).

**Proposed taxonomy (day-2 ubiquitous-language decision, business-analyst + PO to finalize):**
- **Source / Connector:** The generic term. Any queryable endpoint that Prism can
  execute against. Includes security sensors, network infrastructure, identity stores,
  file shares, databases, SIEMs, and data lakes.
- **Sensor:** A subtype of Source/Connector that produces security telemetry (alerts,
  detections, vulnerabilities, threat intelligence). CrowdStrike, Cyberint, Claroty,
  Armis are sensors. OCSF normalization applies to sensor output.
- **Connector (non-security):** A Source that produces structured but non-security-telemetry
  data (network switch, Windows event log as raw syslog, Active Directory, Excel on a file
  share, a SQL database, a SIEM or data lake as a queryable store). Normalization is
  native/structured/schema-on-read — not OCSF.

**Selection principle:** The criterion for including a Source type is: "Is this data
valuable to a security analyst?" This is intentionally broad and analyst-driven, not
domain-restricted.

**Examples of day-2 source types (all beyond current scope):**
- Network infrastructure: switches, firewalls (queries for port state, MAC tables).
- Identity: Active Directory, Okta, Entra ID (user/group/privilege queries).
- Files/shares: Excel files on SMB shares, CSV export feeds (ingest-on-demand).
- Databases: SQL databases containing security-relevant records (vulnerability tracking,
  asset inventory).
- Security APIs: MITRE ATT&CK API, VirusTotal, Shodan (enrichment sources).
- SIEMs/Lakes: Splunk, Google SecOps, Amazon Security Lake, Sentinel data lake
  (federate-into mode, Section 3.5).

**Multi-protocol connectors (day-2):**
- HTTP/REST: today (spec-engine TOML-driven, functional).
- SSH/CLI: query remote hosts via SSH command execution.
- WMI/WinRM: Windows instrumentation queries.
- LDAP: directory queries (Active Directory, LDAP-compliant directories).
- SMB/file: read structured files from network shares.
- SQL: query relational databases via JDBC/ODBC-equivalent Rust connectors.

**Terminology decision status:** "Source/Connector" vs alternatives is a day-2
ubiquitous-language decision owned jointly by business-analyst and PO. The term
"sensor" is retained for the security-telemetry subtype. "Prism Satellite" is CONFIRMED.
The role noun for a satellite that acts as both executor and upstream relay in the chain
(the "relay" or "aggregator" role) is TBD.

> **Day-2 addendum (2026-06-25, side analysis — see Section 10).** The source/connector
> generalization needs a missing technical spine: a **per-connector capability descriptor**.
> Every connector TOML should declare its pushdown capability profile — which predicate
> classes (equality, range, IN, prefix), aggregations, group-by, sort, limit, and join it
> can execute natively vs. what PrismQL must compute centrally in DataFusion. This is the
> direct analog of Trino's connector SPI (`applyFilter`), DataFusion's
> `supports_filters_pushdown`, and Athena's federated-connector pushdown negotiation. Without
> it, "any source valuable to a security analyst" has no planner contract. See Section 10
> §10.3 (ADOPT-1) and §10.5 gap G-2.

> **DECIDED 2026-06-27 (human) — C3 Capability-Descriptor + PrismQL Pushdown + Cross-Source
> Cost Guards.** Four architecture decisions confirmed; leans confirmed. Capture artifact:
> `specs/day2-design-decisions/ADR-PROP-capability-descriptor-pushdown.md`
> (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
> `research/capability-descriptor-pushdown-2026-06-26.md`. Hardening pass on DataFusion 50.x
> mechanics in flight (`research/datafusion-cost-degrade-mechanics-2026-06-27.md`).
>
> **D-C3-1 Join guard = COST-BASED DEGRADE (Trino-lineage), NOT hard-reject.** Cross-source
> joins are ALLOWED. Cost is mitigated by: mandatory row-caps per side + on output,
> DataFusion 50.x inner-equi dynamic filter (sideways-information-passing) as the primary
> cost lever, partitioned-not-broadcast distribution for unknown-cardinality sources, and
> resource-based abort after consumption (query wall-clock timeout + memory limit) with
> partial-result coverage metadata. There is NO plan-time rejection of unbounded joins.
> **This supersedes the earlier §5.3/§12.2 "reject unbounded joins / mandatory selective
> key-predicate reject" framing — see reconciliation notes at §5.3 and §12.2.**
>
> **D-C3-2 Missing time-bound = INJECT DEFAULT WINDOW + DISCLOSE (NOT reject).** If no
> explicit time bound, PrismQL injects a configurable default window (e.g., last 24h) at
> plan-time; pushes as an exact range predicate where the descriptor declares range-pushdown
> on the time column; ALWAYS surfaces the effective time-bound in the response envelope +
> a structured event (`event_type = "query.injected_default_window"`, BC-2.16.002 catalog
> row required at morph). Asymmetric with D-C3-1 by design.
>
> **D-C3-3 Cross-source join shape = ALLOW OUTER/NON-EQUI (central-only).** Inner equi-joins
> get the DataFusion 50.x dynamic-filter (sideways-information-passing) cost lever; outer/
> non-equi cross-source joins are PERMITTED but fall back to full central execution via
> `NestedLoopJoinExec` WITHOUT dynamic-filter optimization (weaker cost guarantee, owned
> consciously). A bare cross-source CROSS JOIN / comma-join is allowed but LOUDLY FLAGGED
> (cost/coverage disclosure in response envelope) and bounded by the row-cap — not silently
> executed, not rejected.
>
> **D-C3-4 Override = AUDITED PrismQL HINT, capped at absolute max.** An explicit query-level
> PrismQL hint can raise the row-cap toward the configured maximum but NEVER beyond the
> absolute maximum. Every override emits `event_type = "query.override_applied"` (BC-2.16.002
> catalog row required at morph).
>
> **Confirmed leans:** declarative TOML capability descriptor per `[[tables]]`, fail-closed
> default (undeclared = Unsupported → central compute); enumerated predicate-class vocabulary
> (eq/range/in_list/prefix/like/null), each tagged exact|inexact; contract split —
> DataFusion `TableProvider::supports_filters_pushdown` for filter/projection/limit,
> PrismQL pre-pass for time-bound injection + row-cap enforcement + join cost guard;
> descriptor is per-(table, schema-class); transform exactness via bijection test;
> collector subtype declares `pushdown_target = buffer`; minimum DataFusion version = 50.x;
> `#[non_exhaustive]` on all descriptor structs (CLAUDE.md discipline).

> **DECIDED 2026-06-27 (human) — C4 Dynamic-Schema / Configure-Schema Connectors.**
> Four architecture decisions confirmed; leans confirmed. Capture artifact:
> `specs/day2-design-decisions/ADR-PROP-dynamic-schema-connectors.md`
> (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
> `research/dynamic-schema-connectors-2026-06-27.md`. Hardening pass on
> boundary-normalization + WASM sandbox mechanics in flight
> (`research/connector-boundary-sanitization-wasm-2026-06-27.md`).
>
> **D-C4-1 Boundary-normalization scope = ALL CONNECTORS including existing OCSF security
> sensors (CrowdStrike/Cyberint/Claroty/Armis). NO trusted-source exemption.** A mandatory
> fail-closed connector-boundary normalization/sanitization chokepoint (NFC + single-script
> allowlist + length-cap + control-char/bidi reject + confusable/skeleton detection +
> structural data/instruction separation + read-only-default action layer) applies to every
> source before any schema element or value reaches an agent. Honest cost: adds a
> normalization chokepoint and latency to the existing prism-sensors hot path — real
> day-2 morph item; concrete buildable mechanism + hot-path performance cost under
> hardening research (OQ-C4-1..3).
>
> **D-C4-2 Drift on upstream column REMOVAL = AUTO-NARROW + structured drift event.**
> When an introspection probe detects a pinned column is gone upstream, Prism
> automatically marks it unavailable and emits `connector.schema.drift.detected`. No
> operator re-pin required for narrowing (safe: surface only shrinks). Column ADD upstream
> → surface + ignore (invisible until re-pinned). Column RETYPE upstream → hard drift:
> mark unavailable + surface + require re-pin. See D-C4-7 (confirmed lean) for full
> drift classification.
>
> **D-C4-3 WASM code-connector escape-hatch COMMITTED in day-2.** Declarative TOML is the
> default; an audited, sandboxed WASM escape-hatch (stronger posture than Airbyte's
> no-sandbox `AIRBYTE_ENABLE_UNSAFE_CODE`) covers imperative cases: custom auth signing,
> stateful/computed pagination, response reshape/flatten, dynamic stream generation,
> async-job polling, non-REST protocols. Builds on Prism's existing plugin SDK
> (`crates/prism-spec-engine/plugins/`); day-2 WASM connector capability/sandbox model
> MUST be reconciled against the existing plugin SDK at morph time (not a parallel
> mechanism). Sandbox details under OQ-C4-6.
>
> **D-C4-4 Hostile/suspicious identifier handling = QUARANTINE + RELABEL to safe
> placeholder; original preserved (encoded) in audit field so operator sees the attack
> without the agent ingesting it raw. HARD-REJECT only on hard violations (control chars /
> bidi overrides / over-length).** Mechanism detail (encoding, placeholder naming scheme,
> audit field format) under OQ-C4-4 hardening pass.
>
> **Confirmed leans:** static-declared TOML is the dogfood default (introspection/inference
> = opt-in confirm-or-narrow-only probes, NEVER auto-widen); two-hop type mapping
> source-native → Arrow → Prism ColumnType (map-to-canonical-or-reject; lossy coercions
> weaken C3 pushdown exactness on that column to inexact; `lossy = true` TOML flag for
> explicit fallback to Json/Text; timezone-naive silent cast = lossy coercion flagged;
> do NOT reintroduce retired `prism_spec_engine::types::ColumnType` shadow enum —
> ADR-024); drift = event to surface, never silent adaptation (Confluent BACKWARD/FORWARD/
> FULL vocabulary; Fivetran supertype promotion explicitly rejected; Iceberg field-ID
> evolution for cold tier); config-vs-code boundary test = formulaic REST/SQL/LDAP →
> TOML; any imperative state / non-REST / custom signing → WASM; DataFusion integration:
> `schema()->SchemaRef` built from PINNED TOML, boot-time C3↔C4 reconciliation invariant
> (descriptor.columns ⊆ provider.schema().fields, fail-closed on over-declaration). C4
> builds on C3 — the schema C4 discovers/declares is the surface C3 annotates with pushdown
> exactness. Downstream SAP-1 obligations: BC-2.16.002 new catalog rows for
> `connector.schema.drift.detected`, `connector.schema.identifier.sanitized`,
> `connector.schema.identifier.rejected`, `connector.schema.coercion.lossy` — morph-time.

### 3.5 SIEM / Security Lake / Data Lake — federate-or-replace dual stance

**Concept:** SIEMs, security lakes, and data lakes are not only competitors; they are
also source types in Prism's adapter model. Prism can:
(a) Replace by capability: deliver the same detection and correlation capabilities from
    live sensor APIs + demand-driven cache, without requiring a SIEM or lake at all.
(b) Federate into: treat an existing SIEM/lake as just another queryable source, querying
    it in place alongside live sensor APIs.

**Amazon Security Lake is the highest-value first lake connector.** OCSF fit is real,
not marketing: Security Lake stores OCSF-normalized Apache Parquet (OCSF v1.1.0 native
sources, OCSF 1.3 for custom sources) exposed as Apache Iceberg tables in AWS Glue Data
Catalog since February 2024. Because Prism already normalizes to OCSF, the lake adapter
transform stage collapses from "parse + map every field to OCSF" to "pass-through OCSF
records + carry unmapped attribute bag" — near-zero semantic normalization work.

**Two access modes for the lake/SIEM adapter type (architect recommendation, routes to ADR):**
- Mode A — query-access subscriber (primary): in-place SQL via Iceberg/Glue/Lake Formation;
  cross-account IAM role + Lake Formation SELECT grants; mandatory `eventDay`/`time_dt`
  predicate push-down (cost guardrail); Parquet reads from S3. Aligns with ephemeral
  federation thesis.
- Mode B — cache-hydrate subscriber: S3 + SQS new-object notifications; pull-into
  RetentionCache on demand or on-event. Aligns with demand-driven caching (Section 3.3).

**Dual-stance credibility:** Prism's incentive structure (sensor-API-native, no ingestion
revenue) is Cribl-like neutral, making the dual stance credible. A dual stance is read
as hedging when the vendor has conflicted incentives (Splunk's "federate OR ingest" is
skeptically received because Splunk's revenue favors ingestion). Prism does not have
this conflict.

**Honest tradeoff:** Deep historical analytics (years of retention) remain a lake
advantage. Prism's federation + demand-driven cache wins on freshness, zero-duplication,
and data-gravity avoidance. The answer to "can Prism replace my SIEM?" is: "For
detection-window correlation and investigative queries, yes. For 3-year cold forensics,
federate into your lake — Prism queries it."

**Epic (day-2):** E-LAKE-CONNECTOR-001. Amazon Security Lake first; generic
Iceberg/Parquet-on-S3 second; Splunk/Sentinel/Snowflake-fronted lakes third.

> **Day-2 addendum (2026-06-25, side analysis — see Section 10).** The competitive research
> on Query.io surfaces a useful connector dichotomy Prism should adopt explicitly:
> **static-schema sources** (pre-mapped sensors — CrowdStrike/Cyberint/Claroty/Armis, the
> current model) vs. **dynamic-schema sources** (Splunk/Sentinel/Snowflake/BigQuery/lakes,
> where the schema must be introspected, partitioning auto-discovered, and fields mapped to
> OCSF via a no-code "configure-schema" workflow). The lake/SIEM Mode-A query-subscriber path
> (above) IS a dynamic-schema source. This split, and the GAV-vs-LAV mediation decision it
> implies, should be ratified in an ADR (Section 10 §10.3 ADOPT-6, ADOPT-8; gap G-5).

### 3.6 Federated-connectivity resilience (S-RESILIENCE-FEDERATED-001)

**Concept:** Prism already auto-recovers from connectivity loss at the architectural level
(no latched failure point: auth re-acquires per-query, reqwest pool recovers, semaphore
permits are RAII-released, AdapterRegistry does not latch). The gap is:
- No separate `connect_timeout` (only total-request timeout) — fail-slow, not fail-fast.
- No boot-degraded mode — a sensor unavailable at startup blocks the boot path.
- No per-sensor `skip_unavailable` flag — a down sensor blocks queries across the board.
- Availability cache missing — no fast-fail path for prolonged outages.
- Hot credential reload for static-token sensors (Armis/Claroty) is missing (FM-002 gap).
- Unwired `timeout_secs` overlay field in prism-spec-engine/src/overlay.rs.

**Scope of S-RESILIENCE-FEDERATED-001 (stub registered D-1329; day-2 epic):**

Per-sensor TOML schema additions:
```toml
[connectivity]
connect_timeout_secs   = 5      # separate from request_timeout (fail-fast on unreachable)
request_timeout_secs   = 30     # existing rule preserved; maps to reqwest .timeout()
skip_unavailable       = true   # CCS-style: partial results if this source is down

[connectivity.retry]
max_attempts           = 3
backoff_initial_ms     = 200
backoff_max_ms         = 5000
jitter                 = "full" # mandatory per AWS/SRE jitter canon
retry_on_status        = [429, 503, 504]
respect_retry_after    = true
```

Additional in-scope behaviors:
- Boot-degraded mode: start with unavailable sensors reporting DEGRADED, not blocking boot.
  Diagnostics log which sensors are unavailable at startup.
- Sensor availability cache: per-sensor health state with TTL; fast-fail path for
  prolonged outages without circuit-breaker complexity (validated: no circuit breakers
  at Prism's QPS per D-1327 research; availability cache is the correct substitute).
- Hot credential reload for static-token sensors (G2 AUTHORIZED by human — see Section 4).
  Armis/Claroty bearer tokens can be reloaded without restart, closing FM-002.
- Recover-without-restart: combined auto-reconnection + availability cache + hot credential
  reload = no restart needed for any connectivity or credential expiry scenario.
- Unify `timeout_secs` overlay field: the existing accepted-but-unwired overlay field
  maps to `request_timeout_secs`; do not leave two parallel timeout concepts.

**No circuit breakers (research-validated D-1327):** Microsoft .NET Polly guidance
explicitly states below ~1 req/s "the traffic volume is too low to justify a circuit
breaker policy" [R-12]. At Prism's QPS, a circuit breaker is inert or harmful under
misconfiguration. Decision: omit circuit breakers; use fast-fail timeouts + bounded
retry + bulkhead (existing MAX_FANOUT_CONCURRENCY=10) + availability cache.

**Partial-result semantics (Elasticsearch/OpenSearch CCS-style):** a query with some
sensors unavailable returns HTTP 2xx with partial results + structured per-source failure
metadata (sensor_errors). Already present in BC-2.01.010; the resilience story wires the
`skip_unavailable` TOML flag to control which sensors are best-effort vs required.

**BC families needed (day-2 PO authorship, blocks story status:ready):**
- BC family for per-sensor connectivity config (connect_timeout, request_timeout,
  retry schema, skip_unavailable).
- BC family for boot-degraded mode.
- BC family for sensor availability cache.
- BC family for hot credential reload (static-token sensors).
- BC family for recover-without-restart test (the no-restart guarantee).

---

## Section 4 — Decisions Ledger

All decisions made or confirmed in the 2026-06-24 session. HUMAN CALL items require
no further authorization for day-2 execution. ADR-NNN items require formal ADR authorship
before implementation begins.

| ID | Decision | Type | Source |
|----|----------|------|--------|
| DC-001 | SIEM/Lake dual stance: replace-by-capability AND federate-into. Both are valid positions for Prism. Framing: capability-first, source-agnostic. | HUMAN CALL | D-1328 H2 "SIEM-replacement=DECIDED" |
| DC-002 | Hot credential reload AUTHORIZED for static-token sensors (Armis/Claroty). FM-002 no-restart recovery is the target. | HUMAN CALL (G2) | D-1327, D-1330 item (h) |
| DC-003 | Boot behavior: start-degraded + diagnostics. A sensor unavailable at startup puts that sensor in DEGRADED state and emits a diagnostic log entry; Prism starts and serves queries for available sensors. | HUMAN CALL | D-1327, D-1330 item (h) |
| DC-004 | Process memory budget ceiling RAISED. 512MB/200MB was per-laptop analyst-machine assumption. Central server deployment justifies configurable GB-range process budget (resolves caching H4 concern). The configurable-via-env approach is preserved; the default target changes from laptop to server-sized. | HUMAN CALL | D-1328 H4, D-1330 item (e) |
| DC-005 | Central deployment pivot: the vision matures from per-analyst stdio to central multi-tenant service. T13 demo UNAFFECTED (runs on current per-analyst path). Data-plane multi-tenant wiring is already correct. Gap is access layer only. | HUMAN CALL | D-1327 value-prop assessment, D-1330 item (d) |
| DC-006 | "Prism Satellite" name CONFIRMED by human. The satellite component name is settled. The relay/aggregator role noun for a satellite that also serves as a chaining upstream is TBD (day-2 terminology decision). | HUMAN CALL | D-1330 item (g) |
| DC-007 | Demo concurrency framing CORRECTED: async fan-out, not sequential, is the correct description. T13 runbook v1.4 and DEMO-SCOPE v1.6 updated accordingly (D-1329). | CORRECTION | D-1329 spec-sync |
| DC-008 | No circuit breakers in the per-query downstream path. Research-validated: below ~1 req/s, circuit breakers are inert or harmful; per-sensor availability cache is the correct substitute. | RESEARCH-VALIDATED | D-1327 research thread 1 §1.4 |
| DC-009 | BLOCKER-001 misdiagnosis adjudication: the hang was connect-timeout, not stale-KV-token. The reset_token_cache/get-token code path IS reachable via DTU health-check Fix B in the runbook. KV staleness theory is architecturally impossible (PluginKvStore is in-memory, fresh per `prism start`). AC-019 re-scoped; BC citation corrected BC-2.06.001 -> BC-2.01.005. Connect-timeout fix DEFERRED to S-RESILIENCE-FEDERATED-001 with concrete story anchor. | ARCHITECT ADJUDICATION | D-1326 |
| DC-010 | Federated search confirmed as CORE value prop. The brief rhetorically buries federation; the architecture, competitive landscape, and research all confirm it is the product. Brief reframe pending human sign-off (Section 5). | RESEARCH-VALIDATED + ARCHITECT ASSESSMENT | D-1327, D-1330 item (a) |
| DC-011 | Amazon Security Lake is the highest-value first lake connector. OCSF fit is real (pass-through normalization, not field mapping). Iceberg/Glue/Lake Formation access path. | RESEARCH-VALIDATED | D-1330 item (c), siem-securitylake research §1.1 |
| DC-012 | Demand-driven caching is a GENERALIZATION of existing Prism patterns, not an architectural rethink. RetentionCache CF + Retention Policy Engine. Three policy sources (detection-rule window, PrismQL RETAIN, config default). | ARCHITECT DESIGN | D-1328, a7ad2eedf80ba88d1 |
| DC-013 | Per-sensor TOML timeout schema: `connect_timeout_secs=5` and `request_timeout_secs=30` are correctly named and defaulted per research canon. Full schema in Section 3.6. | RESEARCH-VALIDATED | D-1327 research §1.7 |
| DC-014 | Satellite chaining architecture: hub-spoke + tree topology, outbound-only dial-home, per-hop mutual auth, deadline + partial-failure propagation through hops. | DESIGN CONFIRMED | D-1330 item (g) |

---

## Section 5 — Intended Spec Changes (To-Be) — Day-2 Execution Checklist

This section enumerates every document that requires amendment in the day-2 morph, per layer,
with the specific nature of each change. All items are PENDING and gated on brief sign-off
(item 5.1) for the foundational framing changes.

### 5.1 Product Brief (product-brief.md)
**REQUIRES EXPLICIT HUMAN SIGN-OFF BEFORE EXECUTION**

- [ ] Reframe headline: from "complete MSSP security operations platform" to the
  ephemeral federated query engine central framing (Section 2.1).
- [ ] Add value pillars: demand-driven caching as SIEM replacement by capability;
  SIEM/lake federation as source type; resilience (fail-fast + auto-recover) as a
  first-class concern.
- [ ] Reconcile per-analyst to central deployment: add central deployment mode
  description; retain per-analyst stdio as a valid deployment variant.
- [ ] Source taxonomy: "sensor" -> "source/connector" for the generic term; "sensor"
  retained as a subtype. Update In Scope item 1 and sensor adapter architecture.
- [ ] Memory budget: update from "512MB process / 200MB per-query" to
  "configurable, server-sized default; 512MB remains supported for laptop use."
- [ ] Add "Prism Satellite" as a named component in scope.

### 5.2 Domain Spec

**ubiquitous-language.md (new section or L2-INDEX update):**
- [ ] Source/Connector: generic term. Sensor: security-telemetry subtype. Define the
  boundary between security-telemetry normalization (OCSF) and native/structured/
  schema-on-read normalization (non-security sources).

**entities.md:**
- [ ] Add RetentionPolicy entity (policy source, window TTL, RETAIN directive, config default).
- [ ] Add CachedRecord entity (retention cache entry: OCSF record, TTL, policy ref,
  event_time index).
- [ ] Add AnalystIdentity entity (central deployment: per-connection identity, authN
  method, authZ scope).
- [ ] Add Satellite entity (topology node: trust anchor, endpoint, health state, parent ref,
  chain depth, loop-prevention ID set).

**invariants.md:**
- [ ] DI-017 amendment: single-process -> single-central-service. Add: "The stdio
  transport constrains to single-analyst per-process; the central transport enables
  multi-analyst per-process. The single-process model per transport session is preserved."
- [ ] New invariant DI-NEW-001: RetentionCache size cap (~200MB configurable; enforced
  by LRU eviction before insertion).
- [ ] New invariant DI-NEW-002: no-retention-without-policy: a record may only enter
  the RetentionCache if an active RetentionPolicy covers it.
- [ ] New invariant DI-NEW-003: OCSF-normalized-only in RetentionCache (raw sensor
  records are normalized before caching).
- [ ] New invariant DI-NEW-004: event_time drives TTL expiry (not wall-clock insertion
  time); out-of-order events do not extend TTL beyond the policy window.
- [ ] New invariant DI-NEW-005: partial-failure-degradation: a query with some sources
  unavailable returns partial results (HTTP 2xx equivalent) + structured per-source
  failure metadata. A source marked skip_unavailable=false failure fails the whole query.
- [ ] New invariant DI-NEW-006: cross-analyst isolation in central deployment: an
  analyst's per-connection session may not access another analyst's in-progress query
  state or confirmation tokens.

**failure-modes.md:**
- [ ] FM-002 amendment: add hot credential reload path for static-token sensors
  (Armis/Claroty). Recovery now includes: "Static-token sensors: hot credential reload
  via `reload_config` + new credential record — no restart required (G2 authorized,
  D-1330)." Remove the "restart Prism" suggestion for static-token sensors.
- [ ] New FM-013: Satellite connectivity loss. Impact: affected satellite's sources
  report DEGRADED; partial results propagate through the chain. Recovery: auto-reconnect
  with backoff; store-and-forward queue drains on reconnect.
- [ ] New FM-014: Boot-degraded mode (sensor unavailable at startup). Impact: affected
  sensor is DEGRADED; Prism starts successfully. Recovery: sensor auto-recovers when
  connectivity is restored; no restart required.

**architecture-concept.md:**
- [ ] "No data lake" statement in Why Ephemeral section: amend to
  "ephemeral by default, cache by demand." Add demand-driven caching paragraph.
- [ ] Comparison table: add RetentionCache row to Prism column.

**differentiators.md:**
- [ ] Add differentiator: "Resilient federated query — fail-fast per-sensor timeouts,
  partial-result semantics, and auto-recovery without restart. Analysts get results from
  healthy sources even when some sources are down; no manual intervention required."
- [ ] Add differentiator: "Demand-driven caching — SIEM-grade stateful detection without
  store-everything cost. Retention is driven by detection-rule windows and explicit analyst
  directives; data is cached exactly as long as needed and no longer."

**L2-INDEX.md:**
- [ ] Update domain summary to reflect central deployment, satellite topology,
  demand-driven caching, and source/connector taxonomy.
- [ ] Add entity cross-references for new entities (Section 5.2 above).
- [ ] Add invariant cross-references for DI-NEW-001 through DI-NEW-006.

### 5.3 PRD and NFR Catalog

**prd.md:**
- [ ] Section 1.2 (Competitive Differentiators): add "Resilient federated query" and
  "Demand-driven caching (SIEM by capability)" as listed differentiators.
- [ ] Section 1.4 (Out of Scope): remove "SIEM/log storage" — replaced by "store-everything
  log ingestion (Prism caches on demand, not persistently)." Retain: Prism is NOT a
  long-retention historical archive.
- [ ] Section 1.1 (Transport): note transport-selectable (stdio for per-analyst,
  HTTP/streamable for central deployment).
- [ ] BC Index: add new BC families once authored (caching, central authN/authZ, satellite,
  resilience).

**nfr-catalog.md:**
- [ ] New NFR-NNN: Partial-failure tolerance. Requirement: a query with k sources
  unavailable (where k < total sources, all skip_unavailable=true) MUST return results
  from available sources within normal SLO. Numerical target: 0 degradation in P50 latency
  vs all-healthy query of same scope. Validation: load test with N-1 sources healthy,
  1 source timing out.
- [ ] New NFR-NNN: Resilience SLO. Requirement: a sensor that was unavailable during query
  time must auto-recover within 60 seconds of the sensor becoming reachable again, without
  operator intervention. Validation: simulate sensor outage, restore connectivity, assert
  next query succeeds within 60s.
- [ ] New NFR-NNN: Central concurrency fairness. Requirement: in central deployment, no
  single analyst's queries may consume >50% of the fan-out concurrency budget
  (MAX_FANOUT_CONCURRENCY) for >10 consecutive seconds. Validation: concurrent load test.
- [ ] NFR-015 amendment: memory budget is now "configurable, default 512MB for laptop,
  recommended GB-range for central server." Update the numerical target to be a
  parameterized constraint, not a fixed 512MB.

> **Day-2 addendum (2026-06-25, side analysis — see Section 10).** The federated-query-language
> research identifies an expressiveness-vs-safety canon the current NFR set does not cover.
> Add the following (see Section 10 §10.5 gaps G-1, G-3):
> - [ ] New NFR: **Cross-source join cost guard.** A PrismQL query that joins two or more
>   sources where neither can execute the join natively MUST require a selective, key-based
>   join predicate; the planner rejects (or demands an explicit override + row cap) joins
>   estimated to fetch more than a bounded row count per side. This is the single largest
>   unguarded runaway risk once PrismQL can join (e.g.) CrowdStrike × Splunk. Distributed-join
>   strategy (broadcast / semi-join / central hash in DataFusion) to be specified.
> - [ ] New NFR: **Mandatory time-bound.** Every federated query MUST carry an effective time
>   predicate (explicit, or an injected default window). Generalizes the Security-Lake
>   `eventDay`/`time_dt` push-down guardrail (§3.5 Mode A) to all sources.
> - [ ] New NFR: **Default + maximum result limit** with limit-pushdown where the connector
>   capability descriptor supports it (§3.4 addendum).

> **RECONCILIATION NOTE 2026-06-27 (D-C3-1).** The cross-source join cost guard NFR item
> above was originally framed as "planner rejects (or demands an explicit override + row cap)
> joins estimated to fetch more than a bounded row count per side." The decision D-C3-1
> (human-confirmed 2026-06-27) supersedes the "planner rejects" framing with **cost-based
> degrade**: cross-source joins are ALLOWED; cost is bounded by row-caps, dynamic filtering,
> partitioned distribution, and resource-based abort after consumption. The override + row-cap
> mechanism is retained as specified but functions as an escape hatch above the default cap, not
> as the primary guard gate. At morph time, this NFR item should be amended to reflect the
> degrade posture. See `specs/day2-design-decisions/ADR-PROP-capability-descriptor-pushdown.md`
> D-C3-1 for the full rationale. Later-more-specific-artifact-wins (CLAUDE.md §Source-of-Truth
> Precedence).

### 5.4 Architecture and ADRs

**system-overview.md:**
- [ ] Deployment Model section: add central deployment mode alongside per-analyst stdio.
  Add architecture diagram for central mode (HTTP transport, analyst identity, shared state).
- [ ] DI-017 reference: update to reflect amended invariant (single-central-service).
- [ ] Add Prism Satellite section: deployment topology diagram, use cases, chaining model.

**ARCH-INDEX.md Subsystem Registry:**
- [ ] SS-23: RetentionCache (prism-retention crate or prism-operations + prism-storage
  extension). New subsystem.
- [ ] SS-24: Central Transport and AuthZ (HTTP/streamable transport, per-connection
  identity, authZ enforcement). New subsystem.
- [ ] SS-25: Satellite Mesh (satellite registration, dial-home transport, chaining,
  partial-failure relay). New subsystem.

**ADRs to author (all day-2):**
- [ ] ADR-047: RetentionCache design — StorageDomain::RetentionCache CF, Retention Policy
  Engine, three policy sources, event_time TTL, zstd compression, ~200MB cap.
- [ ] ADR-048: PrismQL RETAIN semantics — RETAIN <duration> [AS <name>] syntax, FROM
  cache. virtual table, naming rules, scope isolation.
- [ ] ADR-049: Detection window expansion — DI-029 amendment; cache-backed correlation
  window; interval-vs-window constraint relaxation.
- [ ] ADR-050: Central deployment topology — transport selection (stdio vs HTTP/streamable),
  deployment variants, migration path from per-analyst to central.
- [ ] ADR-051: Per-connection analyst identity model — identity propagation through the
  stack, authN mechanism, session isolation guarantees.
- [ ] ADR-052: Central credential custody design — credentials in the service, not on
  analyst machines; multi-analyst credential access model.
- [ ] ADR-053: Shared state access — alerts/cases visible to all analysts in the same
  org in central deployment; per-analyst state isolation preserved in per-analyst mode.
- [ ] ADR-054: Central service operational model — startup, health endpoints, scaling
  characteristics, graceful shutdown under multi-analyst load.

> **Day-2 addendum (2026-06-25, side analysis — see Section 10).** Candidate ADRs surfaced by
> the federated-query-language research (numbers to be allocated by architect):
> - [ ] ADR (TBD): **Connector capability-descriptor model** — per-connector pushdown profile;
>   PrismQL planner push-vs-local contract; DataFusion `supports_filters_pushdown` mapping.
> - [ ] ADR (TBD): **PrismQL pushdown & cross-source-join semantics** — predicate / projection /
>   aggregation / limit / sort pushdown matrix; distributed-join strategy; join guards.
> - [ ] ADR (TBD): **Mediated-schema model (GAV vs LAV)** — OCSF as the global mediated schema
>   for sensors (LAV-style) vs. direct source tables for non-security connectors (GAV-style);
>   schema-on-read normalization boundary.
> - [ ] ADR (TBD): **DataFusion Federation as the Satellite remote-execution substrate** —
>   bind §3.2 satellite mesh to remote-subplan delegation rather than bespoke transport.
> - [ ] ADR (TBD): **PrismQL entity/observable pivot** — `entity:<type>` search expanding to
>   all OCSF attributes that can hold the value (the one Query.io UX primitive worth copying).

### 5.5 Behavioral Contracts

**Existing BCs to amend:**
- [ ] BC-2.10.006: transport binding. Amend to transport-selectable: stdio for per-analyst
  mode, HTTP/streamable for central mode. Add precondition for transport selection at startup.
- [ ] BC-2.10.001: per-analyst per-connection. Amend to per-connection (transport-agnostic).
  Add postcondition: in central mode, each analyst connection has isolated session state.
- [ ] BC-2.04.002 and BC-2.04.011: credential access. Amend to cover central credential
  custody model. Add preconditions for central vs local credential backend.
- [ ] BC-2.05.002: per-connection audit identity. Amend to capture analyst identity from
  transport layer. In per-analyst mode: identity from process context. In central mode:
  identity from authN layer.

**New BC families to author (all day-2, gated on S-RESILIENCE-FEDERATED-001 status:ready):**
- [ ] BC family: per-sensor connectivity config (connect_timeout, request_timeout, retry
  schema, skip_unavailable flag interaction with partial-result semantics).
- [ ] BC family: boot-degraded mode (start-degraded, per-sensor DEGRADED state, diagnostic
  logging, auto-recovery on connectivity restore).
- [ ] BC family: sensor availability cache (health state TTL, fast-fail path, cache
  invalidation on connectivity restore).
- [ ] BC family: hot credential reload for static-token sensors (reload trigger, no-restart
  guarantee, FM-002 closure).
- [ ] BC family: RetentionCache (policy engine, RETAIN directive, FROM cache. virtual table,
  TTL expiry, size cap enforcement).
- [ ] BC family: central transport authN/authZ (per-connection identity, authZ scope
  enforcement, cross-analyst isolation).
- [ ] BC family: Satellite enrollment and chaining (dial-home protocol, per-hop mutual auth,
  partial-failure propagation through hops, loop prevention).

### 5.6 Meta-docs

**MEMORY.md (project memory files — stale entries):**
- [ ] `project_deployment_model.md`: update from "per-analyst MCP in Claude Code" to
  "per-analyst MCP (local) OR central multi-tenant service (server)."
- [ ] `project_core_architecture_insight.md`: "ephemeral federated query engine" is
  correct; add "demand-driven caching layer for detection-window retention."
- [ ] Any memory entry claiming "no data lake" must be updated to "ephemeral by default,
  cache by demand."

**CLAUDE.md:**
- [ ] Deployment model note: per-analyst stdio is one of two deployment modes. Update
  the "per-analyst MCP server in Claude Code (stdio transport)" description in
  Conventions section.
- [ ] Sensor terminology: add note that "sensor" refers to the security-telemetry subtype;
  "source/connector" is the generic term for any queryable source (day-2 terminology,
  pending ubiquitous-language finalization).
- [ ] DI-017 reference in invariants: if cited, update to amended form.

**Demo narrative (DEMO-SCOPE.md, T13 runbook):**
- [ ] No change for T13 demo (FROZEN). Post-T14, update the demo narrative to lead with
  the federated query value prop framing (Section 2.1).

---

## Section 6 — Day-2 Epic List and Sequencing

The following sequencing is recommended. Dependencies are noted. Human sign-off gates
are explicit.

```
T13 capstone demo (CURRENT — FROZEN)
  |
  v
T14 recording (post-T13)
  |
  v
Brief reframe sign-off (HUMAN GATE — blocks vision-threading)
  |
  +---> E-CACHE-DEMAND-001 P1 (can start in parallel with brief sign-off gate)
  |     detection-window retention; prism-operations/prism-storage/prism-core;
  |     no PrismQL dependency; ADR-047/049 authored first.
  |
  +---> S-RESILIENCE-FEDERATED-001 (can start in parallel post-T14)
  |     per-sensor TOML timeouts, boot-degraded, retry/backoff, skip_unavailable,
  |     availability cache, hot credential reload (G2), recover-without-restart;
  |     BC authorship MUST complete before status:ready.
  |     ADR: no new ADR needed (config schema design is in story BCs).
  |
  +---> (brief sign-off acquired) Vision-threading / spec evolution:
        business-analyst rewrite of domain spec (entities, invariants, ubiquitous language);
        PO amends PRD/NFR + existing BCs + new BC families;
        architect authors ADR-047..054 + ARCH-INDEX subsystem additions;
        story-writer decomposes new epics into stories + updates STORY-INDEX.
        |
        v
        E-CENTRAL-TRANSPORT-001 (ADR-050/051 authored first)
        HTTP/streamable transport, per-connection identity propagation.
        |
        v
        E-CENTRAL-AUTHZ-001 (ADR-051/052 authored first; can parallel E-CACHE-DEMAND-001 P2)
        analyst authN/authZ, per-connection capability enforcement, central credential custody.
        |
        v
        E-CENTRAL-OPS-001 (ADR-053/054 authored first)
        central credential custody, shared alert/case state, operational tooling.
        |
        v
        E-CACHE-DEMAND-001 P2 (PrismQL RETAIN + FROM cache.; gates on grammar stability)
        PrismQL RETAIN syntax, ADR-048 authored first.
        |
        v
        E-CACHE-DEMAND-001 P3 (config retention defaults)
        |
        v
        E-SATELLITE-MESH-001 (requires central transport as foundation; ADRs authored first)
        satellite registration, dial-home, chaining, partial-failure relay, loop prevention.
        |
        v
        E-LAKE-CONNECTOR-001 (requires central deployment; ADRs authored first)
        Amazon Security Lake first; generic Iceberg second; Splunk/Sentinel third.
```

**ADR dependency notes (each epic's ADRs must be authored before implementation dispatch):**
- E-CACHE-DEMAND-001 P1: ADR-047, ADR-049.
- E-CACHE-DEMAND-001 P2: ADR-048.
- E-CENTRAL-TRANSPORT-001: ADR-050.
- E-CENTRAL-AUTHZ-001: ADR-051, ADR-052.
- E-CENTRAL-OPS-001: ADR-053, ADR-054.
- E-SATELLITE-MESH-001: new ADRs (not yet numbered; architect to allocate from ADR-055+).
- E-LAKE-CONNECTOR-001: new ADRs (not yet numbered; architect to allocate).

**Each epic also requires BC authorship (PO) before story status can be set to ready
(Spec-First Gate S-7.01).**

---

## Section 7 — Open Naming and Terminology Decisions

| Item | Status | Decision needed from | Notes |
|------|--------|---------------------|-------|
| Generic term for all queryable sources | OPEN — "source/connector" proposed | business-analyst + PO (day-2) | "Sensor" retained for security-telemetry subtype. "source/connector" is proposed. Alternatives: "connector", "adapter", "source". Decision deferred to ubiquitous-language pass. |
| Relay/aggregator role noun in chaining | OPEN | business-analyst + PO (day-2) | A satellite that also relays for downstream satellites has a distinct role. Candidates: "relay", "aggregator", "hub". "Prism Satellite" is the general term; the relay-role noun is TBD. |
| "Prism Satellite" component name | CONFIRMED by human (D-1330) | None — settled | The component name for remote query executors is "Prism Satellite." |
| RETAIN vs CACHE directive keyword | OPEN | PO + architect (ADR-048) | "PrismQL RETAIN <duration> [AS <name>]" is the working proposal from D-1328. ADR-048 may adjust based on grammar analysis. |

---

## Section 8 — References

### Decisions Log (STATE.md)

| D-ID | Date | Summary |
|------|------|---------|
| D-1326 | 2026-06-24 | Wiring fix-burst; BLOCKER-001 misdiagnosis adjudication; root-cause = connect_timeout not stale-KV-token; AC-019 re-scoped; connect-timeout fix deferred to S-RESILIENCE-FEDERATED-001. |
| D-1327 | 2026-06-24 | Federated-search value-prop research + resilience investigation complete. Three parallel investigations: (A) architect resilience — Prism auto-recovers, fail-SLOW gap = no connect_timeout; (B) value-prop — federation architecturally central but rhetorically buried; G2 tension (FM-002 static-token restart); (C) research — circuit-breaker verdict: don't run one at Prism QPS; availability cache preferred; partial-result CCS semantics confirmed canonical; config schema validated. |
| D-1328 | 2026-06-24 | Demand-driven caching design complete; E-CACHE-DEMAND-001 scoped. Human decision H2: SIEM-replacement=DECIDED. ADR-047/048/049 needed. SS-23 new subsystem. Domain: RetentionPolicy+CachedRecord entities; DI-NEW-001..004. |
| D-1329 | 2026-06-24 | Track A spec-sync complete. S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.2->v1.3. BC-2.11.023 v1.1. T13 runbook v1.4. DEMO-SCOPE v1.6. S-RESILIENCE-FEDERATED-001 stub registered. |
| D-1330 | 2026-06-24 | Durability burst — lake-research committed. Day-2 vision backlog inventory captured (9 items a-i). CAPTURE ONLY. Demo target UNCHANGED. |

### Research Artifacts

| File | Date | Scope | Key findings |
|------|------|-------|-------------|
| `.factory/research/federated-search-architecture-2026-06-24.md` | 2026-06-24 | Resilience patterns + federated-search value prop. 39 cited sources. | Circuit breakers: don't use at Prism QPS. Partial-result CCS semantics canonical. Per-sensor schema with connect/request timeout + retry + skip_unavailable. Five value-prop statements. Query.AI is Prism's mirror competitor. |
| `.factory/research/siem-securitylake-datalake-federation-2026-06-24.md` | 2026-06-24 | SIEM/lake replace-vs-federate positioning + Amazon Security Lake technical. 73 cited sources. | Dual stance technically sound. Amazon Security Lake highest-value first connector (OCSF pass-through). Two access modes: query-subscriber and cache-hydrate. Microsoft Sentinel lake GA September 2025 (competitive development). |

### Architect Design Outputs (this session)

| Agent run ID | Topic | Key output |
|-------------|-------|-----------|
| a129e53b8894aab78 | Resilience investigation | Prism auto-recovers; fail-SLOW gap; G2 tension identified; in-story scope defined; follow-up epic scoped. |
| a6b8ace29eb8563e9 | Value-prop assessment | Federation architecturally central; brief lags thesis; G2 = FM-002 gap; brief reframe needs human sign-off. |
| acc2fb81c5abc0539 | Research on resilience patterns | Circuit-breaker verdict; CCS partial-result semantics; TOML schema; codebase flag: unwired timeout_secs overlay + rate_limit_hints unification. |
| a7ad2eedf80ba88d1 | Demand-driven caching design | RetentionCache CF; 3 policy sources; event_time TTL; zstd; ~200MB cap; E-CACHE-DEMAND-001 epic; ADR-047/048/049 needed. |

---

## Section 9 — D-1330 Inventory Coverage Verification

The D-1330 day-2 vision backlog captured 9 items (a) through (i). This section confirms
each item is covered in this document.

| D-1330 Item | Label | Covered in |
|-------------|-------|-----------|
| (a) | Federated search = CORE value prop; brief reframe pending sign-off | Section 2, Section 5.1 |
| (b) | Smart/demand-driven caching = SIEM replacement by capability; E-CACHE-DEMAND-001 | Section 3.3, Section 5.3, Section 6 |
| (c) | SIEM/Security Lake/Data Lake = replace-by-capability AND federate-into; Amazon Security Lake first connector | Section 3.5, Section 5.2, Section 6 |
| (d) | CENTRAL deployment pivot; data-plane multi-tenant already wired; gap = access layer; E-CENTRAL-TRANSPORT/AUTHZ/OPS-001; ADR-050..054 | Section 3.1, Section 5.4, Section 6 |
| (e) | RAISE memory ceiling; configurable GB-range; resolves H4 | Section 4 DC-004, Section 5.3 NFR-015 amendment |
| (f) | SOURCE TAXONOMY generalization: sensor -> source/connector; selection = "valuable to a security analyst"; multi-protocol connectors | Section 3.4, Section 5.1, Section 5.2, Section 7 |
| (g) | PRISM SATELLITE confirmed; satellite chaining (tree topology, multi-hop, per-hop mutual auth, partial-failure propagation); use cases enumerated | Section 3.2, Section 5.2, Section 5.5, Section 6 |
| (h) | RESILIENCE: S-RESILIENCE-FEDERATED-001 stub; per-sensor TOML timeouts; boot-degraded; retry/backoff; skip_unavailable; availability cache; hot credential reload (G2); recover-without-restart; unify timeout_secs overlay | Section 3.6, Section 4 DC-002/003/008/013, Section 5.2, Section 5.5, Section 6 |
| (i) | VISION THREADING / SPEC EVOLUTION (day-2 epic); brief -> domain-spec -> PRD/NFR -> architecture -> BCs -> stories; gated on brief sign-off | Section 5 (all subsections), Section 6 sequencing |

All 9 D-1330 items are covered. Human decisions ledger (Section 4) captures all 14 decisions
including the 5 explicit HUMAN CALL items (DC-001 through DC-006, excluding DC-007/008/009/010
which are corrections or research-validated). The 9-item D-1330 inventory + all human
decisions are accounted for.

---

## Section 10 — Federated Query-Language & Competitive Architecture Analysis (Query.io)

> **PROVENANCE — READ FIRST.** This section is a **2026-06-25 side-analysis addendum**, authored
> out-of-band from the 2026-06-24 capture session. It is NOT yet product-owner-ratified and does
> NOT carry a decision ledger entry. It is grounded in two deep-research passes run 2026-06-25:
> (A) a technical analysis of Query (query.ai / query.io) — their FSQL query language and federated
> data-mesh architecture; and (B) the general architecture of query languages in federated-search
> engines (Apache Calcite, Trino/Presto, Starburst, **Apache DataFusion + DataFusion Federation**,
> AWS Athena, Steampipe, Apollo GraphQL Federation, Elasticsearch/OpenSearch cross-cluster search).
> **These two research bodies are not yet committed as `.factory/research/` artifacts** — a future
> state-manager burst should persist them as `research/queryio-federated-search-2026-06-25.md` and
> `research/federated-query-language-patterns-2026-06-25.md` before any of the ADOPT/ENHANCE items
> below are dispatched. All §10 recommendations are PROPOSALS pending PO + architect adjudication.

### 10.1 How Query (query.io) works — query language

- **FSQL (Federated Search Query Language)** supersedes the earlier **UQL (Unified Query Language)**:
  "one query language across every source." FSQL is an **intermediate representation**, not a
  backend-executable — the mesh "translates FSQL to each source's native language at query time"
  (Splunk SPL, Azure KQL, Chronicle UDM, REST params). It is a **schema-aware DSL over OCSF**,
  referencing OCSF event classes / attributes / entities / enumerations rather than raw fields.
- **FSQL has a documented syntax surface** (corrected from the first-pass assessment after reading
  docs.query.ai directly): typed **sigil attribute selectors** — `%` string (`%email`), `@` decimal
  (`@cvss.base_score`), `#` integer/count (`#network.count`); a `SUMMARIZE` command (with `STATS` as
  an SPL-compatible alias) supporting `COUNT`/`AVG`/`SUM`; a `WITH` filter clause; `GROUP BY`; and a
  relative-time `SINCE <duration>` (e.g. `SINCE 1mo`). Example:
  `SUMMARIZE COUNT authentication WITH %email CONTAINS 'example.com' GROUP BY %email SINCE 1mo`.
  Docs also ship **Entities**, **Subqueries**, **FSQL-for-SPL-users**, **FSQL-for-KQL-users**, a
  **Cheat Sheet**, an **FSQL API**, and an **FAQL** page (FSQL FAQ — NOT a separate language; the
  NL→query path is CoPilot/agent-driven). What is STILL missing:
  a formal grammar/EBNF, a published type system, and cross-source join semantics. So Prism's
  rigor advantage holds, but narrows to *formal-grammar + planner + verification*, not "they have no
  language."
- **Three input modes:** natural language → LLM → FSQL (FAQL); structured FSQL; and **entity/observable
  search**, where an "entity" is an alias spanning multiple OCSF attributes (search one IP and it
  traverses every field that can hold an IP across all sources).
- **Rule translation IN:** Query converts existing **SPL, KQL, and Sigma** detection rules into FSQL
  as a migration on-ramp, and ships a **Hunting Library** of pre-built threat hunts (APT28, BRICKSTORM).

### 10.2 How Query (query.io) works — federation

- **"Centralize insights, not data."** Zero-ETL, no ingestion, no index, read-only API "bridges,"
  data stays at source. Query Data Model (QDM) **= OCSF**.
- **Static-schema connectors** (CrowdStrike, Okta, Defender…) are pre-mapped by Query's engineers;
  **dynamic-schema connectors** (Splunk, Snowflake, BigQuery, Chronicle, Sentinel) use a no-code
  **Configure-Schema** workflow that introspects the source, auto-discovers partitioning, samples
  data, and maps fields → OCSF.
- **Fan-out:** parse → resolve against QDM → select capable connectors → compile per-source native
  query → **execute in parallel** → normalize to OCSF (always populates `time`) → merge.
- **Federated Detections:** scheduled FSQL with explicit evaluation windows + thresholds; each run
  records **time range evaluated, source coverage, match counts**; supports **early termination** on
  threshold; emits a **finding with a replay link** that re-runs the exact window.
- **Weaknesses (Prism whitespace):** partial-failure handling, rate limits, pagination, caching, and
  performance are only *hinted* (via "source coverage" + "Test Connection"), never first-class.
  On-prem is a **reverse-proxy hack** (internet-reachable URL), not a mesh. Credentials are stored
  **centrally in the SaaS control plane**.

### 10.3 What Prism should ADOPT

**Strategic frame:** Prism already runs the engine the entire federation literature points to —
**DataFusion + Chumsky.** Query.io has no published grammar and no real planner story; Prism has a
formal PrismQL grammar, a DataFusion logical/physical planner, and Kani VPs. This inverts the usual
challenger position — Prism can be the *rigorous, formally-grounded* federated query engine.

| ID | Adopt | Rationale / prior art | Maps to |
|----|-------|----------------------|---------|
| ADOPT-1 | **Per-connector capability descriptors** (pushdown profile in TOML) | Trino SPI `applyFilter`; DataFusion `supports_filters_pushdown`; Athena federated connectors | §3.4; new ADR |
| ADOPT-2 | **Entity/observable pivot as a PrismQL primitive** (`entity:ip = …` expands across all OCSF attributes) | Query.io's single best UX idea | grammar epic; new ADR |
| ADOPT-3 | **CCS `skip_unavailable` partial-result semantics** (cite Elastic/OpenSearch CCS as canonical) | already implicit in BC-2.01.010 + S-RESILIENCE | §3.6 |
| ADOPT-4 | **Detection execution record + replay handle + early-termination** (time range, source coverage, match counts) | Query.io Federated Detections | E-CACHE-DEMAND-001 |
| ADOPT-5 | **Rule-translation IN: Sigma → PrismQL** migration on-ramp | Query.io SPL/KQL/Sigma → FSQL | candidate epic |
| ADOPT-6 | **Static- vs dynamic-schema source split + configure-schema workflow** | Query.io connector dichotomy | §3.5; E-LAKE-CONNECTOR |
| ADOPT-7 | **Safety canon in NFRs/BCs:** mandatory time predicate, default+max LIMIT, join guards | Calcite/Trino/CCS expressiveness-vs-safety patterns | §5.3 addendum |
| ADOPT-8 | **Name the mediation model (GAV vs LAV) in an ADR** | mediated-schema theory; OCSF as global schema | §5.4 addendum |

### 10.4 Where Prism should ENHANCE / beat Query.io

| Dimension | Query.io | Prism enhancement |
|-----------|----------|-------------------|
| Query-language rigor | No grammar, no formal semantics | PrismQL Chumsky grammar + DataFusion planner + Kani VPs → a **formally-verified** federated query language |
| Credentials | Stored centrally in SaaS control plane | **AI-opaque, reference-based (AD-017)** + MCP-native + prompt-injection-hardened output |
| Stateful detection | Pure ephemeral; **no** retention story | **Demand-driven RetentionCache (§3.3)** = SIEM-by-capability — a capability Query lacks |
| On-prem / OT | Reverse-proxy hack | **Prism Satellite dial-home mesh + multi-hop chaining (§3.2)**, mapped onto DataFusion-Federation remote-subplan execution |
| Resilience | Hinted, undocumented | First-class: `connect_timeout`, availability cache, no-circuit-breaker-at-low-QPS, recover-without-restart, hot credential reload |
| Language-level retention | None | **`RETAIN <dur> [AS name]` + `FROM cache.<name>`** — unique primitive |
| Cost-aware planning | Translate-and-pray | Capability + cost-based pushdown via DataFusion, with EXPLAIN showing pushdown |

### 10.5 Gaps in this vision doc the research exposes

| Gap | Description | Disposition |
|-----|-------------|-------------|
| G-1 | **Cross-source join strategy + join guards** — distributed-join + runaway-join risk once PrismQL joins across sources | §5.3 addendum NFR + new ADR |
| G-2 | **Per-connector capability descriptors** — the missing planner contract behind source/connector taxonomy | §3.4 addendum + new ADR |
| G-3 | **PrismQL pushdown contract** — what pushes to which connector class vs. runs centrally in DataFusion | new BC family + ADR |
| G-4 | **Entity/observable pivot construct** in PrismQL | new ADR (ADOPT-2) |
| G-5 | **GAV-vs-LAV mediation decision** as an explicit ADR | §5.4 addendum (ADOPT-8) |
| G-6 | **DataFusion Federation as satellite execution substrate** — bind §3.2 to the actual remote-subplan mechanism | §5.4 addendum |
| G-7 | **Sigma → PrismQL translation** epic — migration on-ramp | candidate epic (ADOPT-5) |

### 10.6 References (session research, 2026-06-25 — not yet committed)

| Research pass | Scope | Key primary sources |
|---------------|-------|---------------------|
| Query.io federated-search analysis | FSQL/UQL, OCSF QDM, static/dynamic connectors, federated detections, fan-out, decoupled control plane, residency, AI agents | query.ai/product, query.ai/federated-search, query.ai/federated-detections, docs.query.ai (QDM, Chronicle, MISP), ocsf.io |
| Federated query-language patterns | predicate/query pushdown, capability descriptors, GAV/LAV, schema-on-read, distributed joins, cost-based optimization w/ incomplete stats, partial results / fan-out / timeouts, safety guards | Calcite adapter docs, Trino connector SPI, Starburst pushdown, DataFusion custom-table-providers + datafusion-federation, Athena predicate-pushdown, Steampipe FDW, Apollo query-plans, OpenSearch cross-cluster-search, VLDB mediation, ocsf.io |
| Query deployment / credentials / UI / config + server-secret best practice | multi-tenant SaaS topology, Integration Control Plane, Query Agents, Security Data Pipelines/destinations, 2-role RBAC, envelope encryption + per-tenant DEK + secret broker + short-lived vending, multi-tenant web-UI security | docs.query.ai (product-architecture, team-management, configure-schema, security-and-privacy), query.ai/connectors, query.ai/product, AWS KMS envelope-encryption guidance, HashiCorp Vault / AWS Secrets Manager, OIDC/SAML SSO (Curity, FusionAuth), AWS SaaS multi-tenant guidance |

---

## Section 11 — Server-Deployment Pillars: Credentials, Configuration, UI

> **PROVENANCE.** 2026-06-25 side-analysis addendum (see Section 10 provenance block). These three
> pillars are required by the central-deployment pivot (§3.1, DC-005) but were not scoped in the
> 2026-06-24 capture. Grounded in the 2026-06-25 deployment/credential research pass + direct reads
> of docs.query.ai. All items are PROPOSALS pending PO + architect adjudication.

### 11.0 Why these three are now in scope

The central-deployment pivot (§3.1) explicitly scoped "the access layer only." But a production
central service forces three subsystems the per-analyst stdio model never needed:
1. **Credential storage** — the current reference-based, AI-opaque model resolves credentials on the
   *analyst's machine* (env/CLI/vault paths; `PluginKvStore` is in-memory, fresh per `prism start`).
   **This does not work server-side** (human directive, 2026-06-25): a central multi-tenant service
   must hold and resolve credentials for many orgs without an analyst laptop in the loop.
2. **Configuration management** — per-laptop TOML files + arc-swap hot-reload do not generalize to
   multi-tenant central config with RBAC, audit, and versioned change control.
3. **UI** — Prism currently has **no UI surface at all** (MCP-native, stdio). A central server needs
   at minimum an admin/ops console; whether it also needs an investigations console is a major
   scope decision (§11.3).

These partially overlap planned ADRs (ADR-052 central credential custody; E-CENTRAL-AUTHZ/OPS) but
each needs substantially more design than the capture allotted.

### 11.1 Credential storage (server-grade) — extends ADR-052

**Current (as-is):** reference-based, AI-opaque credentials (AD-017). A credential *reference*
(env var / CLI / vault path) is resolved locally on the analyst machine; the secret value never
transits AI/MCP context; newtypes give redacted `Debug`; `OrgSlug::new_unchecked` is audit-gated.

**Target (to-be) — server secret subsystem.** Preserve the reference-based, AI-opaque contract;
change only the *resolution backend* from "analyst-local" to "server secret backend." Design per
the secret-management canon surfaced in research:
- **DECISION (HUMAN-CONFIRMED 2026-06-25): hybrid — ship BOTH a first-party built-in encrypted secret
  store AND external-vault backends.** Prism is self-sufficient out of the box (built-in store, no
  external dependency required — critical for air-gap/on-prem/satellite) AND integrates with the
  customer's existing secret manager when they have one.
- **Pluggable `SecretBackend` trait** with implementations for (a) the **built-in self-hosted encrypted
  store** (default; envelope-encrypted, per-tenant DEK — see below) and (b) **external backends**:
  HashiCorp Vault, AWS Secrets Manager, GCP Secret Manager, Azure Key Vault. A credential reference
  resolves against the configured backend. (Dogfoods the existing reference model — references stay,
  resolver swaps; built-in vs external is per-tenant/per-deployment config.)
- **Envelope encryption + KMS key hierarchy** for the self-hosted store: per-credential blob
  encrypted under a **per-tenant Data Encryption Key (DEK)**; each DEK wrapped by a KMS master key.
  Per-`OrgId` DEK isolation means compromise of one org's DEK cannot decrypt another's — binds
  cryptographic isolation to the existing OrgId/OrgSlug multi-tenant boundary.
- **Secret broker / short-lived credential vending:** prefer OAuth client-credentials + refresh →
  short-lived access tokens over long-lived API keys; store refresh tokens, not bearer keys, where
  the source supports it. For static-token sensors (Armis/Claroty) that cannot, store the token under
  the per-tenant DEK and use the **hot credential reload** path already authorized in S-RESILIENCE
  (DC-002) for rotation-without-restart.
- **AI-opacity preserved & hardened (the enhancement over Query):** Query stores connector creds
  centrally too — but Prism keeps them **AI-opaque**: the broker injects the resolved secret into the
  adapter HTTP client at the I/O boundary, *never* into PrismQL results, MCP tool output, logs, or
  agent context. Output stays prompt-injection-hardened. This is a genuine trust differentiator for
  MSSP/regulated buyers.
- **Rotation, audit, scrubbing:** rotation API; every credential resolution audited with
  per-connection analyst identity (binds to ADR-051); no secrets in logs (extend existing redacted
  `Debug` discipline); backups encrypted.
- **Satellite/residency interaction (§3.2):** at OT/edge enclaves, credentials may be resolved
  *at the satellite* and never sent to central — only normalized, sanitized results transit upward.
  The `SecretBackend` is satellite-local in that topology.

**Epic/ADR (day-2):** flesh out **ADR-052** into the full secret subsystem; new BC family for
server credential custody (resolution, per-tenant DEK isolation, rotation, audit, AI-opacity
invariant). New subsystem candidate: SS-26 Secret Broker.

### 11.2 Configuration management (server-grade) — feeds E-CENTRAL-OPS-001

**Current (as-is):** per-laptop TOML spec files; arc-swap hot-reload (AD-007); config read via
`ArcSwap::load()`; in-flight queries hold a snapshot.

**Target (to-be):**
- **Central, per-tenant config store** for connector configs, schema mappings, detection/retention
  policies — scoped by OrgId, isolated per tenant.
- **Config API + RBAC** governing who may add/edit/delete connectors and policies (Team-Admin analog
  — see §11.4; routes to E-CENTRAL-AUTHZ-001).
- **Versioned change control + audit + rollback** (an explicit ENHANCEMENT — research confirms Query
  does *not* document config versioning). Every config mutation carries analyst identity + timestamp;
  rollback to known-good is supported. Candidate: Git-backed or DB-backed config history.
- **Declarative / GitOps config (enhancement over Query):** connector + detection + retention
  definitions as version-controlled TOML applied via API/CLI. Prism already ships built-in sensors as
  TOML specs (dogfood) — extend that to a declarative apply model with drift detection.
- **Multi-tenant hot-reload:** generalize arc-swap so a single tenant's config reloads without
  affecting other tenants or in-flight queries (snapshot semantics preserved per tenant).
- **Configure-schema workflow (adopt from Query, §3.5/§10.3 ADOPT-6):** introspect dynamic sources,
  map to OCSF, store the mapping *versioned*.

**Epic/ADR (day-2):** new ADR for central config store + versioning model; extend E-CENTRAL-OPS-001
to own config-plane vs data-plane separation; new BC family for config RBAC + audit + rollback.

### 11.3 Production-grade UI — multi-surface, multi-persona (HUMAN DIRECTIVE 2026-06-25)

**Current (as-is):** NO UI. Prism is MCP-native (Claude Code, stdio).

**Human directive (2026-06-25):** Prism stays **AI-native first (MCP, bring-your-own-agent)** AND
ALSO ships a **full browser experience with the AI capability built into the browser**, so the
product serves **multiple user personas** rather than only AI-forward analysts. The earlier
"admin-console-only / decline the investigations console" recommendation is **overridden**. A full
investigations console is now a **first-class day-2 deliverable**, not an optional U2.

> **Value-prop #5 amendment — DRAFTED replacement (HUMAN-APPROVED 2026-06-25, PO to ratify at
> brief-reframe).** Replace the current *"Federated search built for the analyst's agent, not for yet
> another browser tab"* with:
>
> > **5. Meets every analyst where they work — agent-native first, full browser console included.**
> > Prism is MCP-native so an analyst's AI agent can drive it directly (bring-your-own-agent), AND it
> > ships a full-fidelity investigations console with **built-in AI** for analysts who prefer a GUI —
> > plus a right-click browser extension for triage. One federated query engine, four surfaces
> > (S1–S4, §11.3), one set of guarantees: credentials the AI never sees, output hardened against
> > prompt injection, the same PrismQL underneath. The agent is the differentiator; the console makes
> > it usable by the whole SOC.
>
> Rationale: the original framing rejected the browser console as a category; the matured vision
> embraces it as an *additional* persona surface without surrendering the agent-native core.

**Four surfaces over ONE central backend (E-CENTRAL-TRANSPORT-001):**

| # | Surface | Persona | Notes |
|---|---------|---------|-------|
| S1 | **MCP-native (BYO agent)** | AI-forward analyst (Claude Code / own MCP client) | Existing thesis; primary for power users |
| S2 | **Full browser investigations console** | GUI / SOC-floor analyst | Search bar (PrismQL + NL), time picker, entity/event modes, results explorer, Summary-Insights dashboards, saved queries, detection findings + replay (source-coverage/replay per §10.3 ADOPT-4) |
| S3 | **Embedded AI inside the browser console** | GUI analyst who wants agentic help without a BYO MCP client | Prism-hosted copilot/agent (NL→PrismQL, guided investigation). **Architectural add: the central service now hosts an agent runtime server-side**, not only acts as an MCP server for BYO clients |
| S4 | **Browser extension (IOC right-click pivot)** | Tier-1 / triage analyst | Query-extension analog, improved: select any IOC on any page → federated PrismQL search against the central service; AI-opaque inline result preview; no session-staleness footgun |
| (U1) | **Admin/Ops console** | Admin / operator | Tenant+user mgmt, connector config + configure-schema wizard, credential rotation, health/observability, audit-log viewer |

**Key architectural implication of S3 (server-hosted agent).** Prism is no longer *only*
"bring-your-own-agent" — it also SHIPS an agent embedded in the console. The AI-opaque-credential
(AD-017) and prompt-injection-hardening guarantees apply to BOTH the BYO and the built-in agent. The
central service gains an LLM-agent orchestration layer (model routing, tool-call mediation,
output hardening). This is a notable scope addition — flag for architect.

**Cross-cutting UI requirements:**
- **Web-stack ADR — UI-D5 RESOLVED (HUMAN DECISION 2026-06-25 side-analysis): Option A — TypeScript
  SPA (React) + Rust (Axum/Tokio/DataFusion) backend.** Prism is a Rust workspace with no frontend;
  the choice was Rust-native (Leptos/Dioxus, in-language) vs TS SPA (React). The human selected the
  TS SPA after the trade-off walkthrough: the data-dense SOC console depends on the JS ecosystem's
  home turf — **AG Grid / TanStack Table+Virtual** (10k+ row grids), **ECharts/visx** (dashboards,
  MITRE heatmaps), **Cytoscape.js / sigma.js** (relationship graphs), and **Monaco** (PrismQL editor:
  highlight/autocomplete/lint, an awkward JS island under Rust-WASM). The all-Rust-shop pull toward
  Leptos was weighed and set aside because the Monaco island claws back the unified-language win
  exactly where the UI is most complex. **Shared types via OpenAPI→openapi-typescript codegen** from
  the Rust backend neutralize the type-boundary cost. Served over the E-CENTRAL-TRANSPORT-001 HTTP
  transport. Perf-critical client modules MAY be compiled to WASM. This decision is the input to the
  forthcoming web-stack ADR (architect to allocate the ADR number at morph time).
- **SSO (enhancement over Query):** OIDC/SAML from day one — research shows Query does *not* document
  SSO. Strong enterprise/MSSP differentiator.
- **Multi-tenant web-UI security canon:** tenant-context propagation via signed tokens, per-tenant
  isolation in every view, CSP/XSS/CSRF/clickjacking defenses, HTTP-only/SameSite cookies, session
  expiry/idle-timeout — all binding to per-connection analyst identity (ADR-051).
- **Browser-extension auth (improve on Query):** Query's extension piggybacks on an active console
  session and breaks when the session is stale (documented known issue). Prism's extension should use
  a proper token flow against the central service (no silent staleness failure).
- **Factory pipeline implication:** S2/S3/S4 activate the UI side of the factory prism has never
  exercised — ux-designer, design-system-bootstrap, accessibility-auditor, visual-reviewer,
  e2e-tester, ui-quality-gate, multi-variant-design, responsive-validation. Material process cost.

**Epics/ADRs (day-2):**
- E-UI-ADMIN-001 (U1 admin/ops console).
- E-UI-CONSOLE-001 (S2 full investigations console).
- E-UI-EMBEDDED-AI-001 (S3 server-hosted agent + in-console copilot).
- E-UI-EXTENSION-001 (S4 browser extension IOC pivot).
- ADRs: web-stack selection; SSO/identity-provider integration; UI↔transport binding;
  **server-hosted agent runtime** (model routing, tool mediation, output hardening) — net-new.

#### 11.3.1 S2 — Investigations console screen inventory (DRAFT)

Concrete screen set for E-UI-CONSOLE-001 (ux-designer to formalize; this is the scoping skeleton):

| Screen | Purpose | Key elements |
|--------|---------|--------------|
| **Investigation workspace** | The primary search surface | PrismQL editor (syntax highlight + OCSF-schema/entity autocomplete) · NL toggle (→ S3 agent) · time picker (`SINCE`/`BETWEEN`) · source selector (which connectors) · run/cancel |
| **Results explorer** | Read/triage federated results | OCSF-normalized result grid · **per-source coverage banner** (answered / degraded / timed-out — §3.6 partial-result semantics) · event-mode vs entity-mode toggle · row → raw-record drill-in · click observable → `FIND` pivot (§12.1) · export |
| **Entity / observable profile** | 360° view of an IP/user/host/hash | aggregated cross-source appearances + timeline · related entities · enrichment (threat-intel sources) |
| **Saved queries / query library** | Reuse | saved PrismQL · shared (tenant) vs personal · parameterized |
| **Detection rules** | Author/manage detections | rule list · rule editor (scope informed by axiathon, Section 14) · test/backtest run · enable/disable/version |
| **Findings / alerts** | Triage detection output | findings queue · finding detail with **replay link** (re-run the exact detection window, §10.3 ADOPT-4) · source-coverage record · disposition/notes |
| **Summary Insights (dashboards)** | Cross-source metrics | federated dashboards over OCSF events · time-scoped tiles |
| **Cache / retention browser** | Visibility into demand-driven cache | what's cached, TTL, policy source · browse `FROM cache.<name>` (§3.3) |
| **Sources / connectors** | Connector ops (admin-gated) | list + health · static vs dynamic (Section 13) · configure-schema wizard for dynamic · credential rotation (§11.1) |
| **Satellite / topology** | Mesh health (if satellites deployed) | tree view · per-hop status · degraded-subtree indicators (§3.2) |
| **Admin** (U1) | Tenant/user/audit | tenants · users + roles (fine-grained RBAC, §11.5 G-12) · audit-log viewer |

#### 11.3.2 S3 — Server-hosted agent runtime architecture (DRAFT)

The embedded-AI capability (S3) requires Prism to host an agent server-side, in addition to being an
MCP server for BYO clients (S1). Sketch for E-UI-EMBEDDED-AI-001 + the net-new agent-runtime ADR:

```
Browser console (S2) ──► Agent Orchestrator (server-side, per-tenant session)
                              │
              ┌───────────────┼─────────────────────┐
              ▼               ▼                     ▼
        Model Router    Tool Mediator         Output Hardener
        (LiteLLM-style; (exposes the SAME      (prompt-injection
         model-routing   MCP tool surface       defense — existing
         skill)          as S1 BYO clients)     Prism strength)
              │               │                     ▲
              ▼               ▼                     │
            LLM         PrismQL / federated  ───────┘
                        query engine (creds AI-opaque, §11.1)
```

- **Single tool contract, two consumers.** The hosted agent calls the *same* MCP tool surface BYO
  agents (S1) use — one contract, no fork. Credentials stay AI-opaque: the agent sees normalized
  OCSF results, never raw secrets.
- **Reuses existing strengths:** the `model-routing` skill (model selection/fallback) and Prism's
  prompt-injection-hardened output. **Net-new:** the orchestration loop + a per-tenant, isolated
  conversation store + tool-call audit bound to per-connection analyst identity (ADR-051).
- **LLM access configured server-side** via the §11.1 secret broker (dogfood — the agent's model API
  keys are themselves AI-opaque, broker-resolved). Per-tenant model + cost budgets.
- **Optional / deployment-gated (critical for OT/air-gap):** S3 can be disabled, or pointed at a
  local/on-prem model. Air-gapped and regulated deployments may run S1-only (BYO agent) with S3 off,
  or S3 against a self-hosted model. The federated-query core never depends on S3 being present.

> **Day-2 addendum (2026-06-26 side analysis — HUMAN-DECIDED ADOPT).** S3 adopts the **conversational-canvas**
> paradigm evaluated from the `aletheon_2` generative-UI spike (`/Users/jmagady/Dev/aletheon_2/spike`):
> the chat IS the interface; the embedded AI fetches via PrismQL over federated OCSF sources and
> **generates result widgets on the fly** (Vercel AI SDK `streamText`/`useChat` + Zod tool definitions
> that map to prism's MCP tool surface — the same "single tool contract, two consumers" S1+S3 idea above).
> The spike's widget-generation DSL (54 primitives) is a **UI-generation language ORTHOGONAL to PrismQL**
> (PrismQL fetches; the DSL renders) — no competition. Disposition: **ADOPT as S3 (enhanced + hardened)** —
> a distinct AI-native MODE complementing (not replacing) the structured S2 screens. **ENHANCE:** OCSF-aware
> primitives, multiple-option viz, and a **sandboxed/grammar-parsed expression evaluator (no `eval`/`Function()`)**.
> **DROP:** the spike's PostgreSQL+Kafka lake data layer (prism is ephemeral/federated; canvas state is
> session/local, never server-DB-persisted), OT-specific bits, and ui-tars (defer). **Security (prism-critical):**
> generative UI over attacker-influenceable OCSF data is a new prompt-injection surface — S3 MUST validate
> every widget schema against an allowlist, sandbox expression evaluation (the reason to revive an ANTLR4-style
> safe parser), keep credentials AI-opaque (broker-injected, AD-017), and apply prism output-hardening on both
> S1 and S3 paths; the widget-render layer sits AFTER the Output Hardener. Full disposition + phased adoption
> plan + candidate ADRs: `day2-ui-design/S3-conversational-canvas-disposition.md`. S3 canvas mockups (light+dark):
> `day2-ui-design/mockups/S3-01-ai-canvas.html`, `S3-02-ai-canvas-multioption.html`.

### 11.4 Deeper Query findings (from direct docs read, 2026-06-25)

- **Deployment is multi-tenant SaaS** organized as Organization → tenant → team; sign-up generates a
  tenant + team. **No documented on-prem edge/connector-runtime** — connectors are logical SaaS
  objects that call source APIs directly; on-prem sources need a customer-stood-up reverse proxy.
  → **Prism Satellite (§3.2) is a structural advantage Query lacks entirely.**
- **"Integration Control Plane"** is Query's term for the connector layer: "each integration
  translates your search terms into an efficient query, then normalizes the responses."
- **Query Agents** (mission-specific): Asset Information, Detection Finding Triage, File Hash, Network
  Activity, Threat Research, Vulnerability Intelligence. → maps to Prism's MCP-native agent posture;
  Prism's whole thesis is agent-native, so this is parity-with-differentiation.
- **Security Data Pipelines / Destinations** (newer Query feature): route normalized data OUT to
  Amazon S3, Azure Blob, GCS, Cribl Stream (HTTP), Splunk HEC. → Query has quietly added an *egress*
  capability; a Prism "normalized-result destination" (forward OCSF results to S3/Splunk-HEC) is a
  candidate day-2 connector-egress feature worth noting (complements RETAIN/cache).
- **RBAC is only two roles** (Team Admin / Team Member). → Prism can differentiate with finer-grained,
  connector/dataset-scoped RBAC + custom roles (research flags Query's coarse model as a gap).
- **Credential storage is undocumented** by Query (high-level only) — they do not publish KMS/Vault/
  envelope-encryption details. Prism documenting a rigorous, AI-opaque, per-tenant-DEK model is a
  credibility win.

### 11.5 New gaps (extends §10.5)

| Gap | Description | Disposition |
|-----|-------------|-------------|
| G-8 | **Server credential subsystem** — reference-model resolver backend, per-tenant DEK, secret broker, AI-opacity invariant | §11.1; flesh out ADR-052 + BC family + SS-26 |
| G-9 | **Central config store + versioning/audit/rollback + GitOps apply** | §11.2; new ADR + BC family; E-CENTRAL-OPS-001 |
| G-10 | **Multi-surface UI (S1 MCP + S2 console + S3 embedded AI + S4 extension + U1 admin) + server-hosted agent runtime + web-stack + SSO** | §11.3 (HUMAN DIRECTIVE 2026-06-25); E-UI-ADMIN/CONSOLE/EMBEDDED-AI/EXTENSION-001 + ADRs; value-prop #5 amendment |
| G-11 | **Connector egress / normalized-result destinations** (S3/Splunk-HEC analog) | §11.4; candidate connector feature, complements RETAIN |
| G-12 | **Finer-grained RBAC** (connector/dataset-scoped, custom roles) beyond 2-role | §11.4; E-CENTRAL-AUTHZ-001 |

---

## Section 12 — PrismQL Design Deliverables (entity-pivot grammar + join-guard NFR)

> **PROVENANCE.** 2026-06-25 side-analysis addendum, in response to the "keep iterating" request.
> These are DRAFT design sketches for architect/PO review, not ratified grammar or NFR text. They
> make §10.3 ADOPT-2 (entity pivot) and §10.5 G-1 / §5.3 (join guard) concrete.

### 12.1 PrismQL entity/observable pivot — grammar sketch (DRAFT)

**Goal:** adopt Query's single best UX primitive — search one observable and have it fan across every
OCSF attribute that can hold that value — but expressed natively in PrismQL (Chumsky grammar +
DataFusion planner), composable with full SQL.

**Two surfaces:**

(a) **Standalone `FIND` statement** (ergonomic shorthand):
```
FIND ip '10.0.0.5' SINCE 24h ACROSS crowdstrike, splunk
FIND user 'alice@example.com' BETWEEN '2026-06-01' AND '2026-06-25'
FIND hash 'e3b0c442...' SINCE 7d
```

(b) **`entity(...)` predicate function** (composes inside any PrismQL SELECT):
```
SELECT * FROM federated
WHERE entity('ip', '10.0.0.5')
  AND severity >= 'high'
SINCE 24h
```

**Grammar (EBNF-ish, DRAFT):**
```
entity_pivot   ::= "FIND" entity_type entity_value time_bound? source_scope?
entity_type    ::= "ip" | "user" | "host" | "domain" | "email" | "hash" | "cve" | IDENT
entity_value   ::= STRING
time_bound     ::= "SINCE" duration | "BETWEEN" timestamp "AND" timestamp
source_scope   ::= "ACROSS" ident ( "," ident )*
duration       ::= INT ( "s" | "m" | "h" | "d" | "w" | "mo" )      -- SINCE ergonomics (cf. FSQL)

entity_pred    ::= "entity" "(" STRING "," STRING ")"              -- embeddable in WHERE
```

**Semantics (planner contract):**
1. The **entity registry** maps each `entity_type` → an ordered set of OCSF attribute paths that can
   hold that observable, e.g.
   `ip → [src_endpoint.ip, dst_endpoint.ip, device.ip, src_endpoint.intermediate_ips, …]`.
2. The planner expands the pivot into a **disjunction of equality predicates** over those attribute
   paths: `entity('ip', X)` ⇒ `src_endpoint.ip = X OR dst_endpoint.ip = X OR device.ip = X …`.
3. Per connector, the **capability descriptor (§3.4 addendum / §10.3 ADOPT-1)** decides which of those
   attribute predicates push down natively; the rest are evaluated centrally in DataFusion after
   normalization.
4. **Mandatory time-bound:** if `SINCE`/`BETWEEN` is absent, the planner injects the default window
   (§5.3 mandatory-time-bound NFR). No unbounded observable sweep.
5. Fan-out (MAX_FANOUT_CONCURRENCY), normalize to OCSF, merge, partial-result metadata on degraded
   sources (§3.6).

**Open questions for architect:** keyword `FIND` vs `PIVOT` vs `SEARCH`; whether the entity registry
is config-driven TOML (preferred — dogfoods spec-driven model) or code; interaction with `RETAIN`
(can a `FIND` result be retained as `FROM cache.<name>`? — likely yes).

### 12.2 NFR-JOIN-GUARD — concrete specification (DRAFT)

**ID:** NFR-JOIN-GUARD (new; §5.3 / §10.5 G-1). **Category:** safety / cost-bounding.

**Requirement.** A PrismQL query containing a join across two or more *distinct sources* where neither
side can execute the join natively (different catalogs / no join-pushdown capability) MUST satisfy ALL:

1. **Equality-key requirement** — at least one equality predicate on a join key between the joined
   relations. Cross-source cross-products and non-equi-only cross-source joins are rejected at plan
   time with `E-QUERY-NNN` (cross-source-join-requires-key).
   **[SUPERSEDED by D-C3-1: cost-based DEGRADE, not hard-reject — see ADR-PROP-capability-descriptor-pushdown.md. All join shapes permitted; absence of equality key triggers the degrade path (cap + flag + cost disclosure), not a plan-time error. D-C3-3 explicitly allows outer/non-equi joins (central-only, no dynamic filter).]**
2. **Per-side selectivity** — each non-pushed side carries an effective time-bound plus ≥1 filterable
   attribute the planner estimates returns ≤ **N rows** (default **N = 100_000**, configurable per
   deployment). A side that cannot be bounded is rejected.
   **[SUPERSEDED by D-C3-1: cost-based DEGRADE, not hard-reject — see ADR-PROP-capability-descriptor-pushdown.md. An unbounded side triggers the row-cap + resource-abort degrade stack, not a plan-time rejection. The N row-count survives as the cap trigger, not a rejection threshold.]**
3. **Materialized-row budget** — total fetched rows across all sides ≤ **M** (default **M = 1_000_000**),
   enforced at *execution* by a monotonic row counter. Exceeding M aborts with `E-QUERY-NNN`
   (cross-source-join-budget-exceeded) and returns partial-result metadata (§3.6).
4. **EXPLAIN annotation** — the plan annotates: which joins run centrally vs pushed; estimated per-side
   cardinality; and the chosen distributed-join strategy.

**Distributed-join strategy mapping (DataFusion):**
- **Broadcast join** when the small side ≤ `broadcast_threshold` (default 10_000 rows).
- **Repartitioned hash join** otherwise.
- **Semi-join** when only existence is needed (`WHERE EXISTS` / `IN (subquery)` across sources) — fetch
  keys only, not full rows.

**Override.** An explicit `ALLOW LARGE JOIN` directive (or a per-deployment config grant) may raise
N/M for a specific query, but the equality-key and time-bound requirements remain non-negotiable —
the guard never permits an *unbounded* cross-source join.

**Validation.**
- Planner unit tests: reject missing-key cross-source join; reject unbounded side; accept bounded
  key-join.
  **[SUPERSEDED by D-C3-1: cost-based DEGRADE, not hard-reject — see ADR-PROP-capability-descriptor-pushdown.md. Validation should assert that missing-key / unbounded-side queries trigger the degrade path (row-cap enforcement + cost-disclosure in response envelope), not a plan-time E-QUERY-NNN error. At morph time, rewrite these test assertions to validate the degrade posture.]**
- Execution test: abort on budget breach with structured error + partial-result metadata.
- EXPLAIN test: assert strategy + cardinality annotations present.
- **Kani candidate:** monotonicity of the materialized-row counter (never under-counts → budget can't
  be silently exceeded) — fits Prism's existing VP/Kani discipline.

**Rationale.** Once PrismQL can join across sources (e.g. CrowdStrike × Splunk), the distributed-join
literature (Calcite/Trino/DataFusion) identifies cross-source joins as the dominant runaway-cost risk.
This NFR is the join-guard pattern the federated-query research calls mandatory; it operationalizes
the production-grade default (bound the cost, fail-fast with structured errors + partial results)
rather than allowing an unbounded fetch-both-sides join.

> **RECONCILIATION NOTE 2026-06-27 (D-C3-1 / D-C3-3).** Requirements #1 ("Equality-key requirement
> — cross-source cross-products and non-equi-only cross-source joins are **rejected at plan time**")
> and #2 ("A side that cannot be bounded is **rejected**") in the DRAFT above were written before
> the C3 architecture decisions were confirmed. Decision D-C3-1 (human-confirmed 2026-06-27)
> supersedes those requirements with **cost-based degrade**: cross-source joins of all shapes
> (inner equi, outer, non-equi, bare CROSS JOIN) are ALLOWED; cost is bounded by the guardrail
> stack described in `specs/day2-design-decisions/ADR-PROP-capability-descriptor-pushdown.md`
> (mandatory row-caps, DataFusion 50.x dynamic filter for inner equi, partitioned distribution,
> resource-based abort). The equality-key and selectivity requirements #1/#2 survive as **guard
> triggers** — their presence determines whether the cheaper inner-equi dynamic-filter path fires
> vs the central-execution fallback — but the response to absence is cap + flag + cost disclosure,
> not `E-QUERY-NNN` planner rejection. Decision D-C3-3 explicitly allows outer/non-equi joins
> (central-only, no dynamic filter). At morph time, this NFR should be amended to use "cost-based
> degrade" language throughout. Later-more-specific-artifact-wins (CLAUDE.md §Source-of-Truth
> Precedence).

### 12.3 PrismQL Ergonomics Parity Ledger — FSQL adopt / enhance / map (CONFIRMED 2026-06-25)

> **PROVENANCE.** 2026-06-25 side-analysis addendum. Dispositions CONFIRMED by human directive
> 2026-06-25: (i) the two SQL-colliding ergonomics are **mapped/dropped**, not adopted verbatim;
> (ii) SPL/KQL/Sigma migration is **docs-guides in day-2, automated translator deferred** to a
> follow-up epic. PrismQL's foundation remains SQL (DataFusion + Chumsky); "adopt" means *add as
> ergonomic sugar over SQL*, "enhance" means *do it better than FSQL*. DRAFT for architect/PO review.

**Adopt / enhance:**

| # | FSQL ergonomic | Disposition | How in PrismQL |
|---|----------------|-------------|----------------|
| E1 | `SINCE <dur>` relative time (`1mo`/`24h`/`7d`) | ADOPT + ENHANCE | `SINCE`/`LAST` sugar; add `BETWEEN`, timezones, named windows; wire to mandatory-time-bound NFR (§5.3) |
| E2 | `SUMMARIZE` aggregation (+ `STATS` SPL alias) | ENHANCE | SQL `GROUP BY`/aggregates canonical; add `SUMMARIZE`/`STATS` shorthand alias for SPL migrators |
| E3 | Entities / observable pivot | ADOPT + ENHANCE | `FIND` + `entity()` (§12.1) — composable in `WHERE`, richer than FSQL |
| E4 | Attribute selectors (dot-notation OCSF paths) | ADOPT | Native nested OCSF field access |
| E5 | Search filter operators (`CONTAINS`, comparisons) | ADOPT | Map `CONTAINS`/`IN`/comparators; keep SQL `LIKE` |
| E6 | Subqueries | ADOPT | SQL/DataFusion native |
| E7 | Dates & times | ADOPT + ENHANCE | Relative + absolute + timezone-aware; event-time vs wall-clock (ties to RetentionCache TTL) |
| E8 | NL → query (FAQL) | ADOPT + ENHANCE | NL→PrismQL via embedded agent (S3, §11.3) and BYO MCP (S1) — agent-native |
| E9 | SPL→ / KQL→ migration guides | ADOPT (docs) | Ship "PrismQL for SPL/KQL users" guides + cheat sheet in day-2 |
| E10 | Cheat Sheet / Investigation Patterns / Best Practices | ADOPT (docs) | Docs strategy |
| E11 | FSQL API (programmatic) | ADOPT | PrismQL HTTP API surface (in addition to MCP) |
| E12 | Hunting Library (prebuilt hunts) | ADOPT (content) | Curated PrismQL threat-hunt pack |
| E13 | Detection early-termination on threshold | ADOPT | Already §10.3 ADOPT-4 |

**Map / drop (CONFIRMED — not adopted verbatim):**

| # | FSQL ergonomic | Disposition | Rationale |
|---|----------------|-------------|-----------|
| M1 | `WITH <filter>` clause (FSQL filter keyword) | **MAP, not adopt** | `WITH` is SQL CTE; adopting FSQL's WITH-as-filter collides with PrismQL grammar. Keep SQL `WHERE`; surface FSQL's `WITH` only in the SPL/KQL migration guide (E9). |
| M2 | Typed sigil selectors (`%`/`@`/`#`) | **DROP syntax, keep intent** | PrismQL columns are already typed via the OCSF schema; sigils would fragment a SQL language. Type-aware selection retained without sigil syntax. |

**Deferred (follow-up epic, NOT day-2):**

| # | Item | Note |
|---|------|------|
| D1 | **Automated SPL/KQL/Sigma → PrismQL translator** | Query's marquee migration on-ramp. Day-2 ships only the docs guides (E9); the automated translator is a candidate follow-up epic (E-RULE-XLATE-001, unscheduled). Log so it is not lost. |

**Net position:** PrismQL achieves *ergonomic parity-or-better* with FSQL on every analyst-facing
affordance while keeping a real SQL foundation (DataFusion planner + Chumsky grammar + Kani VPs) —
i.e. Prism matches Query's usability without inheriting FSQL's lack of a formal grammar/type-system/
join semantics (§10.1).

### 12.4 PrismQL `SEQUENCE…THEN` sugar — grammar + desugaring to `MATCH_RECOGNIZE` (DRAFT)

> **PROVENANCE.** 2026-06-25 side-analysis addendum, in response to the "draft the sugar grammar"
> request. Makes §14.2.1 concrete. The readable surface most analysts write; desugars to the full
> SQL:2016 `MATCH_RECOGNIZE` operator (Phase A, in scope per HUMAN decision 2026-06-25). DRAFT for the
> sequence-sugar ADR.

**Design goal:** axiathon-grade readability on the surface; full RPR power underneath. Raw
`MATCH_RECOGNIZE` remains available as a power-user escape hatch for the long tail.

**Grammar (EBNF, DRAFT):**
```
detection      ::= "DETECT" ident sequence_block emit_clause? overlap_clause?
                 | sequence_block                          -- ad-hoc, no rule wrapper
sequence_block ::= "SEQUENCE" "BY" field_list within_clause? step then_step+
within_clause  ::= "WITHIN" duration                       -- overall maxspan
step           ::= "STEP" quant_var ":" predicate
then_step      ::= "THEN" gap_clause? negation? quant_var ":" predicate
               |   "THEN" gap_clause? "ANY" "OF" "[" alt_step ("," alt_step)* "]"
gap_clause     ::= "WITHIN" duration                       -- max gap from previous matched row
negation       ::= "NOT" | "WITHOUT"                       -- absence / non-event
quant_var      ::= pattern_var quantifier?
quantifier     ::= "+" | "*" | "?" | "{" int ("," int?)? "}"
alt_step       ::= pattern_var ":" predicate
emit_clause    ::= "EMIT" emit_item ("," emit_item)*
emit_item      ::= expr ("AS" ident)?
overlap_clause ::= "OVERLAP" ("ALLOWED" | "NONE")
predicate      ::= <PrismQL boolean expr over OCSF/native fields; MAY reference earlier
                    pattern vars, e.g.  host = b.host>
duration       ::= int ("s"|"m"|"h"|"d"|"w"|"mo")          -- shared with SINCE (§12.3 E1)
pattern_var    ::= ident                                    -- a, b, c, …
field_list     ::= field ("," field)*
```

**Desugaring rules:**

| `SEQUENCE…THEN` sugar | `MATCH_RECOGNIZE` target |
|---|---|
| `SEQUENCE BY k1, k2` | `PARTITION BY k1, k2` + implicit `ORDER BY <time-attr>` |
| `STEP a: P` / `THEN b: Q` … | `PATTERN (A B …)` + `DEFINE A AS P, B AS Q, …` |
| `a+` / `a*` / `a?` / `a{n,m}` | `PATTERN` quantifier on the variable |
| `ANY OF [b:…, c:…]` | `PATTERN ( … (B \| C) … )` + `DEFINE B…, C…` (alternation) |
| `THEN WITHIN 10m b: Q` | `DEFINE B AS Q AND B.<t> <= PREV(<t>) + 10m` (per-step gap) |
| `WITHIN 30m` (overall) | trailing constraint `LAST(<t>) - FIRST(<t>) <= 30m` (no standard `WITHIN`; expressed as a `DEFINE`/match predicate, per RPR research §1.3) |
| `NOT` / `WITHOUT b WITHIN W` | pattern exclusion `{- B -}` / non-event timeout — **Phase-A operator feature** (hardest case) |
| cross-step ref (`host = b.host`) | preserved verbatim as a `DEFINE` predicate referencing pattern var `B` (running semantics) |
| `EMIT x AS y` | `MEASURES x AS y` + `ONE ROW PER MATCH` |
| `OVERLAP NONE` / `ALLOWED` | `AFTER MATCH SKIP PAST LAST ROW` / `SKIP TO NEXT ROW` |

**Time-attribute resolution (multi-schema, §13.6):** the implicit `ORDER BY` binds to the source's
mapped time attribute — OCSF `time`/`event_time` for OCSF schemas, the configured timestamp column for
native schema-on-read sources. The sugar never hard-codes a field name.

**Worked examples:**

*(1) Fixed-step kill-chain* — the §14.2.1 `credential_theft` example (`STEP a THEN b THEN c`) →
`PATTERN (A B C)`.

*(2) Quantified + capture* — brute force (one-or-more failures) then success, same user+IP:
```
DETECT brute_then_success
  SEQUENCE BY user.name, src.ip WITHIN 5m
    STEP f+: auth.outcome = 'failure'
    THEN s:  auth.outcome = 'success'
  EMIT user.name, src.ip, count(f) AS failures, s.time AS broke_in
```
→ `PATTERN (F+ S)`, `DEFINE F AS outcome='failure', S AS outcome='success'`,
`MEASURES COUNT(F) AS failures, S.time AS broke_in`, `PARTITION BY user_name, src_ip`,
trailing `LAST(t)-FIRST(t) <= 5m`.

*(3) Non-event (absence)* — account created but NOT approved within 1h:
```
DETECT unapproved_account
  SEQUENCE BY account.uid WITHIN 1h
    STEP c:   activity = 'account.create'
    THEN NOT a: activity = 'account.approve'
  EMIT account.uid, c.time AS created
```
→ pattern exclusion / non-event timeout (Phase-A; the hardest desugaring — see open questions).

**Open questions (sequence-sugar ADR):** final keyword choices (`DETECT`/`SEQUENCE`/`STEP`/`THEN`/
`EMIT`/`OVERLAP`); overall-`WITHIN` as a hard match filter vs a `MEASURES`-surfaced value; exact
running-semantics for cross-step variable references; the `NOT`/`WITHOUT` non-event desugaring
(exclusion `{- … -}` vs timeout) — the single hardest piece; and confirming raw `MATCH_RECOGNIZE` is
exposed as the power-user escape hatch (recommended: yes).

---

### C8 PrismQL Deliverables — DECIDED 2026-06-27 (human)

> **PROVENANCE.** 2026-06-27 side-analysis addendum. Research basis:
> `research/prismql-deliverables-depth-2026-06-27.md` (Q1–Q6 depth pass). Capture artifact:
> `specs/day2-design-decisions/ADR-PROP-prismql-deliverables.md` (`do_not_execute: true`; real
> ADR numbers deferred to morph). Two items (D-C8-2 AS OF reproducibility; D-C8-3 OCSF
> version-binding) are DEFERRED pending targeted research; all others DECIDED 2026-06-27 (human).

**D-C8-1 PIPED SURFACE = SHIP IN DAY-2.** A KQL/PRQL-style `|`-piped sugar surface
(`source | where … | summarize … by … | order … | limit …` plus `FIND`/`entity()` and
`SEQUENCE…THEN` as security-domain pipe operators) DESUGARS to the SAME DataFusion logical plan
as the SQL surface — NOT a second engine. The SQL surface (canonical semantics, full
`MATCH_RECOGNIZE`) is preserved. Proven viable and Rust-native by PRQL and RunReveal `pql`
(both compile to SQL). Core pipe operators mirror KQL/PRQL: `where`, `summarize`/`stats`,
`project`, `extend`, `join`, `order`, `limit`. **MANDATORY:** expose "show desugared SQL /
EXPLAIN" (the DSL-debugging caveat — the desugaring must be inspectable, not a black box).
**HONEST COST:** the LSP server (D-C8-LEAN-3) carries AS MUCH learnability weight as the pipe
syntax — a pipe surface alone does NOT deliver KQL-grade approachability; and learnability
claims in the literature rest on design-rationale + anecdote, NOT controlled studies.

**D-C8-2 ENTITY-RESOLUTION AS OF REPRODUCIBILITY = DEFERRED (OQ-C8-ASOF).** A targeted
research pass is in flight (`research/prismql-asof-version-resolution-2026-06-27.md`) examining:
live-registry-snapshot (fresh, non-reproducible) vs frozen-registry-version (reproducible /
audit-grade) vs bitemporality (valid-time + transaction-time — may give BOTH from one model).
The research is also examining whether one as-of mechanism unifies D-C8-2 AND D-C8-3.
**SETTLED regardless:** weak-tier observables resolve by interval-containment as-of EVENT-TIME
(default, NOT query-time) against closed-open `[valid_from, valid_to)` intervals (SQL:2011
application-time semantics); composite identity key `(observable, namespace/site)` for
simultaneous multi-asset; strong-tier IDs bind exactly; tier policy lives in the registry with
an optional query-level `USING` override; the `AS OF <expr>` clause exists (default EVENT TIME).
The LIVE-vs-FROZEN reproducibility choice is the deferred part.

**D-C8-3 OCSF VERSION-BINDING = DEFERRED (OQ-C8-OCSFVER).** Same research pass.
Examining: version-agnostic-canonical-names + per-source-version schema-catalog reconciliation
(ergonomic) vs explicit `@ocsf:<ver>` pin (predictable/reproducible) vs catalog-version-pinning
(Iceberg-snapshot-id analog). **NOTE interaction with D-C8-2:** a forensic re-query may need
BOTH as-of-event-time entity resolution AND as-of-version schema binding.
**SETTLED regardless:** canonical OCSF field names are the identifiers; native fields reachable
via `native.<source>.<field>` namespace; retain-originals pattern (ASIM lineage — normalized
field never loses its raw source value); compatibility tiers (stable vs version-sensitive fields)
in the catalog. The version-agnostic-vs-explicit-pin + reproducibility choice is the deferred
part.

**LEANS CONFIRMED:**

- **ENTITY-PIVOT GRAMMAR (§12.1):** keep the TWO-SURFACE design — standalone `FIND`
  (ergonomic shorthand, SPL/Chronicle-search lineage) + `entity()` predicate composable in any
  `WHERE` (KQL/SQL lineage). **KEYWORD = `FIND`** (resolves §12.1 open "FIND vs PIVOT vs
  SEARCH" — `PIVOT` collides with SQL PIVOT; `SEARCH` is overloaded). Extended EBNF grammar
  (extends §12.1): adds `AS OF` clause (default EVENT TIME), `tier_hint` (`USING STRONG` /
  `STRONG, WEAK`), and `native.<source>.<field>` field_ref. Planner contract extends §12.1:
  interval-containment as-of EVENT TIME for weak-tier registry expansion; pushdown per C3
  capability descriptor (NOT reopened); mandatory time-bound injection; fan-out/normalize/
  partial-result. A `FIND` result may be RETAIN'd as `FROM cache.<name>` (confirm in morph).

- **MULTI-HOP PIVOT = FORWARD-COMPAT TARGET ONLY (NOT day-2).** Reserve a multi-hop
  path-pattern surface that lowers to SQL/PGQ `GRAPH_TABLE` (SQL:2023 part 16) semantics over
  an entity-graph view of the registry so day-2 single-hop grammar does NOT foreclose IP→asset
  →user→assets pivots later. DataFusion does not implement GRAPH_TABLE today [model-knowledge,
  not version-verified].

- **AUTHORING INTELLIGENCE = single LSP server** reused by THREE consumers: S2 Monaco console
  (via `monaco-languageclient` + WebSocket), CLI (via ariadne rendering), AND the NL→PrismQL
  agent's validate-repair loop (NL2KQL judge pattern). Use Chumsky `Rich` errors + `labelled`
  contexts throughout the grammar (Context7-verified: `Rich.span()` / `expected()` /
  `contexts()`, `labelled`/`labelled_with`, `try_map` for semantic validation); render via
  ariadne (CLI) + translate to LSP publishDiagnostics (Monaco). Four catalogs exposed to the
  server: grammar/operator metadata, OCSF-per-version schema, native-field schema, entity
  registry. Implement unknown-field→nearest-match suggestion + missing-time-bound→quick-fix
  (KQL quick-fix lineage). Schema-aware logic lives in the server, not Monaco. Planner-derived
  semantic diagnostics (DataFusion `LogicalPlanBuilder`/type errors) mapped back to source spans.

- **NL→PrismQL (§12.3 E8, agent-native):** reuse the SAME parser/planner diagnostics as the
  agent's validate-repair signal (NL2KQL two-stage judge: generate → validate against schema +
  parser feedback → repair). Schema-grounding + validation-before-execution + hallucinated-
  field-reject/repair are the guardrails; the existing mandatory-time-bound NFR + C3 cost-degrade
  are the cost/time safety net (no new agent-specific cost machinery). Prompt-injection defense
  out of scope here (agent-harness memory).

- **RECIPE FORMAT (§14.7, ties C6):** query text + Sigma-aligned metadata block (stable
  `rule_id`, semver `version`, `title`/`description`, `status`, `severity`, required
  `entities`+`data_sources`, ATT&CK `tags`, `false_positives`, `references`, author/dates).
  Store recipes in Git with semver (major = breaking entity/schema/logic change). Ship a CI
  harness running recipes against Arrow/Parquet fixtures via DataFusion with expected match /
  non-match assertions. Sigma import maps metadata 1:1 + logic via entity-registry / canonical
  fields; day-2 = docs-guided import (§12.3 E9), automated translator deferred
  (§12.3 D1, E-RULE-XLATE-001, ties C6). Ship Sigma→PrismQL EXAMPLES in recipe library now
  (per C6 L-C6-3 directive).

**DOWNSTREAM SAP-1 (flag, do not action):** desugar-decision / entity-resolution AS OF audit /
injected-window events may need BC-2.16.002 catalog rows (ties C3/C6 SAP-1 obligations).

**OPEN QUESTIONS:**
- OQ-C8-ASOF — entity-resolution AS OF reproducibility (D-C8-2 deferred)
- OQ-C8-OCSFVER — OCSF version-binding model (D-C8-3 deferred)
- OQ-C8-NATIVE-RESIDENCY — `native.<source>.<field>` interaction with §13.6 multi-schema
  descriptor + A7 per-field residency tags (a `native.*` ref may carry `raw` residency class)
- OQ-C8-RECIPE-SCHEMA — exact recipe-format Sigma-metadata schema + CI fixture shape
- OQ-C8-GRAPHTABLE-GRAMMAR — whether day-2 grammar needs a concrete placeholder for SQL/PGQ
  `GRAPH_TABLE` forward-compat, or whether "do not foreclose" is sufficient

**Proposed epic:** E-PRISMQL-GRAMMAR-001 (piped surface + FIND/entity() AS OF grammar +
LSP server + recipe format + CI harness; NL→PrismQL validate-repair loop).

---

## Section 13 — Static & Dynamic Connector Model — Scope

> **PROVENANCE.** 2026-06-25 side-analysis addendum. Deepens §3.4 (source/connector taxonomy),
> §3.5 (lake/SIEM), and §10.3 ADOPT-1/ADOPT-6. DRAFT for architect/PO/business-analyst review.

### 13.0 Two orthogonal axes

A "connector" varies along **two independent axes** — do not conflate them:
- **Schema axis: static vs dynamic** (is the source schema known ahead of time, or must it be
  introspected + mapped per deployment?).
- **Protocol axis: HTTP / SSH / WMI-WinRM / LDAP / SMB-file / SQL** (§3.4) — *orthogonal* to the
  schema axis. A SQL source can be dynamic-schema; an HTTP sensor can be static-schema.

This section scopes the **schema axis**; the protocol axis is tracked in §3.4.

### 13.1 Definitions (proposed; business-analyst + PO to ratify)

| Type | Definition | Examples | Normalization |
|------|------------|----------|---------------|
| **Static-schema connector** (today's "Sensor" subtype) | Schema known at spec/build time; Prism authors the OCSF mapping and capability descriptor; onboarding = credentials only | CrowdStrike, Cyberint, Claroty, Armis (current TOML spec-engine sensors) | Full OCSF mapping shipped by Prism |
| **Dynamic-schema connector** | Schema unknown ahead; introspected at onboarding; customer maps fields via a configure-schema workflow; mapping stored + versioned per tenant | Splunk, Sentinel, Snowflake, BigQuery, Amazon Security Lake, S3/Iceberg, custom SQL/LDAP sources | Security sources → OCSF (Security Lake = OCSF pass-through, §3.5); non-security structured → native schema-on-read |

### 13.2 Onboarding flow per type

- **Static:** supply credential reference (resolved via §11.1 secret broker) → connector live. No
  schema work. Capability descriptor (§10.3 ADOPT-1) ships with the connector spec.
- **Dynamic:** (1) supply credentials; (2) **schema introspection** — enumerate tables/indices/feeds
  and **auto-discover partitioning**; (3) **configure-schema workflow** (adopt from Query, §10.3
  ADOPT-6) maps native fields → OCSF event classes/attributes (or declares native schema-on-read for
  non-security data); (4) **preview/validate** against sample data; (5) **store the mapping
  versioned** (§11.2 config store, with audit + rollback). Capability descriptor is *derived* from the
  source class (e.g. SQL/Iceberg = rich pushdown; REST = filter-only) + what introspection reveals.

### 13.3 Scope dimensions to nail (each → BC/ADR)

1. **Schema-introspection subsystem** — per source class (SQL `information_schema`, Splunk
   `| metadata`, Iceberg/Glue catalog, OpenSearch `_mapping`, LDAP schema). What's in scope day-2:
   SQL + Iceberg/Glue (Security Lake) first; others follow.
2. **Configure-schema mapping model** — mapping artifact schema (source field → OCSF path, type
   coercion, enum normalization, nested-flatten); stored per-tenant, versioned, rollback-able.
3. **Capability-descriptor sourcing** — static = authored in spec; dynamic = inferred from source
   class + introspection; both drive PrismQL pushdown (§10.3 ADOPT-1) and the join-guard (§12.2).
4. **Normalization boundary** — security telemetry → OCSF; non-security structured → native
   schema-on-read; Security Lake → OCSF pass-through (near-zero mapping, §3.5).
5. **Mediation model (GAV vs LAV, §10.3 ADOPT-8)** — static connectors are GAV-ish (pre-defined
   view over a known source); dynamic connectors are LAV-ish (mapped into the OCSF global schema).
   Make explicit in the connector ADR.
6. **Trust/validation** — static: DTU clones (existing). Dynamic: configure-schema sample-preview +
   a mapping-validation gate (no DTU clone for customer-specific schemas).
7. **RBAC + audit** — only admin roles configure connectors/mappings; every mutation audited (§11.2).
8. **Hot-reload** — connector + mapping changes apply via arc-swap per-tenant without dropping
   in-flight queries (§11.2).

### 13.4 Day-2 epics / ADRs

- **E-CONNECTOR-DYNAMIC-001** — schema introspection + configure-schema workflow + versioned mapping
  store + capability-descriptor inference. (Sequences with / extends E-LAKE-CONNECTOR-001; Security
  Lake is the first dynamic connector.)
- **ADR (TBD): connector schema-axis model** — static vs dynamic definitions, onboarding flows,
  mapping artifact schema, GAV/LAV stance, capability-descriptor sourcing.
- BC families: dynamic-connector onboarding/introspection; configure-schema mapping + versioning;
  capability-descriptor contract.

### 13.5 Gaps (extends §10.5 / §11.5)

| Gap | Description | Disposition |
|-----|-------------|-------------|
| G-13 | **Schema-introspection subsystem** per source class | §13.3.1; E-CONNECTOR-DYNAMIC-001 |
| G-14 | **Versioned configure-schema mapping store** (per-tenant, rollback) | §13.3.2; ties to §11.2 config store |
| G-15 | **Dynamic-connector capability-descriptor inference** | §13.3.3; feeds pushdown + join-guard |

### 13.6 Multi-schema reality (authoritative — HUMAN-CONFIRMED 2026-06-25)

> **PROVENANCE.** 2026-06-25 side-analysis addendum, human-confirmed. This is the authoritative
> statement of Prism's schema model; §3.3, §3.4, §3.5, §10.3 ADOPT-8, §12.1, and Section 14 all
> defer to it.

**Prism is a multi-schema engine — NOT OCSF-only.** It contends with four schema families
simultaneously:

1. **OCSF — canonical normalization target for *security telemetry* (sensors).** OCSF is **itself
   versioned** (1.1, 1.3, …). Prism must support **multiple OCSF versions concurrently** + schema
   evolution (Amazon Security Lake = OCSF 1.1 native + 1.3 custom; axiathon already had multi-version
   OCSF support in `version.rs`).
2. **Native / structured schema-on-read** for **non-security connectors** — SQL databases, AD/LDAP,
   switch MAC tables, Excel/CSV, generic warehouses. These are **NOT** normalized to OCSF; they keep
   their native structured schema, queried schema-on-read (§3.4).
3. **Source-native query dialects/schemas** Prism translates to/from at pushdown — Chronicle UDM,
   Splunk fields, KQL/Sentinel, Snowflake/BigQuery tables — bridged by the configure-schema mapping
   on dynamic connectors (§13).
4. **protobuf shapes** alongside OCSF (project vision — sensors emit OCSF + protobuf).

**Consequences threaded through the design:**
- **Iceberg cold tier is multi-schema** — a set of tables keyed by (source-class, schema,
  schema-version): OCSF-vN tables + native schema-on-read tables; Iceberg schema-evolution absorbs
  OCSF version drift (§3.3 addendum).
- **PrismQL's type system is multi-schema-aware** — multiple schema namespaces + field aliasing +
  multi-version resolution (mirror axiathon's `type_system.rs` + `aliases.rs` + `version.rs`).
  Detection-as-query, the entity pivot (§12.1), and `MATCH_RECOGNIZE` (Section 14) all operate
  **across** OCSF *and* native schemas.
- **Mediation is hybrid/multi (§10.3 ADOPT-8)** — OCSF as a LAV global schema for security telemetry;
  native tables GAV-style for non-security sources. Not a single global schema.
- **The entity registry (§12.1) resolves an observable across schemas** — an IP lives at OCSF
  `src_endpoint.ip` *and* a SQL `source_ip` column *and* an AD attribute; the pivot spans all.
- **Capability descriptors carry each source's native schema + its mapping** to the query-time
  logical schema.

| Gap | Description | Disposition |
|-----|-------------|-------------|
| G-16 | **Multi-version OCSF support** (concurrent 1.1/1.3 + evolution) in type system + Iceberg tier | §13.6; ADR + BC |
| G-17 | **Multi-schema PrismQL type system** (namespaces, aliasing, schema-on-read for native sources) | §13.6; mirrors axiathon type_system/aliases/version |

> **Day-2 addendum (2026-06-26 side analysis). PROPOSED. do_not_execute.**
> OT/ICS telemetry is the **flagship native-schema-on-read case** in the multi-schema model above.
> OCSF has no first-class ICS/OT event classes or profiles as of 2026 (open proposal ocsf/ocsf-schema
> issue #1515 "Industrial Control System (ICS) Field Extensions" — not yet standardized; confirmed by
> independent factual check against schema.ocsf.io). OCSF Network Activity (class 4001) carries only
> the generic L3/4 envelope of an OT flow; OT protocol semantics (Modbus function codes, DNP3
> object-groups/points, S7 block/variable access, GOOSE dataset-refs/state-numbers, IEC-104 ASDU types)
> have no OCSF home today and MUST live in native structured tables queried schema-on-read — exactly
> §13.6 #2 (native/structured schema-on-read for non-OCSF sources). When/if ocsf#1515 standardizes,
> prism's multi-version OCSF support (G-16) absorbs ICS extensions without a native-schema migration.
> Until then OT is the canonical example that makes the multi-schema thesis concrete and non-optional.
> Cross-reference: §17.13 (OT protocol matrix, OCSF-OT verdict, safety constraints).

> **DECIDED 2026-06-27 (human) — C4 Dynamic-Schema / Configure-Schema Connectors (architecture
> decisions for §13 scope).** The design decisions for the §13 schema-axis model (static-declared
> TOML as acquisition default; discover-then-pin; two-hop type mapping; drift classification;
> WASM escape-hatch; boundary-normalization for all connectors; DataFusion integration with boot-time
> C3↔C4 reconciliation invariant) have been captured. See §3.4 C4 decision block for the full
> decision summary and `specs/day2-design-decisions/ADR-PROP-dynamic-schema-connectors.md` for the
> full capture artifact (`do_not_execute: true`). These decisions directly address §13.1–§13.5 (onboarding
> flow, scope dimensions, GAV/LAV mediation, WASM trust/validation) and feed E-CONNECTOR-DYNAMIC-001.
> SAP-1 downstream obligations (4 new BC-2.16.002 catalog rows) are flagged for morph time.

---

## Section 14 — Detection Engine & Rule Editor (HUMAN-CONFIRMED 2026-06-25)

> **PROVENANCE.** 2026-06-25 side-analysis addendum. Built from the axiathon exploration (the
> AxiQL-era predecessor) + Query Federated Detections + the MATCH_RECOGNIZE feasibility research, all
> 2026-06-25. Decisions HUMAN-CONFIRMED: detection-as-query (PrismQL, not a separate DSL); phased
> sequence support (B then A); correlation state in RocksDB/RetentionCache (Prism-native, no new
> datastore); rule editor on browser-console (S2) + MCP/agent (S1+S3) + CLI (no TUI); OT in scope.
> DRAFT for architect/PO review.

### 14.1 Core model — Detection-as-Query

**A detection IS a (scheduled) PrismQL query whose result rows are findings, wrapped in YAML rule
metadata.** One rigorous language for both ad-hoc investigation and detection — every query can
become a detection and vice versa. This beats Query (FSQL, no formal grammar) and axiathon (separate
AxD DSL) by keeping a single, formally-grounded language (DataFusion + Chumsky + Kani).

Rule metadata (adopt axiathon's schema): `id`, `name`, `description`, `severity`, `tags`,
`version` (semver), `status` (draft→review→testing→shadow→canary→production→deprecated), `mitre`
(tactic/technique), `schedule`, `window`, `group_by`, `false_positives`, `references`, `quality`
(test_coverage, fp_rate, mttd), `changelog`. The matching logic is **PrismQL**, not a bespoke
condition language.

### 14.2 Correlation type coverage (what PrismQL expresses)

| Correlation type | PrismQL mechanism | Phase |
|---|---|---|
| Single-event match | `WHERE` predicates (multi-schema, §13.6) | now |
| Threshold (`count > N within W group_by k`) | `GROUP BY … HAVING COUNT(*) > N` + time predicate | now |
| Distinct-count (spray / lateral across N) | `COUNT(DISTINCT …) HAVING > N` | now |
| Cross-source correlation | federated joins + **join-guard (§12.2)** | now |
| Statistical / baseline anomaly | window functions + `stddev`; long baseline → Iceberg cold tier (§3.3) or online-learned model (§15) | now (in-window) / later (on-demand ML, §15) |
| **Sequence / kill-chain** (`A then B then C`, `maxspan`, `$var` capture, Kleene `B+`, alternation, non-event) | **Phase A pulled forward (HUMAN-CONFIRMED 2026-06-25):** build the full NFA `MATCH_RECOGNIZE` operator from the start (full richness ≥ axiathon). Phase B (join/window rewrite) is retained only as an optimizer fast-path for simple fixed-step cases. Human surface is a **readable `SEQUENCE…THEN…WITHIN` sugar** that desugars to `MATCH_RECOGNIZE` (§14.2.1) | A now |
| Multi-stage DAG (alert-as-input) | rules consume prior findings within a run | adopt |
| Entity pivot | `FIND` / `entity()` (§12.1) | adopt |

**MATCH_RECOGNIZE facts (research-verified 2026-06-25):** SQL:2016 standard (R010/R020/R030;
`PATTERN`/`DEFINE`/`PARTITION BY`/`ORDER BY`/`MEASURES`/`AFTER MATCH SKIP`). Time windows via `DEFINE`
timestamp predicates — **no standard `WITHIN` keyword**. DataFusion: **parser supports it, core engine
does NOT execute it**, and the core team has signaled low appetite for in-core support → Prism builds
it as a **custom logical/physical operator** (Phase A). A join-rewrite (Microsoft "RPR Using Joins,"
5.4× speedup) validates the Phase-B path. Native vendors: Oracle, Snowflake, Trino, Flink SQL, Azure
Stream Analytics, DeltaStream.

#### 14.2.1 Keeping PrismQL human-friendly for sequences — layered surface (HUMAN-DIRECTED 2026-06-25)

Raw `MATCH_RECOGNIZE` is powerful but verbose and has a real learning curve (`PARTITION BY` /
`ORDER BY` / `MEASURES` / `PATTERN` / `DEFINE`). To keep PrismQL approachable, the **human surface is a
readable `SEQUENCE…THEN…WITHIN` sugar that desugars to `MATCH_RECOGNIZE`** — analysts write the
friendly form; the engine compiles to the full RPR operator. Best of both: axiathon-grade readability
on top, full SQL:2016 power underneath.

Friendly surface (what analysts write):
```
DETECT credential_theft
  SEQUENCE BY user.name WITHIN 30m
    STEP a: process.name = 'mimikatz.exe'
    THEN b: access.type = 'dump' AND resource = 'lsass'
    THEN c: file.path ENDS WITH '.kdbx'
  EMIT user.name, a.time AS started, c.time AS completed
```
Desugars to (what the engine runs):
```sql
SELECT * FROM events MATCH_RECOGNIZE (
  PARTITION BY user_name  ORDER BY event_time
  MEASURES A.event_time AS started, C.event_time AS completed
  PATTERN (A B C)
  DEFINE A AS A.process_name='mimikatz.exe',
         B AS B.access_type='dump' AND B.resource='lsass',
         C AS C.file_path LIKE '%.kdbx'
)
```
Learnability ladder: single-event + threshold + distinct-count read like ordinary SQL (trivial for
anyone with SQL); the `SEQUENCE…THEN` sugar is easy/moderate; raw `MATCH_RECOGNIZE` is reserved for
power users and for the long tail (Kleene quantifiers, alternation, overlap control). Reinforced by:
**NL→PrismQL via the embedded agent (S3)**, a **visual sequence builder in the S2 console** (adopt
axiathon's builder UX), autocomplete, and the **recipe library** (§14.7). Net: most analysts never
write raw RPR — the sugar + agent + recipes cover the common cases; the formal operator guarantees the
ceiling. **The full sugar grammar (EBNF) + desugaring rules to `MATCH_RECOGNIZE` are specified in
§12.4** (sequence-sugar ADR).

### 14.3 Federated/ephemeral adaptation — correlation over the cache

Axiathon assumed *ingested* data (Iceberg lake, inline-with-ingestion detection, RocksDB-persisted
state, backtest-from-local-storage). Prism is **federated/ephemeral**. Reconciliation:
- **Correlation/sequence detection runs over the RetentionCache window (§3.3), not a full lake.** The
  detection rule's window drives what's cached (hot RocksDB tier); the engine correlates over that
  bounded window. No store-everything.
- **Correlation state stays Prism-native:** short-term in-memory; durable correlation + risk/campaign
  state in **RocksDB / RetentionCache** — **NOT** a new datastore (the explore agent suggested
  PostgreSQL; rejected — Prism is RocksDB-native).
- **Backtesting** re-queries remote sources / Iceberg cold tier via the same federated path (not a
  local-lake replay).
- **Multi-schema:** detections operate across OCSF (versioned) + native schemas (§13.6).

> **Day-2 addendum (2026-06-26 side analysis — DECIDED 2026-06-26 (human)).** §14.3's
> RocksDB-native ruling stands for the EPHEMERAL CORRELATION/DETECTION PATH (short-lived,
> ephemeral, federated, append/scan-shaped, high-write, key-range). This is UNCHANGED.
>
> The 2026-06-26 decision introduces a FOUR-ENGINE storage taxonomy that reconciles §14.3 with
> the new control-plane requirements that emerged from the central-deployment pivot (§3.1, §11.1,
> §11.2, ADR-053). The four lanes:
>
> | Engine | Lane | Scope |
> |--------|------|-------|
> | **RocksDB** | Ephemeral/hot DATA-PLANE | Correlation & detection state, RetentionCache hot tier, continuous-operator window/sequence state, store-and-forward queues — central AND every Satellite. §14.3 preserved exactly. |
> | **Apache Iceberg** | Cold ANALYTIC tier | Long-baseline OCSF + native event/metadata, `RETAIN` multi-year, columnar/partition-pruned. Central / regional only. |
> | **PostgreSQL (BUNDLED in the central appliance, NEVER external/cloud)** | Relational CONTROL-PLANE | Case-management + alerts (ADR-053), central config store (§11.2), RBAC, audit log, tenant/user, identity/AS state, result-cache METADATA. **Central-only.** |
> | **SQLite (embedded)** | Satellite-local CONTROL-PLANE | Local config, enrollment/identity state, local policy + operational metadata. Satellite / edge only. |
>
> **Why NOT Iceberg for case-management:** Iceberg is OLAP/append-mostly. Table-level snapshot
> commits are expensive under concurrent multi-analyst writes; there are no point-lookup indexes;
> multi-row CAS requires app-built logic; row updates are expensive (merge-on-read or
> copy-on-write). Case-management is OLTP collaborative — wrong tool for that workload.
>
> **The §14.3 no-PostgreSQL ruling protects the EPHEMERAL CORRELATION PATH and stands.** The
> relational CONTROL-PLANE (case-mgmt/config/RBAC/audit/identity/AS) uses BUNDLED PostgreSQL
> at the central service only. This is a different workload class — a CONSCIOUS decision, not
> a silent reversal. Cross-ref: `ADR-PROP-storage-engine-taxonomy.md` §14.3 Reconciliation
> section and `research/central-deployment-access-layer-2026-06-26.md §Topic 4`.

### 14.4 Rule editor / authoring — surfaces (HUMAN-CONFIRMED: S2 + MCP + CLI; no TUI)

Adopt axiathon's authoring *concepts*, render on Prism's surfaces:
- **S2 browser console** — PrismQL rule editor (Monaco-style: highlight + OCSF/native-schema
  autocomplete + MITRE lookup), lifecycle-state management, **staged rollout** (shadow → canary →
  production, auto-rollback on FP spike), **backtest panel**, **exception/suppression manager** +
  **auto-tune** suggestions, **MITRE ATT&CK coverage dashboard** + gap analysis, community/**Sigma
  import**.
- **MCP / agent (S1 + S3)** — author/test/deploy rules via MCP tools; NL→rule via the embedded agent.
- **CLI** — `prism rules validate|test|deploy|shadow` for engineers + CI/CD (detection-as-code: Git,
  semver, validation gates, backtest TP/FP thresholds).
- **NOT TUI** — axiathon's vim TUI is a UX *reference* only, not a build target.

### 14.5 Alerting, findings & destinations

- **Alert model** (adopt axiathon): `Alert{id(UUIDv7), tenant, rule_id, severity, status, source_events,
  enrichment, assignee, …}`; statuses New→Acknowledged→InProgress→Resolved→Closed→FalsePositive;
  enrichment (threat-intel, asset, user, related alerts, MITRE).
- **Source-coverage record + replay link + early-termination** (adopt from Query Federated Detections
  + §10.3 ADOPT-4): each run records time-range, which sources answered/degraded, match counts;
  findings carry a replay handle.
- **Alert routing engine** (adopt axiathon): priority rules, AND/OR conditions, plugin channels, dedup,
  escalate-after.
- **Destinations** (adopt — Query "Alert Destinations" + axiathon channels): Slack/Teams/PagerDuty/
  email/webhook (notification) + **ServiceNow/Jira (ticketing)** + **Tines/webhook (SOAR)**. HMAC
  verification + interactive actions. **Distinct from response-actions** (reset/isolate) which stay
  **deferred behind feature flags** (project memory: writes gated).
- **Connector egress / Security Data Pipelines** (Query parity, §11.5 G-11): optionally forward
  normalized OCSF results to S3 / Splunk-HEC destinations — complements `RETAIN`/cache.

### 14.6 OT/ICS detection — IN SCOPE (HUMAN-CONFIRMED)

Prism serves OT (Claroty/Armis sensors + Purdue/OT satellite mesh §3.2), so OT detection is **in
scope**, contrary to the explore agent's IT-only assumption. Includes OT-relevant detection content
and (later) OT-protocol-aware semantics. OT detections run via satellites at the appropriate Purdue
layer; partial-result + degraded-subtree semantics (§3.2/§3.6) apply.

> **Day-2 addendum (2026-06-26 side analysis). PROPOSED. do_not_execute.**
> OT detection runs as detection-as-query over native-schema-on-read OT tables (OCSF has no first-class
> OT event classes as of 2026; open proposal ocsf#1515 covers this gap — see §13.6 / §17.13). OCSF
> Network Activity (class 4001) carries only the L3/4 envelope; OT protocol semantics (Modbus function
> codes, DNP3 object-groups/points, S7 block/variable access, GOOSE dataset-refs/state-numbers, IEC-104
> ASDU types) live in native structured tables queried schema-on-read. Dissection of OT packets runs via
> prism's native Spicy-style declarative engine on the OT-layer Satellite (§3.2) under strict passivity
> constraints: TAP preferred over SPAN, placement per Purdue layer / IEC 62443 zones-and-conduits,
> no injection onto the OT segment under any configuration. Encrypted-OT traffic (OPC-UA, MQTT-over-TLS)
> is metadata-only by DEFAULT; bounded decrypt opt-in at OT gateway chokepoints is a later,
> default-OFF, explicitly-authorized capability — never decrypting on the OT segment itself.
> Cross-references: §17.12 (native dissector engine), §17.13 (OT protocol matrix, safety constraints,
> OCSF-OT verdict), §17.14 (synthesis + decisions).

### 14.7 Content libraries (adopt)

- **PrismQL detection + threat-hunt recipe library** (Query "Recipes" + axiathon Hunting Library):
  curated, categorized, MITRE-tagged, **executable + backtested + version-controlled** (not doc
  snippets). Includes Sigma→PrismQL conversion examples (ties to deferred translator E-RULE-XLATE-001).
- **Prebuilt agent personas** (Query "Query Agents" parity): Triage, Threat-Research, Vuln-Intel,
  Asset-Info, File-Hash, Network-Activity — shipped as MCP/S3 agent skills; first-class, not bolt-on.

### 14.8 Epics / ADRs / gaps

- **E-DETECT-ENGINE-001** — detection-as-query model, rule metadata schema, threshold/distinct/
  cross-source/statistical via PrismQL, correlation over RetentionCache, multi-stage DAG.
- **E-DETECT-SEQUENCE-001** — full NFA `MATCH_RECOGNIZE` operator IN SCOPE FROM START (Phase A pulled
  forward, HUMAN-CONFIRMED 2026-06-25) + the `SEQUENCE…THEN…WITHIN` desugaring surface (§14.2.1). Phase
  B join/window rewrite retained only as an optimizer fast-path for simple fixed-step patterns.
- **E-DETECT-EDITOR-001** — S2 rule editor (lifecycle, staged rollout, backtest, exceptions, auto-tune,
  MITRE coverage, Sigma import); MCP + CLI authoring.
- **E-ALERT-ROUTING-001** — alert model, routing engine, notification + ticketing + SOAR destinations.
- **E-DETECT-RECIPES-001** — detection/hunt recipe library + agent personas.
- **ADRs:** detection-as-query semantics; sequence-detection phasing (join-rewrite → RPR operator);
  correlation-state-on-RocksDB; alert routing + destinations; OT detection topology.

| Gap | Description | Disposition |
|-----|-------------|-------------|
| G-18 | **`MATCH_RECOGNIZE` custom operator on DataFusion** (in scope now; core team won't add it, so Prism owns it) + **`SEQUENCE…THEN` sugar + desugaring** | E-DETECT-SEQUENCE-001 (Phase A pulled forward) |
| G-19 | **Backtesting over federated/cold-tier sources** (not local-lake replay) | E-DETECT-ENGINE-001 |
| G-20 | **Ticketing/SOAR destinations** (ServiceNow/Jira/Tines) beyond notifications | E-ALERT-ROUTING-001 |
| G-21 | **Prebuilt agent personas** library | E-DETECT-RECIPES-001 |

### 14.9 C6 Depth Decisions — DECIDED 2026-06-27 (human)

> **PROVENANCE.** 2026-06-27 side-analysis depth pass on the open implementation questions from §14.
> Research basis: `research/detection-engine-depth-2026-06-27.md` (six sonar-deep-research calls at
> `reasoning_effort=high`). Capture artifact: `specs/day2-design-decisions/ADR-PROP-detection-engine-depth.md`
> (`do_not_execute: true`; real ADR numbers deferred to morph). These decisions are ON TOP OF the
> §14 foundation — they add DEPTH/implementation decisions and do NOT re-litigate any settled §14 item.

**D-C6-1 — BACKTESTING POSTURE (gap G-19) = BOTH tiers, ALWAYS with coverage map (DECIDED 2026-06-27).**
Backtest against (a) the Iceberg cold tier deterministically — pin `snapshot-id + rule-version` →
reproducible point-in-time reads, no look-ahead bias; AND (b) remote sources best-effort —
current-state, retention-bounded, NON-deterministic (stated explicitly). MANDATORY coverage map per
`(source × time-slice)` labeled `{full / partial / none}` derived from `(source retention window) ∩
(connector onboarding date) ∩ (query error log) ∩ (schema-version availability)`. The single most
important correctness affordance: **distinguish "EVALUATED, no match" from "NO DATA to evaluate"** —
the entire surveyed prior art (Elastic, Chronicle, Panther, Splunk) is missing this; Prism builds it
from scratch. Mandatory time-bound + per-run volume ceiling + dry-run estimate (Panther-envelope,
generalized). Reuse §14.5 ADOPT-4 source-coverage-record + replay-link machinery for coverage map.
Cold-tier-only alternative rejected (most customer data lives in remote sources; non-determinism is
real but manageable; coverage map is the honesty mechanism).

**D-C6-2 — FALSE-POSITIVE HANDLING = RISK-BASED AGGREGATION (RBA) as default over hard suppression
(DECIDED 2026-06-27).** Prefer Splunk-RBA-style re-aggregation (noisy events accrue risk to an
entity; alert on aggregated risk → retains visibility into underlying events) to silent drop. Hard
suppression/exceptions delivered as **suppression-as-code**: every suppression is a versioned object
in the detection repo with MANDATORY justification + MANDATORY time-box expiry (no immortal
exceptions; expiry forces re-review) + fire-frequency/scope-breadth dashboard (catches over-broad
and stale suppressions). **Auto-tune emits SUGGESTIONS ONLY** — NEVER auto-applies, and NEVER
auto-disables a detection's evaluation without human sign-off. **HONEST CAVEAT stated plainly:**
"never silently mask a true positive" is NOT achievable as an absolute guarantee — the
production-grade posture is transparency + narrow-scope + mandatory-justification + time-boxed-expiry
+ fire-frequency-dashboards + RBA-over-suppression, NOT a proof.

**D-C6-3 — AUTO-ROLLBACK (staged-rollout FP-spike circuit-breaker, §14.4) = RESOLVED 2026-06-27
(human-confirmed). OQ-C6-AUTOROLLBACK CLOSED.** Research basis:
`research/detection-auto-rollback-depth-2026-06-27.md` (six sonar-deep-research calls at
`reasoning_effort=high`, Q1–Q6). Full control-loop design in
`specs/day2-design-decisions/ADR-PROP-detection-engine-depth.md §D-C6-3-RESOLVED`. Summary:

**ROLLBACK ACTION = DEMOTE-TO-SHADOW (auto).** Rule keeps EVALUATING (no coverage blind spot,
full audit trail); routing to analysts stops. Already-emitted findings annotated, not retracted.
FULL-DISABLE of evaluation = explicit HUMAN sign-off (SOAR coverage-reducing-action gate).
REVERT-TO-PRIOR-VERSION = one-click HUMAN action, not auto reflex. Governing rationale: if the
spike was a real attack, demote-to-shadow still detects and logs it — auto-full-disable would
silence the rule at the worst possible moment (error-asymmetry argument).

**CORROBORATION-MASTER-GATE (trip discriminator, the novel piece — no vendor ships it):**
(a) Corroborated by independent rules/threat-intel AND concentrated on few (high-value) entities
→ likely REAL ATTACK → DO NOT auto-rollback, ESCALATE to human.
(b) Uncorroborated + uniformly dispersed (high cardinality, no entity clustering) + sustained over
prolonged window with low incident yield → likely BROKEN → demote-to-shadow is safe.
(c) Ambiguous → HOLD-AND-ESCALATE, never auto-act.
The breaker trips on "persistent uncorroborated noise," NEVER on a "transient high-signal spike."

**CONTROL LOOP = per-tenant alert-rate circuit-breaker on the ROUTING path** (closed=route /
open=demote-routing / half-open=trial-route), coexisting with downstream notification throttles.
SIGNALS (zero-label real-time constraint — DDM/EDDM need labels and CANNOT drive the real-time
trip): primary = CUSUM on per-rule per-tenant alert-rate `λ_t` (`v`/`h` calibrated from the
shadow-mode baseline to a target ARL₀; ADWIN if baseline is strongly diurnal/non-stationary);
secondary = distinct-entity cardinality `U_t` + `N_t/U_t` duplicate-ratio (catches cardinality-
explosion AND duplicate-storm); analyst dispositions = DELAYED VALIDATION only (confirm/tune
trips after the fact). TRIP = N-of-M signal gating (volume-spike AND cardinality/duplicate
anomaly) + RELATIVE-to-shadow-baseline multiplier (+ absolute backstop cap) + minimum-window
count before the breaker may open (the Kayenta ≥50-sample discipline) + the CORROBORATION-MASTER-GATE.
HYSTERESIS/ANTI-FLAP = cool-down (`waitDurationInOpenState`) → half-open trial-route →
`consecutiveSuccessLimit` clean windows to re-close + exponential backoff on repeated trips +
HUMAN CONFIRMATION REQUIRED before re-promotion after any rollback (no auto re-promote loop).
SHARED PRIMITIVE: build the change-detector ONCE (CUSUM/ADWIN/Page-Hinkley/BOCPD) and point it
at C6 alert-volume stream OR C7 drift detection model-input/output stream — same statistical
family, different target (a real architectural economy).

**PROMOTION GATES:** shadow → canary AUTO-gates on metrics over a bake window (scope-limited,
safe to automate); canary → production = HUMAN sign-off (high-blast-radius widening to all
tenants — the Argo no-duration-pause analog). CANARY UNIT = TENANT (Prism is multi-tenant).
Gate thresholds carried in the §14.1 `quality` block (explicit-in-spec, not implicit operator
judgment).

**HONEST CAVEAT:** "never roll back a working rule" is a safety POSTURE (corroboration gate +
concentration test + sustained-window requirement + hold-and-escalate + demote-not-disable +
human re-promote gate), NOT an absolute guarantee — same honesty discipline as D-C6-2's "never
silently mask a TP." No SIEM ships integrated detection auto-rollback; Prism assembles it from
progressive-delivery (Kayenta/Flagger/Argo) + circuit-breaker (resilience4j/Hystrix) +
change-detection (CUSUM/ADWIN/BOCPD) + SOC-triage prior art, and owns the integration.

**Residual pre-implementation items (not blocking forks):** trip signal weighting / N-of-M
composition; relative-multiplier value + absolute backstop (§14.1 quality block vs per-deployment
config); CUSUM/ADWIN parameterization (v/h or window → target ARL₀; acceptable ARL₁ for security);
minimum-window count before breaker may open; corroboration data model (real-time computation of
corroboration + entity-concentration — the novel piece); cool-down/backoff/half-open-trial-size
schedule; per-tenant vs global breaker escalation rule; canary→production human-gate UX surface
(S2/MCP/CLI); shared change-detector primitive boundary with C7. (PIV-C6-RB-1..9 in capture artifact.)

**IMPLEMENTATION LEANS confirmed 2026-06-27 (human non-objection):**

- **MATCH_RECOGNIZE operator (keystone G-18):** build as `MatchRecognizeNode` (`UserDefinedLogicalNode`)
  + `MatchRecognizeExec` (custom `ExecutionPlan`) wrapping a Thompson-NFA INSTRUCTION-PROGRAM matcher
  (`MATCH`/`SPLIT`/`JUMP`/`CHECK`/`ACCEPT`; greedy/reluctant = `SPLIT` branch ordering; `{m,n}` =
  concat + optional tails). Declare required input distribution = hash-partition on `PARTITION BY` +
  required ordering = sort on `event_time` (planner inserts `RepartitionExec`/`SortExec`). `PlanProperties`
  `EmissionType::Incremental + Boundedness::Bounded` (batch-over-window); emit matches incrementally
  via `RecordBatchStreamAdapter`. COMPILE-TIME REJECTION of the SQL:2016 empty-match × `SKIP-TO-FIRST`
  infinite-loop and unbound-SKIP-target cases. Match-context (program-counter + per-variable ordered
  bindings + running aggregates + first/last index per variable) MUST be serializable from day one so
  the §17.7 continuous operator reuses the SAME matcher core. Optimizer fast-path (§14.2 Phase B) =
  a `RelationPlanner` rewrite detecting simple fixed-step `SEQUENCE` → self-join + window (MS "RPR
  Using Joins", 5.4×). MATCHER-REPRESENTATION lean = instruction-program (Trino model — easier
  reluctant-ordering + serialization than Flink pointer-graph); architect confirms against the pinned
  DataFusion version at morph (exact `ExtensionPlanner`/`QueryPlanner` method signatures flagged to
  re-verify — INCONCLUSIVE for pinned version; PIV-C6-1).

- **CONTINUOUS/INCREMENTAL RPR (§17.7 Phase 2):** wrap the SAME matcher core with a watermark +
  per-partition TIMER + incremental-checkpoint layer on RocksDB CFs. `WATCH…UNLESS` / absence /
  non-event = a per-partition event-time TIMER, NOT a relational anti-join (absence over an unbounded
  stream is undecidable without a deadline — confirms §17.14's dual-impl ruling: `AbsenceWindowNode`
  polled/batch + CEP timer continuous). SharedBuffer-equivalent with REFERENCE-COUNTING (dedup-and-free)
  + `event_time`-TTL window pruning bound memory. Window-state CF distinct from `detection_state` CF
  (§17.14). Checkpoint cadence (window-state vs durable detection_state vs ML ModelState) = architect
  open question (PIV-C6-2 + OQ-C6-3; Flink incremental-SSTable model is the template). Honest cost:
  the temporal/checkpoint/fault-tolerance layer is the real build; the matcher core is the cheap part
  (§17.7 self-identifies this as the single most expensive item).

- **SIGMA → PrismQL (deferred E-RULE-XLATE-001, feasibility confirmed):** a pySigma-style `Backend`
  + `ProcessingPipeline` targeting the OCSF taxonomy (extend the existing community Sigma→OCSF
  pipeline). Single-event + selection + threshold/distinct-count map cleanly to `WHERE`/`GROUP BY…HAVING`.
  STRATEGIC ALIGNMENT: Sigma CORRELATION rules (`event_count`/`value_count`/`temporal`/`temporal_ordered`)
  — thinly supported across backends — map onto Prism's `MATCH_RECOGNIZE`/`SEQUENCE` operator, so
  Prism is unusually well-positioned. Lossy edges (`base64offset`, `windash`, exotic regex dialects,
  class-spanning correlations) → translate with a FIDELITY REPORT flagging every non-losslessly-
  expressible modifier/condition (NEVER silent drop; the Q5 analog of the D-C6-1 coverage map). Keep
  deferred; ship Sigma→PrismQL EXAMPLES in the recipe library (§14.7) now to validate the mapping
  surface.

**OPEN QUESTIONS for architect/morph:** PIV-C6-1 (DataFusion ExtensionPlanner wiring confirm vs
pinned version); PIV-C6-2 (continuous-operator window-state↔detection_state CF isolation sufficiency
under shared checkpoint stream; §17.14 open #1); OQ-C6-3 (continuous-operator checkpoint cadence —
Flink incremental-SSTable model is the template, not a decision); OQ-C6-4 (backtest coverage-map
data model); OQ-C6-5 (Iceberg snapshot-retention policy for reproducible backtests — ties coldtier
+ C5); OQ-C6-6 (Sigma fidelity-report schema); PIV-C6-RB-1..9 (auto-rollback pre-implementation
items — not blocking forks; see D-C6-3 and capture artifact for full table). OQ-C6-AUTOROLLBACK =
CLOSED (resolved into D-C6-3 above, 2026-06-27).

**Downstream SAP-1 flag:** backtest-coverage, rollout-transition, suppression-fire, and auto-tune-
suggestion events likely need BC-2.16.002 catalog rows — flagged in capture artifact; NOT actioned here.

---

## Section 15 — On-Demand ML & Behavior Analytics (HUMAN-CONFIRMED 2026-06-25)

> **PROVENANCE.** 2026-06-25 side-analysis addendum, human-confirmed (both the on-demand ML framing
> and the online-learning tier). Ties into §2.4 (tradeoff), §3.3 (retention tiers), §13.6 (multi-schema),
> §14.2 (statistical-anomaly row). DRAFT for architect/PO review.

### 15.0 Core principle — ML obeys the Prism law

**On-demand/ephemeral ML : always-on/store-everything UEBA :: federated query : data lake.**
ML follows the same demand-driven, query-in-place, cache-by-demand law as the rest of Prism.
Anomaly/behavior detection is computed **on demand** over data Prism already fetches or has
demand-cached — NOT as an always-on ingestion-time pipeline over a permanent store-everything corpus.

### 15.1 On-demand computation model

- Anomaly/behavior scoring runs when a **query, detection, or investigation asks for it** — same
  trigger model as everything else in Prism.
- It computes over the **federated pull** (live source data) and/or the **RetentionCache window**
  (§3.3 hot tier).
- Cost-bounded: on-demand baseline pulls obey the mandatory time-bound + join-guard NFRs (§5.3, §12.2)
  — a baseline computation cannot trigger a runaway fetch.

### 15.2 Baselines from the demand-driven cache + scoped federated pull

"Is this anomalous vs. normal?" needs history. History comes from either:
- (a) the **RetentionCache cold tier (Iceberg, §3.3)** — now the natural home for multi-year,
  per-entity baseline data under a `RETAIN` policy; OR
- (b) an **on-demand scoped federated pull** of historical data from the source.
Baselines are **scoped to the entity/time-window the detection needs**, not a global model over all
data. Synthesis: *the cold Iceberg tier IS the baseline store; on-demand ML is its consumer.*

### 15.3 Three retention tiers — the model is a first-class tier (HUMAN-CONFIRMED)

Prism now has three demand-driven retention fidelities, each at a different cost/horizon/fidelity point:

| Tier | Store | Horizon | Fidelity | Replayable? |
|------|-------|---------|----------|-------------|
| Raw hot | RocksDB CF (§3.3) | seconds–hours/days | exact rows | yes |
| Raw cold | Iceberg/Parquet (§3.3) | days–years | exact rows | yes |
| **Model state** | learned model artifact | **longest** | **lossy summary** | **no** |

The **model is a bounded, per-tenant, policy-governed, OCSF/schema-scoped retention artifact** —
persisted like the cache, just far more compact and summarizing a longer horizon than the raw window.

### 15.4 Online / continuous learning — "the model is the memory" (HUMAN-CONFIRMED)

**If Prism supports online (incremental/streaming) learning, the model itself becomes the durable,
compact memory of everything it has ever seen — even though Prism never retained the raw data.**
Every on-demand data touch (query, detection window, federated pull) updates the model incrementally;
its parameters are a lossy, compressed summary of the whole exposed history. **The model retains what
the storage didn't** → long-horizon behavioral baselines at *model-sized* cost, not *data-sized* cost.
This **softens the §2.4 tradeoff**: long-memory baselines no longer require store-everything.

- **Single-pass / streaming algorithms only** (see-once, update, drop raw): streaming mean/variance,
  EWMA, online z-score, t-digest (quantiles), count-min / HyperLogLog (frequency/cardinality),
  reservoir sampling, online isolation-forest / half-space-trees, streaming clustering.
- **Coverage = what Prism touches**, broadenable via optional **scheduled baseline-refresh sampling
  pulls** (HUMAN-CONFIRMED: support both learn-from-what-we-touch AND scheduled sampling).

### 15.5 Honest limits + controls — IN SCOPE (HUMAN-CONFIRMED, not afterthoughts)

The online-learning advantage comes with limits that MUST be engineered for (production-grade):

| Limit | Control (in scope) |
|-------|--------------------|
| **Coverage gaps** — learns only what Prism surfaces | optional scheduled sampling pulls (§15.4) |
| **No replay / no re-derivation** once raw is gone | keep cold Iceberg tier for scopes needing exact replay; model + raw-retention are complementary |
| **Concept drift** — old normal must decay | EWMA/sliding decay; drift detection |
| **Adversarial poisoning** — slow "boiling-frog" retraining of malicious-as-normal | poisoning-resistance (bounded update rates, robust estimators, anomaly-gated learning) |
| **Explainability / audit** — detections fire off model *state* | **model versioning/snapshots**; a finding's replay link (§14.5) points at model state *as of* the decision |
| **Deliberate statefulness** — model is persistent (exception to ephemeral-by-default) | framed as demand-driven retention: bounded, per-tenant, policy-governed (same discipline as §3.3 cache) |
| **Verification** — online updates are order-dependent/stateful | bound + spec the update function; candidate VP/Kani targets for update-invariants |

### 15.6 Expressed as detection-as-query primitives

ML is exposed as **PrismQL functions/constructs**, not a separate subsystem the analyst sees — extends
the §14.2 statistical-anomaly row. Candidate primitives: `ANOMALY_SCORE(...)`, `RARITY(...)`,
`FIRST_SEEN(...)`, `BASELINE_DEVIATION(...)`, `PEER_OUTLIER(...)`, and a `PROFILE <entity> OVER <window>`
construct usable inside a `DETECT` (§14.1). Behavior/anomaly detection is just a richer PrismQL query.

### 15.7 Two model tiers + pluggable backends

- **Lightweight statistical** (z-score, EWMA, MAD, percentile, rarity, first-seen, peer-group) —
  cheap, in-window, **day-2 first** (largely already implied by §14.2).
- **Heavier learned models** (isolation forest, clustering, autoencoders, sequence models) — trained
  **on demand** over cache/federated pull, ephemeral/scoped, online-updated, cached as artifacts —
  **later tier**.
- **Pluggable model backends (built-in + external)** — mirror the §11.1 secret-store stance: ship
  first-party built-in models AND allow bring-your-own/external models. AI-opaque + per-tenant isolated.

### 15.8 Cross-cutting guarantees

Multi-schema (OCSF + native, §13.6; models per entity-class+schema) · OT-aware (behavior baselining at
the satellite/Purdue edge, §3.2) · AI-opaque (models never see raw creds) · prompt-injection-hardened ·
agent-native (S3 agent drives ML-assisted triage) · resilient/partial-result (scoring degrades when
sources down) · multi-tenant isolated.

### 15.9 Tradeoff softening (§2.4 refinement)

Long-horizon behavioral baselines are now available **three** ways — choose per use case:
1. `RETAIN` raw to the cold tier (exact, replayable, costlier);
2. **online-learn a model** (compact, long-memory, lossy, not replayable);
3. **federate into a lake** (someone else stored it, §3.5).
Online learning is the move that lets Prism claim long-memory UEBA **without** betraying the
ephemeral/federated thesis. (The §2.4 honest tradeoff should be updated by PO to reflect this.)

### 15.10 Epics / ADRs / gaps

- **E-ML-ONDEMAND-001** — on-demand anomaly/behavior scoring over federated pull + cache; baseline
  sourcing (cold tier / scoped pull); lightweight statistical tier.
- **E-ML-ONLINE-001** — online/incremental learning; model-as-retention-tier; drift/decay +
  poisoning-resistance + model snapshots/versioning; optional scheduled sampling.
- **E-ML-PRIMITIVES-001** — PrismQL ML functions/constructs (`ANOMALY_SCORE`, `PROFILE … OVER …`, etc.).
- **ADRs:** model-as-retention-tier; online-learning update semantics (drift/decay, poisoning
  resistance); model snapshot/versioning for replay/explainability; pluggable model backend
  (built-in + external); ML cost-bounding.

| Gap | Description | Disposition |
|-----|-------------|-------------|
| G-22 | **On-demand scoring engine** over federated pull + cache (lightweight statistical first) | E-ML-ONDEMAND-001 |
| G-23 | **Online-learning + model-as-retention-tier** (streaming algos, drift, poisoning, snapshots) | E-ML-ONLINE-001 |
| G-24 | **PrismQL ML primitives** (`ANOMALY_SCORE`/`PROFILE`/…) | E-ML-PRIMITIVES-001 |
| G-25 | **Pluggable model backends** (built-in + external, AI-opaque) | E-ML-ONLINE-001 |
| G-26 | **§2.4 tradeoff text update** (three-ways-to-long-baseline) | PO at brief-reframe |

---

### 15.11 DECIDED 2026-06-27 (human) — C7 Implementation Depth

> **PROVENANCE.** 2026-06-27 side-analysis depth pass on the open implementation questions from §15.
> Research basis: `research/ml-behavior-analytics-depth-2026-06-27.md` (six sonar-deep-research calls
> at `reasoning_effort=high`; 13 live crates.io version-verifications). Capture artifact:
> `specs/day2-design-decisions/ADR-PROP-ml-behavior-analytics-depth.md` (`do_not_execute: true`; real
> ADR numbers deferred to morph). These decisions are ON TOP OF the §15 foundation — they add
> DEPTH/implementation decisions and do NOT re-litigate any settled §15 item.

**D-C7-1 — SATELLITE/EDGE MERGEABILITY POSTURE = DEFER-TO-TEST empirical bake-off (DECIDED
2026-06-27).** PRIMARY lean: restrict satellite/Purdue-edge behavioral baselining to the
cleanly-MERGEABLE primitives only — Welford/CGL mean-variance (exact associative reduce-tree via
CGL formula), DDSketch quantiles (exact — add aligned bucket counts), count-min frequency (exact —
element-wise add), HLL cardinality (exact — register-wise max). Non-mergeable primitives (EWMA,
reservoir sampling, streaming clustering, streaming iForest) run CENTRAL-ONLY in the primary
posture. ALTERNATIVE TO TEST empirically at edge-ML build time: allow non-mergeable primitives at
the edge merged via documented approximations (time-aware re-EWMA, weighted re-sampling for
reservoir) — measure the approximation error vs the exact-mergeable approach and decide then. The
binding choice is DEFERRED to the implementation-time measurement (OQ-C7-1). Capture artifact
records both; primary=mergeable-only holds until the bake-off.

**D-C7-2 — MODEL BACKENDS = COMMIT THE FULL PLUGGABLE-BACKEND SET IN DAY-2 (DECIDED 2026-06-27).**
Define a first-party AI-opaque `ModelBackend` trait (`load/infer/train`; backend never receives
raw credentials — mirrors AD-017 + §11.1 BYO secret-store stance; per-tenant isolated). Backends:
(a) **first-party statistical sketches** (statistical tier, Welford/CGL/EWMA/count-min first-party
+ sketches-ddsketch 0.4.0 + hyperloglogplus 0.4.1 + tdigests 1.0.1 — day-2-first);
(b) **candle-core 0.11.0** (built-in learned tier; explicit "serverless inference" goal; healthy
2026-06-26; ~2.1M downloads);
(c) **ort 2.0.0-rc.12** (TRUSTED BYO ONNX; process-isolated — still RC, pin exactly, budget for
API churn);
(d) **wasmtime 46.0.1 Component-Model WASM plugins** (UNTRUSTED BYO; capability-restricted,
per-tenant WASM instance — REUSE the C4 WASM sandbox pattern from
ADR-PROP-dynamic-schema-connectors.md D-C4-3; healthy 2026-06-24; ~6.7M downloads);
(e) **tract-onnx 0.23.3** (satellite/Purdue-edge tiny runtime; pure-Rust; no C++ dep; healthy
2026-06-19). HONEST COSTS: `ort` still RC (2.0.0-rc.12, no stable 2.0 as of 2026-06-27) — pin
exactly + budget for churn; WASM-ML has a performance tax (CPU small/medium models fine; SIMD/GPU
not mature in WASM); this is the LARGER-BUILD option chosen consciously over statistical+candle-first.
Sequencing within the backend set: statistical + candle first; ort + WASM-BYO later (within day-2
scope); tract for satellite edge context. OQ-C7-4: whether the C4 connector WASM grant configuration
is appropriate for inference-weight workloads or needs a separate model-plugin sandbox config.

**D-C7-3 — DRIFT / DECAY / POISONING = DUAL-RATE + QUARANTINE design (DECIDED 2026-06-27).**
Day-2 baseline math = robust estimators (median/MAD) + bounded per-window update rate (cap how
far one update can move the baseline) + anomaly-gated learning (do NOT update model from data
already flagged anomalous). Drift detectors MUST-BUILD in Rust (entire ecosystem is Python —
River/scikit-multiflow/Frouros; `neural-drift` Rust crate does not document its algorithms): ADWIN
first (self-tuning window doubles as the decay/forgetting mechanism) + Page-Hinkley second. Dual-rate
+ quarantine design: a fast model flags; a slow model only ABSORBS drift that PERSISTS past a
quarantine window OR is human/S3-agent-confirmed (§15.8 agent-native). Attacker must sustain
shifted behavior across both the slow window AND quarantine to poison the slow model. OPEN QUESTIONS
(OQ-C7-3): quarantine window length, fast/slow-model promotion threshold, per-tenant policy defaults.
**HONEST CAVEAT STATED PLAINLY:** the anomaly-gated-learning vs concept-drift tension is genuinely
unsolved in general. Dual-rate + quarantine SHIFTS attacker cost; it does NOT eliminate boiling-frog
risk. The spec must say this openly — do NOT imply a guarantee.

**D-C7-4 — MODEL REPLAY / EXPLAINABILITY = PER-UPDATE CHANGELOG + PERIODIC MATERIALIZATION
(DECIDED 2026-06-27).** Append every model update as a delta to a per-tenant changelog; materialize
consolidated snapshots periodically; a finding's replay link (§14.5) references
**(materialization-id + changelog-offset)** → enables replaying EVERY update, not just
per-detection state. Rejected alternative: content-addressed per-detection snapshots (lighter —
SHA-256-hash the serialized `ModelState` per detection fire; dedup via hash; simpler reference) —
this gives per-detection auditability but NOT per-update auditability; the human chose the heavier
mechanism for full per-update auditability. STORAGE = RocksDB (the §3.3 tier); bincode-serialized
`ModelState` envelope (`schema_version, model_type, tenant_id [newtype/redacted-Debug],
schema_scope, payload: Vec<u8>`); `#[non_exhaustive]`; per-tenant via CF (few large tenants) +
**key-prefix** (`tenant_id:schema_scope:entity_class:model_type:...`) within shared CFs for the
long tail — DO NOT map CF-per-tenant for the long tail (RocksDB degrades at thousands of CFs).
HONEST COST: heavier than content-addressed (more storage + changelog/materialization machinery);
changelog retention policy required (keep N materializations + all offsets since oldest retained;
GC older entries). OQ-C7-6: retention policy parameters.

**IMPLEMENTATION LEANS confirmed 2026-06-27 (human non-objection):**

- **STATISTICAL TOOLKIT (day-2-first, §15.7):** first-party Welford+CGL + EWMA + online z-score
  (+ robust median/MAD variant); depend on healthy crates: sketches-ddsketch 0.4.0 (rel-error
  quantiles, fully mergeable), tdigests 1.0.1 (tail-accurate, if needed alongside DDSketch),
  hyperloglogplus 0.4.1 (cardinality, high adoption); first-party count-min with conservative
  update (all count-min crates stale). Vendor-vs-build line: DEPEND on healthy mergeable-sketch
  crates; FIRST-PARTY the math where crates are stale (Welford/CGL/EWMA/count-min/drift-detectors).
  HEAVY/LEARNED TIER (later, §15.7) = MUST-BUILD streaming iForest / HS-Trees / DenStream/CluStream/
  streaming-kmeans (no maintained Rust streaming crate; linfa 0.8.1 + extended-isolation-forest 0.2.3
  cover only the batch/on-demand-train case).

- **VERIFICATION (L-C7-2):** model mergeable sketches as MONOIDS; new VP/Kani targets (siblings to
  VP-014/VP-015) for bounded-state (anti-DoS) + monotonic-count + count-min-never-underestimates +
  HLL-register-monotonicity + Welford-M2-non-negativity + merge-monoid-laws; test order-dependence
  via seeded-permutation + reference-oracle differential + metamorphic-relation proptests; classify
  each sketch order-agnostic / order-bounded-ε / order-dependent in its spec. VP-NNN numbers
  deferred to morph (OQ-C7-2; VP-INDEX propagation obligation requires atomic burst across VP-INDEX
  + verification-architecture.md + verification-coverage-matrix.md).

- **PRIMITIVE→ENGINE COMPILATION (L-C7-3, §15.6):** `PROFILE <entity> OVER <window>` = incremental
  per-entity sketch in a windowed RocksDB state store (window = RetentionCache hot window §3.3 or
  scoped federated pull §15.2); `ANOMALY_SCORE/BASELINE_DEVIATION` = z-score/residual over the
  incremental Welford/EWMA baseline; `RARITY` = per-value frequency/cardinality (count-min/HLL);
  `FIRST_SEEN` = per-entity seen-set updated on novel keys; `PEER_OUTLIER` day-2-first =
  ATTRIBUTE-BASED peer group (`GROUP BY peer_attrs` cohort, scored by z-score vs cohort robust
  mean/MAD), clustering-based peer groups LATER. COST-BOUND (genuinely-novel control, thin vendor
  prior art): sketches bounded-state by construction + existing mandatory time-bound/join-guard
  NFRs + a NEW GROUP-BY-entity CARDINALITY CAP + per-query baseline-compute admission budget +
  time-bound-predicate-pushdown-first (Kusto pattern). OQ-C7-5: cardinality cap value + budget
  design.

**§2.4 TRADEOFF PROSE UPDATE (G-26) is a PO action** — the three-ways-to-long-baseline reframe;
flagged, NOT written here.

**OPEN QUESTIONS for architect/morph:** OQ-C7-1 (edge-mergeability bake-off — measure approx-error
at edge-ML build milestone); OQ-C7-2 (VP-NNN allocation for sketch verification targets; atomic
burst required); OQ-C7-3 (dual-rate+quarantine policy knobs); OQ-C7-4 (WASM sandbox capability
grants for inference vs hook/connector plugins); OQ-C7-5 (entity-cardinality cap + baseline-compute
admission budget); OQ-C7-6 (changelog retention policy parameters); PIV-C7-1 (`ort` RC → stable
status check at morph).

**Downstream SAP-1 flag:** ml.model.update, ml.model.materialization, ml.drift.detected,
ml.quarantine.pending/promoted, ml.anomaly_score.computed events likely need BC-2.16.002 catalog
rows — flagged in capture artifact; NOT actioned here. Epics: E-ML-ONDEMAND-001 +
E-ML-ONLINE-001 + E-ML-PRIMITIVES-001 (§15.10).

---

## Section 16 — Session Continuity & Resume Notes (2026-06-25 side-analysis session)

> **PROVENANCE.** 2026-06-25 side-analysis addendum. Written as a ZERO-CONTEXT RESUME aid before a
> context clear: a fresh session can read this doc alone and continue with no loss. This is SIDE work
> (the live factory continues independently); it does NOT touch STATE.md / SESSION-HANDOFF.md.

### 16.1 What this session produced (all on disk / committed)

- **Sections 10–15** of this doc: Query.io competitive analysis (§10), server pillars — credentials/
  config/multi-surface UI (§11), PrismQL deliverables — entity-pivot grammar / join-guard NFR /
  ergonomics ledger / SEQUENCE-sugar grammar (§12), static-vs-dynamic connectors + multi-schema
  authority (§13), detection engine & rule editor — detection-as-query (§14), on-demand ML + online
  learning (§15).
- **Research artifacts** under `.factory/research/` (committed `0df60da9`): `queryio-federated-search-`,
  `federated-query-language-patterns-`, `queryio-deployment-credentials-ui-`, `match-recognize-rpr-
  feasibility-2026-06-25.md`; plus `axiathon-detection-engine-analysis-2026-06-25.md`.

### 16.2 Decisions CONFIRMED this session (settled — do not re-litigate)

1. UI is **multi-surface, multi-persona** (HUMAN DIRECTIVE): S1 MCP/BYO-agent · S2 full browser
   console · S3 server-hosted embedded AI · S4 browser extension · U1 admin console (§11.3). Value-prop
   #5 to be softened (drafted replacement in §11.3; PO to ratify).
2. Secret storage = **hybrid: built-in encrypted store AND external vault backends** (§11.1).
3. Detection model = **detection-as-query** (PrismQL + YAML metadata), NOT a separate DSL (§14.1).
4. Sequence detection = **full `MATCH_RECOGNIZE` operator pulled forward (Phase A in scope now)**, with
   a readable `SEQUENCE…THEN…WITHIN` sugar on top (§12.4, §14.2.1). Phase-B join-rewrite = optimizer
   fast-path only.
5. Correlation/detection state = **RocksDB / RetentionCache (Prism-native)** — NOT PostgreSQL (§14.3).
6. RetentionCache = **tiered: RocksDB hot + Iceberg cold**, multi-schema, shared read path with the
   Security Lake connector (§3.3 addendum).
7. **Multi-schema** engine confirmed: OCSF (versioned) + native schema-on-read + source-native dialects
   + protobuf; PrismQL type system + Iceberg tier both multi-schema-aware (§13.6).
8. Rule editor surfaces = **S2 + MCP + CLI; NO TUI** (§14.4).
9. **OT detection IN SCOPE** (Claroty/Armis + Purdue satellite mesh) (§14.6).
10. **On-demand ML** confirmed (§15.0–15.3) and **online/continuous learning** confirmed — "the model
    is the memory," model as a third retention tier, honest-limit controls in scope, learn-from-touch
    + optional scheduled sampling (§15.4–15.5).

### 16.3 Residual Query-doc (llms.txt) verdicts — captured here so they're not lost

| Item | Verdict |
|------|---------|
| **Query App for Splunk** (run federated search inside Splunk) | **DEFER** — GTM embedding, not core; candidate post-day-2 "PrismQL app for Splunk" |
| **Security & Privacy posture page** | **ADOPT (docs)** — publish a security/privacy/compliance doc (AI-opacity, residency, SOC2); supports MSSP/regulated positioning; ties to §11.1 |
| **Full OCSF QDM breadth** (70+ event classes, 150+ objects, Data Types) | **ADOPT (raise the bar)** — target comprehensive OCSF coverage, not a 4-sensor subset; feeds dynamic-connector mapping (§13) and multi-version OCSF (§13.6) |
| **Search Progress & Results** streaming UI | **ADOPT** — already folded into S2 results-explorer (per-source coverage banner + streaming, §11.3.1) |
| **Recipes** (200+ FSQL detections/hunts) | **ADOPT as executable, backtested, version-controlled PrismQL recipe + hunt library** (§14.7, E-DETECT-RECIPES-001); includes Sigma→PrismQL examples |
| **CoPilot** (AI schema mapping + assistant) | **ADOPT** — S3 embedded agent drives configure-schema mapping (§13.2) |
| **FAQL** | CORRECTION applied (§10.1): it is an FSQL FAQ page, not a language |

### 16.4 Open items / NEXT STEPS (for the resumed session)

- **UI (substantially built out 2026-06-25/26 side-analysis):** UI-needs research DONE — see
  `research/ui-requirements-2026-06-25.md` (distilled) + the two raw passes. **UI-D5 RESOLVED: Option A —
  TS SPA (React) + Rust backend, OpenAPI→TS shared types** (§11.3 web-stack bullet).
  - **Design system + mockups COMPLETE:** `day2-ui-design/mockups/` holds brand-derived `tokens.css`
    (light = 1898 & Co palette parsed from live CSS; dark = derived), a `style-guide.html` component kit,
    and **21 panel mockups (13 S2 + 8 U1), each light+dark** with 44 screenshots. ux-designer S2/U1 specs
    are in `day2-ui-design/S2-investigations-console.md` + `U1-admin-console-inventory.md`.
  - **Conversational canvas EVALUATED → HUMAN-DECIDED ADOPT as S3** (enhanced + hardened) from the
    `aletheon_2` generative-UI spike. Full verdict: `day2-ui-design/S3-conversational-canvas-disposition.md`
    + §11.3.2 addendum. S3 canvas mockups: `mockups/S3-01-ai-canvas.html`, `S3-02-ai-canvas-multioption.html`.
  - **Additional surfaces built (light+dark):** S4 browser extension (`mockups/S4-01-extension.html`),
    responsive breakpoints for key S2 panels (`mockups/responsive-S2-*.html`), and a canonical state-coverage
    gallery (`mockups/states-gallery.html`).
- **Day-2 design decisions CAPTURED (2026-06-26 side analysis)** — all the prior open items below were
  resolved into PROPOSED capture artifacts under `specs/day2-design-decisions/` (do_not_execute; real ADR
  numbers + brief-reframe ratification deferred to morph). Each carries a firm recommendation + flagged
  residual human decisions:
  - `ADR-PROP-web-stack.md` — formalizes UI-D5 (TS SPA/React + Rust). 
  - `ADR-PROP-sso-identity.md` — OIDC + SAML 2.0 + 7-role RBAC; SCIM flagged.
  - `ADR-PROP-s3-agent-runtime.md` — 4-component server-hosted runtime wrapping the MCP tool surface.
  - `ADR-PROP-widget-dsl-render-and-schema-validation.md` — 54-primitive DSL + mandatory Zod gate + OCSF primitives.
  - `ADR-PROP-sandboxed-expression-evaluator.md` — prohibits `new Function()`; ANTLR4 path mandatory.
  - `secret-subsystem-sketch.md` — `SS-26 Secret Broker`, `SecretBackend` trait, per-tenant-DEK envelope (§11.1).
  - `prismql-sequence-sugar-decisions.md` — keyword set + `WITHIN` + cross-step + NOT/WITHOUT desugaring (§12.4).
  - `ml-depth-phasing.md` — P1 statistical → P2 online (`ModelState` CF) → P3 pluggable `ModelBackend` (§15.7).
  - `po-ratifications.md` — value-prop #5 + §2.4 softening + §1.x framing (PO-ratified-ready, gated on §5.1).
  - **HUMAN decisions RESOLVED 2026-06-26:** (1) PrismQL NOT/WITHOUT = **BOTH** exclusion-between-anchors
    (`{- B -}`) AND timeout/absence (`WATCH…UNLESS` → `AbsenceWindowNode` anti-join), both promoted into
    Phase A; (2) S3 agent runtime = **opt-in** (`s3_agent_runtime_enabled=false` default, air-gap-safe);
    (3) SCIM 2.0 = **in day-2 scope** (with OIDC+SAML+JIT); (4) multi-surface UI (S2–S4 + U1) = **§1 In-Scope**
    (committed v1, not roadmap).
  - **Still deferred to morph (recommended defaults recorded):** ML v1 starting scope; secret-store
    KMS-provider/cipher/DEK-granularity/rotation/satellite-custody defaults; and the lower-impact
    implementation flags (Monaco host, bundler, state-mgmt, MFA level, session durations, etc.).
- **Federated ingestion / collector class CAPTURED in §17 (2026-06-26 side analysis).** New collector/stream-connector
  vision: collect-at-edge → OCSF/native-normalize → demand-driven TTL'd buffer → queryable source (collector = a
  *source+buffer*, not a sink). Research: `research/federated-ingestion-collector-connectors-2026-06-26.md` +
  `research/chain-cache-tiering-replication-deadlines-2026-06-26.md`. **DECIDED 2026-06-26 (human):** (a) full-packet
  pcap retrieval IN day-2 scope (second storage regime, Arkime-style); (b) prism will own a continuous-operator
  capability, PHASED (v1 NRT-over-cache + edge Zeek/Suricata → later native windowed operator); (c) collection locus
  is per-instance, edge-first default. Chain-aware model leans: declarative-policy-floor tiering; residency-first
  per-field ordered-before-forward replication policy (ahead of prior art); Q3 deadline v1 (gRPC + partial+coverage +
  opportunistic hub pre-aggregate), full budget-aware planner ordered later. Open items: §17.10 (11 questions); proposed
  E-COLLECTOR-*/E-CHAIN-CACHE-*/E-STREAM-DETECT-* epics + ADRs (§17.11, gaps G-27–G-31).
  - **Protocol dissectors + #4/#5 reshape CAPTURED in §17.12–§17.14** (research `detection-reshape-protocol-dissectors-2026-06-26.md`,
    R1–R7). **DECIDED 2026-06-26 (human):** prism EMBEDS a native Spicy-style declarative dissector engine (authors its own
    grammars incl. OT — not federating Zeek/Suricata); prism-NATIVE continuous windowed operator on the RocksDB state backend
    (reuse the MATCH_RECOGNIZE NFA + watermark/checkpoint layer, not embedded Flink); the detection spec carries EXPLICIT
    temporal semantics (`lateness`/`accumulation`/window-alignment in §14 YAML — planner picks engine, not meaning); encrypted-OT
    visibility metadata-only by default + bounded decrypt/proxy opt-in later; **OT = flagship native-schema-on-read** (OCSF has NO
    OT classes as of 2026, open proposal ocsf#1515). `WATCH…UNLESS` = dual impl (anti-join polled / per-partition timer continuous).
    Detection-driven packet retention (trigger→pin→retrieve, Community ID). State: unify operator-window + detection_state (distinct
    CFs), ML ModelState separable. New epics E-DISSECTOR-NATIVE-001 / E-DISSECTOR-OT-001 (+ extend E-STREAM-DETECT-001 /
    E-COLLECTOR-PCAP-001); gaps G-32–G-36. Honest cost: prism now owns TWO heavy native engines (Spicy-style dissector + windowed operator).
- **Still OPEN (not yet captured):** SSO↔transport binding detail; the §5.x execution-checklist items all remain
  pending the brief-reframe HUMAN GATE.
- **C1 (central access layer) + the FOUR-ENGINE STORAGE TAXONOMY (RocksDB/Iceberg/bundled-Postgres-central/
  SQLite-edge) + A's 7 ingestion sub-thread leans all DECIDED/captured 2026-06-26.** Capture artifacts on disk:
  `specs/day2-design-decisions/ADR-PROP-storage-engine-taxonomy.md` (storage taxonomy + §14.3 reconciliation);
  `specs/day2-design-decisions/ADR-PROP-central-deployment-access-layer.md` (transport/identity/credentials/
  shared-state/ops — ADR-050..054 inputs); `research/central-deployment-access-layer-2026-06-26.md` and
  `research/ingestion-open-subthreads-2026-06-26.md` (primary research); satellite-mesh and capability-descriptor
  research also on disk. Remaining program for the side-analysis track: **C2 satellite mesh** (research
  already in `research/`) → **C3 capability-descriptor model** (research ready) → **C4–C10** remaining topics
  → **B brief-reframe sign-off** (HUMAN GATE, §5.1). No further open sub-threads from today's session.
- **C2 Satellite Mesh DECIDED + CAPTURED 2026-06-27 (human).** Thirteen architecture decisions
  D-C2-1…13 confirmed. Capture artifact: `specs/day2-design-decisions/ADR-PROP-satellite-mesh.md`
  (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
  `research/satellite-mesh-2026-06-26.md`. Decisions cover: transport (gRPC bidi PRIMARY / NATS
  STRONG-ALT, prototype bake-off gated); relay trust role (terminator-only, no sub-CA); role nouns
  (Coordinator / Relay Satellite / Edge Satellite — lean, §3.4 finalizes); diode OT mode DEFERRED
  (future epic E-SATELLITE-DIODE-001); identity (SPIFFE-model native Rust, no SPIRE runtime);
  trust model (per-hop mTLS only, no transitive trust, IEC-62443 zone separation); bootstrap
  (join-token OOB + optional TPM); loop prevention (request-ID + hop-TTL ceiling + optional
  path-vector); per-hop deadline decrement (gRPC model, ties §17.8 Q3 v1); store-and-forward
  (RocksDB new CF, drop-oldest-loud, at-least-once + dedup); partial-failure relay (extends
  BC-2.01.010 / CCS lineage, no hop swallows downstream gap); residency (structural enforcement,
  IEC-62443 zones-and-conduits map, satellite-local credential resolution hard invariant binding
  AD-017); max chain depth (8-hop production default, configurable). Remaining open questions:
  trust-anchor rotation across deep tree; transport fork bake-off; MITM proxy survival; per-zone
  normalization attestation; diode transport variant. §3.2 decision block updated in-place.
- **C3 Capability-Descriptor + PrismQL Pushdown + Cross-Source Cost Guards DECIDED + CAPTURED
  2026-06-27 (human).** Four architecture decisions D-C3-1..4 confirmed; leans L-C3-1..8 confirmed.
  Capture artifact: `specs/day2-design-decisions/ADR-PROP-capability-descriptor-pushdown.md`
  (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
  `research/capability-descriptor-pushdown-2026-06-26.md`. Hardening pass on DataFusion 50.x
  mechanics in flight: `research/datafusion-cost-degrade-mechanics-2026-06-27.md` (OQ-C3-1..6
  pending fold-on-return). Decisions cover: join guard = cost-based degrade NOT hard-reject
  (D-C3-1, supersedes §5.3/§12.2 reject-framing — reconciliation notes appended at both sections);
  missing time-bound = inject default window + disclose NOT reject (D-C3-2); allow outer/non-equi
  cross-source joins central-only without dynamic-filter (D-C3-3); override = audited PrismQL hint
  capped at absolute max (D-C3-4). Confirmed leans: declarative TOML descriptor per [[tables]]
  fail-closed; enumerated predicate-class vocabulary (Spark-style, NOT open expression trees);
  contract split DataFusion TableProvider vs PrismQL pre-pass; descriptor per-(table, schema-class);
  bijection test for transform exactness; collector declares pushdown_target=buffer; minimum
  DataFusion=50.x; #[non_exhaustive] on all descriptor structs. Downstream spec dependencies
  flagged: BC-2.16.002 new catalog rows for query.pushdown.decision + query.injected_default_window
  + query.override_applied (SAP-1); §12.2 NFR-JOIN-GUARD language amendment; E-CONNECTOR-
  CAPABILITY-DESCRIPTOR-001 proposed epic. §3.4 C3 decision block appended in-place.
- **C4 Dynamic-Schema / Configure-Schema Connectors DECIDED + CAPTURED 2026-06-27 (human).**
  Four architecture decisions D-C4-1..4 confirmed; leans L-C4-1..6 confirmed. Capture artifact:
  `specs/day2-design-decisions/ADR-PROP-dynamic-schema-connectors.md`
  (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
  `research/dynamic-schema-connectors-2026-06-27.md`. Hardening pass on boundary-normalization
  + WASM sandbox in flight: `research/connector-boundary-sanitization-wasm-2026-06-27.md`
  (OQ-C4-1..6 pending fold-on-return). Decisions cover: boundary-normalization scope = ALL
  connectors including existing OCSF security sensors, NO trusted-source exemption
  (D-C4-1 — honest cost: adds normalization chokepoint + latency to existing prism-sensors
  hot path); drift on upstream column removal = auto-narrow + structured drift event,
  NO re-pin required for narrowing (D-C4-2); WASM code-connector escape-hatch COMMITTED
  in day-2, sandboxed (stronger posture than Airbyte no-sandbox), must reconcile with
  existing plugin SDK at morph (D-C4-3); hostile identifier handling = quarantine + relabel
  to safe placeholder + original in audit field, hard-reject only on control chars/bidi/
  over-length (D-C4-4). Confirmed leans: static TOML = schema acquisition default
  (introspection/inference = confirm-or-narrow-only, NEVER auto-widen); two-hop type
  mapping source-native → Arrow → Prism ColumnType (map-to-canonical-or-reject; lossy
  coercions weaken C3 pushdown exactness to inexact; `lossy = true` TOML opt-in for
  Json/Text fallback; do NOT reintroduce retired shadow enum per ADR-024); drift = event
  to surface never silent adaptation (Confluent vocabulary; Fivetran supertype promotion
  rejected; Iceberg field-ID evolution for cold tier); config-vs-code boundary = formulaic
  REST/SQL/LDAP → TOML, imperative state / non-REST / custom signing → WASM; DataFusion
  integration = schema() from pinned TOML, boot-time C3↔C4 reconciliation invariant
  (descriptor.columns ⊆ provider.schema().fields, fail-closed on over-declaration).
  Downstream SAP-1 obligations: BC-2.16.002 new catalog rows for
  connector.schema.drift.detected + connector.schema.identifier.sanitized +
  connector.schema.identifier.rejected + connector.schema.coercion.lossy (morph-time).
  §3.4 C4 decision block appended in-place. §13 C4 pointer appended in-place.
  Proposed epic: E-CONNECTOR-DYNAMIC-001 (§13.4).

- **C5 SIEM / Security-Lake Federation DECIDED + CAPTURED 2026-06-27 (human).** Five architecture
  decisions D-C5-1..5 confirmed; leans confirmed. Capture artifact:
  `specs/day2-design-decisions/ADR-PROP-siem-lake-federation.md`
  (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
  `research/siem-lake-federation-2026-06-27.md` (C5 flagship research) +
  `research/coldtier-iceberg-vs-hive-parquet-2026-06-27.md` (head-to-head, Iceberg reaffirmed).
  **D-C5-1 CORRECTION (load-bearing):** Amazon Security Lake is OCSF Parquet in Hive-style
  partitions (`region=/accountId=/eventDay=`) cataloged in AWS Glue Data Catalog + Lake Formation —
  NOT Apache Iceberg (Iceberg appears only in the separate S3 Tables service). The §3.3 addendum
  sentence has been corrected in-place above: "one DataFusion ENGINE, two TableProviders — the
  self-managed cold tier reads via `IcebergTableProvider`; Amazon Security Lake reads via a distinct
  Glue/Hive-Parquet (`ListingTable`) provider. The engine-level unification holds; the
  storage-format equivalence does not." This correction stands regardless of the cold-tier format
  choice (Security Lake is Hive-Parquet either way).
  **D-C5-1b COLD TIER = APACHE ICEBERG, REAFFIRMED 2026-06-27 (human).** The head-to-head
  (`coldtier-iceberg-vs-hive-parquet-2026-06-27.md`) leaned SWITCH-to-Hive-Parquet but the human
  REAFFIRMED Iceberg for: ACID + field-ID schema-evolution + row-level-mutation HEADROOM (GDPR
  erasure / customer offboarding / event correction — research flip conditions #1/#2), choosing
  durability + future-proofing over the one-provider simplification. ADR-PROP row 2 (Apache Iceberg
  cold tier) STANDS. iceberg-rust reached 0.9.0 (2026-03-10) with significantly expanded
  DataFusion integration (DDL-via-SQL, limit + broad predicate pushdown, sort-clustered partitioned
  insert) but remains pre-1.0. R4's original "Security Lake IS Iceberg → one mechanism"
  justification was a false premise (corrected per D-C5-1); the Iceberg cold-tier decision is
  reaffirmed on R2/R3 (schema evolution) + row-level-mutation headroom, NOT on R4.
  **D-C5-2 SECURITY LAKE BINDING = S3 data-access subscriber DEFAULT** (read raw OCSF Parquet from
  S3 with IAM creds; prism does partition projection on `region/accountId/eventDay` + OCSF
  interpretation — sidesteps the iceberg-rust Glue gap, dogfood-consistent), with
  `LAKEFORMATION` query-access (Glue/Athena, LF-governed) as an OPT-IN for deployments that
  mandate it.
  **D-C5-3 RESIDENCY = REJECT AT PLAN-TIME, UNIFORM with D-C2-12.** A query targeting a lake in a
  residency-disallowed region/tenant is REJECTED at PrismQL plan-time before any S3 GET, via
  fail-closed descriptor binding (out-of-region tables not bound) + explicit residency-denied
  structured audit event. REVERSES an initial degrade lean — residency is a HARD boundary
  everywhere (mesh AND lake), NOT a cost-degrade surface. Asymmetric with D-C3-1 cost-based-degrade
  join posture (residency = hard/reject; cost guards = degrade); asymmetry is intentional.
  **D-C5-4 iceberg-rust LINEAGE = DEFER binding to morph-time prototype bake-off.** Both ASF
  `apache/iceberg-rust` and `JanKaul/iceberg-rust` recorded as candidates. Lean: ASF + REST/S3-Tables
  catalog (icepick-proven) as probable default; JanKaul only if direct-Glue/equality-deletes become
  hard requirements.
  **D-C5-5 FEDERATION SOURCES ARE CONNECTORS → PLUGINS (HUMAN DIRECTIVE, load-bearing).** Amazon
  Security Lake AND every SIEM/lake federation source (Splunk, Elastic/OpenSearch, Microsoft
  Sentinel, Google SecOps/Chronicle, Snowflake, Databricks) is a Connector per §3.4 taxonomy, and
  connectors ARE REQUIRED to be PLUGINS (per the connector-plugin model + existing plugin SDK at
  `crates/prism-spec-engine/plugins/`). They are NOT core-engine built-ins. CRITICAL ASYMMETRY:
  the self-managed Iceberg cold-tier provider is prism-INTERNAL/CORE (prism's own storage); the
  Security Lake (Glue/Hive-Parquet) provider AND all SIEM adapters are CONNECTOR PLUGINS. These
  federation connector-plugins INHERIT all C4 connector decisions: D-C4-1 mandatory
  boundary-normalization chokepoint; D-C4-3 capability-sandboxed (Wasmtime WASI-P2
  no-ambient-authority) WASM plugin host; opaque credentials resolved satellite-local (AD-017 / C2);
  discover-then-pin static-TOML-default with confirm-or-narrow probes; the C3 capability-descriptor
  with per-(table, schema-class, schema-VERSION) key.
  Confirmed leans: two adapter archetypes — PUSHDOWN-API ("fetch-then-residual":
  Splunk/Elastic/OpenSearch/Sentinel/SecOps; native-query-evaluable predicates exact; JOIN always
  central; fetch + re-check residuals centrally; OCSF-normalize at boundary) and LAKE-BULK-READ
  ("prune-then-scan": Security Lake/Snowflake-external/Databricks-Delta; DataFusion scans
  Parquet/Iceberg/Delta directly with partition + column-stat pruning; time-range as partition prune
  exact, equality/IN on stats-bearing columns as file-prune often inexact → central re-check). NO
  SIEM IS BULK-READABLE (SmartStore/Elastic-frozen/Sentinel-ADLS proprietary; Sentinel-lake and
  SecOps bulk-read paths UNCONFIRMED — treat as pushdown-only until vendor-documented). OCSF VERSION
  AXIS: descriptor key = per-(table, schema-class, schema-VERSION); Security Lake lags 1.1/1.3 vs
  upstream 1.6.0; normalize inbound UP to target at boundary; carry `metadata.version` for audit.
  COST: mandatory time-bound (C3 Topic 4) is primary control — tighter default window for lakes than
  live sensors; plus egress ceiling + default/max result-limit. WRITE PATH (cold tier, Iceberg):
  append-only RETAIN→Iceberg realistic today; single-writer-per-table; catalog = REST or S3 Tables
  (icepick-proven), NOT Glue; record `ingest_time` per row. Open questions: DataFusion SQL `AS OF`
  time-travel surfacing incomplete in iceberg-datafusion; pre-scan bytes-scanned estimation
  precision for disclosure envelope; whether federation connector-plugins use TOML declarative path
  or WASM escape-hatch (lake Parquet + IAM + partition projection may need WASM — reconcile against
  existing plugin SDK at morph). Downstream SAP-1 obligations (NOT actioned here): lake-read
  pushdown-decision, injected-window disclosure, egress-estimate, residency-denied, and
  OCSF-version-skew events each need new BC-2.16.002 Canonical Structured Event Catalog rows.
  Proposed epic: E-LAKE-CONNECTOR-001 (§3.5).

- **C6 Detection Engine DEPTH FULLY DECIDED + CAPTURED 2026-06-27 (human).** All three
  architecture decisions D-C6-1/2/3 confirmed (D-C6-3 auto-rollback RESOLVED 2026-06-27 — folded
  from deep-research pass); implementation leans L-C6-1/2/3 confirmed. Capture artifact:
  `specs/day2-design-decisions/ADR-PROP-detection-engine-depth.md`
  (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
  `research/detection-engine-depth-2026-06-27.md` (six sonar-deep-research calls at
  `reasoning_effort=high`, 2 Context7 calls for DataFusion API) +
  `research/detection-auto-rollback-depth-2026-06-27.md` (six sonar-deep-research calls at
  `reasoning_effort=high`, Q1–Q6 depth on the auto-rollback control loop).
  **D-C6-1 BACKTESTING (G-19) = BOTH cold-tier deterministic (Iceberg snapshot-id + rule-version
  pin, reproducible, no look-ahead bias) AND remote best-effort (current-state, NON-deterministic,
  retention-bounded — stated explicitly), ALWAYS with a mandatory coverage map per (source ×
  time-slice) labeled `{full/partial/none}`. The single most important correctness affordance:
  "EVALUATED, no match" vs "NO DATA to evaluate" — ALL surveyed prior art (Elastic/Chronicle/
  Panther/Splunk) is missing this; Prism builds it from scratch.** Mandatory time-bound +
  volume ceiling + dry-run estimate. Reuse §14.5 ADOPT-4 source-coverage-record + replay-link.
  Cold-tier-only rejected (remote sources = most customer data). C6-decision block appended at
  §14.9 in-place.
  **D-C6-2 FALSE-POSITIVE HANDLING = RBA as default over hard suppression.** Noisy events accrue
  risk to entity; alert on aggregated risk → retains underlying-event visibility. Hard suppression
  = suppression-as-code (versioned, mandatory justification, mandatory time-box expiry, no immortal
  exceptions, fire-frequency/scope-breadth dashboard). Auto-tune = suggestions only, NEVER
  auto-applies, NEVER auto-disables evaluation without human sign-off. Honest caveat stated
  plainly: "never silently mask a true positive" is NOT achievable as an absolute guarantee — the
  production-grade posture is transparency + narrow-scope + mandatory-justification + time-boxed-
  expiry + fire-frequency-dashboards + RBA-over-suppression, NOT a proof.
  **D-C6-3 AUTO-ROLLBACK RESOLVED — OQ-C6-AUTOROLLBACK CLOSED (2026-06-27).** AUTO action =
  DEMOTE-TO-SHADOW (rule keeps evaluating, stops routing; idempotent; already-emitted findings
  annotated not retracted). FULL-DISABLE = explicit human sign-off (SOAR coverage-reducing-action
  gate). REVERT-TO-PRIOR-VERSION = one-click human action, not auto reflex. Trip gated by
  CORROBORATION-MASTER-GATE: (a) corroborated + entity-concentrated → DO NOT auto-rollback,
  ESCALATE; (b) uncorroborated + uniformly dispersed + sustained + low incident yield → demote-
  to-shadow is safe; (c) ambiguous → HOLD-AND-ESCALATE. Per-tenant circuit-breaker on the routing
  path (closed=route / open=demote / half-open=trial-route). Signals: CUSUM on alert-rate `λ_t`
  (primary; ADWIN if diurnal) + cardinality `U_t` + `N_t/U_t` duplicate-ratio (secondary);
  analyst dispositions = delayed validation only (DDM/EDDM need labels, cannot drive real-time
  trip). Trip = N-of-M + relative-to-baseline multiplier + absolute backstop + minimum-window
  count + corroboration gate. Hysteresis: cool-down → half-open trial → `consecutiveSuccessLimit`
  clean windows to re-close + exponential backoff on repeated trips + human confirmation before
  re-promotion (no auto re-promote). Shared change-detector primitive with C7. Promotion: shadow→
  canary auto-gated on metrics + bake window; canary→production human-gated (high-blast-radius).
  Canary unit = TENANT. Gate thresholds in §14.1 `quality` block. Honest caveat: "never roll back
  a working rule" is a safety POSTURE (corroboration gate + concentration test + sustained-window
  + hold-and-escalate + demote-not-disable + human re-promote), not an absolute guarantee. No SIEM
  ships integrated detection auto-rollback; Prism assembles from progressive-delivery + circuit-
  breaker + change-detection + SOC-triage prior art and owns the integration. Residual pre-
  implementation items PIV-C6-RB-1..9 (not blocking forks; see capture artifact §D-C6-3).
  **LEANS confirmed:** MATCH_RECOGNIZE = `MatchRecognizeNode` (UserDefinedLogicalNode) +
  `MatchRecognizeExec` (custom ExecutionPlan) wrapping Thompson-NFA instruction-program matcher
  (`MATCH`/`SPLIT`/`JUMP`/`CHECK`/`ACCEPT`; greedy/reluctant = SPLIT branch ordering; serializable
  match-context from day one for §17.7 reuse); optimizer fast-path = RelationPlanner rewrite for
  simple fixed-step SEQUENCE → self-join + window (5.4×); ExtensionPlanner wiring flagged for
  pre-implementation re-verify at pinned version (PIV-C6-1). Continuous RPR (§17.7 Phase 2) =
  SAME matcher core + watermark + per-partition event-time TIMER + incremental-checkpoint on
  RocksDB CFs (window-state CF distinct from detection_state CF; CF isolation PIV-C6-2).
  Sigma→PrismQL (deferred E-RULE-XLATE-001, feasibility confirmed) = pySigma-style Backend +
  ProcessingPipeline targeting OCSF taxonomy; lossy edges → fidelity report (NEVER silent drop);
  Sigma correlation rules map cleanly onto MATCH_RECOGNIZE (Prism well-positioned); ship
  Sigma→PrismQL EXAMPLES in recipe library (§14.7) now. Open questions: PIV-C6-1/2, OQ-C6-3
  (checkpoint cadence), OQ-C6-4 (coverage-map data model), OQ-C6-5 (Iceberg snapshot-retention
  policy), OQ-C6-6 (Sigma fidelity-report schema), PIV-C6-RB-1..9 (auto-rollback pre-impl items).
  OQ-C6-AUTOROLLBACK = CLOSED. Downstream SAP-1: backtest-coverage/rollout-transition/suppression-
  fire/auto-tune-suggestion events likely need BC-2.16.002 catalog rows (flagged, NOT actioned).
  Proposed epics: E-DETECT-ENGINE-001 + E-DETECT-SEQUENCE-001 + E-DETECT-EDITOR-001 +
  E-ALERT-ROUTING-001 + E-RULE-XLATE-001 (§14.8).

- **C7 On-Demand ML & Behavior Analytics DEPTH DECIDED + CAPTURED 2026-06-27 (human).** Four
  architecture decisions D-C7-1..4 confirmed; implementation leans L-C7-1..3 confirmed. Capture
  artifact: `specs/day2-design-decisions/ADR-PROP-ml-behavior-analytics-depth.md`
  (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
  `research/ml-behavior-analytics-depth-2026-06-27.md` (six sonar-deep-research calls at
  `reasoning_effort=high`; 13 live crates.io version-verifications 2026-06-27).
  **D-C7-1 C7 FOLD RESOLVED 2026-06-27.** Prior posture (mergeable-only as conservative default;
  non-mergeable primitives EWMA/reservoir/clustering central-only) UPGRADED by depth research
  (`research/edge-ml-mergeability-depth-2026-06-27.md`): representation-change escape hatches make
  formerly non-mergeable primitives **mergeable-EXACT broadly**: (a) EWMA → forward-decay `(U,V)`
  sufficient-statistic representation: mergeable-EXACT (Cormode–Shkapenyuk–Srivastava–Xu;
  [LIT-SETTLED]); (b) Reservoir → random-key/bottom-k (Efraimidis–Spirakis): mergeable-EXACT, no
  per-shard count relay required ([LIT-SETTLED]); (c) Clustering → BIRCH CF-vectors `(N, LS, SS)`
  (additive): mergeable-EXACT at the CF level ([LIT-SETTLED] — CONFIRMS the prior human hypothesis
  that clustering can be made additive); bounded approximation only in fading-weight time-alignment
  (`2^{λΔ}` clock-skew bound) and macro-clustering coreset error (which also exists single-machine).
  **Consequences:** edge-ML mergeability is the BROAD DEFAULT via mergeable-exact representations,
  not a narrow constraint. Scalar-state approximate-merge (scalar EWMA, Algorithm-R + weighted
  re-sampling) is a constrained-edge FALLBACK ONLY (error bounds extrapolated, not literature-settled).
  **Coarsening ≠ privacy:** representation coarsening is an accuracy/footprint lever, NOT a privacy
  mechanism; local-DP on mergeable DP sketches is the separate concern if a formal privacy guarantee
  is required (PIV-C7-3). **Remaining empirical item (narrowed):** macro-clustering drift test —
  measure whether BIRCH CF-vector merges preserve macro-cluster fidelity under adversarial
  cross-shard skew (ARI/NMI/CMM vs central reference; OQ-C7-1 narrowed to this single item).
  ADR-PROP D-C7-1 updated in-place; PIV-C7-2 (representation correctness gate) + PIV-C7-3
  (coarsening≠privacy invariant probe) added to Open Questions table.
  **D-C7-2 MODEL BACKENDS = COMMIT THE FULL PLUGGABLE-BACKEND SET IN DAY-2.** First-party
  `ModelBackend` trait (AI-opaque, per-tenant isolated, AD-017 compatible). Backends: first-party
  statistical sketches (statistical tier, day-2-first) + candle-core 0.11.0 (built-in learned,
  healthy 2026-06-26) + ort 2.0.0-rc.12 (TRUSTED BYO ONNX, process-isolated — still RC, pin exactly)
  + wasmtime 46.0.1 Component-Model WASM plugins (UNTRUSTED BYO, REUSE C4 WASM sandbox pattern from
  ADR-PROP-dynamic-schema-connectors.md D-C4-3) + tract-onnx 0.23.3 (satellite/edge tiny runtime).
  HONEST COSTS: ort is RC; WASM-ML has performance tax (SIMD/GPU build flags; GPU-in-WASM immature);
  this is the larger-build option chosen over statistical+candle-first. OQ-C7-4: whether inference
  workloads need different WASM capability grants than connector/hook plugins.
  **D-C7-3 DRIFT / DECAY / POISONING = DUAL-RATE + QUARANTINE.** Day-2 baseline math = robust
  estimators (median/MAD) + bounded per-window update rate + anomaly-gated learning. Drift
  detectors MUST-BUILD in Rust (Python-only ecosystem — River/scikit-multiflow; `neural-drift`
  undocumented algorithms): ADWIN (self-tuning window = doubles as decay) + Page-Hinkley first.
  Fast model flags; slow model only absorbs drift persisting past a quarantine window OR confirmed
  by human/S3-agent (§15.8). HONEST CAVEAT: dual-rate + quarantine SHIFTS attacker cost; does NOT
  eliminate boiling-frog risk — state this plainly, do NOT imply a guarantee. OQ-C7-3: policy knobs.
  **D-C7-4 MODEL REPLAY / EXPLAINABILITY = PER-UPDATE CHANGELOG + PERIODIC MATERIALIZATION (human
  chose full per-update auditability over the lighter per-detection content-addressed option).**
  Append every model update as a delta to a changelog; materialize consolidated snapshots periodically;
  finding's replay link (§14.5) references (materialization-id + changelog-offset) → enables replaying
  EVERY update. HONEST COST: heavier than content-addressed (more storage + changelog/materialization
  machinery). Content-addressed per-detection snapshots = rejected lighter alternative (per-detection
  auditability only, not per-update). Storage = RocksDB (the §3.3 tier); bincode schema-versioned
  `ModelState` envelope (redacted Debug, `#[non_exhaustive]`); per-tenant via CF (few large tenants)
  + key-prefix (long tail) — RocksDB degrades at thousands of CFs so do NOT map CF-per-tenant for
  the long tail. OQ-C7-6: retention policy parameters.
  **LEANS confirmed:** statistical toolkit (first-party Welford+CGL+EWMA+count-min+drift-detectors;
  depend on sketches-ddsketch 0.4.0 + hyperloglogplus 0.4.1 + tdigests 1.0.1; heavy streaming
  tier MUST-BUILD later); verification (mergeable sketches as MONOIDs; new VP/Kani targets siblings
  to VP-014/VP-015 — OQ-C7-2 allocates VP-NNNs atomically at morph); primitive→engine compilation
  (PROFILE→incremental RocksDB sketch; ANOMALY_SCORE/BASELINE_DEVIATION→z-score/residual;
  RARITY→count-min/HLL; FIRST_SEEN→seen-set; PEER_OUTLIER day-2-first = attribute-based GROUP BY
  cohort + z-score vs cohort robust mean/MAD; COST-BOUND = cardinality cap + admission budget +
  time-bound pushdown first). §2.4 tradeoff prose (G-26) = PO action flagged; NOT written here.
  Downstream SAP-1 obligations: ml.model.update/materialization, ml.drift.detected,
  ml.quarantine.pending/promoted, ml.anomaly_score.computed events likely need BC-2.16.002 catalog
  rows (flagged, NOT actioned). Open questions: OQ-C7-1..6 + PIV-C7-1. Proposed epics:
  E-ML-ONDEMAND-001 + E-ML-ONLINE-001 + E-ML-PRIMITIVES-001 (§15.10).

- **C8 PrismQL Deliverables FULLY DECIDED + C8 FOLD COMPLETE 2026-06-27.** D-C8-1 piped
  surface (DECIDED) + D-C8-2 entity-resolution AS OF reproducibility (RESOLVED via C8 FOLD) +
  D-C8-3 OCSF version-binding (RESOLVED via C8 FOLD) + leans L-C8-1..5 (CONFIRMED). Capture
  artifact: `specs/day2-design-decisions/ADR-PROP-prismql-deliverables.md`
  (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
  `research/prismql-deliverables-depth-2026-06-27.md` (four sonar-deep-research + 1
  perplexity_reason + 3 Context7 calls) + **C8 FOLD research:
  `research/prismql-asof-version-resolution-2026-06-27.md`** (3 perplexity_research at
  reasoning_effort=high + 2 perplexity_ask for targeted verification — bitemporality as
  unifying answer, Fork A entity-AS-OF, Fork B OCSF version-binding, interaction, costs).
  **D-C8-1 PIPED SURFACE = SHIP IN DAY-2.** KQL/PRQL-style `|`-pipe syntax desugars to the
  SAME DataFusion logical plan — NOT a second engine. Proven viable by PRQL + RunReveal pql.
  MANDATORY: expose "show desugared SQL / EXPLAIN."
  **D-C8-2 ENTITY-RESOLUTION AS OF REPRODUCIBILITY = RESOLVED: BITEMPORALITY (C8 FOLD
  2026-06-27).** OQ-C8-ASOF CLOSED. Adopt the BITEMPORAL REGISTRY (valid-time
  interval-containment, settled + transaction-time axis, resolved here). A single `AS OF KNOWN
  <T>` decision-time knob pins the entity-registry transaction-time. Fresh-by-default (absent
  `AS OF KNOWN <T>`, queries use the LATEST registry state). Forensic / saved-finding path:
  stamp findings at §14.5 replay-link with decision-time T → `AS OF KNOWN T` on replay.
  Prism-novel differentiator: no commercial security tool (Chronicle/Sentinel/Splunk-ES/
  ServiceNow CMDB) implements true bitemporality for entity resolution. HONEST COST: (a)
  transaction-time axis on registry = real storage (bounded to registry + catalog, not event
  stream; magnitude INCONCLUSIVE — must measure); (b) data-snapshot pinning for full
  `AS OF KNOWN <T>` over C5 cold tier DEFERRED — DataFusion + iceberg-rust lacks native
  time-travel as of 2026 (OQ-C8-DATASNAPSHOT, new cost-gated open item). PIV-C8-1/2/3 added
  to capture artifact (storage axiom; fresh-by-default; scope discipline).
  **D-C8-3 OCSF VERSION-BINDING = RESOLVED: PINNABLE CATALOG + UNIFIED AS OF KNOWN <T>
  (C8 FOLD 2026-06-27).** OQ-C8-OCSFVER CLOSED. Keep version-agnostic canonical OCSF names
  as default; make the schema-catalog VERSION an IMMUTABLE, PINNABLE artifact (Confluent
  schema-id lineage). `AS OF KNOWN <T>` pins BOTH the entity-registry transaction-time (D-C8-2)
  AND the active catalog-version — one decision-time coordinate governs "the world as prism
  interpreted it at T." Fresh-by-default (absent `AS OF KNOWN <T>`, latest catalog version
  active). OCSF compatibility tiers (stable vs version-sensitive) must be derived from real
  OCSF 1.1→1.3→1.6 diffs (OCSF publishes none — prism-novel work). Catalog-pin ≠ full data
  reproducibility: for the live-sensor tier (upstream API data not under prism version control),
  `AS OF KNOWN <T>` pins interpretation only; upstream data may have changed — MUST be disclosed
  in result metadata. Prism-novel differentiator: unified `AS OF KNOWN <T>` spanning
  entity-registry + schema-catalog = "what did we know, and what was true, as of T." PIV-C8-4/5/6
  added (catalog immutability; version-sensitive field diagnostic; honest result-metadata).
  **LEANS CONFIRMED:** FIND keyword; entity-pivot two-surface design; multi-hop GRAPH_TABLE
  forward-compat (NOT day-2); single LSP server (Monaco + CLI ariadne + NL→PrismQL loop);
  NL→PrismQL guardrails reuse existing diagnostics; recipe format Sigma-aligned + CI harness.
  Open questions: OQ-C8-DATASNAPSHOT (cold-tier data-snapshot pinning, cost-gated, new),
  OQ-C8-NATIVE-RESIDENCY, OQ-C8-RECIPE-SCHEMA, OQ-C8-GRAPHTABLE-GRAMMAR. Downstream SAP-1:
  desugar-decision / AS OF KNOWN audit / injected-window events may need BC-2.16.002 catalog
  rows (flagged, NOT actioned). Proposed epic: E-PRISMQL-GRAMMAR-001.

- **C9 Config Management FULLY DECIDED + CAPTURED 2026-06-27 (human). All three open questions
  (Q1 authority/versioning, Q2 canary mechanics, Q3 schema-versioning/deployment-awareness)
  RESOLVED.** Seven architecture decisions confirmed: Q1-AUTHORITY (DB-authoritative, UI-only
  authoring in production, no hand-edited TOML runtime path); Q1-VERSION (versioning split by
  domain — runtime-config = DB-native temporal/system-versioned history with in-transaction
  exactly-once semantics; detection-content + recipes = real embedded git via git2 0.19.0,
  opt-in residency-gated remote for detection content only, off/air-gap-safe by default; optional
  async git projection of runtime-config history = nicety only, not authoritative); FAST-REVERT
  (ArcSwap hot-swap, append-only/forward-only, seconds, no restart, satellites self-revert +
  pick up on next dial-home, anchors canary auto-rollback; applies to hot-reloadable config
  only — NOT restart-class/bootstrap keys); APPROVAL (dropped to DAY-3; canary + fast-revert
  is the day-2 safety gate); BOOTSTRAP (4-layer for restart-class keys: (1) validate-before-persist
  cheap-only / port-bindable+store-connects RACY → boot-time backstop; (2) A/B dual-slot
  active=last-known-good / pending=new / promote only after readiness probe; (3) supervisor
  watchdog N-failed-boots → revert+reboot / sd-notify 0.5.0 mature / bundled-PID-1 0.x flag
  for maturity check at morph; (4) satellite autonomous self-recovery TIERED — Tier-1
  local-validation-fail → revert; Tier-2 dial-home-fail → ESCALATION not revert, locally-healthy
  satellite does NOT flap on network partition); wrapped by fleet-staged canary (Azure-Device-
  Update-style); safe-mode console = new attack surface → security-reviewer required before ship;
  NIST-800-82/IEC-62443 = separate standards pass.
  **Q2 CANARY (three sub-decisions):** Q2-HEALTH = soft regressions INCLUDED (coverage-banner
  drop §3.6, availability-cache degradation, query error-rate uptick, empty-result-rate climb,
  normalization-failure rate) PLUS hard failures at CONSERVATIVE threshold; trip CORRELATED to
  this-config-push-hitting-this-cohort (upstream outage must not be misread as bad config);
  reuses C6 shared CUSUM/ADWIN change-detector primitive. Q2-COHORT = config-scope-dependent
  (tenant for tenant-scoped config; satellite/site for fleet-distributed config). Q2-TIERS =
  TWO-TIER: HIGH-BLAST classes (connector-defs, pushdown-descriptors, retention-policies,
  satellite-trust, detection-rule production promotion) ALWAYS canary; LOW-BLAST (feature flags,
  log-level, TTLs, UI config) apply directly + fast-revert available; classification locked at
  config-type level, NOT value/magnitude level.
  **Q3 SCHEMA-VERSIONING (four sub-decisions, RESOLVED 2026-06-27 human-confirmed):**
  **Q3-MODEL = HYBRID + per-domain split + HUB-AND-SPOKE.** Additive-forward-compat by default
  (serde `#[serde(default)]` + tolerant deserialization) covers the additive majority with zero
  migration code. Explicit per-domain `schema_version` + ordered idempotent migration chain for
  breaking changes only. ONE migration-runner abstraction, N independent per-domain version
  registries, HUB-AND-SPOKE conversion (one canonical current schema per domain; 2×(N-1) total
  functions vs quadratic all-pairs — Kubebuilder pattern). Skip-version-RELEASE supported; skip-
  migration-STEP forbidden. Per-domain: runtime-config → migration chain; detection-content +
  recipes → git IS version axis + thin content schema_version; RocksDB hot data → per-CF
  `__schema_meta__` key + on-open chain (CF-per-version REJECTED, rust-rocksdb #608); Iceberg +
  OCSF cold tier → additive (C5 decision) + Iceberg column-id evolution; OCSF version per
  partition; NO proprietary version axis for cold tier. KEY CORRECTION: `#[non_exhaustive]` is a
  COMPILE-TIME cross-crate API guardrail with ZERO effect on serialization compat — NEVER cite it
  as the mechanism that makes skip-version safe; the serialization-compat story is carried entirely
  by serde `#[serde(default)]` + the explicit migration chain. **Q3-SKIP = bounded window + LTS
  required-stops.** Mechanism built now; exact supported window (K minors) + required-stop cadence
  = OPEN BUSINESS DECISION set at GA. Testing posture non-negotiable: golden fixture per released
  version per domain + round-trip + forward-migration tests + upgrade-matrix CI across
  supported-window skip-pairs. **Q3-FORMAT = stay serde 1.0.228 + RocksDB value bytes.** Additive
  evolution patterns (`#[serde(default)]`, internally-tagged version enums, alias/rename) + value-
  level version tag. savefile 0.20.4 REJECTED as default (reserve for measured perf need).
  serde_version 0.5.1 REJECTED (abandoned/nightly-only). **Q3-TIMING = synchronous at boot**
  (Grafana-style; config volume small; pairs with A/B dual-slot validate-before-cutover + watchdog;
  RocksDB on-open migration runs synchronously before handle returned, idempotent + atomic per step
  via `write_batch()`). **DEPLOYMENT-AWARE:** SaaS = walks every release, chain barely exercised,
  blue-green rollback; MSSP-managed = bundle carries full chain, A/B slot validates, watchdog
  covers boot-bricking migration; client-managed = HIGHEST skip-version exposure, full chain +
  supported-window + required-stop + golden-fixture CI + idempotent/atomic/resumable on-open.
  FULL three-operating-model deployment matrix has its OWN forthcoming ADR-PROP-dual-deployment.md
  — C9 captures only the migration-posture slice.
  Capture artifact: `specs/day2-design-decisions/ADR-PROP-config-management.md`
  (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
  `research/config-schema-versioning-migration-2026-06-27.md` (Q3 PRIMARY),
  `research/config-management-depth-2026-06-27.md`, `research/config-authority-narrow-git-2026-06-27.md`,
  `research/git-as-primary-vs-write-behind-2026-06-27.md`, `research/bootstrap-config-recovery-2026-06-27.md`.
  Downstream SAP-1 obligations (NOT actioned): config-generation-written, config-generation-reverted,
  config-canary-trip, config-canary-rolled-back, config-migration-completed, config-migration-step-failed,
  config-satellite-reverted events each need new BC-2.16.002 Canonical Structured Event Catalog rows.
  Open items: OQ-C9-1 (git2 vs gix), OQ-C9-2 (skip-version window + LTS cadence business decision),
  OQ-C9-3 (bundled-PID-1 selection), OQ-C9-4 (savefile opt-in measured), OQ-C9-5 (NIST-800-82/IEC-62443
  standards pass), OQ-C9-6 (safe-mode console security review), OQ-C9-7/8 (calibration).

- **DEPLOYMENT MATRIX (cross-cutting, three-operating-model) FULLY DECIDED + CAPTURED 2026-06-27 (human).**
  Six architecture decisions confirmed: D-DEPLOY-001 (single-codebase + runtime DEPLOYMENT-PROFILE
  ~90% shared; divergent-fork anti-pattern explicitly rejected, validated by GitLab/Sentry/Elastic/
  Mattermost/GitHub/Grafana prior art); D-DEPLOY-002 (THREE named operating models on TWO AXES —
  WHO HOSTS × WHO OPERATES: **SaaS** = vendor-hosted + vendor-operated + multi-CUSTOMER tenancy;
  **MSSP-managed** = customer/MSSP infra + MSSP-operated + multi-CLIENT tenancy; **Client-managed**
  = client infra + client-SOC-operated + single-org or internal-BU tenancy); D-DEPLOY-003
  (operator-role as PROFILE DIMENSION not build target; RBAC defaults + day-3 workflow defaults vary
  by operator; no `#[cfg(feature = "mssp")]` or equivalent compile-time gate);
  D-DEPLOY-004 (uniform `OrgId`/`OrgSlug`/`OrgRegistry` abstraction across the full single-org →
  multi-BU → multi-client → multi-customer spectrum; NO super-tenant layer above OrgId);
  D-DEPLOY-005 (**BYOC ZERO-ACCESS BY CONSTRUCTION — HEADLINE DIFFERENTIATOR**: satellite mesh IS
  the BYOC data-plane by construction; C2 residency invariant + AD-017 satellite-local credentials
  → SaaS central NEVER receives raw data or creds; egress-blocked CI invariant = STANDING GUARD
  across all three models; thesis STRENGTHENED across all operating models — client-managed is the
  purest air-gap expression); D-DEPLOY-006 (deployment-conditional C9 migration posture cross-ref
  only — see ADR-PROP-config-management.md §D-C9-Q3-DEPLOYMENT: SaaS = blue-green chain-barely-
  exercised; MSSP-managed = bundle carries full chain + A/B validates; client-managed = HIGHEST
  skip-version exposure, full chain + required-stop + golden-fixture CI + idempotent/atomic/resumable).
  **A client-managed deployment CAN admit MSSP oversight WITHOUT re-architecting** (operator-role is
  a profile config change, not a codebase change — PIV-DEPLOY-007). Config-authority model (DB +
  UI + git-backed) is IDENTICAL across all three; what varies by operator = RBAC defaults + day-3
  workflow defaults (PIV-DEPLOY-006).
  **OPEN sub-choices (NOT resolved — explicitly flagged):** (1) **OQ-DEPLOY-1 tenancy-isolation depth**
  (pool / bridge / silo / cell-per-customer) — open architectural sub-choice needing a targeted
  morph decision; compliance/cost implications for SOC 2 / GDPR; (2) **OQ-DEPLOY-2 residual BYOC
  hardening gaps** — four open hardening items: result-transit residency policy, metadata-leakage
  audit, ephemeral dial-home token rotation cadence, CMEK for central metadata; NOT blocking
  architecture but must close before SaaS launch.
  Capture artifact: `specs/day2-design-decisions/ADR-PROP-dual-deployment.md`
  (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
  `research/dual-deployment-saas-onprem-2026-06-27.md` (PRIMARY). Cross-refs:
  ADR-PROP-config-management.md (D-C9-Q3-DEPLOYMENT migration posture);
  ADR-PROP-central-deployment-access-layer.md (C1); ADR-PROP-satellite-mesh.md (C2 residency +
  AD-017 satellite-local creds); secret-subsystem-sketch.md (SS-26); matured-vision §3.1/§3.2.
  Ripple effects: E-BUNDLE-DEPLOY-001 (offline bundle tooling epic — not yet registered);
  OQ-DEPLOY-1/2 require targeted morph passes; SS-26 morph ADR must cite PIV-DEPLOY-004.

- **C10 Query.io Competitive Refresh & Positioning Gap-Check DISCUSSION COMPLETE + CAPTURED
  2026-06-27 (human).** All eight gaps from `research/queryio-competitive-refresh-2026-06-27.md`
  addressed (no conscious declines); C3 join framing corrected; identity-vs-differentiation split
  recorded; honest concessions locked (binding per §2.4); positioning headline leading candidate
  recorded (ratification deferred to B capstone / §5.1 brief-reframe). Capture artifact:
  `specs/day2-design-decisions/ADR-PROP-competitive-positioning.md`
  (`do_not_execute: true`; real ADR numbers deferred to morph).
  **D-C10-1 C3 JOIN FRAMING CORRECTION (load-bearing):** The research's §2a/§4.3 mischaracterized
  Prism as "hard-rejecting unbounded cross-source joins." CORRECTED: C3 (D-C3-1,
  `ADR-PROP-capability-descriptor-pushdown.md`) is a **cost-based-DEGRADE stack**, not a
  plan-time rejection. All join shapes permitted; bounded at execution time via per-side row-caps
  + dynamic filtering + EXPLAIN-visible pushdown + injected time-window. Correct positioning claim:
  "safe, cost-guarded cross-source joins with plan-visible degradation" — a Prism STRENGTH vs
  Query's cost-guard-absent "translate-and-pray." 3 pre-implementation residuals (PIV-C3-1..3) in
  C3 capture artifact — do not let residuals reintroduce the false "hard-reject" framing.
  **D-C10-2 IDENTITY vs DIFFERENTIATION:** Agent-native is the product IDENTITY (what Prism is)
  but is now PARITY territory (Query Workers + MCP + A2A GA 2026-03). The DEFENSIBLE WEDGE =
  OT/edge/air-gap satellite mesh (structural; Query has no answer) + AI-opaque trust layer
  (AD-017 + SS-26; no Query equivalent). Do NOT claim "first/only agent-native."
  **D-C10-3 HONEST CONCESSIONS (binding per §2.4):** concede shipping maturity, connector
  breadth, and SaaS time-to-value to Query ("Query ships; Prism plans"); DO NOT concede
  OT/air-gap moat, AI-opaque trust, formal PrismQL, or (corrected) cost-guarded cross-source
  joins.
  **D-C10-4 ALL EIGHT GAPS ADDRESSED (no declines):** GAP-Q1 (OOTB detection content + rule
  translation OUT → E-DETECTION-CONTENT-001 + E-RULE-XLATE-001 expansion); GAP-Q2 (auditable
  S3-agent evidence package + self-QA gate → E-EVIDENCE-PACKAGE-001); GAP-Q3 (A2A protocol →
  ADD to day-2 transport scope alongside MCP, E-A2A-TRANSPORT-001, cross-ref C1); GAP-Q4
  (connector-egress / Security Data Pipelines analog → E-EGRESS-PIPELINE-001, DISTINCT from
  internal RETAIN); GAP-Q5 (Amazon Security Lake subscriber pattern → fold into C5
  E-LAKE-CONNECTOR-001 scope, D-C5-2 already covers this); GAP-Q6 (alert-destination fan-out +
  severity routing → expand E-ALERT-ROUTING-001 into C6); GAP-Q7 (graph-investigation views +
  dashboards → E-GRAPH-INVESTIGATION-001, Cytoscape/ECharts already chosen); GAP-Q8 (onboarding
  → ADDRESS BOTH HALVES: Configure-Schema wizard E-CONFIGURE-SCHEMA-WIZARD-001 + optional
  vendor-hosted managed-mapping E-MANAGED-MAPPING-001 for SaaS-model only).
  **D-C10-5 HEADLINE LEADING CANDIDATE:** "the agent-native federated query platform for the data
  Query can't reach — OT/edge/air-gap — with credentials the AI never sees." NOT locked; ratified
  at B capstone / §5.1.
  Open questions: OQ-C10-1 (product-fact re-verification before any external claim — RBAC status
  highest decay); OQ-C10-2 (final headline at §5.1); OQ-C10-3 (landscape [INCONCLUSIVE] re-verify
  before B: Anvilogic/Matano/Vega/Tenzir depth); OQ-C10-4 (C3 PIV-1..3 pre-impl verification);
  OQ-C10-5 (A2A spec version pin before E-A2A-TRANSPORT-001 morph). Proposed epics: 10 new
  (table in capture artifact). SAP-1 downstream obligations: 5 new BC-2.16.002 event-catalog rows
  flagged in capture artifact (morph-time).

- **C11 Prism Intel (Threat-Advisory / Vulnerability-Intel Add-On) FULLY DECIDED + CAPTURED
  2026-06-27 (human). Eight architecture decisions confirmed: D-C11-1 (FEED-DOWN MATCH-AT-EDGE —
  central aggregates+normalizes CVE/CPE/KEV/EPSS/CSAF-VEX and ships corpus DOWN; satellite/edge
  joins to Entity 360 LOCALLY; central NEVER receives raw asset identifiers BY CONSTRUCTION —
  the BYOC zero-access differentiator; no surveyed full-stack VM vendor — Tenable/Qualys/Rapid7/
  Nucleus/runZero — stays blind to inventory); D-C11-2 (OPT-IN CENTRAL-MATCH — edge-match is
  DEFAULT everywhere; consent-gated central-match carve-out for non-BYOC SaaS customers who
  explicitly waive zero-access for cross-tenant analytics; the ONLY inventory-leaves-edge path;
  PIV-C11-004 governs); D-C11-3 (PRIVACY MECHANISMS — full public CVE/CPE corpus shipped down,
  no crypto needed; HMAC-keyed hashed indicators + Bloom filters reserved for large IOC feeds or
  indicator-obfuscation contracts only; PSI EXPLICITLY REJECTED as default — over-serves one-sided
  privacy, no mainstream SOC production precedent as of 2026); D-C11-4 (ADVISORY GENERATION AT
  EDGE — `priority = f(KEV, EPSS, CVSS v4, asset_exposure, asset_criticality, compensating_controls)`;
  KEV-listed + high EPSS = "act now"; VEX "not affected" demotes false positives; computed at edge
  with local context; standards: FIRST CVSS v4.0 2023-11-01, FIRST EPSS daily, CISA KEV BOD-22-01,
  NVD CVE API 2.0, CPE 2.3, OASIS CSAF 2.0/VEX ISO/IEC 20153 2025-05-20, SSVC decision-support);
  D-C11-5 (AIR-GAP = REUSE C9 SIGNED-BUNDLE — Ed25519/sigstore-signed delta tarball over the
  SAME C9 offline-signed-bundle mechanism, same key custody + integrity verification + delta
  discipline, NOT a parallel trust path; precedent: Tenable Nessus offline signed plugin feed);
  D-C11-6 (METERING = DEPLOYMENT-CONDITIONAL ZERO-ACCESS-PRESERVING — air-gap/on-prem =
  license-entitlement cap, nothing phoned home; SaaS/online = edge-reported AGGREGATE count,
  NEVER asset-level telemetry; meter dimension = asset count in Entity 360);
  D-C11-7 (PACKAGING = SEPARATE ANNUAL SKU — free baseline: NVD/CVE+CPE+KEV+EPSS+CSAF/VEX
  public feeds already actionable; paid premium: commercial/curated feeds + analyst-validated
  per-CVE advisories [asset-AGNOSTIC, ship down clean] + VEX suppression; pricing dimension =
  asset count); D-C11-8 (DOWN-FEED STANDARDS = custom signed-delta HTTPS API [online] + C9 signed
  bundle [air-gap] + CSAF/VEX-native ingestion; TAXII 2.1 optional interop only).
  **Entity 360 + ARO integration:** advisories decorate entity at asset→exposures→advisories→
  recommended-actions; Observation ("asset X runs Y vV; CVE-Z; KEV=true; EPSS=0.74") →
  Recommendation ("patch/compensating control; priority=critical") → Action (gated C15, never
  auto-fired). **C20 relevance:** advisory priority conditioned on Purdue zone (C12 D-C12-6 OT
  zone attribute); ties NERC CIP-010 vulnerability assessment.
  **INVARIANTS (PIV-C11-001..007):** central never receives raw asset IDs (001); corpus is public
  + signed (002); metering never emits asset-level telemetry (003); opt-in central-match is the
  ONLY inventory-leaves-edge path + requires recorded consent (004); air-gap reuses C9 trust path,
  no parallel path (005); advisory priority computed at edge (006); analyst advisories are
  asset-agnostic at authorship (007).
  **Open questions:** OQ-C11-1 (commercial feed partners + OEM terms: RF/GTI/MISP);
  OQ-C11-2 (analyst-advisory authorship pipeline); OQ-C11-3 (opt-in central-match consent/
  governance + cross-tenant analytics scope); OQ-C11-4 (TAXII 2.1 interop demand);
  OQ-C11-5 (metering enforcement posture per deployment).
  Downstream SAP-1 obligations (NOT actioned): intel.feed.bundle.ingested,
  intel.advisory.generated, intel.feed.bundle.rejected events → BC-2.16.002 at morph.
  Capture artifact: `specs/day2-design-decisions/ADR-PROP-prism-intel.md`
  (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
  `research/prism-intel-threat-advisory-2026-06-27.md` (Q1–Q6; 8 MCP tool calls; vendor
  landscape, privacy-preserving matching, risk-scoring standards, feed sourcing, packaging).
  Cross-links: C12 (Entity 360 substrate), C15 (ARO model), C9 (signed-bundle), C20 (OT/NERC
  CIP-010), AD-017 (zero-access extends to asset inventory by construction), deployment matrix
  (opt-in central-match + metering deployment-conditional). Proposed epic: E-PRISM-INTEL-001
  (feeds B).

- **C12 Prism Context (KG + Vector + Entity 360) DECIDED + CAPTURED 2026-06-27 (human).** Six
  architecture decisions D-C12-1..6 confirmed. Capture artifact:
  `specs/day2-design-decisions/ADR-PROP-prism-context.md`
  (`do_not_execute: true`; real ADR numbers deferred to morph). Research basis:
  `research/prism-context-kg-vector-2026-06-27.md` (3× perplexity_research sonar-deep-research +
  1× perplexity_ask; 12 live crates.io version-verifications 2026-06-27). Aletheon spike
  (`spike/init-db.sql` + `spike/docs/aletheon-vision.md`) read and cited for AGE+pgvector design
  + control/process edges + ARO model + institutional-memory thesis.
  **Overall: a two-layer, embedded, air-gap-first Context engine on the OCSF-normalized layer,
  per-tenant isolated.**
  **D-C12-1 STORAGE = TWO-LAYER MAINTAINED + TIERED.** Graph = `indradb` 5.0.0
  (RocksDB-backed, co-located with existing ~19 CFs, MPL-2.0, actively maintained 2025-08).
  Vector = `usearch` 2.25.3 hot in-memory ANN with INT8/binary quantization (512MB/200MB budget)
  + `lancedb` 0.30.0 on-disk COLD tier (maps to hot→Iceberg-cold pattern). REJECT `cozo` 0.7.6
  (last published 2023-12-11, ~2.5 yr stale) — record considered-and-rejected; OQ-C12-4 for
  future re-check if upstream resumes.
  **D-C12-2 EMBEDDINGS = ON-BOX IN-PROCESS.** `fastembed` 5.17.2 on `ort` 2.0.0-rc.12 (shared
  with C7 ModelBackend D-C7-2) DEFAULT; `candle` 0.11.0 pure-Rust fallback for air-gap audit /
  avoid ort-RC. Final model (BGE-small / all-MiniLM / nomic-embed / multilingual-E5) via
  perf/recall benchmark (OQ-C12-1). Models pre-staged for air-gap; raw telemetry vectorized
  in-process, NEVER transits AI context (AD-017 + C16) — PIV-C12-2 invariant.
  **D-C12-3 ENTITY-RESOLUTION = DETERMINISTIC-ONLY AUTO-MERGE + SUSPECTED-LINKS.** Auto-merge
  ONLY on strong IDs (SID/UUID/MAC); weak/fuzzy = `suspected` edges NEVER auto-merged;
  temporal validity intervals on identity edges (DHCP); strictly per-tenant. Security-reviewer
  sign-off required (PIV-C12-4).
  **D-C12-4 RETRIEVAL = PERPLEXITY-STYLE HYBRID MULTI-STAGE + MANDATORY CITATIONS.** Lexical/
  structured filter → graph-neighborhood expand → vector similarity → re-rank (severity /
  criticality / TI-credibility / recency) → LLM synthesis. Mandatory inline citations (claim →
  OCSF event ID / rule / asset). Route via C7 ModelBackend (fast vs reasoning).
  **D-C12-5 GRAPHRAG = PHASED.** Phase 1 = local-search (entity-neighborhood + vector = Entity
  360 query) SHIPS FIRST. Phase 2 = full GraphRAG global community-summarization (Hierarchical
  Leiden + LLM community-summaries) for corpus-wide incident/campaign clustering — COMMITTED
  (not just deferred), cost acknowledged; recompute cadence OQ-C12-2.
  **D-C12-6 DEPLOYMENT = EMBEDDED AIR-GAP-FIRST, UNIVERSAL DEFAULT; per-tenant partitioned.**
  `indradb` + `usearch` + `lancedb` are all embedded. Server-backed option NOT built
  speculatively. **DEFERRED CENTRAL-TIER OPTION (record explicitly): Apache AGE (graph) +
  pgvector co-resident on Prism's ALREADY-BUNDLED PostgreSQL** is the recorded concrete
  server-backend escape-hatch for the CENTRAL deployment tier ONLY — revisit if a concrete need
  emerges; does NOT change the embedded edge/satellite decision. Aletheon spike validates
  the AGE+pgvector single-store pattern.
  **ENTITY 360 EXPANSION (7-part view):** identity panel (canonical + aliases + binding
  confidence); timeline (normalized OCSF activity, time-windowed identity); relationship graph
  (host↔user↔IP↔process↔alert↔asset, typed+temporal edges — INCLUDING control/process edges
  CONTROLS/MONITORS/CONNECTS_VIA from aletheon); explainable risk score (cited events);
  exposures (vulns/misconfig/unpatchable); related findings/similar entities (vector);
  operational/business-context edges (asset→process→mission) AND **Purdue-level/network-zone
  (ot_level_0/1/2/dmz/it) as a first-class entity attribute** (OT relevance, ties C20 NERC CIP).
  **ALETHEON SPIKE CAPTURE (CORRECTED — it DOES have a spike memory design):**
  Apache AGE + pgvector in ONE PostgreSQL; OT asset graph (`create_graph('ot_assets')`);
  graph edges = control/process relationships (CONTROLS, MONITORS, CONNECTS_VIA), NOT just
  network; asset nodes carry criticality/location/`network_zone=Purdue level`;
  `assets.description_embedding vector(1536)` (semantic search on assets); `events` table
  (normalized telemetry). `aros` table (Actions/Recommendations/Observations with dual-audience
  text, AI provenance, confidence/model_version, source_event_ids[], ack/resolve workflow) is a
  **DIRECT REFERENCE INPUT FOR C15 (SOAR ARO model) — NOT captured here; cross-linked to C15.**
  Institutional-memory thesis ("the system learns your environment, your patterns, your
  preferences") = the C12 product framing. What Prism does NOT pull: aletheon's cross-client
  "community defense" (collides with per-tenant isolation + AD-017).
  **INVARIANTS (PIV-C12-*):** in-process on-box embeddings (raw telemetry never transits AI
  context); strictly per-tenant graph/vector/resolution (no cross-tenant edges/similarity);
  auto-merge only on strong identifiers; temporal validity on identity edges; every LLM-surfaced
  claim carries an OCSF-event/rule/asset citation; embedded single-binary works air-gap with
  pre-staged models.
  **Open questions:** OQ-C12-1 (embedding-model benchmark); OQ-C12-2 (GraphRAG Phase 2
  recompute cost/cadence); OQ-C12-3 (AGE+pgvector central-tier evaluation trigger);
  OQ-C12-4 (cozo upstream liveness check); OQ-C12-5 (usearch boot-time index load strategy).
  Cross-links: C7 ModelBackend (ort shared), C1 storage taxonomy (lancedb cold tier),
  C11 Prism Intel (Entity 360 parts 4+5), **C15 (ARO model — aletheon aros table banked)**,
  C20 (Purdue-zone/OT), AD-017, C16 masking, S3 agent runtime.
  Proposed epic: E-PRISM-CONTEXT-001 (feeds B).

- **C13 §16.4 open-items closeout COMPLETE 2026-06-27 (human).** All residual open items from
  the six §16.4 capture sketches resolved. Key resolutions applied to files under
  `specs/day2-design-decisions/`:
  - **s3 OD-1 (conversation history):** server-side per-tenant-DEK-encrypted conversation store
    from day one; NOT browser-only. Configurable retention; feeds C10 GAP-Q2 evidence package.
  - **s3 OD-2 (model budget):** per-tenant token+cost accounting with SOFT warning (configurable
    %) then HARD cutoff at 100%. Both enforcement stages required.
  - **secret HD-1:** built-in encrypted store = DEFAULT (SoftwareKms, air-gap/BYOC-first);
    external KMS = pluggable opt-in backend.
  - **secret HD-2:** AES-256-GCM = default (FIPS-140/AES-NI/NERC-CIP-friendly); ChaCha20-Poly1305
    = optional fallback (no-AES-NI environments).
  - **secret HD-3:** automatic SCHEDULED DEK rotation (configurable interval) + on-demand manual
    override. Not manual-only.
  - **secret HD-4:** per-tenant DEK envelope day-2; per-credential DEK = FUTURE ENHANCEMENT
    (OQ-SECRET-DEK-GRANULARITY) — finer blast-radius isolation, heavier key mgmt; NOT day-2 scope.
  - **secret HD-5:** satellite holds credentials FULL-LOCAL (C2 + AD-017 BYOC zero-access —
    central never holds satellite creds); central-vend-then-cache = managed-mode option only
    (per deployment-profile).
  - **ml OD-3 (Phase 3 inference engine):** RESOLVED via C7 reconciliation — not a single pick;
    it is the pluggable `ModelBackend` from C7 ADR-PROP: candle + ort/ONNX + wasmtime-WASM +
    tract. Single-engine framing superseded.
  - **ml recommendation-level items (OD-1/OD-2/OD-4/OD-5):** all ACCEPTED — graceful
    degradation to live-only baselines if cache not yet merged (OD-1); bounded-update-rate AND
    anomaly-gated learning both in scope from Phase 2 day one (OD-2); Phase 3 defined narrowly,
    feature store = follow-on E-ML-FEATURE-STORE-001 (OD-4); scheduled sampling configured
    per-detection-rule initially (OD-5).
  - **sso recommendation-level items (OD-3/OD-4/OD-5):** all ACCEPTED — JIT provisioning IN
    scope separate from SCIM (OD-3); TOTP for Prism-local accounts + IdP-enforced MFA for SSO
    (OD-4); 8h max + 30min idle session defaults, Tenant-Admin configurable (OD-5). OD-2
    (SAML native-Rust vs Dex sidecar) remains IMPLEMENTATION-TIME architect evaluation
    (genuinely deferred — outcome depends on adversarial review of `samael` at impl time).
  - **prismql HD-2 (raw MATCH_RECOGNIZE escape hatch):** ACCEPTED — YES expose it, lower
    priority / later phase within Phase A.

- **Architecture Design System (ADS) PRODUCED 2026-06-27 (day-2 side-analysis).** Canonical
  cross-cutting architecture reference for all Day-2 features (C1–C20 + B capstone) — the
  architecture-tier analog of the UI design system. Seeded from the ripple audit
  (`research/central-surfacing-ripple-analysis-2026-06-27.md`). Artifact:
  `specs/day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md` (`do_not_execute: true`).
  Provides: 12 Principles (P-ADS-01..12), 11 Patterns (PAT-ADS-01..11), 8 cross-cutting
  Invariants (INV-ADS-01..08), per-feature Conformance Checklist, 10 Anti-Patterns, and a
  punch-list of the six non-conformances already identified in the ripple audit (Section C.3).
  All C1–C20 + B feature ADR-PROPs must pass the conformance checklist before morph promotion.
  The conformance pass (next step) patches the P0/P1 non-conforming ADR-PROPs identified in
  the ripple audit.

### 16.5 Status & boundaries reminder

- This is a **CAPTURE artifact** (`do_not_execute: true`). Nothing here modifies the live brief/PRD/
  BC/architecture/stories. Day-2 morph begins post-demo, post-T14, gated on brief-reframe sign-off
  (§5.1).
- The live factory (Phase 3 grammar-remediation cascade) ran **independently** throughout this side
  session and continues; its branch advanced past our commits with no collision. Do NOT confuse §10–§16
  day-2 vision work with the active demo/cascade workstream.
- Epic IDs introduced in §10–§15 (E-CACHE-DEMAND, E-CENTRAL-*, E-SATELLITE-MESH, E-LAKE-CONNECTOR,
  E-UI-*, E-CONNECTOR-DYNAMIC, E-DETECT-*, E-ALERT-ROUTING, E-RULE-XLATE, E-ML-*) are PROPOSED, not yet
  registered in STORY-INDEX; gaps G-1…G-26 track the new findings.

---

## Section 17 — Federated Ingestion: The Collector / Stream-Connector Class

> **PROVENANCE.** 2026-06-26 side-analysis capture. PROPOSED; `do_not_execute: true`; gated on
> brief-reframe sign-off (§5.1). Sources: `research/federated-ingestion-collector-connectors-2026-06-26.md`
> and `research/chain-cache-tiering-replication-deadlines-2026-06-26.md`.

### 17.1 Concept and the "prism way" reframe

Federated ingestion of push/stream/capture data follows a single pattern: **collect at the edge →
OCSF/native normalize at the boundary → demand-driven TTL'd buffer (RetentionCache) → expose as a
queryable source.** The reframe that keeps the federation thesis intact is:

> **A collector is a SOURCE and a buffer — NOT a central sink.**

"No ingestion" has never meant "never persist a byte." The RetentionCache (§3.3) already buffers
demand-pulled data; a collector extends the same buffer with a *receiver endpoint* bolted on the front.
Raw data lands at the edge node (the Satellite hosting the receiver), gets normalized at the
boundary, is TTL'd per the retention policy, and surfaces as `FROM cache.<collector>` — central Prism
stays pull-based and ephemeral against that source.

**Cribl neutral-incentive analogue.** Cribl's "no ingest-priced analytics backend" stance means
aggressive edge reduction is a *selling point*, not a conflict — the same incentive structure Prism
claims (sensor-API-native, no ingestion revenue). The "collect-and-reduce at the edge, route only
the reduction" posture reinforces, not dilutes, the federation thesis. Prism can use the identical
reframe: a collector is a telemetry *service*; ingestion is the act of loading into a priced
analytics store. Prism never does the latter.

### 17.2 The four-stage abstraction and collector-as-subtype

Every mature edge-pipeline product (Cribl Stream/Edge, Vector, Fluent Bit/Fluentd, Logstash/Beats)
implements the same spine:

| Stage | What it is | Prism mapping |
|-------|-----------|---------------|
| **1. Receiver / listener endpoint** | Externally addressable ingress the source pushes to | **GAP — the only genuinely new primitive** |
| **2. Local buffer (store-and-forward)** | Bridges bursty arrival against controlled downstream processing | Satellite store-and-forward + RetentionCache hot tier (§3.2, §3.3) |
| **3. Boundary normalization** | Wire format → OCSF or native schema-on-read | Connector taxonomy §3.4 + multi-schema §13.6 |
| **4. Queryable / forwardable surface** | Where detection/analytics/query runs | PrismQL `FROM cache.<name>` + detection-as-query §14 |

Prism already has stages 2–4. **The only genuinely new primitive is stage 1 — a receiver/listener
endpoint** — plus the consequences of accepting push semantics (no backpressure to the source, loss
handling, store-and-forward durability).

**DECISION-LEAN: model collector as a connector SUBTYPE** — specifically a `collector-connector`
sitting alongside `sensor` and `pull-connector` in the existing taxonomy (§3.4). This subtype
shares the capability-descriptor, RetentionCache, and Satellite machinery. It does NOT become a
separate component class, because 3 of 4 abstraction stages are pure reuse. A collector's
capability descriptor declares "push source, no pushdown, buffer-backed" — a new descriptor class
that gives the PrismQL join-guard (§12.2) the information it needs to reason about a no-pushdown
surface.

### 17.3 The collector source class

The six push/stream/capture source families, with their backpressure characteristics:

| Source | Receiver endpoint | Backpressure | Buffer regime | Federatability |
|--------|-------------------|-------------|---------------|----------------|
| **Syslog (UDP/RFC 3164)** | UDP port 514 | **None — silent drop** | NIC ring + socket buffer | Low — loss is intrinsic to transport |
| **Syslog (TCP/TLS/RELP)** | TCP port 514/6514 | TCP flow-control (weak); RELP per-message ack (strongest) | rsyslog/syslog-ng disk queue | Medium — RELP gives reliable delivery |
| **NetFlow v9 / IPFIX (RFC 7011)** | UDP (e.g. 2055) | **None — silent drop**; IPFIX adds TCP option | Socket buffer + template store | Low — template loss → misparse |
| **sFlow** | UDP | None | Socket buffer | Low — sampled; scale by sample rate |
| **Kafka topic** | Consumer-group subscription | **Consumer lag — data waits, no drop** within configured retention | Durable partitioned log | **High — replayable, offset-addressable** |
| **Webhook / HTTP-push** | HTTP endpoint | HTTP 2xx ack; sender retries → **must be idempotent** (event-ID dedup) | App-level queue | Medium — ack+retry, dedup needed |
| **S3 / object-drop** | SQS queue / SNS sub on PutObject | **SQS managed queue** (at-least-once, visibility timeout) | SQS durable queue + S3 object | Medium — notify-then-pull, durable |
| **pcap** | NIC promiscuous capture | No external backpressure | Disk-bounded rolling packet buffer (Arkime model) | Metadata high; raw packets: separate regime |

**Backpressure spectrum:** UDP (silent drop — worst) → HTTP/SQS (ack + retry) → Kafka (lag, no
loss — best federatable). This spectrum is the key engineering variable for each collector subtype.

Push data **must land before it can be queried** — this is intrinsic to push transport, not an
implementation choice. That landing buffer is the RetentionCache. The reframe holds.

**LEAN: Kafka-first to prototype the abstraction.** Kafka is the most federatable push source —
replayable, offset-addressable, consumer-lag-not-loss. Prototyping the `collector-connector` subtype
against a Kafka topic validates the abstraction with the best-behaved source before tackling UDP
syslog or pcap capture.

### 17.4 Collection locus — per-instance, edge-first default

**Locus** = which node in the Satellite tree hosts the receiver endpoint and its buffer. This is a
per-instance property declared in the connector's configuration; there is no single universal winner.

**DECIDED 2026-06-26 (human): locus is a per-INSTANCE property; edge-first default** where
residency, volume economics, or air-gap mandate it; central deployment is valid where the source is
already cloud-native and residency is not at stake.

The three locus options converge architecturally:

- **(a) Satellite-edge** — receiver + buffer hosted by an existing Satellite node in-region, co-located
  with the data source. Strongest residency guarantee: raw never crosses a network boundary before
  normalization. Best fit for OT/ICS (Purdue-layer chaining, §3.2 #1/#2/#5), air-gapped enclosures,
  and any source with volume that would be prohibitive to egress raw.
  *OT/Purdue example:* a Level 2 satellite hosts a syslog receiver for PLC/HMI syslog on the OT VLAN;
  only OCSF Network Activity / Authentication events transit the conduit to the Level 4 Satellite; raw
  syslog stays on the OT segment.
- **(b) Standalone collector** — a dedicated "collection-capable Satellite" deployed specifically to host
  receivers, no other workloads. Architecturally this converges with (a): it is a satellite whose
  primary role is collection. Not a distinct component class.
- **(c) Central / cloud-hosted** — the root or a cloud-managed node hosts a listener for sources that are
  already cloud-native (e.g., a SaaS vendor's outbound webhook). Appropriate when residency is not
  an issue and the source has no on-prem path.

Locus (a) and (b) converge as a "collection-capable Satellite" — the Satellite gains a listener-hosting
capability. This is the key new Satellite capability the collector subtype requires.

### 17.5 Chain interaction — push lands locally, pull retrieves on query

The collector locus node owns the receiver and the local RetentionCache buffer. The rest of the
Satellite chain interacts with that buffer through the **existing inward-plan / outward-result PULL
mechanics** — a parent Satellite queries the locus node just as it would any other Satellite, using
the established per-hop deadline-decrement + partial-result propagation (§3.2, §3.6, BC-2.01.010).

**No new "stream upstream" mechanism is needed by default.** The design consequences:

- **Residency preserved by construction.** Raw push data never crosses a Satellite boundary; only
  OCSF-normalized result rows transit upward (the reduction happened at the locus). This is the same
  residency invariant as pull connectors (§3.2 #6), now extended to push sources.
- **Subtree-drop = "unreachable, not lost."** If the locus node is offline, the Satellite chain's
  existing partial-result/coverage mechanics (§3.6 CAQP / Elastic CCS lineage) surface the gap as a
  coverage annotation on the query result — never silent. Push data that arrived while the node was
  offline is in the local store-and-forward buffer and replays when connectivity restores (§3.2
  store-and-forward).
- **Deadline budget makes locus depth cost latency.** A deep locus adds chain hops; the chain-aware
  deadline model (§17.8) applies. Shallow placement (edge-first) is also the low-latency choice.
- **Storage tiers map onto topology.** Hot RocksDB at the locus edge node; warm intermediate
  materialization at a regional hub if Q1/Q2 policy places it there (§17.8); cold Iceberg at
  central. The §3.3 Retention Policy Engine runs at each node that holds a tier.
- **Satellite must gain listener-hosting to be a collection locus.** The existing Satellite abstraction
  is a remote *executor* (pull); adding receiver-hosting is the new capability that turns it into a
  collection-capable Satellite. This is a new Satellite mode, not a new component.

### 17.6 DECIDED 2026-06-26 (human): full-packet pcap retrieval is in day-2 scope

Full-packet capture retrieval (#4) adds a **second distinct storage regime**, separate from the
RocksDB-hot / Iceberg-cold tiers:

- **The regime:** a disk-bounded rolling **packet buffer** at the deepest edge node (Arkime/Moloch
  model) — PCAP files written by a capture process (`libpcap` / `AF_PACKET`), rotating on disk-fill
  or time boundary, indexed by session ID. Retention is bounded by edge disk capacity and a declared
  retention policy (typically days to weeks).
- **The queryable surface:** flow/session **metadata** (Zeek conn.log / Suricata EVE flow / Arkime SPI)
  → OCSF Network Activity (class_uid 4001). This is the federated surface: structured, OCSF-normalized,
  queryable via `FROM cache.<collector>`. Cross-tool pivoting uses Zeek UID / Suricata `flow_id` /
  Community ID as the session identifier.
- **On-demand packet retrieval:** PrismQL gains a `retrieve-packets-by-session` affordance (session ID →
  PCAP bytes or stream). S2 console gains a "download PCAP" action on a flow result row. This is a
  retrieve-blob capability, not a federated query over raw packets.
- **Distinct from the cache tiers:** the packet buffer is NOT RocksDB and NOT Iceberg. It is a disk-file
  regime with its own rotation/retention/residency governance. Conflating it with the cache tiering
  model (§3.3, §17.8) would be an architecture error.
- **Security-review surface:** a packet buffer at the edge is a sensitive data store (captures
  credentials in plaintext traffic, PII, etc.). It requires a dedicated security review covering
  access control, at-rest encryption, residency enforcement (raw never egresses), and retention
  policy governance. This is net-new security scope compared to pull connectors.
- **Normalizers:** Zeek and Suricata are the preferred session/flow metadata normalizers. Arkime/Moloch
  is the reference model for the PCAP-on-disk + SPI-metadata split. Prism does not replace these
  tools; it federates their output.

### 17.7 DECIDED 2026-06-26 (human): prism will own a continuous-operator capability — phased

Continuous/streaming detection (#5) is in day-2 scope, ordered in two phases:

**Phase 1 — NRT-over-cache (reuse existing §14 detection-as-query):**
Detection-as-query over a short-TTL RetentionCache is the Splunk real-time / Sentinel NRT lineage —
a polled window over the last W minutes. This reuses §14 as-is and is the correct starting point.
For OT/ICS edge detection, Zeek scripts and Suricata rules run inline at the capture point and emit
structured OCSF events that Prism federates — those tools are the continuous operators for their
domain; Prism federates their output.

**Phase 2 — native continuous windowed operator (ordered later):**
A native continuous operator with event-time semantics, watermarks, late-arrival handling, and
fault-tolerant state (RocksDB state backend — note alignment with Flink's RocksDB state backend).
This is what lets §12.4 `MATCH_RECOGNIZE` / `WATCH…UNLESS` run truly real-time over an arriving
stream rather than as a polled window. It is also what closes the gap that the NRT-over-cache model
cannot fully express (arbitrary stateful windowed correlation with watermark-driven window closure).

**This is the single most expensive item in the collector space** — new correlation-state machinery
(watermarks, event-time, late arrivals, fault-tolerant state checkpointing). It is ordered later as
a whole feature (Canonical Principle Rule 2 — feature ordering, not a quality shortcut within the
current cycle). The RocksDB state-backend alignment means the existing §3.3 hot tier provides a
natural home for operator state when Phase 2 arrives.

### 17.8 Chain-aware cache / replication / deadline model (Q1 + Q2 + Q3 compose into one model)

The three research questions from `chain-cache-tiering-replication-deadlines-2026-06-26.md` compose
into a single chain-aware model. Each question is summarized with its lean, then the composition is
stated.

#### 17.8.1 Q1 — Chain-aware cache tiering (LEAN: declarative policy is the authoritative floor)

Mapping cache tiers onto chain topology — RocksDB hot at the locus edge node → warm at a regional
hub intermediate node → Iceberg cold at central — is validated by prior art (CDN edge/shield/origin,
Prometheus-local/Thanos-hub/object-store, Kafka tiered local/remote, S3 Intelligent-Tiering /
Lifecycle). The topology mapping is sound.

**Correction to "automatic by default":** mature practice is **declarative envelope + automatic
optimization inside it.** The declarative per-collector policy is the **authoritative floor**:
retention duration, max tier, what may transit, residency class. Automatic age/temperature demotion
optimizes *within* that declared envelope. It may **never** cross a declared residency boundary or
move raw data to a tier that violates the policy, regardless of access temperature. For a
residency-first, regulated-data system (MSSP/OT), the automatic layer must yield to the policy
layer every time they conflict.

The "warm hub" is not a new storage regime — it is the §3.3 Retention Policy Engine running at an
intermediate node (a regional hub can hold its own RocksDB hot and optionally Iceberg cold as a
residency fan-in point). `event_time` TTL (§3.3) neutralizes eviction-race skew across tiers:
freshness is data-intrinsic, not insertion-wall-clock-relative.

**Failure modes to design against** (from CDN/HSM/ICN prior art, research §2.4):

| # | Failure mode | Prism mitigation |
|---|-------------|-----------------|
| F1 | Centralized incoherence — stale hub poisons whole subtree | Targeted purge / stale-while-revalidate with bounded windows; hub single-flight on revalidation |
| F2 | Cache stampede at the hub | Request collapsing / single-flight on hub re-materialization (Redis SET…NX pattern or Prism-native seen-request-ID dedup §3.2) |
| F3 | Double-caching (edge + hub + central same result) | `event_time`-anchored TTL as cache key component; coverage-metadata reconciliation at query time |
| F4 | Eviction races across tiers | `event_time` TTL (§3.3) makes freshness data-intrinsic; removes wall-clock cross-tier skew |
| F5 | Delete-raw-before-aggregate-exists | Never expire edge-hot record before its hub/cold materialization is durable (Thanos retention rule generalized); silent coverage loss = SOUL.md §4 violation |
| F6/F7 | Residency-as-afterthought / redact-after-forward ordering bug | Residency enforcement ordered BEFORE destination selection (§17.8.2 transform-ordering); raw-forward across a residency boundary must be inexpressible in the policy DSL |

#### 17.8.2 Q2 — Upstream replication policy language (LEAN: residency-first, per-field, ordered)

The replication policy is a declarative rule set:

```
{ selector → reduction/normalization → retention(tier) → destination(tier) → RESIDENCY }
```

**Validated primitives** (confirmed across Cribl, OTel Collector/OTTL, rsyslog/syslog-ng, S3/Iceberg
ILM, Kafka MM2, Prometheus remote-write, OPA/Rego):

- **Selector** — which records/fields route to this rule (Cribl Routes/Eval filters; OTel OTTL
  conditions; rsyslog property filters; Prometheus `write_relabel_configs`)
- **Reduction/normalization** — project/aggregate/sample/redact (Cribl Eval/Parser keep/remove;
  OTel attributes processor `delete/hash/redact`; Prometheus `labeldrop` + recording rules)
- **Retention** — TTL / RETAIN duration, tier assignment (S3 Lifecycle; Kafka tiered local-vs-remote;
  Iceberg `expire_snapshots`; Thanos retention flags)
- **Destination** — which parent tier receives the reduced output (Cribl destinations; OTel pipelines;
  rsyslog actions; Kafka MM2 replication policy)
- **Residency** — raw-stays vs metadata-only-up, per-field, evaluated BEFORE destination selection

The **residency primitive** is first-class in Prism's policy language. Every surveyed tool treats
residency as an emergent afterthought (Cribl enforces it indirectly by dropping fields before a
cross-region destination; OTel has no residency DSL; Prometheus's residency is de-facto).
**Prism making residency explicit and per-field is ahead of, not behind, the prior art** — a genuine
differentiator consistent with vision §3.2 #6 and §2.3 sovereignty.

**Three missing primitives** the survey adds (the base five-field rule is incomplete without these):

1. **Store-and-forward / buffering** — what happens to upstream-bound data when the parent is
   unreachable: drop vs memory-buffer vs disk-spool (rsyslog `ActionQueueType LinkedList` + disk
   persistence). Prism already commits to store-and-forward in §3.2; the policy language must express
   it explicitly per edge, not leave it implicit in satellite config.
2. **QoS / durability level** — how hard does this hop try to deliver upward? Maps to §3.6
   `skip_unavailable` and best-effort vs required posture. MQTT bridge QoS + rsyslog retry semantics
   are the prior-art models.
3. **Transform ordering** — filter/residency/redact steps are ordered, not a bag. **Residency
   enforcement and redaction MUST be ordered BEFORE any upstream destination selection.** If a
   route/forward step precedes a redact step, raw data has already crossed the boundary — the
   residency invariant is silently broken. The policy schema must make ordering explicit and
   machine-verifiable, not implicit.

**Hard constraint:** raw-forward across a residency boundary must be **inexpressible** in the policy
language, not merely discouraged. The type system or schema validation layer must prevent this at
policy-author time, not detect it at runtime.

#### 17.8.3 Q3 — Deadline budget model (LEAN: v1 gRPC + partial+coverage; full planner ordered later)

The fixed "mandatory intermediate cache below N hops" rule is **rejected** — it has no substantiation
in prior art (Dremel/BigQuery, Trino, Drill, Spark all determine tree depth from infrastructure
topology and data distribution, not a latency rule).

The adaptive budget-aware planner's direction is **validated**, but the full integrated planner is
research-grade assembly — not off-the-shelf. The building blocks exist separately:

- **gRPC per-hop deadline decrement** (absolute deadline → per-hop timeout by subtracting elapsed
  time, clock-skew-safe, `DEADLINE_EXCEEDED` on late calls) — available off the shelf. This is
  exactly Prism §3.2's per-hop deadline propagation mechanism.
- **Tail amplification argument for intermediate materialization** (Dean & Barroso, CACM 2013):
  `P(≥1 slow) ≈ n·p`; deeper + wider trees blow the 99th percentile. The trigger for inserting an
  intermediate materialization is **fan-out width × tail risk × remaining budget**, not depth.
- **Partial + coverage on deadline** (CAQP bounded-error-and-time, Elastic CCS `timed_out` /
  `skip_unavailable`) — validates §3.6 / BC-2.01.010 directly.
- **Hub pre-aggregate as descent shortcut** (Dremel multi-level serving tree) — an intermediate hub
  materialization (cached partial aggregate) lets the coordinator avoid descending into a deep subtree
  that cannot meet the budget.

**LEAN — production-grade v1:** per-hop gRPC deadline decrement + partial-results-on-deadline with
coverage metadata (§3.6) + opportunistic hub pre-aggregate when a popular subtree repeatedly times
out (latency-induced-probation-style, Dean & Barroso). This is production-grade on the cycle it
ships. The full cost-model-driven adaptive cache-placement planner is a whole *feature* ordered
later (Canonical Principle Rule 2 — not a quality shortcut). Hedging (duplicate to replica on
deadline) is likely N/A for Prism's residency-partitioned single-path subtrees — partial+coverage
is the deadline escape.

#### 17.8.4 Composition — one model

The three questions are not independent:

1. **Q2 policy decides Q1 placement.** The per-collector declarative rule `{select → reduce →
   retain(duration) → destination-tier → RESIDENCY}` determines which records materialize at which
   tier. Retention duration is the routing key into the tier (short → RocksDB hot; long/`RETAIN` →
   Iceberg cold). Q1 is the storage projection of Q2's policy.
2. **Q1 materializations become Q3 descent shortcuts.** A hub-tier pre-aggregate placed by Q1/Q2
   policy is exactly the intermediate materialized cache Q3's planner uses to avoid descending into a
   deep subtree that cannot make the budget.
3. **Q3 budget decides whether to use Q1 cache, descend, or return partial.** Remaining budget + per-hop
   latency + fan-out tail risk → (a) read the Q1 hub materialization, (b) return §3.6 partial +
   coverage, or (c) descend fully. The Q1 cache's `event_time` TTL feeds the coverage metadata:
   stale-but-within-window = valid coverage; expired = gap to report, never to silently swallow.
4. **Residency constrains all three simultaneously.** Q1: automatic demotion may never move raw
   across a residency boundary. Q2: residency evaluated before destination selection, ordered before
   forward. Q3: a hub materialization used as a descent shortcut must itself be residency-clean —
   the planner cannot satisfy a deadline by reading a cache that holds raw data that should never
   have transited.

**One-sentence composition:** *A declarative per-locus replication policy (Q2) projects records into
topology-aligned cache tiers (Q1) under a hard residency invariant, and a deadline-budget-aware
coordinator (Q3) reads those tiers as descent shortcuts or returns partial+coverage — never crossing
a residency boundary, never silently dropping coverage.*

**Coverage-metadata unification (LEAN):** one Prism coverage schema across the §3.6/BC-2.01.010,
CAQP, and Elastic CCS vocabularies: `{which subtrees contributed, which timed out, which tier served
the data, freshness per source, residency-clean assertion}`. Single schema, not three separate
partial-result dialects.

### 17.9 Honest strains — do not minimize

| Strain | Description |
|--------|-------------|
| **Full-take pcap volume** | Second storage regime; TB/day at 10Gbps. Not a RocksDB/Iceberg budget — requires a separate disk-bounded rolling-buffer regime with its own sizing, retention policy, and residency governance (§17.6). |
| **High-rate stream buffer budget** | A 200MB hot cap (§3.3, DC-004) is seconds of busy syslog or NetFlow. Requires edge reduction policy (drop/aggregate before buffering), per-collector buffer sizing guidance, or explicit shedding policy. "Configurable server-sized memory budget" helps but does not eliminate the tension. Never silent (SOUL.md §4): buffer-full events must surface as loss metrics. |
| **Streaming correlation state** | The continuous windowed operator (§17.7 Phase 2) is the single most expensive item. Watermarks, event-time windows, late-arrival handling, fault-tolerant state checkpointing — net-new machinery. Ordered later as a whole feature. |
| **Receiver endpoints are listeners** | A UDP/TCP/HTTP listener is new attack surface: auth, rate-limiting, DoS amplification, credential-in-payload interception. Unlike outbound pull connectors, a receiver is publicly (or network-) addressable. Dedicated security review scope — not covered by the existing pull-connector security model. |
| **Push-loss on UDP is un-fixable at the transport layer** | UDP syslog / NetFlow / sFlow have no backpressure and no delivery acknowledgement. Loss is intrinsic to the transport. Prism must: (a) expose loss metrics (`UdpRcvbufErrors`-equivalent) visibly in the coverage metadata, (b) never present UDP-sourced data as complete without a coverage annotation, (c) offer TLS/RELP / TCP upgrade paths for critical sources where loss is unacceptable. Silent loss = SOUL.md §4 violation. |
| **Idempotency / dedup for at-least-once sources** | Webhook + SQS sources retry on failure → duplicates. Event-ID based dedup required at the receiver boundary. Dedup window sizing interacts with the detection-window correlation model. |

### 17.10 Open design questions — consolidated (NOT decided)

These questions span both research docs (collector class + chain-cache model) and are consolidated
here for the human discussion. All marked NOT decided.

1. **Collector subtype vs class boundary.** The LEAN is `collector-connector` subtype sharing all
   existing machinery. Is the receiver endpoint cleanly addable to the Satellite without a new
   component class, or does the operational model (fleet management of listeners, receiver-specific
   config, distinct monitoring) justify a named `CollectorSatellite` type?
2. **Durability contract for un-ackable push.** For UDP syslog / NetFlow / sFlow: does Prism
   (a) accept best-effort + surface loss metrics, (b) require TLS/RELP/TCP for all critical sources
   and treat UDP as "optional coverage source," or (c) mandate a local rsyslog/syslog-ng aggregator
   that converts UDP→RELP before Prism's receiver? What does BC-2.01.010 say about loss from a
   transport-layer drop?
3. **Coherence model for a demand-driven, event_time-TTL'd result cache across hops.** CDN coherence
   assumes stable URL-keyed objects; Prism's "object" is a query+window result set. What is the cache
   key? What does cross-hop invalidation look like? Is any cross-hop coherence guarantee offered, or
   is each tier independently TTL'd with coverage-metadata reconciliation at query time?
4. **Residency primitive granularity and vocabulary.** Per-field? Per-OCSF-attribute? Per
   (source-class, schema, schema-version) table? What is the classification vocabulary (`raw` /
   `normalized` / `metadata-only` / region-tag)? How is the transform-ordering constraint
   machine-verified (F7 in §17.8.1)?
5. **Where is the replication policy authored and enforced?** Per-collector TOML? Per-Satellite
   config? A new dedicated policy artifact? Who owns the `{select → reduce → retain → destination →
   RESIDENCY}` rule — the connector author, the Satellite operator, or a chain-level governance layer?
6. **Pre-aggregate semantics for security detection.** Dremel pre-aggregates are SUM/COUNT/histogram.
   Detection-window correlation (§14, §3.3 DI-029) may require event-level rows, not aggregates. Does
   a hub intermediate materialization hold reduced events or true aggregates? How does this interact
   with the streaming-correlation-state open question (§17.7 Phase 2)?
7. **Budget-aware planner — v1 or full?** §17.8.3 LEAN = v1 (gRPC + partial+coverage +
   opportunistic hub pre-aggregate). Human confirms feature ordering: full adaptive planner is a
   separate cycle.
8. **Hedging vs residency-partitioned topology.** Hedged/tied requests assume replica choice. Prism's
   tree is residency-partitioned (a leaf owns its layer's sources uniquely, §3.2 #1). Is there any
   replica to hedge to below a single-path subtree, or is partial+coverage the only deadline escape?
9. **Coverage-metadata schema unification.** §3.6/BC-2.01.010, CAQP, Elastic CCS are three
   vocabularies. Where is the canonical Prism coverage schema defined? What artifact owns it?
10. **Stampede / single-flight ownership.** When a hub pre-aggregate expires and N descendants
    simultaneously want it, who single-flights the recompute — the hub, the coordinator, or a
    distributed lock? How does this interact with §3.2 seen-request-ID loop prevention?
11. **Capability-descriptor for a no-pushdown push source.** A collector cannot push down predicates
    to the *source* — pushdown applies only to the *buffer*. Does the descriptor model need a
    `buffer-backed-push` class, and what are its join-guard implications (§12.2)?

### 17.11 Proposed epics and candidate ADRs (day-2; numbers deferred to architect at morph)

All items below are PROPOSED. Epic IDs and ADR numbers are illustrative placeholders; the architect
assigns real IDs during the brief-reframe morph.

**Proposed epics:**

| Epic (placeholder ID) | Scope |
|-----------------------|-------|
| **E-COLLECTOR-CLASS-001** | Collector-connector subtype: receiver endpoint abstraction, push-source capability descriptor, Satellite listener-hosting mode, UDP/TCP/HTTP/Kafka receiver implementations (Kafka-first), store-and-forward durability contract, loss metrics surface |
| **E-COLLECTOR-PCAP-001** | Full-packet capture: Arkime-model disk-bounded rolling packet buffer, PCAP-file rotation/retention/residency governance, Zeek/Suricata metadata normalization → OCSF Network Activity, PrismQL `retrieve-packets-by-session` affordance, S2 "download PCAP" action |
| **E-CHAIN-CACHE-001** | Chain-aware cache tiering: declarative per-collector replication policy (Q2 language, residency-first per-field, transform-ordering enforcement), topology-aligned tier placement (Q1, declarative-floor), v1 deadline-budget coordinator (Q3: gRPC + partial+coverage + opportunistic hub pre-aggregate), coverage-metadata schema unification |
| **E-STREAM-DETECT-001** | Streaming detection: NRT-over-cache Phase 1 (reuse §14, integrates with collector buffer), continuous windowed operator Phase 2 (event-time, watermarks, late arrivals, RocksDB state backend) — Phase 2 ordered later |

**Candidate ADRs:**

1. `ADR-PROP-collector-connector-subtype` — collector as connector subtype vs separate class;
   receiver endpoint model; `collector-connector` capability descriptor class.
2. `ADR-PROP-collection-locus` — locus as per-instance property; edge-first default; Satellite
   listener-hosting mode; convergence of (a) satellite-edge and (b) standalone-collector.
3. `ADR-PROP-chain-aware-cache-tiering` — declarative policy as authoritative floor; automatic
   demotion within envelope; event_time TTL for cross-tier freshness; failure-mode mitigations.
4. `ADR-PROP-upstream-replication-policy-language` — five primitives + three missing
   (store-and-forward, QoS, transform-ordering); residency-first per-field as first-class constraint;
   raw-forward across residency boundary inexpressible by construction.
5. `ADR-PROP-deadline-budget-planner` — v1 = gRPC per-hop decrement + partial+coverage + opportunistic
   hub pre-aggregate; full adaptive planner as separate feature cycle; hedging vs residency-partitioned
   topology.
6. `ADR-PROP-pcap-packet-store-regime` — second storage regime distinct from RocksDB/Iceberg; Arkime
   model; metadata/payload split; residency + security-review scope.
7. `ADR-PROP-continuous-operator-roadmap` — Phase 1 NRT-over-cache; Phase 2 native windowed operator;
   RocksDB state-backend alignment; ordering rationale.

**Cross-references:** `research/federated-ingestion-collector-connectors-2026-06-26.md` (collector
class, four-stage abstraction, locus, syslog deep, pcap deep, Cribl neutral-incentive, streaming
detection prior art, open questions §8) and `research/chain-cache-tiering-replication-deadlines-2026-06-26.md`
(Q1 tiering verdict, Q2 replication-policy primitive survey + residency gap, Q3 deadline-budget
verdict, composed chain model, failure modes, open questions §7).

**Gap registry additions:**

| Gap | Description | Proposed epic |
|-----|-------------|---------------|
| G-27 | Collector-connector subtype: receiver endpoint, push-source descriptor, Satellite listener mode | E-COLLECTOR-CLASS-001 |
| G-28 | Full-packet capture: Arkime-model packet buffer, PCAP-retrieve affordance, S2 action | E-COLLECTOR-PCAP-001 |
| G-29 | Chain-aware tiering + replication policy language (residency-first, transform-ordered) | E-CHAIN-CACHE-001 |
| G-30 | v1 deadline-budget coordinator (gRPC + partial+coverage + opportunistic hub pre-aggregate) | E-CHAIN-CACHE-001 |
| G-31 | Streaming detection Phase 2: continuous windowed operator (event-time, watermarks, RocksDB state) | E-STREAM-DETECT-001 |

---

*2026-06-26 side-analysis capture; PROPOSED; gated on brief-reframe sign-off; sources: `research/federated-ingestion-collector-connectors-2026-06-26.md` and `research/chain-cache-tiering-replication-deadlines-2026-06-26.md`.*

### 17.12 Protocol-Dissector Layer (the keystone)

> **PROPOSED. do_not_execute. Gated on brief-reframe sign-off.**

The dissector IS §17 stage-3 normalization for packet/stream sources: packet bytes →
structured OCSF Network Activity (class 4001, L3/4 envelope) + native schema-on-read
protocol/OT fields. It is pluggable, declarative, and spec-driven — the dogfood of
prism's TOML-connector philosophy applied to packets.

**DECIDED 2026-06-26 (human): prism EMBEDS a NATIVE Spicy-style declarative
parser-generator from the start.** Prism does NOT depend on running external Zeek or
Suricata as a process. Protocol grammars — Spicy-style `unit`/field/hook definitions
plus an interface-definition binding that compiles to a parser — are first-class prism
artifacts, and prism authors its own grammars INCLUDING OT protocols. Implementation
nuance deferred to the morph: embed the open-source BSD-licensed Spicy runtime/toolchain
versus re-implement an equivalent engine — either way the dissection engine is
prism-owned and prism-native; embedding the existing Spicy runtime is the pragmatic
route. In both paths prism controls the grammar lifecycle.

A dissector-backed packet sensor becomes "just another collector" (`FROM cache.<collector>`);
new protocols or OT dialects are new declarative grammar plugins with no core change;
per-site grammars fit the residency-first model perfectly — each satellite carries
exactly the grammars its equipment needs.

The dissector emits three things per session:
- **(a) OCSF Network Activity (4001)** — the portable L3/4 envelope (src/dst IP/port,
  transport, bytes).
- **(b) Native schema-on-read protocol/OT semantics** — Modbus function codes, DNP3
  object groups/points, S7 block/variable access, GOOSE dataset-refs/state-numbers,
  IEC-104 ASDU types, etc. — queried as prism native tables (§13.6; §17.13).
- **(c) The Community ID session key** — the hash of the 5-tuple that links
  normalized metadata (axis-1) to pinned raw packets (axis-2 §17.6) and enables the
  trigger→pin→retrieve loop (§17.14).

**Honest cost:** heaviest dissector build of any approach; re-treads coverage already
present in ICSNPP. The return is a first-class, integrated, memory-safe, fuzzable,
spec-driven dissection layer that prism fully owns, with no external process dependency.
Declarative grammars > hand-written imperative parsers for safety and fuzzability
(Spicy generates bounds-checked parsers). Research anchor: R5 in
`research/detection-reshape-protocol-dissectors-2026-06-26.md`.

---

### 17.13 OT/ICS Dissection + Safety + OCSF-OT Verdict

> **PROPOSED. do_not_execute. Gated on brief-reframe sign-off.**

**OT protocol matrix** (Purdue placement, open-source dissection status, prism grammar
responsibility):

| Protocol | Purdue layer | ICSNPP (Spicy) coverage | Prism grammar responsibility |
|----------|-------------|------------------------|------------------------------|
| Modbus / Modbus-TCP (TCP 502) | L0–L2 | YES — ICSNPP-Modbus | Adopt/extend ICSNPP |
| DNP3 (TCP 20000) | Substation L1–L3 | YES — ICSNPP-DNP3 | Adopt/extend ICSNPP |
| S7comm / S7comm-plus (TCP 102) | L1–L2 (engineering/HMI↔PLC) | YES — ICSNPP-S7COMM | Adopt/extend ICSNPP |
| IEC 60870-5-104 (TCP 2404) | Substation L2–L3 | YES — ICSNPP-IEC104 (flagged "outdated"; verify) | Audit + patch ICSNPP |
| IEC 61850 GOOSE (L2 multicast) | Process bus L0–L1 | YES — ICSNPP-GOOSE | Adopt/extend ICSNPP |
| PROFINET-IO-CM (L2 industrial) | L0–L2 | YES — ICSNPP-PROFINET-IO-CM | Adopt/extend ICSNPP |
| EtherNet/IP + CIP (TCP 44818) | L0–L2 | GAP in cited ICSNPP set | Prism authors grammar |
| OPC-UA (TCP 4840 / 443) | L2–L3 DMZ | GAP — often encrypted; Wireshark only | Prism authors grammar; encrypted-OT caveat applies (see below) |
| BACnet (UDP 47808) | Building OT L2–L3 | GAP in cited ICSNPP set | Prism authors grammar |
| MQTT (TCP 1883 / 8883 TLS) | L3–L3.5 / IIoT edge | GAP — often TLS; Wireshark only | Prism authors grammar; encrypted-OT caveat applies |
| IEC 61850 MMS (TCP 102) | Substation L2–L3 | GAP in cited ICSNPP set | Prism authors grammar |
| IEC 61850 Sampled Values (L2) | Process bus L0–L1 | Wireshark only; demanding | Prism authors grammar (later) |

(Source: CISA ICSNPP GitHub + deep-research R6 in `research/detection-reshape-protocol-dissectors-2026-06-26.md`.)

**DECIDED 2026-06-26 (human): OT/ICS telemetry is the flagship NATIVE-SCHEMA-ON-READ
case (§13.6).** OCSF has NO first-class ICS/OT event classes as of 2026; there is an
open proposal (ocsf/ocsf-schema issue #1515 "Industrial Control System (ICS) Field
Extensions") but nothing standardized. OCSF Network Activity (class 4001) carries only
the generic L3/4 envelope. OT protocol semantics — Modbus function codes, DNP3
object-groups/points, S7 block/variable access, GOOSE dataset-refs/state-numbers, IEC-104
ASDU types — have no OCSF home and MUST live in native structured tables queried
schema-on-read. When/if ocsf#1515 standardizes, prism's multi-version OCSF support
(G-16 §13.6) absorbs ICS extensions without a native-schema migration.

**Safety constraints — NON-NEGOTIABLE:**

- **Passive / read-only only.** Active polling or scanning can fault, hang, or
  fail-safe PLCs/RTUs/relays. OT monitoring is passive analysis from a TAP or SPAN;
  prism never injects onto an OT segment under any configuration.
- **TAP preferred over SPAN.** SPAN/mirror ports can drop under congestion and may lose
  time-critical L2 frames (GOOSE, SV, PROFINET RT). TAPs are strongly preferred for
  process-bus protocols; SPAN is an acceptable fallback for L3+ flows.
- **Purdue + IEC 62443 zones-and-conduits placement.** OT dissectors run on the
  OT-layer Satellite (§3.2) at the correct Purdue layer — L0–L1 process-bus protocols
  (GOOSE/SV/PROFINET) need a sensor on that segment with L2 access; L2↔L3 conduit
  sensors cover the supervisory layer.
- **Determinism / no-injection.** Parsing must be robust and lightweight; a
  malformed-parse must never misinterpret safety-relevant data; the dissector must not
  add buffering or forwarding load to RT I/O paths.

**DECIDED 2026-06-26 (human): encrypted-OT visibility (OPC-UA, MQTT-over-TLS) =
metadata-only by DEFAULT.** Passive capture of encrypted OT traffic yields only
L3/4 metadata (IP/port/bytes) — no protocol semantics. An OPT-IN, carefully-bounded
decryption/proxy posture at OT gateway chokepoints (known keys + known endpoints) is
a LATER capability — explicitly tensioned with strict passivity; default-OFF;
requires explicit per-site authorization; decryption NEVER occurs on the OT segment
itself, only at a governed IT-side gateway chokepoint.

Research anchor: R6 in `research/detection-reshape-protocol-dissectors-2026-06-26.md`.

---

### 17.14 How #4 + #5 + the Dissector Reshape Storage and Detection (Synthesis + Decisions)

> **PROPOSED. do_not_execute. Gated on brief-reframe sign-off.**

This section synthesizes how the protocol-dissector layer (§17.12), full-packet retrieval
(§17.6 #4), and continuous-operator capability (§17.7 #5) compose into a coherent storage
and detection architecture. It records the DECIDED items from the 2026-06-26 discussion
and the remaining OPEN questions.

**Storage — two axes + a state substrate:**

- **Axis-1 (normalized metadata)** — OCSF Network Activity (class 4001) L3/4 envelope +
  native schema-on-read OT/protocol semantics — produced by the dissector (§17.12),
  stored in the federated/tiered metadata stores per §3.3 and §17.8, pushed up through
  the chain.
- **Axis-2 (raw packets)** — the §17.6 Arkime-style edge-local rolling buffer,
  disk-bounded, detection-pinned; retrieved by session-ID on demand; raw packets never
  cross a satellite residency boundary.
- **State substrate** — RocksDB column families for continuous-operator window/correlation
  state and `detection_state`; ML `ModelState` (§15) logically separable (see below).

**Detection — ONE language, TWO engines:**

**DECIDED 2026-06-26 (human): the continuous-operator (#5 Phase 2) is prism-NATIVE on
the RocksDB state backend** — reuses the `MATCH_RECOGNIZE` NFA operator prism already
owns (DataFusion will not execute it; G-18) extended with a watermark/checkpoint/late-data
layer. This is NOT an embedded Flink; it is prism's own operator running on the existing
RocksDB infrastructure. One engine, one language; the watermark/checkpoint layer is the
expensive build item (consistent with §17.7 "Phase 2 = the single most expensive item").

**DECIDED 2026-06-26 (human): the detection spec carries EXPLICIT temporal semantics.**
The §14 YAML rule metadata gains `lateness` / `accumulation` / window-alignment fields.
The planner picks the EXECUTION ENGINE (polled-NRT vs continuous) but NEVER the
semantics. This prevents silent polled-vs-continuous alert divergence — a rule cannot
mean two different things depending on which physical engine the planner selects. Research
anchor: R2 in `research/detection-reshape-protocol-dissectors-2026-06-26.md`.

Note: `WATCH…UNLESS` (§12.4 absence/exclusion operator) now has TWO physical
implementations:
- **Relational anti-join `AbsenceWindowNode`** — polled/batch path (existing §12.4
  desugaring, closes the bounded retrospective window).
- **CEP-style per-partition TIMER** — continuous path (absence over an unbounded stream
  requires a timer/watermark to ever "complete" a non-match; a pure relational anti-join
  cannot confirm absence without a deadline). Research anchor: R1.

**Detection-driven packet retention (the #4 × #5 loop):**

The canonical flow is: **trigger → pin → retrieve.** The continuous operator (Phase 2)
or the v1 edge detector (Phase 1 NRT path) emits session identifiers on detection; the
§17.6 rolling buffer receives a pin signal for those session-IDs, extending their
retention beyond rolling expiry; analysts or automation retrieve the pinned PCAP
on-demand via the Community ID session key. Prior-art validation: Suricata conditional
pcap / Zeek Time Machine / Corelight Smart PCAP / Stenographer / Arkime (R3). Required
infrastructure: synchronized clocks across the dissector, operator, and PCAP buffer;
the Community ID session key as the consistent cross-tool session identifier; pin-policy
ownership in the §17.8 retention-policy engine (extended).

**State unification (DECIDED-LEAN, 2026-06-26):**

Continuous-operator window state + `detection_state` share the **RocksDB/RetentionCache
family in DISTINCT column families** per the existing 19-CF pattern — column-family
isolation within one engine, not a separate datastore. This matches Kafka Streams
precedent (operator state + long-lived materialized state on one RocksDB-backed mechanism)
and honors §14.3 "no new datastore." ML `ModelState` (§15) is kept **logically separable**
— different access pattern (random-read at inference), different lifecycle (versioned
releases), different blast-radius / recovery profile; dedicated CF or dedicated RocksDB
instance (no new datastore either way). The continuous operator's window state gets its
own incremental-checkpoint cadence, distinct from the durable `detection_state` checkpoint.
Research anchor: R4.

**Honest costs:**

1. Prism now owns TWO heavy native engines: the Spicy-style declarative dissector + the
   windowed continuous operator with watermark/checkpoint machinery.
2. Heavy edge compute — deep OT dissection + RocksDB-backed continuous operator at the
   capture point; drop/backpressure risk; per-site DevOps complexity.
3. OT passivity ceiling — strict observe-only; lightweight parsing mandatory; no inline
   OT enforcement; detection must not perturb TAP/SPAN infra.
4. Multi-schema burden — OCSF portable baseline + OT-site-specific native schema;
   per-site schema discovery/registry; detection portability splits (OCSF-portable rules
   globally deployable; OT-native rules residency/site-specific, must degrade gracefully
   where OT fields absent).
5. Checkpoint/recovery coupling — shared checkpoint stream couples fast operator state to
   slow campaign state; column-family boundary may not be sufficient isolation (open
   question).

**Remaining OPEN questions (NOT decided):**

1. State checkpoint-cadence specifics — cadence for continuous-operator window state vs
   durable `detection_state` vs ML `ModelState`.
2. Pin-policy detail — which detections pin which sessions; first-N-bytes vs full-session;
   who owns the policy (detection spec, retention-policy engine, or per-site config).
3. Detection-portability governance — OCSF-portable vs OT-native rule lifecycle; how a
   globally-deployed rule degrades gracefully where OT fields are absent.
4. Entity-registry (§12.1) OT-observable resolution — how an OT observable (PLC IP +
   unit-ID) resolves across OCSF + native + IT schemas.
5. Per-satellite edge compute budget vs the OT passivity ceiling — what is the per-site
   compute envelope and how is "must not perturb TAP/SPAN" enforced/measured.

**Proposed epics and candidate ADRs (numbers deferred to architect at morph):**

| Proposed epic | Scope |
|---------------|-------|
| **E-DISSECTOR-NATIVE-001** | Embedded Spicy-style parser-generator engine (runtime embed or equivalent); core protocol grammars (Modbus/DNP3/S7comm/IEC-104/GOOSE/PROFINET from ICSNPP); Community ID session-key emission; §17 stage-3 integration |
| **E-DISSECTOR-OT-001** | OT grammar gaps: EtherNet/IP, OPC-UA, BACnet, MQTT, IEC-61850 MMS/SV; per-site grammar plugin lifecycle; safety/passivity enforcement |
| **E-STREAM-DETECT-001** (extend) | prism-native continuous operator + explicit-semantics detection spec (`lateness`/`accumulation` fields in §14 YAML); `WATCH…UNLESS` CEP-timer path; watermark/checkpoint machinery |
| **E-COLLECTOR-PCAP-001** (extend) | Trigger→pin→retrieve: detection-driven pin signals to rolling buffer; Community ID cross-tool session linkage; clock-sync requirements |

ADR candidates: native Spicy-style dissector engine selection (embed vs reimplement);
prism-native continuous windowed operator on RocksDB; detection-spec explicit temporal
semantics; OT native-schema-on-read model + encrypted-OT bounded-decrypt opt-in policy.

Gap registry additions:

| Gap | Description | Proposed epic |
|-----|-------------|---------------|
| G-32 | Native Spicy-style declarative dissector engine (embedded runtime or equivalent) | E-DISSECTOR-NATIVE-001 |
| G-33 | OT grammar gaps (EtherNet/IP, OPC-UA, BACnet, MQTT, IEC-61850 MMS/SV) | E-DISSECTOR-OT-001 |
| G-34 | Detection-spec explicit temporal semantics (`lateness`/`accumulation` fields) | E-STREAM-DETECT-001 (extend) |
| G-35 | `WATCH…UNLESS` CEP-timer path for continuous/unbounded-stream absence detection | E-STREAM-DETECT-001 (extend) |
| G-36 | Detection-driven pin signals + Community ID cross-tool session linkage for trigger→pin→retrieve | E-COLLECTOR-PCAP-001 (extend) |

---

*2026-06-26 side-analysis capture; PROPOSED; gated on brief-reframe sign-off; source: `research/detection-reshape-protocol-dissectors-2026-06-26.md` (R1–R7, OT matrix, honest costs, open questions).*

---

### 17.15 — Ingestion Open Sub-Threads: Resolved Leans (2026-06-26)

> **ACCEPTED 2026-06-26 (human).** Seven ingestion open sub-threads from
> `research/ingestion-open-subthreads-2026-06-26.md` (7 × `perplexity_research` at
> `reasoning_effort: high`, 2026-06) were presented as pressure-tested recommendations
> and accepted as design leans. They do NOT constitute finalised ADR decisions; they become
> the input to the respective epic/ADR authorship at morph time. All items remain
> `do_not_execute: true` and gated on brief-reframe sign-off (§5.1).
>
> **Research citation:** `research/ingestion-open-subthreads-2026-06-26.md` (all seven
> sub-threads; read-coverage caveat in research front-matter: sub-threads 3–7 mined via
> targeted grep of oversized result files; sub-threads 1–2 read in full through §6 of 8).

#### A1 — Detection-Portability Governance (Sub-thread 1)

**Lean (ACCEPTED):** per-rule **data-dependency manifest** carrying the OCSF classes/attribute
paths AND/OR native-OT tables the detection requires. The planner checks the manifest against
available capability descriptors (§3.4/§10.3 ADOPT-1) at enable-time. **Three explicit rule
states — not silent zero-rows:**
- `runnable` — all required data present.
- `degraded` — some optional fields absent; runs a reduced predicate set, marks findings
  `coverage=partial`.
- `unavailable` — required data absent; rule auto-disabled with a surfaced reason.

**Two rule classes in ONE lifecycle (§14.1):** OCSF-portable detections declare OCSF classes;
OT-native/site-specific detections declare native-OT tables (§17.12). Same lifecycle
(draft→production), CI/test, MITRE mapping; only the manifest + site-applicability differ.

**No prior art for graceful degradation at this granularity** — existing tools (Sigma, Elastic
`required_fields`, Sentinel data-connector gating) all produce silent zero-rows or require
manual disable. Prism making it declarative is greenfield.

#### A2 — Result-Cache Coherence Across Hops (Sub-thread 2)

**Lean (ACCEPTED):** canonical cache key =
`canonicalized(query_intent) + canonicalized(time_window) + residency_scope + schema_version`.
Parse to logical plan, sort predicates, normalize literals/operators, hash (Intent Signature
pattern). Align time windows to **event-time buckets** for key collision.

**DECLINE cross-hop coherence as a guarantee in v1 (the conscious scope-limit):** each tier
independently TTLs its entries; **reconcile at query time via coverage-metadata** (§3.6 /
BC-2.01.010) rather than promising a cross-hop coherence invariant. This sidesteps stale-parent
poisons-children, eviction races, and double-caching failure modes. Coverage metadata says what
window/sources each tier's contribution actually covers; the coordinator merges + reports partial.

**Subsumption as optimization, not correctness:** a cached wider window may answer a narrower
query only if its watermark ≥ the narrow window end AND its residency scope ⊇ the requester's
allowed scope.

**Single-flight the recompute:** coalesce N concurrent recompute requests for the same canonical
key into one; integrate with §3.2 seen-request-ID loop prevention.

**Note:** result-cache METADATA (coverage annotations, freshness watermarks, canonical key store)
lands on **BUNDLED PostgreSQL** (per the storage taxonomy, §17.15 and §14.3 addendum) — NOT in
RocksDB. The cache PAYLOAD stays in RocksDB hot / Iceberg cold.

#### A3 — Continuous-Operator Checkpoint Cadence + Recovery Isolation (Sub-thread 3)

**Lean (ACCEPTED):** **at-least-once + idempotent finding-emit** (relax exactly-once). Findings
are deduped by `(rule_id, match_key, window)`; duplicate emits are harmless. Avoids two-phase-
commit cost on a demand-driven, ephemeral operator.

**Two-tier state by criticality (greenfield — no prior art supports this off-the-shelf):**
- (a) **Hot window correlation state** — large, fast-changing, *recomputable* from the
  §3.3 RetentionCache window → checkpoint rarely or not at all; rebuild from cache on recovery.
- (b) **Durable detection/campaign/risk state + fired-finding dedup** — small, must survive →
  checkpoint frequently (short-cadence, changelog-style) to RocksDB.

Handling the **processing-time-timer storm** on restore: prefer event-time timers; on restart,
clamp/coalesce processing-time timers that "should have fired" rather than bursting.

#### A4 — Detection-Driven Packet-Pin Policy (Sub-thread 4)

**Lean (ACCEPTED):** **decoupled three-stage pipeline** — dissector/detection (§17.12/§14)
emits flow-tied trigger → a **retention-policy engine OWNS the pin decision** (NOT the
detection rule directly). Pin policy is a residency-aware retention decision, not an analytics
decision (consistent with §17.8 Q2 policy-as-artifact locus).

**First-N-bytes default, full-session escalation:** mirror Zeek Time Machine class-based
cutoffs. Default pin = headers + first-N-bytes (forensic triage); high-severity / specific
detection classes escalate to full-session.

**Pin = explicit object** keyed by **Community ID** (§17.12 axis-2 session key):
`{community_id, byte_depth, retain_until, residency_scope}`. Severity-tiered retention
duration is the gap prism fills — no prior tool carries this as a first-class pin attribute.

**PTP/NTP is a hard prerequisite** at the satellite — alert timestamps must align to
captured-packet timestamps; surface clock-sync health as a satellite fleet signal (§17.10 Q2).

#### A5 — Entity-Registry Cross-Schema Resolution incl. OT (Sub-thread 5)

**Lean (ACCEPTED):** **prism-native, config-driven entity registry** (§12.1 TOML path is
correct — no standard to adopt; OCSF defines NO cross-event canonical identity through 1.3.x).
Map `entity_type → ordered set of attribute paths` spanning OCSF + native-OT + IT schemas.

**Deterministic-first, with strong/weak identifier tiers** (adopt Sentinel's taxonomy):
resolve on strong IDs (device serial, MAC, CIP identity object) exactly; treat weak IDs (IP,
hostname, unit-ID-alone) as probabilistic with **temporal-validity windows** (DHCP-reassigned
IP must not merge two assets).

**OT identity is prism's own concern.** OCSF has no OT entity model; SIEM models map poorly
to OT sub-device identity (Modbus unit-ID, DNP3 station-address, CIP identity object). The
entity registry + §17.12/§17.13 dissector together ARE the OT entity layer. prism is
*defining*, not adopting — own the spec surface.

**Flow vs asset distinction held:** Community ID = flow/session key (A4 pin linkage);
entity registry = asset layer. Do not conflate.

#### A6 — Per-Satellite Edge Compute Budget + Backpressure Governance (Sub-thread 6)

**Lean (ACCEPTED):** **protect the capture path FIRST — hard isolation invariant.** Pin
capture to dedicated cores via cgroups v2 + CPU affinity + IRQ steering; size NIC rings for
burst absorption. The dissector (§17.12), windowed operator (§17.7), and packet buffer (§17.6)
run in a **lower-priority cgroup** that can be CPU/IO-throttled but can NEVER starve capture.

**Shed analysis before dropping packets (ordered degradation under pressure):**
1. Shed/sample the windowed operator work (cheapest to lose, recomputable from cache per A3).
2. Reduce dissector depth (parse fewer protocols / shallower).
3. Only as last resort accept capture loss.
Adopt Aurora/Borealis window-aware shedding for the operator tier.

**Surface canonical signals upward as the fleet budget contract:** per-satellite drop-rate
(capture_loss / kernel_drops), **ring-utilization**, **analysis-lag** (operator backlog),
dissector queue depth, **clock-sync health** (A4). A satellite exceeding a drop threshold is
*over-budget* — fleet either provisions more cores or narrows that site's grammar/operator scope.

#### A7 — Per-Field Residency Classification + Replication-Policy Authorship Locus (Sub-thread 7)

**Lean (ACCEPTED):** **classify per-field, scoped within a per-(source-class, schema, version)
table.** Tag vocabulary: a small residency-class enum per field (e.g., `raw` / `normalized` /
`metadata-only`) + a region/zone tag, attached to the field within its
`(source_class, schema, schema_version)` table descriptor (§13.6 multi-schema).

**Locus: author residency/replication policy as a DEDICATED policy-as-code artifact; reserve
inline per-collector config for BINDING + EXCEPTIONS only.** The OPA/Kyverno/SCP pattern is
decisive — core governance logic belongs in a separable artifact (central audit, consistency,
separation of authorship: connector-author ≠ compliance-owner). The `{select → reduce → retain
→ destination → RESIDENCY}` rule (§17.8.2) lives in a **chain-level governance policy artifact**.

**Transform-ordering machine-verification:** because residency is a separate policy artifact
evaluated over field-level tags, the "residency enforced BEFORE destination selection" invariant
(§17.8.2 F6/F7) becomes a checkable property of the policy engine — a raw-tagged field crossing
a region boundary must be **inexpressible by construction** in the policy DSL.

**Prior art note:** field/column tagging is mature for sensitivity/masking (Purview, Snowflake
tags, BigQuery policy tags) but all existing systems express residency at resource/bucket/region
level, NOT per-field. prism's residency-first per-field model is ahead of, not behind, prior art.

#### Cross-cutting observations

- Sub-threads 1 (data-dependency manifest) + 5 (entity registry) + 7 (residency-tag vocabulary)
  all converge on a **prism-native residency-and-capability-aware metadata layer** that both the
  §17.8 chain-cache/replication policy and the §14 detection lifecycle read. Build as one coherent
  descriptor family, not three silos.
- Sub-threads 2 (decline cross-hop coherence) + 3 (recompute-cheap state) + 6 (shed-before-drop)
  all converge on **graceful degradation as the first-class primitive** — the §3.6 partial-result
  thesis extended from query fan-out to caching, recovery, and edge overload. Unifying rule:
  **degrade explicitly with a surfaced signal, never silently.**
- The §17.12 dissector + §17.7 operator co-residence (A6) is the load-bearing physical constraint.
  If the operator must shed-to-hub under OT line-rate pressure, that reshapes §17.7 Phase-2
  placement — the edge-budget sub-thread is not cosmetic; it gates where heavy analysis can live.

#### Honest cost line

Adopting these seven leans adds: four declarative metadata/policy surfaces — (i) per-rule
data-dependency manifest + tri-state enablement (A1); (ii) canonical result-cache key + unified
coverage-metadata schema (A2); (iii) prism-native entity registry with strong/weak + temporal-
validity (A5); (iv) per-field residency-tag vocabulary scoped within (source-class, schema,
version) tables + dedicated policy-as-code artifact (A7) — plus two operational disciplines —
capture-path-first cgroup isolation with ordered analysis-shedding (A6), and a severity-tiered
pin-retention policy engine (A4) — plus one engine design — at-least-once, two-tier
(recompute-cheap window vs durable detection) state for the continuous operator (A3).
**No new datastore; all RocksDB-native + TOML-spec-driven**, consistent with the ephemeral /
federated / residency-first / OCSF+native thesis.
