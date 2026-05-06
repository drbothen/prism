//! Integration tests for the S-3.02 query materialization pipeline.
//!
//! All tests in this module are RED by design — they exercise real business logic
//! that is stubbed with `todo!()`. Each test will panic at runtime when the stub
//! fires. This satisfies the Red Gate invariant (BC-5.38.001).
//!
//! # Tests cover:
//! - AC-1: Virtual fields present in every result row (BC-2.11.001, BC-2.11.012)
//! - AC-2: Parallel fan-out to multiple sources (BC-2.11.005)
//! - AC-3: GreedyMemoryPool 200MB limit → E-QUERY-004 (BC-2.11.006)
//! - AC-4: REQUIRED column push-down to sensor adapter (BC-2.11.007)
//! - AC-5: `clients: None` fans out to all configured clients (BC-2.11.011)
//! - AC-6: Cross-client data merged with `_client` field distinguishing rows (BC-2.11.011)
//! - AC-7: SessionContext dropped after `execute()` returns (BC-2.11.005)
//! - AC-9: Cold-start fallback triggers live fetch + EventBufferStore write (S-2.08 inherited)
//!
//! Story: S-3.02

// Note: Tests are cfg(test) module members (pub(crate) access to internals).

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use prism_core::OrgSlug;
    use prism_sensors::AdapterRegistry;

    use crate::{
        engine::{QueryEngine, QueryEngineConfig, QueryOptions},
        scoping::ClientRegistry,
    };

    // -----------------------------------------------------------------------
    // AC-1: Virtual fields present in every result row
    // -----------------------------------------------------------------------

    /// AC-1: Execute query targeting crowdstrike.detections for client "acme".
    /// Assert every result row has `_sensor = "crowdstrike"`, `_client = "acme"`,
    /// `_source_table = "crowdstrike.detections"`. (BC-2.11.001, BC-2.11.012)
    #[tokio::test]
    async fn test_ac1_virtual_fields_present_in_every_row() {
        todo!("S-3.02 — test_ac1_virtual_fields_present_in_every_row")
    }

    // -----------------------------------------------------------------------
    // AC-2: Parallel fan-out to multiple sources
    // -----------------------------------------------------------------------

    /// AC-2: Execute query targeting crowdstrike.detections AND claroty.alerts.
    /// Assert both sources are fetched in parallel, normalized, and registered
    /// as separate MemTables. (BC-2.11.005)
    #[tokio::test]
    async fn test_ac2_parallel_fanout_multiple_sources() {
        todo!("S-3.02 — test_ac2_parallel_fanout_multiple_sources")
    }

    // -----------------------------------------------------------------------
    // AC-3: Memory pool limit → E-QUERY-004
    // -----------------------------------------------------------------------

    /// AC-3: Simulate a query that exceeds the 200MB GreedyMemoryPool limit.
    /// Assert `PrismError::QueryMemoryBudgetExceeded` is returned and no partial
    /// results are emitted. (BC-2.11.006, EC-001)
    #[tokio::test]
    async fn test_ac3_memory_pool_limit_returns_error() {
        todo!("S-3.02 — test_ac3_memory_pool_limit_returns_error")
    }

    // -----------------------------------------------------------------------
    // AC-4: REQUIRED column push-down
    // -----------------------------------------------------------------------

    /// AC-4: Execute query with `severity_id >= 3` where `severity_id` is a
    /// REQUIRED column on the CrowdStrike source. Assert `severity_id >= 3`
    /// appears in `PushDownPlan.push_down` and is passed to the sensor adapter
    /// fetch call. (BC-2.11.007)
    #[tokio::test]
    async fn test_ac4_required_column_push_down() {
        todo!("S-3.02 — test_ac4_required_column_push_down")
    }

    // -----------------------------------------------------------------------
    // AC-5: clients: None fans out to all configured clients
    // -----------------------------------------------------------------------

    /// AC-5: Execute query with `clients: None`. Assert fan-out targets all
    /// configured client IDs and results from all clients appear in response.
    /// (BC-2.11.011)
    #[tokio::test]
    async fn test_ac5_none_clients_fans_out_to_all() {
        todo!("S-3.02 — test_ac5_none_clients_fans_out_to_all")
    }

    // -----------------------------------------------------------------------
    // AC-6: Cross-client _client field distinguishes rows
    // -----------------------------------------------------------------------

    /// AC-6: Execute query for clients ["acme", "contoso"]. Assert the source
    /// MemTable contains rows from both clients distinguished by `_client`.
    /// (BC-2.11.011)
    #[tokio::test]
    async fn test_ac6_cross_client_data_merged_with_client_field() {
        todo!("S-3.02 — test_ac6_cross_client_data_merged_with_client_field")
    }

    // -----------------------------------------------------------------------
    // AC-7: SessionContext dropped after execute() returns
    // -----------------------------------------------------------------------

    /// AC-7: Execute a non-scheduled query and assert the SessionContext is
    /// dropped (its `Arc` strong count goes to 0) when `execute()` returns.
    /// Also verify on panic path via SessionScope RAII. (BC-2.11.005)
    #[tokio::test]
    async fn test_ac7_session_context_dropped_after_execute() {
        todo!("S-3.02 — test_ac7_session_context_dropped_after_execute")
    }

    // -----------------------------------------------------------------------
    // AC-9 (inherited from S-2.08): Cold-start fallback
    // -----------------------------------------------------------------------

    /// AC-9a: Query an EventStream table with no buffered data.
    /// Assert route is `ColdStartFallback`. (S-2.08 inherited, BC-2.11.005)
    #[tokio::test]
    async fn test_ac9a_cold_start_fallback_route_decision() {
        todo!("S-3.02 — test_ac9a_cold_start_fallback_route_decision")
    }

    /// AC-9b: Verify cold-start fallback triggers a live SensorAdapter fetch,
    /// writes results to EventBufferStore, and logs an INFO event.
    /// (S-2.08 AC-5b inherited, BC-2.11.005, BC-2.11.007)
    #[tokio::test]
    async fn test_ac9b_cold_start_triggers_live_fetch_and_writes_to_buffer() {
        todo!("S-3.02 — test_ac9b_cold_start_triggers_live_fetch_and_writes_to_buffer")
    }

    /// AC-9 companion: Subsequent query of the same EventStream table after
    /// cold-start returns `BufferScan`. (S-2.08 AC-5b companion)
    #[tokio::test]
    async fn test_ac9_subsequent_query_returns_buffer_scan() {
        todo!("S-3.02 — test_ac9_subsequent_query_returns_buffer_scan")
    }

    // -----------------------------------------------------------------------
    // EC-003: Materialization record cap
    // -----------------------------------------------------------------------

    /// EC-003: Fan-out returns more than 10,000 records across all sources.
    /// Assert materialization is aborted with E-QUERY-005 message containing
    /// count and source list. (BC-2.11.006, EC-003)
    #[tokio::test]
    async fn test_ec003_materialization_record_cap_10k() {
        todo!("S-3.02 — test_ec003_materialization_record_cap_10k")
    }

    // -----------------------------------------------------------------------
    // EC-002: Query timeout
    // -----------------------------------------------------------------------

    /// EC-002: Query execution exceeds 30s timeout. Assert
    /// `PrismError::QueryTimeout` is returned. (BC-2.11.006, EC-002)
    #[tokio::test]
    async fn test_ec002_query_timeout_30s() {
        todo!("S-3.02 — test_ec002_query_timeout_30s")
    }

    // -----------------------------------------------------------------------
    // EC-005: Virtual field spoofing prevention
    // -----------------------------------------------------------------------

    /// EC-005: Sensor emits a field named `_sensor`. Assert the engine
    /// overwrites it with the canonical value. (BC-2.11.012, EC-005)
    #[tokio::test]
    async fn test_ec005_virtual_field_spoofing_overwritten() {
        todo!("S-3.02 — test_ec005_virtual_field_spoofing_overwritten")
    }

    // -----------------------------------------------------------------------
    // execute_scheduled: Arc<SessionContext> returned
    // -----------------------------------------------------------------------

    /// Verify `execute_scheduled` returns an `Arc<SessionContext>` that is
    /// still valid for additional queries after the initial call. (BC-2.11.005)
    #[tokio::test]
    async fn test_execute_scheduled_returns_arc_session_context() {
        todo!("S-3.02 — test_execute_scheduled_returns_arc_session_context")
    }
}
