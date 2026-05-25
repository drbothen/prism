---
document_type: pr-level-pr-reviewer-review
cycle: wave-0-plugin-prereqs
story: S-CONFIG-MULTI-TENANT-OVERRIDE-001
pr: 155
pass: 1-redux
reviewer: pr-reviewer
fresh_context: true
post_clear_recovery: true
feature_head: 515fdc2e
develop_baseline: f19575ff
timestamp: 2026-05-24T23:51:40Z
input-hash: fdc0be10591b2ca1fbdaf4c72f4ac4051b0d69fee9cff7bdb3b65e54e481bd60
---

# PR #155 — pr-reviewer pass-1-redux

Fresh-context final review of PR #155 (S-CONFIG-MULTI-TENANT-OVERRIDE-001 — per-org sensor
endpoint overlay loading per ADR-029). Diff baseline `develop@f19575ff`; feature HEAD
`515fdc2e`. 44 files changed; +4263 / -28; 22 demo artifacts; 11 new Red Gate tests;
overlay loading + boot wiring + fanout dispatch + error taxonomy + #[non_exhaustive]
gate updates.

The review re-derives all findings from the PR diff, the story spec, the BC bodies
referenced in the story frontmatter, and the canonical taxonomy. The pr-reviewer cannot
see `.factory/` artifacts beyond the story spec; analysis is grounded in the diff alone.

The dominant finding class is **paper-fix-consumer-doesnt-read** (Lesson 50, TD-VSDD-059).
Three independent overlay tunable scalars are accepted by the overlay grammar, merged into
`ResolvedSensorSpec`, AND have a passing demo/test — but none of the three actually reaches
a production consumer at runtime. The PR's claim of end-to-end ADR-029 delivery is
overstated; only `base_url` reaches `sensor_config["base_url"]` (which is itself unread by
any production adapter). This dwarfs the original F-PR155-HIGH-001 finding from pass-1.

---

## F-PR155-REDUX-PRR-001 — `timeout_secs` overlay field is a paper-fix; never reaches reqwest::Client::builder().timeout(...)

**Severity:** HIGH
**Category:** paper-fix / consumer-doesnt-read (Lesson 50, TD-VSDD-059)
**File:lines:** `crates/prism-spec-engine/src/overlay.rs:82-86, 624-627` + `crates/prism-sensors/src/fanout.rs:587-649` + `crates/prism-sensors/src/auth/armis.rs:376-388`

**Finding:**
`SensorInstanceOverlay.timeout_secs: Option<u64>` is declared with a doc-comment that
claims it is an "Optional HTTP timeout override for this org's instance (seconds). When
`None`, the TYPE spec or global default timeout is used." It is included in
`ALLOWED_OVERLAY_FIELDS`, accepted by the overlay TOML deserializer, and tracked in
`OverlayProvenance.timeout_secs_from_overlay`. The test
`test_BC_2_06_012_timeout_secs_overlay_merge_with_provenance` (overlay_loading_tests.rs:1560)
verifies that the provenance bool flips to `true`.

However:

1. **There is no `timeout_secs` field on `prism_spec_engine::spec_parser::SensorSpec`**
   (spec_parser.rs:384-407). The merge function `merge_overlay_onto_type_spec`
   (overlay.rs:589-639) only flips a provenance bool — it does NOT store the value
   anywhere because `merged_spec` has no field to hold it.
2. **`resolve_spec_for_fanout` only injects `base_url`** into
   `sensor_config` (fanout.rs:619-636). It never reads `resolved.spec.<anything timeout>`
   because there is no such field, and never injects `sensor_config["timeout_secs"]`.
3. **No `reqwest::Client::builder().timeout(...)` callsite in the production HTTP path
   reads the overlay timeout.** `ArmisAdapter::new` (auth/armis.rs:377-380) builds its
   `reqwest::Client` with `Client::builder().cookie_store(false).build()` — no `.timeout()`,
   no per-org override. Likewise `prism-spec-engine::plugin` uses a single global 30-second
   timeout from a separate constant.

The value is dropped on the floor. A documented "HTTP timeout override" that doesn't override
the HTTP timeout is the canonical Lesson 50 paper-fix shape: schema field accepted, internal
state mutated (provenance bool), test passes (asserts only the provenance bool), production
consumer never sees the value.

**Suggested fix (choose one):**

A. **Wire the field end-to-end:** add `pub timeout_secs: Option<u64>` to
   `prism_spec_engine::spec_parser::SensorSpec`; merge it through
   `merge_overlay_onto_type_spec`; inject it into `sensor_config["timeout_secs"]` in
   `resolve_spec_for_fanout`; thread it into each adapter's `reqwest::Client::builder()` at
   adapter construction. Add a test that confirms the production adapter sees the overlay
   timeout (analogous to the `CapturingAdapter` test for base_url, but inside the real adapter
   construction path).

B. **Remove the field from the grammar:** drop `timeout_secs` from
   `SensorInstanceOverlay`, `ALLOWED_OVERLAY_FIELDS`, `OverlayProvenance`, the BC body,
   the error-taxonomy snapshot, and the E-SPEC-023 message. If you keep the field in the
   spec but cannot ship the consumer in this story, that is a Canonical Principle Rule 1
   violation (MVP-driven deferral; spec promises a behavior that the code does not
   deliver).

Option A is the production-grade default per CLAUDE.md Six Rules; Option B is acceptable
only if the architect explicitly defers per Rule 3 to a named future story with concrete
dependency. There is no such deferral in the story frontmatter (the `timeout_secs` field
appears only in passing in §AC-001 — "scalar tunables" — and in the E-SPEC-023 message).

**Paper-fix-consumer-doesnt-read class?:** YES

---

## F-PR155-REDUX-PRR-002 — `rate_limit_hints` overlay merge is a paper-fix; PipelineExecutor reads TYPE spec, never the merged ResolvedSensorSpec

**Severity:** HIGH
**Category:** paper-fix / consumer-doesnt-read (Lesson 50, TD-VSDD-059)
**File:lines:** `crates/prism-spec-engine/src/overlay.rs:607-622` + `crates/prism-spec-engine/src/pipeline.rs:351-356` + `crates/prism-sensors/src/fanout.rs:587-649`

**Finding:**
`merge_overlay_onto_type_spec` merges `overlay.rate_limit_hints.requests_per_second` and
`burst_size` into `merged_spec.rate_limit_hints` (overlay.rs:607-622) and updates the
provenance bools. This produces a `ResolvedSensorSpec` whose `spec.rate_limit_hints`
correctly reflects the per-org override.

However, the rate-limit consumer is `PipelineExecutor::execute_impl` (pipeline.rs:351),
which reads `spec.rate_limit_hints.requests_per_second` from its `spec: &SensorSpec`
parameter. That parameter is the spec_engine `SensorSpec` loaded from the
`ConfigSnapshot.sensor_specs` map in `parse_spec_directory` — NOT the overlay-merged
`ResolvedSensorSpec` from `OverlayLoader::load_overlays`. There is no production path that
threads `ResolvedSensorSpec.spec` (with merged rate_limit_hints) into
`PipelineExecutor::execute`.

Furthermore, `resolve_spec_for_fanout` only injects `base_url` into the adapter's
`sensor_config`. The merged `rate_limit_hints` are never injected. So even if a future
adapter consulted `sensor_config["rate_limit_hints"]`, the value would be absent.

The two test paths that exercise the merge
(`test_BC_2_06_012_overlay_discovered_and_merged` and the F-LP1-MED-002 test) only assert
that `resolved.spec.rate_limit_hints` carries the value and `provenance.rps_from_overlay`
is true. Neither asserts that the production rate-limiter sees the per-org rate. This is
the same paper-fix shape as F-PR155-REDUX-PRR-001.

**Suggested fix:**
Same options as F-PR155-REDUX-PRR-001:
A. Thread `ResolvedSensorSpec` (or its rate_limit_hints sub-tree) into the production
   `PipelineExecutor` invocation, OR inject `sensor_config["rate_limit_hints"]` in
   `resolve_spec_for_fanout` AND wire the consumer to read it. Add a test that drives
   the production rate-limiter with an overlay-provided rate and asserts the delay
   between requests reflects the override.
B. Remove `rate_limit_hints` from `SensorInstanceOverlay`, the merge function, and the
   error-taxonomy E-SPEC-023 message — if no consumer is ready in this story.

**Paper-fix-consumer-doesnt-read class?:** YES

---

## F-PR155-REDUX-PRR-003 — `base_url` overlay injection into sensor_config is read by zero production adapters; only the test's CapturingAdapter consumes it

**Severity:** HIGH
**Category:** paper-fix / consumer-doesnt-read (Lesson 50, TD-VSDD-059)
**File:lines:** `crates/prism-sensors/src/fanout.rs:619-637` + `crates/prism-sensors/src/auth/armis.rs:517, 560-610` + `crates/prism-sensors/src/auth/{crowdstrike,cyberint,claroty}.rs`

**Finding:**
`resolve_spec_for_fanout` (fanout.rs:619-636) injects the overlay `base_url` into
`resolved_adapter_spec.sensor_config["base_url"]` before dispatch. The
F-LP2-CRIT-001 E2E test (`test_F_LP2_CRIT_001_fan_out_with_overlay_map_routes_to_overlay_url`,
fanout.rs:965) verifies that a `CapturingAdapter` — defined inside the test module — reads
`spec.sensor_config["base_url"]` and observes the overlay URL.

The catch: **no production adapter reads `sensor_config["base_url"]`.** Verified by
greppping `sensor_config` across all four production adapters in
`crates/prism-sensors/src/auth/{armis,crowdstrike,cyberint,claroty}.rs`:

- `ArmisAdapter::fetch` (auth/armis.rs:572-610) uses `self.instance_url` (set at
  construction from `auth.instance_url`, auth/armis.rs:384). The `spec.sensor_config`
  field is only inspected for `aql_query` (auth/armis.rs:419), never `base_url`.
- Other adapters use the same `self.<some>_url` pattern set at constructor time.

The adapter is constructed once via `AdapterRegistry::register` (presumably at boot with
a fixed URL from the auth credential or a single TYPE spec base_url) and the per-org overlay
URL passed via `sensor_config` is silently ignored. The E2E test passes because the test
explicitly authors an adapter that reads `sensor_config["base_url"]` — that adapter does not
exist in production code.

This means **AC-003 is not satisfied in production.** The acceptance criterion states:
"The sensor adapter's HTTP client dispatches to the per-org endpoint" — but no real adapter
currently does so. Multi-tenant Armis dispatch would all hit whatever URL was bound at the
single `ArmisAdapter` construction, NOT per-org overlays.

This is the F-LP2-CRIT-001 "paper-fix" recurrence: the fix-burst added
`fan_out_with_overlay_map` and the Arc-DI plumbing, but did not change the adapters to read
the threaded value. The test-only `CapturingAdapter` masks the gap.

**Suggested fix:**
Either:

A. **Refactor adapter construction to be per-org per-fetch:** instead of constructing
   `ArmisAdapter` once with a constructor URL, construct it (or rebuild its HTTP client)
   per `fetch()` call from `spec.sensor_config["base_url"]`. This is the production-grade
   path; it makes per-org dispatch real.
B. **Refactor adapter to read `spec.sensor_config["base_url"]` on each `fetch()`:**
   `ArmisAdapter::fetch` would override `self.instance_url` for the request when
   `spec.sensor_config["base_url"]` is `Some`. Less invasive than (A).
C. **Use an adapter registry keyed by `(org_id, sensor_id)`** so the registry lookup
   already returns an adapter pre-bound to the overlay URL. This matches the existing
   `AdapterRegistry.get(org_id, sensor_id)` signature in fanout.rs:337.

Whichever path, add a production-adapter test (not `CapturingAdapter`) that hits a wiremock
endpoint and asserts the actual outbound request URL matches the overlay base_url. The
F-LP2-CRIT-001 closure relied on a test-only adapter, which is the canonical SID-1 problem
("Red Gate test passes via a substitute that doesn't exercise the production path").

**Paper-fix-consumer-doesnt-read class?:** YES

---

## F-PR155-REDUX-PRR-004 — Production reqwest::Client constructed without 30s timeout in ArmisAdapter (CLAUDE.md forbidden pattern)

**Severity:** HIGH
**Category:** convention / production-grade-default
**File:lines:** `crates/prism-sensors/src/auth/armis.rs:377-380`

**Finding:**
`ArmisAdapter::new` constructs its production HTTP client as:

```rust
let http = Client::builder()
    .cookie_store(false)
    .build()
    .unwrap_or_default();
```

CLAUDE.md "Forbidden patterns" table lists `reqwest::Client::new() without .timeout() in
production code` as a TD-S-PLUGIN-PREREQ-B-005 P2 open gap with mandatory 30s timeout. The
above also fails this discipline — no `.timeout(Duration::from_secs(30))`. While this
predates the current PR (it is not strictly an in-diff issue), the PR introduces a new code
path (`fan_out_with_overlay_map`) that funnels every per-org request through this
unbounded-timeout HTTP client. The story spec §Library and Framework Requirements explicitly
calls out: "the fanout wiring change touches the HTTP dispatch path — verify the existing
client has the timeout set."

Additionally, `unwrap_or_default()` on a `Client::build()` failure silently degrades to a
default client that may also lack the timeout. The fallback is silent.

**Suggested fix:**
```rust
let http = Client::builder()
    .cookie_store(false)
    .timeout(Duration::from_secs(30))
    .build()
    .map_err(|e| SensorError::Internal {
        detail: format!("ArmisAdapter HTTP client construction failed: {e}"),
    })?;
```
(Adjust the `new()` signature to return `Result` if needed; or panic at boot, since a
failed HTTP client build is unrecoverable.) Sibling-sweep all four adapters
(`armis`, `crowdstrike`, `cyberint`, `claroty`) — the story says only `armis` is changed by
the overlay flow but the pattern likely repeats.

**Paper-fix-consumer-doesnt-read class?:** NO (this is a missing-timeout, not a missing-consumer)

---

## F-PR155-REDUX-PRR-005 — `build_type_spec_map_for_overlay` silently swallows parse/IO errors; can produce misleading E-SPEC-019 downstream

**Severity:** MED
**Category:** error-handling / silent-partial-failure (Standing Rule 3 §2)
**File:lines:** `crates/prism-bin/src/boot.rs:756-786`

**Finding:**
`build_type_spec_map_for_overlay` walks `spec_dir` for `*.sensor.toml` files and runs
`SpecLoader::parse` on each. On parse failure OR IO read failure for an individual file, the
function `tracing::warn!`s and continues — the failed sensor is NOT inserted into the
returned `type_specs` map. The comment claims "already validated by step4_load_sensor_specs
above; log and skip" — but that is not actually true:

- `parse_spec_directory` (config_manager.rs:75-143) returns `Ok(ConfigSnapshot)` even when
  individual specs fail to parse (they are recorded in `failed_specs` but not returned as
  errors).
- `step4_load_sensor_specs` does not check `failed_specs.is_empty()`; it
  returns `Ok(manager)` regardless.
- So a `.sensor.toml` file that fails to parse will:
  1. Survive `step4_load_sensor_specs` silently (no boot abort).
  2. Be re-attempted by `build_type_spec_map_for_overlay`, fail again, and be silently
     skipped.
  3. Cause any overlay file `customers/<org>/<failed_sensor>.sensor.toml` to emit
     `E-SPEC-019: extends references unknown TYPE` — but the real problem is a parse
     failure in the TYPE spec, not an unknown extends.

The user sees a misleading error and may waste hours adjusting their `extends` field.

Additionally, the "log and skip" pattern violates Standing Rule 3 §2 (silent Vec::new()
return where partial-failure data should propagate). The two `tracing::warn!` calls do not
have an event_type field, so they may not be visible at the default log level — and even
if visible, they are advisory rather than error.

**Suggested fix:**
Either:

A. Surface the partial failure as a `BootError::ConfigInvalid` that includes the failed
   spec path and the parse error, aborting boot at step 4. This is the production-grade
   default — boot should not proceed silently with a corrupt TYPE spec.
B. At minimum, add `event_type = "type_spec_parse_failed_during_overlay_build"` to the
   warn macros (with a corresponding BC-2.16.002 catalog row per SAP-1) and adjust the
   overlay E-SPEC-019 emission to mention "TYPE spec may have failed to parse — check
   step4 warnings" so the user has a chance to diagnose.

Option A is preferred — silent skips of unparseable TYPE specs at boot are a
TD-VSDD-091-class production gap.

**Paper-fix-consumer-doesnt-read class?:** NO

---

## F-PR155-REDUX-PRR-006 — `OverlayLoader::e_spec_022_unknown_org_slug` is associated-fn while siblings are free fns; naming pattern inconsistent

**Severity:** MED
**Category:** naming / API consistency
**File:lines:** `crates/prism-spec-engine/src/overlay.rs:645, 668, 686, 704, 726`

**Finding:**
Five E-SPEC-NNN error builders are defined in `overlay.rs`:

| Builder | Signature kind | Naming prefix |
|---------|----------------|---------------|
| `OverlayLoader::e_spec_022_unknown_org_slug` | associated fn on `OverlayLoader` | NO `make_` prefix |
| `make_e_spec_019_unknown_extends` | free fn (module-scope) | `make_` prefix |
| `make_e_spec_020_instance_id_mismatch` | free fn | `make_` |
| `make_e_spec_021_tables_in_overlay` | free fn | `make_` |
| `make_e_spec_023_unrecognized_field` | free fn | `make_` |

Four of the five are free functions with a `make_` prefix; E-SPEC-022 is an associated
function on `OverlayLoader` with no prefix. This is gratuitous inconsistency. Callers of
the error builders have to remember "E-SPEC-022 is special". A future maintainer adding
E-SPEC-024 has to decide which precedent to follow.

The justification in the code says "Canonical message template per error-taxonomy.md row
E-SPEC-022" — the same justification appears on all five. Nothing about E-SPEC-022's
semantics requires it to be a method on `OverlayLoader`.

**Suggested fix:**
Move `e_spec_022_unknown_org_slug` out of `impl OverlayLoader` to be a free function
`make_e_spec_022_unknown_org_slug(customers_dir_name: &str, slug: &str) -> PrismError`
sibling to the other four. Update the one caller at overlay.rs:308. Sibling-sweep
test-code references (none in production code outside `load_overlays`).

**Paper-fix-consumer-doesnt-read class?:** NO

---

## F-PR155-REDUX-PRR-007 — Demo evidence cites stale SHA d600f7f4 (HEAD is 515fdc2e); evidence-report.md is one commit behind

**Severity:** MED
**Category:** evidence-fidelity / process
**File:lines:** `docs/demo-evidence/S-CONFIG-MULTI-TENANT-OVERRIDE-001/evidence-report.md:3`

**Finding:**
The demo `evidence-report.md` line 3 says: `**Feature HEAD:** d600f7f4`. Actual feature
HEAD is `515fdc2e` (the F-LP14-CI-001 CI portability fix landed after demo recording). All
22 demo artifacts (gif/tape/webm) were captured from `d600f7f4` and have NOT been re-recorded
to reflect the AC-005 test change from runtime walk-up to `include_str!()`-based fixture.

This is not a functional defect (AC-005's behavior is unchanged; the test now uses an
embedded fixture instead of walking up to find `.factory/`), but it is a process gap. The
PR-LEVEL convergence claim "demo evidence present — 22 artifacts, 7 ACs" in the PR body
implies the demos reflect the merged code. They do not.

**Suggested fix:**
Re-record AC-005's demo from current HEAD `515fdc2e`, OR add a note to evidence-report.md
that AC-005 demo was captured at `d600f7f4` and the only post-recording change was the
F-LP14-CI-001 CI-portability fix (mechanical refactor of fixture loading; behavior
unchanged). Update the `Feature HEAD:` line to `515fdc2e`.

**Paper-fix-consumer-doesnt-read class?:** NO

---

## F-PR155-REDUX-PRR-008 — AC-005 byte-equality safety net relies on manually-synced `fixtures/error-taxonomy-snapshot.md` with no CI gate

**Severity:** MED
**Category:** test-gap / drift-risk
**File:lines:** `crates/prism-spec-engine/fixtures/error-taxonomy-snapshot.md` + `crates/prism-spec-engine/tests/overlay_loading_tests.rs:757`

**Finding:**
The F-LP14-CI-001 fix (commit 515fdc2e) replaces a runtime walk-up to
`.factory/specs/prd-supplements/error-taxonomy.md` with an `include_str!()` of a
branch-tracked snapshot `crates/prism-spec-engine/fixtures/error-taxonomy-snapshot.md`.
The justification is correct (CI doesn't check out the `.factory/` orphan branch), but the
solution creates a NEW drift surface: the snapshot must be kept byte-equal to the canonical
source manually.

The snapshot file's header says:

> "this fixture must be kept byte-equal to the corresponding rows in the canonical source.
> When the canonical taxonomy E-SPEC-019..023 rows are updated, this fixture MUST be updated
> in the same commit."

But there is **no CI gate** that enforces this. A future change to error-taxonomy.md (e.g.,
a typo fix or wording amendment) would not fail any check until someone runs the AC-005
test against new production emission. The original POL-25 safety net was designed to fail
on drift between canonical taxonomy and code; the new design fails on drift between
snapshot-fixture and code, but the canonical taxonomy can silently diverge from the
snapshot without anyone noticing until a separate audit.

**Suggested fix:**
Add a script (e.g., `scripts/check-error-taxonomy-snapshot.sh`) that compares the snapshot
file to the canonical rows in `.factory/specs/prd-supplements/error-taxonomy.md` when the
factory-artifacts worktree IS mounted (e.g., in pre-commit or as a `just check`
sub-recipe). When `.factory/` is not present (CI default), the script no-ops with a
warning. Alternatively, expose a `prism` CLI subcommand that round-trips the production
emission against both the snapshot AND the canonical taxonomy when run locally with
`.factory/` mounted.

**Paper-fix-consumer-doesnt-read class?:** NO (it's a drift gap, not a consumer gap)

---

## F-PR155-REDUX-PRR-009 — `OverlayLoader::load_overlays` allocates `slug_str` twice and re-constructs `OrgSlug` in the second loop

**Severity:** LOW
**Category:** code quality / minor inefficiency
**File:lines:** `crates/prism-spec-engine/src/overlay.rs:295-323`

**Finding:**
The two-pass design of `load_overlays`:

1. First pass: iterate `org_dirs`, construct `OrgSlug::new(slug_str)` once, push validity
   bool into `slug_registered_map`.
2. Second pass: iterate `org_dirs.iter().zip(slug_registered_map.iter())`, construct
   `OrgSlug::new(slug_str)` AGAIN (line 323), then use it.

The `OrgSlug` produced in the first loop is discarded. The two-pass design is needed for
multi-error aggregation (E-SPEC-022 errors must be collected before any file-level errors),
but the `OrgSlug` could be stored once and re-used. Either store `(slug_str, OrgSlug,
PathBuf, is_registered)` tuples after pass 1, or precompute `Vec<(String, PathBuf, OrgSlug,
bool)>` so pass 2 doesn't recreate `OrgSlug::new`.

`OrgSlug::new` runs the kebab-case regex on each call. For O(N orgs) overlay loads at boot,
this is negligible. The smell is duplication, not perf.

**Suggested fix:**
```rust
struct OrgDirEntry { slug_str: String, path: PathBuf, org_slug: OrgSlug, is_registered: bool }
let org_entries: Vec<OrgDirEntry> = org_dirs.into_iter().map(|(slug_str, path)| {
    let org_slug = OrgSlug::new(slug_str.as_str());
    let is_registered = org_slug.is_ok() && org_registry.resolve(&org_slug).is_some();
    if !is_registered {
        errors.push(Self::e_spec_022_unknown_org_slug(&format!("customers/{slug_str}/"), &slug_str));
    }
    OrgDirEntry { slug_str, path, org_slug, is_registered }
}).collect();
// then loop org_entries once for file scanning
```

**Paper-fix-consumer-doesnt-read class?:** NO

---

## F-PR155-REDUX-PRR-010 — E-SPEC-022 error message echoes raw user-controlled directory name; minor log-injection vector

**Severity:** LOW
**Category:** security / log-injection
**File:lines:** `crates/prism-spec-engine/src/overlay.rs:645-657, 307-308`

**Finding:**
The E-SPEC-022 error message is constructed as:

```rust
"Per-org overlay directory 'customers/{customers_dir_name}/' references org slug '{slug}' \
 which is not registered in OrgRegistry. ..."
```

where `customers_dir_name` and `slug` come from `slug_str` (the raw filesystem directory
name). `OrgSlug::new` documents that raw input may contain attacker-controlled data and
explicitly avoids echoing it for that reason (tenant.rs:67-72 — "Do NOT echo the raw input —
it may contain attacker-controlled data (null bytes, Unicode, shell metacharacters) that
would constitute a log-injection vector").

The overlay loader violates this discipline: the raw `slug_str` is echoed into both the
error message AND `SpecError.file_path`, regardless of whether the directory name passed
the `OrgSlug` validation. If an operator (or an attacker who can create dirs in the prism
config tree) places `customers/$(curl evil.com)/`, the error message and log forwards will
contain the raw string.

The threat model is small: the `customers/` directory is operator-controlled config. But
defense-in-depth says echo a sanitized version (or just the byte-length) when the slug
fails `OrgSlug::new`.

**Suggested fix:**
In `e_spec_022_unknown_org_slug`, check whether the slug satisfies the `ORG_SLUG_PATTERN`
regex (via `OrgSlug::new(slug).is_ok()`). If invalid, replace with a sanitized form like
`<invalid-slug-{N}-chars>`. Pre-existing canonical message templates in the taxonomy would
need updating — this is a sibling-sweep impact.

Minimum acceptable fix: log a `tracing::warn!` with `event_type = "invalid_slug_dir_present"`
before emitting E-SPEC-022, so the audit log carries the raw name (where ops expect it)
but the user-facing error message uses a placeholder.

**Paper-fix-consumer-doesnt-read class?:** NO

---

## F-PR155-REDUX-PRR-011 — Overlay grammar permits no test for file-permission denied (chmod 000) or oversized files

**Severity:** LOW
**Category:** test-gap / negative-path coverage
**File:lines:** `crates/prism-spec-engine/tests/overlay_loading_tests.rs` (entire file)

**Finding:**
The overlay test suite covers happy paths and the 5 E-SPEC validation paths, plus
multi-error aggregation. It does NOT cover:

1. **`customers/acme/armis.sensor.toml` with chmod 000** (file unreadable): the
   `PrismError::Io` branch in `load_overlays` (overlay.rs:329, 338, 360) is unexercised.
2. **Oversized overlay file** (e.g., 100 MB of TOML): there is no size limit on
   `std::fs::read_to_string` (overlay.rs:357). A pathologically large overlay file would
   OOM the boot process before parse failure.
3. **Symlink loops or recursion** under `customers/`: `std::fs::read_dir` does not follow
   symlinks but doesn't reject them either; a `customers/acme -> ../other/` symlink could
   produce surprising behavior.
4. **Mixed-case directory** (e.g., `customers/ACME/`): the boot regex requires kebab-case
   for `[[orgs]]` entries in prism.toml but `OrgSlug::new` accepts mixed case. `ACME` would
   produce E-SPEC-022 (unregistered) which is correct, but the error message says
   "unregistered" rather than the more accurate "case mismatch with registered slug" —
   confusing.

The story spec §Edge Cases enumerates 12 EC-NNN cases but doesn't include any of these. The
absence is a Canonical Principle Rule 1 violation only if the cases are "should have been
covered in scope"; for permission-denied and oversized-file, they ARE in-scope per the
production-grade default ("write the edge case test now").

**Suggested fix:**
Add 3-4 negative tests:
- `test_BC_2_06_012_overlay_file_unreadable_returns_io_error` (chmod 000)
- `test_BC_2_06_013_oversized_overlay_file_rejected` (with a sane size cap added to
  `load_overlays`, e.g., 64 KB per overlay)
- `test_BC_2_06_015_mixed_case_org_dir_produces_e_spec_022`

The size-cap test motivates adding a `MAX_OVERLAY_BYTES = 64 * 1024` constant and an
`std::fs::metadata().len()` check before `read_to_string`.

**Paper-fix-consumer-doesnt-read class?:** NO

---

## F-PR155-REDUX-PRR-012 — Story spec references `OrgRegistry::slug_exists(slug)`; code calls `org_registry.resolve(&org_slug).is_some()` instead

**Severity:** LOW
**Category:** naming / spec-vs-code drift
**File:lines:** Story spec line 261 + `crates/prism-spec-engine/src/overlay.rs:299-300`

**Finding:**
Story AC-004 says "every `customers/<slug>/` directory entry is cross-checked against
`OrgRegistry` via `OrgRegistry::slug_exists(slug)`". The implementation uses
`org_registry.resolve(&org_slug).is_some()` because `OrgRegistry` does not define a
`slug_exists` method. Behavior is equivalent; naming differs.

This is a low-impact drift. The story spec wins on naming (Source-of-Truth Precedence rule
#7 — "For code-vs-spec conflicts: the SPEC wins"). Either:

A. Add a `pub fn slug_exists(&self, slug: &OrgSlug) -> bool` thin wrapper on `OrgRegistry`
   and have the overlay code call it for readability.
B. Amend the story spec to read "via `OrgRegistry::resolve(slug).is_some()`" — but the spec
   wins per the precedence rule, so this is the wrong direction.

**Suggested fix:** Option A. Two-line wrapper on `prism_core::org_registry`.

**Paper-fix-consumer-doesnt-read class?:** NO

---

## F-PR155-REDUX-PRR-013 — Test crate `non-exhaustive-violation` uses `OrgSlug::new_unchecked("acme")` outside `#[cfg(feature = "test-helpers")]`

**Severity:** OBS
**Category:** convention / forbidden-pattern (advisory)
**File:lines:** `tests/external/non-exhaustive-violation/src/struct_violations.rs:341`

**Finding:**
CLAUDE.md "Forbidden patterns" lists `OrgSlug::new_unchecked outside #[cfg(feature =
"test-helpers")]` as a credential-safety violation (AD-017). The new function
`v35_resolved_sensor_spec` calls `OrgSlug::new_unchecked("acme")` directly with no feature
gate.

The mitigating context: the `non-exhaustive-violation` crate has an empty `[workspace]`
table in its `Cargo.toml`, opting out of the parent workspace. It is never built by the
production binary. The `new_unchecked_audit.rs` only scans `crates/prism-core/src/`. So the
production audit gate doesn't fire on this call.

Still, the forbidden-pattern rule is "outside test-helpers"; the `non-exhaustive-violation`
crate is a test crate but not feature-gated. This is on the boundary of acceptable.

**Suggested fix:**
Either:
A. Use `OrgSlug::new("acme").unwrap()` instead — it's safe since "acme" passes the regex.
B. Add `required-features = ["test-helpers"]` to the relevant test in
   `prism-spec-engine/Cargo.toml`, OR add a `cfg(feature = "test-helpers")` propagation.
C. Document the exception inline ("test fixture in compile-fail crate; not production").

Option A is the smallest cost. The `unwrap()` is safe because "acme" is a literal known to
pass the regex.

**Paper-fix-consumer-doesnt-read class?:** NO

---

## F-PR155-REDUX-PRR-014 — `merge_overlay_onto_type_spec` clones the entire TYPE spec per overlay; for many sensors × many orgs this scales poorly

**Severity:** OBS
**Category:** performance / scale
**File:lines:** `crates/prism-spec-engine/src/overlay.rs:589-639`

**Finding:**
For each `(org, sensor)` overlay, the function deep-clones `type_spec.clone()` (line 595)
to produce a new `SensorSpec`. Each `SensorSpec` carries `tables: Vec<TableSpec>` with
nested `columns` and `steps` Vecs. For a deployment with 50 orgs × 10 sensors with overlays,
this is 500 deep clones at boot. For each sensor spec averaging a few KB of nested data,
this is single-digit MB peak — acceptable for boot.

However, the merged `ResolvedSensorSpec` field `spec` holds an *owned* copy of the entire
TYPE spec including all `tables`, even though INV-OVL-001 guarantees tables are immutable
and inherited from TYPE spec. This is wasted memory: 500 copies of identical table arrays.

**Suggested fix (deferred-acceptable):**
Change `ResolvedSensorSpec.spec` to hold only the overlay-derived scalars (base_url,
rate_limit_hints) and reference the TYPE spec by Arc:

```rust
pub struct ResolvedSensorSpec {
    pub type_spec: Arc<SensorSpec>,  // shared, immutable
    pub effective_base_url: String,
    pub effective_rate_limit_hints: Option<RateLimitHints>,
    pub provenance: OverlayProvenance,
    pub org_slug: OrgSlug,
    pub instance_id: String,
}
```

This is a meaningful refactor and may be deferred to a follow-up story (e.g., when overlay
hot-reload is wired). For now, the boot-time memory cost is bounded. Filing as OBS, not
MED.

**Paper-fix-consumer-doesnt-read class?:** NO

---

## Summary

| Severity | Count |
|----------|-------|
| CRIT | 0 |
| HIGH | 4 |
| MED | 4 |
| LOW | 4 |
| OBS | 2 |

**Total findings:** 14.

**CLEAN(strict):** no
**CLEAN(PR-merge):** no — 4 HIGH findings exceed the PR-merge threshold (CLEAN(PR-merge) requires zero CRIT+HIGH+MED)

### Severity rationale

The 4 HIGH findings are all production-grade-default violations that cluster around the
same theme: **the overlay grammar promises behavior its consumers don't deliver.**

- **F-PR155-REDUX-PRR-001 (timeout_secs paper-fix)** — Documented "HTTP timeout override"
  with zero downstream wiring. This is the canonical instance from the original pr-pass-1
  HIGH finding; verified and amplified.
- **F-PR155-REDUX-PRR-002 (rate_limit_hints paper-fix)** — Same class as 001; merged into
  ResolvedSensorSpec but the rate-limit consumer reads TYPE spec.
- **F-PR155-REDUX-PRR-003 (base_url not read by production adapters)** — The PR's F-LP2-CRIT-001
  closure passes via a test-only CapturingAdapter. No production adapter
  consumes `sensor_config["base_url"]`. AC-003 not actually satisfied in production. This is
  arguably CRIT but I have classed it HIGH because the test infrastructure exists and the
  refactor is small.
- **F-PR155-REDUX-PRR-004 (reqwest::Client without timeout)** — Pre-existing pattern, but
  the PR's new fanout path funnels every per-org dispatch through this unbounded client.

The 4 MED findings are real bugs that don't block on production-grade default (silent
parse swallow, naming inconsistency, stale demo SHA, drift-risk on the AC-005 snapshot
fixture).

The 4 LOW + 2 OBS findings are quality concerns that are fixable in scope but not blockers.

### Recommended disposition

REQUEST_CHANGES on PR #155. The 4 HIGH findings together represent a "the wiring is not
done" gap that the LOCAL adversarial cascade (13 passes) and the prior pr-pass-1 missed.
The 3-CLEAN(PR-merge) convergence claim was reached against tests that mask the production
gaps — F-LP2-CRIT-001's E2E test uses a test-only adapter; the timeout_secs test only
checks provenance bool. None of these tests would fail with the current
production-adapter-doesn't-read state.

Per CLAUDE.md Companion Principle (Surface vs Defer): route F-PR155-REDUX-PRR-001/002/003
to `implementer` for end-to-end wiring; route F-PR155-REDUX-PRR-004 to `implementer` for
the `.timeout()` addition with sibling-sweep across all 4 adapters; route
F-PR155-REDUX-PRR-005 to `implementer` or `product-owner` (depending on whether the fix is
to add a BootError variant or amend BC-2.06.012). F-PR155-REDUX-PRR-006/009/012/013 are
cosmetic/naming and can be batched into one fix-burst.

This is the third-line review fresh-context catch that the PR-LEVEL cascade is designed
for. The Lesson 50 paper-fix-consumer-doesnt-read class (lessons.md entries from
PLUGIN-MIGRATION-001-D pass-1 FB-IMPL-1) is the dominant pattern here. The 4 HIGH findings
re-derive cleanly without access to `.factory/` and are anchored to the diff file:line
references plus the diff-and-spec mismatch (story §AC-003 wording "The sensor adapter's HTTP
client dispatches to the per-org endpoint").

### Notes

- BC-2.16.002 `overlay.loaded` catalog row is present (verified in source on develop). SAP-1
  compliant.
- Error taxonomy E-SPEC-019..023 rows are present and 3-way aligned (BC ↔ taxonomy ↔ code).
  AC-005 verified.
- `#[non_exhaustive]` discipline maintained — 3 new overlay types added; EXPECTED=35 in
  ci.yml and check-non-exhaustive.sh both bumped; struct_violations.rs sweeps in 3 new
  v33/v34/v35 violations.
- No `todo!()` / `unimplemented!()` / production `.unwrap()` in overlay.rs (acceptable
  defensive arm at overlay.rs:389-403 with structured error fallback).
- `prism-spec-engine` does NOT take a new dep on `prism-sensors` (forbidden-deps rule
  respected).
- 11 Red Gate tests authored (1808-line test file); no `#[ignore]` (SID-1 compliant).
- Diff size 4263+/28- is large but largely test code and demo evidence. Production code
  delta is ~1100 LOC (overlay.rs 756 + boot extension ~180 + fanout extension ~125 + error
  variants ~35).
