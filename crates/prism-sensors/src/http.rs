//! Global HTTP connection semaphore.
//!
//! All sensor HTTP clients MUST acquire a permit from `HTTP_SEMAPHORE` before
//! sending a request and release it on response or error. This caps total
//! outbound connections at `HTTP_SEMAPHORE_PERMITS` process-wide (S-2.06 §Task 7).
//!
//! # Timeout
//! Permit acquisition is timeout-bounded to 30 seconds. If the timeout is
//! exceeded, the task returns `SensorError::ConnectionPoolExhausted` with a
//! structured tracing log recording the current permit count (EC-004).
//!
//! # Thread Safety
//! `HTTP_SEMAPHORE` is a `OnceLock<Semaphore>` initialized by `init_http_semaphore()`.
//! Call `init_http_semaphore()` once at process startup (before any sensor fetches).
//!
//! Story: S-2.06 | BC: AC-5, EC-003, EC-004

use std::{sync::OnceLock, time::Duration};

use tokio::sync::{Semaphore, SemaphorePermit};

use crate::adapter::SensorError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of concurrent outbound HTTP connections across all sensors.
pub const HTTP_SEMAPHORE_PERMITS: usize = 200;

/// Timeout for semaphore permit acquisition.
///
/// If a task cannot acquire a permit within 30 seconds, it returns
/// `SensorError::ConnectionPoolExhausted` (EC-004).
pub const HTTP_SEMAPHORE_TIMEOUT: Duration = Duration::from_secs(30);

// ---------------------------------------------------------------------------
// Global semaphore
// ---------------------------------------------------------------------------

/// Process-wide HTTP connection semaphore.
///
/// Initialized with `HTTP_SEMAPHORE_PERMITS` permits. Acquired via
/// `acquire_http_permit()`. Never dropped — the `OnceLock` holds the value
/// for the lifetime of the process.
static HTTP_SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();

// ---------------------------------------------------------------------------
// Initializer
// ---------------------------------------------------------------------------

/// Initializes the global HTTP semaphore with `HTTP_SEMAPHORE_PERMITS` permits.
///
/// MUST be called once at process startup, before any `acquire_http_permit()`
/// call. Subsequent calls are no-ops (idempotent — `OnceLock` semantics).
pub fn init_http_semaphore() {
    HTTP_SEMAPHORE.get_or_init(|| Semaphore::new(HTTP_SEMAPHORE_PERMITS));
}

// ---------------------------------------------------------------------------
// Permit acquisition
// ---------------------------------------------------------------------------

/// Acquires a single permit from the global HTTP semaphore.
///
/// Blocks the calling task until a permit is available or the 30-second
/// timeout expires (AC-5, EC-003).
///
/// Calls `init_http_semaphore()` internally if the semaphore has not yet been
/// initialized, so callers do not need to guarantee initialization order.
///
/// # Errors
/// Returns `SensorError::ConnectionPoolExhausted` if the permit cannot be
/// obtained within `HTTP_SEMAPHORE_TIMEOUT`. A `tracing::error!` event is
/// emitted with the current available_permits count (EC-004).
pub async fn acquire_http_permit() -> Result<SemaphorePermit<'static>, SensorError> {
    // Idempotent: initializes if not yet done; no-op if already initialized.
    init_http_semaphore();
    // SAFETY: we just initialized it above, so `get()` is guaranteed `Some`.
    let semaphore = HTTP_SEMAPHORE
        .get()
        .unwrap_or_else(|| unreachable!("init_http_semaphore() guarantees Some"));

    match tokio::time::timeout(HTTP_SEMAPHORE_TIMEOUT, semaphore.acquire()).await {
        Ok(Ok(permit)) => Ok(permit),
        Ok(Err(_closed)) => {
            // Semaphore was closed — treat as pool exhaustion
            tracing::error!(
                available_permits = semaphore.available_permits(),
                "HTTP semaphore closed during permit acquisition"
            );
            Err(SensorError::ConnectionPoolExhausted)
        }
        Err(_timeout) => {
            tracing::error!(
                available_permits = semaphore.available_permits(),
                "HTTP semaphore acquisition timed out after {}s",
                HTTP_SEMAPHORE_TIMEOUT.as_secs()
            );
            Err(SensorError::ConnectionPoolExhausted)
        }
    }
}

/// Returns the number of permits currently available in the global HTTP
/// semaphore, or `None` if the semaphore has not been initialized.
///
/// Used for structured error logging in the timeout path (EC-004).
pub fn available_http_permits() -> Option<usize> {
    HTTP_SEMAPHORE.get().map(|s| s.available_permits())
}
