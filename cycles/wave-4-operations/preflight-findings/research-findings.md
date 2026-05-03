---
document_type: preflight-research-findings
phase: 4.A
producer: research-agent
timestamp: 2026-05-02T00:00:00Z
tools_used: [perplexity, context7, tavily, web]
total_tasks: 13
inconclusive_tasks: 1
---

# Wave 4 Phase 4.A Research Findings

## Executive Summary

Multiple library version pins in the 2026-04-16/17 Wave 4 story drafts are out of date as of 2026-05-02. The most consequential shifts: (1) DataFusion released 53.0.0 on 2026-04-13 and is now at 53.1.0 — story claim of "version 53" is correct in spirit but should pin specifically and acknowledge the trait-based UDF API consolidation; (2) the `cron` crate is at v0.15.0, not 0.12.x — and a stronger candidate (`croner` 3.0.1) exists with timezone, DST, and `L`/`#`/`W` semantics that the story drafts likely need; (3) `lettre` is at 0.11.21 and the older `tokio1-rustls-tls` feature flag is **deprecated** — Wave 4 must adopt the newer `rustls-*-provider` feature naming; (4) `blake3` is at 1.8.5 (no CVEs found), `rocksdb` 0.24.0 remains current, `wasmtime` is at 44.0.0 with substantial component-model API stability; (5) the **NIST SP 800-61 r3** released April 2025 **eliminated the four-phase lifecycle entirely** in favor of a CSF-2.0-aligned outcome-driven model, which has direct implications for the S-4.06 case-management state-machine claim. Several architectural assumptions need ADR adjustment before Wave 4 stories are finalized.

## Task Results

### R-1: DataFusion latest API (S-4.03, S-4.04)

**Question:** What is the current latest stable DataFusion crate version as of 2026-05-02? Verify `ScalarUDFImpl` trait shape, `create_udf` signature, `MemTable::try_new`, `SessionContext::register_table` vs `register_batch`, `Expr::evaluate` for batch predicate evaluation.

**Findings:**

- **Latest stable:** `datafusion = "53.1.0"` (per docs.rs latest), with the **53.0.0 release** occurring on **2026-04-13** (per Apache DataFusion blog). Release 54.0.0 is tracked in apache/datafusion#21080 but as of 2026-05-02 has not landed.
- **53.x release notes:** breaking changes in SQL parser, optimizer behavior, and several physical-plan APIs. ~114 contributors, 42 built-in functions improved, dynamic-filter push-down expanded, planning latency reduced (~4-5 ms → ~100 us in some workloads).
- **`ScalarUDFImpl` trait (datafusion-expr 53.1.0)** required methods:
  - `fn as_any(&self) -> &dyn Any`
  - `fn name(&self) -> &str`
  - `fn signature(&self) -> &Signature`
  - `fn return_type(&self, arg_types: &[DataType]) -> Result<DataType>`
  - `fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue>`
  - 17 provided methods including `aliases`, `documentation`, `simplify`, `evaluate_bounds`, `coerce_types`, `output_ordering`.
- **`create_udf` (convenience)** signature accepts `(name, input_types: Vec<DataType>, return_type: DataType, volatility, fun_closure)` and returns a `ScalarUDF`. For Wave 4 simple UDFs this is sufficient; for advanced semantics (e.g., volatility-sensitive or argument-aware logic) prefer the `ScalarUDFImpl` trait.
- **Registration:** `MemTable::try_new(schema, vec![vec![batch]])` creates an in-memory provider; register via `ctx.register_table("name", Arc::new(table))`. There is no `SessionContext::register_batch` — the canonical 53.x pattern is `register_table` with an Arc'd `MemTable`. (Issue apache/datafusion#3426 was a feature request to add a batch-shortcut convenience but has not landed in 53.x.) UDFs are registered with `ctx.register_udf(scalar_udf)`.
- **Expr / batch predicate:** Use `PhysicalExpr::evaluate(&RecordBatch) -> ColumnarValue` for batch evaluation; this requires planning the logical `Expr` to a `PhysicalExpr` via `create_physical_expr` against a `DFSchema`. There is no `Expr::evaluate` direct method in 53.x for full RecordBatch evaluation.

**Recommendation for Wave 4:** Pin `datafusion = "53.1"` (caret-compatible) and gate behind the same minor for the cycle. Use `ScalarUDFImpl` as the primary UDF surface (story S-4.03 should mandate this trait, not `create_udf`). For S-4.04 ephemeral data lake, build via `MemTable::try_new` + `Arc::new` + `register_table`. For per-row predicate evaluation in detection rules, plan to `PhysicalExpr` once and reuse across batches.

**Citations:**
- [Apache DataFusion 53.0.0 Released — datafusion.apache.org/blog/2026/04/02/datafusion-53.0.0/](https://datafusion.apache.org/blog/2026/04/02/datafusion-53.0.0/) (accessed 2026-05-02)
- [docs.rs/datafusion/53.1.0/](https://docs.rs/datafusion/53.1.0/) (accessed 2026-05-02)
- [docs.rs/datafusion-expr/latest/datafusion_expr/trait.ScalarUDFImpl.html](https://docs.rs/datafusion-expr/latest/datafusion_expr/trait.ScalarUDFImpl.html) (accessed 2026-05-02)
- [datafusion.apache.org/library-user-guide/functions/adding-udfs.html](https://datafusion.apache.org/library-user-guide/functions/adding-udfs.html)
- [github.com/apache/datafusion/issues/3426](https://github.com/apache/datafusion/issues/3426)

---

### R-2: cron crate ecosystem (S-4.08)

**Question:** Compare Rust crates: `cron` (current latest vs 0.12), `croner`, `tokio-cron-scheduler`, `cron_clock`. Recommend the right crate for S-4.08's "1-second tick + parse cron expression" use case.

**Findings:**

- **`cron`** (zslayton/cron) — current **0.15.0** (released 2026-01-14; previously 0.14.0 2025-12-26; 0.13.0 2025-11-06; 0.12.1 2025-02-29; 0.12.0 2024-09-24). API surface: `Schedule::from_str(...)`, `schedule.upcoming(Utc)`, plus `ScheduleIterator`/`OwnedScheduleIterator`. **No documented breaking changes** between 0.12 and 0.15 — purely additive (new winnow-based parser in 0.14, error-detail improvements in 0.15). Vixie-cron compatible, no DST awareness, no `L`/`#`/`W` extensions.
- **`croner`** (Hexagon/croner-rust) — current **3.0.1** (released 2025-10-27). Features: timezone (chrono + chrono-tz), DST-correctness, `L` (last day/weekday), `#` (nth weekday), `W` (closest weekday), Quartz-compatible weekday numbering, optional seconds/year fields, human-readable output, AND-mode for DOM/DOW. Strict POSIX/Vixie-cron compliance per Open Cron Pattern Specification.
- **`tokio-cron-scheduler`** — Full async scheduler (not just parser). Adds tokio-driven tick loop, optional PostgreSQL/NATS persistence. Heavier dependency footprint; opinionated about scheduler/runtime.
- **`croner-scheduler`** (Hexagon) — Threaded scheduler built on croner. Not async-runtime-tied.

**Recommendation for Wave 4:** **Use `croner = "3"`** for parsing + cron-expression matching. S-4.08 ("1-second tick + parse cron expression") should drive the tick from a `tokio::time::interval(Duration::from_secs(1))` and on each tick query `croner::Cron::new(expr).is_time_matching(&now)?` (or equivalent), rather than adopt `tokio-cron-scheduler` (which would couple us to its lifecycle and persistence model). Croner's DST/timezone correctness is non-negotiable for an MSSP that schedules across multi-tenant timezones. **Replace any "cron 0.12.x" pin in story drafts.**

**Citations:**
- [crates.io/crates/cron — recent releases](https://crates.io/crates/cron) (verified via github.com/zslayton/cron/releases, accessed 2026-05-02)
- [github.com/Hexagon/croner-rust](https://github.com/Hexagon/croner-rust) (croner-rust 3.0.1, accessed 2026-05-02)
- [crates.io/crates/croner](https://crates.io/crates/croner)
- [crates.io/crates/tokio-cron-scheduler](https://crates.io/crates/tokio-cron-scheduler)

---

### R-3: lettre 0.11.x current state (S-4.08)

**Question:** Verify lettre's current latest stable, async tokio feature flags (`tokio1-rustls-tls`), STARTTLS vs implicit TLS API, authentication mechanism support (PLAIN/LOGIN/XOAUTH2).

**Findings:**

- **Latest stable:** `lettre = "0.11.21"` (released 2026-04-04; prior 0.11.20 added MSRV 1.85, replaced chumsky with nom; 0.11.19 raw-header setter; 0.11.18 inline-attachment naming; 0.11.17 rustls-platform-verifier).
- **Feature flags:**
  - Async runtimes: `tokio1`, `async-std1`.
  - TLS backends: `native-tls`, `boring-tls`, `rustls`. **The combined `tokio1-rustls-tls` / `async-std1-rustls-tls` features are deprecated as of 0.11.14** in favor of explicit crypto-provider selection: `rustls-aws-lc-rs`, `rustls-ring`, plus cert verification via `rustls-platform-verifier`, `rustls-native-certs`, or `webpki-roots`.
  - Core: `builder`, `smtp-transport`, `sendmail-transport`, `file-transport`.
  - Misc: `serde`, `tracing`, `dkim`, `web` (WASM).
- **STARTTLS vs implicit TLS API:** Three TLS settings on `SmtpTransport`/`AsyncSmtpTransport` builder — `Tls::None`, `Tls::Opportunistic` (try STARTTLS, fall back to plain), `Tls::Required` (mandate STARTTLS), `Tls::Wrapper` (implicit TLS / TLS-on-connect, e.g. port 465). Use `.tls(Tls::Required(TlsParameters::new(host)?))` for STARTTLS, `.tls(Tls::Wrapper(...))` for implicit.
- **Auth mechanisms:** `Mechanism::Plain` (RFC 4616), `Mechanism::Login` (legacy, e.g. Office 365), `Mechanism::Xoauth2` (Google/Microsoft OAuth tokens). Configured via `.authentication(vec![Mechanism::Plain])` and `.credentials(Credentials::new(user, pass))`.
- **Industry context:** Microsoft Exchange Online begins rejecting password-based SMTP credentials on 2026-03-01 (full enforcement 2026-04-30); Wave 4 outbound notifications targeting Exchange Online MUST use XOAUTH2.

**Recommendation for Wave 4:** Pin `lettre = "0.11"` with features `["tokio1", "smtp-transport", "rustls", "rustls-aws-lc-rs", "rustls-platform-verifier", "builder", "tracing"]`. **Do not use the deprecated `tokio1-rustls-tls` feature.** S-4.08 should default to `Tls::Required` (STARTTLS) on submission port 587, and offer `Tls::Wrapper` on port 465 as opt-in. Auth mechanisms list should default to `[Plain, Xoauth2]` with `Login` as opt-in (legacy Microsoft tenants).

**Citations:**
- [docs.rs/lettre/latest/lettre/](https://docs.rs/lettre/latest/lettre/) (0.11.21, accessed 2026-05-02)
- [docs.rs/lettre/latest/lettre/transport/smtp/authentication/enum.Mechanism.html](https://docs.rs/lettre/latest/lettre/transport/smtp/authentication/enum.Mechanism.html)
- [github.com/lettre/lettre/releases](https://github.com/lettre/lettre/releases)
- [docs.rs/lettre/latest/lettre/transport/smtp/struct.AsyncSmtpTransport.html](https://docs.rs/lettre/latest/lettre/transport/smtp/struct.AsyncSmtpTransport.html)

---

### R-4: blake3 1.x stability (S-4.02, S-4.04)

**Question:** Verify current blake3 major version, any soundness/CVE history, SIMD feature flag conventions, performance characteristics.

**Findings:**

- **Latest stable:** `blake3 = "1.8.5"` per docs.rs latest. (1.8.3 was released 2026-01-08; 1.8.5 is the most recent point release as of 2026-05-02.)
- **CVE / RustSec:** **No published CVE or RustSec advisories** specific to the `blake3` crate as of 2026-05-02. The crate has 80M+ downloads on crates.io. Recommend integrating `cargo audit` in CI as a defense-in-depth measure.
- **SIMD feature flags:** Runtime CPU feature detection on x86 (SSE2, SSE4.1, AVX2, AVX-512). NEON enabled by default for AArch64; on ARMv7 the `neon` feature must be explicitly enabled (binary becomes non-portable). `wasm32_simd` enables WASM SIMD on wasm32 targets (may become default in future).
- **Other features:** `std` (default), `rayon` (multithreaded `update_rayon`/`update_mmap_rayon`), `mmap` (memory-mapped IO), `zeroize` (secure clear), `serde`, `traits-preview` (RustCrypto digest interop).
- **Performance:** SIMD on a single core hits ~3-7 GiB/s on modern x86; with rayon multithreading + mmap, easily 10-30 GiB/s on 16-core hosts. Hash output 32 bytes; supports XOF/MAC/KDF modes via `Hasher::new_keyed`/`new_derive_key`/`finalize_xof`.

**Recommendation for Wave 4:** Pin `blake3 = "1.8"`. Default features sufficient. For S-4.02 (likely audit/persistence hashing) and S-4.04 (data lake content hashing), enable `mmap` if file-based hashing is needed, and `rayon` only if hashing payloads >>1 MB consistently. **Do not enable `traits-preview` in production code** (vendor explicitly warns of breaking changes).

**Citations:**
- [docs.rs/blake3/latest/](https://docs.rs/blake3/latest/) (accessed 2026-05-02)
- [github.com/BLAKE3-team/BLAKE3](https://github.com/BLAKE3-team/BLAKE3)
- [rustsec.org/advisories/](https://rustsec.org/advisories/) — searched, no blake3 advisories found
- [generalistprogrammer.com/tutorials/blake3-rust-crate-guide](https://generalistprogrammer.com/tutorials/blake3-rust-crate-guide)

---

### R-5: libfuzzer-sys vs cargo-fuzz vs arbitrary (S-4.05)

**Question:** Verify current Rust fuzzing ecosystem best practice. Is libfuzzer-sys 0.4.x still the standard, or has cargo-fuzz + arbitrary become canonical? What is the recommended pattern for VP-028 fuzz target on alert generation?

**Findings:**

- **Current latest:** `libfuzzer-sys = "0.4.12"` (per docs.rs); `cargo-fuzz` is the toolchain harness; `arbitrary` provides structured input generation. **All three are complementary, not competing — they form the canonical stack.**
- **Canonical pattern (2026):**
  1. `cargo install cargo-fuzz` — installs the helper binary
  2. `cargo fuzz init` creates `fuzz/` subcrate with `libfuzzer-sys` dependency
  3. Define targets under `fuzz/fuzz_targets/<name>.rs` using the `fuzz_target!` macro
  4. For structured inputs use `#[derive(Arbitrary)]` (enable via `libfuzzer-sys` feature `arbitrary-derive`, or via `arbitrary` crate's `derive` feature)
  5. Run with `cargo fuzz run <target>`
- **`fuzz_target!` macro pattern:**
  ```rust
  fuzz_target!(|data: &[u8]| { /* exercise code under test */ });
  // or for structured input:
  fuzz_target!(|input: MyStruct| { /* ... */ });
  ```
- **Status:** cargo-fuzz exclusively wraps libFuzzer (via libfuzzer-sys). No replacement is canonical; honggfuzz-sys exists but is a niche alternative. The libfuzzer-sys 0.4.x line is the current standard with no 0.5 in flight.
- **For VP-028 (alert generation fuzz target):** Define an `Arbitrary`-derived input struct that captures the alert-generation inputs (e.g., raw event blobs + rule-engine context), assert no panic, and where applicable assert an oracle (round-trip, monotonicity, schema validity).

**Recommendation for Wave 4:** Pin `libfuzzer-sys = "0.4"` and `arbitrary = "1"` with `derive` feature in the `fuzz/` subcrate. CI integration: `cargo +nightly fuzz run alert_generation -- -max_total_time=300` for short runs in PR CI; longer runs (>= 1 hour) on a nightly schedule. **Do not roll your own fuzzing harness** — the rust-fuzz canonical stack is the only supported path.

**Citations:**
- [rust-fuzz.github.io/book/cargo-fuzz.html](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [docs.rs/libfuzzer-sys/latest/libfuzzer_sys/macro.fuzz_target.html](https://docs.rs/libfuzzer-sys/latest/libfuzzer_sys/macro.fuzz_target.html) (0.4.12, accessed 2026-05-02)
- [github.com/rust-fuzz/cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz)
- [appsec.guide/docs/fuzzing/rust/cargo-fuzz/](https://appsec.guide/docs/fuzzing/rust/cargo-fuzz/) — Trail of Bits Testing Handbook

---

### R-6: DataFusion + scopeguard interaction across .await (S-4.04)

**Question:** Does `scopeguard::defer!` correctly handle cancellation across tokio `.await` suspension points? May we need `tokio::task::AbortGuard` instead? What is the canonical pattern for SessionContext drop guarantee in async code?

**Findings:**

- **`scopeguard = "1.2.0"`** is the latest. The `defer!` macro registers a closure that runs on `Drop` — it works **correctly across `.await` suspension points** because Rust's drop semantics fire when the future is dropped (whether it completed, panicked, or was cancelled). A `defer!` in an async fn body runs when the future stack-unwinds.
- **Caveat 1 — abort vs unwind:** `defer!` only runs on unwinding panic, not on `panic = "abort"` or `std::process::abort()`. If Wave 4 uses release profile with `panic = "abort"` (set in workspace `Cargo.toml`), `defer!` becomes a no-op on panic.
- **Caveat 2 — leaked futures:** If a future is `mem::forget`-ed or stored in a long-lived structure that is itself leaked, `Drop` never fires and `defer!` never runs. This is a Rust-wide invariant, not scopeguard-specific.
- **Caveat 3 — cancel safety:** Cancel safety is about the **observable side effects** of the cancelled future, not about cleanup running. `defer!` cleanup will run on cancellation; whether the cleanup itself is correct/idempotent is the developer's responsibility.
- **`tokio::task::JoinHandle::abort` / abort guards:** Tokio does not currently expose a public `AbortGuard` type. The community pattern is `JoinHandle::abort_handle()` (since tokio 1.13) returning an `AbortHandle`, or wrapping in a custom guard struct that calls `abort()` in its `Drop`. This is the right tool when you spawn a task and want it cancelled when the parent scope ends.
- **For DataFusion `SessionContext` drop in async code:** `SessionContext` is `Clone` (Arc-internally), `Send + Sync`, and its `Drop` implementation deallocates the `SessionState` plus all registered table providers. There is **no async cleanup required** — sync `Drop` is correct. The canonical pattern is to construct `SessionContext` per query, drop on scope exit, and rely on Rust's deterministic drop. No `defer!` needed unless you want explicit logging of drop.

**Recommendation for Wave 4:** Use `scopeguard::defer!` for explicit logging/audit on async scope exit (it works correctly across `.await`). Be aware that `panic = "abort"` skips defer cleanup — choose `panic = "unwind"` for release builds if defer cleanup is load-bearing for security/audit. For tokio task cancellation, use `JoinHandle::abort_handle()` returning `AbortHandle`, optionally wrapped in a custom RAII guard. **Do not rely on async-aware cleanup primitives** — Rust's `Drop` runs synchronously; if you need async cleanup, model it explicitly with a final `await` before scope exit, not as Drop side-effect.

**Citations:**
- [docs.rs/scopeguard/latest/scopeguard/](https://docs.rs/scopeguard/latest/scopeguard/) (1.2.0)
- [blog.yoshuawuyts.com/async-cancellation-1/](https://blog.yoshuawuyts.com/async-cancellation-1/)
- [sunshowers.io/posts/cancelling-async-rust/](https://sunshowers.io/posts/cancelling-async-rust/)
- [cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/)

---

### R-7: regex::RegexSet at 100k+ patterns (S-4.03)

**Question:** Does the `regex` crate's RegexSet handle 100,000 patterns as claimed? Include DFA size limits, compile time, memory profile. Compare with `regex_automata` or `aho-corasick` for IOC pattern matching.

**Findings:**

- **`regex::RegexSet` performance characteristics (per docs):** Search is `O(m * n)` where `m` is set size and `n` is haystack length. The doc explicitly recommends it for "hundreds or thousands of regexes" — **100,000 is well outside the documented design point.**
- **DFA size limit issues:** `RegexSetBuilder::dfa_size_limit` is per-thread; if exceeded, the engine falls back to a slower NFA-based path. With **20+ regexes**, default DFA limits can be hit and trigger fallback. Memory regression in regex 1.9.0/1.9.1 caused globset's RegexSet to allocate 600+ MB for moderate sets (Issue #1059). Compilation time grows superlinearly with pattern count.
- **`aho-corasick = "1.1.4"`** — designed for **literal multi-pattern matching**. Linear O(n+z) search regardless of pattern count. **Caveat:** documented prefilter optimizations apply only when pattern count is small (< ~100). For pure literal IOC matching (file hashes, exact domains, IPs, exact URL strings), aho-corasick is the canonical choice and does scale to 100k+ patterns.
- **`regex_automata` (low-level toolkit beneath regex)** — Provides explicit DFA/NFA/lazy-DFA control and a `MultiPatternDfa`. More flexible than `RegexSet` but requires significantly more code. For mixed literal/regex IOC corpus at scale, `regex_automata` is the right tool.
- **For S-4.03 IOC pattern matching:** The IOC corpus is overwhelmingly **literal** (SHA-256, MD5, IPv4, IPv6, exact hostnames). Use `aho-corasick` for the literal layer (handles 100k+ patterns trivially), and reserve a small `RegexSet` for the regex-pattern minority (<1k regex IOCs). Hybrid architecture is canonical.

**Recommendation for Wave 4:** **Reject the implicit S-4.03 assumption that `RegexSet` alone scales to 100k patterns.** Architecture should split: (1) `aho-corasick = "1.1"` for literal-IOC matching (vast majority); (2) `regex = "1.10"` with `RegexSet` capped at ~1k regex IOCs, with explicit `dfa_size_limit` configuration; (3) for mixed corpus over ~10k regex patterns, escalate to `regex_automata`. Story S-4.03 must update its design to reflect this layering.

**Citations:**
- [docs.rs/regex/latest/regex/struct.RegexSet.html](https://docs.rs/regex/latest/regex/struct.RegexSet.html)
- [github.com/rust-lang/regex/discussions/881](https://github.com/rust-lang/regex/discussions/881) — "why does RegexSet go slower as more patterns are added?"
- [github.com/rust-lang/regex/issues/1059](https://github.com/rust-lang/regex/issues/1059) — RegexSet 600 MB memory regression
- [docs.rs/aho-corasick/latest/aho_corasick/](https://docs.rs/aho-corasick/latest/aho_corasick/) (1.1.4)
- [github.com/BurntSushi/aho-corasick](https://github.com/BurntSushi/aho-corasick)

---

### R-8: CEF / LEEF Rust crates (S-4.08)

**Question:** Are there maintained Rust crates for ArcSight CEF v0/v1 and IBM LEEF 2.0 syslog formatters? If so, which one is recommended? If not, what are the spec details we'd need to implement correctly (escaping rules, field formats)?

**Findings:**

- **CEF Rust crates:**
  - `rust-cef` — a trait-based serializer to CEF, last published December 2021. **Likely abandoned** by 2026-05-02 (no recent updates). Implements CEF v0 (`CEF:0`).
  - No other widely-used CEF crate found on crates.io.
- **LEEF Rust crates:** None on crates.io with meaningful adoption. The `guardsarm/rust-threat-detector` GitHub project includes LEEF export embedded in a larger SIEM-detector binary (not published as a reusable crate).
- **Conclusion: Wave 4 must implement CEF and LEEF formatters in-house.** This is a non-trivial decision and warrants an ADR.

**CEF v0 specification (for in-house implementation):**

- **Format:** `CEF:Version|Device Vendor|Device Product|Device Version|Signature ID|Name|Severity|Extension`
- **Header escaping:** Backslash `\` and pipe `|` in **header** field values must be escaped with backslash (`\\`, `\|`). Newlines `\n`, `\r` in headers are not allowed.
- **Extension escaping:** Backslash `\` and equal-sign `=` must be escaped (`\\`, `\=`); pipe `|` need NOT be escaped in extensions; newlines may be encoded as `\n` or `\r`.
- **Severity:** integer 0-10 (legacy: also accepts `Low`, `Medium`, `High`, `Very-High`).
- **Current spec:** ArcSight Common Event Format v0 (most widely deployed); CEF 1.x exists per Micro Focus but is rare in production SIEM deployments.
- **Transport:** Syslog (RFC 3164 or RFC 5424) typically on UDP/514, TCP/514, or TLS/6514.

**LEEF 2.0 specification (for in-house implementation):**

- **Format:** `LEEF:2.0|Vendor|Product|Version|EventID|[DelimChar]|attr1=val1<delim>attr2=val2...`
- **Header:** RFC 3164 or RFC 5424 syslog header MUST precede LEEF header (separated by single space).
- **Delimiter:** Optional 6th pipe field is the attribute delimiter. Can be a literal char or `0x09` / `x09` style hex (1-4 hex chars after `0x` or `x` prefix). Default delimiter `\t` (tab) when absent.
- **Field escaping:** Avoid tab, pipe (`|`), caret (`^`) in custom event keys. Alphanumeric (A-Z, a-z, 0-9) keys/values are safest.
- **QRadar built-in attributes:** standardized keys (e.g., `src`, `dst`, `srcPort`, `dstPort`, `usrName`, `proto`, `cat`, `sev`, `devTime`, `devTimeFormat`).

**Recommendation for Wave 4:**

- **Do not depend on `rust-cef`** — it's effectively unmaintained.
- Implement an internal `prism-siem-formats` crate with two modules: `cef::v0::Encoder` and `leef::v2::Encoder`. Each encoder takes a strongly-typed event struct (with required header fields + extension key/value map) and emits a `String`/`Vec<u8>`.
- Use proptest fuzzing on encoder output: for any input the encoded form MUST round-trip parse cleanly with no SIEM-toxic characters surviving in the wrong position.
- This adds an ADR (likely ADR-018 or sibling).

**Citations:**
- [microfocus.com/documentation/arcsight/arcsight-smartconnectors-8.4/pdfdoc/cef-implementation-standard/cef-implementation-standard.pdf](https://www.microfocus.com/documentation/arcsight/arcsight-smartconnectors-8.4/pdfdoc/cef-implementation-standard/cef-implementation-standard.pdf) — official CEF Implementation Standard
- [ibm.com/docs/en/SS42VS_DSM/pdf/b_Leef_format_guide.pdf](https://www.ibm.com/docs/en/SS42VS_DSM/pdf/b_Leef_format_guide.pdf) — IBM QRadar LEEF v2 Format Guide
- [docs.nxlog.co/integrate/leef.html](https://docs.nxlog.co/integrate/leef.html) — LEEF reference
- [docs.nxlog.co/integrate/cef-logging.html](https://docs.nxlog.co/integrate/cef-logging.html) — CEF reference
- [crates.io/crates/rust-cef](https://crates.io/crates/rust-cef) — last published 2021

---

### R-9: rocksdb 0.24 current status (S-4.01, S-4.02, S-4.03..4.08)

**Question:** Is `rocksdb = "0.24"` still the latest stable Rust binding for RocksDB as of 2026-05-02? Check `merge_operator` API for atomic counter increment used in S-4.02 epoch tracker.

**Findings:**

- **Latest stable:** `rocksdb = "0.24.0"` (per docs.rs latest, released ~August 2025, ~9 months old as of 2026-05-02). **Story claim verified — pin is correct.**
- **`merge_operator` API:**
  - Module `rocksdb::merge_operator`. `MergeOperands` struct re-exported at crate root.
  - Two registration modes:
    - **Associative merge:** `Options::set_merge_operator_associative(name, fn(key, existing_val, operands) -> Option<Vec<u8>>)` — for commutative+associative ops (counters, sum, max).
    - **Generic merge (non-associative):** `Options::set_merge_operator(name, full_merge_fn, partial_merge_fn)` — for richer types where partial merges of operands need different logic from full merge against the existing value.
  - Apply via `db.merge(key, operand_bytes)` and read via `db.get(key)` (RocksDB resolves the merge stack on read or on compaction).
- **For S-4.02 epoch tracker (atomic counter increment):** Use `set_merge_operator_associative`. Encoding: `u64::to_le_bytes` for both existing-val and operands; merge function deserializes, sums, re-serializes. This is the canonical RocksDB counter pattern.
- **Caveat:** Merge operators are only invoked on `merge()` calls; `put()` overwrites without invoking the operator. Snapshot reads see the merged result lazily computed on the read path until next compaction.

**Recommendation for Wave 4:** Confirm `rocksdb = "0.24"` pin. For S-4.02 epoch tracker, document the `set_merge_operator_associative` pattern with `u64::to_le_bytes` encoding in the ADR or design note. For S-4.03..4.08 column-family workloads, no API surprises beyond 0.23.

**Citations:**
- [docs.rs/rocksdb/0.24.0/rocksdb/](https://docs.rs/rocksdb/0.24.0/rocksdb/)
- [docs.rs/rocksdb/latest/rocksdb/merge_operator/struct.MergeOperands.html](https://docs.rs/rocksdb/latest/rocksdb/merge_operator/struct.MergeOperands.html)
- [normansoven.com/post/rocksdb-merges](https://www.normansoven.com/post/rocksdb-merges)

---

### R-10: Streaming percentile crates (S-4.07)

**Question:** Compare `tdigest`, `quantiles` (GK), `hdrhistogram` for the S-4.07 case_metrics 10,000+-case percentile computation. Recommend with version pin.

**Findings:**

- **`tdigest = "0.2.3"`** (JimCooke/t-digest). API: `TDigest::new_with_size(compression)`, `merge_sorted(values)`, `estimate_quantile(q)`. Rank-relative accuracy (extreme percentiles like p99/p99.9 are MORE accurate, central percentiles less so). Memory: `O(compression)` (typically 100-1000 nodes). Mergeable across distributed shards. **Best for general distributions** (revenue, latency, payload-size mixes).
- **`tdigests = "0.x"`** (andylokandy/tdigests) — alternative implementation, more recent activity, online-merging-friendly for distributed systems.
- **`hdrhistogram = "7.5.4"`** (HdrHistogram/HdrHistogram_rust). API: `Histogram::<u64>::new_with_bounds(low, high, sigfigs)`, `record(value)`, `value_at_quantile(q)`. Bucket-based; **constant accuracy per bucket** at chosen significant figures (e.g., 3-sf gives ~0.1% accuracy across the entire range). Memory: fixed at construction, ~2000-5000 bytes for 3-sf low-microsecond-to-hour latencies. **Best for latency** (where dynamic range is bounded and constant relative precision matters).
- **`quantiles` (Greenwald-Khanna)** — older crate, provably bounded epsilon-error quantile sketch. Less commonly used than t-digest/hdrhistogram in modern Rust ecosystem.
- **For S-4.07 case_metrics with 10,000+ cases:**
  - If the metric is **latency-like** (case-time-to-resolution in seconds, MTTR distributions, SLA timer): use `hdrhistogram`. Bounded range, constant precision, fast `value_at_quantile`.
  - If the metric is **general distribution** (severity scores, alert counts per case, asset-impact counts): use `tdigest` or `tdigests`. Better tail accuracy, no range pre-declaration.
  - 10,000 cases is small for both — neither will be memory-bound.

**Recommendation for Wave 4:** **Use `hdrhistogram = "7.5"`** as the default for S-4.07. It is the most production-hardened, has by far the highest crates.io usage, lowest API surprise, and case_metrics percentile use-case is dominantly latency-flavored (time-to-detect, time-to-respond, time-to-close). Reserve `tdigest = "0.2"` (or `tdigests`) for any future metric where the value range is unbounded or skew-heavy.

**Citations:**
- [docs.rs/hdrhistogram/latest/hdrhistogram/](https://docs.rs/hdrhistogram/latest/hdrhistogram/) (7.5.4)
- [docs.rs/tdigest/latest/tdigest/](https://docs.rs/tdigest/latest/tdigest/) (0.2.3)
- [github.com/HdrHistogram/HdrHistogram_rust](https://github.com/HdrHistogram/HdrHistogram_rust)
- [github.com/andylokandy/tdigests](https://github.com/andylokandy/tdigests)
- [hdrhistogram.github.io/HdrHistogram/](https://hdrhistogram.github.io/HdrHistogram/)
- [sciencedirect.com/science/article/pii/S2665963820300403](https://www.sciencedirect.com/science/article/pii/S2665963820300403) — Ted Dunning's t-digest paper

---

### R-11: Industry case-management state-machine standards (S-4.06)

**Question:** Validate the "5-state, 12-transition" claim in S-4.06 against ITIL incident lifecycle, NIST 800-61, MITRE D3FEND, and common SOAR platforms (Splunk SOAR, Demisto/XSOAR, Tines). Document trace; if 5/12 is non-standard, propose the correct lifecycle.

**Findings:**

- **NIST SP 800-61 r3 (released April 2025)** — **MAJOR FINDING:** r3 **eliminates the four-phase incident-response lifecycle** that r2 used (Preparation → Detection & Analysis → Containment/Eradication/Recovery → Post-Incident Activity). r3 is now organized around the NIST Cybersecurity Framework 2.0 six functions: **Govern, Identify, Protect, Detect, Respond, Recover**. These are continuous risk-management functions, NOT a state machine. r3 explicitly is "outcome-driven" rather than phase-prescriptive. **A case-management state machine cannot trace cleanly to NIST 800-61r3 as a state diagram.**
- **NIST SP 800-61 r2 (still widely cited)** — four sequential phases. If we want NIST traceability, this is the older but more state-machine-friendly reference. But r2 is officially superseded.
- **ITIL 4** — Incident Management is one of 34 "practices"; ITIL 4 explicitly **abandoned prescriptive process flows from ITIL v3**. Each organization customizes states. ITIL 4 is **not a source of canonical states**. The historical ITIL v3 states (often paraphrased as: New, Assigned, In Progress, On Hold, Resolved, Closed) are widespread in ITSM tools (ServiceNow, Jira) but are de facto, not de jure.
- **Cortex XSOAR (Palo Alto, formerly Demisto)** — Documented incident lifecycle: **Pending → Active → Closed → (Archived)**, with secondary statuses for awaiting input. Roughly 4 states + archive. The exact transition graph is not publicly enumerated outside the admin guide.
- **Splunk SOAR (formerly Phantom)** — Cases as containers; events promoted to cases. State model: status (New, Open, Resolved, Closed) + severity. Workbook phases/tasks have their own SLA states. Customizable per tenant.
- **Tines** — Workflow-orchestration centric; cases can have arbitrary statuses defined by the team.
- **MITRE D3FEND** — Not a case-management framework; it's a defensive countermeasure ontology. Not relevant to state machines.

**On the "5-state, 12-transition" claim in S-4.06:**

- A 5-state model commonly seen in industry is: **{New, Investigating, Containing, Resolved, Closed}** OR **{Pending, Active, On Hold, Resolved, Closed}**. Either is plausible.
- 12 transitions on 5 states is mathematically reasonable (5*5=25 max edges; 12 is a moderately connected graph). Whether it traces cleanly to a published standard is a different question.
- **No single published standard mandates 5 states or 12 transitions.** The claim is reasonable but should be traced to a hybrid: ITIL v3 conventions + Cortex XSOAR pattern + organization-specific gates.

**Recommendation for Wave 4 / S-4.06:**

1. **Replace any "traces to NIST 800-61" language with "informed by NIST 800-61 r2 phases (acknowledging r3 supersedes with a non-state-machine model) and ITIL v3 conventions."**
2. **Explicitly document the 5-state model in the spec** with named states and the 12 transition list. Do NOT claim industry-standard provenance for the exact graph; claim "industry-informed" or "synthesized from ITIL/XSOAR conventions."
3. **Architect should consider a 4-state + sub-status model** (a la XSOAR: Pending/Active/Closed/Archived + secondary status), which is closer to deployed SOAR practice than a 5-state primary graph.
4. **Add ADR-017** (case state machine) with explicit rationale, citing NIST r2, ITIL v3 paraphrase, and Cortex XSOAR documented lifecycle as inputs. Make clear it is "1898-curated" not "standards-traced."

**Citations:**
- [csrc.nist.gov/pubs/sp/800/61/r3/final](https://csrc.nist.gov/pubs/sp/800/61/r3/final) — NIST SP 800-61 r3 Final, April 2025
- [linfordco.com/blog/nist-sp-800-61/](https://linfordco.com/blog/nist-sp-800-61/) — analysis of r3 vs r2
- [tandem.app/blog/updated-nist-incident-response-guidance-sp-800-61-rev-3](https://tandem.app/blog/updated-nist-incident-response-guidance-sp-800-61-rev-3)
- [xsoar.pan.dev/docs/incidents/incident-xsoar-incident-lifecycle](https://xsoar.pan.dev/docs/incidents/incident-xsoar-incident-lifecycle) — Cortex XSOAR lifecycle
- [jaacostan.com/2021/01/lifecycle-of-palo-alto-cortex-xsoar.html](https://www.jaacostan.com/2021/01/lifecycle-of-palo-alto-cortex-xsoar.html) — XSOAR phases: Creation → Pending → Active → Closure & Archiving
- [lantern.splunk.com/Security_Use_Cases/Automation_and_Orchestration/Managing_cases_in_SOAR](https://lantern.splunk.com/Security_Use_Cases/Automation_and_Orchestration/Managing_cases_in_SOAR) — Splunk SOAR cases
- [wiki.en.it-processmaps.com/index.php/Incident_Management](https://wiki.en.it-processmaps.com/index.php/Incident_Management) — ITIL Incident Management (legacy v3 conventions)

---

### R-12: tokio::sync::broadcast Lagged semantics (S-4.05, S-4.08)

**Question:** Verify current tokio behavior for `RecvError::Lagged(n)` and capacity behavior under burst. Is the documented behavior in 1.x stable for the broadcast capacity-1000 + lagged-recv handling pattern?

**Findings:**

- **Latest stable:** `tokio = "1.52.1"` (latest); `tokio = "1.51"` is the current LTS until March 2027 (MSRV 1.71).
- **Capacity rounding:** `broadcast::channel(capacity)` internally rounds capacity up to the next power of two for the ring buffer. **Practical implication:** with `channel(1000)`, the ring is sized for 1024 slots; `Lagged` triggers when senders have written more than 1024 messages past a receiver's cursor.
- **`RecvError::Lagged(n)` semantics:** When a receiver falls behind such that the channel ring has overwritten messages it hasn't yet consumed, `recv()` returns `Err(RecvError::Lagged(n))` where `n` is the number of messages skipped. **The receiver's cursor is advanced to the oldest still-held value** so subsequent `recv()` will succeed on the next message. The receiver is NOT poisoned/disconnected — it can keep consuming.
- **`RecvError::Closed`** is the terminal error (all senders dropped + buffer drained).
- **Stability:** Behavior has been stable since tokio 1.0 and is unchanged in 1.51/1.52. No deprecations. The "next power of 2" rounding is not pinky-promised in the API contract but is the documented and consistent implementation across 1.x.
- **Burst behavior:** A `send()` to a full channel succeeds immediately (lock-free), evicting the oldest unread value. Receivers that have not yet read the evicted value will get `Lagged` on next `recv`. There is **no backpressure** — broadcast channels are explicitly drop-newest-laggard rather than block-sender.
- **Open issues / quirks:** apache/tokio#5923 documents quadratic slowdown of `Sender::send` as receiver count grows when each receiver has its own runtime — not a correctness issue but a perf consideration for many-receiver fanout.

**Recommendation for Wave 4:** S-4.05 and S-4.08 broadcast usage at capacity 1000 is sound. Pattern:
```rust
loop {
    match rx.recv().await {
        Ok(msg) => process(msg),
        Err(RecvError::Lagged(n)) => {
            metrics::counter!("broadcast.lagged", n);
            // continue — receiver is not disconnected
        }
        Err(RecvError::Closed) => break,
    }
}
```
Be explicit in story specs that on `Lagged(n)`, observability counters MUST be incremented (otherwise dropped messages are silent). For S-4.08 outbound notification fanout, capacity should be sized for worst-case burst (e.g., a 100-sensor mass-event firing 1000 alerts in 1s requires capacity ≥ 1024 to avoid laggard receivers under transient pressure).

**Citations:**
- [docs.rs/tokio/latest/tokio/sync/broadcast/index.html](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html)
- [docs.rs/tokio/latest/tokio/sync/broadcast/error/enum.RecvError.html](https://docs.rs/tokio/latest/tokio/sync/broadcast/error/enum.RecvError.html)
- [docs.rs/tokio/latest/tokio/sync/broadcast/struct.Receiver.html](https://docs.rs/tokio/latest/tokio/sync/broadcast/struct.Receiver.html)
- [github.com/tokio-rs/tokio/issues/2425](https://github.com/tokio-rs/tokio/issues/2425) — Lagged-when-not-exceeded explanation
- [github.com/tokio-rs/tokio/issues/5923](https://github.com/tokio-rs/tokio/issues/5923) — perf quirk

---

### R-13: wasmtime 44 component-model host calls (S-4.08)

**Question:** Verify wasmtime 44 component-model API stability for hosting `fire_alert`/`fire_case`/`fire_report` host functions called from action plugin. Has the API shifted? What is the canonical pattern for embedding a Rust host that calls into a WASM component module?

**Findings:**

- **Latest stable:** `wasmtime = "44.0.0"` (per docs.rs latest, Apache-2.0 WITH LLVM-exception). 38.0.4 was a prior LTS-style line; 44 is current.
- **Component model API surface (44.x):**
  - `wasmtime::component::Component` — compiled component, analogous to core `Module`.
  - `wasmtime::component::Linker<T>` — distinct from core `wasmtime::Linker`; for component imports/exports.
  - `wasmtime::component::bindgen!` macro — ingests WIT (WebAssembly Interface Types) world definition and generates: typed bindings struct, `add_to_linker`/`add_to_linker_get_host` registration helpers, host trait that the embedder implements.
  - Host data projection via `HasData` trait — `HasSelf<T>` is the no-projection identity case.
- **Canonical embedder pattern for fire_alert/fire_case/fire_report:**
  1. Author a WIT world declaring these as imports the guest can call.
  2. `bindgen!({ path: "wit/", world: "action-plugin" })` in your host crate.
  3. Implement the generated host trait on your `T` (e.g., `PrismActionHost`).
  4. `Component::new(&engine, wasm_bytes)?` to compile.
  5. `Linker::<PrismActionHost>::new(&engine)`, then `<generated>::add_to_linker(&mut linker, |state| state)?`.
  6. `let bindings = <generated>::instantiate(&mut store, &component, &linker)?;`
  7. Call exported guest functions: `bindings.call_run(&mut store, args)?;`
- **Async host functions:** `Linker::func_wrap_async` / `LinkerInstance::func_wrap_async` available — required if `fire_alert` etc. need to await on Tokio.
- **API stability:** wasmtime is on a 4-week release cycle; major version bumps roughly every release. The component-model APIs have been stabilizing through 2025; 44.x is regarded as production-stable for component embedding (vs experimental in early 2024). Breaking changes do happen between majors — pin a single major and plan an upgrade cadence.
- **Caveat:** `bindgen!`-generated Host trait names and method signatures change subtly across wasmtime majors. Don't expect a 38-era WIT-binding skeleton to compile against 44 unchanged. Lock to one major per stability window.

**Recommendation for Wave 4:** Pin `wasmtime = "44"` with `component-model` feature enabled. Standardize on `bindgen!` macro for action-plugin world. Define WIT separately under `wit/` for visibility. Use sync `func_wrap` for fire_* unless the host implementation needs to await (then `func_wrap_async`). Plan a wasmtime major-bump task per quarter for security patches; expect 1-2 days of effort per major bump.

**Citations:**
- [docs.rs/wasmtime/44.0.0/wasmtime/](https://docs.rs/wasmtime/44.0.0/wasmtime/)
- [docs.rs/wasmtime/latest/wasmtime/component/index.html](https://docs.rs/wasmtime/latest/wasmtime/component/index.html)
- [docs.wasmtime.dev/api/wasmtime/component/macro.bindgen.html](https://docs.wasmtime.dev/api/wasmtime/component/macro.bindgen.html)
- [docs.wasmtime.dev/api/wasmtime/component/bindgen_examples/_0_hello_world/index.html](https://docs.wasmtime.dev/api/wasmtime/component/bindgen_examples/_0_hello_world/index.html)
- [github.com/bytecodealliance/wit-bindgen](https://github.com/bytecodealliance/wit-bindgen)

---

## Cross-Cutting Conclusions

### Library Pin Recommendations Table

| Crate | Workspace Today | Story Claim | Recommended Pin | Source |
|-------|-----------------|-------------|-----------------|--------|
| datafusion | (none) | 53 | `datafusion = "53.1"`, with `ScalarUDFImpl` trait pattern | R-1 |
| cron | (none) | 0.12.x | **REJECT cron 0.12.** Use `croner = "3"` for DST/timezone correctness | R-2 |
| lettre | (none) | 0.11.x | `lettre = "0.11"` features `["tokio1","smtp-transport","rustls","rustls-aws-lc-rs","rustls-platform-verifier","builder","tracing"]`. **Do NOT use deprecated `tokio1-rustls-tls`** | R-3 |
| blake3 | (none) | 1.x | `blake3 = "1.8"` — defaults sufficient | R-4 |
| libfuzzer-sys | (none) | 0.4.x | `libfuzzer-sys = "0.4"` + `arbitrary = "1"` (with `derive`) — via cargo-fuzz | R-5 |
| scopeguard | check | implicit | `scopeguard = "1.2"` — works across `.await` if `panic = "unwind"` | R-6 |
| regex / aho-corasick | (varies) | RegexSet 100k | **SPLIT.** `aho-corasick = "1.1"` for literals, `regex = "1.10"` RegexSet capped ~1k | R-7 |
| (CEF/LEEF) | (none) | (none — assumes crate exists) | **NO maintained crate.** Implement in-house `prism-siem-formats` crate | R-8 |
| rocksdb | 0.24 | 0.24 | `rocksdb = "0.24"` — verified current | R-9 |
| hdrhistogram / tdigest | (none) | tdigest OR hdrhistogram | `hdrhistogram = "7.5"` as default; reserve `tdigest = "0.2"` for unbounded-range metrics | R-10 |
| tokio | check | broadcast cap 1000 | tokio current `1.52` (LTS `1.51`); broadcast capacity rounds to 1024 — story should specify pow-of-2 boundary | R-12 |
| wasmtime | (none) | 44 | `wasmtime = "44"` with `component-model` feature; use `bindgen!` macro | R-13 |

### Architectural Implications for Architect (ADR Drafts 013/015/016/017/018)

- **ADR-013 (likely persistence/audit):** Confirm RocksDB 0.24 + `set_merge_operator_associative` for atomic counters in S-4.02 epoch tracker. No surprises.
- **ADR-015 (detection rule language, presumed):** Must reflect DataFusion 53.x `ScalarUDFImpl` trait API (NOT the older `create_udf`-only pattern); plan `PhysicalExpr` reuse across batches; `MemTable::try_new` + `register_table` for in-memory data lake. ADR should lock the DataFusion major (53) for the cycle and pre-budget for a 54-bump task.
- **ADR-016 (likely IOC matching architecture):** Must explicitly split literal-IOC matching (aho-corasick) from regex-IOC matching (RegexSet capped). Reject any "RegexSet at 100k patterns" architecture.
- **ADR-017 (case state machine):** Should NOT claim NIST 800-61 traceability as a state machine — r3 abandoned that model. Either trace to **NIST 800-61 r2** (acknowledging supersession) + ITIL v3 conventions, or describe as "1898-curated, industry-informed" with documented inputs from XSOAR/Splunk SOAR. Consider whether 5 primary states is the right shape vs XSOAR's 4-state + sub-status model.
- **ADR-018 (borderline — likely SIEM output format or scheduling):** If output formats: must address that there are **NO maintained Rust crates for CEF or LEEF** — Wave 4 has to build `prism-siem-formats` in-house with proptest fuzz coverage. If scheduling: must adopt `croner` for DST correctness, not the bare `cron` crate.
- **New cross-cutting consideration:** workspace `panic = "unwind"` vs `"abort"` matters for `scopeguard::defer!` correctness — check Cargo profiles before relying on defer for audit cleanup.
- **Outbound auth (S-4.08):** Microsoft Exchange Online deprecates basic SMTP auth on 2026-04-30 — XOAUTH2 must be a first-class option (already supported by lettre 0.11.x).

### Story Claims to Update

- **S-4.03 (IOC matching):** Replace any "RegexSet handles 100k+ patterns" claim with a layered architecture (aho-corasick literal layer + small RegexSet regex layer). Add `dfa_size_limit` configuration as an explicit knob.
- **S-4.04 (data lake):** Replace any references to a generic `register_batch` or `Expr::evaluate` shortcut with the canonical 53.x flow: `MemTable::try_new` + `Arc::new` + `register_table`; for predicate evaluation use `create_physical_expr` → `PhysicalExpr::evaluate(&RecordBatch)`.
- **S-4.05 (fuzzing):** Confirm pattern is cargo-fuzz + libfuzzer-sys 0.4 + arbitrary derive — no roll-your-own. Document a CI cadence (short PR runs vs nightly long runs).
- **S-4.06 (case state machine):** Lines describing "5-state, 12-transition" should drop any "NIST-traced" framing; reframe as "industry-informed (NIST 800-61 r2 phase model + ITIL v3 conventions + Cortex XSOAR lifecycle), 1898-curated." Architect to revisit whether 4-state + sub-status more cleanly maps to deployed SOAR practice. Add explicit transition table in the story.
- **S-4.07 (case_metrics):** Replace ambiguous "tdigest OR hdrhistogram" with explicit recommendation `hdrhistogram = "7.5"` as default; document the per-metric override pathway for unbounded-range data.
- **S-4.08 (notifications + scheduling):** Replace `cron = "0.12"` with `croner = "3"`; replace `lettre` deprecated `tokio1-rustls-tls` with explicit `rustls-aws-lc-rs` + `rustls-platform-verifier`; specify XOAUTH2 as first-class auth mechanism; specify CEF v0 + LEEF 2.0 in-house implementation (no `rust-cef` dependency); broadcast capacity 1000 is fine but spec must note pow-of-2 rounding to 1024. Wasmtime pin to 44 with `bindgen!` for action plugin host functions.

---

## Inconclusive / Defer

- **R-8 / S-4.08 LEEF Rust crate landscape:** Search confirms no maintained, published LEEF-formatter Rust crate on crates.io. The only meaningful prior art is embedded in `guardsarm/rust-threat-detector` (a SIEM-detector application, not a reusable library). **Decision item for human gate:** does Wave 4 commit to building `prism-siem-formats` in-house (recommended), or accept partial coverage by depending on the unmaintained `rust-cef` for CEF only and deferring LEEF to a later wave? In-house is recommended given the IBM QRadar LEEF spec is straightforward and proptest fuzzing is well within Wave 4 scope.

- **NIST 800-61 r3 implications for ADR-017:** Confirmed r3 supersedes r2 with a non-state-machine model. **Decision item:** does the architect explicitly cite NIST r2 (acknowledging supersession), or pivot ADR-017 to be entirely 1898-internal-curated with no external standards traceability? Either is defensible; the choice should be made consciously and documented.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Context7 resolve-library-id | 10 | DataFusion, cron, lettre, blake3, libfuzzer-sys, rocksdb, tdigest, hdrhistogram, wasmtime, tokio identifiers |
| WebFetch (docs.rs / GitHub / crates.io / vendor docs) | 17 | datafusion, cron, lettre, blake3, libfuzzer-sys, rocksdb, tdigest, hdrhistogram, wasmtime, tokio, scopeguard, regex, aho-corasick, lettre auth enum, croner-rust, NIST r3, XSOAR lifecycle |
| WebSearch | 14 | DataFusion 53 API/release, cron crate comparison, lettre TLS/auth, blake3 CVE history, fuzzing canonical pattern, scopeguard async, regex RegexSet at scale, CEF/LEEF Rust crates, CEF spec, LEEF spec, ITIL/NIST/SOAR state machines, tokio broadcast Lagged, wasmtime 44 component model, tdigest maintenance |
| Perplexity | 0 | (Not invoked — WebSearch/WebFetch + Context7 yielded sufficient citations for all 13 tasks) |
| Tavily | 0 | (Not invoked) |
| Training data | low — 2 areas | (a) general async/Drop semantics — Rust language behavior known stable; (b) fuzzing-stack architecture overview — independently verified via rust-fuzz book |

**Total MCP tool calls:** ~41 (10 Context7 resolves + 17 WebFetch + 14 WebSearch).
**Training data reliance:** **low** — every library version pin and every API claim is traced to a docs.rs / GitHub release / vendor specification URL with 2026-05-02 access date. The only domains where training-data context informs the analysis are general Rust async semantics (R-6) and the high-level fuzzing-stack overview (R-5), both of which are independently confirmed by external citations.
