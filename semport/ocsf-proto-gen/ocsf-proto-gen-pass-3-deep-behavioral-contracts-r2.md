# Pass 3 Deep Dive Round 2: Behavioral Contracts -- ocsf-proto-gen

## Objective

Examine inter-subsystem contracts, edge cases in helper functions, and any implicit behavioral guarantees not captured in Round 1.

---

## Inter-Subsystem Consistency Contracts

### BC-5.05.001: Enum references in event protos match enum definitions in enum protos

**Preconditions:** Event proto references `ocsf.{v}.events.{cat}.enums.{CLASS}_{ATTR}`
**Postconditions:** The corresponding enum proto file contains `enum {CLASS}_{ATTR} { ... }` with the same name
**Invariant:** The name construction in `resolve_event_field_type` (codegen.rs:484-485) uses the same `{class_upper}_{attr_upper}` pattern as `generate_class_enums_proto` (codegen.rs:287)
**Evidence:** Both functions use identical `to_screaming_snake` calls on the same inputs; integration test `generated_proto_has_correct_content` verifies the event reference and `generated_enums_have_correct_values` verifies the definition exist
**Confidence:** HIGH (implicit from shared code path + dual test coverage)

### BC-5.05.002: Object enum references match object enum definitions

**Preconditions:** Object proto references `ocsf.{v}.objects.enums.{OBJ}_{ATTR}`
**Postconditions:** The objects enum proto file contains `enum {OBJ}_{ATTR} { ... }`
**Invariant:** Same as above but for `resolve_object_field_type` (codegen.rs:519) and `generate_object_enums_proto` (codegen.rs:393)
**Evidence:** `generated_objects_have_correct_fields` checks the reference; `generated_enums_have_correct_values` checks the definition
**Confidence:** HIGH

### BC-7.06.001: Objects proto imports its own enums proto

**Preconditions:** Objects proto file generated
**Postconditions:** Contains `import "ocsf/{v}/objects/enums/enums.proto";`
**Evidence:** Code inspection (codegen.rs:311-315)
**Confidence:** MEDIUM (from code, not explicitly tested -- tests check event imports but not object imports)

### BC-7.06.002: Event proto imports both its category enums and shared objects

**Preconditions:** Event proto file generated
**Postconditions:** Contains two import statements: category enums and objects
**Evidence:** Already captured as BC-7.01.002, confirmed HIGH
**Confidence:** HIGH

---

## Version Slug Edge Cases

### BC-4.05.001: version_to_slug handles standard semver

**Preconditions:** Version string like `"1.7.0"`
**Postconditions:** Returns `"v1_7_0"`
**Evidence:** Code inspection (codegen.rs:622-624) -- `format!("v{}", version.replace(['.', '-'], "_"))`; all tests use `"1.7.0"` and verify `"v1_7_0"` in output paths
**Confidence:** HIGH (implicitly tested through all integration tests)

### BC-4.05.002: version_to_slug handles pre-release versions with hyphens

**Preconditions:** Version string like `"1.8.0-dev"`
**Postconditions:** Returns `"v1_8_0_dev"` (both `.` and `-` replaced with `_`)
**Evidence:** Doc comment at codegen.rs:622 states this behavior
**Confidence:** LOW (documented but not tested)

---

## write_file Guarantees

### BC-7.07.001: write_file creates parent directories recursively

**Preconditions:** Parent directory does not exist
**Postconditions:** All parent directories created via `create_dir_all`; file written
**Error Cases:** `Error::Write` if directory creation or file write fails
**Evidence:** Code inspection (codegen.rs:627-639); exercised by all integration tests since temp dirs start empty
**Confidence:** HIGH (exercised in every test)

### BC-7.07.002: write_file overwrites existing files

**Preconditions:** File already exists at path
**Postconditions:** File content replaced (uses `std::fs::write` which truncates and writes)
**Evidence:** Code inspection (codegen.rs:634) -- `std::fs::write(path, content)`; determinism test runs twice to same dir implicitly
**Confidence:** MEDIUM (from code + determinism test writes different dirs, so overwrite not directly tested)

---

## Field Numbering Guarantees

### BC-7.08.001: Field numbers are sequential starting at 1

**Preconditions:** Message with N non-deprecated attributes
**Postconditions:** Fields numbered 1 through N with no gaps
**Invariant:** `field_num` initialized to `1u32` and incremented by 1 for each non-deprecated field
**Evidence:** Code inspection (codegen.rs:225, 249 for events; 328, 352 for objects)
**Confidence:** MEDIUM (from code; no test explicitly verifies field numbers)

### BC-7.08.002: Field numbering excludes deprecated fields

**Preconditions:** Message has deprecated attributes interspersed with non-deprecated
**Postconditions:** Deprecated fields do not consume a field number; numbering is contiguous for non-deprecated fields only
**Evidence:** Code inspection -- `continue` at codegen.rs:229 skips before `field_num` increment
**Confidence:** MEDIUM (from code)

### BC-7.08.003: Field order is alphabetical by attribute name

**Preconditions:** Attributes stored in `BTreeMap<String, OcsfAttribute>`
**Postconditions:** Iteration order is alphabetical (BTreeMap guarantee); first alphabetical attribute gets field number 1
**Evidence:** BTreeMap iterator guarantee + code structure
**Confidence:** HIGH (structural guarantee from Rust std)

---

## Enum Definition Ordering Guarantees

### BC-5.06.001: Enum variants are sorted by integer key value

**Preconditions:** Integer-keyed enum with arbitrary key ordering
**Postconditions:** Variants emitted in ascending integer order
**Evidence:** Code inspection (codegen.rs:601) -- `entries.sort_by_key(|(k, _)| *k)`
**Confidence:** MEDIUM (from code; test data happens to be pre-sorted)

### BC-5.06.002: Enums within a class are emitted in attribute name order

**Preconditions:** Class has multiple enum attributes
**Postconditions:** Enums appear in the output file in alphabetical attribute name order (BTreeMap iteration)
**Evidence:** Code inspection (codegen.rs:275) -- iterates `cls.attributes` which is BTreeMap
**Confidence:** MEDIUM (from code)

---

## Error Variant Construction Patterns

### BC-9.02.001: Error::ClassNotFound includes available classes (up to 10)

**Preconditions:** Requested class not in schema
**Postconditions:** `available` field contains comma-separated list of up to 10 class names; if > 10, shows "... and N more"
**Evidence:** Code inspection (codegen.rs:52-62)
**Confidence:** MEDIUM

### BC-9.02.002: Error::Write includes the failed path

**Preconditions:** File/directory operation fails
**Postconditions:** `path` field is set to the path that failed (may be the parent dir for `create_dir_all` failures)
**Evidence:** Code inspection (codegen.rs:629-631, 634-637)
**Confidence:** MEDIUM

### BC-9.02.003: Error::Download is only available with "download" feature

**Preconditions:** Crate compiled without `download` feature
**Postconditions:** `Error::Download` variant does not exist; pattern matches are exhaustive without it
**Evidence:** Code inspection (error.rs:36) -- `#[cfg(feature = "download")]`
**Confidence:** HIGH (structural guarantee from cfg)

---

## Test Helper Contracts

### BC-T.01.001: tempdir() produces unique directories across threads and processes

**Preconditions:** Multiple concurrent calls to `tempdir()`
**Postconditions:** Each call returns a unique path using `AtomicU64` counter + process ID
**Evidence:** Code inspection (integration.rs:574-583)
**Confidence:** HIGH (structural from AtomicU64 + process::id())

### BC-T.01.002: tempdir() cleans up prior run artifacts

**Preconditions:** Directory from prior run may exist
**Postconditions:** `remove_dir_all` called before `create_dir_all`; errors from removal are silently ignored (`let _ = ...`)
**Evidence:** Code inspection (integration.rs:580-582)
**Confidence:** HIGH

### BC-T.02.001: test_schema() provides a complete test fixture

**Preconditions:** Called with no arguments
**Postconditions:** Returns schema with:
- 1 class (authentication) with 9 attributes covering: integer enum, string enum, object ref, repeated object ref, empty object ref, deprecated field, scalar, timestamp, severity enum
- 3 objects: network_endpoint (4 attrs with enum), enrichment (2 attrs), object (0 attrs)
- Version "1.7.0"
**Evidence:** Code inspection (integration.rs:13-345)
**Confidence:** HIGH

---

## Updated Coverage Matrix

| Subsystem | Round 1 BCs | Round 2 BCs | Total | HIGH | MEDIUM | LOW |
|-----------|-------------|-------------|-------|------|--------|-----|
| 1. Schema | 7 | 0 | 7 | 4 | 2 | 1 |
| 2. Codegen | 5 | 0 | 5 | 3 | 2 | 0 |
| 3. Type Map | 7 | 0 | 7 | 7 | 0 | 0 |
| 4. Name Conv | 5 | 2 | 7 | 6 | 0 | 1 |
| 5. Enum | 7 | 4 | 11 | 7 | 4 | 0 |
| 6. Object Res | 5 | 0 | 5 | 2 | 2 | 1 |
| 7. Output | 8 | 5 | 13 | 8 | 5 | 0 |
| 8. CLI | 4 | 0 | 4 | 0 | 4 | 0 |
| 9. Error | 2 | 3 | 5 | 2 | 3 | 0 |
| Test helpers | 0 | 3 | 3 | 3 | 0 | 0 |
| **Total** | **50** | **17** | **67** | **42** | **22** | **3** |

---

## Updated Gap List

Gaps from Round 1 remain -- no new test coverage was found to close them. One new gap identified:

11. **Object proto enum import** -- `objects.proto` imports `objects/enums/enums.proto` but this import is not explicitly verified in tests (BC-7.06.001 is MEDIUM from code)

All other Round 1 gaps (items 1-10) remain open with the same assessment.

---

## Delta Summary
- New items added: 17 behavioral contracts (inter-subsystem consistency, version slug edge cases, write guarantees, field numbering, enum ordering, error variant patterns, test helpers)
- Existing items refined: 0 existing contracts changed
- Remaining gaps: 11 gaps total (10 from Round 1 + 1 new); all are edge cases or untested paths, not missing core behaviors

## Novelty Assessment
Novelty: NITPICK
Round 2 contracts are refinements: inter-subsystem consistency checks that are structurally guaranteed by shared code paths, field numbering details from code inspection, and test helper documentation. None of these change how you would spec the system -- they confirm that the system is internally consistent. The 17 new contracts are "nice to know" but do not reveal new behavioral dimensions.

## Convergence Declaration
Pass 3 has converged -- findings are nitpicks, not gaps. 67 behavioral contracts captured across 10 subsystems with 63% HIGH confidence, 33% MEDIUM, and 4% LOW. The LOW-confidence contracts are all code-only observations for edge cases unlikely to occur in practice (tier-3 object lookup, pre-release version slugs, download validation).

## State Checkpoint
```yaml
pass: 3
round: 2
status: complete
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
