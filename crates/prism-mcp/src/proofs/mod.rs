//! Verification property proofs for prism-mcp (Phase 6 formal hardening targets).
//!
//! | VP ID | Method | Property |
//! |-------|--------|----------|
//! | VP-050 | proptest | Sensor resource response redacts credentials and full API URLs |

pub mod sensor_resource_redaction;
