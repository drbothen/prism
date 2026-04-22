//! Multi-step fetch pipeline executor (BC-2.16.002).
//!
//! Steps execute sequentially in spec-declared order. Variables from each step
//! are available to subsequent steps via `${step_name.field}` interpolation.
//! Fan-out: when a variable resolves to an array, the step is batched.
//! Rate limit hints from SensorSpec apply between API calls.
//!
//! NOTE: This stub does NOT depend on an HTTP client — the full implementation
//! will accept a `dyn HttpClient` injection. Tests mock via trait objects.

use prism_core::{PrismError, TenantId};

use crate::spec_parser::{FetchStep, SensorSpec, TableSpec};

/// Context provided to each pipeline execution.
#[derive(Debug, Clone)]
pub struct FetchContext {
    /// The client/tenant this query is executing for.
    pub client_id: TenantId,
    /// Push-down filter values from the query planner (${query.filter.*}).
    pub query_filters: std::collections::HashMap<String, String>,
}

/// The output of a successful pipeline execution.
///
/// Contains the raw JSON records from the final step. OCSF mapping (BC-2.16.003)
/// is applied by `ColumnMapper` separately.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// Raw records from the final step, as JSON values.
    pub records: Vec<serde_json::Value>,
    /// Name of the table spec that was fetched.
    pub table_name: String,
    /// Total number of API requests made (for rate limit tracking).
    pub request_count: u32,
}

/// Executes a multi-step fetch pipeline for a sensor table (BC-2.16.002).
///
/// All methods are `unimplemented!()` — implemented in S-1.11.
pub struct PipelineExecutor;

impl PipelineExecutor {
    /// Execute all steps of a table's fetch pipeline sequentially.
    ///
    /// - Steps run in spec-declared order (invariant: no parallel execution).
    /// - Variables from step N are available to steps N+1, N+2, ... but not prior.
    /// - Fan-out: if a variable resolves to an array, the step batches requests
    ///   (default batch_size = 100 per FetchStep configuration).
    /// - Rate limit hints apply between each API call.
    /// - The 10K materialization limit (DI-019) applies to the final collected records.
    pub async fn execute(
        spec: &SensorSpec,
        table: &TableSpec,
        context: &FetchContext,
    ) -> Result<PipelineResult, PrismError> {
        unimplemented!("PipelineExecutor::execute — implement in S-1.11 (BC-2.16.002)")
    }

    /// Execute a single fetch step, given resolved variables from prior steps.
    ///
    /// Returns the raw JSON response body for the step.
    pub async fn execute_step(
        step: &FetchStep,
        spec: &SensorSpec,
        prior_vars: &std::collections::HashMap<String, serde_json::Value>,
        context: &FetchContext,
    ) -> Result<serde_json::Value, PrismError> {
        unimplemented!("PipelineExecutor::execute_step — implement in S-1.11 (BC-2.16.002)")
    }

    /// Resolve and expand fan-out: if a variable resolves to an array, return
    /// batches of `batch_size` items each (BC-2.16.002 Fan-Out Behavior).
    pub fn fan_out_batches(
        values: &serde_json::Value,
        batch_size: usize,
    ) -> Vec<Vec<serde_json::Value>> {
        unimplemented!("PipelineExecutor::fan_out_batches — implement in S-1.11 (BC-2.16.002)")
    }
}
