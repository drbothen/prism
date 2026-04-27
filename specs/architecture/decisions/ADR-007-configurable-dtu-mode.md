---
document_type: adr
adr_id: ADR-007
title: "Configurable shared/client DTU mode — per-type default registry, config schema, and isolation semantics"
status: PROPOSED
date: 2026-04-27
wave: 3
phase: 3.A
version: "0.4"
authors: [architect]
related_decisions: [D-042, D-045, D-049, D-051]
related_adrs: [ADR-006, ADR-008, ADR-010]
related_bcs_planned: [BC-3.2.004, BC-3.2.005, BC-3.3.001]
anchored_capabilities: [CAP-040]
subsystems_affected: [SS-03, SS-05, SS-06]
supersedes: null
superseded_by: null
traces_to: specs/architecture/ARCH-INDEX.md
inputs:
  - .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
  - crates/prism-dtu-common/src/config.rs
  - crates/prism-dtu-common/src/clone.rs
  - crates/prism-dtu-claroty/src/state.rs
  - crates/prism-dtu-crowdstrike/src/state.rs
  - crates/prism-dtu-slack/src/state.rs
  - crates/prism-dtu-pagerduty/src/state.rs
  - crates/prism-dtu-jira/src/state.rs
  - .factory/STATE.md (D-042, D-045)
---

# ADR-007: Configurable Shared/Client DTU Mode — Per-Type Default Registry, Config Schema, and Isolation Semantics

## Status

PROPOSED — decision D-042 recorded. Extends ADR-006 Section 2.4. BCs to be authored in subsequent
Phase 3.A spec-writer dispatch. Implementation BLOCKED until Phase 3.A converges (D-045).

---

## 1. Context

### 1.1 Two Categories of DTU in the MSSP Deployment

ADR-006 established that the per-analyst MCP deployment model (memory:
`project_deployment_model.md`) must manage two structurally distinct categories of
downstream integration:

**Client-scoped sensors** are security telemetry sources where each managed customer
organization has its own independent vendor account. Claroty xDome, Armis Centrix,
CrowdStrike Falcon, and Cyberint Argos each require dedicated credentials, unique
API endpoints, and completely isolated data. An Armis query for Org A must never
dispatch to Org B's Armis instance, and device records from Org A's environment must
not appear in Org B's query results.

**MSSP-internal coordination tools** are shared infrastructure operated by the MSSP
itself, not provisioned per-customer. Slack, PagerDuty, and Jira are operated by the
MSSP; one Slack workspace, one PagerDuty account, and one Jira instance serve all
managed organizations simultaneously. A PagerDuty alert created on behalf of Org A
and an alert created on behalf of Org B both route to the same PagerDuty API endpoint,
distinguished by metadata fields (summary text, labels, custom details) rather than by
separate API credentials or endpoints.

The built-in sensors philosophy (memory: `feedback_builtin_sensors_config_driven.md`)
requires that this distinction be encoded in operator-visible TOML configuration, not
hardcoded in the type system. This allows the MSSP operator to override the default
behavior for non-standard deployments (e.g., a customer who has procured a dedicated
Slack workspace, or an NVD integration that is scoped to a single org).

### 1.2 What ADR-006 Deferred

ADR-006 Section 2.4 introduced the `mode = "shared" | "client"` field for `[[dtu]]`
blocks and sketched the semantic distinction, but deferred to ADR-007 the specification
of:

- The canonical per-DTU-type default mode registry
- The precise semantics of `shared` vs `client` at the state and routing layers
- The validation rule that prevents a sensor DTU from being misconfigured as `shared`
- The migration path for the 11 existing DTU crates

### 1.3 Security Threat: Misconfigured Sensor Mode

The primary threat introduced by configurable mode is a configuration error where an
operator sets `mode = "shared"` for a client-scoped sensor DTU type (e.g., Claroty).
Under a `shared` configuration, the DTU instance would be shared across all orgs,
which would cause cross-tenant data leakage: Org A's Claroty device data would be
accessible in Org B's query context because the DTU instance's state store is not
org-partitioned for shared-mode adapters (ADR-008 specifies that shared-mode adapters
do not receive per-org HashMap keying).

This threat must be caught at startup config validation, not at query time.

---

## 2. Decision

### 2.1 DTU Type Classification

DTU types are classified into two categories for the purpose of default mode assignment.
This classification is the canonical source of truth for mode defaults and for the
startup validation rule.

**Category: Security Telemetry (default mode: `client`)**

| DTU Type | Crate | Rationale |
|----------|-------|-----------|
| `claroty` | `prism-dtu-claroty` | Per-org xDome credentials; distinct API endpoints per tenant |
| `armis` | `prism-dtu-armis` | Per-org Armis tenant; AQL queries are org-scoped by account |
| `crowdstrike` | `prism-dtu-crowdstrike` | Per-org Falcon CID; containment/detection stores are org-exclusive |
| `cyberint` | `prism-dtu-cyberint` | Per-org Argos account; alert stores are org-exclusive |
| `demo-server` | `prism-dtu-demo-server` | Test fixture; behaves like a sensor for isolation testing |

Security Telemetry DTUs MUST default to `client` mode. Config validation MUST reject
`mode = "shared"` for any Security Telemetry type. **Wave 3 status: the guard is
unconditional — `allow_shared_override` is NOT IMPLEMENTED in Wave 3. See BC-3.3.001
and §7 OQ-1 (DEFERRED to Wave 4).** Any `allow_shared_override` field in a
`customers/*.toml` file is rejected as an unknown field (`E-CFG-010`) by serde
`deny_unknown_fields`.

**Category: MSSP Coordination (default mode: `shared`)**

| DTU Type | Crate | Rationale |
|----------|-------|-----------|
| `slack` | `prism-dtu-slack` | MSSP-operated webhook; single workspace handles all orgs |
| `pagerduty` | `prism-dtu-pagerduty` | MSSP-operated Events API; single routing key covers all orgs |
| `jira` | `prism-dtu-jira` | MSSP-operated Jira project; all org tickets share one instance |
| `nvd` | `prism-dtu-nvd` | Public CVE database; org-unscoped enrichment lookup |
| `threatintel` | `prism-dtu-threatintel` | Shared indicator-of-compromise lookup; data is org-independent |

MSSP Coordination DTUs MUST default to `shared` mode. Config validation MUST allow
`mode = "client"` for MSSP Coordination types (e.g., a customer who has procured a
dedicated Slack workspace). No warning is required for this override.

### 2.2 Mode Semantics

**`client` mode — per-org exclusive instance**

In client mode, a DTU instance belongs exclusively to a single org. The operator
specifies a `credential_ref` (see ADR-010) that resolves to org-specific API
credentials. The DTU instance's state stores (when present) are keyed with that
org's `OrgId` as the primary key (per ADR-008). The adapter is dispatched only
when the query context carries the matching `OrgId`. The instance does not accept
dispatch requests for any other `OrgId`.

Enforcement: the adapter dispatch layer in `prism-spec-engine` MUST verify that the
`OrgId` in the query plan's org-boundary constraint matches the `OrgId` registered
for the adapter instance before calling any DTU method. A mismatch is a fatal dispatch
error, not a graceful not-found.

**`shared` mode — cross-org single instance**

In shared mode, a DTU instance is MSSP infrastructure. All orgs' queries and action
requests may dispatch to this single instance. The `OrgId` is passed to the adapter
as a payload annotation — it appears in the notification body, ticket label, or audit
record, but it is NOT used as a routing discriminant for the upstream vendor API call.

Shared-mode adapters do not maintain per-org state partitions. Their state stores
(e.g., `SlackState::received_payloads`, `JiraState::issue_registry`) are global to
the instance. Cross-org data privacy in shared-mode adapters relies on the upstream
service's access control model (e.g., different Slack channels per org, different
Jira project keys per org), not on Prism's isolation layer. BC-3.2.004 formalizes
this boundary.

A shared-mode adapter MUST include the `OrgId` in each upstream API payload so that
downstream audit and forensic queries can attribute the action to the correct org.
The `OrgId` MUST NOT be embedded in HTTP headers or URL paths that would be visible
to third-party observers of the upstream service (e.g., not in a Slack webhook URL
path segment).

### 2.3 Default Mode Registry

The default mode registry is a compile-time constant in `prism-core` (where
`OrgRegistry` resides per ADR-006 D-047). Its type is a static
mapping from DTU type string to `DtuMode`:

```rust
/// Canonical default mode for each known DTU type.
/// Security Telemetry → Client. MSSP Coordination → Shared.
pub static DTU_DEFAULT_MODE: &[(&str, DtuMode)] = &[
    ("claroty",       DtuMode::Client),
    ("armis",         DtuMode::Client),
    ("crowdstrike",   DtuMode::Client),
    ("cyberint",      DtuMode::Client),
    ("demo-server",   DtuMode::Client),
    ("slack",         DtuMode::Shared),
    ("pagerduty",     DtuMode::Shared),
    ("jira",          DtuMode::Shared),
    ("nvd",           DtuMode::Shared),
    ("threatintel",   DtuMode::Shared),
];
```

The registry is the authoritative source for:
1. Startup config validation (does the declared `mode` conflict with the Security
   Telemetry classification for this type?)
2. Default-filling the `mode` field when the operator omits it from a `[[dtu]]` block
   (defaulting is optional to implement; explicit `mode` is preferred for readability)

An unknown `type` that does not appear in `DTU_DEFAULT_MODE` is a startup error.
This prevents silently accepting a mistyped DTU type string.

### 2.4 Config Schema (Summary)

Full customer config schema is specified in ADR-010. The mode-relevant portion of a
`[[dtu]]` block is:

```toml
[[dtu]]
type = "claroty"           # must appear in DTU_DEFAULT_MODE
mode = "client"            # "shared" | "client"; required (no silent defaulting in production)
credential_ref = "vault://sensors/claroty/api-key"
# ... additional fields per ADR-010
```

Validation rules enforced at startup (cargo: `prism-config` or inline in startup
pipeline):

1. `type` MUST appear in `DTU_DEFAULT_MODE`. Unknown types → startup error.
2. `mode` MUST be `"shared"` or `"client"`. Any other value → startup error.
3. If `type` is a Security Telemetry type AND `mode = "shared"`: startup error
   (message: "DTU type '{type}' is a Security Telemetry type and MUST be mode=client").
   **Wave 3 note: the error message does NOT mention `allow_shared_override` because
   the escape hatch is not implemented. See §7 OQ-1 (DEFERRED to Wave 4).**
4. **Wave 3 status: NOT IMPLEMENTED — see BC-3.3.001.** `allow_shared_override = true`
   is NOT a recognized field in Wave 3. Any `allow_shared_override` field in a
   `customers/*.toml` produces `E-CFG-010` (unknown field from `deny_unknown_fields`).
   Future intent (Wave 4+): `allow_shared_override = true` will only be valid when
   `mode = "shared"` for a Security Telemetry type; it will disable the guard and
   force an audit-log warning at startup.
5. Mode is read once at startup. It is stored in the sensor spec metadata alongside
   the `OrgId` for the owning org. It is not re-read while the process is running.

### 2.5 Mode Change Semantics and Enforcement (BC-3.2.005)

Mode is deployment-time only. The `mode` field is read during startup when
`customers/*.toml` files are parsed, stored immutably in the in-memory sensor registry,
and never updated by any MCP tool, runtime API, or administrative endpoint.

The enforcement chain is:

1. Startup: `mode` parsed from TOML and stored in `SensorSpec` (or equivalent
   registration struct). The field is typed as `enum DtuMode { Shared, Client }`, not
   as `String`, so its value space is validated at parse time by serde.
2. Runtime: adapter dispatch checks `sensor_spec.mode` before routing. This check is
   not security-critical (the mode is already validated at startup); it exists to
   produce a clear error message if a bug causes incorrect dispatch.
3. No MCP tool exposes `mode` as a writable field. No `POST /dtu/configure` endpoint
   (per ADR-001 and ADR-003) accepts a `mode` parameter. The config endpoint is for
   failure injection and test control only.

BC-3.2.005 (from ADR-006): "The `mode` field in `[[dtu]]` stanzas is read at startup
and immutable for the lifetime of the process. No MCP tool or runtime API may change
a sensor's mode without a restart."

### 2.6 Migration Path for the 11 Existing DTU Crates

All 11 existing DTU crates are unaffected at the binary and HTTP-API level. Mode is
a property of the sensor spec and the dispatch layer, not of the DTU clone itself.
However, three coordination steps are required:

**Step 1: Classify and document.** Each crate's `README.md` or `lib.rs` module doc
gains a one-line annotation indicating its default mode and category:
`// DTU category: Security Telemetry — default mode: client`

**Step 2: State key migration (ADR-008 dependency).** The four client-mode sensor
crates (`claroty`, `armis`, `crowdstrike`, `cyberint`) must migrate their state
HashMap keys from `String` to `(OrgId, String)` as specified in ADR-008. This is
the only code-level change driven by mode classification.

**Step 3: Shared-mode OrgId threading.** The three MSSP Coordination crates
(`slack`, `pagerduty`, `jira`) must accept `OrgId` as a parameter in their payload
construction call sites. Currently `SlackState::capture_payload` at
`prism-dtu-slack/src/state.rs:153` accepts a bare `Value`; in multi-tenant mode,
the caller constructs the payload with the `OrgId` already embedded in the appropriate
field. The state struct itself does not change (no HashMap re-keying required). The
change is in the route handler that constructs the outgoing API payload.

`nvd` and `threatintel` require no threading change because they are read-only
enrichment lookups; org identity is not part of their query or response model.

The `demo-server` and `common` crates coordinate testing infrastructure and are
addressed in the Wave 3 test harness ADR (ADR-009, planned).

---

## Rationale

The three design choices in Section 2 are jointly necessary and individually motivated.

**Why a centralized type registry (DTU_DEFAULT_MODE) rather than per-crate declarations?**
A per-crate declaration (e.g., a `const DEFAULT_MODE: DtuMode` in each crate's `lib.rs`)
would scatter the classification across 11 crates and require reading 11 files to audit
the full isolation posture. A centralized registry in `prism-core` (where `OrgRegistry`
resides per D-047) makes the full classification visible in one place, auditable
in one grep, and enforceable in one validation function. It also prevents a crate author
from silently changing their own classification without updating the authoritative source.
The built-in sensors config-driven philosophy (memory: `feedback_builtin_sensors_config_driven.md`)
means new sensor types are added by config, not by new Rust crates; the registry must
be extensible without modifying existing crate code.

**Why explicit `mode` required in TOML rather than always using the default?**
Silent defaulting in production configuration is a known operational hazard. If an
operator copies a `[[dtu]]` block for a new org without reviewing it, a missing `mode`
field that silently defaults to the correct value provides no signal that the operator
understood the isolation choice. An explicit `mode = "client"` in every sensor block
is a legible declaration of intent that survives copy-paste errors and is auditable in
config review. The validation rule in Section 2.4 rule 2 ensures that any value other
than `"shared"` or `"client"` is rejected immediately, preventing subtle typos from
silently being treated as a default.

**Why deployment-time mode change rather than runtime?**
The per-org state stored in client-mode DTU instances (ADR-008: `HashMap<(OrgId, String), V>`)
is keyed on the org's identity. If an instance switches from `client` to `shared` mode
at runtime, all existing per-org state in the HashMap becomes semantically incorrect
for a shared instance — the state was written under org-scoped keys but will now be
accessed (or not accessed) by a shared-mode adapter that does not use org-keyed
lookups. Clearing the state on mode change would be a silent data loss; preserving it
would be a correctness hazard. The only safe semantics are a restart, which rebuilds
all DTU state from scratch under the new mode. Deployment-time-only is therefore the
correct constraint, not a limitation.

**Why `allow_shared_override` rather than outright prohibition?**
The "quality over speed" principle (memory: `feedback_quality_over_speed.md`) means
best-in-class design, not inflexibility. An MSSP may legitimately need a Security
Telemetry sensor in shared mode for a non-standard deployment scenario (e.g., a
single-tenant evaluation environment where all orgs share one Claroty instance for
demo purposes). Prohibiting this absolutely would force operators to modify Rust source
code. The escape hatch preserves flexibility while making the non-default case
auditable and loudly logged.

---

## 3. Threat Model

### 3.1 Sensor Mode Misconfiguration (BC-3.3.001)

**Threat:** An operator accidentally sets `mode = "shared"` for a client-scoped
sensor like Claroty. All orgs' Claroty queries then route to a single DTU instance
whose state store is not org-partitioned. A query from Org B's context reads device
tags written by Org A's query context.

**Attack vector:** TOML config error combined with absent startup validation.

**Mitigation:** Section 2.4 rule 3 above. The startup validation gate is the first
line of defense. The Security Telemetry classification is centralized in
`DTU_DEFAULT_MODE`; the validator compares the declared type's category against the
declared mode and rejects the process start. **Wave 3 status: NOT IMPLEMENTED — see
BC-3.3.001.** The guard is unconditional in Wave 3; `allow_shared_override` is not
a recognized config field and any attempt to use it produces `E-CFG-010`.

**Residual risk (Wave 4+ only):** If an operator provides `allow_shared_override = true`
(once implemented), the isolation guarantee is downgraded to a process-level concern.
The audit warning is the only runtime signal. BC-3.3.001 must specify that
`allow_shared_override` in production config is a security finding and must appear in
the MSSP's change-approval checklist. In Wave 3, no residual risk from this path
exists because the path is not implemented.

### 3.2 Shared-Mode Payload Leakage (BC-3.2.004, inherited from ADR-006 §3.5)

**Threat:** A Slack DTU delivering a notification on behalf of Org A embeds Org A's
identity in a location visible to Org B's Slack users.

**Mitigation:** The `OrgId` is embedded in the Slack Block Kit message body as a
structured field (e.g., a context block with `org_id: <uuid>`). It is NOT embedded
in the Slack webhook URL token (which is MSSP-controlled, not customer-visible) or in
`X-` HTTP headers. Org B's Slack users cannot observe Org A's notification. The UUID
form of `OrgId` (not the `OrgSlug`) is used in the payload body because it is
opaque to observers (memory: `project_ai_opaque_credentials.md` principle applied to
shared-mode payloads).

### 3.3 AI Context Leakage of Mode Metadata

**Threat:** In the per-analyst MCP deployment model (memory: `project_deployment_model.md`),
the analyst's Claude session sees the results of MCP tool calls. If mode metadata
(e.g., "this DTU is shared across all orgs") is surfaced in tool output, an analyst
could infer the org topology of other customers.

**Mitigation:** MCP tool responses expose `mode` only in administrative/diagnostic
contexts (e.g., a `list_sensors` tool call). Query result rows do not include mode
metadata. The mode field is not included in OCSF-normalized event records returned
to the analyst. BC-3.2.004 must specify that shared-mode routing metadata MUST NOT
appear in analyst-facing query results.

---

## 4. Alternatives Considered

| Option | Description | Decision |
|--------|-------------|----------|
| **Hardcode mode per type in the adapter type system** | Enum variants `SharedAdapter` and `ClientAdapter`; no TOML field | Rejected: inconsistent with the built-in sensors config-driven philosophy (memory: `feedback_builtin_sensors_config_driven.md`). An operator cannot override the default for a non-standard deployment without modifying Rust code. |
| **Single isolation model (all client-mode)** | All DTUs are per-org instances; shared MSSP tools get stub-per-org with identical credentials | Rejected: creating eleven per-org Slack instances sharing the same webhook token is semantically incorrect and wasteful. The shared/client distinction accurately reflects the operational reality. |
| **Runtime mode change via MCP tool** | Allow mode change without restart via administrative API | Rejected: introduces a window where the in-memory state and the on-disk config diverge. The per-tenant state stored in the DTU instance (ADR-008) is partitioned based on mode. Changing mode at runtime would leave stale per-org state in the store from when the instance was client-mode, creating a data integrity hazard. Deployment-time only removes this hazard entirely. |
| **Mode auto-detection from credential type** | Infer `shared` if `credential_ref` points to an MSSP-scoped credential, `client` if org-scoped | Rejected: makes the validation logic depend on credential store implementation details that are outside the spec-engine's scope. Explicit `mode` in TOML is unambiguous and operator-auditable. |
| **Separate TOML stanza for shared vs client** | `[[shared_dtu]]` and `[[client_dtu]]` tables instead of `[[dtu]] mode = ...` | Rejected: increases TOML schema surface and requires two separate parsing paths. A single `[[dtu]]` table with a required `mode` field is more ergonomic and consistent with the existing sensor TOML patterns. |

---

## 5. Consequences

### Positive

- The Security Telemetry / MSSP Coordination classification is formally specified
  and centralized in one registry constant. Any new DTU type must explicitly declare
  its default mode, preventing silent omission.
- Startup validation catches the highest-impact misconfiguration (sensor mode set to
  shared) before the process accepts any requests.
- The `allow_shared_override` escape hatch satisfies the "quality over speed"
  principle (memory: `feedback_quality_over_speed.md`): non-standard deployments are
  possible, but they require an explicit acknowledgment that carries a logged warning.
- Mode metadata is static after startup, making formal verification feasible: a Kani
  proof harness can treat `DtuMode` as a constant for the lifetime of the process
  and verify isolation invariants accordingly.
- No changes to the DTU clone HTTP API layer (no new endpoints, no changed schemas).
  All 11 existing clone servers remain valid behavioral clones for their current
  integration tests.

### Negative

- Operators must declare `mode` explicitly in every `[[dtu]]` block. (Default-filling
  from the registry is possible but not required; explicit declaration is preferred for
  auditability.)
- The `allow_shared_override` mechanism requires implementation and testing of a
  non-obvious escape hatch path. If poorly documented, operators may use it as a
  workaround for config errors rather than fixing the underlying classification issue.
- Shared-mode OrgId threading (Step 3 in Section 2.6) modifies the route handler
  call sites in `slack`, `pagerduty`, and `jira` without changing the HTTP API. These
  changes are internal to the crate but require test updates for the payload shape.

---

## 6. Behavioral Contracts Scoped by This ADR

| BC ID | Title | Postcondition summary |
|-------|-------|-----------------------|
| BC-3.2.004 | Shared-mode adapters pass OrgId as payload annotation only | A shared-mode adapter MUST include `OrgId` in the upstream API payload body. It MUST NOT use `OrgId` as an HTTP routing discriminant (URL path or header). |
| BC-3.2.005 | Mode is deployment-time only | (Inherited from ADR-006.) The `mode` field is read at startup and immutable for the process lifetime. No runtime API changes mode. |
| BC-3.3.001 | Startup rejects Security Telemetry type with shared mode | If `type` is a Security Telemetry type and `mode = "shared"`, the process MUST NOT start and MUST emit a diagnostic error naming the offending `[[dtu]]` block. **Wave 3: guard is unconditional; `allow_shared_override` is NOT IMPLEMENTED (see §7 OQ-1 DEFERRED).** |

---

## 7. Open Questions for Next Dispatch

1. **`allow_shared_override` field: implement or defer?** **RESOLVED: DEFERRED to
   Wave 4.** Wave 3 ST guard is unconditional; `allow_shared_override` is NOT
   IMPLEMENTED. Any `allow_shared_override` field in `customers/*.toml` is rejected
   as an unknown field (`E-CFG-010` via serde `deny_unknown_fields`). No Wave 3 story
   requires a Security Telemetry type in shared mode. Wave 4 may implement the escape
   hatch when at least one concrete use case is identified. BC-3.3.001 reflects this
   resolution. Rationale: "Wave 3 ST guard is unconditional; escape hatch deferred
   until at least one Wave 4 use case requires it."

2. **Mode field in `SensorSpec` vs separate `DtuInstanceSpec`?** The current
   `SensorSpec` (`prism-sensors/src/adapter.rs:38`) has a `client_id: String` field
   (to be migrated to `org_id: OrgId` per ADR-006 Step 4). Should `mode: DtuMode`
   be added to `SensorSpec`, or should a new `DtuInstanceSpec` struct wrap
   `SensorSpec` with the multi-tenant metadata? The choice affects the migration
   scope of ADR-006 Step 4.

3. **`demo-server` classification as Security Telemetry?** The `demo-server` crate
   (`prism-dtu-demo-server`) is a test harness that instantiates all 10 real DTU
   clones. Its own "mode" as a top-level DTU type is conceptually `client` (because
   it participates in per-org isolation tests), but it is not a production sensor
   type. Should it appear in `DTU_DEFAULT_MODE` at all, or only in test configuration?

4. **Enrichment types (nvd, threatintel) in `shared` mode: scope of OrgId threading?**
   NVD and ThreatIntel are read-only lookups with no per-org state. The OrgId
   threading requirement in Section 2.6 Step 3 does not apply to them (there is no
   outgoing payload). However, enrichment query results may be included in audit
   records that carry `OrgId`. Confirm whether `nvd` and `threatintel` route handlers
   need to accept an `OrgId` parameter for audit purposes, or whether audit attribution
   is handled at the query-engine layer upstream of the adapter call.

---

## 8. ADR Chain — Related Documents

- **ADR-006** (antecedent): Establishes `OrgId`/`OrgSlug`/`OrgRegistry` and introduces
  the `mode` field concept. ADR-007 extends Section 2.4 of ADR-006.
- **ADR-008** (consequent): Specifies the HashMap key migration from `String` to
  `(OrgId, String)` for client-mode sensor DTUs. ADR-008 depends on this ADR's
  Security Telemetry classification to know which crates require re-keying.
- **ADR-010** (consequent): Specifies the full customer config schema including
  the `[[dtu]]` block fields. ADR-010 depends on this ADR's `mode` field validation
  rules and the `allow_shared_override` flag definition.
- **ADR-009** (planned): Multi-tenant test harness. Will reference the Security
  Telemetry / MSSP Coordination classification when constructing per-org DTU instance
  maps for integration tests.

---

## 9. Source / Origin

- **PO decision:** D-042 — configurable shared/client mode, deployment-time only.
  Recorded in `.factory/STATE.md`, Wave 3 kickoff 2026-04-27.
- **Code as-built — BehavioralClone trait:**
  `crates/prism-dtu-common/src/clone.rs:36` — `BehavioralClone` trait; no mode
  field currently (mode is a dispatch-layer concept, not a clone-level concept).
- **Code as-built — StubConfig:**
  `crates/prism-dtu-common/src/config.rs:5` — `StubConfig`; does not currently
  carry mode. Mode will be added to the sensor registration struct, not `StubConfig`.
- **Code as-built — Slack payload capture:**
  `crates/prism-dtu-slack/src/state.rs:153` — `capture_payload(payload: Value)`;
  the call site that will receive `org_id` as a construction-time parameter in
  shared-mode OrgId threading.
- **Code as-built — PagerDuty incident registry:**
  `crates/prism-dtu-pagerduty/src/state.rs:91` — `incident_registry: Mutex<HashMap<String, IncidentRecord>>`
  with `String` dedup_key; not re-keyed to `(OrgId, String)` because PagerDuty is
  shared-mode by default (MSSP-global dedup_key space).
- **Code as-built — Jira issue registry:**
  `crates/prism-dtu-jira/src/state.rs:90` — `issue_registry: Mutex<HashMap<String, IssueRecord>>`
  with `String` issue key; not re-keyed for same reason.

---

## Decision Refinements (2026-04-27)

The following questions surfaced during BC authoring (Phase 3.A) and were resolved by the orchestrator on 2026-04-27. Each refinement is recorded here for historical traceability and is binding for Wave 3 implementation.

### D-049 — NVD/ThreatIntel DTUs accept OrgId optionally

**Question:** Do enrichment DTUs (nvd, threatintel) that operate in `shared` mode need to accept `OrgId` at all? They have no per-org state and no outgoing payload that needs annotation.

**Resolution:** NVD and ThreatIntel DTUs accept `OrgId` optionally — not for routing or state keying, but for audit attribution. The `OrgId` is available as an optional parameter in their route handler signatures and is passed through to the audit record emitted for each enrichment lookup. It is never used as a routing discriminant or storage key.

**Rationale:** Enrichment lookups that occur during a query scoped to `OrgId(A)` should produce audit records attributed to `OrgId(A)`. Without optional `OrgId` threading, enrichment audit records would have no org attribution, making it impossible to reconstruct the full audit trail for a given org's query session. "Optional" here means the route handler accepts `Option<OrgId>` — `None` is valid for MSSP-initiated lookups not scoped to a specific org (e.g., background CVE refresh jobs).

**Affected BCs:** BC-3.2.004

### D-051 — `demo-server` exclusion from customer-configurable DTU types

**Question:** Should `demo-server` appear in `DTU_DEFAULT_MODE` alongside production sensor types, making it customer-configurable? Or should it be excluded?

**Resolution:** `demo-server` exclusion from customer-configurable DTU types is implemented by registry pre-population and absence check — NOT by an explicit denylist. `demo-server` appears in `DTU_DEFAULT_MODE` (so the registry knows its default mode is `client`), but the production config validator checks that no `customers/*.toml` file declares `type = "demo-server"` in a `[[dtu]]` block. The check is: "if this type is absent from the production-allowed set, reject it." The production-allowed set is implicitly all types in `DTU_DEFAULT_MODE` minus test-only types identified by a `test_only = true` annotation in the registry entry.

**Rationale:** An explicit denylist (e.g., `DENIED_PRODUCTION_TYPES = ["demo-server"]`) is a parallel data structure that must be kept in sync with `DTU_DEFAULT_MODE`. If a new test-only type is added to `DTU_DEFAULT_MODE` without adding it to the denylist, it becomes customer-configurable by accident. The absence-check approach — annotating registry entries as `test_only` and checking that flag in the production validator — keeps the classification co-located with the registry entry, making omission impossible. This is the same "co-locate policy with data" principle that drove the centralized registry decision.

**Affected BCs:** BC-3.3.001

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.4 | 2026-04-27 | product-owner | M-001 fix: §2.1 stale "unless the operator also provides explicit allow_shared_override = true" dangling text removed — contradicted the C-2 Wave 3 NOT IMPLEMENTED framing. M-002 fix: §2.3 and Rationale "prism-orgs (or wherever OrgRegistry resides)" replaced with "prism-core (where OrgRegistry resides per D-047)" — D-047 locked OrgRegistry in prism-core. |
| 0.3 | 2026-04-27 | product-owner | C-2 sync: §2.1 updated with Wave 3 unconditional guard note; §2.4 rule 3 error message de-references allow_shared_override; §2.4 rule 4 marked NOT IMPLEMENTED in Wave 3; §3.1 mitigation updated to Wave 3 unconditional guard; §6 BC-3.3.001 row updated; §7 OQ-1 locked as DEFERRED to Wave 4. CAP-040 added to anchored capabilities. |
| 0.2 | 2026-04-27 | architect | Decision Refinements: D-049 (NVD/ThreatIntel optional OrgId for audit attribution), D-051 (demo-server exclusion via registry annotation + absence check, not denylist) |
| 0.1 | 2026-04-27 | architect | Initial draft — per-type default registry, mode semantics, validation rules, migration path; extends ADR-006 §2.4 |
