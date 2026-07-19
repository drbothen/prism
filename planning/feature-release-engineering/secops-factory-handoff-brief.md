---
title: "secops-factory Integration Handoff Brief"
status: draft
date: 2026-07-19
audience: secops-factory pipeline (separate repo/session)
prism_version_target: v1.0.0-rc.1
---

# secops-factory Integration Handoff Brief

This document is a self-contained brief for the secops-factory feature-mode pipeline
(separate session, separate repo). It captures all approved decisions, the integration
surface inventory, required skill upgrades, and the monitoring-loop requirement.

**Do not cross-contaminate with prism's `.factory/` state.** secops-factory is a separate
pipeline; its STATE.md, stories, and ADRs are independent.

---

## 1. Approved Decisions (Human-Approved, 2026-07-19)

| Decision | Detail |
|----------|--------|
| Consume prism via stdio MCP | The `activate` skill writes local Claude Code MCP config pointing at the prism binary |
| Minimum prism version | `v1.0.0-rc.1` (check via `prism --version`) |
| RC acceptance gate | Successful live demo through secops-factory IS the RC gate for prism v1.0.0-rc.1 |
| Demo assets | Ship as separate `prism-demo-bundle-${TAG}-${target}` release asset; secops-factory does NOT reference the demo bundle |
| Distribution | GitHub Releases binary + `install.sh`/`install.ps1`; Homebrew deferred |
| Windows parity | `install.ps1` for binary install; bash demo scripts require Git Bash/WSL (RC-1 scope) |
| v1.0.0 final | After soak period; `main` fast-forwarded at GA |

---

## 2. Integration Surface Inventory

### 2.1 `activate` skill — MCP wiring

The `activate` skill is the entry point. When invoked, it:

1. Locates or downloads the `prism` binary (from GitHub Releases if not present)
2. Runs `prism --version` and asserts >= `1.0.0-rc.1`
3. Writes the MCP server block into `~/.claude/settings.json` (or the platform equivalent):

```json
{
  "mcpServers": {
    "prism": {
      "command": "/path/to/prism",
      "args": ["--config-dir", "<config-dir>", "start"],
      "env": {
        "CROWDSTRIKE_BASE_URL": "<value>",
        "ARMIS_INSTANCE_URL": "<value>",
        "CLAROTY_INSTANCE_URL": "<value>",
        "CYBERINT_ENVIRONMENT": "<value>",
        "RUST_LOG": "off"
      }
    }
  }
}
```

4. Verifies the MCP connection via `prism --version` subprocess (not MCP; that requires
   Claude Code restart to pick up the new server config)
5. Prints next-step instructions for credential provisioning

**Important:** `RUST_LOG=off` must always be injected. If omitted, prism's tracing subscriber
writes structured JSON to stdout, which corrupts the MCP JSON-RPC framing and causes the
MCP host to fail all tool calls with parse errors.

### 2.2 `rules/secops-protocol.md` registry entry

The secops-protocol.md rule registry must gain a `prism` entry documenting:
- When to use the `query` MCP tool (table enumeration via `prism_describe`, data fetch)
- PrismQL syntax summary (SQL SELECT form; pipe form rejected)
- Error recovery (E-SPEC-024 boot abort → check env vars; E-CRED-008 → check keyring)
- Multi-org awareness: queries fan out to all orgs that have the sensor configured

The registry entry should also include **Tavily addition**: web-grounded threat context
enrichment via Tavily during investigation and priority assessment. The Tavily MCP tool
augments prism's OCSF-normalized sensor data with external threat intelligence when
prism's built-in enrichment (ThreatIntel/NVD infusion) is insufficient for a specific
indicator.

### 2.3 Onboarding skills — customer and sensor overlay provisioning

Two new skills are needed for customer onboarding:

#### `onboard-customer` skill

Creates a new org entry:
1. Prompts for org slug and UUID (or generates UUID v7 automatically)
2. Appends `[[orgs]]` entry to `prism.toml`
3. Creates `<spec_dir>/customers/<org_slug>/` directory
4. Prints credential provisioning instructions

#### `onboard-sensor` skill (per customer)

Adds a sensor overlay for an existing customer:
1. Prompts for sensor ID (crowdstrike/armis/claroty/cyberint)
2. Prompts for `base_url` (the sensor API endpoint for this customer)
3. Writes `<spec_dir>/customers/<org_slug>/<sensor_id>.sensor.toml` with `extends`, `instance_id`, `base_url`
4. Walks through credential provisioning per AD-017 (piped stdin):

```bash
echo "<credential_value>" | prism --config-dir <dir> credential set \
  --sensor <sensor_id> \
  --name <credential_name> \
  --org-slug <org_slug>
```

5. Verifies the credential write succeeded
6. Runs `SELECT 1` via the `query` MCP tool to confirm prism accepts connections

**AD-017 credential walkthrough:** The skills must never ask the user to paste credentials
into a chat message. The pattern is:
- Describe what credential is needed and its name
- Provide the exact `echo | prism credential set` command
- Ask the user to run it in their terminal
- Verify via `prism_describe` (not credential re-read) that the sensor is registered

### 2.4 Skill upgrades

#### `investigate-event` — evidence collection grounding

Current behavior: unknown (to be determined by secops-factory pipeline team).

Required enhancement: When investigating a security event, the skill must:
1. Query prism for the event's raw sensor data (OCSF-normalized)
2. Query adjacent events in the time window (via PrismQL temporal predicates)
3. Enrich indicators via prism's ThreatIntel/NVD infusion UDFs (if configured)
4. Optionally augment with Tavily web search for CVEs, threat actor TTPs
5. Return structured evidence bundle: sensor data + enrichment + web context

**PrismQL evidence query patterns:**
```sql
-- Raw event lookup:
SELECT * FROM crowdstrike_detections
WHERE _time >= now() - INTERVAL 1 HOUR
LIMIT 50

-- Enriched lookup (requires ThreatIntel infusion):
SELECT *, enrich_threatintel(sha256) AS threat_context
FROM crowdstrike_detections
WHERE _time >= now() - INTERVAL 1 HOUR
```

#### `assess-priority` — prism-grounded priority scoring

Required enhancement: Priority assessment must:
1. Pull historical baseline from prism (last 30-day detection frequency for this indicator)
2. Check NVD enrichment for CVSS score (via `enrich_nvd()` UDF if available)
3. Apply Bayesian TP/FP/BTP disposition framework
4. Output: priority (CRIT/HIGH/MED/LOW), confidence, disposition, rationale

#### `metrics` — per-org sensor telemetry

New skill: Computes per-org sensor health and query volume metrics via prism:
```sql
SELECT * FROM prism_sensor_health
```
Returns: last-seen timestamp, table row counts, error rate per org × sensor.

#### `scan-threats` — proactive threat hunting

New skill: Executes predefined PrismQL hunting queries across all orgs, returns findings
grouped by severity. Uses `prism_describe` to enumerate available tables before querying.

---

## 3. Monitoring Loop — Research-Corrected Design

The monitoring loop is the secops-factory's autonomous patrol mode. It runs on a schedule
(via OS scheduler invoking `claude -p` headless) and performs full Tier-1 alert triage.

**Research basis:** `.factory/research/soc-analyst-workflow-2026.md` (81 cited sources, 2026-07-19).
All numbered section references below ([Q1]–[Q7]) refer to that document.

### 3.1 Per-Alert Pipeline Order (Research-Corrected)

The canonical order from [Q1, Cross-Cutting Synthesis] is:

```
validate (sensor health + data completeness)
  → known-FP / dedup lookup
  → categorize (ATT&CK stage)
  → THEN enrich (prism ThreatIntel/NVD, Perplexity, Tavily)
  → score (recalibrate by rule fidelity + per-tenant asset criticality)
  → dispose (TP / FP / BTP / Indeterminate)
  → append / link / reopen / new-ticket decision
  → structured defensible documentation
```

**Critical ordering constraint:** validation and known-FP matching come BEFORE enrichment.
Enrichment is expensive (external API calls via prism infusion, Perplexity, Tavily). Exaforce
[Q1 §"known-pattern lookup first"] documents that 30–40% of alert volume exits at the
known-pattern check in well-tuned environments — that volume must never spend enrichment
budget. Put the cheap exit first. [Q1]

### 3.2 Full Loop Algorithm

```
FOR EACH customer (org_slug) in prism.toml [[orgs]]:

  STAGE 0: SENSOR HEALTH CHECK
    Query: SELECT * FROM prism_sensor_health WHERE org_slug = <org_slug>
    For each sensor:
      - If last_seen_ts > now() - INTERVAL 24h AND row_count == 0:
          → EMIT sensor-silence finding (§3.7 — positive risk signal, not null)
      - If sensor status = degraded/error:
          → FLAG: subsequent verdicts on this sensor are Indeterminate-due-to-missing-telemetry [Q5]

  FOR EACH healthy sensor:

    STAGE 1: INGEST
      Load watermark from ${CLAUDE_PLUGIN_DATA}/watermarks/<org_slug>/<sensor_id>
      Query: SELECT * FROM <sensor_table>
             WHERE _time > <watermark> ORDER BY _time ASC

    STAGE 2: VALIDATE (before enrichment spend — [Q1])
      For each raw event:
        a. Confirm sensor reported correctly (complete record, not parse error)
        b. Match against known-FP / accepted-risk pattern store (per-tenant, <2 min budget)
           - If known-FP match: DISPOSE as FP immediately, log tuning signal, skip enrichment
        c. Check for duplicates in Jira (per §3.4) before proceeding

    STAGE 3: CATEGORIZE
      Map event to MITRE ATT&CK tactic/technique [Q7]
      Classify attack stage (initial access / lateral movement / exfil / etc.)

    STAGE 4: ENRICH (only for events that passed stages 1–3)
      a. prism ThreatIntel infusion (enrich_threatintel() UDF)
      b. prism NVD infusion (enrich_nvd() UDF) for CVE indicators
      c. Perplexity: external threat context for high-value indicators
      d. Tavily: web-grounded IOC/actor/CVE enrichment (cross-validate)

    STAGE 5: SCORE
      Priority = f(asset_criticality_per_tenant, rule_fidelity, detection_confidence)
      Recalibrate detector-supplied severity by rule FP history [Q1 §"severity is a starting point"]
      Apply per-tenant asset criticality weights [Q4]

    STAGE 6: DISPOSE — four canonical verdicts [Q1 §seven-stage model]
      TP  (True Positive)           → escalate / create ticket
      FP  (False Positive)          → close + emit tuning signal (rule id + asset + reason)
      BTP (Benign True Positive)    → close + consider tuning
      Indeterminate                 → ALWAYS route to human; never auto-close [Q1, Q6]

      Indeterminate triggers when:
        - Insufficient evidence in the available time budget
        - Degraded/silent sensor (verdict depends on missing telemetry) [Q5]
        - Novel alert pattern not matched by any known-FP store
        - Any ambiguous/low-confidence case [Q6]

    STAGE 7: TICKET ACTION — per §3.4 append/link/reopen/new rules
    STAGE 8: DOCUMENT — per §3.8 ICD-203 defensible record

  UPDATE watermark to most recent _time processed
  PERSIST to ${CLAUDE_PLUGIN_DATA}/watermarks/<org_slug>/<sensor_id>
```

### 3.3 Dispositions — Four Canonical Verdicts (Hard Requirement)

These four dispositions are first-class in the loop output schema, not free-text strings.
They map directly to documented SOC practice [Q1 §"disposition — four verdicts"]:

| Disposition | Meaning | Ticket action | Auto-allowed? |
|-------------|---------|---------------|---------------|
| TP | Confirmed malicious | Create/escalate | Only for LOW/MED with documented FP-history gate |
| FP | Confirmed benign | Close (if open) + tuning signal | Yes (narrow scope only) |
| BTP | Real behavior, authorized | Close (if open) + consider tuning | Yes (narrow scope only) |
| Indeterminate | Insufficient evidence | Surface to human; never auto-close | NO — always human |

**Indeterminate is a hard floor:** the loop MUST surface Indeterminate verdicts to a human
via Jira ticket (tagged `[REVIEW-REQUIRED]`) or draft for review. Auto-closing an
Indeterminate verdict is a defect, not a feature. [Q1, Q6]

### 3.4 Jira-First: Append / Link / Reopen / New Rules (Research-Corrected)

The loop checks Jira before creating any ticket. The decision sequence [Q3]:

1. **Duplicate (same root cause, open ticket):** Add a comment to the existing ticket.
   Google's "Find First Alert" pattern: only the first alert creates the ticket; subsequent
   grouped alerts comment "new alert auto-grouped." [Q3 §"append for correlated alerts"]

2. **Related (same asset/IOC, open ticket, different root cause):** Link as related (Jira
   "is related to" link type). Keep tickets separate; link at the case level. [Q3 §"link / parent-child"]

3. **Resolved ticket (same root cause, status = Resolved):** Surface to human — propose
   reopen but do NOT auto-reopen. The loop drafts a comment with evidence and flags for
   human decision. SLA implications must be stated explicitly (see §3.5). [Q3 §"reopen vs new"]

4. **Closed ticket (same root cause, status = Closed):** NEVER auto-reopen. Open a NEW
   ticket and link it to the closed one. [Q3 §"reopen vs new — and the time window"]
   Platform constraint: ServiceNow's Closed state cannot be reopened; same policy applies
   here regardless of ITSM platform. [Q3 citing [27]]

5. **No existing ticket:** Create new ticket if disposition = TP. Draft for review if
   `require-review=true`. Emit tuning signal if disposition = FP/BTP.

**Never auto-reopen a closed ticket.** This is not a configurable option. [Q3]

### 3.5 SLA Impact — Surface, Never Assume (Research-Corrected)

When the loop's ticket action would affect an SLA (append to open ticket, link related,
propose reopen), the loop MUST state the SLA impact explicitly in its output:

> "Appending to ticket PRISM-DEMO-42 (open, created 2h ago, SLA deadline 4h from now).
> Appending does not reset the SLA clock. If this is a new SLA-triggering event, a new
> ticket should be opened."

The loop must NOT silently make choices that change SLA accounting. SLA-clock behavior
on reopen is **configuration-dependent** per platform — there is no universal rule. [Q3
§"SLA implications" citing [29]] The loop surfaces the decision; a human (or explicit
per-tenant SLA policy config) resolves it.

Per-tenant SLA policy config fields:
- `sla_reopen_behavior`: `surface` (default) | `reset` | `resume` — what the loop states
- `resolved_to_closed_window_hours`: the reopen window before auto-escalate-to-new

### 3.6 Cross-Tenant Campaign Correlation — Anonymized Derived Artifacts Only

Cross-tenant correlation is a **separate, explicitly-privileged, audited code path**.
It operates ONLY on derived artifacts (IOCs, hashes, ATT&CK techniques) — never on raw
per-tenant sensor records. [Q4 §"cross-customer / cross-tenant campaign detection"]

**Architecture (Databahn/XSIAM/Google model [Q4]):**
- Raw data: fully isolated per tenant (Prism's `OrgSlug` newtype + RBAC, AD-017 AI-opaque credentials)
- Cross-tenant correlation path: extracts canonical IOC/hash/technique from each tenant's OCSF-normalized records → feeds into an MSSP-internal global correlation store keyed on `{normalized_ioc, technique_id}` → when the same IOC appears in N tenants, emits ONE anonymized MSSP-internal finding: "IOC <hash> seen in {N} tenants"
- Per-tenant view: shows only that tenant's own events; the count N is shared context but not which other tenants
- Every cross-tenant query is audited: who, when, what scope [Q4 citing [47]]

**Contractual constraint [Q4]:** Some customer contracts prohibit cross-tenant use of raw data
beyond anonymized threat intelligence. The default must be derived-artifact-only cross-tenant
analytics. A per-tenant `cross_tenant_correlation: allow | deny` config field controls
whether that tenant's derived artifacts can feed the MSSP-global correlation store.

**Demo Act-3 cross-client pivot — framing note:** The demo's cross-client "same IOC across
customers" pivot must be demonstrated via the anonymized/derived-artifact path described above.
The demo narrative should make this explicit: "Prism detected this IOC in 2 client environments.
Each client sees only their own findings. The MSSP-level correlation is derived from normalized
IOC artifacts, not raw sensor data." This framing is both technically correct and a
differentiating product narrative for MSSP sales. [Q4 §"cross-tenant / cross-tenant campaign"]

### 3.7 Sensor Absence-of-Data as Positive Risk Signal (Research-Corrected)

**Absence of expected data is a first-class alert, not a null result.** [Q5]

If a sensor returns zero rows for a query window AND its last-seen timestamp exceeds the
configured silence threshold (default 24h):

1. Emit a sensor-silence finding — do NOT treat it as "nothing to report"
2. Create or update a `[BLIND-SPOT]` Jira ticket per §3.4 rules
3. The disposition is Indeterminate-due-to-missing-telemetry, NOT FP
4. Do NOT auto-close or suppress — each silence event is independently actionable

**Crucially:** the loop's output for any alert that depended on a now-silent sensor must
distinguish "no threat found" from "could not observe." [Q5 §"design implications"]
These are different statements; conflating them is a silent failure mode that a consuming
LLM agent would misread as a clean clearance.

The `sensor_health_status` field in every loop verdict must be:
- `healthy` — sensor reported within threshold, data is complete
- `degraded` — sensor returned partial data or parse errors
- `silent` — sensor returned no data; verdict is Indeterminate

A `silent` or `degraded` status does NOT suppress the verdict — it qualifies it.

### 3.8 Documentation Standard: ICD-203-Grade Defensible Record (Research-Corrected)

Every disposition the loop produces must be a **structured, defensible record**. [Q7]
The documentation standard draws on NIST SP 800-61r3 (IR lifecycle), NIST SP 800-86
(chain of custody), and ODNI ICD-203 (analytic standards). [Q7 §"what a defensible record contains"]

**Mandatory fields per verdict (schema):**

```
disposition:            TP | FP | BTP | Indeterminate
confidence:             high | medium | low
sensor_health_status:   healthy | degraded | silent
evidence_artifacts:     [ { source, fetch_time_utc, ocsf_normalized_id, hash } ]
timeline_events:        [ { ts_utc, event_type, actor_type (agent|human), description } ]
hypotheses_considered:  [ { hypothesis, supporting_evidence, contradicting_evidence, verdict } ]  # ICD-203
alternatives_rejected:  [ { alternative, reason_rejected } ]                                        # ICD-203
uncertainty_explicit:   "Log source X unavailable for window Y" | null                             # Q5/Q7
attack_techniques:      [ { mitre_technique_id, mitre_tactic, confidence } ]                       # ATT&CK [Q7]
agent_actions:          [ { ts_utc, action, result, requires_human_approval: bool } ]
human_actions:          [ { ts_utc, actor_id, action } ]
tuning_signal:          { rule_id, asset, reason } | null    # for FP/BTP only
```

**ICD-203 analytic rigor [Q7 citing [76][77]]:** The loop must state assumptions, source
quality/confidence, and alternative explanations — especially important for autonomous
agents whose reasoning must be auditable and challengeable by a human reviewer who reads
only the structured output, not the agent's reasoning trace.

**Chain-of-custody provenance [Q7 citing [75]]:** Evidence artifacts are referenced by
identifier + hash (never mutated); fetch time is recorded; the OCSF normalization boundary
is the chain-of-custody boundary (both raw-source reference and normalized form are logged).

**Agent/human attribution on every action [Q6/Q7]:** The record must show what the
autonomous loop decided vs what a human approved. Never blur the two.

**Extension point:** the secops-factory plugin's existing disposition-guard hook is the
natural enforcement point for documentation completeness. The hook should block any
disposition that does not carry all mandatory fields before the ticket action executes.
The disposition-guard hook mirrors TheHive's mandatory custom fields pattern [Q7 citing [81]].

### 3.9 Autonomy Policy — Auto-Close Default-OFF + Hard Floors (Research-Corrected)

**Auto-close is the highest-risk automation.** [Q6 §"auto-closing is the headline risk"]
Default: OFF for all dispositions except narrowly-scoped, documented, low-severity known-FP
patterns with explicit operator sign-off.

**Mandatory human-surface list (hard floors — not configurable off) [Q6]:**

| Category | Examples |
|----------|---------|
| High-severity alerts | Any alert scored HIGH or CRIT |
| Critical-asset alerts | Domain controllers, privileged accounts, OT safety systems |
| High-impact ATT&CK techniques | Credential dumping (T1003), privilege escalation (T1068), lateral movement (T1021), data exfiltration (T1041) |
| Indeterminate verdicts | All (see §3.3) |
| Novel / ambiguous / low-confidence | Any alert not matched by known-FP store |
| Degraded-sensor-dependent | Any verdict where `sensor_health_status != healthy` |
| Cross-tenant campaign correlation findings | MSSP-internal global alerts (always human review before customer notification) |

The loop **proposes** for these categories; a human **disposes**. The autonomy-policy
`require_review` field adds further restrictions on top of the hard floors, never removes them.

**Kill switch:** a per-loop `autonomy_enabled: bool` config flag halts all autonomous
actions while preserving evidence collection and Jira drafting. Provides emergency circuit-breaker
when automation produces unexpected output. [Q6 §"kill switch"]

**Automation-boundary annotation per action:**
- `auto` — loop executes without human approval (narrow FP/BTP scope only)
- `propose-only` — loop drafts; human executes
- `requires-human-approval` — loop creates draft + blocks until approved

Every action in `agent_actions` carries this annotation. [Q6 §"annotated as automated / requires-approval / manual"]

### 3.10 Scheduling and Watermarks

The loop is invoked via OS scheduler (cron on Linux/macOS, Task Scheduler on Windows)
running `claude -p "<monitoring-loop-prompt>" --output-format json`. No interactive UI.

**Suggested schedule:** every 15 minutes for CRIT/HIGH sensors (as classified by
`prism_sensor_health`); every 60 minutes for MED/LOW sensors.

**Watermarks:** `${CLAUDE_PLUGIN_DATA}/watermarks/<org_slug>/<sensor_id>`.
Format: ISO 8601 timestamp of last processed event `_time`.
First run (no watermark): query `WHERE _time >= now() - INTERVAL 24 HOURS` to avoid
processing full sensor history.

---

## 4. Demo Jira Project — B5 Resolved (Human Decision 2026-07-19)

**Resolved decision:** Demo intake is **ticket-first via `jr` against a demo Jira project**.
The monitoring loop is a full feature (patrol, correlation-first, watermarks, Perplexity+Tavily)
but it is NOT the demo's primary intake path. The demo flow demonstrates the SOC analyst
workflow starting from existing tickets.

### 4.1 Demo environment Jira requirements

The demo environment requires:

1. **A demo Jira project** (e.g., key `PRISM-DEMO`) configured in `jr`
2. **`jr` authenticated** in the demo environment — the secops-factory activate skill must
   include a `jr` auth check as part of the onboarding checklist
3. **Demo tickets seeded** in the demo Jira project aligned to DTU scenario stages

### 4.2 Demo ticket seeding — open design item

The demo tickets must be pre-seeded in the demo Jira project to match the DTU fixture data.
The DTU demo server seeds fixtures with stage-aligned timing:
- org-a seed=100 (CrowdStrike, Armis detections)
- org-b seed=150 (Claroty, Cyberint alerts)
- org-c seed=200 (all sensors)

The tickets must correspond to events in these fixtures so `investigate-event` has real
sensor data to pull when investigating from a ticket.

**Open design item — who seeds the demo tickets?** Three approaches:

| Approach | Description | Pros | Cons |
|----------|-------------|------|------|
| **A: demo-ops script** | `demo-seed-jira.sh` (or `.ps1`) creates N demo tickets via `jr` or Jira API at demo setup time | Automated, reproducible, part of the bundle | Requires Jira API creds in demo setup; adds story |
| **B: manual seeding** | Demo operator creates 3-5 Jira tickets manually before running the demo | No automation needed | Fragile; demo operator must remember; breaks scripted demo path |
| **C: monitor loop first pass** | Run the monitor loop once before the demo; it creates tickets from DTU fixture events | Loop gets exercised, ticket seeding is automatic | Loop output may not match demo narrative; ticket titles from raw OCSF data may not be human-readable |

**Recommendation:** Option A (`demo-seed-jira.sh`/`demo-seed-jira.ps1`) — a demo setup script
that creates 3-5 narrative-aligned Jira tickets matching the DTU fixture stages (recon,
lateral movement, exfiltration timing). This makes the demo reproducible and independent of
the monitor loop being run first.

This is an **open design item** — the secops-factory pipeline team must decide which approach
to implement and add the appropriate story. It is NOT a prism release engineering concern.

### 4.3 Monitor loop role in the demo

The monitor loop is a full production feature:
- Runs on a schedule (cron / Task Scheduler)
- Jira-first correlation (duplicate → comment, related → link, new → create)
- Watermarks persisted in `${CLAUDE_PLUGIN_DATA}/watermarks/`
- Perplexity + Tavily enrichment for external context
- Sensor-silence = fail-loud blind-spot finding
- Require-review autonomy policy

In the **demo**, the loop is shown as a background capability — "here's what runs every 15 minutes." The live demo flow uses the pre-seeded tickets (from approach A) as the intake point for `investigate-event`. After the core demo, the loop can be triggered manually to show one additional ticket being created.

### 4.4 `jr` configuration in the demo environment

`jr` must be configured and authenticated before the demo. The secops-factory `activate`
skill MUST include a `jr` auth check step:

```bash
jr auth status  # or equivalent jr health check command
```

If `jr` is not configured, `activate` should print a blocking error with instructions to
set up `jr` against the demo Jira project before proceeding.

The demo Jira project key and server URL are configuration inputs to the secops-factory
activate skill, not hardcoded. The `activate` skill stores these in the plugin config.

### 4.5 Updated demo sequence

```
demo-setup.sh / demo-setup.ps1     (prism binary config + DTU server + keyring)
    ↓
demo-seed-jira.sh / demo-seed-jira.ps1  (create N demo Jira tickets)
    ↓
secops-factory: activate            (version check + MCP config write + jr auth check)
    ↓
secops-factory: onboard-customer    (org-a, org-b, org-c)
    ↓
secops-factory: onboard-sensor      (per-org sensor overlays + credentials)
    ↓
investigate-event on demo ticket    (Jira-first: pull ticket → prism evidence → enrich → score)
    ↓
[show monitor loop as background capability — run once manually]
```

---

## 5. Research Input — SOC Analyst Workflow

The file `.factory/research/soc-analyst-workflow-2026.md` is being authored by a
research-agent in the prism repo. It contains multi-source analysis of SOC analyst
workflows, triage patterns, and MSSP-specific operational constraints.

**Status at time of this brief:** in-flight (not yet committed).

**How to use it:** once committed, the secops-factory pipeline team should read it as input
for the monitoring loop design and skill behavior. It is not a blocking dependency — the
loop design above is sufficient to start implementation. The research artifact may refine:
- Alert triage priority scoring weights
- Correlation window sizes (currently 7d for duplicates, 30d for related)
- False-positive rate thresholds for disposition automation

---

## 6. `.sh` / `.ps1` Parity + BATS Testing

All secops-factory shell scripts must ship with both `.sh` (bash) and `.ps1` (PowerShell)
variants where they perform system operations (MCP config writes, binary install, credential
setup). Pure informational scripts may be bash-only for RC-1.

BATS (Bash Automated Testing System) test coverage is required for all `.sh` scripts that:
- Modify `~/.claude/settings.json` (activate skill shell helper)
- Run `prism credential set` (credential provisioning helper)
- Check binary version (version check helper)

The BATS test suite should test:
1. Happy path for each script
2. Missing binary path (error message + exit code)
3. Version mismatch (error message + download link)
4. Settings.json write (verify correct JSON after write)

---

## 7. Summary: RC-1 Blocking Items for secops-factory

These items must be complete before the prism RC acceptance gate demo:

| Item | Owner | Status |
|------|-------|--------|
| `activate` skill: binary version check (`prism --version` canonical) + MCP config write + `jr` auth check | secops-factory pipeline | Not started |
| `rules/secops-protocol.md` prism entry (+ Tavily) | secops-factory pipeline | Not started |
| `onboard-customer` skill | secops-factory pipeline | Not started |
| `onboard-sensor` skill (with AD-017 credential walkthrough) | secops-factory pipeline | Not started |
| `investigate-event` prism-grounded evidence collection | secops-factory pipeline | Not started |
| Monitoring loop (full: Jira-first + watermarks + silence detection + Perplexity/Tavily) | secops-factory pipeline | Not started |
| Demo Jira project — create project + configure `jr` authentication | Human / secops-factory pipeline | OPEN — project must exist before demo |
| Demo ticket seeding approach decision (§4.2) | Human | OPEN — Option A recommended (demo-seed-jira.sh/ps1) |
| prism v1.0.0-rc.1 binary available at GitHub Releases | prism release pipeline | Depends on S-REL-001 through S-REL-007 |
