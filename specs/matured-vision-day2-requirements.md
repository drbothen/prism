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
