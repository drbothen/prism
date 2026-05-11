---
document_type: adversarial-review
level: LOCAL
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T19:45:00Z
phase: 3
inputs: []
input-hash: "cb2e499"
traces_to: prd.md
pass: 2
previous_review: S-PLUGIN-PREREQ-B-pass-1.md
target_artifact: S-PLUGIN-PREREQ-B
target_sha: 7511e749
base_sha: 90d7c80f
verdict: BLOCKED-hard
streak: 0/3
finding_summary: { critical: 0, high: 2, medium: 3, low: 3, obs: 2 }
prior_passes: pass-1 BLOCKED-hard 20 findings; fix-burst-1 closed 12 actionable + 2 TDs + BC-2.16.002 v1.4
---

# Adversarial Review: S-PLUGIN-PREREQ-B (Pass 2)

**Verdict:** BLOCKED-hard
**Target SHA:** 7511e749
**Base SHA:** 90d7c80f (PREREQ-A merge point)
**Finding summary:** 0 CRITICAL / 2 HIGH / 3 MEDIUM / 3 LOW / 2 OBS
**Streak:** 0/3 (HIGH findings present — cannot advance)

## Finding ID Convention

Finding IDs use the format: `F-LP<PASS>-<SEV>-<SEQ>` for LOCAL-pass reviews in this cascade.

- `F`: Fixed prefix
- `LP<PASS>`: LOCAL pass number (e.g., `LP2`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`, `OBS`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

Examples: `F-LP2-HIGH-001`, `F-LP2-MED-003`

## Part A — Pass-1 Closure Verification

Fix-burst-1 claimed closure of 12 actionable findings. This section verifies each claim.

| Finding | Fix-burst-1 Claim | Verification Result | Notes |
|---------|-------------------|---------------------|-------|
| F-LP1-CRIT-001 (body_template not interpolated + Content-Type hardcoded) | Closed — interpolation engine implemented; Content-Type derivation added | CLOSED | Interpolation engine verified at engine call sites; Content-Type checks `{` prefix at step body string |
| F-LP1-CRIT-002 (cursor not percent-encoded) | Closed — percent_encode call added to cursor token before URL construction | CLOSED | percent_encode call visible in cursor path; URL assembly confirmed |
| F-LP1-CRIT-003 (intermediate-step records leak into PipelineResult) | Closed — accumulation logic revised to emit only final-step records | CLOSED | Record collection guarded by is_final_step / terminal-step flag |
| F-LP1-CRIT-004 (crowdstrike test asserts nothing + live network calls) | Closed — converted to wiremock mock server; assertions added | CLOSED | wiremock server pattern confirmed; real HTTP calls eliminated |
| F-LP1-HIGH-001 (AC-5 audit-log absent) | Closed — audit_log entries emitted per step | CLOSED | Audit emission call sites confirmed at step completion points |
| F-LP1-HIGH-002 (AC-6 fan-out entirely missing — zero call sites) | Closed — fan_out_batches called from execute path | CLOSED-STRUCTURAL-BUG — elevates to F-LP2-HIGH-001 (see Part B) | Batch context key mismatch: batches stored under step-name key but template references prior-step key; test counts requests only, not query-string content — false-green |
| F-LP1-HIGH-003 (rate-limit guard absent — AC-7 uncovered) | Closed — rate-limit guard added with token-bucket | CLOSED | Rate-limit guard confirmed; AC-7 Red Gate test present |
| F-LP1-HIGH-004 (truncation sentinel not emitted — AC-8 uncovered) | Closed — truncation guard emits sentinel record | CLOSED | Truncation sentinel emission confirmed; AC-8 Red Gate test present |
| F-LP1-HIGH-006 (is_first_pipeline_request not hoisted to call site — per-step re-detection) | Closed — hoisted to pipeline entry | CLOSED-WITH-GAP | Adjacent to F-LP2-HIGH-002; cursor same-cursor guard also at first-request boundary |
| F-LP1-MED-001 (BC-2.16.002 missing fan-out postcondition) | Closed — product-owner amended BC v1.3→v1.4 | CLOSED | BC amendment at factory-artifacts c2e7b376 verified |
| F-LP1-MED-004 (audit log missing timestamps) | Closed — timestamps added to audit entries | CLOSED | |
| F-LP1-MED-005 (pipeline error type erases original cause) | Closed — error chain preserved via source wrapping | CLOSED | |
| F-LP1-HIGH-005 (page_size not sent on first cursor request) | Deferred as TD-S-PLUGIN-PREREQ-B-001 P2 (PREREQ-C scope) | DEFERRED-AS-TD | Confirmed in tech-debt-register.md |
| F-LP1-MED-002 (AuthToken zeroize on Drop) | Deferred as TD-S-PLUGIN-PREREQ-B-002 P3 (PREREQ-D scope) | DEFERRED-AS-TD | Confirmed in tech-debt-register.md |
| F-LP1-LOW-001 (fan_out_batches never exercised by any test) | Acknowledged via Red Gate expansion | ACKNOWLEDGED | 7 new Red Gate tests added (wiremock conversion + fan-out path) |
| F-LP1-LOW-002 (step naming collision not detected at spec load) | Acknowledged non-blocking per fix-burst-1 disposition | ACKNOWLEDGED | |
| F-LP1-OBS-001 (batch parameter ordering in URL not deterministic) | Acknowledged non-blocking | ACKNOWLEDGED | |
| F-LP1-OBS-002 (template parse errors swallowed) | Acknowledged non-blocking — follow-up story scope | ACKNOWLEDGED/SKIPPED | |

**12 CLOSED + 1 CLOSED-STRUCTURAL-BUG (elevates to F-LP2-HIGH-001) + 2 DEFERRED-AS-TD + 4 ACKNOWLEDGED/SKIPPED + 1 CLOSED-WITH-GAP (adjacent to F-LP2-HIGH-002)**

---

## Part B — New Findings

### HIGH

#### F-LP2-HIGH-001: Fan-out batch values never reach HTTP URL (REGRESSION introduced by fix-burst-1)

- **Severity:** HIGH
- **Category:** spec-fidelity / coverage-gap
- **Location:** execute path — fan_out_batches call site + template context map construction
- **Description:** Fix-burst-1 wired the call to `fan_out_batches` but the resulting batch values are stored under the `{step.name}.batch` key in the template context map. The body template uses `${step1.ids}` — the prior step's full-array key — not the per-batch-slice key. The interpolation engine resolves `${step1.ids}` to the complete batch array (same as before fan-out), so every fan-out iteration sends the full list rather than its slice. AC-6 fan-out postcondition (BC-2.16.002 v1.4) is unmet.
- **Evidence:** The wiremock test introduced by fix-burst-1 asserts that three HTTP requests are made (one per batch), which passes because the fan-out loop executes three iterations. However, the test does not assert on the query string content of each request — only counts calls. All three requests carry the same full-array value; the test cannot distinguish this from correct behavior. Test is false-green.
- **Proposed Fix:** Store fan-out batch slices under a key matching the template (e.g., `{step.name}.ids` or update the template convention to `{step.name}.batch`). Add wiremock request matchers that assert distinct query parameter values across the three calls. Red Gate test must be upgraded to fail if all three requests carry identical query strings.

#### F-LP2-HIGH-002: Cursor infinite-loop if API returns same non-empty cursor (latent gap missed by pass-1)

- **Severity:** HIGH
- **Category:** missing-edge-cases / security-surface
- **Location:** cursor-pagination loop — no same-cursor guard, no MAX_PAGES sentinel
- **Description:** The cursor-pagination loop has no same-cursor guard and no MAX_PAGES sentinel. If an API implementation returns a non-empty cursor token identical to the previous cursor token (a known failure mode in several sensor APIs), the loop spins indefinitely, consuming memory as records accumulate and never emitting results. Additionally, the 10,000-record truncation guard applies only to the final step's accumulator; intermediate fan-out steps iterate without any bound, so a misbehaving cursor in a fan-out step causes unbounded growth before the final-step guard can fire.
- **Evidence:** Cursor loop body: compare new cursor against previous cursor before continuing — no such comparison present. MAX_PAGES constant: absent. Fan-out step pagination bound: absent.
- **Proposed Fix:** (1) Before appending cursor-page results, compare new cursor to previous cursor; if equal, break and emit a structured warning to the audit log. (2) Add a MAX_PAGES constant (default: 1000) enforced on every paginating step including intermediate fan-out steps.

### MEDIUM

#### F-LP2-MED-001: red_gate_tests: 16 in story v1.1 frontmatter vs actual 26 by grep (PG-LP7-002 discipline drift; 2nd occurrence)

- **Severity:** MEDIUM
- **Category:** coverage-gap
- **Location:** story S-PLUGIN-PREREQ-B v1.1 frontmatter — `red_gate_tests: 16`
- **Description:** Story frontmatter field `red_gate_tests: 16` was set during fix-burst-1 after adding 7 new + 1 upgraded tests. However, a grep of the test corpus shows 26 tests matching the Red Gate naming pattern for this story. The frontmatter count was updated by arithmetic ("8 + 7 + 1 = 16") rather than by grep re-count. This is the second occurrence of this discipline gap (first: S-PLUGIN-PREREQ-A pass-7). PG-LP7-002 requires the count to be verified by grep, not derived by arithmetic.
- **Evidence:** `grep -c 'fn test_BC_2_16' crates/prism-spec-engine/tests/` returns 26; story frontmatter says 16.
- **Proposed Fix:** grep for the Red Gate test pattern, count actual tests, update `red_gate_tests:` frontmatter to the verified count. OBS-LP2-002 proposes a process-gap mitigation.

#### F-LP2-MED-002: Content-Type derivation only checks `{` prefix; misclassifies JSON arrays as form-urlencoded

- **Severity:** MEDIUM
- **Category:** interface-gaps / spec-fidelity
- **Location:** Content-Type derivation function
- **Description:** The Content-Type derivation added in fix-burst-1 inspects the body string and returns `application/json` if the body starts with `{`. This correctly handles JSON objects but misclassifies JSON arrays (`[...]`) as `application/x-www-form-urlencoded`. Sensor APIs that use array bodies (e.g., batch-delete endpoints that POST `["id1","id2"]`) will receive a wrong Content-Type and likely reject the request.
- **Evidence:** Content-Type check: `if body.trim_start().starts_with('{')` — no `[` check present.
- **Proposed Fix:** Check for both `{` and `[` as JSON-body indicators, or attempt `serde_json::from_str::<serde_json::Value>` as a prefix probe.

#### F-LP2-MED-003: Numeric / non-string cursors silently terminate pagination (SOUL.md #4 silent-failure)

- **Severity:** MEDIUM
- **Category:** missing-edge-cases
- **Location:** cursor extraction from API response
- **Description:** When the API response contains a cursor field that is numeric (e.g., `"next_cursor": 42`) or a JSON boolean, the cursor extraction code attempts to read it as a string. On type mismatch it returns `None`, which the pagination loop interprets as "no more pages" and terminates cleanly. No warning is emitted to the audit log, no error returned, no indication that pagination was silently truncated. Several sensor APIs (including some Armis endpoints) return integer page tokens — this gap causes queries against those sensors to return only the first page without any signal that results are incomplete.
- **Evidence:** Cursor extraction: `.as_str().map(|s| s.to_string())` — integer cursor returns `None`; loop exits with no diagnostic.
- **Proposed Fix:** When cursor extraction encounters a non-string JSON type, emit a structured warning to the audit log with the raw JSON value, and convert numeric cursors to their string representation to allow pagination to continue.

### LOW

#### F-LP2-LOW-001: AuthAcquisitionFailed variant has no construction site (dead code or future-impl scaffolding)

- **Severity:** LOW
- **Category:** code-quality
- **Location:** `PluginError::AuthAcquisitionFailed` enum variant declaration
- **Description:** The `PluginError::AuthAcquisitionFailed` enum variant is declared but never constructed anywhere in the codebase. It appears to be placeholder scaffolding for a future credential-acquisition path, but there is no TD citation or doc-comment explaining its intent.
- **Evidence:** `grep -r 'AuthAcquisitionFailed' crates/` returns only the declaration and Display impl — zero construction sites.
- **Proposed Fix:** Either add a TD comment citing the story/TD where this will be wired, or remove the variant if it was accidentally left from an earlier design.

#### F-LP2-LOW-002: extract_at_path missing RFC 6901 `~` escape sequences + missing inline TD comment

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** `extract_at_path` function
- **Description:** The `extract_at_path` function implements a subset of JSON Pointer (RFC 6901) but does not handle the two-character escape sequences: `~0` must decode to `~` and `~1` must decode to `/`. Without this, any step that references a field named `foo~bar` or uses escaped pointer syntax receives `None` silently. Additionally, a TODO comment at the extraction site references this gap but does not cite a TD register entry, violating the TD citation discipline established in PG-LP5-001.
- **Evidence:** Path segment processing: no `replace("~1", "/").replace("~0", "~")` call present. TODO comment present but no `TD-S-PLUGIN-PREREQ-B-NNN` citation.
- **Proposed Fix:** Add `~1` → `/` and `~0` → `~` decoding to each path segment before lookup. File TD-S-PLUGIN-PREREQ-B-003 covering the RFC 6901 escaping gap and update the TODO comment to cite it.

#### F-LP2-LOW-003: percent_encoding `use` inside closure body rather than module top-level

- **Severity:** LOW
- **Category:** code-quality
- **Location:** cursor-encoding closure — `use percent_encoding::{percent_encode, NON_ALPHANUMERIC};`
- **Description:** The `use percent_encoding::{percent_encode, NON_ALPHANUMERIC};` import added by fix-burst-1 is placed inside a closure body rather than at the module's `use` block at the top of the file. Rust allows this but it is non-idiomatic and creates reader confusion about whether the import is conditional or guarded.
- **Evidence:** Import statement located inside closure braces, not at module level.
- **Proposed Fix:** Move the `use` statement to the module-level import block.

### OBS

#### OBS-LP2-001: Fan-out + truncation guard latent resource bound (compound with F-LP2-HIGH-002)

- **Severity:** OBS (non-blocking)
- **Category:** missing-edge-cases / observability
- **Description:** Compound with F-LP2-HIGH-002: when MAX_PAGES is added, the fan-out loop should also emit a pagination-cap-reached signal to the audit log so operators can distinguish "query returned N records" from "query was truncated at the pagination cap." Without this signal, silent truncation is operationally invisible. Non-blocking because the primary guard (MAX_PAGES) is tracked as HIGH-002. This OBS captures the observability complement.

#### OBS-LP2-002: [process-gap] red_gate_tests count discipline needs automated check

- **Severity:** OBS (process-gap candidate)
- **Category:** coverage-gap
- **Description:** F-LP2-MED-001 is the second occurrence of the Red Gate count drift pattern. Root cause: frontmatter counts updated by arithmetic rather than grep. PG-LP7-002 already mandates exact grep verification but is not enforced mechanically. Recommended mitigation: add a story-consistency linter that greps the actual Red Gate test count and compares it to `red_gate_tests:` frontmatter. This should run as a pre-commit hook or as part of the state-manager burst recording fix-burst completion. Track as a process-gap candidate for VSDD TD registration.

---

## KUDOs

1. **Closure verification rigor (fix-burst-1):** The implementer closed 4 CRIT findings in a single burst with individually verified evidence — each critical closure had a specific code location cited.
2. **is_first_pipeline_request hoist (F-LP1-HIGH-006 closure):** Hoisting first-request detection to the pipeline entry boundary eliminates per-step re-detection and makes the page_size guard compositionally correct.
3. **AuthToken Debug redaction:** `AuthToken` implements `Debug` with a redacted placeholder rather than exposing credential content — SOUL.md #2 applied proactively without an explicit finding requiring it.
4. **JSON Pointer adoption (extract_at_path):** Adopting RFC 6901 JSON Pointer syntax for nested field extraction is the correct long-term decision; the foundation is right even though the escape-sequence implementation is incomplete.
5. **wiremock conversion (CRIT-004):** Converting the CrowdStrike live-network test to a wiremock mock server eliminates a CI flakiness source and an external-dependency violation in a single fix.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 3 |
| LOW | 3 |
| OBS | 2 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate (streak 0/3; HIGH findings block advancement)
**Readiness:** requires revision — fix-burst-2 dispatching; pass-3 target streak 1/3

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 2 |
| **New findings** | 10 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (10 / (10 + 0)) |
| **Median severity** | MEDIUM |
| **Trajectory** | 20→10 |
| **Verdict** | FINDINGS_REMAIN |
