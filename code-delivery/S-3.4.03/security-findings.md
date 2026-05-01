# Security Review Findings — S-3.4.03

**PR:** #109
**Story:** Migrate prism-dtu-crowdstrike tests to prism-dtu-harness
**Review Date:** 2026-04-30
**Reviewer:** security-review skill (claude-sonnet-4-6)

## Summary

**CLEAN — No HIGH or MEDIUM findings.**

### Scope Analyzed

| File | Lines Changed | Classification |
|------|--------------|----------------|
| `crates/prism-dtu-crowdstrike/Cargo.toml` | +7 | Config (dev-dep addition) |
| `crates/prism-dtu-harness/Cargo.toml` | +3 | Config (lru, rand, rand_chacha to lib) |
| `crates/prism-dtu-harness/src/clones/crowdstrike.rs` | +1450 | New: harness clone router |
| `crates/prism-dtu-harness/src/clones/mod.rs` | +15 | New: module declaration |
| `crates/prism-dtu-harness/src/builder.rs` | +34 net | Dispatch branch additive |
| `crates/prism-dtu-harness/src/lib.rs` | +1 | Module re-export |
| `crates/prism-dtu-crowdstrike/tests/harness_tests.rs` | +2621 | Test-only |
| `docs/demo-evidence/S-3.4.03/` | Binary + md | Recordings only |

### Findings

| Severity | Count | Notes |
|----------|-------|-------|
| Critical | 0 | — |
| High | 0 | — |
| Medium | 0 | — |
| Low | 2 | Informational only; no action required |

---

## Detailed Analysis

### OWASP A01 — Broken Access Control

**Finding:** NONE

- `POST /dtu/configure` is guarded by `X-Admin-Token` header comparison (`provided != Some(state.admin_token.as_str())`). The admin token is generated per-instance as `uuid::Uuid::new_v4().to_string()` — unpredictable, ephemeral, not hardcoded.
- Bearer auth on API routes (`check_bearer_auth`) correctly rejects empty/missing tokens with HTTP 401.
- Network-mode bearer guard (`check_network_bearer`) correctly enforces per-org token binding: org_a's token presented to org_b's port returns 401.
- Admin-token comparison uses `!=` string comparison (not timing-safe). **Accepted:** this is a test harness, not production auth infrastructure. Tokens are 36-character UUIDs bound to ephemeral loopback ports; timing-attack surface is nil in CI context.

### OWASP A03 — Injection

**Finding:** NONE

- All ID generation is purely derived: `format!("det-{org_slug}-{seed}-{i:03}")`. Input is `org_slug` (builder-controlled, not user-controlled) and `seed` (u64, numeric).
- The `filter: Option<String>` field in `PaginationParams` is deserialized but never interpolated into any query or evaluated expression — it is currently dead code (`#[allow(dead_code)]`).
- `ConfigureBody` uses `#[serde(deny_unknown_fields)]` — rejects unexpected fields at the boundary.
- Custom `url_decode()` function: decodes `%XX` percent-encoding and `+` → space. The decoded value is used only as a string key in `HashMap` lookups and `Vec` pushes — never evaluated, never passed to shell, never interpolated into SQL. **No injection risk.**

### OWASP A07 — Identification and Authentication Failures

**Finding:** NONE

- `test_BC_3_5_002_ac_network_cross_creds_401` explicitly validates the 401 boundary (cross-org credential mismatch). This test provides observable evidence that the auth boundary functions correctly.
- `check_bearer_auth` correctly handles missing, empty, and non-Bearer authorization headers.
- Admin token is rotated per-instance (UUID v4), not shared across harness instances.

### OWASP A08 — Software and Data Integrity

**Finding:** NONE

- `lru`, `rand`, `rand_chacha` are added to `prism-dtu-harness` production dependencies (not dev-deps). All three are well-established crates with known security posture. `lru` is a pure data structure crate; `rand`/`rand_chacha` are the de facto standard for Rust CSPRNG.
- `Cargo.lock` is updated consistently — no supply-chain anomaly.

### OWASP A09 — Security Logging and Monitoring

**Finding:** LOW (informational)

- `POST /dtu/reset` and `POST /dtu/configure` do not emit any tracing spans. For a test harness this is acceptable; production code is unaffected.
- **Action required:** None (test-only infrastructure).

### Production Surface Area

**Finding:** NONE

- `prism-dtu-harness` is added to `prism-dtu-crowdstrike` as `[dev-dependencies]` only — confirmed by diff context. It does NOT appear in `[dependencies]`.
- Verification: `git diff` shows the addition is in the `[dev-dependencies]` section (line 36 of Cargo.toml diff).
- `AC-007` compliance: `harness_tests.rs` imports only `prism_dtu_common`, `prism_dtu_harness` — no `use prism_dtu_crowdstrike` import; `CrowdStrikeClone::start()` does not appear in the test file.

### LRU Capacity Bound

**Finding:** LOW (informational)

- `SESSION_REGISTRY_CAPACITY = 1_000` is a compile-time constant. `NonZeroUsize::new(1_000).expect(...)` is correct and safe at startup.
- In a test harness context with ephemeral instances, this capacity is never approached. No DoS risk in practice.
- **Action required:** None.

### `url_decode()` Custom Implementation

**Finding:** LOW (informational, already noted above)

- Hand-rolled percent-decoder is simple, bounded, and purely String → String with no side effects.
- Decoded value is used only as a HashMap key (session-id lookup and `ids` parameter extraction) — never interpreted as code, markup, or query language.
- **Action required:** None. Using `percent-encoding` crate would be cleaner but is not required for correctness or security.

### Mutex Poisoning Panic Risk

**Finding:** LOW (informational)

- All `Mutex::lock().expect("... poisoned")` calls will panic if a thread panics while holding the lock. In the test harness context, this is the correct behavior (crash propagation is the intended response). The crash monitor design relies on this panic propagating to the `crash_tx` channel.
- **Action required:** None (intended design, test infrastructure only).

---

## Conclusion

**APPROVED — CLEAN security posture.** This is a test-only migration. All new code is gated behind `[dev-dependencies]` or `#[cfg(feature = "dtu")]`. No production binary surface is introduced or modified. The four LOW findings are informational and do not require remediation. The PR correctly implements cross-org isolation boundaries and validates them with dedicated test cases.
