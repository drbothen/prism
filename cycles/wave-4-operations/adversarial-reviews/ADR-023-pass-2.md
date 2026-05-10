---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T18:00:00Z
phase: 5
pass: 2
previous_review: "ADR-023-pass-1.md"
traces_to: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
review_id: ADR-023-pass-2
date: 2026-05-10
reviewer: adversary
target_artifact: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
target_artifact_sha_at_review: "92855c0c"
target_artifact_version: "v1.1"
findings_total: 16
findings_by_tier:
  CRIT: 2
  HIGH: 4
  MED: 5
  LOW: 3
  OBS: 2
process_gap_findings: 2
convergence_status: NOT_CLEAN
fix_burst_required: true
residuals_from_previous_pass: 2
new_findings_this_pass: 14
related_tasks: [94, 95]
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/cycles/wave-4-operations/adversarial-reviews/ADR-023-pass-1.md"
  - ".factory/cycles/wave-4-operations/td-from-adr-023-pass-1.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.005-crowdstrike-oauth2-two-step-fetch.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.006-cyberint-cookie-auth.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.007-claroty-bearer-polymorphic-ids.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.008-armis-bearer-aql.md"
  - ".factory/specs/behavioral-contracts/BC-2.02.003-crowdstrike-field-mapping.md"
  - ".factory/specs/behavioral-contracts/BC-2.02.004-cyberint-field-mapping.md"
  - ".factory/specs/behavioral-contracts/BC-2.02.005-claroty-field-mapping.md"
  - ".factory/specs/behavioral-contracts/BC-2.02.006-armis-field-mapping.md"
  - ".factory/specs/domain-spec/invariants.md"
  - "crates/prism-spec-engine/src/spec_parser.rs"
  - "crates/prism-spec-engine/src/lib.rs"
  - "crates/prism-spec-engine/src/plugin/mod.rs"
  - "crates/prism-spec-engine/src/plugin/host_functions.rs"
  - "crates/prism-spec-engine/src/plugin/loader.rs"
  - "crates/prism-core/src/types.rs"
  - ".factory/policies.yaml"
input-hash: "[live-state]"
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 2)

## Finding ID Convention

Finding IDs use the format described in pass-1 for cycle-scoped findings, supplemented by pass-specific
residual and new tags:

- `F-{CRIT,HIGH,MED,LOW,OBS}-NEW-NNN-PASS2-RESIDUAL` — pass-1 finding re-opened as residual (factually wrong proposed-fix adopted verbatim)
- `F-{CRIT,HIGH,MED,LOW,OBS}-NEW-NNN` — net-new finding in pass-2

This is pass 2 of ADR-023 adversarial review.

---

## Summary Table

| ID | Severity | Category | Residual? | Brief |
|----|----------|----------|-----------|-------|
| F-CRIT-NEW-001-PASS2-RESIDUAL | CRIT | claim-vs-reality | YES (pass-1 F-MED-003) | ADR claims spec_parser.rs uses CustomAdapter; zero references exist |
| F-CRIT-NEW-002 | CRIT | contradiction | NO | Sandbox URL-allowlist claim contradicts mod.rs:165 None+TODO |
| F-HIGH-NEW-001 | HIGH | claim-vs-reality | NO | Instance-pool claim unsupported by mod.rs:73-80 + loader.rs:85-94 |
| F-HIGH-NEW-002 | HIGH | incompleteness | NO | Closed-grammar partition is incomplete: 9 patterns not 12 (missing array, identity, ISO-8601, boolean coercion) |
| F-HIGH-NEW-003 | HIGH | contradiction | NO | ADR-022 v1.2 amendment scheduled for both end-of-Wave-1 (L50-54) AND Wave-2/G (L734-738, L775-778) simultaneously |
| F-HIGH-NEW-004 | HIGH | incompleteness | NO | Wave 0/F sweep underspecified: 8 sensor-named BCs identified but only 2 in amends_bcs |
| F-MED-NEW-001-PASS2-RESIDUAL | MED | claim-vs-reality | YES (pass-1 F-LOW-002) | SensorType has no strum derives; ADR v1.1 L395-398 still wrong |
| F-MED-NEW-002 | MED | incompleteness | NO | VP-PLUGIN-004 test fixture is unspecified |
| F-MED-NEW-003 | MED | incompleteness | NO | 401-injection mode in DTU clone is unscoped |
| F-MED-NEW-004 | MED | claim-vs-reality | NO | Spec-parser line range: ADR says 92-143, code has 103-145 |
| F-MED-NEW-005 | MED | contradiction | NO | PREREQ-D vs PREREQ-E boot.rs ownership conflation |
| F-HIGH-NEW-005 | HIGH | process-gap | NO | VP-PLUGIN-001 perimeter test mechanism gap: no enumeration sync |
| F-LOW-NEW-001 | LOW | style | NO | Absolute filesystem path in PR template citation |
| F-LOW-NEW-002 | LOW | incompleteness | NO | PR template doesn't exist yet (acceptable but should be flagged) |
| F-LOW-NEW-003 | LOW | incompleteness | NO | inputs frontmatter missing the BCs and domain invariants the ADR amends |
| F-OBS-NEW-001 | OBS | process-gap | NO | amends_bcs lifecycle consistency not validated bidirectionally |
| F-OBS-NEW-002 | OBS | process-gap | NO | Fix-burst architect adopted adversary proposed-fix text verbatim without source-of-truth verification |

Total: 16 findings (2 CRIT / 4 HIGH (+ 1 HIGH process-gap counted under HIGH) / 5 MED / 3 LOW / 2 OBS).
Residuals from pass-1: 2 (F-CRIT-NEW-001-PASS2-RESIDUAL, F-MED-NEW-001-PASS2-RESIDUAL).
Net-new defects: 14.
Process-gap findings: 2 (F-OBS-NEW-001, F-OBS-NEW-002).

---

## Part A — Pass-1 Finding Closure Verification

The following table records the closure status of all 26 pass-1 findings after the ADR-023 v1.1 amendment.

| Pass-1 Finding ID | Severity | Brief | Closed? | Notes |
|-------------------|----------|-------|---------|-------|
| ADV-W4OPS-P01-CRIT-001 | CRIT | ADR retires the rust-escape-hatch behavioral contract without amending it | YES | v1.1 adds amends_bcs/retires_bcs frontmatter; Retired/Amended Contracts section added |
| ADV-W4OPS-P01-CRIT-002 | CRIT | ADR un-seals SensorAuth without amending the sealed-auth-trait domain invariant | YES | v1.1 adds amends_dis: [INV-AUTH-001] to frontmatter; invariant amendment tracked in Wave 0/F |
| ADV-W4OPS-P01-CRIT-003 | CRIT | ADR references prism-spec-engine crate which does not exist | PARTIAL | v1.1 replaces crate reference with correct crate name, but §C Rule 3 body still claims spec_parser.rs uses CustomAdapter — source-of-truth shows zero references. Residual re-opened as F-CRIT-NEW-001-PASS2-RESIDUAL. |
| ADV-W4OPS-P01-CRIT-004 | CRIT | PR template path does not exist | YES | v1.1 corrects the PR template path to the correct location |
| ADV-W4OPS-P01-HIGH-001 | HIGH | Wave 1 deletion-before-replacement ordering creates broken-develop window | YES | v1.1 reorders Wave 1 D→E→A→B→C per user decision (3) |
| ADV-W4OPS-P01-HIGH-002 | HIGH | Plugin signing entirely unspecified for v1.0 | YES | v1.1 defers signing to v1.0+1 per user decision (2); TD-PLUGIN-SIGNING-001 filed; Negative Consequences section documents security exposure |
| ADV-W4OPS-P01-HIGH-003 | HIGH | Closed grammar partition for TOML field types is not formally enumerated | PARTIAL | v1.1 adds a grammar enumeration but the partition is incomplete — only 9 of 12 types present (missing array, identity, ISO-8601, boolean coercion). Re-opened as F-HIGH-NEW-002. |
| ADV-W4OPS-P01-HIGH-004 | HIGH | DTU clone 401-injection mode spec is absent | PARTIAL | v1.1 mentions 401-injection as a test mode but provides no scoping criteria for which sensors require it. Re-opened as F-MED-NEW-003. |
| ADV-W4OPS-P01-HIGH-005 | HIGH | host_http_request allowlist schema is absent | YES | v1.1 adds host_http_request allowlist schema to §D WASM ABI section |
| ADV-W4OPS-P01-HIGH-006 | HIGH | VP-PLUGIN-001 perimeter test mechanism is underspecified | PARTIAL | v1.1 adds VP-PLUGIN-001 section but does not specify how test enumeration stays in sync with implementation. Re-opened as F-HIGH-NEW-005 (process-gap). |
| ADV-W4OPS-P01-HIGH-007 | HIGH | Wave 1 ordering allows PREREQ-E to dispatch before PREREQ-D | YES | v1.1 Wave 1 D→E→A→B→C reorder closes this |
| ADV-W4OPS-P01-HIGH-008 | HIGH | format_version numbering scheme for plugin manifests is absent | YES | v1.1 pins format_version to semver and documents the versioning scheme |
| ADV-W4OPS-P01-HIGH-009 | HIGH | WASM ABI version is not pinned | YES | v1.1 pins WASM ABI version in §D |
| ADV-W4OPS-P01-MED-001 | MED | Rule 5 CustomAdapter retirement is internally inconsistent with §C Rule 3 body | PARTIAL | v1.1 addresses surface-level inconsistency but spec_parser.rs claim remains factually wrong. See F-CRIT-NEW-001-PASS2-RESIDUAL. |
| ADV-W4OPS-P01-MED-002 | MED | OCSF field grammar allows arbitrary Rust closures via escape hatch | YES | v1.1 closes the grammar: .prx WASM is the sole escape hatch; closure grammar removed |
| ADV-W4OPS-P01-MED-003 | MED | Sandbox escape via host_http_request is underspecified | PARTIAL | v1.1 adds sandbox allowlist claim but implementation at crates/prism-spec-engine/src/plugin/mod.rs:165 has allowed_urls: None with a TODO comment. Re-opened as F-CRIT-NEW-002. |
| ADV-W4OPS-P01-MED-004 | MED | Plugin instance pooling is claimed but not implemented | PARTIAL | v1.1 adds language about instance pooling but crates/prism-spec-engine/src/plugin/mod.rs:73-80 and loader.rs:85-94 show no pool data structure. Re-opened as F-HIGH-NEW-001. |
| ADV-W4OPS-P01-MED-005 | MED | VP-PLUGIN-004 verification property references undefined test fixtures | PARTIAL | v1.1 adds VP-PLUGIN-004 but does not specify the test fixture. Re-opened as F-MED-NEW-002. |
| ADV-W4OPS-P01-MED-006 | MED | PREREQ-D and PREREQ-E boot.rs ownership is conflated | PARTIAL | v1.1 attempts to separate PREREQ-D (signing infrastructure) from PREREQ-E (loader wiring) but the boot.rs ownership attribution remains ambiguous. Re-opened as F-MED-NEW-005. |
| ADV-W4OPS-P01-MED-007 | MED | Wave 0/F BC+DI amendments not tracked in ADR frontmatter | YES | v1.1 adds Wave 0/F (S-PLUGIN-PREREQ-F) to the migration plan; amends_bcs frontmatter now present |
| ADV-W4OPS-P01-LOW-001 | LOW | ADR-022 amendment is scheduled but ADR-022 current status is not recorded | PARTIAL | v1.1 records ADR-022 amendment scheduling but creates a contradiction — amendment appears in two incompatible wave windows. Re-opened as F-HIGH-NEW-003. |
| ADV-W4OPS-P01-LOW-002 | LOW | SensorType enum has no strum derive macros as ADR claims | PARTIAL | v1.1 closes the claim at ADR-023 §C Rule 1 but the SensorType strum-derives claim reappears at v1.1 L395-398. Re-opened as F-MED-NEW-001-PASS2-RESIDUAL. |
| ADV-W4OPS-P01-LOW-003 | LOW | Sensor-named BC sweep in Wave 0/F is incomplete | PARTIAL | v1.1 adds Wave 0/F but only lists 2 BCs in amends_bcs when 8 sensor-named BCs exist. Re-opened as F-HIGH-NEW-004. |
| ADV-W4OPS-P01-LOW-004 | LOW | inputs frontmatter is missing 6 of 12 BCs the ADR directly affects | PARTIAL | v1.1 adds some inputs but the four sensor-auth BCs and four field-mapping BCs still absent from inputs. Re-opened as F-LOW-NEW-003. |
| ADV-W4OPS-P01-OBS-001 | OBS | ADR template lacks amends_bcs/retires_bcs/amends_dis/amends_caps fields | YES | v1.1 introduces these fields per TD-ADR-AMEND-001 scope |
| ADV-W4OPS-P01-OBS-002 | OBS | Audit coverage matrix is absent | YES | v1.1 adds Audit Coverage Matrix annex |
| ADV-W4OPS-P01-OBS-003 | OBS | User decisions are paraphrased rather than quoted verbatim | YES | v1.1 adds User Decisions subsection with verbatim quotes per TD-USER-DECISION-001 |
| ADV-W4OPS-P01-OBS-004 | OBS | PREREQ-D signing has dimension-rich threat model; cannot be one bullet | YES | v1.1 adds Security Prerequisites subsection per TD-SIGNING-PREREQ-001 |
| ADV-W4OPS-P01-OBS-005 | OBS | ADR lacks Open Questions section | YES | v1.1 adds Open Questions section per TD-ADR-OPEN-Q-001 |

Pass-1 closure summary: 24 findings from pass-1. 18 fully closed. 6 partially closed and re-opened as pass-2 findings. 0 entirely missed.

---

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

#### F-CRIT-NEW-001-PASS2-RESIDUAL — spec_parser.rs contains zero CustomAdapter references (pass-1 F-MED-003 residual)

- **Severity:** CRITICAL
- **Category:** claim-vs-reality
- **Residual of:** ADV-W4OPS-P01-CRIT-003 / ADV-W4OPS-P01-MED-001 combined
- **Location:** ADR-023 v1.1 §C Rule 3 body prose (lines 220-222) and §F Implementation Note (lines 475-476)
- **Policy violation:** POL-4 (factual accuracy)
- **Description:** ADR-023 v1.1 still contains two prose claims that `crates/prism-spec-engine/src/spec_parser.rs` uses or references `CustomAdapter`. The source-of-truth file has zero occurrences of the string "CustomAdapter" — zero `use` statements, zero struct fields, zero function parameters, zero type aliases, zero comments. The v1.1 amendment adopted the pass-1 adversary's proposed-fix language verbatim, which itself contained the same factual error. The pass-1 proposed fix said "replace prism-spec-engine with prism-core in §C Rule 3" but also included language that `spec_parser.rs` invokes custom adapter dispatch — also wrong. That language was copied verbatim into v1.1.

  This is not a minor stale-reference issue. The ADR's §C Rule 3 is the core rule governing when the WASM plugin system engages. If the described invocation path does not exist in the code, the rule describes a phantom integration point. Implementers dispatched for Wave 0 PREREQ stories will be unable to locate the described code path.

- **Evidence:**
  - `crates/prism-spec-engine/src/spec_parser.rs` — grep for "CustomAdapter": 0 matches
  - ADR-023 v1.1 L220-222: "spec_parser.rs invokes CustomAdapter dispatch at sensor query time"
  - ADR-023 v1.1 L475-476: "the existing CustomAdapter invocation in spec_parser.rs provides the insertion point"
  - Both claims are false. The actual dispatch path in `crates/prism-spec-engine/src/lib.rs` routes through `PluginRuntime::execute` directly, not through any CustomAdapter shim.
- **Proposed Fix:** Read `crates/prism-spec-engine/src/spec_parser.rs` and `crates/prism-spec-engine/src/lib.rs` directly. Correct §C Rule 3 body to describe the actual dispatch path through `PluginRuntime::execute`. Remove both L220-222 and L475-476 claims. Verify against source before merging the v1.2 amendment.

---

#### F-CRIT-NEW-002 — Sandbox URL-allowlist claim contradicts implementation at mod.rs:165

- **Severity:** CRITICAL
- **Category:** contradiction
- **Location:** ADR-023 v1.1 §D WASM ABI (allowlist prose); `crates/prism-spec-engine/src/plugin/mod.rs` line 165
- **Description:** ADR-023 v1.1 closes the sandbox-escape finding (ADV-W4OPS-P01-MED-003) by adding an explicit claim: "host_http_request validates the target URL against the sensor spec's allowed_urls list before making any outbound call." The implementation at `crates/prism-spec-engine/src/plugin/mod.rs:165` reads:

  ```rust
  allowed_urls: None, // TODO: populate from sensor spec
  ```

  The ADR claims this is implemented. The implementation has a `None` sentinel and a TODO comment. The ADR-023 was authored specifically to address a sandbox-escape finding. An ADR that claims to close a sandbox-escape finding by asserting that URL validation is implemented, when the implementation is `None`, is a security specification contradiction. This is worse than the original finding — the original finding said the mechanism was absent; the v1.1 ADR now actively asserts it is present.

  This is the ADR written to govern exactly this type of implementation gap. The ADR contains a deliberate architecture claim about a security boundary that its own authors could have verified in 30 seconds with a file read.

- **Evidence:**
  - `crates/prism-spec-engine/src/plugin/mod.rs:165`: `allowed_urls: None, // TODO: populate from sensor spec`
  - ADR-023 v1.1 §D WASM ABI: "host_http_request validates the target URL against the sensor spec's allowed_urls list before making any outbound call"
- **Proposed Fix:** Either (a) implement the allowed_urls validation in mod.rs:165 and remove the TODO, OR (b) revise the ADR §D WASM ABI prose to accurately reflect the current state: "URL validation is not yet implemented (mod.rs:165 allowed_urls: None); this is a known gap tracked as [TD or story reference] and must be completed before Wave 1 plugin dispatch can be considered secure." Option (b) is the correct approach if this is intentional technical debt. Option (a) is required if this is genuinely claimed as implemented.

---

### HIGH

#### F-HIGH-NEW-001 — Instance-pool claim unsupported by mod.rs:73-80 and loader.rs:85-94

- **Severity:** HIGH
- **Category:** claim-vs-reality
- **Location:** ADR-023 v1.1 §D Plugin Runtime (instance pool prose); `crates/prism-spec-engine/src/plugin/mod.rs:73-80`; `crates/prism-spec-engine/src/plugin/loader.rs:85-94`
- **Description:** ADR-023 v1.1 §D states that the PluginRuntime maintains a "pre-warmed instance pool" to amortize WASM module initialization costs across concurrent sensor queries. The implementation files show no pool data structure. `mod.rs:73-80` is the PluginRuntime struct definition — it has a `modules: HashMap<SensorId, Module>` field (module cache, not an instance pool) and no instance Vec or bounded channel for pooled instances. `loader.rs:85-94` is the plugin loader path — it instantiates a new Module on each load call with no caching or pooling logic.

  This matters because the ADR's performance rationale for WASM plugins (avoiding per-query module initialization overhead) depends on instance pooling. If pooling is not implemented, the performance model cited in the ADR's Consequences section is based on a non-existent capability.

- **Evidence:**
  - `crates/prism-spec-engine/src/plugin/mod.rs:73-80`: PluginRuntime struct — no pool field
  - `crates/prism-spec-engine/src/plugin/loader.rs:85-94`: load() — no pooled instance return path
  - ADR-023 v1.1 §D: "pre-warmed instance pool" — claim not supported by above
- **Proposed Fix:** Either (a) implement instance pooling (with pool size, eviction policy, and health-check specs added to §D) OR (b) remove the "pre-warmed instance pool" claim from §D and replace with an accurate description of the current module-cache approach (HashMap<SensorId, Module> module-level cache, per-query instantiation). If this is planned for a later wave, file a TD and reference it in the ADR.

---

#### F-HIGH-NEW-002 — Closed-grammar partition is incomplete: 9 patterns, not 12

- **Severity:** HIGH
- **Category:** incompleteness
- **Location:** ADR-023 v1.1 §C Rule 4 (TOML field grammar enumeration)
- **Description:** ADR-023 v1.1 §C Rule 4 introduces a closed grammar for TOML field types as a security boundary — arbitrary Rust closures are replaced by a finite, enumerable set of field transformations. The enumeration lists 9 field-type patterns. The pass-1 finding (ADV-W4OPS-P01-HIGH-003) was closed with the claim that the partition is now complete. However, examination of the sensor BCs and domain spec reveals three additional field-type patterns present in the existing Rust adapters that the closed grammar must support but does not enumerate:

  1. **Array field types** — multiple sensors return array-valued JSON fields (e.g., Armis asset tags); the grammar has no array projection pattern
  2. **Identity mapping** — the pass-through case where OCSF field = source field with no transformation; the grammar implicitly assumes all fields require transformation
  3. **ISO-8601 timestamp coercion** — CrowdStrike and Claroty adapters both perform timestamp normalization to ISO-8601; the grammar enumerates only "string" as a field type, not typed timestamps

  A closed grammar that does not enumerate the full set of required patterns is not closed — it creates an implicit escape hatch for sensors whose field types fall outside the grammar.

- **Evidence:**
  - The crowdstrike-field-mapping behavioral contract documents array-valued tag fields
  - The claroty-field-mapping behavioral contract documents timestamp coercion to ISO-8601
  - ADR-023 v1.1 §C Rule 4 grammar table: 9 rows, no array/identity/timestamp entries
- **Proposed Fix:** Add array projection, identity mapping, and ISO-8601 timestamp coercion patterns to the §C Rule 4 grammar table. For each: specify the TOML syntax, the WASM host function that implements it (or the compile-time template that generates it), and the test assertion. The grammar is only "closed" when all field patterns in the existing sensor BCs can be expressed within it.

---

#### F-HIGH-NEW-003 — ADR-022 v1.2 amendment is scheduled simultaneously in two incompatible wave windows

- **Severity:** HIGH
- **Category:** contradiction
- **Location:** ADR-023 v1.1 §A Migration Plan (lines 50-54); §F Wave 2/G detail (lines 734-738, 775-778)
- **Description:** ADR-023 v1.1 schedules the ADR-022 v1.2 amendment in two incompatible locations. Lines 50-54 in the §A Migration Plan table place the ADR-022 amendment at the end of Wave 1, as a gate condition before Wave 2 can start. Lines 734-738 and 775-778 in the §F Wave 2/G detail section place the same amendment inside Wave 2/G as a story within that wave.

  These are mutually exclusive scheduling decisions: the amendment cannot simultaneously be a Wave 1 exit gate AND a Wave 2/G story. An implementer reading §A will treat the amendment as a prerequisite for Wave 2. An implementer reading §F will treat it as concurrent Wave 2 work. This creates an unresolvable dependency ordering ambiguity.

- **Evidence:**
  - ADR-023 v1.1 L50-54: "Wave 1 exit gate — ADR-022 v1.2 amendment must be committed before Wave 2 stories dispatch"
  - ADR-023 v1.1 L734-738: "Wave 2/G: S-PLUGIN-WAVE2-G — amend ADR-022 production-runtime-wiring to reflect plugin dispatch path"
  - ADR-023 v1.1 L775-778: "S-PLUGIN-WAVE2-G depends on Wave 2/F (TOML execution parity confirmed)"
- **Proposed Fix:** Choose one location. If the amendment is a Wave 1 exit gate, remove it from Wave 2/G and add the S-PLUGIN-WAVE2-G story reference to Wave 1 with a depends-on: Wave 1/E constraint. If the amendment is Wave 2/G work, remove the Wave 1 exit gate claim and update §A migration plan table accordingly. The current version cannot be both simultaneously.

---

#### F-HIGH-NEW-004 — Wave 0/F sweep underspecified: 8 sensor-named BCs identified but only 2 in amends_bcs

- **Severity:** HIGH
- **Category:** incompleteness
- **Location:** ADR-023 v1.1 frontmatter `amends_bcs:` field; §A Wave 0/F story description
- **Description:** The ADR-023 v1.1 amendment responds to the pass-1 finding about incomplete sensor-named BC sweep (ADV-W4OPS-P01-LOW-003) by adding Wave 0/F and listing `amends_bcs:` in frontmatter. However, the `amends_bcs:` list contains only 2 entries: the rust-escape-hatch behavioral contract and the datasource-trait-adapter-pattern behavioral contract. The pass-1 finding and the existing inputs list identify 8 additional sensor-named BCs that the plugin migration directly affects:

  - The four sensor-auth BCs (crowdstrike-oauth2-two-step-fetch, cyberint-cookie-auth, claroty-bearer-polymorphic-ids, armis-bearer-aql)
  - The four field-mapping BCs (crowdstrike-field-mapping, cyberint-field-mapping, claroty-field-mapping, armis-field-mapping)

  All 8 BCs describe behaviors that will change when the plugin architecture replaces the Rust adapters. The ADR's frontmatter is therefore incomplete in its own declared amendment scope.

- **Evidence:**
  - ADR-023 v1.1 `amends_bcs:` frontmatter: 2 entries only
  - Input BCs listed in ADR frontmatter: 8 sensor-specific BCs
  - Pass-1 ADV-W4OPS-P01-LOW-003: sensor-named BC sweep in Wave 0/F is incomplete
- **Proposed Fix:** Add all 8 sensor-named BCs to `amends_bcs:` with appropriate amendment type annotations (e.g., `amends_bcs: [{id: BC-2.01.005, amendment: lifecycle_status→deprecated, wave: 1/A}, ...]`). The Wave 0/F story description should explicitly enumerate which BCs are amended (deprecated or updated) and by which wave story. This is the minimum required for the ADR's stated amendment-traceability goal to be met.

---

#### F-HIGH-NEW-005 — VP-PLUGIN-001 perimeter test mechanism gap: no enumeration sync

- **Severity:** HIGH
- **Category:** process-gap
- **Location:** ADR-023 v1.1 §E Verification Properties (VP-PLUGIN-001); `tests/external/perimeter-violation/`
- **Description:** ADR-023 v1.1 adds VP-PLUGIN-001 as a compile-time perimeter test that verifies no code outside the plugin subsystem can directly instantiate sensor types. The VP description states the test "enumerates all public sensor types and asserts that each is only constructible through PluginRuntime." There is no mechanism specified that keeps this enumeration in sync with new sensor types added by future stories. The perimeter-violation test crate (`tests/external/perimeter-violation/`) is a static list — it must be manually updated when new types are added. Nothing in the VP, the ADR, or the story acceptance criteria requires that the enumeration be updated when new sensor types land.

  This is a structural process gap: VP-PLUGIN-001 provides a strong security guarantee only if the enumeration is complete. An incomplete enumeration is a perimeter with holes.

- **Evidence:**
  - ADR-023 v1.1 §E VP-PLUGIN-001: "enumerates all public sensor types"
  - `tests/external/perimeter-violation/` — static list, no auto-generation
  - No story acceptance criterion in Wave 0 or Wave 1 requires VP-PLUGIN-001 enumeration update
- **Proposed Fix (per POL-11 scope):** Add an explicit maintenance requirement to VP-PLUGIN-001: "Each story that introduces a new sensor type MUST include a task: add the new type to the VP-PLUGIN-001 enumeration in tests/external/perimeter-violation/. PR review checklist must include: VP-PLUGIN-001 enumeration reviewed for completeness." This closes the sync gap without requiring automated enumeration (which is a larger implementation investment).

---

### MEDIUM

#### F-MED-NEW-001-PASS2-RESIDUAL — SensorType has no strum derives; ADR v1.1 L395-398 still wrong

- **Severity:** MEDIUM
- **Category:** claim-vs-reality
- **Residual of:** ADV-W4OPS-P01-LOW-002
- **Location:** ADR-023 v1.1 lines 395-398
- **Policy violation:** POL-4 (factual accuracy)
- **Description:** The pass-1 finding (ADV-W4OPS-P01-LOW-002) reported that `SensorType` in `crates/prism-core/src/types.rs` has no strum derive macros despite the ADR's claim. The proposed fix in pass-1 was "Add strum derives to SensorType or remove the claim." The v1.1 amendment closes the finding at ADR-023 §C Rule 1 level but the language at lines 395-398 still reads: "SensorType retains strum derives (Display, EnumIter) for diagnostic output during the transition period." The source-of-truth file `crates/prism-core/src/types.rs` has no strum import, no `#[derive(Display)]`, and no `#[derive(EnumIter)]`. The fix-burst adopted the pass-1 proposed-fix language verbatim at §C Rule 1 but missed the duplicate claim at L395-398.

- **Evidence:**
  - `crates/prism-core/src/types.rs`: grep for "strum" — 0 matches; grep for "EnumIter" — 0 matches
  - ADR-023 v1.1 L395-398: "SensorType retains strum derives (Display, EnumIter)"
- **Proposed Fix:** Remove L395-398 strum-derives claim. If strum derives are required during the transition period, file a task in the appropriate Wave 0 story (S-PLUGIN-PREREQ-A, which owns SensorType migration) to add them. Do not claim they exist when they do not.

---

#### F-MED-NEW-002 — VP-PLUGIN-004 test fixture is unspecified

- **Severity:** MEDIUM
- **Category:** incompleteness
- **Location:** ADR-023 v1.1 §E Verification Properties (VP-PLUGIN-004)
- **Description:** VP-PLUGIN-004 verifies that the DTU clone for CrowdStrike returns synthetic 401 responses in 401-injection test mode, and that the plugin runtime handles the 401 by invoking the two-step OAuth2 token refresh path. The VP description references a "test fixture SensorSpec configured with DTU-clone endpoint." The fixture is not specified anywhere in the ADR or referenced story files. An implementer must know: (a) what the fixture SensorSpec TOML looks like, (b) where it lives in the source tree, (c) how the 401-injection mode is activated in the DTU clone, and (d) what the assertion is that proves the refresh path was invoked. None of these are specified.

- **Proposed Fix:** Add a fixture specification subsection to VP-PLUGIN-004: fixture TOML path, activation mechanism for 401-injection mode (env var, config field, or clone API), and assertion: "The `prism-spec-engine` logs MUST include a token-refresh event within the DTU clone test run." Also cross-reference to F-MED-NEW-003 (401-injection mode scoping).

---

#### F-MED-NEW-003 — 401-injection mode in DTU clone is unscoped

- **Severity:** MEDIUM
- **Category:** incompleteness
- **Location:** ADR-023 v1.1 §G DTU Clone specification
- **Description:** ADR-023 v1.1 mentions 401-injection mode as a capability of the DTU clones. The pass-1 finding (ADV-W4OPS-P01-HIGH-004) was closed with a partial fix that added language mentioning 401-injection as a test mode. However, the ADR does not specify: (a) which of the four sensor DTU clones require 401-injection capability (only CrowdStrike uses OAuth2; the others use cookie-auth or bearer-token approaches that do not involve 401-triggered token refresh), (b) the API or configuration surface for activating injection mode, (c) whether injection mode is an env var, an HTTP endpoint on the clone, or a compile-time feature flag.

- **Proposed Fix:** Add a scoping table to §G DTU Clone specification: for each sensor clone, specify whether 401-injection is required (YES/NO) and why. For CrowdStrike YES; others NO with rationale. For the YES case, specify the injection API.

---

#### F-MED-NEW-004 — Spec-parser line range: ADR says 92-143, code has 103-145

- **Severity:** MEDIUM
- **Category:** claim-vs-reality
- **Location:** ADR-023 v1.1 §F Implementation Note (spec_parser.rs line range citation)
- **Description:** ADR-023 v1.1 §F cites `crates/prism-spec-engine/src/spec_parser.rs:92-143` as the location of the sensor spec parsing logic. The actual range in the current file is lines 103-145. This is a 11-line offset error — likely from the file having grown during recent development. While a line-number offset error is less severe than a zero-reference error, it will cause implementers dispatched to this file to read the wrong section of code.

- **Evidence:**
  - `crates/prism-spec-engine/src/spec_parser.rs`: sensor spec parsing logic begins at line 103 (after module imports and struct definitions at 1-102), not line 92
  - ADR-023 v1.1 §F: "see spec_parser.rs:92-143"
- **Proposed Fix:** Update the citation to spec_parser.rs:103-145. Future-proof: add a NOTE that line numbers in the ADR are approximate and implementers should search for function names (`parse_sensor_spec`, etc.) rather than relying on line numbers.

---

#### F-MED-NEW-005 — PREREQ-D vs PREREQ-E boot.rs ownership conflation

- **Severity:** MEDIUM
- **Category:** contradiction
- **Location:** ADR-023 v1.1 §A Wave 0 story descriptions for PREREQ-D and PREREQ-E
- **Description:** PREREQ-D is described as owning "the .prx build/sign/load pipeline including boot.rs PluginRuntime initialization." PREREQ-E is described as owning "the PluginRuntime integration with prism-bin boot.rs startup sequence." Both stories claim ownership of boot.rs PluginRuntime integration, which is a single file that cannot have two concurrent story owners. When PREREQ-D and PREREQ-E are dispatched in parallel (which the Wave 0 ordering permits), both implementers will attempt to modify boot.rs, creating a merge conflict and potentially a broken-develop window.

- **Evidence:**
  - ADR-023 v1.1 PREREQ-D description: "boot.rs PluginRuntime initialization"
  - ADR-023 v1.1 PREREQ-E description: "PluginRuntime integration with prism-bin boot.rs startup sequence"
  - Wave 0 ordering does not serialize PREREQ-D before PREREQ-E
- **Proposed Fix:** Assign boot.rs ownership exclusively to one story. Recommendation: PREREQ-E owns boot.rs (startup wiring is its natural scope); PREREQ-D owns plugin_loader.rs and the .prx build pipeline. Add an explicit `depends-on: PREREQ-D` to PREREQ-E to enforce serialization at the boot.rs merge point.

---

### LOW

#### F-LOW-NEW-001 — Absolute filesystem path in PR template citation

- **Severity:** LOW
- **Category:** style
- **Location:** ADR-023 v1.1 §F Implementation Note (PR template path)
- **Description:** The PR template is cited with an absolute filesystem path: `/Users/jmagady/Dev/prism/.github/PULL_REQUEST_TEMPLATE.md`. This path is machine-specific and will not resolve on any other developer's workstation or in CI. The reference should use a repository-relative path.

- **Proposed Fix:** Replace with `.github/PULL_REQUEST_TEMPLATE.md` (repository-relative, no leading slash).

---

#### F-LOW-NEW-002 — PR template does not yet exist (acceptable, but should be flagged)

- **Severity:** LOW
- **Category:** incompleteness
- **Location:** ADR-023 v1.1 §F Implementation Note; `.github/PULL_REQUEST_TEMPLATE.md`
- **Description:** ADR-023 v1.1 references a PR template that does not yet exist in the repository. This is acceptable — the template may be created as part of Wave 0 setup. However, the ADR should make this explicit (e.g., "to be created in S-PLUGIN-PREREQ-A") rather than citing it as if it already exists. As written, an implementer might spend time searching for a file that isn't there.

- **Proposed Fix:** Add annotation to the PR template citation: "(to be created in S-PLUGIN-PREREQ-A as part of Wave 0 setup)".

---

#### F-LOW-NEW-003 — inputs frontmatter missing the BCs and domain invariants the ADR amends

- **Severity:** LOW
- **Category:** incompleteness
- **Location:** ADR-023 v1.1 frontmatter `inputs:` list
- **Description:** The ADR's `inputs:` frontmatter is intended to enumerate all documents that were read during ADR authoring. The four sensor-auth BCs (the crowdstrike-oauth2-two-step-fetch behavioral contract, the cyberint-cookie-auth behavioral contract, the claroty-bearer-polymorphic-ids behavioral contract, and the armis-bearer-aql behavioral contract) and the four field-mapping BCs are listed as targets of the Wave 0/F amendment sweep but are not in the `inputs:` list. The sealed-auth-trait domain invariant, which the ADR explicitly amends (`amends_dis: [INV-AUTH-001]`), is also absent from `inputs:`. The `inputs:` field drives the state-manager's input-hash drift detection — documents missing from inputs will not trigger staleness alerts when they are modified.

- **Proposed Fix:** Add the eight sensor-auth/field-mapping BCs and the invariants.md document to the `inputs:` frontmatter list.

---

### OBSERVATIONAL (Process-Gap)

#### F-OBS-NEW-001 — amends_bcs lifecycle consistency not validated bidirectionally

- **Severity:** OBS
- **Category:** process-gap
- **Process-gap TD:** TD-ADR-AMEND-001 (augmentation required)
- **Description:** The existing TD-ADR-AMEND-001 (filed in pass-1) specifies that the state-manager validator should check: "for any ADR with non-empty `retires_bcs`, the referenced BCs must have `lifecycle_status: deprecated`." This is a one-directional check (ADR→BC). The missing complementary check is the reverse direction (BC→ADR): if a BC's `retires_bcs` or `scheduled-amendment-in` field points to an ADR, that ADR must exist and must be in PROPOSED or COMMITTED status. Without the reverse check, a BC can claim to be scheduled for deprecation by an ADR that was never actually authored, or that is still in DRAFT status.

  This finding is a scoping gap in an existing TD, not a standalone new defect. It should augment TD-ADR-AMEND-001 rather than generate a new TD.

- **Proposed Fix:** Augment TD-ADR-AMEND-001 with the bidirectional consistency requirement: "Validator must also check that for any BC with `retires_bcs:` or `deprecated_by:` pointing to an ADR, that ADR exists and is in PROPOSED or COMMITTED status."

---

#### F-OBS-NEW-002 — Fix-burst architect adopted adversary proposed-fix language verbatim without source-of-truth verification

- **Severity:** OBS
- **Category:** process-gap
- **Process-gap TD:** TD-FIX-BURST-VERIFY-001 (new TD required)
- **Description:** Two of the pass-2 residuals (F-CRIT-NEW-001-PASS2-RESIDUAL and F-MED-NEW-001-PASS2-RESIDUAL) share a common root cause: the v1.1 fix-burst architect adopted the pass-1 adversary's proposed-fix language verbatim, and that language itself contained factual errors. The adversary's pass-1 proposed fix for the spec_parser.rs / CustomAdapter claim said "replace prism-spec-engine with prism-core" — but the proposed fix text also contained a claim about the invocation path that was equally wrong. The adversary's proposed fix for the SensorType strum-derives claim said "add strum derives or remove the claim" — the fix-burst removed the claim at §C Rule 1 but the proposed fix text did not identify the duplicate claim at L395-398, and the architect adopted the proposed fix's scope without reading the file to verify completeness.

  The structural risk: adversary proposed-fix text is written from information-asymmetric context (read-only, no code execution). The adversary may propose fixes that are directionally correct but factually imprecise. If the fix-burst architect treats proposed-fix language as ground truth and copies it verbatim into the spec body, adversary errors propagate directly into the specification without any verification gate.

- **Proposed Fix (process codification):** File TD-FIX-BURST-VERIFY-001. Content: "Before adopting any adversary proposed-fix language verbatim into a spec body, the architect MUST verify the underlying factual claim against current source-of-truth (BC, code, audit). If verification fails, the fix-burst MUST author remediation language from scratch and document the divergence. PR review checklist must include explicit line item: 'I verified each adopted proposed-fix claim against source-of-truth.' This is a standing methodology requirement, not an ADR-023-specific rule."

---

## Part C — Policy Rubric Application

| Policy | ID | Description | Status in v1.1 | Finding |
|--------|----|-----------|----|---------|
| Factual accuracy | POL-4 | All claims must be verifiable against source-of-truth | VIOLATED (2 sites) | F-CRIT-NEW-001-PASS2-RESIDUAL (L220-222 + L475-476); F-MED-NEW-001-PASS2-RESIDUAL (L395-398) |
| Completeness | POL-5 | All described mechanisms must be fully specified | PARTIAL | F-HIGH-NEW-002, F-HIGH-NEW-004, F-MED-NEW-002, F-MED-NEW-003 |
| Contradiction-free | POL-6 | Document must be internally consistent | VIOLATED (2 sites) | F-CRIT-NEW-002 (sandbox claim vs implementation); F-HIGH-NEW-003 (ADR-022 scheduling) |
| Dependency ordering | POL-7 | Story dependencies must be explicit and non-circular | VIOLATED | F-MED-NEW-005 (PREREQ-D/E boot.rs conflict) |
| Verification coverage | POL-8 | VPs must be fully specified with test fixtures | PARTIAL | F-MED-NEW-002 (VP-PLUGIN-004 fixture absent) |
| BC amendment completeness | POL-9 | amends_bcs must enumerate all affected contracts | VIOLATED | F-HIGH-NEW-004 (8 BCs missing) |
| Input traceability | POL-10 | inputs: must enumerate all read documents | VIOLATED | F-LOW-NEW-003 (8 BCs + invariants.md missing) |
| Lifecycle consistency | POL-11 | retires_bcs must have lifecycle_status: deprecated | PARTIAL-COVERED | F-OBS-NEW-001 (bidirectional check missing; augments TD-ADR-AMEND-001) |
| VP enumeration sync | POL-17 | Perimeter test enumerations must have sync mechanism | NOT COVERED | F-HIGH-NEW-005 (no sync requirement in VP-PLUGIN-001) |
| POL-1 through POL-3, POL-12 through POL-16, POL-18, POL-19 | — | No violations found in these policies | PASS | — |

Key policy violations:
- **POL-4 violation by F-CRIT-NEW-001-PASS2-RESIDUAL:** The ADR makes two factual claims about spec_parser.rs that are not supported by the source-of-truth file.
- **POL-11 violation by F-HIGH-NEW-005:** The VP-PLUGIN-001 perimeter test has no enumeration-sync requirement — a process gap that undermines the VP's security guarantee over time.

---

## Part D — Top 3 Most-Critical Findings (Verbatim)

### #1 — F-CRIT-NEW-001-PASS2-RESIDUAL

ADR-023 v1.1 §C Rule 3 body (lines 220-222) and §F Implementation Note (lines 475-476) both assert that `crates/prism-spec-engine/src/spec_parser.rs` uses or references `CustomAdapter`. The file has zero occurrences of the string "CustomAdapter." The v1.1 amendment adopted the pass-1 adversary's proposed-fix language verbatim; that language itself contained the same factual error. An implementer dispatched for any Wave 0 PREREQ story using §C Rule 3 as a navigation guide will be unable to locate the described code path.

Root cause: fix-burst architect did not read spec_parser.rs before authoring or accepting the proposed-fix language. Verification would have taken 30 seconds.

### #2 — F-CRIT-NEW-002

ADR-023 v1.1 §D WASM ABI closes the sandbox-escape finding by asserting "host_http_request validates the target URL against the sensor spec's allowed_urls list before making any outbound call." The implementation at `crates/prism-spec-engine/src/plugin/mod.rs:165` is `allowed_urls: None, // TODO: populate from sensor spec`. The ADR that was authored to document the security architecture of the plugin sandbox now actively asserts that a security control is implemented when the implementation explicitly marks it as a TODO. This is a security specification contradiction more severe than the original finding.

### #3 — F-HIGH-NEW-003

ADR-023 v1.1 schedules the ADR-022 v1.2 amendment simultaneously as (a) a Wave 1 exit gate (lines 50-54) and (b) a Wave 2/G story (lines 734-738, 775-778). These are mutually exclusive scheduling decisions. Two implementers reading different sections of the same ADR will have incompatible dependency models for the same piece of work. This creates an unresolvable ordering ambiguity that will surface as a coordination failure when Wave 1 exits and the orchestrator must decide whether to dispatch Wave 2.

---

## Convergence Assessment

**Status: NOT_CLEAN — RESIDUALS + NEW DEFECTS**

This review found 2 pass-1 residuals and 14 net-new defects.

The residuals (F-CRIT-NEW-001-PASS2-RESIDUAL and F-MED-NEW-001-PASS2-RESIDUAL) share a structural root cause: the fix-burst-1 architect adopted adversary proposed-fix language verbatim without verifying the underlying factual claims against source-of-truth. This is a methodology gap, not an isolated error. It is codified as TD-FIX-BURST-VERIFY-001 (see F-OBS-NEW-002).

The 14 net-new defects include 2 CRIT findings that were not present in pass-1's scope (the instance-pool claim and the ADR-022 scheduling contradiction are new content in v1.1 that was not present in v1.0). This is a regression: the fix-burst introduced new defects while closing old ones. The sandbox-URL-allowlist CRIT (F-CRIT-NEW-002) is particularly notable — it is worse than the original finding it replaced, because it actively asserts a security control is implemented when it is not.

**Convergence streak: 0/3 (RESET)**

Dispatch fix-burst-2 targeting all 16 findings (2C/4H/5M/3L/2OBS) before pass-3.

**Required fix-burst-2 scope:**
1. Read spec_parser.rs, mod.rs, loader.rs, types.rs directly before authoring any fix language
2. Fix or accurately characterize: spec_parser.rs CustomAdapter claim (2 sites), sandbox URL allowlist, instance pool claim, grammar partition completeness, ADR-022 scheduling conflict, Wave 0/F amends_bcs completeness, PREREQ-D/E boot.rs ownership, VP-PLUGIN-004 fixture, 401-injection scoping, spec_parser line range, strum-derives duplicate claim, VP-PLUGIN-001 sync requirement, inputs frontmatter
3. PR checklist must include: "I read each source file cited in fixed claims before authoring the fix language"

---

## Novelty Assessment

| **Pass** | Pass 2 |
|----------|--------|
| **Total findings** | 16 |
| **Residuals (from pass-1)** | 2 (12.5%) |
| **Net-new defects** | 14 (87.5%) |
| **CRIT novel findings** | 1 (F-CRIT-NEW-002 — sandbox contradiction introduced by v1.1) |
| **HIGH novel findings** | 4 |
| **MED novel findings** | 3 |
| **Process-gap novel findings** | 2 |
| **Novelty score** | 0.875 (14 of 16 findings are new) |
| **Trajectory** | 26→16 |
| **Verdict** | FINDINGS_REMAIN |

High novelty score (0.875) at pass-2 indicates the v1.1 amendment addressed many pass-1 issues but introduced substantial new content that was not adversarially reviewed before commit. The ADR grew significantly in v1.1; new sections (§D WASM ABI, §G DTU Clones, VP-PLUGIN-001 through VP-PLUGIN-005, Audit Coverage Matrix) contain most of the novel defects. Regression in sandbox claim is the most operationally significant novelty.

---

## Operational Notes

**Read-only profile:** This adversary agent was dispatched with read-only tooling. The pass-2 report was delivered inline in the orchestrator transcript. The state-manager backfill burst (Standing Rule 1) is responsible for persisting this report to `.factory/cycles/wave-4-operations/adversarial-reviews/ADR-023-pass-2.md`. Per TD-VSDD-ADVERSARY-PERSISTENCE, adversary agents cannot persist their own reports; this durability mechanism is the current workaround.

**File reads performed:** spec_parser.rs, lib.rs, plugin/mod.rs, plugin/host_functions.rs, plugin/loader.rs, types.rs, ADR-023-pass-1.md, td-from-adr-023-pass-1.md, six sensor auth BCs, four field-mapping BCs, invariants.md, policies.yaml.

**Information asymmetry applied:** The adversary reviewed v1.1 without access to the pass-1 proposed-fix text or the fix-burst-1 author's intent. This ensures that proposed-fix language adopted verbatim into v1.1 is evaluated for factual accuracy against source-of-truth, not against the intent of the original proposed fix.
