---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-07-26T00:00:00Z
phase: wave-a
inputs:
  - specs/behavioral-contracts/BC-2.16.009
  - stories/S-WAVE-A-ENGINE-001-header-scheme-field-rule9-validation-auth-dispatch.md
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
introduced: FB62
modified: "2026-07-27"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-161: Rule 9 Error Message Echo Cap and CTL Escaping

## Property Statement

For the E-SPEC-027 template (a) error message construction in `prism-spec-engine`:

**Property 1 (CWE-400 echo cap):** The `{value}` substitution in the E-SPEC-027(a) error
message is always capped at 64 codepoints. For any `header_scheme` value passed to the
error construction, the resulting message's echoed portion never exceeds 64 codepoints.
The truncation is performed by `truncate_at_char_boundary(&raw_value, 64)` before
`{value}` substitution; the truncated slice contains no appended marker — it is a plain
byte-verbatim prefix of the input.

**Property 2 (CWE-117 CTL escaping):** After applying the 64-codepoint cap, the error
message construction replaces every CTL byte (0x00–0x1F, and 0x7F) in the
capped value with the four-character ASCII literal `\xNN` (literal backslash, lowercase
`x`, two uppercase hex digits). No raw CTL byte survives into the emitted error message.
For values containing no CTL bytes, the escaping step is a no-op.

These two properties target the **error message composition function** in the Rule 9
validation path — distinct from `is_valid_cookie_name_tchar` (VP-160's proof target).
VP-160 proves that the predicate correctly accepts only tchar-valid characters. VP-161
proves that when the predicate rejects a value, the resulting error message bounds and
sanitizes the echoed invalid input — preventing a CWE-400 log-flooding vector (unbounded
echo of untrusted input) and a CWE-117 log-injection vector (raw CTL bytes enabling
terminal control sequences or log forging in the analyst's terminal output).

**Security References:** CWE-400 (Uncontrolled Resource Consumption — unbounded echo
from untrusted input) and CWE-117 (Improper Output Neutralization for Logs — raw CTL
bytes in log messages). Both apply directly to the `prism` MCP server's error message
surface, which is consumed by the analyst's terminal and may be emitted to log aggregators.

**Platform note:** Kani requires Linux or macOS (CBMC backend). Windows contributors use
concrete unit tests as equivalents (AC-024/RG-028 for CWE-400, AC-025/RG-029 for
CWE-117 in S-WAVE-A-ENGINE-001). One proof establishes truth for all platforms (same
Rust source; see CLAUDE.md §Formal Verification).

## Source Contract

- **Anchor Story:** `S-WAVE-A-ENGINE-001`
  — Anchor justification (POL-5): `S-WAVE-A-ENGINE-001` §Tasks T-B02 "Template (a)
  `{value}` construction — cap then escape (CWE-400/CWE-117)" is the sole authoring site
  for both mechanisms. T-B02 Step 1 specifies `truncate_at_char_boundary(&raw_value, 64)`
  as the cap operation. T-B02 Step 2 specifies the CTL-byte-to-`\xNN` iteration as the
  escape operation. AC-024 (EC-009-047 / CWE-400) and AC-025 (EC-009-048 / CWE-117) are
  the covering story acceptance criteria with Red Gate tests RG-028 and RG-029 respectively.
  No other Wave-A story introduces or owns these mechanisms.
- **Source BC:** BC-2.16.009 — Spec File Validation — Rule 9 §64-codepoint echo cap
  (EC-009-047 / CWE-400) and §CTL-character escaping (EC-009-048 / CWE-117)
- **Security References:** CWE-400 (echo cap) / CWE-117 (CTL escaping)
- **Module:** prism-spec-engine
- **Category:** Security / Output Sanitization

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | kani (CBMC model checking) | Yes — Property 1: string inputs bounded to ≤128 bytes; Property 2: string inputs bounded to ≤32 bytes; both exhaustive over all byte sequences within the bound | CWE-400: `truncate_at_char_boundary` always returns string with `char_count ≤ 64`; CWE-117: CTL-escape function never emits raw CTL bytes (0x00–0x1F, 0x7F) in output |

**Why Kani:** Both `truncate_at_char_boundary` and the CTL-escape function are pure
(inputs → output) with no I/O, no async, and no global state. The CWE-400 cap property
(`char_count ≤ 64`) and the CWE-117 no-raw-CTL property are bounded invariants amenable
to Kani's model-checking approach. Proptest would cover these probabilistically; Kani
provides exhaustive proof for all byte sequences within the bounded input space.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
//
// SYMBOL RESOLUTION — formal-verifier must verify grounding before authoring harnesses
//
// HARNESS 1 target: truncate_at_char_boundary — CONFIRMED REAL
//   Module: prism_spec_engine::validation (confirmed present in prism-spec-engine,
//   pub(crate)). Plain citation; no [PLANNED] marker required.
//
// HARNESS 2 target: [PLANNED: escape_ctl_bytes_for_error_message] — PROVISIONAL NAME
//   The CTL-escaping function specified by BC-2.16.009 E-SPEC-027 template (a)
//   cap-then-escape step (S-WAVE-A-ENGINE-001 §Tasks T-B02 Step 2). The identifier
//   `escape_ctl_bytes_for_error_message` is PROVISIONAL: it does not exist in the
//   codebase as of Phase 3. Anchor obligation: S-WAVE-A-ENGINE-001 AC-025 / RG-029
//   (EC-009-048 / CWE-117). The formal-verifier resolves the actual symbol name at
//   Phase 5 once the implementer has landed S-WAVE-A-ENGINE-001 §Tasks T-B02 Step 2
//   — grep prism-spec-engine for the function mapping CTL bytes 0x00–0x1F,
//   0x7F to `\xNN` four-char ASCII literals in the E-SPEC-027(a) construction path.
//
// HARNESS 1 — CWE-400 cap: truncate_at_char_boundary always produces char_count ≤ max
// (CONFIRMED — truncate_at_char_boundary grounded in prism_spec_engine::validation)
//
// #[kani::proof]
// fn verify_truncate_at_char_boundary_cap() {
//     let bytes: [u8; 128] = kani::any();
//     // Guard: only proceed for valid UTF-8 (truncate_at_char_boundary takes &str)
//     if let Ok(s) = std::str::from_utf8(&bytes) {
//         let capped = truncate_at_char_boundary(s, 64);
//         // Primary cap property: result char count never exceeds the bound
//         assert!(capped.chars().count() <= 64,
//             "truncate_at_char_boundary must never return more than 64 chars");
//         // Prefix property: capped result is always a prefix of the input
//         assert!(s.starts_with(capped),
//             "truncate_at_char_boundary must return a prefix of the input");
//     }
// }
//
// HARNESS 2 — CWE-117: CTL-escape output never contains raw CTL bytes
// (TARGET IS [PLANNED] — resolve actual symbol before authoring; see SYMBOL RESOLUTION above)
//
// #[kani::proof]
// fn verify_ctl_escape_removes_raw_ctl_bytes() {
//     let bytes: [u8; 32] = kani::any();
//     if let Ok(s) = std::str::from_utf8(&bytes) {
//         // [PLANNED: escape_ctl_bytes_for_error_message] — provisional name for the
//         // CTL-escaping function specified by BC-2.16.009 E-SPEC-027 template (a)
//         // cap-then-escape step. Formal-verifier: replace with actual symbol name
//         // after landing S-WAVE-A-ENGINE-001 §Tasks T-B02 Step 2. AC-025 / RG-029.
//         let escaped = escape_ctl_bytes_for_error_message(s); // [PLANNED]
//         // Primary CTL property: no raw CTL byte survives in the output
//         for b in escaped.as_bytes() {
//             let is_ctl = *b <= 0x1F || *b == 0x7F;
//             assert!(!is_ctl,
//                 "CTL-escape must replace all CTL bytes with \\xNN literals");
//         }
//     }
// }
//
// Kill conditions (mutation testing — these mutations MUST be caught):
//   Harness 1:
//   - Change truncation bound 64 → 65  → harness fails on any 65-char input
//   - Remove the truncation entirely   → harness fails on any >64-char input
//   Harness 2:
//   - Remove a CTL byte range from the escape predicate
//     → harness fails when input contains a byte from that range
//   - Replace `\xNN` substitution with identity (emit raw byte)
//     → harness fails on any CTL-byte input
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Harness 1: 128-byte UTF-8 strings (covers ≤128 codepoints, sufficient to stress the 64-codepoint cap); Harness 2: 32-byte strings (sufficient to prove no CTL byte survives — all 256 byte values reached within 32-byte exhaustive space) |
| Tool support? | Full | Both functions are pure with no I/O, no async, no side effects; directly verifiable by Kani |
| Execution time budget | < 60 seconds | Bounded string space + simple per-byte/per-char operations; Kani CBMC handles well for both harnesses |
| Assumptions required | None | No external dependencies; no test helpers required beyond `kani::any::<[u8; N]>()` |
| Platform constraint | Linux/macOS only | Kani uses CBMC backend; not supported on Windows. CI runs proof on Linux; Windows contributors rely on AC-024/RG-028 and AC-025/RG-029 concrete unit tests in S-WAVE-A-ENGINE-001. |
| UTF-8 validity | Conditional | Harnesses assume valid UTF-8 (using `from_utf8` guard). Invalid byte sequences are structurally excluded from `&str`; the error message construction operates on `&str` values already decoded from TOML. |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-07-26 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | FB68c | 2026-07-27 | architect | F-WASE-P65-LOW-001: `modified: []` corrected to `modified: "2026-07-27"` (last modification date per dominant VP convention established by VP-153, VP-154, VP-155, VP-156, and VP-159 — `modified: "YYYY-MM-DD"` tracks last-modification; `modified: null` or `modified: []` indicates never modified since creation). `timestamp: 2026-07-26T00:00:00Z` confirmed correct as creation date (v1.0, FB62). Frontmatter-only hygiene fix; no property semantics, method, module, or priority changed. |
| 1.2 | FB64 | 2026-07-27 | architect | F-WASE-P65-HIGH-001: CTL-escape byte domain corrected from `0x00–0x08, 0x0A–0x1F, 0x7F` (excludes TAB) to `0x00–0x1F, 0x7F` (inclusive, TAB included). Three sites in artifact body corrected: §Property Statement Property 2 prose, §Proof Method table Coverage cell, SYMBOL RESOLUTION comment byte-set description. Harness 2 predicate corrected: `*b <= 0x08 \|\| (*b >= 0x0A && *b <= 0x1F) \|\| *b == 0x7F` → `*b <= 0x1F \|\| *b == 0x7F`. Authority: BC-2.16.009 §Validation Rule 9 §CTL-character escaping clause (`(b as u8) <= 0x1F \|\| (b as u8) == 0x7F`), confirmed by error-taxonomy E-SPEC-027 `{value}` description and S-WAVE-A-ENGINE-001 AC-025 §Tasks T-B02 Step 2 (all three specify full inclusive range). Same-burst POL-9 propagation: VP-INDEX v2.16→v2.17; verification-architecture v1.47→v1.48. |
| 1.1 | FB62 | 2026-07-27 | architect | Pre-commit defect fix: `escape_ctl_bytes_for_error_message` was an unmarked provisional symbol absent from the codebase. SYMBOL RESOLUTION block added to §Proof Harness Skeleton preamble distinguishing CONFIRMED target (`truncate_at_char_boundary`, grounded in `prism_spec_engine::validation`) from PROVISIONAL target. `[PLANNED: escape_ctl_bytes_for_error_message]` marker added at Harness 2 declaration and call site; anchor obligation AC-025/RG-029, Phase 5 resolution path, and behavior description (BC-2.16.009 E-SPEC-027 template (a) cap-then-escape) anchored inline. Harness 1 labeled CONFIRMED; Harness 2 labeled TARGET IS [PLANNED]. |
| 1.0 | FB62 | 2026-07-26 | architect | Initial draft. F-WASE-P64-OBS-002: CWE-400 64-codepoint echo cap (`truncate_at_char_boundary`) and CWE-117 CTL-byte `\xNN` escaping in E-SPEC-027 template (a) error message construction. Successor to VP-160 scope note deferral — VP-160 §Property Statement previously deferred these formatting concerns to "a separate property"; this VP is that property. Method: Kani. P0. Anchor: S-WAVE-A-ENGINE-001 §Tasks T-B02 cap-then-escape specification (AC-024/RG-028 for CWE-400; AC-025/RG-029 for CWE-117). |
