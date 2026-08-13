---
document_type: adversarial-review-pass
story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pass: 5
phase: LOCAL
frozen_head: "(story v1.4 + code commits e21b0cdc3 / dff20e910 / 8f6b5e131 / 67638ce07 + spec leg: BC-2.16.002 v2.17 / BC-2.08.002 v1.6 / BC-2.01.010 v1.6 / ADR-050 v2.2)"
verdict_strict: "NO"
verdict_pr_merge: "NO"
findings_count: 2
streak_before: 0
streak_after: 0
closed_by: "TD-VSDD-096 records-only micro-burst — story v1.4→v1.5 (F-P5-MED-001 duplicate holdout_scenarios frontmatter key removed — kept populated [HS-TLS-XDOME-001/002/003]; F-P5-LOW-001 AC-ERR-001/002 call-signature prose corrected to 3-arg form) / STORY-INDEX v2.782→v2.783"
timestamp: 2026-08-13
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — LOCAL Adversary Pass 5

**CLEAN(strict): NO | CLEAN(PR-merge): NO**
**BC-5.39.001 streak: RESET 0/3** (F-P5-MED-001 is [MED] which blocks both strict and PR-merge gate)
**All 2 findings CLOSED via TD-VSDD-096 records-only micro-burst**

---

## Code Core Verdict

Pass-5 adversarial review confirmed the frozen HEAD CRIT/HIGH-clean. Comprehensive PASS list confirmed:

- Cargo `http2` feature entries: 3 production crates (prism-spec-engine, prism-sensors, prism-bin) — PASS
- §D6 User-Agent sweep: `build_http_client_with_custom_timeout §spec_driven_adapter` + both PluginRuntime builders + `build_http_client_with_timeout §pipeline` — PASS (4 sites)
- Error source-chain wiring (`prism_core::sanitize_body_snippet`, non-2xx body capture) — PASS
- Non-2xx capture + error mapping (`HttpError{4xx}` → `auth_valid: false` BC-2.08.002 EC-08-006) — PASS
- SAP-1: catalog row 91 `fan_out_target_failed` WARN registered in BC-2.16.002 §Postconditions — PASS
- SAP-3: reachability of all 12 RG tests confirmed from public surface — PASS
- All 12 RGTs (RG-001..RG-012) each exercise a distinct production code path with real assertions — PASS
- POL-7 BC titles verbatim in story §Behavioral Contracts table (post-pass-4 fix) — PASS
- All BC/ADR pins current (post-pass-4 fix: BC-2.16.002 v2.17 / BC-2.08.002 v1.6 / BC-2.01.010 v1.6 / ADR-050 v2.2) — PASS
- Wire-shape assertions present on MCP-visible surfaces — PASS
- No new `tracing::*!(event_type=…)` emissions without BC-2.16.002 catalog rows — PASS

No CODE findings.

---

## Findings

### F-P5-MED-001 [MED] — Duplicate `holdout_scenarios:` frontmatter key in story

**Status: CLOSED**

**Description:** The story frontmatter contained two separate `holdout_scenarios:` keys. YAML parsers resolve duplicate keys by taking the last value. The first (populated) key `holdout_scenarios: [HS-TLS-XDOME-001, HS-TLS-XDOME-002, HS-TLS-XDOME-003]` was shadowed by the second (unpopulated / empty) key that appeared lower in the frontmatter block. This would silently cause the story-level holdout gate to skip the three registered scenarios — a BLOCKING gate bypass (see CLAUDE.md §Pipeline Authority holdout gate rule).

**Fix applied:** Story DEFECT-ADAPTER-TLS-XDOME-LIVE-001 v1.4→v1.5 — duplicate `holdout_scenarios:` key removed; the populated key `[HS-TLS-XDOME-001, HS-TLS-XDOME-002, HS-TLS-XDOME-003]` is the sole surviving entry.

**Classification:** Records-tier (frontmatter key deduplication). Mechanism impact: holdout gate would have silently skipped scenarios; fixed before any holdout dispatch occurs.

---

### F-P5-LOW-001 [LOW] — AC-ERR-001/002 call-signature prose cites 2-arg form instead of 3-arg

**Status: CLOSED**

**Description:** Acceptance criteria AC-ERR-001 and AC-ERR-002 described the error-mapping function call using a 2-argument signature in the narrative prose. The actual production function in `prism-spec-engine` requires 3 arguments. The prose drift was records-tier (no code ambiguity; the RG tests invoke the correct 3-arg form), but it would cause implementer confusion at TDD time.

**Fix applied:** Story DEFECT-ADAPTER-TLS-XDOME-LIVE-001 v1.4→v1.5 — AC-ERR-001 and AC-ERR-002 call-signature prose corrected to the real 3-arg form.

**Classification:** Records-tier. No behavioral impact. No code change required.

---

## TD-VSDD-097 Three-Dimension Discharge

**Dim 1 (sibling pair):** Story DEFECT-ADAPTER-TLS-XDOME-LIVE-001 is the sole story in this pass scope; no sibling twin. STORY-INDEX row updated in same burst (v2.782→v2.783). CLEAR.

**Dim 2 (downstream copy target):** AC-ERR-001/002 prose is not a verbatim copy-source for any downstream artifact body; the call-signature is story-local. No additional verbatim copy-source section identified. CLEAR.

**Dim 3 (mandate anchor):** No new MUST blocks authored in records-only micro-burst. All existing story MUSTs retain their BC/RG anchors (unchanged from v1.4). CLEAR.

---

## Convergence Trajectory

| Pass | Findings | Delta | Severity breakdown |
|------|----------|-------|-------------------|
| 1 | 5 | null | 0 CRIT / 0 HIGH / 2 MED / 2 LOW / 1 OBS |
| 2 | 6 | +1 | 0 CRIT / 0 HIGH / 2 MED / 1 LOW / 2 OBS (records-tier additions from fresh review angles) |
| 3 | 1 | -5 | 0 CRIT / 1 HIGH / 0 MED / 0 LOW / 0 OBS |
| 4 | 3 | +2 | 0 CRIT / 0 HIGH / 1 MED / 2 LOW / 0 OBS (records-tier sweep — ADR-050 v2.2 count correction + pin re-sync) |
| 5 | 2 | -1 | 0 CRIT / 0 HIGH / 1 MED / 1 LOW / 0 OBS (duplicate holdout key + AC call-signature prose) |

**Trajectory shorthand:** 5→6→1→3→2
