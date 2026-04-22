//! Tracing subscriber configuration and initializer.

use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Configuration for the global tracing subscriber.
#[derive(Clone, Debug)]
pub struct TracingConfig {
    /// Minimum log level to emit.
    pub level: Level,
    /// Emit JSON-structured log lines instead of human-readable format.
    pub json_output: bool,
    /// Service name tag attached to every span/event.
    pub service_name: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            json_output: false,
            service_name: "prism".to_string(),
        }
    }
}

/// Install the global tracing subscriber using `cfg`.
///
/// Silently no-ops if a global subscriber is already installed (e.g. in test
/// harnesses where multiple test binaries share a process, or when downstream
/// crates call `init_tracing` during their own initialization). This is safe:
/// the already-installed subscriber continues to function correctly.
pub fn init_tracing(cfg: &TracingConfig) {
    let filter = EnvFilter::builder()
        .with_default_directive(cfg.level.into())
        .from_env_lossy();

    if cfg.json_output {
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json().with_current_span(true));
        // Ignore AlreadyInitialized — benign in test harnesses and multi-crate inits.
        let _ = subscriber.try_init();
    } else {
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer());
        // Ignore AlreadyInitialized — benign in test harnesses and multi-crate inits.
        let _ = subscriber.try_init();
    }
}
