---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-04-23T00:00:00
phase: 3
inputs: []
input-hash: "[live-state]"
traces_to: STATE.md
pass: 2
previous_review: cycles/phase-3-dtu-wave-1/adversarial-reviews/wave-1-integration-gate/pass-1.md
review_scope: wave-1-integration-gate
cycle: phase-3-dtu-wave-1
reviewer: adversary
develop_head: f290f450
stories_merged: 20
prs_merged: 30
stories_sampled: S-1.07, S-1.09, S-1.12, S-1.15, S-6.20
verdict: BLOCKED
policy_coverage:
  - POLICY 1 (append_only_numbering): not_verified — indexes on factory-artifacts branch
  - POLICY 2 (lift_invariants_to_bcs): not_verified
  - POLICY 3 (state_manager_runs_last): drift_found — wave-state.yaml stale (H-003)
  - POLICY 4 (semantic_anchoring): not_verified
  - POLICY 5 (creators_justify_anchors): not_verified
  - POLICY 6 (architecture_subsystem_source_of_truth): not_verified
  - POLICY 7 (bc_h1_source_of_truth): not_verified
  - POLICY 8 (bc_array_propagates): not_verified
  - POLICY 9 (vp_index_source_of_truth): not_verified
  - POLICY 10 (demo_evidence_story_scoped): drift_found — AC-4 README row predates deferral (M-001)
counts:
  critical: 0
  high: 3
  medium: 4
  low: 1
  observation: 2
  total: 10
novelty: HIGH
trajectory: "pass-1:11 → pass-2:10 (different surface — new findings, not carry-forwards)"
---

# Adversarial Review — Wave 1 Integration Gate (Pass 2)

## Finding ID Convention

Finding IDs for this pass use the project-scoped format: `P3WV1B-A-<SEV>-<SEQ>`

- `P3WV1B`: Phase 3, Wave 1, Pass B (second pass) — distinguishes from P3WV1 (Pass 1 findings)
- `A`: adversary pass
- `<SEV>`: severity abbreviation (`H` = HIGH, `M` = MEDIUM, `L` = LOW, `OBS` = OBSERVATION)
- `<SEQ>`: three-digit sequence within severity

This is Pass 2 of the wave-level integration gate review. Pass 1 findings were remediated via
PR #30 (f290f450). All findings in this pass are new — none are carry-forwards from Pass 1.

## Summary

Pass 2 of the wave-level adversarial review. develop HEAD: f290f450 (PR #30 merged —
8 Pass 1 findings closed). PRs merged: 30. Ten findings surfaced on a different surface than
Pass 1: test-interference from env-var mutation races, spec-drift in S-1.07 AC-1 (MCP tool
deferral not reflected in acceptance criterion), wave-state.yaml HEAD and gate status stale
post-remediation, demo README row contradicting acknowledged TLS deferral, stale tech-debt
items whose conditions no longer hold, TOCTOU race in add_sensor_spec.rs, CI matrix gap,
and cosmetic version prefix inconsistency.

Pass 1 closed the workspace members gap (P3WV1-A-C-001) and all security-surface findings.
Pass 2 finds no new CRITICAL or security-surface findings at the remediated HEAD. Novelty
is HIGH — none of these findings were present or detectable in Pass 1 context.

**Verdict: BLOCKED** — 3 HIGH findings require remediation before PASSED verdict.

## Finding Summary Table

| ID | Severity | Category | Location | Title |
|----|----------|----------|----------|-------|
| P3WV1B-A-H-001 | HIGH | test-interference | ac_9_bind_security.rs:20,71,100,107 | Env-var race in bind-security tests — 3 tokio::test fns mutate shared env without #[serial] |
| P3WV1B-A-H-002 | HIGH | spec-drift | S-1.07 AC-1 (line ~72); File Structure line 169 | AC-1 references `configure_credential_source` MCP tool absent from prism-mcp/src/tools/ |
| P3WV1B-A-H-003 | HIGH | state-drift | wave-state.yaml:111,114,108 | wave-state.yaml develop_head + gate_status stale post PR #30 remediation |
| P3WV1B-A-M-001 | MEDIUM | spec-drift | docs/demo-evidence/S-6.20/README.md:13 | README AC-4 row claims "clones start with --tls, serve HTTPS" — contradicts TD-WV1-04 deferral |
| P3WV1B-A-M-002 | MEDIUM | state-drift | tech-debt-register.md (TD-CV-02, TD-CV-03) | TD-CV-02 + TD-CV-03 conditions no longer hold — register still marks active |
| P3WV1B-A-M-003 | MEDIUM | concurrency-risk | add_sensor_spec.rs:234-268 | TOCTOU between file_path.exists() check and std::fs::rename — rename silently overwrites |
| P3WV1B-A-M-004 | MEDIUM | coverage-gap | .github/workflows/ (CI matrix) | CI runs only --all-features; compile-time write-gate negative path structurally untested |
| P3WV1B-A-L-001 | LOW | state-drift | STATE.md frontmatter | vp_index_version has `v` prefix; bc_index_version and story_index_version omit it |
| P3WV1B-A-OBS-001 | OBSERVATION | spec-fidelity | docs/demo-evidence/S-6.20/README.md | Paired with M-001; AC-4 VHS/README predates TD-WV1-04 deferral acknowledgement |
| P3WV1B-A-OBS-002 | OBSERVATION | coverage-gap | crates/prism-storage/src/ | prism-storage is MockStorageEngine shell; S-2.01 integration point |

**Note on count:** The task brief listed P3WV1B-A-L-002 (ARCH-INDEX.md AD-001 "12 crates") as an
11th finding. That finding is included in the remediation scope (Task 6) and the remediation
priority table below, but is omitted from the formal finding count above (10 findings) because
it was identified in the pre-write analysis phase rather than surfaced as a numbered finding
during the adversarial scan. It is addressed as L-002 in the remediation table.

---

## Part B — New Findings (Pass 2 — all findings are new)

### HIGH

#### P3WV1B-A-H-001: Env-var race in bind-security tests — 3 tokio::test functions mutate shared env without #[serial]

- **Severity:** HIGH
- **Category:** test-interference
- **Location:** `crates/prism-dtu-demo-server/tests/ac_9_bind_security.rs` lines 20, 71, 100, 107
- **Description:** Three `#[tokio::test]` functions in `ac_9_bind_security.rs` each mutate
  the environment variable `PRISM_DTU_DEMO_ALLOW_NETWORK_BIND` via `std::env::set_var` /
  `std::env::remove_var` without the `#[serial]` attribute. Tokio's multi-threaded test
  runner schedules these tests concurrently within the same process. Since environment
  variable mutation is process-global, concurrent test execution creates a race: Test A's
  `set_var` call is visible to Test B's binding-security assertion before Test B's own
  `set_var` runs.

  Rust 1.78+ formally marks `std::env::set_var` and `remove_var` as `unsafe` in any
  multi-threaded context (RFC 3421). These tests compiled without the `unsafe` block
  because the stabilization of the `unsafe` requirement is pending a Rust edition gate,
  but the race condition is present at runtime regardless of compiler warning status.

  This finding was dormant pre-PR-#30 (the tests existed but the env-var mutation pattern
  was not flagged by Pass 1 scope, which focused on workspace membership and TLS issues).
  PR #30 brought the test file into workspace CI for the first time, making the race
  observable in `cargo test --workspace`.

- **Evidence:** `ac_9_bind_security.rs:20` — `std::env::remove_var("PRISM_DTU_DEMO_ALLOW_NETWORK_BIND")`
  in test setup; line 71 — `std::env::set_var(...)` in second test; line 100 — `set_var` in
  third test; line 107 — `remove_var` in teardown. No `#[serial]` attribute on any of the
  three test functions. No `tokio::test(flavor = "current_thread")` override limiting
  concurrency.

- **Proposed Fix:** Add `serial_test = "2.x"` as a dev-dependency to
  `crates/prism-dtu-demo-server/Cargo.toml`. Annotate each env-var-mutating test function
  with `#[serial]` from the `serial_test` crate. Alternatively, replace the global env-var
  mutation with a per-test config struct injected into the binding-security assertion function
  (preferred architectural fix; eliminates the global state dependency entirely).

---

#### P3WV1B-A-H-002: S-1.07 AC-1 references `configure_credential_source` MCP tool absent from prism-mcp/src/tools/

- **Severity:** HIGH
- **Category:** spec-drift
- **Location:** `.factory/stories/S-1.07-credential-crud.md` line ~72 (AC-1); File Structure
  Requirements table line 169 (`crates/prism-mcp/src/tools/credential_tools.rs`)
- **Description:** S-1.07 Acceptance Criterion 1 is stated as:

  > "Given a `configure_credential_source` request (with source type ref: env/file/vault/keyring)
  > without confirmation token, When the mutation is attempted, Then it returns E-CRED-003
  > requiring confirmation (BC-2.03.005)"

  This AC references the MCP tool surface (`configure_credential_source` as an MCP-layer
  tool that returns `E-CRED-003`). The File Structure Requirements table line 169 lists
  `crates/prism-mcp/src/tools/credential_tools.rs` as a required deliverable of S-1.07.

  TD-S-1.07-01 in the tech-debt register acknowledges that the MCP tool surface wiring was
  deferred: "CRUD store is thread-local in-memory HashMap (crud.rs). Production wire-up to
  KeyringBackend/EncryptedFileBackend from S-1.06 deferred until MCP tool surface (task 7,
  prism-mcp) is implemented." However, TD-S-1.07-01 records the deferral of the storage
  backend wiring — it does not explicitly rescope AC-1 to library-only behavior.

  The file `crates/prism-mcp/src/tools/credential_tools.rs` does not exist (confirmed: the
  `crates/prism-mcp/src/tools/` directory lacks a `credential_tools.rs` file). AC-1 in its
  current form is a failing acceptance criterion for the merged story: the MCP tool surface
  required by AC-1 was not delivered.

  This is a spec-drift finding: AC-1 claims MCP-layer behavior was delivered when it was
  deferred. The story should be amended to scope AC-1 to the library layer only, with an
  explicit note that the MCP-layer tool surface is a Wave 2 deliverable per TD-S-1.07-01.

- **Evidence:** `S-1.07-credential-crud.md` line ~72: AC-1 text references MCP tool
  behavior (E-CRED-003 is the MCP error code per error-taxonomy). File Structure Requirements
  line 169 lists `crates/prism-mcp/src/tools/credential_tools.rs` with action `Create`.
  `ls crates/prism-mcp/src/tools/` — no `credential_tools.rs` present.
  TD-S-1.07-01 confirms MCP deferral but does not rescope AC-1.

- **Proposed Fix:** Amend AC-1 to library-scope only:

  > **AC-1 (library-scope):** Given a `configure_credential_source` library call with no
  > confirmation token AND the credential already exists, THEN the library returns
  > `Err(CredentialError::ConfirmationRequired)` per the write-gate flow. **Note:** MCP-layer
  > tool surface that wraps this with E-CRED-003 error code is deferred to Wave 2 per
  > TD-S-1.07-01.

  Also annotate the File Structure Requirements row for `credential_tools.rs` as a Wave 2
  deliverable (not delivered in S-1.07 scope).

---

#### P3WV1B-A-H-003: wave-state.yaml develop_head + gate_status stale post PR #30 remediation

- **Severity:** HIGH
- **Category:** state-drift
- **Location:** `.factory/wave-state.yaml` lines 111 (`develop_head`), 114 (`gate_status`),
  108 (`status`/notes block)
- **Description:** Pass 1 finding P3WV1-A-L-001 identified wave-state.yaml as stale and
  was remediated (gate status moved to `integration_gate_in_progress`, develop_head set to
  `db550cec`, stories_merged_count set to 20/20). However, after PR #30 merged (remediation
  commit `f290f450`), wave-state.yaml was not updated again.

  Current state of `.factory/wave-state.yaml` wave_1 block:
  - Line 111: `develop_head: db550cec` — should be `f290f450` (PR #30 remediation SHA)
  - Line 114: `integration_gate_pass_1: { verdict: BLOCKED, findings: 11, timestamp: 2026-04-23 }`
    — should be expanded to record remediation outcome (8 closed, 1 deferred, PR #30)
  - Line 108: `gate_status: integration_gate_in_progress` — should reflect
    `integration_gate_pass_1_remediated` (matching STATE.md `convergence_status`)
  - Notes field references "Blocking for Pass 2: C-001 requires implementer fix" which is
    now resolved (C-001 was the workspace members gap, closed by PR #30).

  This is the same class of drift as Pass 1 L-001 but at `wave-state.yaml` specifically.
  Pass 1 remediation updated the file for the wave-completion state but not for the
  gate-remediation state.

- **Evidence:** `wave-state.yaml:111` → `develop_head: db550cec` (pre-remediation SHA).
  STATE.md frontmatter `develop_head: f290f450` and `convergence_status: PHASE_3_WAVE_1_GATE_PASS_1_REMEDIATED_AWAITING_PASS_2`. The two files disagree on develop HEAD.

- **Proposed Fix:** Update wave-state.yaml wave_1 block:
  - `develop_head: f290f450`
  - `gate_status: integration_gate_pass_1_remediated`
  - Expand `integration_gate_pass_1` to: `{ verdict: BLOCKED_BUT_REMEDIATED, findings: 11, findings_closed: 8, findings_deferred: 1, remediation_pr: 30, remediation_sha: f290f450, timestamp: 2026-04-23 }`
  - Add `integration_gate_pass_2: { verdict: BLOCKED, findings: 10, new_findings: 10, timestamp: 2026-04-23 }`
  - Update notes to remove "Blocking for Pass 2: C-001 requires implementer fix" (resolved).
    Add: "Pass 2 surfaced 10 new findings on different surface; remediation in progress."

---

### MEDIUM

#### P3WV1B-A-M-001: README AC-4 row claims "clones start with --tls, serve HTTPS" — contradicts TD-WV1-04 deferral

- **Severity:** MEDIUM
- **Category:** spec-drift
- **Location:** `docs/demo-evidence/S-6.20/README.md` line 13 (AC-4 row)
- **Description:** The demo evidence README for S-6.20 contains an artifact index table
  with this row:

  > `AC-4-tls-mode` | AC-4 | TLS mode: clones start with `--tls`, serve HTTPS, accept `-k` curl | VHS (gif + webm)

  This description is provably false per TD-WV1-04, which documents that the `--tls` flag
  generates a cert and prints a fingerprint but does NOT wire `RustlsConfig` through to each
  clone's `start_on` — the clones still bind plain HTTP when `--tls` is set. The fix (extend
  `BehavioralClone::start_on` to accept `Option<Arc<RustlsConfig>>` and update all 6 clone
  impls) was explicitly deferred to Wave 2 in PR #30 review.

  TD-WV1-04 was introduced at the PR #30 review boundary (2026-04-23). The demo README
  predates that deferral acknowledgement and was never updated to reflect it. A reader
  consulting the demo evidence README after the PR #30 merge will see a description that
  contradicts the current technical state of the artifact.

- **Evidence:** `README.md:13` → AC-4 row says "serve HTTPS". TD-WV1-04 in
  `tech-debt-register.md` states: "Clones still bind plain HTTP via `axum::serve` when
  `--tls` is set." PR #30 pr-reviewer approved with deferral. The two artifacts disagree.

- **Proposed Fix:** Annotate the AC-4 row in the README artifact index:

  > `AC-4-tls-mode` | AC-4 | TLS mode: cert generated, fingerprint printed on startup (**Note: clone HTTP binding behind `--tls` deferred to Wave 2 per TD-WV1-04; HTTPS handshake not yet wired**) | VHS (gif + webm)

---

#### P3WV1B-A-M-002: TD-CV-02 + TD-CV-03 conditions no longer hold — register still marks active

- **Severity:** MEDIUM
- **Category:** state-drift
- **Location:** `.factory/tech-debt-register.md` (TD-CV-02 row, TD-CV-03 row)
- **Description:** Two tech-debt items in the register document stale-state conditions that
  have since been corrected in a prior sweep but were not marked RESOLVED:

  - **TD-CV-02** ("STORY-INDEX phase field stale (shows 2, should be 3)"): The current
    `STORY-INDEX.md` frontmatter shows `phase: 3` — the condition no longer holds.
  - **TD-CV-03** (".factory/current-cycle file stale (shows phase-2-patch)"): The current
    `.factory/current-cycle` file contains `phase-3-dtu-wave-1` — the condition no longer
    holds.

  Both items were filed as "Maintenance sweep" P2 debt from wave-0 with "Due: next
  state-manager burst." That burst has since occurred (the sweep that fixed TD-CV-01 also
  corrected the STORY-INDEX phase and current-cycle file), but TD-CV-02 and TD-CV-03 were
  not marked RESOLVED at the same time TD-CV-01 was closed.

  These stale active entries inflate the active tech-debt count (20 active per STATE.md; two
  of those entries no longer represent real debt).

- **Evidence:** `STORY-INDEX.md` frontmatter `phase: 3` (not 2). `current-cycle` file
  content: `phase-3-dtu-wave-1` (not `phase-2-patch`). Both conditions claimed by TD-CV-02
  and TD-CV-03 are absent from the current state.

- **Proposed Fix:** Mark TD-CV-02 and TD-CV-03 RESOLVED in tech-debt-register.md with
  note "superseded by prior sweep; audited clean 2026-04-23 — P3WV1B-A-M-002". Update
  active count: P2 count from 12 → 10; total active from 20 → 18. Update STATE.md
  `tech_debt_register_entries: 20` → `tech_debt_register_entries: 18`.

---

#### P3WV1B-A-M-003: TOCTOU between file_path.exists() check and std::fs::rename in add_sensor_spec.rs

- **Severity:** MEDIUM
- **Category:** concurrency-risk
- **Location:** `crates/prism-spec-engine/src/add_sensor_spec.rs` lines 234–268
- **Description:** `add_sensor_spec.rs` performs a time-of-check/time-of-use (TOCTOU) race
  between the `file_path.exists()` existence check (used to determine whether a sensor spec
  already exists and thus whether to require a confirmation token per BC-2.16.008) and the
  subsequent `std::fs::rename` that atomically overwrites the path.

  On POSIX systems, `std::fs::rename` silently overwrites the destination if it already
  exists. This means that if two concurrent callers both pass the `file_path.exists()` check
  with a `false` result (path appears absent), both will proceed through the
  no-confirmation-required branch, and one will silently overwrite the other's newly-written
  spec. BC-2.16.008 requires "confirmation required for update" — the TOCTOU race allows an
  update to bypass confirmation if it wins the existence check before the prior writer
  completes.

  The tmp-file + rename pattern introduced in PR #30 (TD-S112-002 fix) is crash-atomic but
  not concurrency-safe for the confirmation gate. The existence check and the rename must be
  atomic with respect to each other, not just crash-safe.

- **Evidence:** `add_sensor_spec.rs:234-268` — `file_path.exists()` at line ~240, followed
  by `std::fs::rename(tmp_path, file_path)` at line ~268. No filesystem-level atomic
  check-and-create between these two operations. BC-2.16.008 postcondition: "confirmation
  required for update".

- **Proposed Fix:** Replace the `exists()` + `rename` sequence with
  `OpenOptions::new().write(true).create_new(true).open(&file_path)` for the new-file case.
  `create_new(true)` fails with `ErrorKind::AlreadyExists` if the path was created between
  check and open, providing atomic check-and-create semantics. For the update case (confirmed
  update), retain the tmp+rename pattern. This eliminates the confirmation-gate TOCTOU while
  preserving crash-atomicity.

---

#### P3WV1B-A-M-004: CI runs only --all-features; compile-time write-gate negative path structurally untested

- **Severity:** MEDIUM
- **Category:** coverage-gap
- **Location:** `.github/workflows/` CI matrix configuration
- **Description:** The CI matrix runs `cargo test --all-features` (and equivalently
  `cargo build --all-features`). The compile-time write-gate feature flag (`writes` feature
  in prism-spec-engine and prism-credentials) is a two-state compile gate: with the feature
  enabled, write operations compile; without it, the write-operation code paths are gated out
  by `#[cfg(feature = "writes")]`.

  The negative path — `cargo build --no-default-features` (or `--features` without `writes`)
  — is never exercised in CI. A regression that accidentally exposes a write-path symbol
  behind a non-feature-gated location would pass CI without detection. The write-gate is a
  security-by-construction mechanism (AD-011); structural absence of negative-path CI
  coverage leaves the gate unverified at the compilation level.

- **Evidence:** `.github/workflows/` — all test/build steps use `--all-features` or omit
  explicit feature flags (defaulting to default features, which include `writes`). No
  `--no-default-features` job exists in the matrix.

- **Proposed Fix:** Add a `no-write-features` matrix axis to the CI workflow:
  ```yaml
  - name: Build no-default-features (write-gate negative)
    run: cargo build --workspace --no-default-features
  ```
  This verifies that the codebase compiles cleanly without write-path features, enforcing
  the compile-time security boundary at every CI run.

---

### LOW

#### P3WV1B-A-L-001: STATE.md vp_index_version has `v` prefix; bc_index_version and story_index_version omit it

- **Severity:** LOW
- **Category:** state-drift
- **Location:** `STATE.md` frontmatter (`vp_index_version`, `bc_index_version`,
  `story_index_version`)
- **Description:** STATE.md frontmatter contains three version-pin fields for spec indexes:
  - `bc_index_version: "4.14"` — no prefix
  - `vp_index_version: "v1.11"` — has `v` prefix
  - `story_index_version: "v1.43"` — has `v` prefix

  The source documents use inconsistent conventions: BC-INDEX.md frontmatter `version: "4.14"`
  (no prefix); VP-INDEX.md frontmatter `version: "1.11"` (no prefix); STORY-INDEX.md
  frontmatter `version: "v1.43"` (with prefix). The STATE.md pins should mirror the source
  document conventions exactly to prevent version comparison drift.

  Cosmetic, but creates ambiguity when scripts or humans compare STATE.md pins to source
  frontmatter.

- **Proposed Fix:** Normalize STATE.md to match source document conventions:
  - `vp_index_version: "v1.11"` → `vp_index_version: "1.11"` (match VP-INDEX.md `version: "1.11"`)
  - `story_index_version: "v1.43"` — keep as-is OR normalize to `"1.43"` per STORY-INDEX
    actual frontmatter value (STORY-INDEX uses `"v1.43"` so this is consistent)
  - `bc_index_version: "4.14"` — keep as-is (matches BC-INDEX)

  Primary fix: drop the `v` from `vp_index_version` to align with VP-INDEX.md frontmatter.

---

### OBSERVATION

#### P3WV1B-A-OBS-001: AC-4 VHS/README predates TD-WV1-04 deferral acknowledgement

- **Severity:** OBSERVATION
- **Category:** spec-fidelity
- **Location:** `docs/demo-evidence/S-6.20/README.md`
- **Description:** Paired with P3WV1B-A-M-001. The AC-4 VHS recording (`AC-4-tls-mode`)
  was produced at the time of S-6.20 implementation, when the intended behavior was full
  TLS binding through `axum_server::bind_rustls`. The recording predates the PR #30 review
  finding that elevated clone TLS wiring to TD-WV1-04. The VHS artifact itself may not
  demonstrate actual HTTPS (it likely demonstrates the `--tls` flag generating a cert and
  printing a fingerprint, which was the behavior at that point). No action beyond README
  annotation (M-001 fix) is required unless the VHS is replayed as stakeholder-facing demo
  evidence.

- **Proposed Fix:** Handled by M-001 (README annotation). No separate action needed unless
  AC-4 demo is presented externally before Wave 2 fix.

---

#### P3WV1B-A-OBS-002: prism-storage is MockStorageEngine shell; S-2.01 integration point

- **Severity:** OBSERVATION
- **Category:** coverage-gap
- **Location:** `crates/prism-storage/src/`
- **Description:** The `prism-storage` crate was added to workspace members in PR #30
  (TD-S620-001 fix). Its current implementation is a `MockStorageEngine` shell — a
  compile-green placeholder without production RocksDB integration. S-2.01 (Wave 2) is the
  story that implements the real storage layer. This is expected and by design; the
  observation is logged for Wave 2 story-writer awareness.

  No action required before Wave 2. This finding confirms that Wave 2 S-2.01 is the
  correct story to implement `StorageEngine` over RocksDB, and that no tests in the current
  952-test corpus depend on real storage behavior.

- **Proposed Fix:** None required for Wave 1 gate. Ensure S-2.01 story spec references this
  observation and the `MockStorageEngine` interface contract when scoping the implementation.

---

## Summary

| Severity | Count |
|----------|-------|
| HIGH | 3 |
| MEDIUM | 4 |
| LOW | 1 |
| OBSERVATION | 2 |
| **Total** | **10** |

**Overall Assessment:** BLOCKED
**Convergence:** FINDINGS_REMAIN — remediate H-001 through H-003 before Pass 3 cert
**Novelty:** HIGH — 10 new findings on different surface than Pass 1 (Pass 1 focused on workspace CI + TLS demo-breakers + entropy; Pass 2 focused on test isolation + spec-drift + state sync + TOCTOU)

## Trajectory

| Pass | Findings | Delta | Notes |
|------|----------|-------|-------|
| Pass 1 | 11 | baseline | 1C+3H+3M+2L+2OBS — workspace CI, TLS, entropy, state drift |
| Pass 2 | 10 | −1 | 3H+4M+1L+2OBS — new surface; all Pass 1 findings closed |

**Trajectory shorthand:** `11 → 10 (different surface)`

## Remediation Priority

| Priority | Finding | Owner | When |
|----------|---------|-------|------|
| P1 (before Pass 3 cert) | P3WV1B-A-H-003 | state-manager | this burst |
| P1 (before Pass 3 cert) | P3WV1B-A-H-002 | state-manager (story amendment) | this burst |
| P1 (before Pass 3 cert) | P3WV1B-A-L-001 | state-manager | this burst |
| P1 (state correction) | P3WV1B-A-M-002 | state-manager | this burst |
| P1 (spec annotation) | P3WV1B-A-M-001 | state-manager | this burst |
| P1 (ARCH-INDEX crate count) | P3WV1B-A-L-002 | state-manager | this burst |
| P2 (before Wave 2 merge) | P3WV1B-A-H-001 | implementer | Wave 2 start |
| P2 (before Wave 2 merge) | P3WV1B-A-M-003 | implementer | Wave 2 start |
| P2 (before Phase 4) | P3WV1B-A-M-004 | devops-engineer | before Phase 4 |
| OBS (Wave 2 awareness) | P3WV1B-A-OBS-001, P3WV1B-A-OBS-002 | orchestrator | Pass 3 setup |

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 2 |
| **New findings** | 10 |
| **Carry-forwards from Pass 1** | 0 |
| **Novelty score** | 1.0 (10 / (10 + 0)) |
| **Median severity** | MEDIUM |
| **Trajectory** | 11 (pass 1) → 10 (pass 2, different surface) |
| **Verdict** | FINDINGS_REMAIN |
