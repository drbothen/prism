AC-1 — SensorAuth Sealed Marker Removed
========================================
Story: S-PLUGIN-PREREQ-E (v1.50) | BC: BC-2.01.016 | HEAD: 051eab95

EVIDENCE TYPE: Code inspection (grep for sealed-marker absence) + trait definition excerpt

-------------------------------------------------------------------------------
GREP: Search for sealed marker patterns in crates/prism-sensors/src/auth/
-------------------------------------------------------------------------------

Command: grep -rn 'sealed_marker|impl Sealed|private::Sealed|: Sealed\b' crates/prism-sensors/src/auth/

Output: (no matches — exit 0)

RESULT: ZERO matches. No sealed marker, private::Sealed supertrait, or `impl Sealed`
block exists anywhere in crates/prism-sensors/src/auth/.

-------------------------------------------------------------------------------
SOURCE EXCERPT: crates/prism-sensors/src/auth/mod.rs — SensorAuth trait definition
-------------------------------------------------------------------------------

/// Open authentication credential trait for a sensor adapter.
///
/// As of S-PLUGIN-PREREQ-E (BC-2.01.016 + ADR-026), the sealed marker has been
/// removed. External crates (including `.prx` WASM plugins) may implement this
/// trait to register custom auth strategies. Runtime cross-composition rules
/// (E-SPEC-012/013/014) enforce safe usage at spec-load time (ADR-023 Rule 2).
///
/// Each auth subtype carries ONLY its own credentials (no field overlap across
/// sensor types). Credentials MUST NOT appear in `Debug` output or log output
/// at any level (AI-opaque credential model).
///
/// Story: S-2.06 (initial) | S-PLUGIN-PREREQ-E (unsealing) | BC: BC-2.01.013, BC-2.01.016
pub trait SensorAuth: Send + Sync + 'static {
    fn as_any(&self) -> &dyn std::any::Any;
    fn auth_type_name(&self) -> &'static str;
}

(Trait has NO `private::Sealed` supertrait bound — it is `pub` and externally implementable.)

-------------------------------------------------------------------------------
TEST OUTPUT: test_BC_2_01_016_001_sensor_auth_external_impl_compiles (prism-sensors)
-------------------------------------------------------------------------------

    Starting 2 tests across 12 binaries (267 tests skipped)
        PASS [   0.025s] (1/2) prism-sensors auth::tests::test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing
        PASS [   0.028s] (2/2) prism-sensors auth::tests::test_BC_2_01_016_001_sensor_auth_external_impl_compiles
    Summary [   0.028s] 2 tests run: 2 passed, 267 skipped

RESULT: PASS — SensorAuth is publicly implementable (external impl compiles successfully).
The sealed marker is absent. BC-2.01.016 postcondition satisfied.
