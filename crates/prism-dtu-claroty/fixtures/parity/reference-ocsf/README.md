# Claroty Parity Reference OCSF Fixtures

Fixtures not yet recorded — parity tests are tagged `#[ignore]` until
DTU clone story S-6.08 merges.

## Recording Procedure (ADR-028 §D3)

1. Start Claroty DTU clone server
2. Run legacy `ClarotyAdapter::fetch()` against DTU clone
3. Capture OCSF-normalized output
4. Serialize to canonical JSON (sorted keys, `serde_json::to_string`)
5. Save to `alerts.json` in this directory
6. Commit to git — fixtures are NEVER regenerated at test runtime

## Files Expected

- `alerts.json` — reference OCSF output for alerts table (polymorphic ID cases)
