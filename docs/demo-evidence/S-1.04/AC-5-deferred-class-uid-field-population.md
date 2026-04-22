# AC-5 Deferred: class_uid Field Population (S-1.05 Scope)

**Story:** S-1.04  
**Deferred to:** S-1.05  
**Test:** `test_BC_2_02_002_normalized_message_has_class_uid_2004`  
**Status:** `#[ignore]` — recorded in test suite at commit 76837cf

## Summary

The acceptance criterion for verifying that a normalized `DynamicMessage` has
`class_uid = 2004` set as a field value (not just the correct descriptor) is
explicitly deferred to S-1.05. The test exists in the codebase and is marked
`#[ignore]` with the annotation:

```
#[ignore = "S-1.05 scope: class_uid field population requires sensor-specific mappers (S-1.05 AC-3)"]
```

## Reason for Deferral

S-1.04 establishes the normalization infrastructure: `DescriptorPool`,
`OcsfNormalizer`, `EventClassSelector`, and `OcsfEnumMap`. Field population
for each sensor — including writing `class_uid = 2004` into the `DynamicMessage`
payload — requires sensor-specific mappers that are the responsibility of S-1.05.

The `normalize()` call returns a structurally valid `DynamicMessage` wrapping
the correct OCSF `DetectionFinding` descriptor (verified by AC-3). Setting the
`class_uid` field value inside that message is S-1.05 work.

## Test Evidence

The ignored test is present at:

```
crates/prism-ocsf/src/tests/bc_2_02_002_normalizer.rs
fn test_BC_2_02_002_normalized_message_has_class_uid_2004
```

When S-1.05 lands and field mappers are implemented, this `#[ignore]` annotation
must be removed and the test must pass as part of S-1.05 acceptance.
