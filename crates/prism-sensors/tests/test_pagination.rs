//! Tests for OffsetCursor + paginate_claroty() stream.
//!
//! Covers BC-2.01.004:
//! - OffsetCursor construction (GREEN-BY-DESIGN)
//! - OffsetCursor::is_exhausted (GREEN-BY-DESIGN)
//! - OffsetCursor::advance (RED — todo!())
//! - paginate_claroty stream: AC-5 exact 5 requests for total_count=500, page_size=100
//! - Cursor invariant DI-001: offset never regresses
//! - Empty page halts pagination (EC-01-005)
//!
//! Story: S-2.07 | BC: BC-2.01.004
#![allow(clippy::expect_used, clippy::unwrap_used)]

use futures::StreamExt;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use prism_sensors::pagination::{paginate_claroty, OffsetCursor};

// ---------------------------------------------------------------------------
// OffsetCursor construction — GREEN-BY-DESIGN
// ---------------------------------------------------------------------------

/// GREEN-BY-DESIGN: OffsetCursor::new starts at offset=0, total_count=usize::MAX.
///
/// BC-2.01.004 precondition: fresh cursor has offset=0.
#[test]
fn test_BC_2_01_004_offset_cursor_new_starts_at_zero() {
    let cursor = OffsetCursor::new(100);
    assert_eq!(cursor.offset, 0, "fresh cursor offset must be 0");
    assert_eq!(
        cursor.page_size, 100,
        "page_size must match constructor arg"
    );
    assert_eq!(
        cursor.total_count,
        usize::MAX,
        "total_count must start at usize::MAX (sentinel for 'not yet received')"
    );
    assert!(cursor.timestamp.is_none(), "timestamp starts None");
}

/// GREEN-BY-DESIGN: OffsetCursor::is_exhausted returns false on a fresh cursor.
///
/// BC-2.01.004: offset < total_count → not exhausted.
#[test]
fn test_BC_2_01_004_offset_cursor_is_not_exhausted_when_fresh() {
    let cursor = OffsetCursor::new(100);
    assert!(
        !cursor.is_exhausted(),
        "fresh cursor must not be exhausted (offset=0 < total_count=MAX)"
    );
}

/// GREEN-BY-DESIGN: OffsetCursor::is_exhausted returns true when offset == total_count.
///
/// BC-2.01.004 postcondition: "pagination halts when offset >= total_count".
#[test]
fn test_BC_2_01_004_offset_cursor_is_exhausted_when_offset_equals_total() {
    let mut cursor = OffsetCursor::new(100);
    cursor.total_count = 500;
    cursor.offset = 500;
    assert!(
        cursor.is_exhausted(),
        "cursor with offset == total_count must be exhausted"
    );
}

/// GREEN-BY-DESIGN: OffsetCursor::is_exhausted returns true when offset > total_count.
///
/// BC-2.01.004 invariant DI-001 boundary: offset may overshoot by page_size on
/// the final advance.
#[test]
fn test_BC_2_01_004_offset_cursor_is_exhausted_when_offset_exceeds_total() {
    let mut cursor = OffsetCursor::new(100);
    cursor.total_count = 450;
    cursor.offset = 500; // overshot by 50
    assert!(
        cursor.is_exhausted(),
        "cursor with offset > total_count must be exhausted"
    );
}

// ---------------------------------------------------------------------------
// OffsetCursor::advance
// ---------------------------------------------------------------------------

/// BC-2.01.004: advance() increments offset by page_size.
///

#[test]
fn test_BC_2_01_004_offset_cursor_advance_increments_offset_by_page_size() {
    let mut cursor = OffsetCursor::new(100);
    cursor.advance(500, None);
    assert_eq!(
        cursor.offset, 100,
        "advance() must increment offset by page_size (100)"
    );
}

/// BC-2.01.004: advance() updates total_count from API response.
///

#[test]
fn test_BC_2_01_004_offset_cursor_advance_updates_total_count() {
    let mut cursor = OffsetCursor::new(100);
    cursor.advance(500, None);
    assert_eq!(
        cursor.total_count, 500,
        "advance() must update total_count from API response"
    );
}

/// BC-2.01.004 invariant DI-001: advance() called multiple times never decreases offset.
///

#[test]
fn test_BC_2_01_004_invariant_cursor_offset_never_regresses() {
    let mut cursor = OffsetCursor::new(100);
    let mut last_offset = cursor.offset;
    for _ in 0..5 {
        cursor.advance(500, None);
        assert!(
            cursor.offset > last_offset || cursor.is_exhausted(),
            "DI-001: cursor offset must never decrease; was {last_offset}, now {}",
            cursor.offset
        );
        last_offset = cursor.offset;
    }
}

/// BC-2.01.004: After 5 advances on total_count=500/page_size=100, cursor is exhausted.
///

#[test]
fn test_BC_2_01_004_cursor_exhausted_after_5_advances_for_500_total() {
    let mut cursor = OffsetCursor::new(100);
    for _ in 0..5 {
        cursor.advance(500, None);
    }
    assert!(
        cursor.is_exhausted(),
        "cursor must be exhausted after 5 × 100 advances on total_count=500 (AC-5)"
    );
}

/// BC-2.01.004: advance() with timestamp updates the timestamp anchor.
///

#[test]
fn test_BC_2_01_004_offset_cursor_advance_stores_page_timestamp() {
    use chrono::Utc;
    let mut cursor = OffsetCursor::new(100);
    let ts = Utc::now();
    cursor.advance(500, Some(ts));
    assert_eq!(
        cursor.timestamp,
        Some(ts),
        "advance() must store the page timestamp anchor"
    );
}

// ---------------------------------------------------------------------------
// paginate_claroty stream — AC-5 (5 requests for total_count=500, page_size=100)
// ---------------------------------------------------------------------------

/// AC-5 / TV-BC-2.01.004-001: paginate_claroty yields exactly 5 pages for
/// total_count=500, page_size=100 (offsets 0, 100, 200, 300, 400).
#[tokio::test]
async fn test_BC_2_01_004_paginate_claroty_five_pages_for_500_total() {
    // Set up a wiremock server to handle paginated requests.
    let server = MockServer::start().await;

    // Register mock responses for each of the 5 expected offsets.
    for offset in [0usize, 100, 200, 300, 400] {
        let records: Vec<serde_json::Value> = (0..100)
            .map(|i| serde_json::json!({ "id": offset + i, "event": "test" }))
            .collect();
        let body = serde_json::json!({
            "total_count": 500,
            "data": records,
        });
        Mock::given(method("GET"))
            .and(path("/audit_logs"))
            .and(query_param("offset", offset.to_string().as_str()))
            .and(query_param("limit", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .expect(1) // each offset called exactly once (AC-5)
            .mount(&server)
            .await;
    }

    let endpoint = format!("{}/audit_logs", server.uri());
    let client = reqwest::Client::new();
    let stream = paginate_claroty(endpoint, 100, client);

    let pages: Vec<_> = stream.collect().await;

    assert_eq!(
        pages.len(),
        5,
        "AC-5: paginate_claroty must yield exactly 5 pages for total_count=500, page_size=100"
    );

    let mut total_records = 0usize;
    for page in &pages {
        let records = page.as_ref().expect("each page must be Ok");
        total_records += records.len();
    }
    assert_eq!(
        total_records, 500,
        "AC-5: total records across all 5 pages must be 500"
    );
}

/// TV-BC-2.01.004-002 / EC-01-005: When offset equals total_count, pagination halts.
///
/// A response with 0 records and total_count=0 must yield 0 pages.
#[tokio::test]
async fn test_BC_2_01_004_paginate_claroty_halts_when_offset_equals_total() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/audit_logs"))
        .and(query_param("offset", "0"))
        .and(query_param("limit", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total_count": 0,
            "data": [],
        })))
        .expect(1)
        .mount(&server)
        .await;

    let endpoint = format!("{}/audit_logs", server.uri());
    let client = reqwest::Client::new();
    let stream = paginate_claroty(endpoint, 100, client);
    let pages: Vec<_> = stream.collect().await;

    assert_eq!(
        pages.len(),
        0,
        "TV-BC-2.01.004-002: empty total_count must halt immediately"
    );
}

/// TV-BC-2.01.004-003: HTTP 400 from Claroty returns a SensorError in the stream.
///
/// BC-2.01.004 error case: invalid offset → SensorError with category api_contract.
#[tokio::test]
async fn test_BC_2_01_004_paginate_claroty_http_400_yields_sensor_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/audit_logs"))
        .respond_with(ResponseTemplate::new(400).set_body_string("invalid offset"))
        .mount(&server)
        .await;

    let endpoint = format!("{}/audit_logs", server.uri());
    let client = reqwest::Client::new();
    let stream = paginate_claroty(endpoint, 100, client);
    let pages: Vec<_> = stream.collect().await;

    assert_eq!(pages.len(), 1, "One error item expected in the stream");
    assert!(
        pages[0].is_err(),
        "TV-BC-2.01.004-003: HTTP 400 must yield Err in stream"
    );
}
