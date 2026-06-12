//! `ClarotyState` — shared mutable state for the Claroty DTU server.
//!
//! Maintains a stateful device tag store (`(OrgId, device_id) → {tag_keys}`),
//! request counter for rate-limit enforcement, and runtime failure mode.
//! All mutation is Mutex-guarded; reset restores base fixture state.
//!
//! # S-3.2.01 — Multi-tenant state segregation
//!
//! Per ADR-008 §2.1 Step 6a, `tag_store` is keyed by `(OrgId, device_id)` rather
//! than bare `device_id`, eliminating cross-org tag bleed (BC-3.2.001).

use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        Mutex,
    },
};

use prism_core::OrgId;
use prism_dtu_common::FailureMode;

/// Sentinel `OrgId` used in unit tests that operate on a single organisation.
///
/// Gated `#[cfg(test)]` — any production reference is a compile error (AC-006,
/// BC-3.2.001 invariant 3).
#[cfg(test)]
pub const DEFAULT_ORG_ID: OrgId = OrgId(uuid::uuid!("00000000-0000-7000-8000-000000000001"));

/// Shared mutable state for the Claroty xDome DTU behavioral clone.
///
/// # Stateful behavior
/// - `tag_store`: Device tag write paths (AC-3, AC-4). Maps `(OrgId, device_id)` to the
///   set of tag keys assigned to that device. Cleared on `reset_all()`.
///   Composite key enforces per-org isolation (BC-3.2.001).
/// - `request_counter`: Incremented per API request; used by FailureLayer for
///   rate-limit and internal-error injection (AC-6, AC-7).
/// - `failure_mode`: Current failure injection mode; updated via `/dtu/configure`.
/// - `latency_ms`: Artificial latency in milliseconds (EC-006); updated via `/dtu/configure`.
/// - `instance_org_id`: The authoritative `OrgId` for this clone instance, assigned at
///   startup by the harness. Route handlers validate `X-Org-Id` against this value
///   (W3-FIX-SEC-001 / AC-001..AC-003).
pub struct ClarotyState {
    /// Maps `(OrgId, device_uid)` → set of tag keys. Stateful across requests until reset.
    ///
    /// The composite key is the EXCLUSIVE keying scheme post ADR-008 §2.1 Step 6a.
    /// No bare-String mutable store may exist (BC-3.2.001).
    pub tag_store: Mutex<HashMap<(OrgId, String), HashSet<String>>>,
    /// Monotonically increasing request counter (1-indexed).
    pub request_counter: AtomicU32,
    /// Current failure injection mode for this clone.
    pub failure_mode: Mutex<FailureMode>,
    /// Artificial latency in milliseconds added to every API response (EC-006).
    pub latency_ms: AtomicU64,
    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    pub admin_token: String,
    /// Authoritative `OrgId` for this clone instance (W3-FIX-SEC-001).
    ///
    /// Set at startup; route handlers compare the `X-Org-Id` header against this value
    /// and return HTTP 401 on mismatch (W3-FIX-SEC-001).
    pub instance_org_id: OrgId,

    // -----------------------------------------------------------------------
    // Story A: generated fixture field (BC-2.06.018 / ADR-036 §2.3)
    // -----------------------------------------------------------------------
    /// Generated records from the fixture generator (BC-2.06.018 postcondition 1).
    ///
    /// Non-empty when the clone is constructed via `new_with_seed` AND the archetype
    /// produces records (e.g., CompromisedEndpoint). EMPTY for DormantTenant archetype
    /// even when seeded.
    ///
    /// Route handlers MUST use `fixture_gen_seeded` (not `generated_records.is_empty()`)
    /// to decide between the generated path and the static-fixture fallback.
    /// Otherwise DormantTenant (seeded=true, generated_records=[]) would silently fall
    /// back to the static fixture — violating F-P6-HIGH-001 / BC EC-018-003.
    ///
    /// ADR-036 §2.3: immutable after construction (NOT Arc<Mutex<...>>).
    #[cfg(feature = "fixture-gen")]
    pub generated_records: Vec<serde_json::Value>,

    /// True when the clone was constructed via `new_with_seed` (fixture-gen path).
    ///
    /// Used by route handlers as the dual-path sentinel INSTEAD of
    /// `generated_records.is_empty()`. This correctly handles DormantTenant, which
    /// produces 0 records — route handlers MUST serve empty, not fall back to static fixture.
    ///
    /// ADR-036 v2.2 / F-P6-HIGH-001 fix: always false on `new()` path.
    #[cfg(feature = "fixture-gen")]
    pub fixture_gen_seeded: bool,

    // -----------------------------------------------------------------------
    // Story B: scenario timeline (BC-2.06.019 / ADR-036 v2.3 §2.3)
    // -----------------------------------------------------------------------
    /// Scenario incident timeline. `Some` when constructed via `new_with_scenario`.
    ///
    /// Must be `#[cfg(feature = "fixture-gen")]`-gated because `chrono` is only
    /// available under `fixture-gen` in this crate (dep:chrono gating).
    /// ADR-036 v2.3 §2.3: threaded as `Arc<IncidentTimeline>` (NOT `Arc<Mutex<...>>`).
    #[cfg(feature = "fixture-gen")]
    pub timeline: Option<std::sync::Arc<prism_dtu_common::IncidentTimeline>>,
}

impl ClarotyState {
    /// Create state with a specific admin token and the nil-UUID instance OrgId.
    ///
    /// W3-FIX-SEC-001: `instance_org_id` defaults to the nil UUID so that
    /// clones created with `new()` skip org-header validation (backward compat
    /// for test callers that do not supply `X-Org-Id`). Callers that need strict
    /// per-org header validation must use `with_admin_token_and_org` with a
    /// real, non-nil `OrgId`.
    pub fn with_admin_token(admin_token: String) -> Self {
        Self::with_admin_token_and_org(admin_token, OrgId::from_uuid(uuid::Uuid::nil()))
    }

    /// Create state with a specific admin token and explicit `instance_org_id`.
    ///
    /// Used by test helpers that need deterministic org identity for multi-tenant
    /// cross-org header validation tests (W3-FIX-SEC-001 AC-001..AC-003).
    pub fn with_admin_token_and_org(admin_token: String, instance_org_id: OrgId) -> Self {
        Self {
            tag_store: Mutex::new(HashMap::new()),
            request_counter: AtomicU32::new(0),
            failure_mode: Mutex::new(FailureMode::None),
            latency_ms: AtomicU64::new(0),
            admin_token,
            instance_org_id,
            // Story A: generated_records defaults empty (static-JSON path).
            #[cfg(feature = "fixture-gen")]
            generated_records: Vec::new(),
            // fixture_gen_seeded is false on new() path — route handlers use static fixture.
            #[cfg(feature = "fixture-gen")]
            fixture_gen_seeded: false,
            // Story B: timeline is None on new() and new_with_seed() paths.
            #[cfg(feature = "fixture-gen")]
            timeline: None,
        }
    }

    /// Reset all tag state and counters to initial values (base fixture state).
    ///
    /// After `reset_all()`, device queries return devices with empty `tags` arrays
    /// (AC-8). Request counter is zeroed; failure mode cleared to `None`.
    ///
    /// This is the canonical implementation. `reset()` is a one-wave compatibility
    /// shim that delegates here. `BehavioralClone::reset()` calls this via `reset()`.
    pub fn reset_all(&self) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        self.tag_store.lock().expect("tag_store poisoned").clear();
        self.request_counter.store(0, Ordering::SeqCst);
        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        {
            *self.failure_mode.lock().expect("failure_mode poisoned") = FailureMode::None;
        }
        self.latency_ms.store(0, Ordering::SeqCst);
    }

    /// Compatibility shim — delegates to `reset_all()`.
    ///
    /// One-wave shim per ADR-008 §2.4. Must be removed before Wave 3 closes.
    /// `BehavioralClone::reset()` calls this method.
    pub fn reset(&self) {
        self.reset_all();
    }

    /// Selectively evict all entries belonging to `org_id`.
    ///
    /// After `reset_for(org_id_A)`:
    /// - All `(org_id_A, *)` entries are removed.
    /// - All entries for other orgs are untouched (EC-003, BC-3.2.001 invariant 1).
    pub fn reset_for(&self, org_id: OrgId) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut store = self.tag_store.lock().expect("tag_store poisoned");
        store.retain(|(id, _), _| *id != org_id);
    }

    /// Apply a new failure mode at runtime (called by `/dtu/configure` handler).
    pub fn apply_config(&self, mode: FailureMode) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        {
            *self.failure_mode.lock().expect("failure_mode poisoned") = mode;
        }
    }

    /// Set the artificial latency in milliseconds (called by `/dtu/configure` handler).
    pub fn apply_latency(&self, ms: u64) {
        self.latency_ms.store(ms, Ordering::SeqCst);
    }

    /// Increment the request counter and return its new 1-indexed value.
    pub fn increment_counter(&self) -> u32 {
        self.request_counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Add a tag key to the device's tag set, scoped to `org_id`.
    ///
    /// Returns `true` if the tag was newly inserted, `false` if already present.
    /// The key `(org_id, device_id)` ensures tags written under `org_id_A` are
    /// invisible to `org_id_B` lookups (BC-3.2.001 postcondition 2).
    pub fn add_tag(&self, org_id: OrgId, device_id: &str, tag_key: &str) -> bool {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut store = self.tag_store.lock().expect("tag_store poisoned");
        store
            .entry((org_id, device_id.to_string()))
            .or_default()
            .insert(tag_key.to_string())
    }

    /// Remove a tag key from the device's tag set, scoped to `org_id`.
    ///
    /// Returns `true` if the tag existed and was removed, `false` if not found (EC-002).
    pub fn remove_tag(&self, org_id: OrgId, device_id: &str, tag_key: &str) -> bool {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut store = self.tag_store.lock().expect("tag_store poisoned");
        if let Some(tags) = store.get_mut(&(org_id, device_id.to_string())) {
            tags.remove(tag_key)
        } else {
            false
        }
    }

    /// Return the set of tag keys for a given `(org_id, device_id)`. Empty set if unknown.
    ///
    /// A lookup under `org_id_B` never sees entries written under `org_id_A`
    /// (BC-3.2.001 postcondition 1).
    pub fn get_tags(&self, org_id: OrgId, device_id: &str) -> HashSet<String> {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let store = self.tag_store.lock().expect("tag_store poisoned");
        store
            .get(&(org_id, device_id.to_string()))
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for ClarotyState {
    fn default() -> Self {
        Self::with_admin_token(uuid::Uuid::new_v4().to_string())
    }
}

// ---------------------------------------------------------------------------
// In-crate unit tests — BC-3.2.001 / AC-006
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// BC-3.2.001 invariant 3 / AC-006 — `DEFAULT_ORG_ID` is accessible within
    /// the crate's `#[cfg(test)]` scope and is distinct from the zero UUID.
    ///
    /// The compile-time enforcement (production code cannot reference this const)
    /// is structural: removing the `#[cfg(test)]` gate on the const would expose
    /// it in production and is a compile error for downstream crates that import
    /// `prism-dtu-claroty` without the `dtu` feature.
    ///
    /// Traces to: BC-3.2.001 invariant 3, AC-006.
    #[test]
    fn test_bc_3_2_001_default_org_id_is_test_gated_and_non_zero() {
        // Accessible here because we are in #[cfg(test)].
        let zero = OrgId::from_uuid(uuid::Uuid::nil());
        assert_ne!(
            DEFAULT_ORG_ID, zero,
            "DEFAULT_ORG_ID must be a non-zero sentinel"
        );
        // Verify the UUID variant/version bytes match the declared constant.
        assert_eq!(
            DEFAULT_ORG_ID.as_uuid().to_string(),
            "00000000-0000-7000-8000-000000000001",
            "DEFAULT_ORG_ID must match the declared sentinel value"
        );
    }
}
