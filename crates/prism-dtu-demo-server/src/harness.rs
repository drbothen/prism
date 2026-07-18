//! `DemoHarness` — multi-clone boot, supervisor tasks, and URL table.
//!
//! Owns clone objects by-value in a `Vec<ClonePair>`. Provides:
//! - `start_all()`: starts all enabled clones, populates `StartReport`
//! - `stop_all()`: sends graceful shutdown signal, hard-aborts after 5s
//! - `print_url_table()`: prints the URL table to stdout
//! - `last_start_report()`: returns `&StartReport` for the most recent `start_all()`
//!
//! # Ownership Model (ADR-002 Amendment §H1)
//!
//! The harness owns clone objects by-value in `Vec<ClonePair>`. There is no `Mutex`
//! wrapping — each clone is accessed sequentially during startup (no concurrent `&mut`
//! borrows). This satisfies the workspace `await_holding_lock = "deny"` lint.
//!
//! # Shutdown (ADR-002 Amendment §H2)
//!
//! Graceful drain via `shutdown_tx` broadcast; hard-abort via `JoinHandle::abort()`
//! inside `clone.stop()` after a 250ms timeout (S-PERF-GATE-005).
//! The harness supervisor join at `stop_all()` uses a separate 5s window to
//! accommodate multiple clones draining sequentially in CI.

use std::{collections::HashMap, net::SocketAddr};

use anyhow::Context;
use prism_dtu_common::BehavioralClone;
use tokio::task::JoinHandle;

use crate::config::{CloneConfig, DemoConfig};

/// A clone name + instance + bound address, held by value in the harness.
pub struct ClonePair {
    /// Human-readable clone name (e.g. `"crowdstrike"`).
    pub name: String,
    /// The clone instance owned by this pair.
    pub clone: Box<dyn BehavioralClone>,
    /// Set after `start_on()` returns successfully; `None` if not yet started.
    pub bound_addr: Option<SocketAddr>,
    /// When `true`: a bind failure logs WARN and the harness skips this clone.
    /// When `false` (default): a bind failure aborts startup (AC-11 cleanup path).
    pub continue_on_error: bool,
}

impl ClonePair {
    /// Construct a new `ClonePair` from a named clone instance.
    pub fn new(name: impl Into<String>, clone: Box<dyn BehavioralClone>) -> Self {
        Self {
            name: name.into(),
            clone,
            bound_addr: None,
            continue_on_error: false,
        }
    }

    /// Return the admin shared-secret token for `/dtu/configure` on this clone.
    ///
    /// Delegates to `BehavioralClone::admin_token()`. The demo-server test suite
    /// uses this to include the `X-Admin-Token` header in configure requests.
    pub fn admin_token(&self) -> &str {
        self.clone.admin_token()
    }
}

/// Describes the outcome of the most recent `DemoHarness::start_all()` call.
///
/// Exactly one of the following conditions holds after `start_all()` returns:
///
/// - **All success**: `successfully_started.len() == 6`, all other vecs/fields empty/None.
/// - **Abort** (`continue_on_error=false`, one clone failed): `failed_at.is_some()`,
///   `cleaned_up_after_failure` has the rolled-back clones, `skipped_due_to_error.is_empty()`.
/// - **Partial success** (`continue_on_error=true`, ≥1 clone failed): `skipped_due_to_error`
///   has the failures, `successfully_started` has the survivors, `failed_at.is_none()`,
///   `cleaned_up_after_failure.is_empty()` (no rollback in continue mode).
///
/// Used by tests (AC-11, AC-12, AC-13) to observe partial-startup cleanup behavior.
#[derive(Debug, Default)]
pub struct StartReport {
    /// Names of clones that bound successfully and are now serving.
    pub successfully_started: Vec<String>,
    /// Names of clones that were started and then stopped during partial-startup cleanup
    /// (abort path only — `continue_on_error=false`).
    pub cleaned_up_after_failure: Vec<String>,
    /// Set when `continue_on_error=false` and a clone failed — the harness aborted and
    /// rolled back `cleaned_up_after_failure`. Always `None` under `continue_on_error=true`.
    pub failed_at: Option<(String, std::io::Error)>,
    /// Clones that failed to bind and were skipped (only under `continue_on_error=true`).
    /// Always empty under `continue_on_error=false`.
    pub skipped_due_to_error: Vec<(String, std::io::Error)>,
}

/// Multi-clone demo harness.
///
/// Manages the lifecycle of all enabled DTU clone instances:
/// start, supervise, shutdown.
pub struct DemoHarness {
    /// Clone pairs owned by value; indexed by clone position.
    pub pairs: Vec<ClonePair>,
    /// Supervisor task handles; parallel index to `pairs`.
    tasks: Vec<JoinHandle<()>>,
    /// Broadcast sender for the graceful-shutdown signal.
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
    /// Populated by `start_all()`; describes the most recent startup outcome.
    last_start_report: StartReport,
}

impl DemoHarness {
    /// Create a new harness from a list of clone pairs.
    pub fn new(pairs: Vec<ClonePair>) -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        Self {
            pairs,
            tasks: Vec::new(),
            shutdown_tx,
            last_start_report: StartReport::default(),
        }
    }

    /// Start all enabled clone pairs.
    ///
    /// Accepts an optional `RustlsConfig` to propagate TLS to every clone's
    /// `start_on` call.  When `None`, clones bind via plain HTTP (backward-compatible).
    ///
    /// # ADR-002 Amendment #2 (TD-WV1-04)
    ///
    /// The second parameter `tls` is `Option<Arc<RustlsConfig>>` under the `tls`
    /// feature, and `Option<()>` otherwise — matching the `BehavioralClone::start_on`
    /// signature.
    ///
    /// On success all pairs are bound and serving. On failure (when `continue_on_error=false`),
    /// the already-started clones are stopped and `Err` is returned.
    pub async fn start_all(
        &mut self,
        config: &DemoConfig,
        #[cfg(feature = "tls")] tls: Option<std::sync::Arc<axum_server::tls_rustls::RustlsConfig>>,
        #[cfg(not(feature = "tls"))] tls: Option<()>,
    ) -> anyhow::Result<()> {
        // Reset report for this run.
        self.last_start_report = StartReport::default();

        // Validate bind security before starting any clone (AC-9).
        validate_bind_security(config)?;

        // Build ordered list of (pair_index, clone_config) for enabled clones.
        let enabled_indices: Vec<(usize, &CloneConfig, bool)> = self
            .pairs
            .iter()
            .enumerate()
            .map(|(i, pair)| {
                let cfg = clone_config_by_name(config, &pair.name);
                let coe = pair.continue_on_error;
                (i, cfg, coe)
            })
            .collect();

        for (pair_idx, clone_cfg, continue_on_error) in &enabled_indices {
            let bind_addr = clone_bind_addr(clone_cfg)?;
            let shutdown_rx = self.shutdown_tx.subscribe();

            let pair = &mut self.pairs[*pair_idx];

            // Clone the TLS config for this clone's start_on call.
            #[cfg(feature = "tls")]
            let tls_for_clone = tls.clone();
            #[cfg(not(feature = "tls"))]
            let tls_for_clone: Option<()> = tls;

            match pair
                .clone
                .start_on(bind_addr, Some(shutdown_rx), tls_for_clone)
                .await
            {
                Ok(bound) => {
                    pair.bound_addr = Some(bound);
                    self.last_start_report
                        .successfully_started
                        .push(pair.name.clone());

                    // Spawn a no-op supervisor task (the server runs inside the clone's JoinHandle).
                    // We track a dummy task here to keep tasks parallel to pairs.
                    let task_handle = tokio::spawn(async {});
                    self.tasks.push(task_handle);
                }
                Err(e) => {
                    // Convert to std::io::Error if possible (for AddrInUse kind).
                    let io_err = to_io_error(e);

                    if *continue_on_error {
                        // Log warning and skip this clone.
                        tracing::warn!(
                            "[WARN] {} failed to start: {} — continuing per continue_on_error=true",
                            pair.name,
                            io_err
                        );
                        self.last_start_report
                            .skipped_due_to_error
                            .push((pair.name.clone(), io_err));
                        // Push dummy handle to keep indices in sync.
                        self.tasks.push(tokio::spawn(async {}));
                    } else {
                        // Abort path: stop the N already-started clones.
                        let failed_name = pair.name.clone();
                        self.last_start_report.failed_at = Some((failed_name.clone(), io_err));

                        // Stop previously started clones in reverse order (or forward — spec says in order).
                        // The report must list them as cleaned_up_after_failure.
                        // We need to stop pairs[0..started_count].
                        let already_started: Vec<String> = self
                            .last_start_report
                            .successfully_started
                            .drain(..)
                            .collect();

                        for name in &already_started {
                            // Find the pair by name and stop it.
                            if let Some(p) = self.pairs.iter_mut().find(|p| &p.name == name) {
                                let _ = p.clone.stop().await;
                            }
                        }

                        // Record cleanup.
                        self.last_start_report.cleaned_up_after_failure = already_started;

                        return Err(anyhow::anyhow!(
                            "clone '{}' failed to start: abort triggered (R-DEMO-001 cleanup)",
                            failed_name
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Stop all running clones.
    ///
    /// Sends the graceful-shutdown broadcast. Waits up to 5 seconds for all tasks to
    /// complete. Any task that has not completed is hard-aborted via `clone.stop()`.
    /// Calls `clone.reset()` on every pair regardless of drain outcome.
    pub async fn stop_all(&mut self) {
        // Send shutdown signal (ignore "no receivers" error).
        let _ = self.shutdown_tx.send(());

        // Wait up to 5 seconds for all supervisor tasks to complete.
        let graceful_timeout = std::time::Duration::from_secs(5);

        let result = tokio::time::timeout(graceful_timeout, async {
            for task in &mut self.tasks {
                let _ = task.await;
            }
        })
        .await;

        if result.is_err() {
            // Timeout elapsed — hard-abort any remaining clones.
            for pair in &mut self.pairs {
                if pair.bound_addr.is_some() {
                    let _ = pair.clone.stop().await;
                }
            }
        }

        // Reset every pair regardless of path taken.
        for pair in &mut self.pairs {
            let _ = pair.clone.reset().await;
        }

        // Clear tasks.
        self.tasks.clear();
    }

    /// Return the `StartReport` for the most recent `start_all()` call.
    pub fn last_start_report(&self) -> &StartReport {
        &self.last_start_report
    }

    /// Return a map from clone name to bound URL string.
    ///
    /// Uses each clone's `base_url()` which returns `https://` when TLS is active,
    /// or `http://` otherwise.
    ///
    /// Only includes clones with a bound address (i.e., successfully started).
    /// Used by the binary to write the URL sidecar file for the `configure` subcommand.
    pub fn url_map(&self) -> HashMap<String, String> {
        self.pairs
            .iter()
            .filter_map(|pair| {
                if pair.bound_addr.is_some() {
                    Some((pair.name.clone(), pair.clone.base_url()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Return a map from clone name to admin token string.
    ///
    /// Only includes clones with a bound address (i.e., successfully started).
    /// Used by `write_token_sidecar_to_path` to write `TOKEN_FILE` so that
    /// `cmd_configure` can obtain per-clone tokens for the `X-Admin-Token` header
    /// (ADR-003 Amendment #5 / DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 T-03).
    pub fn token_map(&self) -> HashMap<String, String> {
        self.pairs
            .iter()
            .filter_map(|pair| {
                if pair.bound_addr.is_some() {
                    Some((pair.name.clone(), pair.admin_token().to_string()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Print the URL table to stdout.
    ///
    /// Only lists clones with a bound address (i.e., successfully started).
    /// Uses each clone's `base_url()` so HTTPS URLs are shown when TLS is active.
    pub fn print_url_table(&self) {
        println!(
            "| {:<13} | {:<25} | {:<7} | {:<7} |",
            "Clone", "URL", "Fixture", "Failure"
        );
        println!("|{:-<15}|{:-<27}|{:-<9}|{:-<9}|", "", "", "", "");
        for pair in &self.pairs {
            if pair.bound_addr.is_some() {
                println!(
                    "| {:<13} | {:<25} | {:<7} | {:<7} |",
                    pair.name,
                    pair.clone.base_url(),
                    "default",
                    "none"
                );
            }
        }
    }
}

/// Write the flat admin-token sidecar `{name: token}` atomically (tmp+rename).
///
/// Called from `cmd_start` in `main.rs` immediately after `write_url_sidecar`, so that
/// `cmd_configure` (a separate process invocation) can read per-clone admin tokens for
/// the `X-Admin-Token` header required by ADR-003 Amendment #5.
///
/// Written to `path` (typically `TOKEN_FILE = ".prism-dtu-demo-server.admin-tokens.json"`).
/// Path-parameterised so that tests can target per-test temp paths without touching
/// the shared crate-CWD `TOKEN_FILE` constant.
///
/// # Atomic write (GAP-3 sidecar-availability guarantee)
///
/// Writes to a `.tmp` file first, then renames atomically, so that `cmd_configure`
/// (polling in a separate process) never reads a partial file.
///
/// # AD-017 credential safety
///
/// Token values MUST NOT appear in structured log fields. This function does not log
/// token values. All tokens are ephemeral UUID v4 strings generated at clone construction.
/// The tmp file is created with mode 0600 on Unix to prevent world-readable token files.
///
/// (DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 T-03 / F-ADMTOK-P1-OBS-002)
pub fn write_token_sidecar_to_path(
    token_map: &std::collections::HashMap<String, String>,
    path: &std::path::Path,
) -> anyhow::Result<()> {
    let json = serde_json::to_string(token_map)
        .map_err(|e| anyhow::anyhow!("Failed to serialise token map: {}", e))?;
    let tmp_path = {
        let fname = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("tokens");
        path.with_file_name(format!("{fname}.tmp"))
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
                anyhow::anyhow!("Failed to write token sidecar tmp {:?}: {}", tmp_path, e)
            })?;
    }
    #[cfg(not(unix))]
    std::fs::write(&tmp_path, &json)
        .map_err(|e| anyhow::anyhow!("Failed to write token sidecar tmp {:?}: {}", tmp_path, e))?;
    std::fs::rename(&tmp_path, path)
        .map_err(|e| anyhow::anyhow!("Failed to rename token sidecar {:?}: {}", path, e))?;
    Ok(())
}

/// Build all clone pairs from a `DemoConfig`.
///
/// Handles both infallible constructors (crowdstrike, claroty, threatintel) and fallible
/// constructors (cyberint, armis, nvd) by propagating errors with `?`.
///
/// # Story A — seed-forwarding (BC-2.06.018 / ADR-036 §2.4)
///
/// When `fixture-gen` is active AND a generator-backed clone (crowdstrike, armis,
/// claroty, cyberint) has a non-default archetype OR has `org_id` set, this function:
///
/// 1. Validates `fixture_set` → `Archetype` via `fixture_set_to_archetype` (E-DEMO-001).
/// 2. If the archetype is non-HealthyOtEnvironment AND `org_id` is None → E-DEMO-004.
/// 3. If `org_id` is Some, parses it as UUID → E-DEMO-005 on failure.
/// 4. Calls `new_with_seed(seed, archetype, org_id)` on the clone (ADR-036 v2.2).
///
/// When `fixture-gen` is NOT active, or `org_id` is None AND `fixture_set = "default"`,
/// falls back to the static-JSON `new()` path (backward-compatible, ADR-036 §2.5).
pub fn build_clone_pairs(config: &DemoConfig) -> anyhow::Result<Vec<ClonePair>> {
    use prism_dtu_armis::ArmisClone;
    use prism_dtu_claroty::ClarotyClone;
    use prism_dtu_crowdstrike::CrowdstrikeClone;
    use prism_dtu_cyberint::CyberintClone;
    use prism_dtu_nvd::NvdClone;
    use prism_dtu_threatintel::ThreatIntelClone;

    // ---------------------------------------------------------------------------
    // B-P1-03 + B-P4-01 fix: Architecture Compliance Rules (story spec AC-017):
    // guard order is E-DEMO-002 → E-DEMO-006 → E-DEMO-003 → E-DEMO-004, all before any constructor.
    //
    // This block (E-DEMO-002 prescan) runs first, before the E-DEMO-003 prescan
    // and before `validated_gen` (which checks E-DEMO-004), so that seed-mismatch
    // is detected first even when org_id and/or scenario archetype are also invalid.
    // ---------------------------------------------------------------------------
    #[cfg(feature = "fixture-gen")]
    {
        let all_gen_clones_prescan: [(&'static str, &CloneConfig); 4] = [
            ("crowdstrike", &config.clones.crowdstrike),
            ("armis", &config.clones.armis),
            ("claroty", &config.clones.claroty),
            ("cyberint", &config.clones.cyberint),
        ];
        let scenario_seeds: Vec<(&'static str, u64)> = all_gen_clones_prescan
            .iter()
            .filter_map(|(name, cfg)| {
                if !cfg.enabled {
                    return None;
                }
                let sc = cfg.scenario.as_ref()?;
                if !sc.enabled {
                    return None;
                }
                Some((*name, cfg.seed))
            })
            .collect();
        if scenario_seeds.len() >= 2 {
            let (first_name, first_seed) = scenario_seeds[0];
            for (name, seed) in &scenario_seeds[1..] {
                if *seed != first_seed {
                    anyhow::bail!(
                        "demo-server: E-DEMO-002: scenario clones '{}' (seed={}) and '{}' \
                         (seed={}) have different seeds; cross-DTU coherence requires all \
                         scenario-enabled clones to share the same seed",
                        first_name,
                        first_seed,
                        name,
                        seed
                    );
                }
            }
        }
    }

    // ---------------------------------------------------------------------------
    // BC-2.06.019 PRE-6: E-DEMO-006 (org_id mismatch across scenario-enabled clones)
    // prescan runs AFTER E-DEMO-002 (seed mismatch) and BEFORE E-DEMO-003 (bad
    // archetype). Canonical guard order: E-DEMO-002 → E-DEMO-006 → E-DEMO-003 → E-DEMO-004.
    //
    // Compare org_ids only for scenario-enabled clones where org_id is Some(_).
    // Clones with org_id=None are absent from the equality check — their missing
    // org_id will surface as E-DEMO-004 later (that is a distinct error class).
    // Rationale: the ScenarioEntityCatalog is derived from the first scenario-enabled
    // clone's (seed, org_id). A second clone with a different org_id generates device
    // IDs using dev-{slug_B}-{seed}-0, which cannot equal catalog.primary_device_id
    // derived from slug_A — causing INV-CROSS-DTU-ENTITY-COHERENCE-001 (BC-2.06.020)
    // cross-DTU join to return empty with no diagnostic (SOUL.md §4 silent partial-
    // failure). This guard prevents silent incoherence at construction time.
    // ---------------------------------------------------------------------------
    #[cfg(feature = "fixture-gen")]
    {
        let all_gen_clones_e006_prescan: [(&'static str, &CloneConfig); 4] = [
            ("crowdstrike", &config.clones.crowdstrike),
            ("armis", &config.clones.armis),
            ("claroty", &config.clones.claroty),
            ("cyberint", &config.clones.cyberint),
        ];
        // Collect (name, org_id_str, parsed_bytes) for scenario-enabled clones with a
        // present org_id that is a parseable UUID.
        // Clones with org_id=None are excluded from equality (E-DEMO-004 handles them).
        // Clones with an unparseable org_id string are also excluded here and will
        // surface as E-DEMO-005 (invalid UUID) during the validated_gen phase below.
        // Comparison is on PARSED UUID bytes to be case/format-insensitive:
        // "deadbeef-..." and "DEADBEEF-..." round-trip to the same 16-byte OrgId, so they
        // must NOT trigger E-DEMO-006. The guard's invariant is byte-based, not string-based.
        // (BC-2.06.019 PRE-6 rationale comment above, B-P5-05 fix.)
        let scenario_org_ids: Vec<(&'static str, &str, [u8; 16])> = all_gen_clones_e006_prescan
            .iter()
            .filter_map(|(name, cfg)| {
                if !cfg.enabled {
                    return None;
                }
                let sc = cfg.scenario.as_ref()?;
                if !sc.enabled {
                    return None;
                }
                // Only participate when org_id is present.
                let org_id_str = cfg.org_id.as_deref()?;
                // Parse to UUID bytes; skip (don't bail) if unparseable — E-DEMO-005
                // will fire for this clone in the validated_gen phase.
                let uuid = uuid::Uuid::parse_str(org_id_str).ok()?;
                Some((*name, org_id_str, *uuid.as_bytes()))
            })
            .collect();
        if scenario_org_ids.len() >= 2 {
            let (first_name, first_org_id_str, first_bytes) = scenario_org_ids[0];
            for (name, org_id_str, bytes) in &scenario_org_ids[1..] {
                if *bytes != first_bytes {
                    anyhow::bail!(
                        "demo-server: E-DEMO-006: scenario clones '{}' (org_id={}) and '{}' \
                         (org_id={}) have different org_ids; cross-DTU coherence requires all \
                         scenario-enabled clones to share the same org_id",
                        first_name,
                        first_org_id_str,
                        name,
                        org_id_str
                    );
                }
            }
        }
    }

    // ---------------------------------------------------------------------------
    // B-P4-01 fix: E-DEMO-003 (bad scenario archetype) must fire BEFORE E-DEMO-004
    // (missing org_id). Architecture Compliance Rules (story spec): guard order is
    // E-DEMO-002 → E-DEMO-006 → E-DEMO-003 → E-DEMO-004, all before any constructor.
    //
    // This prescan runs BEFORE `validated_gen` (which checks E-DEMO-004) so that
    // unrecognized/unsupported/contradictory scenario archetypes are detected first
    // even when org_id is also missing.
    // Mirrors the E-DEMO-002 prescan pattern above (lines 351-388).
    // ---------------------------------------------------------------------------
    #[cfg(feature = "fixture-gen")]
    {
        let gen_clones_e003_prescan: [(&'static str, &CloneConfig); 4] = [
            ("crowdstrike", &config.clones.crowdstrike),
            ("armis", &config.clones.armis),
            ("claroty", &config.clones.claroty),
            ("cyberint", &config.clones.cyberint),
        ];
        for (name, cfg) in &gen_clones_e003_prescan {
            if !cfg.enabled {
                continue;
            }
            let sc = match cfg.scenario.as_ref() {
                Some(s) if s.enabled => s,
                _ => continue,
            };
            // Validate scenario.archetype string; unrecognized → E-DEMO-003.
            let scenario_archetype = match sc.archetype.as_str() {
                "compromised_endpoint" => prism_dtu_common::Archetype::CompromisedEndpoint,
                "healthy" => prism_dtu_common::Archetype::HealthyOtEnvironment,
                other => anyhow::bail!(
                    "demo-server: E-DEMO-003: clone '{}': unrecognized scenario archetype \
                     '{}'; valid values: compromised_endpoint, healthy",
                    name,
                    other
                ),
            };
            // Direction 1: only CompromisedEndpoint supports scenario progression.
            if !matches!(
                scenario_archetype,
                prism_dtu_common::Archetype::CompromisedEndpoint
            ) {
                anyhow::bail!(
                    "demo-server: E-DEMO-003: clone '{}': unrecognized scenario archetype \
                     '{}'; valid values: compromised_endpoint, healthy",
                    name,
                    sc.archetype.as_str()
                );
            }
            // Direction 2: scenario.archetype must agree with fixture_set-derived archetype.
            // fixture_set_to_archetype is callable without org_id (no E-DEMO-004 risk).
            // If fixture_set is invalid, E-DEMO-001 fires here (before E-DEMO-003/004) —
            // that is the correct order since an invalid fixture_set makes the comparison moot.
            let fixture_archetype = fixture_set_to_archetype(&cfg.fixture_set, name)?;
            if fixture_archetype != scenario_archetype {
                anyhow::bail!(
                    "demo-server: E-DEMO-003: clone '{}': unrecognized scenario archetype \
                     '{}'; valid values: compromised_endpoint, healthy",
                    name,
                    sc.archetype.as_str()
                );
            }
            // Stage count validation (moved here from scenario_ctx as part of B-P5-OBS-2
            // single-source-of-truth): 0 = use defaults (allowed); non-zero must be exactly 4.
            // CompromisedEndpoint requires exactly 4 stage_duration_secs entries.
            let expected_stages: usize = 4; // CompromisedEndpoint (only supported archetype)
            if !sc.stage_duration_secs.is_empty() && sc.stage_duration_secs.len() != expected_stages
            {
                anyhow::bail!(
                    "demo-server: E-DEMO-003: clone '{}': stage_duration_secs has {} \
                     entries but archetype '{}' requires exactly {}",
                    name,
                    sc.stage_duration_secs.len(),
                    sc.archetype.as_str(),
                    expected_stages
                );
            }
        }
    }

    // ---------------------------------------------------------------------------
    // Story A: Validate ALL generator-backed clone configs BEFORE constructing any clone
    // (INV-CONSTRUCTION-TIME-FAILURE-001 — errors surface before constructors are called)
    //
    // The pre-loop both validates AND collects the parsed (Archetype, Option<OrgId>) for
    // each enabled clone into `validated_gen`. Construction blocks below look up from this
    // map — no re-parse, no `.expect()`.
    // ---------------------------------------------------------------------------
    #[cfg(feature = "fixture-gen")]
    let validated_gen: std::collections::HashMap<
        &'static str,
        (prism_dtu_common::Archetype, Option<prism_dtu_common::OrgId>),
    > = {
        // Names and configs for generator-backed clones.
        let gen_clones: [(&'static str, &CloneConfig); 4] = [
            ("crowdstrike", &config.clones.crowdstrike),
            ("armis", &config.clones.armis),
            ("claroty", &config.clones.claroty),
            ("cyberint", &config.clones.cyberint),
        ];
        let mut map = std::collections::HashMap::new();
        for (name, cfg) in &gen_clones {
            if !cfg.enabled {
                continue;
            }
            // E-DEMO-001: validate fixture_set (always, regardless of org_id)
            let archetype = fixture_set_to_archetype(&cfg.fixture_set, name)?;
            // E-DEMO-004: non-HealthyOtEnvironment archetype requires org_id
            if !matches!(archetype, prism_dtu_common::Archetype::HealthyOtEnvironment)
                && cfg.org_id.is_none()
            {
                require_org_id(&cfg.org_id, name)?;
            }
            // E-DEMO-005: if org_id present, parse and store it now (one parse per name)
            let org_id = if let Some(org_id_str) = &cfg.org_id {
                Some(parse_org_id(org_id_str, name)?)
            } else {
                None
            };
            map.insert(*name, (archetype, org_id));
        }
        map
    };

    // ---------------------------------------------------------------------------
    // Story B: Scenario context builder.
    //
    // The E-DEMO-002 (seed mismatch) and E-DEMO-003 (unrecognized archetype /
    // wrong stage_duration_secs length) guards have already fired as prescans
    // above (lines ~351-509). Any remaining scenario-enabled clones here have
    // already passed those checks. This block only builds the shared
    // `scenario_ctx` (Arc<IncidentTimeline> + ScenarioEntityCatalog) used by
    // construction blocks below. (B-P5-OBS-2: removed dead duplicate guards.)
    //
    // (BC-2.06.019 / INV-CONSTRUCTION-TIME-FAILURE-001)
    // ---------------------------------------------------------------------------
    #[cfg(feature = "fixture-gen")]
    let scenario_ctx: Option<(
        std::sync::Arc<prism_dtu_common::IncidentTimeline>,
        prism_dtu_common::ScenarioEntityCatalog,
        chrono::DateTime<chrono::Utc>,
    )> = {
        use prism_dtu_common::{build_default_incident_timeline, build_scenario_entity_catalog};

        // Collect all scenario-enabled generator-backed clones (name, seed).
        // archetype_str and stage_duration_secs were needed by the E-DEMO-003 guard
        // which now lives exclusively in the prescan above. Only name and seed are
        // needed here to build the scenario context. (B-P5-OBS-2)
        let all_gen_clones: [(&'static str, &CloneConfig); 4] = [
            ("crowdstrike", &config.clones.crowdstrike),
            ("armis", &config.clones.armis),
            ("claroty", &config.clones.claroty),
            ("cyberint", &config.clones.cyberint),
        ];

        let scenario_enabled: Vec<(&'static str, u64)> = all_gen_clones
            .iter()
            .filter_map(|(name, cfg)| {
                if !cfg.enabled {
                    return None;
                }
                let sc = cfg.scenario.as_ref()?;
                if !sc.enabled {
                    return None;
                }
                Some((*name, cfg.seed))
            })
            .collect();

        if scenario_enabled.is_empty() {
            None
        } else {
            // All E-DEMO-002/003/006 guards have already fired as prescans above.
            // Build the shared scenario context from the verified inputs.
            // (B-P5-OBS-2 single-source-of-truth: guards live in prescans only.)
            // Use the first scenario-enabled clone's config for seed + org_id.
            // (E-DEMO-002 guarantees all seeds match; E-DEMO-004 guarantees org_id present.)
            let (first_sc_name, scenario_seed) = &scenario_enabled[0];
            let first_cfg = match *first_sc_name {
                "crowdstrike" => &config.clones.crowdstrike,
                "armis" => &config.clones.armis,
                "claroty" => &config.clones.claroty,
                _ => &config.clones.cyberint,
            };
            let sc_config = first_cfg.scenario.as_ref().ok_or_else(|| {
                anyhow::anyhow!("internal: scenario config missing after enable check")
            })?;

            // Parse org_id from the first scenario-enabled clone.
            let org_id_str = first_cfg.org_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!(
                    "demo-server: E-DEMO-004: clone '{}': scenario.enabled requires \
                         org_id to be set (UUID string)",
                    first_sc_name
                )
            })?;
            let org_id = parse_org_id(org_id_str, first_sc_name)?;

            // Build entity catalog from seed + org_id.
            let catalog = build_scenario_entity_catalog(*scenario_seed, &org_id);

            // Derive scenario_start_epoch_secs: config value or current system time.
            let scenario_start_secs: i64 = sc_config
                .scenario_start_secs
                .unwrap_or_else(|| chrono::Utc::now().timestamp());

            // Build the IncidentTimeline.
            let timeline = build_default_incident_timeline(
                catalog.clone(),
                scenario_start_secs,
                &sc_config.stage_duration_secs,
            );
            let timeline_arc = std::sync::Arc::new(timeline);

            // time_anchor for generators: use scenario_start_secs as the anchor epoch.
            // BC-2.06.019 §2.3: time_anchor is derived once from scenario_start_secs.
            let time_anchor =
                chrono::DateTime::from_timestamp(scenario_start_secs, 0).ok_or_else(|| {
                    anyhow::anyhow!(
                        "demo-server: scenario_start_secs={} is out of valid timestamp range",
                        scenario_start_secs
                    )
                })?;

            Some((timeline_arc, catalog, time_anchor))
        }
    };

    let mut pairs = Vec::new();

    if config.clones.crowdstrike.enabled {
        #[cfg(feature = "fixture-gen")]
        let pair_clone: Box<dyn BehavioralClone> = {
            let cfg = &config.clones.crowdstrike;
            let scenario_active = cfg.scenario.as_ref().map(|s| s.enabled).unwrap_or(false);
            if scenario_active {
                if let (
                    Some((archetype, Some(org_id))),
                    Some((timeline_arc, catalog, time_anchor)),
                ) = (
                    validated_gen.get("crowdstrike").cloned(),
                    scenario_ctx.as_ref(),
                ) {
                    // Story B: scenario path — call new_with_scenario (BC-2.06.019 §2.3).
                    // F-PIVOT003-R2-001: thread catalog so detection 0 carries ioc_value.
                    Box::new(CrowdstrikeClone::new_with_scenario(
                        cfg.seed,
                        archetype,
                        org_id,
                        std::sync::Arc::clone(timeline_arc),
                        *time_anchor,
                        catalog,
                    ))
                } else {
                    Box::new(CrowdstrikeClone::new())
                }
            } else if let Some((archetype, Some(org_id))) =
                validated_gen.get("crowdstrike").cloned()
            {
                // Story A: seeded path — new_with_seed (ADR-036 §2.3).
                Box::new(CrowdstrikeClone::new_with_seed(cfg.seed, archetype, org_id))
            } else {
                Box::new(CrowdstrikeClone::new())
            }
        };
        #[cfg(not(feature = "fixture-gen"))]
        let pair_clone: Box<dyn BehavioralClone> = Box::new(CrowdstrikeClone::new());

        let mut pair = ClonePair::new("crowdstrike", pair_clone);
        pair.continue_on_error = config.clones.crowdstrike.continue_on_error;
        pairs.push(pair);
    }

    if config.clones.claroty.enabled {
        #[cfg(feature = "fixture-gen")]
        let pair_clone: Box<dyn BehavioralClone> = {
            let cfg = &config.clones.claroty;
            let scenario_active = cfg.scenario.as_ref().map(|s| s.enabled).unwrap_or(false);
            if scenario_active {
                if let (Some((archetype, Some(org_id))), Some((timeline_arc, _, time_anchor))) =
                    (validated_gen.get("claroty").cloned(), scenario_ctx.as_ref())
                {
                    Box::new(ClarotyClone::new_with_scenario(
                        cfg.seed,
                        archetype,
                        org_id,
                        std::sync::Arc::clone(timeline_arc),
                        *time_anchor,
                    ))
                } else {
                    Box::new(ClarotyClone::new())
                }
            } else if let Some((archetype, Some(org_id))) = validated_gen.get("claroty").cloned() {
                Box::new(ClarotyClone::new_with_seed(cfg.seed, archetype, org_id))
            } else {
                Box::new(ClarotyClone::new())
            }
        };
        #[cfg(not(feature = "fixture-gen"))]
        let pair_clone: Box<dyn BehavioralClone> = Box::new(ClarotyClone::new());

        let mut pair = ClonePair::new("claroty", pair_clone);
        pair.continue_on_error = config.clones.claroty.continue_on_error;
        pairs.push(pair);
    }

    if config.clones.cyberint.enabled {
        // Seed the access_token allowlist at construction time via new_with_access_token()
        // (ADR-031 §D3-a). When initial_access_token is set in demo.toml, it is registered
        // in the Cyberint clone's static allowlist before the server starts.
        #[cfg(feature = "fixture-gen")]
        let clone: Box<dyn BehavioralClone> = {
            let cfg = &config.clones.cyberint;
            let scenario_active = cfg.scenario.as_ref().map(|s| s.enabled).unwrap_or(false);
            if scenario_active {
                if let (
                    Some((archetype, Some(org_id))),
                    Some((timeline_arc, catalog, time_anchor)),
                ) = (
                    validated_gen.get("cyberint").cloned(),
                    scenario_ctx.as_ref(),
                ) {
                    Box::new(
                        CyberintClone::new_with_scenario(
                            cfg.seed,
                            archetype,
                            org_id,
                            std::sync::Arc::clone(timeline_arc),
                            *time_anchor,
                            catalog,
                        )
                        .context("failed to construct CyberintClone::new_with_scenario")?,
                    )
                } else {
                    Box::new(CyberintClone::new().context("failed to construct CyberintClone")?)
                }
            } else if let Some((archetype, Some(org_id))) = validated_gen.get("cyberint").cloned() {
                Box::new(
                    CyberintClone::new_with_seed(cfg.seed, archetype, org_id)
                        .context("failed to construct CyberintClone::new_with_seed")?,
                )
            } else if let Some(token) = &cfg.initial_access_token {
                Box::new(
                    CyberintClone::new_with_access_token(token.clone())
                        .context("failed to construct CyberintClone with access token")?,
                )
            } else {
                Box::new(CyberintClone::new().context("failed to construct CyberintClone")?)
            }
        };
        #[cfg(not(feature = "fixture-gen"))]
        let clone: Box<dyn BehavioralClone> = {
            let cfg = &config.clones.cyberint;
            if let Some(token) = &cfg.initial_access_token {
                Box::new(
                    CyberintClone::new_with_access_token(token.clone())
                        .context("failed to construct CyberintClone with access token")?,
                )
            } else {
                Box::new(CyberintClone::new().context("failed to construct CyberintClone")?)
            }
        };
        let mut pair = ClonePair::new("cyberint", clone);
        pair.continue_on_error = config.clones.cyberint.continue_on_error;
        pairs.push(pair);
    }

    if config.clones.armis.enabled {
        #[cfg(feature = "fixture-gen")]
        let pair_clone: Box<dyn BehavioralClone> = {
            let cfg = &config.clones.armis;
            let scenario_active = cfg.scenario.as_ref().map(|s| s.enabled).unwrap_or(false);
            if scenario_active {
                if let (
                    Some((archetype, Some(org_id))),
                    Some((timeline_arc, catalog, time_anchor)),
                ) = (validated_gen.get("armis").cloned(), scenario_ctx.as_ref())
                {
                    Box::new(
                        ArmisClone::new_with_scenario(
                            cfg.seed,
                            archetype,
                            org_id,
                            std::sync::Arc::clone(timeline_arc),
                            *time_anchor,
                            catalog,
                        )
                        .context("failed to construct ArmisClone::new_with_scenario")?,
                    )
                } else {
                    Box::new(ArmisClone::new().context("failed to construct ArmisClone")?)
                }
            } else if let Some((archetype, Some(org_id))) = validated_gen.get("armis").cloned() {
                // org_slug is derived internally by new_with_seed (no longer a constructor arg).
                Box::new(
                    ArmisClone::new_with_seed(cfg.seed, archetype, org_id)
                        .context("failed to construct ArmisClone::new_with_seed")?,
                )
            } else {
                Box::new(ArmisClone::new().context("failed to construct ArmisClone")?)
            }
        };
        #[cfg(not(feature = "fixture-gen"))]
        let pair_clone: Box<dyn BehavioralClone> =
            Box::new(ArmisClone::new().context("failed to construct ArmisClone")?);

        let mut pair = ClonePair::new("armis", pair_clone);
        pair.continue_on_error = config.clones.armis.continue_on_error;
        pairs.push(pair);
    }

    if config.clones.threatintel.enabled {
        #[cfg(feature = "fixture-gen")]
        let threatintel_clone: Box<dyn BehavioralClone> = {
            // In scenario mode, inject all scenario IOCs into the fixture registry.
            if let Some((_, catalog, _)) = &scenario_ctx {
                Box::new(ThreatIntelClone::new_with_scenario(catalog))
            } else {
                Box::new(ThreatIntelClone::new())
            }
        };
        #[cfg(not(feature = "fixture-gen"))]
        let threatintel_clone: Box<dyn BehavioralClone> = Box::new(ThreatIntelClone::new());

        let mut pair = ClonePair::new("threatintel", threatintel_clone);
        pair.continue_on_error = config.clones.threatintel.continue_on_error;
        pairs.push(pair);
    }

    if config.clones.nvd.enabled {
        #[cfg(feature = "fixture-gen")]
        let nvd_clone: Box<dyn BehavioralClone> = {
            // In scenario mode, inject synthetic CVE records for device_cves.
            if let Some((_, catalog, _)) = &scenario_ctx {
                Box::new(
                    NvdClone::new_with_scenario(catalog)
                        .context("failed to construct NvdClone::new_with_scenario")?,
                )
            } else {
                Box::new(NvdClone::new().context("failed to construct NvdClone")?)
            }
        };
        #[cfg(not(feature = "fixture-gen"))]
        let nvd_clone: Box<dyn BehavioralClone> =
            Box::new(NvdClone::new().context("failed to construct NvdClone")?);

        let mut pair = ClonePair::new("nvd", nvd_clone);
        pair.continue_on_error = config.clones.nvd.continue_on_error;
        pairs.push(pair);
    }

    Ok(pairs)
}

// ---------------------------------------------------------------------------
// Story A: fixture_set → Archetype mapping + E-DEMO error guards
// (BC-2.06.018 / ADR-036 §2.4)
// ---------------------------------------------------------------------------

/// Canonical `fixture_set` string → `Archetype` mapping (INV-FIXTURE-SET-ARCHETYPE-MAP-001).
///
/// Returns `Err` with E-DEMO-001 message for unrecognized values.
///
/// # Error format (BC-2.06.018 §Error Codes E-DEMO-001)
///
/// ```text
/// demo-server: E-DEMO-001: clone '{clone_name}': unrecognized fixture_set '{value}';
/// valid values: default, compromised, auth_outage, large_scale, pagination_edges,
///               schema_drift, high_churn, dormant
/// ```
///
/// Gated `#[cfg(feature = "fixture-gen")]` because `Archetype` is only available
/// when the generator module is active.
#[cfg(feature = "fixture-gen")]
pub fn fixture_set_to_archetype(
    fixture_set: &str,
    clone_name: &str,
) -> anyhow::Result<prism_dtu_common::Archetype> {
    use prism_dtu_common::Archetype;
    match fixture_set {
        "default" => Ok(Archetype::HealthyOtEnvironment),
        "compromised" => Ok(Archetype::CompromisedEndpoint),
        "auth_outage" => Ok(Archetype::AuthOutage),
        "large_scale" => Ok(Archetype::LargeScale),
        "pagination_edges" => Ok(Archetype::PaginationEdgeCases),
        "schema_drift" => Ok(Archetype::SchemaDrift),
        "high_churn" => Ok(Archetype::HighChurn),
        "dormant" => Ok(Archetype::DormantTenant),
        other => anyhow::bail!(
            "demo-server: E-DEMO-001: clone '{}': unrecognized fixture_set '{}'; \
             valid values: default, compromised, auth_outage, large_scale, pagination_edges, \
             schema_drift, high_churn, dormant",
            clone_name,
            other
        ),
    }
}

/// Parse `org_id` string to `prism_dtu_common::OrgId`.
///
/// Returns E-DEMO-005 if the string is not a valid UUID.
///
/// # Error format (BC-2.06.018 §Error Codes E-DEMO-005)
///
/// ```text
/// demo-server: E-DEMO-005: clone '{clone_name}': org_id '{value}' is not a valid UUID
/// ```
///
/// Gated `#[cfg(feature = "fixture-gen")]` because `OrgId` (the generator's [u8;16] type)
/// is only available when the generator module is active.
#[cfg(feature = "fixture-gen")]
pub fn parse_org_id(org_id_str: &str, clone_name: &str) -> anyhow::Result<prism_dtu_common::OrgId> {
    let uuid = uuid::Uuid::parse_str(org_id_str).map_err(|_| {
        anyhow::anyhow!(
            "demo-server: E-DEMO-005: clone '{}': org_id '{}' is not a valid UUID",
            clone_name,
            org_id_str
        )
    })?;
    Ok(prism_dtu_common::OrgId(*uuid.as_bytes()))
}

/// Validate that `org_id` is present when `new_with_seed` is required.
///
/// Returns E-DEMO-004 if `org_id` is `None` and the fixture_set requires seeding
/// (i.e., when `new_with_seed` would be called).
///
/// # Error format (BC-2.06.018 §Error Codes E-DEMO-004)
///
/// ```text
/// demo-server: E-DEMO-004: clone '{clone_name}': scenario.enabled requires org_id to be set (UUID string)
/// ```
///
pub fn require_org_id(org_id: &Option<String>, clone_name: &str) -> anyhow::Result<()> {
    if org_id.is_none() {
        anyhow::bail!(
            "demo-server: E-DEMO-004: clone '{}': scenario.enabled requires org_id to be set (UUID string)",
            clone_name
        );
    }
    Ok(())
}

/// Parse a `SocketAddr` from a `CloneConfig` bind IP and port.
pub fn clone_bind_addr(cfg: &CloneConfig) -> anyhow::Result<SocketAddr> {
    let addr_str = format!("{}:{}", cfg.bind, cfg.port);
    addr_str
        .parse::<SocketAddr>()
        .with_context(|| format!("Invalid bind address: {}", addr_str))
}

/// Validate bind security (AC-9, R-DEMO-001).
///
/// Non-loopback binds require BOTH `--bind-any` flag AND
/// `PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK`.
///
/// In the harness, `bind_any` is tracked via the env var only (the CLI flag sets it
/// before calling `start_all`). The harness checks the config for non-loopback IPs.
fn validate_bind_security(config: &DemoConfig) -> anyhow::Result<()> {
    let allow_env = std::env::var("PRISM_DTU_DEMO_ALLOW_NETWORK_BIND").unwrap_or_default();
    let allowed = allow_env == "I-UNDERSTAND-THE-RISK";

    // Check harness bind.
    if !is_loopback(&config.harness.bind) && !allowed {
        anyhow::bail!(
            "Non-loopback bind address '{}' rejected. \
             Set --bind-any AND PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK \
             to allow network binding (R-DEMO-001).",
            config.harness.bind
        );
    }

    // Check each clone bind.
    for (name, cfg) in all_clone_configs(config) {
        if cfg.enabled && !is_loopback(&cfg.bind) && !allowed {
            anyhow::bail!(
                "Non-loopback bind address '{}' for clone '{}' rejected. \
                 Set --bind-any AND PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK \
                 to allow network binding (R-DEMO-001). --bind-any loopback",
                cfg.bind,
                name
            );
        }
    }

    Ok(())
}

/// Return true if the IP string is a loopback address.
fn is_loopback(ip: &str) -> bool {
    if let Ok(addr) = ip.parse::<std::net::IpAddr>() {
        addr.is_loopback()
    } else {
        // Unparseable — be safe and treat as non-loopback.
        false
    }
}

/// Return all clone configs as (name, &CloneConfig).
fn all_clone_configs(config: &DemoConfig) -> Vec<(&'static str, &CloneConfig)> {
    vec![
        ("crowdstrike", &config.clones.crowdstrike),
        ("claroty", &config.clones.claroty),
        ("cyberint", &config.clones.cyberint),
        ("armis", &config.clones.armis),
        ("threatintel", &config.clones.threatintel),
        ("nvd", &config.clones.nvd),
    ]
}

/// Get the `CloneConfig` for a named clone.
fn clone_config_by_name<'a>(config: &'a DemoConfig, name: &str) -> &'a CloneConfig {
    match name {
        "crowdstrike" => &config.clones.crowdstrike,
        "claroty" => &config.clones.claroty,
        "cyberint" => &config.clones.cyberint,
        "armis" => &config.clones.armis,
        "threatintel" => &config.clones.threatintel,
        "nvd" => &config.clones.nvd,
        other => panic!("unknown clone name: {other}"),
    }
}

/// Convert an `anyhow::Error` into a `std::io::Error`, preserving `AddrInUse` kind
/// when the underlying error is an OS bind error.
fn to_io_error(err: anyhow::Error) -> std::io::Error {
    // Walk the error chain to find an io::Error.
    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
        std::io::Error::new(io_err.kind(), io_err.to_string())
    } else {
        // Check if source chain has an io::Error.
        let mut source: Option<&dyn std::error::Error> = err.source();
        while let Some(s) = source {
            if let Some(io_err) = s.downcast_ref::<std::io::Error>() {
                return std::io::Error::new(io_err.kind(), io_err.to_string());
            }
            source = s.source();
        }
        std::io::Error::other(err.to_string())
    }
}

/// Test utilities.
///
/// Always compiled (this crate is test/demo infrastructure only; it never
/// links into production binaries). Integration tests access this module via
/// `prism_dtu_demo_server::harness::test_utils`.
pub mod test_utils {
    use std::net::SocketAddr;

    /// Assert that the given `SocketAddr` is no longer bound within `timeout`.
    ///
    /// Used by AC-11 tests to verify no listener leak after partial-startup failure.
    pub async fn assert_port_released(addr: SocketAddr, timeout: std::time::Duration) {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if tokio::net::TcpListener::bind(addr).await.is_ok() {
                return;
            }
            if tokio::time::Instant::now() >= deadline {
                panic!("port {} still bound after {:?}", addr, timeout);
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
}
