# DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 — LOCAL Adversarial Pass 21

- **Reviewer:** vsdd-factory adversary (fresh context, TD-VSDD-005; no prior pass reports read)
- **Date:** 2026-07-17
- **Branch:** `fix/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001`
- **Frozen HEAD:** `828449de` — VERIFIED (`git rev-parse --short HEAD` = `828449de`; `git status --porcelain` = empty)
- **Diff base:** `develop` = `84062ced90e4848b69042f1deaa3dc508d0f74d6` (verified merge-base == develop head)
- **Story spec:** `DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md` — frontmatter `version: "0.15"` VERIFIED
- **Anchoring BC:** BC-2.06.017 — frontmatter `version: "1.12"` VERIFIED; BC-3.6.001 `version: "0.8"` matches story BC-table pin

## Verdict

```
CLEAN (strict): yes
CLEAN (PR-merge): yes
```

**ZERO FINDINGS** (no CRIT / HIGH / MED / LOW / OBS / PROCESS-GAP).

---

## Scope reviewed

Full `git diff develop...HEAD` (10 files, +2224/−22):

| File | Reviewed |
|------|----------|
| `.gitignore` | yes — new sidecar + `.tmp` patterns covered |
| `crates/prism-bin/tests/helpers/mod.rs` | yes — stale doc-comment reference to `configure_cyberint_dtu_access_token()` removed; workspace grep confirms zero remaining references to that identifier |
| `crates/prism-dtu-demo-server/README.md` | yes — sidecar table + configure docs accurate vs code (E-DEMO-007, four sidecar files, atomic write, shutdown removal) |
| `crates/prism-dtu-demo-server/src/harness.rs` | yes — `DemoHarness::token_map()`, `write_token_sidecar_to_path` (atomic tmp+rename, 0600 Unix) |
| `crates/prism-dtu-demo-server/src/lib.rs` | yes — `TOKEN_FILE`/`TOKEN_MULTI_FILE` pub consts + re-exports |
| `crates/prism-dtu-demo-server/src/main.rs` | yes — T-04/T-06/T-08/T-09 wiring, SWEEP-MIRROR block, `resolve_configure_token` call + `X-Admin-Token` header attach |
| `crates/prism-dtu-demo-server/src/multi_instance.rs` | yes — `token_map` field, `admin_token_map()`, pre-move token extraction (OWNERSHIP), both `MultiInstanceServers` construction sites updated |
| `crates/prism-dtu-demo-server/src/multi_org_cmd.rs` | yes — `write_multi_admin_token_sidecar_to_path` (fail-loud, `_global` enrichment arm), `resolve_configure_token` (flat-first / nested / bare-name / EC-005 ambiguity / EC-003 / EC-004), determinism sorts |
| `crates/prism-dtu-demo-server/tests/defect_demo_configure_admintoken_001.rs` | yes — all 1612 lines; Tests A,B,C,D,E,F,G,H,I,J,K |
| `crates/prism-dtu-demo-server/tests/td_wv1_04_binary_tls_e2e.rs` | yes — KillGuard RAII + disarm-after-wait pattern at 3 sites |

## AC satisfaction

| AC | Verdict | Evidence |
|----|---------|----------|
| AC-001 | SATISFIED | Test A contract lock (401 without header, `{"auth_mode":"accept"}` payload per AC literal); Test E binary E2E asserts exit 0 + "200" stdout (load-bearing against T-08 revert); both pass |
| AC-002 | SATISFIED | T-01..T-09 all present as specified: pub consts in lib.rs; `admin_token_map()` with pre-move extraction; `write_token_sidecar` in `cmd_start` immediately after `write_url_sidecar`; nested sidecar in `cmd_start_multi`; flat-first/nested-fallback/bare-name precedence mirrors `resolve_configure_url`; `_global` enrichment arm with fail-loud `ok_or_else(...)?` (Test K a–d); shutdown removal in both signal handlers; atomic tmp+rename with 0600 Unix perms (Tests B/F/K `mode & 0o077 == 0` umask-robust locks) |
| AC-003 | SATISFIED | E-DEMO-007 emitted via `e_demo_007` closure with byte-verbatim template; EC-003 (Tests H flat-arm no-fallthrough + I nested zero-match), EC-004 (Test C), EC-005 (Test D, `{:?}`-quoted sorted org list); Err propagation → stderr + exit 1 via existing `anyhow::Result<()>` main; no panic paths; no POST attempted on unresolved token (`?` before client build) |
| AC-004 | SATISFIED | SWEEP-MIRROR blocks present in `cmd_configure` body and test module header; command forms byte-identical to story footnote; counts independently reproduced this pass: (1)=447, (2)=131, (3)=6, (4)=8 → 131+7+8=146 |

## Test execution (this pass, on frozen 828449de)

- `cargo nextest run -p prism-dtu-demo-server --test defect_demo_configure_admintoken_001 --no-fail-fast` → **10/10 PASS** (A,B,C,D,E,F,H,I,J,K)
- `--features fixture-gen -E 'test(startmulti)'` → **Test G PASS**
- Full crate `cargo nextest run -p prism-dtu-demo-server --no-fail-fast` → **60/63 pass**; 3 failures are exactly the KNOWN-ACCEPTED `bc_2_06_018_seeding` red-gate tests (pre-adjudicated item 1)
- `cargo clippy -p prism-dtu-demo-server --all-targets` → sole warning is KNOWN-ACCEPTED `DEMO_ORG_UUID_B` (pre-adjudicated item 2)

## SAP-1 — Tracing emission catalog completeness: PASS

- `rg 'event_type\s*=' crates/ --type rust` → 230 sites workspace-wide; 172 with same-line string literals → **91 distinct literal values**
- Scripted check of all 91 values against BC-2.16.002 §Postconditions Canonical Structured Event Catalog: **91/91 present**. Sole initial non-match `timestamp_parse_failure` is a COMMENT documenting a removed emission (`pipeline.rs:561`, F-LP2-HIGH-001 removal — no action needed per SAP-1 rule 5); the live sibling `ocsf.timestamp_parse_failed` has catalog row (BC-2.16.002 row at line 139)
- Non-literal `event_type =` matches are test assertions, PQL query strings, and "SAP-1 exempt" comments — no uncataloged emissions
- **Diff delta: ZERO new `event_type=` emissions.** The one new tracing site (`tracing::debug!(clone = %clone_name, token_present = true, ...)` in `cmd_configure`) carries no `event_type` field (catalog not required) and no token value (AD-017 compliant — `token_present=true` placeholder exactly as story Architecture Compliance Rules mandate)

## POL-22 two-phase verification: PASS

**Phase A (lexical-vs-semantic anchors):**
- ADR-003 Amendment #5 §Decision block-quote in story §Root Cause — verbatim match at ADR lines 628–632 ("POST /dtu/configure on every DTU clone MUST require a valid X-Admin-Token header… {\"error\": \"missing or invalid X-Admin-Token\"}")
- ADR-003 Amendment #5 §Implementation item 4 quote — verbatim match at ADR line 664 ("All 12 existing `td_wv0_04` configure tests…"); `### Decision` / `### Implementation` headings exist under `## Amendment #5` (POL-21 clean)
- BC-3.6.001 Precondition 4 — verbatim match ("The `inject_failure` call uses `POST /dtu/configure` … authenticated with that clone's `admin_token` (ADR-003 Amendment §5)")
- BC-2.06.017 v1.12 Postcondition 1 — enumerates `admin_token_map() -> &HashMap<String, String>` (line 90) and TOKEN_MULTI_FILE sidecar feed (line 98); v1.12 changelog row confirms phantom-anchor correction; `modified: 2026-07-17` matches newest changelog date (POL-27/POL-32 clean)
- E-DEMO-007: story §Error Taxonomy row ↔ error-taxonomy.md v2.54 row (line 615) ↔ code format string in `resolve_configure_token` — message template byte-verbatim across all three (POL-24 clean)
- EC-005 template: code ambiguity arm renders sorted `Vec<String>` via `{:?}` → `["org-a", "org-b"]` exactly as story specifies; Test D locks quoting + order

**Phase C (named-entity existence):**
- `write_token_sidecar_to_path` (harness.rs, re-exported lib.rs), `write_token_sidecar` binary wrapper (main.rs), `DemoHarness::token_map()`, `MultiInstanceServers::admin_token_map()`, `write_multi_admin_token_sidecar_to_path`, `resolve_configure_token` — all exist at the story's stated locations (v0.14 as-built table accurate)
- `Harness::admin_token_for()` exists (`prism-dtu-harness/src/harness.rs:183`)
- `KNOWN_ENRICHMENT_CLONES = ["threatintel", "nvd"]` exists (`config.rs:283`)
- ENRICH-3 resolves to real Red Gate tests RG-E3-001/RG-E3-002 in `enrich_23_dtu_wiring.rs` incl. the cited URL-twin `test_enrich3_sidecar_emits_global_key_for_enrichment`
- EC-007 recovery form documented in `resolve_configure_url` rustdoc/body (multi_org_cmd.rs lines 826/907/942) — story's disambiguation from its own §Edge Cases EC-007 (TLS) is correct
- Test helpers `common::single_clone_config` / `common::http_client` exist; dev-deps (`prism-dtu-crowdstrike`, `prism-dtu-threatintel`, `tempfile`, `libc`) present in Cargo.toml

## TD-VSDD-060 sibling sweep verification: PASS

- No public function signature changes; `start_instances` return type unchanged; `MultiInstanceServers` gained a private field — BOTH construction sites (empty-config early return + main bind loop) updated in the same diff
- Determinism sweep independently re-run: all 5 diagnostic key-list sites in the crate (`socket_map.keys()` ×2, `token_map.keys()` ×2, `url_map.keys()` ×1) are sorted before rendering; `bare_matches.sort_by` present in BOTH `resolve_configure_url` and `resolve_configure_token`; Tests J/F/K lock the sorted rendering as load-bearing
- Sweep-count reproduction on frozen HEAD: 447 / 131 / 6 / 8 — all match story v0.12 footnote and both code mirrors

## Conventions / policy compliance checks

| Check | Result |
|-------|--------|
| unwrap()/expect() in diff-added production paths | NONE (pre-existing `.expect()` sites in multi_org_cmd.rs clone-factory are outside diff scope); `unwrap_or("tokens")` fallback is non-panicking |
| POLICY 12 stub residue (`todo!`/`unimplemented!`) | NONE in crate src |
| `#[non_exhaustive]` discipline | No new pub types added; `MultiInstanceServers` already `#[non_exhaustive]`; gate count 92 untouched |
| reqwest rustls-tls + timeout | No new reqwest dep or client; existing `cmd_configure` client keeps 10s timeout (story-ratified crate-local exception) |
| `println!` | Only pre-existing CLI output formatting (ratified exception per story Architecture Compliance Rules) |
| AD-017 credential safety | Token values never logged (`token_present=true` only); sidecars 0600 on Unix; tokens ephemeral UUID v4 |
| POLICY 13 story↔index | STORY-INDEX row shows `draft v0.15` — agrees with story frontmatter `status: draft`, `version: "0.15"` |
| POLICY 16 inverted-polarity tests | None — Test A is a documented permanent server-contract lock (asserts server 401, not a stub panic) |
| POLICY 34 fail-loud enforcement | `write_multi_admin_token_sidecar_to_path` uses release-active `ok_or_else(...)?` (no `debug_assert!`); Tests F/K assert the Err path |
| Tautology / paper-fix scan (TD-VSDD-059) | Every test inventory row's "what revert breaks this" claim spot-verified against production code structure; 0600 assertions are umask-robust (`& 0o077`) and mutation-sensitive (0o644 → 0o044 ≠ 0); Test H's no-fallthrough lock genuinely poisons the fallthrough path by pre-loading the nested sidecar |
| .gitignore coverage | `.prism-dtu-demo-server.admin-tokens*.json` + `.tmp` variants + `pid.tmp` covered |

## KNOWN-ACCEPTED items encountered (not reported, per pre-adjudication)

1. 3 failing `bc_2_06_018_seeding` red-gate tests (observed in full-crate run)
2. `DEMO_ORG_UUID_B` clippy dead-code warning (observed in clippy run)
3. DRIFT-HARNESS-ADMIN-TOKEN-CT-001 server-side constant-time comparison (story explicitly out-of-scopes it; folded into S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001)
4. Pre-existing `.tmp` sidecar file-permissions edge on paths predating this branch (`OpenOptions::create(true)` reuses an existing stale tmp file's mode — adjudicated below demo-infra threat floor)
5. Missing `[[test]]` Cargo.toml entry pattern (crate uses explicit `[[test]]` sections with default `autotests = true` auto-discovery; matches tolerated develop-side pattern)

## Findings

**NONE.**

## Streak note

This pass is CLEAN (strict) on frozen HEAD `828449de`. Per BC-5.39.001 + the frozen-HEAD streak rule, it counts toward the 3-CLEAN streak only alongside other CLEAN(strict) passes taken against this same unchanged HEAD.
