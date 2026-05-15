---
document_type: adversarial-review-report
pass_id: impl-pass-2
story: S-PLUGIN-PREREQ-D
date: 2026-05-14
base_sha: "feature/S-PLUGIN-PREREQ-D@c87592e8"
policy_count: 18
verdict: BLOCKED
streak_pre: "0/3"
streak_post: "0/3"
total_findings: 12
findings_by_severity:
  CRIT: 2
  HIGH: 3
  MED: 6
  LOW: 1
  OBS: 5
net_new_process_gaps: 5
trajectory_note: "impl-pass-1 (18) → impl-pass-2 (12) — decreasing; convergence signal"
state_decision: D-549
---

# S-PLUGIN-PREREQ-D Adversarial Review — Implementation Pass 2

**Pass ID:** impl-pass-2
**Date:** 2026-05-14
**Branch:** feature/S-PLUGIN-PREREQ-D
**HEAD SHA:** c87592e8 (post fix-burst-impl-1 closure of all 18 impl-pass-1 findings)
**Policies active:** 18 (POL-1 through POL-18; POL-15 ADR-023 boot gate directly implicated in F-PASS2-CRIT-001)
**Adversary model:** different-model-family (fresh context; no access to prior pass reports)

---

## 3-Clean Streak

| Pre-pass | Post-pass | Delta |
|----------|-----------|-------|
| 0/3 | 0/3 | BLOCKED — reset; no advance |

---

## Verdict: BLOCKED

12 in-perimeter findings (2 CRIT + 3 HIGH + 6 MED + 1 LOW). 5 process-gap OBS candidates routed session-reviewer. Standing Rule 3 §1 working as designed — adversary independently verified that 2 of the 18 fix-burst-impl-1 closures were paper-fixes rather than load-bearing fixes.

---

## Prior Finding Closure Verification

18 impl-pass-1 findings verified against feature/S-PLUGIN-PREREQ-D@c87592e8.

| Finding ID | Prior Severity | Closure Status | Notes |
|------------|---------------|----------------|-------|
| F-IMPL-LP1-CRIT-001 | CRIT | PAPER-FIX-REOPENED | `run_boot_sequence` calls `plugin_load_step` inside helper, but `main.rs::PrismCommand::Start` bypasses `run_boot_sequence` entirely → dead code. See F-PASS2-CRIT-001. |
| F-IMPL-LP1-CRIT-002 | CRIT | VERIFIED-CLOSED | `PrismConfig.plugin_dir: PathBuf` field present; `#[non_exhaustive]` on struct; AC-1/AC-2 PASS. |
| F-IMPL-LP1-CRIT-003 | CRIT | PARTIAL-CLOSE-OPEN-AS-NEW | `register_host_functions` now registers 5 host functions via `func_new`. Structural shape closed. BUT callback bodies are no-op stubs. See F-PASS2-CRIT-002. |
| F-IMPL-LP1-HIGH-001 | HIGH | VERIFIED-CLOSED | BC-2.16.002 `modified:` field now set to ISO date; frontmatter-sync PASS. |
| F-IMPL-LP1-HIGH-002 | HIGH | VERIFIED-CLOSED | `PluginLoadAuditSink` trait + `RocksDbPluginAuditSink` + `PluginRuntime::new_with_audit_sink` wiring present; durable audit test passing. AC-4 PASS. |
| F-IMPL-LP1-HIGH-003 | HIGH | VERIFIED-CLOSED | `semver::Version::parse` used for manifest version validation. AC-9 PASS. |
| F-IMPL-LP1-HIGH-004 | HIGH | VERIFIED-CLOSED | `ManifestParseError` returns `E-PLUGIN-017` code. Error taxonomy PASS. |
| F-IMPL-LP1-HIGH-005 | HIGH | VERIFIED-CLOSED | `ManifestNotFound` returns `E-PLUGIN-018` code. Error taxonomy PASS. |
| F-IMPL-LP1-HIGH-006 | HIGH | VERIFIED-CLOSED | `FormatVersionMissing` returns `E-PLUGIN-019` code. Error taxonomy PASS. |
| F-IMPL-LP1-MED-001 | MED | VERIFIED-CLOSED | `sensor_id` field corrected in BC-2.16.002 catalog rows. |
| F-IMPL-LP1-MED-002 | MED | VERIFIED-CLOSED | `message` field added to BC-2.16.002 catalog rows. |
| F-IMPL-LP1-MED-003 | MED | VERIFIED-CLOSED | `PluginLoadAuditSink` cascade — wired via HIGH-002 fix. |
| F-IMPL-LP1-MED-004 | MED | VERIFIED-CLOSED | `execute_with_max_requests` pipeline cap branch + wiremock-driven test at c87592e8. AC-16 PASS. |
| F-IMPL-LP1-MED-005 | MED | VERIFIED-CLOSED | `ManifestParseError` variant present in `PluginError` enum. |
| F-IMPL-LP1-MED-006 | MED | VERIFIED-CLOSED | WASI-importing component negative-proof test at 30a7a304. |
| F-IMPL-LP1-MED-007 | MED | VERIFIED-CLOSED | Cosmetic field-name tracing fixes at 1d620e63. |
| F-IMPL-LP1-LOW-001 | LOW | VERIFIED-CLOSED | `plugin_dir` wiring in `run_boot_sequence` at 0e0c85d0. |
| F-IMPL-LP1-LOW-002 | LOW | VERIFIED-CLOSED | Tracing field-name cosmetic fix at 1d620e63. |

**Summary:** 13 VERIFIED-CLOSED + 1 PARTIAL-CLOSE-OPEN-AS-NEW (CRIT-003) + 2 PAPER-FIX-REOPENED (CRIT-001, CRIT-003 body)

---

## New Findings — Implementation Pass 2

### CRITICAL

#### F-PASS2-CRIT-001 — `run_boot_sequence` wiring in dead-code path (POL-15 / ADR-023 §C4 violation)

**Severity:** CRITICAL
**Routing:** implementer
**Anti-pattern class:** F-IMPL-LP1-CRIT-001 recurrence at a different boundary (binary entry point vs. boot helper function)

**Description:**

`run_boot_sequence` in `crates/prism-bin/src/boot/` correctly calls `plugin_load_step`. The helper function implementation is correct. BUT `prism-bin/src/main.rs::PrismCommand::Start` (lines ~107-133) bypasses `run_boot_sequence` entirely — it calls `boot_to_step_6 → step7_init_storage` directly, skipping the plugin-load step altogether.

The plugin-load wiring exists in dead code. The production binary entry point does not invoke it. ADR-023 §C4 mandates the plugin pre-traffic gate runs before any sensor traffic is accepted. POL-15 enforces this. The pre-traffic gate is NOT wired into the production boot path.

**Fix prescription:**

Option A (preferred): Update `main.rs::PrismCommand::Start` to call `run_boot_sequence` instead of `boot_to_step_6 → step7_init_storage` directly.

Option B: Restructure so `plugin_load_step` runs as an explicit step inside the path `main.rs::PrismCommand::Start` takes, before the first `todo!()` or traffic-acceptance point.

**Verification gate:** Grep `main.rs` for `plugin_load_step` OR `run_boot_sequence`; must appear in the `PrismCommand::Start` execution path, not only in a helper function that is never called from Start.

---

#### F-PASS2-CRIT-002 — `register_host_functions` callback bodies are no-op stubs (TD-VSDD-059 paper-fix recurrence)

**Severity:** CRITICAL
**Routing:** implementer
**Anti-pattern class:** TD-VSDD-059 paper-fix; same pattern class as F-IMPL-LP1-CRIT-003 (register-shape-but-not-substance)
**Related finding:** F-PASS2-HIGH-003 (silent error swallowing in kv_set stub)

**Description:**

`register_host_functions` now correctly registers 5 host functions in the Component Model linker via `func_new`. This closed the "unsatisfied import" structural surface (F-IMPL-LP1-CRIT-003 structural claim verified). BUT the callback bodies are no-op stubs:

```rust
// Example from host_functions.rs — actual callback body:
|_caller, _params, _results| {
    trace!(event_type = "host_http_request", "stub called");
    // Results left as default — full response construction deferred to S-4.08.
    Ok(())
}
```

None of the 5 callbacks delegate to the corresponding `host_http_request` / `host_log` / `host_get_config` / `host_kv_get` / `host_kv_set` production functions. AC-7 (VP-PLUGIN-007 plugin-callable allowlist gate) is NOT verified end-to-end through the Component Model entry point.

The adversary independently verifies (Standing Rule 3 §1): the structural registration was fixed; the substance (delegation) was not fixed. This is a paper-fix — the shape changed but the behavior did not.

**Fix prescription:**

For each of the 5 registered host functions, the callback body must:
1. Deserialize `Val` params to typed arguments (e.g., `&str` for URL, `Vec<u8>` for body)
2. Call the corresponding `host_*` production function (e.g., `host_http_request`, `host_log`, `host_get_config`, `host_kv_get`, `host_kv_set`)
3. Serialize the return value back into `results` slice

This does NOT require completing S-4.08 response construction — the delegation to existing production functions must happen now. The "deferred to S-4.08" comment is incorrect justification for stub callbacks in production code.

---

### HIGH

#### F-PASS2-HIGH-001 — BC-2.16.002 prose intro line cites stale catalog version count (sibling-sweep gap)

**Severity:** HIGH
**Routing:** product-owner
**Rule:** TD-VSDD-060 sibling-site sweep; sibling-sweep gap on prose intro lines

**Description:**

BC-2.16.002 line 74 prose still reads:

> `**Canonical Structured Event Catalog (v1.12)** ... contains 25 structured events.`

After 3 amendments (v1.13/v1.14/v1.15), the actual table contains 31 rows. The version label says "v1.12" (should be v1.15) and the count says "25" (should be 31).

The fix-burst-impl-1 sweep updated the `modified:` frontmatter field and the changelog, but missed the prose intro line. This is a sibling-sweep gap (TD-VSDD-060 class).

**Fix prescription:** Update BC-2.16.002 prose intro line 74 to read `**Canonical Structured Event Catalog (v1.15)** ... contains 31 structured events.`

---

#### F-PASS2-HIGH-002 — POL-18 violation: `prism-spec-engine` `[[test]]` blocks lack `required-features = ["test-helpers"]`

**Severity:** HIGH
**Routing:** implementer
**Rule:** POL-18 (feature-gated test helpers must gate their `[[test]]` blocks)

**Description:**

`crates/prism-spec-engine/Cargo.toml` contains `[[test]]` blocks that consume test-helper symbols (`execute_with_max_requests`, `HostState::test_with_*`) without `required-features = ["test-helpers"]`. This means the tests will attempt to compile on targets that don't have test-helpers enabled, and may fail with confusing errors or silently skip.

**Fix prescription:** Add `required-features = ["test-helpers"]` to all `[[test]]` blocks in `prism-spec-engine/Cargo.toml` that use test-helpers symbols.

---

#### F-PASS2-HIGH-003 — `host_kv_set` callback silently swallows errors via `let _ = ...`

**Severity:** HIGH
**Routing:** implementer
**Rule:** SOUL.md #4 + Standing Rule 3 §2 (silent partial-failure data loss)
**Related finding:** F-PASS2-CRIT-002

**Description:**

In `host_functions.rs` (approximately line 423):

```rust
let _ = host_kv_set(state, &key, &value);
```

The `host_kv_set` return value is silently discarded. Any `Err(...)` from `host_kv_set` is dropped without propagation. This is a Standing Rule 3 §2 violation — partial-failure data must propagate, not be swallowed via `let _ = ...`.

**Fix prescription:** Propagate the `host_kv_set` error via `?` or map it to an appropriate `WasmError` / Component Model trap. Silent discard is never acceptable for operations that can fail.

---

### MEDIUM

#### F-PASS2-MED-001 — `test_wasi_not_linked` test has early `return;` escape hatch before assertion

**Severity:** MEDIUM
**Routing:** implementer
**Anti-pattern class:** PG-IMPL-LP2-005 (test escape hatch — negative-coverage gap)

**Description:**

`test_wasi_not_linked` in `crates/prism-spec-engine/tests/plugin_integration_tests.rs` contains an early `return;` before the primary assertion. The test compiles and "passes" without exercising the WASI-linking-blocked behavior it is intended to cover.

**Fix prescription:** Remove the early `return;` escape hatch. The test must exercise the full scenario path to count as a load-bearing closure for F-IMPL-LP1-CRIT-003's WASI negative-proof.

---

#### F-PASS2-MED-002 — Story body has 12 stale `BC-2.16.002 v1.12` references after BC is at v1.15

**Severity:** MEDIUM
**Routing:** story-writer (or product-owner for BC citation lines)
**Rule:** POL-23 BC-version-bump sibling-site grep gate

**Description:**

The story body (S-PLUGIN-PREREQ-D v1.32) contains 12 occurrences of `BC-2.16.002 v1.12` as a version pin, predating the 3 amendments (v1.13/v1.14/v1.15). The POL-23 sibling-site grep gate requires that fix-bursts which bump a BC version sweep all dependent stories for stale pins. This sweep was not performed for v1.13/v1.14/v1.15 bumps.

**Fix prescription:** Update all 12 `BC-2.16.002 v1.12` occurrences in the story to `BC-2.16.002 v1.15`.

---

#### F-PASS2-MED-003 — Story §Structured Event Catalog Additions should enumerate 12 events (not 9)

**Severity:** MEDIUM
**Routing:** story-writer
**Rule:** Accuracy of story implementation spec

**Description:**

The story's §Structured Event Catalog Additions section enumerates 9 event entries. However, 3 additional catalog rows were added during fix-burst-impl-1 (via BC-2.16.002 v1.15): `parse_error`, `not_found`, and `format_version_missing`. The story section should now enumerate 12 events (9 original + 3 from fix-burst-impl-1).

**Fix prescription:** Add 3 rows to story §Structured Event Catalog Additions for `parse_error` / `not_found` / `format_version_missing`.

---

#### F-PASS2-MED-004 — BC-INDEX `timestamp:` field lacks `Z` suffix (non-ISO-8601)

**Severity:** MEDIUM
**Routing:** product-owner (or state-manager for index files)
**Rule:** POL-20 anchored-regex ISO timestamp format

**Description:**

BC-INDEX frontmatter `timestamp:` value lacks the `Z` (UTC) suffix, making it non-conformant with ISO-8601. All other index files use `Z`-suffixed timestamps per POL-20.

**Fix prescription:** Add `Z` suffix to BC-INDEX `timestamp:` field.

---

#### F-PASS2-MED-005 — error-taxonomy frontmatter `timestamp:` is stale and lacks `modified:` field

**Severity:** MEDIUM
**Routing:** product-owner
**Rule:** BC-INDEX sync discipline; frontmatter currency

**Description:**

`error-taxonomy.md` frontmatter shows:
- `timestamp: 2026-05-11T02:00:00Z` (predates v1.23 amendment which added E-PLUGIN-017/018/019)
- No `modified:` field (pattern established for active files)

After the v1.22→v1.23 amendment in fix-burst-impl-1, the timestamp should reflect the amendment date and a `modified:` field should be added.

**Fix prescription:** Update `error-taxonomy.md` frontmatter: set `timestamp:` to `2026-05-14T00:00:00Z` (or the actual commit date), add `modified: 2026-05-14`.

---

#### F-PASS2-MED-006 — error-taxonomy `status: draft` while referenced by active BC-2.16.002

**Severity:** MEDIUM
**Routing:** product-owner
**Rule:** Artifact lifecycle consistency; active BCs should not reference draft supplements

**Description:**

`error-taxonomy.md` has `status: draft` in its frontmatter while BC-2.16.002 (status: active) references the E-PLUGIN error codes that live in error-taxonomy. The referenced document should not be in draft status if it is being actively consumed by a production BC.

**Fix prescription:** Update `error-taxonomy.md` `status: draft` → `status: active` (or an appropriate intermediate status). Align with the lifecycle model used by other active supplements.

---

### LOW

#### F-PASS2-LOW-001 — Story uses `updated:` not `modified:` in frontmatter (intent verification pending)

**Severity:** LOW
**Routing:** story-writer (intent clarification)
**Rule:** POL-20 frontmatter field name normalization

**Description:**

The story (S-PLUGIN-PREREQ-D v1.32) uses `updated:` in frontmatter while project convention uses `modified:` per POL-20. This may be intentional (stories use `updated:` by convention) or may be a field-name drift. Requires intent verification before closure.

**Fix prescription (if not intentional):** Rename `updated:` → `modified:` in story frontmatter. If stories intentionally use `updated:`, codify this exception in POL-20.

---

## Scope-Expansion Adjudication (carry-forward from impl-pass-1)

The 3 scope-expansion adjudications from impl-pass-1 carry forward:

1. **iter_module behavioral substitution (AC-8/AC-11):** REJECTED as insufficient — AC-8 and AC-11 require direct behavioral proof through the Component Model dispatch chain, not behavioral substitution via iter_module. This remains an open AC gap (not re-raised as a net-new finding this pass; carry-forward to impl-pass-3 for final adjudication).

2. **HostState test-helper constructors:** ACCEPTED. `test_with_plugin_id` and `test_with_allowed_urls` are production-grade adaptation to `#[non_exhaustive]`. No defect.

3. **3 net-new event_type emission sites:** PARTIALLY ACCEPTED. The 3 new sites are legitimate precision observability (plugin_directory_not_found, plugin_load_failed_read_error, plugin_load_failed_compilation). The additional 3 sites from fix-burst-impl-1 (parse_error, not_found, format_version_missing) are also accepted. All catalog rows present in BC-2.16.002 v1.15.

---

## Policy Verification Summary

| Policy | Verdict | Notes |
|--------|---------|-------|
| POL-1 (slug preservation) | PASS | No heading changes this burst |
| POL-2 (BC versioning) | FAIL | BC-2.16.002 prose intro cites stale v1.12/count-25 (F-PASS2-HIGH-001) |
| POL-3 (state-manager-last) | N/A | State burst, not applicable |
| POL-4 (factory commit) | N/A | |
| POL-7 (sibling-sweep) | FAIL | BC-2.16.002 prose intro missed; story v1.12 pins missed (F-PASS2-HIGH-001 + F-PASS2-MED-002) |
| POL-11 (index-bump) | PASS | BC-INDEX v4.77 current |
| POL-14 (BC promotion at merge) | N/A | Pre-merge |
| POL-15 (ADR-023 boot gate) | FAIL | `PrismCommand::Start` bypasses `run_boot_sequence` (F-PASS2-CRIT-001) |
| POL-18 (required-features gate) | FAIL | `[[test]]` blocks missing `required-features` (F-PASS2-HIGH-002) |
| POL-20 (ISO timestamp / field naming) | FAIL | BC-INDEX timestamp no-Z; error-taxonomy stale timestamp + missing modified:; story updated: vs modified: (F-PASS2-MED-004/005/LOW-001) |
| POL-22 (BC citation verbatim) | PASS | No new BC citation drift found |
| POL-23 (BC-version-bump sibling grep) | FAIL | Story pins not swept after v1.13/v1.14/v1.15 amendments (F-PASS2-MED-002) |
| POL-24 (error template verbatim) | PASS | Error message templates consistent |
| POL-25 (multi-cite propagation) | PASS | No new propagation gaps |

---

## Carry-Forward OBS (non-blocking, routed session-reviewer)

From impl-pass-1:
- OBS-001 (process-gap): No CI gate verifying BC-INDEX row version matches BC file version — codification queue item #18
- OBS-002 (process-gap): Boot-step "registered but not called" anti-pattern needs lint — codification queue item #19

From impl-pass-2 (new):
- OBS-003 (process-gap): iter_module behavioral substitution gap (AC-8/AC-11) — carry to impl-pass-3 adjudication; NOT re-raised as CRIT this pass per scope-expansion adjudication above

---

## Process-Gap Candidates: Codification Queue Expansion 19 → 24

These are process-gap findings (not code defects). Routed to session-reviewer at cycle-close for codification adjudication. Do NOT add to policies.yaml this burst.

| ID | Description | Trigger finding |
|----|-------------|-----------------|
| PG-IMPL-LP2-001 (#20) | Adversary must specifically verify production binary entry-point coverage after wiring closures. When a boot helper is fixed, adversary must grep the binary `main.rs` entry point to confirm the helper is in the call path. | F-PASS2-CRIT-001 |
| PG-IMPL-LP2-002 (#21) | Component Model host-function registration without callback delegation is a paper-fix pattern. Adversary must inspect callback bodies for delegation to `host_*` function names, not just verify `func_new` was called. | F-PASS2-CRIT-002 |
| PG-IMPL-LP2-003 (#22) | Prose-version-label-vs-changelog drift detection on BC amendments. When a BC is amended, a sibling-sweep must include the prose intro line (e.g., "Catalog (v1.NN) ... contains NN events") in addition to frontmatter fields. | F-PASS2-HIGH-001 |
| PG-IMPL-LP2-004 (#23) | POL-18 `required-features` audit not currently run by consistency-validator or adversary. Should be codified as a mandatory check whenever a new `[[test]]` block is added that uses `test-helpers` symbols. | F-PASS2-HIGH-002 |
| PG-IMPL-LP2-005 (#24) | Test-escape-hatch detection: adversary must scan for early `return;` without prior assertion in negative-coverage tests. These are structurally valid (compile + pass) but functionally hollow. | F-PASS2-MED-001 |

---

## Next-Pass Dispatch Template — fix-burst-impl-2

**Dispatch target:** implementer

**In-perimeter findings to fix (12 total; 2 CRIT + 3 HIGH + 6 MED + 1 LOW):**

### CRIT fixes (implementer)

1. **F-PASS2-CRIT-001 — Production entry point wiring:**
   - File: `crates/prism-bin/src/main.rs` (PrismCommand::Start branch, lines ~107-133)
   - Action: Route through `run_boot_sequence` (Option A) OR add explicit `plugin_load_step` call before traffic acceptance (Option B)
   - Verification: `grep -n 'plugin_load_step\|run_boot_sequence' crates/prism-bin/src/main.rs` must show the call in the Start branch

2. **F-PASS2-CRIT-002 — Component Model callback delegation:**
   - File: `crates/prism-spec-engine/src/host_functions.rs` (all 5 callback bodies)
   - Action: Deserialize `Val` params → call `host_http_request` / `host_log` / `host_get_config` / `host_kv_get` / `host_kv_set` → serialize results
   - Verification: Each callback body must grep-show a call to the corresponding `host_*` function name

### HIGH fixes (implementer)

3. **F-PASS2-HIGH-002 — POL-18 required-features:**
   - File: `crates/prism-spec-engine/Cargo.toml`
   - Action: Add `required-features = ["test-helpers"]` to `[[test]]` blocks consuming test-helper symbols

4. **F-PASS2-HIGH-003 — Silent error swallowing in kv_set:**
   - File: `crates/prism-spec-engine/src/host_functions.rs` (~line 423)
   - Action: Replace `let _ = host_kv_set(state, &key, &value);` with error propagation

5. **F-PASS2-HIGH-001 — BC-2.16.002 prose intro (routed product-owner via orchestrator):**
   - File: `.factory/specs/behavioral-contracts/BC-2.16.002-*.md`
   - Action: Update prose intro line to `v1.15` and count `31`

### MED fixes (implementer + story-writer + product-owner)

6. **F-PASS2-MED-001 — WASI test escape hatch (implementer):**
   - File: `crates/prism-spec-engine/tests/plugin_integration_tests.rs`
   - Action: Remove early `return;` from `test_wasi_not_linked`

7. **F-PASS2-MED-002 — Story stale pins (story-writer):**
   - File: `.factory/stories/S-PLUGIN-PREREQ-D.md`
   - Action: Update 12 occurrences of `BC-2.16.002 v1.12` → `BC-2.16.002 v1.15`

8. **F-PASS2-MED-003 — Story §Structured Event Catalog Additions (story-writer):**
   - File: `.factory/stories/S-PLUGIN-PREREQ-D.md`
   - Action: Add 3 rows for parse_error / not_found / format_version_missing

9. **F-PASS2-MED-004 — BC-INDEX timestamp Z-suffix (state-manager or product-owner):**
   - File: `.factory/specs/behavioral-contracts/BC-INDEX.md`
   - Action: Add `Z` suffix to `timestamp:` field

10. **F-PASS2-MED-005 — error-taxonomy timestamp + modified field (product-owner):**
    - File: `.factory/specs/prd-supplements/error-taxonomy.md`
    - Action: Update `timestamp:` to 2026-05-14T00:00:00Z; add `modified: 2026-05-14`

11. **F-PASS2-MED-006 — error-taxonomy status: draft (product-owner):**
    - File: `.factory/specs/prd-supplements/error-taxonomy.md`
    - Action: Update `status: draft` → `status: active`

### LOW fix (story-writer — intent clarification first)

12. **F-PASS2-LOW-001 — Story frontmatter `updated:` vs `modified:` (story-writer):**
    - File: `.factory/stories/S-PLUGIN-PREREQ-D.md`
    - Action: Verify intent; if drift, rename `updated:` → `modified:`

### impl-pass-3 dispatch prerequisites

After fix-burst-impl-2 closes all 12 findings:
1. Verify `PrismCommand::Start` calls `run_boot_sequence` or equivalent
2. Verify all 5 Component Model callbacks delegate to `host_*` production functions
3. Verify `test_wasi_not_linked` has no early escape hatch
4. Verify BC-2.16.002 prose intro updated to v1.15/31 rows
5. Verify story pins updated to v1.15
6. Verify error-taxonomy status and timestamps corrected
7. Target: streak advance 0/3 → 1/3 (first clean pass of impl cascade)

**Trajectory:** impl-pass-1 (18 findings) → impl-pass-2 (12 findings) = -6 net. Decreasing trajectory is a convergence signal. The 2 paper-fix recurrences (CRIT-001/002) are structural problems requiring real fixes, not cosmetic; expect total to decrease further after fix-burst-impl-2.

---

_Report persisted by state-manager at D-549. Not written by the adversary (read-only tool profile); reified from orchestrator's conversation capture._
