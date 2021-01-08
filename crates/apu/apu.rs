use crate::types::{u4, Duty, Generator, RegisterError, Sample, Stateful};
use crate::util::within;

/// A module for note length counter.
#[derive(Debug)]
pub struct LengthCounter {
    /// Wheather LengthCounter is enabled.
    pub enable: bool,
    /// Sound duration count. 6 bits.
    pub length: u8,

    /// Denotes the note continues for. If it's zero that denotes the note is off.
    count: u8,
}

impl LengthCounter {
    /// Returns initialized length counter object.
    fn init() -> LengthCounter {
        LengthCounter {
            enable: false,
            length: 0,
            count: 0,
        }
    }

    pub fn set_length_load(&mut self, length: u32) -> Result<(), RegisterError> {
        if within(length, 6) {
            self.length = length as u8;
            self.count = self.length;
            Ok(())
        } else {
            Err(RegisterError::TooLargeNumberInBits(length.into(), 6))
        }
    }

    /// Returns `true` if the note is on.
    fn note_on(&self) -> bool {
        self.count != 0
    }
}

impl Stateful for LengthCounter {
    /// Update length counter state. This must be called at 256Hz frequency.
    fn update(&mut self) {
        if self.count != 0 {
            self.count -= 1;
        }
    }
}

impl Generator for LengthCounter {
    fn generate(&self) -> u4 {
        if self.note_on() {
            u4::new(1)
        } else {
            u4::new(0)
        }
    }
}

/// A module for controlling channel volume.
#[derive(Debug)]
pub struct VolumeEnvelope {
    /// Volume at start time. 4 bits.
    pub starting_volume: u8,
    /// Flag to switch envelope add mode. Use adding if it's true, otherwise subtracting. 1 bits.
    pub add_mode: bool,
    /// Envelope speed. 3 bits.
    pub period: u8,

    /// Denotes current volume within 0 to 15 range.
    volume: u8,
}

impl VolumeEnvelope {
    /// Returns initialized volume envelope object.
    fn init() -> VolumeEnvelope {
        VolumeEnvelope {
            starting_volume: 0,
            add_mode: true,
            period: 0,
            volume: 0,
        }
    }

    pub fn set_starting_volume(&mut self, starting_volume: u32) -> Result<(), RegisterError> {
        if within(starting_volume.into(), 4) {
            self.starting_volume = starting_volume as u8;
            self.volume = self.starting_volume;
            Ok(())
        } else {
            Err(RegisterError::TooLargeNumberInBits(
                starting_volume.into(),
                3,
            ))
        }
    }

    pub fn set_period(&mut self, period: u32) -> Result<(), RegisterError> {
        if within(period.into(), 3) {
            self.period = period as u8;
            Ok(())
        } else {
            Err(RegisterError::TooLargeNumberInBits(period.into(), 3))
        }
    }
}

impl Stateful for VolumeEnvelope {
    /// Update volume envelope state. This must be called at 64Hz frequency.
    fn update(&mut self) {
        if self.add_mode {
            if self.volume < 0xf {
                self.volume += 1;
            }
        } else {
            if self.volume > 0 {
                self.volume -= 1;
            }
        }
    }
}

impl Generator for VolumeEnvelope {
    fn generate(&self) -> u4 {
        u4::new(self.volume)
    }
}

/// Frequency-sweeping-related paramaters for square wave channel.
#[derive(Debug)]
pub struct FrequencySweep {
    /// Sweeping speed. 3 bits.
    pub period: u8,
    /// A modifier for frequency calculation. 1 bits.
    pub negate: bool,
    /// Sweeping intensity. 3 bits.
    pub shift: u8,

    count: u8,
}

impl FrequencySweep {
    pub fn init() -> FrequencySweep {
        FrequencySweep {
            period: 0,
            negate: false,
            shift: 0,
            count: 0,
        }
    }

    pub fn set_period(&mut self, period: u32) -> Result<(), RegisterError> {
        if within(period.into(), 3) {
            self.period = period as u8;
            Ok(())
        } else {
            Err(RegisterError::TooLargeNumberInBits(period.into(), 3))
        }
    }

    pub fn set_shift(&mut self, shift: u32) -> Result<(), RegisterError> {
        if within(shift.into(), 3) {
            self.shift = shift as u8;
            Ok(())
        } else {
            Err(RegisterError::TooLargeNumberInBits(shift.into(), 3))
        }
    }
}

impl Stateful for FrequencySweep {
    /// Updates internal states. This function must be called at 128Hz frequency.
    fn update(&mut self) {}
}

pub struct DutyCycler {
    /// Duty ratio selector.
    pub duty: Duty,

    // internal regsiters
    reg12_5: u8,
    reg25: u8,
    reg50: u8,
    reg75: u8,
}

impl DutyCycler {
    /// Returns initialized duty cycler object.
    pub fn init() -> DutyCycler {
        DutyCycler {
            duty: Duty::Percent50,
            reg12_5: 0b00000001,
            reg25: 0b10000001,
            reg50: 0b10000111,
            reg75: 0b01111110,
        }
    }
}

impl Stateful for DutyCycler {
    fn update(&mut self) {
        self.reg12_5.rotate_left(1);
        self.reg25.rotate_left(1);
        self.reg50.rotate_left(1);
        self.reg75.rotate_left(1);
    }
}

impl Generator for DutyCycler {
    fn generate(&self) -> u4 {
        let v = 0b00000001
            & match self.duty {
                Duty::Percent12_5 => self.reg12_5,
                Duty::Percent25 => self.reg25,
                Duty::Percent50 => self.reg50,
                Duty::Percent75 => self.reg75,
            };
        let v = v << 4;
        u4::new(v)
    }
}

#[derive(Debug)]
pub struct APU {
    /// timer count of frame sequencer in APU sound processing.
    timer_count: u32,
}

impl APU {
    /// Returns initialized Gameboy's APU object.
    pub fn init() -> APU {
        APU { timer_count: 0 }
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
}

impl Generator for APU {
    /// Generate one signal depends on APU states.
    /// This function may be called at arbitrary time.
    fn generate(&self) -> u4 {
        let square1 = u4::new(0);
        let square2 = u4::new(0);
        let wave = u4::new(0);
        let noise = u4::new(0);

        // square1.add(&square2).add(&wave).add(&noise)
        square1
    }
}

impl Stateful for APU {
    /// Update APU internal states.
    /// This function must be called at every 1/512 seconds because of timer event timing.
    fn update(&mut self) {
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
}
