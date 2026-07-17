# PR-LEVEL Adversarial Pass 1 — DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 (PR #225)

**Pass type:** PR-LEVEL (fresh-context, TD-VSDD-005; no prior-pass knowledge; no `adversarial-review/` reads)
**Date:** 2026-07-17
**Reviewer:** adversary (fresh context)
**Story:** DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 v0.15 (`.factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md`)
**BCs:** BC-3.6.001 v0.8 (Precondition 4), BC-2.06.017 v1.12 (Postcondition 1)
**Error taxonomy:** E-DEMO-007 (error-taxonomy.md v2.54)

---

## Frozen-HEAD + PR-state verification

| Check | Expected | Observed | Result |
|-------|----------|----------|--------|
| `git rev-parse --short HEAD` (worktree) | `828449de` | `828449de` (`828449dedb8a3206df5dba4e0d63dc50b4c8246a`) | PASS |
| Working tree | clean | `git status --porcelain` empty | PASS |
| PR #225 state | OPEN | OPEN | PASS |
| PR #225 headRefOid | `828449de…` | `828449dedb8a3206df5dba4e0d63dc50b4c8246a` | PASS |
| Base branch | develop (`84062ced`) | develop; merge-base `84062ced` | PASS |
| Branch commits develop..HEAD | 21 (per PR claim) | 21 | PASS |

No push, commit, or file modification (outside this report) performed during this pass. Streak discipline preserved (DRIFT-ORCH-PRLEVEL-PUSH-001).

---

## Scope reviewed

1. Full diff `git diff develop...HEAD` (10 files, +2224/−22): `.gitignore`, `crates/prism-bin/tests/helpers/mod.rs`, `crates/prism-dtu-demo-server/{README.md, src/harness.rs, src/lib.rs, src/main.rs, src/multi_instance.rs, src/multi_org_cmd.rs, tests/defect_demo_configure_admintoken_001.rs, tests/td_wv1_04_binary_tls_e2e.rs}`.
2. PR #225 description — every factual claim checked against git/filesystem/test-run ground truth.
3. CI status (`gh pr checks 225`).
4. Story v0.15, BC-3.6.001 v0.8, BC-2.06.017 v1.12, error-taxonomy v2.54, ADR-003 Amendment #5.
5. Standing probes SAP-1, POL-22 Phase A/C, TD-VSDD-060.
6. Local test execution: full crate suite + defect suite with `--features fixture-gen`.

---

## Findings

### F-ADMTOK-PR1-MED-001 — PR description cites a nonexistent test (`test_BC_2_06_017_sweep_mirror_lock`) as Test J / AC-004 evidence

- **Severity:** MED
- **File/anchor:** PR #225 description — §Test Evidence "New Tests (This PR)" table (Test J row), §Traceability table (BC-2.06.017 / AC-004 row), §Spec Traceability mermaid (`TJ["Test J\nSWEEP-MIRROR\nlock"]` node).
- **Description:** The PR body claims Test J is `test_BC_2_06_017_sweep_mirror_lock` with purpose "SWEEP-MIRROR comment block present in cmd_configure + test module — PASS", and the Traceability table asserts AC-004 is verified by "Test J (SWEEP-MIRROR lock)". No test with that name exists anywhere in the branch. The actual Test J (per the test-file module inventory, line 79 of `crates/prism-dtu-demo-server/tests/defect_demo_configure_admintoken_001.rs`) is `test_resolve_configure_url_ambiguity_message_uses_sorted_org_list` — the F-ADMTOK-P12-OBS-001 sorted-org-list determinism lock on `resolve_configure_url`, unrelated to SWEEP-MIRROR block presence. There is no automated test asserting the SWEEP-MIRROR comment block exists; the story's AC-004 mechanism is the comment mirrors + story §Root Cause table + Test A's doc-comment cross-reference — which is fine per the spec, but the PR body fabricates test evidence that does not exist.
- **Evidence:** `rg -n 'sweep_mirror' crates/prism-dtu-demo-server/tests/defect_demo_configure_admintoken_001.rs` → zero function matches; full `fn test_` inventory of the file lists 11 tests, none named `test_BC_2_06_017_sweep_mirror_lock`. POL-22 Phase C (named-entity existence) FAIL on the PR surface.
- **Proposed routing:** pr-manager — amend the PR description (Test J row, Traceability AC-004 row, mermaid TJ node) to cite the real Test J name/purpose and restate AC-004's actual verification mechanism (SWEEP-MIRROR mirrors + story table + Test A cross-reference). PR-description edit only; does NOT touch the frozen HEAD, so no streak impact from the fix itself.

### F-ADMTOK-PR1-MED-002 — AC-004 ¶2 PR-surface MUST unmet: full grep outputs + explicit per-site reconciliation table absent from PR description

- **Severity:** MED
- **File/anchor:** Story v0.15 §AC-004 ¶2 vs PR #225 description §"TD-VSDD-060 Sibling Sweep (AC-004 requirement)".
- **Description:** Story AC-004 ¶2 (unchanged through v0.15) states: "The implementer MUST execute the following two-step reconciliation sweep and include the full output of both commands plus the reconciliation table in the PR description: Step 1 — `rg -n 'dtu/configure' crates/ --type rust`; Step 2 — `rg -n 'X-Admin-Token' crates/ --type rust` … The PR description MUST include the full output of both greps AND an explicit reconciliation table mapping each POST call site to its header status." The PR body instead carries: (a) the four count-only SWEEP-MIRROR commands (not the two `-n` full-output commands), (b) no grep output lines at all, (c) a 6-row per-class tally instead of a per-site table, delegating the per-site enumeration to the story §Root Cause table by pointer. Additionally, the PR body lists "Command (3) → 6 dynamic-URL" then states "Grand total: 131 + 7 dynamic + 8 …" without the "+1 `cmd_configure` dynamic caller not captured by command (3)" reconciliation sentence that the story footnote and code mirrors carry — leaving an unexplained 6-vs-7 discrepancy on the PR surface.
- **Context (spec-internal tension):** AC-004 ¶1 (v0.13/v0.14) codified the ratified SWEEP-MIRROR convention — condensed code mirrors + story §Root Cause table as source of truth. ¶2's literal "full output of both greps in the PR description" MUST (~450+ output lines) was never amended to match. The sweep SUBSTANCE is fully satisfied and independently reproduced this pass (counts 447/131/6/8 reproduce exactly; per-class tally sums to 146). The violation is the evidence-carrier form on the PR surface.
- **Evidence:** PR body §TD-VSDD-060 section (verified in full via `gh pr view 225 --json body`); story AC-004 ¶2 text; reproduction commands run from worktree root this pass: 447 / 131 / 6 / 8.
- **Proposed routing:** orchestrator adjudication with dual option: (a) product-owner amends AC-004 ¶2 (story v0.16) to ratify the condensed PR-surface form consistent with the ¶1 SWEEP-MIRROR convention (spec-internal contradiction repair — requires the standard spec-amendment authorization path), OR (b) pr-manager appends the two full `rg -n` outputs plus the per-site reconciliation table to the PR description. Either closes the finding; (a) is the durable fix.

### F-ADMTOK-PR1-LOW-001 — PR description numeric claims do not match ground truth (file-table +/- figures; test-file line count)

- **Severity:** LOW
- **File/anchor:** PR #225 description — §Test Evidence "Diff Stats" file table; opening summary paragraph; "New Tests (This PR)" heading line.
- **Description:** The per-file "+/-" column systematically presents `git diff --stat` total-changed-lines as additions with an invented "−1"/"−7" deletion figure, instead of real `--numstat` values:
  | File | PR claims | Actual (numstat +/−) |
  |------|-----------|----------------------|
  | `crates/prism-bin/tests/helpers/mod.rs` | +9/−1 | +4/−5 |
  | `crates/prism-dtu-demo-server/README.md` | +24/−1 | +17/−7 |
  | `crates/prism-dtu-demo-server/src/lib.rs` | +19/−1 | +17/−2 |
  | `crates/prism-dtu-demo-server/src/main.rs` | +126/−1 | +123/−3 |
  | `crates/prism-dtu-demo-server/src/multi_instance.rs` | +41/−1 | +40/−1 |
  | `crates/prism-dtu-demo-server/src/multi_org_cmd.rs` | +306/−7 | +302/−4 |
  Also: "A 1 612-line defect test suite" / "1 612 lines, 11 tests" — the file is 1611 lines (`wc -l` = 1611; numstat +1611). Aggregate diffstat claim "10 files changed, 2224 insertions(+), 22 deletions(-)" is CORRECT.
- **Proposed routing:** pr-manager — correct the file-table figures to numstat values and the line count to 1611. PR-description edit only.

### F-ADMTOK-PR1-LOW-002 — Known-Accepted §1 lists fabricated test names for the bc_2_06_018_seeding Red Gate failures

- **Severity:** LOW
- **File/anchor:** PR #225 description — §Known-Accepted / Pre-Adjudicated Items §1.
- **Description:** The PR body lists the three pre-adjudicated failing tests as `bc_2_06_018_seeding_1`, `bc_2_06_018_seeding_2`, `bc_2_06_018_seeding_3`. No tests with those names exist. The actual failing tests (confirmed by local run this pass) are, in `crates/prism-dtu-demo-server/tests/bc_2_06_018_seeding.rs`: `test_BC_2_06_018_e_demo_004_absent_org_id_at_construction`, `test_BC_2_06_018_e_demo_005_invalid_uuid_at_construction`, `test_BC_2_06_018_e_demo_001_at_construction_not_request_time`. The pre-adjudication itself (KNOWN-ACCEPTED item 1) is NOT re-flagged — only the fabricated names on the PR surface. POL-22 Phase C fail on PR citations.
- **Proposed routing:** pr-manager — replace with the three real test names.

### F-ADMTOK-PR1-OBS-001 — SAP-1 "91/91" figure in PR body is extraction-method-imprecise (substantive invariant holds)

- **Severity:** OBS
- **File/anchor:** PR #225 description — §Test Evidence Coverage Summary row "SAP-1 tracing catalog completeness | 91/91 distinct event_type literals present in BC-2.16.002 catalog".
- **Description:** Workspace-wide extraction of distinct quoted `event_type = "…"` strings yields exactly 91, but two of them are comment artifacts, not emissions: (a) `"..."` in `crates/prism-bin/tests/bc_2_10_006_mcp_stdout_purity.rs:167` (prose comment), and (b) `"timestamp_parse_failure"` in `crates/prism-spec-engine/src/pipeline.rs:561` — a comment documenting a REMOVED emission (F-LP2-HIGH-001), which correctly has no catalog row per SAP-1 rule 5 (removals need no row). `"timestamp_parse_failure"` is NOT present in the BC-2.16.002 catalog, so "91/91 present" is not literally reproducible; the accurate statement is "89/89 actual emissions cataloged; 2 comment artifacts excluded". The substantive SAP-1 invariant PASSES (see SAP-1 result below).
- **Proposed routing:** pr-manager — optional wording correction in the Coverage Summary row.

---

## PR-description claim verification (ground-truth results)

| Claim | Verification | Result |
|-------|--------------|--------|
| 21 commits develop → HEAD | `git log --oneline develop..HEAD \| wc -l` = 21 | PASS |
| Frozen HEAD `828449de`; convergence passes 19/20/21 on frozen HEAD | HEAD verified; STORY-INDEX v2.701 (D-1804) records pass-19 dispatched on frozen 828449de after fb-15 streak reset — consistent (pass reports not read per fresh-context wall) | PASS (consistent) |
| Full crate 60/60 pass + 3 known-accepted failures | Local `cargo nextest run -p prism-dtu-demo-server --no-fail-fast`: 63 run, 60 passed, 3 failed (the three bc_2_06_018 tests) | PASS |
| Defect suite 10/10 (11/11 with `--features fixture-gen`) | Local run with `--features fixture-gen -E 'binary(defect_demo_configure_admintoken_001)'`: 11/11 PASS (10 compiled without fixture-gen: Test G is `cfg(all(unix, feature="fixture-gen"))`) | PASS |
| Sweep counts 447 / 131 / 6 / 8; grand total 146 | All four commands reproduce exactly from worktree root; per-class tally 1+1+111+17+15+1 = 146 | PASS (but see MED-002 re: PR-surface form) |
| Story version 0.15 | Frontmatter `version: "0.15"` | PASS |
| BC-3.6.001 v0.8 / BC-2.06.017 v1.12 pins | Frontmatter versions match; BC-2.06.017 `modified: 2026-07-17` matches newest changelog row (POL-27 PASS) | PASS |
| E-DEMO-007 registered, taxonomy v2.54 | error-taxonomy.md v2.54, row present, template byte-identical to code (POL-24 PASS) | PASS |
| New `event_type=` emissions in diff = 0 | `git diff develop...HEAD \| grep -c event_type` = 0 | PASS |
| AD-017: `token_present=true` placeholder, no token in log fields | `cmd_configure` `tracing::debug!(clone = %clone_name, token_present = true, …)` — no token value | PASS |
| 0600 perms + atomic tmp+rename on token sidecars | `write_token_sidecar_to_path` / `write_multi_admin_token_sidecar_to_path`: `OpenOptions … .mode(0o600)` + `fs::rename`; contract-locked by umask-robust `mode & 0o077 == 0` assertions in Tests B/F/K | PASS |
| DRIFT-HARNESS-ADMIN-TOKEN-CT-001 out of scope | Story §Excluded from scope confirms (D-1666 fold into S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001) — KNOWN-ACCEPTED §3 | PASS |
| "1 612-line" test suite | 1611 lines | FAIL → F-ADMTOK-PR1-LOW-001 |
| File-table +/- figures | numstat mismatch on 6 of 10 rows | FAIL → F-ADMTOK-PR1-LOW-001 |
| Test J = `test_BC_2_06_017_sweep_mirror_lock` | Test does not exist | FAIL → F-ADMTOK-PR1-MED-001 |
| Known-accepted test names `bc_2_06_018_seeding_{1,2,3}` | Names do not exist | FAIL → F-ADMTOK-PR1-LOW-002 |
| SAP-1 "91/91" | 89/89 real emissions cataloged; 2 comment artifacts | IMPRECISE → F-ADMTOK-PR1-OBS-001 |

---

## Code review (diff substance) — no findings

- **Fix correctness:** `cmd_configure` resolves the token via `resolve_configure_token` BEFORE building the POST (no POST on token-resolution failure — AC-003 item 3 satisfied structurally: `?` propagation precedes client construction), attaches `X-Admin-Token`. Flat-first / nested-fallback / bare-name disambiguation mirrors `resolve_configure_url`; flat-miss no-fallthrough locked by Test H; nested zero-match by Test I; EC-005 ambiguity by Test D; EC-004 by Test C.
- **E-DEMO-007 template:** `e_demo_007` helper produces `"configure: E-DEMO-007: admin token for clone '{clone_name}' could not be resolved: {reason}"` — byte-identical to error-taxonomy v2.54 `message_template` (POL-24 PASS). Reason strings for EC-003/EC-004 match story AC-003 verbatim.
- **Ownership constraint (BC-2.06.017 Postcondition 1):** `start_instances` extracts `instance.clone.admin_token().to_string()` before the `tokio::spawn(async move { drop(clone) })` move; `token_map` populated at bind time, never mutated; `admin_token_map()` accessor matches the BC signature `-> &HashMap<String, String>`.
- **`_global` enrichment arm:** emitted per ENRICH-3 mirror; fail-loud `ok_or_else(...)?` on missing enrichment token, locked by Test K assertion (d); non-leakage locked by Test K (c1)/(c2).
- **Fail-loud discipline (Standing Rule 3 §2):** no silent skips; missing `{org_slug}-{sensor_id}` entries error loudly with sorted key lists (determinism sorted at all 4 diagnostic-list sites + `resolve_configure_url` ambiguity sibling, locked by Tests F/J).
- **No unwrap/expect in production paths** (test file carries `#![allow(clippy::unwrap_used, clippy::expect_used, …)]`; `unwrap_or("tokens")` is an infallible-fallback, not a Result unwrap). No `todo!()`/`unimplemented!()` residue (POL-12 PASS). No new pub types needing `#[non_exhaustive]` (functions/consts/fields only; `MultiInstanceServers` already `#[non_exhaustive]`). No new reqwest client (existing 10s-timeout client retained per story's ratified crate-local exception). No `println!` additions outside the existing CLI-formatter exception.
- **T-09 cleanup:** both shutdown paths remove the corresponding token sidecar; locked by Tests E/G post-shutdown assertions. README "all files atomic + removed on shutdown" claim verified including PID file (`write_pid_file` uses tmp+rename); README "read by configure" correction accurate (`cmd_stop` reads only PID_FILE).
- **KillGuard RAII (td_wv1_04 + Tests E/G):** SIGKILL-on-panic with `std::mem::forget` disarm after `wait()` — correct recycled-pid protection; TD-VSDD-060 sibling sweep applied at all 3 pre-existing td_wv1_04 sites.
- **Test quality (TD-VSDD-059):** no tautological tests. Load-bearing verified by inspection: Test B/E fail if the header line is removed (401→exit 1); Test H's nested-sidecar-contains-clone construction makes fallthrough observable; Tests F/K fail-loud probes distinguish `?` from silent-skip; 0600 locks are mutation-sensitive (`mode & 0o077`, umask-robust). Test A is a ratified defect-observability contract lock per story AC-001 note (not an inverted-polarity stub test — POL-16 PASS).
- **`.gitignore`:** token sidecar + `.tmp` patterns added, including `pid.tmp` parity.

## Spec verification — no findings

- Story v0.15 frontmatter/changelog consistent (POL-32: newest-first, no gaps v0.1–v0.15); STORY-INDEX row agrees with frontmatter status `draft` v0.15 (POL-13 PASS).
- BC-3.6.001 v0.8 Precondition 4 quoted verbatim in story (POL-22 Phase A PASS). BC H1 titles match story citations exactly (POL-7 PASS): "Per-Org Failure Injection"; "Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing".
- BC-2.06.017 v1.12 Postcondition 1 enumerates `admin_token_map()` + `TOKEN_MULTI_FILE` nested format + pre-move extraction — code conforms exactly.
- ADR-003 Amendment #5 §Decision paragraph and §Implementation item 4 quoted byte-verbatim in story §Root Cause (POL-22 Phase A PASS).
- SS-22 subsystem anchor: story cites ARCH-INDEX SS-22 (Binary Entrypoint) for the CLI surface — consistent with crate/binary placement (POL-6, no verbatim-name deviation found).

---

## SAP-1 result

`rg 'event_type\s*=' crates/ --type rust` → 230 raw hits; 91 distinct quoted string literals. 89 are real emission values — ALL 89 present in the BC-2.16.002 §Postconditions Canonical Structured Event Catalog (`BC-2.16.002-multi-step-fetch-pipeline.md`). The 2 non-emission artifacts: `"..."` (prose comment, `bc_2_10_006_mcp_stdout_purity.rs:167`) and `"timestamp_parse_failure"` (comment documenting a removed emission, `pipeline.rs:561`; no row required per SAP-1 rule 5 / D-765). The diff adds ZERO new `event_type=` emissions (the new `tracing::debug!` in `cmd_configure` carries no `event_type` field). **SAP-1: PASS** (see F-ADMTOK-PR1-OBS-001 for the PR-body "91/91" wording only).

## POL-22 Phase A / Phase C results

- **Phase A (lexical-vs-semantic anchors):** BC-3.6.001 Precondition 4 — verbatim + semantically correct (normative admin-token requirement for configure callers). BC-2.06.017 Postcondition 1 — semantically correct (admin_token_map/TOKEN_MULTI_FILE enumerated as of v1.11/v1.12). ADR-003 Amendment #5 §Decision + §Implementation item 4 — verbatim. E-DEMO-007 template — byte-verbatim in taxonomy, story, and code. GAP-3 citation form ("sidecar-poll note, Changelog v2.1") — conforms to the v1.12/v0.13 corrected citation. **PASS.**
- **Phase C (named-entity existence):** All spec-cited code entities exist: `resolve_configure_token`, `resolve_configure_url`, `write_token_sidecar_to_path`, `write_multi_admin_token_sidecar_to_path`, `write_token_sidecar`, `DemoHarness::token_map`, `MultiInstanceServers::admin_token_map`, `TOKEN_FILE`, `TOKEN_MULTI_FILE`, `KNOWN_ENRICHMENT_CLONES`, `Harness::admin_token_for`, `Harness::inject_failure`, all 11 defect-suite test names cited in the story. **PASS for story/BC citations.** **FAIL for two PR-description citations:** `test_BC_2_06_017_sweep_mirror_lock` (F-ADMTOK-PR1-MED-001) and `bc_2_06_018_seeding_{1,2,3}` (F-ADMTOK-PR1-LOW-002).

## TD-VSDD-060 sibling-sweep verification

Independently re-ran all four SWEEP-MIRROR commands: 447 / 131 / 6 / 8 — exact match with code mirrors (main.rs + defect test module header), story v0.15 footnote, and PR body counts. The 6→7 dynamic-caller reconciliation (+1 `cmd_configure` via `resolve_configure_url` → `.post(&url)`) is documented in code and story (missing only from the PR body — folded into MED-002). New identifiers introduced by the branch (`TOKEN_FILE`, `TOKEN_MULTI_FILE`, `resolve_configure_token`, `admin_token_map`, `token_map`, `write_token_sidecar_to_path`, `write_multi_admin_token_sidecar_to_path`) — all call/re-export sites consistent across `lib.rs`, `main.rs`, `multi_org_cmd.rs`, `harness.rs`, `multi_instance.rs`, and the test suite. KillGuard pattern swept to all 3 td_wv1_04 sites. **PASS.**

## CI status (`gh pr checks 225`, at review time)

- **PASS (completed):** ADR-023 compile-fail gate, Cargo audit, Cargo deny, Clippy (AD-008), Deep-recursion stack-guard lint, Format check, Non-exhaustive compile-fail, Perimeter compile-fail + symbols sync, Semver compatibility, Shellcheck demo scripts, ThreatIntel .prx staleness guard, Verify workflow structure, WASM32 compile check, Workspace crate layout — all green on both workflow runs (29598844886, 29598871177).
- **PENDING-CI (not findings):** Test (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl, x86_64-pc-windows-msvc), Test (no-default-features), Fuzz smoke (vp021_parse_fuzz), E2E smoke.
- **FAILED:** none.

## KNOWN-ACCEPTED items honored (not re-flagged)

1. 3 `bc_2_06_018_seeding` Red Gate failures (observed locally; pre-adjudicated). 2. `DEMO_ORG_UUID_B` clippy warning (observed under fixture-gen build). 3. DRIFT-HARNESS-ADMIN-TOKEN-CT-001. 4. Pre-existing `.tmp` sidecar file-permissions edge (0600 applies at tmp-file creation only). 5. Missing `[[test]]` Cargo.toml entry pattern.

---

## Verdict

Findings: **2 MED, 2 LOW, 1 OBS** (0 CRIT, 0 HIGH). All five findings are PR-description-surface defects; the code diff, tests, and spec artifacts are clean under this pass.

```
CLEAN (strict): no    [zero findings of ANY severity]
CLEAN (PR-merge): no  [zero CRIT+HIGH+MED]
```

Streak: remains **0/3** on frozen HEAD `828449de`. Note for orchestrator: F-ADMTOK-PR1-MED-001, LOW-001, LOW-002, OBS-001 are closable by PR-description edits (no push, no HEAD change); F-ADMTOK-PR1-MED-002 is closable either by PR-description amendment or by product-owner story AC-004 ¶2 amendment (spec-internal ¶1↔¶2 contradiction repair).
