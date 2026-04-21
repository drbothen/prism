---
document_type: verification-property
level: L4
version: "1.2"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.09-confirmation-tokens.md]
input-hash: "63a9714"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.04.012
module: prism-security
priority: P0
proof_method: kani
verification_method: kani
feasibility: medium
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-009: Confirmation Token — Content Hash Mismatch Rejects

## Property Statement

For every token `t` bound to action params with content hash `h`, and every subsequent
`confirm_action(t, params')` call, if `sha256(params') != h` then the call returns a
content-hash-mismatch error and does not execute the action. Token binding to its
original action content is non-forgeable.

## Source Contract

- **Anchor Story:** `S-1.09`
- **Source BC:** BC-2.04.012 — Token Content Hash Verification Prevents Action Tampering
- **Module:** prism-security
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — params bounded to 32 bytes | All tampered-content cases within bound |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_security::tokens::confirm_action
//
// Sketch: generate a token for params p1; confirm with p2 where p2 != p1 and
// assert result is Err(TokenError::ContentMismatch).
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Short param byte strings |
| Tool support? | Partial | SHA-256 may require abstraction (uninterpreted function) |
| Execution time budget | <10 minutes | Hash modeling adds cost |
| Assumptions required | SHA-256 modeled as collision-resistant UF | Standard crypto abstraction |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.2 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.09-confirmation-tokens.md) to pure ID (S-1.09). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
