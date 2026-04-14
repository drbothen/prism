# Pass 5 Deep: Convention & Pattern Catalog -- Round 1

**Project:** Axiathon
**Pass:** 5 (Conventions)
**Round:** 1
**Date:** 2026-04-13

---

## Purpose

Deepen the broad sweep's convention catalog by verifying pattern consistency across the codebase, documenting the full tooling enforcement chain, identifying conventions that diverge between production and spike, and cataloging anti-patterns discovered during Tier 1/2 analysis.

---

## 1. Code Formatting Conventions

### 1.1 Rustfmt Configuration (rustfmt.toml)

```toml
edition = "2024"
group_imports = "StdExternalCrate"
trailing_comma = "Vertical"
```

**Enforcement chain:**
- lefthook pre-commit hook: `cargo +nightly fmt --all -- --check`
- CI job: `check-fmt` runs same command
- Nightly rustfmt required (explicit guard in justfile `_require-nightly`)

**NEW finding:** `group_imports = "StdExternalCrate"` means imports are grouped into three sections:
1. `std::*` imports
2. External crate imports
3. Crate-local imports

This is enforced by nightly rustfmt and verified in CI. Every file must follow this order.

### 1.2 TOML Formatting (.taplo.toml)

```toml
exclude = ["spike/**", "target/**"]
[formatting]
allowed_blank_lines = 1
column_width = 80
indent_string = "  "
reorder_keys = true
```

**KEY FINDING:** spike/ is EXCLUDED from TOML formatting enforcement. Only production TOML files are formatted.

### 1.3 Spell Checking (.typos.toml)

Custom word list includes project-specific terms: `axiathon`, `ocsf`, `ot`, acronym substrings (BA, PN, SIE), parser test typos (SELCT), and crate names (Ratatui, Hashi).

**KEY FINDING:** `spike/` is EXCLUDED from spell checking via `extend-exclude`.

---

## 2. Module Organization Conventions

### 2.1 lib.rs Barrel Export Pattern (PERVASIVE in production)

Every production crate follows this exact pattern:

```rust
//! Crate-level doc comment describing purpose
#![forbid(unsafe_code)]

pub mod module_a;
pub mod module_b;
// ...

pub use module_a::{Type1, Type2};
pub use module_b::{Type3, Type4};
```

Rules:
- lib.rs is ONLY re-exports -- no implementation code
- Doc comment on first line
- `#![forbid(unsafe_code)]` on second line
- `pub mod` declarations
- `pub use` re-exports for key types

**Consistency:** 100% in production crates (8/8). Spike crates follow the pattern for pub mod but are less consistent on re-exports.

### 2.2 Module File Layout Convention (.claude/rules/rust.md)

```
axiathon-{crate}/
  src/
    lib.rs          # Public API re-exports only
    error.rs        # Crate-specific error types
    config.rs       # Configuration types
    {domain}/       # Feature-specific modules
```

**Consistency:** Both production implemented crates (core, query) follow this exactly. Spike crates follow it loosely -- some have error.rs and config.rs patterns, others embed them in other files.

### 2.3 Test File Organization

**Production pattern (integration tests in tests/ dir):**
- `tests/core_types_integration.rs` -- integration tests by feature area
- `tests/property_*.rs` -- property-based tests (proptest)
- `tests/snapshot_*.rs` -- snapshot tests (insta)
- `tests/parser_test.rs` -- per-module integration tests

**Spike pattern (inline #[cfg(test)] modules):**
- Every spike source file has `#[cfg(test)] mod tests { ... }` at the bottom
- 49 inline test modules across the spike codebase
- 4 separate integration test files in tests/ directories (storage)
- 2 benchmark files in benches/ directories (detection)

**Convention divergence:** Production uses separate test files; spike uses inline test modules.

---

## 3. Naming Conventions (Verified)

### 3.1 Type Naming

| Element | Convention | Consistency | Examples |
|---------|-----------|-------------|---------|
| Structs | PascalCase | 100% | `AxiathonEvent`, `TenantContext`, `StorageWriter` |
| Enums | PascalCase | 100% | `FilterExpr`, `CompareOp`, `CaseStatus` |
| Enum variants | PascalCase | 100% | `SingleEvent`, `Correlation`, `Sequence` |
| Traits | PascalCase, adjective/noun | 100% | `TenantScoped`, `ConnectorPlugin`, `EventEnricher` |
| Type aliases | PascalCase | 100% | `PluginConfig = serde_json::Value` |

### 3.2 Function and Method Naming

| Pattern | Convention | Examples |
|---------|-----------|---------|
| Constructors | `new()`, `new_unchecked()` | `TenantId::new()`, `EventId::new()` |
| Fallible constructors | `new() -> Result<Self>` | `TenantId::new()`, `FieldRef::new()` |
| Conversion | `from_*()`, `to_*()`, `as_*()` | `from_event()`, `to_string_repr()`, `as_str()` |
| Predicate | `is_*()`, `has_*()`, `can_*()` | `has_array_index()`, `can_parse()`, `can_transition_to()` |
| Builder | `with_*()` | `with_meta()`, `with_on_commit()`, `with_context()` |
| Factory | `create()` | `NativePluginFactory::create()` |

### 3.3 Constant Naming

| Pattern | Convention | Examples |
|---------|-----------|---------|
| Module-level constants | SCREAMING_SNAKE_CASE | `MAX_QUERY_LENGTH`, `MAX_NESTING_DEPTH`, `NAMESPACE` |
| Embedded constants | inline in Default impl | `buffer_size: 1000`, `flush_interval: Duration::from_secs(5)` |

**Inconsistency:** Some limits use named constants (MAX_QUERY_LENGTH in parser.rs), while others embed magic numbers in Default impls (buffer_size: 1000 in WriterConfig). The production code is more consistent about named constants.

### 3.4 File/Directory Naming

| Element | Convention | Examples |
|---------|-----------|---------|
| Rust source files | snake_case.rs | `query_types.rs`, `tenant_filter.rs` |
| Test files | `*_test.rs` or `test_*.rs` | `parser_test.rs`, `test_fixtures.rs` |
| Property tests | `property_*.rs` | `property_fieldref.rs` |
| Snapshot tests | `snapshot_*.rs` | `snapshot_types.rs` |
| Benchmark files | `*.rs` in benches/ | `detection_stateless.rs` |
| Crate directories | kebab-case | `axiathon-core`, `axiathon-plugin-sdk` |
| Detection rules | kebab-case.axd | `brute-force.axd` (from benchmark references) |

---

## 4. Design Patterns (Refined)

### 4.1 Validated Constructor / Newtype Pattern (PERVASIVE)

**Status:** Applied to 4+ types in production, 2+ in spike
**Mechanism:** `new()` validates at trust boundaries, `new_unchecked()` bypasses for trusted sources
**Applied to:** TenantId, EventId, AlertId, FieldRef, PluginId
**Consistency:** 100% for ID types. Other types (WriterConfig, CompactionConfig, etc.) use public fields with Default -- no validation.

### 4.2 Non-Exhaustive Enum Pattern (PERVASIVE)

**Status:** 29 `#[non_exhaustive]` annotations across 10 files
**Distribution:** 
- Production: 16 instances (core: 4, query: 12)
- Spike: 13 instances (detection: 8, core: 2, event: 1, error: 1, aliases: 1)

**Convention for when to apply:**
- APPLY: Enums that will gain variants over time (operations, AST nodes, error types, plugin kinds)
- SKIP: Semantically closed enums (SortDirection: Asc/Desc, FieldsMode: Include/Exclude)

**Consistency:** Strong. The production codebase follows this rule without exception.

### 4.3 Dual-Context Pattern (CONSISTENT)

```rust
// User operations
pub struct TenantContext { tenant_id, user_id, roles, permissions, trace_id }

// Background jobs
pub struct SystemContext { tenant_id, system_component, trace_id }

// Accepted by any function needing tenant scope
pub trait TenantScoped: Send + Sync {
    fn tenant_id(&self) -> &TenantId;
    fn trace_id(&self) -> &str;
}
```

**Consistency:** Defined in production axiathon-core. The spike uses a simplified TenantContext (pub fields, no user_id/roles/permissions, no TenantScoped trait). The spike's API middleware creates TenantContext from just tenant_id (no user identity).

### 4.4 FromRef State Decomposition Pattern (spike API -- NEW)

AppState decomposes into domain-scoped substates via Axum's `FromRef`:
```rust
impl FromRef<AppState> for DetectionServices { ... }
impl FromRef<AppState> for StorageServices { ... }
impl FromRef<AppState> for PluginServices { ... }
impl FromRef<AppState> for CredentialServices { ... }
```

Route handlers extract only the substate they need, preventing coupling to the full AppState.

### 4.5 Per-Tenant Engine Map Pattern (spike detection -- NEW)

```rust
pub rule_engines: Arc<RwLock<HashMap<TenantId, RuleEngine>>>
pub correlation_engines: Arc<RwLock<HashMap<TenantId, CorrelationState>>>
pub sequence_engines: Arc<RwLock<HashMap<TenantId, SequenceState>>>
```

Every stateful detection subsystem is isolated per-tenant via separate engine instances in a tenant-keyed HashMap. This prevents noisy-neighbor effects and ensures rule changes for one tenant don't affect others.

### 4.6 Builder-with-Notify Pattern (spike storage -- NEW)

```rust
CompactionTask::new(config, catalog, table_ident)
    .with_on_commit(catalog_changed.clone())
```

Background tasks accept an optional `Arc<Notify>` that fires after each successful operation. This allows the query engine to refresh its table providers without polling. The pattern is used for both compaction and GC tasks.

### 4.7 SECURITY Comment Convention (PERVASIVE)

Security-relevant decisions are annotated with structured comments:
```rust
/// **SECURITY:** Display output is for internal logging only.
/// **SECURITY(CWE-209):** ...
/// **SECURITY(CWE-798):** Hardcoded passphrase...
/// **SECURITY(OWASP A01:2021):** Admin endpoints...
```

Format: `SECURITY(identifier):` followed by explanation.

**Consistency:** 100% in production code. Present in spike code but fewer call sites.

### 4.8 Comment-Preserving Preprocessor Pattern (AxiQL parser)

The AxiQL parser strips comments by replacing comment characters with spaces (not removing them), preserving byte offsets for error span reporting. This is a deliberate design decision to support error recovery with accurate source locations.

---

## 5. Error Handling Conventions

### 5.1 Error Type Pattern

```rust
// Per-crate error enum with thiserror
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AxiathonError {
    #[error("domain-specific message: {0}")]
    Variant(String),
    
    #[error(transparent)]
    Wrapped(#[from] OtherError),
    
    #[error("structured: {resource} ({id})")]
    Structured { resource: String, id: String },
}

// Per-crate Result alias
pub type Result<T> = std::result::Result<T, AxiathonError>;
```

**Consistency:** Production crates follow this exactly. Spike adds `Other(anyhow::Error)` catch-all variant.

### 5.2 From Conversions

Production: `From<std::io::Error>` and `From<serde_json::Error>` via `#[from]`
Spike: Additional `From<arrow::error::ArrowError>`, `From<serde_json::Error>`, `From<anyhow::Error>`

### 5.3 Error-to-API Sanitization

The production code has SECURITY comments mandating sanitization but no implementation yet. The spike routes manually construct error responses -- no centralized error middleware.

---

## 6. Testing Conventions

### 6.1 Test Naming

Convention from SOUL.md #8: `{subject}_{action}_{expected_outcome}()`

Examples verified from source:
- `tenant_id_new_rejects_empty()`
- `parse_filter_has_existence()`
- `parse_filter_wildcard_ordering_op_rejects()`
- `case_status_can_transition_to_valid()`
- `alert_from_single_event_creates_alert()`
- `filter_injected_without_user_filter()`
- `wrong_tenant_rejected()`
- `or_bypass_prevented()`

**Consistency:** High in both production and spike. Some spike tests use shorter names (`test_encrypt_decrypt`) but the majority follow the convention.

### 6.2 Test Strategy by Type

| Type | Framework | Location | Purpose |
|------|-----------|----------|---------|
| Unit | #[test] | #[cfg(test)] inline or tests/*.rs | Single function behavior |
| Property | proptest | tests/property_*.rs | Invariant verification over random inputs |
| Snapshot | insta | tests/snapshot_*.rs | Regression detection for serialization |
| Integration | #[tokio::test] | tests/*.rs (spike/storage) | End-to-end pipeline validation |
| Benchmark | criterion | benches/*.rs | Performance measurement |

### 6.3 Test Data Patterns

- `make_event()` / `make_auth_event()` helper functions in test modules
- Direct DynamicMessage construction for OCSF events
- `tempfile::TempDir` for storage tests (Iceberg catalog)
- Hard-coded tenant IDs: "acme-corp", "globex-inc", "bench-tenant"
- Hard-coded IP ranges: "10.0.0.0/8", "203.0.113.0/24"

---

## 7. Dependency Management Conventions

### 7.1 Workspace-Level Dependencies

All shared dependencies declared in root `Cargo.toml` `[workspace.dependencies]`. Individual crates reference via `dep = { workspace = true }`.

### 7.2 Action Pinning

CI workflows pin actions to commit SHAs, not version tags:
```yaml
uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6
uses: Swatinem/rust-cache@e18b497796c12c097a38f9edb9d0641fb99eee32 # v2
uses: step-security/harden-runner@fa2e9d605c4eeb9fcad4c99c224cee0c6c7f3594 # v2.16.0
```

### 7.3 Git Flow Branching

From CONTRIBUTING.md:
- Branch from `develop`, PRs target `develop`
- Never commit directly to `main`
- Naming: `feature/story-X.X-desc`, `fix/issue-123-desc`, `backlog/X-XX-desc`, `spike/XXX-topic`
- Conventional commits enforced by lefthook

---

## 8. Anti-Patterns and Technical Debt (Expanded)

### 8.1 Production Anti-Patterns

| # | Pattern | Impact | Count |
|---|---------|--------|-------|
| 1 | QueryConfig values are hard-coded defaults with no config file loading | Medium -- every change requires rebuild | 1 file |
| 2 | No centralized error-to-API middleware | Low -- not yet needed (stubs) | -- |
| 3 | Production parser not connected to execution engine | High -- dual parser path | 1 subsystem |

### 8.2 Spike Anti-Patterns (from Tier 1 + verified)

| # | Pattern | Impact | Count |
|---|---------|--------|-------|
| 1 | Public fields on AxiathonEvent | High -- 78 call sites use struct literals | 78 sites |
| 2 | Public tenant_id on TenantContext | High -- 93 call sites access directly | 93 sites |
| 3 | Duplicate Severity enums | Medium -- SeverityId (0-6) vs Severity (Info-Critical) | 2 types |
| 4 | Duplicate FieldValue enums | Medium -- event.rs vs ocsf.rs versions | 2 types |
| 5 | Duplicate AxiathonError types | Medium -- production (9 variants) vs spike (12 variants) | 2 types |
| 6 | No ReDoS protection in detection DSL | Medium -- unlike production parser | 1 subsystem |
| 7 | Two different parser frameworks | Medium -- Chumsky vs Pest for different DSLs | 2 parsers |
| 8 | Two different AxiQL syntaxes | High -- Lucene-like (spike) vs SQL-like (production) | 2 parsers |
| 9 | std::sync::RwLock in async context | Medium -- potential runtime blocking | writer.rs |
| 10 | Hardcoded tenant list | High -- 2 tenants in AppState::new() | state.rs |
| 11 | mem::forget on TempDir | Low -- spike only, marked for production fix | state.rs |
| 12 | anyhow::Error as catch-all variant | Medium -- loses structured error information | error.rs |

---

## 9. Convention Consistency Assessment

| Convention | Production | Spike | Overall |
|-----------|-----------|-------|---------|
| forbid(unsafe_code) | 100% (8/8 crates) | 0% (0/19 crates) | Split |
| lib.rs barrel export | 100% | ~70% | Good |
| #[non_exhaustive] on extensible enums | 100% | ~80% | Good |
| Validated constructors for IDs | 100% | 100% (for IDs) | Excellent |
| SECURITY comments | 100% (where applicable) | ~60% | Mixed |
| Test naming convention | ~95% | ~80% | Good |
| Workspace-level deps | 100% | 100% | Excellent |
| Error type pattern | 100% | ~70% (has anyhow catch-all) | Mixed |
| Private fields on security types | 100% | 0% (marked with TODOs) | Split |
| tracing structured logging | N/A (stubs) | Present but minimal | Minimal |

---

## Delta Summary
- New items added: Formatting enforcement chain (rustfmt + taplo + typos with specific exclusions), module layout convention documented from .claude/rules/rust.md, test strategy matrix (5 test types), 4 new design patterns (FromRef decomposition, per-tenant engine map, builder-with-notify, SECURITY comment convention), dependency management conventions (workspace deps, action pinning, Git Flow), expanded anti-pattern list (12 items vs 5 in broad sweep), convention consistency assessment table
- Existing items refined: All conventions verified with consistency percentages, spike exclusion from quality tools documented (taplo, typos both exclude spike/)
- Remaining gaps: Detection rule .axd file conventions (only referenced by benchmarks, not directly examined), docs/.archive/ coding standards, CONTRIBUTING.md full review

## Novelty Assessment
Novelty: SUBSTANTIVE
The formatting enforcement chain reveals that spike/ is systematically EXCLUDED from quality gates (taplo, typos, no CI, no forbid(unsafe_code)). This is a model-changing insight -- it confirms the spike is intentionally treated as throwaway prototype code, not production-quality. The 4 new patterns (FromRef, per-tenant engine map, builder-with-notify, SECURITY comment convention) and the expanded anti-pattern list with call-site counts provide spec-relevant detail. The convention consistency assessment quantifies the production-spike quality gap.

## Convergence Declaration
Another round needed -- detection rule file conventions (.axd files), CONTRIBUTING.md full review, and consistency audit of newly discovered patterns across spike subsystems.

## State Checkpoint
```yaml
pass: 5
round: 1
status: complete
files_scanned: 25+
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
next_pass: 5-r2
```
