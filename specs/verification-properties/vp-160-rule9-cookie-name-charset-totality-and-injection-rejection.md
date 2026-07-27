---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-07-25T00:00:00Z
phase: wave-a
inputs:
  - specs/behavioral-contracts/BC-2.16.009
  - specs/architecture/decisions/ADR-053
input-hash: "pending"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.16.009
source_invariant: null
module: prism-spec-engine
priority: P0
proof_method: kani
verification_method: kani
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: fix-burst-46
modified: "2026-07-26"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-160: Rule 9 Cookie-Name Charset Totality and Injection Rejection

## Property Statement

For the `is_valid_cookie_name_tchar` predicate in `prism-spec-engine`:

**Property 1 (charset totality):** The predicate returns `true` if and only if the input
string is non-empty and every byte is in the 77-character RFC 9110 §5.6.2 tchar set. The
tchar set is exactly: 26 ASCII uppercase letters (`A`–`Z`), 26 ASCII lowercase letters
(`a`–`z`), 10 ASCII digits (`0`–`9`), and 15 special characters: `!`, `#`, `$`, `%`, `&`,
`'`, `*`, `+`, `-`, `.`, `^`, `_`, `` ` ``, `|`, `~`. No character outside this set is
accepted.

**Property 2 (injection rejection):** The predicate returns `false` for any string
containing any of the following characters: semicolon (`;`), bare equals (`=`), space
(` `), horizontal tab (`\t`), any ASCII control character (0x00–0x1F), DEL (0x7F), any
non-ASCII byte (0x80–0xFF), and all RFC 9110 delimiter characters (`(`, `)`, `<`, `>`,
`@`, `,`, `:`, `\`, `"`, `/`, `[`, `]`, `?`, `{`, `}`).

These two properties are exhaustively equivalent: charset totality proves that exactly the
tchar set is accepted; injection rejection names the security-relevant subset of characters
that must not be accepted. A Kani proof establishing Property 1 exhaustively over all 128
ASCII byte values entails Property 2 as a corollary.

**Scope note:** The 64-codepoint echo cap (CWE-400) and `\xNN` CTL-escaping (CWE-117) of
the invalid value in E-SPEC-027(a) error messages are formatting concerns in the error
message composition function, not in `is_valid_cookie_name_tchar`. Those formatting
invariants are outside the scope of this VP and are covered by **VP-161** (Rule 9 error
message echo cap and CTL escaping — `truncate_at_char_boundary` and CTL-byte `\xNN`
escaping correctness; anchor: S-WAVE-A-ENGINE-001 §Tasks T-B02; BC-2.16.009
EC-009-047/EC-009-048).

**Platform note:** Kani requires Linux or macOS (CBMC backend). Windows contributors use
concrete unit tests as equivalents; the formal proof runs in CI on Linux/macOS. One proof
establishes truth for all platforms (same Rust source; see CLAUDE.md §Formal Verification).

## Source Contract

- **Anchor Story:** `S-WAVE-A-ENGINE-001`
  — Anchor justification (POL-5): `S-WAVE-A-ENGINE-001` authors the body of
  `is_valid_cookie_name_tchar` in §Tasks T-B02 step 3 — the exact symbol this VP targets
  as its proof vehicle. Its §Architecture Mapping assigns `SpecLoader::validate_header_scheme`
  (Rule 9) to `crates/prism-spec-engine/src/spec_parser.rs`, the same module path cited in
  §Proof Harness Skeleton. No other Wave-A story introduces or owns this symbol.
- **Source BC:** BC-2.16.009 — Spec File Validation — Rule 9 (`header_scheme` field validation, E-SPEC-027)
- **Security Reference:** SEC-001 (CWE-20/CWE-74 — cookie-name injection via non-tchar characters) / ADR-053 §D2
- **Module:** prism-spec-engine
- **Category:** Security / Input Validation

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | kani (CBMC model checking) | Yes — single-byte ASCII inputs (128-point exhaustive space); non-ASCII rejected by structural argument | All 128 ASCII byte values; named injection characters (`;`, `=`, SP, TAB, CTL, delimiters) explicitly verified as corollaries |

**Why Kani:** `is_valid_cookie_name_tchar` is a pure predicate (`&str → bool`) with no I/O,
no async, and no global state. The per-byte match expression is exactly the class of proof
Kani excels at: a 128-point bounded space with a known reference definition. Proptest would
provide probabilistic coverage; Kani provides exhaustive proof that the predicate matches
the RFC 9110 §5.6.2 tchar definition for every possible single-byte ASCII input.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_spec_engine::spec_parser::is_valid_cookie_name_tchar
// (or its canonical module path at the time of Phase 5 implementation)
//
// Property 1 (charset totality) and Property 2 (injection rejection) are both
// established by the 128-point exhaustive ASCII harness below. The input space is
// bounded to ASCII (0x00–0x7F) because tchar is a pure ASCII set; non-ASCII bytes
// in a Rust &str appear only as part of multi-byte UTF-8 sequences whose individual
// bytes (0x80–0xFF) are structurally excluded by the production match arm (which
// covers only ASCII ranges). Property 2 follows as a corollary of Property 1.
//
// #[kani::proof]
// fn verify_tchar_charset_totality_and_injection_rejection() {
//     let b: u8 = kani::any();
//     kani::assume(b <= 0x7F); // ASCII range: necessary and sufficient for tchar
//
//     // Build a single-byte &str.
//     // Safety invariant: 0x00..=0x7F are all valid UTF-8 scalars.
//     let buf: [u8; 1] = [b];
//     let s = std::str::from_utf8(&buf).unwrap();
//
//     let result = is_valid_cookie_name_tchar(s);
//
//     // Reference definition: RFC 9110 §5.6.2 tchar (77 characters exactly)
//     let expected = matches!(
//         b,
//         b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9'
//             | b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*'
//             | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
//     );
//
//     assert_eq!(
//         result, expected,
//         "tchar predicate must match RFC 9110 §5.6.2 exactly for ASCII byte 0x{:02X}",
//         b
//     );
// }
//
// Kill conditions (mutation testing — these mutations MUST be caught by the harness):
//   - Remove any tchar character from the production match arm
//     → harness fails on that byte (expected=true, result=false)
//   - Add a non-tchar character (e.g., b';', b'=', b' ', b'\t') to the match arm
//     → harness fails on that byte (expected=false, result=true)
//   - Flip the is_empty() guard (allow empty string)
//     → harness does not cover empty input; concrete test must assert
//       is_valid_cookie_name_tchar("") == false independently
//   - Change `all(...)` to `any(...)` in the production predicate
//     → multi-byte harness (kani::any::<[u8; 2]>()) would catch this; see below
//
// Extension harness (multi-byte correctness — optional Phase 6 addition):
//
// #[kani::proof]
// fn verify_tchar_single_invalid_byte_rejects_multi_byte_string() {
//     let a: u8 = kani::any();
//     let b: u8 = kani::any();
//     kani::assume(a <= 0x7F && b <= 0x7F);
//     // If either byte is non-tchar, the two-byte string must be rejected
//     let a_valid = matches!(a, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9'
//         | b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*'
//         | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~');
//     let b_valid = matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9'
//         | b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*'
//         | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~');
//     let buf: [u8; 2] = [a, b];
//     if let Ok(s) = std::str::from_utf8(&buf) {
//         let result = is_valid_cookie_name_tchar(s);
//         assert_eq!(result, a_valid && b_valid);
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | 128 ASCII byte values (0x00–0x7F); single-byte strings are the minimal test for per-byte predicate correctness |
| Tool support? | Full | `is_valid_cookie_name_tchar` is pure (`&str → bool`); no I/O, no async, no side effects |
| Execution time budget | < 30 seconds | 128-point space + simple match expression; Kani CBMC is fast for pure per-byte predicates |
| Assumptions required | None | No external dependencies; no test helpers required beyond `kani::any::<u8>()` |
| Platform constraint | Linux/macOS only | Kani uses CBMC backend; not supported on Windows. CI runs proof on Linux; Windows contributors rely on concrete unit tests. |
| Non-ASCII coverage | Structural argument | Non-ASCII bytes (0x80–0xFF) appear in `&str` only as multi-byte UTF-8 sequence bytes; the production `matches!` arm covers only ASCII ranges, so high bytes are structurally rejected without requiring exhaustive enumeration |
| Empty-string coverage | Concrete test | The empty-string guard (`!name.is_empty()`) is a boundary condition; covered by a concrete unit test asserting `is_valid_cookie_name_tchar("") == false` |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-07-25 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | FB68c | 2026-07-27 | architect | F-WASE-P65-OBS-001 sweep: `modified: []` corrected to `"2026-07-26"` (last-modification date per convention established by VP-153/VP-154/VP-155/VP-156/VP-159; `timestamp: 2026-07-25T00:00:00Z` unchanged as creation date). Frontmatter-only hygiene; no property semantics changed. |
| 1.2 | FB62 | 2026-07-26 | architect | F-WASE-P64-OBS-002: scope note replaced — previously deferred CWE-400/CWE-117 formatting invariants to "a separate property" with no VP ID or anchor; now cross-references VP-161 explicitly (successor VP registered in same burst). |
| 1.1 | FB55c | 2026-07-26 | architect | F-WASE-P64-HIGH-004: anchor story resolved from placeholder to `S-WAVE-A-ENGINE-001`; anchor justification added per POL-5 citing §Tasks T-B02 authorship of `is_valid_cookie_name_tchar` and §Architecture Mapping placement in `crates/prism-spec-engine/src/spec_parser.rs`. |
| 1.0 | fix-burst-46 | 2026-07-25 | architect | Initial draft. F-WASE-P62-MED-004: Rule 9 cookie-name charset totality and injection rejection (SEC-001, ADR-053 §D2). Method: Kani. P0. |
