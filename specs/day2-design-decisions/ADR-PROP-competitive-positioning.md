---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C10-1: C3 join framing CORRECTED — cost-based-degrade, not hard-reject"
  - "ADR-PROP-C10-2: Identity-vs-differentiation split — agent-native is identity, OT/trust is defensible wedge"
  - "ADR-PROP-C10-3: Honest concessions framing (binding per §2.4)"
  - "ADR-PROP-C10-4: Gap dispositions — all eight ADDRESS (no declines)"
  - "ADR-PROP-C10-5: Headline deferred to brief-reframe (B) — leading candidate recorded"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Side-analysis C10 capture; human-confirmed decisions 2026-06-27 session.
  Research basis: research/queryio-competitive-refresh-2026-06-27.md (C10 flagship —
  full head-to-head matrix §2, differentiation §4, gap table §3/§6, competitor landscape §5,
  honest-cost caveats §7, C3-join-framing correction §2a). Does NOT modify live ADR files,
  ARCH-INDEX.md, STATE.md, SESSION-HANDOFF.md, or any live factory artifact.
traces_to:
  - matured-vision-day2-requirements.md §10.2 (positioning thesis)
  - matured-vision-day2-requirements.md §11.4 (agent-native posture)
  - matured-vision-day2-requirements.md §11.5 (MCP-native transport)
  - matured-vision-day2-requirements.md §16.4 (C10 decisions log entry)
  - day2-design-decisions/ADR-PROP-capability-descriptor-pushdown.md (C3 — D-C3-1 join-guard cost-based-degrade)
  - day2-design-decisions/ADR-PROP-satellite-mesh.md (C2 — OT/edge/air-gap moat)
  - day2-design-decisions/ADR-PROP-central-deployment-access-layer.md (C1 transport — A2A scope)
  - day2-design-decisions/ADR-PROP-s3-agent-runtime.md (S3 agent — GAP-Q2 evidence-package)
  - day2-design-decisions/ADR-PROP-detection-engine-depth.md (C6 — GAP-Q1 + GAP-Q6)
  - day2-design-decisions/ADR-PROP-siem-lake-federation.md (C5 — GAP-Q5 fold)
  - day2-design-decisions/ADR-PROP-dual-deployment.md (deployment posture)
  - secret-subsystem-sketch.md (SS-26 AI-opaque credentials, AD-017)
  - CLAUDE.md (AD-017 AI-opaque credentials; SAP-1 structured event catalog)
  - proposed epics: E-DETECTION-CONTENT-001, E-RULE-XLATE-001 expansion, E-EVIDENCE-PACKAGE-001,
      E-A2A-TRANSPORT-001, E-EGRESS-PIPELINE-001, E-ALERT-DEST-FANOUT-001,
      E-GRAPH-INVESTIGATION-001, E-CONFIGURE-SCHEMA-WIZARD-001, E-MANAGED-MAPPING-001
---

# ADR-PROP — Competitive Positioning: Query.io Gap-Check & Disposition (C10)

> **STATUS: DECIDED 2026-06-27 (human).** This is a CAPTURE artifact for the side-analysis C10
> program. `do_not_execute: true`. Real ADR numbers and formal ARCH-INDEX.md rows are deferred to
> the morph execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

> **Research basis:** `research/queryio-competitive-refresh-2026-06-27.md` — C10 flagship research
> (one `perplexity_research` at `reasoning_effort=high` + four `perplexity_search` calls). Full
> head-to-head matrix (§2), differentiation analysis (§4), gap table (§3/§6), competitor landscape
> (§5), honest-cost caveats (§7). Load-bearing factual claims are web-sourced and date-stamped
> 2026-06-27. **All product facts decay rapidly; re-verify before any external positioning claim.**

> **Scope.** C10 covers: (1) the C3 join-framing correction (the research's §2a/§4.3
> mischaracterizes Prism's join behavior — corrected here); (2) identity-vs-differentiation split
> in agent-native positioning; (3) dispositions for all eight gaps identified in the research; (4)
> competitor landscape summary; (5) proposed epics and open questions. C10 does NOT cover:
> implementation mechanics (owned by C1–C9 artifacts); live pipeline spec changes; any live ADR.

---

## Context

The `research/queryio-competitive-refresh-2026-06-27.md` (C10 research) established:

1. **Query.io has materially accelerated on the agentic axis since the prior 2026-06-25 read.**
   Query Workers (GA 2026-03-30) delivers multi-stage auditable evidence packages, a
   nine-check "Senior Analyst Review" quality gate, BYO-Agent support, and dual-protocol
   **MCP + A2A** transport. The "agent-native / MCP-first" framing Prism planned to claim as
   whitespace is now **parity territory** — not whitespace.

2. **Query retains two structural weaknesses Prism's day-2 plan directly targets:** no
   on-prem/OT/air-gap deployment (SaaS-only; reverse-proxy workaround for on-prem), and no
   RBAC in-product (vendor's own 2024 compliance page: "we do not currently support RBAC in
   our product, but this is a product feature that is slated for QX-202X").

3. **Eight gaps exist** where Query ships capabilities Prism's C1–C9 plan does not yet answer.
   The human addressed all eight — no conscious declines on any gap.

4. **The research's §2a/§4.3 contains a load-bearing factual error** on Prism's C3 join
   behavior that must be corrected before any positioning artifact references it.

---

## Decision Ledger

### D-C10-1 — C3 JOIN FRAMING CORRECTED (Load-Bearing)

**CORRECTED 2026-06-27 (human). The research's §2a/§4.3 description of Prism's join behavior
is WRONG. This correction is binding on all downstream positioning artifacts.**

**The research states** (§2a, §1.1, §4.3): "Prism C3 deliberately hard-rejects unbounded
cross-source joins at plan-time" and "Prism is choosing to *hard-reject for unbounded
cross-source joins* at plan-time." The research even cautions "Do NOT claim 'more joins.'"

**The correct framing** (per human-confirmed C3 decision D-C3-1 in
`ADR-PROP-capability-descriptor-pushdown.md`):

Prism does **NOT** hard-reject cross-source joins. The C3 join guard is a
**cost-based-DEGRADE stack**, not a plan-time rejection. ALL join shapes are permitted:

| Join shape | Treatment |
|------------|-----------|
| Inner equi-join | DataFusion 50.x dynamic filtering (sideways-information-passing); `exact` cost guarantee |
| Outer/non-equi cross-source | Falls back to `NestedLoopJoinExec`; allowed; weaker cost guarantee |
| Bare Cartesian | Allowed; LOUDLY FLAGGED with plan-disclosure |

No production engine (Trino/DataFusion/Spark/BigQuery) hard-rejects unbounded cross-source
joins. Prism bounds cost at **execution time** via:
- Mandatory per-side row-caps
- Dynamic filtering (sideways-information-passing)
- EXPLAIN-visible pushdown disclosure
- Injected time-window (D-C3-2)

**The correct positioning claim:** *"safe, cost-guarded cross-source joins with
plan-visible degradation"* — a Prism **STRENGTH** vs Query's published-cost-guard-absent
"translate-and-pray" marketing of cross-lake federated joins.

**RESIDUAL:** The cost-based-degrade posture carries 3 pre-implementation verification items
(PIV-C3-1..3) recorded in `ADR-PROP-capability-descriptor-pushdown.md`. These are
pre-implementation residuals — the architecture decision is settled, the residuals are
implementation gates. Do not let residuals reintroduce the false "hard-reject" framing.

**Drop** the research's "don't claim more joins" caveat. It was premised on the wrong
characterization of C3. The correct caveat is: do not claim joins that are currently only
Cartesian (loudly flagged) as production-efficient without implementing the degrade stack first.

---

### D-C10-2 — Identity vs Differentiation Split (Agent-Native Framing)

**DECIDED 2026-06-27 (human). The agent-native framing must separate IDENTITY from
DIFFERENTIATION.**

Query Workers (GA 2026-03), MCP + A2A dual-protocol, and BYO-Agent support mean the
agent-native space is **parity territory**, not whitespace. Prism can no longer position
"agent-native" as a primary differentiator — it is the product **identity** (what Prism is),
but every serious competitor in this space will ship agent support.

**The defensible wedge (where Query has no answer):**

1. **OT/edge/air-gap satellite mesh** — structural, not a feature flag. Query is SaaS-only
   with a reverse-proxy workaround. No OT classes. Won a major utility on the IT-side
   value prop, confirming the market while exposing the IT-only ceiling. Prism's passive/
   read-only OT dissection, IEC-62443 placement, multi-hop mTLS satellite mesh is a moat
   Query cannot close without rebuilding their deployment model.

2. **AI-opaque trust layer** — Prism's AD-017 (credentials never transit AI context),
   per-tenant DEK envelope encryption (SS-26), prompt-injection-hardened output, and
   formally-verified parser-safety properties (Kani proofs bounding query size / recursion
   depth — VP-014/VP-015). Query stores connector credentials centrally in the SaaS control
   plane; no published AI-opacity guarantee.

**Framing rule:** say "the agent-native platform for the data Query can't reach" — the
agent-native carries identity; OT/air-gap + trust carries defensibility. Do not say
"first/only agent-native" — Query ships Workers.

---

### D-C10-3 — Honest Concessions (Binding per §2.4)

**DECIDED 2026-06-27 (human). These concessions are BINDING on all downstream positioning
artifacts per the §2.4 honest-tradeoff discipline.**

| Concession | Honest statement |
|------------|------------------|
| Shipping maturity | Query ships; Prism plans. Federated Detections GA 2026-01, Query Workers GA 2026-03, 50+ connectors, named major-utility enterprise deal. Prism's C1–C9 is CAPTURE-stage. |
| Connector breadth | 50+ Query-engineered connectors + Configure-Schema wizard vs Prism's 4 built-in + author-your-own framework. |
| SaaS time-to-value | "<15-min new source," zero-ops SaaS. Prism's self-hosted model is higher-friction. |
| OOTB detection content | Query ships an OOTB rule library + SPL/KQL/Sigma/NL→FSQL. Prism ships an empty detection engine with one candidate inbound translator. |
| First/only agent-native | **Do NOT claim.** Query ships Workers + MCP + A2A. |

**What NOT to concede:** Prism's cross-source join framing (D-C10-1 corrected), OT/air-gap
moat (structural, Query has nothing), AI-opaque trust (no Query equivalent), formally-verified
parser-safety properties (Kani proofs bounding query size / recursion depth — VP-014/VP-015;
Query's FSQL has no published formal grammar or parser-safety proofs), AI recommendation rigor
(W3C-PROV provenance + calibrated confidence + conformal-prediction uncertainty sets + mandatory
per-citation post-hoc faithfulness enforcement — D-C15-5, required from day one — a structural
output-assurance contract, not a heuristic review checklist; the structural answer to Query's
nine-check "Senior Analyst Review").

> **OVER-05 scoping note (positioning fidelity fold 2026-06-28):** All downstream artifacts
> citing Prism's Kani / formal-verification posture MUST use the scoped form
> "formally-verified parser-safety properties (Kani proofs bounding query size / recursion
> depth — VP-014/VP-015)" — NOT "formally-verified query language" or "Kani-verified grammar."
> VP-014 and VP-015 are bounded parser-safety proofs; they are NOT an end-to-end proof of
> PrismQL semantics or grammar. Exact VP scope: `VP-014` (query size limit),
> `VP-015` (recursion depth limit). Source: CLAUDE.md §Formal Verification + VP-INDEX.md.

---

### D-C10-4 — Gap Dispositions (All Eight ADDRESS — No Conscious Declines)

**DECIDED 2026-06-27 (human). ALL EIGHT gaps from the research are addressed. The human
chose the fuller path on every fork; no gap was consciously declined.**

#### D-C10-4-Q1 — GAP-Q1: OOTB Detection Content + Rule-Translation OUT (ADDRESS)

Research severity: HIGH.

**Decision:** Address. Scope an OOTB PrismQL detection-content library and extend rule
translation to NL/SPL/KQL→PrismQL (not just Sigma-in). The research's ADOPT-5 (Sigma→PrismQL
inbound) is extended to a full translation matrix:

- Inbound: Sigma→PrismQL (existing candidate), SPL→PrismQL, KQL→PrismQL, NL→PrismQL
- OOTB library: shipped PrismQL detection rules, curated by Prism (dogfooded Sigma-aligned format, §14.7 recipe)
- Architecture: pySigma-style Backend + ProcessingPipeline targeting OCSF taxonomy (per C6 lean); lossy edges → fidelity report (NEVER silent drop); Sigma correlation → MATCH_RECOGNIZE

**Proposed epics:** `E-DETECTION-CONTENT-001` (OOTB PrismQL library) + expansion of
`E-RULE-XLATE-001` to include NL/SPL/KQL translation OUT and inbound.

**Ties C6** (`ADR-PROP-detection-engine-depth.md` — Sigma→PrismQL feasibility confirmed,
examples in recipe library §14.7).

#### D-C10-4-Q2 — GAP-Q2: Auditable Agent Evidence-Package + Self-QA Gate (ADDRESS)

Research severity: HIGH.

**Decision:** Address. Define an S3-agent **OUTPUT contract** mirroring Query Workers' structure:

| Artifact | Description |
|----------|-------------|
| Investigation Report | Findings + recommended disposition + ATT&CK mapping + timeline + next steps |
| Replayable Query Log | Every search, source, result-count — replayable + auditable (ties §14.5 replay-link) |
| IOC Ledger | Every indicator typed, sourced, enriched |
| Self-QA quality-gate | Analog to Query's nine-check "Senior Analyst Review" on high-severity findings (evidence completeness, logic verification, missed indicators, severity calibration, blind-spot check) |

**Critical distinction:** this is the **output assurance** side of the trust story. Prism's
existing input-side trust (AD-017 AI-opaque credentials, prompt-injection hardening) covers
the input; this fills the output-assurance gap. These **complement, not duplicate** each other.

**Proposed epic:** `E-EVIDENCE-PACKAGE-001` (S3-agent output contract: Investigation Report
+ Query Log + IOC Ledger + self-QA gate).

**Ties S3** (`ADR-PROP-s3-agent-runtime.md`) and **C6** (detection findings + IOC emission).

#### D-C10-4-Q3 — GAP-Q3: A2A Protocol (ADDRESS — ADDED TO DAY-2)

Research severity: MEDIUM (lean had been "consciously decline").

**Decision:** ADDRESS — human chose to ADD A2A support alongside MCP in day-2 transport
scope. This is NOT a decline. Match Query's dual-protocol (MCP + A2A) posture.

**Scope:** A2A transport support as an additional protocol surface in the C1 central
deployment access layer (`ADR-PROP-central-deployment-access-layer.md`), alongside the
existing MCP Streamable HTTP + stdio transport.

**Note scope cost:** A2A is an early standard; implementation complexity + protocol stability
risk are real. The human's explicit choice is to bear that cost rather than cede the
positioning to Query. Pin the A2A spec version used; budget for spec evolution.

**Proposed epic:** `E-A2A-TRANSPORT-001`.

**Cross-ref C1 transport** (`ADR-PROP-central-deployment-access-layer.md`).

#### D-C10-4-Q4 — GAP-Q4: Egress / Security Data Pipelines Analog (ADDRESS)

Research severity: MEDIUM.

**Decision:** Address. Scope an OCSF gold-data **connector-egress** feature: route normalized
data OUT to customer-owned lake targets (S3, Azure Blob/ADLSv2, GCS, Splunk-class targets)
as OCSF Parquet, Hive-style partitioned.

**Critical distinction from internal RETAIN:** The internal `RETAIN <dur> AS name` →
RetentionCache is for demand-driven on-demand caching inside Prism. Connector-egress is a
separate flow: OCSF-normalized data pushed OUT to customer-owned external storage. These are
**distinct mechanisms** serving different use cases. Do not conflate.

**Proposed epic:** `E-EGRESS-PIPELINE-001` (connector-egress / OCSF gold-data pipeline,
distinct from internal RETAIN/RetentionCache).

**Resolves G-11** (connector egress / normalized-result destinations) as ADDRESS.

#### D-C10-4-Q5 — GAP-Q5: Amazon Security Lake Subscriber Pattern (ADDRESS — fold into C5)

Research severity: MEDIUM.

**Decision:** Address, folded into C5. Don't just implement generic Iceberg cold-tier; match
the **Security-Lake-via-subscriber pattern** that Query, Cribl, and Panther all ship.

Query.AI is a listed Amazon Security Lake Subscriber that queries Security Lake tables via
Athena. The C5 decision (D-C5-2, `ADR-PROP-siem-lake-federation.md`) already defines the
`S3` data-access subscriber binding as the default — this confirms C5's scope covers this gap.

**Action:** Ensure the C5 Security Lake connector explicitly matches the "AWS Security Lake
Subscriber" registration pattern (S3 data-access model + OCSF Parquet + partition projection
on `region/accountId/eventDay`). C5 E-LAKE-CONNECTOR-001 is the implementing epic; this is
a C5 **scope addition** confirmation, not a new epic.

**Cross-ref:** `ADR-PROP-siem-lake-federation.md` D-C5-2.

#### D-C10-4-Q6 — GAP-Q6: Alert-Destination Fan-Out + Severity Routing (ADDRESS — engineering)

Research severity: LOW-MEDIUM.

**Decision:** Address as engineering. Multi-destination fan-out catalog + severity-based
routing. Target destinations (mirrors Query's catalog, extensible):

**Tier 1 (day-2):** Amazon SNS, Azure Sentinel, Google Pub/Sub, Google SecOps, Jira,
Microsoft Teams, Slack, PagerDuty, ServiceNow, Tines (Cases + Webhooks), Generic Webhook.

**Routing:** Multi-destination (single finding can fan-out to N destinations); severity
threshold per destination; secrets from Prism's SS-26 Secret Broker (AD-017 — never in AI
context; note Query uses AWS Secrets Manager, SaaS-central).

**Proposed epic:** fold into **C6 findings-emission** (`E-ALERT-ROUTING-001`, already exists
in C6 proposed epics — see `ADR-PROP-detection-engine-depth.md` §14.8). Expand E-ALERT-ROUTING
scope to include the full multi-destination catalog + severity routing.

#### D-C10-4-Q7 — GAP-Q7: Graph-Investigation Views + Configurable Dashboards (ADDRESS — UX-spec depth)

Research severity: LOW.

**Decision:** Address as UX-spec depth in S2. The building blocks are already chosen:
Cytoscape.js (graph library, UI-D5) + ECharts (charts). The gap is specifying the
graph-investigation UX depth and a Summary-Insights-analog dashboard surface.

**Scope:**
- Graph-investigation views in S2 Investigations Console (entity-pivot → graph expansion;
  attack path visualization; lateral-movement graph from MATCH_RECOGNIZE output)
- Configurable summary/insights dashboard (alert-summary widgets; detection-health; source-coverage)
- Query's 2026 roadmap explicitly names "graph-based views for investigations & decision support"
  — Prism has the library, needs the spec

**Proposed epic:** `E-GRAPH-INVESTIGATION-001` (graph-investigation views + configurable
dashboards in S2 console). Ties `day2-ui-design/S2-investigations-console.md` (UX spec).

#### D-C10-4-Q8 — GAP-Q8: Time-to-Value / Onboarding (ADDRESS BOTH HALVES)

Research severity: MEDIUM.

**Decision:** Address BOTH halves. The human chose to pursue both, not just the wizard:

**Half 1 — No-code Configure-Schema wizard UX (ADDRESS):**
Match Query's no-code Configure-Schema onboarding UX: schema-introspect → sample → map to
OCSF → save. Makes self-hosted Prism competitive on onboarding friction without requiring
TOML authorship. Per ADOPT-6, ties C4 (`ADR-PROP-dynamic-schema-connectors.md`).

**Half 2 — Optional vendor-hosted managed-mapping service (ADDRESS, NOT decline):**
The human chose to pursue an optional managed-mapping service for SaaS-model customers —
Prism pre-maps connectors so SaaS customers don't author TOML from scratch. This is an
OPT-IN service for the SaaS operating model only (does not apply to MSSP-managed or
client-managed deployments; cross-ref `ADR-PROP-dual-deployment.md` D-DEPLOY-002).

**Note deployment scope:** The managed-mapping service adds a hosted-service surface to the
SaaS operating model. It is NOT available in air-gap / client-managed deployments by design
(those use the Configure-Schema wizard + TOML/WASM authoring). The SaaS ↔ air-gap trade-off
is explicit (per D-C10-3 honest concessions).

**Proposed epics:**
- `E-CONFIGURE-SCHEMA-WIZARD-001` (no-code wizard UX, all deployment profiles)
- `E-MANAGED-MAPPING-001` (optional vendor-hosted connector mapping service, SaaS-model only)

---

### D-C10-5 — Positioning Headline: Leading Candidate Recorded (Deferred to B)

**DECIDED 2026-06-27 (human). The final positioning headline is RATIFIED at the
human-gated brief-reframe (§5.1) / B capstone. This records the leading candidate only.**

**Leading-candidate synthesis:**

> *"The agent-native federated query platform for the data Query can't reach — OT/edge/air-gap
> — with credentials the AI never sees."*

- **"Agent-native"** carries the product IDENTITY (what Prism is). No longer a differentiator
  alone — Query ships Workers + MCP + A2A.
- **"For the data Query can't reach — OT/edge/air-gap"** carries the structural MOAT. This is
  Prism's clearest single differentiator. Query won a major utility on IT-side federation,
  validating the market while exposing the IT-only ceiling.
- **"Credentials the AI never sees"** carries the TRUST differentiator (AD-017 AI-opaque,
  per-tenant DEK, satellite-local resolution; Query stores creds centrally with no published
  AI-opacity guarantee).

**What this framing excludes (correctly):**
- "First/only agent-native" — Query ships (D-C10-3 binding concession)
- "Formally-verified query language" — accurate and defensible, but excluded from the
  one-liner for brevity; surfaces in the expanded pitch
- "Safe cost-guarded cross-source joins" (D-C10-1 corrected claim) — accurate but supporting,
  not headline

**This is the leading input to B, not a locked decision.** The final headline is ratified
only after the brief-reframe human gate (§5.1).

---

## Proposed Epics (Feeding B)

All epics below are PROPOSED; registration in STORY-INDEX deferred to morph / B capstone.

| Epic ID | Description | Tied Gap | Tied Capture Artifact |
|---------|-------------|----------|-----------------------|
| `E-DETECTION-CONTENT-001` | OOTB PrismQL detection-content library | GAP-Q1 | C6 / `ADR-PROP-detection-engine-depth.md` |
| `E-RULE-XLATE-001` (expansion) | Extend Sigma-in translator to NL/SPL/KQL→PrismQL OUT | GAP-Q1 | C6 |
| `E-EVIDENCE-PACKAGE-001` | S3-agent output contract: Investigation Report + Query Log + IOC Ledger + self-QA gate | GAP-Q2 | S3 / `ADR-PROP-s3-agent-runtime.md` |
| `E-A2A-TRANSPORT-001` | A2A protocol support alongside MCP in C1 transport layer | GAP-Q3 | C1 / `ADR-PROP-central-deployment-access-layer.md` |
| `E-EGRESS-PIPELINE-001` | Connector-egress: OCSF gold-data OUT to customer-owned lake (S3/Azure/GCS/Splunk-class) | GAP-Q4 | C5 adjacent |
| `E-LAKE-CONNECTOR-001` (scope add) | C5 epic — add explicit AWS Security Lake Subscriber pattern | GAP-Q5 | C5 / `ADR-PROP-siem-lake-federation.md` |
| `E-ALERT-ROUTING-001` (expansion) | Multi-destination fan-out catalog + severity routing | GAP-Q6 | C6 / `ADR-PROP-detection-engine-depth.md` §14.8 |
| `E-GRAPH-INVESTIGATION-001` | Graph-investigation views + configurable dashboards in S2 | GAP-Q7 | S2 UI / `day2-ui-design/S2-investigations-console.md` |
| `E-CONFIGURE-SCHEMA-WIZARD-001` | No-code Configure-Schema wizard UX (all profiles) | GAP-Q8 | C4 / `ADR-PROP-dynamic-schema-connectors.md` |
| `E-MANAGED-MAPPING-001` | Optional vendor-hosted managed connector mapping (SaaS-model only) | GAP-Q8 | `ADR-PROP-dual-deployment.md` |

---

## Competitor Landscape (Brief — For B Capstone)

Full detail in `research/queryio-competitive-refresh-2026-06-27.md` §5. Summary for B:

**Query.io** — $19.6M raised + 2026 convertible note; GA products: Federated Detections (2026-01),
Query Workers (2026-03), Security Data Pipelines; 50+ connectors; AWS Security Lake Subscriber
partner. The primary benchmark. Prism's moat: OT/air-gap + AI-opaque trust; Query's advantage:
ships today.

**Cribl Search** — ~$600M raised, $3.5B valuation. Search-in-place across S3/Security Lake/Azure
Blob/GCS + live API. The 800-lb gorilla on search-in-place. More mature than Query on the
pipeline side. No formal grammar, no OT/air-gap. Prism differentiators hold; maturity gap larger.

**Scanner.dev** — index-in-S3 security data lake, continuous streaming detections. Different
thesis (index vs federate). Prism's no-centralization + OT differentiate.

**Observo → SentinelOne; Onum → CrowdStrike** — active consolidation signals. SDPP category
converging toward "federated search + retention + AI SOC" — Prism's exact target space.

**Tenzir/VAST** — open-source-rooted security data pipeline + node. Edge/node model loosely
satellite-adjacent; worth a C2 comparison. [INCONCLUSIVE on current 2026 product depth.]

**Abstract Security** — tiered-lake + real-time query; $15M Series A 2024. Tiered-retention
overlap. Prism's language-level RETAIN primitive is the differentiator.

**Amazon Security Lake** — OCSF data lake. Prism federates **over** it (C5 subscriber) — a source
not a rival. Same pattern Query, Cribl, Panther all use. Confirms OCSF-as-lingua-franca.

**Anvilogic, Matano, Vega Analytics Mesh, Tenzir (depth), Panther** — [INCONCLUSIVE / model-knowledge;
no fresh 2026 citations surfaced. Must re-verify before citing in B.]

**SDPP consolidation takeaway:** Federated search is no longer a niche. Converging from two
directions — federated-search vendors (Query, Cribl Search) and security-data-pipeline platforms
(Cribl, Abstract, Databahn, Tenzir) adding search/retention/AI-SOC. Prism's durable
cross-category differentiators: (1) formally-verified query language, (2) OT/edge/air-gap
satellite mesh, (3) AI-opaque credential trust layer, (4) language-level demand retention.
Maturity and breadth favor the field over Prism across the board.

---

## Open Questions (OQ-C10-*)

### OQ-C10-1 — Product-Fact Re-Verification (HIGHEST DECAY)

Every product fact in `research/queryio-competitive-refresh-2026-06-27.md` is a
**2026-06-27 snapshot**. Must re-verify before any external positioning claim.

Highest-decay items (could flip before B capstone is delivered):
- **RBAC status** — Query's own compliance page says "slated for QX-202X"; could ship at any
  time. If Query ships RBAC before B, the RBAC differentiator narrows significantly.
- **Connector count** — stated as "50+" on homepage; grows continuously.
- **A2A standard maturity** — early; spec could stabilize or diverge from Query's implementation.
- **Pricing** — entirely unknown; could change positioning economics.

Re-verify at: (1) brief-reframe session (§5.1, B capstone), and (2) any external messaging event.

### OQ-C10-2 — Final Headline Ratification (Deferred to B)

The leading-candidate positioning synthesis (D-C10-5) is the primary input to the brief-reframe
/ B capstone. The final headline statement is RATIFIED only after the human-gated §5.1 sign-off.

No artifact outside this capture should treat D-C10-5 as a locked headline.

### OQ-C10-3 — Competitor Landscape [INCONCLUSIVE] Re-Verification (Before B)

The following landscape entries lack fresh 2026 citations and carry explicit [INCONCLUSIVE]
flags in the research. Must re-verify before citing in B:

- Anvilogic (no 2026 search hits; detection-orchestration overlap)
- Matano (2025 listing only; cloud-SIEM/lake category)
- Vega Analytics Mesh (2025 listing only; analytics mesh category)
- Tenzir current product depth (2026 depth unverified; edge/node model C2-adjacent)

### OQ-C10-4 — C3 Residual PIV-1..3 (Pre-Implementation Verification)

The corrected join positioning (D-C10-1 — "safe cost-guarded cross-source joins with
plan-visible degradation") carries 3 pre-implementation verification items (PIV-C3-1..3)
recorded in `ADR-PROP-capability-descriptor-pushdown.md`. These are morph-time gates.

Until PIV-C3-1..3 are verified, do NOT claim the degrade-stack behavior as shipping — claim
it as the designed and committed architecture. The positioning is accurate for the B
brief-reframe; it must be re-validated against implementation results before any external claim.

### OQ-C10-5 — A2A Transport Spec Version Pin

E-A2A-TRANSPORT-001 depends on a standards-body A2A specification. Before implementing:
- Identify the current A2A spec version (Google/Google Workspace A2A Protocol 2025/2026)
- Pin the spec version in the epic story
- Assess API stability risk (early standard; changes between now and morph)

---

## Honest Costs and Risks

| Item | Cost / Risk |
|------|-------------|
| **Parity reached on agent-native** | Query Workers + MCP + A2A + BYO-Agent is GA. "Agent-native" is Prism's identity, not a differentiator. The positioning must be rebuilt around OT/trust moat, not agent-existence. |
| **All 8 gaps addressed = scope expansion** | The human chose the fuller path on every fork. Combined, these add ~10 new epics to the day-2 scope. Each represents real engineering cost. B capstone must include an honest scope-load assessment. |
| **A2A early standard risk** | A2A is an early protocol (announced 2025-12, per research). Implementing against an unstable spec carries maintenance risk. OQ-C10-5 must be answered before morph. |
| **E-MANAGED-MAPPING-001 hosted surface** | The managed-mapping service adds a SaaS-model hosted API surface. New attack surface; security-reviewer required before ship. Does NOT apply to air-gap/client-managed profiles. |
| **PRODUCT-FACT DECAY** | Every Query product claim is a 2026-06-27 snapshot. RBAC in particular ("slated for QX-202X") could flip. All external positioning claims must be re-verified at B and before any public statement. |
| **Competitor landscape [INCONCLUSIVE] flags** | OQ-C10-3 items carry no 2026 citations. B must not cite them as facts. Re-verify or omit. |
| **"Prism plans; Query ships" remains the honest headline** | C1–C9 are CAPTURE-stage, `do_not_execute`. Every "BEAT" in the head-to-head is design intent, not shipped capability. Concession D-C10-3 is binding. |

---

## Downstream Spec Dependencies (Note — Not Actioned Here)

SAP-1 obligations for new epics (BC-2.16.002 catalog rows needed at morph):
- `event_type = "agent.evidence_package.produced"` — emitted when S3-agent completes an
  evidence package. Fields: investigation_id, query_log_entry_count, ioc_ledger_count,
  self_qa_gate_result, severity; audit role = output assurance; recurrence = per investigation.
- `event_type = "agent.self_qa.finding"` — emitted per self-QA check. Fields:
  investigation_id, check_name, result, detail; audit role = quality gate transparency.
- `event_type = "egress.pipeline.delivered"` — emitted per egress batch. Fields:
  destination_id, record_count, ocsf_version, bytes, partition; audit role = egress audit.
- `event_type = "transport.a2a.connection"` — emitted on A2A protocol handshake. Fields:
  agent_id, protocol_version, endpoint; audit role = transport observability.
- Alert-destination fan-out events (ties E-ALERT-ROUTING-001 expansion) — per-destination
  delivery events; exact schema deferred to C6 spec at morph.

All events flagged here; BC-2.16.002 amendment is morph-time work (SAP-1 obligation
per CLAUDE.md).

---

## Ripple Effects (Must Be Picked Up at Morph Time)

| Affected area | Ripple |
|---------------|--------|
| **matured-vision §10.2 / §11.4** | Agent-native positioning framing must be updated: identity NOT differentiator; WEDGE = OT/trust. Morph-time prose update (PO action). |
| **ADR-PROP-capability-descriptor-pushdown.md §D-C3-1** | The D-C10-1 correction confirms D-C3-1 cost-based-degrade is the canonical framing. No change needed to C3 artifact — it is already correct. This document IS the source for the corrected join claim. |
| **ADR-PROP-s3-agent-runtime.md** | E-EVIDENCE-PACKAGE-001 adds the output-contract scope to the S3 agent. Needs a scope-addition note at morph. |
| **ADR-PROP-central-deployment-access-layer.md** | E-A2A-TRANSPORT-001 adds A2A as a transport protocol. Needs a transport-scope-addition note at morph. |
| **ADR-PROP-dual-deployment.md** | E-MANAGED-MAPPING-001 is SaaS-model-only. Must be reflected in the deployment-profile capability matrix at morph. |
| **ADR-PROP-siem-lake-federation.md** | GAP-Q5 fold confirms E-LAKE-CONNECTOR-001 scope includes Security Lake Subscriber pattern. Already captured in C5 D-C5-2 — no change needed. |
| **ADR-PROP-detection-engine-depth.md §14.8** | E-ALERT-ROUTING-001 expansion (full fan-out catalog) + E-DETECTION-CONTENT-001 + E-RULE-XLATE-001 expansion are C6 scope additions. Morph-time scope-addition notes. |
| **BC-2.16.002 §Postconditions** | Five new SAP-1 event types flagged in §Downstream Spec Dependencies above (morph-time BC work). |
| **ARCH-INDEX.md** | At morph time: new subsystem entries for E-EVIDENCE-PACKAGE-001, E-A2A-TRANSPORT-001, E-EGRESS-PIPELINE-001, E-DETECTION-CONTENT-001 (numbers assigned at morph). |
| **Brief-reframe B capstone** | D-C10-5 leading-candidate headline is the primary input. Positioned framing, honest concessions (D-C10-3), and gap dispositions (D-C10-4) must all be reflected in B. |
