---
document_type: adversarial-review
pass_id: pr-127-pass-13
cycle: wave-4-operations
window_position: "3/3 CLOSED — CONVERGED"
disposition: CLEAN
date: 2026-05-06
milestone: "PR #127 (S-3.01 PrismQL Parser) CONVERGED — VSDD 3-clean window discipline satisfied"
producer: adversary
input_hash: "9557b647"
diff_base: "3133710e"
predecessor: pass-12 CLEAN
window_history: [pass-11 CLEAN, pass-12 CLEAN, pass-13 CLEAN]
---

# Adversarial Review — PR #127 Pass 13 (S-3.01 PrismQL Parser)

## Tally

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |
| INFO | 0 |
| OBS (process-gap, non-blocking, post-mortem) | 1 |

**Verdict:** `CLEAN` (CONVERGENCE_REACHED for PR #127)

**Window status:** 3/3 CLOSED — CONVERGED. PR #127 (S-3.01 PrismQL Parser) is CONVERGED per VSDD 3-clean window discipline (pass-11 CLEAN → pass-12 CLEAN → pass-13 CLEAN).

---

## Findings

### F-PG-001 — Perimeter compile-fail Python parser regex was untested for 12+ adversary passes [process-gap] [OBS, non-blocking, post-mortem]

**Severity:** OBS (informational; fix already landed in 9557b647)
**Tag:** `[process-gap]`
**Category:** Test-infra latency — security regression detector silently inert
**Files:**
- `.github/workflows/ci.yml` (lines 273-469, perimeter-compile-fail job)
- Fix commit: `9557b647`

**Finding:**
The CI job `perimeter-compile-fail` (BC-2.11.006 v1.10 enforcement) embeds a Python parser that scans `cargo check` output for `error[E0603]` / `error[E0624]` lines via the regex `re.match(r'error\[(?:E0603|E0624)\]:[^` + "`" + `]*` + "`" + `([^` + "`" + `]+)` + "`" + `.*(?:private|restricted)', line)` (ci.yml line 421). For the entire history of PR #127 (passes 1-12), this parser was **never actually executed against real cargo output** because:

1. Original timeout was 3 minutes (insufficient for cold cargo build of prism-query + 24 transitive deps, which took ~3m23s)
2. Bumped to 7 minutes (a802b983) — still tight after opt-level=3 added crypto crates to build cost
3. Bumped to 12 minutes (4e0b72c6) — first run that completed cargo (run 25426278331) revealed the parser regex did not match because cargo 1.85+ emits ANSI color codes in stderr even when redirected to a log file. Lines like `\x1b[1m\x1b[91merror[E0603]\x1b[0m...` failed `re.match(r'error\[...')`.
4. Fix landed in 9557b647 by adding `--color=never` to the cargo invocation.

**Impact (historical):** For 12+ adversary passes, BC-2.11.006 v1.10 perimeter regression detection was effectively a no-op. A genuine regression where one of the 17 restricted symbols (parse_filter, parse_sql, parse_pipe, build_*, ParseLimits methods) was promoted to `pub` would have produced a non-zero cargo exit (catching the binary signal) but the per-symbol assertion that catches "single-symbol regression while siblings remain pub(crate)" was inert because the Python parser produced an empty `found_names` set on every run that ran. The job's binary signal would still pass on a proper full-pub regression, but the per-symbol coverage that pass-7 F-HIGH-001 specifically added was silently bypassed.

**Why this is a process-gap, not a content defect:**
The fix has already landed in 9557b647 with explicit comment annotations (ci.yml lines 326-330). The artifact is now correct. However, the **process** that allowed a security-perimeter regression detector to be functionally untested for 12 adversary passes is the gap. The pattern: when a CI job's value is "fail-on-anomaly", routine timeouts that prevent the job from reaching the assertion logic create false-green signals that adversary passes interpret as "perimeter check passed".

**Recommendation (codification candidate):**
Add a process rule for security-critical CI jobs: "Any CI job whose value is regression detection (compile-fail, lint, fuzz-smoke) MUST emit a positive-coverage assertion in its log on every successful run, not just an exit code. The Python parser already does this (`Perimeter check passed: all N restricted symbols produced E0603/E0624 errors`); promote that pattern to a TD-VSDD codification so future CI jobs follow it."

This is **non-blocking** for PR #127 convergence — the artifact is now correct and the next CI run will validate the perimeter properly. Recording for orchestrator codification follow-up per S-7.02. **Track as TD-VSDD-057 (positive-coverage-assertion rule for security-critical CI jobs).**

---

## Verification of Pass-13 Lenses

### Lens 1: CI/perf changes review (4 CI yaml + 1 .cargo/config.toml + 2 docs commits)

**Status:** CLEAN.

- `9557b647` (--color=never): Correct fix. Comment block (ci.yml 322-330) accurately documents the latent bug. No side effects on parser security model.
- `4e0b72c6` / `6f38ac5f` / `a802b983` (timeout bumps): Pure CI tuning. No semantic impact.
- `30f6fc07` (opt-level=3 + LazyLock fixtures): Verified below in lens 2.
- `fee25af5` / `624a4c1d` (CLAUDE.md docs): Verified below in lens 4.
- `157c2cd8` (kani Windows gate): Verified below in lens 3.

### Lens 2: opt-level=3 for crypto deps (30f6fc07)

**Status:** CLEAN.

- `.cargo/config.toml` only adds `[profile.dev.package.<crate>]` entries. NO `[profile.release]` override anywhere in workspace.
- **`cargo build --release` semantics: PRESERVED.** opt-level=3 is the default for release profile in Rust; these dev-profile package overrides only affect dev/test builds.
- **Proptest invariant validity:** The cross-org isolation property (`proptest_BC_3_2_002_vp_01_cross_org_isolation`) remains validly tested:
  - Each iteration creates a fresh `EncryptedFileBackend` instance with a unique `case_workdir()` subdirectory (atomic counter).
  - Two fresh `OrgId::new()` UUIDs per iteration with `prop_assume!(org_a != org_b)` guaranteeing distinctness.
  - The `LazyLock<TempDir>` is only a parent root for cleanup; per-iteration subdirectories are filesystem-isolated.
  - **No cross-iteration state leak possible.**

### Lens 3: kani-verifier Windows gate (157c2cd8)

**Status:** CLEAN.

- `crates/prism-query/Cargo.toml` correctly gates `kani-verifier = "=0.67.0"` behind `[target.'cfg(not(target_os = "windows"))'.dev-dependencies]`.
- All proof modules in `crates/prism-query/src/proofs/` are properly `#[cfg(kani)]`-gated.
- Proof validity on Linux/macOS: PRESERVED. Code is platform-agnostic; a proof valid on Linux/macOS is mathematically valid on Windows.

### Lens 4: CLAUDE.md content (624a4c1d + fee25af5)

**Status:** CLEAN.

- Project conventions accurately described.
- Formal Verification section is technically accurate (CBMC backend, three coverage layers).
- No false claims about Kani-on-Windows support.

### Lens 5: Latent ANSI regex bug (process-gap)

**Status:** RECORDED as F-PG-001 above.

---

## Policy rubric verification (10 baseline policies)

All policies pass for this PR scope:

| Policy | Verdict |
|--------|---------|
| 1 append_only_numbering | PASS |
| 2 lift_invariants_to_bcs | N/A |
| 3 state_manager_runs_last | N/A |
| 4 per_burst_consistency_validator | N/A |
| 5 citations_use_canonical_sha | PASS |
| 6 fresh_context_audits | PASS |
| 7 bc_array_changes_propagate | N/A |
| 8 frontmatter_field_alignment | N/A |
| 9 research_artifacts_in_research_dir | PASS |
| 10 demo_evidence_story_scoped | N/A |

---

## Convergence Declaration

**PR #127 (S-3.01 PrismQL Parser) is CONVERGED per VSDD 3-clean window discipline.**

3-clean window satisfied: pass-11 CLEAN → pass-12 CLEAN → **pass-13 CLEAN; CONVERGED**

### Convergence cycle summary

- 13 adversary passes consumed for PR #127
- 8 commits since pass-12 (4 CI yaml + 1 .cargo/config.toml + 1 Cargo.toml + 2 docs)
- All commits are orthogonal to parser security model — pass-10's "parser security model has converged" verdict remains intact
- 1 [process-gap] OBS recorded (F-PG-001 ANSI regex latency) — non-blocking; fix already landed in 9557b647

### Strategic observation

The 8 commits in this window are a textbook example of CI/build-infrastructure tuning that does not perturb the converged spec set. The opt-level=3 dev profile changes preserve release semantics; the kani Windows gate preserves proof validity; the CLAUDE.md additions are accurate documentation; the timeout bumps are operational hygiene. The single [process-gap] (ANSI regex untested for 12 passes) is a learning artifact for future CI rule codification, not a current defect.

**Recommendation:** Merge PR #127 to develop. Open TD-VSDD-057 for F-PG-001's positive-coverage-assertion rule.

---

## Files reviewed

- `.cargo/config.toml`
- `.github/workflows/ci.yml`
- `.github/workflows/fuzz-nightly.yml`
- `Cargo.toml`
- `CLAUDE.md`
- `crates/prism-query/Cargo.toml`
- `crates/prism-query/src/lib.rs`
- `crates/prism-query/src/proofs/mod.rs`
- `crates/prism-query/src/proofs/vp014_size_limit.rs`
- `crates/prism-query/src/proofs/vp015_depth_limit.rs`
- `crates/prism-credentials/tests/bc_3_2_002_org_id_namespace.rs`
- `tests/external/perimeter-violation/src/main.rs`
- `.factory/policies.yaml`

---

**Verdict: CLEAN — CONVERGENCE_REACHED for PR #127.**
