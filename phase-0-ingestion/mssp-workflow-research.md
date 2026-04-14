# MSSP Analyst Workflow Research

**Date:** 2026-04-13
**Purpose:** Inform the architecture of Prism, a Rust MCP server that MSS analysts will use in Claude Code
**Scope:** Practical day-to-day workflows, not marketing material

---

## 1. Analyst Roles and Workflows

### 1.1 SOC Analyst Tiers in an MSSP

MSSPs typically organize their SOC around a tiered model, but with a critical difference from enterprise SOCs: every analyst juggles multiple client environments simultaneously.

| Tier | Role | Daily Activities | Client Load |
|------|------|-----------------|-------------|
| L1 (Triage) | SOC Analyst | Monitor alert queues, perform initial triage, escalate or close. Follow runbooks. | 5-15 clients per shift |
| L2 (Investigation) | Threat Detection Engineer | Deep-dive investigations, correlate across data sources, write detection rules, tune false positives | 3-8 clients per rotation |
| L3 (Hunt/Response) | Security Engineer / Threat Hunter | Proactive threat hunting, incident response, forensics, custom tool development | 1-3 active incidents + background monitoring |
| Engineering | Detection/Platform Engineer | Build and maintain detection content, sensor integrations, automation playbooks | All clients (platform-wide) |

### 1.2 A Typical L1/L2 Shift (The "Inner Loop")

A concrete hour-by-hour picture of what an MSSP analyst does:

1. **Shift handoff (5-10 min):** Read shift log from outgoing analyst. Check for any ongoing incidents. Review any client-specific advisories or maintenance windows.

2. **Queue triage (continuous):** Alerts arrive in a shared queue, tagged by client. The analyst:
   - Reads alert metadata (sensor source, severity, client name, timestamp)
   - Opens the relevant client context (credentials, sensor access, network topology docs)
   - Investigates: pivots to the sensor's native console (CrowdStrike Falcon, Claroty xDome, etc.) to get details
   - Makes a triage decision: false positive, true positive requiring escalation, informational
   - Documents the decision in the ticketing system

3. **Client context switches (constant, painful):** The single biggest workflow friction in MSSP operations. An analyst might:
   - Triage a CrowdStrike critical alert for Client A
   - Immediately switch to investigating a Claroty OT alert for Client B
   - Then handle an escalation callback from Client C about yesterday's incident
   - Each switch requires loading different credentials, different sensor consoles, different client contact info, different SLA timers

4. **Escalation and communication:** When a true positive is found:
   - Create an incident ticket with client-specific SLA deadlines
   - Notify the client POC (email, phone, Slack -- varies by client)
   - Coordinate with L2/L3 if technical depth needed
   - Track SLA compliance (e.g., "Client A requires 15-minute notification for critical, Client B requires 1 hour")

5. **Shift log and handoff (10-15 min):** Document all open items, pending investigations, and any client-specific context the next shift needs.

### 1.3 The L2/L3 Investigation Workflow

Deeper investigations follow a different pattern:

1. **Receive escalation** with initial triage notes
2. **Establish client context:** Load client's sensor inventory, network topology, known-good baselines
3. **Cross-sensor correlation:** "CrowdStrike detected a suspicious process on host X -- does Claroty xDome show any anomalous OT network traffic from that host's subnet? Does Cyberint threat intel show this hash in any recent campaigns?"
4. **Evidence collection:** Pull raw logs, alert details, device inventories from multiple sensors
5. **Impact assessment:** How many devices affected? What's the blast radius? Is this client's OT environment at risk?
6. **Response coordination:** Work with client's internal team, provide remediation guidance
7. **Post-incident:** Write incident report, update detection rules, share indicators across clients (if non-attributable)

---

## 2. Multi-Client Tooling and UX Patterns

### 2.1 The Two UX Paradigms

Existing MSSP tools use two fundamentally different approaches, and most mature platforms support both:

**Paradigm A: Client Context Switching (Dominant)**
- Analyst selects a client from a dropdown or client list
- The entire UI refilters to show only that client's data
- Like "switching accounts" in a SaaS product
- Used by: Most SIEM/SOAR platforms in multi-tenant mode (Splunk with tenant indices, Microsoft Sentinel with workspaces, Arctic Wolf)
- Pros: Clean isolation, impossible to accidentally mix client data
- Cons: Slow context switches, no cross-client visibility

**Paradigm B: Cross-Client Queue View**
- All alerts from all clients appear in a single prioritized queue
- Each alert is tagged with client identity
- Analyst can filter by client, severity, sensor type, or see everything
- Used by: ConnectWise SIEM (Perch), Datto/Kaseya, some custom MSSP platforms
- Pros: Efficient triage, can spot patterns across clients, no wasted context switches
- Cons: Risk of data leakage if isolation is imperfect, cognitive overload at scale

**Paradigm C: Hybrid (Best Practice)**
- Default view is a cross-client alert queue (sorted by severity, then SLA deadline)
- Drilling into an alert automatically enters that client's context
- Explicit "client focus mode" available for deep investigations
- Cross-client analytics available as a separate, privileged view

### 2.2 What This Means for Prism's MCP Interface

For a CLI/MCP tool used in Claude Code, the interaction model maps to:

```
# Cross-client view (the default)
"Show me all critical alerts across all clients in the last 24 hours"
"Which clients have CrowdStrike sensors that haven't reported in 12+ hours"

# Client-scoped view (investigation mode)
"Switch to client: Acme Corp"
"Show me all Claroty xDome alerts for this client"
"What devices are in this client's OT network?"

# Cross-client analytics (privileged)
"Compare CrowdStrike alert volumes across all clients this week"
"Which clients are missing EDR coverage on their critical servers?"
```

The MCP server needs to support all three patterns. The key design question is: **should `tenant_id` be an explicit parameter on every tool call, or should there be a "current client context" that persists across calls?**

Recommendation for MCP: **Explicit `tenant_id` per call with an optional "default context" mechanism.** Reasons:
- MCP calls are stateless by design (JSON-RPC request/response)
- Claude Code conversations maintain context naturally ("I'm investigating Client A" stays in the conversation)
- The AI agent can inject the tenant_id automatically once the user establishes context
- Cross-client queries use a special `tenant_id: "*"` or `tenant_id: null` pattern

### 2.3 Existing MSSP Platform Architectures

Common architectural patterns in multi-tenant security platforms:

| Platform | Tenant Isolation | Analyst UX | API Model |
|----------|-----------------|------------|-----------|
| Splunk (MSSP) | Index-per-tenant | Role-based index access, search across multiple | Per-index RBAC |
| Microsoft Sentinel | Workspace-per-tenant (Azure Lighthouse) | Cross-workspace queries (KQL union) | ARM + Log Analytics API |
| CrowdStrike Falcon (MSSP) | CID (Customer ID) per tenant | Flight Control for cross-CID management | OAuth2 per CID or MSSP parent CID |
| Arctic Wolf | Proprietary multi-tenant | Single pane of glass with client filtering | Proprietary |
| Secureworks Taegis | Shared platform, logical isolation | Cross-tenant dashboards, per-tenant drill-down | REST API with tenant header |

---

## 3. Sensor Management in an MSSP

### 3.1 What "Managing Sensors" Actually Means

"Sensor management" in an MSSP context encompasses several distinct operational concerns:

**A. Sensor Inventory and Health Monitoring**
- Which sensors does each client have deployed? (CrowdStrike on N endpoints, Claroty monitoring M OT devices, etc.)
- Are sensors reporting? When was the last check-in?
- Are sensor versions current? Any agents in reduced functionality mode?
- This is continuous, background monitoring -- anomalies trigger internal alerts

**B. Configuration Management**
- Sensor policies per client (CrowdStrike prevention policies, Claroty alert thresholds)
- Detection rule tuning (suppressing false positives, adding custom rules)
- Integration configuration (API keys, webhook URLs, data forwarding rules)
- This is periodic, often driven by client onboarding or incident findings

**C. Alert Collection and Triage (Primary Analyst Activity)**
- Pulling alerts from sensor APIs (what the pollers do today)
- Normalizing alert data across different sensor formats (what OCSF normalization does)
- Triaging alerts: assigning severity, determining if actionable
- This is the core, continuous workload

**D. Investigation and Response**
- Querying sensor APIs for additional context during investigations
- Pulling device inventories, network maps, vulnerability data
- Running queries against historical data
- Initiating containment actions (isolating hosts, blocking IPs) -- WRITE operations

**E. Reporting**
- Client-facing monthly reports (alert volumes, incident summaries, SLA compliance)
- Internal metrics (analyst efficiency, alert-to-incident ratios)
- Compliance evidence (demonstrating monitoring coverage)

### 3.2 How This Maps to Prism's MCP Tools

Based on the existing poller analysis and the recovered architecture, here is how sensor management maps to MCP tool categories:

| Operational Concern | MCP Tool Category | Current Codebase Coverage |
|--------------------|--------------------|--------------------------|
| A. Sensor health | `check_sensor_status`, `list_sensors_for_client` | Health server (all pollers), but only self-health, not sensor agent health |
| B. Configuration | Out of scope for initial Prism (write operations are high-risk) | Config is env-var based in all pollers |
| C. Alert collection | `poll_alerts`, `get_alerts`, `get_recent_alerts` | Full coverage in all 4 pollers |
| D. Investigation | `get_devices`, `get_vulnerabilities`, `query_sensor` | Partial: poller-bear has 9 sources, mcp-claroty-xdome has 5 tools |
| E. Reporting | `alert_summary`, `sla_status` | No coverage yet |

### 3.3 The Per-Client Sensor Map

Each MSSP client has a unique sensor deployment:

```
Client: Acme Manufacturing
  Sensors:
    - CrowdStrike Falcon (CID: abc123)
      - 2,400 endpoints
      - Data sources: alerts, detections
      - API: us-1 region
    - Claroty xDome (instance: acme.claroty.cloud)
      - 850 OT devices across 3 sites
      - Data sources: alerts, devices, vulnerabilities
      - API: https://acme.claroty.cloud
    - Cyberint Argos (account: acme-mfg)
      - Threat intel: brand monitoring, credential leaks
      - Data sources: alerts, assets

Client: Beta Financial
  Sensors:
    - CrowdStrike Falcon (CID: def456)
      - 12,000 endpoints
      - Data sources: alerts, detections, hosts
      - API: us-2 region
    - Armis Centrix (instance: beta.armis.com)
      - 3,200 IoT/OT devices
      - Data sources: alerts, devices, activities, vulnerabilities
```

This per-client sensor map is the core metadata that Prism must model. It determines:
- Which API credentials to use for each query
- Which data sources are available per client
- How to route MCP tool calls to the correct sensor instance

---

## 4. AI/MCP Tools in SOC/MSSP Workflows

### 4.1 How AI Is Being Used in SOCs Today (as of early 2026)

AI adoption in SOC operations is real but still early. The dominant patterns:

**Tier 1: Alert Enrichment and Summarization (Widely Adopted)**
- AI summarizes complex alerts into plain-language descriptions
- Auto-enriches IOCs (IP reputation, hash lookups, domain age)
- Generates initial triage recommendations
- Examples: Microsoft Copilot for Security, CrowdStrike Charlotte AI, SentinelOne Purple AI

**Tier 2: Investigation Assistance (Growing)**
- AI suggests investigation steps based on alert type
- Generates KQL/SPL queries for analysts
- Correlates findings across multiple data sources
- Summarizes investigation findings for incident reports
- Examples: Google SecOps Gemini integration, Torq Hyperautomation

**Tier 3: Autonomous Triage (Experimental)**
- AI autonomously triages low-severity alerts
- Makes close/escalate decisions based on historical patterns
- Human reviews AI decisions in batch
- Examples: Limited deployments, mostly in-house at large MSSPs

### 4.2 What an MCP Server Needs to Provide for Claude Code

For a security analyst using Claude Code with Prism as an MCP server, the tool surface needs to support these specific workflows:

**Must-Have Tools (L1/L2 Analyst Inner Loop):**

| Tool | Purpose | Example Invocation |
|------|---------|-------------------|
| `list_clients` | See all managed clients | "Show me my client roster" |
| `get_client_sensors` | See sensor inventory for a client | "What sensors does Acme have?" |
| `get_alerts` | Pull alerts with filtering | "Show CrowdStrike criticals for Acme in last 24h" |
| `get_alert_details` | Deep-dive a specific alert | "Tell me more about alert CS-12345" |
| `get_devices` | Query device inventory | "List all Claroty devices on Acme's Plant 2 network" |
| `get_vulnerabilities` | Query vulnerability data | "What CVEs affect Acme's OT devices?" |
| `check_sensor_health` | Verify sensors are reporting | "Are all of Beta Financial's CrowdStrike agents healthy?" |

**Should-Have Tools (L2/L3 Investigation):**

| Tool | Purpose | Example Invocation |
|------|---------|-------------------|
| `cross_client_alerts` | Compare alerts across clients | "Show all clients with critical alerts today" |
| `search_ioc` | Search for an indicator across all clients | "Has anyone seen this IP: 203.0.113.42?" |
| `get_client_context` | Load full client context for investigation | "Load everything I need to investigate Acme" |
| `alert_timeline` | Chronological view of alerts for correlation | "Show me the alert timeline for Acme 12h before and after this incident" |

**Nice-to-Have Tools (Engineering/Reporting):**

| Tool | Purpose | Example Invocation |
|------|---------|-------------------|
| `alert_statistics` | Volume and trend analysis | "Alert volume trends for all clients this month" |
| `sla_status` | SLA compliance tracking | "Which clients are approaching SLA breach?" |
| `sensor_coverage_gaps` | Identify missing coverage | "Which clients lack OT monitoring?" |

### 4.3 MCP Design Considerations for Security Workflows

**Statelessness with Conversation Context:**
MCP tools are stateless, but security investigations are inherently stateful ("I'm looking at Acme, I found alert X, now I want to pivot to the device"). Claude Code handles this naturally through conversation context -- the AI remembers what client and alert the analyst is investigating. The MCP server should NOT try to maintain session state; it should accept explicit parameters (tenant_id, alert_id, etc.) on every call.

**Response Size Management:**
Security queries can return enormous result sets (10,000+ alerts). MCP tools should:
- Default to small page sizes (25-50 results)
- Support cursor-based pagination
- Return summary counts alongside detailed results
- Include "too many results, please refine your query" guidance

**Credential Isolation:**
Each tool call that touches a sensor API must resolve credentials for the specific client+sensor combination. The MCP server needs a credential resolution chain:
1. Look up client by tenant_id
2. Look up sensor by sensor_type within that client
3. Resolve API credentials for that specific sensor instance
4. Execute the API call with those credentials
5. Never expose credentials in tool responses

**Error Context:**
When a sensor API fails, the error must include enough context for the analyst to act:
- Which client and sensor failed
- Whether this is a transient (retry) or permanent (credential expired) error
- What the analyst should do about it (not just "500 Internal Server Error")

---

## 5. Client Onboarding

### 5.1 The Onboarding Process

Adding a new MSSP client involves several phases:

**Phase 1: Commercial and Scope (Not Prism's concern)**
- Contract negotiation, SLA definition, scope of services
- Output: Signed agreement with defined sensor scope

**Phase 2: Sensor Deployment and Integration (Partially Prism's concern)**
- Deploy sensors at client site or configure cloud-based sensors
- Obtain API credentials for each sensor
- Verify connectivity (test API calls)
- Configure data collection parameters (which data sources, polling intervals, filters)

**Phase 3: Platform Registration (Directly Prism's concern)**
- Register client in the MSSP platform
- Map sensor instances to the client
- Store API credentials securely
- Configure SLA timers and notification rules
- Set up client-specific detection tuning

**Phase 4: Burn-in and Tuning (Ongoing, Prism-relevant)**
- Run initial data collection to establish baselines
- Tune false positive suppression rules
- Verify data normalization (OCSF mapping) is correct
- Train analysts on client-specific context

### 5.2 Per-Client Metadata Model

Based on the existing codebase analysis and MSSP operational patterns, here is the metadata Prism needs to track per client:

```rust
// Core client identity
struct Client {
    tenant_id: TenantId,          // UUID, from axiathon pattern
    display_name: String,         // "Acme Manufacturing"
    short_code: String,           // "ACME" -- for log prefixes, alert tags
    status: ClientStatus,         // Active, Onboarding, Suspended, Offboarded
    onboarded_at: DateTime<Utc>,
    
    // Contact info
    primary_contact: ContactInfo,
    escalation_contacts: Vec<ContactInfo>,
    
    // SLA configuration
    sla_config: SlaConfig,
    
    // Sensor inventory
    sensors: Vec<SensorInstance>,
}

struct ContactInfo {
    name: String,
    email: String,
    phone: Option<String>,
    role: String,                  // "CISO", "SOC Lead", "IT Director"
    notification_preferences: Vec<NotificationChannel>,
}

struct SlaConfig {
    critical_response_minutes: u32,    // e.g., 15
    high_response_minutes: u32,        // e.g., 60
    medium_response_minutes: u32,      // e.g., 240
    low_response_minutes: u32,         // e.g., 1440 (24h)
    business_hours_only: bool,
    timezone: String,                  // "America/Chicago"
}

// Per-sensor instance within a client
struct SensorInstance {
    sensor_id: SensorId,
    sensor_type: SensorType,       // CrowdStrike, Claroty, Cyberint, Armis
    instance_name: String,         // Human-readable: "Acme CrowdStrike US"
    
    // Connection details (varies by sensor type)
    connection: SensorConnection,
    
    // Data sources enabled for this instance
    enabled_sources: Vec<DataSourceConfig>,
    
    // Polling configuration
    poll_interval: Duration,
    
    // Status
    status: SensorStatus,          // Healthy, Degraded, Unreachable, Disabled
    last_successful_poll: Option<DateTime<Utc>>,
    last_error: Option<String>,
}

enum SensorConnection {
    CrowdStrike {
        client_id: CredentialRef,     // Reference to credential store, NOT inline
        client_secret: CredentialRef,
        region: String,               // "us-1", "eu-1", etc.
        cid: String,                  // Customer ID
    },
    Claroty {
        api_token: CredentialRef,
        base_url: Url,
    },
    Cyberint {
        api_token: CredentialRef,
        cookie_auth: CredentialRef,   // The cookie-based auth from poller-express
        environment: String,
    },
    Armis {
        api_key: CredentialRef,
        base_url: Url,
    },
}

struct DataSourceConfig {
    source_type: String,           // "alerts", "devices", "vulnerabilities", etc.
    enabled: bool,
    filter: Option<String>,        // Sensor-specific filter (FQL for CrowdStrike, etc.)
    poll_interval_override: Option<Duration>,
}
```

### 5.3 Credential Management for Onboarding

The credential lifecycle for a new client:

1. **Credential provisioning:** Client provides API keys/secrets (often via secure portal, not email)
2. **Credential storage:** Prism stores them via the `CredentialStore` trait (keyring or encrypted file)
3. **Credential reference:** The sensor configuration stores a `CredentialRef`, not the raw secret
4. **Credential rotation:** Client rotates keys periodically; MSSP updates the stored credential
5. **Credential revocation:** When offboarding, all credentials for the client are purged

Key insight from the serveMyAPI analysis: the current credential model is single-namespace. Prism needs **namespaced credentials** -- `(tenant_id, sensor_id, credential_name)` as the composite key.

---

## 6. Cross-Client Analytics

### 6.1 When and Why Analysts Need Cross-Client Views

Cross-client analysis is not just a "nice to have" -- it is operationally critical for several scenarios:

**Scenario A: Shift Start Triage**
"Show me all critical and high alerts across all my clients, sorted by age (oldest first, because those are closest to SLA breach)."

This is the single most common cross-client query. Every L1 analyst does this at the start of every shift.

**Scenario B: Threat Campaign Detection**
"We just confirmed a ransomware incident at Client A. The attacker used IP 203.0.113.42 and hash abc123. Do any other clients show activity from these indicators?"

This is time-critical. When an MSSP discovers an active threat, they must immediately check all other clients.

**Scenario C: Sensor Health Dashboard**
"Which client sensors haven't reported in the last hour? Are there systemic issues (e.g., CrowdStrike API outage affecting all CrowdStrike clients)?"

This helps distinguish "Client A has a problem" from "CrowdStrike is having an API outage."

**Scenario D: Operational Reporting**
"Alert volume by client for the last 30 days." "Mean time to triage by severity across all clients." "Which clients generate the most false positives (and need detection tuning)?"

This is periodic (weekly/monthly) but essential for MSSP operations management.

### 6.2 Cross-Client Query Patterns for MCP

The MCP tool design should support these cross-client patterns:

```
# Pattern 1: Aggregate with per-client breakdown
Tool: cross_client_alert_summary
Input: { "time_range": "24h", "min_severity": "high" }
Output: [
  { "client": "Acme", "critical": 2, "high": 7, "sensors_reporting": true },
  { "client": "Beta", "critical": 0, "high": 3, "sensors_reporting": true },
  { "client": "Gamma", "critical": 1, "high": 0, "sensors_reporting": false }
]

# Pattern 2: IOC search across all clients
Tool: search_ioc_across_clients
Input: { "ioc_type": "ip", "ioc_value": "203.0.113.42", "time_range": "7d" }
Output: [
  { "client": "Acme", "matches": 3, "sensors": ["CrowdStrike"], "latest": "2026-04-13T10:30:00Z" },
  { "client": "Beta", "matches": 0 }
]

# Pattern 3: Sensor health across all clients
Tool: sensor_health_overview
Input: {}
Output: [
  { "client": "Acme", "sensors": [
    { "type": "CrowdStrike", "status": "healthy", "last_poll": "2m ago" },
    { "type": "Claroty", "status": "healthy", "last_poll": "5m ago" }
  ]},
  { "client": "Gamma", "sensors": [
    { "type": "CrowdStrike", "status": "unreachable", "last_poll": "2h ago", "error": "401 Unauthorized" }
  ]}
]
```

### 6.3 Cross-Client Data Isolation Concerns

Cross-client queries are powerful but carry risk:

- **Data leakage:** An MCP tool that returns cross-client data must ensure the calling analyst has permission to see all clients in the result set
- **Attribution:** Some threat intelligence is client-attributable. An IOC found at Client A should not be shared to Client B unless the MSSP has explicit permission
- **Audit trail:** All cross-client queries should be logged with the analyst's identity, for compliance

For Prism's initial implementation, the simplest safe approach:
- The MCP server runs with a set of configured clients (from config file or credential store)
- All configured clients are visible to the analyst using that Prism instance
- Access control is at the Prism instance level, not per-tool
- Cross-client queries are implicitly permitted for all configured clients

This matches the existing poller model where each poller instance is configured for specific clients via environment variables.

---

## 7. Architectural Implications for Prism

### 7.1 The Client Registry

Prism needs a client registry that is:
- **Loaded at startup** from a configuration file (TOML or YAML) or environment
- **Queryable by MCP tools** ("list all clients", "get client by tenant_id")
- **The authority for credential resolution** (client -> sensor -> credential ref)
- **Not a database** -- MSSPs manage tens to low hundreds of clients, not millions

Suggested config format:

```toml
[[clients]]
tenant_id = "550e8400-e29b-41d4-a716-446655440000"
display_name = "Acme Manufacturing"
short_code = "ACME"

[[clients.sensors]]
sensor_type = "crowdstrike"
instance_name = "Acme CrowdStrike US"
region = "us-1"
cid = "abc123def456"
credential_ref = "acme-cs-client-id"        # Key in credential store
credential_secret_ref = "acme-cs-secret"     # Key in credential store
sources = ["alerts", "detections"]
poll_interval = "30s"

[[clients.sensors]]
sensor_type = "claroty"
instance_name = "Acme Claroty"
base_url = "https://acme.claroty.cloud"
credential_ref = "acme-claroty-token"
sources = ["alerts", "devices", "vulnerabilities"]
poll_interval = "60s"
```

### 7.2 MCP Tool Routing

Every MCP tool call that touches sensor data needs this resolution chain:

```
tool_call(tenant_id, sensor_type?, query_params)
  -> client_registry.get(tenant_id)?
  -> client.get_sensor(sensor_type)? or client.all_sensors()
  -> for each sensor:
       credential_store.resolve(sensor.credential_ref)?
       sensor_adapter.execute(credentials, query_params)?
       ocsf_normalizer.normalize(raw_response)?
  -> aggregate results
  -> return MCP response
```

### 7.3 Key Design Decisions Informed by This Research

| Decision | Recommendation | Rationale |
|----------|---------------|-----------|
| Tenant scoping | Explicit `tenant_id` parameter on every tool, not session state | MCP is stateless; AI maintains context naturally |
| Cross-client queries | Support via `tenant_id: null` meaning "all configured clients" | Critical for shift-start triage and threat hunting |
| Sensor health | First-class tool, not just internal monitoring | Analysts need to know if their data is fresh |
| Credential storage | Namespaced by `(tenant_id, sensor_id, key_name)` | Prevents cross-client credential confusion |
| Response pagination | Cursor-based, default 25 results, max 100 | Prevents overwhelming Claude Code context window |
| Error messages | Include client name, sensor type, and actionable guidance | Analysts need to know which client is affected |
| Client config | File-based (TOML), not database | Scale is tens of clients, not millions |
| Write operations | Not in initial scope | Too risky for AI-initiated actions without human approval loop |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| WebSearch | 2 (denied) | Attempted MSSP workflow research, multi-tenant platform UX |
| WebFetch | 2 (denied) | Attempted Gartner MSSP reviews, ConnectWise MSSP documentation |
| Context7 | 1 (denied) | Attempted MCP SDK documentation lookup |
| Codebase analysis | 12 files read | Existing semport analyses for all 9 repos, recovered architecture, STATE.md |
| Training data | 6 areas | MSSP SOC operations, tiered analyst model, multi-tenant platform UX patterns, sensor management operations, client onboarding processes, cross-client analytics patterns |

**Total MCP tool calls:** 5 attempted, 0 successful (all denied)
**Total codebase reads:** 12 files across semport/ and phase-0-ingestion/
**Training data reliance:** HIGH -- All external research tools were denied. Findings are based on (1) extensive analysis of the 9 existing codebases in this project (verified, high-confidence), and (2) model training data about MSSP operations (unverified against 2026 sources, but based on well-established industry patterns that are slow to change). The codebase-derived findings (sensor types, API patterns, multi-tenant model, credential handling) are high-confidence because they come from actual production code analysis. The MSSP workflow descriptions reflect standard industry practice but could not be cross-referenced with current 2026 sources.
