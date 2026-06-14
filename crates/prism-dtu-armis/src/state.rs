//! `ArmisState` — in-memory state for the Armis Centrix DTU behavioral clone.
//!
//! Maintains:
//! - Immutable device fixture registry pre-loaded from `fixtures/devices.json`
//! - Immutable activity fixture pre-loaded from `fixtures/device-activity.json`
//! - Immutable alert fixture pre-loaded from `fixtures/alerts.json`
//! - Stateful tag store: `(OrgId, device_id) → {tag_keys}` — mutated by tag write endpoints
//!   (BC-3.2.001: composite key enforces per-org state isolation; ADR-008 §2.1 Step 6b)
//! - AQL capture log: ordered list of all AQL strings received since last reset
//!
//! No HTTP-layer types (`axum::Json`, `axum::extract::*`) appear here.
//! `ArmisState` is pure Rust — no Axum dependency for its public methods.

use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use prism_core::OrgId;
use prism_dtu_common::FailureMode;

use crate::types::{ActivityRecord, AlertRecord, DeviceRecord};

/// Default `OrgId` for test fixtures — `#[cfg(test)]` only.
///
/// Production code paths MUST NOT reference this constant.
/// BC-3.2.001 invariant 3: `DEFAULT_ORG_ID` is compile-error in non-test.
#[cfg(test)]
pub const DEFAULT_ORG_ID: OrgId = OrgId(uuid::uuid!("00000000-0000-7000-8000-000000000001"));

/// Single-tenant DTU route `OrgId` — used by HTTP route handlers when no per-request
/// org context is available (DTU clone runs as a single-org HTTP server per test instance).
///
/// Not feature-gated: route handlers (`routes/tags.rs`, `routes/devices.rs`) import this
/// constant unconditionally and must compile with `--no-default-features`.
/// MUST NOT be used in any production (non-DTU) code path.
pub const DTU_ROUTE_ORG_ID: OrgId = OrgId(uuid::uuid!("00000000-0000-7000-8000-000000000001"));

/// Default `instance_org_id` for `ArmisClone::new()` — matches the `dummy_org` used in
/// `x_org_id_auth::test_AC_001_x_org_id_validated_against_bearer_token`.
///
/// W3-FIX-SEC-001: `ArmisClone::new()` uses this deterministic OrgId so that the AC-001
/// test (which sends this UUID as `X-Org-Id`) passes the validate_org_id check. Callers
/// that need a different `instance_org_id` must use `ArmisState::with_admin_token_and_org`.
///
/// Not feature-gated: imported by state and clone modules unconditionally.
/// MUST NOT be used in any production (non-DTU) code path.
pub const DTU_DEFAULT_INSTANCE_ORG_ID: OrgId =
    OrgId(uuid::uuid!("00000000-0000-7000-8000-0000000000AA"));

/// Validated configuration payload for `POST /dtu/configure` (TD-WV0-04).
///
/// Unknown fields are rejected by serde to prevent silent misconfiguration.
///
/// # Wire formats accepted
///
/// The harness generic `failure_mode_to_json()` sends `{"auth_mode": "reject"}` (clone_server
/// format). Armis also accepts `{"failure_mode": "auth_reject"}` (its own natural schema).
/// Both forms are listed here so `deny_unknown_fields` admits them without relaxing the check.
#[derive(Debug, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ConfigPayload {
    /// Failure mode (Armis-native schema): `"none"`, `"rate_limit"`, `"malformed_response"`,
    /// `"auth_reject"`, `"internal_error"`, `"network_timeout"`.
    #[serde(default)]
    failure_mode: Option<String>,
    /// Failure mode (harness generic schema, clone_server compat): `"reject"` → AuthReject,
    /// `"none"` → None. Sent by `HarnessBuilder::inject_failure` for all clone types.
    #[serde(default)]
    auth_mode: Option<String>,
    /// Companion for `"rate_limit"`: number of requests before triggering 429.
    #[serde(default)]
    after_n_requests: Option<u32>,
    /// Companion for `"rate_limit"`: seconds in `Retry-After` header.
    #[serde(default)]
    retry_after_secs: Option<u32>,
    /// Companion for `"internal_error"`: inject 500 at this 1-indexed request number.
    #[serde(default)]
    at_request_n: Option<u32>,
    /// Companion for `"network_timeout"`: milliseconds to delay.
    #[serde(default)]
    after_ms: Option<u64>,
    /// Harness generic: `{"clear": true}` → FailureMode::None.
    #[serde(default)]
    clear: Option<bool>,
    /// Harness generic: `{"rate_limit_after": N}` companion for RateLimit.
    #[serde(default)]
    rate_limit_after: Option<u32>,
    /// Harness generic: `{"internal_error_at": N}` companion for InternalError.
    #[serde(default)]
    internal_error_at: Option<u32>,
    /// Harness generic: `{"network_timeout_ms": ms}` companion for NetworkTimeout.
    #[serde(default)]
    network_timeout_ms: Option<u64>,
    /// Harness generic: `{"unprocessable_at": N}` companion for Unprocessable.
    #[serde(default)]
    unprocessable_at: Option<u32>,
    /// Harness generic: `{"malformed_response": true}` for MalformedResponse.
    #[serde(default)]
    malformed_response: Option<bool>,
}

/// Shared mutable state for the Armis Centrix DTU clone.
///
/// `Arc<ArmisState>` is passed to every axum route handler via `axum::extract::State`.
pub struct ArmisState {
    // --- Immutable fixture registries (loaded once at construction) ---
    /// Device fixture registry, keyed by `device_id`.
    /// Loaded from `fixtures/devices.json`.
    pub device_registry: HashMap<String, DeviceRecord>,

    /// All device records in insertion order (for pagination).
    pub devices_ordered: Vec<DeviceRecord>,

    /// Activity fixture list (for all device_ids).
    /// Loaded from `fixtures/device-activity.json`.
    pub activity_fixture: Vec<ActivityRecord>,
    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    pub admin_token: String,

    /// Alert fixture list.
    /// Loaded from `fixtures/alerts.json`.
    pub alert_fixture: Vec<AlertRecord>,

    // --- Mutable state (reset by `reset_all()` / `reset_for(org_id)`) ---
    /// Stateful tag store: `(org_id, device_id) → set of tag_keys`.
    ///
    /// BC-3.2.001: composite key enforces per-org state isolation (ADR-008 §2.1 Step 6b).
    /// Populated via `POST /api/v1/devices/{device_id}/tags/`.
    /// Drained by `DELETE /api/v1/devices/{device_id}/tags/{tag_key}`.
    /// Merged into device records at query time by route handlers.
    pub tag_store: Mutex<HashMap<(OrgId, String), HashSet<String>>>,

    /// AQL capture log: ordered list of AQL strings received since last reset.
    ///
    /// Every AQL string from device query requests is appended here verbatim
    /// (no parsing, no validation — per R-DTU-002 mitigation).
    pub aql_log: Mutex<Vec<String>>,

    /// Per-instance HTTP request counter.
    ///
    /// Incremented atomically by the counting tower layer on EVERY request received
    /// by this clone's router (vendor API + DTU-internal routes alike).
    ///
    /// # Test-infra usage (AC-006 / INV-ISOLATION-001)
    ///
    /// Multi-tenant isolation tests read this counter via
    /// `ArmisClone::request_count()` to prove zero cross-tenant leakage
    /// from INSIDE the clone's router:
    ///
    /// ```text
    /// let count_a = clone_a.request_count();  // requests received by instance A's router
    /// let count_b = clone_b.request_count();  // requests received by instance B's router
    /// // After dispatching N requests to S_A and 0 to S_B:
    /// assert_eq!(count_a, N);
    /// assert_eq!(count_b, 0);   // zero cross-tenant leakage
    /// ```
    ///
    /// Reset by `reset_all()` / `BehavioralClone::reset()`.
    ///
    /// (BC-2.06.017 AC-006 / INV-ISOLATION-001; mirrors `ClarotyState::request_counter`)
    pub request_counter: AtomicU64,

    /// Shared failure mode, read by `FailureLayerShared` on every request.
    ///
    /// Wrapped in `Arc` so `build_router()` can clone it into the tower layer
    /// while `apply_config()` can mutate it after the server starts.
    pub failure_mode: Arc<Mutex<FailureMode>>,

    /// Authoritative `OrgId` for this clone instance (W3-FIX-SEC-001).
    ///
    /// Set at startup; route handlers compare the `X-Org-Id` header against this value
    /// and return HTTP 401 on mismatch (BC-3.5.002 precondition 3).
    pub instance_org_id: OrgId,

    // -----------------------------------------------------------------------
    // Story A: generated fixture field (BC-2.06.018 / ADR-036 §2.3)
    // -----------------------------------------------------------------------
    /// Generated device records from the fixture generator (BC-2.06.018 postcondition 1).
    ///
    /// Non-empty when the clone is constructed via `new_with_seed` AND the archetype produces
    /// records. EMPTY for DormantTenant even when seeded.
    ///
    /// Route handlers MUST use `fixture_gen_seeded` (not `generated_records.is_empty()`) to
    /// decide between the generated path and the static-fixture fallback.
    ///
    /// ADR-036 §2.3: immutable after construction (NOT Arc<Mutex<...>>).
    #[cfg(feature = "fixture-gen")]
    pub generated_records: Vec<serde_json::Value>,

    /// True when the clone was constructed via `new_with_seed` (fixture-gen path).
    ///
    /// Route handlers MUST use this flag (not `generated_records.is_empty()`) as the
    /// dual-path sentinel. F-P6-HIGH-001 / ADR-036 v2.2.
    #[cfg(feature = "fixture-gen")]
    pub fixture_gen_seeded: bool,

    // -----------------------------------------------------------------------
    // Story B: scenario timeline (BC-2.06.019 / ADR-036 v2.3 §2.3)
    // -----------------------------------------------------------------------
    /// Scenario incident timeline. `Some` when constructed via `new_with_scenario`.
    ///
    /// Route handlers branch on `timeline.is_some()` (scenario path) vs
    /// `fixture_gen_seeded && timeline.is_none()` (seeded-no-scenario path) vs
    /// `!fixture_gen_seeded` (static-JSON path).
    ///
    /// ADR-036 v2.3 §2.3: threaded as `Arc<IncidentTimeline>` (NOT `Arc<Mutex<...>>`).
    #[cfg(feature = "fixture-gen")]
    pub timeline: Option<std::sync::Arc<prism_dtu_common::IncidentTimeline>>,
}

impl ArmisState {
    /// Construct a fresh `ArmisState` with pre-loaded fixture registries.
    pub fn new(
        devices: Vec<DeviceRecord>,
        activity: Vec<ActivityRecord>,
        alerts: Vec<AlertRecord>,
    ) -> Self {
        Self::with_admin_token(devices, activity, alerts, uuid::Uuid::new_v4().to_string())
    }

    /// Construct with a specific admin token (used by clone to share between
    /// the route handler and BehavioralClone::admin_token()).
    ///
    /// W3-FIX-SEC-001: `instance_org_id` defaults to `DTU_DEFAULT_INSTANCE_ORG_ID`
    /// (`0000000000AA`) so that `ArmisClone::new()` has a deterministic org identity
    /// matching the `dummy_org` used in `x_org_id_auth::test_AC_001`.
    /// Callers that need a custom `instance_org_id` must use `with_admin_token_and_org`.
    pub fn with_admin_token(
        devices: Vec<DeviceRecord>,
        activity: Vec<ActivityRecord>,
        alerts: Vec<AlertRecord>,
        admin_token: String,
    ) -> Self {
        Self::with_admin_token_and_org(
            devices,
            activity,
            alerts,
            admin_token,
            DTU_DEFAULT_INSTANCE_ORG_ID,
        )
    }

    /// Construct with a specific admin token and explicit `instance_org_id`.
    ///
    /// Used by test helpers that need deterministic org identity for multi-tenant
    /// cross-org header validation tests (W3-FIX-SEC-001 AC-001..AC-003).
    pub fn with_admin_token_and_org(
        devices: Vec<DeviceRecord>,
        activity: Vec<ActivityRecord>,
        alerts: Vec<AlertRecord>,
        admin_token: String,
        instance_org_id: OrgId,
    ) -> Self {
        let device_registry: HashMap<String, DeviceRecord> = devices
            .iter()
            .map(|d| (d.device_id.clone(), d.clone()))
            .collect();

        Self {
            device_registry,
            devices_ordered: devices,
            activity_fixture: activity,
            alert_fixture: alerts,
            tag_store: Mutex::new(HashMap::new()),
            aql_log: Mutex::new(Vec::new()),
            request_counter: AtomicU64::new(0),
            failure_mode: Arc::new(Mutex::new(FailureMode::None)),
            admin_token,
            instance_org_id,
            // Story A: generated_records defaults empty (static-JSON path).
            // Populated by ArmisClone::new_with_seed().
            #[cfg(feature = "fixture-gen")]
            generated_records: Vec::new(),
            // fixture_gen_seeded: false on new() path — route handlers use static fixture.
            #[cfg(feature = "fixture-gen")]
            fixture_gen_seeded: false,
            // Story B: timeline is None on new() and new_with_seed() paths.
            #[cfg(feature = "fixture-gen")]
            timeline: None,
        }
    }

    /// Reset all mutable state to initial values (called by `BehavioralClone::reset`).
    ///
    /// Delegates to `reset_all()`. BC-3.2.001: `BehavioralClone::reset()` must
    /// continue to clear state for all orgs.
    pub fn reset(&self) {
        self.reset_all();
    }

    /// Reset all mutable state across all orgs (full-clear variant of reset).
    ///
    /// - Clears the tag store (all `(OrgId, device_id)` entries removed).
    /// - Clears the AQL log (all captured AQL strings removed).
    /// - Resets the failure mode to `FailureMode::None`.
    /// - Immutable fixture registries are NOT affected.
    pub fn reset_all(&self) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut tags = self.tag_store.lock().expect("tag_store poisoned");
        tags.clear();

        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        let mut aql = self.aql_log.lock().expect("aql_log poisoned");
        aql.clear();

        // Reset the per-instance request counter.
        self.request_counter.store(0, Ordering::SeqCst);

        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        let mut mode = self.failure_mode.lock().expect("failure_mode poisoned");
        *mode = FailureMode::None;
    }

    /// Remove all tag store entries for `org_id`, leaving other orgs' entries intact.
    ///
    /// BC-3.2.001 edge case EC-004: selective per-org reset. AQL log and failure mode
    /// are global (not per-org) and are NOT affected by `reset_for`.
    pub fn reset_for(&self, org_id: OrgId) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut tags = self.tag_store.lock().expect("tag_store poisoned");
        tags.retain(|(key_org, _), _| *key_org != org_id);
    }

    /// Apply a JSON configuration patch (from `POST /dtu/configure`).
    ///
    /// Accepted schemas:
    ///
    /// **Armis-native schema:**
    /// - `"failure_mode"` — one of `"none"`, `"rate_limit"`, `"malformed_response"`,
    ///   `"auth_reject"`, `"internal_error"`, `"network_timeout"`.
    ///   For `"rate_limit"` companions: `"after_n_requests"` (u32), `"retry_after_secs"` (u32).
    ///
    /// **Harness generic schema (clone_server compat, CR-011 fix):**
    /// - `{"auth_mode": "reject"}` → `AuthReject`
    /// - `{"auth_mode": "none"}` → `None`
    /// - `{"clear": true}` → `None`
    /// - `{"rate_limit_after": N, "retry_after_secs": M}` → `RateLimit`
    /// - `{"internal_error_at": N}` → `InternalError`
    /// - `{"network_timeout_ms": ms}` → `NetworkTimeout`
    /// - `{"malformed_response": true}` → `MalformedResponse`
    /// - `{"unprocessable_at": N}` → `Unprocessable`
    ///
    /// Unknown fields are rejected (TD-WV0-04: `deny_unknown_fields`).
    pub fn apply_config(&self, config: &serde_json::Value) -> anyhow::Result<()> {
        let payload: ConfigPayload = serde_json::from_value(config.clone())
            .map_err(|e| anyhow::anyhow!("invalid /dtu/configure payload: {e}"))?;

        // Determine new_mode from whichever schema was provided.
        // Priority: Armis-native `failure_mode` field > harness generic fields.
        let new_mode: Option<FailureMode> = if let Some(mode_str) = payload.failure_mode.as_deref()
        {
            // Armis-native schema.
            let m = match mode_str {
                "none" => FailureMode::None,
                "rate_limit" => {
                    let after_n = payload.after_n_requests.unwrap_or(0);
                    let retry_after = payload.retry_after_secs.unwrap_or(30);
                    FailureMode::RateLimit {
                        after_n_requests: after_n,
                        retry_after_secs: retry_after,
                    }
                }
                "malformed_response" => FailureMode::MalformedResponse,
                "auth_reject" => FailureMode::AuthReject,
                "internal_error" => {
                    let at_n = payload.at_request_n.unwrap_or(1);
                    FailureMode::InternalError { at_request_n: at_n }
                }
                "network_timeout" => {
                    let after_ms = payload.after_ms.unwrap_or(5000);
                    FailureMode::NetworkTimeout { after_ms }
                }
                other => {
                    anyhow::bail!("unknown failure_mode: {other}");
                }
            };
            Some(m)
        } else if payload.clear == Some(true) {
            // Harness generic: {"clear": true} → None.
            Some(FailureMode::None)
        } else if let Some(auth_mode) = payload.auth_mode.as_deref() {
            // Harness generic: {"auth_mode": "reject"} / {"auth_mode": "none"}.
            let m = match auth_mode {
                "reject" => FailureMode::AuthReject,
                "none" => FailureMode::None,
                other => anyhow::bail!("unknown auth_mode: {other}"),
            };
            Some(m)
        } else if let Some(rate_limit_after) = payload.rate_limit_after {
            // Harness generic: {"rate_limit_after": N, "retry_after_secs": M}.
            let retry_after = payload.retry_after_secs.unwrap_or(30);
            Some(FailureMode::RateLimit {
                after_n_requests: rate_limit_after,
                retry_after_secs: retry_after,
            })
        } else if let Some(at_n) = payload.internal_error_at {
            // Harness generic: {"internal_error_at": N}.
            Some(FailureMode::InternalError { at_request_n: at_n })
        } else if let Some(after_ms) = payload.network_timeout_ms {
            // Harness generic: {"network_timeout_ms": ms}.
            Some(FailureMode::NetworkTimeout { after_ms })
        } else if payload.malformed_response == Some(true) {
            // Harness generic: {"malformed_response": true}.
            Some(FailureMode::MalformedResponse)
        } else {
            // Harness generic: {"unprocessable_at": N}, or no recognized field → no-op.
            payload
                .unprocessable_at
                .map(|at_n| FailureMode::Unprocessable { at_request_n: at_n })
        };

        if let Some(mode) = new_mode {
            // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
            #[allow(clippy::expect_used)]
            let mut guard = self
                .failure_mode
                .lock()
                .expect("ArmisState: failure_mode lock poisoned");
            *guard = mode;
        }
        Ok(())
    }

    /// Return the number of HTTP requests received by this clone's router since construction
    /// (or last `reset_all()`).
    ///
    /// Incremented by the counting tower layer in `ArmisClone::build_router()` on every
    /// incoming request — vendor API routes AND DTU-internal routes alike.
    ///
    /// # Multi-tenant isolation usage
    ///
    /// This is the load-bearing server-side counter for AC-006 / INV-ISOLATION-001.
    /// Test code reads this value AFTER dispatching requests to verify zero cross-tenant
    /// leakage from INSIDE the clone's router (not merely from the client side):
    ///
    /// ```text
    /// // Before: both clones at 0.
    /// // Dispatch 5 requests to clone A's SocketAddr.
    /// assert_eq!(clone_a.request_count(), 5);  // all 5 reached A's router
    /// assert_eq!(clone_b.request_count(), 0);  // zero reached B's router
    /// ```
    ///
    /// (BC-2.06.017 AC-006 / INV-ISOLATION-001)
    pub fn received_request_count(&self) -> u64 {
        self.request_counter.load(Ordering::SeqCst)
    }

    /// Append an AQL string to the capture log.
    ///
    /// Called by device query route handlers. Stores verbatim — no parsing.
    pub fn capture_aql(&self, aql: &str) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut log = self.aql_log.lock().expect("aql_log poisoned");
        log.push(aql.to_owned());
    }

    /// Return all AQL strings received since last reset (for `GET /dtu/aql-log`).
    pub fn aql_log(&self) -> Vec<String> {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let log = self.aql_log.lock().expect("aql_log poisoned");
        log.clone()
    }

    /// Add a tag to a device's tag set under a specific org.
    ///
    /// BC-3.2.001: key is `(org_id, device_id)` — writes are org-scoped.
    /// Returns `true` if newly added (idempotent on re-add).
    pub fn add_tag(&self, org_id: OrgId, device_id: &str, tag_key: &str) -> bool {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut store = self.tag_store.lock().expect("tag_store poisoned");
        store
            .entry((org_id, device_id.to_owned()))
            .or_default()
            .insert(tag_key.to_owned())
    }

    /// Remove a tag from a device's tag set under a specific org.
    ///
    /// BC-3.2.001: key is `(org_id, device_id)` — removes are org-scoped.
    /// Returns `true` if tag was present and removed.
    pub fn remove_tag(&self, org_id: OrgId, device_id: &str, tag_key: &str) -> bool {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut store = self.tag_store.lock().expect("tag_store poisoned");
        if let Some(tags) = store.get_mut(&(org_id, device_id.to_owned())) {
            tags.remove(tag_key)
        } else {
            false
        }
    }

    /// Return the merged tag set for a device under a specific org (fixture tags + tag_store tags).
    ///
    /// BC-3.2.001 postcondition 1: lookup under org_id_B for an entry stored under org_id_A returns
    /// the default (empty / fixture-only) value. Cross-org leakage is structurally impossible.
    pub fn tags_for(&self, org_id: OrgId, device_id: &str, fixture_tags: &[String]) -> Vec<String> {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let store = self.tag_store.lock().expect("tag_store poisoned");
        let mut tags: Vec<String> = fixture_tags.to_vec();
        if let Some(stored) = store.get(&(org_id, device_id.to_owned())) {
            for t in stored {
                if !tags.contains(t) {
                    tags.push(t.clone());
                }
            }
        }
        tags
    }
}
