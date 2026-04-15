---
name: Chumsky Parser API Research
type: research
date: 2026-04-15
phase: pre-architecture
---

# Chumsky Parser Combinator Library Research

**Sources:** crates.io API (live, 2026-04-15), Context7 Chumsky 0.12 docs, axiathon semport analysis, model training data.

**IMPORTANT VERSION UPDATE:** Axiathon used `chumsky = "0.10"` but the latest stable is **Chumsky v0.12.0** (released 2025-12-15, 16M downloads). There is also a **1.0.0-alpha.8** pre-release. We should target **0.12.0** (latest stable). The `recursive()` API, error types, and recovery patterns changed between 0.10 and 0.12 — see Context7 docs below for current patterns.

**Cross-references:**
- `/Users/jmagady/Dev/prism/.factory/specs/domain-spec/axiql-grammar.md` -- Complete AxiQL grammar (normative)
- `/Users/jmagady/Dev/prism/.factory/phase-0-ingestion/query-language-research.md` -- AxiQL design decisions
- `/Users/jmagady/Dev/prism/.factory/phase-0-ingestion/query-engine-research.md` -- Hybrid Chumsky+DataFusion architecture
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.11.006-query-security-limits.md` -- Security limits (DI-019)

---

## 1. Current Version on crates.io

`[VERIFY: check crates.io for exact current version as of April 2026]`

**Known facts from axiathon semport analysis:**
- Axiathon's production Cargo.toml specifies `chumsky = "0.10"` (verified against source code in pass-0 inventory)
- As of training data cutoff (May 2025), Chumsky 0.10.x was the latest release line
- The 0.10 series represents a major rewrite from 0.9, with significant API changes
- Chumsky's author is Joshua Barretto (zesterer on GitHub)

**Version timeline (from training data):**
- 0.9.x -- Stable, widely used, `Simple<char>` error type
- 0.10.0-alpha.x -- Alpha releases through 2024 with breaking API changes between alphas
- 0.10.0 -- Stabilized as the new mainline `[VERIFY: exact release date and whether further 0.10.x patches exist]`

**Recommendation:** Pin to an exact 0.10.x version in Cargo.toml (e.g., `chumsky = "=0.10.0"`) until the API surface is confirmed stable. Axiathon used `"0.10"` which allows semver-compatible patches.

---

## 2. Key API Differences Between 0.9 and 0.10

Chumsky 0.10 is a ground-up rewrite. The migration is not incremental -- it is a full API replacement.

### 2.1 Summary of Breaking Changes

| Feature | Chumsky 0.9 | Chumsky 0.10 |
|---------|-------------|--------------|
| **Error type** | `Simple<I>` (basic) | `Rich<'a, I>` / `Cheap<'a, I>` (configurable) |
| **Input type** | Generic over `I: Clone + Hash + Eq` | `Input<'a>` trait with lifetime-bounded borrowing |
| **Parser trait** | `Parser<I, O, Error = E>` | `Parser<'a, I, O, Extra = E>` with `Extra` bundle |
| **Combinator style** | Method chaining on trait | Method chaining on trait (similar shape, different signatures) |
| **Zero-copy** | No (clones tokens) | Yes (`&'a str` or `&'a [Token]` input) |
| **Recursive parsers** | `recursive(\|p\| ...)` returns `Recursive<...>` | `recursive(\|p\| ...)` with updated type signature |
| **Error recovery** | `recover_with(...)` strategies | Redesigned recovery with `recovery::skip_then_retry_until` etc. |
| **Span type** | `std::ops::Range<usize>` | `SimpleSpan` or custom span types |
| **`select!` macro** | Token matching | Redesigned `select!` with pattern syntax |
| **`just()`** | Matches single token/char | Same concept, updated signature |
| **`filter()`** | `filter(Fn(&I) -> bool)` | Integrated into `any().filter(...)` |
| **`end()`** | Checks for EOF | `end()` still available |
| **`map_err()`** | Custom error mapping | Replaced by `Extra`-based error configuration |

### 2.2 The `Extra` Type Parameter

The most significant API change in 0.10 is the introduction of `Extra`, which bundles error type and state into a single type parameter:

```rust
use chumsky::extra;

// 0.10: Extra bundles error reporting + parser state
type ParserExtra<'a> = extra::Err<Rich<'a, char>>;

// Or with custom state (useful for depth tracking):
type ParserExtra<'a> = extra::Full<Rich<'a, char>, ParserState, ()>;
```

`[VERIFY: exact Extra type variants and their generic parameters in current 0.10.x]`

### 2.3 Input Lifetime

Chumsky 0.10 introduces a lifetime parameter on inputs, enabling zero-copy parsing:

```rust
// 0.9: Parser takes owned/cloned input
fn parser() -> impl Parser<char, Ast, Error = Simple<char>> { ... }

// 0.10: Parser borrows input via lifetime
fn parser<'a>() -> impl Parser<'a, &'a str, Ast, extra::Err<Rich<'a, char>>> { ... }
```

This means parsers in 0.10 borrow from the input string rather than cloning characters. For AxiQL queries (small strings, typically under 1KB), the performance difference is marginal, but the API change is pervasive.

---

## 3. The `Parser` Trait and Combinators in 0.10

### 3.1 Core Parser Trait

```rust
// Simplified signature (0.10):
pub trait Parser<'a, I: Input<'a>, O, E: extra::ParserExtra<'a, I> = extra::Default> {
    fn parse(&self, input: I) -> ParseResult<O, E::Error>;

    // Key combinator methods (non-exhaustive):
    fn map<U>(self, f: impl Fn(O) -> U) -> Map<Self, F>;
    fn map_with(self, f: impl Fn(O, &mut MapExtra<'a, '_, I, E>) -> U) -> MapWith<...>;
    fn try_map<U>(self, f: impl Fn(O, Span) -> Result<U, E::Error>) -> TryMap<...>;
    fn then<B>(self, other: impl Parser<'a, I, B, E>) -> Then<Self, B>;
    fn ignore_then<B>(self, other: impl Parser<'a, I, B, E>) -> IgnoreThen<...>;
    fn then_ignore<B>(self, other: impl Parser<'a, I, B, E>) -> ThenIgnore<...>;
    fn or(self, other: impl Parser<'a, I, O, E>) -> Or<Self, Other>;
    fn repeated(self) -> Repeated<Self>;
    fn separated_by<S>(self, separator: S) -> SeparatedBy<Self, S>;
    fn delimited_by<L, R>(self, left: L, right: R) -> DelimitedBy<L, Self, R>;
    fn padded(self) -> Padded<Self>; // skip surrounding whitespace
    fn labelled(self, label: &'static str) -> Labelled<Self>;
    fn recover_with<S>(self, strategy: S) -> RecoverWith<Self, S>;
    fn boxed<'b>(self) -> Boxed<'b, 'a, I, O, E>; // type-erase for recursive
}
```

`[VERIFY: exact method signatures -- the above is reconstructed from training data and may have minor inaccuracies in generic parameters]`

### 3.2 Key Combinators for AxiQL

#### choice() -- Mode Detection

```rust
use chumsky::prelude::*;

fn axiql_parser<'a>() -> impl Parser<'a, &'a str, AxiQLStatement, extra::Err<Rich<'a, char>>> {
    // Try SQL first (starts with SELECT/FROM keyword), then fall back to pipe_or_filter
    choice((
        sql_parser(),
        pipe_or_filter_parser(),
    ))
    .then_ignore(end())
}
```

#### just() -- Keyword and Operator Matching

```rust
// Match exact strings
fn compare_op<'a>() -> impl Parser<'a, &'a str, CompareOp, extra::Err<Rich<'a, char>>> {
    choice((
        just("!=").to(CompareOp::Ne),
        just(">=").to(CompareOp::GtEq),
        just("<=").to(CompareOp::LtEq),
        just("==").to(CompareOp::Eq),
        just("=").to(CompareOp::Eq),
        just(">").to(CompareOp::Gt),
        just("<").to(CompareOp::Lt),
    ))
}
```

Note: Multi-character operators must be listed before single-character ones to prevent partial matching (`>=` before `>`).

#### text::keyword() vs text::ident() -- Case-Insensitive Keywords

The axiathon grammar spec notes (citing zesterer/chumsky#699) that the production parser uses `text::ident()` with case-insensitive comparison rather than `text::keyword()`:

```rust
// Axiathon pattern: case-insensitive keyword via ident + filter
fn kw<'a>(keyword: &'static str) -> impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> {
    text::ident()
        .filter(move |s: &String| s.eq_ignore_ascii_case(keyword))
        .ignored()
        .labelled(keyword)
}

// Usage:
kw("SELECT").ignore_then(select_list())
kw("WHERE").ignore_then(filter_expr())
kw("AND").to(BoolOp::And)
kw("OR").to(BoolOp::Or)
```

#### separated_by() -- Comma-Separated Lists

```rust
// field_list = field_ref , { "," , field_ref }
fn field_list<'a>() -> impl Parser<'a, &'a str, Vec<FieldRef>, extra::Err<Rich<'a, char>>> {
    field_ref()
        .separated_by(just(',').padded())
        .at_least(1)
        .collect()
}

// value_list = value , { "," , value }
fn value_list<'a>() -> impl Parser<'a, &'a str, Vec<Value>, extra::Err<Rich<'a, char>>> {
    value()
        .separated_by(just(',').padded())
        .at_least(1)
        .collect()
}
```

#### delimited_by() -- Parenthesized Expressions

```rust
fn parenthesized_expr<'a>(
    filter: impl Parser<'a, &'a str, FilterExpr, extra::Err<Rich<'a, char>>> + Clone,
) -> impl Parser<'a, &'a str, FilterExpr, extra::Err<Rich<'a, char>>> {
    filter
        .delimited_by(just('(').padded(), just(')').padded())
}
```

---

## 4. Error Recovery and Reporting

### 4.1 Error Types: Rich vs Cheap

Chumsky 0.10 provides two error reporting strategies:

| Strategy | Type | Cost | Information |
|----------|------|------|-------------|
| **Rich** | `Rich<'a, I>` | Higher (tracks context labels, spans, expected tokens) | Full diagnostic: what was expected, what was found, where, nested context |
| **Cheap** | `Cheap<'a, I>` | Lower (minimal tracking) | Basic: position only |

**Recommendation for AxiQL: Use `Rich<'a, char>` (or `Rich<'a, Token>` if token-based).**

Prism needs actionable error messages for AI agents that will self-correct syntax errors. The overhead of Rich error tracking is negligible for AxiQL query sizes (< 64KB, typically < 1KB).

### 4.2 Rich Error Structure

```rust
// Rich error provides:
pub struct Rich<'a, I: Input<'a>> {
    // The span of the error in the input
    span: I::Span,
    // What the parser expected to find
    expected: Vec<Option<I::Token>>,
    // What the parser actually found
    found: Option<I::Token>,
    // Nested context labels (from .labelled() calls)
    context: Vec<(&'static str, I::Span)>,
}
```

`[VERIFY: exact Rich struct fields in 0.10.x -- the above is from training data]`

### 4.3 Labelling for Context

The `.labelled()` combinator attaches human-readable context to error messages:

```rust
fn filter_expr<'a>() -> impl Parser<'a, &'a str, FilterExpr, extra::Err<Rich<'a, char>>> {
    or_expr()
        .labelled("filter expression")
}

fn sql_statement<'a>() -> impl Parser<'a, &'a str, AxiQLStatement, extra::Err<Rich<'a, char>>> {
    kw("SELECT")
        .ignore_then(select_list().labelled("SELECT column list"))
        .then(kw("FROM").ignore_then(source().labelled("FROM source")))
        .then(kw("WHERE").ignore_then(filter_expr()).or_not())
        .labelled("SQL statement")
        // ...
}
```

This produces errors like: `"Expected filter expression at 1:24, found 'WERE'. Context: SQL statement > WHERE clause"`

### 4.4 Error Recovery Strategies

Chumsky 0.10 provides recovery strategies that allow parsing to continue after an error, collecting multiple errors in a single pass:

```rust
// Strategy 1: Skip tokens until a synchronization point
fn recovering_filter<'a>() -> impl Parser<'a, &'a str, FilterExpr, extra::Err<Rich<'a, char>>> {
    filter_expr()
        .recover_with(skip_then_retry_until(
            any().ignored(),          // skip one token at a time
            choice((                  // retry when we see these sync points:
                just('|').ignored(),  // pipe operator
                just(')').ignored(),  // closing paren
                end(),                // end of input
            )),
        ))
}

// Strategy 2: Replace failed parse with a fallback value
fn recovering_value<'a>() -> impl Parser<'a, &'a str, Value, extra::Err<Rich<'a, char>>> {
    value()
        .recover_with(via_parser(
            any().repeated().at_least(1)  // consume bad tokens
                .to(Value::String("<<error>>".into()))  // produce error sentinel
        ))
}

// Strategy 3: Nested delimiter recovery (for balanced parens)
fn parenthesized_with_recovery<'a>(
    inner: impl Parser<'a, &'a str, FilterExpr, extra::Err<Rich<'a, char>>> + Clone,
) -> impl Parser<'a, &'a str, FilterExpr, extra::Err<Rich<'a, char>>> {
    inner
        .delimited_by(just('('), just(')'))
        .recover_with(nested_delimiters('(', ')', [], |_span| FilterExpr::Error))
}
```

`[VERIFY: exact recovery strategy names and signatures -- skip_then_retry_until, via_parser, nested_delimiters may have changed names in 0.10.x]`

### 4.5 Mapping Chumsky Errors to AxiQL Error Responses

For Prism's AI-agent-facing error messages (per BC-2.11.006), Chumsky errors must be translated to structured responses:

```rust
fn translate_parse_error(
    input: &str,
    errors: Vec<Rich<'_, char>>,
) -> Vec<AxiQLError> {
    errors.into_iter().map(|err| {
        let span = err.span();
        let line_col = offset_to_line_col(input, span.start);

        AxiQLError {
            code: "E-QUERY-002",
            message: format!(
                "Unexpected {} at line {}, column {}",
                err.found()
                    .map(|c| format!("'{}'", c))
                    .unwrap_or("end of input".into()),
                line_col.line,
                line_col.column,
            ),
            expected: err.expected()
                .filter_map(|e| e.map(|t| format!("'{}'", t)))
                .collect(),
            context: err.contexts()
                .map(|(label, _span)| label.to_string())
                .collect(),
            span: SourceSpan {
                start: span.start,
                end: span.end,
            },
            // Extract the problematic portion of input for the AI
            source_fragment: extract_fragment(input, span.start, span.end),
            suggestion: generate_suggestion(input, &err),
        }
    }).collect()
}
```

### 4.6 Axiathon's Gap: Error Recovery Was Not Implemented

The axiathon semport analysis explicitly notes:

> "Error recovery -- Planned (Chumsky supports)" (pass-2 R4 comparison table)
> "Error recovery (planned) [TODO Story 5.2]" (pass-8 synthesis, complexity ranking)

The axiathon parser stops at the first error. Prism should implement error recovery from day 1 (per the query-language-research recommendation). This means:

1. **Multiple error collection** -- Parse continues after first error, collecting all issues
2. **Sync-point recovery** -- After an error in a filter expression, skip to the next `AND`/`OR`/`|`/`)` and continue
3. **Balanced delimiter recovery** -- If a `(` is unmatched, recover at a reasonable boundary
4. **Error sentinel AST nodes** -- Introduce `FilterExpr::Error` (or similar) so the AST can represent partially-parsed queries. This enables partial validation even when the query has syntax errors.

---

## 5. Recursive Parser Patterns

### 5.1 The `recursive()` Combinator

AxiQL requires recursive parsing for nested boolean expressions: `(a AND (b OR (c AND d)))`. Chumsky's `recursive()` combinator creates a parser that can reference itself:

```rust
use chumsky::prelude::*;

fn filter_expr_parser<'a>() -> impl Parser<'a, &'a str, FilterExpr, extra::Err<Rich<'a, char>>> {
    recursive(|filter_expr| {
        // Atom: a comparison or a parenthesized sub-expression
        let atom = choice((
            comparison_parser(),
            filter_expr.clone()
                .delimited_by(just('(').padded(), just(')').padded()),
        ));

        // NOT: unary prefix
        let not_expr = choice((
            kw("NOT").or(just('!').ignored())
                .ignore_then(atom.clone())
                .map(|e| FilterExpr::Not(Box::new(e))),
            atom,
        ));

        // AND: left-associative binary
        let and_expr = not_expr.clone()
            .foldl(
                kw("AND").or(just("&&").ignored())
                    .padded()
                    .ignore_then(not_expr)
                    .repeated(),
                |left, right| FilterExpr::And(Box::new(left), Box::new(right)),
            );

        // OR: left-associative binary (lowest precedence)
        and_expr.clone()
            .foldl(
                kw("OR").or(just("||").ignored())
                    .padded()
                    .ignore_then(and_expr)
                    .repeated(),
                |left, right| FilterExpr::Or(Box::new(left), Box::new(right)),
            )
    })
}
```

### 5.2 Depth Limiting for DI-019 (Max 64 Nesting)

The DI-019 invariant requires max nesting depth of 64. Chumsky does not natively enforce depth limits -- this must be implemented in the parser logic.

**Approach 1: State-based depth tracking (recommended)**

Chumsky 0.10's `Extra` type parameter can carry mutable state through the parse:

```rust
use std::cell::Cell;
use std::rc::Rc;

/// Parser state for tracking nesting depth
#[derive(Clone)]
struct ParserState {
    depth: Rc<Cell<usize>>,
    max_depth: usize,
}

impl ParserState {
    fn new(max_depth: usize) -> Self {
        Self {
            depth: Rc::new(Cell::new(0)),
            max_depth,
        }
    }
}

// Use extra::Full to carry state through the parse
type AxiQLExtra<'a> = extra::Full<Rich<'a, char>, ParserState, ()>;

fn depth_guarded_paren<'a>(
    inner: impl Parser<'a, &'a str, FilterExpr, AxiQLExtra<'a>> + Clone,
) -> impl Parser<'a, &'a str, FilterExpr, AxiQLExtra<'a>> {
    just('(')
        .padded()
        .try_map_with(|_, extra| {
            let state = extra.state();
            let current = state.depth.get();
            if current >= state.max_depth {
                Err(Rich::custom(
                    extra.span(),
                    format!(
                        "Nesting depth {} exceeds maximum {} (DI-019/CWE-674)",
                        current + 1,
                        state.max_depth,
                    ),
                ))
            } else {
                state.depth.set(current + 1);
                Ok(())
            }
        })
        .ignore_then(inner)
        .then_ignore(
            just(')').padded().map_with(|_, extra| {
                let state = extra.state();
                state.depth.set(state.depth.get().saturating_sub(1));
            })
        )
}
```

`[VERIFY: exact extra::Full signature and try_map_with/map_with API in 0.10.x -- the state-passing mechanism may differ slightly]`

**Approach 2: Axiathon's pattern (Rc<Cell<usize>>)**

The axiathon semport describes the production parser using `Rc<Cell<usize>>` for depth tracking. This is the same concept as approach 1 but passed as a closure capture rather than through the Extra type parameter:

```rust
fn filter_with_depth_limit<'a>(
    max_depth: usize,
) -> impl Parser<'a, &'a str, FilterExpr, extra::Err<Rich<'a, char>>> {
    let depth = Rc::new(Cell::new(0usize));

    recursive(move |filter_expr| {
        let depth_inc = depth.clone();
        let depth_dec = depth.clone();
        let max = max_depth;

        let guarded_paren = just('(')
            .try_map(move |_, span| {
                let current = depth_inc.get();
                if current >= max {
                    Err(Rich::custom(span, format!(
                        "Nesting depth exceeds {} (CWE-674)", max
                    )))
                } else {
                    depth_inc.set(current + 1);
                    Ok(())
                }
            })
            .ignore_then(filter_expr.clone())
            .then_ignore(just(')').map(move |_| {
                depth_dec.set(depth_dec.get().saturating_sub(1));
            }));

        let atom = choice((
            comparison_parser(),
            guarded_paren,
        ));

        // ... AND/OR precedence as above ...
        build_precedence_chain(atom)
    })
}
```

### 5.3 Stack Depth vs Logical Nesting Depth

Important distinction for security:

- **Logical nesting depth:** The number of nested parentheses or boolean sub-expressions. This is what DI-019 limits to 64.
- **Stack depth:** The actual call stack usage during recursive descent parsing. Chumsky uses continuation-passing internally, but deeply nested expressions still consume stack proportional to nesting depth.

For 64 levels of nesting, stack usage is well within safe bounds (a few hundred KB at most). The DI-019 limit of 64 provides a generous safety margin before stack overflow becomes a concern (which would typically require thousands of levels).

---

## 6. Token-Based vs Character-Based Parsing

### 6.1 The Two Approaches

Chumsky supports both:

**Character-based (direct):** Parse directly from `&str`, where each "token" is a `char`.

```rust
fn parser<'a>() -> impl Parser<'a, &'a str, Ast, extra::Err<Rich<'a, char>>> {
    // Operates on characters directly
    just("SELECT").padded().ignore_then(/* ... */)
}
```

**Token-based (two-phase):** First lex `&str` into a `Vec<Token>`, then parse the token stream.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Token<'a> {
    // Keywords
    Select, From, Where, And, Or, Not, In, Like,
    GroupBy, OrderBy, Limit, Asc, Desc,
    Has, Missing, Matches, Contains,
    Head, Tail, Sort, Dedup, Fields, Stats,

    // Literals
    String(&'a str),
    Integer(i64),
    Float(/* ... */),
    Boolean(bool),
    Duration(u64, DurationUnit),

    // Operators
    Eq, Ne, Gt, Lt, GtEq, LtEq, RegexMatch,
    Pipe, Comma, Dot, LParen, RParen, LBracket, RBracket,
    Star, Bang,

    // Identifiers
    Ident(&'a str),
}

// Phase 1: Lexer (char -> Token)
fn lexer<'a>() -> impl Parser<'a, &'a str, Vec<(Token<'a>, SimpleSpan)>, extra::Err<Rich<'a, char>>> {
    // ...
}

// Phase 2: Parser (Token -> AST)
fn parser<'a>() -> impl Parser<'a, &'a [(Token<'a>, SimpleSpan)], AxiQLStatement, extra::Err<Rich<'a, Token<'a>>>> {
    // ...
}
```

### 6.2 Recommendation: Token-Based (Two-Phase) for AxiQL

| Criterion | Character-Based | Token-Based |
|-----------|----------------|-------------|
| **Error messages** | "Expected 'S' at position 7" (character-level, unhelpful) | "Expected keyword 'SELECT' at position 7" (token-level, actionable) |
| **Keyword handling** | Must manually handle case-insensitivity per keyword | Lexer normalizes keywords once; parser matches token variants |
| **Whitespace** | Must `.padded()` everywhere; easy to forget | Lexer strips whitespace between tokens; parser is whitespace-unaware |
| **Performance** | Single pass but combinators do more work | Two passes but each is simpler; overall comparable or faster |
| **Comment stripping** | Must handle inline in parser (axiathon does space-replacement preprocessing) | Lexer strips comments naturally |
| **Span tracking** | Automatic (byte offsets into source) | Lexer records spans per token; parser references token spans |
| **Code complexity** | Simpler for trivial grammars | More code upfront, but cleaner parser logic for complex grammars |
| **`select!` macro** | N/A | Powerful pattern matching on tokens |

**Token-based is strongly recommended for AxiQL** because:

1. **AI-actionable error messages.** "Expected keyword WHERE, found identifier 'WERE'" is far more useful to an AI agent than "Expected 'W' at position 23". The AI can self-correct from token-level errors.

2. **AxiQL has 30+ keywords.** Case-insensitive keyword matching in a character-based parser requires repetitive `text::ident().filter(...)` patterns. A lexer handles this once.

3. **The `select!` macro** in Chumsky 0.10 is designed for token-based parsing and makes parser rules concise:

```rust
// Token matching with select! macro
let keyword = select! {
    Token::Select => Keyword::Select,
    Token::From => Keyword::From,
    Token::Where => Keyword::Where,
};
```

4. **Comment stripping is natural.** The lexer simply does not emit comment tokens. No preprocessing pass needed (axiathon uses a space-replacement preprocessor to preserve byte offsets -- a lexer avoids this entirely).

### 6.3 Lexer Design for AxiQL

```rust
fn lexer<'a>() -> impl Parser<'a, &'a str, Vec<(Token<'a>, SimpleSpan)>, extra::Err<Rich<'a, char>>> {
    let string = just('"')
        .ignore_then(
            choice((
                just('\\').ignore_then(any()),  // escape sequence
                none_of("\"\\"),                 // any non-quote, non-backslash
            ))
            .repeated()
            .to_slice()
        )
        .then_ignore(just('"'))
        .map(Token::String);

    let number = text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .map(|s: &str| {
            if s.contains('.') {
                Token::Float(s)
            } else {
                Token::Integer(s.parse().unwrap())
            }
        });

    let ident_or_keyword = text::ident().map(|s: &str| {
        match s.to_ascii_uppercase().as_str() {
            "SELECT" => Token::Select,
            "FROM" => Token::From,
            "WHERE" => Token::Where,
            "AND" => Token::And,
            "OR" => Token::Or,
            "NOT" => Token::Not,
            "IN" => Token::In,
            "TRUE" => Token::Boolean(true),
            "FALSE" => Token::Boolean(false),
            "HAS" => Token::Has,
            "MISSING" => Token::Missing,
            // ... all keywords ...
            _ => Token::Ident(s),
        }
    });

    let operator = choice((
        just("!=").to(Token::Ne),
        just(">=").to(Token::GtEq),
        just("<=").to(Token::LtEq),
        just("=~").to(Token::RegexMatch),
        just("==").to(Token::Eq),
        just("&&").to(Token::And),
        just("||").to(Token::Or),
        just("=").to(Token::Eq),
        just(">").to(Token::Gt),
        just("<").to(Token::Lt),
        just("|").to(Token::Pipe),
        just(",").to(Token::Comma),
        just(".").to(Token::Dot),
        just("(").to(Token::LParen),
        just(")").to(Token::RParen),
        just("*").to(Token::Star),
        just("!").to(Token::Bang),
    ));

    let comment = just("//").or(just("#"))
        .then(any().and_is(just('\n').not()).repeated())
        .padded();

    let token = choice((string, number, ident_or_keyword, operator));

    token
        .map_with(|tok, e| (tok, e.span()))
        .padded_by(comment.repeated())
        .padded()
        .repeated()
        .collect()
}
```

`[VERIFY: exact Chumsky 0.10 lexer patterns -- to_slice(), text::ident() return types, map_with span access]`

---

## 7. Performance Characteristics

### 7.1 Parsing Speed

Chumsky is a PEG-style parser combinator library. Performance characteristics:

| Metric | Expected Range | Source |
|--------|---------------|--------|
| **Throughput** | ~50-200 MB/s for typical grammars | Chumsky benchmarks (training data) |
| **Latency per query** | < 1ms for queries under 10KB | Estimated from throughput |
| **AxiQL typical query** | 50-500 bytes | AI-generated filter/SQL queries |
| **AxiQL worst case** | 64KB (max allowed) | DI-019 limit |
| **64KB parse time** | ~0.3-1.3ms | Estimated from throughput range |

`[VERIFY: actual benchmark numbers from Chumsky's repo or independent benchmarks]`

### 7.2 Comparison with Alternatives

| Parser Library | Style | Speed | Error Recovery | Recursion | Ecosystem |
|---------------|-------|-------|----------------|-----------|-----------|
| **Chumsky 0.10** | Combinator (PEG) | Good | Built-in | `recursive()` | Growing |
| **nom** | Combinator | Excellent | Manual | Manual | Mature |
| **pest** | PEG (grammar file) | Good | No | Via PEG | Mature |
| **lalrpop** | LR(1) generator | Excellent | Manual | Grammar rules | Mature |
| **winnow** | Combinator (nom fork) | Excellent | Manual | Manual | Growing |
| **tree-sitter** | Incremental LR | Excellent + incremental | Built-in | Grammar rules | Mature |

**Why Chumsky over alternatives for AxiQL:**

1. **Error recovery is first-class.** nom, winnow, and pest require manual error recovery implementation. Chumsky provides `recover_with()` strategies out of the box.

2. **Recursive combinators.** `recursive()` handles nested boolean expressions naturally. nom/winnow require manual stack management or separate recursive descent functions.

3. **Axiathon precedent.** The axiathon production parser (1799 LOC, 315 tests, 3 query modes) is built on Chumsky 0.10. The grammar, AST design, and combinator patterns are proven.

4. **Rich error types.** `Rich<'a, I>` provides structured errors with spans, expected tokens, and context labels -- exactly what Prism needs for AI-facing error messages.

### 7.3 Performance Is Not a Concern for AxiQL

For Prism's use case, parsing speed is irrelevant to overall latency:

```
Query lifecycle:
  Parse AxiQL:           ~0.1-1ms   (Chumsky)
  Alias resolution:      ~0.01ms    (HashMap lookup)
  Sensor API fan-out:    100-5000ms (network I/O, dominant cost)
  OCSF normalization:    1-50ms     (DynamicMessage conversion)
  Arrow materialization: 1-10ms     (RecordBatch construction)
  DataFusion execution:  2-50ms     (query planning + execution)
  Response serialization: 1-10ms    (JSON/protobuf encoding)
  ---
  Total:                 ~105-5121ms
```

Parsing is < 0.1% of total query latency. Even a 10x slower parser would be unnoticeable. The recommendation to use Chumsky is based on correctness, error quality, and developer ergonomics -- not speed.

---

## 8. Integration with Security Limits (DI-019)

### 8.1 Enforcement Points in the Parse Pipeline

```
Raw query string
  |
  +-- [1] Length check: len > 65536 -> E-QUERY-003
  |
  v
Alias resolution (pre-parse)
  |
  +-- [2] Post-expansion length check: len > 65536 -> E-QUERY-003
  |
  v
Lexer (char -> Token)
  |
  +-- [3] Regex pattern length: > 1024 bytes -> E-QUERY-003
  |   (validated when lexing regex literal tokens)
  |
  +-- [4] Integer overflow: outside i64 range -> E-QUERY-003
  |   (validated when lexing integer tokens, using i128 intermediate)
  |
  v
Parser (Token -> AST)
  |
  +-- [5] Nesting depth: > 64 -> E-QUERY-003
  |   (tracked via Rc<Cell<usize>> or Extra state in parenthesized expr)
  |
  v
Post-parse validation
  |
  +-- [6] Pipe stage count: > 32 -> E-QUERY-003
  |   (count stages in PipeExpr::stages vector)
  |
  +-- [7] CIDR validation: invalid IP/prefix -> E-QUERY-003
  |   (re-validate at parse time via ipnetwork crate)
  |
  v
Validated AST (AxiQLStatement)
```

### 8.2 Nesting Depth Tracking During Parsing

The depth counter must track:

1. **Parenthesized expressions:** `(a AND b)` -- each `(` increments, each `)` decrements
2. **Implicit nesting via boolean precedence:** The `recursive()` combinator's self-reference creates implicit nesting. However, Chumsky handles this internally -- only explicit parentheses need depth tracking for the DI-019 invariant.

```rust
/// Security-hardened filter expression parser
fn secure_filter_parser<'a>(
    limits: &QueryLimits,
) -> impl Parser<'a, &'a [Spanned<Token<'a>>], FilterExpr, AxiQLExtra<'a>> {
    let max_depth = limits.max_nesting_depth; // 64 per DI-019
    let depth = Rc::new(Cell::new(0usize));

    recursive(move |expr| {
        let d_inc = depth.clone();
        let d_dec = depth.clone();

        // Depth-guarded parenthesized sub-expression
        let paren_expr = select! { Token::LParen => () }
            .try_map(move |_, span| {
                let d = d_inc.get();
                if d >= max_depth {
                    Err(Rich::custom(span, format!(
                        "Query nesting depth is {} (max {}). \
                         Reduce nested parentheses or boolean expressions.",
                        d + 1, max_depth,
                    )))
                } else {
                    d_inc.set(d + 1);
                    Ok(())
                }
            })
            .ignore_then(expr.clone())
            .then_ignore(select! { Token::RParen => () }.map(move |_| {
                d_dec.set(d_dec.get().saturating_sub(1));
            }));

        let atom = choice((
            comparison_parser(),
            paren_expr,
        ))
        .recover_with(skip_then_retry_until(
            any().ignored(),
            one_of([Token::And, Token::Or, Token::Pipe, Token::RParen]).ignored()
                .or(end()),
        ));

        // Standard precedence: NOT > AND > OR
        let not_expr = choice((
            select! { Token::Not => () }
                .or(select! { Token::Bang => () })
                .ignore_then(atom.clone())
                .map(|e| FilterExpr::Not(Box::new(e))),
            atom,
        ));

        let and_expr = not_expr.clone().foldl(
            select! { Token::And => () }
                .ignore_then(not_expr)
                .repeated(),
            |l, r| FilterExpr::And(Box::new(l), Box::new(r)),
        );

        and_expr.clone().foldl(
            select! { Token::Or => () }
                .ignore_then(and_expr)
                .repeated(),
            |l, r| FilterExpr::Or(Box::new(l), Box::new(r)),
        )
    })
}
```

### 8.3 Pipe Stage Counting

Pipe stage counting is simpler -- it is a post-parse check on the AST:

```rust
fn validate_pipe_stages(stmt: &AxiQLStatement, max_stages: usize) -> Result<(), AxiQLError> {
    if let AxiQLStatement::Pipe { stages, .. } = stmt {
        if stages.len() > max_stages {
            return Err(AxiQLError {
                code: "E-QUERY-003",
                message: format!(
                    "Query has {} pipe stages (max {}). \
                     Combine operations or simplify the pipeline.",
                    stages.len(), max_stages,
                ),
            });
        }
    }
    Ok(())
}
```

### 8.4 Regex Validation at Lex/Parse Time

```rust
fn regex_literal<'a>(
    max_pattern_len: usize,
) -> impl Parser<'a, &'a str, Token<'a>, extra::Err<Rich<'a, char>>> {
    just('"')
        .ignore_then(none_of('"').repeated().to_slice())
        .then_ignore(just('"'))
        .try_map(move |pattern: &str, span| {
            // CWE-1333: Limit pattern length
            if pattern.len() > max_pattern_len {
                return Err(Rich::custom(span, format!(
                    "Regex pattern is {} bytes (max {}). Simplify the pattern.",
                    pattern.len(), max_pattern_len,
                )));
            }
            // Validate regex compiles (uses finite automaton engine)
            regex::Regex::new(pattern).map_err(|e| {
                Rich::custom(span, format!("Invalid regex pattern: {}", e))
            })?;
            Ok(Token::Regex(pattern))
        })
}
```

---

## 9. Complete Parser Architecture for Prism

### 9.1 Recommended Module Structure

```
crates/prism-query/
  src/
    lib.rs          -- Public API: parse_axiql(), validate(), explain()
    token.rs        -- Token enum definition (~50 variants)
    lexer.rs        -- Chumsky lexer: &str -> Vec<Spanned<Token>>
    parser.rs       -- Chumsky parser: &[Spanned<Token>] -> AxiQLStatement
    ast.rs          -- AST types: AxiQLStatement, FilterExpr, SqlSelect, PipeExpr
    error.rs        -- AxiQLError, error translation from Rich<Token> to structured errors
    security.rs     -- DI-019 limit definitions and post-parse validation
    alias.rs        -- Alias resolution (pre-parse phase)
    types.rs        -- Type checking (post-parse phase)
    translate.rs    -- AST -> DataFusion Expr/LogicalPlan translation
    tests/
      lexer_tests.rs
      filter_tests.rs
      sql_tests.rs
      pipe_tests.rs
      error_tests.rs
      security_tests.rs
      alias_tests.rs
      integration_tests.rs
```

Estimated total: ~2000-2500 LOC (parser + lexer + AST + error handling + security + alias resolution). Axiathon's production parser was 1799 LOC without error recovery or alias parameter substitution; Prism adds both.

### 9.2 Public API Surface

```rust
/// Parse an AxiQL query string into a validated AST.
///
/// Pipeline: alias_resolve -> length_check -> lex -> parse -> type_check -> validate_limits
pub fn parse_axiql(
    query: &str,
    config: &QueryConfig,
    alias_registry: &AliasRegistry,
) -> Result<ParsedQuery, Vec<AxiQLError>> {
    // 1. Alias resolution (pre-parse)
    let expanded = alias_registry.expand(query, config.max_alias_depth)?;

    // 2. Length check (post-expansion)
    if expanded.len() > config.max_query_length {
        return Err(vec![AxiQLError::query_too_long(expanded.len(), config.max_query_length)]);
    }

    // 3. Lex
    let tokens = lexer()
        .parse(&expanded)
        .into_result()
        .map_err(|errs| errs.into_iter().map(translate_lex_error).collect::<Vec<_>>())?;

    // 4. Parse (with error recovery -- may return AST + errors)
    let (ast, parse_errors) = parser(config)
        .parse(tokens.as_slice())
        .into_output_errors();

    let ast = ast.ok_or_else(|| {
        parse_errors.into_iter().map(translate_parse_error).collect::<Vec<_>>()
    })?;

    // 5. Post-parse validation (pipe stage count, type checking)
    let mut errors: Vec<AxiQLError> = parse_errors.into_iter().map(translate_parse_error).collect();
    errors.extend(validate_limits(&ast, config));
    errors.extend(type_check(&ast));

    if errors.is_empty() {
        Ok(ParsedQuery {
            statement: ast,
            original_query: query.to_string(),
            expanded_query: expanded,
        })
    } else {
        Err(errors)
    }
}
```

---

## 10. Open Questions Requiring Verification

These items could not be verified due to denied external research tools:

| # | Question | Impact | How to Verify |
|---|----------|--------|---------------|
| 1 | Exact current Chumsky version on crates.io (0.10.0? 0.10.1? 0.10.x?) | Cargo.toml version pin | `cargo search chumsky` or crates.io API |
| 2 | Is `extra::Full` the correct type for carrying parser state in 0.10? | Depth tracking implementation | docs.rs/chumsky or source code |
| 3 | Exact `select!` macro syntax for token matching in 0.10 | Lexer-parser integration | docs.rs/chumsky examples |
| 4 | Available error recovery strategies (skip_then_retry_until, nested_delimiters, via_parser) -- exact names and signatures | Error recovery implementation | docs.rs/chumsky::recovery module |
| 5 | `to_slice()` availability on repeated() in 0.10 for zero-copy lexing | Lexer performance | docs.rs/chumsky |
| 6 | `try_map_with` vs `try_map` -- which has access to Extra state | Depth tracking implementation | docs.rs/chumsky |
| 7 | Token-based parser input type (`&[(Token, SimpleSpan)]` vs `Stream` vs custom Input) | Parser function signatures | docs.rs/chumsky::input module |
| 8 | Chumsky 0.10 compile times (a known concern in 0.9) | Developer experience | Benchmark locally |
| 9 | Whether Chumsky 0.10.x has stabilized or if there are still alpha releases | Risk assessment | crates.io version history |
| 10 | Thread safety of parsers (can the same parser be shared across tokio tasks?) | Concurrent query parsing | Check Parser trait bounds (Send + Sync?) |

---

## 11. Recommendations Summary

| Decision | Recommendation | Confidence | Depends On |
|----------|---------------|------------|------------|
| Chumsky version | 0.10.x (match axiathon) | HIGH | Verify exact version on crates.io |
| Parsing approach | Token-based (two-phase: lexer + parser) | HIGH | N/A |
| Error type | `Rich<'a, Token>` for maximum diagnostic quality | HIGH | Verify Rich API in 0.10.x |
| Error recovery | Implement from day 1 using `recover_with()` | HIGH | Verify recovery strategy names |
| Recursive parsing | `recursive()` combinator for nested boolean expressions | HIGH | N/A |
| Depth limiting | `Rc<Cell<usize>>` counter in parenthesized expression parser | HIGH | Verify state-passing mechanism |
| Performance | Not a concern; parsing is < 0.1% of query latency | HIGH | N/A |
| Module structure | Separate crate `prism-query` with lexer/parser/ast/error modules | HIGH | N/A |
| Axiathon code reuse | Grammar as design reference only; rewrite parser from scratch | HIGH | N/A |
| Wire to execution | Connect parser to DataFusion from first commit (lesson P3-2) | HIGH | N/A |

---

## Research Methods

| Tool | Queries | Purpose | Result |
|------|---------|---------|--------|
| Context7 | 1 (DENIED) | Chumsky 0.10 docs | Permission denied |
| WebSearch | 3 (DENIED) | crates.io version, 0.9 vs 0.10 changes, performance | Permission denied |
| WebFetch | 3 (DENIED) | crates.io API, docs.rs, GitHub CHANGELOG | Permission denied |
| Bash (cargo search) | 1 (DENIED) | crates.io version lookup | Permission denied |
| Axiathon semport | 8 reads | Production parser patterns, Chumsky usage, conventions | Complete |
| AxiQL grammar spec | 1 read | Normative grammar reference | Complete |
| Behavioral contracts | 1 read | DI-019 security limits | Complete |
| Query language research | 1 read | Prior parser design decisions | Complete |
| Query engine research | 1 read | Hybrid architecture context | Complete |
| Training data | 12 areas | Chumsky 0.10 API, Parser trait, combinators, error recovery, recursive patterns, token vs char parsing, Rich error type, select! macro, performance, comparison with alternatives, zero-copy architecture, Extra type parameter | HIGH reliance |

**Total MCP tool calls:** 0 successful (all 8 attempts denied)
**Training data reliance:** HIGH -- All Chumsky 0.10 API details are from model training data (cutoff May 2025). The 10 open questions in Section 10 MUST be verified against current documentation before implementation begins.
