---
document_type: adr
adr_id: "ADR-016"
title: "Action Delivery Framework"
status: PROPOSED
version: "0.10"
date: 2026-05-03
wave: 4
phase: 4.A
authors: [architect]
producer: architect
timestamp: 2026-05-02T00:00:00Z
inputs:
  - .factory/cycles/wave-4-operations/cycle-manifest.md
  - .factory/cycles/wave-4-operations/preflight-findings/research-findings.md
  - .factory/STATE.md
  - .factory/stories/S-4.08-action-delivery.md
  - .factory/stories/S-4.05-alert-generation.md
  - .factory/stories/S-4.06-case-management.md
  - .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
  - .factory/specs/architecture/decisions/ADR-008-dtu-state-segregation.md
  - .factory/specs/architecture/decisions/ADR-010-customer-config-schema.md
  - .factory/specs/architecture/decisions/ADR-013-schedule-execution-semantics.md
  - .factory/specs/architecture/decisions/ADR-015-detection-rule-language.md
anchor_stories: [S-4.08, S-4.05, S-4.06]
aligns_with: [ADR-006, ADR-008, ADR-010, ADR-013, ADR-015]
references_phase3_siblings: [ADR-019]
locked_decisions: [D-208, D-209, D-210, D-211]
references_research: [R-2, R-3, R-12, R-13]
verification_properties: [VP-044, VP-045, VP-046, VP-047, VP-143]
subsystems_affected: [SS-18, SS-12, SS-13, SS-14]
supersedes: []
superseded_by: null
traces_to: specs/architecture/ARCH-INDEX.md
---

# ADR-016: Action Delivery Framework

## Status

PROPOSED 2026-05-03, v0.10. Pending review and acceptance prior to story remediation and BC authoring.

---

## 1. Context

### 1.1 The Need for a Unified Action Delivery Surface

Wave 4 introduces three categories of operational output that must reach external systems: alert notifications (S-4.05), case lifecycle events (S-4.06), and scheduled report deliveries (S-4.01/S-4.08). Prior to this ADR, each category was siloed in story-level implementation notes with no shared design for credential handling, retry semantics, or observability.

The Action Delivery Framework unifies these under a single `.action.toml` configuration model, a shared `ActionDeliveryEngine` in `prism-operations` (`action/delivery.rs` per ADR-012 `src/` convention), and a common set of verifiable invariants. The framework delivers to four destination kinds: webhook, email (SMTP), syslog (CEF/LEEF), and WASM plugin. Each action spec declares one trigger mode and one destination.

### 1.2 Trigger Plurality

Four trigger modes are required:
- **Alert** — fires when the detection engine (ADR-015) emits an alert matching the filter predicate.
- **Case** — fires when a case undergoes a state-change transition (S-4.06 timeline events).
- **Schedule** — fires at cron-scheduled intervals, registered in the schedule executor (ADR-013 §2.6).
- **Manual** — fires on explicit MCP tool invocation with a confirmation token (S-1.09 gate).

Each mode has distinct delivery semantics. Sharing a single framework across all four while preserving per-mode semantics is the central design challenge this ADR resolves.

### 1.3 Why Separate Decisions Were Needed

Three gaps in the S-4.08 story draft (as of 2026-04-16) required formal resolution:

- **Semaphore sharing.** Story draft proposed a shared semaphore with the schedule executor. D-209 (LOCKED 2026-05-02) overrides this with independent 8-permit semaphores (see §2.11).
- **Cron library.** Story drafts cited `cron 0.12.x`; current latest at adversarial review is 0.15.0; both rejected for the same R-2 reasons (DST/timezone correctness gap). R-2 established `croner = "3"` as the required library (see ADR-013 §2.8 for the canonical decision; ADR-016 inherits it for the schedule trigger mode).
- **Email auth.** Story draft did not address Microsoft Exchange Online's 2026-04-30 SMTP auth deprecation (R-3). XOAUTH2 is now first-class for the email destination.

---

## 2. Decision

### 2.1 `.action.toml` Specification Format

Action specs are declared in TOML files with the `.action.toml` extension. The schema:

```toml
[action]
name        = "slack-critical-alerts"     # unique name within org
description = "..."                       # optional
version     = "1.0"                       # semver string

[trigger]
kind        = "alert"                     # alert | case | schedule | manual
filter      = { severity = ["critical", "high"] }  # optional predicate; see §2.2

# OrgId / ClientId scoping per D-208
org_id      = "01975e4e-9f00-7abc-8def-000000000001"  # UUID v7; required (D-208)
clients     = ["client-acme"]            # non-empty list or ["*"]; see §2.5

[destination]
kind        = "webhook"                  # webhook | email | syslog | plugin
url         = "https://hooks.slack.com/..."
credential_ref = "vault://action-creds/slack/webhook"  # opaque ref per ADR-010 §2.3.1

[retry]
max_attempts = 5                         # bounded per VP-044
schedule     = "exp"                     # exp | linear
```

Top-level field validation follows the same `deny_unknown_fields` + explicit error model as ADR-010 §2.3 (unknown fields are rejected at load time, not silently ignored).

### 2.2 Trigger Model (4 Modes)

**Alert trigger** (`kind = "alert"`): subscribes to the alert broadcast channel (§2.12). The `[trigger].filter` predicate is evaluated against alert metadata fields (`severity`, `rule_id`, `client_id`, `org_id`). Predicate evaluation is performed in the `ActionDeliveryEngine`; non-matching alerts are dropped without delivery attempt. Alert trigger semantics: at-least-once with idempotency-key dedup (§2.5).

**Case trigger** (`kind = "case"`): subscribes to case state-change events emitted by S-4.06 timeline. Filter predicates on case fields (`status`, `severity`, `client_id`). Semantics: at-least-once (audit-relevant).

**Schedule trigger** (`kind = "schedule"`): registers an entry in the `schedules` CF (ADR-013 §2.6) with `kind = "action"` discriminator field in the `ScheduleEntry`. The schedule executor tick loop fires action delivery at the cron-computed interval. Cron parsing uses `croner = "3"` (R-2, canonical decision at ADR-013 §2.8). Semantics: best-effort — if delivery fails, the next scheduled tick fires fresh; no catch-up retry (consistent with ADR-013 §2.4 skip-not-queue policy).

**Manual trigger** (`kind = "manual"`): invoked via MCP tool call (`prism_fire_action`). Semantics: fire-and-forget execution with no confirmation token required inside the action delivery engine. The MCP confirmation gate (S-1.09) is enforced at the MCP-tool-call boundary, NOT inside the `ActionDeliveryEngine`. The engine receives an already-confirmed manual trigger; it performs the delivery and returns the result inline. No automatic retry on failure; the MCP tool call returns the error directly to the caller (AC-11 in S-4.08).

### 2.3 Credential Reference Model (Extends ADR-010 §2.3.1)

The `credential_ref` field in `.action.toml` `[destination]` blocks uses the identical opaque reference scheme set established in ADR-010 §2.3.1:

| Scheme prefix | Example | Resolved by |
|---|---|---|
| `vault://` | `vault://action-creds/slack/webhook` | HashiCorp Vault KV path |
| `env://` | `env://SLACK_WEBHOOK_URL` | Environment variable |
| `file://` | `file:///etc/prism/action-creds/smtp-pass` | File on disk (mode 0600 enforced at load) |
| `keyring://` | `keyring://prism/actions/smtp` | OS keyring (wire-up deferred to S-1.07; TD-S-1.07-01 P1 blocks Wave 5 close) — **REJECTED AT LOAD TIME** until S-1.07 completes |

**`keyring://` load-time rejection:** Any `.action.toml` file whose `credential_ref` uses the `keyring://` scheme is rejected at load time with `E-ACTION-KEYRING-DEFERRED` (pending S-1.07 completion per TD-S-1.07-01). This prevents accepting actions that will never deliver due to the unimplemented keyring backend.

Credential values never appear in the `.action.toml` file. ADR-010 §2.3.1 is the single source of truth for the allowed scheme set; this ADR consumes that scheme set without extending it.

**Inline credential rejection (VP-046):** Any `.action.toml` file whose `[destination]` block contains a literal credential value (a value not prefixed by one of the four allowed scheme strings) returns `E-ACTION-INLINE-CRED` at validation time. Validation occurs at file load, not at delivery time.

**UUID v7 validation for interpolated values (VP-047):** Body template variables (`{{var}}`) in webhook payloads and email subjects are validated at action-spec load time. Each `{{var}}` must be a UUID v7 reference OR an explicitly typed scalar (string literal, integer, or boolean). Free-form template variable names that are not resolvable at load time are rejected with `E-ACTION-TEMPLATE-INJECTION-UNTYPED`.

### 2.4 Delivery Semantics Per Trigger Mode

**Idempotency key definition by trigger mode:**

- **Alert trigger:** `idempotency_key = alert_id` (UUID v7 from the alert generator in S-4.05).
- **Case trigger:** `idempotency_key = timeline_entry_id` (UUID v7 of the `TimelineEntry.id` field in `prism-core::case::TimelineEntry` that caused the state-change event — see §3.6 and ADR-017 §3.6).

Schedule and manual triggers do not use dedup; the `{org_id}:\x02:{action_id}:{idempotency_key}` dedup-key format does not apply to them.

The dedup key stored in the `action_state` CF (§2.5) is uniformly `{org_id}:\x02:{action_id}:{idempotency_key}` for all trigger modes that use dedup (alert and case). The mode-specific idempotency_key definitions above specify what that field contains per trigger type.

| Trigger | Delivery Semantics | Idempotency Key | Rationale |
|---|---|---|---|
| Alert | At-least-once with idempotency-key dedup | `alert_id` (UUID v7 from alert generator) | A lost alert notification is a security event; dedup prevents double-fire on retry |
| Case | At-least-once with idempotency-key dedup | `timeline_entry_id` (UUID v7 of `TimelineEntry.id` per `prism-core::case::TimelineEntry`) | Audit-relevant; each state-change event has a unique timeline entry; see ADR-017 §3.6 |
| Schedule | Best-effort | N/A | Next tick fires fresh; thundering-herd avoidance (ADR-013 §2.4) |
| Manual | Fire-and-forget | N/A | No dedup; fire-and-forget; confirmation token (S-1.09) consumed at MCP-tool-call boundary, not inside action engine |

Alert and case dedup entries are stored in the `action_state` CF (§2.5) with a 24-hour TTL. After TTL expiry, a re-fired alert with the same `alert_id` will fire again — this is the intended behaviour (stale dedup entries should not suppress new firings in a new dedup window).

Dedup window for alert trigger is derived from the linked rule's `effective_dedup_window` (ADR-015 §2.7 owns the resolution; this ADR consumes the resolved value). No OrgRegistry calls occur in the delivery hot path.

### 2.5 OrgId / ClientId Scoping (D-208 + D-210 — LOCKED)

**Rust types:**

```rust
struct ActionSpec {
    org_id: OrgId,                          // required (D-208)
    client_id_filter: ClientFilter,          // see below
    name: String,
    trigger: TriggerConfig,
    destination: DestinationConfig,
    retry: RetryConfig,
}

enum ClientFilter {
    All,                                     // TOML: clients = ["*"]
    Specific(Vec<ClientId>),                 // TOML: clients = ["client-a", "client-b"]
}
```

**`clients = []` is REJECTED (D-210 — LOCKED):** An empty `clients` list returns `E-ACTION-CLIENTS-EMPTY` at validation time. The empty list is an operator error (ambiguous intent: does it mean "no clients" or "all clients"?). The `["*"]` sentinel is the explicit form for "all clients in the org," deserializing to `ClientFilter::All`.

**`action_state` CF key design (per ADR-008 universal re-keying):**

All action-related state (rate limits, last-fire, dedup, dead-letter) lives exclusively in the `action_state` CF. Action state MUST NOT be written to the `detection_state` CF — that CF is owned by ADR-015 (correlation trackers, sequence trackers, dedup trackers keyed by rule). S-4.05 alert-rate-limiting writes go to `action_state` CF, NOT `detection_state` CF; story-writer will remediate S-4.05 in lockstep with this fix.

Note: a future `prism-storage::cf_keys::DiscriminatorRegistry` (planned, not yet implemented) will provide a canonical registry of per-CF discriminator byte assignments to prevent cross-CF collisions.

All keys in the `action_state` CF are prefixed with `{org_id_bytes}:` per ADR-008's universal re-keying rule. Single-byte type discriminators follow the OrgId prefix for efficient per-type scans within an org:

| Entry type | Key format | Value encoding |
|---|---|---|
| Rate limit counter | `{org_id}:\x00:{action_id}:{hour_bucket}` | bincode 2.x: `u64` counter (RocksDB merge_operator per ADR-018 §2 pattern) |
| Last-fire timestamp | `{org_id}:\x01:{action_id}` | bincode 2.x: `DateTime<Utc>` |
| Dedup entry | `{org_id}:\x02:{action_id}:{idempotency_key}` | bincode 2.x: ack `DateTime<Utc>`; TTL 24h |
| Dead-letter entry | `{org_id}:\x03:{action_id}:{idempotency_key}` | bincode 2.x: `DeadLetterRecord`; terminal — written after max_attempts exhausted. Same `idempotency_key` definition as the dedup row immediately above — alert→`alert_id`, case→`timeline_entry_id`, manual/schedule N/A (no dead-letter for fire-and-forget/best-effort modes). |
| Retry state | `{org_id}:\x04:{action_id}:{idempotency_key}` | bincode 2.x: `RetryState { attempt: u8, next_attempt_at: Timestamp, last_error: Option<String> }`; TTL 24h (matches dedup TTL). Same `idempotency_key` definition as dedup/dead-letter rows — alert→`alert_id`, case→`timeline_entry_id`, manual/schedule N/A. |

The `{org_id}:` prefix ensures per-org `reset_for(org_id)` semantics are correct: a prefix-scan on `{org_id}:` deletes all org-A action state without touching org-B entries (ADR-008 guarantee).

### 2.6 Retry and Exponential Backoff (VP-044)

The `[retry]` block in `.action.toml` configures delivery retry. Default and recommended schedule is `exp`.

**`exp` (exponential backoff with jitter) — default:**
- Base: 2s. Multiplier: 2x. Cap: 32s. Max attempts: 5 (default).
- Sequence (nominal): 2s, 4s, 8s, 16s, 32s. Cumulative range: 55.8s–68.2s (nominal ±10% jitter applied per attempt).
- Jitter: ±10% per attempt, applied uniformly. Avoids thundering herd on shared upstreams when many actions target the same destination (e.g., a common Slack webhook).

**`linear` (linear backoff with jitter) — opt-in:**
- Enabled via `[retry] schedule = "linear"` in `.action.toml`. Not the default.
- Base: 5s. Increment: 5s per attempt. Max attempts: 5.
- Sequence (nominal): 5s, 10s, 15s, 20s, 25s.
- Jitter: ±10% per attempt.
- Use case: destinations with known fixed recovery times (e.g., a scheduled maintenance window with predictable end time).

**Terminal failure:** after `max_attempts` exhausted, the event is written to the dead-letter entry in the `action_state` CF (§2.5) for operator inspection. A `ActionDeliveryFailed` event is emitted to the audit log with: `action_id`, `org_id`, `trigger_kind`, `destination_kind`, `attempts`, `last_error`.

VP-044 formally verifies: (a) the retry loop terminates after at most `max_attempts` attempts for any combination of inputs; (b) no infinite retry loop is possible.

VP-045 (in-flight skip): if the semaphore `try_acquire` fails (non-blocking) because all 8 permits are held, the delivery is skipped for this tick/event and a `ActionDeliverySkipped { reason: SemaphoreExhausted }` audit event is emitted. No blocking wait; no queue growth.

### 2.7 WASM Plugin Delegation (R-13)

**Destination kind:** `kind = "plugin"`.

**Wasmtime version pin (R-13):** `wasmtime = "44"` with `features = ["component-model"]`. Major-version bumps follow a 1–2 day per-quarter cadence (R-13 caveat: wasmtime's API surface drifts on each major release; the cadence is necessary to stay current).

**WIT world definition:** Host call surfaces are defined in a WIT file under `crates/prism-operations/wit/action-delivery.wit` (or shared workspace `wit/`):

```wit
interface action-delivery {
    enum severity {
        critical,
        high,
        medium,
        low,
        info,
    }

    enum case-status {
        new,
        acknowledged,
        investigating,
        resolved,
        closed,
    }

    record alert {
        alert-id: string,           // UUID v7 stringified
        org-id: string,             // UUID v7
        client-id: string,          // string slug
        rule-id: string,            // UUID v7
        severity: severity,         // enum: critical, high, medium, low, info
        triggered-at: u64,          // unix epoch micros
        metadata: list<tuple<string, string>>,
    }

    record case {
        case-id: string,
        org-id: string,
        client-id: string,
        status: case-status,        // enum: new, acknowledged, investigating, resolved, closed
        timeline-entry-id: string,  // UUID v7 of triggering TimelineEntry.id (prism-core::case::TimelineEntry)
        triggered-at: u64,
    }

    record report {
        report-id: string,
        org-id: string,
        schedule-id: string,
        fire-epoch: u64,
        metadata: list<tuple<string, string>>,
    }

    fire-alert: func(alert: alert) -> result<_, string>;
    fire-case:  func(case: case)   -> result<_, string>;
    fire-report: func(report: report) -> result<_, string>;
}

world prism-action-plugin {
    import action-delivery;
    export fire: func() -> result<_, string>;
}
```

**Embedder pattern (per Wasmtime 44 component-model API):**

1. Component compiled at plugin load: `Component::new(&engine, wasm_bytes)?`
2. Linker: `Linker::<PrismActionHost>::new(&engine)`; `<generated>::add_to_linker(&mut linker, |state| state)?` where `<generated>` comes from `wasmtime::component::bindgen!` reading the WIT world.
3. Instantiate per-fire: `<generated>::instantiate(&mut store, &component, &linker)?`
4. Host function calls: `instance.fire_alert(&mut store, &alert)?` (etc.)
5. Isolation: instantiating per-fire bounds side effects to a single delivery; the component's internal state does not persist across fires.

**Async host functions:** Use `Linker::func_wrap_async` when the host must `await` on Tokio (e.g., HTTP fanout from within the plugin host context).

**Action plugin authoring SDK** (helper crate for plugin authors) is OUT OF SCOPE for Wave 4. This ADR specifies the host embedder side only. Wave 5+ work.

### §2.7.1 WIT Type Synchronization

Build-time enforcement: `cargo check` in `prism-operations` fails if WIT record definitions in `crates/prism-operations/wit/action-delivery.wit` drift from the canonical Rust struct shapes in `crates/prism-operations/src/types/`. Enforcement mechanism: `cargo-component` or a `bindgen!`-generated output diff'd against canonical Rust types. The CI pipeline runs this check on every PR that touches either the WIT file or the types module. Drift produces a compile error, not a runtime failure.

### 2.8 SMTP / Email Destination with XOAUTH2 (R-3)

**Library pin (R-3):** `lettre = "0.11"` with features:

```toml
features = [
  "tokio1",
  "smtp-transport",
  "rustls",
  "rustls-aws-lc-rs",
  "rustls-platform-verifier",
  "builder",
  "tracing"
]
```

The deprecated `tokio1-rustls-tls` feature flag MUST NOT be used. Story drafts referencing it must be remediated (Wave 4 drift audit item).

**TLS modes:**
- Default: `Tls::Required(TlsParameters::new(host)?)` — STARTTLS on submission port 587.
- Opt-in: `Tls::Wrapper(...)` — implicit TLS on port 465. Configured via `[destination].tls = "wrapper"` in `.action.toml`.

**Auth mechanisms list:** `[Mechanism::Xoauth2, Mechanism::Plain]` default — XOAUTH2 is attempted first; Plain is attempted only as a fallback. `Login` available as opt-in override for legacy Microsoft tenants (configured via `[destination].smtp_auth_fallback = true`). **Ops note:** `Plain` is transmitted only inside a successfully-negotiated TLS session (STARTTLS or TLS wrapper per §2.8 TLS modes); if TLS negotiation fails, no credentials are transmitted and the delivery returns `DeliveryError::TlsFailure`.

**Microsoft Exchange Online (R-3):** Microsoft deprecated basic SMTP auth for Exchange Online on **2026-04-30**. XOAUTH2 is first-class for Wave 4 outbound email to Exchange Online. Deployment note: operators connecting to Exchange Online MUST configure XOAUTH2 with:
- A registered Azure AD application with `SMTP.Send` permission.
- `tenant_id`, `client_id`, and either `client_secret` or `client_certificate` — all stored as opaque `credential_ref` values (§2.3).

Attempting to use `Login` or `Plain` against Exchange Online without SMTP auth re-enabled in the Exchange admin center will produce a `535 5.7.3 Authentication unsuccessful` error at the first delivery attempt, surfaced as a `ActionDeliveryFailed` audit event.

### §2.8.1 OAuth2 Token Refresh

XOAUTH2 bearer tokens have limited lifetimes and must be refreshed before use. Token refresh is handled by the `oauth2 = "4"` crate (or the latest compatible version — verify the current Rust ecosystem pin in the workspace `Cargo.toml`). Refresh flow:

1. On delivery attempt, check if the cached token expires within 60 seconds (`expires_in - 60s` grace window).
2. If within the grace window, proactively refresh the token before initiating the SMTP connection.
3. Refresh failure is mapped to `DeliveryError::Transient` and enters the standard retry backoff schedule (§2.6). The delivery is NOT immediately dead-lettered on token refresh failure — the retry may succeed if the OAuth2 endpoint recovers.
4. Tokens are cached in memory per `(org_id, action_id, credential_ref)` tuple for `expires_in - 60s` seconds.

### 2.9 Webhook Destination with Injection Scanning

**HTTP client:** `reqwest = "0.12"` with features `["json", "rustls-tls", "cookies"]`. Aligned with workspace pin (per research findings; verify in `Cargo.toml` root `[workspace.dependencies]`).

**Body templating:** Handlebars-style `{{var}}` interpolation. All interpolated values pass through `prism-security::InjectionScanner::scan()` BEFORE template render (BC-2.09.004). The scanner detects: SQL injection characters, shell metacharacters, JSON injection sequences, template injection markers. Injection detection returns `E-ACTION-TEMPLATE-INJECTION`; delivery is aborted.

**UUID v7 validation for template variables** is also enforced here at spec-load time (VP-047, §2.3).

**Request signing (optional):** HMAC-SHA256 signature over the request body using a shared secret stored via `credential_ref`. The signature is placed in the `X-Prism-Signature: sha256=<hex>` header. Enabled when `[destination].signing_secret_ref` is set to an opaque `credential_ref`.

### 2.10 Syslog Destination (CEF / LEEF)

Format encoding is defined in **ADR-019** (sibling Phase 3 dispatch — `prism-siem-formats` crate providing `cef::v0::Encoder` and `leef::v2::Encoder`). ADR-016 specifies the syslog DESTINATION semantics; format semantics are delegated to ADR-019.

**Transport:** RFC 3164 / RFC 5424 syslog. Transport selection:
- Default: UDP port 514.
- Opt-in: TCP port 514 (`[destination].transport = "tcp"`).
- Opt-in: TLS port 6514 (`[destination].transport = "tls"`); `credential_ref` for client cert if mutual TLS required.

**Format selection:** `[destination].format = "cef" | "leef"`. Default: `"cef"`. The selected format is passed to the `prism-siem-formats` encoder at delivery time.

### 2.11 Per-Subsystem Semaphore (D-209 — LOCKED)

Per ADR-013 §2.3 (D-209 LOCKED): the `ActionDeliveryEngine` constructs its own `Arc<Semaphore>` with 8 permits at init time inside `action/delivery.rs`. This semaphore is not shared with the `schedule_executor_semaphore` from `schedule/executor.rs`.

**Action delivery liveness invariant (VP-143 — proposed in this ADR):** Action delivery cannot be starved by schedule execution, and schedule execution cannot be starved by action delivery. Proof: the semaphores are disjoint by construction (module-private, not passed between modules). No code path in `action/delivery.rs` can block on `schedule_executor_semaphore`; no code path in `schedule/executor.rs` can block on `action_delivery_semaphore`.

VP-143 is the symmetric pair to VP-137 (per ADR-013 §5.3). Together VP-137 and VP-143 cover the full starvation-freedom invariant for the dual-semaphore design.

**Coordination note re sibling ADR-019:** VP-143 is assigned to this ADR (action delivery non-starvation). ADR-019 (SIEM Output Formats) takes VP-144 as its next available number. No collision.

### 2.12 Broadcast Channel Sizing (R-12)

Alert trigger mode subscribes to the alert broadcast channel via `tokio::sync::broadcast`. Channel capacity: **1000** (rounds to 1024 internally — `tokio::sync::broadcast` uses a power-of-two ring buffer; a capacity request of 1000 produces a ring of 1024 slots per R-12 finding).

This rounding behavior must be documented in the operational runbook so that operators do not assume exactly 1000 in-flight alerts can be buffered before lag occurs.

**Lagged receiver handling:**

```rust
loop {
    match rx.recv().await {
        Ok(alert) => {
            // deliver
        }
        Err(RecvError::Lagged(n)) => {
            metrics::counter!("action_delivery.broadcast.lagged", n);
            warn!(lagged = n, "action delivery receiver lagged; {} alerts dropped");
            // continue — do not break; the receiver is still valid after a lag
        }
        Err(RecvError::Closed) => break,
    }
}
```

Lagged events MUST increment an observability counter. Silently dropping lagged alerts with no metric is a security observability gap — lagging means alerts are being lost without any signal to the operator. The `action_delivery.broadcast.lagged` counter is the actionable signal for tuning the channel capacity or the delivery semaphore permits.

---

## Rationale

**`subsystems_affected` lists all subsystems whose events trigger or are consumed by the framework.** ADR-016 owns the action-delivery framework (SS-18). The `subsystems_affected` field is extended to `[SS-18, SS-12, SS-13, SS-14]` because: SS-12 (Scheduler) provides schedule-trigger events; SS-13 (Alert/Detection) provides alert-trigger events as broadcast channel source; SS-14 (Case Management) provides case-trigger events as state-change sources; SS-18 owns the delivery engine itself. This extension aligns `subsystems_affected` with S-4.08's declared `subsystems: [SS-12, SS-13, SS-14, SS-18]`.

**Per-subsystem semaphore (§2.11) is required for formal liveness.** D-209 rejected the shared 16-permit design because it could not structurally guarantee starvation-freedom between two independent consumers. Independent 8-permit pools allow VP-143 and VP-137 to be proven by structural argument (module-private semaphores with no cross-module passing), not by runtime analysis. This is a stronger guarantee.

**XOAUTH2 first-class (§2.8) is required for Exchange Online continuity.** R-3 documents Microsoft's 2026-04-30 deprecation of basic SMTP auth. Any Wave 4 deployment that targets Exchange Online for alert notifications will fail at first delivery after this date unless XOAUTH2 is available. Adding XOAUTH2 as an opt-in or post-Wave-5 feature would make the action delivery framework non-functional for Exchange Online operators from day one.

**Broadcast capacity 1000 (§2.12) is required for observability.** A capacity that is too small causes silent alert drops (lagged receivers). R-12 found that 1000 (1024 effective) provides adequate headroom for realistic MSSP alert bursts (a policy violation sweep might produce 50–200 alerts; capacity 1000 provides 5–20x headroom). The `RecvError::Lagged` metric path ensures drops are not silent even when capacity is exceeded.

**Credential reference extends ADR-010 §2.3.1 without modification.** Introducing a new credential scheme for `.action.toml` would fragment the credential model across two ADRs. ADR-010's scheme set (vault, env, file, keyring) covers all action-delivery credential scenarios. Reuse ensures the credential store resolver has a single code path.

**WASM plugin isolation per-fire (§2.7) prevents state leakage.** Instantiating a fresh component per `fire_*` call bounds side effects and state to a single delivery. A long-lived component instance would accumulate Wasm linear memory state across fires, creating an implicit state coupling between deliveries that cannot be verified or reasoned about. Per-fire instantiation is a deliberate correctness choice, not a performance oversight; the performance cost (component instantiation) is acceptable at action delivery rates (seconds-to-minutes between fires).

---

## 3. Consequences

### 3.1 Positive

- **Unified outbound surface.** All four trigger modes converge on the same `ActionDeliveryEngine`, `ActionSpec` struct, and `action_state` CF. Observability, retry, and dead-letter handling are centralized — no per-trigger ad-hoc implementation.
- **Per-subsystem isolation (D-209).** Independent semaphores structurally prevent cross-subsystem starvation, enabling VP-137 + VP-143 to be proven. Operational reliability of schedule execution is not coupled to action delivery load, and vice versa.
- **XOAUTH2 future-proofing.** Exchange Online operators are not broken by the 2026-04-30 auth deprecation (R-3). First-class XOAUTH2 support means no emergency patch immediately after Wave 4 deployment.
- **Bounded retry with dead-letter.** VP-044's formal proof of retry termination means operators can rely on at-most-5-attempt behaviour with dead-letter visibility, rather than an action that silently loops forever.
- **Credential model consistency.** Using ADR-010 §2.3.1 scheme set for `.action.toml` means operators familiar with `customers/*.toml` credential references need no new mental model for action credential configuration.

### 3.2 Negative

- **WASM plugin debuggability.** Per-fire instantiation and the WIT interface boundary make step-debugging plugin behavior difficult. Plugin authors must rely on the `fire_*` return value and log output surfaced through the host; native debugger attach is not possible across the host/guest boundary. Mitigation: the plugin authoring SDK (Wave 5+) will provide test harness utilities.
- **Broadcast capacity sizing constraints.** The 1024-slot ring buffer is fixed at channel construction. If sustained alert rates exceed channel throughput, lagged drops occur. The `action_delivery.broadcast.lagged` counter provides the signal, but the only remediation is increasing the capacity constant (requires code change + redeploy). A dynamic capacity is not supported by `tokio::sync::broadcast`.
- **XOAUTH2 configuration complexity.** Exchange Online operators must pre-provision an Azure AD application with `SMTP.Send` permission, register the tenant/client/secret via opaque `credential_ref`, and test the flow before deployment. This is operationally more complex than the deprecated `Login` flow. The deployment note (§2.8) documents the required steps.
- **`lettre` feature flag discipline.** The `tokio1-rustls-tls` deprecation flag is easy to accidentally include from stale documentation or pre-existing `Cargo.toml` stubs. Story-writer must audit all S-4.08-related `Cargo.toml` changes for this flag.

---

## 4. Alternatives Considered

### 4.1 Shared 16-Permit Semaphore (Rejected — D-209)

The S-4.08 story draft proposed a shared `Arc<Semaphore>` with 16 permits covering both schedule execution and action delivery. Rejected by D-209 (LOCKED 2026-05-02) for the cross-subsystem starvation hazard: a burst of action deliveries (e.g., a policy-violation sweep firing 16 simultaneous webhook calls) would starve schedule execution until all permits were released. The 8/8 split eliminates this hazard structurally. No further analysis required; D-209 is locked.

### 4.2 `clients = []` as "All Clients" Default (Rejected — D-210 — LOCKED)

An alternative design would treat `clients = []` as a sentinel for "all clients in the org," saving operators from typing `["*"]`. Rejected by D-210 (LOCKED 2026-05-02) because an empty list is ambiguous: it could represent an operator error (forgot to specify clients), a future "no clients" scope for org-level actions, or the "all clients" intent. Forcing the explicit `["*"]` sentinel makes intent unambiguous and prevents the silent misconfiguration of a broad-scope action caused by a typo or partial edit.

### 4.3 Bare `cron` Crate for Schedule Trigger (Rejected — R-2)

Story drafts cited `cron 0.12.x`; current latest at adversarial review is 0.15.0; both rejected for the same R-2 reasons (DST/timezone correctness gap). Rejected for the same reasons established in ADR-013 §4.2: no DST awareness, no timezone handling, no Quartz-compatible extensions. `croner = "3"` strictly dominates for multi-tenant MSSP scheduling. Canonical decision is at ADR-013 §2.8; this ADR inherits it.

### 4.4 `tokio1-rustls-tls` Feature Flag for Email (Rejected — R-3)

The deprecated `tokio1-rustls-tls` feature flag in `lettre 0.11` was present in story draft `Cargo.toml` stubs. Rejected: this flag is a legacy alias that adds no capability over the `rustls` + `rustls-aws-lc-rs` feature combination and is explicitly deprecated in the lettre 0.11 changelog. Using deprecated flags creates confusion and will break on the next major release.

### 4.5 Inline Credential Values (Rejected — AI-Opaque Credentials Policy)

Placing API keys or webhook URLs as literal string values in `.action.toml` would simplify operator configuration but would expose credentials in any context where the action spec file is read (MCP tool output, git history, log files). The AI-opaque credentials policy (memory: `project_ai_opaque_credentials.md`) prohibits credentials from transiting the AI context. VP-046 formalizes this rejection at the code level.

---

## 5. Verification Plan

### 5.1 VP-044 — Retry Loop Termination (Pre-existing)

**Property:** For any delivery attempt sequence, the retry loop terminates after at most `max_attempts` attempts. No infinite loop is possible regardless of whether the destination returns errors, timeouts, or HTTP 5xx responses on every attempt.

**Method:** Kani (model checking).

**Harness skeleton:**

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_retry_terminates() {
    let max: u32 = kani::any_where(|m| *m >= 1 && *m <= 5);
    let mut attempts = 0u32;
    loop {
        if attempts >= max {
            break;
        }
        attempts += 1;
        let _: bool = kani::any(); // simulated success/failure
        // unconditional break after max attempts
    }
    kani::assert(attempts <= max, "retry must not exceed max_attempts");
}
```

**Status:** draft (VP-044 file exists; harness skeleton to be added per S-4.08 remediation).
**Module:** `prism-operations` | **Priority:** P0 | **Anchor story:** S-4.08

### 5.2 VP-045 — In-Flight Delivery Skip is Non-Blocking (Pre-existing)

**Property:** When the `action_delivery_semaphore` is exhausted (0 permits), `try_acquire` returns `Err(TryAcquireError::NoPermits)` immediately without blocking the calling task. The delivery is skipped; no task is parked waiting for a permit.

**Method:** Proptest + integration test.

**Approach:** Integration test saturates the semaphore with 8 long-running mock deliveries, then calls `try_deliver` for a 9th. Assert the 9th returns `Skipped` within 1ms (no blocking). Proptest verifies the `try_acquire` path is reached (not the blocking `acquire` path) in all code paths through `ActionDeliveryEngine::deliver`.

**Status:** draft (VP-045 file exists; harness skeleton to be added per S-4.08 remediation).
**Module:** `prism-operations` | **Priority:** P1 | **Anchor story:** S-4.08

### 5.3 VP-046 — Inline Credential Rejection (Pre-existing)

**Property:** `ActionSpec::validate` returns `Err(E-ACTION-INLINE-CRED)` for any `.action.toml` where the `[destination].credential_ref` field is set to a value that does not begin with one of the four allowed scheme prefixes (`vault://`, `env://`, `file://`, `keyring://`).

**Method:** Proptest.

**Harness skeleton:**

```rust
proptest! {
    #[test]
    fn inline_cred_rejected(
        value in any::<String>().prop_filter("not a scheme prefix", |s| {
            !s.starts_with("vault://") && !s.starts_with("env://")
            && !s.starts_with("file://") && !s.starts_with("keyring://")
        })
    ) {
        let spec = make_action_spec_with_credential_ref(value);
        let err = ActionSpec::validate(&spec).unwrap_err();
        prop_assert_eq!(err.code(), "E-ACTION-INLINE-CRED");
    }
}
```

**Status:** draft (VP-046 file exists; harness skeleton to be added per S-4.08 remediation).
**Module:** `prism-operations` | **Priority:** P0 | **Anchor story:** S-4.08

### 5.4 VP-047 — Template Variable UUID v7 Validation (Pre-existing)

**Property:** `ActionSpec::validate` returns `Err(E-ACTION-TEMPLATE-INJECTION-UNTYPED)` for any `.action.toml` whose webhook body or email subject contains a `{{var}}` expression where `var` is neither a valid UUID v7 nor a recognized explicitly typed scalar binding.

**Method:** Proptest.

**Approach:** Generate arbitrary `{{var}}` expression strings. For each, verify that only UUID v7 strings and explicitly typed scalar bindings pass validation; all others are rejected with the canonical error code.

**Status:** draft (VP-047 file exists; harness skeleton to be added per S-4.08 remediation).
**Module:** `prism-operations` | **Priority:** P1 | **Anchor story:** S-4.08

### 5.5 VP-143 — Action Delivery Non-Starvation (PROPOSED — NEW)

**Property:** Action delivery cannot be starved by schedule execution, and schedule execution cannot be starved by action delivery. Formally: no code path in `action/delivery.rs` holds or waits on `schedule_executor_semaphore`; no code path in `schedule/executor.rs` holds or waits on `action_delivery_semaphore`.

**Method:** Proptest (structural module-boundary check) + integration test.

**Structural test:** Verify at compile time (via module visibility rules) that `action_delivery_semaphore` is constructed inside `action/delivery.rs` and is not exported; and `schedule_executor_semaphore` is constructed inside `schedule/executor.rs` and is not exported. The semaphore types are module-private; cross-module access is a compile error.

**Integration test:** Saturate the `action_delivery_semaphore` with 8 concurrent mock deliveries that hold their permits for 5 seconds. Trigger 3 schedule fires during this period. Assert all 3 schedule fires complete within 2 tick intervals (2 × 60s default tick = 120s wall-clock), confirming schedule execution is not blocked by action delivery saturation.

**Symmetric pair:** VP-143 is the action-delivery counterpart to VP-137 (schedule-execution liveness, defined in ADR-013 §5.3). Together they cover the full starvation-freedom invariant for the dual-semaphore design (D-209).

**Coordination note:** VP-143 is assigned here (action delivery non-starvation). Sibling ADR-019 takes VP-144 (SIEM format correctness). No collision between Phase 3 sibling ADRs.

**Status:** proposed; VP-143 assigned in this ADR. VP file and VP-INDEX update to be produced before Phase 4.B BC authoring begins.
**Module:** `prism-operations` | **Priority:** P1 | **Anchor stories:** S-4.08

(VP-137 — the symmetric semaphore-liveness VP for schedule executor — has its own anchors S-4.01/S-4.08 per ADR-013 §5.3; VP-143 is anchored to S-4.08 only because the action-delivery semaphore lives in prism-operations/action_dispatcher built by S-4.08, not S-4.01.)

---

## 6. Migration Path

Not applicable. The `prism-operations` crate's action delivery subsystem is greenfield for Wave 4. There is no prior action delivery engine to migrate from. The `action_state` CF does not exist in production RocksDB instances prior to Wave 4 deployment.

Upgrade note for Wave 4 deployment: the `action_state` CF must be created via `create_cf` during process startup if it does not exist. Missing CF on first run is not an error; it is created on-demand at `ActionDeliveryEngine::init()` or pre-created in the RocksDB startup initialization sequence (to be specified in BC-2.14.001 by the story-writer).

---

## Phase 4.A Pass 16 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 16 fix-burst (2026-05-03). Version bumped 0.7 → 0.8.

- **F-P16-M-001 fix (VP-143 anchor §5.5 correction):** Dropped "S-4.01 (secondary)" claim from §5.5 Anchor stories — it was inconsistent with VP-INDEX line 164 (S-4.08 only) and S-4.01 frontmatter (which does NOT carry VP-143). Added explanatory note: VP-137 is the symmetric VP for schedule executor (S-4.01/S-4.08 anchors per ADR-013 §5.3); VP-143 is action-delivery-only (S-4.08 anchor), because the action-delivery semaphore lives in prism-operations/action_dispatcher built by S-4.08, not S-4.01.

---

## Phase 4.A Pass 17 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 17 fix-burst (2026-05-03). Version bumped 0.8 → 0.9.

| 0.9 | F-P17-M-001 | 2026-05-03 | architect | Pass 17 MEDIUM: frontmatter `date:` synced 2026-05-02 → 2026-05-03 (matches body Status). Sibling-fix gap with ADR-013 v0.7 / ADR-018 v0.6 which were already synced. |
| 0.10 | F-PreP18-H-001 | 2026-05-03 | architect | Pre-Pass-18 sweep: Status H2 line synced from stale v0.8 to v0.10 — was missed in Pass 17 M-001 date-fix burst (sister-line regression of F-P16-H-002 pattern). |

---

## Phase 4.A Pass 10 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 10 fix-burst (2026-05-02). Version bumped 0.6 → 0.7.

- **v0.7 (P10 fix — F-P10-H-002):** §2.5 retry-state row key changed from `{org_id}:\x04:{action_id}:{alert_id}` to `{org_id}:\x04:{action_id}:{idempotency_key}` for sister-row symmetry with the dead-letter row (Pass 9 fix) and dedup row. The prior `{alert_id}` placeholder was a Pass 9 partial-fix regression — the dead-letter row was corrected but the retry-state row was not. Clarifying note appended matching dead-letter pattern: alert→`alert_id`, case→`timeline_entry_id`, manual/schedule N/A.

---

## Phase 4.A Pass 9 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 9 fix-burst (2026-05-02). Version bumped 0.5 → 0.6.

- **F-P9-H-002 fix (dead-letter key field name adjudication):** §2.5 dead-letter row key changed from `{org_id}:\x03:{action_id}:{event_id}` to `{org_id}:\x03:{action_id}:{idempotency_key}`. The prior `{event_id}` placeholder was inconsistent with BC-2.18.001 v1.5 (`{alert_id}`) and the §2.5 dedup row (`{idempotency_key}`). The canonical abstract field `{idempotency_key}` is now used uniformly across both the dedup row and the dead-letter row. A clarifying note appended to the dead-letter row defines the concrete value by trigger mode: alert→`alert_id`, case→`timeline_entry_id`, manual/schedule N/A (no dead-letter for fire-and-forget/best-effort modes). BC-2.18.001 and S-4.08 will be aligned by product-owner/story-writer in parallel dispatch.
- **F-P9-M-002 fix (idempotency bullets cleanup):** §2.4 idempotency_key bullet list reduced from 4 entries to 2 (alert and case only). The prior schedule (`N/A`) and manual (fire-and-forget prose) entries were redundant with the §2.4 table and the §2.2 per-mode semantics prose. Replaced with a single sentence: "Schedule and manual triggers do not use dedup; the `{org_id}:\x02:{action_id}:{idempotency_key}` dedup-key format does not apply to them."

---

## Phase 4.A Pass 8 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 8 fix-burst (2026-05-02). Version bumped 0.4 → 0.5.

- **P8-BC-2.18.001-A-H-002 prerequisite fix (retry-state CF row):** Added `\x04` discriminator row to §2.5 `action_state` CF key table. Key format: `{org_id}:\x04:{action_id}:{alert_id}`; value: bincode 2.x `RetryState { attempt: u8, next_attempt_at: Timestamp, last_error: Option<String> }`; TTL 24h. Dead-letter row description updated to mark it as terminal (written after max_attempts exhausted), distinct from retry-state (which tracks in-progress retry attempts).
- **P8-ADR-016-A-M-004 fix (VP-143 tick disambiguation):** §5.5 VP-143 integration test updated from "120s default tick" to "2 × 60s default tick = 120s wall-clock". Default tick is 60s per ADR-013 §2.1; the prior phrasing implied 120s was the default.
- **P8-ADR-013-A-M-005 fix (cron version reconcile, inherited):** §1.3 and §4.3 cron library rejection now use the canonical phrasing: "Story drafts cited cron 0.12.x; current latest at adversarial review is 0.15.0; both rejected for the same R-2 reasons (DST/timezone correctness gap)."

---

## Phase 4.A Pass 4 Remediation Notes

v0.4 body Status section synced from stale v0.3 (P4-XADR-A-H-001).

---

## Phase 4.A Pass 3 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 3 fix-burst (2026-05-02). Version bumped 0.3 → 0.4.

- **P3-ADR-016-A-M-005 fix (manual-trigger dedup contradiction):** Resolved the §2.4 internal contradiction between the "Manual trigger: `idempotency_key = client_supplied_token`" bullet and the §2.4 table "Manual | Fire-and-forget | N/A". Deleted the `client_supplied_token` idempotency_key definition for manual trigger. Manual is now consistently documented as fire-and-forget with no dedup at the action engine layer. The S-1.09 confirmation token is enforced at the MCP-tool-call boundary, not inside the `ActionDeliveryEngine`. §2.4 table Manual row updated with explicit rationale.

## Phase 4.A Pass 2 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 2 fix-burst (2026-05-02). Version bumped 0.2 → 0.3.

- **P2-ADR-016-A-H-001 fix (alert dedup-key contradiction):** §2.4 and §2.5 were inconsistent. Resolved by defining `idempotency_key` explicitly for each trigger mode at the top of §2.4: alert → `alert_id` (UUID v7); case → `timeline_entry_id` (UUID v7 of `prism-core::case::TimelineEntry.id`); schedule → N/A; manual → `client_supplied_token`. The dedup-key format in §2.5 (`{org_id}:\x02:{action_id}:{idempotency_key}`) is now the single canonical form. The §2.4 table was updated to use `idempotency_key` column naming rather than embedding the raw key fragments inline.
- **P2-ADR-016-A-H-002 fix (case dedup key references undefined event_seq):** Case dedup key changed from `{org_id}:{action_id}:{case_id}:{event_seq}` to `{org_id}:\x02:{action_id}:{timeline_entry_id}` where `timeline_entry_id: Uuid` is the existing `TimelineEntry.id` field in `prism-core::case::TimelineEntry`. Cross-reference to `prism-core::case::TimelineEntry` added in §2.4 table and idempotency_key definition block.
- **P2-S-4.08-A-H-001 fix (subsystem mismatch — ADR-016 side):** `subsystems_affected` extended from `[SS-18]` to `[SS-18, SS-12, SS-13, SS-14]`. Rationale added to Rationale section: ADR-016 owns the action-delivery framework; `subsystems_affected` lists all subsystems whose events trigger or are consumed by the framework. This aligns with S-4.08's declared `subsystems:` list.
- **P2-ADR-016-A-M-001 fix (auth order):** §2.8 auth mechanisms list reordered from `[Plain, Xoauth2]` to `[Mechanism::Xoauth2, Mechanism::Plain]`. Ops note added: Plain transmitted only inside successfully-negotiated TLS; if TLS fails, no credentials are transmitted.
- **P2-ADR-016-A-M-002 fix (WIT fields elided):** §2.7 WIT `record alert`, `record case`, `record report` now contain canonical field definitions matching AlertContext/CaseContext/ReportContext. `case.timeline-entry-id` documents the UUID v7 of the triggering `TimelineEntry.id`. Supporting `severity` and `case-status` enums added to the WIT interface.

## Phase 4.A Pass 1 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 1 fix-burst (2026-05-02). Version bumped 0.1 → 0.2.

- **P1-ADR-016-A-H-001 fix:** `subsystems_affected` corrected from `[SS-04]` to `[SS-18]` (Action Delivery Engine). SS-04 = Feature Flags; action delivery framework lives in SS-18.
- **P1-ADR-016-A-H-002 fix:** §2.6 retry schedule reconciled: `exp` mode is the default and authoritative sequence (2s, 4s, 8s, 16s, 32s; cumulative 55.8s–68.2s nominal-jittered). `linear` mode retained as opt-in via `[retry] schedule = "linear"` field. Removed the presentation ambiguity that suggested both were co-equal alternatives.
- **P1-ADR-016-A-H-003 fix:** §2.5 discriminator collision resolved. ALL action-related state (rate limits, last-fire, dedup, dead-letter) lives in `action_state` CF, NOT `detection_state` CF. S-4.05 alert-rate-limiting writes go to `action_state`; story-writer to remediate S-4.05 in lockstep. `DiscriminatorRegistry` future addition noted.
- **P1-ADR-016-A-H-004 fix:** §2.2 Manual trigger clarified: fire-and-forget WITHOUT confirmation token inside the action engine. MCP confirmation gate (S-1.09) enforced at MCP-tool-call boundary; engine receives already-confirmed trigger.
- **P1-ADR-016-A-M-006 fix:** §2.8.1 added: OAuth2 token refresh via `oauth2 = "4"` crate; refresh failure → `DeliveryError::Transient`; `expires_in - 60s` grace window; per-tuple token cache.
- **P1-ADR-016-A-M-007 fix:** §2.7.1 added: build-time WIT type sync check via `cargo-component` / `bindgen!` diff; CI enforced; drift is a compile error.
- **P1-ADR-016-A-M-008 fix:** §2.3 `keyring://` scheme now explicitly rejected at load time with `E-ACTION-KEYRING-DEFERRED` (pending S-1.07). Prevents silent delivery failures from unimplemented keyring backend.

---

## Source / Origin

- **Architectural decisions (STATE.md §Wave 4 Decision Log):**
  - D-207: 6-ADR topology; ADR-016 scoped to action delivery framework; action semaphore half of D-209 documented here (logged 2026-05-02).
  - D-208 (LOCKED): OrgId/ClientId dual hierarchy; `ActionSpec` carries `org_id: OrgId` and `client_id_filter: ClientFilter` (logged 2026-05-02).
  - D-209 (LOCKED): Independent 8-permit semaphores per subsystem; no shared semaphore; `action_delivery_semaphore` is the action-side half (logged 2026-05-02).
  - D-210 (LOCKED): `clients = []` rejected at validation time; `clients = ["*"]` is the explicit sentinel for "all clients in org" (logged 2026-05-02).
  - D-211: Dedup window resolved at scheduling-time; ADR-015 §2.7 owns the resolution; ADR-016 consumes `effective_dedup_window` from `DetectionRuleCache` for alert-trigger dedup TTL (logged 2026-05-02).
- **Research findings (research-findings.md):**
  - R-2 §croner: `croner 3.0.1` recommended; `cron 0.12.x` rejected for DST/timezone deficiency. Canonical decision at ADR-013 §2.8; ADR-016 schedule trigger mode inherits it (2026-05-02).
  - R-3 §lettre+XOAUTH2: `lettre = "0.11"` with `tokio1` + `rustls` features; `tokio1-rustls-tls` deprecated; Exchange Online basic SMTP auth deprecated 2026-04-30; XOAUTH2 first-class for Wave 4 (2026-05-02).
  - R-12 §broadcast: `tokio::sync::broadcast` capacity 1000 rounds to 1024 power-of-two ring; `RecvError::Lagged(n)` must emit observability counter to avoid silent drops (2026-05-02).
  - R-13 §wasmtime: `wasmtime = "44"` with `component-model` feature; `bindgen!` macro; per-fire instantiation for isolation; 1–2 day major-bump cadence per quarter (2026-05-02).
- **Story drafts:**
  - S-4.08-action-delivery.md: primary source for `.action.toml` schema, destination kinds, retry block, VPs (VP-044..VP-047). Story text contains pre-D-209 shared semaphore, pre-R-2 `cron 0.12.x`, pre-R-3 `tokio1-rustls-tls` flag, and non-standard retry sequence — all superseded by this ADR.
  - S-4.05-alert-generation.md: alert broadcast channel as source of alert-trigger events; alert idempotency-key dedup requirement.
  - S-4.06-case-management.md: case state-change timeline events as source of case-trigger events.
- **Prior ADRs:**
  - ADR-006 §2.1: OrgId canonical routing key; `ActionSpec.org_id: OrgId` derives from this.
  - ADR-008: Universal `{org_id}:` CF key prefix rule; `action_state` CF key design derives from this rule.
  - ADR-010 §2.3.1: Opaque `credential_ref` scheme set (`vault://`, `env://`, `file://`, `keyring://`); `.action.toml` uses the identical scheme set without extension.
  - ADR-013 §2.3: D-209 (LOCKED) semaphore split; ADR-016 documents the action-delivery half.
  - ADR-013 §2.6: `schedules` CF `ScheduleEntry`; schedule trigger mode adds `kind = "action"` discriminator.
  - ADR-013 §2.7: `tokio::sync::watch` schedule-change reload hook; ADR-015 consumes it; ADR-016 uses the resulting `effective_dedup_window`.
  - ADR-015 §5: Dedup-window resolution timing (D-211 owned by ADR-015); ADR-016 references the resolved value for alert-trigger dedup TTL.
- **Verification properties:**
  - VP-044 (vp-044-retry-bound.md): pre-existing; harness skeleton to be added per S-4.08 remediation.
  - VP-045 (vp-045-inflight-skip.md): pre-existing; harness skeleton to be added per S-4.08 remediation.
  - VP-046 (vp-046-inline-cred-rejection.md): pre-existing; harness skeleton to be added per S-4.08 remediation.
  - VP-047 (vp-047-uuid-v7-interpolation.md): pre-existing; harness skeleton to be added per S-4.08 remediation.
  - VP-143: proposed in this ADR (action delivery non-starvation; symmetric to VP-137). VP file and VP-INDEX update to be produced before Phase 4.B BC authoring begins. Sibling ADR-019 takes VP-144 — no collision.

---

## 7. References

### Research Findings

- **R-2** (`research-findings.md §R-2`): `croner 3.0.1` recommended for schedule trigger DST/timezone correctness; `cron 0.12.x` and `cron 0.15.0` rejected. Canonical decision at ADR-013 §2.8; ADR-016 inherits it.
- **R-3** (`research-findings.md §R-3`): `lettre = "0.11"` with `tokio1` + `rustls` + `rustls-aws-lc-rs` features; `tokio1-rustls-tls` deprecated and rejected; Microsoft Exchange Online basic SMTP auth deprecated 2026-04-30; XOAUTH2 is first-class.
- **R-12** (`research-findings.md §R-12`): `tokio::sync::broadcast` capacity 1000 rounds to 1024 power-of-two ring; `RecvError::Lagged(n)` must emit observability counters to prevent silent drops.
- **R-13** (`research-findings.md §R-13`): `wasmtime = "44"` with `component-model` feature; `wasmtime::component::bindgen!` macro; `Linker::func_wrap_async` for async host functions; 1–2 day major-bump cadence per quarter.

### Architectural Decisions

- **D-208** (STATE.md §Wave 4 Decision Log — LOCKED): OrgId/ClientId dual hierarchy; all Wave 4 domain types gain `org_id: OrgId`; `ActionSpec` carries `org_id: OrgId` and `client_id_filter: ClientFilter`.
- **D-209** (STATE.md §Wave 4 Decision Log — LOCKED): Independent 8-permit semaphores per subsystem; no shared semaphore; starvation hazard eliminated. ADR-013 §2.3 is authoritative; this ADR consumes the action-delivery half.
- **D-210** (STATE.md §Wave 4 Decision Log — LOCKED): `clients = []` is rejected at validation time; `clients = ["*"]` is the explicit sentinel for "all clients in org." Locked in this ADR.
- **D-211** (STATE.md §Wave 4 Decision Log): Dedup-window resolved at scheduling-time; invalidated on schedule change. ADR-015 §2.7 owns the resolution mechanic; ADR-016 consumes the `effective_dedup_window` value in alert-trigger dedup logic without re-resolving it at delivery time.

### Prior ADRs

- **ADR-006 §2.1**: OrgId is canonical routing key; `ActionSpec.org_id: OrgId` derives from this.
- **ADR-008**: Universal `{org_id}:` CF key prefix rule; `action_state` CF key design in §2.5 derives from this rule.
- **ADR-010 §2.3.1**: Opaque `credential_ref` scheme set (`vault://`, `env://`, `file://`, `keyring://`); `.action.toml` `credential_ref` fields use the identical scheme set.
- **ADR-013 §2.3**: Per-subsystem semaphore split (D-209 LOCKED); ADR-016's `action_delivery_semaphore` is the action-side half of this design.
- **ADR-013 §2.6**: `schedules` CF `ScheduleEntry` — schedule trigger mode adds a `kind = "action"` discriminator entry here.
- **ADR-013 §2.7**: `tokio::sync::watch` schedule-change reload hook; ADR-015 §2.7 consumes this for dedup-window invalidation; ADR-016 alert-trigger delivery uses `effective_dedup_window` from the `DetectionRuleCache` populated by that path.
- **ADR-013 §2.8**: `croner = "3"` cron library decision; schedule trigger mode inherits this.
- **ADR-015 §5**: Alert dedup-window is resolved at scheduling-time (D-211); ADR-016 consumes the resolved `effective_dedup_window` from `DetectionRuleCache` for alert-trigger dedup key TTL. No re-resolution at delivery time.

### Phase 3 Sibling ADR

- **ADR-019** (SIEM Output Formats — Phase 3 sibling dispatch): defines the `prism-siem-formats` crate, `cef::v0::Encoder`, and `leef::v2::Encoder`. ADR-016's syslog destination (§2.10) delegates format encoding to ADR-019; no encoding logic is duplicated here.

### Drift Audit Items Addressed

- Story drift: S-4.08 `[retry]` block cited "2s, 4s, 8s, 30s, 60s" non-standard sequence — resolved by §2.6 establishing standard exponential-backoff sequence (2s, 4s, 8s, 16s, 32s).
- Story drift: S-4.08 referenced `cron 0.12.x` — resolved by inheriting ADR-013 §2.8 `croner = "3"` decision.
- Story drift: S-4.08 referenced `tokio1-rustls-tls` feature flag — resolved by §2.8 rejecting the deprecated flag.
- Story drift: S-4.08 shared 16-permit semaphore — resolved by D-209 (LOCKED); §2.11 documents the 8-permit action-delivery semaphore.
