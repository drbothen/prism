---
document_type: proposed-adr
status: capture
do_not_execute: true
provenance: "2026-06-26 side-analysis — day-2 design decision capture; PROPOSED, gated on brief-reframe sign-off; separate from live factory."
proposed_number: "ADR-PROP-sandboxed-expression-evaluator"
note_on_numbering: "Real number allocated by architect at morph time via create-adr skill. ADR-047–054 are reserved in matured-vision §5.4. Do not reuse."
related_specs:
  - spike evaluator (ANTLR4): /Users/jmagady/Dev/aletheon_2/spike/dashboard/src/dsl/evaluator.ts
  - spike grammar: /Users/jmagady/Dev/aletheon_2/spike/dashboard/src/dsl/grammar/BindingExpr.g4
  - spike Function() evaluator (REJECTED path): /Users/jmagady/Dev/aletheon_2/spike/dashboard/src/components/primitives/realtime/expressionEvaluator.ts
  - disposition: .factory/specs/day2-ui-design/S3-conversational-canvas-disposition.md §4.1 item 2
  - ADR-PROP-widget-dsl-render-and-schema-validation (governs the render layer this sits within)
---

# ADR-PROP: Sandboxed Expression Evaluator (ANTLR4) for Widget Data-Binding

## Status

PROPOSED — capture artifact. Gated on brief-reframe sign-off per disposition §9.

## Context

The widget DSL (adopted via ADR-PROP-widget-dsl-render-and-schema-validation) includes reactive
container primitives that support `{{expression}}` binding syntax. Examples from the grammar:

- `{{cpu}}` — direct field access
- `{{cpu >= 90 ? 'critical' : 'normal'}}` — conditional expression
- `{{round(cpu, 2)}}` — built-in function call
- `{{events[0].severity}}` — array index + member access
- `{{value ?? 'N/A'}}` — nullish coalescing

In prism's S3 context, these bindings are present in widget schemas emitted by an LLM that
processes attacker-influenceable OCSF data. The expression syntax is a direct code-execution
surface if not carefully constrained.

### The Function() Vulnerability

The spike contains two expression evaluator implementations:

**Path 1 (rejected): `expressionEvaluator.ts`** (at
`spike/dashboard/src/components/primitives/realtime/expressionEvaluator.ts`)

This implementation uses `new Function()`:

```typescript
const fn = new Function(...contextKeys, `"use strict"; return (${safeExpression});`);
return fn(...contextValues);
```

`new Function(string)` compiles and executes arbitrary JavaScript at runtime. The spike's
attempt to "sanitize" by pre-processing `?.` and `??` with string replacement is not a security
boundary — it is a cosmetic transformation that does not prevent injection:

- `{{constructor.constructor('alert(1)')()}}` bypasses the variable-allowlist approach.
- `{{[].constructor.constructor('fetch(...)')()}}` accesses Function via prototype chain.
- Any LLM-influenced string that reaches `new Function()` as the body argument is a code
  execution vector. Prism processes LLM output derived from attacker-influenceable OCSF data
  (hostnames, event titles, process command lines). This is an unacceptable risk surface.

**Path 2 (adopted): `evaluator.ts`** (at
`spike/dashboard/src/dsl/evaluator.ts`)

This implementation uses an ANTLR4-generated parser (`BindingExprLexer`, `BindingExprParser`)
from a formal grammar (`BindingExpr.g4`). The expression is parsed to a concrete syntax tree;
a tree-walking `ExpressionEvaluator` class evaluates the AST against a data context. No
`eval()`, no `new Function()`, no string-template-to-code path.

The grammar (`BindingExpr.g4`) is already authored, tested, and the ANTLR4 TypeScript parser
is pre-generated in `spike/dashboard/src/dsl/generated/`. This is NOT a rewrite — it is a
configuration choice of which evaluator path ships in prism.

The disposition §4.1 item 2 mandates:

> "The `{{expr}}` binding syntax in reactive container nodes must be evaluated via the ANTLR4
> grammar-parsed evaluator (`evaluator.ts`), NOT via `new Function()` or `eval()`."

### Threat Model Rationale

The prompt-injection threat model for `{{expr}}` bindings in prism S3:

1. Attacker controls OCSF data fields (e.g., process.name = `'{{constructor.constructor(...)()}}}`).
2. LLM processes attacker data and emits a widget schema containing binding expressions.
3. The LLM itself may be injected into including attacker-crafted expression values.
4. The expression evaluator executes those values.

With `new Function()`: step 4 is JavaScript execution. With the ANTLR4 evaluator: step 4 is
bounded AST evaluation that can only access explicitly allowlisted data and built-in functions.
The grammar defines the complete language; anything outside the grammar fails at parse time with
a `ParseError`, not at runtime with unexpected execution.

## Decision

**Mandate the ANTLR4-grammar-parsed evaluator (`evaluator.ts`) as the exclusive path for
evaluating `{{expression}}` bindings in widget DSL reactive containers. The `Function()`-based
evaluator path (`expressionEvaluator.ts`) is explicitly prohibited in prism S3.**

### Evaluator Specification

The prism S3 evaluator is based on the spike's `evaluator.ts` with the following constraints
explicitly enforced:

#### No JavaScript Execution

- `eval()` is never called anywhere in the expression evaluation path.
- `new Function()` is never called anywhere in the expression evaluation path.
- `setTimeout(string)`, `setInterval(string)`, `document.write()`, and any equivalent
  string-to-code mechanisms are never used.
- The evaluator operates purely on the ANTLR4 CST: it walks the tree, dispatches to typed
  visitor methods, and returns a value. No code generation occurs.

#### No Prototype Chain Access

The grammar's `IDENTIFIER` rule matches `[a-zA-Z_][a-zA-Z0-9_]*`. The evaluator's
`visitPrimary` method resolves identifiers against two sources only:

1. The data context object passed to the evaluator (`Record<string, any>`).
2. The hardcoded built-in function registry.

Prototype chain properties (`constructor`, `__proto__`, `prototype`) are not in the data
context and not in the built-in registry. Resolution returns `undefined` for any name not in
those two sources. This blocks `constructor.constructor` and similar prototype-walk attacks.

An explicit denylist provides defense-in-depth: even if the data context were somehow
populated with `constructor`, the evaluator explicitly rejects the identifier names
`constructor`, `__proto__`, `prototype`, and `__defineGetter__` during identifier lookup,
returning `undefined` regardless of context content.

#### Built-in Function Allowlist

The built-in function registry is a closed, hardcoded allowlist. No dynamic extension at
runtime. The spike's current allowlist (reproduced for reference):

| Function | Signature | I/O constraints |
|---|---|---|
| `round` | `(n: number, decimals?: number) => number` | Pure math, no I/O |
| `floor` | `(n: number) => number` | Pure math, no I/O |
| `ceil` | `(n: number) => number` | Pure math, no I/O |
| `abs` | `(n: number) => number` | Pure math, no I/O |
| `min` | `(...n: number[]) => number` | Pure math, no I/O |
| `max` | `(...n: number[]) => number` | Pure math, no I/O |
| `upper` | `(s: string) => string` | Pure string, no I/O |
| `lower` | `(s: string) => string` | Pure string, no I/O |
| `len` | `(arr: any[]) => number` | Pure collection, no I/O |
| `format` | `(n: number, decimals?: number) => string` | Pure string, no I/O |

Every function in the allowlist is:
- Pure: no side effects, no I/O, no network, no DOM access.
- Bounded: terminates on all inputs in O(1) or O(n) where n is argument size.
- No higher-order: does not accept or return functions. The evaluator's function-call path
  resolves function names from the allowlist by name; it cannot execute arbitrary function
  values from the data context.

Adding a new built-in requires: (a) definition in the allowlist registry, (b) verification
that the function is pure and bounded, (c) security review sign-off. No new built-ins are added
at runtime.

#### Resource Bounds

Expressions that could produce denial-of-service at the evaluator level are bounded:

- **Parse time limit:** 50ms wall time for parsing an expression. Expressions that take longer
  are rejected with a `ParseTimeoutError`. The ANTLR4 lexer/parser is fast on inputs of
  realistic expression lengths (< 500 characters); this bound protects against adversarially
  crafted long expressions.
- **Expression length limit:** 500 characters. Expressions longer than this are rejected
  before being passed to the lexer.
- **Evaluation depth limit:** The tree-walker tracks recursion depth; expressions producing
  deeper than 50 recursive descent steps are terminated with `EvaluationDepthError`. This
  bounds stack depth for adversarially nested ternaries.
- **No loops:** the grammar does not define any loop construct. Iteration is only possible via
  function calls on arrays (e.g., `len(arr)`) whose built-ins are all bounded.

#### No I/O, No DOM, No Network

The `ExpressionEvaluator` class is constructed with a `data: Record<string, any>` context.
That context is populated only with:

1. Widget node data fields (OCSF-normalized values from the sensor query result).
2. Built-in function entries from the allowlist.

The context does not contain: `window`, `document`, `fetch`, `XMLHttpRequest`,
`localStorage`, `sessionStorage`, `crypto`, or any other browser API. The evaluator has no
mechanism to inject these because identifier resolution is a simple key lookup in a plain
object, not a scope-chain walk.

### Render Layer Position

The expression evaluator sits inside the widget renderer, called after the Zod schema
validation gate (see ADR-PROP-widget-dsl-render-and-schema-validation):

```
Validated widget schema
        │
        ▼
Widget Renderer (recursive renderNode())
        │ for each node with {{expr}} binding props:
        ▼
[ANTLR4 evaluator]
    parse expression (≤50ms, ≤500 chars)
    resolve identifiers (data context + allowlist only)
    return typed value or undefined
        │
        ▼
React component prop (typed value)
```

`undefined` results from failed expressions are rendered as empty strings / zero / false
depending on the prop type's fallback policy. They never throw unhandled exceptions into the
React render tree.

### Migration from Function() Path

When porting from the spike to prism:

1. `expressionEvaluator.ts` (the `Function()` path) is NOT included in the prism codebase.
2. All imports of `evaluateExpression`, `resolveValue`, and `resolveBindings` in reactive
   container components reference `dsl/evaluator.ts` (the ANTLR4 path).
3. A compile-time guard: a lint rule (`no-restricted-imports` or a custom ESLint rule) bans
   any import from `expressionEvaluator.ts` or any direct use of `new Function` in the
   `dashboard/` TypeScript source tree. This is the same pattern prism uses for compile-fail
   gates in Rust (e.g., `tests/external/perimeter-violation/`).
4. CI enforcement: the lint rule runs in CI and blocks merge on violation.

## Consequences

**Positive:**
- The `new Function()` vector is eliminated. An attacker-influenced expression string has no
  path to JavaScript execution.
- Prototype chain attacks (`constructor.constructor`) cannot succeed because the evaluator
  does not walk the prototype chain during identifier resolution.
- Resource bounds (parse timeout, expression length, evaluation depth) make the evaluator
  resistant to denial-of-service via adversarially crafted binding expressions.
- The grammar is formally specified and versioned; any extension to the expression language
  requires a grammar change, which is a reviewable artifact (not an ad-hoc string-manipulation
  patch).
- The spike already has the grammar and generated parser — adopting it is configuration, not
  implementation from scratch.

**Negative / trade-offs:**
- The ANTLR4 evaluator is described in the spike's own docs as "slightly slower but more
  robust" than the `Function()` path. For expression-heavy widget schemas (e.g., a table with
  reactive bindings in every cell, 1000 rows), the per-expression parse overhead accumulates.
  Mitigation: expression parse results are memoized by expression string within a single render
  cycle; parsed ASTs are reused for repeated evaluations of the same expression against
  different data contexts.
- The ANTLR4 runtime (`antlr4ng`, ~100KB minified) is a new bundle dependency. This is a
  known and acceptable cost for the security benefit.
- The grammar does not include advanced JavaScript features (template literals, destructuring,
  spread, regex). This limits expression expressiveness — by design.

## Alternatives Considered

**A. Use the `Function()`-based evaluator with a hardened allowlist approach.** Rejected: the
history of JavaScript sandboxing via `new Function()` with variable allowlists is a history of
bypasses. Prototype chain access, accessor property descriptors, and cross-realm references
provide multiple bypass vectors that cannot all be closed via input pre-processing. The ANTLR4
approach eliminates the attack surface category, not individual bypass paths.

**B. Use a third-party expression sandboxing library (e.g., `expr-eval`, `jexl`, `filtrex`).**
These libraries provide pre-built sandboxed expression evaluators. They were not adopted for
prism because: (a) the spike already ships a working, tested ANTLR4 implementation, making
a third-party library redundant overhead; (b) third-party libraries introduce a dependency
with its own security update cadence; (c) the ANTLR4 grammar is fully auditable — the grammar
file IS the threat model. With a third-party library, the threat model depends on their
internal implementation choices.

**C. Prohibit all `{{expr}}` bindings — use static widget schemas only.** This eliminates the
evaluator attack surface entirely but removes reactivity from the widget DSL. Reactive
bindings are used in metric cards (showing live-updating values), status widgets, and
conditional visibility. Losing them significantly degrades the UX value of the canvas. The
ANTLR4 evaluator is the correct trade-off: bounded, auditable reactivity without JavaScript
execution.

**D. Server-side expression evaluation.** Evaluate `{{expr}}` on the server before sending the
rendered widget to the browser. This would eliminate the client-side attack surface but:
(a) requires a server round-trip for every data update in reactive widgets, breaking the
real-time SSE-driven update model; (b) moves the evaluation into the Rust backend, which has
no natural TypeScript/ANTLR4 implementation path; (c) the data context for expressions is
ephemeral browser-side widget state. Server-side evaluation is architecturally incompatible
with the ephemeral-canvas design.

## Open Decisions for Human

1. **Memoization scope.** The decision recommends memoizing parsed ASTs by expression string
   within a single render cycle. Should the memo scope be broader (e.g., per-session, since
   the same `{{round(cpu, 2)}}` expression appears in many widgets)? Broader memoization
   saves parse cost but holds parsed AST objects in memory longer. With the 500-character
   expression length limit and typical widget schemas, the memory cost is bounded — but the
   explicit scope is a human call.

2. **Extended built-in functions for SOC workflows.** The current allowlist covers math and
   string formatting. SOC analyst widget bindings may want date/time formatting (e.g.,
   `formatTimestamp(ts, 'relative')`, `formatSeverity(severity_id)`). Should these be
   pre-included in the allowlist, or strictly gate them behind the Primitive Upgrade Protocol
   in a follow-on story? Pre-including avoids a later story; strict gating preserves the
   invariant that no built-in ships without security review.

3. **Compile-time guard mechanism.** The decision specifies an ESLint `no-restricted-imports`
   rule banning `expressionEvaluator.ts`. Should this be a custom ESLint plugin (stronger,
   catches dynamic require paths) or a standard `no-restricted-imports` config (simpler,
   sufficient for static imports)? The former requires more setup; the latter is adequate for
   the current codebase structure where imports are static.
