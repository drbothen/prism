# Demo Evidence Report — W3-FIX-CODE-002

| Field | Value |
|-------|-------|
| Story ID | W3-FIX-CODE-002 |
| Title | prism-customer-config + prism-dtu-harness: validation and dispatch hygiene bundle |
| Branch | feature/W3-FIX-CODE-002 |
| HEAD SHA | 85a453d4 |
| Recorded | 2026-05-01 |
| Recorder | Demo Recorder (Claude Code) |

---

## Findings Resolved

| Finding | Crate | Description | Severity |
|---------|-------|-------------|----------|
| CR-003 | prism-customer-config | OrgSlug regex pattern validated in structural pass; E-CFG-019 introduced | MEDIUM |
| CR-004 | prism-dtu-harness | `start_clone` sequential if-chains replaced with exhaustive `match` | MEDIUM |
| CR-005 | prism-customer-config | `validate_all` made `pub(crate)` — public surface is `load_and_validate` only | MEDIUM |
| CR-006 | prism-dtu-harness | `poll_test_hook` backoff (10ms spin replaced) | MEDIUM |
| SEC-006 | prism-customer-config | `sanitize_error_message` covers multi-line TOML credential values | MEDIUM |
| SEC-007 | prism-audit | `org_slug` cross-checked against `OrgRegistry::slug_for(org_id)` at write time | MEDIUM |

**6 MEDIUM findings resolved. E-CFG-019 (InvalidOrgSlugPattern) introduced as next sequential error code after E-CFG-018 (W3-FIX-SEC-003).**

---

## Coverage Map

| AC | Title | Recording | BC Trace | Tests | Result |
|----|-------|-----------|----------|-------|--------|
| AC-001 | OrgSlug pattern validated; E-CFG-019 for space/unicode/dot/length>64 | [AC-001-org-slug-pattern-validation.gif](AC-001-org-slug-pattern-validation.gif) | BC-3.3.004 R-CUST-002 | cr003_slug_pattern (10 tests) | PASS |
| AC-002 | `validate_all` made `pub(crate)` | Covered by AC-001 (compile-time guard: external callers compile without validate_all) | BC-3.3.004 Invariant 1 | cr003_slug_pattern: test_BC_3_3_004_CR003_load_and_validate_is_the_public_entry_point | PASS |
| AC-003 | `start_clone` dispatch exhaustive `match` | [AC-003-armis-network-dispatch-403.gif](AC-003-armis-network-dispatch-403.gif) | BC-3.5.001 postcondition 1 | cr004_dispatch_consolidation (9 tests) | PASS |
| AC-004 | `poll_test_hook` backoff (10ms → 50ms) | Covered by AC-003 test suite (hook fires correctly) | BC-3.5.001 startup budget | cr004_dispatch_consolidation suite | PASS |
| AC-005 | `sanitize_error_message` multi-line TOML credential redaction | [AC-005-toml-multiline-redaction.gif](AC-005-toml-multiline-redaction.gif) | BC-3.3.001 Invariant | sec006_toml_multiline_redaction (5 tests) | PASS |
| AC-006 | Audit `org_slug` cross-checked vs OrgRegistry at write time | [AC-006-org-slug-cross-check.gif](AC-006-org-slug-cross-check.gif) | BC-3.1.002 postcondition | sec007_org_slug_cross_check (7 tests) | PASS |
| AC-007 | Network-mode dispatch also correct (Armis returns 403) | Covered by AC-003 recording: `test_BC_3_5_002_CR004_armis_network_mode_dispatch_is_correct` | BC-3.5.002 postcondition 1 | cr004_dispatch_consolidation (included) | PASS |
| AC-008 | Each defect has a regression test | All 4 test files present: 10 + 5 + 9 + 7 = 31 tests | All above BCs | 31/31 pass | PASS |

---

## AC-001 / CR-003: OrgSlug Pattern Validation (E-CFG-019)

**Recording:** [AC-001-org-slug-pattern-validation.gif](AC-001-org-slug-pattern-validation.gif)
**Source tape:** [AC-001-org-slug-pattern-validation.tape](AC-001-org-slug-pattern-validation.tape)
**Archival:** [AC-001-org-slug-pattern-validation.webm](AC-001-org-slug-pattern-validation.webm)

**What is demonstrated:**

`validate_structural` now calls `OrgSlug::new` after the filename-stem check (R-CUST-002). Slugs
containing invalid characters or exceeding 64 characters emit `ConfigError::InvalidOrgSlugPattern`
(E-CFG-019). Valid slugs pass without error.

Rejection cases shown:
- `"my org"` — space character rejected (E-CFG-019)
- `"acmé"` — Unicode 'é' rejected (EC-003)
- `"my.org"` — dot character rejected (not in `[a-zA-Z0-9_-]`)
- 65-char slug — length exceeds 64 limit

Valid cases confirmed:
- `"a"` — single character (EC-001, min boundary)
- `"a".repeat(64)` — exactly 64 characters (EC-002, max boundary)
- `"acme-corp"`, `"acme_corp"` — hyphens and underscores accepted

AC-002 (validate_all visibility) is verified by the compile-time guard:
`prism_customer_config::validate_all` is not accessible as a public symbol from external crates.

**Test suite:** `crates/prism-customer-config/tests/cr003_slug_pattern.rs`

```
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

---

## AC-005 / SEC-006: Multi-line TOML Credential Redaction

**Recording:** [AC-005-toml-multiline-redaction.gif](AC-005-toml-multiline-redaction.gif)
**Source tape:** [AC-005-toml-multiline-redaction.tape](AC-005-toml-multiline-redaction.tape)
**Archival:** [AC-005-toml-multiline-redaction.webm](AC-005-toml-multiline-redaction.webm)

**What is demonstrated:**

`sanitize_error_message` now applies conservative multi-line redaction: when a TOML snippet
contains a credential-named field opening (`password = """`, `bearer_token = """`,
`api_secret = """`), all continuation lines until the closing `"""` are also redacted.
Non-credential fields (e.g., `display_name`) retain their multi-line values for diagnostics.

Redaction cases shown:
- `password = """\nmy-secret-value\nline2-of-secret\n"""` — both continuation lines absent from error message
- `bearer_token = """\nabc123\nmy-bearer-secret\n"""` — secrets absent from error message
- `api_secret = """\nsuper-secret\nsecond-secret-line\n"""` — secrets absent from error message

Non-redaction case confirmed:
- `display_name = """\nACME Corporation\n..."` — value retained (diagnostic information, not a credential)

Single-line regression guard:
- `password = "single-line-secret"` — still redacted (existing behavior preserved)

**Test suite:** `crates/prism-customer-config/tests/sec006_toml_multiline_redaction.rs`

```
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

---

## AC-003 + AC-007 / CR-004: Exhaustive Match Dispatch (Armis Network Mode 403)

**Recording:** [AC-003-armis-network-dispatch-403.gif](AC-003-armis-network-dispatch-403.gif)
**Source tape:** [AC-003-armis-network-dispatch-403.tape](AC-003-armis-network-dispatch-403.tape)
**Archival:** [AC-003-armis-network-dispatch-403.webm](AC-003-armis-network-dispatch-403.webm)

**What is demonstrated:**

Sequential `if dtu_type == Armis` / `if dtu_type == Claroty` chains in `clone_server.rs:start_clone`
and `build_network()` are replaced with an exhaustive `match dtu_type`. Adding a new `DtuType`
variant without updating the match produces a compile error.

Key scenario (the production gap test): `DtuType::Armis` in `IsolationMode::Network` now
dispatches to the Armis-specific router. The Armis router enforces Bearer token authentication;
an unauthenticated GET `/api/v1/devices` returns **HTTP 403** (not 200 from the generic stub).

All 7 DtuType variants covered:
- Claroty — `/assets/v1/assets` returns HTTP 200 with `"assets"` key
- Armis (Logical) — `/api/v1/devices` without Bearer returns HTTP 403
- Armis (Network) — `/api/v1/devices` without Bearer returns HTTP 403 (the fixed production gap)
- CrowdStrike (Logical + Network) — `/dtu/health` returns HTTP 200
- Cyberint — `/dtu/health` returns HTTP 200
- Slack — `/dtu/received-payloads` returns HTTP 200
- PagerDuty, Jira — `/dtu/health` returns HTTP 200

AC-004 (poll_test_hook 10ms → 50ms backoff) is exercised by the harness startup in this same
test suite — the test hook fires correctly under the 50ms polling cadence.

**Test suite:** `crates/prism-dtu-harness/tests/cr004_dispatch_consolidation.rs` (requires `--features dtu`)

```
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

---

## AC-006 / SEC-007: Audit org_slug Cross-Check

**Recording:** [AC-006-org-slug-cross-check.gif](AC-006-org-slug-cross-check.gif)
**Source tape:** [AC-006-org-slug-cross-check.tape](AC-006-org-slug-cross-check.tape)
**Archival:** [AC-006-org-slug-cross-check.webm](AC-006-org-slug-cross-check.webm)

**What is demonstrated:**

`prism_audit::org_slug_guard::validate_org_slug_cross_check(registry, entry)` is called at
audit record construction time. The function returns a `SlugCheckResult` enum:

| Scenario | SlugCheckResult | Production action |
|----------|-----------------|-------------------|
| `slug_for(org_id) == entry.org_slug` | `Matched` | No warning |
| `slug_for(org_id) != entry.org_slug` | `Mismatched { registry_slug }` | `tracing::warn!` |
| `slug_for(org_id)` returns `None` | `OrgNotInRegistry` | `tracing::warn!` |

Audit-must-not-fail semantics (BC-3.1.002): in all three cases the audit entry is fully
constructed and emitted. The cross-check never panics; it uses pattern-matching on the
`Option<OrgSlug>` returned by `slug_for` (no `unwrap()`). EC-007 verified via
`std::panic::catch_unwind` confirming no panic on empty registry.

**Test suite:** `crates/prism-audit/tests/sec007_org_slug_cross_check.rs`

```
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

---

## Full Test Suite Summary

| Test file | Crate | Tests | Result |
|-----------|-------|-------|--------|
| cr003_slug_pattern.rs | prism-customer-config | 10 | ok |
| sec006_toml_multiline_redaction.rs | prism-customer-config | 5 | ok |
| cr004_dispatch_consolidation.rs | prism-dtu-harness | 9 | ok |
| sec007_org_slug_cross_check.rs | prism-audit | 7 | ok |
| **Total** | | **31** | **ok** |

---

## Architecture Notes

- **E-CFG-019** (`InvalidOrgSlugPattern`) introduced as next sequential code after E-CFG-018 (added by W3-FIX-SEC-003). No enum variant conflicts.
- **`validate_all` visibility** changed `pub` → `pub(crate)` in `validator.rs`. `load_and_validate` remains the sole public entry point; `cargo build --workspace` confirmed no external callers break.
- **Exhaustive match** in `start_clone` and `build_network()`: no `_ =>` wildcard arm — adding a new `DtuType` variant is a compile error until the match is updated.
- **poll_test_hook backoff**: 10ms spin replaced with `tokio::time::sleep(Duration::from_millis(50))` with doc comment noting the Notify upgrade path (CR-006).
- **org_slug_guard module** (`prism_audit::org_slug_guard`): new public module exposing `validate_org_slug_cross_check` and `SlugCheckResult`. Wired into `AuditEmitterService::call()` via `tracing::warn!` on non-Matched results.

---

_Generated by Demo Recorder per POL-010. Evidence committed to `feature/W3-FIX-CODE-002`._
