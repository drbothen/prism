---
document_type: adversarial-review
review_id: wave-a-spec-pass-64
pass_number: 64
reviewer: vsdd-factory:adversary
review_type: spec-content
artifact_scope:
  reviewed:
    - .factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md (v1.26)
    - .factory/specs/behavioral-contracts/BC-2.16.008-add-sensor-spec-tool.md (v1.6)
    - .factory/specs/behavioral-contracts/BC-2.16.014-declarative-auth-acquisition-token-lifecycle.md (v1.19)
    - .factory/specs/behavioral-contracts/BC-2.01.016-sensor-auth-open-trait-contract.md (v1.15)
    - .factory/specs/behavioral-contracts/BC-2.01.017-static-cookie-auth-provider-no-login-roundtrip.md (v1.10)
    - .factory/specs/behavioral-contracts/BC-2.01.018-cyberint-alerts-cookie-auth.md (v1.4)
    - .factory/specs/behavioral-contracts/BC-2.01.006-cyberint-cookie-auth.md (v1.8)
    - .factory/specs/behavioral-contracts/BC-2.02.004-cyberint-field-mapping.md (v1.10)
    - .factory/specs/architecture/decisions/ADR-053-wave-a-sensor-fidelity-remediation-openapi-grounding-armis-token-exchange-cyberint-dual-surface.md (v0.35)
    - .factory/specs/architecture/decisions/ADR-054-native-declarative-http-auth-acquisition.md (v0.52)
    - .factory/specs/architecture/decisions/ADR-055-validate-sensor-spec-production-wiring.md (v1.0)
    - .factory/specs/architecture/decisions/ADR-026-AMENDMENT-rule-c-keyring-scope.md
    - .factory/specs/architecture/decisions/ADR-026-sensorauth-unsealing.md (v1.41)
    - .factory/specs/architecture/decisions/ADR-028-toml-spec-grounding-vs-dtu-routes.md (v1.28)
    - .factory/specs/verification-properties/vp-153-sensorauth-runtime-cross-composition-prevention.md (v0.28)
    - .factory/specs/verification-properties/vp-159-declarative-http-auth-lazy-acquisition-and-refresh-on-expiry.md (v1.26)
    - .factory/specs/verification-properties/vp-160-rule9-cookie-name-charset-totality-and-injection-rejection.md (v1.0)
    - .factory/specs/prd-supplements/error-taxonomy.md (v2.69)
    - .factory/stories/S-WAVE-A-ENGINE-001-header-scheme-field-rule9-validation-auth-dispatch.md (v2.3)
    - .factory/stories/S-WAVE-A-CYBERINT-PATCH-001-cyberint-header-scheme-patch.md (v1.0)
    - .factory/stories/S-WAVE-A-CYBERINT-SPEC-001-cyberint-dual-surface-spec-migration.md (v1.0)
    - .factory/stories/S-WAVE-A-MCP-001-add-sensor-spec-structured-error-response.md (v1.0)
    - .factory/stories/S-ADR054-WAVE-A-001-declarative-http-auth-acquisition.md (v1.0)
    - .factory/stories/S-ADR055-WAVE-A-001-validate-sensor-spec-production-wiring.md (v1.0)
    - .factory/stories/S-WAVE-A-ARMIS-REMEDIATION-001-armis-token-exchange-spec-and-dtu-reclone.md (v1.0)
frozen_head: factory-artifacts@a68286836
streak_pre_pass: "0/3"
streak_post_pass: "0/3"
verdict: BLOCKED
findings_count: 34
severity_breakdown:
  critical: 5
  high: 8
  medium: 17
  low: 2
  observation: 2
  process_gap: 2
novelty: HIGH
related_state_decision: D-2020
date: 2026-07-25
---

# Adversarial Review — Wave-A Spec-Evolution Perimeter, PASS 64

**Frozen HEAD:** `factory-artifacts @ a68286836`
**Mode:** spec-content semantics (contradictions, unsatisfiable ACs, missing edge cases, incorrect cross-artifact claims, anchor mis-semantics, traceability gaps)
**Cite convention:** section / symbol / field anchors only (TD-VSDD-091 / records-lint L9)

**Version verification (read from each file's own frontmatter on disk):** BC-2.16.009 `1.26`, BC-2.01.017 `1.10`, BC-2.16.014 `1.19`, BC-2.01.016 `1.15`, BC-2.01.018 `1.4`, BC-2.01.006 `1.8`, BC-2.02.004 `1.10`, BC-2.16.008 `1.6`, BC-2.10.007 `1.19`, error-taxonomy `2.69`, VP-153 `0.28`, VP-159 `1.26`, VP-160 `1.0`, ADR-053 `0.35` (accepted), ADR-054 `0.52` (accepted), ADR-055 `1.0` (**proposed**). Every version pin asserted in S-WAVE-A-ENGINE-001 §Behavioral Contracts and in its ACs (`error-taxonomy.md v2.69`) matches disk. **No version-pin drift found** — the preceding records pass held.

**Records dimension (clean — pass-63 held):** Every version pin in the perimeter matches disk. All changelogs are ordered. The Rule 9 tchar charset is consistent at 15 special characters / 77 total across every live site checked. RFC 9110 §5.6.2 arithmetic verified (26+26+10+15=77). The anchor story's AC⇄RGT bijection is complete. No L1, L7, or L9 violations found. The preceding records-compliance pass (pass 63) results remain valid.

**Standing probes summary:** SAP-1 produced NO findings (18 distinct `event_type =` values sampled against BC-2.16.002 §Postconditions — all present). SAP-2 produced NO findings (no sensor TOML modified at frozen HEAD). SAP-3 produced CRIT-002 and HIGH-003. SID-1 produced NO findings. SID-2's concern is realized by CRIT-004 (missing wire-shape assertion requirement for `Authorization` header).

---

## Critical Findings

### F-WASE-P64-CRIT-001 — S-ADR054-WAVE-A-001 AC-003 declares eight Rule 10 sub-conditions that contradict ADR-054 §D10, BC-2.16.009 Rule 10, and error-taxonomy E-SPEC-028
**Artifact/anchor:** `.factory/stories/S-ADR054-WAVE-A-001-declarative-http-auth-acquisition.md` §Acceptance Criteria → AC-003 (and §Edge Cases EC-001..EC-005, §Tasks T-04)
**Confidence:** HIGH

**What is wrong.** AC-003 enumerates `D10(a)`–`D10(h)` as the "8 sub-conditions from ADR-054 §D10". Compared against the three ratified sources (ADR-054 §D10 sub-condition headings `(a) Required block absent` … `(h) token_exchange-only fields on non-token_exchange block`; BC-2.16.009 §Validation Rules 10 "Sub-conditions (8 checks)"; error-taxonomy E-SPEC-028 row), the story's letters are systematically mis-mapped **and four sub-conditions are inventions**:

| Story AC-003 label | Story text | Ratified reality |
|---|---|---|
| D10(a) | block present + auth_type ∉ declarative | This is ratified **(g)**, not (a) |
| D10(b) | auth_plugin on declarative | matches ratified (b) |
| D10(c) | token_exchange without block | subset of ratified **(a)** |
| D10(d) | `token_path` absent/empty | second half of ratified **(a)** |
| D10(e) | `absolute_utc_string` without `expiry_field` | **not in ADR-054 §D10, BC, or taxonomy** |
| D10(f) | `relative_seconds` without `expiry_field` | **not in ADR-054 §D10, BC, or taxonomy** |
| D10(g) | `token_response_path` must start with `$.` | **not in ADR-054 §D10, BC, or taxonomy** — and ADR-054 §D3 defines `token_response_path` as a **dotted** path (`data.access_token`), i.e. explicitly *not* `$.`-prefixed |
| D10(h) | `ttl_buffer_secs = 0` → error | **not in ADR-054 §D10, BC, or taxonomy** — and ADR-054 §D3 makes `ttl_buffer_secs` optional with default 30; BC Rule 10(d) states omitting it is VALID |

Ratified sub-conditions **(c) invalid `expiry_mode` value, (d) four token_exchange required fields, (e) `credential_body_field` not in `[[credential_refs]]`, (f) oauth2 missing `client_id`/`client_secret`, (h) token_exchange-only fields on wrong auth_type** are entirely absent from the story.

**Why it matters.** This is the **sole** implementation story for Rule 10 (§Authority: "This story is the single implementation story for ADR-054. All five ADR-054 decision points … are in scope"). An implementer following AC-003 builds an E-SPEC-028 that rejects valid specs (`ttl_buffer_secs = 0`, dotted `token_response_path` per ADR-054 §D3's own Armis example) and accepts invalid ones (oauth2 with no `client_id`/`client_secret`, token_exchange missing `expiry_field`). ADR-054 §D10's own header note declares §D10 authoritative for trigger logic; the story overrides it silently. §Edge Cases EC-001–EC-005 and T-04 all propagate the same wrong letters.

**Routing:** story-writer (rewrite AC-003 / EC-001–EC-005 / T-04 against ADR-054 §D10 verbatim).

---

### F-WASE-P64-CRIT-002 — S-ADR054-WAVE-A-001 places Rule 10 in `validate_sensor_spec()`, which no production spec-load path calls; its own SAP-3 test instruction is self-contradictory
**Artifact/anchor:** `S-ADR054-WAVE-A-001` AC-003 opening sentence, §Architecture Mapping `validate_sensor_spec()` row, §Tasks T-04
**Confidence:** HIGH

**What is wrong.** AC-003 states "`validate_sensor_spec()` implements Rule 10 with E-SPEC-028"; §Architecture Mapping assigns the change to `crates/prism-spec-engine/src/validation.rs` `validate_sensor_spec()`. This contradicts:
- **BC-2.16.009 §Validation Rules 9 → §Integration function**: "The S-WAVE-A-ENGINE-001 implementation adds Rules 9 and 10 inside `SpecLoader::parse()` — **not inside `validate_sensor_spec()`** — ensuring they execute on every path that calls `parse()`."
- **BC-2.16.009 §Security requirement (Rule 9)**: "an implementation that calls only `validate_sensor_spec()` in the `add_sensor_spec` handler bypasses Rules 8/9/10 (none of Rules 8–10 are in `validate_sensor_spec()`)."
- **Verified code:** `parse_and_validate_spec_toml` in `crates/prism-spec-engine/src/add_sensor_spec.rs` calls `SpecLoader::parse`, then `resolve_env_var_tokens`, then `validate_step_methods` — it does **not** call `validate_sensor_spec`. ADR-055 §Context ("Production spec-loading paths verified", Surfaces 1–3) independently confirms `validate_sensor_spec()` has **zero production callers** today.

**Self-contradiction inside the same AC:** AC-003 closes with "A test for each sub-condition verifies rejection via `parse_and_validate_spec_toml()` (SAP-3: reachable from integration surface)". If Rule 10 lives in `validate_sensor_spec()`, that surface cannot reach it — the AC's own verification instruction is unsatisfiable under the AC's own placement decision. T-04 repeats both halves.

**Why it matters.** Shipping as written makes E-SPEC-028 unreachable from the `add_sensor_spec` MCP tool and from hot-reload — the two write surfaces where a malicious or malformed `[auth_acquisition]` block enters. It is the same class of bypass BC-2.16.009 §Security requirement was authored to prevent for Rule 9.

**Routing:** story-writer (re-anchor to `SpecLoader::parse()` per BC-2.16.009 §Integration function), with architect confirmation of the interaction with ADR-055 §D1/§D3 (which argues *against* putting semantic validation in `parse()` for the env-var-ordering reason — Rule 10 does not read `base_url`, so the objection does not apply, but the two ADRs should state this explicitly).

---

### F-WASE-P64-CRIT-003 — S-WAVE-A-CYBERINT-SPEC-001 AC-003 and AC-004 are jointly unsatisfiable: `offset_limit` never emits `page`/`size`
**Artifact/anchor:** `S-WAVE-A-CYBERINT-SPEC-001` AC-003 ("Pagination type: offset_limit with `page_size = 100`"), AC-004 ("Page/size pagination replaces cursor pagination"), §Tasks T-02 pagination block and its note
**Confidence:** HIGH

**What is wrong.** AC-004 requires the wire contract `POST /alert/api/v1/alerts?page=N&size=M`. AC-003/T-02 declare the spec's pagination as `type = "offset_limit"`, `page_size = 100`. Verified in `crates/prism-spec-engine/src/pipeline.rs`:
- `build_paged_url_impl`, `PaginationConfig::OffsetLimit` arm — for POST returns the URL unchanged; for any non-POST method appends `offset=N&limit=M`.
- The POST branch injects `"offset"` and `"limit"` as **top-level JSON body keys** (the code comment anchors this to BC-2.16.002 §Postconditions "OffsetLimit Pagination Dispatch: POST-body vs GET-URL").
- The only other arm, `PaginationConfig::CursorToken`, emits `cursor=` and `page_size=`.

**There is no pagination type in the engine grammar that emits `page`/`size`.** T-02's assertion "`offset_limit` pagination with `page_size = 100` maps to the Cyberint API's `page` and `size` query parameters" is false.

**Why it matters.** Prism would POST `{"offset":0,"limit":100}` to a DTU (and real API) that reads `?page=&size=` — the parameters are ignored, page 1 is returned on every iteration, and `build_paged_url`'s termination condition (`page_record_count < ps` → break) exits after one page. Silent truncation of a security alerts feed: exactly the SOUL.md §4 / CWE-390 class this cascade has been closing elsewhere. T-02 defers resolution to the implementer ("MUST verify … and either use the matching pagination type **or request a spec grammar extension**") — a grammar extension is a scoped architectural change, not an implementer-time contingency (CLAUDE.md Canonical Principle Rules 1 and 6).

**Routing:** architect (decide: extend the pagination grammar with a `page_size_numbered` variant, or re-ground the DTU + spec on `offset`/`limit` against `cyberint_alerts_openapi_06.20.2026.json`), then story-writer to restate AC-003/AC-004 consistently.

---

### F-WASE-P64-CRIT-004 — S-WAVE-A-ARMIS-REMEDIATION-001 target spec omits `header_scheme = "raw"`; the migrated Armis sensor would send `Authorization: Bearer` and 401 on every request
**Artifact/anchor:** `S-WAVE-A-ARMIS-REMEDIATION-001` §Current State vs Target State → "Target state (this story)" TOML block; AC-001 required-fields list; §File Structure Requirements `armis.sensor.toml` MODIFY row
**Confidence:** HIGH

**What is wrong.** The target-state TOML block and AC-001's enumerated requirements list `auth_type = "token_exchange"`, absence of `auth_plugin`, an `[auth_acquisition]` block, and updated `credential_refs`. **`header_scheme` appears nowhere in the story** (verified by grep across the whole file). Per BC-2.16.009 Rule 9 absence path A, an absent `header_scheme` on a non-`cookie_roundtrip` auth_type passes validation silently and `build_request()` applies the runtime `"bearer"` default.

The ratified requirement is the opposite:
- **ADR-054 §D3 "Armis wiring (`token_exchange`)"** declares `header_scheme = "raw"` with the inline rationale `# Authorization: {token} — no Bearer prefix (ADR-053 D2)`.
- **BC-2.01.017 §P2 dispatch table**, `"raw"` row: `Authorization: {token}` (no "Bearer" prefix) — "`token_exchange` (e.g., Armis Centrix — **Bearer prefix causes HTTP 401**)".
- BC-2.16.009 Rule 9 §Syntactic check names `"raw"` as "used by sensors whose API rejects the prefix, e.g., Armis Centrix".

**Why it matters.** Rule 9 cannot catch this — absence path A is a legitimate pass for `token_exchange`. The spec loads clean, the story's ACs all pass, and every Armis data fetch fails at runtime with HTTP 401. The story is the sole Armis migration anchor and would ship a non-functional sensor with a green gate.

**Routing:** story-writer (add `header_scheme = "raw"` to the target-state block and to AC-001's required-fields list, with an AC asserting the emitted `Authorization` header carries no `Bearer ` prefix — wire-shape assertion discipline).

---

### F-WASE-P64-CRIT-005 — S-WAVE-A-ENGINE-001 has no AC and no Red Gate test for three normative Rule 9 mechanisms that BC-2.16.009 v1.26 mandates (POL-38)
**Artifact/anchor:** `S-WAVE-A-ENGINE-001` §Acceptance Criteria (AC-001..AC-022), §Tasks → Red Gate tests (RG-001..RG-026), §Tasks T-B02 step 3 `is_valid_cookie_name_tchar`; against BC-2.16.009 §Validation Rules 9 sub-sections "Length bound (MED-005 / CWE-390)", "64-codepoint echo cap for `{value}` in template (a)", "CTL-character escaping for `{value}` in template (a)", and EC-009-047, EC-009-048, EC-009-049, EC-009-051
**Confidence:** HIGH

**What is wrong.** BC v1.26 added five edge cases (EC-009-047..051). The story's §Behavioral Contracts row for BC-2.16.009 acknowledges all five by name. Coverage as authored:

| BC EC | Normative mechanism | Story AC | Story RGT |
|---|---|---|---|
| EC-009-047 | 64-codepoint `{value}` echo cap (CWE-400) | **none** | **none** |
| EC-009-048 | `\xNN` CTL escaping in emitted message (CWE-117) | **none** | **none** |
| EC-009-049 | TAB (0x09) rejection | **none** (AC-019 names `;`, `=`, SP, "CTL bytes (e.g., LF/CR)") | **none** |
| EC-009-050 | high-byte rejection | AC-022 | RG-026 |
| EC-009-051 | ≤128-codepoint cookie-name bound (CWE-390) | **none** | **none** |

Worse, the implementation task actively omits two of them. **T-B02 step 3** gives the full body of `is_valid_cookie_name_tchar` — `!name.is_empty() && name.bytes().all(|b| matches!(…))` — **with no length guard**. T-B02's error-construction bullet specifies `message: verbatim_template_text` with **no cap and no escape step**. An implementer executing T-B02 literally produces a Rule 9 that lacks the CWE-390 length bound, the CWE-400 echo cap, and the CWE-117 log-injection escaping.

**Why it matters.** POL-38 (`bc_new_ec_story_ac_rgt_obligation`, HIGH) requires new BC ECs on a draft `tdd_mode: strict` story to produce ACs and Red Gate tests in the same scope or an explicit deferral with a concrete blocker and future story anchor. There is neither. These are not cosmetic ECs — they are the three security mitigations product-owner added in the same BC bump (F-WASE-P62-HIGH-002 and F-WASE-P62-MED-005 per the BC v1.26 changelog row). Under CLAUDE.md production-grade default, shipping Rule 9 without them is not an MVP tradeoff.

**Routing:** story-writer (add ACs + RG-027..RG-030 for the 128-codepoint bound, the 64-codepoint cap, the `\xNN` escaping, and TAB; extend T-B02's `is_valid_cookie_name_tchar` snippet with the length guard and add a cap+escape step to the template-(a) construction bullet).

---

## Important Findings

### F-WASE-P64-HIGH-001 — Mutual `blocks:` cycle between S-WAVE-A-ENGINE-001 and S-WAVE-A-CYBERINT-PATCH-001; the ENGINE-001 edge direction is semantically inverted
**Artifact/anchor:** `S-WAVE-A-ENGINE-001` frontmatter `blocks:` (first entry `S-WAVE-A-CYBERINT-PATCH-001`, comment "registered FB47a") and `S-WAVE-A-CYBERINT-PATCH-001` frontmatter `blocks:` (`S-WAVE-A-ENGINE-001`)
**Confidence:** HIGH

Both stories list each other in `blocks:`, and both carry `depends_on: []`. In a DAG-based wave scheduler this is an unschedulable cycle. Semantically, `A blocks B` means B is gated on A. PATCH-001's own justification is "ENGINE-001 MUST NOT merge without this story landing simultaneously" — i.e. PATCH gates ENGINE, so PATCH→ENGINE is the correct edge. ENGINE-001's reciprocal edge asserts the opposite (PATCH gated on ENGINE), which is precisely the ordering that opens the boot-broken window the gate exists to close. A co-land constraint is not expressible as two opposed `blocks:` edges; it needs a single directional edge plus the co-land marker (which both stories already carry in prose as `MERGE-GATE-ENGINE-001`).

**Note on FB47b:** The FB47b changelog row documented the `blocks:` edge addition as FB47a made it. That records entry remains valid — it documented what landed. Pass 64 has now found the underlying edge direction itself is semantically wrong; that is HIGH-001's scope, not a records defect.

**Routing:** story-writer + state-manager (drop the ENGINE-001 → PATCH-001 `blocks:` edge, retain PATCH-001 → ENGINE-001, keep the prose co-land gate; sweep STORY-INDEX dependency rows).

---

### F-WASE-P64-HIGH-002 — S-WAVE-A-CYBERINT-PATCH-001 attributes Rule 9's liveness to the wrong function and to the wrong story, and contradicts itself on whether the boot failure is unconditional
**Artifact/anchor:** `S-WAVE-A-CYBERINT-PATCH-001` §MERGE-GATE-ENGINE-001 first paragraph; AC-002 second paragraph
**Confidence:** HIGH

Two false load-bearing claims:
1. §MERGE-GATE-ENGINE-001: "After `S-WAVE-A-ENGINE-001` merges, **`validate_sensor_spec()` is wired into the production spec-loading pipeline** with Rule 9 live." Rule 9 lands inside `SpecLoader::parse()` (BC-2.16.009 §Integration function; ENGINE-001 T-B03: "`SpecLoader::parse()` is the unconditional call point — calling `validate_header_scheme` only from `validate_sensor_spec()` bypasses Rules 8/9/10"). Wiring `validate_sensor_spec()` into production is ADR-055 / S-ADR055-WAVE-A-001's scope, not ENGINE-001's.
2. AC-002: "This test exercises the full load path (… → Rule 9 **via `validate_sensor_spec()` when S-ADR055-WAVE-A-001 is also merged**)."

**Internal contradiction:** §MERGE-GATE-ENGINE-001 asserts unconditionally "every prism startup fails at spec load time with exit code 2 until this patch lands." AC-002 makes Rule 9's liveness conditional on ADR-055 wiring. If AC-002 were true, ENGINE-001 alone would not break boot and the whole P0 co-land gate would be unnecessary. The gate *is* necessary — because Rule 9 is in `parse()`, which `load_all()` (boot) and `parse_and_validate_spec_toml()` both call — but the story's stated mechanism is wrong at both sites, so a reviewer verifying the gate against the story's own reasoning would conclude it can be relaxed.

Independently: ADR-055 is `status: proposed`, so binding a P0 boot-critical gate's rationale to it is unsound regardless.

**Routing:** story-writer.

---

### F-WASE-P64-HIGH-003 — ADR-053 §D6 deferral integrity broken: the deferred E-SPEC-027 wire-level obligation is not carried by any AC in S-WAVE-A-MCP-001
**Artifact/anchor:** ADR-053 §D6 "Follow-up story anchor (deferred remap)" → "Acceptance shape" bullet; `S-WAVE-A-MCP-001` §Acceptance Criteria AC-001..AC-007 and §Edge Cases EC-005
**Confidence:** HIGH

ADR-053 §D6 states the deferred obligation precisely: *"Acceptance shape: `add_sensor_spec` with a non-tchar cookie name returns `isError: true` + `structuredContent.error.code == "E-SPEC-027"` at the MCP wire level."*

In the receiving story:
- AC-001 and AC-002 use `base_url = "ftp://x"` → `"E-SPEC-001"`. AC-003 uses two Rule-1–5 violations. AC-004/AC-005/AC-006/AC-007 are envelope/negative/completeness/no-regression ACs. **No AC exercises a non-tchar cookie name, and no AC asserts `code == "E-SPEC-027"`.**
- The obligation appears only as §Edge Cases EC-005 — "E-SPEC-027 errors (Rule 9 from ENGINE-001) appear in errors array | The same structured error shape applies; AC-003 covers multi-error cases" — which is narration, has no AC, has no Red Gate test, and mis-cites AC-003 (AC-003 is about Rule-1–5 errors, not E-SPEC-027).

Meanwhile the deferring story correctly disclaims the coverage (S-WAVE-A-ENGINE-001 AC-020: "do NOT assert `structuredContent.error.code` in this story… **S-WAVE-A-MCP-001 carries the original wire-level intent**"), and BC-2.16.008 §Error Conditions E-SPEC-027 row binds the normative MUST to the literal story ID. So the hand-off is airtight on the sending side and the ADR side — and **drops on the receiving side**. Under CLAUDE.md Canonical Principle Rule 3, a deferral must restate the obligation, not merely name the recipient.

**Routing:** story-writer (add an AC + Red Gate test to S-WAVE-A-MCP-001 asserting `isError: true` and `structuredContent.error.code == "E-SPEC-027"` for `header_scheme = "cookie:bad;name"`, mirroring ADR-053 §D6's acceptance shape verbatim).

---

### F-WASE-P64-HIGH-004 — VP-160's anchor story is a placeholder in both the VP file and VP-INDEX, while the story that builds the proof target omits VP-160 (mis-anchoring — always blocks convergence)
**Artifact/anchor:** `vp-160-rule9-cookie-name-charset-totality-and-injection-rejection.md` §Source Contract → "**Anchor Story:** `[PLANNED — Wave-A Rule 9 cookie-name charset story]`"; `VP-INDEX.md` VP-160 row anchor-story cell (same placeholder); `S-WAVE-A-ENGINE-001` frontmatter `verification_properties: [VP-153]`
**Confidence:** HIGH

VP-160's proof target is `prism_spec_engine::spec_parser::is_valid_cookie_name_tchar`. That symbol is **introduced by S-WAVE-A-ENGINE-001** — its full body is authored in that story's §Tasks T-B02 step 3, and §Architecture Mapping assigns `SpecLoader::validate_header_scheme` (Rule 9) to `crates/prism-spec-engine/src/spec_parser.rs`. Under the VSDD anchoring rule (the VP's `anchor_story` builds the test vehicle), the anchor is knowable and is S-WAVE-A-ENGINE-001. Instead both the VP file and the VP-INDEX row carry a `[PLANNED — …]` placeholder, and the story's `verification_properties` array omits VP-160 — so there is no traversable edge in either direction.

This is also a CLAUDE.md Rule 6 violation: a placeholder standing in for a question answerable in current scope. POL-4 (semantic anchoring integrity), POL-5 (creators justify anchors), POL-9 (VP-INDEX is the VP catalog source of truth) all apply.

**Routing:** architect (VP-160 §Source Contract Anchor Story → `S-WAVE-A-ENGINE-001`), state-manager (VP-INDEX VP-160 anchor-story cell), story-writer (add VP-160 to S-WAVE-A-ENGINE-001 `verification_properties` with an anchor justification comment).

---

### F-WASE-P64-HIGH-005 — S-ADR054-WAVE-A-001's two declared `status: ready` blockers are both unsound: one cites a phantom BC section (POL-21), the other claims a BC gap that was closed 14 BC versions ago
**Artifact/anchor:** `S-ADR054-WAVE-A-001` §Product-Owner Dependencies → PO-001 and PO-002
**Confidence:** HIGH

**PO-001** — "BC-2.01.017 **§Adapter Registration** dispatch table must gain a `TokenExchange` row". BC-2.01.017 v1.10 has no such section. Its complete heading set is: §Description, §Preconditions, §Postconditions (§P1 Token Acquisition, §P2 Request Header Injection, §P3 Auth Type Dispatch, §P4 Zero Login-Shaped Requests), §Invariants, §Error Cases, §Edge Cases, §Canonical Test Vectors, §Verification Properties, §Related BCs, §Architecture Anchors, §Story Anchor, §VP Anchors, §Traceability, §Notes for Implementers, §Changelog. POL-21 (`phantom_section_anchor_prohibited`, HIGH) applies. Additionally the requirement itself is largely moot: the §P2 dispatch table is `header_scheme`-keyed (per ADR-053 D2), and its `"raw"` row already names `token_exchange` explicitly.

**PO-002** — "BC-2.16.009 §Validation Rules section must add Rule 10 description covering E-SPEC-028 and its 8 sub-conditions from ADR-054 §D10. **The rule exists in the ADR but is not yet in the BC.**" False: BC-2.16.009 §Validation Rules 10 is fully specified with all eight sub-conditions and has been since v1.12 (BC changelog row v1.12: "added §Validation Rule 10 — `[auth_acquisition]` Coherence Validation (E-SPEC-028): 8 sub-conditions"). ADR-054 §D11's `BC-2.16.009 Rule set` row is annotated **[EXECUTED — Wave-A spec evolution burst 3, 2026-07-22]**.

**Why it matters.** Both are declared as gates that "BLOCKS status: ready". A gate anchored on a nonexistent section can never be closed; a gate anchored on already-satisfied work invites a redundant PO burst on an active BC. Combined with CRIT-001, S-ADR054-WAVE-A-001 has not been re-grounded against its own ADR since it was stubbed.

**Routing:** story-writer (retarget PO-001 to §P3/§P2 with a concrete delta or retire it; retire PO-002 as already-satisfied).

---

### F-WASE-P64-HIGH-006 — S-WAVE-A-ARMIS-REMEDIATION-001 binds `status: ready` to resolving seven `# TBD` values against an "Armis OpenAPI" that ADR-053 states does not exist, and its guessed values contradict ADR-054 §D3's ratified wiring
**Artifact/anchor:** `S-WAVE-A-ARMIS-REMEDIATION-001` §Current State vs Target State → "Target state" TOML block and the closing sentence "**All TBD values above MUST be resolved against the Armis OpenAPI spec before this story reaches `status: ready`.** The implementer must read the Armis OpenAPI (locate in the research directory or codebase)"
**Confidence:** HIGH

Two compounding defects.

**(a) The named grounding artifact does not exist.** ADR-053 §Context states plainly: "**For Armis, no downloadable OpenAPI exists.** Claims are web-corroborated from multiple independent production connectors…". ADR-053 §D1 "No-OpenAPI governance" names Armis and CrowdStrike as the no-OpenAPI sensors. `.factory/reference/api-specs/` contains only `cyberint_alerts_openapi_06.20.2026.json`, `cyberint_assets_openapi_06.20.2026.json`, `xdome_openapi_06.20.2026.json`, `README.md` — no Armis file, and no Armis OpenAPI anywhere in the repo. The story's `status: ready` gate is therefore unreachable as specified.

**(b) The TBD guesses contradict the ratified wiring that already exists.** ADR-054 §D3 "Armis wiring (`token_exchange`)" is a complete, accepted, normative TOML block. Comparing:

| Field | Story target-state (`# TBD`) | ADR-054 §D3 (accepted, v0.52) |
|---|---|---|
| `token_path` | `/access_management/oauth/authorize` | `/api/v1/access_token/` |
| `token_response_path` | `$.access_token` | `data.access_token` (dotted, no `$.` — see §D3 field table "Dotted JSON path") |
| `expiry_field` | `expiration_utc` | `data.expiration_utc` |
| `credential_body_field` | `secret_key` | `secret_key` ✓ |
| `expiry_mode` | `absolute_utc_string` | `absolute_utc_string` ✓ |

The story also derives the DTU route from its own wrong guess ("The Armis DTU serves a `POST /access_management/oauth/authorize` route"), propagating the divergence into AC-002.

**Why it matters.** Per CLAUDE.md Source-of-Truth Precedence rule 2, ADR-054 §D3 is the ratified answer; the story frames it as an open question and answers it wrongly. Seven `# TBD` markers plus "TBD — verify response shape" in a spec artifact is a textbook Canonical-Principle Rule 1/Rule 6 pattern.

**Routing:** story-writer (replace the entire target-state block with ADR-054 §D3's Armis wiring verbatim, add `header_scheme = "raw"` per CRIT-004, delete all `# TBD` markers, and re-anchor the confidence-tier gate to ADR-053 §D1's no-OpenAPI governance rather than a nonexistent OpenAPI).

---

### F-WASE-P64-HIGH-007 — ADR-053 names a machine-local absolute path outside the repository as the canonical normative grounding reference for Armis
**Artifact/anchor:** ADR-053 §Context, the paragraph beginning "For Armis, no downloadable OpenAPI exists" → "Source file: `/Users/jmagady/Dev/test-soc/demo-soc/findings/prism-armis-endpoint-plan.md`"; ADR-053 §D1 "No-OpenAPI governance"
**Confidence:** HIGH

ADR-053 §D1 makes that document's Confirmed/Partial/Unconfirmed confidence tiers **the contract**: "Only Confirmed-tier claims (corroborated by two or more independent production connectors) may be spec'd without live-tenant validation. Partial-tier claims require a DTU-EXT-NNN blocker reference." The document is not in the prism repository (no match for `prism-armis-endpoint-plan.md` anywhere under the repo root) and the cited path is a developer-machine absolute path in a different repository (`test-soc`).

**Why it matters.** No other agent, no reviewer, and no CI job can verify a single Armis spec value against the contract that ADR-053 declares binding. Every Armis endpoint/auth claim in the Wave-A perimeter is therefore unauditable, which is the root enabler of HIGH-006's wrong guesses surviving. This is the same failure class the Cyberint half of ADR-053 avoided by vendoring both OpenAPI JSONs into `.factory/reference/api-specs/`.

**Routing:** architect (vendor the findings document into `.factory/reference/` — or extract the Confirmed-tier endpoint/auth claim table inline into ADR-053 §D1 — and replace the machine-local path with a repo-relative cite).

---

### F-WASE-P64-HIGH-008 — BC-2.16.009 §Integration function asserts S-WAVE-A-ENGINE-001 adds Rule 10; the story explicitly scopes Rule 10 out
**Artifact/anchor:** BC-2.16.009 §Validation Rules 9 → "**Integration function:**" paragraph, final sentence: "The S-WAVE-A-ENGINE-001 implementation adds Rules 9 and 10 inside `SpecLoader::parse()`"
**Confidence:** HIGH

S-WAVE-A-ENGINE-001 §Behavioral Contracts BC-2.16.009 row: "**Rule 10 (`[auth_acquisition]`) is OUT OF SCOPE**." Its §Architecture Mapping lists only `SpecLoader::validate_header_scheme` (Rule 9). Rule 10 is owned by S-ADR054-WAVE-A-001 (which additionally places it in the wrong function — CRIT-002). The BC's sentence is therefore false about the story it names.

**Why it matters.** This sentence is the only place in the BC that binds Rule 10 to a placement decision and a story. Its falsity is exactly what let S-ADR054-WAVE-A-001's `validate_sensor_spec()` placement go unchallenged: a reviewer checking "who puts Rule 10 in `parse()`?" is pointed at a story that disclaims it. POL-22 (adversarial citation and entity verification), POL-25 (multi-cite propagation sweep).

**Routing:** product-owner (split the sentence: Rule 9 → S-WAVE-A-ENGINE-001; Rule 10 → S-ADR054-WAVE-A-001, with the `SpecLoader::parse()` placement requirement restated for both).

---

## Medium Findings

### F-WASE-P64-MED-001 — BC-2.16.009's VP-160 row claims 256-byte exhaustive proof; VP-160 bounds its harness to 128 ASCII values with a structural argument for high bytes
**Artifact/anchor:** BC-2.16.009 §Verification Properties, VP-160 row ("…are all rejected — **exhaustive proof across all 256 byte values**"); VP-160 §Proof Method table Bounded? cell ("Yes — single-byte ASCII inputs (**128-point exhaustive space**); non-ASCII rejected by **structural argument**") and §Proof Harness Skeleton (`kani::assume(b <= 0x7F)`), §Feasibility Assessment "Non-ASCII coverage | Structural argument"
**Confidence:** HIGH

The VP is internally consistent and its reasoning is sound (0x80–0xFF cannot appear as standalone bytes in a valid `&str`). The BC row overstates it. POL-9 makes VP-INDEX/VP the catalog authority; the VP-INDEX VP-160 row correctly avoids the "256" claim, so the BC row is the lone outlier. Consequence if left: a formal-verifier in Phase 6 would look for a 256-point harness that the VP deliberately does not specify, or a reviewer would credit exhaustive high-byte coverage that does not exist.

**Routing:** product-owner (BC row → "exhaustive proof across all 128 ASCII byte values; non-ASCII structurally excluded per VP-160 §Feasibility Assessment").

---

### F-WASE-P64-MED-002 — S-WAVE-A-ENGINE-001 EC-009-047 specifies an ellipsis marker that the BC and the existing helper do not produce (POL-24 byte-identity break)
**Artifact/anchor:** `S-WAVE-A-ENGINE-001` §Edge Cases EC-009-047 row: "`{value}` substitution in the error message is capped at 64 codepoints (**excess replaced with `…`**)"
**Confidence:** HIGH

BC-2.16.009 EC-009-047 states the opposite: "the emitted `{value}` is **the first 64 `X` characters**", and Rule 9's §64-codepoint echo cap prescribes `truncate_at_char_boundary(&header_scheme_value, 64)` — "the same helper used by Rule 7's method echo cap". The as-built helper `truncate_at_char_boundary` in `crates/prism-spec-engine/src/validation.rs` returns `&s[..idx]` (or `s` unchanged) — a plain slice, no appended marker. Implementing the story's ellipsis would break POL-24 byte-identity against error-taxonomy E-SPEC-027 template (a) and diverge from the Rule 7 / `base_url` truncation precedents.

**Routing:** story-writer (strike "excess replaced with `…`"; state plain truncation via `truncate_at_char_boundary`).

---

### F-WASE-P64-MED-003 — E-SPEC-027 template (a) is reused for the 128-codepoint length violation but its text names only the charset, producing actively misleading operator guidance
**Artifact/anchor:** BC-2.16.009 §Error message (syntactic) and §Error Conditions E-SPEC-027 row ("template (a) also fires when cookie `<name>` exceeds 128 codepoints (MED-005 / CWE-390; EC-009-051)"); error-taxonomy E-SPEC-027 template (a); BC-2.16.009 EC-009-051
**Confidence:** HIGH

For a 129-character all-`a` cookie name, EC-009-051 states explicitly "The tchar check would PASS for all-`a` input … only the length bound triggers the rejection" — yet the emitted message is `"… Valid values: bearer, raw, cookie:<name> (non-empty name required, RFC 6265 token characters only: A-Z a-z 0-9 ! # $ % & ' * + - . ^ _ \` | ~)"`. Every character in the operator's value satisfies the only constraint the message names. Compounding it, the 64-codepoint echo cap truncates the offending value, so the operator cannot even see that it is overlong. There is no third template and no length clause.

**Why it matters.** The stated rationale for the MED-005 length bound was to *eliminate* a CWE-390 deferred-opaque-failure mode (opaque HTTP 431). Substituting an opaque load-time message that misattributes the cause reintroduces the diagnostic opacity at a different layer. Under the production-grade lens this is a blocker, not an advisory: the fix is a bounded template amendment, not new mechanism.

**Routing:** product-owner (add a length clause to template (a) — e.g. `… (non-empty name, ≤128 codepoints, RFC 6265 token characters only: …)` — or register a distinct template (d) for the length violation; propagate byte-verbatim to error-taxonomy E-SPEC-027 and to S-WAVE-A-ENGINE-001 AC text per POL-24 / POL-29).

---

### F-WASE-P64-MED-004 — BC-2.16.009 §Invariants states an unqualified no-fail-fast rule that Rule 9's ratified design violates
**Artifact/anchor:** BC-2.16.009 §Invariants first bullet ("Validation is **always** a single-pass, all-errors-collected operation (no fail-fast on first error)") and §Multi-Error Reporting first bullet; against `S-WAVE-A-ENGINE-001` §Tasks T-B02 "Q2 VP-059 explicit exclusion note" ("Rule 9 is **fail-fast** in `parse()` following the Rule 8 precedent") and BC-2.16.009 §Validation Rules 10 opening ("All sub-conditions are checked in a single pass (no fail-fast); all errors are collected")
**Confidence:** HIGH

Three artifacts scope the invariant differently and only one of them is the contract. ADR-055 §Context ("The collect-all invariant vs. parse() single-error signature") argues the invariant "explicitly refers to the semantic validation pass within `validate_sensor_spec()`" — but the BC text carries no such qualifier and applies to all ten rules. Observable consequence: a spec with both an invalid `header_scheme` and an `[auth_acquisition]` coherence violation yields **one** error (Rule 9 returns `Err` from `parse()`, so Rule 10 never executes), contradicting §Invariants and defeating Rule 10's own collect-all promise for any spec that also trips Rule 9.

Per CLAUDE.md Source-of-Truth Precedence rule 1, the BC supersedes the story on contract semantics — so as written, the story is in violation. The correct resolution is almost certainly to scope the BC invariant, but that is a product-owner decision, not something a story's inline note can settle.

**Routing:** product-owner (scope the §Invariants bullet explicitly — per-function collect-all semantics, with the cross-rule fail-fast boundary between `parse()`-resident rules stated); then story-writer removes the unilateral Q2 assertion.

---

### F-WASE-P64-MED-005 — S-WAVE-A-MCP-001 AC-005 declares "8 required fields" and enumerates nine; BC-2.10.007 has nine
**Artifact/anchor:** `S-WAVE-A-MCP-001` AC-005 heading and body; BC-2.10.007 §Complete field specification for `structuredContent.error`
**Confidence:** HIGH

BC-2.10.007 v1.19 §Complete field specification lists nine rows all marked Required "Always": `code`, `message`, `category`, `retryable`, `retry_after_seconds`, `suggestion`, `source`, `original_params_valid`, `upstream_message`. AC-005 says "all 8 required fields" in both its heading trace and its body, then enumerates all nine, then instructs "The test must check each field individually." A count-driven test author writes eight assertions.

**Routing:** story-writer.

---

### F-WASE-P64-MED-006 — S-WAVE-A-MCP-001 EC-001 specifies a phantom error code `E-SPEC-000` (POL-22)
**Artifact/anchor:** `S-WAVE-A-MCP-001` §Edge Cases EC-001 ("emit errors: [] and code = `"E-SPEC-000"` or the taxonomy's general spec error code"); §Implementation Notes code snippet `.unwrap_or("E-SPEC-001")`
**Confidence:** HIGH

`E-SPEC-000` does not exist anywhere in `.factory/specs/` (zero matches corpus-wide). The alternative offered ("the taxonomy's general spec error code") is unresolved — there is no general spec error code; `E-SPEC-001` is specifically schema/variable-reference/TOML-parse. The `.unwrap_or("E-SPEC-001")` fallback in the snippet fabricates a schema-error code for a zero-error response. POL-22 (entity verification, HIGH) and CLAUDE.md Rule 6 (answerable in scope) both apply.

**Routing:** story-writer (resolve EC-001 to a real code or make the zero-error case a hard invariant with a `debug_assert`), product-owner if a sentinel taxonomy code is genuinely required.

---

### F-WASE-P64-MED-007 — S-WAVE-A-ENGINE-001 declared AC count (21) disagrees with the body (23)
**Artifact/anchor:** `S-WAVE-A-ENGINE-001` frontmatter `# Points justification:` → "TDD test coverage across **all 21 ACs** (unit + integration + MCP surface): 1.5 pt"
**Confidence:** HIGH

Enumerated ACs: Tier 1 = AC-001..004 (4); Tier 2 = AC-005..011 (7); Tier 3 = AC-012, 013, 014, 014b (4); Tier 4 = AC-015, 016 (2); Tier 5 = AC-017, 018 (2); Tier 6 = AC-019..022 (4). Total **23**.

**Positive finding on the same axis (AC ⇄ RGT bijection):** the RG list is otherwise sound. Declared "**26 failing tests**" matches RG-001..RG-026 exactly. Every one of the 23 ACs has at least one covering RGT, and every RGT maps to a real AC (AC-016 → RG-016 + RG-017; AC-019 → RG-020..023; AC-001 and AC-011 share RG-001). No orphan RGTs.

**Routing:** story-writer.

---

### F-WASE-P64-MED-008 — Four of seven perimeter stories pin BC versions as the literal word "current", defeating POL-23's version-bump sibling-grep gate (pattern flag)
**Artifact/anchor:** §Behavioral Contracts tables, Version column: `S-WAVE-A-CYBERINT-PATCH-001` (BC-2.16.009 "current"), `S-WAVE-A-CYBERINT-SPEC-001` (BC-2.16.009 "current"), `S-ADR054-WAVE-A-001` (BC-2.16.009 "current", BC-2.01.017 "current"), `S-ADR055-WAVE-A-001` (all five rows "current")
**Confidence:** HIGH

`S-WAVE-A-ENGINE-001` and `S-WAVE-A-MCP-001` pin real versions (v1.26 / v1.10 / v1.19 / v1.15 and v1.6 / v1.19 respectively — all verified correct against disk). The other four use "current". POL-23 (`bc_version_bump_sibling_grep_gate`, HIGH) works by grepping for the old version string when a BC bumps; a story pinning "current" is invisible to that sweep and silently claims currency against whatever the BC happens to be at read time. This is precisely how CRIT-001 and HIGH-005 survived: S-ADR054-WAVE-A-001 pins BC-2.16.009 as "current" while its AC-003 reflects no BC version that has ever existed.

Same-layer sibling inconsistency, blast radius 4 → HIGH per the partial-fix regression discipline; recorded at MED because the remedy is mechanical and no behavioral claim is wrong on its face.

**Routing:** story-writer (pin explicit versions in all four).

---

### F-WASE-P64-MED-009 — S-ADR055-WAVE-A-001 body BC table lists BC-2.16.002; frontmatter `behavioral_contracts:` omits it (POL-8)
**Artifact/anchor:** `S-ADR055-WAVE-A-001` frontmatter `behavioral_contracts:` (`BC-2.16.009`, `BC-2.16.001`, `BC-2.16.007`, `BC-2.16.008`) vs §Behavioral Contracts table, which adds a `BC-2.16.002` row ("Canonical Structured Event Catalog; PO-001 catalog row for spec.validation_warning")
**Confidence:** HIGH

BC-2.16.002 is also referenced by AC-010 (SAP-1 blocking gate) and PO-001. POL-8 (`bc_array_changes_propagate_to_body_and_acs`, HIGH) requires the two to agree bidirectionally; POL-14 auto-promotion at merge also keys off the frontmatter array, so BC-2.16.002 would not be swept at merge.

**Routing:** story-writer.

*(Related, non-blocking: `spec.validation_warning` is confirmed absent from BC-2.16.002 today — it appears only in ADR-055 §D6, this story, and SESSION-HANDOFF. The story handles this correctly: AC-010 marks it a "SAP-1 blocking gate" and PO-001 marks it a PR-merge blocker. No finding.)*

---

### F-WASE-P64-MED-010 — S-WAVE-A-ENGINE-001's `blocks:` justification for CYBERINT-SPEC-001 asserts a co-land requirement that both Cyberint stories explicitly disclaim
**Artifact/anchor:** `S-WAVE-A-ENGINE-001` frontmatter `# blocks anchor justifications:` → `S-WAVE-A-CYBERINT-SPEC-001` entry ("The Cyberint spec migration story MUST add `header_scheme = "cookie:<name>"` to that file before or in the same batch as this story merging — **the stories must co-land**"); and the `blocks:` inline comment on the same entry ("cyberint.sensor.toml must gain header_scheme before engine story lands, **or vice versa**")
**Confidence:** HIGH

Three-way contradiction with the two stories that own the constraint:
- `S-WAVE-A-CYBERINT-PATCH-001` §MERGE-GATE-ENGINE-001: "**Do not add this story's merge gate to `S-WAVE-A-CYBERINT-SPEC-001`** (the full dual-surface migration) — the large migration is independent ADR-053 work and **must NOT be forced into the atomic merge**."
- `S-WAVE-A-CYBERINT-SPEC-001` §Scheduling Note (No Co-land Constraint): "This story does **NOT** need to co-land atomically with `S-WAVE-A-ENGINE-001`."

The ENGINE-001 text is stale post-split (it predates PATCH-001 being carved out) and its "or vice versa" makes a normative frontmatter justification explicitly ambiguous about ordering.

**Routing:** story-writer + state-manager (rewrite the justification to reflect the split; resolve "or vice versa").

---

### F-WASE-P64-MED-011 — S-WAVE-A-CYBERINT-SPEC-001 AC-006 points at the wrong task for the assets-table derivation requirements
**Artifact/anchor:** `S-WAVE-A-CYBERINT-SPEC-001` AC-006 final bullet ("table structure derived from the Cyberint assets OpenAPI; **see Task T-06 for derivation requirements**")
**Confidence:** HIGH

T-06 is "Replace `ac_6_cursor_pagination.rs` with `ac_6_page_size_pagination.rs`". The assets-table derivation requirements are in **T-03** ("Author cyberint-assets.sensor.toml"). Mis-anchored intra-document reference; POL-21-adjacent.

**Routing:** story-writer.

---

### F-WASE-P64-MED-012 — S-ADR054-WAVE-A-001 AC-002 declares `Option<String>` token-exchange fields; ADR-054 §D11 specifies empty-string defaults, and Rule 10(d) needs absence detection
**Artifact/anchor:** `S-ADR054-WAVE-A-001` AC-002 field list ("`credential_body_field: Option<String>`, `token_response_path: Option<String>`, `expiry_field: Option<String>`, `expiry_mode: Option<ExpiryMode>`"); ADR-054 §D11 `AuthAcquisitionConfig` constructors row ("`credential_body_field`, `token_response_path`, and `expiry_field` **default to empty string**"); BC-2.16.009 Rule 10(d) ("One `E-SPEC-028` is emitted **per absent field**")
**Confidence:** MEDIUM

If the fields are `String` with an empty-string default (D11's constructor description), an absent TOML key and an explicitly-empty value deserialize identically, and Rule 10(d)'s per-field absence detection becomes an `is_empty()` check that neither ADR-054 §D3 nor BC-2.16.009 Rule 10(d) specifies. If they are `Option<String>` (AC-002), D11's "default to empty string" constructor language is wrong. One of the two must change; the decision determines whether E-SPEC-028(d) is implementable as specified.

**Routing:** architect (ratify the Rust encoding in ADR-054 §D3 or §D11 and state the absence-detection predicate), then story-writer aligns AC-002.

---

### F-WASE-P64-MED-013 — S-ADR054-WAVE-A-001 AC-006 cites `E-SPEC-009` for the auth_plugin/oauth2 rejection; that code is "Duplicate sensor_id across spec files"
**Artifact/anchor:** `S-ADR054-WAVE-A-001` AC-006 final paragraph ("without **E-SPEC-009** (auth_plugin with oauth2_client_credentials is rejected by D10(b))")
**Confidence:** HIGH

BC-2.16.009 §Error Conditions: `E-SPEC-009` = "Duplicate `sensor_id` across spec files | Second file rejected; first wins". The correct code for the auth_plugin-on-declarative-auth_type rejection is `E-SPEC-028` sub-condition (b). POL-22.

**Routing:** story-writer.

---

### F-WASE-P64-MED-014 — S-WAVE-A-CYBERINT-SPEC-001 T-03 points the implementer at two directories that do not contain the assets OpenAPI, then licenses a `# TBD`-stubbed sensor spec as the fallback
**Artifact/anchor:** `S-WAVE-A-CYBERINT-SPEC-001` §Tasks T-03 ("derive from the Cyberint Assets OpenAPI (located in `crates/prism-dtu-cyberint/` or the research directory — search for `cyberint_assets_openapi` or equivalent file)" … "**If the assets OpenAPI file is not present in the codebase, document the table structure as a stub with explicit `# TBD: requires assets OpenAPI grounding` comments on each placeholder column**")
**Confidence:** HIGH

The file **does** exist, at `.factory/reference/api-specs/cyberint_assets_openapi_06.20.2026.json` — the exact path ADR-053 §Context names ("`cyberint_assets_openapi_06.20.2026.json` (title 'FastAPI', v0.1.0, server `/asset-configuration`, 5 paths)") and lists again in its reference section. It is in neither of T-03's two named locations. Because the pointer is wrong, the TBD escape hatch is the path of least resistance, and the fallback would ship a sensor spec whose columns are placeholders — while AC-006 accepts it ("At least one `[[tables]]` block"). Under CLAUDE.md Canonical Principle Rules 1 and 6, a reachable TBD-stub escape hatch in a spec artifact is a defect, not a hedge.

**Routing:** story-writer (cite the exact repo-relative path; delete the TBD-stub fallback).

---

### F-WASE-P64-MED-015 — ADR-055 is `status: proposed` with `anchor_stories: []` while its P0 anchor story is authored and a separate P0 gate is conditioned on it
**Artifact/anchor:** `ADR-055-validate-sensor-spec-production-wiring.md` frontmatter `status: proposed`, `anchor_stories: []`, §Status ("Ready for human review and story-writer dispatch"); `S-ADR055-WAVE-A-001` §Authority ("ADR-055 v1.0 … is the authoritative design document")
**Confidence:** HIGH

The story-writer dispatch has already occurred (S-ADR055-WAVE-A-001 v1.0, P0, 8 points, `tdd_mode: strict`), so §Status's "Ready for … dispatch" and the empty `anchor_stories` are both stale on the day they were written. Separately, `S-WAVE-A-CYBERINT-PATCH-001` AC-002 makes Rule 9's liveness conditional on this story (see HIGH-002) — routing a P0 boot-critical gate through an unaccepted ADR. POL-4 / POL-5 anchoring; POL-15 adjacency.

**Routing:** architect (populate `anchor_stories: [S-ADR055-WAVE-A-001]`; advance status or record the human-review gate explicitly).

---

### F-WASE-P64-MED-016 — Six of seven perimeter stories are `tdd_mode: strict` with no enumerated Red Gate test list and no BC-5.38.001 density check `[process-gap]`
**Artifact/anchor:** §Tasks sections of `S-WAVE-A-CYBERINT-PATCH-001` (T-01..T-03), `S-WAVE-A-CYBERINT-SPEC-001` (T-01..T-08), `S-WAVE-A-MCP-001` (T-01..T-05), `S-ADR054-WAVE-A-001` (T-01..T-09), `S-ADR055-WAVE-A-001`, `S-WAVE-A-ARMIS-REMEDIATION-001`; contrast `S-WAVE-A-ENGINE-001` §Tasks → "Red Gate tests (to be written by test-writer BEFORE implementation)" RG-001..RG-026 + "**Red Gate density check** (BC-5.38.001): **26 failing tests** before implementation begins"
**Confidence:** MEDIUM

All seven carry `tdd_mode: strict`; all seven are `status: draft`. Only ENGINE-001 enumerates named Red Gate tests with an AC mapping and a density count. The other six embed test-writing inside implementation tasks (e.g. MCP-001 T-04 "Write AC-001/AC-002/AC-003 wire-shape tests" sits after T-01/T-02 read-and-decide tasks), which inverts the red-then-green ordering that prism's Phase-3 sub-workflow (stubs → failing tests → TDD green) requires. This is how HIGH-003's dropped obligation became invisible: with no RG list, there is no artifact in which the missing E-SPEC-027 wire assertion would show up as an absent row.

Same-layer sibling inconsistency at blast radius 6. Tagged `[process-gap]`: no hook or validator gates a `tdd_mode: strict` story on the presence of an enumerated Red Gate list + BC-5.38.001 density check before `status: ready`. Intent caveat: if the convention is that RG enumeration lands at the draft→ready transition, ENGINE-001 is the outlier rather than the model — the orchestrator should adjudicate which is normative and then codify it.

**Routing:** story-writer (add RG lists), plus a codification follow-up for the missing gate.

---

### F-WASE-P64-MED-017 — Six story-local edge cases in S-WAVE-A-ENGINE-001 have no Red Gate test, and AC-001's universal quantifier is exercised for one auth_type
**Artifact/anchor:** `S-WAVE-A-ENGINE-001` §Edge Cases rows EC-001, EC-002, EC-004, EC-007, EC-010, EC-014; §Tasks Red Gate list RG-001..RG-026; AC-001
**Confidence:** HIGH

Unmapped ECs: EC-001 (`header_scheme = ""`), EC-002 (`"BEARER"` wrong case), EC-004 (`" bearer"` leading space), EC-007 (`cookie:tok` + `bearer_static` coherence), EC-010 (`cookie:name` + `oauth2_client_credentials` coherence), EC-014 (oauth2 + absent → loads OK). EC-011/012/013 are covered transitively by AC-010/AC-014b/AC-016 and EC-006 by AC-009; the six above are not.

Separately, AC-001 asserts "For **all** `auth_type` values OTHER than `cookie_roundtrip`, `None` passes spec-load validation silently", but its only test (RG-001, `test_rule9_absent_header_scheme_bearer_static_passes`) exercises `bearer_static`. Absence path A is unverified for `oauth2_client_credentials`, `api_key`, and `custom_via_plugin` — and EC-014 (the oauth2 case) is one of the six unmapped rows. The coherence matrix's five-arm exhaustiveness is the stated reason for T-B02's no-wildcard `match auth_type` (Q4 ruling); leaving three arms' absence path untested undercuts that design.

**Routing:** story-writer (add RGTs for the six ECs; parameterize RG-001 across the four non-cookie auth_types).

---

## Observations

### F-WASE-P64-LOW-001 — S-WAVE-A-MCP-001 §Architecture Compliance Rule 2 cites BC-2.10.007 "as of v1.18"; the story pins v1.19 and the file is v1.19
**Artifact/anchor:** `S-WAVE-A-MCP-001` §Architecture Compliance Rules item 2 ("The catch-all is reserved for FUTURE/unknown variants only (ZERO currently-known variants fall to it **as of v1.18**)") vs §Authority and §Behavioral Contracts (both `BC-2.10.007 v1.19`) and the file's own `version: "1.19"`. Records-tier. **Routing:** story-writer.

### F-WASE-P64-LOW-002 — S-WAVE-A-CYBERINT-SPEC-001's Header-Scheme Sweep Report scope is narrower than the workspace-wide conclusion it supports
**Artifact/anchor:** `S-WAVE-A-CYBERINT-SPEC-001` §Header-Scheme Sweep Report opening sentence ("A sweep of all spec files in `crates/prism-sensors/specs/` and `.prism/specs/sensors/`") and §Narrative clause (d) ("**no spec file in the workspace** carries the stale `auth_type = "cookie_roundtrip"` + absent `header_scheme` combination")
**Confidence:** HIGH

I independently re-ran the sweep. The report's per-file rows are all correct, including the `.prism/specs/sensors/` row ("directory does not exist on disk" — confirmed). But `crates/prism-bin/fixtures/sensors/` was not in scope; it contains `test-sensor-with-cred-refs.sensor.toml`. Verified benign (`auth_type = "api_key"`, no `header_scheme` → absence path A), so "Result: Only `cyberint.sensor.toml` is affected" holds. Recording because the scope statement does not support the workspace-wide claim, and a future sensor fixture added under `fixtures/` would not be caught by a re-run of this sweep as scoped.

Also verified in the same pass: `SensorSpec` deserialization carries **no** `#[serde(deny_unknown_fields)]` anywhere in `crates/prism-spec-engine/src/`, so CYBERINT-PATCH-001 EC-001's claim ("field is parsed as an unknown field or ignored; spec loads without error") is **correct** — adding `header_scheme` before ENGINE-001 lands is safe. No finding there.

**Routing:** story-writer (widen the stated sweep scope or narrow the Narrative claim).

### F-WASE-P64-OBS-001 — ADR frontmatter schema is inconsistent across the perimeter `[process-gap]`
ADR-053 and ADR-054 carry **no** `anchor_stories` key at all; ADR-050, ADR-051, ADR-052, ADR-055 carry it (three of the four with `[]`). No validator enforces either the key's presence or its population. With `anchor_stories` empty or absent on all five Wave-A-relevant ADRs, ADR→story traceability in this perimeter is one-directional (stories cite ADRs; ADRs cite no stories). **Routing:** codification follow-up + architect.

### F-WASE-P64-OBS-002 — VP-160 defers the error-message formatting invariants to an unnamed future property
**Artifact/anchor:** VP-160 §Property Statement → "**Scope note:**" ("The 64-codepoint echo cap and `\xNN` CTL-escaping … are outside the scope of this VP and **belong to a separate property covering error message formatting**")

No VP ID, no story anchor, no VP-INDEX placeholder row. Combined with CRIT-005 (no AC/RGT for the same two mechanisms in the anchor story), the CWE-400 cap and the CWE-117 escaping currently have **zero** verification obligation anywhere in the perimeter — neither a Red Gate test nor a VP. The deferral is narrated, not anchored (CLAUDE.md Canonical Principle Rule 3). **Routing:** architect (either extend VP-160's scope or register the successor VP with an anchor story).

---

## Novelty Assessment

**Novelty: HIGH.** The findings cluster in a region a records/index-currency pass structurally cannot see: **story→ADR/BC semantic re-grounding**. Every version pin in the perimeter is correct on disk, every changelog is ordered, the error-taxonomy templates are byte-identical to the BC across all four E-SPEC-027 sites, the AC⇄RGT bijection in the anchor story is complete, and the tchar charset is consistent at 15 special characters / 77 total in every live-body site checked (BC-2.16.009 §Syntactic check, §Error message (syntactic), §Error Conditions E-SPEC-027 template (a), error-taxonomy E-SPEC-027 template (a), S-WAVE-A-ENGINE-001 AC-005/AC-007/AC-021 and T-B02's `matches!` arm, VP-160 §Property Statement, VP-INDEX VP-160 row) — the records dimension is **clean**, and the RFC 9110 §5.6.2 arithmetic (26+26+10+15=77) checks out everywhere.

What is not clean is that four of the seven stories were stubbed against earlier ADR/BC states and never re-derived: S-ADR054-WAVE-A-001 reflects no version of §D10 that has ever existed (CRIT-001, CRIT-002, HIGH-005, MED-012, MED-013); S-WAVE-A-ARMIS-REMEDIATION-001 treats already-ratified ADR-054 §D3 values as open TBDs and gets three of five wrong (CRIT-004, HIGH-006, HIGH-007); S-WAVE-A-CYBERINT-SPEC-001 declares a pagination contract the engine grammar cannot emit (CRIT-003, MED-011, MED-014); and BC-2.16.009 v1.26's three security mitigations landed with no story-side AC/RGT obligation (CRIT-005, a POL-38 violation). Three findings turned on reading Rust rather than prose — `build_paged_url_impl`, `truncate_at_char_boundary`, `parse_and_validate_spec_toml`'s actual call sequence — and two on filesystem reality (`prism-armis-endpoint-plan.md` absent; `cyberint_assets_openapi_06.20.2026.json` present but mis-pointed).

**Standing probes.** SAP-1: swept all `event_type =` sites under `crates/`; sampled 18 distinct values against BC-2.16.002 §Postconditions — all present. `spec.validation_warning` is absent from the catalog but is not yet emitted, and S-ADR055-WAVE-A-001 AC-010 + PO-001 correctly gate it as a PR-merge blocker. **No SAP-1 finding.** SAP-2: no sensor TOML in the perimeter has changed on disk (all four canonical specs unmodified at the frozen HEAD), so no TOML↔DTU column delta exists to score; the two stories that will author new tables carry SAP-2 language (CYBERINT-SPEC-001 T-03 explicitly, "Missing-column-in-DTU = P1 CRITICAL"), though T-03's grounding pointer is wrong (MED-014). **No P1 SAP-2 parity finding.** SAP-3: reachability is well handled in ENGINE-001 (AC-020/RG-024 at the `prism-spec-engine` public API, T-B02's Q4 defense-in-depth comment requirement) and in ADR-055 §Acceptance shape item 6; it is **broken** in S-ADR054-WAVE-A-001 AC-003 (CRIT-002) and **absent** for the deferred MCP wire arm (HIGH-003). SID-1: no `#[ignore]`-satisfiable AC found. SID-2: S-WAVE-A-MCP-001 AC-004 correctly demands a composed-output assertion; CRIT-004's missing `header_scheme = "raw"` is the composed-output failure this perimeter *would* have shipped (correct `[auth_acquisition]` components, wrong emitted `Authorization` header).

---

## Severity Breakdown

| Severity | Count | IDs |
|---|---|---|
| CRIT | 5 | CRIT-001 … CRIT-005 |
| HIGH | 8 | HIGH-001 … HIGH-008 |
| MED | 17 | MED-001 … MED-017 |
| LOW | 2 | LOW-001, LOW-002 |
| OBS | 2 | OBS-001, OBS-002 |
| PROCESS-GAP tags | 2 | MED-016, OBS-001 |
| **Total** | **34** | |

```
CLEAN (strict): no
CLEAN (PR-merge): no
```

BC-5.39.001 streak resets to **0/3**. Five CRIT + eight HIGH findings block the PR-merge gate independently of the strict criterion.

**Recommended routing summary:** story-writer owns 18 findings (the dominant class — stories not re-grounded against their ADRs/BCs); architect owns 6 (ADR-054 §D10/§D3/§D11 story alignment, ADR-053 Armis grounding source, ADR-055 status/anchors, VP-160 anchor + successor property, Cyberint pagination-grammar decision); product-owner owns 4 (BC-2.16.009 §Integration function, §Invariants scoping, VP-160 row count, E-SPEC-027 template (a) length clause); state-manager owns 2 (VP-INDEX VP-160 anchor cell, STORY-INDEX dependency edges). **TD-VSDD-096 records-only micro-burst does NOT apply** — the majority of findings touch mechanism, contract, and correctness. Full cascade ceremony required.
