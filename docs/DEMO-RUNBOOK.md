# Prism DTU Demo Runbook

This runbook guides an MSSP analyst or demo operator through standing up the Prism DTU demo
environment, connecting Claude Code as an MCP client, and issuing live PrismQL queries against
all sensor DTU clones across three demo orgs (org-a, org-b, org-c).

Stories: S-DEMO-003 | S-DEMO-LAUNCHER-CONSOLIDATION-001 | ACs: AC-009, AC-011

---

## 1. Prerequisites

Before running setup, ensure the following are installed:

| Tool | Check | Install |
|------|-------|---------|
| Rust toolchain | `cargo --version` | `curl https://sh.rustup.rs -sSf \| sh` (then follow `rust-toolchain.toml`) |
| `just` | `just --version` | `cargo install just` |
| `cargo-nextest` | `cargo nextest --version` | `cargo install cargo-nextest` |
| `shellcheck` | `shellcheck --version` | `brew install shellcheck` (macOS) or `apt install shellcheck` |
| macOS Keychain or libsecret | OS-provided | Built-in on macOS; `apt install gnome-keyring` on Debian/Ubuntu |

Verify the repo is cloned and the Rust toolchain matches `rust-toolchain.toml`:

```bash
cargo --version  # should print the pinned stable version
just --list      # should list available recipes
```

---

## 2. One-time Setup

Run the setup script from the repo root (idempotent — safe to run multiple times):

```bash
bash scripts/demo-setup.sh
```

What the script does:

1. **Checks** prerequisites (`cargo` must be available)
2. **Builds** `prism` and `prism-dtu-demo-server` (release mode, features: `dtu,fixture-gen`)
3. **Creates** `~/.config/prism-demo/` with `specs/`, `specs/customers/{org-a,org-b,org-c}/`, `state/`, `plugins/`, and `run/` subdirectories
4. **Copies** the four sensor TOML specs (`crowdstrike`, `armis`, `claroty`, `cyberint`) to `~/.config/prism-demo/specs/`
5. **Copies** `crowdstrike-oauth2.prx` plugin artifact to `~/.config/prism-demo/plugins/`
6. **Writes** `~/.config/prism-demo/plugins/crowdstrike-oauth2.manifest.toml` — a DTU-safe plugin manifest that extends `allowed_urls` with `"127.0.0.1"` so the SEC-003 OAuth2 token-endpoint host check passes against the local DTU clone
7. **Writes** `~/.config/prism-demo/prism.toml` with **3 `[[orgs]]` entries**: org-a, org-b, org-c (UUID v7 `org_id` values matching `scripts/demo.toml [orgs.*]`)
8. **Stores** 10 dummy credentials in the OS keyring via `prism credential set` (N×M: one per org × sensor combo) and prints next-step instructions

### 3-Org Demo Environment

| Org | Sensors | Credentials |
|-----|---------|-------------|
| org-a | crowdstrike, armis | 3 (client_id, client_secret, bearer_token) |
| org-b | claroty, cyberint | 2 (bearer_token, api_key) |
| org-c | crowdstrike, armis, claroty, cyberint | 5 (all sensors) |
| **Total** | | **10 credentials** |

Override the config directory if needed:

```bash
bash scripts/demo-setup.sh --config-dir /custom/path/prism-demo
```

---

## 3. Daily Demo Run

After setup, start the DTU server and follow the instructions printed:

```bash
bash scripts/demo-run.sh
```

The script:

1. **Starts** `prism-dtu-demo-server start-multi --config scripts/demo.toml` in the background (one process manages all 3 orgs × N sensors = 8 DTU clone instances on ephemeral ports)
2. **Polls** for the nested sidecar `~/.config/prism-demo/run/.prism-dtu-demo-server.urls-multi.json` for up to 30 seconds (EC-006)
3. **Prints** the DTU clone URLs grouped by org slug:
   ```
   org-a/crowdstrike    : http://127.0.0.1:<port>
   org-a/armis          : http://127.0.0.1:<port>
   org-b/claroty        : http://127.0.0.1:<port>
   org-b/cyberint       : http://127.0.0.1:<port>
   org-c/crowdstrike    : http://127.0.0.1:<port>
   org-c/armis          : http://127.0.0.1:<port>
   org-c/claroty        : http://127.0.0.1:<port>
   org-c/cyberint       : http://127.0.0.1:<port>
   ```
4. **Generates N×M overlay TOMLs** — reads the nested `{org_slug: {sensor_id: url}}` sidecar and writes 8 files at `~/.config/prism-demo/specs/customers/<org_slug>/<sensor_id>.sensor.toml`. Each overlay sets `base_url = "http://127.0.0.1:<ephemeral-port>"` so prism routes fetch requests to the correct per-org DTU clone. Per BC-2.06.013, each overlay contains only three scalar fields (`extends`, `instance_id`, `base_url`). prism reads these at boot step 4c via `customers_dir = spec_dir + "/customers"`.
5. **Prints** the exact command to start `prism`

Start `prism` in a **new terminal** using the command printed by `demo-run.sh`:

```bash
CROWDSTRIKE_BASE_URL=http://127.0.0.1 \
ARMIS_INSTANCE_URL=http://127.0.0.1 \
CLAROTY_INSTANCE_URL=http://127.0.0.1 \
CYBERINT_ENVIRONMENT=demo \
target/release/prism --config-dir ~/.config/prism-demo start
# (use the exact command printed by demo-run.sh — it includes the full absolute path)
```

`prism start` boots all 11 boot steps and emits `boot.step9a.adapter_registry_populated`
with `sensor_count=4, org_count=3` when all DTU adapters are registered. It then accepts
MCP connections via stdio transport.

---

## 4. Connecting Claude Code

Add `prism` as an MCP server in your Claude Code settings:

**File location:** `~/.claude/settings.json`

**Add the following JSON** under the `mcpServers` key (create the key if absent):

```json
{
  "mcpServers": {
    "prism": {
      "command": "/Users/YOUR_USERNAME/Dev/prism/target/release/prism",
      "args": ["--config-dir", "/Users/YOUR_USERNAME/.config/prism-demo", "start"],
      "env": {
        "CROWDSTRIKE_BASE_URL": "http://127.0.0.1",
        "ARMIS_INSTANCE_URL": "http://127.0.0.1",
        "CLAROTY_INSTANCE_URL": "http://127.0.0.1",
        "CYBERINT_ENVIRONMENT": "demo"
      }
    }
  }
}
```

Replace `/Users/YOUR_USERNAME` with your actual home directory path (use `echo $HOME` to find it).

**Why these 4 env vars are required** (E-SPEC-024 at boot step 4a):
The four sensor TYPE specs use `${env.VAR}` tokens in their `base_url` fields.
Prism resolves these tokens at boot step 4a (env resolution, before step 4c overlay
loading). If any variable is absent or empty, the boot process hard-aborts with exit
code 2 before the per-org overlay can wire the DTU clone URL. The values above satisfy
step 4a resolution — the per-org sensor overlays (generated by `demo-run.sh` at step 4c)
replace `base_url` with the actual DTU clone port before any HTTP request is made.

`CROWDSTRIKE_BASE_URL` must be `http://127.0.0.1` specifically: the `crowdstrike-oauth2`
plugin manifest's `allowed_urls` includes `["api.crowdstrike.com", "127.0.0.1"]`. SEC-003
validates the TYPE spec `base_url` host against this list at step 7.5b. The DTU-safe
manifest written by `demo-setup.sh` already includes `127.0.0.1`.

**Tier-2 env var fallback names** (if keyring is unavailable — BC-2.06.003 ADR-032):

```bash
# org-a (3 credentials)
PRISM_CLIENTS_ORG_A_SENSORS_CROWDSTRIKE_CLIENT_ID="demo-cs-client-id-org-a"
PRISM_CLIENTS_ORG_A_SENSORS_CROWDSTRIKE_CLIENT_SECRET="demo-cs-client-secret-org-a"
PRISM_CLIENTS_ORG_A_SENSORS_ARMIS_BEARER_TOKEN="demo-armis-bearer-token-org-a"

# org-b (2 credentials)
PRISM_CLIENTS_ORG_B_SENSORS_CLAROTY_BEARER_TOKEN="demo-claroty-bearer-token-org-b"
PRISM_CLIENTS_ORG_B_SENSORS_CYBERINT_API_KEY="demo-cyberint-api-key-org-b"

# org-c (5 credentials)
PRISM_CLIENTS_ORG_C_SENSORS_CROWDSTRIKE_CLIENT_ID="demo-cs-client-id-org-c"
PRISM_CLIENTS_ORG_C_SENSORS_CROWDSTRIKE_CLIENT_SECRET="demo-cs-client-secret-org-c"
PRISM_CLIENTS_ORG_C_SENSORS_ARMIS_BEARER_TOKEN="demo-armis-bearer-token-org-c"
PRISM_CLIENTS_ORG_C_SENSORS_CLAROTY_BEARER_TOKEN="demo-claroty-bearer-token-org-c"
PRISM_CLIENTS_ORG_C_SENSORS_CYBERINT_API_KEY="demo-cyberint-api-key-org-c"
```

**Verify the MCP connection from Claude Code:**

```
/mcp list
```

You should see `prism` listed as a connected server with `query` available (the canonical MCP tool name registered via `pub async fn query` in prism-mcp; BC-2.11.001).

---

## 5. Example Queries

Once connected, issue PrismQL queries via the `query` MCP tool (the canonical tool name;
`FROM x LIMIT n` is pipe syntax and is rejected — use SQL form `SELECT * FROM x LIMIT n`).
Use `/mcp` in Claude Code to invoke MCP tools.

**CrowdStrike — recent detections (org-a):**

```sql
SELECT * FROM crowdstrike_detections LIMIT 5
```

Expected output: OCSF-normalized Arrow batch with columns `sensor`, `_time`, and detection fields. When multiple orgs (org-a, org-c) have CrowdStrike enabled, prism fans out to both and returns rows from each distinct DTU clone.

**Armis — device inventory:**

```sql
SELECT * FROM armis_devices LIMIT 5
```

Expected output: OCSF-normalized rows with device attributes from org-a and org-c Armis DTU clones.

**Claroty — OT asset inventory:**

```sql
SELECT * FROM claroty_devices LIMIT 5
```

Expected output: OCSF-normalized rows with OT asset metadata from org-b and org-c Claroty clones.

**Cyberint — threat intelligence alerts:**

```sql
SELECT * FROM cyberint_alerts LIMIT 5
```

Expected output: OCSF-normalized alert rows from org-b and org-c Cyberint clones.

All queries fan out to the appropriate DTU clones running on localhost. No real external
API calls are made during demo mode. Each org's DTU clone is seeded with a distinct `seed`
value (org-a=100, org-b=150, org-c=200) so each org's data is distinct.

---

## 6. Troubleshooting

### (a) Keyring write failure at credential-set time — E-CRED-004

**Symptom:** `prism credential set` prints `Keyring unavailable: ...` and exits
non-zero. The credential is NOT stored. This is the **write path** error.

**Error code:** `E-CRED-004` (`PrismError::CredentialStoreError`, displayed as
`"E-CRED-004: credential store error (backend=...): {reason}"`), surfaced by
`handle_credential_set` in `credential_cli.rs`.

**Cause:** The OS keyring service is unavailable at credential-set time (headless
server, Docker container, missing Gnome Keyring on Linux, or locked macOS Keychain).

**Fix (macOS):** Unlock the macOS Keychain:
```bash
security unlock-keychain ~/Library/Keychains/login.keychain-db
```
Then re-run `prism credential set ...`.

**Fix (Linux):** Start the D-Bus keyring service:
```bash
eval "$(dbus-launch --sh-syntax)"
gnome-keyring-daemon --start --components=secrets
```
Then re-run `prism credential set ...`.

**Fallback (CI / headless):** Use environment variables instead of the keyring.
Prism resolves credentials from env vars before checking the keyring (Tier 1/2 in
the four-tier resolution chain — BC-2.06.003). See the 10 `PRISM_CLIENTS_*` env vars
listed in §4 above for all fallback names.

### (b) Keyring read failure at query time — E-CRED-008

**Symptom:** A query against a demo sensor fails at authentication time with:
`BackendUnavailable { detail: "E-CRED-008: OS keyring unavailable: ..." }`.
This is the **read path** error — the credential was previously stored but the OS keyring
is inaccessible during Tier-3 resolution.

**Error code:** `E-CRED-008` (surfaced by `resolution.rs` Tier-3 keyring backend).
There is no Tier-4 fallthrough: if the keyring is unavailable at read time, the error is
hard-returned with no credential value leak.

**Fix (macOS):** Unlock the macOS Keychain:
```bash
security unlock-keychain ~/Library/Keychains/login.keychain-db
```

**Fix (Linux):** Restart the D-Bus keyring service:
```bash
eval "$(dbus-launch --sh-syntax)"
gnome-keyring-daemon --start --components=secrets
```

**Alternative:** Use the `PRISM_CLIENTS_*` environment variable fallback (Tier 2). See §4 for all 10 names.

### (c) Port already in use

**Symptom:** `demo-run.sh` exits with "demo-server start-multi did not write sidecar
within 30s" or the DTU log shows `Address already in use (os error 98)`.

**Cause:** A previous DTU server is still running, or another process holds the port.

**Fix:**

1. Kill any stale DTU server:
   ```bash
   cat ~/.config/prism-demo/run/.prism-dtu-demo-server.pid | xargs kill 2>/dev/null || true
   ```

2. The demo uses **ephemeral ports** (bound at runtime). Read the actual port numbers
   from the nested sidecar if it was written:
   ```bash
   URLS=~/.config/prism-demo/run/.prism-dtu-demo-server.urls-multi.json
   if [[ -f "$URLS" ]]; then
     python3 -c "
import json, sys
with open('$URLS') as f:
    nested = json.load(f)
for org, sensors in nested.items():
    for sensor, url in sensors.items():
        port = url.rsplit(':', 1)[-1]
        print(port)
" | sort -u | xargs -I{} lsof -ti :{} 2>/dev/null | sort -u | xargs kill -9 2>/dev/null || true
   fi
   ```

3. Re-run `demo-run.sh`.

### (d) TOML spec not found

**Symptom:** `prism start` exits 2 with an error like `spec_dir does not exist` or
`parse_spec_directory: no .sensor.toml files found`.

**Cause:** The `spec_dir` in `prism.toml` points to an empty or nonexistent directory,
or the sensor TOML files were not copied by `demo-setup.sh`.

**Fix:**

1. Verify `spec_dir` in `~/.config/prism-demo/prism.toml`:
   ```bash
   cat ~/.config/prism-demo/prism.toml | grep spec_dir
   ```

2. List sensor TOML files in the specs directory:
   ```bash
   ls "$(cat ~/.config/prism-demo/prism.toml | grep spec_dir | awk -F'"' '{print $2}')"
   ```
   Expected: `armis.sensor.toml  claroty.sensor.toml  crowdstrike.sensor.toml  cyberint.sensor.toml`

3. If files are missing, re-run setup:
   ```bash
   bash scripts/demo-setup.sh
   ```

---

## 7. Cleanup

To completely remove the demo environment:

```bash
bash scripts/demo-teardown.sh
```

What the script does:

1. **Kills** the single `prism-dtu-demo-server start-multi` process (via PID file at `~/.config/prism-demo/run/.prism-dtu-demo-server.pid`)
2. **Deletes** all 10 demo OS keyring entries via `prism credential delete` (OrgId-keyed namespace; ADR-034 §D3). Keyring deletes run **BEFORE** `rm -rf` so `prism.toml` is still available for OrgId resolution (F-P10-HIGH-001 invariant).
   - org-a: `crowdstrike/client_id`, `crowdstrike/client_secret`, `armis/bearer_token`
   - org-b: `claroty/bearer_token`, `cyberint/api_key`
   - org-c: `crowdstrike/client_id`, `crowdstrike/client_secret`, `armis/bearer_token`, `claroty/bearer_token`, `cyberint/api_key`
3. **Removes** `~/.config/prism-demo/` entirely (all config, specs, state, plugins, logs)

If no PID file is found (EC-005), the server kill is skipped and teardown continues idempotently.

The binary artifacts (`target/release/prism`, `target/release/prism-dtu-demo-server`) are NOT
removed — run `cargo clean` separately if needed.

Override the config directory:

```bash
bash scripts/demo-teardown.sh --config-dir /custom/path/prism-demo
```
