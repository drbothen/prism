---
document_type: uncertainty-map
story_id: S-3.05
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
---

# S-3.05 Uncertainty Map — Pagination and Caching

## Summary verdict

**RED** — A direct cross-reference conflict was found: story pins `lru 0.12.x`
but the **prism-query workspace already declares `lru = "0.17"`** as a
dev-dep (S-3.2.08 introduced it). Adopting `0.12.x` from the story would
either downgrade or cause two-version coexistence in the same crate.
Additionally, the story is ambiguous on its caching backend — it lists `lru`
in the dep table but the body text on line 209 says "moka or similar".

## Findings

| Severity | Category | Finding | Recommended action |
|---|---|---|---|
| Critical | version-pin | **Workspace conflict.** Story line 252: `lru | 0.12.x | LRU eviction policy`. Existing `crates/prism-query/Cargo.toml` line 27: `lru = "0.17"` (dev-dep introduced for S-3.2.08 LruCache mirroring). The lru crate had a major API redesign between 0.12 and 0.13 (`get`/`put` return semantics, generic `S` hasher param), and again between 0.13 and 0.14+. 0.17 is a different major. | Story must be updated to pin `lru = "0.17"` and align API references. RESEARCH-NEEDED: confirm 0.17 is still latest stable as of 2026-05-04 and any API changes since 0.17.x cut. |
| Important | feature-claim | Line 209: "response cache (`moka` or similar) keyed by query hash + tenant" — but the dep table lists only `lru`. Two different cache crates implied. | Resolve which crate ships: `lru` is in-process bounded LRU; `moka` is async-aware concurrent w/TTL. The TTL-based eviction (BC-2.07.003) leans toward moka. RESEARCH-NEEDED: pick one and pin. |
| Important | architecture-pattern | ADR-008 universal re-keying via `{org_id}:` prefix is NOT explicitly threaded into the cache key derivation (line 95: `query_str: String, client_id: TenantId`). `TenantId` may or may not be the same construct as `org_id`. | RESEARCH-NEEDED: confirm `TenantId` is the canonical org_id and that cache key derivation includes it as an isolation prefix per ADR-008 multi-tenant requirement. |
| Important | version-pin | `sha2 0.10.x` (line 253) is current major but should be pinned with explicit minor for VP-025 reproducibility (Kani proof determinism). | Pin `sha2 = "=0.10.x"` once exact patch chosen. |
| Suggestion | unpinned-version | `kani` (line 259) version not pinned. Same issue as S-3.04. | Cross-reference Kani version with S-3.04. |
| Suggestion | feature-claim | Cursor expiry / token lifecycle (BC-2.07.002) implies background task. No `tokio` features pinned for `time`. | Confirm `tokio = { version = "1", features = ["time", "sync"] }` matches workspace pattern. |

## Cross-references

- **CONFLICT:** prism-query Cargo.toml dev-dep `lru = "0.17"` vs story `lru 0.12.x`.
- BC-2.07.001..006 all active in BC-INDEX v4.32.
- ADR-008 (org_id re-keying) must thread through cache_key.

## RESEARCH-NEEDED queries

1. "Latest stable lru crate version for Rust as of 2026-05-04. API changes between 0.12, 0.17, and current. Is `LruCache::get` still `&mut self` or has interior mutability been added?"
2. "moka crate latest stable version. moka vs lru tradeoff for concurrent in-process cache with TTL eviction in Rust async services."
3. "sha2 0.10 latest patch. Has `Sha256` Digest API changed?"

