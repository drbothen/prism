---
document_type: proposed-adr
status: capture
do_not_execute: true
candidate: true
gated_on: "§5.1 brief-reframe human sign-off (PENDING)"
produced_by: architect
timestamp: "2026-06-28"
provenance: >
  Out-of-band side-analysis; ALTERNATIVE positioning candidate to D-C10-5
  (ADR-PROP-competitive-positioning.md). Problem-framed rather than query-relative.
  Synthesized from the mandated-7 customer problems and the 2026-06-28 adversarial
  fitness pressure-test. Touches NO live artifacts — no STATE.md, no SESSION-HANDOFF.md,
  no live ADR registry, no BC, no story, no ARCH-INDEX.md.
seeded_from:
  - The mandated-7 customer problems (human-confirmed, authoritative)
  - 2026-06-28 adversarial pressure-test (CONDITIONAL-PASS verdict)
cross_refs:
  - day2-design-decisions/ADR-PROP-competitive-positioning.md (D-C10-5 leading competitor-relative candidate; D-C10-3 BINDING honest concessions; D-C10-2 identity-vs-differentiation)
  - day2-design-decisions/ADR-PROP-detection-engine-depth.md (C6 — D-C6-2 RBA-default; D-C6-3 auto-rollback; CORROBORATION-MASTER-GATE)
  - day2-design-decisions/ADR-PROP-siem-lake-federation.md (C5 — ephemeral federation; demand-driven RETAIN; SIEM-by-capability)
  - day2-design-decisions/ADR-PROP-prismql-deliverables.md (C8 — NL->PrismQL; LSP; labor reduction)
  - day2-design-decisions/ADR-PROP-satellite-mesh.md (C2 — D-C2-12 residency; BYOC; mTLS mesh; air-gap reference profile INV-ADS-08)
  - day2-design-decisions/ADR-PROP-dual-deployment.md (D-DEPLOY-005 BYOC zero-access; PIV-DEPLOY-004 satellite-local; air-gap)
  - day2-design-decisions/ADR-PROP-active-query-devices.md (C14 Reading A federation; Reading B gated polling; OQ-C14-OCSF; OQ-C14-SAFETY-LIABILITY)
  - day2-design-decisions/ADR-PROP-prism-context.md (C12 Entity-360; KG; PAT-ADS-08 mandatory citations)
  - day2-design-decisions/ADR-PROP-prism-intel.md (C11 enrichment; CVE/KEV/EPSS advisory)
  - day2-design-decisions/ADR-PROP-soar-actions-aro.md (C15 recommend-only v1; autonomy ladder; E-EVIDENCE-PACKAGE-001)
  - day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md (P-ADS-08 OCSF-at-boundary; INV-ADS-08 air-gap reference profile; PAT-ADS-08 mandatory-faithful-citations)
---

# ADR-PROP — Problem-Framed Positioning Candidate (Alternative to D-C10-5)

> **STATUS: CANDIDATE — NOT RATIFIED.** This is the second positioning candidate,
> produced during the 2026-06-28 out-of-band day-2 side-analysis session.
> It is an ALTERNATIVE to D-C10-5 in `ADR-PROP-competitive-positioning.md`, not a
> supersession of it. Both candidates are inputs to the human-gated §5.1 brief-reframe.
> `do_not_execute: true`. Final headline is ratified ONLY at §5.1 (PENDING).

> **Scope boundary.** This document captures a positioning candidate synthesized
> from the mandated-7 customer problems and the architectural decisions in the
> day-2 C1–C20 ADR-PROP corpus. It touches no live artifacts. It does NOT propose
> changes to any live spec, ADR, BC, story, ARCH-INDEX.md, STATE.md, or
> SESSION-HANDOFF.md.

---

## §1 — Purpose and Relationship to D-C10-5

`ADR-PROP-competitive-positioning.md` produced the D-C10-5 leading-candidate headline
by reasoning from the competitive landscape: Query.io as the benchmark, the gaps Prism
addresses, and the structural moat (OT/air-gap + AI-opaque trust) Query cannot close.
That is a **query-relative frame**: the headline's meaning shifts if Query's capabilities
shift (OQ-C10-1 highest-decay risk).

This document builds a **problem-framed alternative** — the same architecture, the same
honest concessions, but organized around the buyer's articulated pain rather than the
vendor comparison. A problem-framed frame is competitor-agnostic and decay-resistant:
the buyer's problem does not change when Query ships a new feature.

Both frames share the same identity and moat: "agent-native" is the product identity
(D-C10-2 settled), and OT/air-gap/trust is the defensible wedge that competitor-relative
and problem-framed positioning both arrive at. The difference is entry point: D-C10-5
leads with "the data Query can't reach"; this candidate leads with "the problem your IT
SOC can't solve today."

The human picks — or synthesizes — between these two candidates at §5.1. This document
does NOT lock a headline. All headline candidates in §4 are proposals for §5.1
ratification.

**What this candidate explicitly does NOT claim to do:**
- Supersede or replace D-C10-5.
- Ratify any headline.
- Modify any live factory artifact.

---

## §2 — The Mandated-7 to Day-2 Architecture Map

Strength and caveat grades reflect the 2026-06-28 adversarial fitness pressure-test
(CONDITIONAL-PASS). Caveats are baked in; none are softened.

| # | Customer problem | Day-2 architecture answer (citations) | Strength | Honest caveat |
|---|-----------------|--------------------------------------|----------|---------------|
| 1 | **Cheap threat hunts** (cost/economics of hunting) | Ephemeral federated query over sensor APIs (no data lake, no copy cost); demand-driven `RETAIN` caching (C5/C8, D-C5-2); NL→PrismQL authoring reduces analyst labor (C8 L-C8-4); `SIEM by capability, without store-everything cost` | SOLID on the cost MODEL | Initial hunt _content authoring_ still costs analyst labor until the OOTB detection-content library (E-DETECTION-CONTENT-001, PROPOSED) lands. The engine ships content-empty; the cost-of-authoring problem is addressed architecturally but not shipped day-one. |
| 2 | **Tuning is hard** (detection-engineering / false-positive burden) | D-C6-2: RBA-as-default (noisy events accrue risk to entity, not suppressed); suppression-as-code (versioned, mandatory justification, mandatory time-box expiry); fire-frequency dashboard; D-C6-3: auto-rollback (shadow→canary-auto / canary→production-human); CORROBORATION-MASTER-GATE (do not auto-rollback a rule that may be catching a real attack); per-tenant circuit-breaker + exponential backoff | SOLID (tuning machinery is decided and strong) | The continuous/streaming hunt operator (L-C6-2) is the most expensive, novel, Prism-owns-correctness piece in C6; the OOTB rule library (E-DETECTION-CONTENT-001) is the content gap that makes the tuning machinery useful. |
| 3 | **Can't ship data out** (residency / air-gap / OT segmentation) | D-C2-12 (HARD INVARIANT): raw sensor data NEVER crosses the conduit; only OCSF-normalized results transit upward (P-ADS-03); BYOC zero-access by construction (D-DEPLOY-005, PIV-DEPLOY-004); Option-3 operator-zero-access-at-rest (P-ADS-02); air-gap reference profile (INV-ADS-08, `ADR-PROP-config-management.md` C9 signed-bundle); outbound-dial mTLS mesh (D-C2-6) | SOLID (strongest pillar) | Claim: "outbound-dial mTLS mesh + signed-bundle air-gap" — NOT "data-diode air-gap." True one-way diode (D-C2-4) is explicitly DEFERRED. Also: OCSF-normalized ≠ PII-free (D-C2-12 caveat; CONFLICT-7 in ADS; OQ-DEPLOY-2(a) result-transit residency governance is a pre-launch required item). |
| 4 | **Want IT to watch OT** (IT/OT convergence — IT SOC visibility without OT-protocol expertise) | OCSF normalization at the OT edge (P-ADS-08, INV-ADS-07) converts OT-vendor output to IT-readable tables; IT analysts query OT assets in the same PrismQL surface without protocol expertise; C14 OT-as-OCSF tables (D-C14-4); C2 mesh traverses Purdue layers (D-C2-3: Relay/Edge topology); C20 ESP-aware normalization; dual-deployment (three operating models, D-DEPLOY-002) | PARTIAL — abstraction is sound | The OCSF-OT SCHEMA substrate is an OPEN LEAN (OQ-C14-OCSF): OCSF has no native OT classes as of 2026 (open proposal ocsf#1515). The D-C14-4 table model is architecturally decided; specific OCSF class assignments are pending OQ-C14-OCSF follow-up research. Until that closes, OT table normalization uses a private `prism_ot` extension — a decided lean, not a ratified schema. |
| 5 | **What devices exist?** (OT asset inventory / discovery) | C14 Reading A: federate EXISTING OT-discovery platforms (Industrial Defender `asmdataservice`, Nozomi OpenAPI, Claroty API Explorer/xDome, Dragos web API, Armis Centrix) — Prism consumes their asset inventory northbound; C14 Reading B: direct OT-protocol polling (Modbus/OPC-UA/DNP3/SNMP) as poller-of-last-resort for customers with NO OT platform (D-C14-2); OCSF asset tables in PrismQL | PARTIAL — leaning THIN for NATIVE discovery | **CRITICAL — biggest overclaim trap.** Claim: "federates your EXISTING OT-discovery platforms; direct field-device polling as a gated last resort." Do NOT claim "Prism discovers your OT." Reading-B native polling is blocked by three open questions that MUST close before ship: OQ-C14-SAFETY-LIABILITY (legal/insurance — who owns risk if a Prism query contributes to a controller fault; NOT an engineer call), OQ-C14-CADENCE-NUMBERS (safe poll-cadence defaults — no published standards numbers; non-production validation required), OQ-C14-PACKAGING (WASM-compilability of Rust OT-protocol crates — must be evaluated at morph). |
| 6 | **Full OT environment context** (relationships / Purdue zones / criticality / attacker-reachability) | C12 Entity-360 (seven-part view: entity identity + attributes + neighbors + timeline + risk posture + anomalies + cited answer); two-layer KG + vector (indradb graph + usearch hot-ANN + lancedb cold-on-disk); deterministic entity-resolution auto-merge on strong IDs (D-C12-3); suspected-links for fuzzy matches; Purdue zone as first-class attribute; C11 enrichment (CVE/KEV/EPSS/CSAF/VEX advisory decoration at the edge — D-C11-1 feed-down, match-at-edge, central stays blind to asset inventory by construction); PAT-ADS-08 mandatory inline citations | SOLID (architecturally) | GraphRAG Phase 2 (global community/campaign sense-making, Leiden algorithm) is committed but PHASED — Phase 1 Entity-360 ships first. The ~57%-unfaithful-RAG risk is GATED by mandatory citations + post-hoc faithfulness check (PAT-ADS-08) — this is "best achievable, not a proof." Claim: "cited entity context with confidence," NOT "ground-truth topology." |
| 7 | **Can't find talent** (OT security skills shortage) | C15 recommend-only v1 (D-C15-2: advisory-level, no autonomous action); autonomy ladder designed now (advisory → suggested → auto-with-approval → autonomous), with per-action-class, evidence-measured, REVERSIBLE promotion; OT/safety-critical actions are HITL-permanent by design; S3 agent + GAP-Q2 evidence package (E-EVIDENCE-PACKAGE-001, PROPOSED: Investigation Report + Replayable Query Log + IOC Ledger + self-QA gate); C8 NL→PrismQL (L-C8-4) + LSP (L-C8-3) — a tier-1 analyst authors tier-3 detection queries without OT-protocol expertise; agent-native identity | PARTIAL | Claim: "cited-recommendation AUGMENTATION, human-in-the-loop" — NOT "autonomous AI analyst." v1 is recommend-only, ZERO autonomous action. The agent augments judgment; it does not replace headcount by acting. Do NOT claim the talent gap is "solved." |

---

## §3 — Positioning Pillars (Synthesized from the Mandated-7)

The seven problems cluster into three pillars. Each pillar identifies the buyer-voice problem(s),
the decided architecture, and the honest boundary.

---

### Pillar A — "Your data never leaves; your IT SOC still sees everything."

**Problems answered:** #3 (can't ship data out) + #4 (want IT to watch OT)

**Why it is the structural moat:** These two problems are coupled by the same architectural
property — edge-normalization. Because raw sensor data NEVER crosses the conduit (D-C2-12,
P-ADS-03) and OCSF normalization happens at the OT edge (P-ADS-08, INV-ADS-07), the
satellite mesh simultaneously satisfies "data stays at the site" AND "IT analysts get
readable tables." The same architectural constraint that enforces residency is what enables
cross-domain readability. No feature flag; no opt-in. It is structural.

**The decided architecture:**
- Outbound-dial mTLS mesh traverses Purdue layers without inbound firewall rules (D-C2-6, D-C2-3).
- OCSF-normalized results (not raw) transit the conduit (D-C2-12 HARD INVARIANT).
- BYOC zero-access: satellite-local credential resolution, operator stays blind (D-DEPLOY-005, PIV-DEPLOY-004, P-ADS-02).
- Air-gap reference profile (INV-ADS-08): signed-bundle delivery (C9), on-box inference, no internet dependency.
- Three operating models (SaaS / MSSP-managed / client-managed) with the zero-access character varying by spectrum, strongest at client-managed (D-DEPLOY-002).

**The honest boundary:**
- Not a data-diode (true one-way diode is DEFERRED, D-C2-4); it is outbound-dial mTLS.
- OCSF-normalized ≠ PII-free (OQ-DEPLOY-2(a) is a pre-launch required hardening item).
- The OT-OCSF schema (OQ-C14-OCSF) is an open lean; the abstraction works before that closes but class assignments are provisional.

---

### Pillar B — "Know your OT environment — what's on it and how it connects."

**Problems answered:** #5 (what devices exist?) + #6 (full OT environment context)

**Why this is hard for any competitor without the structural foundation:** Neither problem
is solvable with a SaaS-only architecture. Discovering field devices requires a mesh that
traverses segmented OT networks (Pillar A). Understanding context requires a knowledge
graph that can model Purdue-zone topology, control relationships, and criticality, fed by
the same OCSF-normalized stream that Pillar A produces at the edge.

**The decided architecture:**
- Reading A: federate existing OT-discovery platforms (Industrial Defender/Nozomi/Claroty/Dragos/Armis) as HTTP source adapters — Prism queries their northbound REST APIs; zero OT-protocol safety risk on this path (D-C14-1, D-C14-3, E-ACTIVE-QUERY-001).
- Reading B (gated): direct OT-protocol polling (Modbus/OPC-UA/DNP3/SNMP) for customers with no OT platform (D-C14-2, E-OT-PROTOCOL-CONNECTORS-001) — gated on OQ-C14-SAFETY-LIABILITY, OQ-C14-CADENCE-NUMBERS, OQ-C14-PACKAGING before shipping.
- C12 Entity-360: seven-part view per entity; indradb graph + usearch hot-ANN + lancedb cold-on-disk (D-C12-1); Purdue zone as first-class attribute; deterministic entity-resolution on strong IDs + suspected-links for fuzzy (D-C12-3); PAT-ADS-08 mandatory citations on all synthesized answers (E-PRISM-CONTEXT-001).
- C11 enrichment: CVE/KEV/EPSS/CVSS-v4/VEX advisory decoration at the edge (D-C11-1 feed-down, match-at-edge); asset inventory NEVER reaches central in default path (PIV-C11-001, D-C11-2 consent-gated exception SaaS-only).

**The honest boundary:**
- Native discovery (Reading B) is gated — three open questions (OQ-C14-SAFETY-LIABILITY is a LEGAL question, not engineering). Do NOT headline "Prism discovers your OT"; say "federates your existing discovery platforms."
- Entity-360 is SOLID architecturally; GraphRAG Phase 2 (global community sense-making) is committed but post-Phase-1.
- The OCSF-OT schema is an open lean (OQ-C14-OCSF); OT tables are provisionally `prism_ot` extension classes.

---

### Pillar C — "Threat hunting your team can actually afford and run — AI-augmented, no store-everything cost, no unicorn hire."

**Problems answered:** #1 (cheap threat hunts) + #2 (tuning is hard) + #7 (can't find talent)

**Why these three problems compound:** A team that can't afford hunting (#1) also can't
absorb tuning labor (#2), and without OT security talent (#7) neither problem gets better
over time. Prism addresses the economics, the labor, and the skill-floor in combination.
The three answers reinforce each other: cheaper hunting attracts more analyst time; better
tuning tools reduce the labor burden on that time; AI augmentation lowers the skill floor
for the analysts who show up.

**The decided architecture:**
- Ephemeral federated query (no data lake, no copy cost): each hunt fetches on demand and materializes only what the query touches; `RETAIN` demand-driven caching (D-C5-2) for repeated patterns — SIEM capability at the economics of a query engine (C5/C8).
- C8 NL→PrismQL (L-C8-4): natural-language to PrismQL translation; the parser/planner diagnostics as validate-repair signal; LSP server (L-C8-3) reused across Monaco/CLI/NL agent — a tier-1 analyst authors tier-3 detection logic without PrismQL mastery.
- C6 tuning machinery (D-C6-2): RBA as default (noisy events accrue risk, not silenced); suppression-as-code (versioned, mandatory justification, mandatory time-box expiry); fire-frequency dashboard; auto-tune suggestions only (human sign-off required, never autonomous).
- D-C6-3 auto-rollback: shadow→canary-auto / canary→production-human gate; CORROBORATION-MASTER-GATE discriminates real-attack spikes from broken rules before auto-rollback trips; per-tenant circuit-breaker + exponential backoff on repeated trips.
- C15 recommend-only v1 (D-C15-2): S3 agent produces Investigation Report + Replayable Query Log + IOC Ledger + self-QA gate (E-EVIDENCE-PACKAGE-001, PROPOSED); autonomy ladder designed with per-action-class, REVERSIBLE promotion gates; OT/safety actions are HITL-permanent.

**The honest boundary:**
- The cost model is SOLID and uncontested; the OOTB detection content (E-DETECTION-CONTENT-001, PROPOSED) is the content gap that makes #1 and #2 land for a buyer who has no existing rules.
- #7 claim is augmentation, NOT replacement. Claim: "an analyst who knows IT can learn OT with Prism"; do NOT claim "you don't need an OT analyst."
- v1 autonomy = recommend-only, zero autonomous action.

---

## §4 — Candidate Problem-Framed Headlines (NOT Locked — For §5.1 Ratification)

All three options below are candidates only. "Agent-native" is maintained as identity
in all of them (D-C10-2 settled). OT/air-gap/trust remains the structural moat.
Cross-reference D-C10-5 as the competitor-relative alternative; the human picks or
synthesizes at §5.1.

**Option H-PF-1 (Pillar A leads):**
> "See your OT environment from your IT SOC — data that never leaves, context you can
> actually query, and the AI-augmented headcount you don't have to hire."

- OT/air-gap + trust leads (Pillar A moat).
- #4/#3/#6/#7 all surface.
- Honest: agent-native is identity; the augmentation is explicit.

**Option H-PF-2 (Economics-first, OT-trust as differentiator):**
> "Threat hunt across OT and IT, at the cost of a query instead of a lake — your data
> stays at the site, your analysts stop drowning in alerts."

- #1/#2 economics lead (Pillar C entry point).
- "Data stays at the site" carries the residency moat without claiming data-diode.
- Honest: no OOTB content claim; "query instead of a lake" is uncontested architecture.

**Option H-PF-3 (Synthesized across all three pillars):**
> "The agent-native platform that brings OT security into your IT SOC — on your
> infrastructure, with your data, without hiring the OT analyst you can't find."

- Leads with the agent-native identity.
- "OT into IT SOC" (Pillar A/B); "on your infrastructure" (Pillar A residency); "without hiring" (Pillar C talent).
- Honest: "without hiring" means augmentation, not headcount elimination.

All three options should be compared against D-C10-5 at §5.1:
> D-C10-5 (competitor-relative): *"The agent-native federated query platform for the data
> Query can't reach — OT/edge/air-gap — with credentials the AI never sees."*

---

## §5 — Binding Honest Concessions

These carry D-C10-3 verbatim in spirit. They are BINDING on this document and on any
positioning artifact that derives from it.

| Concession | What to say | What NOT to say |
|------------|-------------|----------------|
| **Shipping maturity** | "Prism is in active development; these are decided and committed architecture decisions." | "Prism ships this today." C1–C20 are CAPTURE-stage, `do_not_execute: true`. |
| **OOTB detection content** | "The detection engine ships with your own detection logic; an OOTB rule library is proposed (E-DETECTION-CONTENT-001)." | "Prism ships detection content out of the box." |
| **Native OT discovery (Reading B)** | "Prism federates your existing OT-discovery platforms; direct field-device polling is a gated last resort for customers with no OT platform." | "Prism discovers your OT." Reading B is gated on three open items, including a legal/insurance question (OQ-C14-SAFETY-LIABILITY). |
| **Autonomous AI analyst** | "Prism's AI agent produces cited recommendations; a human approves every action. v1 is recommend-only." | "Prism is an autonomous AI analyst" or "replaces your SOC headcount." OT/safety-critical actions are HITL-permanent by design. |
| **Data-diode air-gap** | "Outbound-dial mTLS mesh; signed-bundle air-gap delivery." | "Data-diode air-gap." True one-way diode is explicitly DEFERRED (D-C2-4). |
| **Agent-native differentiation** | "Agent-native is what Prism is — MCP-first, built for AI agents." | "First or only agent-native security platform." Query ships Workers + MCP + A2A (D-C10-3 binding). |
| **First/only OT-OCSF normalization** | "Prism normalizes OT data to OCSF at the edge using a decided schema model." | Claim a ratified OCSF OT standard. OQ-C14-OCSF is open; OCSF has no native OT classes as of 2026. |

---

## §6 — Adversarial Fitness Appendix

**Date:** 2026-06-28
**Verdict:** CONDITIONAL-PASS

The 2026-06-28 adversarial pressure-test evaluated the problem-framed positioning against
each of the seven mandated customer problems for: (a) architectural support — is there a
decided architectural answer? and (b) overclaim risk — what does the positioning break if
a buyer asks a follow-up question?

### Per-problem strength table

| # | Problem | Strength verdict | Primary risk |
|---|---------|-----------------|--------------|
| 1 | Cheap hunts | SOLID on cost model | OOTB content gap (E-DETECTION-CONTENT-001 proposed, not shipped) |
| 2 | Tuning is hard | SOLID (tuning machinery complete) | Continuous operator cost; OOTB content makes the machinery useful |
| 3 | Can't ship data out | SOLID (strongest pillar) | Data-diode overclaim; OQ-DEPLOY-2(a) pre-launch hardening item; OCSF ≠ PII-free |
| 4 | IT watches OT | PARTIAL | OQ-C14-OCSF open lean; `prism_ot` extension is provisional |
| 5 | What devices exist? | PARTIAL / THIN for native | Reading B gated on OQ-C14-SAFETY-LIABILITY (legal) + two engineering open questions |
| 6 | OT context | SOLID (architecturally) | GraphRAG Phase-2 phased; mandatory-citation gating is "best achievable, not a proof" |
| 7 | Talent gap | PARTIAL | v1 is recommend-only; "augmentation" must be stated plainly, not elided |

### Five binding caveats (from the pressure-test)

1. **OOTB content empty:** Problems #1 and #2 rest on a content-empty detection engine until E-DETECTION-CONTENT-001 ships. The cost-model and tuning-machinery claims are uncontested; the "hunting productivity" claim requires content.

2. **Reading B gated:** Problem #5 native-discovery claim is blocked by OQ-C14-SAFETY-LIABILITY (legal/insurance — NOT engineering-resolvable before ship), OQ-C14-CADENCE-NUMBERS, and OQ-C14-PACKAGING. Do not headline native OT discovery unconditioned.

3. **OT-OCSF schema open lean:** Problem #4 convergence abstraction is sound (P-ADS-08, INV-ADS-07, D-C14-3); the OCSF class assignments are OQ-C14-OCSF-pending. Claim the abstraction; do not claim a ratified OT-OCSF standard.

4. **Not data-diode:** Problem #3 air-gap claim must be precise. Outbound-dial mTLS + signed-bundle = decided and strong. True one-way data-diode = explicitly deferred (D-C2-4). Any claim beyond "outbound-dial / no inbound firewall rule required" and "signed-bundle air-gap delivery" is an overclaim.

5. **Recommend-only v1:** Problem #7 augmentation claim must be explicit. The autonomy ladder is designed; v1 is advisory-only. "The agent recommends; you decide" is the correct claim. "AI SOC analyst" or any framing implying autonomous action is out of bounds for v1.

### The "no true GAP" finding

The adversarial pass found no problem in the mandated-7 for which Prism has NO architectural
answer. The five CONDITIONAL items above are honesty gates, not gaps. The architecture
genuinely addresses all seven problems; the caveats govern how each is positioned.

### Note on problems #1 and #2 cost grading

The #1 cost angle (hunting economics) was not independently graded in the adversarial pass —
it was evaluated as part of the combined #1+#2 detection-operations surface. The cost-model
claim (ephemeral/demand-driven vs store-everything) rests on uncontested decided architecture
(C5/C8 ephemeral federation + RETAIN primitive). No adversary finding challenged the cost
model. The OOTB-content-labor caveat carries over from the content-emptiness finding, not
from any cost-model challenge.

---

## §7 — Open Items and Gating

| Item | Status | What gates it |
|------|--------|---------------|
| **§5.1 brief-reframe human sign-off** | PENDING | Final headline ratified only at §5.1; this document and D-C10-5 are the two inputs |
| **OQ-C10-1 product-fact re-verification** | Required before any external claim | All competitor product facts decay; re-verify at §5.1 and before any public statement |
| **Native OT discovery (Reading B) claims** | BLOCKED | OQ-C14-SAFETY-LIABILITY (legal), OQ-C14-CADENCE-NUMBERS, OQ-C14-PACKAGING must all close before Reading B can be headlined |
| **OT-OCSF schema (OQ-C14-OCSF)** | Open lean | Follow-up research in-flight; OCSF class assignments provisional until it closes |
| **OOTB content library (E-DETECTION-CONTENT-001)** | PROPOSED | Not a registered story; must not be headlined as "shipped" until it is |
| **E-EVIDENCE-PACKAGE-001** | PROPOSED | S3-agent output contract; not a registered story |
| **Autonomy ladder v2+ promotion** | Post-v1 | v1 = recommend-only; any claim about autonomous actions is post-v1 and gated by per-action-class evidence requirements + D-C15-2 |
| **GraphRAG Phase 2** | Committed, post-Phase-1 | Phase-1 Entity-360 ships first; Phase-2 global community sense-making is committed but not first-ship |

**Proposed resolution for §5.1:**
The human compares this candidate (problem-framed, D-C10-7-PF) against D-C10-5
(competitor-relative) and either: (a) picks one as the headline frame, (b) synthesizes
a hybrid (problem-framed entry with competitor-relative moat specificity), or
(c) splits them by audience (D-C10-5 for analyst/competitive context; this candidate
for buyer discovery/executive context). All three dispositions are valid; the human
decides at §5.1.
