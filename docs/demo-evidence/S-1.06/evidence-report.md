# S-1.06 Demo Evidence Report

**Story:** S-1.06 — prism-credentials: Credential Store Trait and Backends  
**Branch:** feature/S-1.06-credential-store  
**Commit at recording:** 5e96540  
**Test result:** 35/35 pass (0 fail, 0 ignored)  
**Tool:** VHS 0.10.0 (CLI recordings)  
**Font:** FiraCode Nerd Font Mono 14pt  
**Theme:** Dracula  

---

## Coverage Map

| Recording | AC | BC | VP | Path | Files |
|-----------|----|----|----|----- |-------|
| AC-1-credential-store-set-get | AC-1 | BC-2.03.001, BC-2.03.002 | — | success | [.gif](AC-1-credential-store-set-get.gif) [.webm](AC-1-credential-store-set-get.webm) [.tape](AC-1-credential-store-set-get.tape) |
| AC-2-backend-selector-auto-fallback | AC-2 | BC-2.03.012 | — | success + error | [.gif](AC-2-backend-selector-auto-fallback.gif) [.webm](AC-2-backend-selector-auto-fallback.webm) [.tape](AC-2-backend-selector-auto-fallback.tape) |
| AC-3-encrypted-file-round-trip | AC-3 | BC-2.03.003 | — | success | [.gif](AC-3-encrypted-file-round-trip.gif) [.webm](AC-3-encrypted-file-round-trip.webm) [.tape](AC-3-encrypted-file-round-trip.tape) |
| AC-4-namespace-isolation | AC-4 | BC-2.03.004 | — | success | [.gif](AC-4-namespace-isolation.gif) [.webm](AC-4-namespace-isolation.webm) [.tape](AC-4-namespace-isolation.tape) |
| AC-5-path-traversal-rejection | AC-5 | BC-2.03.008 | — | error (6 variants) | [.gif](AC-5-path-traversal-rejection.gif) [.webm](AC-5-path-traversal-rejection.webm) [.tape](AC-5-path-traversal-rejection.tape) |
| AC-6-startup-probe | AC-6 | BC-2.03.011 | — | success + unavail | [.gif](AC-6-startup-probe.gif) [.webm](AC-6-startup-probe.webm) [.tape](AC-6-startup-probe.tape) |
| AC-7-explicit-keyring-hard-error | AC-7 | BC-2.03.012 | — | error (hard) | [.gif](AC-7-explicit-keyring-hard-error.gif) [.webm](AC-7-explicit-keyring-hard-error.webm) [.tape](AC-7-explicit-keyring-hard-error.tape) |
| AC-8-VP034-encryption-round-trip | AC-8 | BC-2.03.003 | VP-034 | proptest 256 cases | [.gif](AC-8-VP034-encryption-round-trip.gif) [.webm](AC-8-VP034-encryption-round-trip.webm) [.tape](AC-8-VP034-encryption-round-trip.tape) |
| AC-9-VP035-key-derivation-deterministic | AC-9 | BC-2.03.003 | VP-035 | proptest 256 cases | [.gif](AC-9-VP035-key-derivation-deterministic.gif) [.webm](AC-9-VP035-key-derivation-deterministic.webm) [.tape](AC-9-VP035-key-derivation-deterministic.tape) |
| AC-10-list-returns-all-entries | AC-10 | BC-2.03.001 | — | success + empty | [.gif](AC-10-list-returns-all-entries.gif) [.webm](AC-10-list-returns-all-entries.webm) [.tape](AC-10-list-returns-all-entries.tape) |
| AC-ERR-error-paths | EC-001..005 | BC-2.03.003, BC-2.03.008, BC-2.03.012 | — | error paths | [.gif](AC-ERR-error-paths.gif) [.webm](AC-ERR-error-paths.webm) [.tape](AC-ERR-error-paths.tape) |
| AC-SUITE-all-35-tests | AC-1..10 + ECs | all | VP-034, VP-035 | full suite | [.gif](AC-SUITE-all-35-tests.gif) [.webm](AC-SUITE-all-35-tests.webm) [.tape](AC-SUITE-all-35-tests.tape) |

---

## Acceptance Criteria Status

| AC | Description | BC | Recorded | Result |
|----|-------------|-----|---------|--------|
| AC-1 | set("acme","crowdstrike","api_key","secret123") then get returns Some("secret123") | BC-2.03.001, BC-2.03.002 | AC-1-credential-store-set-get | PASS |
| AC-2 | backend="auto" + keyring unavailable → EncryptedFileBackend + WARN log | BC-2.03.012 | AC-2-backend-selector-auto-fallback | PASS |
| AC-3 | EncryptedFileBackend AES-256-GCM set→get round-trip | BC-2.03.003 | AC-3-encrypted-file-round-trip | PASS |
| AC-4 | get("beta","crowdstrike","api_key") returns None when only "acme" stored it | BC-2.03.004 | AC-4-namespace-isolation | PASS |
| AC-5 | CredentialName("../../etc/passwd") returns Err(InvalidCredentialName) | BC-2.03.008 | AC-5-path-traversal-rejection | PASS |
| AC-6 | probe_keyring() detects keyring availability without panic | BC-2.03.011 | AC-6-startup-probe | PASS |
| AC-7 | backend="keyring" + probe fails → Err (no silent downgrade) | BC-2.03.012 | AC-7-explicit-keyring-hard-error | PASS |
| AC-8 | VP-034 proptest: AES-256-GCM encrypt→decrypt = original for all inputs | VP-034, BC-2.03.003 | AC-8-VP034-encryption-round-trip | PASS |
| AC-9 | VP-035 proptest: same passphrase+salt → same derived key always | VP-035, BC-2.03.003 | AC-9-VP035-key-derivation-deterministic | PASS |
| AC-10 | list("acme") after 3 stores returns exactly 3 (sensor,name) pairs | BC-2.03.001 | AC-10-list-returns-all-entries | PASS |

**Error paths recorded:** EC-001 (NoEntry→None), EC-002 (corrupt decrypt→Err), EC-003 (path traversal→Err), EC-004 (explicit keyring hard error), EC-005 (empty passphrase→Err)

---

## Verification Properties

| VP | Description | Test | Result |
|----|-------------|------|--------|
| VP-034 | AES-256-GCM encrypt→decrypt round-trip (proptest, 256 cases) | `test_BC_2_03_003_prop_encrypt_decrypt_round_trip` | PASS |
| VP-035 | Argon2id key derivation deterministic (proptest, 256 cases) | `test_BC_2_03_003_prop_key_derivation_deterministic` | PASS |

Note: VP-034 and VP-035 use proptest (property-based testing), not Kani formal verification. Kani verification of these properties is deferred — the prism-credentials crate is classified **Effectful** (file I/O, OS keyring calls) per `architecture/purity-boundary-map.md`, making Kani proofs inapplicable. Proptest provides the required coverage for VP-034 and VP-035.

---

## Deferred / Not Applicable

| Item | Reason |
|------|--------|
| Kani formal proofs | prism-credentials is Effectful (OS keyring + file I/O); Kani applies to Pure modules only. VP-034/VP-035 covered by proptest per story spec. |
| Playwright recordings | CLI/library product — no web UI. VHS is the correct toolchain. |

---

## File Inventory

```
docs/demo-evidence/S-1.06/
├── evidence-report.md
├── AC-1-credential-store-set-get.tape
├── AC-1-credential-store-set-get.gif
├── AC-1-credential-store-set-get.webm
├── AC-2-backend-selector-auto-fallback.tape
├── AC-2-backend-selector-auto-fallback.gif
├── AC-2-backend-selector-auto-fallback.webm
├── AC-3-encrypted-file-round-trip.tape
├── AC-3-encrypted-file-round-trip.gif
├── AC-3-encrypted-file-round-trip.webm
├── AC-4-namespace-isolation.tape
├── AC-4-namespace-isolation.gif
├── AC-4-namespace-isolation.webm
├── AC-5-path-traversal-rejection.tape
├── AC-5-path-traversal-rejection.gif
├── AC-5-path-traversal-rejection.webm
├── AC-6-startup-probe.tape
├── AC-6-startup-probe.gif
├── AC-6-startup-probe.webm
├── AC-7-explicit-keyring-hard-error.tape
├── AC-7-explicit-keyring-hard-error.gif
├── AC-7-explicit-keyring-hard-error.webm
├── AC-8-VP034-encryption-round-trip.tape
├── AC-8-VP034-encryption-round-trip.gif
├── AC-8-VP034-encryption-round-trip.webm
├── AC-9-VP035-key-derivation-deterministic.tape
├── AC-9-VP035-key-derivation-deterministic.gif
├── AC-9-VP035-key-derivation-deterministic.webm
├── AC-10-list-returns-all-entries.tape
├── AC-10-list-returns-all-entries.gif
├── AC-10-list-returns-all-entries.webm
├── AC-ERR-error-paths.tape
├── AC-ERR-error-paths.gif
├── AC-ERR-error-paths.webm
├── AC-SUITE-all-35-tests.tape
├── AC-SUITE-all-35-tests.gif
└── AC-SUITE-all-35-tests.webm
```

Total: 37 files (12 .tape + 12 .gif + 12 .webm + 1 evidence-report.md)
