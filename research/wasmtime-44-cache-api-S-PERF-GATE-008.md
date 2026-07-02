# Research: wasmtime 44 Cache / CacheConfig API — S-PERF-GATE-008 SID-1 Test Design

- **Story:** S-PERF-GATE-008 (wasmtime compilation cache in `prism-spec-engine`)
- **Date:** 2026-07-01
- **Type:** general (remove-uncertainty pass)
- **Pin under evaluation:** `wasmtime = "44"`, features `["component-model", "cache"]`
- **Objective:** Confirm exact API signatures for `Cache` / `CacheConfig` and — most importantly — the deterministic, no-external-service mechanism to force `Cache::new(...)` to return `Err` for the SID-1 degradable-path unit test.

All API-level facts below are verified against the **v44.0.0 source tree** on GitHub
(`crates/cache/src/lib.rs`, `crates/cache/src/config.rs`) unless flagged `INFERRED` or `INCONCLUSIVE`.

---

## 1. `Cache` / `CacheConfig` API (wasmtime 44) — VERIFIED

### `Cache::new` — exact signature and error type

Verified from `crates/cache/src/lib.rs` @ `v44.0.0`:

```rust
pub fn new(mut config: CacheConfig) -> Result<Self> {
    config.validate()?;          // <-- eager validation, this is the seam SID-1 needs
    Ok(Self {
        worker: Worker::start_new(&config),
        config,
        state: Default::default(),
    })
}

pub fn from_file(path: Option<&Path>) -> Result<Self> {
    let config = CacheConfig::from_file(path)?;
    Self::new(config)
}
```

- **Signature:** `pub fn new(config: CacheConfig) -> Result<Cache, wasmtime::Error>`.
- **Error type `E` = `wasmtime::Error`.** Under the default `std` build (prism uses std), `wasmtime::Error`
  is a re-export/alias of `anyhow::Error`. For test purposes it behaves as an `anyhow` error:
  `.is_err()`, `.unwrap_err()`, and `.to_string()` all work; no bespoke error enum to match on.
  (Confirmed via docs.rs rendering `Result<Cache, Error>` and Context7's `wasmtime::error::format_err!`
  doc which treats `wasmtime::Error` as `anyhow`-compatible / convertible from foreign errors.)
- **KEY FACT (resolves earlier ambiguity):** `Cache::new` **eagerly** calls `config.validate()?`.
  `validate()` is invoked in **both** the `from_file` path *and* the programmatic `Cache::new(CacheConfig)`
  path. So a mis-built `CacheConfig` makes `Cache::new` return `Err` at construction time — no deferral
  to runtime. This is the seam the SID-1 test exploits.
- `Cache` derives/impls `Clone + Debug + Send + Sync`.

### `CacheConfig` — constructor + relevant builders (VERIFIED)

```rust
pub fn new() -> CacheConfig                                  // Self::default(); working defaults
pub fn from_file(path: Option<&Path>) -> Result<CacheConfig> // None => system default TOML path

// builder methods (return &mut Self, chainable):
pub fn with_directory(&mut self, directory: impl Into<PathBuf>) -> &mut Self  // MUST be absolute
pub fn with_baseline_compression_level(&mut self, level: i32) -> &mut Self    // valid 0..=21
pub fn with_optimized_compression_level(&mut self, level: i32) -> &mut Self   // valid 0..=21, >= baseline
pub fn with_file_count_limit_percent_if_deleting(&mut self, percent: u8) -> &mut Self       // <= 100
pub fn with_files_total_size_limit_percent_if_deleting(&mut self, percent: u8) -> &mut Self // <= 100
// (plus worker-queue / size / interval / clock-drift setters — not relevant to forced failure)
```

`CacheConfig::new()` yields a working default (no directory set => OS default dir resolved at validate time).

### `Config::cache` — correct attach method (VERIFIED)

```rust
pub fn cache(&mut self, cache: Option<Cache>) -> &mut Self
```

- `Config::cache(Some(cache))` is the **current, correct** way to attach a cache in wasmtime 44.
- The older TOML-file methods `Config::cache_config_load(path)` / `cache_config_load_default()` are the
  legacy API. **INCONCLUSIVE (non-blocking):** I could not confirm from the 44.0.0 changelog whether the
  legacy methods are `#[deprecated]`, removed, or still present in exactly 44.0.0. It does not matter for
  this story — the prototype already compiles green using `Config::cache(Option<Cache>)`, which is the
  verified-correct modern path. Do **not** use `cache_config_load*`.

---

## 2. Forcing `Cache::new(...)` to return `Err` deterministically — VERIFIED, RECOMMENDED

`Cache::new` -> `config.validate()`. Verified order of checks inside `validate()`
(`crates/cache/src/config.rs` @ `v44.0.0`):

1. `validate_directory_or_default()`  ← **runs FIRST**
2. `validate_worker_event_queue_size()` (warning only)
3. `validate_baseline_compression_level()` — range `0..=21`
4. `validate_optimized_compression_level()` — range `0..=21` AND `optimized >= baseline`
5. `validate_file_count_limit_percent_if_deleting()` — `<= 100`
6. `validate_files_total_size_limit_percent_if_deleting()` — `<= 100`

Inside `validate_directory_or_default()`, verified:

```rust
if !cache_dir.is_absolute() {
    bail!("Cache directory path has to be absolute, path: {}", cache_dir.display());
}
```

This `is_absolute()` check runs **before** any `fs::create_dir_all()` / `fs::canonicalize()`.

### RECOMMENDED forced-failure mechanism (primary)

Set a **relative (non-absolute) cache directory**. It is check #1 and short-circuits via `bail!`
**before touching the filesystem** — so it is fully deterministic, platform-independent (macOS + Linux +
Windows), leaves **zero filesystem side effects**, and needs no external service, no temp dirs, no
permission juggling:

```rust
let mut cfg = wasmtime::CacheConfig::new();
cfg.with_directory("relative/not/absolute");   // relative => is_absolute() == false
let err = wasmtime::Cache::new(cfg).unwrap_err();
assert!(err.to_string().contains("has to be absolute"));
```

This exercises the degradable `match` arm in `apply_wasmtime_cache` (the `Err(_) =>` branch) with the
cleanest possible input. **Use this as the SID-1 forced-failure driver.**

### Alternate forced-failure mechanisms (backups, ranked)

- **B. Out-of-range compression level** — `cfg.with_baseline_compression_level(99)` (valid `0..=21`).
  Purely in-memory validation. CAVEAT: this is check #3, so check #1 (`validate_directory_or_default`)
  runs first; if no directory is set it will try to resolve+`create_dir_all` the OS default dir (a
  filesystem side effect) before reaching the compression check. To keep it side-effect-free you'd also
  have to set a valid absolute writable dir. This makes it strictly worse than the relative-path option.
- **C. Path-is-a-file / unwritable dir** — point `with_directory` at an absolute path that is an existing
  regular file (or an unwritable location); `create_dir_all` then fails. Deterministic but requires a
  tempfile fixture and has OS-specific permission semantics (e.g. root ignores mode bits, Windows differs).
  More fragile than the relative-path option.
- **D. Injected-Result seam** — if the extracted helper is shaped as
  `apply_wasmtime_cache(config: &mut Config, cache_result: Result<Cache, E>)`, the test can inject a
  synthetic `Err` directly and never call `Cache::new`. Cleanest for pure unit isolation of the `match`
  arm, but does NOT prove wasmtime's real API can produce the `Err`. Prefer option A (real `Cache::new`
  error) for the SID-1 "actually exercise the production code path" requirement; option D is a fine
  complement if the helper signature already takes a `Result`.

**Recommendation:** primary = **option A (relative directory path)**. It exercises the real
`Cache::new` error path, is deterministic on all platforms, and has no filesystem side effects. Assert on
`is_err()` (and optionally the `"has to be absolute"` substring; treat the substring as best-effort since
error text is not a stability contract).

---

## 3. Default cache directory behavior — VERIFIED (design) + INFERRED (exact paths)

- When no directory is set, `validate_directory_or_default()` resolves an OS-default absolute path and
  `create_dir_all`s it. Cache-config file default path is documented as
  `$HOME/.config/wasmtime/config.toml` on Unix (docs.rs / docs.wasmtime.dev). The cache **data** dir
  follows platform cache conventions.
- **Exact default data-dir paths (INFERRED, not read verbatim from 44.0.0 source):** Linux
  `$XDG_CACHE_HOME` / `~/.cache/...`; macOS `~/Library/Caches/...`. Direction confirmed; exact leaf paths
  not verbatim-verified for 44.0.0.
- **Read-only HOME:** YES, default-dir resolution CAN fail. With no directory set, validate resolves the
  HOME-derived default and calls `create_dir_all`; on a read-only HOME this returns an I/O `Err` (surfaced
  as `wasmtime::Error`). This is exactly why the SID-1 degradable path exists and why prism must handle
  `Cache::new(...) == Err` gracefully rather than `unwrap()`. (Note: this is also a *second* real-world
  forced-failure vector, but it is environment-dependent and non-deterministic in CI — do not use it as
  the test driver; use option A.)

---

## Bottom line for SID-1

1. `Cache::new(CacheConfig) -> Result<Cache, wasmtime::Error>` and it **validates eagerly** (`config.validate()?`). VERIFIED.
2. Force the `Err` with a **relative cache directory** — `CacheConfig::with_directory("relative/path")` then
   `Cache::new(cfg)` returns `Err` before any filesystem I/O. Deterministic, cross-platform, side-effect-free. VERIFIED.
3. `Config::cache(Some(cache))` is the correct attach method; avoid `cache_config_load*`. VERIFIED.
4. Read-only HOME is a genuine real-world `Err` source (justifies the degradable path) but is unsuitable as
   a CI test driver — prefer the relative-path driver.

---

## Sources

- `crates/cache/src/lib.rs` @ tag `v44.0.0` — `Cache::new` / `Cache::from_file` bodies (via GitHub blob fetch, 2026-07-01).
- `crates/cache/src/config.rs` @ tag `v44.0.0` — `validate()` check order, `validate_directory_or_default()` `is_absolute` bail, builder signatures (via GitHub blob fetch, 2026-07-01).
- docs.rs `wasmtime/44.0.0` — `struct.Cache.html`, `struct.CacheConfig.html`, `struct.Config.html` (method list + `Result<Cache, Error>` rendering).
- Context7 `/websites/rs_wasmtime` — `CacheConfig::with_directory` "must be absolute" doc; `wasmtime::error::format_err!` (error-type / anyhow relationship).
- Perplexity deep-research (sonar-deep-research) — API-evolution history (cache_config_load* -> Config::cache), default-dir + read-only-HOME behavior; several 44.0.0-exact points flagged INFERRED there and re-verified above against v44.0.0 source.

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | API history, error-type semantics, default-dir + read-only-HOME failure modes (reasoning_effort=high) |
| Context7 resolve-library-id | 1 | Locate wasmtime library IDs |
| Context7 query-docs | 1 | `with_directory` absolute-path rule + error-type doc |
| WebFetch | 6 | docs.rs 44.0.0 Cache/CacheConfig/Config pages + v44.0.0 GitHub source (lib.rs, config.rs x2, type.Error 404) |
| Training data | 1 area | `wasmtime::Error`↔`anyhow::Error` std-alias relationship (cross-checked against Context7 format_err doc) |

**Total MCP tool calls:** 3 (1 perplexity_research + 2 Context7). Plus 6 WebFetch (source-of-truth GitHub + docs.rs).
**Training data reliance:** low — every load-bearing API fact was read from v44.0.0 source or docs.rs; only the well-established `wasmtime::Error = anyhow::Error` (std) relationship leaned on model knowledge and was cross-checked.
