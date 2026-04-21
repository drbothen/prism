---
document_type: verification-property
level: L4
version: "1.2"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.09-confirmation-tokens.md]
input-hash: "e15549f"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.04.010
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

# VP-008: Confirmation Token — Single-Use Enforcement

## Property Statement

For every `ConfirmationTokenStore` and every token `t`, after a successful call to
`consume(t)` any subsequent call `consume(t)` returns an error (token already
consumed / not found). No token may be successfully consumed more than once,
regardless of interleavings.

## Source Contract

- **Anchor Story:** `S-1.09`
- **Source BC:** BC-2.04.010 — Confirmation Token Consumption via confirm_action
- **Module:** prism-security
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|------|----------|
| kani | Kani (latest) | Yes — store with up to 2 tokens, 3 consume calls | All single-thread consume orderings |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_security::tokens::ConfirmationTokenStore::consume
//
// Sketch: seed store with a token t; first consume returns Ok; subsequent
// consume calls on t return Err(TokenError::NotFound|AlreadyConsumed).
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Small store, bounded consume count |
| Tool support? | Full | Kani handles HashMap/BTreeMap mutations |
| Execution time budget | <2 minutes | Bounded state |
| Assumptions required | Single-threaded harness; concurrency verified separately | Mutex/RwLock tested elsewhere |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.2 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.09-confirmation-tokens.md) to pure ID (S-1.09). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
