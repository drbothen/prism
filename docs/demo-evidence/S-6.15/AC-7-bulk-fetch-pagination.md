# AC-7: Bulk CVE Fetch with Pagination (includes EC-001 and EC-003)

## AC Statement

Given `GET /rest/json/cves/2.0` with `startIndex=0&resultsPerPage=5`, then the response
contains `resultsPerPage: 5`, `totalResults: 10`, and 5 CVE entries.

Also covers:
- EC-001: Both `cveId` and `startIndex` present — `cveId` takes precedence; single CVE
  lookup behavior.
- EC-003: `startIndex` beyond total results — returns empty `vulnerabilities` array with
  correct `totalResults`.

## Test File

`crates/prism-dtu-nvd/tests/ac_7_bulk_fetch_pagination.rs`

## Test Functions

- `ac_7_bulk_fetch_first_page_returns_five_of_ten` — page 1: startIndex=0, resultsPerPage=5
- `ac_7_bulk_fetch_second_page_returns_remaining_five` — page 2: startIndex=5, resultsPerPage=5
- `ac_7_ec003_start_index_beyond_total_returns_empty` — startIndex=100 returns empty array
- `ac_7_ec001_cve_id_takes_precedence_over_pagination` — cveId + startIndex returns single result

## Implementation Excerpt

`crates/prism-dtu-nvd/src/routes/cves.rs` — `handle_bulk_fetch`:

```rust
async fn handle_bulk_fetch(
    state: &NvdState,
    start_index: Option<u32>,
    results_per_page: Option<u32>,
) -> impl IntoResponse {
    let start = start_index.unwrap_or(0) as usize;
    // EC-002: resultsPerPage=0 treated as 1.
    let page_size = results_per_page.map(|r| r.max(1)).unwrap_or(2000) as usize;

    let mut all_cves: Vec<_> = state.cve_registry.values().cloned().collect();
    all_cves.sort_by(|a, b| a.id.cmp(&b.id));
    let total = all_cves.len();

    // EC-003: startIndex beyond total returns empty array.
    let page: Vec<VulnerabilityWrapper> = if start >= total {
        vec![]
    } else {
        all_cves.into_iter().skip(start).take(page_size)
            .map(|cve| VulnerabilityWrapper { cve })
            .collect()
    };
    // ...
}
```

The `cveId`-takes-precedence dispatch is in `get_cves`:

```rust
// cveId takes precedence over pagination (EC-001).
if let Some(cve_id) = params.cve_id {
    return handle_single_cve(&state, &cve_id).await.into_response();
}
```

## Test Run Output

```
Running tests/ac_7_bulk_fetch_pagination.rs

running 4 tests
test ac_7_ec003_start_index_beyond_total_returns_empty ... ok
test ac_7_ec001_cve_id_takes_precedence_over_pagination ... ok
test ac_7_bulk_fetch_first_page_returns_five_of_ten ... ok
test ac_7_bulk_fetch_second_page_returns_remaining_five ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

## Mapping

AC-7 (all 4 sub-tests including EC-001 and EC-003) is satisfied: `handle_bulk_fetch`
paginates the 10-CVE fixture set deterministically (sorted by ID), returning correct
`totalResults`, `resultsPerPage`, and `startIndex` in every case; edge cases EC-001 and
EC-003 are exercised explicitly.
