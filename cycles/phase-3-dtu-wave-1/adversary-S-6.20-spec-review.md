---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-04-22T00:00:00
phase: 3
inputs:
  - .factory/stories/S-6.20-dtu-demo-server.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-002-l2-dtu-clone-template.md
  - .factory/specs/architecture/decisions/ADR-003-dtu-reset-lookup-and-fidelity-auth.md
  - .factory/stories/S-6.07-dtu-crowdstrike.md
  - .factory/stories/S-6.08-dtu-claroty.md
  - .factory/stories/S-6.09-dtu-cyberint.md
  - .factory/stories/S-6.10-dtu-armis.md
  - .factory/stories/S-6.14-dtu-threatintel.md
  - .factory/stories/S-6.15-dtu-nvd.md
  - .factory/stories/S-5.05-config-loading.md
  - .factory/specs/architecture/dtu-assessment.md
  - .factory/tech-debt-register.md
input-hash: "caab672"
traces_to: S-6.20-dtu-demo-server.md
pass: 1
previous_review: null
cycle: phase-3-dtu-wave-1
recommendation: BLOCKED pending spec fixes
---

# Adversarial Review — S-6.20 Unified DTU Demo Harness — Pass 1

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: `P3W1` (phase-3-dtu-wave-1)
- `<PASS>`: Two-digit pass number (`P01`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

Examples from this pass: `ADV-P3W1-P01-CRIT-001`, `ADV-P3W1-P01-HIGH-001`

## Part A — Fix Verification (pass >= 2 only)

N/A — this is Pass 1.

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

#### ADV-P3W1-P01-CRIT-001 (F-6.20-01): SS-18 subsystem anchor is semantically wrong (POL-004 violation)

- **Severity:** CRITICAL
- **Category:** spec-fidelity
- **Location:** S-6.20 lines 8, 34, 251–253; ARCH-INDEX line 109
- **Description:** Story assigns subsystems: [SS-18] and anchor_subsystem: SS-18, but SS-18 is the Action Delivery Engine (Slack/PagerDuty/Jira/email/syslog dispatch) owned by prism-operations. It has no relationship to DTU test infrastructure.
- **Evidence:** Story line 8 `subsystems: [SS-18]`, line 34 `anchor_subsystem: SS-18`, lines 251–253 narrative fabricates "SS-18 owns ... Action Delivery + DTU infra". ARCH-INDEX line 109 defines SS-18 as "Action Delivery Engine | actions.md (AD-021) | prism-operations | Phase 3". S-6.09 precedent uses `anchor_subsystem: null`; S-6.07 uses SS-01.
- **Proposed Fix:** Set `subsystems: []` and `anchor_subsystem: null` (matching S-6.09), OR introduce a new "DTU Test Infrastructure" subsystem in ARCH-INDEX (architect decision), OR use SS-01 if anchoring to sensor surface.

#### ADV-P3W1-P01-CRIT-002 (F-6.20-02): "7 merged clones" is factually wrong (actual is 6)

- **Severity:** CRITICAL
- **Category:** spec-fidelity
- **Location:** S-6.20 lines 59, 180, 197, 199, 223, 333; AC-1; AC-6
- **Description:** The story repeatedly states "7 clones" but enumerates only 6 stories and lists only 6 ports.
- **Evidence:** Story line 59 enumerates "S-6.07..S-6.10, S-6.14, S-6.15" = 6 stories. Lines 180/197/199/223/333 repeatedly say "7". Port assignment Task 8 lists 6 ports (17080–17085). Cargo features list 6 dtu deps. AC-1 "all 7 clones bind" is un-satisfiable as written.
- **Proposed Fix:** Global search-replace `7` → `6` in narrative, AC-1, AC-6, and file-structure table.

#### ADV-P3W1-P01-CRIT-003 (F-6.20-03): ADR-002 §6 "admin endpoint bind rule" is fabricated citation

- **Severity:** CRITICAL
- **Category:** spec-fidelity
- **Location:** S-6.20 lines 171, 280–281; AC-9 (lines 234–236)
- **Description:** The story cites "ADR-002 §6 admin endpoint rule" for loopback-only binding, but that rule does not exist in ADR-002. AC-9 is also logically incoherent — a startup-time bind cannot be "rejected" per request.
- **Evidence:** grep "loopback|127.0.0.1|bind-any|admin.*endpoint" in ADR-002 returns zero matches.
- **Proposed Fix:** Drop the ADR-002 citation and declare as new policy inline in this story, OR architect amends ADR-002 with a new §10 "Demo/Admin endpoint binding". Rewrite AC-9 to distinguish startup-refusal vs request-refusal.

### HIGH

#### ADV-P3W1-P01-HIGH-001 (F-6.20-04): BehavioralClone::seed() trait method duplicates configure({"seed": N})

- **Severity:** HIGH
- **Category:** interface-gaps
- **Location:** S-6.20 Task 2; ADR-002 §4–§5; AC-3
- **Description:** A new `seed()` trait method duplicates the already-established `configure()` path for seeding, contradicting ADR-002 §4 (configure() is the single config-change path).
- **Evidence:** Task 2 proposes `seed()` default method. `CrowdstrikeState.apply_config` already reads `config.get("seed")`. AC-3 says `UnsupportedOperation → 400`, but no clone will naturally return this (all silently ignore unknown keys per ADR-002 §5).
- **Proposed Fix:** Remove Task 2. Route `/dtu/<clone>/seed` as a harness wrapper forwarding to each clone's `/dtu/configure` with `{"seed": payload}`. Or drop the admin endpoint and document that seeding uses each clone's `/dtu/configure`.

#### ADV-P3W1-P01-HIGH-002 (F-6.20-05): --bind-any is a single-flag secure-default violation

- **Severity:** HIGH
- **Category:** security-surface
- **Location:** S-6.20 Task 5; AC-9
- **Description:** A single `--bind-any` flag exposes fixtures + admin endpoints + seed (>1MB write surface per EC-004) on any network interface with only a log line as mitigation.
- **Evidence:** No second factor required; admin endpoints exposed on non-loopback interfaces when flag is set.
- **Proposed Fix:** Require 2-factor (env var `PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK` + flag). Disable admin endpoints when bound to non-loopback, or split control/data plane onto separate ports. Narrow AC-9 to "admin endpoints return 403 from non-loopback regardless of --bind-any".

#### ADV-P3W1-P01-HIGH-003 (F-6.20-06): #![cfg(feature = "dtu")] on src/main.rs is a binary-crate footgun; AC-8 hedges

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** S-6.20 Task 4; AC-8
- **Description:** Applying `#![cfg(feature = "dtu")]` to main.rs produces a "main function not found" compile error when the feature is absent, not a clean skip. AC-8 acknowledges this ambiguity ("build succeeds but binary not produced (or emits compile error)").
- **Evidence:** S-6.09 precedent works because it is a `src/bin/` sub-binary, not a cfgd-out main.rs. A `cfg`'d-out main.rs does not compile cleanly.
- **Proposed Fix:** Use `[[bin]] name = "prism-dtu-demo-server", required-features = ["dtu"]` in Cargo.toml. Remove `#![cfg]` from main.rs. Rewrite AC-8: "cargo build without --features dtu: no binary produced (cargo skips target)."

#### ADV-P3W1-P01-HIGH-004 (F-6.20-07): AC-6 production Prism credential_ref integration unspecified

- **Severity:** HIGH
- **Category:** missing-edge-cases
- **Location:** S-6.20 AC-6
- **Description:** AC-6 states production Prism loads `configs/prism-demo.toml` and routes sensor queries, but does not specify what `credential_ref` entries that config uses, nor how S-5.05 resolution applies in a demo context.
- **Evidence:** S-5.05 resolves `credential_ref` to keyring/env/vault. Harness forbids depending on production crates. Story is silent on the credential_ref scheme.
- **Proposed Fix:** Document credential_ref scheme (e.g., `"env:DEMO_FAKE_TOKEN"` with `DEMO_FAKE_TOKEN=dtu-fake-cs-token` in demo launch). Add AC-6 precondition. Or defer AC-6 to a follow-up integration story with S-5.05.

#### ADV-P3W1-P01-HIGH-005 (F-6.20-08): AC-7 byte-identical VHS across runs unachievable without timestamp control

- **Severity:** HIGH
- **Category:** verification-gaps
- **Location:** S-6.20 AC-7
- **Description:** `tracing-subscriber` emits ISO-8601 timestamps by default; `seeded_rng` controls clone RNG but not log output. VHS itself renders with wall-clock info. Byte-identical replay is not achievable without additional controls.
- **Evidence:** No `--deterministic-logging` flag or timestamp-suppression mechanism specified.
- **Proposed Fix:** Weaken AC-7 to "JSON response bodies byte-identical with `--deterministic-logging`; VHS rendering determinism out of scope". Add task to implement `--deterministic-logging`. Or move to integration test that diffs JSON bodies against stored fixtures.

#### ADV-P3W1-P01-HIGH-006 (F-6.20-09): Hidden dependency on TD-WV0-05 retroactive cleanup

- **Severity:** HIGH
- **Category:** missing-story
- **Location:** S-6.20 (implicit); ADR-002 Retroactive Cleanup
- **Description:** ADR-002 Retroactive Cleanup requires `prism-dtu-threatintel` to gain `POST /dtu/reset` and `GET /dtu/health`, but this work has not landed. The harness implicitly expects a uniform endpoint set across all 6 clones.
- **Evidence:** Glob for `routes/dtu.rs` in `prism-dtu-threatintel` returns empty.
- **Proposed Fix:** Add `depends_on: [TD-WV0-05-resolved]` to story metadata. Add task: "verify TD-WV0-05 cleanup landed before implementation". Add AC: all 6 clones return 200 on `/dtu/health`.

### MEDIUM

#### ADV-P3W1-P01-MED-001 (F-6.20-10): Wave 2/3 clone onboarding plan is vague

- **Severity:** MEDIUM
- **Category:** ambiguous-language
- **Location:** S-6.20 narrative (onboarding section)
- **Description:** The story does not specify the concrete steps required to add a Wave-2/3 clone to the harness.
- **Proposed Fix:** Specify: new Wave-2/3 clones add path dep + feature flag entry + `clones.<name>` config block + harness match arm. Or introduce inventory/linkme registration pattern.

#### ADV-P3W1-P01-MED-002 (F-6.20-11): Fixed ports 17080–17085 contradict dtu-assessment §4 "random ports on 127.0.0.1"

- **Severity:** MEDIUM
- **Category:** contradictions
- **Location:** S-6.20 Task 8; dtu-assessment §4
- **Description:** `dtu-assessment §4` mandates random ports for zero-conflict. Story fixes ports for VHS replay determinism without acknowledging the deviation.
- **Proposed Fix:** Acknowledge the deviation in Architecture Compliance Rules; add `--ephemeral-ports` option for non-VHS usage.

#### ADV-P3W1-P01-MED-003 (F-6.20-12): DemoHarness::stop_all semantics conflict with BehavioralClone lifecycle

- **Severity:** MEDIUM
- **Category:** interface-gaps
- **Location:** S-6.20 DemoHarness API; BehavioralClone trait
- **Description:** `BehavioralClone` trait has no `stop()` method. `reset()` is state-clear, not shutdown. `stop_all()` has no defined implementation path.
- **Proposed Fix:** Either extend trait with `stop()` or have `start()` return a `JoinHandle`/`Sender` that `stop_all()` cancels.

#### ADV-P3W1-P01-MED-004 (F-6.20-13): AC-2 sample curl uses "Bearer x" but S-6.07 requires valid token

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** S-6.20 AC-2
- **Description:** S-6.07's ADR-003 `check_auth` has no bypass. `Authorization: Bearer x` → 401.
- **Proposed Fix:** Use `Bearer dtu-fake-cs-token` in the sample, or switch sample to `/dtu/health` (unauthenticated).

#### ADV-P3W1-P01-MED-005 (F-6.20-14): AC-4 TLS test cert provenance unspecified

- **Severity:** MEDIUM
- **Category:** missing-edge-cases
- **Location:** S-6.20 Task 7; AC-4
- **Description:** Task 7 does not specify where `rcgen` writes the generated certificate.
- **Proposed Fix:** Write to `./.prism-dtu-demo-server-cert.pem`, print path + fingerprint on startup, delete on SIGTERM.

#### ADV-P3W1-P01-MED-006 (F-6.20-15): AC-1 port-binding is not atomic; partial-start cleanup unspecified

- **Severity:** MEDIUM
- **Category:** missing-edge-cases
- **Location:** S-6.20 AC-1
- **Description:** If clone 4 fails to bind, the fate of clones 1–3 (already bound) is unspecified, creating a resource leak.
- **Proposed Fix:** Implement atomic all-or-nothing startup: abort all tasks, exit 1 with combined error message. Add EC-009.

### LOW

#### ADV-P3W1-P01-LOW-001 (F-6.20-16): points: 5 disproportionate for 3-day story with no BCs/VPs

- **Severity:** LOW
- **Category:** ambiguous-language
- **Location:** S-6.20 frontmatter
- **Proposed Fix:** Reduce to 3 or justify the 5.

#### ADV-P3W1-P01-LOW-002 (F-6.20-17): estimated_days: 3 underestimates TLS + VHS integration

- **Severity:** LOW
- **Category:** ambiguous-language
- **Location:** S-6.20 frontmatter
- **Proposed Fix:** Raise to 5 or defer TLS to a follow-up story.

#### ADV-P3W1-P01-LOW-003 (F-6.20-18): POL-010 demo-evidence scope ambiguous for this story

- **Severity:** LOW
- **Category:** ambiguous-language
- **Location:** S-6.20 (POL-010 compliance note)
- **Proposed Fix:** Clarify: harness doesn't produce evidence itself; downstream stories' VHS tapes use this harness.

#### ADV-P3W1-P01-LOW-004 (F-6.20-19): rustls 0.23.x + axum 0.7.x bridge unspecified

- **Severity:** LOW
- **Category:** missing-edge-cases
- **Location:** S-6.20 Task 7 (TLS)
- **Proposed Fix:** Add `axum-server 0.7.x` with `tls-rustls` feature, or `hyper-rustls 0.27.x`. Confirm workspace compatibility.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 3 |
| HIGH | 6 |
| MEDIUM | 6 |
| LOW | 4 |
| OBSERVATION | 3 |
| **Total** | **22** |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires revision — do not create worktree until CRITICALs resolved

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 1 |
| **New findings** | 22 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (22 / 22) |
| **Median severity** | 3.0 (MEDIUM) |
| **Trajectory** | 22 |
| **Verdict** | FINDINGS_REMAIN |

## Observations

- **F-6.20-20:** Review checklist premise about "ADR-003 static endpoints" is incorrect — ADR-003 has no such rule.
- **F-6.20-21:** seed vs rand naming collision — `rand` has no `seed()` function; `SeedableRng::seed_from_u64` is the actual API. Minor confusion; subsumed by ADV-P3W1-P01-HIGH-001.
- **F-6.20-22:** Novelty HIGH — Pass 1 on fresh story; 22-finding density indicates a pre-implementation editorial pass is due before worktree creation.

## Recommendation

BLOCKED pending spec fixes. 3 CRITICAL findings (subsystem anchor, clone count, fabricated citation) mislead implementers about authoritative facts. 6 HIGH findings would cause implementation friction or insecure outcomes if not addressed up front.

Next step: product-owner triages CRITICALs, fixes or explicitly defers HIGHs with rationale. Re-run adversarial review (Pass 2) before worktree creation.
