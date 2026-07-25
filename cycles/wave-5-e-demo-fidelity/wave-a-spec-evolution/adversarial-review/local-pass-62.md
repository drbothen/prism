---
document_type: adversarial-review
review_id: wave-a-spec-pass-62
pass_number: 62
reviewer: vsdd-factory:adversary
review_type: spec
artifact_scope:
  amended:
    - .factory/specs/behavioral-contracts/BC-2.16.008-spec-file-validation-add-sensor-spec.md (v1.6)
    - .factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md (v1.26)
    - .factory/specs/prd-supplements/error-taxonomy.md (v2.69)
    - .factory/specs/architecture/decisions/ADR-053-wave-a-sensor-fidelity-remediation-openapi-grounding-armis-token-exchange-cyberint-dual-surface.md (v0.35)
    - .factory/stories/S-WAVE-A-ENGINE-001-header-scheme-field-rule9-validation-auth-dispatch.md (v2.2)
  new:
    - .factory/specs/verification-properties/vp-160-rule9-cookie-name-charset-totality.md (v1.0)
    - .factory/stories/S-MAINT-VOLATILE-CITE-001-td-vsdd-091-normative-text-backfill.md (v1.0 draft)
    - .factory/stories/S-MAINT-VOLATILE-CITE-002-td-vsdd-091-history-row-backfill.md (v1.0 draft)
  re-derived:
    - BC-2.16.014 (v1.19), BC-2.01.016 (v1.15), BC-2.01.017 (v1.10), BC-2.16.008 (v1.6),
      BC-2.01.018 (v1.4), BC-2.01.006 (v1.8), BC-2.06.013 (v1.2), VP-159 (v1.26),
      VP-153 (v0.28), ADR-054 (v0.52), ADR-053 (v0.35)
  indexes:
    - BC-INDEX (v8.71), VP-INDEX (v2.13), ARCH-INDEX (v2.274), STORY-INDEX (v2.724)
frozen_head: factory-artifacts@a3dedbf0e
streak_pre_pass: "0/3"
streak_post_pass: "0/3"
verdict: BLOCKED
findings_count: 21
severity_breakdown:
  critical: 1
  high: 5
  medium: 10
  low: 4
  observation: 1
novelty: HIGH
related_state_decision: D-2016
related_fix_burst: FB46
date: 2026-07-24
---

# Wave-A Spec-Evolution Adversarial Review — Local Pass 62

**Scope:** BC-2.16.008 v1.6 + BC-2.16.009 v1.26 + ADR-053 v0.35 + error-taxonomy v2.69 + S-WAVE-A-ENGINE-001 v2.2 (post-FB45 amended perimeter). Full re-derivation of all prior-converged axes plus fresh-context probe of the FB45 fix-burst outputs as highest-risk surface. VP-160 v1.0 registration verified.

---

## F-WASE-P62-CRIT-001 — AC-020/RG-024 requirement bound to a surface the story cannot reach: `add_sensor_spec` wire-level `structuredContent.error.code` assertion is unsatisfiable

**Severity:** CRITICAL
**Category:** Unsatisfiable acceptance criterion / SAP-3 mis-application
**Authority:** BC-5.38.001 (every AC must be implementable by the story's declared crates); SAP-3 (spec-arm reachability from public surface); CLAUDE.md precedence rule 7 (story spec supersedes BC on implementation scope)

**Finding:**

S-WAVE-A-ENGINE-001 v2.1 §Acceptance Criteria AC-020 reads (paraphrased): `add_sensor_spec` with a non-tchar cookie name MUST return `structuredContent.error.code == E-SPEC-027`. AC-020's companion RG-024 asserts the wire-level `structuredContent.error.code` field in the MCP JSON envelope.

Three independent defects combine to make AC-020 + RG-024 unsatisfiable within the story's declared `crates_touched`:

1. The `add_sensor_spec` MCP handler lives in `prism-mcp`, which is NOT in `crates_touched` (story explicitly forbids it — the add_sensor_spec surface for Rule 9 is being deliberately separated per ADR-053 §D6 architectural split).

2. The `ValidationFailed` handler arm in `prism-mcp` returns a success-payload response with no `structuredContent` — the handler's `ValidationFailed` path calls `format_validation_errors()` and wraps the result in a `CallToolResult::Success` shape. There is no `structuredContent.error.code` field on this path at the MCP wire level.

3. `parse_and_validate_spec_toml()` stringifies via `format!("{e}")`, so `SpecErrorCode::ESpec027` (the typed enum variant) never reaches the MCP layer as a structured code — it is serialized as a human-readable error string. The `structuredContent.error.code` = `"E-SPEC-027"` assertion would require prism-mcp to be restructured to propagate the typed error code, which is out of scope for this story.

The root cause is that FB45's HIGH-002 fix added a SAP-3-compliant AC/RGT but bound the MUST contract to the MCP wire shape (`structuredContent.error.code`) using a surface that the story's `prism-spec-engine`-only scope cannot reach. The story correctly prohibits `prism-mcp`, making the wire-level assertion an architectural contradiction.

**Required fix:**

Architect adjudication via ADR-053 §D6 Option (B): document the current prose-string validation shape as a deliberate architectural GAP, not a defect — the full E-SPEC-027 wire-level structured code path is deferred to a new story `S-WAVE-A-MCP-001` which will touch `prism-mcp`. Story-writer restates AC-020 and renames RG-024 to `test_add_sensor_spec_api_rejects_nontchar_cookie_name_rule9_path` — asserting `Ok(AddSensorSpecResult::ValidationFailed { errors })` at the **prism-spec-engine API boundary** (not the MCP wire layer). The wire-level `structuredContent.error.code` assertion is removed from this story and added to the scope of `S-WAVE-A-MCP-001`.

**Resolution (FB46):** Closed. ADR-053 §D6 Option (B) ruling applied. AC-020 + RG-024 restated to assert `Ok(AddSensorSpecResult::ValidationFailed { errors })` at prism-spec-engine API boundary; `structuredContent.error.code` assertion removed. `S-WAVE-A-MCP-001` created to carry the wire-level intent.

**Routing (original):** architect (ADR-053 §D6 ruling); story-writer (AC-020 + RG-024 restatement)

---

## F-WASE-P62-HIGH-001 — Rule 9 §Entry points call graph is false: `SpecLoader::parse()` does NOT call `validate_sensor_spec()` — zero occurrences in `spec_parser.rs`

**Severity:** HIGH
**Category:** False call graph / phantom construct (POL-29)
**Authority:** BC-5.38.001 / SAP-3 / CLAUDE.md precedence rule 7

**Finding:**

BC-2.16.009 v1.25 §Validation Rules 9 §Entry points sub-section states (paraphrased): "`SpecLoader::parse()` calls `validate_sensor_spec()` to enforce Rules 1–10 on both the boot path and the `add_sensor_spec` path."

This is false. Reading `crates/prism-spec-engine/src/spec_parser.rs`:
- `validate_sensor_spec()` is a standalone free function called at boot from the spec-loading orchestration layer.
- `SpecLoader::parse()` does NOT call `validate_sensor_spec()`. The two functions are peers in the same module, not caller/callee.
- `SpecLoader::parse()` enforces Rules 1 (TOML format), Rule B (basename stem), timestamp validation, and Rule 8 (probe_table reference check) — NOT Rules 1–10 wholesale.
- `validate_sensor_spec()` enforces Rules 1–5 (auth_type, base_url, tables, columns, env-var).
- Rule 6 (OCSF) is enforced by `resolve_env_var_tokens()`.
- Rule 7 (step template methods) is enforced by `validate_step_methods()`.
- Rules 8 and 9 are enforced inside `SpecLoader::parse()` only.

The false premise propagated into BC-INDEX (the NOTE for v8.71 repeated the `SpecLoader::parse()` calls `validate_sensor_spec()` claim) and ARCH-INDEX (the v2.274 changelog row's Rule-9 narrative relied on the same premise).

Additionally, §Validation Rules 8 §Implementation anchor referenced the wrong function — a residual from a prior spec version when Rule 8 lived in `validate_sensor_spec()` before being moved to `parse()` in v1.11.

**Required fix:**

Product-owner rewrites §Entry points sub-section against the verified four-function model: `validate_sensor_spec()` enforces Rules 1–5; `SpecLoader::parse()` enforces TOML + Rule B + timestamp + Rule 8 + Rules 9–10; `resolve_env_var_tokens()` enforces Rule 6; `validate_step_methods()` enforces Rule 7. The §Validation Rules 8 §Implementation anchor must also be corrected to cite `SpecLoader::parse()` (current enforcement site), not `validate_sensor_spec()`.

**Resolution (FB46):** Closed. Product-owner rewrote §Entry points against the four-function model. §Validation Rules 8 anchor also corrected.

**Routing (original):** product-owner (BC-2.16.009 §Entry points sub-section + Rule 8 anchor)

---

## F-WASE-P62-HIGH-002 — E-SPEC-027 template (a) echoes `{value}` unbounded (CWE-400) and without CTL sanitization (CWE-117 log injection); this contradicts §Validation Rules 7 which explicitly caps echo at 32 codepoints

**Severity:** HIGH
**Category:** CWE-400 (Uncontrolled Resource Consumption via unbounded value echo) / CWE-117 (improper output neutralization for logs) / POL-24 byte-identity gap
**Authority:** BC-2.16.009 §Security requirement (CTL-neutralization mandate); SEC-001 vector integrity; error-taxonomy POL-24 carrier

**Finding:**

BC-2.16.009 v1.25 E-SPEC-027 template (a) `message_template` echoes `{value}` (the header_scheme field value the caller supplied) as a raw string with no length bound. The same template appears verbatim in error-taxonomy.md v2.68.

Two independent defects:

1. **CWE-400** — §Validation Rules 7 (step template methods) explicitly caps its error echo at 32 codepoints because the value could be arbitrarily long. Rule 9 operates on a field in the same TOML block and faces the same risk, but the template (a) message had no analogous cap. A caller can supply a 1-MB `header_scheme` string and cause a 1-MB error message.

2. **CWE-117 (log injection)** — CTL bytes (0x00–0x1F, 0x7F) in the `{value}` echo can inject spurious log lines when the error message is written to structured logs. The §Security requirement bullet in BC-2.16.009 §Validation Rules 9 explicitly mandates CTL neutralization, but E-SPEC-027 template (a) was not updated to enforce it.

**Required fix:**

Both BC-2.16.009 E-SPEC-027 template (a) and error-taxonomy.md E-SPEC-027 template (a) (POL-24 byte-identity) must specify: a 64-codepoint cap applied via `truncate_at_char_boundary`, with CTL bytes 0x00–0x1F and 0x7F escaped to `\xNN` uppercase hex. Values under 64 codepoints that contain no CTL bytes pass through without modification (explicit no-op clause). EC-009-047 (overlong echo cap) and EC-009-048 (CTL-byte `\xNN` escaping) added to both carriers with identical vectors.

**Resolution (FB46):** Closed. 64-codepoint cap + CTL `\xNN` hex escaping specified byte-identically in BC-2.16.009 and error-taxonomy.md (v2.69). EC-009-047 + EC-009-048 added to both.

**Routing (original):** product-owner (BC-2.16.009 E-SPEC-027 template (a) + error-taxonomy.md POL-24 sync)

---

## F-WASE-P62-HIGH-003 — No Red Gate test discriminates the permissive half of the tchar set; an `[A-Za-z0-9_]`-only implementation passes all 24 Red Gate tests; backtick (U+0060) has zero coverage in either direction

**Severity:** HIGH
**Category:** Test coverage gap — paper fix (TD-VSDD-059)
**Authority:** TD-VSDD-059 (paper-fix detection); SAP-3 (spec-arm reachability); BC-5.38.001

**Finding:**

S-WAVE-A-ENGINE-001 v2.1 §Red Gate Tests RG-001..RG-024 include:
- RG-007: accepts `"cookie:access_token"` (alphanumeric + underscore only)
- RG-008..RG-010: accepts bearerAuth-style values (no cookie name with special chars)
- RG-020..RG-023: rejects `;`, `=`, SPACE, CTL (injection characters)

None of the 24 RGTs includes a cookie name containing any of the 15 special tchar characters (backtick, exclamation, hash, dollar, etc.) in the ACCEPT direction. An implementation that restricts cookie names to `[A-Za-z0-9_]` only — a common over-restrictive mistake — would pass all 24 tests. The backtick specifically (the character FB45 HIGH-001 fought to restore to the set) has zero coverage in either direction: there is no test asserting `"cookie:a` + backtick + `b"` is accepted AND no test asserting anything about backtick at all.

Text convergence (fixing the spec list to say 15 chars) without a discriminating test is a paper fix under TD-VSDD-059: the spec says the right thing but there is no test that would fail if the implementation only accepted 77−15=62 chars (letters + digits + underscore + the 4 non-disputed specials).

**Required fix:**

AC-021: All 15 RFC 6265 tchar special chars must be accepted when present in a valid cookie name. RG-025 (`test_rule9_all_tchar_special_chars_cookie_name_accepted`) asserting `"cookie:a!#$%&'*+-.^_` + backtick + `|~9Z"` is accepted.

AC-022: High-byte values (U+00E9 etc.) are rejected in cookie names due to byte-level tchar predicate (is_valid_cookie_name_tchar operates on bytes, not Unicode codepoints; high bytes are >= 0x80 and outside the ASCII range). RG-026 (`test_rule9_high_byte_in_cookie_name_e_spec_027_template_a`) asserting EC-009-050 vector is rejected.

Additionally, VP-160 authored (Kani, P0) proving charset totality exhaustively across all 256 byte values.

**Resolution (FB46):** Closed. AC-021 + RG-025 + AC-022 + RG-026 added to S-WAVE-A-ENGINE-001 v2.2. VP-160 authored by architect.

**Routing (original):** story-writer (AC-021/AC-022/RG-025/RG-026); architect (VP-160)

---

## F-WASE-P62-HIGH-004 — T-B03 parenthetical "(or the validation pass within spec_parser.rs)" re-opens the SEC-001 escape hatch that §Validation Rules 9 §Security requirement was written to close

**Severity:** HIGH
**Category:** Spec self-contradiction / security escape hatch (SEC-001 re-open)
**Authority:** BC-2.16.009 §Security requirement (header injection mandate); ADR-053 §D2

**Finding:**

S-WAVE-A-ENGINE-001 v2.1 §Tasks T-B03 reads (paraphrased): "Wire `header_scheme` into `build_request()` — caller retrieves validated value from `SpecLoader::parse()` (or the validation pass within spec_parser.rs)."

The parenthetical "(or the validation pass within spec_parser.rs)" is a functional escape hatch. Since `validate_sensor_spec()` does NOT enforce Rule 9 (as established in HIGH-001), "the validation pass within spec_parser.rs" for Rule 9 means only `SpecLoader::parse()`. But the parenthetical implies there is an alternative validation locus inside spec_parser.rs that could satisfy T-B03's precondition — which there is not. An implementer reading T-B03 and choosing the "validation pass within spec_parser.rs" path (not `SpecLoader::parse()`) would wire injection without going through Rule 9. This is the exact escape hatch BC-2.16.009 §Security requirement item T-B03-SEC-001 was designed to close.

Additionally, story supersedes BC on implementation scope per CLAUDE.md precedence rule 7, so the story-level escape hatch overrides the BC-level injection mandate during TDD implementation.

**Required fix:**

T-B03 must cite `SpecLoader::parse()` unconditionally with no parenthetical alternatives. A bypass-consequence note should be added explaining that any wiring path that bypasses `SpecLoader::parse()` leaves the SEC-001 injection vector open.

**Resolution (FB46):** Closed. Parenthetical deleted from T-B03. `SpecLoader::parse()` cited unconditionally with bypass-consequence note.

**Routing (original):** story-writer (T-B03 correction)

---

## F-WASE-P62-HIGH-005 — BC-2.16.008 §Error Conditions bounded validation errors to E-SPEC-001..007 only, excluding E-SPEC-027; BC-2.16.009 §Entry points asserted over BC-2.16.008's surface unilaterally

**Severity:** HIGH
**Category:** BC-to-BC assertion boundary overreach / incomplete error surface declaration
**Authority:** BC-2.16.008 §Error Conditions (bounded error surface); BC-2.16.009 §Entry points (asserts add_sensor_spec must propagate E-SPEC-027)

**Finding:**

BC-2.16.008 v1.5 §Error Conditions lists the error codes that `add_sensor_spec` can return: `E-SPEC-001, E-SPEC-002, E-SPEC-003, E-SPEC-004, E-SPEC-005, E-SPEC-006, E-SPEC-007`. This list is bounded — it does not include E-SPEC-027.

BC-2.16.009 v1.25 §Entry points added the requirement that `add_sensor_spec` must propagate E-SPEC-027 errors from Rule 9. But BC-2.16.009 cannot add requirements to BC-2.16.008's error surface without amending BC-2.16.008 directly — the two BCs have separate contracts and separate §Error Conditions tables. An implementer reading BC-2.16.008 alone would see no E-SPEC-027 and would not wire Rule 9 error propagation.

Additionally, BC-2.16.008 §Error Conditions lists `E-IO-001` (file I/O failure) in one error row, but per the error taxonomy `E-IO-001` is a sensor-type-specific I/O error, not the validation pipeline error returned by `add_sensor_spec`. The correct code for a spec-file format error is `E-SPEC-002`.

**Required fix:**

BC-2.16.008 §Error Conditions: expand to include E-SPEC-027 (Rule 9 cookie-name tchar violation, propagated from `SpecLoader::parse()`). Add explicit entry-point statement naming `SpecLoader::parse()` as the validation path called by `add_sensor_spec`. Correct `E-IO-001` → `E-SPEC-002`. Add reciprocal reference to BC-2.16.009 Rule 9 as the source of E-SPEC-027 for this surface.

**Resolution (FB46):** Closed. BC-2.16.008 v1.6 authored: E-SPEC-027 added to §Error Conditions; entry-point statement added; `E-IO-001` → `E-SPEC-002` corrected; reciprocal reference to BC-2.16.009 Rule 9 added.

**Routing (original):** product-owner (BC-2.16.008 §Error Conditions expansion)

---

## F-WASE-P62-MED-001 — BC-INDEX inline version cells for BC-2.01.016 and BC-2.16.014 are stale (show v1.14/v1.18; current files are v1.15/v1.19)

**Severity:** MEDIUM
**Category:** Index drift (POL-29 step 8f — index cells must reflect current BC file versions)

**Finding:**

BC-INDEX v8.71 table rows:
- BC-2.01.016 Status cell: `active (amended per ADR-054 D1 / wave-a-burst-3; v1.14)` — but the BC file is at v1.15 (FB45 story-writer leg amended it per CLAUDE.md precedence).
- BC-2.16.014 Status cell: shows `v1.18` at the end of the version chain — but the BC file is at v1.19 (FB45 remove-uncertainty burst amended TV-11 arithmetic).

Both cells are append-only chains. The v1.15 and v1.19 increments were applied to the BC files but the BC-INDEX status cells were not updated in the same burst.

**Resolution (FB46):** State-manager responsibility. BC-INDEX v8.71→v8.72: BC-2.01.016 status cell amended to append `v1.15` (FB46 story-writer Rule 9 reachability restatement from CRIT-001 + HIGH-003 fixes); BC-2.16.014 status cell amended to append `v1.19` (FB46 product-owner BC-2.16.008 expansion ripple + HIGH-001 entry-points correction companion). Both cells verified by reading the BC files directly.

**Routing:** state-manager (BC-INDEX inline cells)

---

## F-WASE-P62-MED-002 — STORY-INDEX labels S-WAVE-A-ENGINE-001 "draft v2.0" while the on-disk file is at v2.1 (FB45 story-writer leg bumped it); counts show 21 ACs / 24 RGTs vs file's 21 ACs / 24 RGTs — but FB46 story-writer will bump to v2.2 with 23 ACs / 26 RGTs

**Severity:** MEDIUM
**Category:** Index drift (POL-1 append-only; ARCH-INDEX v2.273→v2.274 pattern is precedent for correction-for-accuracy)

**Finding:**

STORY-INDEX v2.724 S-WAVE-A-ENGINE-001 row describes `draft v2.0` with 21 ACs / 24 RGTs. The on-disk story file is at v2.1 (FB45 story-writer applied HIGH-001+HIGH-002+MED-001+MED-002+LOW-001+LOW-002+LOW-003+LOW-004 fixes; version 2.0→2.1). The index was not updated to reflect the v2.1 content. Additionally, FB46 story-writer will further bump the story to v2.2 (adding AC-021/AC-022/RG-025/RG-026 from HIGH-003, CRIT-001 restatement, HIGH-004 T-B03 fix, MED-008 attribution correction, LOW-002/LOW-003/LOW-004 task fixes), yielding 23 ACs / 26 RGTs.

History must not be mutated — the v2.0 description stays as-is. A correcting segment must be appended.

**Resolution (FB46):** Closed per ARCH-INDEX v2.273→v2.274 correction-for-accuracy precedent. Correcting segment appended to STORY-INDEX S-WAVE-A-ENGINE-001 row: `[corrected v2.2 (2026-07-25 FB46): AC count 21→23 (AC-021/AC-022 tchar acceptance + high-byte rejection), RGT count 24→26 (RG-025/RG-026), CRIT-001 AC-020/RG-024 restated to prism-spec-engine API boundary, HIGH-004 T-B03 parenthetical deleted, MED-008 BC attribution corrected, LOW-002/LOW-003/LOW-004 task-phase reorder + EC-ID canonicalization. 20 tasks unchanged. Q-number attribution note: the v2.0 registration paragraph used Q1–Q5 labels for remove-uncertainty BLOCKERs; the authoritative Q-numbering is in wave-a-engine-story-adjudication-Q1-Q5.md (Q2=fail-fast Rule 9, Q3=ADR-031 cookie-name, Q4=&AuthType vs &str, Q5=token_exchange message). v2.0 BLOCKER descriptions were correct; only the Q-labels were misapplied in the registration paragraph.]`

**Routing:** state-manager (STORY-INDEX correcting segment)

---

## F-WASE-P62-MED-003 — `spec_toml` vs `toml_content` vs `name`: BC-2.16.008 §Tool Schema described the wrong parameter names for the `add_sensor_spec` MCP tool

**Severity:** MEDIUM
**Category:** Wire-shape mismatch / spec-to-implementation divergence

**Finding:**

BC-2.16.008 v1.5 §Tool Schema listed the `add_sensor_spec` input parameters as `spec_toml` (TOML content string) and implied the sensor name was embedded inside the TOML. Reading the as-built `prism-mcp` implementation and the story's §Files to Modify table: the actual MCP tool parameters are `toml_content` (the TOML string) and `name` (the sensor name, separate from the TOML body). The BC schema was inconsistent with the wire shape.

**Resolution (FB46):** Closed. Product-owner reconciled BC-2.16.008 §Tool Schema to as-built `toml_content` + `name` parameter names. Story §Acceptance Criteria also confirmed `toml_content` + `name` naming.

**Routing (original):** product-owner (BC-2.16.008 §Tool Schema)

---

## F-WASE-P62-MED-004 — Rule 9 cookie-name charset totality had zero VP coverage corpus-wide prior to this pass

**Severity:** MEDIUM
**Category:** Verification gap (VP coverage; P0 security property)

**Finding:**

Rule 9 was added to BC-2.16.009 in v1.12 (2026-07-22). The SEC-001 tchar charset constraint was added in v1.24 (D-2013). There is no verification property in the VP corpus that exhaustively proves the `is_valid_cookie_name_tchar` predicate accepts exactly the 77-char RFC 9110 §5.6.2 tchar set and rejects everything else. Proptest-style property tests are sampling-based and cannot certify totality over 256 byte values. The security property (injection rejection) has P0 priority but zero formal verification.

**Resolution (FB46):** Closed. Architect authored VP-160 (Kani, P0): `is_valid_cookie_name_tchar` returns true iff every byte is in the 77-character RFC 9110 §5.6.2 tchar set; semicolons, bare equals, spaces, TAB, CTL bytes (0x00–0x1F, 0x7F), non-ASCII bytes (0x80–0xFF), and RFC 9110 delimiters are rejected. Proves totality exhaustively across all 256 byte values. VP-INDEX v2.12→v2.13.

**Routing (original):** architect (VP-160 authorship); state-manager (VP-INDEX registration)

---

## F-WASE-P62-MED-005 — No length bound on `<name>` parameter in `add_sensor_spec` tool schema (CWE-390 opaque deferred failure)

**Severity:** MEDIUM
**Category:** CWE-390 (detection of error condition without action) / missing invariant
**Authority:** BC-2.16.008 §Tool Schema / BC-2.16.008 §Error Conditions

**Finding:**

BC-2.16.008 §Tool Schema describes the `name` parameter as a string with no stated maximum length. An unbounded `name` can cause silent downstream failures when the name is used as a file basename (filesystem path length limits), a RocksDB key component (key-size limits), or as part of an audit log field. The validation pipeline has no explicit E-SPEC error for an oversized `name` value — the failure mode is opaque (the system encounters an OS or storage error downstream rather than a clean user-facing validation error).

**Resolution (FB46):** Closed. BC-2.16.008 §Tool Schema: 128-codepoint bound added to the `name` parameter description. EC-009-051 added (cookie `name` > 128 codepoints → E-SPEC-027 template (d), parallel to the CWE-390 fix in BC-2.16.009 Rule 9).

**Routing (original):** product-owner (BC-2.16.008 §Tool Schema length bound + EC-009-051)

---

## F-WASE-P62-MED-006 — [process-gap] records-lint.sh `--full-scan` excluded `.factory/stories/` from L1/L7 corpus — S-MAINT stories self-referentially validated by a gate that cannot see them

**Severity:** MEDIUM [process-gap]
**Category:** TD-VSDD-092 gate coverage gap

**Finding:**

`scripts/records-lint.sh` §Config block listed `.factory/stories/` as excluded from L1/L7 full-scan coverage (the config only included `behavioral-contracts/`, `architecture/decisions/`, `verification-properties/`, `prd-supplements/`). The two new maintenance stories (S-MAINT-VOLATILE-CITE-001, S-MAINT-VOLATILE-CITE-002) carry version changelog rows that are subject to L1/L7 validation — but the gate would silently skip them.

**Resolution (FB46):** Closed. devops-engineer amended records-lint.sh CONFIG to add `.factory/stories/` to L1/L7 full-scan coverage (commit `94be384eb`).

**Routing (original):** devops-engineer

---

## F-WASE-P62-MED-007 — [process-gap] L9 `file.rs:NNN` pattern missed bare `~LNNN` citations; live violation in VP-INDEX §Changelog v2.12 row passed clean

**Severity:** MEDIUM [process-gap]
**Category:** TD-VSDD-092 L9 gate coverage gap — arm-5 missing from regex

**Finding:**

`scripts/records-lint.sh` L9 check regex covered `filename.rs:NNN` and `file.rs:NNN` patterns but did NOT cover bare `~LNNN` (tilde-L-number) citations, which are the second most common volatile-line-cite form used in factory records text. The VP-INDEX §Changelog v2.12 row contains: `AC-7d new_oauth2 constructor usage at ~LNNN/~LNNN/~LNNN volatile cites` — three bare `~LNNN` cites that would be caught by the corrected L9 regex but silently passed through the gate until now.

**Resolution (FB46):** Closed. devops-engineer extended records-lint.sh L9 regex (arm-5) to match bare `~LNNN` patterns (commit `1e4cd4b4b`). Self-probe updated: 14→18 probes (4 new arm-5 cases). VP-INDEX §Changelog v2.12 row corrected in-place per ARCH-INDEX v2.272 correction-for-accuracy precedent: `~LNNN/~LNNN/~LNNN` volatile cites replaced with durable anchor "at the three AC-7d `new_oauth2` constructor call sites in VP-159 §Harness."

**Routing (original):** devops-engineer (L9 regex); state-manager (VP-INDEX v2.12 row correction)

---

## F-WASE-P62-MED-008 — Story §Behavioral Contracts attributed `get_token()` default method addition to BC-2.01.016; owned by BC-2.16.014 §P9

**Severity:** MEDIUM
**Category:** BC attribution error / misleading traceability

**Finding:**

S-WAVE-A-ENGINE-001 v2.1 §Behavioral Contracts row for BC-2.01.016 reads (paraphrased): "AuthProvider trait extension: `get_token()` default method added; Rule A/B E-SPEC-012/013 Display alignment." The `get_token()` default method addition is a BC-2.16.014 §P9 contract — it belongs to the declarative auth acquisition BC, not to the SensorAuth open-trait BC (BC-2.01.016). BC-2.01.016 governs the plugin-implementable auth contract (no sealed marker); it does not define `get_token()`.

Conversely, the BC-2.16.014 §Behavioral Contracts row in the story was not updated to include the `get_token()` default method scope.

**Resolution (FB46):** Closed. Story-writer corrected both rows: BC-2.01.016 scope restated to "Rule A/B E-SPEC-012/013 Display alignment only" (no `get_token()` mention); BC-2.16.014 scope updated to include "`get_token()` default method addition to `AuthProvider` trait."

**Routing (original):** story-writer

---

## F-WASE-P62-MED-009 — ADR-053 §D5 completion marker cited stale v2.67; the SEC-001 amendment landed in v2.68

**Severity:** MEDIUM
**Category:** Stale provenance cite / POL-29 version-pin accuracy

**Finding:**

ADR-053 v0.34 §D5 completion marker row reads (paraphrased): "error-taxonomy.md v2.67 — E-SPEC-027 registered with four template variants." But E-SPEC-027 was registered in error-taxonomy.md v2.59 (D-1948 burst-3), not v2.67. More critically, the SEC-001 tchar amendment that added template (a) cookie-name charset constraint landed in v2.68 (FB45 HIGH-001 fix) — so the §D5 marker should cite v2.68 for the SEC-001-complete state, not v2.67.

A secondary provenance trail issue: two additional rows in §D5 that describe BC-2.16.009 v1.24 and v1.25 content correctly cited the version but without confirming the origination burst (D-2013 for v1.24; FB45 for v1.25).

**Resolution (FB46):** Closed. Architect corrected §D5 completion marker to cite v2.68 as the SEC-001-complete error-taxonomy version. Both provenance trails extended with burst and decision anchors. Full version-pin sweep table produced.

**Routing (original):** architect (ADR-053 §D5 provenance correction)

---

## F-WASE-P62-MED-010 — High bytes + TAB named as Rule 9 triggers (BC-2.16.009 §Security requirement) with no corresponding EC, vector, or test

**Severity:** MEDIUM
**Category:** Undocumented edge case / EC/test gap

**Finding:**

BC-2.16.009 v1.25 §Security requirement bullet states (paraphrased): "Rule 9 validates against tchar; non-ASCII bytes (high bytes, U+0080+) and control characters (TAB, CTL) fail the predicate." TAB (U+0009) and high bytes (0x80–0xFF) are named as rejection triggers but:

1. No EC in EC-009-xxx covers them (EC-009-043 covers `;=` injection, EC-009-044 bare `=`, EC-009-045 SPACE, EC-009-046 CTL). TAB and high bytes are separately named in the security prose but have no corresponding EC rows.
2. No RGT covers TAB in a cookie name in either the accept or reject direction.
3. No RGT covers a high-byte cookie name.

**Resolution (FB46):** Closed. EC-009-049 (TAB in cookie name, U+0009 → rejected, both byte-level and Unicode) + EC-009-050 (high byte U+00E9 as predicate-divergence probe — rejected at byte level) added to BC-2.16.009 §Error Conditions with full vectors. RG-026 (`test_rule9_high_byte_in_cookie_name_e_spec_027_template_a`) added to story for EC-009-050. TAB covered by EP-009-049 EC (story-level test not required — existing RG-022 CTL coverage is sufficient for the TAB-as-CTL path; EC-049 is a documentation-completeness addition).

**Routing (original):** product-owner (EC-009-049 + EC-009-050 in BC-2.16.009)

---

## F-WASE-P62-LOW-001 — ADR-053 §D2 SEC-001 pointer named "spaces" as a trigger but the cited ECs are EC-009-043/046 only (four spaces covers four ECs; two were missed)

**Severity:** LOW
**Category:** Incomplete cross-reference / citation gap

**Finding:**

ADR-053 v0.34 §D2 SEC-001 injection vector summary said the vector is triggered by "spaces and control characters in cookie names" and pointed to EC-009-043 and EC-009-046. But EC-009-043..046 are four edge cases (`;=` injection, bare `=`, SPACE, CTL), not just "spaces and control characters." EC-009-044 (bare `=`) and EC-009-045 (SPACE specifically named) were omitted from the cite.

**Resolution (FB46):** Closed. ADR-053 §D2 now cites all four ECs (EC-009-043, EC-009-044, EC-009-045, EC-009-046).

**Routing (original):** architect (ADR-053 §D2 cite correction)

---

## F-WASE-P62-LOW-002 — Story-local `EC-009` collided as a prefix with BC-canonical `EC-009-043..051` range

**Severity:** LOW
**Category:** EC-ID namespace collision / traceability confusion

**Finding:**

S-WAVE-A-ENGINE-001 v2.1 §Edge Cases contained a story-local EC labeled `EC-009` (for "oauth2 auth_type + absent header_scheme → spec loads successfully"). This conflicts with the BC-canonical `EC-009-043..EC-009-051` range used in BC-2.16.009 for Rule 9 tchar edge cases. Any traceability tool looking for EC-009 in the story would find both the story-local entry and BC-canonical entries — the prefix collision causes ambiguity.

**Resolution (FB46):** Closed. Story-local `EC-009` renumbered to `EC-014` (non-conflicting namespace within story-local IDs). No BC-canonical EC IDs were changed.

**Routing (original):** story-writer

---

## F-WASE-P62-LOW-003 — EC canonicalization applied to 4 of 7 BC-mapped EC rows in story; 3 rows still use story-local IDs (EC-003, EC-005, EC-008) that have BC-canonical analogs

**Severity:** LOW
**Category:** EC-ID consistency (incomplete canonicalization pass)

**Finding:**

S-WAVE-A-ENGINE-001 v2.1 §Edge Cases canonicalized EC-043..046 to BC-canonical IDs EC-009-043..EC-009-046 (FB45 fix for anchor ambiguity). But three other story EC rows that map 1:1 to BC-2.16.009 §Error Conditions ECs were not canonicalized: EC-003 → EC-009-033, EC-005 → EC-009-031, EC-008 → EC-009-042.

**Resolution (FB46):** Closed. Story-writer canonicalized remaining 3 rows: EC-003→EC-009-033, EC-005→EC-009-031, EC-008→EC-009-042. Stated rule added to §Edge Cases: "Story ECs that have a 1:1 BC analog use the BC-canonical ID; story ECs with no BC analog use story-local IDs (currently EC-010..EC-014)."

**Routing (original):** story-writer

---

## F-WASE-P62-LOW-004 — §Tasks section ran A→B→C→D→E→G→F: Phase G appeared before Phase F (non-monotonic ordering)

**Severity:** LOW
**Category:** Document ordering / readability (non-functional)

**Finding:**

S-WAVE-A-ENGINE-001 v2.1 §Tasks listed phases in order: Phase A, Phase B, Phase C, Phase D, Phase E, Phase G (Tasks T-G01..T-G05), Phase F (Tasks T-F01..T-F02). Phase G was added in v2.0 (five exhaustive SensorSpec literal sites) at the end of the document before Phase F was finalized, resulting in non-alphabetical ordering.

**Resolution (FB46):** Closed. Phases reordered to A→B→C→D→E→F→G (alphabetical). No cross-references broken (no task IDs changed, no intra-document phase citations).

**Routing (original):** story-writer

---

## F-WASE-P62-OBS-001 — STORY-INDEX Q-number attribution for S-WAVE-A-ENGINE-001 v2.0 registration diverges from the story's §Version History Q2/Q3/Q4/Q5 mapping

**Severity:** OBSERVATION
**Category:** Index description accuracy / Q-label disambiguation

**Finding:**

STORY-INDEX v2.724 S-WAVE-A-ENGINE-001 row (v2.0 registration) describes: "5 BLOCKERs resolved: Q1 ESpec027 prism-core routing, Q2 validate_header_scheme placement, Q3 cookie-name tchar audit → SEC-001 discovery, Q4 serde bare default for Option<String> UNSAFE→adjudicated safe-as-bare, Q5 token_exchange arm routing to S-ADR054-WAVE-A-001."

The story's §Version History (v2.0 entry) uses Q1-Q6 as labels for architect adjudication decisions from `wave-a-engine-story-adjudication-Q1-Q5.md`, where Q2=fail-fast in parse(), Q3=ADR-031 cookie-name mandate, Q4=&AuthType vs &str, Q5=ship token_exchange message.

The two Q-numbering systems describe different things. The STORY-INDEX used Q-labels for remove-uncertainty BLOCKERs; the adjudication file uses Q-labels for architect decisions. Q2 in STORY-INDEX ("validate_header_scheme placement") is effectively Q4 in the adjudication file ("&AuthType vs &str"). Q4 in STORY-INDEX ("serde Option<String>") is not one of the adjudication Q-numbers at all. Q5 in STORY-INDEX ("token_exchange routing") partially overlaps with adjudication Q5 ("ship message") but is not the same decision.

The STORY-INDEX v2.0 history entry is append-only and cannot be modified. However, the Q-mislabeling creates ongoing confusion for anyone cross-referencing the registration with the adjudication file.

**Resolution (FB46):** State-manager correcting segment for v2.2 includes a Q-number attribution note clarifying which Q-numbering is which. STORY-INDEX v2.0 history preserved unchanged (append-only).

**Routing:** state-manager (STORY-INDEX correcting segment note)

---

## Verified-Clean (Adversary Confidence: HIGH)

The following axes were re-derived from first principles and found clean:

1. **Charset re-derivation (RFC 9110 §5.6.2):** 15 specials (`! # $ % & ' * + - . ^ _ `` | ~`) + 10 digits + 52 letters = 77 tchar characters. Confirmed across all seven enumeration sites in BC-2.16.009 v1.26, error-taxonomy.md v2.69, ADR-053 v0.35, and S-WAVE-A-ENGINE-001 v2.2. All carry the RFC-ordered 15-char list with backtick (U+0060) in the correct position.

2. **Grandfathered-history integrity:** Only three changelog rows legitimately retain the pre-SEC-001 14-character list: BC-2.16.009 v1.24 changelog row (D-2013 burst), error-taxonomy.md v2.68 changelog row (FB45), ADR-053 v0.33 changelog row. These are immutable historical records; no falsification occurred. No live-body sites reference the 14-char list.

3. **EC-ID rename completeness:** EC renaming (EC-009 → EC-014, EC-003 → EC-009-033, EC-005 → EC-009-031, EC-008 → EC-009-042) is complete. Zero surviving old EC-003/EC-005/EC-008/EC-009 story-local references found across all seven story §Edge Cases rows.

4. **`total_contracts: 268` verification by enumeration:** Active 251 + draft 4 + retired 6 + removed 7 = 268. The adversary noted that prior passes raised "BC status counts not independently verified — 251 + 4 + 6 = 261 ≠ 268." **This is not a defect.** There are also 7 `removed` BCs. The complete decomposition is: 251 active + 4 draft + 6 retired + 7 removed = 268. This decomposition is now explicit in BC-INDEX v8.72 §Note: "251 active + 4 draft + 0 deprecated + 7 removed + 6 retired = 268; prior passes occasionally raised 261 ≠ 268 because the removed-7 addend was omitted; all four lifecycle statuses must be summed."

5. **ADR-053 de-normativization structurally sound:** §D2 retains templates (b)/(c) normatively in ADR-053. Template (a) is delegated to BC-2.16.009 + error-taxonomy.md as sole POL-24 carriers. The delegation chain is closed.

6. **SAP-1 (tracing emission catalog):** Not applicable — no `event_type =` additions in this burst.

7. **SAP-2 (DTU↔TOML schema parity):** Not applicable — no TOML spec files touched.

8. **AD-017 credential opacity:** `CachedAuthToken` stores no credential values. New VP-160 proof harness confirmed to use no credential inputs. BC-2.16.014 §AD-017 references verified intact.

9. **POL-36/Q3 non-implication:** ADR-031 §D4 cookie-name fidelity survives as Q3 conclusion. ADR-031 is not superseded; the mechanism changes but the obligation persists.

10. **`&s[7..]` slice panic-safety:** `"cookie:"` prefix is exactly 7 ASCII bytes; slicing at byte 7 is safe. The boundary is checked by the `header_scheme.starts_with("cookie:")` predicate that gates the slice, which only fires if `header_scheme.len() >= 7`.

11. **Overlay bypass (BC-2.06.013):** No bypass path found in the amended perimeter. BC-2.06.013 constraints intact.

---

## Coverage Limitations

1. **BC status count decomposition** (see Verified-Clean item 4): the adversary confirmed this is NOT a defect — the 7 removed BCs are the missing addend. BC-INDEX v8.72 makes all four lifecycle-status addends explicit to close this recurring concern in future passes.

2. **Proptest sampling** for charset coverage: VP-160 (Kani) closes the exhaustive-proof gap for the 256 byte values. Proptest tests in the story cover selected vectors (EC-009-043..051) but are not claimed to be exhaustive.

3. **Wire-level MCP assertions** deferred to `S-WAVE-A-MCP-001`: the pass confirms this deferral is correct under ADR-053 §D6 Option (B). No wire-level `structuredContent.error.code` assertion exists in the current story's scope — this is a documented architectural gap, not an uncontrolled escape.

---

## Pass Verdict

```
CLEAN (strict): no
CLEAN (PR-merge): no
```

21 findings (1 CRIT / 5 HIGH / 10 MED / 4 LOW / 1 OBS). All 21 closed across five legs in FB46 (architect: ADR-053 §D6 + VP-160; product-owner: BC-2.16.008/009 + error-taxonomy; story-writer: S-WAVE-A-ENGINE-001 v2.2; devops-engineer: records-lint arm-5 + stories coverage; state-manager: indexes + STATE.md). Streak 0/3 → 0/3 (new findings reset advancement). NEXT = adversary pass 63 on frozen post-FB46 perimeter.
