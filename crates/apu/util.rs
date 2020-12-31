//! Some utilities for APU.

/// Check if `n` can be represent within a number of `bits` bits.
pub fn within(n: u32, bits: u8) -> bool {
    if n < 2u32.pow(bits.into()) {
        true
    } else {
        false
    }
}
