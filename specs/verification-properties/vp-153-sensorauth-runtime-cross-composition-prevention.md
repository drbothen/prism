---
document_type: verification-property
level: L4
version: "0.23"
status: active
producer: architect
timestamp: 2026-05-16T16:00:00Z
phase: prereq-e
inputs:
  - .factory/specs/architecture/decisions/ADR-026-sensorauth-unsealing.md
  - .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
input-hash: "18485b2"
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
proof_completed_date: "2026-05-18"
proof_file_hash: null
lifecycle_status: active
introduced: "2026-05-15"
modified: "2026-07-22"
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
   cookie_roundtrip, api_key, custom_via_plugin, token_exchange}`, MUST be rejected at spec-load time with
   structured error **E-SPEC-012** ("auth_type for sensor '{sensor_id}' must be a single value;
   got: {value}. Valid values: oauth2_client_credentials, bearer_static, cookie_roundtrip,
   api_key, custom_via_plugin, token_exchange").

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

**Feasibility:** The valid `auth_type` enumerated set has 6 members. The credential structural
types are similarly finite (OAuth2 token, bearer token, cookie session, API key, WASM plugin
token). The Cartesian product is small and fully enumerable. proptest can cover all pairs
deterministically with a small strategy and provide regression coverage for the invariant.

## Proof Harness Skeleton

```rust
// AS-BUILT HARNESS (proof-completed-date 2026-05-18; all 8 proptests PASS)
//
// ── FILE 1 (Rules A + B) ─────────────────────────────────────────────────────
// crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs
//
// Constants:
//   const VALID_AUTH_TYPES: &[&str] = &[
//       "oauth2_client_credentials", "bearer_static", "cookie_roundtrip",
//       "api_key", "custom_via_plugin",
//       "token_exchange",  // [PLANNED — ADR-054 D1 engine story]
//   ]; // 6 members — "token_exchange" added per ADR-054 D1
//
// Generators:
//   fn arb_valid_auth_type() -> impl Strategy<Value = &'static str>
//     — prop_oneof![ Just("oauth2_client_credentials"), Just("bearer_static"),
//         Just("cookie_roundtrip"), Just("api_key"), Just("custom_via_plugin"),
//         Just("token_exchange") ]  // [PLANNED — ADR-054 D1 engine story]
//
//   fn arb_invalid_auth_type() -> impl Strategy<Value = String>
//     — generates invalid auth_type strings across 7 classes:
//       (a) random lowercase+underscore strings; (b) empty string; (c) whitespace-only;
//       (d) capitalised near-misses ("BEARER_STATIC", "ApiKey", etc.);
//       (e) dot/slash-prefixed (".oauth2_client_credentials"); (f) arbitrary Unicode;
//       (g) composite array-repr strings ("[oauth2_client_credentials,bearer_static]")
//     — .prop_filter("must not accidentally be a valid auth_type",
//         |s| !VALID_AUTH_TYPES.contains(&s.as_str()))
//     — (filter auto-expands when VALID_AUTH_TYPES gains "token_exchange" — no separate edit)
//
//   fn arb_multi_credential_count() -> impl Strategy<Value = usize>
//     — range 2usize..=8usize (violates Rule B for non-oauth2; see also prop 6 range 2..=32)
//
// Target function:
//   SpecLoader::validate_cross_composition(
//       sensor_id: &str, auth_type: &str, count: usize,
//       expected_shape: &str, actual_shape: &str,
//   ) -> Result<(), SpecEngineError>
//   Pure function; no I/O. Rules A+B fire on auth_type + count inputs.
//   Rule C gate (actual_shape != expected_shape) is exercised in FILE 2 via ShapedProbe.
//   oauth2_client_credentials allowed_count = 2; all others = 1.
//
// Proptests (6):
//   1. prop_rule_a_invalid_auth_type_rejected_with_e_spec_012
//      arb_invalid_auth_type() → validate_cross_composition(count=1, same_shapes)
//      → result.is_err() + err_str.contains("E-SPEC-012")
//   2. prop_rule_a_valid_auth_type_accepted
//      arb_valid_auth_type() → allowed_count = if valid_type == "oauth2_client_credentials" { 2 } else { 1 }
//      → validate_cross_composition(count=allowed_count, same_shapes) → result.is_ok()
//   3. prop_rule_b_multi_credential_refs_rejected_with_e_spec_013
//      (arb_valid_auth_type(), arb_multi_credential_count())
//      + prop_assume!(count != allowed_count)  // skips valid (oauth2_client_credentials, 2) pair
//      → validate_cross_composition → err_str.contains("E-SPEC-013")
//   4. prop_rule_b_single_credential_ref_accepted
//      arb_valid_auth_type() → allowed_count → validate_cross_composition → result.is_ok()
//   5. prop_rule_a_boundary_whitespace_and_empty_rejected
//      prop_oneof!["", " ", "  ", "\t", "\n"] → validate_cross_composition(count=1, same_shapes)
//      → result.is_err() + err_str.contains("E-SPEC-012")
//   6. prop_rule_b_credential_count_boundary
//      (arb_valid_auth_type(), 2usize..=32)
//      + prop_assume!(count != allowed_count)
//      → validate_cross_composition → err_str.contains("E-SPEC-013")
//
// ── FILE 2 (Rule C) ──────────────────────────────────────────────────────────
// crates/prism-bin/tests/vp153_rule_c_shaped_probe.rs
// Lives in prism-bin to avoid workspace cycle: prism-bin depends on prism-spec-engine;
// adding prism-bin as prism-spec-engine dev-dep would create a cycle.
// (ADR-026 §D3 Rule C Backend Scope / D-706)
//
// Constants:
//   const VALID_AUTH_TYPES: &[&str] = &[...same 6 members as FILE 1, incl. "token_exchange" per ADR-054 D1 (Wave-A; [PLANNED — engine story])...];
//
// Test fixture:
//   struct ShapedProbe { reported_shape: String }
//     — implements CredentialRefProbe; returns Ok(Some(self.reported_shape.clone()))
//     — AD-017: reported_shape is always a canonical auth_type identifier, never a credential value
//
// Generators:
//   fn arb_mismatched_auth_type_pair() -> impl Strategy<Value = (&'static str, &'static str)>
//     — (0usize..6, 0usize..5).prop_map(|(spec_idx, offset)| ...)  // updated per ADR-054 D1 ([PLANNED — engine story])
//     — covers all 30 ordered mismatched pairs (6×5) from VALID_AUTH_TYPES
//
//   fn arb_matching_auth_type() -> impl Strategy<Value = &'static str>
//     — prop_oneof![ Just("oauth2_client_credentials"), ..., Just("custom_via_plugin"),
//         Just("token_exchange") ]  // [PLANNED — ADR-054 D1 engine story]
//
// Target function:
//   step5_init_credential_store_with_probe(
//       config: &PrismConfig, config_manager: &Arc<ArcSwap<ConfigManager>>,
//       org_registry: &Arc<OrgRegistry>, probe: &dyn CredentialRefProbe,
//   ) -> Result<Arc<dyn CredentialStore>, BootError>
//   (defined in crates/prism-bin/src/boot.rs)
//
// Proptests (2):
//   1. prop_rule_c_shape_mismatch_rejected_via_shaped_probe
//      arb_mismatched_auth_type_pair() → ShapedProbe { reported_shape: probe_shape }
//      where probe_shape != spec_auth_type
//      → step5_init_credential_store_with_probe
//      → Err(BootError::AuthTypeCredentialMismatch { .. })
//      + err_str.contains("E-SPEC-014")
//      + err_str.contains(spec_auth_type) + err_str.contains(probe_shape)
//   2. prop_rule_c_shape_match_accepted_via_shaped_probe
//      arb_matching_auth_type() → ShapedProbe { reported_shape: auth_type }
//      where probe_shape == spec_auth_type
//      → step5_init_credential_store_with_probe → result.is_ok()
```

## Re-verification Gate

> **Re-verification gate (F-WASE-P4-OBS-002):** The ADR-054 engine story MUST re-run VP-153
> with the `token_exchange` proptest arms **activated** (dropping `[PLANNED — ADR-054 D1 engine
> story]` markers from `Just("token_exchange")` in `arb_valid_auth_type()` and
> `arb_matching_auth_type()`, and from the updated `arb_mismatched_auth_type_pair()` range bounds
> in both FILE 1 and FILE 2) as an **explicit story gate before the engine story PR can merge**.
> Until the engine story lands, the current green proof (proof-completed-date 2026-05-18) covers
> the **5-value as-built set**; the `token_exchange` arms are spec-only scaffolding that have not
> yet executed. See ADR-054 §D11 for the harness-amendment checklist.

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Small and finite | 6 auth_type variants × 5 credential structural shapes = 30 pairs; all enumerable |
| Proof complexity | Low–medium | Boolean accept/reject plus error-message redaction check |
| Tool support | Full | proptest 1.x; no special infrastructure needed |
| Harness dependencies | Moderate | Requires `SpecLoader::validate_cross_composition` — as-built in `crates/prism-spec-engine/src/spec_parser.rs` (callable as pure function per proof-completed-date 2026-05-18) |
| Estimated proof time | <5 seconds | Small finite space; no I/O or async |

**Harness authoring note (as-built, proof-completed-date 2026-05-18):** `SpecLoader::validate_cross_composition` is callable as a pure function — the test-writer authored 6 proptests in `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs` (Rules A+B) and 2 proptests in `crates/prism-bin/tests/vp153_rule_c_shaped_probe.rs` (Rule C via ShapedProbe injection path). Rule C lives in prism-bin due to dependency direction (prism-bin depends on prism-spec-engine; reverse dep would create workspace cycle). All 8 proptests PASS.

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
| 0.23 | Wave-A-spec-evolution-fix-burst-6 | 2026-07-22 | architect | F-WASE-P6-MED-001: §Feasibility Assessment "Harness dependencies" row — stale file path `spec_loader.rs` corrected to `spec_parser.rs` (file does not exist; `SpecLoader::validate_cross_composition` is as-built in `crates/prism-spec-engine/src/spec_parser.rs` at line 1382, inside `impl SpecLoader`). §Source Contract and §Proof Harness Skeleton already cited `spec_parser.rs` correctly; this row was the sole stale reference. Sweep confirmed: only one `spec_loader` hit across all of `.factory/specs/`. POL-32. |
| 0.22 | Wave-A-fix-burst-4 | 2026-07-22 | architect | F-WASE-P4-OBS-002: §Re-verification Gate section added — explicit engine-story gate note stating that the ADR-054 engine story MUST re-run all 8 VP-153 proptests with the `token_exchange` arms activated (dropping `[PLANNED]` markers from `Just("token_exchange")` in `arb_valid_auth_type()`, `arb_matching_auth_type()`, and the updated `arb_mismatched_auth_type_pair()` range bounds) before the engine story PR can merge; until then the green proof covers the 5-value as-built set. See ADR-054 §D11 for companion engine-story gate row (added in ADR-054 v0.37). |
| 0.21 | Wave-A-spec-evolution-burst-3 | 2026-07-22 | architect | ADR-054 D11 manifest execution (POL-24 same-commit atomicity with error-taxonomy.md v2.57 E-SPEC-012 amendment). §Property Statement Rule A: (1) enumerated set expanded 5→6 values — `token_exchange` appended; (2) E-SPEC-012 "Valid values:" clause updated verbatim from error-taxonomy.md v2.57 source of truth (POL-24). §Proof Method: "5 members" → "6 members" (D11 §Proof Method member-count row). §Feasibility Assessment table: "5 auth_type variants × 5 credential structural shapes = 25 pairs" → "6 auth_type variants × 5 credential structural shapes = 30 pairs" (D11 §Feasibility Assessment row). §Proof Harness Skeleton FILE 1: `VALID_AUTH_TYPES` gains `"token_exchange"` as 6th entry `[PLANNED — ADR-054 D1 engine story]`; `arb_valid_auth_type()` `prop_oneof!` gains `Just("token_exchange")` arm `[PLANNED]`; `arb_invalid_auth_type()` filter unchanged (auto-expands via VALID_AUTH_TYPES reference) (D11 §Proof Harness Skeleton row). FILE 2: `VALID_AUTH_TYPES` comment updated to note 6 members; `arb_mismatched_auth_type_pair()` range `(0..5, 0..4)→(0..6, 0..5)` covers 30 ordered pairs; `arb_matching_auth_type()` gains `Just("token_exchange")` arm `[PLANNED]` (D11 §Proof Harness Skeleton row). `modified:` synced to 2026-07-22. |
| 0.20 | FIX-BURST 24 | 2026-07-21 | architect | MED root fix (adversary pass 30): §Proof Harness Skeleton reconciled to as-built harness — replaced entire divergent pseudocode block (phantom typed-enum constructs `arb_auth_type()`, `AuthType`, `MockCredentialType`, `is_coherent_pair`, `matching_credential_for`, `arb_credential_type()`, `valid_auth_type_credential_pairs_accepted`, `mismatched_auth_type_credential_rejected`, `build_spec_with`, `credential_value_str`) with accurate as-built documentation of the real constructs: `VALID_AUTH_TYPES: &[&str]`, `arb_valid_auth_type()`, `arb_invalid_auth_type()`, `arb_multi_credential_count()`, `SpecLoader::validate_cross_composition()` (FILE 1, Rules A+B) and `ShapedProbe`, `arb_mismatched_auth_type_pair()`, `arb_matching_auth_type()`, `step5_init_credential_store_with_probe()` (FILE 2, Rule C). §Proof Method / §Feasibility prose unchanged (5-member count and 25-pairs statements are current-truth pending ADR-054 D1 amendment). POL-22 Phase C: all symbols cited verified present in as-built test files. |
| 0.19 | D-1915 | 2026-07-21 | state-manager | OBS-1 (adversary pass-15, pre-existing defect NOT introduced by the ADRs): §Changelog rows v0.16 (FB75) and v0.15 (FB71) were out of monotonic order — v0.16 appeared above v0.15 in the table (both dated 2026-05-17; in ascending convention v0.15 must precede v0.16; the pair was inverted). Changelog table converted to newest-first (descending) convention per validate-changelog-monotonicity hook enforcement; v0.15/v0.16 now correctly ordered in descending layout (v0.16 > v0.15 → v0.16 appears first). POL-26/POL-32. Bump v0.18→v0.19. |
| 0.18 | pass-10-spec-hygiene | 2026-05-18 | product-owner | F-LP-IMPL-P10-IMP-001 closure: §Proof Harness Skeleton stale symbol corrections. (1) Rule A assertion line: `SpecEngineError::AuthTypeInvalid { .. }` → `SpecEngineError::AuthTypeCrossComposition { .. }` (as-built enum variant name in `crates/prism-spec-engine/src/error.rs`). (2) Rule C skeleton: `validate_auth_coherence(&spec)` → `SpecLoader::validate_cross_composition(&spec)` (2 occurrences; as-built API callable as pure function). (3) Feasibility Assessment harness-dependencies row and Harness authoring note updated to reflect as-built proof state (proof-completed-date 2026-05-18; 8 proptests PASS across 2 crates). Spec brought into alignment with code per CLAUDE.md Source-of-Truth Precedence Rule 7. |
| 0.17 | FB-IMPL-6 | 2026-05-18 | test-writer | F-LP-IMPL-P8-IMP-001 closure: proptest harness authored and passing. Rules A+B (E-SPEC-012/013) in `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs` (6 proptests via `SpecLoader::validate_cross_composition`). Rule C (E-SPEC-014) via ShapedProbe in `crates/prism-bin/tests/vp153_rule_c_shaped_probe.rs` (2 proptests via `step5_init_credential_store_with_probe`). Rule C lives in prism-bin due to dependency direction — prism-bin depends on prism-spec-engine; adding prism-bin as prism-spec-engine dev-dep would create a workspace cycle. Per ADR-026 §D3 Rule C Backend Scope (D-706): Rule C exercises the ShapedProbe injection path (architecturally-sanctioned test fixture; keyring backend returns Ok(None) in production until PLUGIN-MIGRATION-001-A). `lifecycle_status: draft → active`. `proof_completed_date: "2026-05-18"`. All 8 proptests PASS. Pass-8 adversary caught this blind spot; passes 1-7 audited validator logic but never verified the P0 artifact existed. |
| 0.16 | FB75 | 2026-05-17 | product-owner | F-LP87-HIGH-001 closure (PO scope): error-taxonomy v1.37→v1.38 propagation at VP-153 proof-harness comments lines 167 + 210 (2 sites). Sibling: story v1.48 + HS-001 v1.11 + ADR-026 v1.24 swept in same burst. |
| 0.15 | FB71 | 2026-05-17 | product-owner | F-LP83-HIGH-001 closure (PO scope): error-taxonomy v1.35→v1.37 propagation at VP-153 proof-harness comment lines 167 + 210 (2 live-narrative sites; "error-taxonomy.md v1.35" → "error-taxonomy.md v1.37"). Recurrence #23+ class (a) — FB69 step 8d transitive closure gap. Sibling: story v1.45 + HS-001 v1.10 (PO) + ADR-026 v1.23 (architect). |
| 0.14 | FB62 | 2026-05-17 | state-manager | POL-29 v1.18 step 8b TRANSITIVE CLOSURE CATCH: error-taxonomy v1.34→v1.35 propagation at VP-153 proof-harness comment lines 167 + 210 (2 live-narrative sites; "error-taxonomy.md v1.34" → "error-taxonomy.md v1.35"). FB62 error-taxonomy bumped v1.34→v1.35 in PO dispatch; step 8b transitive closure detected these 2 sites as missed by PO sweep. State-manager applies pin advancement in-scope per Canonical Principle Rule 4. post-grep: 0 live-narrative. |
| 0.13 | FB57 | 2026-05-17 | state-manager | POL-26-COROLLARY bookkeeping repair: rows v0.11 + v0.12 swapped (FB56 PO row + FB56+FB56b SM catch row inserted in wrong order during 17-file LARGEST-burst). 8th POL-26 recurrence closed (F-LP69-MED-001). No content edits; row content preserved verbatim. |
| 0.12 | FB56+FB56b SM step 8a catch | 2026-05-17 | state-manager | POL-29 v1.17 step 8a FINAL EMPIRICAL VERIFICATION CATCH: error-taxonomy v1.33→v1.34 propagation incomplete — VP-153 proof-harness comment lines 167 + 210 were not updated by FB56b. State-manager step 8a catch: both code-comment sites updated to `error-taxonomy.md v1.34`. post-grep: 0 live-narrative. |
| 0.11 | FB56 | 2026-05-17 | product-owner | F-LP68-HIGH-001 closure (PO scope): error-taxonomy.md v1.32→v1.33 propagation at VP-153 lines 167, 210 §Proof Harness comments (2 live-narrative sites; backtick-quoted variant form). POL-29 v1.16 step 3a (a) recurrence #20 within-burst closure. |
| 0.10 | FB52 | 2026-05-17 | architect | F-LP64-HIGH-001 closure: error-taxonomy.md v1.31→v1.32 sibling-sweep at 2 proof harness skeleton comments (lines 167, 210). POL-29 v1.13 grep evidence: 2 pre → 0 post. |
| 0.9 | FB40 | 2026-05-16 | state-manager | FB40 D-659: F-LP50-MED-002 §Changelog row ordering corrected to monotonic ascending (oldest first) per POL-26. Prior order was 0.7 → 0.8 → 0.6 → 0.1 → 0.2 → 0.3 → 0.4 → 0.5 (non-monotonic; rows 0.6 through 0.5 appended after 0.7+0.8 in the wrong sequence). Corrected order: 0.1 → 0.2 → 0.3 → 0.4 → 0.5 → 0.6 → 0.7 → 0.8 → 0.9. Pre-existing defect surviving 49 prior passes — fresh-context catch via vector rotation (lateral vector: VP-153 §Changelog row ordering audit). |
| 0.8 | FB39 | 2026-05-16 | architect | FB39: F-LP49-HIGH-001 sites 2+3 closure — §Proof Harness Skeleton inline comments lines 167 (Rule A E-SPEC-012 byte-verbatim provenance) + 210 (Rule B E-SPEC-013 byte-verbatim provenance) advanced from v1.30 to v1.31 per FB38 D-657 cascade. |
| 0.7 | FB34 | 2026-05-16 | architect | F-LP44-MED-002 — §Proof Harness Skeleton expanded: Rule A (E-SPEC-012 multi-valued/out-of-set auth_type) proptest `multi_valued_or_out_of_set_auth_type_rejected_with_e_spec_012` + Rule B (E-SPEC-013 multiple credential_refs) proptest `multiple_credential_refs_per_method_rejected_with_e_spec_013` scaffolded. Previously only Rule C was scaffolded (2 proptests); skeleton now covers all 3 Rules (4 proptests total). Eliminates under-coverage risk for security-critical spec-load rejection layer. Harness file name kept as `vp153_sensorauth_cross_composition.rs` with scope note; test-writer may rename to match VP slug. |
| 0.6 | FB29 | 2026-05-16 | architect | F-LP37-MED-003 — Rule A/B/C message-format quotations updated to match canonical error-taxonomy.md v1.30 byte-for-byte (Option A: verbatim sync). Prior divergent strings were: Rule A `"Auth type cross-composition rejected for sensor '{sensor}': auth_type must be a single value from the enumerated set; got '{values}'"`, Rule B `"Multiple credential_refs for auth method '{method}' in sensor '{sensor}': exactly one credential_ref is required"`, Rule C `"Credential type mismatch for sensor '{sensor}': auth_type '{auth_type}' requires credential type '{expected}', got '{actual}'"`. Closes POL-24 (error_message_template_verbatim) violation surviving since pre-pass-37 era. TD-VSDD-060 sibling-site sweep: old divergent strings found only in VP-153 itself — no other spec artifact affected. POL-25 multi-cite sweep: E-SPEC-012/013/014 cited in 12+ artifacts; none misquote the canonical template. Future amendment discipline: POL-23/POL-25 sibling-sweep must include VP-153 Rule A/B/C whenever error-taxonomy.md E-SPEC-012/013/014 message_template fields are amended. |
| 0.5 | fix-burst-5 renumber-repair-redo | 2026-05-15 | state-manager | F-LP5-HIGH-003 renumber-repair-redo. FB4 assigned both the changelog-repair row and the modified-field-sync row to v0.4, producing two rows at the same version and violating monotonic strict order. Repair row renumbered 0.4→0.5. Absorbs FB4 modified-field-sync content: `modified:` field confirmed synced to ISO date "2026-05-15" per F-LP4-LOW-002 / POL-27. Content summary retained: prior changelog had non-monotonic sequence (0.1 → 0.3 → 0.3 → 0.2); corrected to 0.1 → 0.2 → 0.3 → 0.4 (state-manager POL-20 catch) → 0.5 (this row). Each distinct content change now holds a unique version. Frontmatter version updated to 0.5. Monotonic sequence verified: 0.1 → 0.2 → 0.3 → 0.4 → 0.5. |
| 0.4 | fix-burst-1 state-manager catch | 2026-05-15 | state-manager | (state-manager catch in fix-burst-1) F-LP1-HIGH-004 POL-20: introduced field canonicalized to ISO date 2026-05-15. Prior value `plugin-prereq-e` was informal slug; POL-20 requires `YYYY-MM-DD` for artifacts created outside greenfield cycles. |
| 0.3 | S-PLUGIN-PREREQ-E-reconciliation | 2026-05-15 | product-owner | VP153-OPEN-001 closed — E-SPEC-012/013/014 authored in error-taxonomy.md v1.25. BC-2.01.016 error references updated. Harness skeleton uses correct error code labels. Open Issues table updated with closure record. |
| 0.2 | plugin-prereq-e-cross-review | 2026-05-15 | architect | Q3 resolution: remove "error code to be assigned" placeholder. Assign E-SPEC-012 (Rule A), E-SPEC-013 (Rule B), E-SPEC-014 (Rule C). Document E-SPEC-010 collision (already taken). Update source_bc to BC-2.01.016 (primary auth-surface BC). Route E-SPEC-012/013/014 authoring to PO via VP153-OPEN-001. |
| 0.1 | plugin-prereq-e-adr-burst | 2026-05-15 | architect | Initial stub. Traces to ADR-026 D3 / ADR-023 Rule 2 / DI-012 runtime enforcement replacement. Harness skeleton provided; full authoring in S-PLUGIN-PREREQ-E test-writer dispatch. |
