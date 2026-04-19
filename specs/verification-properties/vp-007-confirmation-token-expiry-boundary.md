---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.09-confirmation-tokens.md]
input-hash: "6365b81"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.04.011
module: prism-security
proof_method: kani
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

# VP-007: Confirmation Token Expiry — Expired at Boundary (Inclusive)

## Property Statement

For every confirmation token with `issued_at = t0` and expiry window `W = 300s`,
the token is considered expired by `is_expired(now)` for all `now >= t0 + W`
(boundary inclusive). Tokens at exactly `t0 + W` are rejected; only tokens strictly
before the boundary remain valid.

## Source Contract

- **Anchor Story:** `S-1.09-confirmation-tokens.md`
- **Source BC:** BC-2.04.011 — Token Expiry at 300 Seconds with Structured Error Recovery
- **Module:** prism-security
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — u64 timestamps with bounded delta | All relative offsets around expiry boundary |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_security::tokens::ConfirmationToken::is_expired
//
// Sketch: for symbolic t0, delta, assert is_expired(t0 + delta) == (delta >= 300).
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Bounded delta to avoid u64 overflow |
| Tool support? | Full | Kani handles integer arithmetic cleanly |
| Execution time budget | <1 minute | Simple comparison |
| Assumptions required | Monotonic clock; no overflow in now - issued_at | Enforced via kani::assume |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |
