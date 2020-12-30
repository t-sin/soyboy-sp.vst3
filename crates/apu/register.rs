#[derive(Debug)]
pub struct SquareSweep {
    period: u8,
    nagete: bool,
    shift: u8,
}

#[derive(Debug)]
pub struct Envelope {
    starting_volume: u8,
    envelope_add_mode: bool,
    envelope_period: u8,
}

#[derive(Debug)]
pub enum VolumeCode {
    Percent0,
    Percent25,
    Percent50,
    Percent100,
}

#[derive(Debug)]
pub struct Pitch {
    frequency: u16,
}

#[derive(Debug)]
pub struct Event {
    trigger: bool,
    length_enable: bool,
}

#[derive(Debug)]
pub struct SquareCommon {
    duty: u8,
    length_load: u8,
    envelope: Envelope,
    pitch: Pitch,
    event: Event,
}

#[derive(Debug)]
pub struct Square1 {
    sweep: SquareSweep,
    common: SquareCommon,
}

#[derive(Debug)]
pub struct Square2 {
    common: SquareCommon,
}

#[derive(Debug)]
pub struct Wave {
    dac_power: bool,
    length_load: u8,
    volume: VolumeCode,
    pitch: Pitch,
    event: Event,
}

#[derive(Debug)]
pub struct Noise {
    length_load: u8,
    envelope: Envelope,
    pitch: Pitch,
    event: Event,
}

#[derive(Debug)]
pub struct Status {
    square1: bool,
    square2: bool,
    wave: bool,
    noise: bool,
}

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

#[derive(Debug)]
pub struct WaveTable {
    table: [u8; 32],
}

#[derive(Debug)]
pub struct Register {
    square1: Square1,
    square2: Square2,
    wave: Wave,
    noise: Noise,
    wavetable: WaveTable,
}
