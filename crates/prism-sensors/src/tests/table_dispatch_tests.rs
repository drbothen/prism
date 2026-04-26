//! Tests for `table_dispatch::route_table_query` — table-type routing logic.
//!
//! Story: S-2.08 | AC-2, AC-3, AC-5, AC-8
//!
//! # Coverage
//! - TableTypeRouteDecision variants exist and compare correctly (GREEN-BY-DESIGN)
//! - PointInTime → LiveFetch regardless of has_buffer_data (AC-3)
//! - EventStream + has_data=true → BufferScan (AC-2)
//! - EventStream + has_data=false → ColdStartFallback (AC-5, EC-001)
//!
//! # Status
//! All tests pass (S-2.08 implementation complete).

use prism_core::TableType;

use crate::table_dispatch::{route_table_query, TableTypeRouteDecision};

// ---------------------------------------------------------------------------
// TableTypeRouteDecision variants — GREEN-BY-DESIGN
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_route_decision_live_fetch_variant() {
    // GREEN-BY-DESIGN: variant existence
    assert_eq!(
        TableTypeRouteDecision::LiveFetch,
        TableTypeRouteDecision::LiveFetch
    );
}

#[test]
fn test_BC_2_08_route_decision_buffer_scan_variant() {
    // GREEN-BY-DESIGN: variant existence
    assert_eq!(
        TableTypeRouteDecision::BufferScan,
        TableTypeRouteDecision::BufferScan
    );
}

#[test]
fn test_BC_2_08_route_decision_cold_start_fallback_variant() {
    // GREEN-BY-DESIGN: variant existence
    assert_eq!(
        TableTypeRouteDecision::ColdStartFallback,
        TableTypeRouteDecision::ColdStartFallback
    );
}

#[test]
fn test_BC_2_08_route_decision_variants_not_equal() {
    // GREEN-BY-DESIGN
    assert_ne!(
        TableTypeRouteDecision::LiveFetch,
        TableTypeRouteDecision::BufferScan
    );
    assert_ne!(
        TableTypeRouteDecision::LiveFetch,
        TableTypeRouteDecision::ColdStartFallback
    );
    assert_ne!(
        TableTypeRouteDecision::BufferScan,
        TableTypeRouteDecision::ColdStartFallback
    );
}

// ---------------------------------------------------------------------------
// route_table_query — AC-2, AC-3, AC-5
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_route_table_query_point_in_time_no_data_returns_live_fetch() {
    // AC-3: PointInTime → LiveFetch always
    let decision = route_table_query(TableType::PointInTime, false);
    assert_eq!(
        decision,
        TableTypeRouteDecision::LiveFetch,
        "AC-3: PointInTime + has_data=false must return LiveFetch"
    );
}

#[test]
fn test_BC_2_08_route_table_query_point_in_time_has_data_returns_live_fetch() {
    // AC-3: PointInTime ALWAYS → LiveFetch; has_buffer_data is irrelevant
    let decision = route_table_query(TableType::PointInTime, true);
    assert_eq!(
        decision,
        TableTypeRouteDecision::LiveFetch,
        "AC-3: PointInTime + has_data=true must still return LiveFetch (has_data is irrelevant)"
    );
}

#[test]
fn test_BC_2_08_route_table_query_event_stream_with_data_returns_buffer_scan() {
    // AC-2: EventStream + has_data=true → BufferScan (serve from local RocksDB)
    let decision = route_table_query(TableType::EventStream, true);
    assert_eq!(
        decision,
        TableTypeRouteDecision::BufferScan,
        "AC-2: EventStream + has_data=true must return BufferScan"
    );
}

#[test]
fn test_BC_2_08_route_table_query_event_stream_no_data_returns_cold_start_fallback() {
    // AC-5, EC-001: EventStream + has_data=false → ColdStartFallback
    let decision = route_table_query(TableType::EventStream, false);
    assert_eq!(
        decision,
        TableTypeRouteDecision::ColdStartFallback,
        "AC-5: EventStream + has_data=false must return ColdStartFallback (EC-001)"
    );
}
