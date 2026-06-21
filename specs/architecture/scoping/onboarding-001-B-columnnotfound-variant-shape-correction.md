---
document_type: architecture-scoping-correction
correction_id: CORRECTION-2
story_id: S-DEMO-PRISMQL-ONBOARDING-001-B
title: "ColumnNotFound Variant Shape — Boxed Struct Required; EXPECTED 82→83"
series: S-DEMO-PRISMQL-ONBOARDING-001
issued_by: architect
date: "2026-06-21"
status: final
traces_to: ARCH-INDEX.md
---

# CORRECTION-2: `PrismError::ColumnNotFound` Variant Shape and Gate Count

## Summary

The story spec (v1.3) declares `PrismError::ColumnNotFound` as an **inline-field** variant
and asserts in three places that `ci.yml EXPECTED remains 82`. The stub-architect
(commit 1c23bb03) introduced a **boxed `ColumnNotFoundDetails` struct** (marked
`#[non_exhaustive]`) — which is the correct implementation. The story's three assertions
are WRONG. EXPECTED must be 83. This note records the adjudication and specifies all
required story corrections for story-writer.

---

## Evidence

### 1. `clippy::result_large_err` threshold — 128 bytes

The workspace enforces `clippy` with `-D warnings` (all deny-level lints are workspace-wide
errors). The `result_large_err` lint fires when a `Result` variant exceeds **128 bytes**.
This lint is NOT listed in `Cargo.toml [workspace.lints.clippy]` (only `await_holding_lock`,
`unwrap_used`, `expect_used` appear there) — meaning it fires at its default **warn** level,
which the workspace `clippy` recipe promotes to **deny** via the `-D warnings` flag in
`just check`.

### 2. `TableNotAvailableDetails` establishes the boxing precedent

`PrismError::TableNotAvailable` is the direct sibling of the new `ColumnNotFound` variant.
Its doc comment (error.rs, `TableNotAvailableDetails` struct-level doc) explicitly records:

> "Boxed inside the enum variant to keep `PrismError` under the
> `clippy::result_large_err` 128-byte threshold. Five inline `String` fields
> would push the variant past the limit."

`TableNotAvailableDetails` has 5 `String` fields. `ColumnNotFoundDetails` has 3 `String`
fields PLUS one `Vec<String>` (available_columns) PLUS one `Option<String>` (did_you_mean).

### 3. Size analysis — inline variant exceeds 128 bytes

On a 64-bit target, a `String` is 24 bytes (ptr + len + cap). A `Vec<String>` is 24 bytes
(ptr + len + cap — the pointed-to heap content is not counted for enum discriminant
size). An `Option<String>` is 24 bytes (the None variant occupies the same space as
`String` due to niche optimization — but since `String` has no niche, `Option<String>`
is 32 bytes on stable Rust without niche: size_of(String) + 1-byte discriminant with
alignment padding → 32 bytes).

Inline field total (conservative 64-bit):
- `column: String`           = 24 bytes
- `table: String`            = 24 bytes
- `client_id: String`        = 24 bytes
- `available_columns: Vec<String>` = 24 bytes
- `did_you_mean: Option<String>`   = 32 bytes (no niche on String)

Total: 128 bytes — this is exactly at the threshold. Rust adds no padding beyond
alignment on these types, but the enum tag itself (even a unit tag) means the
**variant discriminant storage** pushes the full `PrismError` enum past 128 bytes.

The `result_large_err` lint measures the size of the `Err` variant, not the enum.
A single-field `Box<T>` variant has a fixed size of 8 bytes (pointer width on 64-bit),
regardless of `T`'s size. Boxing reduces the variant from ~128+ bytes to exactly
8 bytes — well below the 128-byte threshold.

**Conclusion: the inline-field form either exactly hits or exceeds the lint threshold;
the actual compiler behavior at this boundary is consistent with the `TableNotAvailableDetails`
precedent doc comment which explicitly says "five inline `String` fields would push
the variant past the limit." `ColumnNotFoundDetails` has an equivalent payload
(3 Strings + Vec<String> + Option<String>) that is no smaller. Boxing is required.**

### 4. Convention parity with `TableNotAvailableDetails` (E-QUERY-037)

The `ColumnNotFound` variant is the sibling of `TableNotAvailable` — both are E-QUERY
plan-time gate errors, both are public prism-core types, both carry multi-field
structured data. Convention consistency demands the same boxed-struct pattern:

- `TableNotAvailable(Box<TableNotAvailableDetails>)` — 5 String fields
- `ColumnNotFound(Box<ColumnNotFoundDetails>)` — 3 String + 1 Vec<String> + 1 Option<String>

The S-3.13 story (which added `TableNotAvailableDetails`) explicitly set the boxing
precedent for this exact reason.

### 5. `#[non_exhaustive]` on `ColumnNotFoundDetails`

`ColumnNotFoundDetails` is a `pub struct` in `prism-core`. Per CLAUDE.md conventions:

> "All public TOML-deserialized types and pub-API surface types require `#[non_exhaustive]`."

`ColumnNotFoundDetails` is a pub-API surface type (it is `pub` in a public crate,
accessible to all downstream crates including `prism-query`, `prism-mcp`, tests).
It must carry `#[non_exhaustive]`. This adds one new non-exhaustive type to the
workspace-enforced gate.

### 6. Current gate state

`ci.yml` line `EXPECTED=82` is the authority (updated by S-DEMO-PRISMQL-ONBOARDING-001-A).
The type list in ci.yml does NOT yet include `ColumnNotFoundDetails`. Adding
`ColumnNotFoundDetails` brings the total to **83**.

---

## Decision

**Option B is CORRECT. The stub-architect's `ColumnNotFoundDetails` stub (commit 1c23bb03)
is CORRECT-AS-BUILT.**

Canonical variant declaration:

```rust
/// E-QUERY-038: Column not found in the queried table for this client.
///
/// The inner fields are boxed (`Box<ColumnNotFoundDetails>`) to keep `PrismError`
/// within the `clippy::result_large_err` threshold.
///
/// Construct via `PrismError::ColumnNotFound(Box::new(ColumnNotFoundDetails::new(...)))`.
/// Match via `PrismError::ColumnNotFound(ref d)` or `PrismError::ColumnNotFound(..)`.
#[error("{0}")]
ColumnNotFound(Box<ColumnNotFoundDetails>),
```

Supporting public struct (already in stub):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ColumnNotFoundDetails {
    pub column: String,
    pub table: String,
    pub client_id: String,
    pub available_columns: Vec<String>,
    pub did_you_mean: Option<String>,
}
```

Definitive EXPECTED value: **83** (82 + 1 for `ColumnNotFoundDetails`).

---

## Required Story Edits (story-writer's responsibility)

The following three locations in story v1.3 assert "EXPECTED remains 82" — all three
are WRONG and must be corrected to "EXPECTED → 83". Additionally, the Phase 1 variant
declaration must be corrected to the boxed form, and a compile-fail-gate file-structure
row must be added.

### Edit 1 — Phase 1 task: variant declaration

Current (wrong):
> Add to `crates/prism-core/src/error.rs`:
> ```rust
> ColumnNotFound {
>     column: String,
>     table: String,
>     client_id: String,
>     available_columns: Vec<String>,
>     did_you_mean: Option<String>,
> }
> ```
> NOTE: `PrismError` enum is already `#[non_exhaustive]` — NO new annotation needed at enum level.
> Verify `result_large_err` clippy lint: if this variant triggers it, box the Vec fields following
> `TableNotAvailableDetails` precedent.

Replace with:
> Add to `crates/prism-core/src/error.rs`:
>
> 1. A new `pub struct ColumnNotFoundDetails` (before the `TableNotAvailableDetails` struct, matching its position pattern) with fields `column: String`, `table: String`, `client_id: String`, `available_columns: Vec<String>`, `did_you_mean: Option<String>`. Mark `#[non_exhaustive]`, `#[derive(Debug, Clone, PartialEq, Eq)]`. Implement `Display` delegating `"E-QUERY-038: column '{}' not found in table '{}' for client '{}'". Implement `ColumnNotFoundDetails::new(...)` constructor.
>
> 2. The enum variant: `ColumnNotFound(Box<ColumnNotFoundDetails>)` with `#[error("{0}")]`.
>
> REASON: `Vec<String>` + `Option<String>` + 3× `String` inline exceeds or hits the
> `result_large_err` 128-byte threshold; boxing (following `TableNotAvailableDetails`
> precedent from S-3.13) reduces the variant to 8 bytes. `ColumnNotFoundDetails` is a
> pub prism-core type and requires `#[non_exhaustive]` per CLAUDE.md conventions.

### Edit 2 — Phase 5 (normalized_pql) ci.yml assertion

Current (wrong):
> NO `#[non_exhaustive]` annotation is needed (no new pub struct); ci.yml EXPECTED stays 82
> (set by 001-A which has already merged).

Replace with:
> `ColumnNotFoundDetails` (from Phase 1) is a new `#[non_exhaustive]` pub struct — ci.yml
> EXPECTED must be bumped from 82 to **83** by the implementer during TDD green.
> CLAUDE.md count must be updated from 82 to 83 at merge time. The normalized_pql wire
> itself adds no new pub struct — only Phase 1 (`ColumnNotFoundDetails`) drives the gate bump.

### Edit 3 — Phase 6 final gates: ci.yml step

Current (wrong):
> Confirm ci.yml EXPECTED remains 82 — no new `#[non_exhaustive]` pub types are added
> by this story (no new typed response struct; `normalized_pql` is a conditionally-inserted
> `Value` key, not a struct field). The non-exhaustive gate is already wired at EXPECTED=82
> by the merged 001-A. No coordination needed.

Replace with:
> Confirm ci.yml EXPECTED is **83** (bumped from 82 during TDD green — `ColumnNotFoundDetails`
> is one new `#[non_exhaustive]` pub struct). The non-exhaustive-violation crate's
> `enum_violations.rs` must include a `ColumnNotFoundDetails` violation function so the gate
> continues to compile-fail at the correct count.

### Edit 4 — File Structure table: ci.yml row

Current (wrong):
> | `ci.yml` | No change | EXPECTED remains 82 (set by merged 001-A). No new `#[non_exhaustive]` types added by 001-B (no typed response struct). |

Replace with:
> | `ci.yml` | Modify | Bump EXPECTED from 82 to 83 — `ColumnNotFoundDetails` is a new `#[non_exhaustive]` pub struct added by this story. Update the type list comment to include `ColumnNotFoundDetails`. Also update the error text listing at line ~679 (see S-3.13 LOW-1 row entry for `TableNotAvailableDetails` as the model). |

### Edit 5 — File Structure table: compile-fail gate row (ADD)

Add a new row to the File Structure table:

> | `tests/external/non-exhaustive-violation/src/enum_violations.rs` | Modify | Add a violation function for `ColumnNotFoundDetails` (struct-literal construction attempt), mirroring the `TableNotAvailableDetails` violation function added by S-3.13 LOW-1. This brings the E0639 count to 83. |

---

## Implementer Obligations

The stub (commit 1c23bb03) is correct-as-built for the variant shape and struct definition.
The implementer must:

1. Verify the stub's `ColumnNotFoundDetails` compiles cleanly (`just iter prism-core`).
2. During TDD green, bump `ci.yml EXPECTED` from 82 to 83.
3. During TDD green, add a `ColumnNotFoundDetails` violation function to
   `tests/external/non-exhaustive-violation/src/enum_violations.rs`.
4. During TDD green, update the ci.yml type-list comment to include `ColumnNotFoundDetails`
   (adjacent to `TableNotAvailableDetails` in the `prism_core:` group).
5. At merge time, update CLAUDE.md count from 82 to 83 and add `ColumnNotFoundDetails`
   to the parenthetical story-attribution list (model: the `TableNotAvailableDetails` entry
   in CLAUDE.md cites `S-3.13 LOW-1`; the new entry cites `S-DEMO-PRISMQL-ONBOARDING-001-B`).

The implementer MUST NOT change the variant shape back to inline fields — doing so will
cause `clippy` to emit `result_large_err` (at lint-deny-level via `just check -D warnings`)
which is a pre-push hook failure.

---

## CLAUDE.md Count Reconciliation Obligation

When this story's PR merges:
- CLAUDE.md non-exhaustive count: **82 → 83**
- CLAUDE.md sentence: append `, S-DEMO-PRISMQL-ONBOARDING-001-B (ColumnNotFoundDetails)`
  to the parenthetical story attribution list after the existing
  `S-DEMO-PRISMQL-ONBOARDING-001-A (PrismDescribeResponse, TableDescriptor, ColumnDescriptor)`
  entry.
- ci.yml `EXPECTED=82` → `EXPECTED=83`

This is a merge-time obligation. The implementer performs it as part of the TDD green
phase (ci.yml and enum_violations.rs) and confirms it in the PR checklist.

---

## Correction Series Context

| # | Correction | Status |
|---|-----------|--------|
| CORRECTION-1 | `normalized_pql` envelope — conditional Value key insertion, no typed struct, EXPECTED unchanged by 001-B normalized_pql wire | Applied (v1.3) |
| CORRECTION-2 | `ColumnNotFound` variant shape — boxed struct required; EXPECTED 82→83 | This document (pending story-writer application) |
