---
document_type: adversarial-review-pass
story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pass: 2
phase: LOCAL
frozen_head: "(story v1.2 + code commit ac9563192 + spec leg: BC-2.16.002 v2.15 / BC-2.08.002 v1.5 / BC-2.01.013 v1.17 / error-taxonomy v2.73)"
verdict_strict: "NO"
verdict_pr_merge: "NO"
findings_count: 6
streak_before: 0
streak_after: 0
closed_by: "code commits e21b0cdc3 / dff20e910 / 8f6b5e131 / 67638ce07 + spec-leg (BC-2.16.002 v2.16 / BC-2.01.010 v1.6 / ADR-050 v2.1 / story v1.3 / BC-INDEX v9.00 / STORY-INDEX v2.781)"
timestamp: 2026-08-13
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — LOCAL Adversary Pass 2

**CLEAN(strict): NO | CLEAN(PR-merge): NO**
**BC-5.39.001 streak: RESET 0/3** (findings present)
**All 6 findings CLOSED by fix-burst (code commits e21b0cdc3 / dff20e910 / 8f6b5e131 / 67638ce07 + spec leg)**

---

## Findings

### MED-1a — RG-004 test weak (does not assert fan_out_target_failed WARN was emitted)
**Severity:** MED | **Owner:** test-writer / implementer | **Status:** CLOSED
**Closure:** RG-004 test strengthened in code commit to assert that `fan_out_target_failed` WARN tracing event is actually emitted per BC-2.16.002 §Canonical Structured Event Catalog row 91. Code commits e21b0cdc3 / dff20e910.

### MED-1b — RG-004 citation mis-anchor in BC-2.16.002 postcondition
**Severity:** MED | **Owner:** product-owner / state-manager | **Status:** CLOSED
**Closure:** BC-2.16.002 v2.16 corrected RG-004 cite to match the actual test symbol `test_fanout_all_failed_emits_fan_out_target_failed_warn`; phantom anchor removed. BC-INDEX v8.99→v9.00 pin bump.

### OBS-4 [process-gap] — infusion-client UA sibling-sweep gap in ADR-050 §D6
**Severity:** OBS / PROCESS-GAP | **Owner:** architect / implementer | **Status:** CLOSED
**Closure:** ADR-050 §D6 v2.1 scope extended to include `build_http_client_with_timeout §pipeline` in prism-spec-engine (`HttpLookupSource` outbound factory); code commit 8f6b5e131 adds `.user_agent(...)` call to the infusion HTTP client factory. `test_infusion_http_client_sends_prism_user_agent` is the load-bearing RG-012. ARCH-INDEX v2.300→v2.301 pin.

### OBS-5 — RG-007 not load-bearing (routes through test double, not production check_one)
**Severity:** OBS | **Owner:** implementer | **Status:** CLOSED
**Closure:** RG-007 reworked in code commit 67638ce07 to route through production `check_one` path, eliminating test-double bypass. SAP-3 §Rule 1 now satisfied for this arm.

### LOW-2 — phantom AC doc anchors in story (AC-ERR-003 / AC-SAP1-001 cite stale §section)
**Severity:** LOW | **Owner:** story-writer | **Status:** CLOSED
**Closure:** Story v1.3 corrected phantom AC doc-anchor cites to point to live §section headings. ADR-050 pin in story frontmatter advanced to v2.1.

### LOW-3 — RG-name drift: story RG-table lists different test symbols than code
**Severity:** LOW | **Owner:** story-writer / state-manager | **Status:** CLOSED
**Closure:** Story v1.3 RG-table reconciled to code. RG-012 (`test_infusion_http_client_sends_prism_user_agent`) added for the infusion-UA gate (OBS-4 companion). Total RGTs 11→12; density 12/14 = 0.857. STORY-INDEX v2.780→v2.781.

---

## TD-VSDD-097 Three-Dimension Sweep Verdict

**Dim-1 (Sibling pair):** BC-2.16.002 and BC-2.01.010 are both touched in this burst (sibling BCs on the same story perimeter); both updated in same burst. CLEAR.

**Dim-2 (Downstream copy target):** ADR-050 §D6 scope extension was NOT verbatim-copied into a BC postcondition in this burst; the BC-2.16.002 §HTTP Client Compliance postcondition references ADR-050 §D5/§D6 by section anchor only (no verbatim copy), so no copy-target sweep gap. CLEAR.

**Dim-3 (Mandate anchor):** OBS-4's new `MUST` in ADR-050 §D6 v2.1 ("all outbound third-party HTTP client builders") names anchor story DEFECT-ADAPTER-TLS-XDOME-LIVE-001 + AC (infusion-client builder AC) + RG-012 (`test_infusion_http_client_sends_prism_user_agent`). CLEAR.
