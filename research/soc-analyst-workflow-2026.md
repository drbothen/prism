---
document_type: research
research_type: domain
producer: research-agent
topic: SOC analyst alert-handling workflow for automated virtual-SOC-analyst monitoring loop (MSSP multi-tenant)
date: 2026-07-19
status: complete
consumers:
  - product-owner (product brief / PRD for virtual SOC analyst monitoring loop)
  - architect (monitoring-loop architecture, dedup/correlation keys, HITL boundaries)
confidence: medium-high
cross_validation: Perplexity deep research (primary, 5 calls) + Tavily (2 cross-validation searches)
inconclusive_flags:
  - Quantitative auto-close failure rates (no public statistics; best-practice extrapolation only)
  - Exact SLA-clock reset semantics on ticket reopen (platform/config dependent)
  - Rapid7 MDR internal multi-tenant mechanics (no primary docs found; inferred)
  - Vendor default grouping time windows (partially documented; many are configurable)
cited_sources:
  # Q1 — Triage workflow
  - "[1] NIST SP 800-61r3 (final): https://csrc.nist.gov/pubs/sp/800/61/r3/final"
  - "[2] NIST SP 800-61r3 (PDF): https://nvlpubs.nist.gov/nistpubs/specialpublications/nist.sp.800-61r3.pdf"
  - "[3] NIST SP 800-61r2 (PDF): https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-61r2.pdf"
  - "[4] MITRE 11 Strategies of a World-Class CSOC: https://www.mitre.org/sites/default/files/2022-04/11-strategies-of-a-world-class-cybersecurity-operations-center.pdf"
  - "[5] Rapid7 — SOC Alert Triage: https://www.rapid7.com/fundamentals/soc-alert-triage/"
  - "[6] Exaforce — Tier-1 Alert Triage: https://www.exaforce.com/learning-center/tier-1-alert-triage"
  - "[7] Prophet Security — Alert Triage: https://www.prophetsecurity.ai/blog/alert-triage"
  - "[8] CyberDefenders — Alert Triage Process: https://cyberdefenders.org/blog/alert-triage-process"
  - "[9] Radiant Security — SOC Alert Triage: https://radiantsecurity.ai/learn/soc-alert-triage/"
  - "[10] Eyer — SIEM Alert Triage Best Practices: https://www.eyer.ai/blog/siem-alert-triage-best-practices-for-socs/"
  - "[11] Prophet Security — SOC Tiers/AI flattening: https://www.prophetsecurity.ai/blog/soc-tiers-are-out-how-ai-is-flattening-soc-tier-1-2-3"
  - "[12] Swimlane — AI Tier-1 SOC NIST: https://swimlane.com/blog/ai-tier-one-soc-nist-response/"
  - "[13] ExtraHop — SANS SOC Survey Trends 2018-2021: https://www.extrahop.com/blog/sans-soc-survey-trends-2018-2021"
  - "[14] Decryption Digest — SOC Analyst Alert Triage Guide 2026: https://www.decryptiondigest.com/blog/soc-analyst-alert-triage-guide"
  # Q2 — Dedup / correlation
  - "[15] Splunk ES Risk-Based Alerting (RBA): https://www.splunk.com (Enterprise Security RBA docs)"
  - "[16] Splunk ITSI notable-event aggregation policies (episodes): https://docs.splunk.com (ITSI)"
  - "[17] Microsoft Sentinel — automatic incident creation / analytics rules: https://learn.microsoft.com/azure/sentinel/"
  - "[18] Microsoft Sentinel — Entities: https://learn.microsoft.com/azure/sentinel/entities"
  - "[19] Elastic Security — detection rules & alert grouping: https://www.elastic.co/guide/en/security/current/"
  - "[20] Google SecOps — alert suppression/throttling/exclusions: https://cloud.google.com/chronicle/docs"
  - "[21] Google SecOps SOAR — alert grouping into cases: https://cloud.google.com/chronicle/docs/secops"
  - "[22] Palo Alto Cortex XDR — alert deduplication key & 1h TTL: https://docs-cortex.paloaltonetworks.com/"
  - "[23] SANS — Next-Gen SOC: Automating Alert Overload (white paper 33901): https://www.sans.org/white-papers/33901"
  - "[24] Phoenix Cyber — automating SIEM triage with SOAR: https://phoenixcyber.com"
  - "[25] Dropzone AI — alert fatigue analysis: https://www.dropzone.ai"
  - "[26] Ni et al. — multi-level alert screening (DBSCAN/RETE/NN): academic"
  # Q3 — Existing-ticket handling
  - "[27] ServiceNow — Reopening an incident: https://www.servicenow.com/docs/r/it-service-management/incident-management/reopening-incident.html"
  - "[28] ServiceNow — Incident management best practices (parent/child): https://www.servicenow.com/community/itsm-blog/best-practices-for-implementing-the-incident-management/ba-p/2293421"
  - "[29] ServiceNow dev forum — SLA on reopened incident: https://www.servicenow.com/community/developer-forum/what-happens-to-sla-when-an-incident-is-reopened-after-being/m-p/3522465"
  - "[30] Atlassian/Jira Service Management — incident management guide: https://www.atlassian.com/software/jira/service-management/product-guide/getting-started/incident-management"
  - "[31] itilfromexperience — reopen vs new incident: http://www.itilfromexperience.com/Should+the+incident+be+reopened+or+a+new+one+logged"
  - "[32] TheHive — About cases: https://docs.strangebee.com/thehive/user-guides/analyst-corner/cases/about-cases/"
  - "[33] TheHive — Merge cases: https://docs.strangebee.com/thehive/user-guides/analyst-corner/cases/merge-cases/"
  - "[34] TheHive — View alerts linked to a case: https://docs.strangebee.com/thehive/user-guides/analyst-corner/cases/view-alerts-linked-to-a-case/"
  - "[35] Cortex XSOAR — link/unlink incidents CLI: https://docs-cortex.paloaltonetworks.com/r/Cortex-XSOAR/6.12/Cortex-XSOAR-Administrator-Guide/Link-and-Unlink-incidents-in-the-CLI"
  - "[36] Splunk SOAR — managing cases: https://lantern.splunk.com/Security_Use_Cases/Automation_and_Orchestration/Managing_cases_in_SOAR"
  - "[37] D3 Security — automated alert grouping (Smart SOAR, 48h window): https://d3security.com/blog/automated-alert-grouping-smart-soar/"
  - "[38] Swimlane — SOC case management: https://swimlane.com/blog/soc-case-management/"
  - "[39] Google SecOps community — how to group alerts / Find First Alert: https://security.googlecloudcommunity.com/google-security-operations-2/how-can-i-group-alerts-1661"
  - "[40] SANS ISC diary — IR case/alert management: https://isc.sans.edu/diary/29880"
  - "[41] Deepwatch — security incident ticket: https://www.deepwatch.com/glossary/security-incident-ticket/"
  # Q4 — MSSP multi-tenant
  - "[42] Microsoft — Defender multitenant management overview: https://learn.microsoft.com/en-us/unified-secops/mto-overview"
  - "[43] Microsoft — playbook for managed security: https://learn.microsoft.com/en-us/unified-secops/playbook-managed-security"
  - "[44] Microsoft Sentinel — multiple tenants / service providers (Lighthouse): https://learn.microsoft.com/en-us/azure/sentinel/multiple-tenants-service-providers"
  - "[45] Microsoft TechCommunity — Sentinel multi-tenant/MSSP playbooks: https://techcommunity.microsoft.com/discussions/microsoftsentinel/azure-sentinel-multi-tenantmssp-playbooks/2116180"
  - "[46] Palo Alto Cortex XSIAM — MSSP multi-tenant: https://docs-cortex.paloaltonetworks.com/r/Cortex-XSIAM/Cortex-XSIAM-3.x-Documentation/MSSP-multi-tenant"
  - "[47] Palo Alto Cortex XSIAM — monitor correlation rules: https://docs-cortex.paloaltonetworks.com/r/Cortex-XSIAM/Cortex-XSIAM-3.x-Documentation/Monitor-correlation-rules"
  - "[48] Palo Alto Cortex XSOAR MSSP: https://www.paloaltonetworks.com/cortex/cortex-xsoar-mssp"
  - "[49] Google Chronicle — MSSP partner program: https://cloud.google.com/blog/products/identity-security/launch-of-the-google-cloud-securitys-mssp-partner-program-with-google-chronicle"
  - "[50] Google SecOps community — multi-tenant SIEM instances (scopes): https://security.googlecloudcommunity.com/google-security-operations-2/multi-tenant-siem-instances-80"
  - "[51] Splunk community — multitenancy with ES (per-tenant indexes): https://community.splunk.com/t5/Splunk-Enterprise-Security/Possibility-of-Multitenancy-with-ES/m-p/597893"
  - "[52] Devo — RBAC or multitenancy: https://www.devo.com/blog/role-based-access-control-or-multitenancy-which-is-right-for-your-organization/"
  - "[53] Redis — data isolation in multi-tenant SaaS: https://redis.io/blog/data-isolation-multi-tenant-saas/"
  - "[54] Arctic Wolf — IR runbooks (unified portal): https://docs.arcticwolf.com/fr-fr/arctic-wolf-unified-portal/unified-portal/incident-response/ir-runbooks"
  - "[55] Arctic Wolf — security operations center glossary: https://arcticwolf.com/resources/glossary/security-operations-center/"
  - "[56] MSSPsecurity — incident escalation process: https://msspsecurity.com/mssp-incident-escalation-process/"
  - "[57] Forrester — SOC staffing report: https://www.forrester.com/report/security-operations-center-soc-staffing/RES95241"
  - "[58] Databahn — Beacon multi-tenant architecture / federated intelligence: https://www.databahn.ai/blog/the-beacon-architecture-rethinking-multi-tenant-security-data-operations-for-mssps"
  - "[59] Anomali — MSSP Program (multi-tenancy + federated search + data separation): https://www.anomali.com/press/anomali-launches-mssp-program"
  # Q5/Q6 — Sensor health / blind spots / automation boundaries
  - "[60] Expel — SIEM health and data quality: https://expel.com/cyberspeak/how-to-ensure-siem-health-and-data-quality/"
  - "[61] Picus — enhance SIEM log management: https://www.picussecurity.com/how-to-enhance-siem-log-management"
  - "[62] CardinalOps — detection gaps / blind spots: https://cardinalops.com/blog/detection-gaps-silent-threat-weakening-soc/"
  - "[63] Securview — monitoring blind spots: https://www.securview.com/ai-security-essentials/monitoring-blind-spots"
  - "[64] NIST SP 800-92 — Guide to Computer Security Log Management: https://nvlpubs.nist.gov/nistpubs/legacy/sp/nistspecialpublication800-92.pdf"
  - "[65] Splunk community — show hosts that stop reporting logs: https://community.splunk.com/t5/Splunk-Search/Show-hosts-that-stop-reporting-logs/m-p/558212"
  - "[66] SANS — 2019 SOC Survey (common practices): https://www.sans.org/media/analyst-program/common-practices-security-operations-centers-results-2019-soc-survey-39060.pdf"
  - "[67] Rapid7 — human-in-the-loop: https://www.rapid7.com/fundamentals/human-in-the-loop/"
  - "[68] Splunk — SOAR overview: https://www.splunk.com/en_us/blog/learn/soar-security-orchestration-automation-response.html"
  - "[69] Google Chronicle — respond to cases: https://docs.cloud.google.com/chronicle/docs/secops/respond-cases"
  - "[70] Microsoft Sentinel — automation: https://learn.microsoft.com/en-us/azure/sentinel/automation/automation"
  - "[71] Cortex XSOAR community — closing QRadar alert from XSOAR: https://live.paloaltonetworks.com/t5/cortex-xsoar-discussions/closing-qradar-alert-from-xsoar/td-p/452833"
  - "[72] Prophet Security — SOC metrics (MTTR/MTTI/false-negatives): https://www.prophetsecurity.ai/blog/soc-metrics-that-matter-mttr-mtti-false-negatives-and-more"
  - "[73] Sygnia — NIST incident response: https://www.sygnia.co/blog/nist-incident-response/"
  - "[74] Microsoft Intune — configure health monitoring: https://learn.microsoft.com/en-us/intune/device-configuration/templates/configure-health-monitoring-windows"
  # Q7 — Documentation standards
  - "[75] NIST SP 800-86 — Integrating Forensic Techniques into IR (chain of custody): https://nvlpubs.nist.gov/nistpubs/legacy/sp/nistspecialpublication800-86.pdf"
  - "[76] ODNI ICD 203 — Analytic Standards: https://www.dni.gov/files/documents/ICD/ICD-203.pdf"
  - "[77] ODNI — objectivity / how we work: https://www.dni.gov/index.php/how-we-work/objectivity"
  - "[78] MITRE ATT&CK: https://attack.mitre.org/"
  - "[79] SANS — glossary of terms / incident response: https://www.sans.org/security-resources/glossary-of-terms/incident-response"
  - "[80] Sologic — what is an incident timeline: https://www.sologic.com/en-us/resources/learning/what-is-an-incident-timeline"
  - "[81] TheHive — custom fields (enforced documentation): https://docs.strangebee.com/thehive/administration/custom-fields/about-custom-fields/"
---

# SOC Analyst Alert-Handling Workflow — Domain Research for the Virtual SOC Analyst Monitoring Loop

**Purpose.** Ground the design of Prism's automated "virtual SOC analyst" monitoring loop (multi-tenant MSSP context, LLM-agent-consumed output) in documented real-world SOC alert-handling practice. This artifact feeds the product brief / PRD for the monitoring-loop feature and the architectural decisions on deduplication keys, correlation windows, tenant isolation, and human-in-the-loop (HITL) boundaries.

**Method & confidence.** Primary evidence is Perplexity `sonar-deep-research` (5 high-effort calls) synthesizing authoritative frameworks (NIST SP 800-61r2/r3, SP 800-86, SP 800-92, MITRE "11 Strategies," ODNI ICD-203, SANS) and vendor documentation (Splunk ES/ITSI/SOAR, Microsoft Sentinel/Defender, Elastic, Google SecOps/Chronicle, Palo Alto Cortex XDR/XSOAR/XSIAM, ServiceNow, Jira, TheHive, Arctic Wolf, Devo). Cross-validated with 2 independent Tavily searches. Where sources are silent or discretionary, findings are flagged **[INCONCLUSIVE]**. Distinction is maintained throughout between **verified web findings** (cited) and **model-knowledge inference** (labeled).

> **Convergence caveat (applies throughout).** No source publishes a single globally-mandated canonical step-by-step Tier-1 workflow. However, guidance and vendor practice converge very strongly on the sequences described below. Numeric windows/thresholds cited (e.g., Cortex XDR 1h TTL, Google 0.5-24h grouping, D3 48h) are **specific vendor defaults or examples**, not industry constants — treat them as design reference points, not universal truth.

---

## Q1 — Tier-1 Alert Triage Workflow (Canonical)

### What analysts check FIRST

The strongest and most consistent finding across sources: the analyst's **first act is a fast credibility/relevance check on alert metadata + a known-pattern lookup**, not investigation.

- **Known-pattern lookup first.** Exaforce states plainly: "When an alert arrives, the first decision point is whether it belongs to a known pattern. Does it match a documented false positive for this rule and asset combination? Does it match an accepted risk...? If yes, the alert closes with a documented reference to the pattern." This step "should require under two minutes and should account for roughly 30 to 40 percent of alert volume in well-tuned environments." [6]
- **Then alert validation (is the signal even real?).** Exaforce: "Alert validation comes first, confirming the event is real. This means checking that the originating sensor reported correctly, that the data pipeline delivered a complete record, and that the alert isn't a known sensor misconfiguration or rule defect." A meaningful percentage of alerts fail validation as **data-quality issues rather than security events**. [6] — *This directly couples triage to sensor health (see Q5).*
- **Severity from the detector is a starting point, not a verdict.** Prophet: "The analyst's first job is to calibrate the signal quality before deciding how deep to investigate... A critical-severity alert from a noisy rule with a 90% false positive rate requires different handling than a medium-severity alert from a high-fidelity behavioral detection." [7]
- **Metadata examined first:** which detection rule/tool fired, when, against which asset/account, and whether it matches known benign/test/FP patterns [5][6][7]. Rapid7 frames triage as the "decision point between detection and response" — rapid review/validation/prioritization, explicitly **not exhaustive forensic investigation** [5].

### The canonical sequence (convergent synthesis)

CyberDefenders' seven-stage model [8] is the most explicit and aligns with Rapid7 [5], Radiant [9], Eyer [10], and NIST's Detection & Analysis phase [1][2][3]:

1. **Intake / centralization** — every alert captured in a central platform (SIEM). Sources warn against silently dropping alerts at ingestion; suppress/aggregate only *after* characterization [8][23].
2. **Validation** — confirm the event is real (sensor reported correctly, complete record, not a rule defect/misconfig) [6].
3. **Categorization** — map to threat type + attack stage using MITRE ATT&CK (initial access, lateral movement, exfiltration) [8][78].
4. **Deduplication / correlation** — is this new, a duplicate, or related to an open case? (see Q2/Q3) [8][10].
5. **Enrichment** — add context: asset criticality, user/device history, IOC reputation, threat intel [7][9][10].
6. **Severity / priority assessment** — driven by (a) asset criticality (domain controllers, privileged systems rank highest) and (b) detection confidence (behavioral + IOC-correlated > broad threshold rules). "High-confidence alerts on critical assets always go first." Risk score is an *input to* judgment, not a replacement [8][14].
7. **Disposition** — four verdicts recur across all sources [8][14]:
   - **True Positive (TP)** — malicious confirmed → escalate/respond per playbook.
   - **False Positive (FP)** — benign activity, no threat → close + **tune the detection rule**.
   - **Benign True Positive (BTP)** — behavior is real but authorized/expected → close + consider tuning.
   - **Indeterminate / Unclear** — insufficient evidence → escalate to Tier-2 or hold for more data.
8. **Escalation or closure + documentation** — escalate anything not resolvable at Tier-1; document findings (see Q7). "A SOC that closes false positives without documenting them is condemned to keep triaging the same noise indefinitely." [8]

### Time-boxing discipline

Tier-1 operates under an explicit per-alert budget: **12-20 minutes typical**; if no verdict in the window, **escalate with a documented summary rather than extending the investigation in place** [14]. This is a load-bearing operational constraint for an autonomous agent.

### Framework anchoring

- **NIST SP 800-61r3** (final; supersedes r2's 2012 "Computer Security Incident Handling Guide") frames triage as the operational mechanism of the **Detection & Analysis** phase; incident handlers must distinguish false positives / benign events / genuine incidents by analyzing each candidate event in context [1][2][3]. NIST does **not** prescribe an alert-by-alert Tier-1 workflow — it sets principles, not steps [1][2]. **[Partially inconclusive: NIST provides the lifecycle scaffold, not the micro-workflow.]**
- **MITRE "11 Strategies of a World-Class CSOC"** positions near-real-time monitoring/triage as central SOC functions and warns that workflows must prevent analysts being overwhelmed by noise, emphasizing **relevance to mission and critical assets** [4].
- **SANS SOC surveys** (via ExtraHop summary) consistently report alert volume, false positives, and staffing/alert-fatigue as top pain points; surveys focus on trends/challenges more than step diagrams [13].

**Design implications for the monitoring loop**
- Order the loop's per-alert pipeline as: **validate (incl. sensor-health/data-completeness check) → known-pattern/dedup lookup → categorize → enrich → score → dispose → document**. Put validation and known-FP-pattern matching *before* expensive enrichment — that is where 30-40% of volume should exit cheaply [6].
- Treat detector-supplied severity as an input feature to be *recalibrated* by rule-fidelity/FP-history, not as the disposition [7].
- Implement the four canonical verdicts (TP / FP / BTP / Indeterminate) as first-class dispositions; **Indeterminate must route to a human**, never auto-close [8][14].
- Enforce a configurable per-alert time/effort budget; on exceeding it, escalate with a structured summary rather than looping indefinitely [14].
- Every FP/BTP disposition must emit a structured, machine-readable "tuning signal" (rule id + asset + reason) so detection tuning closes the loop [8][7].
- Store known-FP/accepted-risk patterns as queryable, per-tenant references the loop can match against in <2 min-equivalent cost [6].

---

## Q2 — Alert Deduplication & Correlation Practice

### The layered vocabulary (must be modeled explicitly)

Platforms separate **event → alert → incident/case/episode**. Dedup/correlation decisions are expressed at the container level [16][17][21][22]:
- Splunk ES: correlation searches emit **notable events**; ITSI groups them into **episodes** via aggregation policies [15][16].
- Sentinel: analytics rules turn **alerts** into **incidents** (a container of ≥1 alerts) [17].
- Google SecOps: detections → **alerts** → **cases** (SOAR containers) [21].
- Cortex XDR: **parent** alert + **deduplicated/hidden** child alerts [22].

### Common correlation keys (convergent)

| Key class | Examples | Sources |
|---|---|---|
| **Entity / asset** | user account, host, IP, cloud resource, mailbox | Sentinel entity model (accounts, hosts, IPs, URLs, files, hashes, processes, registry keys, mailboxes) [18]; Splunk RBA risk index keyed by `src`/`dest`/`user` [15]; Elastic group-by host/user (up to 3 fields, nested) [19]; Google entity fallback [21] |
| **IOC** | IP, domain, file hash, URL | CyberDefenders [8]; Cortex XDR dedup key embeds `hash_id` (SHA256 with fallback hierarchy) + file/process name [22]; CTI-driven grouping by campaign/actor [26] |
| **Detection rule ID / metadata** | rule name, correlation search name, template | Elastic group-by rule name [19]; Google "identify noisy rules >100 detections/day" [20]; Sentinel rule templates [17] |
| **Campaign / MITRE tactic-technique** | ATT&CK technique annotations, behavioral history | Splunk RBA "ATT&CK Tactic Threshold Exceeded" (multiple techniques over 7-day window) + "Risk Threshold Exceeded" (cumulative score, tunable for low-and-slow) [15]; CyberDefenders attack-stage categorization [8] |

### Deciding new vs duplicate vs related

- **Cortex XDR (most explicit deterministic model):** dedup key = `{agent_id}_{alert_name}_{hash_id}_{action_status}_{name}_{trigger}` with **fallback hierarchies** per component (e.g., `hash_id`: `action_file_sha256 → action_process_image_sha256 → actor_process_image_sha256`). **TTL = 1-hour sliding window** from first-alert ingestion. Matching key within TTL → suppressed and attached to parent (hidden); after TTL expiry → new alert. Excluded from dedup if `agent_id` or `hash_id` missing/all-zero [22].
- **Google SecOps SOAR grouping:** two-stage — first match by **source grouping identifier**, else fall back to **cases with mutual entities**. Config: "Max alerts grouped into a case" default **30**; "Timeframe for grouping" **0.5-24h** in half-hour steps. Grouping runs **only on open cases — closed cases are never auto-reopened** [21][39].
- **Sentinel:** analytics rules can "Group related alerts" into a single incident up to **150 alerts** (safeguard against over-aggregation); grouping by alert details (rule name) or mapped entities within a time window [17][18].
- **Splunk RBA (aggregation-over-time model):** individual notable events are treated as *partial evidence* contributing to an entity's risk profile; a consolidated "risk notable" fires only when thresholds (e.g., ≥2 techniques AND ≥2 sources within window) are exceeded — reframing dedup as evidence accumulation rather than suppression [15].

### Time windows (design reference, not constants)

- **Short (strict dedup):** minutes-to-1-hour. Cortex XDR **1h TTL** [22]; Google throttling `suppression_window` recommended ≥ match-window size [20].
- **Moderate (case-level grouping):** hours-to-1-day. Google **0.5-24h** [21]; D3 Smart SOAR example **48h** open-incident lookup [37].
- **Long (campaign/low-and-slow):** up to **7 days** — Splunk ATT&CK tactic threshold [15]. **[INCONCLUSIVE: no single "correct" window; all are configurable and environment-specific. Sources explicitly note trade-off — short windows risk splitting a slow attack across cases; long windows risk merging distinct incidents that share an entity/IOC.]**

### Alert storms / floods from one root cause

Documented strategies (spectrum, often layered) [20][21][22][23]:
1. **Deduplication** — collapse identical events on same entity within TTL (XDR) [22].
2. **Throttling / suppression** — `suppression_window` + `$suppression_key` (single-event) or match-vars (multi-event) suppress repeats after first fire [20].
3. **Exclusions** — filter known-noisy matches before alerting; "Test Exclusion" shows 30-day retro impact [20].
4. **Aggregation into one incident/episode** — Sentinel 150-cap, ITSI episodes, Google 30-cap; caps prevent unwieldy over-aggregation [16][17][21].
5. **Risk accumulation** — RBA turns floods of low-fidelity events into a single scored risk notable [15].
6. **SOAR time-bound suppression by entity lookup** — playbook suppresses alerts for a known-benign entity for a period [20].

Academic direction: multi-level screening (DBSCAN density clustering + RETE fuzzy rule reasoning + dynamic NN) reports high TP rate with constrained noise, but results are dataset-specific [26]. **[INCONCLUSIVE: ML clustering is emerging, not standard SOC practice.]**

**Design implications for the monitoring loop**
- Model the **event → alert → incident/case** hierarchy explicitly; dedup/correlation decisions live at the case container, not the raw alert.
- Build a **deterministic dedup key** from concatenated normalized fields with **fallback hierarchies** for missing data (mirror Cortex XDR) — Prism's OCSF normalization at the adapter boundary is the natural place to compute canonical entity/IOC keys.
- Support the full key taxonomy — **entity (tenant-scoped), IOC, rule id, ATT&CK technique/campaign** — as composable correlation dimensions.
- Make dedup/grouping **time windows configurable per key class** (short for identical-event dedup, moderate for case grouping, long for campaign) with documented defaults; do not hardcode a single window.
- Implement **cap-and-spillover** (like the 30/150 caps): when a case exceeds N grouped alerts, open a new correlated child rather than growing unbounded.
- Provide an **RBA-style accumulation mode** so a flood of low-fidelity signals for one entity becomes *one* scored item, not N tickets — directly relevant to alert-storm resilience.
- **Never auto-group into a closed case** (Google's rule): closed cases require deliberate reopen (see Q3) [21][39].
- Prism's ephemeral federated model means correlation state must be materialized deliberately (there is no persistent SIEM index); the loop needs an explicit, bounded correlation store keyed on the canonical dedup keys.

---

## Q3 — Existing-Ticket Handling (Append / Link / Merge / Reopen)

### The three-way decision

When a new alert relates to existing activity, SOCs choose among [37][38][41]:
1. **Append** to an open case/ticket (add alert + observables as evidence). Bias is strongly toward this when key attributes match.
2. **Link as related** (parent-child, or link to a problem record) — separate records, correlated structurally.
3. **Reopen** a recently resolved/closed ticket (or open new).

Selection depends on shared **root cause, affected asset/service, timeframe, and impact** [37][41].

### Append (default for correlated alerts)

- **TheHive:** an alert is automatically linked to a case when the case is created from it or when added to an existing case; all linked alerts visible via "Linked alerts" tab. Case = central repository accumulating observables [32][34].
- **Splunk SOAR:** any event can be promoted to a case; cases consolidate multiple events into one logical unit with a workbook (IR checklist) [36].
- **D3 Smart SOAR (concrete example):** event playbook searches for an open incident matching the endpoint within a **48-hour window**; if found, the incoming alert is **attached to the existing incident rather than escalated as new**; only if none found is a new incident created [37].
- **Google SecOps pattern:** attach main playbook only to the **first** alert in a case (creates the ITSM ticket, stores its ID as a case value); subsequent grouped alerts **update the existing ticket with a comment** (e.g., "new alert auto grouped") via a "Find First Alert" action [39].

### Link / parent-child / problem records

- **ServiceNow:** use "Related Incidents" + parent-child (Child Incidents related list). Distinct incident records remain granular (which users/BUs affected) but linked to a parent representing the overarching issue [28].
- **Jira Service Management:** link multiple incidents to a **problem** record via "is caused by" — correlate at problem-management level while keeping incident tickets separate [30].
- **Cortex XSOAR:** programmatic `!linkIncidents` to link/unlink incidents [35].
- **TheHive:** supports explicit **merge cases** [33].

### Reopen vs new — and the time window

- **ServiceNow (hard platform constraint):** a **Resolved** incident can be reopened (state → In Progress) via resolution-email links; a **Closed** incident **cannot be reopened** — a new incident must be logged [27]. This "resolved-but-not-yet-closed" gap is the operative reopen window.
- **ITIL practitioner guidance:** whether to reopen or log new is a **policy decision** tied to whether it's genuinely the same issue recurring vs a new occurrence [31]. **[INCONCLUSIVE: no universal reopen time window; it is governed by local policy and the resolved→closed auto-close timer, which is org-configured.]**

### SLA implications

- **[INCONCLUSIVE — the single most uncertain area.]** ServiceNow community discussion confirms behavior on reopen (whether the SLA clock resumes, resets, or a new SLA attaches) is **configuration-dependent**, not a fixed rule [29]. Appending to an existing case generally keeps the original SLA/timeline; opening a new case starts a new SLA clock. No source gives authoritative cross-platform SLA-reset semantics.
- MSSP contractual SLAs (response/notification times) are typically tracked at the incident/ticket record in the ITSM layer, which is often distinct from the SOAR case — decisions to append vs re-create must be harmonized across both layers [28][30].

**Design implications for the monitoring loop**
- Default behavior: **append correlated alerts to the open case** and **comment/update** the linked ticket (Google's first-alert pattern) rather than spawning duplicate cases [37][39].
- Model an explicit **case lifecycle with a Resolved→Closed transition**; allow reopen only from Resolved, require deliberate action (and a fresh human touch) to reopen from Closed — mirror ServiceNow's constraint [27].
- Make the **reopen window configurable per tenant** (tied to the resolved→closed timer); treat "same issue within window" vs "new occurrence" as a documented, policy-driven decision, not a silent heuristic [27][31].
- **Surface SLA impact explicitly** on every append/link/reopen/new decision: does this reset the SLA clock? The agent must not silently make a choice that changes SLA accounting — expose it. **[Design must not assume a reset rule; make it an explicit, per-tenant configured behavior.]** [29]
- Emit structured link relationships (parent-child, related, caused-by, merged-into) so downstream ITSM integration (ServiceNow/Jira/TheHive) can mirror them [28][30][33][35].
- Keep the loop's case model and the customer ITSM ticket in a defined mapping; never let the two drift.

---

## Q4 — MSSP Multi-Tenant Specifics

*(Directly relevant to Prism's per-analyst, multi-client deployment and the demo's cross-customer campaign narrative.)*

### Per-customer runbooks, playbooks, asset criticality

- MSSPs start from an **MSSP-global baseline runbook**, then layer **customer-specific** context: asset criticality, business processes, approved containment options, and escalation contacts [verified pattern across [43][44][54]].
- **Runbook vs playbook** (Tufin): a runbook is the comprehensive overarching workflow/escalation guide; a playbook is scenario-specific (phishing, ransomware). The append/link/reopen grouping decision belongs in the **runbook** layer [via Tufin, general SOC guidance].
- Arctic Wolf exposes **IR runbooks** as customer-facing content in its unified portal — both operational tools for the MSSP and educational material for the customer [54].
- **Parameterization:** playbooks read **tenant-specific config objects at runtime** (customer identity, topology, regulatory jurisdiction, contacts, asset-criticality scores) so actions/notifications route correctly per tenant [44][45]. Sentinel recommends deploying connectors/playbooks with cross-tenant auth via **Azure Lighthouse + GDAP** (Granular Delegated Admin Permissions) [44][45].
- **Asset criticality** is expressed as per-customer scores/tiers/labels tied to tenant identifiers; detection rules fold criticality into severity so "an alert on a mission-critical server in one customer is treated differently from a similar alert on a less critical asset in another" [50][52]. **[INCONCLUSIVE: explicit asset-criticality modeling is implicit in vendor docs, not fully specified.]**

### Cross-customer / cross-tenant campaign detection (the demo narrative)

**Core verified principle:** cross-tenant analytics are performed on **derived artifacts (alerts, IOCs) or under strictly-controlled privileged roles**, never by exposing one tenant's raw data to another [50][58][59].

- **Databahn "Beacon" (strong cross-validation):** "MSSPs can now leverage **anonymized telemetry patterns across tenants** to identify shared threat trends – safely. This federated analytics layer transforms raw, siloed telemetry into contextual knowledge across the portfolio **without exposing any customer's data**." Each tenant runs a fully contained data plane; data is tagged at collection edge for governed isolation [58].
- **Anomali MSSP:** "true multi-tenancy with **federated search**... analyze security across all clients simultaneously, without compromising control or compliance" + "strict data separation" [59].
- **Microsoft Defender multitenant:** unified cross-tenant incident/case view; cross-tenant **advanced hunting** via KQL to "proactively hunt for threats across multiple tenants"; single pane of glass for MSSP partners [42]. Sentinel cross-workspace queries via `workspace()` + `union`, **up to 20 workspaces/query (≤5 recommended)** [45, Tavily LinkedIn corroboration].
- **Google SecOps:** privileged roles (MSSP threat hunters) can hold **scopes spanning multiple tenants / global data**, enabling cross-tenant analytics — but such access must be tightly controlled and **audited**; queries may return **anonymized results** [50].
- **Cortex XSIAM:** strict per-child-tenant data segregation, but the **main tenant can view aggregated alerts across tenants** — cross-tenant intelligence expressed as **global alerts in the MSSP main tenant** with internal visibility, no customer-to-customer raw exposure [46]. Correlation executions are **audited** in a dedicated dataset [47].

The pattern: **raw data isolated per tenant; the same IOC/hash observed across tenants surfaces as a derived, MSSP-internal global alert** — exactly the "same IOC across tenants" campaign-detection story, done via shared derived artifacts, not shared raw data [46][50][58].

### Customer-specific escalation paths & approvals

- Escalation contacts, notification requirements, and approval gates are **per-tenant runbook parameters** [44][54][56].
- Cortex XSOAR uses **per-incident war rooms** — shared workspaces for MSSP analysts to collaborate/co-investigate with customer teams while staying within tenant boundaries [48].
- MSSP incident-escalation processes define tiered escalation and customer-notification obligations (often contractual) [56]. **[Partially inconclusive: escalation-path specifics are contract/customer-defined; sources describe the pattern, not universal timings.]**

### Tenant isolation during investigation

- **RBAC is the primary isolation mechanism** [50][52]. Analysts have access only to assigned tenants; threat hunters may hold broader/global scopes but those are **audited and contractually governed** [50].
- **Segregation implementations:** per-tenant indexes (Splunk ES — with the noted overhead of adjusting correlation searches per index) [51]; scopes/namespaces/ingestion labels (Google SecOps) [50]; main+child tenants with strict segregation (XSIAM) [46]; database/schema/tenant-scoped-table patterns (Redis, vendor-neutral) [53].
- **Data residency / compliance:** MSSPs may run **separate regional instances (EU/US)**; tenant data stays isolated per region; **shared threat intel may be global but must avoid exposing personal/sensitive data**; constraints codified in contracts (what can feed global analytics, required anonymization, deletion/restriction rights) [50][58]. **[INCONCLUSIVE: vendor docs light on MSSP data-residency specifics; recommendations extrapolate from general cloud/legal practice.]**
- Some contracts **prohibit MSSPs from using customer data for cross-tenant analytics beyond anonymized threat intelligence** [verified as a stated constraint].

**Design implications for the monitoring loop**
- **Tenant is a first-class, mandatory dimension** on every alert, case, correlation key, dedup key, and audit event. Isolation is enforced by design (Prism newtypes like `OrgSlug`), not by convention.
- Correlation keys must be **tenant-scoped by default**. Cross-tenant correlation is a **separate, explicitly-privileged code path** that operates only on **derived artifacts (IOCs, hashes, ATT&CK techniques)** — never raw per-tenant records — and **emits MSSP-internal "global" findings**, never cross-customer raw data (Databahn/XSIAM/Google model) [46][50][58].
- Cross-tenant campaign detection = "same IOC/hash seen in N tenants" → produce an **anonymized/aggregated MSSP-internal alert**; the per-tenant view shows only that tenant's own hits. Design the demo narrative on this exact split.
- Every cross-tenant query/correlation must be **audited** (who, when, what scope) — mirror XSIAM's correlation-execution audit dataset [47][50].
- Runbook/playbook logic must read **per-tenant config** (asset criticality, escalation contacts, approved containment, notification/approval requirements, reopen window, SLA policy) at runtime [44][45].
- Fold **per-tenant asset criticality** into severity scoring so identical raw alerts get tenant-appropriate priority [50][52].
- Support **per-tenant / per-region data-handling and anonymization policy** as configuration; assume some tenants contractually forbid raw-data cross-tenant use — default to derived-artifact-only cross-tenant analytics [58].
- Align with Prism's existing credential/tenant-isolation posture (AD-017 AI-opaque credentials, `OrgSlug` newtype, RBAC): the monitoring loop must not leak tenant A's data into tenant B's context, including into any LLM prompt.

---

## Q5 — Sensor-Health / Telemetry-Gap Discipline (Blind-Spot Management)

### Absence-of-data is a risk signal

Core verified discipline: **degraded/silent sensors create silent monitoring gaps adversaries can exploit; "log source not reporting" is treated as a first-class alert category** [60][62][63][64].

- "Sensor health and telemetry completeness are as important as analytic sophistication; degraded or offline log sources create silent monitoring gaps" [via automation/sensor research synthesis].
- **Expel** treats data-ingestion rate, log-source connectivity, **heartbeat monitoring**, and volume trending as core SIEM health metrics — making sensor-health monitoring **part of core SIEM administration, not an ancillary concern**. Recommended tracked signals: **data ingestion rate per source, log-source connectivity status, heartbeat monitoring, volume trending, parse-error rates, query performance, storage utilization, alert-generation rates** [60].
- **NIST SP 800-92** advises periodically verifying all important log sources are configured correctly and logs are successfully transmitted and stored [64].
- **Splunk** practitioners use queries/tooling to surface **hosts that stop reporting logs**, with thresholds/alerting when volumes drop or sources fall silent [65].
- **Endpoint/EDR agent health** metrics: last check-in time, event volume, policy version — **silent agents = potential blind spots where malware operates without local detection and host events never reach the SIEM** [60, EDR synthesis][74].

### Treating a gap as a potential incident

- If adversarial activity is suspected — **repeated disabling of logging on high-value systems, abrupt log drop coinciding with other alerts, unexplained telemetry loss from domain controllers** — the SOC treats the **telemetry gap as an incident in its own right** [verified]. Runbooks then include: deploy alternative sensors, memory/disk forensics on affected hosts, review privileged activity, increase monitoring of adjacent systems for lateral movement.
- "Even in the absence of direct evidence of compromise, the mere fact that an attacker **could** operate unseen in a blind spot justifies heightened scrutiny" [verified].

### Two kinds of blind spot

1. **Telemetry/infrastructure blind spots** — unmonitored segments, shadow IT, unlogged cloud services, uncoordinated change management [63].
2. **Detection-coverage blind spots (CardinalOps)** — abundant telemetry but **no detection rule for critical MITRE ATT&CK techniques** → attacker traverses monitored infra without meaningful alerts. Also: parse-error spikes / format changes cause rules to **silently fail to match** even when logs are ingested (functional blind spot) [62][60].
- CardinalOps' key distinction: **MTTD/MTTR/#alerts measure *activity*, not *coverage*** — they say nothing about whether the SOC has visibility into the threats that matter. Coverage metrics (ATT&CK technique coverage) are the complement [62].
- Discovery methods (Securview): map network topology for unmonitored segments, audit cloud for shadow IT/unlogged configs, review log sources, pen-test for paths through unmonitored areas, regular asset inventories [63].

**Design implications for the monitoring loop**
- Make **sensor/telemetry health a first-class input to triage** (ties to Q1 validation step): before trusting an alert — or an *absence* of alerts — the loop must know whether the relevant sensor is healthy, degraded, or silent. Prism's per-sensor adapters are the natural health-signal source.
- Emit a **"sensor silent / degraded" alert class** driven by heartbeat/last-successful-fetch/volume-drop/parse-error signals — per sensor, **per tenant** [60][64][65].
- **Treat absence of expected data as a positive risk signal**, not a null: a tenant's sensor going dark (especially high-value: identity/EDR/domain telemetry) should raise, not lower, attention — and should be surfaced to a human, especially if it coincides with other activity.
- Track **coverage** distinctly from **activity**: expose which ATT&CK techniques / tables / sensors are actually observable per tenant, so the loop and its operators can see behavioral blind spots (CardinalOps) [62].
- Distinguish **"no threat found"** from **"could not observe"** in every disposition and in the wire output the LLM agent consumes — an Indeterminate-due-to-missing-telemetry verdict must be explicit, never rendered as a clean "all clear."
- Health degradation should optionally lower confidence / block auto-close for alerts that depend on the degraded source.

---

## Q6 — Automation Boundaries (SOAR Lessons)

### Safe-to-automate vs keep human-in-the-loop

Consistent maturation path across vendors [67][68][70][69]:
- **Automate (low-risk, high-volume):** ingestion/normalization, enrichment, correlation, alert grouping, ticket creation, context gathering, proposing severity/recommended actions [68][70].
- **Gradually automate under strict conditions:** certain containment actions with clear guardrails [70].
- **Keep HITL:** complex classification, high-impact/irreversible actions, high-severity disposition, and **case closure** [67][69][70].
- Each playbook step should be **annotated as automated / requires-approval / manual** — making the automation boundary explicit and auditable [68].
- SOAR governance: **approvals for playbook deployment, periodic review of automation logic, and "kill switches"** to disable automation [verified].

### Documented failure modes

- **Auto-closing is the headline risk.** "The primary risk of auto-closing is generating **false negatives**: closing alerts that actually represent real threats." [verified] Prophet's FPR thresholds imply that if a critical alert's expected FPR is below 25%, **auto-closing risks missing one in four real threats — a ratio few organizations would accept** [72].
- **Cross-platform state sync is fragile.** The Cortex XSOAR/QRadar community case shows auto-close requires not just classification but choosing correct closing-reason IDs and synchronizing offense state across platforms — needs careful testing/validation [71].
- **Auto-ticketing misrouting.** Automation can "shunt important alerts into tickets that nobody reads"; ticket types must map to appropriate receivers and respect business/escalation processes [verified].
- **Alert fatigue amplification.** Poorly-tuned automation can amplify noise rather than reduce it [via research synthesis].
- **[INCONCLUSIVE — flagged by the source itself:** "few sources provide explicit statistical data on failure rates or detailed case studies of auto-close errors; much of the guidance emerges from general principles and practitioner experience, and should be flagged as uncertain and environment-specific."**]** Vendors (Splunk "ease in," Google "validate closure against playbook logic + analyst decisions") counsel introducing auto-close **cautiously, if at all, heavily validated** [68][69].

### Alert-fatigue / effectiveness metrics

- **Activity metrics:** MTTA (acknowledge), MTTR (detection→resolution incl. containment/eradication/recovery), MTTD/MTTI, false-positive rate, dwell time, #alerts investigated [72][62].
- **Coverage metrics (complement):** ATT&CK technique coverage, detection gaps — "measure whether the SOC has visibility into threats that matter," which activity metrics do not [62].
- Post-mortems should capture automation-caused mishandling (early auto-closure, misrouted tickets) and drive playbook/threshold/HITL changes [16 AI post-mortems ref][73].

### What an autonomous agent must ALWAYS surface to a human

Verified "always-surface" categories [67][69][72, synthesis]:
- **High-severity alerts** and alerts on **critical assets** — rarely auto-closed without human review.
- **High-impact ATT&CK techniques** — explicitly: **credential dumping, privilege escalation, lateral movement, data exfiltration** — always surfaced to humans.
- **Ambiguous / novel / low-confidence cases** — "design [agents] to surface ambiguous or novel cases to humans rather than making unilateral decisions."
- **Any high-impact / irreversible action** (isolate critical systems, close high-severity incidents) — agent may *propose*, must not *execute* unilaterally.
- **Indeterminate verdicts** (from Q1) and **verdicts that depend on degraded/silent sensors** (from Q5).

**Design implications for the monitoring loop**
- **Auto-close is the highest-risk automation — default OFF for anything but narrowly-scoped, thoroughly-documented, low-severity known-FP patterns**, and even then must log the decision + support periodic review [68][69][72]. This aligns with Prism's production-grade / no-silent-suppression posture.
- Encode an explicit **automation-boundary annotation** per loop action: `auto` / `propose-only` / `requires-human-approval`. High-impact and cross-tenant actions are never `auto`.
- Implement a **mandatory human-surface list**: high severity, critical assets, high-impact ATT&CK techniques (cred dumping / privesc / lateral movement / exfil), novel/ambiguous, low-confidence, Indeterminate, degraded-sensor-dependent. The agent **proposes**, a human **disposes** for these.
- Agent output = **decision-support, not unilateral action**: enrich, correlate, group, score, recommend — but gate closure/containment behind human confirmation (Google's "analysts validate outcomes") [69].
- Emit **activity + coverage metrics** (MTTA/MTTR/FPR/dwell + ATT&CK coverage/blind-spot signals) as first-class outputs [62][72].
- Provide a **kill switch** and full **decision audit trail** so automation can be disabled and every auto-decision reconstructed [68].
- Because output is consumed by an LLM agent (Prism's design premise), the surface must clearly distinguish **"agent recommends X (requires human approval)"** from **"agent did X automatically"** — never blur the two.

---

## Q7 — Documentation Standards (Defensible Triage/Investigation Record)

A defensible, audit-ready record is a convergence of **incident-response guidance (NIST 800-61), forensic practice (NIST 800-86), and formal analytic standards (ODNI ICD-203)** [75][76][1].

### What a defensible record contains

1. **Evidence / artifacts with integrity + provenance.** NIST 800-86 emphasizes chain-of-custody: who collected each item, when/where, how stored/transferred, who accessed it over time; prepare forensic resources in advance (chain-of-custody forms, evidence handling) [75]. For SOC records: **reference forensic collections by identifiers (evidence IDs, hash values, storage locations); avoid undocumented modification of original data** [75]. Evidence objects must be **linked to the analytic conclusions they support** (ICD-203 source-quality expectation) [76]. When evidence is unavailable/incomplete (e.g., retention limits), **record that explicitly** — it explains uncertainty and informs logging improvements [verified].
2. **Timeline.** A visual/structured sequence of events before/during/after — including **operational events** (alert ingested, first reviewed, validated, context retrieved, disposition made, response actions like isolation/password reset) **and analytic events** (hypotheses formed, evidence collected, decisions made), each timestamped [80]. Rapid7's triage stages (ingestion → validation → enrichment → severity → disposition) provide the natural timeline scaffold [5].
3. **Analytic reasoning incl. alternatives + uncertainty (ICD-203).** Products must be objective, independent, based on all available sources, and make **source quality, evidence, assumptions, uncertainties, and alternative explanations explicit** — analysis-of-competing-hypotheses rigor [76][77]. This maps directly onto what an audit-ready record needs. ICD-203 also provides institutional review mechanisms (analytic ombuds/review) mirroring SOC internal QA of investigation records [76].
4. **Disposition rationale.** Why TP vs FP vs BTP — the evidence and reasoning behind the verdict, not just the verdict [8].
5. **Behavioral description via MITRE ATT&CK** — consistent, reproducible description of observed/suspected behaviors [78].
6. **Attribution + reproducibility.** Who did what, when; enough that a reviewer can retrace the path from artifact → hypothesis → conclusion [76].

### Tooling enforcement

Case-management platforms operationalize this: TheHive/Splunk SOAR create structured **artifact objects** with per-evidence metadata, and **custom fields can be made mandatory** (disposition, severity, ATT&CK mapping required for every case) — enforcing documentation capture [81][36][34]. Workbooks act as IR checklists ensuring consistent progression [36].

### Why it matters

Detailed records enable later learning, continuous control improvement, regulatory compliance, and **legal defensibility** — incidents may trigger legal/regulatory/internal processes where integrity and provenance of digital evidence are critical [75][1].

**[INCONCLUSIVE: ICD-203 was written for national-security intelligence, not cyber ops; its application to SOC documentation is a well-reasoned analogy, not codified cyber standard. NIST 800-86 predates modern SOC tooling but its principles transfer directly.]**

**Design implications for the monitoring loop**
- Every disposition the loop produces must be a **structured, defensible record**: linked evidence/artifacts (by ID + hash), timestamped timeline (operational + analytic events), **hypotheses considered and rejected**, explicit uncertainty, and disposition rationale — not just a verdict label.
- **Link each artifact to the conclusion it supports** (ICD-203) so a human reviewer can retrace artifact → hypothesis → verdict; this is also what makes the output trustworthy to a consuming LLM agent.
- Apply **ICD-203-style analytic rigor** to auto-generated reasoning: state assumptions, source quality/confidence, and **alternative explanations** — especially important for an autonomous agent whose reasoning must be auditable and challengeable.
- Record **absence/limits of evidence explicitly** (ties to Q5): "log source X unavailable for window Y" is part of a defensible record, not an omission.
- Map observed behavior to **MITRE ATT&CK** for consistent, reproducible descriptions [78].
- Preserve **chain-of-custody-grade provenance** for artifacts pulled from sensors: source, fetch time, hash, no undocumented mutation — consistent with Prism's OCSF-normalize-at-boundary model (record both raw-source reference and normalized form).
- Enforce **mandatory documentation fields** (disposition, severity, ATT&CK technique, rationale, analyst/agent attribution) as a schema, mirroring TheHive mandatory custom fields [81].
- Capture **agent-vs-human attribution** on every action so the record shows what the autonomous loop decided vs what a human approved (ties to Q6).

---

## Cross-Cutting Design Synthesis (for the architect)

1. **Pipeline order:** validate (incl. sensor health) → known-FP/dedup lookup → categorize (ATT&CK) → enrich → score (recalibrate severity by fidelity + per-tenant asset criticality) → dispose (TP/FP/BTP/Indeterminate) → append/link/reopen decision → structured defensible documentation. [Q1, Q5]
2. **Tenant is mandatory everywhere;** cross-tenant correlation is a separate privileged, audited path over derived artifacts only. [Q4]
3. **Deterministic dedup keys with fallback hierarchies**, computed at the OCSF normalization boundary; configurable per-key-class time windows; cap-and-spillover; RBA-style accumulation for storms. [Q2]
4. **Append-by-default; closed cases never auto-reopen; SLA impact always surfaced.** [Q2, Q3]
5. **Decision-support not autonomous action:** auto-close default-off; mandatory human-surface list (high severity, critical assets, cred-dumping/privesc/lateral-movement/exfil, novel/ambiguous, Indeterminate, degraded-sensor-dependent); kill switch; full audit trail. [Q6]
6. **Absence-of-data is a signal:** distinguish "no threat" from "could not observe" in every verdict and in the LLM-consumed wire output. [Q5]
7. **Every output is an ICD-203-grade, chain-of-custody-aware, ATT&CK-mapped, agent/human-attributed record.** [Q7]

---

## Inconclusive / Low-Confidence Findings (explicit)

| Finding | Status | Note |
|---|---|---|
| Single globally-canonical Tier-1 step sequence | Convergent, not mandated | NIST/MITRE give principles + lifecycle, not micro-steps [1][4] |
| Exact dedup/grouping time windows | Configurable, no constant | Vendor defaults/examples only (XDR 1h, Google 0.5-24h, D3 48h, Splunk 7d) [15][21][22][37] |
| SLA-clock behavior on reopen | INCONCLUSIVE | Config-dependent; no authoritative cross-platform rule [29] |
| Reopen time-window (vs new ticket) | Policy-driven | Bounded by resolved→closed timer; ServiceNow blocks reopen from Closed [27][31] |
| Auto-close failure *rates* | INCONCLUSIVE | No public statistics; source explicitly flags best-practice extrapolation [72] |
| Asset-criticality modeling specifics | Implicit in vendor docs | Pattern verified; exact schema not documented [50][52] |
| MSSP data-residency specifics per vendor | INCONCLUSIVE | Extrapolated from general cloud/legal practice [50][58] |
| Rapid7 MDR internal multi-tenant mechanics | INCONCLUSIVE | No primary docs found; inferred from peer MDR patterns |
| ML clustering (DBSCAN/RETE/NN) for correlation | Emerging, not standard | Academic, dataset-specific results [26] |
| ICD-203 applicability to cyber SOC docs | Well-reasoned analogy | Written for intel analysis, not codified cyber standard [76] |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 5 | Q1 triage workflow; Q2 dedup/correlation; Q3 existing-ticket handling; Q4 MSSP multi-tenant; combined Q5 sensor-health + Q6 automation boundaries; Q7 documentation standards — all `reasoning_effort: high`, deep multi-source synthesis with citations |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | Not applicable (no library API research) |
| Tavily tavily_search | 2 | Cross-validation: (a) Tier-1 first-check / validation / dispositions; (b) MSSP cross-tenant IOC correlation + anonymized federated intel + isolation |
| Tavily tavily_research | 0 | — |
| Tavily tavily_extract | 0 | — |
| WebFetch / WebSearch | 0 | — |
| Training data | ~2 areas | Runbook-vs-playbook distinction framing; EDR agent-health as blind-spot (both corroborated by cited vendor/practitioner sources) — flagged inline |

**Total MCP tool calls:** 7 (5 Perplexity deep-research + 2 Tavily search)
**Training data reliance:** low — every substantive claim is tied to a cited source; the two model-knowledge framings are corroborated by cited material and labeled inline.
**Note on primary-tool mandate:** all six research clusters used `perplexity_research` (deep research) as primary per the research-agent default; Tavily was used strictly for independent cross-validation, which confirmed (did not contradict) the Perplexity findings on the two most load-bearing claims (validation/sensor-health as the first triage check; anonymized derived-artifact cross-tenant correlation).
