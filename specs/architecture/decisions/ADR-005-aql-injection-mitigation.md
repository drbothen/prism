---
document_type: adr
adr_id: ADR-005
title: "AQL Injection Mitigation — Armis Adapter Query Trust Model"
status: accepted
date: 2026-04-26
version: "0.3"
subsystems_affected: [SS-01]
supersedes: null
superseded_by: null
security_finding: WGS-W2-001
cwe: CWE-943
owasp: A03:2021
inputs:
  - crates/prism-sensors/src/auth/armis.rs (build_aql, line 116-124)
  - .factory/cycles/phase-3-dtu-wave-2/gate-step-d-security-review.md (WGS-W2-001)
  - .factory/specs/behavioral-contracts/BC-2.01.008-armis-bearer-aql.md
  - .factory/specs/architecture/security-architecture.md (threat model)
traces_to: specs/architecture/ARCH-INDEX.md
runtime_deliverables:
  - prism-sensors::auth::armis::validate_aql  # pure AQL allowlist validator function (ADR-005 primary control)
  - prism-sensors::auth::armis::AqlValidationError  # structured rejection error type
  - prism-sensors::adapter::SensorError::ConfigValidation  # new error variant for pre-wire rejection
wiring_deferred_to: null  # All three deliverables confirmed implemented in crates/prism-sensors (W2-FIX-I merged)
---

# ADR-005: AQL Injection Mitigation — Armis Adapter Query Trust Model

## Status

ACCEPTED — PO decisions on all three open questions recorded below (2026-04-26).
W2-FIX-I implementation is cleared to proceed.

## Context

### The Vulnerability

`ArmisAdapter::build_aql()` in `crates/prism-sensors/src/auth/armis.rs:116-124`
reads `spec.sensor_config["aql_query"]` and forwards it verbatim to the Armis
GetSearch API. A comment at line 119 explicitly marks this as unsanitized. The
value originates from a `SensorSpec.sensor_config` JSON blob — a `serde_json::Value`
populated from TOML sensor spec content loaded by `prism-spec-engine`.

This is CWE-943 (Improper Neutralization of Special Elements in Data Query Logic).

### The MSSP Multi-Tenant Threat Model

Prism is a multi-tenant process: multiple analysts share a single MCP server
instance, and per-client credentials live in an OS keyring namespace. The security
architecture (`security-architecture.md`) documents cross-client data leakage as
a primary threat (T2), mitigated by the `TenantId` newtype at the storage layer.
However, the AQL injection vector bypasses that layer entirely — a malicious or
misconfigured query reaches the Armis API before `TenantId` enforcement has any
effect.

### How `aql_query` Enters the System

Two distinct entry paths exist today:

1. **Spec-author path (TOML):** An operator or spec author writes an `.sensor.toml`
   file containing `aql_query = "..."` in a table config stanza. This content is
   loaded by `parse_and_validate_spec_toml` in `prism-spec-engine/src/add_sensor_spec.rs`
   and stored as an opaque `sensor_config: serde_json::Value`. No AQL validation
   occurs during spec loading. This is the current production code path.

2. **Filter push-down path (PrismQL → AQL translation):** BC-2.11.007 specifies
   that PrismQL WHERE predicates are translated into Armis AQL WHERE clauses by
   the push-down layer. This path generates AQL programmatically from typed
   PrismQL AST nodes — it does not pass through `build_aql`'s `sensor_config`
   lookup. This path is NOT in scope for this ADR, but the distinction matters
   for mitigation design.

3. **Runtime upload via `add_sensor_spec` MCP tool (BC-2.16.008):** Analysts with
   sufficient permissions can upload a new `.sensor.toml` at runtime via the MCP
   tool. This means the TOML path is not restricted to pre-deployment PR review;
   a live analyst session can inject `aql_query` content.

### Why "No Sanitization" Is Unsafe

AQL supports operators that reach beyond the caller's intended scope: free-form
`WHERE` clauses, `AND NOT site:` exclusions, field projections that bypass column
pruning, and sub-queries. In a shared Armis account model (multiple MSSP clients
under one Armis organization), a crafted AQL can enumerate or exfiltrate records
belonging to other clients. Armis's own API-level input validation is not a
reliable backstop — it validates syntax, not semantic scope.

Even in a dedicated Armis account per client, injection can be used to: (a) read
fields beyond what the sensor spec intends to expose, (b) bypass pagination and
rate-limit budgets by issuing unbounded queries, and (c) exfiltrate data via the
HTTP 400 error response that BC-2.01.008 mandates the error message include the
AQL string (TV-BC-2.01.008-005 reveals the query to logs).

### Why the Existing Mitigations Are Insufficient

The `TenantId` newtype, dual semaphores (BC-2.01.012), and audit pipeline address
parallel-but-different risks. They do not constrain the content of the AQL string
that reaches the Armis wire.

---

## Decision

Adopt **Option C — Hybrid trust-boundary enforcement**:

- **For AQL that enters via the TOML / `add_sensor_spec` path** (the current
  production path): apply a *syntax-shape allowlist validator* at the point of
  spec load and at `build_aql()` call time. The validator is a small hand-rolled
  tokenizer (no parser-combinator crate) that verifies the AQL string conforms to
  a curated operator and field-name allowlist. Queries that fail validation are
  rejected with a structured `SensorError::ConfigValidation` before the HTTP call
  is issued.

- **For AQL generated by the push-down layer** (PrismQL → AQL translation,
  BC-2.11.007): no runtime validation is added. The push-down layer produces AQL
  programmatically from a typed AST; the resulting string is known-safe by
  construction. It does not flow through `build_aql`'s `sensor_config` branch.

- **Audit logging** at `HIGH` severity (using the existing `prism-audit`
  pipeline) whenever the `sensor_config["aql_query"]` branch executes in
  `build_aql()`, logging `(client_id, sensor_id, table, aql_query_hash,
  aql_query_preview_64_chars, validation_outcome)`. This provides a forensic
  trail even when validation passes. Do NOT log the full AQL string — log a
  SHA-256 hash plus a 64-character truncated prefix; the full string appears
  in the `SensorError` on rejection (which is already collected by the audit
  pipeline).

---

## Rationale

Option C is chosen because it maps precisely to the two actual trust boundaries
present in the Prism sensor architecture:

| AQL origin | Trust level | Mitigation |
|---|---|---|
| TOML spec, `add_sensor_spec` MCP tool | Analyst-controlled at runtime | Allowlist validator + audit |
| PrismQL push-down (BC-2.11.007) | Generated from typed AST | No validation needed |

This is the least-surprising invariant: *"any AQL that came from human-authored
text is validated; any AQL we generated ourselves is not."*

**Option A (allowlist-only, applied universally) was rejected for the push-down
path.** The push-down layer (BC-2.11.007) generates AQL programmatically from a
typed PrismQL AST — the resulting string is safe by construction. Applying the
allowlist validator to that path adds latency to the sensor-fetch hot path for
zero security gain. The allowlist is retained for the `sensor_config` path where
it provides real protection.

**Option B (verbatim + audit-only) was rejected as the primary defense.** The
MSSP multi-tenant context makes relying on Armis API-level input validation
insufficient — Armis validates syntax, not semantic scope. Option B provides
forensic traceability after the fact but does not block injection before the wire.
Critically, `add_sensor_spec` (BC-2.16.008) is an MCP tool callable by an analyst
at runtime, not only a PR-reviewed TOML loaded at deployment time. The threat
surface is live-session injection, making a pre-wire rejection essential. Audit
logging is retained as a secondary layer within Option C.

**Option D (HMAC signing) was rejected** as it adds key-management complexity for
a non-cryptographic problem. An HMAC proves that the AQL string came from a
particular signer; it does not constrain what the AQL string says or does.

---

## AQL Allowlist Scope

The validator must be intentionally minimal. The Armis GetSearch API's full AQL
grammar is broad. The sensor specs shipped with Prism (CrowdStrike, Claroty,
Armis, Cyberint built-in specs) use a narrow subset. The allowlist targets that
subset.

### Permitted constructs

- `in:{table}` — the default template (table name from `source_table`)
- `in:{table} where {FIELD} {OP} {VALUE}` — simple predicate
- Field names matching `[a-zA-Z][a-zA-Z0-9_.]*` (no whitespace, no parens, no
  subquery markers)
- Operators: `=`, `!=`, `<`, `>`, `<=`, `>=`, `like`, `in`
- Value shapes: quoted string literals, unquoted integer/float literals, and
  parenthesized comma-separated lists of the above (for `in` operator)
- Logical combinators: `and`, `or`, `not` (case-insensitive)
- `orderBy {FIELD} {asc|desc}` suffix
- Maximum total length: 512 bytes (prevents unbounded queries)

### Rejected constructs (examples, not exhaustive)

- Sub-queries (presence of `(select`, `(in:`, nested `in:` outside the leading
  position)
- SQL-style comments (`--`, `/*`)
- Stacked queries (`;`)
- Excessively long strings (> 512 bytes)
- Field names containing path-traversal characters or SQL keywords

The validator does not need to parse full AQL grammar — it needs to reject
anything outside the curated subset. Unknown constructs are rejected by default
(allowlist, not blocklist).

---

## Consequences

### Positive

- Closes WGS-W2-001 / CWE-943. Injection from operator-supplied AQL is blocked
  at `build_aql()` call time, before the HTTP semaphore is acquired.
- Audit trail for every `sensor_config["aql_query"]` execution provides forensic
  evidence if a malformed spec reaches production.
- No latency regression on the push-down path — the validator is only called when
  `sensor_config["aql_query"]` is present.
- Allowlist is small enough to maintain in a single source file with no
  external crate dependencies.

### Negative / Trade-offs

- Spec authors who use advanced AQL constructs outside the allowlist will receive
  a validation error. They must either simplify the query or request an allowlist
  extension via a PR.
- The allowlist must be updated when Prism's built-in sensor specs evolve to use
  new AQL operators. This is low-frequency but must be tracked.
- Two code paths exist in `build_aql()`: the validated `sensor_config` branch and
  the un-validated default-template branch. The default-template branch
  (`DEFAULT_AQL_TEMPLATE`) is safe by construction (it only substitutes
  `spec.source_table`, which is validated at spec load time) and does not need
  validation.

### Follow-on Work

| Item | Type | Triggered by |
|------|------|-------------|
| Implement `validate_aql()` in `armis.rs` | Story (W2-FIX-I) | This ADR |
| Add HIGH-severity audit event for `aql_query` branch in `build_aql()` | Story (W2-FIX-I) | This ADR |
| Add `TV-BC-2.01.008-006` (pre-wire `ConfigValidation` rejection) to BC-2.01.008 | BC update (W2-FIX-I) | Q3 decision |
| Add VP for `validate_aql()` rejection of disallowed constructs | VP (W2-FIX-I or Wave 3) | Q1 decision (primary control) |
| Add `@1898co/security` CODEOWNERS entry for `crates/prism-sensors/src/auth/` | TD-ADR005-001 (P2) | Q2 decision |
| Validate `source_table` at spec-load time to harden the default-template branch | Enhancement | Security review |

---

## Implementation Pointer

The implementer for W2-FIX-I must touch these specific locations:

### 1. New function: `validate_aql()`

**File:** `crates/prism-sensors/src/auth/armis.rs`

Add a pure function (no I/O, no `&self`) before `build_aql()`:

```rust
/// Validates an operator-supplied AQL string against the Prism allowlist.
///
/// Called for AQL that originates from `SensorSpec.sensor_config["aql_query"]`
/// (i.e., operator-authored TOML or runtime `add_sensor_spec` uploads).
/// NOT called for push-down-generated AQL (BC-2.11.007), which is safe by
/// construction.
///
/// Returns `Ok(())` if the query passes, `Err(AqlValidationError)` otherwise.
/// ADR-005.
pub fn validate_aql(query: &str) -> Result<(), AqlValidationError> {
    // ... hand-rolled tokenizer / regex shape check ...
}

#[derive(Debug, thiserror::Error)]
#[error("AQL validation failed: {reason}")]
pub struct AqlValidationError {
    pub reason: String,
}
```

### 2. Call site: `build_aql()`

**File:** `crates/prism-sensors/src/auth/armis.rs:116-124`

Replace the verbatim-return branch:

```rust
// Current (WGS-W2-001 vulnerability):
return aql.to_string();

// After ADR-005:
validate_aql(aql).map_err(|e| /* convert to SensorError::ConfigValidation */)?;
// emit HIGH-severity audit event here (client_id from spec.client_id)
return aql.to_string();
```

`SensorError::ConfigValidation` is a new variant to add to the enum in
`crates/prism-sensors/src/adapter.rs`.

### 3. Audit emission

Use the existing `prism-audit` pipeline. The audit event type is
`AuditEventKind::SensorQueryValidated` (or reuse `CredentialAccess` if a new
variant is out of scope for W2-FIX-I). Fields: `client_id`, `sensor: "armis"`,
`table`, `aql_hash: sha256(aql)[0..16]`, `aql_preview: &aql[..64.min(aql.len())]`,
`validation_outcome: "pass" | "reject"`, `reason: Option<String>`.

### 4. Spec-load validation (optional hardening, separate story)

`prism-spec-engine/src/add_sensor_spec.rs:parse_and_validate_spec_toml()` could
call `validate_aql()` for any `aql_query` field encountered during TOML parse.
This would surface the error at upload time rather than at first fetch. Deferred
to a separate story to keep W2-FIX-I focused.

---

## Alternatives Considered

| Option | Summary | Decision |
|--------|---------|---------|
| A — Allowlist-only, applied universally | Simplest, defense-in-depth, but adds hot-path latency to push-down-generated AQL | Rejected for push-down path; adopted for sensor_config path |
| B — Verbatim forwarding + audit | Zero complexity, full AQL feature support, forensic trace | Rejected as primary defense; audit retained as secondary layer |
| C — Hybrid (chosen) | Allowlist for sensor_config branch; no validation for push-down branch; audit on sensor_config branch | Accepted |
| D — Signed AQL (HMAC over spec-author-supplied AQL, verified at build_aql time) | Strong provenance, tamper-evident | Rejected: adds key-management complexity for a non-cryptographic problem; HMAC only proves authenticity, not semantic safety |

---

## Open Questions (PO Sign-Off Required)

1. **Spec-author trust classification:** Should `add_sensor_spec` (BC-2.16.008)
   be restricted to a privileged analyst role so that runtime TOML uploads cannot
   originate from an untrusted session? If yes, the allowlist validator is a
   defense-in-depth layer; if no, it is the primary control. This affects how
   strictly the allowlist must be enforced and whether a formal VP is warranted.

2. **Allowlist extension process:** When a legitimate Armis sensor spec needs an
   AQL construct outside the initial allowlist, what is the approval process?
   PR review by whom? This must be codified before W2-FIX-I ships.

3. **Error exposure:** BC-2.01.008 TV-BC-2.01.008-005 states that on AQL syntax
   error (HTTP 400), the error message must include the AQL query text. If we
   reject the query at `build_aql()` before the HTTP call, the structured error
   will also include the AQL text (for diagnostics). Confirm this is acceptable
   given the audit pipeline will also log it.

---

## PO Decisions (2026-04-26)

### Q1 — `add_sensor_spec` Access Control

The `add_sensor_spec` MCP tool is **not** restricted to a separate privileged-analyst
role. It is gated by the `sensor_spec.write` per-client capability flag in the
analyst's TOML config (BC-2.16.008, BC-2.04.005). Any analyst whose configuration
grants `sensor_spec.write = true` for at least one client can invoke the tool in
their MCP session. There is no separate "spec-author tier" above the normal
trusted-analyst identity. This is consistent with the Prism deployment model:
one process per analyst, analyst is a trusted MSSP employee (ASM-006, ASM-007,
ASM-011), and per-analyst scoping is managed by providing different TOML configs —
not by role-based access control within a single session.

Consequence for the ADR: the allowlist validator is the **primary runtime
enforcement control**, not a defense-in-depth layer behind a human review gate.
The validator must be enforced strictly. A formal VP covering `validate_aql()`
rejection is warranted (see Follow-on Work table). There is no story-level
precondition blocking W2-FIX-I; the validator implementation IS the access control.

### Q2 — Allowlist Extension PR Gate

The allowlist is a **code-level constant** in `crates/prism-sensors` (a dedicated
`aql_allowlist.rs` module or inline in `armis.rs`). It is not config-driven.
This is intentional: making it config-driven would allow allowlist expansion
without code review, undermining its security value.

Any extension to the allowlist requires a PR to `crates/prism-sensors`. Under the
current CODEOWNERS stub (`* @1898co/prism-core`), all PRs already require
`@1898co/prism-core` review. This is the minimum sufficient gate for now.

However, a dedicated `@1898co/security` CODEOWNERS entry for
`crates/prism-sensors/src/auth/` is not yet defined. A TD entry
(`TD-ADR005-001`) is filed to add this before production deployment: the security
reviewer role should be a required approver for any change to `aql_allowlist.rs`
or `validate_aql()`. Implementers writing W2-FIX-I must document in the
PR description that the allowlist scope is intentionally minimal (covering only
the four built-in sensor specs: CrowdStrike, Cyberint, Claroty, Armis).

### Q3 — Error-Message AQL Exposure

The two cases are distinct and must be handled differently:

**Case A — Armis HTTP 400 (AQL reached the wire, Armis rejected it):** This is
the scenario covered by TV-BC-2.01.008-005. AQL that passes `validate_aql()` can
still produce an HTTP 400 if Armis rejects it for reasons the allowlist does not
catch. The existing contract — include AQL text in the `PrismError::Sensor`
`category: "api_contract"` error — is **preserved**. This error is surfaced to
the analyst for debugging and the AQL has already transited the wire, so
suppressing it from the local error adds no security benefit.

**Case B — Pre-wire `ConfigValidation` rejection (validate_aql() fails):** The
proposed `SensorError::ConfigValidation { sensor, detail }` carrying the rejected
AQL in `detail` is **acceptable**. The analyst population is trusted MSSP
employees (ASM-006); the threat model for "analyst probing the allowlist via
repeated 400s" is weak — a trusted employee who can write TOML already has full
read access to the `validate_aql()` source code in the repository. Including the
AQL text aids legitimate debugging of spec authoring errors.

The audit pipeline (hash + 64-char preview logged at HIGH severity on every
`sensor_config["aql_query"]` branch execution) is sufficient forensic coverage.
A new test vector **TV-BC-2.01.008-006** must be added to BC-2.01.008 documenting
the pre-wire `ConfigValidation` rejection case: input = AQL containing a
sub-query construct; expected = `SensorError::ConfigValidation` with `detail`
containing the rejected AQL and the validation failure reason; no HTTP call issued.
This TV addition is in scope for W2-FIX-I.

---

## Source / Origin

- **Security finding:** WGS-W2-001 (HIGH, CWE-943) — Wave 2 integration gate
  Step D security review, 2026-04-26
- **Vulnerable code:** `crates/prism-sensors/src/auth/armis.rs:116-124`,
  `build_aql()`, comment "Return VERBATIM — no modification, sanitization, or
  injection prevention."
- **Behavioral contract:** BC-2.01.008 (AQL forwarding postcondition)
- **Security architecture:** `security-architecture.md` threat model, T2
  (cross-client data leakage)

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.2 | 2026-04-26 | product-owner | PO decisions on Q1/Q2/Q3; status PROPOSED → ACCEPTED; added TD-ADR005-001 (CODEOWNERS security reviewer) |
| 0.1 | 2026-04-26 | architect | Initial ADR — closes WGS-W2-001 (HIGH, CWE-943) |
