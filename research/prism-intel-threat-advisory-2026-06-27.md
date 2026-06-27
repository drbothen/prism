# Research: Prism Intel — Hosted Threat-Intelligence / Auto-Advisory Service Tied to Entity 360 (C11)

> **SIDE-ANALYSIS item:** C11 — Prism Intel (hosted threat-intelligence / auto-advisory service tied to Entity 360)
> **Mode:** CAPTURE / research only (`do_not_execute`). No live spec/BC/ADR/STATE.md/SESSION-HANDOFF.md modified. No git run.
> **Date:** 2026-06-27
> **Type:** general (technology + architecture research; feeds the C11 design decision)
> **Author:** research-agent
> **Status:** complete

---

## Scope & Context

Prism today is a **read-only federated security query engine**: PrismQL issues queries, sensor adapters fan out to vendor APIs (CrowdStrike, Cyberint, Claroty, Armis...), results are normalized to **OCSF/protobuf**, and an **Entity 360** view aggregates cross-source appearances of an asset/host/user/hash with a timeline, related entities, and enrichment (matured-vision-day2-requirements.md L1391: *"360° view of an IP/user/host/hash | aggregated cross-source appearances + timeline · related entities · enrichment (threat-intel sources)"*).

**Concept under evaluation — "Prism Intel":** a (likely hosted, purchasable) add-on service that ties **threat intelligence + auto-generated advisories** to the customer's **discovered assets** in Entity 360 — e.g., *"you have asset X running software Y vulnerable to CVE-Z; here is the advisory and the recommended action."*

**The hard constraint (the BYOC privacy crux):** Prism's three deployment models include **air-gap** (the purest expression per C9/D-DEPLOY anchors), AD-017 **AI-opaque credentials**, and a **BYOC zero-access thesis** (the central/vendor service never sees raw customer data). So: **how do you deliver asset-keyed intel WITHOUT the central service ever seeing the customer's raw asset inventory?**

Relevant project anchors:
- **Entity 360** = entity/observable profile, already enrichment-aware (matured-vision-day2-requirements.md L1391, L2117).
- **C15** = Actions · Recommendation · Observation (ARO) model + on-prem/air-gap model decisions (research/prismql-actions-soar-onprem-models-2026-06-27.md). Prism Intel advisories should decorate Entity 360 and feed the **Recommendation/Observation** legs of ARO.
- **C9** = config management + offline-signed-bundle / air-gap posture; the intel-feed redistribution path to air-gapped sites must reuse C9's signed-bundle mechanism.
- **AD-017** = credentials never transit AI context; by extension the BYOC thesis says raw asset inventory should not transit the central plane.

All non-obvious claims are cited. Standards/vendor facts are date-stamped. Where the deep-research model flagged uncertainty I cross-checked with a second query and noted conflicts inline.

---

## Q1 — Threat-Intel-as-a-Service: how commercial platforms deliver asset-aware advisories

### The canonical four-entity data model

Across **every** platform surveyed, asset-aware advisories rest on a four-entity model linking **asset inventory ↔ vulnerability/CVE ↔ advisory ↔ prioritization** [TI-VENDORS]. Formally: assets `A`, vulnerabilities `V`, threat-intel signals `T`; the platform materializes **exposures** as tuples `(a, v, t)` and applies a **prioritization function** `P(a, v, t, c)` where `c` encodes asset criticality + context. The variation between vendors is (1) *where the asset inventory lives* (cloud vs edge) and (2) *how proprietary the scoring is* [TI-VENDORS].

| Vendor (as of 2026) | Asset inventory location | How intel maps to assets | Advisory/priority mechanism |
|---|---|---|---|
| **Tenable** (Vuln Mgmt / Tenable One) | **Central cloud** ("cloud-based VM platform... see and track all of your assets with unmatched accuracy"); scanners/agents collect locally | Per-asset `(a,v)` join; each `v` carries a **VPR** score (0.1–10, updated **daily**) | **VPR** (7 drivers: age, CVSSv3 impact, exploit-code maturity, product coverage, threat sources, threat intensity, threat recency) → **Asset Exposure Score (AES, 0–1000)** weighting VPR × **Asset Criticality Rating (ACR)** → org-wide **Cyber Exposure Score (CES)** [TI-VENDORS] |
| **Qualys VMDR** | **Central cloud** ("global, always-updated IT asset inventory"); single agent + sensors collect locally | `(a,v)` join keyed by CVE; each `v` decorated with **Real-Time Threat Indicators (RTIs)** | **VMDR Prioritization** correlates RTIs + asset context (age, RTI, attack surface) → closed-loop to Patch Mgmt / EDR [TI-VENDORS] |
| **Rapid7 InsightVM / Exposure Command** | **Central cloud** console; agents/scanners collect | InsightVM supplies `(a,v)`; attack-surface mgmt adds internet-facing/exposure context | Risk score combining severity + exposure context + (historically) "Real Risk Score"/Active Risk; *exact scoring undocumented in public sources* [TI-VENDORS] |
| **Microsoft Defender TI / Security Exposure Mgmt** | **Central cloud** (unified posture across Defender products) | **Threat analytics** maps each threat/campaign/CVE to the customer's discovered assets ("learn if you're currently under attack... assess the impact of the threat to your assets") | Advisory objects = threat narrative + affected products + mitigations, generated when a threat-analytic's conditions intersect customer assets [TI-VENDORS] |
| **Recorded Future** (Vulnerability Intelligence) | **No asset inventory** — CVE-centric intel layer only | Intel keyed by **CVE**; asset-awareness emerges via **integration** with the customer's own scanner/asset DB | Exploitation-risk scores + **weaponization-lifecycle** timeline per CVE; the *customer's* platform joins to assets [TI-VENDORS] |
| **Mandiant Advantage / Google Threat Intelligence** | **No asset inventory** of its own; intel keyed by CVE/actor, embedded into Google SecOps or OEM'd | Intel enriches `(a,v)` records held by the consuming platform (e.g. Nucleus embeds Mandiant intel) | Actor + exploitability context per CVE; consumed via API/enrichment connectors [TI-VENDORS] |
| **runZero** | **Central cloud**, but **asset-centric** by design (deep fingerprinting + correlation; active scan + passive sampling + integrations) | Imports vuln data from 3rd-party VM/EDR tools; query-based detection logic over a unified inventory | Asset-centric exposure mgmt hub; prioritization over the unified inventory [TI-VENDORS] |
| **Nucleus Security** | **Central cloud** aggregation of vuln + asset data from many scanners | **Nucleus Insights** uses AI to scan exploit repos, dark-web forums, malware reports, vendor advisories, OSS feeds → analyst-validated per-CVE intel; embeds Mandiant intel | "Ready-to-automate intelligence for every CVE" tied to vuln instances; closed-loop remediation [TI-VENDORS] |

### The defining observation for Prism

**Every full-stack asset-aware platform surveyed (Tenable, Qualys, Rapid7, MS, runZero, Nucleus) holds the asset inventory CENTRALLY in a multi-tenant cloud.** The two products that *don't* (Recorded Future, Mandiant/GTI) achieve asset-awareness only by being a **CVE-keyed intel layer that the customer's own platform joins to local assets** [TI-VENDORS]. **No surveyed mainstream product delivers central-cloud asset-aware prioritization while remaining blind to the asset inventory.** That gap is precisely Prism's BYOC differentiator — and it forces the architecture answered in Q3.

---

## Q2 — Auto-advisory generation & prioritization (CVSS + EPSS + KEV + exposure)

### The four canonical risk signals (all standards-backed, free to consume)

| Signal | What it is | Maintainer | Cadence / mechanism | Source |
|---|---|---|---|---|
| **CVSS v4.0** | Vulnerability severity scoring (Base / Threat / Environmental / Supplemental metrics) | **FIRST** | Released **2023-11-01** | [STD-CVSS] |
| **EPSS** | ML model: probability a CVE is **exploited in the wild within 30 days** (0–1) | **FIRST** | **Refreshed daily** | [STD-EPSS] |
| **CISA KEV** | Catalog of CVEs with **known exploitation in the wild**; ransomware column; BOD-22-01 per-entry **remediation due dates** for federal agencies | **CISA** | Updated continuously; JSON feed | [STD-KEV] |
| **SSVC** | Decision-tree categorization (exploitation status × technical impact × mission/business importance → Act/Track/Attend) | **CISA / CMU SEI** | Decision-support framework, combined with CVSS/EPSS/KEV | [STD-SSVC] |

### How leading platforms combine them, and what makes an advisory *actionable*

Commercial platforms fuse these public signals with **asset context** into a single proprietary priority: Tenable **VPR** (CVSSv3 impact + exploit-code maturity + threat recency/intensity/sources, daily-updated), Qualys **TruRisk/RTIs**, Rapid7 **Active Risk** [TI-VENDORS]. The decisive multiplier is **asset context that only the customer's environment provides**: *is the vulnerable asset internet-facing? business-critical? reachable? does a compensating control exist?* Tenable encodes this as `AES(a) = g({R(a,v) | v ∈ V_a}, ACR(a))` — the same CVE contributes more to a domain controller's exposure than to an isolated test box [TI-VENDORS].

**Actionable vs noise:** an advisory becomes actionable when it answers *"which of MY assets, how exploitable right now, how exposed, what do I do."* The industry consensus prioritization recipe (highest → lowest urgency) is roughly:

```
priority = f( KEV-listed?           # binary: actively exploited in the wild -> top
            , EPSS                  # probability of near-term exploitation
            , CVSS v4 base/threat   # inherent severity + exploit maturity
            , asset_exposure        # internet-facing / reachable / segmentation
            , asset_criticality     # business value, crown-jewel tags
            , compensating_controls # already-mitigated -> demote
            )
```

KEV-membership and high EPSS are the strongest "act now" signals; raw CVSS alone is widely regarded as insufficient for prioritization [STD-KEV][STD-EPSS][TI-VENDORS]. **VEX** (Q4) is the noise-suppressor: a vendor "not affected" assertion lets the advisory engine *demote* a CVE that a naive CPE match would otherwise flag [STD-CSAF].

---

## Q3 — Asset-keyed intel WITHOUT central seeing raw assets (THE BYOC CRUX)

This is the load-bearing question. The research converges on a clear answer.

### Finding 1 — The dominant industry pattern is "push the feed DOWN, match locally"

Across Palo Alto, Cisco, Microsoft, Imperva, CrowdStrike, Fortinet and RFC 9424, the consistent operational model is: the vendor **maintains and distributes the intel/IOC feed**; the customer **pulls the feed into local tools (SIEM/EDR/IDS/firewall)**; and **matching happens at the edge** where telemetry is collected — the vendor never receives asset identifiers [PRIV-EDGE]. This is **exactly** the BYOC posture: feed-down, never data-up.

Why this dominates over the alternative (ship asset data up for central matching):
1. **Scalability/performance** — edge match is constant-time membership testing (hash table / Bloom filter, `O(k)`); shipping multi-million-element asset sets up per query is prohibitive [PRIV-EDGE][PRIV-BLOOM].
2. **Data-minimization / regulatory** — asset inventories + identities raise GDPR/HIPAA/contractual exposure; keep them at the edge [PRIV-EDGE].
3. **Integration simplicity** — publish feed → customer replicates it locally. No per-customer cryptographic key management.
4. **Threat-model fit** — the actual privacy requirement is usually "don't leak my asset inventory to the vendor," which feed-down + local match satisfies *without* heavyweight cryptography [PRIV-PSI].

### Finding 2 — Private Set Intersection (PSI) is real but NOT the right default here

PSI is a cryptographic MPC primitive that lets two parties compute `S_A ∩ S_B` and learn **nothing else** about each other's sets [PRIV-PSI][PRIV-NIST-PSI]. Built from OT/OPRF, garbled circuits, Bloom-filter encodings, or homomorphic encryption.

**Scaling reality as of 2026 (cross-checked):**
- Garbled-circuit PSI handles **million-element** sets on desktop hardware [PRIV-PSI-KATZ].
- **Server-aided PSI** reaches **billion-element** sets but needs heavy parallelization + an untrusted powerful server [PRIV-PSI-SAPSI].
- **Unbalanced PSI** (small client set vs large server set): PEPSI matches 1,024 client items against 1M server items in **<1s, <5MB** non-interactively — but naive HE approaches took **>600 minutes** in the same setting [PRIV-PSI-PEPSI].

**Commercial deployment reality:** PSI is in production *outside* SOC use cases (Apple Password Monitoring / compromised-credential checks) [PRIV-PSI-APPLE], and NIST tracks it as a privacy-enhancing primitive [PRIV-NIST-PSI]. But for **SOC asset/IOC matching specifically, no surveyed mainstream SIEM/EDR/XDR product advertises PSI in production — it remains research/pilot** [PRIV-PSI][PRIV-EDGE]. PSI's strength is **mutual** privacy (both parties' sets secret); Prism's requirement is **one-sided** (the intel feed is *vendor-public-ish*; only the asset inventory must stay secret), which is over-served by full PSI.

### Finding 3 — Hashed-indicator & Bloom-filter local matching keep central blind, cheaply

Two complementary mechanisms let the edge match while central stays blind [PRIV-BLOOM][PRIV-PPRL][PRIV-EDGE]:

- **Hashed indicators (feed-down):** central publishes intel as **keyed hashes (HMAC with a Prism-tenant secret)** of indicators (domains, file hashes, and — critically for Prism — **CPE/product fingerprints**). The edge hashes its local observables/assets with the same key and does set-membership lookup. Central never receives asset identifiers. Keyed (HMAC) hashing defeats offline dictionary attacks that plague unkeyed low-entropy indicators [PRIV-PSI §4][PRIV-PPRL].
- **Bloom filters:** central encodes the intel set into a Bloom filter `BF_I` and ships only `BF_I`; the edge tests `h_i(x)` membership in `O(k)` constant time; space-efficient for millions of indicators. The filter reveals approximate membership of the *intel* set (acceptable — it is the vendor's data) but the customer's assets never leave the edge [PRIV-BLOOM]. Privacy-preserving record-linkage research (Churches & Christen; PoPETs 2023) and **differentially-private Bloom filters** (DLDP-BF 2023, DP-BF 2025) extend this with formal guarantees if the *reverse* direction (customer→central) is ever needed [PRIV-PPRL][PRIV-DPBF].

### Finding 4 — For Prism specifically, the match is mostly CVE↔CPE, which makes feed-down trivial

Prism's advisory match is fundamentally **"does any of my assets run a product (CPE) that appears in a vulnerability's applicability statement?"** CVE applicability is expressed as **CPE 2.3 match strings in the NVD/CVE `configurations` array** [STD-CPE]. The *entire* CVE↔CPE↔KEV↔EPSS corpus is **public, bounded (~hundreds of thousands of CVEs), and shippable as a signed bundle**. So Prism can push the *whole intel corpus* (CVE + CPE applicability + EPSS scores + KEV flags) DOWN to the satellite/edge, and the edge joins it against the locally-discovered Entity 360 inventory. **No PSI, no Bloom filter, no asset data leaves the edge.** Bloom/HMAC techniques become relevant only for *IOC* matching (domains/hashes/IPs) where feeds may be larger or the operator wants to obscure the indicator set, or for any *optional* opt-in central aggregation.

**This is the architecturally clean answer and it is the LEAN (see §Analysis).**

---

## Q4 — Threat-intel feed sourcing, normalization, redistribution (incl. air-gap bundles)

### Sourcing & standards (all date-stamped)

| Standard | Role | Maintainer | Key facts | Source |
|---|---|---|---|---|
| **STIX 2.1** | CTI language + JSON serialization (indicators, actors, malware, campaigns) | **OASIS CTI TC** | OASIS Standard **2021-06-10** | [STD-STIXTAXII] |
| **TAXII 2.1** | RESTful HTTPS protocol to transport CTI (collections, polling) | **OASIS CTI TC** | OASIS Standard **2021-06-10** | [STD-STIXTAXII] |
| **MISP** | Open-source CTI sharing/correlation hub; feeds, sync, PyMISP; STIX/TAXII bridges | MISP Project / CIRCL | De-facto OSS sharing platform | [STD-MISP] |
| **NVD CVE API 2.0** | RESTful programmatic CVE access (descriptions, metrics, **CPE configurations**) | **NIST NVD** | API 2.0 schema | [STD-NVDAPI] |
| **CVE Record Format 5.x** | JSON schema for CVE entries; optional `configurations` array mirroring NVD CPE applicability | **CVE Program** | Supports CPE applicability statements (CVE CNA guide 2023-06-05) | [STD-CVEREC] |
| **CPE 2.3** | Structured product naming (URI/formatted-string); the join key for CVE↔product | **MITRE / NIST** | Spec 2019-11-06; NVD `configurations` array encodes per-CVE applicability | [STD-CPE] |
| **CSAF 2.0** | Machine-readable security advisory format (JSON) | **OASIS** | OASIS Standard **2022-11**; approved **ISO/IEC 20153** 2025-05-20 | [STD-CSAF] |
| **VEX** | CSAF 2.0 profile asserting affected / not-affected / fixed / under-investigation per CVE×product | **OASIS (CSAF profile)** | The noise-suppressor for advisory generation | [STD-CSAF] |
| **CISA KEV / AIS** | KEV JSON feed; AIS Automated Indicator Sharing | **CISA** | KEV due-date mechanism (BOD 22-01) | [STD-KEV] |

### The hosted-service pipeline pattern

A hosted intel service **aggregates** many feeds (NVD/CVE, KEV, EPSS, CSAF/VEX vendor advisories, STIX/TAXII collections, MISP, commercial), **normalizes** them to one internal model (CVE-keyed records + CPE applicability + scores + VEX status), and **redistributes** to consumers via API/feed or — for **air-gapped customers** — via **signed offline bundles** [STD-STIXTAXII][STD-MISP][STD-CSAF].

### Air-gap redistribution = reuse the C9 signed-bundle mechanism

The air-gap precedent is well established: **Tenable Nessus ships a signed, offline plugin feed** (manual challenge/response license activation + a **digitally-signed plugin archive** the offline scanner verifies for integrity/authenticity before applying) [STD-NESSUS-OFFLINE]; antivirus signature bundles work the same way. For Prism, the intel bundle is conceptually identical: a periodically-built, **Ed25519/Sig-store-signed tarball** (CVE + CPE + EPSS + KEV + VEX deltas) carried across the air gap on removable media and verified by the satellite before ingest. **This MUST reuse C9's offline-signed-bundle path** (the air-gap config/migration bundle mechanism) rather than inventing a parallel one — same signing key custody, same integrity verification, same delta-update discipline. Delta updates (only changed CVE records since last bundle) keep bundle size manageable.

---

## Q5 — Commercial / packaging model

### How the market sells intel add-ons (date-stamped 2026 benchmarks)

The universal pattern is **annual SaaS subscription, packaged in tiers/modules, priced by a mix of platform-tier + usage (API/feeds) + capacity (assets/identities)** — *not* per-user [PKG]. The platform license stays separate; intel is a **distinct subscription SKU/module that "wires into" the base platform** via API/enrichment connectors [PKG].

| Vendor | Packaging | Pricing dimension (2026) | Source |
|---|---|---|---|
| **Recorded Future** | Intelligence Cloud: **Core/Professional/Elite** packages + **add-on modules** (Vulnerability Intel, Brand, Third-Party, Geopolitical...) | Annual ACV, custom-quoted; ~$50k–100k (1–2 modules), $100k–250k (mid), $250k–500k+ (enterprise); median ~$70k. AWS Marketplace: Vulnerability Intel ~$56k/yr (≤4 users) | [PKG] |
| **Google Threat Intelligence** (ex-Mandiant) | **Standard / Enterprise / Enterprise+** + API-call packs + IOC-feed add-ons + private-scanning packs | **Flat annual rate per tier + API-call packs**; tiers gate queries/month + API requests/day (5k → 30k → 3M/day). No public $ list price | [PKG] |
| **Tenable** | Base VM platform + modules | **Per-asset / per-host**, tiered by asset count, annual | [PKG] |
| **Microsoft Defender TI** | Free tier (basic enrichment in Defender XDR/Sentinel) + **premium MDTI add-on SKU** | Premium = separate per-user/per-tenant Defender add-on | [PKG] |
| **Nucleus / runZero** | SaaS platform priced by **assets/vuln-items managed**; intel feeds are separate subscriptions integrated via connectors | Per-asset / environment tier | [PKG] |

### The analog for "buy Prism Intel as an add-on that wires into your Prism"

The clean commercial fit: **Prism Intel = a separate annual subscription SKU that layers onto a Prism deployment**, priced by **asset count discovered in Entity 360** (the most natural meter and the industry-standard VM dimension) and/or a **feed-tier** (free baseline = public-only feeds: NVD/CVE + KEV + EPSS + CPE; paid tier = curated/commercial feeds + analyst-validated advisories + VEX-enriched suppression). Because Prism is BYOC/air-gap, the **per-asset meter must be computed at the edge and reported as an aggregate count** (or via license-entitlement, not telemetry) so metering itself doesn't violate the zero-access thesis — a genuine sub-fork (see §Analysis).

---

## Q6 — Entity 360 integration (ties to C15 ARO)

Entity 360 already aggregates cross-source appearances + enrichment (matured-vision L1391, L2117). Prism Intel decorates the entity view along the path **asset → exposures → advisories → recommended actions**, which maps directly onto the **C15 Action · Recommendation · Observation (ARO)** model (research/prismql-actions-soar-onprem-models-2026-06-27.md):

- **Observation** = "asset X runs software Y at version V; CVE-Z applies (CPE match); KEV=true; EPSS=0.74." A factual, sourced enrichment on the entity timeline.
- **Recommendation** = "patch to V', or apply compensating control; priority = critical (KEV + internet-facing + crown-jewel)." Generated by the prioritization function of Q2, computed **at the edge** with local asset context.
- **Action** = (gated, per C15) "open ticket / trigger patch workflow / isolate" — flows through C15's separate orchestration layer with HITL approval gates, never auto-fired for irreversible operations.

This mirrors how Microsoft Threat Analytics and Nucleus Insights surface advisories as contextualized findings on assets [TI-VENDORS]. The Prism-specific win: because the join happens at the edge, the **advisory and its asset context are computed where the data already is** — the central plane only ever shipped the public intel corpus down.

---

## ANALYSIS + LEANS

### Recommended Prism Intel architecture (privacy-preserving, BYOC + air-gap native)

**LEAN: "Feed-down, match-at-edge" — central ships the public intel corpus DOWN; the satellite/edge joins it to Entity 360 locally. Central stays blind to the asset inventory by construction.**

Rationale: (1) It is the **dominant, battle-tested industry pattern** for privacy-preserving intel delivery [PRIV-EDGE]. (2) Prism's match is fundamentally **CVE↔CPE**, and the entire CVE/CPE/KEV/EPSS/VEX corpus is **public, bounded, and signable as a bundle** [STD-CPE][STD-NVDAPI] — so there is no need to ship asset data up *at all* for the core advisory use case. (3) It is the **same shape as the air-gap requirement** (signed bundle down, reuse C9), so the air-gap and online cases share one mechanism. (4) It is the **genuine differentiator** vs Tenable/Qualys/Nucleus, all of which hold the inventory centrally [TI-VENDORS].

Concretely:
- **Central "Prism Intel" service:** aggregates NVD/CVE + KEV + EPSS + CSAF/VEX + (paid) commercial/MISP/STIX-TAXII feeds → normalizes to a CVE-keyed model (CPE applicability + scores + VEX status) → publishes (a) an **online feed** (TAXII-2.1-style or simple signed HTTPS delta API) and (b) **C9-style signed offline bundles** for air-gap.
- **Satellite/edge:** ingests the corpus, joins CPE applicability against Entity 360's discovered software/asset CPEs **locally**, computes the Q2 priority with **local** exposure/criticality context, emits ARO Observations + Recommendations onto the entity view.
- **Crypto only where needed:** for *IOC* matching (domains/hashes/IPs) or any optional central aggregation, use **HMAC-keyed hashed indicators or Bloom filters** so central stays blind [PRIV-BLOOM][PRIV-PSI]. **PSI is explicitly NOT recommended** for the default path — it over-serves a one-sided privacy requirement and has no mainstream SOC production precedent [PRIV-PSI].

### Feed-sourcing + advisory-generation approach

- **Free/baseline tier:** public feeds only — **NVD CVE API 2.0 + CPE 2.3 + CISA KEV + FIRST EPSS + CSAF/VEX vendor advisories** [STD-NVDAPI][STD-CPE][STD-KEV][STD-EPSS][STD-CSAF]. This alone delivers genuinely actionable, KEV/EPSS-prioritized, asset-keyed advisories.
- **Paid tier:** curated/commercial feeds, analyst-validated per-CVE intel (the Nucleus-Insights / Recorded-Future model), VEX-driven false-positive suppression, faster cadence.
- **Advisory generation:** `priority = f(KEV, EPSS, CVSS v4, asset_exposure, asset_criticality, compensating_controls)`; KEV + high EPSS = "act now"; VEX "not affected" demotes; all computed at the edge.

### Packaging model

- **Separate annual SKU** layered onto a Prism deployment (the universal add-on pattern [PKG]).
- **Meter = asset count discovered in Entity 360** (industry-standard VM dimension), computed at the edge and reported as an **aggregate count or via license entitlement** — never as asset-level telemetry (preserves zero-access).
- **Free baseline (public feeds) + paid premium (curated feeds + analyst advisories + VEX)**, matching Defender-TI's free/premium split [PKG].

### Genuine sub-forks needing a HUMAN decision

| # | Fork | Options | Lean / note |
|---|---|---|---|
| **OQ-C11-1** | Match locality | (a) **Edge-match** (LEAN); (b) central-match (rejected — breaks BYOC); (c) hybrid (central match for opt-in customers who waive zero-access) | Strongly lean (a). Human decides whether to *offer* an opt-in (c) for non-BYOC SaaS customers who want central cross-tenant analytics. |
| **OQ-C11-2** | IOC-match privacy mechanism | (a) ship whole corpus down (simplest, fine for CVE/CPE); (b) **HMAC-keyed hashed indicators**; (c) **Bloom filters**; (d) PSI (rejected as default) | Lean (a) for CVE/CPE; (b)/(c) only if IOC feeds get large or indicator-obfuscation is contractually required. |
| **OQ-C11-3** | Feed tiers (free vs paid) | (a) free = public feeds only; (b) paid = + commercial/curated/analyst/VEX | Lean both — (a) baseline, (b) premium. Human decides *which* commercial feeds (Recorded Future / GTI / MISP communities) and OEM terms. |
| **OQ-C11-4** | Air-gap bundle mechanism | (a) **reuse C9 signed-bundle path** (LEAN); (b) parallel intel-specific bundle pipeline | Strongly lean (a) — same signing-key custody + integrity verification + delta discipline as C9. Human confirms C9 bundle can carry intel payloads. |
| **OQ-C11-5** | Metering without telemetry | (a) edge-reported aggregate count; (b) license-entitlement cap (no count phoned home); (c) honor-system | Lean (b) for purest air-gap; (a) for online. Human decides commercial enforcement posture vs zero-access purity. |
| **OQ-C11-6** | Advisory authorship | (a) auto-generated from public signals only; (b) + analyst-validated (paid, central-authored, asset-agnostic so it ships down clean) | Lean both tiers; note analyst advisories are *asset-agnostic per-CVE* so they ship down without any asset data flowing up. |
| **OQ-C11-7** | Standards alignment for the down-feed | (a) custom signed-delta HTTPS API; (b) **TAXII 2.1** collection; (c) CSAF/VEX-native | Lean (a) for the bundle + (c) ingestion-compatibility; (b) optional for interop with customers' existing TAXII clients. Human/architect call. |

### Confidence & limitations

- **High confidence:** the four-entity data model, the central-inventory pattern across full-stack vendors, the standards facts (CVSS v4 date, EPSS daily, KEV/BOD-22-01, STIX/TAXII 2021, CSAF/ISO, CPE 2.3), the feed-down-match-locally dominance, and PSI's scaling/deployment reality — all multi-source corroborated.
- **Medium confidence (vendor-internal):** exact proprietary scoring formulas (VPR/TruRisk/Active Risk) are **undocumented**; described qualitatively only [TI-VENDORS]. Rapid7's current scoring and Tenable/MS/Nucleus/runZero exact list prices were **not publicly confirmable** — packaging *patterns* are well-sourced, specific dollar figures for several vendors reflect 2024–2025 industry practice and should be re-verified at quote time [PKG].
- **Inconclusive:** whether any 2026 SOC product ships PSI in production — public docs show none; flagged as research/pilot [PRIV-PSI].

---

## Sources

**Threat-intel-as-a-service vendor landscape (Q1, Q2, Q6)**
- [TI-VENDORS] Perplexity Deep Research synthesis (sonar-deep-research, 2026-06-27) over Tenable (first.org/tenable.com VPR/AES/CES docs), Qualys (blog.qualys.com VMDR/RTI), Rapid7 (Exposure Command), Microsoft (Defender Threat Analytics / Security Exposure Management docs), Recorded Future (recordedfuture.com Vulnerability Intelligence), Mandiant/Google TI, runZero, Nucleus Security (Nucleus Insights). Vendor facts as of 2026.

**Risk-scoring standards (Q2)**
- [STD-CVSS] FIRST CVSS v4.0 — https://www.first.org/cvss/v4.0/ ; release 2023-11-01 (https://nvd.nist.gov/general/news/cvss-v4-0-official-support)
- [STD-EPSS] FIRST EPSS — https://www.first.org/epss/ (probability of exploitation in next 30 days; refreshed daily)
- [STD-KEV] CISA KEV catalog — https://www.cisa.gov/known-exploited-vulnerabilities-catalog (BOD 22-01 due dates)
- [STD-SSVC] CISA/CMU-SEI SSVC — https://www.cisa.gov/sites/default/files/publications/SSVC-v2-public.pdf

**Feed sourcing & redistribution standards (Q4)**
- [STD-STIXTAXII] OASIS STIX 2.1 / TAXII 2.1 — https://www.oasis-open.org/2021/06/23/stix-v2-1-and-taxii-v2-1-oasis-standards-are-published/ (OASIS Standards 2021-06-10)
- [STD-MISP] MISP Project — https://www.misp-project.org/
- [STD-NVDAPI] NVD CVE API 2.0 — https://nvd.nist.gov/developers/vulnerabilities
- [STD-CVEREC] CVE Program, CPE-in-CVE-Records guide (2023-06-05) — https://www.cve.org/Resources/Roles/Cnas/CPEinCVERecordsGuide.pdf
- [STD-CPE] CPE 2.3 spec (MITRE 2019-11-06) — https://cpe.mitre.org/specification/ ; NVD products/applicability — https://nvd.nist.gov/products
- [STD-CSAF] OASIS CSAF 2.0 (OASIS Std 2022-11; ISO/IEC 20153 2025-05-20) — https://docs.oasis-open.org/csaf/csaf/v2.0/os/csaf-v2.0-os.html ; ISO announcement https://www.oasis-open.org/2025/05/20/csaf-v2-approved-as-iso-iec-international-standard/ ; VEX profile FAQ https://github.com/oasis-tcs/csaf/blob/master/csaf_2.0/guidance/faq.md
- [STD-NESSUS-OFFLINE] Tenable Nessus offline/air-gapped signed plugin feed (manual challenge/license + signed plugin archive) — https://docs.tenable.com/nessus (workflow per vendor docs; confirm against current Tenable docs)

**Privacy-preserving matching (Q3)**
- [PRIV-EDGE] IOC feed-down / local-match pattern — Palo Alto https://www.paloaltonetworks.com/cyberpedia/indicators-of-compromise-iocs ; Cisco https://www.cisco.com/site/us/en/learn/topics/security/what-are-indicators-of-compromise-ioc.html ; Microsoft https://www.microsoft.com/en-us/security/business/security-101/what-are-indicators-of-compromise-ioc ; CrowdStrike https://www.crowdstrike.com/en-us/cybersecurity-101/threat-intelligence/indicators-of-compromise-ioc/ ; RFC 9424 https://datatracker.ietf.org/doc/rfc9424/
- [PRIV-PSI] Perplexity synthesis on PSI commercial/research status (2026-06-27)
- [PRIV-NIST-PSI] NIST Privacy-Enhancing Cryptography: PSI — https://csrc.nist.gov/Projects/pec/psi
- [PRIV-PSI-KATZ] Katz et al., garbled-circuit PSI (million-element) — https://www.cs.umd.edu/~jkatz/papers/psi.pdf
- [PRIV-PSI-SAPSI] Pinkas et al., "Scaling PSI to Billion-Element Sets" — https://cs.brown.edu/people/seny/pubs/sapsi.pdf
- [PRIV-PSI-PEPSI] Mahdavi et al., PEPSI unbalanced PSI (USENIX Security 2024) — https://www.usenix.org/system/files/usenixsecurity24-mahdavi.pdf
- [PRIV-PSI-APPLE] Apple PSI (Password Monitoring) — https://www.appsflyer.com/glossary/private-set-intersection/
- [PRIV-BLOOM] Bloom filter (constant-time membership) — https://en.wikipedia.org/wiki/Bloom_filter
- [PRIV-PPRL] Churches & Christen, Bloom-filter privacy-preserving record linkage (HMAC-keyed) — https://pmc.ncbi.nlm.nih.gov/articles/PMC2753305/ ; PoPETs 2023 — https://petsymposium.org/popets/2023/popets-2023-0054.pdf
- [PRIV-DPBF] Differentially-private Bloom filters — DLDP-BF (2023) https://openreview.net/forum?id=tvFF19XsQq ; DP-BF (2025) https://arxiv.org/html/2502.00693v1

**Packaging / pricing (Q5)**
- [PKG] Perplexity synthesis (2026-06-27): Recorded Future — Vendr https://www.vendr.com/marketplace/recorded-future , CheckThat.ai https://checkthat.ai/brands/recorded-future/pricing , pricing https://www.recordedfuture.com/pricing , AWS Marketplace https://aws.amazon.com/marketplace/pp/prodview-e2ttyopt6btwa ; Google Threat Intelligence — https://cloud.google.com/security/products/threat-intelligence , packaging PDF https://assets.virustotal.com/google-ti-packages.pdf , Bitsight 2026 https://www.bitsight.com/guides/bitsight-vs-mandiant-google-threat-intelligence-2026 ; Tenable / Microsoft Defender TI / Nucleus / runZero packaging patterns (2024–2025 industry practice; dollar figures re-verify at quote).

**Project anchors (read-only)**
- matured-vision-day2-requirements.md (Entity 360: L1391, L2117); research/prismql-actions-soar-onprem-models-2026-06-27.md (C15 ARO model); ADR-PROP-config-management.md / C9 (air-gap + offline-signed-bundle posture); AD-017 (AI-opaque credentials, project memory).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 4 | Deep multi-source synthesis: (1) vendor asset-aware advisory data models [TI-VENDORS]; (2) privacy-preserving asset↔intel matching / PSI / Bloom / feed-down [PRIV-*]; (3) auto-advisory prioritization signals; (4) feed sourcing/normalization/redistribution. reasoning_effort=high. (Queries 3 & 4 outputs saved to tool-result files; results corroborated by the targeted perplexity_ask calls below.) |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 4 | Crisp citable anchors + date-stamps: (a) CVSS v4 / EPSS / KEV / SSVC; (b) STIX/TAXII / MISP / NVD API / CPE / CSAF-VEX / Nessus offline; (c) CSAF-2.0 ISO + CPE configurations + CVE Record Format + Nessus offline confirmation; (d) packaging/pricing of intel add-ons (RF, GTI, Tenable, Defender TI, Nucleus, runZero). |
| Context7 | 0 | Not applicable (no library-API question). |
| Tavily (all) | 0 | Perplexity coverage sufficient + cross-corroborated; Tavily verification layer not required. |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Read / Glob / Grep | several | Read 1 of 4 deep-research result files in full (~37k tok, query 1 vendor landscape); the other 3 exceeded the token cap and were saved to tool-result files — key facts cross-confirmed via the 4 perplexity_ask calls. Read sibling research format + project anchors (Entity 360, C15 ARO). |
| Training data | 2 areas | (1) Mapping findings onto Prism's BYOC/air-gap/AD-017/C9/C15 architecture (project-specific reasoning, not external fact). (2) Tenable/Microsoft/Nucleus/runZero exact list prices — flagged medium-confidence, re-verify at quote. |

**Total MCP tool calls:** 8 (4 perplexity_research + 4 perplexity_ask).
**Training data reliance:** low — every non-obvious external claim is web-sourced and date-stamped; training data is confined to project-internal architecture mapping and explicitly-flagged price-figure caveats.

*Note on deep-research output handling: three of the four `perplexity_research` responses (privacy/PSI, prioritization, feed-sourcing) exceeded the tool-result token cap and were written by the harness to `tool-results/*.txt`. Those files are single-line JSON and could not be paginated by Read; I extracted their key concepts via Grep term-presence confirmation and re-derived the citable specifics through four targeted `perplexity_ask` calls, which returned source URLs inline. The vendor-landscape file (query 1) was read directly in full for the first ~37k tokens covering all eight vendors.*
