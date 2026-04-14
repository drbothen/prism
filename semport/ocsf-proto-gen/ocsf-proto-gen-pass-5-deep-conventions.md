# Pass 5 Deep Dive Round 1: Convention & Pattern Catalog -- ocsf-proto-gen

## Objective

Extract every coding convention, design pattern, naming pattern, and consistency observation from the codebase. The broad sweep touched on conventions briefly (5 name functions, error handling, determinism, testing). This round is exhaustive.

---

## 1. Naming Conventions

### 1.1 Rust Naming (Codebase Conventions)

| Entity Type | Convention | Example | Consistent? |
|-------------|-----------|---------|-------------|
| Struct names | PascalCase | `OcsfSchema`, `GenerationStats` | Yes (100%) |
| Enum names | PascalCase | `Error`, `Commands` | Yes (100%) |
| Enum variants | PascalCase | `ClassNotFound`, `DownloadSchema` | Yes (100%) |
| Function names | snake_case | `resolve_object_graph`, `to_pascal_case` | Yes (100%) |
| Variable names | snake_case | `version_slug`, `field_num`, `class_upper` | Yes (100%) |
| Constants | SCREAMING_SNAKE | `COUNTER` (AtomicU64 in tests) | Yes (only 1 constant) |
| Module names | snake_case | `type_map`, `codegen` | Yes (100%) |
| Test function names | snake_case, descriptive | `empty_object_type_emits_string` | Yes (100%) |

### 1.2 OCSF-Specific Naming Conventions

| Pattern | Convention | Examples |
|---------|-----------|---------|
| Schema struct prefix | `Ocsf` prefix on all schema types | `OcsfSchema`, `OcsfClass`, `OcsfObject`, `OcsfAttribute`, `OcsfEnumValue`, `OcsfDeprecated` |
| Screaming snake suffix | `_upper` suffix for SCREAMING_SNAKE vars | `class_upper`, `attr_upper`, `obj_upper`, `enum_name` |
| Version slug | `version_slug` (never `version_str` or `ver`) | Consistent across all 15+ uses |
| Object sanitized name | `sanitized` | Used in `lookup_object`, `resolve_object_ref`, `resolve_object_graph` |
| Proto output buffer | `out` | All `generate_*_proto` functions use `out: String` |
| Stats parameter | `stats: &mut GenerationStats` | Consistent across all generation functions |

### 1.3 Proto Output Naming Conventions

| Entity | Convention | Example |
|--------|-----------|---------|
| Package | dot-separated, lowercase | `ocsf.v1_7_0.events.iam` |
| Message name | PascalCase (from `to_pascal_case`) | `Authentication`, `NetworkEndpoint` |
| Enum type name | SCREAMING_SNAKE | `AUTHENTICATION_ACTIVITY_ID` |
| Enum variant name | `{ENUM_NAME}_{CAPTION_SCREAMING}` | `AUTHENTICATION_ACTIVITY_ID_LOGON` |
| Field name | snake_case (passed through from OCSF) | `activity_id`, `src_endpoint` |
| File names | snake_case category or `objects`/`enums` | `iam.proto`, `objects.proto`, `enums.proto` |

---

## 2. Module Organization Patterns

### 2.1 Flat Module Structure

The codebase uses a flat `src/` layout with no subdirectories:
```
src/
  main.rs      -- binary
  lib.rs       -- library re-exports
  codegen.rs   -- generation logic
  schema.rs    -- types + I/O
  type_map.rs  -- pure functions
  error.rs     -- error types
```

No barrel exports, no feature folders, no `mod.rs` files. This is appropriate for 6 files / ~1,500 lines.

### 2.2 Re-Export Hub Pattern

`lib.rs` is exclusively `pub mod` declarations -- no functions, no types, no logic:
```rust
pub mod codegen;
pub mod error;
pub mod schema;
pub mod type_map;
```

This gives library consumers access to all public items via `ocsf_proto_gen::{module}::{item}`.

### 2.3 Inline Test Module Pattern

Unit tests are inside the same file as the code they test, using the standard Rust pattern:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // ...
}
```

Used in: `schema.rs` (3 tests), `type_map.rs` (12 tests). NOT used in: `codegen.rs` (no unit tests), `error.rs` (no tests), `main.rs` (no tests).

### 2.4 Separate Integration Test Pattern

Integration tests are in `tests/integration.rs` (Rust convention). They import the library crate:
```rust
use ocsf_proto_gen::codegen;
use ocsf_proto_gen::schema::{...};
```

---

## 3. Error Handling Patterns

### 3.1 thiserror Derive Pattern

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("schema error: {0}")]
    Schema(String),
    // ...
}
```

Every variant has a `#[error("...")]` format string. The `Error` type implements `Display` and `std::error::Error` via derive.

### 3.2 Error Propagation via `?`

All fallible operations use `?` with explicit `.map_err()` for context:
```rust
std::fs::read_to_string(path).map_err(|e| Error::Read {
    path: path.to_path_buf(),
    source: e,
})?;
```

This pattern appears 8 times in the codebase (2 in schema.rs, 4 in codegen.rs write_file and download_schema, 2 in main.rs).

### 3.3 Auto-Conversion via `#[from]`

Only one: `Json(#[from] serde_json::Error)`. All other error types require explicit construction.

### 3.4 Error Chain Printing

```rust
eprintln!("error: {e}");
let mut source = std::error::Error::source(&e);
while let Some(cause) = source {
    eprintln!("  caused by: {cause}");
    source = std::error::Error::source(cause);
}
process::exit(1);
```

This is the only place errors are displayed. Library consumers handle errors themselves.

### 3.5 The writeln!().unwrap() Exception

`writeln!(out, ...).unwrap()` is used ~30 times in codegen.rs. The team explicitly documents that this is safe because `fmt::Write` for `String` never fails. This exception is noted in CLAUDE.md and CONTRIBUTING.md.

---

## 4. Design Patterns

### 4.1 Pipeline Pattern

The generation pipeline is a linear sequence of transformations:
```
validate -> resolve graph -> group by category -> generate per-category -> generate objects -> generate enums -> generate enum map -> write files
```

No branching, no parallelism, no event-driven dispatch. Pure sequential pipeline.

### 4.2 Builder Pattern (Proto String Construction)

All proto file content is built via the same pattern:
1. Create `let mut out = String::new()`
2. Append header (syntax, package, imports)
3. Loop over entities, appending message/enum definitions
4. Return `out`

This is a lightweight string-builder pattern, not a formal builder with method chaining.

### 4.3 BFS Graph Traversal Pattern

Object graph resolution uses textbook BFS:
1. Seed queue from direct references
2. Pop from queue, look up, scan attributes for new references
3. Use visited set (BTreeSet) to prevent cycles
4. Continue until queue is empty

### 4.4 Fallback Chain Pattern (Object Lookup)

3-tier name resolution:
1. Exact match
2. Sanitized match (prefix stripped)
3. Linear scan with sanitized comparison

This is a classic "try in order of increasing cost" pattern.

### 4.5 Accumulator Pattern (Stats)

`GenerationStats` is initialized with `Default::default()` and passed as `&mut` through the entire pipeline. Functions increment counters directly. This is simpler than returning stats from each function and merging.

### 4.6 Feature Gate Pattern

`#[cfg(feature = "download")]` applied at 4 locations to conditionally compile network-dependent code. The same feature name is used consistently. No feature flags interact with each other (there is only one feature).

---

## 5. Testing Patterns

### 5.1 Programmatic Test Fixture

`test_schema()` constructs an `OcsfSchema` in code rather than loading from a JSON file:
```rust
fn test_schema() -> OcsfSchema {
    let mut classes = BTreeMap::new();
    // ... manually construct all fields
}
```

**Advantages:** No file system dependency, no network, no JSON parsing in tests, full control over test data.

**`default_attr()` helper:** Returns an `OcsfAttribute` with all fields empty/None/false. Used with struct update syntax:
```rust
OcsfAttribute {
    type_name: "integer_t".to_string(),
    caption: "Activity ID".to_string(),
    ..default_attr()
}
```

### 5.2 String Assertion Pattern

Tests verify generated proto content via `String::contains()`:
```rust
assert!(proto.contains("message Authentication {"));
assert!(proto.contains("ocsf.v1_7_0.events.iam.enums.AUTHENTICATION_ACTIVITY_ID activity_id"));
```

**Advantages:** Robust against whitespace and ordering changes. Easy to read.
**Disadvantages:** Cannot verify exact field order, field numbers, or complete file structure.

### 5.3 Negative Assertion Pattern

Tests verify absence of incorrect content:
```rust
assert!(!proto.contains("old_field"));        // deprecated field absent
assert!(!proto.contains("Object unmapped"));  // empty object not referenced
assert!(!proto.contains("google.protobuf.Struct")); // no Struct type
assert!(!proto.contains("AUTH_PROTOCOL"));    // string enum not generated
```

### 5.4 Custom tempdir Pattern

Instead of a `tempfile` crate dependency, the tests use a custom `tempdir()`:
```rust
static COUNTER: AtomicU64 = AtomicU64::new(0);
let id = COUNTER.fetch_add(1, Ordering::Relaxed);
let dir = std::env::temp_dir().join(format!("ocsf-proto-gen-test-{}-{}", process::id(), id));
let _ = std::fs::remove_dir_all(&dir);  // clean up prior
std::fs::create_dir_all(&dir).unwrap();
```

Process ID + atomic counter ensures uniqueness across threads and processes. Prior runs cleaned up on creation.

### 5.5 Custom walkdir Pattern

Instead of the `walkdir` crate, a recursive `walkdir()` function:
```rust
fn walkdir(dir: &Path) -> Vec<PathBuf> { ... }
```

Returns sorted file paths for deterministic comparison.

### 5.6 Test Naming Convention

All test functions are descriptive, verb-first or noun-first:
- `end_to_end_generate_and_validate`
- `generated_proto_has_correct_content`
- `generated_enums_have_correct_values`
- `generated_objects_have_correct_fields`
- `enum_value_map_is_valid_json`
- `deterministic_output`
- `invalid_class_name_returns_error`
- `schema_load_from_file`
- `empty_object_type_emits_string`
- `parse_minimal_schema`
- `parse_class_attributes`
- `parse_deprecated_attributes`
- `primitive_type_mapping`
- `all_string_derived_types`
- `timestamp_is_int64_not_string`
- `datetime_is_string`
- `json_t_maps_to_string`
- `object_t_returns_none`
- `unknown_type_falls_back_to_string`
- `pascal_case_conversion`
- `pascal_case_strips_extension_prefix`
- `screaming_snake_conversion`
- `enum_variant_name_conversion`
- `sanitize_object_name_strips_prefix` (note: inconsistent -- this one is `noun_verb`, most are `verb_noun`)

No `test_` prefix (Rust convention -- the `#[test]` attribute is sufficient).

---

## 6. Documentation Patterns

### 6.1 Module-Level Doc Comments

Every library module has a `//!` doc comment block:
- `lib.rs`: 30 lines, includes usage example with `no_run`
- `codegen.rs`: 11 lines, describes what the module generates
- `schema.rs`: 6 lines, describes data source
- `type_map.rs`: 20 lines, includes ASCII type mapping table
- `error.rs`: 1 line

### 6.2 Function-Level Doc Comments

All public functions and most private functions have `///` doc comments:
- Public: 100% coverage
- Private: ~80% coverage (some small helpers like `is_integer_enum` have shorter comments)

### 6.3 Inline Comments at Decision Points

Key decisions annotated with `//` comments:
- `"// Proto3 requires the first enum value to be 0."` (codegen.rs:606)
- `"// json_t maps to string, NOT google.protobuf.Struct."` (type_map.rs:36-38)
- `"// Empty objects ... produce empty proto messages that cannot hold data."` (codegen.rs:563-565)
- `"// Fallback: unknown types emit as string."` (type_map.rs:57)

### 6.4 CLAUDE.md Pattern

A dedicated `CLAUDE.md` file provides architecture reference for AI code assistants. Contains:
- Project overview (6 lines)
- Build and test commands
- Architecture diagram (ASCII directory tree)
- Data flow description
- Key design decisions (8 items)
- Code standards
- Git conventions
- Testing overview
- OCSF schema reference links
- Release process

---

## 7. Git/Release Conventions

| Convention | Standard | Evidence |
|-----------|----------|---------|
| Commit format | Conventional commits | CLAUDE.md: `feat:`, `fix:`, `docs:`, `test:`, `chore:` |
| Tag format | Semver with `v` prefix | `v0.1.0` format (CLAUDE.md + release.yml trigger) |
| Branch model | Single `main` branch | CI triggers on `push: [main]` and `pull_request: [main]` |
| CHANGELOG format | `## version -- date` | `## 0.1.1 -- 2026-02-25` |
| License | MIT | `LICENSE` file + `Cargo.toml license = "MIT"` |

---

## 8. Consistency Assessment

### Fully Consistent Patterns (100%)

- BTreeMap/BTreeSet everywhere (zero HashMap/HashSet)
- `?` operator for error propagation
- `thiserror` for all error types
- snake_case for functions and variables
- PascalCase for types
- `#[serde(default)]` for optional JSON fields
- `pub` only on library API items, not internal helpers
- stderr for all user-facing output

### Mostly Consistent Patterns (>90%)

- Doc comments on functions (~80% of private functions)
- `Ocsf` prefix on schema types (100% of schema types, but `GenerationStats` breaks the pattern -- it is in codegen domain, not schema domain, so this is intentional)
- Test naming (one inconsistency: `sanitize_object_name_strips_prefix` vs the majority pattern)

### Partially Applied Patterns

- **Feature gating**: Only one feature (`download`). Consistently applied where needed, but no granular feature flags for individual capabilities.
- **Quiet mode**: Applied in main.rs for generate command, but NOT applied to download command diagnostics or codegen warnings.

### Anti-Patterns

- **Dual implementation of object lookup**: `lookup_object()` and inline in `resolve_object_ref()` -- same logic, two locations
- **Dual field resolution functions**: `resolve_event_field_type` and `resolve_object_field_type` differ only in enum package path
- **`Error::Schema` for tokio runtime failure**: Semantic mismatch between variant name and error cause

---

## Delta Summary
- New items added: 8 convention categories with 40+ specific observations, 24 test names cataloged, 6 documentation patterns, 4 design patterns beyond broad sweep, consistency assessment with 3 tiers
- Existing items refined: Name conversion functions now documented as both Rust-side and proto-output-side conventions; error handling expanded from 2 patterns to 5
- Remaining gaps: None at the convention extraction level

## Novelty Assessment
Novelty: SUBSTANTIVE
The broad sweep's convention section was 5 brief subsections. This round adds 8 detailed categories covering naming conventions (3 levels), module organization (4 patterns), error handling (5 patterns), design patterns (6 patterns), testing patterns (6 patterns), documentation patterns (4 patterns), git/release conventions, and a structured consistency assessment. The discovery of the quiet-mode gap, the object lookup duplication, and the comprehensive test naming catalog all change how you would spec conventions for a replacement.

## Convergence Declaration
Another round needed -- hallucination audit and cross-reference with other passes.

## State Checkpoint
```yaml
pass: 5
round: 1
status: complete
timestamp: 2026-04-13T23:25:00Z
novelty: SUBSTANTIVE
```
