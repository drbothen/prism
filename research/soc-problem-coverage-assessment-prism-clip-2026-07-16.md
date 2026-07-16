# SOC Problem-Coverage Assessment — Prism + CLIP vs. the SOC Team's Five Problems

- **Date:** 2026-07-16
- **Status:** informational research artifact (no pipeline gate)
- **Authored by:** orchestrator session synthesis, from three fresh-context research passes over prism (demo scope, day-2 vision, as-built inventory) and two over the CLIP multi-repo (`/Users/jmagady/Dev/multi-repo`: `.factory-project/` spec package, as-built 11-repo inventory)
- **Companion artifact:** `.factory/research/clip-shared-market-positioning-intel-2026-06-30.md`
- **Inputs (SOC team problem statement, 5 problems):** (1) Fragmented workflows & analyst inefficiency, (2) Lack of data visibility & context, (3) Manual & unscalable processes, (4) Disjointed client communication & reporting, (5) Immature threat management capabilities.

---

## Part 1 — Prism assessment

### Capability horizons

**Horizon 1 — Live demo (as of 2026-07-13, STATE v8.327):** Read-only federated query. One MCP endpoint in Claude Code; PrismQL across CrowdStrike EDR, Armis, Claroty OT, and Cyberint for 3 isolated demo clients (org-a/b/c); OCSF normalization in-flight; in-query ThreatIntel + NVD (CVSS) enrichment via the real WASM infusion path; AI self-onboards schema via `list_capabilities` / `prism_describe` / `prismql://reference`; pedagogical errors enable agent self-correction. No writes, no detections, no history. Demo thesis: "ten minutes into the incident, the full picture is clear." Remaining path to recording: two defect PR lanes + AUDIT-COVERAGE-001 (live audit 98/106 PASS, DEMO-READY: NO at time of assessment) → T13 capstone story → T14 recording.

**Horizon 2 — As-built beyond the demo:** SOC2/ISO27001 audit middleware on every MCP tool call; AI-opaque credentials (keyring + AES-256-GCM); 3-tier write gating + confirmation tokens (framework built, action tools stubbed); RocksDB with 19 column families incl. reserved schedules/rules/alerts/cases; spec-driven TOML connectors (new sensor = TOML, not code); hot reload; per-org overlays. **Wave 4 `prism-operations` (scheduler, detection rules, alerts, cases, action delivery — 8 stories, 45 BCs) is specified but NOT built**; ~32 MCP tools return structured NYA stubs.

**Horizon 3 — Day-2 vision (decided, `do_not_execute`, gated on post-demo brief-reframe sign-off):** Central multi-tenant service (Streamable HTTP, OIDC/SAML, RBAC); browser investigations console (S2) + embedded AI agent (S3); demand-driven caching (hot RocksDB → cold Iceberg, `RETAIN`, bitemporal `AS OF KNOWN`); satellite mesh for OT/air-gapped sites; detection-as-query engine (MATCH_RECOGNIZE sequences, Sigma import, staged rollout, FP auto-rollback); on-demand ML primitives (`ANOMALY_SCORE()`, `BASELINE_DEVIATION()`, `PEER_OUTLIER()`); collectors (syslog, NetFlow, pcap, Kafka); SIEM/lake connectors (Security Lake, Splunk, Sentinel); Prism Intel (CVE/KEV/EPSS feed-down); Prism Context (entity-360 knowledge graph); recommend-first SOAR/ARO with autonomy ladder; shared case management on bundled PostgreSQL; nested MSSP tenancy; compliance profiles (SOC2/ISO27001/IEC-62443/NERC-CIP).

The **Virtual SOC Analyst** (triage/planning/verdict agents, specialist sub-agents) is the layer *above* Prism in the diagrams — Prism is its grounded observation tool. That layer is diagram/vision-level, not spec'd stories.

### Problem-by-problem (Prism)

**1. Fragmented workflows — ✅ Strongly addressed (core thesis).** The demo proves it: one question, one endpoint, one query language instead of four consoles; cross-sensor correlation of the same endpoint across CrowdStrike + Armis + Claroty in a single query; TI + CVE pivots inside the query. Caveat: Prism eliminates context switching for *investigation and evidence gathering*; triage-as-workflow (alert queues, verdicts, timelines) depends on unbuilt Wave 4 plus the agent layer.

**2. Data visibility & context — 🟡 partially today, ✅ strongly in day-2.** Live cross-domain context (EDR + OT + asset inventory + TI, always-current) is the strength. Gaps today: historical network traffic, firewall logs, long-window "low and slow" — a purely ephemeral engine has no history. Day-2 targets each gap: firewall/switch sources; NetFlow/syslog/pcap collectors (Arkime-model packet retrieval); Iceberg cold tier via demand-driven caching; DI-029 correlation-window fix; bitemporal `AS OF KNOWN` (no surveyed commercial equivalent). The "low and slow" answer lives entirely in day-2 — none built.

**3. Manual/unscalable processes — 🟡 addressed in vision, mostly unbuilt, one real hole.** Per-client detection rule management: day-2 detection-as-query (global/per-client/ad-hoc scopes, semver lifecycle, shadow/canary rollout, Sigma import, detection-as-code CLI) — zero built. Repeatable product: genuinely addressed by built architecture (client onboarding = TOML specs + credential refs + overlays). Alert volume: day-2/agent-layer. **Monthly service reports: NOT addressed — brief explicitly scopes client reporting out of Prism** ("generated by the AI agent using Prism's tools, not by Prism itself"); nothing in the corpus productizes report generation.

**4. Client communication & reporting — ❌ largely not addressed (weakest fit).** No client portal or secure case-file sharing anywhere in the vision (day-2 S2 console and case management are analyst-facing). ServiceNow/Jira appear only as one-way alert-routing destinations (E-ALERT-ROUTING-001); no bidirectional sync, no ServiceNow-as-source, no CMDB. Day-2 SSO fixes credentials for analysts, not clients. Lone indirect win: OCSF normalization gives consistent terminology. (Resolution: see Part 3 — this is CLIP's charter.)

**5. Immature threat management — ✅ strongly addressed in day-2, ❌ nothing built.** Detection engine on paper is best-in-class (cross-source correlation joins, distinct-count spray, statistical baselining, NFA sequences, detection DAGs, OT-native Modbus/DNP3/S7/IEC-104); FP-spike auto-rollback; historical incident search via queryable internal tables (`prism_cases`/`prism_alerts`/`prism_audit`) + Iceberg + backtesting with coverage maps; threat-hunt recipe library (MITRE-tagged, backtested) + agent personas. "Playbooks" don't exist by name — investigation-consistency is really the (unspec'd) agent layer's promise. Everything here is Wave 4 or day-2.

### Prism summary matrix

| SOC Problem | Live demo | As-built | Day-2 vision | Verdict |
|---|---|---|---|---|
| 1. Fragmented workflows | ✅ core demo thesis | ✅ | ✅ (+agent layer) | Strong — Prism's raison d'être |
| 2. Data visibility & context | 🟡 live context; no history | 🟡 | ✅ caching, collectors, bitemporal | Strong, but "low and slow" is 100% day-2 |
| 3. Manual/unscalable | 🟡 repeatable TOML onboarding | 🟡 | ✅ except reporting | Good, minus service reports (out of scope) |
| 4. Client comms & reporting | ❌ | ❌ | ❌ (one-way ITSM alerts; OCSF terminology only) | Not addressed — by design (see CLIP) |
| 5. Immature threat mgmt | ❌ substrate only | ❌ Wave 4 unbuilt | ✅ detection-as-query, ML, hunting | Strong on paper, zero built |

---

## Part 2 — CLIP assessment

### Capability horizons

**Horizon 1 — As-built (11-repo brownfield):** Client incident portal over Jira. Clients (Cognito + TOTP): dashboard with ticket-volume charts + action items; four case types (Security incidents, Health cases, Service requests, Change requests) with search/filter, Lexical rich-text comments, status-history timelines, SSE live updates; file attachments; Reports page (download-only — admins upload finished PDFs; report types: Asset / Hunt / Monthly / Threat Intelligence / Vulnerability); Documents page; org switching. Staff (clip-admin, Okta): cross-org ticket/user/org/file management, report publishing, masquerade. Bidirectional Kafka bridge (kafka-connector-jirasm) syncs tickets/comments/statuses/MITRE TTPs with Jira Service Management — attachments do NOT sync. Dark capability: clip-signal email service is unauthenticated AND unwired (zero callers — clients receive no ticket-update emails); ~2.6k LOC dormant ECS security-event + STIX threat-intel models (designed-but-never-shipped MDR telemetry). Security criticals (P0): CB-001 tenancy IDOR (caller-supplied client-id not JWT-bound, org-scope gates untested), JWT-in-browser via SSR props, fail-open ephemeral RSA-1024 signing, per-request JWKS SPOF, 10-day non-revocable JWTs, unauthenticated sendEmail. Zero ServiceNow references in 14 repos.

**Horizon 2 — Target-state spec package (`.factory-project/target-state/`, Phase 1 spec crystallization in progress; Phase 0 PASSED 2026-06-29):** CLIP re-founded as a **regulatory-native, AI-native OT/ICS MSSP platform — the portal/evidence/compliance/workflow layer on top of Prism**, with a human-ratified boundary (2026-06-30): Prism owns See/Decide/Act-ENGINE (detection, query, SOAR execution); CLIP owns Assure/Operate + Act-SURFACE (approval UX, ARO cards, playbook authoring, evidence, notifications). Differentiator: "make the MSSP's work visible" — Decision-DNA, immutable command ledger, ARO action cards; "AI advises, humans retain final authority." Locked: 11 repos → ~8 macroservices; native helpdesk replaces Jira (strangler-fig; jirasm bridge RETIRED); Keycloak with BYO-IdP federation (Cognito dropped); net-new Public Integration Edge (PIE: public REST + client-facing MCP + outbound webhooks; D-NO-BESPOKE-CONNECTORS); evidence mesh (AEAD+Merkle+TPM, 7-yr WORM, BCSI JIT); Universal Compliance Framework (PDF/A + OSCAL exports); CAP-009 notification trees with SLA-breach escalation + per-recipient ack audit; cloud-only SaaS, multi-cloud portable via hexagonal ports; PostgreSQL+JSONB primary. 29 ADRs accepted; 5 seam ADRs (TS-001/003/004/006/007) HELD on the external Prism §5.1 brief-reframe sign-off. Capability catalog CAP-001..029; persona-storyboard workstream in Stage 6 hi-fi (frame-01 ARO approval complete). PRD/behavioral contracts and story decomposition deferred to PRD phase / Phase 2.

### Problem-by-problem (CLIP)

**1. Fragmented workflows — 🟡 the case/response half, not the investigation half.** As-built, CLIP is the client-facing case surface; analysts still investigate in vendor consoles. Target state: 7-state SOC case lifecycle (CAP-003), ARO action cards with approval UX (CAP-013 — first hi-fi storyboard frame), playbook authoring (CAP-022), collaborative authoring/presence (CAP-024/025), Decision-DNA (CAP-011), entitlement-gated embedded Prism-in-CLIP (held ADR-TS-001). Investigation context-switching is fenced to Prism by design. Complementary-partial: the pair solves it, neither alone.

**2. Data visibility & context — ❌ by design (delegated to Prism).** The "do-NOT-rebuild fence" forbids CLIP telemetry; dormant ECS/STIX models are the fossil of an earlier attempt correctly routed to Prism via the seam (CAP-016/PIS). CLIP's contribution: evidence-by-reference (GUID manifests, BCSI-filtered client views).

**3. Manual/unscalable processes — 🟡 half-addressed; reporting half flagged but unspecified.** Repeatable product: strong — 5-level tenant hierarchy with CLIP-driven Prism provisioning (CAP-001), zero-touch onboarding (CAP-020), self-service tenant/user mgmt via PIE (CAP-029), metered billing (CAP-023), MSP partner ecosystem (CAP-019), compliance presets. Detection rules: Prism's domain (CLIP provides the authoring surface). **Monthly service reports: as-built productizes the manual process, not the report** (human writes PDF → admin uploads → client downloads). Gap A-09 explicitly dispositioned: Conductor reference has a full monthly-report QA state machine (auto-generated exec summary + metrics; Draft→InQA→Approved→Sent; checklist-gated approval; DOCX/PDF export) — CLIP disposition "PRD-phase: new `MonthlyReport` entity." Recognized, zero spec today.

**4. Client communication & reporting — ✅ CLIP's charter; target state addresses it near line-by-line.**

| SOC complaint | As-built CLIP | Target-state spec |
|---|---|---|
| Case files shared via email = insecure | 🟡 Portal exists, but attachments don't sync to Jira; CB-001 IDOR + 10-day JWTs undermine "secure" | ✅ Zero-Trust File Reception Gateway (Epic 00: multi-AV/YARA/sandbox), evidence-by-reference, BCSI JIT "see-but-can't-take" (4h TTL), automatic BCSI sanitization, full audit trails (CAP-005/006/008) |
| Reporting is manual | ❌ upload/download of hand-made PDFs only | 🟡 Compliance exports specified (CAP-007: PDF/A, OSCAL, <5s SLO); monthly service-report generation = gap A-09, PRD-phase, unspecified |
| Separate credentials/portals | ❌ separate Cognito accounts | ✅ Keycloak broker, BYO-IdP federation — clients sign in with their own Okta/Azure AD/SAML (SD-08/ADR-TS-014) |
| Not integrated with client systems (ServiceNow) | ❌ zero ServiceNow in 14 repos; only 1898's own Jira SM | 🟡 Pattern locked: PIE public REST + outbound webhooks (HMAC, DLQ, at-least-once) + client-facing MCP; D-NO-BESPOKE-CONNECTORS (ServiceNow integrates as consumer of the public surface; 1898's first-party integration must dogfood it). No ServiceNow connector itself specified — socket, not plug |
| Inconsistent terminology | 🟡 CLIP exhibits it (UI "Cases & incidents" vs API "tickets") | ✅ ubiquitous-language shard; state-machine reconciliations; "ARO" replaces "alert" |
| Proactive client notification (implied) | ❌ effectively nonexistent (clip-signal unwired) | ✅ CAP-009 notification trees, multi-channel, SLA-breach escalation with per-recipient ack audit; residual gap A-17 (per-channel delivery lifecycle) PRD-phase |

Strongest problem/product fit in either portfolio. Two asterisks: A-09 (report generation) and an actual ServiceNow connector are recognized-but-unspecified; the six P0 security criticals must land first for "secure" to be true (spec sequences them first, D-SD01-LOCKED).

**5. Immature threat management — 🟡 CLIP supplies process maturity; detection depth stays with Prism.** CAP-022 playbook authoring (CLIP authors, Prism executes), CAP-011 Decision-DNA, ARO cards with deterministic safety gates, and the CIP-anchored case lifecycle deliver "consistent, high-quality investigation process" as governed workflow — filling exactly the playbook gap flagged in the Prism assessment. Historical incident search: CIM on PostgreSQL + KAG/RAG agent memory (CAP-018) + community defense (CAP-017, P2). As-built: none of this exists; the maturity story is 100% spec-package.

---

## Part 3 — Combined coverage and residual gaps

| SOC Problem | Prism (engine) | CLIP (portal/assurance) | Combined verdict |
|---|---|---|---|
| 1. Fragmented workflows | ✅ investigation: one query surface (demo-proven) | ✅ case/response: ARO approvals, case lifecycle, embedded Prism | Fully covered — split by boundary |
| 2. Data visibility & context | ✅ (historical = day-2) | ❌ by design | Covered by Prism alone; hinges on Prism day-2 caching/collectors |
| 3. Manual/unscalable | ✅ detection-as-code, TOML onboarding | ✅ zero-touch onboarding, self-service, billing; 🟡 report gen = A-09 | Mostly covered; monthly-report automation is the orphan |
| 4. Client comms & reporting | ❌ (correctly out of scope) | ✅ charter capability | Covered by CLIP — validates Prism's exclusion |
| 5. Immature threat mgmt | ✅ detection/ML/hunting (unbuilt) | ✅ playbooks, Decision-DNA, case QA (spec'd) | Fully covered on paper — nothing built either side |

**Cross-checks:** Prism's problem-4 hole is precisely CLIP's charter (encoded in CLIP's `prism-boundary.md`). Prism's "playbooks absent" gap in problem 5 is CLIP CAP-022 + CAP-011.

**Residual gaps owned by neither spec (raise at next gates):**
1. **Monthly service report generation (A-09)** — dispositioned to CLIP PRD phase, zero spec exists. The only SOC complaint with no owner-of-record in either package. Conductor reference gives a proven shape to harvest.
2. **Concrete ServiceNow connector** — PIE socket locked; nobody has spec'd the plug. SOC complaint names ServiceNow specifically.
3. **Per-channel notification delivery lifecycle (A-17)** — needed for "did the client actually receive it" assurance.
4. **External gate:** 5 CLIP seam ADRs HELD on Prism's §5.1 brief-reframe sign-off — the same post-demo human gate unlocking Prism day-2. Critical path for both portfolios' answers to problems 1, 2, and 5.
5. **CLIP's six P0 security criticals** (CB-001 IDOR foremost) — "secure client communication" is a false claim until they land; spec correctly sequences them first.

**Bottom line:** The SOC team's five problems are, in aggregate, well covered by the Prism + CLIP pair *as specified* — problem 4 flips from Prism's worst fit to CLIP's best. The exposure is execution horizon (Prism Wave 4 + day-2 unbuilt; CLIP still in Phase 1 spec) plus the orphaned specifics above — monthly report generation being the only complaint neither package currently owns in writing.
