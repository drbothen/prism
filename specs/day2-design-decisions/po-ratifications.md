---
document_type: po-ratification-draft
status: capture
do_not_execute: true
provenance: "2026-06-26 side-analysis — PO ratification drafts; PENDING human brief-reframe sign-off; does NOT modify live brief/PRD; separate from live factory."
traces_to:
  - specs/matured-vision-day2-requirements.md §11.3 (value-prop #5 amendment)
  - specs/matured-vision-day2-requirements.md §15.9 (three-ways-to-long-baseline)
  - specs/matured-vision-day2-requirements.md §2.1 (corrected central framing)
  - specs/matured-vision-day2-requirements.md §2.3 (five value-props)
  - specs/matured-vision-day2-requirements.md §2.4 (honest tradeoffs)
  - specs/matured-vision-day2-requirements.md §16.2 (confirmed decisions)
gate: "All three items below are PO-ratified-ready DRAFTS. Execution is GATED on §5.1 human brief-reframe sign-off. Do not apply to product-brief.md, prd.md, domain-spec, or any live artifact until that gate is explicitly cleared."
---

# PO Ratification Drafts — Day-2 Brief-Reframe

> **CAPTURE ONLY.** This file stages the product-owner ratifications for three specific
> changes that are pending the brief-reframe human sign-off (matured-vision §5.1). None
> of these modify live specs. Each item is marked PO-RATIFIED-READY where the PO has
> reviewed and approved the draft wording; each is marked GATED on the human sign-off.
>
> Cross-reference map:
> - Item 1 (value-prop #5) amends §2.3 #5 of the product brief and prd.md §1.2.
> - Item 2 (§2.4 tradeoff softening) amends product-brief.md §2.4 and any prd.md prose
>   referencing the deep-historical-analytics limitation.
> - Item 3 (§1.x framing) targets product-brief.md §1 headline and deployment model.

---

## Item 1 — Value-Prop #5 Rewrite

**Source:** matured-vision §11.3 (drafted replacement, human-approved 2026-06-25).
**Supersedes:** Current §2.3 value-prop #5: *"Federated search built for the analyst's
agent, not for yet another browser tab."*
**Gate:** Brief-reframe human sign-off (§5.1).
**Status:** PO-RATIFIED-READY.

### Ratified Replacement Wording

> **5. Meets every analyst where they work — agent-native first, full browser console
> included.**
> Prism is MCP-native so an analyst's AI agent can drive it directly
> (bring-your-own-agent), AND it ships a full-fidelity investigations console with
> built-in AI for analysts who prefer a GUI — plus a right-click browser extension for
> triage. One federated query engine, four surfaces (S1–S4), one set of guarantees:
> credentials the AI never sees, output hardened against prompt injection, the same
> PrismQL underneath. The agent is the differentiator; the console makes it usable by
> the whole SOC.

### PO Rationale

**Multi-surface, multi-persona.** The original #5 positioned Prism as explicitly
rejecting the browser tab ("not for yet another browser tab"). The human directive
(2026-06-25) overrides this: Prism stays agent-native first AND ships a full browser
experience. The replacement wording preserves the MCP-native differentiator as the
lead ("agent-native first") while removing the self-limiting rejection of GUI. The
four surfaces (S1 MCP/BYO-agent, S2 full investigations console, S3 embedded AI, S4
browser extension) are now treated as additive, not competing.

**Agent is the differentiator; console is the accessibility layer.** The last sentence
of the replacement wording encapsulates the positioning logic: the formal
guarantee — AI-opaque credentials, prompt-injection-hardened output, formal PrismQL
grammar — is the differentiator that no competitor currently matches. The browser
console makes that differentiator accessible to the full SOC (not only the subset who
run Claude Code). This is an expansion, not a dilution.

**Competitive whitespace preserved.** The research (matured-vision §2.3 citation
[V-4][V-11][V-12]) established that AI-native MCP is Prism-unique among the cited 39
sources. The new wording preserves that claim ("agent-native first") while noting the
additional surface for completeness. The differentiation claim remains defensible.

**Downstream propagation required at brief-reframe:**
- product-brief.md §2.3 value-prop list: replace #5 verbatim.
- prd.md §1.2 Competitive Differentiators: the existing differentiator entry for
  MCP-native needs to be expanded to reference both the MCP and console surfaces.
- differentiators.md: if a "MCP-native / agent-first" differentiator entry exists,
  update its description to match the multi-surface framing.

---

## Item 2 — §2.4 Honest-Tradeoff Softening

**Source:** matured-vision §15.9 (three-ways-to-long-baseline), §2.4 (original honest
tradeoffs), §15.3 (three retention tiers), §15.4 (online-learning / "model is the
memory").
**Gate:** Brief-reframe human sign-off (§5.1).
**Status:** PO-RATIFIED-READY.

### Current §2.4 Tradeoff Language (to be amended)

The existing tradeoff text reads (paraphrased from §2.4):

> Federation is NOT equivalent to a centralized lake for:
> - Deep historical analytics and complex long-window correlation (lake wins on retained data).
> - Very large-scan queries (PB-scale interactive; object-store round-trips add seconds; not hot-path).

### Ratified Replacement Wording for the Deep-Historical-Analytics Limitation

Replace the existing first bullet with the following. Retain the second bullet
(PB-scale interactive) unchanged — that limitation holds.

---

**Deep historical analytics and long-baseline correlation.** Federation without
caching is weakest here: a query that needs years of history must go live to the
source, which may not retain it at all. Prism closes the gap three ways — choose per
use case:

1. **`RETAIN` raw data to the cold tier** (Apache Iceberg on object storage). Exact,
   replayable, correct for forensic queries and detection backtesting. Cost scales with
   retention volume.

2. **Online-learn a model** (incremental/streaming anomaly and behavior algorithms).
   Prism updates a compact per-tenant, per-entity model from every data touch and from
   optional scheduled baseline-refresh pulls. The model retains a long-memory summary
   of behavioral history at model-sized cost, not data-sized cost. This is lossy and
   not replayable, but it enables long-memory UEBA (user and entity behavior analytics)
   without store-everything.

3. **Federate into an existing lake** (§3.5 federate-into dual stance). Prism queries
   an existing SIEM or security lake in place — Amazon Security Lake, Splunk, Sentinel,
   Snowflake. The lake retains the data; Prism queries it on demand.

**Remaining honest limits.** PB-scale interactive queries and multi-year cold forensics
where neither the cold tier nor a federated lake is present still favor a store-everything
lake. Prism's cold Iceberg tier and online-learning do NOT claim equivalence with a
purpose-built data lake at full PB retention. The claim is: Prism narrows the gap
materially for the typical MSSP use case (detection-window correlation, investigative
entity queries, behavioral anomaly over months) without requiring the analyst or operator
to maintain a full data lake.

---

### PO Rationale

**The online-learning tier changes the calculus without abandoning honesty.** The
original §2.4 had two honest concessions: deep historical analytics and PB-scale
interactive. The second (PB-scale interactive) still stands and is retained. The first
(deep historical analytics) was correct for an ephemeral-only Prism — it is no longer
fully accurate now that Prism has (a) a cold Iceberg retention tier (§3.3 addendum),
(b) an online-learning model tier (§15.4, human-confirmed), and (c) the federate-into
dual stance (§3.5). All three were confirmed in the 2026-06-25 session.

**Three ways to long baseline (§15.9) is the organizing principle.** The matured
vision (§15.9) explicitly directs: "The §2.4 honest tradeoff should be updated by PO
to reflect this." This item executes that directive. The replacement wording:
- Names the three mechanisms precisely (raw cold tier, model tier, federate-into).
- Does not overclaim: PB-scale interactive and full multi-year forensic cold storage
  without a lake are still listed as remaining lake-favorable cases.
- Preserves the positioning discipline from §2.5 (neutral-incentive credibility): the
  federate-into option is framed as a capability the analyst can use, not a concession.

**Iceberg cold tier unifies with the Security Lake connector.** The §3.3 addendum
establishes that the cold-tier read path and the Security Lake read path are the same
DataFusion + Iceberg TableProvider mechanism. This means the cold tier is not a new
architectural bet — it is the natural generalization of an already-confirmed connector.
This strengthens the credibility of claiming cold-tier retention without adding
complexity.

**Downstream propagation required at brief-reframe:**
- product-brief.md §2.4: replace the deep-historical-analytics bullet with the three-
  ways language above.
- architecture-concept.md "Why Ephemeral" section: the §5.2 checklist already flags
  "ephemeral by default, cache by demand." The three-ways language should be referenced
  there as well.
- prd.md §1.4 Out of Scope: the §5.3 checklist item directing removal of "SIEM/log
  storage" should also note that long-retention historical archive (store-everything)
  remains out of scope; on-demand cold retention via Iceberg is IN scope.

---

## Item 3 — §1.x Framing Alignment

**Source:** matured-vision §2.1 (corrected central framing), §2.2 (per-analyst to
central reconciliation), §11.3 (multi-surface UI), §16.2 confirmed decisions #1 and #5.
**Gate:** Brief-reframe human sign-off (§5.1).
**Status:** PO-RATIFIED-READY.

> **Scope constraint.** This item lists TARGETED edits to the brief §1.x framing. It
> does NOT rewrite the full brief. The edits are minimal and surgical: change the
> headline framing, add the central deployment mode, update the per-analyst description,
> add Prism Satellite as a named component. Everything else in §1.x stays as-is pending
> the full §5.1 sign-off process.

### Ratified Targeted Edits

**Edit 1 — Lead sentence / headline framing.**

Change from (current brief lead, paraphrased from §2.1 context):
> "complete MSSP security operations platform" [with federation as a support mechanism]

Change to (the corrected central framing from §2.1, verbatim):
> Prism is an ephemeral federated query engine for security operations — central,
> multi-tenant, AI-native via MCP. It queries any source valuable to a security analyst,
> in place, normalizing security telemetry to OCSF and other sources to their native
> structured schema, on demand. Demand-driven caching delivers SIEM-grade stateful
> detection and historical correlation without store-everything cost. Resilient to its
> sources: fail-fast timeouts, partial-result semantics, and auto-recovery without
> restart.

**Rationale.** The brief currently leads with platform completeness and buries the
federated query engine claim. The architecture (§2.1), competitive research, and the
confirming decision (DC-010, DC-005) all establish that federation IS the product. The
headline must reflect this. "Complete MSSP security operations platform" remains true as
a consequence of Prism's capabilities — but it is NOT the lead differentiator and MUST
NOT be the opening claim.

---

**Edit 2 — Deployment model description.**

Change from (current, paraphrased):
> Per-analyst MCP server in Claude Code (stdio transport).

Change to (multi-mode framing):
> Prism deploys in two modes: (a) per-analyst — a local stdio MCP server in Claude
> Code, for single-analyst or developer use; and (b) central — a multi-tenant service
> with an HTTP/streamable transport, serving multiple analysts simultaneously over one
> shared query engine. The data-plane is multi-tenant in both modes. The per-analyst
> stdio deployment remains fully supported.

**Rationale.** The existing per-analyst stdio framing is not wrong — it describes the
current implementation correctly. The DC-005 decision (human-confirmed) pivots the
vision to the central deployment mode as the target for day-2. The amended framing:
- Retains stdio as a valid and explicitly supported mode (it is architecturally
  correct and technically complete; it does not disappear).
- Introduces central deployment at the brief level so it is scoped from the start.
- Does not claim the central mode is already implemented — it is day-2 scope.

---

**Edit 3 — Multi-surface UI mention.**

In the §1.x In Scope list, add a new item after the MCP-native entry:

> Multi-surface analyst access: MCP-native (S1, bring-your-own-agent), full browser
> investigations console with built-in AI (S2+S3), and browser extension for IOC
> right-click pivot (S4). All surfaces use the same PrismQL engine, credentials remain
> AI-opaque on every path.

**Rationale.** The human directive (2026-06-25, captured in §11.3 and confirmed in
§16.2 item 1) makes multi-surface UI a first-class day-2 deliverable. The brief must
scope it at §1 so UX, architecture, and story decomposition agents treat it as
in-scope from the start, not as a late addition.

---

**Edit 4 — Prism Satellite as a named §1.x component.**

In the §1.x In Scope list, add:

> Prism Satellite — a remote query executor deployed at a client site, OT/ICS
> Purdue-layer boundary, or air-gapped enclave. Satellites use an outbound dial-home
> connection to the central Prism service and can chain (satellite → satellite → Prism)
> for multi-hop topologies. Per-hop mutual authentication and partial-failure
> propagation through the chain are non-negotiable.

**Rationale.** DC-006 confirms "Prism Satellite" as the human-approved component name.
Section 3.2 scopes its topology and use cases. The satellite is a core architectural
pillar (OT/ICS coverage, air-gap bridging, MSSP nested topology). Naming it at §1
ensures it is treated as a first-class scope item in architecture ADRs (ADR-050-range),
domain-spec entity additions, and story decomposition.

---

**Edit 5 — Memory budget note.**

In the §1.x technical constraints or NFR summary (wherever the current "512MB process /
200MB per-query" cap is stated):

Change from:
> 512MB process budget, 200MB per-query

Change to:
> Configurable process budget — default 512MB for per-analyst laptop deployment;
> server-sized GB-range recommended for central deployment. The 200MB per-query cap
> applies to in-memory query working set; the demand-driven cache (hot RocksDB tier)
> operates within the configurable process budget. Human decision DC-004 (2026-06-24).

**Rationale.** DC-004 explicitly raises the memory budget ceiling. The current
per-laptop 512MB figure is correct for the per-analyst stdio mode but artificially
constrains the central deployment design. The amended note preserves the 512MB figure
as a valid operating point (per-analyst laptops still exist) while opening the design
space for the central service.

---

## Summary for Human Review

Three PO-ratified-ready items, each GATED on the §5.1 brief-reframe human sign-off:

| # | Item | Source in Matured Vision | Key change |
|---|------|--------------------------|-----------|
| 1 | Value-prop #5 rewrite | §11.3 (human-approved draft) | Adds multi-surface (MCP + console + extension); removes the "not another browser tab" rejection; preserves agent-native as the lead differentiator |
| 2 | §2.4 tradeoff softening | §15.9, §15.3, §15.4 | Replaces the blanket "lake wins on deep history" concession with the three-ways-to-long-baseline framing (cold Iceberg RETAIN, online-learn model, federate-into lake); retains PB-scale interactive as a still-honest limitation |
| 3 | §1.x framing edits (5 targeted edits) | §2.1, §2.2, §11.3, §16.2 | Leads with ephemeral federated query engine (not platform); adds central deployment mode alongside stdio; names Prism Satellite and multi-surface UI as §1 in-scope items; updates memory budget note |

**Items needing explicit human confirmation before dispatch:**

None of the above require human clarification — all three are derived directly from
human-approved content (§11.3 drafted replacement, §15.9 three-ways direction, §2.1
corrected framing, DC-004/DC-005/DC-006). The single required gate is the §5.1
brief-reframe sign-off, which the human grants separately.

~~The only open question the PO flags for the human: Edit 3 (multi-surface UI in §1.x
In Scope) scopes S2+S3+S4 at the brief level.~~

**RESOLVED 2026-06-26 (human): multi-surface UI (S2 console + S3 embedded AI + S4 extension
+ U1 admin) IS listed in the brief's §1.x In Scope** — the surfaces are committed v1-scope per
the multi-surface/multi-persona directive (§16.2 #1), NOT relegated to a day-2 roadmap section.
Edit 3 stands as written (add the multi-surface UI item to the §1.x In Scope list). The only
remaining gate is the §5.1 brief-reframe sign-off.
