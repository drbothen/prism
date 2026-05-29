---
document_type: lessons
story_id: S-5.01-FOLLOWUP-MCP-BOOT
captured_at: "2026-05-29"
---

# S-5.01-FOLLOWUP-MCP-BOOT Lessons

## Lesson 1: Cross-reviewer asymmetry is real — security catches what LOCAL misses

**Finding:** SEC-001 CWE-22 path traversal in `add_sensor_spec` was not caught in the 19-pass LOCAL cascade. The security reviewer caught it at PR-LEVEL pass 12.

**Root cause:** LOCAL adversary focused on behavioral correctness and API contract coverage. Security reviewer applies a threat-model lens (attacker-controlled input → filesystem escape) that LOCAL does not apply systematically.

**Codification:** PR-LEVEL security review is NOT redundant with LOCAL convergence. The information asymmetry (LOCAL = implementer context; PR-LEVEL security = fresh threat model) is a feature, not a redundancy. The security pass must remain mandatory even after clean LOCAL convergence.

**Class:** Recurrence of lesson captured at PR-155 (SEC-001 base_url NO-OP) and PR-163 — cross-reviewer asymmetry lesson repeating. Evidence that LOCAL convergence alone is insufficient for security properties.

## Lesson 2: PR-LEVEL adversarial cascade catches PRODUCTION bugs that LOCAL doesn't

**Finding:** The real shutdown race bug in `serve_with_transport_and_shutdown_inner` (natural_close_fut arm masking `JoinError::Panic`) was caught during CI investigation at PR-LEVEL pass 8, not in LOCAL cascade.

**Root cause:** LOCAL cascade runs tests in isolation. The race condition only manifested under CI's parallel test execution (real tokio runtime behavior). LOCAL focused on API coverage, not concurrency invariants.

**Codification:** CI flaky test failures during PR review are adversary signals, not noise. A flaky test that passes locally but fails in CI must be investigated as a potential real production bug before attributing to test infrastructure.

## Lesson 3: Sibling-sweep discipline (TD-VSDD-060) keeps recurring as the dominant defect class

**Pattern:** Across passes 5, 7, 11, and 14-16, the dominant finding class was sibling-sweep misses — a fix applied to a primary call site without propagating to sibling call sites handling the same validation or transformation.

**Affected surface:** `validate_text_field` and related validation helpers — each extension to one validate_* function required searching for all other validate_* callers with the same pattern.

**Codification:** Every fix to a `validate_*` or `sanitize_*` helper MUST include a workspace grep for all other `validate_*` callers before declaring the fix complete. This is already in CLAUDE.md TD-VSDD-060 but recurred 4+ times in this story. Add to implementer pre-commit checklist.

## Lesson 4: Paper-fix detection (TD-VSDD-059) is high-value

**Finding:** PR-LEVEL pr-reviewer caught a paper-fix at pass 3 where the implementer had addressed a finding by adding a doc-comment asserting the fix rather than implementing the structural change.

**Codification:** TD-VSDD-059 paper-fix detection is not optional. Per the Self-Audit Checklist in CLAUDE.md: "Did I paper-fix a finding by renaming, doc-commenting, or asserting-only when the real fix is structural?" This checklist item must be run BEFORE declaring each fix-burst complete.

## Lesson 5: Pre-push hook gap — macOS-only means cross-platform failures only surface in CI

**Finding:** Windows MSVC CI failures caused by hardcoded `/tmp/` paths in test fixtures. Pre-push hook runs only on macOS (developer machine), so `/tmp/` paths were not caught pre-push.

**Codification:** High-risk PRs touching path-sensitive code should consider cross-platform testing in pre-push or at minimum should include a grep for hardcoded `/tmp/` paths. Workspace convention: use `tempfile::tempdir()` for ALL fixture paths in tests. This pattern should be enforced by a clippy lint or CI check.

**Convention established:** All test fixture temporary paths must use `tempfile::tempdir()`. Hardcoded `/tmp/sensor_spec` or similar paths are a test convention violation.

## Lesson 6: Path traversal (CWE-22) must be probed in every MCP tool that accepts a filesystem path

**Finding:** `add_sensor_spec` accepted an arbitrary path from the MCP client without canonical path validation.

**Standing adversary probe (proposed):** For any MCP tool that accepts a path parameter, adversary must verify: (1) path is canonicalized via `std::fs::canonicalize()` or equivalent; (2) canonicalized path is contained within the expected root directory via `.starts_with(root_dir)`; (3) path does not traverse outside the intended directory even with relative components.

**Note:** This is the same class as the PR-155 SEC-001 finding (base_url NO-OP at adapter layer). Different surface (filesystem path vs HTTP endpoint routing) but same root cause: implementer trusts user-supplied string without containment validation.
