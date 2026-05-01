---
document_type: adr
adr_id: ADR-008
title: "DTU State Segregation — `HashMap<(OrgId, String), V>` Keying Pattern, Per-Tenant Lock Granularity, and Reset Semantics"
status: ACCEPTED
date: 2026-05-01
wave: 3
phase: 3.A
version: "0.13"
authors: [architect]
related_decisions: [D-041, D-042, D-045, D-048, D-049]
related_adrs: [ADR-006, ADR-007, ADR-009, ADR-010, ADR-011]
anchored_capabilities: [CAP-001, CAP-004]
related_bcs_planned: [BC-3.2.001, BC-3.2.002, BC-3.2.003]
subsystems_affected: [SS-01, SS-03, SS-05, SS-21]
supersedes: null
superseded_by: null
traces_to: specs/architecture/ARCH-INDEX.md
inputs:
  - .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
  - .factory/specs/architecture/decisions/ADR-007-configurable-dtu-mode.md
  - crates/prism-dtu-claroty/src/state.rs
  - crates/prism-dtu-armis/src/state.rs
  - crates/prism-dtu-crowdstrike/src/state.rs
  - crates/prism-dtu-cyberint/src/state.rs
  - crates/prism-dtu-slack/src/state.rs
  - crates/prism-dtu-pagerduty/src/state.rs
  - crates/prism-dtu-jira/src/state.rs
  - crates/prism-dtu-nvd/src/state.rs
  - crates/prism-dtu-threatintel/src/state.rs
  - .factory/STATE.md (D-041, D-042, D-045)
---

# ADR-008: DTU State Segregation — `HashMap<(OrgId, String), V>` Keying Pattern, Per-Tenant Lock Granularity, and Reset Semantics

## Status

ACCEPTED 2026-04-28; implementation merged through Wave 3 closure (PRs #73–#112). Wave 3 integration gate findings tracked in `cycles/wave-3-multi-tenant/`.

---

## 1. Context

### 1.1 The Current Single-Tenant HashMap Pattern

All four Security Telemetry DTU crates maintain stateful in-memory stores that are
currently keyed by a bare `String` (the vendor-assigned device identifier or resource
identifier). The stores exist to support write-then-read behavioral contracts: an
analyst action that modifies device state on one API call can observe that modification
in a subsequent read call within the same test session.

The exact current state field declarations are:

- `prism-dtu-claroty/src/state.rs:24` — `tag_store: Mutex<HashMap<String, HashSet<String>>>` —
  maps `device_uid` to the set of tag keys assigned to that device. Populated by
  `add_tag` (line 85) using `device_id.to_string()` as the key.

- `prism-dtu-armis/src/state.rs:72` — `tag_store: Mutex<HashMap<String, HashSet<String>>>` —
  same semantic: maps `device_id` to tag key set. Populated by `add_tag` (line 216)
  using `device_id.to_owned()` as the key.

- `prism-dtu-crowdstrike/src/state.rs:86` — `containment_store: Mutex<HashMap<String, ContainmentStatus>>` —
  maps `device_id` to `ContainmentStatus`. Additionally:
  `prism-dtu-crowdstrike/src/state.rs:88` — `detection_status_store: Mutex<HashMap<String, String>>` —
  maps `detection_id` to status string.

- `prism-dtu-cyberint/src/state.rs:52` — `alert_store: Mutex<HashMap<String, AlertStatus>>` —
  maps `alert_id` to `AlertStatus`. Additionally:
  `prism-dtu-cyberint/src/state.rs:56` — `session_store: Mutex<HashSet<String>>` —
  set of valid session token UUIDs.

In a single-org deployment these `String` keys are unambiguous: there is only one
organization's devices, so `device_id = "abc123"` can only mean one thing. In a
multi-org deployment, two customers may have devices with identical vendor-assigned IDs
(this is common for network device identifiers, internal asset IDs, and CrowdStrike
device IDs which are CID-scoped but not globally unique). A write from Org A's query
context under key `"abc123"` would collide with a read from Org B's query context
under the same key.

### 1.2 MSSP Coordination DTUs Do Not Require Re-keying

As established in ADR-007 Section 2.1, the three MSSP Coordination DTUs operate in
`shared` mode by default. Their state stores are semantically global to the MSSP
instance:

- `prism-dtu-pagerduty/src/state.rs:91` — `incident_registry: Mutex<HashMap<String, IncidentRecord>>` —
  keyed by PagerDuty dedup_key, which is an MSSP-generated identifier (not org-scoped).

- `prism-dtu-jira/src/state.rs:90` — `issue_registry: Mutex<HashMap<String, IssueRecord>>` —
  keyed by Jira issue key (e.g., `"PROJ-1000"`), which is MSSP-scoped.

- `prism-dtu-slack/src/state.rs:28` — `received_payloads: Mutex<Vec<Value>>` —
  an ordered list, not keyed by any tenant identifier.

These stores are intentionally shared. The `OrgId` appears in the payload content
(BC-3.2.004) but not in the storage key. No re-keying is required or desirable.

The enrichment DTUs (`nvd`, `threatintel`) maintain no per-org mutable state; their
stores are either immutable fixture registries or global rate-limit counters. No
re-keying is required.

The `prism-dtu-demo-server` and `prism-dtu-common` crates contain no per-org state
stores of their own (they orchestrate the other clones).

### 1.3 Why Formal Verification Requires Typed Keys

ADR-006 Section 2 explained that `HashMap<String, V>` keyed stores cannot support
formal verification of cross-tenant isolation: Kani model checking and proptest
property-based testing cannot meaningfully reason about whether the `String` key
in a lookup is the "correct" org-scoped key without being given a proof harness that
encodes the intended isolation invariant. With `HashMap<(OrgId, String), V>`, the
isolation invariant becomes:

> A call carrying `OrgId(A)` and resource key `k` can only access the entry at
> `(OrgId(A), k)`. It cannot produce the tuple `(OrgId(B), k)` without explicitly
> constructing `OrgId(B)`, which requires possessing `OrgId(B)` — which is not
> available in a call context that only has `OrgId(A)`.

This invariant is structural and compile-time enforced: the key type `(OrgId, String)`
means that constructing an out-of-band lookup requires constructing the wrong `OrgId`,
which is a different type from a `String` and cannot be produced by accident from
a `String` key.

The mutation testing implication (TD-DTU-MUTATE-COVERAGE-001 from D-046 housekeeping
triage): a mutation that flips the `OrgId` component of the key — replacing
`(org_id, resource_id)` with `(other_org_id, resource_id)` — is a correctness-critical
mutation that tests must kill. Under `String` keying, this class of mutation is
untestable without external fixtures that guarantee device ID collision. Under
`(OrgId, String)` keying, proptest can generate adversarial org pairs and assert that
cross-org lookups return `None`.

---

## 2. Decision

### 2.1 Universal Re-keying Rule for Client-Mode DTU State

Every `HashMap<String, V>` field in a client-mode Security Telemetry DTU state struct
is changed to `HashMap<(OrgId, String), V>`. The transformation is mechanical and
universal: no field is exempt.

The four affected crate-level transformations are:

**`prism-dtu-claroty/src/state.rs`:**

```rust
// Before
pub tag_store: Mutex<HashMap<String, HashSet<String>>>,

// After
pub tag_store: Mutex<HashMap<(OrgId, String), HashSet<String>>>,
```

All call sites in `add_tag`, `remove_tag`, and `get_tags` change their signature from
`fn ...(device_id: &str, ...)` to `fn ...(org_id: OrgId, device_id: &str, ...)`,
constructing the key as `(org_id, device_id.to_string())`.

**`prism-dtu-armis/src/state.rs`:**

```rust
// Before (line 72)
pub tag_store: Mutex<HashMap<String, HashSet<String>>>,

// After
pub tag_store: Mutex<HashMap<(OrgId, String), HashSet<String>>>,
```

Same call site transformation as Claroty. Additionally, `device_registry` (line 51)
and `devices_ordered` (line 55) are immutable fixture registries loaded at construction
time and are NOT re-keyed — they are pre-populated fixture data, not per-org write
targets.

**`prism-dtu-crowdstrike/src/state.rs`:**

```rust
// Before (lines 86, 88)
pub containment_store: Mutex<HashMap<String, ContainmentStatus>>,
pub detection_status_store: Mutex<HashMap<String, String>>,

// After
pub containment_store: Mutex<HashMap<(OrgId, String), ContainmentStatus>>,
pub detection_status_store: Mutex<HashMap<(OrgId, String), String>>,
```

The `session_registry: Mutex<LruCache<String, SessionData>>` (line 90) is a two-step
pagination cache keyed by `X-DTU-Session-Id` header value. Session IDs are generated
by the query engine and are already globally unique (UUID v7). They do not require
re-keying, but BC-3.2.003 must verify that session IDs from one org's context cannot
be used to retrieve another org's session data. The enforcement mechanism is that
session IDs are constructed by the org-scoped query planner and carry the `OrgId`
in their UUIDv7 payload; a session ID from Org A's context contains Org A's temporal
prefix and cannot collide with Org B's session IDs in the same time window.

**`prism-dtu-cyberint/src/state.rs`:**

```rust
// Before (lines 52, 56)
pub alert_store: Mutex<HashMap<String, AlertStatus>>,
pub session_store: Mutex<HashSet<String>>,

// After
pub alert_store: Mutex<HashMap<(OrgId, String), AlertStatus>>,
pub session_store: Mutex<HashSet<(OrgId, String)>>,
```

`alert_store` maps `(org_id, alert_id)` to `AlertStatus`. `session_store` tracks
valid session tokens; re-keyed to `(org_id, token)` so that a session token issued
to Org A cannot be used by a request arriving in Org B's context.

### 2.2 Lookup Contract

The authoritative public interface for per-tenant state access:

```rust
impl ClarotyState {
    /// Look up the tag set for a device within a specific org's context.
    /// Returns an empty set if the (org_id, device_id) pair has no tags.
    /// Cross-tenant lookup is structurally prevented: the caller must possess
    /// the correct OrgId to construct the lookup key.
    pub fn get_tags(&self, org_id: OrgId, device_id: &str) -> HashSet<String> {
        let store = self.tag_store.lock().expect("tag_store poisoned");
        store.get(&(org_id, device_id.to_owned())).cloned().unwrap_or_default()
    }
}
```

The general contract (BC-3.2.001): a `fetch` or `write` call carrying `OrgId(A)` MUST
NOT read or modify entries keyed under `OrgId(B)` for any `B ≠ A`. This is structurally
enforced by the tuple key: the only way to read `(OrgId(B), resource_id)` is to pass
`OrgId(B)` explicitly, which is impossible in a call context that only has `OrgId(A)`.

### 2.3 Per-Tenant Lock Granularity

The current implementation uses a single `Mutex` guard over the entire `HashMap`
for each state field (e.g., `tag_store: Mutex<HashMap<...>>`). This is a global lock
over all tenants sharing the same DTU instance.

Under the multi-tenant model, all client-mode DTU instances are per-org (one instance
per org per DTU type). A client-mode instance is only ever accessed from one org's
query context. Therefore, the per-tenant lock granularity question does not arise for
client-mode instances: there is exactly one tenant per instance, so the existing
whole-HashMap Mutex is already per-tenant in practice.

For shared-mode DTU instances (where multiple orgs' requests dispatch to the same
instance), finer-grained locking would be necessary for write-heavy paths. However,
shared-mode state stores do not contain per-org keyed data (see Section 1.2), so
the contention model is: all orgs' requests contend on the same global Mutex.
PagerDuty incident creation, Jira issue creation, and Slack payload capture are all
infrequent relative to read operations. The whole-HashMap Mutex is acceptable for
Wave 3.

Decision: **retain the existing whole-HashMap Mutex pattern** for Wave 3. The
per-tenant partitioned lock (e.g., `HashMap<OrgId, Mutex<HashMap<String, V>>>`) is
deferred to Wave 4 if performance profiling identifies contention on the shared-mode
stores. This is not a correctness concern for Wave 3 because:

1. Client-mode instances have exactly one tenant; no lock contention between orgs.
2. Shared-mode instances use global locks that are acceptable for MSSP coordination
   action rates (ticket creation, Slack notification) which are order-of-magnitude
   lower frequency than sensor telemetry queries.

### 2.4 Reset Semantics: Per-Tenant and Full Reset

The existing `reset()` method on each state struct clears all stores unconditionally.
Under multi-tenant operation, two reset variants are required:

**`reset_for(org_id: OrgId)`** — clears only entries keyed under `(org_id, *)`.
Used by test harness when resetting one org's state between test cases without
affecting other orgs in a concurrent multi-org test.

```rust
pub fn reset_for(&self, org_id: OrgId) {
    let mut store = self.tag_store.lock().expect("tag_store poisoned");
    store.retain(|(id, _), _| *id != org_id);
}
```

**`reset_all()`** — clears all entries for all orgs. Used by test harness in full
teardown. Renames the existing `reset()` method to `reset_all()` in all four
client-mode state structs for clarity; the existing `BehavioralClone::reset()` trait
method continues to call `state.reset_all()` internally.

For shared-mode state structs (`SlackState`, `PagerDutyState`, `JiraState`), the
existing `reset()` method is sufficient and is not renamed. There is no per-tenant
partition to reset individually.

### 2.5 Default OrgId for Single-Tenant Compatibility

Existing single-tenant tests and integration test fixtures pass device IDs without an
`OrgId`. To preserve these tests during the migration window (ADR-006 Steps 1-7),
each state struct gains a `DEFAULT_ORG_ID` test constant:

```rust
#[cfg(test)]
pub const DEFAULT_ORG_ID: OrgId = OrgId(uuid::uuid!("00000000-0000-7000-8000-000000000001"));
```

This is a deterministic UUID v7 (time bits = 0, version = 7, variant = 8) that is
valid per the UUID v7 spec but has zero time component, making it visually distinct
from production UUIDs. It is `#[cfg(test)]` only: production code paths MUST NOT use
it. Any call site that attempts to use `DEFAULT_ORG_ID` in non-test code will fail to
compile (the constant is test-gated).

During the migration window, existing tests call `state.add_tag(DEFAULT_ORG_ID, device_id, tag)`
rather than the old `state.add_tag(device_id, tag)`. This is a mechanical find-replace.
After migration is complete, `DEFAULT_ORG_ID` is removed from production code paths
and retained only in unit test fixtures.

### 2.6 Scope by Crate

| Crate | Re-key required | Stores affected | Notes |
|-------|-----------------|-----------------|-------|
| `prism-dtu-claroty` | YES | `tag_store` | 3 call sites: `add_tag`, `remove_tag`, `get_tags` |
| `prism-dtu-armis` | YES | `tag_store` | 3 call sites; `device_registry` + `devices_ordered` NOT re-keyed |
| `prism-dtu-crowdstrike` | YES | `containment_store`, `detection_status_store` | `session_registry` NOT re-keyed (see §2.1) |
| `prism-dtu-cyberint` | YES | `alert_store`, `session_store` | Session token isolation via (org_id, token) tuple |
| `prism-dtu-slack` | NO | n/a | Shared-mode; OrgId in payload body, not key |
| `prism-dtu-pagerduty` | NO | n/a | Shared-mode; OrgId in incident metadata field |
| `prism-dtu-jira` | NO | n/a | Shared-mode; OrgId in issue fields payload |
| `prism-dtu-nvd` | NO | n/a | Enrichment; `cve_registry` is read-only fixture |
| `prism-dtu-threatintel` | NO | n/a | Enrichment; `fixture_registry` is read-only |
| `prism-dtu-demo-server` | NO | n/a | Harness; delegates to other clones |
| `prism-dtu-common` | NO | n/a | Trait + config; no per-org state |

---

## Rationale

**Why `(OrgId, String)` tuple rather than a nested `HashMap<OrgId, HashMap<String, V>>`?**

A nested `HashMap<OrgId, HashMap<String, V>>` would partition state by org first,
then by resource ID within the org. This structure has two disadvantages:

1. It requires two lock acquisitions (outer map, then inner map) or a single outer
   lock that holds while the inner map is accessed — functionally equivalent to the
   tuple approach in terms of lock granularity.
2. It makes the `reset_for(org_id)` operation a simple `outer.remove(org_id)` rather
   than a `retain` filter, which is marginally faster. However, it complicates
   iteration over all entries (e.g., to find all devices across orgs) and makes the
   lookup API asymmetric with the single-tenant pattern.

The `(OrgId, String)` flat key is simpler, hashes well (both components are small
and well-distributed), and makes the lookup API uniform: all callers construct the
full composite key at the call site, which keeps the isolation contract visible at
every call site rather than hidden behind an outer-map lookup.

**Why retain the whole-HashMap Mutex rather than switching to per-tenant RwLock?**

The per-tenant lock (`HashMap<OrgId, Mutex<HashMap<String, V>>>`) would eliminate
contention between tenants in shared-mode instances. However, client-mode instances
(the only instances that have per-org keyed stores) have exactly one tenant by
construction (ADR-007 Section 2.2). There is no contention to eliminate. The
optimization is meaningless for the correctness target and adds implementation
complexity that would need to be verified separately. Wave 4 can introduce per-tenant
locks if profiling reveals contention in shared-mode instances.

**Why `DEFAULT_ORG_ID` is `#[cfg(test)]` only?**

The single greatest risk in the migration is a production code path that accidentally
uses a default or sentinel `OrgId` as a fallback when the real `OrgId` is unavailable.
Making `DEFAULT_ORG_ID` compile only under `#[cfg(test)]` makes this class of bug
impossible: any production code that tries to use it fails to compile. This follows
the "best in class" principle (memory: `feedback_quality_over_speed.md`): design
out the failure mode rather than document it.

**Why re-key `session_store` in Cyberint from `HashSet<String>` to `HashSet<(OrgId, String)>`?**

Session tokens issued by `CyberintState::register_session` at
`prism-dtu-cyberint/src/state.rs:214` are UUID v4 strings generated by
`uuid::Uuid::new_v4().to_string()`. UUID v4 tokens are probabilistically unique
across orgs (collision probability negligible), so a token from Org A would not
coincidentally be valid for Org B in practice. However, the formal verification
requirement (BC-3.2.001) mandates structural isolation, not probabilistic isolation.
A proptest harness generating adversarial tokens (e.g., the same UUID for two orgs)
must be able to verify that `is_valid_session` in Org B's context returns `false`
for a token issued in Org A's context. This requires that the session store key
carry the `OrgId`.

---

## 3. Threat Model

### 3.1 Cross-Tenant State Collision via Device ID Overlap (BC-3.2.001)

**Threat:** Org A has a device with vendor-assigned ID `"host-001"`. Org B also has
a device with vendor-assigned ID `"host-001"`. Under `HashMap<String, V>` keying,
a write from Org A's context (e.g., `tag_store.insert("host-001", {"malware-detected"})`)
and a read from Org B's context (`tag_store.get("host-001")`) resolve to the same
HashMap entry.

**Attack vector:** MSSP query engine dispatches multiple orgs' requests to a
hypothetical shared client-mode DTU instance (which would be a misconfiguration
caught by ADR-007 validation). Even under correct configuration, a bug in the
dispatch layer that passes the wrong `OrgId` to a client-mode adapter is mitigated
by the `(OrgId, String)` key requiring the correct `OrgId` to construct the lookup.

**Mitigation:** `HashMap<(OrgId, String), V>` keying. A read carrying `OrgId(A)` can
only access `(OrgId(A), "host-001")`. It cannot access `(OrgId(B), "host-001")`
without explicitly constructing `OrgId(B)`, which requires the dispatch layer to have
been given `OrgId(B)` — a different code path that would only occur if the query plan
itself was wrong.

### 3.2 Cross-Tenant Session Token Reuse (BC-3.2.003)

**Threat:** An analyst query for Org A obtains a Cyberint session token. A query
execution bug passes that token (or a semantically equivalent token) to a request
dispatched in Org B's context. Org B's context gains authenticated access to Cyberint
APIs.

**Mitigation:** `session_store` re-keyed to `HashSet<(OrgId, String)>`. The
`is_valid_session` check takes `(org_id, token)` as its input. A token registered
under `(OrgId_A, token_uuid)` is not found when looked up under `(OrgId_B, token_uuid)`
even if the token string is identical.

### 3.3 Mutation That Flips OrgId in Lookup (TD-DTU-MUTATE-COVERAGE-001)

**Threat:** A code mutation replaces `(org_id, resource_id)` with
`(other_org_id, resource_id)` in a state lookup call. The mutation passes existing
tests because tests only verify single-org behavior and do not check that a different
org's context does not produce the same result.

**Mitigation:** The `(OrgId, String)` keying pattern makes this class of mutation
testable. A proptest harness generates two distinct `OrgId` values and a shared
resource ID, writes state under `OrgId(A)`, then verifies that a lookup under `OrgId(B)`
returns `None`. This kills any mutation that weakens the org component of the key.
BC-3.2.001 requires this property; the proptest strategy is specified in the
verification-properties for each affected crate.

---

## 4. Migration Strategy

Migration is coordinated with ADR-006 Steps 6 and 7, which specify the order of
workspace-wide migration. The per-crate steps within Step 6 are:

**Step 6a — Claroty state re-keying.**
Change `tag_store` type. Update `add_tag`, `remove_tag`, `get_tags` signatures.
Add `DEFAULT_ORG_ID` test constant. Update all existing unit tests to pass
`DEFAULT_ORG_ID`. Add multi-org proptest: two orgs, same device ID, verify isolation.
Gate: `cargo test -p prism-dtu-claroty` green.

**Step 6b — Armis state re-keying.**
Change `tag_store` type. Update `add_tag`, `remove_tag`, `tags_for` signatures.
Leave `device_registry`, `devices_ordered`, `activity_fixture`, `alert_fixture` unchanged.
Add multi-org proptest. Gate: `cargo test -p prism-dtu-armis` green.

**Step 6c — CrowdStrike state re-keying.**
Change `containment_store` and `detection_status_store` types. Update all call sites in
route handlers that currently call e.g.
`state.containment_store.lock()?.insert(device_id, status)` to
`state.containment_store.lock()?.insert((org_id, device_id), status)`.
Document `session_registry` non-re-keying rationale in a code comment.
Add multi-org proptest. Gate: `cargo test -p prism-dtu-crowdstrike` green.

**Step 6d — Cyberint state re-keying.**
Change `alert_store` to `HashMap<(OrgId, String), AlertStatus>`.
Change `session_store` to `HashSet<(OrgId, String)>`.
Update `register_session`, `is_valid_session`, `reset` call sites.
The `build_alert_store` helper at `state.rs:109` must accept an `OrgId` parameter
to construct per-org fixture keys: `(org_id, alert.alert_id.clone())`.
Add multi-org proptest. Gate: `cargo test -p prism-dtu-cyberint` green.

**Step 6e — Add `reset_for` to all four client-mode state structs.**
Each struct gains `reset_for(org_id: OrgId)`. Existing `reset()` renamed to
`reset_all()` with a one-wave compatibility shim: `pub fn reset(&self) { self.reset_all(); }`.
Gate: `cargo test` workspace green.

---

## 5. Alternatives Considered

| Option | Description | Decision |
|--------|-------------|----------|
| **Nested HashMap keying: `HashMap<OrgId, HashMap<String, V>>`** | Two-level structure; outer key is `OrgId`, inner key is resource ID | Rejected: two lock acquisitions per access; complicates iteration; no correctness advantage over flat tuple key. See Rationale section. |
| **`Arc<OrgState>` per-org structs instead of keyed maps** | Each org gets its own `ClarotyState` instance held in `HashMap<OrgId, Arc<ClarotyState>>` | Rejected: this is equivalent to client-mode DTU instances (ADR-007 §2.2), which is already the chosen architecture. The state struct holds a single org's state by definition in client-mode. No need for a HashMap of state structs. |
| **Per-tenant `RwLock` for read-heavy stores** | `HashMap<(OrgId, String), V>` protected by `RwLock` instead of `Mutex` | Deferred: correct choice for write-heavy shared stores; not needed for Wave 3 where client-mode instances are single-tenant and shared-mode instance access rates are low. |
| **Skip session_store re-keying for Cyberint** | Session tokens are UUID v4; probabilistically unique across orgs | Rejected: formal verification requires structural isolation, not probabilistic. A proptest that generates the same token for two orgs must be able to kill the mutation. UUID v4 probability argument is insufficient for a BC-3.2 isolation property. |
| **Preserve `reset()` name without alias** | Hard-rename `reset()` to `reset_all()` immediately, break all callers | Deferred: the deprecation alias strategy (mirroring ADR-006 Step 2's `TenantId` alias) reduces blast radius during migration. One wave of aliases, then remove in the same wave that drops `TenantId`. |

---

## 6. Consequences

### Positive

- Cross-tenant state collision via device ID overlap becomes structurally impossible.
  No runtime check, guard, or assertion is required to enforce BC-3.2.001; the type
  system enforces it.
- Mutation testing for the `(OrgId, String)` key pattern (TD-DTU-MUTATE-COVERAGE-001)
  can now target the org component of the key specifically. The 115 missed DTU clone
  mutations identified in the Wave 3 housekeeping triage (D-046) include this class.
- `reset_for(org_id)` enables concurrent multi-org test scenarios where one org's
  state is reset between sub-cases without disturbing other orgs' state.
- The migration is mechanical: the transformation from `HashMap<String, V>` to
  `HashMap<(OrgId, String), V>` is a find-replace plus call-site signature update.
  No algorithmic changes are required.

### Negative

- HashMap key size increases from `String` (~24 bytes heap-allocated) to
  `(OrgId, String)` (`OrgId` is 16 bytes on-stack + `String` 24 bytes heap) — a
  16-byte overhead per entry. At typical Wave 3 scale (hundreds to low thousands
  of devices per org per DTU instance), this is negligible. The `(OrgId, String)`
  hash is still well-distributed because both components are high-entropy.
- All existing call sites in the four client-mode crates must be updated to pass
  `OrgId`. This is a non-trivial change surface (~10-20 call sites per crate) that
  requires test updates. The `DEFAULT_ORG_ID` compatibility shim limits the blast
  radius during migration.
- `build_alert_store` in `CyberintState` (line 109) must be updated to accept
  `OrgId` as a parameter, which changes the construction-time API for that crate.
  This affects any test that constructs `CyberintState` directly.

---

## 7. Behavioral Contracts Scoped by This ADR

The following BCs were authored during Phase 3.A; see BC-INDEX for canonical metadata.

| BC ID | Title | Postcondition summary |
|-------|-------|-----------------------|
| BC-3.2.001 | Per-Org Sensor Data Isolation via Composite HashMap Key | A fetch or write call carrying `OrgId(A)` MUST NOT read or modify DTU state entries keyed under `OrgId(B)` for any `B ≠ A`. |
| BC-3.2.002 | Per-Org Credential Isolation via OrgId-Keyed Namespace | (Shared with ADR-006.) Credential lookup under `OrgId(A)` MUST NOT return credentials stored under `OrgId(B)`. This ADR provides the state-level isolation that supports this property. |
| BC-3.2.003 | Per-Org Session Token Isolation via (OrgId, token) Composite Key | A session token registered under `OrgId(A)` MUST NOT be accepted as valid by `is_valid_session` called with `OrgId(B)` for any `B ≠ A`. |

---

## 8. Open Questions for Next Dispatch

1. **Session registry (CrowdStrike) re-keying deferred — confirm scope.** The
   `session_registry: Mutex<LruCache<String, SessionData>>` at
   `prism-dtu-crowdstrike/src/state.rs:90` is not re-keyed (Section 2.1 rationale).
   The spec-writer should verify that the CrowdStrike pagination session ID generation
   (in the query engine, not in the DTU clone) is already org-scoped before authoring
   BC-3.2.003 for the CrowdStrike adapter.

2. **`build_alert_store` in CyberintState: OrgId parameter at construction time.**
   If `CyberintState` is constructed with fixture data before the `OrgId` is known
   (e.g., in a test harness that loads fixtures first, then assigns org IDs), the
   `build_alert_store` API change may require a two-phase construction pattern.
   Confirm with story-writer that the harness (ADR-009) can supply `OrgId` at
   construction time.

3. **`reset_for` in shared-mode structs: needed or not?** Section 2.4 states that
   shared-mode structs do not require `reset_for`. Confirm with test harness ADR
   (ADR-009) whether any multi-tenant test scenario requires selectively resetting
   shared-mode state for one org without affecting others. If yes, shared-mode structs
   may need a `reset_for` variant that filters entries by OrgId in their payload
   content (not key), which is more complex.

4. **Atomicity of `reset_for` under concurrent writes.** The `reset_for(org_id)` uses
   `HashMap::retain`, which holds the lock for the duration of the retain. If a
   concurrent write for the same `org_id` is in flight while `retain` executes, the
   write may be dropped. For test harness use (where `reset_for` is called between
   test cases with no concurrent activity), this is acceptable. Confirm that the
   test harness guarantees no in-flight requests during `reset_for` calls.

---

## 9. ADR Chain — Related Documents

- **ADR-006** (antecedent): Establishes `OrgId` type and mandates the migration of
  DTU state HashMap keys. ADR-008 specifies the exact migration pattern.
- **ADR-007** (antecedent): Establishes the Security Telemetry / MSSP Coordination
  classification that determines which crates require re-keying.
- **ADR-009** (consequent): Multi-Tenant Data Generator. The generator produces
  org-tagged fixture data keyed by `(OrgId, seed, archetype, scale)`, directly
  consuming the per-org keying pattern established by this ADR.
- **ADR-010** (consequent): Customer config schema. The `data.archetype`, `data.scale`,
  and `data.seed` fields in `[[dtu]]` blocks are used by the multi-tenant data generator
  (ADR-009) which produces fixture data keyed by `(OrgId, seed, archetype, scale)`.
  The per-org keying of the data generator output is consistent with the per-org keying
  established by this ADR.
- **ADR-011** (consequent): Network isolation. Per-org DTU isolation at the network
  level (D-044) operates on client-mode instances that are already state-isolated by
  this ADR. The network layer adds defense-in-depth above the state-level isolation.

---

## 10. Source / Origin

- **PO decisions:** D-041 (OrgId/OrgSlug identity), D-042 (configurable mode),
  D-045 (spec-first phasing) — recorded in `.factory/STATE.md`, Wave 3 kickoff 2026-04-27.
- **TD-DTU-MUTATE-COVERAGE-001:** Housekeeping item from D-046 triage — 115 missed
  DTU clone mutations; per-org keying makes this class testable.
- **Code as-built — Claroty:**
  `crates/prism-dtu-claroty/src/state.rs:24` — `tag_store: Mutex<HashMap<String, HashSet<String>>>` (migration target).
  `crates/prism-dtu-claroty/src/state.rs:85` — `add_tag` call site.
- **Code as-built — Armis:**
  `crates/prism-dtu-armis/src/state.rs:72` — `tag_store: Mutex<HashMap<String, HashSet<String>>>` (migration target).
  `crates/prism-dtu-armis/src/state.rs:216` — `add_tag` call site.
- **Code as-built — CrowdStrike:**
  `crates/prism-dtu-crowdstrike/src/state.rs:86` — `containment_store: Mutex<HashMap<String, ContainmentStatus>>` (migration target).
  `crates/prism-dtu-crowdstrike/src/state.rs:88` — `detection_status_store: Mutex<HashMap<String, String>>` (migration target).
  `crates/prism-dtu-crowdstrike/src/state.rs:90` — `session_registry: Mutex<LruCache<String, SessionData>>` (NOT migrated).
- **Code as-built — Cyberint:**
  `crates/prism-dtu-cyberint/src/state.rs:52` — `alert_store: Mutex<HashMap<String, AlertStatus>>` (migration target).
  `crates/prism-dtu-cyberint/src/state.rs:56` — `session_store: Mutex<HashSet<String>>` (migration target).
  `crates/prism-dtu-cyberint/src/state.rs:109` — `build_alert_store` (API change required).
- **Code as-built — Slack, PagerDuty, Jira, NVD, ThreatIntel:**
  Not re-keyed. State stores confirmed as non-per-org (shared or enrichment pattern).

---

## Decision Refinements (2026-04-27)

The following questions surfaced during BC authoring (Phase 3.A) and were resolved by the orchestrator on 2026-04-27. Each refinement is recorded here for historical traceability and is binding for Wave 3 implementation.

### D-048 — CrowdStrike session_registry is org-scoped at the query-engine layer

**Question:** The `session_registry: Mutex<LruCache<String, SessionData>>` in `prism-dtu-crowdstrike/src/state.rs:90` was explicitly not re-keyed in Section 2.1 (rationale: session IDs are globally unique UUID v7). How is org-scoping enforced for CrowdStrike pagination sessions?

**Resolution:** CrowdStrike session_registry is org-scoped at the query-engine layer — pagination session IDs are scoped per `org_id` at the query-engine layer. The `prism-dtu-crowdstrike` clone's `session_registry` remains keyed by session ID string (unchanged), but the query engine that generates session IDs ensures each session ID is constructed with the calling `OrgId` embedded in the UUID v7 time field (org-temporal uniqueness). The BC-3.2.003 verification property for CrowdStrike confirms that a session ID generated in Org A's context resolves only to Org A's session data.

**Rationale:** Re-keying `session_registry` to `(OrgId, session_id)` would require the clone to know the `OrgId` at session-lookup time — but the clone's session registry is keyed by the session ID token, which arrives in the `X-DTU-Session-Id` HTTP header without any org context. The query engine is the right enforcement point: it generates session IDs that are org-namespaced by construction, so the clone never sees a session ID that could accidentally match across orgs. This preserves the clone's stateless-from-org-perspective design while satisfying the isolation requirement at the correct layer.

**Affected BCs:** BC-3.2.003

### D-049 — NVD/ThreatIntel optional OrgId threading (cross-reference from ADR-007)

**Question:** Do enrichment DTUs need `OrgId` threading in their state layer?

**Resolution:** NVD and ThreatIntel have no per-org mutable state (their stores are immutable fixture registries or global rate-limit counters). No state re-keying is required. `OrgId` is accepted optionally at the route handler level for audit attribution only (see ADR-007 D-049 refinement). This ADR's Section 2.6 scope table remains accurate: `prism-dtu-nvd` and `prism-dtu-threatintel` have `Re-key required: NO`.

**Rationale:** Consistent with the ADR-007 resolution. The state layer and the route handler layer have different responsibilities; audit attribution is a route handler concern, not a state keying concern.

**Affected BCs:** BC-3.2.001 (enrichment DTUs confirmed out of scope for re-keying)

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.13 | 2026-05-01 | state-manager | ACCEPTED→IMPLEMENTED status promoted post-Wave-3 closure. §2 Status block updated from PROPOSED to ACCEPTED per D-183. Wave 3 integration gate findings tracked in cycles/wave-3-multi-tenant/. |
| 0.12 | 2026-04-28 | product-owner | Phase 3.A APPROVED by user — status: PROPOSED → ACCEPTED. D-136. Wave 3 implementation cleared to begin per D-045 (Spec-First Discipline) post-approval. |
| 0.11 | 2026-04-27 | product-owner | M-24-002 (Pass 24): `related_adrs` frontmatter corrected — ADR-010 added (body §9 listed ADR-010 as consequent but frontmatter array was missing it). Body §9 updated to add ADR-009 entry (data generator is a related consequent consuming per-org keying). |
| 0.10 | 2026-04-27 | product-owner | m-19-001 (pass-19-remediation): `related_adrs` extended with ADR-009. §9 ADR chain body: "ADR per D-043" → "ADR-009". |
| 0.9 | 2026-04-27 | product-owner | pass-14-remediation: SS-21 added to `subsystems_affected` — the composite key `(OrgId, String)` pattern introduced by this ADR depends on the `OrgId` type exported from prism-core (SS-21). |
| 0.8 | 2026-04-27 | product-owner | M-003 (pass-13-remediation): Status block updated — "BCs to be authored in subsequent Phase 3.A spec-writer dispatch" → "BCs authored at v0.3+ during Phase 3.A; see BC-INDEX." §7 preamble updated to match. |
| 0.7 | 2026-04-27 | product-owner | m-001/m-002 (pass-10-remediation): §7 BC table titles updated to Title Case matching BC-INDEX H1 source-of-truth: "Per-org sensor data isolation"→"Per-Org Sensor Data Isolation via Composite HashMap Key"; "Per-org credential isolation"→"Per-Org Credential Isolation via OrgId-Keyed Namespace"; "Per-org session token isolation"→"Per-Org Session Token Isolation via (OrgId, token) Composite Key". |
| 0.6 | 2026-04-27 | product-owner | M-003 (pass-6-remediation): Frontmatter `title:` corrected to Title Case to match H1 heading (POL 7 H1 source-of-truth). |
| 0.5 | 2026-04-27 | product-owner | M-003 (pass-4-remediation): SS-01 added to subsystems_affected (DTU state stores live in prism-dtu-* crates which are SS-01; the (OrgId,String) keying pattern touches SS-01 state layer). |
| 0.4 | 2026-04-27 | product-owner | m-002 (Pass 3): `anchored_capabilities` corrected [CAP-038] → [CAP-001, CAP-004]. BC-3.2.001/2/3 (the BCs this ADR governs) anchor to CAP-001 and CAP-004, not CAP-038; the triangle was broken. Minimal blast-radius fix per finding recommendation. |
| 0.3 | 2026-04-27 | product-owner | m-001 fix: added `anchored_capabilities: [CAP-038]` to frontmatter (per adversary Pass 2 minor finding). |
| 0.2 | 2026-04-27 | architect | Decision Refinements: D-048 (CrowdStrike session_registry org-scoped at query-engine layer, not clone state), D-049 cross-ref (NVD/ThreatIntel no state re-keying required) |
| 0.1 | 2026-04-27 | architect | Initial draft — (OrgId, String) keying pattern, per-tenant locks deferred, reset semantics, full crate scope table |
