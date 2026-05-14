---
document_type: fix-burst-closure-report
target_artifact: S-PLUGIN-PREREQ-D
fix_burst_N: 29
target_pass: 31
findings_closed: 3 in-scope (2 HIGH F-LP31-HIGH-001 message-template three-site drift + F-LP31-HIGH-002 cross-spec security-semantic; 1 MED F-LP31-MED-001 AuthToken Debug example) + 1 sibling-site catch (BC-2.17.002 v1.5→v1.6 story version-pin propagation)
findings_intent_adjudicated: 0
findings_deferred: 0
producer: state-manager (orchestrator-coordinated; story-writer + product-owner parallel Stage 1 + state-manager Stage 2)
specialist_routing: "Multi-agent burst: story-writer (3 story fixes) + product-owner (BC-2.17.002 v1.6 amendment + BC-INDEX v4.72 bump) parallel; state-manager closes single commit with story BC version-pin propagation"
story_v_before: 1.28
story_v_after: 1.29
bc_2_17_002_v_before: 1.5
bc_2_17_002_v_after: 1.6
bc_index_v_before: 4.71
bc_index_v_after: 4.72
factory_shas: [bc9a9d2a, cc4679ab, "<closure SHA TBD>"]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → 1 → 4 → 5 → 1 → 1 → 3 → CLOSED"
next_action: "Adversary pass-32 dispatch — target streak 0/3 → 1/3 if CLEAN; codifications #11-#15 + #13 sub-extension + #16 candidate (cross-table sweep for error message templates) all active. Trajectory pass-25..pass-31: 4→1→4→5→1→1→3."
codification_candidate_16: "Verbatim cross-table sweep extension for error message templates (not just BC titles). Pass-31 surfaced E-PLUGIN-013/014 §Error Taxonomy Additions table drift from canonical."
---

# Fix-Burst-29 Closure Report — S-PLUGIN-PREREQ-D

**Pass:** 31 (BLOCKED) → Fix-Burst-29 (CLOSED)
**Story version:** v1.28 → v1.29
**BC-2.17.002:** v1.5 → v1.6
**BC-INDEX:** v4.71 → v4.72
**STORY-INDEX:** v2.98 → v2.99
**Specialist routing:** story-writer (3 story fixes) + product-owner (BC amendment + BC-INDEX bump) PARALLEL; state-manager closes single commit (POL-3 last-runner + TD-VSDD-053 single-commit)

---

## Findings Closed

### F-LP31-HIGH-001 — §Error Taxonomy Additions E-PLUGIN-013/014 Message Template Drift [POL-7 axis extension, codification #16 candidate]

**Class:** process-gap — message template text not swept by codifications #11-#15 (which covered BC titles, body-table titles, §References, exclusion-note paragraphs, and §References completeness, but NOT error message template strings in §Error Taxonomy Additions table).

**Before (story §Error Taxonomy Additions, E-PLUGIN-013):**
```
"Plugin manifest at '{path}' is missing required field 'allowed_urls'"
```

**After (story §Error Taxonomy Additions, E-PLUGIN-013):**
```
"Plugin manifest at '{path}' missing required field 'allowed_urls'; field must be an explicit list (use 'allowed_urls = []' for no URLs)"
```

**Before (story §Error Taxonomy Additions, E-PLUGIN-014):**
```
"Plugin manifest at '{path}' has unsupported format_version {actual}"
```

**After (story §Error Taxonomy Additions, E-PLUGIN-014):**
```
"Plugin manifest at '{path}' format_version {actual} exceeds maximum supported version {supported}"
```

**Sites aligned:** Three-site drift across story §Error Taxonomy Additions table, AC-5 body prescription, and canonical error-taxonomy.md. E-PLUGIN-015/016 were already verbatim — only 013/014 drifted.

**Story-writer sibling sweep:** 5/5 CLEAN — no remaining mis-aligned error message template strings at active-body sites for E-PLUGIN-013/014.

**Codification #16 raised:** Verbatim cross-table sweep must extend to error message template text (not just BC H1 titles). Pre-adjudication by session-reviewer at cycle-close.

---

### F-LP31-HIGH-002 — BC-2.17.002 EC-17-007 Cross-Spec Security-Semantic Conflict [Source-of-Truth Precedence Rule 1]

**Class:** cross-spec security-semantic — BC-2.17.002 v1.5 EC-17-007 described pre-AC-7 allow-all semantics ("Request allowed to any URL (open by default)") that contradict story AC-7's Vec<String> default-deny design. Per CLAUDE.md Source-of-Truth Precedence Rule 1, BC text supersedes for contract semantics; stale EC-17-007 was a security drift risk.

**Story site (F-LP31-HIGH-002 STORY SITE):** story-writer added §BC Amendments In-Scope section directing product-owner to amend BC-2.17.002 EC-17-007. Story v1.28 → v1.29.

**BC site (F-LP31-HIGH-002 BC SITE):** product-owner amended BC-2.17.002 v1.5 → v1.6.

**Before (BC-2.17.002 v1.5 EC-17-007 §Error Conditions table):**
```
EC-17-007 | E-PLUGIN-005 | host_http_request — allowlist deny | Request denied | N/A | Request allowed to any URL (open by default)
```

**After (BC-2.17.002 v1.6 EC-17-007 §Error Conditions table):**
```
EC-17-007 | E-PLUGIN-005 | host_http_request — allowlist deny | Request denied; returns Err(PluginError::AllowlistRejected { url }) | allowed_urls: vec![] (empty Vec = deny all; default-deny per AC-7) | URL does not match any entry in allowed_urls list using host-only == comparison; empty allowed_urls vec denies all requests
```

**BC-2.17.002 frontmatter:** v1.5 → v1.6; timestamp bumped.
**BC-2.17.002 §Changelog:** v1.6 row added — F-LP31-HIGH-002 EC-17-007 rewritten from pre-AC-7 allow-all to post-AC-7 default-deny semantics.
**BC-INDEX line 216:** BC-2.17.002 version annotation v1.5 → v1.6.
**BC-INDEX frontmatter:** v4.71 → v4.72; v4.72 changelog row added.

**Product-owner sibling sweep:** 4/4 CLEAN — no remaining stale EC-17-007 "open by default" text in active spec corpus.

**POL-14 note:** BC-2.17.002 remains `lifecycle_status: draft` — promotion to active gated on PREREQ-D PR merge per POL-14.

---

### F-LP31-MED-001 — AC-15 §Credential Safety AuthToken Debug Example Mismatch

**Class:** implementer ambiguity — story AC-15 §Credential Safety prescribed `AuthToken("[REDACTED]")` (double-quote form, uppercase REDACTED) but existing auth_provider.rs:68 uses `AuthToken(<redacted>)` (angle-bracket form, lowercase, no inner quotes).

**Before (story AC-15 §Credential Safety):**
```rust
// wrong: AuthToken("[REDACTED]")
```

**After (story AC-15 §Credential Safety):**
```rust
// correct: AuthToken(<redacted>)
```

**Story-writer sibling sweep:** 5/5 CLEAN — no other sites in story body use the wrong form.

---

## Sibling-Site Catch — BC-2.17.002 Version Pin Propagation (state-manager scope)

**Catch class:** TD-VSDD-060 sibling-site sweep — product-owner bumped BC-2.17.002 to v1.6 in the BC file and BC-INDEX; state-manager sweeps the story body for active-body version pins that reference the old v1.5.

**Site found:** story line 373 AC-9 trace header:
```
### AC-9 — Single shared reqwest::Client constructed once at boot with 30-second timeout; injected into PluginRuntime (traces to BC-2.17.002 v1.5 §Error Conditions E-PLUGIN-005; closes TD-S-PLUGIN-PREREQ-B-005)
```

**After (state-manager closure scope):**
```
### AC-9 — Single shared reqwest::Client constructed once at boot with 30-second timeout; injected into PluginRuntime (traces to BC-2.17.002 v1.6 §Error Conditions E-PLUGIN-005; closes TD-S-PLUGIN-PREREQ-B-005)
```

**Sweep scope:** grep `BC-2.17.002 v1\.5` across story body — found 1 active-body site (line 373). Remaining matches at lines 1000, 1002, 1005, 1056, 1062, 1073 are all in §BC Amendments In-Scope section, §Changelog rows, and historical audit-trail rows — immutable, not updated per TD-VSDD-091 convention.

**Sibling catch count:** 1 site propagated (story line 373).

---

## Sibling-Sweep Results

**Story-writer (3 story fixes):** 5/5 CLEAN
**Product-owner (BC amendment + BC-INDEX):** 4/4 CLEAN
**State-manager (sibling version-pin catch):** grep BC-2.17.002 v1.5 across story — 1 active-body site caught and fixed; remaining matches are audit-trail/changelog (immutable).

---

## Commit Discipline

- TD-VSDD-053: single atomic commit covering all 6 artifact modifications (story v1.29 + BC-2.17.002 v1.6 + BC-INDEX v4.72 + STORY-INDEX v2.99 + STATE.md v7.231 + SESSION-HANDOFF v7.231).
- POL-3: state-manager ran last; sibling-catch applied in closure commit.
- POL-11: STORY-INDEX v2.98 → v2.99 + BC-INDEX v4.71 → v4.72 both mandatory bumps applied.
- POL-14: BC-2.17.002 remains draft; promotion deferred to PREREQ-D PR merge.
- TD-VSDD-060: sibling-site sweep applied to BC-2.17.002 v1.5 → v1.6 propagation in story body.
- TD-VSDD-091: changelog/audit-trail rows citing v1.5 preserved (historical record, immutable).
- No Co-Authored-By. No --no-verify.
