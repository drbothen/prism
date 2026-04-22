# Evidence Report — S-6.15: prism-dtu-nvd

| Field | Value |
|-------|-------|
| Story ID | S-6.15 |
| Title | prism-dtu-nvd: DTU for NVD/NIST CVSS API — L2 (stateful) |
| Date | 2026-04-21 |
| Impl SHA | d652ab3 |
| Branch | feature/S-6.15-dtu-nvd |
| Product Type | Library crate (no UI) — artifact-based evidence |
| Recording Tool | Artifact (cargo test output + per-AC .md files) |

## Green Gate

```
cargo test --features prism-dtu-nvd/dtu

test result: ok. 12 passed; 0 failed (prism-dtu-nvd AC suite)
exit code: 0
```

Full output: `test-run.txt`

## AC Coverage Matrix

| AC File | AC Statement (summary) | Test Function(s) | Tests | Status |
|---------|------------------------|-----------------|-------|--------|
| AC-1-cve-lookup.md | Single CVE lookup returns fixture (CVSS 9.8, KEV date) | `ac_1_cve_lookup_returns_fixture_cve_with_kev_and_cvss` | 1 | GREEN |
| AC-2-request-count.md | Request counter increments per lookup; test API returns count | `ac_2_request_count_increments_per_cve_lookup` | 1 | GREEN |
| AC-3-unknown-cve-404.md | Unknown CVE returns HTTP 404 with error body | `ac_3_unknown_cve_id_returns_404_not_found` | 1 | GREEN |
| AC-4-unauthenticated-rate-limit.md | 6th unauthenticated request returns HTTP 403 | `ac_4_unauthenticated_rate_limit_403_on_sixth_request` | 1 | GREEN |
| AC-5-authenticated-rate-limit.md | 51st authenticated request returns HTTP 429 | `ac_5_authenticated_rate_limit_429_after_50_requests` | 1 | GREEN |
| AC-6-auth-mode-reject.md | auth_mode=reject: any apiKey request returns 403; unauth unaffected | `ac_6_auth_mode_reject_returns_403_for_any_api_key`, `ac_6_auth_mode_reject_does_not_affect_unauthenticated_requests` | 2 | GREEN |
| AC-7-bulk-fetch-pagination.md | Bulk fetch: page 1/2 correct; EC-001 (cveId precedence); EC-003 (startIndex beyond total) | `ac_7_bulk_fetch_first_page_returns_five_of_ten`, `ac_7_bulk_fetch_second_page_returns_remaining_five`, `ac_7_ec003_start_index_beyond_total_returns_empty`, `ac_7_ec001_cve_id_takes_precedence_over_pagination` | 4 | GREEN |

**Total: 7 AC files / 12 test cases — all GREEN**

## API Summary

Endpoint: `GET /rest/json/cves/2.0` — unified CVE endpoint (NVD API 2.0 contract)

| Query Param | Effect |
|-------------|--------|
| `cveId` | Single CVE lookup; takes precedence over pagination params (EC-001) |
| `apiKey` | Upgrades to authenticated rate-limit bucket (50/30s) |
| `startIndex` | Pagination offset (default 0) |
| `resultsPerPage` | Page size (default 2000; minimum 1 per EC-002) |

Test API endpoints (DTU-internal):
- `GET /dtu/request-count/{cve_id}` — returns `{"cve_id": "...", "count": N}`
- `POST /dtu/configure` — applies JSON config (`auth_mode`, `exhaust_authenticated_bucket`)
- `POST /dtu/reset` — resets all mutable state

Rate limits:
- Unauthenticated: 5 requests / 30s window → HTTP 403 on breach
- Authenticated: 50 requests / 30s window → HTTP 429 on breach
- `auth_mode=reject`: HTTP 403 for any request bearing `apiKey`

Fixture: 10 CVEs (CVE-2024-0001 through CVE-2024-0010), CVSS spectrum 0.0–10.0,
3 CISA KEV entries (CVE-2024-0001, CVE-2024-0006, CVE-2024-0010).

## POL-010 Compliance

All evidence files are under `docs/demo-evidence/S-6.15/` only. No files placed at
`docs/demo-evidence/*.md`, `.factory-demos/`, or `.factory/demo-recordings/`.

## Evidence Files

| File | Purpose |
|------|---------|
| `evidence-report.md` | This aggregator |
| `AC-1-cve-lookup.md` | AC-1 per-AC evidence |
| `AC-2-request-count.md` | AC-2 per-AC evidence |
| `AC-3-unknown-cve-404.md` | AC-3 per-AC evidence |
| `AC-4-unauthenticated-rate-limit.md` | AC-4 per-AC evidence |
| `AC-5-authenticated-rate-limit.md` | AC-5 per-AC evidence |
| `AC-6-auth-mode-reject.md` | AC-6 per-AC evidence (2 sub-tests) |
| `AC-7-bulk-fetch-pagination.md` | AC-7 per-AC evidence (4 sub-tests, EC-001, EC-003) |
| `public-api.md` | NvdClone + BehavioralClone impl; NvdState methods; types; fixture inventory |
| `usage-example.md` | 20-line usage snippet: start, lookup, rate-limit, paginate, configure, reset |
| `test-run.txt` | Full `cargo test --features prism-dtu-nvd/dtu` output |
