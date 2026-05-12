# AC-4 Evidence — Interpolator `$${...}` Context-Free Literal Escape

**Story:** S-PLUGIN-PREREQ-C v1.3
**Status: SATISFIED**
**Resolves:** TD-S-PLUGIN-PREREQ-B-008 (P3)
**BC anchor:** BC-2.16.002 postcondition — path_template and body_template are interpolated
against variables; the escape mechanism is a grammar extension of the interpolation surface.

---

## AC Summary (quoted from story v1.3)

> The `Interpolator` is updated to support a double-dollar escape sequence: `$${...}`
> interpolates to the literal string `${...}` without triggering variable substitution.
>
> **Exact escape semantics (context-free implementation):**
> The escape is context-free — any `$$` pair collapses to `$` regardless of what follows.
>
> - `$${var}` → literal `${var}` (no lookup in variable map)
> - `${var}` → interpolated value of `var` (existing behavior, unchanged)
> - `$$${var}` → literal `$` followed by interpolated value of `var`

---

## Red Gate Tests

Four tests in the `interpolation::escape_tests` in-module block within
`crates/prism-spec-engine/src/interpolation.rs`:

| Test | Assertion |
|------|-----------|
| `test_BC_2_16_002_interpolator_escape_double_dollar` | `$${var}` with `var="hello"` in scope → literal `${var}`, NOT `hello` |
| `test_BC_2_16_002_interpolator_live_reference_unaffected` | `${var}` with `var="hello"` in scope → `hello` (backward compat) |
| `test_BC_2_16_002_interpolator_triple_dollar_escape` | `$$${var}` with `var="hello"` → `$hello` (one literal dollar + interpolated) |
| `test_AC4_escape_context_free_double_dollar_to_single` | HIGH-003 locking test: context-free `$$` collapse verifies implementation is not context-sensitive |

---

## Real Test Output

### Escape tests (3 named BC tests):

```
$ cd /Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-C && \
  cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_16_002_interpolator_)' --no-fail-fast

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
────────────
 Nextest run ID a09b65af-7dc3-4bf2-96f2-5b73da971036 with nextest profile: default
    Starting 3 tests across 18 binaries (324 tests skipped)
        PASS [   0.012s] (1/3) prism-spec-engine interpolation::escape_tests::test_BC_2_16_002_interpolator_live_reference_unaffected
        PASS [   0.013s] (2/3) prism-spec-engine interpolation::escape_tests::test_BC_2_16_002_interpolator_triple_dollar_escape
        PASS [   0.013s] (3/3) prism-spec-engine interpolation::escape_tests::test_BC_2_16_002_interpolator_escape_double_dollar
────────────
     Summary [   0.013s] 3 tests run: 3 passed, 324 skipped
```

### HIGH-003 locking test:

```
$ cd /Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-C && \
  cargo nextest run -p prism-spec-engine -E 'test(test_AC4_escape_)' --no-fail-fast

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
────────────
 Nextest run ID 6d1e5cdd-b6df-4700-ae82-3e611400e41c with nextest profile: default
    Starting 1 test across 18 binaries (326 tests skipped)
        PASS [   0.011s] (1/1) prism-spec-engine interpolation::escape_tests::test_AC4_escape_context_free_double_dollar_to_single
────────────
     Summary [   0.012s] 1 test run: 1 passed, 326 skipped
```

All 4 tests pass.

---

## Production Code Reference

**File:** `crates/prism-spec-engine/src/interpolation.rs`

**`Interpolator::interpolate`** is the public entry point for template substitution. Before
the regex-based `${var}` substitution loop runs, the method calls `replace_double_dollar_escapes`
to perform a pre-pass over the template string.

**`replace_double_dollar_escapes`** performs a single pass through the template, collapsing
every `$$` pair into a single `$`. This is deliberately context-free: the function does not
inspect what follows the `$$` pair. The collapse fires regardless of whether the next character
is `{`, another `$`, or anything else. This choice:

- Avoids lookahead complexity (no need to parse brace-matching)
- Gives TOML spec authors a simple mental model: "write `$$` anywhere to get a literal `$`"
- Matches the expected triple-dollar behavior: `$$${var}` becomes `$${var}` after the escape
  pre-pass, and the remaining `${var}` is then live-interpolated to the variable's value

After `replace_double_dollar_escapes` runs, the standard `${var}` regex substitution loop
operates on the collapsed template. Any `${...}` token that survived the escape pre-pass
(i.e., was not preceded by `$$`) is subject to variable lookup.

**Escape semantics summary (from `Interpolator::interpolate` doc-comment):**
- `$${var}` → after pre-pass becomes `${var}` (one dollar + brace) → regex no longer sees a
  live reference because the brace is not preceded by a `$` in the original position — literal
  `${var}` output
- `$$${var}` → pre-pass collapses `$$` to `$`, leaving `${var}` live → interpolated to `$value`

---

## Cross-References

- BC-2.16.002 v1.10 postcondition: interpolation surface grammar extension
- HIGH-003 closure: `test_AC4_escape_context_free_double_dollar_to_single` locks the
  context-free implementation as the specified behavior, preventing regression to an earlier
  draft that attempted context-sensitive lookahead (which failed the triple-dollar case)
- Fix-burst-1 closure: escape semantics were clarified from "brace-aware" to "context-free"
  during adversary pass-1 review; story v1.1 amended AC-4 narrative to match
