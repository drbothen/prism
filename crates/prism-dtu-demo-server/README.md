# prism-dtu-demo-server

Unified multi-clone demo harness that binds all 6 DTU clones on stable ports
for live demos and CI regression. Combines `seed = 42` with
`--deterministic-logging` to make response bodies reproducible across runs for
the same request sequence (AC-7).

Spec: `.factory/stories/S-6.20-dtu-demo-server.md`
ADR: `.factory/specs/architecture/decisions/ADR-002-l2-dtu-clone-template.md`

---

## Clones included

| Clone        | Fidelity | Default port |
|--------------|----------|--------------|
| CrowdStrike  | L4       | 17080        |
| Claroty      | L4       | 17081        |
| Cyberint     | L2       | 17082        |
| Armis        | L2       | 17083        |
| ThreatIntel  | L2       | 17084        |
| NVD          | L2       | 17085        |

Fidelity taxonomy per ADR-002 + Amendments.

---

## Build

```bash
# Plain HTTP (minimum required feature)
cargo build --release -p prism-dtu-demo-server --features dtu

# HTTPS support (adds TLS feature; self-signed cert generated at runtime)
cargo build --release -p prism-dtu-demo-server --features dtu,tls
```

---

## Run

### Start (plain HTTP)

```bash
prism-dtu-demo-server start --config configs/demo.toml
```

### Start (HTTPS — requires `tls` feature)

```bash
prism-dtu-demo-server start --config configs/demo.toml --tls
```

The binary prints the self-signed certificate's SHA-256 fingerprint to stdout
**before** the URL table. Stakeholders should pin this value:

```
sha256:<hex>   ← pin this fingerprint
clone crowdstrike => https://127.0.0.1:17080
...
```

### Stop

Sends SIGTERM to the backgrounded harness via the PID file:

```bash
prism-dtu-demo-server stop
```

### Configure a clone at runtime

Forwards a JSON payload to a clone's `/dtu/configure` endpoint:

```bash
prism-dtu-demo-server configure crowdstrike '{"failure_mode":"Timeout"}'
```

The URL is resolved from whichever URL sidecar is present:
`.prism-dtu-demo-server.urls.json` (flat, written by `start`) or
`.prism-dtu-demo-server.urls-multi.json` (nested, written by `start-multi`).
The admin token is resolved from the corresponding token sidecar
(`.prism-dtu-demo-server.admin-tokens.json` for `start`,
`.prism-dtu-demo-server.admin-tokens-multi.json` for `start-multi`). Both
the URL sidecar and the token sidecar must be present — if the token sidecar
is absent the subcommand exits with error code E-DEMO-007. The harness must be
running.

---

## Quickstart launcher

### Single-org flat model (ports 17080–17085, plain HTTP)

Use `configs/demo.toml` with the `start` subcommand. Export `DEMO_FAKE_*` env
vars so Prism can resolve the `credential_ref` values in `configs/prism-demo.toml`.

```bash
# 1. Start all 6 DTU clones on fixed ports 17080–17085
prism-dtu-demo-server start --config configs/demo.toml

# 2. Export fake credential tokens (in a separate shell where you'll run prism)
export DEMO_FAKE_CROWDSTRIKE_TOKEN=dtu-fake-cs-token
export DEMO_FAKE_CLAROTY_TOKEN=dtu-fake-claroty-token
export DEMO_FAKE_CYBERINT_TOKEN=dtu-fake-cyberint-token
export DEMO_FAKE_ARMIS_TOKEN=dtu-fake-armis-token
export DEMO_FAKE_THREATINTEL_TOKEN=dtu-fake-ti-token
export DEMO_FAKE_NVD_TOKEN=dtu-fake-nvd-token

# 3. Run Prism against the DTU harness
prism start --config crates/prism-dtu-demo-server/configs/prism-demo.toml

# 4. Tear down clones
prism-dtu-demo-server stop
```

### Multi-org model (ephemeral OS ports, 3 orgs × N sensors)

Use `scripts/demo.toml` with the `start-multi` subcommand (requires the
`fixture-gen` feature; `demo-setup.sh` seeds the per-org keyring credentials).

`scripts/demo-run.sh` launches `start-multi` in the background automatically —
do not run `start-multi` manually before calling `demo-run.sh`. Running both
would start two competing fleets on separate ephemeral ports (PID confusion,
stale sidecar). The authoritative sequence is:

```bash
# 1. Bootstrap keyring credentials and build binaries (one-time setup)
scripts/demo-setup.sh

# 2. Start DTU clones + generate per-org overlays (start-multi runs inside this script)
scripts/demo-run.sh

# 3. Tear down clones and clean up credentials
scripts/demo-teardown.sh
```

To run `start-multi` directly without the overlay generation that `demo-run.sh`
provides, use it as a standalone command (bare fleet, no overlay writes):

```bash
prism-dtu-demo-server start-multi --config scripts/demo.toml
```

Do not combine the standalone command with `demo-run.sh` — use one or the other.

---

## CLI flags

| Flag                       | Description |
|----------------------------|-------------|
| `--config <PATH>`          | Path to the demo TOML config (required) |
| `--tls`                    | Enable HTTPS; generates a self-signed cert and prints the SHA-256 fingerprint. Requires the `tls` feature. |
| `--bind-any`               | Allow non-loopback binding (R-DEMO-001 two-factor gate; also requires `PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK`) |
| `--deterministic-logging`  | Suppress timestamps, PIDs, and request IDs from log output for AC-7 determinism |

---

## Config files

### `configs/demo.toml` — canonical demo preset

All 6 clones on ports 17080–17085 with `seed = 42` and loopback binding.
Use for local demos and recorded walkthroughs.

### `configs/prism-demo.toml` — Prism production preset

Routes Prism sensor queries through the demo harness. Uses bare-name
`credential_ref` values per S-5.05 Task 3 / BC-2.03.009. Resolution chain:
`<NAME>_FILE` env var → `<NAME>` env var → keyring.

Before running `prism start` with this config, export the six `DEMO_FAKE_*`
env vars so the `<NAME>` env-var tier of the resolution chain resolves the
`credential_ref` values below. These are fake tokens for the DTU harness —
they have no real credential value:

```bash
export DEMO_FAKE_CROWDSTRIKE_TOKEN=dtu-fake-cs-token
export DEMO_FAKE_CLAROTY_TOKEN=dtu-fake-claroty-token
export DEMO_FAKE_CYBERINT_TOKEN=dtu-fake-cyberint-token
export DEMO_FAKE_ARMIS_TOKEN=dtu-fake-armis-token
export DEMO_FAKE_THREATINTEL_TOKEN=dtu-fake-ti-token
export DEMO_FAKE_NVD_TOKEN=dtu-fake-nvd-token
```

Any value you export before the above takes precedence. Credentials never
transit the AI context (AI-opaque model). Note: `scripts/demo-setup.sh` seeds
per-org keyring credentials for the multi-org model (org-a/org-b/org-c), not
the `DEMO_FAKE_*` flat-model tokens used here.

---

## Security model

- **Loopback-only by default.** All clones bind `127.0.0.1`; no network
  exposure without explicit opt-in.
- **R-DEMO-001 two-factor gate for non-loopback.** Both `--bind-any` AND
  `PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK` are required.
  Either alone is rejected.
- **TLS is self-signed.** DO NOT use beyond localhost demos or CI. The
  self-signed certificate is ephemeral (generated fresh each `start`).
- **TLS fingerprint verification.** The `sha256:<hex>` fingerprint is printed
  to stdout at startup (before the URL table). Stakeholders running the demo
  should verify this value matches what they copied from a trusted prior run.

---

## Files written to cwd

| File                                                   | Purpose |
|--------------------------------------------------------|---------|
| `.prism-dtu-demo-server.pid`                           | PID of the running harness process; read by `stop` |
| `.prism-dtu-demo-server.urls.json`                     | Flat URL map for the `start` model; read by `configure` |
| `.prism-dtu-demo-server.urls-multi.json`               | Nested URL map for the `start-multi` model; read by `configure` |
| `.prism-dtu-demo-server.admin-tokens.json`             | Admin token map for the flat (`start`) model; read by `configure` |
| `.prism-dtu-demo-server.admin-tokens-multi.json`       | Admin token map for the multi-org (`start-multi`) model; read by `configure` |

All files are written atomically (tmp + rename) and removed on clean shutdown.
