---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.12-hot-reload.md]
input-hash: "f49106f"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.16.005
module: prism-spec-engine
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

# VP-032: Hot Reload Atomicity — Failed Validation Retains Old Config

## Property Statement

For every live `ConfigManager` holding snapshot `S_old` and every attempted reload
with candidate snapshot `S_new`, if validation of `S_new` fails then the
`ConfigManager`'s active snapshot remains `S_old` after the reload call returns.
In-flight readers observe exactly one of `S_old` or `S_new`, and they observe
`S_new` only if validation succeeded.

## Source Contract

- **Anchor Story:** `S-1.12-hot-reload.md`
- **Source BC:** BC-2.16.005 — reload_config Atomic Swap
- **Module:** prism-spec-engine
- **Category:** Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — random config candidates, fault-injected validators | Valid and invalid candidates |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_spec_engine::config::ConfigManager::reload
//
// Sketch: seed with valid S_old; attempt reload with S_new and oracle
// validation verdict; assert active snapshot == S_new iff verdict == Ok,
// else == S_old.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Size-bounded config generator |
| Tool support? | Full | proptest + arc-swap |
| Execution time budget | <60 seconds for 10k cases | Atomic swap is cheap |
| Assumptions required | Validator is deterministic during test | Fixed oracle |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
