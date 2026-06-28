---
document_type: iterate-list
status: working
do_not_execute: true
iterate_later: true
produced_by: "positioning-fidelity-audit workflow (8 agents) + architect capture"
timestamp: "2026-06-28"
gated_on: "§5.1 brief-reframe (most folds gated; exec-narrative oversell fixes recommended BEFORE sign-off)"
provenance: "out-of-band side-analysis; master iterate-list from the 2026-06-28 fresh-context multi-agent positioning-fidelity sweep; touches-no-live-artifacts"
---

# Positioning Fidelity Iterate-List

Master iterate-list from the 2026-06-28 fresh-context multi-agent positioning-fidelity sweep (8 agents, corpus C1–C20 + §17). This is the consolidated backlog for reconciling the positioning artifacts — ADR-PROP-positioning-problem-framed.md, ADR-PROP-competitive-positioning.md (D-C10-5), positioning-executive-narrative.md, the §8 feature-map, and both diagrams — with the C1–C20 + §17 corpus.

**This is NOT a decision document. Folds are applied later — most gated on §5.1 brief-reframe sign-off. Do not execute.**

Cross-links:
- `prism-as-ot-sensor-note.md` — passive OT sensor §5.1 gating rationale
- `ADR-PROP-positioning-problem-framed.md` — candidate positioning; primary affected artifact
- `positioning-executive-narrative.md` — highest-risk customer-facing surface
- `diagrams/` — prism-architecture-conceptual.drawio + prism-architecture-technical.drawio
- `consistency-audit-2026-06-27.md` — earlier corpus-internal consistency audit; this sweep is the complementary corpus→positioning fidelity axis

---

## Fidelity Verdict and Counts

**FIDELITY VERDICT: MEDIUM-to-HIGH drift** between the day-2 corpus decisions and the positioning artifacts, concentrated in two patterns:

1. Decided high-value capabilities with NO customer/exec home — passive-OT sensor, backup/recovery + crypto-shred, compliance profiles, on-demand ML, SSO/SCIM/RBAC, bitemporal replay.
2. The customer-facing exec narrative overselling absolute residency/operator-zero-access claims that the corpus deliberately scopes with caveats.

**Total: 36 entries** (35 from the 8-agent synthesis + GAP-07 added by architect capture).

NOTE: The workflow summary states "29 findings after dedupe" — this was an arithmetic slip in the summary prose. The workflow's own category breakdown sums to 35 (14 + 6 + 7 + 5 + 3), and the true total including GAP-07 is 36.

| Category | Count | HIGH | MEDIUM | LOW |
|----------|-------|------|--------|-----|
| Undersell | 14 | 5 | 7 | 2 |
| Coverage-gap | 7 (incl. GAP-07) | 4 | 3 | 0 |
| Oversell | 7 | 2 | 4 | 1 |
| Misalignment | 5 | 1 | 3 | 1 |
| Naming | 3 | 0 | 3 | 0 |
| **TOTAL** | **36** | **12 + GAP-07 (medium)** | **20** | **4** |

**Severity rollup: HIGH = 12, MEDIUM = 20 (incl. GAP-07), LOW = 4.**

---

## Top Priorities

1. **UNDER-01 (HIGH):** Fold the DECIDED passive-OT-sensor path (PCAP §17.6 + native dissector §17.12 + strict-passivity TAP/SPAN §17.13) into Pillar B / Problems 4-5-6 and exec Theme B, and stop framing OT-standalone solely on the GATED Reading B. Reconcile with the diagrams in the same pass (MISAL-04). This is the single strongest mis-told story and the explicitly-flagged known case. Gate: after §5.1 sign-off.

2. **OVER-01 + OVER-02 (HIGH x2):** Correct the exec narrative's absolute residency/zero-access claims BEFORE the §5.1 brief-reframe — distinguish RAW (never leaves site) from DERIVED OCSF results (which DO transit and may persist at Central, carry PII, OQ-DEPLOY-2(a) pre-launch), and qualify "operator cannot read your data" by operating model (absolute only in self-hosted/air-gap; audited/revocable in MSSP-managed). These are buyer-facing claims that will not survive a follow-up question.

3. **GAP-02 + GAP-04 (HIGH):** Give compliance posture (SOC2/ISO27001/IEC-62443/NERC-CIP presets, CIP-deployable + evidence-generating) and enterprise identity (OIDC/SAML/SCIM + fine-grained RBAC) a customer-facing home — these are decided, competitor-beating (Query.io ships neither RBAC nor documented SSO), and directly aligned with the narrative's named utilities/critical-infra/MSSP buyer.

4. **GAP-01 (HIGH):** Add the C17 backup/recovery cluster — crypto-shred GDPR erasure-by-key-destruction, tenant-held recovery key + M-of-N escrow ("no unilateral operator access", not "no access ever"), CIP-009 recovery-evidence — to the feature map and add a Backup/Recovery/DR box to the technical diagram. Decided 2026-06-27 with zero positioning footprint.

5. **UNDER-02 + UNDER-04 + UNDER-05 (HIGH):** Surface the three decided prism-NOVEL / identity differentiators absent from customer prose — bitemporal AS-OF-KNOWN replay (no commercial-tool equivalent), RBAC depth vs Query's none, and agent-native/MCP-first product IDENTITY (the lead term of the recorded headline candidate, yet the exec narrative never mentions MCP/BYO-agent).

6. **NAME-01 (MEDIUM, but cheap + propagates):** Set the "S3" disambiguation convention (descriptive surface names; always "Amazon S3" for storage) in the §5.1 brief-reframe before any external derivative is authored — five-reader convergence and the collision already sits in a buyer-facing feature map.

7. **MISAL-01 + MISAL-02 (one HIGH + one MEDIUM):** Fix the two [DECIDED]-tagged factual errors in §8.1 before they propagate into the brief — S4 is the browser-extension IOC pivot (not "mobile"), and auto-merge strong-IDs are SID/UUID/asset-UUID/MAC (hostname is explicitly forbidden per PIV-C12-4, SPIFFE is not a C12 key). These are cheap one-line corrections of decided contradictions.

8. **GAP-07 (MEDIUM, cheap diagram+prose fold):** Add the PCAP retrieve/query affordance (on-demand analyst packet-retrieval flow) to both diagrams and positioning prose. Distinct from UNDER-01's passive-capture story and MISAL-04's diagram-vs-prose drift — GAP-07 is the retrieve/query path from packet store to analyst/S2 download. Cross-link: prism-as-ot-sensor-note.md.

---

## Gating and Sequencing

**(a) Fix BEFORE §5.1 sign-off (buyer-facing accuracy):**
- OVER-01, OVER-02 — the two HIGH exec-narrative oversell corrections (residency precision; operating-model spectrum)
- MISAL-01 — S4 surface mislabel (cheap one-line correction of a [DECIDED]-tagged factual error)
- MISAL-02 — hostname in strong-ID auto-merge set (cheap one-line correction; contradicts corpus invariant PIV-C12-4)

**(b) Feed into the §5.1 positioning decision (undersell/gap folds):**
- All UNDER-* and GAP-* folds give decided differentiators a customer home; they inform what the brief positions on. Most require §5.1 sign-off on the OT-standalone and agent-native framing before the brief is authored.

**(c) Diagram batch (after §5.1):**
- UNDER-01 + MISAL-04 (passive-OT prose + diagram reconciliation) — must land in the SAME pass
- GAP-07 (PCAP retrieve/query flow in diagrams + prose) — batch with the UNDER-01 diagram pass
- MISAL-03 (three-operating-model vs four-peer-items taxonomy fix)
- NAME-03 (RBA label correction in technical diagram)

**(d) UNDER-01 passive-OT fold gated on §5.1** per prism-as-ot-sensor-note.md — do not execute before sign-off.

**(e) Naming convention (NAME-01) set at §5.1 brief-reframe** before any external derivative is authored.

---

## Full Findings

---

### Undersell (14 entries — UNDER-01..14)

---

#### UNDER-01 — Passive-OT-sensor capability (PCAP + native dissector + strict passivity) absent from all positioning prose

- **Severity:** HIGH
- **Corpus citation:** matured-vision-day2-requirements.md §17.6 (DECIDED 2026-06-26 full-packet PCAP, E-COLLECTOR-PCAP-001), §17.12 (native Spicy-style dissector, E-DISSECTOR-NATIVE-001/OT-001), §17.13 (OT/ICS flagship + STRICT PASSIVITY TAP/SPAN, never injects); prism-as-ot-sensor-note.md §1/§2/§3/§4
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§2 map rows #4/#5, §3 Pillar B, §8.1 Problems 4/5/6, §8.2 matrix); positioning-executive-narrative.md (Theme A/B); plus the Pillar-B "honest boundary" line and §5 binding-concession row that reduce "standalone OT" to the GATED Reading B only
- **Description:** [MERGE: ingestion-ot coverage-gap + 3 misalignments + ui-surfaces-naming undersell — the KNOWN PCAP/passive-OT case; diagram-vs-prose axis split out as MISAL-04.] Both problem-framed and exec-narrative artifacts contain ZERO mention of the passive-OT-sensor capability (full-packet PCAP + native Spicy-style dissector + TAP/SPAN passive listen-and-dissect). §17.6/§17.12/§17.13 are all human-DECIDED 2026-06-26 and constitute exactly the incumbent (Claroty/Nozomi/Dragos) passive-monitoring mechanism — a SAFE, UNGATED standalone path (strict passivity is a hard invariant; no active-polling safety gate applies). The positioning frames the entire OT-standalone story on C14 Reading B (active polling, GATED) and Reading A (federate existing platforms), omitting the strongest non-gated wedge. Pillar B's honest boundary and the §5 concession row treat the GATED Reading B as the ONLY "do-it-yourself" OT path; exec Theme B tells a prospect with no OT platform their only Prism option is a capability "approached carefully" — directly misaligned with decided §17 scope. The §8 feature enumeration never lists the PCAP collector or native dissector as features for problems #4/#5/#6.
- **Recommended fold:** Add a DECIDED feature line to §8.1 Problems #4/#5/#6 + matrix rows: "Passive OT sensor — full-packet PCAP capture (§17.6) + native Spicy-style protocol dissector (§17.12) emitting OCSF Network Activity 4001 + native OT schema-on-read (Modbus/DNP3/S7/GOOSE/PROFINET); STRICT PASSIVITY (TAP/SPAN, never injects, §17.13)." Add a "Reading C / Passive" path to Pillar B / Problem 5, contrasting it with the GATED Reading B. Add a second sentence to the Pillar-B honest boundary and a distinct §5 concession row separating passive-sensor (decided/safe/ungated) from active-polling (gated on OQ-C14-SAFETY-LIABILITY). Revise exec Theme B to "Federate the OT sensors you already have — or let Prism be your OT sensor, passively, the same way the established tools work (TAP/SPAN listen-and-dissect) — with active device polling as the carefully-gated frontier." Carry §17.9 build-stage caveat (day-2 planned, heaviest dissector build, protocol breadth phased, encrypted-OT = metadata-only). Gate: only after §5.1 sign-off.

---

#### UNDER-02 — Bitemporal forensic reproducibility (AS OF KNOWN <T>) absent from customer/exec home

- **Severity:** HIGH
- **Corpus citation:** D-C8-2 + D-C8-3 (ADR-PROP-prismql-deliverables.md): bitemporal `AS OF KNOWN <T>` registry+catalog pinning — flagged "Prism-novel differentiator — No surveyed commercial security tool (Chronicle, Sentinel, Splunk ES, ServiceNow CMDB) implements true Snodgrass bitemporality for entity resolution … a DFIR prior-art gap prism closes systematically"
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§2 mandated-7 map, §8 feature map Problems #1/#2/#6, §3 Pillar C); positioning-executive-narrative.md; both diagrams
- **Description:** Bitemporal forensic reproducibility (`AS OF KNOWN <T>` — "what did we know, and what was true, as of T") is a corpus-DECIDED, explicitly prism-NOVEL differentiator with NO surveyed commercial-tool equivalent, directly serving forensic/saved-findings replay (core DFIR value). It appears only as a tiny grammar token in the technical diagram ("Bitemporality: AS OF KNOWN <T>"); absent from §2/§8 feature enumerations for problems #1/#2/#6, absent from Pillar C, absent from the executive narrative. The single largest undersell in the query-federation area: a decided, prism-only differentiator with no customer/exec home.
- **Recommended fold:** Add a DECIDED feature line under Problem #2 (tuning/forensics) and Problem #6 (context/replay) in §8, and a Pillar C / exec-narrative sentence: "Replay any past investigation exactly as Prism saw it at decision-time — entity identity and schema interpretation pinned to that moment (AS OF KNOWN <T> bitemporality), a capability no surveyed commercial security tool offers." Tag [DECIDED] for the entity-registry+catalog-version halves; caveat the cold-tier data-snapshot half as deferred (OQ-C8-DATASNAPSHOT).

---

#### UNDER-03 — Dual-tier backtest + mandatory coverage map absent from exec narrative; technical diagram incomplete

- **Severity:** HIGH
- **Corpus citation:** D-C6-1 (ADR-PROP-detection-engine-depth.md, Decision Ledger): cold-tier deterministic backtest (snapshot-id + rule-version, reproducible) + remote best-effort backtest, ALWAYS with mandatory coverage map distinguishing "evaluated, no match" from "no data to evaluate" — marked "genuinely novel; Prism builds it from scratch", "unprecedented in the prior art" (Elastic/Chronicle/Panther/Splunk all fail it)
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§8.1 Problem 2, §3 Pillar C); positioning-executive-narrative.md (Theme C); both diagrams (technical shows only "Iceberg / Deterministic backtests", omitting remote tier + coverage map)
- **Description:** D-C6-1 decides dual-tier backtest (deterministic cold-tier + remote best-effort) ALWAYS with a mandatory coverage map whose key affordance — "evaluated, no match" vs "no data to evaluate" — is the one thing the corpus marks every surveyed competitor failing at and that Prism builds from scratch. Problem-framed §8.1 captures backtest as a single bullet but never surfaces the coverage-map / no-data affordance; the exec narrative omits backtesting entirely; the technical diagram shows only the deterministic tier. A decided, explicitly-novel correctness differentiator understated to a one-liner / absent.
- **Recommended fold:** Add a §8.1 Problem-2 + Pillar C bullet naming the coverage map and the "evaluated, no-match vs no-data-to-evaluate" distinction as a decided, novel correctness affordance (cite D-C6-1). Add one plain-language exec-narrative Theme C sentence ("Prism tells you when a clean hunt means no threat vs. simply no data was available"). Update the technical-diagram backtest label to note dual-tier + coverage map.

---

#### UNDER-04 — Layered RBAC depth absent from all three pillars and customer-facing prose

- **Severity:** HIGH
- **Corpus citation:** D-C18 §4 + D-C18-1 (ADR-PROP-rbac-depth.md): "Query.io ships no in-product RBAC [VERIFIED-WEB] … C18 depth — per-connector/source/table/column scoping, hierarchy inheritance, approver-gated actions, central-authored/edge-enforced policy, decision-level audit — is a clean, defensible differentiator"; D-C10-3 honest-concessions; OQ-C10-1 RBAC "slated for QX-202X"
- **Affected artifacts:** positioning-executive-narrative.md (Why This Is Different); ADR-PROP-positioning-problem-framed.md (§3 Pillars, §4 headlines)
- **Description:** Layered RBAC depth (RBAC+ReBAC+ABAC, per-connector/source/table/column scoping, decision-level audit) is one of the corpus's two cleanest competitor-relative differentiators — C10 research found Query.io ships NO in-product RBAC. Yet the customer/exec narrative never mentions access control, least-privilege, or auditability, and all three pillars (A/B/C) plus all three §4 headline candidates omit it. The §8.2 matrix lists "RBAC + tenancy (C18/C19)" only against problems #3/#4 and never surfaces in customer-facing prose. A decided, web-verified differentiator with a time-sensitive competitor gap given zero customer-facing weight.
- **Recommended fold:** Add an access-control/auditability value statement to exec-narrative "Why This Is Different" and Pillar A: "Every analyst action is scoped by role down to the individual data source and column, and every authorization decision is logged for audit — capabilities Query.io does not offer in-product." Keep the honest caveat that C18 is CAPTURE-stage ("Prism plans; Query ships", D-C10-3).

---

#### UNDER-05 — Agent-native/MCP-first product identity absent from exec narrative; conceptual diagram missing S1 surface

- **Severity:** HIGH
- **Corpus citation:** ADR-PROP-competitive-positioning.md D-C10-2 ("agent-native is the product IDENTITY") + D-C10-5 headline leads with "agent-native"; matured-vision §11.3 S1 ("MCP-native BYO agent … primary for power users") + Value-prop #5 amendment ("agent-native first, full browser console included"); diagrams/prism-architecture-technical.drawio actor_s1 ("S1 — BYO Agent: MCP client / Claude Code")
- **Affected artifacts:** positioning-executive-narrative.md (entire doc); diagrams/prism-architecture-conceptual.drawio (no S1/MCP/BYO-agent surface shown)
- **Description:** Agent-native (MCP-first, bring-your-own-agent / S1) is the corpus-DECLARED product IDENTITY and the lead term of headline candidate D-C10-5. The plain-language exec narrative never mentions MCP, bring-your-own-agent, or that an analyst's own AI agent (e.g. Claude Code) can drive Prism directly — it presents only an embedded "AI assistant". The conceptual diagram shows only the console and IT SOC analysts, omitting the S1 BYO-agent surface the technical diagram includes. Customer/exec materials under-tell the product's stated core identity.
- **Recommended fold:** Add a "Built for your AI agents" framing to the exec narrative: "Prism speaks MCP, so your team's AI agents can query it directly — and it also ships a built-in AI assistant for analysts who prefer the console." Add an S1 BYO-agent box/actor to the conceptual diagram so both diagrams agree and the identity is visible at exec altitude.

---

#### UNDER-06 — Streaming-operator phasing (Phase 1 NRT vs Phase 2 native windowed) not surfaced

- **Severity:** MEDIUM
- **Corpus citation:** matured-vision §17.7 (DECIDED phased continuous-operator: Phase 1 NRT-over-cache; Phase 2 native windowed operator = "single most expensive item in the collector space", ordered later) + §17.9 strain row "Streaming correlation state"
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§8.1 Problem 2 caveat, §6 caveat #2); positioning-executive-narrative.md (Theme C)
- **Description:** The §17.7 continuous-operator is decided but explicitly PHASED — Phase 1 reuses NRT-over-cache; Phase 2 (native windowed operator, watermarks/event-time/fault-tolerant state) is "the single most expensive item", ordered later. §8.1 acknowledges operator cost via the C6-LEAN-2 cross-link but does not surface the Phase-1-vs-Phase-2 split; the exec narrative omits it. A buyer could assume full streaming correlation is in the near-term tranche. §17.9 says this strain must "not be minimized".
- **Recommended fold:** In §8.1 Problem 2 + §6 caveat #2 state the phasing: "Real-time detection ships in two phases — Phase 1 near-real-time over a short-TTL cache (reuses detection-as-query); Phase 2 (true streaming windowed correlation) is the single most expensive item, ordered later." Add a one-line near-term-roadmap note in the exec narrative "Where We Are Today" section.

---

#### UNDER-07 — Full rule-translation matrix (SPL/KQL/NL + OUT direction) and Sigma temporal-correlation superiority absent

- **Severity:** MEDIUM
- **Corpus citation:** L-C6-3 (ADR-PROP-detection-engine-depth.md) + D-C10-4-Q1 (ADR-PROP-competitive-positioning.md): rule-translation matrix extended to NL/SPL/KQL→PrismQL inbound + OUT (E-RULE-XLATE-001); strategic point that Sigma temporal/temporal_ordered correlation rules are thinly supported across all backends and Prism's MATCH_RECOGNIZE makes them first-class
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§8.1 Problem 2, Problem 7); diagrams/prism-architecture-technical.drawio ("Sigma importer" only)
- **Description:** D-C10-4-Q1 decided full rule-translation matrix (Sigma/SPL/KQL/NL → PrismQL, inbound + OUT) plus OOTB PrismQL detection-content library (E-DETECTION-CONTENT-001). §8.1 Problem-2 captures "Sigma/SPL/KQL/NL→PrismQL" but the diagram shows only "Sigma importer" (omitting SPL/KQL/NL + OUT direction), and the strategic differentiator — Prism's MATCH_RECOGNIZE expresses Sigma temporal correlations as first-class where other backends only approximate (L-C6-3 Strategic alignment) — is never surfaced. A decided competitive edge (Splunk/Sentinel migration on-ramp + superior temporal-correlation fidelity) understated.
- **Recommended fold:** Update diagram "OOTB Content Library" box to "Sigma/SPL/KQL/NL → PrismQL translation (+ fidelity report)". Add a one-line §8.1 Problem-2/Pillar C note that Prism's MATCH_RECOGNIZE expresses Sigma temporal/temporal_ordered correlations as first-class where other backends approximate (cite L-C6-3). Keep [PARTIAL]/E-RULE-XLATE-001-OUT-deferred tag.

---

#### UNDER-08 — MSSP nested-tenancy value prop absent from exec narrative and pillars

- **Severity:** MEDIUM
- **Corpus citation:** ADR-PROP-nested-tenancy.md D-C19-1..9 (unlimited-depth nested tenancy: partner→sub-partner→customer→BU), D-C19-8 ("MSSP default key custody: CLIENT-HELD CMEK by default"), §4.5; ADR-PROP-dual-deployment.md D-DEPLOY-002 MSSP-managed multi-CLIENT tenancy
- **Affected artifacts:** positioning-executive-narrative.md (no MSSP/multi-client framing); ADR-PROP-positioning-problem-framed.md (§3 Pillars, no tenancy framing)
- **Description:** Nested/hierarchical multi-tenancy (the explicit MSSP operating model with per-child client-held CMEK by default) is a decided capability squarely aligned with the stated primary buyer (1898 & Co, an MSSP). The positioning is built around an end-customer "your data/your site" frame and never surfaces the MSSP-serving-many-clients value prop — that an MSSP can manage many isolated client tenants with cryptographic per-client isolation and client-held keys. A core selling point with no positioning home.
- **Recommended fold:** Add an MSSP-audience value thread (or short section) to the exec narrative: "For managed security providers: serve many clients from one deployment with cryptographic per-client isolation and client-held keys by default — your access to each client's data is scoped, audited, and revocable." Cite C19 nested-tenancy + client-held-CMEK-default. Honest caveat: CAPTURE-stage.

---

#### UNDER-09 — Cross-source join cost-safety framing absent from exec narrative and pillars

- **Severity:** MEDIUM
- **Corpus citation:** D-C10-1 + PAT-ADS-04 + §8.1 Problem-1: cost-based-degrade join guard is the CORRECTED framing; D-C10-3 lists it under "What NOT to concede" as a Prism STRENGTH ("safe, cost-guarded cross-source joins with plan-visible degradation") vs Query's "translate-and-pray"
- **Affected artifacts:** positioning-executive-narrative.md; ADR-PROP-positioning-problem-framed.md (§3 pillars / §4 headlines)
- **Description:** The corrected cross-source join framing (D-C10-1) is a corpus-designated STRENGTH explicitly flagged "do NOT concede." Problem-framed captures it only as one DECIDED line in §8.1 Problem-1; the exec narrative omits the cross-source federation-with-cost-safety story entirely. Federating safely across IT and OT (the core of the product) without runaway cost is a customer-relevant strength undersold to silence.
- **Recommended fold:** Add a plain-language line to exec-narrative Theme C cost section: "When a question spans multiple systems, Prism bounds the cost automatically and shows you exactly what it had to trim — no runaway queries, no silent gaps." Optionally a §3 Pillar C honest-strength note. Keep OQ-C10-4 / PIV-C3-1..3 caveat (committed-architecture, not yet shipping).

---

#### UNDER-10 — AI output-assurance rigor (W3C-PROV, calibrated confidence, per-citation faithfulness) absent from competitive artifact

- **Severity:** MEDIUM
- **Corpus citation:** D-C15-5 + PIV-C15-5 (ADR-PROP-soar-actions-aro.md): mandatory W3C-PROV provenance, calibrated confidence + conformal prediction sets, NISTIR-8312 explanation block, per-citation post-hoc faithfulness check via Output Hardener — "mandatory from day one"
- **Affected artifacts:** ADR-PROP-competitive-positioning.md (D-C10-5 headline + D-C10-4-Q2 evidence-package disposition + D-C10-3 concessions)
- **Description:** The competitive-positioning artifact is explicitly framed against Query.io's nine-check "Senior Analyst Review" (D-C10-4-Q2), yet its only C15/AI-trust differentiator is "credentials the AI never sees" (input-side) plus the proposed E-EVIDENCE-PACKAGE-001 gate. It omits decided output-assurance rigor in D-C15-5: mandatory W3C-PROV provenance, calibrated confidence + conformal prediction SETS, mandatory per-citation post-hoc faithfulness enforcement (the ~57%-unfaithful-RAG correctness gate). A decided, mandatory-day-one differentiator that out-rigors Query's heuristic gate, under-told in the competitor-relative artifact. (Problem-framed §8.1 P7 captures it — gap is specifically the competitive artifact + D-C10-5 expanded pitch.)
- **Recommended fold:** Add to D-C10-3 "what NOT to concede" / the expanded-pitch supporting points: "AI recommendation rigor — W3C-PROV provenance + calibrated confidence + conformal-prediction uncertainty sets + mandatory per-citation faithfulness enforcement (D-C15-5), required from day one — a structural output-assurance contract, not a heuristic review checklist." Position as the answer to Query's nine-check gate.

---

#### UNDER-11 — Piped-query sugar surface (KQL/SPL ergonomic on-ramp) absent from customer-facing positioning

- **Severity:** MEDIUM
- **Corpus citation:** D-C8-1 (ADR-PROP-prismql-deliverables.md): KQL/PRQL-style piped surface SHIPS day-2, desugars to the identical DataFusion logical plan, with mandatory "show desugared SQL / EXPLAIN"; §12.3 FSQL Ergonomics Parity Ledger
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§8 feature map #7 talent / #1 cost); positioning-executive-narrative.md
- **Description:** The piped sugar surface (explicit ergonomic on-ramp for SPL/KQL-trained analysts, decided day-2) has no home in customer/exec positioning. The talent-gap pillar (#7) and labor-reduction story lean entirely on NL→PrismQL + LSP, omitting the corpus's deliberate ergonomic answer for the large population of analysts already fluent in piped query languages. For a product positioning IT analysts to cover OT, "analysts can write in the familiar pipe style they already know" is directly relevant and dropped.
- **Recommended fold:** Add a [DECIDED] line under Problem #7 (cross-list #1) in §8: "Familiar KQL/Splunk-style piped query surface that compiles to the same engine plan as SQL — SPL/KQL-trained analysts are productive immediately (C8 D-C8-1)." Add a plain-language sentence to exec-narrative Theme C talent section.

---

#### UNDER-12 — Bundled-PostgreSQL (NEVER external/cloud) as concrete air-gap proof point absent from Pillar A / §8 P3

- **Severity:** MEDIUM
- **Corpus citation:** ADR-PROP-storage-engine-taxonomy.md (PostgreSQL row: "BUNDLED in the central appliance — NEVER external/cloud"; BUNDLED constraint non-negotiable: "preserves air-gap compatibility … self-sufficiency … no external managed DB endpoint"; Alternative B rejected)
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (Pillar A, §8 P3 air-gap items); positioning-executive-narrative.md (air-gap bullet)
- **Description:** A decided, load-bearing air-gap differentiator is understated: PostgreSQL is BUNDLED in the central appliance and NEVER external/cloud-managed (RDS/Cloud SQL/Azure), explicitly to preserve air-gap compatibility (Alternative B rejected on these grounds). The air-gap pillar leans on SoftwareKms CMEK + signed bundles + on-box inference but never states the ENTIRE storage stack (bundled Postgres + embedded RocksDB/SQLite + object-store Iceberg) is self-contained with zero external managed-DB dependency. A concrete procurement advantage for air-gapped/OT/utility buyers omitted.
- **Recommended fold:** Add a bullet to Pillar A / §8 P3 air-gap cluster: "Fully self-contained storage stack — bundled PostgreSQL (control-plane only, NEVER external/cloud), embedded RocksDB + SQLite, object-store Iceberg — no external managed database to configure or expose; air-gap valid by construction (ADR-PROP-storage-engine-taxonomy.md, P-ADS-04 boundary)." Tag [DECIDED].

---

#### UNDER-13 — On-box AI inference in air-gapped environments not connected to the air-gap story

- **Severity:** LOW
- **Corpus citation:** D-C15-7 + C7 D-C7-2 (on-box pluggable ModelBackend, candle pure-Rust, air-gap-safe inference); ADR-PROP-s3-agent-runtime.md Deployment Gating ("model_backend: local (vLLM/Ollama); no outbound internet required")
- **Affected artifacts:** positioning-executive-narrative.md (Theme C "On talent" + "Why This Is Different" AI bullet + "Works even when you're disconnected")
- **Description:** The exec narrative twice promises "works even when disconnected / fully offline air-gap" and separately describes an AI assistant, but never connects the two: it does not tell the air-gap buyer the AI/model ITSELF runs on-box (no outbound inference call). The decided capability (on-box pluggable ModelBackend, air-gap-safe inference) is precisely what makes "AI that works air-gapped" credible to OT/critical-infra buyers — the audience this narrative targets. Leaving it implicit understates the strongest air-gap-AI proof point.
- **Recommended fold:** In the air-gap bullet of "Why This Is Different" (and Theme C "On talent") add: "In a disconnected environment, the AI itself runs on your own hardware on-site — Prism uses self-hosted models so nothing about your environment is sent to an outside AI service." Decided posture; do not name specific models (see OVER-04 candidate caveat).

---

#### UNDER-14 — NERC-CIP passive-by-default classification benefit + matching caveat absent from positioning

- **Severity:** LOW
- **Corpus citation:** ADR-PROP-nerc-cip-support.md D-C20-SF3 (passive read-only edge = lighter CIP classification by default; write/control feature-flagged OFF) + caveat "Read-Only Is NOT a Safe Harbor" (PIV-C20-001 + CIP-002 reasonable-foreseeability)
- **Affected artifacts:** positioning-executive-narrative.md (Theme B device-discovery); ADR-PROP-positioning-problem-framed.md (§5 honest concessions)
- **Description:** C20 decides a buyer-relevant CIP posture: passive read-only by default keeps Prism at a lighter CIP classification; write/control paths are feature-flagged off so the operator consciously opts into heavier EACMS/BCS weight — a real risk-reduction selling point for CIP-regulated buyers, carrying a binding honest caveat ("read-only is not a safe harbor"). Neither value nor caveat appears in positioning. Undersells a differentiator and leaves a known honest-caveat absent from §5 concessions.
- **Recommended fold:** When the compliance/NERC-CIP value thread is added (see GAP-02), include the passive-by-default classification benefit AND fold the matching caveat into the honest-concessions table: "Passive read-only deployment reduces CIP classification weight; it does not by itself remove Prism from CIP scope — the operator still classifies per CIP-002" (mirrors D-C20-SF3).

---

### Coverage-Gap (7 entries — GAP-01..07)

---

#### GAP-01 — C17 backup/recovery (crypto-shred, tenant-held recovery key, M-of-N escrow, CIP-009 evidence) entirely absent from all positioning artifacts

- **Severity:** HIGH
- **Corpus citation:** ADR-PROP-backup-recovery.md (entire C17): D-C17-SF1 (tenant-held recovery key + M-of-N Shamir escrow, "no unilateral operator access"), crypto-shred (GDPR right-to-erasure for pooled stores), D-C17-SF4 (DR tier ladder), D-C17-CIP009 (restore-test runs + integrity records + CIP-010 baseline diff as recovery evidence), PIV-C17-001/003
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§2 map, §3 pillars, §8 feature map + §8.2 matrix); ADR-PROP-competitive-positioning.md (D-C10-3 concessions); positioning-executive-narrative.md; both diagrams
- **Description:** C17 backup & recovery is a fully DECIDED ADR (human-confirmed 2026-06-27) with high-value compliance capabilities — crypto-shred as GDPR right-to-erasure for pooled multi-tenant stores, tenant-held recovery keys with optional M-of-N threshold escrow (operator can never recover tenant data alone), full DR tier ladder, CIP-009 recovery-evidence generation. NONE appears in ANY positioning artifact (grep for backup/recovery/DR/restore/crypto-shred/escrow/RTO/RPO returns zero substantive hits; neither diagram has a backup/recovery/DR box). A decided capability with strong enterprise-procurement + compliance value with no home anywhere.
- **Recommended fold:** Add a backup/recovery/erasure cluster to the feature map + coverage matrix. Fold under Problem #3 (residency/compliance) the crypto-shred GDPR-erasure + CIP-009 recovery-evidence [DECIDED]; add tenant-held-recovery-key / M-of-N escrow "no unilateral operator access" to Pillar A. Exec narrative: "You can erase a tenant by destroying its key; we can never recover your data without your key; we generate recovery-test evidence for your auditors." Add a Backup/Recovery/DR box to the technical diagram. Carry D-C17-SF1 precise wording ("no unilateral operator access", NOT "no access under any circumstance") to avoid overselling the M-of-N tier.

---

#### GAP-02 — Compliance profiles (SOC2/ISO27001/IEC-62443/NERC-CIP presets) entirely absent from customer-facing narrative

- **Severity:** HIGH
- **Corpus citation:** ADR-PROP-compliance-profiles.md D-PROF-1/-6 + §3 five shipped presets (baseline/soc2/iso27001/iec-62443-ot/nerc-cip); PAT-ADS-12; ADR-PROP-nerc-cip-support.md D-C20 "CIP-deployable + CIP-evidence-generating"
- **Affected artifacts:** positioning-executive-narrative.md (all sections); ADR-PROP-positioning-problem-framed.md (§3 Pillars, §4 headlines, §8.1 per-problem prose)
- **Description:** The Compliance-Profile engine (five named presets including SOC2, ISO 27001, IEC-62443-OT, NERC-CIP) and the "CIP-deployable + CIP-evidence-generating" NERC-CIP posture are DECIDED, flagged in C18 §4 as satisfying "NERC CIP (CIP-004/005), SOC2 CC6.1/CC6.3, ISO 27001 Annex A.9" buyer requirements. The exec narrative explicitly names "utilities … critical infrastructure" as target buyer — the NERC-CIP audience — yet neither compliance profiles nor NERC-CIP appear in the customer-facing narrative or the three pillars. No positioning home for a decided, audience-aligned capability. (§8.2 matrix has a "Compliance profiles (C20)" column but it is an appendix, not customer-facing.)
- **Recommended fold:** Add to exec-narrative "Why This Is Different": "Built for regulated industries — ship with named compliance presets (SOC 2, ISO 27001, IEC-62443, NERC-CIP) and generate the audit evidence your CIP auditors ask for." Honest caveat per D-PROF / C20: presets DECIDED architecture, CAPTURE-stage; "CIP-deployable + CIP-evidence-generating" is accurate — NOT "CIP-certified" (no such certification exists for a tool, C20 §1).

---

#### GAP-03 — On-demand ML / behavior analytics (anomaly scoring, UEBA, "model is the memory") absent from detection/hunting framing

- **Severity:** HIGH
- **Corpus citation:** matured-vision-day2-requirements.md §15 (On-Demand ML & Behavior Analytics, HUMAN-CONFIRMED 2026-06-25) + §15.11 D-C7-1..D-C7-4 (DECIDED 2026-06-27); epics E-ML-ONDEMAND-001 / E-ML-ONLINE-001 / E-ML-PRIMITIVES-001
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§8.1 Problem 2 tuning, Problem 6); positioning-executive-narrative.md (all themes)
- **Description:** The entire on-demand ML / online-behavior-analytics set — anomaly/behavior scoring as PrismQL primitives (ANOMALY_SCORE, PROFILE…OVER, RARITY, FIRST_SEEN, PEER_OUTLIER), model-as-third-retention-tier ("model is the memory" for long-horizon UEBA without store-everything), online/streaming learning, dual-rate+quarantine poisoning resistance — is HUMAN-CONFIRMED decided with three epics, yet has essentially NO home in the positioning's detection/hunting framing. §8.1 Problem 2 lists only RBA/suppression/auto-rollback/MATCH_RECOGNIZE; Problem 6 covers entity context but not behavioral baselining. The technical diagram shows "On-box ML / Pluggable ModelBackend" but no prose ties ML to buyer problems. The "model is the memory" tradeoff-softening (long-memory baselines at model-sized cost) is exactly a #1-cost-economics differentiator and entirely absent.
- **Recommended fold:** Add an ML/behavior-analytics feature cluster to §8.1 (Problems 1, 2, 6) and a distinct §8.2 matrix column, citing §15 + §15.11 (D-C7-1..4) + the three E-ML-* epics with PARTIAL/PHASED tags (statistical tier day-2-first; learned/online tier later). Add one plain-language exec-narrative Theme C paragraph on behavior-baseline anomaly detection + "model remembers what the storage didn't" cost benefit, with caveat that the heavy/learned tier is later-phase.

---

#### GAP-04 — Enterprise identity (OIDC/SAML/SCIM + fine-grained RBAC) absent from customer-facing positioning

- **Severity:** HIGH
- **Corpus citation:** ADR-PROP-sso-identity.md (DECIDED: OIDC + SAML 2.0 per-tenant from day one; SCIM 2.0 RESOLVED_IN_SCOPE_2026-06-26; fine-grained RBAC G-12 "documented differentiator over Query.io 2-role model"); matured-vision §11.3 SSO bullet; ADR-PROP-competitive-positioning.md D-C10-3
- **Affected artifacts:** positioning-executive-narrative.md (no SSO/RBAC/SCIM); ADR-PROP-positioning-problem-framed.md (RBAC only inside C18/C19 matrix cell for Problems 3/4; SSO/SAML/OIDC/SCIM absent entirely)
- **Description:** Enterprise SSO (OIDC + SAML 2.0 per-tenant), SCIM 2.0 auto-provisioning/deprovisioning, and fine-grained RBAC are DECIDED day-2 capabilities and explicitly named MSSP/enterprise differentiators (Query.io does not document SSO and does not support RBAC). No home in either customer/exec-facing artifact. For MSSP buyers facing SOC 2 / ISO 27001 / procurement-checklist requirements (the exact buyer addressed), the SSO/SCIM/RBAC story is a concrete, competitor-beating differentiator silently dropped. NOTE: partially overlaps UNDER-04 (RBAC depth) — UNDER-04 is the competitor-relative RBAC-depth undersell; GAP-04 is the broader SSO+SCIM+RBAC enterprise-identity coverage gap. Kept both: distinct folds.
- **Recommended fold:** Add an enterprise-identity bullet to exec-narrative "Why This Is Different": "Plugs into your existing identity provider — Okta, Entra ID, ADFS — with automatic user provisioning and fine-grained roles, from day one." Add an SSO/RBAC/SCIM row to the problem-framed feature map (under Problem 7 or as a cross-cutting enterprise-readiness feature). Honest scoping per ADRs (SAML impl-path is implementation-time architect call; SCIM committed in-scope).

---

#### GAP-05 — Read-only PrismQL RECOMMEND projection (declarative recommendation authoring) absent from all positioning artifacts

- **Severity:** MEDIUM
- **Corpus citation:** D-C15-4 + PIV-C15-1 (ADR-PROP-soar-actions-aro.md): read-only PrismQL RECOMMEND projection, source-discriminated provenance, perimeter compile-fail-tested; declarative recommendation-authoring inside a saved detection recipe (C8)
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§2/§3 Pillar C/§8 feature map); ADR-PROP-competitive-positioning.md; positioning-executive-narrative.md
- **Description:** The corpus decided a genuinely distinctive capability: a read-only PrismQL `RECOMMEND` projection letting a detection query/saved recipe EMIT a typed Recommendation as DATA (never executing an action), enforced by a perimeter compile-fail test in the same family as the security-perimeter gates. This is the declarative authoring path for recommendations and is structurally what keeps the query language read-only while closing the ARO loop. It has NO home in ANY positioning artifact — every C15 mention is recommend-only/HITL gating, never the RECOMMEND-as-read-only-projection that is the architecturally clean differentiator (PrismQL expresses recommendations without ever gaining a write path).
- **Recommended fold:** Add a Pillar C / Problem-2 + Problem-7 bullet: "PrismQL `RECOMMEND` projection (D-C15-4): a detection recipe can emit a typed recommendation as read-only DATA — perimeter-compile-fail-tested so the query language never gains a write path; recommendations authored declaratively in the same surface as detections." Tag [DECIDED]. Keep read-only-perimeter framing as the trust point.

---

#### GAP-06 — Edge entity masking / RSI tokenizing clearing house absent from exec narrative and conceptual diagram

- **Severity:** MEDIUM
- **Corpus citation:** ADR-PROP-entity-masking.md D-C16-1/-3/-8/-10 (edge tokenizing clearing house: deterministic vaulted tokenization, FF1 FPE, redaction, NER; per-tenant token vault + DEK at edge; dual-index human-IR vs masked AI/RAG; agent path zero vault wiring); §5 regulatory cross-regime (GDPR/HIPAA/PCI/NERC-CIP-011-3)
- **Affected artifacts:** positioning-executive-narrative.md (entire doc, "Why This Is Different"); diagrams/prism-architecture-conceptual.drawio
- **Description:** C16 entity masking / RSI tokenizing clearing house is DECIDED and IS represented in problem-framed §8 P3 + technical diagram ("RSI Entity Masking [decided]"), but completely absent from the customer/exec narrative and the conceptual diagram. The narrative's residency/trust story stops at "encrypted with a key only you hold" and never conveys that regulated-sensitive fields (PII/BCSI) are masked/tokenized at the edge before transit AND that the AI/agent path is structurally never wired to the vault (the data analogue of AD-017). The strongest trust/privacy differentiator for the regulated verticals the narrative targets, satisfying GDPR/HIPAA/PCI/NERC-CIP custody — a non-technical buyer would not learn the AI never sees their sensitive data.
- **Recommended fold:** Add a "Your sensitive data is masked before the AI ever sees it" beat to exec-narrative "Why This Is Different": regulated identifiers (asset names, IPs, BCSI configs) are tokenized at the edge; the AI operates on masked surrogates and is never wired to reveal raw values; authorized humans reveal under audited RBAC. Add a masking/clearing-house node to the conceptual diagram on the trust-boundary path. Reference C16 regulatory cross-regime fit where compliance buyers are addressed.

---

#### GAP-07 — PCAP retrieve/query analyst affordance absent from both diagrams and positioning prose [ARCHITECT-IDENTIFIED]

- **Severity:** MEDIUM
- **Corpus citation:** matured-vision-day2-requirements.md §17.6 (E-COLLECTOR-PCAP-001: full-packet PCAP decided 2026-06-26); implied by the packet-store model — captured packets must be retrievable by session/flow to serve analyst forensics; S2 console "download PCAP" action affordance
- **Affected artifacts:** diagrams/prism-architecture-conceptual.drawio; diagrams/prism-architecture-technical.drawio (shows "PCAP Capture" box but no retrieve/query flow); ADR-PROP-positioning-problem-framed.md §8 + ADR-PROP-positioning-problem-framed.md §8 Problems #2/#5/#6; positioning-executive-narrative.md
- **Description:** §17.6 decides not just PCAP CAPTURE but the analyst-facing PCAP RETRIEVE/QUERY affordance — PrismQL `retrieve-packets-by-session` + the S2 console "download PCAP" action (E-COLLECTOR-PCAP-001). The technical diagram shows a "PCAP Capture" box but NEITHER diagram shows the retrieve/query flow (packet store → query → analyst/S2 download); positioning prose omits it entirely. This is distinct from UNDER-01 (which is the passive capture+dissect capability missing from prose) and MISAL-04 (diagram-vs-prose capture inconsistency) — GAP-07 is specifically the on-demand PACKET-RETRIEVAL affordance that makes passive capture useful to an analyst: the ability to pull back full packets for a session after the fact without a separate full-packet-capture appliance.
- **Recommended fold:** Add a "retrieve full packets on demand for forensics (no separate full-packet-capture appliance)" feature line to §8 Problems #2/#5/#6. Add a retrieve flow — packet store → PrismQL query → analyst/S2 download — to the technical diagram (and a plain-language forensics line in the exec narrative). Cross-link prism-as-ot-sensor-note.md. Batch this diagram fold with the UNDER-01 / MISAL-04 diagram pass (same §5.1-gated milestone).

---

### Oversell (7 entries — OVER-01..07)

---

#### OVER-01 — Exec narrative makes unqualified absolute "data never leaves your site" claim; OCSF derived results DO transit and carry PII

- **Severity:** HIGH
- **Corpus citation:** ADR-PROP-dual-deployment.md OQ-DEPLOY-2 GAP(a) (Option 3 LOCKED: OCSF-normalized results ARE persisted at Central under CMEK; data at-rest subject to Central geography even with CMEK; EU tenants may require EU-region Central; Status OPEN — pre-launch required); ADR-PROP-satellite-mesh.md D-C2-12 (OCSF-normalized ≠ PII-safe; carries hostnames/IPs/user accounts); ADR-PROP-entity-masking.md D-C16-4 / P-ADS-03
- **Affected artifacts:** positioning-executive-narrative.md (One-Line Value Statement; Theme A "What Prism does"; "Why This Is Different" first bullet)
- **Description:** The customer-facing exec narrative makes an unqualified absolute residency claim — "your data never leaves your site" (one-liner), "Your raw operational data never travels off your premises. Only the normalized, security-relevant results do, encrypted end-to-end with a key that only you hold" (Theme A), "Only the normalized, security-relevant answers travel" ("Why This Is Different"). The corpus is explicit that under LOCKED Option-3, OCSF-normalized DERIVED results ARE persisted at Central, carry first-class PII (hostnames/IPs/user accounts, D-C2-12), and CMEK does NOT resolve the data-jurisdiction/residency question (EU tenants may need EU-region Central). The problem-framed candidate correctly carries this caveat (line 122/363: "OCSF-normalized ≠ PII-free; OQ-DEPLOY-2(a) pre-launch required"), but the exec narrative drops it entirely, converting a precise architectural boundary into an absolute sales promise. "A key that only you hold" is also imprecise under M-of-N escrow (D-C17-SF1) where the operator may hold one Shamir share.
- **Recommended fold:** Tighten the exec narrative to distinguish RAW data (never leaves the site — true/structural) from DERIVED OCSF results (which DO transit and may persist at Central under your CMEK, and carry security-relevant identifiers). Mirror the problem-framed honest boundary: residency-of-results governance (OQ-DEPLOY-2(a)) is a deployment-topology concern (EU-region Central for EU tenants), not solved by encryption alone. Soften "a key that only you hold" to corpus-precise "no unilateral operator access" / "the operator cannot read your data" to stay accurate under both tenant-held-key default and M-of-N escrow.

---

#### OVER-02 — Exec narrative states unconditional "operator cannot read your data"; corpus is explicit this is a spectrum by operating model

- **Severity:** HIGH
- **Corpus citation:** ADR-PROP-nested-tenancy.md §4.1/§4.3/§4.4 ("MSSP-managed … Not cryptographic impossibility of analyst access — cryptographic proof that analyst access is authorized, attributable, and revocable"); P-ADS-02 clarification: operator-zero-access is a SPECTRUM, MSSP-managed is the "honest middle"; C19 §4.2 P2 "standard MSSP operating posture"
- **Affected artifacts:** diagrams/prism-architecture-conceptual.drawio ("Your-key-encrypted results (operator cannot read your data)"); positioning-executive-narrative.md ("encrypted end-to-end with a key that only you hold"; "operator cannot read your data" implied across all three models)
- **Description:** The conceptual diagram and exec narrative state operator-zero-access as a flat, unconditional guarantee ("operator cannot read your data", "a key that only you hold"). The corpus is explicit this is a SPECTRUM: cryptographically absolute only in client-managed/BYOC; in SaaS the vendor cannot decrypt WITHOUT the tenant key; in MSSP-managed it is specifically NOT cryptographic impossibility — authorized MSSP analysts get audited, mediated, RBAC-scoped access to the derived corpus under the client's DEK. Stating "operator cannot read your data" unconditionally contradicts the corpus's own honest MSSP framing and will not survive a buyer follow-up about how an MSSP analyst investigates their data. NOTE: same precision problem as the "key only you hold" clause in OVER-01; OVER-02 specifically addresses the operating-model SPECTRUM, OVER-01 the raw-vs-derived residency claim. Kept both.
- **Recommended fold:** Qualify the claim by operating model. Exec narrative + diagram: "In the self-hosted and air-gapped models, only you hold the key — the vendor cannot decrypt your data under any circumstance. In the managed model, MSSP analyst access to your data is authorized, audited, and revocable, never silent or unbounded." Mirror C19 §4.4 spectrum table in spirit.

---

#### OVER-03 — Exec narrative says Prism "can automatically roll it back"; the AUTO action is demote-to-shadow, not revert

- **Severity:** MEDIUM
- **Corpus citation:** D-C6-3 (ADR-PROP-detection-engine-depth.md): auto-rollback RESOLVED but residual pre-implementation items PIV-C6-RB-1..9; AUTO action is DEMOTE-TO-SHADOW (rule keeps evaluating, routing stops); REVERT-TO-PRIOR-VERSION is one-click HUMAN action; FULL-DISABLE needs human sign-off; CORROBORATION-MASTER-GATE "the novel, hardest piece … no vendor ships"; status do_not_execute / CAPTURE-stage
- **Affected artifacts:** positioning-executive-narrative.md ("Prism can automatically roll it back to its last known-good version before it causes harm"); diagrams/prism-architecture-technical.drawio (Auto-rollback [decided] label)
- **Description:** The exec narrative states Prism "can automatically roll it back to its last known-good version" — but D-C6-3 AUTO action is DEMOTE-TO-SHADOW (rule keeps evaluating, routing stops); REVERT-TO-PRIOR-VERSION is explicitly a one-click HUMAN action, never auto-triggered. So "automatically roll it back to last known-good version" misstates the decided behavior (auto-demote, not auto-revert) and overstates autonomy. Separately the diagram tags auto-rollback "[decided]" with no caveat while the corpus flags it assembled-from-primitives with the novel CORROBORATION-MASTER-GATE "the novel, hardest piece" that "no vendor ships", whole doc CAPTURE-stage / do_not_execute. The §8.1 problem-framed map handles this correctly — exec narrative + diagram are the drift points.
- **Recommended fold:** Correct the exec-narrative sentence to: "If a detection rule starts misbehaving, Prism can automatically take it out of the alert stream while it keeps watching in the background — and a human decides whether to revert it." Matches D-C6-3 demote-to-shadow (auto) vs revert (human). On the diagram, keep "[decided]" but reference the CORROBORATION gate as the trip discriminator.

---

#### OVER-04 — Named model picks tagged [DECIDED] in §8.1 P7; corpus explicitly says these are CANDIDATES pending OQ-C15-4 benchmark

- **Severity:** MEDIUM
- **Corpus citation:** D-C15-7 (ADR-PROP-soar-actions-aro.md): "These are CANDIDATES pending the OQ-C15-4 benchmark; candidates are not locked final picks" + "NOTE — Flagged uncertainty: Llama-4 specifics … UNCONFIRMED as of 2026-06-27"; mirrored in C7 OQ-C7-1/PIV-C7-1 (ort still RC)
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§8.1 Problem 7, line 415)
- **Description:** The §8.1 Problem-7 bullet lists the on-prem model stack — "Qwen3/Mistral-class at Central, Phi-4-mini/Ministral-class at Edge; Llama Prompt Guard 2 + Mistral Moderation as guardrails; wasmtime wasi-nn" — and tags the whole line [DECIDED], naming specific models as if pinned. The corpus explicitly says these picks are CANDIDATES pending OQ-C15-4 (latency/quality/memory/WASM-vs-ort) and Llama-4 lineage was UNCONFIRMED. The architecture (pluggable AI-opaque ModelBackend, on-box/air-gap inference, guardrail layer) is decided; the named model PICKS are not. Tagging named models [DECIDED] contradicts the corpus's own flagged-uncertainty caveat.
- **Recommended fold:** Re-tag: the pluggable on-prem ModelBackend + guardrail-classifier-layer + on-box/air-gap inference posture is [DECIDED]; specific model names are [PARTIAL] pending OQ-C15-4. Reword to "on-box pluggable model backends (candidate central/edge SLMs + a prompt-injection/moderation guardrail layer), final model picks pending benchmark" rather than asserting pinned models as decided.

---

#### OVER-05 — "Formally-verified query language" / "Kani-verified grammar" overstates the scope of what is formally proven

- **Severity:** MEDIUM
- **Corpus citation:** C8 formal/Kani: D-C10-5 and D-C10-3 cite "formally-verified PrismQL (Kani VPs)" / "Kani-verified grammar". Live VP coverage (CLAUDE.md / VP-INDEX): VP-014/VP-015 (size + depth limits) — bounded parser-safety proofs, NOT an end-to-end "formally-verified query language"
- **Affected artifacts:** ADR-PROP-competitive-positioning.md (D-C10-5 "formally-verified query language"; D-C10-3 "formal PrismQL (Kani-verified grammar)"); inherited into ADR-PROP-positioning-problem-framed.md §1 by reference
- **Description:** The phrases "formally-verified query language" and "Kani-verified grammar" overstate the scope of what is formally verified. The corpus C8 deliverables describe formal/Kani at the level of specific verification properties (parser size/depth bounds), not a verified grammar or whole query language. As written, the claim invites a reading that PrismQL's semantics are formally proven, which the corpus does not support. D-C10-5 already soft-excludes this from the headline "for brevity" but the supporting-pitch framing still asserts "formally-verified query language" without scoping the proof.
- **Recommended fold:** Scope the claim wherever it appears: "formally-verified parser-safety properties (Kani proofs bounding query size/recursion depth)" rather than "formally-verified query language" / "Kani-verified grammar." Add a D-C10-3 honest-concessions scoping note so derived artifacts inherit the precise claim.

---

#### OVER-06 — Technical diagram's A2A box has no grounding in the authoritative transport ADR (C1-D1 decides MCP only)

- **Severity:** MEDIUM
- **Corpus citation:** ADR-PROP-competitive-positioning.md D-C10-4-Q3 (A2A "ADDRESS — ADDED TO DAY-2", E-A2A-TRANSPORT-001, "PROPOSED", spec-version pin OQ-C10-5 unresolved, "early standard … protocol stability risk are real"); ADR-PROP-central-deployment-access-layer.md C1-D1 (transport: MCP Streamable HTTP + stdio ONLY, no A2A); diagrams/prism-architecture-technical.drawio cen_a2a [roadmap]
- **Affected artifacts:** diagrams/prism-architecture-technical.drawio (cen_a2a) vs ADR-PROP-central-deployment-access-layer.md (authoritative C1 transport ADR)
- **Description:** A2A transport is a PROPOSED epic (E-A2A-TRANSPORT-001) added at C10, with unresolved spec-version pin and acknowledged early-standard risk. The authoritative C1 transport decision (C1-D1) decides MCP Streamable HTTP + stdio only — no A2A. The technical diagram correctly tags A2A [roadmap] (good), but there is a coverage/misalignment gap: the transport ADR the diagram derives from has no A2A note, so the diagram introduces a transport-surface box with no grounding in its own transport ADR. Risk: a downstream reader treats A2A as part of the decided transport layer.
- **Recommended fold:** (a) Add a one-line scope-addition note to ADR-PROP-central-deployment-access-layer.md C1-D1 referencing E-A2A-TRANSPORT-001 as a PROPOSED additive protocol (the competitive-positioning Ripple-Effects table already calls for this morph-time note), so the diagram's [roadmap] A2A box has a home; (b) keep the [roadmap] tag and ensure no positioning prose claims dual-protocol MCP+A2A as decided/shipping.

---

#### OVER-07 — Exec narrative cost framing implies connector-egress value without the PROPOSED roadmap caveat

- **Severity:** LOW
- **Corpus citation:** D-C10-4-Q4 (ADR-PROP-competitive-positioning.md): connector-egress is PROPOSED (E-EGRESS-PIPELINE-001). §8.1 Problem-1 correctly tags it [PROPOSED], but the Problem-1 honest-caveat names only the OOTB-content gap; exec narrative Theme C "On cost" implies write-out-to-cheap-storage value without the roadmap caveat
- **Affected artifacts:** positioning-executive-narrative.md (Theme C cost section); ADR-PROP-positioning-problem-framed.md (§8.1 Problem-1 caveat)
- **Description:** Connector-egress (OCSF gold-data OUT to customer-owned cheap storage) is correctly tagged [PROPOSED] in the §8 capability bullet, but the Problem #1 honest-caveat enumerates only the OOTB-content gap and does not flag egress as roadmap. The exec narrative's cost framing ("you pay for what you query, not a warehouse") is accurate for the ephemeral model but does not distinguish decided ephemeral-federation economics from the PROPOSED egress-to-cheap-lake capability. Minor, because the bullet itself carries [PROPOSED].
- **Recommended fold:** Extend the §8.1 Problem-1 honest-caveat to name egress as PROPOSED alongside the OOTB-content gap. In the exec narrative, keep egress out of the cost-today claim or qualify it as roadmap.

---

### Misalignment (5 entries — MISAL-01..05)

---

#### MISAL-01 — S4 surface labeled "mobile" in §8.1 P7; corpus decides S4 = browser extension IOC pivot

- **Severity:** HIGH
- **Corpus citation:** matured-vision-day2-requirements.md §11.3 four-surfaces table (S4 = "Browser extension (IOC right-click pivot)") + §11.3.1; corroborated by day2-ui-design/mockups/S4-01-extension.html + mockups/README.md line 234
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§8.1 Problem 7: "Four analyst surfaces S1–S4 (MCP agent / S2 Central console / S3 conversational canvas / S4 mobile)")
- **Description:** The problem-framed positioning doc labels S4 as "mobile". The corpus authoritatively defines S4 as the browser extension (IOC right-click federated pivot). "Mobile" is not a surface at all — the only mobile dimension is responsive/read-only triage breakpoints of the S2 console (a property of S2). Tagging this surface enumeration [DECIDED] while mislabeling one of the four decided surfaces is a fidelity defect that would propagate into the §5.1 brief-reframe.
- **Recommended fold:** Change "/ S4 mobile" to "/ S4 browser-extension IOC pivot" in §8.1 Problem 7. If responsive/mobile triage of S2 is worth surfacing, add it as a separate note that it is an S2 breakpoint mode, not surface S4.

---

#### MISAL-02 — Hostname listed as strong-ID auto-merge example; corpus explicitly forbids it per PIV-C12-4

- **Severity:** MEDIUM
- **Corpus citation:** D-C12-3 strong-ID table (ADR-PROP-prism-context.md: "SID, UUID, stable asset UUID, MAC address") + PIV-C12-4 ("IP addresses, hostnames … MUST NOT be in the strong-ID set unless … validated against a known-authoritative source")
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§8.1 Problem 6, line 395)
- **Description:** The §8.1 Problem-6 bullet states deterministic auto-merge happens "on strong identifiers (MAC, hostname, SPIFFE SVID)" tagged [DECIDED]. This contradicts the corpus: D-C12-3's canonical strong-ID set is SID/UUID/stable-asset-UUID/MAC, and PIV-C12-4 explicitly names "hostnames in some environments" as an identifier that MUST NOT be auto-merged without security-reviewer sign-off (spoofable/non-authoritative). "SPIFFE SVID" is not in the C12 strong-ID examples (SPIFFE appears as mesh/transport identity in C2, not a C12 auto-merge key). Listing hostname as a strong-ID auto-merge example undercuts the corpus's own catastrophic-over-merge caution and could mislead a buyer/auditor about how conservatively merges are made.
- **Recommended fold:** Correct to the canonical set: "strong identifiers (SID, UUID, stable asset UUID, MAC) auto-merge; spoofable/rotatable identifiers (IP, hostname) are suspected-links only and gated by security-reviewer sign-off (PIV-C12-4)". Remove hostname from the auto-merge example; drop or re-source SPIFFE SVID.

---

#### MISAL-03 — Both diagrams present air-gap as a fourth peer operating model; corpus decides exactly three operating models + air-gap as a profile

- **Severity:** MEDIUM
- **Corpus citation:** ADR-PROP-dual-deployment.md D-DEPLOY-002 ("THREE Named Operating Models": SaaS / MSSP-managed / client-managed); air-gap is a deployment PROFILE/shape (client-managed specifics: "This is the air-gap deployment shape"); ARCHITECTURE-DESIGN-SYSTEM.md P-ADS-11 lists air-gap as a profile dimension, not a fourth operating model; D-PROF-5 / PIV-PROF-2
- **Affected artifacts:** diagrams/prism-architecture-conceptual.drawio ("Runs as: SaaS | MSSP-managed | Self-hosted / on-site | Air-gapped"); diagrams/prism-architecture-technical.drawio ("Deployment Profiles: SaaS / MSSP-managed / Client-managed / Air-gapped")
- **Description:** Both diagrams present four peer "deployment" options. The corpus decides exactly THREE operating models on a WHO-HOSTS x WHO-OPERATES axis (D-DEPLOY-002); air-gap is NOT a fourth operating model — it is a deployment profile/shape overlaying client-managed (and reachable in other profiles per INV-ADS-08). Listing air-gap as a co-equal fourth "Runs as" item conflates the operating-model axis with the deployment-profile axis — the collapse D-PROF-5 / PIV-PROF-2 warns against. Low-stakes for a customer diagram but technically misaligned with the decided taxonomy.
- **Recommended fold:** Re-label to reflect the two-axis model: present three operating models (SaaS / MSSP-managed / Self-hosted) and show "air-gapped" as a profile applying across self-hosted (and on-prem), e.g. "Three operating models — SaaS, MSSP-managed, Self-hosted — with a fully air-gapped option for isolated environments." Aligns with D-DEPLOY-002 + P-ADS-11.

---

#### MISAL-04 — Diagrams already render passive-OT-sensor capability; prose positioning does not mention it — diagram-vs-prose drift

- **Severity:** MEDIUM
- **Corpus citation:** diagrams/prism-architecture-conceptual.drawio ("passively captures OT traffic", "Listens to your IT/OT network traffic") + diagrams/prism-architecture-technical.drawio ("PCAP Capture [decided]", "Native Protocol Dissector [decided] … STRICT PASSIVITY: TAP/SPAN, never injects") — grounded in §17.6/§17.12/§17.13; prism-as-ot-sensor-note.md provenance ("not yet folded into positioning")
- **Affected artifacts:** Cross-artifact: diagrams (conceptual + technical) vs ADR-PROP-positioning-problem-framed.md prose and positioning-executive-narrative.md prose
- **Description:** Internal inconsistency: the diagrams ALREADY render the passive-OT-sensor capability (PCAP Capture [decided], Native Protocol Dissector [decided], conceptual "passively captures OT traffic"), but the prose positioning artifacts the diagrams accompany do not mention it (see UNDER-01). The diagrams are ahead of the prose. A reviewer reading the narrative then the diagram hits an unexplained box that no prose pillar/feature/concession supports. The note says the passive-sensor capability has NOT yet been folded into positioning — yet the diagrams have folded it, creating diagram-vs-prose drift. This is the consistency-axis of UNDER-01; the fold must reconcile both.
- **Recommended fold:** When the §5.1-gated fold happens (UNDER-01), ensure the prose folds land in the SAME pass as the diagram labels so prose and diagram agree. Until then, either (a) carry the diagram's passive-capture boxes into the prose, or (b) add a diagram caveat note ("passive PCAP/dissector capability — DECIDED, fold pending §5.1") so maturity tags are consistent. Diagrams and narrative must not assert different capability inventories.

---

#### MISAL-05 — §8.1 Problem 7 pins "9-check" as Prism's self-QA count; D-C10-4-Q2 describes five Prism checks modeled on Query's nine-check gate

- **Severity:** LOW
- **Corpus citation:** D-C10-4-Q2 (ADR-PROP-competitive-positioning.md): self-QA quality-gate "Analog to Query's nine-check Senior Analyst Review" — enumerates FIVE named Prism checks (evidence completeness, logic verification, missed indicators, severity calibration, blind-spot), modeled on a nine-check gate; does NOT decide a nine-check Prism gate
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§8.1 Problem 7 "9-check self-QA gate" vs §2/§3 Pillar C/exec-narrative which say "self-QA gate" without count)
- **Description:** §8.1 Problem-7 pins the self-QA gate at "9-check", but D-C10-4-Q2 describes Prism's gate as an analog to Query's nine-check "Senior Analyst Review" and enumerates five Prism checks — it does not decide Prism's gate has nine checks. Stating "9-check" as Prism's own count borrows the competitor's number as a decided Prism spec. The exact check count is not a decided Prism quantity.
- **Recommended fold:** Change "9-check self-QA gate" in §8.1 Problem 7 to "self-QA quality gate (evidence completeness, logic verification, missed-indicator, severity-calibration, blind-spot — analog to Query Workers' Senior Analyst Review)". Drop the specific "9-check" count for Prism's own gate; cite D-C10-4-Q2.

---

### Naming (3 entries — NAME-01..03)

---

#### NAME-01 — "S3" token overloaded three ways in positioning corpus; collides with Amazon S3 in buyer-facing feature map

- **Severity:** MEDIUM
- **Corpus citation:** ADR-PROP-s3-agent-runtime.md (S3 = server-hosted conversational-canvas agent runtime / analyst surface S3); collides with Amazon S3 named as egress target in ADR-PROP-competitive-positioning.md D-C10-4-Q4 ("S3, Azure Blob/ADLSv2, GCS") and AWS Security Lake "S3 data-access subscriber binding" D-C10-4-Q5 / D-C5-2; E-EGRESS-PIPELINE-001
- **Affected artifacts:** ADR-PROP-positioning-problem-framed.md (§2 row 7, §8.1 Problem-1 egress line + Problem-7 "S3 agent" / "S3 conversational canvas" / surface "S3"); diagrams; P-ADS-07 conformance text uses "S3 agent". (Executive narrative correctly avoids it by saying "AI assistant".)
- **Description:** [MERGE: 5 of 7 readers flagged this same collision — the KNOWN S3-agent case. Severity raised from the low individual readings to MEDIUM on five-reader convergence + presence in a buyer-facing feature map.] The token "S3" is overloaded three ways in the same positioning corpus: (1) Amazon S3 the egress/Security-Lake storage target (D-C10-4-Q4/Q5, D-C5-2), (2) "S3 agent" = the embedded AI conversational-canvas agent runtime, (3) "S3" = analyst surface number 3 (S1–S4). In §8 the egress line ("write OCSF results out to … S3") sits one problem-cluster away from "S3 embedded AI agent", and P-ADS-07 conformance text writes "S3 agent". For a security/cloud audience this is a guaranteed misread — "S3 agent" parses as an Amazon-S3 connector. The exec narrative wisely uses "AI assistant"; the problem-framed ADR + diagrams + feature map reintroduce the internal surface code into prose that may feed customer/exec materials.
- **Recommended fold:** In any customer/exec-facing artifact, replace bare surface codes (S1/S2/S3/S4) with descriptive names ("embedded AI assistant / conversational canvas", "browser console", "BYO-agent MCP surface", "browser extension"); write "Amazon S3" (never bare "S3") for the storage/egress target. Gloss the surface taxonomy on first use in the problem-framed ADR (e.g., "the conversational-canvas surface (Prism surface S3, not Amazon S3)"). Reserve S1–S4 shorthand for engineer-altitude docs with a defining legend. Flag this naming-collision guard in the §5.1 brief-reframe so the convention is set before external materials are authored. Keep the exec narrative's "AI assistant" as canonical customer-facing term.

---

#### NAME-02 — S4 surface carries three inconsistent descriptors across positioning artifacts

- **Severity:** MEDIUM
- **Corpus citation:** matured-vision §11.3 (S4 = "IOC right-click pivot"); diagrams/prism-architecture-technical.drawio cell actor_s4 ("S4 — Browser Extension … Contextual enrichment overlay"); ADR-PROP-positioning-problem-framed.md §8.1 P7 ("S4 mobile")
- **Affected artifacts:** diagrams/prism-architecture-technical.drawio + ADR-PROP-positioning-problem-framed.md (vs matured-vision §11.3 canonical)
- **Description:** Surface S4 carries three inconsistent descriptions across the positioning set: "IOC right-click federated pivot" (canonical corpus), "Contextual enrichment overlay" (technical diagram actor label), and "mobile" (problem-framed §8.1). The technical-diagram label is defensible (the mockup includes an inline injected enrichment card) but drifts from the canonical IOC-pivot framing; the three labels together create cross-artifact ambiguity for the same surface. NOTE: the "mobile" mislabel is the [DECIDED] fidelity defect tracked as MISAL-01; NAME-02 is the separate cross-artifact descriptor-consistency issue (three different names for one surface).
- **Recommended fold:** Standardize S4's one-line descriptor across all artifacts to canonical "Browser extension — IOC right-click federated pivot (with inline enrichment card)". Update the technical-diagram actor_s4 cell and the problem-framed §8.1 line (resolving MISAL-01 at the same time).

---

#### NAME-03 — Technical diagram labels detection sub-box "Risk-Based Aggregation"; corpus term is "Risk-Based Alerting (RBA)"

- **Severity:** MEDIUM
- **Corpus citation:** D-C6-2 (ADR-PROP-detection-engine-depth.md): "Risk-Based Alerting (RBA) as the DEFAULT posture" (industry-standard Splunk RBA term); D-C6-3 (auto-rollback / staged-rollout circuit-breaker is a SEPARATE decided capability)
- **Affected artifacts:** diagrams/prism-architecture-technical.drawio (Detection Engine "Risk-Based Aggregation" box)
- **Description:** The technical diagram labels the detection-engine sub-box "Risk-Based Aggregation" and nests MATCH_RECOGNIZE, SEQUENCE…THEN, and Auto-rollback underneath. The corpus term is "Risk-Based Alerting (RBA)" (D-C6-2), an industry-standard term the positioning explicitly anchors to. "Risk-Based Aggregation" is a non-canonical rename that (a) loses the recognized RBA term security buyers know, and (b) mislabels the box — MATCH_RECOGNIZE / SEQUENCE / auto-rollback are NOT "risk-based aggregation"; they are the correlation operator and the staged-rollout circuit-breaker (D-C6-3), separate decided capabilities. The single box conflates three distinct corpus decisions under a coined term.
- **Recommended fold:** Rename to "Detection Engine (C6)" with sub-labels matching corpus terms: "Risk-Based Alerting (RBA, default)", "MATCH_RECOGNIZE NFA correlation", and "Staged rollout + auto-rollback (CORROBORATION gate)". Use "Risk-Based Alerting" verbatim per D-C6-2; drop the coined "Aggregation" term.
