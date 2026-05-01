---
document_type: demo-evidence-report
product: "prism-dtu-cyberint (S-3.4.04 harness migration)"
pipeline_run: "2026-04-30T12:37:00Z"
demo_type: "cli"
recording_tool: "vhs"
status: complete
story_id: S-3.4.04
behavioral_contracts: [BC-3.5.001, BC-3.5.002, BC-3.6.001]
---

# Demo Evidence Report — S-3.4.04

## Product: prism-dtu-cyberint (Cyberint harness migration)
## Pipeline Run: 2026-04-30T12:37:00Z
## Demo Type: CLI (Rust integration tests via cargo)

---

## Per-AC Demo Recordings

| AC | Story | Description | Tape | GIF | WebM | Size (webm) | Status |
|----|-------|-------------|------|-----|------|-------------|--------|
| AC-001 | S-3.4.04 | All 26 harness_tests green (BC-3.5.001 postcondition 1) | [tape](AC-001-cyberint-harness-migration-green.tape) | [gif](AC-001-cyberint-harness-migration-green.gif) | [webm](AC-001-cyberint-harness-migration-green.webm) | 414K | recorded |
| AC-002 | S-3.4.04 | Multi-org logical isolation — disjoint alert sets (BC-3.5.001 postcondition 2, TV-2) | [tape](AC-002-multi-org-logical-isolation.tape) | [gif](AC-002-multi-org-logical-isolation.gif) | [webm](AC-002-multi-org-logical-isolation.webm) | 164K | recorded |
| AC-003 | S-3.4.04 | Network cross-org credential mismatch → HTTP 401 (BC-3.5.002 postcondition 2, TV-3) | [tape](AC-003-network-cross-creds-401.tape) | [gif](AC-003-network-cross-creds-401.gif) | [webm](AC-003-network-cross-creds-401.webm) | 162K | recorded |
| AC-004 | S-3.4.04 | Failure injection via with_failure — OrgB unaffected (BC-3.6.001 postcondition 1, EC-001) | [tape](AC-004-failure-injection-with-failure.tape) | [gif](AC-004-failure-injection-with-failure.gif) | [webm](AC-004-failure-injection-with-failure.webm) | 168K | recorded |
| AC-005 | S-3.4.04 | Harness regression-safe — 70/70 prism-dtu-harness tests pass | [tape](AC-005-harness-regression-safe.tape) | [gif](AC-005-harness-regression-safe.gif) | [webm](AC-005-harness-regression-safe.webm) | 716K | recorded |
| AC-006 | S-3.4.04 | Cyberint legacy tests still pass — 93 total (26 harness + 67 legacy) | [tape](AC-006-cyberint-legacy-tests-pass.tape) | [gif](AC-006-cyberint-legacy-tests-pass.gif) | [webm](AC-006-cyberint-legacy-tests-pass.webm) | 911K | recorded |

---

## Coverage Summary

| Behavioral Contract | ACs Covered | Evidence |
|--------------------|-------------|----------|
| BC-3.5.001 (Harness Logical Isolation Invariants) | AC-001, AC-002, AC-005, AC-006 | 26 harness_tests green; multi-org disjoint sets; 70 harness regressions clean; 93 cyberint tests clean |
| BC-3.5.002 (Harness Network Isolation Invariants) | AC-003 | Cross-org credential mismatch → 401 |
| BC-3.6.001 (Failure Injection) | AC-004 | Timeout injection on OrgA; OrgB unaffected |

---

## Test Count Evidence

| Suite | Tests | Result |
|-------|-------|--------|
| `prism-dtu-cyberint` `--test harness_tests` | 26 | 26 passed, 0 failed |
| `prism-dtu-cyberint` (all, incl. 67 legacy) | 93 | 93 passed, 0 failed |
| `prism-dtu-harness` (regression) | 70 | 70 passed, 0 failed |

---

## Toolchain

| Tool | Version | Status |
|------|---------|--------|
| VHS | 0.10.0 | installed |
| cargo | rustup-managed | installed |
| FiraCode Nerd Font Mono | — | installed |

---

## PR Embedding Snippet

```markdown
## Demo Evidence — S-3.4.04 Cyberint Harness Migration

| AC | Description | Recording |
|----|-------------|-----------|
| AC-001 | 26 harness_tests green (BC-3.5.001) | ![AC-001](docs/demo-evidence/S-3.4.04/AC-001-cyberint-harness-migration-green.gif) |
| AC-002 | Multi-org logical isolation — disjoint sets | ![AC-002](docs/demo-evidence/S-3.4.04/AC-002-multi-org-logical-isolation.gif) |
| AC-003 | Cross-org creds → HTTP 401 (BC-3.5.002) | ![AC-003](docs/demo-evidence/S-3.4.04/AC-003-network-cross-creds-401.gif) |
| AC-004 | Failure injection via with_failure (BC-3.6.001) | ![AC-004](docs/demo-evidence/S-3.4.04/AC-004-failure-injection-with-failure.gif) |
| AC-005 | Harness regression-safe (70/70) | ![AC-005](docs/demo-evidence/S-3.4.04/AC-005-harness-regression-safe.gif) |
| AC-006 | 93 cyberint tests pass (26 harness + 67 legacy) | ![AC-006](docs/demo-evidence/S-3.4.04/AC-006-cyberint-legacy-tests-pass.gif) |
```

---

## Notes

- All recordings use VHS 0.10.0 with Catppuccin Mocha theme, FiraCode Nerd Font Mono, 1200x600.
- Recordings show `cargo test` output in real-time — no mocked output.
- WebM is the primary format; GIF provided for inline PR embedding.
- Tests run against pre-built binaries in the S-3.4.04 worktree (debug profile).
- AC-005 and AC-006 GIFs are larger (2.1MB / 2.8MB) due to the full test suite output scrolling — WebM sizes are within acceptable range (716K / 911K).
