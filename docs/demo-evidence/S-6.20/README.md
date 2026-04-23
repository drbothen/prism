# S-6.20 Demo Artifacts — prism-dtu-demo-server

Per-AC VHS demo recordings for **S-6.20: prism-dtu-demo-server: Unified Multi-Clone
Demo Harness** (v1.7 CONVERGED, 13 ACs, 30 integration tests).

## Artifact Index

| File prefix | AC | Description | Medium |
|---|---|---|---|
| `AC-1-all-clones-start` | AC-1 | All 6 clones bind configured ports; URL table prints 6 entries | VHS (gif + webm) |
| `AC-2-crowdstrike-fixture` | AC-2 | `GET /detects/queries/detects/v1` returns fixture via CrowdStrike clone | VHS (gif + webm) |
| `AC-3-configure-endpoint` | AC-3 | `POST /dtu/configure` accepted on CrowdStrike (L4) and cyberint (L2) | VHS (gif + webm) |
| `AC-4-tls-mode` | AC-4 | TLS cert generation + SHA-256 fingerprint (library-level via `axum_server::bind_rustls`). NOTE: the `--tls` CLI flag generates the cert but does NOT yet wire it into each clone's `start_on` — TLS-serving from the binary is deferred to Wave 2 per TD-WV1-04. | VHS (gif + webm) |
| `AC-5-graceful-shutdown` | AC-5 | SIGINT within 5s; all clones stop, ports released, no listener leak | VHS (gif + webm) |
| `AC-6-prism-demo-toml` | AC-6 | `prism-demo.toml` loaded; all 6 clone ports + bare-name cred refs verified | VHS (gif + webm) |
| `AC-7-determinism` | AC-7 | Two runs with seed=42 yield byte-identical fixture bodies for same request | VHS (gif + webm) |
| `AC-8-feature-gate` | AC-8 | Without `--features dtu`: binary skipped (required-features); with flag: builds | VHS (gif + webm) |
| `AC-9-bind-security` | AC-9 | Loopback always allowed; non-loopback rejected without `--bind-any` + env var | VHS (gif + webm) |
| `AC-10-health-endpoints` | AC-10 | `GET /dtu/health` returns 200 on all 6 clone ports | VHS (gif + webm) |
| `AC-11-partial-startup-cleanup` | AC-11 | Bind collision on clone 4 (armis): abort, 3 clones cleaned up, no port leak | VHS (gif + webm) |
| `AC-12-continue-on-error` | AC-12 | Same collision but `continue_on_error=true`: armis skipped, 5 clones serve | VHS (gif + webm) |
| `AC-13-start-report-three-states` | AC-13 | All three `StartReport` shapes observable: all-success, abort, partial-success | VHS (gif + webm) |
| `E2E-aggregate-all-acs` | all 13 | Full 30-test suite in one run (`--features dtu,tls`), all green | VHS (gif + webm) |
| `E2E-binary-cli` | capstone | Real release binary: `start` (6 clones, URL table, PID file) + health checks + `stop` (graceful shutdown, PID file removed) | VHS (gif + webm) |

## Demo Medium

Demos #1–14 are VHS terminal recordings driven by integration tests (`cargo test --test <ac-file>`).
They exercise the library API directly (`DemoHarness`, `build_clone_pairs`, `start_on`,
`stop_all`, `print_url_table`, `StartReport`).

**Binary CLI capstone** (`E2E-binary-cli`): recorded after commits e82a1764 + 48d472ed fully
wired the `start` and `stop` subcommands in `main.rs`. This tape runs the actual
`./target/release/prism-dtu-demo-server` binary end-to-end — no library API, no test
harness — and is the canonical evidence that the CLI integration is complete.

## Build Requirements

```sh
# Build binary (required before recording)
cargo build -p prism-dtu-demo-server --features dtu,tls --release

# Run full test suite (all 30 tests, 13 ACs)
cargo test -p prism-dtu-demo-server --features dtu,tls
```

## Port Table (stable across recordings — AC-7/AC-8)

| Clone | Port |
|---|---|
| crowdstrike | 17080 |
| claroty | 17081 |
| cyberint | 17082 |
| armis | 17083 |
| threatintel | 17084 |
| nvd | 17085 |
