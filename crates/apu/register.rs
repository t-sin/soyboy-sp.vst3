//! Types and functions for GameBoy's APU registers.

use crate::util::within;

pub enum RegisterError {
    TooLargeNumberInBits(u32, u8),
}

/// Frequency-sweeping-related paramaters for square wave channel.
#[derive(Debug)]
pub struct Sweep {
    /// Sweeping speed. 3 bits.
    pub period: u8,
    /// A modifier for frequency calculation. 1 bits.
    pub negate: bool,
    /// Sweeping intensity. 3 bits.
    pub shift: u8,
}

impl Sweep {
    pub fn init() -> Sweep {
        Sweep {
            period: 0,
            negate: false,
            shift: 0,
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

/// Envelope generator paramaters for some channels.
#[derive(Debug)]
pub struct Envelope {
    /// Volume at start time. 4 bits.
    pub starting_volume: u8,
    /// Flag to switch envelope add mode. Use adding if it's true, otherwise subtracting. 1 bits.
    pub add_mode: bool,
    /// Envelope speed. 3 bits.
    pub period: u8,
}

impl Envelope {
    pub fn init() -> Envelope {
        Envelope {
            starting_volume: 0,
            add_mode: false,
            period: 0,
        }
    }

    pub fn set_starting_volume(&mut self, starting_volume: u32) -> Result<(), RegisterError> {
        if within(starting_volume.into(), 4) {
            self.starting_volume = starting_volume as u8;
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

/// Available values of Wavetable channel volume.
#[derive(Debug)]
pub enum Volume {
    Percent0,
    Percent25,
    Percent50,
    Percent100,
}

/// Frequency in Hz for oscillators pitch.
#[derive(Debug)]
pub struct Frequency {
    /// Ocsllator frequency in Hz. 11 bits.
    pub frequency: u16,
}

impl Frequency {
    pub fn init() -> Frequency {
        Frequency { frequency: 0 }
    }

    pub fn set_frequency(&mut self, f: u32) -> Result<(), RegisterError> {
        if within(f, 11) {
            self.frequency = f as u16;
            Ok(())
        } else {
            Err(RegisterError::TooLargeNumberInBits(f.into(), 11))
        }
    }
}

/// Event-related statuses and paramaters.
#[derive(Debug)]
pub struct Event {
    pub trigger: bool,
    pub length_enable: bool,
}

impl Event {
    pub fn init() -> Event {
        Event {
            trigger: false,
            length_enable: false,
        }
    }
}

/// Available duty ratio for square oscillators.
#[derive(Debug)]
pub enum Duty {
    Percent12_5,
    Percent25,
    Percent50,
    Percent75,
}

/// Parameters for both (square1 and square2) square wave channel.
#[derive(Debug)]
pub struct SquareCommon {
    /// Duty ratio of square wave. 2 bit.
    pub duty: Duty,
    /// Sound duration count. 6 bits.
    pub length_load: u8,
    pub envelope: Envelope,
    pub frequency: Frequency,
    pub event: Event,
}

impl SquareCommon {
    pub fn init() -> SquareCommon {
        SquareCommon {
            duty: Duty::Percent50,
            length_load: 0,
            envelope: Envelope::init(),
            frequency: Frequency::init(),
            event: Event::init(),
        }
    }

    pub fn set_length_load(&mut self, length_load: u32) -> Result<(), RegisterError> {
        if within(length_load, 6) {
            self.length_load = length_load as u8;
            Ok(())
        } else {
            Err(RegisterError::TooLargeNumberInBits(length_load.into(), 6))
        }
    }
}

/// Square1 channel. It can sweep frequency.
#[derive(Debug)]
pub struct Square1 {
    pub sweep: Sweep,
    pub common: SquareCommon,
}

/// Square2 channel. It is without frequency sweeping.
#[derive(Debug)]
pub struct Square2 {
    pub common: SquareCommon,
}

/// Wavetable channel.
#[derive(Debug)]
pub struct Wave {
    dac_power: bool,
    length_load: u8,
    volume: Volume,
    frequency: Frequency,
    event: Event,
}

impl Wave {
    pub fn init() -> Wave {
        Wave {
            dac_power: false,
            length_load: 0,
            volume: Volume::Percent50,
            frequency: Frequency::init(),
            event: Event::init(),
        }
    }

    pub fn set_length_load(&mut self, length_load: u32) -> Result<(), RegisterError> {
        if within(length_load, 8) {
            self.length_load = length_load as u8;
            Ok(())
        } else {
            Err(RegisterError::TooLargeNumberInBits(length_load.into(), 8))
        }
    }
}

/// Noise channel.
#[derive(Debug)]
pub struct Noise {
    pub length_load: u8,
    pub envelope: Envelope,
    pub frequency: Frequency,
    pub event: Event,
}

impl Noise {
    pub fn init() -> Noise {
        Noise {
            length_load: 0,
            envelope: Envelope::init(),
            frequency: Frequency::init(),
            event: Event::init(),
        }
    }

    pub fn set_length_load(&mut self, length_load: u32) -> Result<(), RegisterError> {
        if within(length_load, 8) {
            self.length_load = length_load as u8;
            Ok(())
        } else {
            Err(RegisterError::TooLargeNumberInBits(length_load.into(), 8))
        }
    }
}

/// Statases of all (four) channels.
#[derive(Debug)]
pub struct Status {
    pub square1: bool,
    pub square2: bool,
    pub wave: bool,
    pub noise: bool,
}

impl Status {
    pub fn init() -> Status {
        Status {
            square1: false,
            square2: false,
            wave: false,
            noise: false,
        }
    }
}

/// Registers for controlling and checking statuses.
#[derive(Debug)]
pub struct Control {
    pub vin_l_enabled: bool,
    pub vin_l_volume: u8,
    pub vin_r_enabled: bool,
    pub vin_r_volume: u8,
    pub left_enabled: Status,
    pub right_enabled: Status,
    pub power_control: Status,
}

impl Control {
    pub fn init() -> Control {
        Control {
            vin_l_enabled: false,
            vin_l_volume: 0,
            vin_r_enabled: false,
            vin_r_volume: 0,
            left_enabled: Status::init(),
            right_enabled: Status::init(),
            power_control: Status::init(),
        }
    }

    pub fn set_vin_l_volume(&mut self, vin_l_volume: u32) -> Result<(), RegisterError> {
        if within(vin_l_volume, 3) {
            self.vin_l_volume = vin_l_volume as u8;
            Ok(())
        } else {
            Err(RegisterError::TooLargeNumberInBits(vin_l_volume.into(), 3))
        }
    }

    pub fn set_vin_r_volume(&mut self, vin_r_volume: u32) -> Result<(), RegisterError> {
        if within(vin_r_volume, 3) {
            self.vin_r_volume = vin_r_volume as u8;
            Ok(())
        } else {
            Err(RegisterError::TooLargeNumberInBits(vin_r_volume.into(), 3))
        }
    }
}

/// Waveform table itself for Wave channel.
#[derive(Debug)]
pub struct WaveTable {
    pub table: [u8; 16],
}

impl WaveTable {
    pub fn init() -> WaveTable {
        WaveTable {
            table: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}

/// All registers of GameBoy's APU.
#[derive(Debug)]
pub struct Register {
    pub square1: Square1,
    pub square2: Square2,
    pub wave: Wave,
    pub noise: Noise,
    pub control: Control,
    pub wavetable: WaveTable,
}

impl Register {
    pub fn init() -> Register {
        Register {
            square1: Square1 {
                sweep: Sweep::init(),
                common: SquareCommon::init(),
            },
            square2: Square2 {
                common: SquareCommon::init(),
            },
            wave: Wave::init(),
            noise: Noise::init(),
            control: Control::init(),
            wavetable: WaveTable::init(),
        }
    }
}
