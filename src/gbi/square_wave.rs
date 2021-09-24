use crate::gbi::{
    sound_processing::{frequency_from_note_number, pulse},
    types::{i4, AudioProcessor, Oscillator},
};

pub enum SquareWaveDuty {
    Ratio12_5,
    Ratio25,
    Ratio50,
}

impl SquareWaveDuty {
    fn to_ratio(&self) -> f64 {
        match self {
            SquareWaveDuty::Ratio12_5 => 0.125,
            SquareWaveDuty::Ratio25 => 0.25,
            SquareWaveDuty::Ratio50 => 0.5,
        }
    }
}

pub struct SquareWaveOscillator {
    phase: f64,
    pub freq: f64,
    pub duty: SquareWaveDuty,
}

impl SquareWaveOscillator {
    pub fn new() -> Self {
        SquareWaveOscillator {
            phase: 0.0,
            freq: 440.0,
            duty: SquareWaveDuty::Ratio50,
        }
    }

    pub fn set_duty(&mut self, duty: SquareWaveDuty) {
        self.duty = duty;
    }
}

impl AudioProcessor<i4> for SquareWaveOscillator {
    fn process(&mut self, sample_rate: f64) -> i4 {
        let phase_diff = (self.freq / sample_rate);
        let v = pulse(self.phase, self.duty.to_ratio());

        self.phase += phase_diff;
        v
    }
}

const NOTE_NUMBER_OF_440_HZ: u16 = 69;

impl Oscillator for SquareWaveOscillator {
    /// https://steinbergmedia.github.io/vst3_doc/vstinterfaces/structSteinberg_1_1Vst_1_1NoteOnEvent.html の pitch の項目
    fn set_pitch(&mut self, note: i16) {
        self.freq = frequency_from_note_number(note as u16, NOTE_NUMBER_OF_440_HZ);
    }
}
