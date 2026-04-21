---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [prd.md, architecture/module-decomposition.md]
input-hash: "f0b2356"
traces_to: prd.md
source_bc: BC-2.06.010
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

# VP-001: TenantId Rejects Invalid Characters

## Property Statement

For all input strings `s`, `TenantId::new(s)` returns `Ok(TenantId)` if and only if `s` matches the regex `^[a-zA-Z0-9_-]+$` (non-empty, alphanumeric plus underscore and hyphen). All other inputs return `Err`.

## Source Contract

- **BC:** BC-2.06.010 — Client ID Validation Enforces Allowed Character Set
- **Invariant:** DI-008 — Client Data Separation (TenantId is the enforcement mechanism)

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — strings up to length 8 | All valid/invalid character combinations within bound |

## Proof Harness Skeleton

```rust
#[kani::proof]
#[kani::unwind(9)] // length 8 + 1
fn verify_tenant_id_validation() {
    let len: usize = kani::any();
    kani::assume(len <= 8);
    let bytes: [u8; 8] = kani::any();
    let s = std::str::from_utf8(&bytes[..len]);
    if let Ok(input) = s {
        let result = TenantId::new(input);
        let is_valid = !input.is_empty()
            && input.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if is_valid {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Small (bounded string) | Length-8 strings are tractable for Kani |
| Proof complexity | Low | Single function, no recursion, no I/O |
| Tool support | Full | Kani handles string validation well |
| Estimated proof time | <30 seconds | Simple character-level checks |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-04-15 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
