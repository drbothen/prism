# AC-10 — MCP Traffic Gate: Deferred Demo

**Status:** deferred-testable  
**Deferred to:** S-3.02-FOLLOWUP-RUNTIME  

## Why No Live Demo

AC-10 requires the MCP stdio transport (step 9) and QueryEngine (step 8) to be running
so that a tool call can arrive and be held at the traffic gate until step 8 completes.

In the current chassis (`prism-bin` v0.1.0), steps 7-11 are annotated `todo!()` stubs
per ADR-022 §G. The `PrismServer` struct does not yet exist (resolved by
`S-5.01-FOLLOWUP-MCP-BOOT`) and `QueryEngine::execute` is also a stub (resolved by
`S-3.02-FOLLOWUP-RUNTIME`). Without these, no MCP traffic can arrive on the stdio
transport during boot steps 1-7 for the gate to intercept.

The traffic gate implementation is specified at `BC-2.22.001` invariant 4 and is
enforced by step 8's completion state. The chassis ships the gate contract but cannot
demonstrate it end-to-end until the sibling stories fill steps 7-9.

## What the Story Says

From AC-10 in `S-WAVE5-PREP-01-prism-bin-chassis.md`:

> Given `prism start` has completed boot steps 1-7 (RocksDB open) but has not yet
> completed step 8 (QueryEngine ready), When an MCP tool call arrives on the stdio
> transport, Then the binary does not service the request until step 8 completes
> (traffic gate enforced by BC-2.22.001).

The story itself traces this to BC-2.22.001 invariant — MCP traffic gate must block
until step 8 completes. It is the only AC in this story that depends on sibling story
implementation.

## Evidence Trail

- `BC-2.22.001` specifies the traffic gate contract.
- `crates/prism-bin/src/boot.rs` step 8 stub includes the comment:
  `// TODO(S-WAVE5-PREP-01/S-3.02-FOLLOWUP-RUNTIME): Construct QueryEngine + WriteExecutor.`
- Step 9 stub: `todo!("S-WAVE5-PREP-01 step 9 — MCP server boot — resolved by S-5.01-FOLLOWUP-MCP-BOOT")`
- The gate will be demo'd in the `S-3.02-FOLLOWUP-RUNTIME` evidence package.
