# Review Findings — hotfix-fuzz-rustup-kani-arbitrary

## Convergence Table

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1     | 0        | 0        | 0     | 0 → APPROVE |

## Verdict

**APPROVE** — Converged in 1 cycle. No blocking findings.

## Review Notes (Cycle 1)

**YAML correctness:** `env: RUSTUP_TOOLCHAIN: nightly` placed at job level (before `steps:`) is correct GitHub Actions YAML. Job-level env vars take precedence over workflow-level and are visible to all steps. The `toolchain: nightly` addition to `dtolnay/rust-toolchain` `with:` block is belt-and-suspenders correct.

**cfg_attr correctness:** `#[cfg_attr(kani, derive(kani::Arbitrary))]` is correctly placed between the `#[derive(...)]` and the `pub enum CaseStatus` declaration. The derive is emitted only when `cfg(kani)` is set (i.e., only during `cargo kani` invocations), making it a true no-op for all production builds. `cfg(kani)` is already declared as a known cfg in the workspace `[lints.rust]`.

**RUSTUP_TOOLCHAIN vs. alternatives:** The env var approach is the canonical rustup override mechanism, taking precedence over `rust-toolchain[.toml]` per rustup specification. This is the minimal, correct fix. Alternatives (per-directory rust-toolchain.toml in fuzz/, cargo +nightly) are more invasive. No concerns.

## Post-Merge Findings

### PMV-001 — Kani HTTP 502 (transient)
- **Severity:** TRANSIENT — not a code issue
- **Run:** 24926516444
- **Detail:** Kani setup downloaded `kani-0.67.0` tarball from GitHub releases and received HTTP 502. CDN hiccup. `RUSTUP_TOOLCHAIN: nightly` env var was correctly set (visible in logs). No code change needed.
- **Recommendation:** Re-run the Post-Merge Verification workflow manually once CDN recovers.

### PMV-002 — Fuzz target name mismatch (pre-existing)
- **Severity:** BLOCKING (for Post-Merge Verification)
- **Run:** 24926516444
- **Detail:** The workflow runs `cargo fuzz run fuzz_prismql_parser`, `fuzz_alias_expansion`, `fuzz_normalize`, `fuzz_spec_parser`, `fuzz_template_interpolation`, `fuzz_injection_scanner`. But `fuzz/Cargo.toml` only defines bins: `fuzz_injection_scanner`, `spec_parser`, `normalize_fuzz`. The other 5 target names are not defined. This failure was previously masked because fuzz failed at the toolchain level (before reaching target discovery). Now that RUSTUP_TOOLCHAIN is fixed, this pre-existing mismatch is exposed.
- **Recommendation:** Hotfix #3 — align `fuzz/Cargo.toml` [[bin]] entries with `post-merge.yml` fuzz step names, OR update `post-merge.yml` to use the actual target names that exist.
