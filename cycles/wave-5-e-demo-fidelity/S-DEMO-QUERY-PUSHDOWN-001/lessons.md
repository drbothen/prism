# Lessons Learned — S-DEMO-QUERY-PUSHDOWN-001 (wave-5-e-demo-fidelity)

## Codification Candidates

### [process-gap] ADV-P04-OBS-001 — Test-docstring file.rs:NNN line-pin discipline gap (TD-VSDD-091 adjacency)

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1011
**Finding:** ADV-P04-OBS-001 (v2.x LOCAL pass-4)
**Tags:** [process-gap] [codification-candidate] [TD-VSDD-091-adjacency]
**Classification:** NOT blocking — process improvement candidate only.

**Description:**
Several test function doc-comments in `crates/prism-dtu-crowdstrike/` and `crates/prism-query/src/pushdown.rs` cite source-file locations as `file.rs:NNN` (line-number pins). TD-VSDD-091 (anti-volatile-pin discipline) prohibits line-number pins in narrative spec content because they decay on subsequent diffs. The rule is currently enforced by adversary pass probes for `.factory/` SPEC artifacts.

However, there is no automated enforcement path for test-docstring line-number pins inside `crates/**/*.rs` files. A test's `/// See file.rs:123` style comment is technically in-code narrative — it can decay the same way a spec pin does — but it falls outside the current TD-VSDD-091 adversary probe scope and no pre-commit lint exists for it.

**Codification direction (for future session-reviewer / factory process improvement):**
- Option A: Extend adversary SAP probe to grep `crates/**/*.rs` doc-comments for `\w+\.rs:\d+` pattern and flag as LOW findings.
- Option B: Add a clippy custom lint or a pre-commit hook regex scan for `\w+\.rs:\d+` inside `///` comment lines.
- Option C: Record as CLAUDE.md standing convention and address opportunistically during fix-bursts that touch doc-comments.

**Resolution:** Option C is the minimum bar; Option A is the right permanent fix. Codification decision deferred to session-reviewer at cycle-close. Do NOT add a new story now — this is a process/lint gap, not an unmet user-facing requirement.

**Non-blocking confirmation:** The line-pin sites do not affect test behavior or correctness. Code passes `just check 4032/4032`. This record exists only as a cycle-close codification prompt.
