# Security Review Findings — W3-FIX-WIN-001

**PR:** #105
**Branch:** fix/W3-FIX-WIN-001
**Reviewer:** security-reviewer (pr-manager inline, claude-sonnet-4-6)
**Agent ID:** pr-manager-step4-dispatch-2026-04-30T16:35Z
**Date:** 2026-04-30
**Commits reviewed:**
- `18963c65` — crates/prism-dtu-harness/tests/logical_isolation_test.rs (39 lines changed)
- `02c3e991` — docs/demo-evidence/W3-FIX-WIN-001/ (evidence files, shell scripts)
- `ae4cb896` — crates/prism-dtu-harness/tests/network_isolation_test.rs (51 lines changed)
- `e3f4abfa` — Cargo.lock only (wasmtime 44.0.0 → 44.0.1, RUSTSEC-2026-0114 fix)

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |

**Verdict: CLEAN — no security findings across all 4 commits.**

---

## Scope

This PR modifies two test files, demo evidence, and the Cargo.lock:
- `crates/prism-dtu-harness/tests/logical_isolation_test.rs` — bind-after-drop fix (BC-3.5.001 postcondition 4)
- `crates/prism-dtu-harness/tests/network_isolation_test.rs` — sibling bind-after-drop fix (BC-3.5.002 postcondition 6)
- `docs/demo-evidence/W3-FIX-WIN-001/` — VHS recordings and shell scripts (read-only documentation)
- `Cargo.lock` — wasmtime 44.0.0 → 44.0.1 (RUSTSEC-2026-0114 remediation + cranelift-* transitive bumps)

No production source is touched. No new Cargo.toml dependencies. No unsafe code added.

---

## OWASP Top 10 Checklist

| Category | Finding |
|----------|---------|
| A01 Broken Access Control | N/A — no access control code |
| A02 Cryptographic Failures | N/A — no crypto |
| A03 Injection | CLEAN — `bind(addr)` takes typed `SocketAddr`, not a string; no shell injection |
| A04 Insecure Design | CLEAN — rebind pattern is well-defined; no timeout race; semantically equivalent to original |
| A05 Security Misconfiguration | CLEAN — `std::net::TcpListener` does not set `SO_REUSEADDR` on Windows by default (EC-004 correctly handled) |
| A06 Vulnerable Components | CLEAN — wasmtime upgraded FROM vulnerable (44.0.0) TO fixed (44.0.1); net improvement |
| A07 Auth Failures | N/A — no auth code |
| A08 Software Integrity | CLEAN — evidence files are documentation only; no executable artifacts in evidence dir |
| A09 Logging Failures | N/A — test panic messages include only typed SocketAddr/port values; no sensitive data |
| A10 SSRF | N/A — loopback only (`127.0.0.1`); addresses are harness-assigned ephemeral ports, not user-controlled |

---

## Detailed Review

### logical_isolation_test.rs (18963c65)

**`std::net::TcpListener::bind(addr)`:**
- `addr` is a `std::net::SocketAddr` produced by the test harness (OS-assigned ephemeral port on loopback). It is never derived from user input or external data.
- No injection surface. The `bind` call is a typed API, not a string shell command.
- The `Ok(_listener)` arm drops the listener immediately — no resource leak.
- Panic messages in `AddrInUse` and `Err(e)` arms expose only the port number and an `std::io::Error` message — both are test-internal values.
- The 100ms `tokio::time::sleep` before bind is defensive padding — no correctness dependency. No security impact.

**Assessment: CLEAN**

### network_isolation_test.rs (ae4cb896)

- Identical bind-after-drop pattern as logical_isolation_test.rs. Same security posture.
- The removed `tokio::net::TcpListener::bind` check was asserting the same invariant twice. Removal is correct and introduces no security regression.
- Panic message in `Err(e)` arm includes TV-6/AC-005 traceability tags — no sensitive data.

**Assessment: CLEAN**

### Cargo.lock (e3f4abfa)

**wasmtime 44.0.0 → 44.0.1:**
- RUSTSEC-2026-0114 (panic on table allocation exceeding address space) is **resolved** by this bump. This is a security improvement, not a regression.
- Advisory: https://rustsec.org/advisories/RUSTSEC-2026-0114
- wasmtime 44.0.1 was released by the Bytecode Alliance specifically to address this advisory.

**cranelift-* 0.131.0 → 0.131.1:**
- Transitive dependency of wasmtime. Patch bumps from crates.io with updated checksums. No new crates introduced. No known advisories against 0.131.1.

**windows-sys version changes:**
- Some dependencies downgrade windows-sys from 0.61.2 to 0.60.2. This is not a security regression — windows-sys is a thin FFI binding library with no known advisories. The version change reflects wasmtime 44.0.1's transitive requirements vs 44.0.0.

**itertools 0.14.0 → 0.13.0 (in prost-build):**
- Downgrade in a build-tool transitive. No known advisories. No runtime impact on the Prism binary.

**Assessment: NET SECURITY IMPROVEMENT (RUSTSEC-2026-0114 resolved)**

### Shell Scripts (run-ac001.sh, run-ac002.sh)

- `cargo test -p prism-dtu-harness --features dtu --test logical_isolation_test ...` with hardcoded flags.
- No user input, no `eval`, no dynamic argument construction, no credential references.
- Located in `docs/demo-evidence/` — only used during local demo recording, not in CI or production.

**Assessment: CLEAN**

---

## Dependency Audit Summary

- No new external Cargo dependencies introduced.
- `std::net::TcpListener` is Rust stdlib — no Cargo entry.
- RUSTSEC-2026-0114 (wasmtime 44.0.0) is **fixed** by this PR's Cargo.lock bump.
- `cargo deny check` and `cargo audit` will pass after this bump (as confirmed by the successful pre-push hook run).

---

## Conclusion

**APPROVE — zero security findings across all 4 commits.**

| Commit | Assessment |
|--------|-----------|
| `18963c65` | CLEAN — stdlib bind, no injection, loopback only |
| `02c3e991` | CLEAN — documentation/evidence files only |
| `ae4cb896` | CLEAN — identical bind pattern, same posture |
| `e3f4abfa` | NET IMPROVEMENT — RUSTSEC-2026-0114 resolved |

Reviewer: security-reviewer (pr-manager inline dispatch), claude-sonnet-4-6
Agent ID: pr-manager-step4-dispatch-2026-04-30T16:35Z
