# AC-6 — VP-001 Kani Proof (Formal Verification Placeholder)

**Story:** S-1.01 — prism-core: Foundational Types
**AC:** AC-6: VP-001 Kani proof compiles and passes with 0 verification failures.

## Status

**NOT YET RUN** — Formal verification is scheduled for Phase 5. This placeholder
documents the harness location and the exact command to reproduce.

## Proof Harness Location

```
crates/prism-core/src/proofs/tenant_id.rs
```

The file is gated with `#[cfg(kani)]` and has zero effect on normal test or release builds.

## Harnesses (4 proofs)

| Proof Function | Description |
|----------------|-------------|
| `proof_empty_string_rejected` | VP-001 Proof 1: `TenantId::new("")` always returns `Err` |
| `proof_65_chars_rejected` | VP-001 Proof 2: 65-char string of valid chars always returns `Err` |
| `proof_slash_rejected` | VP-001 Proof 3: string containing `'/'` always returns `Err` |
| `proof_valid_input_accepted` | VP-001 Proof 4: `"acme_corp-01"` always returns `Ok` |
| `verify_tenant_id_validation` | VP-001 Full harness: bounded model check over all inputs up to length 8 |

## Reproduction Command

```bash
cargo kani --proof verify_tenant_id_validation -p prism-core --features kani
```

To run all proofs in the file:

```bash
cargo kani -p prism-core --features kani
```

## Passing Condition

- 0 verification failures
- All 4 named proof cases pass
- `verify_tenant_id_validation` passes with `#[kani::unwind(9)]` (length 8 + 1)

## Phase Scheduled

Phase 5 (Formal Hardening). Tracked in VSDD verification properties table as VP-001.
