# Demo Evidence — S-6.09: prism-dtu-cyberint

**Story:** S-6.09 — DTU for Cyberint API, L2 (stateful)
**Branch:** feature/S-6.09-dtu-cyberint
**Tool:** VHS 0.10.0 (FiraCode Nerd Font Mono, Dracula theme)
**All tests pass:** `cargo test -p prism-dtu-cyberint --features dtu` (100% green)

---

## Coverage Matrix

| AC | Description | Result | Recording |
|----|-------------|--------|-----------|
| AC-1 | Cookie auth round-trip | PASS | [gif](AC-001-cookie-auth-roundtrip.gif) / [webm](AC-001-cookie-auth-roundtrip.webm) |
| AC-2 | Unauthenticated request returns 401 | PASS | [gif](AC-002-unauthenticated-401.gif) / [webm](AC-002-unauthenticated-401.webm) |
| AC-3 | Alert status transition persists (stateful) | PASS | [gif](AC-003-alert-status-transition.gif) / [webm](AC-003-alert-status-transition.webm) |
| AC-4 | Irreversible close enforced | PASS | [gif](AC-004-irreversible-close.gif) / [webm](AC-004-irreversible-close.webm) |
| AC-5 | Mixed timestamps in fixtures (ISO-8601 + Unix epoch) | PASS | [gif](AC-005-mixed-timestamps.gif) / [webm](AC-005-mixed-timestamps.webm) |
| AC-6 | Cursor pagination (page 1 + page 2) | PASS | [gif](AC-006-cursor-pagination.gif) / [webm](AC-006-cursor-pagination.webm) |
| AC-7 | Rate limit — HTTP 429 after threshold (E-SENSOR-003) | PASS | [gif](AC-007-rate-limit.gif) / [webm](AC-007-rate-limit.webm) |
| AC-8 | Reset semantics — reverts state, clears sessions | PASS | [gif](AC-008-reset-semantics.gif) / [webm](AC-008-reset-semantics.webm) |

**Coverage: 8/8 ACs. 0 non-demo-able ACs.**

---

## Demo Architecture

The DTU has no standalone CLI binary — it runs in-process via `BehavioralClone::start()`.
A thin demo server binary (`src/bin/demo_server.rs`, gated behind `--features dtu`) binds
the axum router on an ephemeral port and prints `READY http://127.0.0.1:<port>` to stdout.

Each tape runs a corresponding `.sh` driver script that:
1. Calls `start_dtu` (from `demo-lib.sh`): builds the binary if needed, starts it in the
   background, waits for the READY line, exports `BASE_URL` and `SERVER_PID`.
2. Executes the AC-specific `curl` sequence.
3. Calls `stop_dtu` via `trap EXIT`.

This pattern keeps the VHS tape simple (one `Type` line) while the shell script handles
all credential extraction, variable interpolation, and server lifecycle.

---

## Reproducing Locally

```bash
cd /path/to/prism/.worktrees/S-6.09-cyberint

# Run any single demo script directly:
bash docs/demo-evidence/S-6.09/AC-001-cookie-auth-roundtrip.sh

# Re-record all tapes (requires VHS >= 0.10.0):
for tape in docs/demo-evidence/S-6.09/*.tape; do vhs "$tape"; done

# Run full test suite (same behavior as demos):
cargo test -p prism-dtu-cyberint --features dtu
```

---

## File List

| File | Type | Purpose |
|------|------|---------|
| `demo-lib.sh` | Shell helper | Shared `start_dtu` / `stop_dtu` / `login` functions |
| `AC-001-cookie-auth-roundtrip.{sh,tape,gif,webm}` | AC-1 | Cookie auth round-trip |
| `AC-002-unauthenticated-401.{sh,tape,gif,webm}` | AC-2 | 401 on missing cookie |
| `AC-003-alert-status-transition.{sh,tape,gif,webm}` | AC-3 | Stateful PATCH persists |
| `AC-004-irreversible-close.{sh,tape,gif,webm}` | AC-4 | close is irreversible |
| `AC-005-mixed-timestamps.{sh,tape,gif,webm}` | AC-5 | ISO-8601 + epoch in fixtures |
| `AC-006-cursor-pagination.{sh,tape,gif,webm}` | AC-6 | Two-page cursor pagination |
| `AC-007-rate-limit.{sh,tape,gif,webm}` | AC-7 | HTTP 429 rate limit |
| `AC-008-reset-semantics.{sh,tape,gif,webm}` | AC-8 | Reset reverts all state |
| `README.md` | Report | This file |
