# Pass 2 Deep Dive Round 2: Domain Model -- ocsf-proto-gen

## Objective

Verify completeness of the domain model from Round 1 by tracing cross-module data flows, lifetime relationships, and examining any remaining structural details.

---

## Cross-Module Data Flow Analysis

### Flow 1: Schema Load Path

```
load_schema(path: &Path) -> Result<OcsfSchema>
  1. std::fs::read_to_string(path) -> String | Error::Read
  2. serde_json::from_str(&content) -> OcsfSchema | Error::Json (via #[from])
  3. Return Ok(OcsfSchema)
```

**Ownership:** The `OcsfSchema` is fully owned. No borrowed references survive from the JSON string. The `content: String` is dropped after deserialization.

### Flow 2: Download Path (feature-gated)

```
download_schema(version: &str, output_path: &Path, base_url: &str) -> Result<()>
  1. Construct URL: format!("{base_url}?version={version}")
  2. reqwest::get(&url).await -> Response | Error::Download
  3. response.text().await -> String (body) | Error::Download
  4. serde_json::from_str::<OcsfSchema>(&body) -> validate | Error::Schema
  5. create_dir_all(parent) | Error::Write
  6. std::fs::write(output_path, &body) -- writes raw JSON, NOT re-serialized struct | Error::Write
  7. Print stats from parsed schema
```

**Critical observation:** Step 4 parses the schema only for validation. Step 6 writes the original `body` string, not the re-serialized struct. This means the saved file preserves the exact API response including fields not captured by the `OcsfSchema` struct.

### Flow 3: Generation Pipeline

```
generate(schema: &OcsfSchema, class_names: &[String], output_dir: &Path)
  |
  +-- validate_class_names(schema.classes, class_names) -> early return Error::ClassNotFound
  |
  +-- resolve_object_graph(schema, class_names) -> BTreeSet<String> (needed_objects)
  |       |
  |       +-- seed: scan class_names -> class.attributes -> attr.object_type
  |       +-- BFS: pop queue -> lookup_object() -> scan obj.attributes -> attr.object_type
  |       |           |
  |       |           +-- lookup_object: 3-tier search
  |       |                 1. schema.objects.get(name) -- exact match
  |       |                 2. schema.objects.get(sanitized) -- prefix-stripped
  |       |                 3. schema.objects.values().find(|o| sanitize(o.name) == sanitized) -- scan
  |       +-- return BTreeSet<String> of sanitized names
  |
  +-- group_by_category: BTreeMap<String, Vec<&OcsfClass>>
  |
  +-- for each category:
  |       +-- generate_events_proto() -> String
  |       |       +-- for each class:
  |       |               +-- for each attr (BTreeMap order = alphabetical):
  |       |                       +-- skip if deprecated
  |       |                       +-- resolve_event_field_type() -> (bool, String)
  |       |                               +-- object_t? -> resolve_object_ref()
  |       |                               +-- integer enum? -> qualified enum ref
  |       |                               +-- string enum? -> increment skip counter, fall through
  |       |                               +-- primitive -> ocsf_to_proto_type()
  |       +-- generate_class_enums_proto() -> String
  |       |       +-- for each class, for each attr:
  |       |               +-- skip deprecated, skip non-enum, skip string-keyed enum
  |       |               +-- write_enum_definition()
  |       +-- write both files to disk
  |
  +-- generate_objects_proto() -> String
  |       +-- for each obj_name in needed_objects (BTreeSet order = sorted):
  |               +-- lookup_object() -- may warn to stderr if not found
  |               +-- for each attr: same resolution as events but uses resolve_object_field_type()
  |
  +-- generate_object_enums_proto() -> String
  |
  +-- generate_enum_value_map() -> Result<String>
  |       +-- for each class -> collect_enum_entries()
  |       +-- for each needed object -> collect_enum_entries()
  |       +-- serde_json::to_string_pretty() | Error::Codegen
  |
  +-- write all files, return stats
```

### Flow 4: Field Type Resolution (Two Parallel Paths)

The codebase has two nearly identical field resolution functions:

| Aspect | `resolve_event_field_type` | `resolve_object_field_type` |
|--------|---------------------------|----------------------------|
| Location | codegen.rs:464-497 | codegen.rs:503-531 |
| Parameters | `class_upper`, `category` | `obj_upper` |
| Enum reference | `ocsf.{v}.events.{cat}.enums.{CLASS}_{ATTR}` | `ocsf.{v}.objects.enums.{OBJ}_{ATTR}` |
| Object ref | Same (`resolve_object_ref`) | Same (`resolve_object_ref`) |
| Primitive | Same (`ocsf_to_proto_type`) | Same (`ocsf_to_proto_type`) |

The only difference between these two functions is the enum package path. This is a candidate for refactoring (pass the package prefix as a parameter), but as-is it is correct and clear.

---

## Lifetime Analysis

### `lookup_object<'a>`

```rust
fn lookup_object<'a>(schema: &'a OcsfSchema, name: &str) -> Option<&'a OcsfObject>
```

This is the only function with an explicit lifetime parameter. It borrows an `OcsfObject` from the schema, meaning:
- The returned reference lives as long as the schema
- No cloning is needed to access object data during generation
- The schema must outlive all generation operations (which it does since `generate()` takes `&OcsfSchema`)

### `classes_by_category`

```rust
let mut classes_by_category: BTreeMap<String, Vec<&OcsfClass>> = BTreeMap::new();
```

Borrows `OcsfClass` references from the schema. The lifetime is implicit (tied to `schema` parameter). This avoids cloning entire class structs.

---

## Enum Map JSON Structure

The `generate_enum_value_map` function produces a JSON file with this structure:

```json
{
  "AUTHENTICATION_ACTIVITY_ID_LOGON": {
    "name": "Logon",
    "value": 1
  },
  "AUTHENTICATION_ACTIVITY_ID_OTHER": {
    "name": "Other",
    "value": 99
  }
}
```

Key format: `{PARENT_SCREAMING}_{ATTR_SCREAMING}_{VARIANT_SCREAMING}`
Value structure: `{"name": caption, "value": integer_key}` -- uses `serde_json::json!()` macro inline, not a dedicated struct.

---

## Proto3 Zero-Value Enforcement

The `write_enum_definition` function enforces proto3's zero-value rule:

```rust
if !entries.iter().any(|(k, _)| *k == 0) {
    writeln!(out, "\t{enum_name}_UNSPECIFIED = 0;").unwrap();
}
```

If OCSF defines a `0` value (e.g., `"0": {"caption": "Unknown"}`), it is used as-is. If no `0` value exists, a synthetic `{ENUM_NAME}_UNSPECIFIED = 0` is added. This is critical for proto3 compliance.

The OCSF `severity_id` and `activity_id` enums both have `"0": "Unknown"`, so the synthetic unspecified is not needed for those. But it would be needed for any OCSF enum that starts at 1.

---

## `to_enum_variant_name` Edge Cases

The function's chain: `chars().map(|c| if alphanumeric: uppercase, else: '_').collect::<String>().replace("__", "_").trim_matches('_')`

**Known behavior:**
- `"TLP:AMBER+STRICT"` -> `"TLP_AMBER_STRICT"` (colon and plus become underscores, collapsed)
- Single-char non-alphanum at boundaries is trimmed
- Only collapses doubles (`"__"` -> `"_"`), not triples. If input has three consecutive non-alphanum chars, result would have `"__"` (two underscores) since `replace("__", "_")` is not applied iteratively. However, this edge case is unlikely in OCSF captions.

---

## Complete Trait Implementation Summary

| Type | Traits |
|------|--------|
| `Error` | `Debug`, `Display`, `std::error::Error`, `From<serde_json::Error>` |
| `OcsfSchema` | `Debug`, `Deserialize` |
| `OcsfClass` | `Debug`, `Deserialize` |
| `OcsfObject` | `Debug`, `Deserialize` |
| `OcsfAttribute` | `Debug`, `Deserialize` |
| `OcsfEnumValue` | `Debug`, `Deserialize` |
| `OcsfDeprecated` | `Debug`, `Deserialize` |
| `GenerationStats` | `Debug`, `Default` |
| `Cli` | `clap::Parser` |
| `Commands` | `clap::Subcommand` |

No types implement `Clone`, `Serialize`, `PartialEq`, `Eq`, `Hash`, or `Send`/`Sync` (though all types are auto-Send/Sync because they contain only owned data or primitives).

---

## Fields Parsed But Never Used in Codegen

These fields are deserialized from JSON but never read during proto generation:

| Type | Field | Used for |
|------|-------|----------|
| `OcsfSchema` | `types` | Nothing |
| `OcsfSchema` | `base_event` | Nothing |
| `OcsfClass` | `description` | Nothing |
| `OcsfClass` | `extends` | Nothing (inheritance already resolved) |
| `OcsfClass` | `category_uid` | Nothing |
| `OcsfClass` | `category_name` | Nothing |
| `OcsfClass` | `profiles` | Nothing |
| `OcsfObject` | `description` | Nothing |
| `OcsfObject` | `extends` | Nothing |
| `OcsfObject` | `observable` | Nothing |
| `OcsfAttribute` | `description` | Nothing |
| `OcsfAttribute` | `requirement` | Nothing |
| `OcsfAttribute` | `group` | Nothing |
| `OcsfAttribute` | `sibling` | Nothing |
| `OcsfAttribute` | `profile` | Nothing |
| `OcsfEnumValue` | `description` | Nothing |

**Fields actively used in codegen:**
- `OcsfSchema`: `version`, `classes`, `objects`
- `OcsfClass`: `name`, `uid`, `caption`, `category`, `attributes`
- `OcsfObject`: `name`, `caption`, `attributes`
- `OcsfAttribute`: `type_name`, `caption`, `is_array`, `object_type`, `enum_values`, `deprecated`
- `OcsfEnumValue`: `caption`
- `OcsfDeprecated`: (presence only -- fields never read by codegen, just `deprecated.is_some()`)

---

## Delta Summary
- New items added: 4 data flow traces, 1 lifetime analysis, enum map JSON structure, zero-value enforcement detail, variant name edge case, unused field inventory (16 fields)
- Existing items refined: Field resolution dual-path documented with comparison table; download path's raw-write behavior clarified
- Remaining gaps: None -- every type, function, field, trait, and data flow in the codebase has been documented

## Novelty Assessment
Novelty: NITPICK
Round 2 findings are refinements of Round 1's type catalog. The data flow traces, lifetime analysis, and unused-field inventory are useful reference material but do not change the domain model. No new entities, relationships, or structural patterns were discovered. The only genuinely new insight is the 16 unused-field inventory, which is a refinement rather than a model change.

## Convergence Declaration
Pass 2 has converged -- findings are nitpicks, not gaps. The domain model is complete at both structural and behavioral levels.

## State Checkpoint
```yaml
pass: 2
round: 2
status: complete
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
