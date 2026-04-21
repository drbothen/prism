---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [prd.md, architecture/security-architecture.md]
input-hash: "5e8c9da"
traces_to: prd.md
source_bc: BC-2.04.003
module: prism-core
priority: P0
proof_method: kani
verification_method: kani
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
removal_reason: null
removed: null
withdrawal_reason: null
---

# VP-002: Capability Resolution — Deny by Default

## Property Statement

For all capability paths `p` and empty capability maps, `evaluate_capability(p, &BTreeMap::new())` returns `Effect::Deny`. The implicit deny fallback is always reached when no rules exist.

## Source Contract

- **BC:** BC-2.04.003 — Hierarchical Capability Resolution
- **Invariant:** DI-003 — Feature Flag Deny-by-Default with Deny Override

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — paths up to 4 segments, 8 chars each | All path structures within bound |

## Proof Harness Skeleton

```rust
#[kani::proof]
fn verify_deny_by_default() {
    let path: String = kani::any();
    kani::assume(path.len() <= 32);
    kani::assume(path.chars().all(|c| c.is_alphanumeric() || c == '.'));
    let empty_caps: BTreeMap<String, Effect> = BTreeMap::new();
    let result = evaluate_capability(&path, &empty_caps);
    assert_eq!(result.effect, Effect::Deny);
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Medium (bounded paths) | Dot-separated paths, tractable with bounds |
| Proof complexity | Low | BTreeMap lookup with no matching entries |
| Tool support | Full | Kani handles BTreeMap operations |
| Estimated proof time | <60 seconds | |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-04-15 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
