---
document_type: verification-property
level: L4
version: "1.4"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.06-credential-store.md]
input-hash: "a96509c"
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

For every combination of (master passphrase, salt) inputs, `derive_key(passphrase, salt)`
using Argon2id (m=65536, t=3, p=1) returns the same 32-byte output across repeated
invocations. Argon2id is a pure function of its declared inputs and does not read from
system randomness or time. Proptest runs MUST use scaled-down parameters (m=256, t=1, p=1)
to keep suite execution under 60 s; production parameters are validated separately via a
single integration test marked `#[ignore]` by default.

## Source Contract

- **Anchor Story:** `S-1.06`
- **Source BC:** BC-2.03.003 — AES-256-GCM Encrypted File Backend Fallback
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
// Sketch: for arbitrary non-empty passphrase (UTF-8) and 16-byte salt,
// derive k1 and k2 via two independent Argon2id calls with SCALED-DOWN
// params (m=256, t=1, p=1) to keep proptest suite fast; assert k1 == k2.
// Separate #[ignore] integration test verifies production params
// (m=65536, t=3, p=1) produce a 32-byte key without error.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Size-bounded inputs |
| Tool support? | Full | proptest |
| Execution time budget | <60 seconds for 10k cases | Use Argon2id m=256,t=1,p=1 for proptest; production params tested separately under `#[ignore]` |
| Assumptions required | KDF parameters held constant during test | Fixture |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.4 | red-gate-S-1.06 | 2026-04-22 | product-owner | Resolved BC-2.03.003 KDF conflict: property statement now names Argon2id (m=65536, t=3, p=1) as the mandated primitive; removed hedged "(e.g. Argon2id, HKDF)" language. Proof harness sketch updated to target Argon2id. Feasibility note retained: use reduced params (m=256, t=1, p=1) for proptest runs to keep suite under 60 s. |
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.06-credential-store.md) to pure ID (S-1.06). |
| 1.2 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "EncryptedFileBackend key derivation" → "AES-256-GCM Encrypted File Backend Fallback" (matches BC-2.03.003 H1). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
