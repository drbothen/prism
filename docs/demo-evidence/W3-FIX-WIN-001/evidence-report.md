# Demo Evidence Report — W3-FIX-WIN-001

**Story:** W3-FIX-WIN-001 — prism-dtu-harness: cross-platform fix for drop_releases_ports test (Windows winsock)
**Implementation commit:** `18963c65`
**Branch:** `fix/W3-FIX-WIN-001`
**Date recorded:** 2026-04-30
**Platform:** macOS (darwin-arm64)
**Behavioral contract:** BC-3.5.001 postcondition 4 ("drop releases ports")
**Verification property:** VP-124 / TV-6

---

## Coverage Map

| AC | Title | Recording | Result |
|----|-------|-----------|--------|
| AC-001 | drop_releases_ports passes on macOS | [AC-001-drop-releases-ports-macos-pass.gif](AC-001-drop-releases-ports-macos-pass.gif) / [.webm](AC-001-drop-releases-ports-macos-pass.webm) | PASS — 1/1 |
| AC-002 | Full prism-dtu-harness suite 70/70 regression safe | [AC-002-harness-suite-regression-safe.gif](AC-002-harness-suite-regression-safe.gif) / [.webm](AC-002-harness-suite-regression-safe.webm) | PASS — 70/70 |
| AC-003 | Diff visualization (bind-after-drop vs connect-refused) | Embedded below | N/A |
| AC-004 | BC-3.5.001 postcondition 4 traceability | Embedded below | N/A |

---

## AC-001: drop_releases_ports passes on macOS

**Recording:** `AC-001-drop-releases-ports-macos-pass.{gif,webm}`
**Tape:** `AC-001-drop-releases-ports-macos-pass.tape`
**Traces to:** BC-3.5.001 postcondition 4 / VP-124 / TV-6

The recording shows `cargo test -p prism-dtu-harness --features dtu --test logical_isolation_test test_BC_3_5_001_drop_releases_ports -- --nocapture` completing with:

```
running 1 test
test test_BC_3_5_001_drop_releases_ports ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 33 filtered out; finished in 0.10s
```

---

## AC-002: Full prism-dtu-harness suite 70/70 regression safe

**Recording:** `AC-002-harness-suite-regression-safe.{gif,webm}`
**Tape:** `AC-002-harness-suite-regression-safe.tape`
**Traces to:** BC-3.5.001 (all postconditions and invariants) / VP-122, VP-123, VP-124, VP-125

The recording shows `cargo test -p prism-dtu-harness --features dtu` completing with all 70 tests passing. No regressions introduced by the cross-platform port-release fix.

---

## AC-003: Diff Visualization — bind-after-drop vs connect-refused

This is the code change in `crates/prism-dtu-harness/tests/logical_isolation_test.rs` from commit `18963c65`.

### BEFORE (connect-refused pattern — Windows-incompatible)

```rust
/// TV-6: After `drop(harness)`, all clone ports are released.
///
/// Connects to the recorded address after drop and expects `ConnectionRefused`.
/// (BC-3.5.001 postcondition 4; Invariant 4; AC-004; VP-124)
#[tokio::test]
async fn test_BC_3_5_001_drop_releases_ports() {
    // ... harness setup and drop ...

    // Give OS a moment to release the port (should be immediate on drop)
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let connect_result = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        tokio::net::TcpStream::connect(addr),
    )
    .await;

    match connect_result {
        Ok(Ok(_)) => {
            panic!("TCP connection to {addr} succeeded after drop — port was not released (TV-6)")
        }
        Ok(Err(e)) => {
            assert_eq!(
                e.kind(),
                std::io::ErrorKind::ConnectionRefused,
                "expected ConnectionRefused after drop; got: {e} (TV-6; VP-124)"
            );
        }
        Err(_timeout) => {
            panic!("TCP connect to {addr} did not resolve within 1s after drop (TV-6; VP-124)")
        }
    }
}
```

**Why this fails on Windows:** POSIX stacks (`ECONNREFUSED`) return `ConnectionRefused` immediately when no listener is present. Windows winsock can hang for several seconds before producing `WSAECONNREFUSED`, causing the 1s `tokio::time::timeout` to fire — resulting in the `TCP connect to ... did not resolve within 1s` panic on every Windows CI run.

### AFTER (bind-after-drop pattern — cross-platform)

```rust
/// TV-6: After `drop(harness)`, all clone ports are released.
///
/// Uses a rebind-success assertion instead of connect-refused to remain
/// cross-platform: on Linux/macOS `TcpStream::connect` to a port with no
/// listener returns `ConnectionRefused` promptly, but on Windows (winsock)
/// the same call can hang for several seconds before timing out.  Attempting
/// to `TcpListener::bind` the same address is semantically equivalent — it
/// succeeds iff the port is truly free — and behaves identically across all
/// tier-1 platforms without any async timeout dance.
///
/// (BC-3.5.001 postcondition 4; Invariant 4; AC-004; VP-124)
#[tokio::test]
async fn test_BC_3_5_001_drop_releases_ports() {
    // ... harness setup and drop ...

    // Give OS a moment to release the port (should be immediate on drop)
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Rebind to the same address — succeeds only if the port was truly released.
    // This is the cross-platform equivalent of expecting ConnectionRefused on
    // connect: bind succeeds on Linux, macOS, and Windows once the listener is
    // gone, whereas connect-refused can time out on winsock.  (TV-6; VP-124)
    match std::net::TcpListener::bind(addr) {
        Ok(_listener) => {
            // Port was released; new listener dropped immediately.
            // TV-6 / VP-124 satisfied.
        }
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            panic!(
                "port {} still bound after harness drop — port leaked (TV-6; VP-124)",
                addr.port()
            );
        }
        Err(e) => {
            panic!("unexpected bind error after drop on {addr}: {e}");
        }
    }
}
```

**Why this works on all platforms:** `TcpListener::bind` returns `AddrInUse` consistently across POSIX (Linux/macOS) and winsock when the port is still held. No timeout dance required. `std::net::TcpListener` is in `std` — no new dependencies. The semantic invariant is identical: "if bind succeeds, the OS released the port."

---

## AC-004: BC-3.5.001 Postcondition 4 Traceability

**Traces to:** BC-3.5.001 / TV-6 / VP-124
**Anchor subsystem:** SS-01 (prism-dtu-harness)

### Behavioral Contract Clause

**BC-3.5.001 Postcondition 4:** After `drop(harness)`, all OS-level TCP ports that were bound by the harness are released. No port leakage occurs after the harness is dropped.

This is designated as **TV-6** (Test Vector 6) in the BC and tied to **VP-124** (Verification Property 124).

### Old Assertion Semantics

The previous assertion verified BC-3.5.001 postcondition 4 by attempting `TcpStream::connect(addr)` and expecting `ConnectionRefused`. The semantic reasoning was: if the port has no listener, a connect attempt returns `ConnectionRefused`. While this is correct on POSIX, it is **not guaranteed on Windows** — winsock may delay or silently drop the SYN without returning `WSAECONNREFUSED` promptly.

**Platform coverage:** Linux (PASS), macOS (PASS), Windows (FAIL — 6 consecutive CI failures, all at line 241 of `logical_isolation_test.rs`).

### New Assertion Semantics

The replacement assertion verifies BC-3.5.001 postcondition 4 by attempting `std::net::TcpListener::bind(addr)`. The semantic reasoning is: if the port is free (was released by `drop`), `bind` succeeds; if the port is still held (`AddrInUse`), `bind` fails and the test panics. This is **semantically equivalent** — both assertions prove the OS released the port — but the `bind` mechanism has uniform error semantics across all 6 CI matrix legs (linux-gnu, linux-musl, darwin-x86_64, darwin-arm64, windows-msvc, no-default-features).

**Platform coverage:** Linux (PASS), macOS (PASS — demonstrated by AC-001), Windows (expected PASS — verified post-PR by CI; `AddrInUse` is consistent across winsock and POSIX stacks when a port is still bound).

### Traceability Comments Preserved

Both `TV-6`, `BC-3.5.001 postcondition 4`, and `VP-124` appear in the updated test body and doc comment, preserving the full traceability chain from spec to code.

---

## Files in This Evidence Package

| File | Purpose |
|------|---------|
| `AC-001-drop-releases-ports-macos-pass.tape` | VHS tape source for AC-001 |
| `AC-001-drop-releases-ports-macos-pass.gif` | GIF recording — AC-001 pass |
| `AC-001-drop-releases-ports-macos-pass.webm` | WebM recording — AC-001 pass |
| `AC-002-harness-suite-regression-safe.tape` | VHS tape source for AC-002 |
| `AC-002-harness-suite-regression-safe.gif` | GIF recording — AC-002 70/70 pass |
| `AC-002-harness-suite-regression-safe.webm` | WebM recording — AC-002 70/70 pass |
| `run-ac001.sh` | Helper script invoked by AC-001 tape |
| `run-ac002.sh` | Helper script invoked by AC-002 tape |
| `evidence-report.md` | This file |
