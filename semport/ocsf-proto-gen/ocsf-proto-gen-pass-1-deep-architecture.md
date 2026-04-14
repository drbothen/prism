# Pass 1 Deep Dive Round 1: Architecture -- ocsf-proto-gen

## Objective

Deepen the architectural analysis beyond the broad sweep's high-level component catalog. Focus on: precise module visibility boundaries, function-level API surface, cross-cutting concerns, the async/sync boundary, and the feature-gated compilation model.

---

## Corrections to Broad Sweep

1. **Dependency graph incomplete**: The broad sweep shows `CODEGEN --> SCHEMA`, `CODEGEN --> TYPE_MAP`, `CODEGEN --> ERROR`, `SCHEMA --> ERROR`. Missing: `MAIN --> SCHEMA` (direct, for `download_schema`) and `MAIN --> CODEGEN` (direct, for `generate`). The main.rs does NOT go through lib.rs in the sense of a single dispatch -- it calls schema and codegen modules directly via their fully qualified paths (`ocsf_proto_gen::schema::*`, `ocsf_proto_gen::codegen::*`).

2. **Layer diagram oversimplified**: The broad sweep shows "CLI -> Library -> Schema/Codegen" but main.rs calls schema and codegen directly. The lib.rs is a re-export hub, not a dispatch layer. There is no function in lib.rs -- it is purely `pub mod` declarations.

3. **Data flow diagram missing write step**: The broad sweep shows generation functions producing strings but does not show the `write_file()` calls that actually create the output. The `write_file` function is in codegen.rs and handles directory creation + file writing.

---

## Module Visibility Analysis

### Public API Surface (lib.rs re-exports)

Everything in these four modules is publicly accessible to library consumers:

| Module | Public Items | Purpose |
|--------|-------------|---------|
| `codegen` | `generate()`, `GenerationStats` | Generation entry point + result type |
| `error` | `Error`, `Result<T>` | Error types |
| `schema` | `OcsfSchema`, `OcsfClass`, `OcsfObject`, `OcsfAttribute`, `OcsfEnumValue`, `OcsfDeprecated`, `load_schema()`, `download_schema()` | All domain types + I/O |
| `type_map` | `ocsf_to_proto_type()`, `to_pascal_case()`, `to_screaming_snake()`, `sanitize_object_name()`, `to_enum_variant_name()` | All name/type conversion utilities |

### Private Items (not accessible to library consumers)

| Module | Private Items | Count |
|--------|--------------|-------|
| `codegen` | `resolve_object_graph`, `lookup_object`, `generate_events_proto`, `generate_class_enums_proto`, `generate_objects_proto`, `generate_object_enums_proto`, `generate_enum_value_map`, `collect_enum_entries`, `resolve_event_field_type`, `resolve_object_field_type`, `resolve_object_ref`, `is_integer_enum`, `write_enum_definition`, `version_to_slug`, `write_file` | 15 |
| `schema` | `minimal_schema_json` (test-only) | 1 |
| `main` | `Cli`, `Commands`, `main`, `run` | 4 (binary-only, not library API) |

**Observation:** The public API is minimal (2 functions + 8 types). The codegen module has 15 private functions -- this is a deliberate encapsulation choice. Library consumers only see `generate()` and `GenerationStats`.

---

## Async/Sync Boundary Analysis

The codebase has a sharp async/sync boundary:

```
main.rs: sync
  |
  +-- Commands::DownloadSchema
  |     |-- tokio::runtime::Runtime::new()   <-- creates tokio runtime
  |     |-- rt.block_on(download_schema())   <-- sync-to-async bridge
  |     |       |
  |     |       +-- download_schema(): async fn
  |     |             |-- reqwest::get().await
  |     |             |-- response.text().await
  |     |             |-- serde_json::from_str() (sync)
  |     |             |-- std::fs::write() (sync)
  |
  +-- Commands::Generate
        |-- load_schema() (sync)
        |-- generate() (sync)
```

**Key design decision:** The binary creates a tokio runtime manually via `Runtime::new()` + `block_on()` rather than using `#[tokio::main]`. This is because:
1. The `generate` subcommand is entirely synchronous -- no runtime needed
2. The download subcommand is feature-gated -- the runtime is only created when `download` feature is active
3. This avoids making the entire binary async when only one code path needs it

**Error mapping at the boundary:** `Runtime::new()` failure is mapped to `Error::Schema(e.to_string())` -- a somewhat misleading variant name for a tokio runtime creation error. This is a minor code smell.

---

## Feature-Gated Compilation Model

The `download` feature gates code at three levels:

| Level | Location | Gated Items |
|-------|----------|-------------|
| Dependencies | `Cargo.toml` | `reqwest`, `tokio` |
| Error variant | `error.rs:35-37` | `Error::Download` variant |
| CLI subcommand | `main.rs:21-38` | `Commands::DownloadSchema` variant |
| Schema function | `schema.rs:197-242` | `download_schema()` async function |
| Main dispatch | `main.rs:86-100` | `Commands::DownloadSchema` match arm |

**Without `download` feature:**
- Binary has only the `generate` subcommand
- No network dependencies (reqwest, tokio)
- `Error` enum has 6 variants (not 7)
- Library crate is fully synchronous, no async runtime
- Crate size significantly smaller (no TLS, HTTP, async machinery)

**This matters for Prism:** If Prism uses `ocsf-proto-gen` as a library dependency, it should use `default-features = false` to avoid pulling in network deps it doesn't need.

---

## Cross-Cutting Concerns

### 1. Determinism (Pervasive)

Determinism is not localized to a single module -- it is enforced by consistent choices across the entire codebase:

| Location | Mechanism |
|----------|-----------|
| `schema.rs` | All collections are `BTreeMap` (sorted keys) in serde-derived types |
| `codegen.rs` | Object graph uses `BTreeSet<String>` for sorted iteration |
| `codegen.rs` | Category grouping uses `BTreeMap<String, Vec<&OcsfClass>>` |
| `codegen.rs` | Enum entries sorted by integer key before emission |
| `codegen.rs` | Field numbering sequential from 1, alphabetical order via BTreeMap |
| `codegen.rs` | Enum value map uses `BTreeMap<String, serde_json::Value>` |
| `type_map.rs` | Pure functions, no state |

**No HashMap or HashSet appears anywhere in the codebase.** This is a strict convention.

### 2. Error Handling (Layered)

```
codegen.rs / schema.rs
  |-- Returns error::Result<T>
  |-- Uses ? operator with Error variants
  |-- Error::Json auto-converts via #[from]
  |
main.rs::run()
  |-- Propagates Result<()> via ?
  |
main.rs::main()
  |-- Catches Err(e)
  |-- Prints error + cause chain via Error::source() traversal
  |-- Exits with code 1
```

No `panic!()` or `unwrap()` in production code paths. The sole exception is `writeln!(String, ...).unwrap()` which is safe because `fmt::Write` for `String` is infallible.

### 3. Stderr Diagnostics

All user-facing output goes to stderr, never stdout:
- Progress messages in `main.rs` (`eprintln!`)
- Warnings in `codegen.rs` (`eprintln!("warning: ...")`)
- Error chain in `main.rs` (`eprintln!("error: ...")`)
- Download progress in `schema.rs` (`eprintln!("Downloading...")`)

The generated .proto files go to disk. Nothing goes to stdout. This is correct for a code generator -- stdout could be captured by shell pipelines.

### 4. Logging (Absent)

There is no logging framework (no `log`, `tracing`, `env_logger`). All diagnostics are raw `eprintln!`. This is appropriate for the codebase size but would not scale.

---

## Component Interaction Patterns

### Pattern 1: String-Buffer Code Generation

All proto generation functions follow the same pattern:

```rust
fn generate_X_proto(...) -> String {
    let mut out = String::new();
    writeln!(out, "syntax = \"proto3\";").unwrap();
    writeln!(out, "package ...").unwrap();
    // ... generate content into `out` ...
    out
}
```

Then the caller writes the string to disk:
```rust
write_file(&path, &events_proto)?;
```

This is a simple, testable pattern: generation is pure (string in, string out), I/O is isolated in `write_file`.

### Pattern 2: Stats Accumulator Threading

`GenerationStats` is created as `Default::default()` in `generate()`, then passed as `&mut GenerationStats` to every generation and resolution function. It is the only mutable state threaded through the pipeline (besides the output String buffers).

This is a simple alternative to a more complex approach (like returning stats from each function and merging). It works because the pipeline is single-threaded and sequential.

### Pattern 3: Object Lookup Fallback Chain

`lookup_object()` uses a 3-tier fallback:
1. Exact key match: `schema.objects.get(name)`
2. Sanitized key match: `schema.objects.get(&sanitize_object_name(name))`
3. Linear scan: `schema.objects.values().find(|o| sanitize_object_name(&o.name) == sanitized)`

This exists twice: in `resolve_object_graph` (for BFS) and in `resolve_object_ref` (for field type resolution). However, `resolve_object_ref` uses a different lookup: it searches `objects` (the `BTreeMap<String, OcsfObject>` parameter, which is `schema.objects`), not via the `lookup_object` function. This is a subtle duplication -- `resolve_object_ref` re-implements the 3-tier lookup inline at lines 548-555.

---

## Deployment Topology (Expanded)

```
                     +-------------------+
                     |  crates.io        |
                     |  ocsf-proto-gen   |
                     +-------------------+
                            |
              +-------------+-------------+
              |                           |
    +---------v---------+      +----------v----------+
    | Binary: ocsf-proto-gen |  | Library: ocsf_proto_gen |
    | (includes CLI layer)   |  | (no CLI, no main)       |
    +------------------------+  +-------------------------+
              |                           |
    +---------+---------+      +----------+----------+
    | With download      |      | default-features    |
    | feature (default)  |      | = false             |
    +--------------------+      +---------------------+
    | - download-schema  |      | - generate only     |
    | - generate         |      | - no network deps   |
    | - reqwest+tokio    |      | - fully sync        |
    +--------------------+      +---------------------+
```

---

## Architectural Anti-Patterns and Debt

1. **Dual object lookup implementation**: `lookup_object()` (codegen.rs:182-192) and `resolve_object_ref()` (codegen.rs:548-555) both implement the 3-tier name fallback. If the lookup logic changes, both must be updated. This is minor tech debt.

2. **Dual field resolution functions**: `resolve_event_field_type` and `resolve_object_field_type` differ only in the enum package path. Could be unified with a `package_prefix` parameter. Noted as minor code smell in Pass 2 deep dive.

3. **`Error::Schema` variant reused for tokio runtime failure** (main.rs:94): The runtime creation error is mapped to `Error::Schema(e.to_string())` which is semantically incorrect. Should probably be its own variant or a more generic variant.

4. **No structured logging**: `eprintln!` for all diagnostics. Acceptable for current scope but would need to be replaced for library consumption (libraries should not print to stderr).

5. **Objects proto as single file**: All needed objects go into one `objects.proto` file. For the full OCSF schema (170 objects), this could produce a very large single file. This is a known limitation noted in the broad sweep.

---

## Delta Summary
- New items added: Module visibility analysis (public vs private API surface), async/sync boundary analysis, feature-gated compilation model (5 gating levels), cross-cutting concern analysis (4 concerns), component interaction patterns (3 patterns), deployment topology expansion, 5 architectural anti-patterns/debt items
- Existing items refined: Dependency graph corrected (main.rs direct module access), layer diagram corrected (lib.rs is re-export hub not dispatch), data flow missing write step
- Remaining gaps: None -- architecture is fully characterized at function-level granularity

## Novelty Assessment
Novelty: SUBSTANTIVE
The async/sync boundary analysis, feature-gated compilation model, and the discovery of dual object lookup implementation are all new findings that change how you would architect a replacement. The public/private API surface distinction is essential for determining what Prism can depend on. The correction of the dependency graph (main.rs bypasses lib.rs) changes the architectural model.

## Convergence Declaration
Another round needed -- hallucination audit required and cross-reference with Pass 2/3 deep dive findings for consistency.

## State Checkpoint
```yaml
pass: 1
round: 1
status: complete
timestamp: 2026-04-13T23:15:00Z
novelty: SUBSTANTIVE
```
