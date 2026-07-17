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
/// convention) to the correct clone constructor.
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
/// # Scenario path (BLOCKER 3)
///
/// When `org_cfg.scenario.as_ref().map(|s| s.enabled)` is `true`, all sensors for
/// that org are constructed via `new_with_scenario` with a shared `Arc<IncidentTimeline>` +
/// `ScenarioEntityCatalog`. The scenario context is built ONCE per org BEFORE the closure
/// is returned (keyed by org_slug in `scenario_ctxs`), so that:
/// - The `Arc<IncidentTimeline>` is shared across all sensors in the org (they advance in sync).
/// - The factory closure is `Fn` (not `FnMut`) — no state mutation needed inside the closure.
///
/// Mirrors `build_clone_pairs` in harness.rs (the `start` path). The same `ScenarioConfig`
/// type and `build_default_incident_timeline` / `build_scenario_entity_catalog` helpers are
/// used. Differences vs `build_clone_pairs`:
/// - No E-DEMO-002/003/006 prescans (multi-org config has one scenario per org, not per-clone;
///   seed uniqueness is already guaranteed by distinct org seeds).
/// - `new_with_scenario` is called for ALL sensors in the org, not per-clone opt-in.
///
/// # Reuses harness.rs helpers (Architecture Compliance)
///
/// Uses:
/// - `crate::harness::parse_org_id(str, name)` → `OrgId`
/// - `crate::harness::fixture_set_to_archetype("default", name)` → `Archetype`
/// - `prism_dtu_common::demo_time_anchor()` for the time anchor
/// - `new_with_seed(seed, archetype, org_id)` seeded constructors (non-scenario path)
/// - `new_with_scenario(seed, archetype, org_id, timeline, time_anchor, catalog)` (scenario path)
///
/// # Cyberint composite path (GAP-2)
///
/// `CyberintClone::new_with_seed` does NOT set `initial_access_token`. To satisfy BOTH
/// seed-based data distinctness AND access-token auth, the factory uses the composite pattern:
/// 1. Call `new_with_seed` or `new_with_scenario` to produce the seeded clone.
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
    // ENRICH-3: enrichment clone imports for global instance dispatch.
    // Used in the KNOWN_ENRICHMENT_CLONES dispatch arm in the returned closure.
    use prism_dtu_nvd::NvdClone;
    use prism_dtu_threatintel::ThreatIntelClone;

    // Known sensor names in suffix-search order. The name convention is
    // "{org_slug}-{sensor_id}" where both parts may contain '-' themselves
    // (e.g. "org-a-crowdstrike"). We match by stripping known sensor suffixes.
    //
    // TD-VSDD-060 sibling-awareness: use the shared const from config.rs so that
    // sensor validation (MultiOrgDemoConfig::from_str) and dispatch (here) cannot drift.
    // Adding a new sensor updates KNOWN_SENSORS once; both sites pick it up automatically.
    use crate::config::KNOWN_SENSORS;

    // ---------------------------------------------------------------------------
    // BLOCKER 3: Pre-build per-org scenario contexts BEFORE returning the closure.
    //
    // Key: org_slug (String).
    // Value: (Arc<IncidentTimeline>, ScenarioEntityCatalog, time_anchor).
    //
    // Built here (not inside the closure) because:
    // 1. The closure is `Fn` (not `FnMut`), so no mutable interior state.
    // 2. The Arc<IncidentTimeline> must be SHARED across all sensors in the org
    //    so they advance in sync — building it once guarantees this.
    // 3. Mirrors harness.rs `scenario_ctx` which is also pre-built before
    //    the clone-construction loop.
    // ---------------------------------------------------------------------------
    use prism_dtu_common::{build_default_incident_timeline, build_scenario_entity_catalog};
    use std::collections::HashMap;

    // ScenarioCtx carries the archetype alongside the timeline, catalog, and time anchor.
    //
    // The archetype is derived from `sc.archetype` (the SCENARIO archetype, e.g. "compromised_endpoint"
    // → Archetype::CompromisedEndpoint), NOT from `fixture_set_to_archetype("default", ...)` which
    // returns HealthyOtEnvironment. Using the wrong archetype causes `generate_with_scenario_iocs` to
    // early-return with the un-stamped path (it only stamps ioc_value for CompromisedEndpoint).
    //
    // Mirrors harness.rs lines 484-504 (scenario archetype validation + conversion).
    type ScenarioCtx = (
        std::sync::Arc<prism_dtu_common::IncidentTimeline>,
        prism_dtu_common::ScenarioEntityCatalog,
        chrono::DateTime<chrono::Utc>,
        prism_dtu_common::Archetype,
    );

    let mut scenario_ctxs: HashMap<String, ScenarioCtx> = HashMap::new();

    for (org_slug, org_cfg) in &cfg.orgs {
        let scenario_enabled = org_cfg
            .scenario
            .as_ref()
            .map(|s| s.enabled)
            .unwrap_or(false);

        if !scenario_enabled {
            continue;
        }

        #[allow(clippy::expect_used)]
        let sc = org_cfg
            .scenario
            .as_ref()
            .expect("scenario field must be Some when scenario_enabled is true (checked above)");

        // Convert scenario archetype string to Archetype enum.
        // Mirrors harness.rs lines 484-504 — only "compromised_endpoint" and "healthy" are valid.
        // Unrecognized archetypes are a programming error: MultiOrgDemoConfig::from_str validates
        // the archetype string at config parse time via the same match arm.
        let scenario_archetype = match sc.archetype.as_str() {
            "compromised_endpoint" => prism_dtu_common::Archetype::CompromisedEndpoint,
            "healthy" => prism_dtu_common::Archetype::HealthyOtEnvironment,
            other => {
                panic!(
                    "start-multi: E-DEMO-003: org '{}': unrecognized scenario archetype '{}'; \
                     valid values: compromised_endpoint, healthy. \
                     This is a programming error — MultiOrgDemoConfig::from_str must validate \
                     archetype strings before this point.",
                    org_slug, other
                )
            }
        };

        // Parse org_id to OrgId ([u8; 16]) for catalog construction.
        // from_str / from_file validates UUID format at config parse time; expect() here
        // is a programming-error guard (same rationale as the existing closure below).
        #[allow(clippy::expect_used)]
        let org_id = crate::harness::parse_org_id(&org_cfg.org_id, org_slug)
            .expect("org_id in OrgConfig must be a valid UUID (validated at config parse time)");

        // Build ScenarioEntityCatalog from seed + org_id.
        let catalog = build_scenario_entity_catalog(org_cfg.seed, &org_id);

        // Derive scenario_start_secs: config value or current system time.
        let scenario_start_secs: i64 = sc
            .scenario_start_secs
            .unwrap_or_else(|| chrono::Utc::now().timestamp());

        // Build IncidentTimeline from catalog + start time + stage durations.
        let timeline = build_default_incident_timeline(
            catalog.clone(),
            scenario_start_secs,
            &sc.stage_duration_secs,
        );

        // Derive time_anchor from scenario_start_secs.
        // chrono::DateTime::from_timestamp returns None only for out-of-range values.
        // scenario_start_secs comes from config (bounded i64) so this expect is safe.
        #[allow(clippy::expect_used)]
        let time_anchor = chrono::DateTime::from_timestamp(scenario_start_secs, 0)
            .expect("scenario_start_secs is a valid epoch timestamp");

        scenario_ctxs.insert(
            org_slug.clone(),
            (
                std::sync::Arc::new(timeline),
                catalog,
                time_anchor,
                scenario_archetype,
            ),
        );
    }

    // Pre-build a per-org ScenarioEntityCatalog lookup for enrichment clone construction.
    // When any org has scenario.enabled=true, enrichment clones are seeded with that org's
    // catalog so IOCs/CVEs correlate with the incident. We prefer the scenario org (usually
    // "org-c") over a non-scenario org. When no scenario org exists, use new()/new()? instead.
    //
    // We collect all scenario catalogs; enrichment will use the first scenario one found
    // (or None if none are configured). This lookup is done once here, not inside the closure,
    // for Fn-compatibility.
    let scenario_catalog_for_enrichment: Option<prism_dtu_common::ScenarioEntityCatalog> = cfg
        .orgs
        .iter()
        .find(|(_, org_cfg)| {
            org_cfg
                .scenario
                .as_ref()
                .map(|s| s.enabled)
                .unwrap_or(false)
        })
        .map(|(org_slug, org_cfg)| {
            #[allow(clippy::expect_used)]
            let org_id = crate::harness::parse_org_id(&org_cfg.org_id, org_slug)
                .expect("org_id validated at config parse time");
            build_scenario_entity_catalog(org_cfg.seed, &org_id)
        });

    Box::new(move |entry: &InstanceEntry| -> Box<dyn BehavioralClone> {
        let entry_name = &entry.name;

        // ENRICH-3: check for global enrichment clone names BEFORE attempting org-suffix parsing.
        // Global enrichment entries ("threatintel", "nvd") have no org-prefix and would panic
        // in the KNOWN_SENSORS suffix-strip logic if not intercepted here.
        use crate::config::KNOWN_ENRICHMENT_CLONES;
        if KNOWN_ENRICHMENT_CLONES.contains(&entry_name.as_str()) {
            return match entry_name.as_str() {
                "threatintel" => {
                    // Use scenario catalog (from the scenario org) when available so IOCs
                    // correlate with the incident; otherwise use the default fixture registry.
                    if let Some(ref catalog) = scenario_catalog_for_enrichment {
                        Box::new(ThreatIntelClone::new_with_scenario(catalog))
                    } else {
                        Box::new(ThreatIntelClone::new())
                    }
                }
                "nvd" => {
                    // NvdClone::new() is fallible (loads fixtures/cves.json from embed).
                    // NvdClone::new_with_scenario is also fallible. Either way use expect()
                    // with an actionable message — fixture load failure is a build-time defect.
                    #[allow(clippy::expect_used)]
                    if let Some(ref catalog) = scenario_catalog_for_enrichment {
                        Box::new(
                            NvdClone::new_with_scenario(catalog)
                                .expect("NvdClone::new_with_scenario must succeed — fixtures/cves.json is embedded at build time"),
                        )
                    } else {
                        Box::new(
                            NvdClone::new()
                                .expect("NvdClone::new must succeed — fixtures/cves.json is embedded at build time"),
                        )
                    }
                }
                other => {
                    // EC-010: unknown global enrichment name — programming error.
                    panic!(
                        "start-multi: EC-010: KNOWN_ENRICHMENT_CLONES contains '{}' but no \
                         dispatch arm handles it. Update build_multi_clone_factory.",
                        other
                    )
                }
            };
        }

        // Parse (org_slug, sensor_id) from entry.name by matching known sensor suffixes.
        // Try each sensor as a suffix "-{sensor_id}"; first match wins.
        let (org_slug, sensor_id) = KNOWN_SENSORS
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

        // Derive base Archetype from "default" fixture_set for the NON-scenario seeded path.
        // SAFETY: "default" is a known-valid fixture_set; expect() is appropriate here.
        // NOTE: for the scenario path, scenario_archetype from ScenarioCtx is used instead —
        //       using fixture_set archetype (HealthyOtEnvironment) for new_with_scenario would
        //       cause generate_with_scenario_iocs to early-return without stamping ioc_value.
        #[allow(clippy::expect_used)]
        let base_archetype = crate::harness::fixture_set_to_archetype("default", entry_name)
            .expect("'default' is a valid fixture_set; this expect cannot fail");

        let seed = org_cfg.seed;

        // Check if this org has a pre-built scenario context.
        let scenario_ctx = scenario_ctxs.get(org_slug);

        // Dispatch to the correct constructor based on sensor_id (EC-008).
        // When scenario_ctx is Some: call new_with_scenario with the SCENARIO archetype (BLOCKER 3).
        // When scenario_ctx is None: call new_with_seed with the base fixture_set archetype.
        match sensor_id {
            "crowdstrike" => {
                if let Some((timeline_arc, catalog, time_anchor, scenario_archetype)) = scenario_ctx
                {
                    Box::new(CrowdstrikeClone::new_with_scenario(
                        seed,
                        *scenario_archetype,
                        org_id,
                        std::sync::Arc::clone(timeline_arc),
                        *time_anchor,
                        catalog,
                    ))
                } else {
                    // CrowdstrikeClone::new_with_seed is infallible.
                    Box::new(CrowdstrikeClone::new_with_seed(
                        seed,
                        base_archetype,
                        org_id,
                    ))
                }
            }
            "claroty" => {
                if let Some((timeline_arc, _, time_anchor, scenario_archetype)) = scenario_ctx {
                    Box::new(ClarotyClone::new_with_scenario(
                        seed,
                        *scenario_archetype,
                        org_id,
                        std::sync::Arc::clone(timeline_arc),
                        *time_anchor,
                    ))
                } else {
                    // ClarotyClone::new_with_seed is infallible.
                    Box::new(ClarotyClone::new_with_seed(seed, base_archetype, org_id))
                }
            }
            "armis" => {
                if let Some((timeline_arc, catalog, time_anchor, scenario_archetype)) = scenario_ctx
                {
                    #[allow(clippy::expect_used)]
                    Box::new(
                        ArmisClone::new_with_scenario(
                            seed,
                            *scenario_archetype,
                            org_id,
                            std::sync::Arc::clone(timeline_arc),
                            *time_anchor,
                            catalog,
                        )
                        .expect("ArmisClone::new_with_scenario must succeed for valid args"),
                    )
                } else {
                    // ArmisClone::new_with_seed is fallible (returns Result).
                    #[allow(clippy::expect_used)]
                    Box::new(ArmisClone::new_with_seed(seed, base_archetype, org_id).expect(
                        "ArmisClone::new_with_seed must succeed for valid seed/archetype/org_id",
                    ))
                }
            }
            "cyberint" => {
                // GAP-2 composite path (both scenario and seeded):
                //   1. new_with_scenario or new_with_seed → seeded clone (no access_token yet)
                //   2. if initial_access_token.is_some() → apply token synchronously via
                //      clone.state.apply_config (CyberintState::apply_config is sync).
                //
                // We call clone.state.apply_config BEFORE Box::new(clone) so that the
                // concrete CyberintClone type is still in scope. After boxing to
                // Box<dyn BehavioralClone>, only the async configure() trait method is
                // accessible — which would require block_on and panic on a tokio thread.
                #[allow(clippy::expect_used)]
                let clone = if let Some((timeline_arc, catalog, time_anchor, scenario_archetype)) =
                    scenario_ctx
                {
                    CyberintClone::new_with_scenario(
                        seed,
                        *scenario_archetype,
                        org_id,
                        std::sync::Arc::clone(timeline_arc),
                        *time_anchor,
                        catalog,
                    )
                    .expect("CyberintClone::new_with_scenario must succeed for valid args")
                } else {
                    CyberintClone::new_with_seed(seed, base_archetype, org_id).expect(
                        "CyberintClone::new_with_seed must succeed for valid seed/archetype/org_id",
                    )
                };

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

/// Build `MultiInstanceConfig` + start all org clone instances and any enabled global
/// enrichment DTU instances.
///
/// Extracted as a separately-testable async fn so RG-005 can verify socket isolation
/// without subprocess overhead. `cmd_start_multi` in `main.rs` delegates to this.
///
/// # Contract
///
/// - Reads `cfg.orgs` to build `MultiInstanceConfig` entries named `"{org_slug}-{sensor_id}"`.
/// - When `cfg.enrichment.threatintel = true`, appends an entry named `"threatintel"` (global).
/// - When `cfg.enrichment.nvd = true`, appends an entry named `"nvd"` (global).
/// - Calls `build_multi_clone_factory(cfg)` to produce the clone factory.
/// - Calls `start_instances(multi_cfg, clone_factory).await` from `crate::multi_instance`.
/// - Returns `Ok(MultiInstanceServers)` with a socket_map keyed by `"{org_slug}-{sensor_id}"`
///   for per-org sensors and `"threatintel"` / `"nvd"` for global enrichment instances.
///
/// # Entry name convention
///
/// Per-org entries: `"{org_slug}-{sensor_id}"` (e.g. `"org-a-crowdstrike"`).
/// Global enrichment entries: `"threatintel"`, `"nvd"` (stable names, no org prefix).
/// The `build_multi_clone_factory` closure uses `KNOWN_ENRICHMENT_CLONES` to detect global
/// names before attempting org-suffix parsing, so the two namespaces do not conflict.
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

    // Bind-address parser helper.
    let make_bind = |bind_ip: &str| -> anyhow::Result<std::net::SocketAddr> {
        let bind_str = format!("{bind_ip}:0");
        bind_str
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid bind address '{}': {}", bind_str, e))
    };

    // Build MultiInstanceConfig entries named "{org_slug}-{sensor_id}".
    // The iteration order of a HashMap is not deterministic, but the binding order
    // does not matter — all N entries must bind before this returns Ok.
    let mut instances = Vec::new();
    for (org_slug, org_cfg) in &cfg.orgs {
        for sensor_id in &org_cfg.sensors {
            let name = format!("{org_slug}-{sensor_id}");
            instances.push(InstanceEntry::new(name, make_bind(&cfg.harness.bind)?));
        }
    }

    // Append global enrichment instances (ENRICH-3).
    // These use stable names ("threatintel", "nvd") that are not org-prefixed.
    // The factory closure handles them via KNOWN_ENRICHMENT_CLONES before attempting
    // the org-suffix-parse path (which would panic on these names).
    if cfg.enrichment.threatintel {
        instances.push(InstanceEntry::new(
            "threatintel",
            make_bind(&cfg.harness.bind)?,
        ));
    }
    if cfg.enrichment.nvd {
        instances.push(InstanceEntry::new("nvd", make_bind(&cfg.harness.bind)?));
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
    use crate::config::KNOWN_ENRICHMENT_CLONES;
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

    // ENRICH-3: emit global enrichment DTU URLs under the reserved "_global" key.
    //
    // The "_global" key is NOT an org slug — it is a reserved top-level key that demo-run.sh
    // reads specifically to export PRISM_THREATINTEL_BASE_URL / PRISM_NVD_BASE_URL env vars
    // without generating per-org sensor overlay TOMLs. This keeps the sidecar format
    // backward-compatible: existing demo-run.sh code that iterates org slugs and skips
    // "_global" (which is not in cfg.orgs) is unaffected.
    //
    // Validation: if an enrichment clone is enabled in cfg.enrichment but its global
    // socket_map key is absent, fail LOUDLY — same production-grade principle as per-org sensors.
    let mut global_urls: HashMap<String, String> = HashMap::new();
    for &enrichment_name in KNOWN_ENRICHMENT_CLONES {
        // Determine whether this enrichment clone was requested in the config.
        let enabled = match enrichment_name {
            "threatintel" => cfg.enrichment.threatintel,
            "nvd" => cfg.enrichment.nvd,
            _ => false,
        };
        if enabled {
            let addr = socket_map.get(enrichment_name).ok_or_else(|| {
                anyhow::anyhow!(
                    "write_multi_url_sidecar: socket_map is missing expected enrichment entry \
                     '{}'. This is a programming error — enrichment clones enabled in \
                     EnrichmentConfig must have been started by start_instances before writing \
                     the sidecar. Available socket_map keys: {:?}",
                    enrichment_name,
                    socket_map.keys().collect::<Vec<_>>()
                )
            })?;
            global_urls.insert(enrichment_name.to_string(), format!("http://{addr}"));
        }
    }
    if !global_urls.is_empty() {
        nested.insert("_global".to_string(), global_urls);
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

/// Write the nested `{org_slug: {sensor_id: token}}` admin-token sidecar to a caller-specified path.
///
/// Mirror of `write_multi_url_sidecar_to_path` for admin tokens (tokens instead of URLs).
/// The binary's `cmd_start_multi` delegates to this function with `path = TOKEN_MULTI_FILE`.
///
/// # Format
///
/// `{org_slug: {sensor_id: token}}` — nested JSON object mirroring `URL_MULTI_FILE` format
/// but containing admin tokens instead of URLs. Each `sensor_id` key matches the corresponding
/// key in `URL_MULTI_FILE` for the same org.
///
/// # Production-grade: no silent drops
///
/// Returns `Err` with an actionable message when ANY expected `{org_slug}-{sensor_id}` key
/// is missing from `token_map`. A missing token means `cmd_configure` would receive HTTP 401
/// — a runtime defect that must be caught at write time, not at configure time.
///
/// # Atomic write
///
/// Written atomically (tmp + rename) to prevent `cmd_configure` from reading a partial file
/// during the poll loop (GAP-3 sidecar-availability guarantee).
///
/// # AD-017 credential safety
///
/// Token values MUST NOT appear in structured log fields. This function does not log
/// token values. All tokens are ephemeral UUID v4 strings generated at clone construction.
///
/// (DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 T-05)
pub fn write_multi_admin_token_sidecar_to_path(
    servers: &MultiInstanceServers,
    cfg: &MultiOrgDemoConfig,
    path: &Path,
) -> anyhow::Result<()> {
    use crate::config::KNOWN_ENRICHMENT_CLONES;
    use std::collections::HashMap;

    let token_map = servers.admin_token_map();

    // Build nested map: {org_slug → {sensor_id → token}}.
    // Errors LOUDLY if any expected {org_slug}-{sensor_id} entry is missing from token_map.
    let mut nested: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (org_slug, org_cfg) in &cfg.orgs {
        let mut sensor_tokens: HashMap<String, String> = HashMap::new();
        for sensor_id in &org_cfg.sensors {
            let entry_name = format!("{org_slug}-{sensor_id}");
            let token = token_map.get(&entry_name).ok_or_else(|| {
                anyhow::anyhow!(
                    "write_multi_admin_token_sidecar: token_map is missing expected entry '{}'. \
                     This is a programming error — all sensors declared in MultiOrgDemoConfig \
                     must have been started by start_instances before writing the sidecar. \
                     Available token_map keys: {:?}",
                    entry_name,
                    token_map.keys().collect::<Vec<_>>()
                )
            })?;
            sensor_tokens.insert(sensor_id.clone(), token.clone());
        }
        nested.insert(org_slug.clone(), sensor_tokens);
    }

    // Emit global enrichment DTU tokens under the reserved "_global" key.
    // Mirrors the `_global` key pattern from write_multi_url_sidecar_to_path (ENRICH-3).
    let mut global_tokens: HashMap<String, String> = HashMap::new();
    for &enrichment_name in KNOWN_ENRICHMENT_CLONES {
        let enabled = match enrichment_name {
            "threatintel" => cfg.enrichment.threatintel,
            "nvd" => cfg.enrichment.nvd,
            _ => false,
        };
        if enabled {
            let token = token_map.get(enrichment_name).ok_or_else(|| {
                anyhow::anyhow!(
                    "write_multi_admin_token_sidecar: token_map is missing expected enrichment \
                     entry '{}'. This is a programming error — enrichment clones enabled in \
                     EnrichmentConfig must have been started by start_instances before writing \
                     the sidecar. Available token_map keys: {:?}",
                    enrichment_name,
                    token_map.keys().collect::<Vec<_>>()
                )
            })?;
            global_tokens.insert(enrichment_name.to_string(), token.clone());
        }
    }
    if !global_tokens.is_empty() {
        nested.insert("_global".to_string(), global_tokens);
    }

    let json = serde_json::to_string(&nested)
        .map_err(|e| anyhow::anyhow!("Failed to serialise nested token map: {}", e))?;

    // Atomic write: tmp file + rename.
    let tmp_path = {
        let mut p = path.to_path_buf();
        let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("tokens");
        p.set_file_name(format!("{fname}.tmp"));
        p
    };
    // Write tmp file with 0600 permissions on Unix (AD-017 credential safety:
    // tokens are ephemeral but perms hardening is cheap consistency — F-ADMTOK-P1-OBS-002).
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp_path)
            .and_then(|mut f| f.write_all(json.as_bytes()))
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to write nested token sidecar tmp {:?}: {}",
                    tmp_path,
                    e
                )
            })?;
    }
    #[cfg(not(unix))]
    std::fs::write(&tmp_path, &json).map_err(|e| {
        anyhow::anyhow!(
            "Failed to write nested token sidecar tmp {:?}: {}",
            tmp_path,
            e
        )
    })?;
    std::fs::rename(&tmp_path, path)
        .map_err(|e| anyhow::anyhow!("Failed to rename nested token sidecar {:?}: {}", path, e))?;

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
            // Sort for deterministic diagnostic output.
            let mut available: Vec<_> = url_map.keys().collect();
            available.sort();
            anyhow::bail!(
                "Clone '{}' not found in flat sidecar '{}'. Available: {:?}",
                clone_name,
                flat_path.display(),
                available
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
            // Sort for deterministic error messages.
            bare_matches.sort_by(|a, b| a.0.cmp(&b.0));
            match bare_matches.len() {
                0 => {
                    // Not found by any lookup strategy.
                    let mut all_keys: Vec<String> = nested
                        .iter()
                        .flat_map(|(org, sensors)| {
                            sensors
                                .keys()
                                .map(|s| format!("{org}-{s}"))
                                .collect::<Vec<_>>()
                        })
                        .collect();
                    // Sort for deterministic diagnostic output.
                    all_keys.sort();
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

/// Resolve the admin token for a clone, reading from whichever token sidecar exists.
///
/// # E-DEMO-007 error contract
///
/// Returns `Err` with message matching the E-DEMO-007 template:
/// `"configure: E-DEMO-007: admin token for clone '{clone_name}' could not be resolved: {reason}"`
///
/// Error reasons:
/// - `"token sidecar not found (start the demo server first with start or start-multi)"` —
///   neither `flat_sidecar_path` nor `nested_sidecar_path` exists on disk (EC-004).
/// - `"clone '{clone_name}' not found in token sidecar '{path}'"` — sidecar exists but
///   the clone name is absent (EC-003).
/// - `"Bare sensor name '{name}' is ambiguous — found in N orgs: ["org-a", "org-b"]. Use full \
///   '{org_slug}-{sensor_id}' form."` — EC-005: multiple orgs have the same bare sensor name.
///
/// # Lookup logic
///
/// Mirrors `resolve_configure_url` for the token sidecar:
/// 1. If `flat_sidecar_path` is `Some` and exists: look up `clone_name` directly (flat format).
/// 2. Else if `nested_sidecar_path` is `Some` and exists: look up as literal
///    `{org_slug}-{sensor_id}` key or as a bare sensor name (EC-007 recovery form).
///    Bare names that match exactly one org return that org's token; bare names matching
///    multiple orgs return E-DEMO-007 (EC-005 ambiguity).
/// 3. Otherwise: E-DEMO-007 with "token sidecar not found" reason (EC-004).
///
/// # Parameters
///
/// - `clone_name`: the name argument to `configure` — full `{org_slug}-{sensor_id}` key
///   or a bare sensor name.
/// - `flat_sidecar_path`: path to the flat token sidecar written by `start`, if known.
/// - `nested_sidecar_path`: path to the nested token sidecar written by `start-multi`, if known.
///
/// (DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 T-07 / BC-3.6.001 Precondition 4)
pub fn resolve_configure_token(
    clone_name: &str,
    flat_sidecar_path: Option<&Path>,
    nested_sidecar_path: Option<&Path>,
) -> anyhow::Result<String> {
    use std::collections::HashMap;

    // Helper to produce the E-DEMO-007 error with the canonical message template.
    let e_demo_007 = |reason: &str| -> anyhow::Error {
        anyhow::anyhow!(
            "configure: E-DEMO-007: admin token for clone '{}' could not be resolved: {}",
            clone_name,
            reason
        )
    };

    // --- 1. Try flat sidecar first (written by `start`) ---
    if let Some(flat_path) = flat_sidecar_path {
        if flat_path.exists() {
            let sidecar_str = std::fs::read_to_string(flat_path).map_err(|e| {
                e_demo_007(&format!(
                    "failed to read token sidecar {:?}: {}",
                    flat_path, e
                ))
            })?;
            let token_map: HashMap<String, String> = serde_json::from_str(&sidecar_str)
                .map_err(|e| e_demo_007(&format!("failed to parse token sidecar: {}", e)))?;
            if let Some(token) = token_map.get(clone_name) {
                return Ok(token.clone());
            }
            return Err(e_demo_007(&format!(
                "clone '{}' not found in token sidecar '{}'",
                clone_name,
                flat_path.display()
            )));
        }
    }

    // --- 2. Try nested sidecar (written by `start-multi`) ---
    if let Some(nested_path) = nested_sidecar_path {
        if nested_path.exists() {
            let sidecar_str = std::fs::read_to_string(nested_path).map_err(|e| {
                e_demo_007(&format!(
                    "failed to read token sidecar {:?}: {}",
                    nested_path, e
                ))
            })?;
            let nested: HashMap<String, HashMap<String, String>> =
                serde_json::from_str(&sidecar_str).map_err(|e| {
                    e_demo_007(&format!("failed to parse nested token sidecar: {}", e))
                })?;

            // First: try clone_name as a literal {org_slug}-{sensor_id} key.
            let mut exact_match: Option<String> = None;
            for (org_slug, sensor_map) in &nested {
                for (sensor_id, token) in sensor_map {
                    let full_key = format!("{org_slug}-{sensor_id}");
                    if full_key == clone_name {
                        exact_match = Some(token.clone());
                        break;
                    }
                }
                if exact_match.is_some() {
                    break;
                }
            }
            if let Some(token) = exact_match {
                return Ok(token);
            }

            // Second: try clone_name as a bare sensor_id (EC-007 recovery form).
            let mut bare_matches: Vec<(String, String)> = Vec::new(); // (org_slug, token)
            for (org_slug, sensor_map) in &nested {
                if let Some(token) = sensor_map.get(clone_name) {
                    bare_matches.push((org_slug.clone(), token.clone()));
                }
            }
            // Sort for deterministic error messages.
            bare_matches.sort_by(|a, b| a.0.cmp(&b.0));

            match bare_matches.len() {
                0 => {
                    return Err(e_demo_007(&format!(
                        "clone '{}' not found in token sidecar '{}'",
                        clone_name,
                        nested_path.display()
                    )));
                }
                1 => {
                    let (_, token) = bare_matches.remove(0);
                    return Ok(token);
                }
                _ => {
                    // EC-005: Ambiguous bare sensor name — multiple orgs have this sensor.
                    let org_list: Vec<String> =
                        bare_matches.iter().map(|(org, _)| org.clone()).collect();
                    return Err(e_demo_007(&format!(
                        "Bare sensor name '{}' is ambiguous — found in {} orgs: {:?}. \
                         Use full '{{org_slug}}-{{sensor_id}}' form.",
                        clone_name,
                        org_list.len(),
                        org_list
                    )));
                }
            }
        }
    }

    // --- 3. Neither sidecar found (EC-004) ---
    Err(e_demo_007(
        "token sidecar not found (start the demo server first with start or start-multi)",
    ))
}
