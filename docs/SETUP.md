# Prism — Operator Setup and Installation Guide

**Version**: v1.0.0-rc.1  
**Audience**: New operators deploying Prism against a live sensor tenant for the first time.

This guide takes you from a fresh machine to a running `prism start` with a working
Claroty xDome sensor connection and Claude Code wired as an MCP client.

> **Note on future releases**: v1.0.0-rc.1 requires the operator to supply sensor
> spec files from the GitHub repository (see §4). A future release will embed built-in
> sensor specs directly in the binary, making the spec placement step optional.
> That work is tracked as S-REL-010 and is not part of this release.

---

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [Install the Binary](#2-install-the-binary)
3. [Verify the Installation](#3-verify-the-installation)
4. [Obtain Sensor Specs](#4-obtain-sensor-specs)
5. [Create a Config Directory](#5-create-a-config-directory)
6. [Edit prism.toml](#6-edit-prismtoml)
7. [Generate a UUID v7 for org_id](#7-generate-a-uuid-v7-for-org_id)
8. [Configure Credentials](#8-configure-credentials)
9. [Onboard a Claroty xDome Client](#9-onboard-a-claroty-xdome-client)
10. [First Boot](#10-first-boot)
11. [Wire Prism into Claude Code](#11-wire-prism-into-claude-code)
12. [First Smoke Query](#12-first-smoke-query)
13. [Next Steps](#13-next-steps)

---

## 1. Prerequisites

| Tool | Minimum | Check | Install |
|------|---------|-------|---------|
| `curl` | any | `curl --version` | OS package manager |
| `tar` | any | `tar --version` | OS package manager (Linux) / built-in (macOS) |
| `gh` CLI | any | `gh --version` | [cli.github.com](https://cli.github.com) (optional — for provenance verification) |
| PowerShell | 5.1+ | `$PSVersionTable.PSVersion` | Built-in on Windows; [download](https://github.com/PowerShell/PowerShell) for older versions |

The install scripts have no Rust toolchain dependency — they download a pre-built binary.

---

## 2. Install the Binary

### macOS and Linux (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.sh | bash
```

The script auto-detects the platform (macOS Apple Silicon, macOS Intel, Linux glibc, Linux musl)
and installs to `/usr/local/bin` (or `~/.local/bin` if `/usr/local/bin` is not writable).
It verifies the SHA-256 checksum before installing and, if `gh` is available, verifies
build provenance via Sigstore.

To pin a specific version:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.sh) \
  --version v1.0.0-rc.1
```

To preview without installing:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.sh) \
  --version v1.0.0-rc.1 --dry-run
```

To skip provenance verification (e.g., `gh` CLI not installed):

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.sh) \
  --skip-verify-provenance
```

### Windows (PowerShell 5.1+)

```powershell
irm https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.ps1 | iex
```

Installs to `%LOCALAPPDATA%\prism\bin\prism.exe`. Verifies SHA-256 checksum before installing.

To pin a specific version (environment variable is required for `irm | iex` — positional
arguments cannot be passed through that pattern):

```powershell
$env:PRISM_INSTALL_VERSION = 'v1.0.0-rc.1'
irm https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.ps1 | iex
```

### Manual download (fallback)

If the install scripts are unavailable, download the archive directly from the
[v1.0.0-rc.1 release page](https://github.com/drbothen/prism/releases/tag/v1.0.0-rc.1):

| Platform | Archive |
|----------|---------|
| macOS (Apple Silicon) | `prism-v1.0.0-rc.1-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `prism-v1.0.0-rc.1-x86_64-apple-darwin.tar.gz` |
| Linux (glibc — Debian, Ubuntu, RHEL) | `prism-v1.0.0-rc.1-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (musl — Alpine, static) | `prism-v1.0.0-rc.1-x86_64-unknown-linux-musl.tar.gz` |
| Windows (x86_64) | `prism-v1.0.0-rc.1-x86_64-pc-windows-msvc.zip` |

Extract and place the binary in a directory on your `PATH`:

```bash
# macOS / Linux example
tar xzf prism-v1.0.0-rc.1-<target>.tar.gz
chmod +x prism
mv prism /usr/local/bin/prism
```

**The install scripts are preferred** — they verify the SHA-256 checksum and (with `gh`)
the Sigstore build provenance, which the manual path requires separately (see §3).

---

## 3. Verify the Installation

### Confirm the binary version

```bash
prism --version
# Expected: prism 1.0.0-rc.1
```

### Verify checksum (manual download only)

Download `checksums.txt` from the release page alongside the archive, then:

```bash
# macOS
shasum -a 256 -c checksums.txt

# Linux
sha256sum -c checksums.txt
```

The install scripts perform this check automatically and abort on mismatch.

### Verify build provenance (optional, requires `gh` CLI)

```bash
gh attestation verify prism-v1.0.0-rc.1-<target>.tar.gz \
  --repo drbothen/prism \
  --signer-workflow drbothen/prism/.github/workflows/release.yml
```

Replace `<target>` with the exact archive filename for your platform. A successful
verification confirms the binary was built by the official GitHub Actions release
workflow, not a third party.

---

## 4. Obtain Sensor Specs

**Important**: The install scripts deploy the `prism` binary only. Sensor TOML spec files
are not bundled in the binary. You must place them at the `spec_dir` path declared in
`prism.toml` (configured in §6).

Download the sensor specs directly from the GitHub repository:

```bash
# Create a directory to hold specs (use any path you prefer)
mkdir -p /etc/prism/specs

# Download the Claroty xDome spec (v1.0.0-rc.1 supported sensor)
curl -fsSL https://raw.githubusercontent.com/drbothen/prism/main/crates/prism-sensors/specs/claroty.sensor.toml \
  -o /etc/prism/specs/claroty.sensor.toml
```

Verify the file arrived:

```bash
grep -c '^\[\[tables\]\]' /etc/prism/specs/claroty.sensor.toml
# Expected: 14
```

Additional sensor specs (CrowdStrike, Armis, Cyberint) are in the repository at
`crates/prism-sensors/specs/`. They are present in the workspace but not validated
against live tenants in rc.1; the Claroty xDome sensor is the only fully supported
sensor in this release. Download additional specs only if you have those sensor tenants
and intend to test them.

---

## 5. Create a Config Directory

Choose a directory where Prism will read its configuration:

```bash
# Recommended paths
mkdir -p /etc/prism        # system-wide
# or
mkdir -p ~/.config/prism   # per-user
```

Download `prism.toml.example` from the repository as a starting template:

```bash
curl -fsSL https://raw.githubusercontent.com/drbothen/prism/main/prism.toml.example \
  -o /etc/prism/prism.toml.example

# Copy to prism.toml to edit
cp /etc/prism/prism.toml.example /etc/prism/prism.toml
```

Also create subdirectories prism expects:

```bash
mkdir -p /etc/prism/state
mkdir -p /etc/prism/specs
```

The `specs/` directory is where you placed (or will place) the sensor TOML specs from §4.

---

## 6. Edit prism.toml

Open `/etc/prism/prism.toml` (or your chosen path) in a text editor. The following
fields require your attention:

### spec_dir

Path to the directory containing `*.sensor.toml` spec files. Point this at the directory
where you placed the Claroty spec in §4.

```toml
spec_dir = "/etc/prism/specs"
```

Relative paths are resolved from the config directory (the directory containing `prism.toml`).

### state_dir

Path for RocksDB persistent state. Prism creates this directory at boot if absent.
Must be writable by the process running `prism start`.

```toml
state_dir = "/var/lib/prism/state"
# or for per-user:
state_dir = "/etc/prism/state"
```

### plugin_dir (optional)

Path for `*.prx` WASM plugin files. No enrichment plugins are bundled in v1.0.0-rc.1.
Omit this field or leave it commented out.

### [[orgs]]

At least one org entry is required. Each entry maps a UUID v7 `org_id` to a kebab-case
`org_slug`. See §7 for how to generate a UUID v7.

```toml
[[orgs]]
org_id   = "REPLACE-WITH-UUID-V7"   # see §7
org_slug = "my-client"              # kebab-case; no spaces; used in audit logs and MCP context
# name   = "My Client Name"        # optional display name
```

**org_id must be UUID version 7** (time-ordered). Prism validates this at boot and exits 2
with `"must be a UUID v7"` if a UUID v4 or v1 is supplied. See §7 for generation methods.

For a multi-tenant deployment, add additional `[[orgs]]` blocks:

```toml
[[orgs]]
org_id   = "REPLACE-WITH-ANOTHER-UUID-V7"
org_slug = "second-client"
```

### credential_backend (optional)

The default backend is the OS keyring (macOS Keychain, Linux Secret Service / kernel
keyutils, Windows Credential Manager). No configuration needed unless the keyring is
unavailable (see §8 for the environment variable fallback).

---

## 7. Generate a UUID v7 for org_id

Prism requires a UUID version 7 for each org's `org_id` field. UUID v7 is time-ordered
(monotonically sortable), which is required by Prism's identity model. UUID v4 and v1
are rejected at boot.

Choose one of these verified methods:

### Linux (util-linux 2.41+)

```bash
uuidgen -7
# or: uuidgen --time-v7
```

`uuidgen` without flags generates UUID v4 — rejected. `uuidgen --time` generates UUID v1
— also rejected. Use `-7` explicitly.

**Note**: Most LTS distros (RHEL 9, Ubuntu 24.04) ship util-linux < 2.41 and do not
support `-7`. Use the Python method below if `uuidgen -7` is not available.

### Python (any platform with Python 3.6+)

```bash
pip install uuid6
python3 -c "import uuid6; print(uuid6.uuid7())"
```

`uuid6` is the standard Python library for RFC-9562 UUID versions 6 and 7. The output
is an RFC-9562-compliant v7 UUID with a 48-bit unix millisecond timestamp prefix.

### Python 3.14+ (stdlib, no install needed)

```bash
python3 -c "import uuid; print(uuid.uuid7())"
```

UUID v7 support was added to the Python standard library in Python 3.14.

### macOS

macOS ships BSD `uuidgen` which generates UUID v4 only. Use the Python method above.

Place the generated UUID in `prism.toml`:

```toml
[[orgs]]
org_id   = "019f80f5-d021-7dfd-bc60-561409449c1e"   # example — generate your own
org_slug = "my-client"
```

---

## 8. Configure Credentials

Credentials are stored in the OS keyring, never in prism.toml or environment variables
visible to shell history. This is the AD-017 opaque credential model: Prism resolves
credential values at query time from the keyring; the values never transit AI context.

### Credential scope

Each credential is scoped to a specific `org_slug × sensor × credential_name` triple.
For the Claroty xDome sensor:

| Field | Value |
|-------|-------|
| Sensor ID | `claroty` |
| Credential name | `bearer_token` |

### prism credential set

Run the following command once for each org that will use Claroty:

```bash
prism credential set \
  --sensor claroty \
  --name bearer_token \
  --config-dir /etc/prism
```

If you have multiple `[[orgs]]` entries in `prism.toml`, also pass `--org-slug`:

```bash
prism credential set \
  --sensor claroty \
  --name bearer_token \
  --org-slug my-client \
  --config-dir /etc/prism
```

The command prompts `Enter value for prism/claroty/bearer_token:` on stderr with
terminal echo disabled. Paste your Claroty xDome API bearer token and press Enter.
The value is written to the OS keyring under the namespace
`{org_id_uuid}/claroty/bearer_token`. It never appears in stdout, logs, or shell history.

**`--value` flag does not exist**: passing `--value <token>` is rejected by design
(the flag is intentionally absent). Value input via stdin only.

**`prism.toml` must exist before running this command.** The CLI loads `prism.toml` to
resolve the `org_id` UUID for the matching `org_slug`. If `prism.toml` is missing, the
command exits 2 with an actionable error.

### Environment variable fallback

If the OS keyring is unavailable (e.g., headless CI, Docker without keyring daemon),
set credentials via environment variables. The format is:

```
PRISM_CLIENTS_{ORG_SLUG_UPPER}_SENSORS_{SENSOR_UPPER}_{CRED_UPPER}
```

Where:
- `{ORG_SLUG_UPPER}` is the `org_slug` uppercased with hyphens replaced by underscores
- `{SENSOR_UPPER}` is the `sensor_id` uppercased
- `{CRED_UPPER}` is the credential name uppercased with hyphens replaced by underscores

Example for `org_slug = "my-client"`, sensor `claroty`, credential `bearer_token`:

```bash
export PRISM_CLIENTS_MY_CLIENT_SENSORS_CLAROTY_BEARER_TOKEN="<token>"
```

A `_FILE` variant is also supported — set it to the path of a file containing the token:

```bash
export PRISM_CLIENTS_MY_CLIENT_SENSORS_CLAROTY_BEARER_TOKEN_FILE=/run/secrets/claroty_token
```

**AD-017**: Do not place actual credential values in this documentation, in AI prompts,
in commit messages, or anywhere they might be logged or replayed.

---

## 9. Onboard a Claroty xDome Client

Each Claroty xDome tenant requires a **customer overlay TOML file** that supplies the
tenant-specific API base URL. This overrides the `base_url = "${env.CLAROTY_INSTANCE_URL}"`
placeholder in the base sensor spec.

### Step 1 — Create the overlay directory

```bash
mkdir -p /etc/prism/specs/customers/my-client
```

The directory name must match the `org_slug` declared in `prism.toml`.

### Step 2 — Create the overlay TOML

Create `/etc/prism/specs/customers/my-client/claroty.sensor.toml` with the following
content (replace the `base_url` value with your actual Claroty xDome API endpoint):

```toml
# Per-org overlay for claroty sensor — my-client
extends     = "claroty"
instance_id = "claroty@my-client"
base_url    = "https://your-tenant.claroty.com"
```

**Do not put credentials in this file.** The `base_url` is the API host URL only;
the bearer token is managed by `prism credential set` (§8).

### Step 3 — Set the boot env var

The base sensor spec uses `base_url = "${env.CLAROTY_INSTANCE_URL}"`. Prism resolves
this env-var token at boot step 4a, before it loads per-org overlays at step 4c.
Even though the customer overlay overrides `base_url`, the env var must be set to a
non-empty value to satisfy the boot resolver. A placeholder value is acceptable:

```bash
export CLAROTY_INSTANCE_URL="http://127.0.0.1"
```

Add this export to the environment where `prism start` runs (shell profile, systemd
unit `Environment=`, Docker `ENV`, or the `env` block in `.mcp.json`). The actual
API endpoint comes from the customer overlay at step 4c — the env var value is never
used for HTTP requests against a real tenant when a customer overlay is present.

### Step 4 — Set credentials

Run `prism credential set` as described in §8 for this org:

```bash
prism credential set \
  --sensor claroty \
  --name bearer_token \
  --org-slug my-client \
  --config-dir /etc/prism
```

### Resulting config directory layout

```
/etc/prism/
  prism.toml
  specs/
    claroty.sensor.toml
    customers/
      my-client/
        claroty.sensor.toml   ← tenant overlay with base_url
  state/
    credential_index.json     ← written by prism credential set
```

---

## 10. First Boot

### Validate config before starting

```bash
prism validate-config --config-dir /etc/prism
```

This runs Prism's boot steps 1–7 (configuration loading, spec parsing, org resolution,
sensor spec validation) and exits 0 if everything is correct, or exits 2 with a
diagnostic message describing the first failure. Run this before `prism start` to
catch configuration errors without binding the MCP server.

Common exit-2 causes and fixes:

| Message fragment | Likely cause | Fix |
|-----------------|--------------|-----|
| `spec_dir does not exist` | `spec_dir` path wrong or missing | Create the directory and place sensor TOMLs there |
| `must be a UUID v7` | `org_id` is UUID v4 or v1 | Regenerate with `uuidgen -7` or `pip install uuid6` (§7) |
| `CLAROTY_INSTANCE_URL` not set | Env var absent | `export CLAROTY_INSTANCE_URL=http://127.0.0.1` |
| `no tables found for sensor` | Sensor spec not in `spec_dir` | Download the TOML and place it at `spec_dir` |

### Start the MCP server

```bash
prism start --config-dir /etc/prism
```

Or use the `PRISM_CONFIG_DIR` environment variable instead of `--config-dir`:

```bash
export PRISM_CONFIG_DIR=/etc/prism
prism start
```

`prism start` blocks and serves on stdio (it is an MCP stdio server). It does not bind
a TCP port. Normal operation: the process stays running until stdin closes or SIGTERM
arrives. Structured log output goes to stderr.

Expected startup log lines (structured JSON or text depending on `RUST_LOG` format):

```
boot.step1.tracing_init   ... done
boot.step2.config_loaded  spec_dir=/etc/prism/specs ...
boot.step3.org_resolved   org_slug=my-client ...
boot.step4.sensor_specs   tables=14 sensor=claroty ...
boot.step9.mcp_ready      ...
```

If `boot.step9.mcp_ready` appears, the server is accepting MCP messages on stdio.

---

## 11. Wire Prism into Claude Code

Prism exposes an MCP stdio server — Claude Code connects to it by spawning the `prism start`
process and communicating over its stdin/stdout.

### Option A — Project-local `.mcp.json`

Create `.mcp.json` at the root of the Claude Code project directory (or any parent):

```json
{
  "mcpServers": {
    "prism": {
      "command": "/usr/local/bin/prism",
      "args": ["--config-dir", "/etc/prism", "start"],
      "env": {
        "CLAROTY_INSTANCE_URL": "http://127.0.0.1"
      }
    }
  }
}
```

Replace `/usr/local/bin/prism` with the actual install path (check with `which prism`).
Replace `/etc/prism` with your config directory.

The `CLAROTY_INSTANCE_URL` env var must be present so the boot resolver can start up —
the customer overlay overrides the actual value before any HTTP call is made (see §9).

### Option B — Global `~/.claude/settings.json`

Add to the `mcpServers` key in `~/.claude/settings.json` (create the key if absent):

```json
{
  "mcpServers": {
    "prism": {
      "command": "/usr/local/bin/prism",
      "args": ["--config-dir", "/etc/prism", "start"],
      "env": {
        "CLAROTY_INSTANCE_URL": "http://127.0.0.1"
      }
    }
  }
}
```

The global settings file applies to all Claude Code sessions. Use this if you want Prism
available in every project workspace.

### Windows path

```json
{
  "mcpServers": {
    "prism": {
      "command": "C:\\Users\\YOU\\AppData\\Local\\prism\\bin\\prism.exe",
      "args": ["--config-dir", "C:\\prism", "start"],
      "env": {
        "CLAROTY_INSTANCE_URL": "http://127.0.0.1"
      }
    }
  }
}
```

### Verify the MCP connection

In Claude Code, run:

```
/mcp
```

The output should list `prism` as a connected server. If it shows `failed` or `timeout`,
check:
1. The `command` path is correct (`which prism` to confirm)
2. `prism validate-config --config-dir /etc/prism` exits 0
3. `CLAROTY_INSTANCE_URL` is set in the `env` block

---

## 12. First Smoke Query

With Claude Code connected to the `prism` MCP server, run these queries to confirm
end-to-end connectivity:

### Check sensor health

Ask Claude Code:

> "Run check_sensor_health for the my-client client."

Expected: `overall_status: "healthy"` with `reachable: true` and `auth_valid: true`
for the claroty sensor.

If `overall_status: "unhealthy"` with `E-SENSOR-030`:

- The bearer token is wrong or expired — re-run `prism credential set`
- The `base_url` in the customer overlay is unreachable — check network connectivity

### List all available tables

Ask Claude Code:

> "Run prism_describe for the my-client client."

Expected: 14 Claroty xDome tables listed under `results.tables`.

### Run a live query

Ask Claude Code (or use the MCP `query` tool directly):

> "Query claroty_devices for the my-client client, limit 3 rows."

Equivalent PrismQL: `FROM claroty_devices | limit 3`

Expected: 3 rows with `_source_type: "live"`, `class_uid` present, `raw_extensions` present.

If rows return with `_source_type: "live"`, setup is complete.

---

## 13. Next Steps

| Resource | Description |
|----------|-------------|
| `docs/DEMO-RUNBOOK.md` | Full demo execution guide with PrismQL query examples, multi-table SOC workflows, and troubleshooting |
| `RELEASING.md` | Release procedure for maintainers — version bumps, tagging, CI gates |
| `prism.toml.example` | Annotated config template with all supported fields |
| `CHANGELOG.md` | Full change log for v1.0.0-rc.1 |

### Supported sensors in v1.0.0-rc.1

Only **Claroty xDome** is fully validated in this release (14 tables, live-tenant tested
2026-09-04). CrowdStrike Falcon, Cyberint, and Armis sensor code is present in the binary
but not validated against live tenants; those sensors return in v1.0.0.

### Multi-tenant setup

Add additional `[[orgs]]` entries to `prism.toml`, create a customer overlay directory
for each org, and run `prism credential set` once per org. The `--org-slug` flag selects
which org's credential entry to write.

### Enrichment (not available in rc.1)

The `| enrich` PrismQL operator and the enrichment engine are present in the binary.
No enrichment infusion specs (`*.infusion.toml`) or WASM enrichment plugins are bundled
in v1.0.0-rc.1. Attempting an enrich query returns E-QUERY-039 (UDF not found) with an
empty available-UDFs list — this is expected behavior, not a crash.
