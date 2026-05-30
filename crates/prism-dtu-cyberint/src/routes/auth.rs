//! Auth route handlers for the Cyberint DTU clone.
//!
//! As of S-DTU-CYBERINT-AUTH-FIDELITY-001, the `POST /login` route has been removed
//! per ADR-031 §D3-a rule 1: the real Cyberint API requires no login step.
//!
//! Auth is validated via the `access_token` cookie on every data request (see
//! `routes/alerts.rs::check_auth` and `routes/alerts.rs::extract_access_token`).
//!
//! This module is retained as an empty placeholder. Future auth-related DTU handlers
//! (if any) should be added here. The `pub mod auth;` declaration in `routes/mod.rs`
//! is kept to avoid churn if auth helpers are added later.
//!
//! AC-001 (S-DTU-CYBERINT-AUTH-FIDELITY-001): `build_router` does NOT register POST /login.
