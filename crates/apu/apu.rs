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

/// A module for note length counter.
#[derive(Debug)]
struct LengthCounter {
    /// Denotes the note continues for. If it's zero that denotes the note is off.
    count: u8,
}

impl LengthCounter {
    /// Returns initialized length counter object.
    fn init() -> LengthCounter {
        LengthCounter { count: 0 }
    }

    /// Update length counter state.
    fn update(&mut self) {
        if count != 0 {
            count -= 1;
        }
    }

    /// Returns `true` if the note is on.
    fn note_on(&self) -> bool {
        count != 0
    }
}

#[derive(Debug)]
pub struct APU {
    /// Registers to control behavior of APU.
    pub registers: Register,

    /// timer count of frame sequencer in APU sound processing.
    timer_count: u32,
}

impl APU {
    /// Returns initialized Gameboy's APU object.
    pub fn init() -> APU {
        APU {
            registers: Register::init(),
            timer_count: 0,
        }
    }

    /// Return `true` if length counter is triggered. (512Hz / 2 = 256Hz).
    fn length_counter_triggered(&self) -> bool {
        self.timer_count % 2 == 0
    }

    /// Return `true` if volume emvelope is triggererd. (512Hz / 8 = 64Hz).
    fn volume_envelope_triggered(&self) -> bool {
        self.timer_count % 8 == 7
    }

    /// Return `true` if frequency sweep is triggered. (512Hz / 4 = 128Hz).
    fn frequency_sweep_triggered(&self) -> bool {
        self.timer_count % 4 == 3
    }

    /// Update APU internal states.
    /// This function must be called at every 1/512 seconds because of timer event timing.
    pub fn update(&mut self) {
        if self.length_counter_triggered() {}

        if self.volume_envelope_triggered() {}

        if self.frequency_sweep_triggered() {}

        // increment timer count
        if self.timer_count + 1 > 0xFFFF {
            self.timer_count = 0;
        } else {
            self.timer_count += 1;
        }
    }

    /// Generate one signal depends on APU states.
    /// This function may be called at arbitrary time.
    pub fn generate(&self) -> Sample {
        let square1 = Sample::create(0, 0);
        let square2 = Sample::create(0, 0);
        let wave = Sample::create(0, 0);
        let noise = Sample::create(0, 0);

        square1.add(&square2).add(&wave).add(&noise)
    }
}
