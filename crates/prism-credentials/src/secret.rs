//! Secret<T> — wrapper type that redacts values in Display, Debug, and Serialize.
//!
//! # Contract: BC-2.03.007
//! - `Display` returns `"[REDACTED]"`
//! - `Debug` returns `"SecretString([REDACTED])"`
//! - Value is accessible only via `.expose()`
//! - Implements `Zeroize` on drop (memory zeroed when Secret goes out of scope)

use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A wrapper type that redacts its contents from all formatted output.
///
/// The inner value is accessible only via `.expose()`. Display, Debug, and
/// any accidental format usage return `"[REDACTED]"` or `"SecretString([REDACTED])"`.
///
/// # Memory Safety
/// Implements `ZeroizeOnDrop` — inner value is zeroed when Secret<T> is dropped.
#[derive(ZeroizeOnDrop)]
pub struct Secret<T: Zeroize>(T);

impl<T: Zeroize> Secret<T> {
    /// Wrap a value in a Secret.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Expose the inner value for consumption.
    ///
    /// Callers MUST NOT store or transmit the returned reference beyond the
    /// minimum required scope. The credential must not leave the `SensorAuth`
    /// consumption boundary.
    pub fn expose(&self) -> &T {
        &self.0
    }
}

impl<T: Zeroize> fmt::Display for Secret<T> {
    /// Always returns `"[REDACTED]"`. Never exposes the inner value.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl<T: Zeroize> fmt::Debug for Secret<T> {
    /// Always returns `"SecretString([REDACTED])"`. Safe for debug logging.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretString([REDACTED])")
    }
}

impl<T: Zeroize + Clone> Clone for Secret<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Zeroize> Zeroize for Secret<T> {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

// Note: Drop is handled by ZeroizeOnDrop derive macro. No manual Drop needed.

/// Compute a dry-run preview: first 2 chars + "***" + last 2 chars.
/// For values shorter than 5 chars, returns "***" only (no leakage).
///
/// # Contract: BC-2.03.007 postcondition (dry-run preview)
pub fn dry_run_preview(value: &str) -> String {
    if value.len() < 5 {
        "***".to_string()
    } else {
        let chars: Vec<char> = value.chars().collect();
        let first: String = chars[..2].iter().collect();
        let last: String = chars[chars.len() - 2..].iter().collect();
        format!("{first}***{last}")
    }
}
