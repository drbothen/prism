//! Tracing subscriber configuration and initializer.

use tracing::Level;

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
/// Stub body — S-1.01 implementation will wire up tracing-subscriber layers.
pub fn init_tracing(_cfg: &TracingConfig) {
    todo!("S-1.01: implement init_tracing")
}
