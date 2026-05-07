# TD-S305-MCP-PAYLOAD-TEST-001: Vacuous token-not-in-row-data test (O-3)

**Filed:** 2026-05-07
**Story:** S-3.05
**Adversary pass:** LOCAL pass-1 observation O-3
**Severity:** tech_debt
**Status:** open / deferred

## Finding

`test_BC_2_07_001_token_not_embedded_in_row_data` in `pagination_tests.rs` asserts that
cursor tokens do not appear in row data. As of S-3.05, cursor tokens are never propagated
to the MCP response layer, so the test is vacuous — it checks a trivially-true condition
against the internal `CursorEntry` struct rather than the actual MCP tool response payload.

## Why Deferred

Testing token absence in MCP payloads requires integration with the MCP tool handler
(S-5.x scope). The unit test provides no meaningful coverage of the actual behavior.

## Resolution Path

In S-5.x MCP integration: add an integration test that:
1. Issues a multi-page query via the MCP `query` tool
2. Inspects the returned JSON blob
3. Asserts no field named `cursor_token`, `token`, or matching UUID v4 format appears
   in the response rows

## Test Location

`crates/prism-query/src/tests/pagination_tests.rs::test_BC_2_07_001_token_not_embedded_in_row_data`
