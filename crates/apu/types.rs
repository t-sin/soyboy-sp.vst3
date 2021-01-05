//! Types and traits for GameBoy's APU.

/// Type of output sample including two channel values. Each channels have 4 bit value.
pub struct Sample(u8);

impl Sample {
    pub fn create(left: u8, right: u8) -> Sample {
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

        Self::create(l, r)
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
    fn generate(&self) -> Sample;
}

/// Denote errors when setting value to registers.
pub enum RegisterError {
    TooLargeNumberInBits(u32, u8),
}
