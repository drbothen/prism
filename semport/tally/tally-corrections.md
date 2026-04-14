# Extraction Correction Log: tally

**Date:** 2026-04-13
**Source:** tally-extraction-validation.md
**Corrections applied:** 6 categories, 22 individual edits

---

## Correction 1: MCP Tool Count 23 -> 24

**Root cause:** `update_batch_status` at server.rs:1747 was missed by all analysis passes.

| File | Line(s) | Old | New |
|------|---------|-----|-----|
| tally-broad-sweep.md | 68 | "23 tools, 8 prompts" | "24 tools, 8 prompts" |
| tally-broad-sweep.md | 160 | "Each tool method (23 total)" | "Each tool method (24 total)" |
| tally-broad-sweep.md | 726 | "all 23 tools, 8 prompts" | "all 24 tools, 8 prompts" |
| tally-broad-sweep.md | 744 | "23 tools covering CRUD" | "24 tools covering CRUD" |
| tally-pass-1-deep-architecture.md | 55 | "23 tool methods" | "24 tool methods" |
| tally-pass-1-deep-architecture.md | 60 | "All 23 Tools" | "All 24 Tools" |
| tally-pass-1-deep-architecture.md | 193 | "All 23 tools" | "All 24 tools" |
| tally-pass-1-deep-architecture-r2.md | 122 | "23 tools, 8 prompts, 14 resources" | "24 tools, 8 prompts, 14 resources" |
| tally-pass-2-deep-domain-model.md | 534 | "23 tool input structs" | "24 tool input structs" |
| tally-pass-3-deep-behavioral-contracts.md | 568 | "The 23 MCP tools" | "The 24 MCP tools" |
| tally-pass-3-deep-behavioral-contracts.md | 590 | "MCP tool contracts (23 tools)" | "MCP tool contracts (24 tools)" |
| tally-pass-5-deep-conventions.md | 163 | "23+" | "24" |
| tally-pass-5-deep-conventions-r2.md | 33 | "23 tool methods" | "24 tool methods" |
| tally-pass-5-deep-conventions-r2.md | 124 | "all 23 tools" | "all 24 tools" |
| tally-coverage-audit.md | 44 | "23 tools, 8 prompts" | "24 tools, 8 prompts" |

**Total edits:** 15

---

## Correction 2: BC-4.03.001 Precondition Incomplete

**Root cause:** Only the Levenshtein arm was documented; two substring containment arms were omitted.

| File | Location | Old | New |
|------|----------|-----|-----|
| tally-pass-3-deep-behavioral-contracts-r2.md | BC-4.03.001 postconditions | "normalized Levenshtein >= 0.6" | "f.contains(name) \|\| name.contains(f) \|\| normalized_levenshtein >= 0.6" |

**Total edits:** 1

---

## Correction 3: git_store.rs LOC (Copy-Paste Error)

**Root cause:** Coverage audit row copied mcp/server.rs LOC (~3300) into the git_store.rs row.

| File | Location | Old | New |
|------|----------|-----|-----|
| tally-coverage-audit.md | line 24 | ~3300 | ~973 |

**Total edits:** 1

---

## Correction 4: cli/mod.rs LOC

**Root cause:** Underestimate in coverage audit.

| File | Location | Old | New |
|------|----------|-----|-----|
| tally-coverage-audit.md | line 25 | ~320 | ~552 |

**Total edits:** 1

---

## Correction 5: cli/rule.rs LOC

**Root cause:** Underestimate in coverage audit.

| File | Location | Old | New |
|------|----------|-----|-----|
| tally-coverage-audit.md | line 37 | ~350+ | ~630 |

**Total edits:** 1

---

## Correction 6: query/ and session.rs LOC Underestimates

**Root cause:** Systematic underestimation of smaller files in coverage audit.

| File | Location | Old | New |
|------|----------|-----|-----|
| tally-coverage-audit.md | query/ast.rs row | ~50 | ~112 |
| tally-coverage-audit.md | query/fields.rs row | ~55 | ~118 |
| tally-coverage-audit.md | query/error.rs row | ~30 | ~82 |
| tally-coverage-audit.md | session.rs row | ~80 | ~95 |

**Total edits:** 4

---

## Summary

| Category | Edits | Files Touched |
|----------|-------|---------------|
| Tool count 23->24 | 15 | 8 |
| BC-4.03.001 precondition | 1 | 1 |
| git_store.rs LOC | 1 | 1 |
| cli/mod.rs LOC | 1 | 1 |
| cli/rule.rs LOC | 1 | 1 |
| query/ + session LOC | 4 | 1 |
| **Total** | **22** | **9 unique files** |

**Not modified:** tally-extraction-validation.md (validation report documents original findings and should retain them as-is for audit trail).
