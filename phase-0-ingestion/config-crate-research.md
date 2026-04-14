# Configuration Crate Evaluation for Prism

**Date:** 2026-04-13
**Type:** General (technology research)
**Status:** Complete
**Purpose:** Evaluate Rust configuration crate options for prism-config: layered config (CLI > env > TOML > defaults) with per-client sensor/credential/capability mappings.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Prism's Configuration Requirements](#2-prisms-configuration-requirements)
3. [Option 1: figment](#3-option-1-figment)
4. [Option 2: config-rs](#4-option-2-config-rs)
5. [Option 3: Direct serde + toml](#5-option-3-direct-serde--toml)
6. [Option 4: Other Crates](#6-option-4-other-crates)
7. [Comparison Matrix](#7-comparison-matrix)
8. [Recommendation](#8-recommendation)
9. [Reference Repo Precedent](#9-reference-repo-precedent)
10. [Research Methods](#10-research-methods)

---

## 1. Executive Summary

**Recommendation: Option 3 -- Direct `serde` + `toml` + `clap` with a thin custom layering module (~200 lines).**

Neither figment nor config-rs provides meaningful value for Prism's specific requirements. Prism's configuration model is fundamentally different from the web-server "overlay TOML files by environment" pattern that both libraries target. Prism has:

- A single TOML file (not environment-based overlays)
- Per-client nested sections with hierarchical capability merge
- Custom `_FILE` suffix secret resolution (neither library supports this natively)
- clap-driven CLI args (both libraries require manual serialization to inject clap values)
- Multi-error validation (both libraries fail on first error)

The custom approach costs ~200 lines of straightforward Rust code, eliminates two dependencies (with their transitive trees), gives full control over error reporting and `_FILE` resolution, and matches the Go pollers' proven patterns exactly.

---

## 2. Prism's Configuration Requirements

Extracted from product-brief.md, recovered-architecture.md, module-criticality.md, cross-repo-dependencies.md, and feature-flag-research.md:

| # | Requirement | Source |
|---|------------|--------|
| R1 | TOML as primary config format (single file, not environment overlays) | product-brief.md |
| R2 | Env var overrides with `_FILE` suffix priority for K8s secret mounts | All 4 Go pollers (cross-repo-dependencies.md Section 2.3) |
| R3 | CLI args via clap derive macros | recovered-architecture.md (prism-config) |
| R4 | Layered precedence: CLI > env > TOML > compiled defaults | product-brief.md, recovered-architecture.md |
| R5 | Strongly typed deserialization via serde | Convention reconciliation |
| R6 | Per-client sections: `[clients.acme]`, `[clients.beta]` | feature-flag-research.md Section 5 |
| R7 | Hierarchical capability merge: `defaults < client-specific` | ADR-012, feature-flag-research.md |
| R8 | Multi-error validation (report all problems) | Go pollers' `Validate()` (uses `errors.Join`) |
| R9 | `--dry-run` mode: validate + print redacted config | Go pollers' `ValidateConfig()` + `redactSecret()` |
| R10 | Cross-platform (Linux, macOS, Windows) | product-brief.md |
| R11 | 30+ env vars across sensor configs | module-criticality.md |
| R12 | No hot-reload needed (Prism is per-analyst stdio MCP, restarted per session) | product-brief.md ("single session per analyst") |

**Key observation:** Prism does NOT need hot-reload, environment-based file overlays (`dev.toml` / `prod.toml`), or remote config sources. These are the primary features that justify using figment or config-rs. Without them, the value proposition of both libraries collapses.

---

## 3. Option 1: figment

### Overview

figment is a layered configuration library created by Sergio Benitez (Rocket framework author). It uses a "Provider" abstraction where each config source implements `Provider`, and sources are merged via `Figment::merge()` or `Figment::join()`.

### Version & Maintenance (as of training data, May 2025)

| Attribute | Value |
|-----------|-------|
| Latest version | 0.10.x (never reached 1.0) |
| Last release | Active development as of mid-2024 |
| Repository | github.com/SergioBenitez/Figment |
| Primary user | Rocket web framework |
| Downloads | ~2M total on crates.io |
| MSRV | Unclear; generally recent stable |

**CONFIDENCE: LOW.** These version numbers are from training data. MCP tools were unavailable to verify current crates.io state. **Must verify before finalizing.**

### Strengths for Prism

1. **Clean layering API.** `Figment::from(defaults).merge(Toml::file("config.toml")).merge(Env::prefixed("PRISM_"))` is readable and declarative.

2. **Excellent merge semantics.** `merge()` = later source wins (higher priority). `join()` = earlier source wins (fill gaps). This matches Prism's CLI > env > TOML > defaults model.

3. **Good serde integration.** `figment.extract::<Config>()` deserializes directly into typed structs.

4. **TOML provider built in** (via feature flag `toml`).

5. **Env provider built in** with prefix filtering, separator mapping (`_` or `__` to `.`).

6. **Error reporting includes config key paths.** Errors like "key `clients.acme.sensors.crowdstrike.client_id`: missing" with source attribution (which provider supplied the value).

### Weaknesses for Prism

1. **No `_FILE` suffix support.** figment's Env provider reads env var values directly. It has no concept of "if `FOO_FILE` is set, read the file at that path as the value." This must be implemented as a custom Provider -- doable (~50 lines) but negates the "just use the library" value proposition.

2. **No native clap integration.** Must serialize clap args to a `figment::value::Map` or JSON and add as a provider. This is awkward because clap args include `None` for unset optionals, which would override TOML values. Requires filtering out `None` values before merging.

3. **Error reporting is single-error.** `extract()` returns the first deserialization error, not all errors. Multi-error validation (R8) must be done as a separate pass after extraction, which means figment's error reporting adds nothing over plain `toml::from_str()`.

4. **Pre-1.0 semver.** Breaking changes between 0.x versions are possible with no deprecation guarantees.

5. **Tightly coupled to Rocket's worldview.** The library works well for Rocket's `Rocket.toml` pattern (dev/release profiles). Prism's per-client config model is structurally different.

6. **Nested env var mapping is fragile.** Mapping `PRISM_CLIENTS__ACME__SENSORS__CROWDSTRIKE__CLIENT_ID` to `clients.acme.sensors.crowdstrike.client_id` requires careful separator configuration and may conflict with legitimate underscores in client names.

7. **Dependency weight.** Pulls in `serde`, `toml`, `uncased`, `pear` (figment's own parsing library), `version_check`. Not heavy, but not zero.

### figment Assessment for Each Requirement

| Req | Support | Notes |
|-----|---------|-------|
| R1 TOML | YES | `Toml::file()` provider |
| R2 `_FILE` suffix | NO | Custom Provider needed (~50 lines) |
| R3 clap CLI | PARTIAL | Manual serialization with None filtering |
| R4 Layered precedence | YES | `merge()` chain |
| R5 Typed serde | YES | `extract::<T>()` |
| R6 Per-client sections | YES | Nested struct deserialization works |
| R7 Hierarchical merge | NO | Must implement post-extraction |
| R8 Multi-error validation | NO | Returns first error only |
| R9 `--dry-run` | NO | Must implement separately |
| R10 Cross-platform | YES | Pure Rust |
| R11 30+ env vars | PARTIAL | Works but env var naming gets unwieldy for deeply nested keys |
| R12 No hot-reload | N/A | Not needed, not a factor |

---

## 4. Option 2: config-rs

### Overview

config-rs (crate name: `config`) is a layered configuration library using a Builder pattern. It was the de facto standard for Rust config loading for several years and has broad ecosystem adoption.

### Version & Maintenance (as of training data, May 2025)

| Attribute | Value |
|-----------|-------|
| Latest version | 0.15.x (never reached 1.0) |
| Last release | Sporadic; the gap between 0.13 and 0.14 was >2 years |
| Repository | github.com/mehcode/config-rs |
| Dependents | ~35K+ (large ecosystem) |
| Downloads | Very high on crates.io |
| MSRV | Generally recent stable |

**CONFIDENCE: LOW.** Must verify current version and maintenance status on crates.io. config-rs has historically had long gaps between releases.

### Strengths for Prism

1. **Very broad adoption.** 35K+ dependents means well-tested in production.

2. **Builder pattern for layered loading.** `Config::builder().add_source(File::with_name("config")).add_source(Environment::with_prefix("PRISM")).build()` is straightforward.

3. **TOML support** via feature flag.

4. **Environment variable support** with prefix and separator mapping.

5. **Adopted by axiathon** (our reference repo) for its configuration infrastructure. See Section 9 for details.

### Weaknesses for Prism

1. **No `_FILE` suffix support.** Same limitation as figment. Must implement separately.

2. **No native clap integration.** config-rs documentation suggests serializing clap args to JSON and adding as a source. Same None-filtering problem as figment.

3. **Error reporting is mediocre.** Errors are stringly-typed (`ConfigError` with string messages). No structured error paths. Multi-error validation (R8) is not supported.

4. **Merge semantics for nested maps are... surprising.** config-rs merges at the leaf level for maps, but replaces arrays entirely. For Prism's `[clients.acme.capabilities]` merging, this could cause unexpected behavior when a client config should inherit some capabilities from defaults but override others.

5. **Pre-1.0 semver with slow release cadence.** The 0.13 to 0.14 gap was approximately 2 years. Relying on this for a new project is a risk.

6. **Internal value representation.** config-rs deserializes all sources into an internal `Value` tree (similar to `serde_json::Value`) before deserializing to the target type. This adds a layer of indirection that makes error messages harder to understand -- errors reference positions in the internal tree, not in the source file.

7. **Dependency weight.** Similar to figment but pulls in `nom` (parser combinator library) for path expression parsing, which is unnecessary for Prism's use case.

### config-rs Assessment for Each Requirement

| Req | Support | Notes |
|-----|---------|-------|
| R1 TOML | YES | Via feature flag |
| R2 `_FILE` suffix | NO | Must implement separately |
| R3 clap CLI | PARTIAL | Manual JSON serialization |
| R4 Layered precedence | YES | Builder.add_source() chain |
| R5 Typed serde | YES | `config.try_deserialize::<T>()` |
| R6 Per-client sections | YES | Nested struct deserialization works |
| R7 Hierarchical merge | PARTIAL | Map merge exists but array replacement may surprise |
| R8 Multi-error validation | NO | Returns first error only |
| R9 `--dry-run` | NO | Must implement separately |
| R10 Cross-platform | YES | Pure Rust |
| R11 30+ env vars | PARTIAL | Same env var naming issues as figment |
| R12 No hot-reload | N/A | Not needed |

---

## 5. Option 3: Direct serde + toml

### Overview

Skip the config library entirely. Use `clap` (already decided) for CLI args, `toml` crate for TOML deserialization, and a thin custom module for env var resolution and `_FILE` suffix handling.

### Architecture

```rust
// prism-config/src/lib.rs -- ~200 lines total

/// 1. Parse CLI args via clap derive
let cli = Cli::parse();

/// 2. Load TOML file (path from CLI or default)
let toml_config: TomlConfig = load_toml(&cli.config_path)?;

/// 3. Merge: CLI > env > TOML > defaults
let config = Config::build(cli, toml_config)?;

/// 4. Resolve _FILE secrets
let config = config.resolve_secrets()?;

/// 5. Validate (multi-error)
config.validate()?;
```

### Strengths for Prism

1. **Full control over `_FILE` resolution.** Implement exactly the pattern from the Go pollers:
   ```rust
   fn resolve_secret(file_env: &str, direct_env: &str) -> Result<Option<String>> {
       if let Ok(path) = env::var(file_env) {
           let content = fs::read_to_string(path.trim())?;
           return Ok(Some(content.trim().to_string()));
       }
       if let Ok(val) = env::var(direct_env) {
           return Ok(Some(val.trim().to_string()));
       }
       Ok(None)
   }
   ```

2. **Full control over multi-error validation.** Implement the exact pattern from poller-cobra's `Validate()`:
   ```rust
   fn validate(&self) -> Result<(), Vec<ConfigError>> {
       let mut errors = Vec::new();
       if self.source.client_id.is_empty() {
           errors.push(ConfigError::Missing("source.client_id"));
       }
       // ... all validations ...
       if errors.is_empty() { Ok(()) } else { Err(errors) }
   }
   ```

3. **Native clap integration with no serialization gymnastics.** clap's `#[arg(env = "PRISM_...")]` attribute handles env var fallback natively. The precedence is: CLI arg > env var > default. TOML values are applied as defaults before clap parsing, or merged after.

4. **`--dry-run` is trivial.** No library abstraction to work around.

5. **Minimal dependency weight.** Only `toml` (which Prism needs anyway for per-client config) and `clap` (already decided). No additional crate.

6. **Per-client hierarchical merge is explicit code.** The Go pollers' pattern of "defaults < client-specific" capability merge is ~20 lines of explicit Rust code. With figment or config-rs, this merge logic lives outside the library anyway.

7. **Error messages are exactly what we want.** No intermediate `Value` tree, no provider attribution noise. Just "field `clients.acme.sensors.crowdstrike.client_id` is required."

8. **Matches Go pollers' proven patterns.** All 4 Go pollers used direct env var loading with `_FILE` priority, not a config library. The pattern is battle-tested in production.

### Weaknesses for Prism

1. **More code to write.** ~200 lines of config merging logic vs. ~20 lines of figment/config-rs wiring (but the custom `_FILE` Provider, clap integration, and multi-error validation code would add ~150 lines to either library approach anyway).

2. **No declarative merge specification.** The merge order is imperative code, not a declarative chain. This is less self-documenting than `Figment::from(...).merge(...).merge(...)`.

3. **Must handle env var prefix filtering manually.** clap's `#[arg(env = "PRISM_...")]` handles this per-field, but there's no bulk `PRISM_*` prefix scanning. This is fine for Prism since each field explicitly declares its env var.

### Direct Approach Assessment for Each Requirement

| Req | Support | Notes |
|-----|---------|-------|
| R1 TOML | YES | `toml` crate |
| R2 `_FILE` suffix | YES | Direct implementation, proven Go pattern |
| R3 clap CLI | YES | Native, no serialization needed |
| R4 Layered precedence | YES | clap handles CLI > env natively; TOML loaded separately |
| R5 Typed serde | YES | `toml::from_str::<Config>()` |
| R6 Per-client sections | YES | `HashMap<String, ClientConfig>` in serde struct |
| R7 Hierarchical merge | YES | Explicit merge function (~20 lines) |
| R8 Multi-error validation | YES | Explicit `Vec<ConfigError>` collection |
| R9 `--dry-run` | YES | Trivial to implement |
| R10 Cross-platform | YES | Standard library only |
| R11 30+ env vars | YES | clap `#[arg(env = "...")]` per field |
| R12 No hot-reload | N/A | Not needed |

---

## 6. Option 4: Other Crates

### 6.1 confique

A newer config crate focused on strongly typed configuration with TOML and env var support. Generates documentation and env var lists from config struct definitions.

| Attribute | Assessment |
|-----------|-----------|
| Maturity | Relatively new (post-2023). Limited adoption. |
| Layering | Supports TOML + env vars but less flexible than figment/config-rs |
| `_FILE` support | No |
| clap integration | No |
| Multi-error | No |
| Verdict | **Not a contender.** Less mature, same gaps. |

### 6.2 serde_env

Deserializes environment variables directly into serde structs. Could complement the direct approach.

| Attribute | Assessment |
|-----------|-----------|
| Maturity | Small, niche crate |
| Value add | Minimal -- clap's `#[arg(env)]` already handles this |
| Verdict | **Not needed.** clap covers env var deserialization. |

### 6.3 envy

Another env-var-to-struct crate. Supports nested structs via `_` separator.

| Attribute | Assessment |
|-----------|-----------|
| Maturity | Established but low-activity |
| Value add | Same as serde_env -- redundant with clap |
| Verdict | **Not needed.** |

---

## 7. Comparison Matrix

| Criterion | figment | config-rs | Direct serde+toml | Weight |
|-----------|---------|-----------|-------------------|--------|
| `_FILE` secret resolution | Custom Provider (~50 lines) | Custom source (~50 lines) | Direct (~15 lines) | HIGH |
| clap integration | Manual serialize + None filter | Manual serialize + None filter | Native `#[arg(env)]` | HIGH |
| Multi-error validation | Not supported, separate pass | Not supported, separate pass | Direct `Vec<ConfigError>` | HIGH |
| Per-client hierarchical merge | Post-extraction code | Post-extraction code | Explicit merge code | MEDIUM |
| TOML deserialization | Built-in provider | Built-in (feature flag) | `toml` crate direct | LOW |
| Env var overrides | Built-in provider | Built-in | clap `#[arg(env)]` | LOW |
| Error message quality | Good (key paths + source) | Mediocre (string messages) | Full control | MEDIUM |
| `--dry-run` mode | Manual implementation | Manual implementation | Manual implementation | LOW |
| Dependency count | +4-5 transitive deps | +5-6 transitive deps (incl. nom) | +0 (toml + clap already needed) | MEDIUM |
| Semver stability | 0.x (pre-1.0) | 0.x (pre-1.0, slow releases) | N/A (own code) | MEDIUM |
| Learning curve | figment-specific API | config-rs-specific API | Standard Rust (serde, clap) | LOW |
| Code to write | ~80 lines (wiring + custom providers) | ~80 lines (wiring + custom sources) | ~200 lines (all explicit) | LOW |
| **Net custom code needed** | ~200 lines (wiring + providers + validation) | ~200 lines (wiring + sources + validation) | ~200 lines | -- |

**Key insight:** The "net custom code" is approximately the same for all three options. figment and config-rs save ~20 lines on the merge chain but require ~20+ lines of adapter code for clap integration and ~50 lines for the `_FILE` custom provider. The validation pass is identical across all three.

---

## 8. Recommendation

### Primary: Direct serde + toml + clap (Option 3)

**Rationale:**

1. **The custom code cost is the same.** All three options require ~200 lines of prism-specific code. The library options just redistribute where those lines live (custom Providers vs. explicit merge functions).

2. **Zero additional dependencies.** `toml` and `clap` are already required. No config library adds zero-cost features.

3. **`_FILE` resolution is a first-class citizen, not an afterthought.** This is Prism's most distinctive config requirement (inherited from all 4 Go pollers) and must be correct, auditable, and testable. A custom Provider buried inside a library's abstraction is harder to test than a standalone function.

4. **clap integration is native.** No serialization, no None-filtering, no impedance mismatch. `#[derive(Parser)]` with `#[arg(env = "PRISM_...")]` is the idiomatic Rust pattern.

5. **Multi-error validation is explicit.** The Go pollers' `errors.Join()` pattern translates directly to `Vec<ConfigError>`. No library workaround needed.

6. **No semver risk.** Own code, own stability guarantees. No surprise breakage from 0.x library updates.

7. **Prism does not need the features these libraries provide.** No environment overlays (dev.toml/prod.toml), no remote config sources, no hot-reload. Prism has one TOML file, one env var namespace, and one CLI.

### If the team prefers a library: figment (secondary choice)

If the team values the declarative `merge()` chain over the direct approach, figment is the better choice over config-rs:

- Better error messages (key paths with source attribution)
- Cleaner API (Providers vs. Builder + Sources)
- More actively maintained
- Better documentation

config-rs is not recommended despite axiathon's adoption, because:
- axiathon's requirements are fundamentally different (multi-environment server deployment with hot-reload, per-tenant DB-backed overrides, environment overlay files)
- config-rs's error reporting is mediocre
- config-rs's release cadence is unreliable

### Implementation Sketch (Direct Approach)

```
prism-config/
  src/
    lib.rs          -- re-exports
    cli.rs          -- clap derive struct (~80 lines)
    toml_config.rs  -- serde TOML structs (~100 lines)
    secrets.rs      -- _FILE resolution (~30 lines)
    merge.rs        -- CLI > env > TOML > defaults (~40 lines)
    validate.rs     -- multi-error validation (~80 lines)
    dry_run.rs      -- redacted config printing (~40 lines)
    client.rs       -- per-client config + capability merge (~60 lines)
    error.rs        -- ConfigError enum (~30 lines)
```

Total: ~460 lines of focused, testable Rust code with zero external config dependencies.

---

## 9. Reference Repo Precedent

### axiathon: Chose config-rs 0.15

**Context:** axiathon is a multi-tenant security analytics platform with a long-running server process, environment-based deployment (dev/staging/production), hot-reload requirements, and per-tenant overrides stored in PostgreSQL.

**Why config-rs made sense for axiathon:**
- Environment overlay files (`default.toml` -> `{env}.toml` -> `local.toml`) -- config-rs's core use case
- Hot-reload via `arc-swap` + `notify` -- config-rs works well as the initial loader
- Server process that runs continuously -- worth the setup cost
- 35K+ dependents gave confidence in production readiness

**Why axiathon's choice does NOT transfer to Prism:**
- Prism has a single TOML file, not environment overlays
- Prism has no hot-reload (per-analyst stdio MCP, restarted per session)
- Prism's `_FILE` secret resolution is its most critical config feature -- config-rs has no support
- Prism's per-client capability merge is custom logic regardless of config crate

### Go pollers: Direct env var loading (no config library)

All 4 Go pollers (poller-cobra, poller-express, poller-bear, poller-coaster) used direct `os.Getenv()` calls with manual `_FILE` priority logic. No Go config library (viper, envconfig, etc.) was used. The pattern:

1. `DefaultConfig()` -- compiled defaults
2. `LoadFromEnvironment(cfg)` -- env vars override defaults, `_FILE` takes priority
3. `cfg.Validate()` -- multi-error aggregation via `errors.Join()`
4. `ValidateConfig()` -- dry-run: validate + print redacted values

This is exactly the direct approach (Option 3) in Go. It worked in production across all 4 pollers. The Rust translation is straightforward.

### axiathon KOTS research: Mentioned figment

The axiathon KOTS research document (technical-kots-kubernetes-off-the-shelf-research-2026-01-30.md) includes a code snippet using figment for Vault-injected secret loading. However, axiathon's actual implementation decision was config-rs, not figment. The figment mention was in a research appendix showing an alternative pattern for Vault Agent integration.

---

## 10. Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Context7 | 0 (denied) | Was attempted for figment and config-rs docs |
| WebSearch | 0 (denied) | Was attempted for crates.io version verification |
| WebFetch | 0 (denied) | Was attempted for crates.io API |
| Local file reads | 12 | Reference repos (axiathon config architecture, poller-cobra config.go, recovered-architecture.md, module-criticality.md, cross-repo-dependencies.md, feature-flag-research.md, product-brief.md, project-context.md) |
| Local grep searches | 6 | Finding config crate references across all repos and factory docs |
| Training data | 4 areas | figment API/features, config-rs API/features, confique/envy/serde_env existence, dependency weight estimates |

**Total MCP tool calls:** 0 (all denied)
**Training data reliance:** HIGH -- Version numbers, maintenance status, dependency counts, and API details for figment and config-rs are from training data (knowledge cutoff May 2025). These MUST be verified against crates.io before finalizing the decision. The recommendation (Option 3: direct approach) is robust to version changes since it does not depend on either library.

### Verification Needed Before Finalizing

| Item | How to Verify | Impact on Recommendation |
|------|--------------|-------------------------|
| figment latest version | `cargo search figment` or crates.io | Low -- recommendation is Option 3 |
| figment 1.0 release? | crates.io changelog | Medium -- 1.0 would reduce semver risk |
| config-rs latest version | `cargo search config` or crates.io | Low -- recommendation is Option 3 |
| config-rs maintenance activity | GitHub pulse | Low -- recommendation is Option 3 |
| New config crate since May 2025 | `cargo search configuration` | Medium -- a new crate with native `_FILE` support would change the calculus |
