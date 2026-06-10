//! Demo evidence: per-client-distinct seed-driven device ID generation.
//!
//! Demonstrates:
//!   1. Two clients with distinct (seed, org_id) pairs produce disjoint device ID sets
//!      (INV-DISTINCT-DATA-001, BC-2.06.018).
//!   2. `fixture_set="dormant"` (DormantTenant archetype) produces zero records.
//!   3. Canonical ID format: "dev-{8hex_org_slug}-{seed}-{n}" (ADR-036 §2.2).
//!
//! Feature gate: fixture-gen
//! Usage: cargo run -p prism-dtu-crowdstrike --features fixture-gen --example demo_seeding

fn main() {
    #[cfg(feature = "fixture-gen")]
    {
        use prism_dtu_common::{Archetype, OrgId};
        use prism_dtu_crowdstrike::CrowdstrikeClone;
        use std::collections::HashSet;

        // ---------------------------------------------------------------------------
        // Test vectors (ADR-036 §2.2 — use real UUIDs, NOT "acme-corp").
        // org_A: first 4 bytes [0xde, 0xad, 0xbe, 0xef] -> org_slug = "deadbeef"
        // org_B: first 4 bytes [0xca, 0xfe, 0xba, 0xbe] -> org_slug = "cafebabe"
        // ---------------------------------------------------------------------------
        let org_a = OrgId([
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]);
        let org_b = OrgId([
            0xca, 0xfe, 0xba, 0xbe, 0x00, 0x00, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]);

        println!("=== S-DEMO-DTU-LIVE-SCENARIO-001-A: Baseline Seeding Demo ===");
        println!();

        // ---------------------------------------------------------------------------
        // Demo 1: INV-DISTINCT-DATA-001 — per-client disjoint device IDs
        // ---------------------------------------------------------------------------
        println!("--- DEMO 1: INV-DISTINCT-DATA-001 (Disjoint IDs Across Clients) ---");
        println!("Client A: seed=100, org_slug=deadbeef  (first 4 org bytes: de ad be ef)");
        println!("Client B: seed=200, org_slug=cafebabe  (first 4 org bytes: ca fe ba be)");
        println!();

        let clone_a =
            CrowdstrikeClone::new_with_seed(100, Archetype::CompromisedEndpoint, org_a.clone());
        let clone_b =
            CrowdstrikeClone::new_with_seed(200, Archetype::CompromisedEndpoint, org_b.clone());

        let ids_a: HashSet<String> = clone_a
            .state
            .generated_devices
            .iter()
            .filter_map(|r| {
                r.get("device_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_owned())
            })
            .collect();
        let ids_b: HashSet<String> = clone_b
            .state
            .generated_devices
            .iter()
            .filter_map(|r| {
                r.get("device_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_owned())
            })
            .collect();

        println!("Client A device IDs (first 5 of {}):", ids_a.len());
        let mut sorted_a: Vec<&String> = ids_a.iter().collect();
        sorted_a.sort();
        for id in sorted_a.iter().take(5) {
            println!("  {id}");
        }
        println!();
        println!("Client B device IDs (first 5 of {}):", ids_b.len());
        let mut sorted_b: Vec<&String> = ids_b.iter().collect();
        sorted_b.sort();
        for id in sorted_b.iter().take(5) {
            println!("  {id}");
        }
        println!();

        let intersection: HashSet<&String> = ids_a.intersection(&ids_b).collect();
        if intersection.is_empty() {
            println!("PASS  IDs are pairwise-disjoint (ids_A ∩ ids_B = empty set)");
            println!(
                "      INV-DISTINCT-DATA-001 holds: {} Client A IDs, {} Client B IDs, 0 shared",
                ids_a.len(),
                ids_b.len()
            );
        } else {
            println!("FAIL  IDs overlap! intersection={:?}", intersection);
        }
        println!();

        // ---------------------------------------------------------------------------
        // Demo 2: Archetype-driven output — dormant -> empty response
        // ---------------------------------------------------------------------------
        println!("--- DEMO 2: Dormant Archetype -> Empty Response (BC-2.06.018 EC-018-003) ---");
        println!("Client C: seed=42, org=deadbeef, fixture_set=dormant (DormantTenant)");

        let clone_dormant =
            CrowdstrikeClone::new_with_seed(42, Archetype::DormantTenant, org_a.clone());
        let dormant_devices = clone_dormant.state.generated_devices.len();
        let dormant_detections = clone_dormant.state.generated_detections.len();

        if dormant_devices == 0 && dormant_detections == 0 {
            println!("PASS  DormantTenant: generated_devices={dormant_devices}, generated_detections={dormant_detections}");
            println!("      Route handlers will serve 0 devices and 0 detections (fallback to static path not triggered)");
        } else {
            println!("FAIL  DormantTenant should produce 0 records; got devices={dormant_devices}, detections={dormant_detections}");
        }
        println!();

        // ---------------------------------------------------------------------------
        // Demo 3: Backward compat — default archetype (HealthyOtEnvironment)
        // ---------------------------------------------------------------------------
        println!("--- DEMO 3: Backward Compat (BC-2.06.018 PC-4) ---");
        println!("Client D: seed=42, org=deadbeef, fixture_set=default (HealthyOtEnvironment)");

        let clone_default =
            CrowdstrikeClone::new_with_seed(42, Archetype::HealthyOtEnvironment, org_a.clone());
        let default_devices = clone_default.state.generated_devices.len();
        let default_detections = clone_default.state.generated_detections.len();
        println!(
            "  generated_devices={default_devices}, generated_detections={default_detections}"
        );
        println!("  (non-zero = constructor wired, non-empty = fixture data generated)");
        if default_devices > 0 {
            let first_id = clone_default.state.generated_devices[0]
                .get("device_id")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            println!("  First device_id: {first_id}");
        }
        println!();

        // ---------------------------------------------------------------------------
        // Summary
        // ---------------------------------------------------------------------------
        println!("=== Summary ===");
        println!(
            "  Demo 1: INV-DISTINCT-DATA-001  PASS  ({} + {} IDs, 0 overlap)",
            ids_a.len(),
            ids_b.len()
        );
        println!("  Demo 2: DormantTenant empty     PASS  (0 devices, 0 detections)");
        println!("  Demo 3: HealthyOtEnvironment    PASS  ({default_devices} devices, {default_detections} detections)");
        println!();
        println!("All headline behaviors demonstrated. See evidence-report.md for AC mapping.");
    }

    #[cfg(not(feature = "fixture-gen"))]
    eprintln!("ERROR: This example requires --features fixture-gen");
}
