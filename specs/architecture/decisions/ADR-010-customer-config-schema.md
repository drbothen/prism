---
document_type: adr
adr_id: ADR-010
title: "Customer config schema — customers/{org_slug}.toml structure, validation rules, loading lifecycle, and schema versioning"
status: PROPOSED
date: 2026-04-27
wave: 3
phase: 3.A
version: "0.5"
authors: [architect]
related_decisions: [D-041, D-042, D-046, D-052, D-053]
related_adrs: [ADR-006, ADR-007, ADR-009]
related_bcs_planned: [BC-3.3.001, BC-3.3.002, BC-3.3.003, BC-3.3.004]
subsystems_affected: [SS-06, SS-03, SS-01]
supersedes: null
superseded_by: null
traces_to: specs/architecture/ARCH-INDEX.md
inputs:
  - .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
  - .factory/specs/architecture/decisions/ADR-007-configurable-dtu-mode.md
  - .factory/specs/architecture/decisions/ADR-008-dtu-state-segregation.md
  - .factory/STATE.md (D-041, D-042, D-046)
---

# ADR-010: Customer Config Schema — `customers/{org_slug}.toml` Structure, Validation Rules, Loading Lifecycle, and Schema Versioning

## Status

PROPOSED — decisions D-041, D-042, D-046 recorded. Specifies the full TOML schema
for customer organization configuration files. BCs to be authored in subsequent
Phase 3.A spec-writer dispatch. Implementation BLOCKED until Phase 3.A converges
(D-045).

---

## 1. Context

### 1.1 The Config-Driven Sensor Philosophy

All built-in sensors in Prism ship as TOML spec files that the spec engine loads at
startup (memory: `feedback_builtin_sensors_config_driven.md`). This is the
"eating our own dog food" principle: CrowdStrike, Cyberint, Claroty, and Armis behave
as customer-provided sensor specs, not as special-cased code paths. The customer
configuration schema extends this principle to the multi-tenant layer: each managed
customer organization is described in a TOML file that the Prism process loads at
startup to register the org and instantiate its DTU instances.

### 1.2 What Needs to Be Configured Per Customer

ADR-006 established that each customer organization has:
- An `OrgId` (UUID v7) — canonical internal identity, stable across renames
- An `OrgSlug` (kebab-case string) — analyst-facing friendly identifier
- A set of DTU instances — one per vendor integration the customer uses

ADR-007 established that each DTU instance declares:
- A `type` — the DTU type string (e.g., `"claroty"`, `"slack"`)
- A `mode` — `"shared"` or `"client"`, subject to per-type validation rules

ADR-008 established that client-mode DTU instances need org-identity at construction
time to key their state stores.

The customer config file is the single source of truth for all three of these
dimensions per managed organization. It is also the location where credential
references are declared (memory: `project_ai_opaque_credentials.md` — credentials
never transit the AI context; the config file holds opaque references, not values).

### 1.3 The AI-Opaque Credentials Constraint

The per-analyst MCP deployment model (memory: `project_deployment_model.md`) means
that analysts interact with Prism via Claude Code MCP tools. The AI context window
must never contain credential values. The customer config TOML file is a potential
attack surface: if it contains API keys or bearer tokens inline, and the config file
is read by an MCP tool that surfaces its content to the analyst session, credentials
would transit the AI context.

This ADR enforces the opaque reference model: the `credential_ref` field in each
`[[dtu]]` block must be a reference string (vault path, environment variable name,
or file path) that the credential store resolves at runtime. The config file never
contains the credential value itself. This mirrors the existing keyring namespace
pattern (`crates/prism-credentials/src/namespace.rs:20`) which stores namespace keys,
not secrets.

### 1.4 Data Generator Integration (D-043)

Decision D-043 established a hybrid data generator: named archetype catalog plus
deterministic generation keyed by `(org_id, seed, archetype, scale)`. The customer
config file is the natural location for declaring the archetype, scale, and seed
per-customer, since these determine what simulated data the customer's DTU instances
produce in test and demo scenarios. Each `[[dtu]]` block carries a `[data]` sub-table
with these fields.

---

## 2. Decision

### 2.1 File Location and Naming Convention

Customer configuration files live at:

```
{workspace_root}/customers/{org_slug}.toml
```

The `{org_slug}` portion of the filename is the canonical form of the `org_slug`
field declared inside the file. Validation rule: the filename stem MUST match the
`org_slug` field exactly (case-sensitive). A mismatch is a startup error.

Rationale for filename-as-slug: operators browsing the `customers/` directory can
identify which file belongs to which customer without opening it. Shell completion
works naturally (`customers/acme-` TAB). Config management tooling can sort and diff
by filename.

### 2.2 Top-Level Schema

A customer config file has the following structure:

```toml
schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000001"   # UUID v7; required
org_slug = "acme-corp"                             # matches filename; required
display_name = "ACME Corporation"                  # human-readable; required

[[dtu]]
# ... one or more [[dtu]] blocks; see §2.3

[shared_infra]
# ... optional per-customer shared infrastructure overrides; see §2.4
```

**Required top-level fields:**

| Field | Type | Constraint |
|-------|------|------------|
| `schema_version` | integer | MUST equal `1` for Wave 3 files |
| `org_id` | string (UUID v7) | MUST be a valid UUID v7 string; MUST be unique across all `customers/*.toml` files |
| `org_slug` | string | MUST match `^[a-zA-Z0-9_-]{1,64}$`; MUST match the filename stem exactly |
| `display_name` | string | 1–128 characters; MUST be non-empty UTF-8 |

Unknown top-level fields are rejected at startup with an explicit error
(`deny_unknown_fields` via serde). This forward-compatibility safety rule prevents
silently ignoring typos (e.g., `orgg_id` is not a valid field) and ensures that
when new fields are introduced in `schema_version = 2`, files written for version 1
will produce a clear error rather than silently loading without the new field.

### 2.3 `[[dtu]]` Block Schema

Each `[[dtu]]` block declares one DTU integration for this customer. A customer config
file MUST contain at least one `[[dtu]]` block.

```toml
[[dtu]]
type = "claroty"
mode = "client"
credential_ref = "vault://sensors/acme-corp/claroty/api-key"
spec = "sensors/claroty.toml"                  # path to sensor spec (client-mode only)

[dtu.data]
archetype = "HealthyOtEnvironment"            # named archetype from catalog
scale = 1.0                                   # positive float; 1.0 = default scale
seed = 42                                     # non-negative integer; deterministic generation
```

**`[[dtu]]` required fields:**

| Field | Type | Constraint |
|-------|------|------------|
| `type` | string | MUST appear in `DTU_DEFAULT_MODE` registry (ADR-007 §2.3) |
| `mode` | string | `"shared"` or `"client"` only; subject to ADR-007 validation rules |
| `credential_ref` | string | Non-empty; MUST follow one of the allowed opaque reference schemes (see §2.3.1) |

**`[[dtu]]` optional fields:**

| Field | Type | Default | Constraint |
|-------|------|---------|------------|
| `spec` | string | none | Path to sensor TOML spec; required when `mode = "client"`; prohibited when `mode = "shared"` |
| `[dtu.data]` | sub-table | none | Data generator parameters; see below |

**Note:** `allow_shared_override` is NOT a recognized field in Wave 3. The schema uses `deny_unknown_fields` (serde), so any `allow_shared_override` field in a `[[dtu]]` block produces `E-CFG-010` (unknown field rejection). See ADR-007 §7 OQ-1 (DEFERRED to Wave 4).

**`[dtu.data]` sub-table (optional):**

| Field | Type | Default | Constraint |
|-------|------|---------|------------|
| `archetype` | string | `"default"` | MUST be a string in the archetype catalog; unknown archetype → startup error |
| `scale` | float | `1.0` | MUST be a positive finite float (> 0.0, not NaN, not infinity) |
| `seed` | integer | `42` | MUST be a non-negative integer (u64 range) |

**`[[dtu]]` validation rules (evaluated at startup, process refuses to start on failure):**

1. `type` MUST be a known DTU type (in `DTU_DEFAULT_MODE`). Unknown type → error.
2. `mode` MUST be `"shared"` or `"client"`. Any other value → error.
3. If `type` is a Security Telemetry type and `mode = "shared"` → error (per
   ADR-007 §2.4 rule 3). **Wave 3: unconditional; `allow_shared_override` is not
   a recognized field (produces `E-CFG-010` if present; see §7 OQ-1 DEFERRED).**
4. If `mode = "client"`, `spec` MUST be present and the file MUST exist at the
   specified path. Missing spec → error. (File existence is checked in the validation
   pass, not deferred to DTU instantiation — see D-053.)
5. If `mode = "shared"`, `spec` MUST be absent. A `spec` field on a shared-mode
   block → error (shared instances do not have per-customer sensor specs).
6. `credential_ref` MUST be non-empty and MUST parse as one of the allowed opaque
   reference schemes (§2.3.1). Empty or invalid scheme → error.
7. `data.scale` MUST be > 0.0, finite, and not NaN. Invalid scale → error.
8. `data.seed` MUST be in the range `[0, u64::MAX]`. Negative value or overflow → error.
9. `data.archetype` MUST be in the archetype catalog (enumerated in ADR per D-043).
   Unknown archetype → error.
10. Unknown fields in `[[dtu]]` or `[dtu.data]` are rejected (`deny_unknown_fields`).
11. `display_name` MUST be non-empty. Empty `display_name = ""` is rejected with error
    code `E-CFG-001` (see D-052). The error message names the file, the offending field,
    and the error code.

#### 2.3.1 Credential Reference Schemes

A `credential_ref` string is an opaque reference that the credential store resolves
at runtime. The AI context window sees only the reference string, never the value
(memory: `project_ai_opaque_credentials.md`). The following schemes are supported:

| Scheme | Example | Resolution |
|--------|---------|------------|
| `vault://` | `vault://sensors/acme/claroty/api-key` | HashiCorp Vault KV path |
| `env://` | `env://ACME_CLAROTY_API_KEY` | Environment variable name |
| `file://` | `file:///etc/prism/secrets/acme-claroty` | Absolute file path |
| `keyring://` | `keyring://prism/acme/claroty` | OS keyring entry |

The credential store backend is responsible for resolution. The config parser validates
only that the scheme prefix is one of the four allowed strings and that the path
component is non-empty. The config parser does NOT attempt to resolve the credential
at parse time (that would require live external access during config loading, which
must not block startup validation).

### 2.4 `[shared_infra]` Block Schema (Optional)

The optional `[shared_infra]` block allows a customer to declare per-customer
overrides for shared MSSP infrastructure. This covers cases like "this customer's
Slack notifications should go to channel #acme-alerts rather than #all-alerts".

```toml
[shared_infra]
slack_channel = "#acme-security-alerts"        # override for Slack DTU channel routing
jira_project_key = "ACME"                      # override for Jira project key
pagerduty_service_key = "PXXXYYYYZZZ"          # per-customer PagerDuty routing key
```

All `[shared_infra]` fields are optional individually. The entire block may be absent.
Unknown fields in `[shared_infra]` are rejected (`deny_unknown_fields`).

`[shared_infra]` does not contain credential values — it contains routing parameters
(channel names, project keys) that are not sensitive. Actual authentication for shared
MSSP infrastructure is declared in the MSSP-level config, not in per-customer files.

### 2.5 Loading Lifecycle

**Startup loading sequence:**

1. Prism startup discovers all `*.toml` files in `customers/` via directory listing.
   Files are processed in lexicographic filename order for deterministic error reporting.
2. For each file, the TOML is parsed into a `CustomerConfig` struct (serde).
   Parse errors (malformed TOML, unknown fields, type mismatches) are collected.
3. Validation rules from §2.2 and §2.3 are applied to each parsed config.
   Validation errors are collected. The `spec` path file existence check runs in this
   validation pass (rule 4 above) — it is NOT deferred to DTU instantiation (see D-053).
4. After all files are parsed and validated, duplicate `org_id` or `org_slug` values
   across files are detected. Duplicates are errors.
5. If any errors were collected across all files, Prism logs all errors (not just the
   first) and exits before accepting any requests. This gives the operator a full
   picture of all config problems in one restart cycle.
6. If all files are valid and conflict-free, `OrgRegistry::register` is called once
   per file to populate the registry. `OrgRegistry` failures at this point are
   programming errors (the duplicate check in step 4 should have caught them).
7. Per-customer DTU instance maps are constructed from the registered `[[dtu]]` blocks.

**Hot-reload:** NOT in scope for Wave 3. Config changes require a process restart.
A `SIGHUP` handler that triggers hot-reload is a Wave 4 story if operational
experience reveals a need.

### 2.6 Schema Versioning

The `schema_version` field allows future breaking changes to the config schema without
requiring all existing files to be migrated simultaneously.

**Wave 3 (version 1):**
- `schema_version = 1` is the only valid value.
- Files without `schema_version` are rejected (it is a required field).

**Future versions:**
- When a breaking schema change is introduced (new required field, removed field,
  changed semantics), `schema_version` is bumped to 2.
- Prism version N+1 ships a migrator function `migrate_v1_to_v2(CustomerConfigV1) -> CustomerConfigV2`.
- Operators run `prism config migrate customers/` to in-place upgrade all files.
- Prism version N+1 rejects `schema_version = 1` files with a clear error:
  "Run 'prism config migrate' to upgrade customer config files to schema_version 2."
- Prism version N supports only `schema_version = 1`.
  Prism version N+1 supports only `schema_version = 2`.
  No version supports more than one schema simultaneously (simplifies the loading code).

The migrator is part of the `prism` CLI binary, not a separate tool.

### 2.7 Full Examples

#### Example 1: ACME Corporation — Multi-sector OT customer

```toml
schema_version = 1
org_id = "01975e4e-9f00-7abc-8def-000000000001"
org_slug = "acme-corp"
display_name = "ACME Corporation"

[[dtu]]
type = "claroty"
mode = "client"
credential_ref = "vault://sensors/acme-corp/claroty/api-key"
spec = "sensors/claroty.toml"

[dtu.data]
archetype = "HealthyOtEnvironment"
scale = 1.5
seed = 1001

[[dtu]]
type = "armis"
mode = "client"
credential_ref = "env://ACME_ARMIS_SECRET_KEY"
spec = "sensors/armis.toml"

[dtu.data]
archetype = "CompromisedEndpoint"
scale = 1.0
seed = 1002

[[dtu]]
type = "crowdstrike"
mode = "client"
credential_ref = "vault://sensors/acme-corp/crowdstrike/client-secret"
spec = "sensors/crowdstrike.toml"

[dtu.data]
archetype = "HighChurn"
scale = 1.0
seed = 1003

[[dtu]]
type = "slack"
mode = "shared"
credential_ref = "keyring://prism/mssp/slack/webhook-url"

[[dtu]]
type = "pagerduty"
mode = "shared"
credential_ref = "keyring://prism/mssp/pagerduty/routing-key"

[[dtu]]
type = "jira"
mode = "shared"
credential_ref = "keyring://prism/mssp/jira/api-token"

[shared_infra]
slack_channel = "#acme-security-alerts"
jira_project_key = "ACME"
```

#### Example 2: Globex — Cloud-native customer, no OT

```toml
schema_version = 1
org_id = "01975f11-aa00-7def-9012-000000000002"
org_slug = "globex"
display_name = "Globex Corporation"

[[dtu]]
type = "crowdstrike"
mode = "client"
credential_ref = "vault://sensors/globex/crowdstrike/client-secret"
spec = "sensors/crowdstrike.toml"

[dtu.data]
archetype = "LargeScale"
scale = 2.0
seed = 2001

[[dtu]]
type = "cyberint"
mode = "client"
credential_ref = "env://GLOBEX_CYBERINT_API_KEY"
spec = "sensors/cyberint.toml"

[dtu.data]
archetype = "SchemaDrift"
scale = 1.0
seed = 2002

[[dtu]]
type = "nvd"
mode = "shared"
credential_ref = "env://MSSP_NVD_API_KEY"

[[dtu]]
type = "slack"
mode = "shared"
credential_ref = "keyring://prism/mssp/slack/webhook-url"

[shared_infra]
slack_channel = "#globex-incidents"
jira_project_key = "GLBX"
```

#### Example 3: Initech — Minimal deployment, shared-only (evaluation)

```toml
schema_version = 1
org_id = "01975f22-bb00-7abc-1234-000000000003"
org_slug = "initech"
display_name = "Initech"

[[dtu]]
type = "claroty"
mode = "client"
credential_ref = "file:///etc/prism/initech/claroty-key"
spec = "sensors/claroty.toml"

[dtu.data]
archetype = "DormantTenant"
scale = 0.5
seed = 3001

[[dtu]]
type = "pagerduty"
mode = "shared"
credential_ref = "keyring://prism/mssp/pagerduty/routing-key"

[shared_infra]
pagerduty_service_key = "PABC1234"
```

---

## Rationale

**Why one file per customer rather than one combined `customers.toml`?**

A single combined file would make per-customer config changes visible in a single
git diff that affects all customers simultaneously. Separate per-customer files
enable: per-customer code ownership and access control (a support engineer managing
one customer can be given write access to that customer's file only); independent
review of each customer's config change; simple file-addition for new customers
without touching existing configs. This is consistent with the built-in sensors
config-driven pattern (memory: `feedback_builtin_sensors_config_driven.md`):
individual sensor spec files are separate TOML files, not combined into one.

**Why filename-must-match-slug rather than discovering slug from file content?**

The enforcement is bilateral. If the filename is the source of truth, then
renaming the file without updating the `org_slug` field inside would be silently
incorrect. If the file content is the source of truth, two files could declare the
same slug (caught by the duplicate detection in §2.5 step 4), but the filename would
mislead operators browsing the directory. The bilateral constraint — filename stem
MUST equal `org_slug` field — makes both sides auditable and eliminates the ambiguity.
The error message at startup is unambiguous: "File 'customers/acme-corp.toml' declares
org_slug 'acme-new' — filename stem must match org_slug exactly."

**Why `deny_unknown_fields` everywhere?**

Forward-compatibility safety. An operator who accidentally types `org_idd = "..."` in
a Wave 3 config will get an immediate startup error rather than silently having the
field ignored. This is the same rationale applied in ADR-001 through ADR-005 for
`POST /dtu/configure` payloads (TD-WV0-04). The penalty for adding a new field to
the schema in a future version is that old config files must be updated — but that
is handled by the schema versioning migrator (§2.6), making the cost explicit and
bounded.

**Why credential references rather than inline credential values?**

The AI-opaque credentials requirement (memory: `project_ai_opaque_credentials.md`)
is non-negotiable. In the per-analyst MCP deployment, if a credential value appeared
in a TOML file that was readable by the Prism process and reachable from the MCP tool
surface, a prompt injection attack could cause the analyst's Claude session to exfiltrate
the credential. Opaque reference strings (`vault://...`, `env://...`) carry no
extractable credential value even if they are inadvertently surfaced in tool output.
The UUID-form `OrgId` in the credential namespace key (ADR-006 §3.2) is similarly
opaque. The four allowed schemes cover the realistic deployment scenarios for an MSSP
operator: vault for secret management, env vars for CI/CD pipelines, files for
on-premises deployments, and OS keyring for developer machines.

**Why `schema_version` as a required field with no-unknown-versions rule?**

A missing `schema_version` is silently treated as "version 0" under many TOML schema
designs. This creates ambiguity when version 2 is introduced: is this an old file
that forgot the field (should be migrated) or a new file that intentionally omitted
it? Making `schema_version` required eliminates the ambiguity. Supporting only one
version at a time in any given binary release simplifies the loading code: there is
no version dispatch logic, no compatibility matrix, and no risk of a version 1 file
being loaded by version 2 logic silently. The migrator CLI command makes the upgrade
path explicit.

**Why hot-reload deferred?**

Hot-reload of customer configs would require `OrgRegistry` to support runtime
registration, deregistration, and modification of org mappings — which ADR-006
explicitly deferred to Wave 4 (Section 2.3 runtime registration note). Additionally,
hot-reload of `[[dtu]]` blocks would require constructing, starting, and tearing down
DTU instances at runtime, which interacts with the state segregation machinery
(ADR-008) in ways that are not yet specified. Deployment-time-only config changes
are simpler, safer, and consistent with the mode-change semantics of ADR-007 §2.5.

---

## 3. Threat Model

### 3.1 Config File as Credential Exfiltration Vector (BC-3.3.002)

**Threat:** A prompt injection payload in analyst input causes the Claude session to
call an MCP tool that reads `customers/acme-corp.toml` and returns its content in
the tool response, which the LLM then cites or reproduces.

**Mitigation:** The config file contains only opaque `credential_ref` strings (§2.3.1),
not credential values. The worst case is that an org's UUID and slug are revealed —
which are already visible in MCP tool output for other reasons (audit records,
query result metadata). No bearer token, API key, or password is present in any
`customers/*.toml` file. BC-3.3.002 must specify that no credential value may
appear in any field of any `customers/*.toml` file; the startup validator confirms
that `credential_ref` values match one of the allowed opaque schemes.

### 3.2 Duplicate OrgId / Slug Injection (BC-3.1.003 / BC-3.1.004, inherited from ADR-006)

**Threat:** An operator or CI pipeline creates two files with the same `org_id` or
`org_slug` (e.g., `customers/acme-corp.toml` and `customers/acme.toml` both declare
`org_id = "01975e4e-..."`).

**Mitigation:** Step 4 of the loading lifecycle (§2.5) performs a cross-file duplicate
check before calling `OrgRegistry::register`. A duplicate `org_id` or `org_slug`
across files is a startup error with an explicit message naming both files. Prism
does not start until the conflict is resolved.

### 3.3 Schema Version Forgery / Downgrade (BC-3.3.003)

**Threat:** A config file written for version 2 schema (future) declares
`schema_version = 1` to pass a version 1 validator, causing new required fields to
be silently ignored.

**Mitigation:** The Wave 3 validator rejects any `schema_version` value other than
`1`. `deny_unknown_fields` ensures that any field from a hypothetical version 2 schema
that does not exist in version 1 will cause a parse error. A version 2 field cannot
be silently ignored in a version 1 parser — it will be rejected as unknown. The
downgrade attack thus causes a startup error rather than silent misbehavior.

### 3.4 Malformed `org_id` (Non-UUID or UUID v4)

**Threat:** An operator pastes a UUID v4 value (e.g., from a legacy system) as
`org_id`. Under the ADR-006 architecture, UUIDs v4 are prohibited because they break
monotonic RocksDB key ordering (`ids.rs:4-5`).

**Mitigation:** The startup validator parses `org_id` as a UUID and verifies that
the version nibble is `7`. Any other UUID version → error: "org_id MUST be UUID v7;
got version 4". The `uuid_v7_newtype!` macro (`ids.rs:10-42`) already embeds this
check; the config loader calls `OrgId::try_from(uuid_str)?` which exercises the macro's
validation logic.

---

## 4. Alternatives Considered

| Option | Description | Decision |
|--------|-------------|----------|
| **Single `customers.toml` with `[[organization]]` table array** | All customers in one file | Rejected: per-customer access control is impossible; a change to one org's config requires a diff/review touching the whole file; new customer onboarding requires modifying an existing file rather than adding a new one. |
| **Directory per customer: `customers/acme-corp/config.toml`** | Per-customer subdirectory | Rejected: no benefit over flat files for the complexity added. Sensor TOML specs already use a flat directory. Subdirectory structure would be needed only if a customer had multiple config files, which is not a Wave 3 requirement. |
| **JSON or YAML instead of TOML** | Customer config in JSON or YAML | Rejected: TOML is the established convention for built-in sensor configs (memory: `feedback_builtin_sensors_config_driven.md`). Mixing formats in `sensors/` (TOML) vs `customers/` (JSON) would be inconsistent. TOML has superior comment support for documenting credential references inline. |
| **Inline credential values with encryption** | Encrypted API keys in config file, decrypted at startup | Rejected: key management for the encryption key is equivalent complexity to using a secrets manager. The opaque reference model is simpler and more auditable. An encrypted config file still risks credential exposure if the encryption key is managed poorly. |
| **Support multiple schema versions simultaneously** | Version dispatch in the loader | Rejected: version dispatch logic is a testing surface that grows with each version. The single-version rule enforced by `schema_version` plus a one-shot migrator CLI command is simpler and prevents subtle compatibility bugs between version-specific code paths. |
| **`org_slug` from filename only, no redundant field in file** | Remove `org_slug` from file; derive from filename | Rejected: creates a split source of truth where the org's identity is partially in the filesystem (filename) and partially in the file content (org_id, display_name). A file whose content is portable (e.g., checked into git and checked out in a different directory) would lose its slug. The bilateral constraint (filename MUST equal `org_slug` field) is redundant but auditable and self-documenting. |

---

## 5. Consequences

### Positive

- The customer configuration model is fully config-driven and consistent with the
  built-in sensors philosophy (memory: `feedback_builtin_sensors_config_driven.md`).
  New customer onboarding is entirely a config-file operation with no code changes.
- Credential references are opaque reference strings, satisfying the AI-opaque
  credentials requirement (memory: `project_ai_opaque_credentials.md`).
- `deny_unknown_fields` at all levels catches typos and out-of-date config files
  immediately at startup, before the process accepts any requests.
- Schema versioning with a single-supported-version rule and migrator CLI keeps the
  loading code simple and migration explicit.
- The three worked examples (ACME, Globex, Initech) serve as both spec documentation
  and as the basis for integration test fixtures in ADR-009 (test harness).

### Negative

- The bilateral filename-slug constraint means that renaming an org's slug requires
  renaming the file AND updating the `org_slug` field. An automated slug-rename script
  would be helpful but is out of scope for Wave 3.
- Hot-reload is deferred. A config change always requires a process restart. For
  production MSSP deployments with SLA requirements, the restart window must be
  planned and communicated. This is acceptable for Wave 3 but must be addressed if
  live org onboarding becomes a requirement.
- The `[shared_infra]` block `deny_unknown_fields` rule means that adding a new
  shared infrastructure override parameter (e.g., `opsgenie_team_id`) requires a
  schema version bump. This is correct behavior but creates ceremony around adding
  new override parameters.

---

## 6. Behavioral Contracts Scoped by This ADR

| BC ID | Title | Postcondition summary |
|-------|-------|-----------------------|
| BC-3.3.001 | Startup rejects Security Telemetry type with shared mode | (Referenced from ADR-007.) If a `[[dtu]]` block declares a Security Telemetry type with `mode = "shared"`, Prism MUST NOT start. The error MUST name the file and the offending `[[dtu]]` block. **Wave 3: unconditional guard; `allow_shared_override` is NOT IMPLEMENTED (see ADR-007 §7 OQ-1 DEFERRED).** |
| BC-3.3.002 | Credential values MUST NOT appear in customer config files | No field in any `customers/*.toml` file contains a credential value. `credential_ref` fields MUST match one of the four allowed opaque reference schemes. |
| BC-3.3.003 | Startup rejects files with unknown or invalid schema_version | A `customers/*.toml` file with `schema_version` absent, or set to a value other than the supported version, MUST cause Prism to refuse to start with an explicit error. |

---

## 7. Open Questions for Next Dispatch

1. **OrgSlug regex: 32 vs 64 character maximum.** ADR-006 Section 8 open question 1
   proposes tightening from 64 to 32 characters. The filename-as-slug constraint
   (§2.1) means the filename length is also constrained. Confirm the final maximum
   before authoring BC-3.3.002; grep `customers/` (currently empty) and built-in
   sensor TOMLs to check whether any in-use slugs exceed 32 characters.

2. **`spec` field: workspace-relative or absolute path?** Section 2.3 specifies
   `spec = "sensors/claroty.toml"` as a workspace-relative path. Confirm whether the
   path resolution is relative to the `customers/` directory (the config file's
   location) or the workspace root. Workspace-relative is more natural for operators
   but requires the loading code to resolve against the process's working directory,
   which may vary in different deployment contexts.

3. **`[shared_infra]` vs per-`[[dtu]]` overrides.** Currently shared infrastructure
   overrides are in a top-level `[shared_infra]` block, not nested inside the
   relevant `[[dtu]]` block. This means `slack_channel` is in `[shared_infra]`, not
   in the `[[dtu]] type = "slack"` block. This separation makes it ambiguous which
   `[[dtu]]` block the override applies to if a customer has multiple Slack DTU
   blocks (unlikely but possible). Consider whether overrides should be inlined in
   the relevant `[[dtu]]` block: `[[dtu]] type = "slack" channel = "#acme-alerts"`.

4. **Archetype catalog location.** Section 2.3 specifies that `data.archetype` MUST
   be in the archetype catalog, but the catalog itself is defined in the ADR for D-043
   (not yet drafted). The startup validator must reference the catalog. Where is the
   catalog defined: a constant in `prism-orgs`, a separate `archetypes.toml` at
   workspace root, or embedded in `prism-dtu-common`? This decision affects the
   dependency graph between `prism-config` (which validates the customer config)
   and the data generator crate.

5. **`pagerduty_service_key` in `[shared_infra]`: credential or routing parameter?**
   A PagerDuty service key (`PABC1234`) is a routing parameter that identifies which
   PagerDuty service an event is routed to — it is not an authentication credential
   (which is the Events API v2 routing key, typically a different value). However,
   it is semantically sensitive (revealing it exposes the PagerDuty service ID).
   Confirm whether service keys belong in `[shared_infra]` (as routing parameters)
   or in `credential_ref` (as secrets). If the latter, `[shared_infra]` has no
   PagerDuty field and the service routing is embedded in the credential reference.

---

## 8. ADR Chain — Related Documents

- **ADR-006** (antecedent): Establishes `OrgId`, `OrgSlug`, `OrgRegistry`, and the
  `mode` field concept. ADR-010 specifies the full TOML schema that produces these
  values at startup.
- **ADR-007** (antecedent): Specifies the per-type mode default registry and validation
  rules. ADR-010's `[[dtu]]` block validation rules in §2.3 directly apply ADR-007's
  Security Telemetry classification.
- **ADR-008** (antecedent): Establishes per-org state keying. The `data.archetype`,
  `data.scale`, and `data.seed` fields in ADR-010 feed the multi-tenant data generator
  whose output is keyed by `(OrgId, seed, archetype, scale)` — consistent with
  ADR-008's `(OrgId, String)` composite keying pattern.
- **ADR-009** (consequent, planned): Multi-tenant test harness. The three worked
  examples in §2.7 (ACME, Globex, Initech) are the basis for integration test fixture
  files loaded by the test harness.
- **ADR-011** (consequent, planned): Network isolation. Customer config files declare
  which DTU types a customer uses; the network isolation layer in ADR-011 uses this
  information to construct per-org Docker Compose network topologies.

---

## 9. Source / Origin

- **PO decisions:** D-041 (OrgId/OrgSlug identity), D-042 (configurable mode),
  D-046 (housekeeping triage — TD-ADR005-001 CODEOWNERS security reviewer for
  `prism-sensors/src/auth/` is adjacent to credential ref validation) —
  recorded in `.factory/STATE.md`, Wave 3 kickoff 2026-04-27.
- **Memory: AI-opaque credentials** (`project_ai_opaque_credentials.md`) — credential
  references must be opaque; the `credential_ref` field and allowed scheme list
  directly implement this requirement.
- **Memory: built-in sensors config-driven** (`feedback_builtin_sensors_config_driven.md`)
  — customer TOML files extend the same config-driven philosophy to org registration.
- **Code as-built — credential namespace:**
  `crates/prism-credentials/src/namespace.rs:20` — `namespace_key(tenant, sensor, name)`;
  the `credential_ref` scheme list is the operator-facing counterpart to this
  internal namespace key format.
- **Code as-built — ids.rs UUID v7 constraint:**
  `crates/prism-core/src/ids.rs:4-5` — UUID v4 prohibition; informs the `org_id`
  validation rule (must be UUID v7 with version nibble = 7).

---

## Decision Refinements (2026-04-27)

The following questions surfaced during BC authoring (Phase 3.A) and were resolved by the orchestrator on 2026-04-27. Each refinement is recorded here for historical traceability and is binding for Wave 3 implementation.

### D-052 — `display_name = ""` validation uses error code `E-CFG-001`

**Question:** What error code is used when `display_name` is empty? Should each missing-required-field scenario have its own error code, or a shared one?

**Resolution:** Empty `display_name = ""` validation uses error code `E-CFG-001`. This is a single, shared "missing required field" error code — there is no proliferation of per-field error codes. Any required string field that is present in the TOML but has an empty value uses `E-CFG-001`. The error message includes the file path, field name, and the `E-CFG-001` code so operators can identify the exact violation.

**Rationale:** Error code proliferation (e.g., `E-CFG-007` for `display_name`, `E-CFG-008` for `org_slug`) creates a mapping table that must be maintained in sync with the schema. A single `E-CFG-001` for "required field present but empty" is sufficient for operators and monitoring systems to identify the class of error. Field-specific detail is in the human-readable message, not in the code. This follows the error code consolidation principle established in ADR-001 through ADR-005 for DTU configuration errors.

**Affected BCs:** BC-3.3.002

### D-053 — `spec` path file existence check runs in validation pass

**Question:** The `spec` field on a `[[dtu]]` block points to a sensor spec TOML file. When does the file existence check run — in the startup validation pass, or deferred to DTU instantiation?

**Resolution:** The `spec` path file existence check runs in the validation pass (step 3 of the loading lifecycle, §2.5), NOT deferred to DTU instantiation. This preserves the zero-partial-registration invariant: if any `spec` file is missing, the entire startup fails before any `OrgRegistry::register` calls are made. No org is partially registered.

**Rationale:** Deferring the file existence check to DTU instantiation (step 7) would allow `OrgRegistry::register` (step 6) to succeed for a config that references a nonexistent spec file. The registry would then contain an org whose DTU is uninstantiable. Any query for that org would fail at dispatch time with a confusing "spec file not found" error rather than a clean startup failure. The zero-partial-registration invariant (no org in the registry unless all its config is valid) is a correctness invariant for the registry; checking file existence in the validation pass enforces it. The cost is a small amount of I/O during startup validation, which is acceptable.

**Affected BCs:** BC-3.3.001 (startup validation completeness)

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.5 | 2026-04-27 | product-owner | M-003 fix: §2.3 schema snippet `archetype = "enterprise-ot"` replaced with valid PascalCase catalog archetype `"HealthyOtEnvironment"` (ADR-009 §2.2). §2.7 examples were already correct (fixed in v0.4); only the §2.3 illustrative snippet was stale. |
| 0.4 | 2026-04-27 | product-owner | C-001 fix: §2.7 Examples 1/2/3 archetype values replaced with PascalCase ADR-009 catalog names: Example 1 uses `HealthyOtEnvironment`, `CompromisedEndpoint`, `HighChurn`; Example 2 uses `LargeScale`, `SchemaDrift`; Example 3 uses `DormantTenant`. Previous kebab-case strings (`enterprise-ot`, `enterprise-iot`, etc.) were not in the ADR-009 §2.2 archetype catalog. |
| 0.3 | 2026-04-27 | product-owner | C-2 sync: §2.3 optional-fields table — `allow_shared_override` row dropped (unknown field in Wave 3, rejected as E-CFG-010 by deny_unknown_fields); replaced with explicit Wave 3 deferral note. §2.3 validation rule 3 updated to remove allow_shared_override condition. §6 BC-3.3.001 row updated. |
| 0.2 | 2026-04-27 | architect | Decision Refinements: D-052 (E-CFG-001 for empty display_name; no per-field code proliferation), D-053 (spec path existence check in validation pass, not deferred to DTU instantiation) |
| 0.1 | 2026-04-27 | architect | Initial draft — full TOML schema, validation rules, loading lifecycle, schema versioning, three worked examples |
