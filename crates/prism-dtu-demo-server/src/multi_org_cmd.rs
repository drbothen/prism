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
use std::path::Path;

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
        // SAFETY: parse_org_id returns Err only for invalid UUIDs. This is a true
        //         programming-error guard: MultiOrgDemoConfig::from_str validates all
        //         org_id fields as UUIDs before returning Ok, so any non-UUID value is
        //         rejected at config parse time (MED-B fix in config.rs). If this expect
        //         ever fires, it means a MultiOrgDemoConfig was constructed without going
        //         through from_str / from_file (i.e., a test or code path that bypasses
        //         the parse boundary — a programming error, not an operator error).
        #[allow(clippy::expect_used)]
        let org_id = crate::harness::parse_org_id(&org_cfg.org_id, entry_name)
            .expect("org_id in OrgConfig must be a valid UUID (validated at config parse time by MultiOrgDemoConfig::from_str)");

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

/// Write the nested `{org_slug: {sensor_id: url}}` sidecar to a caller-specified path.
///
/// This is the testable, path-parameterised variant of `write_multi_url_sidecar` in
/// `main.rs`. The binary's `write_multi_url_sidecar` delegates to this function with
/// `path = URL_MULTI_FILE`.
///
/// # Production-grade: no silent drops (MED-2 fix)
///
/// Previous implementation used `filter_map`, which silently dropped any sensor in the
/// config whose `{org_slug}-{sensor_id}` key was absent from `servers.socket_map()`.
/// A dropped entry means demo-run.sh would not generate an overlay TOML for that sensor,
/// causing prism boot failure for affected org×sensor queries — a production defect.
///
/// This function instead returns `Err` with an actionable message when ANY expected
/// `{org_slug}-{sensor_id}` key is missing from `socket_map`. This matches the
/// production-grade default: no silent partial-failure propagation
/// (CLAUDE.md Standing Rule 3 §2).
///
/// # Atomic write
///
/// The sidecar is written atomically (tmp + rename) to prevent demo-run.sh from reading
/// a partial file during the poll loop (GAP-3 sidecar-availability guarantee).
pub fn write_multi_url_sidecar_to_path(
    servers: &MultiInstanceServers,
    cfg: &MultiOrgDemoConfig,
    path: &Path,
) -> anyhow::Result<()> {
    use std::collections::HashMap;

    let socket_map = servers.socket_map();

    // Build nested map: {org_slug → {sensor_id → url}}.
    // Errors LOUDLY if any expected {org_slug}-{sensor_id} entry is missing from socket_map.
    let mut nested: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (org_slug, org_cfg) in &cfg.orgs {
        let mut sensor_urls: HashMap<String, String> = HashMap::new();
        for sensor_id in &org_cfg.sensors {
            let entry_name = format!("{org_slug}-{sensor_id}");
            let addr = socket_map.get(&entry_name).ok_or_else(|| {
                anyhow::anyhow!(
                    "write_multi_url_sidecar: socket_map is missing expected entry '{}'. \
                     This is a programming error — all sensors declared in MultiOrgDemoConfig \
                     must have been started by start_instances before writing the sidecar. \
                     Available socket_map keys: {:?}",
                    entry_name,
                    socket_map.keys().collect::<Vec<_>>()
                )
            })?;
            sensor_urls.insert(sensor_id.clone(), format!("http://{addr}"));
        }
        nested.insert(org_slug.clone(), sensor_urls);
    }

    let json = serde_json::to_string(&nested)
        .map_err(|e| anyhow::anyhow!("Failed to serialise nested URL map: {}", e))?;

    // Atomic write: tmp file + rename.
    let tmp_path = {
        let mut p = path.to_path_buf();
        let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("sidecar");
        p.set_file_name(format!("{fname}.tmp"));
        p
    };
    std::fs::write(&tmp_path, &json).map_err(|e| {
        anyhow::anyhow!(
            "Failed to write nested URL sidecar tmp {:?}: {}",
            tmp_path,
            e
        )
    })?;
    std::fs::rename(&tmp_path, path)
        .map_err(|e| anyhow::anyhow!("Failed to rename nested URL sidecar {:?}: {}", path, e))?;

    Ok(())
}

/// Resolve the `/dtu/configure` base URL for a clone, reading from whichever sidecar exists.
///
/// # Lookup logic
///
/// 1. If `flat_sidecar_path` is `Some` and exists: parse as `HashMap<String, String>` and
///    look up `clone_name` directly (flat format written by `start`).
/// 2. Else if `nested_sidecar_path` is `Some` and exists: parse as
///    `HashMap<String, HashMap<String, String>>` (nested format written by `start-multi`).
///    - First try `clone_name` as a literal key in the outer map or as `{org_slug}-{sensor_id}`.
///    - If not found as a literal key, try `clone_name` as a bare sensor_id:
///      scan all org entries for a sensor with that name (EC-007 documented recovery form).
///      If exactly one match exists, use it. If multiple orgs have the same sensor, return Err
///      (ambiguous — caller must use the full `{org_slug}-{sensor_id}` key form).
/// 3. Otherwise: return Err explaining which sidecars were checked.
///
/// # HIGH-1 fix
///
/// Before this function existed, `cmd_configure` only read the flat `URL_FILE`. After
/// `start-multi`, only `URL_MULTI_FILE` (nested) exists, so `configure cyberint <json>`
/// failed with "URL sidecar not found". This function implements the detection/resolution
/// logic to make the documented EC-007 recovery path actually work.
///
/// # Parameters
///
/// - `clone_name`: the name argument to `configure` — either a full `{org_slug}-{sensor_id}`
///   key (e.g. `"org-b-cyberint"`) or a bare sensor name (e.g. `"cyberint"`).
/// - `flat_sidecar_path`: path to the flat sidecar file (written by `start`), if known.
/// - `nested_sidecar_path`: path to the nested sidecar file (written by `start-multi`), if known.
pub fn resolve_configure_url(
    clone_name: &str,
    flat_sidecar_path: Option<&Path>,
    nested_sidecar_path: Option<&Path>,
) -> anyhow::Result<String> {
    use std::collections::HashMap;

    // --- 1. Try flat sidecar first (written by `start`) ---
    if let Some(flat_path) = flat_sidecar_path {
        if flat_path.exists() {
            let sidecar_str = std::fs::read_to_string(flat_path).map_err(|e| {
                anyhow::anyhow!("Failed to read flat URL sidecar {:?}: {}", flat_path, e)
            })?;
            let url_map: HashMap<String, String> = serde_json::from_str(&sidecar_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse flat URL sidecar: {}", e))?;
            if let Some(url) = url_map.get(clone_name) {
                return Ok(format!("{url}/dtu/configure"));
            }
            anyhow::bail!(
                "Clone '{}' not found in flat sidecar '{}'. Available: {:?}",
                clone_name,
                flat_path.display(),
                url_map.keys().collect::<Vec<_>>()
            );
        }
    }

    // --- 2. Try nested sidecar (written by `start-multi`) ---
    if let Some(nested_path) = nested_sidecar_path {
        if nested_path.exists() {
            let sidecar_str = std::fs::read_to_string(nested_path).map_err(|e| {
                anyhow::anyhow!("Failed to read nested URL sidecar {:?}: {}", nested_path, e)
            })?;
            let nested: HashMap<String, HashMap<String, String>> =
                serde_json::from_str(&sidecar_str)
                    .map_err(|e| anyhow::anyhow!("Failed to parse nested URL sidecar: {}", e))?;

            // First: try clone_name as a literal {org_slug}-{sensor_id} key.
            // The entry format is nested so we must check if clone_name splits into
            // a known {org_slug}-{sensor_id} pair present in the outer map.
            //
            // Strategy: scan all (org_slug, sensor_map) pairs. For each org, check
            // if sensor_map contains a key K such that "{org_slug}-{K}" == clone_name.
            let mut exact_match: Option<String> = None;
            for (org_slug, sensor_map) in &nested {
                for (sensor_id, url) in sensor_map {
                    let full_key = format!("{org_slug}-{sensor_id}");
                    if full_key == clone_name {
                        exact_match = Some(url.clone());
                        break;
                    }
                }
                if exact_match.is_some() {
                    break;
                }
            }
            if let Some(url) = exact_match {
                return Ok(format!("{url}/dtu/configure"));
            }

            // Second: try clone_name as a bare sensor_id (EC-007 recovery form).
            // Scan all org entries for a sensor named clone_name.
            let mut bare_matches: Vec<(String, String)> = Vec::new(); // (org_slug, url)
            for (org_slug, sensor_map) in &nested {
                if let Some(url) = sensor_map.get(clone_name) {
                    bare_matches.push((org_slug.clone(), url.clone()));
                }
            }
            match bare_matches.len() {
                0 => {
                    // Not found by any lookup strategy.
                    let all_keys: Vec<String> = nested
                        .iter()
                        .flat_map(|(org, sensors)| {
                            sensors
                                .keys()
                                .map(|s| format!("{org}-{s}"))
                                .collect::<Vec<_>>()
                        })
                        .collect();
                    anyhow::bail!(
                        "Clone '{}' not found in nested sidecar '{}'. \
                         Use the full '{{org_slug}}-{{sensor_id}}' form or a bare sensor name \
                         that exists in exactly one org. \
                         Available full keys: {:?}",
                        clone_name,
                        nested_path.display(),
                        all_keys
                    );
                }
                1 => {
                    // Exactly one match — EC-007 bare-sensor recovery form works.
                    let (_, url) = bare_matches.remove(0);
                    return Ok(format!("{url}/dtu/configure"));
                }
                _ => {
                    // Ambiguous — multiple orgs have this sensor.
                    let org_list: Vec<String> =
                        bare_matches.iter().map(|(org, _)| org.clone()).collect();
                    anyhow::bail!(
                        "Bare sensor name '{}' is ambiguous — found in {} orgs: {:?}. \
                         Use the full '{{org_slug}}-{{sensor_id}}' form to disambiguate \
                         (e.g. '{}-{}').",
                        clone_name,
                        org_list.len(),
                        org_list,
                        org_list[0],
                        clone_name
                    );
                }
            }
        }
    }

    // --- 3. Neither sidecar found ---
    anyhow::bail!(
        "No URL sidecar found. Checked: flat='{}', nested='{}'. \
         Is the demo harness running? \
         Start with `prism-dtu-demo-server start --config ...` (writes '{}') \
         or `prism-dtu-demo-server start-multi --config ...` (writes '{}').",
        flat_sidecar_path
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<not provided>".to_string()),
        nested_sidecar_path
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<not provided>".to_string()),
        crate::URL_FILE,
        crate::URL_MULTI_FILE
    )
}
