---
document_type: library-version-research
level: L3
version: "1.0"
producer: research-agent
timestamp: 2026-05-04T00:00:00Z
status: draft
---

# W3 Library Version Research

Reference data for Wave 3 implementation. All versions verified against crates.io API endpoints (`https://crates.io/api/v1/crates/<name>`) on 2026-05-04 unless flagged UNVERIFIED.

## Summary Table

| Library | Latest Stable | Pre-release | MSRV (rustc) | crates.io | Last Release |
|---|---|---|---|---|---|
| chumsky | **0.12.0** | 1.0.0-alpha.8 | 1.65 | https://crates.io/crates/chumsky | 2025-12-15 |
| datafusion | **53.1.0** | none current | 1.85 (via arrow) | https://crates.io/crates/datafusion | 2026-04-16 |
| arrow | **58.2.0** | none current | 1.85 | https://crates.io/crates/arrow | 2026-05-02 |
| rocksdb (`rust-rocksdb/rocksdb` crate) | **0.24.0** | none current | 1.85 | https://crates.io/crates/rocksdb | 2025-08-10 |
| moka | **0.12.15** | none current | 1.71.1 | https://crates.io/crates/moka | 2026-03-22 |
| kani-verifier | **0.67.0** | none current | nightly (Kani-bundled) | https://crates.io/crates/kani-verifier | 2026-01-16 |
| proptest | **1.11.0** | none current | 1.85 | https://crates.io/crates/proptest | 2026-03-24 |
| cargo-mutants | **27.0.0** | none current | 1.88 | https://crates.io/crates/cargo-mutants | 2026-03-07 |
| serde | **1.0.228** | none current | 1.56 | https://crates.io/crates/serde | 2025-09-27 |
| regex | **1.12.3** | none current | 1.65 | https://crates.io/crates/regex | 2026-02-03 |
| uuid | **1.23.1** | none current | (per features) | https://crates.io/crates/uuid | 2026-04-16 |
| tracing | **0.1.44** | none current | 1.65 | https://crates.io/crates/tracing | 2025-12-18 |
| tracing-subscriber | **0.3.23** | none current | 1.65 | https://crates.io/crates/tracing-subscriber | 2026-03-13 |

> **Note on Kani toolchain:** kani-verifier installs a separate Rust toolchain via `cargo kani setup`; the user-facing rustc requirement for the *installer crate* is Rust 1.58+, but actual proof runs use the nightly snapshot bundled by Kani. Source: `model-checking.github.io/kani/install-guide.html`.

## Per-Library Deep Dive

### 1. chumsky
- **State:** 0.12.0 stable (2025-12-15). 0.11.0 was **yanked**; 0.11.1/0.11.2 superseded by 0.12. 1.0.0-alpha.8 (Jan 2025) is the most recent alpha and is **stale** relative to 0.12 — the maintainer has pivoted the 0.x line away from being a stop-gap and 0.12 contains the work previously alpha-tagged.
- **API stability:** BC between 0.9 → 0.10 was a from-scratch rewrite (`then_with` removed; combinators consolidated). 0.11 → 0.12 contains breaking changes per release notes; `Rich`/`Simple` error types remain in `chumsky::error`; `delimited_by`, `then_ignore`, `recover_with`, `nested_delimiters`, `via_parser`, `skip_then_retry_until` are present in 0.12 (Context7 examples confirm).
- **Recommendation:** **`chumsky = "0.12"`** (pin minor with `^0.12`).
- **Rationale:** 0.12.0 is the current stable; 0.10.x is two minors behind and missing a year of fixes; 1.0.0-alpha.8 has been frozen for ~10 months, indicating the alpha line is no longer the active migration target. Use `Rich<'a, char>` (or `Rich<'a, Token>` for token-level) per the Context7 examples.

### 2. datafusion
- **State:** 53.1.0 (2026-04-16) is the current stable on crates.io. 53.0.0 shipped 2026-03-23. 54.0.0 was scheduled per GitHub issue #21080 for Apr/May 2026 but **had not been published to crates.io as of 2026-05-04**.
- **API stability:** Major-version cadence (~6 weeks). Each major contains BCs. Per `datafusion.apache.org/user-guide/crate-configuration.html`, plan to track within one major behind unless a feature is required.
- **Recommendation:** **`datafusion = "53.1"`** (pin minor; allow patches).
- **Rationale:** 53.1 is the most recent shipped stable; 54 not yet on crates.io. Avoid pinning to exact `=53.1.0` so patch fixes flow through.

### 3. arrow (apache/arrow-rs)
- **State:** 58.2.0 (2026-05-02) is the current stable. 58.1.0 (2026-03-24), 58.0.0 (2026-02-23). Release issue notes a 59.0.0 planned for May 2026 with potentially BC API changes.
- **API stability:** Major-bumped frequently; BC between majors is normal. DataFusion 53.x depends on a specific arrow major — verify dep tree alignment when pinning both.
- **Recommendation:** Let DataFusion transitive-pin arrow; if directly depending: **`arrow = "58"`** matching the version DataFusion 53.1 requires (verify via `cargo tree`).
- **Rationale:** Direct pinning of arrow alongside DataFusion creates upgrade friction; DataFusion's Cargo.toml dictates the compatible arrow major.

### 4. rocksdb (Rust binding)
- **State:** The crate **`rocksdb`** (NOT `rust-rocksdb`) is the canonical published name; latest is 0.24.0 (2025-08-10). Maintained by `rust-rocksdb/rust-rocksdb` GitHub org.
- **API stability:** 0.24 was a moderate release; bundled RocksDB C++ ABI matches what the crate vendors at compile time (not separately versioned for users).
- **Recommendation:** **`rocksdb = "0.24"`**.
- **Rationale:** Current stable; no newer minor on crates.io as of 2026-05-04. MSRV 1.85 aligns with arrow/datafusion — no separate toolchain bump.
- **Note on the `zaidoon1/rust-rocksdb` fork:** A separate fork exists publishing a different crate name. Stick with the canonical `rocksdb` crate unless a specific feature is missing.

### 5. moka
- **State:** 0.12.15 (2026-03-22). The 0.12.x line has been the active stable line since late 2023.
- **API stability:** Patch-stable; no BC announced for 1.0 imminent. `moka::sync::Cache` and `moka::future::Cache` are the two top-level APIs.
- **Recommendation:** **`moka = { version = "0.12", features = ["future"] }`** (or `"sync"` per use case).
- **Rationale:** Mature, actively maintained, MSRV 1.71.1 well below project's other deps.

### 6. kani-verifier
- **State:** 0.67.0 (2026-01-16). Bi-monthly release cadence (0.65 → 0.66 → 0.67 over 2025-08 → 2025-11 → 2026-01).
- **Toolchain implications:** The `kani-verifier` *installer* compiles on stable Rust 1.58+, but `cargo kani setup` downloads a Kani-bundled nightly snapshot used for actual proof runs. Project rustc-toolchain is **not** affected — Kani is invoked out-of-band via `cargo kani`.
- **Install procedure** (verified at `model-checking.github.io/kani/install-guide.html`): `cargo install --locked kani-verifier && cargo kani setup`.
- **Recommendation for CI gate:** Pin **`kani-verifier = "=0.67.0"`** in dev-dependencies *or* an installer script. Document the `cargo kani setup` step in W3 CI bootstrap.
- **Rationale:** Kani's own version bump may require Kani harness syntax changes; exact-pin avoids surprise breakage in S-3.01 AC-10 gates. Re-pin deliberately.

### 7. proptest
- **State:** 1.11.0 (2026-03-24). 1.10.0 (2026-02-05), 1.9.0 (2025-10-26). 1.x line is API-stable.
- **API stability:** Patch-stable within 1.x; new combinators added monotonically.
- **Recommendation:** **`proptest = "1.11"`**.
- **Rationale:** Current; MSRV 1.85 matches workspace.

### 8. cargo-mutants
- **State:** 27.0.0 (2026-03-07). Versioning is YY.N — 27.0 = 2027 series start? (No; project uses non-CalVer integer majors despite the look. Recent: 26.x in early 2026, 27.0.0 shipped March.)
- **Toolchain implications:** **MSRV 1.88** — this is the *highest* of any tool in the W3 set. Confirm the workspace `rust-toolchain.toml` (if pinned) is at least 1.88; otherwise stay on 26.x line.
- **Recommendation:** **`cargo-mutants = "27.0"`** *if* workspace MSRV >= 1.88; else **`= "26.2"`** (2026-02-01) which has lower MSRV.
- **Rationale:** 27.0 likely contains improvements but may not be worth a workspace MSRV bump just for a dev tool. Verify current `mutants.out/` was produced by what version (check for `cargo-mutants.json` or release notes).

### 9. serde
- **State:** 1.0.228 (2025-09-27). Minor cadence is rapid (1.0.225 → 1.0.226 → 1.0.227 → 1.0.228 in two weeks).
- **API stability:** 1.0 line is canonical-stable; patches are forward-compatible.
- **Recommendation:** **`serde = { version = "1", features = ["derive"] }`**.
- **Rationale:** Caret-1 idiomatic; 1.0.228 is current floor.

### 10. regex
- **State:** 1.12.3 (2026-02-03). 1.12.2 (Oct 2025).
- **API stability:** 1.x stable; MSRV bumps allowed in minors per the crate's policy (currently 1.65).
- **Recommendation:** **`regex = "1.12"`**.
- **Rationale:** Current; well below project MSRV.

### 11. uuid
- **State:** 1.23.1 (2026-04-16). v7 generation is **stable** via the `v7` feature and has been since 1.10 (Sept 2024 timeframe). UUIDv7 conforms to RFC 9562.
- **API:** `Uuid::now_v7()` requires `v7` feature; ensure `+ rand` or `+ getrandom` per env.
- **Recommendation:** **`uuid = { version = "1.23", features = ["v7", "serde"] }`** for W4 idempotency_key model.
- **Rationale:** v7 support is stable and current; 1.23.1 fixes go through patch increments.

### 12. tracing + tracing-subscriber
- **State:** `tracing` 0.1.44 (2025-12-18). `tracing-subscriber` 0.3.23 (2026-03-13). 0.1.42 of `tracing` was **yanked** — do not pin it. 0.3.21 of `tracing-subscriber` was **yanked**.
- **API stability:** Both 0.1.x and 0.3.x lines have been stable for years.
- **Recommendation:** **`tracing = "0.1.44"`**, **`tracing-subscriber = "0.3.23"`** with features `["env-filter", "fmt", "json"]` per project needs.
- **Rationale:** Avoid yanked versions; pin floor at the latest non-yanked.

## Critical Version Conflicts to Resolve in Story Specs

### Chumsky 0.10 vs 0.11 vs 0.12 vs 1.0-alpha — VERDICT: **0.12**
- **0.10** (S-3.01 may reference this): two minors stale; 13 months old; missing accumulated bug fixes.
- **0.11**: 0.11.0 was **yanked**; 0.11.1/0.11.2 are superseded by 0.12 within ~3 months. No reason to land here.
- **1.0.0-alpha.8**: Stale since Jan 2025 (~16 months). The maintainer's "Tracking: Stable 1.0" issue (#543) is not converging; alpha line is effectively dormant relative to 0.12's continued development.
- **0.12.0** (recommended): Current stable; same conceptual API surface as 0.11 (the rewrite from 0.10) with continued refinement; `Rich` errors and `extra::Err<Rich<...>>` pattern is canonical.

**Action for story-writer:** Update S-3.01 through S-3.13 to specify `chumsky = "0.12"`. Replace any `0.10` references. Replace any `1.0.0-alpha.x` references. Keep the `Rich` error type as the carrier (not `Simple`) — it preserves multi-error reporting needed for PrismQL diagnostics.

### DataFusion vs Arrow alignment
- DataFusion 53.1 transitively pins a specific arrow major. Do **not** independently pin `arrow` to a different major (e.g., 58 vs 57) without verifying DataFusion 53.1's `Cargo.toml`. Run `cargo tree -p datafusion | grep arrow` after first build to confirm.

### kani-verifier pinning policy
- Story specs should pin `kani-verifier` **exactly** (`=0.67.0`) because Kani's harness language and proof attribute syntax change across minors. Loose pins create silent CI failures.

## Toolchain Implications

| Item | rustc Required | Notes |
|---|---|---|
| arrow 58.x | **1.85** | Floor for the W3 query stack |
| datafusion 53.1 | **1.85** (via arrow) | Same |
| rocksdb 0.24 | **1.85** | Same |
| proptest 1.11 | **1.85** | Same |
| cargo-mutants 27.0 | **1.88** | **Highest** in set; dev-tool only |
| Kani 0.67 (proof runs) | nightly (bundled) | Out-of-band via `cargo kani` — does not affect workspace `rust-toolchain.toml` |
| Everything else | <= 1.71 | Below floor |

**Conclusion:** Workspace MSRV must be at minimum **1.85** for production code. If `cargo-mutants 27.0` is desired in CI, bump CI runner toolchain to **1.88** OR pin `cargo-mutants = "26.2"` for the lower MSRV. No nightly is needed for production builds; Kani is the only nightly-touching tool and runs out-of-band.

## Sources

- crates.io API: `https://crates.io/api/v1/crates/<name>` for each library on 2026-05-04
- chumsky releases: https://github.com/zesterer/chumsky/releases
- chumsky 1.0 tracking: https://github.com/zesterer/chumsky/issues/543
- chumsky 0.10 discussion: https://github.com/zesterer/chumsky/discussions/743
- DataFusion 54 release issue: https://github.com/apache/datafusion/issues/21080
- DataFusion crate config docs: https://datafusion.apache.org/user-guide/crate-configuration.html
- arrow-rs: https://github.com/apache/arrow-rs
- Kani install guide: https://model-checking.github.io/kani/install-guide.html
- Kani GitHub: https://github.com/model-checking/kani
- cargo-mutants changelog: https://mutants.rs/changelog.html
- moka GitHub: https://github.com/moka-rs/moka
- proptest book: https://altsysrq.github.io/proptest-book/
- regex GitHub: https://github.com/rust-lang/regex
- uuid GitHub: https://github.com/uuid-rs/uuid
- tracing GitHub: https://github.com/tokio-rs/tracing
- Context7 chumsky docs: `/zesterer/chumsky` (verified `Rich`, `delimited_by`, `recover_with`, `via_parser`, `nested_delimiters` API surface in 0.12-era examples)

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity search | 0 | (unavailable in this environment; substituted with WebSearch + crates.io API) |
| Perplexity deep_research | 0 | n/a |
| Perplexity reason | 0 | n/a |
| Context7 | 7 resolve + 1 query-docs | chumsky API verification, library ID resolution for 12 libs |
| Tavily | 0 | n/a |
| WebFetch | 12 (crates.io API endpoints + GitHub releases) | Ground-truth version metadata |
| WebSearch | 14 | Cross-verification of release dates, BC notes, MSRV claims |
| Training data | 0 areas of standalone reliance | All version numbers cross-checked against live registry |

**Total MCP tool calls:** ~34 (Context7 + WebFetch + WebSearch combined)
**Training data reliance:** low — every version number was retrieved from crates.io API or the project's GitHub releases page on 2026-05-04. Where WebFetch initial calls returned no content (crates.io HTML page renders client-side), the JSON API endpoint `/api/v1/crates/<name>` was used instead and returned authoritative data.
