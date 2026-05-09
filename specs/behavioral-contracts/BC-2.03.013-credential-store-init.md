---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-08T00:00:00Z
phase: 3
origin: greenfield
subsystem: "SS-03"
capability: "CAP-004"
lifecycle: draft
anchored_stories: [S-WAVE5-PREP-01]
verifying_vps: []
crates: [prism-bin, prism-credentials]
inputs:
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
  - .factory/specs/architecture/module-decomposition.md
input-hash: "[md5]"
traces_to: ["CAP-004"]
---

# BC-2.03.013: CredentialStore Initialization — Reference Validation Only, No Values in Memory at Process Start

## Description

This BC is the Credential Management subsystem's (SS-03) startup-time contract. It specifies
how `prism-bin` constructs the `CredentialStore` and validates credential references at boot step
5 (per ADR-022 §B). The orchestration of this and the other 3 subsystem init contracts in §B
order is specified separately in BC-2.22.001.

`prism-bin` constructs the `CredentialStore` and validates all credential references declared in
loaded sensor specs. CRITICAL: this step implements the AI-opaque credential model (AD-017) —
credential VALUES are never loaded into memory at init time. Only references (keyring key names,
environment variable names, file paths) are validated for resolvability. On any failure (reference
unresolvable, backend unavailable, permission denied), the process exits with code 5
(permission-denied) or code 2 (config-invalid reference). This step is BLOCKING: step 6 (audit
subsystem init) does not begin until credential store construction and reference validation complete.

No `todo!()`, `unimplemented!()`, or `panic!("stub...")` may appear in the production code path
for this step at or after story S-WAVE5-PREP-01 merges (POL-12 enforcement).

## Preconditions

- BC-2.06.011 is satisfied: valid `PrismConfig` handle available
- BC-2.21.001 is satisfied: valid `OrgRegistry` handle available
- Boot steps 2 and 3 have completed without error (ADR-022 §B ordering)
- Sensor TOML specs have been loaded in step 4 — credential references are known
- The credential backend (OS keyring or AES-encrypted file) is accessible to the process

## Postconditions

**Happy path:**
- `CredentialStore` handle is constructed using the backend declared in `prism.toml`
  (keyring or AES-file per BC-2.03.002 / BC-2.03.003)
- All credential references declared in sensor specs are validated as resolvable:
  - For each credential ref: the reference target EXISTS and the process has READ permission
  - Validation is reference-only — the actual credential value is NOT read, NOT decrypted,
    NOT stored in process memory at this point
- The `CredentialStore` handle (wrapped in `Arc`) is available to all subsequent boot steps
  for on-demand credential resolution at query time (BC-2.03.006)
- Boot continues to step 6 (audit subsystem init) per ADR-022 §B ordering

**Failure path — credential reference unresolvable:**
- A declared credential ref cannot be located in the backend (keyring entry missing, env var
  not set, file path does not exist)
- The process emits a `tracing::error!` naming the unresolvable reference
- The process exits with code **2** (config-invalid) per ADR-022 §A
- Step 6 never begins

**Failure path — permission denied:**
- The credential backend returns an access-denied error (keyring locked, file permission denied,
  env var inaccessible)
- The process emits a `tracing::error!` identifying the backend type and denied operation
- The process exits with code **5** (permission-denied) per ADR-022 §A
- Step 6 never begins

**Failure path — backend unavailable:**
- The credential backend cannot be opened (keyring service not running, encrypted file corrupted)
- The process emits a `tracing::error!` describing the backend failure
- The process exits with code **5** (permission-denied) per ADR-022 §A
- Step 6 never begins

## Invariants

- Boot step 5 is blocking: no concurrent execution with step 6 (ADR-022 §B "Traffic gate")
- **AI-opacity invariant (AD-017):** Credential values (API keys, passwords, tokens) are NEVER
  loaded into the process's heap memory during init. Only references are validated. Values are
  resolved on-demand at query time by the sensor adapter (BC-2.03.006). This is the central
  invariant of this BC.
- Exit codes: permission-denied failure → exit 5; config-invalid ref → exit 2; no other exit
  codes are valid for this step (ADR-022 §A canonical table)
- The `CredentialStore` public API for `new()` / `open()` MUST return no credential values —
  if the return type of any init function contains a credential value, that is an API design
  violation of AD-017 and must be rejected in code review

## Critical Invariant: Credential Non-Leak at Init Time

The following must be true immediately after `CredentialStore::open()` returns `Ok(store)`:
- `store` contains no decrypted credential value in any field
- No `String`, `SecretString`, `Vec<u8>`, or similar heap allocation in `store` holds
  a credential value
- The only heap data held by `store` is: reference metadata (key names, file paths, env var
  names), backend configuration (keyring service name, file path), and connectivity state

This invariant is the behavioral specification of the reference-based model. It is the
complement to BC-2.03.007 (secret redaction) and BC-2.03.003 (value resolution at query time).

## Error Cases

| Error Code | Condition | Behavior |
|------------|-----------|----------|
| Exit 2 | Credential reference declared in sensor spec but not found in backend | "Unresolvable credential ref: {ref_name} for sensor {sensor}"; exit 2 |
| Exit 5 | Backend returns permission denied (keyring locked, file chmod) | "Credential store access denied: {backend_type}: {detail}"; exit 5 |
| Exit 5 | Keyring service unavailable (daemon not running) | "Credential backend unavailable: keyring service not responding"; exit 5 |
| Exit 5 | AES-file backend exists but is corrupted (decryption fails) | "Credential backend unavailable: file backend decryption failed"; exit 5 |
| Exit 2 | Credential ref syntax invalid (malformed ref string) | "Invalid credential ref syntax: {ref}"; exit 2 |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-013-001 | No sensor specs declare any credential refs (empty validation set) | CredentialStore opens successfully; zero refs validated; boot continues |
| EC-03-013-002 | One of 10 credential refs is missing from backend | Exit 2 with name of missing ref; all refs reported (one-pass) |
| EC-03-013-003 | Keyring backend selected but OS keyring service not running (e.g., macOS Keychain locked) | Exit 5 with "keyring service not responding" |
| EC-03-013-004 | Environment variable reference (`$PRISM_CRED_API_KEY`) where env var exists but is empty string | Empty string is considered resolvable (the value is a valid empty string); boot continues; adapter behavior on empty credential is a query-time concern |
| EC-03-013-005 | File-backed credential reference where file exists but process has no read permission | Exit 5 with "permission denied: {path}" |
| EC-03-013-006 | AES-file backend with valid file, wrong decryption key | Backend unavailable; exit 5 |
| EC-03-013-007 | Backend selection from `prism.toml` specifies unknown backend type | Schema validation in step 2 would have caught this; cannot occur in step 5 |

## Canonical Test Vectors

| ID | Scenario | Setup | Expected Exit Code | Expected Log Output |
|----|----------|-------|-------------------|---------------------|
| TV-03-013-001 | All refs resolvable (keyring backend) | Keyring populated for all declared refs | Boot continues | `tracing::info!("CredentialStore initialized: {n} refs validated")` |
| TV-03-013-002 | Zero sensor specs, no refs | Empty spec_dir | Boot continues | `tracing::info!("CredentialStore initialized: 0 refs validated")` |
| TV-03-013-003 | Missing keyring entry | Ref declared in spec but not in keyring | 2 | "Unresolvable credential ref: {ref_name}" |
| TV-03-013-004 | Keyring service locked | macOS Keychain locked at boot | 5 | "Credential store access denied: keyring..." |
| TV-03-013-005 | File backend, permission denied | AES file exists, chmod 000 | 5 | "Credential store access denied: permission denied" |
| TV-03-013-006 | No values in CredentialStore after init | Inspect returned `CredentialStore` struct fields | N/A (unit test) | `store` fields contain only ref metadata, no secret values |

## Test Strategy

### Integration Tests (subprocess-level)

`crates/prism-bin/tests/boot_tests.rs`:
- `test_BC_2_03_013_valid_refs` — all credential refs resolvable; assert boot continues past step 5
- `test_BC_2_03_013_missing_ref` — one ref not in backend; assert exit code 2 + ref name in output
- `test_BC_2_03_013_permission_denied` — chmod 000 on AES file; assert exit code 5
- `test_BC_2_03_013_no_creds_needed` — empty spec_dir; assert boot continues

### Credential Non-Leak Invariant Test Strategy

Testing that "no credential value transits memory at init time" is a non-trivial assertion.
Three complementary approaches are recommended:

**Approach A (API surface inspection — preferred):** The `CredentialStore::open()` return type
MUST NOT include any field that is a secret value. This is testable by inspecting the struct's
field types in the type system. A unit test in `crates/prism-credentials/tests/` can assert
that `CredentialStore` fields are limited to ref metadata types (String for key names,
PathBuf for paths, etc.) and that no field is typed as `SecretString`, `Vec<u8>` raw bytes,
or similar. This is the cheapest and most maintainable approach — it's a type-level invariant.

**Approach B (runtime field inspection):** After `CredentialStore::open()` returns, call a
test-only function `store.debug_value_count() -> usize` that walks all ref slots and asserts
the count of non-`None` stored values is 0. This requires a test-gated API addition to the
struct, which is acceptable under `#[cfg(test)]`.

**Approach C (process memory scan — heavyweight, not required for P0):** A separate test
harness injects a known canary string into the expected credential value, calls
`CredentialStore::open()`, and scans `/proc/self/mem` (Linux) or uses `vm_read` (macOS) for
the canary. This is expensive and platform-specific. Treat as a post-P0 security hardening
test; not required for S-WAVE5-PREP-01.

**Recommendation:** Implement Approach A + B for S-WAVE5-PREP-01. Approach C is filed as a
future holdout scenario if the security team prioritizes it.

## Verification Properties

No formal VP is proposed for this BC. The AI-opacity invariant is enforced via:
1. API surface (Approach A above) — type-level check
2. Runtime field assertion (Approach B above) — unit test
3. Code review enforcement: any change to `CredentialStore` fields that introduces a
   secret-value type is a mandatory review blocker

Future VP candidate: a proptest property that generates random ref sets, calls `open()`,
and asserts all slots are empty of values. This would be filed as VP-NNN once the proptest
pattern is applied to `prism-credentials`.

## Related BCs

- BC-2.22.001 — Boot Orchestration (orchestrates: this BC is one of 4 subsystem init contracts
  whose ordering and exit-code mapping are specified in BC-2.22.001)
- BC-2.06.011 — Config load (depends on: this BC requires BC-2.06.011)
- BC-2.21.001 — OrgRegistry init (depends on: this BC requires BC-2.21.001)
- BC-2.05.012 — Audit subsystem init (depends on: step 6 follows this step)
- BC-2.03.001 — CredentialStore Trait with Tenant-Scoped Operations (composes with: this BC
  invokes the CredentialStore trait's init function; BC-2.03.001 specifies the trait contract)
- BC-2.03.006 — Credential Resolution at Sensor Query Time (complements: this BC covers init;
  BC-2.03.006 covers the on-demand resolution at query time — the two together implement AD-017)
- BC-2.03.007 — Secret Redaction in Logs, Errors, and MCP Responses (enforced by: the
  non-leak invariant in this BC is the upstream version of BC-2.03.007's redaction guarantee)
- BC-2.03.011 — Keyring Startup Probe for Permission Pre-Authorization (composes with: the
  startup probe called during this init step is specified in BC-2.03.011)

## Architecture Anchors

- `specs/architecture/decisions/ADR-022-production-runtime-wiring.md` §B step 5 (boot step spec)
- `specs/architecture/decisions/ADR-022-production-runtime-wiring.md` §A exit-code contract
- Architecture decision AD-017 (AI-opaque credential management — reference-based model)
- `specs/architecture/security-architecture.md` §Credentials (AD-017 implementation)
- `specs/architecture/module-decomposition.md` COMP-001 `prism-bin` (SS-22), COMP-009 `prism-credentials` (SS-03)

## Story Anchor

S-WAVE5-PREP-01 — prism-bin: Binary Chassis, CLI, and Boot Sequence

## VP Anchors

None (see Verification Properties)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| Capability Anchor Justification | CAP-004 ("Credential Management") per capabilities.md §CAP-004 — this BC specifies the startup-time initialization of the `CredentialStore` and reference validation, which is the initialization phase of the "Store, retrieve, rotate, and delete per-client per-sensor credentials" behavior that CAP-004 defines. The AI-opacity invariant ("Credential values are never included in logs, MCP responses, or error messages") is the direct behavioral specification of this BC's critical non-leak invariant. |
| L2 Invariants | AD-017 AI-opaque credential model (reference-based, no inline values). No standalone DI covers the credential non-leak boot invariant; the property is specified in AD-017 and enforced by BC-2.03.006. |
| ADR Source | ADR-022 §B step 5, §A exit-code table; AD-017 (AI-opaque credential management) |
| Priority | P0 |
| POL-12 Note | The production code path satisfying this BC MUST contain no `todo!()`, `unimplemented!()`, or `panic!("stub...")` before S-WAVE5-PREP-01 transitions to `merged`. |

## Open Questions

**OQ-1 (Test approach for credential non-leak invariant):** The three-tier test approach
(API type inspection, runtime field count assertion, process memory scan) is described in
the Test Strategy section. The implementer of S-WAVE5-PREP-01 should confirm which approach
is feasible given the actual `CredentialStore` struct layout in `prism-credentials`. If the
struct uses a `SecretString` wrapper (from the `secrecy` crate or similar), Approach A needs
to verify that the wrapper field is `None`/unset at init time, not that the type is absent.

Flag to human if the `prism-credentials` crate uses a zero-on-drop memory strategy that
renders the process-scan approach (Approach C) unreliable.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | bundle-B-phase-B-1b-ss22-bcs-2026-05-08 | 2026-05-08 | product-owner | Initial authorship — Bundle B Phase B-1b SS-22 boot-sequence BCs |
| 1.0 | redirect-option-d-2026-05-08 | 2026-05-08 | product-owner | Relocated from BC-2.22.003 (SS-22) to BC-2.03.013 (SS-03 Credential Management) per Option (d) decomposition. Capability anchor updated CAP-034 → CAP-004. EC/TV IDs renumbered to EC-03-013-NNN / TV-03-013-NNN. |
