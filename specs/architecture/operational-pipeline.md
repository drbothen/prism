---
document_type: architecture-section
level: L3
section: "operational-pipeline"
version: "1.2"
status: draft
producer: architect
timestamp: 2026-04-27T00:00:00
phase: 1b
inputs: [prd.md, domain-spec/capabilities.md, domain-spec/scheduled-detection-concept.md]
traces_to: ARCH-INDEX.md
---

# Operational Pipeline

## [Section Content]

## Overview

Beyond ad-hoc queries, Prism provides a continuous operations loop: scheduled queries -> differential results -> detection evaluation -> alert generation -> case management. All operational components live in `prism-operations` and use `prism-query` (the query engine) as their data source.

```mermaid
graph TD
    subgraph LOOP["Continuous Operations Loop"]
        TICK["Scheduler Tick<br/><i>tokio::time::interval (60s default per ADR-013 §2.1)<br/>try_acquire schedule_executor_semaphore (8 permits per D-209 LOCKED)</i>"]
        QE["Query Engine<br/><i>execute_scheduled()<br/>Same pipeline as ad-hoc</i>"]
        DIFF["Differential Engine<br/><i>SHA-256 hash compare<br/>Row-level added/removed</i>"]
        DET["Detection Engine<br/><i>Single / Correlation / Sequence<br/>Serialized, in-memory only</i>"]
        ALERT["Alert Generation<br/><i>Template interpolation<br/>Dedup, rate limit, persist</i>"]
        NOTIF["MCP Notification<br/><i>alerts://{client_id}<br/>resources/updated</i>"]
    end

    CASE["Case Management<br/><i>5-state lifecycle<br/>MTTD / MTTR metrics</i>"]

    TICK --> QE --> DIFF
    DIFF -->|"Added records only"| DET
    DIFF -->|"Hash match<br/>(no changes)"| SKIP["Silent skip"]
    DET -->|"Rule fires"| ALERT --> NOTIF
    DET -->|"No match"| DONE["Next tick"]
    NOTIF -->|"Analyst creates case"| CASE

    style LOOP fill:#0f3460,stroke:#533483,color:#e0e0e0
    style DET fill:#e94560,stroke:#ff6b6b,color:#fff
    style ALERT fill:#f39c12,stroke:#f1c40f,color:#fff
    style CASE fill:#533483,stroke:#7c3aed,color:#fff
    style SKIP fill:#636e72,stroke:#b2bec3,color:#e0e0e0
    style DONE fill:#636e72,stroke:#b2bec3,color:#e0e0e0
```

## Detection Engine — Three Match Modes

```mermaid
graph TB
    DIFF_IN["Differential Results<br/>(added records)"]

    subgraph SINGLE["Single-Event Mode"]
        S1["Per-record predicate evaluation<br/><i>Stateless — fires immediately on match</i><br/><br/>Example: severity_id >= 4"]
    end

    subgraph CORR["Correlation Mode"]
        C1["Add to sliding window<br/><i>Persisted in RocksDB per (rule, group_key)</i>"]
        C2["Evaluate threshold<br/><i>COUNT(*) >= N within window</i>"]
        C3["Window reset after fire"]
        C1 --> C2 -->|"Threshold met"| C3
    end

    subgraph SEQ["Sequence Mode"]
        SEQ1["Step 1: recon<br/><i>activity = 'PortScan'</i>"]
        SEQ2["Step 2: exploit<br/><i>activity = 'Exploitation'<br/>AND ip = ${recon.src_ip}</i>"]
        SEQ3["Step 3: exfil<br/><i>activity = 'DataExfiltration'</i>"]
        SEQ1 -->|"Match"| SEQ2 -->|"Match"| SEQ3
    end

    DIFF_IN --> SINGLE
    DIFF_IN --> CORR
    DIFF_IN --> SEQ

    SINGLE -->|"Match"| FIRE["Alert Generated"]
    C2 -->|"Threshold met"| FIRE
    SEQ3 -->|"All steps matched<br/>within time window"| FIRE

    style SINGLE fill:#27ae60,stroke:#2ecc71,color:#fff
    style CORR fill:#f39c12,stroke:#f1c40f,color:#fff
    style SEQ fill:#e94560,stroke:#ff6b6b,color:#fff
    style FIRE fill:#533483,stroke:#7c3aed,color:#fff
```

## Case Management Lifecycle

```mermaid
stateDiagram-v2
    [*] --> New
    New --> Acknowledged
    New --> Investigating
    New --> Resolved
    New --> Closed

    Acknowledged --> Investigating
    Acknowledged --> Resolved
    Acknowledged --> Closed

    Investigating --> Resolved
    Investigating --> Closed

    Resolved --> Closed
    Resolved --> Investigating : Reopen

    Closed --> Investigating : Reopen

    state Resolved {
        [*] --> disposition
        disposition: Disposition Required
        note right of disposition
            TruePositive { impact_level }
            FalsePositive { reason }
            Benign { explanation }
            Inconclusive
        end note
    }

    note right of New
        MTTD = created_at - earliest_alert.created_at
    end note

    note right of Resolved
        MTTR = resolved_at - created_at
    end note
```

## Scheduler

The scheduler operates on a tick-based loop using `tokio::time::interval`. Each tick:

1. Scan all active schedules
2. For each schedule where `now >= next_run`: check schedule_executor_semaphore (8 permits per D-209 LOCKED 8/8 split; action_delivery_semaphore is independent 8-permit pool per ADR-016 §2.11)
3. If previous execution for same `(query, client)` is in-flight → skip (DEC-028)
4. Execute via standard query engine pipeline
5. Compute differential results
6. Run detection evaluation on differential output
7. Update schedule state in RocksDB (last_run, next_run, epoch, counter)

**Detection state on spec reload:** When `reload_config` changes a sensor spec's `table_name` or column schema, detection_state entries for rules whose `condition.source` references the changed table are not synchronously purged. Stale entries expire naturally via the 7-day eviction sweep. Stale group_by values harmlessly fail to match against the new schema's field names.

**Splay distribution:** `splay_offset = (interval * splay_percent / 100) * hash(client_id, schedule_name) / MAX_HASH`. Deterministic per `(query_name, client_id)`, persisted to RocksDB for stability across restarts.

**Time drift compensation:** If a tick runs late (e.g., system was busy), the next pause duration is shortened to compensate. Accumulated drift beyond 60s is dropped.

## Differential Results Engine

Per `(query_name, client_id)` pair, maintains in RocksDB:
- `previous_results_hash` — SHA-256 for fast change detection
- `previous_results` — bincode-serialized Arrow RecordBatch for row-level diff
- `epoch` / `counter` — exactly-once semantics

**Algorithm:**
1. Hash current results
2. Compare against stored hash → if equal, silent skip (no output)
3. If different, compute row-level diff using per-row hashes: identify added and removed records
4. Store current results as new previous state
5. Pass DiffResults.added to detection engine (removed records do not trigger detection rules)

Large diffs (10K+ new records) are truncated with analyst notification (DEC-029).

## Detection Engine

Three match modes evaluated against differential results:

### Single-Event Mode
Stateless per-record evaluation. Each new record from the differential is tested against the rule's PrismQL predicate. Fires immediately on match.

### Correlation Mode
Threshold over time window with group-by. New records are added to the persisted sliding window state (RocksDB `detection_state` domain). The full window is evaluated after each addition. Fires when threshold is met; resets window after fire.

### Sequence Mode
Ordered multi-event pattern matching. New records advance the persisted sequence tracker. The tracker maintains progress through the step list per group key. Fires when all steps are matched in order within the time window.

**Rule-to-SQL compilation:** Detection rule predicates are compiled to DataFusion WHERE clauses for push-down optimization. This allows the same DataFusion engine to evaluate both ad-hoc queries and detection rules.

**Rule scoping:** Global (MSSP baseline) → per-client (overrides/additions) → analyst-defined (ad-hoc, runtime). Per-client rules with the same `rule_id` override global rules.

## Alert Generation

When a detection rule fires:
1. Generate `alert_id` (UUID v7, time-sortable)
2. **Snapshot matched events** — While the differential RecordBatch is still in memory, extract `EventSnapshot` for each matched record. This captures the hot column values (severity, hostname, IP, time, message, OCSF class), virtual fields (_sensor, _client, _source_table), and an excerpt of the `event_data` JSON blob (configurable, default 4096 bytes). This is the only opportunity to capture the event data — after SessionContext teardown, the sensor data is gone.
3. Render alert template with variable interpolation (4 resolution levels)
4. Check deduplication key (varies by match mode)
5. Run injection scanner on interpolated sensor values and snapshot data (flag, don't strip)
6. Persist alert (including `matched_event_snapshots`) to RocksDB `alerts` domain
7. Broadcast via `notifications/resources/updated` on `alerts://{client_id}` resource

**Deduplication keys by match mode:**
- Single-event: `(rule_id, event_uid)` — same event cannot trigger same rule twice
- Correlation: `(rule_id, group_by_value_hash, window_bucket)` — one alert per correlation window
- Sequence: `(rule_id, sequence_completion_hash)` — one alert per completed sequence

**RocksDB key encoding for detection_state:** Keys use length-prefixed encoding with a type tag byte: `[rule_id_len: u16][rule_id bytes][type_tag: u8][group_key bytes]`. Type tags:
- `\x00` = correlation/sequence group key (UTF-8 group_by values concatenated, or SHA-256 hash for keys > 128 bytes)
- `\x01` = rate limit entry (group_key bytes = ASCII `rate_limit`)
- `\x02` = dedup entry (group_key bytes = dedup key hash)

The type tag byte prevents collision between group keys and sentinel entries regardless of whether the group_key is UTF-8 or a SHA-256 hash (both use type `\x00`, while rate limit uses `\x01`). No two entry types share the same type tag prefix.

## Case Management

5-state lifecycle: New -> Acknowledged -> Investigating -> Resolved -> Closed.

12 valid transitions: 4 forward linear, 6 skip-ahead, 2 reopen (Resolved/Closed -> Investigating). Exhaustive match in `CaseStatus::can_transition_to()`.

**Auto-computed metrics:**
- MTTD: `case.created_at - earliest_linked_alert.created_at`
- MTTR: `case.resolved_at - case.created_at`

Cases are scoped by `client_id` / `org_slug` (Wave 3+, ADR-006). Cross-client case access prevented by `OrgSlug` typing (formerly `TenantId`).

## Query Packs

Named bundles of scheduled queries + detection rules + aliases for specific MSSP workflows:
- **incident-response** — recent detections, quarantined hosts, lateral movement (every 5 min)
- **daily-triage** — overnight alerts, new assets, credential changes (every 24 hours)
- **compliance** — policy violations, config drift, audit gaps (every 12 hours)

Discovery queries: optional PrismQL query that must return >= 1 row for the pack to activate for a client. Results cached 3600s per `(pack_id, client_id)`.

## End-to-End Walkthrough: Brute Force Detection

This walkthrough traces a complete operational cycle — from scheduler tick to Slack notification — for a brute force detection on MSSP client "Acme Corp."

**Setup:** Acme has CrowdStrike Falcon configured. A scheduled query checks for new detections every 5 minutes. A correlation rule detects 5+ failed logins from the same IP within 5 minutes. A Slack action notifies the SOC on high-severity alerts.

```mermaid
sequenceDiagram
    participant SCH as Scheduler
    participant QE as Query Engine
    participant CS as CrowdStrike API
    participant DIFF as Differential Engine
    participant DET as Detection Engine
    participant DB as RocksDB
    participant ALERT as Alert Generator
    participant ACT as ActionDeliveryEngine
    participant SLACK as Slack
    participant MCP as MCP → Claude Code

    Note over SCH,MCP: Every 5 minutes (background, automatic)

    SCH->>SCH: Tick: cs_detections due for acme<br/>try_acquire(semaphore) → OK
    SCH->>QE: execute_scheduled("SELECT * FROM<br/>crowdstrike_detections WHERE time > 5m", acme)
    QE->>CS: GET /detects/queries/detects/v1?filter=...
    CS-->>QE: 250 detection IDs
    QE->>CS: POST /detects/entities/summaries/GET/v1<br/>(3 batches × 100)
    CS-->>QE: 47 full detection records
    QE->>QE: OCSF normalize → Arrow RecordBatch
    QE-->>SCH: (47 records, live SessionContext)

    SCH->>DIFF: compute_diff(previous, current)
    DIFF->>DB: Load previous hash
    DIFF-->>SCH: 8 added, 3 removed
    DIFF->>DB: Store current as new previous

    SCH->>DET: Evaluate rules against 8 added records

    Note over DET,DB: Rule: brute_force (correlation)
    DET->>DET: Filter: activity="Auth" AND status="Failure"<br/>3 of 8 records match
    DET->>DB: Load correlation window for IP 10.0.1.50
    DET->>DET: Window: 2 previous + 3 new = 5 total<br/>Threshold 5 → FIRE!
    DET->>DB: Reset correlation window

    DET->>ALERT: Generate alert
    ALERT->>ALERT: Snapshot 5 matched events
    ALERT->>ALERT: Render template:<br/>"Brute Force: 5+ failed logins from 10.0.1.50"
    ALERT->>ALERT: Check dedup + rate limits → OK
    ALERT->>DB: Persist alert (alerts CF)

    ALERT->>MCP: notifications/resources/updated<br/>alerts://acme
    ALERT->>ACT: New alert (severity=high, client=acme)

    ACT->>ACT: Evaluate actions:<br/>slack_soc filter matches (severity >= 4)
    ACT->>SLACK: POST webhook with alert details
    SLACK-->>ACT: 200 OK

    SCH->>SCH: Drop SessionContext (memory freed)<br/>Release semaphore permit<br/>Schedule next_run = now + 300s + splay

    Note over MCP: AI agent reacts to notification
    MCP-->>MCP: "New high-severity alert for Acme Corp.<br/>Brute force: 5+ failed logins from 10.0.1.50.<br/>Checking threat intel..."
```

### What the Analyst Experiences

The analyst is working in Claude Code. They might be in the middle of an unrelated task when the AI interrupts:

> **Claude:** A new high-severity alert just fired for Acme Corp. A brute force attack was detected — 5 failed login attempts from IP 10.0.1.50 within 5 minutes, targeting WORKSTATION-7. The SOC Slack channel has also been notified.
>
> Let me check threat context...
>
> *[calls query with geoip and threat_intel infusions]*
>
> The IP 10.0.1.50 geolocates to Saint Petersburg, Russia. Threat score: 92/100 (known malicious per GreyNoise). This IP has been involved in credential stuffing campaigns since March 2026.
>
> I recommend:
> 1. Create a case for this incident
> 2. Contain the affected host via CrowdStrike
> 3. Check if this IP has targeted other clients
>
> Would you like me to proceed?

The entire detection → enrichment → notification chain happened automatically while the analyst was doing other work. The AI has full context (alert details, matched events, threat intel) and can take immediate action.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-04-27 | product-owner | Pass 15 sweep: Case scoping line updated TenantId → OrgSlug (ADR-006 Wave 3); added `## [Section Content]` template compliance marker. |
| 1.2 | F-P23-H-001+M-001 | 2026-05-04 | architect | Pass 23 SUBSTANTIVE: D-209 8/8 split + ADR-013 §2.1 60s tick + ActionDeliveryEngine rename propagated. Sites: line 25 Mermaid Scheduler Tick label, line 135 prose, line 240 Mermaid participant. Pre-Pass-21 sweep target list missed this file (TD-VSDD-048 candidate — broad-sweep methodology needs grep-completeness enforcement). |
