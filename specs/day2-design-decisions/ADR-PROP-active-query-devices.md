---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C14-1: Scope = BOTH Reading A (OT-platform northbound REST APIs) AND Reading B (direct OT-protocol polling) in v1"
  - "ADR-PROP-C14-2: Poller-of-last-resort = YES (Reading B supported for customers with no OT platform)"
  - "ADR-PROP-C14-3: Modeling = capability-descriptor AXIS on the existing unified adapter (C3/C4), NOT a new connector class"
  - "ADR-PROP-C14-4: OT asset data modeled as first-class PrismQL-queryable OCSF source tables, normalized at satellite boundary"
  - "ADR-PROP-C14-5: OT-SAFETY GUARDRAILS = HARD INVARIANTS (mandatory esp. Reading B)"
  - "ADR-PROP-C14-6: Edge-executed, OCSF-at-boundary, reuse C2 topology unchanged"
  - "ADR-PROP-C14-7: Reading-B protocol packaging = plugins / sidecar (NOT in the core satellite binary)"
produced_by: architect
timestamp: "2026-06-27"
provenance: "side-analysis C14 capture; human-confirmed decisions 2026-06-27 session. Research basis: research/active-query-devices-2026-06-27.md (4 perplexity_research sonar-deep-research calls at reasoning_effort=high + 1 perplexity_ask for ownership correction). Does NOT modify live ADR files, ARCH-INDEX.md, STATE.md, SESSION-HANDOFF.md, or any live factory artifact."
traces_to:
  - matured-vision-day2-requirements.md §16.4 (C14 decisions log entry)
  - day2-design-decisions/ADR-PROP-dynamic-schema-connectors.md (C4 — unified adapter model; WASM plugin packaging)
  - day2-design-decisions/ADR-PROP-capability-descriptor-pushdown.md (C3 — capability-descriptor axis; cost-based-degrade PAT-ADS-04)
  - day2-design-decisions/ADR-PROP-satellite-mesh.md (C2 — edge-executed, OCSF-at-boundary, topology unchanged)
  - day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md (P-ADS-01/05/06/08; PAT-ADS-04; INV-ADS-01/05/07/08; AP-ADS-01/06)
  - research/active-query-devices-2026-06-27.md (primary research basis — all six Q topics)
  - CLAUDE.md (ADR-022 Arc-DI wiring; ADR-024 ColumnType; non_exhaustive; AD-017 AI-opaque credentials)
cross_links:
  - C3 (capability-descriptor pushdown — axis extension)
  - C4 (dynamic-schema connectors — unified adapter model)
  - C2 (satellite mesh — edge-executed topology unchanged)
  - C11 (device-vuln ties threat intel / Prism Intel)
  - C12 (OT asset graph — entity linkage in Prism Context)
  - C15 (gated-action / SOAR — writes/commands via C15 ONLY, not query fanout)
  - C20 (OT/NERC-CIP compliance context)
  - ADS (P-ADS-01/05/06/08; PAT-ADS-04; INV-ADS-01/05/07/08; AP-ADS-01/06)
epic_anchors:
  - E-ACTIVE-QUERY-001 (API-tier active query — Reading A: OT-platform northbound REST adapters + OT table modeling)
  - E-OT-PROTOCOL-CONNECTORS-001 (Reading B: direct OT-protocol satellite polling — Modbus/OPC-UA/DNP3/SNMP)
---

# ADR-PROP — Active-Query Device Support: C14 Active-Query Connectors

> **STATUS: DECIDED 2026-06-27 (human).** This is a CAPTURE artifact for the side-analysis C14
> program. `do_not_execute: true`. Real ADR numbers and formal ARCH-INDEX.md rows are deferred to
> the morph execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

> **Research basis:** `research/active-query-devices-2026-06-27.md` — four `perplexity_research`
> (sonar-deep-research, `reasoning_effort=high`) calls covering Industrial Defender product and API
> model, peer-vendor active-query interfaces (Nozomi/Claroty/Dragos/Armis), connector-architecture
> patterns (Trino/Calcite/Steampipe/Apollo/EIP), and OT/ICS active-query safety standards (IEC 62443,
> NIST SP 800-82 Rev 3, CISA), plus one `perplexity_ask` for Industrial Defender ownership
> correction. All load-bearing claims are source-grounded in that research document.

> **Scope reminder:** C14 covers the active-query device connector model only — not story
> decomposition, not live BCs. The Industrial Defender ownership correction (Teleo Capital /
> Cuadrilla Capital, NOT GE Vernova) is recorded here as fact.

---

## Context

Prism is an ephemeral federated query engine over sensor APIs. Existing sensors (CrowdStrike,
Claroty, Armis) are all request-response REST connectors — federated reads that Prism initiates.
C14 extends the connector surface to encompass "active-query devices": sources that Prism
POLLS on demand to answer a PrismQL query.

There are two distinct readings of "active-query device support":

- **Reading A (API-tier):** Prism federates OT-platform northbound REST APIs (Industrial Defender
  `asmdataservice`, Nozomi OpenAPI, Claroty API Explorer/xDome, Dragos web API, Armis Centrix) as
  HTTP source adapters. The platform owns the device-side active polling; Prism inherits ZERO
  OT-protocol safety risk. This is a thin extension to the existing adapter model.

- **Reading B (protocol-tier):** Prism (at the Edge Satellite) directly speaks OT protocols
  (Modbus / OPC-UA / DNP3 / SNMP) to field devices. This is for customers with NO OT platform —
  Prism becomes the poller of last resort. It carries full OT-safety obligations.

**Key research finding:** Industrial Defender is NOT owned by GE Vernova (correction per
`perplexity_ask` 2026-06-27). Ownership chain: Independent → Lockheed Martin (2014) → Capgemini
→ Teleo Capital (2020, PE-backed spin-out) → Cuadrilla Capital (~2025). Independent, U.S.-owned,
PE-backed OT/ICS vendor headquartered near Boston. This does not change the C14 architecture.

**Key research finding:** The federated-query literature (Trino SPI, Apache Calcite, Steampipe,
Apollo Federation) is unanimous — "active-query vs passive-ingest" is NOT a connector CLASS. It
is a capability axis on a unified adapter interface. This directly informs D-C14-3 below.

---

## Decision Ledger

### D-C14-1 — Scope: BOTH Reading A AND Reading B in v1

**DECIDED 2026-06-27 (human).**

Both readings are in v1 scope:

- **(A) Federation of OT-platform northbound REST APIs** — Industrial Defender / Nozomi / Claroty /
  Dragos HTTP source adapters. These are API-tier: Prism polls the platform's REST API; the
  platform owns the device-side collection. Inherits zero OT-protocol safety risk. Same connector
  shape as existing prism-sensors.

- **(B) Direct OT-protocol polling of field devices** — Modbus / OPC-UA / DNP3 / SNMP from the
  Edge Satellite to the device. For customers with NO OT platform. Prism is the poller of last
  resort. Carries full OT-safety obligation under §3.3 guardrails.

**Research lean for Reading A:** LEAN 1/LEAN 2 in the research document — HIGH confidence,
API-tier as the primary shape. All five peer vendors expose REST/JSON northbound APIs.

**Cost of both-in-v1:** Reading B is a genuinely new connector class with the full §3.3 guardrail
obligation, IEC-62443/NIST-SP-800-82 risk-assessment burden, and liability exposure recorded as
F3 (OQ-C14-SAFETY-LIABILITY). It must not be confused with Reading A in implementation scope.
This ADR-PROP's D-C14-7 addresses the packaging isolation.

---

### D-C14-2 — Poller-of-Last-Resort: YES

**DECIDED 2026-06-27 (human).**

Prism supports customers with NO OT platform by directly polling field devices (Reading B). This
makes Prism own OT-safety risk on the Reading-B path.

**Consequence:** The OT-safety guardrails in D-C14-5 are MANDATORY, not optional, because D-C14-2
places Prism in the ACTIVE-POLLER role for those customers. There is no "we'll sort out safety
later" path — the risk-based-justification and non-production-validation requirements in D-C14-5
apply from the first Reading-B connector deployment.

**What "poller-of-last-resort" means operationally:** A Reading-B source requires an explicit
risk-based justification acknowledgment from the customer (operator-configured, not default-on)
AND non-production validation before enabling in a production environment. The capability is
available but gated, not a default.

---

### D-C14-3 — Modeling: Capability-Descriptor AXIS on C3/C4, NOT a New Connector Class

**DECIDED 2026-06-27 (human).**

Active-query device support is modeled as an axis extension on the existing unified adapter
interface (C3 capability-descriptor / C4 dynamic-schema connector), NOT as a parallel connector
taxonomy or new connector class.

This is the unanimous recommendation of the federated-query connector literature (Trino SPI,
Apache Calcite adapters, Steampipe plugin SDK, Apollo Federation) — all model capability as
orthogonal flags on a unified adapter. None has a first-class "active-query connector" type.

**Extensions to the C3/C4 capability descriptor:**

| Field | Type | Purpose |
|-------|------|---------|
| `active_query` | enum: `on_demand_read` / `cached_read` / `streaming` | Query initiation model for this source. `on_demand_read` = Prism polls on demand (Reading A/B). `cached_read` = Prism reads from a local cache refreshed on a schedule. `streaming` = push-ingest (C17 territory). |
| `protocol` | enum: `http` / `modbus` / `opcua` / `dnp3` / `snmp` | Wire protocol for Reading-B sources. `http` covers Reading A and all existing sensors. OT-protocol values require D-C14-5 guardrails. |
| `poll_cadence` / `freshness` | duration hints | Min poll interval, recommended cache TTL. Feeds cost-based-degrade (PAT-ADS-04) to prefer cached snapshots over re-polling a rate-sensitive OT source. |
| `rate_limit` | per-source descriptor | Max requests/sec, concurrent-connection cap, backoff policy. DISTINCT from `HTTP_SEMAPHORE_PERMITS` (global). Per-source. |
| `read_only` | boolean assertion | REQUIRED on every active-query descriptor. MUST be `true`. A source with `read_only: false` is a configuration error, not a valid connector state. |

**C3↔C14 reconciliation invariant:** The C3 cost-based-degrade planner MUST treat
`poll_cadence.min_interval` as a per-source cost floor. Prefer cached snapshots via the local
satellite buffer over re-polling a source whose last poll is within `poll_cadence.min_interval`.
This mirrors the C3 injected-time-bound and join-guard patterns (D-C3-1/D-C3-2).

**All descriptor structs are `#[non_exhaustive]`** per CLAUDE.md conventions — existing match
arms receive a wildcard `_ => {}` arm; new protocol variants are addable without breaking callers.

---

### D-C14-4 — Data Modeling: First-Class PrismQL-Queryable OCSF Source Tables

**DECIDED 2026-06-27 (human).**

OT asset-inventory, configuration baselines/exceptions, device-vulnerability data, and PLC/RTU
state are modeled as first-class PrismQL-queryable OCSF source tables, normalized at the
satellite boundary. This mirrors Industrial Defender's own internal decomposition:
`AdminProp` / `Exception` / `Vulnerability`.

**Proposed table decomposition (mirroring ID/Nozomi/Claroty data shapes):**

| Table | Sourced From | OCSF mapping |
|-------|-------------|--------------|
| `ot_assets` | Asset inventory (ID `AdminProp`, Nozomi asset entities, Claroty asset reports) | Entity class — pending OQ-C14-OCSF |
| `ot_config_baselines` | Configuration baselines and exceptions (ID `Exception`, Claroty configuration data) | Configuration class — pending OQ-C14-OCSF |
| `ot_device_vulns` | Device vulnerabilities (ID `Vulnerability`, Nozomi CVE feed, Dragos vuln data) | Vulnerability Finding class — pending OQ-C14-OCSF |
| `ot_device_state` | Live PLC/RTU state for Reading-B sources only (register reads, tag values) | Network Activity or custom — pending OQ-C14-OCSF |

**OCSF normalization caveat (OQ-C14-OCSF — OPEN QUESTION):**
OCSF OT schema coverage is being researched in a parallel follow-up pass
(`research/ocsf-ot-coverage-2026-06-27.md`, in flight as of 2026-06-27). OCSF has NO dedicated
OT classes as of 2026 (open proposal `ocsf#1515`; cross-referenced with detection-reshape
decisions in §17.12–§17.14). The normalization design (generic OCSF class vs. OCSF extension
vs. custom schema-on-read) is gated on the OQ-C14-OCSF follow-up research. This table is a
STRUCTURAL DECISION (first-class queryable tables, normalized at boundary) but the specific OCSF
class assignments are OQ-C14-OCSF pending. Per D-C4-1, ALL connectors including these OT tables
use the full normalization chokepoint — no trusted-source exemption.

---

### D-C14-5 — OT-Safety Guardrails: HARD INVARIANTS

**DECIDED 2026-06-27 (human).**

Active polling has caused real PLC crashes (CODESYS Forge multi-year intermittent crash,
root-caused to a network scan). IEC 62443, NIST SP 800-82 Rev 3, and CISA frame active scanning
as high-risk requiring risk-based justification and read-only semantics. These guardrails are
MANDATORY — failure to implement them is not a production-grade default.

**The guardrails bind hard for Reading B (protocol-tier) and apply as good engineering for
Reading A (API-tier rate-sensitive OT platforms):**

| Guardrail | Description | Where Enforced |
|-----------|-------------|----------------|
| **Read-only semantics** | Active-query read capability MUST NOT imply write. Writes route through C15 gated-action at the strictest autonomy tier, NEVER through the query fanout. Enforced via the `read_only: true` mandatory assertion on every active-query descriptor (D-C14-3). | Capability descriptor; C15 routing |
| **Rate-limiting** | Per-source `rate_limit` descriptor (max req/sec, connection cap) in the capability descriptor. Cost-based-degrade (PAT-ADS-04) enforces this at plan time — prefers cached snapshots over re-polling. | C3/C4 descriptor; degrade logic |
| **Connection-cap management** | Small, stable, long-lived connection set. Rapid open/close churn is forbidden for OT-protocol sources — it destabilizes device connection tables. | Reading-B plugin implementation |
| **Maintenance-window scheduling** | Option to schedule active polls during designated maintenance windows to avoid peak-criticality periods. Field on the source descriptor; operator-configured. | Source TOML descriptor |
| **Risk-based justification + non-production validation** | BEFORE enabling a Reading-B (OT-protocol) source in production: operator acknowledges risk, non-production environment validated first. | Operator-gated enable flag; deployment runbook |
| **Cost-based-degrade prefers cache** | Cache is the default, not re-polling. The planner must prefer a cached snapshot from the satellite buffer over re-polling a rate-sensitive or fragile source whenever the cached data is within the `freshness` TTL. | PAT-ADS-04 degrade logic |

**Rationale for hard-invariant status (not guidelines):**
Standards define principles (risk-based, read-only, rate-limited, scheduled) but do NOT publish
exact safe poll-cadence/packet-rate numbers — those are vendor/integrator engineering.
Any Prism safety numbers are original engineering, validated non-production first. The FAILURE
MODE is a controller crash that shuts down physical infrastructure; this is not a UX defect.
OT-safety is a correctness invariant, not an operational preference.

**Liability note (OQ-C14-SAFETY-LIABILITY — OPEN QUESTION, NOT engineering):**
Who owns the risk when a Prism Reading-B query contributes to a controller fault is a
legal/insurance/customer-contract decision, NOT an engineering one. This must be resolved by
legal/sales/customer-success before Reading-B connectors ship to customers. Not an architect
call; flagged here so morph planning does not treat it as closed.

---

### D-C14-6 — Edge-Executed, OCSF-at-Boundary, C2 Topology Unchanged

**DECIDED 2026-06-27 (human).**

Active-query device access is edge-local. The central Coordinator NEVER directly contacts a
field device or OT platform. This is identical to the existing federated-read topology (C2):

```
User query @ Coordinator (Central)
  → Central dispatches to the relevant Edge/Relay Satellite (C2 mesh)
    → Satellite actively queries the local device or OT platform
    → Satellite normalizes raw response to OCSF AT THE BOUNDARY (P-ADS-08)
    → Normalized result transits the conduit to Central (P-ADS-03)
      → Surfaces to user at the Central console (P-ADS-01)
```

**Reading A** adds nothing topologically new — an OT-platform REST API is reached edge-locally
the same way any other HTTP sensor API is reached. The satellite holding the platform credential
(AD-017) and performing the request is identical to existing connector behavior.

**Reading B** changes ONLY the satellite's LOCAL protocol from HTTPS to Modbus/OPC-UA/etc. The
conduit between Satellite and Coordinator carries OCSF-normalized results regardless of what
protocol the satellite used to acquire them. This is a packaging and capability concern (D-C14-7),
not a topology change.

**Per-tenant isolation:** Each satellite is tenant-scoped (P-ADS-06). OT data normalization at
the satellite boundary is per-tenant by construction — no cross-tenant OT data joins are possible.

---

### D-C14-7 — Reading-B Protocol Packaging: Plugins / Sidecar (NOT Core Satellite Binary)

**DECIDED 2026-06-27 (human).**

OT-protocol client libraries for Reading B (Modbus / OPC-UA / DNP3 / SNMP) ship as plugins via
the Wave-5 plugin SDK OR as sidecar processes. They are NOT compiled into the core satellite
binary.

**Rationale:**

| Concern | How plugin/sidecar addresses it |
|---------|--------------------------------|
| **Attack surface isolation** | OT-protocol parsers (Modbus, DNP3, OPC-UA) handle protocol messages from field devices that may be malformed, exploited, or actively adversarial (CISA ICS advisories document fragile stacks). Isolating them in a WASM sandbox (plugin) or separate process (sidecar) limits the blast radius of a parsing bug or supply-chain vulnerability. |
| **Supply-chain auditability** | OT-protocol Rust crates are new dependencies with smaller audit histories than the core satellite libs. Shipping as a plugin keeps them out of the core binary's supply-chain audit scope and allows per-plugin updates without rebuilding the satellite. |
| **Minimal core satellite** | Edge satellites run on constrained OT hardware. OT-protocol libs add non-trivial binary size and memory footprint. Plugin/sidecar loading is opt-in and avoids bloating every deployment with protocol stacks that most customers never use. |
| **P-ADS-11 Single-Codebase** | Protocol availability varies by deployment profile. Plugin/sidecar enables profile-appropriate protocol loading without code forks (feature-flag-gated plugin loading, not conditional compilation). |

**Packaging preference (pending F6 / OQ-C14-PACKAGING):**
- **WASM plugin (preferred):** Aligns with D-C4-3 WASM code-connector escape-hatch; memory-safe,
  sandboxed, same plugin API surface. Requires the OT-protocol lib to be compilable to WASM
  (`#![no_std]` or WASI-compatible). Some OT-protocol crates may not be WASM-compilable.
- **Sidecar process (fallback):** Runs the OT-protocol client as a separate OS process with a
  defined IPC interface (e.g., Unix socket, gRPC on localhost). Larger boundary surface than WASM
  but no WASM-compilability requirement. Supervisor (PAT-ADS-09) monitors the sidecar.

The final plugin-vs-sidecar choice is OQ-C14-PACKAGING (F6 in the research), gated on
evaluating actual WASM-compilability of candidate Rust OT-protocol crates at morph.

---

## Provable Invariants (PIV-C14-*)

These invariants must hold unconditionally in all C14 implementations. Violations are P0
adversarial findings.

| ID | Invariant | Enforcement Mechanism |
|----|-----------|----------------------|
| **PIV-C14-001** | Active-query read capability NEVER implies write capability. Writes are exclusively routed through C15 gated-action at the strictest autonomy tier. A connector with `read_only: false` is a configuration error, not a valid state. | `read_only: true` mandatory on every active-query descriptor; C15 routing; compile-time or boot-time assertion |
| **PIV-C14-002** | Central NEVER directly contacts a field device, OT platform, or any OT network endpoint. All active device queries execute at the tenant-scoped Edge Satellite. | C2 topology enforcement; no device-endpoint routing at Central |
| **PIV-C14-003** | Reading-B (OT-protocol) sources require explicit risk-based justification and non-production validation before production enablement. Default state is disabled. | Operator-gated enable flag; deployment runbook; no default-on |
| **PIV-C14-004** | OT data normalizes to OCSF at the satellite boundary. Raw PLC register reads, Modbus responses, or OT-platform vendor schemas NEVER transit the conduit to Central. | D-C14-6 / D-C2-12 structural enforcement; P-ADS-08 / INV-ADS-07 |
| **PIV-C14-005** | OT-protocol client libraries (Modbus / OPC-UA / DNP3 / SNMP) are isolated in plugins or sidecar processes. They are NOT compiled into the core satellite binary. | D-C14-7; plugin architecture; build system enforcement |
| **PIV-C14-006** | Cost-based-degrade (PAT-ADS-04) prefers cached satellite-local snapshots over re-polling rate-sensitive OT sources whenever the cached data is within the declared `freshness` TTL. | C3 degrade planner; `poll_cadence`/`freshness` descriptor fields |

---

## Open Questions (OQ-C14-*)

| ID | Question | Why not engineering | Status |
|----|----------|---------------------|--------|
| **OQ-C14-OCSF** | OCSF OT schema coverage: does OCSF have classes for OT asset inventory, config baselines/exceptions, PLC state, device-vuln mappings? Where do gaps require OCSF extension vs. custom schema-on-read? | Depends on `research/ocsf-ot-coverage-2026-06-27.md` (in-flight). Table structure in D-C14-4 is architectural; class-level assignments await research completion. | In-flight follow-up research; gating D-C14-4 class assignments |
| **OQ-C14-SAFETY-LIABILITY** | If a Prism Reading-B query contributes to a controller fault, who owns the risk (Prism/1898 / MSSP customer / industrial operator)? | Legal/insurance/customer-contract decision. Standards require risk-based justification but specify no safe poll numbers. Not an architect call. | Requires legal/sales/CS resolution before Reading-B ships to customers |
| **OQ-C14-CADENCE-NUMBERS** | What are safe poll-cadence / packet-rate defaults for Reading-B OT-protocol sources? | Original engineering — no standards-published numbers exist (IEC 62443, NIST SP 800-82, CISA all define principles but no specific cadence values). Must be validated in non-production OT environment. | Non-production validation required at morph |
| **OQ-C14-DESCRIPTOR-SCHEMA** | Exact capability-descriptor schema: field names, types, validation rules, `#[non_exhaustive]` expansion points. | Architect decision at morph. D-C14-3 specifies WHAT fields; the HOW of the schema (TOML key names, Rust struct layout, C3↔C4 reconciliation invariant integration) is implementation-level. | F5 in research; close at morph with architect + C3/C4 implementers |
| **OQ-C14-PACKAGING** | Final plugin-vs-sidecar packaging for OT-protocol client libs. WASM preferred but depends on `#![no_std]`/WASI-compilability of candidate Rust OT-protocol crates. | Requires evaluating actual candidate crates at morph (cannot determine compilability without examining specific libraries). | F6 in research; evaluate candidate crates at morph |

---

## ADS Conformance Checklist

```
CONFORMANCE CHECKLIST — ADR-PROP-active-query-devices.md (C14) — 2026-06-27

P-ADS-01: Central-Sole-Surface
  [x] User-interaction paths terminate at Central. Active-query results surface at the Central
      console (D-C14-6). Analyst does not interact with the satellite directly.
  [x] Satellite is strictly headless. Device polling executes at the Edge Satellite with no
      user-login surface (D-C14-6; C2 topology unchanged per ADR-PROP-satellite-mesh.md).

P-ADS-02: Operator-Zero-Access-At-Rest
  [x] Derived OT query results persisted at Central use the SS-26 per-tenant DEK (P-ADS-04 /
      PAT-ADS-02 applies universally to all Central-cached derived results).
  [x] Operator infrastructure holds ciphertext only. OT asset/vuln/config query results follow
      the same Tenant-Keyed-Central-Persistence pattern as all other derived results.

P-ADS-03: Derived-Results-Only-At-Central
  [x] OCSF-normalized OT results transit the conduit (D-C14-6). Raw PLC register reads,
      Modbus responses, OT-platform vendor schemas NEVER leave the satellite (PIV-C14-004).
  [x] No opt-in path where raw OT asset identifiers transit to Central (OCSF normalized =
      derived per P-ADS-03; individual OT field device addresses and register maps stay
      at the satellite).

P-ADS-04: Tenant-Keyed-Central-Persistence
  [x] OT query results follow the universal PAT-ADS-02 cache pattern (RocksDB hot / Iceberg
      cold, NOT PostgreSQL for query result caching).
  [x] Forensic replay distinction: this ADR-PROP concerns live OT query results, not
      Iceberg data-snapshots (OQ-C8-DATASNAPSHOT is a separate concern).

P-ADS-06: Per-Tenant-Isolation
  [x] Each satellite is tenant-scoped (P-ADS-06). OT normalization at the satellite boundary
      is per-tenant by construction. No cross-tenant OT data joins (PIV-C14-002: Central never
      aggregates across tenants' device endpoints).
  [x] No cross-tenant graph edges or similarity scores from OT data.

P-ADS-07: AI-Opaque
  [x] OT device credentials (Modbus device addresses, OPC-UA node credentials, SNMP community
      strings) resolve at the Satellite's local SecretBackend (D-C14-6; AD-017). They NEVER
      transit the conduit or the Central AI context (PIV-C14-002).
  [x] AI components (C12 GraphRAG, C7 ML, S3 agent) receive OCSF-normalized OT data or feature
      vectors only — not raw register reads or credential values.

P-ADS-08: OCSF-Normalize-At-Boundary
  [x] ALL C14 data sources normalize to OCSF at the satellite boundary (D-C14-6; PIV-C14-004).
      Reading A (OT-platform REST APIs) and Reading B (OT-protocol polling) both normalize at
      the Edge Satellite before any result transits the conduit. No trusted-source exemption
      (D-C4-1 binding per ADR-PROP-dynamic-schema-connectors.md).
  [x] OT OCSF version axis: OQ-C14-OCSF tracks the schema-coverage question. `metadata.version`
      carried for audit. The P-ADS-08 version-axis note applies.

P-ADS-09: Config-DB-Authoritative
  [x] No config-authoring path at the satellite or edge. OT source TOML descriptors are
      authored at the Central UI and pushed to satellites as signed bundles (PAT-ADS-03).
      Satellite does not author or modify its own capability-descriptor TOML in production.

P-ADS-10: Idempotent-Gated-Actions
  [x] Write/command paths are EXPLICITLY SEPARATED from the query fanout and route through
      C15 gated-action at the strictest autonomy tier (PIV-C14-001; D-C14-5 read-only invariant).
  [x] The read-only enforcement (`read_only: true` mandatory on every active-query descriptor)
      makes the query fanout structurally incapable of executing a write.

INV-ADS check (all eight):
  [x] INV-ADS-01: No raw sensor data at Central. OT data normalizes at satellite boundary
      (PIV-C14-004; D-C14-6).
  [x] INV-ADS-02: Operator zero-access at rest. Universal PAT-ADS-02 applies to OT derived
      results (per P-ADS-02 above).
  [x] INV-ADS-03: Per-tenant isolation enforced. Satellite is tenant-scoped; no cross-tenant
      OT device joins (PIV-C14-002).
  [x] INV-ADS-04: Config authored only at Central. OT source descriptors authored at Central
      UI/DB, pushed as signed bundles (P-ADS-09 above).
  [x] INV-ADS-05: Actions gated and idempotent. Write/command path routes to C15 ONLY
      (PIV-C14-001). Query fanout is structurally read-only.
  [x] INV-ADS-06: AI-opaque. OT device credentials never transit AI context (PIV-C14-002;
      AD-017). OCSF-normalized results or feature vectors only in AI components.
  [x] INV-ADS-07: OCSF normalization at ALL boundaries. Reading A and Reading B both normalize
      at the satellite (PIV-C14-004; D-C14-6; D-C4-1 no-exemption).
  [x] INV-ADS-08: Air-gap deployment is valid reference profile. Reading A (HTTP) and Reading
      B (OT-protocol) both execute at the satellite — no internet connectivity required for
      query execution. OT-protocol libs as plugins/sidecar (D-C14-7) load locally.
```

**CONFORMANCE RESULT: PASS** — All 8 INV-ADS invariants satisfied. No non-conformances.

**D-C14 HIGH-LIABILITY NOTE:** PIV-C14-001 (read-only perimeter / writes via C15 only) is the
primary safety control on the high-liability Reading-B path. This invariant must be independently
verified by adversarial review at both the capability-descriptor validation layer AND the C15
routing layer when E-OT-PROTOCOL-CONNECTORS-001 is implemented. Self-reported compliance by
the implementer is not authoritative (Standing Rule 3 §4; BC-5.39.001 3-CLEAN cascade applies).

---

## Honest Costs

| Cost | Description |
|------|-------------|
| **OT-safety obligation on Reading B** | Reading B (D-C14-2) places Prism in the active-poller-of-last-resort role. The §3.3 guardrails (D-C14-5) are mandatory engineering, not guidelines. Risk-based-justification + non-prod validation before production enablement is operational overhead. The CODESYS Forge PLC-crash incident is the reference for what happens when this is skipped. |
| **OQ-C14-SAFETY-LIABILITY (legal/contractual)** | Prism owning OT-safety risk on Reading B has legal and insurance implications that are NOT engineering-resolvable. This must be closed by legal/sales/customer-success before Reading-B ships. It is a genuine open item, not a deferral pattern. |
| **OQ-C14-OCSF gaps** | OCSF has no OT-specific classes as of 2026 (open proposal ocsf#1515). The D-C14-4 table model is structurally decided but the specific OCSF class assignments await OQ-C14-OCSF research closure. Until that research closes, OT table normalization may require custom or extension classes — which must not reintroduce the retired shadow enum (ADR-024) or bypass the unified type-mapping path (D-C4-6). |
| **WASM-compilability of OT-protocol crates** | D-C14-7 prefers WASM plugins but this depends on candidate crates supporting `#![no_std]` / WASI. Some Rust Modbus/OPC-UA/DNP3 crates may require full `std`; if so, sidecar is the fallback. Evaluate at morph — do not assume WASM-compilability without checking. |
| **New capability-descriptor fields (C3/C4 ripple)** | D-C14-3 extends the C3/C4 capability descriptor with five new fields. This is an additive extension (`#[non_exhaustive]` required) but it touches the schema that all existing connectors use. The C3↔C4 reconciliation invariant at boot (D-C4-9) must be extended to validate the new fields. |

---

## Alternatives Considered and Rejected

### Alternative: New First-Class "Active-Query Connector" Class

Model active-query devices as a distinct connector class in the connector taxonomy, separate
from the existing HTTP source adapters.

**Rejected because:** The federated-query literature is unanimous against this (Trino, Calcite,
Steampipe, Apollo all use capability flags on a unified interface, not a separate class).
A parallel connector taxonomy would create N×M maintenance surface — every cross-cutting concern
(normalization, predicate pushdown, cost degrade, per-tenant isolation) would need two
implementations. The unified-adapter model handles Reading A and Reading B identically at the
interface level; only the `protocol` field value differs.

### Alternative: Reading A Only in v1; Defer Reading B Indefinitely

Model C14 as exclusively API-tier (federated OT-platform REST APIs) and defer direct OT-protocol
polling to a future cycle.

**Considered** as the research recommendation (LEAN 2 in the research, high confidence). The human
chose BOTH in v1 (D-C14-1), establishing Reading B as in-scope and selecting Prism as the
poller-of-last-resort for customers with no OT platform. The decision is made; D-C14-5 guardrails
address the safety obligations that Reading-B-in-v1 creates.

---

## Ripple Effects (must be picked up at morph time)

| Affected area | Ripple |
|---------------|--------|
| **C3 capability-descriptor (ADR-PROP-capability-descriptor-pushdown.md)** | Add `active_query`, `protocol`, `poll_cadence`, `freshness`, `rate_limit`, `read_only` fields to the descriptor schema. Update C3↔C4 reconciliation invariant (D-C4-9) to validate these fields at boot. |
| **C4 dynamic-schema connectors (ADR-PROP-dynamic-schema-connectors.md)** | WASM code-connector escape-hatch (D-C4-3) is the preferred packaging path for Reading-B OT-protocol plugins. The OT-protocol WASM plugin uses the same plugin API surface as other WASM connectors. |
| **C2 satellite mesh (ADR-PROP-satellite-mesh.md)** | D-C14-6 reconfirms D-C2-12 structural residency enforcement. Add a note: Reading B changes the satellite's LOCAL acquisition protocol but not the conduit's data format (OCSF-normalized results only). D-C2-12's "OCSF normalization governs data FORMAT, not PII content" note applies equally to OT data. |
| **C11 Prism Intel (ADR-PROP-prism-intel.md)** | `ot_device_vulns` table (D-C14-4) is the OT-specific device vulnerability source. Correlates with the C11 CVE/EPSS feed (PAT-ADS-01 Feed-Down/Match-At-Edge). The match runs AT the satellite against local OT asset inventory — OT asset identifiers do not transit to Central (INV-ADS-01). |
| **C12 Prism Context (ADR-PROP-prism-context.md)** | `ot_assets` and `ot_device_state` tables feed the C12 knowledge graph (entity: OT device node). Per-tenant isolation required (PIV-C14-003; INV-ADS-03). |
| **C15 ARO Loop (referenced ADR-PROP)** | PIV-C14-001 makes the separation explicit: active-query read = C14 query fanout; any OT device write/command = C15 gated-action at strictest autonomy tier. C15 must explicitly enumerate OT commands as the highest-consequence action class. |
| **OCSF-normalize-at-boundary (P-ADS-08)** | New OT OCSF class assignments (gated on OQ-C14-OCSF) add new class mappings to the boundary-normalization registry. The D-C4-1 no-exemption rule applies — OT tables use the same normalization chokepoint as all other connectors. |
| **BC-2.16.002 (Canonical Structured Event Catalog)** | New `event_type` values emitted by the C14 active-query path (e.g., `ot.query.polled`, `ot.query.degrade`, `ot.safety.guardrail_applied`) must appear in the BC-2.16.002 Canonical Structured Event Catalog before PRs merge (SAP-1). |
| **E-ACTIVE-QUERY-001** | Epic covering Reading A: OT-platform northbound REST HTTP adapters + OT table modeling (`ot_assets`, `ot_config_baselines`, `ot_device_vulns`). Gated on OQ-C14-OCSF closure. |
| **E-OT-PROTOCOL-CONNECTORS-001** | Epic covering Reading B: direct OT-protocol satellite polling (Modbus/OPC-UA/DNP3/SNMP). Gated on OQ-C14-SAFETY-LIABILITY closure (legal) + OQ-C14-CADENCE-NUMBERS (non-prod validation) + OQ-C14-PACKAGING (crate WASM-compilability). |
| **New domain entities** | `OTDevice` (field device entity: protocol, address, register/tag map, read_only flag, risk-justification record); `OTPlatformAdapter` (northbound REST adapter: vendor, base URL, data types exposed). Add to domain-spec/entities.md at morph. |
| **New domain invariants** | "Active-query read never implies write (reads-only perimeter)"; "OT-protocol queries execute at satellite, never at Central"; "Reading-B source requires explicit risk-justification before production enablement." Add to domain-spec/invariants.md at morph. |
