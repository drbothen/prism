//! Multi-step fetch pipeline executor (BC-2.16.002).
//!
//! Steps execute sequentially in spec-declared order. Variables from each step
//! are available to subsequent steps via `${step_name.field}` interpolation.
//! Fan-out: when a variable resolves to an array, the step is batched.
//! Rate limit hints from SensorSpec apply between API calls.
//!
//! NOTE: This implementation is pure domain logic — it does NOT depend on an
//! HTTP client. The full S-1.12 implementation will accept a `dyn HttpClient`
//! injection. Tests use the pure `fan_out_batches` and interpolation paths.

use prism_core::{OrgSlug, PrismError};

use crate::spec_parser::{FetchStep, SensorSpec, TableSpec};

/// Context provided to each pipeline execution.
#[derive(Debug, Clone)]
pub struct FetchContext {
    /// The client/tenant this query is executing for.
    pub client_id: OrgSlug,
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
    ///
    /// This stub returns the table name and empty records — full HTTP implementation
    /// comes in S-1.12 which injects an HTTP client trait.
    pub async fn execute(
        _spec: &SensorSpec,
        table: &TableSpec,
        _context: &FetchContext,
    ) -> Result<PipelineResult, PrismError> {
        // Stub: full HTTP execution in S-1.12. Returns success with empty records
        // so the test can `drop(result)` without panicking.
        Ok(PipelineResult {
            records: Vec::new(),
            table_name: table.table_name.clone(),
            request_count: 0,
        })
    }

    /// Execute a single fetch step, given resolved variables from prior steps.
    ///
    /// Returns the raw JSON response body for the step.
    /// Full implementation requires an HTTP client injected from outside.
    pub async fn execute_step(
        _step: &FetchStep,
        _spec: &SensorSpec,
        _prior_vars: &std::collections::HashMap<String, serde_json::Value>,
        _context: &FetchContext,
    ) -> Result<serde_json::Value, PrismError> {
        // Stub: full HTTP execution in S-1.12.
        Ok(serde_json::Value::Null)
    }

    /// Resolve and expand fan-out: if a variable resolves to an array, return
    /// batches of `batch_size` items each (BC-2.16.002 Fan-Out Behavior).
    ///
    /// - Array input: batches of up to `batch_size` elements each.
    /// - Scalar input: single batch containing that one value.
    /// - Empty array: zero batches.
    pub fn fan_out_batches(
        values: &serde_json::Value,
        batch_size: usize,
    ) -> Vec<Vec<serde_json::Value>> {
        match values {
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    return Vec::new();
                }
                arr.chunks(batch_size).map(|chunk| chunk.to_vec()).collect()
            }
            scalar => {
                // Non-array: single batch of one item.
                vec![vec![scalar.clone()]]
            }
        }
    }
}
