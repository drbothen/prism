---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C11-1: Architecture = FEED-DOWN, MATCH-AT-EDGE — central Prism Intel service ships CVE/CPE/KEV/EPSS/VEX corpus down; satellite/edge joins locally; central stays blind to asset inventory by construction"
  - "ADR-PROP-C11-2: Opt-in central-match mode (additive) — edge-match is default everywhere; consent-gated central-match carve-out for non-BYOC SaaS customers who explicitly waive zero-access"
  - "ADR-PROP-C11-3: Privacy mechanisms — full public corpus shipped down for CVE/CPE (no crypto needed); HMAC-keyed hashed indicators / Bloom filters for large IOC feeds only; PSI rejected as default"
  - "ADR-PROP-C11-4: Advisory generation at the edge — priority = f(KEV, EPSS, CVSS v4, asset_exposure, asset_criticality, compensating_controls); VEX demotes false positives; standards: CVSS v4.0 (2023-11), EPSS daily, CISA KEV BOD-22-01, NVD CVE API 2.0, CPE 2.3, CSAF 2.0/VEX (ISO/IEC 20153 2025-05), SSVC decision-support"
  - "ADR-PROP-C11-5: Air-gap = REUSE C9 signed-bundle — Ed25519/sigstore-signed delta tarball (CVE+CPE+EPSS+KEV+VEX deltas) over the same C9 offline-signed-bundle mechanism; no parallel trust path"
  - "ADR-PROP-C11-6: Metering = deployment-conditional zero-access-preserving — air-gap/on-prem = license-entitlement cap (no telemetry); SaaS/online = edge-reported aggregate count (never asset-level)"
  - "ADR-PROP-C11-7: Packaging = separate annual SKU — free baseline (NVD/CVE+CPE+KEV+EPSS+CSAF/VEX public feeds) + paid premium (commercial/curated feeds + analyst-validated per-CVE advisories + VEX suppression)"
  - "ADR-PROP-C11-8: Down-feed standards — custom signed-delta HTTPS API (online) + C9 signed bundle (air-gap) + CSAF/VEX-native ingestion; TAXII 2.1 optional for interop"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Side-analysis C11 capture; human-confirmed decisions 2026-06-27 session.
  Research basis: research/prism-intel-threat-advisory-2026-06-27.md (Q1–Q6, 8 MCP tool calls:
  4× perplexity_research sonar-deep-research reasoning_effort=high + 4× perplexity_ask;
  six-vendor landscape TI-VENDORS; privacy-preserving matching PRIV-*; risk-scoring standards
  STD-CVSS/EPSS/KEV/SSVC/CSAF/CPE; packaging/pricing PKG). Does NOT modify live ADR files,
  ARCH-INDEX.md, STATE.md, SESSION-HANDOFF.md, or any live factory artifact.
traces_to:
  - matured-vision-day2-requirements.md §16.4 (C11 decisions log entry)
  - day2-design-decisions/ADR-PROP-prism-context.md (C12 — Entity 360 substrate for advisory decoration: exposures + advisories + recommended-actions)
  - day2-design-decisions/ADR-PROP-config-management.md (C9 — signed-bundle mechanism reused for air-gap intel delivery)
  - day2-design-decisions/ADR-PROP-dual-deployment.md (DEPLOY — opt-in central-match carve-out is deployment-profile-conditional; metering deployment-conditional)
  - research/prism-intel-threat-advisory-2026-06-27.md (primary research basis)
  - CLAUDE.md (AD-017 zero-access / AI-opaque credentials — extends to asset-inventory by construction in D-C11-1)
---

# ADR-PROP — Prism Intel (C11)

> **STATUS: FULLY DECIDED 2026-06-27 (human) — all eight decision blocks confirmed.** This is a
> CAPTURE artifact for the side-analysis C11 program. `do_not_execute: true`. Real ADR numbers
> and formal ARCH-INDEX.md rows are deferred to the morph execution (post-demo, post-T14, gated
> on brief-reframe sign-off §5.1).

> **Research basis:** `research/prism-intel-threat-advisory-2026-06-27.md` — Q1 vendor landscape
> (Tenable/Qualys/Rapid7/Microsoft/Recorded Future/Mandiant/runZero/Nucleus), Q2 advisory
> prioritization (CVSS v4/EPSS/KEV/SSVC fusion), Q3 BYOC crux (feed-down vs PSI vs Bloom),
> Q4 feed sourcing/normalization/air-gap redistribution, Q5 packaging/pricing, Q6 Entity 360
> integration with C15 ARO.

> **BYOC zero-access differentiator:** every surveyed full-stack asset-aware platform (Tenable,
> Qualys, Rapid7, Microsoft, runZero, Nucleus) holds the asset inventory CENTRALLY. Only
> Recorded Future and Google Threat Intelligence avoid central inventory — by being a CVE-keyed
> intel layer that the customer's own platform joins locally. Prism delivers central-cloud
> asset-aware advisory prioritization while remaining blind to the asset inventory by construction
> — a gap no surveyed mainstream product fills. (Source: [TI-VENDORS] in research document.)

---

## Context

Prism Intel is a purchasable add-on service that ties threat intelligence and auto-generated
advisories to the customer's discovered assets in Entity 360 — e.g., "you have asset X running
software Y vulnerable to CVE-Z; here is the advisory and the recommended action."

The hard constraint is Prism's BYOC zero-access thesis: the central service must never see raw
customer asset inventory. This drives the fundamental architecture choice between
(a) feed-down + local match, (b) PSI-based private computation, or (c) opt-in central match.

The core advisory match is fundamentally CVE↔CPE: does any of my assets run a product (CPE)
that appears in a vulnerability's applicability statement? The entire CVE/CPE/KEV/EPSS/VEX
corpus is public, bounded (~hundreds of thousands of CVEs), and shippable as a signed bundle.
This makes feed-down + local match not only privacy-preserving but also architecturally clean.

Three Prism deployment models (SaaS, MSSP-managed, client-managed/air-gap) each have different
intel delivery requirements, handled by a single mechanism via C9 signed-bundle reuse.

---

## Decision Ledger

### D-C11-1 — Architecture: FEED-DOWN, MATCH-AT-EDGE

**DECIDED 2026-06-27 (human).**

Central "Prism Intel" service aggregates and normalizes public intel (CVE/CPE/KEV/EPSS/CSAF-VEX)
and ships the corpus DOWN to the satellite/edge. The satellite joins it to Entity 360 LOCALLY
with local exposure/criticality context. Central stays blind to the asset inventory BY
CONSTRUCTION — the match is fundamentally CVE↔CPE; the corpus is public, bounded, and signable.

**Rationale:** (1) Dominant, battle-tested industry pattern for privacy-preserving intel delivery
across Palo Alto, Cisco, Microsoft, CrowdStrike, Fortinet, and RFC 9424 (feed-down, local match).
(2) The entire CVE/CPE/KEV/EPSS/VEX corpus is public and bounded — no asset data needs to leave
the edge for the core advisory use case. (3) Same shape as the air-gap requirement (signed bundle
down, reuse C9), so online and air-gap cases share one mechanism. (4) The genuine BYOC
differentiator: no surveyed full-stack vendor stays blind to the inventory.

**BYOC invariant (PIV-C11-001):** central Prism Intel service NEVER receives raw asset
identifiers in the default edge-match path. The down-feed corpus is public and signed.

---

### D-C11-2 — Opt-In Central-Match Mode (Additive)

**DECIDED 2026-06-27 (human).**

Edge-match (D-C11-1) is the DEFAULT everywhere. An OPT-IN central-match mode is ALSO offered
as a consent-gated carve-out for non-BYOC SaaS customers who EXPLICITLY waive zero-access for
cross-tenant analytics and benchmarking.

This is the ONLY path where inventory leaves the edge. Governance requirements:
- Explicit consent record from the customer (deployment-profile flag + signed consent)
- Scope limit on what inventory fields transit central
- Central-match mode is inaccessible in the MSSP-managed and client-managed deployment models
- Cross-tenant analytics scope bounds are an OQ (see OQ-C11-3)

**PIV-C11-004:** The opt-in central-match mode is the ONLY inventory-leaves-edge path and
requires explicit recorded consent. No other code path transmits raw asset identifiers to central.

**ADS conformance note (P-ADS-02, P-ADS-03, P-ADS-04; 2026-06-27):** When D-C11-2 central-match
results are persisted at Central (applicable under Option 3 / Tenant-Keyed-Central-Persistence),
they MUST be encrypted under the tenant's DEK via SS-26 (P-ADS-04). The operator has zero at-rest
read access to the match results (P-ADS-02 Operator-Zero-Access-At-Rest). The client views their
advisory results through their authenticated Central session — "Central blind" means operator-blind,
not client-blind. OQ-C11-3 (consent governance model for scope limits, data-retention, and
deletion-on-consent-withdrawal) MUST be closed before SaaS launch of the central-match mode.

---

### D-C11-3 — Privacy Mechanisms

**DECIDED 2026-06-27 (human).**

**CVE/CPE core = ship the WHOLE public corpus down.** No cryptographic privacy mechanism is
needed for the primary advisory use case — the corpus is public, the matching happens locally,
and there is nothing to hide.

**HMAC-keyed hashed indicators / Bloom filters are RESERVED for:**
- Large IOC feeds (domains/hashes/IPs) where the feed is large or indicator-obfuscation is
  contractually required
- Any optional opt-in central aggregation where some indicator privacy is needed

**PSI (Private Set Intersection) EXPLICITLY REJECTED as default:**
- PSI provides mutual privacy (both parties' sets secret). Prism's requirement is one-sided
  (the intel feed is vendor-public-ish; only the asset inventory must stay secret) — PSI
  over-serves this.
- No surveyed mainstream SIEM/EDR/XDR product ships PSI in production for SOC asset/IOC matching
  as of 2026. It remains research/pilot (Source: [PRIV-PSI] in research document).
- Feed-down + local match is simpler, faster, and directly sufficient.

---

### D-C11-4 — Advisory Generation at the Edge

**DECIDED 2026-06-27 (human).**

Advisory priority is computed AT THE EDGE with local asset context:

```
priority = f( KEV-listed?              # binary: actively exploited → top
            , EPSS                     # probability of near-term exploitation
            , CVSS v4 base/threat      # inherent severity + exploit maturity
            , asset_exposure           # internet-facing / reachable / segmentation
            , asset_criticality        # business value, crown-jewel tags
            , compensating_controls    # already-mitigated → demote
            )
```

- KEV-listed + high EPSS = "act now" (strongest signals per CISA BOD-22-01 and FIRST EPSS)
- VEX "not affected" assertions demote CVEs that a naive CPE match would otherwise flag
- SSVC as decision-support framework (not a replacement for the above signals)

**Standards (date-stamped):**

| Standard | Role | Maintainer | Date |
|---|---|---|---|
| CVSS v4.0 | Vulnerability severity (Base/Threat/Environmental/Supplemental) | FIRST | 2023-11-01 |
| EPSS | ML model: probability of exploitation in next 30 days | FIRST | Daily refresh |
| CISA KEV | Known exploited CVEs; BOD-22-01 due dates | CISA | Continuous |
| NVD CVE API 2.0 | CVE records + CPE applicability configurations | NIST NVD | Current |
| CPE 2.3 | Structured product naming; the CVE↔asset join key | MITRE/NIST | 2019-11-06 |
| OASIS CSAF 2.0 / VEX | Machine-readable advisory format + VEX noise-suppression profile | OASIS | ISO/IEC 20153 2025-05-20 |
| SSVC | Decision-tree categorization (decision-support) | CISA/CMU SEI | Current |

The advisory decorates the Entity 360 view along the C15 ARO model:
- **Observation** = "asset X runs Y vV; CVE-Z applies (CPE match); KEV=true; EPSS=0.74"
- **Recommendation** = "patch to V' / apply compensating control; priority = critical (KEV +
  internet-facing + crown-jewel)"; computed at the edge with local context
- **Action** = gated via C15 SOAR ARO layer; NEVER auto-fired for irreversible operations

---

### D-C11-5 — Air-Gap: REUSE C9 Signed-Bundle

**DECIDED 2026-06-27 (human).**

Intel ships to air-gapped sites as an Ed25519/sigstore-signed delta tarball
(CVE+CPE+EPSS+KEV+VEX deltas) over the SAME C9 offline-signed-bundle mechanism — NOT a
parallel pipeline.

**Same:** signing key custody, integrity verification, delta discipline, transport path.
**Reuse:** the satellite verifies the bundle using the same trust anchor already established
for config bundles (C9 D-C9-BOOTSTRAP).

Precedent: Tenable Nessus offline signed plugin feed (manual challenge/response license
activation + signed plugin archive, integrity-verified before applying).

**PIV-C11-005:** Air-gap intel delivery MUST use the C9 signed-bundle mechanism. No parallel
trust path, no separate key custody, no alternative signing scheme.

---

### D-C11-6 — Metering: Deployment-Conditional Zero-Access-Preserving

**DECIDED 2026-06-27 (human).**

Metering is deployment-conditional to preserve the zero-access thesis:

| Deployment model | Metering mechanism |
|---|---|
| Air-gap / on-prem (client-managed) | License-ENTITLEMENT cap — nothing phoned home |
| MSSP-managed | License-entitlement cap per customer bundle |
| SaaS / online | Edge-reported AGGREGATE asset count — never asset-level telemetry |

Meter dimension = asset count discovered in Entity 360 (industry-standard VM dimension per
Tenable/Qualys/Nucleus pricing patterns).

**PIV-C11-006:** Metering NEVER emits asset-level telemetry to central in any deployment model.
Air-gap/on-prem uses entitlement caps only. SaaS uses aggregate counts only.

---

### D-C11-7 — Packaging: Separate Annual SKU

**DECIDED 2026-06-27 (human).**

Prism Intel = a SEPARATE annual subscription SKU layered onto a Prism deployment.

**Free baseline (public feeds — already actionable):**
- NVD CVE API 2.0 + CPE 2.3 + CISA KEV + FIRST EPSS + OASIS CSAF 2.0/VEX
- Delivers KEV/EPSS-prioritized, asset-keyed advisories out of the box

**Paid premium:**
- Commercial/curated feeds (specific partners = OQ-C11-1)
- Analyst-validated per-CVE advisories (asset-AGNOSTIC per-CVE — ships down clean; no asset
  data flows up for analyst authorship)
- VEX-driven false-positive suppression at higher coverage
- Faster cadence (e.g., same-day EPSS refreshes, commercial TI enrichment)

**Pricing dimension:** asset count discovered in Entity 360 (edge-reported aggregate or
entitlement-based per D-C11-6 above).

The free/premium split mirrors Microsoft Defender TI's free/premium tier structure and
Recorded Future's modular packaging pattern.

---

### D-C11-8 — Down-Feed Standards

**DECIDED 2026-06-27 (human).**

| Channel | Mechanism |
|---|---|
| Online (SaaS/MSSP) | Custom signed-delta HTTPS API — CVE+CPE+EPSS+KEV+VEX deltas, signed per D-C11-5 key model |
| Air-gap | C9 signed bundle (same mechanism) |
| Ingest compatibility | CSAF/VEX-native ingestion from vendor advisory feeds |
| Interop (optional) | TAXII 2.1 collection endpoint for customers with existing TAXII clients |

TAXII 2.1 is optional interop, not the primary distribution mechanism. The primary mechanism
is the custom signed-delta HTTPS API + C9 bundle.

---

## Provable Invariants (PIV-C11-*)

| ID | Invariant |
|----|-----------|
| **PIV-C11-001** | Central Prism Intel service NEVER receives raw asset identifiers in the **default (edge-match) path** (D-C11-1). The down-feed corpus is public + signed. **CONSENTED EXCEPTION (ADS conformance 2026-06-27, P-ADS-03):** The opt-in central-match mode (D-C11-2) is the ONLY permitted inventory-leaves-edge path; it requires explicit recorded consent (PIV-C11-004) and is inaccessible in MSSP-managed and client-managed deployment models. The invariant holds unconditionally for the default path; D-C11-2 is the named, consent-gated, sole carve-out. Any new code path that transmits asset identifiers to Central without satisfying PIV-C11-004 violates this invariant regardless of the data format. **"Central blind" nuance (P-ADS-02):** when D-C11-2 opt-in central-match results ARE stored at Central, they are encrypted under the tenant's DEK (P-ADS-04 Tenant-Keyed-Central-Persistence). The OPERATOR has zero at-rest read access; the authenticated CLIENT views their own advisory results through their Central session. |
| **PIV-C11-002** | The down-feed corpus is public (NVD/CVE/CPE/EPSS/KEV/VEX) and integrity-verified via Ed25519/sigstore signature before ingest. Unsigned bundles are rejected. |
| **PIV-C11-003** | Metering NEVER emits asset-level telemetry to central in any deployment model. |
| **PIV-C11-004** | The opt-in central-match mode is the ONLY inventory-leaves-edge path and requires explicit recorded consent. No other code path transmits raw asset identifiers to central. |
| **PIV-C11-005** | Air-gap intel delivery MUST use the C9 signed-bundle mechanism. No parallel trust path is created. |
| **PIV-C11-006** | Advisory priority is computed at the edge with local exposure/criticality context. The central service does not compute per-asset advisory priorities. |
| **PIV-C11-007** | Analyst-authored per-CVE advisories (paid tier) are ASSET-AGNOSTIC at time of authorship. No asset inventory data flows to the analyst authorship pipeline. |

---

## Open Items (OQ-C11-*)

| ID | Question |
|----|---------|
| **OQ-C11-1** | Commercial feed partners and OEM terms: Recorded Future (RF), Google Threat Intelligence (GTI), MISP community feeds. Which partners ship with v1 of the paid tier? What are the redistribution/OEM license terms? Morph-time business development decision. |
| **OQ-C11-2** | Analyst-advisory authorship pipeline: staffing model, tooling, per-CVE publication cadence, QA gates, asset-agnosticity enforcement (how do we confirm no asset data enters the authorship flow?). Operations/product decision at morph. |
| **OQ-C11-3** | Opt-in central-match consent/governance model: consent record format, scope limits on which inventory fields may transit, cross-tenant analytics scope, data-retention limit for central-match results, deletion-on-consent-withdrawal. Legal/privacy review required before SaaS launch. |
| **OQ-C11-4** | TAXII 2.1 interop demand: do target MSSP/SOC customers already have TAXII clients they would use? Determines whether TAXII 2.1 ships in v1 or v2 of Prism Intel. Customer discovery / sales engineering input. |
| **OQ-C11-5** | Metering enforcement posture per deployment: what is the enforcement mechanism for license-entitlement caps in air-gap/on-prem deployments (honor-system, cryptographic license token, periodic license check)? Business policy decision at morph. |

---

## Entity 360 + ARO Integration

Prism Intel advisories decorate the entity at `asset → exposures → advisories → recommended-actions`:

```
Entity 360 view
└── exposures (CVE-Z, CPE match, KEV=true, EPSS=0.74)
    └── advisories (priority=critical; patch V'→V''; compensating: firewall rule X)
        └── recommended-actions → C15 ARO Action gate
```

The `recommended-actions` leg maps directly onto the C15 ARO model:
- `Observation` node = factual enrichment on the entity timeline (computed at edge)
- `Recommendation` node = generated by the D-C11-4 priority function with local context
- `Action` = gated via C15 SOAR; never auto-fired for irreversible operations

Cross-links: C12 (Entity 360 substrate), C15 (ARO model), AD-017 (zero-access), C9 (signed-bundle).

C20 relevance: CVE/advisory ties map to NERC CIP-010 vulnerability assessment for OT assets.
Entity 360's Purdue-zone attributes (C12 D-C12-6) let advisory priority be conditioned on
operational zone — a CVE on a Purdue Level 0/1 OT device in an air-gap OT network has
different priority than the same CVE on a corporate laptop.

---

## EPIC: E-PRISM-INTEL-001

Proposed epic identifier: `E-PRISM-INTEL-001` (feeds B brief-reframe).

Sub-epics anticipated at morph:
- `E-INTEL-FEED-INGEST-001` — central aggregation + normalization pipeline (CVE/CPE/EPSS/KEV/VEX)
- `E-INTEL-EDGE-MATCH-001` — edge CPE-match engine + priority function (D-C11-4)
- `E-INTEL-BUNDLE-DELIVERY-001` — signed-delta API + C9 bundle delivery path (D-C11-5, D-C11-8)
- `E-INTEL-ENTITY360-DECORATE-001` — advisory decoration of Entity 360 exposures view (C12 integration)
- `E-INTEL-ARO-LINK-001` — ARO Observation/Recommendation generation from advisories (C15 integration)
- `E-INTEL-PREMIUM-FEEDS-001` — commercial feed ingestion (paid tier, gated on OQ-C11-1)

---

## Downstream SAP-1 Obligations (Morph-Time)

The following `event_type` values will be needed in BC-2.16.002 Canonical Structured Event Catalog
at morph. NOT actioned here.

- `event_type = "intel.feed.bundle.ingested"` — emitted when a signed bundle is verified and
  ingested at the satellite. Fields: bundle_id, feed_version, cve_count, cpe_count, kev_count,
  epss_date, signature_verified (bool), duration_ms; audit role = intel feed audit; recurrence = per bundle.
- `event_type = "intel.advisory.generated"` — emitted when an advisory is generated for an asset.
  Fields: asset_id (local-only, never transits central), cve_id, priority, kev_listed, epss_score,
  cvss_v4_base, compensating_control_applied (bool); audit role = advisory generation; recurrence = per advisory.
- `event_type = "intel.feed.bundle.rejected"` — emitted when a bundle fails signature verification.
  Fields: bundle_id, rejection_reason; audit role = integrity audit; recurrence = per rejection.

All three categories flagged here; BC-2.16.002 amendment is morph-time work.

---

## Honest Costs

| Item | Cost / Risk |
|------|-------------|
| **Central aggregation pipeline maintenance** | Central Prism Intel must track NVD/CVE API 2.0 updates (daily EPSS, continuous KEV, CSAF VEX from vendors), normalize to one internal model, build and sign delta bundles. This is ongoing operational work — not a one-time implementation. |
| **CPE match quality** | CPE applicability in NVD is known to have gaps, errors, and delayed updates. The advisory quality depends directly on CPE data quality. False negatives (asset IS affected but no CPE match) and false positives (CPE match but VEX says "not affected") both occur. VEX suppression reduces the false-positive rate; CPE gaps are a known industry limitation across all VM vendors. |
| **Edge CPE discovery** | The advisory engine is only as good as the CPE fingerprinting of assets in Entity 360. If sensor adapters don't report CPE-formatted software inventory, the match will miss affected assets. Prism must normalize sensor software-inventory fields to CPE 2.3 format. This is a sensor-adapter obligation. |
| **Opt-in central-match governance** | The OQ-C11-3 consent/governance model must be robust before SaaS launch. A misconfigured consent gate could silently transmit asset inventory to central — violating the zero-access thesis. The PIV-C11-004 invariant must be enforced by the implementation (not just the spec). Security-reviewer sign-off required before SaaS launch of central-match mode. |
| **Bundle size management** | The full CVE/CPE/EPSS/KEV/VEX corpus is hundreds of MBs at initial bootstrap but delta bundles are small. Initial bootstrap over a narrow air-gap link may require split transfer or pre-staged media. Bundle compression + delta discipline (D-C11-5) bounds ongoing transfer size. |
| **CSAF/VEX sourcing** | CSAF VEX advisories require per-vendor ingestion pipelines. Coverage is uneven — major vendors (Red Hat, Microsoft, Google, Cisco) publish CSAF VEX; many smaller vendors do not. The paid tier's VEX suppression coverage depends on the breadth of CSAF vendor participation, which is improving but not universal as of 2026. |

---

## Alternatives Considered and Rejected

### Alternative A: PSI (Private Set Intersection) as the Default Match Mechanism

Use PSI cryptography to allow central to compute CVE↔asset matches without seeing the asset set.

**Rejected (D-C11-3) because:** (1) PSI provides mutual privacy — neither party sees the other's
set. Prism's requirement is one-sided (the CVE/CPE corpus is vendor-public; only the asset
inventory must stay private). PSI over-serves a one-sided requirement. (2) No mainstream
SIEM/EDR/XDR product ships PSI for SOC asset/IOC matching in production as of 2026; it remains
research/pilot. (3) Feed-down + local match is architecturally simpler, faster, and directly
sufficient for the CVE↔CPE use case where the entire corpus is public and bounded.

### Alternative B: Central-Only Asset-Aware Advisory (Tenable/Qualys Model)

Hold the asset inventory centrally and compute advisories server-side, matching the pattern of
every full-stack VM vendor surveyed.

**Rejected (D-C11-1) because:** This is the BYOC anti-pattern. Prism's zero-access thesis
(AD-017, D-DEPLOY-005) is structurally incompatible with uploading asset inventory to a central
cloud service. The entire product's BYOC differentiator depends on not doing this.

### Alternative C: TAXII 2.1 as the Primary Distribution Mechanism

Use TAXII 2.1 as the canonical transport for delivering the intel corpus to satellites.

**Rejected (D-C11-8) because:** TAXII 2.1 is interop infrastructure (for customers who already
have TAXII clients). It is not designed as a high-throughput signed-bundle delivery mechanism
for air-gap environments. The custom signed-delta HTTPS API + C9 bundle path is more suitable
for Prism's three deployment models and reuses existing trust infrastructure.

---

## Ripple Effects (Morph-Time)

| Affected area | Ripple |
|---|---|
| **C9 (signed-bundle)** | D-C11-5 adds intel payload types (CVE/CPE/EPSS/KEV/VEX deltas) to the C9 bundle format. The C9 morph ADR must confirm the bundle format can carry intel payloads alongside config payloads, and that the bundle verification path is shared. |
| **C12 (Entity 360)** | Advisory decoration of the Entity 360 exposures view (the C12 morph stories must reserve the `exposures` sub-view for Prism Intel hooks). The C12 ENTITY 360 EXPANSION view already references exposures (part 4 = vulns/misconfig/unpatchable); Prism Intel is the implementation of that part. |
| **C15 (ARO)** | Advisory-generated Observations and Recommendations must be structurally compatible with C15's ARO model. The ARO Observation/Recommendation schema in C15 must accommodate CVE/advisory provenance fields. |
| **C20 (NERC CIP / OT)** | Advisory prioritization for Purdue Level 0/1/2 assets (OT) must weight asset_criticality appropriately for operational technology. OT CVE advisories tie NERC CIP-010 vulnerability assessment requirements. Priority function must accept Purdue zone as a criticality input. |
| **Sensor adapters (CPE fingerprinting)** | Every sensor adapter that reports software inventory must normalize software names + versions to CPE 2.3 format (or provide enough information for Prism to do the normalization). Without CPE-normalized software inventory in Entity 360, the CVE↔CPE match cannot fire. This is a blocking prerequisite for E-PRISM-INTEL-001. |
| **BC-2.16.002 §Postconditions** | Three SAP-1 event type categories listed in §Downstream SAP-1 Obligations above (morph-time BC work). |
| **matured-vision §16.4** | C11 decision block appended in-place (2026-06-27). |
