---
document_type: research
produced_by: research-agent
captured_by: day-2-side-analysis
timestamp: "2026-06-25"
scope: "UI requirements for the prism security investigations console (S2) + admin console (U1): competitive teardown, table-stakes vs differentiators, SOC workflow/cognitive needs, embedded-AI/copilot UX, accessibility/streaming, web-stack decision, multi-tenant admin/RBAC"
sources: ["ui-investigations-console-ux-2026-06-25 (raw)", "ui-webstack-admin-rbac-2026-06-25 (raw)"]
do_not_execute: true
note: "Distilled synthesis of two 2026-06-25 deep-research passes (raw bodies committed alongside). Feeds ux-designer for S2/U1 design. Competitive set: Splunk ES, Microsoft Sentinel/Defender XDR, Google SecOps/Chronicle, Elastic Security, Exabeam, Hunters, Panther, Query.ai."
---

# UI Requirements — Prism Investigations Console (S2) + Admin Console (U1)

## 1. Table-stakes screens (non-negotiable; every serious console has them)

1. **Triage queue** — sortable/filterable incident/alert list: severity, risk, status, owner, summary, source-coverage; saved views; card/compact/table modes (Chronicle pattern).
2. **Case/incident detail page** (primary unit of work) — header (title, risk, severity, status, tags, owners, summary) + tabs: **Overview** (summary + MITRE map) · **Evidence** (events/alerts/artifacts) · **Entities** (profiles + risk) · **Timeline/Case Wall** (chronological event + analyst-action log) · **Actions** (playbooks/response history) · **AI Insights** (summaries + queries, clearly labeled).
3. **Entity 360 profile** — IP/user/host/hash/etc.; Sentinel 3-pane model (identity panel · event timeline · behavioral insights); aggregated across all federated sources via OCSF; cross-source coverage + risk.
4. **Investigation workspace** — Elastic-Timeline-style: collect events/alerts from multiple sources, build queries, event renderers for context, correlation queries, attach to a case.
5. **Search/query workspace** — explore raw events beyond alerts; structured PrismQL + NL-generated queries.
6. **Detection rule editor + library** — rule metadata, query logic, thresholds, MITRE mapping; convert investigative queries → rules; library search/filter by tactic/technique/FP-rate.
7. **Posture dashboards** — alert trends, incident counts, risk distributions, MITRE ATT&CK / compliance coverage.
8. **Reporting** — PDF export of investigations/dashboards.

## 2. Differentiators (where prism wins)

- **Federation surface** — source-coverage indicators, cross-source correlation views, unified entity profiles across systems WITHOUT centralization. (Prism's structural edge.)
- **AI-generated attack narratives** — present attack stories, not raw alert lists (Elastic Attack Discovery / Hunters / Exabeam Nova / Splunk AI Assistant pattern).
- **UEBA dynamic risk scoring** — focus triage on high-risk entities (Exabeam pattern) → ties to §15 on-demand ML.
- **Relationship graphs / entity canvas** — Chronicle/Sentinel-style; cross-source edges expose blind spots.
- **MITRE coverage heatmaps** with fine-grained filtering (Cortex XDR compliance-overview pattern).
- **UX cohesion** — unified triage→investigate→hunt→detect→respond with minimal context-switching (the single biggest UX differentiator).

## 3. Design principles (the load-bearing ones)

- **Case/entity-centric**, not alert-centric. Cases are the unit of work; entities are the pivot.
- **Preserve context across pivots** — one-click pivots (entity→events, alert→related), keep investigative state; this is what reduces MTTD/MTTR and alert fatigue.
- **Tier-1 vs tier-3** need different defaults: tier-1 = guided triage + summaries; tier-3 = raw query, hunting, detection engineering.
- **DESIGN FOR TRUST (critical for prism's AI-native + prompt-injection-hardened thesis):**
  - AI outputs are **suggestive, not authoritative**.
  - **Always show the query the AI ran** + link to the source data/results (explainability).
  - **Record AI actions in the case wall** (audit trail).
  - **Human approval required for impactful actions** (agentic automation staged, never silently executed).
  - **Embed AI in workflows, not a siloed console** (inline actions + chat sidecar, with clear machine-generated labels). Black-box AI erodes trust; verifiable/editable AI builds it.
- **Detection feedback loop** — "propose detection rule" from any investigative query/hunt; show which rules fired which incidents + performance.

## 4. Embedded-AI / Copilot UX (S3) patterns

- **Integration modes:** chat sidecar + **inline actions** (add AI note/query/filter to the workspace) + command palette; agentic autonomous only with evidence + human-in-the-loop gates.
- **NL→PrismQL** with the generated query always visible and editable (Security Copilot / Elastic AI Assistant pattern).
- **Summarize-this-alert / guided investigation** with MITRE mapping + next-steps, every claim evidence-linked.
- Maps directly to prism S1 (BYO agent) + S3 (server-hosted agent), AI-opaque credentials, prompt-injection-hardened output.

## 5. Accessibility & real-time/streaming (table-stakes for a dense SOC tool)

- **Virtualized tables** for 10k+ rows (TanStack Virtual / AG Grid).
- **Streaming/partial results** with **per-source coverage indicators** (directly realizes §3.6 partial-result semantics in the UI).
- **WCAG-aligned**; **color-blind-safe severity** (never color-only — pair with icons/text; matches axiathon SOUL.md §11).
- **Keyboard navigation / command-driven** interaction; dark + light themes.

## 6. Web-stack decision (research recommendation + the prism caveat)

**Research recommendation (2026):** **TypeScript SPA front-end (React, with SolidJS/Svelte as alternatives) + Rust backend (Axum/Tokio/DataFusion) + perf-critical client modules as WASM.** Rationale:
- The data-dense security UI ecosystem is JS-centric and mature: **AG Grid / TanStack Table+Virtual** (10k+ row grids), **ECharts/visx** (charts), **Cytoscape.js / sigma.js** (relationship graphs), **Monaco / CodeMirror** (PrismQL editor: highlight + autocomplete + lint).
- **Monaco for the PrismQL editor** is much easier/idiomatic in TS; in a Rust-WASM front-end it becomes an awkward JS island (loses the unified-Rust benefit).
- Ecosystem richness > Rust shared-types for data-dense UIs (practitioner consensus; Snowsight, Grafana, Panther all use JS front-ends).
- Shared types via **OpenAPI → openapi-typescript** codegen from the Rust backend.

**The prism caveat (the real judgment call — flag for human/architect):** the research says Rust-native (**Leptos**/Dioxus) is "feasible" precisely when *"the team is overwhelmingly Rust-based"* or it's *"more an internal tool than wide-market SaaS"* — and **prism is an all-Rust shop**. So the decision is genuinely: **(A) TS SPA** (research-pragmatic, best ecosystem, but introduces a second language + the type-boundary) vs **(B) Leptos/Rust-native** (one language, shared types, philosophically aligned, but build/wrap data-grid + graph + Monaco-island yourself). This is the web-stack ADR — NOT auto-decided.

## 7. Multi-tenant Admin/RBAC console (U1)

- **Per-tenant policy stores** (AWS Verified Permissions pattern) — tenant isolation at the cryptographic + policy layer (ties to §11.1 per-tenant DEK, §11.2 config).
- **Fine-grained RBAC** beyond admin/member: connector/dataset/dashboard-scoped roles + **custom roles** (Grafana JSON-role-definition pattern). Roles like Tenant-Admin / Security-Analyst / Detection-Engineer / Connector-Admin / Read-Only. (Realizes §11.5 G-12.)
- **Sectioned IA:** Investigations · Dashboards · Connectors · Admin · Settings.
- **Connector config + schema-mapping wizard** — DataFusion-schema-aware; realizes §13 dynamic-connector configure-schema.
- **Credential rotation UX** — write-only/masked secrets, never display after entry, "rotate" workflow + test-connection, integrate built-in store + Vault/AWS-SM (§11.1); audit every create/update/rotate (metadata only, never the secret).
- **Audit-log viewer** — user/action/target/timestamp/outcome; filters; optional AI clustering of related events.
- **Health/observability dashboards** — connector up/down, ingestion/query latency, DataFusion exec times, errors; tenant-scoped + cross-tenant for platform admins.
- **SSO wizard** — SAML/OIDC, per-tenant, validate + test login (realizes §11.3 SSO differentiator over Query).
- **Least-privilege UX + dangerous-action guards** — role wizards default to minimal perms; destructive actions (delete logs, disable alerts, delete tenant, rotate secret, change SSO) require confirmation that restates consequences + re-enter-name + optional MFA; all logged.

## 8. Decisions for prism (carry into design)

| # | Decision | Status |
|---|----------|--------|
| UI-D1 | Case/entity-centric IA; the 8 table-stakes screens are the S2 baseline | recommend-adopt |
| UI-D2 | Trust-first AI UX (show query, evidence-link, case-wall log, human approval) | recommend-adopt (aligns with prism thesis) |
| UI-D3 | Virtualized tables + streaming + per-source coverage + color-blind-safe severity | recommend-adopt |
| UI-D4 | Fine-grained per-tenant RBAC + dangerous-action guards + audit | recommend-adopt (realizes G-12) |
| UI-D5 | **Web-stack: TS SPA (React) vs Rust-native (Leptos)** | **OPEN — genuine judgment call (§6); web-stack ADR** |
| UI-D6 | Federation/source-coverage + AI-narrative + UEBA-risk + graph + MITRE-heatmap as the differentiator set | recommend-adopt |
