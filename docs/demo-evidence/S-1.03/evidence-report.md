# S-1.03 Demo Evidence Report

**Story:** S-1.03 — prism-core: Capability Resolution Engine  
**Branch:** feature/S-1.03-capability-resolution  
**Commit at recording:** cf76e5e  
**Test suite result:** 69/69 tests pass (26 S-1.03 + 43 inherited S-1.01)  
**Recording tool:** VHS 0.10.0  
**Font:** FiraCode Nerd Font Mono  

---

## Coverage Map

| AC | Description | Recording | Status |
|----|-------------|-----------|--------|
| AC-1 | Empty `ClientCapabilities` returns `(false, "deny-by-default")` | [AC-001-deny-by-default.gif](AC-001-deny-by-default.gif) / [.webm](AC-001-deny-by-default.webm) | RECORDED |
| AC-2 | Parent `Allow` covers child path via prefix traversal | [AC-002-parent-allow-covers-child.gif](AC-002-parent-allow-covers-child.gif) / [.webm](AC-002-parent-allow-covers-child.webm) | RECORDED |
| AC-3 | Most-specific `Deny` overrides parent `Allow` | [AC-003-explicit-deny-overrides-parent-allow.gif](AC-003-explicit-deny-overrides-parent-allow.gif) / [.webm](AC-003-explicit-deny-overrides-parent-allow.webm) | RECORDED |
| AC-4 | Exact match — `explanation.matched_path` equals queried path | [AC-004-exact-match-explanation.gif](AC-004-exact-match-explanation.gif) / [.webm](AC-004-exact-match-explanation.webm) | RECORDED |
| AC-5 | `CapabilityPath::new("a..b")` returns `Err`; error paths shown | [AC-005-path-validation-rejects-empty-segment.gif](AC-005-path-validation-rejects-empty-segment.gif) / [.webm](AC-005-path-validation-rejects-empty-segment.webm) | RECORDED |
| AC-6 | `parent()` called twice on `"a.b.c"` reaches `"a"`; single-segment returns None | [AC-006-parent-traversal.gif](AC-006-parent-traversal.gif) / [.webm](AC-006-parent-traversal.webm) | RECORDED |
| AC-7 | `"crowdstrike.hosts" → Allow` covers `"crowdstrike.hosts.read"` | [AC-007-parent-allow-covers-grandchild.gif](AC-007-parent-allow-covers-grandchild.gif) / [.webm](AC-007-parent-allow-covers-grandchild.webm) | RECORDED |
| AC-8 | VP-002 Kani proof: deny-by-default (symbolic) | [AC-008-009-010-kani-proofs.md](AC-008-009-010-kani-proofs.md) | PLACEHOLDER (unit proxy PASSED) |
| AC-9 | VP-003 Kani proof: most-specific wins (both directions) | [AC-008-009-010-kani-proofs.md](AC-008-009-010-kani-proofs.md) | PLACEHOLDER (unit proxies PASSED) |
| AC-10 | VP-004 Kani proof: exact match explanation correctness | [AC-008-009-010-kani-proofs.md](AC-008-009-010-kani-proofs.md) | PLACEHOLDER (unit proxy PASSED) |

---

## Recording Details

### AC-1 — deny-by-default

**Test:** `test_S_1_03_ac1_empty_caps_deny_by_default`  
**Tape:** [AC-001-deny-by-default.tape](AC-001-deny-by-default.tape)  
**Paths demonstrated:** success path (empty caps → false), `explanation.reason == "deny-by-default"`

### AC-2 — parent Allow covers child

**Test:** `test_S_1_03_ac2_parent_allow_covers_child`  
**Tape:** [AC-002-parent-allow-covers-child.tape](AC-002-parent-allow-covers-child.tape)  
**Paths demonstrated:** `{"crowdstrike" → Allow}` → `is_allowed("crowdstrike.hosts.write")` = true

### AC-3 — explicit Deny overrides parent Allow

**Test:** `test_S_1_03_ac3_specific_deny_overrides_parent_allow`  
**Tape:** [AC-003-explicit-deny-overrides-parent-allow.tape](AC-003-explicit-deny-overrides-parent-allow.tape)  
**Paths demonstrated:** success path (specific Deny wins), `explanation.reason == "explicit-deny"`

### AC-4 — exact match explanation

**Test:** `test_S_1_03_ac4_exact_match_explanation_matched_path`  
**Tape:** [AC-004-exact-match-explanation.tape](AC-004-exact-match-explanation.tape)  
**Paths demonstrated:** `explanation.matched_path == Some("audit.read")`, `reason == "explicit-allow"`

### AC-5 — path validation

**Test (success path):** `test_S_1_03_ac5_rejects_empty_segment_path`  
**Test (error paths):** `test_S_1_03_ec_rejects_empty_string`, `test_S_1_03_ec_rejects_nine_segments`, `test_S_1_03_ec_rejects_exceeds_256_chars`, `test_S_1_03_ec_rejects_invalid_chars`  
**Tape:** [AC-005-path-validation-rejects-empty-segment.tape](AC-005-path-validation-rejects-empty-segment.tape)  
**Paths demonstrated:** `"a..b"` → Err; empty string → Err; 9-segment → Err; 257-char → Err; invalid chars → Err

### AC-6 — parent traversal

**Test (success path):** `test_S_1_03_ac6_parent_called_twice_reaches_grandparent`  
**Test (error path):** `test_S_1_03_ec_parent_of_single_segment_is_none`  
**Tape:** [AC-006-parent-traversal.tape](AC-006-parent-traversal.tape)  
**Paths demonstrated:** `"a.b.c".parent().parent() == Some("a")`; `"a".parent() == None`

### AC-7 — parent Allow covers grandchild

**Test:** `test_S_1_03_ac7_parent_allow_covers_grandchild`  
**Tape:** [AC-007-parent-allow-covers-grandchild.tape](AC-007-parent-allow-covers-grandchild.tape)  
**Paths demonstrated:** `{"crowdstrike.hosts" → Allow}` → `is_allowed("crowdstrike.hosts.read")` = true

### AC-8, AC-9, AC-10 — Kani Proofs (VP-002, VP-003, VP-004)

**Document:** [AC-008-009-010-kani-proofs.md](AC-008-009-010-kani-proofs.md)  
**Status:** Proof harnesses committed in `crates/prism-core/src/proofs/capability.rs`.
Unit-level proxies for all three proofs pass in the standard `cargo test` suite.
Kani execution requires the Kani toolchain and is deferred to a dedicated
formal-verification pipeline.

| Proof | Unit Proxy | Result |
|-------|-----------|--------|
| `proof_deny_by_default` (VP-002) | `test_S_1_03_vp002_deny_by_default_unit` | ok |
| `proof_most_specific_wins_*` (VP-003) | `test_S_1_03_vp003_most_specific_wins_*` (2 tests) | ok |
| `proof_exact_match_explanation` (VP-004) | `test_S_1_03_vp004_exact_match_explanation_fields` | ok |

---

## File Inventory

```
docs/demo-evidence/S-1.03/
  AC-001-deny-by-default.tape
  AC-001-deny-by-default.gif
  AC-001-deny-by-default.webm
  AC-002-parent-allow-covers-child.tape
  AC-002-parent-allow-covers-child.gif
  AC-002-parent-allow-covers-child.webm
  AC-003-explicit-deny-overrides-parent-allow.tape
  AC-003-explicit-deny-overrides-parent-allow.gif
  AC-003-explicit-deny-overrides-parent-allow.webm
  AC-004-exact-match-explanation.tape
  AC-004-exact-match-explanation.gif
  AC-004-exact-match-explanation.webm
  AC-005-path-validation-rejects-empty-segment.tape
  AC-005-path-validation-rejects-empty-segment.gif
  AC-005-path-validation-rejects-empty-segment.webm
  AC-006-parent-traversal.tape
  AC-006-parent-traversal.gif
  AC-006-parent-traversal.webm
  AC-007-parent-allow-covers-grandchild.tape
  AC-007-parent-allow-covers-grandchild.gif
  AC-007-parent-allow-covers-grandchild.webm
  AC-008-009-010-kani-proofs.md
  evidence-report.md
  run-ac1.sh  (VHS helper script)
  run-ac2.sh
  run-ac3.sh
  run-ac4.sh
  run-ac5.sh
  run-ac6.sh
  run-ac7.sh
```
