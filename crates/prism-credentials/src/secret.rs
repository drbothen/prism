//! Secret<T> — wrapper type that redacts values in Display, Debug, and Serialize.
//!
//! # Contract: BC-2.03.007
//! - `Display` returns `"[REDACTED]"`
//! - `Debug` returns `"SecretString([REDACTED])"`
//! - `Serialize` is intentionally unimplemented (compile error at use site)
//! - Value is accessible only via `.expose()`
//! - Implements `Zeroize` on drop (memory zeroed when Secret goes out of scope)
//!
//! # Dev Note (S-1.07)
//! The story references `secrecy::SecretString` (S-1.06 pattern) alongside a
//! custom `Secret<T>` wrapper. This module provides the custom `Secret<T>` type
//! that wraps arbitrary `Zeroize` types while enforcing redaction at all output
//! surfaces. `secrecy::SecretString` is re-exported for backwards compat with S-1.06.

use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A wrapper type that redacts its contents from all formatted output.
///
/// The inner value is accessible only via `.expose()`. Display, Debug, and
/// any accidental format usage return `"[REDACTED]"` or `"SecretString([REDACTED])"`.
///
/// # Memory Safety
/// Implements `ZeroizeOnDrop` — inner value is zeroed when Secret<T> is dropped.
pub struct Secret<T: Zeroize>(T);

impl<T: Zeroize> Secret<T> {
    /// Wrap a value in a Secret.
    pub fn new(value: T) -> Self {
        todo!("S-1.07: implement Secret::new")
    }

    /// Expose the inner value for consumption.
    ///
    /// Callers MUST NOT store or transmit the returned reference beyond the
    /// minimum required scope. The credential must not leave the `SensorAuth`
    /// consumption boundary.
    pub fn expose(&self) -> &T {
        todo!("S-1.07: implement Secret::expose")
    }
}

impl<T: Zeroize> fmt::Display for Secret<T> {
    /// Always returns `"[REDACTED]"`. Never exposes the inner value.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("S-1.07: implement Secret Display — must return [REDACTED]")
    }
}

impl<T: Zeroize> fmt::Debug for Secret<T> {
    /// Always returns `"SecretString([REDACTED])"`. Safe for debug logging.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("S-1.07: implement Secret Debug — must return SecretString([REDACTED])")
    }
}

impl<T: Zeroize + Clone> Clone for Secret<T> {
    fn clone(&self) -> Self {
        todo!("S-1.07: implement Secret Clone")
    }
}

impl<T: Zeroize> Zeroize for Secret<T> {
    fn zeroize(&mut self) {
        todo!("S-1.07: implement Secret Zeroize")
    }
}

impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        todo!("S-1.07: implement Secret Drop with zeroize")
    }
}

/// Compute a dry-run preview: first 2 chars + "***" + last 2 chars.
/// For values shorter than 5 chars, returns "***" only (no leakage).
///
/// # Contract: BC-2.03.007 postcondition (dry-run preview)
pub fn dry_run_preview(value: &str) -> String {
    todo!("S-1.07: implement dry_run_preview")
}
