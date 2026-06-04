# PR #171 Re-Review — S-DEMO-002 (E2E Subprocess Smoke Test)

**Head:** `a99ca196` · **Base:** `develop` @ `b38c1abc` · **Verdict: APPROVE**

Re-review of the two prior BLOCKERs (prior verdict REQUEST-CHANGES at `60f2854a`).
Both are resolved. No new blockers.

---

## Prior BLOCKER-1 — Inaccurate PR body ("integration test only — no production code changed") — RESOLVED

The PR body now contains an accurate **Blast Radius (Production Code Changed)** section
enumerating 10 production crates. I verified each major claim against the actual
`b38c1abc..a99ca196` diff rather than trusting the prose:

- `prism-core/src/error.rs` — new `PrismError::SensorNotRegisteredForOrg { sensor_id, org_slug }`
  variant, `#[error("E-QUERY-032: ...")]`. **Present in diff.**
- `prism-bin/src/spec_driven_adapter.rs` — `pub struct BearerStaticCredentialAuthProvider`
  + AuthProvider impl, fail-closed bearer resolution. **Present in diff.**
- `prism-credentials/src/resolution.rs` — 4-tier chain + `per_client_env_var()` /
  `per_client_file_env_var()` helpers. **Present in diff.**
- `prism-query/src/materialization.rs` — `resolve_source_refs` returns
  `Err(PrismError::SensorNotRegisteredForOrg ...)` on explicit-scope org/sensor mismatch.
  **Present in diff.**
- `prism-query/src/engine.rs` — `None`-scope preservation fix (`effective_clients = None`
  when `options.clients.is_none()`), replacing the prior `Some(clients.clone())`. **Present in diff.**
- `prism-mcp/src/error_mapping.rs` — `SensorNotRegisteredForOrg` → MCP `-32602`. **Present in diff.**
- `prism-mcp/src/safety_envelope.rs` — object-shaped `{"rows":[...]}` extraction in the
  injection scanner path (`results.get("rows").and_then(|v| v.as_array())`). **Present in diff.**

Security note present and accurate: **SEC-001 (CWE-668, cross-tenant credential isolation)**
is documented as closed, with the actual fix verified at the Tier 4 "env" backend branch
(`resolution.rs`: "use per-client env-var format (ADR-032 Tier 2), NOT global lookup …
CWE-668: cross-tenant isolation gap via global env namespace"), backed by a worked-example
unit test (`test_per_client_env_var_worked_examples`). SEC-003 and SEC-004/MED-002 also
documented and verified in-diff.

The full E-QUERY-032 isolation chain is wired end-to-end: raise (materialization) →
map to `-32602` (error_mapping) → MCP response. Real plumbing, not a doc-comment paper-fix.

## Prior BLOCKER-2 — Committed `e2e-run-output.txt` was a RED run; count mismatch — RESOLVED

`docs/demo-evidence/S-DEMO-002/e2e-run-output.txt` at head is a real **GREEN** capture:

- Command: `cargo nextest run -p prism-bin --profile e2e --run-ignored ignored-only`
- Result: **13 tests run: 13 passed, 0 failed, 110 skipped** — captured at `0af51150`.
- Counts reconcile: `e2e_smoke.rs` contains exactly **13** `#[test]` functions
  (verified by enumeration), all 13 appear PASS in the log. Standard profile = 110 PASS / 13 skipped.
  **123 total = 110 standard + 13 e2e** — internally consistent. The prior 121-vs-123 mismatch is gone.
- `evidence-report.md` agrees with the log (13/13 e2e GREEN; 110/110 standard; per-AC table).
- 8 `.gif` recordings present, ≥1 per AC group; superseded pre-convergence artifacts are
  explicitly labeled as retained-for-traceability (not passed off as current evidence).

---

## Reconfirmations

- **SUGGESTION-1 (AC-008 SIGTERM unit substitute):** Now a justified SID-1 §2 deferral.
  `signals.rs` (the `std::process::exit(0)` shutdown path) is **not** in this diff —
  consistent with the "unmockable without architectural refactor" rationale. The deferral cites a
  concrete future anchor (**S-1.12-FOLLOWUP**) per SID-1 §5. AC-008 is still covered by a GREEN
  subprocess E2E test. **Acceptable.**

- **SEC-001 fix + AC-007 robustness change — coherence:** No incoherence. AC-007 robustness
  (accepting both string and array `data_source` serialization) is documented in evidence-report.md
  and traced to ADV-SDEMO002-PR-P01-OBS-002. The `safety_flags == []` assertion is noted as
  non-vacuous (requires ≥1 row). Demo-evidence claims match the test set; no overstated claims found.

- **Coherence / scope:** All changes are within `crates/`, `docs/demo-evidence/`, `sensors/`,
  `.config/`. No unrelated changes. All 7 SID-1 unit substitutes named in the body verified to
  exist as real `#[test]` functions.

---

## Findings

| Severity | Category | Finding | Suggestion |
|----------|----------|---------|------------|
| NIT | description | Summary line "13 E2E tests + 7 SID-1 unit substitutes — 123 total" can read as 13+7+... additive; the 123 is actually 110 standard + 13 e2e (the 7 SID-1 substitutes are a subset of the 110). The Test Evidence section states it correctly. | Optionally reword so the 123 arithmetic is unambiguous. Non-blocking. |

## Diff Size

77 files / +5639 / −277. Above the 500-line flag threshold, but justified for a keystone E2E
story spanning the adapter→query→MCP→DTU pipeline plus an IMPORTANT security fix. Production
`src/**` portion is +1757/−160 across the enumerated crates. Not a blocker.

## Checklist

1. Diff Coherence — PASS
2. Description Accuracy — PASS (Blast Radius added; verified against diff)
3. Test Coverage — PASS (13 e2e + 7 SID-1 unit substitutes, all verified to exist)
4. Demo Evidence — PASS (GREEN log + 8 GIFs + evidence-report.md, per-AC)
5. Commit Quality — n/a at review (squash-merge)
6. Diff Size — WARN-but-justified (keystone story + security fix)
7. Missing Changes — PASS
8. Dependency Status — PASS (PR #166, PR #155 merged)

**Verdict: APPROVE.** Both prior BLOCKERs resolved; one NIT (non-blocking).
