//! `WriteVerbRegistry` — immutable verb set for write-stage grammar productions.
//!
//! At parser initialization, the registry is loaded from a `WriteVerbSource`
//! trait object (`WriteEndpointRegistry` in production; `HashSet<String>` in
//! tests). The registry is immutable after initialization — no hot-reload
//! during a parse call, keeping the parser pure (BC-2.11.004 purity rule).
//!
//! # Architecture Compliance
//! - `WriteVerbSource` trait abstraction allows write parser tests to inject a
//!   static verb set without requiring `WriteEndpointRegistry` to be
//!   initialized with actual sensor specs (Story dev notes).
//! - Registry is built once before `PrismQlParser::parse`; never mutated during
//!   a parse call — no I/O during parsing.
//! - `pub(crate)` visibility on all builder/access functions. Callers outside
//!   `prism-query` must use `PrismQlParser::parse`. (BC-2.11.006 INV-SEC-PERIMETER-001)
//!
//! Story: S-3.06 | BC-2.11.004

use std::collections::HashSet;

use prism_spec_engine::write_endpoint::WriteEndpointRegistry;

// ─────────────────────────────────────────────────────────────────────────────
// WriteVerbSource trait
// ─────────────────────────────────────────────────────────────────────────────

/// Abstraction over write-verb providers.
///
/// Implemented by both `WriteEndpointRegistry` (production) and `HashSet<String>`
/// (test injection). The `WriteVerbRegistry` accepts any `&dyn WriteVerbSource`
/// at initialization time.
///
/// # Implements BC-2.11.004 — Write Parser Extension
pub trait WriteVerbSource {
    /// Returns `true` if `verb` is a registered write verb across any sensor.
    fn is_registered_verb(&self, verb: &str) -> bool;

    /// Returns all registered verbs for the given sensor, in insertion order.
    ///
    /// Used by the parser to provide suggestions in `E-QUERY-023` error messages.
    fn verbs_for_sensor(&self, sensor: &str) -> Vec<String>;

    /// Returns all globally registered verbs across all sensors.
    ///
    /// Used to build the Chumsky `choice()` parser over all known write verbs.
    fn all_verbs(&self) -> Vec<String>;
}

// ─────────────────────────────────────────────────────────────────────────────
// WriteVerbSource implementations
// ─────────────────────────────────────────────────────────────────────────────

/// Production implementation: `WriteEndpointRegistry` as a verb source.
///
/// `verbs_for_sensor` delegates to `WriteEndpointRegistry::verbs_for_sensor`.
/// `all_verbs` is derived by aggregating across all known sensors — the registry
/// does not expose a flat "all verbs" iterator directly, so we use `table_descriptors`.
///
/// # WIRING-EXEMPT: trait delegation to single-call helpers.
impl WriteVerbSource for WriteEndpointRegistry {
    fn is_registered_verb(&self, _verb: &str) -> bool {
        todo!("S-3.06 — WriteEndpointRegistry::is_registered_verb")
    }

    fn verbs_for_sensor(&self, _sensor: &str) -> Vec<String> {
        todo!("S-3.06 — WriteEndpointRegistry::verbs_for_sensor")
    }

    fn all_verbs(&self) -> Vec<String> {
        todo!("S-3.06 — WriteEndpointRegistry::all_verbs")
    }
}

/// Test implementation: `HashSet<String>` as a verb source.
///
/// `verbs_for_sensor` returns all verbs (test sets are not sensor-partitioned).
/// `all_verbs` returns all entries.
///
/// # WIRING-EXEMPT: trait delegation to trivial container methods.
impl WriteVerbSource for HashSet<String> {
    fn is_registered_verb(&self, _verb: &str) -> bool {
        todo!("S-3.06 — HashSet<String>::is_registered_verb")
    }

    fn verbs_for_sensor(&self, _sensor: &str) -> Vec<String> {
        todo!("S-3.06 — HashSet<String>::verbs_for_sensor")
    }

    fn all_verbs(&self) -> Vec<String> {
        todo!("S-3.06 — HashSet<String>::all_verbs")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WriteVerbRegistry
// ─────────────────────────────────────────────────────────────────────────────

/// Immutable verb registry used by the PrismQL write-stage parser.
///
/// Built once from a `WriteVerbSource` at parser initialization.
/// Immutable after construction — no mutation during a parse call.
///
/// # Implements BC-2.11.004 — Write Parser Extension
pub struct WriteVerbRegistry {
    /// Sorted, deduplicated set of all registered write verbs.
    verbs: HashSet<String>,
    /// Sensor → verb list mapping (preserves insertion order per sensor).
    /// Populated by `from_source`; read by `verbs_for_sensor`.
    #[allow(dead_code)] // Used by verbs_for_sensor (stub); will be read in implementation.
    sensor_verbs: std::collections::HashMap<String, Vec<String>>,
}

impl WriteVerbRegistry {
    /// Build a `WriteVerbRegistry` from any `WriteVerbSource`.
    ///
    /// Consumes all verbs from the source and stores them immutably.
    /// Called once before `PrismQlParser::parse`; never called during parsing.
    ///
    /// # Implements BC-2.11.004 — Write Parser Extension
    pub fn from_source(_source: &dyn WriteVerbSource) -> Self {
        todo!("S-3.06 — WriteVerbRegistry::from_source")
    }

    /// Returns `true` if `verb` is a registered write verb.
    ///
    /// Used by the Chumsky parser to distinguish write stages from unknown
    /// identifiers in terminal pipe position.
    ///
    /// # Implements BC-2.11.004 — Write Parser Extension
    pub fn is_write_verb(&self, _s: &str) -> bool {
        todo!("S-3.06 — WriteVerbRegistry::is_write_verb")
    }

    /// Returns the registered write verbs for a given sensor, in insertion order.
    ///
    /// Used to populate `E-QUERY-023` error suggestion lists for the source
    /// sensor referenced in the failing pipeline.
    ///
    /// Returns an empty slice if no verbs are registered for `sensor`.
    ///
    /// # Implements BC-2.11.004 — Write Parser Extension
    pub fn verbs_for_sensor(&self, _sensor: &str) -> Vec<&str> {
        todo!("S-3.06 — WriteVerbRegistry::verbs_for_sensor")
    }

    /// Returns all registered write verbs across all sensors.
    ///
    /// Used by the Chumsky grammar to build a `choice()` over all known verbs
    /// at parser-build time (runtime dynamic grammar — not a static `one_of![]`).
    ///
    /// # Implements BC-2.11.004 — Write Parser Extension
    pub fn all_verbs(&self) -> impl Iterator<Item = &str> {
        todo!("S-3.06 — WriteVerbRegistry::all_verbs");
        // Unreachable after todo!() — satisfies return-type requirement at compile time.
        #[allow(unreachable_code)]
        self.verbs.iter().map(|s| s.as_str())
    }

    /// Returns `true` if no write verbs are registered.
    ///
    /// When the registry is empty, any terminal identifier in pipe position
    /// produces `E-QUERY-023`.
    ///
    /// # GREEN-BY-DESIGN: zero branching, no I/O, ≤ 3 lines, no helpers.
    pub fn is_empty(&self) -> bool {
        self.verbs.is_empty()
    }
}

impl Default for WriteVerbRegistry {
    /// Construct an empty `WriteVerbRegistry`.
    ///
    /// # GREEN-BY-DESIGN: trivial struct construction, no logic.
    fn default() -> Self {
        WriteVerbRegistry {
            verbs: HashSet::new(),
            sensor_verbs: std::collections::HashMap::new(),
        }
    }
}
