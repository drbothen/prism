---
pass: PR-LEVEL-2
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
pr: 203
head: e0374818
clean_strict: NO
clean_pr_merge: YES
findings: 2
findings_severity: [LOW, LOW]
streak_after: 0/3 (RESET — new code HEAD 752e22ce)
date: 2026-06-25
---

# PR-LEVEL Pass 2 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (PR #203)

**HEAD reviewed:** e0374818  
**CLEAN (strict):** NO — 2 LOW findings (OBS-1, OBS-2)  
**CLEAN (PR-merge):** YES — zero CRIT/HIGH/MED findings  
**Streak after:** 0/3 RESET (implementer pushed fix commits 1275a2c1 + 752e22ce → new frozen PR HEAD = 752e22ce)

## Findings

### OBS-1 (LOW) — Stale NYA handler doc comments (CLAUDE.md Standing Rule 3 §3)

**Severity:** LOW  
**Location:** `crates/prism-mcp/src/handlers/` — 33 NOT_YET_AVAILABLE tool handler doc comments  
**Finding:** Handler doc comments on the 33 NYA handlers claimed "scanned for prompt injection" but the NOT_YET_AVAILABLE guard reorder (BC-2.10.017, AC-017) now returns `-32003` immediately before any input processing occurs. The guard fires first, so no scanning takes place. The doc comment was factually incorrect per CLAUDE.md Standing Rule 3 §3 ("Doc comment claiming 'this requires capability X' with no capability check" — must either implement the gate or remove the docs).  
**CLAUDE.md Rule:** Standing Rule 3 §3 (forbidden pattern: doc comment claiming behavior without implementation).  
**AD-017 status:** Credential handler comments correctly retained "no credential data accessed" — those are unaffected by the guard-reorder.

**Resolution:** CLOSED at code HEAD 1275a2c1.  
Implementer corrected all 33 NYA handler doc comments to: "returns E-INFRA-NYA / -32003 immediately; no input processing occurs." AD-017 preserved on credential handlers (separate annotation).

**Verification:** `rg 'scanned for prompt injection' crates/prism-mcp/` → zero hits after fix.

---

### OBS-2 (LOW) — AC-024 GRAMMAR-013 table cited informal section labels not matching real `build_reference_content` headers

**Severity:** LOW  
**Location:** `docs/demo-evidence/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001/evidence-report.md`, GRAMMAR-013 discoverability table; also `code-delivery/.../pr-description.md`  
**Finding:** The Teaching Surface column in the AC-024 GRAMMAR-013 table used informal section labels such as "enrichment paragraph" and "Datetime Arithmetic" that did not exactly match the headers produced by `build_reference_content()` in `crates/prism-mcp/src/resources.rs`. Real headers include `## Clause Grammar (BNF)`, `## Datetime and Temporal Arithmetic`, `## Operators, Types, and Aggregate Functions`, etc. A reader verifying AC-024 by scanning the reference output would not find the informal labels.  
**Impact:** AC-024 discoverability claim was imprecise — not a runtime defect, but a documentation accuracy gap that could cause confusion during demo verification.

**Resolution:** CLOSED at code HEAD 752e22ce.  
Implementer corrected `evidence-report.md` GRAMMAR-013 table to cite real `build_reference_content` headers verbatim. pr-manager synced the corrected table into `code-delivery/.../pr-description.md` and updated GitHub PR #203 body description.

**Corrected GRAMMAR-013 table (real headers after fix):**

| GRAMMAR-013 Item | Teaching Surface |
|-----------------|-----------------|
| Infusion names | `prism_describe` → `list_infusions` → `## Clause Grammar (BNF)` enrichment paragraph |
| Column argument form `fn(col)` | Error message (AC-022/025) + `## Clause Grammar (BNF)` (`\| enrich <fn>(<col>)` in Pipe Mode BNF) |
| Pipe stage keywords | D1 mode-bridge message (AC-009) + `## Clause Grammar (BNF)` (Pipe Mode BNF block) |
| SQL+pipe composition syntax | `## Clause Grammar (BNF)` (SqlPipe Mode block) + D1 numbered alternatives |
| Pipe-mode `FROM t \| where` form | `## Clause Grammar (BNF)` (Pipe Mode BNF block) + D2 mode-bridge (AC-027) |
| Filter mode syntax | `## What is PrismQL` (mode summary table, Filter row) |
| INTERVAL literal form | `## Datetime and Temporal Arithmetic` + E-QUERY-001 error (AC-005) |
| NOW() semantics | `## Datetime and Temporal Arithmetic` |
| IS NOT NULL on JSON lists | `## Operators, Types, and Aggregate Functions` (null semantics paragraph) (AC-023) |
| Aggregate syntax `stats agg [by field]` | `## Operators, Types, and Aggregate Functions` (aggregate functions list) — `percentile`, `distinct_count` (AC-026) |

---

## Post-Fix State

- Code HEAD progressed: e0374818 → 1275a2c1 (OBS-1 fix) → 752e22ce (OBS-2 fix)
- New FROZEN PR HEAD = **752e22ce**
- just check EXIT=0 on 752e22ce (4929 tests; ~21 min wall-clock confirmed)
- fmt-canonical: clean
- non-exhaustive gate: EXPECTED=87 (unchanged)
- pr-description.md GRAMMAR-013 table: corrected real headers
- GitHub PR #203 body: synced with corrected table
- 3-CLEAN streak RESET 0/3 on 752e22ce (code HEAD moved by OBS-1/OBS-2 fix)
- CI: one transient runner failure on e0374818 run (x86_64-unknown-linux-gnu "No space left on device" — NOT a code failure; all other CI gates green); CI re-runs on 752e22ce push

## Next

PR-LEVEL Pass 3 dispatched on FROZEN HEAD 752e22ce.  
3 consecutive CLEAN(strict) on UNCHANGED 752e22ce required for BC-5.39.001 PR-LEVEL convergence.

## Verdict

CLEAN (strict): NO  
CLEAN (PR-merge): YES  

Both findings CLOSED at 752e22ce. Streak RESET 0/3 on new PR HEAD 752e22ce.
