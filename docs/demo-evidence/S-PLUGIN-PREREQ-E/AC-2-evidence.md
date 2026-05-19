AC-2 — Four Built-In Auth Impls: One New Method Body Each
==========================================================
Story: S-PLUGIN-PREREQ-E (v1.50) | BC: BC-2.01.016 | HEAD: 051eab95

EVIDENCE TYPE: Source excerpts (4 impl files) + test output

-------------------------------------------------------------------------------
SOURCE EXCERPTS: auth_type_name method bodies in 4 concrete impls
-------------------------------------------------------------------------------

crates/prism-sensors/src/auth/crowdstrike.rs (line 62):
    fn auth_type_name(&self) -> &'static str {
        "oauth2_client_credentials"
    }

crates/prism-sensors/src/auth/cyberint.rs (line 57):
    fn auth_type_name(&self) -> &'static str {
        "bearer_static"
    }

crates/prism-sensors/src/auth/claroty.rs (line 63):
    fn auth_type_name(&self) -> &'static str {
        "cookie_roundtrip"
    }

crates/prism-sensors/src/auth/armis.rs (line 59):
    fn auth_type_name(&self) -> &'static str {
        "api_key"
    }

Each impl adds EXACTLY ONE method body (`auth_type_name`) returning the canonical ADR-026
§D3 auth-type string. No other changes to the `impl SensorAuth for X` blocks.

-------------------------------------------------------------------------------
TEST OUTPUT: test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing
-------------------------------------------------------------------------------

Command: cargo nextest run -p prism-sensors -E 'test(BC_2_01_016)' --no-fail-fast

    Starting 2 tests across 12 binaries (267 tests skipped)
        PASS [   0.025s] (1/2) prism-sensors auth::tests::test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing
        PASS [   0.028s] (2/2) prism-sensors auth::tests::test_BC_2_01_016_001_sensor_auth_external_impl_compiles
    Summary [   0.028s] 2 tests run: 2 passed, 267 skipped

AUTH-TYPE NAME MAPPING (ADR-026 §D3 canonical set):
  CrowdStrikeAuth  → "oauth2_client_credentials"
  CyberintAuth     → "bearer_static"
  ClarotyAuth      → "cookie_roundtrip"
  ArmisAuth        → "api_key"

RESULT: PASS — All four built-in auth impls compile with exactly one new method body.
`cargo build -p prism-sensors` exits 0 with zero warnings (verified by just check).
BC-2.01.016 postcondition satisfied. INV-AUTH-OPEN-002 holds.
