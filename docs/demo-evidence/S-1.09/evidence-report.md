# Evidence Report — S-1.09: Confirmation Tokens (P1)

**Story:** S-1.09  
**Branch:** feature/S-1.09-confirmation-tokens  
**Policy:** POL-010  
**Date:** 2026-04-23  
**Test state:** 200/200 pass (`cargo test -p prism-security`)

---

## Coverage Summary

| AC | Title | BC | Recording | Error Path |
|----|-------|----|-----------|------------|
| AC-1 / AC-7 | RiskTier gate routing (Read→Allow, Reversible→DryRun, Irreversible→Token) | BC-2.04.007 | `AC-001-risktier-routing.gif` | EC-006: Read tier never gated (all variants) |
| AC-1 | Token generation — CSPRNG 256-bit ID, cap check, sweep-before-issue | BC-2.04.009 | `AC-002-token-generation.gif` | EC-004: 101st token → E-FLAG-007 |
| AC-2 | Token consumption — single-use, atomic consumed-before-dispatch | BC-2.04.010 | `AC-003-token-consumption.gif` | EC-002: double-consume → E-FLAG-004 |
| AC-3 | Token expiry — 299s valid; 301s → E-FLAG-003; boundary 300s expired | BC-2.04.011 | `AC-004-token-expiry.gif` | EC-001: boundary 300s treated as expired |
| AC-4 | Content hash invariance under key reordering; tampered params rejected | BC-2.04.012 | `AC-005-content-hash.gif` | EC-003: device_id:B mismatch → E-FLAG-005 |
| AC-5 | Reversible dry-run default — no mutation without `dry_run:false` | BC-2.04.008 | `AC-006-dry-run-reversible.gif` | EC-005: explicit `dry_run:false` → Allow |
| AC-6 | VP-007/VP-008/VP-009/VP-010 Kani proofs + unit counterparts | VP-007..010 | `AC-007-kani-proofs.md` (Markdown) | N/A — formal proof |
| EC-004 | 100-token cap overflow → E-FLAG-007, no eviction | BC-2.04.009; VP-010 | `AC-008-overflow-eflag007.gif` | Cap freed after consume (VP-010 slot test) |
| EC-001 | Expiry sweep removes expired tokens, frees cap slots | BC-2.04.011 | `AC-009-expiry-sweep.gif` | Consumed tokens excluded from active count |

---

## Full Suite Recording

| Recording | Description |
|-----------|-------------|
| `FULL-SUITE.gif` | All 200 prism-security tests — 25 test binaries, all green |

---

## File Index

### VHS Recordings

| File | AC | Format |
|------|----|--------|
| `AC-001-risktier-routing.gif` | AC-1, AC-7 | GIF (embed) |
| `AC-001-risktier-routing.webm` | AC-1, AC-7 | WebM (archival) |
| `AC-001-risktier-routing.tape` | AC-1, AC-7 | VHS source |
| `AC-002-token-generation.gif` | AC-1 | GIF |
| `AC-002-token-generation.webm` | AC-1 | WebM |
| `AC-002-token-generation.tape` | AC-1 | VHS source |
| `AC-003-token-consumption.gif` | AC-2 | GIF |
| `AC-003-token-consumption.webm` | AC-2 | WebM |
| `AC-003-token-consumption.tape` | AC-2 | VHS source |
| `AC-004-token-expiry.gif` | AC-3 | GIF |
| `AC-004-token-expiry.webm` | AC-3 | WebM |
| `AC-004-token-expiry.tape` | AC-3 | VHS source |
| `AC-005-content-hash.gif` | AC-4 | GIF |
| `AC-005-content-hash.webm` | AC-4 | WebM |
| `AC-005-content-hash.tape` | AC-4 | VHS source |
| `AC-006-dry-run-reversible.gif` | AC-5 | GIF |
| `AC-006-dry-run-reversible.webm` | AC-5 | WebM |
| `AC-006-dry-run-reversible.tape` | AC-5 | VHS source |
| `AC-008-overflow-eflag007.gif` | EC-004 / VP-010 | GIF |
| `AC-008-overflow-eflag007.webm` | EC-004 / VP-010 | WebM |
| `AC-008-overflow-eflag007.tape` | EC-004 / VP-010 | VHS source |
| `AC-009-expiry-sweep.gif` | EC-001 | GIF |
| `AC-009-expiry-sweep.webm` | EC-001 | WebM |
| `AC-009-expiry-sweep.tape` | EC-001 | VHS source |
| `FULL-SUITE.gif` | All | GIF |
| `FULL-SUITE.webm` | All | WebM |
| `FULL-SUITE.tape` | All | VHS source |

### Markdown Documentation

| File | Purpose |
|------|---------|
| `AC-007-kani-proofs.md` | AC-6: Kani proof harnesses VP-007..010, unit counterparts, invocation instructions |
| `evidence-report.md` | This file |

---

## Verification

All VHS recordings produced `.gif` and `.webm` outputs via `vhs 0.10.0`.
Font: `FiraCode Nerd Font Mono`. Theme: `Catppuccin Mocha`.

```
$ cargo test -p prism-security 2>&1 | grep "test result.*ok"
test result: ok. 0 passed ...    # (doc-tests)
test result: ok. 8 passed ...    # bc_2_04_001_test
test result: ok. 7 passed ...    # bc_2_04_002_test
test result: ok. 10 passed ...   # bc_2_04_003_test
test result: ok. 8 passed ...    # bc_2_04_004_test
test result: ok. 8 passed ...    # bc_2_04_005_test
test result: ok. 8 passed ...    # bc_2_04_006_test
test result: ok. 12 passed ...   # bc_2_04_007_test
test result: ok. 7 passed ...    # bc_2_04_008_test
test result: ok. 10 passed ...   # bc_2_04_009_test
test result: ok. 11 passed ...   # bc_2_04_010_test
test result: ok. 10 passed ...   # bc_2_04_011_test
test result: ok. 11 passed ...   # bc_2_04_012_test
test result: ok. 8 passed ...    # bc_2_04_013_test
test result: ok. 7 passed ...    # bc_2_04_015_test
test result: ok. 8 passed ...    # bc_2_09_002_test
test result: ok. 16 passed ...   # bc_2_09_003_test (proptest)
test result: ok. 7 passed ...    # bc_2_09_004_test
test result: ok. 10 passed ...   # bc_2_09_005_test
test result: ok. 4 passed ...    # bc_2_09_006_test
test result: ok. 8 passed ...    # bc_2_09_007_test
test result: ok. 4 passed ...    # proptest_injection
test result: ok. 11 passed ...   # vp_007_010_test
test result: ok. 7 passed ...    # vp_020_test
Total: 200 passed; 0 failed
```
