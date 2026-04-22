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
/// Panics if called more than once (tracing-subscriber enforces single init).
pub fn init_tracing(cfg: &TracingConfig) {
    let filter = EnvFilter::builder()
        .with_default_directive(cfg.level.into())
        .from_env_lossy();

    if cfg.json_output {
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json().with_current_span(true));
        subscriber
            .try_init()
            .expect("failed to install JSON tracing subscriber");
    } else {
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer());
        subscriber
            .try_init()
            .expect("failed to install tracing subscriber");
    }
}
