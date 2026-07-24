# Mechanical-Gate Coverage Parity

## Why This Lesson Exists

On 2026-07-24, the CLIP email-notifications Stage-3 adversarial cascade introduced the
records-lint gate (`scripts/records-lint.sh`, TD-VSDD-092) as a blocking check for
record-tier violations. During initial implementation, the self-probe revealed that three
of six claimed checks silently returned exit 0 on synthetic violations (false-green).

Root causes:
- `extract_frontmatter_version` used a 3-argument `match()` form not supported by
  macOS BSD awk; the function returned empty output and the check short-circuited to pass.
- `extract_changelog_versions` matched version numbers from ANY column in a pipe-table
  row, not just the version column; version strings embedded in burst-ID and change-
  description columns produced false positives and false negatives.
- The git staging probe in the L9 self-probe used `.factory` as the subdirectory name,
  which triggered the factory-dispatcher destructive-command guard on cleanup.

In each case the check CLAIMED to enforce a rule but did not. A gate deployed in
blocking mode with unprobed checks is worse than no gate: it creates false confidence
while silently passing violations.

---

## Lesson 1 — MECHANICAL-GATE COVERAGE PARITY

**Every check a gate CLAIMS must be self-probed against a synthetic violation before the
gate is deployed as a blocker. Claimed coverage that cannot demonstrate a failing case
on a known-bad input is not coverage — it is false confidence.**

### Implementation rule

Every automated check (pre-commit script, lefthook hook, CI job) must include or
reference a `--self-probe` mode that:

1. Constructs a minimal synthetic artifact that violates the rule.
2. Runs the check against the violating artifact.
3. Asserts the check EXIT CODE is non-zero.
4. Constructs a minimal synthetic artifact that SATISFIES the rule.
5. Runs the check against the clean artifact.
6. Asserts the check EXIT CODE is zero.

A check with no synthetic violation probe MUST be marked `[UNPROBED]` in the gate's
check list. `[UNPROBED]` checks are informational only and must not drive blocking behavior
until a probe is added.

### Failure signatures to recognize

| Symptom | Root cause |
|---------|-----------|
| Check always exits 0 | Underlying tool failure returns empty/silent output; check short-circuits on empty → false-green |
| Check blocks clean inputs | Extraction regex too broad; matches non-target data in adjacent columns or text |
| Self-probe passes but real violations slip through | Probe artifact is too simple; real artifacts have surrounding structure that changes regex anchoring |

### Prism application

`scripts/records-lint.sh --self-probe` is the canonical self-probe for the records-lint
gate. It must be run after any change to the script's check logic. CI invocation of the
gate should include: `bash scripts/records-lint.sh --self-probe && bash scripts/records-lint.sh`
— self-probe first to verify gate integrity, then the real check.

---

## Lesson 2 — SCAN-POPULATION COROLLARY

**When defects are found OUTSIDE the gate's current scan population, the correct response
is to extend the scan population — not to add a prose reminder.**

### Rule

If an adversary or reviewer finds a records-tier violation in a file class that the
mechanical gate does not cover (e.g., STATE.md decision-log entries, burst-log rows,
ratification memos), the fix has two parts:

1. Immediately fix the specific violation.
2. Extend the gate's `RECORD_DIRS_L9` or `VERSIONED_ARTIFACT_DIRS` config block to
   cover that file class, AND add a self-probe for the new population.

Documenting the gap as a TODO comment or prose advisory satisfies neither requirement.
A defect that recurs in a gap the gate does not cover is evidence that the gate's
population is wrong, not that humans need to be more careful.

### The prose-advisory anti-pattern

Prose advisories ("remember to check X") decay. Gates enforce. When you find yourself
writing "also manually verify that <rule> holds for <file class>," ask: why is that
file class not in the gate's scan population?

If the answer is "it's hard to check mechanically," that is a gate DESIGN problem to
solve — not a license to defer to human vigilance. The design goal is: the gate's
scan population equals the full set of files where the rule applies.

### Prism application

The records-lint gate's L9 scan population is currently `.factory` (entire factory
directory). Any `.md`, `.toml`, `.yaml`, or `.yml` file in `.factory` is in scope for
the line-cite ban. The `VERSIONED_ARTIFACT_DIRS` for L1/L7 currently covers BC, ADR,
and VP directories. Extension instructions are in the script's config block.

---

## Application

These lessons apply to every gate added to prism, not just records-lint:

- `scripts/check-non-exhaustive.sh` — confirmed self-probed (two-layer: count + per-symbol)
- `scripts/check-crate-layout.sh` — confirmed self-probed (violations are compile errors
  caught by `cargo check`; clean is a passing workspace)
- `scripts/records-lint.sh` — confirmed self-probed (6/6 pass/fail cases, 2026-07-24)
- TD-VSDD-093 (lefthook structured-event-catalog hook, P3 open) — must include a self-probe
  when implemented

When adding any new check to `lefthook.yml`, `.factory/hooks/`, or `scripts/`:
1. Write the `--self-probe` or equivalent before writing the main check logic.
2. The self-probe must fail (exit non-zero) BEFORE you add the check, confirming the
   probe itself works.
3. Then implement the check until the self-probe passes.

---

_Captured: 2026-07-24. Source: CLIP email-notifications Stage-3 cascade — records-lint
gate implementation, TD-VSDD-092 initial self-probe revealing 3/6 false-green checks.
Cross-applied from the CLIP email-notifications Stage-3 cascade (trend-gate #4
structural intervention + S3-39..S3-42 evidence), human-directed 2026-07-24._
