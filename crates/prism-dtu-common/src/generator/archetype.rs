//! Archetype catalog — 8 named deployment scenarios (BC-3.4.003).
//!
//! All archetypes are `#[non_exhaustive]` so future waves can add variants
//! without breaking existing match arms.

/// The eight deployment-scenario archetypes for multi-tenant fixture generation.
///
/// Declared `#[non_exhaustive]` per BC-3.4.003 invariant 2.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Archetype {
    /// Stable OT network; no active threats (BC-3.4.003).
    HealthyOtEnvironment,
    /// Elevated alerts; containment state changes (BC-3.4.003).
    CompromisedEndpoint,
    /// First API call returns HTTP 401; recovers after configurable delay (BC-3.4.003).
    AuthOutage,
    /// 10 000 devices; exercises pagination and memory budget (BC-3.4.003).
    LargeScale,
    /// Exact multiples of page size; single-page and empty-final-page variants (BC-3.4.003).
    PaginationEdgeCases,
    /// One record has a non-conformant field shape (BC-3.4.003).
    SchemaDrift,
    /// Devices appear/disappear between polling cycles; tombstone records present (BC-3.4.003).
    HighChurn,
    /// No data; simulates recently onboarded or offline tenant (BC-3.4.003).
    DormantTenant,
}

/// Returns all 8 archetypes for exhaustive iteration in parameterised tests.
///
/// BC-3.4.003 invariant 2: `all_archetypes()` provides the authoritative list.
pub fn all_archetypes() -> &'static [Archetype] {
    &[
        Archetype::HealthyOtEnvironment,
        Archetype::CompromisedEndpoint,
        Archetype::AuthOutage,
        Archetype::LargeScale,
        Archetype::PaginationEdgeCases,
        Archetype::SchemaDrift,
        Archetype::HighChurn,
        Archetype::DormantTenant,
    ]
}
