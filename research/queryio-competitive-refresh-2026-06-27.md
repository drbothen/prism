---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
scope: "C10 — Query (query.ai) competitive positioning gap-check vs Prism day-2 vision (post C1–C9). OUT-OF-BAND day-2 SIDE-ANALYSIS; SEPARATE from the live VSDD factory pipeline. Does NOT modify STATE.md, SESSION-HANDOFF.md, the live ADR registry, any live spec/BC/story, RESEARCH-INDEX.md, or any prior research file."
provenance: "REFRESH of the 2026-06-25 Query analysis (queryio-federated-search-2026-06-25.md + queryio-deployment-credentials-ui-2026-06-25.md). Read first for non-contradiction: matured-vision-day2-requirements.md §10.2/§11.4/§11.5 + day2-design-decisions/* (C1–C9 leans/decisions per SESSION-RESUME-2026-06-27.md §2)."
confidence_note: "Product facts decay rapidly; every claim is date-stamped. Leans are discussion input only, NOT decisions. [model-knowledge] / [INCONCLUSIVE] flags inline."
---

# C10 — Query.io Competitive Refresh & Positioning Gap-Check (2026-06-27)

> **READ-FIRST NON-CONTRADICTION.** This refresh is consistent with the prior Query analysis
> (2026-06-25) and the C1–C9 day-2 decisions/leans recorded in `SESSION-RESUME-2026-06-27.md §2`.
> Where this pass *updates* the 2026-06-25 picture, it is flagged **[CHANGED]**. Where it merely
> confirms, it is flagged **[CONFIRMED]**. Nothing here is a decision; the day-2 morph remains gated
> on the brief-reframe HUMAN sign-off (§5.1).

## 0. Executive summary (the gap-check, up front)

Since the 2026-06-25 read, Query has **materially accelerated on the agentic + detection axes** and
**confirmed two structural weaknesses Prism's day-2 plan already targets** (no on-prem/OT/air-gap
deployment; no RBAC in-product as of their own 2024 compliance page). The single biggest *delta vs
the prior read* is **Query Workers** (GA "available now," announced 2026-03-30) — a multi-stage,
auditable, evidence-package agent layer with a built-in "Senior Analyst Review" nine-check quality
gate and **explicit BYO-Agent + MCP + A2A support**. This is a direct, named encroachment on Prism's
"agent-native / MCP-first" thesis and must be treated as **parity-reached, not whitespace**, on the
agentic-runtime dimension.

Where Prism still leads on *design intent*: formal query language (PrismQL vs un-grounded FSQL),
OT/edge/air-gap via satellite mesh, AI-opaque credentials, demand-driven retention as
SIEM-by-capability, and finer-grained RBAC. Where Query is genuinely stronger **today**: shipping
maturity, connector breadth (50+ connectors, "150+ technologies" legacy claim), a real customer base
(named major-utility enterprise deal, 2026-03), a GA detection product (Federated Detections GA
2026-01), a shipped egress product (Security Data Pipelines), and SaaS time-to-value ("new sources
live in <15 minutes"). **Prism's lead is on the drawing board; Query's lead is in production.** The
§2.4 honest-tradeoff discipline must govern any positioning claim accordingly.

**Three gaps Prism's C1–C9 plan does NOT currently answer** (detailed in §3):
1. **Out-of-the-box detection content library + rule-translation OUT** (Query ships SPL/KQL/Sigma↔FSQL
   *and* an OOTB rule library; Prism has Sigma→PrismQL as a *candidate* epic only, IN-bound only).
2. **Auditable agent evidence-package + automated senior-analyst QA review** (Query Workers ships a
   structured Investigation Report + Query Log + IOC Ledger + 9-check review; Prism's S3 embedded
   agent has no equivalent "evidence package + self-QA" contract yet).
3. **A2A (agent-to-agent) protocol support** alongside MCP (Query announced both; Prism's day-2 is
   MCP-only — A2A is not in scope).

---

## 1. Query.io current state (2026 refresh)

### 1.1 Architecture & thesis — [CONFIRMED, stable]

Query's core thesis is unchanged and actively reinforced through mid-2026: a **federated search /
"security data mesh"** that **"centralize[s] insights, not data"** — zero-ETL, no ingestion, no
index, read-only API "bridges," data stays at source, normalized to an **OCSF-aligned Query Data
Model (QDM)** at query time.[1][2][5][8] Product architecture docs describe Query as a "modern,
cloud-native web application."[deep-research][2] FSQL (Federated Search Query Language) remains the
unified syntax; it supersedes the older UQL.[1][8] FSQL is **still not published as a formal grammar**
— described only by design goals (windowed aggregations, thresholds, grouping, deterministic
evaluation), confirming the prior read's whitespace finding.[1][3-deep][prior-2026-06-25]

**[CHANGED] FSQL now has a documented REST API.** The 2025 Year-in-Review (2025-12-22) states the
**"FSQL REST API enables machine-to-machine access to federated search results, schema metadata, and
connector information — supporting SOAR, detection engineering, and AI workflows."**[1] This is a
programmatic surface the 2026-06-25 read did not emphasize.

**[CHANGED] Explicit federated-join messaging.** A 2025-08-12 Query blog explicitly markets
**"federated joins … like joining authentication events in Sentinel's cold tier to EDR alerts in
CrowdStrike FDR — in a single search."**[8] The prior read inferred cross-source correlation; Query
now markets cross-lake *joins* as a headline capability. (Bearing on Prism C3's join-guard decision:
Query advertises the capability Prism is choosing to *hard-reject for unbounded cross-source joins*
at plan-time — a deliberate safety-vs-expressiveness fork worth naming in positioning; see §2a.)

### 1.2 Connector catalog — [CONFIRMED, growing]

- **"50+ sources"** is the current homepage/Federated-Detections figure (2026-06-11).[10] The
  "150+ technologies" claim is legacy (pre-2024) and does not appear verbatim in current pages; the
  connectors page now emphasizes "constantly growing" over a fixed count.[deep-research][4]
- **Static vs dynamic schema split CONFIRMED.** Static connectors (CrowdStrike, Okta, Entra ID,
  Defender, SentinelOne, etc.) are Query-pre-mapped; dynamic-schema sources (Splunk, Snowflake,
  BigQuery, Chronicle/Google SecOps, Sentinel, Databricks, S3) use the no-code **Configure-Schema**
  workflow (introspect → sample → map to OCSF → save).[deep-research][8][11]
- **[CHANGED] Explicit data-lake federation messaging (2025-08-12):** named targets now include
  **Microsoft Sentinel Data Lake, Amazon Security Lake, Delta Lake, Apache Iceberg-in-Athena,
  CrowdStrike LogScale/FDR, Cribl Lake, Elastic.**[8] Confirmed by AWS's own docs: **Query.AI is a
  listed Amazon Security Lake "Subscriber" integration that queries Security Lake tables via Amazon
  Athena.**[7-comp] This directly overlaps Prism's C5 (SIEM/Security-Lake federation + Iceberg cold
  tier) — Query already ships Security-Lake/Iceberg-via-Athena federation **today**. (See §3 gap.)
- New-source time-to-value claim: **"New sources live in <15 minutes."**[10]

### 1.3 Agentic / AI layer — [CHANGED — the big delta]

This is where the most has changed since 2026-06-25. Two product lines now exist:

- **Query Agents / Mission-Specific Agents** (the prior read's term) — Asset Information, Detection
  Finding Triage, File Hash, Network Activity, Threat Research, Vulnerability Intelligence. Each is an
  LLM + curated tools (e.g., `get_network_activity_by_ip`, `is_public_ip`,
  `retrieve_ip_geolocation_data`) + a knowledge base (MITRE ATT&CK), operating over the mesh.[9-blog]
  [12-deep] Six agents were in preview as of 2025-07-24.[9-blog]
- **[NEW] Query Workers** — announced **2026-03-30, "available now."**[3-news] Higher-order than the
  single-purpose agents: **multi-stage, structured workflows** that compose "specialized skills"
  (classification, scoring, enrichment, identity analysis, network analysis) "called as needed based
  on what the investigation uncovers." Launch workflows: **Investigation Worker, Threat Hunting
  Worker, Identity Threat Assessment Worker** (sweeps 8 identity attack patterns). Each run produces a
  full **auditable evidence package**: Investigation Report (findings + recommended disposition +
  ATT&CK mapping + timeline + next steps), **Query Log** (every search/source/result-count, replayable
  + auditable), **IOC Ledger** (every indicator typed/sourced/enriched), and — on high-severity
  findings — an automated **"Senior Analyst Review" nine-check QA** (evidence completeness, logic
  verification, missed indicators, severity calibration, blind spots).[3-news] **Workers do NOT take
  actions** — they produce findings + recommendations; the human decides.[3-news][10]

- **[NEW / CRITICAL FOR PRISM] MCP + A2A + BYO-Agent.** Query's 2025 Year-in-Review (2025-12-22):
  **"With support for the MCP and A2A protocols, customers can bring their own models and agents,
  leveraging normalized access to distributed security data."**[1] Query has positioned its platform
  as **"a practical MCP server, purpose-built for security operations"** — the **"USB-C hub for
  security data"** — since at least 2025-04-16.[6-blog] Query Workers also explicitly **"supports BYO
  Agent access."**[3-news] **This is direct, named overlap with Prism's MCP-native / S1 BYO-agent
  thesis.** Prism's prior "parity-with-differentiation" framing (§11.4) still holds, but the parity
  half is now *shipped and dual-protocol* (MCP **and** A2A), whereas Prism's day-2 is MCP-only.
- **[NEW] CoPilots + NL/SPL/KQL/Sigma → FSQL conversion agents.** Query ships agents that convert
  Natural Language, SPL, KQL, and Sigma into FSQL.[1] Confirms + extends the prior ADOPT-5 finding
  (rule translation IN), and adds NL→FSQL and a data-onboarding/schema-mapping CoPilot.[1]

### 1.4 Federated Detections — [CHANGED — now GA]

- **[CHANGED] Federated Detections reached General Availability January 2026** (public preview late
  2025).[1] The 2026-06-25 read had it as "coming soon / GA-only-in-Splunk-app." It is now a GA
  console product.
- Mechanics CONFIRMED: scheduled FSQL with explicit evaluation windows + thresholds; deterministic
  outcomes (`MATCHED` / `ERROR`); records time-range evaluated, **source coverage**, match counts;
  early-termination on threshold; emits a **finding with a replay link** that re-runs the exact
  window.[3-deep][8-deep] Marketed as "scheduled, auditable and deterministic … no mystery logic."[10]
- **[NEW] Out-of-the-box detection rule library.** Homepage (2026-06-11): "Translate SPL, KQL, or
  Sigma rules, **or use the out of the box library**."[10] Prism has no OOTB content library in
  C1–C9 (see §3 gap).

### 1.5 Alert Destinations & Security Data Pipelines (egress) — [CONFIRMED + matured]

- **Security Data Pipelines** (announced 2025-08-04, in preview then; described as mature by mid-2026):
  route OCSF-normalized "gold" datasets OUT to **Amazon S3, Azure Blob (ADLSv2), Google Cloud
  Storage, Splunk** — written as **Parquet (ZSTD/Snappy), Hive-style partitions by source/event-type/
  time**, orchestrated by Query's cloud (no customer containers/code). **Snowflake, Databricks, Amazon
  Security Lake "coming soon."**[7-news][7-deep] This is the realized form of the prior read's G-11
  whitespace ("connector egress / normalized-result destinations").
- **Alert Destinations** (detection outcome routing): Amazon SNS, Azure Sentinel, Google Pub/Sub,
  Google SecOps, Jira, Microsoft Teams, Slack, PagerDuty, ServiceNow, Tines (Webhooks + Cases),
  Generic Webhook; multi-destination + severity-based routing; secrets from **AWS Secrets Manager**;
  CloudWatch-logged.[6-deep] (Note: the AWS-Secrets-Manager + CloudWatch detail confirms an
  **AWS-hosted SaaS backend**.)[6-deep]

### 1.6 Deployment, RBAC, credentials, residency — [CONFIRMED — Prism's structural advantages hold]

- **[CONFIRMED] Multi-tenant SaaS only.** No on-prem / edge / air-gapped / OT deployment option
  appears anywhere in current Query materials; on-prem data sources are reached via API or a
  customer-stood-up **reverse proxy** (e.g., MISP-behind-DMZ guidance).[deep-research][2][prior-06-25]
  Org→tenant→team topology. **→ Prism Satellite mesh (C2) + native OT (§13.6) remain structural
  advantages Query lacks entirely.** [INCONCLUSIVE on internals: Query may offer bespoke private
  deployments under enterprise contracts; nothing public confirms this.]
- **[CONFIRMED — and stronger than the prior read flagged] RBAC.** Query's OWN
  compliance-and-security page (dated 2024-10-03) states verbatim: **"we do not currently support RBAC
  in our product, but this is a product feature that is slated for QX-202X."**[3-onprem] Earlier reads
  cited a "2-role Team Admin/Team Member" model; the vendor's own compliance page is more damning —
  **in-product authorization is SSO-login-gated only, no role granularity.** Even if 2-role exists in
  team-management UI, there is no fine-grained, connector/dataset-scoped RBAC. **→ Prism's
  finer-grained RBAC (C9 / G-12 / E-CENTRAL-AUTHZ-001) is a real differentiator, and a stronger one
  than the prior read implied.** [Decay caveat: "slated for QX-202X" means Query may ship RBAC at any
  time; re-verify before any external positioning claim.]
- **[CONFIRMED] Credentials undocumented + AWS-Secrets-Manager-backed centrally.** Query does not
  publish KMS/Vault/envelope-encryption details; Alert-Destination docs reveal AWS Secrets Manager as
  the secret store.[6-deep] Critically, **Query stores connector creds centrally in the SaaS control
  plane**, and there is **no "AI-opaque credential" guarantee** published. **→ Prism's AD-017
  AI-opaque, per-tenant-DEK, reference-based model (§11.1, SS-26) is a genuine trust differentiator,
  especially for MSSP/regulated buyers.**
- **Residency:** "data stays at source" is the residency story; cross-border *query paths* and
  Query's role as processor remain the customer's concern. **"Query does not store any customer data.
  Period."**[3-onprem] No documented data-residency *controls* beyond non-storage. **→ Prism's
  per-field residency policy-as-code (A-lean #7) + edge-local credential resolution (§11.1 satellite)
  is materially more rigorous.**

### 1.7 Funding / traction / company — [CHANGED — fresh datapoints]

- Founded 2018 (Atlanta), CEO Dhiraj Sharan.[3-fund] Funding history: **Seed $4.6M (2020/2021,
  ClearSky + DNX Ventures + South Dakota Equity Partners)**[7-fund]; **Series A $15M (2021-10,
  led by SYN Ventures + ClearSky + South Dakota)**[1-fund][2-fund]; **Series A-II (2024-09-25,
  Cisco Investments)**[4-fund]. Total raised ≈ **$19.6M across 3 rounds**.[3-fund][6-fund]
- **[NEW — 2026] Convertible-note round (2026-03), ClearSky "Security I," "building on Query.AI's
  momentum following a significant enterprise deal with a major utility provider"** + the Federated
  Detections launch.[5-fund / ClearSky Spring-2026 newsletter, 2026-04-22] **→ Two fresh signals:
  (a) a named major-UTILITY (i.e., critical-infrastructure / likely-OT-adjacent) enterprise customer,
  and (b) continued investor confidence in 2026.** The utility deal is notable given Query has NO OT
  deployment story — they won a utility on the IT-side federation value prop, not OT depth. [INCONCLUSIVE:
  round size/valuation/customer name not disclosed.]
- Company size / revenue / customer count: **[INCONCLUSIVE]** — no reliable public 2026 figures.

### 1.8 Pricing — [INCONCLUSIVE]

No public pricing page surfaced in 2026 materials. Packaging tiers tied to Workers / Detections /
Pipelines are not disclosed. Pricing posture must be treated as unknown. [model-knowledge: federated
"search-in-place" vendors typically price per-connector + per-seat + usage; not Query-confirmed.]

---

## 2. Head-to-head: where Prism's day-2 position MATCHES / BEATS / LAGS Query

Leans (discussion input only). "Prism" = the C1–C9 day-2 decisions/leans per `SESSION-RESUME §2`,
which are CAPTURE-stage and not yet ratified.

| # | Dimension | Query (2026, cited) | Prism day-2 (C1–C9) | LEAN |
|---|-----------|---------------------|---------------------|------|
| (a) | **Query language** | FSQL — no formal grammar, no published semantics; markets cross-lake federated *joins*; FSQL REST API[1][8] | PrismQL — Chumsky grammar + DataFusion planner + **Kani VPs**; MATCH_RECOGNIZE; piped surface; entity-pivot; **hard-reject unbounded cross-source joins** (C3) | **BEAT on rigor; LAG on shipped breadth.** Prism is the *formally-grounded* engine; Query is the *shipping* engine. Query markets the very unbounded-join capability Prism deliberately restricts — frame as safety-vs-expressiveness, not a deficiency (§2.4). |
| (b) | **Detection** | Federated Detections **GA 2026-01**; scheduled FSQL + windows + thresholds + replay + source-coverage; **OOTB rule library**; SPL/KQL/Sigma→FSQL[1][10] | detection-as-query + MATCH_RECOGNIZE + **prism-native continuous operator** (phased); explicit temporal semantics in §14 YAML; WATCH…UNLESS absence-window | **MATCH on detection-as-query; BEAT on temporal rigor (continuous operator, absence windows, planner-picks-engine); LAG on GA + OOTB content.** Query ships *today* with a rule library; Prism's continuous operator is phased + has no OOTB content. |
| (c) | **ML / UEBA** | No published ML/UEBA primitive; agents do enrichment/scoring, not online learning[3-news][9-blog] | on-demand ML + online learning (C7, beyond §15 phasing) | **BEAT on intent (Prism has an explicit ML/online-learning thesis; Query does not).** Caveat: Prism's is phasing-sketch; Query's agent "scoring" skills may cover practical UEBA-lite use cases today. Verify Prism delivers before claiming. |
| (d) | **Satellite / OT / Purdue / air-gap** | **None.** SaaS-only; reverse-proxy hack for on-prem; no OT classes; won a *utility* on IT-side federation[3-onprem][5-fund] | **Satellite mesh (C2)** + multi-hop + per-hop mTLS + native **OT dissection** (flagship native-schema-on-read, §13.6); passive/read-only, TAP>SPAN, IEC-62443 placement | **DECISIVE BEAT — structural, Query has no answer.** This is Prism's clearest defensible moat. The Query utility win actually *validates* the OT/critical-infra market while exposing Query's IT-only ceiling. |
| (e) | **Retention / cache** | Pure-ephemeral federation + **egress to customer S3/Splunk (Security Data Pipelines)**; Security-Lake/Iceberg federation via Athena[7-news][7-comp] | **demand-driven RetentionCache** (RocksDB hot) + **Iceberg cold tier** (RETAIN) — SIEM-by-capability; `RETAIN <dur> AS name` PrismQL primitive | **BEAT on language-level retention (unique `RETAIN` primitive); MATCH-ish on cold-tier (both touch Iceberg).** BUT Query *ships* egress-to-customer-lake today; Prism's egress (G-11) is a candidate. Note: different philosophies — Query externalizes to customer-owned storage; Prism caches internally on demand. |
| (f) | **Deployment** | Multi-tenant SaaS (AWS-hosted)[2][6-deep] | per-analyst-MCP (S1) + **central deployment** (C1, MCP Streamable HTTP + stdio) + **bundled-Postgres control-plane** (storage taxonomy) + SQLite-at-edge | **BEAT on deployment optionality (on-prem/bundled/air-gap-capable); LAG on SaaS simplicity + time-to-value.** Query's "<15-min new source" + zero-ops SaaS is a real adoption advantage Prism's self-hosted model must consciously trade against. |
| (g) | **Connectors** | 50+ static/dynamic connectors, Query-engineered mappings, Configure-Schema for dynamic[10][8] | **TOML + WASM connector-plugins** (C4 dynamic-schema, builds on C3 descriptor) + capability descriptors | **BEAT on extensibility model (customer/community-authorable TOML+WASM, capability-aware pushdown); LAG on breadth + maturity (Query has 50+ battle-tested; Prism has 4 built-in + a plugin framework).** |
| (h) | **Agent-native** | **Query Workers (GA 2026-03)** — multi-stage, evidence-package, 9-check self-QA, no-auto-action; **MCP + A2A + BYO-Agent**[1][3-news] | MCP-native (S1 BYO) + **S3 server-hosted embedded agent**; AI-opaque creds; prompt-injection-hardened output | **MATCH on agent-native posture (parity REACHED, no longer whitespace); BEAT on trust guarantees (AI-opaque creds + PI-hardening); LAG on shipped agent depth + A2A.** Query's evidence-package + self-QA + dual-protocol is shipped; Prism's S3 agent has no equivalent evidence/QA contract and is MCP-only. |

### 2a. The cross-source-join positioning subtlety (for §2.4 honesty)

Query markets cross-lake federated joins as a *feature* ("join Sentinel cold-tier auth events to
CrowdStrike FDR EDR alerts in a single search"[8]). Prism C3 **deliberately hard-rejects unbounded
cross-source joins at plan-time** (restricting to inner-equi + DF 50.x sideways-information-passing).
Honest framing: this is **not** "Query can do something Prism can't" — it is a **conscious
safety/correctness trade**: Prism refuses the runaway-join footgun and surfaces the guard, where Query
appears to permit (and "translate-and-pray") the general case. Positioning should claim *"safe,
cost-aware joins with plan-time guards + EXPLAIN-visible pushdown"* — NOT *"more joins."* Overclaiming
join breadth would violate §2.4.

---

## 3. What Query does that Prism's C1–C9 plan does NOT cover (the real gap-check)

Adversarial/honest. These are capabilities/moves Query ships or markets that Prism's day-2 decisions
have **no current answer** for. Each is a candidate to address-or-consciously-decline.

| Gap | Query capability (cited) | Prism C1–C9 status | Severity |
|-----|--------------------------|--------------------|----------|
| **GAP-Q1 — OOTB detection content library** | Homepage: "use the out of the box library" of detections, + SPL/KQL/Sigma→FSQL conversion agents[1][10] | Prism has Sigma→PrismQL as a *candidate* epic (ADOPT-5, IN-bound translation only). **No OOTB rule library; no NL/SPL/KQL→PrismQL; no rule-translation OUT.** | **HIGH** — content library is a major adoption driver for detection products; Prism ships an empty detection engine. |
| **GAP-Q2 — Auditable agent evidence-package + automated senior-analyst self-QA** | Query Workers: Investigation Report + Query Log + IOC Ledger + **9-check Senior Analyst Review** on high-sev findings[3-news] | Prism S3 embedded agent: AI-opaque + PI-hardened, but **no defined evidence-package output contract, no self-QA/quality-gate, no IOC-ledger artifact.** | **HIGH** — this is the trust/auditability story buyers ask for; Prism's agent guarantees are about *input safety* (creds) not *output assurance* (evidence quality). |
| **GAP-Q3 — A2A (agent-to-agent) protocol** | Query supports **MCP AND A2A**[1] | Prism day-2 is **MCP-only** (C1 transport = MCP Streamable HTTP + stdio). A2A not in scope. | **MEDIUM** — A2A adoption is early; but Query's dual-protocol claim is a marketing checkbox Prism lacks. |
| **GAP-Q4 — Shipped egress / Security Data Pipelines (gold-dataset to customer lake)** | GA-ish: OCSF Parquet → S3/Azure/GCS/Splunk, partitioned, scheduled, zero-ops[7-news] | Prism G-11 (connector egress / normalized-result destinations) is a **candidate feature**, not decided. | **MEDIUM** — Prism's RETAIN/cache is *internal*; customers may also want OCSF gold-data *out* to their own lake. Query ships this; Prism hasn't decided. |
| **GAP-Q5 — Native Amazon Security Lake / Iceberg-via-Athena subscriber integration** | Query.AI is a **listed AWS Security Lake Subscriber**, querying Security Lake tables via Athena[7-comp][8] | Prism C5 (SIEM/Security-Lake federation + Iceberg cold tier) is **not started** (⏳). | **MEDIUM** — Query already ships the exact federation Prism's C5 plans. Prism must at least match the Security-Lake-subscriber pattern, not just the generic Iceberg cold tier. |
| **GAP-Q6 — Rich alert-destination fan-out breadth** | 11+ destinations (SNS, Sentinel, Pub/Sub, SecOps, Jira, Teams, Slack, PagerDuty, ServiceNow, Tines, Generic Webhook) + severity routing[6-deep] | Prism: findings emission exists, but **no documented multi-destination fan-out catalog / severity-routing layer** in C1–C9. | **LOW-MEDIUM** — table-stakes SOAR/ITSM plumbing; mostly engineering, not architecture. |
| **GAP-Q7 — Configurable data-visualization + graph-based investigation views (2026 roadmap)** | Query's stated 2026 roadmap: "configurable data visualization, graph-based views for investigations & decision support"[1] | Prism S2 console has results-explorer + Cytoscape (graph lib chosen, UI-D5) + ECharts; **dashboards/Summary-Insights analog + graph-investigation views are sketched (S2) but depth undecided.** | **LOW** — Prism has the building blocks (Cytoscape/ECharts) but no decided graph-investigation/dashboard depth spec. Mostly UX-spec work. |
| **GAP-Q8 — Time-to-value / zero-ops onboarding ("<15 min new source")** | SaaS, Query-engineered connector mappings, Configure-Schema wizard[10][11] | Prism: TOML+WASM plugins are powerful but **author-it-yourself**; central-deployment is self-hosted. No "<15-min, we-mapped-it-for-you" equivalent. | **MEDIUM (GTM, not tech)** — Query's managed-mapping + SaaS onboarding is a genuine adoption-friction advantage. Prism trades this for control/air-gap; ensure the trade is conscious (§2.4). |

**Gaps Prism may CONSCIOUSLY DECLINE** (with rationale): GAP-Q3 (A2A — defer until standard matures;
MCP-first is defensible); GAP-Q8's "we-map-it-for-you" managed service (conflicts with self-hosted/
air-gap thesis — but the *Configure-Schema wizard UX* should still be matched per ADOPT-6). Declining
must be an explicit human-directed decision per Canonical Principle Rule 3, not a silent omission.

---

## 4. Positioning / differentiation (tie to §2.4 honest-tradeoff)

### 4.1 Prism's defensible differentiation (where Prism genuinely leads)

1. **Formally-grounded query language.** PrismQL (Chumsky grammar + DataFusion planner + **Kani
   verification properties**) vs FSQL's unpublished, un-grounded syntax. This is a *category-of-one*
   claim Query cannot match without rebuilding their engine. **Defensible.** Honest caveat: rigor ≠
   feature-breadth; Query ships more *working* query surface today.
2. **OT / edge / air-gap via satellite mesh (C2) + native OT dissection (§13.6).** Query is SaaS-only
   with a reverse-proxy hack and **no OT classes**. Prism's passive/read-only OT, Purdue/IEC-62443
   placement, multi-hop mesh is a **structural moat**. The Query major-utility win *validates the
   market* while exposing Query's IT-only ceiling. **Most defensible single differentiator.**
3. **On-prem / bundled / data-residency-first deployment.** Bundled-Postgres control-plane,
   SQLite-at-edge, per-field residency policy-as-code, air-gap-capable built-in secret store. For
   MSSP/regulated/critical-infra buyers this is decisive; Query has no on-prem story. **Defensible.**
4. **Agent-native + MCP-first + AI-opaque trust guarantees.** Parity on *agent-native posture* is now
   reached (Query Workers + MCP/A2A), so the differentiator narrows to the **trust layer**: AI-opaque
   credentials (AD-017), prompt-injection-hardened output, credential values never transit AI context.
   Query stores creds centrally with no published AI-opacity guarantee. **Defensible on TRUST, NOT on
   agent-existence.** Do not claim "first agent-native" — Query ships agents too.
5. **Demand-driven retention as SIEM-replacement-by-capability.** `RETAIN <dur> AS name` +
   `FROM cache.<name>` is a unique language-level primitive; Query is pure-ephemeral (+ egress-to-
   customer-lake, a different model). **Defensible as a primitive**; honest caveat — Prism must prove
   the cache scales to SIEM-replacement claims before marketing them.

### 4.2 Where Query is genuinely stronger (no overclaiming — §2.4)

1. **Shipping maturity.** Federated Detections GA (2026-01), Query Workers GA (2026-03), Security Data
   Pipelines, 50+ connectors — all *in production*. Prism's C1–C9 are CAPTURE-stage. **Query ships;
   Prism plans.** This is the honest headline.
2. **Connector breadth + managed mappings.** 50+ Query-engineered connectors + Configure-Schema wizard
   vs Prism's 4 built-in + author-your-own TOML/WASM framework.
3. **Market presence + traction.** 2018-founded, $19.6M raised + 2026 note round, named major-utility
   enterprise deal, Cisco Investments on the cap table, AWS Security Lake partner listing. Prism has no
   market presence (pre-launch).
4. **SaaS simplicity / time-to-value.** Zero-ops, "<15-min new source," no infra to run. Prism's
   self-hosted model is more capable but higher-friction.
5. **OOTB detection content + rule-translation breadth** (SPL/KQL/Sigma/NL→FSQL + a rule library).
   Prism ships an empty detection engine with one *candidate* inbound translator.

### 4.3 One-paragraph positioning recommendation (lean)

Position Prism as **"the rigorous, deployable, agent-native federated query engine for the data Query
can't reach — OT, edge, air-gap, on-prem — with a formally-verified query language and credentials the
AI never sees."** Lead with the **structural moat (OT/edge/air-gap satellite mesh)** and the **trust
layer (AI-opaque creds + PI-hardening + formal PrismQL)** — the two places Query has *no answer*.
**Explicitly concede** maturity, connector breadth, and SaaS simplicity to Query (the §2.4-honest
move), and frame Prism's self-hosted/air-gap model as a *deliberate trade for the regulated/critical-
infra/MSSP buyer*, not a limitation. Do **not** claim "first/only agent-native" (Query ships Workers)
or "more joins" (Prism deliberately restricts them). The brief-reframe value-prop #5 amendment
(agent-native first + full console) is consistent with this and should stand.

---

## 5. Other federated-search / security-data-fabric competitors (brief landscape, for the B capstone)

So the B positioning isn't Query-only. Brief gap-check; all date-stamped; depth deliberately shallow.

| Competitor | What it is (cited) | Overlap with Prism | Note for B |
|------------|--------------------|--------------------|-----------|
| **Cribl Search** (Cribl, ~$600M raised, $3.5B val, Series E)[6-comp] | Search-in-place across S3/Security Lake/Azure Blob/GCS/Cribl Lake + **live API endpoints**; predicate+projection pushdown; OCSF/Parquet; own query syntax (`dataset="*_flowlogs" | limit`)[2-comp][5-comp][10-comp] | Closest *federated-search-in-place* analog after Query; **dominant SDPP market leader**. No formal grammar, no OT/air-gap, pipeline-centric. | The 800-lb gorilla on search-in-place. Prism differentiators (formal PrismQL, OT, air-gap, agent-trust) hold; maturity gap is larger than vs Query. |
| **Scanner.dev**[8-comp] | Cloud-native **security data lake** — index data in S3, **continuous streaming detections on full stream**, full-text search 100x faster than Athena | Continuous-detection overlap with Prism's continuous operator; but Scanner *indexes/centralizes in S3* (not pure-federated) | Different thesis (index-in-S3 lake vs federate). Prism's no-centralization + OT remain differentiators. |
| **Anvilogic** [model-knowledge — not in 2026 search hits; verify] | Multi-data-platform detection orchestration / "detection-as-code" across Splunk/Snowflake/etc. | Detection-orchestration overlap; data-platform-agnostic detection | [INCONCLUSIVE — no fresh 2026 citation surfaced; re-verify before citing in B.] |
| **Observo AI**[6-comp] | Security data pipeline / AI-driven telemetry optimization — **acquired by SentinelOne**[6-comp] | Pipeline layer (egress/optimization), not federated search | Now part of SentinelOne; signals pipeline-layer consolidation. |
| **Tenzir / VAST**[6-comp] | Open-source-rooted security data pipeline + node ("VAST" columnar store); founded 2017, $3.3M seed[6-comp] | Pipeline + storage; some detection | Edge/node model is loosely satellite-adjacent; worth a deeper look for C2/edge comparison. [INCONCLUSIVE on current 2026 product depth.] |
| **Abstract Security**[6-comp] | Security data pipeline + **"Lake Villa" hot/warm/cold tiers + real-time query over normalized data**; 2023-founded, $15M Series A (2024)[6-comp] | Tiered-lake + real-time-query overlaps Prism's RetentionCache tiering | Tiered-retention is a crowded space; Prism's *language-level* RETAIN primitive is the differentiator. |
| **Amazon Security Lake (native)**[5-comp][7-comp] | AWS-native OCSF data lake; centralizes AWS+SaaS+on-prem+cloud; queried by partners (Query, Cribl, Splunk Federated Analytics, Panther, ChaosSearch) via Athena | Prism C5 federates *over* Security Lake — Security Lake is a *source*, not a head-to-head competitor | Frame Security Lake as a **source Prism federates**, like Query/Cribl do — not a rival. Confirms OCSF-as-lingua-franca. |
| **Cribl/CeTu/Brava/Databahn/Realm (SDPP)**[6-comp] | Emerging security-data-pipeline platforms adding federated/NL query + cold-data retrieval from the pipeline layer | Pipeline-layer encroachment on federated search | The whole SDPP category is converging toward "federated search + retention + AI SOC" — exactly Prism's space. **Crowded.** Prism's formal-PrismQL + OT + air-gap + agent-trust are the cross-category differentiators. |
| **Matano, Panther, Scanner, Vega Analytics Mesh, Databricks Lakewatch** [model-knowledge / 2025 listing][4-comp] | Security-data-lake / cloud-SIEM / "analytics mesh" players | Adjacent SIEM-by-lake category | Mostly centralize-in-lake; Prism's no-centralization + OT differentiate. Verify any specific claim before B. |

**Landscape takeaway:** Federated search is **no longer a category-of-one** (Query). It is converging
from two directions — **federated-search vendors** (Query, Cribl Search) and **security-data-pipeline
platforms** (Cribl, Abstract, Databahn, Tenzir, Observo→SentinelOne, Onum→CrowdStrike) adding
search/retention/AI-SOC. **Consolidation is active** (SentinelOne→Observo, CrowdStrike→Onum). Prism's
durable cross-category differentiators against the *whole field*: (1) formally-verified query language,
(2) OT/edge/air-gap satellite mesh, (3) AI-opaque credential trust layer, (4) language-level demand
retention. **Maturity and breadth are the field's advantage over Prism across the board.**

---

## 6. Consolidated gaps Prism's day-2 plan should address (or consciously decline)

Ordered by severity. (Disposition = lean; human decides at brief-reframe / B capstone.)

1. **GAP-Q1 (OOTB detection content + rule-translation OUT) — HIGH.** Lean: **address.** An empty
   detection engine is an adoption non-starter. Scope an OOTB PrismQL detection library + extend
   ADOPT-5 to NL/SPL/KQL→PrismQL (not just Sigma-in). Candidate: E-RULE-XLATE expansion + new
   detection-content epic. Ties to C6 (detection-engine depth).
2. **GAP-Q2 (auditable agent evidence-package + self-QA gate) — HIGH.** Lean: **address.** Define an
   S3-agent output contract: Investigation Report + Query Log (replayable) + IOC Ledger + a self-QA
   quality-gate. This *complements* (does not duplicate) Prism's input-side AI-opaque trust story and
   closes the most-asked buyer question (output assurance). Ties to S3 (ADR-PROP-s3-agent-runtime) + C6.
3. **GAP-Q8 (time-to-value / Configure-Schema wizard UX) — MEDIUM.** Lean: **address the wizard, decline
   the managed-mapping-as-a-service.** Match Query's no-code Configure-Schema onboarding UX (ADOPT-6,
   C4) so self-hosting isn't author-everything-by-hand; but consciously decline "we-map-it-for-you SaaS"
   (conflicts with air-gap thesis). Human-direct the decline per Rule 3.
4. **GAP-Q5 (Amazon Security Lake subscriber pattern) — MEDIUM.** Lean: **address in C5.** Don't just do
   generic Iceberg cold-tier; match the Security-Lake-via-Athena subscriber pattern Query+Cribl+Panther
   all ship. C5 is not-started — fold this in.
5. **GAP-Q4 (egress / Security Data Pipelines analog) — MEDIUM.** Lean: **decide G-11 consciously.**
   Customers want OCSF "gold data" OUT to their own lake (Query ships it). Prism's internal RETAIN ≠
   external egress. Decide address-or-decline at B; if address, scope a connector-egress feature.
6. **GAP-Q3 (A2A protocol) — MEDIUM.** Lean: **consciously decline for day-2; revisit.** MCP-first is
   defensible; A2A is early. Record the decline with a revisit trigger (A2A standard maturity).
7. **GAP-Q6 (alert-destination fan-out catalog + severity routing) — LOW-MEDIUM.** Lean: **address as
   engineering.** Table-stakes SOAR/ITSM plumbing; mostly implementation, fold into C6/findings-emission.
8. **GAP-Q7 (graph-investigation views + configurable dashboards) — LOW.** Lean: **address as UX-spec
   depth** (Cytoscape/ECharts already chosen; spec the graph-investigation + Summary-Insights-analog
   depth in S2). Ties to day2-ui-design.

---

## 7. Honest costs & caveats (§2.4 discipline)

- **PRODUCT-FACT DECAY (highest-risk caveat).** Query ships fast: Federated Detections went preview→GA
  in ~2 months (late-2025 → 2026-01); Query Workers GA 2026-03; RBAC is "slated for QX-202X" and could
  land any time. **Every product-fact in this report is a 2026-06-27 snapshot and MUST be re-verified
  before any external positioning claim.** Highest-decay items: RBAC status (§1.6 — could flip from
  "no RBAC" to "RBAC shipped"), connector count, A2A maturity, pricing.
- **[INCONCLUSIVE] Query internals.** Credential encryption details (KMS/Vault/envelope) NOT published
  — AWS Secrets Manager is inferred from Alert-Destination docs[6-deep], not a full credential-custody
  spec. FSQL formal grammar NOT published (the "no formal grammar" claim is an *absence-of-evidence*,
  strong but not proof Query lacks one internally). On-prem/private-deployment: NOT publicly offered,
  but enterprise bespoke deployments cannot be ruled out. Pricing: entirely unknown.
- **Prism side is CAPTURE-stage, not shipped.** Every "Prism BEAT" in §2 is a *design intent* from
  C1–C9 leans/decisions that are `do_not_execute`, not ratified, not implemented. The honest headline
  remains **"Query ships; Prism plans."** Do not let any "BEAT" lean read as a shipped capability.
- **Deep-research vs search-tool divergence.** The `perplexity_research` deep pass UNDER-found the
  agentic/MCP layer (its corpus missed the 2026-03 Query Workers + MCP/A2A announcements and dated
  funding to 2021), while the targeted `perplexity_search` calls surfaced the current facts. Where they
  conflict, **the dated search-tool citations (2025-12 / 2026-03 / 2026-04 / 2026-06) supersede the
  deep-research's "as of 2021/2024-bounded" conclusions** (Source-of-Truth: later + more-specific wins).
  This is flagged inline via `[deep-research]` vs numeric source tags.
- **Naming-collision guard.** "Legible Query" (legiblequery.ai, an NL-to-SQL on-prem tool) surfaced in
  on-prem searches and is a DIFFERENT product — NOT the security Query.ai. Not conflated; excluded.
- **Anvilogic / Matano / Vega / Tenzir current depth — [INCONCLUSIVE].** Landscape entries flagged
  [model-knowledge] / [INCONCLUSIVE] lack fresh 2026 citations; re-verify before citing in the B capstone.
- **Leans are discussion input only.** Section 2/4/6 LEANs and dispositions are NOT decisions. The
  day-2 morph is gated on the brief-reframe HUMAN sign-off (§5.1). No file outside this one was modified.

---

## Sources (date-stamped)

Primary (Query, dated):
- [1] Query 2025 Year in Review — query.ai/resources/blogs/query-2025-year-in-review/ (2025-12-22): FSQL REST API, Mission-Specific Agents, NL/SPL/KQL/Sigma→FSQL agents, CoPilots, **MCP + A2A + BYO**, **Federated Detections GA Jan 2026**, 2026 roadmap (viz + graph views).
- [2] Query Product / Security Data Mesh Platform — query.ai/product/ (2025-11-24): cloud-native, API-bridge, no-pipeline thesis, dynamic-schema sources.
- [3-news] Query Workers launch — einpresswire.com/article/902555414 (2026-03-30): Workers GA, 3 workflows, evidence package, 9-check Senior Analyst Review, no-auto-action, BYO-Agent.
- [4-blog] "What Federated Search Taught Us" — query.ai/...case-for-a-security-data-mesh/ (2025-05-22): FSQL, OCSF, MCP server framing, copilot.
- [5-fed] Federated Search for Security — query.ai/federated-search/ (2026-06-01).
- [6-blog] "What MCP Means for Federated Security" — query.ai/...mcp.../ (2025-04-16): "practical MCP server," "USB-C hub for security data."
- [7-news] "Query.ai Launches Agents and Data Pipelines" — einpresswire.com/article/836291861 (2025-08-04): Query Agents + Security Data Pipelines preview; S3/Azure/GCS/Splunk; Snowflake/Databricks/Security Lake "coming soon."
- [8-blog] "Navigate Fast-Evolving Security Data Lakes" — query.ai/...security-data-mesh-federated-search/ (2025-08-12): Sentinel Data Lake, Amazon Security Lake, Delta/Iceberg-in-Athena, CrowdStrike FDR/LogScale, federated joins, auto translation to KQL/SQL/SPL.
- [9-blog] Network Activity Agent — query.ai/...network-activity-info-agent.../ (2025-07-24): 6 agents in preview, agent tool design, MITRE ATT&CK KB.
- [10] Query home — query.ai (2026-06-11): Query Workers, Federated Detections "50+ sources," OOTB rule library, "<15-min new source."
- [3-onprem] Query Compliance & Security — query.ai/legal/compliance-and-security/ (2024-10-03): **"we do not currently support RBAC … slated for QX-202X,"** SSO-login, "Query does not store any customer data. Period."

Funding/company (dated):
- [1-fund] Series A blog — query.ai (2023-06-15 / round 2021-10): $15M Series A, SYN Ventures + ClearSky + South Dakota.
- [2-fund] VentureBeat (2021-10-19): $15M Series A; ~$20M total; CEO Dhiraj Sharan.
- [3-fund] TheCompanyCheck (profile): founded 2018 Atlanta; $19.58M / 3 rounds; Cisco Investments, ClearSky, SYN, DNX.
- [4-fund] CB Insights financials (2024-09-25): latest round **Series A-II, Cisco Investments**.
- [5-fund] ClearSky Spring 2026 Newsletter — clear-sky.com/about/spring-2026-newsletter/ (2026-04-22): **March 2026 convertible-note round (ClearSky Security I); major-utility enterprise deal; Federated Detections launch.**
- [6-fund] Employbl: $19.6M total; Seed $4.6M + Series A $15M.
- [7-fund] SecurityWeek (2023-01-23 / launch): $4.6M seed (ClearSky + DNX + South Dakota).

Competitor landscape (dated):
- [2-comp] Cribl blog — Cribl Search 4.2 federated search (2023-08-11): all-cloud + native Security Lake; predicate/projection pushdown.
- [4-comp] CB Insights / CybersecTools Scanner alternatives (2025-01): Query.ai, Matano, Panther, Vega Analytics Mesh, Databricks Lakewatch, Exabeam, etc.
- [5-comp] SourceForge Amazon Security Lake vs Cribl Search compare: Security Lake OCSF centralization; Cribl Search-in-place + live API.
- [6-comp] SoftwareAnalyst Substack — "Rise of Security Data Pipeline Platforms" (2025-11-20): Cribl ($600M+/$3.5B), Abstract ($15M A 2024), Tenzir ($3.3M seed, 2017), Databahn, Realm, **Observo→SentinelOne, Onum→CrowdStrike**, CeTu, Brava; federated-search/retention/AI-SOC convergence.
- [7-comp] AWS Security Lake third-party integrations docs: **Query.AI = Security Lake Subscriber (queries via Athena)**; Cribl, Panther, Splunk Federated Analytics, ChaosSearch.
- [8-comp] scanner.dev (2026): index-in-S3 security data lake, continuous streaming detections, 100x-Athena search.

Deep-research synthesis (Sonar deep-research, reasoning_effort=high; **corpus bounded ~2021/2024 on
business facts, under-found 2026 agentic/MCP layer — superseded by dated search citations above where
they conflict**): `[deep-research]` tag = perplexity_research output file
`tool-results/mcp-perplexity-perplexity_research-1782577919220.txt`.

Prism-internal (read for non-contradiction, NOT modified):
- matured-vision-day2-requirements.md §10.2/§10.3/§10.4/§11.4/§11.5; day2-design-decisions/SESSION-RESUME-2026-06-27.md §2 (C1–C9 decisions/leans); prior research queryio-federated-search-2026-06-25.md.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source synthesis of Query's 2026 architecture/connectors/detections/pipelines/deployment/RBAC/funding (reasoning_effort=high; strip_thinking). NOTE: corpus was business-fact-bounded ~2021/2024 and under-found the 2026 agentic/MCP layer — its gaps were filled + superseded by the dated perplexity_search calls. |
| Perplexity perplexity_search | 4 | Current-as-of-2026 product facts: (1) Query 2026 features/MCP/agents; (2) Query funding 2024-2026; (3) Query on-prem/air-gap/OT/RBAC/residency; (4) federated-search competitor landscape (Cribl/Scanner/Abstract/Tenzir/Observo/Security Lake). These surfaced the load-bearing fresh facts (Query Workers 2026-03, Federated Detections GA 2026-01, MCP+A2A, "no RBAC" compliance page, March-2026 note round + utility deal, SDPP consolidation). |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — (no library-doc question) |
| Tavily (any) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Read | 4 | Prism non-contradiction context (matured-vision §10.2/§11.4/§11.5; SESSION-RESUME §2; prior 2026-06-25 Query research) + the deep-research output file. |
| Grep | 3 | Locate §10.2/§11.4/§11.5 anchors; scan deep-research file tail (funding/RBAC/conclusion). |
| Training data | ~2 areas | (a) Anvilogic/Matano/Vega current depth — flagged [model-knowledge]/[INCONCLUSIVE], not cited as fact; (b) generic federated-vendor pricing patterns — flagged [model-knowledge]. |

**Total MCP tool calls:** 5 (1 perplexity_research at reasoning_effort=high + 4 perplexity_search).
**Training data reliance:** low — every load-bearing product/funding/competitor fact is web-cited and
date-stamped; the only training-data content is explicitly flagged [model-knowledge]/[INCONCLUSIVE] in
the landscape table and excluded from the differentiation/gap conclusions.
**Resilience note:** the high-effort deep-research call succeeded on first attempt (no overload retry
needed); its output exceeded the token cap and was read from the saved tool-results file per protocol.
