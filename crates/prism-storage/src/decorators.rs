//! DecorationStore — three-phase decorator cache (S-2.03, BC-2.15.010).
//!
//! Manages decorator values across the three lifecycle phases:
//!
//! - **Phase 1 (config-time):** populated at startup and on config reload from
//!   TOML; stored in an in-memory `HashMap` keyed by `TenantId`.
//! - **Phase 2 (query-time):** computed inline per query by the caller
//!   (prism-query, S-3.02); never stored here.
//! - **Phase 3 (periodic):** refreshed on a configurable interval; serialized
//!   with bincode v2 and persisted in the RocksDB `decorators` CF so values
//!   survive process restarts.
//!
//! ## Merge semantics
//!
//! `merge(config_time, query_time, periodic)` applies fields in ascending
//! priority order: config-time < query-time < periodic (last-write-wins).
//! If a field is `Some` in a higher-priority phase it overrides any `Some`
//! value from a lower-priority phase.  `None` values never override `Some`.
//!
//! ## Architecture compliance
//!
//! - `DecorationStore` holds `Arc<dyn RocksStorageBackend>`, NOT a concrete
//!   `RocksDbBackend`, so tests can inject `InMemoryBackend` (S-2.02 pattern).
//! - bincode v2 API only (`encode_to_vec` / `decode_from_slice`).
//! - No DataFusion, Arrow, or arrow-schema imports (architecture hard boundary).

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use prism_core::{DecoratorContext, PrismError, TenantId};

use crate::backend::RocksStorageBackend;

/// Key prefix for periodic decorator entries in the `decorators` CF.
#[allow(dead_code)]
const PERIODIC_KEY_PREFIX: &str = "periodic:";

/// Thread-safe store for Phase 1 and Phase 3 decorator values.
///
/// Phase 2 (query-time) values are always computed inline by prism-query
/// (S-3.02) and passed directly to [`DecorationStore::merge`].  They are
/// never stored in this struct.
#[allow(dead_code)]
pub struct DecorationStore {
    /// Phase 1: in-memory map keyed by TenantId, updated at startup and on
    /// config reload (BC-2.15.010 Phase 1 — config-time decorators always
    /// available after startup with no delay).
    config_time: Arc<RwLock<HashMap<TenantId, DecoratorContext>>>,

    /// Phase 3: periodic cache backed by the RocksDB `decorators` CF
    /// (BC-2.15.010 Phase 3 — periodic decorators cached for persistence
    /// across restarts).
    backend: Arc<dyn RocksStorageBackend>,
}

impl DecorationStore {
    /// Construct a new `DecorationStore` backed by the provided storage backend.
    ///
    /// The `config_time` map starts empty; callers must call
    /// [`store_config_time`][DecorationStore::store_config_time] for each
    /// tenant during startup.
    pub fn new(backend: Arc<dyn RocksStorageBackend>) -> Self {
        Self {
            config_time: Arc::new(RwLock::new(HashMap::new())),
            backend,
        }
    }

    /// Store a config-time `DecoratorContext` for the given tenant.
    ///
    /// Called at startup and on config reload (BC-2.15.010 Phase 1).
    /// This is an in-memory write; it does not touch RocksDB.
    ///
    /// # AC-1
    /// After this call, `get_config_time(tenant_id)` returns a context with
    /// `client_name` and `prism_version` populated (assuming the caller set
    /// those fields).
    pub async fn store_config_time(&self, _tenant: TenantId, _ctx: DecoratorContext) {
        todo!(
            "BC-2.15.010 Phase 1 postcondition: write ctx into the config_time map \
             keyed by tenant; called at startup and on config reload. \
             AC-1: after this call, get_config_time returns the stored context."
        )
    }

    /// Read the config-time `DecoratorContext` for the given tenant.
    ///
    /// Returns `None` if the tenant has no config-time entry (e.g., before
    /// startup initialization for this tenant).  Never panics (BC-2.15.009 —
    /// decorator injection cannot fail).
    pub async fn get_config_time(&self, _tenant: &TenantId) -> Option<DecoratorContext> {
        todo!(
            "BC-2.15.010 Phase 1 — read from config_time map; return None for missing \
             tenants without error. AC-1: returned context has client_name + prism_version."
        )
    }

    /// Persist a periodic `DecoratorContext` in the RocksDB `decorators` CF.
    ///
    /// Serializes `ctx` with `bincode::encode_to_vec` (bincode v2 API) and
    /// writes it under the key `"periodic:{tenant_id}"`.
    ///
    /// On failure the caller MUST log a `tracing::warn!` event and continue
    /// using the last successfully cached value (stale-value pattern,
    /// E-DECOR-001, AC-6).
    ///
    /// # Errors
    /// Returns `Err(PrismError::StorageWriteFailed { … })` if the RocksDB
    /// write fails.
    pub async fn store_periodic(
        &self,
        _tenant: &TenantId,
        _ctx: &DecoratorContext,
    ) -> Result<(), PrismError> {
        todo!(
            "BC-2.15.010 Phase 3 postcondition: serialize ctx with bincode::encode_to_vec \
             and write to decorators CF under key 'periodic:{{tenant_id}}'. \
             AC-4: round-trip via bincode + RocksDB must survive process restart. \
             E-DECOR-001: on error, log tracing::warn! and return Err so caller can \
             apply stale-value pattern."
        )
    }

    /// Load the cached periodic `DecoratorContext` from the RocksDB `decorators` CF.
    ///
    /// Returns `None` if no cached entry exists for the tenant yet (e.g., before
    /// the first periodic refresh — EC-15-039).  Deserializes with
    /// `bincode::decode_from_slice` (bincode v2 API).
    ///
    /// # Errors
    /// Returns `Err` if the RocksDB read fails or bincode deserialization fails
    /// on a non-empty entry.
    pub async fn load_periodic(
        &self,
        _tenant: &TenantId,
    ) -> Result<Option<DecoratorContext>, PrismError> {
        todo!(
            "BC-2.15.010 Phase 3 — prefix-scan decorators CF for 'periodic:{{tenant_id}}'; \
             deserialize with bincode::decode_from_slice. Return None if no entry exists \
             (EC-15-039: first query before first periodic refresh). \
             AC-4: deserialized context must match the value written by store_periodic."
        )
    }

    /// Merge three phase contexts into a single `DecoratorContext`.
    ///
    /// Priority order (last-write-wins, highest priority last):
    ///   config-time < query-time < periodic
    ///
    /// Fields are merged per-field: a `Some` value from a higher-priority phase
    /// overrides any value (including `Some`) from a lower-priority phase.
    /// `None` values do NOT override `Some` values from lower phases.
    ///
    /// `periodic` is `Option<&DecoratorContext>` because it may not yet be
    /// cached (EC-15-039 — first query before first periodic refresh).
    ///
    /// # AC-2
    /// `merge(config_time, query_time, None)` where `query_time` has
    /// `analyst_id = Some("joshua")` → merged result has `analyst_id = Some("joshua")`
    /// and all Phase 1 fields carried through unchanged.
    ///
    /// # AC-5
    /// `merge(config_time, query_time, Some(periodic))` where config-time has
    /// `client_name = Some("Acme")` and periodic has `sensor_health_status =
    /// Some("healthy")` → merged result contains all three values.
    pub fn merge(
        _config_time: &DecoratorContext,
        _query_time: &DecoratorContext,
        _periodic: Option<&DecoratorContext>,
    ) -> DecoratorContext {
        todo!(
            "BC-2.15.010 invariant: last-write-wins merge of config_time < query_time < periodic. \
             AC-2: query-time fields override config-time for overlapping keys. \
             AC-5: periodic > query-time > config-time for any overlapping keys. \
             None values from a higher phase must NOT clobber Some values from a lower phase."
        )
    }
}

/// Private helper: construct the RocksDB key for a periodic decorator entry.
///
/// Key format: `"periodic:{tenant_id}"` (UTF-8 bytes).
#[allow(dead_code)]
fn periodic_key(tenant: &TenantId) -> Vec<u8> {
    format!("{}{}", PERIODIC_KEY_PREFIX, tenant.as_str()).into_bytes()
}
