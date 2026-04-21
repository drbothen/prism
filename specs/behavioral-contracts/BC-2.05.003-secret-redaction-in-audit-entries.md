---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-05"
capability: "CAP-007"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "b1e4604"
traces_to: ["CAP-007"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.05.003: Credential Values Are Never Present in Audit Entries

## Description

When the audit middleware serializes the `parameters` field of an `AuditEntry`, all
credential values are replaced with `"[REDACTED]"`. Credential names (e.g., `"api_key"`,
`"client_secret"`) are preserved for traceability; only the values are redacted. Redaction
applies to all known secret field patterns (fields ending in `_key`, `_secret`, `_token`,
`_password`, `_credential`, and any field resolved via the credential store) and is applied
recursively at any nesting depth.

No credential value appears in `result_summary`, `capability_checks`, `safety_flags`, or
any other audit entry field. Unrecognized secret fields are a code-level bug to be caught
by review and test.

## Preconditions
- An MCP tool invocation includes parameters that reference or contain credential material (API keys, tokens, secrets)
- The audit middleware is serializing the `parameters` field of the `AuditEntry`

## Postconditions
- The `parameters` field in the audit entry replaces all credential values with a redaction marker (e.g., `"[REDACTED]"`)
- Credential names (e.g., `"api_key"`, `"client_secret"`) are preserved for traceability -- only the values are redacted
- The redaction applies to all known secret field patterns: fields ending in `_key`, `_secret`, `_token`, `_password`, `_credential`, and any field resolved via the credential store
- No credential value appears in `result_summary`, `capability_checks`, `safety_flags`, or any other audit entry field

## Invariants
- DI-002: Credential isolation per client -- credential values are never included in logs

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Unrecognized secret field | A new tool parameter contains a secret in a field not matching known patterns | The value is logged unredacted; this is a code-level bug caught by review and test. Known patterns must be updated when new secret fields are introduced. |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-004 | Tool parameter contains a nested JSON object with a secret buried in a sub-field | Redaction applies recursively to all fields matching secret patterns at any nesting depth |
| EC-05-005 | Tool parameter value coincidentally looks like a secret pattern name (e.g., a hostname containing `_token`) | Only fields whose names match secret patterns are redacted; values containing pattern substrings are not modified |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.05.003.

| Scenario | Input Parameter | Audit Entry `parameters` |
|----------|----------------|--------------------------|
| Top-level secret | `{api_key: "secret123"}` | `{api_key: "[REDACTED]"}` |
| Nested secret | `{config: {client_secret: "abc"}}` | `{config: {client_secret: "[REDACTED]"}}` |
| Non-secret field name | `{hostname: "api_token_server.example.com"}` | `{hostname: "api_token_server.example.com"}` (value not redacted; field name doesn't match) |
| Mixed fields | `{sensor: "crowdstrike", api_key: "secret"}` | `{sensor: "crowdstrike", api_key: "[REDACTED]"}` |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify secret redaction in audit entries. Placeholder for future VP (proptest: no audit entry emitted by Prism contains a known-secret value).

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-002 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
