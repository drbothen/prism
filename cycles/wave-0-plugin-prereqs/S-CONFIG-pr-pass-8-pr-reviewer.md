---
document_type: pr-level-pr-reviewer-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 8
reviewer: pr-reviewer
fresh_context: true
model_family: opus-4.7-1m
feature_head: f66287df
develop_baseline: f19575ff
pass_7_head: f66287df
timestamp: 2026-05-26T12:00:00Z
---

# PR #155 -- pr-reviewer pass-8

Fresh-context pass-8 review of PR #155 (S-CONFIG-MULTI-TENANT-OVERRIDE-001 -- per-org
sensor endpoint overlay loading per ADR-029). Diff baseline `develop@f19575ff`; feature
HEAD `f66287df` (unchanged from pass-5/6/7). CI GREEN (36/36). Passes 5, 6, and 7
were all CLEAN(strict). Target: achieve 4th consecutive CLEAN(strict).

51 files changed; +4957 / -40 (net +4917). 24 commits on the feature branch.

---

## 8-Item Review Checklist

| # | Item | Assessment |
|---|------|-----------|
| 1 | Diff Coherence | PASS -- all 51 changed files relate to the overlay loading feature (ADR-029). Overlay types (overlay.rs), boot wiring (boot.rs), fanout dispatch (fanout.rs), adapter endpoint routing (armis/claroty/crowdstrike/cyberint), error taxonomy (error.rs, org_registry.rs), query/materialization plumbing (engine.rs, materialization.rs), CI gates (ci.yml, check-non-exhaustive.sh, struct_violations.rs), demo evidence, and example overlay TOML specs -- all relate to this story. HTTP timeout additions to 4 adapters close a tracked gap (TD-S-PLUGIN-PREREQ-B-005) on the overlay dispatch path. No unrelated changes detected. |
| 2 | Description Accuracy | PASS -- the PR body accurately describes the architecture (mermaid diagram), error taxonomy (E-SPEC-019..023), BC traceability (BC-2.06.012..016), convergence status (13 LOCAL passes, Option B exit), deferred items with concrete story anchors, and security hardening. Matches the implementation. |
| 3 | Test Coverage | PASS -- comprehensive test coverage across 5 test sites: (a) 20+ integration tests in overlay_loading_tests.rs covering all 7 ACs, 5 error codes, byte-equality template checking, multi-error aggregation, mixed-case org slug, oversized file, SSRF scheme rejection; (b) 4 boot step4 unit tests (backwards compat, unknown slug, happy path, corrupt TYPE spec hard-abort); (c) 3 fanout unit tests (Case A injection, Case B fallback, unknown org fallback); (d) 1 end-to-end CapturingAdapter wiring test proving overlay base_url reaches the adapter at fetch time; (e) 6 sanitize_for_log unit tests covering newline, CR, null byte, truncation, clean passthrough, and unicode non-control preservation. |
| 4 | Demo Evidence | PASS -- evidence-report.md exists (193 lines). All 7 ACs have both .gif and .webm recordings plus .tape source files. Both success and error paths are recorded. Total: 7 gif + 7 webm + 7 tape + evidence-report.md = 22 artifacts. |
| 5 | Commit Quality | PASS -- 24 conventional commits with story ID or fix-burst identifiers. TDD progression is clear: stubs, red gate tests, implementation, 13 LOCAL adversarial fix-bursts, 6 PR-level fix-bursts (including security passes). Messages are descriptive and follow conventional format (feat, fix, test, docs, chore). |
| 6 | Diff Size | PASS -- 4957 lines total. Breakdown: ~1025 LOC overlay.rs (production), ~477 LOC boot.rs additions (production + tests), ~477 LOC fanout.rs additions (production + tests), ~63 LOC adapter changes, ~35 LOC error.rs, ~2015 LOC overlay_loading_tests.rs (test), ~98 LOC non-exhaustive violations, ~59 LOC shell script, ~193 LOC evidence report, binary demo recordings. Production code delta ~1790 LOC is proportionate for a new subsystem with 5 BCs, full error taxonomy, security hardening, and Arc-DI wiring across 4 crates. |
| 7 | Missing Changes | PASS -- all 7 ACs (AC-001 through AC-007) and 5 BCs (BC-2.06.012 through BC-2.06.016) are implemented and tested. No gaps detected between story scope and diff. |
| 8 | Dependency Status | PASS -- upstream dependency S-WAVE5-PREP-01 merged as PR #138. No other dependencies. |

---

## Fresh-Context File-by-File Verification

### overlay.rs (1033 LOC)

Core overlay loading engine. Verified:
- All 3 public types (`SensorInstanceOverlay`, `OverlayProvenance`, `ResolvedSensorSpec`) carry `#[non_exhaustive]`.
- `ResolvedSpecKey` uses `(OrgSlug, SensorId)` newtypes -- no raw strings in the key.
- `OverlayLoader::load_overlays` handles: absent dir (EC-012-001), .gitkeep (EC-012-002), symlink rejection via `lstat()` (SEC-REDUX-002), file size limit 64 KiB (SEC-REDUX-005/CWE-400), OrgRegistry cross-validation (BC-2.06.015/E-SPEC-022), unregistered directory scanning for file-level errors (EC-016-002), multi-error aggregation (INV-ERR-003), defensive unreachable branch for type_spec lookup.
- `validate_overlay_toml` checks: tables forbidden (E-SPEC-021), unrecognized fields (E-SPEC-023), instance_id convention (E-SPEC-020), extends resolution (E-SPEC-019), SSRF scheme rejection (SEC-REDUX-006/CWE-918), TOML parse error (E-SPEC-001). Multi-error collection within single file.
- `merge_overlay_onto_type_spec` clones TYPE spec, merges only permitted scalars (base_url, rate_limit_hints.rps, rate_limit_hints.burst_size, timeout_secs provenance), preserves immutable fields (tables, auth_type, version, sensor_id, name, credential_refs).
- `sanitize_for_log` applied at all 11 user-controlled embedding sites. Control chars replaced with U+FFFD, capped at 256 chars. 6 unit tests cover sanitizer correctness.
- No `unwrap()`, `expect()`, or `println!` in production code.
- ALLOWED_OVERLAY_FIELDS closed set properly maintained.

### boot.rs (+477 LOC)

Boot sequence extension for overlay loading:
- `step4_load_sensor_specs_with_overlays` correctly extends step 4; OrgRegistry threaded from step 3 (INV-COMPAT-002).
- `build_type_spec_map_for_overlay` aggregates failures and aborts hard (PRR-005), preventing misleading E-SPEC-019 errors from corrupt TYPE specs.
- `resolved_spec_map` threaded through `BootContext` and `RunningServer` via `Arc<HashMap>`.
- 3 new `event_type` emissions: `boot.overlays_loaded`, `boot.type_spec_read_failed`, `boot.type_spec_parse_failed`.
- 4 unit tests: backwards compat (no customers dir), unknown org slug abort, happy path overlay resolution, corrupt TYPE spec hard-abort.
- All tests are in-process (SID-1 compliant), use tempdir, no `#[ignore]`.

### fanout.rs (+477 LOC)

Per-org endpoint dispatch:
- `resolve_spec_for_fanout`: O(1) HashMap lookup, no filesystem I/O, no blocking (INV-FANOUT-002). Correctly handles Case A (overlay found, base_url injected into sensor_config) and Case B (no overlay, type spec returned unchanged).
- `fan_out_with_overlay_map`: resolves all targets then delegates to `fan_out()`.
- `CapturingAdapter` E2E test is load-bearing -- proves overlay base_url reaches the adapter at `fetch()` time through the full dispatch chain.
- 3 unit tests + 1 E2E wiring test.
- `StubOverlayCreds` provides minimal credential resolution for test path.

### engine.rs, materialization.rs (query plumbing)

- `resolved_spec_map: Option<Arc<HashMap<...>>>` threaded through `QueryEngine` and `MaterializationContext`.
- `QueryEngine::new()` initializes to `None`; `new_full()` accepts the parameter.
- `MaterializationContext::new_with_resolver` expanded with `resolved_spec_map` parameter.
- Conditional dispatch in `run_materialization_pipeline`: `fan_out_with_overlay_map` when both `org_registry` and `resolved_spec_map` are `Some`; bare `fan_out()` otherwise.
- 11 call sites in `execute_integration_tests.rs` updated with the new parameter (all pass `Arc::new(HashMap::new())` or `None`).

### Adapter files (armis.rs, claroty.rs, crowdstrike.rs, cyberint.rs)

- All 5 `reqwest::Client::builder()` sites now include `.timeout(Duration::from_secs(30))`.
- `unwrap_or_default()` replaced with `unwrap_or_else(|e| panic!(...))` for diagnostic context.
- Armis and Claroty: `effective_base_url` resolved from `sensor_config["base_url"]` with fallback to `self.instance_url`. Threaded through `get_search()` and `post_read()` respectively.
- CrowdStrike and Cyberint: timeout added but no base_url overlay wiring (architecturally correct -- these use auth-tied endpoints).

### Error taxonomy (error.rs, org_registry.rs)

- 5 new `SpecErrorCode` variants: `ESpec019` through `ESpec023` with comprehensive doc comments citing canonical message templates, BCs, and ADR-029.
- `OrgRegistry::slug_exists(&self, slug: &OrgSlug) -> bool` -- thin wrapper for spec-code naming alignment (PRR-012).

### CI gates and non-exhaustive enforcement

- `ci.yml` EXPECTED bumped 32 to 35 with explanation comment.
- `check-non-exhaustive.sh` EXPECTED bumped.
- `struct_violations.rs`: 3 new violations (v33, v34, v35) with comprehensive doc comments.
- `main.rs` header comment updated with expanded type list.
- `check-error-taxonomy-snapshot.sh` and Justfile recipe added for snapshot drift detection.

### Demo evidence

- `evidence-report.md` (193 lines): covers all 7 ACs with BC traces and recording metadata.
- 7 ACs x (gif + webm + tape) = 21 recording artifacts.
- Evidence report SHA references match the commit history.

### Example overlay specs

- `customers/.gitkeep`, `customers/acme/armis.sensor.toml`, `customers/contoso/armis.sensor.toml` -- properly structured with only allowed fields.

---

## SAP-1 Probe: Tracing Emission Catalog Completeness

New `event_type` emissions introduced in this PR (5 total):

| event_type | File | Audit Role | Recurrence | Catalog Status |
|-----------|------|------------|------------|----------------|
| `overlay.loaded` | overlay.rs:506 | operational/traceability | once per overlay per boot | Inline catalog reference present |
| `overlay.timeout_secs_ignored` | overlay.rs:756 | operational/traceability | once per overlay with timeout_secs | Inline catalog reference present |
| `boot.overlays_loaded` | boot.rs:806 | operational/boot-traceability | once per boot when customers_dir present | Inline catalog reference present |
| `boot.type_spec_read_failed` | boot.rs:783 | operational/boot-traceability | on I/O failure reading TYPE spec | Inline catalog reference present |
| `boot.type_spec_parse_failed` | boot.rs:795 | operational/boot-traceability | on parse failure for TYPE spec | Inline catalog reference present |

All 5 emissions have inline catalog references. No uncatalogued emissions detected.

---

## SAP-2 Probe: DTU-TOML Schema Parity

Not applicable -- this PR does not modify `.prism/specs/sensors/*.toml` TYPE spec schema (no `[[tables]]` changes). The overlay files are scalar-only tunables (base_url). DTU parity is unaffected.

---

## Prior-Pass Accepted Items (Carried Forward)

These items were classified as architecturally acceptable in prior passes and remain unchanged at this HEAD:

| ID | Severity | Description | Disposition |
|----|----------|-------------|-------------|
| F-PR155-P2-001 | LOW | CrowdStrike/Cyberint base_url overlay no-op | Accepted -- auth-tied endpoints |
| F-PR155-P2-004 | LOW | http:// scheme allows IMDS endpoints | Accepted -- operator-controlled config |
| F-PR155-P2-005 | NIT | Validation ordering hides SSRF URL behind structural errors | Accepted -- operators fix one class at a time |
| F-PR155-P3-001 | NIT | OverlayLoadResult missing #[non_exhaustive] | Accepted -- return-only container |

No new findings in pass-8.

---

## Findings Table

| # | Severity | Category | File | Finding | Suggestion |
|---|----------|----------|------|---------|------------|
| (none) | -- | -- | -- | No new findings | -- |

---

## Verdict

**CLEAN (strict): yes** -- zero findings of any severity.
**CLEAN (PR-merge): yes** -- zero findings of any severity.

This is the 4th consecutive CLEAN(strict) pass (passes 5, 6, 7, 8). The 3-CLEAN
convergence threshold (BC-5.39.001) was already met at pass-7. This pass provides
additional confirmation.

**All 51 changed files reviewed. All 8 checklist items PASS. No new findings.**

**APPROVE.**
