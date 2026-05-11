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

use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
/// Use `SensorId::from("crowdstrike")` (panics on invalid input — only for trusted
/// compile-time literals) or `SensorId::try_from_str(s)?` (returns
/// `Err(SensorIdValidationError)`) for fallible construction from untrusted input.
/// Valid strings: 1..=64 chars, `[a-z0-9_-]`, no leading/trailing `-` or `_`.
///
/// Note: `TryFrom<&str>` and `TryFrom<String>` are not implemented because Rust's
/// blanket `impl<T, U: Into<T>> TryFrom<U> for T` already provides those impls
/// (delegating to the infallible `From`, which panics on invalid input). Use
/// `try_from_str` as the canonical fallible constructor.
///
/// ```rust,ignore
/// let id = SensorId::from("crowdstrike");            // panics on invalid — trusted literals only
/// let id = SensorId::from(String::from("armis"));   // same semantics
/// let id = SensorId::from(Arc::from("claroty"));    // same semantics
/// let id = SensorId::try_from_str("my-sensor")?;   // fallible — for untrusted input
/// ```
#[derive(Clone)]
pub struct SensorId(Arc<str>);

impl SensorId {
    /// Construct a `SensorId` from any value that can be converted to `Arc<str>`.
    pub fn new(s: impl Into<Arc<str>>) -> Self {
        SensorId(s.into())
    }
}

impl From<&str> for SensorId {
    /// Construct a `SensorId` from a string slice.
    ///
    /// # Panics
    /// Panics if `s` fails `validate_sensor_id_string`. **Only use this for
    /// trusted compile-time literals** (e.g. `SensorId::from("crowdstrike")`).
    /// For untrusted or user-supplied input use `SensorId::try_from_str(s)` instead.
    ///
    /// Valid inputs: 1..=64 lowercase alphanumeric + `_` + `-`; no leading/trailing `_` or `-`.
    fn from(s: &str) -> Self {
        if let Err(e) = validate_sensor_id_string(s) {
            panic!("S-PLUGIN-PREREQ-A: invalid SensorId string '{s}': {e}");
        }
        SensorId(Arc::from(s))
    }
}

impl From<String> for SensorId {
    /// Construct a `SensorId` from an owned `String`.
    ///
    /// # Panics
    /// Panics if `s` fails `validate_sensor_id_string`. **Only use this for
    /// trusted compile-time literals.** For untrusted or user-supplied input
    /// use `SensorId::try_from_str(&s)` instead.
    fn from(s: String) -> Self {
        if let Err(e) = validate_sensor_id_string(&s) {
            panic!("S-PLUGIN-PREREQ-A: invalid SensorId string '{s}': {e}");
        }
        SensorId(Arc::from(s.as_str()))
    }
}

impl From<Arc<str>> for SensorId {
    /// Construct a `SensorId` from an `Arc<str>`.
    ///
    /// # Panics
    /// Panics if the string fails `validate_sensor_id_string`. **Only use this for
    /// trusted compile-time or internally-validated strings.** For untrusted input
    /// use `SensorId::try_from_str(&s)` instead.
    fn from(s: Arc<str>) -> Self {
        if let Err(e) = validate_sensor_id_string(&s) {
            panic!("S-PLUGIN-PREREQ-A: invalid SensorId string '{s}': {e}");
        }
        SensorId(s)
    }
}

impl std::fmt::Display for SensorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Debug for SensorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SensorId({:?})", &*self.0)
    }
}

impl PartialEq for SensorId {
    fn eq(&self, other: &Self) -> bool {
        // Content-based: same string value → equal, regardless of Arc pointer.
        *self.0 == *other.0
    }
}

impl Eq for SensorId {}

impl std::hash::Hash for SensorId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the string bytes — consistent with PartialEq (content-based).
        (*self.0).hash(state);
    }
}

impl Ord for SensorId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self.0).cmp(&*other.0)
    }
}

impl PartialOrd for SensorId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Canonical form: delegate to Ord::cmp (clippy::non_canonical_partial_ord_impl).
        Some(self.cmp(other))
    }
}

impl Borrow<str> for SensorId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for SensorId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Serialize for SensorId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SensorId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        SensorId::try_from_str(&raw).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// SensorId validation
// ---------------------------------------------------------------------------

/// Validation errors for `SensorId` string values.
///
/// Returned by `SensorId::try_from_str()` and `validate_sensor_id_string()`.
/// `From<&str>`, `From<String>`, and `From<Arc<str>>` panic on invalid input —
/// use `try_from_str` for fallible construction from untrusted input.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SensorIdValidationError {
    /// String is empty (length 0).
    TooShort,
    /// String exceeds 64 characters.
    TooLong { len: usize },
    /// String contains characters outside `[a-z0-9_-]`.
    InvalidChars { offending: String },
    /// String begins or ends with `-` or `_`.
    InvalidBoundary,
    /// First character is not a lowercase ASCII letter `[a-z]`.
    ///
    /// Matches the canonical rule in `prism_spec_engine::validation::validate_sensor_id`
    /// (regex `^[a-z][a-z0-9_-]*$`). Digit-first identifiers (e.g. `"1sensor"`) are
    /// rejected here so that no TOML spec can register a key that the newtype refuses.
    InvalidFirstChar { actual: char },
}

impl std::fmt::Display for SensorIdValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort => write!(f, "sensor id must not be empty"),
            Self::TooLong { len } => write!(f, "sensor id is {len} characters; maximum is 64"),
            Self::InvalidChars { offending } => write!(
                f,
                "sensor id contains invalid characters '{offending}'; \
                 only [a-z0-9_-] are allowed"
            ),
            Self::InvalidBoundary => write!(f, "sensor id must not start or end with '-' or '_'"),
            Self::InvalidFirstChar { actual } => write!(
                f,
                "sensor id must start with a lowercase letter [a-z]; got '{actual}'"
            ),
        }
    }
}

impl std::error::Error for SensorIdValidationError {}

/// Validate a sensor id candidate string.
///
/// This function is `pub` to provide a standalone validation primitive that can
/// be used without constructing a `SensorId` — e.g., for testing input strings
/// before deserialization, or for external integration tests that need to verify a
/// string would pass the SensorId newtype invariant. Currently no crate-external
/// callers; the canonical fallible constructor is `SensorId::try_from_str` which
/// validates internally. Future callers landing here: this function returns
/// `Result<(), SensorIdValidationError>` so the caller can decide what to do on
/// validation failure without paying the cost of newtype construction.
///
/// **Parity contract:** Validation rules match
/// `prism_spec_engine::validation::validate_sensor_id` (regex `^[a-z][a-z0-9_-]*$`);
/// the invariant is locked by the cross-crate proptest in
/// `crates/prism-core/tests/sensor_id_validator_parity.rs`.
///
/// # Example
/// ```rust,ignore
/// // Pre-validate before construction:
/// validate_sensor_id_string(user_input)?;
/// // Or construct directly (preferred when you need the SensorId):
/// let id = SensorId::try_from_str(user_input)?;
/// ```
///
/// Rules (checked in this order):
/// 1. Charset: `[a-z0-9_-]` (lowercase alphanumeric, underscore, hyphen).
///    Multi-byte / non-ASCII characters are rejected here, ensuring that the
///    subsequent byte-length check is equivalent to a character-count check.
/// 2. Length: 1..=64 characters (= bytes after charset validation).
/// 3. First character: must be a lowercase ASCII letter `[a-z]` (not a digit or
///    boundary character). Matches `prism_spec_engine::validation::validate_sensor_id`.
/// 4. No leading or trailing `-` or `_`.
///
/// Returns `Ok(())` for valid inputs; `Err(SensorIdValidationError)` otherwise.
pub fn validate_sensor_id_string(s: &str) -> Result<(), SensorIdValidationError> {
    // Step 1: charset check FIRST — rejects non-ASCII and disallowed characters.
    // This must precede the length check so that `s.len()` is always measured in
    // bytes == characters (all valid chars are single-byte ASCII).
    let invalid: String = s
        .chars()
        .filter(|c| !matches!(c, 'a'..='z' | '0'..='9' | '_' | '-'))
        .collect();
    if !invalid.is_empty() {
        return Err(SensorIdValidationError::InvalidChars { offending: invalid });
    }

    // Step 2: length check — after charset validation all chars are ASCII, so
    // `s.len()` (bytes) equals `s.chars().count()` (characters).
    if s.is_empty() {
        return Err(SensorIdValidationError::TooShort);
    }
    if s.len() > 64 {
        return Err(SensorIdValidationError::TooLong { len: s.len() });
    }

    // Step 3: first-char check — must be a lowercase ASCII letter [a-z].
    // Matches prism_spec_engine::validation::validate_sensor_id (regex ^[a-z][a-z0-9_-]*$).
    // Digit-first ids like "1sensor" pass the charset check but are rejected here.
    let first = s.chars().next().expect("non-empty checked above");
    if !first.is_ascii_lowercase() {
        return Err(SensorIdValidationError::InvalidFirstChar { actual: first });
    }

    // Step 4: trailing boundary check — first char already checked above.
    let last = s.chars().next_back().expect("non-empty checked above");
    if last == '-' || last == '_' {
        return Err(SensorIdValidationError::InvalidBoundary);
    }
    Ok(())
}

impl SensorId {
    /// Fallible construction from a string slice — use when input is untrusted.
    ///
    /// Returns `Err(SensorIdValidationError)` if the string fails validation.
    /// Use this for deserialized or user-supplied input. Do NOT use
    /// `SensorId::try_from(s)?` — Rust's blanket `TryFrom<U> for T where U: Into<T>`
    /// makes that call infallible (`Error = Infallible`) and delegates to the PANICKING
    /// `From::from` impl. Always use `try_from_str` for fallible construction.
    ///
    /// # Example
    /// ```rust,ignore
    /// let id = SensorId::try_from_str("my-plugin")?;
    /// ```
    pub fn try_from_str(s: &str) -> Result<Self, SensorIdValidationError> {
        validate_sensor_id_string(s)?;
        Ok(SensorId(Arc::from(s)))
    }
}

// ---------------------------------------------------------------------------
// Unit tests — Green (all passing post-implementation)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    /// BC-2.01.013 postcondition: sensor identity is a runtime string value.
    /// Verifies BC-2.01.013 postcondition: From<&str> and Display round-trip correctly.
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
    /// Verifies BC-2.01.013 postcondition: From<String> and From<&str> produce equal SensorIds.
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
    /// Verifies BC-2.01.013 postcondition: hash and equality are content-based, not pointer-based.
    ///
    /// Exercises AC-1 (Hash, PartialEq, Eq), AC-9(a) (hash + equality invariant),
    /// EC-001 (two SensorIds from identical strings must be equal).
    #[test]
    #[allow(non_snake_case)]
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

    // ---------------------------------------------------------------------------
    // Validation tests (S-PLUGIN-PREREQ-A / F-LP1-HIGH-005)
    // ---------------------------------------------------------------------------

    /// S-PLUGIN-PREREQ-A: validation rejects uppercase characters.
    #[test]
    fn test_sensorid_validation_rejects_uppercase() {
        let result = SensorId::try_from_str("CrowdStrike");
        assert!(
            matches!(result, Err(SensorIdValidationError::InvalidChars { .. })),
            "uppercase characters must be rejected by SensorId validation"
        );
    }

    /// S-PLUGIN-PREREQ-A: validation rejects control characters.
    #[test]
    fn test_sensorid_validation_rejects_control_chars() {
        let result = SensorId::try_from_str("crowdstrike\x00");
        assert!(
            matches!(result, Err(SensorIdValidationError::InvalidChars { .. })),
            "control characters must be rejected by SensorId validation"
        );
    }

    /// S-PLUGIN-PREREQ-A: validation rejects empty string and strings exceeding 64 chars.
    #[test]
    fn test_sensorid_validation_rejects_empty_and_too_long() {
        let empty = SensorId::try_from_str("");
        assert_eq!(
            empty,
            Err(SensorIdValidationError::TooShort),
            "empty string must be rejected"
        );

        let too_long = SensorId::try_from_str("a".repeat(65).as_str());
        assert!(
            matches!(too_long, Err(SensorIdValidationError::TooLong { .. })),
            "65-character string must be rejected (max 64)"
        );

        // Exactly 64 chars must be accepted.
        let ok_64 = SensorId::try_from_str("a".repeat(64).as_str());
        assert!(ok_64.is_ok(), "64-character string must be accepted");
    }

    /// S-PLUGIN-PREREQ-A / F-LP2-MED-002: validation rejects strings with leading or
    /// trailing `-` or `_` boundary characters.
    ///
    /// Leading `-`/`_` are caught by Step 3 (first-char check, `InvalidFirstChar`)
    /// because neither `-` nor `_` is an ascii_lowercase letter.
    /// Trailing `-`/`_` are caught by Step 4 (boundary check, `InvalidBoundary`).
    #[test]
    fn test_sensorid_validation_rejects_boundary_chars() {
        // Leading `-` or `_` → InvalidFirstChar (Step 3 fires before Step 4).
        for case in ["-foo", "_foo"] {
            let result = SensorId::try_from_str(case);
            assert!(
                matches!(result, Err(SensorIdValidationError::InvalidFirstChar { .. })),
                "'{case}' must be rejected with InvalidFirstChar (leading boundary char is not [a-z])"
            );
        }
        // Trailing `-` or `_` → InvalidBoundary (Step 4; first char passes Step 3).
        for case in ["foo-", "foo_"] {
            let result = SensorId::try_from_str(case);
            assert_eq!(
                result,
                Err(SensorIdValidationError::InvalidBoundary),
                "'{case}' must be rejected with InvalidBoundary"
            );
        }
    }

    /// F-LP4-MED-001: digit-first sensor ids are rejected (parity with spec-engine rule).
    ///
    /// SensorId::try_from_str("1sensor") must return Err(InvalidFirstChar) because
    /// `^[a-z][a-z0-9_-]*$` requires the first character to be a lowercase letter.
    #[test]
    fn test_sensorid_validation_rejects_digit_first() {
        let cases = ["1sensor", "9digits", "0abc"];
        for case in cases {
            let result = SensorId::try_from_str(case);
            assert!(
                matches!(
                    result,
                    Err(SensorIdValidationError::InvalidFirstChar { .. })
                ),
                "'{case}' must be rejected with InvalidFirstChar (digit-first violates ^[a-z])"
            );
        }
    }

    /// S-PLUGIN-PREREQ-A: Deserialize rejects invalid sensor id strings.
    #[test]
    fn test_sensorid_deserialize_rejects_invalid() {
        let json_uppercase = r#""CrowdStrike""#;
        let result: Result<SensorId, _> = serde_json::from_str(json_uppercase);
        assert!(
            result.is_err(),
            "deserializing an uppercase sensor id must fail"
        );

        let json_empty = r#""""#;
        let result: Result<SensorId, _> = serde_json::from_str(json_empty);
        assert!(
            result.is_err(),
            "deserializing an empty sensor id must fail"
        );

        // Valid values round-trip correctly.
        let json_valid = r#""crowdstrike""#;
        let result: Result<SensorId, _> = serde_json::from_str(json_valid);
        assert!(result.is_ok(), "valid lowercase sensor id must deserialize");
    }
}
