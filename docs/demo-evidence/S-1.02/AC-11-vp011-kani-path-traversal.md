# AC-11 — VP-011 Kani Proof: CredentialName Rejects Path Traversal

**Status:** Phase 5 formal verification (Kani not run in standard CI)

## Acceptance Criterion

VP-011 Kani proof passes: all path traversal patterns are rejected by `CredentialName::new`.

## Proof Location

`crates/prism-core/src/proofs/credential_name.rs`

Proof function: `proof_path_traversal_rejected`

## Reproduction Command

```sh
cd crates/prism-core
cargo kani --harness proof_path_traversal_rejected
```

## What the Proof Verifies

Concretely verifies five path traversal inputs all return `Err`:

| Input | Pattern |
|---|---|
| `"a/b"` | Forward slash |
| `"a\b"` | Backslash |
| `"../../passwd"` | Double-dot directory traversal |
| `"key\0value"` | Null byte |
| `"/etc/passwd"` | Absolute path (also contains `/`) |

## Runtime Coverage

The same inputs are verified at runtime by:

- `test_BC_S_02_003_ac4_rejects_path_traversal_double_dot`
- `test_BC_S_02_003_ac5_rejects_null_byte`
- `test_BC_S_02_003_vp011_rejects_forward_slash`
- `test_BC_S_02_003_vp011_rejects_backslash`
- `test_BC_S_02_003_vp011_rejects_absolute_path`

All pass as part of the 103-test suite at commit 44906b8.

## Phase Gate

Phase 5 formal verification. Placeholder replaces GIF/WEBM until Kani toolchain is
available in CI.
