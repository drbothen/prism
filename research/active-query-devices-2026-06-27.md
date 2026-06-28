# Active-Query Device Support (Industrial Defender Class) — Cited Research

| Field | Value |
|-------|-------|
| Date | 2026-06-27 |
| Type | general (technology + domain hybrid) |
| SIDE-ANALYSIS item | C14 — active-query device support |
| Mode | CAPTURE / research-only (do_not_execute). No live spec/BC/ADR/STATE/SESSION-HANDOFF modified. |
| Producer | research-agent |
| Status | complete |

> **Scope note.** This is a cited research pass to inform a future architectural decision about whether/how Prism should support devices and systems it ACTIVELY queries (request-response polling), of which Industrial Defender is the user's named exemplar. It produces no spec changes. The closing ANALYSIS + LEANS section presents the modeling options and flags the genuine sub-forks that require a human decision.

> **Citation discipline.** Vendor facts are date-stamped "as of 2026" and attributed to the deep-research passes summarized below. Where the deep-research model could only infer (not confirm) a fact from public docs, this is flagged explicitly as `[inferred]` or `[inconclusive]`. Architectural reasoning that is the research-agent's own synthesis (not a sourced claim) is marked `[synthesis]`.

---

## 0. Executive Summary

1. **The user's framing premise contains one factual error that must be corrected up front.** Industrial Defender is **NOT owned by GE Vernova** as of 2026. Ownership chain: Independent (2006) → Lockheed Martin (2014) → Capgemini → Teleo Capital (2020 spin-out, independent PE-backed) → Cuadrilla Capital (~2025). It is today an independent, U.S.-owned, private-equity-backed OT/ICS cybersecurity vendor headquartered near Boston (perplexity_ask, sources: industrialdefender.com/about-us; lockheedmartin news 2014; dialectica.io company profile). This does not change the C14 architecture question but corrects the record.

2. **Industrial Defender already exposes exactly the shape Prism federates today: a REST/JSON API.** ASM/IDCM (the "Industrial Defender Central Manager" since v7.4) exposes a RESTful JSON API at a base path `…/asmdataservice`, token-authenticated, returning data types `AdminProp` (asset administrative properties), `Exception` (baseline configuration deviations), and `Vulnerability` (CVE data); event data flows via syslog (`asm:event:syslog`). This is documented via the Splunk/ServiceNow/QRadar integration guides, not via a full public OpenAPI spec (research pass 1, sources: Industrial Defender Splunk app release notes; ID integration blog; ID FAQ). **Critical implication: an "Industrial Defender-class" device is, at the API tier, just another HTTP source adapter — the same connector shape Prism already federates for CrowdStrike/Claroty/Armis.** [synthesis]

3. **"Active query" in the OT vendor world is NOT what the user's phrasing might first suggest.** It does NOT (primarily) mean "Prism reaches down to a PLC over Modbus." In OT vendor parlance, *active query / Smart Polling / Safe Queries / EV-Agent collection* is a **collection method internal to the OT platform** (Nozomi/Claroty/Dragos/Armis/Industrial Defender) by which THAT platform safely interrogates OT devices. Prism consumes the **already-collected, already-normalized result** from the platform's northbound API. The "active vs passive" distinction lives at the *device↔OT-platform* boundary, not the *Prism↔OT-platform* boundary. [synthesis, grounded in research passes 1 & 2]

4. **Therefore there are two distinct readings of "support devices we actively query," and the human must pick which C14 means:**
   - **Reading A (API-tier active query, v1-appropriate):** Prism actively polls an OT-platform's REST API on demand (request-response) when a user runs a query, vs the federated-search default which is also already request-response. The novelty here is *modeling devices/asset-inventory as queryable source tables*, plus possibly a polling-cadence/cache axis. This is a thin extension to the existing adapter model.
   - **Reading B (protocol-tier active query, later/maybe-never):** Prism (at the edge/satellite) speaks OT protocols (Modbus/OPC-UA/DNP3/SNMP) directly to field devices. This is a genuinely new connector class with severe OT-safety obligations and is where IEC 62443 / NIST SP 800-82 guardrails bind hard.

5. **The federated-query connector-architecture literature (Trino SPI, Apache Calcite adapters, Steampipe plugin SDK, Apollo Federation) is near-unanimous: "active-query vs passive-ingest" is NOT modeled as a distinct connector CLASS. It is modeled as capability axes/flags on a unified adapter interface** (predicate pushdown, aggregation pushdown, supported operations, cost hints). None of these engines has a first-class "active-query connector" type; they presuppose active-query and treat passive-ingest as a separate ingestion pipeline (research pass 3, sources: Trino connector SPI docs; Calcite adapter docs; Steampipe plugin SDK; Apollo Federation docs; Enterprise Integration Patterns). **This strongly favors modeling active-query as a capability-descriptor axis (C3/C4) rather than a new connector class for Reading A.** [synthesis]

6. **OT-safety is a hard gate for Reading B and a soft consideration for Reading A.** Passive monitoring is the documented default in OT; active polling has caused real PLC crashes (a multi-year intermittent PLC-crash incident on CODESYS Forge was root-caused to a network scan). IEC 62443, NIST SP 800-82 Rev 3, and CISA all frame active scanning as a high-risk control requiring risk-based justification, read-only semantics, rate-limiting, connection limits, and maintenance-window scheduling — but none specify exact safe cadence/packet-rate numbers (those are vendor/integrator engineering, not standard) (research pass 4, sources: NIST SP 800-82 Rev 3; ISA/IEC 62443-3-3; CISA ICS advisories; Shieldworkz IEC 62443 guide; Nozomi/Claroty/Dragos technical guidance; ICS-LTU2022 dataset).

---

## 1. Q1 — Industrial Defender + Peer Active-Query Interfaces

### 1.1 Industrial Defender (the product) — data/API model

**As of 2026** (research pass 1, sources cited inline):

- **Product identity.** Originally "Automation Systems Manager (ASM)"; the central server is renamed **Industrial Defender Central Manager (IDCM)** as of v7.4, with field sensors called **Industrial Defender Collector (IDC)**. It is an OT-specific security & compliance platform — *not* a generic SIEM, *not* a pure NIDS — consolidating asset inventory, configuration/change management, vulnerability management, security monitoring, and compliance reporting (NERC CIP, NIST CSF) into one asset-centric system of record (sources: ID overview pages; ID FAQ; ID v7.4 press release; Splunk app release notes).
- **What it exposes (query surface):**
  - **Asset inventory** with rich attributes — software/firmware versions, known vulnerabilities, installed patches, firewall rules, and OT-specific physical state such as **PLC key-switch positions**; plus administrative properties (location, criticality, owner contact). Exposed via the `AdminProp` API data type. (sources: ID overview; ID Splunk partner brief; Splunk release notes)
  - **Configuration baselines + change events** via the `Exception` data type (deviations from baseline). (sources: Splunk release notes; ID integration blog)
  - **Vulnerability data** (CVE-mapped) via the `Vulnerability` data type. (sources: Splunk release notes; ID vuln-mgmt brief)
  - **Per-endpoint risk scores** combining unpatched vulns, security events, and health status (v7.4 "per endpoint risk calculations"). (source: ID v7.4 press release)
  - **Events/logs** primarily via **syslog** (`asm:event:syslog` source type), not necessarily via REST. (sources: Splunk release notes; ID integration blog)
  - **Compliance/policy status** — surfaced in UI/reports; programmatic API exposure is `[inferred, not confirmed]` from the FAQ's "API shares any data the system collects" claim. (source: ID FAQ — flagged inconclusive in research pass 1)
- **How it is queried:** RESTful JSON over HTTPS at base URL `https://<host>/asmdataservice`, token-based auth (tokens managed under System Administration > Settings > API Management). **Cadence is pull-based and externally scheduled** — the consuming tool (Splunk/QRadar/etc.) decides poll frequency; ID's API returns current state on demand (request-response). No public OpenAPI/Swagger spec was found; endpoint catalog is `[inferred]` from integration behavior. (sources: Splunk app release notes; ID integration blog)
- **How ID COLLECTS from OT devices** (the device↔ID boundary, distinct from the ID↔consumer boundary): integrated multi-modal — **active + agentless + passive**. The IDC combines active/agentless endpoint monitoring with a NIDS. Active collection `[inferred]` uses IT-management protocols (SNMP, WMI) for endpoints; OT-specific protocol use is implied by PLC-state collection but **not enumerated in public docs** `[inconclusive]`. Polling cadence numbers are **not published** (site-specific config). (sources: ID FAQ; ID v7.4 press release; ID "safe and complete data collection" blog)

### 1.2 Peer vendors — actively-queryable northbound interfaces (as of 2026)

All five expose **REST/JSON over HTTPS** northbound APIs an external system can query on-demand for asset inventory, vulnerabilities, and alerts. **GraphQL is uncommon-to-absent** across the board; the dominant model is REST/JSON, complemented by NDJSON log export (research pass 2).

| Vendor | Northbound API (Prism↔platform) | Primary stance (device↔platform) | Branded active-query feature | Active-query protocols (device-side) |
|--------|--------------------------------|----------------------------------|------------------------------|--------------------------------------|
| **Nozomi (Guardian/Vantage)** | OpenAPI/REST over HTTP on Guardian or Central Mgmt Console; query entities for assets/alerts/vulns | Passive-first (SPAN/TAP + DPI), "workhorse" | **Smart Polling** — agentless, low-volume, protocol-aware active polling; targetable per-segment/asset | "protocol-specific messages" / "various protocols" — Modbus/OPC-UA/EtherNet-IP/DNP3 + SNMP/WMI `[inferred]`; exact list not in snippet |
| **Claroty (CTD/xDome)** | **API Explorer** (CTD) for custom feeds/asset reports; xDome REST API w/ token, NDJSON alert export | Passive-first (SPAN/TAP + DPI) | **Safe Queries** — "Claroty's version of active scanning," targeted non-disruptive, returns firmware/patch levels; 1 of 5 collection methods | ICS + IT protocols `[inferred]`; "proven-safe, read-only, rate-limited, protocol-aware" framing |
| **Dragos Platform** | Web APIs (e.g., OSIsoft PI ingestion API documented); REST asset/alert APIs for SIEM/SOAR `[inferred for export]` | Passive-first across Purdue L1–L3.5; "gold standard" | **Extended Visibility Agent (EV Agent)** — host-based, controlled, deliberate, **read-only**, scheduled active collection on Windows OT hosts | Host-level (WMI, registry, service enum) primarily; network ICS protocols where used `[inferred]` |
| **Armis Centrix** | REST API w/ secret-token → access-key; integrates w/ Query Federated Search (Entities + Events incl. "Device Inventory Info"); device entities annotated w/ related CVEs | Historically agentless/passive discovery | **Selective safe active queries** — "90% less network impact than traditional scanners" | Not enumerated; "selective, reduced-volume" framing |
| **Industrial Defender** | REST/JSON `…/asmdataservice` (AdminProp/Exception/Vulnerability) + syslog events | Active + agentless + passive (multi-modal) | (collection methods, not a single brand) | SNMP/WMI `[inferred]`; OT protocols `[inconclusive]` |

**Cross-vendor takeaway** [synthesis grounded in passes 1 & 2]: Every peer is *passive-first* at the device boundary with a *branded, safety-marketed active-query supplement*, and every peer offers a *request-response REST API* northbound. **For Prism, "supporting an Industrial Defender-class device" overwhelmingly means consuming the northbound REST API** — which is the federated-read pattern Prism already implements. The platform owns the dangerous device-side active polling; Prism does not inherit that risk in Reading A.

---

## 2. Q2 — Active-Query vs Passive-Ingest Connector Patterns; Where Capability Descriptor Needs an Axis

### 2.1 Pattern definitions (research pass 3, sources: Enterprise Integration Patterns; Ably EDA; webhook docs)

- **Active-query (pull / request-response):** the engine initiates a request when a query needs data. Maps to EIP **Request–Reply** + **Polling Consumer**. The consumer controls *when* and *what scope* → natural backpressure (just delay/limit the next request), deterministic-ish load, optimizer-friendly (predicate/projection pushdown), freshness bounded by poll interval.
- **Passive-ingest (push / streaming):** the source (or an event broker) initiates. Maps to EIP **Event-Driven Consumer** + **Publish–Subscribe Channel**; webhooks are the canonical HTTP push. Lower latency / fresher, less idle traffic — but requires always-on availability, buffering, explicit backpressure, **idempotency/dedup** (retries → duplicates), and ordering handling. Heavier operational/observability burden (consumer lag, queue depth, silent callback failures).

### 2.2 When each is appropriate

| Dimension | Active-query (pull) favored | Passive-ingest (push) favored |
|-----------|----------------------------|-------------------------------|
| Freshness need | Periodic / on-demand analytical | Real-time alerting/control |
| Update frequency | Infrequent changes | Frequent / continuous |
| Load shape | Controllable, schedulable | Bursty, must buffer |
| Backpressure | Trivial (throttle requests) | Hard (broker flow control) |
| Idempotency | Source semantics handle it | Connector must dedup |
| Optimizer pushdown | Strong | Weak (data already arrived) |

**Prism's existing federated-read fanout is squarely active-query/request-response already** [synthesis]. The C2 satellite mesh, C3 capability-descriptor + cost-based-degrade pushdown, and C4 dynamic-schema connectors all assume the engine initiates the read. An "Industrial Defender-class active-query device" (Reading A) therefore fits the existing fanout **as another HTTP source adapter** — it is the same initiation model. The only genuinely new elements are: (a) modeling device/asset-inventory as queryable source tables with OCSF mapping, and (b) optionally a polling-cadence / freshness-cache axis if the source is rate-sensitive.

### 2.3 How comparable engines model connector capabilities (research pass 3)

- **Trino connector SPI:** declares **predicate pushdown, aggregation pushdown, join pushdown** support via interfaces/config; optimizer reads these flags + table stats for cost-based planning. No "active-query" connector class — active-query is presupposed; streaming sources are just tables to scan. (source: Trino connector & pushdown docs; OFFSET-pushdown GitHub issue showing extensibility)
- **Apache Calcite adapters:** declare relational capabilities + calling conventions (enumerable iterator); optimizer (100s of rules) decides pushdown by adapter capability + cost. Supports relational/semi-structured/**streaming**/geospatial models — but streaming is modeled as data-model heterogeneity, **not** as an "active vs passive" capability class. (source: Calcite adapter docs)
- **Steampipe plugin SDK:** external APIs as virtual SQL tables; `Get`/`List` **hydrate functions** retrieve on demand; SQL predicates pushed into API params. Inherently active-query; plugins *may* add caching/periodic ingest, but that's not a declared class. (source: Steampipe plugin SDK docs)
- **Apollo GraphQL Federation:** subgraphs declare resolvable fields/entities via schema directives; gateway orchestrates synchronous calls. Request-driven; subscriptions exist in GraphQL but are peripheral to the federation model. (source: Apollo Federation docs)

**Unanimous finding:** these systems model capability as **orthogonal axes/flags on a unified adapter interface**, and **none formalizes "active-query vs passive-ingest" as a first-class connector type.** Active-query is the default; passive-ingest is pushed to a separate pipeline (research pass 3, §5.4 explicitly: "they do not typically formalize a binary distinction between connectors that support 'active-query' versus 'passive-ingest' modes as a first-class capability descriptor").

### 2.4 Where Prism's C3/C4 capability descriptor needs an axis [synthesis]

If C14 = Reading A, the capability-descriptor (C3/C4) extensions are modest and additive:

- **`active_query` capability flag** (boolean/enum): does this source answer on-demand request-response asset/inventory queries? (Most already do — this may be implicit/default, consistent with Trino/Steampipe.)
- **`poll_cadence` / `freshness` hints**: min poll interval, recommended cache TTL, whether the source is rate-sensitive — feeds the **cost-based-degrade** logic (C3) so the planner can prefer cached snapshots over re-polling a rate-limited OT API.
- **`rate_limit` descriptor**: max requests/sec, concurrent-connection cap, backoff policy — distinct from HTTP_SEMAPHORE_PERMITS; per-source.
- **(Reading B only)** a `protocol` axis (`http` | `modbus` | `opcua` | `dnp3` | `snmp`) and a `read_only` safety assertion — this is where the new connector-class question genuinely arises.

---

## 3. Q3 — Safety of Actively Querying OT/ICS Devices

(research pass 4; binds hard only for Reading B, soft for Reading A)

### 3.1 Why passive > active in OT, with a real incident

- Passive monitoring (TAP/SPAN + DPI) is the documented OT **default** because ICS devices run legacy/brittle protocol stacks with poor input validation and operate under **watchdog timers** that interpret load-induced scan-cycle overruns as faults → reset to safe state. (sources: Nozomi passive-monitoring FAQ; ICS-LTU2022 academic dataset on improper input validation; PLC watchdog/fault-code practitioner material)
- **Documented incident:** engineers on the CODESYS Forge experienced intermittent PLC crashes *over several years*, eventually root-caused to a **network scan**. (source: CODESYS Forge thread, cited in research pass 4) — concrete evidence that even routine inventory/vuln scans can destabilize controllers and that such faults are hard to attribute.
- CISA ICS advisories document fragile stacks (e.g., Siemens Interniche IP-stack TCP-sequence validation weakness) where edge-case traffic → DoS — illustrating how scan traffic can inadvertently exercise latent bugs. (source: CISA ICS advisories)

### 3.2 What the standards say (and don't)

- **IEC/ISA 62443:** organizes IACS into **zones and conduits**; requires risk-based control selection that must not compromise availability/safety. Does **not** prescribe scan rates/protocols. Integrator interpretation (Shieldworkz IEC 62443 guide) explicitly advises **avoiding active scanning** (and vendors whose products rely exclusively on it) that "might disrupt real-time communication" — this is an *interpretation*, not a verbatim clause. (sources: ISA/IEC 62443-3-3; Cisco 62443-3-3 whitepaper; Shieldworkz guide; Dragos 62443 series)
- **NIST SP 800-82 Rev 3 (Guide to OT Security):** treats **active scanning as a control concept** but contextualizes with OT constraints — test in non-production first, consult device vendors, restrict production scanning to non-disruptive modes. The 2026 OT zero-trust overlay says risk assessments should integrate asset inventory + vuln scanning while remaining adaptable to OT constraints. (sources: NIST SP 800-82 Rev 3; OT zero-trust overlay)
- **CISA:** recommends limiting network exposure of control devices; implicitly disfavors heavy non-essential traffic to controllers. No specific safe-cadence numbers.
- **Key meta-finding for citation honesty:** **exact safe rate-limits/packet-rates/cadences are vendor/integrator engineering, NOT codified in any standard.** Standards define principles (risk-based, non-disruptive, read-only-preferred, scheduled); vendors (Nozomi Smart Polling "low-volume," Claroty Safe Queries "proven-safe," Dragos EV-Agent "read-only/scheduled") supply the engineering. Any Prism safety numbers would be Prism's own engineering, defensibly modeled on these.

### 3.3 Engineering guardrails for safe active polling (the checklist)

From research pass 4 (synthesis of vendor + standards guidance), the documented guardrails any Reading-B implementation MUST satisfy:

1. **Protocol-awareness** — only well-formed, spec-compliant requests; no fuzzing/malformed packets.
2. **Read-only enforcement** — no write/command function codes under any circumstance in the query path (writes belong to C15 gated-action, see §5).
3. **Rate-limiting** — low requests/min/device; stagger across devices to avoid simultaneous bursts.
4. **Connection management** — small, stable, long-lived connection set; respect device connection-table limits; avoid rapid open/close churn.
5. **Resource protection** — avoid verbose-logging-triggering or memory-heavy operations on constrained devices.
6. **Scheduling / maintenance windows** — coordinate with operations; avoid peak-criticality periods.
7. **Test/validate in non-production first** + continuous monitoring for latent impact (scan-time creep, watchdog near-faults, process-parameter drift).
8. **TAP > SPAN** for the passive baseline (TAPs are out-of-band, full-fidelity, config-change-resilient; SPAN drops under load and decays with switch reconfig).

---

## 4. Q4 — Edge Execution (ties C2)

(synthesis grounded in passes 1, 2, 4 + Prism design-system constraints)

The user's reconciliation question is correct and the resolution is clean:

- **Central-Sole-Surface is a USER-facing invariant**, not a network-reachability invariant. The user triggers a query at central; central is the sole surface the user touches.
- **Device access is edge-local.** The OT device (PLC/RTU, or the OT platform like Industrial Defender/Nozomi) lives in the customer/OT environment. The **satellite/edge** performs the active query (whether API-tier in Reading A or protocol-tier in Reading B); **central never directly touches the device**. This matches "Edge-Computes / Central-Surfaces."
- **Flow:** user query @ central → central dispatches to the relevant satellite (C2 mesh) → satellite actively queries the local device/OT-platform → satellite normalizes to **OCSF at the boundary** → normalized result returns to central → surfaces to user.
- **This is the SAME topology Prism already uses for federated reads.** Reading A adds nothing topologically new — the OT platform's REST API is reached edge-locally just like any sensor API. Reading B keeps the same topology but changes the satellite's *local protocol* from HTTPS to Modbus/OPC-UA/etc.
- **OCSF-at-boundary is critical for OT** [synthesis]: raw PLC register reads / Modbus responses / vendor asset schemas must normalize to OCSF *at the satellite*, exactly as Industrial Defender itself normalizes device data into its `AdminProp`/`Exception`/`Vulnerability` model before exposing it. Per-tenant isolation holds because each satellite is tenant-scoped.

**The design system already accommodates active-query devices without architectural violation.** The only edge-execution novelty is, for Reading B, hosting OT-protocol client libraries on the satellite — a packaging/capability concern, not a topology change.

---

## 5. Q5 — Active-Query for Actions/Writes (ties C15)

(synthesis grounded in pass 4 read-only guardrail + Prism's gated-action model)

- Some active-query devices/platforms support **writes/commands** (PLC logic download, set-point change, IED control). Research pass 4 is emphatic: **safe active polling is read-only by definition.** Write/command operations "can change control logic, configuration parameters, or state variables in ways that directly impact process behavior" and must be "explicitly authorized and scheduled under change control."
- **Therefore the clean line for Prism:** **read active-query and gated writes are DIFFERENT subsystems and must stay different.**
  - **Active-query (C14, read path):** request-response asset/inventory/config-read queries → fan-out → OCSF → surface. Safety = read-only enforcement + rate-limit. No human-in-the-loop required for a read (beyond normal authz).
  - **Gated writes (C15, action/command path):** any device command/write goes through the C15 gated-action model (autonomy gating, HITL/HOTL approval, audit, provenance). An OT write is the highest-consequence action class and should sit at the strictest gating tier.
- **The capability descriptor should make this non-negotiable:** an `active_query` read capability MUST NOT imply any write capability. Write/command capability is a separate, independently-gated descriptor that routes through C15, never through the query fanout. [synthesis]
- This mirrors the vendor world: Nozomi/Claroty/Dragos active-query features are read-only inventory/vuln enrichment; none of them frame active query as a write channel.

---

## 6. Q6 — Protocols: API-level (v1) vs Protocol-level (later)

(synthesis grounded in passes 1, 2, 4)

- **OT protocols that imply an active-poll model:** Modbus, OPC-UA, DNP3, EtherNet/IP, SNMP, WMI. These are request-response by nature (poll a register, read a tag, GET an OID). They are the *device-side* protocols the OT platforms use internally for their active collection.
- **The pivotal observation:** **Prism does not need to speak any OT protocol to "support an Industrial Defender-class device."** Industrial Defender (and Nozomi/Claroty/Dragos/Armis) already do the OT-protocol active polling and expose the normalized result over **HTTPS REST/JSON**. Prism federating that REST API gets full OT asset/vuln/config visibility **with zero OT-protocol risk inherited by Prism**.
- **v1 recommendation (research-leaning, see §7):** **API-level only.** Build the active-query device support as HTTP source adapters against OT-platform northbound APIs (Industrial Defender `asmdataservice`, Nozomi OpenAPI, Claroty API Explorer/xDome, Dragos web API, Armis Centrix API). This is in-scope of the existing federated-read model, carries no OT-safety obligation on Prism, and covers the user's exemplar exactly.
- **Protocol-level (later/maybe-never):** direct Modbus/OPC-UA/DNP3/SNMP from the satellite to field devices is a *genuinely new connector class* with the full §3.3 guardrail obligation, IEC-62443/NIST-800-82 risk-assessment burden, and liability exposure. It should be a deliberate, separately-scoped future decision — and may be unnecessary if the customer already runs an OT platform whose API Prism can federate.

---

## 7. ANALYSIS + LEANS

### 7.1 How active-query support should be modeled in Prism

**LEAN 1 — Adapter MODE/capability axis, NOT a new connector class (for Reading A).**
The federated-query literature is unanimous (Trino/Calcite/Steampipe/Apollo all model capability as flags on a unified adapter, none has an "active-query class"). Prism's federated fanout is already request-response. An Industrial Defender-class device at the API tier is just another HTTP source adapter. → Model active-query as a **capability-descriptor axis on the existing adapter interface (C3/C4)**: `active_query` flag + `poll_cadence`/`freshness` + `rate_limit` hints feeding cost-based-degrade. Avoid a parallel connector taxonomy. **Confidence: HIGH** for Reading A.

**LEAN 2 — API-level for v1; protocol-level deferred as a distinct future decision.**
v1 = HTTP adapters against OT-platform northbound REST APIs. This covers the user's named exemplar (Industrial Defender's `asmdataservice`) and all peers, inherits zero OT-protocol safety risk, and fits the existing model. Protocol-level (Modbus/OPC-UA/DNP3/SNMP direct-to-device) = separately-scoped later class. **Confidence: HIGH.**

**LEAN 3 — Model device/asset-inventory as first-class queryable OCSF source tables.**
The genuinely new modeling work (even in Reading A) is representing OT asset inventory, configuration baselines/exceptions, and device vulnerability state as PrismQL-queryable tables normalized to OCSF at the satellite boundary — mirroring Industrial Defender's own `AdminProp`/`Exception`/`Vulnerability` decomposition. **Confidence: MEDIUM-HIGH** (depends on OCSF class coverage for OT asset/config — see sub-fork F4).

**LEAN 4 — Edge-executed, OCSF-at-boundary, read-only; reuse C2 topology unchanged.**
Active queries originate at the tenant-scoped satellite; central never touches the device; result normalizes to OCSF at the edge. No topology change vs existing federated reads. **Confidence: HIGH.**

**LEAN 5 — Keep read active-query strictly separate from gated writes (C15).**
`active_query` read capability MUST NOT imply write capability. Any OT command/write routes through C15 gated-action at the strictest autonomy tier, never through the query fanout. **Confidence: HIGH.**

**LEAN 6 — Bake the §3.3 guardrails into the capability descriptor and degrade logic, even for API-tier.**
Rate-limit, connection-cap, and freshness-cache hints protect rate-sensitive OT-platform APIs (and become mandatory if Reading B is ever pursued). Cost-based-degrade (C3) should prefer cached snapshots over re-polling a rate-limited source. **Confidence: MEDIUM-HIGH.**

### 7.2 Genuine sub-forks requiring a HUMAN decision

| # | Sub-fork | Why it needs a human | Research lean |
|---|----------|----------------------|---------------|
| **F1** | **Reading A vs Reading B scope for C14.** Does "support devices we actively query" mean (A) federate OT-platform northbound APIs, or (B) Prism directly speaks OT protocols to field devices? | Fundamentally different risk/liability/scope; the user's phrasing ("like an Industrial Defender") is API-shaped but "devices we actively query" could be read as protocol-direct. This is a product-scope + risk-acceptance decision. | Lean A for v1; B as deliberate later class. **Inconclusive without human intent.** |
| **F2** | **Does Prism ever want to be the active-poller of last resort** (customer has NO OT platform), or only a federation layer over existing OT platforms? | If customers without Nozomi/Claroty/ID exist, Reading B becomes necessary and Prism inherits full OT-safety liability. Business/market call. | Research can't decide; depends on MSSP customer base (1898 & Co context). |
| **F3** | **Acceptable OT-safety liability posture.** If Reading B: who owns the risk when a Prism active query contributes to a controller fault? | Standards require risk-based justification but specify no safe numbers; this is a legal/insurance/customer-contract decision, not an engineering one. | Standards favor passive-first; if B pursued, mandatory §3.3 guardrails + non-prod validation + customer sign-off. |
| **F4** | **OCSF coverage for OT asset/config/baseline data.** Does the OCSF schema (as Prism uses it) have classes that cleanly represent OT asset inventory, config baselines/exceptions, PLC state, device-vuln mappings? | If OCSF gaps exist for OT, normalization design needs architect input on extension vs custom classes. | Flagged as **inconclusive** — not researched here; recommend a follow-up OCSF-OT-coverage pass before C14 implementation. |
| **F5** | **Capability-descriptor granularity.** Is `active_query` a single boolean, or a richer enum (`on_demand_read` / `cached_read` / `streaming`)? Does it merge with existing C3/C4 descriptors or add a new sub-schema? | Architect-owned schema decision; affects every adapter. | Lean: enum axis on unified interface (per Trino/Calcite precedent), but exact schema is architect's call. |
| **F6** | **Reading B connector packaging.** If protocol-level is ever pursued, do OT-protocol client libs ship in the satellite binary, as plugins (Wave-5 plugin SDK), or as sidecar processes? | Ties to plugin architecture + satellite footprint + supply-chain/audit of OT-protocol libs. | Defer with F1; if B, lean plugin/sidecar to isolate OT-protocol attack surface. |

### 7.3 Inconclusive / not-found flags

- Industrial Defender's full REST endpoint catalog and exact OT protocols used for active collection: **not in public docs** (`[inconclusive]`; only Splunk-integration-derived data types + `asmdataservice` base path confirmed).
- ID compliance/policy-status programmatic API exposure: `[inferred]` from FAQ, not confirmed.
- Exact "safe" active-poll cadence/rate numbers for any vendor or standard: **do not exist publicly** — vendor engineering, not codified. Any Prism numbers would be original engineering.
- OCSF schema fitness for OT asset/config (F4): **not researched** — recommend follow-up pass.
- Peer vendors' exact active-query protocol lists: `[inferred]` (Modbus/OPC-UA/EtherNet-IP/DNP3 + SNMP/WMI); not enumerated verbatim in retrieved snippets.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 4 | (1) Industrial Defender product/data/API/collection model; (2) peer-vendor (Nozomi/Claroty/Dragos/Armis/ID) active-query northbound interfaces; (3) active-query vs passive-ingest connector architecture patterns + capability modeling (Trino/Calcite/Steampipe/Apollo/EIP); (4) OT/ICS active-query safety standards (IEC 62443, NIST SP 800-82 Rev 3, CISA) + guardrails + TAP/SPAN. All at reasoning_effort=high. |
| Perplexity perplexity_ask | 1 | ≤2-sentence factual lookup: Industrial Defender corporate ownership (correcting the GE Vernova premise). |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Context7 | 0 | — |
| Tavily (all) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | ~2 areas | OCSF/Prism-architecture framing context and EIP pattern names (cross-checked against pass-3 sources); flagged as `[synthesis]` where used for reasoning rather than sourced claims. |

**Total MCP tool calls:** 5 (4 perplexity_research + 1 perplexity_ask)
**Training data reliance:** low — all vendor facts, standards, and connector-architecture claims are sourced to the four deep-research passes (with their inline numeric citations) and the ownership ask; training data is used only for architectural synthesis explicitly marked `[synthesis]` and for naming well-established EIP patterns that the pass-3 sources independently confirm.

### Source inventory (as surfaced by the deep-research passes; primary domains)

- **Industrial Defender:** industrialdefender.com (overview, FAQ, about-us, vuln-mgmt brief, integration blog, v7.4 press release, Splunk app + REST API add-on release notes); industrialcyber.co vendor profile; lockheedmartin.com 2014 acquisition news; dialectica.io company profile (ownership).
- **Peer vendors:** Nozomi Networks (OT asset inventory mgmt, Smart Polling overview + strategies reference, passive-monitoring FAQ); Claroty (Safe Queries, API Explorer feature spotlight, xDome/Hunters integration guide); Dragos (asset visibility & inventory, passive-monitoring & active-collection blog, ISA/IEC 62443 series, platform release notes); Armis (Centrix VM announcement, Query Federated Search integration, Intelligence Center docs).
- **Standards/safety:** NIST SP 800-82 Rev 3 + 2026 OT zero-trust overlay; ISA/IEC 62443-3-3 (Cisco whitepaper, Dragos explainer, Shieldworkz implementation guide); CISA ICS advisories; ICS-LTU2022 academic dataset; CODESYS Forge PLC-crash thread; PLC watchdog/fault-code practitioner sources; Garland Technology + Industrial Cyber TAP-vs-SPAN.
- **Connector architecture:** Trino connector SPI + pushdown docs + OFFSET-pushdown GitHub issue; Apache Calcite adapter docs; Steampipe plugin SDK; Apollo GraphQL Federation docs; Enterprise Integration Patterns (Request-Reply, Polling Consumer, Event-Driven Consumer, Publish-Subscribe); Ably EDA; Amazon Athena federated-query predicate-pushdown docs; webhook integration docs.
