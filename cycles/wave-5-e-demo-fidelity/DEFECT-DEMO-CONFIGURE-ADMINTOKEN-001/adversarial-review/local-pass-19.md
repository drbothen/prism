# DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 — LOCAL Adversarial Pass 19

**Reviewer:** vsdd-factory adversary (fresh context, TD-VSDD-005 — no prior pass reports read)
**Date:** 2026-07-17
**Branch:** `fix/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001`
**Frozen HEAD:** `828449de` — VERIFIED (`git rev-parse --short HEAD` = `828449de`; `git status --porcelain` empty)
**Diff base:** `develop` (84062ced) — `git diff develop...HEAD` = 10 files, +2224/−22
**Story:** `.factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md` — frontmatter `version: "0.15"` VERIFIED (matches dispatch pin)
**Anchoring BCs:** BC-2.06.017 `version: "1.12"` VERIFIED; BC-3.6.001 `version: "0.8"` VERIFIED (matches story BC table pin)

---

## Verdict

```
CLEAN (strict): yes
CLEAN (PR-merge): yes
```

**ZERO FINDINGS** (all severities). Full evidence trail below, including two candidate findings that were investigated and adjudicated NOT-A-FINDING with recorded rationale.

---

## 1. Frozen-HEAD verification

| Check | Result |
|---|---|
| `git rev-parse --short HEAD` | `828449de` ✓ |
| `git branch --show-current` | `fix/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001` ✓ |
| `git status --porcelain` | empty ✓ |

## 2. Test evidence (executed this pass, at frozen HEAD)

| Run | Result |
|---|---|
| `cargo nextest run -p prism-dtu-demo-server --test defect_demo_configure_admintoken_001` (default features) | **10/10 PASS** (Tests A, B, C, D, E, F, H, I, J, K; Test E binary E2E included) |
| Same, `--features fixture-gen` | **11/11 PASS** (Test G binary start-multi E2E compiled in and passed) |
| `cargo nextest run -p prism-dtu-demo-server --no-fail-fast` (full crate) | 60 passed, **3 failed = exactly the 3 known-accepted `bc_2_06_018_seeding` Red Gate tests** (pre-adjudicated, different BC's red gate). No other regression. |

## 3. AC satisfaction audit

- **AC-001** — Test A (`test_BC_3_6_001_replicates_defect_401_without_admin_token`) POSTs `{"auth_mode": "accept"}` without header → asserts 401 (contract lock, correctly documented per story v0.7 clarification). Red-Gate obligation carried by Test E binary E2E (`configure` exits 0 + stdout "200"; reverting T-08 → 401 → exit 1) and sidecar-existence assertions. SATISFIED.
- **AC-002** — `cmd_configure` reads `resolve_configure_token(TOKEN_FILE, TOKEN_MULTI_FILE)` with flat-first / nested-fallback / bare-name-disambiguation precedence mirroring `resolve_configure_url`; attaches `.header("X-Admin-Token", &admin_token)`. `cmd_start` writes `TOKEN_FILE` via `write_token_sidecar` → `write_token_sidecar_to_path` (atomic tmp+rename, 0600 Unix) immediately after `write_url_sidecar`. `cmd_start_multi` writes `TOKEN_MULTI_FILE` via `write_multi_admin_token_sidecar_to_path` incl. `_global` enrichment arm (fail-loud, Test K a–d). `MultiInstanceServers::admin_token_map()` populated pre-move in `start_instances` watcher-spawn loop (OWNERSHIP constraint honored — token extracted via `to_string()` before `tokio::spawn(async move {{ drop(clone) }})`). T-09 cleanup in both shutdown handlers, locked by Test E / Test G assertions. `TOKEN_FILE`/`TOKEN_MULTI_FILE` are `pub const` in `lib.rs`; re-exports present. SATISFIED.
- **AC-003** — E-DEMO-007 template in `resolve_configure_token` is byte-identical to story + error-taxonomy.md line 615 (taxonomy v2.54 registered per POL-24; T-11 done). EC-003 (Tests H, I), EC-004 (Test C), EC-005 (Test D) all covered with genuine load-bearing `expect_err` + message-content assertions. Token resolution occurs BEFORE the POST is built — no silent-401 path; `?` propagation → anyhow main → stderr + exit 1; no panic. SATISFIED.
- **AC-004** — All four SWEEP-MIRROR reproducible commands re-executed at frozen HEAD: **447 / 131 / 6 / 8** — byte-identical to the counts pinned in story §Root Cause footnote, `cmd_configure` mirror block, and defect-test module header. Grand total 146 HTTP POST client calls; `cmd_configure` confirmed the only defect site. SATISFIED.

## 4. SAP-1 — tracing emission catalog completeness

- `rg 'event_type\s*=' crates/ --type rust` across the ENTIRE worktree: 230 hits in 34 files.
- **Branch delta:** `git diff develop...HEAD | grep event_type` → zero hits. The one new tracing site (`tracing::debug!(clone = %clone_name, token_present = true, ...)` in `cmd_configure`) carries NO `event_type` field and logs `token_present=true`, not the token value (AD-017 ✓). No catalog obligation triggered.
- **Workspace set-diff:** every distinct string-literal `event_type` value (91 values) was checked for presence in BC-2.16.002 (`BC-2.16.002-multi-step-fetch-pipeline.md`). Single unmatched token `timestamp_parse_failure` traced to `prism-spec-engine/src/pipeline.rs:561` — a COMMENT documenting a REMOVED emission (F-LP2-HIGH-001; removal needs no catalog row per D-765 / SAP-1 rule 5). Non-literal sites (`%self.event_type` in prism-credentials audit, `event.event_type` in prism-security flag_audit) are pre-existing dynamic pass-throughs unchanged by this branch.
- **SAP-1 result: PASS — no P1.**

## 5. POL-22 Phase A (lexical-vs-semantic anchor verification)

| Citation | Verified |
|---|---|
| ADR-003 Amendment #5 §Decision block-quote in story §Root Cause | Byte-verbatim match against `decisions/ADR-003-dtu-reset-lookup-and-fidelity-auth.md` `### Decision` under `## Amendment #5` ✓ |
| ADR-003 Amendment #5 §Implementation item 4 block-quote | Byte-verbatim ("All 12 existing `td_wv0_04` configure tests (2 per clone) and all other integration tests calling `/dtu/configure`: updated to include `.header(\"X-Admin-Token\", clone.admin_token())`.") ✓ — and `### Implementation` is a real heading (POL-21 ✓; §Decision/§Rationale/§Implementation/§Backward Compatibility/§Rejected Alternatives/§Scope all real `###` headings) |
| BC-3.6.001 Precondition 4 quote (story BC table + AC traces) | Verbatim: "The `inject_failure` call uses `POST /dtu/configure` on the clone's admin endpoint, authenticated with that clone's `admin_token` (ADR-003 Amendment §5)." ✓ |
| BC-2.06.017 v1.12 Postcondition 1 claims | `admin_token_map() -> &HashMap<String, String>` accessor and `TOKEN_MULTI_FILE` sidecar both enumerated in Postcondition 1; v1.12 changelog row (2026-07-17) documents the phantom-anchor correction exactly as the story v0.13 changelog describes; `modified: "2026-07-17"` matches newest changelog date (POL-27 ✓); v1.11 row not rewritten (POL-32 ✓) |
| E-DEMO-007 message template (POL-24) | Byte-identical across story §Error Taxonomy Addition, error-taxonomy.md line 615 (v2.54 changelog row present), and `resolve_configure_token` code ✓ |
| EC-005 template with `{:?}`-quoted sorted org list | Matches code (`bare_matches.sort_by` + `{:?}` render) and locked by Test D assertion `["org-a", "org-b"]` ✓ |

**Phase A result: PASS.**

## 6. POL-22 Phase C (named-entity existence verification)

All load-bearing named entities in the story/tests confirmed to exist at frozen HEAD:

- `Harness::admin_token_for` — `crates/prism-dtu-harness/src/harness.rs:183` ✓; `Harness::inject_failure` attaches `x-admin-token` (harness.rs:291) ✓
- `test_build_harness_http_client_timeout_is_load_bearing` — `crates/prism-dtu-harness/src/builder.rs:1224` ✓ (synthetic hung-socket, header N/A — classification correct)
- `test_enrich3_sidecar_emits_global_key_for_enrichment` (Test K's cited URL twin) — `crates/prism-dtu-demo-server/tests/enrich_23_dtu_wiring.rs` ✓
- `CrowdstrikeState::default` → `Self::with_admin_token(uuid::Uuid::new_v4().to_string())` (state.rs:413-416) — EC-002 "no dev-mode no-token path" claim ✓
- `sec_p3_003_constant_time_admin_token.rs` — exists in prism-dtu-claroty/tests ✓
- `KNOWN_ENRICHMENT_CLONES = &["threatintel", "nvd"]` — config.rs:283 ✓ (matches `_global` arm's threatintel/nvd match)
- `TOKEN_FILE` / `TOKEN_MULTI_FILE` pub consts, `resolve_configure_token`, `write_token_sidecar_to_path`, `write_multi_admin_token_sidecar_to_path`, `DemoHarness::token_map`, `MultiInstanceServers::admin_token_map` — all exist and re-exported via lib.rs ✓
- Out-of-scope pointer `S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001` — story file exists in `.factory/stories/` ✓
- Test names cited in story EC-005/AC-002 (Test D `test_BC_3_6_001_e_demo_007_ec005_ambiguous_bare_sensor_name`, Test K `test_BC_2_06_017_start_multi_enrichment_token_global_key_written_and_resolved`) — exist and pass ✓

**Phase C result: PASS.**

## 7. TD-VSDD-060 sibling-site sweep (diff-changed identifiers)

- `MultiInstanceServers` gained private `token_map` field — both construction sites (empty-config early return + main bind loop in `start_instances`) updated in the same diff; no other constructors exist. ✓
- `socket_map.insert(instance.name → instance.name.clone())` — internal-only change. ✓
- lib.rs re-export list extended for the two new pub fns; no stale importers. ✓
- Deterministic-sort changes to `write_multi_url_sidecar_to_path` / `resolve_configure_url` error messages — sibling determinism locked by Test J (URL arm), Test F (token write arm), Test K (enrichment arm); full-crate run shows no test asserting the old unsorted form. ✓
- KillGuard sweep into `td_wv1_04_binary_tls_e2e.rs` (3 subprocess tests) with disarm-after-wait `mem::forget` — correct (prevents recycled-pid SIGKILL). ✓

## 8. CLAUDE.md §Conventions compliance (diff scope)

- **unwrap/expect in production paths:** none — all fallible ops use `map_err`/`ok_or_else`/`?`; the only `unwrap_or` is `Option::unwrap_or("tokens")` on a filename (not a Result). ✓
- **reqwest:** existing `cmd_configure` client retains `.timeout(10s)` (ratified crate-local exception documented in story Architecture Compliance Rules); Cargo.toml declares `default-features = false, features = ["rustls-tls"]` in both dep sections (ADR-050 ✓); no new reqwest dep entry. ✓
- **#[non_exhaustive]:** no new public structs/enums; `MultiInstanceServers` already `#[non_exhaustive]`; new field private. Gate count 92 untouched. ✓
- **println!:** only in CLI output formatting paths (ratified exception). ✓
- **AD-017:** token values never logged (`token_present=true` placeholder); sidecar docs prohibit token logging; 0600 Unix perms on both sidecar writers, locked by umask-robust `mode() & 0o077 == 0` assertions in Tests B, F, K. ✓
- **POLICY 12/16:** no `todo!()`/`unimplemented!()`/stub-panic in production paths; Test A is a documented contract lock (server 401 gate pre-dates branch), not an inverted-polarity stub test. ✓
- **POLICY 10:** LOCAL cascade stage — no demo-evidence obligation evaluated here (demo-recorder runs per-AC after LOCAL 3-CLEAN per pipeline order).

## 9. Test-quality audit (TD-VSDD-059 paper-fix screen)

Every test's "Load-bearing: what revert breaks this" claim was checked against the code paths:

- Test E asserts subprocess exit-0 + stdout "200" + post-shutdown TOKEN_FILE absence → genuinely load-bearing against T-08 and T-09 reverts.
- Test H writes the nested sidecar WITH the clone to make fall-through observable — a genuine no-fallthrough lock, not tautological.
- Test F fail-loud arm asserts `is_err()` + missing-entry name + sorted-keys rendering → load-bearing against silent-skip regression (Standing Rule 3 §2).
- Test K (a)-(d) cover `_global` write, bare-scan resolution, non-leakage both directions, and fail-loud enrichment arm.
- Tests D/J lock the `{:?}` sorted rendering (would flake ~50% without the sorts).
No tautological assertions, no assert-only paper-fixes found.

## 10. Candidate findings investigated and adjudicated NOT-A-FINDING

1. **Pre-existing `.tmp` permissions edge (both sidecar writers).** `OpenOptions::mode(0o600)` applies only at file creation; a pre-existing `.tmp` (e.g., adversarially pre-created with wide perms) would retain its mode through truncate+rename. Adjudication: NOT a finding. (a) The only non-adversarial producer of a stale `.tmp` is a crashed prior run of the same binary, which created it 0600. (b) The adversarial scenario requires attacker write access to the operator's demo cwd, at which point the attacker controls the config file and both URL sidecars — a strictly stronger position than reading ephemeral demo admin tokens. (c) The crate is feature-gated test/demo infrastructure (`required-features = ["dtu"]`, never links into production); the 0600 control itself was added as consistency hardening beyond spec (F-ADMTOK-P1-OBS-002 lineage), and the doc claim ("tmp file is created with mode 0600") is accurate for every creation path. Below the artifact's threat-model floor; spec (story/BC/ADR) imposes no requirement this violates.
2. **No `[[test]]` Cargo.toml entry for `defect_demo_configure_admintoken_001.rs`.** The crate declares explicit `[[test]]` targets with `required-features = ["dtu"]` for 24 test files, but three pre-existing develop-side files (`enrich_23_dtu_wiring.rs`, `bc_2_06_019_served_route_enrich_composed.rs`, `bc_2_06_019_served_route_enrich_armis_nvd_composed.rs`) are already auto-discovered without entries — the new file follows the existing tolerated pattern, not a branch-introduced deviation. CI's no-default-features job runs `--exclude prism-dtu-demo-server` (ci.yml:181), so no exercised configuration is affected; default features include `dtu`, and Test G is correctly `#[cfg(all(unix, feature = "fixture-gen"))]`-gated in-file (required-features on the whole target would wrongly exclude Tests A–F/H–K from default runs — in-file gating is the correct design for this mixed-feature file). Pre-existing, inert, and design-appropriate; not chargeable to this fix.

## 11. Known-accepted items observed (not reported, per dispatch)

1. 3 failing `bc_2_06_018_seeding` Red Gate tests — observed in full-crate run, excluded.
2. `DEMO_ORG_UUID_B` clippy warning — pre-adjudicated, excluded.
3. DRIFT-HARNESS-ADMIN-TOKEN-CT-001 (server-side constant-time comparison) — accepted-excluded; story §Excluded-from-scope correctly routes it to S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001 (D-1666).

---

## Findings

**NONE.** (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

## Dual verdict

```
CLEAN (strict): yes
CLEAN (PR-merge): yes
```
