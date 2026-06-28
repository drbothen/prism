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

**Formal verification precision note (OVER-05):** Where this document references formal
verification of PrismQL, the correct scope is "formally-verified parser-safety properties
(Kani proofs bounding query size/recursion depth — VP-014/VP-015)" — not "formally-verified
query language" or "Kani-verified grammar." The VP coverage is bounded parser-safety, not
an end-to-end semantic proof of the whole language. This precision constraint propagates
to any derivative of this document.

---

## §2 — The Mandated-7 to Day-2 Architecture Map

Strength and caveat grades reflect the 2026-06-28 adversarial fitness pressure-test
(CONDITIONAL-PASS). Caveats are baked in; none are softened.

| # | Customer problem | Day-2 architecture answer (citations) | Strength | Honest caveat |
|---|-----------------|--------------------------------------|----------|---------------|
| 1 | **Cheap threat hunts** (cost/economics of hunting) | Ephemeral federated query over sensor APIs (no data lake, no copy cost); demand-driven `RETAIN` caching (C5/C8, D-C5-2); NL→PrismQL authoring reduces analyst labor (C8 L-C8-4); `SIEM by capability, without store-everything cost`; "model is the memory" ML tier for long-horizon baselines at fraction of store-everything cost (§15/§15.11) | SOLID on the cost MODEL | Initial hunt _content authoring_ still costs analyst labor until the OOTB detection-content library (E-DETECTION-CONTENT-001, PROPOSED) lands. The engine ships content-empty; the cost-of-authoring problem is addressed architecturally but not shipped day-one. Connector egress to customer lakes (E-EGRESS-PIPELINE-001) is also PROPOSED. |
| 2 | **Tuning is hard** (detection-engineering / false-positive burden) | D-C6-2: RBA-as-default (noisy events accrue risk to entity, not suppressed); suppression-as-code (versioned, mandatory justification, mandatory time-box expiry); fire-frequency dashboard; D-C6-3: auto-rollback (shadow→canary-auto / canary→production-human); CORROBORATION-MASTER-GATE (do not auto-rollback a rule that may be catching a real attack); per-tenant circuit-breaker + exponential backoff; dual-tier backtest + mandatory coverage map (D-C6-1); bitemporal AS OF KNOWN <T> replay (D-C8-2/D-C8-3) | SOLID (tuning machinery is decided and strong) | The continuous/streaming hunt operator (C6-LEAN-2, Phase 2) is the most expensive, novel, Prism-owns-correctness piece; Phase 1 ships NRT-over-cache. The OOTB rule library (E-DETECTION-CONTENT-001) is the content gap that makes the tuning machinery useful. |
| 3 | **Can't ship data out** (residency / air-gap / OT segmentation) | D-C2-12 (HARD INVARIANT): raw sensor data NEVER crosses the conduit; only OCSF-normalized results transit upward (P-ADS-03); BYOC zero-access by construction (D-DEPLOY-005, PIV-DEPLOY-004); Option-3 operator-zero-access-at-rest (P-ADS-02); air-gap reference profile (INV-ADS-08, `ADR-PROP-config-management.md` C9 signed-bundle); outbound-dial mTLS mesh (D-C2-6); crypto-shred GDPR erasure-by-key-destruction + tenant-held recovery key (C17) | SOLID (strongest pillar) | Claim: "outbound-dial mTLS mesh + signed-bundle air-gap" — NOT "data-diode air-gap." True one-way diode (D-C2-4) is explicitly DEFERRED. Also: OCSF-normalized ≠ PII-free (D-C2-12 caveat; OQ-DEPLOY-2(a) result-transit residency governance is a pre-launch required item). |
| 4 | **Want IT to watch OT** (IT/OT convergence — IT SOC visibility without OT-protocol expertise) | OCSF normalization at the OT edge (P-ADS-08, INV-ADS-07) converts OT-vendor output to IT-readable tables; IT analysts query OT assets in the same PrismQL surface without protocol expertise; C14 OT-as-OCSF tables (D-C14-4); C2 mesh traverses Purdue layers (D-C2-3: Relay/Edge topology); C20 ESP-aware normalization; dual-deployment (three operating models, D-DEPLOY-002); **passive OT sensor** (full-packet PCAP + native Spicy-style dissector, §17.6/§17.12/§17.13) | PARTIAL — abstraction is sound | The OCSF-OT SCHEMA substrate is an OPEN LEAN (OQ-C14-OCSF): OCSF has no native OT classes as of 2026 (open proposal ocsf#1515). The D-C14-4 table model is architecturally decided; specific OCSF class assignments are pending OQ-C14-OCSF follow-up research. Until that closes, OT table normalization uses a private `prism_ot` extension — a decided lean, not a ratified schema. |
| 5 | **What devices exist?** (OT asset inventory / discovery) | C14 Reading A: federate EXISTING OT-discovery platforms (Industrial Defender `asmdataservice`, Nozomi OpenAPI, Claroty API Explorer/xDome, Dragos web API, Armis Centrix) — Prism consumes their asset inventory northbound; C14 Reading B: direct OT-protocol polling (Modbus/OPC-UA/DNP3/SNMP) as poller-of-last-resort for customers with NO OT platform (D-C14-2); **Reading C / Passive: full-packet PCAP capture + native Spicy-style dissector** (§17.6/§17.12/§17.13) — standalone OT visibility with STRICT PASSIVITY (TAP/SPAN, never injects), safe and ungated; OCSF asset tables in PrismQL | PARTIAL — leaning THIN for NATIVE discovery | **CRITICAL — biggest overclaim trap.** Claim: "federates your EXISTING OT-discovery platforms; passive TAP/SPAN sensor as the safe standalone path; direct field-device polling as a gated last resort." Do NOT claim "Prism discovers your OT" without qualification. Reading-B native polling is blocked by three open questions that MUST close before ship: OQ-C14-SAFETY-LIABILITY (legal/insurance — who owns risk if a Prism query contributes to a controller fault; NOT an engineer call), OQ-C14-CADENCE-NUMBERS (safe poll-cadence defaults — no published standards numbers; non-production validation required), OQ-C14-PACKAGING (WASM-compilability of Rust OT-protocol crates — must be evaluated at morph). Passive sensor (Reading C) is DECIDED, safe, and carries no safety gate. |
| 6 | **Full OT environment context** (relationships / Purdue zones / criticality / attacker-reachability) | C12 Entity-360 (seven-part view: entity identity + attributes + neighbors + timeline + risk posture + anomalies + cited answer); two-layer KG + vector (indradb graph + usearch hot-ANN + lancedb cold-on-disk); deterministic entity-resolution auto-merge on strong IDs (SID, UUID, stable asset UUID, MAC — D-C12-3); suspected-links with confidence scores for fuzzy matches (hostname/IP are SUSPECTED-LINKS only, gated by security-reviewer sign-off per PIV-C12-4); Purdue zone as first-class attribute; C11 enrichment (CVE/KEV/EPSS/CSAF/VEX advisory decoration at the edge — D-C11-1 feed-down, match-at-edge, central stays blind to asset inventory by construction); PAT-ADS-08 mandatory inline citations; bitemporal AS OF KNOWN <T> replay (D-C8-2/D-C8-3); passive PCAP + native dissector feeding OT network activity tables (§17.6/§17.12) | SOLID (architecturally) | GraphRAG Phase 2 (global community/campaign sense-making, Leiden algorithm) is committed but PHASED — Phase 1 Entity-360 ships first. The ~57%-unfaithful-RAG risk is GATED by mandatory citations + post-hoc faithfulness check (PAT-ADS-08) — this is "best achievable, not a proof." Claim: "cited entity context with confidence," NOT "ground-truth topology." |
| 7 | **Can't find talent** (OT security skills shortage) | C15 recommend-only v1 (D-C15-2: advisory-level, no autonomous action); autonomy ladder designed now (advisory → suggested → auto-with-approval → autonomous), with per-action-class, evidence-measured, REVERSIBLE promotion; OT/safety-critical actions are HITL-permanent by design; the conversational-canvas / embedded-AI surface (Prism surface S3 — not Amazon S3) + GAP-Q2 evidence package (E-EVIDENCE-PACKAGE-001, PROPOSED: Investigation Report + Replayable Query Log + IOC Ledger + self-QA quality gate); C8 NL→PrismQL (L-C8-4) + LSP (L-C8-3) — a tier-1 analyst authors tier-3 detection queries without OT-protocol expertise; KQL/SPL-style piped query surface compiling to the same engine plan — SPL/KQL-trained analysts productive immediately (D-C8-1); agent-native identity | PARTIAL | Claim: "cited-recommendation AUGMENTATION, human-in-the-loop" — NOT "autonomous AI analyst." v1 is recommend-only, ZERO autonomous action. The agent augments judgment; it does not replace headcount by acting. Do NOT claim the talent gap is "solved." |

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
- Fully self-contained storage stack — bundled PostgreSQL (control-plane only, NEVER external/cloud), embedded RocksDB + SQLite, object-store Iceberg — no external managed database; air-gap valid by construction (ADR-PROP-storage-engine-taxonomy.md, P-ADS-04 boundary) **[DECIDED]**.
- Backup, recovery, and erasure cluster (C17): crypto-shred GDPR erasure-by-key-destruction for pooled multi-tenant stores; tenant-held recovery key + optional M-of-N Shamir escrow ("no unilateral operator access — not no access under any circumstance"); DR tier ladder; CIP-009 recovery-test evidence generation **[DECIDED]**.
- Compliance profiles: five named presets (baseline / SOC2 / ISO 27001 / IEC-62443-OT / NERC-CIP); "CIP-deployable + CIP-evidence-generating" posture (NOT "CIP-certified") (D-PROF-1/-6, D-C20) **[DECIDED]**.
- Enterprise identity: OIDC + SAML 2.0 per-tenant SSO; SCIM 2.0 auto-provisioning/deprovisioning; fine-grained RBAC down to source/column with decision-level audit (ADR-PROP-sso-identity.md) **[DECIDED]**.

**The honest boundary:**
- Not a data-diode (true one-way diode is DEFERRED, D-C2-4); it is outbound-dial mTLS.
- OCSF-normalized ≠ PII-free (OQ-DEPLOY-2(a) is a pre-launch required hardening item).
- The OT-OCSF schema (OQ-C14-OCSF) is an open lean; the abstraction works before that closes but class assignments are provisional.
- "No unilateral operator access" is the precise claim for backup/recovery (D-C17-SF1); not "no access under any circumstance."
- Compliance profiles are DECIDED architecture (CAPTURE-stage); "CIP-deployable + CIP-evidence-generating" is accurate; NOT "CIP-certified."
- SSO/SCIM/RBAC are DECIDED architecture (CAPTURE-stage); "Prism plans; Query ships" caveat (D-C10-3) applies.

---

### Pillar B — "Know your OT environment — what's on it and how it connects."

**Problems answered:** #5 (what devices exist?) + #6 (full OT environment context)

**Why this is hard for any competitor without the structural foundation:** Neither problem
is solvable with a SaaS-only architecture. Discovering field devices requires a mesh that
traverses segmented OT networks (Pillar A). Understanding context requires a knowledge
graph that can model Purdue-zone topology, control relationships, and criticality, fed by
the same OCSF-normalized stream that Pillar A produces at the edge.

**The decided architecture:**

- **Reading A (decided):** Federate existing OT-discovery platforms (Industrial Defender/Nozomi/Claroty/Dragos/Armis) as HTTP source adapters — Prism queries their northbound REST APIs; zero OT-protocol safety risk on this path (D-C14-1, D-C14-3, E-ACTIVE-QUERY-001).
- **Reading B (gated):** Direct OT-protocol polling (Modbus/OPC-UA/DNP3/SNMP) for customers with no OT platform (D-C14-2, E-OT-PROTOCOL-CONNECTORS-001) — gated on OQ-C14-SAFETY-LIABILITY, OQ-C14-CADENCE-NUMBERS, OQ-C14-PACKAGING before shipping.
- **Reading C / Passive (decided, safe, ungated):** Full-packet PCAP capture (§17.6, E-COLLECTOR-PCAP-001) + native Spicy-style protocol dissector (§17.12, E-DISSECTOR-NATIVE-001/OT-001) emitting OCSF Network Activity + native OT schema-on-read (Modbus/DNP3/S7/GOOSE/PROFINET); STRICT PASSIVITY (TAP/SPAN, never injects, §17.13). This is the same passive listen-and-dissect mechanism used by incumbent OT security platforms (Claroty/Nozomi/Dragos). No OT-protocol safety gate applies — strict passivity IS the safety property. Caveat: §17.9 build-stage note — the full dissector is the heaviest collector build; protocol breadth is phased; encrypted OT traffic = metadata-only.
- PCAP retrieve/query analyst affordance: `retrieve-packets-by-session` + S2 console download — full packets retrievable on demand for forensics, no separate full-packet-capture appliance (§17.6, E-COLLECTOR-PCAP-001) **[DECIDED]**.
- C12 Entity-360: seven-part view per entity; indradb graph + usearch hot-ANN + lancedb cold-on-disk (D-C12-1); Purdue zone as first-class attribute; deterministic entity-resolution on strong IDs (SID, UUID, stable asset UUID, MAC — D-C12-3) + suspected-links for fuzzy (hostname/IP = SUSPECTED-LINKS, gated by security-reviewer sign-off, PIV-C12-4); PAT-ADS-08 mandatory citations on all synthesized answers (E-PRISM-CONTEXT-001).
- Bitemporal AS OF KNOWN <T> replay (D-C8-2/D-C8-3): replay any investigation exactly as Prism saw it at decision-time — entity identity and schema interpretation pinned; no surveyed commercial-tool equivalent **[DECIDED for registry+catalog halves; cold-tier data-snapshot deferred, OQ-C8-DATASNAPSHOT]**.
- C11 enrichment: CVE/KEV/EPSS/CVSS-v4/VEX advisory decoration at the edge (D-C11-1 feed-down, match-at-edge); asset inventory NEVER reaches central in default path (PIV-C11-001, D-C11-2 consent-gated exception SaaS-only).

**The honest boundary:**
- The correct external claim is "federates your existing OT-discovery platforms, passively captures network traffic, or actively polls as a carefully-gated last resort" — lead with Reading A and Reading C; qualify Reading B explicitly.
- Native active-polling (Reading B) is gated — three open questions (OQ-C14-SAFETY-LIABILITY is a LEGAL question, not engineering). Do NOT headline "Prism discovers your OT."
- Entity-360 is SOLID architecturally; GraphRAG Phase 2 (global community sense-making) is committed but post-Phase-1.
- The OCSF-OT schema is an open lean (OQ-C14-OCSF); OT tables are provisionally `prism_ot` extension classes.
- Passive sensor (Reading C) is decided and safe but the dissector build is heavy and phased (§17.9).

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
- "Model is the memory": on-demand ML/behavior analytics tier (§15/§15.11, D-C7-1..4) — long-horizon behavioral baselines (UEBA) maintained at model-sized cost without store-everything; anomaly/behavior scoring as PrismQL primitives (ANOMALY_SCORE, PROFILE…OVER, RARITY, FIRST_SEEN, PEER_OUTLIER) **[PARTIAL/PHASED — statistical tier day-2-first; online/learned tier later]**.
- C8 NL→PrismQL (L-C8-4): natural-language to PrismQL translation; the parser/planner diagnostics as validate-repair signal; LSP server (L-C8-3) reused across Monaco/CLI/NL agent — a tier-1 analyst authors tier-3 detection logic without PrismQL mastery.
- KQL/SPL-style piped query surface (D-C8-1): desugars to the identical DataFusion logical plan; SPL/KQL-trained analysts productive immediately without relearning SQL **[DECIDED]**.
- Safe, cost-guarded cross-source joins with plan-visible degradation (D-C10-1, PAT-ADS-04): per-side row caps, dynamic filtering, mandatory time-bound injection — query cost bounded structurally, not by analyst discipline. A STRENGTH, not a concession (D-C10-3 "do NOT concede") **[DECIDED; committed-architecture, PIV-C3-1..3]**.
- C6 tuning machinery (D-C6-2): RBA as default (noisy events accrue risk, not silenced); suppression-as-code (versioned, mandatory justification, mandatory time-box expiry); fire-frequency dashboard; auto-tune suggestions only (human sign-off required, never autonomous).
- D-C6-3 auto-rollback: shadow→canary-auto / canary→production-human gate; CORROBORATION-MASTER-GATE discriminates real-attack spikes from broken rules before auto-rollback trips; per-tenant circuit-breaker + exponential backoff on repeated trips.
- Dual-tier backtest + mandatory coverage map (D-C6-1): cold-tier deterministic (snapshot-id + rule-version pair, reproducible) AND remote best-effort (labeled non-deterministic), ALWAYS with a coverage map distinguishing "evaluated, no match" from "no data to evaluate" — a decided, novel correctness affordance no surveyed competitor ships **[DECIDED]**.
- Bitemporal AS OF KNOWN <T> replay (D-C8-2/D-C8-3): replay any past investigation exactly as Prism saw it at decision-time — entity identity and schema interpretation pinned; Prism-novel differentiator with no commercial-tool equivalent **[DECIDED for registry+catalog halves]**.
- Rule translation Sigma/SPL/KQL/NL→PrismQL with fidelity report (C6 ADR-PROP-C6-LEAN-3, E-RULE-XLATE-001); MATCH_RECOGNIZE expresses Sigma temporal correlations as first-class where other backends approximate (L-C6-3) **[PARTIAL; OUT direction deferred]**.
- PrismQL RECOMMEND projection (D-C15-4): a detection recipe can emit a typed recommendation as read-only DATA — perimeter-compile-fail-tested so the query language never gains a write path; recommendations authored declaratively in the same surface as detections **[DECIDED]**.
- C15 recommend-only v1 (D-C15-2): the conversational-canvas / embedded-AI surface (Prism surface S3 — not Amazon S3) produces Investigation Report + Replayable Query Log + IOC Ledger + self-QA quality gate (E-EVIDENCE-PACKAGE-001, PROPOSED); autonomy ladder designed with per-action-class, REVERSIBLE promotion gates; OT/safety actions are HITL-permanent.

**The honest boundary:**
- The cost model is SOLID and uncontested; the OOTB detection content (E-DETECTION-CONTENT-001, PROPOSED) is the content gap that makes #1 and #2 land for a buyer who has no existing rules.
- #7 claim is augmentation, NOT replacement. Claim: "an analyst who knows IT can learn OT with Prism"; do NOT claim "you don't need an OT analyst."
- v1 autonomy = recommend-only, zero autonomous action.
- Continuous/streaming detection (Phase 2) is the most expensive single item; Phase 1 ships NRT-over-cache.
- ML/behavior analytics: statistical tier day-2-first; online/learned tier is later-phase.

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
| **Passive OT sensor (Reading C)** | "Passive PCAP capture + native protocol dissector (TAP/SPAN only, never injects) — same approach as incumbent OT security platforms; decided, safe, ungated; dissector build is phased." | Imply the full dissector protocol breadth ships day-one; the heavy build (§17.9) is phased. |
| **Autonomous AI analyst** | "Prism's AI agent produces cited recommendations; a human approves every action. v1 is recommend-only." | "Prism is an autonomous AI analyst" or "replaces your SOC headcount." OT/safety-critical actions are HITL-permanent by design. |
| **Data-diode air-gap** | "Outbound-dial mTLS mesh; signed-bundle air-gap delivery." | "Data-diode air-gap." True one-way diode is explicitly DEFERRED (D-C2-4). |
| **Agent-native differentiation** | "Agent-native is what Prism is — MCP-first, built for AI agents." | "First or only agent-native security platform." Query ships Workers + MCP + A2A (D-C10-3 binding). |
| **First/only OT-OCSF normalization** | "Prism normalizes OT data to OCSF at the edge using a decided schema model." | Claim a ratified OCSF OT standard. OQ-C14-OCSF is open; OCSF has no native OT classes as of 2026. |
| **Backup/recovery operator access** | "No unilateral operator access to tenant data — tenant-held key + optional M-of-N escrow." | "The operator has no access under any circumstance." M-of-N escrow gives the operator one Shamir share; the D-C17-SF1 precision is "no unilateral access." |
| **Compliance posture** | "CIP-deployable + CIP-evidence-generating — we generate the audit evidence your CIP auditors ask for." | "CIP-certified." No certification for a tool exists (C20 §1). |
| **Passive-read-only CIP classification** | "Passive read-only deployment reduces CIP classification weight by default; write/control paths are feature-flagged off." | "Read-only is a CIP safe harbor — the operator still classifies per CIP-002 regardless." (D-C20-SF3: both the benefit AND this caveat are binding.) |
| **Formally-verified query** | "Formally-verified parser-safety properties (Kani proofs bounding query size/recursion depth — VP-014/VP-015)." | "Formally-verified query language" or "Kani-verified grammar." The Kani VPs are bounded parser-safety proofs, not end-to-end semantic verification of the language. |
| **Enterprise identity** | "OIDC + SAML 2.0 per-tenant SSO, SCIM auto-provisioning, fine-grained RBAC — CAPTURE-stage decided architecture." | Imply SSO/RBAC/SCIM are shipping today. |

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
| 1 | Cheap hunts | SOLID on cost model | OOTB content gap (E-DETECTION-CONTENT-001 proposed, not shipped); connector-egress also PROPOSED |
| 2 | Tuning is hard | SOLID (tuning machinery complete) | Continuous operator is Phase 2 (NRT-over-cache in Phase 1); OOTB content makes the machinery useful |
| 3 | Can't ship data out | SOLID (strongest pillar) | Data-diode overclaim; OQ-DEPLOY-2(a) pre-launch hardening item; OCSF ≠ PII-free |
| 4 | IT watches OT | PARTIAL | OQ-C14-OCSF open lean; `prism_ot` extension is provisional |
| 5 | What devices exist? | PARTIAL / THIN for active-native; SOLID for passive | Reading B gated on OQ-C14-SAFETY-LIABILITY (legal) + two engineering open questions; Reading C (passive) is decided and safe |
| 6 | OT context | SOLID (architecturally) | GraphRAG Phase-2 phased; mandatory-citation gating is "best achievable, not a proof" |
| 7 | Talent gap | PARTIAL | v1 is recommend-only; "augmentation" must be stated plainly, not elided |

### Five binding caveats (from the pressure-test)

1. **OOTB content empty:** Problems #1 and #2 rest on a content-empty detection engine until E-DETECTION-CONTENT-001 ships. The cost-model and tuning-machinery claims are uncontested; the "hunting productivity" claim requires content. Connector-egress (E-EGRESS-PIPELINE-001) is separately PROPOSED.

2. **Reading B gated:** Problem #5 native-active-discovery claim is blocked by OQ-C14-SAFETY-LIABILITY (legal/insurance — NOT engineering-resolvable before ship), OQ-C14-CADENCE-NUMBERS, and OQ-C14-PACKAGING. Do not headline native active OT discovery unconditioned. Reading C (passive) is safe and ungated — headline that path instead for standalone OT coverage.

3. **OT-OCSF schema open lean:** Problem #4 convergence abstraction is sound (P-ADS-08, INV-ADS-07, D-C14-3); the OCSF class assignments are OQ-C14-OCSF-pending. Claim the abstraction; do not claim a ratified OT-OCSF standard.

4. **Not data-diode:** Problem #3 air-gap claim must be precise. Outbound-dial mTLS + signed-bundle = decided and strong. True one-way data-diode = explicitly deferred (D-C2-4). Any claim beyond "outbound-dial / no inbound firewall rule required" and "signed-bundle air-gap delivery" is an overclaim.

5. **Recommend-only v1:** Problem #7 augmentation claim must be explicit. The autonomy ladder is designed; v1 is advisory-only. "The agent recommends; you decide" is the correct claim. "AI SOC analyst" or any framing implying autonomous action is out of bounds for v1.

6. **Continuous-operator phasing:** The streaming windowed correlation operator (C6-LEAN-2, Phase 2) is the single most expensive item in the collector space; Phase 1 delivers NRT-over-cache. A buyer could otherwise assume full streaming correlation ships near-term. The §17.9 strain must not be minimized.

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
| **Bitemporal cold-tier data-snapshot** | Deferred | OQ-C8-DATASNAPSHOT; the registry+catalog halves of bitemporality are decided; cold-tier snapshot is deferred |
| **Streaming windowed correlation operator (Phase 2)** | Phased | Phase 1 ships NRT-over-cache; Phase 2 native windowed operator is the single most expensive item, ordered later |
| **ML/behavior analytics online/learned tier** | Phased | Statistical tier day-2-first; online learning + poisoning-resistance tier is later-phase |
| **Passive OT sensor dissector breadth** | Build-stage | Full-packet PCAP + dissector is decided; §17.9 build-stage strain — protocol breadth is phased; encrypted OT = metadata-only |
| **Connector egress (E-EGRESS-PIPELINE-001)** | PROPOSED | Not a registered story; must not be headlined as a current cost benefit |

**Proposed resolution for §5.1:**
The human compares this candidate (problem-framed, D-C10-7-PF) against D-C10-5
(competitor-relative) and either: (a) picks one as the headline frame, (b) synthesizes
a hybrid (problem-framed entry with competitor-relative moat specificity), or
(c) splits them by audience (D-C10-5 for analyst/competitive context; this candidate
for buyer discovery/executive context). All three dispositions are valid; the human
decides at §5.1.

---

## §8 — Feature Map: Mandated-7 to Satisfying Day-2 Features

> **STATUS: APPENDIX — CANDIDATE ONLY.** This appendix maps DECIDED day-2 architecture
> to the mandated-7 customer problems for the benefit of the §5.1 brief-reframe review.
> It does NOT ratify any headline, change any decision, or constitute a live artifact.
> All maturity tags reflect the 2026-06-28 adversarial CONDITIONAL-PASS (§6).
>
> **Maturity tag legend:**
> - **[DECIDED]** — solid, settled decision with no blocking open questions
> - **[PARTIAL]** — decided with material caveat or phasing dependency
> - **[GATED]** — decided but blocked on open question(s) before it can ship or be claimed externally
> - **[PROPOSED]** — epic-level intent, not a registered story
>
> **Surface naming note (NAME-01):** Prism has four analyst surfaces: S1 (BYO-agent MCP surface),
> S2 (browser console / Central), S3 (conversational-canvas / embedded-AI surface — not Amazon S3),
> S4 (browser-extension IOC pivot). In customer/exec materials use descriptive names; reserve
> S1–S4 codes for engineer-altitude docs with a defining legend. When the egress storage target
> appears, write "Amazon S3" (never bare "S3") to eliminate the collision.

---

### §8.1 — Per-Problem Feature Enumeration

---

#### Problem 1 — Cheap threat hunts (cost / economics of hunting)

- Ephemeral federated query over live sensor APIs — no data lake, no per-GB ingest cost (C5, C8) **[DECIDED]**
- Demand-driven `RETAIN` caching → `RetentionCache` primitive: pay for hot storage only on repeated access patterns, "SIEM capability without store-everything cost" (C5 D-C5-2) **[DECIDED]**
- SIEM / Security-Lake / Iceberg cold-tier federation as a queryable source type — query-in-place over existing SIEM investments, no second copy (C5 D-C5-2) **[DECIDED]**
- "Model is the memory" — on-demand ML/behavior analytics tier (§15/§15.11, D-C7-1..4): long-horizon behavioral baselines (UEBA) at model-sized cost; no store-everything required for anomaly detection over months of history **[PARTIAL/PHASED — statistical tier day-2-first; learned/online tier later]**
- Cost-based-degrade join guard: per-side row caps, dynamic filtering, mandatory time-bound injection — query cost bounded structurally, not by analyst discipline (C3 D-C3-1, PAT-ADS-04); a STRENGTH vs competitor "translate-and-pray" (D-C10-1, D-C10-3 "do NOT concede") **[DECIDED]**
- NL→PrismQL translation reusing parser/planner diagnostics as validate-repair signal; single LSP server across Monaco/CLI/agent — lowers authoring labor, tier-1 analyst authors tier-3 logic (C8 L-C8-4, L-C8-3) **[DECIDED]**
- Connector egress to cheap customer lakes — write OCSF results out to customer-owned cold storage (Amazon S3, Azure Blob/ADLSv2, GCS) (C10 GAP-Q4, E-EGRESS-PIPELINE-001) **[PROPOSED]**

**Honest caveat:** The cost model is uncontested and architecturally solid. Hunt *content authoring* still costs analyst labor until the OOTB detection-content library (E-DETECTION-CONTENT-001) lands — the engine ships content-empty. Connector egress (E-EGRESS-PIPELINE-001) is additionally PROPOSED and must not be headlined as a current cost benefit.

---

#### Problem 2 — Tuning is hard (detection-engineering / false-positive burden)

- RBA as default: noisy events accrue risk scores on entities rather than being suppressed — the analyst sees the entity's story, not a flood of individual alerts (C6 D-C6-2) **[DECIDED]**
- Suppression-as-code: versioned, mandatory written justification, mandatory time-box expiry — suppressions are auditable and self-expiring **[DECIDED]**
- Auto-tune suggests-only: fire-frequency dashboard surfaces candidate suppressions; analyst approves every one **[DECIDED]**
- Auto-rollback: shadow→canary-auto / canary→production-human staged gates; CORROBORATION-MASTER-GATE distinguishes a real-attack spike from a broken rule before auto-rollback trips (C6 D-C6-3) **[DECIDED]**
- Per-tenant alert-rate circuit-breaker with exponential backoff on repeated trips (C6 D-C6-3) **[DECIDED]**
- Dual-tier backtest + mandatory coverage map: cold-tier-deterministic (snapshot-id + rule-version pair; reproducible) AND remote-best-effort (non-deterministic, labeled); ALWAYS with a mandatory coverage map distinguishing "evaluated, no match" from "no data to evaluate" — a decided, explicitly-novel correctness affordance every surveyed competitor (Elastic/Chronicle/Panther/Splunk) fails (D-C6-1) **[DECIDED]**
- Bitemporal AS OF KNOWN <T> replay (D-C8-2/D-C8-3): replay an investigation exactly as Prism saw it at decision-time — entity identity + schema interpretation pinned; Prism-novel differentiator, no surveyed commercial-tool equivalent **[DECIDED for registry+catalog halves; cold-tier data-snapshot deferred OQ-C8-DATASNAPSHOT]**
- CUSUM/ADWIN change detectors as shared primitive for canary trip signals (C9 D-C9-Q2-HEALTH; C6 cross-link) **[DECIDED]**
- MATCH_RECOGNIZE sequence/correlation + absence-as-timer operator: NFA-based event-sequence correlation native to PrismQL; expresses Sigma temporal/temporal_ordered correlations as first-class where other backends only approximate (C6 ADR-PROP-C6-LEAN-1, L-C6-3 Strategic alignment) **[DECIDED]**
- Rule translation Sigma/SPL/KQL/NL→PrismQL with fidelity report for lossy edges; SPL/KQL-trained analysts migrate existing rules with on-ramp (C6 ADR-PROP-C6-LEAN-3; E-RULE-XLATE-001) **[PARTIAL]** — inbound confirmed feasible; OUT direction deferred (E-RULE-XLATE-001-OUT)
- PCAP retrieve/query affordance: `retrieve-packets-by-session` + S2 console download — full packets on demand for forensics; no separate full-packet-capture appliance required (§17.6, E-COLLECTOR-PCAP-001) **[DECIDED]**
- PrismQL RECOMMEND projection (D-C15-4): detection recipe emits a typed recommendation as read-only DATA — perimeter-compile-fail-tested so the query language never gains a write path; authored declaratively in the same surface as detections **[DECIDED]**

**Honest caveat:** The continuous/streaming detection operator (C6 ADR-PROP-C6-LEAN-2) is PHASED — Phase 1 delivers NRT-over-cache; Phase 2 (true streaming windowed correlation) is the single most expensive, novel piece in C6, correctness entirely Prism's responsibility, ordered later. The OOTB rule library is the content gap that makes the tuning machinery useful.

---

#### Problem 3 — Can't ship data out (residency / air-gap / OT segmentation)

- D-C2-12 HARD INVARIANT: raw sensor data NEVER crosses the conduit; only OCSF-normalized results transit upward — structural, not configurable off (P-ADS-03, INV-ADS-01) **[DECIDED]**
- Edge-computes / Central-surfaces: compute and credentials live at the satellite; Central sees only derived, tenant-keyed results (P-ADS-05) **[DECIDED]**
- BYOC zero-access by construction: satellite-local credential resolution, operator infrastructure holds ciphertext only, cannot decrypt without tenant key (D-DEPLOY-005, PIV-DEPLOY-004, SS-26 per-tenant DEK, P-ADS-02/P-ADS-04) **[DECIDED]**
- Operator-zero-access-at-rest / Option-3 tenant-keyed Central persistence: all Central-cached derived results encrypted under tenant-held CMEK — operator blind by cryptographic construction (P-ADS-02, P-ADS-04) **[DECIDED]**
- Air-gap first-class reference profile: SoftwareKms CMEK backend, no internet requirement, embedded on-box inference (INV-ADS-08) **[DECIDED]**
- Outbound-dial mTLS mesh — satellite dials out to Central; no inbound firewall rules required at the OT site; Relay/Edge topology traverses Purdue layers (C2 D-C2-6) **[DECIDED]**
- Signed-bundle air-gap delivery for config, intel feeds, model weights, schema updates — same Ed25519/sigstore mechanism across all bundle types (C9 ADR-PROP-C9-1; PAT-ADS-03) **[DECIDED]**
- Fully self-contained storage stack — bundled PostgreSQL (control-plane only, NEVER external/cloud), embedded RocksDB + SQLite, object-store Iceberg — no external managed database to configure or expose; air-gap valid by construction (ADR-PROP-storage-engine-taxonomy.md, P-ADS-04) **[DECIDED]**
- C16 edge tokenizing clearing house: RSI-classified fields masked/tokenized immediately after OCSF normalization at the edge before transit to Central or the AI path (C16 D-C16-3) **[DECIDED]**
- C20 NERC CIP-011 BCSI entity-key zero-access: "CIP-deployable + CIP-evidence-generating" posture; BCSI as first RSI profile (C20, C16 D-C16-4) **[DECIDED]**
- Backup / recovery / erasure cluster (C17): crypto-shred GDPR erasure-by-key-destruction for pooled multi-tenant stores; tenant-held recovery key + optional M-of-N Shamir threshold escrow ("no unilateral operator access"); CIP-009 restore-test runs + integrity records + CIP-010 baseline diff as recovery evidence; DR tier ladder **[DECIDED]**
- Compliance profiles — five named presets (baseline / SOC2 / ISO 27001 / IEC-62443-OT / NERC-CIP): "CIP-deployable + CIP-evidence-generating" posture (D-PROF-1/-6, D-C20, PAT-ADS-12) **[DECIDED]**
- Passive read-only deployment reduces CIP classification weight by default: write/control paths are feature-flagged OFF so the operator consciously opts into heavier EACMS/BCS classification weight (D-C20-SF3) **[DECIDED]**

**Honest caveat:** This is the strongest pillar architecturally. Precision boundaries: (a) "outbound-dial mTLS + signed-bundle," NOT "data-diode" — true one-way diode is explicitly DEFERRED (C2 D-C2-4); (b) OCSF-normalized ≠ PII-free — OQ-DEPLOY-2(a) result-transit residency governance remains a pre-launch required hardening item (P-ADS-03 caution note); (c) backup/recovery: "no unilateral operator access" (D-C17-SF1), NOT "no access under any circumstance"; (d) compliance: "CIP-deployable + CIP-evidence-generating," NOT "CIP-certified"; (e) passive-read-only reduces CIP classification weight but is NOT a safe harbor — operator classifies per CIP-002 regardless.

---

#### Problem 4 — Want IT to watch OT (IT/OT convergence without OT-protocol expertise)

- OCSF normalization at the OT edge: OT-vendor output converted to IT-readable OCSF tables at the adapter boundary — IT analyst queries OT assets in PrismQL without any protocol expertise (P-ADS-08, INV-ADS-07) **[DECIDED]**
- Passive OT sensor — full-packet PCAP capture (§17.6, E-COLLECTOR-PCAP-001) + native Spicy-style protocol dissector (§17.12, E-DISSECTOR-NATIVE-001/OT-001) emitting OCSF Network Activity + native OT schema-on-read (Modbus/DNP3/S7/GOOSE/PROFINET); STRICT PASSIVITY (TAP/SPAN, never injects, §17.13): Prism sees OT network traffic without any OT-protocol expertise required of the IT analyst **[DECIDED, build-stage — §17.9 dissector breadth phased; encrypted OT = metadata-only]**
- OT asset, config, and vuln data modeled as first-class PrismQL-queryable OCSF source tables, normalized at the satellite boundary (C14 D-C14-4) **[PARTIAL]** — table model decided; OCSF class assignments pending OQ-C14-OCSF
- Single-pane Central console (S2): all query, alert, entity context across OT and IT in one surface — no analyst tool-switching (P-ADS-01) **[DECIDED]**
- Satellite mesh topology traverses Purdue layers — Relay/Edge chain reaches into OT L2/L3 without requiring network reachability from Central (C2 D-C2-3) **[DECIDED]**
- C18 RBAC + C19 nested tenancy scope IT→OT access with CIP-002 impact-level/entity/site boundary map as a first-class tenant-tree dimension **[DECIDED]**
- Dual-deployment three-operating-model matrix: MSSP-managed or client-managed Prism is deployed AT the OT site; SaaS-only customers retain the federated read path (D-DEPLOY-002) **[DECIDED]**

**Honest caveat:** The convergence abstraction is architecturally sound. The OCSF-OT SCHEMA substrate carries an open lean (OQ-C14-OCSF): OCSF has no ratified native OT classes as of 2026 (open proposal ocsf#1515). OT tables ship as a `prism_ot` private extension — a decided lean, not a ratified standard. Passive-sensor dissector breadth is phased (§17.9).

---

#### Problem 5 — What devices exist? (OT asset inventory / discovery)

- **Reading A (decided):** Federate existing OT-discovery platforms via northbound REST APIs — Industrial Defender `asmdataservice`, Nozomi OpenAPI, Claroty xDome API Explorer, Dragos web API, Armis Centrix (C14 D-C14-1) **[DECIDED]** — zero OT-protocol safety risk on this path; platform owns device-side collection
- **Reading C / Passive (decided, safe, ungated):** Full-packet PCAP capture + native Spicy-style protocol dissector (§17.6/§17.12/§17.13, E-COLLECTOR-PCAP-001/E-DISSECTOR-NATIVE-001) — passively discovers OT devices from network traffic (TAP/SPAN, never injects); same mechanism as incumbent OT security platforms; no OT-protocol safety gate applies since strict passivity IS the safety property **[DECIDED, build-stage — §17.9 dissector breadth phased]**
- PCAP retrieve/query affordance: on-demand packet retrieval by session (§17.6, E-COLLECTOR-PCAP-001) — forensic packet access without a separate full-packet-capture appliance **[DECIDED]**
- Active-query as a capability axis on the unified adapter interface (C3/C4) rather than a separate connector class — "active-query" is a descriptor dimension, not a new connector type (C14 D-C14-3) **[DECIDED]**
- C12 Entity-360 entity resolution merges discovered assets from multiple platforms into a single authoritative record per physical asset **[DECIDED]**
- **Reading B (gated):** Direct OT-protocol polling of field devices (Modbus/OPC-UA/DNP3/SNMP) as poller-of-last-resort for customers with NO OT platform (C14 D-C14-2, E-OT-PROTOCOL-CONNECTORS-001) **[GATED]** — blocked on OQ-C14-SAFETY-LIABILITY (legal/insurance — NOT an engineering-resolvable question), OQ-C14-CADENCE-NUMBERS (safe poll-cadence defaults; no published standards numbers), and OQ-C14-PACKAGING (WASM-compilability of Rust OT-protocol crates at morph)

**Honest caveat (biggest overclaim trap):** The correct external claim is "Prism federates your EXISTING OT-discovery platforms (Reading A) or passively captures your OT network traffic to discover devices without touching them (Reading C, TAP/SPAN only)." Do NOT claim "Prism discovers your OT" without this qualification. Reading B (direct field-device polling) is gated on three open questions including one legal/insurance question that is not engineering-resolvable before ship. Positioning must lead with Reading A and Reading C, qualify Reading B explicitly.

---

#### Problem 6 — Full OT environment context (relationships / zones / criticality / attacker-reachability)

- Two-layer embedded KG+vector: indradb (RocksDB-backed graph) for relationship traversal + usearch (hot ANN) + lancedb (cold on-disk) for semantic similarity — all embedded, air-gap capable (C12 D-C12-1, PAT-ADS-07) **[DECIDED]**
- Entity-360 seven-part view per entity: identity + attributes + neighbors + timeline + risk posture + anomalies + cited synthesized answer **[DECIDED]**
- Deterministic entity resolution auto-merge on strong identifiers (SID, UUID, stable asset UUID, MAC — D-C12-3); suspected-links with explicit confidence scores for fuzzy matches; hostname and IP are SUSPECTED-LINKS only, gated by security-reviewer sign-off (PIV-C12-4 — spoofable/rotatable, MUST NOT be auto-merged without validation); Purdue zone as a first-class entity attribute **[DECIDED]**
- Bitemporal AS OF KNOWN <T> replay (D-C8-2/D-C8-3): replay any investigation exactly as Prism saw it at decision-time — entity identity and schema interpretation pinned; Prism-novel differentiator, no surveyed commercial security tool (Chronicle/Sentinel/Splunk ES/ServiceNow CMDB) implements true Snodgrass bitemporality for entity resolution **[DECIDED for registry+catalog halves; cold-tier data-snapshot deferred OQ-C8-DATASNAPSHOT]**
- Passive OT sensor feeding OT network activity context: full-packet PCAP + native Spicy-style dissector (§17.6/§17.12) emitting OCSF Network Activity tables, passively populating OT device relationships and protocol context without active polling **[DECIDED, build-stage]**
- PCAP retrieve/query affordance: `retrieve-packets-by-session` + S2 console download — full forensic packets for session reconstruction (§17.6, E-COLLECTOR-PCAP-001) **[DECIDED]**
- On-demand ML / behavior analytics (§15/§15.11, D-C7-1..4): ANOMALY_SCORE, PROFILE…OVER, RARITY, FIRST_SEEN, PEER_OUTLIER as PrismQL primitives — behavioral baselining and anomaly detection on entity activity **[PARTIAL/PHASED — statistical tier day-2-first; online/learned tier later]**
- GraphRAG Phase-1 local-search (entity neighborhood retrieval + LLM synthesis + mandatory citations) ships first (C12 D-C12-5) **[DECIDED]**
- GraphRAG Phase-2 global community summarization (Leiden algorithm, campaign-level sense-making) committed but post-Phase-1 (C12 D-C12-5) **[PARTIAL]** — phased
- On-box embeddings via fastembed/ort (default) + candle pure-Rust fallback for air-gap audit compliance — no telemetry leaves the satellite for embedding (C12 D-C12-2, PAT-ADS-07 step 3) **[DECIDED]**
- Mandatory faithful citations on all AI-generated output: every factual claim cites the source OCSF event, detection rule, or asset record; Output Hardener validates before surfacing to the analyst (PAT-ADS-08, C12) **[DECIDED]**
- C11 CVE/KEV/EPSS/CVSS-v4/CSAF/VEX enrichment: feed-down + match-at-edge — advisory priority conditioned on OT zone, asset criticality, compensating controls (D-C11-1, D-C11-4) **[DECIDED]**

**Honest caveat:** "Cited entity context with confidence" is the correct claim — NOT "ground-truth topology." Faithfulness is enforced and measured (PAT-ADS-08) but the ~57%-unfaithful-RAG baseline means mandatory citations + post-hoc faithfulness scoring are "best achievable, not a proof." GraphRAG Phase-2 community/campaign sense-making is committed but ships after Phase-1 Entity-360. Hostname/IP are NEVER in the strong-ID auto-merge set (PIV-C12-4).

---

#### Problem 7 — Can't find talent (OT security skills shortage)

- The conversational-canvas / embedded-AI surface (Prism surface S3 — not Amazon S3): NL→PrismQL, guided investigation workflow, agent-native MCP-first surface (S1 BYO-agent MCP) from day one (C8 L-C8-4; S1/S3 surfaces) **[DECIDED]**
- Agent-native MCP-first product identity: built to be consumed by AI agents, not retrofitted (D-C10-2, agent-native identity) **[DECIDED]**
- Familiar KQL/Splunk-style piped query surface that compiles to the same engine plan as SQL — SPL/KQL-trained analysts productive immediately, no relearning required (C8 D-C8-1) **[DECIDED]**
- C15 ARO recommend-only v1: W3C-PROV provenance on every recommendation, calibrated confidence + conformal sets, per-citation faithfulness check, evidence package in the Observation layer (C15 D-C15-2, D-C15-5) **[DECIDED]**
- Autonomy ladder designed now — advisory→suggested→auto-with-approval→autonomous — with per-action-class, evidence-measured, REVERSIBLE promotion; OT/safety-critical actions are HITL-permanent by design (C15 D-C15-2) **[DECIDED]**
- PrismQL RECOMMEND projection (D-C15-4): detection recipe emits a typed recommendation as read-only DATA — perimeter-compile-fail-tested; authored declaratively in the same surface as detections **[DECIDED]**
- Evidence package: Investigation Report + Replayable Query Log + IOC Ledger + self-QA quality gate (evidence completeness, logic verification, missed-indicator, severity-calibration, blind-spot — analog to Query Workers' Senior Analyst Review; D-C10-4-Q2) (C10 GAP-Q2, E-EVIDENCE-PACKAGE-001) **[PROPOSED]** — not a registered story
- NL→PrismQL + LSP server: tier-1 analyst authors tier-3 detection queries; single LSP reused across Monaco/CLI/NL agent (C8 L-C8-3, L-C8-4) **[DECIDED]**
- Rule translation Sigma/SPL/KQL/NL→PrismQL with fidelity report; MATCH_RECOGNIZE expresses Sigma temporal correlations as first-class (L-C6-3) — an analyst's existing rule investment ports with a fidelity report for lossy edges **[PARTIAL; OUT deferred]**
- Four analyst surfaces: S1 (BYO-agent MCP surface), S2 (browser console / Central), S3 (conversational-canvas / embedded-AI surface — not Amazon S3), S4 (browser-extension IOC right-click pivot) **[DECIDED]**
- On-box pluggable model backends: candidate central/edge SLMs + a prompt-injection/moderation guardrail layer, final model picks pending benchmark (OQ-C15-4) — wasmtime wasi-nn for AI-opaque per-tenant model isolation (C15 D-C15-7, C7 D-C7-2) **[DECIDED architecture; [PARTIAL] model picks pending OQ-C15-4]**

**Honest caveat:** This is AUGMENTATION, not replacement. v1 is recommend-only — zero autonomous action. "A tier-1 analyst who knows IT can learn OT investigation with Prism" is the correct claim; "Prism replaces your OT analyst" is not. The autonomy ladder is designed; every rung above advisory requires per-action-class evidence requirements and CISA-aligned human-gate confirmation. Specific model names are candidates pending OQ-C15-4 benchmark; the pluggable ModelBackend architecture is [DECIDED].

---

#### Cross-cutting features serving all seven problems

Two features underpin every problem in the mandated-7 and are not specific to any one:

**(a) OCSF-normalize-at-the-edge (P-ADS-08, INV-ADS-07)** — the lingua franca behind federation economics (#1), detection tuning (#2), data-residency transit (#3), IT-readable OT tables (#4), asset context (#5/#6), and AI-consumable telemetry (#7). Without normalization at the boundary, each higher-order capability would require N×M coupling to N vendor schemas. The "no trusted-source exemption" rule is what makes this structural rather than aspirational.

**(b) The single PrismQL surface + shared LSP (C8)** — one query language across hunts (#1), detection rules (#2), NL authoring (#7), the AI agent (#7), and the Central console (#4). The LSP is compiled once and reused across Monaco, CLI, and the NL-to-PrismQL validator-repair loop. This single surface is what lets a tier-1 IT analyst author tier-3 OT detection logic — they learn one language, not one per vendor.

**(c) Enterprise identity + compliance posture (C18/C19/C20/ADR-PROP-sso-identity.md)** — OIDC + SAML 2.0 per-tenant SSO, SCIM 2.0 auto-prov/deprov, fine-grained RBAC down to source/column with decision-level audit, and five named compliance presets (baseline/SOC2/ISO27001/IEC-62443-OT/NERC-CIP). Query.io ships neither in-product RBAC nor documented SSO; this is a decided, competitor-beating cross-cutting differentiator for regulated/enterprise buyers. All CAPTURE-stage ("Prism plans; Query ships"). **[DECIDED architecture]**

---

### §8.2 — Capability-Coverage Matrix

**Legend:** **[D]** = DECIDED | **[P]** = PARTIAL | **[G]** = GATED | **[PR]** = PROPOSED | blank = not primary for this problem/cluster pairing

| Problem | Federated-query + RETAIN economics | Detection + tuning (C6) | Residency + air-gap mesh (C2/C9/Option-3) | OCSF-edge-normalization (P-ADS-08) | Passive OT sensor (§17.6/§17.12/§17.13) | OT active-query (C14 A+B) | Prism Context (C12) | Intel enrichment (C11) | AI agent + ARO (C15/S3) | RBAC + tenancy (C18/C19) | Backup/recovery/erasure (C17) | Compliance profiles (C20) | ML/behavior analytics (§15/§15.11) | Enterprise identity (SSO/SCIM) |
|---------|-----------------------------------|------------------------|------------------------------------------|-----------------------------------|------------------------------------------|--------------------------|--------------------|-----------------------|------------------------|--------------------------|-------------------------------|--------------------------|-------------------------------------|-------------------------------|
| #1 Cheap hunts | **[D]** | | **[D]** | **[D]** | | | | | | | | | **[P]** | |
| #2 Tuning is hard | **[D]** | **[D]** | | **[D]** | | | | | **[P]** | | | | **[P]** | |
| #3 Can't ship data out | | | **[D]** | **[D]** | | | | | | **[D]** | **[D]** | **[D]** | | |
| #4 IT watches OT | **[D]** | | **[D]** | **[D]** | **[D]** | **[P]** | **[D]** | | | **[D]** | | | | |
| #5 What devices exist? | **[D]** | | | **[D]** | **[D]** | **[D]+[G]** | **[D]** | | | | | | | |
| #6 Full OT context | | | **[D]** | **[D]** | **[D]** | **[P]** | **[D]** | **[D]** | **[D]** | | | | **[P]** | |
| #7 Can't find talent | **[D]** | **[D]** | | | | | **[D]** | | **[D]** | | | | | **[D]** |

The matrix shows every mandated problem is served by at least one **[D]** (DECIDED) feature-cluster; the **[G]**, **[PR]**, and **[P]** tags are the honesty gates from the 2026-06-28 adversarial CONDITIONAL-PASS (§6), not coverage gaps.

---

## Fold log (2026-06-28)

Applied in this pass from `positioning-fidelity-iterate-list.md`:

**Misalignment corrections (cheap [DECIDED] errors):**
- MISAL-01: §8.1 P7 — corrected "S4 mobile" → "S4 browser-extension IOC right-click pivot"
- MISAL-02: §8.1 P6 — corrected strong-ID auto-merge set to SID/UUID/stable-asset-UUID/MAC; moved hostname/IP to SUSPECTED-LINKS with PIV-C12-4 gating; removed hostname from auto-merge example; dropped SPIFFE SVID (C2 mesh identity, not C12 merge key)
- MISAL-05: §8.1 P7 — replaced "9-check self-QA gate" with "self-QA quality gate (evidence completeness, logic verification, missed-indicator, severity-calibration, blind-spot — analog to Query Workers' Senior Analyst Review)"

**Oversell corrections:**
- OVER-04: §8.1 P7 — re-tagged named model stack as [DECIDED architecture; [PARTIAL] model picks pending OQ-C15-4]; reworded to "candidate central/edge SLMs + guardrail layer, final picks pending benchmark"
- OVER-05: §1 — added formal-verification precision note scoping Kani proofs to VP-014/VP-015 parser-safety bounds, not "formally-verified query language/grammar"; added binding constraint to §5 concessions
- OVER-07: §8.1 P1 honest-caveat — added connector-egress (E-EGRESS-PIPELINE-001) as PROPOSED alongside OOTB-content gap

**Undersell / coverage-gap additions:**
- UNDER-01 (HIGH): §8.1 P4/P5/P6, §3 Pillar B, §8.2 matrix, §2 map rows — added passive OT sensor (full-packet PCAP + native Spicy-style dissector + STRICT PASSIVITY TAP/SPAN) as Reading C; added distinct §5 concession row separating passive-sensor (decided/safe/ungated) from active-polling (gated); carried §17.9 build-stage caveat
- GAP-07 (med): §8.1 P2/P5/P6 — added PCAP retrieve/query analyst affordance (retrieve-packets-by-session + S2 console download; no separate full-packet-capture appliance)
- UNDER-02 (HIGH): §8.1 P2/P6, §3 Pillar B/Pillar C, §2 map — added bitemporal AS OF KNOWN <T> replay as Prism-novel differentiator; tagged registry+catalog halves [DECIDED], cold-tier snapshot deferred; added to §7 open items
- UNDER-03 (HIGH): §8.1 P2, §3 Pillar C — expanded backtest bullet to surface coverage map and "evaluated, no-match vs no-data-to-evaluate" as the novel correctness affordance (D-C6-1); noted no surveyed competitor ships this
- UNDER-06 (med): §8.1 P2 caveat, §6 strength table — stated continuous-operator Phase 1/Phase 2 phasing explicitly; added Phase-2 as single most expensive item; added to §7 open items
- UNDER-07 (med): §8.1 P2 — expanded MATCH_RECOGNIZE bullet to note Sigma temporal/temporal_ordered correlation superiority vs other backends (L-C6-3); expanded rule-translation bullet to include SPL/KQL/NL alongside Sigma
- UNDER-09 (med): §3 Pillar C, §8.1 P1 — added cross-source join cost-safety (D-C10-1) as a STRENGTH explicitly; cited D-C10-3 "do NOT concede"
- UNDER-11 (med): §8.1 P7, §2 map row 7 — added KQL/SPL-style piped query surface (D-C8-1) as decided ergonomic on-ramp
- UNDER-12 (med): §3 Pillar A, §8.1 P3 — added fully self-contained storage stack (bundled PostgreSQL NEVER external/cloud + embedded RocksDB/SQLite + Iceberg) as concrete air-gap proof point
- UNDER-14 (low): §5 concessions, §8.1 P3, §3 Pillar A — added passive-read-only CIP classification benefit AND binding "not a safe harbor" caveat (D-C20-SF3); both required
- GAP-01 (HIGH): §3 Pillar A, §8.1 P3, §8.2 matrix, §7 — added C17 backup/recovery cluster (crypto-shred, tenant-held recovery key, M-of-N escrow, CIP-009 evidence, DR tier ladder); precise "no unilateral operator access" wording throughout
- GAP-02 (HIGH): §3 Pillar A, §8.1 P3, §8.2 matrix, §5 concessions, cross-cutting — added five named compliance presets (baseline/SOC2/ISO27001/IEC-62443-OT/NERC-CIP); "CIP-deployable + CIP-evidence-generating" NOT "CIP-certified"
- GAP-03 (HIGH): §3 Pillar C, §8.1 P1/P2/P6, §8.2 matrix, §2 map — added on-demand ML/behavior analytics cluster (ANOMALY_SCORE/PROFILE/RARITY/FIRST_SEEN/PEER_OUTLIER, "model is the memory" cost angle, online learning tier); tagged [PARTIAL/PHASED]; added to §7 open items
- GAP-04 (HIGH): §3 Pillar A, §8 cross-cutting, §8.2 matrix, §5 concessions — added enterprise identity cluster (OIDC+SAML2.0 per-tenant SSO, SCIM 2.0 auto-prov/deprov, fine-grained RBAC down to source/column, decision-level audit)
- GAP-05 (med): §3 Pillar C, §8.1 P2/P7 — added PrismQL RECOMMEND projection (D-C15-4) as read-only DATA emission from detection recipes, perimeter-compile-fail-tested

**Naming:**
- NAME-01: §8 preamble + §8.1 P7 + inline — added S3/Amazon S3 disambiguation on first use in §8; added "conversational-canvas / embedded-AI surface (Prism surface S3 — not Amazon S3)" gloss; wrote "Amazon S3" for storage/egress target throughout; added customer/exec surface-naming note

**Not applied in this pass (scope boundary):**
- OVER-01, OVER-02: exec-narrative residency/operating-model precision — exec-narrative is a separate artifact; folds apply there, not here
- OVER-03: exec-narrative auto-rollback framing — exec-narrative scope
- OVER-06: A2A diagram — diagram scope; no prose fold needed here
- MISAL-03, MISAL-04: diagram taxonomy and prose-diagram reconciliation — diagram scope (batch with §5.1-gated pass)
- UNDER-04, UNDER-05, UNDER-08, UNDER-10, UNDER-13: exec-narrative / competitive-positioning scope; some covered implicitly by cross-cutting additions here
- NAME-02, NAME-03: diagram scope
