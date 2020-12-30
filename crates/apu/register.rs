//! Types and functions for GameBoy's APU registers.

/// Sweep-related paramaters for square wave channel.
#[derive(Debug)]
pub struct SquareSweep {
    period: u8,
    nagate: bool,
    shift: u8,
}

/// Envelope generator paramaters for some channels.
#[derive(Debug)]
pub struct Envelope {
    starting_volume: u8,
    envelope_add_mode: bool,
    envelope_period: u8,
}

/// Available values of Wavetable channel volume.
#[derive(Debug)]
pub enum VolumeCode {
    Percent0,
    Percent25,
    Percent50,
    Percent100,
}

/// Frequency in Hz for oscillators pitch.
#[derive(Debug)]
pub struct Frequency {
    frequency: u16,
}

/// Event-related statuses and paramaters.
#[derive(Debug)]
pub struct Event {
    trigger: bool,
    length_enable: bool,
}

/// Parameters for both (square1 and square2) square wave channel.
#[derive(Debug)]
pub struct SquareCommon {
    duty: u8,
    length_load: u8,
    envelope: Envelope,
    frequency: Frequency,
    event: Event,
}

/// Square1 channel. It can sweep frequency.
#[derive(Debug)]
pub struct Square1 {
    sweep: SquareSweep,
    common: SquareCommon,
}

/// Square2 channel. It is without frequency sweeping.
#[derive(Debug)]
pub struct Square2 {
    common: SquareCommon,
}

/// Wavetable channel.
#[derive(Debug)]
pub struct Wave {
    dac_power: bool,
    length_load: u8,
    volume: VolumeCode,
    frequency: Frequency,
    event: Event,
}

/// Noise channel.
#[derive(Debug)]
pub struct Noise {
    length_load: u8,
    envelope: Envelope,
    frequency: Frequency,
    event: Event,
}

/// Statases of all (four) channels.
#[derive(Debug)]
pub struct Status {
    square1: bool,
    square2: bool,
    wave: bool,
    noise: bool,
}

/// Registers for controlling and checking statuses.
#[derive(Debug)]
pub struct Control {
    vin_l_enabled: bool,
    vin_l_volume: u8,
    vin_r_enabled: bool,
    vin_r_volume: u8,

    left_enabled: Status,
    right_enabled: Status,
    power_control: Status,
}

/// Waveform table itself for Wave channel.
#[derive(Debug)]
pub struct WaveTable {
    table: [u8; 32],
}

/// All registers of GameBoy's APU.
#[derive(Debug)]
pub struct Register {
    square1: Square1,
    square2: Square2,
    wave: Wave,
    noise: Noise,
    wavetable: WaveTable,
}
