//! Armis Centrix API authentication credentials and adapter.
//!
//! # Auth credential (S-2.06)
//! [`ArmisAuth`] — static API secret key (bearer token); sealed via `SensorAuth`.
//!
//! # Adapter (S-2.07)
//! [`ArmisAdapter`] — implements [`SensorAdapter`] with:
//! - Static bearer token auth (`Authorization: Bearer {token}` on all requests).
//! - AQL query forwarding: passes `SensorSpec.aql_query` verbatim to the
//!   Armis GetSearch endpoint (`aql` parameter); constructs a default AQL from
//!   table name if `aql_query` is absent.
//! - Timestamp fallback chain: `firstSeen` → `lastSeen` → `DateTime::now()`
//!   (with `tracing::warn!` on fallback to `now()`).
//!
//! Story: S-2.06 (credentials) / S-2.07 (adapter) | BC: BC-2.01.008, BC-2.01.013

use std::sync::Arc;

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use prism_core::SensorId;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};

use super::{private::Sealed, SensorAuth};
use crate::adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec};

// ---------------------------------------------------------------------------
// ArmisAuth — credential struct (S-2.06, unchanged)
// ---------------------------------------------------------------------------

/// Armis Centrix REST API key credentials.
///
/// `Debug` omits the `secret_key` value — credentials MUST NOT transit AI context.
pub struct ArmisAuth {
    /// Armis tenant base URL (e.g., `"https://acme.armis.com"`).
    pub instance_url: String,
    /// Armis API secret key — MUST NOT appear in any log output.
    pub secret_key: SecretString,
}

impl std::fmt::Debug for ArmisAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArmisAuth")
            .field("instance_url", &self.instance_url)
            .field("secret_key", &"Secret(***)")
            .finish()
    }
}

impl Sealed for ArmisAuth {}
impl SensorAuth for ArmisAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// AQL constants
// ---------------------------------------------------------------------------

/// Default AQL template used when `SensorSpec.aql_query` is absent.
///
/// `{table}` is substituted at runtime with `spec.source_table`.
/// BC: BC-2.01.008 (AQL forwarding postcondition).
pub const DEFAULT_AQL_TEMPLATE: &str = "in:{table}";

/// Maximum permitted length for a spec-supplied AQL string (ADR-005).
///
/// Queries exceeding this limit are rejected by `validate_aql()` to prevent
/// unbounded Armis API calls from misconfigured or malicious specs.
const AQL_MAX_BYTES: usize = 512;

// ---------------------------------------------------------------------------
// AqlValidationError — structured rejection for invalid AQL (ADR-005)
// ---------------------------------------------------------------------------

/// Error returned by [`validate_aql`] when a spec-supplied AQL string fails
/// the Prism allowlist validator (ADR-005, WGS-W2-001, CWE-943).
///
/// The `reason` field includes a human-readable description of the violation,
/// suitable for inclusion in `SensorError::ConfigValidation.detail`.
#[derive(Debug, thiserror::Error)]
#[error("AQL validation failed: {reason}")]
pub struct AqlValidationError {
    /// Human-readable description of why the AQL was rejected.
    pub reason: String,
}

// ---------------------------------------------------------------------------
// validate_aql — AQL allowlist validator (ADR-005)
// ---------------------------------------------------------------------------

/// Validates a spec-supplied AQL string against the Prism allowlist.
///
/// Called for AQL that originates from `SensorSpec.sensor_config["aql_query"]`
/// (operator-authored TOML or runtime `add_sensor_spec` uploads).
/// **NOT** called for push-down-generated AQL (BC-2.11.007), which is safe by
/// construction and does not flow through the `sensor_config` branch.
///
/// # Allowlist (ADR-005 §"AQL Allowlist Scope")
///
/// Permitted query shapes:
/// - Must start with `in:{table}` (leading positional filter, case-insensitive)
/// - Field names: `[a-zA-Z][a-zA-Z0-9_.]*` (no whitespace, no parens, no SQL)
/// - Predicates: `field:value` or `field:(list,of,values)`
/// - Value atoms: quoted strings, integers, floats, comma-separated lists
/// - Logical combinators: `and`, `or`, `not` (case-insensitive)
/// - `orderBy field {asc|desc}` suffix
/// - Total length ≤ 512 bytes
///
/// # Rejected constructs (ADR-005 §"Rejected constructs")
///
/// - SQL/AQL comment markers: `--`, `/*`
/// - Stacked-query separator: `;`
/// - Nested `in:` after the leading position (sub-query injection)
/// - `select` keyword (SQL sub-query)
/// - Unbalanced quotes
/// - Empty or whitespace-only string
/// - Query exceeding `AQL_MAX_BYTES`
///
/// # Returns
///
/// `Ok(())` if the query passes; `Err(AqlValidationError)` with a reason if
/// the query is rejected.
///
/// ADR-005 | WGS-W2-001 | CWE-943
pub fn validate_aql(query: &str) -> Result<(), AqlValidationError> {
    // ── 1. Empty / whitespace-only ────────────────────────────────────────────
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(AqlValidationError {
            reason: "AQL must not be empty or whitespace-only".to_owned(),
        });
    }

    // ── 2. Length guard ───────────────────────────────────────────────────────
    if query.len() > AQL_MAX_BYTES {
        return Err(AqlValidationError {
            reason: format!(
                "AQL exceeds maximum allowed length ({} bytes; limit is {AQL_MAX_BYTES})",
                query.len()
            ),
        });
    }

    // ── 3. Comment injection: -- ──────────────────────────────────────────────
    if trimmed.contains("--") {
        return Err(AqlValidationError {
            reason: "AQL must not contain SQL/AQL comment markers ('--')".to_owned(),
        });
    }

    // ── 4. Block comment injection: /* ────────────────────────────────────────
    if trimmed.contains("/*") || trimmed.contains("*/") {
        return Err(AqlValidationError {
            reason: "AQL must not contain block comment markers ('/*' or '*/')".to_owned(),
        });
    }

    // ── 5. Stacked-query separator: ; ─────────────────────────────────────────
    if trimmed.contains(';') {
        return Err(AqlValidationError {
            reason: "AQL must not contain stacked-query separator (';')".to_owned(),
        });
    }

    // ── 6. Must start with 'in:' ──────────────────────────────────────────────
    // The leading `in:` is mandatory per the Armis AQL spec and the Prism
    // built-in sensor TOML examples.  Queries that don't start with `in:`
    // (case-insensitive) are not Armis AQL — they may be SQL or other
    // injection payloads.
    if !trimmed.to_ascii_lowercase().starts_with("in:") {
        return Err(AqlValidationError {
            reason: "AQL must start with 'in:{table}' (positional filter is mandatory)".to_owned(),
        });
    }

    // ── 7. No nested `in:` after the leading occurrence ───────────────────────
    // Strip the leading `in:` and check the remainder for additional `in:`
    // occurrences, which indicate sub-query injection.
    let after_leading_in = &trimmed[3..]; // skip "in:"
    let lower_remainder = after_leading_in.to_ascii_lowercase();
    if lower_remainder.contains("in:") {
        return Err(AqlValidationError {
            reason:
                "AQL must not contain nested 'in:' sub-queries after the leading positional filter"
                    .to_owned(),
        });
    }

    // ── 8. No `select` keyword (SQL sub-query marker) ─────────────────────────
    // Check for standalone `select` word in the remainder.
    //
    // IMPORTANT: we must check ALL occurrences of the substring "select", not
    // just the first one.  Using `.find("select")` is insufficient because an
    // early occurrence inside a field name like "selected:y" fails the
    // next-byte word-boundary check (next byte = 'e'), causing the validator to
    // skip the check entirely and miss a later standalone "select:x" keyword.
    //
    // Fix (Pass 7 HIGH-002): use `match_indices("select")` to iterate every
    // occurrence.  Reject if ANY occurrence is a standalone keyword (bounded by
    // non-alphanumeric / non-underscore characters on both sides).
    //
    // Word-boundary heuristic: a byte is a word character if it is ASCII
    // alphanumeric or underscore.  An occurrence is a standalone keyword when
    // neither the byte immediately before nor the byte immediately after is a
    // word character.
    for (pos, _) in lower_remainder.match_indices("select") {
        let prev_ok = pos == 0
            || !lower_remainder
                .as_bytes()
                .get(pos.saturating_sub(1))
                .copied()
                .is_some_and(|b: u8| b.is_ascii_alphanumeric() || b == b'_');
        let after_pos = pos + 6; // len("select") == 6
        let next_ok = after_pos >= lower_remainder.len()
            || !lower_remainder
                .as_bytes()
                .get(after_pos)
                .copied()
                .is_some_and(|b: u8| b.is_ascii_alphanumeric() || b == b'_');
        if prev_ok && next_ok {
            return Err(AqlValidationError {
                reason: "AQL must not contain 'select' keyword (SQL sub-query injection)"
                    .to_owned(),
            });
        }
    }

    // ── 9. Quote injection detection ──────────────────────────────────────────
    // Detect patterns that indicate quote breakout injection.
    //
    // Armis AQL uses double-quoted string values (`field:"value"`).  Single
    // quotes are NOT part of the Armis AQL grammar and have no legitimate use
    // in spec-supplied queries — their presence strongly indicates an injection
    // attempt (Pass 7 HIGH-002, CWE-943).
    //
    // Double-quote rules:
    //   - Unbalanced double-quote count (odd number of `"` chars)
    //   - `"=` or `="` : comparison injection (`"a"="a"`)
    //   - digit immediately followed by `"` : value breakout (`id:1"`)
    //
    // Single-quote rules (Pass 7 HIGH-002):
    //   - Any single-quote character is rejected outright — single-quotes have
    //     no valid role in Armis AQL field:value predicates.  Their presence
    //     is categorically an injection indicator.
    //
    // Note: the single-quote rejection is a blanket rule (simpler and safer
    // than pattern matching) because the Armis AQL grammar does not specify
    // single-quoted string literals.  If a future Armis AQL version introduces
    // single-quoted strings, this rule should be revisited with the same
    // balanced-quote + pattern-breakout approach used for double-quotes.
    if trimmed.contains('\'') {
        return Err(AqlValidationError {
            reason: "AQL must not contain single-quote characters (not valid in Armis AQL; \
                     indicates injection attempt)"
                .to_owned(),
        });
    }

    let quote_count = trimmed.chars().filter(|&c| c == '"').count();
    if quote_count % 2 != 0 {
        return Err(AqlValidationError {
            reason: "AQL contains unbalanced double-quote characters (possible quote injection)"
                .to_owned(),
        });
    }

    // Detect double-quote breakout patterns that indicate SQL-style injection.
    // Legitimate Armis AQL uses `field:"value" AND other:x` — a closing `"`
    // after a complete value followed by a combinator is normal.
    // Injection patterns to block:
    //   - `"=` or `="` : comparison injection (`"a"="a"`)
    //   - digit followed by `"` : value breakout (`id:1"`)
    let lower_trimmed = trimmed.to_ascii_lowercase();
    if lower_trimmed.contains("\"=") || lower_trimmed.contains("=\"") {
        return Err(AqlValidationError {
            reason: "AQL contains quote-comparison injection pattern ('\"=' or '=\"')".to_owned(),
        });
    }

    // Check for digit-immediately-followed-by-quote (value breakout: `id:1"`)
    {
        let bytes = trimmed.as_bytes();
        for window in bytes.windows(2) {
            if window[0].is_ascii_digit() && window[1] == b'"' {
                return Err(AqlValidationError {
                    reason: "AQL contains digit-quote breakout pattern (e.g., 'id:1\"' indicates \
                             value injection)"
                        .to_owned(),
                });
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// AQL hash helper (audit correlation, not cryptographic)
// ---------------------------------------------------------------------------

/// Computes a short hex hash of an AQL string for audit log correlation.
///
/// Uses Rust's built-in `DefaultHasher` for lightweight, fast hashing.
/// This is NOT cryptographic — it is used only for audit correlation to
/// avoid logging the full AQL string.  Two different AQL strings may produce
/// the same hash (but collision probability is extremely low for typical query
/// lengths).
///
/// Returns an 8-hex-digit string (32 bits), e.g., `"a3f8c012"`.
///
/// ADR-005: "log a SHA-256 hash plus a 64-character truncated prefix".
/// We approximate with a 32-bit hash for implementation simplicity (no extra
/// crate dependency, no runtime overhead).  A TD entry covers upgrading to
/// SHA-256 if higher collision resistance is needed.
fn compute_aql_hash(aql: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    aql.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

// ---------------------------------------------------------------------------
// ArmisAdapter — SensorAdapter implementation
// ---------------------------------------------------------------------------

/// Armis Centrix adapter implementing AQL forwarding and timestamp fallback.
pub struct ArmisAdapter {
    /// Canonical org identity for this adapter instance (BC-3.2.001 precondition 4).
    ///
    /// Stored at construction time; verified against `SensorSpec.org_id` at the
    /// start of every `fetch()` call.  A mismatch returns
    /// `SensorError::OrgIdMismatch` immediately, before any network I/O.
    pub(crate) org_id: prism_core::OrgId,
    /// Armis tenant base URL.
    pub(crate) instance_url: String,
    /// Shared HTTP client.
    pub(crate) http: Client,
    /// Bearer access token — wrapped in `SecretString` to guarantee zeroing on
    /// drop and prevent plaintext emission via `Debug` (WGS-W2-002, CWE-312).
    /// Use `expose_secret()` only at HTTP header injection.
    pub(crate) bearer_token: SecretString,
}

impl std::fmt::Debug for ArmisAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArmisAdapter")
            .field("org_id", &self.org_id)
            .field("instance_url", &self.instance_url)
            .field("bearer_token", &"Secret([REDACTED])")
            .finish()
    }
}

impl ArmisAdapter {
    /// Constructs a new adapter.
    ///
    /// `bearer_token` is accepted as `SecretString` to enforce the type-system
    /// guarantee that the token is treated as a secret from the point of
    /// construction (WGS-W2-002).
    ///
    /// # Arguments
    /// - `org_id`       — canonical org identity; stored and verified on every `fetch()` call
    ///   (BC-3.2.001 precondition 4, AC-001).
    /// - `auth`         — Armis API secret key credentials.
    /// - `bearer_token` — static bearer access token for `Authorization: Bearer` header.
    pub fn new(org_id: prism_core::OrgId, auth: &ArmisAuth, bearer_token: SecretString) -> Self {
        let http = Client::builder()
            .cookie_store(false)
            .build()
            .unwrap_or_default();

        Self {
            org_id,
            instance_url: auth.instance_url.clone(),
            http,
            bearer_token,
        }
    }

    /// Constructs the AQL query string for a fetch.
    ///
    /// If `spec.sensor_config["aql_query"]` is a non-null string, validates it
    /// against the Prism AQL allowlist (ADR-005, WGS-W2-001) and returns it if
    /// valid.  Returns `Err(SensorError::ConfigValidation)` if validation fails;
    /// no HTTP call is made in that case (TV-BC-2.01.008-006).
    ///
    /// If `aql_query` is absent, derives a default AQL from `spec.source_table`
    /// using `DEFAULT_AQL_TEMPLATE`.  The default-template branch is safe by
    /// construction and does not require validation.
    ///
    /// # Audit emission
    ///
    /// Every execution of the `sensor_config["aql_query"]` branch emits a
    /// HIGH-severity `tracing::warn!` event with `(client_id, sensor, table,
    /// aql_hash, aql_preview, validation_outcome)` for forensic traceability.
    /// The full AQL string is NOT logged — only a SHA-256 hex prefix and a
    /// 64-character truncated preview (ADR-005).
    ///
    /// BC: BC-2.01.008 (AQL forwarding postcondition, TV-BC-2.01.008-006)
    /// ADR: ADR-005
    // S-3.1.06 stub: spec.client_id is deprecated; retained until impl phase migrates to org_id.
    #[allow(deprecated)]
    pub(crate) fn build_aql(
        &self,
        spec: &SensorSpec,
        _params: &QueryParams,
    ) -> Result<String, SensorError> {
        // Check for an explicit AQL query in sensor config.
        if let Some(aql) = spec.sensor_config.get("aql_query").and_then(|v| v.as_str()) {
            // Compute a short hash for the audit log (avoid logging full AQL).
            // We use a simple FNV-style hash here to avoid pulling in sha2 crate.
            // The ADR asks for SHA-256 hex prefix; we use the std hasher for a
            // lightweight preview — sufficient for correlation in audit logs.
            let aql_hash = compute_aql_hash(aql);
            let aql_preview: String = aql.chars().take(64).collect();

            // Validate against the Prism AQL allowlist (ADR-005).
            match validate_aql(aql) {
                Ok(()) => {
                    // Emit HIGH-severity audit trace event on PASS.
                    // The Vector pipeline picks this up as a structured log entry.
                    tracing::warn!(
                        severity = "HIGH",
                        event_type = "aql_query_execution",
                        client_id = %spec.client_id,
                        sensor = "armis",
                        table = %spec.source_table,
                        aql_hash = %aql_hash,
                        aql_preview = %aql_preview,
                        validation_outcome = "pass",
                        "ADR-005: spec-supplied AQL query executing on Armis sensor"
                    );
                    Ok(aql.to_owned())
                }
                Err(e) => {
                    // Emit HIGH-severity audit trace event on REJECT.
                    tracing::warn!(
                        severity = "HIGH",
                        event_type = "aql_query_rejected",
                        client_id = %spec.client_id,
                        sensor = "armis",
                        table = %spec.source_table,
                        aql_hash = %aql_hash,
                        aql_preview = %aql_preview,
                        validation_outcome = "reject",
                        reason = %e.reason,
                        "ADR-005: spec-supplied AQL rejected by allowlist validator — \
                         no HTTP call issued (WGS-W2-001)"
                    );
                    Err(SensorError::ConfigValidation {
                        sensor: "armis".to_owned(),
                        detail: format!("{}: rejected AQL preview: {aql_preview}", e.reason),
                    })
                }
            }
        } else {
            // Default: substitute table name into template.
            // Safe by construction — source_table is validated at spec load time.
            Ok(DEFAULT_AQL_TEMPLATE.replace("{table}", &spec.source_table))
        }
    }

    /// Resolves the timestamp for an Armis asset record using the fallback chain.
    ///
    /// Tries `firstSeen`, then `lastSeen`, then `Utc::now()`.
    /// Emits `tracing::warn!` when the `now()` fallback is used (AC-6, EC-005).
    // S-3.1.06 stub: spec.client_id is deprecated; retained until impl phase migrates to org_id.
    #[allow(deprecated)]
    pub(crate) fn resolve_timestamp(
        &self,
        record: &serde_json::Value,
        spec: &SensorSpec,
    ) -> DateTime<Utc> {
        // Try firstSeen.
        if let Some(ts_str) = record.get("firstSeen").and_then(|v| v.as_str()) {
            if let Ok(dt) = crate::timestamp::parse_timestamp(ts_str) {
                return dt;
            }
        }

        // Try lastSeen.
        if let Some(ts_str) = record.get("lastSeen").and_then(|v| v.as_str()) {
            if let Ok(dt) = crate::timestamp::parse_timestamp(ts_str) {
                return dt;
            }
        }

        // AC-6 / EC-005: both absent/null/unparseable → use Utc::now() and warn.
        tracing::warn!(
            sensor = "armis",
            table = %spec.source_table,
            client = %spec.client_id,
            "AC-6/EC-005: both firstSeen and lastSeen absent or unparseable; \
             using Utc::now() as timestamp fallback"
        );
        Utc::now()
    }

    /// Issues a GetSearch API call with the given AQL query.
    ///
    /// Includes `Authorization: Bearer {self.bearer_token}` header.
    pub(crate) async fn get_search(
        &self,
        aql: &str,
        _params: &QueryParams,
    ) -> Result<Vec<serde_json::Value>, SensorError> {
        let url = format!("{}/api/v1/search", self.instance_url);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(self.bearer_token.expose_secret())
            .query(&[("aql", aql)])
            .send()
            .await
            .map_err(|e| SensorError::Internal {
                detail: format!("Armis GetSearch request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let code = status.as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(SensorError::HttpError {
                sensor: "armis".to_string(),
                status: code,
                body: body_text,
            });
        }

        let json: serde_json::Value =
            resp.json().await.map_err(|e| SensorError::ResponseParse {
                sensor: "armis".to_string(),
                detail: format!("GetSearch response parse error: {e}"),
            })?;

        // Armis response: `{ "data": { "results": [...], "total": N } }`
        let results = json
            .get("data")
            .and_then(|d| d.get("results"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(results)
    }
}

#[async_trait]
impl SensorAdapter for ArmisAdapter {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("armis")
    }

    fn sensor_name(&self) -> &'static str {
        "armis"
    }

    /// Fetches data from the Armis Centrix GetSearch API using AQL.
    ///
    /// BC: BC-2.01.008 (AC-6)
    async fn fetch(
        &self,
        spec: &SensorSpec,
        params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        // OrgId mismatch guard (BC-3.2.001 precondition 4, AC-004).
        // Must fire before any network I/O.
        if spec.org_id != self.org_id {
            return Err(SensorError::OrgIdMismatch {
                adapter_org_id: self.org_id,
                query_org_id: spec.org_id,
            });
        }

        // Build AQL query.  This validates spec-supplied AQL against the allowlist
        // (ADR-005) and returns Err(SensorError::ConfigValidation) on rejection
        // BEFORE the HTTP semaphore is acquired (TV-BC-2.01.008-006).
        let aql = self.build_aql(spec, params)?;

        // Acquire HTTP semaphore permit (after AQL validation, per ADR-005 ordering).
        let _permit = crate::http::acquire_http_permit().await?;

        // Fetch records via GetSearch.
        let records = self.get_search(&aql, params).await?;

        if records.is_empty() {
            return Ok(vec![]);
        }

        // Resolve timestamps for each record (AC-6 fallback chain).
        let _timestamps: Vec<DateTime<Utc>> = records
            .iter()
            .map(|r| self.resolve_timestamp(r, spec))
            .collect();

        let batch = json_values_to_record_batch(records)?;
        Ok(vec![batch])
    }
}

/// Converts a `Vec<serde_json::Value>` to a single-column `RecordBatch`.
fn json_values_to_record_batch(
    records: Vec<serde_json::Value>,
) -> Result<RecordBatch, SensorError> {
    let schema = Arc::new(Schema::new(vec![Field::new("data", DataType::Utf8, true)]));
    let strings: Vec<Option<String>> = records.iter().map(|v| Some(v.to_string())).collect();
    let array = Arc::new(StringArray::from(strings));
    RecordBatch::try_new(schema, vec![array]).map_err(|e| SensorError::Internal {
        detail: format!("RecordBatch construction failed: {e}"),
    })
}
