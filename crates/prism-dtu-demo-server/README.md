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

The URL is resolved from the URL sidecar written by `start`; the harness must
be running.

---

## Quickstart launcher

`scripts/start-demo.sh` wraps the above commands with sensible defaults:

```bash
scripts/start-demo.sh                                          # plain HTTP
scripts/start-demo.sh --tls                                    # HTTPS
scripts/start-demo.sh --config configs/prism-demo.toml --deterministic-logging
```

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

Export `DEMO_FAKE_*` env vars (e.g. via `scripts/start-demo.sh`) so the `<NAME>`
tier resolves them. Credentials never transit the AI context (AI-opaque model).

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

| File                              | Purpose |
|-----------------------------------|---------|
| `.prism-dtu-demo-server.pid`      | PID of the running harness process; read by `stop` |
| `.prism-dtu-demo-server.urls.json`| Clone URL map; read by `configure` and `stop` |

Both files are written atomically (tmp + rename) and removed on clean shutdown.
