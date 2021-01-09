//! Types and traits for GameBoy's APU.

use std::ops::Add;

/// Type of output monoraul sample. It's 4 bit.
#[allow(non_camel_case_types)]
pub struct u4(u8);

impl u4 {
    pub fn new(v: u8) -> u4 {
        u4(v & 0x0F)
    }
}

impl Add for u4 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        u4::new(self.0 + other.0)
    }
}

/// Type of output sample including two channel values. Each channels have 4 bit value.
pub struct Sample(u8);

impl Sample {
    pub fn new(left: u8, right: u8) -> Sample {
        let l = (left << 4) & 0b11110000;
        let r = right & 0b00001111;

        Sample(l | r)
    }

    pub fn left(&self) -> u8 {
        self.0 & 0b11110000
    }

    pub fn right(&self) -> u8 {
        self.0 & 0b00001111
    }

    pub fn add(&self, other: &Self) -> Self {
        let l = self.left() + other.left();
        let r = self.right() + other.right();

        Self::new(l, r)
    }
}

/// Objects that has internal state and updatable.
pub trait Stateful {
    /// Update object's internal states.
    fn update(&mut self);
}

/// Objects that generates signal output.
pub trait Generator {
    /// Generate one signal depends on obejct's internal states.
    fn generate(&self) -> u4;
}

/// Denote errors when setting value to registers.
pub enum RegisterError {
    /// Means that given too large number of u8 bits.
    TooLargeNumberInBits(u32, u8),
}

/// Available duty ratio for square oscillators.
#[derive(Debug)]
pub enum Duty {
    /// Means duty ratio 12.5%.
    Percent12_5,
    /// Means duty ratio 25%.
    Percent25,
    /// Means duty ratio 50%.
    Percent50,
    /// Means duty ratio 75%.
    Percent75,
}
