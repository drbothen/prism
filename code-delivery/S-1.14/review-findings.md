# Review Findings — S-1.14: Infusion Spec Loading and UDF Registration

**PR:** #21
**Branch:** feature/S-1.14-infusion-specs
**Reviewed:** 2026-04-22

---

## Convergence Tracking

| Cycle | Findings | Blocking | Fixed | Remaining | Result |
|-------|----------|----------|-------|-----------|--------|
| 1 | 2 | 0 | 0 | 0 | APPROVE |

**Verdict: APPROVE — 0 blocking findings after cycle 1.**

---

## Cycle 1 Findings

### F-001: RUSTSEC-2026-0002 — lru 0.12.5 unsound IterMut

- **Severity:** LOW (non-blocking)
- **Category:** dependency advisory
- **Location:** `crates/prism-spec-engine/Cargo.toml` — `lru = "0.12"`
- **Detail:** RUSTSEC-2026-0002 flags `IterMut` in `lru 0.12.5` as unsound under Stacked Borrows. Our usage exclusively accesses `LruCache` via `get`/`put` behind a `tokio::sync::Mutex` — `IterMut` is never called.
- **Impact:** NONE to our code paths.
- **Action:** Accepted as LOW. Upgrade to patched version when available. Tracked as tech-debt.
- **Status:** ACCEPTED (not blocking)

### F-002: Tier 2 and Tier 3 cache methods are unimplemented!()

- **Severity:** INFORMATIONAL (non-blocking)
- **Category:** spec-fidelity (known scope boundary)
- **Location:** `crates/prism-spec-engine/src/infusion/cache.rs:InfusionLruCache::get/insert`
- **Detail:** `InfusionLruCache::get` and `InfusionLruCache::insert` remain `unimplemented!()`. The story spec notes these are stub bodies for Tier 2/3 — Tier 1 (QueryScopedInfusionCache) is fully implemented and tested by VP-049. Tier 2/3 integration wiring occurs in S-3.02 when prism-query calls the UDF path.
- **Impact:** Tier 1 dedup (the BC-2.19.002 invariant) is fully proven. Tiers 2/3 are architectural stubs awaiting prism-storage wiring.
- **Action:** Expected per story scope. Documented in demo evidence (AC-8 deferred note).
- **Status:** ACCEPTED (not blocking)

---

## Quality Checks

| Check | Result |
|-------|--------|
| AC coverage | 10/10 ACs covered (AC-8 RocksDB deferred by spec design) |
| BC traceability | BC-2.19.001/002/003/004/005 all covered by named test groups |
| VP-048 Kani | Harness authored, compile-check green, Phase 5 formal run |
| VP-049 proptest | 1000 cases, all pass |
| CredentialRef redaction | Type-level enforcement confirmed (Debug = <redacted>) |
| arc_swap::ArcSwap | Used (not RwLock) — confirmed |
| DataFusion dep | NONE confirmed (AD-015) |
| prism-storage dep | NONE confirmed (CacheBackend trait injection) |
| E-RULE-012 interface | is_api_backed() exported correctly |
| E-INFUSE-001/002/003/004/005 | All error codes implemented |

---

## Conclusion

**APPROVE.** Zero blocking findings. Two non-blocking informational items (dependency advisory on lru + Tier 2/3 cache stubs expected per scope). All 5 BCs verified by dedicated test groups. VP-049 proptest passes. PR is ready to merge after CI green.
