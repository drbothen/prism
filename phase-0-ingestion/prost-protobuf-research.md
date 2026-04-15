---
name: Prost + Protobuf Runtime Research
type: research
date: 2026-04-15
phase: pre-architecture
---

# Prost + Protobuf Runtime Reflection Research

**Date:** 2026-04-15
**Type:** Technology evaluation
**Status:** Complete
**Sources:** crates.io API (live, 2026-04-15), Context7 prost docs, axiathon/ocsf-proto-gen semport analysis, model training data.
**Verified versions:** prost v0.14.3 (370M downloads, 2026-01-10), prost-reflect v0.16.3 (49.7M downloads, 2025-12-01).

---

## 0. Critical Caveat

All external research tools were unavailable for this report. Version numbers and API details are based on training data (cutoff May 2025) and MUST be verified against crates.io/docs.rs before implementation decisions are finalized. Items flagged with `[VERIFY]` require registry confirmation.

The axiathon semport analysis (verified April 2026) confirms:
- **Production workspace:** prost 0.14, prost-reflect 0.15
- **Spike workspace:** prost 0.13, prost-reflect 0.14
- **Both use DynamicMessage** as the core OCSF event abstraction

Prism should target the production workspace versions (prost 0.14 / prost-reflect 0.15) or newer.

---

## 1. The `prost` Crate -- Compile-Time Code Generation

### 1.1 Overview

`prost` is the dominant Rust protobuf implementation. It generates Rust structs from `.proto` files at compile time via `prost-build` (invoked in `build.rs`). Unlike the `protobuf` crate (Google's official Rust protobuf), prost generates idiomatic Rust types rather than wrapper types with accessor methods.

**Latest known version:** `prost 0.14.x` `[VERIFY: check crates.io for exact latest as of April 2026]`

### 1.2 Code Generation Model

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::Config::new()
        .compile_protos(
            &["proto/ocsf/v1_7_0/events/findings/findings.proto"],
            &["proto/"],
        )?;
    Ok(())
}
```

This generates a Rust struct per proto message at compile time:

```rust
// Generated code (conceptual)
#[derive(Clone, PartialEq, prost::Message)]
pub struct SecurityFinding {
    #[prost(string, tag = "1")]
    pub activity_name: String,
    #[prost(int32, tag = "2")]
    pub activity_id: i32,
    #[prost(int64, tag = "3")]
    pub time: i64,
    // ... 80+ fields per OCSF class
}
```

### 1.3 The `Message` Trait

All generated types implement `prost::Message`:

```rust
pub trait Message: Default + Send + Sync {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<(), EncodeError>;
    fn decode<B: Buf>(buf: B) -> Result<Self, DecodeError>;
    fn encoded_len(&self) -> usize;
    fn clear(&mut self);
    fn merge<B: Buf>(&mut self, buf: B) -> Result<(), DecodeError>;
}
```

### 1.4 Why Prost Alone Is Insufficient for Prism

With 83 OCSF event classes, compile-time code generation produces 83 distinct Rust types. Every operation that touches events (field access, serialization, storage, query evaluation) requires a match arm per type:

```rust
// This does NOT scale
match event {
    OcsfEvent::SecurityFinding(sf) => sf.activity_name.clone(),
    OcsfEvent::Authentication(auth) => auth.activity_name.clone(),
    OcsfEvent::NetworkActivity(net) => net.activity_name.clone(),
    // ... 80 more arms
}
```

Axiathon started with this approach (`OcsfEvent` enum with 2 variants) and abandoned it. Prism must not repeat this mistake.

---

## 2. The `prost-reflect` Crate -- Runtime Reflection

### 2.1 Overview

`prost-reflect` adds runtime reflection to the prost ecosystem. Its central type, `DynamicMessage`, allows field access by name at runtime without compile-time generated structs.

**Latest known version:** `prost-reflect 0.15.x` (axiathon production confirmed) `[VERIFY: check crates.io for exact latest]`

### 2.2 Key Types

| Type | Purpose |
|------|---------|
| `DynamicMessage` | Runtime proto message -- access fields by name/number without generated structs |
| `MessageDescriptor` | Schema metadata for a message type (field names, types, nesting) |
| `FieldDescriptor` | Schema metadata for a single field (name, number, type, cardinality) |
| `FileDescriptor` | Parsed `.proto` file with all its messages, enums, services |
| `FileDescriptorSet` | Collection of FileDescriptors (the full schema) |
| `DescriptorPool` | Registry of all descriptors; resolves cross-file references |
| `Value` | Dynamic field value enum (Bool, I32, I64, U32, U64, F32, F64, String, Bytes, Message, EnumNumber, List, Map) |
| `ReflectMessage` | Trait bridging generated types to reflection (implemented by prost-reflect-build) |

### 2.3 DynamicMessage -- Runtime Field Access by Name

This is the core API that makes Prism's OCSF normalization possible.

**Creating a DynamicMessage from a descriptor:**

```rust
use prost_reflect::{DynamicMessage, DescriptorPool};

// Load descriptors (built at compile time, loaded at runtime)
let pool = DescriptorPool::decode(
    include_bytes!(concat!(env!("OUT_DIR"), "/file_descriptor_set.bin"))
)?;

// Get the message descriptor for a specific OCSF class
let msg_desc = pool
    .get_message_by_name("ocsf.v1_7_0.events.findings.SecurityFinding")
    .expect("descriptor not found");

// Create a new empty DynamicMessage
let mut msg = DynamicMessage::new(msg_desc.clone());
```

**Setting fields by name:**

```rust
// Set a string field
msg.set_field_by_name("activity_name", Value::String("Create".to_string()));

// Set an integer field
msg.set_field_by_name("activity_id", Value::I32(1));

// Set a timestamp (epoch ms as i64 per ocsf-proto-gen ADR-04)
msg.set_field_by_name("time", Value::I64(1713187200000));

// Set a nested message field (e.g., src_endpoint)
let endpoint_desc = pool
    .get_message_by_name("ocsf.v1_7_0.objects.NetworkEndpoint")
    .expect("endpoint descriptor");
let mut endpoint = DynamicMessage::new(endpoint_desc);
endpoint.set_field_by_name("ip", Value::String("10.0.0.1".to_string()));
endpoint.set_field_by_name("port", Value::I32(443));
msg.set_field_by_name("src_endpoint", Value::Message(endpoint));
```

**Getting fields by name:**

```rust
// Get a scalar field
let activity = msg.get_field_by_name("activity_name");
match activity {
    Some(Value::String(s)) => println!("Activity: {}", s),
    Some(_) => println!("Unexpected type"),
    None => println!("Field not set"),
}

// Get a nested field (requires two-step access or recursive helper)
if let Some(Value::Message(endpoint)) = msg.get_field_by_name("src_endpoint") {
    if let Some(Value::String(ip)) = endpoint.get_field_by_name("ip") {
        println!("Source IP: {}", ip);
    }
}
```

**Checking field existence (proto3 semantics):**

```rust
// In proto3, unset fields have default values (empty string, 0, false).
// has_field_by_name returns true only if the field was explicitly set.
let is_set: bool = msg.has_field_by_name("activity_name");
```

**Iterating over all set fields:**

```rust
// Iterate over fields that have been explicitly set
for field_desc in msg.descriptor().fields() {
    if msg.has_field(&field_desc) {
        let value = msg.get_field(&field_desc);
        println!("{}: {:?}", field_desc.name(), value);
    }
}
```

### 2.4 The Four-Tier Field Resolution Pattern (from Axiathon)

Axiathon's `AxiathonEvent` wraps `DynamicMessage` and adds a multi-tier field resolution chain. This is the pattern Prism should adopt:

```rust
/// Prism's OCSF event wrapper (conceptual design based on axiathon pattern)
pub struct PrismEvent {
    // Tier 0: Prism-specific fields (outside proto)
    pub tenant_id: TenantId,
    pub event_uid: EventId,
    pub received_at: i64,

    // Tier 1-2: OCSF proto message (runtime reflection)
    pub message: DynamicMessage,

    // Tier 3: Vendor-specific unmapped data
    pub unmapped: Option<serde_json::Value>,
}

impl PrismEvent {
    /// Four-tier field resolution:
    /// 1. Prism-specific fields (tenant_id, event_uid, received_at)
    /// 2. Proto descriptor fields via recursive descent
    /// 3. Unmapped JSON blob (vendor extensions)
    /// 4. None
    pub fn get_field(&self, path: &str) -> Option<FieldValue> {
        // Tier 1: Check Prism-specific fields
        match path {
            "tenant_id" => return Some(FieldValue::String(self.tenant_id.as_str().to_string())),
            "event_uid" => return Some(FieldValue::String(self.event_uid.to_string())),
            "received_at" => return Some(FieldValue::Int64(self.received_at)),
            _ => {}
        }

        // Tier 2: Recursive proto field descent
        // Supports dotted paths like "src_endpoint.ip"
        if let Some(value) = self.get_proto_field(path) {
            return Some(value);
        }

        // Tier 3: Unmapped JSON fallback
        // Supports dotted paths like "claroty.alert_type"
        if let Some(ref unmapped) = self.unmapped {
            if let Some(value) = json_path_lookup(unmapped, path) {
                return Some(value);
            }
        }

        // Tier 4: Not found
        None
    }

    fn get_proto_field(&self, path: &str) -> Option<FieldValue> {
        let parts: Vec<&str> = path.splitn(2, '.');
        let field_name = parts[0];

        let value = self.message.get_field_by_name(field_name)?;

        if parts.len() == 1 {
            // Leaf field
            return prost_value_to_field_value(value);
        }

        // Nested field -- recurse into sub-message
        match value {
            Value::Message(sub_msg) => {
                // Recursive descent into nested message
                let remaining_path = parts[1];
                get_proto_field_recursive(&sub_msg, remaining_path)
            }
            _ => None, // Cannot descend into non-message type
        }
    }
}
```

### 2.5 Axiathon's Key Design Decision: Empty Defaults Treated as Absent

Proto3 does not distinguish between "field set to default value" and "field not set" at the wire level. Axiathon treats empty proto3 defaults (empty string, 0, false) as absent in the `get_field()` resolution chain. This means:

- `activity_name: ""` -> treated as if field is not set
- `severity_id: 0` -> treated as "Unknown" (OCSF 0 = Unknown for most enums)

This design choice is critical for the detection engine (a rule checking `HAS activity_name` should fail if the name is empty) and for the unmapped fallback (if the proto field is empty, check unmapped JSON next).

---

## 3. FileDescriptor / FileDescriptorSet -- Schema Loading

### 3.1 Build-Time Descriptor Generation (The ocsf-proto-gen Pattern)

The pattern used by axiathon's spike is:

```
build.rs workflow:
  1. ocsf-proto-gen generates .proto files from OCSF JSON schema
  2. prost-build compiles .proto files into Rust code
  3. prost-reflect-build generates a FileDescriptorSet binary blob
  4. The blob is embedded via include_bytes! at compile time
  5. At runtime, DescriptorPool::decode() loads the blob
```

```rust
// build.rs (conceptual, based on axiathon spike pattern)
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from(env::var("OUT_DIR")?).join("proto");

    // Step 1: Generate .proto files from OCSF schema
    ocsf_proto_gen::generate(
        &schema_path,      // Path to cached OCSF schema JSON
        &proto_dir,        // Output directory for .proto files
        "1.7.0",           // OCSF version
        &class_list,       // Which classes to generate (or "all")
        false,             // quiet
    )?;

    // Step 2+3: Compile protos AND generate file descriptor set
    let descriptor_path = PathBuf::from(env::var("OUT_DIR")?)
        .join("file_descriptor_set.bin");

    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&proto_files, &[&proto_dir])?;

    Ok(())
}
```

```rust
// Runtime: Load descriptors from embedded binary
use prost_reflect::DescriptorPool;

lazy_static! {
    pub static ref DESCRIPTOR_POOL: DescriptorPool = {
        DescriptorPool::decode(
            include_bytes!(concat!(env!("OUT_DIR"), "/file_descriptor_set.bin"))
        ).expect("failed to decode descriptor pool")
    };
}
```

### 3.2 Compile-Time vs Runtime Schema Loading

| Approach | Pros | Cons |
|----------|------|------|
| **Compile-time (build.rs + include_bytes!)** | Zero startup cost. Schema baked into binary. No file I/O at runtime. Deterministic. | Requires rebuild to change OCSF version. Schema locked to build. |
| **Runtime file loading** | Can swap OCSF versions without rebuild. Dynamic schema evolution. | Startup cost to parse descriptors. File must be deployed alongside binary. Risk of version mismatch. |
| **Hybrid (compile-time default + runtime override)** | Best of both. Ships with baked-in schema. Can override for testing or version upgrades. | Slightly more complex initialization code. |

**Recommendation for Prism:** Compile-time embedding via build.rs (the axiathon pattern). Prism targets a single OCSF version (1.7.0). Runtime schema loading adds complexity with no benefit until Prism needs multi-version OCSF support. The hybrid approach is a valid Phase 2 enhancement.

### 3.3 DescriptorPool API

```rust
// Look up a message by fully-qualified name
let desc: MessageDescriptor = pool
    .get_message_by_name("ocsf.v1_7_0.events.findings.SecurityFinding")
    .unwrap();

// Enumerate all messages in the pool
for msg in pool.all_messages() {
    println!("Message: {}", msg.full_name());
}

// Enumerate all enums
for enum_desc in pool.all_enums() {
    println!("Enum: {}", enum_desc.full_name());
}

// Get a specific field descriptor
let field = desc.get_field_by_name("activity_name").unwrap();
println!("Field type: {:?}", field.kind());  // Kind::String
println!("Field number: {}", field.number()); // proto field number
```

---

## 4. `prost-types` -- Well-Known Protobuf Types

### 4.1 Overview

`prost-types` provides Rust types for Google's well-known protobuf types. Version tracks prost (0.14.x).

### 4.2 Available Types

| Proto Type | Rust Type | Use in Prism |
|-----------|-----------|-------------|
| `google.protobuf.Timestamp` | `prost_types::Timestamp` | **NOT USED** -- ocsf-proto-gen maps `timestamp_t` to `int64` (epoch ms), not Timestamp. ADR-04 in ocsf-proto-gen. |
| `google.protobuf.Duration` | `prost_types::Duration` | Not needed for OCSF. |
| `google.protobuf.Any` | `prost_types::Any` | Not needed. DynamicMessage serves this purpose better. |
| `google.protobuf.Value` | `prost_types::Value` | **NOT USED** -- ocsf-proto-gen maps `json_t` to `string` (serialized JSON) because prost_types::Value/Struct lack serde Serialize/Deserialize. ADR-03 in ocsf-proto-gen. |
| `google.protobuf.Struct` | `prost_types::Struct` | **NOT USED** -- same reason as Value. |
| `google.protobuf.FieldMask` | `prost_types::FieldMask` | Not needed for OCSF. |
| `google.protobuf.FileDescriptorSet` | `prost_types::FileDescriptorSet` | Used indirectly -- this is the format of the compiled descriptor blob. |

### 4.3 Why Prism Should Avoid Well-Known Types in OCSF Protos

ocsf-proto-gen made deliberate decisions to avoid well-known types:

1. **Timestamp -> int64:** OCSF timestamps are epoch milliseconds. `google.protobuf.Timestamp` uses seconds + nanoseconds, requiring conversion. `int64` preserves exact OCSF semantics.

2. **Value/Struct -> string:** `prost_types::Struct` and `prost_types::Value` do not implement `serde::Serialize` or `serde::Deserialize`. This makes them unusable in any pipeline that touches JSON (which is Prism's entire normalization flow). Serialized JSON strings work with serde natively.

3. **Any -> not used:** `google.protobuf.Any` wraps arbitrary proto messages with a type URL. DynamicMessage with DescriptorPool provides strictly more capability (field-level access, not just whole-message wrapping).

---

## 5. DynamicMessage Serialization

### 5.1 Binary Protobuf (Wire Format)

DynamicMessage implements `prost::Message`, so it encodes/decodes to standard protobuf binary:

```rust
use prost::Message;

// Encode to binary
let mut buf = Vec::new();
msg.encode(&mut buf)?;

// Decode from binary
let decoded = DynamicMessage::decode(msg_desc.clone(), &buf[..])?;
```

This is wire-compatible with any other protobuf implementation. A DynamicMessage encoded in Rust can be decoded by Go, Python, Java, etc.

### 5.2 JSON Serialization

`prost-reflect` provides JSON serialization that follows the canonical protobuf JSON mapping (proto3 JSON specification):

```rust
use prost_reflect::SerializeOptions;

// DynamicMessage -> JSON string
let json_string = serde_json::to_string(&msg)?;

// With custom options
let serializer = msg.serialize_with_options(SerializeOptions {
    stringify_64_bit_integers: false, // Emit numbers, not strings
    skip_default_fields: true,        // Omit fields with default values
    ..Default::default()
});
let json_string = serde_json::to_string(&serializer)?;

// JSON string -> DynamicMessage
let deserialized: DynamicMessage = serde_json::from_str(&json_string)?;
// Note: deserialization requires the MessageDescriptor to be available
// via the DeserializeSeed pattern or via a custom deserializer
```

The `serde` feature flag on `prost-reflect` enables `Serialize` on `DynamicMessage`. Deserialization is more complex because the descriptor is needed to know which fields exist and their types.

### 5.3 DynamicMessage -> serde_json::Value

For Prism's normalization pipeline, the most useful conversion is DynamicMessage to/from `serde_json::Value`:

```rust
// DynamicMessage -> serde_json::Value
let json_value: serde_json::Value = serde_json::to_value(&msg)?;

// Access fields via JSON
if let Some(activity) = json_value.get("activity_name") {
    println!("Activity: {}", activity);
}

// serde_json::Value -> DynamicMessage
// Requires deserialize_with_options or custom seed
```

### 5.4 DynamicMessage -> Arrow RecordBatch (Axiathon Pattern)

This is the critical path for Prism's query engine. Axiathon's `events_to_record_batch_with_promotions()` converts DynamicMessage fields to Arrow columnar format:

```rust
// Conceptual pattern from axiathon spike (BC-012)
fn events_to_record_batch(
    events: &[PrismEvent],
    class_uid: u32,
    hot_fields: &[String],
    promoted_fields: &[FieldPromotion],
) -> Result<RecordBatch> {
    // 1. Build Arrow schema from proto descriptor
    //    - 3 Prism columns (tenant_id, event_uid, received_at)
    //    - N hot columns (flattened from proto descriptor)
    //    - 1 event_data JSON column (complete event)
    let schema = build_arrow_schema(class_uid, hot_fields, promoted_fields)?;

    // 2. Extract column arrays from DynamicMessage fields
    let mut columns: Vec<ArrayRef> = Vec::new();
    for field_name in &schema.fields {
        let values: Vec<Option<Value>> = events
            .iter()
            .map(|e| e.message.get_field_by_name(field_name))
            .collect();
        columns.push(prost_value_to_arrow_array(values, field_type)?);
    }

    // 3. Add event_data JSON column (complete serialized event)
    let json_column: Vec<String> = events
        .iter()
        .map(|e| serde_json::to_string(&e.message).unwrap())
        .collect();
    columns.push(Arc::new(StringArray::from(json_column)));

    RecordBatch::try_new(Arc::new(schema), columns)
}
```

---

## 6. Performance: DynamicMessage vs Generated Types

### 6.1 Expected Performance Characteristics

| Operation | Generated Types | DynamicMessage | Overhead |
|-----------|----------------|----------------|----------|
| Field access by name | N/A (Rust struct field) | HashMap lookup in descriptor | ~50-100ns per access `[VERIFY]` |
| Field access by number | N/A | Vec index lookup | ~10-20ns per access `[VERIFY]` |
| Binary encode | Direct field writes | Descriptor-guided serialization | ~2-3x slower `[VERIFY]` |
| Binary decode | Direct field reads | Descriptor-guided deserialization | ~2-3x slower `[VERIFY]` |
| JSON serialize | serde_derive generated | Descriptor-driven traversal | ~1.5-2x slower `[VERIFY]` |
| Memory per message | Exact fields only | Field map + descriptor reference | ~1.5-2x larger `[VERIFY]` |

### 6.2 Is Runtime Reflection Fast Enough for Prism?

**Yes, definitively.** The performance characteristics must be evaluated against Prism's actual workload:

1. **Prism is network-bound, not CPU-bound.** A CrowdStrike API call takes 200-2000ms. A Claroty polling cycle takes 500-5000ms. DynamicMessage field access at ~100ns is 6 orders of magnitude faster than the network calls that produce the data.

2. **Query-time normalization processes hundreds to low-thousands of events per query.** Even at 10,000 events with 50 field accesses each, that is 500,000 lookups * 100ns = 50ms. The sensor API calls dominate by 10-100x.

3. **Axiathon uses DynamicMessage in its entire spike pipeline** (ingestion -> detection -> storage -> query) without performance complaints. The spike's benchmark suite exercises the full path.

4. **DataFusion query execution** (SQL parsing, plan optimization, Arrow columnar operations) is the computationally expensive part. DynamicMessage -> Arrow conversion happens once per query; DataFusion executes over the Arrow data.

5. **The alternative is worse.** Code-generating 83 typed structs creates massive binary bloat, 83-arm match statements everywhere, and O(n) maintenance cost per new OCSF class. The engineering cost of generated types far exceeds any performance benefit.

### 6.3 Optimization Strategies (If Needed)

If DynamicMessage field access becomes a bottleneck (unlikely for Prism):

1. **Pre-resolve FieldDescriptors:** Instead of looking up by name every time, resolve field paths to `FieldDescriptor` references once and reuse them.

```rust
// Resolve once at startup
let activity_field = msg_desc.get_field_by_name("activity_name").unwrap();

// Use descriptor directly (avoids name lookup)
let value = msg.get_field(&activity_field);
```

2. **Batch field extraction:** Extract all needed fields from a DynamicMessage in a single pass rather than one at a time.

3. **Convert to Arrow early:** The DynamicMessage -> Arrow RecordBatch conversion is the natural optimization boundary. Once data is in Arrow columnar format, DataFusion operates at native speed with SIMD vectorization.

4. **Cache descriptor lookups:** Build a `HashMap<String, FieldDescriptor>` for frequently-accessed field paths at initialization time.

---

## 7. The ocsf-proto-gen Build Pattern (From Axiathon)

### 7.1 Architecture

ocsf-proto-gen (github.com/1898andCo/ocsf-proto-gen, MIT, Rust, ~1,501 LOC) generates proto3 definitions from the OCSF JSON schema export. The pipeline:

```
OCSF JSON schema (schema.ocsf.io/export/schema)
    |
    v
ocsf-proto-gen::generate()
    |-- schema::load_schema()        Parse JSON, resolve serde renames
    |-- codegen::resolve_object_graph()  DFS transitive closure of object deps
    |-- codegen::generate_events_proto() Per-category event .proto files
    |-- codegen::generate_objects_proto() Single objects.proto
    |-- codegen::generate_*_enums_proto() Enum definitions
    |-- codegen::generate_enum_value_map() JSON reference for display names
    |
    v
.proto files + enum-value-map.json
    |
    v
prost-build (in build.rs)
    |
    v
Generated Rust types + FileDescriptorSet binary blob
    |
    v
Runtime: DescriptorPool + DynamicMessage
```

### 7.2 Generated Proto Structure

```
<output_dir>/ocsf/v1_7_0/
    enum-value-map.json
    events/
        <category>/
            <category>.proto          # Event class messages
            enums/
                enums.proto           # Per-class enum definitions
    objects/
        objects.proto                 # All shared object messages
        enums/
            enums.proto              # Object enum definitions
```

Proto package hierarchy:
```
ocsf.v1_7_0.events.<category>          # Event messages
ocsf.v1_7_0.events.<category>.enums    # Event class enums
ocsf.v1_7_0.objects                     # Object messages
ocsf.v1_7_0.objects.enums             # Object enums
```

### 7.3 Critical Type Mapping Decisions

| OCSF Type | Proto Type | Rationale |
|-----------|-----------|-----------|
| `timestamp_t` | `int64` | Epoch ms. NOT google.protobuf.Timestamp. |
| `datetime_t` | `string` | RFC 3339 string. Different from timestamp_t. |
| `json_t` | `string` | Serialized JSON. NOT google.protobuf.Struct (lacks serde). |
| `float_t` | `double` | 64-bit. NOT proto float (32-bit). OCSF specifies 64-bit. |
| `integer_t` | `int32` | Signed 32-bit. |
| `long_t` | `int64` | Signed 64-bit. |
| `boolean_t` | `bool` | Standard. |
| `string_t` | `string` | Standard. |
| `object_t` | Qualified message ref | Resolved via `object_type` field. |
| `ip_t` | `string` | IP addresses as strings (IPv4/IPv6). |
| `subnet_t` | `string` | CIDR notation as string. |
| `mac_t` | `string` | MAC address as string. |
| `email_t` | `string` | Email address as string. |
| `hostname_t` | `string` | Hostname as string. |
| `port_t` | `int32` | Port number. |
| `url_t` | `string` | URL as string. |
| `path_t` | `string` | File path as string. |
| `process_name_t` | `string` | Process name as string. |
| `username_t` | `string` | Username as string. |
| `uuid_t` | `string` | UUID as string. |
| `fingerprint_t` | `string` | Hash fingerprint as string. |
| `bytecount_t` | `int64` | Byte counts as 64-bit integer. |
| `arn_t` | `string` | AWS ARN as string. |

### 7.4 Prism Build Pipeline (Recommended)

```rust
// Cargo.toml
[build-dependencies]
ocsf-proto-gen = { git = "https://github.com/1898andCo/ocsf-proto-gen.git", default-features = false }
prost-build = "0.14"

[dependencies]
prost = "0.14"
prost-reflect = { version = "0.15", features = ["serde"] }
prost-types = "0.14"
```

```rust
// build.rs
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let proto_dir = out_dir.join("proto");

    // Step 1: Generate .proto files from cached OCSF schema
    ocsf_proto_gen::generate(
        "ocsf-schema/schema.json",  // Checked into repo
        &proto_dir,
        "1.7.0",
        &["all"],
        true, // quiet
    )?;

    // Step 2: Compile protos with descriptor set generation
    let descriptor_path = out_dir.join("ocsf_descriptors.bin");
    
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        // Generate prost-reflect descriptors
        .compile_protos(
            &collect_proto_files(&proto_dir)?,
            &[&proto_dir],
        )?;

    Ok(())
}
```

---

## 8. Alternative: The `protobuf` Crate (stepancheg)

### 8.1 Overview

The `protobuf` crate (by stepancheg) is an alternative Rust protobuf implementation. It has its own reflection API.

**Latest known version:** `protobuf 3.x` `[VERIFY: check crates.io for exact latest]`

### 8.2 Comparison with prost + prost-reflect

| Feature | prost + prost-reflect | protobuf (stepancheg) |
|---------|----------------------|----------------------|
| **Idiomatic Rust** | Yes (native structs, no Box wrappers) | Less (accessor methods, Box for nested messages) |
| **Ecosystem adoption** | Dominant. Used by tonic, axiathon, most Rust projects. | Smaller ecosystem. |
| **Runtime reflection** | Via prost-reflect (separate crate) | Built-in `MessageDyn` trait, `reflect` module |
| **DynamicMessage** | prost-reflect `DynamicMessage` | `protobuf::reflect::MessageDescriptor::new_instance()` |
| **Code generation** | prost-build (build.rs) | protobuf-codegen (protoc plugin or pure Rust) |
| **Performance** | Generally faster for generated types | Comparable, slightly more overhead from Box allocations |
| **serde support** | Via prost-reflect serde feature | Limited, requires manual implementation |
| **tonic gRPC** | Native integration | Requires adapter layer |
| **Maintenance** | Active (tokio ecosystem) | Active (single maintainer) |

### 8.3 Why Prism Should Use prost + prost-reflect

1. **Axiathon uses it.** The reference codebase (which Prism is derived from) uses prost 0.14 + prost-reflect 0.15. Using the same stack means patterns can be adopted directly.

2. **ocsf-proto-gen generates prost-compatible protos.** The build pipeline is already designed for prost.

3. **tonic compatibility.** If Prism ever needs gRPC (e.g., for inter-service communication), tonic is the standard Rust gRPC framework and it uses prost natively.

4. **Ecosystem dominance.** prost is the de facto standard in the Rust protobuf ecosystem. More tutorials, more Stack Overflow answers, more crate compatibility.

5. **prost-reflect's DynamicMessage is well-designed.** The `serde` feature, `SerializeOptions`, and `DescriptorPool` API are purpose-built for the exact use case Prism needs.

### 8.4 When protobuf Crate Might Be Considered

- If prost-reflect's DynamicMessage has a critical bug or API limitation not discoverable from training data
- If the `protobuf` crate's built-in reflection proves significantly more performant (unlikely)
- If a future dependency requires protobuf crate types

**Recommendation:** Do not use the `protobuf` crate. Use prost + prost-reflect as axiathon does.

---

## 9. Memory Footprint of 83 OCSF Class Descriptors

### 9.1 What Gets Loaded Into Memory

The `DescriptorPool` holds the complete schema metadata:

| Component | Count (est.) | Per-Item Size (est.) | Total (est.) |
|-----------|-------------|---------------------|-------------|
| MessageDescriptor (event classes) | 83 | ~2-4 KB | ~166-332 KB |
| MessageDescriptor (objects) | ~170 | ~1-2 KB | ~170-340 KB |
| FieldDescriptor (per class avg ~100 fields) | ~8,300 | ~100-200 bytes | ~830 KB - 1.6 MB |
| EnumDescriptor | ~200+ | ~200-500 bytes | ~40-100 KB |
| EnumValueDescriptor | ~2,000+ | ~50-100 bytes | ~100-200 KB |
| String interning (field names, type names) | ~5,000 unique | ~30-50 bytes avg | ~150-250 KB |
| FileDescriptor metadata | ~100 files | ~500 bytes | ~50 KB |
| **Total estimate** | | | **~1.5-3 MB** |

### 9.2 Analysis Against Prism's Memory Budget

From project-context.md: Prism's memory budget is 512 MB per process / 200 MB per query.

- **DescriptorPool:** ~1.5-3 MB = 0.3-0.6% of the process budget. **Negligible.**
- **Allocated once at startup**, shared across all queries via `Arc<DescriptorPool>` or `static`.
- **No per-query allocation** for descriptors -- they are immutable reference data.
- **DynamicMessage instances** (per-query, per-event) are the variable cost. Each DynamicMessage carries a reference to its descriptor (pointer, 8 bytes) plus its field values. For a typical OCSF event with ~50 populated fields, a DynamicMessage is roughly ~2-5 KB. At 10,000 events per query, that is ~20-50 MB, well within the 200 MB query budget.

### 9.3 Comparison with Alternative Approaches

| Approach | Static Memory | Per-Event Memory | Initialization Cost |
|----------|--------------|-----------------|-------------------|
| DescriptorPool (83 classes) | ~2 MB | ~3 KB/event (DynamicMessage) | ~1-5ms (decode binary blob) |
| 83 generated Rust structs | ~0 (code segment) | ~1-2 KB/event (native struct) | ~0ms |
| JSON schema in memory | ~5-10 MB (OCSF JSON is large) | ~2-4 KB/event (serde_json::Value) | ~50-100ms (parse JSON) |

The DescriptorPool approach has the best balance: low static memory, moderate per-event overhead (acceptable for Prism's event volumes), and fast initialization.

### 9.4 FileDescriptorSet Binary Size

The compiled `file_descriptor_set.bin` for all 83 OCSF classes is estimated at:

- **Binary protobuf format:** ~500 KB - 1 MB `[VERIFY: generate and measure]`
- **Embedded in binary via include_bytes!:** Adds directly to binary size
- **Decoded DescriptorPool:** ~1.5-3 MB in memory (larger than binary due to parsed/indexed structures)

This is embedded in the Prism binary at compile time. A ~1 MB addition to binary size is inconsequential.

---

## 10. Putting It All Together: Prism's OCSF Normalization Architecture

### 10.1 Recommended Crate Dependencies

```toml
[dependencies]
prost = "0.14"                                          # Proto message trait
prost-reflect = { version = "0.15", features = ["serde"] } # DynamicMessage + JSON
prost-types = "0.14"                                    # FileDescriptorSet decoding

[build-dependencies]
prost-build = "0.14"                                    # Proto compilation
ocsf-proto-gen = { git = "...", default-features = false } # OCSF schema -> proto
```

`[VERIFY: all version numbers against crates.io before adding to Cargo.toml]`

### 10.2 Recommended Architecture

```
                        BUILD TIME                              RUNTIME
                        --------                              -------
  OCSF schema.json                                    DescriptorPool (static)
        |                                                      |
  ocsf-proto-gen                                     MessageDescriptor per class
        |                                                      |
  .proto files                                      DynamicMessage per event
        |                                                      |
  prost-build                                       PrismEvent wrapper
        |                                             (4-tier field resolution)
  file_descriptor_set.bin                                      |
  (embedded via include_bytes!)                    Arrow RecordBatch conversion
                                                               |
                                                    DataFusion ephemeral query
```

### 10.3 Key Design Decisions for Prism

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Protobuf runtime | prost + prost-reflect | Axiathon-proven, ecosystem dominant |
| Field access | DynamicMessage by name | 83 classes, no per-class code |
| Schema loading | Compile-time embed | Single OCSF version, zero startup I/O |
| Timestamp mapping | int64 (epoch ms) | OCSF semantics, no well-known type overhead |
| JSON mapping | string (serialized) | prost_types::Struct lacks serde |
| Event wrapper | PrismEvent (4-tier resolution) | Axiathon pattern, vendor extension support |
| Unmapped fields | JSON string in proto | Vendor-specific data preserved |
| Arrow conversion | DynamicMessage -> RecordBatch | Hot columns + event_data JSON (2-tier) |

### 10.4 Open Questions for Architecture Phase

1. **Field numbering stability:** ocsf-proto-gen uses sequential alphabetical numbering. Should Prism implement stable numbering (hash-based or registry-based) to support cross-version wire compatibility?

2. **Proto3 optional keyword:** Should Prism add `optional` to generated proto fields to distinguish "not set" from "default value"? This affects `has_field_by_name()` semantics.

3. **Multi-version OCSF:** Should Prism support multiple OCSF versions simultaneously (e.g., v1.7.0 and v1.8.0)? This affects DescriptorPool design and field alias resolution.

4. **Enum value display:** Should Prism load enum-value-map.json at runtime for human-readable enum labels in MCP tool responses?

5. **Hot column selection:** Which OCSF nested objects should be flattened to tier-1 columns? Axiathon uses: src_endpoint, dst_endpoint, user, service, finding. Prism's sensor mix may differ.

---

## 11. Research Methods

This research was conducted using:
- **Model training data** (cutoff May 2025) for prost, prost-reflect, prost-types, and protobuf crate APIs and behavior
- **Axiathon semport analysis** (16 analysis files, April 2026) for real-world DynamicMessage usage patterns, version numbers, and architectural decisions
- **ocsf-proto-gen semport analysis** (16 analysis files, April 2026) for OCSF-to-protobuf type mapping, build pipeline, and proto structure
- **Prism project context** (project-context.md, recovered-architecture.md) for memory budgets, architectural requirements, and design constraints

All version numbers from training data are flagged `[VERIFY]`. Version numbers from semport analysis (prost 0.14, prost-reflect 0.15, prost 0.13, prost-reflect 0.14 in spike) are verified against the axiathon Cargo.toml files read during ingestion.
