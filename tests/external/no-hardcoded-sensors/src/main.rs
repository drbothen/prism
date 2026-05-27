// ADR-023 Rule 3 compile-fail test.
// PLUGIN-MIGRATION-001-F AC-006: no-hardcoded-sensors compile-fail perimeter.
//
// These symbols were DELETED in PLUGIN-MIGRATION-001-A.
// This file MUST NOT compile successfully.
// CI job `no-hardcoded-sensors-compile-fail` asserts non-zero exit code.
//
// If `cargo check -p no-hardcoded-sensors` ever exits 0, a deleted sensor-named
// symbol has been accidentally re-exported — this is an ADR-023 Rule 3 violation.
//
// Expected compiler error: E0432 "unresolved import"
// for each deleted auth module path below.
use prism_sensors::auth::armis::ArmisAuth; // DELETED in PLUGIN-MIGRATION-001-A
use prism_sensors::auth::claroty::ClarotyAuth; // DELETED in PLUGIN-MIGRATION-001-A
use prism_sensors::auth::crowdstrike::CrowdStrikeAuth; // DELETED in PLUGIN-MIGRATION-001-A
use prism_sensors::auth::cyberint::CyberintAuth; // DELETED in PLUGIN-MIGRATION-001-A

fn main() {
    // These calls are unreachable due to compile failure, but are written
    // to prevent the compiler from eliding the use statements via dead-code
    // elimination before the import resolution fires.
    let _: Option<ArmisAuth> = None;
    let _: Option<ClarotyAuth> = None;
    let _: Option<CrowdStrikeAuth> = None;
    let _: Option<CyberintAuth> = None;
}
