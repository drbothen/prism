---
document_type: story
story_id: "DEFECT-ADAPTER-TLS-XDOME-LIVE-001"
title: "Live xDome HTTPS fails against WAF profile requiring h1-only and User-Agent"
wave: "C"
epic_id: engine-defects
priority: P0
status: draft
version: "0.1"
severity: CRIT
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - findings/prism-pql-deficiencies.md
origin_finding: "F10 (D-1889 triage 2026-07-20)"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts: []
# BC status: pending PO authorship
# F10 requires a NEW behavioral contract and potentially a NEW ADR.
# S-7.01 gate: behavioral_contracts: [] — status MUST remain draft until a product-owner
# authors and anchors a BC with canonical BC-S.SS.NNN ID for this defect.
# Architect adjudication is required BEFORE any BC authorship or implementation.
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: CRIT
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001: Live xDome HTTPS fails against WAF profile requiring h1-only and User-Agent

## Problem

Live HTTPS connections to xDome (Claroty) fail in production. The xDome WAF profile
requires HTTP/1.1 only (no HTTP/2 negotiation) and a `User-Agent` header. No production
crate in the workspace currently compiles in the `http2` reqwest feature, and no
production crate sets a `User-Agent` on outbound sensor API requests. The result is
that live-sensor xDome queries fail at the TLS/connection layer before any data is
returned.

Additionally, the current implementation does not capture error source-chains from
connection failures, and per-target errors are not surfaced in a way that distinguishes
xDome connectivity failures from other sensor errors.

## Origin — D-1889 Triage (F10)

**Triage date:** 2026-07-20  
**Source findings:** `findings/prism-pql-deficiencies.md`  
**Triage capture:** `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
§Bucket-B table row F10

**Consistency-validator cross-check note (2026-07-21):** The original triage capture
note for F10 distorted the primary fix path. A fresh-context consistency-validator
cross-check on 2026-07-21 confirmed that `findings/prism-pql-deficiencies.md` §Finding
10 identifies `http2` as not compiled in and notes that no production crate sets a
`User-Agent`. The source documents `native-tls` as an alternative considered and not as
the primary path. This corrected reading is the authoritative record.

**Primary fix path (ADR-050-compliant, source-recommended):** Add the `http2` reqwest
feature (alongside the existing `rustls-tls` feature declaration — both are additive
features, and `http2` does NOT require `native-tls`) plus a `User-Agent` header on
outbound xDome requests.

**Conflicting alternative (ADR-050-PROHIBITED):** Switching to `native-tls` or adding
`default-tls` / `native-tls-alpn` / `native-tls-vendored` would conflict with ADR-050
§D1/§D2 and is FORBIDDEN workspace-wide. `native-tls` causes ~65 second macOS Keychain
initialization overhead and opens a corporate MITM proxy interception path for outbound
sensor API credentials. The architect must explicitly rule this alternative out in the
governing ADR/BC before implementation begins. It must NOT be presented as a viable path
in implementation guidance.

The finding also requires: (a) error source-chains must be captured for connection
failures and (b) per-target errors must be surfaced to distinguish xDome failures from
other sensor errors.

## Authority

| Artifact | Verbatim Status | Relevant Clause |
|----------|-----------------|-----------------|
| ADR-050 (Workspace reqwest TLS Backend — rustls-tls Mandatory, native-tls Forbidden) | `status: ACCEPTED` | §D1 — `default-features = false, features = ["rustls-tls"]` mandatory; §D2 — `native-tls` and aliases forbidden workspace-wide; `http2` is an additive feature compatible with `rustls-tls` and is not forbidden |
| BC-TBD (new BC required) | — pending authorship — | Governs xDome/Claroty live-sensor HTTPS connection requirements, error source-chain surfacing, and per-target error reporting |
| ADR-TBD (new ADR may be required) | — pending architect decision — | Governs the specific reqwest feature set for sensors with non-default TLS/HTTP negotiation profiles; architect determines whether ADR-050 amendment covers this or a new ADR is needed |

**FINDING-A (MEDIUM, architect-routed):** No governing BC exists yet. The new BC must
be authored by the product-owner after the architect decides the fix mechanism.
ADR-050 is `status: ACCEPTED` and its prohibition of `native-tls` is non-negotiable;
the `http2` feature addition is ADR-050-compliant and is not blocked.

## Routing

Route per triage: **architect FIRST → implementer**

1. **Architect decides first** — the specific decision is:
   - **Primary path (ADR-050-compliant):** Add `reqwest` `http2` feature + `User-Agent` header for xDome requests. Determine whether this requires an ADR-050 amendment or a new ADR, and specify the scope of the `User-Agent` addition (xDome-only vs all sensor adapters).
   - **Conflicting alternative (MUST be ruled out):** `native-tls` / `default-tls` / `native-tls-alpn` — architect records explicit rejection in the governing ADR.
   - **Scope of error-chain and per-target-error surfacing:** architect confirms whether this is covered by existing error-taxonomy architecture or requires a new mechanism.
2. Product-owner authors new BC covering live-sensor HTTPS requirements and error surfacing
3. Story-writer decomposes ACs from the new BC
4. Implementer closes the gap under TDD, following ADR-050 §D1/§D2 strictly

Wave C assignment is contingent on architect adjudication completing before Wave C opens.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, `tdd_mode` declaration, task decomposition, and story-point estimate are deferred
to the architect (fix mechanism decision) and product-owner (BC authorship). This stub
registers the defect as a trackable artifact and records the corrected primary-fix-path
reading from the 2026-07-21 consistency-validator cross-check. No implementation
guidance beyond the architect decision framing is authored here.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage (F10); records corrected primary-fix-path (http2+UA, not native-tls); architect adjudication framing; no ACs or implementation guidance |
