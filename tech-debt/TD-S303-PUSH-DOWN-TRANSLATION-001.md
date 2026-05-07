# TD-S303-PUSH-DOWN-TRANSLATION-001: Sensor-native push-down filter translation deferred

**Story:** S-3.03
**Status:** open
**Severity:** tech_debt

## Description

The `explain_query` tool's `api_filters_pushed` field (per-sensor push-down
predicates, one entry per `ExplainSource`) currently emits PrismQL-native generic
predicate strings (e.g. `"severity = 'critical'"`) rather than sensor-native syntax
(FQL for CrowdStrike, KQL for Armis, etc.).

The original BC-2.11.010 Description stated "per-sensor push-down filters in
sensor-native syntax". This was an aspirational claim; the S-3.03 implementation
defers the translation layer because:

1. The push-down rewrite rules (PrismQL predicate → FQL/KQL/etc.) are non-trivial
   and sensor-specific.
2. The BC postcondition (showing which predicates would be pushed) is satisfied by
   generic strings — the analyst can see filter intent; the exact syntax differs.
3. The sensor adapter layer (S-3.X scope) is the correct home for translation,
   since each adapter owns the sensor vocabulary.

The `predicate_as_string` helper in `crates/prism-query/src/explain.rs` has a
`// TODO (CR-002): translate to sensor-native syntax` marker for this work.

## Current Behavior

`ExplainSource.api_filters_pushed` is a `Vec<String>` of PrismQL-native predicate
strings rendered by `predicate_as_string()`. For example:

- `"severity = 'critical'"` (not FQL `severity:'critical'`)
- `"NOT hostname CIDR '10.0.0.0/8'"` (not sensor-native NOT-CIDR form)

## Required Fix

For each supported sensor adapter, implement a translation function:

```
fn to_sensor_native(pred: &Predicate, sensor_type: SensorType) -> String
```

This function should map PrismQL predicate AST nodes to the target sensor's query
syntax. Initial targets:

- CrowdStrike: FQL (`field:'value'`, range operators, `+` for AND)
- Armis: AQL (`field="value"`, `IN(...)`, `BETWEEN ... TO ...`)
- Claroty: native filter syntax (TBD per adapter)

The fallback for unsupported sensors should remain the current generic string form.

## Deferred In

S-3.03 adversary local pass-3 (I-LOCAL-PASS3-3). The BC postcondition is met
by generic predicate strings; sensor-native rendering is a future enhancement.
Deferred because sensor adapter vocabulary is outside S-3.03 scope.

## References

- BC-2.11.010 Invariants (DI-PUSH-001 deferral note, v1.5)
- `crates/prism-query/src/explain.rs` — `predicate_as_string()`, `ExplainSource`
- BC-2.11.007: Sensor Filter Push-Down (sensor adapter push-down contract)
- TD-S303-PREDICATE-NEGATION-001: related — negated predicate rendering gaps
