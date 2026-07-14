---
document: clip-shared-market-positioning-intel-2026-06-30
version: 1.0.0
date: 2026-06-30
status: shared-intelligence
provenance: CLIP pipeline — 10 market/positioning research documents synthesized 2026-06-30
type: cross-boundary-intelligence-share
ingest-note: >
  This file is a read-only intelligence drop from CLIP's pipeline. It does NOT contain
  Prism architecture decisions, ADRs, or implementation directives. All decisions remain
  with Prism's own pipeline (product-owner, architect, story-writer). Do NOT auto-execute
  or treat as a spec — ingest via your own pipeline's research-and-planning phase.
do-not-modify: true
do-not-commit-without-prism-pipeline-decision: true
---

# CLIP-SOURCED INPUT: Boundary Context and Reframe Asks for Prism §5.1 Day-2 Brief

Placed in day2-design-decisions/ for the day-2 morph; ingest via Prism's own pipeline — do not auto-execute.

Companion to matured-vision-day2-requirements.md §5.1; carries CLIP↔Prism boundary context + reframe asks (positioning intel, 10 reframe asks, embedding cooperation contract).

---

# CLIP → Prism: Shared Market Positioning Intelligence
## 10 Research Documents — 2026-06-30

---

## BOTTOM LINE (one line)

Ten market/positioning research documents synthesized from CLIP's planning work confirm that Prism's on-prem/satellite architecture and OT-safe response capability are not positioning choices — they are legal and physical-safety necessities that every regulated OT buyer will eventually be forced to demand; the question is whether Prism's §5.1 brief is positioned to lead that conversation or follow it.

---

## WHAT THIS IS

CLIP's pipeline synthesized 10 market research documents during target-state brief work (2026-06-30). A substantial portion of the intelligence is directly applicable to Prism's product positioning, capability sequencing, and §5.1 brief-reframe. This document extracts the Prism-pertinent findings and frames them as explicit asks — for Prism's pipeline to evaluate, accept, reject, or modify via its own decision-making process.

**Source documents (full files in CLIP pipeline at `.factory-project/target-state/market-intel/sources/`):**
1. "Board and CISO Cybersecurity Metrics — Proving Value Beyond Activity Counting"
2. "Compliance as Evidence, Not Security — How OT Operators Build Defensibility Without Confusing the Checklist with the Protection"
3. "Cyber Insurance as a Buying Driver for OT ICS Security — Prism Positioning Intelligence"
4. "Data Sovereignty and Local Analysis in OT Security — Why the Architecture of the Service Is as Important as the Service Itself"
5. "Managed Security Service Visibility and Renewal Risk — Why MSSP MDR Customers Churn and How Prism Protects Its ARR"
6. "OT Asset Inventory as a Business Problem — What Becomes Possible Once the Inventory Is Trusted"
7. "OT Downtime and Business Interruption Economics — The Consequence Case for Prism"
8. "OT ICS Cybersecurity Buying Triggers — SOC-in-a-Box / Prism Positioning"
9. "OT-Safe Response and Recovery Gap — The Case for Elevating OT IR to a Core Prism Problem"
10. "What Cybersecurity Buyers Actually Value Beyond 'Being Secure'"

---

## BOUNDARY CONTEXT PRISM SHOULD HOLD

The human has ratified the following boundary definitions. Prism's pipeline should hold these as the architectural and commercial frame when evaluating the asks in this document:

- **Prism = See/Decide/Act.** Prism is the engine: ingest, normalize, detect, query (See/Decide) AND actuation — SOAR, OT-IR response (Act). Prism is deployed INSIDE the client environment (on-prem/satellite, including air-gapped) because actuation must execute locally.

- **CLIP = Assure/Operate.** CLIP makes Prism's work VISIBLE to the client: evidence/compliance/audit layer (Assure) + portal/case/client/business layer (Operate). CLIP is cloud-hosted and receives only intelligence outputs from Prism — not raw OT telemetry.

- **Managed-service clients always have CLIP and Prism together** — the primary commercial model. Three offerings: CLIP SaaS (standalone), Prism standalone (on-prem/SaaS), Managed Services (CLIP+Prism combined, always).

- **Standing decisions in force:** [D-CLOUD-POSTURE] cloud-native + portable (any cloud/BYOC); [D-PRISM-VENDOR-AGNOSTIC]; architecture = ports-and-adapters + config-bound backing services + k8s.

The CLIP↔Prism data flow constraint (established by data sovereignty law, detailed below): raw OT telemetry stays inside the client environment (Prism). Only normalized intelligence outputs — detections, risk signals, compliance artifacts, evidence packages — flow to CLIP. This is the "intelligence transport not data transport" model.

---

## PART I — LOUDEST SIGNALS

### Signal 1 — OT-Safe Response and Recovery Is a Core Prism Problem (Doc 9)

**Source:** "OT-Safe Response and Recovery Gap — The Case for Elevating OT IR to a Core Prism Problem"

**Thesis:** Standard IT incident response applied to OT environments is not merely suboptimal — it is documented as actively dangerous. "Isolate the host" — the foundational IT containment action — can halt production, damage equipment, or cause safety events when applied to OT. This creates a definitional requirement that OT-safe response capability must reside inside the client environment with OT-trained responders who understand the process context. Detection without OT-safe response is a partial service with a defined failure mode.

**The load-bearing arguments:**

The priority inversion (from SANS ICS, Dean Parsons, 2026):
> "Applying IT-centric IR actions in OT — such as aggressive containment, indiscriminate isolation, or automated shutdowns — can halt production, damage equipment, or create unsafe operating conditions. This creates a massive and dangerous gap in critical infrastructure defense."

The IT-vs-OT priority stack (from RelyBlue ICS/OT IR Practitioner Guide, 2026):
> "IT IR is optimized around Confidentiality → Integrity → Availability. OT IR must be built around Safety → Availability → Integrity → Confidentiality — in that order. This is not pedantic. It dictates every decision in the first hour of an incident."

The managed SOC authority problem (from LinkedIn Managed SOC vs MSSP analysis, 2026):
> "If the provider can only notify, escalate, and recommend, then the buyer has not outsourced response. The buyer has outsourced alarm generation. Buying response without giving response authority is theater."

The market verdict from Doc 9's own summary:
> "Detection without OT-safe response capability is a partial service with a defined failure mode: the customer detects the incident, escalates to their own team, and then faces the same structural challenges that Prism was supposed to solve — only now under live incident conditions, at 2 AM, with physical process safety at risk."

**Market data from Doc 9:**
- 40% of OT incidents disrupt operations; 20% require more than a month for full recovery (SANS State of ICS/OT Security 2025, 330 respondents)
- 88% of Dragos tabletop exercises in 2025 revealed degraded detection capabilities
- Colonial Pipeline PHMSA fine ($1M proposed, May 2022): cited "failures to adequately plan and prepare for a manual restart and shutdown operation" — not for being hacked, but for not having a tested plan
- FrostyGoop (January 2024): attackers downgraded firmware on ENCO heating controllers to a version that lacked monitoring capabilities, preventing operators from seeing process state during attack and recovery — establishes that pre-incident firmware baselines are required for clean-to-restart verification

**Six structural gaps the research identifies for OT IR (all requiring on-prem capability):**
1. IT-trained analysts applying wrong containment actions (OT-certified analysts required)
2. No pre-defined authority to act (Response Authorization Matrix as onboarding artifact)
3. No tested manual restart plan (OT IR plan + quarterly tabletop exercises)
4. No configuration baseline for clean-to-restart (continuous asset baseline as recovery verification artifact)
5. No passive OT evidence collection (continuous monitoring as native forensic record — without touching running PLCs)
6. No post-incident defensibility package (telemetry + authorization records + exercise history)

**What this means for Prism's product/roadmap:** The six structural gaps above define a named Prism capability domain: OT-Safe Response and Recovery. Each gap corresponds to a specific Prism deliverable. The research recommends this be elevated from candidate to core — Doc 9's verdict is explicit: "Yes — Immediately." This is intelligence for Prism's pipeline to evaluate; Prism's product-owner and architect decide whether and how to incorporate it.

---

### Signal 2 — Data Sovereignty and Local Analysis Is a Structural Prism Differentiator (Doc 4)

**Source:** "Data Sovereignty and Local Analysis in OT Security — Why the Architecture of the Service Is as Important as the Service Itself"

**Thesis:** OT security telemetry is legally regulated data. The CLOUD Act (18 U.S.C. §2713) requires U.S.-incorporated entities to produce data stored anywhere in the world when compelled by U.S. government order — regardless of physical data location. OT security monitoring data qualifies as BES Cyber System Information (BCSI) under NERC CIP-011-3, requiring entity-managed encryption keys (Model 1). Every major cloud-hosted OT security vendor faces this structural exposure. Prism's on-prem/satellite deployment is the only architecture simultaneously compliant with NERC CIP data handling requirements and immune to CLOUD Act extraterritorial compulsion.

**The CLOUD Act Matrix competitive exposure:**
The sota.io CLOUD Act Matrix scores: Claroty 18/25, Dragos 17/25, Nozomi 14/25 — all rated 5/5 on U.S. government jurisdiction exposure. This means any regulated utility using these vendors to store OT telemetry in cloud infrastructure is creating BCSI handling risk under NERC CIP-011-3.

**The architecture reframe (from Doc 4):**
The correct model is "intelligence transport not data transport": Prism processes OT telemetry on-prem (or in the client's satellite/air-gapped environment); only normalized intelligence outputs flow to CLIP. This reframe — federated query architecture where "MSSP analysts query OT data where it lives; only results return" — is not a marketing preference; it is the regulatory compliance design.

**Regulatory horizon:** NERC CIP-100 Series (NERC Project 2023-09) is in development targeting end 2027. This will add cloud compliance requirements that will formalize the CLOUD Act/BCSI tension into explicit NERC standards. Prism's on-prem architecture is positioned ahead of this regulatory curve; cloud-hosted competitors are positioned behind it.

**What this means for Prism's product/positioning:** Data sovereignty as an explicit differentiator requires that Prism's §5.1 brief name the CLOUD Act + BCSI compliance argument as a structural competitive advantage — not buried in technical architecture notes but surfaced as a headline positioning argument for regulated utility and critical infrastructure buyers. The federated query model ("intelligence transport not data transport") should be a named Prism concept in §5.1.

---

## PART II — CONSEQUENCE AND ECONOMIC FRAMING (Docs 7, 3, 10)

### Doc 7 — OT Downtime Economics: The "Ransom Is Never the Cost" Frame

**Source:** "OT Downtime and Business Interruption Economics — The Consequence Case for Prism"

**Core frame:** In every major industrial cyber incident on record, the ransom payment was the smallest line item. The real cost is downtime — at $260,000/hour for manufacturing and $410,000/hour for energy/utilities (IBM/ITIC 2025).

**Evidence base:**
- Colonial Pipeline: $4.4M ransom; 6-day shutdown of 45% of U.S. East Coast fuel supply
- Norsk Hydro: $0 ransom (refused); $71M total recovery; insurance paid $3.6M (6% recovery rate)
- Maersk/NotPetya: $0 ransom (wiper); $250-300M total impact
- JBS: $11M ransom; $100M+ total impact; 20% of U.S. beef/pork processing offline 48-96 hours
- Clorox: $49M disclosed in SEC 8-K filings across two quarters; production unavailable on retailer shelves for months

**The abundance-of-caution shutdown (highest-value Prism capability):** 56% of organizations cannot see below the IT/OT boundary (Dragos 2026). When IT ransomware hits, the plant manager gets a call: "Did OT get hit?" Without OT monitoring, the only safe answer is "shut it down." With Prism monitoring, the answer is: "Segmentation held — OT networks show no anomalous activity — production is clear to continue." That answer is worth hours of production at $260K+/hour per avoided unnecessary shutdown.

**SEC material disclosure standard (Clorox and Johnson Controls, 2023):** Both companies filed SEC 8-K disclosures for cyber-induced production disruption. Both required CEO/CFO personal certification under Sarbanes-Oxley. Prism's documented evidence trail — the continuous telemetry record that proves what happened and when — is the defense against exposure in this disclosure obligation.

**Recovery time as the cost multiplier:** Norsk Hydro recovered over 90 days. Maersk took 10 days for basic function. Clorox took two quarters. Pre-incident Prism deployment (asset baselines, IR plan, documented RTOs, recovery sequencing) compresses recovery from months to days.

**What this means for Prism's positioning:** "Prism is not a security cost — it is the instrument that keeps your plant running" and "the answer that prevents the precautionary shutdown" are the two load-bearing positioning statements for OT buyer audiences. The downtime cost calculator (prospect's production volume × $260K or $410K/hour × avoided incident duration) should be a standard Prism sales tool.

---

### Doc 3 — Cyber Insurance: The Actuarial Validation of Prism's Capability Stack

**Source:** "Cyber Insurance as a Buying Driver for OT ICS Security — Prism Positioning Intelligence"

**Core frame:** The SANS ICS 5 Critical Controls have actuarial validation from Marsh McLennan's insurance claims database (10 years, thousands of claims). The five controls with their risk reduction percentages:

| Control | Risk Reduction |
|---|---|
| Incident Response Planning | 18.46% |
| Defensible Architecture | 17.09% |
| ICS Network Monitoring | 16.47% |
| Vulnerability Management | 13.87% |
| Secure Remote Access | 12.18% |

These five controls map directly to Prism's core capabilities. Combined: 78.07 percentage points of actuarially-validated OT risk reduction across all five areas. This is not analyst opinion — it is insurance claims data.

**Critical nuance:** Misconfigured MFA causes MORE insurance losses (26%) than missing MFA (8%). Evidence quality and implementation quality matter more than control presence. This reinforces Prism's continuous monitoring and validation capability as the differentiator over point-in-time assessments.

**The $31.1B OT cyber risk market** establishes the commercial scale. The $25M and $150M revenue band thresholds create tiered coverage requirements that map to Prism's market segments. Lloyd's mandatory state-backed attack exclusions (Y5381, LMA5567A/B 2026) mean the largest OT incidents are the least insurable — making Prism's prevention capability the only reliable risk mitigation in state-affiliated threat scenarios.

**What this means for Prism's positioning:** Prism should map its five core capabilities explicitly to the SANS ICS 5 Controls with the actuarial percentages in §5.1. "Prism delivers the actuarially-validated five controls that Marsh McLennan's claims database validates as the highest-signal OT risk reduction levers" is a direct ROI argument grounded in insurance data, not vendor claims.

---

### Doc 10 — What Buyers Actually Value: The Career-Protection and Resilience Frames

**Source:** "What Cybersecurity Buyers Actually Value Beyond 'Being Secure'"

**Core frame:** Enterprise buyers are not purchasing protection — they are purchasing outcomes that protect careers, satisfy regulators, preserve insurance, justify budgets, and maintain board confidence. The security product is a means to these ends.

**The numbers:**
- 78% of CISOs fear personal legal liability for security incidents (Splunk 2026, 650 CISOs) — up from 56% in 2025 (39% increase in 12 months)
- 77% of board directors now discuss material financial implications of cyber incidents (+25 points since 2022, NACD 2025)
- 41% of CISOs cannot calculate ROI on their own security investments (Splunk 2026)
- Only 2% of organizations have implemented firm-wide cyber resilience (PwC 2025, 4,042 executives)
- 65% of large companies cite third-party/supply chain vulnerabilities as their greatest resilience challenge (WEF 2026)

**The executive evidence frame:** The SolarWinds CISO was personally charged. The Uber CISO received a criminal conviction. The Clorox CISO departed post-incident. Colonial Pipeline's CEO testified before Congress under conditions of uncertainty ("we didn't know whether OT was compromised"). 30% of Dragos IR cases began with "something seemed wrong" — no systematic detection, no evidence trail. An organization without OT monitoring has no documentation for congressional testimony, board presentation, or regulatory response. Prism's continuous telemetry is the evidence record.

**The resilience frame (boards want this, not prevention):** Gartner: at SRM 2026, very few hands went up when attendees were asked if they had defined their "minimum viable operations." Gartner predicts 50% of CISOs will own disaster recovery in addition to incident response by 2028. The purchase is increasingly "assured recovery capability" — can we prove to the board, auditors, and insurers that when something fails, we have a tested plan?

**What this means for Prism's positioning:** Prism's OT-IR planning service (Response Authorization Matrix, manual restart procedures, tabletop exercises, clean-to-restart verification) is directly the "minimum viable operations" product the board wants to see documented. The career-protection framing — "when the board asks what we did and when, Prism's telemetry answers with evidence, not memory" — should be a named Prism value proposition for CISO audiences.

---

## PART III — PRISM CAPABILITY INTELLIGENCE (Docs 6, 5, 8, 2, 1)

### Doc 6 — Asset Inventory as Business Problem: Six Decisions, Not One Feature

**Source:** "OT Asset Inventory as a Business Problem — What Becomes Possible Once the Inventory Is Trusted"

**Core reframe:** OT asset inventory is not a security feature — it is the operational data infrastructure for six specific business decisions simultaneously: vulnerability prioritization, segmentation verification, insurance claim defense, configuration change detection, board risk reporting, and EOL capital planning. The old frame ("you can't protect what you don't know") is accurate but insufficient.

**Key data:**
- Only 12.6% of industrial organizations have full visibility across the ICS Cyber Kill Chain (SANS 2025, 330 respondents)
- OT asset inventory is the #1 technology investment priority for 2025-2026 (50% of respondents in 2025; 54% targeting 2026-27)
- Remote sites: only 17.5% of organizations have OT monitoring coverage across distributed operations
- Shadow OT: 42% of enterprise assets are agentless (carrying 64% of mid-to-high risk) — ORDR 2024, 100M+ device profiles
- Compliance uplift: organizations with compliant inventory programs experience 50% fewer financial losses when incidents occur (SANS 2025)
- CISA + NSA joint publication (August 13, 2025): established asset inventory as the foundational building block of OT cybersecurity
- Exploits weaponized within 24 days of disclosure (Dragos 2026); only 13% of OT-applicable CISA KEV entries include vendor workarounds — 87% have no formal compensating control guidance

**The six business decisions enabled by trusted inventory:**

| Decision | Blocked Without Inventory | Enabled With Inventory |
|---|---|---|
| "Which vulnerabilities affect my plant?" | Generic advisories, no site relevance | Site-specific, firmware-version-specific work order |
| "Did segmentation hold?" | Unknown — must shut down | Verified in minutes from network telemetry |
| "Is our insurance application accurate?" | Estimates and spreadsheets | Current, auditable asset register |
| "Is that configuration change authorized?" | No baseline — undetectable drift | Real-time comparison against known-good state |
| "What do we tell the board?" | Qualitative opinion | Dollar-denominated, criticality-classified risk statement |
| "Which EOL assets should we replace?" | No data — gut feel | Prioritized capital plan by criticality, EOL status, KEV coverage |

**The EOL capital planning angle:** OT devices routinely run 20-30 years. A PLC installed in 2010 with Windows XP embedded has had 12+ years of unpatched vulnerabilities. Prism's EOL-flagged asset inventory with criticality classification converts security data into capital planning input — a deliverable that has budget value outside the security function (operations, facilities, finance).

**What this means for Prism's product:** The six business decisions (not "asset inventory as a feature") should be the §5.1 capability frame for Prism's discovery/inventory domain. The EOL capital planning report as a named standard deliverable. Shadow OT discovery (finding devices not in any existing inventory) as a named Prism differentiation claim. MSSP remote site coverage (82.5% of distributed operations currently unmonitored) as a structural MSSP architecture advantage.

---

### Doc 5 — MSSP Churn: Prism's Role in ARR Protection

**Source:** "Managed Security Service Visibility and Renewal Risk — Why MSSP MDR Customers Churn and How Prism Protects Its ARR"

**Core frame:** 58% of managed security customers plan to switch providers despite 94% feeling protected (WatchGuard 2026, ~1,000 respondents). The churn driver is not security outcomes — it is visibility failure. 60% of MSSPs say client communication is harder than the security work itself (MSSP Alert 2024, ~2,000 respondents). Silent churn begins 6-12 months before non-renewal.

**What Prism's role is in ARR protection (per the boundary definition):** The client-facing visibility mechanism is CLIP's job — CLIP's portal, Communication Bridge, ARO framework, and three-tier reporting are the anti-churn architecture. But Prism's role is foundational: the detection telemetry, asset baselines, configuration change records, and compliance evidence that Prism generates are the underlying content that CLIP's visibility layer presents. Without Prism producing substantive, business-translatable security outcomes, CLIP has nothing to make visible. Poor Prism signal quality (high false positive rate, low coverage below SCADA layer, gaps in remote site monitoring) directly degrades CLIP's ability to communicate value.

**73% cite false positives as the #1 MSSP challenge (SANS 2025):** False positive rate is a Prism quality metric, not a CLIP metric. Prism's detection precision directly determines how much of CLIP's communication is noise vs. signal.

**Arctic Wolf competitive benchmark (4.9/5.0, 241 reviews, 99% recommend):** Differentiated by the Concierge Security Team model — business-language translation by named, dedicated contacts. The detection capability underlying this is standard; the translation and visibility is what wins the renewal. Prism's job is to make the detection capability substantive enough to translate.

**What this means for Prism's product:** Detection precision (false positive reduction) and coverage completeness (including Levels 1-2 and remote sites) are ARR-protection metrics for Prism, even though the client-facing visibility is CLIP's domain. Prism's signal quality is the upstream input to CLIP's value communication capability.

---

### Doc 8 — Buying Triggers: Prism-Anchored Entry Points

**Source:** "OT ICS Cybersecurity Buying Triggers — SOC-in-a-Box / Prism Positioning"

**Key Prism-anchored Tier 1 triggers (immediate budget release):**

1. **NERC CIP-015 mandate (~400 U.S. registered entities):** FERC approved June 26, 2025; enforcement phased through 2028-2030. Mandates east-west traffic monitoring inside Electronic Security Perimeters. Prism's INSM capability is the compliance delivery mechanism. Defined, finite prospect list.

2. **Peer ransomware hit:** 119 ransomware groups targeted 3,300+ industrial organizations in 2025 (Dragos 2026); manufacturing accounts for 65%+ of victims. When a peer gets hit, every plant manager in the sector asks "could that happen here?" This is the fastest path from zero budget to approved project. Prism's response: production-downtime cost calculation for their specific vertical + Prism as prevention.

3. **TSA pipeline directive compliance:** SD-2021-02F and SD-2021-01G require MFA for remote access, network segmentation ensuring OT can operate if IT compromised, 100% control testing over 3 years. Non-compliance carries civil penalties up to $15,000/day.

4. **IT/OT convergence:** 58% of ICS/OT incidents originate from IT compromises (SANS/OPSWAT 2025); 81% of OT security assessments found poor IT/OT segmentation (Dragos 2026, field data). 73% of all-time Dragos IR cases involved compromised VPN/jump-host credentials as the OT access mechanism. Prism's IT/OT boundary monitoring is the technical answer.

5. **OT talent shortage (structural outsourcing driver):** Only 9% of professionals dedicate full time to ICS/OT security (SANS/OPSWAT 2025). 47% of leaders cite lack of qualified OT security personnel as a top challenge (PwC 2026, 3,887 executives). OT security market growing at 15.1% CAGR in the U.S. to $9.37B by 2030, driven by managed services adoption. The talent gap is structural and permanent — outsourcing is the only operationally viable path for most industrial operators.

**Key Prism-anchored structural triggers:**

6. **Detection-without-response gap (best near-term addressable market):** Organizations already running Claroty, Nozomi, or Dragos sensors but lacking 24/7 OT IR. They have purchased the visibility argument; Prism provides the response and recovery layer. This is the upgrade-sale motion.

7. **Assessment-led sales motion:** Only 30% of OT networks have adequate visibility (Dragos 2026). An OT risk assessment (30-day asset inventory, IT/OT boundary map, top-10 risk findings) consistently produces written findings that activate purchase authority. The assessment-to-SOC pipeline should be a named Prism go-to-market product.

**CISO consolidation context:** 52% of CISOs now own OT security vs 16% in 2022 (Fortinet 2025, 550+ respondents). This creates a unified buyer for an IT+OT managed service — a CISO who owns OT security but lacks OT expertise is the natural Prism+CLIP managed service buyer.

---

### Doc 2 — Compliance as Evidence: Prism as the Compliance Delivery Mechanism

**Source:** "Compliance as Evidence, Not Security — How OT Operators Build Defensibility Without Confusing the Checklist with the Protection"

**The three-layer framework:**
1. Compliance (what regulators mandate): NERC CIP-015-1 INSM monitoring, TSA CIRP testing, IEC 62443 zone-and-conduit documentation, NIST SP 800-82 control coverage
2. Defensibility (evidence that controls work — visible to regulators/insurers/boards): Prism generates this; CLIP presents it
3. Security (actual risk reduction): Prism delivers this, but it is invisible without layers 1 and 2

**Key regulatory deadlines for Prism's prospect calendar:**
- NERC CIP-015-1 INSM: approved June 2025, enforcement phasing 2028-2030
- SEC Regulation S-P (board active oversight requirement): compliance June 2026
- TSA pipeline directives: annual CIRP testing cycle, 100% control testing over 3 years
- CIP-100 Series cloud compliance (in development): targeting end 2027

**21% insurance claim denial rate:** Poor compliance documentation causes denial of valid claims. Prism generates the underlying control evidence; CLIP packages and presents it as the Defensibility layer.

**IEC 62443 vs. NIST CSF 12% cross-coverage:** Most organizations operate under multiple concurrent regulatory frameworks. Prism's compliance evidence artifacts should be multi-framework (IEC 62443 zone-and-conduit mapping, NIST SP 800-82 control coverage, NERC CIP-015 INSM evidence, TSA directive documentation) rather than framework-specific.

---

### Doc 1 — Board Metrics: Prism as the Evidence Source for Board-Legible ROI

**Source:** "Board and CISO Cybersecurity Metrics — Proving Value Beyond Activity Counting"

**The gap:** 93% of board members agree cyber risk threatens shareholder value (NACD 2025). Most CISOs still present activity metrics (patch percentage, alert counts, MTTD). 41% of CISOs cannot calculate ROI. Boards want financial-risk language (ALE, ROSI, MPL), not security operations dashboards.

**Prism's role in closing this gap:** Prism's detection telemetry and incident evidence are the raw inputs that CLIP's Decision-DNA/evidence mesh translates into board-legible financial risk metrics (ALE, ROSI, ARO). Without Prism generating substantive, structured security evidence, CLIP cannot produce meaningful financial-risk outputs. Prism's data quality, coverage completeness, and detection precision are the upstream determinants of CLIP's board reporting quality.

**The FAIR/CRQ implication:** FAIR (Factor Analysis of Information Risk) adoption growing from 46% to 58% (Doc 1). CRQ (Cyber Risk Quantification) is becoming a board reporting standard. Prism's telemetry should be structured to feed FAIR/CRQ models — frequency of incidents detected/prevented, asset criticality classifications, downtime cost parameters by system type. This is a data modeling consideration for Prism's detection output schema.

---

## PART IV — ASKS FOR PRISM'S §5.1 REFRAME

These are intelligence-derived asks from CLIP's analysis. Prism's pipeline (product-owner and architect) decides whether and how to incorporate each. None are mandates.

**Ask 1 — Elevate OT-Safe Response and Recovery to a Core Prism Problem (from Doc 9):**
The research recommends this be elevated from "candidate" to "core problem definition." The six structural gaps (see Signal 1 above) define a specific capability domain. The ask: name OT-Safe Response and Recovery as a primary Prism pillar in §5.1, alongside detection/visibility. Define the standard deliverables: Response Authorization Matrix (onboarding), tested manual restart plan (onboarding + quarterly), clean-to-restart verification report (post-incident), post-incident defensibility package (post-incident).

**Ask 2 — Name Data Sovereignty / Local Analysis / Federated Query as an Explicit Prism Differentiator (from Doc 4):**
The CLOUD Act + BCSI argument is currently not articulated as a headline differentiator. The ask: explicitly position "intelligence transport not data transport" and the federated query architecture as a named Prism differentiator in §5.1, with the CLOUD Act Matrix competitive context (Claroty/Dragos/Nozomi all rated 5/5 on U.S. government jurisdiction exposure). This should be a top-three §5.1 positioning argument for regulated utility buyers.

**Ask 3 — Map Prism's Capabilities to the SANS ICS 5 Controls with Actuarial Percentages (from Doc 3):**
The five controls have insurance claims data validation. The ask: explicitly map Prism's five core capability domains to the SANS ICS 5 Controls in §5.1 with the Marsh McLennan actuarial reduction figures (IR Planning 18.46%, Defensible Architecture 17.09%, ICS Monitoring 16.47%, Vulnerability Management 13.87%, Secure Remote Access 12.18%). This converts capability claims to actuarially-validated ROI.

**Ask 4 — Name "Abundance-of-Caution Shutdown Prevention" as a Headline Prism Value Proposition (from Doc 7):**
The ability to answer "did segmentation hold?" in minutes — preventing precautionary production shutdowns at $260K+/hour — is the most financially measurable single Prism capability. The ask: make this a §5.1 headline, not a buried technical note. Frame as: "Prism answers the call that keeps your plant running."

**Ask 5 — Name the Six Business Decisions Enabled by Trusted OT Asset Inventory in §5.1 (from Doc 6):**
The old frame ("asset visibility for security") understates the business case. The ask: reframe Prism's inventory capability around the six business decisions (vulnerability prioritization, segmentation verification, insurance claim defense, change detection, board reporting, EOL capital planning) in §5.1. Add EOL asset capital planning report and OT visibility assessment as named standard Prism deliverables.

**Ask 6 — Name the CISO Career-Protection Frame for IR Telemetry (from Docs 9, 10):**
78% of CISOs fear personal legal liability. Prism's continuous telemetry is the forensic record that makes post-incident documentation possible. The ask: name "the evidence record that protects the CISO when the board asks what we did and when" as a Prism value proposition for CISO audiences in §5.1. This is a CISO-level sales message, distinct from the plant manager (downtime avoidance) and board (financial risk) messages.

**Ask 7 — Define the CLIP-Facing Interface Contract Prism Must Expose (boundary architecture ask):**
CLIP receives only normalized intelligence outputs from Prism (not raw OT telemetry). The ask: Prism's §5.1 reframe should include a named "CLIP interface" section specifying what structured outputs Prism exposes to CLIP: (a) normalized detections (MITRE ATT&CK for ICS mapped); (b) asset inventory updates (firmware-level, criticality-classified, EOL-flagged, KEV-correlated); (c) compliance evidence artifacts (framework-mapped, audit-ready); (d) incident telemetry (continuous passive record for forensic reconstruction); (e) configuration baseline snapshots (for clean-to-restart verification). These are the data contracts between Prism and CLIP. Prism's pipeline should define them explicitly so CLIP can build against them.

**Ask 8 — Flag AI-Enabled OT Social Engineering as a Forward-Looking Capability Gap (from Docs 8, 10):**
PYROXENE (new Dragos-tracked threat group, 2025) conducts multi-year supply chain campaigns via fake LinkedIn profiles targeting OT operations personnel. This is behavioral/social detection at the IT/OT boundary — outside traditional OT protocol-layer detection. The ask: flag this in Prism's §5.1 as a forward-looking capability consideration, not an immediate requirement. The question for Prism's roadmap: does behavioral threat detection of AI-enabled social engineering targeting OT operators fall within Prism's detection scope or is it an IT-side problem handled upstream?

**Ask A (newly-confirmed, 2026-06-30) — SOAR Entitlement Awareness: CLIP gates SOAR on Prism entitlement:**
CLIP entitlement-gates SOAR on Prism entitlement. During Prism rollout there will be CLIP clients WITHOUT Prism who get no SOAR (visibility-only mode) until they are Prism-entitled. This is a human-ratified commercial boundary. The ask: Prism should expose an entitlement/capability handshake so CLIP can gate cleanly — specifically, a machine-readable signal from Prism indicating which SOAR/actuation capabilities are active for a given client. Without this handshake, CLIP cannot enforce the visibility-only vs. SOAR-enabled boundary programmatically. Prism's pipeline should decide whether this is a deployment-metadata endpoint, a provisioning flag in the CLIP interface contract, or a capability advertisement in the Prism-to-CLIP channel — the form is for Prism's architect to determine. The requirement is that the signal exists and is authoritative.

**Ask B (newly-confirmed, 2026-06-30) — Outbound Connection Direction: Prism initiates to CLIP, never the reverse:**
For on-prem Prism (not embedded in CLIP), the human requires that **Prism initiates an OUTBOUND, mutually-authenticated connection to CLIP** so that NO inbound firewall rules are opened into the client's OT environment. CLIP never initiates inbound into the client network. This is a concrete build requirement, not a positioning preference. Over this Prism-initiated outbound channel must flow: action/approval lifecycle status, evidence/audit artifacts, and approval-response routing (e.g., server-push or long-poll over the persistent outbound connection so CLIP can push approvals back through the channel Prism opened). The framing the human specified: "CLIP presents an API status-mirror + evidence surface for on-prem Prism, fed by a Prism-initiated outbound channel; Prism owns the egress connector, CLIP owns the ingress." Concrete day-2 build item for Prism's roadmap: an outbound status/evidence/approval connector that establishes and maintains the Prism→CLIP channel without requiring any inbound port openings. Prism's pipeline should scope and sequence this as part of the CLIP interface contract definition (see Ask 7).

### Embedding cooperation-contract asks (from CLIP ADR-TS-001 research spike, 2026-06-30)

**Decision context Prism should hold:** CLIP's v1 UI embedding mechanism for SaaS Prism is a whole-surface `<iframe>` + hardened `postMessage` channel. Module Federation was evaluated and refuted for v1 (`@module-federation/nextjs-mf` is in maintenance/EOL, carries an install-time deprecation notice as of 8.8.69, and has no Next.js App Router/RSC support as of mid-2026; Vercel has shipped no first-party federation). **Scope fence: this applies to SaaS Prism only. On-prem Prism NEVER embeds into CLIP's UI — it is handled exclusively by the Prism-initiated outbound channel (Ask B above) feeding CLIP's status-mirror and evidence surface.**

All items below are ASKS for Prism's pipeline (product-owner and architect) to evaluate, scope, and sequence. None are mandates from CLIP's pipeline.

**Ask C — Serve `frame-ancestors` CSP as an HTTP response header:**
Prism (SaaS) must serve `Content-Security-Policy: frame-ancestors 'self' https://<clip-origin>` as an **HTTP response header** (not a `<meta>` tag — `frame-ancestors` is ignored in meta). The allow-list must be a small, exact set of CLIP origins with no wildcard subdomains. Sensitive Prism endpoints (SOAR/ARO actuation, approval execution) should narrow `frame-ancestors` per-endpoint, down to `'none'` where embedding is not required. `X-Frame-Options` should remain as a legacy fallback only. CLIP will reciprocally serve `frame-src https://<prism-origin>` so the allow-listing is bilateral. The exact CLIP origin(s) to allow-list will be provided when the embedding contract is finalized; the ask is that the mechanism is designed to accept this per-origin configuration rather than hard-coded.

**Ask D — Implement a ready/handshake protocol:**
On iframe load, Prism posts `{ type: "ready", protocolVersion, capabilities, layout: { minHeight, maxHeight }, supportedEvents }` to CLIP's exact origin. CLIP holds a skeleton/spinner until `ready` arrives; if no `ready` within a configured timeout, CLIP surfaces a fallback ("Prism unavailable / retry"). CLIP replies with a `bootstrap` message carrying: theme tokens, locale, tenant/client_id context (non-secret), and deep-link target. Prism emits `{ type: "authError", code }` when its own session fails so CLIP can surface a global re-auth prompt.

**Ask E — Self-authentication handoff: tokens must NOT cross the iframe boundary:**
Prism (SaaS) authenticates **itself** inside the iframe via silent OIDC (`prompt=none`) against the shared IdP — CLIP sends NO token to Prism. The rule is absolute: no JWT or session token in the iframe `src` URL, in `postMessage` payloads, or in SSR props. CLIP may send only non-secret context (tenant/client_id, deep-link target) and, if needed for server-side handshake, a short-lived single-use narrowly-scoped handoff code redeemed server-to-server (Prism BFF ↔ CLIP/IdP), never a bearer token in the URL. Prism's browser session should use a BFF/HttpOnly-cookie posture (tokens server-side; SPA holds only a `Secure; HttpOnly` cookie + CSRF defense). The postMessage channel carries UI events only — no secrets.

**Ask F — Navigation/history-sync, resize, and theme schemas:**
Prism must implement three postMessage schemas so seams between the CLIP shell and embedded Prism surface are invisible:
- **Navigation:** Prism → CLIP: `{ type: "routeChanged", path }` on every internal navigation (CLIP mirrors into its address bar). CLIP → Prism: `{ type: "navigate", path }` for deep-links and back/forward. Internal navigation must go through `postMessage`, not `src` swaps (swapping `src` pushes history entries onto the iframe and hijacks the browser back button).
- **Resize:** Prism → CLIP: `{ type: "resize", height }` on content-height change (via ResizeObserver/MutationObserver). CLIP sets iframe height. CLIP intends to implement a **custom postMessage resize protocol** rather than taking `iframe-resizer` as a dependency (see procurement note below). Prism should expect to implement the resize message contract on its side rather than depending on that library.
- **Theme:** Prism accepts `{ type: "applyTheme", tokens }` carrying CSS custom properties from CLIP's design system, and applies them internally. This requires agreement on a shared design-token vocabulary between CLIP and Prism — an open coordination item.

**Ask G — Error/timeout surfacing and strict origin discipline:**
- Prism emits `{ type: "error", code, message }` (e.g. `AUTH_MISSING`, `INIT_FAILED`) for fatal states; CLIP renders an in-shell error region rather than a broken frame.
- All `postMessage` sends must specify an exact `targetOrigin` (never `*`). All receives must validate `event.origin` with strict equality against an allow-list (no substring checks). `event.data` is treated as data not code: JSON-schema-validated, `type`-tagged messages; no `innerHTML`/`eval`. The channel carries UI events only.

**Ask H — Protocol versioning:**
The `ready` message includes a `protocolVersion` field. Both sides negotiate on version compatibility so CLIP and Prism can evolve their embedding contract independently without forced lockstep deployments. This is a requirement of the iframe mechanism's deployment-independence advantage.

**Ask I (infra/identity constraint — cross-product coordination required):**
Silent OIDC inside the iframe requires the shared IdP to sit on a **custom domain that is first-party to BOTH CLIP and Prism** — i.e., sibling subdomains of a single registrable domain (e.g. `login.<root>`, `app.<root>`, `prism.<root>`) so the SSO cookie is treated as first-party by the browser and survives third-party-cookie deprecation. This is an infrastructure constraint that must be coordinated across both pipelines and surfaced to the architect and devops-engineer. The entitlement gate lives in CLIP, but Prism must still independently authorize every server-side call (defense-in-depth; Prism never trusts CLIP's gate as sufficient).

**Procurement note (FYI — not an ask):** `iframe-resizer` moved from MIT to GPL-3.0 in v5 (latest 5.5.9 as of 2026-06-30, verified against npm registry). For a commercial security product, CLIP intends a custom resize protocol rather than taking a GPL-3.0 runtime dependency. Prism should expect to implement the resize message contract on its side rather than depending on the `iframe-resizer` library. This is informational; no Prism action required until the resize contract schema is finalized.

---

### CLIP storyboard carry asks (from CLIP Stage-6.5 frame-01 findings, 2026-07-13)

Two asks appended 2026-07-13 from CLIP Stage-6.5 frame-01 design-validation findings (Burst 159): SB65-8/FS-10 and SB65-9/FS-14, both CARRIED pending ADR-TS-001 resolution, before-Stage-8-promotion trigger. These concern the CLIP↔Prism PC-contract layer, not the iframe embedding mechanism. All items are asks for Prism's pipeline to evaluate — not mandates.

**Ask J (newly-escalated, 2026-07-13) — CLIP → Prism re-recommendation request contract:**
CLIP requires a contract path to signal Prism to re-analyze a case and issue a refreshed ARO after TTL expiry. The H-2 state ("Expired / awaiting re-recommendation") presents a "Request refreshed recommendation" CTA; that CTA has no backing BC and no PC contract. PC-002 is Prism → CLIP only (ARO delivery); no receive-side contract defines a request path from CLIP back to Prism. The carrier for any such contract is ADR-TS-001 (HELD — Prism boundary.md §Boundary Statement).

Whether Prism accepts client-requested re-analysis vs. purely self-initiating ARO generation is Prism-internal (D-PRISM-OWNS-SOAR). The ask is not to mandate a behavior — it is for Prism to confirm (a) whether the capability exists or is on the roadmap, and (b) if so, to agree a carrier (channel, message type, protocol) so a new PC contract and a CLIP-side BC can be authored for the analyst waiting-state behavior post-CTA click. Without this confirmation, the H-2 CTA remains a [design hypothesis]; no binding BC can be written, and frame-01 cannot advance to Stage-8 promotion.

**Citation:** CLIP STORYBOARD-INDEX SB65-8, frame-01 state H-2, ADR-TS-001 (HELD).

---

**Ask K (newly-escalated, 2026-07-13) — PC-004 `superseded` execution_status enum extension:**
PC-004's `execution_status` enum (success | failed | cancelled) has no value for "ARO was superseded by Prism before or while the approval was consumed." The unmodeled scenario is unknown supersession: Prism supersedes an ARO during a live-channel outage (frame-01 state M); CLIP never received an `AROActionSuperseded` signal during the outage; the analyst approves with staleness acknowledged; CLIP sends PC-003; Prism has already superseded the ARO. The PC-004 return carries no signal to distinguish this case from a clean failure or cancellation. Adding `superseded` to `execution_status` is a contract change requiring ADR-TS-001 ratification for the carrier.

What Prism returns when it receives a PC-003 for a superseded ARO is Prism-internal (D-PRISM-OWNS-SOAR). The ask is for Prism to confirm the actual return behavior and agree the enum extension so CLIP can author a post-approval-supersession BC for the frame-01 state M → G transition path. Note: SB1-2's H-superseded sub-variant covers KNOWN supersession (CLIP received `AROActionSuperseded` before the analyst approved); this item covers UNKNOWN supersession revealed only via PC-004 return — two distinct failure modes requiring separate BC treatment. Without this confirmation, state M and the post-approval G path have no BC for the supersession case, and frame-01 cannot advance to Stage-8 promotion.

**Citation:** CLIP STORYBOARD-INDEX SB65-9, frame-01 states M/G, SB1-2 (known-supersession separately handled), ADR-TS-001 (HELD).

---

## APPENDIX — Source Document Index (Prism-Pertinent Rank Order)

| Doc # | Title (abbreviated) | Primary Prism Relevance | Key Metric |
|---|---|---|---|
| 9 | OT-Safe Response and Recovery Gap | Core Prism capability definition | 40% disruption, 20% >30 days recovery, $1M PHMSA fine |
| 4 | Data Sovereignty and Local Analysis | Primary structural differentiator | CLOUD Act §2713, BCSI, 5/5 jurisdiction exposure for competitors |
| 7 | OT Downtime Economics | Positioning consequence frame | $260K/hr mfg, $410K/hr energy, Norsk 6% insurance recovery |
| 3 | Cyber Insurance as Buying Driver | Actuarial ROI validation | SANS ICS 5 Controls, 78.07% combined risk reduction, $31.1B market |
| 8 | OT ICS Buying Triggers | Go-to-market sequencing | 11 triggers, CISO consolidation 16%→52%, 15.1% CAGR |
| 6 | OT Asset Inventory as Business Problem | Capability business case reframe | 12.6% full kill chain visibility, 6 business decisions |
| 5 | MSSP Renewal Risk | ARR protection through signal quality | 58%/94% split, false positive rate as Prism quality metric |
| 2 | Compliance as Evidence | Compliance delivery mechanism | CIP-015-1, TSA directives, 21% claim denial |
| 10 | What Buyers Actually Value | CISO career protection frame | 78% personal liability, 2% firm-wide resilience |
| 1 | Board and CISO Metrics | FAIR/CRQ data quality upstream | 93% board concern, 41% can't calculate ROI |
