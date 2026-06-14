---
document_type: adversarial-review
story_id: S-DEMO-MULTI-TENANT-DTU-001
pass: 2
scope: LOCAL
protocol: BC-5.39.001 3-CLEAN-strict
produced_by: adversary (fresh-context)
timestamp: 2026-06-13T23:58:00Z
verdict_clean_strict: "NO"
verdict_clean_pr_merge: "YES"
findings_total: 3
findings_crit: 0
findings_high: 0
findings_med: 2
findings_low: 1
all_findings_closed: true
streak_before: 0
streak_after: 0
next_pass: 3
---

# LOCAL Adversary Pass 2 — S-DEMO-MULTI-TENANT-DTU-001

**Story:** prism-dtu-demo-server + prism-dtu-harness: Per-DTU-Instance Multi-Address Binding  
**Protocol:** BC-5.39.001 3-CLEAN-strict (D-779 disambiguation — strict = zero findings of ANY severity)  
**Verdict:** CLEAN(strict)=NO / CLEAN(PR-merge)=YES  
**Streak:** 0/3 (findings present; streak cannot advance)  
**Next:** Pass 3

---

## Part A — Pass-1 Fix Verification

All Pass-1 fixes verified load-bearing. No regressions.

| Pass-1 Finding | Fix Claimed | Verification Result |
|----------------|-------------|---------------------|
| F-P1-HIGH-001 (isolation paper-fix) | Server-side AtomicU64 counter in ArmisClone; middleware; GET /dtu/request-count; 3 isolation tests rewritten to delta assertion (delta_b==1) | VERIFIED LOAD-BEARING — `request_count()` reads AtomicU64 incremented by actual middleware on every inbound request; isolation tests fail at delta_b>1 (cross-tenant leak); client-side count path removed. Genuine isolation guarantee, not paper-fix. |
| F-P1-MED-001 (stale Ok(HashMap::new())) | BC-2.06.017 v1.2→v1.3: EC-017-002 + TV-017-002 corrected to Err(MultiInstanceBindError::BindFailure(...)) | VERIFIED ACCURATE — error-path example and test vector now reflect the actual post-v1.2 return type; no stale HashMap::new() references remain in spec or test comments. |
| F-P1-MED-002 (drain race) | clone.stop().await called on bound instances before returning BindFailure | VERIFIED LOAD-BEARING — error path calls stop().await on the MultiInstanceServers handle accumulated to that point; port is genuinely released before BindFailure propagates; watcher-await-only path eliminated. |
| F-P1-MED-003 (EADDRINUSE test) | TV-017-005: demo-server bind-failure + harness aggregation tests added | VERIFIED — bind-failure test binds a real port, attempts double-bind, asserts BindFailure returned; harness aggregation test exercises stop().await through error path. Tests compile and pass. |
| F-P1-LOW-001/002 (stale 52→59 / "Let me recount") | Non-exhaustive gate crate doc corrected to 52→60 | VERIFIED — stale numeric literals and "Let me recount" artifact removed; value 60 is correct (EXPECTED=60 at merge). |
| F-P1-LOW-003 (misleading watcher comments) | Watcher-task comments rewritten | VERIFIED — new comments accurately describe the async shutdown machinery without implying single-task semantics. |

**SAP-1 check (Pass 2):** Grepped `event_type =` across workspace. All event_type values verified against BC-2.16.002 Structured Event Catalog. No new emission sites added by Pass-1 fix-burst. SAP-1 CLEAN.

**EXPECTED gate:** EXPECTED=60 confirmed exact. No regression.

---

## Part B — New Findings (Pass 2)

Classification: **DOC-SWEEP** — no code bugs; all findings are doc/comment factual errors introduced or exposed by Pass-1 edits.

### F-P2-MED-001 [MED] — Success-path watcher comments factually wrong (4 sites, 2 files)

**File(s):** `crates/prism-dtu-demo-server/src/multi_instance.rs`, `crates/prism-dtu-harness/src/multi_instance.rs`  
**Sites:** 4 doc/inline comments describing the watcher task lifecycle on the success path.

**Finding:** Comments state the watcher task "monitors for shutdown" using language from the pre-v1.2 single-instance `watch_task` pattern. After the MultiInstanceServers lifecycle-handle refactor (D-1145), the success path uses `shutdown_tx` + Drop graceful drain — a Tokio oneshot/broadcast channel pattern, not an ongoing monitoring watcher. The comments:
- Describe `spawn`'d tasks as "watching for server readiness" (wrong: they are tokio::spawn handles awaited on shutdown, not readiness monitors)
- Use present-tense "is watching" where the task has completed binding and is simply running until shutdown signal
- Reference "watcher loop" which implies polling; actual implementation awaits `with_graceful_shutdown` future

**Impact:** Future maintainer confusion; misleads adversary in a context where the shutdown plumbing is security-relevant (port release timing). No runtime bug.

**Status:** CLOSED — implementer corrected all 4 sites to accurately describe `shutdown_tx` broadcast + `with_graceful_shutdown` semantic (task runs until oneshot/broadcast fires, then gracefully drains in-flight requests, then exits; Drop impl sends shutdown_tx signal). Verified: no "watcher loop" / "monitoring" language remains on the success path.

---

### F-P2-MED-002 [MED] — Armis request_counter doc comment falsely claims "mirrors ClarotyState"

**File:** `crates/prism-dtu-armis/src/clone.rs`  
**Site:** Doc comment above `request_counter: Arc<AtomicUsize>` field in ArmisClone struct.

**Finding:** Comment reads: "mirrors ClarotyState request_counter pattern". This is factually wrong:
1. `ArmisClone.request_counter` is an `Arc<AtomicUsize>` added by the Pass-1 implementer to satisfy F-P1-HIGH-001 server-side isolation proof.
2. `ClarotyState` uses `Arc<AtomicU64>`, not `Arc<AtomicUsize>` — different type.
3. ClarotyState's counter was implemented in a different story cycle (S-DEMO-DTU-LIVE-SCENARIO-001-A), with different middleware wiring, different route path `/dtu/request-count` vs `/clone/request-count`, and different initialization strategy.
4. "Mirrors" implies parity that does not exist; a future refactor might incorrectly assume type compatibility.

**Impact:** False cross-component parity claim. Misleads code readers about the type, naming, and origin of the counter; could cause type-mismatch bugs if a refactor assumes AtomicU64=AtomicUsize compatibility.

**Status:** CLOSED — implementer amended doc comment to accurately describe the field: "Per-instance request counter incremented by axum middleware on every inbound request. Used to verify isolation in multi-tenant tests (server-side delta assertion). AtomicUsize (not AtomicU64 — differs from ClarotyState). Added by S-DEMO-MULTI-TENANT-DTU-001 Pass-1 fix-burst F-P1-HIGH-001." Verified accurate.

---

### F-P2-LOW-001 [LOW] — Stale Arc<AtomicUsize> reference comment

**File:** `crates/prism-dtu-armis/src/clone.rs`  
**Site:** 1 inline comment inside the isolation test module.

**Finding:** Comment references `Arc::clone(&state.request_counter)` using language from a draft of the implementation that stored the counter in a local variable before the final version stored it on `ArmisClone.request_counter` directly. The comment says "clone the Arc before moving into handler closure" but the counter is accessed via `Arc::clone` on a field reference, not a local binding. Misleads about ownership model.

**Impact:** Cosmetic/misleading. No functional impact.

**Status:** CLOSED — implementer corrected comment to "access counter field via Arc::clone for handler closure capture." Verified.

---

### OBS-2 [OBS] — Test comment says Ok(HashMap::new()) in isolation test setup block

**File:** `crates/prism-dtu-harness/src/multi_instance.rs` (test module)  
**Site:** 1 comment in test setup helper referencing the pre-v1.2 return type.

**Finding:** Test setup helper has a comment "// old API returned Ok(HashMap::new()) — now returns Ok(MultiInstanceServers)" which is accurate as a migration note but the parenthetical is stale because it implies HashMap::new() was the success value; the actual pre-v1.2 shape was `Ok(HashMap<String,SocketAddr>)` not `Ok(HashMap::new())` (default-value confusion). Minor.

**Impact:** Cosmetic. No functional impact; test logic correct.

**Status:** No action taken — OBS below threshold. Note carried forward to Pass-3 for fresh-context review (Pass-3 may choose to close or escalate).

---

## Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRIT | 0 | — |
| HIGH | 0 | — |
| MED | 2 | CLOSED (F-P2-MED-001, F-P2-MED-002) |
| LOW | 1 | CLOSED (F-P2-LOW-001) |
| OBS | 1 | No-action (OBS-2 — carried to Pass-3) |

**All MED+LOW findings CLOSED via implementer doc-only fix commit. just check GREEN post-fix. EXPECTED=60 unchanged. SAP-1 clean.**

**CLEAN(strict):** NO — OBS-2 present (carries to Pass-3 for fresh review).  
**CLEAN(PR-merge):** YES — zero CRIT/HIGH/MED open.  
**Streak:** 0/3 (strict criterion; OBS-2 prevents strict clean).  
**Next:** Pass 3.
