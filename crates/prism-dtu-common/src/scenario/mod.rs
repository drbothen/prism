//! Scenario entity catalog and derivation helpers (BC-2.06.018 / ADR-036 §2.2).
//!
//! Gated behind `#[cfg(feature = "fixture-gen")]` — see `lib.rs`.
//!
//! The primary entry point is [`build_scenario_entity_catalog`], which derives a
//! [`ScenarioEntityCatalog`] from a `(seed, org_id)` pair using a secondary RNG stream
//! (`gen_seeded_rng(seed.wrapping_add(1), &org_id)`) independent of the primary generator
//! stream.  All derived entity IDs follow the canonical format specified in ADR-036 §2.2:
//!
//! ```text
//! org_slug = hex(org_id.as_bytes()[0..4])   // exactly 8 lowercase hex chars
//! device_id = "dev-{org_slug}-{seed}-{n}"
//! ```
//!
//! This module is the authoritative source of `org_slug_from_org_id`; the formula
//! MUST match `prism_dtu_crowdstrike::generator::org_slug` exactly (ADR-036 §2.2).

use super::generator::{seeded_rng as gen_seeded_rng, OrgId};

// ---------------------------------------------------------------------------
// Story B: IncidentTimeline layer (BC-2.06.019 / ADR-036 v2.3 §2.2)
// ---------------------------------------------------------------------------

/// A single stage in the incident timeline.
///
/// `#[non_exhaustive]` per CLAUDE.md §Conventions (public type in prism-dtu-common).
/// `IncidentStage` is a public type added by S-DEMO-DTU-LIVE-SCENARIO-001-B.
///
/// ADR-036 v2.3 §2.2: `name` is a static string; `activates_after_secs` is the
/// elapsed-time threshold at which this stage becomes current.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct IncidentStage {
    /// Human-readable stage name (e.g. `"Baseline"`, `"Recon"`, …).
    pub name: &'static str,
    /// Seconds after `scenario_start_epoch_secs` at which this stage activates.
    /// Stage 0 (Baseline) always has `activates_after_secs = 0`.
    pub activates_after_secs: u64,
    /// Which entity categories are visible at this stage.
    pub visible_entity_mask: StageMask,
}

/// Which entity categories are visible at a given stage.
///
/// NOT `#[non_exhaustive]` — internal struct, must be exhaustively constructible
/// within `prism-dtu-common` (BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001).
/// ADR-036 v2.3 §2.2 code snippet erroneously marks this `#[non_exhaustive]`;
/// BC-2.06.019 wins per CLAUDE.md Source-of-Truth Precedence for contract semantics.
#[derive(Clone, Debug)]
pub struct StageMask {
    /// Primary compromised device is visible.
    pub primary_device: bool,
    /// Lateral-movement target devices are visible.
    pub lateral_devices: bool,
    /// IOC IPv4 addresses are visible.
    pub ioc_ips: bool,
    /// IOC domain names are visible.
    pub ioc_domains: bool,
    /// IOC SHA256 file hashes are visible.
    pub ioc_hashes: bool,
    /// CVE IDs assigned to the primary device are visible.
    pub device_cves: bool,
}

/// Temporal incident timeline for a single demo client.
///
/// `#[non_exhaustive]` per CLAUDE.md §Conventions (public type in prism-dtu-common).
/// `IncidentTimeline` is a public type added by S-DEMO-DTU-LIVE-SCENARIO-001-B.
///
/// # ADR-036 v2.3 §2.2 — read-only after construction
///
/// `IncidentTimeline` is threaded as `Arc<IncidentTimeline>` (NOT `Arc<Mutex<...>>`).
/// Route handlers call `current_stage_index` with this reference on every request.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct IncidentTimeline {
    /// Shared entity catalog used by all DTUs for this client.
    pub entities: ScenarioEntityCatalog,
    /// Ordered stage list; index 0 is always Baseline (`activates_after_secs = 0`).
    pub stages: Vec<IncidentStage>,
    /// Unix epoch seconds at which the scenario started.
    pub scenario_start_epoch_secs: i64,
}

/// Compute the current stage index given the timeline and the current wall-clock epoch.
///
/// **Pure function** — no side effects, no shared mutable state, no tokio spawn.
/// ADR-036 v2.3 §2.1 mandates: concurrent callers always get the same result for
/// the same `(timeline, now_epoch_secs)` pair.
///
/// # Formula (ADR-036 v2.2 §2.2)
///
/// ```text
/// elapsed = max(0, now_epoch_secs - timeline.scenario_start_epoch_secs) as u64;
/// stage   = last index i where timeline.stages[i].activates_after_secs <= elapsed
/// ```
///
/// Stage index saturates at `stages.len() - 1` (EC-019-004 / EC-002).
pub fn current_stage_index(timeline: &IncidentTimeline, now_epoch_secs: i64) -> usize {
    // Clamp negative elapsed to 0 (EC-019-003 / EC-001: clock skew / future start).
    let elapsed = now_epoch_secs
        .saturating_sub(timeline.scenario_start_epoch_secs)
        .max(0) as u64;

    // Stub: return 0 so compilation succeeds but tests exercising specific stage
    // boundaries FAIL (Red Gate — assertion will mismatch).
    //
    // S-DEMO-DTU-LIVE-SCENARIO-001-B: implementer replaces with the correct loop
    // (last index i where stages[i].activates_after_secs <= elapsed).
    let _ = elapsed;
    0
}

/// Build the default `CompromisedEndpoint` `IncidentTimeline` from a catalog and thresholds.
///
/// `stage_duration_secs`: 4-entry array for stages 1-4 activation thresholds.
/// Stage 0 (Baseline) always activates at 0 — no array entry.
/// When empty, defaults to `[60, 180, 360, 600]` (BC-2.06.019 §Postcondition 2).
///
/// # Stub (S-DEMO-DTU-LIVE-SCENARIO-001-B)
///
/// Returns a 1-stage timeline so compilation succeeds. Tests exercising the 5-stage
/// structure will FAIL (Red Gate).
#[allow(dead_code)] // S-DEMO-DTU-LIVE-SCENARIO-001-B: transient until implementation wires this
pub fn build_default_incident_timeline(
    catalog: ScenarioEntityCatalog,
    start_secs: i64,
    stage_duration_secs: &[u64],
) -> IncidentTimeline {
    let _thresholds: &[u64] = if stage_duration_secs.is_empty() {
        &[60, 180, 360, 600]
    } else {
        stage_duration_secs
    };

    // Stub: builds only the Baseline stage so compilation works but multi-stage
    // tests FAIL (Red Gate — stage count and mask assertions will mismatch).
    let stages = vec![IncidentStage {
        name: "Baseline",
        activates_after_secs: 0,
        visible_entity_mask: StageMask {
            primary_device: true,
            lateral_devices: false,
            ioc_ips: false,
            ioc_domains: false,
            ioc_hashes: false,
            device_cves: false,
        },
    }];

    IncidentTimeline {
        entities: catalog,
        stages,
        scenario_start_epoch_secs: start_secs,
    }
}

/// Shared entity catalog for one client's incident scenario.
///
/// Produced once at harness construction time from `(seed, org_id)`.
/// All DTU projections for this client derive their entity IDs from this catalog.
///
/// # ADR-036 §2.2 — Canonical org_slug derivation
///
/// `org_slug = hex(org_id.as_bytes()[0..4])` — 8 lowercase hex chars.
/// Example: OrgId whose bytes start `[0xde, 0xad, 0xbe, 0xef, ...]` → `org_slug = "deadbeef"`.
///
/// # ADR-036 §3.4 — Location constraint
///
/// This type MUST live in `prism-dtu-common/src/scenario/` — NOT in a separate
/// `prism-dtu-scenario` crate.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct ScenarioEntityCatalog {
    /// Canonical org_slug derived from org_id bytes (hex of first 4 bytes).
    ///
    /// Used by both CrowdStrike and Armis generators for consistent ID derivation.
    /// Formula: `hex(org_id.as_bytes()[0..4])` — 8 lowercase hex chars.
    pub org_slug: String,

    /// The primary compromised device ID in CrowdStrike ID format.
    ///
    /// Format: `"dev-{org_slug}-{seed}-0"`.
    /// Example (org bytes `[0xde, 0xad, 0xbe, 0xef, ...]`, seed=42):
    ///   `"dev-deadbeef-42-0"`.
    pub primary_device_id_cs: String,

    /// The primary compromised device ID in Armis ID format.
    ///
    /// Format: `"dev-{org_slug}-{seed}-0"` — same formula as CrowdStrike.
    /// The Armis generator receives `org_slug` as an explicit `&str` arg.
    pub primary_device_id_armis: String,

    /// Hostname for the compromised device (consistent across DTUs).
    pub primary_hostname: String,

    /// Secondary device IDs involved in lateral movement (CrowdStrike format).
    pub lateral_device_ids_cs: Vec<String>,

    /// Secondary device IDs involved in lateral movement (Armis format).
    pub lateral_device_ids_armis: Vec<String>,

    /// IOC IPv4 addresses introduced during Exfil stage.
    ///
    /// Derived from the secondary RNG stream (`gen_seeded_rng(seed.wrapping_add(1), &org_id)`).
    /// MUST resolve as malicious in ThreatIntel.
    pub ioc_ips: Vec<String>,

    /// IOC domain names introduced during Exfil stage.
    ///
    /// Derived from the secondary RNG stream.
    pub ioc_domains: Vec<String>,

    /// IOC SHA256 file hashes introduced during LateralMovement stage.
    ///
    /// Derived from the secondary RNG stream.
    pub ioc_hashes: Vec<String>,

    /// CVE IDs assigned to the primary device.
    ///
    /// Derived from the secondary RNG stream.
    /// MUST resolve in NVD (base_score >= 7.0).
    pub device_cves: Vec<String>,
}

/// Derive the canonical org_slug from OrgId bytes.
///
/// Formula: `hex(org_id.as_bytes()[0..4])` — exactly 8 lowercase hex characters.
///
/// This formula MUST match `prism_dtu_crowdstrike::generator::org_slug()` exactly
/// (ADR-036 §2.2).  The Armis generator receives this value as the `org_slug: &str`
/// argument.
///
/// # Example
///
/// ```rust,ignore
/// let org = OrgId([0xde, 0xad, 0xbe, 0xef, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
/// assert_eq!(org_slug_from_org_id(&org), "deadbeef");
/// ```
pub fn org_slug_from_org_id(org_id: &OrgId) -> String {
    let bytes = org_id.as_bytes();
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3]
    )
}

/// Build a [`ScenarioEntityCatalog`] from `(seed, org_id)`.
///
/// The IOC IPs, domains, hashes, and CVE IDs are derived from a **secondary** RNG stream:
/// `gen_seeded_rng(seed.wrapping_add(1), org_id)` — completely independent of the
/// primary generator stream used by the individual clone generators.
///
/// The secondary stream is needed so that catalog derivation does not consume RNG state
/// from the primary stream (which would shift all generated record IDs).
///
/// # Formula (ADR-036 §2.2)
///
/// - `org_slug = hex(org_id.as_bytes()[0..4])`
/// - `primary_device_id_cs   = "dev-{org_slug}-{seed}-0"`
/// - `primary_device_id_armis = "dev-{org_slug}-{seed}-0"` (same)
/// - `ioc_ips`, `ioc_domains`, `ioc_hashes`, `device_cves` from secondary RNG stream
pub fn build_scenario_entity_catalog(seed: u64, org_id: &OrgId) -> ScenarioEntityCatalog {
    let org_slug = org_slug_from_org_id(org_id);

    let primary_device_id_cs = format!("dev-{org_slug}-{seed}-0");
    let primary_device_id_armis = format!("dev-{org_slug}-{seed}-0");
    let primary_hostname = format!("host-{org_slug}-{seed}");

    // Lateral device IDs (indices 1..=3)
    let lateral_device_ids_cs: Vec<String> = (1..=3)
        .map(|n| format!("dev-{org_slug}-{seed}-{n}"))
        .collect();
    let lateral_device_ids_armis: Vec<String> = (1..=3)
        .map(|n| format!("dev-{org_slug}-{seed}-{n}"))
        .collect();

    // Secondary RNG stream — completely independent of the primary generator stream.
    // gen_seeded_rng(seed.wrapping_add(1), org_id) per ADR-036 §2.2.
    let mut rng = gen_seeded_rng(seed.wrapping_add(1), org_id);

    let ioc_ips = gen_ioc_ips(&mut rng, 4);
    let ioc_domains = gen_ioc_domains(&mut rng, 4);
    let ioc_hashes = gen_ioc_hashes(&mut rng, 4);
    let device_cves = gen_device_cves(&mut rng, 3);

    ScenarioEntityCatalog {
        org_slug,
        primary_device_id_cs,
        primary_device_id_armis,
        primary_hostname,
        lateral_device_ids_cs,
        lateral_device_ids_armis,
        ioc_ips,
        ioc_domains,
        ioc_hashes,
        device_cves,
    }
}

// ---------------------------------------------------------------------------
// Private helpers (used by build_scenario_entity_catalog implementation)
// ---------------------------------------------------------------------------

/// Generate N random IPv4 addresses in the 10.x.x.x range from RNG.
fn gen_ioc_ips(rng: &mut impl rand::Rng, count: usize) -> Vec<String> {
    (0..count)
        .map(|_| {
            format!(
                "10.{}.{}.{}",
                rng.gen::<u8>(),
                rng.gen::<u8>(),
                rng.gen::<u8>()
            )
        })
        .collect()
}

/// Generate N IOC domain names from RNG.
fn gen_ioc_domains(rng: &mut impl rand::Rng, count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("malicious-{}-{}.example.com", rng.gen::<u32>(), i))
        .collect()
}

/// Generate N IOC SHA256 hashes (as hex strings) from RNG.
///
/// Produces a proper 64-hex-char string from 32 random bytes (`{:02x}` per byte),
/// matching the SHA-256 output representation (256 bits / 8 bits-per-char = 64 chars).
/// The prior `{:01x}` with nibble-masking discarded the upper nibble, yielding only
/// 4 bits per character and a non-representative hash distribution.
fn gen_ioc_hashes(rng: &mut impl rand::Rng, count: usize) -> Vec<String> {
    (0..count)
        .map(|_| {
            (0..32)
                .map(|_| format!("{:02x}", rng.gen::<u8>()))
                .collect::<String>()
        })
        .collect()
}

/// Generate N CVE ID strings from RNG.
///
/// Format: `"CVE-{year}-{n}"` where year and n are RNG-derived.
fn gen_device_cves(rng: &mut impl rand::Rng, count: usize) -> Vec<String> {
    (0..count)
        .map(|_| {
            format!(
                "CVE-{}-{}",
                2020u32 + (rng.gen::<u32>() % 5),
                rng.gen::<u32>() % 100000
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Unit tests (Red Gate tests for BC-2.06.018 and BC-2.06.019)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Org UUID with well-known bytes for canonical ID format assertions.
    ///
    /// First 4 bytes: [0xde, 0xad, 0xbe, 0xef] → org_slug = "deadbeef"
    /// Primary device ID (seed=42): "dev-deadbeef-42-0"
    ///
    /// ADR-036 §2.2: "Any test using 'dev-acme-...' is incorrect."
    fn deadbeef_org() -> OrgId {
        OrgId([
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ])
    }

    // ---------------------------------------------------------------------------
    // RED GATE TEST 1 — test_BC_2_06_019_timeline_types_non_exhaustive_and_structure
    //
    // BC-2.06.019 PRE-3 / ADR-036 v2.2 §2.2
    // Verifies: IncidentTimeline is #[non_exhaustive] with correct fields,
    // IncidentStage is #[non_exhaustive] with correct fields,
    // StageMask is NOT #[non_exhaustive] (internal, exhaustively constructible).
    //
    // FAIL mode: build_default_incident_timeline returns only 1 stage (Baseline),
    // so the assertion stages.len() == 5 will FAIL.
    // ---------------------------------------------------------------------------
    #[test]
    fn test_BC_2_06_019_timeline_types_non_exhaustive_and_structure() {
        let org = deadbeef_org();
        let catalog = build_scenario_entity_catalog(42, &org);

        let timeline = build_default_incident_timeline(catalog, 1_000_000, &[]);

        // IncidentTimeline must have fields: entities, stages, scenario_start_epoch_secs
        assert_eq!(
            timeline.scenario_start_epoch_secs, 1_000_000,
            "IncidentTimeline.scenario_start_epoch_secs must round-trip the provided start_secs"
        );

        // Default CompromisedEndpoint timeline must have exactly 5 stages.
        // FAIL: stub returns 1 stage (Baseline only).
        assert_eq!(
            timeline.stages.len(),
            5,
            "Default CompromisedEndpoint timeline must have exactly 5 stages \
             (Baseline, Recon, LateralMovement, Exfil, Containment); \
             got {} — BC-2.06.019 §Postcondition 2 / ADR-036 v2.2 §2.2",
            timeline.stages.len()
        );

        // Stage 0 must be Baseline at 0 seconds.
        assert_eq!(
            timeline.stages[0].name, "Baseline",
            "stages[0].name must be 'Baseline'; got '{}'",
            timeline.stages[0].name
        );
        assert_eq!(
            timeline.stages[0].activates_after_secs, 0,
            "stages[0].activates_after_secs must be 0 (Baseline always starts at 0)"
        );

        // StageMask must be exhaustively constructible (NOT #[non_exhaustive]).
        // This compiles only if StageMask has no #[non_exhaustive] attribute.
        let _mask = StageMask {
            primary_device: true,
            lateral_devices: false,
            ioc_ips: false,
            ioc_domains: false,
            ioc_hashes: false,
            device_cves: false,
        };
    }

    // ---------------------------------------------------------------------------
    // RED GATE TEST 2 — test_BC_2_06_019_stage_index_pure_function_reproducible
    //
    // BC-2.06.019 INV-PROGRESSION-REPRODUCIBILITY-001 / PC-3
    // Verifies: same (timeline, now_epoch_secs) → same stage from multiple calls.
    //
    // FAIL mode: with a timeline with default stages [0, 60, 180, 360, 600] and
    // now = start + 90, current_stage_index returns 0 (Baseline) but should be 1 (Recon).
    // ---------------------------------------------------------------------------
    #[test]
    fn test_BC_2_06_019_stage_index_pure_function_reproducible() {
        let org = deadbeef_org();
        let catalog = build_scenario_entity_catalog(42, &org);
        let start_secs: i64 = 1_000_000;

        let timeline = build_default_incident_timeline(catalog, start_secs, &[60, 180, 360, 600]);

        let now = start_secs + 90; // elapsed = 90s → should be stage 1 (Recon)

        let result1 = current_stage_index(&timeline, now);
        let result2 = current_stage_index(&timeline, now);
        let result3 = current_stage_index(&timeline, now);

        // Reproducibility: all calls return same value.
        assert_eq!(
            result1, result2,
            "current_stage_index must be reproducible: call 1 returned {result1}, call 2 returned {result2}"
        );
        assert_eq!(
            result2, result3,
            "current_stage_index must be reproducible: call 2 returned {result2}, call 3 returned {result3}"
        );

        // At elapsed=90s with stages [0, 60, 180, 360, 600], stage 1 (Recon) activates.
        // FAIL: stub returns 0.
        assert_eq!(
            result1, 1,
            "at elapsed=90s with default thresholds, stage must be 1 (Recon; activates at 60s); \
             got {result1} — BC-2.06.019 INV-PROGRESSION-REPRODUCIBILITY-001 / PC-3"
        );
    }

    // ---------------------------------------------------------------------------
    // RED GATE TEST 3 — test_BC_2_06_019_stage_boundary_5_thresholds_correct
    //
    // BC-2.06.019 PC-2, PC-3 / TV-019-001..005
    // Verifies: stage boundary correctness for all 6 canonical test vectors.
    //
    // FAIL mode: stub returns 0 for all inputs; TV-019-002..005 will FAIL.
    // ---------------------------------------------------------------------------
    #[test]
    fn test_BC_2_06_019_stage_boundary_5_thresholds_correct() {
        let org = deadbeef_org();
        let catalog = build_scenario_entity_catalog(42, &org);
        let start: i64 = 2_000_000;

        let timeline = build_default_incident_timeline(catalog, start, &[]);

        // TV-019-001: elapsed = 0s → stage 0 (Baseline)
        assert_eq!(
            current_stage_index(&timeline, start),
            0,
            "TV-019-001: at elapsed=0s, stage must be 0 (Baseline); got {}",
            current_stage_index(&timeline, start)
        );

        // TV-019-001b: elapsed = 30s → stage 0 (elapsed 30 < 60)
        assert_eq!(
            current_stage_index(&timeline, start + 30),
            0,
            "TV-019-001b: at elapsed=30s (< 60), stage must be 0 (Baseline); got {}",
            current_stage_index(&timeline, start + 30)
        );

        // TV-019-002: elapsed = 90s → stage 1 (Recon; elapsed 90 >= 60)
        // FAIL: stub returns 0.
        assert_eq!(
            current_stage_index(&timeline, start + 90),
            1,
            "TV-019-002: at elapsed=90s (>= 60), stage must be 1 (Recon); got {} \
             — BC-2.06.019 PC-2 / TV-019-002",
            current_stage_index(&timeline, start + 90)
        );

        // TV-019-003: elapsed = 200s → stage 2 (LateralMovement; >= 180)
        assert_eq!(
            current_stage_index(&timeline, start + 200),
            2,
            "TV-019-003: at elapsed=200s (>= 180), stage must be 2 (LateralMovement); got {} \
             — BC-2.06.019 PC-2 / TV-019-003",
            current_stage_index(&timeline, start + 200)
        );

        // TV-019-004: elapsed = 400s → stage 3 (Exfil; >= 360)
        assert_eq!(
            current_stage_index(&timeline, start + 400),
            3,
            "TV-019-004: at elapsed=400s (>= 360), stage must be 3 (Exfil); got {} \
             — BC-2.06.019 PC-2 / TV-019-004",
            current_stage_index(&timeline, start + 400)
        );

        // TV-019-005: elapsed = 700s → stage 4 (Containment; >= 600; saturates)
        assert_eq!(
            current_stage_index(&timeline, start + 700),
            4,
            "TV-019-005: at elapsed=700s (>= 600), stage must be 4 (Containment); got {} \
             — BC-2.06.019 PC-2 / TV-019-005",
            current_stage_index(&timeline, start + 700)
        );
    }

    // ---------------------------------------------------------------------------
    // RED GATE TEST 4 — test_BC_2_06_019_stage_index_monotonic_over_time
    //
    // BC-2.06.019 INV-STAGE-MONOTONICITY-001
    // Verifies: stage index never decreases over monotonically increasing time.
    //
    // FAIL mode: stub always returns 0 — the monotonicity assertion itself passes
    // (0 >= 0 is always true), but the final assertion that we eventually reach
    // stage 4 FAILS (we never advance beyond 0).
    // ---------------------------------------------------------------------------
    #[test]
    fn test_BC_2_06_019_stage_index_monotonic_over_time() {
        let org = deadbeef_org();
        let catalog = build_scenario_entity_catalog(42, &org);
        let start: i64 = 3_000_000;

        let timeline = build_default_incident_timeline(catalog, start, &[]);

        // Sample 50 time points spanning all stages.
        let time_points: Vec<i64> = (0..=700i64)
            .step_by(14)
            .map(|delta| start + delta)
            .collect();

        let mut prev_stage = 0usize;
        for now in &time_points {
            let stage = current_stage_index(&timeline, *now);
            assert!(
                stage >= prev_stage,
                "INV-STAGE-MONOTONICITY-001 violated: stage went from {prev_stage} to {stage} \
                 at now={now} (elapsed={}) — BC-2.06.019",
                now - start
            );
            prev_stage = stage;
        }

        // Verify we actually reached stage 4 (Containment) at end.
        // FAIL: stub always returns 0.
        let final_stage = current_stage_index(&timeline, start + 700);
        assert_eq!(
            final_stage, 4,
            "At elapsed=700s, stage must have reached 4 (Containment); got {final_stage} \
             — INV-STAGE-MONOTONICITY-001 requires advancement to all stages"
        );
    }

    // ---------------------------------------------------------------------------
    // RED GATE TEST 5 — test_BC_2_06_019_clock_skew_clamped_to_baseline
    //
    // BC-2.06.019 EC-019-003 / TV-019-006
    // Verifies: now < start → elapsed clamped to 0 → stage 0, no panic.
    //
    // FAIL mode: stub returns 0 which is correct for stage but the test also checks
    // a future elapsed > 0 returns > 0 which the stub fails.
    // ---------------------------------------------------------------------------
    #[test]
    fn test_BC_2_06_019_clock_skew_clamped_to_baseline() {
        let org = deadbeef_org();
        let catalog = build_scenario_entity_catalog(42, &org);
        let start: i64 = 1_500_000;

        let timeline = build_default_incident_timeline(catalog, start, &[]);

        // TV-019-006: clock skew — now is before start (future start)
        let skewed_now = start - 100; // 100 seconds before start
        let stage = current_stage_index(&timeline, skewed_now);
        assert_eq!(
            stage, 0,
            "TV-019-006: clock-skewed now (now={skewed_now} < start={start}) must return \
             stage 0 (Baseline); got {stage}. elapsed = max(0, now-start) must clamp to 0. \
             BC-2.06.019 EC-019-003"
        );

        // Extreme past — should still return 0, not panic.
        let extreme_past = i64::MIN / 2;
        let extreme_stage = current_stage_index(&timeline, extreme_past);
        assert_eq!(
            extreme_stage, 0,
            "Extreme past clock skew must return stage 0 without panic; got {extreme_stage}"
        );

        // Validate that elapsed > threshold does advance the stage (proving the clamp is
        // conditional, not always-zero). FAIL: stub always returns 0.
        let future_now = start + 90;
        let future_stage = current_stage_index(&timeline, future_now);
        assert_eq!(
            future_stage, 1,
            "At elapsed=90s (> 60s threshold), stage must be 1 (Recon); got {future_stage} \
             — this validates the clamp does not suppress all advancement"
        );
    }

    // ---------------------------------------------------------------------------
    // RED GATE TEST 6 — test_BC_2_06_019_stage_mask_completeness_all_6_fields
    //
    // BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001 / PC-2 table
    // Verifies: each of the 5 stages has all 6 StageMask bool fields explicitly set
    // per the BC-2.06.019 §Postcondition 2 canonical table.
    //
    // FAIL mode: stub builds only 1 stage (Baseline), so index access at stages[1..4]
    // will panic (index out of bounds). The test will FAIL (panic = test failure).
    // ---------------------------------------------------------------------------
    #[test]
    fn test_BC_2_06_019_stage_mask_completeness_all_6_fields() {
        let org = deadbeef_org();
        let catalog = build_scenario_entity_catalog(42, &org);
        let timeline = build_default_incident_timeline(catalog, 0, &[]);

        // Require at least 5 stages to avoid panic; stub only has 1 → will panic = FAIL.
        assert_eq!(
            timeline.stages.len(),
            5,
            "Timeline must have exactly 5 stages for mask completeness check; got {}",
            timeline.stages.len()
        );

        // Stage 0 (Baseline): primary_device=true; all others false.
        let m0 = &timeline.stages[0].visible_entity_mask;
        assert!(
            m0.primary_device,
            "Stage 0 Baseline: primary_device must be true"
        );
        assert!(
            !m0.lateral_devices,
            "Stage 0 Baseline: lateral_devices must be false"
        );
        assert!(!m0.ioc_ips, "Stage 0 Baseline: ioc_ips must be false");
        assert!(
            !m0.ioc_domains,
            "Stage 0 Baseline: ioc_domains must be false"
        );
        assert!(!m0.ioc_hashes, "Stage 0 Baseline: ioc_hashes must be false");
        assert!(
            !m0.device_cves,
            "Stage 0 Baseline: device_cves must be false"
        );

        // Stage 1 (Recon): primary_device=true; rest false.
        let m1 = &timeline.stages[1].visible_entity_mask;
        assert!(
            m1.primary_device,
            "Stage 1 Recon: primary_device must be true"
        );
        assert!(
            !m1.lateral_devices,
            "Stage 1 Recon: lateral_devices must be false"
        );
        assert!(!m1.ioc_ips, "Stage 1 Recon: ioc_ips must be false");
        assert!(!m1.ioc_domains, "Stage 1 Recon: ioc_domains must be false");
        assert!(!m1.ioc_hashes, "Stage 1 Recon: ioc_hashes must be false");
        assert!(!m1.device_cves, "Stage 1 Recon: device_cves must be false");

        // Stage 2 (LateralMovement): primary_device=true; lateral_devices=true; ioc_hashes=true; rest false.
        let m2 = &timeline.stages[2].visible_entity_mask;
        assert!(
            m2.primary_device,
            "Stage 2 LateralMovement: primary_device must be true"
        );
        assert!(
            m2.lateral_devices,
            "Stage 2 LateralMovement: lateral_devices must be true"
        );
        assert!(
            !m2.ioc_ips,
            "Stage 2 LateralMovement: ioc_ips must be false"
        );
        assert!(
            !m2.ioc_domains,
            "Stage 2 LateralMovement: ioc_domains must be false"
        );
        assert!(
            m2.ioc_hashes,
            "Stage 2 LateralMovement: ioc_hashes must be true"
        );
        assert!(
            !m2.device_cves,
            "Stage 2 LateralMovement: device_cves must be false"
        );

        // Stage 3 (Exfil): primary + lateral + ioc_ips + ioc_domains + ioc_hashes; device_cves=false.
        let m3 = &timeline.stages[3].visible_entity_mask;
        assert!(
            m3.primary_device,
            "Stage 3 Exfil: primary_device must be true"
        );
        assert!(
            m3.lateral_devices,
            "Stage 3 Exfil: lateral_devices must be true"
        );
        assert!(m3.ioc_ips, "Stage 3 Exfil: ioc_ips must be true");
        assert!(m3.ioc_domains, "Stage 3 Exfil: ioc_domains must be true");
        assert!(m3.ioc_hashes, "Stage 3 Exfil: ioc_hashes must be true");
        assert!(!m3.device_cves, "Stage 3 Exfil: device_cves must be false");

        // Stage 4 (Containment): all 6 fields true.
        let m4 = &timeline.stages[4].visible_entity_mask;
        assert!(
            m4.primary_device,
            "Stage 4 Containment: primary_device must be true"
        );
        assert!(
            m4.lateral_devices,
            "Stage 4 Containment: lateral_devices must be true"
        );
        assert!(m4.ioc_ips, "Stage 4 Containment: ioc_ips must be true");
        assert!(
            m4.ioc_domains,
            "Stage 4 Containment: ioc_domains must be true"
        );
        assert!(
            m4.ioc_hashes,
            "Stage 4 Containment: ioc_hashes must be true"
        );
        assert!(
            m4.device_cves,
            "Stage 4 Containment: device_cves must be true"
        );
    }

    // ---------------------------------------------------------------------------
    // RED GATE TEST 12 — test_BC_2_06_019_secondary_rng_independence_no_primary_shift
    //
    // BC-2.06.019 INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 / PC-1
    // Verifies: the secondary RNG stream (seed.wrapping_add(1), org_id) used for
    // catalog derivation does NOT consume state from the primary generator stream.
    // Two catalogs built from the same (seed, org_id) must be identical.
    //
    // NOTE: This is a property test — catalog determinism is what we can verify
    // at the unit level. The full independence test (that fixture records are byte-
    // identical between scenario-enabled and non-scenario paths) requires the clone
    // constructors and is covered by the harness integration tests.
    //
    // FAIL mode: this test should PASS (catalog is deterministic). However, the
    // broader independence invariant (no primary shift) will be validated by the
    // implementer's integration test in the demo-server harness after implementation.
    // We write this as a compile-and-run guard that confirms secondary RNG determinism.
    // The test is designed to FAIL if the secondary stream is non-deterministic.
    // ---------------------------------------------------------------------------
    #[test]
    fn test_BC_2_06_019_secondary_rng_independence_no_primary_shift() {
        let org = deadbeef_org();
        let seed: u64 = 100;

        // Build two catalogs with the same (seed, org_id) — must be identical.
        let catalog1 = build_scenario_entity_catalog(seed, &org);
        let catalog2 = build_scenario_entity_catalog(seed, &org);

        assert_eq!(
            catalog1.ioc_ips, catalog2.ioc_ips,
            "build_scenario_entity_catalog must be deterministic: same (seed, org_id) \
             must produce identical ioc_ips (INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001)"
        );
        assert_eq!(
            catalog1.ioc_domains, catalog2.ioc_domains,
            "build_scenario_entity_catalog must produce identical ioc_domains"
        );
        assert_eq!(
            catalog1.device_cves, catalog2.device_cves,
            "build_scenario_entity_catalog must produce identical device_cves"
        );
        assert_eq!(
            catalog1.primary_device_id_cs, catalog2.primary_device_id_cs,
            "build_scenario_entity_catalog must produce identical primary_device_id_cs"
        );

        // Cross-seed: different seeds must produce different catalogs (independence).
        let catalog_other = build_scenario_entity_catalog(seed.wrapping_add(1), &org);
        assert_ne!(
            catalog1.ioc_ips, catalog_other.ioc_ips,
            "Different seeds must produce different ioc_ips \
             (secondary RNG stream independence)"
        );

        // Verify the secondary stream uses seed.wrapping_add(1): seed=u64::MAX should
        // not panic (wraps to 0). EC-011: gen_seeded_rng(0, &org_id) is valid.
        let max_seed_catalog = build_scenario_entity_catalog(u64::MAX, &org);
        assert!(
            !max_seed_catalog.ioc_ips.is_empty(),
            "seed=u64::MAX: secondary stream (wrapping_add(1) = 0) must still produce \
             non-empty ioc_ips — EC-011 / ADR-036 v2.2"
        );

        // The deeper test — primary generator stream independence — requires the
        // new_with_scenario constructor to be implemented (Story B implementation).
        // The clone-level test will verify that fixture records from scenario-enabled
        // and disabled paths are byte-identical. For now, we confirm catalog determinism
        // as the necessary (but not sufficient) condition.
        // FAIL: the above assertions pass with the stub, but the integration-level
        // test is in test 11 (demo-server unit test) which will FAIL.
    }

    /// RG-1: test_BC_2_06_018_scenario_catalog_secondary_rng_and_canonical_ids
    ///
    /// Traces to: BC-2.06.018 precondition 4 / ADR-036 §2.2
    /// Verifies:
    /// - org_slug = "deadbeef" for deadbeef_org()
    /// - primary_device_id_cs = "dev-deadbeef-42-0"
    /// - primary_device_id_armis = "dev-deadbeef-42-0"
    /// - ioc_ips, ioc_domains, ioc_hashes, device_cves are all non-empty
    /// - secondary RNG stream is independent (catalog fields populated from
    ///   gen_seeded_rng(seed.wrapping_add(1), &org_id), not from seed)
    #[test]
    fn test_BC_2_06_018_scenario_catalog_secondary_rng_and_canonical_ids() {
        let org = deadbeef_org();
        let seed: u64 = 42;

        let catalog = build_scenario_entity_catalog(seed, &org);

        // Org slug must be canonical 8-hex chars derived from first 4 bytes of org_id.
        assert_eq!(
            catalog.org_slug, "deadbeef",
            "org_slug must be 'deadbeef' for org bytes [0xde, 0xad, 0xbe, 0xef, ...]; \
             got '{}'. ADR-036 §2.2 formula: hex(org_id.as_bytes()[0..4])",
            catalog.org_slug
        );

        // Primary device ID — CrowdStrike format: "dev-{org_slug}-{seed}-0"
        assert_eq!(
            catalog.primary_device_id_cs, "dev-deadbeef-42-0",
            "primary_device_id_cs must be 'dev-deadbeef-42-0' for org_slug='deadbeef', seed=42; \
             got '{}'. ADR-036 §2.2 canonical format: dev-{{org_slug}}-{{seed}}-{{n}}",
            catalog.primary_device_id_cs
        );

        // Primary device ID — Armis format: same formula
        assert_eq!(
            catalog.primary_device_id_armis,
            "dev-deadbeef-42-0",
            "primary_device_id_armis must be 'dev-deadbeef-42-0' for org_slug='deadbeef', seed=42; \
             got '{}'",
            catalog.primary_device_id_armis
        );

        // Secondary RNG-derived fields must be non-empty (derived from secondary stream)
        assert!(
            !catalog.ioc_ips.is_empty(),
            "ioc_ips must be non-empty — derived from secondary RNG stream \
             gen_seeded_rng(seed.wrapping_add(1), &org_id); ADR-036 §2.2"
        );
        assert!(
            !catalog.ioc_domains.is_empty(),
            "ioc_domains must be non-empty — derived from secondary RNG stream"
        );
        assert!(
            !catalog.ioc_hashes.is_empty(),
            "ioc_hashes must be non-empty — derived from secondary RNG stream"
        );
        assert!(
            !catalog.device_cves.is_empty(),
            "device_cves must be non-empty — derived from secondary RNG stream"
        );

        // Determinism: same inputs → same catalog (BC-3.4.001 postcondition 3)
        let catalog2 = build_scenario_entity_catalog(seed, &org);
        assert_eq!(
            catalog.ioc_ips, catalog2.ioc_ips,
            "build_scenario_entity_catalog must be deterministic: same (seed, org_id) \
             must produce identical ioc_ips on repeated calls (BC-3.4.001 PC-3)"
        );

        // Different seeds → different secondary RNG output (independence)
        let catalog_other_seed = build_scenario_entity_catalog(seed + 1, &org);
        assert_ne!(
            catalog.ioc_ips, catalog_other_seed.ioc_ips,
            "different seeds must produce different ioc_ips (secondary stream independence)"
        );
    }

    /// RG-2: test_BC_2_06_018_org_slug_from_org_id_canonical_format
    ///
    /// Traces to: BC-2.06.018 §Canonical Org Slug / ADR-036 §2.2
    /// Verifies:
    /// - org_slug_from_org_id returns "deadbeef" for deadbeef_org()
    /// - result is exactly 8 characters
    /// - all characters are in [0-9a-f]
    /// - formula is consistent with CrowdStrike generator's internal org_slug()
    #[test]
    fn test_BC_2_06_018_org_slug_from_org_id_canonical_format() {
        let org = deadbeef_org();

        let slug = org_slug_from_org_id(&org);

        // Golden test vector: [0xde, 0xad, 0xbe, 0xef, ...] → "deadbeef"
        assert_eq!(
            slug, "deadbeef",
            "org_slug_from_org_id must return 'deadbeef' for org bytes \
             [0xde, 0xad, 0xbe, 0xef, ...]; got '{}'. \
             Formula: hex(org_id.as_bytes()[0..4]) — ADR-036 §2.2",
            slug
        );

        // Length invariant: always exactly 8 characters
        assert_eq!(
            slug.len(),
            8,
            "org_slug_from_org_id result must be exactly 8 characters; got {} for '{}'",
            slug.len(),
            slug
        );

        // Character set invariant: only [0-9a-f] (lowercase hex)
        assert!(
            slug.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "org_slug_from_org_id result must contain only lowercase hex chars [0-9a-f]; \
             got '{}' — uppercase chars are forbidden (ADR-036 §2.2)",
            slug
        );

        // All-zeros org → "00000000"
        let zero_org = OrgId([0u8; 16]);
        let zero_slug = org_slug_from_org_id(&zero_org);
        assert_eq!(
            zero_slug, "00000000",
            "org_slug_from_org_id must return '00000000' for all-zero OrgId; got '{}'",
            zero_slug
        );

        // All-ones (0xff) first 4 bytes → "ffffffff"
        let ff_org = OrgId([0xff, 0xff, 0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let ff_slug = org_slug_from_org_id(&ff_org);
        assert_eq!(
            ff_slug, "ffffffff",
            "org_slug_from_org_id must return 'ffffffff' for org bytes [0xff, 0xff, 0xff, 0xff, ...]; \
             got '{}'",
            ff_slug
        );

        // Arbitrary org: [0x01, 0x23, 0x45, 0x67, ...] → "01234567"
        let arbitrary_org = OrgId([0x01, 0x23, 0x45, 0x67, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let arbitrary_slug = org_slug_from_org_id(&arbitrary_org);
        assert_eq!(
            arbitrary_slug, "01234567",
            "org_slug_from_org_id must return '01234567' for org bytes [0x01, 0x23, 0x45, 0x67, ...]; \
             got '{}'",
            arbitrary_slug
        );

        // Verify only first 4 bytes matter (bytes 4+ are ignored)
        let org_a = OrgId([
            0xca, 0xfe, 0xba, 0xbe, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a,
            0x0b, 0x0c,
        ]);
        let org_b = OrgId([
            0xca, 0xfe, 0xba, 0xbe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        ]);
        assert_eq!(
            org_slug_from_org_id(&org_a),
            org_slug_from_org_id(&org_b),
            "org_slug_from_org_id must only use first 4 bytes; \
             different bytes 4-15 with same bytes 0-3 must yield the same slug"
        );
    }
}
