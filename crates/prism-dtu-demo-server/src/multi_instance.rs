//! Multi-instance binding for `prism-dtu-demo-server`.
//!
//! Provides [`MultiInstanceConfig`] and the [`start_instances`] function for
//! binding N named DTU clone instances to distinct socket addresses.
//!
//! # Story anchor
//!
//! S-DEMO-MULTI-TENANT-DTU-001 (BC-2.06.017 Postcondition 1)
//!
//! # Architecture note (D-1075)
//!
//! These types live in `prism-dtu-demo-server`, NOT `prism-dtu-common`, to avoid
//! coupling all downstream clone crates to orchestration types.
//!
//! All public structs and enums carry `#[non_exhaustive]` per
//! `CLAUDE.md §#[non_exhaustive] discipline` and `BC-2.06.017 INV-NONEXHAUSTIVE-001`.

use std::{collections::HashMap, net::SocketAddr};

use prism_dtu_common::BehavioralClone;

/// Configuration for starting N named DTU clone instances at distinct addresses.
///
/// Each entry in `instances` starts exactly one clone instance via
/// `BehavioralClone::start_on`. Duplicate `InstanceEntry::name` values return
/// `Err(MultiInstanceBindError::DuplicateName { name })` before any bind attempts
/// (BC-2.06.017 Postcondition 7).
///
/// A zero-length `instances` vec is valid and returns `Ok(HashMap::new())`
/// with no spawned tasks (BC-2.06.017 EC-017-002).
///
/// (BC-2.06.017 Postcondition 1 — multi-instance bind configuration accepted)
#[non_exhaustive]
pub struct MultiInstanceConfig {
    /// Ordered list of named clone instances to bind.
    pub instances: Vec<InstanceEntry>,
}

/// A single named DTU clone instance bind entry.
///
/// `name` uniquely identifies the instance within a [`MultiInstanceConfig`].
/// Duplicate names return `Err(MultiInstanceBindError::DuplicateName { name })`
/// (BC-2.06.017 EC-017-009 / Postcondition 7).
///
/// `bind` is typically `127.0.0.1:0` so the OS assigns an ephemeral port.
///
/// (BC-2.06.017 Postcondition 1)
#[non_exhaustive]
pub struct InstanceEntry {
    /// Human-readable instance name (e.g., `"armis-acme"`).
    pub name: String,
    /// Bind address; use `127.0.0.1:0` for OS-assigned ephemeral port.
    pub bind: SocketAddr,
}

impl MultiInstanceConfig {
    /// Construct a `MultiInstanceConfig` with the given instances.
    ///
    /// This is the canonical constructor for external callers (required because
    /// `#[non_exhaustive]` prevents struct-literal construction outside this crate).
    pub fn new(instances: Vec<InstanceEntry>) -> Self {
        Self { instances }
    }
}

impl InstanceEntry {
    /// Construct an `InstanceEntry` with the given name and bind address.
    ///
    /// This is the canonical constructor for external callers (required because
    /// `#[non_exhaustive]` prevents struct-literal construction outside this crate).
    pub fn new(name: impl Into<String>, bind: std::net::SocketAddr) -> Self {
        Self {
            name: name.into(),
            bind,
        }
    }
}

/// Error returned by [`start_instances`].
///
/// Two variants:
/// - `DuplicateName`: returned immediately (before any bind) when two
///   [`InstanceEntry`] values share the same `name`.
/// - `BindFailure`: returned after all bind operations are attempted (multi-error
///   aggregation per BC-2.06.017 INV-ERR-003-COMPAT / Postcondition 6).
///
/// (BC-2.06.017 Postconditions 6–7)
#[derive(Debug)]
#[non_exhaustive]
pub enum MultiInstanceBindError {
    /// Two [`InstanceEntry`] items share the same `name` string.
    ///
    /// Returned before any bind attempt (EC-017-009).
    DuplicateName {
        /// The duplicated name string.
        name: String,
    },

    /// One or more bind operations failed.
    ///
    /// All N bind operations are attempted before this error is returned; the
    /// `Vec` contains every failure (INV-ERR-003-COMPAT). Successfully-started
    /// instances are shut down before the error is returned (no zombie instances).
    BindFailure(Vec<DemoBindError>),
}

/// A single bind failure within a [`MultiInstanceBindError::BindFailure`] error.
///
/// (BC-2.06.017 Postcondition 6 + v1.1 Amendment 2 — name disambiguation from
/// the harness-side `BindError` type)
#[derive(Debug)]
#[non_exhaustive]
pub struct DemoBindError {
    /// The `InstanceEntry::name` of the instance that failed to bind.
    pub instance_name: String,
    /// The underlying OS bind error.
    pub source: std::io::Error,
}

/// Start N named DTU clone instances at distinct socket addresses.
///
/// # Behaviour
///
/// - **Duplicate check first:** if any two entries share the same `name`,
///   returns `Err(MultiInstanceBindError::DuplicateName { name })` before
///   any bind attempt (BC-2.06.017 Postcondition 7 / EC-017-009).
/// - **Multi-error aggregation:** all bind operations are attempted before
///   returning any error (INV-ERR-003-COMPAT). If any fail, successfully-started
///   instances are shut down and `Err(BindFailure(failures))` is returned.
/// - **Zero instances:** `MultiInstanceConfig { instances: [] }` returns
///   `Ok(HashMap::new())` — no error, no spawned tasks (EC-017-002).
/// - On success, all N instances are serving requests before the function returns.
///
/// # Parameters
///
/// - `cfg`: the multi-instance configuration specifying each instance name and bind address.
/// - `clone_factory`: a factory closure that produces a `Box<dyn BehavioralClone>` for
///   a given [`InstanceEntry`]. The factory is called at most once per entry.
///
/// # Returns
///
/// `Ok(HashMap<String, SocketAddr>)` — maps each `entry.name` to its OS-assigned bound
/// address. All N entries appear in the map; no entries are silently dropped.
///
/// (BC-2.06.017 Postcondition 1)
pub async fn start_instances(
    _cfg: MultiInstanceConfig,
    _clone_factory: impl Fn(&InstanceEntry) -> Box<dyn BehavioralClone>,
) -> Result<HashMap<String, SocketAddr>, MultiInstanceBindError> {
    todo!(
        "S-DEMO-MULTI-TENANT-DTU-001: start_instances not yet implemented \
         (BC-2.06.017 Postcondition 1 — TDD Red Gate stub)"
    )
}
