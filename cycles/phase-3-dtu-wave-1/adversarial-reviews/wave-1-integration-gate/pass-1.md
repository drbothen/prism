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
pass: 1
previous_review: null
review_scope: wave-1-integration-gate
cycle: phase-3-dtu-wave-1
reviewer: adversary
develop_head: db550cec
stories_merged: 20
prs_merged: 25
stories_sampled: S-1.01, S-1.07, S-1.08, S-1.09, S-1.12, S-1.15, S-6.07, S-6.10, S-6.20
verdict: BLOCKED
policy_coverage:
  - POLICY 1 (append_only_numbering): not_verified — indexes on factory-artifacts branch
  - POLICY 2 (lift_invariants_to_bcs): not_verified
  - POLICY 3 (state_manager_runs_last): no_drift_found
  - POLICY 4 (semantic_anchoring): not_verified
  - POLICY 5 (creators_justify_anchors): not_verified
  - POLICY 6 (architecture_subsystem_source_of_truth): not_verified
  - POLICY 7 (bc_h1_source_of_truth): not_verified
  - POLICY 8 (bc_array_propagates): not_verified
  - POLICY 9 (vp_index_source_of_truth): not_verified
  - POLICY 10 (demo_evidence_story_scoped): partially_verified (sampled S-6.20 — compliant)
counts:
  critical: 1
  high: 3
  medium: 3
  low: 2
  observation: 2
  total: 11
---

# Adversarial Review — Wave 1 Integration Gate (Pass 1)

## Finding ID Convention

Finding IDs for this wave-level gate use the project-scoped format: `P3WV1-A-<SEV>-<SEQ>`

- `P3WV1`: Phase 3, Wave 1 — cycle identifier
- `A`: adversary pass
- `<SEV>`: severity abbreviation (`C` = CRITICAL, `H` = HIGH, `M` = MEDIUM, `L` = LOW, `OBS` = OBSERVATION)
- `<SEQ>`: three-digit sequence within severity

This is the first pass of the wave-level integration gate review (distinct from the
per-story adversarial passes conducted during implementation).

## Summary

Pass 1 of the wave-level adversarial review. Stories merged: 20 of 20; PRs merged: 25;
develop HEAD: db550cec. Eleven findings surfaced spanning workspace CI correctness,
TLS demo-breakers, frontmatter lifecycle drift, state-drift arithmetic, and token-entropy
carryovers.

**Verdict: BLOCKED** — 7 blocking items require remediation before Pass 2.

## Finding Summary Table

| ID | Severity | Category | Location | Title |
|----|----------|----------|----------|-------|
| P3WV1-A-C-001 | CRITICAL | coverage-gap | Cargo.toml:3-14 | 6 crates excluded from workspace — ~700 tests silently skipped |
| P3WV1-A-H-001 | HIGH | security-surface | tls.rs:30-31, main.rs:221-240, ac_4_tls.rs | Expired cert dates + TLS not wired into axum + hollow test |
| P3WV1-A-H-002 | HIGH | spec-fidelity | tls.rs:45-69 | Fingerprint is hex(base64(PEM)) not sha256(DER) |
| P3WV1-A-H-003 | HIGH | spec-fidelity | STATE.md:35 | TD register count 24 in STATE.md; actual is 28 (+4 drift) |
| P3WV1-A-M-001 | MEDIUM | spec-fidelity | stories/*.md | 20 merged stories still status: draft in frontmatter |
| P3WV1-A-M-002 | MEDIUM | security-surface | crud.rs:308-319, add_sensor_spec.rs:186-198 | Weak token entropy (pid+nanos / SystemTime) |
| P3WV1-A-M-003 | MEDIUM | code-quality | add_sensor_spec.rs:252 | Non-atomic write (fs::write not tmp+rename) |
| P3WV1-A-L-001 | LOW | spec-fidelity | wave-state.yaml:86-115 | wave-state.yaml stale: 14/20, develop_head outdated |
| P3WV1-A-L-002 | LOW | missing-edge-cases | prism-dtu-common/src/clone.rs:43-66 | BehavioralClone default start_on/stop panic |
| P3WV1-A-OBS-001 | OBSERVATION | verification-gaps | — | factory-artifacts indexes inaccessible; Policies 1,2,4-9 unverified |
| P3WV1-A-OBS-002 | OBSERVATION | coverage-gap | — | S-1.14 + S-1.09 not in workspace CI; total volume ~2x reported 428 |

## Key Themes

1. **Workspace test blindness** — 6 crates hosting stories from Wave 1 are not in
   Cargo.toml `[workspace.members]`. The "428 workspace tests green" certification
   does NOT cover S-1.05/S-1.08/S-1.09/S-1.10/S-1.12/S-1.14/S-1.15. The reported
   428 is a subset of total test volume.

2. **Demo-breaker TD still P1** — TD-S620-002 (expired certs), TD-S620-003 (TLS not
   wired into axum), TD-S620-006 (wrong fingerprint algorithm) all ship in merged code
   at db550cec.

3. **Story frontmatter status drift** — 20 merged stories still show `status: draft`.
   Remediable by state-manager in-place.

4. **Scope limitation** — factory-artifacts branch spec indexes inaccessible to
   adversary from develop worktree; Policies 1, 2, 4–9 not fully verified in Pass 1.

---

## Part B — New Findings (Pass 1 — all findings are new)

### CRITICAL

#### P3WV1-A-C-001: 6 crates excluded from workspace — ~700 tests silently skipped

- **Severity:** CRITICAL
- **Category:** coverage-gap
- **Location:** `Cargo.toml:3-14` (`[workspace.members]` array)
- **Description:** Root `Cargo.toml` `[workspace.members]` is missing 6 crates:
  prism-mcp, prism-ocsf, prism-security, prism-spec-engine, prism-storage, ocsf-proto-gen.
  These crates exist in the repo tree but are not registered as workspace members.
  `cargo test --workspace` silently skips all tests in those 6 crates. The "428
  workspace tests green" certification does NOT cover the tests in these crates.
- **Evidence:** `cargo metadata --no-deps` lists workspace members; the 6 crates above
  are absent. Estimated skipped tests: ~700 (S-1.09 = 200 tests, S-1.14 = 220 tests,
  remaining 4 crates ~280 across S-1.05/S-1.08/S-1.10/S-1.12/S-1.15). Total Wave 1
  test volume is approximately 1,128 — roughly 2.6x the reported 428. TD-S620-001
  already tracked this as P2; this finding escalates it to blocking for Pass 2
  certification.
- **Proposed Fix:** Add all 6 missing paths to `[workspace.members]` in root
  `Cargo.toml`. Run `cargo test --workspace` after to verify the full suite passes and
  update the certification count in STATE.md and wave-state.yaml.

---

### HIGH

#### P3WV1-A-H-001: TLS demo-breaker bundle (expired certs + HTTP-only server + hollow test)

- **Severity:** HIGH
- **Category:** security-surface
- **Location:** `crates/prism-dtu-demo-server/src/tls.rs:30-31`; `main.rs:221-240`; `tests/ac_4_tls.rs`
- **Description:** Three related sub-issues: (A) TLS cert validity hardcoded to
  `2024-01-01..2024-12-31` — both dates already expired 2026-04-23; any TLS client
  doing validity-window checks rejects the cert immediately. (B) `--tls` flag generates
  a cert and prints the fingerprint but never passes `RustlsConfig` to axum; server
  still binds plain HTTP. (C) `ac_4_tls.rs` only asserts the PEM output is non-empty;
  no HTTPS handshake is performed — a completely invalid cert passes this test.
- **Evidence:** tls.rs:30-31 contains hardcoded `DateTime` literals for 2024;
  main.rs:221-240 generates `RustlsConfig` but passes it to no axum binding; ac_4_tls.rs
  terminal assertion is `assert!(!pem.is_empty())`. AC-4 acceptance criterion requires
  functioning TLS.
- **Proposed Fix:** (A) Replace hardcoded dates with `time::OffsetDateTime::now_utc()`
  as NotBefore and `now_utc() + Duration::days(365)` as NotAfter. (B) Wire
  `RustlsConfig` into axum: `axum_server::bind_rustls(addr, config).serve(...)`. (C)
  Extend `ac_4_tls.rs` to perform a loopback HTTPS handshake and assert HTTP 200.
  Tracked as TD-S620-002 (A), TD-S620-003 (B), TD-S620-006 (C).

#### P3WV1-A-H-002: Fingerprint algorithm incorrect — hex(base64(PEM)) not sha256(DER)

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `crates/prism-dtu-demo-server/src/tls.rs:45-69`
- **Description:** `print_cert_fingerprint()` encodes cert DER bytes to base64, then
  hex-encodes that base64 string, producing `hex(base64(DER))`. S-6.20 AC-12 mandates
  `sha256:<hex(sha256(DER))>` — SHA-256 of the raw DER bytes formatted as
  `sha256:<lowercase_hex>`. The fingerprint displayed on demo server startup is
  cryptographically meaningless for identity verification; an independent computation
  will never match.
- **Evidence:** tls.rs:45-69 shows `base64::encode(der)` piped into `hex::encode()`.
  The `sha2` crate is already available as a transitive dependency in the workspace.
- **Proposed Fix:** Replace with:
  ```rust
  use sha2::{Sha256, Digest};
  let digest = Sha256::digest(&der_bytes);
  println!("Certificate fingerprint: sha256:{}", hex::encode(digest));
  ```
  Tracked as TD-S620-006.

#### P3WV1-A-H-003: STATE.md TD register count 24; actual register has 28 active entries

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `STATE.md:35` (`tech_debt_register_entries: 24`); `.factory/tech-debt-register.md`
- **Description:** `STATE.md` line 35 reads `tech_debt_register_entries: 24`. The
  Debt Items table in `tech-debt-register.md` contains 29 rows total; TD-WV0-05 is
  marked RESOLVED inline, leaving 28 active entries. The 4-entry gap (24 → 28) is
  accounted for by items added after the last STATE.md sync: TD-WV1-01, TD-WV1-02,
  TD-WV1-03, TD-S-1.07-01, TD-S-1.07-02, TD-S112-001, TD-S112-002 (7 additions, of
  which 3 were already included in the prior 24 count, leaving a net +4 drift).
- **Evidence:** `grep "^| TD-" tech-debt-register.md | wc -l` = 29; minus 1 RESOLVED
  inline = 28. STATE.md frontmatter line 35 = 24.
- **Proposed Fix:** Update `tech_debt_register_entries: 28` in STATE.md frontmatter.

---

### MEDIUM

#### P3WV1-A-M-001: 20 merged stories still show status: draft in frontmatter

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** `.factory/stories/` — Wave 1 story files
- **Description:** All 20 Wave 1 stories are merged to develop per the Wave 1 Progress
  table in STATE.md, yet story frontmatter `status:` fields remain `draft`. Sampled
  confirmed: S-1.01 (PR #13 merged 2026-04-22), S-1.08 (PR #23 merged 2026-04-23),
  S-1.09 (PR #25 merged 2026-04-23) all show `status: draft`.
- **Evidence:** `grep "^status:" stories/S-1.01-foundational-types.md` → `status: draft`.
  S-6.06, S-6.14, S-6.15 are correctly `status: merged`. S-1.10 shows `status: delivered`.
  Registered as TD-CV-01.
- **Proposed Fix:** State-manager bulk-update `status: draft` → `status: merged` for
  all 17 stories that remain on draft: S-1.01, S-1.02, S-1.04, S-1.05, S-1.07, S-1.08,
  S-1.09, S-1.11, S-1.12, S-1.13, S-1.14, S-1.15, S-6.07, S-6.08, S-6.09, S-6.10,
  S-6.20. Then mark TD-CV-01 RESOLVED.

#### P3WV1-A-M-002: Weak token entropy in crud.rs and add_sensor_spec.rs

- **Severity:** MEDIUM
- **Category:** security-surface
- **Location:** `crates/prism-credentials/src/crud.rs:308-319`; `crates/prism-spec-engine/src/add_sensor_spec.rs:186-198`
- **Description:** Two independent weak-entropy token generators: (1) `uuid_v4_token()`
  in crud.rs constructs a UUID-format token from `std::process::id()` (pid) and
  `SystemTime::now().duration_since(UNIX_EPOCH).subsec_nanos()`. Two rapid calls with
  the same pid return the same nanosecond value under most OS schedulers. (2)
  `generate_confirmation_token()` in add_sensor_spec.rs uses `SystemTime::now()` hashed
  as the nonce. Both sources are predictable; neither uses a CSPRNG. Tracked as
  TD-S-1.07-02 and TD-S112-001.
- **Evidence:** crud.rs:308-319 contains `std::process::id()` and `subsec_nanos()` in
  token construction. add_sensor_spec.rs:186-198 hashes `SystemTime::now()` for nonce.
  VP-007..010 require confirmation token unforgeability.
- **Proposed Fix:** Replace both entropy sources with `rand::thread_rng().gen::<u128>()`
  or `uuid::Uuid::new_v4()` (uses OS CSPRNG). The `uuid` v4 crate with `v4` feature is
  already in the workspace dependency tree.

#### P3WV1-A-M-003: Non-atomic sensor spec write (std::fs::write not tmp+rename)

- **Severity:** MEDIUM
- **Category:** code-quality
- **Location:** `crates/prism-spec-engine/src/add_sensor_spec.rs:252`
- **Description:** `add_sensor_spec.rs:252` writes the sensor spec with
  `std::fs::write(path, content)`. This is not crash-atomic: a SIGKILL or power failure
  between OS truncation and write completion leaves a zero-byte or partial `.sensor.toml`
  that causes the hot-reload coordinator (S-1.12) to emit `E-RELOAD-003` (parse error)
  on every subsequent reload until manually removed. Tracked as TD-S112-002.
- **Evidence:** add_sensor_spec.rs:252: `std::fs::write(&path, &content)?;` — single
  non-atomic call. No tmp file or rename in surrounding context.
- **Proposed Fix:**
  ```rust
  let tmp = path.with_extension("toml.tmp");
  std::fs::write(&tmp, &content)?;
  std::fs::rename(&tmp, &path)?;
  ```
  `rename` is atomic on POSIX. Add Windows fallback using `MoveFileExW` with
  `MOVEFILE_REPLACE_EXISTING` if Windows is a target platform.

---

### LOW

#### P3WV1-A-L-001: wave-state.yaml stale (14/20 stories, develop_head 7031bb6)

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** `.factory/wave-state.yaml:86-115` (wave_1 block)
- **Description:** wave-state.yaml wave_1 block shows `develop_head: 7031bb6`
  (S-1.08 merge SHA), `stories_merged_count: 14`, `stories_pending: 6`. Actual state:
  develop HEAD is `db550cec`, all 20 stories merged, 0 pending. Six stories are absent
  from `stories_merged` list: S-1.15, S-1.05, S-1.12, S-1.09, S-1.07, S-6.20. The
  `notes:` field confirms the file was last updated mid-wave and not refreshed at wave
  completion.
- **Evidence:** wave-state.yaml line 105: `develop_head: 7031bb6`; line 107:
  `stories_merged_count: 14`; line 108: `stories_pending: 6`.
- **Proposed Fix:** Update wave_1 block: set `develop_head: db550cec`,
  `stories_merged_count: 20`, `stories_pending: 0`, `gate_status: integration_gate_in_progress`.
  Add the 6 missing stories to `stories_merged` list.

#### P3WV1-A-L-002: BehavioralClone default start_on/stop panic with unimplemented!()

- **Severity:** LOW
- **Category:** missing-edge-cases
- **Location:** `crates/prism-dtu-common/src/clone.rs:43-66`
- **Description:** The `BehavioralClone` trait provides default implementations for
  `start_on()` and `stop()` that call `unimplemented!()` (or `panic!()`). All 6 Wave 1
  clones override these methods, so Wave 1 is functionally safe. However, any Wave 2
  story author who adds a new DTU clone and omits `start_on()/stop()` will get a
  runtime panic rather than a compile-time error or graceful degradation. Risk is low
  for Wave 1; medium for Wave 2 (S-6.11, S-6.12, S-6.13).
- **Evidence:** clone.rs:43-66 default method bodies contain `unimplemented!()`. All
  6 merged clones (crowdstrike, claroty, cyberint, armis, threatintel, nvd) provide
  concrete overrides — confirmed by grep.
- **Proposed Fix:** Either (a) remove default impls to make methods required
  (compile-time enforcement, preferred) or (b) replace `unimplemented!()` with a
  no-op plus `tracing::warn!("start_on called on BehavioralClone with default no-op impl")`.

---

### OBSERVATION

#### P3WV1-A-OBS-001: factory-artifacts indexes inaccessible; Policies 1,2,4-9 not verified

- **Severity:** OBSERVATION
- **Category:** verification-gaps
- **Location:** `.factory/` worktree (factory-artifacts branch)
- **Description:** The factory-artifacts branch is mounted as a git worktree at
  `.factory/`. BC-INDEX, STORY-INDEX, VP-INDEX, and other versioned spec artifacts are
  on this branch and were not accessible to the adversary in the Pass 1 develop context.
  Policies 1 (append_only_numbering), 2 (lift_invariants_to_bcs), 4 (semantic_anchoring),
  5 (creators_justify_anchors), 6 (architecture_subsystem_source_of_truth),
  7 (bc_h1_source_of_truth), 8 (bc_array_propagates), and 9 (vp_index_source_of_truth)
  could not be verified.
- **Evidence:** Policy 3 (state_manager_runs_last) and Policy 10 (demo_evidence_story_scoped,
  sampled S-6.20) verified with no drift found.
- **Proposed Fix:** Adversary Pass 2 should be scoped from the factory-artifacts
  worktree context to enable full policy verification.

#### P3WV1-A-OBS-002: S-1.14 (220 tests) + S-1.09 (200 tests) not in workspace CI

- **Severity:** OBSERVATION
- **Category:** coverage-gap
- **Location:** Root `Cargo.toml` `[workspace.members]`; CI certification in STATE.md
- **Description:** S-1.14 (prism-storage, 220 tests) and S-1.09 (prism-security, 200
  tests) are among the 6 crates excluded from workspace members (P3WV1-A-C-001). These
  two stories alone account for 420 additional tests. Total Wave 1 test volume is
  approximately 428 (workspace) + 420 (two largest excluded) + ~280 (remaining 4
  excluded) ≈ 1,128 tests — roughly 2.6x the reported 428.
- **Evidence:** S-1.14 merge notes: "220/220 tests pass"; S-1.09 merge notes:
  "200/200 tests pass". Neither crate is in `[workspace.members]`.
- **Proposed Fix:** After resolving P3WV1-A-C-001, re-run `cargo test --workspace`
  and update the wave certification count in STATE.md and wave-state.yaml.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 1 |
| HIGH | 3 |
| MEDIUM | 3 |
| LOW | 2 |
| OBSERVATION | 2 |
| **Total** | **11** |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — iterate after remediation
**Readiness:** Requires remediation of 7 blocking findings (1C + 3H + 3M) before Pass 2

## Remediation Priority

| Priority | Finding | Owner | When |
|----------|---------|-------|------|
| P0 (before Pass 2 cert) | P3WV1-A-C-001 | implementer | before Pass 2 |
| P1 (before any demo) | P3WV1-A-H-001, P3WV1-A-H-002 | implementer | before demo |
| P1 (state correction) | P3WV1-A-H-003, P3WV1-A-L-001, P3WV1-A-M-001 | state-manager | this burst |
| P2 (before Phase 4) | P3WV1-A-M-002, P3WV1-A-M-003 | implementer | before Phase 4 |
| P3 (Wave 2 risk) | P3WV1-A-L-002 | implementer | Wave 2 story 0 |
| OBS (Pass 2 scoping) | P3WV1-A-OBS-001, P3WV1-A-OBS-002 | orchestrator | Pass 2 setup |

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 1 |
| **New findings** | 11 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (11 / (11 + 0)) |
| **Median severity** | 3.0 (MEDIUM) |
| **Trajectory** | 11 (pass 1 baseline) |
| **Verdict** | FINDINGS_REMAIN |
