use crate::register::Register;

/// Type of output sample including two channel values.
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

#[derive(Debug)]
pub struct APU {
    pub registers: Register,
    // some internal states
}

impl APU {
    pub fn init() -> APU {
        APU {
            registers: Register::init(),
        }
    }
}
