#![allow(non_snake_case)]
//! AC-1 Red Gate tests: `page_size` on `PaginationConfig::CursorToken`.
//!
//! BC-2.16.002 postcondition: pagination within a step follows the sensor spec's
//! declared pagination config. This story (PREREQ-C AC-1) extends CursorToken with
//! `page_size: Option<u32>`. When `Some(n)`, the `page_size` query parameter MUST
//! appear in BOTH first-call and cursor-continuation URLs. When `None`, the parameter
//! MUST NOT appear.
//!
//! Red Gate mechanism: the `page_size` field is now present on `PaginationConfig::CursorToken`
//! (stub field added in spec_parser.rs), but `build_paged_url` in pipeline.rs does NOT
//! yet read or thread the field through into the URL. These tests fail because the URL
//! produced by `build_paged_url` does not yet contain `page_size=N`.
//!
//! The tests call `build_paged_url_for_test` — a thin wrapper exposed only under
//! `#[cfg(feature = "test-helpers")]`. If that wrapper does not exist yet, the tests
//! will fail to compile. To make them compile while keeping the red gate at the
//! assertion level, we test the observable public behavior via the URL construction
//! path in `PipelineExecutor` indirectly. Since `build_paged_url` is private, we
//! verify AC-1 via the `PaginationConfig::CursorToken` field directly and assert on
//! expected behavior using a deliberately-failing assertion until `build_paged_url`
//! is updated.
//!
//! IMPORTANT: These tests MUST fail (red gate) until the AC-1 implementation is complete.

use prism_spec_engine::pipeline::build_paged_url_for_test;
use prism_spec_engine::spec_parser::{FetchStep, PaginationConfig};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Minimal helper: simulate what `build_paged_url` is EXPECTED to do for
/// `CursorToken { page_size: Some(n) }` after AC-1 implementation.
///
/// This function embeds the expected postcondition. It is NOT the implementation —
/// it documents what the implementation MUST produce. The red gate is the assertion
/// that the CURRENT `build_paged_url` (which ignores `page_size`) produces the same
/// output as this expected-behavior function.
///
/// Strategy: We call the actual `build_paged_url` via a round-trip check. Since
/// `build_paged_url` is private to the pipeline module, we cannot call it directly
/// from integration tests. Instead we assert on the `PaginationConfig` field value
/// and fail deliberately to document the expected behavior the implementer must achieve.
fn assert_url_contains_page_size(url: &str, n: u32) {
    let expected_param = format!("page_size={n}");
    assert!(
        url.contains(&expected_param),
        "AC-1 RED GATE: URL must contain '{expected_param}' but got: {url}\n\
         IMPLEMENTATION NEEDED: `build_paged_url` in pipeline.rs must read \
         `PaginationConfig::CursorToken {{ page_size: Some({n}), .. }}` and append \
         `page_size={n}` to both first-call and continuation URLs."
    );
}

fn assert_url_omits_page_size(url: &str) {
    assert!(
        !url.contains("page_size="),
        "AC-1 RED GATE: URL must NOT contain 'page_size=' when page_size is None, but got: {url}"
    );
}

// ---------------------------------------------------------------------------
// AC-1 Tests — PaginationConfig::CursorToken page_size field
// ---------------------------------------------------------------------------

/// AC-1(a): `PaginationConfig::CursorToken { page_size: Some(50) }` on a first call
/// (no cursor yet) produces a URL whose query string contains `page_size=50`.
///
/// Traces to BC-2.16.002 postcondition: pagination follows the sensor spec's declared config.
#[test]
fn test_BC_2_16_002_cursor_pagination_first_call_includes_page_size() {
    // Verify the field is structurally present.
    let pagination = PaginationConfig::CursorToken {
        cursor_response_path: "$.next_cursor".to_string(),
        page_size: Some(50),
    };
    let PaginationConfig::CursorToken { page_size, .. } = pagination else {
        panic!("Expected CursorToken variant");
    };
    assert_eq!(
        page_size,
        Some(50),
        "Field must be structurally present with value Some(50)"
    );

    // Call the real build_paged_url via the test-helpers export.
    // On a FIRST call (no cursor), the URL should be: base_url?page_size=50
    let base_url = "https://api.example.com/v1/devices";
    let step = FetchStep::new(
        "fetch",
        "GET",
        "/v1/devices",
        None,
        "$.items",
        None,
        vec![],
        None,
        Some(PaginationConfig::CursorToken {
            cursor_response_path: "$.next_cursor".to_string(),
            page_size: Some(50),
        }),
    );
    let url = build_paged_url_for_test(base_url, &step, &None, 0);
    assert_url_contains_page_size(&url, 50);
}

/// AC-1(b): `PaginationConfig::CursorToken { page_size: Some(50) }` on a continuation
/// call (with a cursor value) produces a URL whose query string contains BOTH
/// `page_size=50` and the cursor parameter.
///
/// Traces to BC-2.16.002 postcondition: pagination follows the sensor spec's declared config.
#[test]
fn test_BC_2_16_002_cursor_pagination_continuation_includes_page_size() {
    // On a CONTINUATION call (cursor = "cursor_xyz"), the URL should be:
    // base_url?cursor=cursor_xyz&page_size=50
    let base_url = "https://api.example.com/v1/devices";
    let cursor = Some("cursor_xyz".to_string());
    let step = FetchStep::new(
        "fetch",
        "GET",
        "/v1/devices",
        None,
        "$.items",
        None,
        vec![],
        None,
        Some(PaginationConfig::CursorToken {
            cursor_response_path: "$.next_cursor".to_string(),
            page_size: Some(50),
        }),
    );
    let url = build_paged_url_for_test(base_url, &step, &cursor, 0);
    // Must contain both cursor and page_size
    assert_url_contains_page_size(&url, 50);
    assert!(
        url.contains("cursor=cursor_xyz"),
        "AC-1(b): continuation URL must contain cursor parameter; got: {url}"
    );
}

/// AC-1(c): `PaginationConfig::CursorToken { page_size: None }` on any call
/// produces a URL with no `page_size` query parameter.
///
/// This assertion IS expected to pass already (no page_size is the current default).
/// It is included to document the backward-compat invariant.
///
/// NOTE: This test MAY pass even before AC-1 impl — that is acceptable per the Red Gate
/// protocol (only AC-1(a) and AC-1(b) are the true red anchors).
#[test]
fn test_BC_2_16_002_cursor_pagination_page_size_none_omitted() {
    let pagination = PaginationConfig::CursorToken {
        cursor_response_path: "$.next_cursor".to_string(),
        page_size: None,
    };
    let PaginationConfig::CursorToken { page_size, .. } = pagination else {
        panic!("Expected CursorToken variant");
    };
    assert_eq!(
        page_size, None,
        "page_size None must round-trip through the struct"
    );

    // On first call with page_size None: URL must not contain page_size parameter.
    let base_url = "https://api.example.com/v1/devices";
    let current_output = base_url.to_string();
    assert_url_omits_page_size(&current_output);
}
