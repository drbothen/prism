//! Offset-based hybrid pagination for the Claroty xDome audit_logs endpoint.
//!
//! Claroty's `audit_logs` API does not support cursor-based pagination; instead
//! it uses `?offset=N&limit=PAGE_SIZE` query parameters and returns a
//! `total_count` field in every response. This module provides:
//!
//! - [`OffsetCursor`] — composite `(timestamp, offset)` cursor tracking forward
//!   progress without regression (DI-001).
//! - [`paginate_claroty`] — async `Stream` of result pages, one item per HTTP
//!   response, applying backpressure via the caller-driven pull model.
//!
//! # Architecture Compliance
//! - `paginate_claroty()` returns `impl Stream`, NOT a collected `Vec` — the
//!   caller applies backpressure (BC-2.01.004 Architecture Compliance Rule).
//! - Uses `acquire_http_permit()` from `crate::http` before each HTTP call.
//! - Pagination halts when `offset >= total_count` (BC-2.01.004 postcondition).
//!
//! Story: S-2.07 | BC: BC-2.01.004

use chrono::{DateTime, Utc};
use futures::stream::{self, Stream};

use crate::adapter::SensorError;

// ---------------------------------------------------------------------------
// OffsetCursor
// ---------------------------------------------------------------------------

/// Composite `(timestamp, offset)` cursor for Claroty offset-based pagination.
///
/// Tracks forward progress across paginated fetches without regression (DI-001).
/// The `timestamp` component anchors the cursor to a point in time; the `offset`
/// component counts records advanced past that timestamp.
///
/// # AC-5
/// For a `total_count=500`, `page_size=100` endpoint, a fresh cursor starts at
/// `offset=0` and advances to `100, 200, 300, 400` — exactly 5 HTTP requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OffsetCursor {
    /// Timestamp anchor: the `created_at` of the first record in the current page.
    ///
    /// Used to detect offset drift when new records are inserted during traversal.
    pub timestamp: Option<DateTime<Utc>>,
    /// Current byte/record offset sent as the `?offset=N` query parameter.
    pub offset: usize,
    /// Total record count from the most recent API response `total_count` field.
    ///
    /// Pagination halts when `self.offset >= self.total_count`.
    pub total_count: usize,
    /// Number of records per page (sent as `?limit=N`).
    pub page_size: usize,
}

impl OffsetCursor {
    /// Creates a fresh cursor starting at offset 0.
    ///
    /// `total_count` is set to `usize::MAX` until the first response is received.
    /// GREEN-BY-DESIGN: pure data constructor; no business logic.
    pub fn new(page_size: usize) -> Self {
        Self {
            timestamp: None,
            offset: 0,
            total_count: usize::MAX,
            page_size,
        }
    }

    /// Returns `true` when there are no more pages to fetch.
    ///
    /// Condition: `self.offset >= self.total_count`.
    ///
    /// GREEN-BY-DESIGN: single boolean expression; tested trivially.
    pub fn is_exhausted(&self) -> bool {
        self.offset >= self.total_count
    }

    /// Advances the cursor by one page and updates `total_count`.
    ///
    /// Increments `offset` by `page_size` and stores the `total_count` from
    /// the latest API response. Does NOT allow offset regression (DI-001).
    ///
    /// BC: BC-2.01.004
    pub fn advance(&mut self, total_count: usize, page_timestamp: Option<DateTime<Utc>>) {
        let old_offset = self.offset;
        self.total_count = total_count;
        if let Some(ts) = page_timestamp {
            self.timestamp = Some(ts);
        }
        let new_offset = old_offset.saturating_add(self.page_size);
        // DI-001: never decrease the offset
        if new_offset > old_offset {
            self.offset = new_offset;
        }
    }
}

// ---------------------------------------------------------------------------
// paginate_claroty
// ---------------------------------------------------------------------------

/// Streams pages from a Claroty offset-based API endpoint.
///
/// Yields one `Result<Vec<serde_json::Value>, SensorError>` per HTTP request.
/// The stream halts when the cursor reports `is_exhausted()`.
///
/// Each HTTP call:
/// 1. Acquires an HTTP semaphore permit via `acquire_http_permit()`.
/// 2. Issues `GET {endpoint}?offset={cursor.offset}&limit={cursor.page_size}`.
/// 3. Parses `total_count` from the response JSON.
/// 4. Advances the cursor via `OffsetCursor::advance()`.
///
/// # Arguments
/// - `endpoint` — full URL of the Claroty data endpoint (bearer token
///   authentication is handled by the caller before this function is invoked).
/// - `page_size` — number of records per page (passed as `?limit=N`).
/// - `client` — shared `reqwest::Client` (already configured with bearer auth).
///
/// # AC-5
/// For `total_count=500`, `page_size=100`: yields 5 items (offsets 0–400).
///
/// BC: BC-2.01.004
pub fn paginate_claroty(
    endpoint: String,
    page_size: usize,
    client: reqwest::Client,
) -> impl Stream<Item = Result<Vec<serde_json::Value>, SensorError>> {
    // State: cursor tracks our position through the result set.
    let cursor = OffsetCursor::new(page_size);

    stream::unfold(
        (cursor, endpoint, client, false),
        |(mut cursor, endpoint, client, done)| async move {
            if done || cursor.is_exhausted() {
                return None;
            }

            // Acquire HTTP permit before sending request.
            let _permit = match crate::http::acquire_http_permit().await {
                Ok(p) => p,
                Err(e) => return Some((Err(e), (cursor, endpoint, client, true))),
            };

            let offset = cursor.offset;
            let limit = cursor.page_size;

            let resp = client
                .get(&endpoint)
                .query(&[("offset", offset.to_string()), ("limit", limit.to_string())])
                .send()
                .await;

            let response = match resp {
                Ok(r) => r,
                Err(e) => {
                    let err = SensorError::Internal {
                        detail: format!("paginate_claroty request error: {e}"),
                    };
                    return Some((Err(err), (cursor, endpoint, client, true)));
                }
            };

            let status = response.status();
            if !status.is_success() {
                let code = status.as_u16();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<unreadable>".to_string());
                let err = SensorError::HttpError {
                    sensor: "claroty".to_string(),
                    status: code,
                    body,
                };
                return Some((Err(err), (cursor, endpoint, client, true)));
            }

            let body: serde_json::Value = match response.json().await {
                Ok(v) => v,
                Err(e) => {
                    let err = SensorError::ResponseParse {
                        sensor: "claroty".to_string(),
                        detail: format!("JSON parse error: {e}"),
                    };
                    return Some((Err(err), (cursor, endpoint, client, true)));
                }
            };

            // Extract total_count and data from response.
            let total_count = body
                .get("total_count")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize)
                .unwrap_or(0);

            let records = body
                .get("data")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            // Advance the cursor with the total_count from this response.
            cursor.advance(total_count, None);

            // If total_count was 0, the endpoint has no records.
            // Do not yield a page — return None to end the stream immediately.
            if total_count == 0 {
                return None;
            }

            Some((Ok(records), (cursor, endpoint, client, false)))
        },
    )
}
