---
document_type: research
produced_by: Explore-agent
captured_by: day-2-side-analysis
timestamp: "2026-06-25"
scope: "Axiathon (AxiQL-era predecessor) detection engine, rule model/DSL, rule editor, alerting, UX — rip/map for prism day-2 adopt/enhance/drop"
source_repo: /Users/jmagady/Dev/axiathon
do_not_execute: true
note: "Verbatim capture of the 2026-06-25 Explore-agent report. Decisions distilled into matured-vision-day2-requirements.md Section 14. This file preserves the full reference detail (schemas, types, operators, matrices, file paths) so it is not lost on context clear. ASCII TUI mockups summarized; all structured content preserved."
---

# Axiathon Detection Engine & Alerting Architecture — Analysis

## Executive Summary

Axiathon is a Rust-based SIEM/Security Lake designed for detection-as-code. **Early implementation**
(only `axiathon-core` has significant code; other crates are scaffolds). The *specification* is
comprehensive and well-architected, informed by Sigma, EQL, YARA-L, KQL, SPL. Critical caveat for
prism: axiathon assumed an **ingested/centralized** data model (Iceberg/Parquet lake, inline-with-
ingestion detection, RocksDB-persisted correlation state, backtest-from-local-storage) — prism is
**federated/ephemeral**, so correlation/sequence/backtest designs must be adapted (run over the
RetentionCache window, not a lake). See §6.

## 1. Detection Engine Architecture

**Execution model:** inline with ingestion (sub-second alert latency); event-time semantics
(timestamps embedded in events, not wall-clock); streaming + stateful correlation (single-event and
multi-event).

**Data flow:** Sources → Ingestion → Detection (inline) → Alerts → Routing → Notifications, with
Storage (tiered Parquet/Iceberg) off the detection path.

**Key crates (as designed; mostly stubs):**
- `axiathon-detection` — DSL parser, rule engine, Sigma compatibility, alerting
- `axiathon-query` — DataFusion integration, AxiQL parser, multi-version query routing
- `axiathon-core` — OCSF types, TenantContext, AlertId, EventId (only fully implemented crate)

**Performance targets (README):** rule eval < 1s/event (NFR9); event-to-alert < 5s (NFR10); rule
hot-reload < 5s (NFR11); 10,000+ active rules without degradation (NFR12).

**Architecture decisions** (`_bmad-output/planning-artifacts/architecture/core-architectural-decisions/detection-engine/`):
- Rule evaluation: **NFA-based** pattern matching for sequences (Flink CEP / EQL patterns)
- State management: **DashMap (hot cache) + RocksDB (durable backup)**; crash recovery < 10s data loss
- Grouping: field-based keys (e.g. `user.name`, `src_ip`) with cardinality limits
- Sequence detection: FSM with temporal constraints (`max_span`, `max_time_after`)
- Aggregation windows: tumbling (default) + sliding (configurable) with watermarks (event-time)
- Alert-as-input: rules consume alerts from upstream rules (DAG execution)
- Persistent state: RocksDB WAL + column families per tenant
- Risk scoring: per-entity accumulation in RocksDB with temporal decay
- Backfill: DataFusion query → replay through detection engine (retroactive IOC eval, FR1034)

**Correlation types (`correlation-rule-engine.md`, Epic 3 Story 3.13):** Threshold, DistinctCount,
Sequence, CrossSource, Graph, Statistical.

**Correlation state model (per group key):**
```rust
CorrelationBucket {
    group_key: String,           // e.g. "user:alice|src_ip:10.0.0.1"
    events: Vec<Event>,
    sequence_state: Option<u8>,  // current stage in sequence detection
    window_start: DateTime,
    expires_at: DateTime,        // TTL-based expiration
}
```

**Multi-stage detection DAG:** Stage 1 event rules → alerts; Stage 2 alert-to-alert correlation →
escalated alerts; Stage 3 composite (alerts + raw events, windowed hash join); Stage 4 risk + campaign
(per-entity risk accumulation + graph clustering).

**Multi-stage DSL example (AxD):**
```axd
rule detect_campaign {
  input alert
  consumes [detect_malware, detect_lateral_movement]
  match count(alert where rule_id in ("detect_malware","detect_lateral")) >= 2
    group_by src_endpoint.ip within 24h
  risk_score += 25
  alert { title "Multi-stage attack on {src_endpoint.ip}" }
}
```

**Query engine integration (`axiathon-query`):** AST (`src/ast.rs`: AxiQLStatement, FilterExpr,
SelectExpr, SelectItem, StatFunction); Parser (`src/parser.rs`: Chumsky recursive descent); Type
system (`src/type_system.rs`: field type resolution against OCSF); Aliases (`src/aliases.rs`: e.g.
`src_ip` → `src_endpoint.ip`); Config (`src/config.rs`); Versioning (`src/version.rs`: multi-version
OCSF). → These three (type_system/aliases/version) are the multi-schema/multi-version-OCSF reference
for prism (see day-2 §13.6).

## 2. Rule Model & Rule Language

**Rule metadata (YAML)** — fields: `id`, `name`, `description`, `severity`
(critical|high|medium|low|info), `tags`, `version` (semver), `status`
(draft|review|testing|staging|production|deprecated), `author`, `created`, `modified`; `mitre`
(tactic/technique/subtechnique); `detection` (condition, within, group_by, named selections);
`metadata` (author, false_positives[], references[], quality{last_reviewed, review_frequency,
test_coverage, false_positive_rate, mean_time_to_detect}); `changelog[]` (version/date/author/changes).

**Detection DSL (AxD)** (`detection-dsl-grammar.md`, ~20pp, Chumsky parser):
- Logical: `OR`, `AND`, `NOT`
- Comparison: `==`, `!=`, `>`, `<`, `>=`, `<=`
- String: `CONTAINS`, `STARTSWITH`, `ENDSWITH`, `MATCHES` (regex), `ICONTAINS`, …
- List: `IN`, `NOT IN`; CIDR: `IN_CIDR`; Null: `EXISTS`, `NOT EXISTS`
- Named selections (Sigma-compatible reusable blocks): `selection AND NOT filter`
- Precedence (low→high): OR → AND → NOT → comparison/string/list/CIDR/null → ( )

**AST:**
```rust
pub enum Expr {
    Or(Vec<Expr>), And(Vec<Expr>), Not(Box<Expr>),
    Comparison { field: FieldRef, op: CompareOp, value: Value },
    StringOp { field: FieldRef, op: StringOp, value: String },
    ListOp { field: FieldRef, negated: bool, values: Vec<Value> },
    CidrOp { field: FieldRef, cidr: String },
    NullCheck { field: FieldRef, must_exist: bool },
    NamedSelection(String),
}
pub struct FieldRef { segments: Vec<String>, array_index: Option<usize> }
```

**DSL examples:**
- Single-event: `src_endpoint.ip CONTAINS "10." AND process.command_line MATCHES /powershell.*-enc/`
- Threshold: `count(event where status == "failure") > 5 within 5m group_by user.name`
- Sequence: `sequence by user.name with maxspan=30m [a: process.name=="mimikatz.exe"] then [b: access_type=="dump" AND resource=="lsass"] then [c: file_creation AND file_path ENDSWITH ".kdbx"]`

**Sigma compatibility:** primary import format Sigma YAML; native AxiQL/AxD; conversion via pySigma
backend; 90%+ Sigma rules convertible (README).

**Rule lifecycle states:** Draft ○ · Review ◐ · Testing ⟳ · Approved ✓ · Shadow ◑ · Canary ◔ ·
Production ● · Disabled ⊘ · Deprecated ⊗ · Archived ▣. Semver (MAJOR breaking / MINOR enhancement /
PATCH metadata). Git layout: `detection-rules/{rules,tests,exceptions,.github/workflows}`.

**CI/CD pipeline (Story 3.9):** 1 Lint (schema/field resolution) · 2 Unit tests (≥80% cov) ·
3 Backtest (7d, ≥90% TP, ≤15% FP) · 4 Shadow (24h) · 5 Canary (10–25%, auto-rollback if FP>15% or
volume>10×) · 6 Production (canary metrics + analyst approval).

## 3. Rule Editor / Detection Authoring UI

**CLI:** `axiathon rules validate|test --days 7|deploy|shadow <file>`. Backtest output shows
TP/FP/MTTD/confidence + FP sources + auto-tuning recommendations.

**TUI (vim keybindings, SSH-friendly):** Rule list (lifecycle-state icons, filters), Rule detail
(shadow metrics + lifecycle actions + approval), Rule test panel (hits + sample event + FP
candidates), **Staged rollout config** (Shadow→Test-Queue→Canary-tenant→Canary-traffic→Production,
per-stage success thresholds, failure behavior: auto-rollback/pause/manual), Rollout progress monitor,
**Correlation rule builder** (type selector: Threshold/DistinctCount/Sequence/CrossSource/Graph/
Statistical), **Sequence rule builder** (ordered steps, per-step condition, per-step `within`, variable
capture `$user`/`$src_ip`/`$host`, correlation key), **Exception manager** (rule-scoped/global, wildcard/
regex/CIDR patterns, expiry, suppressed counts), **Add-exception modal**, **Auto-tune suggestions**
(confidence-ranked FP exceptions + threshold/window adjustments + estimated impact), **Community rule
import** (SigmaHQ/Elastic/Splunk browser + compatibility check), **Test suite manager** (unit +
backtest + performance cases).

**Web UI:** Monaco editor (AxiQL syntax highlight, real-time validation, autocomplete for fields +
MITRE, inline docs, test panel, rule templates). **MITRE ATT&CK coverage dashboard** (tactic coverage
bars, technique counts, gap analysis with community-rule availability + data-source gaps).

## 4. Alerting / Findings / Notification

**Alert model:**
```rust
pub struct Alert {
  id: AlertId,              // UUID v7
  tenant_id: TenantId, rule_id: RuleId,
  severity: Severity,       // Critical|High|Medium|Low|Info
  status: AlertStatus,      // New|Acknowledged|InProgress|Resolved|Closed|FalsePositive
  title: String, description: String,
  source_events: Vec<EventId>,
  enrichment: AlertEnrichment,  // threat_intel, asset_context, user_context, related_alerts, mitre_tactics
  assignee: Option<UserId>, investigation_id: Option<InvestigationId>,
  created_at/updated_at/acknowledged_at/closed_at, tags: Vec<String>,
}
```

**Alert routing engine:** `RoutingRule { name, condition: RoutingCondition, actions: Vec<RoutingAction>,
priority }`. Conditions: SeverityGte, TagContains, RuleIdMatches, TimeRange, And/Or. Actions: Notify,
AssignTo, EscalateAfter, CreateInvestigation, Suppress, Enrich. Flow: evaluate all rules in priority
order (no short-circuit), execute matching actions in parallel, dedup.

**Notification channels (MVP):** Slack (webhook + bot token, Block Kit interactive buttons, threading),
Teams (Adaptive Cards), PagerDuty (incident + escalation), Email (SMTP HTML), Webhook (HMAC). Slack:
HMAC-SHA256 signature verify, 5-min replay window, Slack-user→Axiathon-user email mapping; actions
Acknowledge/Assign/Create-Investigation/View.

## 5. Other UX/UI

49 wireframes in `_bmad-output/planning-artifacts/wireframes/ui/`: dashboard-main, alert-list-triage,
investigation-timeline, detection-rule-editor, geographic-map, system-administration, temporal-graph-
diff, compliance-dashboard (MITRE heatmap), tenant-provisioning, ot-purdue-model, investigation-
annotations, rule-backtest-results, community-rule-import, report-builder. Accessibility (SOUL.md §11):
no color-only info (pair with symbols), keyboard-accessible, vim keybindings. TUI-first, one-key
pivots, real-time streaming, SSH-friendly (200ms+). Alert subscription config (per-rule/per-severity,
time-based delivery, channel selection, batch vs real-time, escalation).

## 6. Overall Assessment — Adopt / Enhance / Drop

**Genuinely good/reusable:** detection DSL (grammar/AST/type system — Chumsky), correlation engine
architecture (NFA sequences, DashMap+RocksDB, group-by+TTL, multi-stage DAG), alert routing (rule-based
+ plugin channels), rule lifecycle + Git/CI/CD + staged rollout, research artifacts (Detection Rule
System Research 52pp; Detections-as-Code 30pp), MITRE coverage mapping, TUI design.

**Half-baked/abandoned:** implementation mostly scaffolded (only axiathon-core has real code; DSL parser
not implemented; correlation state not built). OT-specific protocol parsing (Modbus/DNP3/S7comm) — but
note: **prism DOES serve OT** (Claroty/Armis + satellite Purdue mesh), so OT detection is IN SCOPE for
prism contrary to the agent's IT-only assumption.

**Misalignment with federated architecture (the key adaptation):**
| Feature | Axiathon (ingested) | Prism (federated) | Status |
|---|---|---|---|
| Single-event match | ingested events | query results | ✓ compatible |
| Stateful correlation | RocksDB-persisted | ephemeral / RetentionCache-window | ⚠ adapt |
| Aggregation windows | continuous arrival | batch over pulled data | ⚠ adapt |
| Sequence detection | FSM over stream | event stream from cache window | ⚠ adapt |
| Backtesting | replay from local Iceberg | re-query remote/cold-tier | ⚠ adapt |
| Alert-as-input DAG | upstream alerts | prior findings same run | ✓ compatible |

**Adopt/Enhance/Drop matrix:**
| Component | Verdict |
|---|---|
| Detection DSL (grammar/AST/type system) | Adopt (but prism = detection-as-query via PrismQL, NOT separate AxD — see day-2 §14.1) |
| Rule schema (YAML) | Adopt |
| Correlation engine architecture | Adopt design, adapt for federation (run over RetentionCache) |
| Correlation state (DashMap+RocksDB) | Adopt — prism keeps RocksDB/RetentionCache (NOT PostgreSQL) |
| Alert model | Adopt |
| Alert routing engine | Adopt |
| Notification channels (Slack/Teams/PagerDuty/email/webhook) | Adopt + add ServiceNow/Jira/Tines |
| Rule lifecycle (Git/CI/CD/staged rollout) | Adopt |
| Exception mgmt + auto-tune | Adopt |
| MITRE coverage + community/Sigma import | Adopt |
| Backtesting | Adapt (federated/cold-tier, not local lake) |
| Rule editor surface (TUI) | Enhance — render on S2 console + MCP + CLI; NO TUI (UX reference only) |
| OT/ICS detection | IN SCOPE for prism (contrary to agent's drop) |
| Ingestion pipeline + Iceberg-as-lake | Drop (prism federated; Iceberg only as cache cold tier) |
