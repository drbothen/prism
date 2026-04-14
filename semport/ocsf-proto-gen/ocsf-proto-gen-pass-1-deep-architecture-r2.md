# Pass 1 Deep Dive Round 2: Architecture -- ocsf-proto-gen

## Objective

Hallucination audit of Round 1 architecture claims. Verify all structural assertions against source code. Cross-reference with Pass 2/3 deep dive findings.

---

## Hallucination Audit

### Round 1 Claim Verification

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "main.rs calls schema and codegen directly via fully qualified paths" | VERIFIED | main.rs:95 `ocsf_proto_gen::schema::download_schema(...)`, main.rs:113 `ocsf_proto_gen::schema::load_schema(...)`, main.rs:133 `ocsf_proto_gen::codegen::generate(...)` |
| "lib.rs is purely pub mod declarations" | VERIFIED | lib.rs lines 32-35: 4 `pub mod` statements, nothing else (plus doc comment) |
| "No function in lib.rs" | VERIFIED | lib.rs has only module declarations and a doc comment with usage example |
| "15 private functions in codegen.rs" | VERIFIED | Counted: `resolve_object_graph`, `lookup_object`, `generate_events_proto`, `generate_class_enums_proto`, `generate_objects_proto`, `generate_object_enums_proto`, `generate_enum_value_map`, `collect_enum_entries`, `resolve_event_field_type`, `resolve_object_field_type`, `resolve_object_ref`, `is_integer_enum`, `write_enum_definition`, `version_to_slug`, `write_file` = 15 |
| "tokio runtime created manually via Runtime::new() + block_on()" | VERIFIED | main.rs:93-96: `tokio::runtime::Runtime::new()...rt.block_on(...)` |
| "Error::Schema for runtime failure is misleading" | VERIFIED | main.rs:94: `.map_err(|e| ocsf_proto_gen::error::Error::Schema(e.to_string()))` maps IO error from `Runtime::new()` to `Schema` variant |
| "Feature gates at 5 levels" | PARTIALLY VERIFIED | Cargo.toml (dep level), error.rs:35 (variant), main.rs:21 (subcommand variant), schema.rs:197 (function), main.rs:86 (match arm). That is 5 locations but arguably 4 levels (dep, type, function, dispatch). Keeping 5. |
| "resolve_object_ref re-implements 3-tier lookup" | VERIFIED | codegen.rs:548-555 does `objects.get(obj_type).or_else(|| objects.get(&sanitized)).or_else(|| objects.values().find(...))` which is the same pattern as `lookup_object` at codegen.rs:183-191 |
| "All diagnostics go to stderr" | VERIFIED | Searched all source: every `eprintln!` goes to stderr; no `println!` in non-test code |
| "No HashMap or HashSet anywhere" | VERIFIED | Only `BTreeMap` and `BTreeSet` imports in all source files. `std::collections::BTreeMap` in codegen.rs:13, schema.rs:8. `BTreeSet` in codegen.rs:13. No `HashMap` or `HashSet` imports anywhere. |
| "writeln!().unwrap() used ~30 times in codegen.rs" | APPROXIMATE | Counted from read: approximately 25-30 `writeln!(...).unwrap()` calls in codegen.rs. Claim is accurate within margin. |
| "No structured logging (no log, tracing, env_logger)" | VERIFIED | No `log` or `tracing` in Cargo.toml dependencies. No imports of logging crates in any source file. |
| "download_schema subcommand has no --quiet flag" | VERIFIED | main.rs:22-38: `DownloadSchema` struct has `ocsf_version`, `output_dir`, `schema_url` but no `quiet` field |
| "codegen warnings not gated by quiet" | VERIFIED | codegen.rs:320 and 558 use `eprintln!("warning: ...")` directly, not checking any quiet flag. The `quiet` variable exists only in main.rs scope. |

### Corrections

1. **"main.rs bypasses lib.rs"**: This phrasing is slightly misleading. main.rs uses `ocsf_proto_gen::schema::*` which goes through the library crate's public API (which is defined by lib.rs's `pub mod` declarations). It does not "bypass" lib.rs -- it uses the crate as a library. The point is that lib.rs has no dispatch logic, just re-exports. Clarifying: the dependency is `main.rs -> lib crate -> {schema, codegen, error} modules`. The lib.rs is a gateway, not a logic layer.

2. **"resolve_object_ref uses different lookup"**: Round 1 says it "uses a different lookup" from `lookup_object`. More precisely: `resolve_object_ref` operates on the `objects` parameter (which is `&BTreeMap<String, OcsfObject>`, passed as `schema.objects` from the caller), while `lookup_object` operates on `&OcsfSchema`. The logic is the same 3-tier pattern, but the code path is different because `resolve_object_ref` does not have access to the full schema, only the objects map.

---

## Cross-Reference with Pass 2/3 Deep Dives

### Domain Model Alignment (Pass 2)

Pass 2 R1 documented the `resolve_object_ref` lookup duplication and the dual field resolution functions. The architecture analysis aligns:
- Architecture correctly identifies both as anti-patterns
- Pass 2 R2 confirmed these are the only structural duplications in the codebase

### Behavioral Contract Alignment (Pass 3)

Pass 3 identified 67 behavioral contracts. Architecture-relevant observations:
- BC-7.06.001 (objects proto imports its own enums) is MEDIUM confidence (from code, not tested). This is an architectural detail that confirms the import structure shown in the data flow diagram.
- BC-8.01.003 (quiet flag) confirms the observability gap identified in architecture R1: codegen warnings bypass the quiet flag.

### No New Architectural Insights

After reading every source file and cross-referencing with Pass 2/3, no additional architectural patterns, layers, or component relationships were discovered. The architecture is simple enough that the broad sweep + R1 captured it completely.

---

## Delta Summary
- New items added: 0
- Existing items refined: 2 clarifications (lib.rs gateway vs bypass, resolve_object_ref parameter difference)
- Remaining gaps: None

## Novelty Assessment
Novelty: NITPICK
Round 2 is entirely verification and minor clarification. No new architectural patterns, components, or relationships were discovered. The two refinements (lib.rs clarification, parameter difference in dual lookup) are precision improvements, not model changes. Removing these findings would not change how you would spec the system.

## Convergence Declaration
Pass 1 has converged -- findings are nitpicks, not gaps. The architecture is fully characterized.

## State Checkpoint
```yaml
pass: 1
round: 2
status: complete
timestamp: 2026-04-13T23:35:00Z
novelty: NITPICK
```
