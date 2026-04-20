---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.06-credential-store.md]
input-hash: "b72885e"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.03.003
module: prism-credentials
priority: P1
proof_method: proptest
verification_method: proptest
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

# VP-035: Key Derivation — Same Inputs Produce Same Key

## Property Statement

For every combination of (master passphrase, salt, context) inputs to the key
derivation function, `derive_key(passphrase, salt, context)` returns the same
byte output across repeated invocations. The KDF (e.g. Argon2id, HKDF) is a pure
function of its declared inputs and does not read from system randomness or time.

## Source Contract

- **Anchor Story:** `S-1.06-credential-store.md`
- **Source BC:** BC-2.03.003 — EncryptedFileBackend key derivation
- **Module:** prism-credentials
- **Category:** Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — random passphrase/salt/context | Random input combinations |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_credentials::kdf::derive_key
//
// Sketch: for arbitrary passphrase/salt/context, compute k1 and k2 via
// two independent calls; assert k1 == k2.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Size-bounded inputs |
| Tool support? | Full | proptest |
| Execution time budget | <60 seconds for 1k cases | Argon2 is slow per call; reduce count |
| Assumptions required | KDF parameters held constant during test | Fixture |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
