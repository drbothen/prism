---
document_type: pr-level-pr-reviewer-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 6
reviewer: pr-reviewer
fresh_context: true
model_family: opus-4.7-1m
feature_head: 9e987c3f
develop_baseline: f19575ff
pass_5_head: 7406458a
timestamp: 2026-05-26T00:00:00Z
---

# PR #155 -- pr-reviewer pass-6

Fresh-context pass-6 review of PR #155 (S-CONFIG-MULTI-TENANT-OVERRIDE-001 -- per-org
sensor endpoint overlay loading per ADR-029). Diff baseline `develop@f19575ff`; feature
HEAD `9e987c3f`. This pass follows the pass-5 fix-burst (commit `9e987c3f`) which
closed SEC-PASS5-001/002 by adding `sanitize_for_log` calls to two remaining sibling-sweep
gaps in overlay.rs. CI GREEN (36/36). Pass-5 was CLEAN(strict). Target: verify fix-burst
did not regress, achieve 2nd consecutive CLEAN(strict).

51 files changed; +4957 / -40 (net +4917). Production code delta ~1790 LOC
(overlay.rs ~1025 + boot ~477 + fanout ~477 + adapters ~63 + error variants ~35).
Test code: ~2015 LOC in overlay_loading_tests.rs + ~257 LOC in boot step4 overlay tests
+ ~290 LOC in fanout unit tests + ~98 LOC in sanitize_for_log unit tests. 22 commits
on the feature branch.

---

## 8-Item Review Checklist

| # | Item | Assessment |
|---|------|-----------|
| 1 | Diff Coherence | PASS -- all 51 changed files relate to S-CONFIG-MULTI-TENANT-OVERRIDE-001 overlay loading (ADR-029). No unrelated changes. The HTTP timeout additions across 4 adapters close TD-S-PLUGIN-PREREQ-B-005 for the overlay dispatch path. The `#[non_exhaustive]` gate expansion from 32 to 35 types covers the 3 new public overlay types. CLAUDE.md `EXPECTED=35` update matches CI yml. |
| 2 | Description Accuracy | PASS -- PR body matches actual changes. Architecture mermaid diagrams, traceability matrix (5 BCs, 7 ACs), error taxonomy (E-SPEC-019..023), convergence narrative (13 LOCAL passes), and deferred items with concrete story anchors all correspond to the implementation. |
| 3 | Test Coverage | PASS -- comprehensive across 4 test sites: (a) 20+ tests in overlay_loading_tests.rs covering all 7 ACs and 5 error codes with byte-equality template matching; (b) 4 boot step4 overlay tests (backwards compat, unknown slug, happy path, corrupt TYPE spec); (c) 3 fanout unit tests (Case A overlay injection, Case B fallback, unknown org); (d) 1 end-to-end CapturingAdapter wiring test proving overlay base_url reaches the adapter; (e) 6 sanitize_for_log unit tests; (f) negative paths: oversized file, SSRF scheme rejection, unreadable file (POSIX), mixed-case org slug. |
| 4 | Demo Evidence | PASS -- 7 ACs x (gif + webm + tape) = 21 recording artifacts + evidence-report.md (193 lines). All ACs have both success and error path coverage. evidence-report.md includes BC traces, recording metadata, and full artifact index. |
| 5 | Commit Quality | PASS -- conventional commits with story ID. 22 commits reflecting TDD progression: stubs, red gate, implementation, 13 LOCAL adversarial fix-bursts, 3 PR-level fix-bursts. Each commit message is clear and descriptive. |
| 6 | Diff Size | PASS -- 4957 lines total, but ~2660 are test code, ~1500 are demo evidence (binary files in diffstat), ~59 shell script, ~193 evidence report. Production code delta ~1790 LOC is reasonable for a new subsystem implementing 5 BCs with full error taxonomy, security hardening, and Arc-DI wiring. |
| 7 | Missing Changes | PASS -- no missing changes detected against the 7 ACs and 5 BCs (BC-2.06.012 through BC-2.06.016). All acceptance criteria are implemented and tested. |
| 8 | Dependency Status | PASS -- S-WAVE5-PREP-01 merged as PR #138. No other upstream dependencies. |

---

## Pass-5 Fix-Burst Verification (9e987c3f)

The pass-5 fix-burst commit `9e987c3f` made exactly two changes to `overlay.rs`:

### SEC-PASS5-001: `expected_instance_id` sanitization in E-SPEC-020 path

**Before:** `let expected_instance_id = format!("{}@{}", expected_sensor_id, expected_org_slug);`

**After:** Both `expected_sensor_id` and `expected_org_slug` are passed through `sanitize_for_log()` before concatenation. This closes the gap where a filesystem-derived sensor stem (no regex validation) and an org slug from the EC-016-002 unregistered path could inject control characters into the E-SPEC-020 error message.

**Regression check:** The change affects the `expected_instance_id` value used in the `overlay.instance_id != expected_instance_id` comparison. Since `sanitize_for_log` only replaces control characters (which are not valid in TOML string values or filesystem names in normal use), the comparison semantics are preserved for all legitimate inputs. The byte-equality test `test_BC_2_06_016_error_messages_match_canonical_templates` would catch any unintended drift. CI GREEN confirms no regression.

### SEC-PASS5-002: `overlay_file_path` sanitization at derivation point

**Before:** `let overlay_file_path = format!("customers/{slug_str}/{file_name}");`

**After:** The entire path is wrapped in `sanitize_for_log()` at its derivation point, covering all 9 downstream error constructors that embed `overlay_file_path`. This is a defense-in-depth measure since `file_name` comes from `readdir` and on Linux/macOS can contain arbitrary bytes including control characters (CWE-117).

**Regression check:** `overlay_file_path` is used only in error messages and `file_path` metadata fields -- never as an actual filesystem path for I/O operations (the real I/O uses `file_entry.path()`). Sanitizing the display path has zero impact on functionality. CI GREEN confirms no regression.

### Documentation update

The `sanitize_for_log` doc comment was updated to enumerate all 9 call sites, adding the two new sites (SEC-PASS5-001, SEC-PASS5-002). Accurate and complete.

**Verdict: Fix-burst is correct, minimal, and introduces no regression.**

---

## Prior-Pass Finding Disposition (Carried from Pass-5)

### F-PR155-P2-001 (LOW) -- CrowdStrike/Cyberint base_url overlay no-op

**Status: STILL PRESENT (accepted -- architectural)**

CrowdStrike and Cyberint adapters do not read `sensor_config["base_url"]` for per-org
endpoint routing. Only Armis and Claroty implement the `effective_base_url` resolution
pattern. This is architecturally correct: CrowdStrike uses Falcon cloud URLs from auth,
Cyberint uses auth-tied endpoints. A user creating overlay files for these sensors gets
no error and no behavioral change. Acceptable as a design-level gap for a future story
if those sensors need per-org endpoint overrides.

### F-PR155-P2-004 (LOW) -- SSRF: http:// scheme allows IMDS endpoints

**Status: STILL PRESENT (accepted -- architectural)**

SEC-REDUX-006 allows `http://` including cloud metadata endpoints. Overlay files are
operator-controlled config, not user-supplied runtime input. Accepted threat model.

### F-PR155-P2-005 (NIT) -- Validation ordering hides SSRF URL behind structural errors

**Status: STILL PRESENT (accepted -- architectural)**

validate_overlay_toml early-returns on structural errors before URL scheme check.
Acceptable UX: operators fix one class of error at a time.

### F-PR155-P3-001 (NIT) -- OverlayLoadResult missing #[non_exhaustive]

**Status: STILL PRESENT (accepted -- architectural)**

`OverlayLoadResult` is a return-only container. Not TOML-deserialized, not part of the
pub-API construction surface. Borderline interpretation of the convention.

---

## Pass-6 Fresh-Context Verification

### Full file-by-file review performed

I independently reviewed all 51 changed files (excluding binary demo recordings) against
the 8-item checklist and CLAUDE.md conventions. Below is a summary of what I verified
for each category of change and why I am satisfied.

#### Production code -- overlay.rs (1025 LOC)

- All 3 public types carry `#[non_exhaustive]` -- verified.
- `ResolvedSpecKey` uses `(OrgSlug, SensorId)` newtypes -- no raw strings.
- `OverlayLoader::load_overlays` handles all edge cases: absent dir, .gitkeep, symlink
  rejection (lstat), file size limit (64 KiB), OrgRegistry cross-validation, unregistered
  directory scanning (EC-016-002), multi-error aggregation, and defensive unreachable branch.
- `validate_overlay_toml` checks in correct order with multi-error collection.
- `merge_overlay_onto_type_spec` correctly clones TYPE spec, merges only permitted scalars,
  tracks provenance, and preserves immutable fields.
- `sanitize_for_log` applied at ALL user-controlled embedding sites (9 total):
  1. `overlay_file_path` at derivation point (SEC-PASS5-002)
  2. `expected_sensor_id` in E-SPEC-021 tables check (SEC-PASS4-002)
  3. `expected_org_slug` in E-SPEC-021 tables check (SEC-PASS4-002)
  4. `expected_sensor_id` in E-SPEC-020 instance_id check (SEC-PASS5-001)
  5. `expected_org_slug` in E-SPEC-020 instance_id check (SEC-PASS5-001)
  6. `overlay_base_url` in SSRF rejection branch
  7. `slug` in `make_e_spec_022_unknown_org_slug`
  8. `field_name` in `make_e_spec_023_unrecognized_field`
  9. `extends_value` in `make_e_spec_019_unknown_extends` (all 3 occurrences)
  Additionally: `actual_instance_id` in `make_e_spec_020_instance_id_mismatch`.
  TD-VSDD-060 sibling-sweep is complete.
- No `unwrap()` or `expect()` in production code paths -- verified.
- No `println!` -- verified.

#### Production code -- boot.rs (+477 LOC)

- `step4_load_sensor_specs_with_overlays` correctly replaces bare step4 in boot sequence.
- `build_type_spec_map_for_overlay` aggregates parse failures and aborts hard (PRR-005).
- OrgRegistry threaded from step 3 to step 4 (INV-COMPAT-002).
- `resolved_spec_map` threaded through `BootContext` and `RunningServer`.
- 3 new `event_type` emissions with inline catalog references.
- 4 unit tests exercise production code path in-process (SID-1 compliant).

#### Production code -- fanout.rs (+477 LOC)

- `resolve_spec_for_fanout`: O(1) map lookup, no I/O, no blocking.
- `fan_out_with_overlay_map`: resolves then delegates to `fan_out()`.
- CapturingAdapter E2E test is load-bearing (would fail against pre-fix code).

#### Production code -- engine.rs, materialization.rs

- `resolved_spec_map` threaded through `QueryEngine` and `MaterializationContext`.
- `new_full()` accepts the new parameter; `new()` initializes to `None`.
- Conditional dispatch: `fan_out_with_overlay_map` when both org_registry and
  resolved_spec_map are `Some`; bare `fan_out()` otherwise.

#### Production code -- adapter files (armis.rs, claroty.rs, crowdstrike.rs, cyberint.rs)

- `.timeout(Duration::from_secs(30))` added to all 5 `reqwest::Client::builder()` sites.
- `unwrap_or_default()` replaced with `unwrap_or_else(|e| panic!(...))` -- provides
  diagnostic information on the (extremely unlikely) client build failure.
- Armis and Claroty: `effective_base_url` resolution from `sensor_config["base_url"]`
  with fallback to `self.instance_url`. Wired through `get_search` and `post_read`.

#### Error taxonomy -- error.rs

- 5 new `SpecErrorCode` variants (ESpec019..ESpec023) with comprehensive doc comments
  citing canonical message templates, BCs, and ADR-029.

#### CI and gates

- `ci.yml` EXPECTED bumped 32 to 35 with explanation comment.
- `check-non-exhaustive.sh` EXPECTED bumped.
- `struct_violations.rs`: 3 new violations (v33, v34, v35) for the 3 overlay types.
- `main.rs` doc comment updated with type list.
- `check-error-taxonomy-snapshot.sh` and Justfile recipe added.

#### Test files

- `overlay_loading_tests.rs` (2015 LOC): comprehensive coverage of all 7 ACs, 5 error
  codes, byte-equality matching, negative paths, and edge cases.
- `execute_integration_tests.rs`: all 12 `new_full()` / `new_with_resolver()` call sites
  updated with the new `resolved_spec_map` parameter.

#### Config and fixture files

- 2 overlay fixture files (acme, contoso) in `specs/customers/`.
- `customers/.gitkeep` sentinel.
- `error-taxonomy-snapshot.md` fixture for CI-portable byte-equality tests.
- `prism-sensors/Cargo.toml`: `tempfile` dev-dependency added.
- `prism-spec-engine/Cargo.toml`: overlay test target added.
- `Cargo.lock` updated.

---

## New Findings (Pass-6)

No new findings. Zero findings of any severity.

The pass-5 fix-burst (SEC-PASS5-001/002) correctly closes the two remaining
`sanitize_for_log` sibling-sweep gaps identified in pass-5. The fix is minimal (21 lines
added, 2 lines replaced), well-documented with CWE references and TD-VSDD-060 compliance
notes, and introduces no behavioral regression (CI GREEN 36/36).

All previously accepted LOW/NIT findings remain unchanged and carry no new information
that would change their disposition. They are architectural design gaps with explicit
future-story anchors, not code defects.

---

## SAP-1 Probe: Tracing Emission Catalog Completeness

New `event_type` emissions added in this branch:
- `overlay.loaded` (info) -- overlay.rs line 501
- `overlay.timeout_secs_ignored` (warn) -- overlay.rs line 751
- `boot.overlays_loaded` (info) -- boot.rs line 710
- `boot.type_spec_read_failed` (error) -- boot.rs line 771
- `boot.type_spec_parse_failed` (error) -- boot.rs line 791

All 5 have inline comments documenting audit role and recurrence policy. Pass-5 confirmed
SAP-1 catalog compliance (BC-2.16.002 rows present in factory-artifacts). No new emissions
added in the pass-5 fix-burst (9e987c3f only modifies sanitization, not tracing).

---

## SAP-2 Probe: DTU-TOML Schema Parity

Not applicable -- this story does not modify `[[tables]]` blocks or sensor TOML schema
definitions. Overlay files are explicitly forbidden from containing `[[tables]]`
(E-SPEC-021). TYPE spec schemas are inherited unchanged (INV-OVL-001).

---

## Convergence Assessment

| Criterion | Status |
|-----------|--------|
| Pass-5 (pr-reviewer) | CLEAN(strict) |
| Pass-6 (pr-reviewer, this pass) | CLEAN(strict) |
| Consecutive CLEAN(strict) streak | 2/3 |

---

## Verdict

**CLEAN(strict): yes**
**CLEAN(PR-merge): yes**

Zero findings of any severity (CRIT, HIGH, MED, LOW, OBS, PROCESS-GAP). The pass-5
fix-burst at `9e987c3f` correctly closed SEC-PASS5-001/002 without regression. All
51 changed files reviewed against the 8-item checklist and CLAUDE.md conventions. This
is the 2nd consecutive CLEAN(strict) pass.
