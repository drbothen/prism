---
document_type: adversarial-review-pass
story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pass: 13
phase: LOCAL
frozen_head: "(story v1.13 + code a1864d3eb + spec leg: BC-2.16.002 v2.19 / BC-2.08.002 v1.6 / BC-2.01.013 v1.18 / BC-2.01.010 v1.6 / BC-2.16.014 v1.22 / BC-2.19.001 v2.4 / ADR-050 v2.3 / error-taxonomy v2.74)"
new_feature_head: "a1864d3eb (UNCHANGED — records-only burst; no code commits)"
verdict_strict: "NO"
verdict_pr_merge: "NO"
findings_count: 2
streak_before: 0
streak_after: 0
closed_by: "F-1 [MED] DD-9 un-propagated delegation-vehicle: 3 BC-2.16.014 summary surfaces in story v1.13 named `build_http_client_with_custom_timeout` as the DeclarativeHttpAuthProvider UA delegation vehicle — stale pre-DD-9 text not updated when BC-2.16.014 v1.22 pin was propagated in D-2123. CLOSED in story v1.14: (1) `# BC status` frontmatter comment; (2) §Authority table BC-2.16.014 row; (3) §Behavioral Contracts table BC-2.16.014 row — all 3 corrected to `build_http_client_with_timeout` in `prism-spec-engine::pipeline` (independent sibling with own `.user_agent()` call), matching BC-2.16.014 v1.22 INV-014-007 + AC-UA-001/T-B01. Code HEAD a1864d3eb UNCHANGED. F-2 [LOW] stale `# BC status` header labels: `(as of 2026-08-13 amendments, v1.2)` corrected to `(current, v1.14)`; BC-2.16.002 catalog label `v1.64` corrected to `v1.63`. CLOSED in story v1.14."
timestamp: 2026-08-13
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — LOCAL Adversary Pass 13

**CLEAN(strict): NO | CLEAN(PR-merge): NO**
**BC-5.39.001 streak: RESET 0/3** (records-only burst; code HEAD a1864d3eb unchanged; streak resets due to MED finding)
**2 findings: F-1 [MED] + F-2 [LOW] BOTH CLOSED (story v1.13→v1.14; records-only; TD-VSDD-096)**

**Context:** Pass-13 ran on the post-D-2126 frozen HEAD (story v1.13 + code a1864d3eb). This pass found 2 records-tier findings: F-1 [MED] un-propagated DD-9 delegation-vehicle correction at 3 body summary surfaces, and F-2 [LOW] stale version labels in the `# BC status` frontmatter comment section. Both are story-only records defects. Code HEAD a1864d3eb unchanged throughout. CLEAN(PR-merge)=NO because MED severity.

---

## Finding F-1 [MED] — CLOSED

**ID:** F-1
**Severity:** MED
**Class:** Records-tier propagation gap (stale function name at 3 BC-2.16.014 summary surfaces)
**Status:** CLOSED (story v1.14)

**Description:**

D-2123 (DD-9) corrected BC-2.16.014 v1.22 INV-014-007 §Invariants to accurately describe the two-path production/test context for DeclarativeHttpAuthProvider UA delegation. INV-014-007 was corrected to distinguish the production path via `build_http_client_with_timeout` in `prism-spec-engine::pipeline` (independent sibling with its own `.user_agent()` call) from `build_http_client_with_custom_timeout` in prism-bin (a separate function).

Story v1.13 was updated in D-2123 for the BC-2.16.014 v1.22 version pin, but 3 body surfaces that described the delegation mechanism were not corrected — they still named `build_http_client_with_custom_timeout` as the UA delegation vehicle:

1. `# BC status` frontmatter comment: stated "via `build_http_client_with_custom_timeout delegation chain`"
2. §Authority table BC-2.16.014 row: stated "via `build_http_client_with_custom_timeout` delegation"
3. §Behavioral Contracts table BC-2.16.014 row: stated "via `build_http_client_with_custom_timeout` delegation chain"

Authoritative source: BC-2.16.014 v1.22 INV-014-007 + AC-UA-001/T-B01 (already correct since D-2123). The correct description is `build_http_client_with_timeout` in `prism-spec-engine::pipeline` (independent sibling with own `.user_agent()` call — not propagation from prism-bin).

**Fix applied:**

Story v1.13→v1.14:
- `# BC status` comment: corrected to `via \`build_http_client_with_timeout (prism-spec-engine::pipeline)\`, an independent sibling with its own \`.user_agent()\` call (not propagation from prism-bin)`
- §Authority table BC-2.16.014 row: corrected to `via \`build_http_client_with_timeout\` in \`prism-spec-engine::pipeline\` (independent sibling with its own \`.user_agent()\` call — not propagation from prism-bin)`
- §Behavioral Contracts table BC-2.16.014 row: corrected to `via \`build_http_client_with_timeout\` in \`prism-spec-engine::pipeline\` (ADR-050 §D6 propagation — independent sibling with its own \`.user_agent()\` call)`; "beyond the `build_http_client_with_custom_timeout` change" → "beyond the `build_http_client_with_timeout` change in `prism-spec-engine::pipeline`"

Final grep confirms no other location names `build_http_client_with_custom_timeout` as the DeclarativeHttpAuthProvider delegation vehicle in the story.

---

## Finding F-2 [LOW] — CLOSED

**ID:** F-2
**Severity:** LOW
**Class:** Records-tier stale version labels
**Status:** CLOSED (story v1.14, same fix as F-1)

**Description:**

Story v1.13 `# BC status` comment section contained two stale version labels:

1. Header label: `(as of 2026-08-13 amendments, v1.2)` — should reflect current story version `(current, v1.14)`
2. BC-2.16.002 catalog label: `v1.64` — the correct catalog label is `v1.63` (consistent with AC-SAP1-001 body reference at §Group D and BC-2.16.002 v2.19 as authored in D-2112 with catalog label v1.63)

**Fix applied:**

Story v1.14:
- Header label `(as of 2026-08-13 amendments, v1.2)` → `(current, v1.14)`
- BC-2.16.002 catalog label `v1.64` → `v1.63`

---

## Code Core Verdict

Pass-13 confirmed code HEAD a1864d3eb is UNCHANGED. All pass-12 code-core verdicts carry forward. Code is CRIT/HIGH/MED-clean on all original story acceptance criteria:

- `http2` feature entries: 3 production crates (prism-spec-engine, prism-sensors, prism-bin) — PASS (unchanged)
- §D6 User-Agent sweep: all 4 sites — PASS (unchanged)
- Non-2xx body snippet byte-cap via §prism_core::sanitize_body_snippet_bytes — PASS (unchanged)
- Error source-chain wiring — PASS (unchanged)
- Error mapping Arm-1/Arm-2/Arm-3 — PASS (unchanged)
- E-INFUSE-015 InfusionError::HttpClientBuildFailed wired to 3 callers — PASS (unchanged)
- SAP-1: catalog row 91 `fan_out_target_failed` present; zero unregistered `event_type` emissions — PASS (unchanged)
- SAP-3: all 13 RGTs load-bearing — PASS (unchanged)
- 95/95 non-exhaustive; 5724 tests green — PASS (unchanged)

---

## TD-VSDD-097 Three-Dimension Sweep

**Dimension 1 — Sibling pair:** BC-2.16.014 v1.22 INV-014-007 (the authoritative source) was already corrected in D-2123 (DD-9). No BC-2.16.014 amendment is required here; the story body surfaces were catching up to the already-corrected BC. S-WAVE-A-ENGINE-001 also carries a BC-2.16.014 v1.22 pin (correct; no delegation-vehicle mechanism prose in its pin cells). No other story names `build_http_client_with_custom_timeout` as the UA delegation vehicle. **CLEAR.**

**Dimension 2 — Downstream copy target:** The `# BC status` comment section and §Authority/§Behavioral Contracts table cells are not verbatim copy-sourced into any downstream artifact by a later agent leg. **CLEAR.**

**Dimension 3 — Mandate anchor:** No new MUST blocks authored in this pass. F-1 is a factual accuracy correction (wrong function name reference); F-2 is a stale label fix. Neither introduces new normative obligations. **CLEAR.**

---

## Convergence Trajectory

Pass 1: 5 findings | Pass 2: 6 | Pass 3: 1 | Pass 4: 3 | Pass 5: 2 | Pass 6: 3 | Pass 7: 1 | Pass 8: 4 | Pass 9: 2(LOW) | Pass 10: 2(LOW+OBS) | Pass 11: 2(F-2 HUMAN-DIRECTED) | Pass 12: 1(MED) | Pass 13: 2(MED+LOW)

Full trajectory: 5→6→1→3→2→3→1→4→2(LOW)→2(LOW+OBS)→2(F-2 HUMAN-FIX)→1(MED)→2(MED+LOW)

BC-5.39.001 streak: 0/3. NEXT: strict LOCAL adversary pass-14 on frozen HEAD a1864d3eb + story v1.14.
