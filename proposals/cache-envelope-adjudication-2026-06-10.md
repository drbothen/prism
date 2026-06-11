---
document_type: architect-decision-note
subject: QRY cascade pass-1 P1-01 / P1-04 / P1-05 adjudication (cache envelope + watchdog code collision)
adjudicator: architect
adjudicated_at: 2026-06-10T00:00:00Z
source: QRY cascade pass-1 (2026-06-10 review cycle), findings P1-01, P1-04, P1-05; orchestrator dispatch
branch_under_review: fix/review-2026-06-10-query-core (.worktrees/FIX-REVIEW-QRY-2026-06-10)
artifacts_amended_in_this_burst: [".factory/specs/prd-supplements/error-taxonomy.md v1.67→v1.68 (D2, applied by architect)"]
consumed_by: orchestrator (work-order dispatch), product-owner (BC amendments), implementer (fix-burst)
precedent: TD-S-PLUGIN-PREREQ-A-004-close-as-superseded-ratification.md (same dir)
out_of_scope: "P1-03 (cached-normalized vs raw-response model) — human adjudication in progress; BC-2.07.003 normalization language is FROZEN in all work-orders below"
---

# Architect Adjudication — Cache Envelope + Watchdog Code Collision (2026-06-10)

Three decisions for the QRY cascade pass-1 findings. Evidence read before deciding:
BC-2.07.003 v4.4, BC-2.07.005 v4.3, BC-2.11.001, BC-2.11.006 (incl. v1.12 mapping
table), BC-2.15.007 v1.4, error-taxonomy.md v1.67 (E-WATCHDOG + E-QUERY sections),
nfr-catalog.md R-010 memory row, and on the QRY branch: `prism-query`
`materialization.rs` (`derive_response_cache_key`, fan-out target construction,
cache store/force_refresh paths), `cache_key.rs`, `cache.rs`
(`force_refresh`/`remove_entry`), `memory.rs` (`map_datafusion_memory_error`),
`prism-core` `error.rs` (`QueryMemoryBudgetExceeded`, `WatchdogKilled`),
`prism-storage` `watchdog.rs` + `denylist.rs` (`record_failure`),
`prism-bin` `spec_driven_adapter.rs` (`query.limit` seeding), and the
F-P1-CRIT-004 load-bearing test in `bc_2_01_013_spec_driven_adapter.rs`.

---

## D1 — P1-01: Cache key vs limit pushdown

### Decision: RATIFY Option (b) — include the effective fetch-limit in the cache key

BC-2.07.005 is amended. The `limit` exclusion's stated premise — "the cache stores
the full sensor API response; `limit` is applied after materialization" — is
factually false since BC-2.01.013 v1.14 / F-P1-CRIT-004: the fan-out target seeds
`QueryParams.limit` from `options.limit` (BC-2.11.001 default 25), and
`SpecDrivenSensorAdapter::fetch()` pushes it as `query.filters["query.limit"]`, so
the response that QRY-02 stores in the cache is **limit-truncated at the sensor
API**. Without the fix, a `limit=25` query populates an entry that a later
`limit=1000` query with identical filters silently hits — returning 25 records and
reporting `total_available` ≈ 25. That is wrong data served to an MSSP analyst,
not a performance nit.

### Precedence justification

This is a BC-vs-BC conflict (same precedence level), routed to architect per
CLAUDE.md §Source-of-Truth Precedence closing rule. Applying the rule-1 spirit
(LATER, MORE-SPECIFIC wins): BC-2.01.013 v1.14 + the F-P1-CRIT-004 load-bearing
test (2026-06, human-ratified fix) post-date and are more specific than
BC-2.07.005 v4.3's exclusion clause (2026-04-14), whose rationale they invalidate.
The earlier clause is brought into alignment.

### Alternatives weighed and rejected

| Option | Verdict | Reason |
|--------|---------|--------|
| (a) Stop pushing limit into fetch | REJECT | Reverts F-P1-CRIT-004's efficiency/quota win (default-25 queries would fetch all pages of potentially 100K-record sources); contradicts BC-2.01.013 v1.14 and its shipped load-bearing test; worsens memory pressure against the 10K/200MB budgets. |
| (b) Include effective fetch-limit in cache key | **ADOPT** | Correctness guaranteed: an entry fetched under limit L serves only queries fetching under L. Default-25 queries (the dominant MCP case) share one entry. Fragmentation is bounded by the small set of distinct limits in practice, and limit-truncated entries are smaller. Minimal blast radius: one hash input, no key-shape change, no `CacheEntry` entity change, prefix-scan invalidation untouched. |
| (c) Skip cache when limit pushed | REJECT | BC-2.11.001 `limit` defaults to 25, so MCP-originated queries ALWAYS push a limit → the cache would never engage from MCP → QRY-02 (cross-query response cache) becomes dead code. Destroys the feature the cascade is reviewing. |
| (b′) 5-tuple key with `fetch_limit` first-class | REJECT | Would change the `CacheEntry` entity definition, BC invariants, and prefix-scan semantics for no additional correctness over hashing it. |
| (b″) Monotone serving (entry with fetch-limit ≥ requested satisfies the query) | REJECT (design choice, not deferral) | Requires range lookup over an exact-key store (moka) plus post-trim logic; the default-25 dominance means the hit-rate gain is marginal. Exact-key inclusion is fully correct on its own. |

### Key-composition design (binding)

- The hashed parameter is the **effective fetch-limit**: the exact `u64` value
  assigned to `QueryParams.limit` for the fan-out target (currently
  `options.limit.map(|l| l as u64).unwrap_or(0)`).
- Canonical parameter key: `fetch.limit` (cannot collide with WHERE filters,
  which are namespaced `filter.<column>`, nor with `start_time`/`end_time`).
- `0` is the no-limit sentinel (EC-008 of BC-2.01.013 v1.14): when the effective
  fetch-limit is 0/absent, the parameter is **omitted** from the canonical form —
  consistent with BC-2.07.005's null/absent-omission rule. Unlimited fetches
  share entries with unlimited fetches.
- Coherence invariant: **the limit hashed is the limit fetched.** The same local
  binding must feed both the cache key derivation and the fan-out target, so the
  two can never drift (including any future pushdown-suppression logic, e.g.
  aggregation queries).
- Inclusion is uniform across sensors. Whether a given adapter honors the pushed
  limit (CrowdStrike does today; others may ignore it) is adapter-internal;
  keying on what was requested is conservative and correctness-safe, and avoids
  per-sensor key-composition logic that would drift as adapters gain pushdown.
- No migration concern: the cache is in-memory; key-shape changes take effect on
  process start.

### PO work-order (D1) — BC-2.07.005 + BC-2.07.003 amendments

**BC-2.07.005 (bump v4.3 → v4.4):**

1. §Postconditions, "Included in hash computation" — append bullet:

   > - The **effective fetch-limit**: the limit value pushed into the sensor
   >   fetch (`QueryParams.limit`, seeded from the `query` tool's `limit` per
   >   BC-2.11.001 and pushed to the sensor API per BC-2.01.013 v1.14 /
   >   F-P1-CRIT-004). Canonicalized as parameter key `fetch.limit` with the
   >   exact `u64` value seeded into the fetch; **omitted when no limit is
   >   pushed** (value 0 / absent — the no-limit sentinel), per the
   >   null/absent-omission rule. Because fetched responses are limit-truncated
   >   at the sensor API, an entry fetched under limit L is valid only for
   >   queries fetching under the same L. Default-limit queries (25) share one
   >   entry; differing limits create distinct entries (bounded fragmentation
   >   accepted — architect adjudication D1,
   >   `proposals/cache-envelope-adjudication-2026-06-10.md`).

2. §Postconditions, "Excluded from hash computation" — replace the bullet
   ``- `limit` on the `query` tool -- the cache stores the full sensor API response; `limit` is applied after materialization``
   with:

   > - The tool-level `limit`'s **post-materialization truncation role** remains
   >   excluded — what is hashed is the *effective fetch-limit* actually pushed
   >   into the sensor fetch (see Included list). When limit push-down applies,
   >   the two are equal in value; the hash input is defined by the fetch, not
   >   by the tool parameter. (Supersedes the pre-v4.4 exclusion, whose premise
   >   — "the cache stores the full sensor response" — was invalidated by
   >   BC-2.01.013 v1.14 limit push-down.)

3. §Invariants — add: "The fetch-limit hashed into `push_down_hash` is always
   the limit value actually pushed into the fan-out fetch (single-binding
   coherence; the key can never describe a different truncation than the stored
   response)."

4. §Edge Cases — add rows:
   - **EC-07-043** | Two queries, identical filters, `limit=25` vs `limit=1000` |
     Different `push_down_hash`; no cache sharing — a 25-truncated response must
     never serve a 1000-limit query.
   - **EC-07-044** | Query with no effective fetch-limit (limit 0 / pushdown
     suppressed) vs the same filters with `fetch.limit` absent | Same
     `push_down_hash` — 0/absent are both omitted from the canonical form.

5. §Canonical Test Vectors — add the two vectors mirroring EC-07-043/044.

**BC-2.07.003 (bump v4.4 → v4.5) — limit-premise sentences ONLY; all
normalization language ("pre-OCSF-normalization", "raw sensor responses",
"no transformation applied before caching") is byte-frozen pending human P1-03
adjudication:**

1. §Postconditions: "the sensor API is queried (all pages fetched), the complete
   response is stored" → "the sensor API is queried (all pages fetched, up to
   the effective fetch-limit when limit push-down applies — BC-2.01.013 v1.14),
   and the complete response *for that effective fetch-limit* is stored".
2. §Postconditions: "The cache stores the full result set from the all-pages
   fan-out fetch (pre-OCSF-normalization sensor records)" → "The cache stores
   the complete result set returned by the fan-out fetch for the effective
   fetch-limit (pre-OCSF-normalization sensor records)" — note the
   parenthetical is preserved byte-identical.
3. Add cross-reference: "Cache keys distinguish effective fetch-limits per
   BC-2.07.005 v4.4 — entries truncated at different limits never alias."

### Implementer work-order (D1) — `prism-query`

1. `materialization.rs`: extract a single binding
   `let fetch_limit: u64 = options.limit.map(|l| l as u64).unwrap_or(0);` and
   use it for BOTH the fan-out target's `QueryParams.limit` AND the cache key
   derivation (coherence invariant above).
2. `derive_response_cache_key`: add a `fetch_limit: u64` parameter; when
   `fetch_limit > 0`, `params.insert("fetch.limit", serde_json::Value::from(fetch_limit))`;
   when 0, omit. Update its doc comment (the "tool-level `limit` … excluded"
   sentence) to the new BC-2.07.005 v4.4 language.
3. `cache_key.rs` module doc: remove "`limit` from the `query` tool" from the
   "Excluded from hash" list; document `fetch.limit` as a hashed component.
4. Sibling-sweep (TD-VSDD-060): all `derive_response_cache_key` call sites
   (cache-read path and cache-store path share one derivation — verify) and all
   tests constructing keys for the response cache.
5. Tests (load-bearing, not doc-only — TD-VSDD-059):
   - identical filters, `fetch_limit` 25 vs 1000 → different `push_down_hash`;
   - identical filters, `fetch_limit` 25 vs 25 → identical hash (shared entry);
   - `fetch_limit` 0 vs parameter-absent → identical hash;
   - end-to-end: query at limit 25 populates cache; same-filter query at limit
     1000 is a cache MISS (regression test for the P1-01 poisoning scenario).
6. VP-025 (`proofs/vp025_cache_key.rs`): determinism property is over arbitrary
   param maps — confirm the proof still passes unchanged; extend the harness
   only if it enumerates parameter names.

---

## D2 — P1-04: E-WATCHDOG-001 collision (DECIDED + taxonomy edit APPLIED)

### Decision: E-WATCHDOG-001 = per-query pool trip (QRY-01 mapping RATIFIED); WatchdogKilled moves to the EXISTING E-WATCHDOG-002

The orchestrator's tentative assignment ("-002 for the per-query pool trip,
keeping -001 for the process watchdog") is **inverted by the evidence**, which the
brief asked me to verify. The row texts and the NFR catalog both already define
the split:

| Evidence | Says |
|----------|------|
| Taxonomy v1.67 E-WATCHDOG-001 row | "Query memory limit exceeded… **The query's memory consumption** exceeded the watchdog budget… **Narrow the query scope** or increase the memory budget" — broken / validation / not retryable. Per-query condition; query at fault. |
| Taxonomy v1.67 E-WATCHDOG-002 row | "concurrent queries consuming {used_bytes} of {budget_bytes} **process budget**… **the query itself is not at fault**" — degraded / transient / retryable. Process-pressure condition. |
| nfr-catalog.md R-010 "Concurrent Query Note" | Explicit: "the later query receives `E-WATCHDOG-002` (retryable: true)… `E-WATCHDOG-001` (retryable: false) is **reserved for per-query limit violations**". |
| Implemented architecture | Per-query 200MB budget is enforced by the DataFusion GreedyMemoryPool → `PrismError::QueryMemoryBudgetExceeded` (BC-2.11.006); the watchdog as built kills on **process RSS** at 95% of the 512MB budget → `PrismError::WatchdogKilled` (BC-2.15.007 EC-15-027 already names the RSS guard as separate from per-query termination; VP-058). |
| Denylist | `prism_storage::denylist::record_failure` (BC-2.15.008) counts **consecutive watchdog-triggered failures** — the watchdog-kill condition, not the pool trip. The denylist sentence therefore belongs on the watchdog code (-002), not on -001 where v1.67 carried it. |

Therefore: **no new code is allocated.** The namespace already contains exactly
the two codes for exactly the two conditions; the defect is that QRY-01 left
`WatchdogKilled` (which pre-dated QRY-01 on E-WATCHDOG-001) colliding with the
newly-correct `QueryMemoryBudgetExceeded` mapping. Allocating E-WATCHDOG-004
(-003 is a permanent tombstone) would create the inverse ADR-038 violation — two
codes for one condition — by orphaning the existing -002 row. ADR-038
one-code-one-condition is satisfied by moving `WatchdogKilled` to -002 and
regularizing both rows' Message Formats to the verbatim shipped displays
(ADR-035 canonical-row convention, code-prefixed).

Considered and rejected: adding a `used_bytes` (RSS-at-kill) field to
`WatchdogKilled` to preserve the old -002 row's `{used_bytes}` placeholder. That
changes the shape of a variant already shipped on develop (constructor +
test blast radius) for diagnostics already available via the watchdog's audit
events; the row's Message Format follows the shipped display per the
adjudication brief.

### Taxonomy diff (APPLIED in this burst — error-taxonomy.md v1.67 → v1.68)

`## WATCHDOG: Watchdog Errors` rows -001 and -002 amended (no rows added or
deleted; POL-1 append-only numbering respected; -003 tombstone untouched):

- **E-WATCHDOG-001** — Message Format → `"E-WATCHDOG-001: query memory budget
  exceeded: limit {limit_mb}MB, used {used_mb}MB"` (verbatim
  `PrismError::QueryMemoryBudgetExceeded` display). Severity broken / category
  validation / Retryable No (unchanged). Description rewritten to the per-query
  DataFusion-pool condition (BC-2.11.006, `map_datafusion_memory_error`); the
  v1.67 denylist sentence moved to -002 (denylisting is watchdog-termination
  driven per BC-2.15.008).
- **E-WATCHDOG-002** — Message Format → `"E-WATCHDOG-002: watchdog killed query
  — process RSS exceeded kill threshold ({budget_bytes} bytes budget); query
  token cancelled"` (verbatim `PrismError::WatchdogKilled` display after the
  implementer's prefix change below). Severity degraded / category transient /
  Retryable Yes (unchanged). Description keeps the process-pressure /
  not-at-fault / retry semantics and gains the denylist sentence
  (BC-2.15.008 → E-QUERY-008).
- Changelog row v1.68 appended; frontmatter version bumped. Input-hash refresh
  is state-manager's burst step.

### Implementer work-order (D2) — `prism-core` + `prism-storage`

1. `prism-core/src/error.rs` `WatchdogKilled`: change the display to exactly
   `"E-WATCHDOG-002: watchdog killed query — process RSS exceeded kill threshold ({budget_bytes} bytes budget); query token cancelled"`
   (only the code prefix changes: `E-WATCHDOG-001:` → `E-WATCHDOG-002:`).
   Update the variant doc comment ("E-WATCHDOG-001 (query kill)" →
   "E-WATCHDOG-002 (query kill)").
2. `PrismError::QueryMemoryBudgetExceeded` display: **UNCHANGED** —
   `"E-WATCHDOG-001: query memory budget exceeded: limit {limit_mb}MB, used {used_mb}MB"`.
3. Sibling-sweep (TD-VSDD-060): `rg 'E-WATCHDOG-001' crates/` — after the fix,
   every remaining hit must refer to the per-query pool condition
   (`memory.rs`, `materialization.rs` comments, `integration_tests.rs`,
   `bc_gap_fill_tests.rs`, `execute_integration_tests.rs`). Watchdog-kill sites
   to flip to E-WATCHDOG-002: `prism-storage/src/watchdog.rs` doc comment on
   the kill path, `watchdog_tests.rs` Display assertions
   (`display.contains("E-WATCHDOG-001")` → `"E-WATCHDOG-002"` plus the
   accompanying comment citing "story v1.7 correction").
4. `prism-mcp/src/error_mapping.rs` maps both variants by enum arm, not by
   string — verify no string-coupled mapping; no change expected.

### PO work-order (D2 companion sweep) — BC alignment to taxonomy v1.68

1. **BC-2.15.007**: §Postconditions memory bullet + §Error Conditions row +
   §Canonical Test Vectors — the watchdog kill code is now `E-WATCHDOG-002`
   with the process-RSS display above. Restate the division of labor explicitly:
   per-query memory budget is enforced by the DataFusion GreedyMemoryPool
   (BC-2.11.006 → E-WATCHDOG-001); the watchdog's memory enforcement is the
   process-RSS kill (→ E-WATCHDOG-002), consistent with EC-15-027. Also correct
   the record-count row: the streaming-counter violation is `E-QUERY-005`
   ("Materialization limit exceeded: fetched {count} records") per taxonomy
   v1.67+, not `E-QUERY-006` (pre-fetch scope estimate). The timeout row's
   `E-QUERY-004` already matches taxonomy v1.67+.
2. **BC-2.11.006**: refresh the v1.12 "canonical mapping table" (it still says
   E-QUERY-004 = QueryMemoryBudgetExceeded / E-QUERY-005 = timeout): post-QRY-01
   + this adjudication the canonical mapping is E-QUERY-004 = timeout,
   E-QUERY-005 = materialization limit, memory budget = E-WATCHDOG-001.
3. **nfr-catalog.md R-010 row**: already consistent with this assignment except
   the clause "per-query limit violations **that trigger denylist**" — amend to
   reflect that denylisting tracks watchdog terminations (BC-2.15.008), not
   pool trips.

---

## D3 — P1-05: force_refresh + failed fetch semantics

### Decision: RATIFY invalidation — a forced refresh whose fetch cannot produce a complete replacement REMOVES the old entry

BC-2.07.003 is silent; the QRY-branch implementation stores only in the
clean-success branch (`fan_result.errors.is_empty()`), so on a failed forced
refresh the distrusted entry keeps serving subsequent **non-forced** queries for
the remainder of its TTL.

`force_refresh` is an explicit analyst distrust signal ("I do not trust the
cached value"). Once expressed, continuing to serve that exact entry to other
queries is a data-integrity failure in an MSSP/IR context — analysts act on this
data. The forcing caller already receives the fetch error via the
partial-failure envelope (BC-2.11.011 `sensor_errors`); the gap is solely the
lingering entry.

**Availability counter-argument weighed** (serve-stale-on-error, HTTP
stale-if-error analogy): rejected for the forced path, for four reasons —
(i) force_refresh is precisely the one signal that stale is known-unacceptable
for this entry; (ii) blast radius is a single cache key, and recovery is
automatic on the next successful fetch; (iii) a subsequent non-forced query
becomes a visible cache miss + fresh fetch attempt — a sensor outage then fails
loudly instead of silently serving distrusted data; (iv) the **non-forced** path
keeps full availability semantics: a normal fetch failure NEVER invalidates an
existing unexpired entry (no change there).

Scope refinement beyond the orchestrator's "fails entirely" wording: the
invalidation trigger is "forced refresh cannot store a complete replacement" —
i.e., BOTH the all-targets-failed case AND the partial-error case (partial
responses are never cached per the complete-responses-only rule, so retaining
the old entry would equally keep serving distrusted data). Boundary documented
explicitly: invalidation is per-entry (per cache key). Sibling entries for the
same filters at OTHER fetch-limits (post-D1) are not enumerable under the hashed
key structure and are not invalidated; their staleness remains bounded by TTL
(≤300s), which is the cache's designed staleness tolerance — force_refresh
tightens it only for the entry the analyst's query addresses, matching
EC-07-041's per-hash replacement semantics.

### PO work-order (D3) — BC-2.07.003 (same v4.4 → v4.5 bump as D1's edits; normalization language frozen per P1-03)

1. §Postconditions — add:

   > - When `force_refresh: true` and the fresh fetch fails to produce a
   >   complete response — either all targets failed, or per-target errors made
   >   the result partial (partial responses are never cached) — the existing
   >   cache entry for the tuple is **invalidated (removed)**, not retained.
   >   The fetch failure is surfaced to the forcing caller via the
   >   partial-failure envelope (BC-2.11.011). Subsequent non-forced queries
   >   for the tuple miss the cache and attempt a fresh fetch. Rationale:
   >   `force_refresh` is an explicit analyst distrust signal; retaining the
   >   distrusted entry would silently serve it to later queries (architect
   >   adjudication D3, `proposals/cache-envelope-adjudication-2026-06-10.md`).
   >   A fetch failure on a **non-forced** query never invalidates an existing
   >   unexpired entry (availability semantics unchanged on the normal path).
   >   Invalidation is per-entry; sibling entries at other fetch-limits age out
   >   by TTL.

2. §Edge Cases — add rows:
   - **EC-07-033** | `force_refresh: true`; fresh fetch fails for all targets |
     Existing entry for the tuple is invalidated; error surfaced to the forcing
     caller (BC-2.11.011); subsequent non-forced identical query is a cache
     miss and re-attempts the fetch.
   - **EC-07-034** | `force_refresh: true`; fresh fetch returns partial results
     (some targets errored) | Partial response is NOT cached
     (complete-responses-only); existing entry is invalidated; partial results
     + `sensor_errors` returned to the forcing caller.

3. §Canonical Test Vectors — add the two vectors mirroring EC-07-033/034.

### Implementer work-order (D3) — `prism-query` `materialization.rs` + `cache.rs`

1. Restructure the fan-out result handling so the derived response-cache key is
   available in the `Err` (all-targets-failed) branch and the
   partial-errors path, not only the clean-success branch (currently the key is
   moved into the success-store arm).
2. When `options.force_refresh` is true and the fetch failed or was partial:
   call the cache's entry-removal operation (`remove_entry`, already used
   internally by `SensorResponseCache::force_refresh`) for that key — expose it
   at the needed visibility if currently private; eviction accounting must stay
   atomic with partition mutation (TD-PRISM-QUERY-CACHE-001 invariant).
3. Do NOT invalidate on non-forced fetch failures (asymmetry is intentional;
   see D3 postcondition).
4. Tests (load-bearing):
   - forced refresh, all targets fail → entry gone (subsequent non-forced
     identical query is a MISS) AND the forced query's response carries the
     partial-failure error;
   - forced refresh, partial errors → nothing cached, old entry gone;
   - NON-forced fetch failure with an existing unexpired entry → entry
     retained (regression guard for the availability asymmetry);
   - accounting: invalidation path does not desync `total_bytes`/partition
     counts (extend the CR-014-style test).

---

## Adjacent findings surfaced (routing per CLAUDE.md Companion Principle)

1. **E-QUERY-004 format drift** — taxonomy v1.67 row: `"Query timed out after
   {seconds}s"`; shipped display: `"E-QUERY-004: query timed out after
   {elapsed_ms}ms"`. Same condition, one code (no collision), but Message Format
   is not verbatim. Route into the QRY cascade fix-burst: regularize the
   taxonomy row to the shipped display (ADR-035 canonical-row convention), or
   have the implementer format seconds — architect recommendation: taxonomy
   follows the shipped `{elapsed_ms}ms` display (millisecond precision is
   diagnostically superior; matches how the timeout is measured). PO applies in
   the D2-companion taxonomy/BC sweep burst.
2. **`record_failure` (BC-2.15.008 denylist) has no production caller** — the
   denylist read path is wired (`watchdog_status`), but no termination site
   records failures yet, so E-QUERY-008 can never fire. This is a wiring gap
   against an active BC (spec wins, rule 7). Route to orchestrator for cascade
   triage / story attachment — it is outside P1-01/04/05 scope but must not be
   lost.

---

## Self-audit checklist (CLAUDE.md Canonical Principle)

- [x] No "MVP / for now / good enough / fix later" rationalizations — every
      rejection above is a design decision with stated rationale, not a deferral.
- [x] No tech-debt-register entries added. Adjacent finding #2 is routed to the
      orchestrator with explicit BC anchor, not parked.
- [x] No "pending architect review" left anywhere — this memo IS the architect
      adjudication; all three questions answered in scope.
- [x] Defects found in others' output were fixed or work-ordered with exact
      text, not surfaced as questions: D2 taxonomy edit applied by me (architect
      owns namespace allocations per ADR-035/038 precedent); BC and code edits
      work-ordered to their owner-specialists (product-owner / implementer) per
      the routing table — correct-agent pattern, same work cycle.
- [x] Correct mechanism over cheap mechanism: D1 hashes the fetch-limit with a
      single-binding coherence invariant rather than skipping the cache; D2
      reuses the semantically-correct existing code instead of allocating a new
      one; D3 invalidates rather than doc-commenting the gap.
- [x] No paper-fixes (TD-VSDD-059): every work-order requires load-bearing
      tests (P1-01 poisoning regression, E-WATCHDOG-002 display assertions,
      force_refresh invalidation + availability-asymmetry guards).
- [x] Sibling-site sweeps specified (TD-VSDD-060) for the display change, the
      key-derivation signature change, and the BC/taxonomy cross-references.
- [x] No volatile line-number pins in normative text (TD-VSDD-091) — anchors
      are function/variant/BC/EC names.
- [x] P1-03 boundary respected: BC-2.07.003 normalization language byte-frozen
      in both PO work-orders; no edit in this burst touches it.
- [x] Anchor justification: ADR-035/038 cited for namespace ownership and
      one-code-one-condition; BC-2.01.013 v1.14 / BC-2.11.001 / BC-2.11.006 /
      BC-2.15.007/008 verified by reading the files, not from the brief.
