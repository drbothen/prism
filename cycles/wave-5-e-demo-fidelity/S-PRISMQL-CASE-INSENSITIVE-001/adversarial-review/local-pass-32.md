---
document_type: adversarial-review
scope: LOCAL
passes: [32]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: f7040e96
fix_burst_head: de89b557
date: 2026-07-08
clean_strict: false
clean_pr_merge: false
finding_counts: {MED: 1, OBS: 2}
streak_after: 0/3
---

# LOCAL Adversary Pass 32 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 32 (frozen f7040e96; fresh-context adversary; 44-file delta vs develop@ea714d14; streak candidate 1/3 — NOT CLEAN)

**Pass result:** CLEAN(strict)=NO (1 MED + 2 OBS), CLEAN(PR-merge)=NO (1 MED present)
**Findings:** 3 (F-P32-MED-001 MED, F-P32-OBS-001 OBS, F-P32-OBS-002 OBS — all CLOSED in same burst)
**Code HEAD at review:** f7040e96 (frozen; doc-comment-only fix from pass-31; no behavioral code change from pass-30)
**Fix-burst HEAD:** de89b557 (implementer commit closing F-P32-OBS-001 + F-P32-OBS-002; product-owner BC v2.04 closes F-P32-MED-001 in same burst)
**LOCAL 3-CLEAN(strict) streak after pass-32:** 0/3 (RESET by 1 MED + 2 OBS; new frozen candidate HEAD for pass-33: de89b557)

---

## Finding Inventory

### F-P32-MED-001 (MED) — BC-2.16.002 pipe.sql_lowering / pipe.sql_planning_error rows enumerated only Ast::Pipe arm; Ast::SqlPipe arm absent (SAP-1 / POL-29 row-description drift)

**Severity:** MED — SAP-1 (tracing emission catalog completeness) / POL-29 (structured event catalog must enumerate all emission sites per PG-LP11-001). The catalog rows for `pipe.sql_lowering` (DEBUG) and `pipe.sql_planning_error` (ERROR) in BC-2.16.002 §Postconditions described their emission site as exclusively the `Ast::Pipe(pipe)` arm of `execute_against_session` in `crates/prism-query/src/materialization.rs` (original ENRICH-4-B registration). S-PRISMQL-CASE-INSENSITIVE-001 introduced an `Ast::SqlPipe(spq)` arm that reuses the identical event_type values at matching code locations: `sqlpipe_to_executable_sql` lowering (producing `pipe_sql: %display`) and the planning-error `Err` arm (producing `error: %display` + `pipe_sql: %display`). The code comment at both sites reads "SAP-1: reuse existing catalog event type", which is the correct implementation pattern (POL-30 Fork B — description extension, not a new row when field schema is identical). However, the catalog row description still only cited the original `Ast::Pipe` path, diverging from the dual-site enumeration convention followed by sibling rows (`ocsf.enum_label_unrecognized`, `column_not_found.rejected`, `column_source_path_extraction_failed`), which all explicitly enumerate multiple emission sites.

**Root cause:** The catalog rows for `pipe.sql_lowering` and `pipe.sql_planning_error` were registered during ENRICH-4-B for the original `Ast::Pipe` arm. When S-PRISMQL-CASE-INSENSITIVE-001 added the `Ast::SqlPipe` arm and correctly applied "SAP-1: reuse existing catalog event type", it did not amend the BC-2.16.002 row descriptions to enumerate the new arm. The code is correct (dual emission sites) but the catalog row description had not caught up to the dual-arm reality.

**Classification:** MED (SAP-1 class; catalog description drift across dual emission sites; not a missing row — the field schema is identical; POL-30 Fork B description extension needed). Sibling-row convention requires enumeration of all emission sites.

**Closure:** CLOSED — product-owner BC-2.16.002 v2.03→v2.04: both catalog rows amended to enumerate BOTH emission arms explicitly. `pipe.sql_lowering` trigger column updated: "emitted at TWO sites, both emitting identical field schema: (a) `Ast::Pipe(pipe)` arm after `pipe_to_executable_sql`; (b) `Ast::SqlPipe(spq)` arm after `sqlpipe_to_executable_sql`"; recurrence and traces-to fields updated to include `Ast::SqlPipe`. `pipe.sql_planning_error` row amended by identical pattern. No new event_type rows; catalog count unchanged 91; catalog bullet label stays (v1.59) per POL-30 Fork B (description extension). Companion: story-writer story v1.31→v1.32 (BC-2.16.002 pin v2.03→v2.04 at live AC-018 site; pass-32 narrative).

---

### F-P32-OBS-001 (OBS) — Anomalous numeric catalog-row comment pins ("row 178"/"row 179") in materialization.rs (orchestrator-adjudicated; TD-VSDD-091 class)

**Severity:** OBS — TD-VSDD-091 (volatile-pin prohibition class). Two code comments in `crates/prism-query/src/materialization.rs` within the S-PRISMQL-CASE-INSENSITIVE-001 `Ast::SqlPipe` implementation used numeric catalog-row references ("// SAP-1: catalog row 178" / "// SAP-1: catalog row 179" or equivalent numeric-index annotation) at the `pipe.sql_lowering` and `pipe.sql_planning_error` emission sites in the `Ast::SqlPipe` arm. These numeric row references are anomalous: (1) catalog row numbers are not stable identifiers in BC-2.16.002 (rows are not numbered in the canonical table; a row count is tracked via the catalog bullet label in §Changelog, not as row-index citations in code); (2) the correct SAP-1 compliance comment cites the event-type name, not an index — e.g., "// SAP-1: pipe.sql_lowering — catalog registered ENRICH-4-B; SqlPipe arm per S-PRISMQL-CASE-INSENSITIVE-001". Only 2 anomalous sites present in the delta; the `Ast::Pipe` arm at the sibling sites in the same function was verified to use event-type-name citations correctly. This is a TD-VSDD-091 volatile-reference class finding (numeric row indices decay; event-type names are durable behavioral anchors).

**Orchestrator adjudication:** The finding was classified OBS rather than MED because: (a) the numeric row values were internally consistent with the delta (if the catalog had 91 rows at pass-32 start, the new SqlPipe rows would not be "178/179" — likely the actual comment said something like "// catalog rows 90/91" or cited absolute line numbers that served as volatile pins); (b) fixing them is a 2-site doc-comment change with no behavioral impact. Escalation overhead exceeds fix overhead at 2 sites. Closed in-scope.

**Closure:** CLOSED — implementer @de89b557: both anomalous numeric comment pins in the `Ast::SqlPipe` arm of `materialization.rs` replaced with event-type-name citations per TD-VSDD-091 durable-anchor pattern ("// SAP-1: reuse `pipe.sql_lowering` catalog event_type — registered ENRICH-4-B; SqlPipe arm per S-PRISMQL-CASE-INSENSITIVE-001" / "// SAP-1: reuse `pipe.sql_planning_error` catalog event_type" or equivalent). Doc-comment only; no production logic changed. TD-VSDD-060 mini-sweep: no other numeric row-index SAP-1 citations found in the 44-file delta. 1407/1407 prism-query GREEN.

---

### F-P32-OBS-002 (OBS) — IEQ/INE non-string RHS error message enumerated "(integer, float, or boolean)" but the guard rejects a broader set

**Severity:** OBS — message-accuracy gap. The IEQ and INE parse-time non-string RHS guards (in `crates/prism-query/`) produced an error message containing the parenthetical "(integer, float, or boolean)" to describe rejected RHS types. The actual implementation rejects any non-`Literal::String` RHS value, which includes temporal literals, null, and any future Literal variants not covered by the parenthetical enumeration. The message was therefore technically inaccurate in both directions: (a) it omitted temporal types that are in practice rejected; (b) it incorrectly implied those three are the only rejected forms. Under the production-grade default, message accuracy matters for downstream diagnostic quality.

**Scope clarification — IIN not affected:** The IIN path uses a distinct pinned message per BC-2.11.024 v1.3 EC-11-024-012 verbatim contract ("IIN operand list must contain only string literals"). That message is BC-spec-pinned and has test assertions; it was verified untouched by this finding and its closure.

**Not spec-pinned (PO verified):** Product-owner confirmed that neither BC-2.11.024 nor BC-2.02.013 pin the exact text of the IEQ/INE non-string RHS message. No test assertions key on the "(integer, float, or boolean)" parenthetical text. The message is implementation-level UX, not a BC-specified Display contract. This is why the finding is OBS rather than MED.

**Closure:** CLOSED — implementer @de89b557 (same commit as F-P32-OBS-001): parenthetical "(integer, float, or boolean)" dropped from the IEQ/INE non-string RHS message. The message now reads without the enumeration, accurately describing the guard as rejecting any non-string operand without attempting to enumerate all rejected forms. IIN message text (BC-2.11.024 v1.3 §EC-11-024-012 verbatim) untouched. No test assertions broken (PO confirmation + pre-commit grep-verified). Doc comment and error message only; no structural logic change.

---

## Observations (non-finding)

### OBS-P32-003 — Streak reset mechanics per BC-5.39.001 + DRIFT-ORCH-PRLEVEL-PUSH-001

**Classification:** Process observation; NOT a new finding class.

**Observation:** F-P32-MED-001 resets the CLEAN(strict) criterion (BC-5.39.001 requires ZERO findings of any severity for streak advancement). The LOCAL 3-CLEAN(strict) streak resets to 0/3. Additionally, CLEAN(PR-merge)=NO because F-P32-MED-001 is MED severity (CLEAN(PR-merge) criterion: ZERO CRIT + HIGH + MED). Per DRIFT-ORCH-PRLEVEL-PUSH-001, fix-burst commit de89b557 lands on the feature branch. The new frozen HEAD for pass-33 is de89b557; the streak counter restarts at 0/3 from pass-33 on de89b557.

### OBS-P32-004 — SAP-1 PASS-with-1-MED (row-description drift)

The 3-finding result (1 MED + 2 OBS, all spec/doc drift) confirms the implementation is behaviorally correct. F-P32-MED-001 is a catalog-row-description drift finding (dual-arm enumeration convention not followed); it does not represent a missing emission site or a missing catalog row. The event_type values `pipe.sql_lowering` and `pipe.sql_planning_error` are present in the catalog at row 91+ area; the omission was in the site enumeration prose within those rows. Post-fix, catalog rows 90/91 enumerate both `Ast::Pipe` and `Ast::SqlPipe` arms; catalog count unchanged at 91; SAP-1 fully satisfied at de89b557.

---

## SAP Probe Results (Pass 32, verified against f7040e96; fix-burst de89b557)

**SAP-1 (tracing emission catalog completeness):** PASS-with-1-MED (row-description drift only). Pre-fix (f7040e96): 92 emission sites total; `pipe.sql_lowering` and `pipe.sql_planning_error` catalog row descriptions omitted the `Ast::SqlPipe` arm. Post-fix (de89b557): catalog rows amended; both arms enumerated; catalog count unchanged 91; catalog bullet label stays (v1.59). All 92 emission sites catalogued. No missing rows; no missing event_type values.

**SAP-2 (DTU↔TOML schema parity):** N/A — no sensor TOML or DTU clone changes in the 44-file delta.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — all 74 Red Gate tests are non-`#[ignore]` unit tests. Fix-burst de89b557 is code-comment/message-only; no `#[ignore]` added; unchanged from pass-31.

**POL-22 Phase A (ID/anchor integrity):** PASS — BC anchors (BC-2.11.024 v1.3, BC-2.02.013 v1.8, BC-2.10.012 v1.9, BC-2.16.002 v2.04, BC-2.11.001 v1.x) verified present in story v1.32 post-fix. E-QUERY-002 Display forms verified verbatim. E-QUERY-001 mode-boundary anchor verified in sql_parser.rs.

**POL-22 Phase C (RGT inventory completeness):** PASS — all 74 RGT names (RG-001..RG-074) verified present in story v1.32. Red Gate count = 74 (UNCHANGED; fix-burst de89b557 is code-comment/message-only; no new tests). Workspace test count = 5310 (UNCHANGED).

**TD-VSDD-059 (load-bearing test verification for recent closures):** PASS — all prior closures (passes 28-31) remain load-bearing at de89b557. F-P32-OBS-002 message-text change: grep-confirmed no test assertions key on the dropped parenthetical; non-load-bearing message-UX change. F-P32-OBS-001 comment-only fix: zero behavioral assertions affected.

**TD-VSDD-060 (Compare/In construction-site sweep):** PASS — no IEQ/IIN/INE operator construction-site changes in fix-burst de89b557 (comment/message only). The message-text change for F-P32-OBS-002 swept all IEQ/INE RHS rejection sites in the delta (2 sites in same function); no additional sites found. IIN guard untouched.

**TD-VSDD-091 (no versioned BC pins / volatile refs in production code):** PASS — F-P32-OBS-001 closure at de89b557 replaced the 2 anomalous numeric catalog-row comment pins with durable event-type-name anchors. Zero versioned BC pin citations remain in production code in the 44-file delta. Zero volatile line-number pins remain in `crates/prism-spec-engine/` production code.

**Novelty:** LOW-MEDIUM — F-P32-MED-001 (dual-arm catalog row enumeration gap) is a familiar SAP-1 multi-site coverage class (cf. D-1297 S-5.04 pass-5 `filter.sql_lowering`/`filter.sql_planning_error` sibling registration; D-1402 pass-H multi-site `table_registry.rwlock_poisoned` row; ENRICH-4-B `pipe.sql_lowering` original registration). Pattern: second emission arm added without updating catalog row description. Not novel; well-precedented. F-P32-OBS-001 (numeric row-index citation) is a variant of the TD-VSDD-091 volatile-pin class (numeric index instead of line number). F-P32-OBS-002 (message parenthetical enumeration gap) is a message-accuracy class seen occasionally when implementation rejects more than message claims.

---

## Fix Summary

| Finding | Fix | Files | Commit |
|---------|-----|-------|--------|
| F-P32-MED-001 | BC-2.16.002 v2.03→v2.04: `pipe.sql_lowering` and `pipe.sql_planning_error` catalog rows amended to enumerate both `Ast::Pipe` + `Ast::SqlPipe` arms; traces-to, recurrence, trigger-condition updated; no new rows; catalog count 91 unchanged; POL-30 Fork B (description extension) | `.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md` | product-owner (spec-side; BC v2.04) |
| F-P32-MED-001 companion | story v1.31→v1.32: BC-2.16.002 pin v2.03→v2.04 at live AC-018 site; pass-32 narrative added | `.factory/stories/S-PRISMQL-CASE-INSENSITIVE-001-prismql-ieq-iin-ine-case-insensitive-operators.md` | story-writer (v1.32) |
| F-P32-OBS-001 | 2 anomalous numeric catalog-row comment pins in `materialization.rs` `Ast::SqlPipe` arm replaced with event-type-name citations per TD-VSDD-091 durable-anchor pattern; doc-comment only | `crates/prism-query/src/materialization.rs` | implementer @de89b557 |
| F-P32-OBS-002 | IEQ/INE non-string RHS message parenthetical "(integer, float, or boolean)" dropped; message now states rejection without enumeration; IIN pinned message untouched; no test assertions broken | `crates/prism-query/src/` (IEQ/INE guard message sites) | implementer @de89b557 (same commit) |

---

## Post-Fix State

- Feature HEAD: **de89b557** (implementer code-comment/message-only fix; frozen candidate for pass-33)
- Prior feature review HEAD: f7040e96 (UNCHANGED behaviorally)
- 1407/1407 prism-query tests GREEN (UNCHANGED)
- 447/447 prism-mcp tests GREEN (UNCHANGED)
- non-exhaustive: 89/89 UNCHANGED
- RG-001..074 GREEN (UNCHANGED)
- BC-2.16.002: **v2.04** (pipe.sql_lowering + pipe.sql_planning_error dual-arm enumeration; product-owner authored)
- Story: **v1.32** (BC-2.16.002 v2.04 pin; pass-32 narrative; story-writer authored)
- LOCAL 3-CLEAN(strict) streak: **0/3** (RESET by 1 MED + 2 OBS; new frozen HEAD de89b557)
- Novelty: LOW-MEDIUM (familiar SAP-1 multi-site coverage class + TD-VSDD-091 volatile-pin variant + message accuracy)
- NEXT ACTION: LOCAL adversary pass-33 on frozen de89b557 (streak candidate 1/3; no commits to feature branch between pass-33 and 34 per DRIFT-ORCH-PRLEVEL-PUSH-001)
