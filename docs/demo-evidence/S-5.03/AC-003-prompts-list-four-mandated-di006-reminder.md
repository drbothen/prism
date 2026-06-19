# AC-003 — `prompts/list` includes four mandated prompts; all include DI-006 security reminder

**AC:** AC-3 (BC-2.10.009 postconditions 1–4; DI-006 invariant)
**Modality:** Test-execution transcript — MCP stdio server (Rust)
**Tests:**
- `test_BC_2_10_009_prompts_list_includes_four_mandated_prompts` — four mandatory names registered
- `test_BC_2_10_009_triage_alerts_includes_security_reminder` — DI-006 reminder in `triage_alerts`
- `test_BC_2_10_009_investigate_host_includes_security_reminder` — DI-006 reminder
- `test_BC_2_10_009_client_overview_includes_security_reminder` — DI-006 reminder
- `test_BC_2_10_009_cross_client_status_includes_security_reminder` — DI-006 reminder
**File:** `crates/prism-mcp/tests/resources.rs`

---

## Scenario

`build_prompt_router()` constructs a `PromptRouter<PrismServer>` with all four mandated prompts.
Each prompt is rendered with its required arguments and the message text is scanned for the
DI-006 security reminder ("untrusted").

Mandated prompt names (BC-2.10.009 canonical):
- `triage_alerts` (argument: `client_id`)
- `investigate_host` (arguments: `client_id`, `hostname`)
- `client_overview` (argument: `client_id`)
- `cross_client_status` (argument: `time_range` — optional)

## Command

```
cargo nextest run -p prism-mcp -E 'test(BC_2_10_009_prompts) or test(BC_2_10_009_triage_alerts) or test(BC_2_10_009_investigate_host) or test(BC_2_10_009_client_overview) or test(BC_2_10_009_cross_client_status)'
```

## Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.62s
────────────
 Nextest run ID a99e402b-eecb-471d-a439-8747d16db3ad with nextest profile: default
    Starting 5 tests across 8 binaries (242 tests skipped)
        PASS [   0.031s] (1/5) prism-mcp::resources test_BC_2_10_009_prompts_list_includes_four_mandated_prompts
        PASS [   0.031s] (2/5) prism-mcp::resources test_BC_2_10_009_cross_client_status_includes_security_reminder
        PASS [   0.031s] (3/5) prism-mcp::resources test_BC_2_10_009_client_overview_includes_security_reminder
        PASS [   0.031s] (4/5) prism-mcp::resources test_BC_2_10_009_investigate_host_includes_security_reminder
        PASS [   0.032s] (5/5) prism-mcp::resources test_BC_2_10_009_triage_alerts_includes_security_reminder
────────────
     Summary [   0.032s] 5 tests run: 5 passed, 242 skipped
```

## Assertions verified

- `router.list_all().len() == 4` — exactly 4 prompts registered
- `names.contains("triage_alerts")` — BC-2.10.009 canonical name present
- `names.contains("investigate_host")` — canonical name present
- `names.contains("client_overview")` — canonical name present
- `names.contains("cross_client_status")` — canonical name present
- All four prompt messages contain `"untrusted"` (DI-006: security reminder about untrusted sensor data)

## Observed result

PASS — all four mandated prompts registered with canonical names; each includes DI-006 security reminder.
