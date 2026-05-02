# Security Findings — S-3.1.06-ImplPhase

**PR:** #117  
**Reviewer:** security-review skill (fresh-context)  
**Date:** 2026-05-01  
**Scope:** `crates/prism-sensors/src/` diff

## Finding Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH     | 0 |
| MEDIUM   | 0 |
| LOW      | 0 |
| INFO     | 0 |

**Verdict: APPROVE — no security findings.**

## Properties Verified

| Property | Result |
|----------|--------|
| OrgIdMismatch guard fires before any network I/O in all 4 adapters | PASS |
| adapter.org_id fields are pub(crate) — not externally mutable after construction | PASS |
| Debug impl: bearer_token = "Secret([REDACTED])", org_id prints as non-secret UUID | PASS |
| DEFAULT_ORG_ID_BYTES remains #[cfg(test)] gated — inaccessible from production | PASS |
| No OrgRegistry import in prism-sensors/src/ (ADR-006 §2.3 forbidden dependency) | PASS |
| OrgId derives Hash + Eq — safe as composite HashMap key | PASS |
| Deprecated init_registry uses nil-UUID sentinel — no cross-tenant bypass path | PASS |
| OWASP A01 (broken access control): cross-tenant dispatch eliminated structurally | PASS |

## Code Scan Evidence (step-4 inline review @ 1d6d45bd)

| File | Guard line | Verified |
|------|-----------|---------|
| `auth/armis.rs` | 577 | `if spec.org_id != self.org_id` — first statement before network I/O |
| `auth/crowdstrike.rs` | 396 | `if spec.org_id != self.org_id` — first statement before OAuth2 token acquire |
| `auth/claroty.rs` | 269 | `if spec.org_id != self.org_id` — first statement before post_read() |
| `auth/cyberint.rs` | 273 | `if spec.org_id != self.org_id` — first statement before login() |

All four `org_id` fields confirmed `pub(crate)`. `bearer_token` redacted in `Debug` for
Armis (armis.rs:357) and Claroty (claroty.rs:162). CrowdStrike/Cyberint have no bearer
field. `DEFAULT_ORG_ID_BYTES` confirmed `#[cfg(test)]` at lib.rs:195.

`OrgRegistry` in lib.rs:194 is a doc comment only — no `use` import. ADR-006 §2.3 satisfied.

## Analysis Notes

The changes are security-positive: they close a silent data-leak path (cross-tenant
adapter dispatch via SensorType-only registry key) and replace it with typed compile-time
and runtime structural isolation. The OrgIdMismatch error returns before any HTTP
semaphore acquisition, cookie-store interaction, or AQL validation — the guard is the
first statement in every fetch() body.

No injection surfaces, no credential exposure, no access control bypasses introduced.
