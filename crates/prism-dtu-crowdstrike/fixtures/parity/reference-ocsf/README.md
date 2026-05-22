# CrowdStrike Parity Reference OCSF Fixtures

Fixtures not yet recorded — parity tests are tagged `#[ignore]` until
DTU clone story S-6.07 merges.

## Recording Procedure (ADR-028 §D3)

1. Start CrowdStrike DTU clone server
2. Run legacy `CrowdStrikeAdapter::fetch()` against DTU clone
3. Capture OCSF-normalized output
4. Serialize to canonical JSON (sorted keys, `serde_json::to_string`)
5. Save to `detections.json` and `devices.json` in this directory
6. Commit to git — fixtures are NEVER regenerated at test runtime

## Files Expected

- `detections.json` — reference OCSF output for detections table
- `devices.json` — reference OCSF output for devices table
