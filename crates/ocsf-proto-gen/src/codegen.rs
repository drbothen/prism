//! Protocol Buffer code generation from OCSF schema.
//!
//! Generates `.proto` files from an [`OcsfSchema`], including:
//! - Event class messages with all attributes as fields
//! - Shared object messages referenced by event classes
//! - Per-class and shared-object enum definitions
//! - An enum-value-map.json reference file
//!
//! The generated output is deterministic: identical input always produces
//! byte-identical output. Fields are sorted alphabetically and numbered
//! sequentially.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::path::Path;

use crate::error::{Error, Result};
use crate::schema::{OcsfAttribute, OcsfClass, OcsfObject, OcsfSchema};
use crate::type_map::{
    ocsf_to_proto_type, sanitize_object_name, to_enum_variant_name, to_pascal_case,
    to_screaming_snake,
};

/// Statistics collected during generation for reporting.
#[derive(Debug, Default)]
pub struct GenerationStats {
    pub classes_generated: usize,
    pub objects_generated: usize,
    pub enums_generated: usize,
    pub deprecated_fields_skipped: usize,
    pub string_enum_fields_skipped: usize,
    pub unknown_types_defaulted: usize,
}

/// Generate proto files for the requested event classes.
///
/// Resolves the transitive object dependency graph, generates proto files
/// for events, objects, and enums, and writes them to `output_dir`.
///
/// Returns generation statistics for reporting.
pub fn generate(
    schema: &OcsfSchema,
    class_names: &[String],
    output_dir: &Path,
) -> Result<GenerationStats> {
    let version_slug = version_to_slug(&schema.version);
    let mut stats = GenerationStats::default();

    // Validate all requested classes exist.
    for name in class_names {
        if !schema.classes.contains_key(name.as_str()) {
            let available: Vec<&str> = schema.classes.keys().map(|s| s.as_str()).collect();
            return Err(Error::ClassNotFound {
                name: name.clone(),
                available: if available.len() > 10 {
                    format!(
                        "{} ... and {} more",
                        available[..10].join(", "),
                        available.len() - 10
                    )
                } else {
                    available.join(", ")
                },
            });
        }
    }

    // Resolve which objects are needed (transitive closure via BFS).
    let needed_objects = resolve_object_graph(schema, class_names);

    // Group classes by category for file organization.
    let mut classes_by_category: BTreeMap<String, Vec<&OcsfClass>> = BTreeMap::new();
    for name in class_names {
        let cls = &schema.classes[name.as_str()];
        classes_by_category
            .entry(cls.category.clone())
            .or_default()
            .push(cls);
    }

    // Generate event proto files per category.
    for (category, classes) in &classes_by_category {
        let events_proto = generate_events_proto(
            &version_slug,
            category,
            classes,
            &schema.objects,
            &mut stats,
        );
        let enums_proto = generate_class_enums_proto(&version_slug, category, classes, &mut stats);

        let category_dir = output_dir
            .join("ocsf")
            .join(&version_slug)
            .join("events")
            .join(category);
        write_file(
            &category_dir.join(format!("{category}.proto")),
            &events_proto,
        )?;
        write_file(
            &category_dir.join("enums").join("enums.proto"),
            &enums_proto,
        )?;
    }
    stats.classes_generated = class_names.len();

    // Generate shared objects proto.
    let objects_proto = generate_objects_proto(&version_slug, schema, &needed_objects, &mut stats);
    let object_enums_proto =
        generate_object_enums_proto(&version_slug, schema, &needed_objects, &mut stats);

    let objects_dir = output_dir.join("ocsf").join(&version_slug).join("objects");
    write_file(&objects_dir.join("objects.proto"), &objects_proto)?;
    write_file(
        &objects_dir.join("enums").join("enums.proto"),
        &object_enums_proto,
    )?;
    stats.objects_generated = needed_objects.len();

    // Generate enum-value-map.json reference.
    let enum_map = generate_enum_value_map(schema, class_names, &needed_objects)?;
    write_file(
        &output_dir
            .join("ocsf")
            .join(&version_slug)
            .join("enum-value-map.json"),
        &enum_map,
    )?;

    Ok(stats)
}

// ── Object graph resolution ────────────────────────────────────────────

/// Compute the transitive closure of all objects referenced by the requested
/// event classes via BFS.
///
/// Starting from objects directly referenced by event class attributes,
/// follows `object_type` references recursively until no new objects are
/// found. Returns sanitized object names (extension prefixes stripped).
fn resolve_object_graph(schema: &OcsfSchema, class_names: &[String]) -> BTreeSet<String> {
    let mut needed: BTreeSet<String> = BTreeSet::new();
    let mut queue: Vec<String> = Vec::new();

    // Seed with objects directly referenced by requested classes.
    for name in class_names {
        if let Some(cls) = schema.classes.get(name.as_str()) {
            for attr in cls.attributes.values() {
                if let Some(obj_type) = &attr.object_type {
                    let key = sanitize_object_name(obj_type);
                    if needed.insert(key.clone()) {
                        queue.push(obj_type.clone());
                    }
                }
            }
        }
    }

    // BFS: follow object → object references.
    while let Some(obj_ref) = queue.pop() {
        if let Some(obj) = lookup_object(schema, &obj_ref) {
            for attr in obj.attributes.values() {
                if let Some(obj_type) = &attr.object_type {
                    let key = sanitize_object_name(obj_type);
                    if needed.insert(key.clone()) {
                        queue.push(obj_type.clone());
                    }
                }
            }
        }
    }

    needed
}

/// Look up an object by name, handling extension-prefixed names.
///
/// OCSF extension objects use path-prefixed names (e.g., `"win/win_service"`).
/// This function tries the original name first, then the sanitized name,
/// then searches all objects by sanitized name comparison.
fn lookup_object<'a>(schema: &'a OcsfSchema, name: &str) -> Option<&'a OcsfObject> {
    schema.objects.get(name).or_else(|| {
        let sanitized = sanitize_object_name(name);
        schema.objects.get(&sanitized).or_else(|| {
            schema
                .objects
                .values()
                .find(|o| sanitize_object_name(&o.name) == sanitized)
        })
    })
}

// ── Event class proto generation ───────────────────────────────────────

fn generate_events_proto(
    version_slug: &str,
    category: &str,
    classes: &[&OcsfClass],
    objects: &BTreeMap<String, OcsfObject>,
    stats: &mut GenerationStats,
) -> String {
    let mut out = String::new();

    writeln!(out, "syntax = \"proto3\";").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "package ocsf.{version_slug}.events.{category};").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "import \"ocsf/{version_slug}/events/{category}/enums/enums.proto\";"
    )
    .unwrap();
    writeln!(out).unwrap();
    writeln!(out, "import \"ocsf/{version_slug}/objects/objects.proto\";").unwrap();

    for cls in classes {
        let class_upper = to_screaming_snake(&cls.name);

        writeln!(out).unwrap();
        writeln!(out, "// Event: {category}").unwrap();
        writeln!(out, "// Class UID: {}", cls.uid).unwrap();
        writeln!(out, "message {} {{", to_pascal_case(&cls.name)).unwrap();

        let mut field_num = 1u32;
        for (attr_name, attr) in &cls.attributes {
            if attr.deprecated.is_some() {
                stats.deprecated_fields_skipped += 1;
                continue;
            }

            let (repeated, proto_type) = resolve_event_field_type(
                attr,
                attr_name,
                &class_upper,
                version_slug,
                category,
                objects,
                stats,
            );
            let repeated_kw = if repeated { "repeated " } else { "" };

            writeln!(
                out,
                "\t{repeated_kw}{proto_type} {attr_name} = {field_num}; // Caption: {};",
                attr.caption
            )
            .unwrap();
            field_num += 1;
        }

        writeln!(out, "}}").unwrap();
    }

    out
}

// ── Class enum generation ──────────────────────────────────────────────

fn generate_class_enums_proto(
    version_slug: &str,
    category: &str,
    classes: &[&OcsfClass],
    stats: &mut GenerationStats,
) -> String {
    let mut out = String::new();

    writeln!(out, "syntax = \"proto3\";").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "package ocsf.{version_slug}.events.{category}.enums;").unwrap();

    for cls in classes {
        let class_upper = to_screaming_snake(&cls.name);

        for (attr_name, attr) in &cls.attributes {
            if attr.deprecated.is_some() {
                continue;
            }
            let Some(enum_vals) = &attr.enum_values else {
                continue;
            };
            if !is_integer_enum(enum_vals) {
                continue;
            }

            let attr_upper = to_screaming_snake(attr_name);
            let enum_name = format!("{class_upper}_{attr_upper}");

            write_enum_definition(&mut out, &enum_name, enum_vals);
            stats.enums_generated += 1;
        }
    }

    out
}

// ── Object proto generation ────────────────────────────────────────────

fn generate_objects_proto(
    version_slug: &str,
    schema: &OcsfSchema,
    needed_objects: &BTreeSet<String>,
    stats: &mut GenerationStats,
) -> String {
    let mut out = String::new();

    writeln!(out, "syntax = \"proto3\";").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "package ocsf.{version_slug}.objects;").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "import \"ocsf/{version_slug}/objects/enums/enums.proto\";"
    )
    .unwrap();

    for obj_name in needed_objects {
        let obj = lookup_object(schema, obj_name);
        let Some(obj) = obj else {
            eprintln!("warning: object '{obj_name}' referenced but not found in schema");
            continue;
        };
        let obj_upper = to_screaming_snake(obj_name);

        writeln!(out).unwrap();
        writeln!(out, "message {} {{", to_pascal_case(obj_name)).unwrap();

        let mut field_num = 1u32;
        for (attr_name, attr) in &obj.attributes {
            if attr.deprecated.is_some() {
                stats.deprecated_fields_skipped += 1;
                continue;
            }

            let (repeated, proto_type) = resolve_object_field_type(
                attr,
                attr_name,
                &obj_upper,
                version_slug,
                &schema.objects,
                stats,
            );
            let repeated_kw = if repeated { "repeated " } else { "" };

            writeln!(
                out,
                "\t{repeated_kw}{proto_type} {attr_name} = {field_num}; // Caption: {};",
                attr.caption
            )
            .unwrap();
            field_num += 1;
        }

        writeln!(out, "}}").unwrap();
    }

    out
}

// ── Object enum generation ─────────────────────────────────────────────

fn generate_object_enums_proto(
    version_slug: &str,
    schema: &OcsfSchema,
    needed_objects: &BTreeSet<String>,
    stats: &mut GenerationStats,
) -> String {
    let mut out = String::new();

    writeln!(out, "syntax = \"proto3\";").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "package ocsf.{version_slug}.objects.enums;").unwrap();

    for obj_name in needed_objects {
        let obj = lookup_object(schema, obj_name);
        let Some(obj) = obj else {
            continue;
        };
        let obj_upper = to_screaming_snake(obj_name);

        for (attr_name, attr) in &obj.attributes {
            if attr.deprecated.is_some() {
                continue;
            }
            let Some(enum_vals) = &attr.enum_values else {
                continue;
            };
            if !is_integer_enum(enum_vals) {
                continue;
            }

            let attr_upper = to_screaming_snake(attr_name);
            let enum_name = format!("{obj_upper}_{attr_upper}");

            write_enum_definition(&mut out, &enum_name, enum_vals);
            stats.enums_generated += 1;
        }
    }

    out
}

// ── Enum value map (JSON reference) ────────────────────────────────────

fn generate_enum_value_map(
    schema: &OcsfSchema,
    class_names: &[String],
    needed_objects: &BTreeSet<String>,
) -> Result<String> {
    let mut map: BTreeMap<String, serde_json::Value> = BTreeMap::new();

    for name in class_names {
        if let Some(cls) = schema.classes.get(name.as_str()) {
            let class_upper = to_screaming_snake(&cls.name);
            collect_enum_entries(&class_upper, &cls.attributes, &mut map);
        }
    }

    for obj_name in needed_objects {
        if let Some(obj) = lookup_object(schema, obj_name) {
            let obj_upper = to_screaming_snake(obj_name);
            collect_enum_entries(&obj_upper, &obj.attributes, &mut map);
        }
    }

    serde_json::to_string_pretty(&map)
        .map_err(|e| Error::Codegen(format!("serializing enum map: {e}")))
}

fn collect_enum_entries(
    prefix: &str,
    attributes: &BTreeMap<String, OcsfAttribute>,
    map: &mut BTreeMap<String, serde_json::Value>,
) {
    for (attr_name, attr) in attributes {
        let Some(enum_vals) = &attr.enum_values else {
            continue;
        };
        if !is_integer_enum(enum_vals) {
            continue;
        }
        let attr_upper = to_screaming_snake(attr_name);
        let enum_name = format!("{prefix}_{attr_upper}");

        for (key_str, val) in enum_vals {
            if let Ok(key) = key_str.parse::<i32>() {
                let variant_name = to_enum_variant_name(&val.caption);
                let full_name = format!("{enum_name}_{variant_name}");
                map.insert(
                    full_name,
                    serde_json::json!({"name": val.caption, "value": key}),
                );
            }
        }
    }
}

// ── Field type resolution ──────────────────────────────────────────────

/// Resolve the proto type for an event class attribute.
///
/// For integer-keyed enum attributes, returns a qualified reference to the
/// generated enum type (e.g., `ocsf.v1_7_0.events.iam.enums.AUTHENTICATION_ACTIVITY_ID`).
fn resolve_event_field_type(
    attr: &OcsfAttribute,
    attr_name: &str,
    class_upper: &str,
    version_slug: &str,
    category: &str,
    objects: &BTreeMap<String, OcsfObject>,
    stats: &mut GenerationStats,
) -> (bool, String) {
    let repeated = attr.is_array;

    // Object references → qualified message type.
    if attr.type_name == "object_t" {
        return resolve_object_ref(attr, version_slug, objects, repeated, stats);
    }

    // Integer-keyed enum → qualified enum type reference.
    if let Some(enum_vals) = &attr.enum_values {
        if is_integer_enum(enum_vals) {
            let attr_upper = to_screaming_snake(attr_name);
            let enum_type =
                format!("ocsf.{version_slug}.events.{category}.enums.{class_upper}_{attr_upper}");
            return (repeated, enum_type);
        }
        stats.string_enum_fields_skipped += 1;
    }

    // Primitive type.
    let proto_type = ocsf_to_proto_type(&attr.type_name).unwrap_or_else(|| {
        stats.unknown_types_defaulted += 1;
        "string"
    });
    (repeated, proto_type.to_string())
}

/// Resolve the proto type for an object attribute.
///
/// Same as event field resolution but enum references go to the objects
/// enum package instead of a per-event-class package.
fn resolve_object_field_type(
    attr: &OcsfAttribute,
    attr_name: &str,
    obj_upper: &str,
    version_slug: &str,
    objects: &BTreeMap<String, OcsfObject>,
    stats: &mut GenerationStats,
) -> (bool, String) {
    let repeated = attr.is_array;

    if attr.type_name == "object_t" {
        return resolve_object_ref(attr, version_slug, objects, repeated, stats);
    }

    if let Some(enum_vals) = &attr.enum_values {
        if is_integer_enum(enum_vals) {
            let attr_upper = to_screaming_snake(attr_name);
            let enum_type = format!("ocsf.{version_slug}.objects.enums.{obj_upper}_{attr_upper}");
            return (repeated, enum_type);
        }
        stats.string_enum_fields_skipped += 1;
    }

    let proto_type = ocsf_to_proto_type(&attr.type_name).unwrap_or_else(|| {
        stats.unknown_types_defaulted += 1;
        "string"
    });
    (repeated, proto_type.to_string())
}

/// Resolve an `object_t` attribute to a qualified proto message reference.
///
/// If the referenced object has no non-deprecated attributes (e.g., the OCSF
/// base `object` type used by the `unmapped` field), emits `string` instead —
/// an empty proto message cannot hold data, so `string` (for JSON) is correct.
fn resolve_object_ref(
    attr: &OcsfAttribute,
    version_slug: &str,
    objects: &BTreeMap<String, OcsfObject>,
    repeated: bool,
    stats: &mut GenerationStats,
) -> (bool, String) {
    let obj_type = attr.object_type.as_deref().unwrap_or("unknown");
    let sanitized = sanitize_object_name(obj_type);

    let obj = objects
        .get(obj_type)
        .or_else(|| objects.get(&sanitized))
        .or_else(|| {
            objects
                .values()
                .find(|o| sanitize_object_name(&o.name) == sanitized)
        });

    let Some(obj) = obj else {
        eprintln!("warning: object type '{obj_type}' not found, defaulting to string");
        stats.unknown_types_defaulted += 1;
        return (repeated, "string".to_string());
    };

    // Empty objects (no non-deprecated attributes) produce empty proto messages
    // that cannot hold data. Emit `string` instead so the field can carry JSON.
    // This handles the OCSF `unmapped` field (type: object_t, object_type: object).
    let has_fields = obj.attributes.values().any(|a| a.deprecated.is_none());
    if !has_fields {
        return (repeated, "string".to_string());
    }

    let pascal = to_pascal_case(&sanitized);
    let qualified = format!("ocsf.{version_slug}.objects.{pascal}");
    (repeated, qualified)
}

// ── Shared helpers ─────────────────────────────────────────────────────

/// Check if an enum has integer keys (valid for proto enum) vs string keys.
///
/// OCSF uses both formats:
/// - Integer-keyed: `{"0": "Unknown", "1": "Logon"}` → becomes proto `enum`
/// - String-keyed: `{"GET": "Get", "POST": "Post"}` → stays as `string` field
fn is_integer_enum(enum_values: &BTreeMap<String, crate::schema::OcsfEnumValue>) -> bool {
    enum_values.keys().all(|k| k.parse::<i32>().is_ok())
}

/// Write a proto enum definition to the output string.
fn write_enum_definition(
    out: &mut String,
    enum_name: &str,
    enum_vals: &BTreeMap<String, crate::schema::OcsfEnumValue>,
) {
    // Collect and sort by integer value.
    let mut entries: Vec<(i32, String)> = Vec::new();
    for (key_str, val) in enum_vals {
        if let Ok(key) = key_str.parse::<i32>() {
            let variant_name = to_enum_variant_name(&val.caption);
            entries.push((key, variant_name));
        }
    }
    entries.sort_by_key(|(k, _)| *k);

    writeln!(out).unwrap();
    writeln!(out, "enum {enum_name} {{").unwrap();

    // Proto3 requires the first enum value to be 0.
    // If OCSF doesn't define a 0 value, add a synthetic UNSPECIFIED.
    if !entries.iter().any(|(k, _)| *k == 0) {
        writeln!(out, "\t{enum_name}_UNSPECIFIED = 0;").unwrap();
    }

    for (key, variant_name) in &entries {
        writeln!(out, "\t{enum_name}_{variant_name} = {key};").unwrap();
    }

    writeln!(out, "}}").unwrap();
}

/// Convert an OCSF version string to a proto package slug.
///
/// `"1.7.0"` → `"v1_7_0"`, `"1.8.0-dev"` → `"v1_8_0_dev"`.
fn version_to_slug(version: &str) -> String {
    format!("v{}", version.replace(['.', '-'], "_"))
}

/// Write content to a file, creating parent directories as needed.
fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::Write {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    std::fs::write(path, content).map_err(|e| Error::Write {
        path: path.to_path_buf(),
        source: e,
    })?;
    Ok(())
}
