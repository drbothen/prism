//! Four-tier field alias resolution for `OcsfEvent`. (BC-2.02.008)
//!
//! Resolution order (deterministic — same input always produces same output):
//!
//!   Tier 1 — Prism metadata: `source_sensor`, `source_record_type`, `client_id`
//!   Tier 2 — Proto descriptor fields: recursive descent into `DynamicMessage` with
//!             dot notation; array indexing supported (e.g., `attacks[0].technique.name`)
//!   Tier 3 — raw_extensions JSON: unmapped vendor fields by original name
//!   Tier 4 — Absent: field not found in any tier → `AliasResult::Absent`
//!
//! The first tier that produces a value wins; later tiers are not consulted.
//! (BC-2.02.008 postcondition, invariant)

use prost_reflect::ReflectMessage;
use serde_json::Value as JsonValue;

use crate::event::OcsfEvent;

/// The outcome of a four-tier field alias resolution call. (BC-2.02.008 postconditions)
#[derive(Debug, PartialEq)]
pub enum AliasResult {
    /// Tier 1: field resolved to a Prism metadata value (`source_sensor`, `client_id`, etc.).
    PrismMetadata(String),

    /// Tier 2: field resolved via recursive descent into the `DynamicMessage`.
    ProtoField(JsonValue),

    /// Tier 3: field resolved from `raw_extensions` JSON blob.
    RawExtension(JsonValue),

    /// Tier 4: field absent from all tiers — not an error.
    Absent,
}

/// Four-tier field alias resolver. (BC-2.02.008)
pub struct AliasResolver;

/// Tier-1 Prism metadata field names (BC-2.02.008 postcondition 1).
const PRISM_METADATA_FIELDS: &[&str] = &["source_sensor", "source_record_type", "client_id"];

impl AliasResolver {
    /// Resolves a field name against an `OcsfEvent` using the four-tier priority order.
    ///
    /// # Tier Resolution
    ///
    /// 1. If `field` is one of `"source_sensor"`, `"source_record_type"`, `"client_id"` →
    ///    returns `AliasResult::PrismMetadata(value)` immediately. (BC-2.02.008 postcondition 1)
    /// 2. If the top-level component of `field` is a known field in the `DynamicMessage`'s
    ///    proto descriptor → returns `AliasResult::ProtoField(value)` via recursive descent.
    ///    Array indexing (e.g. `attacks[0].tactic.name`) is supported;
    ///    out-of-bounds index returns `AliasResult::Absent`. (BC-2.02.008 edge EC-02-014)
    ///    OCSF proto fields take precedence over raw_extensions for the same name (EC-02-015).
    /// 3. If `field` is a key in `event.raw_extensions` →
    ///    returns `AliasResult::RawExtension(value)`. (BC-2.02.008 postcondition 3)
    /// 4. Returns `AliasResult::Absent`. (BC-2.02.008 postcondition 4)
    ///
    /// # Panics
    ///
    /// Never.
    pub fn resolve(field: &str, event: &OcsfEvent) -> AliasResult {
        // Tier 1: Prism metadata fields (BC-2.02.008 postcondition 1)
        if PRISM_METADATA_FIELDS.contains(&field) {
            let value = match field {
                "source_sensor" => event.source_sensor.clone(),
                "source_record_type" => event.source_record_type.clone(),
                "client_id" => event.client_id.clone(),
                _ => unreachable!("PRISM_METADATA_FIELDS contains only the three fields above"),
            };
            return AliasResult::PrismMetadata(value);
        }

        // Tier 2: Proto descriptor fields via dot-notation recursive descent.
        // (BC-2.02.008 postcondition 2, EC-02-014, EC-02-015)
        //
        // We first check whether the top-level path component is a KNOWN field in the
        // descriptor. If yes, tier 2 "claims" the field — we either return the value or
        // Absent (if the path resolves to an out-of-bounds index). We do NOT fall through
        // to tier 3 for descriptor-claimed fields (EC-02-015).
        //
        // If the top-level component is NOT in the descriptor, we fall through to tier 3.
        let top_level = Self::top_level_field_name(field);
        let desc = event.message.descriptor();
        if desc.get_field_by_name(top_level).is_some() {
            // The descriptor knows about this field — tier 2 owns it.
            // Serialize the message to JSON and navigate the path.
            if let Ok(msg_json) = serde_json::to_value(&event.message) {
                if let Some(val) = Self::resolve_path_in_json(field, &msg_json) {
                    return AliasResult::ProtoField(val);
                }
            }
            // Field is in the descriptor but not set (default) or path was out of bounds.
            return AliasResult::Absent;
        }

        // Tier 3: raw_extensions JSON (BC-2.02.008 postcondition 3)
        // Only reached for fields NOT claimed by tier 2's descriptor.
        if let Some(val) = event.raw_extensions.get(field) {
            return AliasResult::RawExtension(val.clone());
        }

        // Tier 4: Absent (BC-2.02.008 postcondition 4)
        AliasResult::Absent
    }

    /// Extracts the top-level field name from a dot-notation path.
    ///
    /// `"device.hostname"` → `"device"`, `"attacks[0].tactic.name"` → `"attacks"`,
    /// `"severity_id"` → `"severity_id"`.
    fn top_level_field_name(path: &str) -> &str {
        // Strip any array index suffix first: `attacks[0]...` → `attacks`
        let first_component = path.split('.').next().unwrap_or(path);
        // Strip `[N]` suffix if present
        first_component
            .find('[')
            .map(|i| &first_component[..i])
            .unwrap_or(first_component)
    }

    /// Resolves a dot-notation path (with optional array indexing) against a JSON value.
    ///
    /// Supports paths like `"device.hostname"`, `"attacks[0].technique.name"`.
    /// Returns `None` if the path cannot be resolved (including out-of-bounds indices).
    /// (BC-2.02.008 EC-02-014)
    fn resolve_path_in_json(path: &str, value: &JsonValue) -> Option<JsonValue> {
        // Split the path into components at '.' boundaries, handling array indices like `[0]`
        let components: Vec<&str> = path.split('.').collect();
        let mut owned_values: Vec<JsonValue> = Vec::new();
        let mut current: &JsonValue = value;

        for component in &components {
            // Check for array indexing: `field[N]`
            if let Some(bracket_pos) = component.find('[') {
                let field_name = &component[..bracket_pos];
                let rest = &component[bracket_pos..];

                // Navigate to the field first
                if !field_name.is_empty() {
                    let obj = current.as_object()?;
                    let field_val = obj.get(field_name)?;
                    owned_values.push(field_val.clone());
                    current = owned_values.last().expect("just pushed");
                }

                // Parse the index: `[N]`
                let close = rest.find(']')?;
                let idx_str = &rest[1..close];
                let idx: usize = idx_str.parse().ok()?;
                let arr = current.as_array()?;
                let item = arr.get(idx)?;
                owned_values.push(item.clone());
                current = owned_values.last().expect("just pushed");
            } else {
                // Plain field name
                let obj = current.as_object()?;
                let field_val = obj.get(*component)?;
                owned_values.push(field_val.clone());
                current = owned_values.last().expect("just pushed");
            }
        }

        Some(current.clone())
    }
}
