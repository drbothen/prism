---
document_type: coverage-analysis
status: capture
do_not_execute: true
produced_by: analysis-agent
timestamp: "2026-06-28"
---

# Customer-Problem → Day-2 Design Coverage Matrix (Skeptical Audit)

> **Out-of-band side-analysis.** READ-ONLY mapping of a customer-problem-definition document
> against the Prism Day-2 design corpus (`.factory/specs/day2-design-decisions/ADR-PROP-*.md`,
> `ARCHITECTURE-DESIGN-SYSTEM.md`, `matured-vision-day2-requirements.md`). This file decides
> nothing and authorizes nothing. It is an honest auditor's read of what the design
> **architecturally decides** versus what it merely **enables** (content / library / managed
> service still to be built or delivered).

## Legend & method

- **COVERED** — the design makes an explicit architectural decision that satisfies the need.
- **PARTIAL** — the design decides part of it, or decides the *engine* but leaves load-bearing
  pieces (content/policy/tuning) to be authored, or mentions it without deciding.
- **GAP** — the design does not address it at all (no decision, no enabling substrate named).
- **A (Architecture)** — the mechanism/engine is designed and is a day-2 deliverable.
- **E (Enables only)** — the platform makes it possible, but the *content / library / managed
  service* that the customer actually consumes still has to be built or delivered.

A skeptical-auditor caution runs through this file: Prism's day-2 corpus is overwhelmingly a
**capability platform**. It is very strong at deciding *engines, invariants, and data models*. It
is consistently weak at delivering the **OT-specific content, the managed-service wrapper, and the
business-case artifacts** (pricing, insurability, ROI) that several of these customer problems are
actually asking for. Where an ADR "addresses" a gap by *proposing an epic* (e.g.
`E-DETECTION-CONTENT-001`), that is **not delivery** — it is a deferred scope note. Such items are
marked PARTIAL/E here even where the ADR text says "ADDRESS."

---

## P1 — "Cheap threat hunts"

| Item | Status | Day-2 decision(s) | A vs E | Honest note on what's missing |
|---|---|---|---|---|
| 1.a advisory auto-checked in-env (advisory feed → auto-query + AI triage; blocker: data cold, federated-vs-ingest) | PARTIAL | C11 D-C11-1/4 (feed-down, match-at-edge, edge priority fn); C8 D-C8-1/2 (PrismQL + `AS OF KNOWN`); C3 D-C3-1/2 (cost-based degrade, default time-window); C7 anomaly primitives; S3 agent runtime (AI triage) | A for the feed + match + query engine; **E for the auto-query/triage *templates*** | The advisory *query templates* and the orchestration that turns an inbound advisory into a hunt are not authored. "Data cold" blocker is explicitly only *mitigated* (C3 degrade + RETAIN cache), not solved. C11 covers advisories; **auto-hunt-on-advisory is not wired in any single ADR** — it spans C11+C8+S3 and is left as composition. |
| 1.b hunt → detection promotion (needs normalized data + stateful detection) | PARTIAL | C6 §14.7 recipe-library structure; C6 §14 alert model + staged-rollout (shadow→canary→prod); C4 boundary-normalization | A for the promotion *machinery* (recipe structure + staged rollout); **E for the saved-hunt → rule content** | The promotion path exists structurally, but the recipe/hunt corpus is content (`E-DETECTION-CONTENT-001`, *proposed epic, deferred to morph*). No one-click "promote this hunt to a detection" UX is decided. |
| 1.c ad-hoc run of pre-built standard hunts (saved hunt library/catalog, one-click via virtual analyst, cross-customer) | PARTIAL | matured-vision §14.7 + §16.3 "executable, backtested, MITRE-tagged PrismQL recipe + hunt library"; E12 "Hunting Library (ADOPT content)"; S3 agent for NL→PrismQL execution | **E** — library *structure* + execution engine decided; the **catalog content** is not authored | The hunt catalog is a **content deliverable** (`E-DETECTION-CONTENT-001`). "One-click via virtual analyst" is enabled by S3 but not specified as a catalog feature. **Cross-customer hunt sharing is NOT decided** (zero-access/BYOC thesis actively complicates it; only opt-in central-match C11-D2 exists). |

**P1 rollup: PARTIAL.** Biggest gap: **the hunt/detection content library itself** (engine decided, corpus deferred to `E-DETECTION-CONTENT-001`) + no decided advisory→auto-hunt orchestration.

---

## P2 — "Tuning is hard"

| Item | Status | Day-2 decision(s) | A vs E | Honest note on what's missing |
|---|---|---|---|---|
| 2.a too many false positives (context + signal/noise scoring + AI triage; blocker: data not integrated) | COVERED | C6 D-C6-2 (RBA-as-default, suppression-as-code w/ mandatory justification + time-box, fire-frequency dashboard); C12 (entity context for triage); S3 agent (AI triage) | A | Strongest single answer in the corpus. **Honest caveat the design itself states:** "Never silently mask a true positive" is *not* an absolute guarantee. Noise *baselines/scores* still require empirical measurement per deployment (content/tuning). |
| 2.b no playbook per alert (playbook library + virtual analyst walks junior; blocker: OT-specific playbook library doesn't exist) | PARTIAL | C15 D-C15-6 (`prism-orchestration` playbook/workflow DAG engine); S3 agent (guided investigation) | A for the **playbook engine**; **GAP for the OT playbook *library*** | The blocker the customer named — "OT-specific playbook library doesn't exist" — **remains true after day-2.** C15 decides the engine that *runs* playbooks; the playbooks are unauthored content. This is a genuine gap, not a misnamed item. |
| 2.c disposition tracking + CISO reporting (actioned%, response times) | PARTIAL | C6 §14.5 alert lifecycle (New→…→Closed/FalsePositive) + replay-link; C15 immutable ARO audit trail; C6 §14.5 ADOPT-4 source-coverage record | A for disposition *tracking* (state machine + audit); **GAP for CISO reporting** | Disposition state is tracked. But "**CISO reporting**", "**actioned %**" metric, and "**response-time SLO**" reporting surfaces are **not found** in any ADR or in matured-vision. The data exists; the reporting product does not. |
| 2.d legit-activity noise / RDP / living-off-land (change-mgmt integration, risk-based rules; blocker: change-mgmt data not integrated + GUI-only sensor automation brittle) | PARTIAL | C6 D-C6-2 (RBA risk-based rules); C15 D-C15-4 (`RECOMMEND` read-only projection — *avoids* GUI-only brittleness by design); C9 config versioning + audit | A for risk-based rules + the *anti-GUI-brittleness* design; **GAP for change-management *integration*** | RBA addresses risk-based scoring. The "GUI-only sensor automation is brittle" complaint is **structurally answered** (C15 routes automation through structured ARO/orchestration, not GUI scripting). But **change-management-system (ServiceNow/CMDB-change) integration is NOT decided** — repeatedly noted as "day-3 / operational." RDP/living-off-land are not named anywhere. |
| 2.e "am I blind when I tune down?" coverage / bill-of-health (blocker: defining healthy baseline) | PARTIAL | C6 D-C6-1 mandatory **coverage map** per `(source × time-slice)` = `{full/partial/none}`; distinguishes "EVALUATED, no match" vs "NO DATA"; C7 baseline mechanics (Welford/MAD, dual-rate) | A for the coverage map (a strong, concrete answer); **E for "healthy baseline" definition** | The "blind when I tune down" question is *well* answered by the coverage map. But the customer's own blocker — "**defining a healthy baseline**" — is exactly what C7 leaves to per-tenant tuning (OQ-C7-3); the math is decided, the *definition of healthy for this site* is not. No "bill-of-health" reporting surface is named. |

**P2 rollup: PARTIAL** (with a strong COVERED on 2.a). Biggest gap: **OT playbook library (2.b) + CISO/bill-of-health reporting (2.c/2.e) + change-mgmt integration (2.d)** — i.e., everything customer-facing *around* the well-decided tuning engine.

---

## P3 — "Can't ship data out"

| Item | Status | Day-2 decision(s) | A vs E | Honest note on what's missing |
|---|---|---|---|---|
| 3.a on-prem only (on-prem server + storage, federated search in-env, hardened/pen-tested devices) | PARTIAL | C2 satellite mesh (outbound-only, mTLS, store-and-forward, ESP residency D-C2-12); C5 federated S3 data-access (analysis-in-place); C10 dual-deployment client-managed | A for on-prem + federated-search; **GAP for "hardened/pen-tested devices"** | On-prem federated search is decided and strong. But **"hardened/pen-tested appliance"** is *assumed, not specified* — no CIS/STIG hardening baseline, no pen-test requirement, no pre-hardened appliance image is a decided deliverable. Operations concern, not architecture. |
| 3.b can't take raw data out (analysis in place) | COVERED | C5 D-C5-3 (residency reject-at-plan-time, fail-closed); C2 D-C2-12 (raw never crosses conduit — hard invariant); §17.8 Q2 (residency-first, ordered-before-forward) | A | Genuinely strong and structural. Raw-stays-in-place is a hard, fail-closed invariant, not a policy toggle. |
| 3.c anonymized too thin → token clearinghouse (irreversible tokenize, still useful) | PARTIAL | C16 D-C16-1/2/3/8 (Prism-native RSI clearing house; deterministic vaulted tokenization join-preserving; FF1 FPE; redaction; edge placement; per-tenant vault) | A for the clearing house; **PARTIAL on "irreversible"** | The clearing house is decided and is a strong answer. **But "irreversible" is only partially designed:** default deterministic tokens are *reversible via vault* (by design, to preserve joins). Irreversible mode = redaction (for agent-never-needs fields) only. The customer's "irreversible-but-still-useful-for-correlation" sweet spot is **not** a designed mode; the design accepts a documented frequency/linkage-attack surface (D-C16-7-TRADEOFF). |
| 3.d cloud OK if in MY VPC/account (customer-owned VPC; blocker: confirm NERC permits) | COVERED | C10 D-DEPLOY-002/005 (BYOC zero-access by construction); C20 (CIP-011-3 Jan-2024 entity-held-key cloud BCSI; Option-3 per-tenant CMEK) | A | BYOC is decided and the NERC-permits question is *answered* for BCSI (entity-held-key cloud storage is permitted post-Jan-2024). Cloud-**BES** (Project 2023-09) is correctly deferred (D-C20-SF2, "leave seams open"; not enforceable ~2029-2030). |
| 3.e [OPEN] automate remote-access (jump-host/VPN/RDP) to become a federated-search call | GAP | — | — | **Not addressed anywhere.** No ADR converts a remote-access session into a federated-query primitive. The customer flagged this OPEN and it remains OPEN. This is the single most-validated-by-threat-model item (see enrichment below) that the design does **not** touch. |

**P3 rollup: PARTIAL** (with COVERED on 3.b/3.d). Biggest gap: **3.e remote-access-as-federated-search (untouched)** + irreversible tokenization is weaker than the customer asked + hardened-appliance is unspecified.

---

## P4 — "IT SOC watches OT"

| Item | Status | Day-2 decision(s) | A vs E | Honest note on what's missing |
|---|---|---|---|---|
| 4.a IT doesn't know OT — context workbench surfaces site/OT context | PARTIAL | C12 D-C12-4 (Entity 360, Purdue-zone as first-class attribute, OT risk flags); S3 agent (guided NL investigation) | A for the context *substrate*; **E for the "workbench" UX** | The graph + Entity 360 substrate is decided. A dedicated "context workbench" UI is enabled (S2 console + S3 agent) but the surfacing UX is `E-GRAPH-INVESTIGATION-001` (proposed epic, deferred). |
| 4.b no OT response playbooks (OT playbook library, must be built) | GAP | C15 playbook engine only | A for engine; **GAP for OT playbook library** | Same as 2.b. The customer literally says "must be built" — and the design agrees by *not building it*. Genuine content gap. |
| 4.c IT can't tune/understand OT sensors (guided tuning + virtual analyst; sensor GUI-only) | PARTIAL | C6 auto-tune *suggestions* (human-gated); C4 configure-schema; S3 agent (guided) | A for guided-tuning engine; **E for OT-specific tuning guidance** | The guided-tuning machinery exists. OT-specific tuning *content/guidance* (what to tune, for which OT sensor) is unauthored. C4 configure-schema helps with the schema side. |
| 4.d long detection→response path (IT/OT comms, prioritization, OT-expert escalation behind IT SOC) | PARTIAL | C15 D-C15-2 (OT/safety = mandatory HITL; OT-expert escalation gate); C15 D-C15-6 (SLA-expiry escalation) | A for the *escalation gate*; **E for the IT/OT comms workflow** | Mandatory-HITL-for-OT and an escalation gate are decided. The *cross-team comms workflow* and prioritization policy are operational/playbook content, not decided. |
| 4.e need IT details in OT context & vice-versa (context workbench bridges IT/OT, flag what matters) | PARTIAL | C12 (unified graph: host↔user↔IP↔process↔alert↔asset + Purdue-zone) | A for the bidirectional data model; **E for "flag what matters"** | The graph genuinely bridges IT and OT entities. "Flag what matters" is a ranking/triage feature — enabled by C12 risk flags + S3 agent, but the OT-relevance ranking logic is unauthored. |

**P4 rollup: PARTIAL.** Biggest gap: **4.b OT response playbook library (must-be-built, not built)** — the same content gap as 2.b, and the highest-frequency missing item across the whole document.

---

## P5 — "What devices exist?"

| Item | Status | Day-2 decision(s) | A vs E | Honest note on what's missing |
|---|---|---|---|---|
| 5.a no asset inventory | COVERED | C14 D-C14-1/2/4 (active-query both readings; `ot_assets` first-class PrismQL table); C12 Entity 360 | A | Decided and concrete. Both API-tier (Reading A) and direct-protocol (Reading B) polling. **Caveat:** OCSF has no dedicated OT classes (OQ-C14-OCSF, in-flight) so normalization class mapping is *not finalized*; safe poll cadence is OQ-C14-CADENCE-NUMBERS (morph validation). |
| 5.b naming conventions don't align (mapping/consolidation layer) | PARTIAL | C12 D-C12-3 (deterministic auto-merge on strong IDs; suspected-links for fuzzy); C4 quarantine+relabel; `E-MANAGED-MAPPING-001` (proposed) | A for strong-ID consolidation; **E for synonym/naming reconciliation** | Strong-identifier consolidation is decided. **Fuzzy/synonym naming reconciliation** ("fw" vs "firewall") is explicitly deferred (suspected-links + `E-MANAGED-MAPPING-001` proposed epic, SaaS-only, deferred to morph). |
| 5.c want network-architecture diagram (topology view) | PARTIAL | C12 (graph edges: CONTROLS/MONITORS/CONNECTS_VIA + Purdue-zone) | A for topology *data*; **GAP for the *diagram*** | The graph stores topology. **Diagram rendering/auto-layout is not a decided deliverable** — it is a UI/reporting feature (related to `E-GRAPH-INVESTIGATION-001`, proposed). The customer asked for a diagram; they get queryable graph data. |

**P5 rollup: PARTIAL** (strong on 5.a). Biggest gap: **network-architecture diagram rendering (5.c) is data-only, not a delivered view**; fuzzy naming reconciliation deferred.

---

## P6 — "Don't know full OT context (Chevron problem)"

| Item | Status | Day-2 decision(s) | A vs E | Honest note on what's missing |
|---|---|---|---|---|
| 6.a don't know full extent, remote sites (context aggregation + asset/topology) | PARTIAL | C2 satellite mesh (per-site edge, 8-hop tree); C12 graph aggregation; C14 asset inventory | A for multi-site aggregation substrate; **E for completeness assurance** | The mesh + graph can aggregate remote-site context. But "knowing the *full* extent" depends on coverage you can't guarantee for un-instrumented sites — and the design honestly only reports coverage (C6-D1 map), it doesn't claim completeness. |
| 6.b central SOC blind to local changes e.g. firewall swap (infer changes from stored data e.g. firmware versions, change tracking) | PARTIAL | C12 D-C12-3 (temporal validity intervals on identity edges); C14 `ot_config_baselines`/`ot_device_state` tables | A for temporal data + config-baseline tables; **E for change-inference logic** | The temporal substrate and config-baseline tables exist. **Change-inference queries/detections (e.g. "firmware version changed → flag") are unauthored** — downstream recipe/detection content, not a decided feature. |
| 6.c tribal knowledge (capture/track changes systematically) | PARTIAL | C12 D-C12-1 "Institutional Memory" thesis; S3 agent synthesis into cited findings | A for the substrate + synthesis engine; **E for the capture workflow** | "Institutional Memory" is a stated thesis backed by the graph + LLM synthesis. There is **no decided workflow for analysts to explicitly capture tribal knowledge** (annotations, notes-as-edges); it is emergent from data, not a deliberate capture product. |
| 6.d need context around devices not just devices — network arch, firewall rules, auth devices/logs (ingest/correlate surrounding context) | PARTIAL | C12 graph (control/process edges); §17 federated ingestion (collector class, syslog/NetFlow/pcap, OT dissectors) | A for the ingestion + correlation substrate; **E for the specific source onboarding** | The ingestion machinery for surrounding context (firewall logs, auth logs, pcap) is decided in §17. **Firewall-rule modeling specifically** and auth-device correlation are enabled but not modeled as decided schema. |

**P6 rollup: PARTIAL.** Biggest gap: **change-inference content (6.b) and deliberate tribal-knowledge capture (6.c)** — the substrate is decided, the *systematic change-tracking product* is not. (The "centralized OT team / central SOC blind to local changes" sub-problem folds here and is equally PARTIAL.)

---

## P7 — "Can't find/get great talent"

| Item | Status | Day-2 decision(s) | A vs E | Honest note on what's missing |
|---|---|---|---|---|
| 7.a can't hire/retain (managed service / virtual analyst absorbs labor) | PARTIAL | C10 D-DEPLOY-002 (MSSP-managed operating model); S3 agent runtime (virtual analyst); §11.3 S3 embedded agent | A for the *virtual-analyst engine* + MSSP *deployment model*; **GAP for the managed *service offering*** | The technical substrate (MSSP deployment profile + S3 virtual analyst) exists. But "**managed service**" as a *delivered service product* — staffing model, SLAs, service catalog — is **not designed**; it is a go-to-market construct the platform enables. |
| 7.b too expensive to self-staff (outsourced-but-affordable middle path; pricing model) | GAP | — (C11-D7 SKU packaging for *Prism Intel* only; C10 §"Pricing — entirely unknown") | — | **Pricing model is explicitly unknown.** ADR-PROP-competitive-positioning states plainly: "Pricing — entirely unknown; could change positioning economics." The "affordable middle path" is a business proposition with **no design or pricing artifact.** Genuine gap. |
| 7.c fast-for-senior/simple-for-junior (give answers not questions, step-by-step juniors, fast-approve seniors) | PARTIAL | S3 agent runtime (NL→PrismQL, guided investigation, evidence-package); C10 D-C10-4-Q2 self-QA "Senior Analyst Review" gate (`E-EVIDENCE-PACKAGE-001`) | A for the answer-generating engine + output-assurance gate; **GAP for explicit persona-tiered UX** | The S3 agent *enables* "give answers" and the evidence-package + self-QA gate *enables* "fast-approve for seniors." But **explicit junior-vs-senior persona tiering** (step-by-step mode vs fast-approve mode) is **not a decided UX**. The surfaces (S1-S4) are persona-segmented by *tooling preference*, not by *seniority workflow*. |

**P7 rollup: PARTIAL** (weakest of the seven). Biggest gap: **the entire managed-service + pricing business model (7.a/7.b) is undesigned** — the platform enables it, the offering doesn't exist; and persona-tiered junior/senior workflow (7.c) is not decided.

---

## Candidate additions (NOT mandated — assessed separately)

| Item | Status | Day-2 decision(s) | A vs E | Honest note |
|---|---|---|---|---|
| **Candidate A** — "I can't see the value / what are you doing" (operational observability; CLIP-portal complaint; AROs + notifications; assurance record) | PARTIAL | C15 ARO model + immutable audit trail (assurance record); `E-EVIDENCE-PACKAGE-001` (Investigation Report + Query Log + IOC Ledger + self-QA); U1 admin/ops console (health/observability/audit viewer) | A for ARO/assurance-record + audit substrate; **GAP for the customer-facing value/observability *portal*** | AROs *are* the assurance record (strong). But "**what are you doing for me**" as a **customer value/operational-observability portal** (the CLIP-portal complaint reframed positively) is **not designed**. U1 is an *admin* console, not a *value-demonstration* surface. Notifications delivery is an integration point (deferred). |
| **Candidate B.a** — who is authorized to act (pre-agreed authority matrix, OT-expert escalation) | PARTIAL | C15 D-C15-3 (`required_approver_role` + C18 RBAC); D-C15-2 (OT mandatory HITL + escalation) | A for the RBAC *gate*; **E for the matrix *policy*** | The enforcement gate is decided; the *authority matrix content* (who, for what action class) is operational policy authoring (C18/C19 scope). |
| **Candidate B.b** — is the response itself OT-safe (risk-based actions, segment-don't-isolate) | PARTIAL | C15 ARO carries `asset_tier`/blast-radius + reversibility metadata; C14 D-C14-5 read-only OT-safety hard invariants (for *query*); D-C15-2 OT HITL | A for the safety *invariants on query* + ARO risk metadata; **GAP for "segment-not-isolate" response semantics** | OT-safety for *active query* is a hard invariant (strong). But **OT-safe *response* semantics** ("segment, don't isolate") are **playbook-authored content**, not decided action vocabulary. The customer's specific ask is unmet at the design layer. |
| **Candidate B.c** — how fast can I safely restore (PLC config / EWS backup, clean-to-restart assurance, manual-ops fallback) | GAP | C17 (config/state/data backup + CIP-009 evidence + restore-test runs); C9 (A/B dual-slot bootstrap, watchdog) | A for *Prism's own* backup/recovery; **GAP for OT device recovery** | C17/C9 decide recovery of **Prism's own state**, and CIP-009 *evidence generation* is first-class. But **PLC config backup from field devices, engineering-workstation recovery, "clean-to-restart" OT orchestration, and manual-ops fallback are NOT addressed** as Prism deliverables. matured-vision search: "not found." Genuine gap. |
| **Candidate C.a** — prove investment paying off (exposure-reduction scoring over time, ROI/risk reporting; blocker: no OT maturity model/actuarial baseline) | GAP | — | — | **Not found anywhere.** No exposure-reduction scoring, no ROI/risk-trend reporting, no OT maturity model. Genuine gap. |
| **Candidate C.b** — insurer asking about OT controls (control-evidence pack mapped to underwriting) | PARTIAL→GAP | C18/C19 compliance-profiles (baseline→soc2→iso27001→iec-62443-ot→nerc-cip, tighten-only); C20 D-C20-SF1 `E-CIP-EVIDENCE-EXPORT-001` (RSAW export, *proposed*) | E (compliance mapping); **GAP for insurer/underwriting** | Compliance-control mapping exists for *regulatory* audit (CIP/SOC2/ISO). But **insurer/underwriting-specific control-evidence packs are explicitly NOT addressed** — the agent's read: "deferred to go-to-market." matured-vision: "not found." |
| **Candidate C.c** — genuinely secured + defensible record not box-checking (AROs + assurance record) | COVERED | C15 immutable ARO audit trail; `E-EVIDENCE-PACKAGE-001` self-QA gate; C17 CIP-009 restore-test + integrity records | A (substrate) | This is well-served: the ARO immutable trail + evidence-package + self-QA gate + tamper-evident audit are a genuine "defensible record, not box-checking" substrate. The *reporting/packaging* of it for a given audience is the deferred part. |

---

## Enrichments

| Enrichment | Status | Day-2 decision(s) | Honest note |
|---|---|---|---|
| cloud-BCSI gated/shifting (CIP-004-7/011-3 permit BCSI-in-cloud; Project 2023-09 for BES-in-cloud) | COVERED | C20 (entity-held-key cloud BCSI per CIP-011-3 Jan-2024; D-C20-SF2 defer cloud-BES, leave seams open) | Accurately captured and correctly scoped. BCSI-in-cloud permitted via Option-3; BES-in-cloud deferred (~2029-2030 horizon). |
| BCSI clearinghouse "Phantom Tollbooth" (proxy finds BCSI, tokenizes to non-BCSI; self-hosted fine-tuned NERC model) | GAP | C16 clearing house (tokenizes *declared* RSI fields at edge) | The C16 clearing house tokenizes **declaratively-tagged** fields. A **proxy that *scans for undeclared* BCSI** ("Phantom Tollbooth") is **NOT designed**. A **self-hosted fine-tuned NERC model** is **out of scope** (Prism ships no domain-model trainer). |
| retention 60-day NERC minimum (ingest-vs-federated input) | COVERED | C17 / matured-vision (CIP-007 R4 ≥90d online + archive; RETAIN/RetentionCache tiers; demand-driven) | Exceeds the 60-day floor; retention primitives are decided. The federated-vs-ingest tension is the core architectural thesis (decided: federated-default, ingest-via-RETAIN-cache on demand). |
| #4 Evergy validation (IT SOC does it) | COVERED (as validation) | P4 architecture (IT-SOC-watches-OT) | Corroborates P4 framing; the design's IT/OT-bridge supports the IT-SOC-runs-OT model. No artifact needed — it validates an existing design choice. |
| #5 supply-chain/foreign-component (PRC-origin BESS/inverter exposure as inventory lens) | GAP | C14 `ot_assets` schema (no provenance fields); §17.12 mentions "supply-chain audit scope" in passing | **Genuine gap.** Asset inventory exists; **supplier/origin provenance fields, PRC-origin flagging, BESS/inverter exposure lens are NOT modeled.** matured-vision: only a passing mention, no design. |
| threat-model reframe (direct PLC manipulation rare ~4%; credential-abuse + internet-facing-remote-access + IT→OT propagation is the workhorse → validates 3.e) | GAP (as stated framing) | OT architecture exists (C2/C12/C14); no threat-model statistics or vector-prioritization stated | The **threat-model reframe itself is not stated** in any artifact. Critically, it **validates 3.e** (remote-access-as-federated-search) as the *highest-value* item — which is precisely the item the design does NOT touch. This is the most important strategic miss the audit surfaces. |
| producer-neutral threat naming (advisory feed reconciliation, ties 1.a) | COVERED | C11 D-C11-4 (CVSS v4 / EPSS / KEV / CSAF-VEX canonical naming; VEX suppression) | Decided via pinned standards. Vendor→canonical reconciliation logic is the enabled-not-delivered part (OQ-C11-1 commercial partners). |

---

## (1) Per-mandated-problem rollup

| Problem | Overall | Single biggest gap |
|---|---|---|
| **P1** Cheap threat hunts | **PARTIAL** | Hunt/detection **content library** unauthored (`E-DETECTION-CONTENT-001`, deferred); no decided advisory→auto-hunt orchestration |
| **P2** Tuning is hard | **PARTIAL** (2.a COVERED) | **OT playbook library** + CISO/bill-of-health **reporting** + change-mgmt **integration** — everything around the well-built tuning engine |
| **P3** Can't ship data out | **PARTIAL** (3.b/3.d COVERED) | **3.e remote-access-as-federated-search is untouched**; irreversible tokenization weaker than asked; hardened appliance unspecified |
| **P4** IT SOC watches OT | **PARTIAL** | **OT response playbook library (must-be-built, not built)** — the highest-frequency missing item in the whole document |
| **P5** What devices exist | **PARTIAL** (5.a strong) | **Network-architecture diagram is data-only, not a rendered view**; fuzzy naming reconciliation deferred |
| **P6** Full OT context (Chevron) | **PARTIAL** | **Change-inference content + deliberate tribal-knowledge capture** — substrate decided, change-tracking *product* not |
| **P7** Talent | **PARTIAL** (weakest) | **Managed-service offering + pricing model entirely undesigned**; persona-tiered junior/senior workflow not decided |

### Candidate verdicts

| Candidate | Verdict |
|---|---|
| **A** value-visibility / observability portal | **PARTIAL** — assurance record (ARO) decided; customer-value/observability portal not |
| **B** OT-safe response & recovery | **PARTIAL→GAP** — authority-gate decided (B.a); OT-safe-response semantics (B.b) and OT recovery/clean-to-restart (B.c) are gaps |
| **C** prove risk is managed | **PARTIAL** — defensible-record substrate (C.c) decided; exposure/ROI scoring (C.a) and insurer/underwriting packs (C.b) are gaps |

## (2) Design's strongest vs weakest problem

- **Strongest problem the design answers: P3 "Can't ship data out"** (specifically 3.b raw-stays-in-place and 3.d BYOC). These are **structural, fail-closed invariants** (C2 D-C2-12 residency, C5 D-C5-3 reject-at-plan-time, C16 edge tokenization, C20 entity-held-key cloud BCSI), not policies — the design's zero-access thesis is its genuine architectural triumph and is *delivered*, not merely enabled. **Runner-up: P2 2.a (false positives)** via RBA-as-default + suppression-as-code + coverage map.
- **Weakest problem the design answers: P7 "Talent."** The entire ask — managed service, pricing, affordable middle path, persona-tiered junior/senior workflow — is **business-model and service-delivery**, which the corpus (correctly, but consequentially) treats as out-of-architecture. Pricing is *self-described* as "entirely unknown." The platform *enables* a managed service; it does not *constitute* one. **P4 (IT SOC watches OT)** is a close second-weakest, because its load-bearing piece — the OT playbook library — is the most-repeatedly-deferred content gap in the document.

## (3) Genuine gaps the day-2 design does NOT address at all

These are real absences (no decision, no enabling artifact named), distinct from "engine-decided-content-deferred":

1. **OT response playbook library** — named "must be built" by the customer (2.b, 4.b); engine decided (C15), library unauthored. Highest-frequency miss.
2. **Remote-access automation (jump-host/VPN/RDP) as a federated-search call** (3.e) — untouched; *and validated as highest-value by the threat-model reframe*.
3. **Pricing model / managed-service offering economics** (7.a/7.b) — explicitly "entirely unknown."
4. **Insurability / underwriting control-evidence packs** (C.b) — compliance-audit mapping exists; insurer-specific packs do not.
5. **Clean-to-restart OT recovery — PLC config backup, engineering-workstation recovery, manual-ops fallback** (B.c) — Prism's own recovery decided; OT *device* recovery not.
6. **Exposure-reduction scoring / ROI / risk-trend reporting / OT maturity model** (C.a) — not found.
7. **Change-management-system integration (ServiceNow/CMDB-change)** (2.d, 6.b) — repeatedly deferred to "day-3/operational."
8. **CISO reporting / actioned-% / response-time SLO / bill-of-health reporting surfaces** (2.c, 2.e) — data tracked, reporting product absent.
9. **Network-architecture diagram rendering** (5.c) — graph data decided, diagram view not.
10. **Supply-chain / foreign-component provenance (PRC/BESS/inverter inventory lens)** (#5 enrichment) — asset schema has no provenance fields.
11. **"Phantom Tollbooth" undeclared-BCSI-scanning proxy + self-hosted fine-tuned NERC model** — C16 tokenizes *declared* fields only; scanning-for-undeclared-BCSI and a domain model trainer are out of scope.
12. **Threat-model reframe as a stated artifact** — no vector-prioritization/statistics anywhere; its absence is why #2 isn't recognized as the priority it is.
13. **Deliberate tribal-knowledge capture workflow** (6.c) — "Institutional Memory" is emergent-from-data, not a capture product.
14. **Customer-facing value/observability portal** (Candidate A / CLIP reframe) — U1 is admin-facing, not value-demonstration.

## (4) The architecture-vs-content/service pattern

The dominant pattern across this entire audit: **the day-2 design is a capability platform, not a product.** It is extremely strong at deciding **engines, invariants, and data models** (detection engine, ML backends, ARO model, satellite mesh, clearing house, bitemporal PrismQL, cost-based federation, backup/recovery, compliance-profile engine). It is consistently silent on the three things several mandated problems are *actually* asking for:

- **OT-specific CONTENT** — detection rules, hunt catalog, **OT response playbooks**, OT tuning guidance, change-inference detections. The engines that *run* this content are decided; the content is unauthored and largely deferred to *proposed* epics (`E-DETECTION-CONTENT-001`, etc.) at "morph." A proposed epic is a scope note, not a deliverable.
- **A managed SERVICE wrapper** — staffing, SLAs, pricing, the "outsourced-but-affordable" offering, persona-tiered junior/senior workflows. The MSSP deployment profile + S3 virtual analyst *enable* this; no service product is designed.
- **BUSINESS-CASE artifacts** — pricing, insurability/underwriting evidence, exposure-reduction/ROI scoring, value-demonstration portal. These are go-to-market constructs the platform can feed but does not contain.

This is a defensible *engineering* posture (build the platform; layer content/service on top). It is a **risk** if the customer-problem document is read as a *product* commitment, because the customer's most acute pains (P4 OT playbooks, P7 talent/pricing, P2 reporting) live almost entirely in the content/service layer the architecture *enables but does not deliver*.

## (5) The two questions, answered candidly

**Q1 — Does the day-2 design FULLY answer this document?**
**No — it SUBSTANTIALLY answers the *platform/architecture* dimension and only PARTIALLY answers the *product/service/content* dimension.** Of the architectural questions the document raises (federated query, zero-access residency, OT querying, detection/tuning engines, knowledge graph, backup/recovery, compliance posture), the design answers them substantially-to-fully, with honest, well-documented caveats. But the document is not purely an architecture document — it is a *customer-problem* document, and a material fraction of its asks (OT playbook content, managed-service offering, pricing, insurability evidence, clean-to-restart OT recovery, remote-access automation, value/ROI reporting) are **not addressed at all**. Net: the design **fully answers the engineering subset, substantially answers the overall intent at the platform layer, and leaves significant content/service/business-case gaps**. It does **not** fully answer the document.

**Q2 — Does it FULLY address the 7 mandated problems?**
**No — it PARTIALLY addresses all seven; none is FULLY addressed end-to-end, though P3 comes closest.** Every mandated problem has a real, decided architectural backbone (so none is a total GAP), but every one also has a load-bearing unmet piece: P1 lacks the hunt content library and advisory→auto-hunt orchestration; P2's tuning *engine* is excellent (2.a near-COVERED) but its playbook/reporting/change-mgmt surround is missing; P3 is strongest (3.b/3.d COVERED) yet 3.e is untouched; P4's OT playbook library — the customer's explicit "must be built" — is not built; P5's diagram is data-only; P6's change-tracking *product* is unbuilt; P7 (talent/pricing/managed-service) is largely undesigned and is the weakest. **Precise verdict: SUBSTANTIALLY addressed at the architecture layer; PARTIALLY addressed as customer-facing problem resolution; FULLY addressed for none of the seven.**
