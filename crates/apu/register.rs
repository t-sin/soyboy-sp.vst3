//! Types and functions for GameBoy's APU registers.

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
