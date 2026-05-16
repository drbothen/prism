---
document_type: verification-property
level: L4
version: "0.4"
status: draft
producer: architect
timestamp: 2026-05-15T00:00:00Z
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
modified: "2026-05-15"
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
   structured error **E-SPEC-012** ("Auth type cross-composition rejected for sensor '{sensor}':
   auth_type must be a single value from the enumerated set; got '{values}'").
   NOTE: E-SPEC-010 is already allocated to "Variable interpolation failed" in error-taxonomy.md.
   E-SPEC-012 is the correct new code. PO must add E-SPEC-012/013/014 to error-taxonomy.md as
   an in-scope amendment to S-PLUGIN-PREREQ-E before the test-writer authors AC-3 tests.

2. **Rule B — One credential per method:** A sensor spec where `credential_refs` references
   more than one credential per auth method MUST be rejected at spec-load time with
   structured error **E-SPEC-013** ("Multiple credential_refs for auth method '{method}' in
   sensor '{sensor}': exactly one credential_ref is required").

3. **Rule C — Auth type / credential type coherence:** A sensor spec where the resolved
   credential type does not structurally match the spec's `auth_type` variant MUST be rejected
   at credential-resolution time, before any HTTP request is issued, with structured error
   **E-SPEC-014** ("Credential type mismatch for sensor '{sensor}': auth_type '{auth_type}'
   requires credential type '{expected}', got '{actual}'").

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
// VP-153: SensorAuth runtime cross-composition prevention
// Method: proptest
// Target: prism_spec_engine::spec_parser (or pipeline validation pass)
//
// use proptest::prelude::*;
// use prism_spec_engine::spec_parser::{AuthType, SensorSpec};
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
| 0.4 | fix-burst-4 changelog repair | 2026-05-15 | state-manager | Renumbering correction per F-LP4-MED-001/002 process-gap-driven repair. Prior changelog had non-monotonic sequence (0.1 → 0.3 → 0.3 → 0.2): cross-review row was mis-labeled 0.3 and out of order; state-manager catch row duplicated the reconciliation 0.3 label. Corrected to monotonic 0.1 → 0.2 → 0.3 → 0.4. Frontmatter version updated from 0.3 to 0.4. |
| 0.4 | prereq-e-fix-burst-4 | 2026-05-15 | architect | F-LP4-LOW-002: `modified:` field synced to ISO date "2026-05-15" (most recent change date from fix-burst-3 state-manager catch). Prior value was empty array `[]`; POL-27 VP-template schema gap codified for cycle-close session-reviewer extension. No version bump (frontmatter field correction only). |
