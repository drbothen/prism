---
document_type: verification-property
level: L4
version: "1.2"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.02-entity-types.md]
input-hash: "7f066a0"
traces_to: architecture/verification-architecture.md
source_bc: "BC-2.03.008"
module: prism-core
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

# VP-011: Credential Name Sanitization — Rejects Path Traversal

## Property Statement

For every input string `s`, `CredentialName::new(s)` returns `Err` if `s` contains
any of: the substring `..`, a path separator (`/` or `\`), a NUL byte, or any
non-printable control character. Only well-formed, traversal-free names are accepted;
dangerous path-traversal inputs are rejected before the value reaches any backend.

## Source Contract

- **Anchor Story:** `S-1.02`
- **Source BC:** `BC-2.03.008` — Credential Name Sanitization Against Path Traversal
- **Module:** prism-core
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — strings up to length 16 | All path-traversal substrings within bound |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_core::credentials::CredentialName::new
//
// Sketch: generate bounded byte string; if contains "..", "/", "\\", NUL, or
// control char => assert result.is_err(); else assert result.is_ok().
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Length-16 strings |
| Tool support? | Full | Kani handles byte-level matching |
| Execution time budget | <2 minutes | Character-level scan |
| Assumptions required | ASCII charset bound | Reduces state explosion |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.2 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.02-entity-types.md) to pure ID (S-1.02). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
