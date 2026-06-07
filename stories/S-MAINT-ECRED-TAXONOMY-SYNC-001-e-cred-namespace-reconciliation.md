---
document_type: story
story_id: "S-MAINT-ECRED-TAXONOMY-SYNC-001"
title: "E-CRED Namespace Reconciliation — Canonical E-CRED-001..010 per ADR-035, collision resolution, KeyringError retirement, taxonomy rewrite"
wave: maintenance
epic_id: maintenance
priority: P2
status: merged
version: "1.2"
spec_version: "v1.0"
level: ops
producer: story-writer
timestamp: "2026-06-07"
modified: "2026-06-07"
input-hash: ""
inputs:
  - .factory/specs/architecture/decisions/ADR-035-e-cred-namespace-reconciliation.md
  - .factory/specs/prd-supplements/error-taxonomy.md
  - .factory/specs/behavioral-contracts/BC-2.06.003-credential-reference-resolution.md
  - crates/prism-core/src/error.rs
  - crates/prism-mcp/src/error_mapping.rs
  - crates/prism-credentials/src/resolve_secret.rs
traces_to: "DRIFT-ECRED-TAXONOMY-001"
anchors: "DRIFT-ECRED-TAXONOMY-001"
drift_anchor: "DRIFT-ECRED-TAXONOMY-001"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems:
  - SS-03
crates_touched:
  - prism-core
  - prism-mcp
  - prism-credentials
target_module: "crates/prism-core/src/error.rs, crates/prism-mcp/src/error_mapping.rs, crates/prism-credentials/src/resolve_secret.rs"
behavioral_contracts:
  - BC-2.03.005
  - BC-2.03.007
  - BC-2.03.009
  - BC-2.06.003
verification_properties: []
depends_on: []
blocks:
  - S-DEMO-003
points: 3
estimated_days: 0.5
risk: LOW
acceptance_criteria_count: 11
red_gate_tests: 5
estimated_passes: "1"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
design_source: "ADR-035"
---

# S-MAINT-ECRED-TAXONOMY-SYNC-001: E-CRED Namespace Reconciliation

## Narrative

As a Prism operator and maintainer, I want every E-CRED error code in prism-core,
prism-credentials, prism-mcp, and the error taxonomy to use the canonical
E-CRED-001..010 namespace defined in ADR-035, so that monitoring rules, alerts,
and operator runbooks can rely on a single unambiguous code per condition —
eliminating the E-CRED-005 collision (encryption error vs. keyring-unavailable),
the undeclared E-CRED-009 file-I/O code in resolve_secret.rs, and the dead
`PrismError::KeyringError` variant.

## Background

Three separate sources defined E-CRED-NNN codes and they contradicted each other:

- **`prism-core/src/error.rs`** (`PrismError` enum): E-CRED-001..006 + E-CRED-010
  variants, with E-CRED-005 assigned to `CredentialEncryptionError`.
- **`error-taxonomy.md`** v1.61: E-CRED-001..005 rows, with E-CRED-005 assigned to
  OS keyring unavailable (Tier-3), plus taxonomy-only conditions that had no code
  variant (generic keyring E-CRED-001, decryption-failed E-CRED-003).
- **`prism-credentials/src/resolve_secret.rs`**: Three inline `E-CRED-009:` strings
  for file-I/O sub-cases embedded as `reason` literals inside
  `PrismError::InvalidCredentialName`. The code E-CRED-009 appears nowhere in
  the taxonomy.

The critical collision: `E-CRED-005` was simultaneously the Display prefix for
`PrismError::CredentialEncryptionError` (on develop) AND the keyring-unavailable
code in ADR-034, BC-2.06.003, and the S-DEMO-003 branch (`resolution.rs`). These
are semantically orthogonal conditions — a monitoring system cannot distinguish them.

ADR-035 (accepted 2026-06-07, human "architect designs fresh" directive) defines the
canonical E-CRED-001..010 namespace and the exact migration mapping. This story
executes the develop-scope half of that migration. The S-DEMO-003 branch scope
(resolution.rs, in_memory_store.rs, bc_2_06_003 test) is handled by the S-DEMO-003
re-baseline, not this story.

**DF-PASS3-001 closure (LOCAL pass-3):** Adversarial finding DF-PASS3-001 identified
two downstream `.factory/specs` docs that still cited `E-CRED-001` for the
`credential not found` condition (canonically `E-CRED-002 = CredentialNotFound` per
ADR-035 §D1). The finding was closed in-scope: `security-architecture.md` (v1.2)
resolution-chain "Not found" node corrected E-CRED-001 → E-CRED-002; and
`interface-definitions.md` (v2.7) `credential_status` errors array corrected
E-CRED-001 → E-CRED-002. ADR-035's blast-radius inventory was extended to list both
docs. See AC-011 for the cross-spec propagation sweep acceptance criterion.

**Forward reservation note:** E-CRED-008 (`KeyringBackendUnavailable`) is defined
in the taxonomy by this story but has NO emitter on develop. Its sole emitter
(`resolution.rs` Tier-3 keyring path) lives on the S-DEMO-003 feature branch
and arrives with that story's merge. This is an intentional, documented
forward-definition per ADR-035 §D2, not spec drift.

## Design Source

**ADR-035** (`architecture/decisions/ADR-035-e-cred-namespace-reconciliation.md`)
is the single authoritative design source for all AC traces in this story.
The canonical E-CRED-001..010 namespace, Display format strings, migration mapping
table, and blast-radius inventory are all defined there. Do not invent or derive
any error code, Display string, or migration action independently.

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-2.03.005 | Credential CRUD Operations via MCP Tools (Mutations Require Confirmation Token) | Postcondition: error responses use canonical E-CRED-NNN codes |
| BC-2.03.007 | Secret Redaction in Logs, Errors, and MCP Responses | Postcondition: error message prefix must be canonical code; no credential value in Display string |
| BC-2.03.009 | resolve_secret() for _FILE Env Var and K8s Secret Mount Compatibility | Postcondition: file-I/O error returns canonical E-CRED-005 prefix |
| BC-2.06.003 | Credential References in Config Resolve to Credential Store Entries | Tier-3 error cite updated to E-CRED-008 per ADR-035 §D5 |

**Anchor justification per POL-5:**

- **BC-2.03.005** anchors AC-007 (E-CRED assertion in bc_2_03_005_credential_crud.rs
  passes without change — E-CRED-001 is preserved; no regression): the CRUD test is
  the canonical existing assertion for the credential error-code contract.
- **BC-2.03.007** anchors AC-001 and AC-002 (Display strings after renumber must not
  leak credential values; the secret-redaction invariant survives the renumber):
  BC-2.03.007 §Postconditions governs Display format safety.
- **BC-2.03.009** anchors AC-003 (resolve_secret.rs file-I/O errors emit E-CRED-005
  after migration): BC-2.03.009 governs the `resolve_secret()` error surface.
- **BC-2.06.003** anchors AC-008 (BC-2.06.003 Tier-3 cite updated to E-CRED-008):
  this BC owns the Tier-3 keyring resolution postconditions.

## Scope Boundary

### IN SCOPE — develop branch

**Code changes (implementer):**

| File | Change |
|------|--------|
| `crates/prism-core/src/error.rs` | Renumber `CredentialEncryptionError` E-CRED-005→E-CRED-006 (`#[error]` Display string + doc comment); renumber `EncryptionKeyMissing` E-CRED-006→E-CRED-007 (same); RETIRE `KeyringError` variant entirely (verify zero constructors before deletion) |
| `crates/prism-mcp/src/error_mapping.rs` | Remove the now-dead `PrismError::KeyringError { .. }` match arm |
| `crates/prism-credentials/src/resolve_secret.rs` | Change 3 inline `E-CRED-009:` strings to `E-CRED-005:` with regularized format per ADR-035 §Blast-Radius |

**Test changes (implementer):**

| File | Change |
|------|--------|
| `crates/prism-core/tests/ac_5_prism_error_display.rs` | Add assertions for E-CRED-006 (encryption) and E-CRED-007 (key); verify E-CRED-001 assertion is unchanged |
| `crates/prism-credentials/tests/bc_2_03_009_resolve_secret.rs` | Tighten the two loose `msg.contains("E-CRED")` assertions to `msg.contains("E-CRED-005")` |

**Spec changes (product-owner / architect):**

| File | Change |
|------|--------|
| `.factory/specs/prd-supplements/error-taxonomy.md` | Wholesale rewrite of E-CRED section to canonical E-CRED-001..010 table per ADR-035 §Blast-Radius / product-owner section |
| `.factory/specs/behavioral-contracts/BC-2.06.003-credential-reference-resolution.md` | Update Tier-3 error cite from E-CRED-005 to E-CRED-008; add ADR-035 to normative references |
| `.factory/specs/architecture/decisions/ADR-034-tier3-keyring-resolution-org-id-threading.md` | Update 3 `E-CRED-005` references in §D4 to `E-CRED-008` per ADR-035 §D5 amendment |

### OUT OF SCOPE — S-DEMO-003 re-baseline handles these

- `crates/prism-credentials/src/resolution.rs` (S-DEMO-003 worktree): keyring
  E-CRED-005→E-CRED-008 string edits in `BackendUnavailable { detail: ... }`
- `crates/prism-credentials/src/in_memory_store.rs` (S-DEMO-003 worktree): doc
  comments referencing E-CRED-005 keyring path
- `crates/prism-credentials/tests/bc_2_06_003_tier3_keyring_resolution.rs`
  (S-DEMO-003 worktree): `detail.contains("E-CRED-005")` → E-CRED-008 assertion
- S-DEMO-003 story spec: AC-011 Case B detail string, EC-001b, DEMO-RUNBOOK §6b
- Any file that exists only on the `feature/S-DEMO-003` worktree branch

Do NOT open those files, do NOT modify them. S-DEMO-003 is paused mid-cascade; any
edits to its worktree files from this story would conflict with the S-DEMO-003
re-baseline burst.

### OUT OF SCOPE — Immutable narrative (POL-1 append-only)

Historical changelog and decision-log entries in `.factory/` that mention E-CRED codes
in a narrative context are immutable per POL-1 (append-only logs) and must NOT be
modified as part of this story's E-CRED sweep:

- **`S-1.07-credential-crud.md`** (`STORY-ID S-1.07`, `story-id s-1-07`, or similar):
  any E-CRED references appearing in §Changelog rows or historical §Previous Story
  Intelligence text record past decisions and must not be retroactively altered.
- **`STORY-INDEX.md`**: E-CRED references in the status/notes column of historical
  index rows are narrative record, not live spec citations. They are intentionally
  NOT modified.

If an E-CRED code appears in these files as an **active** spec citation (not a
changelog/decision-log entry), escalate to the orchestrator — that would be a new
drift item, not covered by this story.

## Acceptance Criteria

### AC-001: CredentialEncryptionError is renumbered to E-CRED-006 (traces to BC-2.03.007 postcondition; ADR-035 §D2)

`PrismError::CredentialEncryptionError` in `crates/prism-core/src/error.rs`:

- The `#[error(...)]` attribute reads `"E-CRED-006: credential encryption error: {reason}"`.
- The doc comment reads `/// E-CRED-006: ...` (no longer `E-CRED-005`).
- Running `rg 'E-CRED-005.*encryption' crates/prism-core/src/error.rs` returns **zero hits**.
- Running `rg 'E-CRED-006.*encryption' crates/prism-core/src/error.rs` returns exactly **one hit**.

### AC-002: EncryptionKeyMissing is renumbered to E-CRED-007 (traces to BC-2.03.007 postcondition; ADR-035 §D2)

`PrismError::EncryptionKeyMissing` in `crates/prism-core/src/error.rs`:

- The `#[error(...)]` attribute reads `"E-CRED-007: encryption key not configured: {reason}"`.
- The doc comment reads `/// E-CRED-007: ...` (no longer `E-CRED-006`).
- Running `rg 'E-CRED-006.*encryption key' crates/prism-core/src/error.rs` returns **zero hits**.
- Running `rg 'E-CRED-007.*encryption key' crates/prism-core/src/error.rs` returns exactly **one hit**.

### AC-003: resolve_secret.rs file-I/O errors emit canonical E-CRED-005 (traces to BC-2.03.009 postcondition; ADR-035 §D1 E-CRED-005)

All three file-I/O error strings in `crates/prism-credentials/src/resolve_secret.rs`
use `E-CRED-005:` with the regularized format from ADR-035:

- Missing file: `"E-CRED-005: credential file I/O error for '{}': file does not exist (configured in env var '{}')"`.
- Is-directory: `"E-CRED-005: credential file I/O error for '{}': path is a directory, not a regular file"`.
- Read failure: `"E-CRED-005: credential file I/O error for '{}': read failed: {}"`.

Running `rg 'E-CRED-009' crates/prism-credentials/src/resolve_secret.rs` returns **zero hits**.
Running `rg 'E-CRED-005' crates/prism-credentials/src/resolve_secret.rs` returns exactly **three hits**.

### AC-004: KeyringError variant is retired and removed (traces to BC-2.03.007 postcondition; ADR-035 §D4)

`PrismError::KeyringError` is deleted from `crates/prism-core/src/error.rs`:

- The variant declaration (`KeyringError { detail: String }`), its `#[error(...)]` attribute,
  and its doc comment are all absent.
- Pre-deletion verification: `rg 'PrismError::KeyringError' crates/` (excluding test and
  error_mapping.rs) must return zero hits before deletion is attempted, confirming zero
  production constructors.
- Running `rg 'KeyringError' crates/prism-core/src/error.rs` returns **zero hits** after deletion.

### AC-005: error_mapping.rs KeyringError arm is removed (traces to BC-2.03.005 postcondition; ADR-035 §D4)

The `PrismError::KeyringError { .. }` match arm in
`crates/prism-mcp/src/error_mapping.rs` is deleted.

- Running `rg 'KeyringError' crates/prism-mcp/src/error_mapping.rs` returns **zero hits**.
- The remaining `PrismError::CredentialEncryptionError { .. }` arm (if present) maps correctly.
- `just check` exits 0 — no unhandled match arm warnings from the deletion.

### AC-006: error-taxonomy.md E-CRED section is rewritten to canonical E-CRED-001..010 (traces to BC-2.03.009 postcondition; ADR-035 §D1)

The E-CRED section in `.factory/specs/prd-supplements/error-taxonomy.md` contains
exactly the rows E-CRED-001 through E-CRED-010 matching the canonical table in
ADR-035 §D1:

- E-CRED-001 `InvalidCredentialName` — Display: `"E-CRED-001: invalid credential name '{name}': {reason}"`
- E-CRED-002 `CredentialNotFound` — Display: `"E-CRED-002: credential not found: {name}"`
- E-CRED-003 `CredentialAccessDenied` — Display: `"E-CRED-003: credential access denied for {name} — credential values never transit AI context"`
- E-CRED-004 `CredentialStoreError` — Display: `"E-CRED-004: credential store error (backend={backend}): {reason}"`
- E-CRED-005 `CredentialFileIo` — Display: `"E-CRED-005: credential file I/O error for '{path}': {reason}"`
- E-CRED-006 `CredentialEncryptionError` — Display: `"E-CRED-006: credential encryption error: {reason}"`
- E-CRED-007 `EncryptionKeyMissing` — Display: `"E-CRED-007: encryption key not configured: {reason}"`
- E-CRED-008 `KeyringBackendUnavailable` — Display: `"E-CRED-008: OS keyring unavailable: {reason}"` (forward-reserved; emitter arrives with S-DEMO-003)
- E-CRED-009 `CredentialDecryptionFailed` — Display: `"E-CRED-009: credential decryption failed for ({client_id}, {sensor_id}): {reason}"`
- E-CRED-010 `(RESERVED)` — no emitter

No row from the old E-CRED namespace (old E-CRED-001 keyring-unavailable, old E-CRED-002
encrypted-file-key-missing, old E-CRED-003 decryption-failed, old E-CRED-004
invalid-name path-traversal, old E-CRED-005 OS-keyring-unavailable-tier3) remains in
the taxonomy. All five old rows are replaced by the canonical ten-row table.

### AC-007: Existing E-CRED-001 test assertion is unaffected (traces to BC-2.03.005 postcondition; ADR-035 §D1 E-CRED-001)

`crates/prism-core/tests/ac_5_prism_error_display.rs` function
`test_ac5_prism_error_display_e_cred_001` continues to pass without modification.
E-CRED-001 (`InvalidCredentialName`) is preserved unchanged — same code, same Display
string, same doc comment. Running `cargo nextest run -p prism-core -E 'test(e_cred_001)'`
exits 0.

### AC-008: BC-2.06.003 and ADR-034 §D4 updated to E-CRED-008 (traces to BC-2.06.003 Tier-3 postcondition; ADR-035 §D5)

Two spec artifacts updated per ADR-035 §D5 amendment:

1. **BC-2.06.003** (`credential-reference-resolution.md`):
   - Tier-3 postconditions table: `BackendUnavailable { detail: "E-CRED-008: OS keyring
     unavailable: {reason}" }`.
   - Invariant note: `E-CRED-008` (not E-CRED-005).
   - Canonical Test Vectors: Tier-3 backend-error output column reads E-CRED-008.
   - Frontmatter `normative_refs` or equivalent: ADR-035 added.

2. **ADR-034** (`tier3-keyring-resolution-org-id-threading.md`):
   - Frontmatter `related_adrs`: `"ADR-035"` added.
   - §D4 table row "Keyring backend error": detail string updated from `"E-CRED-005: ..."`
     to `"E-CRED-008: OS keyring unavailable: {reason}"`.
   - All three occurrences of `E-CRED-005` in §D4 prose and §Consequences updated to
     `E-CRED-008`.

Note: BC-2.06.003 `status:` remains `draft` — the BC is anchored to S-DEMO-003
for promotion. This cite-update does not change the BC lifecycle status.

### AC-009: No-collision invariant — E-CRED-005 is emitted by exactly one path on develop (traces to BC-2.03.009 postcondition; ADR-035 §D2)

After all code changes on develop:

```bash
git grep 'E-CRED-005' -- crates/
```

returns hits in **exactly one file**: `crates/prism-credentials/src/resolve_secret.rs`
(three occurrences, all file-I/O sub-cases).

No occurrence of `E-CRED-005` appears in:
- `crates/prism-core/src/error.rs` (formerly `CredentialEncryptionError`)
- `crates/prism-mcp/src/error_mapping.rs`
- Any other `crates/**/*.rs` file on develop

This verifies the collision is fully resolved in the develop codebase. The
S-DEMO-003 worktree (out of scope) still contains E-CRED-005 until its re-baseline.

### AC-010: bc_2_03_009_resolve_secret.rs loose assertions tightened to E-CRED-005 (traces to BC-2.03.009 postcondition; ADR-035 §Blast-Radius)

In `crates/prism-credentials/tests/bc_2_03_009_resolve_secret.rs`:

- The two loose assertions `msg.contains("E-CRED")` are replaced with
  `msg.contains("E-CRED-005")`.
- All affected test functions continue to pass: `cargo nextest run -p prism-credentials
  -E 'test(bc_2_03_009)'` exits 0.
- The tightened assertions now guard against future code drift that changes the
  file-I/O error code away from E-CRED-005.

### AC-011: All downstream .factory/specs E-CRED-002 citations use the canonical CredentialNotFound code (traces to BC-2.03.005 postcondition; ADR-035 §D1 E-CRED-002; closes DF-PASS3-001)

**Anchor justification per POL-5:** BC-2.03.005 ("Credential CRUD Operations via MCP
Tools") postconditions require that the `credential not found` error condition is
reported with the canonical code. ADR-035 §D1 assigns E-CRED-002 as the sole
canonical code for `CredentialNotFound`. Any downstream `.factory/specs` document
citing the `credential not found` condition as E-CRED-001 is inconsistent with this
contract and is a doc-label drift item. BC-2.03.005 is the appropriate anchor because
it owns the not-found error surface in the CRUD contract.

All `.factory/specs` documents that reference the `credential not found` condition
cite it as `E-CRED-002` (not `E-CRED-001`). Specifically:

1. **`security-architecture.md`**: The credential resolution chain node labeled
   "Not found" (or equivalent) cites `E-CRED-002`, not `E-CRED-001`. Running:
   ```
   rg 'E-CRED-001' .factory/specs/architecture/security-architecture.md
   ```
   in the context of the resolution chain returns **zero hits** for the not-found
   node. Running `rg 'E-CRED-002' .factory/specs/architecture/security-architecture.md`
   returns at least one hit at the not-found resolution point.

2. **`interface-definitions.md`**: The `credential_status` errors array entry for the
   not-found condition cites `E-CRED-002`, not `E-CRED-001`. Running:
   ```
   rg 'E-CRED-001' .factory/specs/prd-supplements/interface-definitions.md
   ```
   in the context of credential status errors returns **zero hits** for the not-found
   entry. Running `rg 'E-CRED-002' .factory/specs/prd-supplements/interface-definitions.md`
   returns at least one hit in the `credential_status` errors definition.

**Verification:** This AC is verified by adversarial sweep of the spec documents, not
by a unit test — the correction is a doc-label fix in `.factory/` specs, not a change
in code behavior. The canonical taxonomy (ADR-035 §D1 E-CRED-002 row) and the fixed
spec documents serve as the authority. No new Red Gate test is added for this AC;
`red_gate_tests` count remains 5. Historical changelog/decision-log E-CRED references
in `S-1.07-credential-crud.md` and `STORY-INDEX.md` are immutable narrative (POL-1
append-only) and are intentionally excluded from this sweep — see §Scope for details.

## Red Gate Tests

These tests must be written as **failing** (Red Gate) before any implementation begins,
per TDD Iron Law. They encode the canonical Display strings from ADR-035 §Exact-Display-String-Changes.

### RG-ECRED-001: E-CRED-006 encryption Display string

**File:** `crates/prism-core/tests/ac_5_prism_error_display.rs`

**Test name:** `test_ac5_prism_error_display_e_cred_006_encryption`

**Behavior:** Construct `PrismError::CredentialEncryptionError { reason: "test reason".to_string() }`
and assert the Display output starts with `"E-CRED-006: credential encryption error:"`.

**Fails before:** `#[error]` still reads `"E-CRED-005: ..."`.
**Passes after:** `#[error]` updated to `"E-CRED-006: ..."`.

### RG-ECRED-002: E-CRED-007 key-missing Display string

**File:** `crates/prism-core/tests/ac_5_prism_error_display.rs`

**Test name:** `test_ac5_prism_error_display_e_cred_007_key_missing`

**Behavior:** Construct `PrismError::EncryptionKeyMissing { reason: "not set".to_string() }`
and assert the Display output starts with `"E-CRED-007: encryption key not configured:"`.

**Fails before:** `#[error]` still reads `"E-CRED-006: ..."`.
**Passes after:** `#[error]` updated to `"E-CRED-007: ..."`.

### RG-ECRED-003: E-CRED-005 file-I/O Display strings in resolve_secret.rs

**File:** `crates/prism-credentials/tests/bc_2_03_009_resolve_secret.rs`

**Test name:** `test_BC_2_03_009_resolve_secret_file_io_emits_e_cred_005`

**Behavior:** Call `resolve_secret()` with a `SOME_CREDENTIAL_FILE` env var pointing to
a nonexistent path and assert:
1. The returned error is `Err(PrismError::InvalidCredentialName { reason })`.
2. `reason.contains("E-CRED-005")` is true.
3. `reason.contains("E-CRED-009")` is false (old code is gone).

**Fails before:** resolve_secret.rs still emits `"E-CRED-009: ..."`.
**Passes after:** resolve_secret.rs updated to `"E-CRED-005: ..."`.

### RG-ECRED-004: KeyringError variant is absent

**File:** `crates/prism-core/tests/ac_5_prism_error_display.rs` (or a new
`crates/prism-core/tests/e_cred_namespace_invariants.rs`)

**Test name:** `test_e_cred_keyring_error_variant_retired`

**Behavior:** This is a compile-time guard. Add a `#[cfg(test)]` function whose body
attempts to reference `PrismError::KeyringError { detail: "".to_string() }`. The test
**must not compile** after the variant is removed. Implement as a build-script or
doc-test that asserts the variant does not exist. Simplest approach: add a
compile-fail assertion comment referencing the retired variant, or use a conditional
`static_assertions::assert_not_impl_any!` idiom. If compile-fail is not feasible in
this crate's layout, replace with a documentation-only assertion and add a
`rg 'KeyringError' crates/prism-core/src/'` invocation in a `#[test]` function that
calls `std::process::Command`.

**Fails before:** variant exists, match arm compiles.
**Passes after:** variant removed, all previous match arms for `KeyringError` are gone.

### RG-ECRED-005: No-collision guard — E-CRED-005 limited to file-I/O sites on develop

**File:** `crates/prism-credentials/tests/bc_2_03_009_resolve_secret.rs`
(or `crates/prism-core/tests/e_cred_namespace_invariants.rs`)

**Test name:** `test_e_cred_005_emitted_only_by_file_io_path`

**Behavior:** This is a workspace-scan invariant test. Using `std::process::Command`,
run:

```rust
let output = std::process::Command::new("git")
    .args(["grep", "E-CRED-005", "--", "crates/"])
    .output()
    .expect("git grep failed");
let hits = String::from_utf8_lossy(&output.stdout);
// All hits must be in resolve_secret.rs; no hit may be in prism-core/src/error.rs
assert!(!hits.contains("prism-core/src/error.rs"),
    "E-CRED-005 must not appear in prism-core/src/error.rs after rename to E-CRED-006");
assert!(hits.contains("resolve_secret.rs"),
    "E-CRED-005 must appear in resolve_secret.rs (file-I/O path)");
```

**Fails before:** `prism-core/src/error.rs` still contains `E-CRED-005` (encryption).
**Passes after:** `CredentialEncryptionError` is renumbered to E-CRED-006; only
resolve_secret.rs contains E-CRED-005.

## Tasks

### Code tasks (implementer — develop-based worktree)

1. **Verify zero KeyringError constructors before deletion:**

   ```bash
   rg 'PrismError::KeyringError' crates/ --type rust
   ```

   Expected: only hits in `crates/prism-mcp/src/error_mapping.rs` (the match arm) and
   tests. No production constructor site. If any production constructor exists, STOP
   and escalate to the orchestrator — the blast-radius analysis in ADR-035 §D4 is wrong.

2. **Write the 5 Red Gate tests** (RG-ECRED-001 through RG-ECRED-005) in their target
   files. Verify each fails with the CURRENT code before proceeding.

3. **Update `crates/prism-core/src/error.rs`:**
   - `CredentialEncryptionError`: change doc comment and `#[error]` from E-CRED-005 to E-CRED-006.
   - `EncryptionKeyMissing`: change doc comment and `#[error]` from E-CRED-006 to E-CRED-007.
   - `KeyringError`: delete the entire variant (doc comment + `#[error]` attribute + field).

4. **Update `crates/prism-mcp/src/error_mapping.rs`:**
   - Delete the `PrismError::KeyringError { .. } => (...)` match arm.
   - Confirm `just iter prism-mcp` exits 0 (no unhandled-variants warning from the match deletion).

5. **Update `crates/prism-credentials/src/resolve_secret.rs`:**
   Change three `"E-CRED-009: ..."` inline reason strings to the canonical E-CRED-005
   format strings from ADR-035 §Exact-Display-String-Changes:
   - File missing: `"E-CRED-005: credential file I/O error for '{}': file does not exist (configured in env var '{}')"`.
   - Is-directory: `"E-CRED-005: credential file I/O error for '{}': path is a directory, not a regular file"`.
   - Read failure: `"E-CRED-005: credential file I/O error for '{}': read failed: {}"`.

6. **Tighten `bc_2_03_009_resolve_secret.rs` assertions:**
   Replace `msg.contains("E-CRED")` with `msg.contains("E-CRED-005")` in both loose-assertion
   locations. Run `just iter prism-credentials` — exits 0.

7. **Add `ac_5_prism_error_display.rs` assertions for E-CRED-006 and E-CRED-007:**
   Add `test_ac5_prism_error_display_e_cred_006_encryption` and
   `test_ac5_prism_error_display_e_cred_007_key_missing` per RG-ECRED-001 and RG-ECRED-002.
   Run `just iter prism-core` — exits 0.

8. **Run final pre-push gate:** `just check` — exits 0 across all crates.

9. **Commit** with message citing `DRIFT-ECRED-TAXONOMY-001`, `S-MAINT-ECRED-TAXONOMY-SYNC-001`,
   and `ADR-035`. No AI attribution per project git conventions.

### Spec tasks (product-owner / architect — .factory/ artifacts)

10. **Rewrite `error-taxonomy.md` E-CRED section** per ADR-035 §Blast-Radius
    product-owner table: remove old E-CRED-001..005 rows; add canonical
    E-CRED-001..010 rows with Display strings, categories, and retryable flag
    from ADR-035 §D1. The E-CRED-008 row includes the forward-reservation note:
    "Emitter arrives with S-DEMO-003 (Tier-3 keyring path); forward-defined per ADR-035 §D2."

11. **Update BC-2.06.003** per ADR-035 §D5: update Tier-3 error cite from E-CRED-005
    to E-CRED-008 in postconditions table, invariant note, and canonical test vectors;
    add ADR-035 to normative references. BC lifecycle status remains `draft`.

12. **Update ADR-034** per ADR-035 §D5: add `"ADR-035"` to `related_adrs` frontmatter;
    update all three E-CRED-005 occurrences in §D4 to E-CRED-008.

13. **Commit** `.factory/` artifacts in a single atomic commit per TD-VSDD-053.

## Previous Story Intelligence

- **S-DEMO-003 LOCAL pass-17** (D-1043): Surfaced `OBS-2` — the `prism-core PrismError E-CRED`
  enum ↔ `error-taxonomy.md` misalignment across E-CRED-001..005. Classified as
  system-level, pre-existing, out of S-DEMO-003 perimeter. Registered
  `DRIFT-ECRED-TAXONOMY-001` and `S-MAINT-ECRED-TAXONOMY-SYNC-001` stub. OBS-2 did
  not reset S-DEMO-003's adversarial streak.

- **ADR-034** (D-1025, accepted): Introduced the keyring-backend-error code as E-CRED-005
  (corrected from E-CRED-003 to avoid a different collision). That correction propagated
  into BC-2.06.003 v1.4, error-taxonomy.md v1.61, and S-DEMO-003. However, E-CRED-005 was
  already allocated in `prism-core/src/error.rs` to `CredentialEncryptionError` — so the
  correction created the collision that ADR-035 now resolves.

- **ADR-035** (D-TBD, accepted 2026-06-07): Human selected "architect designs fresh" —
  no existing source is presumed canonical. ADR-035 is the only design authority for this
  story. All AC traces cite ADR-035 clauses.

- **Key lesson:** When an ADR introduces a new error code, verify the code is not already
  allocated in `prism-core/src/error.rs` before committing. A taxonomy-vs-code scan at
  ADR authorship time would have caught this collision at ADR-034 authorship.

## Architecture Compliance Rules

(Derived from ADR-035, CLAUDE.md §Conventions, and ADR-022.)

1. **ADR-035 is the sole authority for error code assignments.** No Display string,
   code number, or variant name in this story's scope may differ from ADR-035 §D1
   and §Exact-Display-String-Changes. If a discrepancy is found, STOP and escalate —
   do not invent a resolution.

2. **`#[error]` is the sole source of truth for Display output** (per thiserror crate).
   Doc comments are informational. When updating, change BOTH the `#[error]` attribute
   and the doc comment to keep them in sync.

3. **Zero constructors before deletion.** The `KeyringError` variant must have zero
   production constructor sites before it is removed. Verify with `rg` per Task 1.
   If a constructor is found, it is a blast-radius miss in ADR-035 — escalate to
   architect, do not proceed with deletion.

4. **error_mapping.rs match must remain exhaustive.** After deleting the `KeyringError`
   arm, run `just iter prism-mcp` immediately. A non-exhaustive match warning or error
   means an additional arm was missed; fix before proceeding.

5. **E-CRED codes are emitted as string literals in resolve_secret.rs** (inside the
   `reason` field of `PrismError::InvalidCredentialName`). This is architecturally
   impure but intentional — ADR-035 §Negative-Consequences notes the structural refactor
   (new `PrismError::CredentialFileIo` variant) as a follow-up story. Do not introduce
   the structural refactor in this story; update the string literals only.

6. **TD-VSDD-060 sibling-site sweep.** When updating the `#[error]` attribute for
   `CredentialEncryptionError`, grep all files in `crates/prism-core/` and
   `crates/prism-credentials/` for the old string `"E-CRED-005: credential encryption"`
   to ensure no secondary callsite was missed.

7. **No scope expansion.** If grep reveals additional files containing `E-CRED-005`
   outside of `resolve_secret.rs` on develop (e.g., documentation, tests, shell scripts),
   record them as separate DRIFT items. Do not expand this story to cover them.

8. **S-DEMO-003 worktree is off-limits.** Do not open, read, or modify any file that
   exists only in `.worktrees/S-DEMO-003/`. Those files will be updated by the
   S-DEMO-003 re-baseline after this story merges.

9. **No AI attribution in commits** per project git conventions (CLAUDE.md).

10. **`just check` must exit 0** before the PR is opened. This story is P2 and LOW risk;
    its changes are narrow enum-string updates. Any clippy warning or test failure is a
    blocker, not a warning.

## Library & Framework Requirements

No new dependencies. All changes are to existing Rust source files and `.factory/` Markdown.

| Tool | Purpose | Version |
|------|---------|---------|
| thiserror | `#[error(...)]` attribute macro on `PrismError` variants | existing workspace pin |
| ripgrep (`rg`) | Site discovery for sibling-site sweep (TD-VSDD-060) | system |
| `just check` | Final pre-push gate | workspace Justfile |
| `cargo nextest` | Per-crate TDD inner loop | existing workspace pin |

## File Structure Requirements (§FSR)

| File | Action | Crate |
|------|--------|-------|
| `crates/prism-core/src/error.rs` | Modify — renumber 2 variants, retire 1 variant | prism-core |
| `crates/prism-mcp/src/error_mapping.rs` | Modify — remove 1 match arm | prism-mcp |
| `crates/prism-credentials/src/resolve_secret.rs` | Modify — update 3 inline reason strings | prism-credentials |
| `crates/prism-core/tests/ac_5_prism_error_display.rs` | Modify — add 2 new test functions (RG-ECRED-001, RG-ECRED-002); verify existing E-CRED-001 test is unchanged | prism-core |
| `crates/prism-credentials/tests/bc_2_03_009_resolve_secret.rs` | Modify — tighten 2 loose `msg.contains("E-CRED")` assertions to `msg.contains("E-CRED-005")` | prism-credentials |
| `crates/prism-core/tests/ac_5_prism_error_display.rs` OR new `crates/prism-core/tests/e_cred_namespace_invariants.rs` | Add — RG-ECRED-004 (KeyringError retired guard) + RG-ECRED-005 (no-collision workspace scan) | prism-core |
| `.factory/specs/prd-supplements/error-taxonomy.md` | Modify — wholesale rewrite E-CRED section | .factory spec |
| `.factory/specs/behavioral-contracts/BC-2.06.003-credential-reference-resolution.md` | Modify — update E-CRED-005 cites to E-CRED-008 in Tier-3 postconditions + test vectors + normative refs | .factory spec |
| `.factory/specs/architecture/decisions/ADR-034-tier3-keyring-resolution-org-id-threading.md` | Modify — add ADR-035 to related_adrs; update 3 E-CRED-005 occurrences in §D4 to E-CRED-008 | .factory spec |
| `.factory/specs/architecture/security-architecture.md` | Touched — resolution-chain "Not found" node corrected E-CRED-001 → E-CRED-002 (DF-PASS3-001 closure; AC-011) | .factory spec |
| `.factory/specs/prd-supplements/interface-definitions.md` | Touched — `credential_status` errors array E-CRED-001 → E-CRED-002 (DF-PASS3-001 closure; AC-011) | .factory spec |

No new files created in `crates/`. New test functions go into existing test files.
If `e_cred_namespace_invariants.rs` is created as a new test file, add it to the
`crates/prism-core/tests/` directory — no `Cargo.toml` change required for integration
test files in `tests/`.

**Subsystem anchor justification:** SS-03 (Credential Management) owns this story's
scope because all three affected crates (`prism-core`, `prism-credentials`,
`prism-mcp` error_mapping) implement the credential error surface per the ARCH-INDEX
Subsystem Registry definition of SS-03. The `.factory/` spec edits are maintenance-scoped
and follow the SS-03 crate boundary.

**Dependency anchor justification:**
- `depends_on: []` — this story has no runtime code dependency on any other unmerged
  story. It operates entirely on develop's current HEAD. ADR-035 is accepted and on
  the factory-artifacts branch.
- `blocks: [S-DEMO-003]` — S-DEMO-003 cannot resume its LOCAL cascade until this story
  merges and its worktree is re-baselined to E-CRED-008 (keyring path). The S-DEMO-003
  re-baseline is a hard downstream dependency on this migration. The block is documented
  in ADR-035 §S-DEMO-003-Impact.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `rg 'PrismError::KeyringError' crates/'` finds a production constructor outside `error_mapping.rs` | STOP — blast-radius miss in ADR-035. Escalate to architect before proceeding. Do not delete the variant. |
| EC-002 | A doc-test in `error.rs` or another file exercises `CredentialEncryptionError` and asserts `E-CRED-005:` in its output | Locate and update per TD-VSDD-060 sibling-site sweep. Run `cargo test --doc -p prism-core` to verify. |
| EC-003 | S-DEMO-003 worktree checkout is mounted and `git status` reports that `resolution.rs` is modified | This is the S-DEMO-003 branch in its paused state. It is out of scope. Do NOT apply this story's changes to that worktree. |
| EC-004 | ADR-034 prose contains additional E-CRED-005 references beyond the three noted in ADR-035 §D5 blast-radius | Update all occurrences found — this is a TD-VSDD-060 sibling-site sweep obligation. Do not stop at three if grep finds more. |
| EC-005 | `error-taxonomy.md` E-CRED-008 row note about S-DEMO-003 is questioned by adversary as "incomplete implementation" | Correct response: E-CRED-008 is forward-reserved per ADR-035 §D2. The row is complete; the emitter arrives with S-DEMO-003. This is an intentional documented forward-definition. |
| EC-006 | `just check` fails after deleting `KeyringError` due to a non-exhaustive match in another crate | Apply TD-VSDD-060: grep `'KeyringError'` across all crates, update every match arm. This story's scope covers `error_mapping.rs`; any additional sites found are in scope via sibling-site sweep. |

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|----------------|
| `PrismError::CredentialEncryptionError` (renumber) | prism-core | `src/error.rs` | Pure (enum variant, no I/O) |
| `PrismError::EncryptionKeyMissing` (renumber) | prism-core | `src/error.rs` | Pure |
| `PrismError::KeyringError` (retire) | prism-core | `src/error.rs` | Pure |
| MCP error mapping | prism-mcp | `src/error_mapping.rs` | Pure (match arm) |
| File-I/O reason strings | prism-credentials | `src/resolve_secret.rs` | Effectful (file I/O path) |
| Error display tests | prism-core | `tests/ac_5_prism_error_display.rs` | Pure (test) |
| resolve_secret tests | prism-credentials | `tests/bc_2_03_009_resolve_secret.rs` | Effectful (file I/O in test) |
| error-taxonomy.md | .factory/specs | `prd-supplements/error-taxonomy.md` | N/A — spec |
| BC-2.06.003 | .factory/specs | `behavioral-contracts/BC-2.06.003-credential-reference-resolution.md` | N/A — spec |
| ADR-034 | .factory/specs | `architecture/decisions/ADR-034-tier3-keyring-resolution-org-id-threading.md` | N/A — spec |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~6 500 |
| ADR-035 (full — design authority) | ~4 500 |
| `prism-core/src/error.rs` (full PrismError enum) | ~2 000 |
| `prism-mcp/src/error_mapping.rs` | ~1 500 |
| `prism-credentials/src/resolve_secret.rs` | ~1 200 |
| `ac_5_prism_error_display.rs` (current tests) | ~1 000 |
| `bc_2_03_009_resolve_secret.rs` (current tests) | ~800 |
| `error-taxonomy.md` E-CRED section (read before rewrite) | ~800 |
| `BC-2.06.003` (relevant clauses) | ~1 000 |
| `ADR-034` (§D4 + relevant sections) | ~1 200 |
| `just check` output | ~500 |
| `rg` sibling-site sweep output | ~300 |
| **Total** | **~21 300** |

Context window headroom: ~21k tokens is ~6% of a 350k context window.
No splitting required. Implementer and product-owner can operate independently
(code changes vs. spec changes) without context conflict.

## §Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.2 | 2026-06-07 | state-manager | D-1046 POL-14 post-merge: status ready→merged; PR #175 squash-merged develop@c603741d; canonical E-CRED-001..010 namespace live |
| 1.1 | 2026-06-07 | story-writer | Add AC-011 (cross-spec E-CRED-002 propagation sweep; closes DF-PASS3-001): corrected `security-architecture.md` resolution-chain "Not found" node E-CRED-001 → E-CRED-002 and `interface-definitions.md` `credential_status` errors array E-CRED-001 → E-CRED-002; extended ADR-035 blast-radius inventory to list both docs; added immutable-narrative OUT OF SCOPE note for `S-1.07-credential-crud.md` and `STORY-INDEX.md` (POL-1 append-only); added touched-artifact rows to §FSR |
| 1.0 | 2026-06-07 | story-writer | Initial materialization from stub (STORY-INDEX D-1043 row) + ADR-035 (accepted 2026-06-07) |
