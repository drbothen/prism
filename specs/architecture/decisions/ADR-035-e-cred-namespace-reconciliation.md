---
document_type: adr
adr_id: "ADR-035"
title: "E-CRED Namespace Reconciliation — Canonical E-CRED-001..010 Error Codes, Collision Resolution, and Migration Mapping"
status: accepted
date: 2026-06-07
author: architect
decision_made_by: human (architect-designs-fresh directive, S-MAINT-ECRED-TAXONOMY-SYNC-001)
supersedes: null
superseded_by: null
related_adrs: ["ADR-034", "ADR-032", "ADR-026"]
related_bcs: ["BC-2.06.003", "BC-2.03.005", "BC-2.03.006", "BC-2.03.007", "BC-2.03.009"]
traces_to: "ARCH-INDEX.md"
subsystems_affected: ["SS-03"]
drift_anchor: "DRIFT-ECRED-TAXONOMY-001"
story_anchor: "S-MAINT-ECRED-TAXONOMY-SYNC-001"
---

# ADR-035: E-CRED Namespace Reconciliation — Canonical E-CRED-001..010 Error Codes, Collision Resolution, and Migration Mapping

## Status

Accepted. Human selected "architect designs fresh" — no existing source is presumed canonical.

Design phase (complete): This ADR was the gate for the migration burst of
S-MAINT-ECRED-TAXONOMY-SYNC-001. The human reviewed and approved this ADR, authorizing
execution.

Execution phase (complete as of 2026-06-07): The migration burst has executed under story
S-MAINT-ECRED-TAXONOMY-SYNC-001. Code renumbering (CredentialEncryptionError → E-CRED-006,
EncryptionKeyMissing → E-CRED-007, file-I/O string literals → E-CRED-005, KeyringError variant
retired, E-CRED-010 reserved), error-taxonomy.md, BC-2.06.003, ADR-034 §D4 (→ E-CRED-008),
security-architecture.md, and interface-definitions.md (not-found → E-CRED-002) are all updated
to the canonical namespace (E-CRED-008 keyring, E-CRED-002 not-found). Exact artifact versions
are recorded in the §Changelog rows below and the story changelog (git history of
feature/S-MAINT-ECRED-TAXONOMY-SYNC-001).
The S-DEMO-003 keyring emitter (E-CRED-008) remains forward-reserved pending that story's merge.

## Context

### The Three-Source Divergence

Three separate sources define E-CRED-NNN codes and they contradict each other:

**Source A — `error-taxonomy.md` v1.61** (the spec authority for error codes):

| Code | Message | Category |
|------|---------|----------|
| E-CRED-001 | "OS keyring unavailable: {platform_error}" | configuration |
| E-CRED-002 | "Encrypted file backend key material missing" | configuration |
| E-CRED-003 | "Credential decryption failed for ({client_id}, {sensor_id})" | authentication |
| E-CRED-004 | "Invalid credential name: '{name}' does not match [{pattern}]" | validation |
| E-CRED-005 | "E-CRED-005: OS keyring unavailable during Tier-3 credential resolution: {reason}" | authentication |

**Source B — `crates/prism-core/src/error.rs`** (the code authority for PrismError variants):

| Code | Variant | Display |
|------|---------|---------|
| E-CRED-001 | `InvalidCredentialName` | "E-CRED-001: invalid credential name '{name}': {reason}" |
| E-CRED-002 | `CredentialNotFound` | "E-CRED-002: credential not found: {name}" |
| E-CRED-003 | `CredentialAccessDenied` | "E-CRED-003: credential access denied for {name} — credential values never transit AI context" |
| E-CRED-004 | `CredentialStoreError` | "E-CRED-004: credential store error (backend={backend}): {reason}" |
| E-CRED-005 | `CredentialEncryptionError` | "E-CRED-005: credential encryption error: {reason}" |
| E-CRED-006 | `EncryptionKeyMissing` | "E-CRED-006: encryption key not configured: {reason}" |
| E-CRED-010 | `KeyringError` | "E-CRED-010: keyring error: {detail}" |

**Source C — `crates/prism-credentials/src/resolve_secret.rs`** (code emitting an undeclared code):

- `E-CRED-009`: Three subtypes of credential-file I/O failure (file missing, is-directory,
  unreadable). This code is emitted inline inside the `reason` field of
  `PrismError::InvalidCredentialName`. It is not declared in the taxonomy.

**Source D — `crates/prism-credentials/src/resolution.rs` (S-DEMO-003 branch)**:

- Emits `"E-CRED-005: OS keyring unavailable: {inner_detail}..."` inside
  `CredentialResolutionError::BackendUnavailable.detail`.
  This is the Tier-3 keyring-backend-error path per ADR-034 §D4.

### The CODE-INTERNAL COLLISION (DRIFT-ECRED-TAXONOMY-001)

`E-CRED-005` is simultaneously assigned to two entirely different conditions:

1. `PrismError::CredentialEncryptionError` (Source B) — displays
   `"E-CRED-005: credential encryption error: {reason}"`.
   Callers: `crates/prism-credentials/src/file.rs::EncryptedFileBackend::decrypt` (encryption
   failure on read), `prism-credentials/src/file.rs::encrypt` (encryption failure on write).

2. `CredentialResolutionError::BackendUnavailable.detail` (Source C, S-DEMO-003 branch) — embeds
   `"E-CRED-005: OS keyring unavailable: {reason}"` as a string literal.
   Callers: `prism-credentials/src/resolution.rs` Tier-3 keyring error branch.

These are not variants of the same condition — they are semantically orthogonal:
"The AES-GCM encryption operation failed on an encrypted-file backend" vs.
"The OS keyring service is unavailable at credential-resolution time."
A monitoring system or operator cannot distinguish them by code string prefix alone.
This is a production-grade defect: observable operator confusion and metric mis-tagging.

### Additional Gaps Found During Exhaustive Survey

1. **E-CRED-009 not in taxonomy.** `resolve_secret.rs` emits this code for three subtypes of
   Tier-1 credential-file I/O failure. The taxonomy has no E-CRED-009 row.

2. **E-CRED-010 not in taxonomy.** `PrismError::KeyringError` emits `"E-CRED-010: keyring error:
   {detail}"`. No taxonomy row exists. The variant is defined but has zero production callers
   outside `error_mapping.rs` — it is effectively reserved code without callers.

3. **Taxonomy E-CRED-001 (generic OS keyring unavailable) vs. taxonomy E-CRED-005 (Tier-3 OS
   keyring unavailable) represent a design question.** Are these genuinely distinct conditions or
   the same condition at different resolution tiers? Decision required (see §D3 below).

4. **Taxonomy E-CRED-003 (credential decryption failed) has no `PrismError` variant.** The
   spec declares it; the code does not implement it. The code's E-CRED-005 variant
   (`CredentialEncryptionError`) covers the encryption/decryption case but under a different
   code number.

5. **Taxonomy E-CRED-004 (invalid credential name — path traversal) and code E-CRED-001
   (`InvalidCredentialName`) cover the same surface.** The taxonomy conflates path-traversal
   validation with the code's general-purpose name-validation variant.

### ADR-034 §D4 Note

ADR-034 §D4 originally wrote `E-CRED-003` for the keyring-backend-error code, then corrected
it to `E-CRED-005` to avoid collision. The correction note in ADR-034 reads:
"ADR-034 §D4 originally designated E-CRED-003; that code was already allocated ... E-CRED-005 is
the next free code."
That correction propagated into BC-2.06.003 v1.4, error-taxonomy.md v1.61, and
S-DEMO-003. However, E-CRED-005 was already allocated in `prism-core/src/error.rs` to
`CredentialEncryptionError` — so the correction introduced the collision this ADR resolves.

## Decision

### D1: Canonical E-CRED-001..010 Namespace (First-Principles Design)

The following table is the authoritative canonical E-CRED namespace. Every condition that Prism
can produce is assigned exactly one code. No code is assigned to more than one condition.

| Code | Canonical Name | Condition | Display Format | Category | Retryable |
|------|---------------|-----------|----------------|----------|-----------|
| E-CRED-001 | InvalidCredentialName | Credential name fails validation — empty, path-traversal characters, or forbidden pattern | `"E-CRED-001: invalid credential name '{name}': {reason}"` | validation | No |
| E-CRED-002 | CredentialNotFound | Credential is not configured in any tier of the resolution chain | `"E-CRED-002: credential not found: {name}"` | configuration | No |
| E-CRED-003 | CredentialAccessDenied | Credential value access blocked at AI-opaque safety boundary | `"E-CRED-003: credential access denied for {name} — credential values never transit AI context"` | security | No |
| E-CRED-004 | CredentialStoreError | Backend-level credential store operation failed (read/write/delete) | `"E-CRED-004: credential store error (backend={backend}): {reason}"` | infrastructure | No |
| E-CRED-005 | CredentialFileIo | Tier-1 file-backed credential: file missing, is-directory, or unreadable | `"E-CRED-005: credential file I/O error for '{path}': {reason}"` | configuration | No |
| E-CRED-006 | CredentialEncryptionError | Encryption or decryption failure on encrypted-file backend | `"E-CRED-006: credential encryption error: {reason}"` | authentication | No |
| E-CRED-007 | EncryptionKeyMissing | Encryption passphrase/key not configured for encrypted-file backend | `"E-CRED-007: encryption key not configured: {reason}"` | configuration | No |
| E-CRED-008 | KeyringBackendUnavailable | OS keyring service is unavailable (locked, D-Bus down, spawn panic) at any resolution tier | `"E-CRED-008: OS keyring unavailable: {reason}"` | infrastructure | No |
| E-CRED-009 | CredentialDecryptionFailed | Credential data decryption failed — key material changed or file corrupted | `"E-CRED-009: credential decryption failed for ({client_id}, {sensor_id}): {reason}"` | authentication | No |
| E-CRED-010 | (RESERVED) | Reserved for future use | — | — | — |

**Design rationale for each assignment — see §Rationale below.**

### D2: Collision Resolution

The E-CRED-005 collision is resolved by renumbering:

- `CredentialEncryptionError` (was E-CRED-005 in prism-core) → **E-CRED-006**
- `EncryptionKeyMissing` (was E-CRED-006 in prism-core) → **E-CRED-007**
- OS keyring unavailable (was E-CRED-005 in taxonomy + ADR-034 + BC-2.06.003) → **E-CRED-008**
- Tier-1 file I/O failure (was E-CRED-009 in resolve_secret.rs, undeclared in taxonomy) → **E-CRED-005**
- Credential decryption failed (was E-CRED-003 in taxonomy only, not implemented in code) → **E-CRED-009**

E-CRED-010 is reserved. `PrismError::KeyringError` (was E-CRED-010, zero production callers)
is merged into `KeyringBackendUnavailable` (E-CRED-008) in the canonical namespace — the
`KeyringError` variant is subsumed because its semantic condition ("keyring backend error") is
identical to `KeyringBackendUnavailable`. See §D4 for the `KeyringError` variant disposition.

### D3: Single Keyring-Unavailable Code (Not Two)

The taxonomy currently defines both E-CRED-001 ("OS keyring unavailable: {platform_error}",
generic) and E-CRED-005 ("OS keyring unavailable during Tier-3 ... resolution: {reason}",
Tier-3 specific). Decision: **these are the same condition and merge into one code (E-CRED-008)**.

Rationale: The triggering condition in both cases is identical — the OS keyring service is
inaccessible. The resolution tier is context for the operator (available in the audit log's
`source` field), not a structurally distinct error condition. Having two codes for the same
condition violates the 1:1 code↔condition invariant and forces monitoring rules to OR across
two codes for the same alert. A single `E-CRED-008: OS keyring unavailable: {reason}` with a
`source` field in the surrounding audit log context fully satisfies operator needs.

Taxonomy's generic E-CRED-001 ("OS keyring unavailable") had no corresponding `PrismError`
variant in the code — it was a spec-only artifact that never fired. Its semantic content is fully
covered by E-CRED-008.

### D4: `PrismError::KeyringError` Variant Disposition

`PrismError::KeyringError` (currently emits E-CRED-010) has zero production callers outside
`error_mapping.rs` (which pattern-matches it). It was likely scaffolded as an early placeholder
that was never wired up because `prism-credentials` uses `CredentialResolutionError` for
keyring-level errors and only surfaces them to `prism-core` via `CredentialStoreError` or
`BackendUnavailable`. Decision: **the `KeyringError` variant is retired and removed from
`PrismError`**. All keyring backend failures that currently reach `PrismError` do so via
`CredentialStoreError` (E-CRED-004); those that are surfaced from `resolve_credential` reach
the MCP layer via `SpecEngineError::AuthAcquisitionFailed` (not a `PrismError::KeyringError`).
The `error_mapping.rs` arm for `KeyringError` is deleted with the variant. If callers are ever
needed in the future, the code E-CRED-008 (`KeyringBackendUnavailable`) is the canonical slot.

### D5: ADR-034 Relationship

ADR-034 §D4 defined the keyring-backend-error code as `E-CRED-005` (corrected from E-CRED-003).
This ADR assigns that condition to **E-CRED-008** instead. ADR-034 §D4 is **amended** by this
ADR for the specific purpose of the error code assignment:

- All behavioral decisions in ADR-034 (Tier-3 resolution semantics, `Option<&OrgId>` parameter
  injection, `CredentialStoreOrgId::set_by_org` write reconciliation, SOUL.md §4 hard-error
  rationale) remain fully in effect. This ADR amends ONLY the error code designation.
- ADR-034 §D4 table row "Keyring backend error → `BackendUnavailable { detail: "E-CRED-005: ...
  }" }`" is updated to read `"E-CRED-008: OS keyring unavailable: {reason}"`.
- Bidirectional back-ref: ADR-034 frontmatter `related_adrs` gains `"ADR-035"`;
  this ADR's frontmatter `related_adrs` lists `"ADR-034"`.

ADR-034 is **not superseded** — the supersedes/superseded_by mechanism applies when an ADR
replaces another ADR's primary decision. Here, ADR-034's primary decision (Tier-3 implementation
strategy) is unaffected. Only the ancillary error-code allocation is amended.

## Rationale

### Rationale for E-CRED-001: `InvalidCredentialName`

The code currently emits `"E-CRED-001"` for `PrismError::InvalidCredentialName`. The
condition — "credential name fails validation" — is semantically stable and semantically
distinct from all other conditions. The Display format already begins with `"E-CRED-001:"`.
Preserving E-CRED-001 for this condition minimizes churn (the one existing test that asserts
`E-CRED-001` — `ac_5_prism_error_display.rs::test_ac5_prism_error_display_e_cred_001` — remains
unchanged). The taxonomy's E-CRED-004 ("Invalid credential name ... path traversal") covers a
subset of the same condition: it is subsumed by E-CRED-001's general-purpose validation scope.
Maintaining a separate code for the path-traversal sub-case would require callers to distinguish
sub-reasons by code number — correct design routes sub-reason information into the `{reason}`
field of E-CRED-001, not into a separate code.

### Rationale for E-CRED-002: `CredentialNotFound`

The code emits `"E-CRED-002: credential not found: {name}"` for `PrismError::CredentialNotFound`.
Preserved as-is. This is a clean 1:1 condition-to-code assignment.

### Rationale for E-CRED-003: `CredentialAccessDenied`

The code emits `"E-CRED-003: credential access denied..."` for `PrismError::CredentialAccessDenied`.
Preserved as-is. This condition — blocking a credential-value access at the AI-opaque safety
boundary — is structurally distinct from all other conditions and has an explicit MCP mapping
(`-32002 Forbidden` in `error_mapping.rs`). The taxonomy's E-CRED-003 (credential decryption
failed) is reassigned to E-CRED-009 to resolve the conflict.

### Rationale for E-CRED-004: `CredentialStoreError`

The code emits `"E-CRED-004: credential store error (backend={backend}): {reason}"` for
`PrismError::CredentialStoreError`. Preserved as-is. Covers backend-level failures (RocksDB lock,
deserialization error, keyring write failure). The S-DEMO-003 runbook references
`E-CRED-004` for write-path keyring failures — preserved correctly.

### Rationale for E-CRED-005: `CredentialFileIo` (RENUMBERED from E-CRED-009)

`resolve_secret.rs` emits `"E-CRED-009"` for three Tier-1 file-I/O sub-conditions. That code
is currently undeclared in the taxonomy. Moving to E-CRED-005 fills the gap left when the
taxonomy's former E-CRED-005 (keyring-unavailable) is renumbered to E-CRED-008. This preserves
sequential density in the namespace (E-CRED-001 through E-CRED-009 are all occupied, no gaps
through 009). The Display format is regularized to a single format string:
`"E-CRED-005: credential file I/O error for '{path}': {reason}"` where `{reason}` carries
the sub-condition (e.g., "file does not exist", "path is a directory", "read failed: {io_err}").
Using one code with sub-reason in `{reason}` rather than three sub-codes avoids code proliferation
for what is semantically one condition ("Tier-1 file backend is broken").

### Rationale for E-CRED-006: `CredentialEncryptionError` (RENUMBERED from E-CRED-005)

Currently emits `"E-CRED-005: credential encryption error: {reason}"` — this is one side of
the collision. Renumbering to E-CRED-006 resolves the collision. The condition itself is clean
and distinct: AES-GCM or equivalent cipher failure on the encrypted-file backend.

### Rationale for E-CRED-007: `EncryptionKeyMissing` (RENUMBERED from E-CRED-006)

Currently emits `"E-CRED-006: encryption key not configured: {reason}"`. Renumbering to
E-CRED-007 (one step up) because E-CRED-006 is now occupied by `CredentialEncryptionError`.
The condition — passphrase/key env var not set — is logically the prerequisite failure for
E-CRED-006 (you cannot encrypt/decrypt without the key). Sequential numbering 006/007 preserves
the logical relationship.

### Rationale for E-CRED-008: `KeyringBackendUnavailable` (new canonical home for merged keyring conditions)

This code consolidates three previously scattered assignments:
- Taxonomy E-CRED-001 (generic OS keyring unavailable) — was spec-only, no code caller
- Taxonomy E-CRED-005 / ADR-034 §D4 (Tier-3 OS keyring unavailable) — live in S-DEMO-003 branch
- `PrismError::KeyringError` (E-CRED-010) — declared but zero production callers

Number 008 chosen because it is sequential after 007 and above the existing 001-007 range,
leaving 009 available for the credential-decryption condition.

### Rationale for E-CRED-009: `CredentialDecryptionFailed` (new code for taxonomy E-CRED-003 condition)

The taxonomy's E-CRED-003 ("Credential decryption failed for ({client_id}, {sensor_id})") was
declared in the spec but never implemented in code — there is no `PrismError` variant for it.
The condition is real and will need a code when the encrypted-file backend decryption path is
fully implemented. E-CRED-009 (vacated by E-CRED-005's renaming of the former 009 to 005) is
the canonical slot. The Display format includes both `client_id` and `sensor_id` per the taxonomy
to provide operator context.

### Rationale for E-CRED-010: RESERVED

E-CRED-010 was the number of the retired `PrismError::KeyringError` variant. Reserving it rather
than recycling it prevents confusing any historical log entries that may contain
`"E-CRED-010: keyring error:"`. Future extension of the namespace should use E-CRED-011 onward.

## Full Migration Mapping Table

This table is the spec that the downstream implementer (code changes) and product-owner
(taxonomy + BC changes) execute against. Every current code → canonical code mapping is exact.

### Code Renumbering Summary

| Current Code | Current Condition | Current Source | Canonical Code | Action |
|-------------|-------------------|----------------|----------------|--------|
| E-CRED-001 | InvalidCredentialName | prism-core/error.rs | **E-CRED-001** | Preserve — no change |
| E-CRED-001 | OS keyring unavailable (generic) | error-taxonomy.md only | Retired — subsumed by E-CRED-008 | Remove taxonomy row |
| E-CRED-002 | CredentialNotFound | prism-core/error.rs | **E-CRED-002** | Preserve — no change |
| E-CRED-002 | Encrypted file backend key material missing | error-taxonomy.md only | Retired — subsumed by E-CRED-007 | Remove taxonomy row |
| E-CRED-003 | CredentialAccessDenied | prism-core/error.rs | **E-CRED-003** | Preserve — no change |
| E-CRED-003 | Credential decryption failed | error-taxonomy.md only | **E-CRED-009** | Replace taxonomy row with E-CRED-009 |
| E-CRED-004 | InvalidCredentialName (path-traversal) | error-taxonomy.md only | Retired — subsumed by E-CRED-001 | Remove taxonomy row |
| E-CRED-004 | CredentialStoreError | prism-core/error.rs | **E-CRED-004** | Preserve — no change |
| E-CRED-005 | CredentialEncryptionError | prism-core/error.rs | **E-CRED-006** | Rename: E-CRED-005 → E-CRED-006 |
| E-CRED-005 | OS keyring unavailable (Tier-3) | error-taxonomy.md + ADR-034 + BC-2.06.003 + S-DEMO-003 | **E-CRED-008** | Replace: E-CRED-005 → E-CRED-008 |
| E-CRED-006 | EncryptionKeyMissing | prism-core/error.rs | **E-CRED-007** | Rename: E-CRED-006 → E-CRED-007 |
| E-CRED-009 | CredentialFileIo (file missing/is-dir/unreadable) | resolve_secret.rs only | **E-CRED-005** | Renumber: E-CRED-009 → E-CRED-005 + add taxonomy row |
| E-CRED-010 | KeyringError (zero callers) | prism-core/error.rs | RETIRED | Remove variant; reserve code |

### Exact Display String Changes

| Canonical Code | Canonical Display String |
|----------------|--------------------------|
| E-CRED-001 | `"E-CRED-001: invalid credential name '{name}': {reason}"` (unchanged) |
| E-CRED-002 | `"E-CRED-002: credential not found: {name}"` (unchanged) |
| E-CRED-003 | `"E-CRED-003: credential access denied for {name} — credential values never transit AI context"` (unchanged) |
| E-CRED-004 | `"E-CRED-004: credential store error (backend={backend}): {reason}"` (unchanged) |
| E-CRED-005 | `"E-CRED-005: credential file I/O error for '{path}': {reason}"` (new — regularized from 3 subtypes) |
| E-CRED-006 | `"E-CRED-006: credential encryption error: {reason}"` (renumbered from 005) |
| E-CRED-007 | `"E-CRED-007: encryption key not configured: {reason}"` (renumbered from 006) |
| E-CRED-008 | `"E-CRED-008: OS keyring unavailable: {reason}"` (new — merged from taxonomy 001+005 + code 010) |
| E-CRED-009 | `"E-CRED-009: credential decryption failed for ({client_id}, {sensor_id}): {reason}"` (new — was taxonomy-only E-CRED-003) |

## Blast-Radius Inventory

### Owner: implementer (code + test changes)

**`crates/prism-core/src/error.rs`**

| Item | Change |
|------|--------|
| `PrismError::CredentialEncryptionError` doc comment | Update: `/// E-CRED-005:` → `/// E-CRED-006:` |
| `PrismError::CredentialEncryptionError` `#[error]` | Update: `"E-CRED-005: credential encryption error: {reason}"` → `"E-CRED-006: credential encryption error: {reason}"` |
| `PrismError::EncryptionKeyMissing` doc comment | Update: `/// E-CRED-006:` → `/// E-CRED-007:` |
| `PrismError::EncryptionKeyMissing` `#[error]` | Update: `"E-CRED-006: encryption key not configured: {reason}"` → `"E-CRED-007: encryption key not configured: {reason}"` |
| `PrismError::KeyringError` variant | DELETE entire variant (doc comment + `#[error]` + field) |

**`crates/prism-mcp/src/error_mapping.rs`**

| Item | Change |
|------|--------|
| `// E-CRED-003:` comment above `CredentialAccessDenied` arm | Update code reference in comment only |
| `PrismError::KeyringError { .. }` arm | DELETE arm (variant is removed) |

**`crates/prism-credentials/src/resolve_secret.rs`**

| Item | Change |
|------|--------|
| `PrismError::InvalidCredentialName { reason: "E-CRED-009: credential file does not exist..."` | Update: replace `"E-CRED-009: credential file does not exist at path '{}' (env var '{}')"` with `"E-CRED-005: credential file I/O error for '{}': file does not exist (configured in env var '{}')"` |
| `PrismError::InvalidCredentialName { reason: "E-CRED-009: path '{}' points to a directory..."` | Update: replace `"E-CRED-009: path '{}' points to a directory, not a regular file..."` with `"E-CRED-005: credential file I/O error for '{}': path is a directory, not a regular file"` |
| `PrismError::InvalidCredentialName { reason: "E-CRED-009: failed to read credential file..."` | Update: replace `"E-CRED-009: failed to read credential file '{}': {}"` with `"E-CRED-005: credential file I/O error for '{}': read failed: {}"` |

Note: `resolve_secret.rs` embeds the E-CRED code as a string literal inside the `reason` field
of `PrismError::InvalidCredentialName`. The correct long-term design would introduce a dedicated
`PrismError::CredentialFileIo` variant with `#[error("E-CRED-005: ...")]` — but this structural
refactor is separable from the code-number migration and may be done in a follow-up story. The
immediate fix is updating the string literals from `E-CRED-009` to `E-CRED-005`.

**`crates/prism-credentials/src/resolution.rs` (S-DEMO-003 worktree branch)**

| Item | Change |
|------|--------|
| `BackendUnavailable { detail: format!("E-CRED-005: invalid credential name for Tier-3 lookup: {e}") }` | Update: `"E-CRED-008: OS keyring unavailable: invalid credential name for Tier-3 lookup: {e}"` |
| `BackendUnavailable { detail: format!("E-CRED-005: OS keyring unavailable: {inner_detail}. ...") }` | Update: `"E-CRED-008: OS keyring unavailable: {inner_detail}. Check keyring access..."` |

**`crates/prism-core/tests/ac_5_prism_error_display.rs`**

| Item | Change |
|------|--------|
| `test_ac5_prism_error_display_e_cred_001` | No change — `E-CRED-001` is preserved |

**`crates/prism-credentials/tests/bc_2_03_005_credential_crud.rs`**

| Item | Change |
|------|--------|
| `msg.contains("E-CRED-001")` assertion | No change — `E-CRED-001` is preserved |

**`crates/prism-credentials/tests/bc_2_03_009_resolve_secret.rs`**

| Item | Change |
|------|--------|
| `msg.contains("E-CRED")` assertion (loose) | Consider tightening to `msg.contains("E-CRED-005")` now that the code is defined |

**`crates/prism-credentials/tests/bc_2_06_003_tier3_keyring_resolution.rs` (S-DEMO-003 worktree)**

| Item | Change |
|------|--------|
| `detail.contains("E-CRED-005")` assertion (RG-034-005) | Update: `detail.contains("E-CRED-008")` |
| `"BackendUnavailable.detail must contain 'E-CRED-005'"` message | Update: `'E-CRED-008'` |

**`crates/prism-credentials/src/in_memory_store.rs` (S-DEMO-003 worktree)**

| Item | Change |
|------|--------|
| Doc comments referencing `"E-CRED-005: OS keyring unavailable: backend={backend}: ..."` | Update to `"E-CRED-008: OS keyring unavailable: ..."` |

### Owner: product-owner (taxonomy + BC changes)

**`.factory/specs/prd-supplements/error-taxonomy.md`**

| Current Row | Action |
|------------|--------|
| `E-CRED-001` ("OS keyring unavailable: {platform_error}") | REMOVE row — condition subsumed by canonical E-CRED-008 |
| `E-CRED-002` ("Encrypted file backend key material missing") | REMOVE row — condition subsumed by canonical E-CRED-007 |
| `E-CRED-003` ("Credential decryption failed for ({client_id}, {sensor_id})") | UPDATE code to E-CRED-009; update message to `"E-CRED-009: credential decryption failed for ({client_id}, {sensor_id}): {reason}"` |
| `E-CRED-004` ("Invalid credential name: ... path traversal") | REMOVE row — condition subsumed by canonical E-CRED-001 |
| `E-CRED-005` ("E-CRED-005: OS keyring unavailable during Tier-3...") | REPLACE entire row with E-CRED-008 |
| ADD E-CRED-001 row | ADD: `E-CRED-001 | broken | validation | "E-CRED-001: invalid credential name '{name}': {reason}" | No | Name fails validation: empty, path-traversal chars, forbidden pattern` |
| ADD E-CRED-002 row | ADD: `E-CRED-002 | broken | configuration | "E-CRED-002: credential not found: {name}" | No | Credential not configured in any resolution tier` |
| ADD E-CRED-003 row | ADD: `E-CRED-003 | broken | security | "E-CRED-003: credential access denied for {name} — credential values never transit AI context" | No | AI-opaque safety boundary: credential value access blocked` |
| ADD E-CRED-004 row | ADD: `E-CRED-004 | broken | infrastructure | "E-CRED-004: credential store error (backend={backend}): {reason}" | No | Backend-level storage op failed (RocksDB, keyring write, etc.)` |
| ADD E-CRED-005 row | ADD: `E-CRED-005 | broken | configuration | "E-CRED-005: credential file I/O error for '{path}': {reason}" | No | Tier-1 file-backed credential: file missing, is-directory, or unreadable` |
| ADD E-CRED-006 row | ADD: `E-CRED-006 | broken | authentication | "E-CRED-006: credential encryption error: {reason}" | No | AES-GCM or equivalent cipher failure on encrypted-file backend` |
| ADD E-CRED-007 row | ADD: `E-CRED-007 | broken | configuration | "E-CRED-007: encryption key not configured: {reason}" | No | Passphrase/key env var not set for encrypted-file backend` |
| ADD E-CRED-008 row | ADD: `E-CRED-008 | broken | infrastructure | "E-CRED-008: OS keyring unavailable: {reason}" | No | OS keyring service inaccessible (locked, D-Bus down, spawn panic) at any resolution tier. Operator remedy: unlock keyring / restart D-Bus, or use Tier 1/2 env vars. Maps to CredentialResolutionError::BackendUnavailable in prism_credentials::resolution.` |
| ADD E-CRED-009 row | ADD: `E-CRED-009 | broken | authentication | "E-CRED-009: credential decryption failed for ({client_id}, {sensor_id}): {reason}" | No | Key material changed or encrypted file corrupted` |
| ADD E-CRED-010 row | ADD: `E-CRED-010 | — | — | RESERVED | — | Reserved; do not allocate` |

**`BC-2.06.003`** (`credential-reference-resolution.md`)

| Item | Change |
|------|--------|
| Postconditions Tier-3 table: `BackendUnavailable { detail: "E-CRED-005: OS keyring unavailable..." }` | Update to `"E-CRED-008: OS keyring unavailable: {reason}"` |
| Postconditions invariant note: `E-CRED-005 detail is a system error message` | Update to `E-CRED-008` |
| Canonical Test Vectors: Tier-3 backend error TV output column | Update: `E-CRED-005` → `E-CRED-008` |
| Frontmatter: add ADR-035 to normative references | Add `ADR-035` |

### Owner: architect (this ADR + ARCH-INDEX.md row + architecture section docs)

**`ARCH-INDEX.md`**

| Item | Change |
|------|--------|
| ADR Registry table | Add ADR-035 row (done by state-manager post-commit) |

**`.factory/specs/architecture/security-architecture.md`**

| Item | Change |
|------|--------|
| Credential Resolution Order diagram — "Credential not found" terminal node | Corrected `E-CRED-001` → `E-CRED-002`. The terminal node of the credential resolution flowchart (the `ENVCONV → Not found` branch) previously displayed `E-CRED-001` but the canonical condition is `CredentialNotFound` = E-CRED-002. E-CRED-001 = `InvalidCredentialName`. Closed by DF-PASS3-001 / S-MAINT-ECRED-TAXONOMY-SYNC-001. |

**`.factory/specs/prd-supplements/interface-definitions.md`** (product-owner scope — corrected in same burst)

| Item | Change |
|------|--------|
| E-CRED-001 "credential not found" mislabel | Corrected `E-CRED-001` → `E-CRED-002` wherever the interface-definitions doc references the "credential not found" condition using the wrong code. Same root cause as the security-architecture mislabel: pre-ADR-035 drafts used the stale taxonomy code. Closed by DF-PASS3-001 / S-MAINT-ECRED-TAXONOMY-SYNC-001. |

**`ADR-034`** (`tier3-keyring-resolution-org-id-threading.md`)

| Item | Change |
|------|--------|
| Frontmatter `related_adrs` | Add `"ADR-035"` |
| §D4 table row "Keyring backend error" | Update detail string: `"E-CRED-003: OS keyring unavailable: {reason}"` (correction note) → `"E-CRED-008: OS keyring unavailable: {reason}"` |
| §D4 text referencing `E-CRED-005` | Update references to `E-CRED-008` |
| §Consequences Negative: `E-CRED-003 gives operators...` | Update: `E-CRED-008 gives operators...` |
| §File Create / Modify List `error-taxonomy.md` row | Update: `Add E-CRED-003 entry` → `Add E-CRED-008 entry (per ADR-035 canonical namespace)` |

### Owner: story-writer / state-manager (story spec + story index)

**`.factory/stories/S-DEMO-003-demo-setup-scripts-and-runbook.md`**

All S-DEMO-003 references to `E-CRED-005` (keyring path) require re-baselining to `E-CRED-008`.
All S-DEMO-003 references to `E-CRED-004` (write-path store error) are preserved (no change).
See §S-DEMO-003 Impact section below.

## S-DEMO-003 Impact

S-DEMO-003 is paused mid-cascade in worktree `.worktrees/S-DEMO-003`. It aligned its
DEMO-RUNBOOK, AC-006, AC-011, EC-001, EC-001b, EC-008, RG-034-005, and Architecture Compliance
Rules to the CURRENT E-CRED-005 (keyring-unavailable) and E-CRED-004 (write-path store error).

After this ADR lands and the migration burst executes:

**S-DEMO-003 items that require re-baselining (E-CRED-005 → E-CRED-008):**

| S-DEMO-003 Location | Current Reference | Required Change |
|--------------------|-------------------|-----------------|
| Story scope note (line ~19) | "E-CRED-005" | → "E-CRED-008" |
| AC-006 architecture compliance row | "E-CRED-005" | → "E-CRED-008" |
| AC-011 Case B detail string | `"E-CRED-005: OS keyring unavailable: backend=<backend>: <reason>..."` | → `"E-CRED-008: OS keyring unavailable: <reason>..."` |
| AC-011 Case B trace reference | "BC-2.06.003 Tier-3 postcondition: ... E-CRED-005" | → E-CRED-008 |
| EC-001b expected output | `"E-CRED-005: OS keyring unavailable: {reason}"` | → `"E-CRED-008: OS keyring unavailable: {reason}"` |
| EC-008 expected output | "BackendUnavailable / E-CRED-005" | → "BackendUnavailable / E-CRED-008" |
| DEMO-RUNBOOK.md §6(b) troubleshooting | "E-CRED-005" | → "E-CRED-008" |
| Red Gate test `test_BC_2_06_003_tier3_backend_error_returns_e_cred_005` | Function name + assertion | Rename + update assertion |
| Story implementation task list: task 23 (DEMO-RUNBOOK) | "references E-CRED-005" | → "references E-CRED-008" |
| Architecture Compliance Rules table, Tier-3 error row | `"BackendUnavailable { detail: "E-CRED-005:..."}"` | → `"E-CRED-008:..."` |

**S-DEMO-003 items that are UNCHANGED (E-CRED-004 is preserved):**

| S-DEMO-003 Location | Reference | Status |
|--------------------|-----------|--------|
| DEMO-RUNBOOK §6(a) troubleshooting | "E-CRED-004" (write-path store error) | Preserved — no change |
| EC-001 expected write-path error | "E-CRED-004 — PrismError::CredentialStoreError" | Preserved — no change |
| AC-006 description | "§6(a) keyring write / E-CRED-004" | Preserved — no change |

**Re-baseline procedure for S-DEMO-003:** After the migration burst merges, the product-owner
re-baselines S-DEMO-003 story spec in the worktree using the above table. The S-DEMO-003 TDD
restart (currently paused) uses the re-baselined story spec as its source of truth.

## Consequences

### Positive

- Zero collisions: every E-CRED code maps to exactly one semantic condition.
- Monitoring rules and alerting can use a single code prefix for each condition class.
- `resolve_secret.rs` E-CRED code is now declared in the taxonomy, closing the
  undeclared-code gap.
- `PrismError::KeyringError` (zero callers, confusing name overlap with
  `KeyringBackendUnavailable`) is removed, shrinking the PrismError enum by one variant.
- A single E-CRED-008 code covers the merged keyring-unavailable conditions (formerly
  split across taxonomy 001+005 and code 010), reducing operator alert rule complexity.
- The taxonomy is now the definitive authority — every code declared in the taxonomy matches
  a `PrismError` variant or `CredentialResolutionError` site.

### Negative / Trade-offs

- E-CRED-005, E-CRED-006, E-CRED-007, E-CRED-008 are number-changed from their current
  meanings. Any external system that currently indexes by exact code string will see new codes
  after migration. Mitigation: this is a pre-v1 system with no external customers; the migration
  is bounded to the internal codebase + factory spec artifacts.
- S-DEMO-003 must re-baseline 10+ references to `E-CRED-005` → `E-CRED-008` before resuming.
- `resolve_secret.rs` embeds the code as a string literal in a `reason` field (not in the
  variant's `#[error]`). The correct structural fix (new `PrismError::CredentialFileIo` variant)
  is deferred to a follow-up story. The immediate fix (update string literals) is correct but
  architecturally impure.

### Status as of 2026-06-07

Accepted. Design phase complete. Migration burst executed and complete.

The human approved this ADR and authorized execution. Story S-MAINT-ECRED-TAXONOMY-SYNC-001 has
delivered the full migration: code renumbering (CredentialEncryptionError → E-CRED-006,
EncryptionKeyMissing → E-CRED-007, file-I/O string literals → E-CRED-005, KeyringError variant
deleted, E-CRED-010 reserved); error-taxonomy.md, BC-2.06.003, ADR-034 §D4 (→ E-CRED-008),
security-architecture.md, and interface-definitions.md (not-found → E-CRED-002) are all updated
to the canonical namespace (E-CRED-008 keyring, E-CRED-002 not-found). Exact artifact versions
are recorded in the §Changelog rows below and the story changelog. The S-DEMO-003 keyring
emitter (E-CRED-008) is forward-reserved pending that story's merge. F-P5-MED-001 closed.

## Alternatives Considered

**Option A (status-quo bias — adopt code as canonical):** Declare `prism-core/src/error.rs`
as the canonical source, rewrite the taxonomy to match it. Rejected because this would bless
the E-CRED-005 collision rather than resolving it, and would lose the spec-declared conditions
(E-CRED-003 decryption, E-CRED-001 generic keyring) that the code never implemented.

**Option B (status-quo bias — adopt taxonomy as canonical):** Declare `error-taxonomy.md` as
canonical, rewrite the code to match it. Rejected because the taxonomy has its own issues (two
keyring-unavailable codes, no code for `CredentialNotFound`, no code for
`CredentialAccessDenied`, a decryption-failed code with no implementation), and the code has
real callers that test specific code strings.

**Option C (maximum numbering preservation — patch the collision only):** Assign E-CRED-011
to `CredentialEncryptionError` to free E-CRED-005 for the keyring-unavailable code. Rejected
because it leaves the namespace with a gap (010 reserved/retired, 011 assigned) and does not
address the undeclared E-CRED-009 or the two-keyring-unavailable problem. First-principles
design produces a cleaner result at minimal additional churn.

## Source / Origin

- DRIFT-ECRED-TAXONOMY-001 — drift anchor for this reconciliation work
- `crates/prism-core/src/error.rs` — E-CRED-001..006 + E-CRED-010 variants (PrismError enum)
- `crates/prism-credentials/src/resolve_secret.rs` — E-CRED-009 inline strings
- `crates/prism-credentials/src/resolution.rs` (S-DEMO-003 branch) — E-CRED-005 inline strings
- `.factory/specs/prd-supplements/error-taxonomy.md` v1.61 — E-CRED-001..005 taxonomy rows
- ADR-034 §D4 — error code allocation for keyring-backend-error (amended by this ADR)
- BC-2.06.003 v1.4 — E-CRED-005 cite in Tier-3 postconditions
- S-DEMO-003-demo-setup-scripts-and-runbook.md v1.14 — E-CRED-004 (write path) + E-CRED-005 (read path) references
- `crates/prism-mcp/src/error_mapping.rs` — E-CRED-* MCP mapping arms
- `crates/prism-core/tests/ac_5_prism_error_display.rs` — E-CRED-001 display assertion
- `crates/prism-credentials/tests/bc_2_03_005_credential_crud.rs` — E-CRED-001 string assertion
- `crates/prism-credentials/tests/bc_2_06_003_tier3_keyring_resolution.rs` (S-DEMO-003 worktree) — E-CRED-005 assertion

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v1.0 | 2026-06-07 | architect | Initial ADR — E-CRED namespace reconciliation design phase complete. |
| v1.1 | 2026-06-07 | architect | §Status and §Consequences/Status corrected from design-phase "not yet executed" to execution-complete state. F-P5-MED-001 (LOCAL pass-5 finding: stale status assertions) closed. Migration burst confirmed executed under S-MAINT-ECRED-TAXONOMY-SYNC-001 @ c95142d6. |
| v1.2 | 2026-06-07 | architect | F-P14-HIGH-001/ADV-P15-HIGH-001 closure — de-pinned volatile sibling version/SHA references from §Status and §Consequences/Status live prose per TD-VSDD-091 to terminate sibling-sweep cascade; S-MAINT-ECRED-TAXONOMY-SYNC-001. |
