---
document_type: behavioral-contract
level: L3
version: "0.3"
status: draft
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs:
  - ".factory/specs/architecture/decisions/ADR-010-customer-config-schema.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "280b5e7"
traces_to: ".factory/specs/architecture/decisions/ADR-010-customer-config-schema.md"
origin: greenfield
extracted_from: null
subsystem: "SS-06"
capability: "CAP-009"
lifecycle_status: active
introduced: wave-3
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.3.002
title: No Credential Values in Customer Config Files
wave: 3
phase: 3.A
date: 2026-04-27
authors: [product-owner]
related_decisions: [D-041, D-046]
related_adrs: [ADR-010]
inherits_from: null
superseded_by: null
---

# BC-3.3.002: No Credential Values in Customer Config Files

## Description

Customer config TOML files (`customers/*.toml`) MUST NOT contain literal credential values — bearer tokens, API keys, passwords, or secrets of any kind. The startup validator performs static analysis of each parsed config to detect fields that appear to contain credential values rather than opaque reference strings. Any field whose name matches a credential-like pattern (e.g., `*_token`, `*_secret`, `*_key`, `*_password`) and whose value does not match one of the four allowed opaque reference schemes is rejected at parse time, before the process starts or any AI context window sees the file.

## Preconditions

1. The Prism startup validator is processing a `customers/*.toml` file (same pass as BC-3.3.001).
2. The four allowed opaque reference scheme prefixes are known to the validator: `vault://`, `env://`, `file://`, `keyring://`.
3. The credential-field heuristic pattern set is compiled into the binary (not user-configurable).

## Postconditions

**On credential value detected:**

1. The process exits with code `1` before accepting any MCP connections.
2. Stderr contains error code `E-CFG-020` with: the offending filename, the field name, and a message stating the field looks like a literal credential.
3. The detected value itself is NEVER echoed in the error message (to prevent credential exposure in logs).
4. `OrgRegistry` contains zero entries.

**On valid config (opaque refs only):**

1. All `credential_ref` fields contain strings matching one of the four allowed schemes.
2. No field whose name matches a credential-like pattern contains a non-scheme string value.
3. Validation passes; processing continues to BC-3.3.001 rule checks.

**Credential-field heuristic rules:**

| Rule | Pattern | Trigger Condition | Error Code |
|------|---------|-------------------|------------|
| R-CRED-001 | Field name ends in `_token` | Value is non-empty string not starting with a scheme prefix | `E-CFG-020` |
| R-CRED-002 | Field name ends in `_secret` | Value is non-empty string not starting with a scheme prefix | `E-CFG-020` |
| R-CRED-003 | Field name ends in `_key` | Value is non-empty string not starting with a scheme prefix | `E-CFG-020` |
| R-CRED-004 | Field name ends in `_password` | Value is non-empty string not starting with a scheme prefix | `E-CFG-020` |
| R-CRED-005 | Field name is exactly `password` | Value is non-empty string not starting with a scheme prefix | `E-CFG-020` |
| R-CRED-006 | Field name ends in `_pass` | Value is non-empty string not starting with a scheme prefix | `E-CFG-020` |
| R-CRED-007 | `credential_ref` field | Value is empty string OR does not start with one of the four allowed scheme prefixes | `E-CFG-005` (scheme validation, covered also in BC-3.3.004 R-CUST-005) |

Error message format: `customers/<file>.toml: E-CFG-020: field '<name>' appears to contain a literal credential; use a credential_ref with an opaque reference scheme (vault://, env://, file://, keyring://) instead`

Note: The field value is intentionally NOT included in the error message to prevent the credential from being written to operator logs.

## Invariants

1. No credential value ever transits the AI context window via a `customers/*.toml` file — the file may only contain opaque reference strings (ADR-010 §3.1 threat mitigation).
2. The heuristic patterns are applied to ALL fields at ALL nesting levels in the TOML document (top-level, `[[dtu]]`, `[dtu.data]`, `[shared_infra]`).
3. The error message MUST NOT reproduce the field's value (defense-in-depth against log exposure).
4. False positives (a field named `*_key` containing a path like `keyring://...`) are accepted as valid — scheme-prefixed values always pass.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-3.3.002-01 | `bearer_token = "abc123"` | `E-CFG-020`; field name `'bearer_token'` reported; value `"abc123"` NOT in message |
| EC-3.3.002-02 | `api_key = "keyring://prism/acme/claroty"` | Passes; value starts with `keyring://` scheme prefix |
| EC-3.3.002-03 | `api_key_ref = "keyring://prism/acme/claroty"` | Passes; field name ends in `_ref` not a credential pattern; value is a valid scheme |
| EC-3.3.002-04 | `password = "hunter2"` | `E-CFG-020`; field name `'password'` reported; value NOT in message |
| EC-3.3.002-05 | `client_secret = "vault://sensors/acme/cs/secret"` | Passes; value starts with `vault://` scheme prefix |
| EC-3.3.002-06 | `[shared_infra] slack_channel = "#acme-alerts"` | Passes; `slack_channel` does not match credential-like pattern |
| EC-3.3.002-07 | `[shared_infra] api_key = "abc"` (no scheme) | `E-CFG-020`; `[shared_infra]` nesting does not exempt the field |
| EC-3.3.002-08 | `credential_ref = ""` (empty string) | `E-CFG-005` (scheme validation from BC-3.3.001); empty value is not a valid scheme |

## Canonical Test Vectors

| TV-ID | Input | Expected Output | Category |
|-------|-------|-----------------|----------|
| TV-3.3.002-01 | `customers/acme.toml` with `bearer_token = "abc123"` in `[[dtu]]` | Exit 1; stderr contains `E-CFG-020`; names `'bearer_token'`; does NOT contain `"abc123"` | error |
| TV-3.3.002-02 | `customers/acme.toml` with `credential_ref = "keyring://customer/acme/sensor/claroty"` | Exit 0 (for this field); credential ref passes scheme validation | happy-path |
| TV-3.3.002-03 | `customers/acme.toml` with `password = "hunter2"` anywhere in file | Exit 1; stderr contains `E-CFG-020`; names `'password'`; does NOT echo `"hunter2"` | error |
| TV-3.3.002-04 | `customers/acme.toml` with `api_key_ref = "keyring://prism/acme/sensor"` | Exit 0 (for this field); `_ref` suffix with valid scheme passes | happy-path |
| TV-3.3.002-05 | `customers/acme.toml` with `client_secret = "vault://sensors/acme/cs/secret"` | Exit 0 (for this field); `vault://` scheme prefix makes it a valid opaque ref | happy-path |
| TV-3.3.002-06 | `customers/acme.toml` with `[shared_infra] api_key = "rawvalue"` | Exit 1; `E-CFG-020`; pattern matching applies to nested `[shared_infra]` block | error |
| TV-3.3.002-07 | `customers/acme.toml` with `credential_ref = "file:///etc/prism/acme-key"` | Exit 0; `file://` scheme is allowed | happy-path |
| TV-3.3.002-08 | `customers/acme.toml` with `credential_ref = "env://ACME_CLAROTY_KEY"` | Exit 0; `env://` scheme is allowed | happy-path |

## Verification Properties

| VP | Property | Proof Method |
|----|----------|--------------|
| VP-3.3.002-A | For all config files, if any field matching a credential-name pattern contains a non-scheme value, exit code is 1 | proptest over field-name generator |
| VP-3.3.002-B | Error message for `E-CFG-020` never contains the literal field value (no value echo) | manual / static analysis of error formatter |
| VP-3.3.002-C | All four scheme prefixes are accepted for all credential-pattern field names | parameterized unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 |
| Capability Anchor Justification | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 — this BC enforces the AI-opaque credential reference model during config load, which is the security constraint on the "credential references" component of CAP-009's "Load and validate per-client sensor mappings, credential references, and capability overrides from TOML configuration." |
| L2 Domain Invariants | N/A (Wave 3 new capability; DI-NNN assignment pending domain-spec Wave 3 extension) |
| Architecture Module | SS-06 (Client Configuration) per ARCH-INDEX.md |
| Stories | S-3.3.01 |

## Related BCs

- BC-3.3.001 — composes with (credential scheme check is one rule in the full validation pass)
- BC-3.3.003 — related to (same validation pass; schema_version check)

## Architecture Anchors

- ADR-010 §2.3.1 — Credential Reference Schemes table; four allowed scheme prefixes
- ADR-010 §3.1 — Threat model: config file as credential exfiltration vector
- `crates/prism-credentials/src/namespace.rs:20` — `namespace_key(tenant, sensor, name)` — operator-facing counterpart to opaque reference model

## Story Anchor

S-3.3.01

## VP Anchors

- VP-3.3.002-A — proptest: credential-name field with non-scheme value always fails
- VP-3.3.002-B — static: error formatter never echoes field value
- VP-3.3.002-C — unit: all four schemes pass for all credential-pattern field names

## BC Changelog

| Version | Change |
|---------|--------|
| v0.3 | M-004 (Pass 5): Frontmatter `title:` corrected to title-case to match H1 heading. |
| v0.2 | Initial authoring from ADR-010. |
