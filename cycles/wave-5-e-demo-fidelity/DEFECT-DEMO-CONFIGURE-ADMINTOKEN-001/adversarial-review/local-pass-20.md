# DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 — LOCAL Adversarial Pass 20

- **Reviewer:** fresh-context adversary (TD-VSDD-005; no prior-pass knowledge, no `adversarial-review/` reads)
- **Date:** 2026-07-17
- **Worktree:** `/Users/jmagady/Dev/prism/.worktrees/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001`
- **Branch:** `fix/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001`

## Frozen-HEAD Verification

| Check | Expected | Observed | Result |
|-------|----------|----------|--------|
| `git rev-parse --short HEAD` | `828449de` | `828449de` | PASS |
| `git status --porcelain` | empty | empty | PASS |
| Story frontmatter version | v0.15 | `version: "0.15"` | PASS |
| BC-2.06.017 frontmatter version | v1.12 | `version: "1.12"` | PASS |
| BC-3.6.001 version (story pin v0.8) | v0.8 | `version: "0.8"` | PASS |

Diff surface reviewed: `git diff 84062ced...HEAD` — 10 files, +2,224 / −22 (`.gitignore`, `prism-bin/tests/helpers/mod.rs`, `prism-dtu-demo-server/{README.md, src/harness.rs, src/lib.rs, src/main.rs, src/multi_instance.rs, src/multi_org_cmd.rs, tests/defect_demo_configure_admintoken_001.rs, tests/td_wv1_04_binary_tls_e2e.rs}`).

## Review Angle (fresh-eyes)

I approached this pass as an end-to-end operator-path audit: (1) can the fix actually be reverted piecewise without any test failing (paper-fix probe, TD-VSDD-059); (2) do the SWEEP-MIRROR reproducible counts actually reproduce at this HEAD; (3) does every quoted spec anchor exist byte-for-byte in its source-of-truth artifact (POL-22 Phase A); (4) does every named code entity exist (POL-22 Phase C); (5) do the tests actually run GREEN at this HEAD, including the feature-gated Test G; (6) CLAUDE.md conventions sweep on the production diff.

## AC Satisfaction Audit

| AC | Verdict | Evidence |
|----|---------|----------|
| AC-001 | SATISFIED | Test A (contract lock, POST w/o header → 401, `{"auth_mode": "accept"}` payload matches AC literal); Test E binary E2E (exit 0 with sidecar token; load-bearing against T-08 revert). Red-Gate obligation carried by Test E + sidecar-existence assertions per the v0.7 contract-lock adjudication in the story itself. |
| AC-002 | SATISFIED | `resolve_configure_token` (flat-first, nested fallback, bare-name disambiguation) wired into `cmd_configure` with `.header("X-Admin-Token", &admin_token)` (main.rs line 688). `write_token_sidecar` called in `cmd_start` after `write_url_sidecar`; `write_multi_admin_token_sidecar` called in `cmd_start_multi`; `MultiInstanceServers::admin_token_map()` populated BEFORE clone move (OWNERSHIP constraint honored in `start_instances` bind loop — both construction sites of `MultiInstanceServers` updated: multi_instance.rs empty-config arm and main arm). `_global` enrichment arm present with fail-loud `ok_or_else` and locked by Test K (a)–(d). T-09 cleanup in both shutdown handlers, locked by Test E/Test G persistence assertions. `TOKEN_FILE`/`TOKEN_MULTI_FILE` `pub const` in lib.rs. |
| AC-003 | SATISFIED | E-DEMO-007 template in `resolve_configure_token` byte-matches story §Error Taxonomy Addition AND error-taxonomy.md row (line 615, v2.54 changelog row present). EC-003 (Tests H, I), EC-004 (Test C), EC-005 (Test D) all exercised; no-POST-on-miss guaranteed by `?` before client build in `cmd_configure`; no panic path (anyhow propagation). |
| AC-004 | SATISFIED | SWEEP-MIRROR blocks present in `cmd_configure` (main.rs) and defect test module header; all four reproducible commands re-run at frozen HEAD and match exactly (see below). Test A doc-comment cross-references the main.rs sweep block as required. |

## SWEEP-MIRROR Count Reproduction (AC-004 / TD-VSDD-060)

Run at frozen HEAD from worktree root:

| Command | Claimed | Observed | Result |
|---------|---------|----------|--------|
| `rg 'dtu/configure' crates/ --type rust \| grep -v SWEEP-MIRROR \| wc -l` | 447 | 447 | PASS |
| `rg '\.post\(.*dtu/configure' crates/ --type rust \| grep -v SWEEP-MIRROR \| wc -l` | 131 | 131 | PASS |
| `rg 'let url = format.*dtu/configure' crates/ --type rust \| grep -v SWEEP-MIRROR \| wc -l` | 6 | 6 | PASS |
| `rg 'endpoint.*"/dtu/configure"' crates/ --type rust \| grep -v SWEEP-MIRROR \| wc -l` | 8 | 8 | PASS |

Dynamic-caller reconciliation independently re-derived: the 6 `let url = format` lines are exactly the sites named in the mirror (2× `ac_3_configure_endpoint.rs`, 2× defect tests A/B, `builder.rs` hung-socket, `harness.rs::inject_failure`) + `cmd_configure` via `resolve_configure_url` = 7 dynamic. `multi_org.rs` callers of `resolve_configure_url` (lines 815/830) resolve URLs but never POST to them — correctly excluded. 131 + 7 + 8 = 146 holds.

## Test Execution at Frozen HEAD

- `cargo nextest run -p prism-dtu-demo-server -E 'binary(defect_demo_configure_admintoken_001)'` → **10/10 PASS** (Tests A, B, C, D, E, F, H, I, J, K).
- `cargo nextest run -p prism-dtu-demo-server --features fixture-gen -E 'test(test_BC_2_06_017_ac002_binary_startmulti_configure_with_multi_sidecar_token)'` → **1/1 PASS** (Test G).
- `cargo clippy -p prism-dtu-demo-server --all-targets` → only the known-accepted `DEMO_ORG_UUID_B` dead-code warning (KNOWN-ACCEPTED #2).

## SAP-1 — Tracing Emission Catalog Completeness

- `git diff 84062ced...HEAD | grep event_type` → **zero** `event_type =` emissions added or modified by this branch.
- The single new tracing site (`tracing::debug!(clone = %clone_name, token_present = true, ...)` in `cmd_configure`) carries NO `event_type` field, so no BC-2.16.002 catalog row is required; it also honors AD-017 (token value never logged — `token_present=true` placeholder only, exactly as the story's Architecture Compliance Rules mandate).
- Workspace census: `rg 'event_type\s*=' crates/ --type rust` → 230 hits, all pre-existing on develop baseline (none in the diff). SAP-1: **PASS** for this branch scope.

## SAP-2 — DTU↔TOML Schema Parity

No `.prism/specs/sensors/*.toml` or sensor TOML specs touched by this diff. **N/A.**

## POL-22 Phase A — Lexical-vs-Semantic Anchor Verification

| Citation | Source | Result |
|----------|--------|--------|
| Story §Root Cause block-quote of ADR-003 Amendment #5 §Decision ("`POST /dtu/configure` on every DTU clone MUST require a valid `X-Admin-Token` header... HTTP 401 with `{"error": "missing or invalid X-Admin-Token"}`") | ADR-003 lines 626–632 | VERBATIM MATCH |
| Story §Root Cause block-quote of Amendment #5 §Implementation item 4 ("All 12 existing `td_wv0_04` configure tests (2 per clone) and all other integration tests calling `/dtu/configure`: updated to include `.header("X-Admin-Token", clone.admin_token())`") | ADR-003 lines 663–666 | VERBATIM MATCH; item-4 scope (integration tests only, hence cmd_configure uncovered) semantically correct |
| BC-3.6.001 Precondition 4 quote ("The `inject_failure` call uses `POST /dtu/configure` on the clone's admin endpoint, authenticated with that clone's `admin_token` (ADR-003 Amendment §5).") | BC-3.6.001 lines 73–74 | VERBATIM MATCH |
| BC-2.06.017 Postcondition 1 pin (admin_token_map accessor, before-move extraction, TOKEN_MULTI_FILE nested format, atomic tmp+rename cf. GAP-3 sidecar-poll note) | BC-2.06.017 §Postcondition 1 (v1.12) | SEMANTIC MATCH — enumerates `admin_token_map() -> &HashMap<String, String>`, before-spawn extraction, and nested sidecar format exactly as implemented |
| E-DEMO-007 message template (story AC-003, error-taxonomy.md line 615, `resolve_configure_token` `e_demo_007` closure) | all three artifacts | BYTE-IDENTICAL across story ↔ taxonomy ↔ code (POL-24) |
| EC-005 ambiguity template (`{:?}`-quoted sorted org list) | `resolve_configure_token` ambiguity arm + Test D assertion `["org-a", "org-b"]` | MATCH |
| BC-2.06.017 changelog v1.12 top row dated 2026-07-17 = `modified:` field; table monotonic-descending | BC file | PASS (POL-27, POL-32) |

## POL-22 Phase C — Named-Entity Existence Verification

All load-bearing named entities verified present at frozen HEAD: `write_token_sidecar_to_path` (harness.rs, re-exported via lib.rs), `DemoHarness::token_map` (harness.rs), `ClonePair::admin_token` (harness.rs line 58), `MultiInstanceServers::admin_token_map` + private `token_map` field (multi_instance.rs), `write_multi_admin_token_sidecar_to_path` and `resolve_configure_token` (multi_org_cmd.rs, re-exported via lib.rs), `write_token_sidecar` / `write_multi_admin_token_sidecar` binary wrappers (main.rs), `TOKEN_FILE` / `TOKEN_MULTI_FILE` `pub const` (lib.rs), `KNOWN_ENRICHMENT_CLONES` (config), URL twin `test_enrich3_sidecar_emits_global_key_for_enrichment` (enrich_23_dtu_wiring.rs line 178), all 11 test function names in the story/test-inventory tables, `common::single_clone_config` / `common::http_client` (tests/common/mod.rs). E-DEMO-007 registered in error-taxonomy.md v2.54. **PASS.**

## CLAUDE.md Conventions Sweep (production diff)

- No `unwrap()`/`expect()` on `Result` in production paths (only `unwrap_or("tokens")` filename fallback — infallible default, acceptable).
- No new `reqwest` client; existing `cmd_configure` 10s timeout retained per the story's ratified crate-local exception; rustls-tls dependency entry untouched.
- `println!` confined to CLI output formatting (ratified exception).
- No new pub structs/enums → `#[non_exhaustive]` gate unaffected (`MultiInstanceServers` already `#[non_exhaustive]`; gained only a private field). EXPECTED=92 unaffected.
- AD-017: token values never logged; sidecars written 0600 on Unix, atomically (tmp+rename); permissions locked by umask-robust `mode() & 0o077 == 0` assertions in Tests B/F/K.
- Fail-loud discipline: `write_multi_admin_token_sidecar_to_path` errors loudly (sorted deterministic key lists) on any missing org or enrichment token — no silent skip (Standing Rule 3 §2); locked by Test F and Test K (d).
- POLICY 12/16: no stub residue in the diff; Test A asserts real server 401 contract, not stub-panic polarity.
- POLICY 33: no StageMask-relevant DTU route handler changes. N/A.
- `.gitignore` sibling sweep covers all new sidecar + `.tmp` variants including the previously-unignored `.pid.tmp`.
- README.md file table verified against code: PID/URL/token sidecars all tmp+rename atomic (`write_pid_file` main.rs lines 303–306) and all removed on shutdown (T-09).
- `prism-bin/tests/helpers/mod.rs` doc-comment cleanup verified accurate: `configure_cyberint_dtu_access_token` no longer exists anywhere in `crates/` — stale reference correctly removed.
- `td_wv1_04_binary_tls_e2e.rs` KillGuard additions (TD-VSDD-060 sibling sweep of the subprocess-spawn pattern): correct RAII + disarm-after-wait pattern; SIGKILL-to-ESRCH benign.

## KNOWN-ACCEPTED items honored (not reported)

1. 3 failing `bc_2_06_018_seeding` Red Gate tests — not run/reported. 2. `DEMO_ORG_UUID_B` clippy warning — observed, excluded. 3. DRIFT-HARNESS-ADMIN-TOKEN-CT-001 — out of scope. 4. Pre-existing `.tmp` sidecar permissions edge — excluded. 5. Missing `[[test]]` entry for `defect_demo_configure_admintoken_001.rs` (auto-discovered; `dtu` is default feature) — matches tolerated develop-side pattern, excluded.

## Findings

**ZERO FINDINGS.**

No CRIT, HIGH, MED, LOW, OBS, or PROCESS-GAP findings at frozen HEAD 828449de.

## Verdict

```
CLEAN (strict): yes
CLEAN (PR-merge): yes
```
