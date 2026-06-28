# OCSF Schema Coverage for OT/ICS Asset, Configuration, and Device-Vulnerability Data

**Research type:** general (technology / schema coverage)
**Date:** 2026-06-27
**Author:** research-agent (Corverax / Prism)
**Mode:** CAPTURE / research-only (do_not_execute). No live spec/BC/ADR/STATE/SESSION-HANDOFF touched.
**Context fork:** C14 sub-fork F4 follow-up — Prism normalizes all sources to OCSF at the adapter boundary; C14 decided Prism supports active-query OT devices via BOTH OT-platform northbound APIs (Reading A) AND direct field-device polling via OT protocols (Reading B). The open question: does OCSF cleanly represent OT/ICS data (asset inventory / config baselines+exceptions / PLC-RTU state / device-vulnerability), or are extensions / custom classes needed?

> **Source-grounding note.** Findings below are grounded in official OCSF sources — `schema.ocsf.io` (v1.8.0 live browser), the `ocsf/ocsf-schema` GitHub repo (releases, issues, CHANGELOG), and `ocsf/ocsf-docs`. Where a claim rests on near-official vendor docs (Splunk, AWS, Query.ai, VirtualMetric, DataBee) rather than the canonical schema, it is flagged. Version/date claims were verified live against the OCSF release page and schema browser on 2026-06-27. Inconclusive areas are called out explicitly.

---

## 1. Current OCSF version + relevant categories/classes (2026)

**Verified current release: OCSF v1.8.0, published 2026-03-18** (confirmed live against the `ocsf/ocsf-schema` GitHub releases page and the `schema.ocsf.io` browser, which shows v1.8.0 with a v1.9.0-dev branch in flight). This **confirms the prior-research note** that the latest OCSF version is v1.8.0 (2026-03-18) and that a `data_classification` profile ships (see §7).

Recent release line (verified against the releases page):

| Version | Date | Headline additions relevant to Prism |
|---------|------|--------------------------------------|
| **1.8.0** | 2026-03-18 | AI Operation profile + objects, macOS extension, GPU/privilege analysis, Packet object |
| 1.7.0 | 2025-11-14 | **Peripheral Activity** class, Windows service enhancements, `reporter` metadata |
| 1.6.0 | 2025-08-01 | **IAM Analysis Finding** class (2008), email attributes, `port_info` object |
| 1.5.0 | 2025-04-28 | **Application Security Posture Finding** (2007), **Live Evidence Info** (5040), graph/anomaly objects; **expanded the Data Classification profile across databases, emails, files** |
| 1.4.0 | (2024) | **Unmanned Systems** category (drone/aviation) + new profile — shows OCSF *will* admit vertical/domain categories |

### Categories (8 primary) and the classes that could carry OT data

OCSF organizes telemetry into eight primary categories (System Activity 1xxx, Findings 2xxx, IAM 3xxx, Network Activity 4xxx, Discovery/Inventory 5xxx, Application Activity 6xxx, Remediation 7xxx, Unmanned Systems — plus Audit). The classes Prism would map OT data onto:

**Discovery & Inventory (5xxx)** — verified live in the v1.8.0 browser:
- `Device Inventory Info` (5001) — asset identity/inventory
- `User Inventory Info` (5003)
- `Operating System Patch State` (5004)
- `Device Config State` (5002) + `Device Config State Change` (5019) — configuration snapshots + change events
- `Software Inventory Info` (5020, since v1.3)
- `OSINT Inventory Info` (5021, since v1.3) — threat-intel inventory
- `Cloud Resources Inventory Info` (5023)
- `Live Evidence Info` (5040, since v1.5)

**Findings (2xxx)** — verified live in the v1.8.0 browser:
- `Vulnerability Finding` (2002) — device↔CVE/CWE (see §5)
- `Compliance Finding` (2003) — control/baseline conformance + exceptions (see §4)
- `Detection Finding` (2004)
- `Incident Finding` (2005)
- `Data Security Finding` (2006) — DLP/DSPM/data-classification alerts
- `Application Security Posture Finding` (2007)
- `IAM Analysis Finding` (2008)

These are the load-bearing classes for Prism's OT decomposition (mirroring Industrial Defender's AdminProp / Exception / Vulnerability split):
- **Asset inventory (AdminProp)** → `Device Inventory Info` (5001) + `Software Inventory Info` (5020)
- **Config baseline + exceptions (Exception)** → `Device Config State` (5002) / `Device Config State Change` (5019) for the raw state/drift data + `Compliance Finding` (2003) for the baseline-conformance verdict
- **PLC/RTU state** → no native class; modeled as Device Inventory Info / Device Config State attributes or Network Activity (see §2)
- **Device vulnerability (Vulnerability)** → `Vulnerability Finding` (2002)

---

## 2. OT/ICS-specific coverage — the central gap

**Verdict: OCSF v1.8.0 has NO OT/ICS-native classes, no OT device types, and no OT-native attributes in the core schema.** OT must currently be mapped onto generic Device/Inventory/Finding/Network classes.

Verified gaps:
- **Device `type_id` enum** (confirmed against schema browser): contains IT/infra types — Server, Desktop/Workstation, Laptop, Mobile, Router, Switch, Firewall, plus generic IoT/VM/container types. **It does NOT include PLC, RTU, HMI, or "industrial controller."**
- **Device object fields**: hostname, IPs, MAC, OS info, identifiers — but **NO `firmware_version`, NO `network_zone`/`security_zone`, NO Purdue-level field** in the base Device schema.
- **No safety-state attribute**, no control-relationship modeling (e.g., this PLC controls that I/O module), no ladder-logic/program-version field.

### OT/ICS working-group activity (this is the important live signal)

There **is active, but early, OCSF community work on ICS** — found in the `ocsf/ocsf-schema` issue tracker:

- **Issue #1515 — "[Proposal] Industrial Control System (ICS) Field Extensions"** (opened 2025-10-16 by Jeremy Wiley / Corelight; labels: `enhancement`, `network_activity`). **Status: OPEN, no assignee, no milestone, no PR, no visible maintainer response as of research date.** Scope is narrow and protocol/network-oriented — it proposes **6 optional attributes**, NOT new classes or device types:
  - `ics.function_code` (int) — protocol operation id
  - `ics.pdu_type` (string) — request/response
  - `ics.pdu_service` (string) — e.g. read_property/write_property
  - `ics.unit_id` (int) — device id
  - `ics.register_address` (int) — memory location accessed
  - `ics.object_type` (string) — analog_input/binary_output
  - Protocols named: **Modbus, DNP3, BACnet, S7comm, EtherNet/IP** (no OPC-UA in the proposal text).
  - **Explicitly does NOT address Purdue layers or PLC/RTU/HMI equipment classification.**
- Issue #1545 — "More generic 'compute environment' object" (opened 2025-12-09, labeled `v2.0 and later`) — tangential; signals OCSF is rethinking asset/environment modeling but not OT-specifically.

**Interpretation for Prism:** The only OCSF OT work in flight is a *network-protocol-attribute* proposal scoped to Network Activity. It does NOT cover OT **asset inventory, config baselines, device classification, or device-level state** — exactly the data types C14's Reading A/B decomposition centers on. So even if #1515 lands, Prism's OT asset/config/state modeling is still on its own. **(Inconclusive flag: #1515 has no maintainer engagement; do not assume it merges or that its `ics.*` namespace becomes canonical.)**

---

## 3. Device / asset modeling (asset identity, zone, criticality, firmware)

`Device Inventory Info` (5001) carries asset identity well for IT: device identifiers, hostname, IPs/MAC, OS info, and references to owner/org. For OT, the gaps are concrete:

| OT asset attribute | OCSF core coverage | Gap |
|--------------------|--------------------|-----|
| Identity (hostname, IP, MAC, serial) | `device` object fields | OK |
| Device type (PLC/RTU/HMI) | `type_id` enum is IT-only | **GAP** — no enum value; would need extension enum or `type` free-string |
| Firmware version | none in base `device` | **GAP** — closest is `os`/`hw_info`; no firmware semantics |
| Network zone / Purdue level | none | **GAP** — must use a custom attribute or `location`/`network_zone`-style extension |
| Criticality | partial — `risk_level_id`/`risk_score` exist on some finding paths, not a first-class device-criticality field | **PARTIAL** |
| Control relationships (controls/controlled-by) | none | **GAP** — no graph edge for I/O modules or supervisory relationships (the v1.5 graph objects are finding-oriented, not asset-topology) |
| Ladder-logic / program version | none | **GAP** |

Conclusion: identity maps cleanly; **OT-specific physical/control semantics (type, firmware, zone, control topology, program version) have no home in the base Device object.**

---

## 4. Config baseline + exception modeling (Industrial Defender's core value)

This maps **better than expected** — OCSF has a purpose-built triad:
- `Device Config State` (5002) — current/observed configuration snapshot of a device (config items/parameters, device reference, optional baseline association).
- `Device Config State Change` (5019) — change event between two config states (what changed, when, by whom) → **config drift** trail.
- `Compliance Finding` (2003) — the *verdict*: control/policy/baseline id, compliance status (compliant / non-compliant / **exception** / not-evaluated), evidence/reason, device reference.

So Industrial Defender's "baseline + exception" model decomposes naturally:
- **Baseline / current config** → Device Config State (5002)
- **Drift over time** → Device Config State Change (5019)
- **Exception / deviation verdict against a baseline** → Compliance Finding (2003), whose status enum already includes an exception concept.

**Caveat:** these classes provide the *data and verdict containers* but are schema-generic — the **semantics of what a "baseline" is for an OT controller** (firmware level, expected register map, allowed program version) are not modeled; they'd live in untyped config-item key/value pairs. Good enough to carry the data; not self-describing for OT specifics. **Verdict: fits cleanly for structure; OT-specific baseline semantics need convention or light extension.**

---

## 5. Vulnerability modeling (device ↔ CVE ↔ CPE; ties C11 Prism Intel)

**Maps cleanly.** `Vulnerability Finding` (2002), verified against the v1.8.0 schema browser:
- **`vulnerabilities[]`** — required array of **Vulnerability Details** objects.
- **`device`** attribute — the affected device/host; plus `resources[]` (recommended) for affected resources.
- **`finding_info`** (required), `severity_id`, `confidence_id`, `status_id`, `observables`, `enrichments`.

The **Vulnerability Details** object (verified) carries: `cve`, `cwe`, `affected_packages`, `affected_code`, `references`, `fix_coverage`, `vendor_name`, `related_vulnerabilities`. Note the constraint: **`advisory`, `cve`, `cwe` are mutually exclusive** (exactly one present per record). The `cve` object references MITRE CVE details.

**Gaps to flag for OT:**
- **CPE:** No explicit `cpe` field was found on the Vulnerability Details object — affected product is expressed via `affected_packages` / `vendor_name`, not a CPE string. For OT, where CPE matching against ICS-CERT advisories and firmware versions is common, **this is a real gap** — Prism would carry CPE in `affected_packages` package metadata or a custom attribute. (Inconclusive flag: a `cpe`/product sub-field may exist deeper in the package object than the browser excerpt exposed; verify against the raw `vulnerability` object JSON before finalizing.)
- ICS-CERT / vendor-advisory identifiers beyond CVE → use the `advisory` variant (mutually exclusive with cve, so a finding referencing both a CVE and an ICS-CERT advisory needs two Vulnerability Details entries).

**Verdict: device↔CVE↔CWE fits cleanly; device↔CPE needs a convention (affected_packages) or a small extension.**

---

## 6. OCSF extension mechanism (extension vs profile vs custom class; cost/process)

Verified against `ocsf/ocsf-docs` (`patching-core-using-extensions.md`) and the CHANGELOG (macOS/Windows extensions as worked examples):

**Extensions** are packaged schema add-ons under a top-level `extensions/<name>/` directory. Each can:
- define new classes/objects/attributes (each with unique `uid`, `name`, `caption`),
- **patch core objects** (add attributes to e.g. `device`/`process` without forking core),
- introduce profiles.
UIDs must be globally unique; upstream extensions get a coordinated non-overlapping UID block. Private extensions reserve a high-numbered private range. **Compatibility rule:** patches should only *add*; an event with extension attributes must still validate as core OCSF when the extension is absent (unknown attributes ignored).

**Profiles** are cross-cutting attribute overlays (mix-ins) applied across classes via a `profiles` array + per-attribute `profile` clause — e.g. Cloud, Container, OSINT, Security Control, **Data Classification**. A profile is lighter than an extension: no new class hierarchy, just additional attributes that "light up" when the profile applies. An extension can *define* a profile.

**Three Prism options, ordered by cost:**

| Approach | What it is | Cost | Reversibility / upstream risk |
|----------|-----------|------|-------------------------------|
| **Use-generic + conventions** | Map OT onto Device Inventory/Config State/Compliance/Vulnerability with free-string `type`, key/value config items, package-encoded CPE | Lowest — no schema authoring | Lossy/untyped; queries over OT semantics rely on convention, not schema |
| **Private OCSF extension/profile** (`prism_ot`) | Define an `extensions/prism_ot/` package: a `prism_ot` profile adding `device.firmware_version`, `device.purdue_level`, `device.ot_device_type` (PLC/RTU/HMI enum), `ics.*` protocol attrs; patch the `device` object | Medium — author + maintain JSON, reserve private UID range, wire into Prism's normalizer | Self-contained; if OCSF later ships official ICS support, may need UID remap / migration |
| **Contribute upstream** | PR the extension into `ocsf/ocsf-schema`, coordinate UID block, survive review | Highest — community review, UID negotiation, slow; #1515 shows ICS work stalls without maintainer pickup | Canonical if merged; but timeline-uncontrollable and currently no maintainer momentum on ICS |

---

## 7. The Data Classification profile (verify shipped + version) — ties C16 RSI

**Confirmed shipped.** The `schema.ocsf.io` v1.8.0 browser lists **14 profiles**, including **"Data Classification"** explicitly (alongside AI Operation, Cloud, Container, Date/Time, Host, Incident, Linux Users, Load Balancer, Network Proxy, OSINT, Security Control, Trace, macOS Users). The profile predates v1.8 and was **expanded in v1.5.0 (2025-04-28) across databases, emails, and files** (per the release notes). The related `Data Security Finding` class (2006) is the finding-side counterpart for DLP/DSPM/classification alerts.

**Usefulness for tagging OT-sensitive / BCSI fields (C16 RSI):** The Data Classification profile gives Prism a *standard, schema-blessed* way to attach classification labels/sensitivity to objects (file/email/database and, since it's a profile, applicable where registered). For BCSI / OT-sensitive tagging this is the right primitive — but note it is **data-content classification** (is this data sensitive?), not **asset-criticality classification** (is this PLC safety-critical?). The two are distinct: RSI/BCSI tagging of *query results and fields* fits the Data Classification profile cleanly; OT *asset* criticality remains a §3 device-modeling gap. **(Verify the exact attribute set the profile contributes before committing C16 to it.)**

---

## ANALYSIS + LEANS

### Per-data-type verdict

| OT data type | OCSF fit | Verdict |
|--------------|----------|---------|
| **Asset inventory** (identity) | Device Inventory Info (5001) | **Fits cleanly** for IT-style identity |
| **Asset inventory** (OT semantics: device-type PLC/RTU/HMI, firmware, Purdue zone, control topology) | none in base | **Needs extension** (profile + `device` patch) |
| **Config baseline + drift** (structure) | Device Config State (5002) / State Change (5019) | **Fits cleanly** structurally |
| **Config exception** (verdict vs baseline) | Compliance Finding (2003), incl. exception status | **Fits cleanly** structurally; OT baseline *semantics* untyped |
| **PLC/RTU state** (runtime/program state, safety state) | none | **Needs extension or custom modeling** (no class; closest is Device Config State attributes) |
| **Device vulnerability** (device↔CVE↔CWE) | Vulnerability Finding (2002) + Vulnerability Details | **Fits cleanly** |
| **Device vulnerability** (CPE matching) | no explicit cpe field | **Needs convention/extension** (affected_packages) |
| **OT-protocol metadata** (Modbus/DNP3 fields) | none core; Issue #1515 proposes `ics.*` | **Needs extension** (and #1515 is stalled) |
| **RSI/BCSI data tagging** (C16) | Data Classification profile (shipped, v1.5+) | **Fits cleanly** for content sensitivity |

### Recommended Prism OT-normalization approach (research lean — not a binding decision)

**Hybrid: use-generic-classes + a private Prism OCSF extension/profile (`prism_ot`), NOT upstream-first.**

1. **Reuse the existing class skeleton** — Device Inventory Info (5001), Device Config State (5002/5019), Compliance Finding (2003), Vulnerability Finding (2002) — as the PrismQL-queryable OCSF source tables. This is where OCSF genuinely fits cleanly and mirrors Industrial Defender's AdminProp/Exception/Vulnerability decomposition with no schema work.
2. **Author a private extension `prism_ot`** that (a) defines a `prism_ot` profile adding the missing OT attributes — `ot_device_type` (PLC/RTU/HMI/safety-controller enum), `firmware_version`, `purdue_level`, `network_zone`, optional control-relationship references; and (b) patches the `device` object so these light up on Device Inventory Info / Config State. Reserve a private UID block per OCSF rules so a future upstream migration is mechanical.
3. **Adopt the Data Classification profile as-is** for C16 RSI/BCSI field tagging — it ships and is the schema-blessed primitive.
4. **Track Issue #1515** for `ics.*` network-protocol attributes; if Reading B (direct protocol polling) emits protocol-level telemetry, align Prism's attribute names to #1515's `ics.*` namespace defensively so a future merge is low-friction — but do NOT block on upstream.

Rationale: upstream-first is the wrong default here — ICS work in OCSF is stalled (no maintainer engagement on #1515 as of 2026-06-27), and C14/C11 timelines can't be held hostage to community review. A private extension that *follows OCSF conventions and reserves clean UIDs* preserves the option to contribute upstream later without forcing Prism to invent off-schema custom classes (which would break the "everything is OCSF at the adapter boundary" invariant). This is consistent with Prism's production-grade default: model OT data in a standard-aligned, self-describing way now, rather than dumping it into untyped key/value config items "for later."

### Genuine sub-forks needing a human decision

- **SF-1 (extension scope):** Should `prism_ot` be a **profile-only** add (attributes on existing classes — cheapest, keeps everything in 5001/5002) OR introduce **new OT classes** (e.g., a PLC-State class) for PLC/RTU runtime state that has no good existing home? Profile-only is leaner; new classes are more self-describing for PrismQL queries. Architect + product decision.
- **SF-2 (upstream posture):** Contribute `prism_ot` upstream eventually (canonical, but slow, and exposes Prism's OT model publicly) vs keep it permanently private (full control, but consumers outside Prism won't recognize the attributes). Business/strategy decision.
- **SF-3 (CPE handling):** Carry OT CPE in `affected_packages` (convention, no schema change) vs add a `cpe` attribute via the extension (typed, queryable). Ties C11 Prism Intel's CVE↔CPE matching design — decide jointly. **First verify** whether the v1.8 `vulnerability`/package object already exposes a CPE/product sub-field (the browser excerpt was inconclusive).
- **SF-4 (asset criticality vs data classification):** C16 RSI uses the Data Classification profile for *data* sensitivity, but OT *asset* criticality (safety-critical PLC) is a separate axis with no OCSF home. Decide whether asset-criticality is a `prism_ot` device attribute or reuses/abuses a finding risk field. Product + security decision.
- **SF-5 (#1515 dependency):** Accept the risk that the OCSF `ics.*` namespace (Issue #1515) may never merge or may merge with different field names, requiring a Prism rename later. Human risk-acceptance.

### Inconclusive / honesty flags

- The deep-research model could not pin the live version on its own (it hedged at "1.4.0 confirmed, 1.8.0 indirect"); the **v1.8.0 / 2026-03-18** figure here comes from a **direct live fetch** of the OCSF releases page + schema browser and supersedes that hedge.
- Whether the v1.8 `vulnerability`/package object exposes an explicit **CPE** field is **not fully confirmed** — the schema-browser excerpt did not surface one; verify against the raw `objects/vulnerability` JSON export before finalizing C11 design (SF-3).
- The exact attribute set the **Data Classification profile** contributes was confirmed-as-present but not enumerated field-by-field here — enumerate from the profile JSON before C16 commits (§7).
- OPC-UA is **not** named in Issue #1515 (Modbus/DNP3/BACnet/S7comm/EtherNet-IP are) — if Prism Reading B targets OPC-UA, that protocol's metadata is uncovered even by the proposed upstream work.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | Deep multi-source synthesis: (1) OCSF version history + categories/classes + data_classification profile; (2) OCSF OT/ICS coverage, device type_id enum, working-group activity. Both returned large outputs saved to tool-result files; the version-determination was inconclusive in-model and was superseded by direct live fetches. |
| Perplexity perplexity_ask | 3 | Targeted factual lookups: Device type_id/firmware/zone fields; Vulnerability Finding + Device Config State + Compliance Finding schema; OCSF extension-vs-profile mechanism + contribution process (domain-filtered to github.com / schema.ocsf.io / ocsf.io). |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_reason | 0 | — |
| Context7 | 0 | Not used — OCSF is a schema spec, not a code library; canonical source is schema.ocsf.io + GitHub. |
| Tavily (all) | 0 | — |
| WebFetch | 5 | Live verification of canonical sources: OCSF GitHub releases page (current version + dates); schema.ocsf.io home (current version, 14 profiles incl. Data Classification, OT absence); GitHub issue search (found ICS Issue #1515); Issue #1515 detail; Vulnerability Finding + Vulnerability Details object schema (v1.8.0 browser). |
| WebSearch | 0 | — |
| Training data | 1 area | General OCSF architecture framing (categories/profiles concept) — cross-checked against live sources; no version numbers or schema specifics taken from training data. |

**Total MCP tool calls:** 5 Perplexity (2 research + 3 ask). **Plus 5 WebFetch** for canonical live verification.
**Training data reliance:** low — every version number, class id, profile name, enum gap, and the ICS issue were verified against live `schema.ocsf.io` / `ocsf/ocsf-schema` sources on 2026-06-27. Two items explicitly flagged inconclusive (CPE field presence; Data Classification profile's exact attribute set).
