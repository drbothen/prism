---
document_type: gate-step-report
gate_step: d
gate_step_name: security-review
cycle: wave-3-multi-tenant
gate: wave-3-integration-gate
scope: 6696e374^..a3bd5a0f
reviewer: vsdd-factory:security-reviewer
date: 2026-05-01
phase: 3
wave: 3
step: d
develop_sha: a3bd5a0f
verdict: APPROVED_WITH_CONDITIONS
total_findings: 10
critical: 0
high: 3
medium: 4
low: 3
files_reviewed: 28
---

# Wave 3 Integration Gate — Gate Step D: Security Review

**Scope:** 6696e374^..a3bd5a0f (Wave 3 Multi-Tenant, 616 files, +61,891/-522 lines)
**Reviewer:** vsdd-factory:security-reviewer
**Date:** 2026-05-01
**Develop SHA:** a3bd5a0f
**Verdict:** APPROVED WITH CONDITIONS — 10 findings (0 CRITICAL, 3 HIGH, 4 MEDIUM, 3 LOW)

**Condition:** WGS-W3-001, WGS-W3-002, and WGS-W3-003 must be resolved before
production deployment carrying real customer credential payloads. MEDIUM/LOW items
tracked as technical debt.

---

## Wave 2 Carry-Over Status

The two HIGH findings from Wave 2 (WGS-W2-001 AQL injection, WGS-W2-002 bearer token
exposure) were remediated via PRs #69 and #72 (W2-FIX-I) before Wave 3 began.
RUSTSEC-2026-0114 (wasmtime 44.0.0) was patched to 44.0.1 during the W3-FIX cycle.
All Wave 2 conditions are satisfied; they are not carried forward.

---

## Positive Security Patterns Observed in Wave 3

Before reporting findings, the following constitute genuine security improvements
delivered by this wave:

- **OrgId(Uuid v7) as the sole internal routing key** — credential namespaces, RocksDB
  key prefixes, audit record primary keys, and DTU state HashMap keys are all typed
  `OrgId`. Slug collision cannot cause cross-tenant credential access by construction.
  The UUID-stable namespace survives org rename without key migration.

- **OrgRegistry bijectivity enforcement** — `OrgRegistry::register` returns typed
  `RegistrationError` variants (SlugConflict, IdConflict) on any collision. Prism
  refuses to start if duplicates are detected. The idempotent re-registration (D-050)
  is correctly implemented and does not weaken the bijectivity invariant.

- **`deny_unknown_fields` on all four TOML schema structs** — `CustomerConfig`,
  `DtuBlock`, `DtuData`, `SharedInfra` all carry `#[serde(deny_unknown_fields)]`.
  Unknown fields produce `E-CFG-010`, blocking silent schema extension abuse.

- **Credential heuristic scan at config load time** — `scan_for_credentials` walks the
  full TOML value tree recursively, catching credential-pattern field names with
  non-scheme values before any OrgRegistry registration. The error message deliberately
  omits the field value (BC-3.3.002 Invariant 3).

- **Error message sanitization in the TOML parser** — `sanitize_error_message` redacts
  credential-named fields from TOML 0.8 snippet lines before they surface in error logs,
  preventing inadvertent credential disclosure through parse errors.

- **SecretString used throughout credential pipeline** — `CredentialStore::get/set`,
  `CredentialStoreOrgId::get_by_org/set_by_org`, `resolve_secret`, `selector` all use
  `SecretString` with `expose_secret()` only at the HTTP header injection site.

- **Audit entry carries non-nullable org_id and org_slug** — `AuditEntry` has
  `static_assertions::assert_fields!` verifying both fields exist at compile time
  (BC-3.1.002). The `org_id` UUID primary key ensures audit records survive slug renames.

- **Network harness pre-allocates all TCP listeners simultaneously** — `allocate_network_listeners`
  uses synchronous `std::net::TcpListener::bind("127.0.0.1:0")` in a single pass,
  eliminating the bind-drop-rebind race window identified in ADR-011 §2.5 and D-058.
  This correctly addresses the W3-FIX-WIN-001 cross-platform concern.

- **Namespace key validation via `validate_sensor`** — the sensor component in
  `namespace_key_by_org_id` is validated to `[a-zA-Z0-9_-]` before construction,
  preventing `/`-injection into the credential key path.

- **BC-3.3.001 enforcement** — Security Telemetry DTU types (Claroty, Armis,
  CrowdStrike, Cyberint) are rejected when mode=shared in TOML config, preventing
  misconfigured multi-tenant data leakage via shared adapter instances.

- **wasmtime 44.0.1 patched** — RUSTSEC-2026-0114 resolved; Cargo.lock reflects the
  patched version.

---

## HIGH Findings

### SEC-001: X-Org-Id Header Accepted Without Authentication Verification — Unauthenticated Tenant Spoofing in DTU Network Harness

- **Severity:** HIGH
- **CWE:** CWE-287 (Improper Authentication), CWE-639 (Authorization Bypass Through
  User-Controlled Key)
- **OWASP:** A01:2021 — Broken Access Control
- **Files:**
  - `crates/prism-dtu-claroty/src/routes/devices.rs:135-143` (`extract_org_id`)
  - `crates/prism-dtu-crowdstrike/src/routes/hosts.rs:202-213` (`extract_org_id`)
  - `crates/prism-dtu-cyberint/src/routes/alerts.rs:54-60` (`extract_org_id`)

**Attack Vector:**

In the network harness isolation mode (ADR-011, S-3.3.04), each customer organization
receives a dedicated TCP listener running a real HTTP server. The `OrgId` for incoming
requests is extracted exclusively from the `X-Org-Id` header (Claroty/CrowdStrike) or
`X-Prism-Org-Id` header (Cyberint) with no cryptographic binding or server-side
validation. Any client that can reach the per-org TCP port can supply an arbitrary UUID
in the header and read or write state belonging to a different organization.

Specific code:

```rust
// crates/prism-dtu-claroty/src/routes/devices.rs:135
fn extract_org_id(headers: &HeaderMap) -> OrgId {
    const SENTINEL: Uuid = uuid::uuid!("00000000-0000-7000-8000-000000000000");
    headers
        .get("x-org-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .map(OrgId::from_uuid)
        .unwrap_or(OrgId::from_uuid(SENTINEL))
}
```

The comment at line 134 explicitly states: "Structural placeholder until auth
middleware wires validated `OrgId` into request extensions (S-3.2.02)." This
acknowledges the stub nature, but the stub is deployed in the network harness
where actual HTTP connections are accepted.

**Impact:**

In a network-mode harness with two customer organizations, a test or external client
that knows the listening address for Org B's Claroty clone can send requests with
`X-Org-Id: <Org A's UUID>` and read Org A's tag state, effectively breaching the
BC-3.2.001 isolation invariant at the HTTP layer despite the state-layer enforcement
being correct. The state-layer tests pass because they invoke `ClarotyState` methods
directly with the correct `OrgId`; the HTTP-layer trust of the caller-supplied header
is the gap.

**Evidence:**

The test in `crates/prism-dtu-claroty/tests/multi_tenant.rs:256-293` exercises
cross-org isolation by sending `X-Org-Id: ORG_A` and then `X-Org-Id: ORG_B`, but
the test itself supplies the correct headers — it does not test the case where a
client sends the wrong org header on an authenticated connection.

**Contextual Scope:**

This finding applies to the test harness infrastructure, not to a production MCP
server path (the MCP layer does not exist yet in Wave 3). However, the network harness
is described in ADR-011 §2.3 as a "real HTTP" boundary designed to catch isolation bugs
that in-process testing misses. An org header that is simply trusted without validation
defeats this design objective and creates a false sense of isolation assurance.

**Proposed Mitigation:**

For the test harness context: the `OrgId` should be bound at connection time to the
specific per-org clone instance (the clone knows its `instance_org_id`), and any
request whose `X-Org-Id` header does not match the clone's own `OrgId` should return
HTTP 403 rather than silently using the caller-supplied value. Cyberint's
`instance_org_id` fallback pattern is closer to correct, but still accepts a
foreign org header without rejecting it.

**Remediation Routing:** implementer (prism-dtu-claroty, prism-dtu-crowdstrike,
prism-dtu-cyberint route handlers)

---

### SEC-002: `POST /dtu/reset` Unauthenticated on Four Production DTU Clones — Harness State Erasure Without Admin Token

- **Severity:** HIGH
- **CWE:** CWE-306 (Missing Authentication for Critical Function)
- **OWASP:** A07:2021 — Identification and Authentication Failures
- **Files:**
  - `crates/prism-dtu-claroty/src/routes/devices.rs:337-340` (`dtu_reset`)
  - `crates/prism-dtu-crowdstrike/src/routes/mod.rs:29-37` (`dtu_reset`)
  - `crates/prism-dtu-armis/src/routes/dtu.rs:60-69` (`post_reset`)
  - `crates/prism-dtu-slack/src/routes/dtu.rs:55-64` (`post_reset`)

**Attack Vector:**

Wave 2 identified this pattern for PagerDuty, Jira, and Slack DTUs (WGS-W2-003). Wave 2
remediation addressed it for Slack only via the `X-Admin-Token` gate on `POST /dtu/configure`.
However, `POST /dtu/reset` remains unauthenticated on four clones:

1. **Claroty** — `dtu_reset` at `devices.rs:337` takes only `State`; no header check.
2. **CrowdStrike** — comment at `mod.rs:29` explicitly states "No auth required."
3. **Armis** — `post_reset` at `dtu.rs:60` takes only `State`; no header check.
4. **Slack** — `post_reset` at `slack/routes/dtu.rs:61` takes only `State`; no header check.

By contrast, `POST /dtu/configure` on all four of these clones correctly requires
`X-Admin-Token` (the Armis clone has admin_token in its `ArmisState` and checks it
on configure).

**Impact:**

Any client that can reach the harness TCP port can issue `POST /dtu/reset` and erase
all org-keyed state for all tenants without authentication. In a network-isolation-mode
harness, this allows a client connected to Org B's port to erase Org A's state on the
Claroty clone (since the state is shared via `Arc<ClarotyState>`). This undermines test
fidelity in multi-tenant harness tests and would undermine production isolation if the
harness pattern were promoted to a deployed component.

**Evidence:**

Armis `clone.rs` at line 63 generates `admin_token = uuid::Uuid::new_v4().to_string()`
and stores it in `ArmisState`, making it available for configure protection. The reset
handler at `dtu.rs:66` bypasses this entirely:

```rust
pub async fn post_reset(State(state): State<Arc<ArmisState>>) -> impl IntoResponse {
    state.reset();
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}
```

**Proposed Mitigation:**

Apply the same `X-Admin-Token` gate used in `dtu_configure` to `dtu_reset` on all four
affected clones. The admin token is already generated and available in the clone state.

**Remediation Routing:** implementer (prism-dtu-claroty, prism-dtu-crowdstrike,
prism-dtu-armis, prism-dtu-slack)

---

### SEC-003: Spec Path Traversal Not Prevented — Relative `../` Paths in `[[dtu]].spec` Can Access Arbitrary Filesystem Paths

- **Severity:** HIGH
- **CWE:** CWE-22 (Improper Limitation of a Pathname to a Restricted Directory —
  'Path Traversal')
- **OWASP:** A01:2021 — Broken Access Control
- **File:** `crates/prism-customer-config/src/validator.rs:539-548`

**Attack Vector:**

When a `[[dtu]]` block in a customer TOML config carries `mode = "client"`, the
validator requires a `spec` field pointing to a sensor spec file (R-CUST-014). The
validation resolves the path by joining it directly to the parent directory of the
config file:

```rust
let parent = config_path.parent().unwrap_or(Path::new("."));
let resolved = parent.join(spec_path);
if !resolved.exists() {
    errors.push(ConfigError::SpecFileNotFound { ... });
}
```

No path normalization, canonicalization, or boundary check is performed. A customer
TOML file with `spec = "../../../../etc/passwd"` would resolve to a valid path and
pass the `resolved.exists()` check, causing Prism to attempt to load `/etc/passwd`
as a sensor spec file at startup. An operator who processes untrusted TOML files (e.g.,
received from a managed customer) would have their process reading arbitrary filesystem
paths.

The existence check `resolved.exists()` does NOT prevent traversal — it only determines
whether the error variant is `SpecFileNotFound`. If the traversal target file exists,
the check passes silently.

**Impact:**

An attacker who can influence the content of a `customers/*.toml` file can direct Prism
to read any file on the filesystem at startup. Depending on how the spec file content
is subsequently processed, this may expose sensitive system files or cause unexpected
parse errors that reveal path information through error messages.

**Evidence:**

The validator at lines 539-548 uses `parent.join(spec_path)` without any of the
following mitigations: `Path::canonicalize`, prefix check against `customers/` directory,
rejection of `..` components, or TOML-level regex validation of the spec string.

**Proposed Mitigation:**

Two complementary controls:
1. At TOML parse time: validate `spec` field with a regex that rejects `..` components
   (e.g., `^[a-zA-Z0-9_/.-]+$` with an explicit `!spec_path.contains("..")` check).
2. After `parent.join(spec_path)`: call `resolved.canonicalize()` and verify the result
   starts with the `customers/` directory path. Return `E-CFG-015` (SpecFileNotFound)
   if the canonical path escapes the expected directory.

**Remediation Routing:** implementer (prism-customer-config/src/validator.rs)

---

## MEDIUM Findings

### SEC-004: `CredentialStoreOrgId` Backend Methods Are `todo!()` Stubs — OrgId-Keyed Credential Isolation Not Yet Enforced in Keyring Backend

- **Severity:** MEDIUM
- **CWE:** CWE-284 (Improper Access Control)
- **OWASP:** A01:2021 — Broken Access Control
- **Files:**
  - `crates/prism-credentials/src/trait_.rs:89-160` (trait definition, all stub)
  - `crates/prism-credentials/src/keyring.rs:256-` (KeyringBackend impl)

**Description:**

`CredentialStoreOrgId` is defined as the "authoritative interface after the ADR-006 §4
Step 3 migration." The `EncryptedFileBackend` implementation is fully implemented with
correct `OrgId`-keyed path construction (`base_dir/{org_id_uuid}/{sensor}/{name}.enc`).

However, the `KeyringBackend` implementation of `CredentialStoreOrgId` is documented
as "STUB — todo!() pending Red Gate test passage" in the trait module comment. If the
system is configured to use the keyring backend and the `get_by_org`/`set_by_org`
methods are called, they will panic at runtime rather than returning an isolation-safe
response.

More critically, the legacy `CredentialStore` trait (OrgSlug-keyed) remains implemented
and active in both backends. If any callsite still uses the slug-keyed path during the
migration window, BC-3.2.002 isolation is not enforced (slug rename → credential
cross-reach).

**Proposed Mitigation:**

Complete the `KeyringBackend::CredentialStoreOrgId` implementation to remove the panic
paths. Audit all callsites to confirm none invoke the legacy `CredentialStore` (slug-keyed)
interface in contexts where org-scoped isolation is required.

**Remediation Routing:** implementer (prism-credentials)

---

### SEC-005: OrgSlug Validation Allows 64-Character Slugs — ADR-006 Proposed Tightening to 32 Unimplemented

- **Severity:** MEDIUM
- **CWE:** CWE-20 (Improper Input Validation)
- **File:** `crates/prism-core/src/tenant.rs:25`

**Description:**

ADR-006 §2.1 explicitly states: "Slug length: the current maximum is 64 characters.
This ADR proposes tightening to 32 characters... Rationale: analyst-facing surfaces do
not benefit from 64-character slugs; 32 provides sufficient namespace while reducing the
surface for log-injection payloads."

The open question was marked for resolution before BC-3.1.001 authoring. However, the
current implementation retains the 64-character limit:

```rust
pub const ORG_SLUG_PATTERN: &str = r"^[a-zA-Z0-9_-]{1,64}$";
```

A 64-character slug containing valid characters like `aaa...64-chars` still validates
cleanly. The security concern is that long slugs appearing in log lines (as
`org_slug` denormalized in every audit entry) create wider vectors for log injection
if the slug ever appears in a format that does not correctly quote or escape it.

**Proposed Mitigation:**

Resolve ADR-006 §8 OQ-1 by measuring actual slug lengths in `customers/*.toml` fixtures
and tightening to 32 characters if no fixture exceeds it. Update `ORG_SLUG_PATTERN` and
propagate the BC-3.1.001 verification property.

**Remediation Routing:** implementer or product-owner to resolve OQ-1, then implementer

---

### SEC-006: `sanitize_error_message` Pattern Matching May Not Cover All TOML 0.8 Error Formats

- **Severity:** MEDIUM
- **CWE:** CWE-209 (Generation of Error Message Containing Sensitive Information)
- **OWASP:** A09:2021 — Security Logging and Monitoring Failures
- **File:** `crates/prism-customer-config/src/validator.rs:327-349`

**Description:**

The `sanitize_error_message` function in the validator redacts credential-named fields
from TOML 0.8 parse error snippets. The redaction logic relies on the presence of
` | ` as a pipe separator in TOML snippet lines and then pattern-matches on `" = "` to
find assignment lines.

This logic has two fragility points:

1. **Multi-line string values**: TOML supports multi-line basic strings (`"""..."""`).
   If a credential value spans multiple lines, the snippet may render the value on a
   continuation line that does not contain the field name on the same line as the `" = "`.
   The pattern would not recognize the continuation as a credential value to redact.

2. **Array or inline table values**: `credential_ref = ["vault://x", "env://y"]` is
   syntactically unusual but valid TOML. The `starts_with(scheme)` check targets only
   string scalars; an array value would not trigger the heuristic, so a non-scheme
   element in an array that happens to match the credential name pattern would not be
   caught by `scan_for_credentials`.

**Proposed Mitigation:**

Add test cases for multi-line TOML strings containing credential-pattern field names
to verify that `sanitize_error_message` redacts them correctly. Alternatively, use a
more conservative redaction strategy: redact any TOML snippet line that contains a
field name matching credential patterns, regardless of whether the value can be parsed.

**Remediation Routing:** implementer (prism-customer-config/src/validator.rs)

---

### SEC-007: `org_slug` Denormalized in Audit Records Is Not Validated Against the Active Registry at Read Time

- **Severity:** MEDIUM
- **CWE:** CWE-345 (Insufficient Verification of Data Authenticity)
- **File:** `crates/prism-audit/src/audit_entry.rs:198-272`

**Description:**

BC-3.1.002 requires audit entries to carry both `org_id: OrgId` (UUID, stable, primary
key) and `org_slug: OrgSlug` (denormalized at write time). The design explicitly
acknowledges that historical records will show old slugs after a rename — this is
by design for forensic readability.

The security concern is that `org_slug` at write time is accepted as-is from the
caller's `AuditRequest` struct without cross-checking against `OrgRegistry::slug_for(org_id)`.
If the emitting code passes an incorrect slug (e.g., a typo, a slug belonging to a
different org, or a slug that has been transferred to a different OrgId via a config
edit), the audit record permanently records the wrong association.

In the current implementation, audit record construction at `audit_emitter.rs:266-267`
takes `req.org_slug.clone()` directly from the caller. There is no invariant assertion
that `registry.slug_for(req.org_id) == Some(req.org_slug)` at write time.

**Impact:**

Audit records could be written with a `org_slug` that does not correspond to the `org_id`,
making forensic queries that filter by slug misleading. An attacker who can influence
the `AuditRequest` parameters could potentially forge the `org_slug` field to attribute
activity to the wrong organization in the audit trail.

**Proposed Mitigation:**

At audit record construction time, assert (in debug mode) or verify (in production mode)
that `OrgRegistry::slug_for(org_id) == Some(org_slug)`. Log a warning (not an error —
to preserve audit-must-not-fail semantics) if the check fails.

**Remediation Routing:** implementer (prism-audit)

---

## LOW Findings

### SEC-008: `OrgId::from_uuid` Does Not Verify UUID v7 — Non-v7 UUIDs Can Enter Internal Routing

- **Severity:** LOW
- **CWE:** CWE-20 (Improper Input Validation)
- **Files:**
  - `crates/prism-core/src/ids.rs` (`OrgId::from_uuid`)
  - `crates/prism-dtu-claroty/src/routes/devices.rs:142` (parse path)
  - `crates/prism-dtu-crowdstrike/src/routes/hosts.rs:211` (parse path)

**Description:**

The `OrgId::from_uuid` constructor used in route handlers accepts any UUID, including
v1, v4, and v0 (nil). The comment in `boot.rs:88` acknowledges this: "from_uuid does
not re-check (use from_uuid_v7 for stricter enforcement if needed)."

Headers that supply UUID v4 values (random, from external tools) are accepted without
error and route correctly. However, the Architecture Compliance Rules in `ids.rs:4-5`
prohibit UUID v4 for RocksDB iteration ordering reasons. A client sending a v4 UUID in
`X-Org-Id` creates an OrgId that violates the architecture invariant. While this does
not cause a security breach in isolation, it weakens the monotonic-ordering guarantee
relied on by the storage layer.

**Proposed Mitigation:**

Use `OrgId::from_uuid_v7` (if it exists) or add a version check in the `extract_org_id`
helper that returns the sentinel/fallback when the UUID is not version 7.

**Remediation Routing:** implementer (DTU route helpers)

---

### SEC-009: `dtu_reset_for` Path Parameter Echoed Back in Error Response Without Sanitization

- **Severity:** LOW
- **CWE:** CWE-79 (Improper Neutralization of Input During Web Page Generation — Cross-Site Scripting)
- **File:** `crates/prism-dtu-claroty/src/routes/devices.rs:364-370`

**Description:**

The `POST /dtu/reset_for/{org_id}` route reflects the raw `org_id_str` path parameter
in its error response body:

```rust
return (
    StatusCode::BAD_REQUEST,
    Json(
        json!({"error": format!("invalid org_id: {org_id_str:?} is not a valid UUID")}),
    ),
);
```

The `:?` debug format adds Rust's escaping but the value is reflected into a JSON
string. While this is a test-harness endpoint (not a browser-facing surface), reflected
content in JSON responses can create issues if the response is consumed by a logging
system that renders JSON fields as HTML. The format should be truncated or sanitized to
prevent excessively long reflections.

**Proposed Mitigation:**

Truncate `org_id_str` to 64 characters in the error message, or replace the reflection
with a generic message: `"invalid org_id: value is not a valid UUID"`.

**Remediation Routing:** implementer (prism-dtu-claroty)

---

### SEC-010: `admin_token` Stored as Plain `String` in DTU Clone State — Not Zeroized on Drop

- **Severity:** LOW
- **CWE:** CWE-312 (Cleartext Storage of Sensitive Information)
- **Files:**
  - `crates/prism-dtu-armis/src/clone.rs:49` (`admin_token: String`)
  - `crates/prism-dtu-claroty/src/clone.rs` (implied same pattern)
  - `crates/prism-dtu-crowdstrike/src/clone.rs:41` (`admin_token: String`)

**Description:**

The `admin_token` (a UUID v4 string used to authenticate `POST /dtu/configure` calls)
is stored as a plain `String` in the DTU clone's `Arc<State>`. It is not wrapped in
`secrecy::SecretString` and will not be zeroed from heap memory on drop.

This follows the same pattern as WGS-W2-002 from Wave 2 (derived bearer tokens stored
as plain strings). The admin token is generated per-clone at startup, never persisted,
and lives only for the duration of a test run. Its exposure window is bounded to test
execution. Severity is LOW because the token is a random UUID v4 used only in the test
harness, not a customer credential.

**Proposed Mitigation:**

Wrap `admin_token` in `SecretString` with `expose_secret()` at the single comparison
site in `dtu_configure`. This is a low-effort change that maintains consistency with
the credential handling patterns established in Wave 2.

**Remediation Routing:** implementer (prism-dtu-armis, prism-dtu-claroty,
prism-dtu-crowdstrike)

---

## Dependency Advisory Assessment

### wasmtime 44.0.1

RUSTSEC-2026-0114 (wasmtime 44.0.0) has been remediated. Cargo.lock confirms
`wasmtime = "44.0.1"` at checksum
`372db8bbad8ec962038101f75ab2c3ffcd18797d7d3ae877a58ab9873cd0c4bd`. No open advisories
for 44.0.1 found. Status: **CLEAR**.

### bimap 0.6.3

New dependency introduced in Wave 3 (S-3.1.03 — OrgRegistry). Version 0.6.3 at
checksum `230c5f1ca6a325a32553f8640d31ac9b49f2411e901e427570154868b46da4f7`. No RustSec
advisories found for bimap 0.6.x. The crate provides bijective HashMap semantics with
no unsafe code in its public surface. Status: **CLEAR**.

### openssl 0.10.78, ring 0.17.14, rustls 0.23.40

No open advisories for these versions as of the review date. Status: **CLEAR**.

---

## CI Workflow and Secret Hygiene Assessment

The `ci.yml` workflow was modified in W3-FIX-CI-001. All GitHub Actions steps are
pinned to full commit SHAs (not branch names or version tags):

- `actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd`
- `dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8`
- `arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b`
- `Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4`
- `EmbarkStudios/cargo-deny-action@91bf2b620e09e18d6eb78b92e7861937469acedb`
- `taiki-e/install-action@cf525cb33f51aca27cd6fa02034117ab963ff9f1`
- `rui314/setup-mold@9c9c13bf4c3f1adef0cc596abc155580bcb04444`
- `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02`

All actions are SHA-pinned. No floating `@main`, `@master`, or semantic version pins
exist. `GITHUB_TOKEN` is used only for `setup-protoc` rate limiting, which is the
correct and expected use. No customer credentials, API keys, or secrets are referenced
in the workflow file. Assessment: **CLEAN — no workflow secret leakage risk**.

---

## W3-FIX-WIN-001 Cross-Platform Port-Release Assessment

`allocate_network_listeners` in `crates/prism-dtu-harness/src/builder.rs:539-562`
uses synchronous `std::net::TcpListener::bind("127.0.0.1:0")` in a single
pre-allocation pass. All OS port bindings complete before any async clone startup
begins. On clone startup failure, all previously-allocated listeners are explicitly
`drop`ped, releasing their OS ports atomically.

The OS assigns ports via `SO_REUSEADDR`-equivalent behavior on Windows (which differs
from POSIX); by pre-binding with `bind("127.0.0.1:0")` and passing the live listener
to the async server, there is no bind-drop-rebind race. The pattern is sound on all
three target platforms (Linux, macOS, Windows). Assessment: **CORRECT — no race
condition**.

---

## Tenant ID Spoofing at MCP Boundary (HS-003-02)

Per the task scope note: "MCP boundary not yet exists." The MCP tool layer is not
implemented in Wave 3; no MCP tool parameters exist through which an analyst could
override their authenticated identity. The DTU HTTP layer accepts org headers without
auth verification (SEC-001), but this is behind the loopback network harness, not
directly accessible from the MCP tool context. When the MCP boundary is implemented,
the mechanism by which `OrgId` is established for a given tool invocation must be
authenticated server-side, not derived from tool parameters. This is a forward-looking
note, not a current finding.

---

## Risk Register Dispositions (Security-Category R-NNN Entries)

The Wave 3 domain spec does not define an explicit R-NNN Risk Register in the reviewed
artifacts. The following security risks implicit in the ADR threat model are assessed
against the implementation:

| ADR Risk | Mitigation Status | Notes |
|----------|------------------|-------|
| Cross-tenant data leakage at adapter layer (ADR-006 §3.1) | **Partially Mitigated** | State-layer `HashMap<(OrgId, String), V>` keying is correct and tested. HTTP-layer org header trust (SEC-001) creates a residual gap in the network harness. |
| Cross-tenant credential reachability (ADR-006 §3.2) | **Partially Mitigated** | `namespace_key_by_org_id` is correct. `CredentialStoreOrgId::get_by_org` has stub implementation in KeyringBackend (SEC-004). |
| Slug squatting / namespace collision (ADR-006 §3.4) | **Mitigated** | `OrgRegistry::register` enforces bijectivity with typed errors. `validate_all` detects duplicate slugs and UUIDs before any registration. |
| Slug rename forensics (ADR-006 §3.3) | **Mitigated** | Audit entry carries non-nullable `org_id` (UUID) and `org_slug` (denormalized); `static_assertions::assert_fields!` enforces this at compile time. |
| Privacy in shared-infrastructure DTU (ADR-006 §3.5) | **Mitigated** | BC-3.3.001 enforces that Security Telemetry types cannot be mode=shared in config. |
| Path traversal in spec file loading (not in ADR but implied by R-CUST-014) | **Unmitigated** | SEC-003 describes this gap. No path canonicalization or boundary check exists. |

---

## Verdict and Conditions

**APPROVED WITH CONDITIONS**

Wave 3 delivers a structurally sound multi-tenant identity foundation. The OrgId/OrgSlug
separation, bijectivity enforcement, OrgId-keyed credential namespace, and audit field
non-nullability are all correctly implemented and exceed the security posture of Wave 2.
No CRITICAL findings were identified.

The three HIGH findings (SEC-001, SEC-002, SEC-003) must be resolved before production
deployment carrying real customer credentials:

1. **SEC-001** (X-Org-Id header spoofability) — blocks network-harness isolation assurance.
2. **SEC-002** (`POST /dtu/reset` unauthenticated on four clones) — blocks test-harness
   state integrity during parallel multi-tenant test runs.
3. **SEC-003** (spec path traversal) — blocks operator safety when loading customer TOML
   files from untrusted sources.

MEDIUM findings (SEC-004 through SEC-007) and LOW findings (SEC-008 through SEC-010)
are tracked as technical debt and do not block wave progression, but should be addressed
before the Phase 4 holdout evaluation.
