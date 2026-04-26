//! `SlackState` — in-memory state for the Slack Incoming Webhook DTU behavioral clone.
//!
//! Maintains:
//! - Received payload capture store: ordered list of all Block Kit payloads POSTed since last reset
//! - Request counter: `AtomicU32` for rate-limit threshold tracking
//! - Failure mode: configurable via `POST /dtu/configure` for error injection
//!
//! No HTTP-layer types (`axum::Json`, `axum::extract::*`) appear here.
//! `SlackState` is pure Rust — no Axum dependency for its public methods.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use prism_dtu_common::FailureMode;
use serde_json::Value;

use crate::types::SlackConfigPayload;

/// Shared mutable state for the Slack Incoming Webhook DTU clone.
///
/// `Arc<SlackState>` is passed to every axum route handler via `axum::extract::State`.
pub struct SlackState {
    // --- Mutable state (reset by `reset()`) ---
    /// Ordered list of all Block Kit payloads received since last reset.
    ///
    /// Captured on each valid `POST /services/{token}` (after validation).
    /// Returned verbatim by `GET /dtu/received-payloads`.
    pub received_payloads: Mutex<Vec<Value>>,

    /// Total request count since last reset — used for rate-limit threshold tracking.
    ///
    /// Incremented on every `POST /services/{token}` (including rejected ones, before
    /// validation, so rate-limit fires at the right count regardless of payload shape).
    pub request_count: AtomicU32,

    /// Shared failure mode, read on every incoming request.
    ///
    /// Wrapped in `Arc` so `build_router()` can clone it into the tower layer
    /// while `apply_config()` can mutate it after the server starts.
    pub failure_mode: Arc<Mutex<FailureMode>>,

    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    pub admin_token: String,
}

impl Default for SlackState {
    fn default() -> Self {
        Self::new()
    }
}

impl SlackState {
    /// Construct a fresh `SlackState` with a random admin token.
    pub fn new() -> Self {
        Self::with_admin_token(uuid::Uuid::new_v4().to_string())
    }

    /// Construct with a specific admin token (used by clone to share between
    /// the route handler and `BehavioralClone::admin_token()`).
    pub fn with_admin_token(admin_token: String) -> Self {
        Self {
            received_payloads: Mutex::new(Vec::new()),
            request_count: AtomicU32::new(0),
            failure_mode: Arc::new(Mutex::new(FailureMode::None)),
            admin_token,
        }
    }

    /// Reset all mutable state to initial values (called by `BehavioralClone::reset`).
    ///
    /// - Clears the payload capture store.
    /// - Resets the request counter to 0.
    /// - Resets the failure mode to `FailureMode::None`.
    ///
    /// Per AC-6 and EC-005: reset is atomic with respect to in-flight requests
    /// (each field locked/stored separately — in-flight requests complete normally).
    pub fn reset(&self) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut payloads = self
            .received_payloads
            .lock()
            .expect("received_payloads poisoned");
        payloads.clear();

        self.request_count.store(0, Ordering::SeqCst);

        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        let mut mode = self.failure_mode.lock().expect("failure_mode poisoned");
        *mode = FailureMode::None;
    }

    /// Apply a JSON configuration patch (from `POST /dtu/configure`).
    ///
    /// Recognised keys per story Task 6:
    /// - `"failure_mode"` — `"none"`, `"rate_limit"`, `"internal_error"`.
    ///   Companions: `"after_n_requests"`, `"retry_after_secs"`, `"at_request_n"`.
    /// - Shorthand: `"rate_limit_after"` (sets rate_limit mode with given threshold).
    /// - Shorthand: `"fail_with": 500` (sets internal_error mode).
    ///
    /// Unknown fields are rejected (`deny_unknown_fields` on `SlackConfigPayload`).
    pub fn apply_config(&self, config: &Value) -> anyhow::Result<()> {
        let payload: SlackConfigPayload = serde_json::from_value(config.clone())
            .map_err(|e| anyhow::anyhow!("invalid /dtu/configure payload: {e}"))?;

        // Resolve shorthand forms first.
        let effective_mode: Option<String> = if payload.rate_limit_after.is_some() {
            Some("rate_limit".to_string())
        } else if payload.fail_with == Some(500) {
            Some("internal_error".to_string())
        } else {
            payload.failure_mode.clone()
        };

        if let Some(mode_str) = effective_mode.as_deref() {
            let new_mode = match mode_str {
                "none" => FailureMode::None,
                "rate_limit" => {
                    let after_n = payload
                        .rate_limit_after
                        .or(payload.after_n_requests)
                        .unwrap_or(0);
                    let retry_after = payload.retry_after_secs.unwrap_or(30);
                    FailureMode::RateLimit {
                        after_n_requests: after_n,
                        retry_after_secs: retry_after,
                    }
                }
                "internal_error" => {
                    let at_n = payload.at_request_n.unwrap_or(1);
                    FailureMode::InternalError { at_request_n: at_n }
                }
                other => {
                    anyhow::bail!("unknown failure_mode: {other}");
                }
            };
            // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
            #[allow(clippy::expect_used)]
            let mut guard = self
                .failure_mode
                .lock()
                .expect("SlackState: failure_mode lock poisoned");
            *guard = new_mode;
        }
        Ok(())
    }

    /// Capture a validated Block Kit payload (called by the webhook route handler).
    ///
    /// Appends the payload to the ordered capture store. Called only on valid payloads
    /// (after 400 validation is passed, before 200 response is returned).
    pub fn capture_payload(&self, payload: Value) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut store = self
            .received_payloads
            .lock()
            .expect("received_payloads poisoned");
        store.push(payload);
    }

    /// Return all captured Block Kit payloads since the last reset (for `GET /dtu/received-payloads`).
    ///
    /// Per AC-5: returns payloads in insertion order.
    pub fn all_payloads(&self) -> Vec<Value> {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let store = self
            .received_payloads
            .lock()
            .expect("received_payloads poisoned");
        store.clone()
    }

    /// Increment and return the new request count.
    ///
    /// Called on every incoming POST to `/services/{token}` before any other processing.
    /// Used to implement rate-limit threshold (AC-4).
    pub fn increment_request_count(&self) -> u32 {
        self.request_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Read the current failure mode (without holding the lock after return).
    pub fn current_failure_mode(&self) -> FailureMode {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mode = self.failure_mode.lock().expect("failure_mode poisoned");
        mode.clone()
    }
}
