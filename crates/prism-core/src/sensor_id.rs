//! `SensorId` — open newtype identifying a sensor by string key.
//!
//! Replaces the closed `SensorType` enum (ADR-023 §C1, BC-2.01.013).
//! Sensor identity is a runtime string value (`Arc<str>` payload), not a
//! compile-time enum variant. This unblocks the plugin-only sensor
//! architecture where external sensors can be added without recompiling.
//!
//! # Story: S-PLUGIN-PREREQ-A
//! # BC: BC-2.01.013 — DataSource Trait: Spec-Driven Adapter Pattern
//! # VP: VP-PLUGIN-001 — SensorId open-newtype replaces SensorType closed enum

use std::borrow::Borrow;
use std::sync::Arc;

/// Open newtype identifying a sensor type by string ID.
///
/// Replaces the closed `SensorType` enum per ADR-023 §C1 + BC-2.01.013.
///
/// # Design
/// - Inner payload is `Arc<str>` for cheap `Clone` (reference-counted, no heap copy).
/// - All equality and hashing is content-based, not pointer-based.
/// - `Borrow<str>` enables `HashMap<SensorId, V>::get("crowdstrike")` without cloning.
/// - `Display` delegates to the inner string; no round-trip loss.
///
/// # Construction
/// ```rust,ignore
/// let id = SensorId::from("crowdstrike");
/// let id = SensorId::from(String::from("armis"));
/// let id = SensorId::from(Arc::from("claroty"));
/// ```
// The inner Arc<str> field appears "never read" to the compiler because all
// trait impls are todo!() stubs. Allow dead_code until implementation fills them.
#[allow(dead_code)]
#[derive(Clone)]
pub struct SensorId(Arc<str>);

impl From<&str> for SensorId {
    fn from(_s: &str) -> Self {
        todo!("S-PLUGIN-PREREQ-A: implement SensorId::from(&str) — Arc::from(s)")
    }
}

impl From<String> for SensorId {
    fn from(_s: String) -> Self {
        todo!("S-PLUGIN-PREREQ-A: implement SensorId::from(String) — Arc::from(s.as_str())")
    }
}

impl From<Arc<str>> for SensorId {
    fn from(_s: Arc<str>) -> Self {
        todo!("S-PLUGIN-PREREQ-A: implement SensorId::from(Arc<str>) — SensorId(s)")
    }
}

impl std::fmt::Display for SensorId {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("S-PLUGIN-PREREQ-A: implement Display — delegate to inner Arc<str>")
    }
}

impl std::fmt::Debug for SensorId {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("S-PLUGIN-PREREQ-A: implement Debug — write SensorId(inner)")
    }
}

impl PartialEq for SensorId {
    fn eq(&self, _other: &Self) -> bool {
        todo!("S-PLUGIN-PREREQ-A: implement PartialEq — self.0 == other.0 (content-based)")
    }
}

impl Eq for SensorId {}

impl std::hash::Hash for SensorId {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        todo!("S-PLUGIN-PREREQ-A: implement Hash — hash &*self.0 (content-based, consistent with PartialEq)")
    }
}

impl Ord for SensorId {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        todo!("S-PLUGIN-PREREQ-A: implement Ord — (*self.0).cmp(&*other.0)")
    }
}

impl PartialOrd for SensorId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Canonical form required by clippy::non_canonical_partial_ord_impl.
        // Delegates to Ord::cmp, which also panics at todo!() until implemented.
        Some(self.cmp(other))
    }
}

impl Borrow<str> for SensorId {
    fn borrow(&self) -> &str {
        todo!("S-PLUGIN-PREREQ-A: implement Borrow<str> — &*self.0")
    }
}

impl AsRef<str> for SensorId {
    fn as_ref(&self) -> &str {
        todo!("S-PLUGIN-PREREQ-A: implement AsRef<str> — &*self.0")
    }
}

// ---------------------------------------------------------------------------
// Unit tests — Red Gate set
// All three tests MUST FAIL (panic at todo!()) before implementation.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    /// BC-2.01.013 postcondition: sensor identity is a runtime string value.
    /// Red Gate: panics at todo!() in From<&str> or Display.
    ///
    /// Exercises AC-1 (From<&str>, Display), AC-9(a) (equality, hash, display roundtrip).
    #[test]
    fn test_BC_2_01_013_001_sensorid_from_str_roundtrip() {
        let id = SensorId::from("crowdstrike");
        let displayed = format!("{id}");
        assert_eq!(
            displayed, "crowdstrike",
            "Display must reproduce the original string"
        );

        // Also verify HashSet containment (exercises PartialEq + Hash).
        let mut set = HashSet::new();
        set.insert(SensorId::from("crowdstrike"));
        assert!(
            set.contains(&SensorId::from("crowdstrike")),
            "SensorId equality and hash must be content-based"
        );
    }

    /// BC-2.01.013 postcondition: From<String> produces same identity as From<&str>.
    /// Red Gate: panics at todo!() in From<String>.
    ///
    /// Exercises AC-1 (From<String>), AC-9(a).
    #[test]
    fn test_BC_2_01_013_002_sensorid_from_string_roundtrip() {
        let id = SensorId::from(String::from("cyberint"));
        let displayed = format!("{id}");
        assert_eq!(
            displayed, "cyberint",
            "From<String> must preserve the string value through Display"
        );

        // Equality with a separately-constructed SensorId from the same &str.
        let id2 = SensorId::from("cyberint");
        assert_eq!(
            id, id2,
            "SensorId from String and from &str must be equal when content is identical"
        );
    }

    /// BC-2.01.013 invariant: sensor identity equality and hash are content-based.
    /// Red Gate: panics at todo!() in Hash or PartialEq.
    ///
    /// Exercises AC-1 (Hash, PartialEq, Eq), AC-9(a) (hash + equality invariant),
    /// EC-001 (two SensorIds from identical strings must be equal).
    #[test]
    fn test_BC_2_01_013_003_sensorid_hash_eq_invariant() {
        let a = SensorId::from("crowdstrike");
        let b = SensorId::from("crowdstrike");

        // PartialEq: same content → equal.
        assert_eq!(a, b, "two SensorIds from the same string must be equal");

        // HashMap round-trip: exercises both Hash and PartialEq consistency.
        let mut map: HashMap<SensorId, u32> = HashMap::new();
        map.insert(a, 42);
        let retrieved = map.get(&b).copied();
        assert_eq!(
            retrieved,
            Some(42),
            "HashMap lookup via separately-constructed SensorId must find the inserted value"
        );
    }
}
