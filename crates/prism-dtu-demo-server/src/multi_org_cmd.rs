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
/// # Entry name convention
///
/// `entry.name` is `"{org_slug}-{sensor_id}"` (e.g. `"org-a-crowdstrike"`). Because
/// org slugs may contain `-` (e.g. `"org-a"`), we detect the sensor_id by matching the
/// known sensor names as suffixes: `crowdstrike`, `armis`, `claroty`, `cyberint`.
/// The org_slug is everything before the last `-{sensor_id}` suffix.
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
/// Uses:
/// - `crate::harness::parse_org_id(str, name)` → `OrgId`
/// - `crate::harness::fixture_set_to_archetype("default", name)` → `Archetype`
/// - `prism_dtu_common::demo_time_anchor()` for the time anchor
/// - `new_with_seed(seed, archetype, org_id)` seeded constructors on all 4 clone crates
///
/// # Cyberint composite path (GAP-2)
///
/// `CyberintClone::new_with_seed` does NOT set `initial_access_token`. To satisfy BOTH
/// seed-based data distinctness AND access-token auth, the factory uses the composite pattern:
/// 1. Call `new_with_seed(seed, archetype, org_id)` to produce the seeded clone.
/// 2. If `org_cfg.initial_access_token.is_some()`, apply the token synchronously via
///    `clone.state.apply_config(...)`. `BehavioralClone::configure` is declared async in
///    the trait, but `CyberintState::apply_config` is synchronous. Calling `block_on`
///    inside the factory closure (which runs on a tokio worker thread) would panic with
///    "Cannot start a runtime from within a runtime". We bypass the trait's async wrapper
///    and call `clone.state.apply_config` directly while we still hold the concrete
///    `CyberintClone` type (before `Box::new(clone)` erases it to `dyn BehavioralClone`).
///
/// # EC-008 / EC-009
///
/// Unrecognized sensor names or entry names that do not match the `{org_slug}-{sensor_id}`
/// convention result in a panic with an actionable message (programming error, not user error).
#[cfg(feature = "fixture-gen")]
pub fn build_multi_clone_factory(cfg: &MultiOrgDemoConfig) -> CloneFactoryFn<'_> {
    use prism_dtu_armis::ArmisClone;
    use prism_dtu_claroty::ClarotyClone;
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_crowdstrike::CrowdstrikeClone;
    use prism_dtu_cyberint::CyberintClone;

    // Known sensor names in suffix-search order. The name convention is
    // "{org_slug}-{sensor_id}" where both parts may contain '-' themselves
    // (e.g. "org-a-crowdstrike"). We match by stripping known sensor suffixes.
    const SENSORS: &[&str] = &["crowdstrike", "armis", "claroty", "cyberint"];

    Box::new(move |entry: &InstanceEntry| -> Box<dyn BehavioralClone> {
        let entry_name = &entry.name;

        // Parse (org_slug, sensor_id) from entry.name by matching known sensor suffixes.
        // Try each sensor as a suffix "-{sensor_id}"; first match wins.
        let (org_slug, sensor_id) = SENSORS
            .iter()
            .find_map(|&sensor| {
                let suffix = format!("-{sensor}");
                entry_name
                    .strip_suffix(suffix.as_str())
                    .map(|slug| (slug, sensor))
            })
            .unwrap_or_else(|| {
                panic!(
                    "start-multi: EC-009: InstanceEntry name '{entry_name}' does not match \
                     the '{{org_slug}}-{{sensor_id}}' convention. \
                     Valid sensor suffixes: crowdstrike, armis, claroty, cyberint. \
                     This is a programming error in cmd_start_multi (entries are built there)."
                )
            });

        // Look up OrgConfig from cfg.orgs.
        let org_cfg = cfg.orgs.get(org_slug).unwrap_or_else(|| {
            panic!(
                "start-multi: EC-009: org_slug '{org_slug}' (derived from entry '{entry_name}') \
                 not found in MultiOrgDemoConfig.orgs. \
                 This is a programming error — entries must be built from the config's org keys."
            )
        });

        // Derive OrgId from org_cfg.org_id (UUID string).
        // SAFETY: parse_org_id returns Err only for invalid UUIDs; we use expect() with an
        //         actionable message because this is a programming error path (config was
        //         validated by from_str with deny_unknown_fields before we get here).
        #[allow(clippy::expect_used)]
        let org_id = crate::harness::parse_org_id(&org_cfg.org_id, entry_name)
            .expect("org_id in OrgConfig must be a valid UUID (validated at config parse time)");

        // Derive Archetype from "default" fixture_set (start-multi always uses the seeded path).
        // SAFETY: "default" is a known-valid fixture_set; expect() is appropriate here.
        #[allow(clippy::expect_used)]
        let archetype = crate::harness::fixture_set_to_archetype("default", entry_name)
            .expect("'default' is a valid fixture_set; this expect cannot fail");

        let seed = org_cfg.seed;

        // Dispatch to the correct seeded constructor based on sensor_id (EC-008).
        // Each new_with_seed constructor uses demo_time_anchor() internally for the time anchor.
        match sensor_id {
            "crowdstrike" => {
                // CrowdstrikeClone::new_with_seed is infallible.
                Box::new(CrowdstrikeClone::new_with_seed(seed, archetype, org_id))
            }
            "claroty" => {
                // ClarotyClone::new_with_seed is infallible.
                Box::new(ClarotyClone::new_with_seed(seed, archetype, org_id))
            }
            "armis" => {
                // ArmisClone::new_with_seed is fallible (returns Result).
                #[allow(clippy::expect_used)]
                Box::new(ArmisClone::new_with_seed(seed, archetype, org_id).expect(
                    "ArmisClone::new_with_seed must succeed for valid seed/archetype/org_id",
                ))
            }
            "cyberint" => {
                // GAP-2 composite path:
                //   1. new_with_seed → seeded clone (no access_token yet)
                //   2. if initial_access_token.is_some() → apply token synchronously via
                //      clone.state.apply_config (CyberintState::apply_config is sync).
                //
                // We call clone.state.apply_config BEFORE Box::new(clone) so that the
                // concrete CyberintClone type is still in scope. After boxing to
                // Box<dyn BehavioralClone>, only the async configure() trait method is
                // accessible — which would require block_on and panic on a tokio thread.
                #[allow(clippy::expect_used)]
                let clone = CyberintClone::new_with_seed(seed, archetype, org_id).expect(
                    "CyberintClone::new_with_seed must succeed for valid seed/archetype/org_id",
                );

                if let Some(token) = &org_cfg.initial_access_token {
                    // GAP-2: register access token synchronously via the state's sync path.
                    // CyberintState::apply_config is synchronous; calling it directly avoids
                    // the block_on-within-tokio-runtime panic (CRIT-2 fix).
                    #[allow(clippy::expect_used)]
                    clone
                        .state
                        .apply_config(&serde_json::json!({"access_token": token}))
                        .expect("CyberintState::apply_config(access_token) must succeed");
                }

                Box::new(clone)
            }
            other => {
                // EC-008: unrecognized sensor name is a programming error.
                panic!(
                    "start-multi: EC-008: unrecognized sensor '{other}' in entry '{entry_name}'; \
                     valid values: crowdstrike, armis, claroty, cyberint. \
                     Update MultiOrgDemoConfig validation if new sensors are added."
                )
            }
        }
    })
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
/// # Entry name convention
///
/// Each `InstanceEntry` name is `"{org_slug}-{sensor_id}"` (e.g. `"org-a-crowdstrike"`).
/// The `build_multi_clone_factory` closure reconstructs `(org_slug, sensor_id)` from this
/// name by stripping known sensor suffixes (Architecture Compliance Rule).
///
/// # Bind address
///
/// Each instance binds to `"{cfg.harness.bind}:0"` — port 0 means OS-assigned ephemeral
/// port, which `start_instances` resolves to a real `SocketAddr` in the returned
/// `MultiInstanceServers::socket_map()`.
pub async fn start_multi_for_config(
    cfg: &MultiOrgDemoConfig,
) -> anyhow::Result<MultiInstanceServers> {
    use crate::multi_instance::{InstanceEntry, MultiInstanceConfig};

    // Build MultiInstanceConfig entries named "{org_slug}-{sensor_id}".
    // The iteration order of a HashMap is not deterministic, but the binding order
    // does not matter — all N entries must bind before this returns Ok.
    let mut instances = Vec::new();
    for (org_slug, org_cfg) in &cfg.orgs {
        for sensor_id in &org_cfg.sensors {
            let name = format!("{org_slug}-{sensor_id}");
            let bind_str = format!("{}:0", cfg.harness.bind);
            let bind: std::net::SocketAddr = bind_str
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid bind address '{}': {}", bind_str, e))?;
            instances.push(InstanceEntry::new(name, bind));
        }
    }

    let multi_cfg = MultiInstanceConfig::new(instances);

    // Build the clone factory. This requires feature="fixture-gen" — the
    // #[cfg(not(feature="fixture-gen"))] arm panics (GAP-1 enforcement).
    let factory = build_multi_clone_factory(cfg);

    // Start all instances and return the lifecycle handle.
    crate::multi_instance::start_instances(multi_cfg, factory)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start multi-org clone instances: {:?}", e))
}
