#![allow(non_snake_case, clippy::unwrap_used)]
//! VP-153: SensorAuth Runtime Cross-Composition Prevention — Rule C (E-SPEC-014)
//! via ShapedProbe injection through step5_init_credential_store_with_probe.
//!
//! ## Rule C Backend Scope (D-706 amendment)
//!
//! Per ADR-026 §D3 Rule C Backend Scope (D-706, 2026-05-18): Rule C enforcement is
//! conditional on the credential backend providing shape metadata via
//! `CredentialRefProbe::probe()` returning `Some(shape)`. The production keyring
//! backend (`KeyringCredentialProbe`) returns `Ok(None)` — no shape metadata stored.
//! Production enforcement is deferred to PLUGIN-MIGRATION-001-A.
//!
//! In PREREQ-E scope, Rule C is enforced via the `ShapedProbe` test fixture, which
//! returns `Ok(Some(reported_auth_type))` for every probe call. This file exercises
//! the full `step5_init_credential_store_with_probe` production code path with
//! `ShapedProbe` — the architecturally-sanctioned test path (ADR-026 §D3 (a)).
//!
//! ## Location rationale
//!
//! This proptest lives in `crates/prism-bin/tests/` rather than
//! `crates/prism-spec-engine/tests/` because `prism-bin` depends on
//! `prism-spec-engine`. Adding `prism-bin` as a dev-dep to `prism-spec-engine`
//! would create a workspace cycle. The `step5_init_credential_store_with_probe`
//! function and `CredentialRefProbe` trait are both defined in `prism-bin::boot`.
//!
//! ## Test inventory
//!
//! | Prop | Name | Postcondition |
//! |------|------|---------------|
//! | 1 | prop_rule_c_shape_mismatch_rejected_via_shaped_probe | E-SPEC-014 Err |
//! | 2 | prop_rule_c_shape_match_accepted_via_shaped_probe | Ok(store) |
//!
//! PROPTEST_CASES: respects env var (32 for `just iter`, 256 for `just check`).
//!
//! Story: S-PLUGIN-PREREQ-E AC-3c | BC-2.01.016 Rule C | ADR-026 §D3 Rule 3 | ADR-023 Rule 2
//! VP: VP-153 (SensorAuth Runtime Cross-Composition Prevention)
//! Fixes: F-LP-IMPL-P8-IMP-001 (pass-8 blind spot — VP-153 P0 artifact never authored)

use std::sync::Arc;

use arc_swap::ArcSwap;
use prism_bin::{
    BootError,
    boot::{CredentialBackendConfig, CredentialRefProbe, OrgEntry, PrismConfig},
};
use prism_spec_engine::config_manager::{ConfigManager, parse_spec_directory};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Canonical valid auth_type enumeration (ADR-026 §D3 Rule 1 / E-SPEC-012).
const VALID_AUTH_TYPES: &[&str] = &[
    "oauth2_client_credentials",
    "bearer_static",
    "cookie_roundtrip",
    "api_key",
    "custom_via_plugin",
];

// ---------------------------------------------------------------------------
// ShapedProbe — test fixture that returns a specific auth_type shape for all refs
// ---------------------------------------------------------------------------

/// Test double for `CredentialRefProbe` that returns a fixed shape string.
///
/// When `reported_shape == sensor spec's auth_type` → Rule C passes (returns Ok).
/// When `reported_shape != sensor spec's auth_type` → Rule C fires (returns
/// `Err(BootError::AuthTypeCredentialMismatch)`).
///
/// The probe returns `Ok(Some(shape))`, triggering the Rule C comparison gate in
/// `step5_init_credential_store_with_probe`. This is the production code path that
/// is unreachable via the keyring backend (which returns `Ok(None)`).
///
/// AD-017: ShapedProbe uses ONLY auth_type shape strings — not credential VALUES.
/// The `reported_shape` field is always one of the canonical auth_type identifiers
/// ("oauth2_client_credentials", "api_key", etc.), never a password or token.
struct ShapedProbe {
    /// The auth_type shape this probe reports for all credential ref calls.
    /// Must be a logical auth_type identifier, never a credential value (AD-017).
    reported_shape: String,
}

impl CredentialRefProbe for ShapedProbe {
    fn probe(
        &self,
        _sensor_id: &str,
        _ref_name: &str,
        _org_registry: &prism_core::OrgRegistry,
    ) -> Result<Option<String>, BootError> {
        // Returns Some(shape) — activates the Rule C comparison gate in step5.
        Ok(Some(self.reported_shape.clone()))
    }
}

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

/// Write a minimal sensor TOML with the given auth_type and return a parsed
/// ConfigManager wrapped in Arc<ArcSwap<...>> for injection into step5.
///
/// The sensor has exactly 1 credential_ref named "cred" (Rule B passes).
/// The auth_type must be in the valid enumeration (Rule A passes).
/// No `[[tables]]` section is needed — a spec with no tables is valid per
/// `parse_and_validate_spec_toml` (only required fields: sensor_id, name,
/// version, auth_type, base_url).
fn make_config_manager_for_auth_type(
    spec_dir: &std::path::Path,
    sensor_id: &str,
    auth_type: &str,
) -> Arc<ArcSwap<ConfigManager>> {
    // ADR-030 Approach D: SpecLoader::parse uses flat top-level TOML format (no [sensor] section).
    let sensor_toml = format!(
        r#"
sensor_id = "{sensor_id}"
name = "VP153 Test Sensor"
version = "0.1.0"
auth_type = "{auth_type}"
base_url = "https://sensor.example.com"

[[credential_refs]]
name = "cred"
"#,
        sensor_id = sensor_id,
        auth_type = auth_type,
    );

    let file_name = format!("{sensor_id}.sensor.toml", sensor_id = sensor_id);
    std::fs::write(spec_dir.join(&file_name), &sensor_toml)
        .expect("fixture sensor TOML write must succeed");

    let snapshot =
        parse_spec_directory(spec_dir).expect("parse_spec_directory must succeed for valid spec");

    Arc::new(ArcSwap::from_pointee(ConfigManager::new(snapshot)))
}

/// Build a minimal PrismConfig pointing at the given spec_dir.
/// Returns (config, state_tmp) — caller must keep state_tmp alive.
fn make_config(spec_dir: &std::path::Path) -> (PrismConfig, tempfile::TempDir) {
    let state_tmp = tempfile::TempDir::new().unwrap();
    let config = PrismConfig::new_for_test(
        spec_dir.to_path_buf(),
        state_tmp.path().to_path_buf(),
        "plugins",
        vec![OrgEntry::new(
            "0196f000-0000-7000-8000-000000000001",
            "acme",
        )],
        CredentialBackendConfig::Keyring,
    );
    (config, state_tmp)
}

// ---------------------------------------------------------------------------
// Strategy
// ---------------------------------------------------------------------------

/// Strategy: a pair (spec_auth_type, probe_auth_type) where the two differ.
/// Used for Rule C mismatch tests.
///
/// Generates one of the 5 valid auth_types for the spec and one of the REMAINING
/// 4 for the probe — guaranteeing a mismatch. This covers all 20 ordered pairs
/// (5 × 4) from the Cartesian product exhaustively within the bounded strategy.
fn arb_mismatched_auth_type_pair() -> impl Strategy<Value = (&'static str, &'static str)> {
    (0usize..5, 0usize..4).prop_map(|(spec_idx, offset)| {
        let spec_type = VALID_AUTH_TYPES[spec_idx];
        // Pick any valid type except the spec_type
        let probe_idx = {
            let mut i = 0;
            let mut count = 0;
            for (idx, t) in VALID_AUTH_TYPES.iter().enumerate() {
                if *t == spec_type {
                    continue;
                }
                if count == offset {
                    i = idx;
                    break;
                }
                count += 1;
            }
            i
        };
        (spec_type, VALID_AUTH_TYPES[probe_idx])
    })
}

/// Strategy: one of the 5 valid auth_types for both spec and probe (matching pair).
fn arb_matching_auth_type() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("oauth2_client_credentials"),
        Just("bearer_static"),
        Just("cookie_roundtrip"),
        Just("api_key"),
        Just("custom_via_plugin"),
    ]
}

// ---------------------------------------------------------------------------
// Rule C proptests
// ---------------------------------------------------------------------------

proptest! {
    /// VP-153 prop 1 of 2 (Rule C): ShapedProbe reports mismatched shape →
    /// step5_init_credential_store_with_probe returns Err(AuthTypeCredentialMismatch).
    ///
    /// Exercises BC-2.01.016 §Error Cases E-SPEC-014:
    ///   "credential type '{credential_type}' is incompatible with auth_type
    ///    '{auth_type}' for sensor '{sensor_id}'"
    ///
    /// The ShapedProbe returns Ok(Some(probe_shape)) where probe_shape != spec_auth_type.
    /// The Rule C gate in step5 (`if actual_shape != expected_shape`) fires and returns
    /// Err(BootError::AuthTypeCredentialMismatch).
    ///
    /// Per D-706: this exercises the structurally-correct Rule C gate that is unreachable
    /// via the production keyring backend (which returns Ok(None) per ADR-026 §D3).
    ///
    /// AD-017: Neither spec_auth_type nor probe_shape are credential VALUES —
    /// they are auth_type identifiers from the canonical enumeration.
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-3c | BC-2.01.016 Rule C | ADR-026 §D3 Rule 3
    /// VP-153 prop 1/2 (Rule C): F-LP-IMPL-P8-IMP-001
    #[test]
    fn prop_rule_c_shape_mismatch_rejected_via_shaped_probe(
        (spec_auth_type, probe_shape) in arb_mismatched_auth_type_pair(),
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let spec_tmp = tempfile::TempDir::new().unwrap();
            // Use a unique sensor_id per proptest iteration to avoid cross-test file collisions.
            let sensor_id = format!("vp153-rule-c-{}", uuid_slug(spec_auth_type, probe_shape));

            let config_manager = make_config_manager_for_auth_type(
                spec_tmp.path(),
                &sensor_id,
                spec_auth_type,
            );
            let (config, _state_tmp) = make_config(spec_tmp.path());

            // Inject ShapedProbe that reports a DIFFERENT auth_type than the spec declares.
            // Pre-condition: spec has auth_type=spec_auth_type; probe returns probe_shape != spec_auth_type.
            // Post-condition: step5 must return Err(AuthTypeCredentialMismatch).
            let probe = ShapedProbe {
                reported_shape: probe_shape.to_string(),
            };
            // ShapedProbe ignores org_registry; pass an empty registry for the probe's signature.
            let empty_registry = std::sync::Arc::new(prism_core::OrgRegistry::new());

            let result = prism_bin::boot::step5_init_credential_store_with_probe(
                &config,
                &config_manager,
                &empty_registry,
                &probe,
            )
            .await;

            let err = match result {
                Ok(_) => {
                    return Err(proptest::test_runner::TestCaseError::fail(format!(
                        "Rule C (E-SPEC-014): ShapedProbe reports probe_shape={:?} but spec \
                         declares auth_type={:?} — step5 must return Err(AuthTypeCredentialMismatch); \
                         got Ok",
                        probe_shape, spec_auth_type
                    )));
                }
                Err(e) => e,
            };
            let err_str = format!("{err}");

            // Must be the specific AuthTypeCredentialMismatch variant.
            prop_assert!(
                matches!(
                    err,
                    BootError::AuthTypeCredentialMismatch { .. }
                ),
                "Rule C: error variant must be AuthTypeCredentialMismatch; \
                 spec_auth_type={:?}, probe_shape={:?}; got: {:?}",
                spec_auth_type,
                probe_shape,
                err_str
            );

            // Error must cite E-SPEC-014 (BC-2.01.016 §Error Cases).
            prop_assert!(
                err_str.contains("E-SPEC-014"),
                "Rule C: error must cite E-SPEC-014; spec={:?}, probe={:?}; got: {}",
                spec_auth_type,
                probe_shape,
                err_str
            );

            // AD-017: error must NOT contain credential values — only auth_type shape names.
            // spec_auth_type and probe_shape ARE auth_type identifiers (allowed in errors).
            // Credential VALUES (passwords, tokens, API keys) are never in the fixture.
            // Verify error contains expected_shape and actual_shape (per BootError fmt).
            prop_assert!(
                err_str.contains(spec_auth_type),
                "Rule C: error must cite expected_shape={:?}; got: {}",
                spec_auth_type,
                err_str
            );
            prop_assert!(
                err_str.contains(probe_shape),
                "Rule C: error must cite actual_shape={:?}; got: {}",
                probe_shape,
                err_str
            );

            Ok(())
        })
        .map_err(|e: proptest::test_runner::TestCaseError| e)?;
    }
}

proptest! {
    /// VP-153 prop 2 of 2 (Rule C): ShapedProbe reports matching shape →
    /// step5_init_credential_store_with_probe returns Ok(store).
    ///
    /// Regression guard: when the probe returns the SAME auth_type as the spec declares,
    /// the Rule C gate must NOT fire — step5 returns Ok(Arc<dyn CredentialStore>).
    ///
    /// This covers all 5 valid auth_type self-pairs:
    ///   (oauth2_client_credentials, oauth2_client_credentials),
    ///   (bearer_static, bearer_static), etc.
    ///
    /// AD-017: Both auth_type strings are canonical auth_type identifiers, not credential values.
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-3c | BC-2.01.016 postcondition | ADR-026 §D3 Rule 3
    /// VP-153 prop 2/2 (Rule C): F-LP-IMPL-P8-IMP-001
    #[test]
    fn prop_rule_c_shape_match_accepted_via_shaped_probe(
        auth_type in arb_matching_auth_type(),
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let spec_tmp = tempfile::TempDir::new().unwrap();
            let sensor_id = format!("vp153-rule-c-match-{}", auth_type.replace('_', "-"));

            let config_manager = make_config_manager_for_auth_type(
                spec_tmp.path(),
                &sensor_id,
                auth_type,
            );
            let (config, _state_tmp) = make_config(spec_tmp.path());

            // Inject ShapedProbe that reports the SAME auth_type as the spec declares.
            // Rule C gate: actual_shape == expected_shape → no error.
            let probe = ShapedProbe {
                reported_shape: auth_type.to_string(),
            };
            // ShapedProbe ignores org_registry; pass an empty registry for the probe's signature.
            let empty_registry = std::sync::Arc::new(prism_core::OrgRegistry::new());

            let result = prism_bin::boot::step5_init_credential_store_with_probe(
                &config,
                &config_manager,
                &empty_registry,
                &probe,
            )
            .await;

            prop_assert!(
                result.is_ok(),
                "Rule C regression guard: probe reports matching shape {:?} for spec \
                 auth_type {:?} — step5 must return Ok; got: {:?}",
                auth_type,
                auth_type,
                result.err()
            );

            Ok(())
        })
        .map_err(|e: proptest::test_runner::TestCaseError| e)?;
    }
}

// ---------------------------------------------------------------------------
// Helper: deterministic short slug for sensor_id uniqueness within a proptest run
// ---------------------------------------------------------------------------

/// Generate a short deterministic slug from two strings.
/// Used to give each proptest iteration a unique sensor_id (avoids TOML file
/// collision if the same spec_tmp dir is reused across proptest cases by chance).
fn uuid_slug(a: &str, b: &str) -> String {
    // Simple approach: concat first 4 chars of each, joined by dash.
    let a_slug: String = a.chars().take(4).collect();
    let b_slug: String = b.chars().take(4).collect();
    format!("{}-{}", a_slug, b_slug)
}
