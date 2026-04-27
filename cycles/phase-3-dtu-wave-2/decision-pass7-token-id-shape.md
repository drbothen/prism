---
document_type: decision-note
cycle: phase-3-dtu-wave-2
pass: pass-7
blocker: W2-FIX-K
date: 2026-04-26
author: product-owner
---

# Decision: Token ID Shape in Issuance Audit Entries (Pass 7 Contradiction)

## Contradiction Summary

Pass 7 surfaced a spec-vs-spec contradiction:

- **BC-2.05.010 canonical TV table** (authoritative): "Token issued" row — "Token ID in Entry? = No"
- **S-2.05 AC-4** (story): "`emit_token_generated()` audit entry contains `token_id`"
- **Implementation** (`token_events.rs`): `TokenLifecycleDetail.token_id` is always serialized
  into `parameters` JSON, including for `Generated` events — so `token_id` IS persisted at issuance

## Option Chosen: Option A — BC is canonical, AC must change

**Rationale:** VSDD policy is unambiguous — BC trumps AC. The BC's privacy/minimum-disclosure
rationale ("token IDs are intentionally excluded from issuance audit entries to prevent
correlation by log readers") is a deliberate security property, not an oversight. AC-4 was
written imprecisely during story decomposition; it should have said "contains `action_summary`
and `expiry_time` (but NOT `token_id`)" to match the BC postconditions exactly.

No architect sign-off required. This is a spec alignment fix, not an architecture change.

---

## Implementer Brief for W2-FIX-K

### Functions to modify (`crates/prism-audit/src/token_events.rs`)

- **`emit_token_generated`**: Before constructing `parameters`, strip `token_id` from the
  serialized `TokenLifecycleDetail`. The simplest approach: serialize the detail, then remove
  the `"token_id"` key from the resulting `serde_json::Value` before embedding in `parameters`.
  Alternatively, use a separate `IssuanceDetail` struct (without `token_id`) for `Generated`
  events only. Either approach is acceptable — the constraint is that the persisted
  `AuditEntry.parameters` JSON must NOT contain a `"token_id"` key for issuance events.

- **`emit_token_expired`**: Same issue — `TokenLifecycleDetail.token_id` is currently
  persisted in `parameters`. Per BC-2.05.010 canonical TV: "Token expired" row — "Token ID
  in Entry? = No". Strip `token_id` from the serialized parameters for this event type too.

- **`emit_token_consumed`**: Token consumption (success) row says "Yes (in sub-fields)" — 
  `token_id` IS intentionally present. No change needed for this function.

### New persisted shape

For `Generated` and `Expired` events, the `token_lifecycle_detail` JSON in `parameters` must
contain: `event_type`, `action_summary`, `expiry_time` — and must NOT contain `token_id`.

For `Consumed`, `NotFound`, `HashMismatch`, `AlreadyConsumed` events: `token_id` may remain
(these are post-issuance events where the token has already been presented to the system).

### What `test_BC_2_05_010_token_id_excluded_from_result_summary_level_detail` should assert

Current test is a tautology — it only checks struct field layout, never touches the backend.
Replace it with a real integration test:

```rust
// 1. Call emit_token_generated() with a real (in-memory) backend
// 2. Read the persisted AuditEntry from audit_buffer CF
// 3. Parse entry.payload["parameters"] as serde_json::Value
// 4. Navigate to ["token_lifecycle_detail"] object
// 5. Assert: the object does NOT have a "token_id" key
// 6. Assert: "action_summary" IS present and matches input
// 7. Assert: "expiry_time" IS present
```

The test name is correct; the body needs to exercise the real emitter path.

### Sibling emitters with same issue

- **`emit_token_expired`** — confirmed affected (see above). Same `TokenLifecycleDetail`
  struct serialized into parameters; BC TV table says "Token expired → Token ID in Entry? = No".
  Must strip `token_id` from persisted parameters.

- `credential_events.rs`: No `token_id` field; no analog issue. The credential privacy
  invariant (no `value` field) is a different concern and is already enforced.

- `flag_events.rs`: No `token_id` field. Not affected.

---

## Process-Gap Follow-Up (Pass 7 HIGH-001 + HIGH-003)

Pass 7 marked two findings as `[process-gap]`:
- HIGH-001: Test is a tautology (does not exercise the real emitter path)
- HIGH-003: BC canonical TV "Token ID in Entry? = No" not matched by implementation

**Suggested TD entry for `validate-consistency` skill:**

> Add a `tautology-detector` check and a `bc-canonical-tv-consistency` check to the
> `validate-consistency` skill. The tautology check should flag any test function whose
> body never calls an emitter function AND whose name references a BC postcondition
> (pattern: `test_BC_*` functions that construct a struct directly without calling the
> corresponding `emit_*` function). The BC-TV consistency check should parse the
> "Canonical Test Vectors" table from each BC file and cross-reference field-level
> assertions (e.g., "Token ID in Entry? = No") against the serialized struct definitions
> used by the corresponding emitter — specifically, flag cases where a struct field is
> always serialized but the TV table marks it as excluded. Both checks run as part of
> the existing consistency validation pass and report as MEDIUM severity findings.

State-manager: please file this as a TD item targeting `validate-consistency` skill
enhancement, referencing Pass 7 findings HIGH-001 and HIGH-003.
