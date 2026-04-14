# Pass 5 Deep Dive Round 2: Convention & Pattern Catalog -- ocsf-proto-gen

## Objective

Hallucination audit of Round 1 convention claims. Verify naming patterns, design pattern assertions, and consistency assessments against source code.

---

## Hallucination Audit

### Naming Convention Verification

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "Ocsf prefix on all schema types" | VERIFIED | `OcsfSchema`, `OcsfClass`, `OcsfObject`, `OcsfAttribute`, `OcsfEnumValue`, `OcsfDeprecated` -- all 6 schema types have `Ocsf` prefix |
| "`class_upper`, `attr_upper`, `obj_upper` naming" | VERIFIED | codegen.rs:218 `class_upper`, 286 `attr_upper`, 323 `obj_upper` |
| "`out` for string buffer" | VERIFIED | codegen.rs:203, 266, 305, 368 all use `let mut out = String::new()` |
| "`stats` parameter name" | VERIFIED | codegen.rs functions use `stats: &mut GenerationStats` consistently |
| "version_slug never version_str" | VERIFIED | All references use `version_slug`. No `version_str` appears anywhere. |
| "GenerationStats breaks Ocsf prefix pattern" | VERIFIED | `GenerationStats` is in codegen module, not schema module -- intentional distinction |

### Module Organization Verification

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "Flat src/ layout, no subdirectories" | VERIFIED | All source files directly under `src/` |
| "No mod.rs files" | VERIFIED | No `mod.rs` in file listing |
| "Unit tests in schema.rs and type_map.rs only" | VERIFIED | `#[cfg(test)] mod tests` appears in schema.rs:244 and type_map.rs:120. NOT in codegen.rs, error.rs, main.rs, or lib.rs |

### Error Handling Verification

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "Only one #[from]: Json variant" | VERIFIED | error.rs:32: `Json(#[from] serde_json::Error)`. No other `#[from]` attributes. |
| "8 map_err occurrences" | RECOUNTED | schema.rs: 2 (lines 185-188, 229-232) + download path: 3 (lines 203-204, 216, 223-226) + codegen.rs write_file: 2 (lines 629-632, 634-637) + main.rs: 1 (line 94). Total: 8. Wait -- schema.rs download has 4 `.map_err` calls (lines 203-204, 216, 220, 223-226, 229-232). Let me recount from the actual source. schema.rs has: line 185 (Read), line 220 (Schema -- for download validation, but this is not map_err, it's a direct `Err` construction via `map_err`). Actually line 220 is `serde_json::from_str(&body).map_err(|e| Error::Schema(...))` -- yes, that is map_err. Lines 203-204: `reqwest::get(&url).await.map_err(...)`. Line 213-216: `response.text().await.map_err(...)`. Lines 219-220: `serde_json::from_str(&body).map_err(...)`. Lines 223-226: `create_dir_all(parent).map_err(...)`. Lines 229-232: `std::fs::write(output_path, &body).map_err(...)`. That is 5 in download_schema. Plus 1 in load_schema (line 185). Plus 2 in write_file (codegen.rs:629, 634). Plus 1 in main.rs (line 94). Total: **9 map_err calls, not 8.** The serde_json parse in load_schema uses `?` with auto-conversion (line 189: `serde_json::from_str(&content)?` -- no map_err, uses #[from]). So: 5 (download) + 1 (load read) + 2 (write_file) + 1 (main runtime) = 9. **Correction: 9, not 8.** |
| "writeln!().unwrap() is documented as safe exception" | VERIFIED | CLAUDE.md and CONTRIBUTING.md both mention this exception |

### Design Pattern Verification

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "BFS with Vec queue" | VERIFIED | codegen.rs:144: `let mut queue: Vec<String> = Vec::new()` |
| "BFS uses pop() not pop_front()" | VERIFIED | codegen.rs:161: `while let Some(obj_ref) = queue.pop()` -- this is technically DFS (LIFO) not BFS (FIFO). **Correction: The algorithm uses `Vec::pop()` which is LIFO (stack), making this a DFS traversal, not BFS. The result is the same (complete transitive closure) but the traversal order differs.** |
| "3-tier fallback in lookup_object" | VERIFIED | codegen.rs:183-191: `get(name).or_else(|| get(&sanitized).or_else(|| values().find(...)))` |
| "GenerationStats initialized with Default::default()" | VERIFIED | codegen.rs:47: `let mut stats = GenerationStats::default()` |

### Testing Pattern Verification

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "test_schema() builds programmatically" | VERIFIED | integration.rs:13-345 |
| "default_attr() returns all empty/None/false" | VERIFIED | integration.rs:347-361 |
| "AtomicU64 + process::id() for tempdir" | VERIFIED | integration.rs:575-579 |
| "walkdir returns sorted paths" | VERIFIED | integration.rs:600: `files.sort()` |
| "No test_ prefix on test names" | VERIFIED | All 25 test functions lack `test_` prefix |
| "sanitize_object_name_strips_prefix is inconsistent naming" | MINOR NITPICK | This test name follows the same `{function_name}_{behavior}` pattern as others like `pascal_case_strips_extension_prefix`. The inconsistency claim was overstated -- all type_map tests follow `{function_name}_{behavior}` pattern. Integration tests follow a different `{behavior_description}` pattern. Both are internally consistent within their files. |

### Consistency Assessment Verification

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "100% BTreeMap/BTreeSet" | VERIFIED | No HashMap/HashSet in any source |
| "100% thiserror for errors" | VERIFIED | Only error type derives thiserror::Error |
| "~80% doc comments on private functions" | APPROXIMATE | Most private functions in codegen.rs have `///` comments. `is_integer_enum` and `collect_enum_entries` have shorter but present comments. Approximately correct. |

---

## Corrections to Round 1

1. **map_err count**: 9, not 8. The download_schema function has 5 (not 4) map_err calls.
2. **BFS vs DFS**: The object graph traversal uses `Vec::pop()` (LIFO), making it a DFS traversal, not BFS. The doc comments in codegen.rs call it "BFS" (line 137, 142), and the broad sweep repeats this. However, `Vec::pop()` removes from the end (stack behavior). For a true BFS, it would need `VecDeque::pop_front()`. The result (transitive closure) is identical regardless of traversal order, but the terminology is incorrect. This is a documentation bug in the source code, not a behavioral issue.
3. **Test naming inconsistency**: Overstated. The `sanitize_object_name_strips_prefix` name follows the same `{function}_{behavior}` pattern used consistently in type_map.rs tests. The difference is between file-level conventions (type_map uses `{function}_{behavior}`, integration uses `{behavior_description}`), not within-file inconsistency.

---

## New Observation: Source Code Doc Bug

The source code documentation at codegen.rs:137 says "BFS" and line 160 says "BFS: follow object -> object references". But the implementation uses `Vec::pop()` (DFS). This is a genuine documentation inaccuracy in the source. It does not affect correctness (both BFS and DFS produce the same transitive closure for a directed graph), but the terminology is wrong.

The broad sweep, Pass 2, Pass 3, and all deep dive rounds repeat this "BFS" claim from the source. This is now corrected.

---

## Delta Summary
- New items added: 1 source code documentation bug discovered (BFS vs DFS terminology)
- Existing items refined: map_err count corrected (9 not 8), test naming inconsistency claim retracted, BFS terminology corrected to DFS
- Remaining gaps: None

## Novelty Assessment
Novelty: NITPICK
The BFS/DFS correction is a genuine documentation bug discovery, but it does not affect behavior (transitive closure is traversal-order-independent). The map_err count and test naming corrections are trivial. Removing these findings would not change how you would spec the system -- the traversal still produces the correct object set.

## Convergence Declaration
Pass 5 has converged -- findings are nitpicks, not gaps. The convention catalog is complete and verified. The one notable discovery (DFS not BFS) is a documentation correction, not a behavioral or convention gap.

## State Checkpoint
```yaml
pass: 5
round: 2
status: complete
timestamp: 2026-04-13T23:45:00Z
novelty: NITPICK
```
