# Research: NERC CIP Support for Prism (Day-2 Vision SIDE-ANALYSIS item C20)

- **Type:** general (technology / regulatory-compliance research)
- **Date:** 2026-06-27
- **Mode:** CAPTURE/research only (`do_not_execute`). This document modifies no live spec/BC/ADR/STATE.md/SESSION-HANDOFF.md. It is a cited research artifact feeding the C20 day-2 design decision.
- **Topic slug:** `nerc-cip-support`
- **Author:** research-agent
- **Status:** complete

> **Sourcing note.** Findings below are grounded in three Perplexity `sonar-deep-research` (PRIMARY) deep-research passes (CIP standard-by-standard mapping; BCSI + cloud; CIP-010/013 + audit-evidence), each returning ~80-103K characters of synthesized, citation-backed analysis of official NERC/FERC sources. Where a date or docket could not be independently double-confirmed in the retrieved corpus, it is flagged INFERRED or UNCONFIRMED. NERC standard versions and effective dates change; everything here is "as of 2026-06-27." Where dollar-precision matters (an exact effective date driving an architecture gate), the human/architect should confirm against the live NERC standard page before it is treated as load-bearing.

---

## 0. Executive Summary (the answer up front)

"Supporting NERC CIP" for Prism is **not** a checkbox — it is a layered posture. Three distinct facts drive the whole analysis:

1. **Prism is almost certainly an in-scope CIP asset, not a neutral observer.** A federated query/monitoring tool with read access into BES Cyber Systems, that aggregates configs/topology/logs, will be classified under CIP-002 as at minimum a **system that handles BCSI** (CIP-011), very plausibly an **EACMS** (Electronic Access Control or Monitoring System) under CIP-005/CIP-007, and — if it can initiate config changes or remote sessions — potentially part of a **BES Cyber System** carrying the full CIP weight. The architecture choices Prism makes (passive-only vs. write-capable; central-vs-edge data flow) directly set which CIP standards bind it.

2. **The single most decision-critical regulatory fact for the day-2 roadmap is the BCSI cloud pivot.** As of **January 1, 2024**, CIP-004-7 + CIP-011-3 (NERC Project 2019-02, FERC docket **RD21-6-000**) moved BCSI protection from a "designated storage location" model to a **provisioned-access + data-centric-encryption** model. This *explicitly enables* encrypted cloud / third-party storage of BCSI **provided the registered entity holds the keys and the provider has zero plaintext access**. This is the regulatory blessing for a BYOC / zero-access central plane — and it is exactly the architecture C16 (masking/BCSI) and C2 (satellite mesh) need.

3. **Running the actual BES Cyber System / EACMS in public cloud is still NOT settled.** That is the subject of NERC **Project 2023-09 (Risk Management for Third-Party Cloud Services)**, which is in flight and **not yet enforceable** as of 2026. So Prism's central/SaaS plane may *hold encrypted BCSI* today, but a Prism edge node that is itself classified as an EACMS generally **cannot** live in public cloud under current standards without inviting 10-20 CIP requirement violations.

**LEANS (full detail in §10):** Prism should target a **"CIP-deployable + CIP-evidence-generating"** posture, not "CIP-certified" (there is no such certification for a tool — compliance attaches to the *registered entity*, the tool *supports* it). The canonical term for C16 is **BCSI (BES Cyber System Information)**; the universal-name recommendation is to model the masking domain as **"Regulated Sensitive Information (RSI)" with BCSI as the first concrete profile** so the same machinery serves IEC-62443, PCI, ITAR, etc. Genuine human sub-forks: (a) do we build a first-class **CIP audit-evidence / RSAW-export module** (ties C10 GAP-Q2); (b) what is our **cloud-BCSI stance** — do we commit to entity-held-key zero-access as a hard architectural invariant.

---

## 1. Regulatory framing — who NERC/FERC are and what "support" means

NERC is the FERC-designated Electric Reliability Organization for North America; it authors mandatory reliability standards for the Bulk Electric System (BES) and FERC approves/remands them. The CIP (Critical Infrastructure Protection) family runs CIP-002 through CIP-013+ and is *enforceable* — violations carry findings and monetary penalties against the **registered entity** (utility/operator), not against a tool vendor. [perplexity-research pass 1; pass 3 §1.1]

Two crucial consequences for Prism:

- **There is no "NERC CIP certification" for software.** A vendor cannot be "CIP certified." Compliance is the registered entity's obligation; a tool is either (a) a CIP-in-scope asset that must itself meet applicable requirements, and/or (b) an enabler that produces evidence the entity uses to demonstrate its own compliance. "Prism supports NERC CIP" must be read as both senses.
- **Each CIP standard carries a version + effective date + (often) an inactive date.** FERC orders set a "regulatory order effective date"; the standard then has a separate compliance "effective date." Auditing happens against the version enforceable in the audit period. [pass 3 §1.1]

### Enforceable-version snapshot (as of 2026-06-27)

| Standard | Subject | Current enforceable version | Effective date | Notes / pending |
|---|---|---|---|---|
| CIP-002 | BES Cyber System categorization (High/Med/Low) | **CIP-002-5.1a** | in force several years; stable | No CIP-002-6 superseding found. [pass 1] |
| CIP-003 | Security Management Controls (+ Low-impact) | **CIP-003-9** | **April 1, 2026** | Filed Dec 2022, FERC order Mar 16 2023. [pass 1] |
| CIP-004 | Personnel & Training / Access Mgmt / **BCSI access** | **CIP-004-7** | **January 1, 2024** | Project 2019-02; FERC docket RD21-6-000. Adds BCSI-access R6. [pass 2 §CIP-004-7] |
| CIP-005 | Electronic Security Perimeter / remote access | **CIP-005-7** | **October 1, 2022** | Filed Dec 14 2020, FERC order Mar 18 2021. [pass 1] |
| CIP-007 | System Security Mgmt (patching, ports, logging) | **CIP-007-6** | **July 1, 2016** | Inactive date Jun 30 2028 (CIP-007-7 expected ~2028). [pass 1] |
| CIP-008 | Incident reporting & response | **CIP-008-7** | in force | CIP-008-7.1 (docket RM24-8-000) approved, effective **July 1, 2028**. [pass 1] |
| CIP-009 | Recovery plans for BES Cyber Systems | **CIP-009-6** | **July 1, 2016** | Inactive date Jun 30 2028. [pass 1] |
| CIP-010 | Config change mgmt + vuln assessment | **CIP-010-4** | **October 1, 2022** | Inactive date **04/24/2026**; CIP-010-5 effective **July 1, 2028** (FERC order Mar 19 2026). Transition window Apr 2026→Jul 2028 is UNCONFIRMED in detail — verify against NERC implementation plan. [pass 3 §1.2] |
| CIP-011 | Information protection — **BCSI** | **CIP-011-3** | **January 1, 2024** (aligned w/ CIP-004-7; INFERRED-but-strongly-supported) | Project 2019-02, docket RD21-6-000, FERC letter order accession 20211207-3062. [pass 2 §CIP-011-3] |
| CIP-013 | Supply-chain risk management | **CIP-013-2** | **October 1, 2022** | CIP-013-3 filed Jul 10 2024, FERC order Mar 19 2026; compliance effective date UNCONFIRMED (likely 2028). [pass 3 §1.3] |

---

## 2. Q1 — Which CIP standards constrain a federated query/monitoring platform

The deep-research synthesis sorts the CIP standards into **directly constraining** vs. **operator-program-obligations-Prism-supports**. [pass 1, throughout]

### Directly constraining Prism's architecture/behavior

- **CIP-002 (pivotal, indirect).** Does not prescribe controls but *decides Prism's regulatory weight.* Key determinations:
  - If Prism can initiate config changes / remote sessions / automated remediation → likely a **BES Cyber Asset** within a BES Cyber System → full CIP suite at its impact level.
  - If Prism enforces or monitors electronic access at a perimeter → likely an **EACMS** → CIP-005 + CIP-007 + CIP-010 obligations apply to Prism itself.
  - If Prism is strictly passive read-only telemetry with no control path → may be categorized as supporting infra *not* meeting BES Cyber Asset criteria — **but still subject to CIP-011 (BCSI) and possibly CIP-005 Low-impact**.
  - "Read-only" is **not** a safe harbor: CIP-002 uses *reasonable foreseeability* — if compromising Prism could mislead operators (disabled alarms, falsified status) and thereby affect reliability, regulators can pull it into a BES Cyber System. [pass 1 §CIP-002]
- **CIP-005 (most directly constraining for connectivity).** Prism crosses ESP boundaries to collect data. It must either sit *inside* an ESP (then all external access to Prism routes through Electronic Access Points) or interface *through* EAPs with documented, authenticated, logged connections. Any **interactive remote access** feature (launching SSH/RDP into BES assets, remote investigation) MUST go through the entity's IRA controls — encryption, MFA, intermediate systems — and **cannot bypass the ESP**. If Prism implements gateway/access-control functions it is an EACMS held to BCS-equivalent protection. [pass 1 §CIP-005]
- **CIP-007 (directly governs Prism when it is BCS/EACMS; shapes its features always).** Patch management, ports/services minimization, malicious-code prevention, secure config baseline, account mgmt, **and security event logging** all apply to Prism-as-asset. Separately, Prism-as-log-aggregator must ingest/retain/review the event categories CIP-007 mandates (logons, privilege changes, config changes, malware events). CIP-007-6 R4 logging is a direct driver of Prism's audit-trail design. [pass 1 §CIP-007; pass 3 §4.4]
- **CIP-011 (directly applies the moment Prism touches BCSI — which it will).** Configs, topology, baselines, vuln results, detailed logs aggregated by Prism are BCSI. See §3. [pass 2]

### Operator-program obligations Prism *supports* (indirect)

- **CIP-003** — governance/policy + Low-impact attachments. Prism must *support* policy enforcement: fine-grained RBAC, audit of user actions, identity-system integration, per-site/per-impact-level segregation. [pass 1 §CIP-003]
- **CIP-004** — personnel risk assessment, training, access management. Prism must let the entity treat *access to Prism* as logically equivalent to access to the systems it monitors: robust authN, RBAC, account lifecycle, action auditing. The **BCSI-access pivot (R6)** is the part that does bind Prism's data layer (see §3). [pass 1 §CIP-004; pass 2]
- **CIP-008** — incident reporting/response. Prism is a *primary enabler*: detection, correlation, alerting, forensic timeline reconstruction, case/ticket integration, export for lessons-learned. Not directly regulated, but practically necessary. [pass 1 §CIP-008]
- **CIP-009** — recovery plans. See §5 (Q4). Prism both needs its own recovery plan if it's a BCS/EACMS, and supports the entity's recovery evidence. [pass 1 §CIP-009]
- **CIP-010** — config change mgmt + vuln assessment. See §6. Prism can *be* the asset-management-system-of-record producing baselines + change detection + vuln-assessment cadence evidence. [pass 3 §2]
- **CIP-013** — supply chain. Prism (as a vendor product) becomes *subject of* the operator's CIP-013 program. See §8. [pass 3 §3]

---

## 3. Q2 — BCSI: definition, the 2022/2024 revisions, and protection requirements

### Canonical definition

**BES Cyber System Information (BCSI)** is a defined term in the *NERC Glossary of Terms Used in NERC Reliability Standards*: information **about** BES Cyber Systems that **could be used to gain unauthorized access to, or pose a security threat to,** those systems. Sensitivity arises from *exploitability*, not the raw content. [pass 2 §"NERC Glossary definition"; sources cited: NERCipedia glossary entry, NERC Project 2016-02 technical rationale, law-firm-compiled Glossary copy]

**Counts as BCSI** (examples from official rationale): security procedures/runbooks specific to BES Cyber Systems; security configurations (firewall rules, ACLs, account privileges, auth params) for BCS/PACS/EACMS; **network topology diagrams**; **aggregated, role-labeled collections of network addresses**; shared-cyber-infrastructure config (virtualization, central identity, central logging) that supports BCS; PACS/EACMS configs.

**Explicitly excluded** (NERCipedia + Glossary): isolated data points lacking exploitability — a single device name, a lone IP address without context, an ESP *name* without connectivity detail, generic high-level policy statements ("use strong passwords"). The boundary is **risk/aggregation-based**: a labeled, role-mapped IP table *is* BCSI even though a single IP is not. [pass 2 §"What counts" + §"exclusions" + summary table]

### What the 2019-02 revisions (CIP-004-7 / CIP-011-3) changed — the pivot

- **NERC Project 2019-02 "BES Cyber System Information Access Management."** FERC docket **RD21-6-000**, letter order accession **20211207-3062** (Dec 7 2021). CIP-004-7 effective **January 1, 2024**; CIP-011-3 approved in the same package (effective date aligned, INFERRED Jan 1 2024). [pass 2 §"FERC Approval"]
- **The conceptual shift: "designated storage locations" → "provisioned access + data-centric controls."** Old CIP-011-2 required identifying/securing the specific servers/shares where BCSI lived. New model: protect BCSI **wherever it resides** via **file-level encryption + permissions + access management**, and treat BCSI access as a *provisioned privilege* (authorize, review, revoke — like BCS access itself) under new **CIP-004-7 R6**, satisfied alongside **CIP-011-3 R1** (information protection program). [pass 2 §"From Designated Storage Locations to Provisioned Access"]
- **Net effect: encrypted cloud / third-party storage of BCSI is now explicitly permissible** when controls (access mgmt + encryption + entity-held keys) are in place. This is the regulatory door Prism's central plane walks through.

### How BCSI must be protected — at rest / in transit / in use

Driven by CIP-011-3 R1 and NERC's **2025 "Security Guideline: BCSI Cloud Encryption"** + the implementation guidance *"Usage of Cloud Solutions for BCSI"*: [pass 2 §"Protecting BCSI under CIP-011-3"]

- **At rest:** encrypted on provider infrastructure such that **the provider cannot read plaintext** — i.e., **entity-controlled keys** (BYOK/HYOK, client-side encryption, customer-managed keys). Provider-managed default encryption does **not** by itself satisfy CIP intent. This is the explicit **"zero-access" / "zero-knowledge"** model.
- **In transit:** TLS / secure tunneling / private connectivity; entity configures endpoints and manages/rotates protocol keys.
- **In use:** hardest case (plaintext is exposed during processing). BCSI should be decrypted **only on entity-controlled endpoints**, not within provider-managed services beyond storage/transport. Must not leak into logs, error messages, screenshots. Access gated by CIP-004-7 authorization.
- **Key custody is the linchpin:** keys must be generated/stored/rotated/destroyed under entity control, auditable, inaccessible to the CSP. This maps **directly** onto a **bring-your-own-cloud (BYOC) zero-access** architecture — relevant to C2 satellite mesh and C16.

---

## 4. Q3 — Access control & logging (CIP-004 / 005 / 007) → C18 RBAC + audit trail

[pass 1 §§CIP-004/005/007; pass 3 §4]

- **Role-based access + least privilege:** CIP-004-7 requires authorize/verify/revoke of access (R4/R5) and now BCSI access as a provisioned privilege (R6). Prism must enforce **fine-grained RBAC at the data-category level** (who can see which BCSI: which sensor's configs, which tenant's topology) — not just coarse app-level login. This is the regulatory teeth behind **C18 RBAC depth** and synthesizes with **C16 masking** (RBAC decides who sees unmasked BCSI).
- **Authentication:** CIP-005 IRA requires MFA + encryption for interactive remote access into BES assets; Prism's own auth and any remote-session feature must meet this.
- **Audit logging / monitoring of access:** CIP-007-6 R4 mandates logging of logons (success/fail), privilege changes, config changes, detected malicious activity, plus periodic review. Prism's **audit trail** must (a) cover the CIP-007 event categories for systems it monitors, AND (b) log access *to Prism itself / to BCSI within Prism* (who viewed/queried/exported BCSI) — the latter is the evidence operators need for CIP-004 access reviews. [pass 3 §4.3]
- **Synthesis with C18/C19:** per-tenant + per-impact-level + per-site segregation (CIP-003/CIP-002 boundaries) maps onto **C19 nested tenancy** — a Prism tenancy boundary should be expressible as a CIP impact-level / registered-entity / site boundary so access reviews and BCSI scoping line up with the entity's CIP categorization.

---

## 5. Q4 — Recovery (CIP-009) → C17 backup & recovery as first-class

[pass 1 §CIP-009; pass 3 §4.3]

- **CIP-009-6** (effective Jul 1 2016, inactive Jun 30 2028) requires **documented recovery plans** for BES Cyber Systems: backup strategy, restoration procedures, **integrity verification of backups**, roles/responsibilities, and **periodic recovery testing with retained evidence**.
- **Prism is implicated two ways:**
  1. If Prism is classified as a BCS/EACMS, **Prism itself needs a recovery plan** — restore config, restore audit/log data, restore integrations. C17 must therefore cover **Prism's own** recoverability (config-as-data, deterministic re-provision), not only data it manages.
  2. Prism **supports** the entity's CIP-009 program: track backup status/success, store config baselines, capture logs during recovery exercises, and — crucially — **compare post-recovery configuration against the CIP-010 baseline** to prove the recovery restored a known-good secure state. Recovery-test records are explicit audit evidence.
- **Synthesis with C17:** "backup & recovery as first-class" should mean (a) Prism's own state is backup/restore-testable with integrity verification, and (b) Prism *generates recovery-test evidence* (timestamped restore runs, baseline-diff after restore). Integrity verification of backups (CIP-009) pairs with signed/hashed snapshots — reuse the same crypto machinery as BCSI-at-rest and CIP-013 software-integrity.

---

## 6. Q5 — ESP / network & deployment (CIP-005) → C2 satellite mesh, air-gap, central-vs-edge

[pass 1 §CIP-005; pass 2 §"What is permitted vs pending"]

- **The ESP is the dominant network constraint.** Prism nodes that touch BES Cyber Systems must be inside an ESP or connect through documented EAPs. A federated mesh that spans control centers / substations / corporate IT must be reflected in the entity's **ESP diagrams and access lists** — Prism cannot be an "unregulated overlay." [pass 1 §CIP-005]
- **Satellite mesh (C2) implication:** each edge/satellite node is potentially an EACMS *inside* an ESP. Cross-ESP correlation must not create unauthorized access paths. **Passive-only collection (SPAN/mirror, one-way)** is the lowest-risk edge posture — it avoids creating a control path back into BES devices — but even passive devices must appear in ESP documentation and still expose BCSI (so CIP-011 still applies). [pass 1 §"Low Impact ESP / passive monitoring"]
- **Air-gap / data-diode friendliness:** the architecture should support a **one-way egress** edge mode (collect inside ESP, push normalized/encrypted data outward) so the mesh can sit behind a data diode in the most restrictive environments.
- **Can central (SaaS) ever touch BES data? — the pivotal answer:**
  - **BCSI in central/SaaS: YES, conditionally** — permitted under CIP-004-7/CIP-011-3 **if** the central plane stores BCSI only in **entity-key encrypted, zero-access** form (provider/Prism-SaaS cannot read plaintext). This is precisely the **BYOC zero-access** relevance the prompt flags. The central plane can index/route ciphertext + metadata; plaintext decryption happens only on entity-controlled endpoints.
  - **An actual BES Cyber System / EACMS running in central public cloud: NO (not settled).** Industrial Defender's widely-cited analysis: hosting Medium/High-impact BCS, EACMS, or PACS in public cloud would violate ~10-20 CIP requirements (physical access control, dedicated ownership, assured config control). CIP doesn't *name* the cloud or forbid it — but the current standards were written for on-prem entity-controlled systems. This is **Project 2023-09** territory (below). [pass 2 §"What is currently permitted vs pending"]
- **Design lean:** keep the *control-plane-as-EACMS* on-prem/edge inside ESPs; let the *central plane be a zero-access ciphertext + metadata aggregator* that never holds plaintext BCSI and never functions as an EACMS. That keeps the SaaS plane out of the unsettled Project 2023-09 zone while still delivering federated value.

---

## 7. Q6 — Cloud & CIP: current state (2024–2026) and what's pending

[pass 2 §"Beyond BCSI: NERC's Broader Cloud Services and CIP Modernization Efforts"]

| Use case | Status as of 2026 | Authority |
|---|---|---|
| **Storing BCSI in cloud** (encrypted, entity-held keys, zero-access) | **PERMITTED** | CIP-004-7 + CIP-011-3 (eff. Jan 1 2024); NERC 2025 BCSI Cloud Encryption guideline; "Usage of Cloud Solutions for BCSI" guidance. |
| Cloud for **ancillary** functions (analytics, non-BCSI logging, business apps) | Generally acceptable; must not expose BCSI or create new BCS attack paths; CSP may fall under CIP-013 supply chain | pass 2 §"permitted vs pending" |
| **Hosting Medium/High-impact BES Cyber Systems / EACMS / PACS in public cloud** | **PROBLEMATIC / effectively non-compliant** under current standards; not explicitly forbidden but would breach ~10-20 CIP requirements | Industrial Defender analysis; NERC *BES Operations in the Cloud* (2023 white paper) |
| Formal cloud-BES enablement | **PENDING** — NERC **Project 2023-09 "Risk Management for Third-Party Cloud Services"** (likely touches CIP-013 + others); draft/timeline still developing, exact clauses + effective dates **UNCONFIRMED** | pass 2 §"Project 2023-09" |

**Bottom line for Prism:** lean into the *permitted* lane (zero-access encrypted BCSI in a central plane) now; design so that if/when Project 2023-09 lands and broadens cloud-BES, Prism can extend — but do **not** architect on the assumption that a Prism EACMS-class node can live in public cloud today.

---

## 8. Q7 — Audit / evidence, plus CIP-010 & CIP-013 detail

### CIP-010 (config change mgmt + vuln assessment) — Prism as evidence engine

[pass 3 §2]

- **CIP-010-4** (eff. Oct 1 2022, inactive 04/24/2026); **CIP-010-5** approved, effective **July 1, 2028**. Transition window detail UNCONFIRMED — verify NERC implementation plan.
- **R1 baseline configurations:** CIP-010-4 defines a baseline as "a record in an asset management system" of required config items (OS/firmware, installed software+versions, patches, logical network interfaces/addresses, security-relevant settings, enabled/disabled services). **Prism can BE that asset-management-system-of-record** — capture full config snapshot at commissioning, tag as "CIP Baseline," retain prior baselines for audit history.
- **R1/R2 change management + unauthorized-change detection:** link planned changes to change tickets; on execution capture resulting config + update baseline; **continuously diff current config vs. baseline (hashing/param compare/topology analysis)** and flag any deviation without a matching approved change as a potential **unauthorized change** → alert + incident workflow. This is a natural Prism feature and produces a clean evidentiary trail.
- **R3 vulnerability assessment — paper OR active, ≥ once every 15 calendar months** (confirmed verbatim from CIP-010-4 RSAW). Prism can maintain advisory/patch repository correlated to asset inventory (paper), orchestrate scans (active), and **enforce the 15-month cadence** (track last-assessment-date per asset, alert before the window closes).
- **R4 control review / longer cadence (commonly ~36 months — UNCONFIRMED exact cadence in retrieved sources):** Prism can produce control-effectiveness reports (firewall logs prove access control enforced; patch logs prove timeliness; malware logs prove prevention works).

### CIP-013 (supply chain) — Prism-as-vendor becomes subject of operator's program

[pass 3 §3]

- **CIP-013-2** (eff. Oct 1 2022) is enforceable; **CIP-013-3** filed Jul 10 2024, FERC order Mar 19 2026, compliance effective date UNCONFIRMED (likely 2028).
- Operators must run new procurements of Prism through a documented supply-chain risk-management plan addressing: vendor incident notification; vendor remote/onsite **access revocation** (e.g., on staff termination); **vendor vulnerability disclosure**; **verification of software integrity & authenticity of all vendor software/patches** (CIP-013-2 explicitly); coordinated controls for vendor interactive + system-to-system remote access.
- **Vendor attributes that ease the operator's burden (and therefore should be Prism product commitments):**
  - **SBOM** for the Rust binary + dependency tree (lets the entity cross-ref CVEs) — synthesizes with Prism's existing security posture.
  - **Cryptographically signed releases + patches** with published hashes / secure channels — directly satisfies CIP-013-2 integrity/authenticity.
  - **Mature vulnerability-disclosure + timely-patch policy** (answerable against NATF's Electric Sector Supply Chain Risk Questionnaire, 200+ questions).
  - **No undisclosed / hard-coded remote access** — any remote/support/telemetry channel fully documented, configurable, and revocable. (Aligns with Prism's AI-opaque-credentials + ADR-022 wiring discipline.)

### Audit evidence & retention — what operators must produce

[pass 3 §4]

- **Audit instrument:** NERC **Reliability Standard Audit Worksheets (RSAWs)** — per-standard templates listing each requirement + the evidence auditors review. CIP-010-5 RSAW (updated 2024) has explicit "Audit Team Evidence Reviewed" sections → entities must produce **discrete, well-organized per-requirement artifacts** (process docs, logs, baseline records, assessment reports, change tickets).
- **Log retention:** CIP-007-6 requires event logs **retrievable for ≥ 90 days online**, plus **records of disposition** of logs beyond 90 days **up to the full evidence retention period** (typically multi-year). Prism must support **archive + retrieval for the full retention period**, with **tamper-evidence** (integrity + provenance metadata: origin, timestamps, processing) so logs are defensible in formal CEA proceedings.
- **Other evidence Prism can generate/support:** BCSI **access logs** (who viewed/queried/exported BCSI — for CIP-004 access reviews); recovery-test records with post-restore baseline diffs (CIP-009); vuln-assessment schedules + results (CIP-010 R3); control-effectiveness reports (CIP-010 R4); vendor remote-access session logs + software-integrity verification logs (CIP-013).
- **The product opportunity (ties C10 GAP-Q2 evidence package):** a **CIP audit-evidence export module** that emits per-requirement, RSAW-aligned evidence bundles (baseline records, change-detection trails, vuln cadence proof, access-review logs, log-retention attestations) is a high-value differentiator — and the audit-trail Prism already builds is most of the raw material.

---

## 9. Per-standard requirement map — what Prism must DO to "support NERC CIP"

| CIP std | Prism's relationship | Concrete Prism feature requirements |
|---|---|---|
| **CIP-002** | Determines Prism's classification | Configurable **deployment modes** (passive-read-only vs. write-capable vs. control); per-node impact-level tagging; documentation hooks so operators can place Prism in their categorization; **no hidden control paths** that silently escalate classification. |
| **CIP-003** | Supports governance | Policy-enforceable RBAC; CIP-Senior-Manager-approvable config; per-site/impact-level segregation; documented exception handling. |
| **CIP-004** | Supports personnel/access; **R6 binds data layer** | RBAC at **BCSI-category granularity**; account lifecycle (provision/review/revoke); identity-system integration; **BCSI-access logging** for access reviews. |
| **CIP-005** | **Directly constrains connectivity** | ESP-aware deployment; connect via EAPs with authN+logging; **IRA features must go through entity MFA/encryption, never bypass ESP**; support **passive/one-way/data-diode edge mode**; appear in ESP diagrams (export topology evidence). |
| **CIP-007** | Directly governs Prism-as-asset; shapes features | Patchable + port-minimized + hardened Prism; ingest/retain CIP-007 event categories; **≥90-day online log retrievability** + long-term archive; log review/alerting. |
| **CIP-008** | Enabler | Detection/correlation/alerting; case mgmt + ticket integration; forensic timeline export; lessons-learned export. |
| **CIP-009** | Prism needs own plan + supports entity | **Prism self-recovery** (config-as-data, integrity-verified backups, restore testing); capture recovery-exercise logs; **post-restore baseline diff** evidence. |
| **CIP-010** | Prism = evidence engine | Asset-mgmt-of-record baselines; change-ticket linkage; **continuous unauthorized-change detection**; vuln advisory correlation + scan orchestration; **15-month cadence enforcement**; control-effectiveness reports. |
| **CIP-011** | **Directly applies (BCSI)** | **Entity-key, zero-access encryption** at rest; TLS in transit; decrypt only on entity endpoints; **never leak BCSI to logs/errors**; BCSI classification engine (apply the aggregation-based definition); secure media disposal/retention. |
| **CIP-013** | Prism-as-vendor is subject of program | **SBOM**, **signed releases + published hashes**, vuln-disclosure policy, **no undisclosed remote access**, documented/revocable support channels, NATF-ESSCR-answerable posture. |

---

## 10. ANALYSIS + LEANS

### 10.1 What "support NERC CIP" should mean for Prism

Target posture: **"CIP-deployable + CIP-evidence-generating," explicitly NOT "CIP-certified."** There is no tool certification; compliance is the registered entity's. Prism's value proposition splits cleanly:

1. **Deployable-without-breaking-the-entity's-compliance** (CIP-002/005/007/011/013): Prism must be classifiable, ESP-fitting, hardenable, BCSI-protecting, and supply-chain-clean.
2. **Compliance-accelerating** (CIP-004/008/009/010 + audit evidence): Prism generates the artifacts operators otherwise assemble by hand.

### 10.2 How C20 synthesizes the related day-2 items

- **C16 (entity/data masking, "BCSI protection"):** CIP-011-3 is the regulatory spine. Masking = enforcing who sees unmasked BCSI, gated by CIP-004-7-style provisioned access. The crypto requirement (entity-key zero-access at rest, decrypt only on entity endpoints) makes C16 **not just field-masking but a key-custody + zero-access-encryption story**.
- **C17 (backup & recovery first-class):** CIP-009 makes this dual — Prism's *own* recoverability + Prism-generated recovery-test evidence with integrity-verified backups and post-restore baseline diffs. Reuse the BCSI/CIP-013 crypto machinery for backup integrity.
- **C18 (RBAC depth):** CIP-004/007 demand RBAC at BCSI-category granularity + full access auditing, not coarse app-login. C18 is the enforcement layer for C16.
- **C19 (nested tenancy):** map a tenancy boundary onto CIP impact-level / registered-entity / site boundaries so BCSI scoping and access reviews align with the operator's CIP-002 categorization.
- **C2 (satellite mesh) + OT dissection:** edge nodes are likely EACMS inside ESPs; favor passive/one-way collection + data-diode mode; central plane is a **zero-access ciphertext + metadata aggregator** that never holds plaintext BCSI — keeping it out of the unsettled Project 2023-09 cloud-BES zone while still federating. OT dissection at the edge is what produces the BCSI that must then be classified + encrypted before it leaves the ESP.

### 10.3 Canonical BCSI term + universal-name recommendation for C16

- **Canonical term confirmed: "BES Cyber System Information (BCSI)"** — a defined NERC Glossary term; "BCSI" is the correct, audit-recognized abbreviation. The user's example ("BCSI") is exactly right.
- **Universal-name recommendation:** BCSI is *electric-sector-specific*. Prism targets OT/critical-infra broadly (IEC-62443 already in scope; satellite, water, etc. plausible). **Model C16's masking domain around a sector-neutral abstraction** — recommend **"Regulated Sensitive Information (RSI)"** (or "Protected System Information / PSI") as the core type, with **BCSI as the first concrete *profile/policy pack*** alongside future profiles (IEC-62443 sensitive info, ITAR, PCI, etc.). This keeps the masking + key-custody + access-provisioning engine universal while letting BCSI rules (aggregation-based classification, entity-key zero-access, CIP-004-7 access semantics) ship as a pluggable compliance profile. *Do not* bake "BCSI" into type names / table names / API surface; bake in "RSI" + profile.

### 10.4 Genuine sub-forks requiring a HUMAN decision

These are decisions, not work-the-AI-can-just-do — surfaced per the production-grade default's "surface DECISIONS" boundary:

1. **Formal CIP audit-evidence / RSAW-export module — build it or not?** (Ties C10 GAP-Q2.) HIGH product value (differentiator; the raw material is the audit trail Prism already builds) but HIGH scope (per-requirement RSAW-aligned bundles for CIP-004/007/009/010/013, retention attestations, tamper-evidence). **Lean: yes, but phase it** — start with the evidence *substrate* (tamper-evident, long-retention, queryable audit trail with provenance) in the C18/C17 work, then a dedicated export module as its own later story. Human must confirm appetite + priority.
2. **Cloud-BCSI stance — commit to entity-held-key zero-access as a hard architectural invariant?** **Lean: yes** — make "central plane never holds plaintext BCSI; entity holds keys" a non-negotiable design invariant. This is the only currently-compliant cloud posture (CIP-011-3) and future-proofs against Project 2023-09. The fork is whether to *also* design now for the Project-2023-09 "EACMS-in-cloud" future (speculative, UNCONFIRMED timeline) — **lean: no, defer**; design the invariant, not the speculative extension.
3. **Prism's own CIP classification target.** Do we *design Prism to stay out of BES-Cyber-System classification* (strictly passive edge, no control path) to minimize the CIP weight on Prism itself, or do we embrace EACMS/BCS classification and meet the full suite? **Lean: design for the lighter classification by default** (passive-read-only edge, control features feature-flagged per existing Prism feature-flag model) **so operators choose the weight** — but human must ratify, because it constrains the write-capable sensor-API features already in the vision.
4. **CIP-010 asset-management-of-record positioning.** Do we position Prism as the operator's CIP-010 baseline system-of-record (big claim, big value, big audit-evidence surface) or as a feeder into their existing CMDB? **Lean: support both (be the record OR feed the record)**; human decides go-to-market emphasis.

### 10.5 Confidence & limitations

- **HIGH confidence:** BCSI definition + the 2019-02 provisioned-access pivot; CIP-004-7/CIP-011-3 effective Jan 1 2024 + docket RD21-6-000; zero-access entity-key encryption expectation; CIP-005 ESP/IRA constraints; CIP-007 90-day log retention; CIP-010 R3 15-month vuln cadence; CIP-013 software-integrity + no-undisclosed-remote-access; cloud-BES still unsettled (Project 2023-09).
- **MEDIUM / INFERRED:** CIP-011-3 exact effective date (aligned with CIP-004-7 = Jan 1 2024, strongly supported but not independently re-confirmed); CIP-003-9 April 1 2026 effective date.
- **UNCONFIRMED — verify against live NERC pages before treating as load-bearing:** CIP-010-4→CIP-010-5 transition behavior in the Apr 2026–Jul 2028 window; CIP-013-3 compliance effective date; CIP-010 R4 exact 36-month cadence; Project 2023-09 specific clauses + timeline. Standard versions/dates change — re-verify at design time.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | (1) CIP-002→CIP-013 standard-by-standard mapping onto a monitoring/query tool, with versions+effective dates; (2) BCSI definition + CIP-004-7/CIP-011-3 cloud pivot + zero-access encryption + Project 2023-09; (3) CIP-010 + CIP-013 + audit/RSAW/log-retention evidence. All `reasoning_effort: high`, `strip_thinking: true`. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | Not applicable — regulatory research, not library API docs. |
| Tavily (all variants) | 0 | Not used; three deep-research passes returned sufficient, citation-backed coverage of official NERC/FERC sources. |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | 1 area | Synthesis/structuring of findings + mapping to Prism's C16/C17/C18/C19/C2 day-2 items (the Prism-side mapping is model reasoning over the cited regulatory facts; the regulatory facts themselves are sourced). Flagged explicitly. |

**Total MCP tool calls:** 3 (all `perplexity_research`, the mandated PRIMARY tool).
**Training data reliance:** low — every regulatory claim (standard versions, effective dates, dockets, BCSI definition, encryption/cloud stance, evidence requirements) traces to the three Perplexity deep-research passes citing official NERC/FERC sources (NERC standard pages, NERC Glossary/NERCipedia, NERC Project 2016-02 technical rationale, FERC orders incl. docket RD21-6-000 / accession 20211207-3062, FERC 2022 CIP-audit lessons-learned staff report, NERC 2025 BCSI Cloud Encryption guideline, NERC *BES Operations in the Cloud* white paper, NATF supply-chain criteria, Industrial Defender & AssurX analyses). Model reasoning is confined to (a) structuring and (b) the Prism-side feature/synthesis mapping, both explicitly labeled. Inconclusive/unverifiable items are flagged INFERRED/UNCONFIRMED rather than asserted.

### Source corpus (as cited by the deep-research passes)
- NERC standard pages: CIP-002-5.1a, CIP-003-9, CIP-004-7/-8, CIP-005-7, CIP-007-6, CIP-008-7/-7.1, CIP-009-6, CIP-010-4/-5, CIP-011-3, CIP-013-2/-3.
- NERC *Glossary of Terms Used in NERC Reliability Standards*; NERCipedia BCSI entry; NERC Project 2016-02 / 2019-02 technical rationale documents.
- FERC orders: docket RD21-6-000 (CIP-004-7 + CIP-011-3), letter order accession 20211207-3062; FERC 2022 staff report on lessons learned from Commission-led CIP audits.
- NERC implementation guidance "Usage of Cloud Solutions for BCSI"; NERC 2025 "Security Guideline: BCSI Cloud Encryption"; NERC SITES "BES Operations in the Cloud" (Sept 2023) white paper; NERC virtualization/future-technologies draft white paper (Project 2016-02).
- NERC Reliability Standard Audit Worksheets (RSAWs) incl. CIP-010-4 and CIP-010-5 (2024-updated); NERC Rules of Procedure Appendix 4C.
- Industry analyses: Industrial Defender (cloud + CIP / BCSI), AssurX (CIP-013), NATF Electric Sector Supply Chain Risk Questionnaire (ESSCR).

> NERC standards and FERC orders evolve; all versions/dates are "as of 2026-06-27" and should be re-verified against the live NERC standard pages before any is treated as a load-bearing architecture gate.
