---
document_type: verification-property
level: L4
version: "0.1"
status: draft
producer: architect
timestamp: 2026-05-02T20:50:00Z
phase: 4-W4-Phase3-ADR
inputs:
  - .factory/specs/architecture/decisions/ADR-019-siem-output-formats.md
  - .factory/STATE.md
input-hash: "d95ba31"
traces_to: .factory/specs/architecture/decisions/ADR-019-siem-output-formats.md
source_bc: null
source_adr: ADR-019
module: prism-siem-formats
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: wave-4-phase-3-adr
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-144: SIEM Formats Encoder Correctness (CEF v0 + LEEF 2.0)

> **NOTE:** This VP traces to an ADR, not a BC. `source_bc: null` and
> `source_adr: ADR-019` are used because the primary constraint originates
> in the architecture decision record. A BC anchor will be added when
> ADR-019 transitions to Accepted and produces a concrete BC in Phase 2.

> **[STUB — full VP authoring deferred to Phase 2 of Wave 4]**

## Property Statement

The `prism-siem-formats` crate encoders must produce byte-perfect output that
satisfies the 13 proptest invariants enumerated in ADR-019 §7 for both the
CEF v0 encoder (`cef::v0::Encoder`) and the LEEF 2.0 encoder
(`leef::v2::Encoder`). These invariants cover: (1) header field ordering and
pipe-delimiter placement per ArcSight CEF Standard; (2) extension key=value
pair escaping (backslash, pipe, equals, newline); (3) severity clamping to
[0, 10] for CEF and [0, 9] for LEEF; (4) LEEF tab-delimiter between extension
pairs; (5) UTF-8 validity of all output; (6) round-trip fidelity for all
printable ASCII field values; and (7) absence of header injection across all
field positions. A proptest corpus spanning randomized OCSF event inputs must
demonstrate that all 13 invariants hold for both encoders across the full
range of valid and boundary input values.

## Source Contract

> **ADR-sourced stub — BC not yet assigned.**

- **ADR:** ADR-019 — SIEM Output Formats, §7 Proptest Invariants
- **Decision Reference:** D-212 — build `prism-siem-formats` in-house; no maintained Rust crates (rust-cef abandoned 2021)
- **Postcondition/Invariant:** All 13 proptest invariants hold for `cef::v0::Encoder` and `leef::v2::Encoder` across randomized OCSF event inputs.
- **BC:** To be assigned when ADR-019 is Accepted and BC authoring completes in Wave 4 Phase 2.
- **Module:** prism-siem-formats
- **Category:** Correctness / Encoding Fidelity

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest 1.x | Yes — randomized OCSF event inputs across full field value range | 13 CEF/LEEF encoding invariants for both encoders |

**Feasibility:** The encoder output space is deterministic and bounded; proptest can generate randomized OCSF event inputs (including boundary values for severity, empty fields, and special characters) and verify all 13 invariants hold without symbolic execution.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_siem_formats::cef::v0::Encoder, prism_siem_formats::leef::v2::Encoder
//
// proptest! {
//     #[test]
//     fn cef_encoder_correctness(event in arb_ocsf_event()) {
//         let output = cef::v0::Encoder::encode(&event)?;
//         // Assert all 13 CEF invariants
//         prop_assert!(output.is_utf8_valid());
//         prop_assert!(cef_header_pipe_delimiters_correct(&output));
//         prop_assert!(cef_severity_clamped_0_10(&output, event.severity));
//         // ... (remaining invariants per ADR-019 §7)
//     }
//     #[test]
//     fn leef_encoder_correctness(event in arb_ocsf_event()) {
//         let output = leef::v2::Encoder::encode(&event)?;
//         prop_assert!(output.is_utf8_valid());
//         prop_assert!(leef_tab_delimiters_correct(&output));
//         prop_assert!(leef_severity_clamped_0_9(&output, event.severity));
//         // ... (remaining invariants per ADR-019 §7)
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | OCSF event field values are finite; proptest can enumerate all meaningful severity + escaping + delimiter boundary cases |
| Proof complexity | Low–Medium | 13 invariants per encoder; each assertion is a string structural check |
| Tool support | Full | proptest 1.x handles randomized string/integer generation for all field types |
| Estimated proof time | <60 seconds | Two encoders × 13 invariants; no symbolic execution required |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-02 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | W4-Phase3-ADR-commit | 2026-05-02 | architect | Initial stub. Traces to ADR-019 §7 (13 proptest invariants for CEF v0 + LEEF 2.0 encoder correctness). source_bc null pending ADR acceptance + BC authoring. Full harness deferred to Wave 4 Phase 2. |
