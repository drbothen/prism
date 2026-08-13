---
document_type: adversarial-review-pass
story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pass: 14
phase: LOCAL
frozen_head: "(story v1.14 + code a1864d3eb + spec leg: BC-2.16.002 v2.19 / BC-2.08.002 v1.6 / BC-2.01.013 v1.18 / BC-2.01.010 v1.6 / BC-2.16.014 v1.22 / BC-2.19.001 v2.4 / ADR-050 v2.3 / error-taxonomy v2.74)"
new_feature_head: "a1864d3eb (UNCHANGED — records-only burst; no code commits)"
verdict_strict: "NO"
verdict_pr_merge: "NO"
findings_count: 2
streak_before: 0
streak_after: 0
closed_by: "F-1 [HIGH] delegation-framing residual (partial-fix miss from pass-13): AC-UA-001 trace note + T-B01 still named `build_http_client_with_custom_timeout` as the DeclarativeHttpAuthProvider UA delegation vehicle — pass-13 corrected the 3 summary surfaces (`# BC status` comment + §Authority table + §Behavioral Contracts table) but AC-UA-001 and T-B01 were missed; cross-crate impossible (prism-bin cannot propagate UA to prism-spec-engine at call time); in-document contradiction vs already-correct §Authority/§BC-table/frontmatter. CLOSED in story v1.15: AC-UA-001 trace note + T-B01 both restated to independent prism-spec-engine §build_http_client_with_timeout sibling; exhaustive grep zero residuals. F-2 [OBS] ADR-050 §Status narrative listed v2.3 before v2.2 (descending order in ascending-history section). CLOSED: architect reordered §Status entries ascending (v2.1→v2.2→v2.3); no version bump — L1/L7 satisfied (version: stays v2.3; top changelog row remains v2.3; §Status section is narrative prose, not a changelog row)."
timestamp: 2026-08-13
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — LOCAL Adversary Pass 14

**CLEAN(strict): NO | CLEAN(PR-merge): NO**
**BC-5.39.001 streak: RESET 0/3** (records-only burst; code HEAD a1864d3eb unchanged; streak resets due to HIGH finding)
**2 findings: F-1 [HIGH] + F-2 [OBS] BOTH CLOSED (story v1.14→v1.15; ADR-050 §Status reordered; records-only; TD-VSDD-096)**

**Context:** Pass-14 ran on the post-D-2127 frozen HEAD (story v1.14 + code a1864d3eb). This pass found 2 records-tier findings. F-1 [HIGH] is a partial-fix miss from pass-13: the 3 summary surfaces were corrected (§Authority table, §Behavioral Contracts table, `# BC status` comment) but AC-UA-001 and T-B01 — which contain the mechanism framing in full prose — retained the stale `build_http_client_with_custom_timeout` delegation claim. The claim is cross-crate impossible (prism-bin is a binary crate; prism-spec-engine resolves `build_http_client_with_timeout` at its own call site, not via delegation from prism-bin). F-1 was HIGH because an acceptance criterion and task specification both described a false mechanism. F-2 [OBS] is an ordering defect in ADR-050 §Status narrative. Code HEAD a1864d3eb unchanged throughout. CLEAN(PR-merge)=NO because F-1 is HIGH.

---

## Finding F-1 [HIGH] — CLOSED

**ID:** F-1
**Severity:** HIGH
**Class:** Records-tier delegation-framing residual (partial-fix miss from pass-13; AC + task sections)
**Status:** CLOSED (story v1.15)

**Description:**

Pass-13 (D-2127) corrected 3 BC-2.16.014 summary surfaces that named `build_http_client_with_custom_timeout` (prism-bin) as the DeclarativeHttpAuthProvider UA delegation vehicle. However, two additional locations in story v1.14 retained the same stale mechanism framing:

1. **AC-UA-001** trace note: stated that DeclarativeHttpAuthProvider UA compliance is achieved "via the `build_http_client_with_custom_timeout` delegation chain propagation from prism-bin."
2. **T-B01** (Task specification): stated the implementation mechanism as delegating through `build_http_client_with_custom_timeout` in prism-bin.

Both claims describe a cross-crate-impossible mechanism. `prism-spec-engine` is not called from prism-bin at the point where HTTP clients are constructed; `build_http_client_with_timeout` in `prism-spec-engine::pipeline` is an independent sibling that sets its own `.user_agent()` call. The mechanism framing in AC-UA-001 and T-B01 contradicted the already-correct §Authority table, §Behavioral Contracts table, and `# BC status` comment that pass-13 had just updated — an in-document self-contradiction at HIGH severity because an acceptance criterion is normative.

**Authoritative source:** BC-2.16.014 v1.22 INV-014-007 (corrected in D-2123); AC-UA-001 body and T-B01 were catching up.

**Fix applied:**

Story v1.14→v1.15:
- AC-UA-001 trace note: restated to `via \`build_http_client_with_timeout\` in \`prism-spec-engine::pipeline\` (independent sibling with its own \`.user_agent()\` call — not propagation from prism-bin; see BC-2.16.014 v1.22 INV-014-007)`
- T-B01 mechanism description: restated to `build_http_client_with_timeout (prism-spec-engine::pipeline)` as the implementation path, removing the prism-bin delegation framing

Exhaustive grep of story v1.15 for `build_http_client_with_custom_timeout` as a delegation vehicle: zero surviving occurrences. All five locations now consistently name the independent prism-spec-engine sibling.

---

## Finding F-2 [OBS] — CLOSED

**ID:** F-2
**Severity:** OBS
**Class:** Records-tier ordering (ADR-050 §Status history ascending-order violation)
**Status:** CLOSED (ADR-050 §Status reordered; no version bump)

**Description:**

ADR-050 §Status narrative history section listed version v2.3 before v2.2, producing a descending-within-ascending ordering inconsistency. The §Status section documents the amendment history in chronological ascending order (v2.0→v2.1→v2.2→v2.3); the v2.3 entry appeared before the v2.2 entry.

Not a content/mechanism defect. Purely a display ordering issue in prose narrative.

**Fix applied:**

Architect reordered §Status entries ascending: v2.1→v2.2→v2.3. No version bump applied — the version: frontmatter remains v2.3 (unchanged); top changelog row remains v2.3; L1 (document version = top changelog row version) satisfied; L7 (changelog descending order) satisfied. §Status is narrative prose, not a changelog row, so L7 does not apply to it directly, but ascending chronological order is the correct convention for history sections.

---

## Code Core Verdict

Pass-14 confirmed code HEAD a1864d3eb is UNCHANGED. All prior code-core verdicts carry forward. Code is CRIT/HIGH/MED-clean on all original story acceptance criteria:

- `http2` feature entries: 3 production crates (prism-spec-engine, prism-sensors, prism-bin) — PASS (unchanged)
- §D6 User-Agent sweep: all 4 sites (§build_http_client_with_timeout prism-spec-engine, §spec_driven_adapter prism-sensors, prism-bin sensor/plugin clients) — PASS (unchanged)
- Non-2xx body snippet byte-cap via §prism_core::sanitize_body_snippet_bytes — PASS (unchanged)
- Error source-chain wiring — PASS (unchanged)
- Error mapping Arm-1 (HttpRequestFailed non-auth) / Arm-2 (AuthRefreshFailed, CookieAuthFailed) / Arm-3 (AllTargetsFailed) — PASS (unchanged)
- E-INFUSE-015 InfusionError::HttpClientBuildFailed wired to 3 callers (§load_spec, §load_spec_with_runtime, §hot_reload) — PASS (unchanged)
- SAP-1: catalog row 91 `fan_out_target_failed` WARN present; zero unregistered `event_type` emissions in diff scope — PASS (unchanged)
- SAP-3: all 13 RGTs load-bearing with public-surface reachability — PASS (unchanged)
- 95/95 non-exhaustive; 5724 tests green — PASS (unchanged)
- Holdout gate compatibility: HS-TLS-XDOME-001/002/003 (hidden; not read) — sealed

---

## TD-VSDD-097 Three-Dimension Sweep

**Dimension 1 — Sibling pair:** AC-UA-001 and T-B01 are within the same story file (DEFECT-ADAPTER-TLS-XDOME-LIVE-001); both corrected in story v1.15 in the same burst. The companion story S-WAVE-A-ENGINE-001 v3.3 does not contain the delegation-vehicle prose (its BC-2.16.014 pin cells carry version numbers, not mechanism descriptions). No twin artifact requires update. **CLEAR.**

**Dimension 2 — Downstream copy target:** AC-UA-001 trace notes and T-B01 task prose are not verbatim copy-sourced into any downstream artifact by a later agent leg. ADR-050 §Status prose is not transcribed verbatim into any other artifact. **CLEAR.**

**Dimension 3 — Mandate anchor:** No new MUST blocks authored in this pass. F-1 is a factual accuracy correction; F-2 is an ordering correction. Neither introduces new normative obligations. **CLEAR.**

---

## Convergence Trajectory

Pass 1: 5 findings | Pass 2: 6 | Pass 3: 1 | Pass 4: 3 | Pass 5: 2 | Pass 6: 3 | Pass 7: 1 | Pass 8: 4 | Pass 9: 2(LOW) | Pass 10: 2(LOW+OBS) | Pass 11: 2(F-2 HUMAN-DIRECTED) | Pass 12: 1(MED) | Pass 13: 2(MED+LOW) | Pass 14: 2(HIGH+OBS)

Full trajectory: 5→6→1→3→2→3→1→4→2(LOW)→2(LOW+OBS)→2(F-2 HUMAN-FIX)→1(MED)→2(MED+LOW)→2(HIGH+OBS)

BC-5.39.001 streak: 0/3. NEXT: strict LOCAL adversary pass-15 on frozen HEAD a1864d3eb + story v1.15.
