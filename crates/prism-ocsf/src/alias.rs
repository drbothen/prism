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
//!
//! # Stub Status (S-1.05 Red Gate)
//!
//! `AliasResolver::resolve()` body is `unimplemented!()`.

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

impl AliasResolver {
    /// Resolves a field name against an `OcsfEvent` using the four-tier priority order.
    ///
    /// # Tier Resolution
    ///
    /// 1. If `field` is one of `"source_sensor"`, `"source_record_type"`, `"client_id"` →
    ///    returns `AliasResult::PrismMetadata(value)` immediately. (BC-2.02.008 postcondition 1)
    /// 2. If `field` resolves against `event.message` via dot-notation recursive descent →
    ///    returns `AliasResult::ProtoField(value)`. Array indexing (e.g. `attacks[0].tactic.name`)
    ///    is supported; out-of-bounds index returns `AliasResult::Absent`. (BC-2.02.008 edge EC-02-014)
    /// 3. If `field` is a key in `event.raw_extensions` →
    ///    returns `AliasResult::RawExtension(value)`. (BC-2.02.008 postcondition 3)
    ///    Note: tier 2 takes precedence over tier 3 for same-named fields. (BC-2.02.008 EC-02-015)
    /// 4. Returns `AliasResult::Absent`. (BC-2.02.008 postcondition 4)
    ///
    /// # Panics
    ///
    /// Never.
    ///
    /// # Stub — body unimplemented (S-1.05 Red Gate).
    pub fn resolve(_field: &str, _event: &OcsfEvent) -> AliasResult {
        unimplemented!("AliasResolver::resolve — S-1.05 stub")
    }
}
