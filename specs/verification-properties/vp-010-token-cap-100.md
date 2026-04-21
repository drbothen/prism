---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.09-confirmation-tokens.md]
input-hash: "ff993d9"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.04.009
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

# VP-010: Token Cap — Store Rejects at 100 Active Tokens

## Property Statement

For every `ConfirmationTokenStore`, if the store contains `N` active (non-expired,
non-consumed) tokens and `N >= 100`, every subsequent `generate()` call returns a
capacity-exhausted error without adding a new entry. The active token count is
strictly bounded above by 100.

## Source Contract

- **Anchor Story:** `S-1.09`
- **Source BC:** BC-2.04.009 — Confirmation Token Generation for Irreversible Write Operations (100-Token Active Cap)
- **Module:** prism-security
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — scaled cap (e.g. 3 or 5) proves invariant; 100 by induction | All capacity boundaries |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_security::tokens::ConfirmationTokenStore::generate
//
// Sketch: with CAP = 3 (scaled), fill store to CAP then assert next generate
// returns Err(TokenError::CapacityExhausted); cap value is a const generic.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Scaled cap for proof; parameterize CAP |
| Tool support? | Full | Kani handles bounded collections |
| Execution time budget | <3 minutes | Small cap scale |
| Assumptions required | Cap scales linearly; property generalizes by induction | Documented scaling argument |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.09-confirmation-tokens.md) to pure ID (S-1.09). |
| 1.2 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Confirmation Token Generation with 100-Token Active Cap" → "Confirmation Token Generation for Irreversible Write Operations (100-Token Active Cap)" (matches BC-2.04.009 H1). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
