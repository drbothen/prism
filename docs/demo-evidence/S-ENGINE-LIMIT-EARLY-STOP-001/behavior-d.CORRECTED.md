# behavior-d.json — Correction Note

**Corrected under:** DEFECT-LIVE-ENVELOPE-OBS-001 (PR #251)
**Original capture:** 2026-08-30T08:11:22Z (pre-fix binary)
**Field corrected:** `_meta.has_more` — changed `true` → `false` in both the outer `structuredContent` envelope and the embedded `content[0].text` JSON string

## Rationale

The pre-fix binary incorrectly emitted `has_more: true` on truncated result sets,
violating ADR-060 §D8.7. The capture in `behavior-d.json` reflects that defect.
Rather than re-capturing (which would require a live binary run), the field was
hand-corrected to show the post-fix correct value. The `query_time` field retains
the original timestamp from the pre-fix capture.

## Verification

The corrected shape is mechanically verified by
`test_BC_2_09_008_OBS_2_wire_has_more_always_false_when_truncated` (TEST C) in
`crates/prism-mcp/tests/defect_live_envelope_obs_001_test.rs`, which asserts the
serialized JSON wire shape at the MCP envelope level.
