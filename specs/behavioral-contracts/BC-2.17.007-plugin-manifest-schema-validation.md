---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-13T00:00:00Z
phase: 3.A
origin: greenfield
subsystem: "SS-17"
capability: "CAP-032"
lifecycle_status: draft
introduced: 2026-05-13
modified: 2026-05-13
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
input-hash: "76729b7"
traces_to: ["CAP-032"]
extracted_from: ".factory/specs/prd.md"
closes_finding: "F-LP1-HIGH-004"
---

# BC-2.17.007: Plugin Manifest Schema Validation Before WIT Validation

## Description

Before a `.prx` plugin file is passed to the WASM runtime for interface validation or
instantiation, the plugin's embedded manifest is parsed and validated for required field
presence and value constraints. The manifest must contain the fields `name`, `version`,
`format_version`, and `allowed_urls`. The `format_version` integer must not exceed
`CURRENT_SUPPORTED_VERSION` (a crate constant declared in `prism-spec-engine`). The
`allowed_urls` field must be present as an explicit list (even an empty list is accepted;
the field being absent or `None` is rejected). Manifest validation is the first gate in
the plugin load pipeline — a manifest failure prevents WIT compilation and registration
from being attempted.

## Preconditions

- A `.prx` file is being loaded by `PluginRuntime::load_plugin(path)` (at startup
  discovery or during hot reload)
- The raw bytes of the `.prx` file have been read from disk
- No WASM Component compilation has been attempted yet for this file

## Postconditions

### Valid Manifest (all fields present and in range)

1. **`name` field present and non-empty:** The manifest `name` field is a non-empty
   UTF-8 string. A missing `name` field produces `E-PLUGIN-015`. An empty string produces
   `E-PLUGIN-015`.
2. **`version` field present and semver-parseable:** The manifest `version` field is a
   non-empty string that successfully parses as a semantic version (major.minor.patch).
   A missing `version` field produces `E-PLUGIN-016`. A malformed version string (not
   parseable as semver) produces `E-PLUGIN-016`.
3. **`format_version` field present and within range:** The manifest `format_version`
   integer field is present and satisfies `format_version <= CURRENT_SUPPORTED_VERSION`.
   A missing `format_version` field produces `E-PLUGIN-014`. A value exceeding
   `CURRENT_SUPPORTED_VERSION` produces `E-PLUGIN-014`.
4. **`allowed_urls` field present as explicit list:** The manifest `allowed_urls` field
   is present and is an explicit array (empty array `[]` is accepted; the field entirely
   absent, `null`, or `None` is rejected with `E-PLUGIN-013`). The implementation must
   NOT silently default `allowed_urls` to any value when the field is absent.
5. **On all four validations passing:** Control passes to the WIT interface validation
   stage (BC-2.17.006). The plugin is NOT registered yet — manifest validation passing
   is necessary but not sufficient for registration.

### Invalid Manifest (any field fails validation)

6. The plugin is NOT added to the `PluginRuntime.registry`.
7. An `ERROR`-level log is emitted identifying the manifest path and the specific
   validation failure.
8. The appropriate structured error is returned (see Error Conditions table).
9. Other plugins in the directory continue loading; this plugin's failure is isolated
   and does not block sibling plugins.

## Invariants

- **Manifest-before-WIT ordering:** Manifest schema validation MUST complete successfully
  before any WASM Component compilation or WIT interface checking is attempted.
  This ordering is unconditional — it applies to startup discovery and hot reload alike.
- **No partial registration on manifest failure:** A plugin that fails manifest validation
  is not registered in any partial state. The registry is not modified.
- **`allowed_urls` None is never a valid loaded state:** After PREREQ-D lands, no
  `PluginRuntime` entry may carry `HostState { allowed_urls: None }`. The manifest
  validation gate enforces this: a missing `allowed_urls` field is a rejection, not a
  default. This closes the silent-allow-all vulnerability identified in ADR-023 §C4
  current-state description.
- **`CURRENT_SUPPORTED_VERSION` is a compile-time crate constant** in `prism-spec-engine`.
  Its value is not configurable at runtime. Plugins with `format_version` exceeding this
  constant are rejected regardless of operator configuration.

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-PLUGIN-013` | `allowed_urls` field absent or `None` in manifest | Plugin not registered; error logged citing missing field; other plugins unaffected |
| `E-PLUGIN-014` | `format_version` field absent OR `format_version > CURRENT_SUPPORTED_VERSION` | Plugin not registered; error logged citing expected vs actual; other plugins unaffected |
| `E-PLUGIN-015` | `name` field absent or empty string | Plugin not registered; error logged; other plugins unaffected |
| `E-PLUGIN-016` | `version` field absent or not semver-parseable | Plugin not registered; error logged citing malformed value; other plugins unaffected |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-17-029 | Manifest has `allowed_urls` key with empty list `[]` | Accepted — empty list means no HTTP requests permitted, but field is present; plugin proceeds to WIT validation |
| EC-17-030 | Manifest has `format_version = CURRENT_SUPPORTED_VERSION` exactly | Accepted — boundary-equal value is within range |
| EC-17-031 | Manifest has `format_version = CURRENT_SUPPORTED_VERSION + 1` | Rejected with `E-PLUGIN-014`; error message cites `format_version: {actual} exceeds supported: {CURRENT_SUPPORTED_VERSION}` |
| EC-17-032 | Multiple manifest fields invalid simultaneously (e.g., `name` empty AND `allowed_urls` absent) | First validation failure encountered in field-check order (name → version → format_version → allowed_urls) returns immediately; one error per plugin load attempt |
| EC-17-033 | Directory scan with 10 plugins; 3 fail manifest validation, 7 pass | 7 plugins proceed to WIT validation; 3 log `E-PLUGIN-01x` errors; startup continues normally |
| EC-17-034 | Hot reload of plugin whose manifest now has `format_version` exceeding `CURRENT_SUPPORTED_VERSION` | Rejected at manifest gate; previously-loaded version of plugin is retained (CI-002 invariant); `E-PLUGIN-014` logged |
| EC-17-035 | Manifest TOML parse failure (structurally malformed TOML) | Rejected before field validation; TOML deserialization error logged; plugin not registered; `E-PLUGIN-014` used as umbrella code for manifest parse failures |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-17-007-happy | Valid manifest with name="my-plugin", version="0.1.0", format_version=1, allowed_urls=[] | Manifest accepted; proceeds to WIT validation stage | Baseline happy path |
| TV-17-007-no-allowed-urls | Manifest missing `allowed_urls` field entirely | `E-PLUGIN-013`; plugin not registered | EC-17-029 inverse |
| TV-17-007-empty-allowed-urls | Manifest with `allowed_urls = []` | Manifest accepted | EC-17-029 |
| TV-17-007-format-exceeded | Manifest with `format_version = CURRENT_SUPPORTED_VERSION + 1` | `E-PLUGIN-014`; plugin not registered | EC-17-031 |
| TV-17-007-name-empty | Manifest with `name = ""` | `E-PLUGIN-015`; plugin not registered | Postcondition 1 |
| TV-17-007-version-malformed | Manifest with `version = "not-semver"` | `E-PLUGIN-016`; plugin not registered | Postcondition 2 |
| TV-17-007-bulk | 10 plugins loaded; 3 fail manifest validation | 7 proceed to WIT stage; 3 logged as E-PLUGIN-01x | EC-17-033 |
| TV-17-007-hot-reload-reject | Hot reload of plugin with format_version exceeded | E-PLUGIN-014; previous plugin version retained | EC-17-034 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-PLUGIN-007 | After PREREQ-D lands, every loaded `.prx` plugin in `PluginRuntime` registry carries an explicit `allowed_urls: Vec<String>` field — manifest omission is a hard load rejection (E-PLUGIN-013) per AC-7 default-deny | Integration test (property assertion on PluginRuntime state post-load) |

Note: VP-PLUGIN-007 numeric alias is VP-152 per VP-INDEX. This BC is the contract that
establishes the invariant VP-PLUGIN-007 verifies.

## Related BCs

- BC-2.17.006 — WIT Interface Validation Before Plugin Registration (manifest validation is the prerequisite gate; this BC runs first, BC-2.17.006 runs second)
- BC-2.17.001 — Plugin Panic Isolation (panic isolation applies only after a plugin successfully completes both manifest and WIT validation)
- BC-2.17.005 — Plugin Hot Reload Atomic Swap (hot reload retains prior version when manifest validation fails, per EC-17-034)

## Architecture Anchors

- ADR-023 §C4 — `.prx` plugin manifest format declares `name`, `version`, `format_version`, `allowed_urls`; loader validates `format_version` against `CURRENT_SUPPORTED_VERSION`; `allowed_urls` absence is a rejection not a default
- `crates/prism-spec-engine/src/plugin/mod.rs` — `PluginRuntime::load_plugin` is the implementation site; manifest parsing occurs before `Component::from_binary`
- VP-PLUGIN-007 (VP-152) — post-load assertion that `allowed_urls` is never `None` in loaded registry entries

## Story Anchor

S-PLUGIN-PREREQ-D — prism-bin/prism-spec-engine: Wire PluginRuntime into Boot Sequence; .prx Load Pipeline (AC-5 anchors to this BC)

## VP Anchors

VP-PLUGIN-007 (VP-152): `PluginRuntime` allowlist explicit `Vec<String>` post-boot assertion — verifies the postcondition that every loaded plugin carries an explicit `allowed_urls` list (manifest omission rejected at load gate per AC-7 default-deny).

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-032 |
| Capability Anchor Justification | CAP-032 ("WASM Plugin Runtime") per capabilities.md §CAP-032 — this BC describes manifest-level schema validation that is part of the `.prx` load pipeline, which is central to CAP-032: "every `.prx` file is validated against the declared WIT interface before registration" and "Plugin ABI is defined via WIT interface definitions" — manifest validation is the prerequisite gate for that WIT ABI check |
| L2 Invariants | INV-PLUGIN-006 (manifest validation is a prerequisite of the WIT validation enforced by INV-PLUGIN-006) |
| ADR | ADR-023 §C4 |
| Story | S-PLUGIN-PREREQ-D (AC-5) |
| Priority | P0 |
| Closes Finding | F-LP1-HIGH-004 (S-PLUGIN-PREREQ-D pass-1 adversarial review) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | fix-burst-33 | 2026-05-14 | product-owner | F-LP35-MED-001 closure: VP-PLUGIN-007 description sweep — line 138 + line 161 rewritten from pre-AC-7 "allowed_urls = None" / "allowlist not-None" Option-semantics to post-AC-7 "explicit allowed_urls: Vec<String>" / "explicit list under AC-7 default-deny" framing. Sibling-document propagation gap from F-LP34-LOW-001 closure (D-533) — fix-burst-32 propagated to VP-INDEX + story §References but missed these 2 in-perimeter BC sites. Cross-document propagation: VP-INDEX v1.35 + story §References:1034 corrected in D-533 fix-burst-32; this BC update closes the in-perimeter remainder. OBS-LP35-001 (verification-architecture.md:282 + ADR-023:732-733 architecture-layer siblings) deferred phase-5 architect adjudication. |
| 1.2 | fix-burst-7-stage-1A | 2026-05-13 | product-owner | F-LP8-HIGH-001 closure (Path B): `lifecycle_status: active` → `lifecycle_status: draft`. S-PLUGIN-PREREQ-D is pre-merge — this BC was introduced during fix-burst and has never been part of a merged story PR. `lifecycle_status: active` was set at initial authorship before POL-14 canonicalization applied to new BCs authored in-burst. Per POL-14 (`bc_vp_promotion_on_anchor_merge`), auto-promotion to `active` will occur at S-PLUGIN-PREREQ-D PR merge. |
| 1.1 | state(D-464) | 2026-05-13 | state-manager | F-LP2-OBS-007 closure — `introduced:` field updated from opaque burst-ID notation to canonical date-keyed format per POL-20 (bc_introduced_field_canonical_format). No spec content change. |
| 1.0 | wave-4-fix-burst-F-LP1-HIGH-004 | 2026-05-13 | product-owner | Initial contract — closes F-LP1-HIGH-004; establishes manifest schema validation as prerequisite gate before WIT validation (BC-2.17.006); authors E-PLUGIN-013/014/015/016 |
