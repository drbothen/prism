# Evidence Report — S-1.01: Foundational Types

**Story:** S-1.01 — prism-core: Foundational Types (TenantId, PrismError, StorageDomain)
**Branch:** feature/S-1.01-foundational-types
**Commit at recording:** dbc5ee1
**Test state:** 43/43 tests pass
**Recorded:** 2026-04-22
**Tool:** VHS 0.10.0 (CLI) — library crate with no binary; demos show `cargo test` runs per AC

---

## Coverage Map

| AC | Description | Recording | Tests | Result |
|----|-------------|-----------|-------|--------|
| AC-1 | TenantId rejects empty/whitespace | [AC-1-tenant-id-rejects-empty.gif](AC-1-tenant-id-rejects-empty.gif) | 3 | PASS |
| AC-2 | TenantId accepts valid inputs (round-trip) | [AC-2-tenant-id-valid-input.gif](AC-2-tenant-id-valid-input.gif) | 3 | PASS |
| AC-3 | TenantId rejects path-traversal patterns | [AC-3-tenant-id-rejects-path-traversal.gif](AC-3-tenant-id-rejects-path-traversal.gif) | 5 | PASS |
| AC-4 | StorageDomain::all() returns 16 variants + distinct names | [AC-4-storage-domain-all-16.gif](AC-4-storage-domain-all-16.gif) | 4 | PASS |
| AC-5 | PrismError Display formats per category prefix | [AC-5-prism-error-display.gif](AC-5-prism-error-display.gif) | 21 | PASS |
| AC-6 | VP-001 Kani proof (formal verification) | [AC-6-kani-proof-vp001.md](AC-6-kani-proof-vp001.md) | — | PLACEHOLDER (Phase 5) |
| AC-7 | TenantId serde round-trip | [AC-7-tenant-id-serde-round-trip.gif](AC-7-tenant-id-serde-round-trip.gif) | 3 | PASS |
| AC-8 | TenantId length boundary: 64 chars valid | [AC-8-AC-9-tenant-id-boundary.gif](AC-8-AC-9-tenant-id-boundary.gif) | 2 | PASS |
| AC-9 | TenantId length boundary: 65+ chars rejected | [AC-8-AC-9-tenant-id-boundary.gif](AC-8-AC-9-tenant-id-boundary.gif) | 2 | PASS |

**Total: 8 of 9 ACs demonstrated by live recording. AC-6 documented as Phase 5 placeholder.**

---

## Recordings Detail

### AC-1: TenantId rejects empty string and whitespace
- **Cargo command:** `cargo test --test ac_1_tenant_id_rejects_empty -p prism-core`
- **Test file:** `crates/prism-core/tests/ac_1_tenant_id_rejects_empty.rs`
- **Tests covered:** `test_ac1_tenant_id_rejects_empty_string`, `test_ac1_tenant_id_rejects_whitespace_only`, `test_ac1_tenant_id_rejects_single_space`
- **GIF:** [AC-1-tenant-id-rejects-empty.gif](AC-1-tenant-id-rejects-empty.gif)
- **WebM:** [AC-1-tenant-id-rejects-empty.webm](AC-1-tenant-id-rejects-empty.webm)
- **Tape:** [AC-1-tenant-id-rejects-empty.tape](AC-1-tenant-id-rejects-empty.tape)

### AC-2: TenantId accepts valid inputs
- **Cargo command:** `cargo test --test ac_2_tenant_id_valid_input -p prism-core`
- **Test file:** `crates/prism-core/tests/ac_2_tenant_id_valid_input.rs`
- **Tests covered:** `test_ac2_tenant_id_valid_round_trip`, `test_ac2_tenant_id_single_char_valid`, `test_ac2_tenant_id_all_valid_char_classes`
- **GIF:** [AC-2-tenant-id-valid-input.gif](AC-2-tenant-id-valid-input.gif)
- **WebM:** [AC-2-tenant-id-valid-input.webm](AC-2-tenant-id-valid-input.webm)
- **Tape:** [AC-2-tenant-id-valid-input.tape](AC-2-tenant-id-valid-input.tape)

### AC-3: TenantId rejects path-traversal patterns
- **Cargo command:** `cargo test --test ac_3_tenant_id_rejects_path_traversal -p prism-core`
- **Test file:** `crates/prism-core/tests/ac_3_tenant_id_rejects_path_traversal.rs`
- **Tests covered:** `test_ac3_tenant_id_rejects_path_traversal`, `test_ac3_tenant_id_rejects_dot`, `test_ac3_tenant_id_rejects_slash`, `test_ac3_tenant_id_rejects_null_byte`, `test_ac3_tenant_id_rejects_at_sign`
- **GIF:** [AC-3-tenant-id-rejects-path-traversal.gif](AC-3-tenant-id-rejects-path-traversal.gif)
- **WebM:** [AC-3-tenant-id-rejects-path-traversal.webm](AC-3-tenant-id-rejects-path-traversal.webm)
- **Tape:** [AC-3-tenant-id-rejects-path-traversal.tape](AC-3-tenant-id-rejects-path-traversal.tape)

### AC-4: StorageDomain::all() returns 16 variants with distinct names
- **Cargo command:** `cargo test --test ac_4_storage_domain_all_16 -p prism-core`
- **Test file:** `crates/prism-core/tests/ac_4_storage_domain_all_16.rs`
- **Tests covered:** `test_ac4_storage_domain_all_returns_16_variants`, `test_ac4_storage_domain_column_family_names_are_distinct`, `test_ac4_storage_domain_spot_check_names`, `test_ac4_storage_domain_all_contains_expected_variants`
- **GIF:** [AC-4-storage-domain-all-16.gif](AC-4-storage-domain-all-16.gif)
- **WebM:** [AC-4-storage-domain-all-16.webm](AC-4-storage-domain-all-16.webm)
- **Tape:** [AC-4-storage-domain-all-16.tape](AC-4-storage-domain-all-16.tape)

### AC-5: PrismError Display formats per category prefix
- **Cargo command:** `cargo test --test ac_5_prism_error_display -p prism-core`
- **Test file:** `crates/prism-core/tests/ac_5_prism_error_display.rs`
- **Tests covered:** 21 tests covering E-AUTH, E-STORE, E-SENSOR, E-QUERY, E-CRED, E-FLAG, E-OCSF, E-CFG, E-MCP, E-SAFETY, E-SCHED, E-DET, E-CASE, E-WATCH, E-SPEC, E-IOC, E-INT
- **GIF:** [AC-5-prism-error-display.gif](AC-5-prism-error-display.gif)
- **WebM:** [AC-5-prism-error-display.webm](AC-5-prism-error-display.webm)
- **Tape:** [AC-5-prism-error-display.tape](AC-5-prism-error-display.tape)

### AC-6: VP-001 Kani Proof (Placeholder)
- **Proof file:** `crates/prism-core/src/proofs/tenant_id.rs`
- **Command:** `cargo kani --proof verify_tenant_id_validation -p prism-core --features kani`
- **Status:** NOT YET RUN — scheduled for Phase 5 (Formal Hardening)
- **Documentation:** [AC-6-kani-proof-vp001.md](AC-6-kani-proof-vp001.md)

### AC-7: TenantId serde round-trip
- **Cargo command:** `cargo test --test ac_7_tenant_id_serde_round_trip -p prism-core`
- **Test file:** `crates/prism-core/tests/ac_7_tenant_id_serde_round_trip.rs`
- **Tests covered:** `test_ac7_tenant_id_serde_round_trip`, `test_ac7_tenant_id_serializes_as_bare_string`, `test_ac7_tenant_id_deserialize_invalid_string_returns_err`
- **GIF:** [AC-7-tenant-id-serde-round-trip.gif](AC-7-tenant-id-serde-round-trip.gif)
- **WebM:** [AC-7-tenant-id-serde-round-trip.webm](AC-7-tenant-id-serde-round-trip.webm)
- **Tape:** [AC-7-tenant-id-serde-round-trip.tape](AC-7-tenant-id-serde-round-trip.tape)

### AC-8 and AC-9: TenantId length boundaries
- **Cargo command:** `cargo test --test ac_8_ac_9_tenant_id_boundary -p prism-core`
- **Test file:** `crates/prism-core/tests/ac_8_ac_9_tenant_id_boundary.rs`
- **Tests covered:** `test_ac8_tenant_id_64_chars_valid`, `test_ac9_tenant_id_65_chars_rejected`, `test_ac8_tenant_id_63_chars_valid`, `test_ac9_tenant_id_100_chars_rejected`
- **GIF:** [AC-8-AC-9-tenant-id-boundary.gif](AC-8-AC-9-tenant-id-boundary.gif)
- **WebM:** [AC-8-AC-9-tenant-id-boundary.webm](AC-8-AC-9-tenant-id-boundary.webm)
- **Tape:** [AC-8-AC-9-tenant-id-boundary.tape](AC-8-AC-9-tenant-id-boundary.tape)

---

## Notes on Recording Approach

`prism-core` is a pure library crate with no CLI binary. Recordings show `cargo test` runs
scoped to individual integration test binaries (one per AC). Each integration test binary
lives at `crates/prism-core/tests/ac_N_*.rs` and covers exactly one acceptance criterion.

VHS `Wait+Line` does not work reliably in non-interactive zsh sessions on this platform
(timeout waiting for `test result` pattern). All tapes use `Sleep 15s` after the test
command; this is adequate since cargo runs in under 1s against already-compiled binaries.
The tape source files document the exact `cargo test` invocation for manual reproduction.
