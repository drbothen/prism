//! Testable extracted functions for the `start-multi` subcommand.
//!
//! These functions live in the library crate (not `main.rs`) so that integration
//! tests in `tests/multi_org.rs` can call them directly without subprocess overhead.
//!
//! # Architecture Compliance (S-DEMO-LAUNCHER-CONSOLIDATION-001)
//!
//! Per the story spec §Architecture Compliance Rules:
//! - `build_multi_clone_factory` is a "separately named, testable function" so RG-004 can
//!   test it directly. It must be `pub(crate)` in `main.rs` originally but re-exported here
//!   for integration test access.
//! - `start_multi_for_config` is a "testable extracted async fn" so RG-005 can verify
//!   socket isolation without spawning a subprocess.
//!
//! # fixture-gen gate (GAP-1)
//!
//! `build_multi_clone_factory` is only meaningful with `feature = "fixture-gen"` because
//! the seeded `new_with_seed` constructors are `#[cfg(feature = "fixture-gen")]`-gated.
//! The `#[cfg(not(feature = "fixture-gen"))]` arm panics unconditionally to prevent
//! silent fallback to unseeded `new()` (INV-DISTINCT-DATA-001 violation).
//!
//! Story anchor: S-DEMO-LAUNCHER-CONSOLIDATION-001 v2.1

use crate::{MultiInstanceServers, MultiOrgDemoConfig};
use prism_dtu_common::BehavioralClone;

use crate::multi_instance::InstanceEntry;

/// Type alias for the clone factory closure returned by `build_multi_clone_factory`.
///
/// Maps `&InstanceEntry` (name = `"{org_slug}-{sensor_id}"`) to a `Box<dyn BehavioralClone>`.
/// Named here to avoid the `clippy::type_complexity` lint on the return type.
pub type CloneFactoryFn<'a> = Box<dyn Fn(&InstanceEntry) -> Box<dyn BehavioralClone> + 'a>;

/// Build the `clone_factory` closure for `start_instances`.
///
/// Returns a boxed `Fn(&InstanceEntry) -> Box<dyn BehavioralClone>` that dispatches
/// `(org_slug, sensor_id)` (derived from `entry.name` via the `"{org_slug}-{sensor_id}"`
/// convention) to the correct seeded clone constructor.
///
/// # fixture-gen hard-requirement (GAP-1)
///
/// This function requires `feature = "fixture-gen"`. Without it, the `#[cfg(not(feature))]`
/// stub panics with a clear error message. Silently falling back to unseeded `new()` is
/// FORBIDDEN — it would make org-a and org-c CrowdStrike clones serve IDENTICAL data,
/// violating INV-DISTINCT-DATA-001 while still passing RG-005 socket-distinctness test.
///
/// # Reuses harness.rs helpers (Architecture Compliance)
///
/// The implementer MUST use:
/// - `crate::harness::parse_org_id(str, name)` → `OrgId`
/// - `crate::harness::fixture_set_to_archetype(fixture_set, name)` → `Archetype`
/// - `prism_dtu_common::demo_time_anchor()` for the time anchor
/// - `new_with_seed(seed, archetype, org_id)` seeded constructors on all 4 clone crates
///
/// # Red Gate stub
///
/// Body is `todo!()`. RG-004 FAILS with "not yet implemented" at the Red Gate phase.
#[cfg(feature = "fixture-gen")]
pub fn build_multi_clone_factory(cfg: &MultiOrgDemoConfig) -> CloneFactoryFn<'_> {
    // Red Gate stub: todo!() causes RG-004 to FAIL.
    // Implementer: replace with the real dispatch closure that:
    //   1. Splits entry.name on the LAST '-' before a known sensor name to get (org_slug, sensor_id)
    //   2. Looks up OrgConfig from cfg.orgs[org_slug]
    //   3. Calls harness::parse_org_id(org_cfg.org_id, entry.name)
    //   4. Calls harness::fixture_set_to_archetype("default", entry.name) for the archetype
    //   5. Constructs the clone via new_with_seed(org_cfg.seed, archetype, org_id)
    //   6. For Cyberint: if initial_access_token.is_some(), calls configure({"access_token": token})
    let _ = cfg;
    todo!(
        "build_multi_clone_factory: not yet implemented — Red Gate stub \
         (S-DEMO-LAUNCHER-CONSOLIDATION-001 Phase 3). \
         Implementer: replace this todo!() with the real dispatch closure."
    )
}

/// Fallback stub for `build_multi_clone_factory` when `fixture-gen` is absent.
///
/// Panics unconditionally. NEVER silently falls back to unseeded `new()` (GAP-1).
#[cfg(not(feature = "fixture-gen"))]
pub fn build_multi_clone_factory(_cfg: &MultiOrgDemoConfig) -> CloneFactoryFn<'static> {
    panic!(
        "prism-dtu-demo-server: start-multi requires the `fixture-gen` feature. \
         Rebuild with `--features dtu,fixture-gen`. \
         Without `fixture-gen`, seeded constructors (new_with_seed) are unavailable. \
         Silently falling back to unseeded new() would violate INV-DISTINCT-DATA-001. \
         This panic is INTENTIONAL (S-DEMO-LAUNCHER-CONSOLIDATION-001 GAP-1)."
    )
}

/// Build `MultiInstanceConfig` + start all org clone instances.
///
/// Extracted as a separately-testable async fn so RG-005 can verify socket isolation
/// without subprocess overhead. `cmd_start_multi` in `main.rs` delegates to this.
///
/// # Contract
///
/// - Reads `cfg.orgs` to build `MultiInstanceConfig` entries named `"{org_slug}-{sensor_id}"`.
/// - Calls `build_multi_clone_factory(cfg)` to produce the clone factory.
/// - Calls `start_instances(multi_cfg, clone_factory).await` from `crate::multi_instance`.
/// - Returns `Ok(MultiInstanceServers)` with a socket_map keyed by `"{org_slug}-{sensor_id}"`.
///
/// # Red Gate stub
///
/// Body is `todo!()`. RG-005 FAILS with "not yet implemented" at the Red Gate phase.
pub async fn start_multi_for_config(
    _cfg: &MultiOrgDemoConfig,
) -> anyhow::Result<MultiInstanceServers> {
    todo!(
        "start_multi_for_config: not yet implemented — Red Gate stub \
         (S-DEMO-LAUNCHER-CONSOLIDATION-001 Phase 3). \
         Implementer: replace this todo!() with the real bind logic."
    )
}
