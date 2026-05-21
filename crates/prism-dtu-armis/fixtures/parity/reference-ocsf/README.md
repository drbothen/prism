# Armis Parity Reference OCSF Fixtures

Fixtures not yet recorded — parity tests are tagged `#[ignore]` until
DTU clone story S-6.10 merges (DTU-EXT-003 + DTU-EXT-004 resolved).

## Recording Procedure (ADR-028 §D3)

1. Start Armis DTU clone server
2. Run legacy `ArmisAdapter::fetch()` against DTU clone for devices table
3. Capture OCSF-normalized output (include both AQL forwarding and timestamp fallback cases)
4. Serialize to canonical JSON (sorted keys, `serde_json::to_string`)
5. Save to `devices.json` and `alerts.json` in this directory
6. Commit to git — fixtures are NEVER regenerated at test runtime

## Files Expected

- `devices.json` — reference OCSF output for devices table (AQL forwarding + timestamp fallback)
- `alerts.json` — reference OCSF output for alerts table
