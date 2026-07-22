---
document_type: behavioral-contract
level: L3
version: "1.13"
status: active
producer: product-owner
timestamp: 2026-05-16T12:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001"
lifecycle_status: active
introduced: "2026-05-15"
modified: "2026-07-22"
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
input-hash: "7d51805"
traces_to: ["CAP-001"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.01.016: SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker)

## Description

The `SensorAuth` trait in `crates/prism-sensors/src/auth/mod.rs` is unsealed: the
`private::Sealed` marker supertrait is removed so that `.prx` WASM plugin authors and
external Rust crate authors can implement `SensorAuth` directly. Cross-sensor
auth-composition prevention moves entirely to the three runtime spec-validation rules
established by ADR-023 Rule 2, which are enforced at TOML spec-load time and
credential-resolution time. No compile-time sealing barrier survives in v1.0. The
companion `CustomAuth` placeholder type in `crates/prism-spec-engine/src/custom_adapter.rs`
is deleted in this story because it existed solely to work around the sealed-trait
restriction.

## Preconditions

- `S-PLUGIN-PREREQ-F` has merged: BC-2.01.013 amended to remove sealed-trait language;
  DI-012 downgraded from compile-time to runtime enforcement; ADR-023 Rule 2 ratified.
- `S-PLUGIN-PREREQ-D` has merged: `PluginRuntime` is wired into the boot sequence; `.prx`
  plugins can be loaded and invoked via the WASM plugin escape hatch.
- `S-PLUGIN-PREREQ-A` has merged: `SensorId(Arc<str>)` is the canonical open sensor-identity
  type; `SensorType` closed enum is deleted.
- No external Rust crate has been published from this workspace with `SensorAuth` as a
  sealed public trait (confirmed by ADR-023 Rule 5 publication-history determination —
  "since `prism-spec-engine` has never been published to crates.io with CustomAdapter
  exposed, no deprecation grace period is required" — and PLUGIN-AUDIT-001 HIGH-3
  dead-code confirmation — "CustomAdapterRegistry and CustomAdapter Rust trait are RETIRED;
  locals are created and immediately dropped at the end of the boot function scope").
- The `SensorAuth` trait has exactly the following two object-safe methods per ADR-026 D1:
  - `fn as_any(&self) -> &dyn std::any::Any` — enables concrete-type recovery via downcast; impls that return an incorrect type produce a failed downcast (`None`), not undefined behavior
  - `fn auth_type_name(&self) -> &'static str` — declares the `auth_type` variant string (e.g., `"oauth2_client_credentials"`); `&'static str` ensures zero-cost vtable dispatch without allocation

## Postconditions

- The `private` module (or equivalent) that defines the `Sealed` marker trait is removed
  from `crates/prism-sensors/src/auth/mod.rs`. The `SensorAuth: Sealed` supertrait bound is
  removed from the `SensorAuth` trait definition.
- The `SensorAuth` trait is `pub` and fully implementable by any Rust code without special
  access. The trait signature remains object-safe (`dyn SensorAuth` is a valid type bound).
- The companion `CustomAuth` duplicate struct in `crates/prism-spec-engine/src/custom_adapter.rs`
  is deleted. This struct existed solely to proxy around the sealed trait; unsealing makes it
  dead code.
- Cross-sensor auth-composition is still rejected — not at compile time but at runtime by
  the three ADR-023 Rule 2 spec-validation rules:
  1. `SensorSpec.auth_type` must be a single value from the canonical enumerated set; arrays
     or mixed types are rejected at spec-load time (E-SPEC-012).
  2. Each auth method declares exactly one `credential_ref` binding; multiple bindings are
     rejected at spec-load time (E-SPEC-013).
  3. The resolved credential type must structurally match the declared `auth_type`; mismatches
     are rejected at credential-resolution time (E-SPEC-014).
- The four built-in sensor auth implementations (`CrowdStrikeAuth`, `CyberintAuth`,
  `ClarotyAuth`, `ArmisAuth`) in `crates/prism-sensors/src/auth/` continue to implement
  `SensorAuth` with one new method body per impl (`fn auth_type_name(&self) -> &'static str { "..." }`
  returning the static auth-type name for that implementation) and no other changes to their impl
  blocks. The `auth_type_name()` body is required because the 2-method trait surface (ADR-026 D1)
  mandates it; the sealed-trait supertrait removal otherwise requires zero changes to these impls.
- A compile-fail perimeter test (or updated grep gate) confirms that `private::Sealed` or
  equivalent sealed-marker import does NOT appear in `crates/prism-sensors/src/auth/mod.rs`
  after this change. If a perimeter test crate enforces sealed-trait invariants, it is updated
  to reflect the new open-trait policy.
- The `prism-spec-engine` public API no longer re-exports `CustomAuth`; the `pub use
  custom_adapter::CustomAuth` (or equivalent) re-export is removed from
  `crates/prism-spec-engine/src/lib.rs` in coordination with the `CustomAdapter` retirement
  (BC-2.16.011).

## Invariants

- **INV-AUTH-OPEN-001:** After this story merges, `grep -rn "private::Sealed\|impl Sealed\|trait Sealed" crates/prism-sensors/src/auth/` returns ZERO matches in production source.
- **INV-AUTH-OPEN-002:** The four built-in auth impls (`CrowdStrikeAuth`, `CyberintAuth`, `ClarotyAuth`, `ArmisAuth`) require exactly ONE new method body each — `fn auth_type_name(&self) -> &'static str { "..." }` — to satisfy the 2-method `SensorAuth` trait surface (ADR-026 D1). No other changes are made to these impl blocks. If any impl references a `Sealed` supertrait, that block is also removed — that removal is a pure deletion with no substitute.
- **INV-AUTH-OPEN-003:** Cross-sensor auth-composition prevention is fully preserved via ADR-023 Rule 2 runtime rules. The absence of compile-time sealing does NOT weaken the threat model. The three spec-load-time and credential-resolution-time rejection rules (E-SPEC-012/013/014) are the sole enforcement mechanism.
- **INV-AUTH-OPEN-004:** `dyn SensorAuth` remains a valid object-safe trait bound. No change to the trait method signatures is made in this story unless required by object-safety (i.e., no associated types are added without boxing).

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-012` | SensorSpec declares multiple auth types in the `auth_type` field, or declares a value outside the canonical enumerated set | Rejected at spec-load time; error cites ADR-023 Rule 2, Rule A. Credential value must not appear in error message (AD-017). |
| `E-SPEC-013` | Auth method has more than one `credential_ref` binding | Rejected at spec-load time; error cites ADR-023 Rule 2, Rule B |
| `E-SPEC-014` | Resolved credential structural type does not match declared `auth_type` | Rejected at credential-resolution time, before any HTTP request; error cites ADR-023 Rule 2, Rule C. Credential value must not appear in error message (AD-017). Backend qualification (D-706): Rule C fires when the credential backend exposes shape metadata via `CredentialRefProbe::probe()` returning `Some(shape)`. The current keyring backend returns `Ok(None)` (no shape metadata stored). Production enforcement is deferred to PLUGIN-MIGRATION-001-A; test-fixture enforcement (`ShapedProbe`) and VP-153 proptest provide regression coverage in PREREQ-E scope. |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-001 | External Rust code in `prism-spec-engine` implements `SensorAuth` for a custom plugin prototype | Compiles without error; runtime cross-sensor checks still apply via spec-validation |
| EC-016-002 | A `.prx` WASM plugin exports an `auth_type = "custom_via_plugin"` sensor spec | Plugin-provided `SensorAuth`-equivalent is resolved via `PluginRuntime`; falls under Rule 2 enforcement |
| EC-016-003 | `CrowdStrikeAuth` is compiled without any `Sealed` bound after removal | Still compiles; impl block requires exactly ONE new method body (`auth_type_name` returning a `&'static str` per ADR-026 §D2 Path B); no other changes to the impl block — the existing `as_any()` body and any inherent methods stay as-is. Only the sealed supertrait is removed from the trait definition. |
| EC-016-004 | `dyn SensorAuth + Send + Sync` bound used at a call site | Compiles; `SensorAuth` methods must remain object-safe; no change to method signatures in this story breaks object safety |
| EC-016-005 | A spec declares `auth_type = "custom_via_plugin"` for a sensor that has no loaded `.prx` plugin | Rejected at spec-load with an error indicating the custom plugin is not registered; tables are not made available |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.016-001 | Attempt to use `CrowdStrikeAuth` with `SensorSpec.auth_type = ["oauth2_client_credentials", "bearer_static"]` (array) | Spec-load rejected with E-SPEC-012; composite auth rejected |
| TV-BC-2.01.016-002 | Compile a test crate that implements `SensorAuth` for a custom struct without any `Sealed` import | Compiles successfully; confirms trait is publicly implementable |
| TV-BC-2.01.016-003 | `grep -rn "private::Sealed\|impl Sealed\|trait Sealed" crates/prism-sensors/src/auth/` | Returns zero matches (post-unsealing grep gate) |
| TV-BC-2.01.016-004 | `CrowdStrikeAuth` `SensorAdapter` integration test: `registry.register(org_id, arc_adapter)` then `registry.get(org_id, SensorId::from("crowdstrike"))` returns `Some` | Passes after unsealing; registry behavior unchanged |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-153 | SensorAuth Runtime Cross-Composition Prevention (proptest). Verifies that all valid `(auth_type, credential_type)` pairs are accepted and all invalid pairs are rejected, per ADR-023 Rule 2 / ADR-026 D3. Authored in PREREQ-E ADR burst; harness authored by test-writer in PREREQ-E test dispatch. |

## Related BCs

- BC-2.01.013 (DataSource Trait — Spec-Driven Adapter Pattern): parent contract; establishes that `SensorAuth` is NOT sealed per ADR-023 Rule 2 (amended in PREREQ-F). This BC operationalizes that amendment.
- BC-2.01.017 (StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection): child contract; specifies the `StaticCookieAuthProvider` concrete implementation for `auth_type = "cookie_roundtrip"` sensors. This BC establishes the `auth_type_name()` return value `"cookie_roundtrip"` as one entry in the 6-value canonical auth_type set; BC-2.01.017 specifies the full behavioral contract for that auth variant (no HTTP call during acquire_token, `Cookie: access_token={token}` header injection, E-AUTH-006 for empty/invalid key).
- BC-2.16.011 (CustomAdapter Rust Trait Retirement): sibling contract retired in this same story; the `CustomAuth` duplicate (which proxied around sealed `SensorAuth`) is deleted here.
- BC-2.16.012 (PluginRegistry Call-Site Migration): sibling contract; the PluginRegistry dispatch path opened by unsealing is exercised in spec_parser.rs migration sites.

## Architecture Anchors

- `crates/prism-sensors/src/auth/mod.rs` — sealed marker removal site
- `crates/prism-spec-engine/src/custom_adapter.rs` — `CustomAuth` deletion site (coordinated with BC-2.16.011)
- ADR-023 §Architectural Constraints (C5 bullet, Rule 2) — authoritative SensorAuth un-sealing specification
- ADR-026 — SensorAuth unsealing architectural decision; §D3 defines the three runtime enforcement rules that map to E-SPEC-012/013/014

## Story Anchor

S-PLUGIN-PREREQ-E

## VP Anchors

- VP-153 (SensorAuth Runtime Cross-Composition Prevention — proptest covering ADR-023 Rule 2 rejection rules)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| Capability Anchor Justification | CAP-001 ("Sensor Adapter Layer (Internal)") per capabilities.md §CAP-001. Un-sealing `SensorAuth` directly enables plugin authors to implement new sensor auth strategies that plug into the data-fetch adapter layer — the core mechanism by which sensors enumerate and fetch data. |
| L2 Invariants | DI-012 (amended: compile-time sealed-trait → runtime spec-validation per ADR-023 Rule 2; this BC implements the runtime-enforcement side) |
| Related BCs | BC-2.01.013 (parent adapter contract), BC-2.16.011 (CustomAdapter retirement), BC-2.16.012 (spec_parser migration) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.13 | wave-a-spec-evolution-burst-3 | 2026-07-22 | product-owner | ADR-054 D1 amendment: §Related BCs BC-2.01.017 reference updated from "5-value canonical auth_type set" to "6-value canonical auth_type set" — reflects addition of `token_exchange` as the 6th variant per ADR-054 D1 + DI-012 v1.8. modified date 2026-07-22. |
| 1.12 | D-849 | 2026-05-29 | product-owner | §Related BCs: added BC-2.01.017 (StaticCookieAuthProvider — No-Login-Roundtrip Cookie Injection) as child contract; BC-2.01.017 operationalizes the `"cookie_roundtrip"` entry in the 5-value canonical auth_type set established here. Cross-reference added per bc_array_changes_propagate_to_body_and_acs anchor-back policy. |
| 1.11 | D-776-post-merge | 2026-05-22 | state-manager | POL-14 verification (no-op confirm): PR #153 (PLUGIN-MIGRATION-001-D) squash-merged to develop@3f2de889 at 2026-05-22T09:05:47Z; status already active (promoted draft→active D-726 per POL-14 PR #151) — idempotent confirm. |
| 1.10 | D-726-post-merge | 2026-05-19 | state-manager | POL-14 auto-promotion at merge: PR #151 (S-PLUGIN-PREREQ-E) squash-merged to develop@80ebe794 at 2026-05-19T18:06:44Z; PR-LEVEL adversary cascade BC-5.39.001 3-CLEAN CONVERGED; status draft→active, lifecycle_status draft→active. |
| 1.9 | FB-IMPL-4 | 2026-05-18 | state-manager | D-707: §Error Cases E-SPEC-014 Behavior cell: backend qualification appended (D-706 architect adjudication text mechanically applied) — Rule C fires only when credential backend exposes shape metadata via `CredentialRefProbe::probe()` returning `Some(shape)`; current keyring backend returns `Ok(None)`; production enforcement deferred to PLUGIN-MIGRATION-001-A; test-fixture `ShapedProbe` + VP-153 proptest provide regression coverage in PREREQ-E scope. Closes F-LP-IMPL-P5-001 spec-amendment route. |
| 1.8 | FB51 | 2026-05-17 | product-owner | F-LP63-MED-001 closure: §Preconditions lines 54-55 PLUGIN-AUDIT-001 HIGH-3 mis-anchored citation corrected to Option (a) split provenance — publication-history routed to ADR-023 Rule 5 (correct source per ADR-027 Context lines 48-49); dead-code claim routed to PLUGIN-AUDIT-001 HIGH-3 (correct source); restores bidirectional traceability. |
| 1.7 | FB34 | 2026-05-16 | product-owner | FB34 ADDENDUM: EC-016-003 Expected Behavior cell corrected — "impl block is unchanged" replaced with explicit "ONE new method body (`auth_type_name`) per ADR-026 §D2 Path B" phrasing. Resolves internal contradiction with §Postconditions + AC-2 (story) + INV-AUTH-OPEN-002 + ADR-026 D1/D2. Within-FB34 sibling-sweep extension per pattern-breaking discipline (POL-29 candidate codification candidate). |
| 1.6 | FB31 | 2026-05-16 | product-owner | F-LP40-MED-001 §Traceability "Capability Anchor Justification" — replaced fabricated quoted-attribution "Enumerate and fetch data from sensor APIs" with verbatim CAP-001 title "Sensor Adapter Layer (Internal)" per capabilities.md (POL-22 Phase A; POL-7 5-citation-surface verbatim discipline; aligns with sibling BC-2.16.011/2.16.012 verbatim CAP-029 citation form). |
| 1.5 | prereq-e-fix-burst-19 | 2026-05-16 | state-manager | F-LP21-HIGH-001 closure — §Changelog renumber-repair-redo (D-611-equivalent pattern applied to sibling BC that was missed in FB14): state-manager catch row v1.2 → v1.3, cascade shift v1.3 → v1.4 (and v1.4 → v1.5 via new repair row insertion). POL-26 monotonic strict-ordering violation pre-existing FB1 (invisible to passes 1-20) now resolved. |
| 1.4 | prereq-e-fix-burst-3 | 2026-05-15 | product-owner | F-LP3-HIGH-002 closure (joint with architect): §Postconditions "without change" rewritten to "with one new method body per impl (`fn auth_type_name(&self) -> &'static str { \"...\" }`)"; INV-AUTH-OPEN-002 rewritten to match — 4 impls require exactly ONE new method body (auth_type_name) per ADR-026 D1 2-method trait surface. Preconditions already listed 2-method surface correctly (fix-burst-1); this is Postconditions/Invariants alignment only. |
| 1.3 | fix-burst-1 state-manager catch | 2026-05-15 | state-manager | (state-manager catch in fix-burst-1) F-LP1-HIGH-004 POL-20: introduced field canonicalized to ISO date 2026-05-15. Prior value `S-PLUGIN-PREREQ-E` was story-ID format; POL-20 requires `YYYY-MM-DD` for artifacts created outside greenfield cycles. |
| 1.2 | S-PLUGIN-PREREQ-E-fix-burst-1 | 2026-05-15 | product-owner | F-LP1-HIGH-001 closure: §Preconditions method surface aligned to ADR-026 D1 (2-method trait: `as_any()` + `auth_type_name()`). Removed incorrect 3-method list (`sensor_id`, `auth_type`, `build_request_auth`) which imported methods from a different trait surface. F-LP1-HIGH-003 closure: §C5 phantom-heading citations corrected — `ADR-023 §C5 Rule 2` → `ADR-023 §Architectural Constraints (C5 bullet, Rule 2)` per POL-21; ADR-023 has no `## C5` heading. |
| 1.1 | S-PLUGIN-PREREQ-E-reconciliation | 2026-05-15 | product-owner | Q1 fix: corrected error codes for ADR-023 Rule 2 rejections — E-SPEC-010/011/012 → E-SPEC-012/013/014 (E-SPEC-010 = variable interpolation field-path miss; E-SPEC-011 = pipe_verb reserved keyword; these pre-existing codes were incorrectly cited). New codes authored in error-taxonomy.md v1.25. VP-153 anchor added (cross-composition proptest). ADR-026/027 architecture anchors confirmed. |
| 1.0 | S-PLUGIN-PREREQ-E-authoring | 2026-05-15 | product-owner | Initial draft. Authored for S-PLUGIN-PREREQ-E; operationalizes ADR-023 §Architectural Constraints (C5 bullet, Rule 2) SensorAuth un-sealing. |
