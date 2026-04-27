---
document_type: adr
adr_id: ADR-006
title: "Multi-tenant DTU Topology — OrgId/OrgSlug Identity, OrgRegistry, Configurable Shared/Client Mode"
status: PROPOSED
date: 2026-04-27
wave: 3
phase: 3.A
version: "0.5"
authors: [architect]
related_decisions: [D-041, D-042, D-044, D-045, D-047, D-050]
related_adrs: [ADR-007, ADR-008, ADR-010, ADR-011]
related_bcs_planned: [BC-3.1.001, BC-3.1.002, BC-3.1.003, BC-3.1.004, BC-3.2.001, BC-3.2.002, BC-3.2.003, BC-3.2.004, BC-3.2.005]
anchored_capabilities: [CAP-038, CAP-040]
subsystems_affected: [SS-03, SS-05, SS-06, SS-01]
supersedes: null
superseded_by: null
traces_to: specs/architecture/ARCH-INDEX.md
inputs:
  - crates/prism-core/src/tenant.rs
  - crates/prism-core/src/ids.rs
  - crates/prism-credentials/src/namespace.rs
  - crates/prism-credentials/src/trait_.rs
  - crates/prism-sensors/src/event_buffer.rs
  - crates/prism-spec-engine/src/pipeline.rs
  - .factory/STATE.md (D-041, D-042, D-044, D-045)
---

# ADR-006: Multi-tenant DTU Topology — OrgId/OrgSlug Identity, OrgRegistry, Configurable Shared/Client Mode

## Status

PROPOSED — decisions D-041, D-042, D-044, D-045 recorded. BCs to be authored in subsequent
Phase 3.A spec-writer dispatch. Implementation BLOCKED until Phase 3.A converges (D-045).

---

## 1. Context

### 1.1 MSSP Deployment Model

Prism runs as a per-analyst MCP server process. One analyst session simultaneously manages
N customer organizations, each with independent security tooling. The deployment model
(memory: `project_deployment_model.md`) has two distinct categories of downstream sensor:

- **Client-scoped sensors** — Claroty, Armis, CrowdStrike, Cyberint. Each is specific
  to one customer organization; credentials, endpoints, and data are never shared across
  organizations. An Armis beacon for Org A must not dispatch to Org B's Armis instance.

- **MSSP-internal tools** — Slack, PagerDuty, Jira. These are shared MSSP infrastructure.
  A Jira instance handles tickets for all managed organizations, distinguished by metadata
  (e.g., a `client:` label or project-key prefix), not by separate instances.

All built-in sensors ship as TOML spec files that the spec engine loads at startup (memory:
`feedback_builtin_sensors_config_driven.md`). The existing `TenantId` newtype
(`crates/prism-core/src/tenant.rs:47`) holds the customer identifier as `Arc<str>`, but it
conflates two distinct axes: a stable canonical identity and a friendly display name. Those
axes must be separated.

### 1.2 What the Current Model Cannot Verify

The single `TenantId` type (`^[a-zA-Z0-9_-]{1,64}$`, regex at `tenant.rs:25`) is used
as both the credential namespace key
(`crates/prism-credentials/src/namespace.rs:20`) and the RocksDB event-buffer key
prefix (`crates/prism-sensors/src/event_buffer.rs:46`). The credential store
(`crates/prism-credentials/src/trait_.rs:27-66`) and the event buffer key
(`event_buffer.rs:82`) both take a `&str` that originates from `TenantId::as_str()`.

Problems with this model at MSSP scale:

1. **Rename instability.** If "acme-corp" rebrands to "acme-na", the operator renames the
   slug in the config file. Any audit records, RocksDB keys, and keyring entries written
   under the old slug become orphaned. Historical queries under the new slug return no
   results for events that pre-date the rename.

2. **Cross-tenant isolation is un-provable.** Both `HashMap<String, V>` keyed DTU state
   stores (e.g., `ClarotyState::tag_store` at `prism-dtu-claroty/src/state.rs:24`,
   `CrowdstrikeState::containment_store` at `prism-dtu-crowdstrike/src/state.rs:86`) and
   the string `client_id` field in `SensorSpec` (`prism-sensors/src/adapter.rs:38`) use
   opaque `String`. There is no type-system enforcement that the org-scoped key used to
   write to these stores is the same org-scoped key used to read them in a query context.
   Kani/proptest cannot verify isolation properties against `String`-keyed maps; they can
   verify them against `OrgId(Uuid)`.

3. **Slug collision in multi-org registration.** Two customer TOML configs that accidentally
   declare the same tenant string are silently last-write-wins. No registration-time conflict
   detection exists.

4. **Credentials are not AI-opaque by type.** The memory item `project_ai_opaque_credentials.md`
   requires that credentials never transit the AI context window — they must be opaque
   references. The existing keyring namespace
   (`crates/prism-credentials/src/namespace.rs:20`) formats as
   `"{tenant}/{sensor}/{name}"`, where `{tenant}` is the slug string. Under the
   UUID-stable identity model, this namespace becomes
   `"{org_id_uuid}/{sensor}/{name}"`, which is both stable across renames and
   structurally opaque to an AI observer that sees only `OrgId` values in trace output.

These four problems collectively motivate the identity model in D-041 and are the
necessary precondition for formal verification of cross-tenant isolation properties
in the BC-3.2 family.

---

## 2. Decision

Three integrated decisions compose the multi-tenant topology.

### 2.1 Identity Model

**`OrgId(Uuid)` — canonical internal identity.**

Added to `crates/prism-core/src/ids.rs` using the existing `uuid_v7_newtype!` macro
(`ids.rs:10-42`). UUID v7 is mandatory; UUID v4 is prohibited per Architecture Compliance
Rules already enforced by the macro. `OrgId` is the key used for all internal routing:
credential namespace lookup, RocksDB event-buffer key prefix, audit record primary key,
DTU state HashMap keys, and query-plan org-boundary enforcement.

```rust
uuid_v7_newtype!(
    /// Canonical identifier for a managed customer organization.
    /// Stable across OrgSlug renames. Internal routing axis only.
    OrgId
);
```

Referencing the macro documented at `ids.rs:10`: "All IDs use UUID v7 (time-ordered) so
that RocksDB iteration is monotonically increasing by creation time."

**`OrgSlug(Arc<str>)` — analyst-facing friendly identifier.**

The current `TenantId` type (`tenant.rs:47`) is renamed to `OrgSlug` with semantics
preserved. The embedded validity state model (`TenantIdInner::Valid/Invalid` at
`tenant.rs:34-37`) is retained unchanged. The regex constant
`TENANT_ID_PATTERN` (`tenant.rs:25`) is renamed to `ORG_SLUG_PATTERN`.

Slug length: the current maximum is 64 characters. This ADR proposes tightening to 32
characters (`^[a-zA-Z0-9_-]{1,32}$`). Rationale: analyst-facing surfaces (PrismQL
`WHERE org = 'acme-corp'`, MCP tool inputs, `customers/*.toml` filenames) do not benefit
from 64-character slugs; 32 provides sufficient namespace for realistic org names while
reducing the surface for log-injection payloads. This is an open question (see Section 8).

`OrgSlug` is used at: PrismQL `WHERE org = '...'` predicates, MCP tool input parameters,
customer TOML config filenames, and human-readable audit log rendering.

**Distinct types make the canonical/friendly axis explicit at every boundary.** A function
signature `fn route(org: OrgId, ...)` documents that it takes the canonical form.
A function signature `fn display(slug: &OrgSlug)` documents that it takes the display
form. Mixed-axis bugs become compile-time errors.

### 2.2 OrgRegistry — Translation Layer

`OrgRegistry` is a new struct in `prism-core` (or a dedicated `prism-orgs` crate;
see Section 8 for the open question on crate placement). It provides the bijective
mapping between `OrgSlug` and `OrgId`.

```rust
pub struct OrgRegistry { /* internal: BiMap<OrgSlug, OrgId> */ }

impl OrgRegistry {
    /// Translate an analyst-supplied slug to the canonical OrgId.
    /// Returns None if the slug is not registered.
    pub fn resolve(&self, slug: &OrgSlug) -> Option<OrgId>;

    /// Translate an OrgId to its current display slug (for rendering).
    /// Returns None if the OrgId is not registered (should not occur in practice).
    pub fn slug_for(&self, id: OrgId) -> Option<OrgSlug>;

    /// Register a new (slug, id) pair.
    /// Returns Err if the slug or id is already registered to a different counterpart.
    /// BC-3.1.003 (bijectivity) and BC-3.1.004 (duplicate rejection) enforced here.
    pub fn register(&self, slug: OrgSlug, id: OrgId) -> Result<(), RegistrationError>;
}
```

Initial population: `OrgRegistry` is populated at startup from `customers/*.toml` config
files. Each customer TOML declares its `org_id` (UUID v7) and `org_slug` (kebab-case string).
`OrgRegistry::register` is called once per file; any duplicate slug or UUID causes a startup
error and Prism does not start.

Runtime registration via MCP tool is deferred to Wave 4 unless a Wave 3 story requires it.

**Persistence:** `OrgRegistry` is rebuilt from `customers/*.toml` at every startup (see
Section 8 — open question on RocksDB-backed persistence). This keeps the implementation
simple and avoids a consistency problem between file-system config and database state.

### 2.3 Translation Flow

The flow from analyst input to internal identity and back to audit output:

```
Analyst:    PrismQL "SELECT * FROM devices WHERE org = 'acme-corp'"
                  ↓ (PQL parser calls OrgRegistry::resolve("acme-corp"))
            OrgId(01975e4e-9f00-7abc-...) flows through:
              - query plan construction (org boundary constraint carries OrgId)
              - DataSource dispatch (adapter selected by OrgId, not slug)
              - credential lookup: namespace_key(org_id, sensor, name)
              - EventBuffer write/read: scope_prefix(sensor_id, table, org_id)
              - DTU state HashMap key: (OrgId, device_id)
                  ↓ (audit pipeline persists BOTH fields)
Audit row:  { org_id: OrgId(uuid), org_slug: OrgSlug("acme-corp"), ... }
                  ↓ (human-readable rendering calls OrgRegistry::slug_for)
MCP output: "acme-corp" displayed to analyst
```

The slug appears in: PrismQL input, MCP output rendering, and audit records (for
forensic readability). The OrgId appears in: all internal routing, storage keys,
credential namespaces, and audit records (for rename stability). Audit records
persist both fields (BC-3.1.002) so that neither rename stability nor forensic
readability requires a join at query time.

### 2.4 Configurable Shared/Client Mode

Per D-042. Each DTU instance declares a `mode` in the customer TOML config:

```toml
# customers/acme-corp.toml

[[dtu]]
type = "claroty"
mode = "client"        # one instance per org; default for sensor types

[[dtu]]
type = "crowdstrike"
mode = "client"        # one instance per org

[[dtu]]
type = "slack"
mode = "shared"        # MSSP-internal; single instance across orgs

[[dtu]]
type = "pagerduty"
mode = "shared"        # MSSP-internal; single instance

[[dtu]]
type = "jira"
mode = "shared"        # MSSP-internal; single instance
```

Mode semantics:

- `client`: the DTU instance belongs exclusively to this org. Credentials, endpoints,
  and data are fully isolated. The adapter is dispatched only when the query context
  carries the matching `OrgId`.

- `shared`: the DTU instance is shared MSSP infrastructure. All orgs may generate events
  that route to this adapter (e.g., a Slack notification about Org A's incident and
  Org B's incident both route to the same Slack DTU). The `OrgId` is passed as payload
  metadata (e.g., a message field), not as a routing discriminant. Cross-org data leakage
  via the shared adapter is not a concern because the adapter writes out, not reads in:
  it delivers a notification, not a data query result.

Mode is deployment-time configuration only. There is no runtime API to change mode
without an operator editing the config file and restarting the process. This is
enforced by BC-3.2.005. The `mode` field is read once at startup during `OrgRegistry`
population and stored in the sensor spec metadata; it is not re-read while the process
is running.

The default mode per DTU type is defined in the sensor type registry (to be specified
in ADR-007): sensor types classified as "security telemetry" default to `client`;
sensor types classified as "MSSP coordination" default to `shared`.

---

## Rationale

The three integrated decisions in Section 2 are jointly necessary: no single piece is
sufficient alone.

**OrgId is required for isolation verifiability.** The core constraint driving D-041 is
not ergonomics — it is formal verification. Kani and proptest can model isolation invariants
over `HashMap<(OrgId, String), V>` (a typed composite key) but cannot do so meaningfully
over `HashMap<String, V>` where the string is an opaque combination of tenant and device
identifier. The BC-3.2 family (cross-tenant data isolation, credential isolation) requires
a property-based proof foundation; that foundation requires typed keys. UUID v7 satisfies
the additional constraint that RocksDB key iteration is monotonically ordered by creation
time — the existing architecture (`ids.rs:4-5`) already prohibits UUID v4 for this reason.

**OrgSlug is required for analyst ergonomics and config legibility.** The per-analyst MCP
deployment model (memory: `project_deployment_model.md`) means analysts interact with the
system via PrismQL and MCP tool inputs in their Claude session. Requiring analysts to type
UUID strings in `WHERE org = '01975e4e-...'` clauses would make the query surface
hostile and error-prone. The built-in sensor TOML specs (memory:
`feedback_builtin_sensors_config_driven.md`) also use human-readable identifiers for
customer organization names; changing those to UUIDs would break operator ergonomics
during TOML authoring. OrgSlug preserves all existing string-identifier ergonomics while
making the display/canonical distinction explicit at the type level.

**OrgRegistry is required to keep the mapping a single source of truth.** Without a
registry, each component that needs to translate between slug and UUID must either maintain
its own mapping (divergence risk) or perform filesystem access at query time (latency and
availability risk). A single bijective in-memory registry, populated at startup and
validated for conflicts before the process accepts any requests, satisfies BC-3.1.001
through BC-3.1.004 with O(1) lookup and no I/O in the hot path.

**Configurable mode is required because not all DTU types have the same isolation
semantics.** Claroty, Armis, CrowdStrike, and Cyberint are per-customer security telemetry
sources — each managed organization has its own vendor account, credentials, and data.
Slack, PagerDuty, and Jira are MSSP-internal coordination tools — the MSSP has one
account shared across all managed organizations. A single isolation model cannot cover
both cases. The `mode` field in TOML externalizes this distinction into operator-visible
configuration (consistent with the config-driven sensor philosophy) rather than hardcoding
it in the adapter type system. ADR-007 will specify the per-type default registry.

---

## 3. Threat Model

### 3.1 Cross-Tenant Data Leakage at Adapter Layer (BC-3.2.001)

**Threat:** A query bound to `OrgId` A returns rows that belong to `OrgId` B.

**Attack vector:** `HashMap<String, V>` in DTU state stores (e.g.,
`ClarotyState::tag_store` at `prism-dtu-claroty/src/state.rs:24`,
`CrowdstrikeState::containment_store` at `prism-dtu-crowdstrike/src/state.rs:86`)
are currently keyed by `String` (device ID). In a multi-tenant DTU server, two orgs
may have devices with the same vendor-assigned device ID. A write from Org A's
query context and a read from Org B's query context would resolve to the same
`HashMap` entry. Similarly, a query plan that loses the `OrgId` constraint during
pushdown would dispatch to the wrong adapter instance.

**Mitigation:** DTU state `HashMap` keys are changed from `String` to `(OrgId, String)`.
BC-3.2.001 formalizes this as an isolation postcondition: a `fetch` or `write` call
carrying `OrgId(A)` MUST NOT read or modify entries keyed under `OrgId(B)`. The
type system enforces this structurally — there is no way to look up `(OrgId_B, device_id)`
in a call that only has access to `OrgId_A`.

The query plan carries `OrgId` as a non-nullable constraint from parse time to adapter
dispatch time. Loss of the `OrgId` constraint in the plan is a build error
(the field is required in the plan's struct type).

### 3.2 Cross-Tenant Credential Reachability (BC-3.2.002)

**Threat:** Bearer tokens for Org A's sensors are reachable when the query context is
scoped to Org B.

**Attack vector:** `namespace_key` at `prism-credentials/src/namespace.rs:20` currently
formats as `"{tenant_id_str}/{sensor}/{name}"`. If credential lookup uses the slug
string and a slug is renamed (A's old slug = B's new slug), the lookup would return
A's credentials when queried with B's new slug.

**Mitigation:** After this ADR, `namespace_key` takes `&OrgId` (UUID string
representation) rather than `&TenantId` (slug string). The namespace format becomes
`"{org_id_uuid}/{sensor}/{name}"` — e.g., `"01975e4e-9f00-7abc-8def-/crowdstrike/api_key"`.
UUID v7 values are unique across orgs and stable across renames. Slug collision (two
orgs claiming the same slug at different times) cannot cause credential cross-reach
because the storage key is never the slug.

This satisfies the AI-opaque credentials requirement (memory:
`project_ai_opaque_credentials.md`): the UUID string in the keyring namespace is
meaningless to an LLM observer; the credential value never appears in the query context.

### 3.3 Slug Rename Forensics (BC-3.1.003 / BC-3.1.002)

**Threat:** If Org A renames from "acme-corp" to "acme-na", historical audit records
written under "acme-corp" become orphaned when an analyst queries under "acme-na".

**Mitigation:** Audit records persist `OrgId` as the primary key and `OrgSlug` as a
denormalized display field (BC-3.1.002). When an analyst queries the audit trail for
"acme-na", the audit pipeline resolves "acme-na" to its `OrgId` via `OrgRegistry::resolve`
and queries by UUID. Records written under the old slug are found because they share
the same `OrgId`. The denormalized `org_slug` field in each record shows "acme-corp"
for historical entries and "acme-na" for entries written after the rename — providing
a complete forensic picture without any data migration.

### 3.4 Slug Squatting / Namespace Collision (BC-3.1.004)

**Threat:** Two customer configs declare the same `org_slug`. The second registration
silently overwrites the first, making Org A's slug resolve to Org B's `OrgId`.

**Mitigation:** `OrgRegistry::register` enforces bijectivity (BC-3.1.003).
Attempting to register a slug already bound to a different `OrgId`, or an `OrgId`
already bound to a different slug, returns `RegistrationError::SlugConflict` or
`RegistrationError::IdConflict`. Prism refuses to start until the operator resolves
the conflict in the config files. There is no silent last-write-wins behavior.

### 3.5 Privacy in Shared-Infrastructure DTU (BC-3.2.004)

**Threat:** When the Slack DTU forwards a payload on behalf of Org A, metadata in
the payload or the DTU's routing state reveals Org A's identity to observers of
Org B's Slack channel.

**Mitigation:** The `OrgId` is passed to shared-mode DTU adapters as payload metadata
(e.g., a message body field or a ticket label). It is not embedded in HTTP headers
or URL paths that would be visible to third-party Slack/Jira users. The shared DTU
instance does not maintain per-org routing tables; it routes solely by the `mode = "shared"`
declaration. BC-3.2.004 specifies that shared-mode adapters MUST NOT use the `OrgId`
as a routing discriminant in their upstream API calls, only as a payload annotation.

---

## 4. Migration Strategy

The `TenantId` → `OrgSlug` rename is a workspace-wide refactor touching ~5 crates and
~11 DTU crates. Migration proceeds in the following order, each step scoped to a story
under Epic E-3.1:

**Step 1 — Additive: Add `OrgId` to `prism-core`.**
Add `uuid_v7_newtype!(OrgId)` to `crates/prism-core/src/ids.rs`. Export from `prism_core`
lib. Add `OrgRegistry` struct (empty impl initially) to `prism-core` or `prism-orgs`
(see Section 8). No existing code changes. Zero breakage. Gate: compiles clean.

**Step 2 — Rename: `TenantId` → `OrgSlug` in `prism-core`.**
Rename `tenant.rs` struct, const, and internal enum variants. Add a one-wave deprecation
alias: `pub type TenantId = OrgSlug;` in `prism_core::lib`. This keeps all downstream
crates compiling without change during the migration window. Regex length open question
(32 vs 64) resolved before this step; the constant is updated once. Gate: `cargo build`
workspace clean with deprecation warnings only (no errors).

**Step 3 — Credential boundary migration.**
Update `prism-credentials/src/namespace.rs:20` (`namespace_key`) to accept `&OrgId`
instead of `&TenantId`. Update `prism-credentials/src/trait_.rs:27-66` to carry
`&OrgId` on all async trait methods. Update `prism-credentials/src/keyring.rs` and
`prism-credentials/src/file.rs` implementations. All credential store tests updated.
Gate: `cargo test -p prism-credentials` green.

**Step 4 — Sensor + spec-engine boundary migration.**
Update `SensorSpec.client_id: String` at `prism-sensors/src/adapter.rs:38` to
`org_id: OrgId`. Update `event_buffer.rs:46,82` `scope_prefix` and `event_key` to
accept `OrgId` (serialized as UUID string for the key bytes). Update
`prism-spec-engine/src/pipeline.rs:20` `PipelineContext.client_id: TenantId` to
`org_id: OrgId`. Gate: `cargo test -p prism-sensors -p prism-spec-engine` green.

**Step 5 — Audit-entry shape change.**
Audit entries gain two fields: `org_id: OrgId` and `org_slug: OrgSlug`.
The existing `client_id: String` audit field (BC-2.04 family) is migrated to carry
`OrgId` as the primary identifier; `OrgSlug` is the denormalized display companion.
This is an additive schema change — existing audit serialization gets new fields,
old fields renamed. BC-3.1.002 verification property coverage added.
Gate: `cargo test -p prism-audit` green.

**Step 6 — DTU state HashMap key migration.**
For each client-mode DTU crate (`prism-dtu-{claroty,armis,crowdstrike,cyberint}`),
update `state.rs` `HashMap<String, V>` keys to `HashMap<(OrgId, String), V>`.
Update all insert/lookup call sites. For shared-mode DTU crates
(`prism-dtu-{slack,pagerduty,jira}`), no HashMap keying change is needed
(shared instances have no per-org state partitioning in their store).
Gate: all 11 DTU crate tests green.

**Step 7 — Remove `TenantId` deprecation alias.**
Remove `pub type TenantId = OrgSlug;` from `prism_core`. Remaining compile errors
(if any) are isolated to test fixtures that used `TenantId::new_unchecked`; update
to `OrgSlug::new_unchecked`. Gate: `cargo build` workspace clean with no deprecated-use
warnings.

---

## 5. Alternatives Considered

| Option | Description | Decision |
|--------|-------------|----------|
| **Slug-only (current state)** | Keep `TenantId` as the sole identifier; add migration logic for renames | Rejected: rename instability breaks audit forensics and RocksDB key continuity. Slug collision in multi-org config is silent. |
| **UUID-only, no slug** | Use `OrgId(Uuid)` everywhere including analyst-facing surfaces | Rejected: analyst ergonomics. Humans remember "acme-corp", not "01975e4e-...". PrismQL queries would require UUIDs, which is hostile to interactive use. |
| **UUID v4 instead of v7** | Random UUIDs for OrgId | Prohibited by Architecture Compliance Rules (documented in `ids.rs:4-5`). UUID v4 breaks monotonic RocksDB iteration ordering. |
| **Single combined `Org` type** | One type with both `id: Uuid` and `slug: Arc<str>` fields | Rejected: boundary types `fn foo(org: Org)` hide which axis is canonical. A `fn route(org: Org)` that only uses `org.id` is indistinguishable from one that accidentally uses `org.slug` as a routing key. Two distinct newtypes make the canonical/display axis explicit and compiler-enforced at every boundary. |
| **Runtime slug rename API** | Allow slug rename via MCP tool without restart | Rejected for Wave 3: introduces transient state where the in-memory registry and on-disk config differ. Deferred to Wave 4 if needed; Wave 3 uses config-file edit + restart. |

---

## 6. Consequences

### Positive

- Stable forensic audit trail across org renames. Audit records are queryable by
  `OrgId` regardless of what slug the org has today.
- Formal verification of cross-tenant isolation is now feasible. `HashMap<(OrgId, String), V>`
  with `OrgId` in the key can be modeled in Kani property harnesses; `HashMap<String, V>`
  with an opaque string key cannot.
- Rename-safe credential namespaces. `"{org_id_uuid}/{sensor}/{name}"` is permanently
  stable; no keyring migration is needed when an org renames.
- `OrgRegistry` is the single source of truth for slug↔id translation. There is no
  secondary lookup table, no dual-write, and no risk of the mapping diverging.
- Consistent with AI-opaque credentials architecture: `OrgId` UUIDs in trace output
  are opaque to an LLM context window without carrying any semantic information about
  the organization.
- Built-in sensors continue to ship as TOML specs (memory: `feedback_builtin_sensors_config_driven.md`).
  The `mode` field in `[[dtu]]` stanzas is an additive key to the existing TOML schema.

### Negative

- Two types to maintain: `OrgId` and `OrgSlug`. All new code must handle both;
  existing code must be migrated in Steps 1-7.
- Migration touches ~5 library crates and ~11 DTU crates. Steps 2-7 carry non-trivial
  refactor cost; the deprecation-alias strategy (Step 2) limits blast radius but
  does not eliminate it.
- `OrgRegistry` is new infrastructure. It must be initialized before any component
  that resolves org identity; initialization order in the server startup sequence
  must be specified (BC-3.1.001).
- Shared-mode DTU adapters (`slack`, `pagerduty`, `jira`) require the `OrgId` to be
  available in the call context at payload-construction time, even though they do not
  use it as a routing key. This threads `OrgId` into contexts that previously had no
  org parameter.

### Open: OrgRegistry Persistence

`OrgRegistry` rebuilds from `customers/*.toml` at every startup. If a config file is
deleted between restarts, its org mapping is lost from the registry. The consequence
is that `OrgRegistry::resolve` returns `None` for the affected slug, and queries fail
cleanly rather than silently returning stale data. Recommendation: **rebuild at startup**
for Wave 3 simplicity. RocksDB-backed persistence of the registry becomes a story if
operational experience reveals a need (see Section 8).

---

## 7. Behavioral Contracts Scoped by This ADR

The following BCs are to be authored in subsequent Phase 3.A spec-writer dispatch.
This ADR establishes their scope and one-line postcondition.

| BC ID | Title | Postcondition summary |
|-------|-------|-----------------------|
| BC-3.1.001 | OrgRegistry resolution semantics | `resolve(slug)` returns `Some(OrgId)` iff the slug was registered; `None` otherwise. Resolution is O(1) lookup, not filesystem access. |
| BC-3.1.002 | Audit entry contains both org_id and org_slug | Every emitted audit entry MUST carry both `org_id: OrgId` and `org_slug: OrgSlug` at construction time. Neither field is nullable. |
| BC-3.1.003 | OrgRegistry bijectivity | At any instant, the registry is a bijection: no two slugs map to the same OrgId, no two OrgIds map to the same slug. |
| BC-3.1.004 | OrgRegistry rejects duplicate slugs and UUIDs at registration | `register(slug, id)` returns `Err(RegistrationError)` if `slug` is already bound to a different `OrgId`, or if `id` is already bound to a different `OrgSlug`. |
| BC-3.2.001 | Per-org sensor data isolation | A fetch or write call carrying `OrgId(A)` MUST NOT read or modify DTU state entries keyed under `OrgId(B)` for any `B ≠ A`. |
| BC-3.2.002 | Per-org credential isolation | `CredentialStore::get(org_id_A, sensor, name)` MUST NOT return credentials stored under `namespace_key(org_id_B, ...)` for any `B ≠ A`. |
| BC-3.2.005 | Configurable mode is deployment-time only | The `mode` field in `[[dtu]]` stanzas is read at startup and immutable for the lifetime of the process. No MCP tool or runtime API may change a sensor's mode without a restart. |

---

## 8. Open Questions for Next Dispatch

1. **OrgSlug regex length: 32 vs 64 characters.** Current `TenantId` allows 64. This ADR
   proposes 32 for analyst-surface usability. Constraint to verify: are there any
   existing customer config files or sensor TOML specs that use slugs longer than 32
   characters? If yes, tighten to 48 as a middle ground, or keep 64 with a documentation
   note. Architect or spec-writer should grep `customers/` and built-in sensor TOMLs
   before authoring BC-3.1.001 to determine the actual maximum in use.

2. **OrgRegistry persistence: rebuild-from-config vs RocksDB-backed.** Current proposal:
   rebuild at startup. Revisit if Wave 3 integration testing reveals that losing the
   registry on config-file deletion causes operational problems. RocksDB persistence
   would add `prism-storage` as a dependency of `prism-orgs`, creating a crate dependency
   that should be evaluated against the dependency graph before committing.

3. **OrgRegistry CRUD via MCP tool: is adding/removing orgs a runtime API?** Current
   proposal: no, deployment-time only via `customers/*.toml` edit + restart. If Wave 4
   requires live org onboarding without restart, this becomes a story at that time.

4. **`TenantId` deprecation alias: one-wave vs hard-rename.** Recommendation is
   `pub type TenantId = OrgSlug;` for one wave (Wave 3), removed at Wave 4 start.
   Confirm with story-writer that the deprecation alias is sufficient for the migration
   stories in E-3.1, or whether a hard-rename in a single large PR is preferable.

5. **Crate placement for `OrgRegistry`.** Two options: (a) add to `prism-core` alongside
   `OrgId`/`OrgSlug` — keeps the identity layer in one crate but grows `prism-core`'s
   scope; (b) new `prism-orgs` crate — cleaner separation but adds a crate to the
   workspace. Decision affects the dependency graph and should align with the subsystem
   decomposition (SS-06 Client Configuration is the closest existing subsystem).
   Architect defers to spec-writer feedback or a follow-on ADR if warranted.

---

## 9. ADR Chain — Related Documents

This ADR is the first in the Wave 3 Phase 3.A ADR chain. It is referenced by:

- **ADR-007** (to be drafted): Configurable shared/client mode — detailed per-DTU-type
  mode registry, default mode assignment, mode declaration schema. ADR-007 extends
  Section 2.4 of this ADR.
- **ADR-008** (to be drafted): DTU state segregation — formal specification of the
  `HashMap<(OrgId, String), V>` keying pattern, segment isolation invariants,
  and test harness design for multi-tenant DTU behavioral clones. ADR-008 extends
  Section 3.1 of this ADR.
- **ADR-010** (to be drafted): Convention sweep — workspace-wide naming conventions
  for `OrgId`/`OrgSlug` usage, import aliasing, and documentation standards.
- **ADR-011** (to be drafted): Network isolation in-wave — Docker Compose topology
  for multi-tenant integration tests; how org-scoped DTU instances are isolated at
  the network level.

---

## Source / Origin

- **PO decisions:** D-041 (OrgId/OrgSlug identity model), D-042 (configurable shared/client
  mode), D-044 (network isolation in-wave), D-045 (spec-first phasing is BLOCKING) —
  recorded in `.factory/STATE.md`, Wave 3 kickoff 2026-04-27.
- **Code as-built — existing identity type:**
  `crates/prism-core/src/tenant.rs:47` — `TenantId` newtype (to be renamed `OrgSlug`);
  validation regex `^[a-zA-Z0-9_-]{1,64}$` at line 25.
- **Code as-built — UUID v7 macro:**
  `crates/prism-core/src/ids.rs:10-42` — `uuid_v7_newtype!` macro; `OrgId` will be
  added here following the same pattern as `ScheduleId`, `RuleId`, `CaseId`, `AlertId`.
- **Code as-built — credential namespace:**
  `crates/prism-credentials/src/namespace.rs:20` — `namespace_key(tenant, sensor, name)`
  formats `"{tenant}/{sensor}/{name}"`; this is the primary migration target for
  slug→UUID keying.
- **Code as-built — DTU state stores:**
  `crates/prism-dtu-claroty/src/state.rs:24` (`tag_store: Mutex<HashMap<String, HashSet<String>>>`),
  `crates/prism-dtu-crowdstrike/src/state.rs:86` (`containment_store: Mutex<HashMap<String, ContainmentStatus>>`) —
  these `HashMap<String, V>` stores are the migration targets for `(OrgId, String)` composite keying.
- **Code as-built — event buffer keying:**
  `crates/prism-sensors/src/event_buffer.rs:46` — `scope_prefix(sensor_id, table_name, client_id)`
  uses `client_id: &str`; to be updated to accept `OrgId`.
- **Behavioral contracts:** BC-3.1.001 through BC-3.1.004, BC-3.2.001, BC-3.2.002,
  BC-3.2.005 — scoped by this ADR; to be authored by spec-writer in Phase 3.A.

---

## Decision Refinements (2026-04-27)

The following questions surfaced during BC authoring (Phase 3.A) and were resolved by the orchestrator on 2026-04-27. Each refinement is recorded here for historical traceability and is binding for Wave 3 implementation.

### D-047 — OrgRegistry crate placement

**Question:** Crate placement for `OrgRegistry` — extend `prism-core` (option a) or create a new `prism-orgs` crate (option b)?

**Resolution:** `OrgRegistry` lives in `prism-core`. Option (a) is chosen: extend the existing crate alongside `OrgId` and `OrgSlug`. No new `prism-orgs` crate is introduced.

**Rationale:** Creating a new crate adds workspace surface and dependency graph complexity without a compelling scope justification. `OrgRegistry` is identity infrastructure that belongs alongside the identity types it maps. Keeping it in `prism-core` avoids a circular dependency risk (`prism-orgs` would need `prism-core` for `OrgId`/`OrgSlug`, while `prism-core` would need `prism-orgs` for registry access). The "cleaner separation" argument for `prism-orgs` is outweighed by the ADR-012 housekeeping principle of stable crate count for Wave 3.

**Affected BCs:** BC-3.1.001, BC-3.1.002, BC-3.1.003, BC-3.1.004

### D-050 — OrgRegistry duplicate registration is idempotent for exact same tuple

**Question:** Should `register(slug, id)` called twice with the exact same (slug, id) pair be `Ok` (idempotent) or `Err`?

**Resolution:** Registering the exact same `(slug, id)` tuple a second time returns `Ok` — it is idempotent. Only conflicting tuples (same slug different id, or same id different slug) produce `RegistrationError`.

**Rationale:** Idempotency is essential for reload scenarios where `OrgRegistry` may be repopulated from config without a full process restart (e.g., a config hot-reload path in a future wave). Making re-registration of an identical pair an error would force callers to check existence before registering, adding unnecessary complexity. The bijectivity invariant (BC-3.1.003) is preserved: idempotent re-registration does not create duplicate entries; it is a no-op. Only a conflicting registration (which would break bijectivity) is an error. This updates the EC-003 edge case in BC-3.1.004: the answer is `Ok`.

**Affected BCs:** BC-3.1.004

### D-080 — ADR↔CAP anchored_capabilities scope convention (narrower-scope rule)

**Question:** Should ADR-006 and ADR-007 `anchored_capabilities` lists be expanded via the D-077 union rule to include all capabilities referenced by their related BCs (CAP-001, CAP-004, CAP-007, CAP-009)?

**Resolution:** No. ADR-006 `anchored_capabilities` lists only its **primary** capabilities: `[CAP-038, CAP-040]`. ADR-007 lists `[CAP-040]`. The ADR↔CAP↔BC traceability triangle is satisfied **transitively** through child ADRs: ADR-008 (per-org state segregation) covers CAP-001/CAP-004 semantics; ADR-010 (customer config schema) covers CAP-009. The D-077 union rule applies to story `bcs:` frontmatter arrays, not to ADR `anchored_capabilities` fields — those are intended to name the domain capability the ADR introduces or primarily governs, not every capability touched by its dependent BCs.

**Rationale:** Applying the union rule to ADR `anchored_capabilities` would create redundant cross-links that are already captured more precisely in the child ADR→BC→CAP chains. The narrower-scope convention keeps each ADR's capability anchors focused on what that ADR owns, while full coverage is provided by the child ADR graph.

**Affected ADRs:** ADR-006, ADR-007

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.5 | 2026-04-27 | product-owner | M-003 (pass-6-remediation): Frontmatter `title:` corrected to Title Case to match H1 heading (POL 7 H1 source-of-truth). |
| 0.4 | 2026-04-27 | product-owner | M-003/D-080 (pass-5-remediation): Documented narrower-scope ADR↔CAP convention — ADR-006 `anchored_capabilities` lists only primary capabilities [CAP-038, CAP-040]; ADR↔CAP↔BC triangle is satisfied transitively via child ADRs (ADR-008 for state segregation, ADR-010 for config schema). Union rule (D-077) is NOT applied to ADR-006/007 anchored_capabilities lists. This avoids redundant cross-linking while preserving full traceability through child ADR chains. Decision recorded as D-080 in ADR-006 and ADR-007. |
| 0.3 | 2026-04-27 | product-owner | C-5 capability anchoring: `anchored_capabilities: [CAP-038, CAP-040]` added to frontmatter. CAP-038 (Multi-Tenant Identity Model) anchors BC-3.1.001, BC-3.1.003, BC-3.1.004. CAP-040 (Multi-Tenant Adapter Dispatch Mode) co-anchors with ADR-007. |
| 0.2 | 2026-04-27 | architect | Decision Refinements: D-047 (OrgRegistry in prism-core, no prism-orgs crate), D-050 (idempotent duplicate registration for exact same tuple) |
| 0.1 | 2026-04-27 | architect | Initial draft — scopes D-041, D-042, D-044, D-045; status PROPOSED |
