---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
topic_slug: nerc-project-2023-09-cloud-bes
scope: >
  C20 SF-2 deep-dive — NERC Project 2023-09 "Risk Management for Third-Party Cloud
  Services" current status + the cloud-BES future + the SF-2 design-now-vs-defer
  decision: should Prism design forward-scaffolding NOW so a Prism EACMS/BCS-class
  node could run in public cloud IF Project 2023-09 enables it, or DEFER and only
  design the entity-key zero-access invariant (already locked), extending later if/
  when the standard lands. CAPTURE/ANALYSIS ONLY. Modifies no live spec, ADR, BC,
  story, STATE.md, or SESSION-HANDOFF.md. No git operation. Single new file.
deepens: research/nerc-cip-support-2026-06-27.md (§6/§7 cloud + Project 2023-09; §10.4 fork 2)
informs: specs/day2-design-decisions/ADR-PROP-compliance-profiles.md (SF-2 open question)
honors_prior_decisions:
  - "Central plane = zero-access ciphertext+metadata aggregator; NEVER holds plaintext BCSI; is NOT an EACMS (C2/C16/C17/Option-3; INV-ADS-01/02)"
  - "EACMS-class nodes stay on-prem/edge inside ESPs today"
  - "P-ADS-11 single-codebase/deployment-profile; P-ADS-12 production-grade-default incl. 'defer entire speculative features' boundary; P-ADS-13 configurable-not-prescriptive"
  - "C20 entity-key zero-access cloud-BCSI invariant already locked (base research §10.4 fork 2 leaned: design the invariant, defer the speculative EACMS-in-cloud extension)"
---

# NERC Project 2023-09 / Cloud-BES Future — C20 SF-2 Deep-Dive (Design-Now vs Defer)

> **CAPTURE ONLY (`do_not_execute: true`).** Out-of-band day-2 side-analysis. Modifies no
> live artifact; performs no git operation. Deepens the Project-2023-09 / cloud-BES question
> the base research (`research/nerc-cip-support-2026-06-27.md` §6/§7, §10.4 fork 2) flagged
> UNCONFIRMED. Decision-oriented; built to feed the SF-2 open question in
> `ADR-PROP-compliance-profiles.md`.

> **Sourcing note.** Grounded in two Perplexity `sonar-deep-research` (PRIMARY) passes at
> `reasoning_effort: high` (~94K + ~79K chars of citation-backed synthesis): (1) Project
> 2023-09 status / SAR / FERC / timeline / related efforts; (2) cloud-provider OT/BES
> readiness + precedents + continuous-monitoring. NERC timelines change fast — every date is
> tagged CONFIRMED / INFERRED / UNCONFIRMED and "as of 2026-06-27." Items tagged UNCONFIRMED
> need live re-verification against nerc.com / ferc.gov before being treated as load-bearing.

---

## 0. Executive Summary + Lean

**The decision in one line: DEFER the speculative EACMS-in-cloud build; LOCK the zero-access
invariant (already done); and take a low-cost "leave-the-seam-open" middle path that costs
near-zero now and avoids the expensive retrofit later.** This refines the base research's
§10.4 fork-2 lean ("design the invariant, not the speculative extension") by identifying
*precisely which seams* are cheap to keep open vs expensive to retrofit, so "defer" does not
silently become "paint into a corner."

Three findings drive it:

1. **Project 2023-09 is real, active, and multi-year-away. It is NOT settled and has NO
   confirmed enforceable date.** CONFIRMED: SAR (filename-dated 2023-12-13), drafting-team
   roster (filename-dated 2024-07-31, chaired by Matt Hyatt, Georgia System Operations Corp),
   and a NERC Reliability Standards Development Plan (Aug 2025) estimating **drafting complete
   by December 2026**. Everything past "drafting complete" — ballot success, NERC Board
   adoption, FERC filing, FERC approval, implementation period — is UNCONFIRMED and, on
   historical CIP cadence (3–5 yr SAR→enforcement), projects to an enforceable date in the
   **2029–2030+** window. Betting Prism engineering on a 2029–2030+ standard whose *text does
   not yet exist publicly* is a textbook P-ADS-12 "defer the entire speculative feature"
   case, not a "ship it production-grade now" case.

2. **What it would enable is genuinely transformative IF it lands — but the gap today is
   enormous.** The widely-cited industry framing: hosting cloud BES Cyber Systems (BCS),
   EACMS, or PACS is "close to impossible" today because "it is highly unlikely that any CSP
   could ever produce the evidence required … to prove compliance with even a few of the
   **100+ currently applicable CIP Requirements and Requirement Parts**." (The base research's
   "~10-20 violations" figure was conservative; the deeper pass surfaces the larger "100+
   requirement-parts" framing — a *harder* gap, reinforcing defer.) Project 2023-09's
   risk-based, outcome-driven, CSP-attestation / shared-responsibility model is the thing that
   would close that gap — but only after it becomes enforceable text.

3. **Cloud providers are READY at the infrastructure tier; the regulation is the gate, not the
   technology.** AWS GovCloud (DoD IL4/IL5), Azure Government (FedRAMP High + IL4/IL5),
   Google Assured Workloads (FedRAMP High + DISA IL2/4/5) all hold the accreditations and have
   each published NERC-CIP-specific BCSI guidance + reference architectures. **But there is NO
   confirmed public precedent of any utility/ISO/RTO running a primary EMS/SCADA or an
   EACMS-class control system in public cloud.** Confirmed cloud adoption is BCSI storage +
   analytics + BES-adjacent IT — exactly the lane Prism's zero-access central plane already
   occupies. So the technology readiness does NOT change the "defer the EACMS-in-cloud build"
   conclusion; it only confirms that *when* the regulation lands, the substrate will exist.

**Net:** the EACMS-in-cloud feature is speculative, multi-year-distant, and bounded by an
unsettled standard — so under P-ADS-12 it is correctly deferred *as a feature*. But three
specific architectural seams are cheap to keep open today and expensive to retrofit later;
keeping them open is NOT "designing forward for a speculative feature," it is ordinary
production-grade hygiene that the locked zero-access invariant + the deployment-profile model
(P-ADS-11) already imply. **Refined recommendation: Sub-Option B — "Defer + Leave-Seam-Open"
(§7).**

---

## 1. Project 2023-09 status table (CONFIRMED vs SPECULATIVE)

| Item | Finding | Status | Source |
|---|---|---|---|
| Project exists / title | "Risk Management for Third-Party Cloud Services" | **CONFIRMED** | NERC project page (nerc.com) [P1-1] |
| Purpose / model | "risk-based, outcome-driven requirements that place cloud services on par with other third-party resources" | **CONFIRMED** (purpose statement) | NERC project page [P1-1] |
| SAR posted | SAR file `2023-09_risk_mgmt_for_3rd-party_cloud_services_sar_12132023.pdf` → dated **2023-12-13** | **CONFIRMED existence; date INFERRED from filename** | NERC SAR doc [P1-11] |
| SAR scope language | Proposes to "create a new standard(s) or revise existing CIP Standards to address the language that includes or implies specific" on-prem/control assumptions | **CONFIRMED (partial — snippet truncated)** | NERC SAR [P1-11] |
| Drafting team formed | DT roster `2023-09_dt_roster_07312024.pdf` → **2024-07-31**; chair **Matt Hyatt (Georgia System Operations Corp)** | **CONFIRMED (filename + chair)** | NERC DT roster [P1-3] |
| Standards in scope | Pass 2 states project "impacts multiple CIP standards, **CIP-002 through CIP-014**" (asset ID, governance, training, ESP, physical, system-sec, IR, recovery, config, info-protection, comms, supply-chain, physical-security) | **CONFIRMED (broad scope); specific per-standard edits UNCONFIRMED** | NERC project docs [P2-17] |
| New cloud-specific standard (e.g. "Cloud CIP" / a CIP-016) | SAR leaves open "new standard(s) OR revise existing"; analysts call the outcome "Cloud CIP" generically; **no official new CIP number confirmed** | **UNCONFIRMED designation** | Industrial Defender analysis [P1-18]; NERC SAR [P1-11] |
| CIP-015 relationship | **CIP-015-1 belongs to Project 2023-03 (Internal Network Security Monitoring / INSM), NOT 2023-09.** Do not conflate. | **CONFIRMED (distinct project)** | NERC Project 2023-03 page [P1-6] |
| CIP-004-7 / CIP-011-3 (BCSI cloud) | Already enforceable; FERC approved Dec 2021; the *existing* regulatory door for encrypted zero-access cloud BCSI | **CONFIRMED** (corroborates base research §3) | FERC approval + NERC guidance [P2-14][P2-18] |
| Posted drafts / ballots | Posting schedule mentions a "formal comment period and additional ballot … Week of June 29" (year not visible); no ballot %, no quorum data, no Board adoption found | **UNCONFIRMED (draft count, ballot results, adoption status)** | NERC posting schedule [P1-12] |
| Drafting completion estimate | NERC Reliability Standards Development Plan (**Aug 2025**) → "drafting estimated to be completed by **December 2026**" | **CONFIRMED (as a planning estimate)** | NERC RSDP Aug 2025 [P1-13] |
| Filing with FERC / FERC approval / enforceable date | None filed/approved; no enforceable date set | **UNCONFIRMED — none exist yet** | absence across [P1-1..14] |
| Projected enforceable date | Speculative chain: drafting done ~Dec 2026 → ballot/adopt 2027 → FERC file mid/late 2027 → FERC approve ~2028 → implementation → **enforceable ~2029–2030** | **SPECULATIVE (analytical projection on historical CIP cadence)** | derived [P1-13][P1-4][P1-7] |

### FERC involvement
- **CONFIRMED:** FERC issued a **Notice of Inquiry (NOI)** "into Virtualization, Cloud Services
  for Power Grid Operations" — four topic areas (scope of use, benefits/risks, impediments,
  emerging tech) — and **directed NERC to make an informational filing + quarterly updates**
  on "two draft CIP standards pertaining to virtualization and cloud computing." [P1-14]
- An NOI is **exploratory, not prescriptive.** FERC said it *will decide* whether to direct
  NERC to modify CIP based on the comments. **UNCONFIRMED:** the NOI's exact docket number/
  date, and whether any **NOPR or Order** specific to cloud-BES (or to Project 2023-09) has
  since issued. No RM-docket number could be confirmed. **[Needs live re-verification.]**

### Related efforts
- **Project 2016-02 (Modifications to CIP / Virtualization & Future Technologies):**
  CONFIRMED active; the virtualization-focused predecessor; likely the source of the "two draft
  CIP standards" FERC referenced. Relationship to 2023-09: 2016-02 = virtualization
  architecture clarifications; 2023-09 = the narrower, newer third-party-cloud-services
  risk-management layer. **[P1-10][P1-14]**
- **NERC SITES "BES Operations in the Cloud" white paper** (Security Integration and
  Technology Enablement Subcommittee): CONFIRMED exists; purpose "to further the assessment of
  securely conducting BES operations in the cloud." Exact publication date UNCONFIRMED in the
  corpus (base research cited Sept 2023). **[P1-9][P1-15]**
- **NERC "Security Guideline — Primer for Cloud and BCSI Protection"** + **"BCSI Cloud
  Encryption" guideline:** CONFIRMED; explicitly reference FedRAMP Low/Moderate/High as a
  CSP-attestation input and lay out encryption / key-management / segregation-of-duties
  expectations. This is the existing, *already-blessed* zero-access-BCSI lane. **[P1-8][P2-18]**
- **CIP-013 supply-chain direction:** third-party CSPs fall under the registered entity's
  CIP-013 supply-chain program; 2023-09 is expected to sharpen CSP-as-vendor obligations.
  CONFIRMED relevance; specific 2023-09 edits UNCONFIRMED. **[P1-7][P1-8]**

---

## 2. Enablement analysis — what changes IF/WHEN 2023-09 lands

**Today (CONFIRMED baseline):**
- **PERMITTED:** encrypted, entity-key, zero-access cloud **BCSI** storage + analytics +
  BES-adjacent IT (CIP-004-7 / CIP-011-3, eff. Jan 1 2024). This is exactly Prism's locked
  central-plane posture.
- **EFFECTIVELY NON-COMPLIANT:** a cloud-hosted **BCS / EACMS / PACS** for Medium/High-impact
  assets — because no CSP can produce the evidence for the **100+ applicable CIP requirement
  parts** that presuppose entity-owned, physically-controlled, dedicated infrastructure.
  [P1-18]

**If Project 2023-09 becomes enforceable (SPECULATIVE, ~2029–2030+):**
- BCS/EACMS/PACS in public cloud could become a **risk-based, outcome-driven** option, where
  the registered entity demonstrates compliance through a **shared-responsibility split** +
  **CSP attestation** (FedRAMP-style authorizations, SOC reports, contractual evidence)
  rather than entity-produced physical/configuration evidence. [P1-18][P2-14..18]
- New CIP **Purpose statements** and **new applicable asset types** (e.g. "cloud service
  asset" categories under a revised CIP-002) would be introduced. [P1-18]
- The "~100+ requirement-part impossibility" collapses to "a documented shared-responsibility
  matrix + CSP attestation + entity-side controls." That is the precise moment a Prism
  EACMS-class node in public cloud becomes a *legal* deployment, not just a *technical* one.

**Caveat (production-grade honesty):** even the enablement model is INFERRED from NERC
guidance + analyst commentary, not from posted draft standard text — which does not exist
publicly yet. The *shape* (risk-based + CSP-attestation + shared-responsibility) is
well-supported; the *details* (which requirement parts delegate to the CSP, what attestation
suffices) are UNCONFIRMED and will materially affect any forward design. **This is the core
reason designing the full feature now is premature: you would be designing against a contract
that has not been written.**

---

## 3. Cloud-provider readiness table (OT/BES, as of 2026)

| Provider / offering | Confirmed accreditations | NERC-CIP-specific effort | OT/BES relevance | Status |
|---|---|---|---|---|
| **AWS GovCloud (US)** | DoD CC SRG **IL2/IL4/IL5** (IL6 in AWS Secret Region); FedRAMP Mod/High (many services) | **NERC CIP BCSI Compliance Guide** + **BCSI Reference Architecture** + **Operational Best Practices Conformance Pack** + 2 BCSI use cases | Strong for US critical-infra needing DoD alignment; BCSI + selected OT | **CONFIRMED** [P2-8][P2-14][P2-1][P2-9] |
| **AWS European Sovereign Cloud** | No critical dependency on non-EU personnel/infra; EU data-residency controls | (NERC is NA-only; positioned for EU critical-infra) | EU grid operators needing sovereignty | **CONFIRMED** [P2-5][P2-13] |
| **Azure Government (US Gov regions)** | FedRAMP **High** P-ATO; DoD **IL2/IL4/IL5**; JSIG PL3 | Azure **NERC CIP + cloud computing** guidance for registered entities | High relevance; BCSI + selected OT | **CONFIRMED** [P2-2][P2-15] |
| **Azure public (US)** | FedRAMP **High** P-ATO (JAB); DoD IL2 | same NERC guidance | BES-related info under FedRAMP High | **CONFIRMED** [P2-2] |
| **Azure sovereign / EU Data Boundary** | On-prem control-plane option; EU-only data processing for EU customers | — | Sovereign data processing for regulated infra | **CONFIRMED** [P2-6] |
| **Google Assured Workloads** | FedRAMP **Mod/High** (early hyperscale FedRAMP-High); personnel controls for FedRAMP-High | **Google Cloud & NERC CIP whitepaper** | Regulated workloads in commercial GCP w/ data boundaries | **CONFIRMED** [P2-3][P2-11][P2-16] |
| **Google Cloud (DISA-authorized)** | DISA **IL2/IL4/IL5** | same whitepaper | DoD-aligned BES workloads | **CONFIRMED** [P2-12] |
| **Google Sovereign Cloud (EU)** | Secure-by-design + EU data residency | — | EU critical-infra | **CONFIRMED** [P2-4] |
| **CMMC / StateRAMP / ITAR specifics** | Claimed in broader marketing; not in cited primary docs | — | — | **UNCONFIRMED — verify per offering** |

**Precedent reality check (the decisive row):**
- **CONFIRMED:** cloud **BCSI** storage + analytics is accepted practice; all three CSPs ship
  NERC-CIP BCSI guidance + reference architectures + conformance packs. OT vendors (e.g.
  Schneider EcoStruxure Power Operation) offer cloud-connected SCADA **monitoring/analytics**
  with primary control on-prem. IEC 62443 permits cloud under risk-based controls (no blanket
  ban). [P2-14..19]
- **UNCONFIRMED — and this is the crux:** **NO public evidence of any named utility, ISO, or
  RTO (PJM, MISO, ERCOT, CAISO, SPP) running a primary real-time EMS/SCADA or an EACMS-class
  control system in mainstream public cloud.** Adoption is BCSI / analytics / BES-adjacent IT
  — never the EACMS/BCS control tier. [P2 §"ISOs/RTOs"; §"Confirmed vs Unconfirmed"]

**Reading:** infrastructure readiness is NOT the bottleneck — the *regulation* is. A Prism
cloud-EACMS is technically buildable today on GovCloud/Azure-Gov/Assured-Workloads; it is the
CIP-compliance evidence model (pending 2023-09) that makes it unlawful for the registered
entity. So CSP readiness does not pull the decision toward "build now"; it only de-risks the
*later* build, reinforcing that the seam (not the feature) is what matters today.

---

## 4. The crux — seams to future-proof, and the cost of each path

The strategic question: which decisions are **cheap to keep open** (abstractions / seams) vs
**expensive to retrofit** (data-flow, key-custody, classification boundaries)? Mapped against
Prism's locked posture (central = zero-access ciphertext+metadata aggregator, NOT an EACMS;
EACMS-class nodes on-prem/edge inside ESPs).

| Seam | Retrofit cost if NOT kept open | Keep-open cost NOW | Verdict |
|---|---|---|---|
| **(S1) Node-role / classification as data, not topology** — "is this node an EACMS / BCS / aggregator?" expressed as a node attribute (rides C19 `regulatory_class` + the Compliance-Profile engine) rather than hardcoded "central=aggregator, edge=EACMS" | **HIGH** — if EACMS-ness is baked into where-code-runs, enabling a cloud-EACMS later means re-plumbing the deployment topology and re-deriving classification everywhere | **~ZERO** — the Compliance-Profile + deployment-profile two-axis model (P-ADS-11 + ADR-PROP-compliance-profiles D-PROF-5) **already** makes role/posture a data attribute. Keeping classification as data is hygiene Prism already does. | **KEEP OPEN (free)** |
| **(S2) Deployment-profile enum is open/extensible** — `saas / mssp-managed / byoc / air-gap` today; a future `cloud-eacms` or `cloud-bcs` profile is *addable* without a code fork | **MEDIUM-HIGH** — a closed/forked deployment model means a new regulated cloud topology is a new code branch (violates P-ADS-11/AP-ADS-07) | **~ZERO** — P-ADS-11 already mandates single-codebase + profile-as-data + `#[non_exhaustive]` enums. A new profile is a new data row, not a fork. | **KEEP OPEN (free — already implied by P-ADS-11)** |
| **(S3) CSP-attestation / shared-responsibility evidence as a first-class evidence type** — the audit-evidence substrate (base research §8 / C10 GAP-Q2) can carry a "control owned by CSP, evidenced by attestation X" provenance, not only "control owned by entity, evidenced by Prism log" | **MEDIUM** — if the evidence model assumes entity-owns-every-control, adding CSP-delegated controls later means reworking the provenance schema | **LOW** — make the audit-evidence provenance field model an *owner* dimension (`entity` / `csp` / `shared`) when that substrate is built anyway. One extra enum field. | **KEEP OPEN (low) — but only when the evidence substrate is being built; do NOT build the substrate early just for this** |
| **(D1) Plaintext BCSI never at central / entity holds keys** — the locked zero-access invariant (INV-ADS-01/02) | N/A — already locked; the *opposite* (letting plaintext into central) would be the expensive, possibly-irreversible mistake | already paid | **ALREADY LOCKED — keep** |
| **(D2) Actual cloud-EACMS data-flow + key-custody-in-cloud-HSM design** — re-routing real control/monitoring traffic through a cloud node, cloud-resident key custody, ESP boundary spanning into cloud | **HIGH (build cost) — but this is the FEATURE, not a seam.** Designing it now means designing against a non-existent standard; high rework risk when the real text differs | **HIGH** — full design-forward = speculative work on unsettled regulation; the §1 timeline (no text, ~2029-2030 enforceable) makes this premature | **DEFER (do NOT build) — P-ADS-12 "defer the entire speculative feature"** |
| **(D3) Classification-boundary semantics that assume EACMS == on-prem** | **HIGH** if hardcoded — but (S1) already neutralizes this by making classification data-driven | covered by S1 | **Covered by S1 — keep S1 open, do not hardcode the assumption** |

**The pattern:** every *cheap* seam (S1, S2, S3, D1) is already implied by Prism's existing
locked principles — single-codebase/deployment-profile (P-ADS-11), profile-as-data
(P-ADS-13 / ADR-PROP-compliance-profiles), zero-access invariant (INV-ADS-01/02), and the
planned audit-evidence substrate. The only *expensive* thing (D2, the real cloud-EACMS
data-flow + key custody) is the speculative feature that P-ADS-12 explicitly says to defer.

**There IS a low-cost middle path** (between full design-forward and pure defer): do nothing
*extra* beyond ensuring the existing seams (S1/S2) are genuinely data-driven and the
`#[non_exhaustive]` discipline holds, and add the one-line evidence-owner dimension (S3) *if
and when* the audit-evidence substrate is built for independent reasons. This costs ~zero
incremental engineering and is indistinguishable from ordinary production-grade hygiene under
the principles already in force.

---

## 5. Risk framing

| Risk | Over-build (full design-forward NOW) | Under-build (pure defer, ignore seams) | Defer + Leave-Seam-Open (recommended) |
|---|---|---|---|
| **Betting on unsettled standard** | HIGH — designing data-flow/key-custody against draft text that does not exist; ~2029-2030 enforceable; high probability the real standard differs → rework | LOW | LOW — no speculative feature built |
| **Reversibility** | LOW — speculative architecture is sticky; hard to unwind cleanly | N/A | HIGH — seams are inert until a profile is added; nothing to unwind |
| **Wasted engineering** | HIGH — full cloud-EACMS subsystem may never be needed, or needed in a different shape | none | minimal — seams ride existing principles |
| **Corner-painting (the real under-build danger)** | none | **MEDIUM-HIGH** — if classification/topology gets hardcoded (EACMS==on-prem), the eventual retrofit is expensive (S1/S3 retrofit) | avoided — seams kept open |
| **P-ADS-12 conformance** | **VIOLATES** the "defer entire speculative features" boundary by building a partial speculative feature | conforms on defer; **risks** the "no partial/shortcut that needs cleanup" rule via hidden hardcoding | **conforms** — defers the feature, keeps no shortcuts, no partial build |
| **Competitive / market** | small upside (first-mover claim) vs high cost | risk of "we'd have to re-architect" if a client demands it in 2029 | preserves optionality at ~zero cost |

**Asymmetry:** over-building is expensive AND low-reversibility AND P-ADS-12-violating;
under-building's only real danger is corner-painting, which the cheap seams (S1/S2/S3) fully
neutralize. The risk-minimizing choice is the middle path.

---

## 6. ANALYSIS

1. **The base research's §10.4 fork-2 lean ("design the invariant, defer the speculative
   EACMS-in-cloud extension") holds and is *strengthened* by this deeper pass.** The deeper
   evidence makes the gap *larger* ("100+ requirement parts," not "10-20 violations") and the
   timeline *more distant and more uncertain* (no public draft text; ~2029-2030 enforceable;
   FERC still at NOI stage). Both factors push harder toward defer.

2. **"Defer" must not be allowed to silently become "paint into a corner."** The genuine value
   this deep-dive adds beyond the base research is the seam-vs-feature decomposition (§4):
   identifying that S1/S2/S3 are cheap, already-implied seams and D2 is the expensive
   speculative feature. A naive "defer" that hardcodes EACMS==on-prem would incur the exact
   expensive retrofit (S1/S3) the analysis warns about. The middle path prevents that.

3. **The locked principles already do most of the work.** P-ADS-11 (single-codebase /
   deployment-profile, no forks), P-ADS-13 + ADR-PROP-compliance-profiles (posture as
   tighten-only data, `regulatory_class` as a profile selector/floor, two distinct
   deployment-vs-compliance axes), and INV-ADS-01/02 (zero-access) collectively mean Prism is
   *already* built so that "node role / classification is data" and "a new regulated cloud
   topology is a new profile, not a fork." The SF-2 "leave-seam-open" path is therefore mostly
   a *conformance check* on existing discipline, not new design work.

4. **CSP readiness changes nothing about the lean — it only de-risks the eventual build.**
   GovCloud/Azure-Gov/Assured-Workloads having IL4/IL5 + FedRAMP-High + NERC-CIP guides means
   *when* 2023-09 lands, the substrate exists. It does not make a cloud-EACMS lawful today
   (no precedent; 100+ requirement-part gap). So it does not pull toward building now.

5. **One honest counter-argument (surfaced, then weighed):** if Prism's go-to-market in
   2027-2029 targets utilities that *want* to be early cloud-BES adopters the moment 2023-09
   lands, a first-mover EACMS-in-cloud capability could be a differentiator. **Weighing:** this
   is a *business-priority/market-timing DECISION* (legitimately human's per P-ADS-12), not an
   engineering necessity. The middle path preserves the option to build it fast later (seams
   open) without paying for it now. If the human judges the market window worth pre-investment,
   that is a deliberate scope decision — but the *default* under P-ADS-12 is defer-the-feature.

---

## 7. LEANS — refined recommendation + sub-options for SF-2

**REFINED RECOMMENDATION: Sub-Option B — "Defer the feature, leave the seams open."**

> Keep the entity-key zero-access cloud-BCSI invariant locked (done). Do NOT build any
> cloud-EACMS/BCS data-flow, cloud key-custody, or ESP-spanning-into-cloud design now (defer
> the speculative feature per P-ADS-12). DO ensure the three cheap seams stay open — which is
> mostly verifying existing principles already keep them open:
> - **S1:** node role / CIP-classification (EACMS / BCS / aggregator) stays a **data attribute**
>   on the C19 tenant-tree / Compliance-Profile engine, never a hardcoded property of where
>   code runs. (Already implied by D-PROF-5/6; confirm no "EACMS==on-prem" hardcode creeps in.)
> - **S2:** the deployment-profile enum stays **open/`#[non_exhaustive]`** so a future
>   `cloud-eacms` profile is an addable data row, not a code fork. (Already mandated by
>   P-ADS-11 / AP-ADS-07; confirm.)
> - **S3:** *when* the audit-evidence substrate (C10 GAP-Q2) is built for independent reasons,
>   give its provenance model an **evidence-owner dimension** (`entity / csp / shared`). Do NOT
>   build the substrate early just for this — only carry the one-field seam when it's built.

The three sub-options, ordered by lean:

- **Sub-Option B (RECOMMENDED) — Defer + Leave-Seam-Open.** Cost ~zero now; preserves full
  optionality; P-ADS-12-conforming; neutralizes the only real under-build risk
  (corner-painting). This is the production-grade default.

- **Sub-Option A — Pure Defer (no explicit seam guarantee).** Acceptable *only* because
  Prism's existing principles (P-ADS-11/13, INV-ADS-01/02) already keep S1/S2 open implicitly.
  The risk is that without an explicit note, a future implementer hardcodes EACMS==on-prem and
  incurs the S1/S3 retrofit. Inferior to B at no cost saving — B is just "A + a conformance
  note." **Lean: choose B over A; they cost the same and B is safer.**

- **Sub-Option C — Design-Forward Now (full cloud-EACMS scaffolding).** NOT recommended as a
  default. Violates the P-ADS-12 "defer entire speculative features" boundary; designs against
  non-existent standard text; high rework risk; low reversibility. **Only** justified if the
  human makes an explicit market-timing decision that a 2027-2029 first-mover cloud-BES
  capability is a business priority worth pre-investment — a DECISION to surface, not a default
  to take.

**Decision owner:** the seam-conformance (B) is AI-executable production-grade hygiene at morph
(no human decision needed — it's just "don't hardcode, keep enums open"). The *only* thing
needing a human DECISION is whether to escalate to Sub-Option C for market-timing reasons; the
default answer under the Canonical Principle is "no — defer the feature, keep the seams open."

**One open item to re-verify live before morph (UNCONFIRMED, fast-moving):** the current
Project 2023-09 milestone past Aug-2025 RSDP — has any draft been balloted/adopted, has NERC
filed with FERC, has FERC issued a NOPR/Order or an RM-docket? Re-check nerc.com Project
2023-09 page + ferc.gov before any SF-2 decision is treated as load-bearing. If the timeline
has *accelerated* materially (e.g. FERC NOPR issued, enforceable date set < 2 yr out), revisit
whether Sub-Option C's market-timing case strengthens.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) Project 2023-09 status — SAR, drafting-team, posted drafts/ballots, FERC NOI/dockets, projected timeline/enforceable date, related efforts (2016-02, SITES white paper, CIP-013). (2) Cloud-provider OT/BES readiness — GovCloud/Azure-Gov/Assured-Workloads accreditations, CSP NERC-CIP guidance, real utility/ISO/RTO precedents, continuous-monitoring. Both `reasoning_effort: high`, `strip_thinking: true`. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | N/A — regulatory/architecture research, not library API docs. |
| Tavily (all variants) | 0 | Not used; two deep-research passes returned sufficient citation-backed coverage of NERC/FERC/CSP sources. |
| WebFetch / WebSearch | 0 | — |
| Training data | 1 area | Synthesis: the seam-vs-feature decomposition (§4), cost/risk framing (§4/§5), and mapping to Prism's locked principles (P-ADS-11/12/13, INV-ADS-01/02, ADR-PROP-compliance-profiles D-PROF-5/6) — model reasoning over (a) the cited regulatory/CSP facts and (b) the in-repo day-2 artifacts I read directly. Flagged explicitly; the regulatory/CSP facts themselves are sourced. |

**Total MCP tool calls:** 2 (both `perplexity_research`, the mandated PRIMARY tool).
**Training data reliance:** low — every regulatory and cloud-provider claim traces to the two
Perplexity deep-research passes citing NERC project pages, the NERC SAR + DT roster + Aug-2025
Reliability Standards Development Plan + posting schedule, NERC SITES/BCSI guidelines, the FERC
NOI announcement, AWS/Azure/Google compliance documentation, FedRAMP/DoD SRG, and industry
analyses (Industrial Defender). The in-repo Prism principle/decision references were read
directly from `ARCHITECTURE-DESIGN-SYSTEM.md` and `ADR-PROP-compliance-profiles.md`. Model
reasoning is confined to the Prism-side synthesis (seams, costs, risk, leans), all labeled.

### Honesty & coverage note (explicit flags — NERC timelines change fast)
- **[UNCONFIRMED — re-verify live]:** Project 2023-09 milestone status *after* Aug-2025 RSDP
  (draft count, ballot results, quorum, NERC Board adoption); FERC NOI exact docket/date; any
  FERC NOPR/Order/RM-docket specific to cloud-BES; CIP-013-3 interplay with 2023-09; any new
  cloud-specific CIP number (no official "CIP-016"/"Cloud CIP" designation confirmed).
- **[INFERRED]:** SAR date (2023-12-13, from filename); the ~2029-2030 enforceable date
  (analytical projection on historical CIP cadence, NOT a published schedule); the
  shared-responsibility/CSP-attestation enablement *details* (shape is supported; specifics
  await draft text).
- **[CONFIRMED]:** project existence + purpose + multi-CIP scope (CIP-002..014); DT roster
  (2024-07-31, chair Matt Hyatt); Aug-2025 RSDP "drafting complete by Dec 2026"; FERC NOI +
  informational-filing directive exists; CIP-015 belongs to Project 2023-03 (INSM), not
  2023-09; CIP-004-7/CIP-011-3 cloud-BCSI door (eff. Jan 1 2024); CSP accreditations
  (GovCloud IL4/5, Azure-Gov FedRAMP-High+IL4/5, Google Assured Workloads FedRAMP-High +
  DISA IL2/4/5); all three CSPs publish NERC-CIP BCSI guidance; NO confirmed public precedent
  of a primary EMS/SCADA or EACMS-class control system in mainstream public cloud.
- **[INCONCLUSIVE]:** CMMC/StateRAMP/ITAR specifics per CSP offering (claimed in marketing,
  not in cited primary docs); exact CSP participation (if any) in the 2023-09 drafting process.

> Citation keys `[P1-n]` = Perplexity pass-1 (Project 2023-09 status), `[P2-n]` = pass-2
> (cloud-provider readiness); `n` is the source index within that pass's bibliography (NERC
> project/SAR/roster/RSDP/posting-schedule pages, NERC SITES + BCSI guidelines, FERC NOI
> announcement, AWS/Azure/Google compliance docs, FedRAMP marketplace, DoD SRG, Industrial
> Defender). NERC standards and FERC proceedings evolve; all dates are "as of 2026-06-27" and
> must be re-verified against live nerc.com / ferc.gov before being treated as load-bearing.
