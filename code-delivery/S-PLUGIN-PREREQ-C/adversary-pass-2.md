# Adversarial Review — S-PLUGIN-PREREQ-C — LOCAL Pass 2

**HEAD**: 5e608b76
**Reviewer**: adversary (fresh-context)
**Date**: 2026-05-12
**Subject**: Fix-burst-1 closure verification (10 commits 07b1f07b → 5e608b76) of pass-1's 18 findings + new-finding sweep
**Streak status**: this pass is attempt 2 toward 3-CLEAN cascade

---

## Closure Verification Table

| Pass-1 Finding | Closure Status | Evidence |
|---|---|---|
| F-LP1-CRIT-001 (AC-5 CI inert) | **PARTIAL — has paper-fix surface** | CI job `non-exhaustive-violation-compile-fail` added; workspace `exclude` updated; positive-coverage log present. BUT assertion threshold is hardcoded `-lt 8` while log claims 14 types — see F-LP2-CRIT-001 below. |
| F-LP1-CRIT-002 (positional `::new()` defeats #[non_exhaustive]) | **REAL** | `impl Default` added for `RateLimitHints`, `FetchStep`, `ColumnSpec`, `TableSpec` (via existing `new_point_in_time`), `CredentialRef` (via derive), `SensorSpec`, `SensorTableDescriptor`. Doc-comments document `..Default::default()` external pattern. Enums (AuthType, PaginationConfig) intentionally lack Default. |
| F-LP1-CRIT-003 (AC-1 page_size:None paper-fix) | **REAL** | `tests/ac_1_cursor_page_size_test.rs` rewritten to call `build_paged_url_for_test` on both first-call AND continuation with page_size:None. In-module duplicate in pipeline.rs also exercises both paths. |
| F-LP1-HIGH-001 (jsonpath_extraction_failed emission) | **REAL** | `SpecEngineError::JsonPathExtractionFailed { detail: String, ... }` in error.rs. Both call sites in pipeline.rs emit `tracing::warn!(event_type="jsonpath_extraction_failed", ...)` BEFORE constructing the error. BC-2.16.002 v1.10 row 15 matches field schema. |
| F-LP1-HIGH-002 (proptest body hardcoded) | **REAL** | `arbitrary_json()` strategy with `prop_recursive(4, 64, 8, ...)`. Path regex includes `~` for RFC 6901. |
| F-LP1-HIGH-003 (escape grammar reconcile) | **REAL** | interpolation.rs doc-comment now says "context-free: `$$` collapses to `$`." Test `test_AC4_escape_context_free_double_dollar_to_single` locks semantics. Story v1.1 AC-4 narrative matches. |
| F-LP1-HIGH-004 (AC-5 sibling-sweep) | **PARTIAL — major siblings still missed** | RateLimitHints, types::SensorTableDescriptor, types::SensorSpec, types::CredentialRef, infusion::CredentialRef, ColumnType, ColumnOptions all now `#[non_exhaustive]`. BUT 11+ additional pub TOML-deserialized types remain unannotated — see F-LP2-HIGH-001 below. |
| F-LP1-HIGH-005 (symbol-keyed allowlist) | **REAL** | `new_unchecked_audit.rs` allowlist now `(file_suffix, type_name)` tuples; `extract_impl_type_name` walks back through `impl <Type>` blocks. |
| F-LP1-HIGH-006 (OrgSlug validated constructor) | **PARTIAL — production migration real, but doc-comment stale** | materialization.rs uses `OrgSlug::new(&synthetic_candidate)` with `synthetic-unmapped` sentinel fallback. Property test exercises 100 UUIDs. BUT tenant.rs doc-comment on `new_unchecked` still asserts "one production fallback path in prism-query/src/materialization.rs" — stale post-migration (see F-LP2-MED-001 below). |
| F-LP1-HIGH-007 (JSONPath caps) | **REAL** | `MAX_JSONPATH_RESULT_SIZE = 100_000`, `MAX_JSONPATH_DEPTH = 32` constants exist. `ExtractionContext` threads correctly: size check before recurse, increment after; depth around wildcard recursion. Tests exercise both caps. `jsonpath_size_cap_exceeded` event matches BC-2.16.002 v1.10 row 16. |
| F-LP1-HIGH-008 (AC-7 doctest tautological) | **REAL** | lib.rs doctest constructs `SensorIdValidationError::TooShort` and matches. Renaming variant would fail-compile. |
| F-LP1-OBS-001 (volatile line-number citations) | **NOT-CLOSED (deferred per pass-1)** | pipeline.rs still has 6+ volatile citations; diff changes have invalidated some (`pipeline.rs:362-370` actual location now line 387 etc.). |
| F-LP1-OBS-002 (page_size=0 doc) | **REAL** | CursorToken page_size doc-comment documents `Some(0)` forwarding behavior. |
| F-LP1-OBS-003 (proptest template strategy too narrow) | **REAL** | Template uses `"\\PC{0,100}"` arbitrary printable strings. |
| F-LP1-OBS-004 (sentinel no-op) | **REAL** | Sentinel removed; comment redirects to in-module pipeline.rs proptest. |
| F-LP1-OBS-005 (TableSpec asymmetric new_point_in_time) | **NOT-CLOSED (LOW deferral acceptable)** | TableSpec still has both `new` and `new_point_in_time`; sibling types only have `new`. Documented in doc-comments. |
| F-LP1-OBS-006 (in-module proptest not exposed) | **NOT-CLOSED (LOW deferral acceptable)** | Design choice — in-module proptest is canonical location. |
| F-LP1-OBS-007 (self-referential dev-dep) | **NOT-CLOSED (LOW deferral acceptable)** | Forward-looking; no production impact. |

**Closure summary: 12 REAL, 3 PARTIAL, 3 deferred-LOW (acceptable).**

---

## Critical Findings

### F-LP2-CRIT-001 — CI assertion threshold hardcoded to 8 while log claims 14 types — false-green vector (POL-11)
- **Severity**: CRITICAL
- **Confidence**: HIGH
- **Category**: ci-as-code / paper-fix / POL-11 violation `[process-gap]`
- **Subject**: `.github/workflows/ci.yml` asserts `if [ "${TOTAL_COUNT}" -lt 8 ]; then ... exit 1` while the success-path log claims "Check passed: ${TOTAL_COUNT} types correctly reject ..." with 14 types enumerated. Reverting `#[non_exhaustive]` from 6 of the 14 types would leave 8 violations — TOTAL_COUNT=8 >= 8 — CI passes silently.
- **Evidence**: ci.yml literal `-lt 8`; tests/external/non-exhaustive-violation/src/main.rs deliberately exercises 14 distinct violations; success-line text enumerates 14 type names but threshold doesn't match.
- **Why CRITICAL**: Textbook POL-11 / CI-as-code false-green vector identified in the META-GAP rule. Same class as prism PR #127 pass-13 F-PG-001 (perimeter-compile-fail inert for 12 passes). HIGH-004's expansion from 8→14 DID propagate to violator crate and log message but the load-bearing threshold assertion stayed at 8 — 6 types can be silently un-annotated.
- **Recommended fix**: Change threshold to:
  ```bash
  EXPECTED=14
  if [ "${TOTAL_COUNT}" -lt "${EXPECTED}" ]; then
      echo "::error::Expected at least ${EXPECTED} E0639/E0004 errors..."
      exit 1
  fi
  ```
  Consider deriving EXPECTED from grep over the violator crate to make threshold runtime-computed.

---

## Important Findings

### F-LP2-HIGH-001 — AC-5 sibling-sweep STILL incomplete — write_endpoint.rs + 7 infusion types remain unannotated
- **Severity**: HIGH
- **Confidence**: HIGH
- **Category**: spec-compliance / sibling-sweep (TD-VSDD-060 BROAD recurrence)
- **Subject**: Workspace still has multiple pub TOML-deserialized types in `prism-spec-engine/src/` lacking `#[non_exhaustive]`:
  1. **`crates/prism-spec-engine/src/write_endpoint.rs`** — `BatchMode` enum, `WriteStep` struct, `WriteEndpointSpec` struct. All `#[derive(Serialize, Deserialize)]`. None have `#[non_exhaustive]`.
  2. **`crates/prism-spec-engine/src/infusion/mod.rs`** — `InfusionType` enum, `BuiltInSourceType` enum, `InfusionSourceConfig`, `InfusionField`, `PipeStageConfig`, `PluginConfig`, `InfusionSpec`. All seven have `#[derive(Serialize, Deserialize)]`. None have `#[non_exhaustive]`. (Only `infusion::CredentialRef` was annotated.)
  3. **`crates/prism-spec-engine/src/types.rs`** — `PaginationType` enum, `ColumnType` enum, `ColumnDef` struct, `SpecStatus` enum, `ClientStatus` enum — all pub Deserialize, none have `#[non_exhaustive]`.
- **Evidence**: grep `^#\[non_exhaustive\]|^#\[derive\(.*Deserialize` against each file confirms absence.
- **Why HIGH**: AC-5 audit promise is "ALL pub TOML-deserialized types in `prism-spec-engine`." HIGH-004 closure expanded scope from 8→14 but 11+ additional sibling types remain unannotated. Adding a new variant to `BatchMode` or field to `WriteEndpointSpec` would silently break every external matcher.
- **Recommended fix**: Either (a) extend AC-5 to apply `#[non_exhaustive]` to all 11+ missed types AND add violation entries to compile-fail crate AND raise CI threshold accordingly, OR (b) explicitly DOCUMENT in story v1.2 that AC-5's audit scope is limited to the 14 enumerated types (file TD for deferred sibling sweep). Choice (a) preferred.

### F-LP2-HIGH-002 — verify-workflow-structure CI job does NOT verify the non-exhaustive job exists — reachability gap
- **Severity**: HIGH
- **Confidence**: HIGH
- **Category**: ci-as-code / POL-11 `[process-gap]`
- **Subject**: `.github/workflows/ci.yml` verify-workflow-structure reachability assertions check for `target:` (AC-5), `cargo.?deny|cargo.?audit` (AC-6), `semver` (AC-7), `no-default-features` (AC-8) — but DO NOT check that the `non-exhaustive-violation-compile-fail` job exists in `ci.yml`. A future PR removing the entire job block would not be caught by structural verification.
- **Why HIGH**: CI job is SOLE enforcement of AC-5's forward-compat property. Future refactor removing/renaming the job, no automated check would detect the regression. META-GAP class — job protecting the perimeter is itself unprotected by reachability tests.
- **Recommended fix**: Add to verify-workflow-structure script:
  ```bash
  grep -qE 'non-exhaustive-violation-compile-fail' .github/workflows/ci.yml || { echo 'AC-5 CI job missing'; exit 1; }
  ```

---

## Observations (Medium/Low)

### F-LP2-MED-001 — OrgSlug::new_unchecked doc-comment is stale post-HIGH-006 migration (S-7.01 partial-fix discipline)
- **Severity**: MEDIUM
- **Confidence**: HIGH
- **Category**: partial-fix discipline (S-7.01) / spec-drift
- **Subject**: `crates/prism-core/src/tenant.rs` doc-comment on `OrgSlug::new_unchecked` still asserts "called from … one production fallback path in `prism-query/src/materialization.rs`" — but materialization.rs migrated to `OrgSlug::new()` (HIGH-006 closure). No new_unchecked call remains in materialization.rs.
- **Why MEDIUM**: Doc-comment describes a production call site that does not exist. Future readers will believe a production caller still depends on `new_unchecked`.
- **Recommended fix**: Update tenant.rs doc-comment: "This function is called from test fixtures only. The prior production caller in prism-query/src/materialization.rs was migrated to `OrgSlug::new()` with a sentinel fallback (HIGH-006 closure, PREREQ-C fix-burst-1). Feature-gating is NOT applied because test fixtures may need it across crate boundaries; the symbol-keyed allowlist in new_unchecked_audit.rs ensures intentional audit."

### F-LP2-MED-002 — Volatile line-number citations have decayed after diff (TD-VSDD-091 worsened)
- **Severity**: MEDIUM
- **Confidence**: HIGH
- **Category**: process-gap (TD-VSDD-091) / partial-fix discipline `[process-gap]`
- **Subject**: pipeline.rs has 6 volatile line-number citations carried over from PREREQ-B. PREREQ-C's diff has shifted line numbers; citations are now demonstrably stale (cited `pipeline.rs:362-370` actual location now line 387 etc.).
- **Why MEDIUM**: Pass-1 OBS-001 deferred this; fix-burst-1 did not address it but the diff worsened staleness. Per partial-fix discipline (S-7.01), prose referencing changed values should be updated.
- **Recommended fix**: Replace each citation with stable anchor (function name): e.g., `"pipeline_truncated emission in PipelineExecutor::execute records-accumulation loop"` instead of `"pipeline.rs:362-370"`.

### F-LP2-OBS-001 — local `just check` does not exercise the non-exhaustive-violation crate
- **Severity**: LOW
- **Confidence**: HIGH
- **Category**: ci-as-code / developer-experience
- **Subject**: `Justfile` has no recipe that runs `cargo check --manifest-path tests/external/non-exhaustive-violation/Cargo.toml` and asserts non-zero exit. Local pre-push `just check` passes even if developer reverts a `#[non_exhaustive]` annotation. Only CI catches it.
- **Recommended fix** (optional): Add `just check-non-exhaustive` recipe mirroring the CI job for local pre-push parity.

### F-LP2-OBS-002 — types.rs has duplicate type families that confuse the surface
- **Severity**: LOW
- **Confidence**: HIGH
- **Category**: design-consistency
- **Subject**: `crates/prism-spec-engine/src/types.rs` declares parallel types — `types::ColumnType` vs `prism_core::column::ColumnType`, `types::PaginationType` vs `spec_parser::PaginationConfig`, `types::SensorSpec` vs `spec_parser::SensorSpec`, etc. Whether they should be unified is architectural beyond pass-2 scope.
- **Recommended fix**: File TD item for "consolidate duplicate types in prism-spec-engine/src/types.rs vs spec_parser.rs vs infusion/mod.rs" — defer to post-PREREQ-C maintenance.

### F-LP2-OBS-003 — TD-VSDD-091 carry-over comment in pipeline.rs cites a TD code (P3) that AC-4 may have closed
- **Severity**: LOW
- **Confidence**: MEDIUM
- **Category**: process-gap
- **Subject**: pipeline.rs has `TD-S-PLUGIN-PREREQ-B-008 P3` comment about "Interpolator grammar has no escape mechanism for literal `${...}`" — but AC-4 (this story) closed TD-S-PLUGIN-PREREQ-B-008 (story frontmatter lists it under `td_resolves`).
- **Recommended fix**: Remove stale TD comment from pipeline.rs since underlying TD is now closed.

---

## Novelty Assessment

**Novelty: MEDIUM.** Most pass-1 findings have real closure. Two NEW substantive findings from fresh-context inspection:
- F-LP2-CRIT-001 is a textbook POL-11 false-green vector that survived pass-1 review.
- F-LP2-HIGH-001 is a recurrence of HIGH-004's sibling-sweep gap, this time covering write_endpoint.rs + infusion/mod.rs.

F-LP2-HIGH-002 is the META-GAP catch (CI job protecting AC-5 is itself unprotected by reachability tests). F-LP2-MED-001 is a textbook S-7.01 partial-fix gap.

The trajectory is improving. PREREQ-A pass-3 caught fix-burst-2 paper-fixes; PREREQ-B pass-8 caught fix-burst-7 paper-fixes; PREREQ-C pass-2 catching fix-burst-1 paper-fix surface (CI threshold) and recurring sibling-sweep gap. Expected pattern.

---

## Total Findings by Severity

| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 1 | F-LP2-CRIT-001 |
| HIGH | 2 | F-LP2-HIGH-001, F-LP2-HIGH-002 |
| MEDIUM | 2 | F-LP2-MED-001, F-LP2-MED-002 |
| LOW (OBS) | 3 | F-LP2-OBS-001, F-LP2-OBS-002, F-LP2-OBS-003 |
| **Total** | **8** | |

---

## 3-CLEAN Streak Status

**0/3.** This pass has 1 CRITICAL + 2 HIGH findings; convergence requires CRIT and HIGH = 0. Streak resets to 0.

---

## Trajectory

PREREQ-C: 18 (pass-1) → **8 (pass-2)** — 56% reduction.

CRIT trajectory: 3 → 1 (closure of CRIT-002 + CRIT-003; CRIT-001 partially closed but NEW CRIT emerged from threshold gap).
HIGH trajectory: 8 → 2 (6 of 8 HIGH closed; HIGH-004 sibling-sweep recurred + new META-GAP).

For comparison: PREREQ-B pass-2 had 10 findings; PREREQ-A pass-2 had 12.

---

## Verdict

**BLOCKED-soft.**

Fix-burst-2 must address:
1. **F-LP2-CRIT-001** — bump CI threshold to 14 (or runtime-computed); positive-coverage assertion alignment with documented count.
2. **F-LP2-HIGH-001** — either extend AC-5 audit to write_endpoint.rs + infusion/mod.rs sibling types (and update compile-fail crate + CI threshold), OR explicitly narrow story scope and file deferred TD items. Implementer must adjudicate intent.
3. **F-LP2-HIGH-002** — add `non-exhaustive-violation-compile-fail` reachability check to verify-workflow-structure script.
4. **F-LP2-MED-001** — update tenant.rs doc-comment on `new_unchecked` to reflect post-HIGH-006 migration (S-7.01 propagation).
5. **F-LP2-MED-002** — clean up volatile line-number citations in pipeline.rs (decided per pass-1 deferral; fix-burst-1 worsened staleness).

After fix-burst-2 lands, pass-3 should re-verify all five close cleanly. If no new HIGH+ findings, pass-3 begins the 3-CLEAN streak (1/3).
