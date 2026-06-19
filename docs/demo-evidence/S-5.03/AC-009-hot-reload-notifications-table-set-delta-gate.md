# AC-009 — Hot-reload sends MCP list_changed notifications on table-set delta only

**AC:** AC-9 (BC-2.16.007 v1.7 — table-set-delta gate)
**Modality:** Test-execution transcript — real duplex MCP transport (Rust)
**Tests:**
- `test_BC_2_16_007_hot_reload_sends_mcp_list_changed_notification` — end-to-end wire-level notification
- `test_BC_2_16_007_reload_config_wires_dispatch_hot_reload_notifications` (server::tests) — wiring into `reload_config` tool
**File:** `crates/prism-mcp/tests/resources.rs`, `crates/prism-mcp/src/server.rs`

---

## Scenario

This test uses a real `tokio::io::duplex` transport + `rmcp::serve_server` to obtain a genuine
`Peer<RoleServer>` and verify JSON-RPC notifications appear on the wire.

**Scenario A — table set CHANGES (crowdstrike_detections only → +claroty_assets added):**
Both notifications must be received on the client side:
- `notifications/resources/list_changed`
- `notifications/tools/list_changed`

**Scenario B — table set UNCHANGED (same old_tables == new_tables):**
Neither notification must be dispatched. The test verifies via a 200ms timeout that
no `list_changed` message appears on the wire.

**Column-only change** (not tested here — column-delta notification is deferred to S-5.11 per BC-2.16.007 v1.7):
BC-2.16.007: notifications fire ONLY when the set of registered table names changes.
A column-only spec change does NOT trigger notifications.

## Command

```
cargo nextest run -p prism-mcp -E 'test(BC_2_16_007)'
```

## Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.56s
────────────
 Nextest run ID cca5c4b7-e4b9-4c37-b368-c719c10fddb1 with nextest profile: default
    Starting 2 tests across 8 binaries (242 tests skipped)
        PASS [   0.063s] (1/2) prism-mcp server::tests::test_BC_2_16_007_reload_config_wires_dispatch_hot_reload_notifications
        PASS [   0.235s] (2/2) prism-mcp::resources test_BC_2_16_007_hot_reload_sends_mcp_list_changed_notification
────────────
     Summary [   0.236s] 2 tests run: 2 passed, 242 skipped
```

## Assertions verified

Scenario A (table set changes):
- `dispatch_result.is_ok()` — both notifications dispatched without error
- `resource_list_changed_received == true` — `notifications/resources/list_changed` seen on wire
- `tool_list_changed_received == true` — `notifications/tools/list_changed` seen on wire

Scenario B (table set unchanged):
- `same_result.is_ok()` — no error when nothing to notify
- No `list_changed` message appears on wire within 200ms timeout

Wiring (server::tests):
- `reload_config` tool handler calls `dispatch_hot_reload_notifications` with the pre/post table sets and `context.peer`

## Observed result

PASS — both notifications dispatched on table-set change; neither dispatched on same-table-set; `reload_config` wired correctly.
