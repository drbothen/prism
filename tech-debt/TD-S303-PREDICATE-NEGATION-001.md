# TD-S303-PREDICATE-NEGATION-001: Negation flag silently dropped for Wildcard/In/IsNull/Between predicates

**Story:** S-3.03
**Status:** open
**Severity:** tech_debt

## Description

`predicates_from_ast` in `crates/prism-query/src/explain.rs` maps several predicate
variants to a bare `Expr::Field(field)` node, discarding the `negated` field entirely:

```rust
Predicate::StringOp { field, .. } => vec![Expr::Field(field.clone())],
Predicate::Regex    { field, .. } => vec![Expr::Field(field.clone())],
Predicate::In       { field, .. } => vec![Expr::Field(field.clone())],
Predicate::Between  { field, .. } => vec![Expr::Field(field.clone())],
Predicate::IsNull   { field, .. } => vec![Expr::Field(field.clone())],
Predicate::Wildcard { field, .. } => vec![Expr::Field(field.clone())],
```

When `negated: true`, the rendered predicate string (e.g. `"hostname"`) is
indistinguishable from the non-negated form. An analyst reading the explain output
cannot tell whether the filter is `hostname LIKE 'web*'` or `NOT hostname LIKE 'web*'`.

This is the same root cause as I-LOCAL-NEW-1 (negated CIDR), which was fixed in
S-3.03 adversary local pass-2. The Wildcard/In/IsNull/Between variants require the
same treatment but involve more rendering complexity (sensor-native NOT syntax varies).

## Affected Predicate Variants

- `Predicate::Wildcard { negated: true }` — `NOT hostname LIKE 'web*'`
- `Predicate::In { negated: true }` — `NOT status IN ('active', 'pending')`
- `Predicate::IsNull { negated: true }` — `field IS NOT NULL`
- `Predicate::Between { negated: true }` — `NOT severity_id BETWEEN 3 AND 5`
- `Predicate::StringOp` / `Predicate::Regex` — negation field existence TBD

## Required Fix

For each affected variant, introduce either:
1. A new `CompareOp` variant (e.g. `NotWildcard`, `NotIn`) following the
   `CompareOp::NotCidr` pattern from I-LOCAL-NEW-1 fix.
2. Or a wrapping `Expr::Not(Box<Expr>)` variant added to the AST so
   `predicate_as_string` can render any predicate with a NOT prefix.

Option 2 is cleaner as a general solution. Option 1 requires N new CompareOp
variants and N rendering arms.

## Deferred In

S-3.03 adversarial local pass-2. I-LOCAL-NEW-1 (Cidr negation) was fixed; these
variants are deferred as lower-frequency edge cases with the same root cause.
Recommend fixing in the same story wave as sensor-native NOT translation (S-3.X).

## References

- `crates/prism-query/src/explain.rs` — `predicate_to_exprs()`
- `crates/prism-query/src/ast.rs` — `Predicate`, `CompareOp`, `CompareOp::NotCidr`
- I-LOCAL-NEW-1 (fixed): negated `Predicate::Cidr` rendered via `CompareOp::NotCidr`
- S-3.03 adversary local pass-2 — parallel finding to I-LOCAL-NEW-1
