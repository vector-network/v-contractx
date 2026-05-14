//! Shared helpers used by the runtime and contract layers.

use crate::errors::KernelError;

pub const AUTH_RATIO_SCALE: u64 = 1_000_000;

pub fn clamp_u128(value: u128, min: u128, max: u128) -> u128 {
    value.min(max).max(min)
}

pub fn ratio_to_micros(numerator: u128, denominator: u128) -> Result<u32, KernelError> {
    if denominator == 0 {
        return Err(KernelError::InvalidParameter(
            "ratio denominator must be non-zero".to_string(),
        ));
    }
    let scaled = numerator
        .saturating_mul(AUTH_RATIO_SCALE as u128)
        .checked_div(denominator)
        .ok_or_else(|| KernelError::InvalidParameter("ratio division failed".to_string()))?;
    Ok(scaled.min(AUTH_RATIO_SCALE as u128) as u32)
}

pub fn canonical_bool(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}
