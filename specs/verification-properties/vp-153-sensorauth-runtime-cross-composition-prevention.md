---
document_type: verification-property
level: L4
version: "0.13"
status: draft
producer: architect
timestamp: 2026-05-16T16:00:00Z
phase: prereq-e
inputs:
  - .factory/specs/architecture/decisions/ADR-026-sensorauth-unsealing.md
  - .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
input-hash: "[pending-recompute]"
traces_to: .factory/specs/architecture/decisions/ADR-026-sensorauth-unsealing.md
source_bc: BC-2.01.016
source_adr: ADR-026
source_invariant: DI-012
module: prism-spec-engine
priority: P0
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: draft
introduced: "2026-05-15"
modified: "2026-05-17"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-153: SensorAuth Runtime Cross-Composition Prevention (DI-012 Runtime Replacement)

## Property Statement

After the removal of `private::Sealed` from `SensorAuth` (ADR-026), cross-sensor auth-composition
prevention transitions from compile-time to runtime enforcement. The spec-validation layer in
`prism-spec-engine` MUST enforce the three runtime rules defined in ADR-026 D3 and ADR-023 Rule 2:

1. **Rule A — Single auth_type:** A sensor spec with more than one `auth_type` value, or with
   an `auth_type` value not in the enumerated set `{oauth2_client_credentials, bearer_static,
   cookie_roundtrip, api_key, custom_via_plugin}`, MUST be rejected at spec-load time with
   structured error **E-SPEC-012** ("auth_type for sensor '{sensor_id}' must be a single value;
   got: {value}. Valid values: oauth2_client_credentials, bearer_static, cookie_roundtrip,
   api_key, custom_via_plugin").

2. **Rule B — One credential per method:** A sensor spec where `credential_refs` references
   more than one credential per auth method MUST be rejected at spec-load time with
   structured error **E-SPEC-013** ("auth method for sensor '{sensor_id}' declares {count}
   credential_refs; exactly one is required").

3. **Rule C — Auth type / credential type coherence:** A sensor spec where the resolved
   credential type does not structurally match the spec's `auth_type` variant MUST be rejected
   at credential-resolution time, before any HTTP request is issued, with structured error
   **E-SPEC-014** ("credential type '{credential_type}' is incompatible with auth_type
   '{auth_type}' for sensor '{sensor_id}'").

A proptest strategy generates arbitrary `(auth_type, credential_type)` pairs across the valid
and invalid space. For all invalid combinations, the validator must return `Err`. For all valid
combinations, the validator must return `Ok`. No `Err` may contain a credential value in its
message text (AD-017 AI-opaque credential model).

## Source Contract

- **ADR:** ADR-026 D3 — Runtime cross-sensor auth-composition prevention
- **ADR:** ADR-023 Rule 2 — SensorAuth Trait Un-Sealing, runtime enforcement rules
- **BC:** BC-2.01.016 — SensorAuth Open Trait (primary; new auth surface contract for PREREQ-E)
- **BC:** BC-2.01.013 — datasource-trait-adapter-pattern (parent pattern; amended by ADR-023; not further amended in PREREQ-E)
- **Invariant:** DI-012 — sealed-auth-trait (downgraded to runtime enforcement per ADR-023)
- **Error codes:** E-SPEC-012 (auth_type cross-composition), E-SPEC-013 (multiple credential_refs), E-SPEC-014 (auth_type/credential mismatch) — PO must add these to error-taxonomy.md in PREREQ-E scope
- **Module:** prism-spec-engine (spec_parser.rs and/or pipeline.rs validation pass)
- **Category:** Security Invariant / Auth Policy Enforcement

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest 1.x | Yes — finite (auth_type × credential_type) space | All valid and invalid (auth_type, credential_type) pairs per ADR-026 D3 |

**Feasibility:** The valid `auth_type` enumerated set has 5 members. The credential structural
types are similarly finite (OAuth2 token, bearer token, cookie session, API key, WASM plugin
token). The Cartesian product is small and fully enumerable. proptest can cover all pairs
deterministically with a small strategy and provide regression coverage for the invariant.

## Proof Harness Skeleton

```rust
// crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs
//
// VP-153: SensorAuth runtime cross-composition prevention — ALL 3 RULES
// Rule A (E-SPEC-012): multi-valued or out-of-set auth_type → spec-load rejection
// Rule B (E-SPEC-013): multiple credential_refs per auth method → spec-load rejection
// Rule C (E-SPEC-014): auth_type/credential structural mismatch → credential-resolution rejection
//
// NOTE (FB34): File name predates Rule A/B scaffolding (originally Rule-C-only); it is kept
// as-is (`vp153_sensorauth_cross_composition.rs`) for continuity with test infrastructure
// references. The test-writer may rename to `vp153_sensorauth_runtime_cross_composition_prevention.rs`
// to match the VP slug if no downstream tooling depends on the current name.
//
// Method: proptest
// Target: prism_spec_engine::spec_parser (or pipeline validation pass)
//
// use proptest::prelude::*;
// use prism_spec_engine::spec_parser::{AuthType, SensorSpec};
// use prism_core::error::SpecEngineError;
//
// fn arb_auth_type() -> impl Strategy<Value = AuthType> {
//     prop_oneof![
//         Just(AuthType::Oauth2ClientCredentials),
//         Just(AuthType::BearerStatic),
//         Just(AuthType::CookieRoundtrip),
//         Just(AuthType::ApiKey),
//         Just(AuthType::CustomViaPlugin),
//     ]
// }
//
// fn arb_credential_type() -> impl Strategy<Value = MockCredentialType> {
//     // MockCredentialType enumerates the structural credential shapes
//     prop_oneof![
//         Just(MockCredentialType::Oauth2ClientCredentials { client_id: "x", client_secret: "y" }),
//         Just(MockCredentialType::BearerToken("token")),
//         Just(MockCredentialType::CookieSession("session")),
//         Just(MockCredentialType::ApiKey("key")),
//         Just(MockCredentialType::PluginToken("plugin-id")),
//     ]
// }
//
// // ── Rule A (E-SPEC-012) ──────────────────────────────────────────────────────
//
// proptest! {
//     #[test]
//     fn multi_valued_or_out_of_set_auth_type_rejected_with_e_spec_012(
//         // Strategy produces one of two invalid auth_type shapes:
//         //   (a) a TOML array value, e.g. `auth_type = ["oauth2_client_credentials", "bearer_static"]`
//         //   (b) an arbitrary string outside the canonical set, e.g. `auth_type = "unknown_strategy"`
//         // The test-writer selects whichever TOML construction path is simpler given the
//         // actual parse API (string-template TOML or typed builder).
//         raw_auth_value in prop_oneof![
//             // (a) multi-valued array: 2+ valid members to trigger array-detection branch
//             Just(r#"["oauth2_client_credentials", "bearer_static"]"#.to_string()),
//             // (b) out-of-set string: proptest generates arbitrary non-empty strings that are
//             //     not members of the canonical set; use prop_filter to exclude valid values
//             "[a-z_]{1,32}".prop_filter("must not be a valid auth_type", |s| {
//                 !matches!(s.as_str(),
//                     "oauth2_client_credentials" | "bearer_static" |
//                     "cookie_roundtrip" | "api_key" | "custom_via_plugin")
//             }),
//         ]
//     ) {
//         let toml = format!(r#"
//             [sensor]
//             sensor_id = "test-sensor"
//             auth_type = {}
//             [[sensor.credential_refs]]
//             name = "my-cred"
//         "#, raw_auth_value);
//         let result = SpecParser::parse_str(&toml);
//         prop_assert!(result.is_err(), "invalid auth_type {:?} was accepted", raw_auth_value);
//         let err = result.unwrap_err();
//         // Assertion: error must be AuthTypeInvalid variant with E-SPEC-012 message_template
//         // byte-verbatim per error-taxonomy.md v1.34:
//         //   "auth_type for sensor '{sensor_id}' must be a single value; got: {value}.
//         //    Valid values: oauth2_client_credentials, bearer_static, cookie_roundtrip,
//         //    api_key, custom_via_plugin"
//         prop_assert!(matches!(err, SpecEngineError::AuthTypeInvalid { .. }),
//             "wrong error variant for Rule A: {:?}", err);
//         let err_msg = err.to_string();
//         prop_assert!(err_msg.contains("must be a single value"),
//             "E-SPEC-012 message_template substring not found in: {}", err_msg);
//     }
// }
//
// // ── Rule B (E-SPEC-013) ──────────────────────────────────────────────────────
//
// proptest! {
//     #[test]
//     fn multiple_credential_refs_per_method_rejected_with_e_spec_013(
//         // Strategy: produce a sensor spec TOML with >1 credential_refs entries under a
//         // single auth method. The count is drawn from [2..=5] to exercise the error path
//         // across different cardinalities.
//         extra_ref_count in 1usize..=4,  // total refs = extra_ref_count + 1 (always ≥ 2)
//         auth_type in arb_auth_type(),
//     ) {
//         // Build a TOML string with (extra_ref_count + 1) credential_refs entries
//         let mut cred_refs = String::new();
//         for i in 0..=(extra_ref_count) {
//             cred_refs.push_str(&format!(
//                 r#"[[sensor.credential_refs]]
//                 name = "cred-{}"
//                 "#, i
//             ));
//         }
//         let toml = format!(r#"
//             [sensor]
//             sensor_id = "test-sensor"
//             auth_type = "{}"
//             {}
//         "#, auth_type.as_str(), cred_refs);
//         let result = SpecParser::parse_str(&toml);
//         prop_assert!(result.is_err(),
//             "spec with {} credential_refs was accepted (Rule B violation)", extra_ref_count + 1);
//         let err = result.unwrap_err();
//         // Assertion: error must be MultipleCredentialRefs variant with E-SPEC-013 message_template
//         // byte-verbatim per error-taxonomy.md v1.34:
//         //   "auth method for sensor '{sensor_id}' declares {count} credential_refs;
//         //    exactly one is required"
//         prop_assert!(matches!(err, SpecEngineError::MultipleCredentialRefs { .. }),
//             "wrong error variant for Rule B: {:?}", err);
//         let err_msg = err.to_string();
//         prop_assert!(err_msg.contains("exactly one is required"),
//             "E-SPEC-013 message_template substring not found in: {}", err_msg);
//     }
// }
//
// // ── Rule C (E-SPEC-014) ──────────────────────────────────────────────────────
//
// proptest! {
//     #[test]
//     fn valid_auth_type_credential_pairs_accepted(
//         auth_type in arb_auth_type(),
//     ) {
//         let matching_credential = matching_credential_for(&auth_type);
//         let spec = build_spec_with(auth_type.clone(), matching_credential);
//         let result = validate_auth_coherence(&spec);
//         prop_assert!(result.is_ok(),
//             "valid (auth_type={:?}, credential=matching) was rejected: {:?}",
//             auth_type, result);
//     }
//
//     #[test]
//     fn mismatched_auth_type_credential_rejected(
//         auth_type in arb_auth_type(),
//         credential_type in arb_credential_type(),
//     ) {
//         prop_assume!(!is_coherent_pair(&auth_type, &credential_type));
//         let spec = build_spec_with(auth_type.clone(), credential_type.clone());
//         let result = validate_auth_coherence(&spec);
//         prop_assert!(result.is_err(),
//             "invalid (auth_type={:?}, credential={:?}) was accepted",
//             auth_type, credential_type);
//         // AD-017: error message must not contain credential values
//         let err_msg = result.unwrap_err().to_string();
//         prop_assert!(!err_msg.contains(credential_value_str(&credential_type)),
//             "error message leaks credential value: {}", err_msg);
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Small and finite | 5 auth_type variants × 5 credential structural shapes = 25 pairs; all enumerable |
| Proof complexity | Low–medium | Boolean accept/reject plus error-message redaction check |
| Tool support | Full | proptest 1.x; no special infrastructure needed |
| Harness dependencies | Moderate | Requires a `validate_auth_coherence` extraction point in spec_parser.rs or pipeline.rs validation pass |
| Estimated proof time | <5 seconds | Small finite space; no I/O or async |

**Harness authoring note:** The test-writer must confirm whether `validate_auth_coherence` is
callable as a pure function or requires constructing a full `ConfigSnapshot`. If it requires
the full snapshot, the harness uses a fixture TOML approach (proptest generates the
auth_type/credential_type fields into a TOML string, then calls `parse_spec_directory` in dry-run
mode against a temp dir). Either approach is feasible.

## Open Issues

| ID | Issue | Owner | Resolution |
|----|-------|-------|-----------|
| ~~VP153-OPEN-001~~ | ~~E-SPEC-012/013/014 not yet in error-taxonomy.md~~ | product-owner | **CLOSED** — PO authored E-SPEC-012/013/014 in error-taxonomy.md v1.25 (S-PLUGIN-PREREQ-E-reconciliation burst). BC-2.01.016 updated to cite E-SPEC-012/013/014. VP-153 harness skeleton updated accordingly. |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-15 | architect (PREREQ-E ADR burst) |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | plugin-prereq-e-adr-burst | 2026-05-15 | architect | Initial stub. Traces to ADR-026 D3 / ADR-023 Rule 2 / DI-012 runtime enforcement replacement. Harness skeleton provided; full authoring in S-PLUGIN-PREREQ-E test-writer dispatch. |
| 0.2 | plugin-prereq-e-cross-review | 2026-05-15 | architect | Q3 resolution: remove "error code to be assigned" placeholder. Assign E-SPEC-012 (Rule A), E-SPEC-013 (Rule B), E-SPEC-014 (Rule C). Document E-SPEC-010 collision (already taken). Update source_bc to BC-2.01.016 (primary auth-surface BC). Route E-SPEC-012/013/014 authoring to PO via VP153-OPEN-001. |
| 0.3 | S-PLUGIN-PREREQ-E-reconciliation | 2026-05-15 | product-owner | VP153-OPEN-001 closed — E-SPEC-012/013/014 authored in error-taxonomy.md v1.25. BC-2.01.016 error references updated. Harness skeleton uses correct error code labels. Open Issues table updated with closure record. |
| 0.4 | fix-burst-1 state-manager catch | 2026-05-15 | state-manager | (state-manager catch in fix-burst-1) F-LP1-HIGH-004 POL-20: introduced field canonicalized to ISO date 2026-05-15. Prior value `plugin-prereq-e` was informal slug; POL-20 requires `YYYY-MM-DD` for artifacts created outside greenfield cycles. |
| 0.5 | fix-burst-5 renumber-repair-redo | 2026-05-15 | state-manager | F-LP5-HIGH-003 renumber-repair-redo. FB4 assigned both the changelog-repair row and the modified-field-sync row to v0.4, producing two rows at the same version and violating monotonic strict order. Repair row renumbered 0.4→0.5. Absorbs FB4 modified-field-sync content: `modified:` field confirmed synced to ISO date "2026-05-15" per F-LP4-LOW-002 / POL-27. Content summary retained: prior changelog had non-monotonic sequence (0.1 → 0.3 → 0.3 → 0.2); corrected to 0.1 → 0.2 → 0.3 → 0.4 (state-manager POL-20 catch) → 0.5 (this row). Each distinct content change now holds a unique version. Frontmatter version updated to 0.5. Monotonic sequence verified: 0.1 → 0.2 → 0.3 → 0.4 → 0.5. |
| 0.6 | FB29 | 2026-05-16 | architect | F-LP37-MED-003 — Rule A/B/C message-format quotations updated to match canonical error-taxonomy.md v1.30 byte-for-byte (Option A: verbatim sync). Prior divergent strings were: Rule A `"Auth type cross-composition rejected for sensor '{sensor}': auth_type must be a single value from the enumerated set; got '{values}'"`, Rule B `"Multiple credential_refs for auth method '{method}' in sensor '{sensor}': exactly one credential_ref is required"`, Rule C `"Credential type mismatch for sensor '{sensor}': auth_type '{auth_type}' requires credential type '{expected}', got '{actual}'"`. Closes POL-24 (error_message_template_verbatim) violation surviving since pre-pass-37 era. TD-VSDD-060 sibling-site sweep: old divergent strings found only in VP-153 itself — no other spec artifact affected. POL-25 multi-cite sweep: E-SPEC-012/013/014 cited in 12+ artifacts; none misquote the canonical template. Future amendment discipline: POL-23/POL-25 sibling-sweep must include VP-153 Rule A/B/C whenever error-taxonomy.md E-SPEC-012/013/014 message_template fields are amended. |
| 0.7 | FB34 | 2026-05-16 | architect | F-LP44-MED-002 — §Proof Harness Skeleton expanded: Rule A (E-SPEC-012 multi-valued/out-of-set auth_type) proptest `multi_valued_or_out_of_set_auth_type_rejected_with_e_spec_012` + Rule B (E-SPEC-013 multiple credential_refs) proptest `multiple_credential_refs_per_method_rejected_with_e_spec_013` scaffolded. Previously only Rule C was scaffolded (2 proptests); skeleton now covers all 3 Rules (4 proptests total). Eliminates under-coverage risk for security-critical spec-load rejection layer. Harness file name kept as `vp153_sensorauth_cross_composition.rs` with scope note; test-writer may rename to match VP slug. |
| 0.8 | FB39 | 2026-05-16 | architect | FB39: F-LP49-HIGH-001 sites 2+3 closure — §Proof Harness Skeleton inline comments lines 167 (Rule A E-SPEC-012 byte-verbatim provenance) + 210 (Rule B E-SPEC-013 byte-verbatim provenance) advanced from v1.30 to v1.31 per FB38 D-657 cascade. |
| 0.9 | FB40 | 2026-05-16 | state-manager | FB40 D-659: F-LP50-MED-002 §Changelog row ordering corrected to monotonic ascending (oldest first) per POL-26. Prior order was 0.7 → 0.8 → 0.6 → 0.1 → 0.2 → 0.3 → 0.4 → 0.5 (non-monotonic; rows 0.6 through 0.5 appended after 0.7+0.8 in the wrong sequence). Corrected order: 0.1 → 0.2 → 0.3 → 0.4 → 0.5 → 0.6 → 0.7 → 0.8 → 0.9. Pre-existing defect surviving 49 prior passes — fresh-context catch via vector rotation (lateral vector: VP-153 §Changelog row ordering audit). |
| 0.10 | FB52 | 2026-05-17 | architect | F-LP64-HIGH-001 closure: error-taxonomy.md v1.31→v1.32 sibling-sweep at 2 proof harness skeleton comments (lines 167, 210). POL-29 v1.13 grep evidence: 2 pre → 0 post. |
| 0.11 | FB56 | 2026-05-17 | product-owner | F-LP68-HIGH-001 closure (PO scope): error-taxonomy.md v1.32→v1.33 propagation at VP-153 lines 167, 210 §Proof Harness comments (2 live-narrative sites; backtick-quoted variant form). POL-29 v1.16 step 3a (a) recurrence #20 within-burst closure. |
| 0.12 | FB56+FB56b SM step 8a catch | 2026-05-17 | state-manager | POL-29 v1.17 step 8a FINAL EMPIRICAL VERIFICATION CATCH: error-taxonomy v1.33→v1.34 propagation incomplete — VP-153 proof-harness comment lines 167 + 210 were not updated by FB56b. State-manager step 8a catch: both code-comment sites updated to `error-taxonomy.md v1.34`. post-grep: 0 live-narrative. |
| 0.13 | FB57 | 2026-05-17 | state-manager | POL-26-COROLLARY bookkeeping repair: rows v0.11 + v0.12 swapped (FB56 PO row + FB56+FB56b SM catch row inserted in wrong order during 17-file LARGEST-burst). 8th POL-26 recurrence closed (F-LP69-MED-001). No content edits; row content preserved verbatim. |
