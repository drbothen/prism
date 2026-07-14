---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [8]
feature_head_at_review: 68b0808a
date: 2026-07-14
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 2
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 2
  process_gap: 0
  out_of_scope_obs: 1
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 8 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 8 (frozen 68b0808a; fresh-context adversary; PR #222 MCP row-shape null serialization + H8b redundancy sweep + threatintel .prx staleness gate; PR-LEVEL cascade; streak RESET 1/3→0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

---

## Findings

### F-MCPRS-PRL8-OBS-001 [OBS/low][doc-drift]

**Severity:** OBS (low)
**Classification:** doc-drift — POL-23 narrow-vs-strict interpretation gap
**BC:** BC-2.11.001 (null-not-absent row serialization)

**Finding:** 10 code and test citation sites in `prism-mcp` are pinned to `"BC-2.11.001 v1.20"` while the canonical BC is now at v1.21 (updated in D-1722 via F-MCPNULL-P16-LOW-001 closure — §D2/§D4 source-returns-None vs null-input-short-circuit disambiguation). The drift is a POL-23 narrow-vs-strict interpretation gap: the POL-23 sweep in prior bursts updated BC-2.11.001 version pins in story spec files and some code comments but did not cover all 10 code/test citation sites in the prism-mcp crate.

**Affected sites (10 total):**
- `crates/prism-mcp/src/server.rs` — approximately 6 doc-comment citation sites in the null-not-absent serialization logic referencing `BC-2.11.001 v1.20`
- `crates/prism-mcp/tests/wire_shape_tests.rs` — approximately 4 test doc-string citation sites referencing `BC-2.11.001 v1.20`

**Impact:** Zero runtime impact. Version pin citations in doc-comments and test doc-strings are non-normative; the tests exercise the correct behavior regardless of the cited version string. The actual serialization behavior matches BC-2.11.001 v1.21. Severity: OBS (low).

**Resolution (fix-burst-20 code @448158f8):** Implementer replaced all 10 `"BC-2.11.001 v1.20"` version-pinned citations plus an additional 895 volatile BC-version pins across 87 files workspace-wide with version-agnostic anchor forms (e.g., `BC-2.11.001` without version suffix), per TD-VSDD-060 sibling-site sweep discipline. Total volatile BC-version pins stripped: 905 across 87 files. Zero assertion values altered; zero runtime code changed; historical facts (e.g., `§D4 null-not-absent spec` annotations) preserved. prism-mcp 473/473 GREEN after sweep. Pre-push `just check` 214s GREEN.

---

### F-MCPRS-PRL8-OBS-002 [OBS/low][phrasing drift]

**Severity:** OBS (low)
**Classification:** doc-drift — BC §RETRYABLE-503 snippet phrasing vs shipped idiomatic Rust code

**Finding:** BC-2.10.007 §RETRYABLE-503 `§Implementer Code Follow-Up` section contained the snippet form `.as_u16()` (e.g., `matches!(status.as_u16(), 408|425|429|...)`) while the FB-19 implementer used the idiomatic `.code()` form in `error_mapping.rs`. Both forms compile to identical semantics for HTTP status code comparison (`StatusCode::as_u16()` returns `u16`; `StatusCode::code()` is a common alias). The discrepancy is documentation-only: the BC narrative was written during fix-burst-18 adjudication using the `.as_u16()` form; the implementer chose the `.code()` idiomatic form when writing the code.

**Impact:** Zero runtime impact; semantically identical. The shipped test correctly exercises the whitelist with the `.code()` form. Severity: OBS (low).

**Resolution (fix-burst-20 PO spec @9d1f6b5e):** BC-2.10.007 v1.16→v1.17 — §RETRYABLE-503 `§Implementer Code Follow-Up` snippet updated from `.as_u16()` form to `.code()` form to match the idiomatic shipped Rust. POL-23 sweep: S-MCP-E003-SERIALIZATION-MIGRATION-001 v0.6→v0.7 (9 pins) + S-TEST-WIRESHAPE-SWEEP-001 v0.15→v0.16 (12 pins) + S-MCP-THREATINTEL-PROD-ENDPOINT-001 v0.2→v0.3 (security_review: required frontmatter + AC-004 CWE-73 anchor + §Security Review Required section). BC-INDEX v8.10→v8.11.

---

## Out-of-Scope Observations

### F-MCPRS-PRL8-OOS-001 [OOS/MED][pre-existing][URL-injection]

**Severity:** MED (out-of-scope for DEFECT-MCP-ROWSHAPE-NULLS-001)
**Classification:** pre-existing security gap — `is_domain` path-traversal + raw URL format! interpolation
**Origin:** `S-DEMO-ENRICHMENT-PIVOT-002` (threatintel-lookup plugin; pre-existing before current branch)
**CWE:** CWE-73 (External Control of File Name or Path), CWE-918 (Server-Side Request Forgery)

**Finding:** The `is_domain` validation function in the `threatintel-lookup` plugin (`crates/prism-spec-engine/plugins/threatintel-lookup/src/lib.rs`) accepts values containing path-traversal characters (e.g., `"../etc/passwd"`, `"foo/bar"`, `"example.com/path"`) as syntactically valid domain inputs. The URL construction site (`format!("{base_url}/lookup?domain={domain_input}")`) does not percent-encode the input value before interpolation. An adversarially-crafted `input_value` binding in a PrismQL query could construct an SSRF request to a path other than `/lookup?domain=<expected>`.

**Current blast radius analysis:**
- `allowed_urls` in the plugin manifest is hardcoded to `["http://127.0.0.1:8765"]` (localhost DTU clone URL only)
- The DTU clone server binds only to localhost and serves only the `/lookup` endpoint; path variations return 404 and do not expose credentials or other services
- No production ThreatIntel API credentials are in scope: the production endpoint URL is not yet configured (business team identification pending; tracked by S-MCP-THREATINTEL-PROD-ENDPOINT-001 P2)
- Blast radius is fully capped at the localhost DTU clone binding in the current deployment

**Scope ruling:** Pre-existing finding predating the current branch. DEFECT-MCP-ROWSHAPE-NULLS-001 covers row-shape null serialization, H8b message/suggestion dedup, and .prx staleness gate — not `is_domain` input sanitization. This finding is **out-of-scope** for PR #222.

**Anchor:** S-MCP-THREATINTEL-PROD-ENDPOINT-001 v0.3 (fix-burst-20 PO @9d1f6b5e) now carries:
- Frontmatter: `security_review: required`
- New AC-004: percent-encode `input_value` + strict `is_domain` charset validation (A-Z, a-z, 0-9, hyphen, dot only) + 2 named unit tests
- New §Security Review Required section: documents CWE-73, SSRF blast radius, and the `is_domain` charset gap

The `security_review: required` gate will block S-MCP-THREATINTEL-PROD-ENDPOINT-001 delivery until CWE-73/CWE-918 remediation is confirmed by security-reviewer. **Not a blocker for PR #222 merge** (pre-existing, out-of-scope, blast radius currently capped, properly anchored to future story with security review gate).

---

## SAP-1 Emission Catalog Probe

**PASS.** No new `event_type =` emissions introduced by the branch relative to develop@5f1b5771. All emissions present in the branch were catalogued in BC-2.16.002 §Postconditions in prior bursts.

---

## Summary

**CLEAN(strict): NO** (2 OBS findings — F-MCPRS-PRL8-OBS-001 doc-drift, F-MCPRS-PRL8-OBS-002 phrasing drift)
**CLEAN(PR-merge): YES** (zero CRIT/HIGH/MED in-scope findings; 2 OBS are non-blocking; 1 MED OOS pre-existing anchored to future story)

Streak: **0/3** RESET from 1/3 (pass-7 CLEAN was the prior streak holder on this HEAD; pass-8 OBS findings reset the strict-clean count per BC-5.39.001).

Both in-scope OBS findings closed by fix-burst-20: spec @9d1f6b5e (OBS-002 BC phrasing + POL-23 sweeps) + code @448158f8 (OBS-001 version-agnostic citation sweep 905 pins). OOS-001 anchored to S-MCP-THREATINTEL-PROD-ENDPOINT-001 v0.3 with `security_review: required` gate.

Branch pushed: 68b0808a→448158f8. NEW MCP FROZEN HEAD: 448158f8. PR-LEVEL streak 0/3 on 448158f8 (push resets per DRIFT-ORCH-PRLEVEL-PUSH-001). PR-LEVEL pass 9 dispatched on frozen 448158f8.
